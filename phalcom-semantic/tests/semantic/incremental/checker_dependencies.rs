use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::checker::analysis::{CallableAnalysisStatus, SemanticDependency};
use phalcom_semantic::dispatch::{CallableSignature, DispatchLookup, SurfaceDispatchResolver};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::surface::DeclarationSurface;
use phalcom_semantic::types::annotation::{SimpleTypeResolver, TypeResolver};
use phalcom_semantic::types::evidence::{EvidenceOrigin, TypeKnowledge};
use phalcom_semantic::types::relation::{MapTypeHierarchy, TypeHierarchy};
use phalcom_semantic::types::store::TypeStore;
use phalcom_semantic::{CheckingContext, DeclarationTypeTable};

fn user_module(project: u32, name: &str) -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(project),
        ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]),
    )
}

fn callable(owner: DeclarationId) -> CallableId {
    CallableId::new(owner, Selector::method("test", vec![]).unwrap(), DispatchSide::Instance)
}

#[test]
fn tracked_resolver_records_current_linked_interface_for_non_local_type() {
    let current = user_module(1, "client");
    let external_module = user_module(2, "external");
    let external = DeclarationId::new(external_module, "External".into());
    let owner = DeclarationId::new(current.clone(), "Client".into());

    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut resolver = SimpleTypeResolver::new();
    resolver.insert("External", external.clone());
    let declarations = DeclarationTypeTable::new();
    let dispatch = SurfaceDispatchResolver::new();

    let ctx = CheckingContext::new_with_dispatch_ref(&mut store, &hierarchy, &resolver, &declarations, &dispatch, current.clone());

    assert_eq!(ctx.resolver.resolve_type_name(&current, "External", &[]), Some(external.clone()));

    let analysis = ctx.finalize(callable(owner), SourceRange::default(), CallableAnalysisStatus::Complete);
    assert!(analysis.semantic_dependencies.contains(&SemanticDependency::LinkedInterface(current)));
    assert!(analysis.semantic_dependencies.contains(&SemanticDependency::DeclarationShell(external)));
}

#[test]
fn unresolved_type_lookup_records_negative_linked_interface_dependency() {
    let current = user_module(1, "client");
    let owner = DeclarationId::new(current.clone(), "Client".into());
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let resolver = SimpleTypeResolver::new();
    let declarations = DeclarationTypeTable::new();
    let dispatch = SurfaceDispatchResolver::new();

    let ctx = CheckingContext::new_with_dispatch_ref(&mut store, &hierarchy, &resolver, &declarations, &dispatch, current.clone());

    assert_eq!(ctx.resolver.resolve_type_name(&current, "Missing", &[]), None);
    let analysis = ctx.finalize(callable(owner), SourceRange::default(), CallableAnalysisStatus::Complete);
    assert!(analysis.semantic_dependencies.contains(&SemanticDependency::LinkedInterface(current)));
}

#[test]
fn tracked_hierarchy_records_every_mutable_edge_traversed() {
    let module = user_module(1, "main");
    let a = DeclarationId::new(module.clone(), "A".into());
    let b = DeclarationId::new(module.clone(), "B".into());
    let c = DeclarationId::new(module.clone(), "C".into());
    let missing = DeclarationId::new(module.clone(), "Missing".into());
    let owner = DeclarationId::new(module.clone(), "Client".into());
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    hierarchy.insert(a.clone(), b.clone());
    hierarchy.insert(b.clone(), c.clone());
    let resolver = SimpleTypeResolver::new();
    let declarations = DeclarationTypeTable::new();
    let dispatch = SurfaceDispatchResolver::new();

    let ctx = CheckingContext::new_with_dispatch_ref(&mut store, &hierarchy, &resolver, &declarations, &dispatch, module);
    assert!(ctx.hierarchy.is_subclass(&a, &c));
    assert!(!ctx.hierarchy.is_subclass(&a, &missing));
    let analysis = ctx.finalize(callable(owner), SourceRange::default(), CallableAnalysisStatus::Complete);
    assert!(analysis.semantic_dependencies.contains(&SemanticDependency::HierarchyEdge(a)));
    assert!(analysis.semantic_dependencies.contains(&SemanticDependency::HierarchyEdge(b)));
    assert!(
        analysis.semantic_dependencies.contains(&SemanticDependency::HierarchyEdge(c)),
        "the missing terminal edge is a negative hierarchy read that must invalidate if a superclass is later added"
    );
}

#[test]
fn builtin_seed_reads_do_not_create_query_dependencies() {
    let current = user_module(1, "client");
    let core = ModuleId::core();
    let int_decl = DeclarationId::new(core.clone(), "Int".into());
    let object_decl = DeclarationId::new(core, "Object".into());
    let owner = DeclarationId::new(current.clone(), "Client".into());
    let mut store = TypeStore::new();
    let int_ty = store.nominal_type(int_decl.clone());
    let mut hierarchy = MapTypeHierarchy::new();
    hierarchy.insert(int_decl.clone(), object_decl.clone());
    let mut resolver = SimpleTypeResolver::new();
    resolver.insert("Int", int_decl.clone());
    let declarations = DeclarationTypeTable::new();
    let selector = Selector::method("value", vec![]).unwrap();
    let mut dispatch = SurfaceDispatchResolver::new();
    let mut int_surface = DeclarationSurface::new(Some(int_decl.clone()));
    int_surface.add_callable(
        DispatchSide::Instance,
        CallableSignature::new(
            selector.clone(),
            Vec::new(),
            TypeKnowledge::assumed(int_ty, EvidenceOrigin::DeveloperAnnotation),
        ),
    );
    dispatch.register_surface(int_decl.clone(), int_surface);

    let mut ctx = CheckingContext::new_with_dispatch_ref(&mut store, &hierarchy, &resolver, &declarations, &dispatch, current.clone());
    assert_eq!(ctx.resolver.resolve_type_name(&current, "Int", &[]), Some(int_decl.clone()));
    assert!(ctx.hierarchy.is_subclass(&int_decl, &object_decl));
    assert!(ctx.resolve_dispatch(int_ty, &selector, DispatchLookup::Normal).is_found());
    let analysis = ctx.finalize(callable(owner), SourceRange::default(), CallableAnalysisStatus::Complete);
    assert!(analysis.semantic_dependencies.is_empty(), "legacy core reads have no staged DB products");
}

