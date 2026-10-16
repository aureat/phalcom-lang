//! Combined semantic world section payload (`PERF003.C2.P7.T7.B/T8`).

use crate::module_world::SpanV1;
use crate::module_world::{Decode, Wire, invalid, record, tagged};
use crate::semantic::aggregates::{DeclaredTypeFactV1, GenericSignatureV1, StructuralPrimaryContractsV1, TraitRefV1};
use crate::semantic::operational::CheckedStaticDefinitionsV1;
use crate::semantic::static_value::StaticValueGraphV1;
use crate::semantic::type_store::{
    ArtifactKindId, ArtifactStaticConstructorId, ArtifactStaticDomainId, ArtifactTypeId, ArtifactTypeParameterId, StaticArgumentV1,
    TypeGraphV1, TypeMemberSelectorV1,
};
use crate::{ArtifactError, BinaryReader, BinaryWriter, ValidationLimits};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::identity::{
    StableAssociatedStaticDomainDeclarationRef, StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide, StableImplRef,
    StableModuleRef, StableVariantRef,
};

pub const SEMANTIC_WORLD_SCHEMA_V2: u32 = 2;

tagged!(ConformanceTargetV1 {
    0 => Declaration { declaration: StableDeclarationRef },
    1 => ExactEnumCase { variant: StableVariantRef },
});

record!(ConformanceHeadRecordV1 {
    impl_id: StableImplRef,
    source_module: StableModuleRef,
    target: ConformanceTargetV1,
    target_type: ArtifactTypeId,
    trait_ref: TraitRefV1,
    generic_signature: Option<GenericSignatureV1>,
    authorized: bool,
    eligible: bool,
});

tagged!(RequirementSelectionV1 {
    0 => ConformanceCallable { callable: StableCallableRef },
    1 => InherentCallable { callable: StableCallableRef },
    2 => TraitDefault { callable: StableCallableRef },
    3 => DataComponent { owner: StableDeclarationRef, index: u32, specialized_type: ArtifactTypeId },
});

record!(RequirementEntryV1 {
    selector: Box<str>,
    side: u8,
    selection: RequirementSelectionV1,
});

tagged!(AssociatedBindingValueV1 {
    0 => Type { ty: ArtifactTypeId },
    1 => Dynamic,
});

record!(AssociatedBindingEntryV1 {
    index: u32,
    value: AssociatedBindingValueV1,
});

record!(WitnessPlanV1 {
    impl_id: StableImplRef,
    source_module: StableModuleRef,
    trait_ref_template: TraitRefV1,
    target_template: ArtifactTypeId,
    generic_signature: Option<GenericSignatureV1>,
    requirements: Vec<RequirementEntryV1>,
    associated_types: Vec<AssociatedBindingEntryV1>,
    fingerprint: u64,
    ordered_selected_callables: Vec<StableCallableRef>,
    slot_layout_fingerprint: [u8; 32],
    completeness: u8,
});

record!(CallableParameterV1 {
    callable: StableCallableRef,
    index: u32,
    local_name: Box<str>,
    external_label: Option<Box<str>>,
    declared_type: DeclaredTypeFactV1,
    rest_mode: u8,
    contract: ParameterContractV1,
    source: Option<EnumRequirementSourceV1>,
});

tagged!(RestValueContractV1 {
    0 => Unconstrained,
    1 => Element { fact: DeclaredTypeFactV1 },
    2 => Lane { ty: ArtifactTypeId },
    3 => Invalid { reason: crate::semantic::aggregates::UnknownReasonV1 },
});

tagged!(ParameterContractV1 {
    0 => Fixed { fact: DeclaredTypeFactV1 },
    1 => Rest { mode: u8, contract: RestValueContractV1 },
});

tagged!(ReturnFlowV1 {
    0 => Value,
    1 => Receiver,
    2 => Argument { index: u64 },
    3 => Never,
    4 => Unknown,
});

tagged!(EvidenceStatusV1 { 0 => Established, 1 => Assumed });

tagged!(EvidenceOriginV1 {
    0 => Syntax,
    1 => DeclarationSemantics,
    2 => ConstructorSemantics,
    3 => CallableSignature,
    4 => NativeSignature,
    5 => DeveloperAnnotation,
    6 => GenericInference,
    7 => Flow,
    8 => FieldLifecycle,
    9 => ContextualDerivation,
    10 => PatternDecomposition,
});

record!(EvidenceSetV1 {
    ranges: Vec<SpanV1>,
    descriptions: Vec<Box<str>>,
});

tagged!(TypeKnowledgeV1 {
    0 => Known { ty: ArtifactTypeId, status: EvidenceStatusV1, origin: EvidenceOriginV1, provenance: EvidenceSetV1 },
    1 => Unknown { reason: crate::semantic::aggregates::UnknownReasonV1 },
    2 => Dynamic { reason: crate::semantic::aggregates::DynamicReasonV1 },
});

tagged!(ReturnContractValidationV1 {
    0 => NotApplicable,
    1 => Unchecked,
    2 => Satisfied { status: EvidenceStatusV1 },
    3 => Refuted,
    4 => Blocked,
    5 => DynamicBoundary,
});

tagged!(ImplementationKindV1 { 0 => Phalcom, 1 => Native, 2 => Foreign, 3 => Intrinsic });

// Native static descriptors carry borrowed data in phalcom-native-meta. The
// portable representation stores their complete symbolic content and key;
// semantic import must match both against the compatible native catalog.
tagged!(TypeExprSpecV1 {
    0 => Unknown,
    1 => Never,
    2 => SelfType,
    3 => Universe { owner: Box<str> },
    4 => Parameter { name: Box<str> },
    5 => Applied { origin: Box<TypeExprSpecV1>, arguments: Vec<TypeExprSpecV1> },
    6 => Union { members: Vec<TypeExprSpecV1> },
    7 => Tuple { tuple: ParameterTupleSpecV1 },
});

record!(LabeledParameterSpecV1 { label: Box<str>, ty: Box<TypeExprSpecV1> });

record!(RestParameterSpecV1 { ty: Option<Box<TypeExprSpecV1>> });

record!(ParameterTupleSpecV1 {
    positional: Vec<TypeExprSpecV1>,
    labeled: Vec<LabeledParameterSpecV1>,
    rest: Option<RestParameterSpecV1>,
});

tagged!(NativeEffectV1 { 0 => Mutation, 1 => Io, 2 => Scheduling, 3 => Reflection, 4 => Nondeterminism, 5 => Blocking });

tagged!(EffectSpecV1 {
    0 => Unknown,
    1 => Pure,
    2 => Known { effects: Vec<NativeEffectV1> },
});

tagged!(RaisesSpecV1 {
    0 => Unknown,
    1 => Known { types: Vec<TypeExprSpecV1> },
});

record!(NativeSurfaceIdV1 {
    owner: Box<str>,
    side: u8,
    selector: Box<str>,
});

record!(NativeLifecycleSpecV1 {
    since: Option<Box<str>>,
    deprecated_since: Option<Box<str>>,
    replacement: Option<Box<str>>,
});

record!(CallableSignatureRecordV1 {
    callable: StableCallableRef,
    owner: StableDeclarationRef,
    side: u8,
    selector: Box<str>,
    generics: Option<GenericSignatureV1>,
    parameters: Vec<CallableParameterV1>,
    declared_return: DeclaredTypeFactV1,
    return_validation: ReturnContractValidationV1,
    inferred_return: Option<TypeKnowledgeV1>,
    source: Option<EnumRequirementSourceV1>,
    implementation: ImplementationKindV1,
    native_id: Option<NativeSurfaceIdV1>,
    effects: EffectSpecV1,
    raises: RaisesSpecV1,
    flow: ReturnFlowV1,
    lifecycle: NativeLifecycleSpecV1,
});

