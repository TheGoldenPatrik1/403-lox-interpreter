# 403-lox-interpreter

This is a basic interpreter for the [Lox](https://craftinginterpreters.com/the-lox-language.html) programming language, implemented using [Rust](https://www.rust-lang.org/).

## Steps to Run

1. Install [Rustup](https://www.rust-lang.org/learn/get-started), which includes Rust and Cargo.
2. Execute `cargo run`.

## Testing

### Testing Plan

1. The `/tests` directory contains `.lox` files which our test harness executes.
2. For each file, the output of the `print` statements is written to a corresponding `.txt` file in the `/output/actual` directory.
3. The test harness then compares this output file to a corresponding `.txt` file in the `/output/expected` directory to ensure that the two files match exactly.

### Steps to Run Test Harness

1. Install [Rustup](https://www.rust-lang.org/learn/get-started), which includes Rust and Cargo, if you have not already done so.
2. Execute `cargo test`.