#[test]
fn declaration_metadata_read_records_declaration_shell_dependency() {
    let module = user_module(1, "main");
    let target = DeclarationId::new(module.clone(), "Target".into());
    let owner = DeclarationId::new(module.clone(), "Client".into());
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let resolver = SimpleTypeResolver::new();
    let declarations = DeclarationTypeTable::new();
    let dispatch = SurfaceDispatchResolver::new();

    let ctx = CheckingContext::new_with_dispatch_ref(&mut store, &hierarchy, &resolver, &declarations, &dispatch, module);
    let _ = ctx.declaration_generic_signature(&target);
    let analysis = ctx.finalize(callable(owner), SourceRange::default(), CallableAnalysisStatus::Complete);
    assert!(analysis.semantic_dependencies.contains(&SemanticDependency::DeclarationShell(target)));
}

#[test]
fn dispatch_lookup_records_surfaces_for_every_owner_inspected() {
    let module = user_module(1, "main");
    let base = DeclarationId::new(module.clone(), "Base".into());
    let child = DeclarationId::new(module.clone(), "Child".into());
    let owner = DeclarationId::new(module.clone(), "Client".into());
    let selector = Selector::method("value", vec![]).unwrap();
    let mut store = TypeStore::new();
    let child_ty = store.nominal_type(child.clone());
    let mut hierarchy = MapTypeHierarchy::new();
    hierarchy.insert(child.clone(), base.clone());
    let resolver = SimpleTypeResolver::new();
    let declarations = DeclarationTypeTable::new();
    let mut dispatch = SurfaceDispatchResolver::new();
    let mut base_surface = DeclarationSurface::new(Some(base.clone()));
    base_surface.add_callable(
        DispatchSide::Instance,
        CallableSignature::new(
            selector.clone(),
            Vec::new(),
            TypeKnowledge::assumed(child_ty, EvidenceOrigin::DeveloperAnnotation),
        ),
    );
    dispatch.register_surface(base.clone(), base_surface);

    let mut ctx = CheckingContext::new_with_dispatch_ref(&mut store, &hierarchy, &resolver, &declarations, &dispatch, module);
    assert!(ctx.resolve_dispatch(child_ty, &selector, DispatchLookup::Normal).is_found());
    let analysis = ctx.finalize(callable(owner), SourceRange::default(), CallableAnalysisStatus::Complete);
    assert!(analysis.semantic_dependencies.contains(&SemanticDependency::DeclarationSurface(child)));
    assert!(analysis.semantic_dependencies.contains(&SemanticDependency::DeclarationSurface(base.clone())));
    assert!(
        analysis
            .semantic_dependencies
            .contains(&SemanticDependency::CallableSignature(CallableId::new(base, selector, DispatchSide::Instance,)))
    );
}

#[test]
fn borrowed_dispatch_detaches_lazily_instead_of_panicking_on_mutation() {
    let module = user_module(1, "main");
    let owner = DeclarationId::new(module.clone(), "Owner".into());
    let nested = DeclarationId::new(module.clone(), "Nested".into());
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let resolver = SimpleTypeResolver::new();
    let declarations = DeclarationTypeTable::new();
    let shared_dispatch = SurfaceDispatchResolver::new();

    let mut ctx = CheckingContext::new_with_dispatch_ref(&mut store, &hierarchy, &resolver, &declarations, &shared_dispatch, module);
    ctx.register_surface(nested.clone(), DeclarationSurface::new(Some(nested.clone())));
    assert!(shared_dispatch.get_surface(&nested).is_none());
    assert!(ctx.get_surface(&nested).is_some());
    let _ = ctx.finalize(callable(owner), SourceRange::default(), CallableAnalysisStatus::Complete);
}

#[test]
fn checker_body_sources_do_not_bypass_tracked_resolver_or_declaration_helpers() {
    let expression = include_str!("../../../src/checker/expression.rs");
    let statement = include_str!("../../../src/checker/statement.rs");
    let predicate = include_str!("../../../src/checker/flow/predicate.rs");

    for (name, source) in [("expression.rs", expression), ("statement.rs", statement), ("flow/predicate.rs", predicate)] {
        let compact = source.split_whitespace().collect::<String>();
        assert!(
            !compact.contains("ctx.resolver.resolve_type_name"),
            "{name} must route type-name reads through the tracked resolver API"
        );
        assert!(
            !compact.contains("ctx.declarations.get") && !compact.contains("ctx.declarations.generic_signature"),
            "{name} must route declaration-table reads through CheckingContext helpers"
        );
    }
}
