use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface};
use phalcom_modules::linker::{GlobalBindingId, ImportBindingId, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout, SymbolId};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::db::{CancellationToken, QueryBudget, QueryKey};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, CallableOwnerId, DeclarationId, DispatchSide};
use phalcom_semantic::impls::{CallableDefinitionOrigin, ConformanceTarget};
use phalcom_semantic::session::{SemanticWorkspaceSession, SemanticWorkspaceUpdate};
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::traits::TraitRef;
use phalcom_semantic::types::environment::TypeEnvironment;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::BTreeMap;
use std::sync::Arc;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

fn named_module(name: &str) -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(42),
        ModulePath::from_components(vec![ModuleComponent::from_identifier(name).expect("valid test module name")]),
    )
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
    let output = session.update(single_module_input(module.clone(), source));
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
fn explicit_conformance_is_indexed_without_polluting_inherent_surface() {
    let module = test_module();
    let source = "trait Tagged { tag -> String }\nclass User {}\nimpl Tagged for User { tag -> String { \"tagged\" } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session
        .update_with_budget_and_cancel(single_module_input(module.clone(), source), QueryBudget::default(), &CancellationToken::new())
        .expect("semantic update should complete");
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let trait_ref = TraitRef::new(
        DeclarationId::new(module.clone(), "Tagged".into()),
        Vec::<phalcom_semantic::TypeId>::new().into_boxed_slice(),
    );
    let user = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "User".into()))
        .expect("User type");
    let mut store = (*output.snapshot.store).clone();
    let matches = output.snapshot.conformance_index.query_exact(&mut store, user, &trait_ref);
    assert_eq!(matches.len(), 1, "expected one exact conformance head");
    assert_eq!(matches[0].impl_id.module, module);
    assert!(
        !output
            .snapshot
            .surfaces()
            .get(&DeclarationId::new(test_module(), "User".into()))
            .unwrap()
            .instance
            .callable_signatures
            .contains_key(&Selector::getter("tag").unwrap())
    );
}

#[test]
fn trait_dispatch_index_buckets_eligible_requirements_by_target_selector_and_side() {
    let module = test_module();
    let source = "trait Tagged { tag -> String }\nclass User {}\nimpl Tagged for User { tag -> String { \"tagged\" } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let user = DeclarationId::new(module, "User".into());
    let family = phalcom_semantic::TraitDispatchTargetFamily::Declaration(user);
    let candidates = output
        .snapshot
        .trait_dispatch_index
        .candidates_for(&family, &Selector::getter("tag").expect("tag selector"), DispatchSide::Instance);
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].requirement.selector.encode(), "tag");
    assert_eq!(output.snapshot.trait_dispatch_index.len(), 1);
}

#[test]
fn trait_dispatch_query_returns_exact_proven_requirement_selection() {
    let module = test_module();
    let source = "trait Tagged { tag -> String }\nclass User {}\nimpl Tagged for User { tag -> String { \"tagged\" } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let user = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "User".into()))
        .expect("User type");
    let resolution = output
        .snapshot
        .resolve_trait_evidenced_candidates(user, &Selector::getter("tag").expect("tag selector"), DispatchSide::Instance);
    let phalcom_semantic::TraitDispatchResolution::Found(selection) = resolution else {
        panic!("expected proven trait dispatch selection, got {resolution:?}");
    };
    assert_eq!(selection.exact_target, user);
    assert_eq!(selection.requirement.owner, DeclarationId::new(module, "Tagged".into()));
    assert!(matches!(
        selection.selection,
        phalcom_semantic::impls::RequirementSelectionTemplate::ConformanceCallable { .. }
    ));
}

#[test]
fn trait_dispatch_query_matches_the_exact_generic_target_application() {
    let module = test_module();
    let source = "trait Tagged { tag -> String }\nclass Value<T> {}\nclass Text {}\nclass Number {}\nimpl Tagged for Value<Text> { tag -> String { \"text\" } }\nimpl Tagged for Value<Number> { tag -> String { \"number\" } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let value = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "Value".into()))
        .expect("Value type");
    let text = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "Text".into()))
        .expect("Text type");
    let number = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "Number".into()))
        .expect("Number type");
    let mut store = (*output.snapshot.store).clone();
    let value_text = store.apply_type_form(value, &[text]).expect("Value<Text>");
    let value_number = store.apply_type_form(value, &[number]).expect("Value<Number>");
    let selector = Selector::getter("tag").expect("tag selector");
    let text_result = phalcom_semantic::trait_dispatch::resolve_trait_evidenced_candidates(
        &output.snapshot.trait_dispatch_index,
        &output.snapshot.conformance_index,
        &output.snapshot.conformance_witness_plans,
        &output.snapshot.trait_surfaces,
        &output.snapshot.declarations,
        &mut store,
        output.snapshot.hierarchy.as_ref(),
        value_text,
        &selector,
        DispatchSide::Instance,
    );
    let number_result = phalcom_semantic::trait_dispatch::resolve_trait_evidenced_candidates(
        &output.snapshot.trait_dispatch_index,
        &output.snapshot.conformance_index,
        &output.snapshot.conformance_witness_plans,
        &output.snapshot.trait_surfaces,
        &output.snapshot.declarations,
        &mut store,
        output.snapshot.hierarchy.as_ref(),
        value_number,
        &selector,
        DispatchSide::Instance,
    );
    let phalcom_semantic::TraitDispatchResolution::Found(text_selection) = text_result else {
        panic!("expected text conformance, got {text_result:?}");
    };
    let phalcom_semantic::TraitDispatchResolution::Found(number_selection) = number_result else {
        panic!("expected number conformance, got {number_result:?}");
    };
    assert_ne!(text_selection.source_impl, number_selection.source_impl);
    assert_eq!(text_selection.exact_target, value_text);
    assert_eq!(number_selection.exact_target, value_number);
}

#[test]
fn trait_dispatch_shared_inherent_witnesses_converge() {
    let module = test_module();
    let source = "trait Named { name -> String }\ntrait DisplayNamed { name -> String }\nclass User { name -> String { \"user\" } }\nimpl Named for User {}\nimpl DisplayNamed for User {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);
    let user = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module, "User".into()))
        .expect("User type");
    let resolution = output
        .snapshot
        .resolve_trait_evidenced_candidates(user, &Selector::getter("name").expect("name selector"), DispatchSide::Instance);
    assert!(
        matches!(resolution, phalcom_semantic::TraitDispatchResolution::Found(_)),
        "expected convergence: {resolution:?}"
    );
}

#[test]
fn trait_dispatch_competing_defaults_are_ambiguous() {
    let module = test_module();
    let source = "trait First { tag -> String { \"first\" } }\ntrait Second { tag -> String { \"second\" } }\nclass User {}\nimpl First for User {}\nimpl Second for User {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);
    let user = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module, "User".into()))
        .expect("User type");
    let resolution = output
        .snapshot
        .resolve_trait_evidenced_candidates(user, &Selector::getter("tag").expect("tag selector"), DispatchSide::Instance);
    assert!(
        matches!(resolution, phalcom_semantic::TraitDispatchResolution::Ambiguous(ref candidates) if candidates.len() == 2),
        "expected default ambiguity: {resolution:?}"
    );
}

#[test]
fn trait_dispatch_distinct_conformance_witnesses_are_ambiguous() {
    let module = test_module();
    let source = "trait First { tag -> String }\ntrait Second { tag -> String }\nclass User {}\nimpl First for User { tag -> String { \"first\" } }\nimpl Second for User { tag -> String { \"second\" } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);
    let user = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module, "User".into()))
        .expect("User type");
    let resolution = output
        .snapshot
        .resolve_trait_evidenced_candidates(user, &Selector::getter("tag").expect("tag selector"), DispatchSide::Instance);
    assert!(
        matches!(resolution, phalcom_semantic::TraitDispatchResolution::Ambiguous(ref candidates) if candidates.len() == 2),
        "expected witness ambiguity: {resolution:?}"
    );
}

