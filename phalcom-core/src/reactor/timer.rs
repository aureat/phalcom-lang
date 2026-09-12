//! Monotonic timer queue and deadline management.
//!
//! Timers are stored in a min-heap ordered by monotonic deadline and insertion
//! sequence, driven directly by the VM executor's safepoint ingress and idle-wait
//! loop without requiring a dedicated timer thread.

use super::registry::ReactorToken;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::{Duration, Instant};

/// A single scheduled timer entry in the priority queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimerEntry {
    /// Monotonic deadline at which this timer expires.
    pub deadline: Instant,
    /// Monotonic sequence number ensuring deterministic tie-breaking for equal deadlines.
    pub sequence: u64,
    /// Reactor registration token to complete on expiration.
    pub token: ReactorToken,
}

impl Ord for TimerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max-heap in Rust, so reverse the comparison for a min-heap:
        // Earlier deadline comes first (greater priority).
        other.deadline.cmp(&self.deadline).then_with(|| other.sequence.cmp(&self.sequence))
    }
}

impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Min-heap of active monotonic timers.
#[derive(Debug, Default)]
pub struct TimerQueue {
    heap: BinaryHeap<TimerEntry>,
    next_sequence: u64,
}

impl TimerQueue {
    /// Creates an empty timer queue.
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            next_sequence: 0,
        }
    }

    /// Schedules a timer to expire after `duration` from `Instant::now()`.
    pub fn schedule(&mut self, token: ReactorToken, duration: Duration) {
        let deadline = Instant::now() + duration;
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);
        self.heap.push(TimerEntry { deadline, sequence, token });
    }

    /// Returns the monotonic deadline of the earliest scheduled timer, if any.
    pub fn peek_deadline(&self) -> Option<Instant> {
        self.heap.peek().map(|entry| entry.deadline)
    }

    /// Pops and returns all timer tokens whose deadline has passed by `now`.
    pub fn pop_due(&mut self, now: Instant) -> Vec<ReactorToken> {
        let mut due = Vec::new();
        while let Some(entry) = self.heap.peek() {
            if entry.deadline <= now {
                let entry = self.heap.pop().unwrap();
                due.push(entry.token);
            } else {
                break;
            }
        }
        due
    }

    /// Returns the number of scheduled timers in the queue.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Returns `true` if no timers are scheduled.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Clears all timers from the queue.
    pub fn clear(&mut self) {
        self.heap.clear();
    }
}
