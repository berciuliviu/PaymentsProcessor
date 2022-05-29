use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Transaction {
    #[serde(rename = "type")]
    tx_type: TxType,
    client: u16,
    tx: u32,

    #[serde(default = "default_amount")]
    amount: f32,
}

// For 3 column rows that don't have amount
pub fn default_amount() -> f32 {
    0_f32
}

impl Transaction {
    pub fn get_tx_id(self) -> u32 {
        self.tx
    }

    pub fn get_client_id(self) -> u16 {
        self.client
    }

    pub fn get_amount(self) -> f32 {
        self.amount
    }

    pub fn get_tx_type(self) -> TxType {
        self.tx_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_transaction() {
        let transaction: Transaction = Transaction {
            tx_type: TxType::Deposit,
            client: 1,
            tx: 1,
            amount: 10.0456_f32,
        };

        assert_eq!(transaction.get_tx_id(), 1);
        assert_eq!(transaction.get_client_id(), 1);
        assert_eq!(transaction.get_tx_type(), TxType::Deposit);
        assert_eq!(transaction.get_amount(), 10.0456_f32);
    }
}
