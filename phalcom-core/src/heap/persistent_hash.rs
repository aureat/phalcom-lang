//! Reusable GC-visible persistent hash trie node representations and pure indexing helpers.
//!
//! Realizes the private node model for persistent collections (INV-04, INV-09, INV-11, INV-12).
//! Nodes are immutable once published in the heap and participate in standard mark-sweep GC tracing.

use crate::heap::ObjRef;
use crate::value::Value;

pub const BITS: u32 = 5;
pub const MASK: u64 = 0x1f;
pub const MAX_SHIFT: u32 = 60;

/// Extracts the 5-bit fragment for the given trie depth (shift).
#[inline]
pub fn fragment(hash: i64, shift: u32) -> u32 {
    (((hash as u64) >> shift) & MASK) as u32
}

/// Computes the single-bit mask corresponding to a 5-bit fragment.
#[inline]
pub fn child_bit(fragment: u32) -> u32 {
    1u32 << fragment
}

/// Computes the index of a child in the dense `children` slice for a given bit.
#[inline]
pub fn compact_index(bitmap: u32, bit: u32) -> usize {
    (bitmap & (bit - 1)).count_ones() as usize
}

/// A node in the 32-way bitmap-indexed persistent hash trie.
#[derive(Debug, Clone)]
pub enum PersistentHashNode {
    /// A terminal node holding a single key-payload pair.
    Leaf { hash: i64, key: Value, payload: Value },

    /// A terminal node holding multiple key-payload pairs that share the exact same hash code.
    Collision { hash: i64, entries: Box<[(Value, Value)]> },

    /// An internal node branching on 5 bits of hash code via a 32-bit popcount bitmap.
    Branch { bitmap: u32, children: Box<[ObjRef]> },
}

impl PersistentHashNode {
    /// Constructs a new Leaf node.
    pub fn new_leaf(hash: i64, key: Value, payload: Value) -> Self {
        Self::Leaf { hash, key, payload }
    }

    /// Constructs a new Collision node.
    pub fn new_collision(hash: i64, entries: Box<[(Value, Value)]>) -> Self {
        debug_assert!(entries.len() >= 2, "Collision node must contain at least 2 entries");
        Self::Collision { hash, entries }
    }

    /// Constructs a new Branch node, verifying INV-09.
    pub fn new_branch(bitmap: u32, children: Box<[ObjRef]>) -> Self {
        debug_assert_eq!(
            bitmap.count_ones() as usize,
            children.len(),
            "INV-09: Branch bitmap popcount must match children count"
        );
        Self::Branch { bitmap, children }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf { .. })
    }

    pub fn is_collision(&self) -> bool {
        matches!(self, Self::Collision { .. })
    }

    pub fn is_branch(&self) -> bool {
        matches!(self, Self::Branch { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heap::{Heap, Object};
    use crate::value::Value;

    #[test]
    fn trie_indexing_and_bitmap_helpers() {
        assert_eq!(fragment(0b00000, 0), 0);
        assert_eq!(fragment(0b11111, 0), 31);
        assert_eq!(fragment(0b00001_00000, 5), 1);
        assert_eq!(fragment(0b10101_00000, 5), 21);

        assert_eq!(child_bit(0), 1);
        assert_eq!(child_bit(5), 32);
        assert_eq!(child_bit(31), 1 << 31);

        let bitmap = (1 << 2) | (1 << 7) | (1 << 15);
        assert_eq!(compact_index(bitmap, 1 << 2), 0);
        assert_eq!(compact_index(bitmap, 1 << 7), 1);
        assert_eq!(compact_index(bitmap, 1 << 15), 2);
    }

    #[test]
    fn object_slot_size_is_unaltered() {
        assert_eq!(
            std::mem::size_of::<Object>(),
            48,
            "INV-12: Object arena slot size must remain 48 bytes with PersistentHashNode"
        );
    }

    #[test]
    fn persistent_node_allocation_and_accessors() {
        let mut heap = Heap::new();

        let leaf = PersistentHashNode::new_leaf(12345, Value::int(1), Value::int(100));
        let leaf_ref = heap.alloc_persistent_hash_node(leaf);

        assert!(heap.as_persistent_hash_node(leaf_ref).is_some());
        let node = heap.persistent_hash_node(leaf_ref);
        assert!(matches!(node, PersistentHashNode::Leaf { hash: 12345, .. }));

        let branch = PersistentHashNode::new_branch(1 << 2, Box::new([leaf_ref]));
        let branch_ref = heap.alloc_persistent_hash_node(branch);
        let b_node = heap.persistent_hash_node(branch_ref);
        assert!(matches!(b_node, PersistentHashNode::Branch { bitmap, .. } if *bitmap == (1 << 2)));
    }
}
