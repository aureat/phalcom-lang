//! Portable type-graph projection and relocation at the TypeStore ownership boundary.
use super::*;
use crate::field_key::FieldKey;
use crate::identity::{AssociatedStaticDomainDeclarationId, CallableId, CallableOwnerId};
use crate::metadata::stable_identity::*;
use crate::types::family::FamilyOperationShape;
use crate::types::id::ScopedTypeId;
use crate::types::parameter::{GenericParameterMode, SelfRole};
use crate::types::type_lambda::*;
use crate::types::type_member::{ConstTerm, TypeMemberArgument};
use phalcom_common::selector::{Selector, SelectorBase};
use phalcom_compiler_artifact::semantic::type_store::*;
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_type_meta::allocation::{AllocationBudget, BudgetedSortedMap};
use phalcom_type_meta::identity::*;
use std::collections::{BTreeMap, BTreeSet};

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeStoreLineageId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeArtifactError {
    ForeignStoreLineage { expected: TypeStoreLineageId, actual: TypeStoreLineageId },
    GraphAlreadyClosed,
    GraphNotClosed,
    DuplicateRootKey(CanonicalTypeRootKey),
    InvalidNativeCoordinate { arena: &'static str, raw: u32 },
    InvalidWireCoordinate { arena: &'static str, raw: u32 },
}

impl std::fmt::Display for TypeArtifactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ForeignStoreLineage { expected, actual } => {
                write!(f, "foreign store lineage: expected {expected:?}, actual {actual:?}")
            }
            Self::GraphAlreadyClosed => write!(f, "type export graph is already closed"),
            Self::GraphNotClosed => write!(f, "type export graph is not closed"),
            Self::DuplicateRootKey(key) => write!(f, "type export root key has conflicting values: {key:?}"),
            Self::InvalidNativeCoordinate { arena, raw } => {
                write!(f, "invalid native coordinate for {arena}: {raw}")
            }
            Self::InvalidWireCoordinate { arena, raw } => {
                write!(f, "invalid wire coordinate for {arena}: {raw}")
            }
        }
    }
}

impl std::error::Error for TypeArtifactError {}

