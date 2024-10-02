use std::cell::Cell;
use std::env;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::Path;

mod ast_printer;
mod environment;
mod expr;
mod interpreter;
mod parser;
mod runtime_error;
mod scanner;
mod stmt;
mod token;
mod token_type;
mod value;
mod callable;
mod lox_function;
mod return_value;

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
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}

fn run_file(file_path: &str) {
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

    run(&contents);
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
                run(&input.trim());
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

fn run(source: &str) {
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
    let mut interp = interpreter::Interpreter::new();
    interp.interpret(statements);
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
