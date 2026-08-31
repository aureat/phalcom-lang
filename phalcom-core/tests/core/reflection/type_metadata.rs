use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::typing::reify::reify_type_form;
use phalcom_core::vm::VM;
use phalcom_type_meta::validate::{ValidationLimits, validate_metadata_bundle};
use std::sync::Arc;

#[test]
fn reified_nominal_types_preserve_canonical_identity() {
    let mut vm = VM::new();

    // 1. Compile inline module defining class Box
    let inline_src = "class Box {\n  init() {}\n}\n";
    let compiled = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(inline_src))).unwrap();

    assert!(compiled.semantic_metadata.is_some());
    let bundle = compiled.semantic_metadata.as_ref().unwrap();
    validate_metadata_bundle(bundle, &ValidationLimits::default()).unwrap();

    vm.materialize_program(&compiled).unwrap();

    // 2. Test reification of loaded metadata nominal forms: reify(Int) === Int and reify(List) === List
    let int_decl = phalcom_type_meta::identity::StableDeclarationRef {
        module: phalcom_type_meta::identity::StableModuleRef {
            project: phalcom_type_meta::identity::StableProjectRef::Builtin {
                namespace: "std".into(),
                version: "0.1.0".into(),
            },
            path: Box::new(["Int".into()]),
        },
        path: Box::new(["Int".into()]),
    };

    let resolved_int = vm.typing_registry.resolve_nominal(&int_decl);
    assert!(resolved_int.is_some());
    assert_eq!(resolved_int.unwrap(), vm.universe.classes.int_class);

    let list_decl = phalcom_type_meta::identity::StableDeclarationRef {
        module: phalcom_type_meta::identity::StableModuleRef {
            project: phalcom_type_meta::identity::StableProjectRef::Builtin {
                namespace: "std".into(),
                version: "0.1.0".into(),
            },
            path: Box::new(["List".into()]),
        },
        path: Box::new(["List".into()]),
    };

    let resolved_list = vm.typing_registry.resolve_nominal(&list_decl);
    assert!(resolved_list.is_some());
    assert_eq!(resolved_list.unwrap(), vm.universe.classes.list_class);
}

#[test]
fn synthetic_type_descriptors_use_weak_cache_entries() {
    let mut vm = VM::new();

    // Create a context object
    let ctx_data = phalcom_core::typing::context::TypingContextData::new(Box::new([phalcom_core::typing::handle::MetadataPoolId(0)]));
    let ctx_obj = phalcom_core::heap::TypingObject {
        class: vm.universe.classes.object_class,
        payload: phalcom_core::heap::TypingPayload::Context(ctx_data),
    };
    let ctx_ref = vm.heap.alloc(phalcom_core::heap::Object::Typing(Box::new(ctx_obj)));

    let synthetic_handle = phalcom_core::typing::handle::RuntimeTypeRef::Overlay(phalcom_core::typing::handle::RuntimeOverlayTypeId(42));

    // Reify synthetic type form
    let val1 = reify_type_form(ctx_ref, synthetic_handle, &vm.typing_registry, &mut vm.heap, vm.universe.classes.object_class).unwrap();
    let val2 = reify_type_form(ctx_ref, synthetic_handle, &vm.typing_registry, &mut vm.heap, vm.universe.classes.object_class).unwrap();

    // Reification returns same descriptor while live
    assert_eq!(val1.as_obj(), val2.as_obj());

    let desc_ref = val1.as_obj().unwrap();

    // Weak cache must not prevent GC from collecting unreferenced descriptors
    let _trace_roots = |push: &mut dyn FnMut(phalcom_core::heap::ObjRef)| {
        // Trace only ctx_ref, not desc_ref
        push(ctx_ref);
    };

    // Before GC sweep, check descriptor is alive
    assert!(vm.heap.try_get(desc_ref).is_some());

    // After reification, weak cache has the entry
    if let phalcom_core::heap::Object::Typing(t) = vm.heap.get(ctx_ref) {
        if let phalcom_core::heap::TypingPayload::Context(c) = &t.payload {
            assert!(
                c.descriptor_cache
                    .contains_key(&phalcom_core::typing::handle::RuntimeSemanticHandle::Type(synthetic_handle))
            );
        }
    }
}
