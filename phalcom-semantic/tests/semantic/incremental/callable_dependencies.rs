//! Integration tests verifying fine-grained callable semantic dependency tracking and invalidation (Spec 04.5 / Wave 5 Section 8.8).

use super::support::single_module_input;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface};
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
        ValueShape::Instance(DeclarationId::new(phalcom_modules::ModuleId::universe_root(), "Int".into()))
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

    // Revision 3: remove the provider. The dependent callable must be part of
    // the invalidation closure rather than retaining products for a vanished
    // linked interface.
    let mut input3 = build_multi_input(api_src2, 3);
    input3.sources.remove(&api_mod);
    let mut linked3 = (*input3.linked).clone();
    linked3.modules.remove(&api_mod);
    linked3.initialization_order.retain(|module| module != &api_mod);
    input3.linked = Arc::new(linked3);
    let update3 = session.update(input3);

    assert!(!update3.snapshot.sources.contains_key(&api_mod));
    assert!(update3.invalidated.contains(&QueryKey::LinkedInterface(api_mod.clone())));
    assert!(
        update3.invalidated.contains(&QueryKey::CallableBody(client_run_id.clone())),
        "provider removal must invalidate dependent callable body"
    );
    assert!(
        update3.invalidated.contains(&QueryKey::AdvisoryCallable(client_run_id)),
        "provider removal must invalidate dependent advisory callable"
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
fn case_i_unannotated_callable_uses_canonical_signature_dependency() {
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
    let signature_key = QueryKey::CallableSignature(callable.clone());
    let body_key = QueryKey::CallableBody(callable.clone());

    assert!(update.snapshot.callable_analyses.contains_key(&callable));
    let signature = update
        .snapshot
        .callable_signatures
        .get(&callable)
        .expect("unannotated callable declaration must still publish a canonical signature");
    assert!(signature.declared_return.is_unknown());
    let dependencies = session
        .db()
        .index()
        .dependencies_of(&body_key)
        .expect("unannotated callable body must retain its canonical signature dependency");
    assert!(dependencies.iter().any(|edge| edge.dependency == signature_key));
    assert!(dependencies.iter().all(|edge| edge.dependency != QueryKey::DeclarationSurface(owner.clone())));
}

/// COMPOSED: body edits reuse callers, signature edits invalidate callers, and removal/re-addition clears stale products.
#[test]
fn dependency_edit_remove_readd_recomputes_affected_summary_deterministically() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let client_owner = DeclarationId::new(module.clone(), "Client".into());
    let client_read = CallableId::new(client_owner, Selector::method("read", []).unwrap(), DispatchSide::Class);
    let api_owner = DeclarationId::new(module.clone(), "Api".into());
    let api_value = CallableId::new(api_owner, Selector::method("value", []).unwrap(), DispatchSide::Class);
    let stable_owner = DeclarationId::new(module.clone(), "Stable".into());
    let stable_keep = CallableId::new(stable_owner, Selector::method("keep", []).unwrap(), DispatchSide::Class);

    let source_v1 = r#"
class Api {
  @class value() -> Int { 1 }
}

class Stable {
  @class keep() -> Bool { true }
}
class Client {
  @class read() { Api.value() }
}
"#;
    let update_v1 = session.update(single_module_input(module.clone(), source_v1, 1));
    let client_v1 = update_v1.snapshot.callable_analyses.get(&client_read).cloned().expect("Client.read v1");
    let stable_v1 = update_v1.snapshot.callable_analyses.get(&stable_keep).cloned().expect("Stable.keep v1");
    assert!(update_v1.snapshot.callable_analyses.contains_key(&api_value));

    let source_v2 = r#"
class Api {
  @class value() -> Int { 2 }
}
class Stable {
  @class keep() -> Bool { true }
}
class Client {
  @class read() { Api.value() }
}
"#;
    let update_v2 = session.update(single_module_input(module.clone(), source_v2, 2));
    assert_eq!(update_v2.stats.callables_recomputed, 1, "body-only edit should recompute Api.value");
    assert_eq!(update_v2.stats.callables_reused, 2, "body-only edit should reuse Client.read and Stable.keep");
    let client_v2 = update_v2.snapshot.callable_analyses.get(&client_read).cloned().expect("Client.read v2");
    assert_eq!(
        client_v1.dependency_fingerprint, client_v2.dependency_fingerprint,
        "reused caller must retain stable semantic result fingerprint"
    );
    assert!(Arc::ptr_eq(&client_v1, &client_v2), "reused caller product must retain Arc identity");
    let stable_v2 = update_v2.snapshot.callable_analyses.get(&stable_keep).expect("Stable.keep v2");
    assert!(Arc::ptr_eq(&stable_v1, stable_v2), "unaffected callable product must retain Arc identity");

    let source_v3 = r#"
class Api {
  @class value() -> String { "changed" }
}
class Stable {
  @class keep() -> Bool { true }
}
class Client {
  @class read() { Api.value() }
}
"#;
    let update_v3 = session.update(single_module_input(module.clone(), source_v3, 3));
    assert_eq!(update_v3.stats.callables_recomputed, 2, "signature edit must invalidate caller");
    let client_v3 = update_v3.snapshot.callable_analyses.get(&client_read).expect("Client.read v3");
    let string_ty = update_v3
        .snapshot
        .declarations
        .form(&DeclarationId::new(ModuleId::universe_root(), "String".into()))
        .expect("String type");
    let call_v3 = client_v3
        .expressions
        .values()
        .find(|expression| source_v3.get(expression.range.start..expression.range.end) == Some("Api.value()"))
        .expect("Client.read call v3");
    assert_eq!(call_v3.knowledge.ty(), Some(string_ty));
    assert!(!Arc::ptr_eq(&client_v1, client_v3));

    let source_v4 = r#"
class Stable {
  @class keep() -> Bool { true }
}
class Client {
  @class read() { Api.value() }
}
"#;
    let update_v4 = session.update(single_module_input(module.clone(), source_v4, 4));
    assert!(
        update_v4
            .snapshot
            .callable_analyses
            .keys()
            .all(|callable| callable.owner.name.as_ref() != "Api"),
        "removed declarations must not leave stale callable products"
    );

    let source_v5 = r#"
class Api {
  @class value() -> String { "restored" }
}
class Stable {
  @class keep() -> Bool { true }
}
class Client {
  @class read() { Api.value() }
}
"#;
    let update_v5 = session.update(single_module_input(module, source_v5, 5));
    let client_v5 = update_v5.snapshot.callable_analyses.get(&client_read).expect("Client.read v5");
    let call_v5 = client_v5
        .expressions
        .values()
        .find(|expression| source_v5.get(expression.range.start..expression.range.end) == Some("Api.value()"))
        .expect("Client.read call v5");
    assert_eq!(call_v5.knowledge.ty(), Some(string_ty));
    assert!(update_v5.snapshot.callable_analyses.contains_key(&api_value));
    assert!(update_v5.snapshot.internal_incidents.is_empty());
}

