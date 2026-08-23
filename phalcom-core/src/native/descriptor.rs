//! Runtime representation of native primitive descriptors and entries.

use crate::error::PhResult;
use crate::heap::ClassId;
use crate::method::{ArgumentView, CallOutcome, RestLayout, RestMode, Signature, SignatureKind};
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
                let positional = self.fixed_positional_count(&selector);
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

    /// Builds runtime signature metadata, including the complete rest family
    /// used by shape gateways such as `Function#call(***)`.
    pub fn runtime_signature(&self, vm: &mut VM, selector: crate::interner::Symbol) -> Signature {
        let kind = self.runtime_signature_kind();
        let parsed = Selector::try_decode_exact(self.surface.key.selector).expect("descriptor selector must be valid");
        if !self.surface.key.selector.contains("***") {
            return Signature::new(selector, kind);
        }

        let fixed_positionals = self.fixed_positional_count(&parsed);
        let marker_index = parsed.slots.len().saturating_sub(1);
        let fixed_labels = parsed.slots[..marker_index]
            .iter()
            .filter_map(|slot| match slot {
                SelectorSlot::Label(label) => Some(vm.interner.intern(label)),
                SelectorSlot::Positional => None,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Signature::new_with_arity(
            selector,
            kind,
            fixed_positionals,
            Some(RestLayout::new(fixed_positionals, fixed_labels, RestMode::Complete { param_index: 0 })),
        )
    }

    fn fixed_positional_count(&self, selector: &Selector) -> u8 {
        let slots = if self.surface.key.selector.contains("***") {
            &selector.slots[..selector.slots.len().saturating_sub(1)]
        } else {
            &selector.slots
        };
        slots.iter().filter(|slot| **slot == SelectorSlot::Positional).count() as u8
    }

    /// Computes the access owner for an internal primitive.
    pub fn internal_access_owner(&self, owner_class: ClassId) -> Option<ClassId> {
        match self.surface.visibility {
            NativeVisibility::Internal => Some(owner_class),
            NativeVisibility::Public => None,
        }
    }
}
