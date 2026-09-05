//! Output sinks owned by each [`super::VM`].

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// Destination for bytes emitted by Phalcom-visible runtime output.
pub trait RuntimeOutput: Send {
    /// Writes bytes without adding separators or line endings.
    fn write(&mut self, bytes: &[u8]) -> io::Result<()>;

    /// Flushes buffered output when the sink has buffering semantics.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Shipping sink that preserves the historical stdout behavior.
#[derive(Debug, Default, Clone, Copy)]
pub struct StdoutOutput;

impl RuntimeOutput for StdoutOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        io::stdout().lock().write_all(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().lock().flush()
    }
}

/// In-memory sink for embedders and fixture-local golden assertions.
pub struct BufferedOutput {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl BufferedOutput {
    /// Creates an empty output buffer.
    pub fn new() -> Self {
        Self {
            bytes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Returns a read handle that remains usable after this sink is moved into a VM.
    pub fn handle(&self) -> OutputHandle {
        OutputHandle {
            bytes: Arc::clone(&self.bytes),
        }
    }

    /// Returns a snapshot of bytes written so far.
    pub fn bytes(&self) -> Vec<u8> {
        snapshot(&self.bytes)
    }
}

impl Default for BufferedOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeOutput for BufferedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        let mut output = self.bytes.lock().map_err(|_| io::Error::other("runtime output sink poisoned"))?;
        output.extend_from_slice(bytes);
        Ok(())
    }
}

/// Read-only handle for a [`BufferedOutput`] owned by a VM.
#[derive(Clone)]
pub struct OutputHandle {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl OutputHandle {
    /// Returns a snapshot of all bytes emitted through the associated sink.
    pub fn bytes(&self) -> Vec<u8> {
        snapshot(&self.bytes)
    }
}

fn snapshot(bytes: &Arc<Mutex<Vec<u8>>>) -> Vec<u8> {
    match bytes.lock() {
        Ok(output) => output.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}
