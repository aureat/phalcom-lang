use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_native_meta::primitive::{EffectSpec, NativeEffect};
use phalcom_semantic::checker::analysis::{AnalysisStatus, CallableAnalysis, CallableAnalysisStatus, ExpressionAnalysis};
use phalcom_semantic::checker::flow::graph::{FlowGraph, FlowNodeKind};
use phalcom_semantic::control_summary::ControlFacts;
use phalcom_semantic::dispatch::DispatchSide;
use phalcom_semantic::effects::atom::{EffectAtom, EffectSet};
use phalcom_semantic::effects::infer::{adapt_effect_spec, infer_intraprocedural_effects};
use phalcom_semantic::effects::summary::{EffectKnowledge, EffectOpaqueReason};
use phalcom_semantic::identity::{BodyId, CallableId, DeclarationId, ExpressionId, LocalExpressionId};
use phalcom_semantic::types::evidence::{DynamicReason, EvidenceOrigin, TypeKnowledge};
use phalcom_semantic::types::store::TypeStore;
use std::collections::BTreeMap;
use std::sync::Arc;

const RANGE: SourceRange = SourceRange { start: 0, end: 0 };

fn test_callable_id() -> CallableId {
    let module = ModuleId::core();
    let decl = DeclarationId::new(module, "Test".into());
    CallableId::new(decl, Selector::getter("foo").unwrap(), DispatchSide::Instance)
}

fn test_expr_id(local: u32) -> ExpressionId {
    ExpressionId::new(BodyId(1), LocalExpressionId(local))
}

fn mock_callable(expressions: BTreeMap<ExpressionId, ExpressionAnalysis>) -> CallableAnalysis {
    CallableAnalysis {
        callable: test_callable_id(),
        body_range: RANGE,
        expressions,
        bindings: BTreeMap::new(),
        flow_graph: Arc::new(phalcom_semantic::checker::flow::graph::FlowGraph::default()),
        entry_flow: phalcom_semantic::checker::FlowStateSummary::default(),
        exits: phalcom_semantic::checker::BodyExitFacts::default(),
        return_validation: phalcom_semantic::ReturnContractValidation::NotApplicable,
        diagnostics: Arc::new([]),
        internal_incidents: Arc::new([]),
        explanations: Arc::new(phalcom_semantic::explain::ExplanationArena::default()),
        return_explanation: None,
        dependencies: Arc::new([]),
        semantic_dependencies: Arc::new([]),
        dependency_fingerprint: phalcom_semantic::db::ProductFingerprint::new(0),
        status: CallableAnalysisStatus::Complete,
    }
}

#[test]
fn test_effect_atom_set_algebra() {
    let empty = EffectSet::EMPTY;
    assert!(empty.is_empty());
    assert_eq!(empty.join(empty), empty);

    let io = empty.insert(EffectAtom::Io);
    let mut_ = empty.insert(EffectAtom::Mutation);

    assert_eq!(io.join(mut_), mut_.join(io), "join commutativity");
    assert_eq!(io.join(io), io, "join idempotency");
    assert_eq!(empty.join(io), io, "empty join identity");

    let both = io.join(mut_);
    assert!(io.is_subset_of(both));
    assert!(mut_.is_subset_of(both));
    assert!(!both.is_subset_of(io));
}

#[test]
fn test_pure_literal_function_infers_empty_effects() {
    let store = TypeStore::new();
    let mut expressions = BTreeMap::new();
    let eid = test_expr_id(1);
    let expr = ExpressionAnalysis::ready(eid, RANGE, TypeKnowledge::established(store.unit(), EvidenceOrigin::Syntax));
    expressions.insert(eid, expr);

    let analysis = mock_callable(expressions);
    let effects = infer_intraprocedural_effects(&analysis);
    assert_eq!(effects, EffectKnowledge::Known(EffectSet::EMPTY));
    assert!(effects.is_known_pure());
}

#[test]
fn test_native_effect_adaptation() {
    let pure = adapt_effect_spec(EffectSpec::Pure);
    assert_eq!(pure, EffectKnowledge::Known(EffectSet::EMPTY));

    let io = adapt_effect_spec(EffectSpec::Known(&[NativeEffect::Io]));
    assert_eq!(io, EffectKnowledge::Known(EffectSet::EMPTY.insert(EffectAtom::Io)));

    let unknown = adapt_effect_spec(EffectSpec::Unknown);
    assert_eq!(unknown, EffectKnowledge::Opaque(EffectOpaqueReason::MissingNativeMetadata));

    // Central law: Opaque != Known(EMPTY)
    assert_ne!(unknown, EffectKnowledge::Known(EffectSet::EMPTY));
}

#[test]
fn test_dynamic_dispatch_produces_opaque_effects() {
    let mut expressions = BTreeMap::new();
    let eid = test_expr_id(1);
    let expr = ExpressionAnalysis {
        id: eid,
        range: RANGE,
        knowledge: TypeKnowledge::Dynamic(DynamicReason::DynamicRestPack),
        callable: None,
        denotation: None,
        status: AnalysisStatus::DynamicBoundary(DynamicReason::DynamicRestPack),
        causal_invalidity: phalcom_semantic::checker::CausalInvalidity::Clean,
        explanation: None,
        call: None,
    };
    expressions.insert(eid, expr);

    let analysis = mock_callable(expressions);
    let effects = infer_intraprocedural_effects(&analysis);
    assert_eq!(effects, EffectKnowledge::Opaque(EffectOpaqueReason::DynamicDispatch));
    assert_ne!(effects, EffectKnowledge::Known(EffectSet::EMPTY));
}

#[test]
fn test_reflective_perform_produces_opaque_effects() {
    let mut expressions = BTreeMap::new();
    let eid = test_expr_id(1);
    let expr = ExpressionAnalysis {
        id: eid,
        range: RANGE,
        knowledge: TypeKnowledge::Dynamic(DynamicReason::RuntimeReflection),
        callable: None,
        denotation: None,
        status: AnalysisStatus::DynamicBoundary(DynamicReason::RuntimeReflection),
        causal_invalidity: phalcom_semantic::checker::CausalInvalidity::Clean,
        explanation: None,
        call: None,
    };
    expressions.insert(eid, expr);

    let analysis = mock_callable(expressions);
    let effects = infer_intraprocedural_effects(&analysis);
    assert_eq!(effects, EffectKnowledge::Opaque(EffectOpaqueReason::ReflectivePerform));
    assert_ne!(effects, EffectKnowledge::Known(EffectSet::EMPTY));
}

#[test]
fn test_control_facts_from_flow_graph() {
    let mut graph = FlowGraph::new();
    let entry = graph.add_node(FlowNodeKind::Entry, RANGE);
    let loop_header = graph.add_node(FlowNodeKind::LoopHeader, RANGE);
    let exit = graph.add_node(FlowNodeKind::Exit, RANGE);

    graph.entry = Some(entry);
    graph.exits.push(exit);
    graph.add_edge(entry, loop_header, None);
    graph.add_edge(loop_header, exit, None);

    let facts = ControlFacts::from_flow_graph(&graph);
    assert!(facts.may_return_normally);
    assert_eq!(facts.cycle_candidates, vec![loop_header]);
    assert!(!facts.may_exit_process);
    assert!(!facts.may_suspend);
}
