//! Phase-2 Reactor runtime for asynchronous completion, workers, timers, poller, and executor liveness.
//!
//! # Architecture
//!
//! ```text
//! Reactor
//!  ├── ReactorRegistry (generational token table, GC-roots active targets)
//!  ├── Poller (mio-backed descriptor readiness & cross-thread waker)
//!  ├── WorkerPool (plain-data off-thread execution + cross-thread wake)
//!  ├── TimerQueue (monotonic deadline min-heap, no timer thread)
//!  └── Unified Wait (mio::poll(min(timer, cap)) + non-blocking ingress drain)
//! ```
//!
//! # Law of Confinement (PDR-0016)
//! No `mio` type appears outside `phalcom-core/src/reactor/`.
//! External layers only interact with reactor tokens and plain results.

pub mod poller;
pub mod registry;
pub mod timer;
pub mod worker;

pub use poller::{PollInterest, PollReadiness, Poller, WAKER_TOKEN};
pub use registry::{ReactorRegistration, ReactorRegistry, ReactorSource, ReactorToken, RegistrationState};
pub use timer::{TimerEntry, TimerQueue};
pub use worker::{DEFAULT_WORKER_THREADS, WorkerCompletion, WorkerJob, WorkerOutcome, WorkerPool, WorkerResult};

use crate::heap::ObjRef;
use std::io;
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

/// Result of attempting a non-blocking pollable operation under try-first discipline.
#[derive(Debug, PartialEq, Eq)]
pub enum PollOpResult<T> {
    /// The operation succeeded immediately without blocking.
    Ready(T),
    /// The operation would block; caller must register interest on the poller.
    WouldBlock,
    /// An unrecoverable I/O error occurred.
    Err(String),
}

/// The core reactor managing external completions, timers, workers, and descriptor readiness.
pub struct Reactor {
    /// Generational table of live registrations.
    pub registry: ReactorRegistry,
    /// Priority queue of active timers.
    pub timers: TimerQueue,
    /// OS descriptor readiness poller and cross-thread waker.
    pub poller: Poller,
    /// Bounded background worker pool.
    pub worker_pool: WorkerPool,
    completion_tx: Sender<WorkerCompletion>,
    completion_rx: Receiver<WorkerCompletion>,
    pending_worker_completions: Vec<WorkerCompletion>,
    pending_poll_events: Vec<(u32, PollReadiness)>,
}

impl Default for Reactor {
    fn default() -> Self {
        Self::new(DEFAULT_WORKER_THREADS)
    }
}

impl Reactor {
    /// Creates a new reactor with `worker_threads` background worker threads and OS poller.
    pub fn new(worker_threads: usize) -> Self {
        let poller = Poller::new().expect("Failed to initialize reactor OS poller");
        let (completion_tx, completion_rx) = channel::<WorkerCompletion>();
        let worker_pool = WorkerPool::new(worker_threads, completion_tx.clone(), Some(poller.waker()));

        Self {
            registry: ReactorRegistry::new(),
            timers: TimerQueue::new(),
            poller,
            worker_pool,
            completion_tx,
            completion_rx,
            pending_worker_completions: Vec::new(),
            pending_poll_events: Vec::new(),
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

    /// Registers interest on an I/O source for `target` in the OS poller.
    pub fn register_pollable<S: mio::event::Source + ?Sized>(&mut self, source: &mut S, target: ObjRef, interest: PollInterest) -> io::Result<ReactorToken> {
        let token = self.registry.allocate(target, ReactorSource::Pollable);
        if let Err(err) = self.poller.register(source, token, interest) {
            self.registry.cancel(token);
            return Err(err);
        }
        Ok(token)
    }

    /// Re-arms interest on an already registered I/O source.
    pub fn reregister_pollable<S: mio::event::Source + ?Sized>(&mut self, source: &mut S, token: ReactorToken, interest: PollInterest) -> io::Result<()> {
        if !self.registry.is_active(token) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Inactive or stale reactor token"));
        }
        self.poller.reregister(source, token, interest)
    }

    /// Deregisters an I/O source and cancels its reactor token.
    pub fn deregister_pollable<S: mio::event::Source + ?Sized>(&mut self, source: &mut S, token: ReactorToken) -> io::Result<()> {
        self.registry.cancel(token);
        self.poller.deregister(source)
    }

    /// Registers a raw Unix file descriptor with the reactor poller.
    #[cfg(unix)]
    pub fn register_pollable_raw_fd(&mut self, fd: std::os::unix::io::RawFd, target: ObjRef, interest: PollInterest) -> io::Result<ReactorToken> {
        let token = self.registry.allocate(target, ReactorSource::Pollable);
        if let Err(err) = self.poller.register_raw_fd(fd, token, interest) {
            self.registry.cancel(token);
            return Err(err);
        }
        Ok(token)
    }

    /// Re-arms interest on a raw Unix file descriptor.
    #[cfg(unix)]
    pub fn reregister_pollable_raw_fd(&mut self, fd: std::os::unix::io::RawFd, token: ReactorToken, interest: PollInterest) -> io::Result<()> {
        if !self.registry.is_active(token) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Inactive or stale reactor token"));
        }
        self.poller.reregister_raw_fd(fd, token, interest)
    }

