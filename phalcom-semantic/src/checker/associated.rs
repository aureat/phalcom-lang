//! Associated resolution and family application models (Part 3).

use crate::associated::AssociatedMemberId;
use crate::identity::{AssociatedFamilyId, DeclarationId, ExpressionId, InvocationTargetId};
use crate::types::family::FamilyOperationShape;
use crate::types::id::TypeId;
use std::collections::BTreeMap;

/// A specialized member belonging to an associated family or lookup outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpecializedAssociatedMember {
    pub member: AssociatedMemberId,
    pub operation: FamilyOperationShape,
    pub value_type: TypeId,
    pub target: Option<InvocationTargetId>,
}

/// Resolved semantic outcome for an associated lookup or direct invocation expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssociatedResolution {
    pub owner_form: TypeId,
    pub lookup_owner: DeclarationId,
    pub family: AssociatedFamilyId,
    pub kind: AssociatedResolutionKind,
}

/// Specific variant of an associated resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssociatedResolutionKind {
    ExactValue {
        member: AssociatedMemberId,
        value_type: TypeId,
    },
    ExactCallable {
        member: AssociatedMemberId,
        target: InvocationTargetId,
        callable_type: TypeId,
    },
    Family {
        family_type: TypeId,
        members: Box<[SpecializedAssociatedMember]>,
    },
    StaticInvoke {
        member: AssociatedMemberId,
        target: InvocationTargetId,
        result_type: TypeId,
    },
    DynamicInvoke {
        candidates: Box<[SpecializedAssociatedMember]>,
        result_type: Option<TypeId>,
    },
}

/// Semantic resolution product for an ordinary invocation on a first-class family value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FamilyApplicationResolution {
    pub family_type: TypeId,
    pub selection: FamilyApplicationSelection,
}

/// Specific member selection for an ordinary family-value invocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FamilyApplicationSelection {
    Static {
        operation: FamilyOperationShape,
        target: Option<InvocationTargetId>,
        callable_type: TypeId,
        result_type: TypeId,
    },
    Dynamic {
        candidates: Box<[FamilyApplicationCandidate]>,
        result_type: Option<TypeId>,
    },
}

/// Candidate operation/member for deferred/dynamic shape selection on a family value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FamilyApplicationCandidate {
    pub operation: FamilyOperationShape,
    pub target: Option<InvocationTargetId>,
    pub callable_type: TypeId,
}

/// Body-local index of associated syntax resolutions.
pub type AssociatedResolutionIndex = BTreeMap<ExpressionId, AssociatedResolution>;

/// Body-local index of ordinary family value application resolutions.
pub type FamilyApplicationResolutionIndex = BTreeMap<ExpressionId, FamilyApplicationResolution>;
