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

/// One global-resolution cache slot, owned by a single `Bytecode::GetGlobal` or
/// `Bytecode::SetGlobal` site.
///
/// Resolving a global name is otherwise a SipHash probe of
/// [`ModuleObject::name_to_slot`](crate::heap::ModuleObject) on *every* access,
/// plus a second probe of the core module when the name is not local (perf-log
/// F12: ~13% of `bare_send` ticks; `for`'s inner loop pays four probes per
/// iteration). This records where the name landed so repeat accesses index
/// straight into [`ModuleObject::globals`](crate::heap::ModuleObject).
#[derive(Debug, Clone, Copy)]
pub struct GlobalCache {
    /// Module the name actually resolved in — the accessing closure's own module,
    /// or the core module when it resolved through the fallback.
    pub module: crate::heap::ObjRef,
    /// Slot index within that module's `globals`.
    pub slot: usize,
    /// `globals_version` **of the accessing closure's module** (not of
    /// [`Self::module`]) at record time. A mismatch means that module declared a
    /// new name since, which may shadow this resolution, so the entry is dropped.
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
    /// Parallel to `code`; only `Bytecode::GetGlobal`/`SetGlobal` indices are ever
    /// non-`None`. Separate from [`Self::caches`] because the two never occupy the
    /// same instruction, and a single union would pay for the wider variant at
    /// every site.
    pub gcaches: Vec<Cell<Option<GlobalCache>>>,
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
            gcaches: Vec::new(),
        }
    }

    /// Appends an instruction, keeping `caches.len() == gcaches.len() == code.len()`.
    pub fn add_instruction(&mut self, opcode: Bytecode, range: SourceRange) {
        self.code.push(opcode);
        self.spans.push(range);
        self.caches.push(Cell::new(None));
        self.gcaches.push(Cell::new(None));
    }

    /// Appends a constant and returns its index.
    pub fn add_constant(&mut self, value: Value) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }
}
