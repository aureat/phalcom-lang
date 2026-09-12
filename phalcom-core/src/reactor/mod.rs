//! Phase-1 Reactor runtime for asynchronous completion, workers, timers, and executor liveness.
//!
//! # Architecture
//!
//! ```text
//! Reactor
//!  ├── ReactorRegistry (generational token table, GC-roots active targets)
//!  ├── WorkerPool (plain-data off-thread execution)
//!  ├── TimerQueue (monotonic deadline min-heap, no timer thread)
//!  └── Completion Transport (MPSC channel drained on VM thread)
//! ```

pub mod registry;
pub mod timer;
pub mod worker;

pub use registry::{ReactorRegistration, ReactorRegistry, ReactorSource, ReactorToken, RegistrationState};
pub use timer::{TimerEntry, TimerQueue};
pub use worker::{DEFAULT_WORKER_THREADS, WorkerCompletion, WorkerJob, WorkerOutcome, WorkerPool, WorkerResult};

use crate::heap::ObjRef;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

/// A completed reactor event ready to be materialized and delivered to its target on the VM thread.
#[derive(Debug, Clone)]
pub struct ReactorCompletionEvent {
    /// The target object handle (e.g. `Future` instance).
    pub target: ObjRef,
    /// Plain-data execution outcome.
    pub outcome: WorkerOutcome,
}

/// The core reactor managing external completions, timers, and off-thread workers.
pub struct Reactor {
    /// Generational table of live registrations.
    pub registry: ReactorRegistry,
    /// Priority queue of active timers.
    pub timers: TimerQueue,
    /// Bounded background worker pool.
    pub worker_pool: WorkerPool,
    completion_tx: Sender<WorkerCompletion>,
    completion_rx: Receiver<WorkerCompletion>,
    pending_worker_completions: Vec<WorkerCompletion>,
}

impl Default for Reactor {
    fn default() -> Self {
        Self::new(DEFAULT_WORKER_THREADS)
    }
}

impl Reactor {
    /// Creates a new reactor with `worker_threads` background worker threads.
    pub fn new(worker_threads: usize) -> Self {
        let (completion_tx, completion_rx) = channel::<WorkerCompletion>();
        let worker_pool = WorkerPool::new(worker_threads, completion_tx.clone());

        Self {
            registry: ReactorRegistry::new(),
            timers: TimerQueue::new(),
            worker_pool,
            completion_tx,
            completion_rx,
            pending_worker_completions: Vec::new(),
        }
    }

    /// Returns a clone of the completion sender for registering external completion sources.
    pub fn completion_sender(&self) -> Sender<WorkerCompletion> {
        self.completion_tx.clone()
    }

    /// Registers a monotonic timer for `target` that expires after `duration`.
    pub fn register_timer(&mut self, target: ObjRef, duration: Duration) -> ReactorToken {
        let token = self.registry.allocate(target, ReactorSource::Timer);
        self.timers.schedule(token, duration);
        token
    }

    /// Submits a plain-data worker job for `target`.
    pub fn register_worker_job<F>(&mut self, target: ObjRef, work: F) -> ReactorToken
    where
        F: FnOnce() -> WorkerOutcome + Send + 'static,
    {
        let token = self.registry.allocate(target, ReactorSource::Worker);
        let job = WorkerJob { token, work: Box::new(work) };
        self.worker_pool.submit(job);
        token
    }

