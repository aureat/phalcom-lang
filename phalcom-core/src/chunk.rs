use crate::bytecode::Bytecode;
use crate::heap::ClassId;
use crate::value::Value;
use phalcom_common::range::SourceRange;
use std::cell::Cell;

/// One monomorphic inline-cache slot, owned by a single `Bytecode::Invoke` site.
#[derive(Debug, Clone, Copy)]
pub struct InlineCache {
    /// Receiver class the cached resolution was recorded for.
    pub class: ClassId,
    /// The resolved `MethodObject` handle.
    pub method: crate::heap::ObjRef,
    /// `VM.world_version` at record time; a mismatch means a method was
    /// (re)defined somewhere since, and the entry must be discarded.
    pub version: u64,
}

/// A chunk of compiled bytecode and its associated constant values.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<Bytecode>,
    pub constants: Vec<Value>,
    pub spans: Vec<SourceRange>,
    /// Parallel to `code`; only `Bytecode::Invoke` indices are ever non-`None`.
    /// Cell enables interior mutability for cache refill through a shared `&Chunk` borrow.
    pub caches: Vec<Cell<Option<InlineCache>>>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    /// Constructs a new, empty chunk.
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            spans: Vec::new(),
            caches: Vec::new(),
        }
    }

    /// Appends an instruction, keeping `caches.len() == code.len()`.
    pub fn add_instruction(&mut self, opcode: Bytecode, range: SourceRange) {
        self.code.push(opcode);
        self.spans.push(range);
        self.caches.push(Cell::new(None));
    }

    /// Appends a constant and returns its index.
    pub fn add_constant(&mut self, value: Value) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }
}
