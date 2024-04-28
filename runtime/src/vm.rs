use std::collections::VecDeque;

use crate::compiler::Compiler;

use crate::chunk::{byte_to_op, Chunk, OpCode};
use crate::common::DEBUG_TRACE_EXECUTION;
use crate::value::{print_value, Value};

pub enum InterpretResult {
    InterpretCompileError,
    InterpretRuntimeError,
}

#[derive(Debug)]
pub struct Vm {
    chunk: Option<Chunk>,
    stack: VecDeque<Value>,
    ip: usize,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            chunk: None,
            stack: VecDeque::new(),
            ip: 0,
        }
    }

    pub fn interpret(&mut self, source: String) -> Result<(), InterpretResult> {
        let mut compiler = Compiler::new(source);
        let chunk = Chunk::new();

        match compiler.to_chunk(chunk) {
            Some(chunk) => self.chunk = Some(chunk),
            None => return Err(InterpretResult::InterpretCompileError),
        };

        self.ip = 0;

        let result = self.run();
        return result;
    }

    pub fn run(&mut self) -> Result<(), InterpretResult> {
        macro_rules! binary_operation {
            ($op: tt) => {
                if let Some(a) = self.pop_stack() {
                    if let Some(b) = self.pop_stack() {
                        self.push_stack(b $op a)
                    }
                }
            };
        }

        let mut offset = 0;

        loop {
            if DEBUG_TRACE_EXECUTION {
                print!("          ");
                for value in &self.stack {
                    print!("[{}]", value);
                }
                println!();

                if let Some(chunk) = &self.chunk {
                    match chunk.dissasemble_instruction(offset) {
                        Ok(new_offset) => offset = new_offset,
                        Err(err) => {
                            println!("{}", err);
                            return Err(InterpretResult::InterpretRuntimeError);
                        }
                    }
                };
            }

            let instruction = self.read_byte()?;
            match byte_to_op(instruction) {
                Ok(operation) => match operation {
                    OpCode::OpReturn => {
                        if let Some(value) = self.pop_stack() {
                            print_value(value);
                            println!();
                        }

                        return Ok(());
                    }
                    OpCode::OpConstant => {
                        let constant = self.read_constant()?;
                        self.push_stack(constant);
                    }
                    OpCode::OpNegate => {
                        if let Some(value) = self.pop_stack() {
                            self.push_stack(-value);
                        }
                    }
                    OpCode::OpAdd => binary_operation!(+),
                    OpCode::OpSubtract => binary_operation!(-),
                    OpCode::OpMultiply => binary_operation!(*),
                    OpCode::OpDivide => binary_operation!(/),
                },
                Err(err) => {
                    println!("{}", err);
                    return Err(InterpretResult::InterpretRuntimeError);
                }
            }
        }
    }

    pub fn push_stack(&mut self, value: Value) {
        self.stack.push_front(value);
    }

    pub fn pop_stack(&mut self) -> Option<Value> {
        return self.stack.pop_front();
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    fn read_byte(&mut self) -> Result<u8, InterpretResult> {
        if let Some(chunk) = &self.chunk {
            let byte = chunk.code[self.ip];
            self.ip += 1;
            return Ok(byte);
        }
        return Err(InterpretResult::InterpretRuntimeError);
    }

    fn read_constant(&mut self) -> Result<Value, InterpretResult> {
        if let Some(chunk) = &self.chunk {
            let constant = chunk.constants[chunk.code[self.ip] as usize];
            self.ip += 1;
            return Ok(constant);
        }
        return Err(InterpretResult::InterpretRuntimeError);
    }
}
