use phalcom_modules::ModuleId;
use phalcom_modules::identity::{ModuleComponent, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::LinkedModuleInterface;
use phalcom_modules::linker::{LinkedModule, LinkedProgram, ModuleBindingLayout};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::BTreeMap;
use std::sync::Arc;

fn module_id() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(31),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    )
}

fn input(module: ModuleId, source: &str, generation: u64) -> SemanticWorkspaceInput {
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let program = Arc::new(parsed.program);
    let linked_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: module.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout::default(),
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let mut modules = BTreeMap::new();
    modules.insert(module.clone(), linked_module);
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: module.clone(),
        initialization_order: vec![module.clone()],
    });
    let mut sources = BTreeMap::new();
    sources.insert(
        module.clone(),
        Arc::new(ParsedModuleUnit::new(module, ModuleKind::Module, None, Arc::from(source), program)),
    );
    SemanticWorkspaceInput::new(linked, sources, generation)
}

fn callable(module: &ModuleId, selector: &str, slots: usize) -> CallableId {
    let selector = match slots {
        0 => phalcom_common::selector::Selector::method(selector, []).unwrap(),
        1 => phalcom_common::selector::Selector::method(selector, [phalcom_common::selector::SelectorSlot::Positional]).unwrap(),
        _ => panic!("unsupported test selector arity"),
    };
    CallableId::new(DeclarationId::new(module.clone(), "Display".into()), selector, DispatchSide::Instance)
}

#[test]
fn trait_body_edit_reuses_surface_and_unrelated_default_but_refreshes_changed_body() {
    let module = module_id();
    let source_a = r#"
trait Display {
  render(_ value: Int) -> Int
  fallback(_ value: Int) -> Int { self.render(value) }
  unrelated() -> Int { 1 }
}
"#;
    let source_b = source_a.replace("self.render(value)", "self.render(1)");
    let mut session = SemanticWorkspaceSession::new();
    let first = session
        .update_with_budget_and_cancel(
            input(module.clone(), source_a, 1),
            phalcom_semantic::db::QueryBudget::default(),
            &phalcom_semantic::db::CancellationToken::new(),
        )
        .unwrap_or_else(|error| panic!("initial update failed: {error:?}"));
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    let declaration = DeclarationId::new(module.clone(), "Display".into());
    let first_surface = first.snapshot.trait_surfaces.get(&declaration).expect("initial trait surface").clone();
    let fallback = callable(&module, "fallback", 1);
    let unrelated = callable(&module, "unrelated", 0);
    let first_fallback = first.snapshot.callable_analyses.get(&fallback).expect("initial fallback analysis").clone();
    let first_unrelated = first.snapshot.callable_analyses.get(&unrelated).expect("initial unrelated analysis").clone();
    let header_key = phalcom_semantic::db::QueryKey::TraitHeader(declaration.clone());
    let surface_key = phalcom_semantic::db::QueryKey::TraitSurface(declaration.clone());
    let body_key = phalcom_semantic::db::QueryKey::CallableBody(fallback.clone());
    assert!(
        session
            .db()
            .index()
            .dependencies_of(&surface_key)
            .is_some_and(|edges| edges.iter().any(|edge| edge.dependency == header_key)),
        "trait surface must depend on its DB-owned header"
    );
    assert!(
        session
            .db()
            .index()
            .dependencies_of(&body_key)
            .is_some_and(|edges| edges.iter().any(|edge| edge.dependency == surface_key)),
        "trait default body must depend on its DB-owned surface"
    );

    let second = session.update(input(module.clone(), &source_b, 2));
    assert!(!second.snapshot.has_errors(), "updated diagnostics: {:?}", second.snapshot.diagnostics);
    let second_surface = second.snapshot.trait_surfaces.get(&declaration).expect("updated trait surface").clone();
    assert_eq!(
        phalcom_semantic::db::fingerprint::trait_surface_product_fingerprint(&first_surface),
        phalcom_semantic::db::fingerprint::trait_surface_product_fingerprint(&second_surface),
        "body-only edit must preserve the complete trait contract fingerprint",
    );
    let second_fallback = second.snapshot.callable_analyses.get(&fallback).expect("updated fallback analysis").clone();
    let second_unrelated = second.snapshot.callable_analyses.get(&unrelated).expect("updated unrelated analysis").clone();
    assert!(
        !Arc::ptr_eq(&first_fallback, &second_fallback),
        "changed default body must refresh its analysis"
    );
    assert!(Arc::ptr_eq(&first_unrelated, &second_unrelated), "unrelated default must remain reusable");
}

#[test]
fn trait_bodyless_to_bodyful_preserves_requirement_and_source_callable_identity() {
    let module = module_id();
    let source_a = "trait Display { render(_ value: Int) -> Int }\n";
    let source_b = "trait Display { render(_ value: Int) -> Int { value } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(input(module.clone(), source_a, 1));
    let second = session.update(input(module.clone(), source_b, 2));
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    assert!(!second.snapshot.has_errors(), "updated diagnostics: {:?}", second.snapshot.diagnostics);
    let declaration = DeclarationId::new(module.clone(), "Display".into());
    let selector = phalcom_common::selector::Selector::method("render", [phalcom_common::selector::SelectorSlot::Positional]).unwrap();
    let first_member = first
        .snapshot
        .trait_surfaces
        .get(&declaration)
        .unwrap()
        .get_by_selector(&selector, DispatchSide::Instance)
        .unwrap();
    let second_member = second
        .snapshot
        .trait_surfaces
        .get(&declaration)
        .unwrap()
        .get_by_selector(&selector, DispatchSide::Instance)
        .unwrap();
    assert_eq!(first_member.requirement, second_member.requirement);
    assert_eq!(first_member.callable, second_member.callable);
    assert!(!first_member.default_present);
    assert!(second_member.default_present);
    assert!(second.snapshot.callable_analyses.contains_key(&second_member.callable));
}

#[test]
fn trait_incremental_signature_and_header_edits_match_cold_diagnostics() {
    let module = module_id();
    let source_a = r#"
trait Display<T> {
  render(_ value: T) -> T
  fallback(_ value: T) -> T { self.render(value) }
}
"#;
    let source_b = r#"
trait Display<T> where T <: Object {
  render(_ value: T) -> Int
  fallback(_ value: T) -> T { self.render(value) }
}
"#;
    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental.update(input(module.clone(), source_a, 1));
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    let updated = incremental.update(input(module.clone(), source_b, 2));
    let mut cold = SemanticWorkspaceSession::new();
    let cold_result = cold.update(input(module.clone(), source_b, 1));
    assert_eq!(updated.snapshot.diagnostics, cold_result.snapshot.diagnostics);
    let declaration = DeclarationId::new(module.clone(), "Display".into());
    let incremental_surface = updated.snapshot.trait_surfaces.get(&declaration).expect("incremental surface");
    let cold_surface = cold_result.snapshot.trait_surfaces.get(&declaration).expect("cold surface");
    assert_eq!(incremental_surface, cold_surface, "header/signature edits must preserve cold parity");
}
