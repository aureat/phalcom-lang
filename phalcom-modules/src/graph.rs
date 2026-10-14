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

/// Product a generated runtime requirement needs from its canonical target.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeRequirementProduct {
    /// Stable semantic identity only.
    CanonicalIdentity,
    /// A descriptor installed from the semantic metadata bundle.
    MetadataDescriptor,
    /// A runtime module or object already materialized by the VM.
    RuntimeObject,
    /// A value produced by executing a source module initializer.
    SourceInitializedValue,
}

/// Latest point by which a generated requirement must be available.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeRequirementTiming {
    /// Before any canonical source initializer runs.
    BeforeSourceExecution,
    /// While a source module initializer is running.
    DuringModuleInitializer,
    /// After the canonical Universe bootstrap has completed.
    AfterBootstrap,
}

/// Semantic producer of a generated runtime requirement.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeRequirementProducer {
    /// A lowered read of a canonical declaration.
    DeclarationRead,
    /// A lowered read originating from an import binding.
    ImportRead,
    /// A compiler-inserted canonical runtime read.
    CompilerGeneratedRead,
    /// A callable owner needed while materializing a callable.
    CallableOwner,
    /// A pattern or variant owner needed by runtime matching.
    PatternOwner,
    /// A generic-conformance type owner.
    ConformanceTypeOwner,
    /// A contextual receiver value read.
    ContextualReceiverRead,
    /// A runtime type metadata owner.
    MetadataTypeOwner,
    /// A static constant initializer read.
    StaticConstantRead,
}

/// Exact canonical target retained for a generated requirement.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeRequirementTarget {
    /// The canonical module object itself.
    Module(ModuleId),
    /// A canonical module-owned binding.
    Binding { module: ModuleId, name: Box<str> },
}

impl RuntimeRequirementTarget {
    /// Returns the canonical module owning the exact target.
    pub fn module(&self) -> &ModuleId {
        match self {
            Self::Module(module) | Self::Binding { module, .. } => module,
        }
    }

    /// Returns the conservative product implied by the exact target.
    ///
    /// Only a binding named by the native installation catalog is guaranteed
    /// to exist before source execution. Other bindings and whole-module
    /// values may depend on their source initializer.
    pub fn conservative_product(&self) -> RuntimeRequirementProduct {
        match self {
            Self::Module(_) => RuntimeRequirementProduct::SourceInitializedValue,
            Self::Binding { module, name } => {
                let native_floor_binding = module.project.is_universe()
                    && phalcom_native_meta::UNIVERSE_BINDINGS.iter().any(|binding| {
                        binding.name == name.as_ref()
                            && binding
                                .key
                                .source_path()
                                .iter()
                                .copied()
                                .eq(module.path.components().iter().map(|component| component.as_str()))
                    });
                if native_floor_binding {
                    RuntimeRequirementProduct::RuntimeObject
                } else {
                    RuntimeRequirementProduct::SourceInitializedValue
                }
            }
        }
    }
}

/// Provenance retained on a generated dependency edge.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RuntimeRequirementProvenance {
    /// Consumer module (same as the edge importer).
    pub consumer: ModuleId,
    /// Requirement producer.
    pub producer: RuntimeRequirementProducer,
    /// Exact canonical target.
    pub target: RuntimeRequirementTarget,
    /// Product required from the target.
    pub product: RuntimeRequirementProduct,
    /// Required availability timing.
    pub timing: RuntimeRequirementTiming,
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
    /// Requirement details carried through executable finalization. Provisional
    /// legacy projections may not have source-phase provenance yet.
    pub provenance: Option<RuntimeRequirementProvenance>,
}

/// Graph of statically resolved source references.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReferenceGraph {
    forward: BTreeMap<ModuleId, Vec<ReferenceEdge>>,
}

impl ReferenceGraph {
    /// Registers a module as a node in the graph even if it has no outgoing edges.
    pub fn add_node(&mut self, module: ModuleId) {
        self.forward.entry(module).or_default();
    }

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

    /// Returns a graph containing only module-level semantic edges.
    ///
    /// Semantic analysis adds declaration-level edges after module linking;
    /// callers can use this projection to compare the linker-owned graph
    /// without confusing it with retained compiler-enrichment edges.
    pub fn module_projection(&self) -> Self {
        let mut projection = Self::default();
        for (node, edges) in &self.forward {
            if matches!(node, SemanticNodeId::Module(_)) {
                projection.forward.insert(node.clone(), edges.clone());
            }
        }
        projection
    }

