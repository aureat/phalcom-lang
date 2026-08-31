//! Associated Family Heap Object (Part 4).

use crate::modules::semantic_lowering::ExecutableFamilyDescriptor;
use crate::value::Value;
use std::sync::Arc;

/// Heap object representing a reified first-class associated family capability.
#[derive(Clone, Debug)]
pub struct AssociatedFamilyObject {
    pub descriptor: Arc<ExecutableFamilyDescriptor>,
    pub bound_owner: Option<Value>,
}

impl AssociatedFamilyObject {
    pub fn new(descriptor: Arc<ExecutableFamilyDescriptor>, bound_owner: Option<Value>) -> Self {
        Self { descriptor, bound_owner }
    }
}
