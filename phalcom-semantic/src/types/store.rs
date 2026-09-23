//! Canonical Type Store with interning and normalization.

use super::application::TypeApplicationError;
use super::family::{FamilyMemberType, FamilyMemberTypeKind, FamilyType, FamilyTypeError, FamilyTypeId};
use super::id::{AlgebraicStaticConstructorId, KindId, ProperTypeId, RecordRowId, StaticDomainId, TypeId, TypeLambdaId, TypeParameterId, TypeStoreId, VariantTypeId};
use super::kind::{KindApplicationError, KindData};
use super::parameter::{SelfTypeTerm, TypeParameterData, TypeParameterOwner};
use super::row::{RecordRowData, RecordRowField, RecordRowTail};
use super::type_lambda::{BetaReductionError, BetaResult, TypeLambdaArena};
use super::type_member::{ProjectionSubject, TypeLevelProjection, TypeMemberSelector};
use super::variance::Variance;
use crate::identity::{AssociatedStaticDomainDeclarationId, DeclarationId, VariantId};
use crate::traits::{AssociatedTypeRequirementId, TraitRef};
use phalcom_common::selector::{SelectorKind, SelectorSlot};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TYPE_STORE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TupleTypeElement {
    pub label: Option<Box<str>>,
    pub ty: TypeId,
}

impl TupleTypeElement {
    pub fn unlabeled(ty: TypeId) -> Self {
        Self { label: None, ty }
    }

    pub fn labeled(label: impl Into<Box<str>>, ty: TypeId) -> Self {
        Self { label: Some(label.into()), ty }
    }
}

pub type RecordTypeField = RecordRowField;

use phalcom_ast::ast::RestMode;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CallableParameterType {
    pub label: Option<Box<str>>,
    pub ty: TypeId,
    pub rest: RestMode,
}

impl CallableParameterType {
    pub fn new(ty: TypeId) -> Self {
        Self {
            label: None,
            ty,
            rest: RestMode::None,
        }
    }

    pub fn with_label(mut self, label: impl Into<Box<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_rest(mut self, rest: RestMode) -> Self {
        self.rest = rest;
        self
    }

    pub fn is_rest(&self) -> bool {
        self.rest != RestMode::None
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CallableType {
    pub parameters: Box<[CallableParameterType]>,
    pub return_type: TypeId,
}

/// Canonical contextual associated-type projection.
///
/// The subject, relevant trait application, and trait-owned requirement
/// identity together determine the projection. Source spelling and ranges are
/// intentionally not part of this semantic type node.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AssociatedTypeProjection {
    pub subject: TypeId,
    pub trait_ref: TraitRef,
    pub requirement: AssociatedTypeRequirementId,
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum PackFormationError {
    #[error("pack element {ty:?} has invalid kind {kind:?}, expected proper type")]
    ElementNotProperType { ty: TypeId, kind: KindId },
    #[error("duplicate label in labeled type pack: {0}")]
    DuplicateLabel(String),
    #[error("invalid tuple pack component {ty:?} with kind {kind:?}")]
    InvalidTuplePackComponent { ty: TypeId, kind: KindId },
    #[error("invalid callable pack component {ty:?} with kind {kind:?}")]
    InvalidCallablePackComponent { ty: TypeId, kind: KindId },
    #[error("invalid complete pack component {ty:?} with kind {kind:?}")]
    InvalidCompletePackComponent { ty: TypeId, kind: KindId },
    #[error("invalid pack concat component {ty:?} with kind {kind:?}, expected {expected:?}")]
    InvalidPackConcatComponent { ty: TypeId, kind: KindId, expected: KindId },
    #[error("cannot project {lane} lane from {ty:?} with kind {kind:?}")]
    InvalidPackProjection { ty: TypeId, kind: KindId, lane: &'static str },
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum StaticDataFormationError {
    #[error("unknown static constructor {constructor:?}")]
    UnknownConstructor { constructor: AlgebraicStaticConstructorId },
    #[error("static constructor domain mismatch: expected {expected:?}, got {actual:?}")]
    DomainMismatch { expected: StaticDomainId, actual: StaticDomainId },
    #[error("static constructor arity mismatch: expected {expected}, got {actual}")]
    ArityMismatch { expected: usize, actual: usize },
    #[error("static constructor argument {index} has kind {actual:?}, expected {expected:?}")]
    ArgumentKindMismatch { index: usize, expected: KindId, actual: KindId },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LabeledTypePackElement {
    pub label: Box<str>,
    pub element: TypeId,
}

/// Structural canonical type definition.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeData {
    /// Canonical bottom type (inhabited by no values).
    Never,
    /// Canonical unit type (inhabited by single unit value).
    Unit,
    /// Proper static value type of a runtime class object (internal semantic representation).
    ClassObject { declaration: DeclarationId },
    /// Canonical nominal class declaration type or constructor form.
    Nominal { declaration: DeclarationId },
    /// Heterogeneous generic/static application (e.g. `List<Int>` or
    /// `Tensor<Float, Shape::Vector(3)>`).
    ///
    /// Record rows remain outside the `TypeId` domain, so they travel through
    /// the same application envelope as static terms without becoming types.
    Applied { origin: TypeId, arguments: Box<[StaticArgument]> },
    /// Exact static enum case type (e.g. `ExactCase<Option::Some(_), Option<Int>>`).
    ExactCase { variant: VariantTypeId, enum_type: TypeId },
    /// Flat, deduplicated, sorted union of two or more distinct types.
    Union(Box<[TypeId]>),
    /// Tuple type.
    Tuple(Box<[TupleTypeElement]>),
    /// Record / structural property map backed by a canonical record row.
    Record(RecordRowId),
    /// Flat, ordered pack of types (kind TypePack).
    TypePack(Box<[TypeId]>),
    /// Flat, ordered pack of labeled types (kind LabeledTypePack).
    LabeledTypePack(Box<[LabeledTypePackElement]>),
    /// Exact type-level product of positional and labeled pack lanes.
    CompleteTypePack { positionals: TypeId, labeled: TypeId },
    /// Flat ordered composition of positional pack contributions.
    TypePackConcat(Box<[TypeId]>),
    /// Flat ordered composition of labeled pack contributions.
    LabeledTypePackConcat(Box<[TypeId]>),
    /// Tuple type formed from symbolic positional and labeled packs.
    TupleFromPacks { positionals: TypeId, labeled: TypeId },
    /// Callable type formed from symbolic positional and labeled packs with a return type.
    CallableFromPacks {
        positionals: TypeId,
        labeled: TypeId,
        return_type: TypeId,
    },
    /// Callable / block signature.
    Callable(CallableType),
    /// First-class structural associated member family.
    Family(FamilyTypeId),
    /// Type variable parameter in generic declaration.
    Parameter(TypeParameterId),
    /// First-class type lambda form.
    Lambda(TypeLambdaId),
    /// Owner-relative `Self` type term.
    SelfType(SelfTypeTerm),
    /// Contextual associated-type projection.
    AssociatedProjection(AssociatedTypeProjection),
    /// Compiler-intrinsic or domain-native type-member projection.
    TypeMemberProjection(TypeLevelProjection),
    /// Canonical compile-time integer constant (static term, not a proper type).
    ///
    /// `ConstInt(n)` has kind `KindData::StaticDomain(CONST_INT_DOMAIN_ID)` and
    /// must never be accepted by `ProperTypeId`-guarded APIs.
    ConstInt(i64),
    /// Canonical structural static algebraic data term (static term, not a proper type).
    ///
    /// Represents one constructor application within a nominal `type data` static
    /// domain. Has kind `KindData::StaticDomain(domain_id)` and must never be
    /// accepted by `ProperTypeId`-guarded APIs.
    StaticData(AlgebraicStaticTerm),
}

/// Semantic canonical data for a nominal static domain.
///
/// Two static domains are identical iff their `StaticDomainData` representations
/// are structurally equal. Raw `StaticDomainId` integers must not be serialized
/// as durable identity; use the `declaration` field or a `ConstInt` marker instead.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum StaticDomainData {
    /// The built-in domain of compile-time integer constants.
    ConstInt,
    /// A nominal static domain introduced by a `type data` declaration.
    Nominal { declaration: DeclarationId },
    /// An associated static domain specialized by its owner's canonical arguments.
    Associated {
        declaration: AssociatedStaticDomainDeclarationId,
        owner_arguments: Box<[StaticArgument]>,
    },
}

/// A canonical static algebraic term: one constructor application within a
/// nominal `type data` static domain.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AlgebraicStaticTerm {
    /// The static domain this term belongs to.
    pub domain: StaticDomainId,
    /// The specific constructor or case within the domain.
    pub constructor: AlgebraicStaticConstructorId,
    /// Canonical static arguments to the constructor.
    pub arguments: Box<[StaticArgument]>,
}

/// An argument to a static algebraic constructor.
///
/// Static constructors may accept static type-level terms or record-row terms.
/// Arbitrary runtime expressions are not accepted; only terms whose identity
/// is deterministic at compile time may appear here.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StaticArgument {
    /// A static term argument (itself a `TypeId` with a static-domain kind).
    Term(TypeId),
    /// A record-row argument (for future structural static row domains; P2).
    RecordRow(RecordRowId),
}

impl StaticArgument {
    #[inline]
    pub fn as_type_id(self) -> Option<TypeId> {
        match self {
            Self::Term(ty) => Some(ty),
            Self::RecordRow(_) => None,
        }
    }
}

/// Canonical semantic data for a registered static algebraic constructor.
///
/// One constructor corresponds to one product `type data` primary constructor,
/// or to one case/variant within a sum `type data`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AlgebraicStaticConstructorData {
    /// The domain this constructor belongs to.
    pub domain: StaticDomainId,
    /// Stable declaration reference for the owning `type data`.
    pub declaration: DeclarationId,
    /// Case name within a sum `type data`, or the type name for a product
    /// `type data` primary constructor.
    pub case_name: Box<str>,
    /// Canonical kinds accepted by this constructor's static arguments.
    pub parameter_kinds: Box<[KindId]>,
}

/// Central store for canonical type interning, hash-consing, and kind assignments.
#[derive(Clone, Debug)]
pub struct TypeStore {
    id: TypeStoreId,
    types: Vec<TypeData>,
    type_to_id: HashMap<(TypeData, KindId), TypeId>,
    kinds: Vec<KindData>,
    kind_to_id: HashMap<KindData, KindId>,
    type_kinds: Vec<KindId>,
    type_parameters: Vec<TypeParameterData>,
    parameter_to_id: HashMap<(TypeParameterOwner, u32), TypeParameterId>,
    parameter_variances: HashMap<(DeclarationId, u32), Variance>,
    lambda_arena: TypeLambdaArena,
    row_arena: Vec<RecordRowData>,
    row_interner: HashMap<RecordRowData, RecordRowId>,
    family_arena: Vec<FamilyType>,
    family_interner: HashMap<FamilyType, FamilyTypeId>,
    variant_identities: Vec<VariantId>,
    variant_identity_to_id: HashMap<VariantId, VariantTypeId>,
    static_domains: Vec<StaticDomainData>,
    static_domain_to_id: HashMap<StaticDomainData, StaticDomainId>,
    static_constructors: Vec<AlgebraicStaticConstructorData>,
    static_constructor_to_id: HashMap<AlgebraicStaticConstructorData, AlgebraicStaticConstructorId>,
    const_int_domain: StaticDomainId,

