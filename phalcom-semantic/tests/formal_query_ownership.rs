//! Regression coverage for DB-owned hierarchy, declaration-surface, and callable-signature queries.

use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::LinkedModuleInterface;
use phalcom_modules::linker::{LinkedModule, LinkedProgram, ModuleBindingLayout};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::db::{
    query_callable_body_with_formal_inputs, query_declaration_surface, CancellationToken, FormalQueryInputs, QueryBudget, QueryKey,
};
use phalcom_semantic::declarations::{bootstrap_universe_declarations, DeclarationTypeInfo};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::types::annotation::SimpleTypeResolver;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::BTreeMap;
use std::sync::Arc;

fn module_id() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    )
}

fn single_module_input(module: ModuleId, source_code: &str, generation: u64) -> SemanticWorkspaceInput {
    let parse_res = phalcom_ast::parse(source_code, 0);
    let program = Arc::new(parse_res.program);

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
        Arc::new(ParsedModuleUnit::new(
            module,
            ModuleKind::Module,
            None,
            Arc::from(source_code),
            program,
        )),
    );

    SemanticWorkspaceInput {
        linked,
        sources,
        generation,
    }
}

fn dependency_keys(session: &SemanticWorkspaceSession, key: &QueryKey) -> Vec<QueryKey> {
    session
        .db()
        .index()
        .dependencies_of(key)
        .map(|dependencies| dependencies.iter())
        .into_iter()
        .flatten()
        .map(|edge| edge.dependency.clone())
        .collect()
}

#[test]
fn formal_products_are_query_owned_and_record_prerequisites() {
    let module = module_id();
    let source = r#"
class Base {}

class Child is Base {
  @class
  identity(_ value: Int) -> Int { value }
}

"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), source, 1));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let child = DeclarationId::new(module.clone(), "Child".into());
    let hierarchy_key = QueryKey::HierarchyEdge(child.clone());
    let surface_key = QueryKey::DeclarationSurface(child.clone());
    let selector = Selector::method("identity", [SelectorSlot::Positional]).unwrap();
    let callable = CallableId::new(child.clone(), selector, DispatchSide::Class);
    let signature_key = QueryKey::CallableSignature(callable.clone());

    let hierarchy = session
        .db()
        .product(&hierarchy_key)
        .and_then(|product| product.as_hierarchy_edge())
        .expect("hierarchy query product");
    assert_eq!(hierarchy.class_decl, child);
    assert_eq!(hierarchy.super_decl.as_ref().map(|decl| decl.name.as_ref()), Some("Base"));

    let hierarchy_dependencies = dependency_keys(&session, &hierarchy_key);
    assert!(
        !hierarchy_dependencies.contains(&QueryKey::ParsedModule(module.clone())),
        "hierarchy source syntax is a direct query input, not a whole-module product dependency"
    );
    assert!(hierarchy_dependencies.contains(&QueryKey::LinkedInterface(module.clone())));

    let surface = session
        .db()
        .product(&surface_key)
        .and_then(|product| product.as_declaration_surface())
        .expect("declaration-surface query product");
    assert!(surface.get_callable(DispatchSide::Class, &callable.selector).is_some());

    let surface_dependencies = dependency_keys(&session, &surface_key);
    assert!(
        !surface_dependencies.contains(&QueryKey::ParsedModule(module.clone())),
        "declaration syntax is a direct query input so body-only parse changes do not invalidate the surface"
    );
    assert!(surface_dependencies.contains(&QueryKey::LinkedInterface(module)));
    assert!(surface_dependencies.contains(&QueryKey::DeclarationShell(child)));
    assert!(
        !surface_dependencies.contains(&surface_key),
        "declaration surface must never record itself as a dependency"
    );

    let signature = session
        .db()
        .product(&signature_key)
        .and_then(|product| product.as_callable_signature())
        .expect("callable-signature query product");
    assert_eq!(signature.callable, callable);
    assert_eq!(signature.parameter_count(), 1);
    assert!(signature.generics.is_none());

    assert_eq!(dependency_keys(&session, &signature_key), vec![surface_key]);
}

