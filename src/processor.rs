use std::collections::HashMap;
use std::fs::File;

use crate::model::account::Account;
use crate::model::error::ProcessorError;
use crate::model::transaction::{Transaction, TransactionInput, TransactionState, TransactionType};


pub struct TransactionProcessor {
    accounts: HashMap<u16, Account>,
    transactions: HashMap<u32, Transaction>,
}

impl TransactionProcessor {

    pub fn new() -> Self {
        TransactionProcessor {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    pub fn process_file(&mut self, file_path: &str) -> Result<(), ProcessorError> {
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

    fn process_transaction(&mut self, record: TransactionInput) {
        match record.transaction_type {
            TransactionType::Deposit => self.handle_deposit(record),
            TransactionType::Withdrawal => self.handle_withdrawal(record),
            TransactionType::Dispute => self.handle_dispute(record),
            TransactionType::Resolve => self.handle_resolve(record),
            TransactionType::Chargeback => self.handle_chargeback(record),
        }
    }

    fn handle_deposit(&mut self, record: TransactionInput) {
        // Deposits must have an amount
        let Some(amount) = record.amount else {
            return;
        };

        // Ignore if amount is negative or zero
        if amount <= rust_decimal::Decimal::ZERO {
            return;
        }

        // Deposits work if account is not locked
        let account = self.get_or_create_account(record.client);
        if account.deposit(amount) {
            let transaction = Transaction::new(
                record.tx,
                record.client,
                record.transaction_type,
                amount,
            );
            self.transactions.insert(transaction.tx_id, transaction);
        }
    }

    fn handle_withdrawal(&mut self, record: TransactionInput) {
        // Withdrawals must have an amount
        let Some(amount) = record.amount else {
            return;
        };

        // Ignore if amount is negative or zero
        if amount <= rust_decimal::Decimal::ZERO {
            return;
        }

        // Withdrawals work if funds are available and account is not locked
        let account = self.get_or_create_account(record.client);
        if account.withdraw(amount) {
            let transaction = Transaction::new(
                record.tx,
                record.client,
                record.transaction_type,
                amount,
            );
            self.transactions.insert(transaction.tx_id, transaction);
        }
    }

    fn handle_dispute(&mut self, record: TransactionInput) {
        // Referenced transaction must exist
        let Some(transaction) = self.transactions.get_mut(&record.tx) else {
            return;
        };

        // Verify the transaction belongs to the same client
        if transaction.client_id != record.client {
            return;
        }

        // Only deposits can be disputed
        if transaction.transaction_type != TransactionType::Deposit {
            return;
        }

        // Transaction must not already be disputed or charged back
        if transaction.state != TransactionState::Normal {
            return;
        }

        // Get the account and hold the funds
        let Some(account) = self.accounts.get_mut(&record.client) else {
            return;
        };

        // Mark transaction as under dispute
        if account.hold_funds(transaction.amount) {
            transaction.state = TransactionState::UnderDispute;
        }
    }

    fn handle_resolve(&mut self, record: TransactionInput) {
        // Referenced transaction must exist
        let Some(transaction) = self.transactions.get_mut(&record.tx) else {
            return;
        };

        // Verify the transaction belongs to the same client
        if transaction.client_id != record.client {
            return;
        }

        // Transaction must be under dispute
        if transaction.state != TransactionState::UnderDispute {
            return;
        }

        // Get the account and release the held funds
        let Some(account) = self.accounts.get_mut(&record.client) else {
            return;
        };

        // Mark transaction as resolved (back to normal)
        if account.release_funds(transaction.amount) {
            transaction.state = TransactionState::Normal;
        }
    }

    fn handle_chargeback(&mut self, record: TransactionInput) {
        // Referenced transaction must exist
        let Some(transaction) = self.transactions.get_mut(&record.tx) else {
            return;
        };

        // Verify the transaction belongs to the same client
        if transaction.client_id != record.client {
            return;
        }

        // Transaction must be under dispute
        if transaction.state != TransactionState::UnderDispute {
            return;
        }

        // Get the account and perform chargeback
        let Some(account) = self.accounts.get_mut(&record.client) else {
            return;
        };

        // Mark transaction as charged back and lock account
        if account.chargeback(transaction.amount) {
            transaction.state = TransactionState::ChargedBack;
        }
    }

    fn get_or_create_account(&mut self, client_id: u16) -> &mut Account {
        self.accounts
            .entry(client_id)
            .or_insert_with(|| Account::new(client_id))
    }

    pub fn output_accounts(&self) -> Result<(), ProcessorError> {
        let mut writer = csv::Writer::from_writer(std::io::stdout());

        let mut accounts: Vec<_> = self.accounts.values().collect();
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