#[test]
fn trait_dispatch_exact_enum_case_does_not_propagate_to_root_or_sibling() {
    let module = test_module();
    let source = "trait Tagged { code -> Int }\nenum Status { Ready Waiting }\nimpl Tagged for Status::Ready { code -> Int { 1 } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let status_decl = DeclarationId::new(module.clone(), "Status".into());
    let status = output.snapshot.declarations.form(&status_decl).expect("Status type");
    let (_, contribution) = output.snapshot.conformance_index.iter().next().expect("case conformance");
    let phalcom_semantic::impls::ConformanceTarget::ExactEnumCase(variant) = &contribution.target else {
        panic!("expected exact enum-case conformance");
    };
    let mut store = (*output.snapshot.store).clone();
    let ready = store.exact_case_type(variant, status).expect("Status::Ready type");
    let selector = Selector::getter("code").expect("code selector");
    let ready_result = phalcom_semantic::trait_dispatch::resolve_trait_evidenced_candidates(
        &output.snapshot.trait_dispatch_index,
        &output.snapshot.conformance_index,
        &output.snapshot.conformance_witness_plans,
        &output.snapshot.trait_surfaces,
        &output.snapshot.declarations,
        &mut store,
        output.snapshot.hierarchy.as_ref(),
        ready,
        &selector,
        DispatchSide::Instance,
    );
    assert!(
        matches!(ready_result, phalcom_semantic::TraitDispatchResolution::Found(_)),
        "ready case should resolve: {ready_result:?}"
    );
    let root_result = phalcom_semantic::trait_dispatch::resolve_trait_evidenced_candidates(
        &output.snapshot.trait_dispatch_index,
        &output.snapshot.conformance_index,
        &output.snapshot.conformance_witness_plans,
        &output.snapshot.trait_surfaces,
        &output.snapshot.declarations,
        &mut store,
        output.snapshot.hierarchy.as_ref(),
        status,
        &selector,
        DispatchSide::Instance,
    );
    assert!(
        matches!(root_result, phalcom_semantic::TraitDispatchResolution::Missing),
        "root must not inherit case conformance: {root_result:?}"
    );
}

#[test]
fn ordinary_body_dispatch_consumes_trait_evidence_after_inherent_miss() {
    let module = test_module();
    let source =
        "trait Tagged { tag -> String { \"default\" } }\nclass User {}\nimpl Tagged for User {}\nclass Caller { read(_ user: User) -> String { user.tag } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);
    let caller = DeclarationId::new(module, "Caller".into());
    assert!(
        output.snapshot.callable_analyses.iter().any(|(callable, analysis)| {
            callable.try_declaration_owner() == Some(&caller) && analysis.expressions.values().any(|expression| expression.trait_dispatch.is_some())
        }),
        "ordinary call must publish trait dispatch evidence"
    );
}

#[test]
fn ordinary_body_dispatch_reports_ambiguous_trait_evidence_at_expression_boundary() {
    let module = test_module();
    let source = "trait First { tag -> String { \"first\" } }\ntrait Second { tag -> String { \"second\" } }\nclass User {}\nimpl First for User {}\nimpl Second for User {}\nclass Caller { read(_ user: User) -> String { user.tag } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    let diagnostics = output
        .snapshot
        .all_diagnostics()
        .filter(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::TraitDispatchAmbiguous)
        .collect::<Vec<_>>();
    assert_eq!(
        diagnostics.len(),
        1,
        "expected one ambiguity diagnostic: {:?}",
        output.snapshot.all_diagnostics().collect::<Vec<_>>()
    );
    assert_eq!(
        diagnostics[0].notes.len(),
        2,
        "each proven candidate needs a stable identity note: {diagnostics:#?}"
    );
    assert!(
        diagnostics[0]
            .notes
            .iter()
            .all(|note| note.contains("exact target") && note.contains("selector"))
    );
    assert_eq!(diagnostics[0].labels.len(), 2, "each source conformance should be labeled: {diagnostics:#?}");

    let caller = DeclarationId::new(module, "Caller".into());
    let caller_analysis = output
        .snapshot
        .callable_analyses
        .iter()
        .find_map(|(callable, analysis)| (callable.try_declaration_owner() == Some(&caller)).then_some(analysis))
        .expect("Caller.read analysis");
    let ambiguous_expression = caller_analysis
        .expressions
        .values()
        .find(|expression| expression.trait_dispatch_candidates.as_ref().is_some_and(|candidates| candidates.len() == 2))
        .expect("ambiguous expression should retain both proven candidates");
    let diagnostic_cause = diagnostics[0].root_cause.expect("ambiguity diagnostic cause");
    assert!(ambiguous_expression.causal_invalidity.contains(diagnostic_cause));
    assert!(
        ambiguous_expression.trait_dispatch.is_none(),
        "ambiguous dispatch must not publish a selected target"
    );
}

#[test]
fn trait_dispatch_interaction_matrix_preserves_precedence_convergence_and_terminals() {
    let cases = [
        (
            "inherent only",
            "class User { tag -> String { \"inherent\" } }\nclass Caller { read(_ user: User) -> String { user.tag } }\n",
            false,
            false,
            None,
        ),
        (
            "trait default only",
            "trait Tagged { tag -> String { \"default\" } }\nclass User {}\nimpl Tagged for User {}\nclass Caller { read(_ user: User) -> String { user.tag } }\n",
            true,
            false,
            Some("Tagged"),
        ),
        (
            "inherent plus default",
            "trait Tagged { tag -> String { \"default\" } }\nclass User { tag -> String { \"inherent\" } }\nimpl Tagged for User {}\nclass Caller { read(_ user: User) -> String { user.tag } }\n",
            false,
            false,
            None,
        ),
        (
            "inherent plus conformance witness",
            "trait Tagged { tag -> String }\nclass User { tag -> String { \"inherent\" } }\nimpl Tagged for User { tag -> String { \"witness\" } }\nclass Caller { read(_ user: User) -> String { user.tag } }\n",
            false,
            false,
            None,
        ),
        (
            "shared concrete witness convergence",
            "trait First { tag -> String }\ntrait Second { tag -> String }\nclass User { tag -> String { \"inherent\" } }\nimpl First for User {}\nimpl Second for User {}\nclass Caller { read(_ user: User) -> String { user.tag } }\n",
            false,
            false,
            None,
        ),
        (
            "competing defaults",
            "trait First { tag -> String { \"first\" } }\ntrait Second { tag -> String { \"second\" } }\nclass User {}\nimpl First for User {}\nimpl Second for User {}\nclass Caller { read(_ user: User) -> String { user.tag } }\n",
            false,
            true,
            None,
        ),
        (
            "incomplete conformance",
            "trait Tagged { tag -> String }\nclass User {}\nimpl Tagged for User { tag -> String }\nclass Caller { read(_ user: User) -> String { user.tag } }\n",
            false,
            false,
            None,
        ),
    ];

    for (label, source, expect_site, expect_ambiguity, expected_trait) in cases {
        let module = test_module();
        let mut session = SemanticWorkspaceSession::new();
        let output = session.update(single_module_input(module.clone(), source));
        let caller = DeclarationId::new(module.clone(), "Caller".into());
        let caller_analysis = output
            .snapshot
            .callable_analyses
            .iter()
            .find_map(|(callable, analysis)| (callable.try_declaration_owner() == Some(&caller)).then_some(analysis))
            .unwrap_or_else(|| panic!("{label}: missing Caller.read analysis"));
        let expression = caller_analysis
            .expressions
            .values()
            .find(|expression| source.get(expression.range.start..expression.range.end) == Some("user.tag"))
            .unwrap_or_else(|| panic!("{label}: missing user.tag expression"));
        let ambiguity_count = output
            .snapshot
            .all_diagnostics()
            .filter(|diagnostic| diagnostic.code == DiagnosticCode::TraitDispatchAmbiguous)
            .count();
        assert_eq!(ambiguity_count, usize::from(expect_ambiguity), "{label}: {expression:#?}");
        assert_eq!(expression.trait_dispatch.is_some(), expect_site, "{label}: {expression:#?}");
        assert_eq!(
            expression.trait_dispatch_candidates.as_ref().map(|candidates| candidates.len()),
            expect_ambiguity.then_some(2),
            "{label}: {expression:#?}"
        );
        if let Some(expected_trait) = expected_trait {
            let phalcom_semantic::trait_dispatch::TraitDispatchSite::Evidenced(selection) =
                expression.trait_dispatch.as_ref().expect("expected proven trait dispatch")
            else {
                panic!("{label}: expected evidenced trait dispatch");
            };
            assert_eq!(selection.exact_trait_ref.declaration.name, expected_trait.into(), "{label}: {selection:#?}");
            assert_eq!(
                selection.exact_target,
                output
                    .snapshot
                    .declarations
                    .form(&DeclarationId::new(module, "User".into()))
                    .expect("User type"),
                "{label}: {selection:#?}"
            );
        }
    }
}