tagged!(CallableAnalysisStatusV1 {
    0 => Complete,
    1 => Partial,
    2 => Blocked,
    3 => Cancelled,
    4 => BudgetExceeded,
});

record!(ConstructorSuperInitializerV1 {
    call_range: SpanV1,
    initializer: StableCallableRef,
});

tagged!(CallableAnalysisSourceOwnerV1 {
    0 => Module { module: StableModuleRef },
    1 => Callable { callable: StableCallableRef },
});

record!(CallableBindingSourceAttachmentV1 {
    binding_ordinal: u32,
    range: SpanV1,
});

tagged!(CallableBindingOriginV1 {
    0 => BodyLocal,
    1 => ModuleValue { module: StableModuleRef, name: String, import_binding: Option<u32> },
});

record!(CallableBindingOriginEntryV1 {
    binding_ordinal: u32,
    origin: CallableBindingOriginV1,
});

record!(CallableStaticArgumentBindingV1 {
    parameter: ArtifactTypeParameterId,
    argument: StaticArgumentV1,
});

record!(CallableCallSpecializationV1 {
    expression_ordinal: u32,
    callable: StableCallableRef,
    bindings: Vec<CallableStaticArgumentBindingV1>,
});

tagged!(CallableFlowNodeKindV1 {
    0 => Entry,
    1 => Exit,
    2 => Statement { index: u32 },
    3 => BranchCondition,
    4 => LoopHeader,
    5 => Join,
    6 => Throw,
    7 => Unreachable,
});

tagged!(CallableFlowEdgeKindV1 {
    0 => Normal,
    1 => TrueBranch,
    2 => FalseBranch,
    3 => BackEdge,
    4 => Break,
    5 => Continue,
    6 => Return,
    7 => Throw,
    8 => Unreachable,
});

record!(CallableFlowNodeV1 {
    kind: CallableFlowNodeKindV1,
    range: SpanV1,
});

record!(CallableFlowEdgeV1 {
    source: u32,
    target: u32,
    kind: CallableFlowEdgeKindV1,
});

// Flow graph rows are sorted by owner-local ordinal. Edges reconstruct node
// predecessor/successor indexes in edge-ordinal order from canonical endpoints.
record!(CallableFlowGraphV1 {
    nodes: Vec<CallableFlowNodeV1>,
    edges: Vec<CallableFlowEdgeV1>,
    entry: Option<u32>,
    exits: Vec<u32>,
});

record!(CallableAnalysisRecordV1 {
    callable: StableCallableRef,
    body_source_owner: Option<CallableAnalysisSourceOwnerV1>,
    body_range: SpanV1,
    status: CallableAnalysisStatusV1,
    exits_unreachable: bool,
    flow_graph: CallableFlowGraphV1,
    constructor_super_initializer: Option<ConstructorSuperInitializerV1>,
    expression_count: u32,
    expression_denotations: Vec<CallableExpressionDenotationV1>,
    binding_denotations: Vec<CallableBindingDenotationV1>,
    call_specializations: Vec<CallableCallSpecializationV1>,
    formal_bindings: Vec<CallableBindingSourceAttachmentV1>,
    binding_origins: Vec<CallableBindingOriginEntryV1>,
});

record!(CallableExpressionDenotationV1 {
    expression_ordinal: u32,
    denotation: SemanticDenotationV1,
});

record!(CallableBindingDenotationV1 {
    binding_ordinal: u32,
    denotation: SemanticDenotationV1,
});

record!(EnumRequirementSourceV1 {
    module: StableModuleRef,
    range: SpanV1,
});

record!(EnumRequirementRecordV1 {
    requirement: StableCallableRef,
    signature: CallableSignatureRecordV1,
    source_order: u32,
    source: Option<EnumRequirementSourceV1>,
});

tagged!(CaseRequirementStatusV1 {
    0 => Satisfied { implementation: StableCallableRef },
    1 => Missing,
    2 => Incompatible { implementation: StableCallableRef },
    3 => Blocked,
});

record!(CaseRequirementResultV1 {
    variant: StableVariantRef,
    requirement: StableCallableRef,
    requirement_order: u32,
    variant_order: u32,
    status: CaseRequirementStatusV1,
});

record!(EnumRequirementOwnerV1 {
    owner: StableDeclarationRef,
    requirements: Vec<EnumRequirementRecordV1>,
    case_statuses: Vec<CaseRequirementResultV1>,
});

record!(TraitSurfaceRecordV1 {
    declaration: StableDeclarationRef,
    generic_signature: Option<GenericSignatureV1>,
    super_traits: Vec<TraitRefV1>,
    required_callables: Vec<StableCallableRef>,
    default_callables: Vec<StableCallableRef>,
    associated_types: Vec<Box<str>>,
});

record!(FieldSignatureV1 {
    owner: StableDeclarationRef,
    name: Box<str>,
    declared_type: DeclaredTypeFactV1,
    mutable: bool,
    side: u8,
});

record!(DeclarationSurfaceRecordV1 {
    declaration: StableDeclarationRef,
    instance_callables: Vec<StableCallableRef>,
    class_callables: Vec<StableCallableRef>,
    fields: Vec<FieldSignatureV1>,
});

record!(ConstructorParamFieldV1 {
    parameter_index: u32,
    field_name: Box<str>,
    field_side: u8,
});

record!(ConstructorInitializationRecordV1 {
    factory: StableCallableRef,
    initializer: StableCallableRef,
    origin: u8,
    field_input_fingerprint: u64,
    default_fields: Vec<Box<str>>,
    parameter_fields: Vec<ConstructorParamFieldV1>,
});

tagged!(TypeMemberDomainV1 {
    0 => Kind { kind: ArtifactKindId },
    1 => ValueType { ty: ArtifactTypeId },
});

record!(TypeMemberParameterV1 {
    name: Box<str>,
    domain: TypeMemberDomainV1,
});

tagged!(IntrinsicTypeOperationV1 {
    0 => TuplePositionals,
    1 => TupleLabeled,
    2 => TupleEntries,
    3 => TupleElement,
    4 => TupleAt,
    5 => TupleMap,
    6 => RecordFields,
    7 => RecordValue,
    8 => RecordMap,
    9 => TypePackMap,
    10 => LabeledTypePackMap,
    11 => RecordRowMap,
    12 => CompleteTypePackPositionals,
    13 => CompleteTypePackLabeled,
    14 => CompleteTypePackMap,
    15 => UnorderedShapeLookup,
    16 => UnorderedShapeView,
    17 => UnorderedShapeTail,
});

tagged!(TypeMemberImplementationV1 {
    0 => Phalcom,
    1 => Native,
    2 => Foreign,
    3 => Intrinsic,
});

record!(TypeMemberDeclarationRecordV1 {
    owner: StableDeclarationRef,
    selector: TypeMemberSelectorV1,
    parameters: Vec<TypeMemberParameterV1>,
    result_domain: TypeMemberDomainV1,
    intrinsic: Option<IntrinsicTypeOperationV1>,
    implementation: TypeMemberImplementationV1,
});

record!(AssociatedStaticDomainRecordV1 {
    declaration: StableAssociatedStaticDomainDeclarationRef,
    domain: ArtifactStaticDomainId,
    constructors: Vec<ArtifactStaticConstructorId>,
});

tagged!(AssociatedMemberV1 {
    0 => Variant { variant: StableVariantRef },
    1 => DataConstructor { owner: StableDeclarationRef },
    2 => DataComponent { owner: StableDeclarationRef, index: u32 },
    3 => StaticDomain { declaration: StableAssociatedStaticDomainDeclarationRef },
    4 => TypeMember { owner: StableDeclarationRef, selector: TypeMemberSelectorV1 },
});