    never_id: TypeId,
    unit_id: TypeId,
}

impl Default for TypeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeStore {
    pub fn new() -> Self {
        let mut store = Self {
            id: TypeStoreId(NEXT_TYPE_STORE_ID.fetch_add(1, Ordering::Relaxed)),
            types: Vec::new(),
            type_to_id: HashMap::new(),
            kinds: Vec::new(),
            kind_to_id: HashMap::new(),
            type_kinds: Vec::new(),
            type_parameters: Vec::new(),
            parameter_to_id: HashMap::new(),
            parameter_variances: HashMap::new(),
            lambda_arena: TypeLambdaArena::new(),
            row_arena: Vec::new(),
            row_interner: HashMap::new(),
            family_arena: Vec::new(),
            family_interner: HashMap::new(),
            variant_identities: Vec::new(),
            variant_identity_to_id: HashMap::new(),
            static_domains: Vec::new(),
            static_domain_to_id: HashMap::new(),
            static_constructors: Vec::new(),
            static_constructor_to_id: HashMap::new(),
            const_int_domain: StaticDomainId::from_index(0),
            never_id: TypeId::DUMMY,
            unit_id: TypeId::DUMMY,
        };

        // Kind::Type is KindId(0)
        let type_kind = store.intern_kind(KindData::Type);
        assert_eq!(type_kind, KindId::TYPE);
        let record_row_kind = store.intern_kind(KindData::RecordRow);
        assert_eq!(record_row_kind, KindId::RECORD_ROW);
        let type_pack_kind = store.intern_kind(KindData::TypePack);
        assert_eq!(type_pack_kind, KindId::TYPE_PACK);
        let labeled_type_pack_kind = store.intern_kind(KindData::LabeledTypePack);
        assert_eq!(labeled_type_pack_kind, KindId::LABELED_TYPE_PACK);
        let complete_type_pack_kind = store.intern_kind(KindData::CompleteTypePack);
        assert_eq!(complete_type_pack_kind, KindId::COMPLETE_TYPE_PACK);

        store.const_int_domain = store.intern_static_domain(StaticDomainData::ConstInt);

        store.never_id = store.intern_with_kind(TypeData::Never, KindId::TYPE);
        store.unit_id = store.intern_with_kind(TypeData::Unit, KindId::TYPE);

        store
    }

    pub fn id(&self) -> TypeStoreId {
        self.id
    }

    pub fn type_count(&self) -> usize {
        self.types.len()
    }

    pub fn proper_type(&self, id: TypeId) -> Result<ProperTypeId, KindId> {
        let kind = self.kind_of(id);
        if kind == KindId::TYPE { Ok(ProperTypeId(id)) } else { Err(kind) }
    }

    pub fn intern_type_parameter(&mut self, data: TypeParameterData) -> TypeParameterId {
        let key = (data.owner.clone(), data.index);
        if let TypeParameterOwner::Declaration(ref decl) = data.owner {
            self.parameter_variances.insert((decl.clone(), data.index), data.variance);
        }

        if let Some(&id) = self.parameter_to_id.get(&key) {
            let existing = &self.type_parameters[id.index()];
            let same_semantics = existing.name == data.name && existing.kind == data.kind && existing.variance == data.variance && existing.mode == data.mode;

            if same_semantics {
                // Source provenance is revision-local presentation data, not semantic
                // identity. Refresh it in the live store while retained snapshot clones
                // continue to preserve the provenance from their own revision.
                self.type_parameters[id.index()].source = data.source;
                return id;
            }
        }

        // `(owner, index)` identifies the current binder slot, not an eternal arena
        // object. If the slot's semantic meaning changes, allocate a new parameter
        // version rather than mutating the old one. Cached products and retained
        // snapshots may still reference the previous ID and must keep its denotation.
        let id = TypeParameterId(self.type_parameters.len() as u32);
        self.type_parameters.push(data);
        self.parameter_to_id.insert(key, id);
        id
    }

    pub fn set_parameter_variance(&mut self, decl: DeclarationId, index: u32, variance: Variance) {
        self.parameter_variances.insert((decl, index), variance);
    }

    pub fn get_parameter_variance(&self, decl: &DeclarationId, index: u32) -> Option<Variance> {
        self.parameter_variances.get(&(decl.clone(), index)).copied()
    }

    pub fn find_type_parameter_id(&self, owner: &TypeParameterOwner, index: u32) -> Option<TypeParameterId> {
        self.parameter_to_id.get(&(owner.clone(), index)).copied()
    }

    #[inline]
    pub fn arena(&self) -> &TypeLambdaArena {
        &self.lambda_arena
    }

    #[inline]
    pub fn arena_mut(&mut self) -> &mut TypeLambdaArena {
        &mut self.lambda_arena
    }

    /// Interns a type lambda form into the store.
    pub fn lambda(&mut self, parameter_kinds: Box<[KindId]>, body: super::id::ScopedTypeId, result_kind: KindId) -> TypeId {
        let lambda_id = self.lambda_arena.intern_lambda(parameter_kinds, body, result_kind, None);
        self.type_lambda(lambda_id)
    }

    /// Interns an existing `TypeLambdaId` into the `TypeStore` with its computed arrow kind.
    pub fn type_lambda(&mut self, lambda_id: TypeLambdaId) -> TypeId {
        let lambda = self.lambda_arena.get_lambda(lambda_id).clone();
        let kind = if lambda.parameter_kinds.is_empty() {
            lambda.result_kind
        } else {
            self.arrow_kind(lambda.parameter_kinds, lambda.result_kind)
        };
        self.intern_with_kind(TypeData::Lambda(lambda_id), kind)
    }

    /// Returns a canonical lambda with its captured free types specialized.
    pub fn substitute_type_lambda(&mut self, lambda_id: TypeLambdaId, substitution: &super::substitution::TypeSubstitution) -> TypeId {
        let mut arena = self.lambda_arena.clone();
        let specialized = arena.substitute_free_types(lambda_id, substitution, self);
        self.lambda_arena = arena;
        self.type_lambda(specialized)
    }

    /// Interns an owner-relative `Self` type term.
    pub fn self_type(&mut self, term: SelfTypeTerm) -> TypeId {
        self.intern_with_kind(TypeData::SelfType(term), KindId::TYPE)
    }

    /// Interns a canonical contextual associated-type projection.
    pub fn associated_projection(&mut self, subject: TypeId, trait_ref: TraitRef, requirement: AssociatedTypeRequirementId) -> TypeId {
        debug_assert_eq!(
            requirement.owner, trait_ref.declaration,
            "associated projection requirement must belong to its trait"
        );
        self.intern_with_kind(
            TypeData::AssociatedProjection(AssociatedTypeProjection {
                subject,
                trait_ref,
                requirement,
            }),
            KindId::TYPE,
        )
    }

