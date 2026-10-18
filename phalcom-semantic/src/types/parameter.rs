//! Type parameter identities and generic signatures.

use super::id::{InferVarId, KindId, TypeId, TypeParameterId};
use super::store::TypeStore;
use super::variance::Variance;
use crate::diagnostic::SemanticSourceSpan;
use crate::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TypeParameterOwner {
    Declaration(DeclarationId),
    Callable(CallableId),
    Impl(crate::identity::ImplId),
}

impl TypeParameterOwner {
    pub(crate) fn clone_with_allocation_budget(&self, allocations: &mut AllocationBudget) -> Result<Self, AllocationError> {
        Ok(match self {
            Self::Declaration(owner) => Self::Declaration(owner.clone_with_allocation_budget(allocations)?),
            Self::Callable(owner) => Self::Callable(owner.clone_with_allocation_budget(allocations)?),
            Self::Impl(owner) => Self::Impl(owner.clone_with_allocation_budget(allocations)?),
        })
    }
}

/// Source binding shape for a generic parameter.
///
/// The mode is intentionally independent from the parameter kind: a single
/// parameter may itself accept a pack value, while a pack binder captures a
/// variable-length source lane into one pack value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GenericParameterMode {
    Single,
    PositionalPack,
    LabeledPack,
    CompletePack,
}

pub fn expansion_mode_accepts_kind(mode: GenericParameterMode, kind: KindId) -> bool {
    match mode {
        GenericParameterMode::Single => true,
        GenericParameterMode::PositionalPack => kind == KindId::TYPE_PACK,
        GenericParameterMode::LabeledPack => kind == KindId::LABELED_TYPE_PACK || kind == KindId::RECORD_ROW,
        GenericParameterMode::CompletePack => kind == KindId::COMPLETE_TYPE_PACK,
    }
}

/// Constraint-free declaration binder product published with a nominal shell.
///
/// This is deliberately distinct from [`GenericSignature`]: it establishes
/// binder identity and application shape before where-clause constraints can
/// be resolved through transparent aliases.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericBinderHeader {
    pub owner: TypeParameterOwner,
    pub parameters: Box<[TypeParameterId]>,
    pub parameter_kinds: Box<[KindId]>,
    pub parameter_variances: Box<[Variance]>,
    pub parameter_modes: Box<[GenericParameterMode]>,
}

