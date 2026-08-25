//! Integration tests verifying fine-grained callable semantic dependency tracking and invalidation (Spec 04.5 / Wave 5 Section 8.8).

use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{InterfaceBuilder, LinkedExport, LinkedExportTarget, LinkedModuleInterface};
use phalcom_modules::linker::{GlobalBindingId, ImportBindingId, LinkedModule, LinkedProgram, ModuleBindingLayout, SymbolId};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::advisory::ValueShape;
use phalcom_semantic::db::QueryKey;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::BTreeMap;
use std::sync::Arc;

fn single_module_input(module: ModuleId, source_code: &str, generation: u64) -> SemanticWorkspaceInput {
    let parse_res = phalcom_ast::parse(source_code, 0);
    let program = Arc::new(parse_res.program);
    let _ = InterfaceBuilder::build(module.clone(), ModuleKind::Module, &program);

    let linked_mod = LinkedModule {
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
    modules.insert(module.clone(), linked_mod);

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
        Arc::new(ParsedModuleUnit::new(module, ModuleKind::Module, None, Arc::from(source_code), program)),
    );

    SemanticWorkspaceInput { linked, sources, generation }
}

#[test]
fn case_a_unchanged_caller_changed_callee_signature_recomputes() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();

    let src1 = r#"
class Api {
  @class value() -> Int { 1 }
}

class Consumer {
  @class read() -> Int {
    Api.value()
  }
}
"#;
    let input1 = single_module_input(module.clone(), src1, 1);
    let update1 = session.update(input1);
    assert_eq!(update1.stats.callables_recomputed, 2);
    assert_eq!(update1.stats.callables_reused, 0);

    let consumer_decl = DeclarationId::new(module.clone(), "Consumer".into());
    let consumer_read_id = CallableId::new(consumer_decl, Selector::method("read", []).unwrap(), DispatchSide::Class);
    let consumer_analysis_v1 = update1
        .snapshot
        .callable_analyses
        .get(&consumer_read_id)
        .cloned()
        .expect("Consumer.read analysis v1");
    // Revision 2: Callee return type changes from Int to String
    let src2 = r#"
class Api {
  @class value() -> String { "hello" }
}

class Consumer {
  @class read() -> Int {
    Api.value()
  }
}
"#;
    let input2 = single_module_input(module.clone(), src2, 2);
    let update2 = session.update(input2);

    // Both callee and caller must recompute
    assert_eq!(update2.stats.callables_recomputed, 2);
    assert_eq!(update2.stats.callables_reused, 0);

    let consumer_analysis_v2 = update2
        .snapshot
        .callable_analyses
        .get(&consumer_read_id)
        .cloned()
        .expect("Consumer.read analysis v2");
    assert!(
        !Arc::ptr_eq(&consumer_analysis_v1, &consumer_analysis_v2),
        "Caller analysis Arc must NOT be reused when callee signature changes"
    );
}

#[test]
fn case_b_callee_body_change_signature_unchanged_reuses_caller() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();

    let src1 = r#"
class Api {
  @class value() -> Int { 1 }
}

class Consumer {
  @class read() -> Int {
    Api.value()
  }
}
"#;
    let input1 = single_module_input(module.clone(), src1, 1);
    let update1 = session.update(input1);
    assert_eq!(update1.stats.callables_recomputed, 2);

    let consumer_decl = DeclarationId::new(module.clone(), "Consumer".into());
    let consumer_read_id = CallableId::new(consumer_decl, Selector::method("read", []).unwrap(), DispatchSide::Class);
    let consumer_analysis_v1 = update1
        .snapshot
        .callable_analyses
        .get(&consumer_read_id)
        .cloned()
        .expect("Consumer.read analysis v1");

    // Revision 2: Change Api.value body 1 -> 2 while signature -> Int stays
    let src2 = r#"
class Api {
  @class value() -> Int { 2 }
}

