use crate::processor::Processor;
use crate::transaction::Transaction;
use crate::utils::*;
use std::error::Error;
use tokio;
use tokio::sync::mpsc;

pub struct ProcessorAsync {
    filename: String,
}

enum ChannelTx {
    Tx(Transaction),
    CloseChannel,
}

/*******************************
< Async Processor >

Processes received CSV file by using Tokio tasks.

Each task creates it's own processors and stores results into
its Client accounts hashmap.

*******************************/
impl ProcessorAsync {
    pub fn new(filename: String) -> Self {
        Self { filename: filename }
    }

    pub async fn process_transactions_async(&mut self) {
        // Create Builder from file
        // - remove spaces
        // - allow different length rows
        let mut csv_reader: csv::Reader<std::fs::File> = create_csv_reader(&self.filename);

        // Create tasks, tasks array and channels
        let mut tasks: Vec<tokio::task::JoinHandle<()>> = std::vec::Vec::new();
        let mut channels: Vec<mpsc::Sender<ChannelTx>> = std::vec::Vec::new();
        for _ in 0..TASKS_COUNT {
            let (tx, mut rx): (mpsc::Sender<ChannelTx>, mpsc::Receiver<ChannelTx>) =
                mpsc::channel(1000);
            channels.push(tx);

            // Create task
            tasks.push(tokio::task::spawn(async move {
                let mut proc = Processor::new("None".to_string());

                while let Some(result) = rx.recv().await {
                    match result {
                        ChannelTx::Tx(tx) => {
                            if let Err(error) = proc.process_transaction(tx) {
                                eprintln!("{}", error);
                            }
                        }
                        ChannelTx::CloseChannel => break,
                    }
                }
                if let Err(error) = proc.print_clients(false) {
                    eprintln!("{}", error);
                };
            }));
        }

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
                let tx = tx.unwrap();

                let ch_idx: usize = tx.get_client_id() as usize % tasks.len() as usize;

                channels[ch_idx]
                    .send(ChannelTx::Tx(tx))
                    .await
                    .unwrap_or_else(|error| {
                        eprintln!("Issues with channel {}: {}.", ch_idx, error);
                        channels.remove(ch_idx);
                    });
            }
        }
        // Print header
        self.print_header().unwrap();

        // Send close message to tasks
        for (ch_id, channel) in channels.iter().enumerate() {
            channel
                .send(ChannelTx::CloseChannel)
                .await
                .unwrap_or_else(|error| {
                    eprintln!("Issues with channel {}: {}.", ch_id, error);
                });
        }

        // Wait for tasks to finish
        for handle in tasks {
            handle.await.unwrap();
        }
    }

    pub fn print_header(&self) -> Result<(), Box<dyn Error>> {
        let mut writer = csv::Writer::from_writer(std::io::stdout());

        writer.write_byte_record(&CSV_TOP_HEADER)?;

        Ok(())
    }
}
