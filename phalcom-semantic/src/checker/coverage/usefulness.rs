//! Finite demand-driven pattern-matrix usefulness and exhaustiveness engine.

use crate::declarations::DeclarationTypeTable;
use crate::enum_semantics::EnumSemanticTable;
use crate::match_semantics::{BranchProofEnvironment, CoverageWitness, ExhaustivenessResult, PatternSpaceSummary, PatternUsefulness};
use crate::types::outcome::BlockReason;
use crate::types::relation::TypeHierarchy;
use crate::types::rigid::RigidArena;
use crate::types::store::TypeStore;

use super::domain::{ConstructorCase, ConstructorHead, DomainDecomposition, decompose_domain};
use super::pattern::{CoveragePattern, CoveragePatternArena, CoveragePatternId};
use super::subject::CoverageSubject;

#[derive(Clone, Debug)]
pub(crate) enum UsefulnessSearch {
    Useful(Option<CoverageWitness>),
    NotUseful,
    Blocked(BlockReason),
}

/// Demand-driven pattern matrix coverage engine.
#[derive(Clone, Debug)]
pub(crate) struct CoverageEngine {
    root: CoverageSubject,
    arena: CoveragePatternArena,
    prior_matrix: Vec<Vec<CoveragePatternId>>,
    blocked: Option<BlockReason>,
}