tagged!(SemanticDenotationV1 {
    0 => TypeForm { ty: ArtifactTypeId },
    1 => Kind { kind: ArtifactKindId },
    2 => StaticDomain { domain: ArtifactStaticDomainId },
    3 => StaticTerm { term: ArtifactTypeId },
    4 => StaticApplication { body: StableDeclarationRef, ordinal: u32 },
    5 => AssociatedValue { value: AssociatedValueDenotationV1 },
    6 => BehavioralFamily { family: BehavioralFamilyDenotationV1 },
});

tagged!(AssociatedValueDenotationV1 {
    0 => Exact {
        owner_form: ArtifactTypeId,
        lookup_owner: StableDeclarationRef,
        member: AssociatedMemberV1,
        target: Option<InvocationTargetV1>
    },
    1 => Family {
        owner_form: ArtifactTypeId,
        lookup_owner: StableDeclarationRef,
        family: AssociatedFamilyIdentityV1,
        members: Vec<CapturedAssociatedMemberV1>
    },
});

record!(AssociatedFamilyIdentityV1 {
    owner: StableDeclarationRef,
    selector: TypeMemberSelectorV1,
});

record!(CapturedAssociatedMemberV1 {
    operation: FamilyOperationShapeV1,
    member: AssociatedMemberV1,
    target: Option<InvocationTargetV1>,
});

record!(CapturedBehavioralMemberV1 {
    operation: FamilyOperationShapeV1,
    target: InvocationTargetV1,
    conditional: Option<ConditionalDispatchSelectionV1>,
});

record!(BehavioralFamilyDenotationV1 {
    receiver_type: ArtifactTypeId,
    receiver_application: Option<ArtifactTypeId>,
    spec: BehavioralFamilySpecV1,
    members: Vec<CapturedBehavioralMemberV1>,
});

tagged!(BehavioralFamilySpecV1 {
    0 => Exact { selector: ExactSelectorV1 },
    1 => Pattern { pattern: SelectorPatternSpecV1 }
});

record!(ExactSelectorV1 {
    base: TypeMemberSelectorV1,
    kind: SelectorKindV1,
    slots: Vec<SelectorSlotV1>,
});

record!(SelectorPatternSpecV1 {
    base: TypeMemberSelectorV1,
    kind: SelectorKindPatternV1,
    prefix: Vec<SelectorSlotV1>,
    suffix: Vec<SelectorSlotV1>,
    has_gap: bool,
});

tagged!(SelectorKindV1 {
    0 => Getter,
    1 => Setter,
    2 => Method,
    3 => SubscriptGet,
    4 => SubscriptSet,
});

tagged!(SelectorKindPatternV1 {
    0 => AnyNamed,
    1 => NamedAccessors,
    2 => AnySubscript,
    3 => Exact { kind: SelectorKindV1 }
});

tagged!(SelectorSlotV1 {
    0 => Positional,
    1 => Label { label: Box<str> }
});

record!(FamilyOperationShapeV1 {
    kind: SelectorKindV1,
    slots: Vec<SelectorSlotV1>,
});

tagged!(InvocationTargetV1 {
    0 => Behavioral { callable: StableCallableRef },
    1 => VariantConstructor { variant: StableVariantRef },
    2 => DataConstructor { owner: StableDeclarationRef }
});

record!(ConditionalDispatchSelectionV1 {
    impl_id: StableImplRef,
    callable: StableCallableRef,
    declaring_owner: StableDeclarationRef,
    side: u8,
});

tagged!(AssociatedFamilyKindV1 {
    0 => Variant,
    1 => DataConstructor,
    2 => StaticDomain,
    3 => TypeMember,
    4 => IntrinsicTypeMember,
});

record!(AssociatedFamilyRecordV1 {
    selector: TypeMemberSelectorV1,
    kind: AssociatedFamilyKindV1,
    members: Vec<AssociatedMemberV1>,
});

record!(AssociatedSurfaceRecordV1 {
    owner: StableDeclarationRef,
    families: Vec<AssociatedFamilyRecordV1>,
});

record!(SemanticWorldV2 {
    type_graph: TypeGraphV1,
    structural_contracts: StructuralPrimaryContractsV1,
    enum_requirements: Vec<EnumRequirementOwnerV1>,
    static_values: StaticValueGraphV1,
    checked_definitions: CheckedStaticDefinitionsV1,
    witness_plans: Vec<WitnessPlanV1>,
    callable_signatures: Vec<CallableSignatureRecordV1>,
    callable_analyses: Vec<CallableAnalysisRecordV1>,
    trait_surfaces: Vec<TraitSurfaceRecordV1>,
    dispatch_surfaces: Vec<DeclarationSurfaceRecordV1>,
    conformance_heads: Vec<ConformanceHeadRecordV1>,
    constructor_initializations: Vec<ConstructorInitializationRecordV1>,
    type_member_declarations: Vec<TypeMemberDeclarationRecordV1>,
    associated_static_domains: Vec<AssociatedStaticDomainRecordV1>,
    associated_surfaces: Vec<AssociatedSurfaceRecordV1>,
});