#[test]
fn inferred_return_refresh_preserves_stable_products_and_counts_final_disposition_once() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    );
    let callable = |owner: &str| {
        CallableId::new(
            DeclarationId::new(module.clone(), owner.into()),
            Selector::method("value", []).unwrap(),
            DispatchSide::Class,
        )
    };
    let leaf = callable("Leaf");
    let middle = callable("Middle");
    let top = callable("Top");
    let mut session = SemanticWorkspaceSession::new();
    let source_v1 = r#"
class Leaf { @class value() { 1 } }
class Middle { @class value() { Leaf.value() } }
class Top { @class value() { Middle.value() } }
"#;
    let first = session.update(single_module_input(module.clone(), source_v1, 1));
    let middle_v1 = first.snapshot.callable_analyses.get(&middle).cloned().expect("Middle v1");
    let top_v1 = first.snapshot.callable_analyses.get(&top).cloned().expect("Top v1");

    let source_v2 = r#"
class Leaf { @class value() { 2 } }
class Middle { @class value() { Leaf.value() } }
class Top { @class value() { Middle.value() } }
"#;
    let second = session.update(single_module_input(module, source_v2, 2));
    let middle_v2 = second.snapshot.callable_analyses.get(&middle).expect("Middle v2");
    let top_v2 = second.snapshot.callable_analyses.get(&top).expect("Top v2");
    assert!(second.snapshot.callable_analyses.contains_key(&leaf));
    assert!(Arc::ptr_eq(&middle_v1, middle_v2), "stable refreshed Middle product must retain Arc identity");
    assert!(Arc::ptr_eq(&top_v1, top_v2), "stable refreshed Top product must retain Arc identity");
    assert_eq!(second.stats.callables_recomputed, 3, "Leaf, Middle, and Top are each finally recomputed once");
    assert_eq!(second.stats.callables_reused, 0, "a refreshed callable must not also be counted reused");
}