impl CoverageEngine {
    pub(crate) fn new(root: CoverageSubject) -> Self {
        Self {
            root,
            arena: CoveragePatternArena::new(),
            prior_matrix: Vec::new(),
            blocked: None,
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

    pub(crate) fn classify_arm(
        &mut self,
        declarations: &DeclarationTypeTable,
        store: &mut TypeStore,
        hierarchy: &dyn TypeHierarchy,
        rigids: &mut RigidArena,
        enum_table: Option<&EnumSemanticTable>,
        pattern: CoveragePatternId,
    ) -> PatternUsefulness {
        // 1. Check if pattern has values in the domain (useful against empty matrix):
        let in_domain = useful_internal(
            declarations,
            store,
            hierarchy,
            rigids,
            enum_table,
            &mut self.arena,
            &[],
            &[pattern],
            std::slice::from_ref(&self.root),
            &BranchProofEnvironment::default(),
        );
        if matches!(in_domain, UsefulnessSearch::NotUseful) {
            return PatternUsefulness::Impossible;
        }

        // 2. Check if useful against prior matrix:
        let against_prior = useful_internal(
            declarations,
            store,
            hierarchy,
            rigids,
            enum_table,
            &mut self.arena,
            &self.prior_matrix,
            &[pattern],
            std::slice::from_ref(&self.root),
            &BranchProofEnvironment::default(),
        );
        match against_prior {
            UsefulnessSearch::Useful(_) => PatternUsefulness::Useful,
            UsefulnessSearch::NotUseful => PatternUsefulness::Redundant,
            UsefulnessSearch::Blocked(b) => {
                self.blocked = Some(b);
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
        // Check in domain:
        let in_domain = useful_internal(
            declarations,
            store,
            hierarchy,
            rigids,
            enum_table,
            &mut self.arena,
            &[],
            &[candidate],
            std::slice::from_ref(&self.root),
            &BranchProofEnvironment::default(),
        );
        if matches!(in_domain, UsefulnessSearch::NotUseful) {
            return PatternUsefulness::Impossible;
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
            &prior_matrix,
            &[candidate],
            std::slice::from_ref(&self.root),
            &BranchProofEnvironment::default(),
        );
        match against_prior {
            UsefulnessSearch::Useful(_) => PatternUsefulness::Useful,
            UsefulnessSearch::NotUseful => PatternUsefulness::Redundant,
            UsefulnessSearch::Blocked(b) => {
                self.blocked = Some(b);
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

        if self.prior_matrix.is_empty() {
            match decompose_domain(declarations, store, hierarchy, rigids, enum_table, &self.root) {
                DomainDecomposition::Empty => return ExhaustivenessResult::Proven,
                DomainDecomposition::Blocked(b) => return ExhaustivenessResult::Blocked(b),
                DomainDecomposition::Closed(cases) => {
                    let witnesses: Vec<CoverageWitness> = cases
                        .iter()
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
}

pub(crate) fn summarize_pattern(
    arena: &CoveragePatternArena,
    pattern: CoveragePatternId,
    subject: &CoverageSubject,
) -> PatternSpaceSummary {
    match arena.get(pattern) {
        CoveragePattern::Wildcard => PatternSpaceSummary::Opaque(subject.canonical),
        CoveragePattern::Variant { candidates, exact_cases, fields } => {
            let field_summaries: Box<[PatternSpaceSummary]> = fields
                .iter()
                .map(|&f| summarize_pattern(arena, f, subject))
                .collect::<Vec<_>>()
                .into_boxed_slice();
            if candidates.len() == 1 {
                PatternSpaceSummary::Variant {
                    variant: candidates[0].clone(),
                    exact_case: exact_cases.first().copied().unwrap_or(subject.canonical),
                    fields: field_summaries,
                }
            } else {
                let summaries = candidates
                    .iter()
                    .zip(exact_cases.iter())
                    .map(|(v, e)| PatternSpaceSummary::Variant {
                        variant: v.clone(),
                        exact_case: *e,
                        fields: field_summaries.clone(),
                    })
                    .collect::<Vec<_>>();
                PatternSpaceSummary::Union(summaries.into_boxed_slice())
            }
        }
        CoveragePattern::Tuple(fields) => {
            let summaries = fields.iter().map(|&f| summarize_pattern(arena, f, subject)).collect();
            PatternSpaceSummary::Tuple(summaries)
        }
        CoveragePattern::List { .. } => PatternSpaceSummary::List,
        CoveragePattern::Or(alts) => {
            let summaries = alts.iter().map(|&alt| summarize_pattern(arena, alt, subject)).collect();
            PatternSpaceSummary::Union(summaries)
        }
        CoveragePattern::RecordPredicate | CoveragePattern::MapPredicate => {
            PatternSpaceSummary::Opaque(subject.canonical)
        }
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
    matrix: &[Vec<CoveragePatternId>],
    candidate: &[CoveragePatternId],
    subjects: &[CoverageSubject],
    proof: &BranchProofEnvironment,
) -> UsefulnessSearch {
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
            match useful_internal(declarations, store, hierarchy, rigids, enum_table, arena, matrix, &new_cand, subjects, proof) {
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
                &sub_matrix,
                &candidate[1..],
                &subjects[1..],
                proof,
            );
        }

        // At least one prior row inspected this column. Decompose domain:
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
                    &wildcard_matrix,
                    &candidate[1..],
                    &subjects[1..],
                    proof,
                )
            }
            DomainDecomposition::Closed(all_constructors) => {
                for case in all_constructors.iter() {
                    let spec_matrix = specialize_matrix_for_case(arena, matrix, case);
                    let mut spec_cand = vec![arena.wildcard(); case.fields.len()];
                    spec_cand.extend_from_slice(&candidate[1..]);

                    let mut spec_subj = case.fields.to_vec();
                    spec_subj.extend_from_slice(&subjects[1..]);

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
                        &spec_matrix,
                        &spec_cand,
                        &spec_subj,
                        &merged_proof,
                    ) {
                        UsefulnessSearch::Useful(child_wit) => {
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
                    let spec_matrix = specialize_matrix_for_case(arena, matrix, case);
                    let mut spec_cand = extract_constructor_fields(arena, &head_pat, case);
                    spec_cand.extend_from_slice(&candidate[1..]);

                    let mut spec_subj = case.fields.to_vec();
                    spec_subj.extend_from_slice(&subjects[1..]);

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
                        &spec_matrix,
                        &spec_cand,
                        &spec_subj,
                        &merged_proof,
                    ) {
                        UsefulnessSearch::Useful(child_wit) => {
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
