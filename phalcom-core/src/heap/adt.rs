//! ADT Case Heap Object (Part 4).

use crate::adt::RuntimeVariantId;
use crate::value::Value;

/// Heap object representing a fresh constructor case result with immutable payload values.
#[derive(Clone, Debug)]
pub struct AdtCaseObject {
    pub variant: RuntimeVariantId,
    pub payload: Box<[Value]>,
}

impl AdtCaseObject {
    pub fn new(variant: RuntimeVariantId, payload: Box<[Value]>) -> Self {
        Self { variant, payload }
    }
}
