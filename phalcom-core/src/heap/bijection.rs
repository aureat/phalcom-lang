//! Heap storage for the persistent bidirectional map (`Bijection<K, V>`).

use crate::heap::ObjRef;

/// Persistent bidirectional map representation.
///
/// Holds dual roots to two `PersistentHashNode` HAMTs (one mapping keys to values,
/// one mapping values to keys) and the cardinality `len`.
#[derive(Clone, Debug)]
pub struct BijectionObject {
    forward_root: Option<ObjRef>,
    reverse_root: Option<ObjRef>,
    len: usize,
}

impl BijectionObject {
    pub fn new() -> Self {
        Self {
            forward_root: None,
            reverse_root: None,
            len: 0,
        }
    }

    pub fn with_roots(forward_root: Option<ObjRef>, reverse_root: Option<ObjRef>, len: usize) -> Self {
        Self {
            forward_root,
            reverse_root,
            len,
        }
    }

    pub fn forward_root(&self) -> Option<ObjRef> {
        self.forward_root
    }

    pub fn reverse_root(&self) -> Option<ObjRef> {
        self.reverse_root
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Default for BijectionObject {
    fn default() -> Self {
        Self::new()
    }
}
