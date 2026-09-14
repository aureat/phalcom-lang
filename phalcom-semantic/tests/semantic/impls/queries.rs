use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::LinkedModuleInterface;
use phalcom_modules::linker::{LinkedModule, LinkedProgram, ModuleBindingLayout};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::db::QueryKey;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::impls::CallableDefinitionOrigin;
use phalcom_semantic::session::{SemanticWorkspaceSession, SemanticWorkspaceUpdate};
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::BTreeMap;
use std::sync::Arc;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

fn single_module_input(module: ModuleId, source: &str) -> SemanticWorkspaceInput {
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);

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
            Arc::from(source),
            Arc::new(parsed.program),
        )),
    );
    SemanticWorkspaceInput::new(linked, sources, 1)
}

fn analyze_impl(source: &str, selector: Selector) -> (SemanticWorkspaceSession, SemanticWorkspaceUpdate, CallableId) {
    let module = test_module();
    let callable = CallableId::new(DeclarationId::new(module.clone(), "User".into()), selector, DispatchSide::Instance);
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module, source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);
    (session, output, callable)
}

fn owner(name: &str) -> DeclarationId {
    DeclarationId::new(test_module(), name.into())
}

fn impl_callable(owner_name: &str, selector: Selector) -> CallableId {
    CallableId::new(owner(owner_name), selector, DispatchSide::Instance)
}

fn surface_fingerprint(output: &SemanticWorkspaceUpdate, declaration: &DeclarationId) -> u64 {
    phalcom_semantic::db::fingerprint::declaration_surface_product_fingerprint(output.snapshot.surfaces().get(declaration).expect("declaration surface")).raw()
}

#[test]
fn test_query_callable_signature_for_impl_method() {
    let (session, output, callable) = analyze_impl(
        "class User {}\nimpl User {\n  name() -> String { \"hello\" }\n}\n",
        Selector::method("name", []).unwrap(),
    );

    let signature = session
        .db()
        .product(&QueryKey::CallableSignature(callable.clone()))
        .and_then(|product| product.as_callable_signature())
        .expect("workspace query should publish the impl signature");
    assert_eq!(signature.callable, callable);
    assert_eq!(signature.owner, DeclarationId::new(test_module(), "User".into()));
    assert_eq!(signature.side, DispatchSide::Instance);
    assert!(output.snapshot.callable_signatures.get(&signature.callable).is_some());
    let definition = output
        .snapshot
        .callable_definitions
        .get(&callable)
        .expect("snapshot must retain accepted impl provenance for lowering");
    assert!(matches!(definition.origin, CallableDefinitionOrigin::InherentImpl(_)));
}

#[test]
fn test_query_callable_body_for_impl_method() {
    let (session, output, callable) = analyze_impl(
        "class User {}\nimpl User {\n  greet() -> String { \"hello world\" }\n}\n",
        Selector::method("greet", []).unwrap(),
    );

    let analysis = session
        .db()
        .product(&QueryKey::CallableBody(callable.clone()))
        .and_then(|product| product.as_callable_body())
        .expect("workspace query should publish the impl body");
    assert_eq!(analysis.callable, callable);
    assert!(analysis.diagnostics.is_empty());
    assert!(output.snapshot.callable_analyses.contains_key(&analysis.callable));
}

#[test]
fn test_rejected_duplicate_impl_definition_is_not_published() {
    let module = test_module();
    let callable = CallableId::new(
        DeclarationId::new(module.clone(), "User".into()),
        Selector::method("name", []).unwrap(),
        DispatchSide::Instance,
    );
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(
        module,
        "class User {}\nimpl User { name() -> String { \"first\" } }\nimpl User { name() -> String { \"second\" } }\n",
    ));
    assert!(output.snapshot.has_errors(), "duplicate impl member must diagnose");

    let definition = session
        .db()
        .product(&QueryKey::CallableDefinition(callable.clone()))
        .and_then(|product| product.as_callable_definition())
        .expect("accepted impl definition");
    assert!(matches!(definition.origin, CallableDefinitionOrigin::InherentImpl(ref id) if id.local.0 == 1));
    assert_eq!(definition.source_member_index, 0);
    let snapshot_definition = output
        .snapshot
        .callable_definitions
        .get(&callable)
        .expect("only the accepted duplicate candidate reaches lowering");
    assert!(matches!(snapshot_definition.origin, CallableDefinitionOrigin::InherentImpl(ref id) if id.local.0 == 1));
}

