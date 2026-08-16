//! Dependency graphs used by module linking and semantic analysis.
//!
//! The graphs deliberately keep reference, semantic, and runtime edges in
//! separate structures. A declaration/interface cycle is valid input to a
//! semantic fixed point; a runtime initialization cycle is not.

use crate::identity::ModuleId;
use phalcom_common::range::SourceRange;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hash;

/// Phase required by a resolved dependency.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DependencyPhase {
    /// The dependency is needed only to resolve/check declarations.
    InterfaceOnly,
    /// The dependency contributes an eagerly initialized runtime binding.
    Runtime,
}

impl DependencyPhase {
    /// Returns the stronger phase required by two observations.
    pub fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::Runtime, _) | (_, Self::Runtime) => Self::Runtime,
            _ => Self::InterfaceOnly,
        }
    }
}

/// Kind of source reference retained by the linker.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReferenceKind {
    /// Imports a module object into the local namespace.
    WholeModuleImport,
    /// Imports one or more exported names.
    SelectiveImport,
    /// Re-exports names from another module.
    ReExport,
    /// A declaration-only/type-level reference.
    InterfaceOnly,
}

/// One statically resolved source reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceEdge {
    /// Module containing the reference.
    pub from: ModuleId,
    /// Resolved target module.
    pub to: ModuleId,
    /// Source-level reference kind.
    pub kind: ReferenceKind,
    /// Source range of the reference.
    pub range: SourceRange,
}

/// Identity of a node in the semantic/interface graph.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticNodeId {
    /// Module-level semantic node.
    Module(ModuleId),
    /// Declaration-level node reserved for type/protocol/ADT linking.
    Declaration { module: ModuleId, name: Box<str> },
}

/// Semantic relationship between declaration/interface nodes.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SemanticEdgeKind {
    /// Module interface dependency.
    ModuleInterface,
    /// Type reference.
    TypeReference,
    /// Superclass relationship.
    Superclass,
    /// Protocol relationship.
    ProtocolReference,
    /// Generic/constraint relationship.
    ConstraintReference,
    /// Callback signature relationship.
    CallbackSignature,
    /// Recursive ADT relationship.
    AdtReference,
}

/// One semantic/interface dependency edge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticEdge {
    /// Source semantic node.
    pub from: SemanticNodeId,
    /// Target semantic node.
    pub to: SemanticNodeId,
    /// Relationship kind.
    pub kind: SemanticEdgeKind,
    /// Source range of the relationship.
    pub range: SourceRange,
}

/// Why a runtime module dependency exists.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeDependencyReason {
    /// A whole-module import creates a module value.
    WholeModuleImport,
    /// A selected value is read at runtime.
    SelectiveValueImport,
    /// A selected value is exposed publicly by re-export.
    ReExport,
    /// Runtime declaration materialization needs the target declaration.
    RuntimeDeclarationReference,
}

/// One eager runtime initialization dependency.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeDependencyEdge {
    /// Importing module. The dependency must initialize before this module.
    pub importer: ModuleId,
    /// Required module.
    pub dependency: ModuleId,
    /// Source range causing the dependency.
    pub range: SourceRange,
    /// Dependency reason.
    pub reason: RuntimeDependencyReason,
}

/// Graph of statically resolved source references.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReferenceGraph {
    forward: BTreeMap<ModuleId, Vec<ReferenceEdge>>,
}

impl ReferenceGraph {
    /// Adds one reference edge.
    pub fn add(&mut self, edge: ReferenceEdge) {
        self.forward.entry(edge.from.clone()).or_default().push(edge);
    }

    /// Replaces all references contributed by one module.
    pub fn replace(&mut self, module: ModuleId, edges: Vec<ReferenceEdge>) {
        self.forward.insert(module, edges);
    }

