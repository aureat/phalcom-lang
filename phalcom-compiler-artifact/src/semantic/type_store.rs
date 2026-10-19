//! Portable canonical type graph. All graph ID families are distinct types.
use crate::module_world::{Decode, Wire, invalid, record, tagged};
use crate::{ArtifactError, BinaryReader, BinaryWriter, ValidationLimits};
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::identity::*;

pub const TYPE_GRAPH_SCHEMA_V1: u32 = 1;
macro_rules! graph_id {
    ($($name:ident),* $(,)?) => { $(
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(pub u32);
        impl Wire for $name {
            fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> { self.0.write(out) }
            fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> { Ok(Self(u32::read(input)?)) }
        }
    )* };
}
graph_id!(
    ArtifactTypeId,
    ArtifactKindId,
    ArtifactTypeParameterId,
    ArtifactScopedTypeId,
    ArtifactTypeLambdaId,
    ArtifactRecordRowId,
    ArtifactFamilyTypeId,
    ArtifactVariantTypeId,
    ArtifactStaticDomainId,
    ArtifactStaticConstructorId
);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CanonicalRootFamilyTag {
    DeclarationShell = 1,
    HierarchyAndAlias = 2,
    GenericSignature = 3,
    ConstructorContract = 4,
    DataContract = 5,
    EnumContract = 6,
    VariantContract = 7,
    ParameterCurrentSlot = 8,
    CallableSignature = 9,
    SurfaceAndDispatch = 10,
    TraitAndConformance = 11,
    TypeMemberAndDomain = 12,
    StaticDefAndBinding = 13,
    CallableAnalysis = 14,
    SelectedQueryRoot = 15,
    StaticValue = 16,
    CallableDefinition = 17,
}