impl SemanticWorldV2 {
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        self.validate(&ValidationLimits::default())?;
        let mut out = BinaryWriter::new();
        out.write_u32(SEMANTIC_WORLD_SCHEMA_V2);
        self.write(&mut out)?;
        Ok(out.into_vec())
    }

    pub fn decode_with_allocation_budget(bytes: &[u8], limits: &ValidationLimits, transaction: &mut AllocationBudget) -> Result<Self, ArtifactError> {
        let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
        let mut section = transaction.child(section_limit);
        let world = {
            let mut input = Decode {
                reader: BinaryReader::new(bytes),
                limits: *limits,
                allocations: &mut section,
                items_left: limits.max_semantic_entries_per_family,
                depth: 0,
            };
            if input.reader.read_u32()? != SEMANTIC_WORLD_SCHEMA_V2 {
                return Err(invalid("unsupported semantic world schema"));
            }
            let world = Self::read(&mut input)?;
            if input.reader.has_remaining() {
                return Err(invalid("trailing semantic world bytes"));
            }
            world
        };
        world.validate_with_allocation_budget(limits, &mut section)?;
        transaction.commit_child(section).map_err(allocation_error)?;
        Ok(world)
    }

    pub fn decode_with_budget(bytes: &[u8], limits: &ValidationLimits, remaining_budget: &mut u64) -> Result<Self, ArtifactError> {
        let available = usize::try_from(*remaining_budget).map_err(|_| allocation_error(AllocationError::Overflow))?;
        let mut transaction = AllocationBudget::new(available);
        let world = Self::decode_with_allocation_budget(bytes, limits, &mut transaction)?;
        *remaining_budget = u64::try_from(transaction.remaining()).map_err(|_| allocation_error(AllocationError::Overflow))?;
        Ok(world)
    }

    pub fn decode(bytes: &[u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        let mut budget = limits.max_decoded_section_bytes;
        Self::decode_with_budget(bytes, limits, &mut budget)
    }

    pub fn validate(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        let limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
        let mut allocations = AllocationBudget::new(limit);
        self.validate_inner(limits, &mut allocations)
    }

    /// Validates semantic references against the caller's load transaction.
    /// Returns `UnaccountedBoundary` until every component validator charges
    /// its scratch allocations to that transaction.
    pub fn validate_with_allocation_budget(
        &self,
        limits: &ValidationLimits,
        transaction: &mut AllocationBudget,
    ) -> Result<(), ArtifactError> {
        // Several component validators still allocate scratch that is not
        // charged through AllocationBudget. Fail before entering them so a
        // semantic decoder cannot commit a partially accounted section.
        require_semantic_validation_allocation_accounting()?;
        let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
        let mut allocations = transaction.child(section_limit);
        self.validate_inner(limits, &mut allocations)?;
        transaction.commit_child(allocations).map_err(allocation_error)
    }

    fn validate_inner(&self, limits: &ValidationLimits, allocations: &mut AllocationBudget) -> Result<(), ArtifactError> {
        self.type_graph.validate(limits)?;
        self.structural_contracts.validate(limits)?;
        self.static_values.validate(limits)?;
        self.checked_definitions.validate(limits)?;
        if self.witness_plans.len() > limits.max_semantic_entries_per_family
            || self.callable_signatures.len() > limits.max_semantic_entries_per_family
            || self.callable_analyses.len() > limits.max_semantic_entries_per_family
            || self.trait_surfaces.len() > limits.max_semantic_entries_per_family
            || self.dispatch_surfaces.len() > limits.max_semantic_entries_per_family
            || self.conformance_heads.len() > limits.max_semantic_entries_per_family
            || self.type_member_declarations.len() > limits.max_semantic_entries_per_family
            || self.associated_static_domains.len() > limits.max_semantic_entries_per_family
            || self.associated_surfaces.len() > limits.max_semantic_entries_per_family
            || self.enum_requirements.len() > limits.max_semantic_entries_per_family
        {
            return Err(invalid("primary family count budget exceeded"));
        }
        if self.callable_signatures.windows(2).any(|pair| pair[0].callable >= pair[1].callable) {
            return Err(invalid("callable signature owners must be unique and sorted"));
        }
        for signature in &self.callable_signatures {
            validate_callable_signature(signature, limits)?;
        }
        if self.enum_requirements.windows(2).any(|pair| pair[0].owner >= pair[1].owner) {
            return Err(invalid("enum requirement owners must be unique and sorted"));
        }
        if self.enum_requirements.len() != self.structural_contracts.enum_contracts.len() {
            return Err(invalid("enum requirement owners do not cover every enum contract"));
        }
        for (owner, enum_contract) in self.enum_requirements.iter().zip(&self.structural_contracts.enum_contracts) {
            if owner.owner != enum_contract.owner {
                return Err(invalid("enum requirement owner ordering disagrees with enum contracts"));
            }

            if owner.requirements.len() > limits.max_semantic_entries_per_family
                || owner.case_statuses.len() > limits.max_semantic_entries_per_family
            {
                return Err(invalid("enum requirement row budget exceeded"));
            }

            for pair in owner.requirements.windows(2) {
                if pair[0].requirement >= pair[1].requirement {
                    return Err(invalid("enum requirement callable owners must be unique and sorted"));
                }
            }

            for requirement in &owner.requirements {
                let StableCallableOwnerRef::Declaration(requirement_owner) = &requirement.requirement.owner else {
                    return Err(invalid("enum requirement must reference a declaration-owned callable"));
                };
                if requirement_owner != &owner.owner
                    || requirement.requirement.side != StableDispatchSide::Instance
                    || requirement.requirement.selector.is_empty()
                    || requirement.requirement.selector.len() > limits.max_string_bytes
                    || requirement.source_order as usize >= owner.requirements.len()
                {
                    return Err(invalid("enum requirement callable identity or source order is invalid"));
                }
                if let Some(source) = &requirement.source {
                    if source.module != owner.owner.module || source.range.start > source.range.end {
                        return Err(invalid("enum requirement source span is invalid"));
                    }
                }
                validate_callable_signature(&requirement.signature, limits)?;
                if requirement.signature.callable != requirement.requirement
                    || requirement.signature.owner != owner.owner
                    || requirement.signature.side != 0
                    || requirement.signature.selector.as_ref() != requirement.requirement.selector.as_ref()
                {
                    return Err(invalid("enum requirement disagrees with its callable signature identity"));
                }
                if let Ok(index) = self.callable_signatures.binary_search_by(|signature| signature.callable.cmp(&requirement.requirement))
                    && self.callable_signatures[index] != requirement.signature
                {
                    return Err(invalid("enum requirement signature disagrees with its ordinary callable owner row"));
                }
            }

            let expected_case_statuses = owner
                .requirements
                .len()
                .checked_mul(enum_contract.variants.len())
                .ok_or_else(|| invalid("enum requirement case status count overflow"))?;
            if owner.case_statuses.len() != expected_case_statuses {
                return Err(invalid("enum requirement case statuses do not cover requirements and variants"));
            }
            if !enum_contract.variants.is_empty() {
                for (index, result) in owner.case_statuses.iter().enumerate() {
                    let requirement_order = index / enum_contract.variants.len();
                    let variant_order = index % enum_contract.variants.len();
                    if result.requirement_order as usize != requirement_order
                        || result.variant_order as usize != variant_order
                        || result.variant != enum_contract.variants[variant_order]
                        || result.variant.owner != owner.owner
                    {
                        return Err(invalid("enum requirement case status order disagrees with enum source order"));
                    }
                    let requirement_index = owner
                        .requirements
                        .binary_search_by(|requirement| requirement.requirement.cmp(&result.requirement))
                        .map_err(|_| invalid("enum requirement case status references an absent requirement"))?;
                    if owner.requirements[requirement_index].source_order as usize != requirement_order {
                        return Err(invalid("enum requirement case status has a mismatched requirement order"));
                    }
                    let implementation = match &result.status {
                        CaseRequirementStatusV1::Satisfied { implementation }
                        | CaseRequirementStatusV1::Incompatible { implementation } => Some(implementation),
                        CaseRequirementStatusV1::Missing | CaseRequirementStatusV1::Blocked => None,
                    };
                    if let Some(implementation) = implementation {
                        if implementation.owner != StableCallableOwnerRef::Variant(result.variant.clone())
                            || implementation.side != StableDispatchSide::Instance
                            || implementation.selector != result.requirement.selector
                        {
                            return Err(invalid("enum requirement implementation does not match its exact case and selector"));
                        }
                        let implementation_index = self
                            .callable_signatures
                            .binary_search_by(|signature| signature.callable.cmp(implementation))
                            .map_err(|_| invalid("enum requirement implementation has no callable signature authority"))?;
                        if self.callable_signatures[implementation_index].side != 0
                            || self.callable_signatures[implementation_index].selector != implementation.selector
                        {
                            return Err(invalid("enum requirement implementation disagrees with its callable signature"));
                        }
                    }
                }
            }
        }
        for analysis in &self.callable_analyses {
            if analysis.body_range.start > analysis.body_range.end {
                return Err(invalid("callable body source range is inverted"));
            }
            CallableAnalysisOrdinalContext::new(analysis).validate(limits, allocations)?;
            if analysis
                .constructor_super_initializer
                .as_ref()
                .is_some_and(|initializer| initializer.call_range.start > initializer.call_range.end)
            {
                return Err(invalid("constructor super-initializer range is inverted"));
            }
            for call in &analysis.call_specializations {
                if call.bindings.len() > limits.max_semantic_entries_per_family
                    || call.bindings.windows(2).any(|pair| pair[0].parameter >= pair[1].parameter)
                {
                    return Err(invalid("call specialization bindings are invalid or unsorted"));
                }
                for binding in &call.bindings {
                    let parameter = self
                        .type_graph
                        .parameters
                        .get(binding.parameter.0 as usize)
                        .ok_or_else(|| invalid("call specialization references an absent type parameter"))?;
                    let kind = self
                        .type_graph
                        .kinds
                        .get(parameter.kind.0 as usize)
                        .ok_or_else(|| invalid("call specialization parameter has an absent kind"))?;
                    let parameter_owner_matches_call = match (&parameter.owner, &call.callable.owner) {
                        (
                            crate::semantic::type_store::ParameterOwnerV1::Callable { owner: parameter_owner },
                            _,
                        ) => parameter_owner == &call.callable,
                        (
                            crate::semantic::type_store::ParameterOwnerV1::Declaration { owner: parameter_owner },
                            StableCallableOwnerRef::Declaration(call_owner),
                        ) => parameter_owner == call_owner,
                        (
                            crate::semantic::type_store::ParameterOwnerV1::Declaration { owner: parameter_owner },
                            StableCallableOwnerRef::Variant(call_owner),
                        ) => parameter_owner == &call_owner.owner,
                        (
                            crate::semantic::type_store::ParameterOwnerV1::Impl { owner: parameter_owner },
                            StableCallableOwnerRef::Conformance(call_owner),
                        ) => parameter_owner == call_owner,
                        _ => false,
                    };
                    if !parameter_owner_matches_call {
                        return Err(invalid("call specialization parameter is not owned by its selected callable"));
                    }
                    match &binding.argument {
                        StaticArgumentV1::Term { term } => {
                            if term.0 as usize >= self.type_graph.types.len() || matches!(kind, crate::semantic::type_store::KindV1::RecordRow) {
                                return Err(invalid("call specialization term disagrees with its parameter kind or type graph"));
                            }
                        }
                        StaticArgumentV1::RecordRow { row } => {
                            if row.0 as usize >= self.type_graph.rows.len() || !matches!(kind, crate::semantic::type_store::KindV1::RecordRow) {
                                return Err(invalid("call specialization row disagrees with its parameter kind or type graph"));
                            }
                        }
                        StaticArgumentV1::DynamicType => {
                            if !matches!(kind, crate::semantic::type_store::KindV1::Type) {
                                return Err(invalid("Dynamic call specialization argument requires a proper-type parameter"));
                            }
                        }
                    }
                }
            }
            for attachment in &analysis.formal_bindings {
                if !matches!(
                    analysis.binding_origins.get(attachment.binding_ordinal as usize).map(|entry| &entry.origin),
                    Some(CallableBindingOriginV1::BodyLocal)
                ) {
                    return Err(invalid("module-value binding cannot have a callable-local source attachment"));
                }
            }
            for entry in &analysis.binding_origins {
                if let CallableBindingOriginV1::ModuleValue { name, .. } = &entry.origin
                    && (name.is_empty() || name.len() > limits.max_string_bytes)
                {
                    return Err(invalid("module-value binding symbol name is invalid"));
                }
            }
            for entry in &analysis.expression_denotations {
                self.validate_callable_denotation(&entry.denotation, limits)?;
            }
            for entry in &analysis.binding_denotations {
                self.validate_callable_denotation(&entry.denotation, limits)?;
            }
        }
        if self.callable_analyses.windows(2).any(|pair| pair[0].callable >= pair[1].callable) {
            return Err(invalid("callable analysis owners must be unique and sorted"));
        }
        Ok(())
    }

    fn validate_callable_denotation(&self, denotation: &SemanticDenotationV1, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        match denotation {
            SemanticDenotationV1::TypeForm { ty } => self.validate_denotation_type(*ty),
            SemanticDenotationV1::Kind { kind } => self.validate_denotation_kind(*kind),
            SemanticDenotationV1::StaticDomain { domain } => {
                let domain_data = self
                    .type_graph
                    .static_domains
                    .get(domain.0 as usize)
                    .ok_or_else(|| invalid("static-domain denotation references an absent domain"))?;
                match domain_data {
                    crate::semantic::type_store::StaticDomainV1::ConstInt => Ok(()),
                    crate::semantic::type_store::StaticDomainV1::Nominal { declaration } => {
                        self.validate_declaration_reference(declaration)
                    }
                    crate::semantic::type_store::StaticDomainV1::Associated { declaration, .. } => {
                        let record = self
                            .associated_static_domains
                            .iter()
                            .find(|record| record.declaration == *declaration)
                            .ok_or_else(|| invalid("associated static-domain denotation has no declaration authority"))?;
                        if record.domain != *domain {
                            return Err(invalid("associated static-domain denotation disagrees with its declaration product"));
                        }
                        Ok(())
                    }
                }
            }
            SemanticDenotationV1::StaticTerm { term } => {
                let entry = self
                    .type_graph
                    .types
                    .get(term.0 as usize)
                    .ok_or_else(|| invalid("static-term denotation references an absent type"))?;
                let kind = self
                    .type_graph
                    .kinds
                    .get(entry.kind.0 as usize)
                    .ok_or_else(|| invalid("static-term denotation has an absent kind"))?;
                if matches!(kind, crate::semantic::type_store::KindV1::StaticDomain { .. }) {
                    Ok(())
                } else {
                    Err(invalid("static-term denotation does not have a static-domain kind"))
                }
            }
            SemanticDenotationV1::StaticApplication { .. } => Err(invalid(
                "static-application denotation has no retained checked-body application row",
            )),
            SemanticDenotationV1::AssociatedValue { value } => match value {
                AssociatedValueDenotationV1::Exact {
                    owner_form,
                    lookup_owner,
                    member,
                    target,
                } => {
                    self.validate_denotation_type(*owner_form)?;
                    self.validate_associated_member_owner(lookup_owner, member)?;
                    if let Some(target) = target {
                        self.validate_invocation_target(target)?;
                        if !self.invocation_target_matches_associated_member(target, member) {
                            return Err(invalid("associated exact-value target disagrees with its captured member"));
                        }
                    }
                    Ok(())
                }
                AssociatedValueDenotationV1::Family {
                    owner_form,
                    lookup_owner,
                    family,
                    members,
                } => {
                    self.validate_denotation_type(*owner_form)?;
                    if family.owner != *lookup_owner {
                        return Err(invalid("associated-family identity disagrees with its lookup owner"));
                    }
                    let surface = self.associated_surface(lookup_owner)?;
                    let declared_family = surface
                        .families
                        .iter()
                        .find(|candidate| candidate.selector == family.selector)
                        .ok_or_else(|| invalid("associated-family capture references an absent declaration family"))?;
                    if members.len() > limits.max_semantic_entries_per_family {
                        return Err(invalid("associated-family capture exceeds its member limit"));
                    }
                    for member in members {
                        validate_family_operation_shape(&member.operation, limits)?;
                        self.validate_associated_member_owner(lookup_owner, &member.member)?;
                        if !declared_family.members.contains(&member.member) {
                            return Err(invalid("associated-family capture contains a member absent from its declaration family"));
                        }
                        if let Some(target) = &member.target {
                            self.validate_invocation_target(target)?;
                            if !self.invocation_target_matches_associated_member(target, &member.member) {
                                return Err(invalid("associated-family member target disagrees with its captured member"));
                            }
                        }
                    }
                    Ok(())
                }
            },
            SemanticDenotationV1::BehavioralFamily { family } => {
                self.validate_denotation_type(family.receiver_type)?;
                if let Some(application) = family.receiver_application {
                    self.validate_denotation_type(application)?;
                }
                validate_behavioral_family_spec(&family.spec, limits)?;
                if family.members.len() > limits.max_semantic_entries_per_family {
                    return Err(invalid("behavioral-family capture exceeds its member limit"));
                }
                for member in &family.members {
                    validate_family_operation_shape(&member.operation, limits)?;
                    self.validate_invocation_target(&member.target)?;
                    if let Some(conditional) = &member.conditional {
                        self.validate_conditional_dispatch_selection(conditional)?;
                        if !matches!(&member.target, InvocationTargetV1::Behavioral { callable } if callable == &conditional.callable)
                        {
                            return Err(invalid("behavioral-family conditional selection disagrees with its captured target"));
                        }
                    }
                    if !behavioral_spec_matches_operation(&family.spec, &member.operation) {
                        return Err(invalid("behavioral-family capture contains an operation outside its selector specification"));
                    }
                }
                Ok(())
            }
        }
    }

    fn validate_denotation_type(&self, ty: ArtifactTypeId) -> Result<(), ArtifactError> {
        if (ty.0 as usize) < self.type_graph.types.len() {
            Ok(())
        } else {
            Err(invalid("semantic denotation references an absent type"))
        }
    }

    fn validate_denotation_kind(&self, kind: ArtifactKindId) -> Result<(), ArtifactError> {
        if (kind.0 as usize) < self.type_graph.kinds.len() {
            Ok(())
        } else {
            Err(invalid("kind denotation references an absent kind"))
        }
    }

    fn validate_declaration_reference(&self, owner: &StableDeclarationRef) -> Result<(), ArtifactError> {
        if self.structural_contracts.declaration_shells.iter().any(|row| &row.declaration == owner)
            || self.structural_contracts.data_contracts.iter().any(|row| &row.owner == owner)
            || self.structural_contracts.enum_contracts.iter().any(|row| &row.owner == owner)
            || self.trait_surfaces.iter().any(|row| &row.declaration == owner)
            || self.dispatch_surfaces.iter().any(|row| &row.declaration == owner)
            || self.associated_surfaces.iter().any(|row| &row.owner == owner)
            || self.type_member_declarations.iter().any(|row| &row.owner == owner)
            || self.associated_static_domains.iter().any(|row| &row.declaration.owner == owner)
        {
            Ok(())
        } else {
            Err(invalid("semantic denotation references an absent declaration owner"))
        }
    }

    fn associated_surface(&self, owner: &StableDeclarationRef) -> Result<&AssociatedSurfaceRecordV1, ArtifactError> {
        let surface = self
            .associated_surfaces
            .iter()
            .find(|surface| &surface.owner == owner)
            .ok_or_else(|| invalid("associated denotation has no associated-surface owner"))?;
        self.validate_declaration_reference(owner)?;
        Ok(surface)
    }

    fn validate_associated_member_owner(&self, owner: &StableDeclarationRef, member: &AssociatedMemberV1) -> Result<(), ArtifactError> {
        if associated_member_owner(member) != owner {
            return Err(invalid("associated member owner disagrees with its lookup owner"));
        }
        let has_primary_owner_authority = match member {
            AssociatedMemberV1::Variant { variant } => self
                .structural_contracts
                .variant_contracts
                .iter()
                .any(|contract| contract.variant == *variant),
            AssociatedMemberV1::DataConstructor { owner } => self
                .structural_contracts
                .data_contracts
                .iter()
                .any(|contract| contract.owner == *owner),
            AssociatedMemberV1::DataComponent { owner, index } => self
                .structural_contracts
                .data_contracts
                .iter()
                .find(|contract| contract.owner == *owner)
                .is_some_and(|contract| (*index as usize) < contract.components.len()),
            AssociatedMemberV1::StaticDomain { declaration } => self
                .associated_static_domains
                .iter()
                .any(|contract| contract.declaration == *declaration),
            AssociatedMemberV1::TypeMember { owner, selector } => self
                .type_member_declarations
                .iter()
                .any(|contract| contract.owner == *owner && contract.selector == *selector),
        };
        if !has_primary_owner_authority {
            return Err(invalid("associated member has no primary owner authority"));
        }
        let surface = self.associated_surface(owner)?;
        if surface.families.iter().any(|family| family.members.contains(member)) {
            Ok(())
        } else {
            Err(invalid("associated denotation member is absent from its declaration surface"))
        }
    }

    fn validate_invocation_target(&self, target: &InvocationTargetV1) -> Result<(), ArtifactError> {
        match target {
            InvocationTargetV1::Behavioral { callable } => {
                let signature = self.callable_signatures.iter().any(|row| &row.callable == callable)
                    || self.enum_requirements.iter().any(|owner| owner.requirements.iter().any(|row| &row.requirement == callable))
                    || self.trait_surfaces.iter().any(|surface| {
                        surface.required_callables.contains(callable) || surface.default_callables.contains(callable)
                    })
                    || self.dispatch_surfaces.iter().any(|surface| {
                        surface.instance_callables.contains(callable) || surface.class_callables.contains(callable)
                    });
                if signature {
                    Ok(())
                } else {
                    Err(invalid("behavioral invocation target has no callable authority"))
                }
            }
            InvocationTargetV1::VariantConstructor { variant } => {
                if self.structural_contracts.variant_contracts.iter().any(|row| &row.variant == variant && row.constructor.is_some()) {
                    Ok(())
                } else {
                    Err(invalid("variant-constructor target has no exact variant constructor authority"))
                }
            }
            InvocationTargetV1::DataConstructor { owner } => {
                if self.structural_contracts.data_contracts.iter().any(|row| &row.owner == owner) {
                    Ok(())
                } else {
                    Err(invalid("data-constructor target has no data declaration authority"))
                }
            }
        }
    }

    fn invocation_target_matches_associated_member(&self, target: &InvocationTargetV1, member: &AssociatedMemberV1) -> bool {
        match (target, member) {
            (InvocationTargetV1::VariantConstructor { variant: target }, AssociatedMemberV1::Variant { variant: member }) => target == member,
            (InvocationTargetV1::DataConstructor { owner: target }, AssociatedMemberV1::DataConstructor { owner: member }) => target == member,
            _ => false,
        }
    }

    fn validate_conditional_dispatch_selection(&self, selection: &ConditionalDispatchSelectionV1) -> Result<(), ArtifactError> {
        if selection.side > 1 || selection.callable.side != if selection.side == 0 { StableDispatchSide::Instance } else { StableDispatchSide::Class } {
            return Err(invalid("conditional dispatch selection has an invalid or inconsistent side"));
        }
        // SemanticWorldV2 does not serialize the inherent-implementation
        // member table that proves this (impl, owner, callable) selection.
        // A conformance head is not that authority, and a callable signature
        // elsewhere cannot establish the relation. Reject until the exact
        // selection relation is transported in the artifact.
        Err(invalid("conditional dispatch selection has no serialized implementation-member authority"))
    }
}

