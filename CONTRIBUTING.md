# Contributing

RustCalc is a small reference application for FerriteDB. Keep changes focused on making the example clearer, smaller, or more correct.

Before submitting a change, run:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Public issues and pull requests should be written in English. Do not commit build output, local database files, WAL files, credentials, or editor-specific configuration.