    /// Non-blockingly drains due timers and worker completions up to a bounded batch size.
    pub fn drain_completions(&mut self, max_batch: usize) -> Vec<ReactorCompletionEvent> {
        let mut events = Vec::new();

        // 1. Drain expired timers.
        let now = Instant::now();
        let due_tokens = self.timers.pop_due(now);
        for token in due_tokens {
            if let Some(target) = self.registry.complete(token) {
                events.push(ReactorCompletionEvent {
                    target,
                    outcome: WorkerOutcome::Success(WorkerResult::Unit),
                });
            }
        }

        // 2. Drain buffered worker completions.
        let buffered = std::mem::take(&mut self.pending_worker_completions);
        for completion in buffered {
            if let Some(target) = self.registry.complete(completion.token) {
                events.push(ReactorCompletionEvent {
                    target,
                    outcome: completion.outcome,
                });
            }
        }

        // 3. Drain new incoming worker completions up to batch limit.
        let mut drained = 0;
        while drained < max_batch {
            match self.completion_rx.try_recv() {
                Ok(completion) => {
                    drained += 1;
                    if let Some(target) = self.registry.complete(completion.token) {
                        events.push(ReactorCompletionEvent {
                            target,
                            outcome: completion.outcome,
                        });
                    }
                }
                Err(_) => break,
            }
        }

        events
    }

    /// Polls cross-thread completion ingress non-blockingly into internal buffers.
    /// Safe to call from safepoints without allocating Phalcom objects.
    pub fn poll_ingress_safepoint(&mut self, max_batch: usize) {
        let mut drained = 0;
        while drained < max_batch {
            match self.completion_rx.try_recv() {
                Ok(completion) => {
                    drained += 1;
                    self.pending_worker_completions.push(completion);
                }
                Err(_) => break,
            }
        }
    }

    /// Returns `true` if there is any pending external progress source (timers or workers).
    pub fn has_pending_progress(&self) -> bool {
        self.registry.has_active() || !self.pending_worker_completions.is_empty()
    }

    /// Returns the number of active reactor registrations.
    pub fn active_count(&self) -> usize {
        self.registry.active_count()
    }

    /// Blocks the current thread until the next timer deadline or until a worker completes.
    pub fn idle_wait(&mut self) {
        if !self.has_pending_progress() {
            return;
        }

        if !self.pending_worker_completions.is_empty() {
            return;
        }

        let earliest_timer = self.timers.peek_deadline();
        match earliest_timer {
            Some(deadline) => {
                let now = Instant::now();
                if now >= deadline {
                    return;
                }
                let timeout = deadline - now;
                if let Ok(completion) = self.completion_rx.recv_timeout(timeout) {
                    self.pending_worker_completions.push(completion);
                }
            }
            None => {
                // Only workers are pending. Wait with a safety timeout so we don't hang indefinitely.
                if let Ok(completion) = self.completion_rx.recv_timeout(Duration::from_millis(50)) {
                    self.pending_worker_completions.push(completion);
                }
            }
        }
    }

    /// Traces all completion targets for active registrations as GC roots.
    pub fn trace_roots(&self, tracer: &mut dyn FnMut(ObjRef)) {
        self.registry.trace_roots(tracer);
    }

    /// Cleanly shuts down the reactor, workers, and registrations.
    pub fn shutdown(&mut self) {
        self.registry.shutdown();
        self.timers.clear();
        self.worker_pool.shutdown();
        while self.completion_rx.try_recv().is_ok() {}
        self.pending_worker_completions.clear();
    }

