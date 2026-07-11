//! REPL session state for the Phalcom interactive interpreter.
//!
//! [`ReplSession`] tracks per-session state such as the working directory and
//! a monotonically-increasing cell counter.  Evaluation is delegated to the
//! compiler/VM; the session layer is responsible only for sequencing and
//! maintaining call-side state.

/// Holds per-session state for one Phalcom REPL invocation.
///
/// Each call to [`ReplSession::eval`] increments an internal cell counter and
/// returns it as the cell identifier, mirroring Jupyter-style `In [N]` / `Out
/// [N]` numbering.
pub struct ReplSession {
    /// The working directory from which the REPL was launched.
    ///
    /// Used to resolve relative import paths and as the base for file-backed
    /// history (via the caller).
    #[allow(dead_code)] // will be read once the VM is wired up
    cwd: std::path::PathBuf,
    /// The 1-based index of the next evaluation cell.
    next_cell: usize,
}

impl ReplSession {
    /// Starts a new REPL session rooted at the given working directory.
    pub fn start(cwd: std::path::PathBuf) -> ReplSession {
        Self { cwd, next_cell: 1 }
    }

    /// Evaluates one input cell.
    ///
    /// Increments the internal cell counter and returns the newly assigned
    /// cell ID.  Currently a stub — compilation and VM execution are not yet
    /// wired up.
    pub fn eval(&mut self, _src: &str) -> usize {
        let cell_id = self.next_cell;
        self.next_cell += 1;
        cell_id
    }
}
