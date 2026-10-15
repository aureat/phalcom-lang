use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_native_meta::primitive::TerminationSpec;
use phalcom_semantic::checker::analysis::{AnalysisStatus, CallableAnalysis, CallableAnalysisStatus, ExpressionAnalysis};
use phalcom_semantic::checker::flow::graph::{FlowGraph, FlowNodeKind};
use phalcom_semantic::dispatch::DispatchSide;
use phalcom_semantic::effects::atom::EffectSet;
use phalcom_semantic::effects::scc::infer_interprocedural_effects_scc;
use phalcom_semantic::effects::summary::{EffectKnowledge, EffectOpaqueReason};
use phalcom_semantic::identity::{BodyId, CallableId, DeclarationId, ExpressionId, LocalExpressionId};
use phalcom_semantic::termination::{TerminationBlockedReason, TerminationEvidence, TerminationKnowledge, analyze_callable_termination, check_cfg_acyclicity};
use phalcom_semantic::types::evidence::{DynamicReason, EvidenceOrigin, TypeKnowledge};
use phalcom_semantic::types::store::TypeStore;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

const RANGE: SourceRange = SourceRange { start: 0, end: 0 };

fn make_callable_id(name: &str) -> CallableId {
    let module = ModuleId::universe_root();
    let decl = DeclarationId::new(module, "Test".into());
    CallableId::new(decl, Selector::getter(name).unwrap(), DispatchSide::Instance)
}

fn test_expr_id(local: u32) -> ExpressionId {
    ExpressionId::new(BodyId(1), LocalExpressionId(local))
}

fn mock_callable(id: CallableId, expressions: BTreeMap<ExpressionId, ExpressionAnalysis>, dependencies: Vec<CallableId>) -> CallableAnalysis {
    CallableAnalysis {
        callable: id,
        body_source_owner: None,
        body_range: RANGE,
        expressions,
        bindings: BTreeMap::new(),
        binding_origins: BTreeMap::new(),
        for_iteration_lanes: Arc::new(BTreeMap::new()),
        positional_expansions: Arc::new(BTreeMap::new()),
        known_invalid_positional_expansions: Arc::new(BTreeMap::new()),
        sequence_literals: Arc::new(BTreeMap::new()),
        associated_resolutions: Arc::new(BTreeMap::new()),
        callable_reference_resolutions: Arc::new(BTreeMap::new()),
        family_applications: Arc::new(BTreeMap::new()),
        match_resolutions: Arc::new(BTreeMap::new()),
        pattern_site_resolutions: Arc::new(BTreeMap::new()),
        flow_graph: Arc::new(phalcom_semantic::checker::flow::graph::FlowGraph::default()),
        entry_flow: phalcom_semantic::checker::FlowStateSummary::default(),
        exits: phalcom_semantic::checker::BodyExitFacts::default(),
        return_validation: phalcom_semantic::ReturnContractValidation::NotApplicable,
        diagnostics: Arc::new([]),
        internal_incidents: Arc::new([]),
        explanations: Arc::new(phalcom_semantic::explain::ExplanationArena::default()),
        return_explanation: None,
        dependencies: Arc::from(dependencies),
        semantic_dependencies: Arc::from([]),
        constructor_super_initializer: None,
        dependency_fingerprint: phalcom_semantic::db::ProductFingerprint::new(0),
        status: CallableAnalysisStatus::Complete,
    }
}

#[test]
fn test_interprocedural_effect_propagation() {
    let leaf_id = make_callable_id("leaf");
    let mid_id = make_callable_id("mid");
    let root_id = make_callable_id("root");

    // leaf has dynamic boundary -> Opaque(DynamicDispatch)
    let mut leaf_exprs = BTreeMap::new();
    let eid = test_expr_id(1);
    leaf_exprs.insert(
        eid,
        ExpressionAnalysis {
            id: eid,
            range: RANGE,
            knowledge: TypeKnowledge::Dynamic(DynamicReason::DynamicRestPack),
            callable: None,
            call_specialization: None,
            conditional_dispatch: None,
            conditional_capability: None,
            trait_dispatch: None,
            trait_dispatch_candidates: None,
            contextual_realization: None,
            denotation: None,
            status: AnalysisStatus::DynamicBoundary(DynamicReason::DynamicRestPack),
            causal_invalidity: phalcom_semantic::checker::CausalInvalidity::Clean,
            explanation: None,
            call: None,
            binding_read_safety: None,
            binding_read: None,
            binding_write: None,
        },
    );
    let leaf = mock_callable(leaf_id.clone(), leaf_exprs, vec![]);

    // mid calls leaf
    let mid = mock_callable(mid_id.clone(), BTreeMap::new(), vec![leaf_id.clone()]);

    // root calls mid
    let root = mock_callable(root_id.clone(), BTreeMap::new(), vec![mid_id.clone()]);

    let mut analyses = HashMap::new();
    analyses.insert(leaf_id.clone(), leaf);
    analyses.insert(mid_id.clone(), mid);
    analyses.insert(root_id.clone(), root);

    let effects = infer_interprocedural_effects_scc(&analyses);

    assert_eq!(effects.get(&leaf_id), Some(&EffectKnowledge::Opaque(EffectOpaqueReason::DynamicDispatch)));
    assert_eq!(effects.get(&mid_id), Some(&EffectKnowledge::Opaque(EffectOpaqueReason::DynamicDispatch)));
    assert_eq!(effects.get(&root_id), Some(&EffectKnowledge::Opaque(EffectOpaqueReason::DynamicDispatch)));
}