    /// Generates diagnostic leak reports for any active registrations that were not completed.
    pub fn leak_report(&self) -> Vec<String> {
        let active = self.registry.active_entries();
        let mut reports = Vec::with_capacity(active.len());
        for (token, source, target) in active {
            reports.push(format!(
                "Unclosed reactor registration token: [slot: {}, gen: {}] source: {:?} target: {:?}",
                token.slot, token.generation, source, target
            ));
        }
        reports
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_registry_lifecycle_and_stale_drop() {
        let mut registry = ReactorRegistry::new();
        let obj1 = ObjRef::from_opaque_u64(100);
        let obj2 = ObjRef::from_opaque_u64(200);

        let token1 = registry.allocate(obj1, ReactorSource::Timer);
        assert_eq!(token1.slot, 0);
        assert_eq!(token1.generation, 1);
        assert!(registry.is_active(token1));
        assert_eq!(registry.active_count(), 1);

        // Completing token1 works
        assert_eq!(registry.complete(token1), Some(obj1));
        assert!(!registry.is_active(token1));
        assert_eq!(registry.active_count(), 0);

        // Double completion is a harmless no-op
        assert_eq!(registry.complete(token1), None);

        // Next allocation reuses slot 0 with incremented generation
        let token2 = registry.allocate(obj2, ReactorSource::Worker);
        assert_eq!(token2.slot, 0);
        assert_eq!(token2.generation, 2);
        assert!(registry.is_active(token2));
        assert_eq!(registry.active_count(), 1);

        // Stale token1 cannot complete slot 0
        assert_eq!(registry.complete(token1), None);
        assert!(registry.is_active(token2));

        // Cancelling token2
        assert!(registry.cancel(token2));
        assert!(!registry.is_active(token2));
        assert_eq!(registry.active_count(), 0);
    }

    #[test]
    fn test_registry_gc_root_tracing() {
        let mut registry = ReactorRegistry::new();
        let obj1 = ObjRef::from_opaque_u64(10);
        let obj2 = ObjRef::from_opaque_u64(20);

        let token1 = registry.allocate(obj1, ReactorSource::Timer);
        let token2 = registry.allocate(obj2, ReactorSource::Worker);

        let mut roots = Vec::new();
        registry.trace_roots(&mut |r| roots.push(r));
        assert_eq!(roots, vec![obj1, obj2]);

        // Completing token1 removes obj1 from roots
        registry.complete(token1);
        roots.clear();
        registry.trace_roots(&mut |r| roots.push(r));
        assert_eq!(roots, vec![obj2]);

        // Cancelling token2 removes obj2
        registry.cancel(token2);
        roots.clear();
        registry.trace_roots(&mut |r| roots.push(r));
        assert!(roots.is_empty());
    }

    #[test]
    fn test_timer_min_heap_ordering_and_tie_breaking() {
        let mut queue = TimerQueue::new();
        let token1 = ReactorToken { slot: 0, generation: 1 };
        let token2 = ReactorToken { slot: 1, generation: 1 };
        let token3 = ReactorToken { slot: 2, generation: 1 };

        let now = Instant::now();
        queue.schedule(token1, Duration::from_millis(50));
        queue.schedule(token2, Duration::from_millis(10));
        queue.schedule(token3, Duration::from_millis(10)); // same delay, higher sequence

        assert_eq!(queue.len(), 3);
        // Popping at now yields nothing
        assert!(queue.pop_due(now).is_empty());

        // Popping at now + 20ms yields token2, then token3
        let due = queue.pop_due(now + Duration::from_millis(20));
        assert_eq!(due, vec![token2, token3]);

        // Popping at now + 60ms yields token1
        let due = queue.pop_due(now + Duration::from_millis(60));
        assert_eq!(due, vec![token1]);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_worker_job_round_trip() {
        let mut reactor = Reactor::new(2);
        let obj = ObjRef::from_opaque_u64(42);

        let _token = reactor.register_worker_job(obj, || WorkerOutcome::Success(WorkerResult::Int(12345)));

        assert!(reactor.has_pending_progress());

        // Wait for worker to finish
        let start = Instant::now();
        let mut events = Vec::new();
        while events.is_empty() && start.elapsed() < Duration::from_secs(2) {
            reactor.idle_wait();
            events = reactor.drain_completions(16);
        }

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].target, obj);
        assert_eq!(events[0].outcome, WorkerOutcome::Success(WorkerResult::Int(12345)));
        assert!(!reactor.has_pending_progress());
    }

    #[test]
    fn test_shutdown_and_leak_reporting() {
        let mut reactor = Reactor::new(1);
        let obj = ObjRef::from_opaque_u64(99);

        let token = reactor.register_timer(obj, Duration::from_secs(60));
        assert_eq!(reactor.active_count(), 1);

        let leaks = reactor.leak_report();
        assert_eq!(leaks.len(), 1);
        assert!(leaks[0].contains("slot: 0"));

        reactor.shutdown();
        assert_eq!(reactor.active_count(), 0);
        assert!(!reactor.registry.is_active(token));
        assert!(reactor.leak_report().is_empty());
    }
}