    /// Returns all edges from `module`.
    pub fn edges_from(&self, module: &ModuleId) -> &[ReferenceEdge] {
        self.forward.get(module).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Returns all modules present in this graph.
    pub fn nodes(&self) -> Vec<ModuleId> {
        let mut nodes = BTreeSet::new();
        for (from, edges) in &self.forward {
            nodes.insert(from.clone());
            nodes.extend(edges.iter().map(|edge| edge.to.clone()));
        }
        nodes.into_iter().collect()
    }
}

/// Graph of semantic/interface dependencies.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SemanticGraph {
    forward: BTreeMap<SemanticNodeId, Vec<SemanticEdge>>,
}

impl SemanticGraph {
    /// Adds one semantic edge.
    pub fn add(&mut self, edge: SemanticEdge) {
        self.forward.entry(edge.from.clone()).or_default().push(edge);
    }

    /// Replaces edges from one node.
    pub fn replace(&mut self, node: SemanticNodeId, edges: Vec<SemanticEdge>) {
        self.forward.insert(node, edges);
    }

    /// Returns edges from one semantic node.
    pub fn edges_from(&self, node: &SemanticNodeId) -> &[SemanticEdge] {
        self.forward.get(node).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Returns all nodes in this graph.
    pub fn nodes(&self) -> Vec<SemanticNodeId> {
        let mut nodes = BTreeSet::new();
        for (from, edges) in &self.forward {
            nodes.insert(from.clone());
            nodes.extend(edges.iter().map(|edge| edge.to.clone()));
        }
        nodes.into_iter().collect()
    }

    /// Computes semantic SCCs without exposing the generic borrowed-adjacency
    /// limitation in callers.
    pub fn components(&self) -> Vec<Vec<SemanticNodeId>> {
        let nodes = self.nodes();
        let adjacency = nodes
            .iter()
            .map(|node| (node.clone(), self.edges_from(node).iter().map(|edge| edge.to.clone()).collect::<Vec<_>>()))
            .collect::<BTreeMap<_, _>>();
        strongly_connected_components(nodes, |node| adjacency.get(node).cloned().unwrap_or_default())
    }
}

/// Graph of eager runtime module initialization dependencies.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RuntimeDependencyGraph {
    forward: BTreeMap<ModuleId, Vec<RuntimeDependencyEdge>>,
}

impl RuntimeDependencyGraph {
    /// Adds a runtime dependency edge.
    pub fn add(&mut self, edge: RuntimeDependencyEdge) {
        self.forward.entry(edge.importer.clone()).or_default().push(edge);
    }

    /// Replaces runtime edges contributed by one importer.
    pub fn replace(&mut self, importer: ModuleId, edges: Vec<RuntimeDependencyEdge>) {
        self.forward.insert(importer, edges);
    }

