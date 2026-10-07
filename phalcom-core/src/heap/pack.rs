//! Immutable first-class Pack runtime storage with O(1) slicing and GC root tracking.
//!
//! Realizes LANG007.C4.P3 Task 3:
//! - Root owns a dedicated Pack product descriptor + ProductStorage.
//! - Slice holds an ObjRef to the root plus active positional and labeled ranges.
//! - Slice-of-slice normalizes directly to the original root with composed offsets.
//! - Trace visits ProductStorage for root, and only the root ObjRef for slice.

use crate::heap::ObjRef;
use crate::product::{ProductStorage, RuntimeAnonymousProductDescriptorId};

/// Internal storage mode for a Pack object.
#[derive(Debug, Clone, PartialEq)]
pub enum PackStorage {
    /// Root Pack owning the underlying product storage.
    Root {
        descriptor: RuntimeAnonymousProductDescriptorId,
        storage: ProductStorage,
    },
    /// O(1) slice sharing an existing root Pack's backing.
    Slice { root: ObjRef },
}

/// A native immutable first-class Pack object.
#[derive(Debug, Clone, PartialEq)]
pub struct PackObject {
    storage: PackStorage,
    positional_start: u32,
    positional_len: u32,
    labeled_start: u32,
    labeled_len: u32,
}

impl PackObject {
    /// Creates a new root Pack object.
    pub fn new_root(descriptor: RuntimeAnonymousProductDescriptorId, storage: ProductStorage, positional_len: u32, labeled_len: u32) -> Self {
        Self {
            storage: PackStorage::Root { descriptor, storage },
            positional_start: 0,
            positional_len,
            labeled_start: 0,
            labeled_len,
        }
    }

    /// Creates an O(1) slice of an existing root Pack object.
    pub fn new_slice(root: ObjRef, positional_start: u32, positional_len: u32, labeled_start: u32, labeled_len: u32) -> Self {
        Self {
            storage: PackStorage::Slice { root },
            positional_start,
            positional_len,
            labeled_start,
            labeled_len,
        }
    }

    /// Returns the internal storage mode of this Pack.
    pub fn storage(&self) -> &PackStorage {
        &self.storage
    }

    /// Returns the active positional start offset relative to root.
    pub fn positional_start(&self) -> u32 {
        self.positional_start
    }

    /// Returns the visible positional count.
    pub fn positional_len(&self) -> u32 {
        self.positional_len
    }

    /// Returns the active labeled start offset relative to root.
    pub fn labeled_start(&self) -> u32 {
        self.labeled_start
    }

    /// Returns the visible labeled count.
    pub fn labeled_len(&self) -> u32 {
        self.labeled_len
    }

    /// Returns the total visible component count.
    pub fn total_len(&self) -> u32 {
        self.positional_len + self.labeled_len
    }

    /// Returns true if this Pack has no visible components in either lane.
    pub fn is_empty(&self) -> bool {
        self.total_len() == 0
    }

    /// If this is a root, returns its product descriptor ID.
    pub fn root_descriptor(&self) -> Option<RuntimeAnonymousProductDescriptorId> {
        match &self.storage {
            PackStorage::Root { descriptor, .. } => Some(*descriptor),
            PackStorage::Slice { .. } => None,
        }
    }

    /// If this is a root, returns its product storage.
    pub fn root_storage(&self) -> Option<&ProductStorage> {
        match &self.storage {
            PackStorage::Root { storage, .. } => Some(storage),
            PackStorage::Slice { .. } => None,
        }
    }

    /// If this is a slice, returns the root ObjRef.
    pub fn slice_root(&self) -> Option<ObjRef> {
        match &self.storage {
            PackStorage::Slice { root } => Some(*root),
            PackStorage::Root { .. } => None,
        }
    }
}
