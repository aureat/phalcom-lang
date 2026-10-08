//! Query-local lifetime of executable sequence construction capabilities.
//!
//! Solver progress is not acceptance. A locally ready owner must also wait for
//! its actual ancestors; rejection removes only products using that evidence.

use super::analysis::{SequenceLiteralResolution, SequenceLiteralResolutionIndex};
use super::inference::{InferenceContextId, InferenceFrameId};
use crate::identity::ExpressionId;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct PublicationOwner(pub InferenceContextId, pub InferenceFrameId);

pub(crate) type PublicationDependencies = BTreeSet<PublicationOwner>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    Open,
    LocallyReady,
    Accepted,
    Rejected,
}

#[derive(Clone, Debug)]
struct Owner {
    state: State,
    parents: PublicationDependencies,
}

#[derive(Clone, Default)]
pub(crate) struct SequencePublication {
    owners: BTreeMap<PublicationOwner, Owner>,
    pending: BTreeMap<ExpressionId, (SequenceLiteralResolution, PublicationDependencies)>,
    pub expression_dependencies: BTreeMap<ExpressionId, PublicationDependencies>,
}

impl SequencePublication {
    pub fn register(&mut self, owner: PublicationOwner, parent: Option<PublicationOwner>) {
        self.owners.entry(owner).or_insert_with(|| Owner {
            state: State::Open,
            parents: parent.into_iter().collect(),
        });
    }

    /// Flow probes interpret inherited owners without sharing product mutation.
    pub fn probe(&self) -> Self {
        Self { owners: self.owners.clone(), ..Self::default() }
    }

    pub fn stage(&mut self, resolution: SequenceLiteralResolution, dependencies: PublicationDependencies, committed: &mut SequenceLiteralResolutionIndex) {
        let expression = resolution.source_expression;
        self.pending.insert(expression, (resolution, dependencies));
        self.flush(committed);
    }

    pub fn local_resolution(&self, expression: ExpressionId) -> Option<&SequenceLiteralResolution> {
        self.pending.get(&expression).map(|(resolution, _)| resolution)
    }

    pub fn discard_expression(&mut self, expression: ExpressionId) {
        self.pending.remove(&expression);
    }

    pub fn locally_ready(&mut self, owner: PublicationOwner, committed: &mut SequenceLiteralResolutionIndex) {
        if let Some(record) = self.owners.get_mut(&owner)
            && record.state == State::Open
        {
            record.state = State::LocallyReady;
        }
        self.flush(committed);
    }

    pub fn reject(&mut self, owner: PublicationOwner, committed: &mut SequenceLiteralResolutionIndex) {
        if let Some(record) = self.owners.get_mut(&owner)
            && record.state != State::Accepted
        {
            record.state = State::Rejected;
        }
        self.flush(committed);
    }

    pub fn dispose(&mut self, context: InferenceContextId, committed: &mut SequenceLiteralResolutionIndex) {
        for (gate, owner) in &mut self.owners {
            if gate.0 == context && !matches!(owner.state, State::Accepted | State::Rejected) {
                owner.state = State::Rejected;
            }
        }
        self.flush(committed);
    }

    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    fn accepted(&self, owner: &PublicationOwner) -> bool {
        self.owners.get(owner).is_some_and(|record| record.state == State::Accepted)
    }

    fn rejected(&self, owner: &PublicationOwner) -> bool {
        self.owners.get(owner).is_none_or(|record| record.state == State::Rejected)
    }

    fn flush(&mut self, committed: &mut SequenceLiteralResolutionIndex) {
        loop {
            let transitions = self.owners.iter().filter_map(|(gate, owner)| {
                if matches!(owner.state, State::Accepted | State::Rejected) {
                    return None;
                }
                if owner.parents.iter().any(|parent| self.rejected(parent)) {
                    Some((*gate, State::Rejected))
                } else if owner.state == State::LocallyReady && owner.parents.iter().all(|parent| self.accepted(parent)) {
                    Some((*gate, State::Accepted))
                } else {
                    None
                }
            }).collect::<Vec<_>>();
            if transitions.is_empty() {
                break;
            }
            for (gate, state) in transitions {
                if let Some(owner) = self.owners.get_mut(&gate) {
                    owner.state = state;
                }
            }
        }
        let completed = self.pending.iter().filter_map(|(expression, (_, dependencies))| {
            if dependencies.iter().any(|owner| self.rejected(owner)) {
                Some((*expression, false))
            } else if dependencies.iter().all(|owner| self.accepted(owner)) {
                Some((*expression, true))
            } else {
                None
            }
        }).collect::<Vec<_>>();
        for (expression, accepted) in completed {
            if let Some((resolution, _)) = self.pending.remove(&expression)
                && accepted
            {
                committed.insert(expression, resolution);
            }
        }
    }
}