#[test]
fn conformance_witness_plan_uses_conformance_owned_callable_identity() {
    let module = test_module();
    let source = "trait Tagged { tag -> String }\nclass User {}\nimpl Tagged for User { tag -> String { \"tagged\" } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let (_, contribution) = output.snapshot.conformance_index.iter().next().expect("conformance");
    let witness = output
        .snapshot
        .callable_signatures
        .iter()
        .find_map(|(callable, _)| (callable.conformance_owner() == Some(&contribution.impl_id)).then_some(callable.clone()))
        .expect("published conformance witness signature");
    assert!(matches!(witness.owner, CallableOwnerId::Conformance(_)));
    assert!(output.snapshot.callable_definitions.contains_key(&witness));
    assert!(
        output.snapshot.callable_analyses.contains_key(&witness),
        "conformance witness body must be analyzed"
    );

    let plan = output
        .snapshot
        .conformance_witness_plans
        .get(&contribution.impl_id)
        .expect("source witness plan");
    assert_ne!(plan.fingerprint.raw(), 0, "source witness plan must publish a semantic fingerprint");
    assert_eq!(plan.completeness, phalcom_semantic::impls::ConformanceCompleteness::Complete);
    assert_eq!(
        plan.requirement_views.len(),
        1,
        "source plan must retain the instantiated trait requirement view"
    );
    assert!(plan.requirements.values().any(|selection| {
        matches!(selection, phalcom_semantic::impls::RequirementSelectionTemplate::ConformanceCallable { callable } if callable == &witness)
    }));

    let user = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "User".into()))
        .expect("User type");
    let tagged = TraitRef::new(DeclarationId::new(module, "Tagged".into()), Vec::new().into_boxed_slice());
    let evidence = output.snapshot.conformance_evidence_for(user, &tagged).expect("exact conformance evidence");
    assert_eq!(evidence.source_impl, contribution.impl_id);
    assert_ne!(evidence.fingerprint.raw(), 0, "exact evidence must publish a semantic fingerprint");
    assert_eq!(
        evidence.requirement_views.len(),
        1,
        "exact evidence must retain the instantiated requirement view"
    );
    assert!(evidence.requirements.values().any(|selection| {
        matches!(selection, phalcom_semantic::impls::RequirementSelectionTemplate::ConformanceCallable { callable } if callable == &witness)
    }));
}

#[test]
fn trait_requirement_view_specializes_trait_parameter_without_rewriting_callable_generics() {
    let module = test_module();
    let source = "trait Tagged<T> { value -> T }\nclass User {}\nclass Marker {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let trait_id = DeclarationId::new(module.clone(), "Tagged".into());
    let marker_id = DeclarationId::new(module, "Marker".into());
    let marker = output.snapshot.declarations.form(&marker_id).expect("Marker form");
    let surface = output.snapshot.trait_surfaces.get(&trait_id).expect("trait surface");
    let member = surface.members.values().next().expect("trait requirement");
    let parameter = surface.generic_signature.as_ref().expect("generic trait signature").parameters[0];
    let mut environment = TypeEnvironment::new();
    environment.bind_param(parameter, marker);
    let mut store = (*output.snapshot.store).clone();
    let view = member.instantiate(&mut store, &environment);
    assert_eq!(view.signature.declared_return.canonical_type(), Some(marker));
    assert_eq!(view.callable, member.callable, "specialization must not mint a callable identity");
}

#[test]
fn incompatible_explicit_conformance_witness_stays_incomplete() {
    let module = test_module();
    let source = "trait Tagged { tag -> String }\nclass User {}\nimpl Tagged for User { tag -> Int { 1 } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    let plan = output.snapshot.conformance_witness_plans.values().next().expect("source witness plan");
    assert!(matches!(plan.completeness, phalcom_semantic::impls::ConformanceCompleteness::Incomplete { .. }));
    assert!(output.snapshot.diagnostics_for(&module).is_some_and(|diagnostics| {
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::ImplConformanceIncomplete)
    }));
    let user = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "User".into()))
        .expect("User form");
    let tagged = TraitRef::new(DeclarationId::new(module, "Tagged".into()), Vec::new().into_boxed_slice());
    assert!(output.snapshot.conformance_evidence_for(user, &tagged).is_none());
}

#[test]
fn missing_conformance_witness_emits_requirement_diagnostic() {
    let module = test_module();
    let source = "trait Sized { size -> Int }\nclass Empty {}\nimpl Sized for Empty {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    let plan = output.snapshot.conformance_witness_plans.values().next().expect("source witness plan");
    let phalcom_semantic::impls::ConformanceCompleteness::Incomplete { failures } = &plan.completeness else {
        panic!("expected incomplete conformance, got {:?}", plan.completeness);
    };
    assert_eq!(failures.len(), 1);
    assert!(matches!(&failures[0], phalcom_semantic::impls::ConformanceFailure::Behavioral(failure) if failure.reason.contains("no explicit witness or trait default")));
    assert!(output.snapshot.diagnostics_for(&module).is_some_and(|diagnostics| {
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == phalcom_semantic::DiagnosticCode::ImplConformanceIncomplete
                && diagnostic.message.contains("does not satisfy all requirements")
                && diagnostic.labels.iter().any(|label| label.message.contains("size"))
        })
    }));
}

