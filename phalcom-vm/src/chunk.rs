use crate::bytecode::Bytecode;
use crate::value::Value;
use phalcom_common::range::SourceRange;

/// A chunk of compiled bytecode and its associated constant values.
#[derive(Debug, Default, Clone)]
pub struct Chunk {
    pub code: Vec<Bytecode>,
    pub constants: Vec<Value>,
    pub spans: Vec<SourceRange>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            spans: Vec::new(),
        }
    }

    pub fn add_instruction(&mut self, opcode: Bytecode, range: SourceRange) {
        self.code.push(opcode);
        self.spans.push(range);
    }

    pub fn add_constant(&mut self, value: Value) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }
}