#[test]
fn incremental_impl_edit_matches_cold_effective_surfaces() {
    let source_v1 = "class Alpha {}\nclass Beta {}\nimpl Alpha { ping() -> String { \"a\" } }\nimpl Beta { ping() -> Int { 1 } }\n";
    let source_v2 = "class Alpha {}\nclass Beta {}\nimpl Alpha { ping() -> String { \"changed\" } }\nimpl Beta { ping() -> Int { 2 } }\n";
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(module.clone(), source_v1));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let second = session.update(single_module_input(module.clone(), source_v2));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);

    let mut cold_session = SemanticWorkspaceSession::new();
    let cold = cold_session.update(single_module_input(module, source_v2));
    assert!(!cold.snapshot.has_errors(), "cold diagnostics: {:?}", cold.snapshot.diagnostics);

    for name in ["Alpha", "Beta"] {
        let declaration = owner(name);
        assert_eq!(surface_fingerprint(&second, &declaration), surface_fingerprint(&cold, &declaration));
    }
    assert_eq!(second.snapshot.callable_definitions, cold.snapshot.callable_definitions);
}

#[test]
fn impl_body_only_edit_preserves_surface_and_reuses_unaffected_callable() {
    let module = test_module();
    let callable = impl_callable("User", Selector::method("greet", []).unwrap());
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(
        module.clone(),
        "class User {}\nimpl User { greet() -> String { \"first\" } }\nclass Other { keep() -> Bool { true } }\n",
    ));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let first_surface = surface_fingerprint(&first, &owner("User"));
    let first_signature = first.snapshot.callable_signatures.get(&callable).expect("impl signature").clone();
    let first_analysis = first.snapshot.callable_analyses.get(&callable).cloned().expect("impl body");

    let second = session.update(single_module_input(
        module,
        "class User {}\nimpl User { greet() -> String { \"second\" } }\nclass Other { keep() -> Bool { false } }\n",
    ));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    assert_eq!(first_surface, surface_fingerprint(&second, &owner("User")), "impl body bytes must not enter effective surface identity");
    assert_eq!(first_signature, *second.snapshot.callable_signatures.get(&callable).expect("reused impl signature"));
    assert!(!Arc::ptr_eq(&first_analysis, second.snapshot.callable_analyses.get(&callable).expect("refreshed impl body")));
    assert!(second.stats.callable_signatures_reused > 0, "signature product should remain reusable");
}

#[test]
fn unrelated_class_body_edit_reuses_impl_callable_and_surface() {
    let module = test_module();
    let callable = impl_callable("User", Selector::method("greet", []).unwrap());
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(
        module.clone(),
        "class User {}\nimpl User { greet() -> String { \"hi\" } }\nclass Other { keep() -> Bool { true } }\n",
    ));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let first_surface = surface_fingerprint(&first, &owner("User"));
    let first_analysis = first.snapshot.callable_analyses.get(&callable).cloned().expect("impl body");

    let second = session.update(single_module_input(
        module,
        "class User {}\nimpl User { greet() -> String { \"hi\" } }\nclass Other { keep() -> Bool { false } }\n",
    ));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    assert_eq!(first_surface, surface_fingerprint(&second, &owner("User")));
    assert!(Arc::ptr_eq(&first_analysis, second.snapshot.callable_analyses.get(&callable).expect("unrelated edit must retain impl body")));
}

#[test]
fn impl_return_annotation_and_member_addition_change_only_affected_surfaces() {
    let module = test_module();
    let ping = impl_callable("Alpha", Selector::method("ping", []).unwrap());
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(
        module.clone(),
        "class Alpha {}\nclass Beta {}\nclass Caller { run(value: Alpha) { value.ping() } }\nimpl Alpha { ping() -> String { \"a\" } }\nimpl Beta { keep() -> Bool { true } }\n",
    ));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let caller = impl_callable("Caller", Selector::method("run", [phalcom_common::selector::SelectorSlot::Label("value".into())]).unwrap());
    let first_caller = first.snapshot.callable_analyses.get(&caller).cloned().expect("caller body");

    let second = session.update(single_module_input(
        module.clone(),
        "class Alpha {}\nclass Beta {}\nclass Caller { run(value: Alpha) { value.ping() } }\nimpl Alpha { ping() -> Int { 1 } extra() -> Bool { true } }\nimpl Beta { keep() -> Bool { true } }\n",
    ));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    assert_ne!(
        first.snapshot.callable_signatures.get(&ping),
        second.snapshot.callable_signatures.get(&ping),
        "return annotation changes the accepted target signature"
    );
    assert_ne!(surface_fingerprint(&first, &owner("Alpha")), surface_fingerprint(&second, &owner("Alpha")));
    assert!(second.snapshot.surfaces().get(&owner("Alpha")).is_some_and(|surface| {
        surface.instance.callable_signatures.keys().any(|selector| selector.encode().starts_with("extra"))
    }));
    assert!(!Arc::ptr_eq(&first_caller, second.snapshot.callable_analyses.get(&caller).expect("caller body")), "target signature change invalidates dependent caller");
}

