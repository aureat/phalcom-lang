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

    pub(crate) fn with_status_and_origin(self, status: EvidenceStatus, origin: EvidenceOrigin, range: SourceRange) -> Self {
        match self {
            Self::Known(evidence) => Self::Known(TypeEvidence {
                ty: evidence.ty,
                status,
                origin,
                provenance: evidence.provenance,
            })
            .with_range(range),
            other => other,
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
    if let Some(reason) = joined_unknown_reason(&inputs) {
        return TypeKnowledge::Unknown(reason);
    }
    if let Some(reason) = joined_dynamic_reason(&inputs) {
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
    let status = known
        .iter()
        .map(|evidence| evidence.status)
        .fold(EvidenceStatus::Established, EvidenceStatus::meet);
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

fn joined_unknown_reason(inputs: &[TypeKnowledge]) -> Option<UnknownReason> {
    inputs
        .iter()
        .filter_map(|knowledge| match knowledge {
            TypeKnowledge::Unknown(reason) => Some(reason.clone()),
            _ => None,
        })
        .reduce(join_unknown_reason)
}

fn joined_dynamic_reason(inputs: &[TypeKnowledge]) -> Option<DynamicReason> {
    inputs
        .iter()
        .filter_map(|knowledge| match knowledge {
            TypeKnowledge::Dynamic(reason) => Some(reason.clone()),
            _ => None,
        })
        .reduce(join_dynamic_reason)
}

/// Composes required type premises without dropping unavailable evidence.
pub(crate) fn compose_required_knowledge(
    inputs: impl IntoIterator<Item = TypeKnowledge>,
    origin: EvidenceOrigin,
    build_type: impl FnOnce(&[TypeId]) -> Result<TypeId, UnknownReason>,
) -> TypeKnowledge {
    let inputs = inputs.into_iter().collect::<Vec<_>>();

    if let Some(reason) = joined_unknown_reason(&inputs) {
        return TypeKnowledge::Unknown(reason);
    }
    if let Some(reason) = joined_dynamic_reason(&inputs) {
        return TypeKnowledge::Dynamic(reason);
    }

    let evidence = inputs
        .into_iter()
        .map(|knowledge| match knowledge {
            TypeKnowledge::Known(evidence) => evidence,
            TypeKnowledge::Unknown(_) | TypeKnowledge::Dynamic(_) => {
                unreachable!("Unknown/Dynamic handled before known composition")
            }
        })
        .collect::<Vec<_>>();
    let types = evidence.iter().map(TypeEvidence::ty).collect::<Vec<_>>();

    let ty = match build_type(&types) {
        Ok(ty) => ty,
        Err(reason) => return TypeKnowledge::Unknown(reason),
    };
    let status = evidence
        .iter()
        .map(TypeEvidence::status)
        .fold(EvidenceStatus::Established, EvidenceStatus::meet);
    let mut result = match status {
        EvidenceStatus::Established => TypeKnowledge::established(ty, origin),
        EvidenceStatus::Assumed => TypeKnowledge::assumed(ty, origin),
    };
    if let TypeKnowledge::Known(result_evidence) = &mut result {
        for input in evidence {
            result_evidence.provenance.ranges.extend(input.provenance.ranges);
            result_evidence.provenance.descriptions.extend(input.provenance.descriptions);
        }
    }
    result
}

/// Merges epistemic reasons with a stable, commutative precedence. More
/// specific solver/control-flow failures outrank ordinary missing-name
/// evidence, so predecessor iteration order cannot change the published fact.
pub(crate) fn join_unknown_reason(left: UnknownReason, right: UnknownReason) -> UnknownReason {
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
        UnknownReason::InferenceAmbiguous => 97,
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

pub(crate) fn join_dynamic_reason(left: DynamicReason, right: DynamicReason) -> DynamicReason {
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

impl EvidenceStatus {
    pub const fn meet(self, other: Self) -> Self {
        match (self, other) {
            (Self::Established, Self::Established) => Self::Established,
            _ => Self::Assumed,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Established => "established",
            Self::Assumed => "assumed",
        }
    }
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
    FieldLifecycle,
    ContextualDerivation,
    PatternDecomposition,
}

impl EvidenceOrigin {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Syntax => "syntax",
            Self::DeclarationSemantics => "declaration_semantics",
            Self::ConstructorSemantics => "constructor_semantics",
            Self::CallableSignature => "callable_signature",
            Self::NativeSignature => "native_signature",
            Self::DeveloperAnnotation => "developer_annotation",
            Self::GenericInference => "generic_inference",
            Self::Flow => "flow",
            Self::FieldLifecycle => "field_lifecycle",
            Self::ContextualDerivation => "contextual_derivation",
            Self::PatternDecomposition => "pattern_decomposition",
        }
    }
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
    InferenceAmbiguous,
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

#[cfg(test)]
mod required_composition_tests {
    use super::*;
    use crate::identity::DeclarationId;
    use crate::types::store::TypeStore;
    use phalcom_common::range::SourceRange;
    use phalcom_modules::identity::ModuleId;

    fn nominal(store: &mut TypeStore, name: &str) -> TypeId {
        store.nominal(DeclarationId::new(ModuleId::universe_root(), name.into()))
    }

    #[test]
    fn required_composition_is_established_only_from_established_inputs() {
        let mut store = TypeStore::new();
        let int_ty = nominal(&mut store, "Int");
        let string_ty = nominal(&mut store, "String");
        let result = compose_required_knowledge(
            [
                TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
                TypeKnowledge::established(string_ty, EvidenceOrigin::Syntax),
            ],
            EvidenceOrigin::Syntax,
            |types| Ok(store.union(types)),
        );
        assert_eq!(result.status(), Some(EvidenceStatus::Established));
        assert!(result.ty().is_some());
    }

    #[test]
    fn required_composition_weakens_when_any_input_is_assumed() {
        let mut store = TypeStore::new();
        let int_ty = nominal(&mut store, "Int");
        let result = compose_required_knowledge(
            [
                TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
                TypeKnowledge::assumed(int_ty, EvidenceOrigin::DeveloperAnnotation),
            ],
            EvidenceOrigin::Syntax,
            |types| Ok(store.union(types)),
        );
        assert_eq!(result.ty(), Some(int_ty));
        assert_eq!(result.status(), Some(EvidenceStatus::Assumed));
        assert_eq!(result.origin(), Some(EvidenceOrigin::Syntax));
    }

    #[test]
    fn required_composition_unknown_is_absorbing() {
        let mut store = TypeStore::new();
        let int_ty = nominal(&mut store, "Int");
        let unknown = UnknownReason::UnresolvedName("missing".into());
        let result = compose_required_knowledge(
            [
                TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
                TypeKnowledge::Unknown(unknown.clone()),
            ],
            EvidenceOrigin::Syntax,
            |_| panic!("builder must not run when a required input is Unknown"),
        );
        assert_eq!(result, TypeKnowledge::Unknown(unknown));
    }

    #[test]
    fn required_composition_dynamic_is_absorbing_after_unknown_check() {
        let mut store = TypeStore::new();
        let int_ty = nominal(&mut store, "Int");
        let reason = DynamicReason::ExplicitEscape;
        let result = compose_required_knowledge(
            [
                TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
                TypeKnowledge::Dynamic(reason.clone()),
            ],
            EvidenceOrigin::Syntax,
            |_| panic!("builder must not run when a required input is Dynamic"),
        );
        assert_eq!(result, TypeKnowledge::Dynamic(reason));
    }

    #[test]
    fn required_composition_preserves_component_provenance() {
        let mut store = TypeStore::new();
        let int_ty = nominal(&mut store, "Int");
        let left = TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax).with_range(SourceRange::default());
        let right = TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax).with_range(SourceRange::default());
        let result = compose_required_knowledge([left, right], EvidenceOrigin::Syntax, |types| Ok(store.union(types)));
        let TypeKnowledge::Known(evidence) = result else {
            panic!("expected known required composition");
        };
        assert_eq!(evidence.provenance().ranges.len(), 2);
    }

    #[test]
    fn evidence_status_meet_is_weakest_support() {
        use EvidenceStatus::{Assumed, Established};
        assert_eq!(Established.meet(Established), Established);
        assert_eq!(Established.meet(Assumed), Assumed);
        assert_eq!(Assumed.meet(Established), Assumed);
        assert_eq!(Assumed.meet(Assumed), Assumed);
    }

    #[test]
    fn evidence_status_meet_is_commutative_and_idempotent() {
        use EvidenceStatus::{Assumed, Established};
        for left in [Established, Assumed] {
            for right in [Established, Assumed] {
                assert_eq!(left.meet(right), right.meet(left));
            }
            assert_eq!(left.meet(left), left);
        }
    }
}