    /// Returns runtime edges from an importer.
    pub fn edges_from(&self, importer: &ModuleId) -> &[RuntimeDependencyEdge] {
        self.forward.get(importer).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Returns dependencies of an importer in deterministic order.
    pub fn dependencies(&self, importer: &ModuleId) -> Vec<ModuleId> {
        let mut dependencies = self.edges_from(importer).iter().map(|edge| edge.dependency.clone()).collect::<Vec<_>>();
        dependencies.sort();
        dependencies.dedup();
        dependencies
    }

    /// Returns every node, including dependency-only targets.
    pub fn nodes(&self) -> Vec<ModuleId> {
        let mut nodes = BTreeSet::new();
        for (from, edges) in &self.forward {
            nodes.insert(from.clone());
            nodes.extend(edges.iter().map(|edge| edge.dependency.clone()));
        }
        nodes.into_iter().collect()
    }

    /// Validates that eager initialization dependencies form a DAG.
    pub fn validate_acyclic(&self) -> Result<(), crate::error::ModuleGraphError> {
        let components = self.components();
        for component in components {
            if component.len() > 1 || self.has_self_edge(&component[0]) {
                return Err(crate::error::ModuleGraphError::RuntimeCycle { cycle: component });
            }
        }
        Ok(())
    }

    /// Returns a deterministic initialization order with dependencies first.
    pub fn initialization_order(&self) -> Result<Vec<ModuleId>, crate::error::ModuleGraphError> {
        self.validate_acyclic()?;
        let nodes = self.nodes();
        let mut indegree = nodes.iter().map(|node| (node.clone(), 0usize)).collect::<BTreeMap<_, _>>();
        let mut dependents: BTreeMap<ModuleId, BTreeSet<ModuleId>> = BTreeMap::new();
        for importer in &nodes {
            for dependency in self.dependencies(importer) {
                *indegree.get_mut(importer).expect("all graph nodes initialized") += 1;
                dependents.entry(dependency).or_default().insert(importer.clone());
            }
        }

        let mut ready = indegree
            .iter()
            .filter_map(|(node, &degree)| (degree == 0).then_some(node.clone()))
            .collect::<BTreeSet<_>>();
        let mut order = Vec::with_capacity(nodes.len());
        while let Some(node) = ready.pop_first() {
            order.push(node.clone());
            for dependent in dependents.get(&node).into_iter().flatten() {
                let degree = indegree.get_mut(dependent).expect("dependent is a graph node");
                *degree -= 1;
                if *degree == 0 {
                    ready.insert(dependent.clone());
                }
            }
        }
        if order.len() != nodes.len() {
            return Err(crate::error::ModuleGraphError::RuntimeCycle { cycle: nodes });
        }
        Ok(order)
    }

    fn has_self_edge(&self, node: &ModuleId) -> bool {
        self.edges_from(node).iter().any(|edge| edge.dependency == *node)
    }

    fn components(&self) -> Vec<Vec<ModuleId>> {
        let nodes = self.nodes();
        let adjacency = nodes.iter().map(|node| (node.clone(), self.dependencies(node))).collect::<BTreeMap<_, _>>();
        strongly_connected_components(nodes, |node| adjacency.get(node).cloned().unwrap_or_default())
    }
}

/// All graph layers produced for one linked program.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModuleGraphs {
    /// Statically resolved source references.
    pub references: ReferenceGraph,
    /// Interface/declaration dependencies.
    pub semantics: SemanticGraph,
    /// Eager runtime initialization dependencies.
    pub runtime: RuntimeDependencyGraph,
}

/// Computes deterministic strongly connected components using an iterative
/// Kosaraju traversal. The caller owns adjacency storage and returns a slice
/// for each visited node.
pub fn strongly_connected_components<N, I>(nodes: impl IntoIterator<Item = N>, successors: impl Fn(&N) -> I) -> Vec<Vec<N>>
where
    N: Clone + Eq + Hash + Ord,
    I: IntoIterator<Item = N>,
{
    let nodes = nodes.into_iter().collect::<Vec<_>>();
    let indices = nodes.iter().enumerate().map(|(index, node)| (node.clone(), index)).collect::<BTreeMap<_, _>>();
    let mut adjacency = vec![Vec::<usize>::new(); nodes.len()];
    let mut reverse = vec![Vec::<usize>::new(); nodes.len()];
    for (index, node) in nodes.iter().enumerate() {
        for successor in successors(node) {
            if let Some(&target) = indices.get(&successor) {
                adjacency[index].push(target);
                reverse[target].push(index);
            }
        }
        adjacency[index].sort_unstable();
        adjacency[index].dedup();
        reverse[index].sort_unstable();
        reverse[index].dedup();
    }

    let mut visited = vec![false; nodes.len()];
    let mut finish = Vec::with_capacity(nodes.len());
    for start in 0..nodes.len() {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        let mut stack = vec![(start, 0usize)];
        while let Some((current, next)) = stack.last_mut() {
            if *next < adjacency[*current].len() {
                let target = adjacency[*current][*next];
                *next += 1;
                if !visited[target] {
                    visited[target] = true;
                    stack.push((target, 0));
                }
            } else {
                let (done, _) = stack.pop().expect("non-empty DFS stack");
                finish.push(done);
            }
        }
    }

    visited.fill(false);
    let mut components = Vec::new();
    while let Some(start) = finish.pop() {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        let mut stack = vec![start];
        let mut component = Vec::new();
        while let Some(current) = stack.pop() {
            component.push(nodes[current].clone());
            for &target in &reverse[current] {
                if !visited[target] {
                    visited[target] = true;
                    stack.push(target);
                }
            }
        }
        component.sort();
        components.push(component);
    }
    components.sort();
    components
}
