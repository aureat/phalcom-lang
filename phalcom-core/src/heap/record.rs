//! Immutable positive-arity record storage.

use crate::product::{ProductStorage, RuntimeAnonymousProductDescriptorId};

#[derive(Debug, Clone, PartialEq)]
pub struct RecordObject {
    descriptor: RuntimeAnonymousProductDescriptorId,
    storage: ProductStorage,
}

impl RecordObject {
    pub(crate) fn new(descriptor: RuntimeAnonymousProductDescriptorId, storage: ProductStorage) -> Self {
        Self { descriptor, storage }
    }

    pub fn descriptor(&self) -> RuntimeAnonymousProductDescriptorId {
        self.descriptor
    }

    pub fn storage(&self) -> &ProductStorage {
        &self.storage
    }
}
