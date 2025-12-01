mod logger;
mod model;
mod processor;

use std::env;
use std::process;
use std::sync::Arc;

use logger::Logger;
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

    if args.len() < 2 || args.len() > 3 {
        return Err(ProcessorError::InvalidArguments(
            "Usage: cargo run -- <transactions.csv> [--log-transactions]".to_string(),
        ));
    }

    let input_file = &args[1];
    let enable_logging = args.len() == 3 && args[2] == "--log-transactions";

    // Create logger for corner case tracking (append-only) if flag is set
    let logger = if enable_logging {
        Logger::new("transactions.log")
            .map(Arc::new)
            .ok()
    } else {
        None
    };

    let mut processor = if let Some(logger) = logger {
        TransactionProcessor::with_logger(logger)
    } else {
        TransactionProcessor::new()
    };

    processor.process_file(input_file)?;
    processor.output_accounts()?;

    Ok(())
}