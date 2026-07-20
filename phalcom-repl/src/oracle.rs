//! Two-source query oracle combining live VM snapshot and static LSP indexing.
//!
//! Provides two-source lookup: live VM snapshot takes precedence for executed
//! cell bindings and runtime class method dictionaries, while static parsing
//! covers the current unexecuted line.

use crate::snapshot::{Member, ReplSnapshot};
use phalcom_core::heap::ClassId;
use std::sync::{Arc, Mutex};

/// Query interface for completion, hints, and syntax highlighting.
pub struct ReplOracle {
    snapshot: Arc<Mutex<ReplSnapshot>>,
}

impl ReplOracle {
    /// Creates a new [`ReplOracle`] backed by a shared snapshot cell.
    pub fn new(snapshot: Arc<Mutex<ReplSnapshot>>) -> Self {
        Self { snapshot }
    }

    /// Returns members for a given [`ClassId`] sorted by `(own_depth, name)`.
    pub fn members_for_class(&self, class_id: ClassId) -> Vec<Member> {
        let snap = self.snapshot.lock().unwrap_or_else(|e| e.into_inner());
        let mut list = snap.members.get(&class_id).cloned().unwrap_or_default();
        list.sort_by(|a, b| a.own_depth.cmp(&b.own_depth).then_with(|| a.name.cmp(&b.name)));
        list
    }

    /// Looks up the class handle for a global variable name bound in the snapshot.
    pub fn global_name_class(&self, name: &str) -> Option<ClassId> {
        let snap = self.snapshot.lock().unwrap_or_else(|e| e.into_inner());
        snap.global_names.get(name).copied()
    }

    /// Looks up a class handle for a class named `name`.
    pub fn find_class_by_name(&self, name: &str) -> Option<ClassId> {
        let snap = self.snapshot.lock().unwrap_or_else(|e| e.into_inner());
        snap.class_names.get(name).copied()
    }

    /// Checks if a global variable name is defined in the snapshot.
    pub fn is_global_name_bound(&self, name: &str) -> bool {
        let snap = self.snapshot.lock().unwrap_or_else(|e| e.into_inner());
        snap.global_names.contains_key(name)
    }

    /// Returns a copy of the current snapshot.
    pub fn snapshot(&self) -> ReplSnapshot {
        self.snapshot.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}