#[test]
fn adding_impl_member_does_not_invalidate_another_target_or_its_caller() {
    let module = test_module();
    let caller = impl_callable("Caller", Selector::method("run", [phalcom_common::selector::SelectorSlot::Label("value".into())]).unwrap());
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(
        module.clone(),
        "class Alpha {}\nclass Beta {}\nclass Caller { run(value: Alpha) { value.ping() } }\nimpl Alpha { ping() -> Int { 1 } }\nimpl Beta { keep() -> Bool { true } }\n",
    ));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let alpha_surface = surface_fingerprint(&first, &owner("Alpha"));
    let caller_v1 = first.snapshot.callable_analyses.get(&caller).cloned().expect("caller body");

    let second = session.update(single_module_input(
        module,
        "class Alpha {}\nclass Beta {}\nclass Caller { run(value: Alpha) { value.ping() } }\nimpl Alpha { ping() -> Int { 1 } }\nimpl Beta { keep() -> Bool { true } extra() -> Int { 2 } }\n",
    ));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    assert_eq!(alpha_surface, surface_fingerprint(&second, &owner("Alpha")));
    assert!(Arc::ptr_eq(&caller_v1, second.snapshot.callable_analyses.get(&caller).expect("unchanged caller body")));
    assert_ne!(surface_fingerprint(&first, &owner("Beta")), surface_fingerprint(&second, &owner("Beta")));
}

#[test]
fn impl_whitespace_and_range_edit_preserves_semantic_surface_fingerprint() {
    let module = test_module();
    let callable = impl_callable("User", Selector::method("greet", []).unwrap());
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(module.clone(), "class User {}\nimpl User { greet() -> String { \"hi\" } }\n"));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let first_surface = surface_fingerprint(&first, &owner("User"));
    let first_signature = phalcom_semantic::db::fingerprint::callable_signature_product_fingerprint(
        first.snapshot.callable_signatures.get(&callable).expect("impl signature"),
    );

    let second = session.update(single_module_input(
        module,
        "class User {}\n\nimpl User {\n  greet() -> String {\n    \"hi\"\n  }\n}\n",
    ));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    assert_eq!(first_surface, surface_fingerprint(&second, &owner("User")));
    assert_eq!(
        first_signature,
        phalcom_semantic::db::fingerprint::callable_signature_product_fingerprint(second.snapshot.callable_signatures.get(&callable).expect("impl signature")),
        "range movement must not change the semantic signature product"
    );
}

#[test]
fn moving_impl_target_retires_old_callable_and_publishes_new_target() {
    let module = test_module();
    let alpha_ping = impl_callable("Alpha", Selector::method("ping", []).unwrap());
    let beta_ping = impl_callable("Beta", Selector::method("ping", []).unwrap());
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(module.clone(), "class Alpha {}\nclass Beta {}\nimpl Alpha { ping() -> Int { 1 } }\n"));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    assert!(first.snapshot.callable_definitions.contains_key(&alpha_ping));

    let second = session.update(single_module_input(module, "class Alpha {}\nclass Beta {}\nimpl Beta { ping() -> Int { 1 } }\n"));
    assert!(!second.snapshot.callable_definitions.contains_key(&alpha_ping), "old target must lose the moved definition");
    assert!(second.snapshot.callable_definitions.contains_key(&beta_ping), "new target must publish the moved definition");
    assert!(!second.snapshot.surfaces().get(&owner("Alpha")).expect("Alpha surface").instance.callable_signatures.keys().any(|selector| selector.encode().starts_with("ping")));
    assert!(second.snapshot.surfaces().get(&owner("Beta")).expect("Beta surface").instance.callable_signatures.keys().any(|selector| selector.encode().starts_with("ping")));
}

