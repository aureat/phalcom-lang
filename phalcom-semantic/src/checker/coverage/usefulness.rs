//! Finite demand-driven pattern-matrix usefulness and exhaustiveness engine.

use crate::declarations::DeclarationTypeTable;
use crate::enum_semantics::EnumSemanticTable;
use crate::match_semantics::{BranchProofEnvironment, CoverageWitness, ExhaustivenessResult, PatternSpaceSummary, PatternUsefulness};
use crate::types::outcome::BlockReason;
use crate::types::evidence::UnknownReason;
use crate::types::relation::TypeHierarchy;
use crate::types::rigid::RigidArena;
use crate::types::store::TypeStore;
use crate::checker::context::CheckerControl;

use super::domain::{ConstructorCase, ConstructorHead, DomainDecomposition, decompose_domain};
use super::inhabitation::{Inhabitation, check_inhabitation};
use super::pattern::{CoveragePattern, CoveragePatternArena, CoveragePatternId};
use super::subject::CoverageSubject;

pub(crate) const MAX_COVERAGE_WITNESSES: usize = 8;

#[derive(Clone, Debug)]
pub(crate) enum UsefulnessSearch {
    Useful(Option<CoverageWitness>),
    NotUseful,
    Blocked(BlockReason),
}

/// Bounded structural work performed by one coverage query.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CoverageMetrics {
    pub(crate) constructor_decompositions: usize,
    pub(crate) matrix_specializations: usize,
    pub(crate) proof_merges: usize,
    pub(crate) witness_states: usize,
    pub(crate) inhabitation_iterations: usize,
}

/// Demand-driven pattern matrix coverage engine.
#[derive(Clone)]
pub(crate) struct CoverageEngine {
    root: CoverageSubject,
    arena: CoveragePatternArena,
    prior_matrix: Vec<Vec<CoveragePatternId>>,
    blocked: Option<BlockReason>,
    inhabitation_cache: Option<Inhabitation>,
    control: CheckerControl,
    metrics: CoverageMetrics,
}

impl CoverageEngine {
    pub(crate) fn new(root: CoverageSubject, control: CheckerControl) -> Self {
        Self {
            root,
            arena: CoveragePatternArena::new(),
            prior_matrix: Vec::new(),
            blocked: None,
            inhabitation_cache: None,
            control,
            metrics: CoverageMetrics::default(),
        }
    }

    pub(crate) fn arena(&self) -> &CoveragePatternArena {
        &self.arena
    }

    pub(crate) fn arena_mut(&mut self) -> &mut CoveragePatternArena {
        &mut self.arena
    }

    pub(crate) fn root(&self) -> &CoverageSubject {
        &self.root
    }

