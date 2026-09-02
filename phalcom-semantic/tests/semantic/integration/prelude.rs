use phalcom_modules::linker::LinkedProgram;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::{ModuleId, ModulePath, SyntheticProjectIdAllocator};
use phalcom_semantic::core_surface::universe_declaration;
use phalcom_semantic::types::TypeResolver;
use phalcom_semantic::{DeclarationId, LinkedTypeResolver, PreludeTypeMap};
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

fn synthetic_module() -> ModuleId {
    let mut ids = SyntheticProjectIdAllocator;
    ModuleId::synthetic(ids.allocate(), ModulePath::root())
}

fn dummy_linked(entry: ModuleId) -> Arc<LinkedProgram> {
    Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: BTreeMap::new(),
        graphs: Default::default(),
        entry: entry.clone(),
        initialization_order: vec![entry],
    })
}

#[test]
fn prelude_map_contains_only_explicit_source_backed_type_names() {
    let prelude = PreludeTypeMap::canonical_universe();

    for name in ["Object", "Int", "Bool", "String", "Option", "Result", "List", "Map", "Unit"] {
        let declaration = prelude.get(name).unwrap_or_else(|| panic!("{name} must be in the canonical prelude"));
        let key = phalcom_native_meta::UniverseKey::from_name(name).expect("canonical Universe key");
        assert_eq!(declaration, &universe_declaration(key), "{name} must preserve its source-owned declaration identity");
    }

    for name in ["Nil", "Some", "None", "Behavior", "Metaclass", "Method", "Family"] {
        assert!(!prelude.contains_name(name), "{name} must not leak into the bare type prelude");
    }
}

#[test]
fn linked_type_resolver_uses_prelude_only_after_local_resolution() {
    let current = synthetic_module();
    let prelude = Arc::new(PreludeTypeMap::canonical_universe());
    let local_int = DeclarationId::new(current.clone(), "Int".into());

    let mut known = phalcom_native_meta::UNIVERSE_BINDINGS
        .iter()
        .map(|binding| universe_declaration(binding.key))
        .collect::<HashSet<_>>();
    known.insert(local_int.clone());

    let resolver = LinkedTypeResolver::with_prelude(dummy_linked(current.clone()), known, prelude);
    assert_eq!(resolver.resolve_type_name(&current, "Int", &[]), Some(local_int));
}

#[test]
fn linked_type_resolver_does_not_reconstruct_non_prelude_universe_names() {
    let current = synthetic_module();
    let prelude = Arc::new(PreludeTypeMap::canonical_universe());
    let known = phalcom_native_meta::UNIVERSE_BINDINGS
        .iter()
        .map(|binding| universe_declaration(binding.key))
        .collect::<HashSet<_>>();
    let resolver = LinkedTypeResolver::with_prelude(dummy_linked(current.clone()), known, prelude);

    assert_eq!(
        resolver.resolve_type_name(&current, "Result", &[]),
        Some(universe_declaration(phalcom_native_meta::UniverseKey::Result))
    );

    for name in ["Nil", "Some", "None", "Behavior", "Metaclass", "Method", "Family"] {
        assert_eq!(resolver.resolve_type_name(&current, name, &[]), None, "{name} must not resolve merely because it has a UniverseKey");
    }
}
