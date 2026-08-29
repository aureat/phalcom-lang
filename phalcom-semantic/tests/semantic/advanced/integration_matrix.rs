use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_native_meta::primitive::TerminationSpec;
use phalcom_semantic::checker::analysis::{AnalysisStatus, CallableAnalysis, CallableAnalysisStatus, ExpressionAnalysis};
use phalcom_semantic::checker::flow::graph::{FlowGraph, FlowNodeKind};
use phalcom_semantic::control_summary::ControlFacts;
use phalcom_semantic::dispatch::DispatchSide;
use phalcom_semantic::effects::atom::EffectSet;
use phalcom_semantic::effects::infer::infer_intraprocedural_effects;
use phalcom_semantic::effects::scc::infer_interprocedural_effects_scc;
use phalcom_semantic::effects::summary::{EffectKnowledge, EffectOpaqueReason};
use phalcom_semantic::identity::{BodyId, CallableId, DeclarationId, ExpressionId, LocalExpressionId};
use phalcom_semantic::prover::deterministic::solve_vc_deterministic;
use phalcom_semantic::prover::ir::{ProofOpaqueReason, ProofTerm};
use phalcom_semantic::prover::vc::{ProofEvidence, ProofObligationKind, VcStatus, VcUnknownReason, VerificationCondition};
use phalcom_semantic::termination::{TerminationBlockedReason, TerminationEvidence, TerminationKnowledge, analyze_callable_termination};
use phalcom_semantic::types::evidence::{DynamicReason, EvidenceOrigin, TypeKnowledge};
use phalcom_semantic::types::relation::{MapTypeHierarchy, is_subtype};
use phalcom_semantic::types::row::{RecordRowData, RecordRowField, RecordRowTail};
use phalcom_semantic::types::row_solver::{RecordRowSolver, RecordRowTerm};
use phalcom_semantic::types::store::TypeStore;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

const RANGE: SourceRange = SourceRange { start: 0, end: 0 };

fn test_callable_id(name: &str) -> CallableId {
    let module = ModuleId::core();
    let decl = DeclarationId::new(module, "Test".into());
    CallableId::new(decl, Selector::getter(name).unwrap(), DispatchSide::Instance)
}

fn test_expr_id(local: u32) -> ExpressionId {
    ExpressionId::new(BodyId(1), LocalExpressionId(local))
}

fn test_decl(name: &str) -> DeclarationId {
    let module = ModuleId::core();
    DeclarationId::new(module, name.into())
}

#[test]
fn test_matrix_1_record_row_subtyping_and_unification() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let int_ty = store.nominal(test_decl("Int"));
    let str_ty = store.nominal(test_decl("String"));

    // sub: #{ a: Int, b: String }
    let sub = store.record(Box::new([
        RecordRowField { name: "a".into(), ty: int_ty },
        RecordRowField { name: "b".into(), ty: str_ty },
    ]));

    // sup: #{ a: Int }
    let sup = store.record(Box::new([RecordRowField { name: "a".into(), ty: int_ty }]));

    // Structural width subtyping: { a: Int, b: String } <: { a: Int }
    assert!(is_subtype(&store, &hier, sub, sup), "Record width subtyping must succeed for read-only fields");
    assert!(!is_subtype(&store, &hier, sup, sub), "Narrow cannot subtype wide");

    // Row solver unification
    let empty_row = store.intern_record_row(RecordRowData {
        fields: Box::new([]),
        tail: RecordRowTail::Closed,
    });
    let single_row = store.intern_record_row(RecordRowData {
        fields: Box::new([RecordRowField { name: "a".into(), ty: int_ty }]),
        tail: RecordRowTail::Closed,
    });

    let mut solver = RecordRowSolver::new(100);
    let r_var = solver.fresh_var();
    let left = RecordRowTerm::Canonical(single_row);
    let right = RecordRowTerm::Extend {
        fields: vec![RecordRowField { name: "a".into(), ty: int_ty }],
        tail: Box::new(RecordRowTerm::Var(r_var)),
    };

    let result = solver.solve(&left, &right, &store);
    assert!(matches!(result, phalcom_semantic::types::row_solver::RecordRowSolveResult::Solved(sol) if {
        sol.substitutions.get(&r_var) == Some(&RecordRowTerm::Canonical(empty_row))
    }));
}

