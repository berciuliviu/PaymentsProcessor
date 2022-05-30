use crate::client::Client;
use crate::transaction::{Transaction, TxType};
use crate::utils::*;
use std::collections::HashMap;
use std::error::Error;

pub struct Processor {
    filename: String,
    clients: HashMap<u16, Client>,
}

/*******************************
< Processor >

Processes received CSV files into Client accounts hashmap.

*******************************/
impl Processor {
    pub fn new(filename: String) -> Self {
        Self {
            filename: filename,
            clients: HashMap::new(),
        }
    }

    pub fn process_transactions(&mut self) {
        // Create Builder from file
        // - remove spaces
        // - allow different length rows
        let mut csv_reader: csv::Reader<std::fs::File> = create_csv_reader(&self.filename);

        // Deserialize each row, based on headers length
        for row in csv_reader.byte_records() {
            if let Ok(result) = row {
                let tx: Result<Transaction, csv::Error> = match result.len() {
                    4 => result.deserialize(Some(&FULL_HEADER)),
                    3 => result.deserialize(Some(&PARTIAL_HEADER)),
                    _ => {
                        eprintln!("Only rows with 3 or 4 fields are allowed.");
                        continue;
                    }
                };
                if let Err(error) = tx {
                    eprintln!("Deserialization error: {}.", error);
                    continue;
                }

                if let Err(error) = self.process_transaction(tx.unwrap()) {
                    eprintln!("{}", error);
                }
            }
        }
    }

    pub fn process_transaction(&mut self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        // We retrieve the client
        // If he doesn't exist, we create a new one
        let client_id: u16 = transaction.get_client_id();

        let client: &mut Client = if let Some(client) = self.clients.get_mut(&client_id) {
            client
        } else {
            self.clients.insert(client_id, Client::new(client_id));
            self.clients.get_mut(&client_id).unwrap()
        };

        match transaction.get_tx_type() {
            TxType::Deposit => {
                client.consume_deposit(transaction)?;
            }

            TxType::Withdrawal => {
                client.consume_withdrawal(transaction)?;
            }

            TxType::Dispute => client.consume_dispute(transaction)?,

            TxType::Resolve => client.consume_resolve(transaction)?,

            TxType::Chargeback => client.consume_chargeback(transaction)?,
        }

        Ok(())
    }

    pub fn print_clients(&self, header: bool) -> Result<(), Box<dyn Error>> {
        let mut writer = csv::Writer::from_writer(std::io::stdout());
        if header {
            writer.write_byte_record(&CSV_TOP_HEADER)?;
        }

        for (_, client) in self.clients.iter() {
            writer.write_byte_record(&client.record())?;
        }

        Ok(())
    }
}
