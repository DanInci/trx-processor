mod model;
mod processor;

use std::env;
use std::process;

use model::error::ProcessorError;
use crate::processor::TransactionProcessor;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), ProcessorError> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        return Err(ProcessorError::InvalidArguments(
            "Usage: cargo run -- <transactions.csv>".to_string(),
        ));
    }

    let input_file = &args[1];

    let mut processor = TransactionProcessor::new();
    processor.process_file(input_file)?;
    processor.output_accounts()?;

    Ok(())
}