//! Arena for interning and managing explanation nodes (Spec 04.5).

use super::node::{ExplanationNode, ExplanationStep};
use crate::identity::ExplanationId;
use crate::types::evidence::EvidenceAuthority;
use std::collections::BTreeMap;

/// Arena holding explanation DAG nodes for a callable body analysis.
#[derive(Clone, Debug, Default)]
pub struct ExplanationArena {
    nodes: BTreeMap<ExplanationId, ExplanationNode>,
    next_id: u32,
}

impl ExplanationArena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alloc(&mut self, step: ExplanationStep, authority: EvidenceAuthority, parents: Vec<ExplanationId>) -> ExplanationId {
        let rule = step.derivation_rule();
        let evidence = Vec::new();
        self.alloc_full(step, rule, authority, evidence, parents)
    }

    pub fn alloc_full(
        &mut self,
        step: ExplanationStep,
        rule: super::node::DerivationRule,
        authority: EvidenceAuthority,
        evidence: Vec<super::node::EvidenceRef>,
        parents: Vec<ExplanationId>,
    ) -> ExplanationId {
        let id = ExplanationId(self.next_id);
        self.next_id += 1;
        self.nodes.insert(
            id,
            ExplanationNode {
                id,
                step,
                rule,
                authority,
                evidence,
                parents,
            },
        );
        id
    }

    pub fn get(&self, id: ExplanationId) -> Option<&ExplanationNode> {
        self.nodes.get(&id)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