#[test]
fn test_matrix_2_effects_pipeline_intra_and_interprocedural() {
    let store = TypeStore::new();
    let leaf_id = test_callable_id("leaf_fn");
    let caller_id = test_callable_id("caller_fn");

    let eid = test_expr_id(1);
    let mut leaf_exprs = BTreeMap::new();
    leaf_exprs.insert(
        eid,
        ExpressionAnalysis::ready(eid, RANGE, TypeKnowledge::established(store.unit(), EvidenceOrigin::Syntax)),
    );

    let leaf_analysis = CallableAnalysis {
        callable: leaf_id.clone(),
        body_range: RANGE,
        expressions: leaf_exprs,
        bindings: BTreeMap::new(),
        flow_graph: Arc::new(phalcom_semantic::checker::flow::graph::FlowGraph::default()),
        entry_flow: phalcom_semantic::checker::FlowStateSummary::default(),
        exits: phalcom_semantic::checker::BodyExitFacts::default(),
        return_validation: phalcom_semantic::ReturnContractValidation::NotApplicable,
        diagnostics: Arc::new([]),
        internal_incidents: Arc::new([]),
        explanations: Arc::new(phalcom_semantic::explain::ExplanationArena::default()),
        return_explanation: None,
        dependencies: Arc::from(vec![]),
        semantic_dependencies: Arc::from([]),
        dependency_fingerprint: phalcom_semantic::db::ProductFingerprint::new(0),
        status: CallableAnalysisStatus::Complete,
    };

    let caller_analysis = CallableAnalysis {
        callable: caller_id.clone(),
        body_range: RANGE,
        expressions: BTreeMap::new(),
        bindings: BTreeMap::new(),
        flow_graph: Arc::new(phalcom_semantic::checker::flow::graph::FlowGraph::default()),
        entry_flow: phalcom_semantic::checker::FlowStateSummary::default(),
        exits: phalcom_semantic::checker::BodyExitFacts::default(),
        return_validation: phalcom_semantic::ReturnContractValidation::NotApplicable,
        diagnostics: Arc::new([]),
        internal_incidents: Arc::new([]),
        explanations: Arc::new(phalcom_semantic::explain::ExplanationArena::default()),
        return_explanation: None,
        dependencies: Arc::from(vec![leaf_id.clone()]),
        semantic_dependencies: Arc::from([]),
        dependency_fingerprint: phalcom_semantic::db::ProductFingerprint::new(0),
        status: CallableAnalysisStatus::Complete,
    };

    // Intraprocedural
    let leaf_intra = infer_intraprocedural_effects(&leaf_analysis);
    assert_eq!(leaf_intra, EffectKnowledge::Known(EffectSet::EMPTY));

    // Interprocedural SCC
    let mut map = HashMap::new();
    map.insert(leaf_id.clone(), leaf_analysis);
    map.insert(caller_id.clone(), caller_analysis);

    let inter = infer_interprocedural_effects_scc(&map);
    assert_eq!(inter.get(&caller_id), Some(&EffectKnowledge::Known(EffectSet::EMPTY)));
}

#[test]
fn test_matrix_3_termination_and_control_facts() {
    let cid = test_callable_id("terminating_fn");
    let mut graph = FlowGraph::new();
    let entry = graph.add_node(FlowNodeKind::Entry, RANGE);
    let exit = graph.add_node(FlowNodeKind::Exit, RANGE);
    graph.entry = Some(entry);
    graph.exits.push(exit);
    graph.add_edge(entry, exit, None);

    let facts = ControlFacts::from_flow_graph(&graph);
    assert!(facts.may_return_normally);
    assert!(facts.cycle_candidates.is_empty());

    let analysis = CallableAnalysis {
        callable: cid.clone(),
        body_range: RANGE,
        expressions: BTreeMap::new(),
        bindings: BTreeMap::new(),
        flow_graph: Arc::new(phalcom_semantic::checker::flow::graph::FlowGraph::default()),
        entry_flow: phalcom_semantic::checker::FlowStateSummary::default(),
        exits: phalcom_semantic::checker::BodyExitFacts::default(),
        return_validation: phalcom_semantic::ReturnContractValidation::NotApplicable,
        diagnostics: Arc::new([]),
        internal_incidents: Arc::new([]),
        explanations: Arc::new(phalcom_semantic::explain::ExplanationArena::default()),
        return_explanation: None,
        dependencies: Arc::from(vec![]),
        semantic_dependencies: Arc::from([]),
        dependency_fingerprint: phalcom_semantic::db::ProductFingerprint::new(0),
        status: CallableAnalysisStatus::Complete,
    };

    let term = analyze_callable_termination(Some(&graph), &analysis, None);
    assert_eq!(term, TerminationKnowledge::Proven(TerminationEvidence::AcyclicCfg));
}

