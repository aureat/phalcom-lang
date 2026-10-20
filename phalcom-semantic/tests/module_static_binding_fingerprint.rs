use phalcom_ast::ast::{Pattern, Statement};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModulePath, ResolvedProjectId};
use phalcom_modules::linker::SymbolId;
use phalcom_semantic::checker::analysis::{BindingAnalysisIndex, BindingState, BodyExitFacts, CallableAnalysis, CallableAnalysisStatus, FlowStateSummary};
use phalcom_semantic::checker::flow::graph::FlowGraph;
use phalcom_semantic::db::fingerprint::module_static_binding_input_fingerprint;
use phalcom_semantic::db::query::query_module_static_bindings;
use phalcom_semantic::db::{ProductFingerprint, QueryOutcome, SemanticDb};
use phalcom_semantic::dispatch::DispatchSide;
use phalcom_semantic::explain::ExplanationArena;
use phalcom_semantic::identity::{BindingId, CallableId, DeclarationId, ExplanationId, ModuleId};
use phalcom_semantic::module_bindings::ModuleStaticBindingIndex;
use phalcom_semantic::signature::ReturnContractValidation;
use phalcom_semantic::types::evidence::{TypeKnowledge, UnknownReason};
use std::collections::BTreeMap;
use std::sync::Arc;

fn module() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("bindings").expect("valid module component")]),
    )
}

fn module_bindings_with_explanation(explanation: Option<ExplanationId>) -> Arc<ModuleStaticBindingIndex> {
    let source = "let value = 1";
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "module binding fixture must parse: {:?}", parsed.errors);
    let binding_range = parsed
        .program
        .statements
        .iter()
        .find_map(|statement| match statement {
            Statement::Let(binding) => match &binding.pattern {
                Pattern::Name { range, .. } => Some(*range),
                _ => None,
            },
            _ => None,
        })
        .expect("top-level name binding");
    let module = module();
    let callable = CallableId::new(
        DeclarationId::new(module.clone(), "<main>".into()),
        Selector::getter("<main>").expect("module initializer selector"),
        DispatchSide::Instance,
    );
    let binding = BindingId(0);
    let mut state = BindingState::new(
        binding,
        "value",
        binding_range,
        None,
        TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
        false,
    );
    state.explanation = explanation;
    let mut bindings = BindingAnalysisIndex::default();
    bindings.insert_source(binding, state).expect("unique source binding");
    let analysis = CallableAnalysis {
        callable,
        body_range: SourceRange { start: 0, end: source.len() },
        body_source_owner: None,
        expressions: Default::default(),
        bindings,
        binding_origins: Default::default(),
        for_iteration_lanes: Arc::new(Default::default()),
        positional_expansions: Arc::new(Default::default()),
        known_invalid_positional_expansions: Arc::new(Default::default()),
        sequence_literals: Arc::new(Default::default()),
        associated_resolutions: Arc::new(Default::default()),
        callable_reference_resolutions: Arc::new(Default::default()),
        family_applications: Arc::new(Default::default()),
        match_resolutions: Arc::new(Default::default()),
        pattern_site_resolutions: Arc::new(Default::default()),
        local_proof_graph: Arc::new(Default::default()),
        flow_graph: Arc::new(FlowGraph::default()),
        entry_flow: FlowStateSummary::default(),
        exits: BodyExitFacts::default(),
        return_validation: ReturnContractValidation::NotApplicable,
        diagnostics: Arc::from([]),
        internal_incidents: Arc::from([]),
        explanations: Arc::new(ExplanationArena::default()),
        return_explanation: None,
        dependencies: Arc::from([]),
        semantic_dependencies: Arc::from([]),
        constructor_super_initializer: None,
        dependency_fingerprint: ProductFingerprint::new(0),
        status: CallableAnalysisStatus::Complete,
        observed_import_phases: Arc::new(phalcom_semantic::checker::analysis_map::AnalysisMap::new()),
    };

    Arc::new(ModuleStaticBindingIndex::from_module_analysis(&module, &parsed.program, &analysis))
}

#[test]
fn module_static_binding_fingerprint_and_query_refresh_include_explanation() {
    let first = module_bindings_with_explanation(None);
    let second = module_bindings_with_explanation(Some(ExplanationId(3)));

    assert_ne!(
        module_static_binding_input_fingerprint(&first),
        module_static_binding_input_fingerprint(&second),
        "owner-local explanation changes are module-binding query inputs"
    );

    let mut db = SemanticDb::new();
    let first = match query_module_static_bindings(&mut db, first) {
        QueryOutcome::Ready(product) => product,
        outcome => panic!("first module binding query was not ready: {outcome:?}"),
    };
    let second = match query_module_static_bindings(&mut db, second) {
        QueryOutcome::Ready(product) => product,
        outcome => panic!("second module binding query was not ready: {outcome:?}"),
    };
    let symbol = SymbolId {
        module: module(),
        name: "value".into(),
    };

    assert_eq!(first.get(&symbol).expect("first binding").state.explanation, None);
    assert_eq!(second.get(&symbol).expect("refreshed binding").state.explanation, Some(ExplanationId(3)));
    assert!(
        !Arc::ptr_eq(&first, &second),
        "changed explanation must replace the cached module binding product"
    );
}
