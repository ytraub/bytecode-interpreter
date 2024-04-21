use std::collections::VecDeque;

use crate::{
    chunk::{byte_to_op, Chunk, OpCode},
    value::{print_value, Value},
};

pub const DEBUG_TRACE_EXECUTION: bool = true;

pub enum InterpretResult {
    InterpretCompileError,
    InterpretRuntimeError,
}

#[derive(Debug)]
pub struct Vm {
    chunk: Option<Chunk>,
    ip: usize,
    stack: VecDeque<Value>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            chunk: None,
            ip: 0,
            stack: VecDeque::new(),
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<(), InterpretResult> {
        self.chunk = Some(chunk);
        self.ip = 0;
        return self.run();
    }

    pub fn run(&mut self) -> Result<(), InterpretResult> {
        macro_rules! binary_operation {
            ($op: tt) => {
                if let Some(a) = self.pop_stack() {
                    if let Some(b) = self.pop_stack() {
                        self.push_stack(a $op b)
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
