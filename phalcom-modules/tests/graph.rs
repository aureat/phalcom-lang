use phalcom_common::range::SourceRange;
use phalcom_modules::{
    DependencyPhase, ModuleComponent, ModuleId, ModulePath, ReferenceEdge, ReferenceGraph, ReferenceKind, RuntimeDependencyEdge, RuntimeDependencyGraph,
    RuntimeDependencyReason, SemanticEdge, SemanticEdgeKind, SemanticGraph, SemanticNodeId,
};

fn module(name: &str) -> ModuleId {
    ModuleId {
        project: phalcom_modules::ResolvedProjectId::from_raw(1).into(),
        path: ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]),
    }
}

fn runtime(importer: &ModuleId, dependency: &ModuleId) -> RuntimeDependencyEdge {
    RuntimeDependencyEdge {
        importer: importer.clone(),
        dependency: dependency.clone(),
        range: SourceRange::default(),
        reason: RuntimeDependencyReason::SelectiveValueImport,
        provenance: None,
    }
}

#[test]
fn empty_and_diamond_runtime_graphs_have_deterministic_order() {
    let empty = RuntimeDependencyGraph::default();
    assert_eq!(empty.initialization_order().unwrap(), Vec::<ModuleId>::new());

    let a = module("a");
    let b = module("b");
    let c = module("c");
    let d = module("d");
    let mut graph = RuntimeDependencyGraph::default();
    graph.add(runtime(&a, &b));
    graph.add(runtime(&a, &c));
    graph.add(runtime(&b, &d));
    graph.add(runtime(&c, &d));
    let order = graph.initialization_order().unwrap();
    assert!(order.iter().position(|id| id == &d).unwrap() < order.iter().position(|id| id == &b).unwrap());
    assert!(order.iter().position(|id| id == &d).unwrap() < order.iter().position(|id| id == &c).unwrap());
    assert_eq!(order.last(), Some(&a));
}

#[test]
fn runtime_self_and_multi_node_cycles_are_rejected() {
    let a = module("a");
    let b = module("b");
    let mut self_cycle = RuntimeDependencyGraph::default();
    self_cycle.add(runtime(&a, &a));
    assert!(self_cycle.validate_acyclic().is_err());

    let mut two_cycle = RuntimeDependencyGraph::default();
    two_cycle.add(runtime(&a, &b));
    two_cycle.add(runtime(&b, &a));
    assert!(two_cycle.initialization_order().is_err());
}

#[test]
fn runtime_cycle_reports_sorted_components_and_a_deterministic_edge_path() {
    let a = module("a");
    let b = module("b");
    let c = module("c");
    let d = module("d");
    let mut graph = RuntimeDependencyGraph::default();
    graph.add(runtime(&a, &c));
    graph.add(runtime(&a, &b));
    graph.add(runtime(&b, &a));
    graph.add(runtime(&c, &a));
    graph.add(runtime(&d, &d));

    let error = graph.initialization_order().expect_err("runtime cycles remain strict");
    let phalcom_modules::ModuleGraphError::RuntimeCycle { components, path, .. } = error;
    assert_eq!(components, vec![vec![a.clone(), b.clone(), c.clone()], vec![d.clone()]]);
    assert_eq!(path.len(), 2);
    assert_eq!((path[0].importer.clone(), path[0].dependency.clone()), (a.clone(), b.clone()));
    assert_eq!((path[1].importer.clone(), path[1].dependency.clone()), (b, a));
}

#[test]
fn semantic_scc_is_retained_without_runtime_cycle_policy() {
    let a = module("a");
    let b = module("b");
    let mut graph = SemanticGraph::default();
    graph.add(SemanticEdge {
        from: SemanticNodeId::Module(a.clone()),
        to: SemanticNodeId::Module(b.clone()),
        kind: SemanticEdgeKind::Superclass,
        range: SourceRange::default(),
    });
    graph.add(SemanticEdge {
        from: SemanticNodeId::Module(b.clone()),
        to: SemanticNodeId::Module(a.clone()),
        kind: SemanticEdgeKind::Superclass,
        range: SourceRange::default(),
    });
    assert_eq!(graph.components().len(), 1);
    assert_eq!(graph.components()[0].len(), 2);
}

#[test]
fn reference_graph_preserves_kind_and_phase_separately() {
    let a = module("a");
    let b = module("b");
    let mut references = ReferenceGraph::default();
    references.add(ReferenceEdge {
        from: a.clone(),
        to: b.clone(),
        kind: ReferenceKind::InterfaceOnly,
        range: SourceRange::default(),
    });
    assert_eq!(references.edges_from(&a)[0].kind, ReferenceKind::InterfaceOnly);
    assert_eq!(DependencyPhase::InterfaceOnly.join(DependencyPhase::Runtime), DependencyPhase::Runtime);
}
