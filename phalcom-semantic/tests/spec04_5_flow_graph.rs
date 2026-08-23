use phalcom_common::range::SourceRange;
use phalcom_modules::DeclarationId;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::flow::graph::{FlowGraph, FlowNodeKind};
use phalcom_semantic::checker::flow::predicate::FlowPredicate;
use phalcom_semantic::checker::flow::state::FlowState;
use phalcom_semantic::checker::flow::transfer::apply_predicate;
use phalcom_semantic::identity::BindingId;
use phalcom_semantic::types::evidence::{EvidenceAuthority, TypeKnowledge};
use phalcom_semantic::types::store::TypeStore;

#[test]
fn test_flow_graph_construction() {
    let mut graph = FlowGraph::new();
    let r0 = SourceRange { start: 0, end: 10 };
    let n1 = graph.add_node(FlowNodeKind::Entry, r0);
    let n2 = graph.add_node(FlowNodeKind::BranchCondition, r0);
    let n3 = graph.add_node(FlowNodeKind::Exit, r0);

    let e1 = graph.add_edge(n1, n2, None);
    let e2 = graph.add_edge(n2, n3, None);

    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 2);
    assert_eq!(graph.nodes.get(&n1).unwrap().successors, vec![e1]);
    assert_eq!(graph.nodes.get(&n2).unwrap().predecessors, vec![e1]);
    assert_eq!(graph.nodes.get(&n2).unwrap().successors, vec![e2]);
}

#[test]
fn test_predicate_refinement_on_union_type() {
    let mut store = TypeStore::new();
    let module = ModuleId::core();
    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let str_decl = DeclarationId::new(module.clone(), "String".into());

    let int_ty = store.nominal(int_decl);
    let str_ty = store.nominal(str_decl);
    let union_ty = store.union(&[int_ty, str_ty]);

    let b1 = BindingId(1);
    let mut state = FlowState::new();
    state.declare(b1, None, TypeKnowledge::known(union_ty, EvidenceAuthority::ExactSyntax), true);

    // Filter by IsInstance { target: Int }
    let pred = FlowPredicate::IsInstance { binding: b1, target: int_ty };
    apply_predicate(&mut state, &pred, &mut store);

    assert_eq!(state.get_current_type(b1).and_then(|k| k.ty()), Some(int_ty));
}
