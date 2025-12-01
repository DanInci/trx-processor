use std::fmt;

#[derive(Debug)]
pub enum ProcessorError {
    InvalidArguments(String),
    IoError(std::io::Error),
    CsvError(csv::Error),
    TransactionError(String),
}

impl fmt::Display for ProcessorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessorError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            ProcessorError::IoError(err) => write!(f, "I/O error: {}", err),
            ProcessorError::CsvError(err) => write!(f, "CSV error: {}", err),
            ProcessorError::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
        }
    }
}

impl std::error::Error for ProcessorError {}

impl From<std::io::Error> for ProcessorError {
    fn from(err: std::io::Error) -> Self {
        ProcessorError::IoError(err)
    }
}

impl From<csv::Error> for ProcessorError {
    fn from(err: csv::Error) -> Self {
        ProcessorError::CsvError(err)
    }
}