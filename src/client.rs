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
    // Transactions that went through dispute -> resolve are
    // put here, so they can't be re-disputed and re-resolved/re-chargedback again
    resolved_transactions: HashSet<u32>,
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
            resolved_transactions: HashSet::new(),
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

    pub fn check_resolved_transaction(&self, transaction_id: u32) -> bool {
        self.resolved_transactions.contains(&transaction_id)
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
        if transaction.get_tx_type() != TxType::Deposit {
            return p_error(format!(
                "Deposit consumer accepts only DEPOSIT type transactions."
            ));
        }
        let tx_id: u32 = transaction.get_tx_id();
        let amount: f32 = transaction.get_amount();

        // Transaction amount has to be bigger than 0
        if amount <= 0_f32 {
            return p_error(format!(
                "Transaction with ID: {} cannot have negative or 0 amount.",
                transaction.get_tx_id()
            ));
        }
        // Transcation ID has to be unique
        if self.transactions.contains_key(&tx_id) {
            return p_error(format!(
                "Transaction with ID: {} already exists.",
                transaction.get_tx_id()
            ));
        }

        self.increase_available_amount(transaction.get_amount());
        self.add_transaction(transaction);

        Ok(())
    }

    pub fn consume_withdrawal(&mut self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        if transaction.get_tx_type() != TxType::Withdrawal {
            return p_error(format!(
                "Withdrawal consumer accepts only WITHDRAWAL type transactions."
            ));
        }
        let tx_id: u32 = transaction.get_tx_id();
        let amount: f32 = transaction.get_amount();

        // Transaction amount has to be bigger than 0
        if amount <= 0_f32 {
            return p_error(format!(
                "Transaction with ID: {} cannot have negative or 0 amount.",
                transaction.get_tx_id()
            ));
        }
        // Transaction ID should be unique
        if self.transactions.contains_key(&tx_id) {
            return p_error(format!(
                "Transaction with ID: {} already exists.",
                transaction.get_tx_id()
            ));
        }

        // Locked accounts do not accept withdrawals
        if self.is_locked() {
            return p_error(format!("Locked accounts cannot accept withdrawals."));
        }

        // Tx amount has to be bigger than available amount
        if self.get_available_amount() < amount {
            return p_error(format!(
                "Invalid withdrawal transaction {}. Available amount is smaller than withdraw amount.", tx_id
            ));
        }

        self.decrease_available_amount(amount);
        self.add_transaction(transaction);

        Ok(())
    }

    pub fn consume_dispute(&mut self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        if transaction.get_tx_type() != TxType::Dispute {
            return p_error(format!(
                "Dispute consumer accepts only DISPUTE type transactions."
            ));
        }
        let tx_id: u32 = transaction.get_tx_id();

        // Transaction can't be already disputed or resolved
        if self.check_disputed_transaction(tx_id) == false
            && self.check_resolved_transaction(tx_id) == false
        {
            if let Some(tx) = self.get_transaction(tx_id) {
                match tx.get_tx_type() {
                    TxType::Deposit => {
                        let disputed_amount: f32 = tx.get_amount();
                        self.held_amount += disputed_amount;
                        self.available_amount -= disputed_amount;
                        self.disputed_transactions.insert(tx_id);
                    }
                    TxType::Withdrawal => {
                        self.disputed_transactions.insert(tx_id);
                    }
                    _ => {
                        return p_error(format!(
                            "Only DEPOSIT and WITHDRAWAL transactions can be disputed."
                        ))
                    }
                }
            } else {
                return p_error(format!(
                    "Transaction {} isn't registered for client {}.",
                    tx_id, self.id
                ));
            }
        } else {
            return p_error(format!(
                "Transaction {} is already disputed/resolved.",
                tx_id
            ));
        }

        Ok(())
    }

    pub fn consume_resolve(&mut self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        if transaction.get_tx_type() != TxType::Resolve {
            return p_error(format!(
                "Resolve consumer accepts only RESOLVE type transactions."
            ));
        }
        let tx_id: u32 = transaction.get_tx_id();

        // Transaction has to be disputed in order to be resolved
        if let true = self.check_disputed_transaction(tx_id) {
            if let Some(tx) = self.get_transaction(tx_id) {
                match tx.get_tx_type() {
                    TxType::Deposit => {
                        let disputed_amount = tx.get_amount();
                        self.held_amount -= disputed_amount;
                        self.available_amount += disputed_amount;
                        self.disputed_transactions.remove(&tx_id);
                        self.resolved_transactions.insert(tx_id);
                    }
                    TxType::Withdrawal => {
                        self.disputed_transactions.remove(&tx_id);
                        self.resolved_transactions.insert(tx_id);
                    }
                    _ => {
                        return p_error(format!(
                            "Only DEPOSIT and WITHDRAWAL transactions can be resolved."
                        ))
                    }
                }
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
        if transaction.get_tx_type() != TxType::Chargeback {
            return p_error(format!(
                "Chargeback consumer accepts only CHARGEBACK type transactions."
            ));
        }
        let tx_id: u32 = transaction.get_tx_id();

        // Transaction has to be disputed in order to be charged back
        if let true = self.check_disputed_transaction(tx_id) {
            if let Some(tx) = self.get_transaction(tx_id) {
                match tx.get_tx_type() {
                    TxType::Deposit => {
                        let disputed_amount = tx.get_amount();
                        self.held_amount -= disputed_amount;
                        self.disputed_transactions.remove(&tx_id);
                        self.lock_account(true);
                        self.resolved_transactions.insert(tx_id);
                    }
                    // Chargebacks for withdrawals mean adding the amount
                    // back to the client account, then locking the account
                    // to prevent further malicious actions. More details
                    // in the README.md
                    TxType::Withdrawal => {
                        let disputed_amount = tx.get_amount();
                        self.available_amount += disputed_amount;
                        self.disputed_transactions.remove(&tx_id);
                        self.lock_account(true);
                        self.resolved_transactions.insert(tx_id);
                    }
                    _ => {
                        return p_error(format!(
                            "Only DEPOSIT and WITHDRAWAL transactions can be chargedback."
                        ))
                    }
                }
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

    // Client CSV record
    pub fn record(&self) -> csv::ByteRecord {
        csv::ByteRecord::from(vec![
            format!("{}", self.id),
            format!("{:.4}", self.get_available_amount()),
            format!("{:.4}", self.get_held_amount()),
            format!("{:.4}", self.get_total_amount()),
            format!("{}", self.locked.to_string()),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client() {
        let client: Client = Client::new(1);

        assert_eq!(client.get_id(), 1);
        assert_eq!(client.get_available_amount(), 0_f32);
        assert_eq!(client.get_held_amount(), 0_f32);
        assert_eq!(client.get_total_amount(), 0_f32);
    }

    #[test]
    fn test_client_amount_operations() {
        let mut client: Client = Client::new(1);

        client.increase_available_amount(4_f32);
        assert_eq!(client.get_available_amount(), 4_f32);

        client.increase_held_amount(5_f32);
        assert_eq!(client.get_held_amount(), 5_f32);

        assert_eq!(client.get_total_amount(), 9_f32);

        client.decrease_held_amount(2_f32);
        client.decrease_available_amount(3_f32);
        assert_eq!(client.get_held_amount(), 3_f32);
        assert_eq!(client.get_available_amount(), 1_f32);
        assert_eq!(client.get_total_amount(), 4_f32);
    }

    #[test]
    fn test_client_tx_operations() {
        let mut client: Client = Client::new(1);
        let mut deposit_transaction: Transaction = Transaction {
            tx_type: TxType::Deposit,
            tx: 1,
            amount: 2_f32,
            client: 1,
        };

        // Verify successful deposit transaction
        assert_eq!((), client.consume_deposit(deposit_transaction).unwrap());
        assert_eq!(2_f32, client.get_available_amount());
        assert_eq!(deposit_transaction, *client.get_transaction(1).unwrap());
        assert_eq!(2_f32, client.get_total_amount());

        let withdraw_transaction: Transaction = Transaction {
            tx_type: TxType::Withdrawal,
            tx: 2,
            amount: 1_f32,
            client: 1,
        };

        // Verify successful withdrawal transaction
        assert_eq!((), client.consume_withdrawal(withdraw_transaction).unwrap());
        assert_eq!(1_f32, client.get_available_amount());
        assert_eq!(withdraw_transaction, *client.get_transaction(2).unwrap());
        assert_eq!(1_f32, client.get_total_amount());

        // Add more deposit transactions, so we can dispute/resolve/chargeback
        deposit_transaction.tx += 2;
        deposit_transaction.amount += 4_f32;
        assert_eq!((), client.consume_deposit(deposit_transaction).unwrap());
        assert_eq!(deposit_transaction, *client.get_transaction(3).unwrap());

        deposit_transaction.tx += 1;
        assert_eq!((), client.consume_deposit(deposit_transaction).unwrap());
        assert_eq!(deposit_transaction, *client.get_transaction(4).unwrap());

        // Dispute DEPOSIT transaction
        let mut dispute_transaction: Transaction = Transaction {
            tx_type: TxType::Dispute,
            tx: 4,
            amount: 0_f32,
            client: 1,
        };
        assert_eq!((), client.consume_dispute(dispute_transaction).unwrap());
        assert_eq!(true, client.check_disputed_transaction(4));
        assert_eq!(6_f32, client.get_held_amount());
        assert_eq!(7_f32, client.get_available_amount());

        // Resolve DEPOSIT transaction
        let resolve_transaction: Transaction = Transaction {
            tx_type: TxType::Resolve,
            tx: 4,
            amount: 0_f32,
            client: 1,
        };

        assert_eq!((), client.consume_resolve(resolve_transaction).unwrap());
        assert_eq!(true, client.check_resolved_transaction(4));
        assert_eq!(false, client.check_disputed_transaction(4));
        assert_eq!(13_f32, client.get_available_amount());
        assert_eq!(0_f32, client.get_held_amount());

        if let Ok(()) = client.consume_dispute(dispute_transaction) {
            panic!("Cannot dispute transaction already resolved.")
        }

        // Dispute another DEPOSIT transaction and do a successful chargeback
        dispute_transaction.tx = 3;
        assert_eq!((), client.consume_dispute(dispute_transaction).unwrap());
        assert_eq!(true, client.check_disputed_transaction(3));
        assert_eq!(6_f32, client.get_held_amount());
        assert_eq!(7_f32, client.get_available_amount());

        let chargeback_transaction: Transaction = Transaction {
            tx_type: TxType::Chargeback,
            tx: 3,
            amount: 0_f32,
            client: 1,
        };

        assert_eq!(
            (),
            client.consume_chargeback(chargeback_transaction).unwrap()
        );
        assert_eq!(false, client.check_disputed_transaction(3));
        assert_eq!(true, client.check_resolved_transaction(3));
        assert_eq!(0_f32, client.get_held_amount());
        assert_eq!(7_f32, client.get_available_amount());
        assert_eq!(true, client.is_locked());
    }

    #[test]
    fn test_client_tx_withdrawal() {
        let mut client: Client = Client::new(1);
        client.increase_available_amount(10_f32);
        let mut withdraw_transaction: Transaction = Transaction {
            tx_type: TxType::Withdrawal,
            tx: 1,
            amount: 2_f32,
            client: 1,
        };

        // Verify first successful withdrawal transaction
        assert_eq!((), client.consume_withdrawal(withdraw_transaction).unwrap());
        assert_eq!(8_f32, client.get_available_amount());
        assert_eq!(withdraw_transaction, *client.get_transaction(1).unwrap());
        assert_eq!(8_f32, client.get_total_amount());

        // Verify second successful withdrawal transaction
        withdraw_transaction.tx += 1;
        assert_eq!((), client.consume_withdrawal(withdraw_transaction).unwrap());
        assert_eq!(6_f32, client.get_available_amount());
        assert_eq!(withdraw_transaction, *client.get_transaction(2).unwrap());
        assert_eq!(6_f32, client.get_total_amount());

        // Dispute both transactions
        let mut dispute_transaction: Transaction = Transaction {
            tx_type: TxType::Dispute,
            tx: 1,
            amount: 0_f32,
            client: 1,
        };

        assert_eq!((), client.consume_dispute(dispute_transaction).unwrap());
        assert_eq!(true, client.check_disputed_transaction(1));
        assert_eq!(0_f32, client.get_held_amount());
        assert_eq!(6_f32, client.get_available_amount());

        dispute_transaction.tx += 1;
        assert_eq!((), client.consume_dispute(dispute_transaction).unwrap());
        assert_eq!(true, client.check_disputed_transaction(2));
        assert_eq!(0_f32, client.get_held_amount());
        assert_eq!(6_f32, client.get_available_amount());

        // Resolve first transaction
        let resolve_transaction: Transaction = Transaction {
            tx_type: TxType::Resolve,
            tx: 1,
            amount: 0_f32,
            client: 1,
        };

        assert_eq!((), client.consume_resolve(resolve_transaction).unwrap());
        assert_eq!(true, client.check_resolved_transaction(1));
        assert_eq!(false, client.check_disputed_transaction(1));
        assert_eq!(6_f32, client.get_available_amount());

        // Chargeback second transaction
        let chargeback_transaction: Transaction = Transaction {
            tx_type: TxType::Chargeback,
            tx: 2,
            amount: 0_f32,
            client: 1,
        };

        assert_eq!(
            (),
            client.consume_chargeback(chargeback_transaction).unwrap()
        );
        assert_eq!(true, client.check_resolved_transaction(2));
        assert_eq!(false, client.check_disputed_transaction(2));
        assert_eq!(8_f32, client.get_available_amount());
        assert_eq!(true, client.is_locked());
    }

    #[test]
    fn test_tx_errors() {
        let mut client: Client = Client::new(1);
        let mut deposit_transaction: Transaction = Transaction {
            tx_type: TxType::Deposit,
            tx: 1,
            amount: 20_f32,
            client: 1,
        };
        // Add two transactions
        assert_eq!((), client.consume_deposit(deposit_transaction).unwrap());
        deposit_transaction.tx += 1;
        assert_eq!((), client.consume_deposit(deposit_transaction).unwrap());

        // Try to withdraw more than available
        let withdrawal_transaction: Transaction = Transaction {
            tx_type: TxType::Withdrawal,
            tx: 3,
            amount: 50_f32,
            client: 1,
        };
        assert_eq!(
            "PROCESSOR ERROR: Invalid withdrawal transaction 3. Available amount is smaller than withdraw amount.",
            client
                .consume_withdrawal(withdrawal_transaction)
                .unwrap_err()
                .to_string()
        );

        // Try to process transaction with same id
        assert_eq!(
            "PROCESSOR ERROR: Transaction with ID: 2 already exists.",
            client
                .consume_deposit(deposit_transaction)
                .unwrap_err()
                .to_string()
        );

        let mut dispute_transaction: Transaction = Transaction {
            tx_type: TxType::Dispute,
            tx: 1,
            amount: 0_f32,
            client: 1,
        };

        // Dispute first transaction
        assert_eq!((), client.consume_dispute(dispute_transaction).unwrap());

        // Double dispute the second transaction
        dispute_transaction.tx = 2;
        assert_eq!((), client.consume_dispute(dispute_transaction).unwrap());
        assert_eq!(
            "PROCESSOR ERROR: Transaction 2 is already disputed/resolved.",
            client
                .consume_dispute(dispute_transaction)
                .unwrap_err()
                .to_string()
        );

        // Resolve second transaction and then try to resolve it again
        let resolve_transaction: Transaction = Transaction {
            tx_type: TxType::Resolve,
            tx: 2,
            amount: 0_f32,
            client: 1,
        };
        assert_eq!((), client.consume_resolve(resolve_transaction).unwrap());
        assert_eq!(
            "PROCESSOR ERROR: Transaction 2 is not disputed.",
            client
                .consume_resolve(resolve_transaction)
                .unwrap_err()
                .to_string()
        );

        // Chargeback first transaction and then try to chargeback again
        let chargeback_transaction: Transaction = Transaction {
            tx_type: TxType::Chargeback,
            tx: 1,
            amount: 0_f32,
            client: 1,
        };
        assert_eq!(
            (),
            client.consume_chargeback(chargeback_transaction).unwrap()
        );
        assert_eq!(
            "PROCESSOR ERROR: Transaction 1 is not disputed.",
            client
                .consume_chargeback(chargeback_transaction)
                .unwrap_err()
                .to_string()
        );
    }
}
