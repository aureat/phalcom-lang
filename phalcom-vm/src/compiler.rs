use crate::chunk::Chunk;

// The compiler's state. A new Compiler is created for each method being compiled.
pub struct Compiler {
    chunk: Chunk,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::default(),
        }
    }
}
