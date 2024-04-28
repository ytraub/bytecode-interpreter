mod common;

mod compiler;
mod scanner;

mod chunk;
mod value;
mod vm;

use compiler::Compiler;
use vm::{InterpretResult, Vm};

use std::{
    env, fs,
    io::{self, BufRead, Write},
};

fn repl() -> Result<(), String> {
    loop {
        print!("> ");

        match io::stdout().flush() {
            Err(_) => return Err(common::repl_error("Failed to flush stdout".to_string())),
            _ => (),
        }

        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut buffer = String::new();

        match handle.read_line(&mut buffer) {
            Err(_) => return Err(common::repl_error("Failed to read from stdin".to_string())),
            Ok(n) => {
                // Handling eof
                if n < 2 {
                    return Ok(());
                }
            }
        }

        match run_repl(buffer) {
            Err(_) => return Err(common::repl_error("Failed to run".to_string())),
            _ => (),
        };
    }
}

fn run_repl(source: String) -> Result<(), InterpretResult> {
    let mut vm = Vm::new();
    return vm.interpret_source(source);
}

fn run_file(path: &str) -> Result<(), String> {
    match fs::read_to_string(path) {
        Err(msg) => {
            return Err(common::runtime_error(format!(
                "Failed to read file:\n\r{}",
                msg
            )))
        }
        Ok(source) => {
            match path.split("/").last() {
                Some(filename) => {
                    if let Some(filename) = filename.strip_suffix(".lox") {
                        match compile_source(source, &format!("lox/bin/{}", filename)) {
                            Err(message) => return Err(message),
                            _ => return Ok(()),
                        };
                    };
                }
                None => (),
            };

            return Err(common::runtime_error(format!("Invalid filename")));
        }
    };
}

fn compile_source(source: String, path: &str) -> Result<(), String> {
    let mut compiler = Compiler::new(source);
    compiler.to_file(path)?;

    match fs::read(path) {
        Err(msg) => {
            return Err(common::runtime_error(format!(
                "Failed to read bin:\n\r{}",
                msg
            )))
        }
        Ok(op_code) => {
            let mut vm = Vm::new();
            vm.interpret_op_code(op_code);
        }
    };
    Ok(())
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
