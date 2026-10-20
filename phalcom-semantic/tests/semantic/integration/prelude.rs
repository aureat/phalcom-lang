use phalcom_ast::ast::{PathSegment, StaticSymbolRef, TypeAnnotation, TypeAnnotationExpr};
use phalcom_common::range::SourceRange;
use phalcom_modules::linker::LinkedProgram;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::{ModuleComponent, ModuleId, ModulePath, SyntheticProjectIdAllocator};
use phalcom_semantic::core_surface::universe_declaration;
use phalcom_semantic::declarations::bootstrap_universe_declarations;
use phalcom_semantic::types::annotation::{TypeFormResolution, TypeFormationSite, TypeResolver, resolve_type_form};
use phalcom_semantic::types::store::TypeStore;
use phalcom_semantic::{DeclarationId, LinkedTypeResolver, PreludeTypeMap};
use std::cell::Cell;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

fn synthetic_module() -> ModuleId {
    let mut ids = SyntheticProjectIdAllocator;
    ModuleId::synthetic(ids.allocate(), ModulePath::root())
}

fn dummy_linked(entry: ModuleId) -> Arc<LinkedProgram> {
    Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::new()),
        graphs: Default::default(),
        entry: entry.clone(),
        initialization_order: vec![entry],
    })
}

#[test]
fn prelude_map_contains_only_explicit_source_backed_type_names() {
    let prelude = PreludeTypeMap::canonical_universe();

    for name in [
        "Object",
        "Int",
        "Bool",
        "String",
        "Option",
        "Result",
        "List",
        "Map",
        "Unit",
        "Duration",
        "Timestamp",
    ] {
        if matches!(name, "Duration" | "Timestamp") {
            assert!(prelude.contains_name(name), "{name} must be in the canonical prelude");
            continue;
        }
        let declaration = prelude.get(name).unwrap_or_else(|| panic!("{name} must be in the canonical prelude"));
        if name == "Unit" {
            let unit_module = ModuleId::universe(ModulePath::from_components(vec![
                ModuleComponent::from_identifier("option").unwrap(),
                ModuleComponent::from_identifier("unit").unwrap(),
            ]));
            assert_eq!(declaration, &DeclarationId::new(unit_module, "Unit".into()));
            assert!(
                phalcom_native_meta::UniverseKey::from_name("Unit").is_none(),
                "Unit must not resolve as a runtime class key"
            );
            continue;
        }
        let key = phalcom_native_meta::UniverseKey::from_name(name).expect("canonical Universe key");
        assert_eq!(
            declaration,
            &universe_declaration(key),
            "{name} must preserve its source-owned declaration identity"
        );
    }

    for name in [
        "Nil",
        "Some",
        "None",
        "Behavior",
        "Metaclass",
        "Method",
        "Family",
        "Clock",
        "SystemClock",
        "Instant",
        "clock",
        "WallClock",
        "SystemWallClock",
        "wallClock",
        "Date",
        "Time",
        "DateTime",
        "Period",
        "UtcOffset",
    ] {
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
        assert_eq!(
            resolver.resolve_type_name(&current, name, &[]),
            None,
            "{name} must not resolve merely because it has a UniverseKey"
        );
    }
}

#[test]
fn unqualified_unit_type_reference_uses_exact_prelude_alias_declaration() {
    struct UnitLookupTrace<'a> {
        resolver: &'a LinkedTypeResolver,
        saw_unit_lookup: Cell<bool>,
    }

    impl TypeResolver for UnitLookupTrace<'_> {
        fn resolve_type_name(&self, current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId> {
            if root == "Unit" && members.is_empty() {
                self.saw_unit_lookup.set(true);
            }
            self.resolver.resolve_type_name(current_module, root, members)
        }

        fn resolve_alias_form(&self, declaration: &DeclarationId) -> Option<phalcom_semantic::types::id::TypeId> {
            self.resolver.resolve_alias_form(declaration)
        }
    }

    let current = synthetic_module();
    let unit_module = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("option").unwrap(),
        ModuleComponent::from_identifier("unit").unwrap(),
    ]));
    let unit_declaration = DeclarationId::new(unit_module, "Unit".into());
    let mut store = TypeStore::new();
    let declarations = bootstrap_universe_declarations(&mut store, &universe_declaration);
    let mut known = phalcom_native_meta::UNIVERSE_BINDINGS
        .iter()
        .map(|binding| universe_declaration(binding.key))
        .collect::<HashSet<_>>();
    known.insert(unit_declaration.clone());

    let resolver = LinkedTypeResolver::with_prelude(dummy_linked(current.clone()), known, Arc::new(PreludeTypeMap::canonical_universe()));
    resolver.insert_alias_form(unit_declaration.clone(), store.unit());
    assert_eq!(resolver.resolve_type_name(&current, "Unit", &[]), Some(unit_declaration.clone()));

    let traced_resolver = UnitLookupTrace {
        resolver: &resolver,
        saw_unit_lookup: Cell::new(false),
    };
    let range = SourceRange { start: 0, end: 4 };
    let annotation = TypeAnnotation {
        expr: TypeAnnotationExpr::Reference(StaticSymbolRef {
            root: "Unit".into(),
            root_range: range,
            members: Vec::<PathSegment>::new(),
            range,
        }),
        range,
    };
    let mut diagnostics = Vec::new();
    let resolved = resolve_type_form(
        &mut store,
        &declarations,
        &traced_resolver,
        &TypeFormationSite::module(current),
        &annotation,
        &mut diagnostics,
    );

    assert!(
        traced_resolver.saw_unit_lookup.get(),
        "bare Unit type references must consult canonical name resolution"
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(resolved, TypeFormResolution::Ready(store.unit()));
}