class Consumer {
  @class read() -> Int {
    Api.value()
  }
}
"#;
    let input2 = single_module_input(module.clone(), src2, 2);
    let update2 = session.update(input2);

    // Api.value recomputes, Consumer.read is REUSED
    assert_eq!(update2.stats.callables_recomputed, 1, "Only Api.value body recomputes");
    assert_eq!(update2.stats.callables_reused, 1, "Consumer.read must be reused");

    let consumer_analysis_v2 = update2
        .snapshot
        .callable_analyses
        .get(&consumer_read_id)
        .cloned()
        .expect("Consumer.read analysis v2");
    assert!(
        Arc::ptr_eq(&consumer_analysis_v1, &consumer_analysis_v2),
        "Consumer.read CallableAnalysis Arc MUST be reused"
    );
}

#[test]
fn case_c_field_annotation_change_recomputes_reader() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();

    let src1 = r#"
class Data {
  _value: Int = 1
  read() -> Int { _value }
}
"#;
    let input1 = single_module_input(module.clone(), src1, 1);
    let update1 = session.update(input1);
    assert_eq!(update1.stats.callables_recomputed, 1);

    let data_decl = DeclarationId::new(module.clone(), "Data".into());
    let data_read_id = CallableId::new(data_decl, Selector::method("read", []).unwrap(), DispatchSide::Instance);
    let data_read_v1 = update1.snapshot.callable_analyses.get(&data_read_id).cloned().expect("Data.read v1");

    // Revision 2: Change _value: Int -> _value: String
    let src2 = r#"
class Data {
  _value: String = "1"
  read() -> Int { _value }
}
"#;
    let input2 = single_module_input(module.clone(), src2, 2);
    let update2 = session.update(input2);

    assert_eq!(update2.stats.callables_recomputed, 1, "Data.read must recompute because field type changed");
    assert_eq!(update2.stats.callables_reused, 0);

    let data_read_v2 = update2.snapshot.callable_analyses.get(&data_read_id).cloned().expect("Data.read v2");
    assert!(
        !Arc::ptr_eq(&data_read_v1, &data_read_v2),
        "Data.read Arc must NOT be reused when field surface changes"
    );
}

#[test]
fn case_d_superclass_change_recomputes_inherited_call() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();

    let src1 = r#"
class A {
  foo() -> Int { 10 }
}

class B {
  foo() -> String { "b" }
}

class Child is A {
  test() -> Int { self.foo() }
}
"#;
    let input1 = single_module_input(module.clone(), src1, 1);
    let update1 = session.update(input1);
    assert_eq!(update1.stats.callables_recomputed, 3);

    let child_decl = DeclarationId::new(module.clone(), "Child".into());
    let child_test_id = CallableId::new(child_decl, Selector::method("test", []).unwrap(), DispatchSide::Instance);
    let child_test_v1 = update1.snapshot.callable_analyses.get(&child_test_id).cloned().expect("Child.test v1");

    // Revision 2: Change superclass of Child from A to B
    let src2 = r#"
class A {
  foo() -> Int { 10 }
}

class B {
  foo() -> String { "b" }
}

class Child is B {
  test() -> Int { self.foo() }
}
"#;
    let input2 = single_module_input(module.clone(), src2, 2);
    let update2 = session.update(input2);

    let child_test_v2 = update2.snapshot.callable_analyses.get(&child_test_id).cloned().expect("Child.test v2");
    assert!(
        !Arc::ptr_eq(&child_test_v1, &child_test_v2),
        "Child.test must recompute when superclass changes"
    );
}