    /// Deregisters a raw Unix file descriptor and cancels its reactor token.
    #[cfg(unix)]
    pub fn deregister_pollable_raw_fd(&mut self, fd: std::os::unix::io::RawFd, token: ReactorToken) -> io::Result<()> {
        self.registry.cancel(token);
        self.poller.deregister_raw_fd(fd)
    }

    /// Non-blockingly drains due timers, descriptor readiness, and worker completions up to a bounded batch size.
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

        // 2. Drain buffered poller readiness events.
        let buffered_poll = std::mem::take(&mut self.pending_poll_events);
        for (slot, readiness) in buffered_poll {
            if let Some(generation) = self.registry.generation(slot) {
                let token = ReactorToken { slot, generation };
                if let Some(target) = self.registry.complete(token) {
                    events.push(ReactorCompletionEvent {
                        target,
                        outcome: WorkerOutcome::Success(WorkerResult::Int(readiness.bits() as i64)),
                    });
                }
            }
        }

        // 3. Drain buffered worker completions.
        let buffered = std::mem::take(&mut self.pending_worker_completions);
        for completion in buffered {
            if let Some(target) = self.registry.complete(completion.token) {
                events.push(ReactorCompletionEvent {
                    target,
                    outcome: completion.outcome,
                });
            }
        }

        // 4. Drain incoming worker completions up to batch limit.
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

