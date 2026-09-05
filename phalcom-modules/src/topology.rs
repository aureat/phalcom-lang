//! Canonical module topology and path resolution fingerprint.
//!
//! Represents the structural DAG of modules, packages, project boundaries,
//! and package exposure boundaries, independent of source method bodies.

use crate::identity::{
    ImportRootTarget, ModuleComponent, ModuleId, ModulePath, ProjectIdentity, SourceId, SourceLocation,
};
use crate::interface::UnlinkedModuleInterface;
use crate::project::ProjectUniverse;
use crate::source::ModuleKind;
use crate::stabilization::ResolverGeneration;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};

/// Deterministic fingerprint representing the namespace topology of a workspace.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TopologyFingerprint(pub u64);

impl TopologyFingerprint {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// A node in the canonical module topology.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopologyNode {
    pub module_id: ModuleId,
    pub kind: ModuleKind,
    pub source: Option<SourceLocation>,
    pub project: ProjectIdentity,
    pub parent: Option<ModuleId>,
}

/// Canonical workspace module topology.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleTopology {
    pub generation: ResolverGeneration,
    pub fingerprint: TopologyFingerprint,
    pub nodes: BTreeMap<ModuleId, TopologyNode>,
    pub source_modules: BTreeMap<SourceId, ModuleId>,
    pub children: BTreeMap<ModuleId, BTreeSet<ModuleId>>,
    pub exposed_children: BTreeMap<ModuleId, BTreeSet<ModuleComponent>>,
}

impl ModuleTopology {
    /// Builds a canonical topology snapshot from current universe, interfaces, and sources.
    pub fn from_parts(
        generation: ResolverGeneration,
        universe: &ProjectUniverse,
        unlinked: &BTreeMap<ModuleId, UnlinkedModuleInterface>,
        sources: &BTreeMap<ModuleId, SourceLocation>,
    ) -> Self {
        let mut nodes = BTreeMap::new();
        let mut source_modules = BTreeMap::new();
        let mut children: BTreeMap<ModuleId, BTreeSet<ModuleId>> = BTreeMap::new();
        let mut exposed_children = BTreeMap::new();

        for (module_id, iface) in unlinked {
            let parent = if module_id.path.is_root() {
                None
            } else {
                let parent_path = module_id.path.parent().unwrap_or_else(ModulePath::root);
                Some(ModuleId {
                    project: module_id.project,
                    path: parent_path,
                })
            };
            let source = sources.get(module_id).cloned();
            let node = TopologyNode {
                module_id: module_id.clone(),
                kind: iface.kind,
                source,
                project: module_id.project,
                parent: parent.clone(),
            };
            nodes.insert(module_id.clone(), node);
            if let Some(p) = parent {
                children.entry(p).or_default().insert(module_id.clone());
            }
            if iface.kind == ModuleKind::Package && !iface.exposed_children.is_empty() {
                exposed_children.insert(module_id.clone(), iface.exposed_children.clone());
            }
        }

        for (module_id, loc) in sources {
            source_modules.insert(loc.source_id.clone(), module_id.clone());
            if !nodes.contains_key(module_id) {
                let parent = if module_id.path.is_root() {
                    None
                } else {
                    let parent_path = module_id.path.parent().unwrap_or_else(ModulePath::root);
                    Some(ModuleId {
                        project: module_id.project,
                        path: parent_path,
                    })
                };
                let node = TopologyNode {
                    module_id: module_id.clone(),
                    kind: ModuleKind::Module,
                    source: Some(loc.clone()),
                    project: module_id.project,
                    parent: parent.clone(),
                };
                nodes.insert(module_id.clone(), node);
                if let Some(p) = parent {
                    children.entry(p).or_default().insert(module_id.clone());
                }
            }
        }

        let fingerprint = compute_topology_fingerprint(universe, &nodes, &exposed_children);

        Self {
            generation,
            fingerprint,
            nodes,
            source_modules,
            children,
            exposed_children,
        }
    }

    /// Whether this topology contains the given module.
    pub fn contains_module(&self, module: &ModuleId) -> bool {
        self.nodes.contains_key(module)
    }

    /// Returns the node for a module if known.
    pub fn get_node(&self, module: &ModuleId) -> Option<&TopologyNode> {
        self.nodes.get(module)
    }

    /// Returns direct child module identities of the given parent.
    pub fn module_children(&self, parent: &ModuleId) -> Option<&BTreeSet<ModuleId>> {
        self.children.get(parent)
    }

