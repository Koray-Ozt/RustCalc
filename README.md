# RustCalc

A deliberately small Rust desktop calculator that demonstrates how to embed [FerriteDB](https://github.com/Koray-Ozt/FerriteDB) in a real application.

> **Example project:** RustCalc is intended as a compact FerriteDB usage example, not as a feature-complete calculator. FerriteDB is currently an unaudited MVP with an unstable API and on-disk format; do not use it for production or irreplaceable data.

## What this example shows

- adding `ferrite-core` as a Git dependency;
- opening or creating a local FerriteDB database;
- writing JSON values with `put_key`;
- reading an ordered group of records with `list` and a key prefix;
- retaining calculator history across application restarts;
- keeping the database handle alive for the lifetime of a GTK desktop application.

The calculator supports addition, subtraction, multiplication, division, decimal input, and division-by-zero validation. Its three most recent calculations are displayed above the keypad.

## FerriteDB usage

FerriteDB is pinned to a known commit because its crates are not currently published to crates.io:

```toml
[dependencies]
ferrite-core = { git = "https://github.com/Koray-Ozt/FerriteDB.git", rev = "da67e8b6079493e915191efb82d8ed0538306f71" }
serde_json = "1"
```

### 1. Open the database

`Database::open` creates the database directory when necessary and recovers committed records from the write-ahead log when it already exists.

```rust
use ferrite_core::Database;

let db = Database::open("data/history.ferrite")?;
```

FerriteDB uses an exclusive writer lock, so the application keeps one database handle open and shares access to it through its own `HistoryStore`.

### 2. Store JSON data

Every completed calculation is stored as JSON under an ordered `history/` key:

```rust
use serde_json::json;

let key = "history/00000000000000000000";
db.put_key(key, json!({ "text": "8 ÷ 2 = 4" }))?;
```

A successful strict write is appended to FerriteDB's WAL and synchronized before returning.

### 3. Read the history

Prefix listing retrieves only calculator-history records:

```rust
let entries = db
    .list(Some("history/"))?
    .into_iter()
    .filter_map(|(_, value)| value.get("text")?.as_str().map(str::to_owned))
    .collect::<Vec<_>>();
```

The zero-padded keys preserve insertion order in FerriteDB's ordered key space. The complete integration is in [`src/lib.rs`](src/lib.rs), while [`tests/history.rs`](tests/history.rs) verifies persistence after closing and reopening the database.

## Requirements

- Rust and Cargo
- GTK 3 development libraries

On Ubuntu or Linux Mint:

```bash
sudo apt update
sudo apt install libgtk-3-dev
```

## Run

```bash
git clone https://github.com/Koray-Ozt/RustCalc.git
cd RustCalc
cargo run --release
```

By default, history is stored in `data/history.ferrite`. Override the location with:

```bash
RUST_CALC_DATA=/tmp/rust-calc-data cargo run --release
```

## Verify

```bash
cargo test --all-targets
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
```

The persistence test exercises a real FerriteDB database and confirms that a recorded calculation survives a restart.

## Project layout

```text
src/lib.rs          Calculator logic and FerriteDB-backed HistoryStore
src/main.rs         GTK desktop interface
tests/calculator.rs Calculator behavior tests
tests/history.rs    FerriteDB restart/persistence test
```

## Scope and limitations

- This example uses FerriteDB's Rust core directly rather than its sidecar protocol.
- History records are append-only and there is no history-management interface.
- The dependency is commit-pinned while FerriteDB's API remains unstable.
- FerriteDB and RustCalc currently provide no production-readiness guarantee.
- This repository currently has no license file; public visibility does not grant permission to copy, modify, or redistribute its contents.

For FerriteDB's architecture, CLI, sidecar protocol, TypeScript SDK, and current limitations, see the [FerriteDB repository](https://github.com/Koray-Ozt/FerriteDB) and its [getting-started guide](https://github.com/Koray-Ozt/FerriteDB/blob/main/docs/GETTING_STARTED.md).
