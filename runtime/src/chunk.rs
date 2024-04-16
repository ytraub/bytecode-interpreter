use crate::error;

#[derive(Debug)]
pub enum OpCode {
    OpReturn = 1,
}

pub fn byte_to_op(byte: u8) -> Result<OpCode, String> {
    match byte {
        1 => return Ok(OpCode::OpReturn),
        _ => {
            return Err(error::runtime_error(format!(
                "Invalid conversion to instruction from byte: '{}'\nInstruction doesn't exist.",
                byte
            )))
        }
    };
}

pub struct Chunk {
    code: Vec<u8>,
}

impl Chunk {
    pub fn new() -> Self {
        Self { code: vec![] }
    }

    pub fn write_instruction(&mut self, instruction: OpCode) {
        self.code.push(instruction as u8);
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.code.push(byte);
    }

    pub fn dissasemble(&self, name: &str) -> Result<(), String> {
        println!("== {} ==", name);

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.dissasemble_instruction(offset)?;
        }

        Ok(())
    }

    pub fn dissasemble_instruction(&self, offset: usize) -> Result<usize, String> {
        print!("{:04} ", offset);

        if let Some(byte) = self.code.get(offset) {
            let instruction = byte_to_op(*byte)?;

            match instruction {
                OpCode::OpReturn => {
                    return Ok(self.simple_instruction("OP_RETURN", offset));
                }
                _ => {
                    return Err(error::dissasemble_error(format!(
                        "Unknown instruction found: '{:?}'\nDissasembling not implemented.",
                        instruction
                    )));
                }
            }
        } else {
            return Err(error::dissasemble_error(format!(
                "Invalid instruction found at offset: '{}'\nOffset out of bounds.",
                offset
            )));
        }
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        return offset + 1;
    }
}