#[test]
fn test_interprocedural_pure_propagation() {
    let store = TypeStore::new();
    let leaf_id = make_callable_id("pure_leaf");
    let root_id = make_callable_id("pure_root");

    let mut leaf_exprs = BTreeMap::new();
    let eid = test_expr_id(1);
    leaf_exprs.insert(
        eid,
        ExpressionAnalysis::ready(eid, RANGE, TypeKnowledge::established(store.unit(), EvidenceOrigin::Syntax)),
    );
    let leaf = mock_callable(leaf_id.clone(), leaf_exprs, vec![]);
    let root = mock_callable(root_id.clone(), BTreeMap::new(), vec![leaf_id.clone()]);

    let mut analyses = HashMap::new();
    analyses.insert(leaf_id.clone(), leaf);
    analyses.insert(root_id.clone(), root);

    let effects = infer_interprocedural_effects_scc(&analyses);
    assert_eq!(effects.get(&leaf_id), Some(&EffectKnowledge::Known(EffectSet::EMPTY)));
    assert_eq!(effects.get(&root_id), Some(&EffectKnowledge::Known(EffectSet::EMPTY)));
}

#[test]
fn test_cfg_acyclicity_termination() {
    let mut acyclic_graph = FlowGraph::new();
    let entry = acyclic_graph.add_node(FlowNodeKind::Entry, RANGE).unwrap();
    let branch = acyclic_graph.add_node(FlowNodeKind::BranchCondition, RANGE).unwrap();
    let join = acyclic_graph.add_node(FlowNodeKind::Join, RANGE).unwrap();
    let exit = acyclic_graph.add_node(FlowNodeKind::Exit, RANGE).unwrap();

    acyclic_graph.try_set_entry(Some(entry)).unwrap();
    acyclic_graph.try_push_exit(exit).unwrap();
    acyclic_graph.add_edge(entry, branch, None).unwrap();
    acyclic_graph.add_edge(branch, join, None).unwrap();
    acyclic_graph.add_edge(join, exit, None).unwrap();

    assert_eq!(check_cfg_acyclicity(&acyclic_graph), Some(TerminationEvidence::AcyclicCfg));

    let mut cyclic_graph = FlowGraph::new();
    let loop_header = cyclic_graph.add_node(FlowNodeKind::LoopHeader, RANGE).unwrap();
    cyclic_graph.try_set_entry(Some(loop_header)).unwrap();
    cyclic_graph.try_push_exit(exit).unwrap();

    assert_eq!(check_cfg_acyclicity(&cyclic_graph), None);
}

#[test]
fn test_termination_analysis_callable() {
    let cid = make_callable_id("straight_line");
    let mut graph = FlowGraph::new();
    let entry = graph.add_node(FlowNodeKind::Entry, RANGE).unwrap();
    let exit = graph.add_node(FlowNodeKind::Exit, RANGE).unwrap();
    graph.try_set_entry(Some(entry)).unwrap();
    graph.try_push_exit(exit).unwrap();
    graph.add_edge(entry, exit, None).unwrap();

    let analysis = mock_callable(cid.clone(), BTreeMap::new(), vec![]);
    let term = analyze_callable_termination(Some(&graph), &analysis, None);
    assert_eq!(term, TerminationKnowledge::Proven(TerminationEvidence::AcyclicCfg));
    assert!(term.is_proven());

    // Recursive callable
    let rec_analysis = mock_callable(cid.clone(), BTreeMap::new(), vec![cid.clone()]);
    let rec_term = analyze_callable_termination(Some(&graph), &rec_analysis, None);
    assert_eq!(rec_term, TerminationKnowledge::Blocked(TerminationBlockedReason::UnsupportedRecursionPattern));
    assert!(!rec_term.is_proven());
}

#[test]
fn test_termination_native_specs() {
    let cid = make_callable_id("native_test");
    let analysis = mock_callable(cid.clone(), BTreeMap::new(), vec![]);

    // Terminates -> Proven(TrustedNative)
    let proven = analyze_callable_termination(None, &analysis, Some(TerminationSpec::Terminates));
    assert_eq!(proven, TerminationKnowledge::Proven(TerminationEvidence::TrustedNative));

    // Unknown -> Blocked(OpaqueNative) (NEVER proven)
    let unknown = analyze_callable_termination(None, &analysis, Some(TerminationSpec::Unknown));
    assert_eq!(unknown, TerminationKnowledge::Blocked(TerminationBlockedReason::OpaqueNative));
    assert!(!unknown.is_proven());

    // MayDiverge -> Blocked(OpaqueNative)
    let may_diverge = analyze_callable_termination(None, &analysis, Some(TerminationSpec::MayDiverge));
    assert_eq!(may_diverge, TerminationKnowledge::Blocked(TerminationBlockedReason::OpaqueNative));
    assert!(!may_diverge.is_proven());
}
