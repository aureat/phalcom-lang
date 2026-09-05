//! Formal match and recursive pattern semantic product models (Part 05.1).

use crate::identity::{BindingId, DeclarationId, ExpressionId, VariantFamilyId, VariantFieldId, VariantId};
use crate::types::TypeConstraint;
use crate::types::evidence::TypeKnowledge;
use crate::types::id::{TypeId, TypeParameterId};
use crate::types::outcome::BlockReason;
use crate::types::rigid::{LocalConstraint, LocalType};
use phalcom_ast::ast::MapPatternKey;
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorPattern};
use std::collections::BTreeMap;

/// Index of match expression analysis products within a callable body.
pub type MatchResolutionIndex = BTreeMap<ExpressionId, MatchResolution>;

/// Comprehensive semantic analysis product for a `match` expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchResolution {
    pub expression: ExpressionId,
    pub scrutinee: TypeKnowledge,
    pub initial_space: PatternSpaceSummary,
    pub arms: Box<[MatchArmResolution]>,
    pub result: TypeKnowledge,
    pub exhaustiveness: ExhaustivenessResult,
}

impl MatchResolution {
    pub fn proof_for_arm(&self, index: usize) -> Option<&BranchProofEnvironment> {
        self.arms.get(index).map(|arm| &arm.proof)
    }
}

/// Semantic analysis product for a single `pattern => branch` match arm.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchArmResolution {
    pub arm_index: u32,
    pub pattern: PatternResolution,
    pub reachable_space: PatternSpaceSummary,
    pub residual_after: PatternSpaceSummary,
    pub bindings: Box<[PatternBindingResolution]>,
    pub proof: BranchProofEnvironment,
    pub usefulness: PatternUsefulness,
    pub branch_result: TypeKnowledge,
}

/// Semantic resolution of a recursive pattern node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatternResolution {
    Wildcard,
    Binding {
        binding: BindingId,
        name: Box<str>,
        knowledge: TypeKnowledge,
    },
    Variant(ResolvedVariantPattern),
    Or(ResolvedOrPattern),
    Tuple(Box<[PatternResolution]>),
    List(ResolvedListPattern),
    Record(Box<[ResolvedRecordFieldPattern]>),
    Map(Box<[ResolvedMapEntryPattern]>),
}

/// Variant pattern resolution containing declaration-backed owner evidence,
/// selector constraint, and exact candidates.
///
/// `owner` and `family` are present only when one canonical owner/family can be
/// established. Ambiguous contextual patterns retain every declaration-backed
/// owner in `owner_candidates` instead of selecting one from spelling order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedVariantPattern {
    pub owner: Option<DeclarationId>,
    pub family: Option<VariantFamilyId>,
    pub owner_candidates: Box<[DeclarationId]>,
    pub selector: VariantSelectorConstraint,
    pub candidates: Box<[ResolvedVariantCandidate]>,
}

/// Selector constraint shape on a variant pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VariantSelectorConstraint {
    Exact(Selector),
    Pattern(SelectorPattern),
    WholeFamily,
}

/// Single resolved exact variant candidate within a variant or family pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedVariantCandidate {
    pub variant: VariantId,
    pub exact_case: TypeId,
    pub fields: Box<[ResolvedFieldPattern]>,
    pub proof: BranchProofEnvironment,
    pub case_instantiation: Option<crate::types::CaseInstantiation>,
}

/// Projection of a candidate field to its specialized type and child pattern resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedFieldPattern {
    pub field: VariantFieldId,
    pub field_type: TypeKnowledge,
    pub local_type: Option<LocalType>,
    pub child: Box<PatternResolution>,
}

/// Resolution of an or-pattern node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedOrPattern {
    pub alternatives: Box<[PatternResolution]>,
}

/// Resolution of a list pattern node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedListPattern {
    pub prefix: Box<[PatternResolution]>,
    pub rest: Option<Box<PatternResolution>>,
}

/// Resolution of a record field pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedRecordFieldPattern {
    pub label: Box<str>,
    pub child: Box<PatternResolution>,
}

/// Resolution of a map entry pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedMapEntryPattern {
    pub key: MapPatternKey,
    pub child: Box<PatternResolution>,
}

/// GADT branch proof environment containing type parameter substitutions and established equalities.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BranchProofEnvironment {
    pub bindings: BTreeMap<TypeParameterId, TypeId>,
    pub equalities: Box<[TypeConstraint]>,
    pub local_bindings: BTreeMap<TypeParameterId, LocalType>,
    pub local_equalities: Box<[LocalConstraint]>,
}

impl BranchProofEnvironment {
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty() && self.equalities.is_empty() && self.local_bindings.is_empty() && self.local_equalities.is_empty()
    }
}

/// Resolved variable binding introduced by a pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatternBindingResolution {
    pub binding: BindingId,
    pub name: Box<str>,
    pub knowledge: TypeKnowledge,
    pub local_type: Option<LocalType>,
    pub source: SourceRange,
}

/// Usefulness classification for a match arm or or-pattern alternative.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PatternUsefulness {
    Useful,
    Redundant,
    Impossible,
}

/// Exhaustiveness proof result for a match expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExhaustivenessResult {
    Proven,
    Missing(Box<[CoverageWitness]>),
    Blocked(BlockReason),
}

/// Compact witness for an uncovered value space.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoverageWitness {
    Variant {
        variant: VariantId,
        exact_case: TypeId,
        fields: Box<[CoverageWitness]>,
    },
    Tuple(Box<[CoverageWitness]>),
    List(Box<[CoverageWitness]>),
    Opaque(TypeId),
    Wildcard,
}

/// Compact summary of a pattern value-space for diagnostics, tooling, and test assertions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatternSpaceSummary {
    Empty,
    /// Coverage could not safely determine this summary. This is distinct
    /// from `Opaque`, which is a known conservative value-space summary.
    Blocked(BlockReason),
    Opaque(TypeId),
    Union(Box<[PatternSpaceSummary]>),
    Variant {
        variant: VariantId,
        exact_case: TypeId,
        fields: Box<[PatternSpaceSummary]>,
    },
    Tuple(Box<[PatternSpaceSummary]>),
    List,
}
