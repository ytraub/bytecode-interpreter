use std::collections::VecDeque;

use crate::compiler::Compiler;

use crate::chunk::{byte_to_op, Chunk, OpCode};
use crate::common::DEBUG_TRACE_EXECUTION;
use crate::value::{Value, ValueType};

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

    pub fn interpret_source(&mut self, source: String) -> Result<(), InterpretResult> {
        self.reset_stack();
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

    pub fn interpret_op_code(&mut self, op_code: Vec<u8>) -> Result<(), InterpretResult> {
        self.reset_stack();
        let mut chunk = Chunk::new();

        let mut lines: Vec<i32> = vec![];
        let mut instructions: Vec<u8> = vec![];
        let mut previous: Option<u8> = None;

        for op in op_code {
            match previous {
                Some(instruction) => {
                    instructions.push(instruction);
                    lines.push(op.into());
                    previous = None;
                }
                None => previous = Some(op),
            }
        }

        let mut i = 0;
        loop {
            if i == instructions.len() {
                break;
            }

            let current = instructions[i];
            match current {
                1 => {
                    if let Some(next) = instructions.get(i + 1) {
                        let constant = chunk.add_constant(Value::from_number(f64::from(*next)));
                        chunk.write_instruction(OpCode::OpConstant, lines[i]);
                        chunk.write_byte(constant, lines[i + 1]);
                        i += 1;
                    }
                }
                _ => chunk.write_byte(current, lines[i]),
            }

            i += 1;
        }

        self.chunk = Some(chunk);
        self.ip = 0;

        self.run()
    }

    pub fn run(&mut self) -> Result<(), InterpretResult> {
        macro_rules! binary_operation {
            ($value_type: expr, $op: tt) => {
                match (self.peek_stack(0), self.peek_stack(1)) {
                    (Some(a), Some(b)) => {
                        if !a.is_number() || !b.is_number() {
                            self.runtime_error("Operands must be numbers.".to_string());
                            return Err(InterpretResult::InterpretRuntimeError);
                        }
                    }
                    _ => {
                        self.runtime_error("Operands missing.".to_string());
                        return Err(InterpretResult::InterpretRuntimeError);
                    }
                }

                if let Some(a) = self.pop_stack() {
                    if let Some(b) = self.pop_stack() {
                        self.push_stack($value_type(b.as_number() $op a.as_number()));
                    }
                }
            };
        }

        let mut offset = 0;

        loop {
            if DEBUG_TRACE_EXECUTION {
                print!("          ");
                for value in &self.stack {
                    print!("[");
                    value.print();
                    print!("]");
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
                            value.print();
                            println!()
                        }

                        return Ok(());
                    }
                    OpCode::OpConstant => {
                        let constant = self.read_constant()?;
                        self.push_stack(constant);
                    }
                    OpCode::OpNil => self.push_stack(Value::from_nil()),
                    OpCode::OpTrue => self.push_stack(Value::from_bool(true)),
                    OpCode::OpFalse => {
                        self.push_stack(Value::from_bool(false));
                    }
                    OpCode::OpNegate => {
                        if let Some(value) = self.peek_stack(0) {
                            if !value.is_number() {
                                self.runtime_error("Operand must be number.".to_string());
                                return Err(InterpretResult::InterpretRuntimeError);
                            }

                            if let Some(value) = &self.pop_stack() {
                                self.push_stack(Value::from_number(-value.as_number()));
                            }
                        }
                    }
                    OpCode::OpNot => {
                        if let Some(value) = &self.pop_stack() {
                            self.push_stack(Value::from_bool(self.is_falsey(value)));
                        }
                    }
                    OpCode::OpAdd => {
                        binary_operation!(Value::from_number, +);
                    }
                    OpCode::OpSubtract => {
                        binary_operation!(Value::from_number, -);
                    }
                    OpCode::OpMultiply => {
                        binary_operation!(Value::from_number, *);
                    }
                    OpCode::OpDivide => {
                        binary_operation!(Value::from_number, /);
                    }
                    OpCode::OpGreater => {
                        binary_operation!(Value::from_bool, >);
                    }
                    OpCode::OpLess => {
                        binary_operation!(Value::from_bool, <);
                    }
                    OpCode::OpEqual => {
                        if let Some(a) = self.pop_stack() {
                            if let Some(b) = self.pop_stack() {
                                self.push_stack(Value::from_bool(self.values_equal(a, b)));
                            }
                        }
                    }
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

    pub fn peek_stack(&self, distance: usize) -> Option<&Value> {
        return self.stack.get(self.stack.len() - (distance + 1));
    }

    fn is_falsey(&self, value: &Value) -> bool {
        return value.is_nil() || (value.is_bool() && !value.as_bool());
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    fn values_equal(&self, a: Value, b: Value) -> bool {
        if a.get_type() != b.get_type() {
            return false;
        }

        match a.get_type() {
            ValueType::ValBool => return a.as_bool() == b.as_bool(),
            ValueType::ValNil => return true,
            ValueType::ValNumber => return a.as_number() == b.as_number(),
        }
    }

    fn runtime_error(&mut self, msg: String) {
        println!("{}", msg);

        if let Some(chunk) = self.chunk.take() {
            let line = chunk.lines[self.ip];
            println!("[line {}] in script\n", line);
            self.chunk = Some(chunk);
        }

        self.reset_stack();
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
            let constant = chunk.constants[chunk.code[self.ip] as usize].clone();
            self.ip += 1;
            return Ok(constant);
        }
        return Err(InterpretResult::InterpretRuntimeError);
    }
}