fn associated_member_owner(member: &AssociatedMemberV1) -> &StableDeclarationRef {
    match member {
        AssociatedMemberV1::Variant { variant } => &variant.owner,
        AssociatedMemberV1::DataConstructor { owner }
        | AssociatedMemberV1::DataComponent { owner, .. }
        | AssociatedMemberV1::TypeMember { owner, .. } => owner,
        AssociatedMemberV1::StaticDomain { declaration } => &declaration.owner,
    }
}

fn validate_selector_base(base: &TypeMemberSelectorV1, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    if let TypeMemberSelectorV1::Named { name } = base
        && (name.is_empty() || name.len() > limits.max_string_bytes || name.contains(['(', ')', '[', ']', ',']))
    {
        return Err(invalid("semantic denotation selector base is empty, oversized, or malformed"));
    }
    Ok(())
}

fn validate_selector_slots(slots: &[SelectorSlotV1], limits: &ValidationLimits) -> Result<(), ArtifactError> {
    if slots.len() > u8::MAX as usize || slots.len() > limits.max_semantic_entries_per_family {
        return Err(invalid("semantic denotation selector slot count exceeds limits"));
    }
    let mut saw_label = false;
    for slot in slots {
        match slot {
            SelectorSlotV1::Positional if saw_label => return Err(invalid("semantic denotation selector has a positional slot after a label")),
            SelectorSlotV1::Positional => {}
            SelectorSlotV1::Label { label } => {
                if label.is_empty() || label.len() > limits.max_string_bytes {
                    return Err(invalid("semantic denotation selector label is empty or oversized"));
                }
                saw_label = true;
            }
        }
    }
    Ok(())
}

