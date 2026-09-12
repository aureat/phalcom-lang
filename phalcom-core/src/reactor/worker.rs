//! Plain-data background worker pool and completion transport.
//!
//! # Law of Thread Isolation
//! Background workers operate exclusively on plain Rust structs ([`WorkerJob`],
//! [`WorkerCompletion`]). Conversion to and from Phalcom values/handles occurs
//! exclusively on the main VM thread. No worker thread ever accesses `Value`,
//! `ObjRef`, `Heap`, or `VM`.

use super::registry::ReactorToken;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, spawn};

/// Plain data result returned by a background worker job.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkerResult {
    /// Unit result (e.g., void effect / timer / ping).
    Unit,
    /// Signed integer payload.
    Int(i64),
    /// Boolean payload.
    Bool(bool),
    /// Raw byte buffer.
    Bytes(Vec<u8>),
    /// UTF-8 string payload.
    String(String),
}

/// Outcome of a background worker execution.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkerOutcome {
    /// Successful completion with a plain result payload.
    Success(WorkerResult),
    /// Failure with an error message string.
    Error(String),
}

/// A background worker job containing only owned, `Send + 'static` data.
pub struct WorkerJob {
    /// The reactor registration token for this job.
    pub token: ReactorToken,
    /// The isolated work closure.
    pub work: Box<dyn FnOnce() -> WorkerOutcome + Send + 'static>,
}

/// A completion event sent from a background worker thread back to the VM thread.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkerCompletion {
    /// The registration token associated with the completed job.
    pub token: ReactorToken,
    /// The outcome of the execution.
    pub outcome: WorkerOutcome,
}

// Compile-time assertion verifying strict `Send + 'static` thread boundary.
fn _assert_send_static<T: Send + 'static>() {}
fn _verify_thread_isolation() {
    _assert_send_static::<WorkerJob>();
    _assert_send_static::<WorkerCompletion>();
    _assert_send_static::<WorkerOutcome>();
    _assert_send_static::<WorkerResult>();
}

enum WorkerMessage {
    Job(WorkerJob),
    Shutdown,
}

/// Default number of background worker threads.
pub const DEFAULT_WORKER_THREADS: usize = 4;

/// Bounded pool of worker threads processing off-thread computational or blocking tasks.
pub struct WorkerPool {
    job_tx: Option<Sender<WorkerMessage>>,
    workers: Vec<JoinHandle<()>>,
}

impl WorkerPool {
    /// Creates a new worker pool with `num_threads` workers, sending completions
    /// back through `completion_tx` and waking `waker` if provided.
    pub fn new(num_threads: usize, completion_tx: Sender<WorkerCompletion>, waker: Option<Arc<mio::Waker>>) -> Self {
        let (job_tx, job_rx) = channel::<WorkerMessage>();
        let job_rx = Arc::new(Mutex::new(job_rx));
        let mut workers = Vec::with_capacity(num_threads);

        for _ in 0..num_threads {
            let rx = Arc::clone(&job_rx);
            let tx = completion_tx.clone();
            let thread_waker = waker.clone();
            let handle = spawn(move || {
                loop {
                    let msg = {
                        let lock = match rx.lock() {
                            Ok(guard) => guard,
                            Err(poisoned) => poisoned.into_inner(),
                        };
                        match lock.recv() {
                            Ok(msg) => msg,
                            Err(_) => break, // Channel closed
                        }
                    };

                    match msg {
                        WorkerMessage::Job(job) => {
                            let outcome = (job.work)();
                            let completion = WorkerCompletion { token: job.token, outcome };
                            let _ = tx.send(completion);
                            if let Some(w) = &thread_waker {
                                let _ = w.wake();
                            }
                        }
                        WorkerMessage::Shutdown => break,
                    }
                }
            });
            workers.push(handle);
        }

        Self { job_tx: Some(job_tx), workers }
    }

    /// Submits a new plain-data job to the pool.
    pub fn submit(&self, job: WorkerJob) -> bool {
        if let Some(tx) = &self.job_tx {
            tx.send(WorkerMessage::Job(job)).is_ok()
        } else {
            false
        }
    }

    /// Signals all worker threads to shut down and joins their handles.
    pub fn shutdown(&mut self) {
        if let Some(tx) = self.job_tx.take() {
            for _ in 0..self.workers.len() {
                let _ = tx.send(WorkerMessage::Shutdown);
            }
            drop(tx);
        }

        for handle in self.workers.drain(..) {
            let _ = handle.join();
        }
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}