impl From<TypeArtifactError> for ArtifactError {
    fn from(err: TypeArtifactError) -> Self {
        ValidationError::ReferentialIntegrity(err.to_string()).into()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TypeGraphRoot {
    Type(TypeId),
    Kind(KindId),
    Parameter(TypeParameterId),
    Row(RecordRowId),
    Family(FamilyTypeId),
    Variant(VariantTypeId),
    Domain(StaticDomainId),
    Constructor(AlgebraicStaticConstructorId),
    Scoped(ScopedTypeId),
    Lambda(TypeLambdaId),
}

/// Guarded export context shared across all artifact sections.
pub struct TypeExportContext {
    expected_lineage: TypeStoreLineageId,
    closed: bool,
    ids: Option<ExportIds>,
    sorted_roots: BTreeMap<CanonicalTypeRootKey, TypeGraphRoot>,
}

impl TypeExportContext {
    pub fn new(store: &TypeStore) -> Self {
        Self {
            expected_lineage: TypeStoreLineageId(store.id().raw()),
            closed: false,
            ids: None,
            sorted_roots: BTreeMap::new(),
        }
    }

    pub fn new_full(store: &TypeStore) -> Result<Self, ArtifactError> {
        let ids = ExportIds::new_full(store)?;
        Ok(Self {
            expected_lineage: TypeStoreLineageId(store.id().raw()),
            closed: true,
            ids: Some(ids),
            sorted_roots: BTreeMap::new(),
        })
    }

    pub fn expected_lineage(&self) -> TypeStoreLineageId {
        self.expected_lineage
    }

    pub fn verify_lineage(&self, lineage: TypeStoreLineageId) -> Result<(), TypeArtifactError> {
        if self.expected_lineage != lineage {
            return Err(TypeArtifactError::ForeignStoreLineage {
                expected: self.expected_lineage,
                actual: lineage,
            });
        }
        Ok(())
    }

    pub fn verify_store(&self, store: &TypeStore) -> Result<(), TypeArtifactError> {
        self.verify_lineage(TypeStoreLineageId(store.id().raw()))
    }

    fn register_root(&mut self, lineage: TypeStoreLineageId, key: CanonicalTypeRootKey, root: TypeGraphRoot) -> Result<(), TypeArtifactError> {
        self.verify_lineage(lineage)?;
        if self.closed {
            return Err(TypeArtifactError::GraphAlreadyClosed);
        }
        if let Some(previous) = self.sorted_roots.get(&key) {
            if *previous != root {
                return Err(TypeArtifactError::DuplicateRootKey(key));
            }
        } else {
            self.sorted_roots.insert(key, root);
        }
        Ok(())
    }

    pub fn register_type_root(&mut self, lineage: TypeStoreLineageId, key: CanonicalTypeRootKey, value: TypeId) -> Result<(), TypeArtifactError> {
        self.register_root(lineage, key, TypeGraphRoot::Type(value))
    }

    pub fn register_kind_root(&mut self, lineage: TypeStoreLineageId, key: CanonicalTypeRootKey, value: KindId) -> Result<(), TypeArtifactError> {
        self.register_root(lineage, key, TypeGraphRoot::Kind(value))
    }

    pub fn register_parameter_root(&mut self, lineage: TypeStoreLineageId, key: CanonicalTypeRootKey, value: TypeParameterId) -> Result<(), TypeArtifactError> {
        self.register_root(lineage, key, TypeGraphRoot::Parameter(value))
    }

    pub fn register_row_root(&mut self, lineage: TypeStoreLineageId, key: CanonicalTypeRootKey, value: RecordRowId) -> Result<(), TypeArtifactError> {
        self.register_root(lineage, key, TypeGraphRoot::Row(value))
    }

    pub fn register_family_root(&mut self, lineage: TypeStoreLineageId, key: CanonicalTypeRootKey, value: FamilyTypeId) -> Result<(), TypeArtifactError> {
        self.register_root(lineage, key, TypeGraphRoot::Family(value))
    }

    pub fn register_variant_root(&mut self, lineage: TypeStoreLineageId, key: CanonicalTypeRootKey, value: VariantTypeId) -> Result<(), TypeArtifactError> {
        self.register_root(lineage, key, TypeGraphRoot::Variant(value))
    }

    pub fn register_domain_root(&mut self, lineage: TypeStoreLineageId, key: CanonicalTypeRootKey, value: StaticDomainId) -> Result<(), TypeArtifactError> {
        self.register_root(lineage, key, TypeGraphRoot::Domain(value))
    }

    pub fn register_constructor_root(
        &mut self,
        lineage: TypeStoreLineageId,
        key: CanonicalTypeRootKey,
        value: AlgebraicStaticConstructorId,
    ) -> Result<(), TypeArtifactError> {
        self.register_root(lineage, key, TypeGraphRoot::Constructor(value))
    }

    pub fn register_scoped_root(&mut self, lineage: TypeStoreLineageId, key: CanonicalTypeRootKey, value: ScopedTypeId) -> Result<(), TypeArtifactError> {
        self.register_root(lineage, key, TypeGraphRoot::Scoped(value))
    }

    pub fn register_lambda_root(&mut self, lineage: TypeStoreLineageId, key: CanonicalTypeRootKey, value: TypeLambdaId) -> Result<(), TypeArtifactError> {
        self.register_root(lineage, key, TypeGraphRoot::Lambda(value))
    }

    pub fn close_graph(&mut self, store: &TypeStore, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        self.verify_store(store)?;
        if self.closed {
            return Err(TypeArtifactError::GraphAlreadyClosed.into());
        }
        let ids = ExportIds::from_roots(store, &self.sorted_roots, limits)?;
        self.ids = Some(ids);
        self.closed = true;
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    fn ids(&self) -> Result<&ExportIds, ArtifactError> {
        self.ids.as_ref().ok_or_else(|| TypeArtifactError::GraphNotClosed.into())
    }

    pub fn project_graph(&self, store: &TypeStore, limits: &ValidationLimits) -> Result<TypeGraphV1, ArtifactError> {
        self.verify_store(store)?;
        store.project_artifact_with_ids(self.ids()?, limits)
    }

    pub fn ty(&self, ty: TypeId) -> Result<ArtifactTypeId, ArtifactError> {
        self.ids()?.ty(ty)
    }

    pub fn kind(&self, kind: KindId) -> Result<ArtifactKindId, ArtifactError> {
        self.ids()?.kind(kind)
    }

    pub fn parameter(&self, param: TypeParameterId) -> Result<ArtifactTypeParameterId, ArtifactError> {
        self.ids()?.parameter(param)
    }

    pub fn row(&self, row: RecordRowId) -> Result<ArtifactRecordRowId, ArtifactError> {
        self.ids()?.row(row)
    }

    pub fn family(&self, family: FamilyTypeId) -> Result<ArtifactFamilyTypeId, ArtifactError> {
        self.ids()?.family(family)
    }

    pub fn variant(&self, variant: VariantTypeId) -> Result<ArtifactVariantTypeId, ArtifactError> {
        self.ids()?.variant(variant)
    }

    pub fn domain(&self, domain: StaticDomainId) -> Result<ArtifactStaticDomainId, ArtifactError> {
        self.ids()?.domain(domain)
    }

    pub fn constructor(&self, constructor: AlgebraicStaticConstructorId) -> Result<ArtifactStaticConstructorId, ArtifactError> {
        self.ids()?.constructor(constructor)
    }

    pub fn scoped(&self, scoped: ScopedTypeId) -> Result<ArtifactScopedTypeId, ArtifactError> {
        self.ids()?.scoped(scoped)
    }

    pub fn lambda(&self, lambda: TypeLambdaId) -> Result<ArtifactTypeLambdaId, ArtifactError> {
        self.ids()?.lambda(lambda)
    }

    pub fn args(&self, values: &[StaticArgument]) -> Result<Vec<StaticArgumentV1>, ArtifactError> {
        self.ids()?.args(values)
    }

    pub fn types(&self, values: &[TypeId]) -> Result<Vec<ArtifactTypeId>, ArtifactError> {
        self.ids()?.types(values)
    }

    pub fn scopes(&self, values: &[ScopedTypeId]) -> Result<Vec<ArtifactScopedTypeId>, ArtifactError> {
        self.ids()?.scopes(values)
    }
}

struct ExportIds {
    types: BTreeMap<TypeId, ArtifactTypeId>,
    kinds: BTreeMap<KindId, ArtifactKindId>,
    parameters: BTreeMap<TypeParameterId, ArtifactTypeParameterId>,
    rows: BTreeMap<RecordRowId, ArtifactRecordRowId>,
    families: BTreeMap<FamilyTypeId, ArtifactFamilyTypeId>,
    variants: BTreeMap<VariantTypeId, ArtifactVariantTypeId>,
    domains: BTreeMap<StaticDomainId, ArtifactStaticDomainId>,
    constructors: BTreeMap<AlgebraicStaticConstructorId, ArtifactStaticConstructorId>,
    scoped: BTreeMap<super::super::id::ScopedTypeId, ArtifactScopedTypeId>,
    lambdas: BTreeMap<TypeLambdaId, ArtifactTypeLambdaId>,
    type_order: Vec<TypeId>,
    kind_order: Vec<KindId>,
    parameter_order: Vec<TypeParameterId>,
    row_order: Vec<RecordRowId>,
    family_order: Vec<FamilyTypeId>,
    variant_order: Vec<VariantTypeId>,
    domain_order: Vec<StaticDomainId>,
    constructor_order: Vec<AlgebraicStaticConstructorId>,
    scoped_order: Vec<ScopedTypeId>,
    lambda_order: Vec<TypeLambdaId>,
}
macro_rules! export_map {
    ($count:expr, $native:ident, $wire:ident) => {
        (0..$count)
            .map(|index| {
                let index = u32::try_from(index).map_err(|_| invalid("arena extent exceeds u32"))?;
                Ok(($native(index), $wire(index)))
            })
            .collect::<Result<BTreeMap<_, _>, ArtifactError>>()?
    };
}
macro_rules! export_lookup {
    ($($method:ident: $field:ident ($native:ty) -> $wire:ty),* $(,)?) => { $(
        fn $method(&self, value: $native) -> Result<$wire, ArtifactError> { self.$field.get(&value).copied().ok_or_else(|| invalid(concat!("unregistered ", stringify!($field), " reference"))) }
    )* };
}
impl ExportIds {
    fn new_full(store: &TypeStore) -> Result<Self, ArtifactError> {
        let (scoped_count, lambda_count) = store.lambda_arena.artifact_counts();
        Ok(Self {
            types: export_map!(store.types.len(), TypeId, ArtifactTypeId),
            kinds: export_map!(store.kinds.len(), KindId, ArtifactKindId),
            parameters: export_map!(store.type_parameters.len(), TypeParameterId, ArtifactTypeParameterId),
            rows: export_map!(store.row_arena.len(), RecordRowId, ArtifactRecordRowId),
            families: export_map!(store.family_arena.len(), FamilyTypeId, ArtifactFamilyTypeId),
            variants: export_map!(store.variant_identities.len(), VariantTypeId, ArtifactVariantTypeId),
            domains: export_map!(store.static_domains.len(), StaticDomainId, ArtifactStaticDomainId),
            constructors: export_map!(store.static_constructors.len(), AlgebraicStaticConstructorId, ArtifactStaticConstructorId),
            scoped: export_map!(scoped_count, ScopedTypeId, ArtifactScopedTypeId),
            lambdas: export_map!(lambda_count, TypeLambdaId, ArtifactTypeLambdaId),
            type_order: (0..store.types.len()).map(|index| TypeId(index as u32)).collect(),
            kind_order: (0..store.kinds.len()).map(|index| KindId(index as u32)).collect(),
            parameter_order: (0..store.type_parameters.len()).map(|index| TypeParameterId(index as u32)).collect(),
            row_order: (0..store.row_arena.len()).map(|index| RecordRowId(index as u32)).collect(),
            family_order: (0..store.family_arena.len()).map(|index| FamilyTypeId(index as u32)).collect(),
            variant_order: (0..store.variant_identities.len()).map(|index| VariantTypeId(index as u32)).collect(),
            domain_order: (0..store.static_domains.len()).map(|index| StaticDomainId(index as u32)).collect(),
            constructor_order: (0..store.static_constructors.len())
                .map(|index| AlgebraicStaticConstructorId(index as u32))
                .collect(),
            scoped_order: (0..scoped_count).map(|index| ScopedTypeId(index as u32)).collect(),
            lambda_order: (0..lambda_count).map(|index| TypeLambdaId(index as u32)).collect(),
        })
    }

    fn empty() -> Self {
        Self {
            types: BTreeMap::new(),
            kinds: BTreeMap::new(),
            parameters: BTreeMap::new(),
            rows: BTreeMap::new(),
            families: BTreeMap::new(),
            variants: BTreeMap::new(),
            domains: BTreeMap::new(),
            constructors: BTreeMap::new(),
            scoped: BTreeMap::new(),
            lambdas: BTreeMap::new(),
            type_order: Vec::new(),
            kind_order: Vec::new(),
            parameter_order: Vec::new(),
            row_order: Vec::new(),
            family_order: Vec::new(),
            variant_order: Vec::new(),
            domain_order: Vec::new(),
            constructor_order: Vec::new(),
            scoped_order: Vec::new(),
            lambda_order: Vec::new(),
        }
    }

    fn from_roots(store: &TypeStore, roots: &BTreeMap<CanonicalTypeRootKey, TypeGraphRoot>, limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        let mut walker = ExportGraphWalker::new(store, limits);
        walker.visit_builtin_seed()?;
        let mut all_roots = roots.clone();
        for ((owner, index), parameter) in store.parameter_to_id.iter() {
            let key = CanonicalTypeRootKey {
                family_tag: CanonicalRootFamilyTag::ParameterCurrentSlot,
                owner: stable_parameter_owner(owner)?,
                product_subkey: *index,
            };
            let root = TypeGraphRoot::Parameter(*parameter);
            if let Some(previous) = all_roots.insert(key.clone(), root)
                && previous != root
            {
                return Err(TypeArtifactError::DuplicateRootKey(key).into());
            }
        }
        for root in all_roots.values().copied() {
            walker.visit_root(root)?;
        }
        walker.finish()
    }
    export_lookup!(ty: types(TypeId) -> ArtifactTypeId, kind: kinds(KindId) -> ArtifactKindId, parameter: parameters(TypeParameterId) -> ArtifactTypeParameterId, row: rows(RecordRowId) -> ArtifactRecordRowId, family: families(FamilyTypeId) -> ArtifactFamilyTypeId, variant: variants(VariantTypeId) -> ArtifactVariantTypeId, domain: domains(StaticDomainId) -> ArtifactStaticDomainId, constructor: constructors(AlgebraicStaticConstructorId) -> ArtifactStaticConstructorId, scoped: scoped(ScopedTypeId) -> ArtifactScopedTypeId, lambda: lambdas(TypeLambdaId) -> ArtifactTypeLambdaId);
    fn args(&self, values: &[StaticArgument]) -> Result<Vec<StaticArgumentV1>, ArtifactError> {
        values
            .iter()
            .map(|value| {
                Ok(match value {
                    StaticArgument::Term(ty) => StaticArgumentV1::Term { term: self.ty(*ty)? },
                    StaticArgument::RecordRow(row) => StaticArgumentV1::RecordRow { row: self.row(*row)? },
                    StaticArgument::DynamicType => StaticArgumentV1::DynamicType,
                })
            })
            .collect()
    }
    fn types(&self, values: &[TypeId]) -> Result<Vec<ArtifactTypeId>, ArtifactError> {
        values.iter().map(|ty| self.ty(*ty)).collect()
    }
    fn scopes(&self, values: &[ScopedTypeId]) -> Result<Vec<ArtifactScopedTypeId>, ArtifactError> {
        values.iter().map(|ty| self.scoped(*ty)).collect()
    }
    fn tuple(&self, elements: &[TupleTypeElement], positionals: AnonymousTupleRemainder, labeled: AnonymousTupleRemainder) -> Result<TupleV1, ArtifactError> {
        Ok(TupleV1 {
            elements: elements
                .iter()
                .map(|element| {
                    Ok(TupleElementV1 {
                        label: element.label.as_ref().map(ToString::to_string),
                        ty: self.ty(element.ty)?,
                    })
                })
                .collect::<Result<_, ArtifactError>>()?,
            positionals: self.remainder(positionals)?,
            labeled: self.remainder(labeled)?,
        })
    }
    fn remainder(&self, value: AnonymousTupleRemainder) -> Result<TupleRemainderV1, ArtifactError> {
        Ok(match value {
            AnonymousTupleRemainder::None => TupleRemainderV1::None,
            AnonymousTupleRemainder::Unconstrained => TupleRemainderV1::Unconstrained,
            AnonymousTupleRemainder::Typed(ty) => TupleRemainderV1::Typed { ty: self.ty(ty)? },
        })
    }
    fn field(&self, value: &RecordRowField) -> Result<FieldV1, ArtifactError> {
        Ok(FieldV1 {
            key: key_out(&value.key),
            ty: self.ty(value.ty)?,
            presence: presence_out(value.presence),
        })
    }
    fn scoped_field(&self, value: &ScopedRecordField) -> Result<ScopedFieldV1, ArtifactError> {
        Ok(ScopedFieldV1 {
            key: key_out(&value.key),
            ty: self.scoped(value.ty)?,
            presence: presence_out(value.presence),
        })
    }
    fn scoped_elements(&self, values: &[ScopedTupleElement]) -> Result<Vec<ScopedElementV1>, ArtifactError> {
        values
            .iter()
            .map(|value| {
                Ok(ScopedElementV1 {
                    label: value.label.as_ref().map(ToString::to_string),
                    ty: self.scoped(value.ty)?,
                })
            })
            .collect()
    }
    fn scoped_remainder(&self, value: ScopedPackRemainder) -> Result<ScopedRemainderV1, ArtifactError> {
        Ok(match value {
            ScopedPackRemainder::None => ScopedRemainderV1::None,
            ScopedPackRemainder::Unconstrained => ScopedRemainderV1::Unconstrained,
            ScopedPackRemainder::Typed(ty) => ScopedRemainderV1::Typed { ty: self.scoped(ty)? },
        })
    }
    fn type_form(&self, value: &TypeData) -> Result<TypeV1, ArtifactError> {
        Ok(match value {
            TypeData::Never => TypeV1::Never,
            TypeData::Unit => TypeV1::Unit,
            TypeData::ClassObject { declaration } => TypeV1::ClassObject {
                declaration: declaration_out(declaration)?,
            },
            TypeData::Nominal { declaration } => TypeV1::Nominal {
                declaration: declaration_out(declaration)?,
            },
            TypeData::StaticDomainForm { domain } => TypeV1::StaticDomainForm { domain: self.domain(*domain)? },
            TypeData::Applied { origin, arguments } => TypeV1::Applied {
                origin: self.ty(*origin)?,
                arguments: self.args(arguments)?,
            },
            TypeData::ExactCase { variant, enum_type } => TypeV1::ExactCase {
                variant: self.variant(*variant)?,
                enum_type: self.ty(*enum_type)?,
            },
            TypeData::Union(members) => TypeV1::Union { members: self.types(members)? },
            TypeData::Tuple(value) => TypeV1::Tuple {
                tuple: self.tuple(&value.elements, value.positional_remainder, value.labeled_remainder)?,
            },
            TypeData::Record(row) => TypeV1::Record { row: self.row(*row)? },
            TypeData::TypePack(members) => TypeV1::TypePack { members: self.types(members)? },
            TypeData::LabeledTypePack(members) => TypeV1::LabeledTypePack {
                members: members
                    .iter()
                    .map(|value| {
                        Ok(LabeledTypeV1 {
                            label: value.label.to_string(),
                            ty: self.ty(value.element)?,
                        })
                    })
                    .collect::<Result<_, ArtifactError>>()?,
            },
            TypeData::CompleteTypePack { positionals, labeled } => TypeV1::CompleteTypePack {
                positionals: self.ty(*positionals)?,
                labeled: self.ty(*labeled)?,
            },
            TypeData::TypePackConcat(members) => TypeV1::TypePackConcat { members: self.types(members)? },
            TypeData::LabeledTypePackConcat(members) => TypeV1::LabeledTypePackConcat { members: self.types(members)? },
            TypeData::TupleFromPacks { positionals, labeled } => TypeV1::TupleFromPacks {
                positionals: self.ty(*positionals)?,
                labeled: self.ty(*labeled)?,
            },
            TypeData::Pack(value) => TypeV1::Pack {
                pack: self.tuple(&value.elements, value.positional_remainder, value.labeled_remainder)?,
            },
            TypeData::PackFromPacks { positionals, labeled } => TypeV1::PackFromPacks {
                positionals: self.ty(*positionals)?,
                labeled: self.ty(*labeled)?,
            },
            TypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => TypeV1::CallableFromPacks {
                positionals: self.ty(*positionals)?,
                labeled: self.ty(*labeled)?,
                result: self.ty(*return_type)?,
            },
            TypeData::Callable(value) => TypeV1::Callable {
                callable: CallableV1 {
                    parameters: value
                        .parameters
                        .iter()
                        .map(|parameter| {
                            Ok(CallableParameterV1 {
                                label: parameter.label.as_ref().map(ToString::to_string),
                                ty: self.ty(parameter.ty)?,
                                rest: rest_out(parameter.rest),
                            })
                        })
                        .collect::<Result<_, ArtifactError>>()?,
                    result: self.ty(value.return_type)?,
                },
            },
            TypeData::Family(family) => TypeV1::Family { family: self.family(*family)? },
            TypeData::Parameter(parameter) => TypeV1::Parameter {
                parameter: self.parameter(*parameter)?,
            },
            TypeData::Lambda(lambda) => TypeV1::Lambda { lambda: self.lambda(*lambda)? },
            TypeData::SelfType(value) => TypeV1::SelfType {
                self_type: SelfTypeV1 {
                    owner: declaration_out(&value.owner)?,
                    side: to_stable_dispatch_side(value.side),
                    role: match value.role {
                        SelfRole::InstanceType => SelfRoleV1::InstanceType,
                        SelfRole::ReceiverValue => SelfRoleV1::ReceiverValue,
                    },
                },
            },
            TypeData::AssociatedProjection(value) => TypeV1::AssociatedProjection {
                projection: AssociatedProjectionV1 {
                    subject: self.ty(value.subject)?,
                    trait_owner: declaration_out(&value.trait_ref.declaration)?,
                    trait_arguments: self.types(&value.trait_ref.arguments)?,
                    requirement_owner: declaration_out(&value.requirement.owner)?,
                    requirement_index: value.requirement.index,
                },
            },
            TypeData::TypeMemberProjection(value) => TypeV1::TypeMemberProjection {
                projection: TypeProjectionV1 {
                    subject: match value.subject {
                        ProjectionSubject::Type(ty) => ProjectionSubjectV1::Type { ty: self.ty(ty)? },
                        ProjectionSubject::RecordRow(row) => ProjectionSubjectV1::RecordRow { row: self.row(row)? },
                    },
                    selector: selector_out(&value.selector),
                    arguments: value
                        .arguments
                        .iter()
                        .map(|argument| {
                            Ok(match argument {
                                TypeMemberArgument::Type(ty) => TypeMemberArgumentV1::Type { ty: self.ty(*ty)? },
                                TypeMemberArgument::Const(ConstTerm::Int(value)) => TypeMemberArgumentV1::ConstInt { decimal: value.to_string() },
                                TypeMemberArgument::RecordRow(row) => TypeMemberArgumentV1::RecordRow { row: self.row(*row)? },
                            })
                        })
                        .collect::<Result<_, ArtifactError>>()?,
                    source: value
                        .source_declaration
                        .as_ref()
                        .map(|source| {
                            Ok::<_, ArtifactError>(StableTypeMemberRef {
                                owner: declaration_out(&source.owner)?,
                                selector: source.selector.name().into(),
                            })
                        })
                        .transpose()?,
                },
            },
            TypeData::ConstInt(value) => TypeV1::ConstInt { decimal: value.to_string() },
            TypeData::StaticData(value) => TypeV1::StaticData {
                domain: self.domain(value.domain)?,
                constructor: self.constructor(value.constructor)?,
                arguments: self.args(&value.arguments)?,
            },
        })
    }
    fn scoped_form(&self, value: &ScopedTypeData) -> Result<ScopedTypeV1, ArtifactError> {
        Ok(match value {
            ScopedTypeData::Bound { depth, index } => ScopedTypeV1::Bound { depth: *depth, index: *index },
            ScopedTypeData::Free(ty) => ScopedTypeV1::Free { ty: self.ty(*ty)? },
            ScopedTypeData::Applied { origin, arguments } => ScopedTypeV1::Applied {
                origin: self.scoped(*origin)?,
                arguments: self.scopes(arguments)?,
            },
            ScopedTypeData::Union(members) => ScopedTypeV1::Union {
                members: self.scopes(members)?,
            },
            ScopedTypeData::Tuple(elements) => ScopedTypeV1::Tuple {
                elements: self.scoped_elements(elements)?,
            },
            ScopedTypeData::Record(fields) => ScopedTypeV1::Record {
                fields: fields.iter().map(|field| self.scoped_field(field)).collect::<Result<_, _>>()?,
            },
            ScopedTypeData::OpenRecord(value) => ScopedTypeV1::OpenRecord {
                row: ScopedRowV1 {
                    fields: value.fields.iter().map(|field| self.scoped_field(field)).collect::<Result<_, _>>()?,
                    tail: match value.tail {
                        ScopedRecordTail::Bound { depth, index } => ScopedRowTailV1::Bound { depth, index },
                        ScopedRecordTail::FreeParameter(parameter) => ScopedRowTailV1::Parameter {
                            parameter: self.parameter(parameter)?,
                        },
                        ScopedRecordTail::Anonymous(value) => self.scoped_anonymous(value)?,
                    },
                },
            },
            ScopedTypeData::RecordRow(value) => ScopedTypeV1::RecordRow {
                row: ScopedRowV1 {
                    fields: value.fields.iter().map(|field| self.scoped_field(field)).collect::<Result<_, _>>()?,
                    tail: match value.tail {
                        ScopedRecordRowTail::Closed => ScopedRowTailV1::Closed,
                        ScopedRecordRowTail::Bound { depth, index } => ScopedRowTailV1::Bound { depth, index },
                        ScopedRecordRowTail::FreeParameter(parameter) => ScopedRowTailV1::Parameter {
                            parameter: self.parameter(parameter)?,
                        },
                        ScopedRecordRowTail::Anonymous(value) => self.scoped_anonymous(value)?,
                        ScopedRecordRowTail::Projection(ty) => ScopedRowTailV1::Projection { ty: self.ty(ty)? },
                    },
                },
            },
            ScopedTypeData::Callable(value) => ScopedTypeV1::Callable {
                callable: ScopedCallableV1 {
                    parameters: value
                        .parameters
                        .iter()
                        .map(|parameter| {
                            Ok(ScopedParameterV1 {
                                label: parameter.label.as_ref().map(ToString::to_string),
                                ty: self.scoped(parameter.ty)?,
                                rest: parameter.rest,
                            })
                        })
                        .collect::<Result<_, ArtifactError>>()?,
                    result: self.scoped(value.return_type)?,
                },
            },
            ScopedTypeData::Pack(value) => ScopedTypeV1::Pack {
                pack: ScopedPackV1 {
                    elements: self.scoped_elements(&value.elements)?,
                    positionals: self.scoped_remainder(value.positional_remainder)?,
                    labeled: self.scoped_remainder(value.labeled_remainder)?,
                },
            },
            ScopedTypeData::TypePack(members) => ScopedTypeV1::TypePack {
                members: self.scopes(members)?,
            },
            ScopedTypeData::LabeledTypePack(members) => ScopedTypeV1::LabeledTypePack {
                members: members
                    .iter()
                    .map(|value| {
                        Ok(ScopedLabeledTypeV1 {
                            label: value.label.to_string(),
                            ty: self.scoped(value.element)?,
                        })
                    })
                    .collect::<Result<_, ArtifactError>>()?,
            },
            ScopedTypeData::CompleteTypePack { positionals, labeled } => ScopedTypeV1::CompleteTypePack {
                positionals: self.scoped(*positionals)?,
                labeled: self.scoped(*labeled)?,
            },
            ScopedTypeData::TypePackConcat(members) => ScopedTypeV1::TypePackConcat {
                members: self.scopes(members)?,
            },
            ScopedTypeData::LabeledTypePackConcat(members) => ScopedTypeV1::LabeledTypePackConcat {
                members: self.scopes(members)?,
            },
            ScopedTypeData::TupleFromPacks { positionals, labeled } => ScopedTypeV1::TupleFromPacks {
                positionals: self.scoped(*positionals)?,
                labeled: self.scoped(*labeled)?,
            },
            ScopedTypeData::PackFromPacks { positionals, labeled } => ScopedTypeV1::PackFromPacks {
                positionals: self.scoped(*positionals)?,
                labeled: self.scoped(*labeled)?,
            },
            ScopedTypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => ScopedTypeV1::CallableFromPacks {
                positionals: self.scoped(*positionals)?,
                labeled: self.scoped(*labeled)?,
                result: self.scoped(*return_type)?,
            },
            ScopedTypeData::PackProjection { subject, lane } => ScopedTypeV1::PackProjection {
                subject: self.scoped(*subject)?,
                lane: match lane {
                    ScopedPackProjectionLane::Positionals => PackLaneV1::Positionals,
                    ScopedPackProjectionLane::Labeled => PackLaneV1::Labeled,
                    ScopedPackProjectionLane::Complete => PackLaneV1::Complete,
                },
            },
            ScopedTypeData::Lambda(lambda) => ScopedTypeV1::Lambda { lambda: self.lambda(*lambda)? },
        })
    }
    fn scoped_anonymous(&self, value: ScopedAnonymousRecordRemainder) -> Result<ScopedRowTailV1, ArtifactError> {
        Ok(match value {
            AnonymousRecordRemainder::Unconstrained { key_type } => ScopedRowTailV1::Unconstrained {
                key_type: self.scoped(key_type)?,
            },
            AnonymousRecordRemainder::Typed { key_type, value_type } => ScopedRowTailV1::Typed {
                key_type: self.scoped(key_type)?,
                value_type: self.scoped(value_type)?,
            },
        })
    }
}

fn stable_parameter_owner(owner: &TypeParameterOwner) -> Result<StableOwnerRef, ArtifactError> {
    Ok(match owner {
        TypeParameterOwner::Declaration(owner) => StableOwnerRef::Declaration(declaration_out(owner)?),
        TypeParameterOwner::Callable(owner) => {
            if !owner.module().project.is_universe() {
                return Err(invalid("non-Universe callable parameter owner"));
            }
            StableOwnerRef::Callable(to_stable_callable(owner))
        }
        TypeParameterOwner::Impl(owner) => {
            if !owner.module.project.is_universe() {
                return Err(invalid("non-Universe impl parameter owner"));
            }
            StableOwnerRef::Impl(to_stable_impl(owner))
        }
    })
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ExportNode {
    Type(TypeId),
    Kind(KindId),
    Parameter(TypeParameterId),
    Row(RecordRowId),
    Family(FamilyTypeId),
    Variant(VariantTypeId),
    Domain(StaticDomainId),
    Constructor(AlgebraicStaticConstructorId),
    Scoped(ScopedTypeId),
    Lambda(TypeLambdaId),
}

macro_rules! assign_export_id {
    ($self:expr, $map:ident, $order:ident, $value:expr, $wire:ident, $extent:expr, $budget:expr, $arena:literal) => {{
        let value = $value;
        if $self.ids.$map.contains_key(&value) {
            false
        } else {
            if value.0 as usize >= $extent {
                return Err(TypeArtifactError::InvalidNativeCoordinate { arena: $arena, raw: value.0 }.into());
            }
            if $self.ids.$order.len() >= $budget {
                return Err(invalid(concat!($arena, " export node budget exceeded")));
            }
            let raw = u32::try_from($self.ids.$order.len()).map_err(|_| invalid(concat!($arena, " export extent exceeds u32")))?;
            $self.ids.$map.insert(value, $wire(raw));
            $self.ids.$order.push(value);
            true
        }
    }};
}

struct ExportGraphWalker<'a> {
    store: &'a TypeStore,
    limits: &'a ValidationLimits,
    ids: ExportIds,
    active: BTreeSet<ExportNode>,
}

impl<'a> ExportGraphWalker<'a> {
    fn new(store: &'a TypeStore, limits: &'a ValidationLimits) -> Self {
        Self {
            store,
            limits,
            ids: ExportIds::empty(),
            active: BTreeSet::new(),
        }
    }

    fn enter(&mut self, node: ExportNode, inserted: bool) -> Result<bool, ArtifactError> {
        if !inserted {
            if self.active.contains(&node) {
                return Err(invalid("cyclic canonical type export graph"));
            }
            return Ok(false);
        }
        if self.active.len() >= self.limits.max_recursion_depth {
            return Err(invalid("canonical type export graph depth exceeds limit"));
        }
        self.active.insert(node);
        Ok(true)
    }

    fn leave(&mut self, node: ExportNode, result: Result<(), ArtifactError>) -> Result<(), ArtifactError> {
        self.active.remove(&node);
        result
    }

    fn visit_builtin_seed(&mut self) -> Result<(), ArtifactError> {
        // Every receiving TypeStore starts with this fixed semantic seed. Keeping
        // it reachable makes ordinary interning restore exactly those primitives
        // without depending on the source store's later allocation history.
        let seed = TypeStore::new();
        for index in 0..seed.kinds.len() {
            self.visit_kind(KindId(index as u32))?;
        }
        for index in 0..seed.static_domains.len() {
            self.visit_domain(StaticDomainId(index as u32))?;
        }
        for index in 0..seed.static_constructors.len() {
            self.visit_constructor(AlgebraicStaticConstructorId(index as u32))?;
        }
        for index in 0..seed.type_parameters.len() {
            self.visit_parameter(TypeParameterId(index as u32))?;
        }
        for index in 0..seed.types.len() {
            self.visit_type(TypeId(index as u32))?;
        }
        for index in 0..seed.row_arena.len() {
            self.visit_row(RecordRowId(index as u32))?;
        }
        for index in 0..seed.family_arena.len() {
            self.visit_family(FamilyTypeId(index as u32))?;
        }
        for index in 0..seed.variant_identities.len() {
            self.visit_variant(VariantTypeId(index as u32))?;
        }
        let (scoped, lambdas) = seed.lambda_arena.artifact_counts();
        for index in 0..scoped {
            self.visit_scoped(ScopedTypeId(index as u32))?;
        }
        for index in 0..lambdas {
            self.visit_lambda(TypeLambdaId(index as u32))?;
        }
        Ok(())
    }

    fn visit_root(&mut self, root: TypeGraphRoot) -> Result<(), ArtifactError> {
        match root {
            TypeGraphRoot::Type(value) => self.visit_type(value),
            TypeGraphRoot::Kind(value) => self.visit_kind(value),
            TypeGraphRoot::Parameter(value) => self.visit_parameter(value),
            TypeGraphRoot::Row(value) => self.visit_row(value),
            TypeGraphRoot::Family(value) => self.visit_family(value),
            TypeGraphRoot::Variant(value) => self.visit_variant(value),
            TypeGraphRoot::Domain(value) => self.visit_domain(value),
            TypeGraphRoot::Constructor(value) => self.visit_constructor(value),
            TypeGraphRoot::Scoped(value) => self.visit_scoped(value),
            TypeGraphRoot::Lambda(value) => self.visit_lambda(value),
        }
    }

    fn visit_static_argument(&mut self, value: StaticArgument) -> Result<(), ArtifactError> {
        match value {
            StaticArgument::Term(value) => self.visit_type(value),
            StaticArgument::RecordRow(value) => self.visit_row(value),
            StaticArgument::DynamicType => Ok(()),
        }
    }

    fn visit_tuple_parts(
        &mut self,
        elements: &[TupleTypeElement],
        positionals: AnonymousTupleRemainder,
        labeled: AnonymousTupleRemainder,
    ) -> Result<(), ArtifactError> {
        for element in elements {
            self.visit_type(element.ty)?;
        }
        for remainder in [positionals, labeled] {
            if let AnonymousTupleRemainder::Typed(value) = remainder {
                self.visit_type(value)?;
            }
        }
        Ok(())
    }

    fn visit_type(&mut self, value: TypeId) -> Result<(), ArtifactError> {
        let inserted = assign_export_id!(
            self,
            types,
            type_order,
            value,
            ArtifactTypeId,
            self.store.type_count(),
            self.limits.max_type_nodes,
            "type"
        );
        let node = ExportNode::Type(value);
        if !self.enter(node, inserted)? {
            return Ok(());
        }
        let result = (|| {
            self.visit_kind(self.store.kind_of(value))?;
            match self.store.get(value).clone() {
                TypeData::Never | TypeData::Unit | TypeData::ClassObject { .. } | TypeData::Nominal { .. } | TypeData::SelfType(_) | TypeData::ConstInt(_) => {}
                TypeData::StaticDomainForm { domain } => self.visit_domain(domain)?,
                TypeData::Applied { origin, arguments } => {
                    self.visit_type(origin)?;
                    for argument in arguments.iter().copied() {
                        self.visit_static_argument(argument)?;
                    }
                }
                TypeData::ExactCase { variant, enum_type } => {
                    self.visit_variant(variant)?;
                    self.visit_type(enum_type)?;
                }
                TypeData::Union(values) | TypeData::TypePack(values) | TypeData::TypePackConcat(values) | TypeData::LabeledTypePackConcat(values) => {
                    for value in values.iter().copied() {
                        self.visit_type(value)?;
                    }
                }
                TypeData::Tuple(value) => {
                    self.visit_tuple_parts(&value.elements, value.positional_remainder, value.labeled_remainder)?;
                }
                TypeData::Record(value) => self.visit_row(value)?,
                TypeData::LabeledTypePack(values) => {
                    for value in values.iter() {
                        self.visit_type(value.element)?;
                    }
                }
                TypeData::CompleteTypePack { positionals, labeled }
                | TypeData::TupleFromPacks { positionals, labeled }
                | TypeData::PackFromPacks { positionals, labeled } => {
                    self.visit_type(positionals)?;
                    self.visit_type(labeled)?;
                }
                TypeData::Pack(value) => {
                    self.visit_tuple_parts(&value.elements, value.positional_remainder, value.labeled_remainder)?;
                }
                TypeData::CallableFromPacks {
                    positionals,
                    labeled,
                    return_type,
                } => {
                    self.visit_type(positionals)?;
                    self.visit_type(labeled)?;
                    self.visit_type(return_type)?;
                }
                TypeData::Callable(value) => {
                    for parameter in value.parameters.iter() {
                        self.visit_type(parameter.ty)?;
                    }
                    self.visit_type(value.return_type)?;
                }
                TypeData::Family(value) => self.visit_family(value)?,
                TypeData::Parameter(value) => self.visit_parameter(value)?,
                TypeData::Lambda(value) => self.visit_lambda(value)?,
                TypeData::AssociatedProjection(value) => {
                    self.visit_type(value.subject)?;
                    for argument in value.trait_ref.arguments.iter().copied() {
                        self.visit_type(argument)?;
                    }
                }
                TypeData::TypeMemberProjection(value) => {
                    match value.subject {
                        ProjectionSubject::Type(value) => self.visit_type(value)?,
                        ProjectionSubject::RecordRow(value) => self.visit_row(value)?,
                    }
                    for argument in value.arguments.iter() {
                        match argument {
                            TypeMemberArgument::Type(value) => self.visit_type(*value)?,
                            TypeMemberArgument::RecordRow(value) => self.visit_row(*value)?,
                            TypeMemberArgument::Const(ConstTerm::Int(_)) => {}
                        }
                    }
                }
                TypeData::StaticData(value) => {
                    self.visit_domain(value.domain)?;
                    self.visit_constructor(value.constructor)?;
                    for argument in value.arguments.iter().copied() {
                        self.visit_static_argument(argument)?;
                    }
                }
            }
            Ok(())
        })();
        self.leave(node, result)
    }

    fn visit_kind(&mut self, value: KindId) -> Result<(), ArtifactError> {
        let inserted = assign_export_id!(
            self,
            kinds,
            kind_order,
            value,
            ArtifactKindId,
            self.store.kind_count(),
            self.limits.max_kind_nodes,
            "kind"
        );
        let node = ExportNode::Kind(value);
        if !self.enter(node, inserted)? {
            return Ok(());
        }
        let result = (|| {
            match self.store.get_kind(value).clone() {
                KindData::Type | KindData::RecordRow | KindData::TypePack | KindData::LabeledTypePack | KindData::CompleteTypePack => {}
                KindData::Arrow { parameters, result } => {
                    for parameter in parameters.iter().copied() {
                        self.visit_kind(parameter)?;
                    }
                    self.visit_kind(result)?;
                }
                KindData::StaticDomain(value) => self.visit_domain(value)?,
            }
            Ok(())
        })();
        self.leave(node, result)
    }

    fn visit_parameter(&mut self, value: TypeParameterId) -> Result<(), ArtifactError> {
        let inserted = assign_export_id!(
            self,
            parameters,
            parameter_order,
            value,
            ArtifactTypeParameterId,
            self.store.parameter_count(),
            self.limits.max_parameters,
            "parameter"
        );
        let node = ExportNode::Parameter(value);
        if !self.enter(node, inserted)? {
            return Ok(());
        }
        let kind = self.store.type_parameter(value).kind;
        let result = self.visit_kind(kind);
        self.leave(node, result)
    }

    fn visit_row(&mut self, value: RecordRowId) -> Result<(), ArtifactError> {
        let inserted = assign_export_id!(
            self,
            rows,
            row_order,
            value,
            ArtifactRecordRowId,
            self.store.row_count(),
            self.limits.max_semantic_entries_per_family,
            "record row"
        );
        let node = ExportNode::Row(value);
        if !self.enter(node, inserted)? {
            return Ok(());
        }
        let row = self.store.record_row(value).clone();
        let result = (|| {
            for field in row.fields.iter() {
                self.visit_type(field.ty)?;
            }
            match row.tail {
                RecordRowTail::Closed => {}
                RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type }) => self.visit_type(key_type)?,
                RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed { key_type, value_type }) => {
                    self.visit_type(key_type)?;
                    self.visit_type(value_type)?;
                }
                RecordRowTail::Parameter(value) => self.visit_parameter(value)?,
                RecordRowTail::Projection(value) => self.visit_type(value)?,
            }
            Ok(())
        })();
        self.leave(node, result)
    }