fn selector_kind_compatible(base: &TypeMemberSelectorV1, kind: &SelectorKindV1) -> bool {
    matches!(
        (base, kind),
        (TypeMemberSelectorV1::Named { .. }, SelectorKindV1::Getter | SelectorKindV1::Setter | SelectorKindV1::Method)
            | (TypeMemberSelectorV1::Subscript, SelectorKindV1::SubscriptGet | SelectorKindV1::SubscriptSet)
    )
}

fn validate_family_operation_shape(shape: &FamilyOperationShapeV1, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    validate_selector_slots(&shape.slots, limits)?;
    if matches!(shape.kind, SelectorKindV1::Getter | SelectorKindV1::Setter) && !shape.slots.is_empty() {
        return Err(invalid("getter or setter family operation has selector slots"));
    }
    Ok(())
}

fn validate_behavioral_family_spec(spec: &BehavioralFamilySpecV1, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    match spec {
        BehavioralFamilySpecV1::Exact { selector } => {
            validate_selector_base(&selector.base, limits)?;
            validate_selector_slots(&selector.slots, limits)?;
            if !selector_kind_compatible(&selector.base, &selector.kind)
                || matches!(selector.kind, SelectorKindV1::Getter | SelectorKindV1::Setter) && !selector.slots.is_empty()
            {
                return Err(invalid("behavioral exact-selector specification is structurally invalid"));
            }
        }
        BehavioralFamilySpecV1::Pattern { pattern } => {
            validate_selector_base(&pattern.base, limits)?;
            validate_selector_slots(&pattern.prefix, limits)?;
            validate_selector_slots(&pattern.suffix, limits)?;
            if pattern.prefix.iter().any(|slot| matches!(slot, SelectorSlotV1::Label { .. }))
                && pattern.suffix.iter().any(|slot| matches!(slot, SelectorSlotV1::Positional))
            {
                return Err(invalid("behavioral selector pattern has impossible prefix/suffix slot order"));
            }
            let kind_valid = match (&pattern.base, &pattern.kind) {
                (TypeMemberSelectorV1::Named { .. }, SelectorKindPatternV1::AnyNamed | SelectorKindPatternV1::NamedAccessors) => true,
                (TypeMemberSelectorV1::Subscript, SelectorKindPatternV1::AnySubscript) => true,
                (base, SelectorKindPatternV1::Exact { kind }) => selector_kind_compatible(base, kind),
                _ => false,
            };
            if !kind_valid
                || (!pattern.has_gap && !matches!(pattern.kind, SelectorKindPatternV1::NamedAccessors))
                || matches!(
                    pattern.kind,
                    SelectorKindPatternV1::Exact { kind: SelectorKindV1::Getter | SelectorKindV1::Setter }
                        | SelectorKindPatternV1::NamedAccessors
                ) && (!pattern.prefix.is_empty() || !pattern.suffix.is_empty())
            {
                return Err(invalid("behavioral selector-pattern specification is structurally invalid"));
            }
        }
    }
    Ok(())
}

