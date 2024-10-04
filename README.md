# 403-lox-interpreter

This is a basic interpreter for the [Lox](https://craftinginterpreters.com/the-lox-language.html) programming language, implemented using [Rust](https://www.rust-lang.org/).

## The Team

- Malachi Crain
- Sawyer Kent

## Steps to Run

1. Install [Rustup](https://www.rust-lang.org/learn/get-started), which includes Rust and Cargo.
2. Execute `cargo run` for a REPL environment.
3. Alternately, execute `cargo run <input filepath>` to run a file of Lox code.

## Testing

Tests are sourced from the [test folder](https://github.com/munificent/craftinginterpreters/tree/master/test) of the GitHub Repository for the Crafting Interpreters textbook.

### Testing Plan

1. The `/tests` directory contains `.lox` files which our test harness executes.
2. For each file, the output of the `print` statements is written to a corresponding `.txt` file in the `/output/actual` directory.
3. The test harness then compares this output file to a corresponding `.txt` file in the `/output/expected` directory to assert that the two files match exactly.
4. In the event that the `.lox` test file is intended to generate an error, the test harness will expect and gracefully handle an error. If no error is encountered, it will assert that the test failed.

### Steps to Run Test Harness

1. Install [Rustup](https://www.rust-lang.org/learn/get-started), which includes Rust and Cargo, if you have not already done so.
2. Execute `cargo test`.

### Steps to Extract Expected Test Values

1. Execute `python scripts/extract_tests.py`.
2. Copy-paste the automatically generated test methods from `tests.rs` into the testing section at the bottom of `src/main.rs`.
3. You are now ready to run `cargo test`.