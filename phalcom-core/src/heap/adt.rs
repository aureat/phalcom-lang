//! ADT Case Heap Object (Part 4).

use crate::adt::RuntimeVariantId;
use crate::product::ProductStorage;

/// Heap object representing a fresh constructor case result with immutable payload values.
#[derive(Clone, Debug)]
pub struct AdtCaseObject {
    pub variant: RuntimeVariantId,
    pub storage: ProductStorage,
}

impl AdtCaseObject {
    pub fn new(variant: RuntimeVariantId, storage: ProductStorage) -> Self {
        Self { variant, storage }
    }
}