    fn visit_family(&mut self, value: FamilyTypeId) -> Result<(), ArtifactError> {
        let inserted = assign_export_id!(
            self,
            families,
            family_order,
            value,
            ArtifactFamilyTypeId,
            self.store.family_count(),
            self.limits.max_semantic_entries_per_family,
            "family"
        );
        let node = ExportNode::Family(value);
        if !self.enter(node, inserted)? {
            return Ok(());
        }
        let members = self.store.get_family(value).members.clone();
        let result = (|| {
            for member in members.iter() {
                self.visit_type(member.ty)?;
            }
            Ok(())
        })();
        self.leave(node, result)
    }

    fn visit_variant(&mut self, value: VariantTypeId) -> Result<(), ArtifactError> {
        let inserted = assign_export_id!(
            self,
            variants,
            variant_order,
            value,
            ArtifactVariantTypeId,
            self.store.variant_count(),
            self.limits.max_semantic_entries_per_family,
            "variant"
        );
        let node = ExportNode::Variant(value);
        if !self.enter(node, inserted)? {
            return Ok(());
        }
        self.leave(node, Ok(()))
    }

    fn visit_domain(&mut self, value: StaticDomainId) -> Result<(), ArtifactError> {
        let inserted = assign_export_id!(
            self,
            domains,
            domain_order,
            value,
            ArtifactStaticDomainId,
            self.store.static_domain_count(),
            self.limits.max_semantic_entries_per_family,
            "static domain"
        );
        let node = ExportNode::Domain(value);
        if !self.enter(node, inserted)? {
            return Ok(());
        }
        let domain = self.store.static_domain(value).cloned().ok_or_else(|| invalid("missing static domain"))?;
        let result = (|| {
            if let StaticDomainData::Associated { owner_arguments, .. } = domain {
                for argument in owner_arguments.iter().copied() {
                    self.visit_static_argument(argument)?;
                }
            }
            Ok(())
        })();
        self.leave(node, result)
    }

    fn visit_constructor(&mut self, value: AlgebraicStaticConstructorId) -> Result<(), ArtifactError> {
        let inserted = assign_export_id!(
            self,
            constructors,
            constructor_order,
            value,
            ArtifactStaticConstructorId,
            self.store.static_constructor_count(),
            self.limits.max_semantic_entries_per_family,
            "static constructor"
        );
        let node = ExportNode::Constructor(value);
        if !self.enter(node, inserted)? {
            return Ok(());
        }
        let constructor = self
            .store
            .static_constructor(value)
            .cloned()
            .ok_or_else(|| invalid("missing static constructor"))?;
        let result = (|| {
            self.visit_domain(constructor.domain)?;
            for kind in constructor.parameter_kinds.iter().copied() {
                self.visit_kind(kind)?;
            }
            Ok(())
        })();
        self.leave(node, result)
    }

    fn visit_scoped_remainder(&mut self, value: ScopedPackRemainder) -> Result<(), ArtifactError> {
        if let ScopedPackRemainder::Typed(value) = value {
            self.visit_scoped(value)?;
        }
        Ok(())
    }

    fn visit_scoped_fields(&mut self, fields: &[ScopedRecordField]) -> Result<(), ArtifactError> {
        for field in fields {
            self.visit_scoped(field.ty)?;
        }
        Ok(())
    }

    fn visit_scoped_row_tail(&mut self, tail: ScopedRecordRowTail) -> Result<(), ArtifactError> {
        match tail {
            ScopedRecordRowTail::Closed | ScopedRecordRowTail::Bound { .. } => {}
            ScopedRecordRowTail::FreeParameter(value) => self.visit_parameter(value)?,
            ScopedRecordRowTail::Projection(value) => self.visit_type(value)?,
            ScopedRecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type }) => self.visit_scoped(key_type)?,
            ScopedRecordRowTail::Anonymous(AnonymousRecordRemainder::Typed { key_type, value_type }) => {
                self.visit_scoped(key_type)?;
                self.visit_scoped(value_type)?;
            }
        }
        Ok(())
    }

    fn visit_scoped(&mut self, value: ScopedTypeId) -> Result<(), ArtifactError> {
        let (scoped_count, _) = self.store.lambda_arena.artifact_counts();
        let inserted = assign_export_id!(
            self,
            scoped,
            scoped_order,
            value,
            ArtifactScopedTypeId,
            scoped_count,
            self.limits.max_type_nodes,
            "scoped type"
        );
        let node = ExportNode::Scoped(value);
        if !self.enter(node, inserted)? {
            return Ok(());
        }
        let scoped = self.store.lambda_arena.get_scoped(value).clone();
        let result = (|| {
            match scoped {
                ScopedTypeData::Bound { .. } => {}
                ScopedTypeData::Free(value) => self.visit_type(value)?,
                ScopedTypeData::Applied { origin, arguments } => {
                    self.visit_scoped(origin)?;
                    for argument in arguments.iter().copied() {
                        self.visit_scoped(argument)?;
                    }
                }
                ScopedTypeData::Union(values)
                | ScopedTypeData::TypePack(values)
                | ScopedTypeData::TypePackConcat(values)
                | ScopedTypeData::LabeledTypePackConcat(values) => {
                    for value in values.iter().copied() {
                        self.visit_scoped(value)?;
                    }
                }
                ScopedTypeData::Tuple(values) => {
                    for value in values.iter() {
                        self.visit_scoped(value.ty)?;
                    }
                }
                ScopedTypeData::Record(values) => {
                    for value in values.iter() {
                        self.visit_scoped(value.ty)?;
                    }
                }
                ScopedTypeData::OpenRecord(value) => {
                    self.visit_scoped_fields(&value.fields)?;
                    match value.tail {
                        ScopedRecordTail::Bound { .. } => {}
                        ScopedRecordTail::FreeParameter(value) => self.visit_parameter(value)?,
                        ScopedRecordTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type }) => self.visit_scoped(key_type)?,
                        ScopedRecordTail::Anonymous(AnonymousRecordRemainder::Typed { key_type, value_type }) => {
                            self.visit_scoped(key_type)?;
                            self.visit_scoped(value_type)?;
                        }
                    }
                }
                ScopedTypeData::RecordRow(value) => {
                    self.visit_scoped_fields(&value.fields)?;
                    self.visit_scoped_row_tail(value.tail)?;
                }
                ScopedTypeData::Callable(value) => {
                    for parameter in value.parameters.iter() {
                        self.visit_scoped(parameter.ty)?;
                    }
                    self.visit_scoped(value.return_type)?;
                }
                ScopedTypeData::Pack(value) => {
                    for element in value.elements.iter() {
                        self.visit_scoped(element.ty)?;
                    }
                    self.visit_scoped_remainder(value.positional_remainder)?;
                    self.visit_scoped_remainder(value.labeled_remainder)?;
                }
                ScopedTypeData::LabeledTypePack(values) => {
                    for value in values.iter() {
                        self.visit_scoped(value.element)?;
                    }
                }
                ScopedTypeData::CompleteTypePack { positionals, labeled }
                | ScopedTypeData::TupleFromPacks { positionals, labeled }
                | ScopedTypeData::PackFromPacks { positionals, labeled } => {
                    self.visit_scoped(positionals)?;
                    self.visit_scoped(labeled)?;
                }
                ScopedTypeData::CallableFromPacks {
                    positionals,
                    labeled,
                    return_type,
                } => {
                    self.visit_scoped(positionals)?;
                    self.visit_scoped(labeled)?;
                    self.visit_scoped(return_type)?;
                }
                ScopedTypeData::PackProjection { subject, .. } => self.visit_scoped(subject)?,
                ScopedTypeData::Lambda(value) => self.visit_lambda(value)?,
            }
            Ok(())
        })();
        self.leave(node, result)
    }

    fn visit_lambda(&mut self, value: TypeLambdaId) -> Result<(), ArtifactError> {
        let (_, lambda_count) = self.store.lambda_arena.artifact_counts();
        let inserted = assign_export_id!(
            self,
            lambdas,
            lambda_order,
            value,
            ArtifactTypeLambdaId,
            lambda_count,
            self.limits.max_type_nodes,
            "type lambda"
        );
        let node = ExportNode::Lambda(value);
        if !self.enter(node, inserted)? {
            return Ok(());
        }
        let lambda = self.store.lambda_arena.get_lambda(value).clone();
        let result = (|| {
            for kind in lambda.parameter_kinds.iter().copied() {
                self.visit_kind(kind)?;
            }
            self.visit_scoped(lambda.body)?;
            self.visit_kind(lambda.result_kind)?;
            Ok(())
        })();
        self.leave(node, result)
    }

    fn finish(self) -> Result<ExportIds, ArtifactError> {
        if !self.active.is_empty() {
            return Err(invalid("type export graph closed with active traversal nodes"));
        }
        Ok(self.ids)
    }
}

fn declaration_out(value: &DeclarationId) -> Result<StableDeclarationRef, ArtifactError> {
    if !value.module.project.is_universe() {
        return Err(invalid("canonical type graph contains non-Universe declaration"));
    }
    Ok(to_stable_declaration(value))
}
fn owner_out(value: &TypeParameterOwner) -> Result<ParameterOwnerV1, ArtifactError> {
    Ok(match value {
        TypeParameterOwner::Declaration(owner) => ParameterOwnerV1::Declaration {
            owner: declaration_out(owner)?,
        },
        TypeParameterOwner::Callable(owner) => {
            if !owner.module().project.is_universe() {
                return Err(invalid("non-Universe callable parameter owner"));
            }
            ParameterOwnerV1::Callable {
                owner: to_stable_callable(owner),
            }
        }
        TypeParameterOwner::Impl(owner) => {
            if !owner.module.project.is_universe() {
                return Err(invalid("non-Universe impl parameter owner"));
            }
            ParameterOwnerV1::Impl { owner: to_stable_impl(owner) }
        }
    })
}
fn key_out(value: &FieldKey) -> FieldKeyV1 {
    match value {
        FieldKey::Symbol(value) => FieldKeyV1::Symbol { value: value.to_string() },
        FieldKey::String(value) => FieldKeyV1::String { value: value.to_string() },
        FieldKey::Int(value) => FieldKeyV1::Int { decimal: value.to_string() },
        FieldKey::Bool(value) => FieldKeyV1::Bool { value: *value },
    }
}
fn presence_out(value: StructuralMemberPresence) -> PresenceV1 {
    match value {
        StructuralMemberPresence::Required => PresenceV1::Required,
        StructuralMemberPresence::Optional => PresenceV1::Optional,
    }
}
fn variance_out(value: Variance) -> VarianceV1 {
    match value {
        Variance::Covariant => VarianceV1::Covariant,
        Variance::Contravariant => VarianceV1::Contravariant,
        Variance::Invariant => VarianceV1::Invariant,
    }
}
fn mode_out(value: GenericParameterMode) -> ParameterModeV1 {
    match value {
        GenericParameterMode::Single => ParameterModeV1::Single,
        GenericParameterMode::PositionalPack => ParameterModeV1::PositionalPack,
        GenericParameterMode::LabeledPack => ParameterModeV1::LabeledPack,
        GenericParameterMode::CompletePack => ParameterModeV1::CompletePack,
    }
}
fn rest_out(value: RestMode) -> RestModeV1 {
    match value {
        RestMode::None => RestModeV1::None,
        RestMode::Positional => RestModeV1::Positional,
        RestMode::Labeled => RestModeV1::Labeled,
        RestMode::Complete => RestModeV1::Complete,
    }
}
fn selector_out(value: &TypeMemberSelector) -> TypeMemberSelectorV1 {
    match value {
        TypeMemberSelector::Named(name) => TypeMemberSelectorV1::Named { name: name.to_string() },
        TypeMemberSelector::Subscript => TypeMemberSelectorV1::Subscript,
    }
}
fn operation_out(value: &FamilyOperationShape) -> OperationV1 {
    OperationV1 {
        kind: match value.kind {
            SelectorKind::Getter => SelectorKindV1::Getter,
            SelectorKind::Setter => SelectorKindV1::Setter,
            SelectorKind::Method => SelectorKindV1::Method,
            SelectorKind::SubscriptGet => SelectorKindV1::SubscriptGet,
            SelectorKind::SubscriptSet => SelectorKindV1::SubscriptSet,
        },
        slots: value
            .slots
            .iter()
            .map(|slot| match slot {
                SelectorSlot::Positional => SelectorSlotV1::Positional,
                SelectorSlot::Label(name) => SelectorSlotV1::Label { name: name.clone() },
            })
            .collect(),
    }
}

impl TypeStore {
    /// Projects finalized canonical arenas without storing process arena identities.
    pub fn project_artifact(&self, limits: &ValidationLimits) -> Result<TypeGraphV1, ArtifactError> {
        let ids = ExportIds::new_full(self)?;
        self.project_artifact_with_ids(&ids, limits)
    }

