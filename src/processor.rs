use std::collections::HashMap;
use std::fs::File;

use crate::model::account::Account;
use crate::model::error::ProcessorError;
use crate::model::transaction::{TransactionInput, TransactionType};


pub struct TransactionProcessor {
    accounts: HashMap<u16, Account>,
}

impl TransactionProcessor {

    pub fn new() -> Self {
        TransactionProcessor {
            accounts: HashMap::new(),
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
        // TODO: Implement deposit logic
    }

    fn handle_withdrawal(&mut self, record: TransactionInput) {
        // TODO: Implement withdrawal logic
    }

    fn handle_dispute(&mut self, record: TransactionInput) {
        // TODO: Implement dispute logic
    }

    fn handle_resolve(&mut self, record: TransactionInput) {
        // TODO: Implement resolve logic
    }

    fn handle_chargeback(&mut self, record: TransactionInput) {
        // TODO: Implement chargeback logic
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