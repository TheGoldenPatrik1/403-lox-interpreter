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
    panic!("{}\n[line {}]", error.message, error.token.line);
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
    panic!("[line {}] Error {}: {}", line, location, message);
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
    fn comments_line_at_eof() {
        match run_test("comments", "line_at_eof") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn comments_only_line_comment() {
        match run_test("comments", "only_line_comment") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn comments_only_line_comment_and_line() {
        match run_test("comments", "only_line_comment_and_line") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn nil_literal() {
        match run_test("nil", "literal") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn if_var_in_then() {
        let result = std::panic::catch_unwind(|| {
            run_test("if", "var_in_then")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn if_dangling_else() {
        match run_test("if", "dangling_else") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn if_truth() {
        match run_test("if", "truth") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn if_fun_in_else() {
        let result = std::panic::catch_unwind(|| {
            run_test("if", "fun_in_else")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn if_class_in_else() {
        let result = std::panic::catch_unwind(|| {
            run_test("if", "class_in_else")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn if_else() {
        match run_test("if", "else") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn if_fun_in_then() {
        let result = std::panic::catch_unwind(|| {
            run_test("if", "fun_in_then")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn if_class_in_then() {
        let result = std::panic::catch_unwind(|| {
            run_test("if", "class_in_then")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn if_var_in_else() {
        let result = std::panic::catch_unwind(|| {
            run_test("if", "var_in_else")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn if_if() {
        match run_test("if", "if") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
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
    fn function_empty_body() {
        match run_test("function", "empty_body") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn function_too_many_arguments() {
        let result = std::panic::catch_unwind(|| {
            run_test("function", "too_many_arguments")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn function_missing_comma_in_parameters() {
        let result = std::panic::catch_unwind(|| {
            run_test("function", "missing_comma_in_parameters")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn function_nested_call_with_arguments() {
        match run_test("function", "nested_call_with_arguments") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn function_body_must_be_block() {
        let result = std::panic::catch_unwind(|| {
            run_test("function", "body_must_be_block")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn function_missing_arguments() {
        let result = std::panic::catch_unwind(|| {
            run_test("function", "missing_arguments")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn function_parameters() {
        match run_test("function", "parameters") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn function_local_recursion() {
        match run_test("function", "local_recursion") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn function_recursion() {
        match run_test("function", "recursion") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn function_print() {
        match run_test("function", "print") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn function_too_many_parameters() {
        let result = std::panic::catch_unwind(|| {
            run_test("function", "too_many_parameters")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn function_mutual_recursion() {
        match run_test("function", "mutual_recursion") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn function_extra_arguments() {
        let result = std::panic::catch_unwind(|| {
            run_test("function", "extra_arguments")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn print_missing_argument() {
        let result = std::panic::catch_unwind(|| {
            run_test("print", "missing_argument")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn number_decimal_point_at_eof() {
        let result = std::panic::catch_unwind(|| {
            run_test("number", "decimal_point_at_eof")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn number_nan_equality() {
        match run_test("number", "nan_equality") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn number_literals() {
        match run_test("number", "literals") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn number_leading_dot() {
        let result = std::panic::catch_unwind(|| {
            run_test("number", "leading_dot")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn number_trailing_dot() {
        let result = std::panic::catch_unwind(|| {
            run_test("number", "trailing_dot")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn call_nil() {
        let result = std::panic::catch_unwind(|| {
            run_test("call", "nil")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn call_bool() {
        let result = std::panic::catch_unwind(|| {
            run_test("call", "bool")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn call_num() {
        let result = std::panic::catch_unwind(|| {
            run_test("call", "num")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn call_object() {
        let result = std::panic::catch_unwind(|| {
            run_test("call", "object")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn call_string() {
        let result = std::panic::catch_unwind(|| {
            run_test("call", "string")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn logical_operator_and() {
        match run_test("logical_operator", "and") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn logical_operator_or() {
        match run_test("logical_operator", "or") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn logical_operator_and_truth() {
        match run_test("logical_operator", "and_truth") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn logical_operator_or_truth() {
        match run_test("logical_operator", "or_truth") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn bool_equality() {
        match run_test("bool", "equality") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn bool_not() {
        match run_test("bool", "not") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn for_return_closure() {
        match run_test("for", "return_closure") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn for_scope() {
        match run_test("for", "scope") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn for_var_in_body() {
        let result = std::panic::catch_unwind(|| {
            run_test("for", "var_in_body")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn for_syntax() {
        match run_test("for", "syntax") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn for_return_inside() {
        match run_test("for", "return_inside") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn for_statement_initializer() {
        let result = std::panic::catch_unwind(|| {
            run_test("for", "statement_initializer")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn for_statement_increment() {
        let result = std::panic::catch_unwind(|| {
            run_test("for", "statement_increment")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn for_statement_condition() {
        let result = std::panic::catch_unwind(|| {
            run_test("for", "statement_condition")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn for_class_in_body() {
        let result = std::panic::catch_unwind(|| {
            run_test("for", "class_in_body")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn for_fun_in_body() {
        let result = std::panic::catch_unwind(|| {
            run_test("for", "fun_in_body")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn class_empty() {
        match run_test("class", "empty") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn class_reference_self() {
        match run_test("class", "reference_self") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn class_local_reference_self() {
        match run_test("class", "local_reference_self") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
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

    #[test]
    fn operator_add_num_nil() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "add_num_nil")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_equals_method() {
        match run_test("operator", "equals_method") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn operator_equals_class() {
        match run_test("operator", "equals_class") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn operator_subtract_num_nonnum() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "subtract_num_nonnum")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_multiply() {
        match run_test("operator", "multiply") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn operator_negate() {
        match run_test("operator", "negate") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn operator_divide_nonnum_num() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "divide_nonnum_num")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_comparison() {
        match run_test("operator", "comparison") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn operator_greater_num_nonnum() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "greater_num_nonnum")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_less_or_equal_nonnum_num() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "less_or_equal_nonnum_num")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_multiply_nonnum_num() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "multiply_nonnum_num")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_not_equals() {
        match run_test("operator", "not_equals") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn operator_add_bool_num() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "add_bool_num")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_negate_nonnum() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "negate_nonnum")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_add() {
        match run_test("operator", "add") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn operator_greater_or_equal_nonnum_num() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "greater_or_equal_nonnum_num")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_equals() {
        match run_test("operator", "equals") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn operator_less_nonnum_num() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "less_nonnum_num")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_add_bool_string() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "add_bool_string")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_divide() {
        match run_test("operator", "divide") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn operator_add_string_nil() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "add_string_nil")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_add_bool_nil() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "add_bool_nil")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_divide_num_nonnum() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "divide_num_nonnum")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_multiply_num_nonnum() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "multiply_num_nonnum")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_less_or_equal_num_nonnum() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "less_or_equal_num_nonnum")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_greater_nonnum_num() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "greater_nonnum_num")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_not() {
        match run_test("operator", "not") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn operator_add_nil_nil() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "add_nil_nil")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_subtract() {
        match run_test("operator", "subtract") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn operator_subtract_nonnum_num() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "subtract_nonnum_num")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_not_class() {
        match run_test("operator", "not_class") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn operator_greater_or_equal_num_nonnum() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "greater_or_equal_num_nonnum")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn operator_less_num_nonnum() {
        let result = std::panic::catch_unwind(|| {
            run_test("operator", "less_num_nonnum")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn constructor_call_init_explicitly() {
        match run_test("constructor", "call_init_explicitly") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn constructor_return_value() {
        let result = std::panic::catch_unwind(|| {
            run_test("constructor", "return_value")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn constructor_init_not_method() {
        match run_test("constructor", "init_not_method") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn constructor_missing_arguments() {
        let result = std::panic::catch_unwind(|| {
            run_test("constructor", "missing_arguments")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn constructor_default() {
        match run_test("constructor", "default") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn constructor_arguments() {
        match run_test("constructor", "arguments") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn constructor_default_arguments() {
        let result = std::panic::catch_unwind(|| {
            run_test("constructor", "default_arguments")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn constructor_call_init_early_return() {
        match run_test("constructor", "call_init_early_return") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn constructor_extra_arguments() {
        let result = std::panic::catch_unwind(|| {
            run_test("constructor", "extra_arguments")
        });
        assert!(result.is_err(), "Expected a panic but did not get one");
    }

    #[test]
    fn constructor_return_in_nested_function() {
        match run_test("constructor", "return_in_nested_function") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn constructor_early_return() {
        match run_test("constructor", "early_return") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn block_empty() {
        match run_test("block", "empty") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }

    #[test]
    fn block_scope() {
        match run_test("block", "scope") {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{}", err),
        }
    }
}