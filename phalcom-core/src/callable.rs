use crate::{chunk::Chunk, interner::Symbol};

#[derive(Debug, Clone)]
pub struct Callable {
    pub chunk: Chunk,
    pub max_slots: usize,
    pub num_upvalues: usize,
    pub arity: usize,
    pub name_sym: Symbol,
}
