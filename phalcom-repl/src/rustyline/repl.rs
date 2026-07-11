pub struct ReplSession {
    cwd: std::path::PathBuf,
    next_cell: usize,
}

impl ReplSession {
    pub fn start(cwd: std::path::PathBuf) -> ReplSession {
        Self { cwd, next_cell: 1 }
    }

    /// Evaluate one input “cell”. If it’s a bare expression, returns Some(Value).
    pub fn eval(&mut self, src: &str) -> usize {
        let cell_id = self.next_cell;
        self.next_cell += 1;
        cell_id
    }
}
