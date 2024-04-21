mod error;

mod compiler;
mod scanner;

mod chunk;
mod value;
mod vm;

use vm::Vm;

use std::{
    env, fs,
    io::{self, BufRead, Write},
};

fn repl() -> Result<(), String> {
    loop {
        print!("> ");

        match io::stdout().flush() {
            Err(_) => return Err(error::repl_error("Failed to flush stdout".to_string())),
            _ => (),
        }

        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut buffer = String::new();
        
        match handle.read_line(&mut buffer) {
            Err(_) => return Err(error::repl_error("Failed to read from stdin".to_string())),
            Ok(n) => {
                // Handling eof
                if n < 2 {
                    return Ok(());
                }
            }
        }

        interpret(buffer)?;
    }
}

fn run_file(path: &str) -> Result<(), String> {
    match fs::read_to_string(path) {
        Err(msg) => Err(error::runtime_error(format!(
            "Failed to read file:\n\r{}",
            msg
        ))),
        Ok(source) => return interpret(source),
    }
}

fn interpret(source: String) -> Result<(), String> {
    compiler::compile(source)?;
    return Ok(());
}

fn main() {
    macro_rules! handle_run {
        ($func: expr) => {
            match $func {
                Err(msg) => println!("{}", msg),
                _ => std::process::exit(0),
            }
        };
    }

    let mut _vm = Vm::new();

    let args: Vec<_> = env::args().collect();
    match args.len() {
        1 => handle_run!(repl()),
        2 => handle_run!(run_file(&args[1])),
        _ => {
            println!("[USAGE]: runtime [source]");
            std::process::exit(64);
        }
    }
}