#[test]
fn c4_invalid_corpus_asserts_canonical_diagnostic_families() {
    let cases = [
        (
            "duplicate exact conformance",
            "trait Tagged {}
class User {}
impl Tagged for User {}
impl Tagged for User {}
",
            DiagnosticCode::ImplConformanceOverlap,
        ),
        (
            "generic conformance overlap",
            "trait Tagged {}
class Value<T> {}
impl<T> Tagged for Value<T> {}
impl<U> Tagged for Value<U> {}
",
            DiagnosticCode::ImplConformanceOverlap,
        ),
        (
            "generic and exact overlap",
            "trait Tagged {}
class Value<T> {}
class Marker {}
impl<T> Tagged for Value<T> {}
impl Tagged for Value<Marker> {}
",
            DiagnosticCode::ImplConformanceOverlap,
        ),
        (
            "missing requirement witness",
            "trait Sized { size -> Int }
class Empty {}
impl Sized for Empty {}
",
            DiagnosticCode::ImplConformanceIncomplete,
        ),
        (
            "incompatible explicit witness",
            "trait Tagged { tag -> String }
class User {}
impl Tagged for User { tag -> Int { 1 } }
",
            DiagnosticCode::ImplConformanceIncomplete,
        ),
        (
            "visibility mismatch",
            "trait Tagged { tag -> String }
class User {}
impl Tagged for User { @private tag -> String { \"user\" } }
",
            DiagnosticCode::ImplConformanceIncomplete,
        ),
        (
            "extra conformance-local member",
            "trait Tagged { tag -> String { \"default\" } }
class User {}
impl Tagged for User { extra -> String { \"extra\" } }
",
            DiagnosticCode::ImplMemberConflict,
        ),
        (
            "duplicate explicit witness",
            "trait Tagged { tag -> String }
class User {}
impl Tagged for User {
  tag -> String { \"one\" }
  tag -> String { \"two\" }
}
",
            DiagnosticCode::ImplMemberConflict,
        ),
        (
            "bodyless explicit witness",
            "trait Tagged { tag -> String { \"default\" } }
class User {}
impl Tagged for User { tag -> String }
",
            DiagnosticCode::ImplBodylessMemberUnsupported,
        ),
        (
            "incompatible inherent candidate",
            "trait Renderable { render -> String }
class Item { render -> Int { 1 } }
impl Renderable for Item { render -> String { \"x\" } }
",
            DiagnosticCode::ImplConformanceIncomplete,
        ),
        (
            "competing trait defaults",
            "trait First { tag -> String { \"first\" } }
trait Second { tag -> String { \"second\" } }
class User {}
impl First for User {}
impl Second for User {}
class Caller { read(_ user: User) -> String { user.tag } }
",
            DiagnosticCode::TraitDispatchAmbiguous,
        ),
    ];

    for (label, source, expected) in cases {
        let module = test_module();
        let mut session = SemanticWorkspaceSession::new();
        let output = session.update(single_module_input(module, source));
        let codes = output.snapshot.all_diagnostics().map(|diagnostic| diagnostic.code).collect::<Vec<_>>();
        assert!(codes.contains(&expected), "{label}: expected {expected}, got {codes:?}");
    }
}

#[test]
fn known_witness_failure_outranks_another_requirement_uncertainty() {
    let module = test_module();
    let source = "trait Tagged {\n  tag -> String\n  size -> Int\n}\nclass Value<T> {}\nimpl<T> Value<T> where T <: Int { size -> Int { 1 } }\nimpl<T> Tagged for Value<T> { tag -> Int { 1 } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    let plan = output.snapshot.conformance_witness_plans.values().next().expect("source witness plan");
    let phalcom_semantic::impls::ConformanceCompleteness::Incomplete { failures } = &plan.completeness else {
        panic!("known incompatibility must dominate uncertainty: {:?}", plan.completeness);
    };
    assert!(failures.iter().any(|failure| matches!(failure, phalcom_semantic::impls::ConformanceFailure::Behavioral(failure) if failure.reason.contains("type relation refuted"))));
}

#[test]
fn generic_inherent_witness_is_specialized_through_exact_conformance_target() {
    let module = test_module();
    let source = "trait Valued<T> { value -> T }\nclass Box<T> { value -> T { value } }\nimpl<U> Valued<U> for Box<U> {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);
    let plan = output.snapshot.conformance_witness_plans.values().next().expect("source witness plan");
    assert!(matches!(plan.completeness, phalcom_semantic::impls::ConformanceCompleteness::Complete));
    assert!(matches!(
        plan.requirements.values().next(),
        Some(phalcom_semantic::impls::RequirementSelectionTemplate::InherentCallable { .. })
    ));
}

#[test]
fn incompatible_inherent_selector_invalidates_explicit_conformance_witness() {
    let module = test_module();
    let source = "trait Renderable { render -> String }\nclass Item { render -> Int { 1 } }\nimpl Renderable for Item { render -> String { \"x\" } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    let plan = output.snapshot.conformance_witness_plans.values().next().expect("source witness plan");
    assert!(matches!(plan.completeness, phalcom_semantic::impls::ConformanceCompleteness::Incomplete { .. }));
    assert!(output.snapshot.diagnostics_for(&module).is_some_and(|diagnostics| {
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::ImplConformanceIncomplete)
    }));
}

#[test]
fn exact_conditional_inherent_witness_can_outrank_trait_default() {
    let module = test_module();
    let source = "trait Tagged { tag -> String { \"default\" } }\nclass Value<T> {}\nclass Number {}\nclass Text {}\nimpl<T> Value<T> where T <: Number { tag -> String { \"number\" } }\nimpl<T> Tagged for Value<T> {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    let plan = output.snapshot.conformance_witness_plans.values().next().expect("source witness plan");
    assert!(matches!(plan.completeness, phalcom_semantic::impls::ConformanceCompleteness::Unknown(_)));
    let value = DeclarationId::new(module.clone(), "Value".into());
    let int = DeclarationId::new(module.clone(), "Number".into());
    let value_form = output.snapshot.declarations.form(&value).expect("Value form");
    let int_form = output.snapshot.declarations.form(&int).expect("Int form");
    let mut store = (*output.snapshot.store).clone();
    let exact_target = store.apply_type_form(value_form, &[int_form]).expect("Value<Int>");
    let tagged = TraitRef::new(DeclarationId::new(module.clone(), "Tagged".into()), Vec::new().into_boxed_slice());
    let surface = output.snapshot.trait_surfaces.get(&tagged.declaration).expect("trait surface");
    let evidence = match phalcom_semantic::impls::resolve_conformance_evidence(
        &output.snapshot.conformance_index,
        &output.snapshot.conformance_witness_plans,
        surface,
        &output.snapshot.declarations,
        &mut store,
        output.snapshot.hierarchy.as_ref(),
        exact_target,
        &tagged,
    ) {
        phalcom_semantic::impls::ConformanceResolution::Proven(evidence) => evidence,
        other => panic!("expected exact conformance evidence, got {other:?}"),
    };
    assert!(matches!(
        evidence.requirements.values().next(),
        Some(phalcom_semantic::impls::RequirementSelectionTemplate::InherentCallable { .. })
    ));

    let text = DeclarationId::new(module.clone(), "Text".into());
    let text_form = output.snapshot.declarations.form(&text).expect("Text form");
    let text_target = store.apply_type_form(value_form, &[text_form]).expect("Value<Text>");
    let text_evidence = match phalcom_semantic::impls::resolve_conformance_evidence(
        &output.snapshot.conformance_index,
        &output.snapshot.conformance_witness_plans,
        surface,
        &output.snapshot.declarations,
        &mut store,
        output.snapshot.hierarchy.as_ref(),
        text_target,
        &tagged,
    ) {
        phalcom_semantic::impls::ConformanceResolution::Proven(evidence) => evidence,
        other => panic!("expected default-backed exact evidence, got {other:?}"),
    };
    assert!(matches!(
        text_evidence.requirements.values().next(),
        Some(phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { .. })
    ));
}

#[test]
fn alpha_renamed_member_generics_are_compared_by_contract() {
    let module = test_module();
    let source = "trait Mapper<T> { map<U>(_ value: U) -> U where U == Int, U <: Int }\nclass User {}\nimpl Mapper<Int> for User { map<V>(_ value: V) -> V where V <: Int { value } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    let plan = output.snapshot.conformance_witness_plans.values().next().expect("source witness plan");
    assert_eq!(plan.completeness, phalcom_semantic::impls::ConformanceCompleteness::Complete);
    assert!(
        plan.requirements
            .values()
            .any(|selection| { matches!(selection, phalcom_semantic::impls::RequirementSelectionTemplate::ConformanceCallable { .. }) })
    );
}

#[test]
fn conformance_witness_plan_selects_trait_default_without_target_surface_pollution() {
    let module = test_module();
    let source = "trait Tagged { tag -> String { \"default\" } }\nclass User {}\nimpl Tagged for User {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let (_, contribution) = output.snapshot.conformance_index.iter().next().expect("conformance");
    let plan = output
        .snapshot
        .conformance_witness_plans
        .get(&contribution.impl_id)
        .expect("source witness plan");
    assert_eq!(plan.completeness, phalcom_semantic::impls::ConformanceCompleteness::Complete);
    assert!(
        plan.requirements
            .values()
            .any(|selection| { matches!(selection, phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { .. }) })
    );

    let user = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "User".into()))
        .expect("User type");
    let tagged = TraitRef::new(DeclarationId::new(module, "Tagged".into()), Vec::new().into_boxed_slice());
    assert!(output.snapshot.conformance_evidence_for(user, &tagged).is_some());
    assert!(
        !output
            .snapshot
            .surfaces
            .get(&DeclarationId::new(test_module(), "User".into()))
            .unwrap()
            .instance
            .callable_signatures
            .contains_key(&Selector::getter("tag").unwrap())
    );
}

#[test]
fn conformance_witness_plan_prefers_compatible_inherent_member_before_default() {
    let module = test_module();
    let source = "trait Tagged { tag -> String { \"default\" } }\nclass User { tag -> String { \"target\" } }\nimpl Tagged for User {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let (_, contribution) = output.snapshot.conformance_index.iter().next().expect("conformance");
    let plan = output
        .snapshot
        .conformance_witness_plans
        .get(&contribution.impl_id)
        .expect("source witness plan");
    let selection = plan.requirements.values().next().expect("tag requirement");
    assert!(matches!(
        selection,
        phalcom_semantic::impls::RequirementSelectionTemplate::InherentCallable { .. }
    ));
}

#[test]
fn conformance_plan_retains_exact_case_inherent_applicability_evidence() {
    let module = test_module();
    let source = "trait Tagged { code -> Int }\nenum Status { Ready }\nimpl Status::Ready { code -> Int { 1 } }\nimpl Tagged for Status::Ready {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);
    let plan = output.snapshot.conformance_witness_plans.values().next().expect("source witness plan");
    let selection = plan.requirements.values().next().expect("inherent witness selection");
    assert!(matches!(
        selection,
        phalcom_semantic::impls::RequirementSelectionTemplate::InherentCallable {
            conditional_impl: Some(_),
            applicability: Some(_),
            ..
        }
    ));
    let status = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "Status".into()))
        .expect("Status form");
    let (_, contribution) = output.snapshot.conformance_index.iter().next().expect("conformance");
    let ConformanceTarget::ExactEnumCase(variant) = &contribution.target else {
        panic!("expected exact enum-case target");
    };
    let mut store = (*output.snapshot.store).clone();
    let exact_target = store.exact_case_type(variant, status).expect("exact case type");
    let trait_ref = TraitRef::new(DeclarationId::new(module, "Tagged".into()), Vec::new().into_boxed_slice());
    let surface = output.snapshot.trait_surfaces.get(&trait_ref.declaration).expect("trait surface");
    let evidence = match phalcom_semantic::impls::resolve_conformance_evidence(
        &output.snapshot.conformance_index,
        &output.snapshot.conformance_witness_plans,
        surface,
        &output.snapshot.declarations,
        &mut store,
        output.snapshot.hierarchy.as_ref(),
        exact_target,
        &trait_ref,
    ) {
        phalcom_semantic::impls::ConformanceResolution::Proven(evidence) => evidence,
        other => panic!("expected exact evidence, got {other:?}"),
    };
    assert!(matches!(
        evidence.requirements.values().next().expect("exact witness selection"),
        phalcom_semantic::impls::RequirementSelectionTemplate::InherentCallable { applicability: Some(_), .. }
    ));
}

#[test]
fn conformance_plan_preserves_unknown_conditional_witness_applicability() {
    let module = test_module();
    let source = "trait Tagged { tag -> Int }\nclass Value<T> {}\nimpl<T> Value<T> where T <: Int { tag -> Int { 1 } }\nimpl<T> Tagged for Value<T> {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module, source));
    let plan = output.snapshot.conformance_witness_plans.values().next().expect("source witness plan");
    assert!(matches!(plan.completeness, phalcom_semantic::impls::ConformanceCompleteness::Unknown(_)));
}

#[test]
fn bodyless_explicit_conformance_member_is_invalid_and_cannot_fall_back_to_default() {
    let module = test_module();
    let source = "trait Tagged { tag -> String { \"default\" } }\nclass User {}\nimpl Tagged for User { tag -> String }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    let plan = output.snapshot.conformance_witness_plans.values().next().expect("source witness plan");
    assert!(matches!(plan.completeness, phalcom_semantic::impls::ConformanceCompleteness::Incomplete { .. }));
    assert!(output.snapshot.diagnostics_for(&module).is_some_and(|diagnostics| {
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::ImplBodylessMemberUnsupported)
    }));
}

#[test]
fn unmatched_explicit_conformance_member_invalidates_the_source_plan() {
    let module = test_module();
    let source = "trait Tagged { tag -> String { \"default\" } }\nclass User {}\nimpl Tagged for User { extra -> String { \"extra\" } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    let plan = output.snapshot.conformance_witness_plans.values().next().expect("source witness plan");
    assert!(plan.invalid_explicit_members);
    assert!(matches!(plan.completeness, phalcom_semantic::impls::ConformanceCompleteness::Incomplete { .. }));
    assert!(output.snapshot.diagnostics_for(&module).is_some_and(|diagnostics| {
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::ImplMemberConflict)
    }));
}

#[test]
fn conformance_witness_plan_selects_data_component_without_fabricating_getter() {
    let module = test_module();
    let source = "trait Named { name -> String }\ndata Person(name: String)\nimpl Named for Person {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let (_, contribution) = output.snapshot.conformance_index.iter().next().expect("conformance");
    let plan = output
        .snapshot
        .conformance_witness_plans
        .get(&contribution.impl_id)
        .expect("source witness plan");
    assert!(matches!(plan.completeness, phalcom_semantic::impls::ConformanceCompleteness::Complete));
    assert!(
        plan.requirements
            .values()
            .any(|selection| { matches!(selection, phalcom_semantic::impls::RequirementSelectionTemplate::DataComponent { .. }) })
    );
}

#[test]
fn exact_data_component_witness_specializes_target_and_trait_parameters() {
    let module = test_module();
    let source = "trait Named<T> { value -> T }\ndata Box<T>(_ value: T)\nclass Marker {}\nimpl<T> Named<T> for Box<T> {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let box_id = DeclarationId::new(module.clone(), "Box".into());
    let marker_id = DeclarationId::new(module.clone(), "Marker".into());
    let box_form = output.snapshot.declarations.form(&box_id).expect("Box form");
    let marker = output.snapshot.declarations.form(&marker_id).expect("Marker form");
    let mut store = (*output.snapshot.store).clone();
    let target = store.apply_type_form(box_form, &[marker]).expect("Box<Marker>");
    let trait_ref = TraitRef::new(DeclarationId::new(module, "Named".into()), vec![marker].into_boxed_slice());
    let surface = output.snapshot.trait_surfaces.get(&trait_ref.declaration).expect("trait surface");
    let evidence = match phalcom_semantic::impls::resolve_conformance_evidence(
        &output.snapshot.conformance_index,
        &output.snapshot.conformance_witness_plans,
        surface,
        &output.snapshot.declarations,
        &mut store,
        output.snapshot.hierarchy.as_ref(),
        target,
        &trait_ref,
    ) {
        phalcom_semantic::impls::ConformanceResolution::Proven(evidence) => evidence,
        other => panic!("expected exact conformance evidence, got {other:?}"),
    };
    let selection = evidence.requirements.values().next().expect("data witness selection");
    let phalcom_semantic::impls::RequirementSelectionTemplate::DataComponent { specialized_type, .. } = selection else {
        panic!("expected data component witness, got {selection:?}");
    };
    assert_eq!(*specialized_type, marker, "exact evidence must specialize the data component type");
    assert_eq!(
        evidence
            .requirement_views
            .values()
            .next()
            .expect("instantiated requirement")
            .signature
            .declared_return
            .canonical_type(),
        Some(marker),
    );
}

#[test]
fn generic_conformance_head_matches_exact_target_specialization() {
    let module = test_module();
    let source = "trait Tagged {}\nclass Value<T> {}\nclass Marker {}\nimpl<T> Tagged for Value<T> {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let value = DeclarationId::new(module.clone(), "Value".into());
    let marker = DeclarationId::new(module.clone(), "Marker".into());
    let value_form = output.snapshot.declarations.form(&value).expect("Value form");
    let marker_form = output.snapshot.declarations.form(&marker).expect("Marker form");
    let mut store = (*output.snapshot.store).clone();
    let specialized = store.apply_type_form(value_form, &[marker_form]).expect("specialized Value form");
    let trait_ref = TraitRef::new(
        DeclarationId::new(module, "Tagged".into()),
        Vec::<phalcom_semantic::TypeId>::new().into_boxed_slice(),
    );
    let matches = output.snapshot.conformance_index.query_exact(&mut store, specialized, &trait_ref);
    assert_eq!(matches.len(), 1, "generic conformance should match one exact specialization");
    assert_eq!(matches[0].impl_bindings.len(), 1, "impl generic parameter must be specialized");
    assert_eq!(matches[0].exact_target, specialized);
}

#[test]
fn overlapping_generic_and_specialized_conformances_are_rejected() {
    let module = test_module();
    let source = "trait Tagged {}\nclass Value<T> {}\nclass Marker {}\nimpl<T> Tagged for Value<T> {}\nimpl Tagged for Value<Marker> {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    let diagnostics = output.snapshot.diagnostics_for(&module).expect("module diagnostics");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::ImplConformanceOverlap),
        "expected overlap diagnostic: {diagnostics:?}"
    );
}

#[test]
fn incremental_conformance_add_edit_delete_updates_snapshot_index() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source_v1 = "trait Tagged {}\nclass User {}\nimpl Tagged for User {}\n";
    let first = session.update(single_module_input(module.clone(), source_v1));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    assert_eq!(first.snapshot.conformance_index.iter().count(), 1);

    let source_v2 = "trait Tagged {}\nclass User {}\n";
    let second = session.update(single_module_input(module.clone(), source_v2));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    assert_eq!(
        second.snapshot.conformance_index.iter().count(),
        0,
        "deleted conformance must not remain published"
    );

    let source_v3 = "trait Tagged {}\nclass User {}\nimpl Tagged for User {}\n";
    let third = session.update(single_module_input(module, source_v3));
    assert!(!third.snapshot.has_errors(), "diagnostics: {:?}", third.snapshot.diagnostics);
    assert_eq!(third.snapshot.conformance_index.iter().count(), 1, "re-added conformance must be published");
}

#[test]
fn incremental_and_cold_conformance_publication_have_the_same_head_identity() {
    let module = test_module();
    let source_v1 = "trait Tagged {}\nclass User {}\n";
    let source_v2 = "trait Tagged {}\nclass User {}\nimpl Tagged for User {}\n";
    let mut incremental = SemanticWorkspaceSession::new();
    let _ = incremental.update(single_module_input(module.clone(), source_v1));
    let incremental_output = incremental.update(single_module_input(module.clone(), source_v2));
    let mut cold = SemanticWorkspaceSession::new();
    let cold_output = cold.update(single_module_input(module.clone(), source_v2));

    let user = incremental_output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "User".into()))
        .expect("User type");
    let trait_ref = TraitRef::new(DeclarationId::new(module, "Tagged".into()), Vec::new().into_boxed_slice());
    let mut incremental_store = (*incremental_output.snapshot.store).clone();
    let mut cold_store = (*cold_output.snapshot.store).clone();
    let incremental_matches = incremental_output
        .snapshot
        .conformance_index
        .query_exact(&mut incremental_store, user, &trait_ref);
    let cold_user = cold_output
        .snapshot
        .declarations
        .form(&DeclarationId::new(test_module(), "User".into()))
        .expect("cold User type");
    let cold_matches = cold_output.snapshot.conformance_index.query_exact(&mut cold_store, cold_user, &trait_ref);
    assert_eq!(incremental_matches.len(), 1);
    assert_eq!(cold_matches.len(), 1);
    assert_eq!(incremental_matches[0].impl_id, cold_matches[0].impl_id);
    assert_eq!(
        incremental_output.snapshot.conformance_witness_plans, cold_output.snapshot.conformance_witness_plans,
        "incremental and cold witness plans must agree"
    );
}