fn behavioral_spec_matches_operation(spec: &BehavioralFamilySpecV1, operation: &FamilyOperationShapeV1) -> bool {
    match spec {
        BehavioralFamilySpecV1::Exact { selector } => selector.kind == operation.kind && selector.slots == operation.slots,
        BehavioralFamilySpecV1::Pattern { pattern } => {
            let kind_matches = match &pattern.kind {
                SelectorKindPatternV1::AnyNamed => matches!(
                    operation.kind,
                    SelectorKindV1::Getter | SelectorKindV1::Setter | SelectorKindV1::Method
                ),
                SelectorKindPatternV1::NamedAccessors => matches!(operation.kind, SelectorKindV1::Getter | SelectorKindV1::Setter),
                SelectorKindPatternV1::AnySubscript => matches!(operation.kind, SelectorKindV1::SubscriptGet | SelectorKindV1::SubscriptSet),
                SelectorKindPatternV1::Exact { kind } => *kind == operation.kind,
            };
            if !kind_matches {
                return false;
            }
            let fixed = pattern.prefix.len() + pattern.suffix.len();
            if (pattern.has_gap && operation.slots.len() < fixed) || (!pattern.has_gap && operation.slots.len() != fixed) {
                return false;
            }
            operation.slots.starts_with(&pattern.prefix) && operation.slots.ends_with(&pattern.suffix)
        }
    }
}

fn validate_callable_signature(signature: &CallableSignatureRecordV1, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    let callable_side = match signature.callable.side {
        StableDispatchSide::Instance => 0,
        StableDispatchSide::Class => 1,
    };
    if signature.side != callable_side
        || signature.selector.is_empty()
        || signature.selector.len() > limits.max_string_bytes
        || signature.selector != signature.callable.selector
        || signature.parameters.len() > limits.max_semantic_entries_per_family
    {
        return Err(invalid("callable signature identity or parameter count is invalid"));
    }
    if let StableCallableOwnerRef::Declaration(owner) = &signature.callable.owner {
        if owner != &signature.owner {
            return Err(invalid("callable signature declaration owner disagrees with callable identity"));
        }
    }
    if let Some(source) = &signature.source {
        if source.range.start > source.range.end {
            return Err(invalid("callable signature source span is inverted"));
        }
    }
    for (index, parameter) in signature.parameters.iter().enumerate() {
        if parameter.callable != signature.callable
            || usize::try_from(parameter.index).ok() != Some(index)
            || parameter.local_name.len() > limits.max_string_bytes
            || parameter.external_label.as_ref().is_some_and(|label| label.len() > limits.max_string_bytes)
        {
            return Err(invalid("callable parameter identity or string size is invalid"));
        }
        if let Some(source) = &parameter.source {
            if source.range.start > source.range.end {
                return Err(invalid("callable parameter source span is inverted"));
            }
        }
        match &parameter.contract {
            ParameterContractV1::Fixed { fact } => {
                if parameter.rest_mode != 0 || fact != &parameter.declared_type {
                    return Err(invalid("fixed callable parameter contract disagrees with its declared facts"));
                }
            }
            ParameterContractV1::Rest { mode, contract } => {
                if !(1..=3).contains(mode) || parameter.rest_mode != *mode {
                    return Err(invalid("rest callable parameter mode is invalid or inconsistent"));
                }
                match contract {
                    RestValueContractV1::Element { fact } if fact != &parameter.declared_type => {
                        return Err(invalid("rest element contract disagrees with its declared type"));
                    }
                    RestValueContractV1::Lane { .. }
                    | RestValueContractV1::Unconstrained
                    | RestValueContractV1::Invalid { .. }
                    | RestValueContractV1::Element { .. } => {}
                }
            }
        }
    }
    if let ReturnFlowV1::Argument { index } = signature.flow {
        let index = usize::try_from(index).map_err(|_| invalid("callable return flow index exceeds address space"))?;
        if index >= signature.parameters.len() {
            return Err(invalid("callable return flow references an absent parameter"));
        }
    }
    if let Some(TypeKnowledgeV1::Known { provenance, .. }) = &signature.inferred_return {
        if provenance.ranges.len() > limits.max_semantic_entries_per_family
            || provenance.descriptions.len() > limits.max_semantic_entries_per_family
            || provenance.descriptions.iter().any(|description| description.len() > limits.max_string_bytes)
            || provenance.ranges.iter().any(|range| range.start > range.end)
        {
            return Err(invalid("inferred callable return provenance exceeds limits or has an inverted range"));
        }
    }
    if let Some(native_id) = &signature.native_id {
        if native_id.owner.is_empty()
            || native_id.owner.len() > limits.max_string_bytes
            || native_id.selector.is_empty()
            || native_id.selector.len() > limits.max_string_bytes
            || native_id.side > 1
        {
            return Err(invalid("native surface identity is invalid"));
        }
    }
    if signature.lifecycle.replacement.is_some() && signature.lifecycle.deprecated_since.is_none()
        || [
            signature.lifecycle.since.as_ref(),
            signature.lifecycle.deprecated_since.as_ref(),
            signature.lifecycle.replacement.as_ref(),
        ]
        .into_iter()
        .flatten()
        .any(|value| value.len() > limits.max_string_bytes)
    {
        return Err(invalid("callable native lifecycle metadata is inconsistent or oversized"));
    }
    validate_effects(&signature.effects, limits)?;
    validate_raises(&signature.raises, limits)?;
    Ok(())
}

fn validate_effects(effects: &EffectSpecV1, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    if let EffectSpecV1::Known { effects } = effects {
        if effects.len() > limits.max_semantic_entries_per_family {
            return Err(invalid("native effect count exceeds limits"));
        }
        for (index, effect) in effects.iter().enumerate() {
            if effects[..index].contains(effect) {
                return Err(invalid("native effect list contains a duplicate"));
            }
        }
    }
    Ok(())
}

