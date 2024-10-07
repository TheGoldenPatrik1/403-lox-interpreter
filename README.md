# Lox Interpreter

This is a basic interpreter for the [Lox](https://craftinginterpreters.com/the-lox-language.html) programming language, implemented using [Rust](https://www.rust-lang.org/).

## The Team

- Malachi Crain (CS-403)
- Sawyer Kent (CS-503)

## Steps to Run

1. Install [Rustup](https://www.rust-lang.org/learn/get-started), which includes Rust and Cargo.
2. Execute `cargo run` for a REPL environment.
3. Alternately, execute `cargo run <input filepath>` to run a file of Lox code. See the `tests/` directory for some example Lox files.

## Testing

Tests are sourced directly from the [test folder](https://github.com/munificent/craftinginterpreters/tree/master/test) of the GitHub Repository for the [Crafting Interpreters](https://craftinginterpreters.com/index.html) textbook.

### Testing Plan

We have **214** tests, covering every aspect of the Lox programming language. They are divided into the following categories:
* **assignment** - 9
* **block** - 2
* **bool** - 2
* **call** - 5
* **class** - 7
* **comments** - 3
* **constructor** - 9
* **field** - 21
* **for** - 10
* **function** - 13
* **if** - 10
* **inheritance** - 7
* **logical_operator** - 4
* **method** - 9
* **misc** - 3
* **nil** - 1
* **number** - 5
* **operator** - 33
* **print** - 1
* **return** - 7
* **string** - 3
* **super** - 17
* **this** - 6
* **variable** - 21
* **while** - 6

### Sample Test Run

An example test run is provided in `sample_test_run.txt`. This file was automatically generated using `cargo test > sample_test_run.txt`.

### Test Harness

1. The `/tests` directory contains `.lox` files which our test harness executes.
2. For each file, the output of the `print` statements is written to a corresponding `.txt` file in the `/output/actual` directory.
3. The test harness then compares this output file to a corresponding `.txt` file in the `/output/expected` directory to assert that the two files match exactly. The `expected` file's contents are directly tied to the `// expect:` statements found in the Lox test files sourced from the textbook.
4. In the event that the `.lox` test file is intended to generate an error, the test harness will expect and gracefully handle an error. If no error is encountered, it will assert that the test failed.

### Steps to Run Test Harness

1. Install [Rustup](https://www.rust-lang.org/learn/get-started), which includes Rust and Cargo, if you have not already done so.
2. Execute `cargo test`.

### Steps to Extract Expected Test Values

1. Execute `cd scripts && python extract_tests.py` to generate the `/output/expected` files from the `/test` files.
2. Copy-paste the automatically generated test methods from `tests.rs` into the testing section towards the bottom of `src/main.rs`. Make sure to overrite the tests already there.
3. You are now ready to run `cargo test`.