#[test]
fn incremental_conformance_body_and_unrelated_edits_preserve_evidence_fingerprints() {
    let module = test_module();
    let source_v1 =
        "trait Tagged { tag -> String }\nclass User {}\nclass Other { keep() -> Bool { true } }\nimpl Tagged for User { tag -> String { \"one\" } }\n";
    let source_v2 =
        "trait Tagged { tag -> String }\nclass User {}\nclass Other { keep() -> Bool { false } }\nimpl Tagged for User { tag -> String { \"two\" } }\n";
    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental.update(single_module_input(module.clone(), source_v1));
    assert!(!first.snapshot.has_errors(), "v1 diagnostics: {:?}", first.snapshot.diagnostics);
    let first_impl = first.snapshot.conformance_index.iter().next().expect("conformance").1.impl_id.clone();
    let trait_ref = TraitRef::new(DeclarationId::new(module.clone(), "Tagged".into()), Vec::new().into_boxed_slice());
    let first_user = first
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "User".into()))
        .expect("User form");
    let first_evidence = first.snapshot.conformance_evidence_for(first_user, &trait_ref).expect("v1 evidence");
    let first_plan_fingerprint = first.snapshot.conformance_witness_plans.get(&first_impl).expect("v1 plan").fingerprint;
    let first_witness_analysis = first
        .snapshot
        .callable_analyses
        .iter()
        .find_map(|(callable, analysis)| (callable.conformance_owner() == Some(&first_impl)).then_some(analysis.clone()))
        .expect("v1 witness body");

    let second = incremental.update(single_module_input(module.clone(), source_v2));
    assert!(!second.snapshot.has_errors(), "v2 diagnostics: {:?}", second.snapshot.diagnostics);
    let second_user = second
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "User".into()))
        .expect("User form");
    let second_evidence = second.snapshot.conformance_evidence_for(second_user, &trait_ref).expect("v2 evidence");
    assert_eq!(
        first_plan_fingerprint,
        second.snapshot.conformance_witness_plans.get(&first_impl).expect("v2 plan").fingerprint,
        "body-only and unrelated edits must preserve the source witness-plan contract"
    );
    assert_eq!(
        first_evidence.fingerprint, second_evidence.fingerprint,
        "body-only edits must preserve exact evidence identity"
    );
    let second_witness_analysis = second
        .snapshot
        .callable_analyses
        .iter()
        .find_map(|(callable, analysis)| (callable.conformance_owner() == Some(&first_impl)).then_some(analysis.clone()))
        .expect("v2 witness body");
    assert!(
        !Arc::ptr_eq(&first_witness_analysis, &second_witness_analysis),
        "body-only witness edit must refresh body analysis"
    );

    let mut cold = SemanticWorkspaceSession::new();
    let cold_output = cold.update(single_module_input(module.clone(), source_v2));
    assert!(!cold_output.snapshot.has_errors(), "cold diagnostics: {:?}", cold_output.snapshot.diagnostics);
    let cold_user = cold_output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module, "User".into()))
        .expect("cold User form");
    let cold_evidence = cold_output.snapshot.conformance_evidence_for(cold_user, &trait_ref).expect("cold evidence");
    assert_eq!(
        second_evidence.fingerprint, cold_evidence.fingerprint,
        "incremental and cold evidence fingerprints must agree"
    );
    assert_eq!(
        second
            .snapshot
            .conformance_witness_plans
            .get(&first_impl)
            .expect("incremental plan")
            .fingerprint,
        cold_output.snapshot.conformance_witness_plans.values().next().expect("cold plan").fingerprint,
        "incremental and cold source-plan fingerprints must agree"
    );
}