#[test]
fn test_matrix_4_deterministic_verification_conditions() {
    // Proven VC
    let mut vc_proven = VerificationCondition {
        id: 1,
        obligation: ProofObligationKind::PreconditionHold,
        antecedent: ProofTerm::IntConst(10),
        consequent: ProofTerm::IntConst(10),
        status: VcStatus::Unknown(VcUnknownReason::IncompleteSolver),
    };
    solve_vc_deterministic(&mut vc_proven);
    assert_eq!(vc_proven.status, VcStatus::Proven(ProofEvidence::DirectSimplification));

    // Disproven VC
    let mut vc_disproven = VerificationCondition {
        id: 2,
        obligation: ProofObligationKind::PostconditionHold,
        antecedent: ProofTerm::TRUE,
        consequent: ProofTerm::FALSE,
        status: VcStatus::Unknown(VcUnknownReason::IncompleteSolver),
    };
    solve_vc_deterministic(&mut vc_disproven);
    assert!(matches!(vc_disproven.status, VcStatus::Disproven(_)));

    // Unknown VC due to opaque term
    let mut vc_unknown = VerificationCondition {
        id: 3,
        obligation: ProofObligationKind::InvariantHold,
        antecedent: ProofTerm::TRUE,
        consequent: ProofTerm::Opaque(ProofOpaqueReason::DynamicValue),
        status: VcStatus::Unknown(VcUnknownReason::IncompleteSolver),
    };
    solve_vc_deterministic(&mut vc_unknown);
    assert_eq!(vc_unknown.status, VcStatus::Unknown(VcUnknownReason::ContainsOpaqueTerm));
}

#[test]
fn test_matrix_5_invariants_hold_across_boundaries() {
    // Dynamic boundary must yield Opaque effect, never Known(EMPTY)
    let cid = test_callable_id("dynamic_fn");
    let eid = test_expr_id(1);
    let mut exprs = BTreeMap::new();
    exprs.insert(
        eid,
        ExpressionAnalysis {
            id: eid,
            range: RANGE,
            knowledge: TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape),
            callable: None,
            denotation: None,
            status: AnalysisStatus::DynamicBoundary(DynamicReason::ExplicitEscape),
            causal_invalidity: phalcom_semantic::checker::CausalInvalidity::Clean,
            explanation: None,
            call: None,
        },
    );
    let analysis = CallableAnalysis {
        callable: cid.clone(),
        body_range: RANGE,
        expressions: exprs,
        bindings: BTreeMap::new(),
        flow_graph: Arc::new(phalcom_semantic::checker::flow::graph::FlowGraph::default()),
        entry_flow: phalcom_semantic::checker::FlowStateSummary::default(),
        exits: phalcom_semantic::checker::BodyExitFacts::default(),
        return_validation: phalcom_semantic::ReturnContractValidation::NotApplicable,
        diagnostics: Arc::new([]),
        internal_incidents: Arc::new([]),
        explanations: Arc::new(phalcom_semantic::explain::ExplanationArena::default()),
        return_explanation: None,
        dependencies: Arc::from(vec![]),
        semantic_dependencies: Arc::from([]),
        dependency_fingerprint: phalcom_semantic::db::ProductFingerprint::new(0),
        status: CallableAnalysisStatus::Complete,
    };
    let eff = infer_intraprocedural_effects(&analysis);
    assert_eq!(eff, EffectKnowledge::Opaque(EffectOpaqueReason::ForeignBoundary));
    assert_ne!(eff, EffectKnowledge::Known(EffectSet::EMPTY));

    // Unknown native termination must yield Blocked, never Proven
    let term = analyze_callable_termination(None, &analysis, Some(TerminationSpec::Unknown));
    assert_eq!(term, TerminationKnowledge::Blocked(TerminationBlockedReason::OpaqueNative));
    assert!(!term.is_proven());
}
