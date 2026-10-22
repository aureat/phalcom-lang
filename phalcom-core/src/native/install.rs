//! Deterministic installation of native descriptors into the VM class hierarchy.

use super::descriptor::{PrimitiveDescriptor, PrimitiveEntry};
use super::registry::PRIMITIVES;
use super::source::NativeSourceIndex;
use crate::error::PhResult;
use crate::heap::Object;
use crate::method::MethodObject;
use crate::vm::VM;
use phalcom_common::selector::{Selector, SelectorBase};
use phalcom_native_meta::{NativeDispatch, UNIVERSE_BINDINGS, UniverseKey};

/// Sorts registered descriptors by key and installs each into the VM.
pub fn install_registered_primitives(vm: &mut VM) -> PhResult<()> {
    super::registry::validate_registry();
    let mut descriptors: Vec<&'static PrimitiveDescriptor> = PRIMITIVES.iter().collect();
    descriptors.sort_by_key(|d| d.surface.key.sort_key());

    for desc in descriptors {
        // Primordial classes exist before source Universe modules run. Native
        // descriptors for ordinary source-owned classes are installed by
        // `install_source_owned_primitives` after those declarations have
        // materialized, using the same descriptor and provenance path.
        if is_primordial_owner(desc.surface.key.owner) {
            install_one(vm, desc)?;
        }
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
        if !is_primordial_owner(desc.surface.key.owner) {
            continue;
        }
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


fn artifact_anchor_for_descriptor<'a>(
    anchors: &'a [phalcom_compiler_artifact::executable::compatibility::PortableNativeAnchorV1],
    desc: &PrimitiveDescriptor,
) -> PhResult<&'a phalcom_compiler_artifact::executable::compatibility::PortableNativeAnchorV1> {
    use phalcom_compiler_artifact::executable::constant::StableClassRefV1;
    use phalcom_type_meta::identity::StableDispatchSide;

    let expected_owner = StableClassRefV1 {
        declaration: phalcom_semantic::metadata::stable_identity::to_stable_declaration(
            &phalcom_semantic::core_surface::universe_declaration(desc.surface.key.owner),
        ),
        side: match desc.surface.key.side {
            NativeDispatch::Instance => StableDispatchSide::Instance,
            NativeDispatch::Class => StableDispatchSide::Class,
        },
    };
    anchors
        .iter()
        .find(|anchor| anchor.owner == expected_owner && anchor.signature.selector == desc.surface.key.selector)
        .ok_or_else(|| {
            crate::error::PhError::StringError(format!(
                "validated artifact native anchor is missing for {} {}",
                desc.surface.key.owner.name(),
                desc.surface.key.selector
            ))
        })
}