    pub(crate) fn blocked_reason(&self) -> Option<&BlockReason> {
        self.blocked.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn metrics(&self) -> &CoverageMetrics {
        &self.metrics
    }

    pub(crate) fn retain_blocked(&mut self, reason: BlockReason) {
        if self.blocked.is_none() {
            self.blocked = Some(reason);
        }
    }

    fn root_inhabitation(
        &mut self,
        declarations: &DeclarationTypeTable,
        store: &mut TypeStore,
        hierarchy: &dyn TypeHierarchy,
        rigids: &mut RigidArena,
        enum_table: Option<&EnumSemanticTable>,
    ) -> Inhabitation {
        if let Some(result) = &self.inhabitation_cache {
            return result.clone();
        }
        let result = check_inhabitation(
            declarations,
            store,
            hierarchy,
            rigids,
            enum_table,
            &self.root,
            &self.control,
            &mut self.metrics,
        );
        self.inhabitation_cache = Some(result.clone());
        result
    }

    fn classify_in_domain(&mut self, result: UsefulnessSearch) -> Option<PatternUsefulness> {
        match result {
            UsefulnessSearch::NotUseful => Some(PatternUsefulness::Impossible),
            UsefulnessSearch::Blocked(reason) => {
                self.retain_blocked(reason);
                Some(PatternUsefulness::Useful)
            }
            UsefulnessSearch::Useful(_) => None,
        }
    }

    #[cfg(test)]
    fn mark_blocked_for_test(&mut self, reason: BlockReason) {
        self.retain_blocked(reason);
    }

    pub(crate) fn classify_arm(
        &mut self,
        declarations: &DeclarationTypeTable,
        store: &mut TypeStore,
        hierarchy: &dyn TypeHierarchy,
        rigids: &mut RigidArena,
        enum_table: Option<&EnumSemanticTable>,
        pattern: CoveragePatternId,
    ) -> PatternUsefulness {
        if self.blocked.is_some() {
            return PatternUsefulness::Useful;
        }
        match self.root_inhabitation(declarations, store, hierarchy, rigids, enum_table) {
            Inhabitation::Uninhabited => return PatternUsefulness::Impossible,
            Inhabitation::Blocked(reason) => {
                self.retain_blocked(reason);
                return PatternUsefulness::Useful;
            }
            Inhabitation::Inhabited | Inhabitation::Unknown => {}
        }

        // 1. Check if pattern has values in the domain (useful against empty matrix):
        let in_domain = useful_internal(
            declarations,
            store,
            hierarchy,
            rigids,
            enum_table,
            &mut self.arena,
            &self.control,
            &mut self.metrics,
            &[],
            &[pattern],
            std::slice::from_ref(&self.root),
            &BranchProofEnvironment::default(),
        );
        if let Some(classification) = self.classify_in_domain(in_domain) {
            return classification;
        }

        // 2. Check if useful against prior matrix:
        let against_prior = useful_internal(
            declarations,
            store,
            hierarchy,
            rigids,
            enum_table,
            &mut self.arena,
            &self.control,
            &mut self.metrics,
            &self.prior_matrix,
            &[pattern],
            std::slice::from_ref(&self.root),
            &BranchProofEnvironment::default(),
        );
        match against_prior {
            UsefulnessSearch::Useful(_) => PatternUsefulness::Useful,
            UsefulnessSearch::NotUseful => PatternUsefulness::Redundant,
            UsefulnessSearch::Blocked(b) => {
                self.retain_blocked(b);
                PatternUsefulness::Useful
            }
        }
    }

    pub(crate) fn commit_arm(&mut self, pattern: CoveragePatternId) {
        self.prior_matrix.push(vec![pattern]);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn check_or_alternative(
        &mut self,
        declarations: &DeclarationTypeTable,
        store: &mut TypeStore,
        hierarchy: &dyn TypeHierarchy,
        rigids: &mut RigidArena,
        enum_table: Option<&EnumSemanticTable>,
        prior_alternatives: &[CoveragePatternId],
        candidate: CoveragePatternId,
    ) -> PatternUsefulness {
        if self.blocked.is_some() {
            return PatternUsefulness::Useful;
        }
        match self.root_inhabitation(declarations, store, hierarchy, rigids, enum_table) {
            Inhabitation::Uninhabited => return PatternUsefulness::Impossible,
            Inhabitation::Blocked(reason) => {
                self.retain_blocked(reason);
                return PatternUsefulness::Useful;
            }
            Inhabitation::Inhabited | Inhabitation::Unknown => {}
        }

        // Check in domain:
        let in_domain = useful_internal(
            declarations,
            store,
            hierarchy,
            rigids,
            enum_table,
            &mut self.arena,
            &self.control,
            &mut self.metrics,
            &[],
            &[candidate],
            std::slice::from_ref(&self.root),
            &BranchProofEnvironment::default(),
        );
        if let Some(classification) = self.classify_in_domain(in_domain) {
            return classification;
        }

        // Check against prior alternatives:
        let prior_matrix: Vec<Vec<CoveragePatternId>> = prior_alternatives.iter().map(|&alt| vec![alt]).collect();
        let against_prior = useful_internal(
            declarations,
            store,
            hierarchy,
            rigids,
            enum_table,
            &mut self.arena,
            &self.control,
            &mut self.metrics,
            &prior_matrix,
            &[candidate],
            std::slice::from_ref(&self.root),
            &BranchProofEnvironment::default(),
        );
        match against_prior {
            UsefulnessSearch::Useful(_) => PatternUsefulness::Useful,
            UsefulnessSearch::NotUseful => PatternUsefulness::Redundant,
            UsefulnessSearch::Blocked(b) => {
                self.retain_blocked(b);
                PatternUsefulness::Useful
            }
        }
    }

    pub(crate) fn finalize_exhaustiveness(
        &mut self,
        declarations: &DeclarationTypeTable,
        store: &mut TypeStore,
        hierarchy: &dyn TypeHierarchy,
        rigids: &mut RigidArena,
        enum_table: Option<&EnumSemanticTable>,
    ) -> ExhaustivenessResult {
        if let Some(reason) = self.blocked.clone() {
            return ExhaustivenessResult::Blocked(reason);
        }
        match self.root_inhabitation(declarations, store, hierarchy, rigids, enum_table) {
            Inhabitation::Uninhabited => return ExhaustivenessResult::Proven,
            Inhabitation::Blocked(reason) => return ExhaustivenessResult::Blocked(reason),
            Inhabitation::Inhabited | Inhabitation::Unknown => {}
        }

        if self.prior_matrix.is_empty() {
            match decompose_domain(declarations, store, hierarchy, rigids, enum_table, &self.root) {
                DomainDecomposition::Empty => return ExhaustivenessResult::Proven,
                DomainDecomposition::Blocked(b) => return ExhaustivenessResult::Blocked(b),
                DomainDecomposition::Closed(cases) => {
                    let witnesses: Vec<CoverageWitness> = cases
                        .iter()
                        .take(MAX_COVERAGE_WITNESSES)
                        .map(|case| construct_case_witness(case, &self.root, None))
                        .collect();
                    return ExhaustivenessResult::Missing(witnesses.into_boxed_slice());
                }
                DomainDecomposition::Open => {
                    return ExhaustivenessResult::Missing(Box::new([CoverageWitness::Opaque(self.root.canonical)]));
                }
            }
        }

        let wildcard = self.arena.wildcard();
        let search = useful_internal(
            declarations,
            store,
            hierarchy,
            rigids,
            enum_table,
            &mut self.arena,
            &self.control,
            &mut self.metrics,
            &self.prior_matrix,
            &[wildcard],
            std::slice::from_ref(&self.root),
            &BranchProofEnvironment::default(),
        );
        match search {
            UsefulnessSearch::NotUseful => ExhaustivenessResult::Proven,
            UsefulnessSearch::Useful(wit) => {
                let witness = wit.unwrap_or(CoverageWitness::Opaque(self.root.canonical));
                ExhaustivenessResult::Missing(Box::new([witness]))
            }
            UsefulnessSearch::Blocked(b) => ExhaustivenessResult::Blocked(b),
        }
    }

    pub(crate) fn summarize_residual(
        &mut self,
        declarations: &DeclarationTypeTable,
        store: &mut TypeStore,
        hierarchy: &dyn TypeHierarchy,
        rigids: &mut RigidArena,
        enum_table: Option<&EnumSemanticTable>,
    ) -> PatternSpaceSummary {
        if let Some(reason) = self.blocked.clone() {
            return PatternSpaceSummary::Blocked(reason);
        }
        match self.root_inhabitation(declarations, store, hierarchy, rigids, enum_table) {
            Inhabitation::Uninhabited => return PatternSpaceSummary::Empty,
            Inhabitation::Blocked(reason) => return PatternSpaceSummary::Blocked(reason),
            Inhabitation::Inhabited | Inhabitation::Unknown => {}
        }
        let wildcard = self.arena.wildcard();
        let search = useful_internal(
            declarations,
            store,
            hierarchy,
            rigids,
            enum_table,
            &mut self.arena,
            &self.control,
            &mut self.metrics,
            &self.prior_matrix,
            &[wildcard],
            std::slice::from_ref(&self.root),
            &BranchProofEnvironment::default(),
        );
        match search {
            UsefulnessSearch::NotUseful => PatternSpaceSummary::Empty,
            UsefulnessSearch::Useful(wit) => {
                let witness = wit.unwrap_or(CoverageWitness::Opaque(self.root.canonical));
                witness_to_summary(&witness)
            }
            UsefulnessSearch::Blocked(reason) => {
                self.retain_blocked(reason.clone());
                PatternSpaceSummary::Blocked(reason)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarations::bootstrap_universe_declarations;
    use crate::identity::{DeclarationId, ModuleId};
    use crate::types::id::TypeId;
    use crate::types::relation::MapTypeHierarchy;
    use crate::types::rigid::RigidArena;
    use crate::types::store::TypeStore;
    use crate::db::budget::{CancellationToken, QueryBudget};

    #[test]
    fn first_blocked_state_is_sticky_and_distinct_from_known_opaque() {
        let reason = BlockReason::RecursiveFixpoint;
        let mut engine = CoverageEngine::new(CoverageSubject::canonical(TypeId(0)), CheckerControl::default());
        assert_eq!(engine.classify_in_domain(UsefulnessSearch::Blocked(reason.clone())), Some(PatternUsefulness::Useful));
        engine.mark_blocked_for_test(BlockReason::ReflectionBoundary);

        assert_eq!(engine.blocked_reason(), Some(&reason));

        let module = ModuleId::universe_root();
        let mut store = TypeStore::new();
        let declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(module.clone(), key.name().into()));
        let hierarchy = MapTypeHierarchy::new();
        let mut rigids = RigidArena::new();
        assert_eq!(
            engine.summarize_residual(&declarations, &mut store, &hierarchy, &mut rigids, None),
            PatternSpaceSummary::Blocked(reason.clone())
        );
        assert_eq!(
            engine.finalize_exhaustiveness(&declarations, &mut store, &hierarchy, &mut rigids, None),
            ExhaustivenessResult::Blocked(reason)
        );
    }

    #[test]
    fn shared_control_budget_and_cancellation_fail_closed() {
        let module = ModuleId::universe_root();
        let mut store = TypeStore::new();
        let declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(module.clone(), key.name().into()));
        let hierarchy = MapTypeHierarchy::new();
        let mut rigids = RigidArena::new();
        let token = CancellationToken::new();
        let control = CheckerControl::new(QueryBudget::new(1), &token);
        let mut engine = CoverageEngine::new(CoverageSubject::canonical(store.unit()), control);
        let wildcard = engine.arena_mut().wildcard();

        assert_eq!(
            engine.classify_arm(&declarations, &mut store, &hierarchy, &mut rigids, None, wildcard),
            PatternUsefulness::Useful
        );
        assert!(engine.metrics().witness_states > 0);
        assert!(engine.metrics().inhabitation_iterations > 0);
        assert!(matches!(engine.finalize_exhaustiveness(&declarations, &mut store, &hierarchy, &mut rigids, None), ExhaustivenessResult::Blocked(BlockReason::BudgetExceeded(_))));

        let cancel = CancellationToken::new();
        cancel.cancel();
        let control = CheckerControl::new(QueryBudget::default(), &cancel);
        let mut engine = CoverageEngine::new(CoverageSubject::canonical(store.unit()), control);
        let wildcard = engine.arena_mut().wildcard();
        assert_eq!(
            engine.classify_arm(&declarations, &mut store, &hierarchy, &mut rigids, None, wildcard),
            PatternUsefulness::Useful
        );
        assert!(matches!(
            engine.finalize_exhaustiveness(&declarations, &mut store, &hierarchy, &mut rigids, None),
            ExhaustivenessResult::Blocked(BlockReason::UnknownType(UnknownReason::InferenceCancelled))
        ));
    }
}

pub(crate) fn witness_to_summary(witness: &CoverageWitness) -> PatternSpaceSummary {
    match witness {
        CoverageWitness::Wildcard => PatternSpaceSummary::Empty,
        CoverageWitness::Opaque(ty) => PatternSpaceSummary::Opaque(*ty),
        CoverageWitness::Variant { variant, exact_case, fields } => {
            let field_summaries: Box<[PatternSpaceSummary]> = fields.iter().map(witness_to_summary).collect();
            PatternSpaceSummary::Variant {
                variant: variant.clone(),
                exact_case: *exact_case,
                fields: field_summaries,
            }
        }
        CoverageWitness::Tuple(fields) => {
            let field_summaries: Box<[PatternSpaceSummary]> = fields.iter().map(witness_to_summary).collect();
            PatternSpaceSummary::Tuple(field_summaries)
        }
        CoverageWitness::List(_) => PatternSpaceSummary::List,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn summarize_pattern(
    declarations: &DeclarationTypeTable,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    rigids: &mut RigidArena,
    enum_table: Option<&EnumSemanticTable>,
    arena: &CoveragePatternArena,
    pattern: CoveragePatternId,
    subject: &CoverageSubject,
) -> PatternSpaceSummary {
    match arena.get(pattern) {
        CoveragePattern::Wildcard => PatternSpaceSummary::Opaque(subject.canonical),
        CoveragePattern::Variant { candidates, fields, .. } => {
            let cases = match decompose_domain(declarations, store, hierarchy, rigids, enum_table, subject) {
                DomainDecomposition::Closed(cases) => cases,
                DomainDecomposition::Empty => return PatternSpaceSummary::Empty,
                DomainDecomposition::Blocked(reason) => return PatternSpaceSummary::Blocked(reason),
                DomainDecomposition::Open => return PatternSpaceSummary::Opaque(subject.canonical),
            };
            let mut summaries = Vec::new();
            for case in cases.iter() {
                let ConstructorHead::Variant(variant) = &case.head else {
                    continue;
                };
                if !candidates.contains(variant) {
                    continue;
                }
                let field_summaries = case
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(index, child_subject)| {
                        fields
                            .get(index)
                            .map(|child| summarize_pattern(declarations, store, hierarchy, rigids, enum_table, arena, *child, child_subject))
                            .unwrap_or(PatternSpaceSummary::Opaque(child_subject.canonical))
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                summaries.push(PatternSpaceSummary::Variant {
                    variant: variant.clone(),
                    exact_case: case.exact_case.unwrap_or(subject.canonical),
                    fields: field_summaries,
                });
            }
            match summaries.len() {
                0 => PatternSpaceSummary::Opaque(subject.canonical),
                1 => summaries.into_iter().next().unwrap_or(PatternSpaceSummary::Opaque(subject.canonical)),
                _ => PatternSpaceSummary::Union(summaries.into_boxed_slice()),
            }
        }
        CoveragePattern::Tuple(fields) => {
            let cases = match decompose_domain(declarations, store, hierarchy, rigids, enum_table, subject) {
                DomainDecomposition::Closed(cases) => cases,
                DomainDecomposition::Empty => return PatternSpaceSummary::Empty,
                DomainDecomposition::Blocked(reason) => return PatternSpaceSummary::Blocked(reason),
                DomainDecomposition::Open => return PatternSpaceSummary::Opaque(subject.canonical),
            };
            let Some(case) = cases.first() else {
                return PatternSpaceSummary::Empty;
            };
            let summaries = case
                .fields
                .iter()
                .enumerate()
                .map(|(index, child_subject)| {
                    fields
                        .get(index)
                        .map(|child| summarize_pattern(declarations, store, hierarchy, rigids, enum_table, arena, *child, child_subject))
                        .unwrap_or(PatternSpaceSummary::Opaque(child_subject.canonical))
                })
                .collect();
            PatternSpaceSummary::Tuple(summaries)
        }
        CoveragePattern::List { .. } => PatternSpaceSummary::List,
        CoveragePattern::Or(alts) => {
            let summaries = alts
                .iter()
                .map(|&alt| summarize_pattern(declarations, store, hierarchy, rigids, enum_table, arena, alt, subject))
                .collect();
            PatternSpaceSummary::Union(summaries)
        }
        CoveragePattern::RecordPredicate | CoveragePattern::MapPredicate => PatternSpaceSummary::Opaque(subject.canonical),
    }
}

#[allow(clippy::too_many_arguments)]
fn useful_internal(
    declarations: &DeclarationTypeTable,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    rigids: &mut RigidArena,
    enum_table: Option<&EnumSemanticTable>,
    arena: &mut CoveragePatternArena,
    control: &CheckerControl,
    metrics: &mut CoverageMetrics,
    matrix: &[Vec<CoveragePatternId>],
    candidate: &[CoveragePatternId],
    subjects: &[CoverageSubject],
    proof: &BranchProofEnvironment,
) -> UsefulnessSearch {
    metrics.witness_states += 1;
    if control.is_cancelled() {
        return UsefulnessSearch::Blocked(BlockReason::UnknownType(UnknownReason::InferenceCancelled));
    }
    if let Err(report) = control.charge_step() {
        return UsefulnessSearch::Blocked(BlockReason::BudgetExceeded(report));
    }

    if subjects.is_empty() || candidate.is_empty() {
        if matrix.is_empty() {
            return UsefulnessSearch::Useful(Some(CoverageWitness::Wildcard));
        } else {
            return UsefulnessSearch::NotUseful;
        }
    }

    let head_pat_id = candidate[0];
    let head_subject = &subjects[0];
    let head_pat = arena.get(head_pat_id).clone();

    // Or-pattern in candidate: useful iff any alternative is useful.
    if let CoveragePattern::Or(alts) = &head_pat {
        for alt in alts.iter() {
            let mut new_cand = candidate.to_vec();
            new_cand[0] = *alt;
            match useful_internal(declarations, store, hierarchy, rigids, enum_table, arena, control, metrics, matrix, &new_cand, subjects, proof) {
                UsefulnessSearch::Useful(wit) => return UsefulnessSearch::Useful(wit),
                UsefulnessSearch::Blocked(b) => return UsefulnessSearch::Blocked(b),
                UsefulnessSearch::NotUseful => continue,
            }
        }
        return UsefulnessSearch::NotUseful;
    }

    // Predicates on open domains:
    if matches!(head_pat, CoveragePattern::RecordPredicate | CoveragePattern::MapPredicate) {
        let wildcard_prior: Vec<Vec<CoveragePatternId>> = matrix
            .iter()
            .filter(|r| !r.is_empty() && matches!(arena.get(r[0]), CoveragePattern::Wildcard))
            .map(|r| r[1..].to_vec())
            .collect();
        return useful_internal(
            declarations,
            store,
            hierarchy,
            rigids,
            enum_table,
            arena,
            control,
            metrics,
            &wildcard_prior,
            &candidate[1..],
            &subjects[1..],
            proof,
        );
    }

    // Candidate head is Wildcard:
    if matches!(head_pat, CoveragePattern::Wildcard) {
        let has_constructor = matrix.iter().any(|r| !r.is_empty() && !matches!(arena.get(r[0]), CoveragePattern::Wildcard));
        if !has_constructor {
            // No row in column 0 is a constructor: drop column 0 without domain decomposition!
            let sub_matrix: Vec<Vec<CoveragePatternId>> = matrix.iter().filter(|r| !r.is_empty()).map(|r| r[1..].to_vec()).collect();
            return useful_internal(
                declarations,
                store,
                hierarchy,
                rigids,
                enum_table,
                arena,
                control,
                metrics,
                &sub_matrix,
                &candidate[1..],
                &subjects[1..],
                proof,
            );
        }

        // At least one prior row inspected this column. Decompose domain:
        metrics.constructor_decompositions += 1;
        match decompose_domain(declarations, store, hierarchy, rigids, enum_table, head_subject) {
            DomainDecomposition::Empty => UsefulnessSearch::NotUseful,
            DomainDecomposition::Blocked(b) => UsefulnessSearch::Blocked(b),
            DomainDecomposition::Open => {
                let wildcard_matrix: Vec<Vec<CoveragePatternId>> = matrix
                    .iter()
                    .filter(|r| !r.is_empty() && matches!(arena.get(r[0]), CoveragePattern::Wildcard))
                    .map(|r| r[1..].to_vec())
                    .collect();
                useful_internal(
                    declarations,
                    store,
                    hierarchy,
                    rigids,
                enum_table,
                arena,
                control,
                metrics,
                &wildcard_matrix,
                    &candidate[1..],
                    &subjects[1..],
                    proof,
                )
            }
            DomainDecomposition::Closed(all_constructors) => {
                for case in all_constructors.iter() {
                    if let Err(reason) = charge_search_work(control) {
                        return UsefulnessSearch::Blocked(reason);
                    }
                    metrics.matrix_specializations += 1;
                    let spec_matrix = specialize_matrix_for_case(arena, matrix, case);
                    let mut spec_cand = vec![arena.wildcard(); case.fields.len()];
                    spec_cand.extend_from_slice(&candidate[1..]);

                    let mut spec_subj = case.fields.to_vec();
                    spec_subj.extend_from_slice(&subjects[1..]);

                    metrics.proof_merges += 1;
                    let merged_proof = match crate::checker::gadt_proof::merge_branch_proofs(store, proof, &case.proof) {
                        crate::checker::gadt_proof::ProofMerge::Compatible(p) => p,
                        crate::checker::gadt_proof::ProofMerge::Contradictory => continue,
                    };

                    match useful_internal(
                        declarations,
                        store,
                        hierarchy,
                        rigids,
                        enum_table,
                        arena,
                        control,
                        metrics,
                        &spec_matrix,
                        &spec_cand,
                        &spec_subj,
                        &merged_proof,
                    ) {
                        UsefulnessSearch::Useful(child_wit) => {
                            if let Err(reason) = charge_search_work(control) {
                                return UsefulnessSearch::Blocked(reason);
                            }
                            let witness = construct_case_witness(case, head_subject, child_wit);
                            return UsefulnessSearch::Useful(Some(witness));
                        }
                        UsefulnessSearch::Blocked(b) => return UsefulnessSearch::Blocked(b),
                        UsefulnessSearch::NotUseful => continue,
                    }
                }
                UsefulnessSearch::NotUseful
            }
        }
    } else {
        // Candidate head is a constructor (Variant, Tuple, List):
        metrics.constructor_decompositions += 1;
        match decompose_domain(declarations, store, hierarchy, rigids, enum_table, head_subject) {
            DomainDecomposition::Empty => UsefulnessSearch::NotUseful,
            DomainDecomposition::Blocked(b) => UsefulnessSearch::Blocked(b),
            DomainDecomposition::Open => {
                // Constructor on an open domain:
                UsefulnessSearch::Useful(Some(CoverageWitness::Opaque(head_subject.canonical)))
            }
            DomainDecomposition::Closed(all_constructors) => {
                for case in all_constructors.iter() {
                    if !case_matches_pattern(arena, case, &head_pat) {
                        continue;
                    }
                    if let Err(reason) = charge_search_work(control) {
                        return UsefulnessSearch::Blocked(reason);
                    }
                    metrics.matrix_specializations += 1;
                    let spec_matrix = specialize_matrix_for_case(arena, matrix, case);
                    let mut spec_cand = extract_constructor_fields(arena, &head_pat, case);
                    spec_cand.extend_from_slice(&candidate[1..]);

                    let mut spec_subj = case.fields.to_vec();
                    spec_subj.extend_from_slice(&subjects[1..]);

                    metrics.proof_merges += 1;
                    let merged_proof = match crate::checker::gadt_proof::merge_branch_proofs(store, proof, &case.proof) {
                        crate::checker::gadt_proof::ProofMerge::Compatible(p) => p,
                        crate::checker::gadt_proof::ProofMerge::Contradictory => continue,
                    };

                    match useful_internal(
                        declarations,
                        store,
                        hierarchy,
                        rigids,
                        enum_table,
                        arena,
                        control,
                        metrics,
                        &spec_matrix,
                        &spec_cand,
                        &spec_subj,
                        &merged_proof,
                    ) {
                        UsefulnessSearch::Useful(child_wit) => {
                            if let Err(reason) = charge_search_work(control) {
                                return UsefulnessSearch::Blocked(reason);
                            }
                            let witness = construct_case_witness(case, head_subject, child_wit);
                            return UsefulnessSearch::Useful(Some(witness));
                        }
                        UsefulnessSearch::Blocked(b) => return UsefulnessSearch::Blocked(b),
                        UsefulnessSearch::NotUseful => continue,
                    }
                }
                UsefulnessSearch::NotUseful
            }
        }
    }
}

fn charge_search_work(control: &CheckerControl) -> Result<(), BlockReason> {
    if control.is_cancelled() {
        return Err(BlockReason::UnknownType(UnknownReason::InferenceCancelled));
    }
    control.charge_step().map_err(BlockReason::BudgetExceeded)
}

fn case_matches_pattern(arena: &CoveragePatternArena, case: &ConstructorCase, pat: &CoveragePattern) -> bool {
    match (pat, &case.head) {
        (CoveragePattern::Wildcard, _) => true,
        (CoveragePattern::Variant { candidates, .. }, ConstructorHead::Variant(vid)) => candidates.contains(vid),
        (CoveragePattern::Tuple(fields), ConstructorHead::Tuple { arity }) => fields.len() == *arity,
        (CoveragePattern::List { prefix, rest }, ConstructorHead::ListNil) => prefix.is_empty() && rest.is_none(),
        (CoveragePattern::List { prefix, rest }, ConstructorHead::ListCons) => !prefix.is_empty() || rest.is_some(),
        (CoveragePattern::Or(alts), _) => alts.iter().any(|alt| case_matches_pattern(arena, case, arena.get(*alt))),
        _ => false,
    }
}

fn extract_constructor_fields(arena: &mut CoveragePatternArena, pat: &CoveragePattern, case: &ConstructorCase) -> Vec<CoveragePatternId> {
    match pat.clone() {
        CoveragePattern::Variant { fields, .. } => {
            let mut f = fields.to_vec();
            if f.len() < case.fields.len() {
                f.resize(case.fields.len(), arena.wildcard());
            } else if f.len() > case.fields.len() {
                f.truncate(case.fields.len());
            }
            f
        }
        CoveragePattern::Tuple(fields) => fields.to_vec(),
        CoveragePattern::List { prefix, rest } => {
            match &case.head {
                ConstructorHead::ListNil => Vec::new(),
                ConstructorHead::ListCons => {
                    let head = if !prefix.is_empty() { prefix[0] } else { arena.wildcard() };
                    let tail = if prefix.len() > 1 {
                        // remaining prefix
                        arena.alloc(CoveragePattern::List {
                            prefix: prefix[1..].to_vec().into_boxed_slice(),
                            rest,
                        })
                    } else if let Some(r) = rest {
                        r
                    } else {
                        arena.alloc(CoveragePattern::List {
                            prefix: Box::new([]),
                            rest: None,
                        })
                    };
                    vec![head, tail]
                }
                _ => vec![arena.wildcard(); case.fields.len()],
            }
        }
        CoveragePattern::Wildcard => vec![arena.wildcard(); case.fields.len()],
        CoveragePattern::Or(alts) => {
            for alt in alts.iter() {
                let alt_pat = arena.get(*alt).clone();
                if case_matches_pattern(arena, case, &alt_pat) {
                    return extract_constructor_fields(arena, &alt_pat, case);
                }
            }
            vec![arena.wildcard(); case.fields.len()]
        }
        _ => vec![arena.wildcard(); case.fields.len()],
    }
}

fn specialize_matrix_for_case(
    arena: &mut CoveragePatternArena,
    matrix: &[Vec<CoveragePatternId>],
    case: &ConstructorCase,
) -> Vec<Vec<CoveragePatternId>> {
    let mut specialized = Vec::with_capacity(matrix.len());
    for row in matrix {
        if row.is_empty() {
            continue;
        }
        let head_id = row[0];
        let head_pat = arena.get(head_id).clone();

        if let CoveragePattern::Or(alts) = &head_pat {
            for alt in alts.iter() {
                let alt_pat = arena.get(*alt).clone();
                if case_matches_pattern(arena, case, &alt_pat) {
                    let mut new_row = extract_constructor_fields(arena, &alt_pat, case);
                    new_row.extend_from_slice(&row[1..]);
                    specialized.push(new_row);
                }
            }
            continue;
        }

        if case_matches_pattern(arena, case, &head_pat) {
            let mut new_row = extract_constructor_fields(arena, &head_pat, case);
            new_row.extend_from_slice(&row[1..]);
            specialized.push(new_row);
        }
    }
    specialized
}

fn construct_case_witness(case: &ConstructorCase, subject: &CoverageSubject, child_wit: Option<CoverageWitness>) -> CoverageWitness {
    match &case.head {
        ConstructorHead::Variant(vid) => {
            let mut fields = Vec::with_capacity(case.fields.len());
            if let Some(wit) = child_wit {
                fields.push(wit);
                for f in case.fields.iter().skip(1) {
                    fields.push(CoverageWitness::Opaque(f.canonical));
                }
            } else {
                for f in case.fields.iter() {
                    fields.push(CoverageWitness::Opaque(f.canonical));
                }
            }
            CoverageWitness::Variant {
                variant: vid.clone(),
                exact_case: case.exact_case.unwrap_or(subject.canonical),
                fields: fields.into_boxed_slice(),
            }
        }
        ConstructorHead::Tuple { .. } => {
            let mut fields = Vec::with_capacity(case.fields.len());
            if let Some(wit) = child_wit {
                fields.push(wit);
                for f in case.fields.iter().skip(1) {
                    fields.push(CoverageWitness::Opaque(f.canonical));
                }
            } else {
                for f in case.fields.iter() {
                    fields.push(CoverageWitness::Opaque(f.canonical));
                }
            }
            CoverageWitness::Tuple(fields.into_boxed_slice())
        }
        ConstructorHead::ListNil => CoverageWitness::List(Box::new([])),
        ConstructorHead::ListCons => CoverageWitness::List(Box::new([CoverageWitness::Wildcard])),
    }
}
