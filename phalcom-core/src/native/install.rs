//! Deterministic installation of native descriptors into the VM class hierarchy.

use super::descriptor::{PrimitiveDescriptor, PrimitiveEntry};
use super::registry::PRIMITIVES;
use crate::error::PhResult;
use crate::heap::Object;
use crate::method::{MemberVisibility, MethodObject};
use crate::vm::VM;
use phalcom_common::selector::{Selector, SelectorBase};
use phalcom_native_meta::{NativeDispatch, NativeVisibility};

/// Sorts registered descriptors by key and installs each into the VM.
pub fn install_registered_primitives(vm: &mut VM) -> PhResult<()> {
    super::registry::validate_registry();
    let mut descriptors: Vec<&'static PrimitiveDescriptor> = PRIMITIVES.iter().collect();
    descriptors.sort_by_key(|d| d.surface.key.sort_key());

    for desc in descriptors {
        install_one(vm, desc)?;
    }

    Ok(())
}

fn install_one(vm: &mut VM, desc: &PrimitiveDescriptor) -> PhResult<()> {
    let owner = vm.universe.classes.resolve(desc.surface.key.owner);

    let target = match desc.surface.key.side {
        NativeDispatch::Instance => owner,
        NativeDispatch::Class => vm.heap.class(owner).class,
    };

    let selector = vm.interner.intern(desc.surface.key.selector);
    let signature = desc.runtime_signature(vm, selector);
    let sig_kind = signature.kind;
    let replaced = vm.heap.class(target).get_method(selector);

    let mut method = match desc.entry {
        PrimitiveEntry::Value(f) => MethodObject::new_primitive(selector, sig_kind, f, owner),
        PrimitiveEntry::Shape(f) => MethodObject::new_shape_primitive(selector, signature, f, owner),
    };

    method.visibility = match desc.surface.visibility {
        NativeVisibility::Public => MemberVisibility::Public,
        NativeVisibility::Internal => MemberVisibility::Internal,
    };
    method.access_owner = desc.internal_access_owner(owner);

    let method_id = vm.heap.alloc(Object::Method(Box::new(method)));
    vm.heap.class_mut(target).add_method(selector, method_id);
    if desc.surface.key.selector.contains("***") {
        let parsed = Selector::try_decode_exact(desc.surface.key.selector).expect("descriptor selector must be valid");
        let SelectorBase::Named(base) = parsed.base else {
            panic!("rest descriptor must use named selector base");
        };
        vm.heap.class_mut(target).add_rest_method(vm.interner.intern(&base), method_id);
    }
    if let Some(replaced) = replaced {
        vm.typing_registry.method_implementations.remove(replaced);
    }
    vm.typing_registry.method_implementations.insert(
        method_id,
        crate::typing::side_table::RuntimeImplementationRef {
            kind: phalcom_native_meta::ImplementationKind::NativePrimitive,
            key: Some(desc.surface.key),
            intrinsic: desc.surface.intrinsic,
            abi: Some(desc.abi),
            source: Some(desc.source),
        },
    );
    vm.world_version += 1;

    Ok(())
}