    pub fn type_member_projection(&mut self, projection: TypeLevelProjection, kind: KindId) -> TypeId {
        // `ConstTerm::Int` is retained as a compatibility input only.  Store
        // projections in the same canonical term universe used by static
        // applications and inference.
        let arguments = projection
            .arguments
            .iter()
            .map(|argument| match argument {
                super::type_member::TypeMemberArgument::Const(super::type_member::ConstTerm::Int(value)) => {
                    super::type_member::TypeMemberArgument::Type(self.const_int(*value))
                }
                other => other.clone(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let projection = TypeLevelProjection { arguments, ..projection };
        self.intern_with_kind(TypeData::TypeMemberProjection(projection), kind)
    }

    #[inline]
    pub fn type_parameter(&self, id: TypeParameterId) -> &TypeParameterData {
        &self.type_parameters[id.index()]
    }

    pub fn try_type_parameter(&self, id: TypeParameterId) -> Option<&TypeParameterData> {
        self.type_parameters.get(id.index())
    }

    pub fn parameter_form(&mut self, id: TypeParameterId) -> TypeId {
        let kind = self.type_parameter(id).kind;
        assert_ne!(
            kind,
            KindId::RECORD_ROW,
            "RecordRow-kinded type parameters must never produce TypeData::Parameter"
        );
        self.intern_with_kind(TypeData::Parameter(id), kind)
    }

    pub fn contains_parameter_type(&self, parameter: TypeParameterId) -> bool {
        self.types.iter().any(|ty| matches!(ty, TypeData::Parameter(id) if *id == parameter))
    }

    #[inline]
    pub fn never(&self) -> TypeId {
        self.never_id
    }

    #[inline]
    pub fn unit(&self) -> TypeId {
        self.unit_id
    }

    pub fn intern_kind(&mut self, data: KindData) -> KindId {
        if let Some(&id) = self.kind_to_id.get(&data) {
            return id;
        }
        let id = KindId(self.kinds.len() as u32);
        self.kinds.push(data.clone());
        self.kind_to_id.insert(data, id);
        id
    }

    pub fn get_kind(&self, id: KindId) -> &KindData {
        &self.kinds[id.index()]
    }

    pub fn format_kind(&self, id: KindId) -> String {
        match self.get_kind(id) {
            KindData::Type => "Type".to_string(),
            KindData::RecordRow => "RecordRow".to_string(),
            KindData::TypePack => "TypePack".to_string(),
            KindData::LabeledTypePack => "LabeledTypePack".to_string(),
            KindData::CompleteTypePack => "CompleteTypePack".to_string(),
            KindData::Arrow { parameters, result } => {
                let parameters = parameters.iter().map(|&parameter| self.format_kind(parameter)).collect::<Vec<_>>().join(", ");
                format!("({parameters}) -> {}", self.format_kind(*result))
            }
            KindData::StaticDomain(_) => "StaticDomain".to_string(),
        }
    }


    pub fn arrow_kind(&mut self, parameters: Box<[KindId]>, result: KindId) -> KindId {
        if parameters.is_empty() {
            return result;
        }
        let (params, final_result) = match self.get_kind(result).clone() {
            KindData::Arrow {
                parameters: sub_params,
                result: sub_res,
            } => {
                let mut combined = parameters.to_vec();
                combined.extend_from_slice(&sub_params);
                (combined.into_boxed_slice(), sub_res)
            }
            _ => (parameters, result),
        };
        self.intern_kind(KindData::Arrow {
            parameters: params,
            result: final_result,
        })
    }

    pub fn apply_kind(&mut self, callee: KindId, arguments: &[KindId]) -> Result<KindId, KindApplicationError> {
        if arguments.is_empty() {
            return Ok(callee);
        }

        let callee_data = self.get_kind(callee).clone();
        match callee_data {
            KindData::Type
            | KindData::RecordRow
            | KindData::TypePack
            | KindData::LabeledTypePack
            | KindData::CompleteTypePack
            | KindData::StaticDomain(_) => Err(KindApplicationError::NotApplicable { kind: callee }),
            KindData::Arrow { parameters, result } => {
                if arguments.len() > parameters.len() {
                    return Err(KindApplicationError::TooManyArguments {
                        supplied: arguments.len(),
                        accepted: parameters.len(),
                    });
                }
                for (i, (&arg, &param)) in arguments.iter().zip(parameters.iter()).enumerate() {
                    if arg != param {
                        return Err(KindApplicationError::ArgumentKindMismatch {
                            index: i,
                            expected: param,
                            actual: arg,
                        });
                    }
                }
                if arguments.len() == parameters.len() {
                    Ok(result)
                } else {
                    let remaining = parameters[arguments.len()..].to_vec().into_boxed_slice();
                    Ok(self.arrow_kind(remaining, result))
                }
            }
        }
    }

    pub fn intern_with_kind(&mut self, data: TypeData, kind: KindId) -> TypeId {
        // Kind is part of a canonical type form's identity. This matters for a
        // persistent store: the same declaration or parameter payload can acquire a
        // different kind in a later semantic revision, and the old TypeId must retain
        // its original denotation for cached products and retained snapshots.
        let key = (data.clone(), kind);
        if let Some(&id) = self.type_to_id.get(&key) {
            return id;
        }

        let id = TypeId(self.types.len() as u32);
        self.types.push(data);
        self.type_kinds.push(kind);
        self.type_to_id.insert(key, id);
        debug_assert_eq!(self.types.len(), self.type_kinds.len());
        id
    }

    #[inline]
    pub fn kind_of(&self, ty: TypeId) -> KindId {
        self.type_kinds[ty.index()]
    }

    #[inline]
    pub fn is_proper_type(&self, form: TypeId) -> bool {
        self.kind_of(form) == KindId::TYPE
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.types.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Interns one nominal or built-in static domain in this type store.
    pub fn intern_static_domain(&mut self, data: StaticDomainData) -> StaticDomainId {
        if let Some(&id) = self.static_domain_to_id.get(&data) {
            return id;
        }
        let id = StaticDomainId::from_index(self.static_domains.len());
        self.static_domains.push(data.clone());
        self.static_domain_to_id.insert(data, id);
        id
    }

    /// Returns the built-in compile-time integer domain.
    pub fn const_int_domain(&self) -> StaticDomainId {
        self.const_int_domain
    }

    pub fn static_domain(&self, id: StaticDomainId) -> Option<&StaticDomainData> {
        self.static_domains.get(id.index())
    }

    pub fn static_constructor_count(&self) -> usize {
        self.static_constructors.len()
    }

    /// Returns the canonical static kind for a domain.
    pub fn static_domain_kind(&mut self, domain: StaticDomainId) -> KindId {
        self.intern_kind(KindData::StaticDomain(domain))
    }

    /// Interns the nominal static domain owned by one declaration.
    pub fn nominal_static_domain(&mut self, declaration: DeclarationId) -> StaticDomainId {
        self.intern_static_domain(StaticDomainData::Nominal { declaration })
    }

    /// Interns one canonical specialization of an associated static domain.
    pub fn associated_static_domain(
        &mut self,
        declaration: AssociatedStaticDomainDeclarationId,
        owner_arguments: Box<[StaticArgument]>,
    ) -> StaticDomainId {
        self.intern_static_domain(StaticDomainData::Associated {
            declaration,
            owner_arguments,
        })
    }

    /// Registers one static constructor or case and returns its store-local identity.
    pub fn intern_static_constructor(
        &mut self,
        domain: StaticDomainId,
        declaration: DeclarationId,
        case_name: impl Into<Box<str>>,
        parameter_kinds: Box<[KindId]>,
    ) -> AlgebraicStaticConstructorId {
        let data = AlgebraicStaticConstructorData {
            domain,
            declaration,
            case_name: case_name.into(),
            parameter_kinds,
        };
        if let Some(&id) = self.static_constructor_to_id.get(&data) {
            return id;
        }
        let id = AlgebraicStaticConstructorId::from_index(self.static_constructors.len());
        self.static_constructors.push(data.clone());
        self.static_constructor_to_id.insert(data, id);
        id
    }

    pub fn static_constructor(&self, id: AlgebraicStaticConstructorId) -> Option<&AlgebraicStaticConstructorData> {
        self.static_constructors.get(id.index())
    }

    /// Returns the canonical static constructors declared by one domain.
    /// This is identity-based enumeration for semantic member selection; it is
    /// not a separate name lookup authority.
    pub fn static_constructor_ids_for_domain(&self, domain: StaticDomainId) -> Vec<AlgebraicStaticConstructorId> {
        self.static_constructors
            .iter()
            .enumerate()
            .filter_map(|(index, constructor)| {
                (constructor.domain == domain).then(|| AlgebraicStaticConstructorId::from_index(index))
            })
            .collect()
    }

    /// Forms a canonical compile-time integer term.
    pub fn const_int(&mut self, value: i64) -> TypeId {
        let kind = self.static_domain_kind(self.const_int_domain);
        self.intern_with_kind(TypeData::ConstInt(value), kind)
    }

    /// Forms and validates a canonical type-data constructor/case term.
    pub fn static_data(
        &mut self,
        domain: StaticDomainId,
        constructor: AlgebraicStaticConstructorId,
        arguments: Box<[StaticArgument]>,
    ) -> Result<TypeId, StaticDataFormationError> {
        let Some(metadata) = self.static_constructor(constructor).cloned() else {
            return Err(StaticDataFormationError::UnknownConstructor { constructor });
        };
        if metadata.domain != domain {
            return Err(StaticDataFormationError::DomainMismatch {
                expected: metadata.domain,
                actual: domain,
            });
        }
        if metadata.parameter_kinds.len() != arguments.len() {
            return Err(StaticDataFormationError::ArityMismatch {
                expected: metadata.parameter_kinds.len(),
                actual: arguments.len(),
            });
        }
        for (index, (argument, &expected)) in arguments.iter().zip(metadata.parameter_kinds.iter()).enumerate() {
            let actual = match argument {
                StaticArgument::Term(ty) => self.kind_of(*ty),
                StaticArgument::RecordRow(_) => KindId::RECORD_ROW,
            };
            if actual != expected {
                return Err(StaticDataFormationError::ArgumentKindMismatch { index, expected, actual });
            }
        }
        let kind = self.static_domain_kind(domain);
        Ok(self.intern_with_kind(
            TypeData::StaticData(AlgebraicStaticTerm {
                domain,
                constructor,
                arguments,
            }),
            kind,
        ))
    }

    #[inline]
    pub fn get(&self, id: TypeId) -> &TypeData {
        &self.types[id.index()]
    }

    pub fn nominal_form(&mut self, declaration: DeclarationId, kind: KindId) -> TypeId {
        self.intern_with_kind(TypeData::Nominal { declaration }, kind)
    }

    pub fn nominal_type(&mut self, declaration: DeclarationId) -> TypeId {
        self.nominal_form(declaration, KindId::TYPE)
    }

    pub fn class_object_type(&mut self, declaration: DeclarationId) -> TypeId {
        self.intern_with_kind(TypeData::ClassObject { declaration }, KindId::TYPE)
    }

    /// Legacy compatibility helper for tests / callers expecting nominal type.
    pub fn nominal(&mut self, declaration: DeclarationId) -> TypeId {
        self.nominal_type(declaration)
    }

    /// Interns a heterogeneous generic/static application with checked kinding
    /// and beta-reduction for type lambdas.
    pub fn apply_static_form(&mut self, origin: TypeId, arguments: &[StaticArgument]) -> Result<TypeId, TypeApplicationError> {
        if arguments.is_empty() {
            return Ok(origin);
        }

        // If origin is a TypeLambda, perform beta-reduction directly
        if let TypeData::Lambda(lambda_id) = self.get(origin).clone() {
            let mut type_arguments = Vec::with_capacity(arguments.len());
            for argument in arguments {
                let StaticArgument::Term(argument) = argument else {
                    return Err(TypeApplicationError::MalformedLambda);
                };
                type_arguments.push(*argument);
            }
            let mut arena = self.lambda_arena.clone();
            let res = arena.beta_reduce(lambda_id, &type_arguments, self);
            self.lambda_arena = arena;
            match res {
                Ok(BetaResult::Canonical(can)) => return Ok(can),
                Ok(BetaResult::ResidualLambda(res_id)) => {
                    return Ok(self.type_lambda(res_id));
                }
                Err(BetaReductionError::TooManyArguments { expected, actual }) => {
                    return Err(TypeApplicationError::TooManyArguments {
                        supplied: actual,
                        accepted: expected,
                    });
                }
                Err(BetaReductionError::KindMismatch {
                    parameter_index,
                    expected,
                    actual,
                }) => {
                    return Err(TypeApplicationError::ArgumentKindMismatch {
                        index: parameter_index as usize,
                        expected,
                        actual,
                    });
                }
                Err(BetaReductionError::UnboundVariable { .. }) => return Err(TypeApplicationError::MalformedLambda),
                Err(BetaReductionError::Application(error)) => return Err(error),
                Err(BetaReductionError::UnsupportedRecordRowArgument | BetaReductionError::RecordRowFormation(_) | BetaReductionError::PackFormation(_)) => {
                    return Err(TypeApplicationError::MalformedLambda);
                }
            }
        }

        let origin_kind = self.kind_of(origin);
        let arg_kinds: Vec<KindId> = arguments
            .iter()
            .map(|argument| match argument {
                StaticArgument::Term(argument) => self.kind_of(*argument),
                StaticArgument::RecordRow(_) => KindId::RECORD_ROW,
            })
            .collect();

        let residual_kind = match self.apply_kind(origin_kind, &arg_kinds) {
            Ok(k) => k,
            Err(KindApplicationError::NotApplicable { kind }) => {
                return Err(TypeApplicationError::NotAConstructor { origin, kind });
            }
            Err(KindApplicationError::TooManyArguments { supplied, accepted }) => {
                return Err(TypeApplicationError::TooManyArguments { supplied, accepted });
            }
            Err(KindApplicationError::ArgumentKindMismatch { index, expected, actual }) => {
                return Err(TypeApplicationError::ArgumentKindMismatch { index, expected, actual });
            }
        };

        let (final_origin, final_args) = match self.get(origin).clone() {
            TypeData::Applied {
                origin: base,
                arguments: old_args,
            } => {
                let mut combined = old_args.to_vec();
                combined.extend_from_slice(arguments);
                (base, combined.into_boxed_slice())
            }
            _ => (origin, arguments.to_vec().into_boxed_slice()),
        };

        Ok(self.intern_with_kind(
            TypeData::Applied {
                origin: final_origin,
                arguments: final_args,
            },
            residual_kind,
        ))
    }

    /// Type-only compatibility wrapper for callers whose arguments are all
    /// proper/static `TypeId` terms rather than record-row terms.
    pub fn apply_type_form(&mut self, origin: TypeId, arguments: &[TypeId]) -> Result<TypeId, TypeApplicationError> {
        let arguments = arguments.iter().copied().map(StaticArgument::Term).collect::<Vec<_>>();
        self.apply_static_form(origin, &arguments)
    }

    /// Interns a tuple type with full validation.
    pub fn tuple(&mut self, elements: Box<[TupleTypeElement]>) -> TypeId {
        if elements.is_empty() {
            return self.unit();
        }
        let mut seen_labels = std::collections::HashSet::new();
        let mut seen_labeled = false;
        for elem in elements.iter() {
            assert!(self.is_proper_type(elem.ty), "tuple element must be a proper type");
            if let Some(ref l) = elem.label {
                seen_labeled = true;
                assert!(seen_labels.insert(l.clone()), "duplicate label in tuple: {}", l);
            } else {
                assert!(!seen_labeled, "unlabeled tuple element cannot follow labeled elements");
            }
        }
        self.intern_with_kind(TypeData::Tuple(elements), KindId::TYPE)
    }

    /// Interns a TypePack into the store after validation.
    pub fn type_pack(&mut self, elements: Box<[TypeId]>) -> Result<TypeId, PackFormationError> {
        for &elem in elements.iter() {
            if !self.is_proper_type(elem) {
                return Err(PackFormationError::ElementNotProperType {
                    ty: elem,
                    kind: self.kind_of(elem),
                });
            }
        }
        Ok(self.intern_with_kind(TypeData::TypePack(elements), KindId::TYPE_PACK))
    }

    /// Interns a LabeledTypePack into the store after validation.
    pub fn labeled_type_pack(&mut self, elements: Box<[LabeledTypePackElement]>) -> Result<TypeId, PackFormationError> {
        let mut seen = std::collections::HashSet::new();
        for elem in elements.iter() {
            if !seen.insert(elem.label.clone()) {
                return Err(PackFormationError::DuplicateLabel(elem.label.to_string()));
            }
            if !self.is_proper_type(elem.element) {
                return Err(PackFormationError::ElementNotProperType {
                    ty: elem.element,
                    kind: self.kind_of(elem.element),
                });
            }
        }
        Ok(self.intern_with_kind(TypeData::LabeledTypePack(elements), KindId::LABELED_TYPE_PACK))
    }

    /// Interns the exact two-lane type-level pack product.
    pub fn complete_type_pack(&mut self, positionals: TypeId, labeled: TypeId) -> Result<TypeId, PackFormationError> {
        let positional_kind = self.kind_of(positionals);
        if positional_kind != KindId::TYPE_PACK {
            return Err(PackFormationError::InvalidCompletePackComponent {
                ty: positionals,
                kind: positional_kind,
            });
        }
        let labeled_kind = self.kind_of(labeled);
        if labeled_kind != KindId::LABELED_TYPE_PACK {
            return Err(PackFormationError::InvalidCompletePackComponent {
                ty: labeled,
                kind: labeled_kind,
            });
        }
        Ok(self.intern_with_kind(TypeData::CompleteTypePack { positionals, labeled }, KindId::COMPLETE_TYPE_PACK))
    }

    /// Returns the positional lane of a complete pack or Tuple-shaped type.
    pub fn project_positionals(&mut self, ty: TypeId) -> Result<TypeId, PackFormationError> {
        if self.kind_of(ty) == KindId::TYPE_PACK {
            return Ok(ty);
        }
        match self.get(ty).clone() {
            TypeData::CompleteTypePack { positionals, .. } | TypeData::TupleFromPacks { positionals, .. } => Ok(positionals),
            TypeData::Tuple(elements) => self.type_pack(
                elements
                    .iter()
                    .filter(|element| element.label.is_none())
                    .map(|element| element.ty)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            TypeData::Unit => self.type_pack(Box::new([])),
            _ if self.kind_of(ty) == KindId::COMPLETE_TYPE_PACK => Ok(self.type_member_projection(
                TypeLevelProjection {
                    subject: ProjectionSubject::Type(ty),
                    selector: TypeMemberSelector::named("Positionals"),
                    arguments: Box::new([]),
                    source_declaration: None,
                },
                KindId::TYPE_PACK,
            )),
            _ => Err(PackFormationError::InvalidPackProjection {
                ty,
                kind: self.kind_of(ty),
                lane: "positional",
            }),
        }
    }

    /// Returns the labeled lane of a complete pack or Tuple-shaped type.
    pub fn project_labeled(&mut self, ty: TypeId) -> Result<TypeId, PackFormationError> {
        if self.kind_of(ty) == KindId::LABELED_TYPE_PACK {
            return Ok(ty);
        }
        match self.get(ty).clone() {
            TypeData::CompleteTypePack { labeled, .. } | TypeData::TupleFromPacks { labeled, .. } => Ok(labeled),
            TypeData::Tuple(elements) => self.labeled_type_pack(
                elements
                    .iter()
                    .filter_map(|element| {
                        element.label.as_ref().map(|label| LabeledTypePackElement {
                            label: label.clone(),
                            element: element.ty,
                        })
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            TypeData::Unit => self.labeled_type_pack(Box::new([])),
            _ if self.kind_of(ty) == KindId::COMPLETE_TYPE_PACK => Ok(self.type_member_projection(
                TypeLevelProjection {
                    subject: ProjectionSubject::Type(ty),
                    selector: TypeMemberSelector::named("Labeled"),
                    arguments: Box::new([]),
                    source_declaration: None,
                },
                KindId::LABELED_TYPE_PACK,
            )),
            _ => Err(PackFormationError::InvalidPackProjection {
                ty,
                kind: self.kind_of(ty),
                lane: "labeled",
            }),
        }
    }

    /// Returns a complete pack only from a complete pack or Tuple-shaped type.
    pub fn project_complete(&mut self, ty: TypeId) -> Result<TypeId, PackFormationError> {
        if self.kind_of(ty) == KindId::COMPLETE_TYPE_PACK {
            return Ok(ty);
        }
        match self.get(ty) {
            TypeData::Tuple(_) | TypeData::TupleFromPacks { .. } | TypeData::Unit => {
                let positionals = self.project_positionals(ty)?;
                let labeled = self.project_labeled(ty)?;
                self.complete_type_pack(positionals, labeled)
            }
            _ => Err(PackFormationError::InvalidPackProjection {
                ty,
                kind: self.kind_of(ty),
                lane: "complete",
            }),
        }
    }

    /// Canonically composes positional pack contributions into one flat term.
    pub fn type_pack_concat(&mut self, segments: &[TypeId]) -> Result<TypeId, PackFormationError> {
        let mut flattened = Vec::new();
        for &segment in segments {
            self.collect_type_pack_concat_segments(segment, &mut flattened)?;
        }
        let mut normalized = Vec::new();
        let mut concrete = Vec::new();
        for segment in flattened {
            match self.get(segment).clone() {
                TypeData::TypePack(elements) => concrete.extend(elements.iter().copied()),
                _ => {
                    if !concrete.is_empty() {
                        normalized.push(self.type_pack(std::mem::take(&mut concrete).into_boxed_slice())?);
                    }
                    normalized.push(segment);
                }
            }
        }
        if !concrete.is_empty() {
            normalized.push(self.type_pack(concrete.into_boxed_slice())?);
        }
        match normalized.len() {
            0 => self.type_pack(Box::new([])),
            1 => Ok(normalized[0]),
            _ => Ok(self.intern_with_kind(TypeData::TypePackConcat(normalized.into_boxed_slice()), KindId::TYPE_PACK)),
        }
    }

    fn collect_type_pack_concat_segments(&self, segment: TypeId, output: &mut Vec<TypeId>) -> Result<(), PackFormationError> {
        let kind = self.kind_of(segment);
        if kind != KindId::TYPE_PACK {
            return Err(PackFormationError::InvalidPackConcatComponent {
                ty: segment,
                kind,
                expected: KindId::TYPE_PACK,
            });
        }
        match self.get(segment) {
            TypeData::TypePackConcat(children) => {
                for &child in children.iter() {
                    self.collect_type_pack_concat_segments(child, output)?;
                }
            }
            TypeData::TypePack(elements) if elements.is_empty() => {}
            _ => output.push(segment),
        }
        Ok(())
    }

    /// Canonically composes labeled pack contributions while preserving unresolved overlap.
    pub fn labeled_type_pack_concat(&mut self, segments: &[TypeId]) -> Result<TypeId, PackFormationError> {
        let mut flattened = Vec::new();
        for &segment in segments {
            self.collect_labeled_type_pack_concat_segments(segment, &mut flattened)?;
        }
        let mut normalized = Vec::new();
        let mut concrete = Vec::new();
        let mut labels = std::collections::HashSet::new();
        for segment in flattened {
            match self.get(segment).clone() {
                TypeData::LabeledTypePack(elements) => {
                    for element in elements.iter() {
                        if !labels.insert(element.label.clone()) {
                            return Err(PackFormationError::DuplicateLabel(element.label.to_string()));
                        }
                    }
                    concrete.extend(elements.iter().cloned());
                }
                _ => {
                    if !concrete.is_empty() {
                        normalized.push(self.labeled_type_pack(std::mem::take(&mut concrete).into_boxed_slice())?);
                    }
                    normalized.push(segment);
                }
            }
        }
        if !concrete.is_empty() {
            normalized.push(self.labeled_type_pack(concrete.into_boxed_slice())?);
        }
        match normalized.len() {
            0 => self.labeled_type_pack(Box::new([])),
            1 => Ok(normalized[0]),
            _ => Ok(self.intern_with_kind(TypeData::LabeledTypePackConcat(normalized.into_boxed_slice()), KindId::LABELED_TYPE_PACK)),
        }
    }

    fn collect_labeled_type_pack_concat_segments(&self, segment: TypeId, output: &mut Vec<TypeId>) -> Result<(), PackFormationError> {
        let kind = self.kind_of(segment);
        if kind != KindId::LABELED_TYPE_PACK {
            return Err(PackFormationError::InvalidPackConcatComponent {
                ty: segment,
                kind,
                expected: KindId::LABELED_TYPE_PACK,
            });
        }
        match self.get(segment) {
            TypeData::LabeledTypePackConcat(children) => {
                for &child in children.iter() {
                    self.collect_labeled_type_pack_concat_segments(child, output)?;
                }
            }
            TypeData::LabeledTypePack(elements) if elements.is_empty() => {}
            _ => output.push(segment),
        }
        Ok(())
    }

    /// Interns a tuple formed from symbolic positional and labeled packs.
    pub fn tuple_from_packs(&mut self, positionals: TypeId, labeled: TypeId) -> Result<TypeId, PackFormationError> {
        let pos_kind = self.kind_of(positionals);
        if pos_kind != KindId::TYPE_PACK {
            return Err(PackFormationError::InvalidTuplePackComponent {
                ty: positionals,
                kind: pos_kind,
            });
        }
        let lab_kind = self.kind_of(labeled);
        if lab_kind != KindId::LABELED_TYPE_PACK {
            return Err(PackFormationError::InvalidTuplePackComponent { ty: labeled, kind: lab_kind });
        }

        let pos_data = self.get(positionals).clone();
        let lab_data = self.get(labeled).clone();
        if let (TypeData::TypePack(pos_elems), TypeData::LabeledTypePack(lab_elems)) = (pos_data, lab_data) {
            if pos_elems.is_empty() && lab_elems.is_empty() {
                return Ok(self.unit());
            }
            let mut elements = Vec::with_capacity(pos_elems.len() + lab_elems.len());
            for &elem in pos_elems.iter() {
                elements.push(TupleTypeElement { label: None, ty: elem });
            }
            for elem in lab_elems.iter() {
                elements.push(TupleTypeElement {
                    label: Some(elem.label.clone()),
                    ty: elem.element,
                });
            }
            return Ok(self.tuple(elements.into_boxed_slice()));
        }

        Ok(self.intern_with_kind(TypeData::TupleFromPacks { positionals, labeled }, KindId::TYPE))
    }

    /// Interns a callable type formed from symbolic positional and labeled packs and a return type.
    pub fn callable_from_packs(&mut self, positionals: TypeId, labeled: TypeId, return_type: TypeId) -> Result<TypeId, PackFormationError> {
        let pos_kind = self.kind_of(positionals);
        if pos_kind != KindId::TYPE_PACK {
            return Err(PackFormationError::InvalidCallablePackComponent {
                ty: positionals,
                kind: pos_kind,
            });
        }
        let lab_kind = self.kind_of(labeled);
        if lab_kind != KindId::LABELED_TYPE_PACK {
            return Err(PackFormationError::InvalidCallablePackComponent { ty: labeled, kind: lab_kind });
        }
        let ret_kind = self.kind_of(return_type);
        if ret_kind != KindId::TYPE {
            return Err(PackFormationError::InvalidCallablePackComponent {
                ty: return_type,
                kind: ret_kind,
            });
        }

        let pos_data = self.get(positionals).clone();
        let lab_data = self.get(labeled).clone();
        if let (TypeData::TypePack(pos_elems), TypeData::LabeledTypePack(lab_elems)) = (pos_data, lab_data) {
            let mut parameters = Vec::with_capacity(pos_elems.len() + lab_elems.len());
            for &elem in pos_elems.iter() {
                parameters.push(CallableParameterType::new(elem));
            }
            for elem in lab_elems.iter() {
                parameters.push(CallableParameterType::new(elem.element).with_label(elem.label.clone()));
            }
            return Ok(self.callable(CallableType {
                parameters: parameters.into_boxed_slice(),
                return_type,
            }));
        }

        Ok(self.intern_with_kind(
            TypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            },
            KindId::TYPE,
        ))
    }

    /// Interns a record row into the store.
    pub fn record_row_checked(&mut self, fields: Vec<RecordRowField>, tail: RecordRowTail) -> Result<RecordRowId, crate::types::row::RecordRowFormationError> {
        for field in &fields {
            if !self.is_proper_type(field.ty) {
                return Err(crate::types::row::RecordRowFormationError::FieldNotProperType {
                    field: field.name.clone(),
                    ty: field.ty,
                });
            }
        }

        if let RecordRowTail::Parameter(parameter) = tail {
            let Some(data) = self.try_type_parameter(parameter) else {
                return Err(crate::types::row::RecordRowFormationError::TailParameterMissing(parameter));
            };
            if data.kind != KindId::RECORD_ROW {
                return Err(crate::types::row::RecordRowFormationError::TailParameterWrongKind { parameter, actual: data.kind });
            }
        }

        let row = RecordRowData::new_with_tail(fields, tail)
            .map_err(|crate::types::row::DuplicateFieldError(field)| crate::types::row::RecordRowFormationError::DuplicateField(field))?;
        Ok(self.intern_record_row(row))
    }

    pub fn record_row_type_checked(&mut self, fields: Vec<RecordRowField>, tail: RecordRowTail) -> Result<TypeId, crate::types::row::RecordRowFormationError> {
        let row = self.record_row_checked(fields, tail)?;
        Ok(self.record_type(row))
    }

    pub(crate) fn intern_record_row(&mut self, data: RecordRowData) -> RecordRowId {
        if let Some(&id) = self.row_interner.get(&data) {
            return id;
        }
        let id = RecordRowId(self.row_arena.len() as u32);
        self.row_arena.push(data.clone());
        self.row_interner.insert(data, id);
        id
    }

    /// Accesses a record row by its ID.
    #[inline]
    pub fn record_row(&self, id: RecordRowId) -> &RecordRowData {
        &self.row_arena[id.index()]
    }

    /// Looks up an already-interned record row.
    pub fn find_record_row(&self, data: &RecordRowData) -> Option<RecordRowId> {
        self.row_interner.get(data).copied()
    }

    /// Returns the number of interned record rows.
    #[inline]
    pub fn record_row_count(&self) -> usize {
        self.row_arena.len()
    }

    /// Interns a record type backed by a canonical closed row.
    pub fn record(&mut self, fields: Box<[RecordRowField]>) -> TypeId {
        self.record_row_type_checked(fields.into_vec(), RecordRowTail::Closed)
            .expect("internal closed Record construction must satisfy canonical row invariants")
    }

    /// Interns a record type from an already-interned row.
    pub(crate) fn record_type(&mut self, row_id: RecordRowId) -> TypeId {
        self.intern_with_kind(TypeData::Record(row_id), KindId::TYPE)
    }

    /// Interns a callable type.
    pub fn callable(&mut self, callable: CallableType) -> TypeId {
        for param in callable.parameters.iter() {
            debug_assert!(self.is_proper_type(param.ty), "callable parameter must be a proper type");
        }
        debug_assert!(self.is_proper_type(callable.return_type), "callable return type must be a proper type");
        self.intern_with_kind(TypeData::Callable(callable), KindId::TYPE)
    }

    /// Interns a structural associated family type.
    pub fn family_type(&mut self, members: impl IntoIterator<Item = FamilyMemberType>) -> Result<TypeId, FamilyTypeError> {
        let member_vec: Vec<FamilyMemberType> = members.into_iter().collect();
        for member in &member_vec {
            if member.member_kind == FamilyMemberTypeKind::Callable && !matches!(self.get(member.ty), TypeData::Callable(_)) {
                return Err(FamilyTypeError::CallableMemberNotCallable {
                    operation: member.operation.clone(),
                    ty: member.ty,
                });
            }
        }

        let mut sorted_members = member_vec;
        sorted_members.sort_by(|a, b| a.operation.cmp(&b.operation));

        let mut deduped: Vec<FamilyMemberType> = Vec::with_capacity(sorted_members.len());
        for member in sorted_members {
            if let Some(last) = deduped.last() {
                if last.operation == member.operation {
                    if last == &member {
                        continue;
                    } else {
                        return Err(FamilyTypeError::DuplicateOperationShape { operation: member.operation });
                    }
                }
            }
            deduped.push(member);
        }

        let family = FamilyType::new(deduped.into_boxed_slice());
        let family_id = if let Some(&id) = self.family_interner.get(&family) {
            id
        } else {
            let id = FamilyTypeId::new(self.family_arena.len() as u32);
            self.family_arena.push(family.clone());
            self.family_interner.insert(family, id);
            id
        };

        Ok(self.intern_with_kind(TypeData::Family(family_id), KindId::TYPE))
    }

    #[inline]
    pub fn get_family(&self, id: FamilyTypeId) -> &FamilyType {
        &self.family_arena[id.index()]
    }

    pub fn family_count(&self) -> usize {
        self.family_arena.len()
    }

    /// Interns a `List<T>` applied type.
    pub fn list_of(&mut self, list_form: TypeId, element: TypeId) -> Result<TypeId, TypeApplicationError> {
        self.apply_type_form(list_form, &[element])
    }

    /// Interns a `Map<K, V>` applied type.
    pub fn map_of(&mut self, map_form: TypeId, key: TypeId, value: TypeId) -> Result<TypeId, TypeApplicationError> {
        self.apply_type_form(map_form, &[key, value])
    }

    /// Interns a `Set<T>` applied type.
    pub fn set_of(&mut self, set_form: TypeId, element: TypeId) -> Result<TypeId, TypeApplicationError> {
        self.apply_type_form(set_form, &[element])
    }

    /// Normalizes and interns a union type.
    ///
    /// Rules:
    /// 1. Flatten nested unions.
    /// 2. Remove duplicates.
    /// 3. Remove `Never` when other members exist.
    /// 4. Sort canonically by TypeId.
    /// 5. Zero members -> `Never`.
    /// 6. One member -> member TypeId.
    pub fn union(&mut self, members: &[TypeId]) -> TypeId {
        for &m in members {
            debug_assert!(self.is_proper_type(m), "union member must be a proper type");
        }
        let mut flattened = Vec::new();
        self.collect_union_members(members, &mut flattened);

        // Deduplicate and sort
        flattened.sort_unstable_by_key(|t| t.0);
        flattened.dedup();

        // Remove Never if other types present
        if flattened.len() > 1 {
            flattened.retain(|&t| t != self.never_id);
        }

        match flattened.len() {
            0 => self.never_id,
            1 => flattened[0],
            _ => self.intern_with_kind(TypeData::Union(flattened.into_boxed_slice()), KindId::TYPE),
        }
    }

    fn collect_union_members(&self, members: &[TypeId], out: &mut Vec<TypeId>) {
        for &m in members {
            match self.get(m) {
                TypeData::Union(nested) => self.collect_union_members(nested, out),
                _ => out.push(m),
            }
        }
    }

    /// Formats a canonical TypeId into human-readable type syntax.
    pub fn format_type(&self, ty: TypeId) -> String {
        match self.get(ty) {
            TypeData::Never => "Never".to_string(),
            TypeData::Unit => "Unit".to_string(),
            TypeData::ConstInt(value) => format!("{value}#const"),
            TypeData::StaticData(term) => format!("StaticData({:?}, {:?})", term.domain, term.constructor),
            TypeData::ClassObject { declaration } => format!("class {}", declaration.name),
            TypeData::Nominal { declaration } => declaration.name.to_string(),
            TypeData::Applied { origin, arguments } => {
                let orig_str = self.format_type(*origin);
                let args_str = arguments
                    .iter()
                    .map(|argument| match argument {
                        StaticArgument::Term(argument) => self.format_type(*argument),
                        StaticArgument::RecordRow(row) => format!("{row:?}"),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{orig_str}<{args_str}>")
            }
            TypeData::Union(types) => types.iter().map(|&t| self.format_type(t)).collect::<Vec<_>>().join(" | "),
            TypeData::Tuple(elements) => {
                let elems = elements
                    .iter()
                    .map(|elem| {
                        let t_str = self.format_type(elem.ty);
                        if let Some(ref l) = elem.label { format!("{l}: {t_str}") } else { t_str }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({elems})")
            }
            TypeData::Record(row_id) => {
                let row = &self.row_arena[row_id.index()];
                let fields = row
                    .fields
                    .iter()
                    .map(|f| format!("{}: {}", f.name, self.format_type(f.ty)))
                    .collect::<Vec<_>>()
                    .join(", ");
                match row.tail {
                    RecordRowTail::Closed => format!("{{{fields}}}"),
                    RecordRowTail::Parameter(p) => {
                        let p_name = &self.type_parameters[p.index()].name;
                        if fields.is_empty() {
                            format!("{{..{p_name}}}")
                        } else {
                            format!("{{{fields}, ..{p_name}}}")
                        }
                    }
                    RecordRowTail::Projection(proj) => {
                        let proj_name = self.format_type(proj);
                        if fields.is_empty() {
                            format!("{{..{proj_name}}}")
                        } else {
                            format!("{{{fields}, ..{proj_name}}}")
                        }
                    }
                }
            }
            TypeData::Callable(callable) => {
                let params = callable
                    .parameters
                    .iter()
                    .map(|p| {
                        let t_str = self.format_type(p.ty);
                        let prefix = match p.rest {
                            RestMode::None => "",
                            RestMode::Positional => "...",
                            RestMode::Labeled => "...#",
                            RestMode::Complete => "...*",
                        };
                        if let Some(ref l) = p.label {
                            format!("{prefix}{l}: {t_str}")
                        } else {
                            format!("{prefix}{t_str}")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret = self.format_type(callable.return_type);
                format!("({params}) -> {ret}")
            }
            TypeData::Family(fid) => {
                let family = &self.family_arena[fid.index()];
                let member_strs = family
                    .members
                    .iter()
                    .map(|m| {
                        let kind_str = match m.operation.kind {
                            SelectorKind::Method => "method",
                            SelectorKind::Getter => "getter",
                            SelectorKind::Setter => "setter",
                            SelectorKind::SubscriptGet => "subscript_get",
                            SelectorKind::SubscriptSet => "subscript_set",
                        };
                        let mut slots_str = String::new();
                        if m.operation.kind == SelectorKind::Method {
                            let slots = m
                                .operation
                                .slots
                                .iter()
                                .map(|s| match s {
                                    SelectorSlot::Positional => "_",
                                    SelectorSlot::Label(l) => l.as_str(),
                                })
                                .collect::<Vec<_>>()
                                .join(", ");
                            slots_str = format!("({slots})");
                        }
                        let t_str = self.format_type(m.ty);
                        format!("{kind_str}{slots_str}: {t_str}")
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                format!("family {{{member_strs}}}")
            }
            TypeData::ExactCase { variant, enum_type } => {
                let variant_id = self.variant_identity(*variant);
                let enum_str = self.format_type(*enum_type);
                format!("ExactCase<{}::{}, {enum_str}>", variant_id.owner.name, variant_id.selector.encode())
            }
            TypeData::Parameter(param_id) => self.type_parameters[param_id.index()].name.to_string(),
            TypeData::Lambda(_) => "[TypeLambda]".to_string(),
            TypeData::SelfType(_) => "Self".to_string(),
            TypeData::TypePack(elements) => {
                let elems = elements.iter().map(|&t| self.format_type(t)).collect::<Vec<_>>().join(", ");
                format!("[{elems}]")
            }
            TypeData::LabeledTypePack(elements) => {
                let elems = elements
                    .iter()
                    .map(|e| format!("{}: {}", e.label, self.format_type(e.element)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{elems}]")
            }
            TypeData::CompleteTypePack { positionals, labeled } => {
                format!("CompleteTypePack({}, {})", self.format_type(*positionals), self.format_type(*labeled))
            }
            TypeData::TypePackConcat(segments) => {
                let segments = segments.iter().map(|&segment| self.format_type(segment)).collect::<Vec<_>>().join(" ++ ");
                format!("TypePackConcat({segments})")
            }
            TypeData::LabeledTypePackConcat(segments) => {
                let segments = segments.iter().map(|&segment| self.format_type(segment)).collect::<Vec<_>>().join(" ++ ");
                format!("LabeledTypePackConcat({segments})")
            }
            TypeData::TupleFromPacks { positionals, labeled } => {
                format!("TupleFromPacks({}, {})", self.format_type(*positionals), self.format_type(*labeled))
            }
            TypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                format!(
                    "CallableFromPacks({}, {}, {})",
                    self.format_type(*positionals),
                    self.format_type(*labeled),
                    self.format_type(*return_type)
                )
            }
            TypeData::AssociatedProjection(projection) => {
                format!(
                    "<{} as {}>::{}",
                    self.format_type(projection.subject),
                    projection.trait_ref.declaration.name,
                    projection.requirement.index
                )
            }
            TypeData::TypeMemberProjection(projection) => {
                let subject_str = match &projection.subject {
                    super::type_member::ProjectionSubject::Type(ty) => self.format_type(*ty),
                    super::type_member::ProjectionSubject::RecordRow(r) => format!("{r:?}"),
                };
                let args_str = if projection.arguments.is_empty() {
                    String::new()
                } else {
                    let formatted_args = projection
                        .arguments
                        .iter()
                        .map(|arg| match arg {
                            super::type_member::TypeMemberArgument::Type(ty) => self.format_type(*ty),
                            super::type_member::TypeMemberArgument::Const(super::type_member::ConstTerm::Int(i)) => i.to_string(),
                            super::type_member::TypeMemberArgument::RecordRow(r) => format!("{r:?}"),
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("<{formatted_args}>")
                };
                format!("{subject_str}::{}{args_str}", projection.selector.name())
            }
        }
    }

    /// Interns a [`VariantId`] into a compact store-relative [`VariantTypeId`].
    pub fn intern_variant_identity(&mut self, variant: VariantId) -> VariantTypeId {
        if let Some(&id) = self.variant_identity_to_id.get(&variant) {
            return id;
        }
        let id = VariantTypeId::from_index(self.variant_identities.len());
        self.variant_identities.push(variant.clone());
        self.variant_identity_to_id.insert(variant, id);
        id
    }

    /// Returns the stable [`VariantId`] corresponding to a [`VariantTypeId`].
    pub fn variant_identity(&self, id: VariantTypeId) -> &VariantId {
        &self.variant_identities[id.index()]
    }

    /// Returns nominal origin declaration of `ty` if it is a nominal type or applied generic nominal type.
    pub fn nominal_origin_declaration(&self, ty: TypeId) -> Option<&DeclarationId> {
        match self.get(ty) {
            TypeData::Nominal { declaration } => Some(declaration),
            TypeData::Applied { origin, .. } => self.nominal_origin_declaration(*origin),
            TypeData::ExactCase { enum_type, .. } => self.nominal_origin_declaration(*enum_type),
            _ => None,
        }
    }

    /// Decomposes a nominal or applied nominal type into `(declaration, type_arguments)`.
    pub fn applied_nominal_parts(&self, ty: TypeId) -> Option<(DeclarationId, Vec<TypeId>)> {
        let (declaration, arguments) = self.applied_nominal_static_parts(ty)?;
        let arguments = arguments.into_iter().map(StaticArgument::as_type_id).collect::<Option<Vec<_>>>()?;
        Some((declaration, arguments))
    }

    /// Decomposes a nominal application without erasing record-row arguments.
    pub fn applied_nominal_static_parts(&self, ty: TypeId) -> Option<(DeclarationId, Vec<StaticArgument>)> {
        match self.get(ty) {
            TypeData::Nominal { declaration } => Some((declaration.clone(), Vec::new())),
            TypeData::Applied { origin, arguments } => {
                let decl = self.nominal_origin_declaration(*origin)?;
                Some((decl.clone(), arguments.to_vec()))
            }
            TypeData::ExactCase { enum_type, .. } => self.applied_nominal_static_parts(*enum_type),
            _ => None,
        }
    }

    /// Validates and interns a canonical exact static case type `ExactCase(variant, enum_type)`.
    pub fn exact_case_type(&mut self, variant: &VariantId, mut enum_type: TypeId) -> Result<TypeId, ExactCaseTypeError> {
        while let TypeData::ExactCase { enum_type: inner, .. } = self.get(enum_type) {
            enum_type = *inner;
        }
        let enum_kind = self.kind_of(enum_type);
        if enum_kind != KindId::TYPE {
            return Err(ExactCaseTypeError::EnumTypeMalformed);
        }
        let Some(origin) = self.nominal_origin_declaration(enum_type) else {
            return Err(ExactCaseTypeError::NominalOriginMissing);
        };
        if *origin != variant.owner {
            return Err(ExactCaseTypeError::WrongOwner {
                expected: variant.owner.clone(),
                got: origin.clone(),
            });
        }
        let variant_type_id = self.intern_variant_identity(variant.clone());
        Ok(self.intern_with_kind(
            TypeData::ExactCase {
                variant: variant_type_id,
                enum_type,
            },
            KindId::TYPE,
        ))
    }

    /// Checks if a type contains a specific type parameter.
    pub fn contains_type_parameter(&self, ty: TypeId, target: TypeParameterId) -> bool {
        match self.get(ty) {
            TypeData::Parameter(p) => *p == target,
            TypeData::Applied { origin, arguments } => {
                self.contains_type_parameter(*origin, target)
                    || arguments.iter().any(|argument| argument.as_type_id().is_some_and(|a| self.contains_type_parameter(a, target)))
            }
            TypeData::Union(members) => members.iter().any(|&m| self.contains_type_parameter(m, target)),
            TypeData::Tuple(elems) => elems.iter().any(|e| self.contains_type_parameter(e.ty, target)),
            TypeData::Record(row_id) => {
                let row = self.record_row(*row_id);
                row.fields.iter().any(|f| self.contains_type_parameter(f.ty, target))
                    || matches!(row.tail, RecordRowTail::Parameter(parameter) if parameter == target)
            }
            TypeData::Callable(call) => {
                call.parameters.iter().any(|p| self.contains_type_parameter(p.ty, target)) || self.contains_type_parameter(call.return_type, target)
            }
            TypeData::ExactCase { enum_type, .. } => self.contains_type_parameter(*enum_type, target),
            TypeData::AssociatedProjection(projection) => {
                self.contains_type_parameter(projection.subject, target)
                    || projection
                        .trait_ref
                        .arguments
                        .iter()
                        .any(|&argument| self.contains_type_parameter(argument, target))
            }
            TypeData::TypeMemberProjection(projection) => {
                let subject_has = match &projection.subject {
                    super::type_member::ProjectionSubject::Type(ty) => self.contains_type_parameter(*ty, target),
                    super::type_member::ProjectionSubject::RecordRow(row_id) => {
                        let row = self.record_row(*row_id);
                        row.fields.iter().any(|f| self.contains_type_parameter(f.ty, target)) || matches!(row.tail, RecordRowTail::Parameter(p) if p == target)
                    }
                };
                subject_has
                    || projection.arguments.iter().any(|arg| match arg {
                        super::type_member::TypeMemberArgument::Type(ty) => self.contains_type_parameter(*ty, target),
                        super::type_member::TypeMemberArgument::RecordRow(row_id) => {
                            let row = self.record_row(*row_id);
                            row.fields.iter().any(|f| self.contains_type_parameter(f.ty, target))
                                || matches!(row.tail, RecordRowTail::Parameter(p) if p == target)
                        }
                        super::type_member::TypeMemberArgument::Const(_) => false,
                    })
            }
            TypeData::CompleteTypePack { positionals, labeled } => {
                self.contains_type_parameter(*positionals, target) || self.contains_type_parameter(*labeled, target)
            }
            TypeData::TypePackConcat(segments) | TypeData::LabeledTypePackConcat(segments) => {
                segments.iter().any(|&segment| self.contains_type_parameter(segment, target))
            }
            _ => false,
        }
    }

    /// Formats TypeKnowledge into human-readable type syntax.
    pub fn format_knowledge(&self, knowledge: &crate::types::evidence::TypeKnowledge) -> String {
        match knowledge {
            crate::types::evidence::TypeKnowledge::Known(ev) => self.format_type(ev.ty()),
            crate::types::evidence::TypeKnowledge::Dynamic(_) => "Dynamic".to_string(),
            crate::types::evidence::TypeKnowledge::Unknown(_) => "Unknown".to_string(),
        }
    }
}

/// Error returned when constructing an invalid exact-case static type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExactCaseTypeError {
    EnumTypeMalformed,
    NominalOriginMissing,
    WrongOwner { expected: DeclarationId, got: DeclarationId },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::DispatchSide;
    use crate::types::environment::{TypeEnvironment, TypeView};
    use crate::types::parameter::{SelfRole, SelfTypeTerm};
    use phalcom_modules::identity::ModuleId;

    fn test_decl(name: &str) -> DeclarationId {
        let module = ModuleId::universe_root();
        DeclarationId::new(module, name.into())
    }

    #[test]
    fn nominal_interning_is_canonical() {
        let mut store = TypeStore::new();
        let decl = test_decl("User");
        let t1 = store.nominal(decl.clone());
        let t2 = store.nominal(decl);
        assert_eq!(t1, t2);
    }

    #[test]
    fn union_normalization_flattens_deduplicates_and_sorts() {
        let mut store = TypeStore::new();
        let d_a = test_decl("A");
        let d_b = test_decl("B");
        let d_c = test_decl("C");
        let t_a = store.nominal(d_a);
        let t_b = store.nominal(d_b);
        let t_c = store.nominal(d_c);

        let u1 = store.union(&[t_a, t_b]);
        let u2 = store.union(&[t_b, t_a]);
        assert_eq!(u1, u2, "union order invariant");

        let u3 = store.union(&[u1, t_c]);
        let u4 = store.union(&[t_c, t_a, t_b, t_a]);
        assert_eq!(u3, u4, "nested and duplicated union invariant");

        let with_never = store.union(&[t_a, store.never()]);
        assert_eq!(with_never, t_a, "never removed from nonempty union");

        let empty_union = store.union(&[]);
        assert_eq!(empty_union, store.never(), "empty union is never");
    }

    #[test]
    fn empty_tuple_normalizes_to_unit() {
        let mut store = TypeStore::new();
        assert_eq!(store.tuple(Box::new([])), store.unit());
    }

    #[test]
    fn associated_projection_interning_uses_trait_and_requirement_identity() {
        let mut store = TypeStore::new();
        let subject = store.nominal_type(test_decl("Subject"));
        let first_trait = test_decl("First");
        let second_trait = test_decl("Second");
        let first_requirement = AssociatedTypeRequirementId::new(first_trait.clone(), 0);
        let second_requirement = AssociatedTypeRequirementId::new(second_trait.clone(), 0);
        let first_ref = TraitRef::new(first_trait, Vec::<TypeId>::new().into_boxed_slice());
        let second_ref = TraitRef::new(second_trait, Vec::<TypeId>::new().into_boxed_slice());

        let first = store.associated_projection(subject, first_ref.clone(), first_requirement.clone());
        assert_eq!(first, store.associated_projection(subject, first_ref, first_requirement));
        assert_ne!(first, store.associated_projection(subject, second_ref, second_requirement));
        assert_eq!(store.kind_of(first), KindId::TYPE);
    }

    #[test]
    fn associated_projection_structural_materialization_rewrites_subject_only() {
        let mut store = TypeStore::new();
        let owner = test_decl("Iterator");
        let self_type = store.self_type(SelfTypeTerm {
            owner: owner.clone(),
            side: DispatchSide::Instance,
            role: SelfRole::InstanceType,
        });
        let subject = store.nominal_type(test_decl("List"));
        let requirement = AssociatedTypeRequirementId::new(owner.clone(), 0);
        let projection = store.associated_projection(self_type, TraitRef::new(owner, Vec::<TypeId>::new().into_boxed_slice()), requirement.clone());
        let nested = store.tuple(vec![TupleTypeElement { label: None, ty: projection }].into_boxed_slice());
        let mut environment = TypeEnvironment::new();
        environment.bind_self(subject);
        let materialized = TypeView::new(nested, environment).materialize(&mut store);
        let TypeData::Tuple(elements) = store.get(materialized) else {
            panic!("expected tuple")
        };
        let TypeData::AssociatedProjection(projected) = store.get(elements[0].ty) else {
            panic!("expected projection")
        };
        assert_eq!(projected.subject, subject);
        assert_eq!(projected.requirement, requirement);
    }

    #[test]
    fn const_int_is_canonical_static_data_and_not_a_proper_type() {
        let mut store = TypeStore::new();

        let first = store.const_int(7);
        let second = store.const_int(7);
        assert_eq!(first, second);
        assert_eq!(store.kind_of(first), store.static_domain_kind(store.const_int_domain()));
        assert!(matches!(store.get_kind(store.kind_of(first)), KindData::StaticDomain(_)));
        assert_eq!(store.proper_type(first), Err(store.kind_of(first)));
    }

    #[test]
    fn nominal_static_domains_and_constructors_are_canonical_and_validated() {
        let mut store = TypeStore::new();
        let declaration = test_decl("Shape");
        let other_declaration = test_decl("OtherShape");
        let domain = store.nominal_static_domain(declaration.clone());
        let same_domain = store.nominal_static_domain(declaration.clone());
        let other_domain = store.nominal_static_domain(other_declaration);
        assert_eq!(domain, same_domain);
        assert_ne!(domain, other_domain);

        let const_int = store.const_int(3);
        let parameter_kinds = vec![store.kind_of(const_int)].into_boxed_slice();
        let constructor = store.intern_static_constructor(domain, declaration, "Point", parameter_kinds);
        let same_constructor = store.intern_static_constructor(domain, test_decl("Shape"), "Point", vec![store.kind_of(const_int)].into_boxed_slice());
        assert_eq!(constructor, same_constructor);

        let formed = store
            .static_data(domain, constructor, vec![StaticArgument::Term(const_int)].into_boxed_slice())
            .expect("matching static constructor arguments should form");
        let formed_again = store
            .static_data(domain, constructor, vec![StaticArgument::Term(const_int)].into_boxed_slice())
            .expect("matching static constructor arguments should be canonical");
        assert_eq!(formed, formed_again);
        assert!(matches!(store.get(formed), TypeData::StaticData(_)));
        assert!(matches!(store.get_kind(store.kind_of(formed)), KindData::StaticDomain(_)));

        assert!(matches!(
            store.static_data(domain, constructor, Box::new([])),
            Err(StaticDataFormationError::ArityMismatch { expected: 1, actual: 0 })
        ));
        assert!(matches!(
            store.static_data(other_domain, constructor, vec![StaticArgument::Term(const_int)].into_boxed_slice()),
            Err(StaticDataFormationError::DomainMismatch { .. })
        ));
        let runtime_type = store.nominal_type(test_decl("Runtime"));
        assert!(matches!(
            store.static_data(domain, constructor, vec![StaticArgument::Term(runtime_type)].into_boxed_slice()),
            Err(StaticDataFormationError::ArgumentKindMismatch { index: 0, .. })
        ));
    }

    #[test]
    fn associated_static_domain_identity_is_stable_and_specialization_is_canonical() {
        let mut store = TypeStore::new();
        let declaration = AssociatedStaticDomainDeclarationId::new(test_decl("Container"), "State");
        let int = store.nominal_type(test_decl("Int"));
        let string = store.nominal_type(test_decl("String"));

        let int_domain = store.associated_static_domain(
            declaration.clone(),
            vec![StaticArgument::Term(int)].into_boxed_slice(),
        );
        let same_int_domain = store.associated_static_domain(
            declaration.clone(),
            vec![StaticArgument::Term(int)].into_boxed_slice(),
        );
        let string_domain = store.associated_static_domain(
            declaration.clone(),
            vec![StaticArgument::Term(string)].into_boxed_slice(),
        );

        assert_eq!(int_domain, same_int_domain);
        assert_ne!(int_domain, string_domain);
        assert_eq!(
            store.static_domain(int_domain),
            Some(&StaticDomainData::Associated {
                declaration: declaration.clone(),
                owner_arguments: vec![StaticArgument::Term(int)].into_boxed_slice(),
            })
        );
        assert_eq!(declaration.owner.name.as_ref(), "Container");
        assert_eq!(declaration.name.as_ref(), "State");

        let owner = test_decl("RowOwner");
        let kind = store.arrow_kind(vec![KindId::TYPE, KindId::RECORD_ROW].into_boxed_slice(), KindId::TYPE);
        let form = store.nominal_form(owner.clone(), kind);
        let row = store.record_row_checked(Vec::new(), RecordRowTail::Closed).expect("closed row");
        let applied = store
            .apply_static_form(form, &[StaticArgument::Term(int), StaticArgument::RecordRow(row)])
            .expect("heterogeneous owner application");
        assert_eq!(
            store.applied_nominal_static_parts(applied),
            Some((owner, vec![StaticArgument::Term(int), StaticArgument::RecordRow(row)]))
        );
    }
}