    fn project_artifact_with_ids(&self, ids: &ExportIds, limits: &ValidationLimits) -> Result<TypeGraphV1, ArtifactError> {
        if ids.type_order.len() > limits.max_type_nodes || ids.kind_order.len() > limits.max_kind_nodes || ids.parameter_order.len() > limits.max_parameters {
            return Err(invalid("canonical type graph extent exceeds budget"));
        }
        let types = ids
            .type_order
            .iter()
            .map(|id| {
                Ok(TypeEntryV1 {
                    kind: ids.kind(self.type_kinds[id.index()])?,
                    form: ids.type_form(&self.types[id.index()])?,
                })
            })
            .collect::<Result<_, ArtifactError>>()?;
        let kinds = ids
            .kind_order
            .iter()
            .map(|id| {
                let value = &self.kinds[id.index()];
                Ok(match value {
                    KindData::Type => KindV1::Type,
                    KindData::RecordRow => KindV1::RecordRow,
                    KindData::TypePack => KindV1::TypePack,
                    KindData::LabeledTypePack => KindV1::LabeledTypePack,
                    KindData::CompleteTypePack => KindV1::CompleteTypePack,
                    KindData::Arrow { parameters, result } => KindV1::Arrow {
                        parameters: parameters.iter().map(|kind| ids.kind(*kind)).collect::<Result<_, _>>()?,
                        result: ids.kind(*result)?,
                    },
                    KindData::StaticDomain(domain) => KindV1::StaticDomain { domain: ids.domain(*domain)? },
                })
            })
            .collect::<Result<_, ArtifactError>>()?;
        let parameters = ids
            .parameter_order
            .iter()
            .map(|id| {
                let value = &self.type_parameters[id.index()];
                Ok(ParameterV1 {
                    owner: owner_out(&value.owner)?,
                    index: value.index,
                    name: value.name.to_string(),
                    kind: ids.kind(value.kind)?,
                    variance: variance_out(value.variance),
                    mode: mode_out(value.mode),
                })
            })
            .collect::<Result<_, ArtifactError>>()?;
        let mut current_slots = self.parameter_to_id.iter().collect::<Vec<_>>();
        current_slots.sort_by(|a, b| a.0.cmp(b.0));
        let current_parameters = current_slots
            .iter()
            .map(|(_, parameter)| ids.parameter(**parameter))
            .collect::<Result<_, _>>()?;
        let rows = ids
            .row_order
            .iter()
            .map(|id| {
                let value = &self.row_arena[id.index()];
                Ok(RowV1 {
                    fields: value.fields.iter().map(|field| ids.field(field)).collect::<Result<_, _>>()?,
                    tail: match value.tail {
                        RecordRowTail::Closed => RowTailV1::Closed,
                        RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type }) => {
                            RowTailV1::Unconstrained { key_type: ids.ty(key_type)? }
                        }
                        RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed { key_type, value_type }) => RowTailV1::Typed {
                            key_type: ids.ty(key_type)?,
                            value_type: ids.ty(value_type)?,
                        },
                        RecordRowTail::Parameter(parameter) => RowTailV1::Parameter {
                            parameter: ids.parameter(parameter)?,
                        },
                        RecordRowTail::Projection(ty) => RowTailV1::Projection { ty: ids.ty(ty)? },
                    },
                })
            })
            .collect::<Result<_, ArtifactError>>()?;
        let families = ids
            .family_order
            .iter()
            .map(|id| {
                let value = &self.family_arena[id.index()];
                Ok(FamilyV1 {
                    members: value
                        .members
                        .iter()
                        .map(|member| {
                            Ok(FamilyMemberV1 {
                                operation: operation_out(&member.operation),
                                kind: match member.member_kind {
                                    FamilyMemberTypeKind::Value => FamilyMemberKindV1::Value,
                                    FamilyMemberTypeKind::Callable => FamilyMemberKindV1::Callable,
                                },
                                ty: ids.ty(member.ty)?,
                            })
                        })
                        .collect::<Result<_, ArtifactError>>()?,
                })
            })
            .collect::<Result<_, ArtifactError>>()?;
        let variants = ids
            .variant_order
            .iter()
            .map(|id| {
                let variant = &self.variant_identities[id.index()];
                Ok(StableVariantRef {
                    owner: declaration_out(&variant.owner)?,
                    selector: variant.selector.encode().into(),
                })
            })
            .collect::<Result<_, ArtifactError>>()?;
        let static_domains = ids
            .domain_order
            .iter()
            .map(|id| {
                let value = &self.static_domains[id.index()];
                Ok(match value {
                    StaticDomainData::ConstInt => StaticDomainV1::ConstInt,
                    StaticDomainData::Nominal { declaration } => StaticDomainV1::Nominal {
                        declaration: declaration_out(declaration)?,
                    },
                    StaticDomainData::Associated { declaration, owner_arguments } => StaticDomainV1::Associated {
                        declaration: StableAssociatedStaticDomainDeclarationRef {
                            owner: declaration_out(&declaration.owner)?,
                            name: declaration.name.clone(),
                        },
                        arguments: ids.args(owner_arguments)?,
                    },
                })
            })
            .collect::<Result<_, ArtifactError>>()?;
        let static_constructors = ids
            .constructor_order
            .iter()
            .map(|id| {
                let value = &self.static_constructors[id.index()];
                Ok(StaticConstructorV1 {
                    domain: ids.domain(value.domain)?,
                    declaration: declaration_out(&value.declaration)?,
                    case_name: value.case_name.to_string(),
                    parameter_kinds: value.parameter_kinds.iter().map(|kind| ids.kind(*kind)).collect::<Result<_, _>>()?,
                })
            })
            .collect::<Result<_, ArtifactError>>()?;
        let scoped = ids
            .scoped_order
            .iter()
            .map(|id| ids.scoped_form(self.lambda_arena.get_scoped(*id)))
            .collect::<Result<_, _>>()?;
        let lambdas = ids
            .lambda_order
            .iter()
            .map(|id| {
                let value = self.lambda_arena.get_lambda(*id);
                Ok(LambdaV1 {
                    parameter_kinds: value.parameter_kinds.iter().map(|kind| ids.kind(*kind)).collect::<Result<_, _>>()?,
                    parameter_modes: value.parameter_modes.iter().map(|mode| mode_out(*mode)).collect(),
                    body: ids.scoped(value.body)?,
                    result_kind: ids.kind(value.result_kind)?,
                })
            })
            .collect::<Result<_, ArtifactError>>()?;
        let mut overrides = self.parameter_variances.iter().collect::<Vec<_>>();
        overrides.sort_by(|a, b| a.0.cmp(b.0));
        let variance_overrides = overrides
            .iter()
            .map(|((owner, index), variance)| {
                Ok(VarianceOverrideV1 {
                    owner: declaration_out(owner)?,
                    index: *index,
                    variance: variance_out(**variance),
                })
            })
            .collect::<Result<_, ArtifactError>>()?;
        Ok(TypeGraphV1 {
            types,
            kinds,
            parameters,
            current_parameters,
            rows,
            families,
            variants,
            static_domains,
            static_constructors,
            scoped,
            lambdas,
            variance_overrides,
        })
    }
}

/// Complete relocation into one fresh receiving store. These are allocation
/// products, not persisted identity, and are checked on every foreign read.
#[derive(Debug)]
pub struct TypeRelocation {
    pub store_id: TypeStoreId,
    pub types: Vec<TypeId>,
    pub kinds: Vec<KindId>,
    pub parameters: Vec<TypeParameterId>,
    pub rows: Vec<RecordRowId>,
    pub families: Vec<FamilyTypeId>,
    pub variants: Vec<VariantTypeId>,
    pub domains: Vec<StaticDomainId>,
    pub constructors: Vec<AlgebraicStaticConstructorId>,
    pub scoped: Vec<ScopedTypeId>,
    pub lambdas: Vec<TypeLambdaId>,
}
macro_rules! relocate_lookup {
    ($($name:ident: $field:ident ($wire:ty) -> $native:ty),* $(,)?) => { $(
        pub fn $name(&self, id: $wire) -> Result<$native, ArtifactError> { self.$field.get(id.0 as usize).copied().ok_or_else(|| invalid(concat!("out-of-range relocated ", stringify!($field)))) }
    )* };
}
impl TypeRelocation {
    pub fn verify_store(&self, store: &TypeStore) -> Result<(), TypeArtifactError> {
        let expected = TypeStoreLineageId(self.store_id.raw());
        let actual = TypeStoreLineageId(store.id().raw());
        if expected != actual {
            return Err(TypeArtifactError::ForeignStoreLineage { expected, actual });
        }
        Ok(())
    }

    relocate_lookup!(ty: types(ArtifactTypeId) -> TypeId, kind: kinds(ArtifactKindId) -> KindId, parameter: parameters(ArtifactTypeParameterId) -> TypeParameterId, row: rows(ArtifactRecordRowId) -> RecordRowId, family: families(ArtifactFamilyTypeId) -> FamilyTypeId, variant: variants(ArtifactVariantTypeId) -> VariantTypeId, domain: domains(ArtifactStaticDomainId) -> StaticDomainId, constructor: constructors(ArtifactStaticConstructorId) -> AlgebraicStaticConstructorId, scoped: scoped(ArtifactScopedTypeId) -> ScopedTypeId, lambda: lambdas(ArtifactTypeLambdaId) -> TypeLambdaId);
}

#[cfg(test)]
mod artifact_relocation_validation_tests {
    use super::{TypeExportContext, TypeStore};
    use phalcom_compiler_artifact::ValidationLimits;

