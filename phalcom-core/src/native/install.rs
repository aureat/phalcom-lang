//! Deterministic installation of native descriptors into the VM class hierarchy.

use super::descriptor::{PrimitiveDescriptor, PrimitiveEntry};
use super::registry::PRIMITIVES;
use super::source::NativeSourceIndex;
use crate::error::PhResult;
use crate::heap::Object;
use crate::method::MethodObject;
use crate::vm::VM;
use phalcom_common::selector::{Selector, SelectorBase};
use phalcom_native_meta::NativeDispatch;

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

/// Rebinds installed native methods to the exact source module that declares
/// their canonical native anchor. Primitive installation happens before the
/// source Universe modules are materialized, so the first pass can only build
/// the method objects; bootstrap calls this reconciliation pass once those
/// module handles exist.
pub fn refresh_registered_primitive_origins(vm: &mut VM, source_index: &NativeSourceIndex) -> PhResult<()> {
    for desc in PRIMITIVES {
        let Some(anchor) = source_index.members.get(&super::verify::source_key(desc.surface.key)) else {
            continue;
        };
        let module = vm
            .module_registry
            .get(&anchor.module)
            .map(|record| record.object)
            .ok_or_else(|| crate::error::PhError::StringError(format!("native declaration module {} is not materialized", anchor.module)))?;
        let owner = vm.universe.classes.resolve(desc.surface.key.owner);
        let target = match desc.surface.key.side {
            NativeDispatch::Instance => owner,
            NativeDispatch::Class => vm.heap.class(owner).class,
        };
        let selector = vm.interner.intern(desc.surface.key.selector);
        let method = vm
            .heap
            .class(target)
            .get_method(selector)
            .ok_or_else(|| crate::error::PhError::StringError(format!("installed native method {} is missing", desc.surface.key.selector)))?;
        vm.heap.method_mut(method).origin = vm.access_origin_for_module(module);
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

    method.visibility = desc.surface.visibility;
    method.access_owner = desc.internal_access_owner(owner);
    // Native descriptors are defined by the canonical Universe project. Keep
    // that origin attached to the installed method so `internal` visibility
    // is authorized by project identity rather than by a selector prefix or
    // an ambient boolean privilege.
    method.origin = crate::method::RuntimeAccessOrigin::universe();

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
            kind: phalcom_native_meta::ImplementationProvenance::NativePrimitive,
            key: Some(desc.surface.key),
            intrinsic: desc.surface.intrinsic,
            abi: Some(desc.abi),
            source: Some(desc.source),
        },
    );
    vm.world_version += 1;

    Ok(())
}
