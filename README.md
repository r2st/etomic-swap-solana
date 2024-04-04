# satomic-swap
Satomic Swap Program allowing Solana atomic swaps on Komodo SDK.

## Setup Dev environment
### Install rust
https://www.rust-lang.org/tools/install

### Install Solana CLI v1.18.8
https://docs.solanalabs.com/cli/install
```
sh -c "$(curl -sSfL https://release.solana.com/v1.18.8/install)"
```
### Build the project

To compile shared object (the program binary .so) run the command
```
cargo build-bpf
```

## Contribution
Before uploading any changes, please make sure that the test suite passes locally before submitting a pull request with your changes.
```
cargo test
```
Use Clippy to avoid common mistakes and rustfmt to make code clear.

Format the code using rustfmt:
```
cargo +nightly fmt
```
Make sure there are no warnings and errors. Run the Clippy:
```
cargo clippy --all-targets -- -D warnings
```