    #[test]
    fn type_graph_import_rejects_duplicate_relocated_type_identity() {
        let limits = ValidationLimits::default();
        let source = TypeStore::new();
        let export = TypeExportContext::new_full(&source).expect("export context");
        let mut graph = export.project_graph(&source, &limits).expect("type graph");
        graph.types.push(graph.types[0].clone());

        let error = TypeStore::import_type_graph(&graph, &limits).expect_err("duplicate canonical type nodes must be rejected");
        assert!(error.to_string().contains("duplicate canonical type node"), "unexpected error: {error}");
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Node {
    Type(u32),
    Kind(u32),
    Parameter(u32),
    Row(u32),
    Family(u32),
    Variant(u32),
    Domain(u32),
    Constructor(u32),
    Scoped(u32),
    Lambda(u32),
}
struct Import<'a, 'b, 'c, 'd> {
    owner_catalog: Option<&'c crate::artifact::StableOwnerCatalog<'c>>,
    receiver: Option<&'d mut TypeStore>,
    graph: &'a TypeGraphV1,
    limits: &'a ValidationLimits,
    allocations: &'b mut AllocationBudget,
    active: Vec<Node>,
    types: Vec<Option<TypeId>>,
    kinds: Vec<Option<KindId>>,
    parameters: Vec<Option<TypeParameterId>>,
    rows: Vec<Option<RecordRowId>>,
    families: Vec<Option<FamilyTypeId>>,
    variants: Vec<Option<VariantTypeId>>,
    domains: Vec<Option<StaticDomainId>>,
    constructors: Vec<Option<AlgebraicStaticConstructorId>>,
    scoped: Vec<Option<ScopedTypeId>>,
    lambdas: Vec<Option<TypeLambdaId>>,

    decoded_types: Vec<Option<TypeData>>,
    decoded_type_kinds: Vec<Option<KindId>>,
    decoded_kinds: Vec<Option<KindData>>,
    decoded_parameters: Vec<Option<TypeParameterData>>,
    decoded_rows: Vec<Option<RecordRowData>>,
    decoded_families: Vec<Option<FamilyType>>,
    decoded_variants: Vec<Option<VariantId>>,
    decoded_domains: Vec<Option<StaticDomainData>>,
    decoded_constructors: Vec<Option<AlgebraicStaticConstructorData>>,
    decoded_scoped: Vec<Option<ScopedTypeData>>,
    decoded_lambdas: Vec<Option<TypeLambdaData>>,
}
macro_rules! resolve_node {
    ($($method:ident : $field:ident ($wire:ty) -> $native:ty, $node:ident, $build:ident),* $(,)?) => { $(
        fn $method(&mut self, id: $wire) -> Result<$native, ArtifactError> {
            let previous = self.$field.get(id.0 as usize).ok_or_else(|| invalid(concat!("out-of-range ", stringify!($field), " edge")))?;
            if let Some(value) = previous { return Ok(*value); }
            let node = Node::$node(id.0);
            if self.active.len() >= self.limits.max_recursion_depth { return Err(invalid("type graph dependency depth exceeds limit")); }
            if self.active.contains(&node) { return Err(invalid("cyclic structural type graph")); }
            self.active.push(node);
            let result = self.$build(id);
            self.active.pop();
            if let Ok(value) = result { self.$field[id.0 as usize] = Some(value); }
            result
        }
    )* };
}
impl<'a, 'b, 'c, 'd> Import<'a, 'b, 'c, 'd> {
    fn new(
        graph: &'a TypeGraphV1,
        owner_catalog: Option<&'c crate::artifact::StableOwnerCatalog<'c>>,
        receiver: Option<&'d mut TypeStore>,
        limits: &'a ValidationLimits,
        allocations: &'b mut AllocationBudget,
    ) -> Result<Self, ArtifactError> {
        fn make_options<T: Clone>(len: usize, allocations: &mut AllocationBudget) -> Result<Vec<Option<T>>, ArtifactError> {
            let mut v = allocations.vector(len).map_err(|e| ArtifactError::Allocation(e))?;
            v.resize(len, None);
            Ok(v)
        }

        let active = allocations.vector(limits.max_recursion_depth).map_err(|e| ArtifactError::Allocation(e))?;
        let types = make_options(graph.types.len(), allocations)?;
        let kinds = make_options(graph.kinds.len(), allocations)?;
        let parameters = make_options(graph.parameters.len(), allocations)?;
        let rows = make_options(graph.rows.len(), allocations)?;
        let families = make_options(graph.families.len(), allocations)?;
        let variants = make_options(graph.variants.len(), allocations)?;
        let domains = make_options(graph.static_domains.len(), allocations)?;
        let constructors = make_options(graph.static_constructors.len(), allocations)?;
        let scoped = make_options(graph.scoped.len(), allocations)?;
        let lambdas = make_options(graph.lambdas.len(), allocations)?;

        let decoded_types = make_options(graph.types.len(), allocations)?;
        let decoded_type_kinds = make_options(graph.types.len(), allocations)?;
        let decoded_kinds = make_options(graph.kinds.len(), allocations)?;
        let decoded_parameters = make_options(graph.parameters.len(), allocations)?;
        let decoded_rows = make_options(graph.rows.len(), allocations)?;
        let decoded_families = make_options(graph.families.len(), allocations)?;
        let decoded_variants = make_options(graph.variants.len(), allocations)?;
        let decoded_domains = make_options(graph.static_domains.len(), allocations)?;
        let decoded_constructors = make_options(graph.static_constructors.len(), allocations)?;
        let decoded_scoped = make_options(graph.scoped.len(), allocations)?;
        let decoded_lambdas = make_options(graph.lambdas.len(), allocations)?;

        Ok(Self {
            graph,
            owner_catalog,
            receiver,
            limits,
            allocations,
            active,
            types,
            kinds,
            parameters,
            rows,
            families,
            variants,
            domains,
            constructors,
            scoped,
            lambdas,
            decoded_types,
            decoded_type_kinds,
            decoded_kinds,
            decoded_parameters,
            decoded_rows,
            decoded_families,
            decoded_variants,
            decoded_domains,
            decoded_constructors,
            decoded_scoped,
            decoded_lambdas,
        })
    }

    fn kind_of(&self, ty: TypeId) -> Result<KindId, ArtifactError> {
        if let Some(receiver) = self.receiver.as_deref() {
            return Ok(receiver.kind_of(ty));
        }
        self.decoded_type_kinds
            .get(ty.index())
            .and_then(|k| *k)
            .ok_or_else(|| invalid("referenced type kind not resolved"))
    }

    fn get_type(&self, ty: TypeId) -> Result<&TypeData, ArtifactError> {
        if let Some(receiver) = self.receiver.as_deref() {
            return Ok(receiver.get(ty));
        }
        self.decoded_types
            .get(ty.index())
            .and_then(|t| t.as_ref())
            .ok_or_else(|| invalid("referenced type data not resolved"))
    }

    fn get_kind(&self, kind: KindId) -> Result<&KindData, ArtifactError> {
        if let Some(receiver) = self.receiver.as_deref() {
            return Ok(receiver.get_kind(kind));
        }
        self.decoded_kinds
            .get(kind.index())
            .and_then(|k| k.as_ref())
            .ok_or_else(|| invalid("referenced kind data not resolved"))
    }

    fn type_parameter(&self, parameter: TypeParameterId) -> Result<&TypeParameterData, ArtifactError> {
        if let Some(receiver) = self.receiver.as_deref() {
            return Ok(receiver.type_parameter(parameter));
        }
        self.decoded_parameters
            .get(parameter.index())
            .and_then(|p| p.as_ref())
            .ok_or_else(|| invalid("referenced type parameter not resolved"))
    }

    fn static_constructor(&self, constructor: AlgebraicStaticConstructorId) -> Result<&AlgebraicStaticConstructorData, ArtifactError> {
        if let Some(receiver) = self.receiver.as_deref() {
            return receiver.static_constructor(constructor).ok_or_else(|| invalid("referenced static constructor not resolved"));
        }
        self.decoded_constructors
            .get(constructor.index())
            .and_then(|c| c.as_ref())
            .ok_or_else(|| invalid("referenced static constructor not resolved"))
    }

    fn finish_kind(&mut self, id: ArtifactKindId, data: KindData) -> KindId {
        if let Some(receiver) = self.receiver.as_deref_mut() { return receiver.intern_kind(data); }
        self.decoded_kinds[id.0 as usize] = Some(data);
        KindId(id.0)
    }

    fn finish_parameter(&mut self, id: ArtifactTypeParameterId, data: TypeParameterData) -> TypeParameterId {
        if let Some(receiver) = self.receiver.as_deref_mut() { return receiver.intern_type_parameter(data); }
        self.decoded_parameters[id.0 as usize] = Some(data);
        TypeParameterId(id.0)
    }

    fn finish_row(&mut self, id: ArtifactRecordRowId, data: RecordRowData) -> RecordRowId {
        if let Some(receiver) = self.receiver.as_deref_mut() { return receiver.intern_record_row(data); }
        self.decoded_rows[id.0 as usize] = Some(data);
        RecordRowId(id.0)
    }

    fn finish_family(&mut self, id: ArtifactFamilyTypeId, data: FamilyType) -> FamilyTypeId {
        if let Some(receiver) = self.receiver.as_deref_mut() { return receiver.intern_family_data(data); }
        self.decoded_families[id.0 as usize] = Some(data);
        FamilyTypeId(id.0)
    }

    fn finish_variant(&mut self, id: ArtifactVariantTypeId, data: VariantId) -> VariantTypeId {
        if let Some(receiver) = self.receiver.as_deref_mut() { return receiver.intern_variant_identity(data); }
        self.decoded_variants[id.0 as usize] = Some(data);
        VariantTypeId(id.0)
    }

    fn finish_domain(&mut self, id: ArtifactStaticDomainId, data: StaticDomainData) -> StaticDomainId {
        if let Some(receiver) = self.receiver.as_deref_mut() { return receiver.intern_static_domain(data); }
        self.decoded_domains[id.0 as usize] = Some(data);
        StaticDomainId(id.0)
    }

    fn finish_constructor(&mut self, id: ArtifactStaticConstructorId, data: AlgebraicStaticConstructorData) -> AlgebraicStaticConstructorId {
        if let Some(receiver) = self.receiver.as_deref_mut() {
            return receiver.intern_static_constructor(data.domain, data.declaration, data.case_name, data.parameter_kinds);
        }
        self.decoded_constructors[id.0 as usize] = Some(data);
        AlgebraicStaticConstructorId(id.0)
    }

    fn finish_type(&mut self, id: ArtifactTypeId, data: TypeData, kind: KindId) -> TypeId {
        if let Some(receiver) = self.receiver.as_deref_mut() { return receiver.intern_with_kind(data, kind); }
        self.decoded_types[id.0 as usize] = Some(data);
        self.decoded_type_kinds[id.0 as usize] = Some(kind);
        TypeId(id.0)
    }

    fn finish_scoped(&mut self, id: ArtifactScopedTypeId, data: ScopedTypeData) -> ScopedTypeId {
        if let Some(receiver) = self.receiver.as_deref_mut() { return receiver.arena_mut().intern_scoped(data); }
        self.decoded_scoped[id.0 as usize] = Some(data);
        ScopedTypeId(id.0)
    }

    fn finish_lambda(&mut self, id: ArtifactTypeLambdaId, data: TypeLambdaData) -> TypeLambdaId {
        if let Some(receiver) = self.receiver.as_deref_mut() {
            return receiver.arena_mut().intern_lambda_with_modes(data.parameter_kinds, data.parameter_modes, data.body, data.result_kind, None);
        }
        self.decoded_lambdas[id.0 as usize] = Some(data);
        TypeLambdaId(id.0)
    }

    resolve_node!(ty: types(ArtifactTypeId) -> TypeId, Type, build_type, kind: kinds(ArtifactKindId) -> KindId, Kind, build_kind, parameter: parameters(ArtifactTypeParameterId) -> TypeParameterId, Parameter, build_parameter, row: rows(ArtifactRecordRowId) -> RecordRowId, Row, build_row, family: families(ArtifactFamilyTypeId) -> FamilyTypeId, Family, build_family, variant: variants(ArtifactVariantTypeId) -> VariantTypeId, Variant, build_variant, domain: domains(ArtifactStaticDomainId) -> StaticDomainId, Domain, build_domain, constructor: constructors(ArtifactStaticConstructorId) -> AlgebraicStaticConstructorId, Constructor, build_constructor, scoped: scoped(ArtifactScopedTypeId) -> ScopedTypeId, Scoped, build_scoped, lambda: lambdas(ArtifactTypeLambdaId) -> TypeLambdaId, Lambda, build_lambda);
    fn args(&mut self, values: &[StaticArgumentV1]) -> Result<Box<[StaticArgument]>, ArtifactError> {
        let mut result = self.allocations.vector(values.len()).map_err(|e| ArtifactError::Allocation(e))?;
        for value in values {
            result.push(match value {
                StaticArgumentV1::Term { term } => StaticArgument::Term(self.ty(*term)?),
                StaticArgumentV1::RecordRow { row } => StaticArgument::RecordRow(self.row(*row)?),
                StaticArgumentV1::DynamicType => StaticArgument::DynamicType,
            });
        }
        self.allocations.boxed_slice(result).map_err(|e| ArtifactError::Allocation(e))
    }
    fn type_list(&mut self, values: &[ArtifactTypeId]) -> Result<Box<[TypeId]>, ArtifactError> {
        let mut result = self.allocations.vector(values.len()).map_err(|e| ArtifactError::Allocation(e))?;
        for ty in values {
            result.push(self.ty(*ty)?);
        }
        self.allocations.boxed_slice(result).map_err(|e| ArtifactError::Allocation(e))
    }
    fn scope_list(&mut self, values: &[ArtifactScopedTypeId]) -> Result<Box<[ScopedTypeId]>, ArtifactError> {
        let mut result = self.allocations.vector(values.len()).map_err(|e| ArtifactError::Allocation(e))?;
        for ty in values {
            result.push(self.scoped(*ty)?);
        }
        self.allocations.boxed_slice(result).map_err(|e| ArtifactError::Allocation(e))
    }
    fn kind_list(&mut self, values: &[ArtifactKindId]) -> Result<Box<[KindId]>, ArtifactError> {
        let mut result = self.allocations.vector(values.len()).map_err(|e| ArtifactError::Allocation(e))?;
        for kind in values {
            result.push(self.kind(*kind)?);
        }
        self.allocations.boxed_slice(result).map_err(|e| ArtifactError::Allocation(e))
    }
    fn remainder(&mut self, value: &TupleRemainderV1) -> Result<AnonymousTupleRemainder, ArtifactError> {
        Ok(match value {
            TupleRemainderV1::None => AnonymousTupleRemainder::None,
            TupleRemainderV1::Unconstrained => AnonymousTupleRemainder::Unconstrained,
            TupleRemainderV1::Typed { ty } => AnonymousTupleRemainder::Typed(self.ty(*ty)?),
        })
    }
    fn tuple(&mut self, value: &TupleV1) -> Result<TupleType, ArtifactError> {
        let mut elements = self.allocations.vector(value.elements.len()).map_err(|e| ArtifactError::Allocation(e))?;
        for element in &value.elements {
            let label = match &element.label {
                Some(label) => Some(self.allocations.boxed_str(label).map_err(|e| ArtifactError::Allocation(e))?),
                None => None,
            };
            elements.push(TupleTypeElement {
                label,
                ty: self.ty(element.ty)?,
            });
        }
        let elements = self.allocations.boxed_slice(elements).map_err(|e| ArtifactError::Allocation(e))?;
        Ok(TupleType {
            elements,
            positional_remainder: self.remainder(&value.positionals)?,
            labeled_remainder: self.remainder(&value.labeled)?,
        })
    }
    fn field_with_allocation_budget(&mut self, value: &FieldV1) -> Result<RecordRowField, ArtifactError> {
        Ok(RecordRowField {
            key: key_in_with_allocation_budget(&value.key, self.limits, self.allocations)?,
            ty: self.ty(value.ty)?,
            presence: presence_in(&value.presence),
        })
    }
    fn scoped_fields(&mut self, values: &[ScopedFieldV1]) -> Result<Box<[ScopedRecordField]>, ArtifactError> {
        let mut result = self.allocations.vector(values.len()).map_err(|e| ArtifactError::Allocation(e))?;
        for value in values {
            result.push(ScopedRecordField {
                key: key_in_with_allocation_budget(&value.key, self.limits, self.allocations)?,
                ty: self.scoped(value.ty)?,
                presence: presence_in(&value.presence),
            });
        }
        self.allocations.boxed_slice(result).map_err(|e| ArtifactError::Allocation(e))
    }
    fn scoped_elements(&mut self, values: &[ScopedElementV1]) -> Result<Box<[ScopedTupleElement]>, ArtifactError> {
        let mut result = self.allocations.vector(values.len()).map_err(|e| ArtifactError::Allocation(e))?;
        for value in values {
            let label = match &value.label {
                Some(label) => Some(self.allocations.boxed_str(label).map_err(|e| ArtifactError::Allocation(e))?),
                None => None,
            };
            result.push(ScopedTupleElement {
                label,
                ty: self.scoped(value.ty)?,
            });
        }
        self.allocations.boxed_slice(result).map_err(|e| ArtifactError::Allocation(e))
    }
    fn scoped_remainder(&mut self, value: &ScopedRemainderV1) -> Result<ScopedPackRemainder, ArtifactError> {
        Ok(match value {
            ScopedRemainderV1::None => ScopedPackRemainder::None,
            ScopedRemainderV1::Unconstrained => ScopedPackRemainder::Unconstrained,
            ScopedRemainderV1::Typed { ty } => ScopedPackRemainder::Typed(self.scoped(*ty)?),
        })
    }
    fn scoped_tail(&mut self, value: &ScopedRowTailV1) -> Result<ScopedRecordRowTail, ArtifactError> {
        Ok(match value {
            ScopedRowTailV1::Closed => ScopedRecordRowTail::Closed,
            ScopedRowTailV1::Bound { depth, index } => ScopedRecordRowTail::Bound { depth: *depth, index: *index },
            ScopedRowTailV1::Parameter { parameter } => ScopedRecordRowTail::FreeParameter(self.parameter(*parameter)?),
            ScopedRowTailV1::Projection { ty } => ScopedRecordRowTail::Projection(self.ty(*ty)?),
            ScopedRowTailV1::Unconstrained { key_type } => ScopedRecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained {
                key_type: self.scoped(*key_type)?,
            }),
            ScopedRowTailV1::Typed { key_type, value_type } => ScopedRecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
                key_type: self.scoped(*key_type)?,
                value_type: self.scoped(*value_type)?,
            }),
        })
    }
    fn build_kind(&mut self, id: ArtifactKindId) -> Result<KindId, ArtifactError> {
        let data = match &self.graph.kinds[id.0 as usize] {
            KindV1::Type => KindData::Type,
            KindV1::RecordRow => KindData::RecordRow,
            KindV1::TypePack => KindData::TypePack,
            KindV1::LabeledTypePack => KindData::LabeledTypePack,
            KindV1::CompleteTypePack => KindData::CompleteTypePack,
            KindV1::Arrow { parameters, result } => {
                if parameters.is_empty() {
                    return Err(invalid("empty kind arrow"));
                }
                KindData::Arrow {
                    parameters: self.kind_list(parameters)?,
                    result: self.kind(*result)?,
                }
            }
            KindV1::StaticDomain { domain } => KindData::StaticDomain(self.domain(*domain)?),
        };
        Ok(self.finish_kind(id, data))
    }
    fn build_parameter(&mut self, id: ArtifactTypeParameterId) -> Result<TypeParameterId, ArtifactError> {
        let data = &self.graph.parameters[id.0 as usize];
        let owner = owner_in_with_allocation_budget(&data.owner, self.allocations, self.owner_catalog)?;
        let kind = self.kind(data.kind)?;
        if !crate::types::parameter::expansion_mode_accepts_kind(mode_in(&data.mode), kind) {
            return Err(invalid("parameter expansion mode has incompatible kind"));
        }
        let name = self.allocations.boxed_str(&data.name).map_err(|e| ArtifactError::Allocation(e))?;
        let param_data = TypeParameterData {
            owner,
            index: data.index,
            name,
            kind,
            variance: variance_in(&data.variance),
            mode: mode_in(&data.mode),
            source: None,
        };
        Ok(self.finish_parameter(id, param_data))
    }
    fn build_domain(&mut self, id: ArtifactStaticDomainId) -> Result<StaticDomainId, ArtifactError> {
        let data = match &self.graph.static_domains[id.0 as usize] {
            StaticDomainV1::ConstInt => StaticDomainData::ConstInt,
            StaticDomainV1::Nominal { declaration } => StaticDomainData::Nominal {
                declaration: declaration_in_with_allocation_budget(declaration, self.allocations, self.owner_catalog)?,
            },
            StaticDomainV1::Associated { declaration, arguments } => StaticDomainData::Associated {
                declaration: AssociatedStaticDomainDeclarationId::new(
                    declaration_in_with_allocation_budget(&declaration.owner, self.allocations, self.owner_catalog)?,
                    self.allocations.boxed_str(&declaration.name).map_err(ArtifactError::Allocation)?,
                ),
                owner_arguments: self.args(arguments)?,
            },
        };
        Ok(self.finish_domain(id, data))
    }
    fn build_constructor(&mut self, id: ArtifactStaticConstructorId) -> Result<AlgebraicStaticConstructorId, ArtifactError> {
        let data = &self.graph.static_constructors[id.0 as usize];
        let domain = self.domain(data.domain)?;
        let declaration = declaration_in_with_allocation_budget(&data.declaration, self.allocations, self.owner_catalog)?;
        let kinds = self.kind_list(&data.parameter_kinds)?;
        if data.case_name.is_empty() {
            return Err(invalid("empty static constructor name"));
        }
        let case_name = self.allocations.boxed_str(&data.case_name).map_err(|e| ArtifactError::Allocation(e))?;
        let ctor_data = AlgebraicStaticConstructorData {
            domain,
            declaration,
            case_name,
            parameter_kinds: kinds,
        };
        Ok(self.finish_constructor(id, ctor_data))
    }
    fn build_variant(&mut self, id: ArtifactVariantTypeId) -> Result<VariantTypeId, ArtifactError> {
        let data = &self.graph.variants[id.0 as usize];
        let owner = declaration_in_with_allocation_budget(&data.owner, self.allocations, self.owner_catalog)?;
        let selector = crate::artifact::import::import_stable_selector_with_allocation_budget(&data.selector, self.allocations)?;
        let variant = VariantId::new(owner, selector);
        Ok(self.finish_variant(id, variant))
    }
    fn build_row(&mut self, id: ArtifactRecordRowId) -> Result<RecordRowId, ArtifactError> {
        let data = &self.graph.rows[id.0 as usize];
        let mut fields = self.allocations.vector(data.fields.len()).map_err(ArtifactError::Allocation)?;
        for field in &data.fields {
            fields.push(self.field_with_allocation_budget(field)?);
        }
        if fields
            .windows(2)
            .any(|pair| pair[0].key.canonical_cmp(&pair[1].key) != std::cmp::Ordering::Less)
        {
            return Err(invalid("record fields are not canonically unique and ordered"));
        }
        let tail = match data.tail {
            RowTailV1::Closed => RecordRowTail::Closed,
            RowTailV1::Unconstrained { key_type } => RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: self.ty(key_type)? }),
            RowTailV1::Typed { key_type, value_type } => RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
                key_type: self.ty(key_type)?,
                value_type: self.ty(value_type)?,
            }),
            RowTailV1::Parameter { parameter } => RecordRowTail::Parameter(self.parameter(parameter)?),
            RowTailV1::Projection { ty } => RecordRowTail::Projection(self.ty(ty)?),
        };
        let fields = self.allocations.boxed_slice(fields).map_err(ArtifactError::Allocation)?;
        let row_data = RecordRowData { fields, tail };
        Ok(self.finish_row(id, row_data))
    }
    fn build_family(&mut self, id: ArtifactFamilyTypeId) -> Result<FamilyTypeId, ArtifactError> {
        let data = &self.graph.families[id.0 as usize];
        let mut members = self.allocations.vector(data.members.len()).map_err(ArtifactError::Allocation)?;
        for member in &data.members {
            members.push(FamilyMemberType {
                operation: operation_in_with_allocation_budget(&member.operation, self.allocations)?,
                member_kind: match member.kind {
                    FamilyMemberKindV1::Value => FamilyMemberTypeKind::Value,
                    FamilyMemberKindV1::Callable => FamilyMemberTypeKind::Callable,
                },
                ty: self.ty(member.ty)?,
            });
        }
        if members.windows(2).any(|pair| pair[0].operation >= pair[1].operation) {
            return Err(invalid("family operations are not unique and ordered"));
        }
        for member in &members {
            if member.member_kind == FamilyMemberTypeKind::Callable {
                let ty_data = self.get_type(member.ty)?;
                if !matches!(ty_data, TypeData::Callable(_)) {
                    return Err(invalid("family callable member type is not callable"));
                }
            }
        }
        let members = self.allocations.boxed_slice(members).map_err(ArtifactError::Allocation)?;
        let family = FamilyType::new(members);
        Ok(self.finish_family(id, family))
    }
    fn build_type(&mut self, id: ArtifactTypeId) -> Result<TypeId, ArtifactError> {
        let entry = &self.graph.types[id.0 as usize];
        let kind = self.kind(entry.kind)?;
        let data = match &entry.form {
            TypeV1::Never => TypeData::Never,
            TypeV1::Unit => TypeData::Unit,
            TypeV1::ClassObject { declaration } => TypeData::ClassObject {
                declaration: declaration_in_with_allocation_budget(declaration, self.allocations, self.owner_catalog)?,
            },
            TypeV1::Nominal { declaration } => TypeData::Nominal {
                declaration: declaration_in_with_allocation_budget(declaration, self.allocations, self.owner_catalog)?,
            },
            TypeV1::StaticDomainForm { domain } => TypeData::StaticDomainForm { domain: self.domain(*domain)? },
            TypeV1::Applied { origin, arguments } => TypeData::Applied {
                origin: self.ty(*origin)?,
                arguments: self.args(arguments)?,
            },
            TypeV1::ExactCase { variant, enum_type } => TypeData::ExactCase {
                variant: self.variant(*variant)?,
                enum_type: self.ty(*enum_type)?,
            },
            TypeV1::Union { members } => {
                let mut members = self.type_list(members)?.into_vec();
                members.sort_unstable();
                if members.len() < 2 || members.windows(2).any(|pair| pair[0] == pair[1]) {
                    return Err(invalid("noncanonical union members"));
                }
                TypeData::Union(self.allocations.boxed_slice(members).map_err(|e| ArtifactError::Allocation(e))?)
            }
            TypeV1::Tuple { tuple } => TypeData::Tuple(self.tuple(tuple)?),
            TypeV1::Record { row } => TypeData::Record(self.row(*row)?),
            TypeV1::TypePack { members } => TypeData::TypePack(self.type_list(members)?),
            TypeV1::LabeledTypePack { members } => {
                let mut elements = self.allocations.vector(members.len()).map_err(|e| ArtifactError::Allocation(e))?;
                for value in members {
                    let label = self.allocations.boxed_str(&value.label).map_err(|e| ArtifactError::Allocation(e))?;
                    elements.push(LabeledTypePackElement {
                        label,
                        element: self.ty(value.ty)?,
                    });
                }
                TypeData::LabeledTypePack(self.allocations.boxed_slice(elements).map_err(|e| ArtifactError::Allocation(e))?)
            }
            TypeV1::CompleteTypePack { positionals, labeled } => TypeData::CompleteTypePack {
                positionals: self.ty(*positionals)?,
                labeled: self.ty(*labeled)?,
            },
            TypeV1::TypePackConcat { members } => TypeData::TypePackConcat(self.type_list(members)?),
            TypeV1::LabeledTypePackConcat { members } => TypeData::LabeledTypePackConcat(self.type_list(members)?),
            TypeV1::TupleFromPacks { positionals, labeled } => TypeData::TupleFromPacks {
                positionals: self.ty(*positionals)?,
                labeled: self.ty(*labeled)?,
            },
            TypeV1::Pack { pack } => {
                let tuple = self.tuple(pack)?;
                TypeData::Pack(PackType {
                    elements: tuple.elements,
                    positional_remainder: tuple.positional_remainder,
                    labeled_remainder: tuple.labeled_remainder,
                })
            }
            TypeV1::PackFromPacks { positionals, labeled } => TypeData::PackFromPacks {
                positionals: self.ty(*positionals)?,
                labeled: self.ty(*labeled)?,
            },
            TypeV1::CallableFromPacks { positionals, labeled, result } => TypeData::CallableFromPacks {
                positionals: self.ty(*positionals)?,
                labeled: self.ty(*labeled)?,
                return_type: self.ty(*result)?,
            },
            TypeV1::Callable { callable } => {
                let mut parameters = self.allocations.vector(callable.parameters.len()).map_err(|e| ArtifactError::Allocation(e))?;
                for parameter in &callable.parameters {
                    let label = match &parameter.label {
                        Some(label) => Some(self.allocations.boxed_str(label).map_err(|e| ArtifactError::Allocation(e))?),
                        None => None,
                    };
                    parameters.push(CallableParameterType {
                        label,
                        ty: self.ty(parameter.ty)?,
                        rest: rest_in(&parameter.rest),
                    });
                }
                let parameters = self.allocations.boxed_slice(parameters).map_err(|e| ArtifactError::Allocation(e))?;
                TypeData::Callable(CallableType {
                    parameters,
                    return_type: self.ty(callable.result)?,
                })
            }
            TypeV1::Family { family } => TypeData::Family(self.family(*family)?),
            TypeV1::Parameter { parameter } => TypeData::Parameter(self.parameter(*parameter)?),
            TypeV1::Lambda { lambda } => TypeData::Lambda(self.lambda(*lambda)?),
            TypeV1::SelfType { self_type } => TypeData::SelfType(SelfTypeTerm {
                owner: declaration_in_with_allocation_budget(&self_type.owner, self.allocations, self.owner_catalog)?,
                side: match self_type.side {
                    StableDispatchSide::Instance => crate::identity::DispatchSide::Instance,
                    StableDispatchSide::Class => crate::identity::DispatchSide::Class,
                },
                role: match self_type.role {
                    SelfRoleV1::InstanceType => SelfRole::InstanceType,
                    SelfRoleV1::ReceiverValue => SelfRole::ReceiverValue,
                },
            }),
            TypeV1::AssociatedProjection { projection } => {
                let trait_owner = declaration_in_with_allocation_budget(&projection.trait_owner, self.allocations, self.owner_catalog)?;
                let requirement_owner = declaration_in_with_allocation_budget(&projection.requirement_owner, self.allocations, self.owner_catalog)?;
                if trait_owner != requirement_owner {
                    return Err(invalid("associated requirement owner differs from its trait"));
                }
                TypeData::AssociatedProjection(AssociatedTypeProjection {
                    subject: self.ty(projection.subject)?,
                    trait_ref: TraitRef {
                        declaration: trait_owner,
                        arguments: self.type_list(&projection.trait_arguments)?,
                    },
                    requirement: AssociatedTypeRequirementId::new(requirement_owner, projection.requirement_index),
                })
            }
            TypeV1::TypeMemberProjection { projection } => {
                let mut arguments = self.allocations.vector(projection.arguments.len()).map_err(ArtifactError::Allocation)?;
                for argument in &projection.arguments {
                    arguments.push(match argument {
                        TypeMemberArgumentV1::Type { ty } => TypeMemberArgument::Type(self.ty(*ty)?),
                        TypeMemberArgumentV1::ConstInt { decimal } => {
                            TypeMemberArgument::Const(ConstTerm::Int(integer_in_with_allocation_budget(decimal, self.limits, self.allocations)?))
                        }
                        TypeMemberArgumentV1::RecordRow { row } => TypeMemberArgument::RecordRow(self.row(*row)?),
                    });
                }
                let arguments = self.allocations.boxed_slice(arguments).map_err(ArtifactError::Allocation)?;

                TypeData::TypeMemberProjection(TypeLevelProjection {
                    subject: match projection.subject {
                        ProjectionSubjectV1::Type { ty } => ProjectionSubject::Type(self.ty(ty)?),
                        ProjectionSubjectV1::RecordRow { row } => ProjectionSubject::RecordRow(self.row(row)?),
                    },
                    selector: selector_in_with_allocation_budget(&projection.selector, self.allocations)?,
                    arguments,
                    source_declaration: projection
                        .source
                        .as_ref()
                        .map(|source| {
                            Ok::<_, ArtifactError>(TypeMemberDeclarationId::new(
                                declaration_in_with_allocation_budget(&source.owner, self.allocations, self.owner_catalog)?,
                                if source.selector.as_ref() == "[]" {
                                    TypeMemberSelector::Subscript
                                } else {
                                    TypeMemberSelector::Named(self.allocations.boxed_str(&source.selector).map_err(ArtifactError::Allocation)?)
                                },
                            ))
                        })
                        .transpose()?,
                })
            }
            TypeV1::ConstInt { decimal } => TypeData::ConstInt(integer_in_with_allocation_budget(decimal, self.limits, self.allocations)?),
            TypeV1::StaticData {
                domain,
                constructor,
                arguments,
            } => TypeData::StaticData(AlgebraicStaticTerm {
                domain: self.domain(*domain)?,
                constructor: self.constructor(*constructor)?,
                arguments: self.args(arguments)?,
            }),
        };
        self.validate_type(&data, kind)?;
        Ok(self.finish_type(id, data, kind))
    }
    fn build_scoped(&mut self, id: ArtifactScopedTypeId) -> Result<ScopedTypeId, ArtifactError> {
        let data = match &self.graph.scoped[id.0 as usize] {
            ScopedTypeV1::Bound { depth, index } => ScopedTypeData::Bound { depth: *depth, index: *index },
            ScopedTypeV1::Free { ty } => ScopedTypeData::Free(self.ty(*ty)?),
            ScopedTypeV1::Applied { origin, arguments } => ScopedTypeData::Applied {
                origin: self.scoped(*origin)?,
                arguments: self.scope_list(arguments)?,
            },
            ScopedTypeV1::Union { members } => ScopedTypeData::Union(self.scope_list(members)?),
            ScopedTypeV1::Tuple { elements } => ScopedTypeData::Tuple(self.scoped_elements(elements)?),
            ScopedTypeV1::Record { fields } => ScopedTypeData::Record(self.scoped_fields(fields)?),
            ScopedTypeV1::OpenRecord { row } => {
                let fields = self.scoped_fields(&row.fields)?;
                let tail = match self.scoped_tail(&row.tail)? {
                    ScopedRecordRowTail::Bound { depth, index } => ScopedRecordTail::Bound { depth, index },
                    ScopedRecordRowTail::FreeParameter(parameter) => ScopedRecordTail::FreeParameter(parameter),
                    ScopedRecordRowTail::Anonymous(tail) => ScopedRecordTail::Anonymous(tail),
                    _ => return Err(invalid("invalid open record tail")),
                };
                ScopedTypeData::OpenRecord(ScopedOpenRecord { fields, tail })
            }
            ScopedTypeV1::RecordRow { row } => ScopedTypeData::RecordRow(ScopedRecordRow {
                fields: self.scoped_fields(&row.fields)?,
                tail: self.scoped_tail(&row.tail)?,
            }),
            ScopedTypeV1::Callable { callable } => {
                let mut parameters = self.allocations.vector(callable.parameters.len()).map_err(|e| ArtifactError::Allocation(e))?;
                for parameter in &callable.parameters {
                    let label = match &parameter.label {
                        Some(label) => Some(self.allocations.boxed_str(label).map_err(|e| ArtifactError::Allocation(e))?),
                        None => None,
                    };
                    parameters.push(ScopedCallableParameter {
                        label,
                        ty: self.scoped(parameter.ty)?,
                        rest: parameter.rest,
                    });
                }
                let parameters = self.allocations.boxed_slice(parameters).map_err(|e| ArtifactError::Allocation(e))?;
                ScopedTypeData::Callable(ScopedCallableType {
                    parameters,
                    return_type: self.scoped(callable.result)?,
                })
            }
            ScopedTypeV1::Pack { pack } => ScopedTypeData::Pack(ScopedPackType {
                elements: self.scoped_elements(&pack.elements)?,
                positional_remainder: self.scoped_remainder(&pack.positionals)?,
                labeled_remainder: self.scoped_remainder(&pack.labeled)?,
            }),
            ScopedTypeV1::TypePack { members } => ScopedTypeData::TypePack(self.scope_list(members)?),
            ScopedTypeV1::LabeledTypePack { members } => {
                let mut elements = self.allocations.vector(members.len()).map_err(|e| ArtifactError::Allocation(e))?;
                for member in members {
                    let label = self.allocations.boxed_str(&member.label).map_err(|e| ArtifactError::Allocation(e))?;
                    elements.push(ScopedLabeledTypePackElement {
                        label,
                        element: self.scoped(member.ty)?,
                    });
                }
                ScopedTypeData::LabeledTypePack(self.allocations.boxed_slice(elements).map_err(|e| ArtifactError::Allocation(e))?)
            }
            ScopedTypeV1::CompleteTypePack { positionals, labeled } => ScopedTypeData::CompleteTypePack {
                positionals: self.scoped(*positionals)?,
                labeled: self.scoped(*labeled)?,
            },
            ScopedTypeV1::TypePackConcat { members } => ScopedTypeData::TypePackConcat(self.scope_list(members)?),
            ScopedTypeV1::LabeledTypePackConcat { members } => ScopedTypeData::LabeledTypePackConcat(self.scope_list(members)?),
            ScopedTypeV1::TupleFromPacks { positionals, labeled } => ScopedTypeData::TupleFromPacks {
                positionals: self.scoped(*positionals)?,
                labeled: self.scoped(*labeled)?,
            },
            ScopedTypeV1::PackFromPacks { positionals, labeled } => ScopedTypeData::PackFromPacks {
                positionals: self.scoped(*positionals)?,
                labeled: self.scoped(*labeled)?,
            },
            ScopedTypeV1::CallableFromPacks { positionals, labeled, result } => ScopedTypeData::CallableFromPacks {
                positionals: self.scoped(*positionals)?,
                labeled: self.scoped(*labeled)?,
                return_type: self.scoped(*result)?,
            },
            ScopedTypeV1::PackProjection { subject, lane } => ScopedTypeData::PackProjection {
                subject: self.scoped(*subject)?,
                lane: match lane {
                    PackLaneV1::Positionals => ScopedPackProjectionLane::Positionals,
                    PackLaneV1::Labeled => ScopedPackProjectionLane::Labeled,
                    PackLaneV1::Complete => ScopedPackProjectionLane::Complete,
                },
            },
            ScopedTypeV1::Lambda { lambda } => ScopedTypeData::Lambda(self.lambda(*lambda)?),
        };
        Ok(self.finish_scoped(id, data))
    }
    fn build_lambda(&mut self, id: ArtifactTypeLambdaId) -> Result<TypeLambdaId, ArtifactError> {
        let value = &self.graph.lambdas[id.0 as usize];
        if value.parameter_kinds.len() != value.parameter_modes.len() {
            return Err(invalid("lambda binder kind/mode count mismatch"));
        }
        let kinds = self.kind_list(&value.parameter_kinds)?;
        let mut modes_vec = self.allocations.vector(value.parameter_modes.len()).map_err(|e| ArtifactError::Allocation(e))?;
        for mode in &value.parameter_modes {
            modes_vec.push(mode_in(mode));
        }
        let modes = self.allocations.boxed_slice(modes_vec).map_err(|e| ArtifactError::Allocation(e))?;
        for (kind, mode) in kinds.iter().zip(modes.iter()) {
            if !crate::types::parameter::expansion_mode_accepts_kind(*mode, *kind) {
                return Err(invalid("lambda binder expansion kind mismatch"));
            }
        }
        let body = self.scoped(value.body)?;
        let result = self.kind(value.result_kind)?;
        let lambda_data = TypeLambdaData {
            parameter_kinds: kinds,
            parameter_modes: modes,
            body,
            result_kind: result,
        };
        Ok(self.finish_lambda(id, lambda_data))
    }
    fn validate_type(&self, data: &TypeData, kind: KindId) -> Result<(), ArtifactError> {
        let proper = |ty| {
            if self.kind_of(ty)? == KindId::TYPE {
                Ok(())
            } else {
                Err(invalid("structural component is not a proper type"))
            }
        };
        match data {
            TypeData::Parameter(parameter) => {
                let parameter_kind = self.type_parameter(*parameter)?.kind;
                if parameter_kind == KindId::RECORD_ROW {
                    return Err(invalid("RecordRow parameter cannot enter the TypeId domain"));
                }
                if kind != parameter_kind {
                    return Err(invalid("parameter type form has mismatched kind"));
                }
            }
            TypeData::Never
            | TypeData::Unit
            | TypeData::ClassObject { .. }
            | TypeData::SelfType(_)
            | TypeData::AssociatedProjection(_)
            | TypeData::Tuple(_)
            | TypeData::Record(_)
            | TypeData::Pack(_)
            | TypeData::Callable(_)
            | TypeData::Family(_)
            | TypeData::ExactCase { .. }
            | TypeData::TupleFromPacks { .. }
            | TypeData::PackFromPacks { .. }
            | TypeData::CallableFromPacks { .. }
            | TypeData::Union(_)
                if kind != KindId::TYPE =>
            {
                return Err(invalid("proper type form has non-Type kind"));
            }
            TypeData::TypePack(_) | TypeData::TypePackConcat(_) if kind != KindId::TYPE_PACK => return Err(invalid("positional pack form has wrong kind")),
            TypeData::LabeledTypePack(_) | TypeData::LabeledTypePackConcat(_) if kind != KindId::LABELED_TYPE_PACK => {
                return Err(invalid("labeled pack form has wrong kind"));
            }
            TypeData::CompleteTypePack { .. } if kind != KindId::COMPLETE_TYPE_PACK => return Err(invalid("complete pack form has wrong kind")),
            _ => {}
        }
        match data {
            TypeData::Tuple(value) => {
                for element in &value.elements {
                    proper(element.ty)?;
                }
                for remainder in [value.positional_remainder, value.labeled_remainder] {
                    if let AnonymousTupleRemainder::Typed(ty) = remainder {
                        proper(ty)?;
                    }
                }
            }
            TypeData::Pack(value) => {
                for element in &value.elements {
                    proper(element.ty)?;
                }
                for remainder in [value.positional_remainder, value.labeled_remainder] {
                    if let AnonymousTupleRemainder::Typed(ty) = remainder {
                        proper(ty)?;
                    }
                }
            }
            TypeData::Callable(value) => {
                for parameter in &value.parameters {
                    proper(parameter.ty)?;
                }
                proper(value.return_type)?;
            }
            TypeData::Union(members) => {
                for ty in members {
                    proper(*ty)?;
                    if matches!(self.get_type(*ty)?, TypeData::Union(_) | TypeData::Never) {
                        return Err(invalid("union is not flattened"));
                    }
                }
            }
            TypeData::TypePack(members) => {
                for ty in members {
                    proper(*ty)?;
                }
            }
            TypeData::LabeledTypePack(members) => {
                for (i, member) in members.iter().enumerate() {
                    proper(member.element)?;
                    for other in &members[..i] {
                        if member.label == other.label {
                            return Err(invalid("duplicate labeled pack label"));
                        }
                    }
                }
            }
            TypeData::ConstInt(_) => {
                let kind_data = self.get_kind(kind)?;
                let matches_const_int = match kind_data {
                    KindData::StaticDomain(domain) => self.decoded_domains.get(domain.index()).and_then(|d| d.as_ref()) == Some(&StaticDomainData::ConstInt),
                    _ => false,
                };
                if !matches_const_int {
                    return Err(invalid("integer term has wrong static domain kind"));
                }
            }
            TypeData::StaticDomainForm { domain } => {
                if !matches!(self.get_kind(kind)?, KindData::StaticDomain(value) if value == domain) {
                    return Err(invalid("static form has wrong domain kind"));
                }
            }
            TypeData::StaticData(value) => {
                if !matches!(self.get_kind(kind)?, KindData::StaticDomain(domain) if *domain == value.domain) {
                    return Err(invalid("static term has wrong domain kind"));
                }
                let constructor = self.static_constructor(value.constructor)?;
                if constructor.domain != value.domain || constructor.parameter_kinds.len() != value.arguments.len() {
                    return Err(invalid("static constructor domain/arity mismatch"));
                }
                for (argument, expected) in value.arguments.iter().zip(constructor.parameter_kinds.iter()) {
                    let actual = match argument {
                        StaticArgument::Term(ty) => self.kind_of(*ty)?,
                        StaticArgument::RecordRow(_) => KindId::RECORD_ROW,
                        StaticArgument::DynamicType => KindId::TYPE,
                    };
                    if &actual != expected {
                        return Err(invalid("static constructor argument kind mismatch"));
                    }
                }
            }
            TypeData::CompleteTypePack { positionals, labeled }
            | TypeData::TupleFromPacks { positionals, labeled }
            | TypeData::PackFromPacks { positionals, labeled }
            | TypeData::CallableFromPacks { positionals, labeled, .. } => {
                if self.kind_of(*positionals)? != KindId::TYPE_PACK || self.kind_of(*labeled)? != KindId::LABELED_TYPE_PACK {
                    return Err(invalid("pack lane kind mismatch"));
                }
                if let TypeData::CallableFromPacks { return_type, .. } = data {
                    proper(*return_type)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
fn declaration_in_with_allocation_budget(value: &StableDeclarationRef, allocations: &mut AllocationBudget, owner_catalog: Option<&crate::artifact::StableOwnerCatalog<'_>>) -> Result<DeclarationId, ArtifactError> {
    if value.path.len() != 1 || value.path[0].is_empty() {
        return Err(invalid("invalid canonical declaration path"));
    }
    let module = module_in_with_allocation_budget(&value.module, allocations, owner_catalog)?;
    let name = allocations.boxed_str(&value.path[0]).map_err(ArtifactError::Allocation)?;
    Ok(DeclarationId::new(module, name))
}
fn module_in_with_allocation_budget(
    value: &StableModuleRef,
    allocations: &mut AllocationBudget,
    owner_catalog: Option<&crate::artifact::StableOwnerCatalog<'_>>,
) -> Result<ModuleId, ArtifactError> {
    if let Some(catalog) = owner_catalog {
        // A stable package owner becomes native only through the exact
        // module rows/external allowlist and the trusted project catalog.
        return catalog.module_parts_with_allocation_budget(&value.project, &value.path, allocations);
    }
    if !matches!(&value.project, StableProjectRef::Builtin { namespace, version } if namespace.as_ref() == "universe" && version.as_ref() == "0.1.0") {
        return Err(invalid("canonical type owner has incompatible project identity"));
    }
    crate::artifact::import::import_stable_module_with_allocation_budget(value, allocations)
}
fn callable_in_with_allocation_budget(value: &StableCallableRef, allocations: &mut AllocationBudget, owner_catalog: Option<&crate::artifact::StableOwnerCatalog<'_>>) -> Result<CallableId, ArtifactError> {
    let owner = match &value.owner {
        StableCallableOwnerRef::Declaration(owner) => CallableOwnerId::Declaration(declaration_in_with_allocation_budget(owner, allocations, owner_catalog)?),
        StableCallableOwnerRef::Variant(variant) => CallableOwnerId::Variant(VariantId::new(
            declaration_in_with_allocation_budget(&variant.owner, allocations, owner_catalog)?,
            crate::artifact::import::import_stable_selector_with_allocation_budget(&variant.selector, allocations)?,
        )),
        StableCallableOwnerRef::Conformance(owner) => {
            let module = module_in_with_allocation_budget(&owner.module, allocations, owner_catalog)?;
            CallableOwnerId::Conformance(crate::identity::ImplId::new(module, crate::identity::ImplLocalId(owner.local_id)))
        }
    };
    Ok(CallableId {
        owner,
        side: match value.side {
            StableDispatchSide::Instance => crate::identity::DispatchSide::Instance,
            StableDispatchSide::Class => crate::identity::DispatchSide::Class,
        },
        selector: crate::artifact::import::import_stable_selector_with_allocation_budget(&value.selector, allocations)?,
    })
}
fn owner_in_with_allocation_budget(value: &ParameterOwnerV1, allocations: &mut AllocationBudget, owner_catalog: Option<&crate::artifact::StableOwnerCatalog<'_>>) -> Result<TypeParameterOwner, ArtifactError> {
    Ok(match value {
        ParameterOwnerV1::Declaration { owner } => TypeParameterOwner::Declaration(declaration_in_with_allocation_budget(owner, allocations, owner_catalog)?),
        ParameterOwnerV1::Callable { owner } => TypeParameterOwner::Callable(callable_in_with_allocation_budget(owner, allocations, owner_catalog)?),
        ParameterOwnerV1::Impl { owner } => {
            let module = module_in_with_allocation_budget(&owner.module, allocations, owner_catalog)?;
            TypeParameterOwner::Impl(crate::identity::ImplId::new(module, crate::identity::ImplLocalId(owner.local_id)))
        }
    })
}
pub(crate) fn integer_in_with_allocation_budget(
    decimal: &str,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<StaticInt, ArtifactError> {
    if decimal.len() > limits.max_string_bytes {
        return Err(invalid("static integer exceeds byte budget"));
    }
    let (negative, digits) = match decimal.strip_prefix('-') {
        Some(digits) => (true, digits),
        None => (false, decimal),
    };
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) || (digits.len() > 1 && digits.starts_with('0')) || (negative && digits == "0") {
        return Err(invalid("static integer is not normalized decimal"));
    }
    if let Ok(small) = decimal.parse::<i64>() {
        return Ok(StaticInt::Small(small));
    }

    let limbs = digits
        .len()
        .checked_mul(4)
        .and_then(|bits| bits.checked_add(31))
        .map(|bits| bits / 32)
        .ok_or(ArtifactError::Allocation(phalcom_type_meta::allocation::AllocationError::Overflow))?;
    let mut words = allocations.vector::<u32>(limbs).map_err(ArtifactError::Allocation)?;
    words.push(0);
    for byte in digits.bytes() {
        let mut carry = u64::from(byte - b'0');
        for word in &mut words {
            let product = u64::from(*word) * 10 + carry;
            *word = product as u32;
            carry = product >> 32;
        }
        if carry != 0 {
            words.push(carry as u32);
        }
    }
    let sign = if negative { num_bigint::Sign::Minus } else { num_bigint::Sign::Plus };
    #[cfg(target_pointer_width = "64")]
    {
        // `BigInt::new` packs pairs of public base-2^32 words into an internal
        // `Vec<u64>` on 64-bit targets. Its `BigDigits::extend` allocates only
        // when that packed vector needs more than the inline digit.
        let packed_digits = words.len().div_ceil(2);
        if packed_digits > 1 {
            allocations.reserve_array::<u64>(packed_digits).map_err(ArtifactError::Allocation)?;
        }
    }
    let value = num_bigint::BigInt::new(sign, words);
    StaticInt::new_with_allocation_budget(value, allocations).map_err(ArtifactError::Allocation)
}
fn key_in_with_allocation_budget(value: &FieldKeyV1, limits: &ValidationLimits, allocations: &mut AllocationBudget) -> Result<FieldKey, ArtifactError> {
    Ok(match value {
        FieldKeyV1::Symbol { value } => FieldKey::Symbol(allocations.arc_str(value).map_err(ArtifactError::Allocation)?),
        FieldKeyV1::String { value } => FieldKey::String(allocations.arc_str(value).map_err(ArtifactError::Allocation)?),
        FieldKeyV1::Int { decimal } => FieldKey::int(integer_in_with_allocation_budget(decimal, limits, allocations)?),
        FieldKeyV1::Bool { value } => FieldKey::bool(*value),
    })
}
fn presence_in(value: &PresenceV1) -> StructuralMemberPresence {
    match value {
        PresenceV1::Required => StructuralMemberPresence::Required,
        PresenceV1::Optional => StructuralMemberPresence::Optional,
    }
}
fn variance_in(value: &VarianceV1) -> Variance {
    match value {
        VarianceV1::Covariant => Variance::Covariant,
        VarianceV1::Contravariant => Variance::Contravariant,
        VarianceV1::Invariant => Variance::Invariant,
    }
}
fn mode_in(value: &ParameterModeV1) -> GenericParameterMode {
    match value {
        ParameterModeV1::Single => GenericParameterMode::Single,
        ParameterModeV1::PositionalPack => GenericParameterMode::PositionalPack,
        ParameterModeV1::LabeledPack => GenericParameterMode::LabeledPack,
        ParameterModeV1::CompletePack => GenericParameterMode::CompletePack,
    }
}
fn rest_in(value: &RestModeV1) -> RestMode {
    match value {
        RestModeV1::None => RestMode::None,
        RestModeV1::Positional => RestMode::Positional,
        RestModeV1::Labeled => RestMode::Labeled,
        RestModeV1::Complete => RestMode::Complete,
    }
}
fn selector_in_with_allocation_budget(value: &TypeMemberSelectorV1, allocations: &mut AllocationBudget) -> Result<TypeMemberSelector, ArtifactError> {
    Ok(match value {
        TypeMemberSelectorV1::Named { name } => TypeMemberSelector::Named(allocations.boxed_str(name).map_err(ArtifactError::Allocation)?),
        TypeMemberSelectorV1::Subscript => TypeMemberSelector::Subscript,
    })
}
fn operation_in_with_allocation_budget(value: &OperationV1, allocations: &mut AllocationBudget) -> Result<FamilyOperationShape, ArtifactError> {
    let kind = match value.kind {
        SelectorKindV1::Getter => SelectorKind::Getter,
        SelectorKindV1::Setter => SelectorKind::Setter,
        SelectorKindV1::Method => SelectorKind::Method,
        SelectorKindV1::SubscriptGet => SelectorKind::SubscriptGet,
        SelectorKindV1::SubscriptSet => SelectorKind::SubscriptSet,
    };
    let mut slots = allocations.vector(value.slots.len()).map_err(ArtifactError::Allocation)?;
    let mut validation_slots = allocations.vector(value.slots.len()).map_err(ArtifactError::Allocation)?;
    for slot in &value.slots {
        match slot {
            SelectorSlotV1::Positional => {
                slots.push(SelectorSlot::Positional);
                validation_slots.push(SelectorSlot::Positional);
            }
            SelectorSlotV1::Label { name } => {
                slots.push(SelectorSlot::Label(allocations.string(name).map_err(ArtifactError::Allocation)?));
                validation_slots.push(SelectorSlot::Label(allocations.string(name).map_err(ArtifactError::Allocation)?));
            }
        }
    }
    let slots = allocations.boxed_slice(slots).map_err(ArtifactError::Allocation)?;
    let validation_slots = allocations.boxed_slice(validation_slots).map_err(ArtifactError::Allocation)?;
    let base = match kind {
        SelectorKind::SubscriptGet | SelectorKind::SubscriptSet => SelectorBase::Subscript,
        _ => SelectorBase::Named(allocations.string("artifact").map_err(ArtifactError::Allocation)?),
    };
    Selector::new(base, kind, validation_slots).map_err(|_| invalid("invalid family operation selector"))?;
    Ok(FamilyOperationShape { kind, slots })
}
fn complete<T: Copy>(values: Vec<Option<T>>, allocations: &mut AllocationBudget) -> Result<Vec<T>, ArtifactError> {
    let mut result = allocations.vector(values.len()).map_err(ArtifactError::Allocation)?;
    for value in values {
        result.push(value.ok_or_else(|| invalid("incomplete type relocation"))?);
    }
    Ok(result)
}
impl TypeStore {
    /// Decodes and relocates into a fresh store. No semantic analysis is performed.
    pub fn decode_artifact(bytes: &[u8], limits: &ValidationLimits) -> Result<(Self, TypeRelocation), ArtifactError> {
        let graph = TypeGraphV1::decode(bytes, limits)?;
        Self::import_type_graph(&graph, limits)
    }

    /// Decodes and relocates into a fresh store with allocation budget tracking.
    pub fn decode_artifact_with_allocation_budget(
        bytes: &[u8],
        limits: &ValidationLimits,
        allocations: &mut AllocationBudget,
    ) -> Result<(Self, TypeRelocation), ArtifactError> {
        let graph = TypeGraphV1::decode(bytes, limits)?;
        Self::import_type_graph_with_allocation_budget(&graph, limits, allocations)
    }

    /// Relocates a validated `TypeGraphV1` into a fresh store.
    pub fn import_type_graph(graph: &TypeGraphV1, limits: &ValidationLimits) -> Result<(Self, TypeRelocation), ArtifactError> {
        let mut allocations = AllocationBudget::new(usize::MAX);
        Self::import_type_graph_with_allocation_budget(graph, limits, &mut allocations)
    }

    /// Relocates a validated `TypeGraphV1` into a fresh store with allocation budgeting.
    pub fn import_type_graph_with_allocation_budget(
        graph: &TypeGraphV1,
        limits: &ValidationLimits,
        allocations: &mut AllocationBudget,
    ) -> Result<(Self, TypeRelocation), ArtifactError> {
        Self::import_type_graph_with_owner_catalog_and_allocation_budget(graph, None, limits, allocations)
    }

    /// Relocate a type graph with exact owner authority supplied by the
    /// caller. This is not a multi-artifact store-composition operation.
    /// Existing Universe-only callers retain the strict legacy entry point.
    pub(crate) fn import_type_graph_with_owner_catalog_and_allocation_budget(
        graph: &TypeGraphV1,
        owner_catalog: Option<&crate::artifact::StableOwnerCatalog<'_>>,
        limits: &ValidationLimits,
        allocations: &mut AllocationBudget,
    ) -> Result<(Self, TypeRelocation), ArtifactError> {
        let mut import = Import::new(graph, owner_catalog, None, limits, allocations)?;
        for index in 0..graph.kinds.len() {
            import.kind(ArtifactKindId(index as u32))?;
        }
        for index in 0..graph.parameters.len() {
            import.parameter(ArtifactTypeParameterId(index as u32))?;
        }
        for index in 0..graph.types.len() {
            import.ty(ArtifactTypeId(index as u32))?;
        }
        for index in 0..graph.rows.len() {
            import.row(ArtifactRecordRowId(index as u32))?;
        }
        for index in 0..graph.families.len() {
            import.family(ArtifactFamilyTypeId(index as u32))?;
        }
        for index in 0..graph.variants.len() {
            import.variant(ArtifactVariantTypeId(index as u32))?;
        }
        for index in 0..graph.static_domains.len() {
            import.domain(ArtifactStaticDomainId(index as u32))?;
        }
        for index in 0..graph.static_constructors.len() {
            import.constructor(ArtifactStaticConstructorId(index as u32))?;
        }
        for index in 0..graph.scoped.len() {
            import.scoped(ArtifactScopedTypeId(index as u32))?;
        }
        for index in 0..graph.lambdas.len() {
            import.lambda(ArtifactTypeLambdaId(index as u32))?;
        }

        let Import {
            types: import_types,
            kinds: import_kinds,
            parameters: import_parameters,
            rows: import_rows,
            families: import_families,
            variants: import_variants,
            domains: import_domains,
            constructors: import_constructors,
            scoped: import_scoped,
            lambdas: import_lambdas,
            decoded_types,
            decoded_type_kinds,
            decoded_kinds,
            decoded_parameters,
            decoded_rows,
            decoded_families,
            decoded_variants,
            decoded_domains,
            decoded_constructors,
            decoded_scoped,
            decoded_lambdas,
            ..
        } = import;

        // Scoped types
        let mut scoped_types_vec = allocations.vector(graph.scoped.len()).map_err(ArtifactError::Allocation)?;
        for opt in decoded_scoped {
            scoped_types_vec.push(opt.ok_or_else(|| invalid("incomplete scoped type node"))?);
        }
        let scoped_types = allocations.boxed_slice(scoped_types_vec).map_err(ArtifactError::Allocation)?;

        let mut scoped_to_id_vec = allocations.vector(graph.scoped.len()).map_err(ArtifactError::Allocation)?;
        for i in 0..graph.scoped.len() {
            scoped_to_id_vec.push(ScopedTypeId(i as u32));
        }
        scoped_to_id_vec.sort_unstable_by(|&a, &b| scoped_types[a.index()].cmp(&scoped_types[b.index()]));
        if scoped_to_id_vec
            .windows(2)
            .any(|pair| scoped_types[pair[0].index()] == scoped_types[pair[1].index()])
        {
            return Err(invalid("duplicate canonical scoped type node"));
        }
        let scoped_to_id = allocations.boxed_slice(scoped_to_id_vec).map_err(ArtifactError::Allocation)?;

        // Lambdas
        let mut lambdas_vec = allocations.vector(graph.lambdas.len()).map_err(ArtifactError::Allocation)?;
        for opt in decoded_lambdas {
            lambdas_vec.push(opt.ok_or_else(|| invalid("incomplete type lambda node"))?);
        }
        let lambdas = allocations.boxed_slice(lambdas_vec).map_err(ArtifactError::Allocation)?;

        let mut lambda_to_id_vec = allocations.vector(graph.lambdas.len()).map_err(ArtifactError::Allocation)?;
        for i in 0..graph.lambdas.len() {
            lambda_to_id_vec.push(TypeLambdaId(i as u32));
        }
        lambda_to_id_vec.sort_unstable_by(|&a, &b| lambdas[a.index()].cmp(&lambdas[b.index()]));
        if lambda_to_id_vec.windows(2).any(|pair| lambdas[pair[0].index()] == lambdas[pair[1].index()]) {
            return Err(invalid("duplicate canonical type lambda node"));
        }
        let lambda_to_id = allocations.boxed_slice(lambda_to_id_vec).map_err(ArtifactError::Allocation)?;

        let mut prov_vec = allocations.vector(graph.lambdas.len()).map_err(ArtifactError::Allocation)?;
        for _ in 0..graph.lambdas.len() {
            prov_vec.push(TypeLambdaProvenance::default());
        }
        let provenance = allocations.boxed_slice(prov_vec).map_err(ArtifactError::Allocation)?;

        let lambda_prefix = FrozenTypeLambdaPrefix {
            scoped_types,
            scoped_to_id,
            lambdas,
            lambda_to_id,
            provenance,
        };
        let lambda_arena = allocations.arc(lambda_prefix).map_err(ArtifactError::Allocation)?;

        // Kinds
        let mut kinds_vec = allocations.vector(graph.kinds.len()).map_err(ArtifactError::Allocation)?;
        for opt in decoded_kinds {
            kinds_vec.push(opt.ok_or_else(|| invalid("incomplete kind node"))?);
        }
        let kinds = allocations.boxed_slice(kinds_vec).map_err(ArtifactError::Allocation)?;

        let mut kind_to_id_vec = allocations.vector(graph.kinds.len()).map_err(ArtifactError::Allocation)?;
        for i in 0..graph.kinds.len() {
            kind_to_id_vec.push(KindId(i as u32));
        }
        kind_to_id_vec.sort_unstable_by(|&a, &b| kinds[a.index()].cmp(&kinds[b.index()]));
        if kind_to_id_vec.windows(2).any(|pair| kinds[pair[0].index()] == kinds[pair[1].index()]) {
            return Err(invalid("duplicate canonical kind node"));
        }
        let kind_to_id = allocations.boxed_slice(kind_to_id_vec).map_err(ArtifactError::Allocation)?;

        if kinds.get(KindId::TYPE.index()) != Some(&KindData::Type)
            || kinds.get(KindId::RECORD_ROW.index()) != Some(&KindData::RecordRow)
            || kinds.get(KindId::TYPE_PACK.index()) != Some(&KindData::TypePack)
            || kinds.get(KindId::LABELED_TYPE_PACK.index()) != Some(&KindData::LabeledTypePack)
            || kinds.get(KindId::COMPLETE_TYPE_PACK.index()) != Some(&KindData::CompleteTypePack)
        {
            return Err(invalid("canonical type graph omits required primitive kinds"));
        }

        // Types and Type Kinds
        let mut types_vec = allocations.vector(graph.types.len()).map_err(ArtifactError::Allocation)?;
        for opt in decoded_types {
            types_vec.push(opt.ok_or_else(|| invalid("incomplete type node"))?);
        }
        let types = allocations.boxed_slice(types_vec).map_err(ArtifactError::Allocation)?;

        let mut type_kinds_vec = allocations.vector(graph.types.len()).map_err(ArtifactError::Allocation)?;
        for opt in decoded_type_kinds {
            type_kinds_vec.push(opt.ok_or_else(|| invalid("incomplete type kind node"))?);
        }
        let type_kinds = allocations.boxed_slice(type_kinds_vec).map_err(ArtifactError::Allocation)?;

        let mut type_to_id_vec = allocations.vector(graph.types.len()).map_err(ArtifactError::Allocation)?;
        for i in 0..graph.types.len() {
            type_to_id_vec.push(TypeId(i as u32));
        }
        type_to_id_vec.sort_unstable_by(|&a, &b| (&types[a.index()], type_kinds[a.index()]).cmp(&(&types[b.index()], type_kinds[b.index()])));
        if type_to_id_vec
            .windows(2)
            .any(|pair| (&types[pair[0].index()], type_kinds[pair[0].index()]) == (&types[pair[1].index()], type_kinds[pair[1].index()]))
        {
            return Err(invalid("duplicate canonical type node"));
        }
        let type_to_id = allocations.boxed_slice(type_to_id_vec).map_err(ArtifactError::Allocation)?;

        let never_id = types
            .iter()
            .position(|t| *t == TypeData::Never)
            .map(|i| TypeId(i as u32))
            .ok_or_else(|| invalid("canonical type graph omits Never primitive"))?;

        let unit_id = types
            .iter()
            .position(|t| *t == TypeData::Unit)
            .map(|i| TypeId(i as u32))
            .ok_or_else(|| invalid("canonical type graph omits Unit primitive"))?;

        // Static domains
        let mut domains_vec = allocations.vector(graph.static_domains.len()).map_err(ArtifactError::Allocation)?;
        for opt in decoded_domains {
            domains_vec.push(opt.ok_or_else(|| invalid("incomplete static domain node"))?);
        }
        let static_domains = allocations.boxed_slice(domains_vec).map_err(ArtifactError::Allocation)?;

        let mut static_domain_to_id_vec = allocations.vector(graph.static_domains.len()).map_err(ArtifactError::Allocation)?;
        for i in 0..graph.static_domains.len() {
            static_domain_to_id_vec.push(StaticDomainId(i as u32));
        }
        static_domain_to_id_vec.sort_unstable_by(|&a, &b| static_domains[a.index()].cmp(&static_domains[b.index()]));
        if static_domain_to_id_vec
            .windows(2)
            .any(|pair| static_domains[pair[0].index()] == static_domains[pair[1].index()])
        {
            return Err(invalid("duplicate canonical static domain node"));
        }
        let static_domain_to_id = allocations.boxed_slice(static_domain_to_id_vec).map_err(ArtifactError::Allocation)?;

        let const_int_domain = static_domains
            .iter()
            .position(|d| *d == StaticDomainData::ConstInt)
            .map(|i| StaticDomainId(i as u32))
            .ok_or_else(|| invalid("canonical type graph omits ConstInt domain"))?;

        // Rows
        let mut rows_vec = allocations.vector(graph.rows.len()).map_err(ArtifactError::Allocation)?;
        for opt in decoded_rows {
            rows_vec.push(opt.ok_or_else(|| invalid("incomplete record row node"))?);
        }
        let row_arena = allocations.boxed_slice(rows_vec).map_err(ArtifactError::Allocation)?;

        let mut row_interner_vec = allocations.vector(graph.rows.len()).map_err(ArtifactError::Allocation)?;
        for i in 0..graph.rows.len() {
            row_interner_vec.push(RecordRowId(i as u32));
        }
        row_interner_vec.sort_unstable_by(|&a, &b| row_arena[a.index()].cmp(&row_arena[b.index()]));
        if row_interner_vec.windows(2).any(|pair| row_arena[pair[0].index()] == row_arena[pair[1].index()]) {
            return Err(invalid("duplicate canonical record row node"));
        }
        let row_interner = allocations.boxed_slice(row_interner_vec).map_err(ArtifactError::Allocation)?;

        // Families
        let mut families_vec = allocations.vector(graph.families.len()).map_err(ArtifactError::Allocation)?;
        for opt in decoded_families {
            families_vec.push(opt.ok_or_else(|| invalid("incomplete family node"))?);
        }
        let family_arena = allocations.boxed_slice(families_vec).map_err(ArtifactError::Allocation)?;

        let mut family_interner_vec = allocations.vector(graph.families.len()).map_err(ArtifactError::Allocation)?;
        for i in 0..graph.families.len() {
            family_interner_vec.push(FamilyTypeId(i as u32));
        }
        family_interner_vec.sort_unstable_by(|&a, &b| family_arena[a.index()].cmp(&family_arena[b.index()]));
        if family_interner_vec
            .windows(2)
            .any(|pair| family_arena[pair[0].index()] == family_arena[pair[1].index()])
        {
            return Err(invalid("duplicate canonical family node"));
        }
        let family_interner = allocations.boxed_slice(family_interner_vec).map_err(ArtifactError::Allocation)?;

        // Variants
        let mut variants_vec = allocations.vector(graph.variants.len()).map_err(ArtifactError::Allocation)?;
        for opt in decoded_variants {
            variants_vec.push(opt.ok_or_else(|| invalid("incomplete variant node"))?);
        }
        let variant_identities = allocations.boxed_slice(variants_vec).map_err(ArtifactError::Allocation)?;

        let mut variant_identity_to_id_vec = allocations.vector(graph.variants.len()).map_err(ArtifactError::Allocation)?;
        for i in 0..graph.variants.len() {
            variant_identity_to_id_vec.push(VariantTypeId(i as u32));
        }
        variant_identity_to_id_vec.sort_unstable_by(|&a, &b| variant_identities[a.index()].cmp(&variant_identities[b.index()]));
        if variant_identity_to_id_vec
            .windows(2)
            .any(|pair| variant_identities[pair[0].index()] == variant_identities[pair[1].index()])
        {
            return Err(invalid("duplicate canonical variant node"));
        }
        let variant_identity_to_id = allocations.boxed_slice(variant_identity_to_id_vec).map_err(ArtifactError::Allocation)?;

        // Constructors
        let mut ctors_vec = allocations.vector(graph.static_constructors.len()).map_err(ArtifactError::Allocation)?;
        for opt in decoded_constructors {
            ctors_vec.push(opt.ok_or_else(|| invalid("incomplete static constructor node"))?);
        }
        let static_constructors = allocations.boxed_slice(ctors_vec).map_err(ArtifactError::Allocation)?;

        let mut static_constructor_to_id_vec = allocations.vector(graph.static_constructors.len()).map_err(ArtifactError::Allocation)?;
        for i in 0..graph.static_constructors.len() {
            static_constructor_to_id_vec.push(AlgebraicStaticConstructorId(i as u32));
        }
        static_constructor_to_id_vec.sort_unstable_by(|&a, &b| static_constructors[a.index()].cmp(&static_constructors[b.index()]));
        if static_constructor_to_id_vec
            .windows(2)
            .any(|pair| static_constructors[pair[0].index()] == static_constructors[pair[1].index()])
        {
            return Err(invalid("duplicate canonical static constructor node"));
        }
        let static_constructor_to_id = allocations.boxed_slice(static_constructor_to_id_vec).map_err(ArtifactError::Allocation)?;

        // Parameters
        let mut parameters_vec = allocations.vector(graph.parameters.len()).map_err(ArtifactError::Allocation)?;
        for opt in decoded_parameters {
            parameters_vec.push(opt.ok_or_else(|| invalid("incomplete type parameter node"))?);
        }
        let type_parameters = allocations.boxed_slice(parameters_vec).map_err(ArtifactError::Allocation)?;

        // parameter_to_id from graph.current_parameters
        let mut parameter_to_id_vec = allocations.vector(graph.current_parameters.len()).map_err(ArtifactError::Allocation)?;
        for id in &graph.current_parameters {
            let parameter = import_parameters
                .get(id.0 as usize)
                .and_then(|p| *p)
                .ok_or_else(|| invalid("unresolved current parameter"))?;
            parameter_to_id_vec.push(parameter);
        }
        parameter_to_id_vec.sort_unstable_by(|&a, &b| {
            let pa = &type_parameters[a.index()];
            let pb = &type_parameters[b.index()];
            (&pa.owner, pa.index).cmp(&(&pb.owner, pb.index))
        });
        if parameter_to_id_vec.windows(2).any(|pair| {
            let pa = &type_parameters[pair[0].index()];
            let pb = &type_parameters[pair[1].index()];
            (&pa.owner, pa.index) == (&pb.owner, pb.index)
        }) {
            return Err(invalid("duplicate current parameter slot"));
        }
        for param in type_parameters.iter() {
            let key = (&param.owner, param.index);
            let found = parameter_to_id_vec
                .binary_search_by(|&id| {
                    let p = &type_parameters[id.index()];
                    (&p.owner, p.index).cmp(&key)
                })
                .is_ok();
            if !found {
                return Err(invalid("missing current binder selection"));
            }
        }
        let parameter_to_id = allocations.boxed_slice(parameter_to_id_vec).map_err(ArtifactError::Allocation)?;

        // Variances
        let mut variances_vec = allocations.vector(graph.variance_overrides.len()).map_err(ArtifactError::Allocation)?;
        for value in &graph.variance_overrides {
            let owner = declaration_in_with_allocation_budget(&value.owner, allocations, owner_catalog)?;
            variances_vec.push(((owner, value.index), variance_in(&value.variance)));
        }
        variances_vec.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        if variances_vec.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(invalid("duplicate parameter variance override"));
        }
        let parameter_variances = BudgetedSortedMap::from_sorted_entries(variances_vec).map_err(|e| invalid(format!("invalid parameter variances: {e:?}")))?;

        let prefix = FrozenTypeStorePrefix {
            id: TypeStore::fresh_id(),
            types,
            type_to_id,
            kinds,
            kind_to_id,
            type_kinds,
            type_parameters,
            parameter_to_id,
            parameter_variances,
            lambda_arena,
            row_arena,
            row_interner,
            family_arena,
            family_interner,
            variant_identities,
            variant_identity_to_id,
            static_domains,
            static_domain_to_id,
            static_constructors,
            static_constructor_to_id,
            const_int_domain,
            never_id,
            unit_id,
        };
        let prefix_arc = allocations.arc(prefix).map_err(ArtifactError::Allocation)?;
        let store = TypeStore::from_frozen(prefix_arc);

        validate_scoped_roots(&store, limits, allocations)?;

        let relocation = TypeRelocation {
            store_id: store.id(),
            types: complete(import_types, allocations)?,
            kinds: complete(import_kinds, allocations)?,
            parameters: complete(import_parameters, allocations)?,
            rows: complete(import_rows, allocations)?,
            families: complete(import_families, allocations)?,
            variants: complete(import_variants, allocations)?,
            domains: complete(import_domains, allocations)?,
            constructors: complete(import_constructors, allocations)?,
            scoped: complete(import_scoped, allocations)?,
            lambdas: complete(import_lambdas, allocations)?,
        };

        Ok((store, relocation))
    }

    /// Relocates one validated artifact type graph into an existing receiving
    /// TypeStore. Every returned coordinate belongs to the receiver's lineage;
    /// artifact-local ordinals remain confined to the relocation table.
    ///
    /// Callers compose artifacts dependency-first and must not publish the
    /// receiver until all dependent semantic families have validated.
    pub(crate) fn import_type_graph_into_store_with_owner_catalog_and_allocation_budget(
        receiver: &mut TypeStore,
        graph: &TypeGraphV1,
        owner_catalog: Option<&crate::artifact::StableOwnerCatalog<'_>>,
        limits: &ValidationLimits,
        allocations: &mut AllocationBudget,
    ) -> Result<TypeRelocation, ArtifactError> {
        // TypeStore clones share persistent backing and preserve the store ID.
        // Mutate a private candidate so validation failure cannot contaminate
        // the receiving generation.
        let mut candidate = receiver.clone();
        let store_id = candidate.id();
        let (
            import_types,
            import_kinds,
            import_parameters,
            import_rows,
            import_families,
            import_variants,
            import_domains,
            import_constructors,
            import_scoped,
            import_lambdas,
        ) = {
            let mut import = Import::new(graph, owner_catalog, Some(&mut candidate), limits, allocations)?;
            for index in 0..graph.kinds.len() {
                import.kind(ArtifactKindId(index as u32))?;
            }
            for index in 0..graph.parameters.len() {
                import.parameter(ArtifactTypeParameterId(index as u32))?;
            }
            for index in 0..graph.types.len() {
                import.ty(ArtifactTypeId(index as u32))?;
            }
            for index in 0..graph.rows.len() {
                import.row(ArtifactRecordRowId(index as u32))?;
            }
            for index in 0..graph.families.len() {
                import.family(ArtifactFamilyTypeId(index as u32))?;
            }
            for index in 0..graph.variants.len() {
                import.variant(ArtifactVariantTypeId(index as u32))?;
            }
            for index in 0..graph.static_domains.len() {
                import.domain(ArtifactStaticDomainId(index as u32))?;
            }
            for index in 0..graph.static_constructors.len() {
                import.constructor(ArtifactStaticConstructorId(index as u32))?;
            }
            for index in 0..graph.scoped.len() {
                import.scoped(ArtifactScopedTypeId(index as u32))?;
            }
            for index in 0..graph.lambdas.len() {
                import.lambda(ArtifactTypeLambdaId(index as u32))?;
            }
            let Import {
                types,
                kinds,
                parameters,
                rows,
                families,
                variants,
                domains,
                constructors,
                scoped,
                lambdas,
                ..
            } = import;
            (types, kinds, parameters, rows, families, variants, domains, constructors, scoped, lambdas)
        };

        // Preserve the current-binder selection invariant independently of
        // artifact ordinals. Two wire slots that relocate to the same native
        // parameter are still a duplicate current binding.
        let mut current_parameters = allocations.vector::<TypeParameterId>(graph.current_parameters.len()).map_err(ArtifactError::Allocation)?;
        for wire_id in &graph.current_parameters {
            current_parameters.push(
                import_parameters
                    .get(wire_id.0 as usize)
                    .and_then(|value| *value)
                    .ok_or_else(|| invalid("unresolved current parameter"))?,
            );
        }
        current_parameters.sort_unstable_by(|left, right| {
            let left = candidate.type_parameter(*left);
            let right = candidate.type_parameter(*right);
            (&left.owner, left.index).cmp(&(&right.owner, right.index))
        });
        if current_parameters.windows(2).any(|pair| {
            let left = candidate.type_parameter(pair[0]);
            let right = candidate.type_parameter(pair[1]);
            (&left.owner, left.index) == (&right.owner, right.index)
        }) {
            return Err(invalid("duplicate current parameter slot after relocation"));
        }
        for parameter in import_parameters.iter().filter_map(|value| *value) {
            let parameter = candidate.type_parameter(parameter);
            if current_parameters
                .binary_search_by(|parameter_id| {
                    let current = candidate.type_parameter(*parameter_id);
                    (&current.owner, current.index).cmp(&(&parameter.owner, parameter.index))
                })
                .is_err()
            {
                return Err(invalid("missing current binder selection after relocation"));
            }
        }

        let mut variance_overrides = allocations.vector(graph.variance_overrides.len()).map_err(ArtifactError::Allocation)?;
        for value in &graph.variance_overrides {
            let owner = declaration_in_with_allocation_budget(&value.owner, allocations, owner_catalog)?;
            variance_overrides.push(((owner, value.index), variance_in(&value.variance)));
        }
        variance_overrides.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        if variance_overrides.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(invalid("duplicate parameter variance override"));
        }
        for ((owner, index), variance) in variance_overrides {
            candidate.set_parameter_variance(owner, index, variance);
        }

        validate_scoped_roots(&candidate, limits, allocations)?;

        let relocation = TypeRelocation {
            store_id,
            types: complete(import_types, allocations)?,
            kinds: complete(import_kinds, allocations)?,
            parameters: complete(import_parameters, allocations)?,
            rows: complete(import_rows, allocations)?,
            families: complete(import_families, allocations)?,
            variants: complete(import_variants, allocations)?,
            domains: complete(import_domains, allocations)?,
            constructors: complete(import_constructors, allocations)?,
            scoped: complete(import_scoped, allocations)?,
            lambdas: complete(import_lambdas, allocations)?,
        };
        *receiver = candidate;
        Ok(relocation)
    }
}

/// Checks de Bruijn addressability in complete global lambda roots, retaining
/// outer binder contexts while traversing nested lambda bodies. Arena fragments
/// are not incorrectly treated as independently closed lambda roots.
fn validate_scoped_roots(store: &TypeStore, limits: &ValidationLimits, allocations: &mut AllocationBudget) -> Result<(), ArtifactError> {
    struct Validator<'a, 'b> {
        arena: &'a TypeLambdaArena,
        limits: &'a ValidationLimits,
        allocations: &'b mut AllocationBudget,
        seen: Vec<(ScopedTypeId, Box<[usize]>)>,
        calls: usize,
    }
    impl Validator<'_, '_> {
        fn bound(&self, depth: u32, index: u32, context: &[usize]) -> Result<(), ArtifactError> {
            if context.get(depth as usize).is_none_or(|width| index as usize >= *width) {
                return Err(invalid("out-of-scope lambda binder reference"));
            }
            Ok(())
        }
        fn lambda(&mut self, id: TypeLambdaId, context: &[usize], depth: usize) -> Result<(), ArtifactError> {
            let lambda = self.arena.get_lambda(id);
            let nested_len = context
                .len()
                .checked_add(1)
                .ok_or(ArtifactError::Allocation(phalcom_type_meta::allocation::AllocationError::Overflow))?;
            let mut nested = self.allocations.vector(nested_len).map_err(ArtifactError::Allocation)?;
            nested.push(lambda.parameter_kinds.len());
            nested.extend_from_slice(context);
            self.walk(lambda.body, &nested, depth + 1)
        }
        fn anonymous(&mut self, value: ScopedAnonymousRecordRemainder, context: &[usize], depth: usize) -> Result<(), ArtifactError> {
            match value {
                AnonymousRecordRemainder::Unconstrained { key_type } => self.walk(key_type, context, depth),
                AnonymousRecordRemainder::Typed { key_type, value_type } => {
                    self.walk(key_type, context, depth)?;
                    self.walk(value_type, context, depth)
                }
            }
        }
        fn walk(&mut self, id: ScopedTypeId, context: &[usize], depth: usize) -> Result<(), ArtifactError> {
            if depth >= self.limits.max_recursion_depth || context.len() > self.limits.max_recursion_depth {
                return Err(invalid("lambda validation depth exceeds limit"));
            }
            self.calls += 1;
            if self.calls > self.limits.max_semantic_entries_per_family {
                return Err(invalid("lambda validation work budget exceeded"));
            }
            let position = self
                .seen
                .binary_search_by(|(seen_id, seen_context)| seen_id.cmp(&id).then_with(|| seen_context.as_ref().cmp(context)));
            let Err(position) = position else {
                return Ok(());
            };
            let mut retained_context = self.allocations.vector(context.len()).map_err(ArtifactError::Allocation)?;
            retained_context.extend_from_slice(context);
            let retained_context = self.allocations.boxed_slice(retained_context).map_err(ArtifactError::Allocation)?;
            self.allocations.grow_vector_exact(&mut self.seen, 1).map_err(ArtifactError::Allocation)?;
            self.seen.insert(position, (id, retained_context));
            let arena = self.arena;
            let next = depth + 1;
            match arena.get_scoped(id) {
                ScopedTypeData::Bound { depth, index } => self.bound(*depth, *index, context)?,
                ScopedTypeData::Free(_) => {}
                ScopedTypeData::Applied { origin, arguments } => {
                    self.walk(*origin, context, next)?;
                    for value in arguments {
                        self.walk(*value, context, next)?;
                    }
                }
                ScopedTypeData::Union(members)
                | ScopedTypeData::TypePack(members)
                | ScopedTypeData::TypePackConcat(members)
                | ScopedTypeData::LabeledTypePackConcat(members) => {
                    for value in members {
                        self.walk(*value, context, next)?;
                    }
                }
                ScopedTypeData::Tuple(elements) => {
                    for value in elements {
                        self.walk(value.ty, context, next)?;
                    }
                }
                ScopedTypeData::Record(fields) => {
                    for value in fields {
                        self.walk(value.ty, context, next)?;
                    }
                }
                ScopedTypeData::OpenRecord(row) => {
                    for value in &row.fields {
                        self.walk(value.ty, context, next)?;
                    }
                    match row.tail {
                        ScopedRecordTail::Bound { depth, index } => self.bound(depth, index, context)?,
                        ScopedRecordTail::FreeParameter(_) => {}
                        ScopedRecordTail::Anonymous(value) => self.anonymous(value, context, next)?,
                    }
                }
                ScopedTypeData::RecordRow(row) => {
                    for value in &row.fields {
                        self.walk(value.ty, context, next)?;
                    }
                    match row.tail {
                        ScopedRecordRowTail::Bound { depth, index } => self.bound(depth, index, context)?,
                        ScopedRecordRowTail::Anonymous(value) => self.anonymous(value, context, next)?,
                        ScopedRecordRowTail::Closed | ScopedRecordRowTail::FreeParameter(_) | ScopedRecordRowTail::Projection(_) => {}
                    }
                }
                ScopedTypeData::Callable(value) => {
                    for parameter in &value.parameters {
                        self.walk(parameter.ty, context, next)?;
                    }
                    self.walk(value.return_type, context, next)?;
                }
                ScopedTypeData::Pack(value) => {
                    for element in &value.elements {
                        self.walk(element.ty, context, next)?;
                    }
                    for tail in [value.positional_remainder, value.labeled_remainder] {
                        if let ScopedPackRemainder::Typed(ty) = tail {
                            self.walk(ty, context, next)?;
                        }
                    }
                }
                ScopedTypeData::LabeledTypePack(members) => {
                    for value in members {
                        self.walk(value.element, context, next)?;
                    }
                }
                ScopedTypeData::CompleteTypePack { positionals, labeled }
                | ScopedTypeData::TupleFromPacks { positionals, labeled }
                | ScopedTypeData::PackFromPacks { positionals, labeled } => {
                    self.walk(*positionals, context, next)?;
                    self.walk(*labeled, context, next)?;
                }
                ScopedTypeData::CallableFromPacks {
                    positionals,
                    labeled,
                    return_type,
                } => {
                    self.walk(*positionals, context, next)?;
                    self.walk(*labeled, context, next)?;
                    self.walk(*return_type, context, next)?;
                }
                ScopedTypeData::PackProjection { subject, .. } => self.walk(*subject, context, next)?,
                ScopedTypeData::Lambda(lambda) => self.lambda(*lambda, context, next)?,
            }
            Ok(())
        }
    }
    let mut validator = Validator {
        arena: &store.lambda_arena,
        limits,
        allocations,
        seen: Vec::new(),
        calls: 0,
    };
    for i in 0..store.type_count() {
        if let TypeData::Lambda(lambda) = store.get(TypeId(i as u32)) {
            validator.lambda(*lambda, &[], 0)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod artifact_owner_catalog_regressions {
    use super::*;
    use crate::artifact::{StableOwnerCatalog, empty_owner_inputs};
    use phalcom_modules::{ProjectIdentity, ProjectUniverse};
    use phalcom_type_meta::fingerprint::Fingerprint128;

    #[test]
    fn shared_receiver_reuses_native_type_ids_on_reimport() {
        let limits = ValidationLimits::default();
        let declaration = DeclarationId::new(ModuleId::universe_root(), "SharedReceiver".into());
        let mut source = TypeStore::new();
        let source_type = source.nominal_type(declaration.clone());
        assert_eq!(source.get(source_type), &TypeData::Nominal { declaration: declaration.clone() });
        let graph = source.project_artifact(&limits).unwrap();
        let artifact_type = graph
            .types
            .iter()
            .position(|entry| matches!(&entry.form, TypeV1::Nominal { declaration: owner } if owner.path.as_ref() == ["SharedReceiver"]))
            .map(|index| ArtifactTypeId(index as u32))
            .unwrap();

        let mut receiver = TypeStore::new();
        let mut allocations = AllocationBudget::new(8 * 1024 * 1024);
        let first = TypeStore::import_type_graph_into_store_with_owner_catalog_and_allocation_budget(
            &mut receiver,
            &graph,
            None,
            &limits,
            &mut allocations,
        )
        .unwrap();
        assert_eq!(first.store_id, receiver.id());
        let native = first.ty(artifact_type).unwrap();
        assert_eq!(receiver.get(native), &TypeData::Nominal { declaration: declaration.clone() });
        let counts = (
            receiver.type_count(),
            receiver.kind_count(),
            receiver.parameter_count(),
            receiver.row_count(),
            receiver.family_count(),
            receiver.variant_count(),
            receiver.static_domain_count(),
            receiver.static_constructor_count(),
        );

        let second = TypeStore::import_type_graph_into_store_with_owner_catalog_and_allocation_budget(
            &mut receiver,
            &graph,
            None,
            &limits,
            &mut allocations,
        )
        .unwrap();
        assert_eq!(second.store_id, receiver.id());
        assert_eq!(second.ty(artifact_type).unwrap(), native);
        assert_eq!(
            counts,
            (
                receiver.type_count(),
                receiver.kind_count(),
                receiver.parameter_count(),
                receiver.row_count(),
                receiver.family_count(),
                receiver.variant_count(),
                receiver.static_domain_count(),
                receiver.static_constructor_count(),
            )
        );
    }

    #[test]
    fn failed_shared_receiver_import_does_not_publish_partial_type_nodes() {
        let limits = ValidationLimits::default();
        let owner = DeclarationId::new(ModuleId::universe_root(), "RollbackOwner".into());
        let mut source = TypeStore::new();
        source.nominal_type(owner.clone());
        source.intern_type_parameter(TypeParameterData::new(
            TypeParameterOwner::Declaration(owner),
            0,
            "T",
            KindId::TYPE,
        ));
        let mut graph = source.project_artifact(&limits).unwrap();
        assert!(!graph.current_parameters.is_empty());
        graph.current_parameters.push(graph.current_parameters[0]);

        let mut receiver = TypeStore::new();
        let before = (
            receiver.type_count(),
            receiver.parameter_count(),
            receiver.row_count(),
            receiver.family_count(),
            receiver.variant_count(),
            receiver.static_domain_count(),
            receiver.static_constructor_count(),
        );
        let mut allocations = AllocationBudget::new(8 * 1024 * 1024);
        assert!(TypeStore::import_type_graph_into_store_with_owner_catalog_and_allocation_budget(
            &mut receiver,
            &graph,
            None,
            &limits,
            &mut allocations,
        )
        .is_err());
        assert_eq!(
            before,
            (
                receiver.type_count(),
                receiver.parameter_count(),
                receiver.row_count(),
                receiver.family_count(),
                receiver.variant_count(),
                receiver.static_domain_count(),
                receiver.static_constructor_count(),
            ),
            "failed shared relocation must not replace the receiving store",
        );
    }

    #[test]
    fn universe_owner_requires_explicit_module_authority_when_catalog_is_supplied() {
        let (mut wire, projects) = empty_owner_inputs();
        let stable = StableModuleRef {
            project: StableProjectRef::Builtin { namespace: "universe".into(), version: "0.1.0".into() },
            path: Box::new([]),
        };
        let catalog = StableOwnerCatalog::new(&wire, &projects).unwrap();
        let mut allocations = AllocationBudget::new(64 * 1024);
        assert_eq!(module_in_with_allocation_budget(&stable, &mut allocations, None).unwrap().project, ProjectIdentity::Universe);
        assert!(module_in_with_allocation_budget(&stable, &mut allocations, Some(&catalog)).is_err());
        wire.external_projects.push(stable.project.clone());
        let catalog = StableOwnerCatalog::new(&wire, &projects).unwrap();
        assert_eq!(module_in_with_allocation_budget(&stable, &mut allocations, Some(&catalog)).unwrap().project, ProjectIdentity::Universe);
    }

    #[test]
    fn catalog_relocates_a_real_persistent_project_but_rejects_an_unlisted_module_owner() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("library");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("project.toml"),
            "[project]\nname = \"library\"\nnamespace = \"library\"\nversion = \"1.0.0\"\n",
        ).unwrap();
        std::fs::write(root.join("src/package.ph"), "").unwrap();
        let mut projects = ProjectUniverse::new();
        let id = projects.load_root(root.join("project.toml")).unwrap();
        let project = projects.get_project(id).unwrap();
        let owner = StableProjectRef::SourceArtifact {
            logical_uri: project.source_identity.0.to_string_lossy().into_owned().into_boxed_str(),
            source_fingerprint: Fingerprint128(project.revision_fingerprint().0),
        };
        let stable = StableModuleRef { project: owner.clone(), path: Box::new([]) };
        let (mut wire, _) = empty_owner_inputs();
        let mut allocations = AllocationBudget::new(64 * 1024);

        let catalog = StableOwnerCatalog::new(&wire, &projects).unwrap();
        assert!(module_in_with_allocation_budget(&stable, &mut allocations, Some(&catalog)).is_err());
        assert!(module_in_with_allocation_budget(&stable, &mut allocations, None).is_err());

        wire.external_projects.push(owner);
        let catalog = StableOwnerCatalog::new(&wire, &projects).unwrap();
        let module = module_in_with_allocation_budget(&stable, &mut allocations, Some(&catalog)).unwrap();
        assert_eq!(module.project, ProjectIdentity::Resolved(id));
        assert!(module.path.is_root());
    }
}
