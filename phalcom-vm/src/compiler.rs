use crate::chunk::Chunk;

// The compiler's state. A new Compiler is created for each method being compiled.
pub struct Compiler {
    chunk: Chunk,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::default(),
        }
    }
}
