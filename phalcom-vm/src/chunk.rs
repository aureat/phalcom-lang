use crate::bytecode::Bytecode;
use crate::value::Value;

/// A chunk of compiled bytecode and its associated constant values.
#[derive(Debug, Default, Clone)]
pub struct Chunk {
    pub code: Vec<Bytecode>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn with_code(code: Vec<Bytecode>, constants: Vec<Value>) -> Self {
        Self { code, constants }
    }

    pub fn add_instruction(&mut self, opcode: Bytecode) {
        self.code.push(opcode);
    }

    pub fn add_constant(&mut self, value: Value) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }
}
