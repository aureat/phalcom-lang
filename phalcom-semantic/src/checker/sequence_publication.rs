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
    pub fn depends_on(&self, child: PublicationOwner, ancestor: PublicationOwner) -> bool {
        child == ancestor
            || self
                .owners
                .get(&child)
                .is_some_and(|owner| owner.parents.iter().any(|parent| self.depends_on(*parent, ancestor)))
    }

    pub fn requires_owner(&self, owner: PublicationOwner) -> bool {
        self.pending
            .values()
            .any(|(_, dependencies)| dependencies.iter().any(|dependency| self.depends_on(*dependency, owner)))
    }
    pub fn register(&mut self, owner: PublicationOwner, parent: Option<PublicationOwner>) {
        self.owners.entry(owner).or_insert_with(|| Owner {
            state: State::Open,
            parents: parent.into_iter().collect(),
        });
    }

    /// Flow probes interpret inherited owners without sharing product mutation.
    pub fn probe(&self) -> Self {
        Self {
            owners: self.owners.clone(),
            ..Self::default()
        }
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
            let transitions = self
                .owners
                .iter()
                .filter_map(|(gate, owner)| {
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
                })
                .collect::<Vec<_>>();
            if transitions.is_empty() {
                break;
            }
            for (gate, state) in transitions {
                if let Some(owner) = self.owners.get_mut(&gate) {
                    owner.state = state;
                }
            }
        }
        let completed = self
            .pending
            .iter()
            .filter_map(|(expression, (_, dependencies))| {
                if dependencies.iter().any(|owner| self.rejected(owner)) {
                    Some((*expression, false))
                } else if dependencies.iter().all(|owner| self.accepted(owner)) {
                    Some((*expression, true))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        for (expression, accepted) in completed {
            if let Some((resolution, _)) = self.pending.remove(&expression)
                && accepted
            {
                committed.insert(expression, resolution);
            }
        }
    }
}

/// Evidence retained from a completed application view, before its local row
/// premises are dropped. This record never owns another inference solver.
#[derive(Clone)]
pub(crate) struct DeferredApplicationRequirements {
    pub owner: PublicationOwner,
    pub expression: ExpressionId,
    pub range: phalcom_common::range::SourceRange,
    pub terms: std::collections::HashMap<crate::types::id::TypeParameterId, super::inference::InferenceTerm>,
    pub return_term: Option<super::inference::InferenceTerm>,
    pub signature_return: crate::types::evidence::TypeKnowledge,
    pub rows: Option<super::row_inference::QueryRowRequirements>,
    pub mandatory_constraints: std::ops::Range<u32>,
    pub signatures: Vec<crate::types::parameter::GenericSignature>,
    pub exact_arguments: BTreeMap<crate::types::id::TypeParameterId, crate::types::id::TypeId>,
    pub deferred: bool,
}

impl DeferredApplicationRequirements {
    /// Reuses the owner's existing final solve; only inspects frame completion
    /// and current row/materialization evidence. It does not solve scalar types.
    pub fn validate(&self, ctx: &mut super::context::CheckingContext<'_>) -> Result<(), super::analysis::AnalysisStatus> {
        use super::analysis::AnalysisStatus;
        use super::inference::{InferenceOutcome, InferenceProofState};
        use crate::types::outcome::BlockReason;
        let blocked = || AnalysisStatus::Blocked(BlockReason::RecursiveFixpoint);
        let session = ctx.inference_session(self.owner.0).ok_or_else(blocked)?;
        let session = session.borrow();
        match session.finish_frame(self.owner.1) {
            InferenceOutcome::Solved(_) => {}
            InferenceOutcome::Cancelled => return Err(AnalysisStatus::Cancelled),
            InferenceOutcome::BudgetExceeded(report) => return Err(AnalysisStatus::BudgetExceeded(report)),
            _ => return Err(blocked()),
        }
        if session
            .residual_obligations()
            .iter()
            .any(|obligation| self.mandatory_constraints.contains(&obligation.constraint_index))
        {
            return Err(blocked());
        }
        let mut instantiation = crate::types::instantiation::GenericInstantiation::default();
        if let Some(rows) = &self.rows {
            let solution = ctx.control.relation(|budget, cancellation| rows.solve(&session, budget, cancellation));
            let solution = match solution {
                crate::types::row_solver::RecordRowSolveResult::Solved(solution) => solution,
                crate::types::row_solver::RecordRowSolveResult::Cancelled => return Err(AnalysisStatus::Cancelled),
                crate::types::row_solver::RecordRowSolveResult::BudgetExceeded(report) => return Err(AnalysisStatus::BudgetExceeded(report)),
                _ => return Err(blocked()),
            };
            instantiation = rows.materialize(&session, &self.terms, &solution, ctx.store).map_err(|_| blocked())?;
            if let Some(term) = &self.return_term {
                if !matches!(
                    rows.proof(&session, term, &solution),
                    InferenceProofState::Established | InferenceProofState::Assumed
                ) {
                    return Err(blocked());
                }
                if let Some(ty) = self.signature_return.ty() {
                    crate::types::materialize_type(ctx.store, ty, &instantiation, crate::types::RowMaterializationMode::RequireSolvedTail)
                        .map_err(|_| blocked())?;
                }
            }
        } else {
            for (&parameter, term) in &self.terms {
                let ty = session.materialize(term, ctx.store).map_err(|_| blocked())?;
                instantiation.bind_type(parameter, ty);
            }
            if let Some(term) = &self.return_term {
                session.materialize(term, ctx.store).map_err(|_| blocked())?;
            }
            if let Some(term) = &self.return_term
                && !matches!(
                    session.proof_state_for_term(term),
                    InferenceProofState::Established | InferenceProofState::Assumed
                )
            {
                return Err(blocked());
            }
        }
        drop(session);
        if self.deferred && !super::call::validate_deferred_application_obligations(ctx, &self.signatures, &instantiation, &self.exact_arguments, self.range) {
            return Err(blocked());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::analysis::{SequenceConstructionPlan, SequenceLiteralFamily};
    use super::*;
    use crate::identity::{BodyId, LocalExpressionId};
    use crate::types::id::TypeId;

    fn owner(index: u32) -> PublicationOwner {
        PublicationOwner(InferenceContextId(index), InferenceFrameId(0))
    }

    fn resolution(index: u32) -> SequenceLiteralResolution {
        SequenceLiteralResolution {
            source_expression: ExpressionId::new(BodyId(0), LocalExpressionId(index)),
            source_range: Default::default(),
            family: SequenceLiteralFamily::Array,
            result_type: TypeId(0),
            element_type: TypeId(0),
            extent: None,
            construction: SequenceConstructionPlan::Fixed { element_count: 0 },
            exact_expansions: Default::default(),
        }
    }

    #[test]
    fn conjunction_waits_for_all_owners_and_rejection_preserves_independent_products() {
        let mut publication = SequencePublication::default();
        let mut committed = BTreeMap::new();
        publication.register(owner(1), None);
        publication.register(owner(2), Some(owner(1)));
        publication.stage(resolution(1), [owner(1), owner(2)].into_iter().collect(), &mut committed);
        publication.stage(resolution(2), Default::default(), &mut committed);
        publication.locally_ready(owner(2), &mut committed);
        assert_eq!(committed.len(), 1);
        publication.reject(owner(1), &mut committed);
        assert_eq!(committed.keys().copied().collect::<Vec<_>>(), vec![resolution(2).source_expression]);
        assert!(!publication.has_pending());
    }

    #[test]
    fn terminal_receipts_cover_late_candidates_and_missing_owners_fail_closed() {
        let mut publication = SequencePublication::default();
        let mut committed = BTreeMap::new();
        publication.register(owner(1), None);
        publication.locally_ready(owner(1), &mut committed);
        publication.dispose(owner(1).0, &mut committed);
        publication.stage(resolution(1), [owner(1)].into_iter().collect(), &mut committed);
        publication.stage(resolution(2), [owner(2)].into_iter().collect(), &mut committed);
        assert_eq!(committed.len(), 1);
        assert!(!publication.has_pending());
    }

    #[test]
    fn probe_rejection_and_context_disposal_cannot_publish_parent_candidates() {
        let mut publication = SequencePublication::default();
        let mut committed = BTreeMap::new();
        publication.register(owner(1), None);
        publication.stage(resolution(1), [owner(1)].into_iter().collect(), &mut committed);
        let mut probe = publication.probe();
        probe.reject(owner(1), &mut BTreeMap::new());
        assert!(publication.has_pending());
        publication.locally_ready(owner(1), &mut committed);
        assert_eq!(committed.len(), 1);
        publication.register(owner(2), None);
        publication.stage(resolution(2), [owner(2)].into_iter().collect(), &mut committed);
        publication.dispose(owner(2).0, &mut committed);
        assert_eq!(committed.len(), 1);
        assert!(!publication.has_pending());
    }
}
