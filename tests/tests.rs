use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_process_sample_transactions_success() {
    Command::new(assert_cmd::cargo::cargo_bin!("trx_processor"))
        .arg("tests/fixtures/sample_transactions.csv")
        .assert()
        .success();
}

#[test]
fn test_missing_file_argument() {
    Command::new(assert_cmd::cargo::cargo_bin!("trx_processor"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn test_nonexistent_file() {
    Command::new(assert_cmd::cargo::cargo_bin!("trx_processor"))
        .arg("nonexistent.csv")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}
