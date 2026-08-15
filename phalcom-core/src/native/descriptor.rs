//! Runtime representation of native primitive descriptors and entries.

use crate::error::PhResult;
use crate::heap::ClassId;
use crate::method::{ArgumentView, CallOutcome, SignatureKind};
use crate::value::Value;
use crate::vm::VM;
use phalcom_common::selector::{Selector, SelectorKind, SelectorSlot};
use phalcom_native_meta::{NativeSourceSpec, NativeVisibility, PrimitiveAbi, PrimitiveSurfaceSpec};

/// Function pointer type for ordinary value-oriented primitives.
pub type PrimitiveValueFn = fn(&mut VM, &Value, &[Value]) -> PhResult<Value>;

/// Function pointer type for shape-aware gateway primitives.
pub type PrimitiveShapeFn = fn(&mut VM, Value, ArgumentView) -> PhResult<CallOutcome>;

/// Callable entry variant matching the primitive ABI.
#[derive(Clone, Copy, Debug)]
pub enum PrimitiveEntry {
    Value(PrimitiveValueFn),
    Shape(PrimitiveShapeFn),
}

/// Runtime descriptor contributed by native primitives.
#[derive(Clone, Copy, Debug)]
pub struct PrimitiveDescriptor {
    pub surface: &'static PrimitiveSurfaceSpec,
    pub abi: PrimitiveAbi,
    pub entry: PrimitiveEntry,
    pub source: NativeSourceSpec,
}

impl PrimitiveDescriptor {
    /// Computes the runtime signature for this descriptor.
    pub fn runtime_signature_kind(&self) -> SignatureKind {
        let selector = Selector::try_decode_exact(self.surface.key.selector).expect("descriptor selector must be valid and canonical");

        match selector.kind {
            SelectorKind::Getter => SignatureKind::Getter,
            SelectorKind::Setter => SignatureKind::Setter,
            SelectorKind::Method => {
                let positional = selector.slots.iter().filter(|s| **s == SelectorSlot::Positional).count() as u8;
                SignatureKind::Method(positional)
            }
            SelectorKind::SubscriptGet => {
                let positional = selector.slots.iter().filter(|s| **s == SelectorSlot::Positional).count() as u8;
                SignatureKind::SubscriptGet(positional)
            }
            SelectorKind::SubscriptSet => {
                let positional = selector.slots.iter().filter(|s| **s == SelectorSlot::Positional).count() as u8;
                SignatureKind::SubscriptSet(positional)
            }
        }
    }

    /// Computes the access owner for an internal primitive.
    pub fn internal_access_owner(&self, owner_class: ClassId) -> Option<ClassId> {
        match self.surface.visibility {
            NativeVisibility::Internal => Some(owner_class),
            NativeVisibility::Public => None,
        }
    }
}
