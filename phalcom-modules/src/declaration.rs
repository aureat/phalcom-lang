//! Static declaration-shell planning for semantic SCCs.
//!
//! Runtime module initialization remains a separate DAG. This layer only
//! predeclares stable declaration identities so mutually-referential semantic
//! declarations can resolve without leaf-name fallback.

use crate::graph::{SemanticEdgeKind, SemanticGraph, SemanticNodeId};
use crate::identity::ModuleId;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DeclarationId {
    pub module: ModuleId,
    pub name: Box<str>,
}

impl DeclarationId {
    pub fn semantic_node(&self) -> SemanticNodeId {
        SemanticNodeId::Declaration {
            module: self.module.clone(),
            name: self.name.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeclarationKind {
    Class,
    Protocol,
    Adt,
    Alias,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationBlueprint {
    pub id: DeclarationId,
    pub kind: DeclarationKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShellState {
    Predeclared,
    Realized,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationShell {
    pub blueprint: DeclarationBlueprint,
    pub state: ShellState,
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DeclarationRealizationError {
    #[error("semantic edge references declaration without a predeclared shell: {0:?}")]
    MissingShell(SemanticNodeId),
    #[error("inheritance cycle is illegal: {cycle:?}")]
    InheritanceCycle { cycle: Vec<SemanticNodeId> },
}

#[derive(Clone, Debug, Default)]
pub struct DeclarationShellTable {
    shells: BTreeMap<SemanticNodeId, DeclarationShell>,
}

impl DeclarationShellTable {
    /// Phase A: allocate stable shells for *all* declarations before any edge
    /// inside a semantic SCC is resolved.
    pub fn predeclare(&mut self, blueprints: impl IntoIterator<Item = DeclarationBlueprint>) {
        for blueprint in blueprints {
            self.shells.entry(blueprint.id.semantic_node()).or_insert(DeclarationShell {
                blueprint,
                state: ShellState::Predeclared,
            });
        }
    }

    pub fn get(&self, node: &SemanticNodeId) -> Option<&DeclarationShell> {
        self.shells.get(node)
    }

    /// Phases B/C: prove every declaration edge targets a canonical shell,
    /// independently reject superclass cycles, then mark the SCC realized.
    pub fn realize_semantic_graph(&mut self, graph: &SemanticGraph) -> Result<(), DeclarationRealizationError> {
        for node in graph.nodes() {
            if matches!(node, SemanticNodeId::Declaration { .. }) && !self.shells.contains_key(&node) {
                return Err(DeclarationRealizationError::MissingShell(node));
            }
            for edge in graph.edges_from(&node) {
                if matches!(edge.to, SemanticNodeId::Declaration { .. }) && !self.shells.contains_key(&edge.to) {
                    return Err(DeclarationRealizationError::MissingShell(edge.to.clone()));
                }
            }
        }

        self.reject_inheritance_cycles(graph)?;
        for component in graph.components() {
            for node in component {
                if let Some(shell) = self.shells.get_mut(&node) {
                    shell.state = ShellState::Realized;
                }
            }
        }
        Ok(())
    }

    fn reject_inheritance_cycles(&self, graph: &SemanticGraph) -> Result<(), DeclarationRealizationError> {
        let mut superclass_only = SemanticGraph::default();
        for node in graph.nodes() {
            for edge in graph.edges_from(&node) {
                if edge.kind == SemanticEdgeKind::Superclass {
                    superclass_only.add(edge.clone());
                }
            }
        }
        for component in superclass_only.components() {
            let self_edge = component.len() == 1
                && superclass_only.edges_from(&component[0]).iter().any(|edge| edge.to == component[0]);
            if component.len() > 1 || self_edge {
                return Err(DeclarationRealizationError::InheritanceCycle { cycle: component });
            }
        }
        Ok(())
    }

    pub fn realized_ids(&self) -> BTreeSet<SemanticNodeId> {
        self.shells
            .iter()
            .filter_map(|(id, shell)| (shell.state == ShellState::Realized).then_some(id.clone()))
            .collect()
    }
}