#[test]
fn case_e_imported_linked_surface_change_recomputes_importer() {
    let proj_id = ResolvedProjectId::from_raw(1);
    let api_mod = ModuleId::resolved(proj_id, ModulePath::from_components(vec![ModuleComponent::from_identifier("api").unwrap()]));
    let client_mod = ModuleId::resolved(proj_id, ModulePath::from_components(vec![ModuleComponent::from_identifier("client").unwrap()]));

    let mut session = SemanticWorkspaceSession::new();

    let api_src1 = r#"
class Service {
  @class serve() -> Int { 1 }
}
export Service
"#;
    let client_src = r#"
import app.api.Service
class Client {
  run() -> Int {
    Service.serve()
  }
}
"#;

    let build_multi_input = |api_code: &str, generation: u64| {
        let api_parse = phalcom_ast::parse(api_code, 0);
        let client_parse = phalcom_ast::parse(client_src, 0);
        let api_prog = Arc::new(api_parse.program);
        let client_prog = Arc::new(client_parse.program);

        let mut sources = BTreeMap::new();
        sources.insert(
            api_mod.clone(),
            Arc::new(ParsedModuleUnit::new(api_mod.clone(), ModuleKind::Module, None, Arc::from(api_code), api_prog)),
        );
        sources.insert(
            client_mod.clone(),
            Arc::new(ParsedModuleUnit::new(
                client_mod.clone(),
                ModuleKind::Module,
                None,
                Arc::from(client_src),
                client_prog,
            )),
        );

        let mut api_exports = BTreeMap::new();
        api_exports.insert(
            "Service".into(),
            LinkedExport {
                public_name: "Service".into(),
                target: LinkedExportTarget::Binding(SymbolId {
                    module: api_mod.clone(),
                    name: "Service".into(),
                }),
                range: phalcom_common::range::SourceRange::default(),
            },
        );

        let mut modules = BTreeMap::new();
        modules.insert(
            api_mod.clone(),
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: api_mod.clone(),
                    kind: ModuleKind::Module,
                    exports: api_exports,
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: BTreeMap::from([("Service".into(), GlobalBindingId(0))]),
                    imports: BTreeMap::new(),
                },
                linked_reads: Vec::new(),
                runtime_dependencies: Vec::new(),
            },
        );

        let mut client_imports = BTreeMap::new();
        client_imports.insert("Service".into(), ImportBindingId(0));
        modules.insert(
            client_mod.clone(),
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: client_mod.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::new(),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: BTreeMap::from([("Client".into(), GlobalBindingId(0))]),
                    imports: client_imports,
                },
                linked_reads: vec![phalcom_modules::linker::LinkedReadSpec::Binding(SymbolId {
                    module: api_mod.clone(),
                    name: "Service".into(),
                })],
                runtime_dependencies: vec![api_mod.clone()],
            },
        );

        let linked = Arc::new(LinkedProgram {
            universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
            modules,
            graphs: phalcom_modules::graph::ModuleGraphs::default(),
            entry: client_mod.clone(),
            initialization_order: vec![api_mod.clone(), client_mod.clone()],
        });

        SemanticWorkspaceInput { linked, sources, generation }
    };

    let input1 = build_multi_input(api_src1, 1);
    let update1 = session.update(input1);

    let client_decl = DeclarationId::new(client_mod.clone(), "Client".into());
    let client_run_id = CallableId::new(client_decl, Selector::method("run", []).unwrap(), DispatchSide::Instance);
    let client_run_v1 = update1.snapshot.callable_analyses.get(&client_run_id).cloned().expect("Client.run v1");
    let client_advisory_v1 = update1.snapshot.advisory().callable(&client_run_id).expect("Client.run advisory v1");
    assert_eq!(
        client_advisory_v1.return_fact.shape,
        ValueShape::Instance(DeclarationId::new(phalcom_modules::ModuleId::core(), "Int".into()))
    );

    // Revision 2: Api module changes Service.serve return type from Int to String
    let api_src2 = r#"
class Service {
  @class serve() -> String { "hello" }
}
export Service
"#;
    let input2 = build_multi_input(api_src2, 2);
    let update2 = session.update(input2);

    let client_run_v2 = update2.snapshot.callable_analyses.get(&client_run_id).cloned().expect("Client.run v2");
    assert!(
        !Arc::ptr_eq(&client_run_v1, &client_run_v2),
        "Client.run must recompute when imported Service signature changes"
    );
    let client_advisory_v2 = update2.snapshot.advisory().callable(&client_run_id).expect("Client.run advisory v2");
    assert_ne!(
        client_advisory_v1.return_fact.shape, client_advisory_v2.return_fact.shape,
        "cross-module advisory return must follow imported callable product"
    );
}

#[test]
fn case_f_unrelated_edit_reuses_unaffected_callables() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();

    let src1 = r#"
class Worker {
  compute() -> Int { 100 }
}

