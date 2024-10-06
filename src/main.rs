use std::cell::Cell;
use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::rc::Rc;

mod callable;
mod environment;
mod expr;
mod interpreter;
mod lox_class;
mod lox_function;
mod lox_instance;
mod native_functions;
mod parser;
mod resolver;
mod return_value;
mod runtime_error;
mod scanner;
mod stmt;
mod token;
mod token_type;
mod value;
mod write_output;

thread_local! {
    static HAD_ERROR: Cell<bool> = Cell::new(false);
}
thread_local! {
    static HAD_RUNTIME_ERROR: Cell<bool> = Cell::new(false);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: cargo run <file_path>");
        std::process::exit(1);
    } else if args.len() == 2 {
        run_file(&args[1], "");
    } else {
        run_prompt();
    }
}

fn run_file(file_path: &str, output_file: &str) {
    let path = Path::new(file_path);
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Error: Could not open file '{}'. {}", file_path, err);
            std::process::exit(1);
        }
    };

    let mut contents = String::new();
    if let Err(err) = file.read_to_string(&mut contents) {
        eprintln!("Error: Could not read from file '{}'. {}", file_path, err);
        std::process::exit(1);
    }

    if HAD_RUNTIME_ERROR.with(|had_error| had_error.get()) {
        std::process::exit(75);
    }

    run(&contents, output_file);
}

fn run_prompt() {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        let bytes_read = io::stdin().read_line(&mut input);
        match bytes_read {
            Ok(0) => {
                break;
            }
            Ok(_) => {
                run(&input.trim(), "");
            }
            Err(err) => {
                eprintln!("Error reading input: {}", err);
                break;
            }
        }
    }
    HAD_ERROR.with(|had_error| {
        if had_error.get() {
            std::process::exit(65);
        }
    });
}

fn run(source: &str, output_file: &str) {
    HAD_ERROR.with(|had_error| {
        had_error.set(false);
    });

    let src = source.to_string();
    let mut scan = scanner::Scanner::new(src); // Create a new Scanner
    let tokens = scan.scan_tokens(); // Scan tokens

    let mut parse = parser::Parser::new(tokens.clone()); // Create a new Parser
    let statements: Vec<Option<stmt::Stmt>> = parse.parse(); // Parse the tokens

    if HAD_ERROR.with(|had_error| had_error.get()) {
        return;
    }

    let interp = Rc::new(RefCell::new(interpreter::Interpreter::new(output_file)));

    let mut resolver = resolver::Resolver::new(interp.clone());
    resolver.resolve(statements.clone());

    interp.borrow_mut().interpret(statements);
}

fn error(line: i32, message: &str) {
    report(line, "", message);
}

fn runtime_error(error: runtime_error::RuntimeError) {
    eprintln!("{}\n[line {}]", error.message, error.token.line);
    HAD_RUNTIME_ERROR.with(|had_error| {
        had_error.set(true);
    }); // Assuming `had_runtime_error` is a thread-local variable
}

fn error_token(token: &token::Token, message: &str) {
    if token.type_ == token_type::TokenType::EoF {
        report(token.line, "at end", message);
    } else {
        report(token.line, &format!("at '{}'", token.lexeme), message);
    }
}

fn report(line: i32, location: &str, message: &str) {
    eprintln!("[line {}] Error {}: {}", line, location, message);
    HAD_ERROR.with(|had_error| {
        had_error.set(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    enum Success {
        Standard,
    }

    fn run_test(folder_name: &str, test_name: &str) -> Result<Success, String> {
        // Define file names
        let test_src = format!("./tests/{}/{}.lox", folder_name, test_name);
        let test_output = format!("./output/actual/{}/{}.txt", folder_name, test_name);
        let test_comparison = format!("./output/expected/{}/{}.txt", folder_name, test_name);

        // Clear the output file
        File::create(&test_output).map_err(|_| "Failed to clear output file")?;

        // Run the test
        run_file(&test_src, &test_output);

        // Open the files
        let output_file = File::open(&test_output).map_err(|_| "Failed to open output file")?;
        let expected_file =
            File::open(&test_comparison).map_err(|_| "Failed to open expected file")?;

        // Compare number of lines in the files (by re-opening the files)
        let output_line_count =
            BufReader::new(File::open(&test_output).map_err(|_| "Failed to open output file")?)
                .lines()
                .count();
        let expected_line_count = BufReader::new(
            File::open(&test_comparison).map_err(|_| "Failed to open expected file")?,
        )
        .lines()
        .count();

        if output_line_count != expected_line_count {
            let err_str = format!(
                "Test {} {} failed: actual and expected files have different numbers of lines.\nActual: {}\nExpected: {}",
                folder_name, test_name, output_line_count, expected_line_count
            );
            return Err(err_str);
        }

        // Create buffered readers for the files
        let output_reader = BufReader::new(output_file);
        let expected_reader = BufReader::new(expected_file);

        // Compare the contents of the files line by line
        for (output_line, expected_line) in output_reader.lines().zip(expected_reader.lines()) {
            let output_line = output_line.map_err(|_| "Failed to read from output file")?;
            let expected_line = expected_line.map_err(|_| "Failed to read from expected file")?;

            if output_line != expected_line {
                let err_str = format!(
                    "Test {} {} failed: actual and expected values differ.\nActual: '{}'\nExpected: '{}'",
                    folder_name, test_name, output_line, expected_line
                );
                return Err(err_str);
            }
        }

        Ok(Success::Standard)
    }

    #[test]
    fn assignment_grouping() {
        let result = std::panic::catch_unwind(|| {
            run_test("assignment", "grouping")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn assignment_syntax() {
        match run_test("assignment", "syntax") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn assignment_global() {
        match run_test("assignment", "global") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn assignment_prefix_operator() {
        let result = std::panic::catch_unwind(|| {
            run_test("assignment", "prefix_operator")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn assignment_associativity() {
        match run_test("assignment", "associativity") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn assignment_to_this() {
        let result = std::panic::catch_unwind(|| {
            run_test("assignment", "to_this")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn assignment_infix_operator() {
        let result = std::panic::catch_unwind(|| {
            run_test("assignment", "infix_operator")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn assignment_local() {
        match run_test("assignment", "local") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn assignment_undefined() {
        let result = std::panic::catch_unwind(|| {
            run_test("assignment", "undefined")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn while_return_closure() {
        match run_test("while", "return_closure") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn while_var_in_body() {
        let result = std::panic::catch_unwind(|| {
            run_test("while", "var_in_body")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    #[ignore]
    fn while_syntax() {
        match run_test("while", "syntax") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn while_return_inside() {
        match run_test("while", "return_inside") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    #[ignore]
    fn while_closure_in_body() {
        match run_test("while", "closure_in_body") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn while_class_in_body() {
        let result = std::panic::catch_unwind(|| {
            run_test("while", "class_in_body")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn while_fun_in_body() {
        let result = std::panic::catch_unwind(|| {
            run_test("while", "fun_in_body")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }
}
