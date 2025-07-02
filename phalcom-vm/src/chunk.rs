use crate::bytecode::OpCode;
use crate::value::Value;

/// A chunk of compiled bytecode and its associated constant values.
#[derive(Debug, Default, Clone)]
pub struct Chunk {
    pub code: Vec<(OpCode, u8)>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn with_code(code: Vec<(OpCode, u8)>, constants: Vec<Value>) -> Self {
        Self { code, constants }
    }

    pub fn add_instruction(&mut self, opcode: OpCode, operand: u8) {
        self.code.push((opcode, operand));
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        (self.constants.len() - 1) as u8
    }
}
