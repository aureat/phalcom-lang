//! Formal match and recursive pattern semantic product models (Part 05.1).

use crate::checker::analysis_map::AnalysisMap;
use crate::field_key::FieldKey;
use crate::identity::{BindingId, DeclarationId, ExpressionId, VariantFamilyId, VariantFieldId, VariantId};
use crate::types::TypeConstraint;
use crate::types::evidence::TypeKnowledge;
use crate::types::id::{RigidScopeId, RigidTypeVariableId};
use crate::types::id::{TypeId, TypeParameterId};
use crate::types::outcome::BlockReason;
use crate::types::rigid::{LocalConstraint, LocalType};
use phalcom_ast::ast::MapPatternKey;
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorPattern};

/// Index of match expression analysis products within a callable body.
pub type MatchResolutionIndex<R = RigidTypeVariableId, S = RigidScopeId, K = FieldKey> = AnalysisMap<ExpressionId, MatchResolution<R, S, K>>;

/// Kind of executable pattern site outside `match`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PatternSiteKind {
    IfLet,
    WhileLet,
    ForLane,
}

/// Disambiguated key for a pattern execution site within a callable body.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PatternSiteKey {
    pub kind: PatternSiteKind,
    pub range: SourceRange,
}

impl PatternSiteKey {
    pub fn new(kind: PatternSiteKind, range: SourceRange) -> Self {
        Self { kind, range }
    }
}

/// Semantic analysis product for an executable pattern site outside `match`
/// (`if let`, `while let`, `for` lane).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatternSiteResolution<R = RigidTypeVariableId, S = RigidScopeId, K = FieldKey> {
    pub kind: PatternSiteKind,
    pub range: SourceRange,
    pub subject: TypeKnowledge,
    pub pattern: PatternResolution<R, S, K>,
    pub bindings: Box<[PatternBindingResolution<R, K>]>,
    pub usefulness: PatternUsefulness,
}

/// Index of non-match executable pattern site resolutions within a callable body.
pub type PatternSiteResolutionIndex<R = RigidTypeVariableId, S = RigidScopeId, K = FieldKey> = AnalysisMap<PatternSiteKey, PatternSiteResolution<R, S, K>>;

/// Comprehensive semantic analysis product for a `match` expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchResolution<R = RigidTypeVariableId, S = RigidScopeId, K = FieldKey> {
    pub expression: ExpressionId,
    pub scrutinee: TypeKnowledge,
    pub initial_space: PatternSpaceSummary,
    pub arms: Box<[MatchArmResolution<R, S, K>]>,
    pub result: TypeKnowledge,
    pub exhaustiveness: ExhaustivenessResult,
}

impl<R, S, K> MatchResolution<R, S, K> {
    pub fn proof_for_arm(&self, index: usize) -> Option<&BranchProofEnvironment<R, K>> {
        self.arms.get(index).map(|arm| &arm.proof)
    }
}

/// Semantic analysis product for a single `pattern => branch` match arm.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchArmResolution<R = RigidTypeVariableId, S = RigidScopeId, K = FieldKey> {
    pub arm_index: u32,
    pub pattern: PatternResolution<R, S, K>,
    pub reachable_space: PatternSpaceSummary,
    pub residual_after: PatternSpaceSummary,
    pub bindings: Box<[PatternBindingResolution<R, K>]>,
    pub proof: BranchProofEnvironment<R, K>,
    pub usefulness: PatternUsefulness,
    pub branch_result: TypeKnowledge,
}

/// Semantic resolution of a recursive pattern node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatternResolution<R = RigidTypeVariableId, S = RigidScopeId, K = FieldKey> {
    Wildcard,
    Char(char),
    Binding {
        binding: BindingId,
        name: Box<str>,
        knowledge: TypeKnowledge,
    },
    Variant(ResolvedVariantPattern<R, S, K>),
    Or(ResolvedOrPattern<R, S, K>),
    Tuple(Box<[PatternResolution<R, S, K>]>),
    List(ResolvedListPattern<R, S, K>),
    Record(Box<[ResolvedRecordFieldPattern<R, S, K>]>),
    Map(Box<[ResolvedMapEntryPattern<R, S, K>]>),
}