#[test]
fn conformance_evidence_fingerprint_tracks_target_trait_and_visibility_contract_edits() {
    let module = test_module();
    let variants = [
        (
            "target",
            "trait Tagged { tag -> String }\nclass User { tag -> String { \"user\" } }\nimpl Tagged for User {}\n",
            true,
        ),
        (
            "target",
            "trait Tagged { tag -> String }\nclass User { tag -> Int { 1 } }\nimpl Tagged for User {}\n",
            false,
        ),
        (
            "trait",
            "trait Tagged { tag -> Int }\nclass User {}\nimpl Tagged for User { tag -> String { \"user\" } }\n",
            false,
        ),
        (
            "visibility",
            "trait Tagged { tag -> String }\nclass User {}\nimpl Tagged for User { @private tag -> String { \"user\" } }\n",
            false,
        ),
    ];
    let mut previous_fingerprint = None;
    for (label, source, complete) in variants {
        let mut session = SemanticWorkspaceSession::new();
        let output = session.update(single_module_input(module.clone(), source));
        let plan = output.snapshot.conformance_witness_plans.values().next().expect("source plan");
        assert_eq!(
            matches!(plan.completeness, phalcom_semantic::impls::ConformanceCompleteness::Complete),
            complete,
            "{label} completeness"
        );
        let mut cold = SemanticWorkspaceSession::new();
        let cold_output = cold.update(single_module_input(module.clone(), source));
        let cold_plan = cold_output.snapshot.conformance_witness_plans.values().next().expect("cold source plan");
        assert_eq!(
            plan.fingerprint, cold_plan.fingerprint,
            "{label} incremental and cold plan fingerprints must agree"
        );
        assert_eq!(
            plan.completeness, cold_plan.completeness,
            "{label} incremental and cold completeness must agree"
        );
        if let Some(previous_fingerprint) = previous_fingerprint {
            assert_ne!(
                plan.fingerprint, previous_fingerprint,
                "{label} contract edit must change the source-plan fingerprint"
            );
        }
        previous_fingerprint = Some(plan.fingerprint);
    }
}

