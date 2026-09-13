//! Heap-allocated storage for positive-arity data objects.

use crate::data::RuntimeDataDescriptorId;
use crate::product::ProductStorage;

/// Heap payload for a materialized, positive-arity `data` instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataObject {
    pub descriptor: RuntimeDataDescriptorId,
    pub storage: ProductStorage,
}

impl DataObject {
    pub fn new(descriptor: RuntimeDataDescriptorId, storage: ProductStorage) -> Self {
        Self {
            descriptor,
            storage,
        }
    }
}
