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
        if let Err(_) = io::stdout().flush() {
            return Err(common::repl_error("Failed to flush stdout".to_string()));
        }

        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut buffer = String::new();

        if let Err(_) = handle.read_line(&mut buffer) {
            return Err(common::repl_error("Failed to read from stdin".to_string()));
        }

        if buffer.len() < 2 {
            return Ok(());
        }

        if let Err(_) = run_source(buffer) {
            return Err(common::repl_error(
                "Failed to run due to above error.".to_string(),
            ));
        }
    }
}

fn run_source(source: String) -> Result<(), InterpretResult> {
    let mut vm = Vm::new();
    vm.interpret_source(source)
}

fn run_file(input_path: &str) -> Result<(), String> {
    match fs::read_to_string(input_path) {
        Err(msg) => {
            return Err(common::runtime_error(format!(
                "Failed to read file:\n\r{}",
                msg
            )))
        }
        Ok(source) => {
            if let Some(filename) = input_path
                .split('/')
                .last()
                .and_then(|name| name.strip_suffix(".lox"))
            {
                match compile_source(source, &format!("lox/bin/{}", filename)) {
                    Ok(op_code) => {
                        let mut vm = Vm::new();
                        if let Err(_) = vm.interpret_op_code(op_code) {
                            return Err(common::runtime_error(
                                "Failed to run due to above error.".to_string(),
                            ));
                        }
                        return Ok(());
                    }
                    Err(message) => return Err(message),
                }
            }
            return Err(common::runtime_error(format!("Invalid filename")));
        }
    }
}

fn run_bin(input_path: &str) -> Result<(), String> {
    match fs::read(input_path) {
        Err(msg) => {
            return Err(common::runtime_error(format!(
                "Failed to read bin at {}:\n\r{}",
                input_path, msg
            )))
        }
        Ok(op_code) => {
            let mut vm = Vm::new();
            if let Err(_) = vm.interpret_op_code(op_code) {
                return Err(common::runtime_error(
                    "Failed to run due to above error.".to_string(),
                ));
            }
            return Ok(());
        }
    }
}

fn compile_file(input_path: &str) -> Result<(), String> {
    match fs::read_to_string(input_path) {
        Err(msg) => {
            return Err(common::runtime_error(format!(
                "Failed to read file at {}:\n\r{}",
                input_path, msg
            )))
        }
        Ok(source) => {
            if let Some(filename) = input_path
                .split('/')
                .last()
                .and_then(|name| name.strip_suffix(".lox"))
            {
                compile_source(source, &format!("lox/bin/{}", filename))?;
                println!("[DONE]: Successfully compiled to bin!");
                return Ok(());
            }
            return Err(common::runtime_error(format!("Invalid filename")));
        }
    }
}

fn compile_source(source: String, output_path: &str) -> Result<Vec<u8>, String> {
    let mut compiler = Compiler::new(source);
    compiler.to_file(output_path)?;

    match fs::read(output_path) {
        Err(msg) => {
            return Err(common::runtime_error(format!(
                "Failed to read bin:\n\r{}",
                msg
            )))
        }
        Ok(op_code) => return Ok(op_code),
    }
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
        3 => match args[1].as_str() {
            "run" => handle_run!(run_file(args[2].as_str())),
            "compile" => handle_run!(compile_file(args[2].as_str())),
            "execute" => handle_run!(run_bin(args[2].as_str())),
            _ => {
                println!("[USAGE]: runtime [action] [source]");
                std::process::exit(64);
            }
        },
        _ => {
            println!("[USAGE]: runtime [action] [source]");
            std::process::exit(64);
        }
    }
}