#[test]
fn distinct_trait_reference_arguments_remain_distinct_conformance_domains() {
    let module = test_module();
    let source = "trait Converter<T> {}\nclass User {}\nclass Int {}\nclass Text {}\nimpl Converter<Int> for User {}\nimpl Converter<Text> for User {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);
    assert_eq!(output.snapshot.conformance_index.iter().count(), 2);

    let mut store = (*output.snapshot.store).clone();
    let user = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "User".into()))
        .expect("User form");
    let int = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "Int".into()))
        .expect("Int form");
    let text = output
        .snapshot
        .declarations
        .form(&DeclarationId::new(module.clone(), "Text".into()))
        .expect("Text form");
    let trait_decl = DeclarationId::new(module, "Converter".into());
    let int_matches = output
        .snapshot
        .conformance_index
        .query_exact(&mut store, user, &TraitRef::new(trait_decl.clone(), vec![int].into_boxed_slice()));
    let text_matches = output
        .snapshot
        .conformance_index
        .query_exact(&mut store, user, &TraitRef::new(trait_decl, vec![text].into_boxed_slice()));
    assert_eq!(int_matches.len(), 1);
    assert_eq!(text_matches.len(), 1);
    assert_ne!(int_matches[0].impl_id, text_matches[0].impl_id);
}

#[test]
fn exact_enum_case_conformance_retains_variant_identity() {
    let module = test_module();
    let source = "trait Tagged {}\nenum Result { Ok }\nimpl Tagged for Result::Ok {}\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);
    let (_, contribution) = output.snapshot.conformance_index.iter().next().expect("conformance contribution");
    let ConformanceTarget::ExactEnumCase(variant) = &contribution.target else {
        panic!("expected exact enum-case target");
    };
    assert_eq!(variant.owner.name.as_ref(), "Result");
    assert_eq!(variant.selector, Selector::getter("Ok").unwrap());
    let variant_target = phalcom_semantic::identity::SemanticTargetId::Variant(variant.clone());
    assert!(
        output.snapshot.source_index.occurrences_for_target(&variant_target).is_some(),
        "exact target source site must retain variant identity"
    );
}

#[test]
fn linked_modules_publish_canonical_ownership_through_reexports() {
    let traits = named_module("traits");
    let models = named_module("models");
    let facade = named_module("facade");
    let foreign = named_module("foreign");
    let trait_symbol = SymbolId {
        module: traits.clone(),
        name: "Tagged".into(),
    };
    let user_symbol = SymbolId {
        module: models.clone(),
        name: "User".into(),
    };
    let module_specs = [
        (
            traits.clone(),
            "trait Tagged {}\nexport Tagged\n",
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: traits.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::from([(
                        "Tagged".into(),
                        LinkedExport {
                            public_name: "Tagged".into(),
                            target: LinkedExportTarget::Binding(trait_symbol.clone()),
                            range: Default::default(),
                        },
                    )]),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: BTreeMap::from([("Tagged".into(), GlobalBindingId(0))]),
                    imports: BTreeMap::new(),
                },
                linked_reads: Vec::new(),
                runtime_dependencies: Vec::new(),
            },
        ),
        (
            models.clone(),
            "from facade import Tagged\nclass User {}\nimpl Tagged for User {}\nexport User\n",
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: models.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::from([(
                        "User".into(),
                        LinkedExport {
                            public_name: "User".into(),
                            target: LinkedExportTarget::Binding(user_symbol.clone()),
                            range: Default::default(),
                        },
                    )]),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: BTreeMap::from([("User".into(), GlobalBindingId(0))]),
                    imports: BTreeMap::from([("Tagged".into(), ImportBindingId(0))]),
                },
                linked_reads: vec![LinkedReadSpec::Binding(trait_symbol.clone())],
                runtime_dependencies: vec![facade.clone()],
            },
        ),
        (
            facade.clone(),
            "from traits import Tagged\nexport Tagged\n",
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: facade.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::from([(
                        "Tagged".into(),
                        LinkedExport {
                            public_name: "Tagged".into(),
                            target: LinkedExportTarget::Binding(trait_symbol.clone()),
                            range: Default::default(),
                        },
                    )]),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: BTreeMap::new(),
                    imports: BTreeMap::from([("Tagged".into(), ImportBindingId(0))]),
                },
                linked_reads: vec![LinkedReadSpec::Binding(trait_symbol.clone())],
                runtime_dependencies: vec![traits.clone()],
            },
        ),
        (
            foreign.clone(),
            "from facade import Tagged\nfrom models import User\nimpl Tagged for User {}\n",
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: foreign.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::new(),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: BTreeMap::new(),
                    imports: BTreeMap::from([("Tagged".into(), ImportBindingId(0)), ("User".into(), ImportBindingId(1))]),
                },
                linked_reads: vec![LinkedReadSpec::Binding(trait_symbol), LinkedReadSpec::Binding(user_symbol)],
                runtime_dependencies: vec![facade.clone(), models.clone()],
            },
        ),
    ];
    let mut sources = BTreeMap::new();
    let mut linked_modules = BTreeMap::new();
    for (module, source, linked_module) in module_specs {
        let parsed = phalcom_ast::parse(source, 0);
        assert!(parsed.errors.is_empty(), "parse errors in {module}: {:?}", parsed.errors);
        sources.insert(
            module.clone(),
            Arc::new(ParsedModuleUnit::new(
                module.clone(),
                ModuleKind::Module,
                None,
                Arc::from(source),
                Arc::new(parsed.program),
            )),
        );
        linked_modules.insert(module, linked_module);
    }
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: linked_modules,
        graphs: Default::default(),
        entry: models.clone(),
        initialization_order: vec![traits.clone(), facade.clone(), models.clone(), foreign.clone()],
    });
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(SemanticWorkspaceInput::new(linked, sources, 1));
    let trait_decl = DeclarationId::new(traits, "Tagged".into());
    let user_decl = DeclarationId::new(models.clone(), "User".into());
    let user = output.snapshot.declarations.form(&user_decl).expect("User type");
    let mut store = (*output.snapshot.store).clone();
    let matches = output
        .snapshot
        .conformance_index
        .query_exact(&mut store, user, &TraitRef::new(trait_decl, Vec::new().into_boxed_slice()));
    assert_eq!(matches.len(), 1, "only the target-owner conformance is eligible");
    assert_eq!(matches[0].impl_id.module, models);
    assert!(
        output.snapshot.diagnostics_for(&foreign).is_some_and(|diagnostics| diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::ImplForeignTarget)),
        "third-party conformance must be rejected even when both names arrive through linked imports"
    );
}

