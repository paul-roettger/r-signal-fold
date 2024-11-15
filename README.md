# Rust Signal Fold

This is a small test project to test Rust's capabilities in signal processing and generic programming.
The goal is to fold two signals mathematically and keep the datatypes for the time and the measured values generic. 
This operation can be used to simulate the output of a linear time-invariant system with a given input signal and the impuls response.

## Run from Source

```bash
git clone https://github.com/paul-roettger/rust-signal-fold.git
cd rust-signal-fold
cargo test
```