impl Wire for CanonicalRootFamilyTag {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        out.write_u8(*self as u8);
        Ok(())
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        match input.reader.read_u8()? {
            1 => Ok(Self::DeclarationShell),
            2 => Ok(Self::HierarchyAndAlias),
            3 => Ok(Self::GenericSignature),
            4 => Ok(Self::ConstructorContract),
            5 => Ok(Self::DataContract),
            6 => Ok(Self::EnumContract),
            7 => Ok(Self::VariantContract),
            8 => Ok(Self::ParameterCurrentSlot),
            9 => Ok(Self::CallableSignature),
            10 => Ok(Self::SurfaceAndDispatch),
            11 => Ok(Self::TraitAndConformance),
            12 => Ok(Self::TypeMemberAndDomain),
            13 => Ok(Self::StaticDefAndBinding),
            14 => Ok(Self::CallableAnalysis),
            15 => Ok(Self::SelectedQueryRoot),
            16 => Ok(Self::StaticValue),
            17 => Ok(Self::CallableDefinition),
            _ => Err(invalid("invalid CanonicalRootFamilyTag")),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StableOwnerRef {
    Declaration(StableDeclarationRef),
    Callable(StableCallableRef),
    Impl(StableImplRef),
    Module(StableModuleRef),
    TypeMember(StableTypeMemberRef),
    Variant(StableVariantRef),
}

impl Wire for StableOwnerRef {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        match self {
            Self::Declaration(decl) => {
                out.write_u8(0);
                decl.write(out)
            }
            Self::Callable(callable) => {
                out.write_u8(1);
                callable.write(out)
            }
            Self::Impl(imp) => {
                out.write_u8(2);
                imp.write(out)
            }
            Self::Module(module) => {
                out.write_u8(3);
                module.write(out)
            }
            Self::TypeMember(tm) => {
                out.write_u8(4);
                tm.write(out)
            }
            Self::Variant(v) => {
                out.write_u8(5);
                v.write(out)
            }
        }
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        input.nested(|input| match input.reader.read_u8()? {
            0 => Ok(Self::Declaration(StableDeclarationRef::read(input)?)),
            1 => Ok(Self::Callable(StableCallableRef::read(input)?)),
            2 => Ok(Self::Impl(StableImplRef::read(input)?)),
            3 => Ok(Self::Module(StableModuleRef::read(input)?)),
            4 => Ok(Self::TypeMember(StableTypeMemberRef::read(input)?)),
            5 => Ok(Self::Variant(StableVariantRef::read(input)?)),
            _ => Err(invalid("invalid StableOwnerRef tag")),
        })
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalTypeRootKey {
    pub family_tag: CanonicalRootFamilyTag,
    pub owner: StableOwnerRef,
    pub product_subkey: u32,
}

impl Wire for CanonicalTypeRootKey {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        self.family_tag.write(out)?;
        self.owner.write(out)?;
        self.product_subkey.write(out)?;
        Ok(())
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        input.nested(|input| {
            Ok(Self {
                family_tag: CanonicalRootFamilyTag::read(input)?,
                owner: StableOwnerRef::read(input)?,
                product_subkey: u32::read(input)?,
            })
        })
    }
}

tagged!(VarianceV1 { 0 => Covariant, 1 => Contravariant, 2 => Invariant });
tagged!(ParameterModeV1 { 0 => Single, 1 => PositionalPack, 2 => LabeledPack, 3 => CompletePack });
tagged!(ParameterOwnerV1 { 0 => Declaration { owner: StableDeclarationRef }, 1 => Callable { owner: StableCallableRef }, 2 => Impl { owner: StableImplRef } });
record!(ParameterV1 limit max_parameters { owner: ParameterOwnerV1, index: u32, name: String, kind: ArtifactKindId, variance: VarianceV1, mode: ParameterModeV1 });
record!(VarianceOverrideV1 {
    owner: StableDeclarationRef,
    index: u32,
    variance: VarianceV1
});
tagged!(StaticArgumentV1 { 0 => Term { term: ArtifactTypeId }, 1 => RecordRow { row: ArtifactRecordRowId }, 2 => DynamicType });
tagged!(StaticDomainV1 { 0 => ConstInt, 1 => Nominal { declaration: StableDeclarationRef }, 2 => Associated { declaration: StableAssociatedStaticDomainDeclarationRef, arguments: Vec<StaticArgumentV1> } });
record!(StaticConstructorV1 { domain: ArtifactStaticDomainId, declaration: StableDeclarationRef, case_name: String, parameter_kinds: Vec<ArtifactKindId> });
tagged!(KindV1 limit max_kind_nodes { 0 => Type, 1 => RecordRow, 2 => TypePack, 3 => LabeledTypePack, 4 => CompleteTypePack, 5 => Arrow { parameters: Vec<ArtifactKindId>, result: ArtifactKindId }, 6 => StaticDomain { domain: ArtifactStaticDomainId } });
tagged!(PresenceV1 { 0 => Required, 1 => Optional });
tagged!(FieldKeyV1 { 0 => Symbol { value: String }, 1 => String { value: String }, 2 => Int { decimal: String }, 3 => Bool { value: bool } });
record!(FieldV1 {
    key: FieldKeyV1,
    ty: ArtifactTypeId,
    presence: PresenceV1
});
tagged!(RowTailV1 { 0 => Closed, 1 => Unconstrained { key_type: ArtifactTypeId }, 2 => Typed { key_type: ArtifactTypeId, value_type: ArtifactTypeId }, 3 => Parameter { parameter: ArtifactTypeParameterId }, 4 => Projection { ty: ArtifactTypeId } });
record!(RowV1 { fields: Vec<FieldV1>, tail: RowTailV1 });
tagged!(TupleRemainderV1 { 0 => None, 1 => Unconstrained, 2 => Typed { ty: ArtifactTypeId } });
record!(TupleElementV1 { label: Option<String>, ty: ArtifactTypeId });
record!(TupleV1 { elements: Vec<TupleElementV1>, positionals: TupleRemainderV1, labeled: TupleRemainderV1 });
record!(LabeledTypeV1 {
    label: String,
    ty: ArtifactTypeId
});
tagged!(RestModeV1 { 0 => None, 1 => Positional, 2 => Labeled, 3 => Complete });
record!(CallableParameterV1 { label: Option<String>, ty: ArtifactTypeId, rest: RestModeV1 });
record!(CallableV1 { parameters: Vec<CallableParameterV1>, result: ArtifactTypeId });
tagged!(SelectorKindV1 { 0 => Getter, 1 => Setter, 2 => Method, 3 => SubscriptGet, 4 => SubscriptSet });
tagged!(SelectorSlotV1 { 0 => Positional, 1 => Label { name: String } });
record!(OperationV1 { kind: SelectorKindV1, slots: Vec<SelectorSlotV1> });
tagged!(FamilyMemberKindV1 { 0 => Value, 1 => Callable });
record!(FamilyMemberV1 {
    operation: OperationV1,
    kind: FamilyMemberKindV1,
    ty: ArtifactTypeId
});
record!(FamilyV1 { members: Vec<FamilyMemberV1> });
tagged!(SelfRoleV1 { 0 => InstanceType, 1 => ReceiverValue });
record!(SelfTypeV1 {
    owner: StableDeclarationRef,
    side: StableDispatchSide,
    role: SelfRoleV1
});
record!(AssociatedProjectionV1 { subject: ArtifactTypeId, trait_owner: StableDeclarationRef, trait_arguments: Vec<ArtifactTypeId>, requirement_owner: StableDeclarationRef, requirement_index: u32 });
tagged!(TypeMemberSelectorV1 { 0 => Named { name: String }, 1 => Subscript });
tagged!(ProjectionSubjectV1 { 0 => Type { ty: ArtifactTypeId }, 1 => RecordRow { row: ArtifactRecordRowId } });
tagged!(TypeMemberArgumentV1 { 0 => Type { ty: ArtifactTypeId }, 1 => ConstInt { decimal: String }, 2 => RecordRow { row: ArtifactRecordRowId } });
record!(TypeProjectionV1 { subject: ProjectionSubjectV1, selector: TypeMemberSelectorV1, arguments: Vec<TypeMemberArgumentV1>, source: Option<StableTypeMemberRef> });
tagged!(TypeV1 {
    0 => Never, 1 => Unit, 2 => ClassObject { declaration: StableDeclarationRef }, 3 => Nominal { declaration: StableDeclarationRef },
    4 => StaticDomainForm { domain: ArtifactStaticDomainId }, 5 => Applied { origin: ArtifactTypeId, arguments: Vec<StaticArgumentV1> },
    6 => ExactCase { variant: ArtifactVariantTypeId, enum_type: ArtifactTypeId }, 7 => Union { members: Vec<ArtifactTypeId> },
    8 => Tuple { tuple: TupleV1 }, 9 => Record { row: ArtifactRecordRowId }, 10 => TypePack { members: Vec<ArtifactTypeId> },
    11 => LabeledTypePack { members: Vec<LabeledTypeV1> }, 12 => CompleteTypePack { positionals: ArtifactTypeId, labeled: ArtifactTypeId },
    13 => TypePackConcat { members: Vec<ArtifactTypeId> }, 14 => LabeledTypePackConcat { members: Vec<ArtifactTypeId> },
    15 => TupleFromPacks { positionals: ArtifactTypeId, labeled: ArtifactTypeId }, 16 => Pack { pack: TupleV1 },
    17 => PackFromPacks { positionals: ArtifactTypeId, labeled: ArtifactTypeId }, 18 => CallableFromPacks { positionals: ArtifactTypeId, labeled: ArtifactTypeId, result: ArtifactTypeId },
    19 => Callable { callable: CallableV1 }, 20 => Family { family: ArtifactFamilyTypeId }, 21 => Parameter { parameter: ArtifactTypeParameterId },
    22 => Lambda { lambda: ArtifactTypeLambdaId }, 23 => SelfType { self_type: SelfTypeV1 }, 24 => AssociatedProjection { projection: AssociatedProjectionV1 },
    25 => TypeMemberProjection { projection: TypeProjectionV1 }, 26 => ConstInt { decimal: String },
    27 => StaticData { domain: ArtifactStaticDomainId, constructor: ArtifactStaticConstructorId, arguments: Vec<StaticArgumentV1> },
});
record!(TypeEntryV1 limit max_type_nodes { kind: ArtifactKindId, form: TypeV1 });
record!(ScopedElementV1 { label: Option<String>, ty: ArtifactScopedTypeId });
record!(ScopedFieldV1 {
    key: FieldKeyV1,
    ty: ArtifactScopedTypeId,
    presence: PresenceV1
});
tagged!(ScopedRemainderV1 { 0 => None, 1 => Unconstrained, 2 => Typed { ty: ArtifactScopedTypeId } });
record!(ScopedPackV1 { elements: Vec<ScopedElementV1>, positionals: ScopedRemainderV1, labeled: ScopedRemainderV1 });
tagged!(ScopedRowTailV1 { 0 => Closed, 1 => Bound { depth: u32, index: u32 }, 2 => Parameter { parameter: ArtifactTypeParameterId }, 3 => Unconstrained { key_type: ArtifactScopedTypeId }, 4 => Typed { key_type: ArtifactScopedTypeId, value_type: ArtifactScopedTypeId }, 5 => Projection { ty: ArtifactTypeId } });
record!(ScopedRowV1 { fields: Vec<ScopedFieldV1>, tail: ScopedRowTailV1 });
record!(ScopedParameterV1 { label: Option<String>, ty: ArtifactScopedTypeId, rest: bool });
record!(ScopedCallableV1 { parameters: Vec<ScopedParameterV1>, result: ArtifactScopedTypeId });
record!(ScopedLabeledTypeV1 {
    label: String,
    ty: ArtifactScopedTypeId
});
tagged!(PackLaneV1 { 0 => Positionals, 1 => Labeled, 2 => Complete });
tagged!(ScopedTypeV1 limit max_type_nodes {
    0 => Bound { depth: u32, index: u32 }, 1 => Free { ty: ArtifactTypeId },
    2 => Applied { origin: ArtifactScopedTypeId, arguments: Vec<ArtifactScopedTypeId> }, 3 => Union { members: Vec<ArtifactScopedTypeId> },
    4 => Tuple { elements: Vec<ScopedElementV1> }, 5 => Record { fields: Vec<ScopedFieldV1> },
    6 => OpenRecord { row: ScopedRowV1 }, 7 => RecordRow { row: ScopedRowV1 }, 8 => Callable { callable: ScopedCallableV1 },
    9 => Pack { pack: ScopedPackV1 }, 10 => TypePack { members: Vec<ArtifactScopedTypeId> },
    11 => LabeledTypePack { members: Vec<ScopedLabeledTypeV1> }, 12 => CompleteTypePack { positionals: ArtifactScopedTypeId, labeled: ArtifactScopedTypeId },
    13 => TypePackConcat { members: Vec<ArtifactScopedTypeId> }, 14 => LabeledTypePackConcat { members: Vec<ArtifactScopedTypeId> },
    15 => TupleFromPacks { positionals: ArtifactScopedTypeId, labeled: ArtifactScopedTypeId }, 16 => PackFromPacks { positionals: ArtifactScopedTypeId, labeled: ArtifactScopedTypeId },
    17 => CallableFromPacks { positionals: ArtifactScopedTypeId, labeled: ArtifactScopedTypeId, result: ArtifactScopedTypeId },
    18 => PackProjection { subject: ArtifactScopedTypeId, lane: PackLaneV1 }, 19 => Lambda { lambda: ArtifactTypeLambdaId },
});
record!(LambdaV1 limit max_type_nodes { parameter_kinds: Vec<ArtifactKindId>, parameter_modes: Vec<ParameterModeV1>, body: ArtifactScopedTypeId, result_kind: ArtifactKindId });
record!(TypeGraphV1 {
    types: Vec<TypeEntryV1>, kinds: Vec<KindV1>, parameters: Vec<ParameterV1>, current_parameters: Vec<ArtifactTypeParameterId>, rows: Vec<RowV1>, families: Vec<FamilyV1>,
    variants: Vec<StableVariantRef>, static_domains: Vec<StaticDomainV1>, static_constructors: Vec<StaticConstructorV1>,
    scoped: Vec<ScopedTypeV1>, lambdas: Vec<LambdaV1>, variance_overrides: Vec<VarianceOverrideV1>,
});
impl TypeGraphV1 {
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        let mut out = BinaryWriter::new();
        out.write_u32(TYPE_GRAPH_SCHEMA_V1);
        self.write(&mut out)?;
        Ok(out.into_vec())
    }
    pub fn decode(bytes: &[u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        if bytes.len() as u64 > limits.max_decoded_section_bytes {
            return Err(invalid("type graph byte budget exceeded"));
        }
        let allocation_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| invalid("type graph allocation limit exceeds address space"))?;
        let mut allocations = AllocationBudget::new(allocation_limit);
        let mut input = Decode {
            reader: BinaryReader::new(bytes),
            limits: *limits,
            allocations: &mut allocations,
            items_left: limits.max_semantic_entries_per_family,
            depth: 0,
        };
        if input.reader.read_u32()? != TYPE_GRAPH_SCHEMA_V1 {
            return Err(invalid("unsupported type graph schema"));
        }
        let graph = Self::read(&mut input)?;
        if input.reader.has_remaining() {
            return Err(invalid("trailing type graph bytes"));
        }
        graph.validate(limits)?;
        Ok(graph)
    }
    pub fn validate(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        if self.kinds.len() > limits.max_kind_nodes
            || self.scoped.len() > limits.max_type_nodes
            || self.lambdas.len() > limits.max_type_nodes
            || self.types.len() > limits.max_type_nodes
            || self.parameters.len() > limits.max_parameters
        {
            return Err(invalid("type graph node budget exceeded"));
        }
        Ok(())
    }
}