impl GenericBinderHeader {
    pub(crate) fn from_signature_with_allocation_budget(
        store: &TypeStore,
        signature: &GenericSignature,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, AllocationError> {
        let metadata_complete = signature.parameter_kinds.len() == signature.parameters.len()
            && signature.parameter_variances.len() == signature.parameters.len()
            && signature.parameter_modes.len() == signature.parameters.len();
        let mut parameters = allocations.vector(signature.parameters.len())?;
        parameters.extend_from_slice(&signature.parameters);
        let mut kinds = allocations.vector(signature.parameters.len())?;
        let mut variances = allocations.vector(signature.parameters.len())?;
        let mut modes = allocations.vector(signature.parameters.len())?;
        for (index, parameter) in signature.parameters.iter().enumerate() {
            if metadata_complete {
                kinds.push(signature.parameter_kinds[index]);
                variances.push(signature.parameter_variances[index]);
                modes.push(signature.parameter_modes[index]);
            } else {
                let data = store.type_parameter(*parameter);
                kinds.push(data.kind);
                variances.push(data.variance);
                modes.push(data.mode);
            }
        }
        Ok(Self {
            owner: signature.owner.clone_with_allocation_budget(allocations)?,
            parameters: allocations.boxed_slice(parameters)?,
            parameter_kinds: allocations.boxed_slice(kinds)?,
            parameter_variances: allocations.boxed_slice(variances)?,
            parameter_modes: allocations.boxed_slice(modes)?,
        })
    }

    pub fn from_signature(store: &TypeStore, signature: &GenericSignature) -> Self {
        let metadata_complete = signature.parameter_kinds.len() == signature.parameters.len()
            && signature.parameter_variances.len() == signature.parameters.len()
            && signature.parameter_modes.len() == signature.parameters.len();
        Self {
            owner: signature.owner.clone(),
            parameters: signature.parameters.clone(),
            parameter_kinds: if metadata_complete {
                signature.parameter_kinds.clone()
            } else {
                signature.parameters.iter().map(|parameter| store.type_parameter(*parameter).kind).collect()
            },
            parameter_variances: if metadata_complete {
                signature.parameter_variances.clone()
            } else {
                signature.parameters.iter().map(|parameter| store.type_parameter(*parameter).variance).collect()
            },
            parameter_modes: if metadata_complete {
                signature.parameter_modes.clone()
            } else {
                signature.parameters.iter().map(|parameter| store.type_parameter(*parameter).mode).collect()
            },
        }
    }

    pub fn matches_signature(&self, signature: &GenericSignature) -> bool {
        self.owner == signature.owner
            && self.parameters == signature.parameters
            && self.parameter_kinds == signature.parameter_kinds
            && self.parameter_variances == signature.parameter_variances
            && self.parameter_modes == signature.parameter_modes
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeParameterData {
    pub owner: TypeParameterOwner,
    pub index: u32,
    pub name: Box<str>,
    pub kind: KindId,
    pub variance: Variance,
    pub mode: GenericParameterMode,
    pub source: Option<SemanticSourceSpan>,
}

impl TypeParameterData {
    pub fn new(owner: TypeParameterOwner, index: u32, name: impl Into<Box<str>>, kind: KindId) -> Self {
        Self {
            owner,
            index,
            name: name.into(),
            kind,
            variance: Variance::Invariant,
            mode: GenericParameterMode::Single,
            source: None,
        }
    }

    pub fn with_variance(mut self, variance: Variance) -> Self {
        self.variance = variance;
        self
    }

    pub fn with_mode(mut self, mode: GenericParameterMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_source(mut self, source: SemanticSourceSpan) -> Self {
        self.source = Some(source);
        self
    }
}

/// Role of a `Self` type term.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SelfRole {
    /// Instance type denoted by the current declaration, including specialization.
    InstanceType,
    /// Type of the lexical/dynamic receiver value itself.
    ReceiverValue,
}

/// Owner-relative `Self` type term.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SelfTypeTerm {
    pub owner: DeclarationId,
    pub side: DispatchSide,
    pub role: SelfRole,
}

/// Term used in generic declaration signatures and constraints.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TypeTerm {
    Canonical(TypeId),
    SelfType(SelfTypeTerm),
    Infer(InferVarId),
}

impl From<TypeId> for TypeTerm {
    fn from(ty: TypeId) -> Self {
        Self::Canonical(ty)
    }
}

/// Canonical generic constraint owned by a generic signature scope.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GenericConstraint {
    Subtype { lower: TypeTerm, upper: TypeTerm },
    Equivalent { left: TypeTerm, right: TypeTerm },
    Conforms { subject: TypeTerm, trait_ref: crate::traits::TraitRef },
}

