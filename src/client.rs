use crate::error::p_error;
use crate::transaction::{Transaction, TxType};
use std::collections::{HashMap, HashSet};
use std::error::Error;

#[derive(Debug, Clone)]
pub struct Client {
    id: u16,
    available_amount: f32,
    held_amount: f32,
    locked: bool,
    transactions: HashMap<u32, Transaction>,
    disputed_transactions: HashSet<u32>,
}

impl Client {
    pub fn new(client_id: u16) -> Self {
        Self {
            id: client_id,
            available_amount: 0_f32,
            held_amount: 0_f32,
            locked: false,
            transactions: HashMap::new(),
            disputed_transactions: HashSet::new(),
        }
    }

    // Getters
    pub fn get_id(&self) -> u16 {
        self.id
    }

    pub fn get_available_amount(&self) -> f32 {
        self.available_amount
    }

    pub fn get_held_amount(&self) -> f32 {
        self.held_amount
    }

    pub fn get_total_amount(&self) -> f32 {
        self.held_amount + self.available_amount
    }

    // Transaction helper functions
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions
            .insert(transaction.get_tx_id(), transaction);
    }

    pub fn get_transaction(&self, transaction_id: u32) -> Option<&Transaction> {
        self.transactions.get(&transaction_id)
    }

    // Disputed transactions helper functions
    pub fn add_disputed_transaction(&mut self, transaction_id: u32) -> bool {
        self.disputed_transactions.insert(transaction_id)
    }

    pub fn remove_disputed_transaction(&mut self, transaction_id: u32) -> bool {
        self.disputed_transactions.remove(&transaction_id)
    }

    pub fn check_disputed_transaction(&self, transaction_id: u32) -> bool {
        self.disputed_transactions.contains(&transaction_id)
    }

    // Amount helper functions
    pub fn increase_available_amount(&mut self, amount: f32) {
        self.available_amount += amount;
    }

    pub fn decrease_available_amount(&mut self, amount: f32) {
        self.available_amount -= amount;
    }

    pub fn increase_held_amount(&mut self, amount: f32) {
        self.held_amount += amount;
    }

    pub fn decrease_held_amount(&mut self, amount: f32) {
        self.held_amount -= amount;
    }

    // Lock helper
    pub fn lock_account(&mut self, lock: bool) {
        self.locked = lock;
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    // Transaction consumers
    pub fn consume_deposit(&mut self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        self.increase_available_amount(transaction.get_amount());
        Ok(())
    }

    pub fn consume_withdrawal(&mut self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        if self.is_locked() {
            return p_error(format!("Locked accounts cannot accept withdrawals."));
        }
        let tx_amount: f32 = transaction.get_amount();
        if self.get_available_amount() < tx_amount {
            return p_error(format!(
                "Invalid withdrawal transaction. Available amount is smaller than withdraw amount."
            ));
        }

        self.decrease_available_amount(tx_amount);

        Ok(())
    }
    pub fn consume_dispute(&mut self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        let tx_id: u32 = transaction.get_tx_id();

        if let false = self.check_disputed_transaction(tx_id) {
            if let Some(tx) = self.get_transaction(tx_id) {
                if TxType::Deposit != tx.get_tx_type() {
                    return p_error(format!("Only DEPOSIT transactions can be disputed."));
                }
                let disputed_amount: f32 = tx.get_amount();
                self.held_amount += disputed_amount;
                self.available_amount -= disputed_amount;
                self.disputed_transactions.insert(tx_id);
            } else {
                return p_error(format!(
                    "Transaction {} isn't registered for client {}.",
                    tx_id, self.id
                ));
            }
        } else {
            return p_error(format!("Transaction {} is already disputed.", tx_id));
        }

        Ok(())
    }

    pub fn consume_resolve(&mut self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        let tx_id = transaction.get_tx_id();

        if let true = self.check_disputed_transaction(tx_id) {
            if let Some(tx) = self.get_transaction(tx_id) {
                if TxType::Deposit != tx.get_tx_type() {
                    return p_error(format!("Only DEPOSIT transactions can be resolved."));
                }
                let disputed_amount = tx.get_amount();
                self.held_amount -= disputed_amount;
                self.available_amount += disputed_amount;
                self.disputed_transactions.remove(&tx_id);
            } else {
                return p_error(format!(
                    "Transaction {} isn't registered for client {}.",
                    tx_id, self.id
                ));
            }
        } else {
            return p_error(format!("Transaction {} is not disputed.", tx_id));
        }

        Ok(())
    }

    pub fn consume_chargeback(&mut self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        let tx_id = transaction.get_tx_id();

        if let true = self.check_disputed_transaction(tx_id) {
            if let Some(tx) = self.get_transaction(tx_id) {
                if TxType::Deposit != tx.get_tx_type() {
                    return p_error(format!("Only DEPOSIT transactions can be chargedback."));
                }
                let disputed_amount = tx.get_amount();
                self.held_amount -= disputed_amount;
                self.disputed_transactions.remove(&tx_id);
                self.lock_account(true);
            } else {
                return p_error(format!(
                    "Transaction {} isn't registered for client {}.",
                    tx_id, self.id
                ));
            }
        } else {
            return p_error(format!("Transaction {} is not disputed.", tx_id));
        }

        Ok(())
    }

    // Client CSV string
    pub fn record(&self) -> Vec<String> {
        vec![
            format!("{:15}", self.id),
            format!("{:15.4}", self.get_available_amount()),
            format!("{:15.4}", self.get_held_amount()),
            format!("{:15.4}", self.get_total_amount()),
            format!("{:15}", self.locked.to_string()),
        ]
    }
}
