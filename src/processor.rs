use std::fs::File;
use std::sync::Arc;

use dashmap::DashMap;

use crate::logger::Logger;
use crate::model::account::Account;
use crate::model::error::ProcessorError;
use crate::model::transaction::{Transaction, TransactionInput, TransactionState, TransactionType};


pub struct TransactionProcessor {
    accounts: DashMap<u16, Account>,
    transactions: DashMap<u32, Transaction>,
    logger: Option<Arc<Logger>>,
}

impl TransactionProcessor {

    pub fn new() -> Self {
        TransactionProcessor {
            accounts: DashMap::new(),
            transactions: DashMap::new(),
            logger: None,
        }
    }

    pub fn with_logger(logger: Arc<Logger>) -> Self {
        TransactionProcessor {
            accounts: DashMap::new(),
            transactions: DashMap::new(),
            logger: Some(logger),
        }
    }

    fn log(&self, message: &str) {
        if let Some(ref logger) = self.logger {
            logger.log(message);
        }
    }

    pub fn process_file(&self, file_path: &str) -> Result<(), ProcessorError> {
        let file = File::open(file_path)?;
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(file);

        for result in reader.deserialize() {
            let record: TransactionInput = result?;
            self.process_transaction(record);
        }

        Ok(())
    }

    fn process_transaction(&self, record: TransactionInput) {
        // Get or create account to ensure ordering lock exists
        let ordering_lock = {
            let account = self.accounts
                .entry(record.client)
                .or_insert_with(|| Account::new(record.client));
            account.ordering_lock.clone()
        };

        // Lock only this client (other clients can process concurrently)
        let _guard = ordering_lock.lock();

        // Process transaction with guaranteed ordering for this client
        match record.transaction_type {
            TransactionType::Deposit => self.handle_deposit(record),
            TransactionType::Withdrawal => self.handle_withdrawal(record),
            TransactionType::Dispute => self.handle_dispute(record),
            TransactionType::Resolve => self.handle_resolve(record),
            TransactionType::Chargeback => self.handle_chargeback(record),
        }
    }

    fn handle_deposit(&self, record: TransactionInput) {
        // Deposits must have an amount
        let Some(amount) = record.amount else {
            self.log(&format!("DEPOSIT REJECTED: client={}, tx={}, reason=missing_amount", record.client, record.tx));
            return;
        };

        // Ignore if amount is negative or zero
        if amount <= rust_decimal::Decimal::ZERO {
            self.log(&format!("DEPOSIT REJECTED: client={}, tx={}, amount={}, reason=non_positive_amount", record.client, record.tx, amount));
            return;
        }

        // Deposits work if account is not locked
        // Note: only deposits are stored since they're the only disputable transactions
        let mut account = self.accounts
            .entry(record.client)
            .or_insert_with(|| Account::new(record.client));

        if account.deposit(amount) {
            let transaction = Transaction::new(
                record.tx,
                record.client,
                record.transaction_type,
                amount,
            );
            self.transactions.insert(transaction.tx_id, transaction);
            self.log(&format!("DEPOSIT SUCCESS: client={}, tx={}, amount={}", record.client, record.tx, amount));
        } else {
            self.log(&format!("DEPOSIT REJECTED: client={}, tx={}, amount={}, reason=account_locked", record.client, record.tx, amount));
        }
    }

    fn handle_withdrawal(&self, record: TransactionInput) {
        // Withdrawals must have an amount
        let Some(amount) = record.amount else {
            self.log(&format!("WITHDRAWAL REJECTED: client={}, tx={}, reason=missing_amount", record.client, record.tx));
            return;
        };

        // Ignore if amount is negative or zero
        if amount <= rust_decimal::Decimal::ZERO {
            self.log(&format!("WITHDRAWAL REJECTED: client={}, tx={}, amount={}, reason=non_positive_amount", record.client, record.tx, amount));
            return;
        }

        // Withdrawals work if funds are available and account is not locked
        // Note: Withdrawals are not stored since they cannot be disputed
        let mut account = self.accounts
            .entry(record.client)
            .or_insert_with(|| Account::new(record.client));

        if account.withdraw(amount) {
            self.log(&format!("WITHDRAWAL SUCCESS: client={}, tx={}, amount={}", record.client, record.tx, amount));
        } else {
            self.log(&format!("WITHDRAWAL REJECTED: client={}, tx={}, amount={}, reason=insufficient_funds_or_locked", record.client, record.tx, amount));
        }
    }