/// Rebinds primordial native methods to the exact canonical Universe module
/// recorded by the validated artifact ABI. This is the source-free analogue
/// of `refresh_registered_primitive_origins`; it reads no source index.
pub(crate) fn refresh_registered_primitive_origins_from_artifact(
    vm: &mut VM,
    anchors: &[phalcom_compiler_artifact::executable::compatibility::PortableNativeAnchorV1],
) -> PhResult<()> {
    for desc in PRIMITIVES {
        if !is_primordial_owner(desc.surface.key.owner) {
            continue;
        }
        let anchor = artifact_anchor_for_descriptor(anchors, desc)?;
        let declaration =
            phalcom_semantic::metadata::stable_identity::from_stable_declaration(&anchor.owner.declaration)
                .map_err(crate::error::PhError::StringError)?;
        let module = vm
            .module_registry
            .get(&declaration.module)
            .map(|record| record.object)
            .ok_or_else(|| {
                crate::error::PhError::StringError(format!(
                    "artifact native declaration module {} is not materialized",
                    declaration.module
                ))
            })?;
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

/// Installs source-owned native descriptors after artifact eager initializers
/// have materialized their exact owner classes. Owner and origin authority come
/// from validated portable native anchors; no source catalog participates.
pub(crate) fn install_source_owned_primitives_from_artifact(
    vm: &mut VM,
    anchors: &[phalcom_compiler_artifact::executable::compatibility::PortableNativeAnchorV1],
) -> PhResult<()> {
    let mut descriptors = PRIMITIVES
        .iter()
        .filter(|desc| !is_primordial_owner(desc.surface.key.owner))
        .collect::<Vec<_>>();
    descriptors.sort_by_key(|desc| desc.surface.key.sort_key());

    for desc in descriptors {
        let anchor = artifact_anchor_for_descriptor(anchors, desc)?;
        let declaration =
            phalcom_semantic::metadata::stable_identity::from_stable_declaration(&anchor.owner.declaration)
                .map_err(crate::error::PhError::StringError)?;
        let module = vm
            .module_registry
            .get(&declaration.module)
            .map(|record| record.object)
            .ok_or_else(|| {
                crate::error::PhError::StringError(format!(
                    "artifact native declaration module {} is not materialized",
                    declaration.module
                ))
            })?;
        let owner = vm
            .typing_registry
            .resolve_nominal(&anchor.owner.declaration)
            .ok_or_else(|| {
                crate::error::PhError::StringError(format!(
                    "artifact native owner {:?} has no exact runtime nominal binding",
                    anchor.owner.declaration
                ))
            })?;
        install_one_on_owner(vm, desc, owner)?;

        let target = match desc.surface.key.side {
            NativeDispatch::Instance => owner,
            NativeDispatch::Class => vm.heap.class(owner).class,
        };
        let selector = vm.interner.intern(desc.surface.key.selector);
        let method = vm
            .heap
            .class(target)
            .get_method(selector)
            .ok_or_else(|| {
                crate::error::PhError::StringError(format!(
                    "installed artifact source native method {} is missing",
                    desc.surface.key.selector
                ))
            })?;
        vm.heap.method_mut(method).origin = vm.access_origin_for_module(module);
    }
    Ok(())
}

/// Installs descriptors whose canonical owner is an ordinary source-declared
/// Universe class. The source module and class global are resolved only after
/// canonical Universe initialization has compiled those declarations.
pub fn install_source_owned_primitives(vm: &mut VM, source_index: &NativeSourceIndex) -> PhResult<()> {
    let mut descriptors = PRIMITIVES
        .iter()
        .filter(|desc| !is_primordial_owner(desc.surface.key.owner))
        .collect::<Vec<_>>();
    descriptors.sort_by_key(|d| d.surface.key.sort_key());

    for desc in descriptors {
        let anchor = source_index
            .members
            .get(&super::verify::source_key(desc.surface.key))
            .ok_or_else(|| crate::error::PhError::StringError(format!("source-owned native {} has no source anchor", desc.surface.key.selector)))?;
        let module = vm
            .module_registry
            .get(&anchor.module)
            .map(|record| record.object)
            .ok_or_else(|| crate::error::PhError::StringError(format!("native declaration module {} is not materialized", anchor.module)))?;
        let class_name = vm.interner.intern(&anchor.class_name);
        let owner = vm
            .heap
            .module(module)
            .get(class_name)
            .and_then(|value| value.as_obj())
            .ok_or_else(|| crate::error::PhError::StringError(format!("source native owner {} is not a class global", anchor.class_name)))?;

        install_one_on_owner(vm, desc, owner)?;
        let selector = vm.interner.intern(desc.surface.key.selector);
        let target = match desc.surface.key.side {
            NativeDispatch::Instance => owner,
            NativeDispatch::Class => vm.heap.class(owner).class,
        };
        let method = vm
            .heap
            .class(target)
            .get_method(selector)
            .ok_or_else(|| crate::error::PhError::StringError(format!("installed source native method {} is missing", desc.surface.key.selector)))?;
        vm.heap.method_mut(method).origin = vm.access_origin_for_module(module);
    }

    Ok(())
}

fn is_primordial_owner(owner: UniverseKey) -> bool {
    UNIVERSE_BINDINGS.iter().any(|binding| binding.key == owner)
}

fn install_one(vm: &mut VM, desc: &PrimitiveDescriptor) -> PhResult<()> {
    let owner = vm.universe.classes.resolve(desc.surface.key.owner);
    install_one_on_owner(vm, desc, owner)
}

fn install_one_on_owner(vm: &mut VM, desc: &PrimitiveDescriptor, owner: crate::heap::ClassId) -> PhResult<()> {
    let target = match desc.surface.key.side {
        NativeDispatch::Instance => owner,
        NativeDispatch::Class => vm.heap.class(owner).class,
    };

    let selector = vm.interner.intern(desc.surface.key.selector);
    let signature = desc.runtime_signature(vm, selector);
    let replaced = vm.heap.class(target).get_method(selector);

    let mut method = match desc.entry {
        PrimitiveEntry::Value(f) => MethodObject::new_primitive(signature.clone(), f, owner),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method::{CallOutcome, RestMode};
    use crate::value::Value;
    use phalcom_native_meta::{
        CallableTypeSpec, EffectSpec, NativeAnchorPolicy, NativeLifecycleSpec, NativeSourceSpec, NativeStability, NativeTrust, ParameterTupleSpec,
        PrimitiveAbi, PrimitiveKey, PrimitiveSurfaceSpec, RaisesSpec, ReturnFlowSpec, TerminationSpec, TypeExprSpec,
    };

    static EMPTY_PARAMS: ParameterTupleSpec = ParameterTupleSpec {
        positional: &[],
        labeled: &[],
        rest: None,
    };
    static UNKNOWN_RETURN: TypeExprSpec = TypeExprSpec::Unknown;
    static CALLABLE: CallableTypeSpec = CallableTypeSpec {
        type_params: &[],
        params: &EMPTY_PARAMS,
        return_type: &UNKNOWN_RETURN,
        constraints: &[],
    };

    fn test_descriptor(selector: &'static str, abi: PrimitiveAbi, entry: PrimitiveEntry) -> PrimitiveDescriptor {
        let surface = Box::leak(Box::new(PrimitiveSurfaceSpec {
            key: PrimitiveKey {
                owner: phalcom_native_meta::UniverseKey::Object,
                side: NativeDispatch::Instance,
                selector,
            },
            visibility: phalcom_native_meta::NativeVisibility::Public,
            stability: NativeStability::Stable,
            anchor: NativeAnchorPolicy::Hidden,
            signature_mode: phalcom_native_meta::NativeSignatureMode::Exact,
            params: &EMPTY_PARAMS,
            returns: &UNKNOWN_RETURN,
            callable: &CALLABLE,
            raises: RaisesSpec::Unknown,
            effects: EffectSpec::Unknown,
            flow: ReturnFlowSpec::Unknown,
            termination: TerminationSpec::Unknown,
            since: None,
            deprecated_since: None,
            replacement: None,
            lifecycle: NativeLifecycleSpec::UNKNOWN,
            intrinsic: None,
            trust: NativeTrust::Ordinary,
            docs: None,
            conceptual: None,
        }));
        PrimitiveDescriptor {
            surface,
            abi,
            entry,
            source: NativeSourceSpec {
                module_path: "test",
                rust_name: "signature_probe",
                file: "install.rs",
                line: 1,
            },
        }
    }

    fn value_primitive(_: &mut VM, _: &Value, _: &[Value]) -> PhResult<Value> {
        Ok(Value::nil())
    }

    fn shape_primitive(_: &mut VM, _: Value, _: crate::method::ArgumentView) -> PhResult<CallOutcome> {
        Ok(CallOutcome::Returned(Value::nil()))
    }

    #[test]
    fn installed_native_methods_preserve_descriptor_signatures() {
        let canonical = crate::modules::canonical_universe_program().expect("canonical Universe product");
        let mut vm = VM::new();

        for descriptor in PRIMITIVES {
            let owner = if is_primordial_owner(descriptor.surface.key.owner) {
                vm.universe.classes.resolve(descriptor.surface.key.owner)
            } else {
                let anchor = canonical
                    .source_index()
                    .members
                    .get(&super::super::verify::source_key(descriptor.surface.key))
                    .unwrap_or_else(|| panic!("native descriptor {} has no canonical source anchor", descriptor.surface.key.selector));
                let module = vm
                    .module_registry
                    .get(&anchor.module)
                    .unwrap_or_else(|| panic!("native descriptor module {} was not materialized", anchor.module))
                    .object;
                let class_name = vm.interner.intern(&anchor.class_name);
                vm.heap
                    .module(module)
                    .get(class_name)
                    .and_then(|value| value.as_obj())
                    .unwrap_or_else(|| panic!("native descriptor owner {} is not a runtime class", anchor.class_name))
            };
            let target = match descriptor.surface.key.side {
                phalcom_native_meta::NativeDispatch::Instance => owner,
                phalcom_native_meta::NativeDispatch::Class => vm.heap.class(owner).class,
            };
            let selector = vm.get_or_intern(descriptor.surface.key.selector);
            let installed_method = vm
                .heap
                .class(target)
                .get_method(selector)
                .unwrap_or_else(|| panic!("native descriptor {} was not installed", descriptor.surface.key.selector));
            let installed_signature = vm.heap.method(installed_method).signature.clone();
            let descriptor_signature = descriptor.runtime_signature(&mut vm, selector);

            assert_eq!(
                installed_signature.selector, descriptor_signature.selector,
                "{}",
                descriptor.surface.key.selector
            );
            assert_eq!(installed_signature.kind, descriptor_signature.kind, "{}", descriptor.surface.key.selector);
            assert_eq!(
                installed_signature.positional_arity, descriptor_signature.positional_arity,
                "{}",
                descriptor.surface.key.selector
            );
            assert_eq!(installed_signature.rest, descriptor_signature.rest, "{}", descriptor.surface.key.selector);
        }

        let cases = [
            ("auditPositionalSignatureProbe(_)", PrimitiveAbi::Value, PrimitiveEntry::Value(value_primitive)),
            ("auditLabeledSignatureProbe(name)", PrimitiveAbi::Value, PrimitiveEntry::Value(value_primitive)),
            ("auditMixedSignatureProbe(_,name)", PrimitiveAbi::Shape, PrimitiveEntry::Shape(shape_primitive)),
            ("[auditIndex]", PrimitiveAbi::Value, PrimitiveEntry::Value(value_primitive)),
            ("[auditIndex]=(_)", PrimitiveAbi::Shape, PrimitiveEntry::Shape(shape_primitive)),
            ("auditValueSignatureProbe=(_)", PrimitiveAbi::Value, PrimitiveEntry::Value(value_primitive)),
            (
                "auditCompleteRestSignatureProbe(***)",
                PrimitiveAbi::Shape,
                PrimitiveEntry::Shape(shape_primitive),
            ),
        ];

        for (selector_text, abi, entry) in cases {
            let descriptor = test_descriptor(selector_text, abi, entry);
            let owner = vm.universe.classes.object_class;
            super::install_one_on_owner(&mut vm, &descriptor, owner).expect("synthetic descriptor installs");
            let selector = vm.get_or_intern(selector_text);
            let target = vm.universe.classes.object_class;
            let installed_method = vm.heap.class(target).get_method(selector).expect("synthetic method is installed");
            let installed_signature = vm.heap.method(installed_method).signature.clone();
            let descriptor_signature = descriptor.runtime_signature(&mut vm, selector);
            assert_eq!(installed_signature.selector, descriptor_signature.selector, "{selector_text}");
            assert_eq!(installed_signature.kind, descriptor_signature.kind, "{selector_text}");
            assert_eq!(installed_signature.positional_arity, descriptor_signature.positional_arity, "{selector_text}");
            assert_eq!(installed_signature.rest, descriptor_signature.rest, "{selector_text}");
            if selector_text.contains("***") {
                assert!(matches!(installed_signature.rest.as_ref().unwrap().mode(), RestMode::Complete { .. }));
            }
        }
    }
}