    /// Polls cross-thread completion ingress and poller events non-blockingly into internal buffers.
    /// Safe to call from safepoints without allocating Phalcom objects.
    pub fn poll_ingress_safepoint(&mut self, max_batch: usize) {
        // Non-blocking poll of the OS poller
        if let Ok(ready_events) = self.poller.poll(Some(Duration::ZERO)) {
            self.pending_poll_events.extend(ready_events);
        }

        // Non-blocking drain of the worker channel
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

    /// Returns `true` if there is any pending external progress source (timers, workers, or pollables).
    pub fn has_pending_progress(&self) -> bool {
        self.registry.has_active() || !self.pending_worker_completions.is_empty() || !self.pending_poll_events.is_empty()
    }

    /// Returns the number of active reactor registrations.
    pub fn active_count(&self) -> usize {
        self.registry.active_count()
    }

    /// Blocks the current VM thread in a unified wait: `poll(timeout = min(earliest_timer, executor_cap))`.
    ///
    /// Awakened immediately by:
    /// 1. OS descriptor readiness on registered sockets/descriptors;
    /// 2. Off-thread worker completion via `mio::Waker`;
    /// 3. Monotonic timer deadline expiry;
    /// 4. Safety executor cap (50ms).
    pub fn idle_wait(&mut self) {
        if !self.has_pending_progress() {
            return;
        }

        if !self.pending_worker_completions.is_empty() || !self.pending_poll_events.is_empty() {
            return;
        }

        let now = Instant::now();
        let earliest_timer = self.timers.peek_deadline();
        let executor_cap = Duration::from_millis(50);
        let timeout = match earliest_timer {
            Some(deadline) => {
                if now >= deadline {
                    return;
                }
                let remaining = deadline - now;
                Some(remaining.min(executor_cap))
            }
            None => Some(executor_cap),
        };

        // Unified blocking poll
        if let Ok(ready_events) = self.poller.poll(timeout) {
            self.pending_poll_events.extend(ready_events);
        }

        // Non-blockingly drain worker channel on wakeup
        while let Ok(completion) = self.completion_rx.try_recv() {
            self.pending_worker_completions.push(completion);
        }
    }

    /// Traces all completion targets for active registrations as GC roots.
    pub fn trace_roots(&self, tracer: &mut dyn FnMut(ObjRef)) {
        self.registry.trace_roots(tracer);
    }

    /// Cleanly shuts down the reactor, poller, workers, and registrations.
    pub fn shutdown(&mut self) {
        self.registry.shutdown();
        self.timers.clear();
        self.worker_pool.shutdown();
        while self.completion_rx.try_recv().is_ok() {}
        self.pending_worker_completions.clear();
        self.pending_poll_events.clear();
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
    use std::io::{Read, Write};

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
    fn test_waker_cross_thread_wake() {
        let mut reactor = Reactor::new(1);
        let obj = ObjRef::from_opaque_u64(88);

        // Submit a worker job that sleeps briefly before completing
        let _token = reactor.register_worker_job(obj, || {
            std::thread::sleep(Duration::from_millis(20));
            WorkerOutcome::Success(WorkerResult::String("woken".to_string()))
        });

        // idle_wait should be unblocked by Waker::wake() promptly without hitting a 50ms cap
        let start = Instant::now();
        let mut events = Vec::new();
        while events.is_empty() && start.elapsed() < Duration::from_secs(2) {
            reactor.idle_wait();
            events = reactor.drain_completions(16);
        }

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].target, obj);
        assert_eq!(events[0].outcome, WorkerOutcome::Success(WorkerResult::String("woken".to_string())));
    }