#[test]
fn p1_generic_specialization_matrix_and_iterable_head_stay_at_head_level() {
    let module = test_module();
    let source = "trait Tagged { tag -> String }\ntrait Iterable<Item, Cursor> {\n  iterate(_ cursor: Cursor) -> Cursor\n  iteratorValue(_ cursor: Cursor) -> Item\n}\nclass Value<T> {}\nclass Text {}\nclass Number {}\nclass Boolish {}\nclass Countdown {}\nimpl Tagged for Value<Text> { tag -> String { \"text\" } }\nimpl Tagged for Value<Number> { tag -> String { \"number\" } }\nimpl Iterable<Countdown, Countdown> for Countdown { iterate(_ cursor: Countdown) -> Countdown { cursor } iteratorValue(_ cursor: Countdown) -> Countdown { cursor } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let value = DeclarationId::new(module.clone(), "Value".into());
    let text = DeclarationId::new(module.clone(), "Text".into());
    let number = DeclarationId::new(module.clone(), "Number".into());
    let boolish = DeclarationId::new(module.clone(), "Boolish".into());
    let countdown = DeclarationId::new(module.clone(), "Countdown".into());
    let value_form = output.snapshot.declarations.form(&value).expect("Value form");
    let text_form = output.snapshot.declarations.form(&text).expect("Text form");
    let number_form = output.snapshot.declarations.form(&number).expect("Number form");
    let boolish_form = output.snapshot.declarations.form(&boolish).expect("Boolish form");
    let countdown_form = output.snapshot.declarations.form(&countdown).expect("Countdown form");
    let mut store = (*output.snapshot.store).clone();
    let value_text = store.apply_type_form(value_form, &[text_form]).expect("Value<Text>");
    let value_number = store.apply_type_form(value_form, &[number_form]).expect("Value<Number>");
    let value_boolish = store.apply_type_form(value_form, &[boolish_form]).expect("Value<Boolish>");
    let tagged = TraitRef::new(DeclarationId::new(module.clone(), "Tagged".into()), Vec::new().into_boxed_slice());
    let text_match = output.snapshot.conformance_index.query_exact(&mut store, value_text, &tagged);
    let number_match = output.snapshot.conformance_index.query_exact(&mut store, value_number, &tagged);
    let boolish_match = output.snapshot.conformance_index.query_exact(&mut store, value_boolish, &tagged);
    assert_eq!(text_match.len(), 1);
    assert_eq!(number_match.len(), 1);
    assert!(boolish_match.is_empty(), "unlisted Value<Boolish> must have no P1 candidate");
    assert_ne!(text_match[0].impl_id, number_match[0].impl_id);
    assert_eq!(text_match[0].exact_target, value_text);
    assert_eq!(number_match[0].exact_target, value_number);

    let iterable = TraitRef::new(
        DeclarationId::new(module, "Iterable".into()),
        vec![countdown_form, countdown_form].into_boxed_slice(),
    );
    let iterable_match = output.snapshot.conformance_index.query_exact(&mut store, countdown_form, &iterable);
    assert_eq!(iterable_match.len(), 1, "generic trait reference must remain an exact indexed domain");
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
    assert_eq!(
        first_surface,
        surface_fingerprint(&second, &owner("User")),
        "impl body bytes must not enter effective surface identity"
    );
    assert_eq!(
        first_signature,
        *second.snapshot.callable_signatures.get(&callable).expect("reused impl signature")
    );
    assert!(!Arc::ptr_eq(
        &first_analysis,
        second.snapshot.callable_analyses.get(&callable).expect("refreshed impl body")
    ));
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
    assert!(Arc::ptr_eq(
        &first_analysis,
        second.snapshot.callable_analyses.get(&callable).expect("unrelated edit must retain impl body")
    ));
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
    let caller = impl_callable(
        "Caller",
        Selector::method("run", [phalcom_common::selector::SelectorSlot::Label("value".into())]).unwrap(),
    );
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
        surface
            .instance
            .callable_signatures
            .keys()
            .any(|selector| selector.encode().starts_with("extra"))
    }));
    assert!(
        !Arc::ptr_eq(&first_caller, second.snapshot.callable_analyses.get(&caller).expect("caller body")),
        "target signature change invalidates dependent caller"
    );
}

#[test]
fn adding_impl_member_does_not_invalidate_another_target_or_its_caller() {
    let module = test_module();
    let caller = impl_callable(
        "Caller",
        Selector::method("run", [phalcom_common::selector::SelectorSlot::Label("value".into())]).unwrap(),
    );
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
    assert!(Arc::ptr_eq(
        &caller_v1,
        second.snapshot.callable_analyses.get(&caller).expect("unchanged caller body")
    ));
    assert_ne!(surface_fingerprint(&first, &owner("Beta")), surface_fingerprint(&second, &owner("Beta")));
}

#[test]
fn impl_whitespace_and_range_edit_preserves_semantic_surface_fingerprint() {
    let module = test_module();
    let callable = impl_callable("User", Selector::method("greet", []).unwrap());
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(
        module.clone(),
        "class User {}\nimpl User { greet() -> String { \"hi\" } }\n",
    ));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    let first_surface = surface_fingerprint(&first, &owner("User"));
    let first_signature =
        phalcom_semantic::db::fingerprint::callable_signature_product_fingerprint(first.snapshot.callable_signatures.get(&callable).expect("impl signature"));

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
    let first = session.update(single_module_input(
        module.clone(),
        "class Alpha {}\nclass Beta {}\nimpl Alpha { ping() -> Int { 1 } }\n",
    ));
    assert!(!first.snapshot.has_errors(), "diagnostics: {:?}", first.snapshot.diagnostics);
    assert!(first.snapshot.callable_definitions.contains_key(&alpha_ping));

    let second = session.update(single_module_input(
        module,
        "class Alpha {}\nclass Beta {}\nimpl Beta { ping() -> Int { 1 } }\n",
    ));
    assert!(
        !second.snapshot.callable_definitions.contains_key(&alpha_ping),
        "old target must lose the moved definition"
    );
    assert!(
        second.snapshot.callable_definitions.contains_key(&beta_ping),
        "new target must publish the moved definition"
    );
    assert!(
        !second
            .snapshot
            .surfaces()
            .get(&owner("Alpha"))
            .expect("Alpha surface")
            .instance
            .callable_signatures
            .keys()
            .any(|selector| selector.encode().starts_with("ping"))
    );
    assert!(
        second
            .snapshot
            .surfaces()
            .get(&owner("Beta"))
            .expect("Beta surface")
            .instance
            .callable_signatures
            .keys()
            .any(|selector| selector.encode().starts_with("ping"))
    );
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
    let a_variant = phalcom_semantic::identity::VariantId::new(owner("Choice"), Selector::getter("A").unwrap());
    let a_target = phalcom_semantic::impls::InherentImplTarget::ExactEnumCase(a_variant);
    assert_eq!(first.snapshot.dispatch.get_conditional_members(&a_target).map(|set| set.members.len()), Some(1));

    // Add another case-only member to B
    let source_v2 = "enum Choice {\n  A\n  B\n}\nimpl Choice::A {\n  aOnly() -> Int { 42 }\n}\nimpl Choice::B {\n  bOnly() -> Int { 99 }\n}\n";
    let second = session.update(single_module_input(module.clone(), source_v2));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    let root_surface_v2 = surface_fingerprint(&second, &owner("Choice"));
    let b_variant = phalcom_semantic::identity::VariantId::new(owner("Choice"), Selector::getter("B").unwrap());
    let b_target = phalcom_semantic::impls::InherentImplTarget::ExactEnumCase(b_variant);
    assert_eq!(
        second.snapshot.dispatch.get_conditional_members(&a_target).map(|set| set.members.len()),
        Some(1)
    );
    assert_eq!(
        second.snapshot.dispatch.get_conditional_members(&b_target).map(|set| set.members.len()),
        Some(1)
    );

    assert_eq!(
        root_surface_v1, root_surface_v2,
        "case-only members must not alter root enum declaration surface"
    );

    let third = session.update(single_module_input(module, "enum Choice {\n  A\n  B\n}\n"));
    assert!(
        !third.snapshot.has_errors(),
        "removal must not leave diagnostics: {:?}",
        third.snapshot.diagnostics
    );
    assert!(third.snapshot.dispatch.get_conditional_members(&a_target).is_none());
    assert!(third.snapshot.dispatch.get_conditional_members(&b_target).is_none());
}

#[test]
fn exact_case_conditional_precedence_shadows_root_requirement() {
    let module = test_module();
    let source = "enum Status { Ready Done }\nimpl Status { code() -> Int }\nimpl Status::Ready { code() -> Int { 1 } }\nimpl Status::Done { code() -> Int { 2 } }\nclass Probe { run() -> Int { Status::Ready.code() } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let output = session.update(single_module_input(module.clone(), source));
    assert!(!output.snapshot.has_errors(), "diagnostics: {:?}", output.snapshot.diagnostics);

    let selection = output
        .snapshot
        .callable_analyses
        .values()
        .flat_map(|analysis| analysis.expressions.values())
        .find_map(|expression| expression.conditional_dispatch.as_ref())
        .expect("exact-case call must publish conditional dispatch selection");
    assert!(matches!(
        &selection.callable.owner,
        CallableOwnerId::Variant(variant) if variant.owner == DeclarationId::new(module, "Status".into())
    ));
    assert_eq!(selection.callable.selector, Selector::method("code", []).unwrap());
}