    /// Returns source-declaration edges owned by one module.
    ///
    /// The linker does not produce these edges; semantic analysis retains and
    /// replaces them per source module across incremental revisions.
    pub fn declaration_edges_from_module(&self, module: &ModuleId) -> Vec<SemanticEdge> {
        self.forward
            .iter()
            .filter(|(node, _)| matches!(node, SemanticNodeId::Declaration { module: owner, .. } if owner == module))
            .flat_map(|(_, edges)| edges.iter().cloned())
            .collect()
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
    /// Registers a module as a node in the graph even if it has no outgoing edges.
    pub fn add_node(&mut self, module: ModuleId) {
        self.forward.entry(module).or_default();
    }

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
        if let Some(error) = self.cycle_error() {
            return Err(error);
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
            return Err(self.cycle_error().unwrap_or_else(|| crate::error::ModuleGraphError::RuntimeCycle {
                cycle: nodes.clone(),
                components: vec![nodes],
                path: Vec::new(),
            }));
        }
        Ok(order)
    }

    pub fn has_self_edge(&self, node: &ModuleId) -> bool {
        self.edges_from(node).iter().any(|edge| edge.dependency == *node)
    }

    pub fn components(&self) -> Vec<Vec<ModuleId>> {
        let nodes = self.nodes();
        let adjacency = nodes.iter().map(|node| (node.clone(), self.dependencies(node))).collect::<BTreeMap<_, _>>();
        let mut components = strongly_connected_components(nodes, |node| adjacency.get(node).cloned().unwrap_or_default());
        for component in &mut components {
            component.sort();
        }
        components.sort_by(|left, right| left.first().cmp(&right.first()));
        components
    }

    fn cycle_error(&self) -> Option<crate::error::ModuleGraphError> {
        let components = self
            .components()
            .into_iter()
            .filter(|component| component.len() > 1 || self.has_self_edge(&component[0]))
            .collect::<Vec<_>>();
        let first = components.first()?.clone();
        let allowed = first.iter().cloned().collect::<BTreeSet<_>>();
        let start = first[0].clone();
        let mut path = Vec::new();
        let mut visited = BTreeSet::from([start.clone()]);
        let _ = self.find_cycle_path(&start, &start, &allowed, &mut visited, &mut path);
        Some(crate::error::ModuleGraphError::RuntimeCycle {
            cycle: first,
            components,
            path,
        })
    }

    fn find_cycle_path(
        &self,
        current: &ModuleId,
        start: &ModuleId,
        allowed: &BTreeSet<ModuleId>,
        visited: &mut BTreeSet<ModuleId>,
        path: &mut Vec<RuntimeDependencyEdge>,
    ) -> bool {
        let mut edges = self
            .edges_from(current)
            .iter()
            .filter(|edge| allowed.contains(&edge.dependency))
            .cloned()
            .collect::<Vec<_>>();
        edges.sort_by(|left, right| {
            (&left.dependency, left.reason, left.range, &left.provenance).cmp(&(&right.dependency, right.reason, right.range, &right.provenance))
        });
        for edge in edges {
            if edge.dependency == *start {
                path.push(edge);
                return true;
            }
            if !visited.insert(edge.dependency.clone()) {
                continue;
            }
            let next = edge.dependency.clone();
            path.push(edge);
            if self.find_cycle_path(&next, start, allowed, visited, path) {
                return true;
            }
            path.pop();
            visited.remove(&next);
        }
        false
    }

    /// Returns a filtered subgraph containing only the specified nodes and edges between them.
    pub fn filtered_subgraph(&self, allowed: &BTreeSet<ModuleId>) -> Self {
        let mut forward = BTreeMap::new();
        for (importer, edges) in &self.forward {
            if !allowed.contains(importer) {
                continue;
            }
            let filtered_edges: Vec<_> = edges.iter().filter(|edge| allowed.contains(&edge.dependency)).cloned().collect();
            forward.insert(importer.clone(), filtered_edges);
        }
        for node in allowed {
            forward.entry(node.clone()).or_default();
        }
        Self { forward }
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

impl ModuleGraphs {
    /// Merges deterministic graph products from another linked component.
    pub fn merge_from(&mut self, other: &Self) {
        for module in other.references.nodes() {
            self.references.add_node(module.clone());
            self.references.replace(module.clone(), other.references.edges_from(&module).to_vec());
        }
        for node in other.semantics.nodes() {
            self.semantics.replace(node.clone(), other.semantics.edges_from(&node).to_vec());
        }
        for module in other.runtime.nodes() {
            self.runtime.add_node(module.clone());
            self.runtime.replace(module.clone(), other.runtime.edges_from(&module).to_vec());
        }
    }
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
