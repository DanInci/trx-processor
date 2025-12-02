# Transaction Processor

A generic transaction processor that takes a transaction events as csv input and outputs the state of accounts.

## Usage

### Default Usage

```bash
cargo run -- transactions.csv
```

Or with the compiled binary:

```bash
./target/release/trx_processor transactions.csv
```

### With Transaction Logging

Enable detailed logging of all operations (successes and rejections):

```bash
cargo run -- transactions.csv --log-transactions
```

This creates `transactions.log` in the current directory with timestamped entries.

## Input Format

CSV file with the following columns:

```csv
type, client, tx, amount
deposit, 1, 1, 100.0
withdrawal, 1, 2, 50.0
dispute, 1, 1,
resolve, 1, 1,
chargeback, 1, 1,
```

## Output Format

CSV output with the following columns to stdout:

```csv
client,available,held,total,locked
1,100.5000,50.0000,150.5000,false
2,200.0000,0.0000,200.0000,true
```

### Key Components

```
src/
├── main.rs              # CLI entry point
├── logger.rs            # Transaction logger
├── processor.rs         # Transaction processing logic
└── model/
    ├── account.rs       # Account types and state management
    ├── transaction.rs   # Transaction types and state management
    └── error.rs         # Error types and error handling
```

## Testing

### Run All Tests

```bash
cargo test
```

## Logging Format

When `--log-transactions` is enabled, logs are written to `transactions.log`:

```
[2025-12-01 23:21:38.168] DEPOSIT SUCCESS: client=1, tx=1, amount=100
[2025-12-01 23:21:38.168] WITHDRAWAL REJECTED: client=1, tx=2, amount=200, reason=insufficient_funds_or_locked
[2025-12-01 23:21:38.168] DISPUTE SUCCESS: client=1, tx=1, amount=100 (moved to held)
```

## Performance Characteristics

- **Time Complexity**: O(n) where n = number of transactions
- **Space Complexity**: O(c + d) where c = unique clients, d = deposits (plus small overhead for locks)
- **CSV Parsing**: Streaming
- **Concurrency**: Thread-safe and ready for concurrent processing