    /// Returns the exposed child components of a package.
    pub fn exposed_children(&self, package: &ModuleId) -> Option<&BTreeSet<ModuleComponent>> {
        self.exposed_children.get(package)
    }

    /// Returns the module associated with a source-provider identity.
    pub fn module_for_source(&self, source: &SourceId) -> Option<&ModuleId> {
        self.source_modules.get(source)
    }

    /// Collects all transitive descendant module identities of a root module.
    pub fn descendants(&self, root: &ModuleId) -> BTreeSet<ModuleId> {
        let mut result = BTreeSet::new();
        let mut queue = std::collections::VecDeque::new();
        if let Some(direct) = self.children.get(root) {
            for child in direct {
                queue.push_back(child);
            }
        }
        while let Some(current) = queue.pop_front() {
            if result.insert(current.clone()) {
                if let Some(direct) = self.children.get(current) {
                    for child in direct {
                        queue.push_back(child);
                    }
                }
            }
        }
        result
    }

    /// Detects whether a directed import dependency graph contains a cycle among topology nodes.
    ///
    /// Returns `Some(cycle_path)` if a cycle exists, where the path starts and ends at the same node.
    pub fn detect_cycle(&self, import_edges: &BTreeMap<ModuleId, BTreeSet<ModuleId>>) -> Option<Vec<ModuleId>> {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Visit {
            Visiting,
            Visited,
        }
        let mut state: BTreeMap<&ModuleId, Visit> = BTreeMap::new();
        let mut path: Vec<ModuleId> = Vec::new();

        fn dfs<'a>(
            node: &'a ModuleId,
            edges: &'a BTreeMap<ModuleId, BTreeSet<ModuleId>>,
            state: &mut BTreeMap<&'a ModuleId, Visit>,
            path: &mut Vec<ModuleId>,
        ) -> Option<Vec<ModuleId>> {
            match state.get(node) {
                Some(Visit::Visiting) => {
                    let mut cycle = Vec::new();
                    let mut started = false;
                    for p in path.iter() {
                        if p == node {
                            started = true;
                        }
                        if started {
                            cycle.push(p.clone());
                        }
                    }
                    cycle.push(node.clone());
                    return Some(cycle);
                }
                Some(Visit::Visited) => return None,
                None => {}
            }

            state.insert(node, Visit::Visiting);
            path.push(node.clone());

            if let Some(deps) = edges.get(node) {
                for dep in deps {
                    if let Some(cycle) = dfs(dep, edges, state, path) {
                        return Some(cycle);
                    }
                }
            }

            path.pop();
            state.insert(node, Visit::Visited);
            None
        }

        for module in self.nodes.keys() {
            if !state.contains_key(module) {
                if let Some(cycle) = dfs(module, import_edges, &mut state, &mut path) {
                    return Some(cycle);
                }
            }
        }
        None
    }
}

fn compute_topology_fingerprint(
    universe: &ProjectUniverse,
    nodes: &BTreeMap<ModuleId, TopologyNode>,
    exposed_children: &BTreeMap<ModuleId, BTreeSet<ModuleComponent>>,
) -> TopologyFingerprint {
    let mut hasher = DefaultHasher::new();

    let projects = universe.projects();
    projects.len().hash(&mut hasher);
    for proj in projects {
        proj.id.hash(&mut hasher);
        proj.name.hash(&mut hasher);
        proj.namespace.hash(&mut hasher);
        proj.persistent_project.hash(&mut hasher);
        let roots = proj.import_roots();
        roots.len().hash(&mut hasher);
        for (root_name, (target, is_self)) in roots {
            root_name.hash(&mut hasher);
            match target {
                ImportRootTarget::Universe => 0u8.hash(&mut hasher),
                ImportRootTarget::Resolved(rid) => {
                    1u8.hash(&mut hasher);
                    rid.hash(&mut hasher);
                }
            }
            is_self.hash(&mut hasher);
        }
    }

    nodes.len().hash(&mut hasher);
    for (id, node) in nodes {
        id.hash(&mut hasher);
        match node.kind {
            ModuleKind::Module => 0u8.hash(&mut hasher),
            ModuleKind::Package => 1u8.hash(&mut hasher),
        }
        node.project.hash(&mut hasher);
        node.parent.hash(&mut hasher);
    }

    exposed_children.len().hash(&mut hasher);
    for (pkg_id, exposed) in exposed_children {
        pkg_id.hash(&mut hasher);
        exposed.len().hash(&mut hasher);
        for comp in exposed {
            comp.hash(&mut hasher);
        }
    }

    TopologyFingerprint::new(hasher.finish())
}
