//! Heap storage for the persistent immutable multiset (`Bag<T>`).

use crate::heap::ObjRef;
use num_bigint::BigInt;
use num_traits::Zero;

/// Persistent multiset representation.
///
/// Holds an optional root to a `PersistentHashNode` HAMT, the exact total occurrence count
/// (`total_count`, represented as an unbounded BigInt), and the number of distinct logical keys
/// (`distinct_count`).
#[derive(Clone, Debug)]
pub struct BagObject {
    root: Option<ObjRef>,
    total_count: BigInt,
    distinct_count: usize,
}

impl BagObject {
    pub fn new() -> Self {
        Self {
            root: None,
            total_count: BigInt::zero(),
            distinct_count: 0,
        }
    }

    pub fn with_root(root: Option<ObjRef>, total_count: BigInt, distinct_count: usize) -> Self {
        Self {
            root,
            total_count,
            distinct_count,
        }
    }

    pub fn root(&self) -> Option<ObjRef> {
        self.root
    }

    pub fn total_count(&self) -> &BigInt {
        &self.total_count
    }

    pub fn distinct_count(&self) -> usize {
        self.distinct_count
    }

    pub fn is_empty(&self) -> bool {
        self.distinct_count == 0
    }
}

impl Default for BagObject {
    fn default() -> Self {
        Self::new()
    }
}
