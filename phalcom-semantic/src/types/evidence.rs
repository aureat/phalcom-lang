//! Formal type knowledge, epistemic status/origin, and bounded provenance.

use super::id::TypeId;
use phalcom_common::range::SourceRange;

/// Epistemic degree of knowledge established for a value or expression.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeKnowledge {
    /// Semantic engine has established a concrete type with supporting evidence.
    Known(TypeEvidence),
    /// Semantic engine cannot establish a type with current evidence.
    Unknown(UnknownReason),
    /// Deliberate static dynamic escape (e.g. `Dynamic` annotation or runtime rest dispatch).
    Dynamic(DynamicReason),
}

impl TypeKnowledge {
    #[inline]
    pub fn is_known(&self) -> bool {
        matches!(self, Self::Known(_))
    }

    #[inline]
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }

    #[inline]
    pub fn is_dynamic(&self) -> bool {
        matches!(self, Self::Dynamic(_))
    }

    #[inline]
    pub fn ty(&self) -> Option<TypeId> {
        match self {
            Self::Known(e) => Some(e.ty),
            _ => None,
        }
    }

    /// Returns epistemic strength for concrete formal knowledge.
    #[inline]
    pub fn status(&self) -> Option<EvidenceStatus> {
        match self {
            Self::Known(e) => Some(e.status),
            _ => None,
        }
    }

    /// Returns the primary derivation origin for concrete formal knowledge.
    #[inline]
    pub fn origin(&self) -> Option<EvidenceOrigin> {
        match self {
            Self::Known(e) => Some(e.origin),
            _ => None,
        }
    }

    pub(crate) fn derive_known_type(&self, ty: TypeId, origin: EvidenceOrigin) -> Self {
        match self {
            Self::Known(evidence) => Self::Known(TypeEvidence {
                ty,
                status: evidence.status,
                origin,
                provenance: evidence.provenance.clone(),
            }),
            other => other.clone(),
        }
    }

    #[inline]
    pub fn is_established(&self) -> bool {
        self.status() == Some(EvidenceStatus::Established)
    }

    #[inline]
    pub fn is_assumed(&self) -> bool {
        self.status() == Some(EvidenceStatus::Assumed)
    }

    /// Constructs formal knowledge established by a semantic derivation.
    pub fn established(ty: impl Into<TypeId>, origin: EvidenceOrigin) -> Self {
        Self::Known(TypeEvidence {
            ty: ty.into(),
            status: EvidenceStatus::Established,
            origin,
            provenance: EvidenceSet::default(),
        })
    }

    /// Constructs formal knowledge usable through an explicit static contract.
    pub fn assumed(ty: impl Into<TypeId>, origin: EvidenceOrigin) -> Self {
        Self::Known(TypeEvidence {
            ty: ty.into(),
            status: EvidenceStatus::Assumed,
            origin,
            provenance: EvidenceSet::default(),
        })
    }

    /// Applies a canonical type transformation without changing epistemic facts.
    pub fn map_type(&self, transform: impl FnOnce(TypeId) -> TypeId) -> Self {
        match self {
            Self::Known(evidence) => Self::Known(TypeEvidence {
                ty: transform(evidence.ty),
                status: evidence.status,
                origin: evidence.origin,
                provenance: evidence.provenance.clone(),
            }),
            Self::Unknown(reason) => Self::Unknown(reason.clone()),
            Self::Dynamic(reason) => Self::Dynamic(reason.clone()),
        }
    }

    pub fn proper_ty(&self, store: &super::store::TypeStore) -> Option<super::id::ProperTypeId> {
        self.ty().and_then(|id| store.proper_type(id).ok())
    }

    pub fn with_range(mut self, range: SourceRange) -> Self {
        if let Self::Known(ref mut ev) = self {
            ev.provenance.ranges.push(range);
        }
        self
    }
}

/// Joins reachable formal knowledge without laundering missing evidence into a
/// concrete type.
///
/// `Unknown` is absorbing for a reachable path. `Dynamic` is absorbing only
/// when every reachable path has at least some formal knowledge. A multi-path
/// known result is established only when every contributing fact is
/// established; otherwise it remains an explicit assumption.
pub fn join_type_knowledge(store: &mut super::store::TypeStore, inputs: impl IntoIterator<Item = TypeKnowledge>) -> TypeKnowledge {
    let inputs = inputs.into_iter().collect::<Vec<_>>();
    if let Some(reason) = inputs
        .iter()
        .filter_map(|knowledge| match knowledge {
            TypeKnowledge::Unknown(reason) => Some(reason.clone()),
            _ => None,
        })
        .reduce(join_unknown_reason)
    {
        return TypeKnowledge::Unknown(reason);
    }
    if let Some(reason) = inputs
        .iter()
        .filter_map(|knowledge| match knowledge {
            TypeKnowledge::Dynamic(reason) => Some(reason.clone()),
            _ => None,
        })
        .reduce(join_dynamic_reason)
    {
        return TypeKnowledge::Dynamic(reason);
    }

    let known = inputs
        .into_iter()
        .filter_map(|knowledge| match knowledge {
            TypeKnowledge::Known(evidence) => Some(evidence),
            TypeKnowledge::Unknown(_) | TypeKnowledge::Dynamic(_) => None,
        })
        .collect::<Vec<_>>();
    let Some(first) = known.first() else {
        return TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence);
    };
    if known.len() == 1 {
        return TypeKnowledge::Known(first.clone());
    }

    let types = known.iter().map(|evidence| evidence.ty).collect::<Vec<_>>();
    let joined_type = store.union(&types);
    let status = if known.iter().all(|evidence| evidence.status == EvidenceStatus::Established) {
        EvidenceStatus::Established
    } else {
        EvidenceStatus::Assumed
    };
    let mut joined = match status {
        EvidenceStatus::Established => TypeKnowledge::established(joined_type, EvidenceOrigin::Flow),
        EvidenceStatus::Assumed => TypeKnowledge::assumed(joined_type, EvidenceOrigin::Flow),
    };
    if let TypeKnowledge::Known(evidence) = &mut joined {
        for input in known {
            evidence.provenance.ranges.extend(input.provenance.ranges);
            evidence.provenance.descriptions.extend(input.provenance.descriptions);
        }
    }
    joined
}