    fn handle_dispute(&self, record: TransactionInput) {
        // Referenced transaction must exist
        let Some(transaction) = self.transactions.get(&record.tx) else {
            self.log(&format!("DISPUTE REJECTED: client={}, tx={}, reason=transaction_not_found", record.client, record.tx));
            return;
        };

        // Verify the transaction belongs to the same client
        let tx_client_id = transaction.client_id;
        if tx_client_id != record.client {
            self.log(&format!("DISPUTE REJECTED: client={}, tx={}, reason=client_mismatch (tx_client={})", record.client, record.tx, tx_client_id));
            return;
        }

        // Only deposits can be disputed
        if transaction.transaction_type != TransactionType::Deposit {
            self.log(&format!("DISPUTE REJECTED: client={}, tx={}, reason=non_deposit_transaction", record.client, record.tx));
            return;
        }

        // Transaction must not already be disputed or charged back
        let tx_state = transaction.state.clone();
        if tx_state != TransactionState::Normal {
            self.log(&format!("DISPUTE REJECTED: client={}, tx={}, reason=invalid_state (state={:?})", record.client, record.tx, tx_state));
            return;
        }

        let tx_amount = transaction.amount;
        drop(transaction);

        // Get the account and hold the funds
        let mut account = match self.accounts.get_mut(&record.client) {
            Some(acc) => acc,
            None => {
                self.log(&format!("DISPUTE REJECTED: client={}, tx={}, reason=account_not_found", record.client, record.tx));
                return;
            }
        };

        // Mark transaction as under dispute
        if account.hold_funds(tx_amount) {
            self.transactions.get_mut(&record.tx).unwrap().state = TransactionState::UnderDispute;
            self.log(&format!("DISPUTE SUCCESS: client={}, tx={}, amount={} (moved to held)", record.client, record.tx, tx_amount));
        } else {
            self.log(&format!("DISPUTE REJECTED: client={}, tx={}, reason=insufficient_available_funds", record.client, record.tx));
        }
    }

    fn handle_resolve(&self, record: TransactionInput) {
        // Referenced transaction must exist
        let Some(transaction) = self.transactions.get(&record.tx) else {
            self.log(&format!("RESOLVE REJECTED: client={}, tx={}, reason=transaction_not_found", record.client, record.tx));
            return;
        };

        // Verify the transaction belongs to the same client
        let tx_client_id = transaction.client_id;
        if tx_client_id != record.client {
            self.log(&format!("RESOLVE REJECTED: client={}, tx={}, reason=client_mismatch (tx_client={})", record.client, record.tx, tx_client_id));
            return;
        }

        // Transaction must be under dispute
        let tx_state = transaction.state.clone();
        if tx_state != TransactionState::UnderDispute {
            self.log(&format!("RESOLVE REJECTED: client={}, tx={}, reason=not_under_dispute (state={:?})", record.client, record.tx, tx_state));
            return;
        }

        let tx_amount = transaction.amount;
        drop(transaction); // Release the read lock

        // Get the account and release the held funds
        let mut account = match self.accounts.get_mut(&record.client) {
            Some(acc) => acc,
            None => {
                self.log(&format!("RESOLVE REJECTED: client={}, tx={}, reason=account_not_found", record.client, record.tx));
                return;
            }
        };

        // Mark transaction as resolved (back to normal)
        if account.release_funds(tx_amount) {
            self.transactions.get_mut(&record.tx).unwrap().state = TransactionState::Normal;
            self.log(&format!("RESOLVE SUCCESS: client={}, tx={}, amount={} (moved to available)", record.client, record.tx, tx_amount));
        } else {
            self.log(&format!("RESOLVE REJECTED: client={}, tx={}, reason=insufficient_held_funds", record.client, record.tx));
        }
    }

    fn handle_chargeback(&self, record: TransactionInput) {
        // Referenced transaction must exist
        let Some(transaction) = self.transactions.get(&record.tx) else {
            self.log(&format!("CHARGEBACK REJECTED: client={}, tx={}, reason=transaction_not_found", record.client, record.tx));
            return;
        };

        // Verify the transaction belongs to the same client
        let tx_client_id = transaction.client_id;
        if tx_client_id != record.client {
            self.log(&format!("CHARGEBACK REJECTED: client={}, tx={}, reason=client_mismatch (tx_client={})", record.client, record.tx, tx_client_id));
            return;
        }

        // Transaction must be under dispute
        let tx_state = transaction.state.clone();
        if tx_state != TransactionState::UnderDispute {
            self.log(&format!("CHARGEBACK REJECTED: client={}, tx={}, reason=not_under_dispute (state={:?})", record.client, record.tx, tx_state));
            return;
        }

        let tx_amount = transaction.amount;
        drop(transaction); // Release the read lock

        // Get the account and perform chargeback
        let mut account = match self.accounts.get_mut(&record.client) {
            Some(acc) => acc,
            None => {
                self.log(&format!("CHARGEBACK REJECTED: client={}, tx={}, reason=account_not_found", record.client, record.tx));
                return;
            }
        };

        // Mark transaction as charged back and lock account
        if account.chargeback(tx_amount) {
            self.transactions.get_mut(&record.tx).unwrap().state = TransactionState::ChargedBack;
            self.log(&format!("CHARGEBACK SUCCESS: client={}, tx={}, amount={} (account locked)", record.client, record.tx, tx_amount));
        } else {
            self.log(&format!("CHARGEBACK REJECTED: client={}, tx={}, reason=insufficient_held_funds", record.client, record.tx));
        }
    }

    pub fn output_accounts(&self) -> Result<(), ProcessorError> {
        let mut writer = csv::Writer::from_writer(std::io::stdout());

        let mut accounts: Vec<_> = self.accounts
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        accounts.sort_by_key(|a| a.client_id);

        for account in accounts {
            writer.serialize(account.to_output())?;
        }

        writer.flush()?;
        Ok(())
    }
}

impl Default for TransactionProcessor {
    fn default() -> Self {
        Self::new()
    }
}