fn validate_raises(raises: &RaisesSpecV1, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    if let RaisesSpecV1::Known { types } = raises {
        if types.len() > limits.max_semantic_entries_per_family {
            return Err(invalid("native raises type count exceeds limits"));
        }
        for ty in types {
            validate_type_expr(ty, limits, 0)?;
        }
    }
    Ok(())
}

fn validate_type_expr(ty: &TypeExprSpecV1, limits: &ValidationLimits, depth: usize) -> Result<(), ArtifactError> {
    if depth >= limits.max_recursion_depth {
        return Err(invalid("native type expression nesting limit exceeded"));
    }
    match ty {
        TypeExprSpecV1::Unknown | TypeExprSpecV1::Never | TypeExprSpecV1::SelfType => Ok(()),
        TypeExprSpecV1::Universe { owner } if owner.is_empty() || owner.len() > limits.max_string_bytes => Err(invalid("native universe type owner is invalid")),
        TypeExprSpecV1::Universe { .. } => Ok(()),
        TypeExprSpecV1::Parameter { name } if name.is_empty() || name.len() > limits.max_string_bytes => Err(invalid("native type parameter name is invalid")),
        TypeExprSpecV1::Parameter { .. } => Ok(()),
        TypeExprSpecV1::Applied { origin, arguments } => {
            if arguments.len() > limits.max_semantic_entries_per_family {
                return Err(invalid("native applied type argument count exceeds limits"));
            }
            validate_type_expr(origin, limits, depth + 1)?;
            for argument in arguments {
                validate_type_expr(argument, limits, depth + 1)?;
            }
            Ok(())
        }
        TypeExprSpecV1::Union { members } => {
            if members.len() > limits.max_semantic_entries_per_family {
                return Err(invalid("native union member count exceeds limits"));
            }
            for member in members {
                validate_type_expr(member, limits, depth + 1)?;
            }
            Ok(())
        }
        TypeExprSpecV1::Tuple { tuple } => {
            if tuple.positional.len() > limits.max_semantic_entries_per_family
                || tuple.labeled.len() > limits.max_semantic_entries_per_family
                || tuple.labeled.iter().any(|parameter| parameter.label.is_empty() || parameter.label.len() > limits.max_string_bytes)
            {
                return Err(invalid("native tuple parameter count or label exceeds limits"));
            }
            for member in &tuple.positional {
                validate_type_expr(member, limits, depth + 1)?;
            }
            for parameter in &tuple.labeled {
                validate_type_expr(&parameter.ty, limits, depth + 1)?;
            }
            if let Some(rest) = &tuple.rest {
                if let Some(ty) = &rest.ty {
                    validate_type_expr(ty, limits, depth + 1)?;
                }
            }
            Ok(())
        }
    }
}

/// Validation context for one callable record's local ordinal coordinate
/// space. Equal ordinal values in different records remain unrelated.
struct CallableAnalysisOrdinalContext<'a> {
    record: &'a CallableAnalysisRecordV1,
}

impl<'a> CallableAnalysisOrdinalContext<'a> {
    fn new(record: &'a CallableAnalysisRecordV1) -> Self {
        Self { record }
    }

    fn has_expression(&self, ordinal: u32) -> bool {
        ordinal < self.record.expression_count
    }

    fn has_binding(&self, ordinal: u32) -> bool {
        (ordinal as usize) < self.record.binding_origins.len()
    }

    fn has_flow_node(&self, ordinal: u32) -> bool {
        (ordinal as usize) < self.record.flow_graph.nodes.len()
    }

    fn validate(&self, limits: &ValidationLimits, allocations: &mut AllocationBudget) -> Result<(), ArtifactError> {
        let record = self.record;
        if record.flow_graph.nodes.len() > limits.max_semantic_entries_per_family
            || record.flow_graph.edges.len() > limits.max_semantic_entries_per_family
            || record.flow_graph.entry.is_some_and(|entry| !self.has_flow_node(entry))
            || record.flow_graph.exits.len() > record.flow_graph.nodes.len()
        {
            return Err(invalid("callable flow graph exceeds its owner-local limits or has an invalid entry/exit"));
        }
        if record.flow_graph.exits.iter().any(|exit| !self.has_flow_node(*exit))
            || record
                .flow_graph
                .edges
                .iter()
                .any(|edge| !self.has_flow_node(edge.source) || !self.has_flow_node(edge.target))
        {
            return Err(invalid("callable flow graph contains a dangling node reference"));
        }
        if let Some(entry) = record.flow_graph.entry
            && !matches!(&record.flow_graph.nodes[entry as usize].kind, CallableFlowNodeKindV1::Entry)
        {
            return Err(invalid("callable flow graph entry ordinal does not identify an entry node"));
        }
        if record.flow_graph.exits.iter().any(|exit| {
            !matches!(
                &record.flow_graph.nodes[*exit as usize].kind,
                CallableFlowNodeKindV1::Exit | CallableFlowNodeKindV1::Throw
            )
        }) {
            return Err(invalid("callable flow graph exit ordinal does not identify an exit or throw node"));
        }
        if !record.flow_graph.exits.is_empty() {
            let mut seen = allocations.vector::<u8>(record.flow_graph.nodes.len()).map_err(allocation_error)?;
            seen.resize(record.flow_graph.nodes.len(), 0);
            for exit in &record.flow_graph.exits {
                let slot = &mut seen[*exit as usize];
                if *slot != 0 {
                    return Err(invalid("callable flow graph contains a duplicate exit"));
                }
                *slot = 1;
            }
        }

        if record.formal_bindings.len() > limits.max_semantic_entries_per_family
            || record.binding_origins.len() > limits.max_semantic_entries_per_family
            || record.expression_denotations.len() > limits.max_semantic_entries_per_family
            || record.binding_denotations.len() > limits.max_semantic_entries_per_family
            || record.call_specializations.len() > limits.max_semantic_entries_per_family
            || record
                .binding_origins
                .iter()
                .enumerate()
                .any(|(index, origin)| origin.binding_ordinal as usize != index)
            || record.formal_bindings.windows(2).any(|pair| pair[0].binding_ordinal >= pair[1].binding_ordinal)
            || record
                .expression_denotations
                .windows(2)
                .any(|pair| pair[0].expression_ordinal >= pair[1].expression_ordinal)
            || record
                .binding_denotations
                .windows(2)
                .any(|pair| pair[0].binding_ordinal >= pair[1].binding_ordinal)
            || record
                .call_specializations
                .windows(2)
                .any(|pair| pair[0].expression_ordinal >= pair[1].expression_ordinal)
            || record.expression_denotations.iter().any(|entry| !self.has_expression(entry.expression_ordinal))
            || record.binding_denotations.iter().any(|entry| !self.has_binding(entry.binding_ordinal))
            || record.call_specializations.iter().any(|call| !self.has_expression(call.expression_ordinal))
            || record
                .formal_bindings
                .iter()
                .any(|attachment| !self.has_binding(attachment.binding_ordinal) || attachment.range.start > attachment.range.end)
        {
            return Err(invalid("callable binding origins or formal attachments are invalid or unsorted"));
        }
        Ok(())
    }
}

fn allocation_error(error: AllocationError) -> ArtifactError {
    invalid(match error {
        AllocationError::Exhausted => "semantic world allocation budget exceeded",
        AllocationError::Overflow => "semantic world allocation layout overflow",
        AllocationError::Failed => "semantic world allocation failed",
        AllocationError::UnaccountedBoundary => "semantic world has an unaccounted allocation boundary",
    })
}

fn require_semantic_validation_allocation_accounting() -> Result<(), ArtifactError> {
    Err(AllocationError::UnaccountedBoundary.into())
}