/// Maps every type-bearing position in one generic constraint.
///
/// `Self` terms remain explicit at this layer so callers can preserve their
/// own receiver semantics while sharing the structural traversal for ordinary
/// operands and trait-reference arguments.
pub(crate) fn map_generic_constraint<Context, MapCanonical, MapSelf>(
    constraint: &GenericConstraint,
    context: &mut Context,
    mut map_canonical: MapCanonical,
    mut map_self: MapSelf,
) -> GenericConstraint
where
    MapCanonical: FnMut(&mut Context, TypeId) -> TypeId,
    MapSelf: FnMut(&mut Context, &SelfTypeTerm) -> TypeTerm,
{
    fn map_term<Context, MapCanonical, MapSelf>(term: &TypeTerm, context: &mut Context, map_canonical: &mut MapCanonical, map_self: &mut MapSelf) -> TypeTerm
    where
        MapCanonical: FnMut(&mut Context, TypeId) -> TypeId,
        MapSelf: FnMut(&mut Context, &SelfTypeTerm) -> TypeTerm,
    {
        match term {
            TypeTerm::Canonical(ty) => TypeTerm::Canonical(map_canonical(context, *ty)),
            TypeTerm::SelfType(self_term) => map_self(context, self_term),
            TypeTerm::Infer(variable) => TypeTerm::Infer(*variable),
        }
    }

    match constraint {
        GenericConstraint::Subtype { lower, upper } => GenericConstraint::Subtype {
            lower: map_term(lower, context, &mut map_canonical, &mut map_self),
            upper: map_term(upper, context, &mut map_canonical, &mut map_self),
        },
        GenericConstraint::Equivalent { left, right } => GenericConstraint::Equivalent {
            left: map_term(left, context, &mut map_canonical, &mut map_self),
            right: map_term(right, context, &mut map_canonical, &mut map_self),
        },
        GenericConstraint::Conforms { subject, trait_ref } => GenericConstraint::Conforms {
            subject: map_term(subject, context, &mut map_canonical, &mut map_self),
            trait_ref: crate::traits::TraitRef::new(
                trait_ref.declaration.clone(),
                trait_ref
                    .arguments
                    .iter()
                    .map(|&argument| map_canonical(context, argument))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericSignature {
    pub owner: TypeParameterOwner,
    pub parameters: Box<[TypeParameterId]>,
    pub parameter_kinds: Box<[KindId]>,
    pub parameter_kind_shapes: Box<[Box<str>]>,
    pub parameter_variances: Box<[Variance]>,
    pub parameter_modes: Box<[GenericParameterMode]>,
    pub constraint_shapes: Box<[Box<str>]>,
    pub constraints: Box<[GenericConstraint]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenericSignaturePublicationError {
    MissingParameter {
        index: u32,
        parameter: TypeParameterId,
    },
    OwnerMismatch {
        parameter: TypeParameterId,
    },
    NonContiguousParameter {
        parameter: TypeParameterId,
        expected: u32,
        actual: u32,
    },
    RecordRowParameterForm {
        parameter: TypeParameterId,
    },
    InvalidCanonicalType {
        ty: TypeId,
    },
    InferenceVariable {
        variable: InferVarId,
    },
    ParameterModeCount {
        expected: usize,
        actual: usize,
    },
    ParameterModeKindMismatch {
        parameter: TypeParameterId,
        mode: GenericParameterMode,
        kind: KindId,
    },
    ParameterModeMismatch {
        parameter: TypeParameterId,
        parameter_mode: GenericParameterMode,
        signature_mode: GenericParameterMode,
    },
    PackParameterVariance {
        parameter: TypeParameterId,
        mode: GenericParameterMode,
        variance: Variance,
    },
}

impl GenericSignature {
    pub(crate) fn clone_with_allocation_budget(&self, allocations: &mut AllocationBudget) -> Result<Self, AllocationError> {
        fn copy_boxed<T: Copy>(values: &[T], allocations: &mut AllocationBudget) -> Result<Box<[T]>, AllocationError> {
            let mut cloned = allocations.vector(values.len())?;
            cloned.extend_from_slice(values);
            allocations.boxed_slice(cloned)
        }

        fn clone_owner(owner: &TypeParameterOwner, allocations: &mut AllocationBudget) -> Result<TypeParameterOwner, AllocationError> {
            Ok(match owner {
                TypeParameterOwner::Declaration(owner) => TypeParameterOwner::Declaration(owner.clone_with_allocation_budget(allocations)?),
                TypeParameterOwner::Callable(owner) => TypeParameterOwner::Callable(owner.clone_with_allocation_budget(allocations)?),
                TypeParameterOwner::Impl(owner) => TypeParameterOwner::Impl(owner.clone_with_allocation_budget(allocations)?),
            })
        }

        fn clone_term(term: &TypeTerm, allocations: &mut AllocationBudget) -> Result<TypeTerm, AllocationError> {
            Ok(match term {
                TypeTerm::Canonical(ty) => TypeTerm::Canonical(*ty),
                TypeTerm::Infer(variable) => TypeTerm::Infer(*variable),
                TypeTerm::SelfType(term) => TypeTerm::SelfType(SelfTypeTerm {
                    owner: term.owner.clone_with_allocation_budget(allocations)?,
                    side: term.side,
                    role: term.role,
                }),
            })
        }

        fn clone_constraint(constraint: &GenericConstraint, allocations: &mut AllocationBudget) -> Result<GenericConstraint, AllocationError> {
            Ok(match constraint {
                GenericConstraint::Subtype { lower, upper } => GenericConstraint::Subtype {
                    lower: clone_term(lower, allocations)?,
                    upper: clone_term(upper, allocations)?,
                },
                GenericConstraint::Equivalent { left, right } => GenericConstraint::Equivalent {
                    left: clone_term(left, allocations)?,
                    right: clone_term(right, allocations)?,
                },
                GenericConstraint::Conforms { subject, trait_ref } => {
                    let mut arguments = allocations.vector(trait_ref.arguments.len())?;
                    arguments.extend_from_slice(&trait_ref.arguments);
                    GenericConstraint::Conforms {
                        subject: clone_term(subject, allocations)?,
                        trait_ref: crate::traits::TraitRef {
                            declaration: trait_ref.declaration.clone_with_allocation_budget(allocations)?,
                            arguments: allocations.boxed_slice(arguments)?,
                        },
                    }
                }
            })
        }

        let mut parameter_kind_shapes = allocations.vector(self.parameter_kind_shapes.len())?;
        for shape in &self.parameter_kind_shapes {
            parameter_kind_shapes.push(allocations.boxed_str(shape)?);
        }
        let mut constraint_shapes = allocations.vector(self.constraint_shapes.len())?;
        for shape in &self.constraint_shapes {
            constraint_shapes.push(allocations.boxed_str(shape)?);
        }
        let mut constraints = allocations.vector(self.constraints.len())?;
        for constraint in &self.constraints {
            constraints.push(clone_constraint(constraint, allocations)?);
        }

        Ok(Self {
            owner: clone_owner(&self.owner, allocations)?,
            parameters: copy_boxed(&self.parameters, allocations)?,
            parameter_kinds: copy_boxed(&self.parameter_kinds, allocations)?,
            parameter_kind_shapes: allocations.boxed_slice(parameter_kind_shapes)?,
            parameter_variances: copy_boxed(&self.parameter_variances, allocations)?,
            parameter_modes: copy_boxed(&self.parameter_modes, allocations)?,
            constraint_shapes: allocations.boxed_slice(constraint_shapes)?,
            constraints: allocations.boxed_slice(constraints)?,
        })
    }

    pub fn new(owner: TypeParameterOwner, parameters: Box<[TypeParameterId]>) -> Self {
        let parameter_count = parameters.len();
        Self {
            owner,
            parameters,
            parameter_kinds: Box::new([]),
            parameter_kind_shapes: Box::new([]),
            parameter_variances: Box::new([]),
            parameter_modes: vec![GenericParameterMode::Single; parameter_count].into_boxed_slice(),
            constraint_shapes: Box::new([]),
            constraints: Box::new([]),
        }
    }

    pub fn with_constraints(owner: TypeParameterOwner, parameters: Box<[TypeParameterId]>, constraints: Box<[GenericConstraint]>) -> Self {
        let parameter_count = parameters.len();
        Self {
            owner,
            parameters,
            parameter_kinds: Box::new([]),
            parameter_kind_shapes: Box::new([]),
            parameter_variances: Box::new([]),
            parameter_modes: vec![GenericParameterMode::Single; parameter_count].into_boxed_slice(),
            constraint_shapes: Box::new([]),
            constraints,
        }
    }

    pub fn with_parameter_metadata(mut self, kinds: Box<[KindId]>, variances: Box<[Variance]>) -> Self {
        self.parameter_kinds = kinds;
        self.parameter_variances = variances;
        self
    }

    pub fn with_parameter_modes(mut self, modes: Box<[GenericParameterMode]>) -> Self {
        self.parameter_modes = modes;
        self
    }

    pub fn with_parameter_kind_shapes(mut self, shapes: Box<[Box<str>]>) -> Self {
        self.parameter_kind_shapes = shapes;
        self
    }

    pub fn with_constraint_shapes(mut self, shapes: Box<[Box<str>]>) -> Self {
        self.constraint_shapes = shapes;
        self
    }

    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }

    pub fn parameter_at(&self, index: usize) -> Option<TypeParameterId> {
        self.parameters.get(index).copied()
    }

    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }

    pub fn constraint_at(&self, index: usize) -> Option<&GenericConstraint> {
        self.constraints.get(index)
    }

    pub fn validate_publishable(&self, store: &TypeStore) -> Result<(), GenericSignaturePublicationError> {
        if self.parameter_modes.len() != self.parameters.len() {
            return Err(GenericSignaturePublicationError::ParameterModeCount {
                expected: self.parameters.len(),
                actual: self.parameter_modes.len(),
            });
        }
        for (index, &parameter) in self.parameters.iter().enumerate() {
            let expected = index as u32;
            let Some(found) = store.find_type_parameter_id(&self.owner, expected) else {
                return Err(GenericSignaturePublicationError::MissingParameter { index: expected, parameter });
            };
            if found != parameter {
                return Err(GenericSignaturePublicationError::OwnerMismatch { parameter });
            }
            let data = store.type_parameter(parameter);
            if data.owner != self.owner {
                return Err(GenericSignaturePublicationError::OwnerMismatch { parameter });
            }
            if data.index != expected {
                return Err(GenericSignaturePublicationError::NonContiguousParameter {
                    parameter,
                    expected,
                    actual: data.index,
                });
            }
            if data.kind == KindId::RECORD_ROW && store.contains_parameter_type(parameter) {
                return Err(GenericSignaturePublicationError::RecordRowParameterForm { parameter });
            }
            let mode = self.parameter_modes[index];
            if data.mode != mode {
                return Err(GenericSignaturePublicationError::ParameterModeMismatch {
                    parameter,
                    parameter_mode: data.mode,
                    signature_mode: mode,
                });
            }
            if !expansion_mode_accepts_kind(mode, data.kind) {
                return Err(GenericSignaturePublicationError::ParameterModeKindMismatch {
                    parameter,
                    mode,
                    kind: data.kind,
                });
            }
            if mode != GenericParameterMode::Single && data.variance != Variance::Invariant {
                return Err(GenericSignaturePublicationError::PackParameterVariance {
                    parameter,
                    mode,
                    variance: data.variance,
                });
            }
        }

        let validate_term = |term: &TypeTerm| match term {
            TypeTerm::Canonical(ty) if ty.index() < store.type_count() => Ok(()),
            TypeTerm::Canonical(ty) => Err(GenericSignaturePublicationError::InvalidCanonicalType { ty: *ty }),
            TypeTerm::SelfType(_) => Ok(()),
            TypeTerm::Infer(variable) => Err(GenericSignaturePublicationError::InferenceVariable { variable: *variable }),
        };
        for constraint in self.constraints.iter() {
            match constraint {
                GenericConstraint::Subtype { lower, upper } => {
                    validate_term(lower)?;
                    validate_term(upper)?;
                }
                GenericConstraint::Equivalent { left, right } => {
                    validate_term(left)?;
                    validate_term(right)?;
                }
                GenericConstraint::Conforms { subject, trait_ref } => {
                    validate_term(subject)?;
                    for &arg in trait_ref.arguments.iter() {
                        if arg.index() >= store.type_count() {
                            return Err(GenericSignaturePublicationError::InvalidCanonicalType { ty: arg });
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