/// Variant pattern resolution containing declaration-backed owner evidence,
/// selector constraint, and exact candidates.
///
/// `owner` and `family` are present only when one canonical owner/family can be
/// established. Ambiguous contextual patterns retain every declaration-backed
/// owner in `owner_candidates` instead of selecting one from spelling order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedVariantPattern<R = RigidTypeVariableId, S = RigidScopeId, K = FieldKey> {
    pub owner: Option<DeclarationId>,
    pub family: Option<VariantFamilyId>,
    pub owner_candidates: Box<[DeclarationId]>,
    pub selector: VariantSelectorConstraint,
    pub candidates: Box<[ResolvedVariantCandidate<R, S, K>]>,
    /// A potentially applicable constructor could not be classified. Its absence
    /// from `candidates` must not be interpreted as formal refutation.
    pub blocked: Option<BlockReason>,
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
pub struct ResolvedVariantCandidate<R = RigidTypeVariableId, S = RigidScopeId, K = FieldKey> {
    pub variant: VariantId,
    pub exact_case: TypeId,
    pub fields: Box<[ResolvedFieldPattern<R, S, K>]>,
    pub proof: BranchProofEnvironment<R, K>,
    pub case_instantiation: Option<crate::types::CaseInstantiation<R, S, K>>,
}

/// Projection of a candidate field to its specialized type and child pattern resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedFieldPattern<R = RigidTypeVariableId, S = RigidScopeId, K = FieldKey> {
    pub field: VariantFieldId,
    pub field_type: TypeKnowledge,
    pub local_type: Option<LocalType<R, K>>,
    pub child: Box<PatternResolution<R, S, K>>,
}

/// Resolution of an or-pattern node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedOrPattern<R = RigidTypeVariableId, S = RigidScopeId, K = FieldKey> {
    pub alternatives: Box<[PatternResolution<R, S, K>]>,
}

/// Resolution of a list pattern node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedListPattern<R = RigidTypeVariableId, S = RigidScopeId, K = FieldKey> {
    pub prefix: Box<[PatternResolution<R, S, K>]>,
    pub rest: Option<Box<PatternResolution<R, S, K>>>,
}

/// Resolution of a record field pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedRecordFieldPattern<R = RigidTypeVariableId, S = RigidScopeId, K = FieldKey> {
    pub label: Box<str>,
    pub child: Box<PatternResolution<R, S, K>>,
}

/// Resolution of a map entry pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedMapEntryPattern<R = RigidTypeVariableId, S = RigidScopeId, K = FieldKey> {
    pub key: MapPatternKey,
    pub child: Box<PatternResolution<R, S, K>>,
}

/// GADT branch proof environment containing type parameter substitutions and established equalities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchProofEnvironment<R = RigidTypeVariableId, K = FieldKey> {
    pub bindings: AnalysisMap<TypeParameterId, TypeId>,
    pub equalities: Box<[TypeConstraint]>,
    pub local_bindings: AnalysisMap<TypeParameterId, LocalType<R, K>>,
    pub local_equalities: Box<[LocalConstraint<R, K>]>,
}

impl<R, K> Default for BranchProofEnvironment<R, K> {
    fn default() -> Self {
        Self {
            bindings: AnalysisMap::new(),
            equalities: Box::new([]),
            local_bindings: AnalysisMap::new(),
            local_equalities: Box::new([]),
        }
    }
}

impl<R, K> BranchProofEnvironment<R, K> {
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty() && self.equalities.is_empty() && self.local_bindings.is_empty() && self.local_equalities.is_empty()
    }
}

/// Resolved variable binding introduced by a pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatternBindingResolution<R = RigidTypeVariableId, K = FieldKey> {
    pub binding: BindingId,
    pub name: Box<str>,
    pub knowledge: TypeKnowledge,
    pub local_type: Option<LocalType<R, K>>,
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
    Char(char),
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
    Char(char),
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