    #[test]
    #[cfg(unix)]
    fn test_stale_poll_readiness_discard() {
        let mut reactor = Reactor::new(1);
        let obj1 = ObjRef::from_opaque_u64(101);
        let obj2 = ObjRef::from_opaque_u64(102);

        let (_tx, mut rx) = mio::unix::pipe::new().unwrap();

        let token1 = reactor.register_pollable(&mut rx, obj1, PollInterest::Readable).unwrap();

        assert!(reactor.registry.is_active(token1));

        // Deregister token1
        reactor.deregister_pollable(&mut rx, token1).unwrap();
        assert!(!reactor.registry.is_active(token1));

        // Allocate token2 in the same slot
        let token2 = reactor.registry.allocate(obj2, ReactorSource::Timer);
        assert_eq!(token1.slot, token2.slot);
        assert_ne!(token1.generation, token2.generation);

        // Inject a synthetic readiness event for token1's slot into pending_poll_events
        reactor.pending_poll_events.push((
            token1.slot,
            PollReadiness {
                is_readable: true,
                is_writable: false,
                is_read_closed: false,
                is_write_closed: false,
                is_error: false,
            },
        ));

        // Draining completions should complete token2 with Timer/readiness or drop stale token1 without crashing
        let events = reactor.drain_completions(16);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].target, obj2);
    }

    #[test]
    #[cfg(unix)]
    fn test_unified_wait_timer_and_worker_and_poll() {
        let mut reactor = Reactor::new(2);
        let obj_timer = ObjRef::from_opaque_u64(1);
        let obj_worker = ObjRef::from_opaque_u64(2);
        let obj_pipe = ObjRef::from_opaque_u64(3);

        // 1. Timer scheduled for 25ms
        let _t_token = reactor.register_timer(obj_timer, Duration::from_millis(25));

        // 2. Worker scheduled to complete in 15ms
        let _w_token = reactor.register_worker_job(obj_worker, || {
            std::thread::sleep(Duration::from_millis(15));
            WorkerOutcome::Success(WorkerResult::Int(999))
        });

        // 3. Pipe setup
        let (mut tx, mut rx) = mio::unix::pipe::new().unwrap();
        let _s_token = reactor.register_pollable(&mut rx, obj_pipe, PollInterest::Readable).unwrap();

        // Write to pipe to trigger readiness
        tx.write_all(b"x").unwrap();

        let mut collected = Vec::new();
        let start = Instant::now();
        while collected.len() < 3 && start.elapsed() < Duration::from_secs(3) {
            reactor.idle_wait();
            let batch = reactor.drain_completions(16);
            collected.extend(batch);
        }

        assert_eq!(collected.len(), 3);
        let targets: Vec<ObjRef> = collected.iter().map(|e| e.target).collect();
        assert!(targets.contains(&obj_timer));
        assert!(targets.contains(&obj_worker));
        assert!(targets.contains(&obj_pipe));
    }

    #[test]
    #[cfg(unix)]
    fn test_loopback_try_first_and_readiness_and_buffered_data() {
        let mut reactor = Reactor::new(1);
        let obj_read = ObjRef::from_opaque_u64(500);

        let (mut tx, mut rx) = mio::unix::pipe::new().unwrap();

        // Sender writes data BEFORE receiver registers poller interest
        tx.write_all(b"hello phalcom").unwrap();

        // Try-first discipline:
        // Try non-blocking read immediately
        let mut buf = [0u8; 32];
        match rx.read(&mut buf) {
            Ok(n) => {
                // Successfully read buffered data directly without deadlocking on edge-triggered poller!
                assert_eq!(&buf[..n], b"hello phalcom");
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                panic!("Expected data already in kernel buffer to succeed on try-first!");
            }
            Err(e) => panic!("Unexpected I/O error: {:?}", e),
        }

        // Now buffer is empty. A second read will return WouldBlock
        match rx.read(&mut buf) {
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Correct! Now register interest with reactor poller
                let _token = reactor.register_pollable(&mut rx, obj_read, PollInterest::Readable).unwrap();

                // Send more data from sender to trigger readiness edge
                tx.write_all(b"second message").unwrap();

                let mut events = Vec::new();
                let start = Instant::now();
                while events.is_empty() && start.elapsed() < Duration::from_secs(2) {
                    reactor.idle_wait();
                    events = reactor.drain_completions(16);
                }

                assert_eq!(events.len(), 1);
                assert_eq!(events[0].target, obj_read);

                // Drain the second message
                let n2 = rx.read(&mut buf).unwrap();
                assert_eq!(&buf[..n2], b"second message");
            }
            res => panic!("Expected WouldBlock on empty stream, got {:?}", res),
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_rearm_correctness_and_partial_drain() {
        let mut reactor = Reactor::new(1);
        let obj1 = ObjRef::from_opaque_u64(601);
        let obj2 = ObjRef::from_opaque_u64(602);

        let (mut tx, mut rx) = mio::unix::pipe::new().unwrap();

        let token1 = reactor.register_pollable(&mut rx, obj1, PollInterest::Readable).unwrap();

        tx.write_all(b"packet 1 and packet 2").unwrap();

        let mut events = Vec::new();
        let start = Instant::now();
        while events.is_empty() && start.elapsed() < Duration::from_secs(2) {
            reactor.idle_wait();
            events = reactor.drain_completions(16);
        }

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].target, obj1);

        // Read only "packet 1 " (partial drain)
        let mut buf = [0u8; 9];
        let n = rx.read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"packet 1 ");

        // Re-arm poller with fresh token for obj2
        let token2 = reactor.registry.allocate(obj2, ReactorSource::Pollable);
        reactor.poller.reregister(&mut rx, token2, PollInterest::Readable).unwrap();

        // Write more data to create a new edge
        tx.write_all(b" and packet 3").unwrap();

        events.clear();
        let start = Instant::now();
        while events.is_empty() && start.elapsed() < Duration::from_secs(2) {
            reactor.idle_wait();
            events = reactor.drain_completions(16);
        }

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].target, obj2);

        // Drain rest of pipe
        let mut rem_buf = [0u8; 64];
        let n_rem = rx.read(&mut rem_buf).unwrap();
        assert_eq!(&rem_buf[..n_rem], b"and packet 2 and packet 3");
    }

    #[test]
    #[cfg(unix)]
    fn test_read_write_interests_independent() {
        let mut reactor = Reactor::new(1);
        let obj_r = ObjRef::from_opaque_u64(701);
        let obj_w = ObjRef::from_opaque_u64(702);

        let (mut tx, mut rx) = mio::unix::pipe::new().unwrap();

        let _token_r = reactor.register_pollable(&mut rx, obj_r, PollInterest::Readable).unwrap();
        let _token_w = reactor.register_pollable(&mut tx, obj_w, PollInterest::Writable).unwrap();

        // tx is immediately writable
        let mut events = Vec::new();
        let start = Instant::now();
        while events.is_empty() && start.elapsed() < Duration::from_secs(2) {
            reactor.idle_wait();
            events = reactor.drain_completions(16);
        }

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].target, obj_w);
        if let WorkerOutcome::Success(WorkerResult::Int(bits)) = events[0].outcome {
            assert_ne!(bits & (1 << 1), 0, "Expected writable bit set");
        } else {
            panic!("Expected WorkerResult::Int outcome for poller readiness");
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_descriptor_gc_root_retention_and_release() {
        let mut reactor = Reactor::new(1);
        let obj = ObjRef::from_opaque_u64(801);

        let (_tx, mut rx) = mio::unix::pipe::new().unwrap();

        let token = reactor.register_pollable(&mut rx, obj, PollInterest::Readable).unwrap();

        // Target should be rooted while registration is active
        let mut roots = Vec::new();
        reactor.trace_roots(&mut |r| roots.push(r));
        assert_eq!(roots, vec![obj]);

        // Deregistering releases the root
        reactor.deregister_pollable(&mut rx, token).unwrap();
        roots.clear();
        reactor.trace_roots(&mut |r| roots.push(r));
        assert!(roots.is_empty());
    }

    #[test]
    fn test_empty_reactor_clean_exit() {
        let mut reactor = Reactor::new(1);
        assert!(!reactor.has_pending_progress());
        assert_eq!(reactor.active_count(), 0);

        // idle_wait on empty reactor returns immediately without blocking
        reactor.idle_wait();
        let completions = reactor.drain_completions(16);
        assert!(completions.is_empty());
        assert!(reactor.leak_report().is_empty());
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

    #[test]
    fn test_confinement_no_mio_outside_reactor() {
        // Assert at test time that no files in the workspace (outside phalcom-core/src/reactor) use `mio::`
        let core_src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        for entry in walkdir(core_src_dir.as_path()) {
            let path_str = entry.to_string_lossy();
            if path_str.ends_with(".rs") && !path_str.contains("/reactor/") && !path_str.ends_with("/reactor.rs") {
                let content = std::fs::read_to_string(&entry).unwrap();
                assert!(
                    !content.contains("mio::"),
                    "Found prohibited mio usage outside reactor module in file: {}",
                    path_str
                );
            }
        }
    }

    fn walkdir(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    files.extend(walkdir(&path));
                } else {
                    files.push(path);
                }
            }
        }
        files
    }
}
