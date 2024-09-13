use std::cell::Cell;
use std::env;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::Path;

mod Scanner;
//use crate::Scanner;

mod Token;
//use crate::Token;

mod TokenType;
//use crate::TokenType;

thread_local! {
    static HAD_ERROR: Cell<bool> = Cell::new(false);
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
    let source = "1 + 2".to_string();
    let mut scanner = Scanner::Scanner::new(source); // Create a new Scanner
    let tokens = scanner.scan_tokens(); // Scan tokens

    // Print the tokens
    for token in tokens {
        println!("{:?}", token); // Assuming Token implements Debug
    }
}

fn error(line: i32, message: &str) {
    report(line, "", message);
}

fn report(line: i32, location: &str, message: &str) {
    eprintln!("[line {}] Error {}: {}", line, location, message);
    HAD_ERROR.with(|had_error| {
        had_error.set(true);
    });
}
