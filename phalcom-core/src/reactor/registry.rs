//! Reactor token registry and generational registration tracking.
//!
//! Maps generational [`ReactorToken`]s to Phalcom completion targets ([`ObjRef`]),
//! ensuring O(1) lookups, generation-checked invalidation/stale completion drops,
//! and accurate GC root tracking while registrations are active.

use crate::heap::ObjRef;

/// A generation-tagged token representing an active or completed reactor registration.
///
/// Distinct from Fiber park generations and scheduler admission identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReactorToken {
    /// Slot index in the registry.
    pub slot: u32,
    /// Generation of the slot at allocation time.
    pub generation: u32,
}

/// The origin/kind of an asynchronous reactor registration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReactorSource {
    /// Monotonic timer registration.
    Timer,
    /// Background worker pool computation or I/O.
    Worker,
}

/// Lifecycle state of a reactor registration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistrationState {
    /// Registration is live and awaiting completion. Its target is GC-rooted.
    Active,
    /// Registration has been completed.
    Completed,
    /// Registration was cancelled or invalidated.
    Cancelled,
}

/// An entry in the reactor registration table.
#[derive(Clone, Debug)]
pub struct ReactorRegistration {
    pub generation: u32,
    pub state: RegistrationState,
    pub target: ObjRef,
    pub source: ReactorSource,
}

/// Generational table of reactor registrations.
#[derive(Debug, Default)]
pub struct ReactorRegistry {
    entries: Vec<Option<ReactorRegistration>>,
    generations: Vec<u32>,
    free_slots: Vec<u32>,
    active_count: usize,
}

impl ReactorRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            generations: Vec::new(),
            free_slots: Vec::new(),
            active_count: 0,
        }
    }

    /// Allocates a new registration slot for `target` from `source`.
    ///
    /// The target will be rooted by GC until this registration is completed,
    /// cancelled, or the registry is shut down.
    pub fn allocate(&mut self, target: ObjRef, source: ReactorSource) -> ReactorToken {
        if let Some(slot) = self.free_slots.pop() {
            let slot_idx = slot as usize;
            self.generations[slot_idx] = self.generations[slot_idx].wrapping_add(1);
            let generation = self.generations[slot_idx];
            self.entries[slot_idx] = Some(ReactorRegistration {
                generation,
                state: RegistrationState::Active,
                target,
                source,
            });
            self.active_count += 1;
            ReactorToken { slot, generation }
        } else {
            let slot = self.entries.len() as u32;
            let generation = 1;
            self.generations.push(generation);
            self.entries.push(Some(ReactorRegistration {
                generation,
                state: RegistrationState::Active,
                target,
                source,
            }));
            self.active_count += 1;
            ReactorToken { slot, generation }
        }
    }

    /// Attempts to complete a registration by its token.
    ///
    /// If the token is valid, matches the current generation, and is in `Active` state:
    /// - Marks the registration as `Completed`;
    /// - Releases the slot to the free list;
    /// - Decrements active count;
    /// - Returns `Some(target)`.
    ///
    /// Stale, invalid, or already-completed tokens return `None` (settle-once).
    pub fn complete(&mut self, token: ReactorToken) -> Option<ObjRef> {
        let slot_idx = token.slot as usize;
        if slot_idx >= self.entries.len() {
            return None;
        }
        let entry = self.entries[slot_idx].as_mut()?;
        if entry.generation != token.generation || entry.state != RegistrationState::Active {
            return None;
        }

        entry.state = RegistrationState::Completed;
        let target = entry.target;
        self.active_count = self.active_count.saturating_sub(1);
        self.free_slots.push(token.slot);
        Some(target)
    }

    /// Cancels / invalidates a live registration.
    ///
    /// If active and matching, transitions to `Cancelled` and frees the slot.
    pub fn cancel(&mut self, token: ReactorToken) -> bool {
        let slot_idx = token.slot as usize;
        if slot_idx >= self.entries.len() {
            return false;
        }
        let Some(entry) = self.entries[slot_idx].as_mut() else {
            return false;
        };
        if entry.generation != token.generation || entry.state != RegistrationState::Active {
            return false;
        }

        entry.state = RegistrationState::Cancelled;
        self.active_count = self.active_count.saturating_sub(1);
        self.free_slots.push(token.slot);
        true
    }

    /// Checks if a token is currently active.
    pub fn is_active(&self, token: ReactorToken) -> bool {
        let slot_idx = token.slot as usize;
        if slot_idx >= self.entries.len() {
            return false;
        }
        match &self.entries[slot_idx] {
            Some(entry) => entry.generation == token.generation && entry.state == RegistrationState::Active,
            None => false,
        }
    }

    /// Returns the number of currently active registrations.
    pub fn active_count(&self) -> usize {
        self.active_count
    }

    /// Returns `true` if there are any live, active registrations pending progress.
    pub fn has_active(&self) -> bool {
        self.active_count > 0
    }

    /// Traces all completion targets for active registrations as GC roots.
    pub fn trace_roots(&self, tracer: &mut dyn FnMut(ObjRef)) {
        for entry in self.entries.iter().flatten() {
            if entry.state == RegistrationState::Active {
                tracer(entry.target);
            }
        }
    }

    /// Invalidate all active registrations and reset active counts (used on shutdown).
    pub fn shutdown(&mut self) {
        for (slot, entry) in self.entries.iter_mut().enumerate() {
            if let Some(e) = entry {
                if e.state == RegistrationState::Active {
                    e.state = RegistrationState::Cancelled;
                    self.free_slots.push(slot as u32);
                }
            }
        }
        self.active_count = 0;
    }

    /// Returns diagnostic information for any unclosed/active registrations.
    pub fn active_entries(&self) -> Vec<(ReactorToken, ReactorSource, ObjRef)> {
        let mut list = Vec::new();
        for (slot, entry) in self.entries.iter().enumerate() {
            if let Some(e) = entry {
                if e.state == RegistrationState::Active {
                    list.push((
                        ReactorToken {
                            slot: slot as u32,
                            generation: e.generation,
                        },
                        e.source,
                        e.target,
                    ));
                }
            }
        }
        list
    }
}