#[test]
fn body_query_ensures_complete_signature_without_signature_prewarm() {
    let module = module_id();
    let source = r#"
class Owner {
  @class read() -> Int { 1 }
}
"#;
    let input = single_module_input(module.clone(), source, 1);
    let unit = input.sources.get(&module).cloned().expect("source unit");
    let linked_interface = Arc::new(input.linked.modules[&module].interface.clone());

    let mut store = TypeStore::new();
    let mut declarations = bootstrap_universe_declarations(&mut store, &|key| {
        DeclarationId::new(phalcom_semantic::identity::ModuleId::core(), key.name().into())
    });
    let owner = DeclarationId::new(module.clone(), "Owner".into());
    declarations.insert(DeclarationTypeInfo {
        declaration: owner.clone(),
        form: store.nominal_type(owner.clone()),
        class_object_type: store.class_object_type(owner.clone()),
        kind: KindId::TYPE,
        generic_signature: None,
        supertype_template: None,
    });

    let int = DeclarationId::new(phalcom_semantic::identity::ModuleId::core(), "Int".into());
    let mut resolver = SimpleTypeResolver::new();
    resolver.insert("Int", int);
    let hierarchy = MapTypeHierarchy::new();
    let mut db = phalcom_semantic::db::SemanticDb::new();

    let surface = match query_declaration_surface(
        &mut db,
        owner.clone(),
        unit.clone(),
        linked_interface,
        &mut store,
        &hierarchy,
        &resolver,
        &declarations,
    ) {
        phalcom_semantic::db::QueryOutcome::Ready(surface) => surface,
        outcome => panic!("surface prerequisite must be ready, got {outcome:?}"),
    };
    let selector = Selector::method("read", []).unwrap();
    let callable = CallableId::new(owner.clone(), selector, DispatchSide::Class);
    let mut dispatch = phalcom_semantic::dispatch::SurfaceDispatchResolver::new();
    dispatch.register_surface(owner.clone(), (*surface).clone());

    let formal_inputs = FormalQueryInputs {
        sources: &input.sources,
        linked: &input.linked,
        hierarchy: &hierarchy,
        base_resolver: &resolver,
        declarations: &declarations,
    };
    let class_def = unit
        .program
        .statements
        .iter()
        .find_map(|statement| match statement {
            phalcom_ast::ast::Statement::Class(class_def) => Some(class_def),
            _ => None,
        })
        .expect("class declaration");
    let method = match &class_def.members[0] {
        phalcom_ast::ast::ClassMember::Method(method) => method,
        member => panic!("expected method, got {member:?}"),
    };
    let body = method.body.statements().expect("method body");
    let outcome = query_callable_body_with_formal_inputs(
        &mut db,
        callable.clone(),
        body,
        method.range,
        &mut store,
        &hierarchy,
        &resolver,
        &declarations,
        &dispatch,
        module.clone(),
        QueryBudget::default(),
        &CancellationToken::new(),
        Some(&formal_inputs),
    );
    assert!(matches!(outcome, phalcom_semantic::db::QueryOutcome::Ready(_)), "body outcome: {outcome:?}");
    assert!(db.product(&QueryKey::DeclarationShell(owner.clone())).is_some());
    assert!(db.product(&QueryKey::LinkedInterface(module.clone())).is_some());
    assert!(db.product(&QueryKey::DeclarationSurface(owner.clone())).is_some());
    assert!(db.product(&QueryKey::CallableSignature(callable.clone())).is_some());
    assert!(db.product(&QueryKey::CallableBody(callable)).is_some());
}

#[test]
fn partial_source_signature_stays_surface_backed_without_truncated_callable_product() {
    let module = module_id();
    let source = r#"
class Api {
  @class
  value(_ input) -> Int { 1 }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), source, 1));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let owner = DeclarationId::new(module.clone(), "Api".into());
    let selector = Selector::method("value", [SelectorSlot::Positional]).unwrap();
    let body_callable = CallableId::new(owner.clone(), selector.clone(), DispatchSide::Class);
    let signature_key = QueryKey::CallableSignature(body_callable.clone());
    let body_key = QueryKey::CallableBody(body_callable);

    assert!(
        session.db().product(&signature_key).is_none(),
        "unknown parameter type must not be dropped to fabricate a shorter canonical signature"
    );

    let body_dependencies = dependency_keys(&session, &body_key);
    assert!(body_dependencies.contains(&QueryKey::DeclarationSurface(owner)));
    assert!(!body_dependencies.contains(&signature_key));
}
