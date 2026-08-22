//! Heap representations for typing context and descriptor objects.

use super::{ClassId, ObjRef};
use crate::typing::context::TypingContextData;
use crate::typing::handle::RuntimeSemanticHandle;

/// Boxed typing object payload: avoids inflating every slotmap arena slot.
#[derive(Debug)]
pub struct TypingObject {
    pub class: ClassId,
    pub payload: TypingPayload,
}

#[derive(Debug)]
pub enum TypingPayload {
    Context(TypingContextData),
    Descriptor { context: ObjRef, handle: RuntimeSemanticHandle },
}