/// Merges epistemic reasons with a stable, commutative precedence. More
/// specific solver/control-flow failures outrank ordinary missing-name
/// evidence, so predecessor iteration order cannot change the published fact.
fn join_unknown_reason(left: UnknownReason, right: UnknownReason) -> UnknownReason {
    let left_rank = unknown_reason_rank(&left);
    let right_rank = unknown_reason_rank(&right);
    if left_rank != right_rank {
        return if left_rank > right_rank { left } else { right };
    }
    let left_key = format!("{left:?}");
    let right_key = format!("{right:?}");
    if left_key >= right_key { left } else { right }
}

fn unknown_reason_rank(reason: &UnknownReason) -> u8 {
    match reason {
        UnknownReason::InferenceCancelled => 100,
        UnknownReason::InferenceBudgetExceeded => 99,
        UnknownReason::InferenceConflict => 98,
        UnknownReason::InferenceBlocked => 97,
        UnknownReason::UnderconstrainedTypeVariable => 96,
        UnknownReason::RecursiveFixpoint => 95,
        UnknownReason::SuppressedByInvalidCause => 94,
        UnknownReason::SyntaxError => 90,
        UnknownReason::UnresolvedName(_) => 80,
        UnknownReason::DynamicMessageSend => 70,
        UnknownReason::OpaqueNative => 60,
        UnknownReason::UncheckedExpression => 50,
        UnknownReason::MissingInitializer => 40,
        UnknownReason::UnannotatedDeclaration => 30,
        UnknownReason::NoTypeEvidence => 20,
    }
}

fn join_dynamic_reason(left: DynamicReason, right: DynamicReason) -> DynamicReason {
    let left_key = format!("{left:?}");
    let right_key = format!("{right:?}");
    if left_key >= right_key { left } else { right }
}

/// A concrete formal type accompanied by epistemic status, origin, and provenance.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeEvidence {
    ty: TypeId,
    status: EvidenceStatus,
    origin: EvidenceOrigin,
    provenance: EvidenceSet,
}

impl TypeEvidence {
    pub fn ty(&self) -> TypeId {
        self.ty
    }

    pub fn status(&self) -> EvidenceStatus {
        self.status
    }

    pub fn origin(&self) -> EvidenceOrigin {
        self.origin
    }

    pub fn provenance(&self) -> &EvidenceSet {
        &self.provenance
    }
}

/// Whether a concrete type is established by formal evidence or usable only as
/// an explicit static assumption.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EvidenceStatus {
    Established,
    Assumed,
}

/// Primary semantic origin of formal type knowledge.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EvidenceOrigin {
    Syntax,
    DeclarationSemantics,
    ConstructorSemantics,
    CallableSignature,
    NativeSignature,
    DeveloperAnnotation,
    GenericInference,
    Flow,
    ContextualDerivation,
    PatternDecomposition,
}

/// Provenance facts explaining where type evidence was synthesized or constrained.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct EvidenceSet {
    pub ranges: Vec<SourceRange>,
    pub descriptions: Vec<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum UnknownReason {
    /// No value evidence exists, and a valid binding contract may supply a
    /// usable static assumption.
    NoTypeEvidence,
    MissingInitializer,
    UnannotatedDeclaration,
    UnresolvedName(Box<str>),
    DynamicMessageSend,
    OpaqueNative,
    RecursiveFixpoint,
    UncheckedExpression,
    SyntaxError,
    UnderconstrainedTypeVariable,
    InferenceConflict,
    InferenceBlocked,
    InferenceCancelled,
    InferenceBudgetExceeded,
    SuppressedByInvalidCause,
}

/// Whether a binding contract may supply usable current knowledge for an
/// unknown value without laundering a checker failure into an assumption.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractAssumptionEligibility {
    MaySupplyAssumption,
    MustRemainUnknown,
}

impl UnknownReason {
    pub fn contract_assumption_eligibility(&self) -> ContractAssumptionEligibility {
        match self {
            Self::NoTypeEvidence => ContractAssumptionEligibility::MaySupplyAssumption,
            _ => ContractAssumptionEligibility::MustRemainUnknown,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DynamicReason {
    ExplicitEscape,
    DynamicRestPack,
    RuntimeReflection,
}
