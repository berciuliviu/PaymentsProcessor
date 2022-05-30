# Payments Processor

This project represents a small payments processor written in [Rust](https://www.rust-lang.org/).

# Workflow

The workflow of the project is the following:

1. A CSV file is given to the processor.
2. The processor consumes every row from the CSV file, calculating client accounts, based on the rules written below.
3. The processor then returns a new CSV with rows representing client accounts.

# Processor rules

There are 5 kinds of transactions:

- **deposit** - add a set amount of money on client account
- **withdrawal** - withdraw a set amount of money on client account
- **dispute** - dispute a transaction
- **resolve** - resolve a disputed transaction
- **chargeback** - chargeback and resolve a disputed transaction

According to my research on different sources, we will consider the following rules as the source of truth for processing the above types of transactions.

1. Deposit transactions will be done even if the client account is locked.
2. Withdrawal transactions imply the amount is smaller than the client's available amount. Locked accounts cannot accept withdrawals.
3. Disputed transactions apply only to **DEPOSIT** and **WITHDRAWAL** transactions.
4. Disputing a DEPOSIT transaction implies substracting an amount X from available funds and adding it to held funds. Transaction will be marked as "disputed".
5. Disputing a WITHDRAWAL transaction implies marking the transaction as "disputed". We go with the premise that a third party stole the client's credit card and did a fraudulent withdrawal from an ATM. Nothing can be held as there isn't anything that can be held.
6. Resolving a DEPOSIT transaction implies substracting the amount held and adding it back to available funds. It means the transaction was legitimate and there is no need for a chargeback. Transaction is marked as `resolved`.
7. Resolving a WITHDRAWAL transaction implies doing nothing to the account. The transaction was not found to be malicious/done by someone else and the client is not refunded. The transaction is marked as `resolved`.
8. Charging back a DEPOSIT transaction implies substracting the amount held, as the transaction was considered fraudulent/unaothorized. The account is `locked`. Transaction is marked as `chargedback`.
9. Charging back a WITHDRAWAL transaction implies adding (crediting) the withdrawn amount into available funds. It means the transaction was not done by the client, but by a malicious party, and the client gets refunded. Transaction is marked as `chargedback`. The account is `locked` to prevent further malicious actions.
10. Transactions marked as "resolved/chargedback" can't be disputed again.
11. Transaction IDs are unique, but not in a set increasing order.
12. Available amount can be negative, hence the client being unable to withdraw until he covers the amount owned to the bank.

# High Level Technical Overview

- Executable accepts only one argument, namely the name of the CSV file.
- CSV serialization/deserialization is done using [Serde](https://serde.rs/) and [CSV](https://docs.rs/csv/latest/csv/) crates.
- Records are read and processed one by one in a single-threaded approach.
- All transactions have their own consumer function.

# Installation

To run the project, run the following commands:

```rust
cargo install
cargo run
```

# Multithreading

There is a multithreading solution using Tokio on branch `tokio_threading_impl`.

The following details were implemented:

1. Tokio is used as the crate providing async functionalities.
2. **Tokio::tasks** are used.
3. There is a new structure implementation, namely ProcessorAsync, which does the following:
4. Create a set number of channels and spawn a set number of tasks, each receiving its own channel to communicate with.
5. Hold task handles and channels in their own containers.
6. While parsing the CSV file, send a specific record to a specific task through its channel by using the formula `client_id % channels_length`.
7. After all records are processed, send a Close message to each task, signaling it to print to STDOUT the client accounts and then exit.
8. Wait for all tasks to finish.

# Tests

Unit tests and integration tests are available. In order to run integration tests, make sure you have `Python` installed and run the following command from the main folder:

```python
python tests/scripts/csv_generator.py
```

This script will generate a huge CSV file needed for a stress test.

All tests can be run by calling:

```rust
cargo test
```

# References

- [Disputes and how they work](https://stripe.com/docs/disputes#:~:text=To%20process%20a%20chargeback%2C%20the,deducted%20from%20your%20account%20balance.)
- [ATM Withdrawal Dispute](https://www.sapling.com/8377915/dispute-atm-withdrawal)