class Unrelated {
  hello() -> String { "world" }
}
"#;
    let input1 = single_module_input(module.clone(), src1, 1);
    let update1 = session.update(input1);
    assert_eq!(update1.stats.callables_recomputed, 2);
    assert_eq!(update1.stats.callables_reused, 0);

    let worker_decl = DeclarationId::new(module.clone(), "Worker".into());
    let worker_compute_id = CallableId::new(worker_decl, Selector::method("compute", []).unwrap(), DispatchSide::Instance);
    let worker_compute_v1 = update1.snapshot.callable_analyses.get(&worker_compute_id).cloned().expect("Worker.compute v1");

    // Revision 2: Edit Unrelated.hello body only
    let src2 = r#"
class Worker {
  compute() -> Int { 100 }
}

class Unrelated {
  hello() -> String { "changed" }
}
"#;
    let input2 = single_module_input(module.clone(), src2, 2);
    let update2 = session.update(input2);

    assert_eq!(update2.stats.callables_recomputed, 1, "Only Unrelated.hello recomputes");
    assert_eq!(update2.stats.callables_reused, 1, "Worker.compute must be reused");

    let worker_compute_v2 = update2.snapshot.callable_analyses.get(&worker_compute_id).cloned().expect("Worker.compute v2");
    assert!(
        Arc::ptr_eq(&worker_compute_v1, &worker_compute_v2),
        "Worker.compute Arc MUST be reused across unrelated edits"
    );
}

#[test]
fn case_g_constructor_body_depends_on_class_side_constructor_signature() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();

    let source = r#"
class Base {
  @constructor
  new() {}
}
"#;

    let update = session.update(single_module_input(module.clone(), source, 1));
    assert!(!update.snapshot.has_errors());

    let owner = DeclarationId::new(module, "Base".into());
    let selector = Selector::method("new", []).unwrap();
    let body_callable = CallableId::new(owner.clone(), selector.clone(), DispatchSide::Instance);
    let signature_callable = CallableId::new(owner, selector, DispatchSide::Class);
    let body_key = QueryKey::CallableBody(body_callable.clone());
    let expected_signature_key = QueryKey::CallableSignature(signature_callable);
    let wrong_instance_signature_key = QueryKey::CallableSignature(body_callable);

    let dependencies = session
        .db()
        .index()
        .dependencies_of(&body_key)
        .expect("constructor body query must retain semantic dependency edges");

    assert!(
        dependencies.iter().any(|edge| edge.dependency == expected_signature_key),
        "constructor body must depend on the class-side constructor signature it consumes"
    );
    assert!(
        dependencies.iter().all(|edge| edge.dependency != wrong_instance_signature_key),
        "constructor body must not record the synthetic instance-side body identity as its signature dependency"
    );
}

#[test]
fn case_h_builtin_seed_dispatch_does_not_require_unpublished_db_products() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let source = r#"
class Main {
  @class value() -> Int { 1 + 2 }
}
"#;

    let update = session.update(single_module_input(module.clone(), source, 1));
    let owner = DeclarationId::new(module, "Main".into());
    let callable = CallableId::new(owner, Selector::method("value", []).unwrap(), DispatchSide::Class);
    assert!(
        update.snapshot.callable_analyses.contains_key(&callable),
        "builtin dispatch must not fail closed on legacy core surfaces that have no DB query products"
    );
}

#[test]
fn case_i_unannotated_callable_uses_surface_dependency_without_missing_signature_product() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let source = r#"
class Untyped {
  value() { 1 }
}
"#;

    let update = session.update(single_module_input(module.clone(), source, 1));
    let owner = DeclarationId::new(module, "Untyped".into());
    let callable = CallableId::new(owner.clone(), Selector::method("value", []).unwrap(), DispatchSide::Instance);
    let body_key = QueryKey::CallableBody(callable.clone());

    assert!(update.snapshot.callable_analyses.contains_key(&callable));
    let dependencies = session
        .db()
        .index()
        .dependencies_of(&body_key)
        .expect("unannotated callable body must retain its declaration-surface dependency");
    assert!(dependencies.iter().any(|edge| edge.dependency == QueryKey::DeclarationSurface(owner.clone())));
    assert!(dependencies.iter().all(|edge| edge.dependency != QueryKey::CallableSignature(callable.clone())));
}
