//! Compiled code unit representation.
//!
//! A [`Callable`] contains the compiled bytecode [`Chunk`]
//! along with metadata like slots, parameters, and upvalue descriptors.

use crate::{chunk::Chunk, interner::Symbol};

/// Describes how an upvalue is captured relative to the enclosing scopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpvalueDescriptor {
    /// True if the variable is in the immediately enclosing stack frame.
    /// False if it is captured from a outer frame.
    pub is_local: bool,
    /// The index on the stack (if `is_local` is true) or the index in the outer
    /// closure's upvalue list (if `is_local` is false).
    pub index: usize,
}

/// A compiled code unit: bytecode, constant pool, and signature metadata.
#[derive(Debug, Clone)]
pub struct Callable {
    /// The compiled bytecode instructions.
    pub chunk: Chunk,
    /// Maximum number of stack slots required by this callable.
    pub max_slots: usize,
    /// Number of upvalues this closure captures.
    pub num_upvalues: usize,
    /// Upvalue descriptors defining how each upvalue is captured.
    pub upvalues: Vec<UpvalueDescriptor>,
    /// Positional parameter count.
    pub arity: usize,
    /// Interned selector/method symbol name.
    pub name_sym: Symbol,
    /// Names of local variables declared in this callable.
    pub local_names: Vec<Symbol>,
}
