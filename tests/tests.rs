use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

// ============================================================================
// Basic CLI Tests
// ============================================================================

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

#[test]
fn test_sample_transactions() {
    Command::new(assert_cmd::cargo::cargo_bin!("trx_processor"))
        .arg("tests/fixtures/sample_transactions.csv")
        .assert()
        .success();
}


// ============================================================================
// Basic Transaction Flow Tests
// ============================================================================

#[test]
fn test_basic_deposits_and_withdrawals() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!("trx_processor"))
        .arg("tests/fixtures/basic_deposits_withdrawals.csv")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();

    // Client 1: 100 + 200 - 50 - 250 = 0
    assert!(output_str.contains("1,0,0,0,false"));

    // Client 2: 1000 - 250 = 750
    assert!(output_str.contains("2,750,0,750,false"));
}

#[test]
fn test_insufficient_funds() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!("trx_processor"))
        .arg("tests/fixtures/insufficient_funds.csv")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();

    // Client 1: deposit 50, withdrawal 100 (fails), withdrawal 25, deposit 30, withdrawal 60 (fails)
    // Result: 50 - 25 + 30 = 55
    assert!(output_str.contains("1,55,0,55,false"));
}

// ============================================================================
// Dispute Flow Tests
// ============================================================================

#[test]
fn test_dispute_and_resolve() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!("trx_processor"))
        .arg("tests/fixtures/dispute_and_resolve.csv")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();

    // Client 1: 100 + 50 = 150, dispute 100 (held), withdrawal 25, resolve (available), withdrawal 100
    // Result: 150 - 25 - 100 = 25
    assert!(output_str.contains("1,25,0,25,false"));
}

#[test]
fn test_dispute_and_chargeback() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!("trx_processor"))
        .arg("tests/fixtures/dispute_and_chargeback.csv")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();

    // Client 1: 100 + 50 = 150, dispute 100, chargeback (removes 100, locks account)
    // Deposit 100 fails (locked), withdrawal 25 fails (locked)
    // Result: 50, locked
    assert!(output_str.contains("1,50,0,50,true"));
}

#[test]
fn test_multiple_disputes_same_transaction() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!("trx_processor"))
        .arg("tests/fixtures/multiple_disputes_same_tx.csv")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();

    // Client 1: deposit 100, dispute once (succeeds), dispute again (fails), dispute again (fails)
    // Result: 0 available, 100 held
    assert!(output_str.contains("1,0,100,100,false"));
}

#[test]
fn test_dispute_withdrawal_ignored() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!("trx_processor"))
        .arg("tests/fixtures/dispute_without_deposit.csv")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();

    // Client 1: deposit 100, withdrawal 50, dispute withdrawal (ignored), dispute non-existent (ignored)
    // Result: 50
    assert!(output_str.contains("1,50,0,50,false"));
}

#[test]
fn test_resolve_without_dispute() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!("trx_processor"))
        .arg("tests/fixtures/resolve_without_dispute.csv")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();

    // Client 1: deposit 100, resolve (fails - not disputed), dispute (succeeds), resolve (succeeds), resolve again (fails)
    // Result: 100 available
    assert!(output_str.contains("1,100,0,100,false"));
}

#[test]
fn test_chargeback_without_dispute() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!("trx_processor"))
        .arg("tests/fixtures/chargeback_without_dispute.csv")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();

    // Client 1: deposit 100, chargeback (fails - not disputed)
    // Result: 100 available, not locked
    assert!(output_str.contains("1,100,0,100,false"));
}

// ============================================================================
// Precision Tests
// ============================================================================

#[test]
fn test_decimal_precision() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!("trx_processor"))
        .arg("tests/fixtures/precision_test.csv")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();

    // Client 1: 1.1111 + 2.2222 + 3.3333 - 4.4444 = 2.2222
    // Should be rounded to 4 decimal places
    assert!(output_str.contains("1,2.2222,0,2.2222,false"));
}

// ============================================================================
// Multiple Client Tests
// ============================================================================

#[test]
fn test_client_mismatch() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!("trx_processor"))
        .arg("tests/fixtures/client_mismatch.csv")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();

    // Client 1: deposit 100, client 2 tries to dispute it (fails)
    assert!(output_str.contains("1,100,0,100,false"));

    // Client 2: deposit 200, client 1 tries to dispute it (fails)
    assert!(output_str.contains("2,200,0,200,false"));
}

#[test]
fn test_multiple_clients_independent() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!("trx_processor"))
        .arg("tests/fixtures/multiple_clients.csv")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();

    // Client 1: 100 - 50 = 50, dispute 100, resolve
    assert!(output_str.contains("1,50,0,50,false"));

    // Client 2: 200, dispute 200 (moves to held), chargeback (removes from held, locks)
    assert!(output_str.contains("2,0,0,0,true"));

    // Client 3: 300 - 150 = 150, no disputes
    assert!(output_str.contains("3,150,0,150,false"));
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_zero_and_negative_amounts() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!("trx_processor"))
        .arg("tests/fixtures/zero_and_negative.csv")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();

    // Client 1: deposit 0 (fails), deposit -10 (fails), deposit 100, withdrawal 0 (fails), withdrawal -5 (fails), withdrawal 50
    // Result: 100 - 50 = 50
    assert!(output_str.contains("1,50,0,50,false"));
}