#[test]
fn incremental_adding_variant_invalidates_closed_requirement_completeness() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source_v1 = "enum Expr {\n  Lit(val: Int)\n}\nimpl Expr {\n  eval() -> Int\n}\nimpl Expr::Lit(val: _) {\n  eval() -> Int { self.val }\n}\n";
    let first = session.update(single_module_input(module.clone(), source_v1));
    assert!(!first.snapshot.has_errors(), "v1 must have no diagnostics: {:?}", first.snapshot.diagnostics);

    // Adding Add variant without an exact case implementation for eval() must emit missing requirement diagnostic
    let source_v2 = "enum Expr {\n  Lit(val: Int)\n  Add(left: Expr, right: Expr)\n}\nimpl Expr {\n  eval() -> Int\n}\nimpl Expr::Lit(val: _) {\n  eval() -> Int { self.val }\n}\n";
    let second = session.update(single_module_input(module.clone(), source_v2));
    assert!(second.snapshot.has_errors(), "v2 must produce missing requirement error for Add");

    // Cold analysis must produce matching diagnostics
    let mut cold_session = SemanticWorkspaceSession::new();
    let cold = cold_session.update(single_module_input(module, source_v2));
    assert!(cold.snapshot.has_errors(), "cold v2 must produce missing requirement error");
    assert_eq!(second.snapshot.has_errors(), cold.snapshot.has_errors());
}

#[test]
fn incremental_exact_case_body_edit_preserves_unrelated_products() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source_v1 = "enum Expr {\n  Lit(val: Int)\n  Add(left: Expr, right: Expr)\n}\nimpl Expr {\n  eval() -> Int\n}\nimpl Expr::Lit(val: _) {\n  eval() -> Int { self.val }\n}\nimpl Expr::Add(left: _, right: _) {\n  eval() -> Int { self.left.eval() + self.right.eval() }\n}\n";
    let first = session.update(single_module_input(module.clone(), source_v1));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);

    let lit_var = phalcom_semantic::identity::VariantId::new(
        owner("Expr"),
        phalcom_common::selector::Selector::method("Lit", [phalcom_common::selector::SelectorSlot::Label("val".into())]).unwrap(),
    );
    let lit_callable = CallableId::new(
        phalcom_semantic::identity::CallableOwnerId::Variant(lit_var),
        Selector::method("eval", []).unwrap(),
        DispatchSide::Instance,
    );
    let first_lit_analysis = first.snapshot.callable_analyses.get(&lit_callable).cloned().expect("lit body analysis");

    // Edit only Lit body
    let source_v2 = "enum Expr {\n  Lit(val: Int)\n  Add(left: Expr, right: Expr)\n}\nimpl Expr {\n  eval() -> Int\n}\nimpl Expr::Lit(val: _) {\n  eval() -> Int { self.val + 1 }\n}\nimpl Expr::Add(left: _, right: _) {\n  eval() -> Int { self.left.eval() + self.right.eval() }\n}\n";
    let second = session.update(single_module_input(module.clone(), source_v2));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);

    let second_lit_analysis = second.snapshot.callable_analyses.get(&lit_callable).expect("updated lit body analysis");
    assert!(!Arc::ptr_eq(&first_lit_analysis, second_lit_analysis), "lit body analysis must be refreshed");

    let mut cold_session = SemanticWorkspaceSession::new();
    let cold = cold_session.update(single_module_input(module, source_v2));
    assert_eq!(second.snapshot.callable_definitions, cold.snapshot.callable_definitions);
}

#[test]
fn incremental_case_only_member_leaves_root_surface_unchanged() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source_v1 = "enum Choice {\n  A\n  B\n}\nimpl Choice::A {\n  aOnly() -> Int { 42 }\n}\n";
    let first = session.update(single_module_input(module.clone(), source_v1));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let root_surface_v1 = surface_fingerprint(&first, &owner("Choice"));

    // Add another case-only member to B
    let source_v2 = "enum Choice {\n  A\n  B\n}\nimpl Choice::A {\n  aOnly() -> Int { 42 }\n}\nimpl Choice::B {\n  bOnly() -> Int { 99 }\n}\n";
    let second = session.update(single_module_input(module.clone(), source_v2));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    let root_surface_v2 = surface_fingerprint(&second, &owner("Choice"));

    assert_eq!(root_surface_v1, root_surface_v2, "case-only members must not alter root enum declaration surface");
}
