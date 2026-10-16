//! Combined semantic world section payload (`PERF003.C2.P7.T7.B/T8`).

use crate::module_world::SpanV1;
use crate::module_world::{Decode, Wire, invalid, record, tagged};
use crate::semantic::aggregates::{DeclaredTypeFactV1, GenericSignatureV1, StructuralPrimaryContractsV1, TraitRefV1, TypeTermV1, UnknownReasonV1};
use crate::semantic::operational::OperationalOpV1;
use crate::semantic::static_value::{ArtifactStaticValueId, StaticValueGraphV1};
use crate::semantic::type_store::{
    ArtifactKindId, ArtifactStaticConstructorId, ArtifactStaticDomainId, ArtifactTypeId, ArtifactTypeParameterId, StaticArgumentV1, TypeGraphV1,
    TypeMemberSelectorV1,
};
use crate::{ArtifactError, BinaryReader, BinaryWriter, ValidationLimits};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::identity::{
    StableAssociatedStaticDomainDeclarationRef, StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide, StableImplRef,
    StableModuleRef, StableProjectRef, StableVariantRef,
};

pub const SEMANTIC_WORLD_SCHEMA_V3: u32 = 3;

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
    requirement_views: Vec<CallableRequirementViewV1>,
    requirements: Vec<CallableRequirementSelectionEntryV1>,
    associated_type_plan: CallableConformanceAssociatedTypePlanV1,
    invalid_explicit_members: bool,
    completeness: CallableConformanceCompletenessV1,
    diagnostics: Vec<CallableEvidenceDiagnosticV1>,
    conformance_evidence: Vec<CallableConformanceEvidenceNodeV1>,
    fingerprint: u64,
    ordered_selected_callables: Vec<StableCallableRef>,
    slot_layout_fingerprint: [u8; 32],
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

tagged!(CallableDefinitionOriginV1 {
    0 => PrimaryDeclaration,
    1 => InherentImpl { impl_id: StableImplRef },
    2 => ConformanceWitness { impl_id: StableImplRef },
});

tagged!(CallableDefinitionBodyProvenanceV1 {
    0 => SourceBody,
    1 => GeneratedDelegation { target_field: Box<str> },
});

record!(CoveringImplParameterBindingV1 {
    impl_parameter: ArtifactTypeParameterId,
    declaration_parameter: ArtifactTypeParameterId,
});

record!(CoveringImplSubstitutionV1 {
    impl_to_decl: Vec<CoveringImplParameterBindingV1>,
    decl_to_impl: Vec<CoveringImplParameterBindingV1>,
});

record!(InherentBodyContextV1 {
    generic_signature: Option<GenericSignatureV1>,
    covering: Option<CoveringImplSubstitutionV1>,
    source_target_head: ArtifactTypeId,
});

record!(CallableDefinitionRecordV1 {
    callable: StableCallableRef,
    origin: CallableDefinitionOriginV1,
    source_member_index: u32,
    signature: CallableSignatureRecordV1,
    body_provenance: CallableDefinitionBodyProvenanceV1,
    inherent_body_context: Option<InherentBodyContextV1>,
});

tagged!(CallableAnalysisStatusV1 {
    0 => Complete,
    1 => Partial,
    2 => Blocked,
    3 => Cancelled,
    4 => BudgetExceeded,
});

tagged!(CallableSuppressionCauseV1 { 0 => One { cause: u32 }, 1 => Multiple });

// InternalFailure remains ineligible while its incident owner has no callable wire row.
tagged!(CallableExpressionStatusV1 {
    0 => Ready,
    1 => Invalid { cause: u32 },
    2 => Suppressed { cause: CallableSuppressionCauseV1 },
    3 => Blocked { reason: CallableBlockReasonV1 },
    4 => DynamicBoundary { reason: crate::semantic::aggregates::DynamicReasonV1 },
    5 => Cancelled,
    6 => BudgetExceeded { report: CallableBudgetReportV1 },
});

tagged!(CallableBindingReadSafetyV1 {
    0 => ProvenInitialized,
    1 => DefinitelyUninitialized,
    2 => RuntimeCheckRequired,
});

// Query identities nested in callable facts have a callable-local portable
// contract. This is intentionally separate from the shared query-image wire:
// it serializes only owner identity, not cached query state or dependencies.
record!(CallableFieldRefV1 {
    owner: StableDeclarationRef,
    side: StableDispatchSide,
    name: Box<str>,
});

tagged!(CallableQueryOwnerV1 {
    0 => CallableSignature { callable: StableCallableRef },
    1 => FieldSignature { field: CallableFieldRefV1 },
    2 => GenericConformance { subject: TypeTermV1, trait_ref: TraitRefV1 },
    3 => TraitDispatchBucket { target_family: ConformanceTargetV1, selector: ExactSelectorV1, side: StableDispatchSide },
});

tagged!(CallableBudgetKindV1 {
    0 => Steps,
    1 => RelationPairs,
    2 => SccIterations,
    3 => TypeDepth,
    4 => DiagnosticNotes,
    5 => ResidualNodes,
});

record!(CallableBudgetReportV1 {
    kind: CallableBudgetKindV1,
    limit: u64,
    used: u64
});

tagged!(CallableBlockReasonV1 {
    0 => UnknownType { reason: UnknownReasonV1 },
    1 => UnresolvedDependency { project_source: Box<str>, module_path: Vec<Box<str>> },
    2 => InvalidAnnotation { diagnostic_code: Box<str> },
    3 => RecursiveFixpoint,
    4 => InferenceDidNotConverge,
    5 => OpaqueNative { message: Box<str> },
    6 => ReflectionBoundary,
    7 => UnavailableSemanticOwner { owner: CallableQueryOwnerV1 },
    8 => BudgetExceeded { report: CallableBudgetReportV1 },
    9 => SuppressedDependency,
});

tagged!(CallableBindingAssumptionBasisV1 {
    0 => MissingValueEvidence { reason: UnknownReasonV1 },
    1 => CallableParameterContract,
    2 => ContextualParameterContract,
    3 => DerivedEvidence { origin: EvidenceOriginV1 },
});

tagged!(CallableRefutationReasonV1 { 0 => IncompatibleNominal, 1 => TypeMismatch, 2 => UnionMemberMismatch });

tagged!(CallableBindingConsistencyV1 {
    0 => Unconstrained,
    1 => Validated,
    2 => Assumed { basis: CallableBindingAssumptionBasisV1 },
    3 => Refuted { actual: ArtifactTypeId, expected: ArtifactTypeId, reason: CallableRefutationReasonV1 },
    4 => DynamicBoundary { reason: Box<str> },
    5 => Blocked { reason: CallableBlockReasonV1 },
    6 => Cancelled,
    7 => BudgetExceeded { report: CallableBudgetReportV1 },
    8 => InternalFailure { message: Box<str> },
});

tagged!(CallableBindingContractOriginV1 {
    0 => SourceAnnotation,
    1 => InferredInitializer,
    2 => CallableParameter,
    3 => ContextualBlockParameter,
    4 => PatternBinding,
    5 => StaticParameter,
});

record!(CallableBindingContractV1 { ty: ArtifactTypeId, origin: CallableBindingContractOriginV1, source: Option<SpanV1> });

tagged!(CallableCausalInvalidityV1 { 0 => Clean, 1 => One { cause: u32 }, 2 => Multiple });

tagged!(CallableInitializationStateV1 { 0 => Uninitialized, 1 => MaybeInitialized, 2 => DefinitelyInitialized });

record!(CallableParameterIdentityV1 {
    callable: StableCallableRef,
    index: u32
});

tagged!(CallableGeneratedBindingReasonV1 { 0 => ImplicitReceiver });

tagged!(CallableBindingAuthorityV1 {
    0 => BodyLocalSource { site_owner: CallableAnalysisSourceOwnerV1, site_local: u32, declaration_range: SpanV1 },
    1 => BodyLocalGenerated { reason: CallableGeneratedBindingReasonV1 },
    2 => ModuleValue { module: StableModuleRef, name: String, import_binding: Option<u32> },
});

record!(ConstructorSuperInitializerV1 {
    call_range: SpanV1,
    initializer: StableCallableRef,
});

tagged!(CallableAnalysisSourceOwnerV1 {
    0 => Module { module: StableModuleRef },
    1 => Callable { callable: StableCallableRef },
});

record!(CallableBindingRowV1 {
    ordinal: u32,
    name: String,
    parameter: Option<CallableParameterIdentityV1>,
    range: SpanV1,
    contract: Option<CallableBindingContractV1>,
    current: TypeKnowledgeV1,
    denotation: Option<SemanticDenotationV1>,
    consistency: CallableBindingConsistencyV1,
    causal_invalidity: CallableCausalInvalidityV1,
    mutable: bool,
    initialization: CallableInitializationStateV1,
    version: u32,
    explanation: Option<u32>,
    authority: CallableBindingAuthorityV1,
});

record!(CallableExpressionRowV1 {
    ordinal: u32,
    range: SpanV1,
    knowledge: TypeKnowledgeV1,
    callable: Option<StableCallableRef>,
    conditional_dispatch: Option<ConditionalDispatchSelectionV1>,
    status: CallableExpressionStatusV1,
    causal_invalidity: CallableCausalInvalidityV1,
    binding_read_safety: Option<CallableBindingReadSafetyV1>,
    binding_read: Option<u32>,
    binding_write: Option<u32>,
    denotation: Option<SemanticDenotationV1>,
    call_specialization: Option<CallableCallSpecializationV1>,
});

record!(CallableStaticArgumentBindingV1 {
    parameter: ArtifactTypeParameterId,
    argument: StaticArgumentV1,
});

record!(CallableCallSpecializationV1 {
    callable: StableCallableRef,
    bindings: Vec<CallableStaticArgumentBindingV1>,
    conformances: Vec<CallableGenericCallConformanceBindingV1>,
});

record!(GenericConformanceKeyV1 {
    subject: TypeTermV1,
    trait_ref: TraitRefV1,
});

record!(CallableGenericCallConformanceBindingV1 {
    callee_conformance: GenericConformanceKeyV1,
    source: CallableGenericCallConformanceSourceV1,
});

tagged!(CallableGenericCallConformanceSourceV1 {
    0 => Exact { evidence: u32 },
    1 => Forwarded { caller_conformance: GenericConformanceKeyV1 },
    2 => Derived { evidence: u32 },
    3 => CompilerDerived { evidence: u32 },
});

// Evidence ordinals are local to one CallableAnalysisRecordV1 and are never
// semantic identities. Recursive proof edges are represented as ordinals so
// the validator can bound and reject cyclic graphs before native construction.
tagged!(CallableConformanceEvidenceNodeV1 {
    0 => Exact { evidence: CallableExactConformanceEvidenceV1 },
    1 => Assumed { key: GenericConformanceKeyV1, owner: crate::semantic::type_store::ParameterOwnerV1 },
    2 => Derived { evidence: CallableDerivedConformanceEvidenceV1 },
    3 => CompilerDerived { evidence: CallableCompilerDerivedConformanceEvidenceV1 },
});

record!(CallableExactConformanceEvidenceV1 {
    source_impl: StableImplRef,
    exact_target: ArtifactTypeId,
    exact_trait_ref: TraitRefV1,
    impl_environment: CallableTypeEnvironmentV1,
    requirement_views: Vec<CallableRequirementViewV1>,
    requirements: Vec<CallableRequirementSelectionEntryV1>,
    associated_types: Vec<CallableExactAssociatedTypeBindingV1>,
    trait_header_contract: Vec<crate::semantic::aggregates::GenericConstraintV1>,
    trait_header_conformance_proofs: Option<Vec<CallableGenericConformanceProofRefV1>>,
});

record!(CallableTypeEnvironmentV1 {
    bindings: Vec<CallableStaticArgumentBindingV1>,
    self_binding: Option<ArtifactTypeId>,
});

record!(CallableGenericConformanceProofRefV1 {
    key: GenericConformanceKeyV1,
    evidence: u32,
});

record!(CallableRequirementIdentityV1 {
    owner: StableDeclarationRef,
    selector: Box<str>,
    side: u8,
});

record!(CallableAssociatedTypeRequirementIdentityV1 {
    owner: StableDeclarationRef,
    index: u32,
});

record!(CallableRequirementViewV1 {
    requirement: CallableRequirementIdentityV1,
    callable: StableCallableRef,
    signature: CallableSignatureRecordV1,
    visibility: crate::semantic::aggregates::VisibilityV1,
    default_callable: Option<StableCallableRef>,
});

record!(CallableRequirementSelectionEntryV1 {
    requirement: CallableRequirementIdentityV1,
    selection: CallableRequirementSelectionV1,
});

tagged!(CallableRequirementSelectionV1 {
    0 => ConformanceCallable { callable: StableCallableRef },
    1 => InherentCallable {
        callable: StableCallableRef,
        conditional_impl: Option<StableImplRef>,
        applicability: Option<CallableInherentImplSpecializationV1>
    },
    2 => ConditionalInherent {
        candidate: CallableEffectiveInherentWitnessV1,
        pending: CallableConformanceCompletenessV1,
        fallback: Option<Box<CallableRequirementSelectionV1>>
    },
    3 => DataComponent { owner: StableDeclarationRef, index: u32, specialized_type: ArtifactTypeId },
    4 => TraitDefault { callable: StableCallableRef },
});

record!(CallableInherentImplSpecializationV1 {
    impl_id: StableImplRef,
    receiver: ArtifactTypeId,
    owner_view: ArtifactTypeId,
    bindings: Vec<CallableTypeBindingV1>,
    environment: CallableTypeEnvironmentV1,
    conformance_provenance: Option<Vec<CallableGenericCallConformanceBindingV1>>,
});

record!(CallableTypeBindingV1 {
    parameter: ArtifactTypeParameterId,
    ty: ArtifactTypeId,
});

record!(CallableEffectiveInherentWitnessV1 {
    callable: StableCallableRef,
    signature: CallableSignatureRecordV1,
    visibility: crate::semantic::aggregates::VisibilityV1,
    owner: StableDeclarationRef,
    conditional_impl: Option<StableImplRef>,
    applicability: Option<CallableInherentImplSpecializationV1>,
    conditional_domain: Option<CallableInherentImplDomainV1>,
});

record!(CallableInherentImplDomainV1 {
    impl_id: StableImplRef,
    target: CallableInherentImplTargetV1,
    head_type: ArtifactTypeId,
    generic_signature: Option<crate::semantic::aggregates::GenericSignatureV1>,
    constraints: Vec<crate::semantic::aggregates::GenericConstraintV1>,
});

tagged!(CallableInherentImplTargetV1 {
    0 => Declaration { declaration: StableDeclarationRef },
    1 => ExactEnumCase { variant: StableVariantRef },
});

record!(CallableExactAssociatedTypeBindingV1 {
    requirement: CallableAssociatedTypeRequirementIdentityV1,
    value: CallableAssociatedTypeBindingValueV1,
    source_impl: StableImplRef,
    source: EnumRequirementSourceV1,
});

record!(CallableAssociatedTypeBindingTemplateV1 {
    requirement: CallableAssociatedTypeRequirementIdentityV1,
    value: CallableAssociatedTypeBindingValueV1,
    source_impl: StableImplRef,
    source: EnumRequirementSourceV1,
});

tagged!(CallableAssociatedTypeBindingValueV1 {
    0 => Type { ty: ArtifactTypeId },
    1 => Dynamic,
});

record!(CallableAssociatedTypeBindingFailureV1 {
    kind: CallableAssociatedTypeBindingFailureKindV1,
    name: Box<str>,
    reason: Box<str>,
    source: EnumRequirementSourceV1,
    requirement: Option<CallableAssociatedTypeRequirementIdentityV1>,
});

tagged!(CallableAssociatedTypeBindingFailureKindV1 {
    0 => Missing,
    1 => Duplicate,
    2 => Unknown,
    3 => Invalid,
});

record!(CallableConformanceAssociatedTypePlanV1 {
    impl_id: StableImplRef,
    bindings: Vec<CallableAssociatedTypeBindingTemplateV1>,
    failures: Vec<CallableAssociatedTypeBindingFailureV1>,
    diagnostics: Vec<CallableEvidenceDiagnosticV1>,
});

tagged!(CallableConformanceCompletenessV1 {
    0 => Complete,
    1 => Incomplete { failures: Vec<CallableConformanceFailureV1> },
    2 => Unknown { reason: crate::semantic::aggregates::UnknownReasonV1 },
    3 => Blocked { reason: CallableBlockReasonV1 },
    4 => Dynamic { reason: Box<str> },
    5 => Cancelled,
    6 => BudgetExceeded { report: CallableBudgetReportV1 },
    7 => InternalFailure { message: Box<str> },
});

tagged!(CallableConformanceFailureV1 {
    0 => Behavioral { requirement: CallableRequirementIdentityV1, reason: Box<str> },
    1 => AssociatedType { failure: CallableAssociatedTypeBindingFailureV1 },
});

record!(CallableDerivedConformanceEvidenceV1 {
    origin: CallableDerivedConformanceOriginV1,
    impl_bindings: Vec<CallableTypeBindingV1>,
    nested: Vec<CallableNestedGenericEvidenceV1>,
});

tagged!(CallableDerivedConformanceOriginV1 {
    0 => ConditionalSource { exact_source: u32 },
    1 => GuardedRule {
        rule: CallableDerivedConformanceRuleV1,
        exact_subject: ArtifactTypeId,
        exact_trait_ref: TraitRefV1
    },
});

record!(CallableDerivedConformanceRuleV1 {
    impl_id: StableImplRef,
    source_module: StableModuleRef,
    target_parameter: ArtifactTypeParameterId,
    target_template: ArtifactTypeId,
    trait_ref: TraitRefV1,
    generic_signature: crate::semantic::aggregates::GenericSignatureV1,
    associated_type_plan: CallableConformanceAssociatedTypePlanV1,
    requirement_views: Vec<CallableRequirementViewV1>,
    requirements: Vec<CallableRequirementSelectionEntryV1>,
    source: EnumRequirementSourceV1,
});

record!(DerivedConformanceRuleRecordV1 {
    rule: CallableDerivedConformanceRuleV1,
    conformance_evidence: Vec<CallableConformanceEvidenceNodeV1>,
});

tagged!(CallableNestedGenericEvidenceV1 {
    0 => Conformance { evidence: u32 },
    1 => Equivalent,
    2 => Subtype,
});

record!(CallableCompilerDerivedConformanceEvidenceV1 {
    key: GenericConformanceKeyV1,
    kind: CallableCompilerDerivedConformanceKindV1,
    classification: CallableStructuralClassificationV1,
});

tagged!(CallableCompilerDerivedConformanceKindV1 {
    0 => Product,
    1 => OrderedProduct,
    2 => UnorderedProduct,
    3 => Sum,
});

tagged!(CallableStructuralClassificationV1 {
    0 => Unsupported,
    1 => OrderedProduct { form: CallableOrderedProductFormV1 },
    2 => UnorderedProduct { form: CallableUnorderedProductFormV1 },
    3 => Sum { owner: StableDeclarationRef },
});

tagged!(CallableOrderedProductFormV1 {
    0 => Unit,
    1 => Tuple,
    2 => TupleFromPacks,
    3 => Pack,
    4 => PackFromPacks,
    5 => Data { owner: StableDeclarationRef },
});

tagged!(CallableUnorderedProductFormV1 {
    0 => Record,
    1 => Data { owner: StableDeclarationRef },
});

record!(CallableEvidenceDiagnosticV1 {
    code: Box<str>,
    severity: CallableDiagnosticSeverityV1,
    message: Box<str>,
    primary: EnumRequirementSourceV1,
    primary_range: SpanV1,
    labels: Vec<CallableEvidenceDiagnosticLabelV1>,
    notes: Vec<Box<str>>,
    helps: Vec<Box<str>>,
    guidance: Vec<CallableEvidenceDiagnosticGuidanceV1>,
    explanations: Vec<CallableEvidenceExplanationRefV1>,
    fixes: Vec<CallableEvidenceDiagnosticFixV1>,
    root_cause: Option<u32>,
});

tagged!(CallableDiagnosticSeverityV1 {
    0 => Error,
    1 => Warning,
    2 => Information,
    3 => Hint,
});

record!(CallableEvidenceDiagnosticLabelV1 {
    source: EnumRequirementSourceV1,
    range: SpanV1,
    message: Box<str>,
});

tagged!(CallableEvidenceDiagnosticGuidanceV1 {
    0 => ChangeAnnotation { range: SpanV1, ty: ArtifactTypeId },
    1 => SupplyAssignableValue { expected: ArtifactTypeId },
    2 => UseCallableShape { callable: StableCallableRef },
    3 => EstablishTypeEvidence { range: SpanV1, expected: Option<ArtifactTypeId> },
    4 => ResolveGenericParameter { parameter: ArtifactTypeParameterId },
});

record!(CallableEvidenceExplanationRefV1 {
    callable: StableCallableRef,
    ordinal: u32,
});

record!(CallableEvidenceDiagnosticFixV1 {
    message: Box<str>,
    replacement: Option<CallableEvidenceDiagnosticReplacementV1>,
});

record!(CallableEvidenceDiagnosticReplacementV1 {
    range: SpanV1,
    text: Box<str>,
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
    expressions: Vec<CallableExpressionRowV1>,
    conformance_evidence: Vec<CallableConformanceEvidenceNodeV1>,
    bindings: Vec<CallableBindingRowV1>,
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

record!(TraitSurfaceMemberV1 {
    requirement: CallableRequirementIdentityV1,
    callable: StableCallableRef,
    signature: CallableSignatureRecordV1,
    visibility: crate::semantic::aggregates::VisibilityV1,
    source: EnumRequirementSourceV1,
    source_member_index: u32,
    default_present: bool,
    default_source: Option<EnumRequirementSourceV1>,
});

record!(TraitAssociatedTypeRequirementV1 {
    requirement: CallableAssociatedTypeRequirementIdentityV1,
    name: Box<str>,
    domain: TypeMemberDomainV1,
    source: EnumRequirementSourceV1,
});

record!(TraitSurfaceRecordV1 {
    declaration: StableDeclarationRef,
    generic_signature: Option<GenericSignatureV1>,
    header_source: EnumRequirementSourceV1,
    members: Vec<TraitSurfaceMemberV1>,
    associated_types: Vec<TraitAssociatedTypeRequirementV1>,
    diagnostics: Vec<CallableEvidenceDiagnosticV1>,
});

record!(FieldSignatureV1 {
    owner: StableDeclarationRef,
    name: Box<str>,
    declared_type: Option<DeclaredTypeFactV1>,
    knowledge: TypeKnowledgeV1,
    mutable: Option<bool>,
    side: u8,
    visibility: crate::semantic::aggregates::VisibilityV1,
    source: Option<SpanV1>,
});

record!(DispatchCallableFactV1 {
    callable: StableCallableRef,
    knowledge: TypeKnowledgeV1,
    visibility: crate::semantic::aggregates::VisibilityV1,
});

record!(DeclarationSurfaceRecordV1 {
    declaration: StableDeclarationRef,
    instance_callables: Vec<DispatchCallableFactV1>,
    class_callables: Vec<DispatchCallableFactV1>,
    fields: Vec<FieldSignatureV1>,
});

tagged!(InherentImplTargetV1 {
    0 => Declaration { declaration: StableDeclarationRef },
    1 => ExactEnumCase { variant: StableVariantRef },
});

record!(InherentImplDomainV1 {
    impl_id: StableImplRef,
    target: InherentImplTargetV1,
    head_type: ArtifactTypeId,
    generic_signature: Option<GenericSignatureV1>,
    constraints: Vec<crate::semantic::aggregates::GenericConstraintV1>,
});

record!(ConditionalInherentMemberV1 {
    impl_id: StableImplRef,
    member_order: u32,
    is_conditionally_applicable: bool,
    callable: StableCallableRef,
    signature: CallableSignatureRecordV1,
    visibility: crate::semantic::aggregates::VisibilityV1,
    source_member_index: u32,
    is_requirement: bool,
});

record!(ConditionalInherentSetV1 {
    target: InherentImplTargetV1,
    domains: Vec<InherentImplDomainV1>,
    members: Vec<ConditionalInherentMemberV1>,
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

tagged!(StaticCaseDomainV1 {
    0 => Nominal { declaration: StableDeclarationRef },
    1 => Associated { declaration: StableAssociatedStaticDomainDeclarationRef }
});

tagged!(StaticCaseShapeV1 {
    0 => Singleton,
    1 => Constructor { slots: Vec<SelectorSlotV1> }
});

record!(StaticCaseHeaderV1 {
    selector: ExactSelectorV1,
    shape: StaticCaseShapeV1,
    source: SpanV1,
});

record!(StaticDomainCaseCatalogRecordV1 {
    domain: StaticCaseDomainV1,
    cases: Vec<StaticCaseHeaderV1>,
});

record!(StaticCaseSignatureRecordV1 {
    domain: StaticCaseDomainV1,
    selector: ExactSelectorV1,
    parameter_kinds: Vec<ArtifactKindId>,
    source: SpanV1,
});

tagged!(StaticValueDomainV1 {
    0 => Kind { kind: ArtifactKindId },
    1 => ValueType { ty: ArtifactTypeId },
});

record!(StaticParameterV1 {
    name: Box<str>,
    domain: StaticValueDomainV1,
    mode: crate::semantic::type_store::ParameterModeV1,
    index: u32,
    type_parameter: Option<ArtifactTypeParameterId>,
    source: SpanV1,
});

record!(StaticDefinitionSignatureV1 {
    parameters: Vec<StaticParameterV1>,
    result_domain: StaticValueDomainV1,
    generic_signature: Option<GenericSignatureV1>,
});

tagged!(CanonicalStaticApplicationArgumentV1 {
    0 => KindedTerm { parameter_index: u32, term: ArtifactTypeId },
    1 => KindedExpression { parameter_index: u32, application_ordinal: u32 },
    2 => Value { parameter_index: u32 },
});

record!(CanonicalCheckedStaticApplicationV1 {
    ordinal: u32,
    source: SpanV1,
    target: StableDeclarationRef,
    signature: StaticDefinitionSignatureV1,
    arguments: Vec<CanonicalStaticApplicationArgumentV1>,
});

record!(StaticConstantSymbolV1 {
    module: StableModuleRef,
    name: Box<str>,
});

record!(StaticFieldDependencyV1 {
    owner: StableDeclarationRef,
    side: StableDispatchSide,
    name: Box<str>,
});

tagged!(StaticSemanticDependencyV1 {
    0 => DeclarationShell { owner: StableDeclarationRef },
    1 => TraitHeader { owner: StableDeclarationRef },
    2 => CallableSignature { callable: StableCallableRef },
    3 => FieldSignature { field: StaticFieldDependencyV1 },
    4 => DeclarationSurface { owner: StableDeclarationRef },
    5 => TraitSurface { owner: StableDeclarationRef },
    6 => ConformanceDispatch { implementation: StableImplRef },
    7 => TraitDispatchBucket { target: ConformanceTargetV1, selector: ExactSelectorV1, side: StableDispatchSide },
    8 => GenericConformance { subject: TypeTermV1, trait_ref: TraitRefV1 },
    9 => HierarchyEdge { owner: StableDeclarationRef },
    10 => LinkedInterface { module: StableModuleRef },
    11 => DataDeclaration { owner: StableDeclarationRef },
    12 => EnumDeclaration { owner: StableDeclarationRef },
    13 => AssociatedSurface { owner: StableDeclarationRef },
    14 => TypeMemberDeclaration { owner: StableDeclarationRef, selector: crate::semantic::type_store::TypeMemberSelectorV1 },
    15 => StaticDomainCaseCatalog { domain: StaticCaseDomainV1 },
    16 => StaticCaseSignature { domain: StaticCaseDomainV1, selector: ExactSelectorV1 },
    17 => StaticDefinition { owner: StableDeclarationRef },
    18 => StaticConstant { symbol: StaticConstantSymbolV1 },
    19 => ModuleStaticBindingCatalog { module: StableModuleRef },
    20 => ConstructorInitialization { owner: StableDeclarationRef },
    21 => ResolvedImport { module: StableModuleRef, local_id: u32 },
    22 => LinkedName { module: StableModuleRef, name: Box<str> },
    23 => PublicExport { module: StableModuleRef, name: Box<str> },
    24 => ModuleStaticBindings { module: StableModuleRef },
});

record!(CanonicalStaticProgramV1 {
    owner: StableDeclarationRef,
    parameter_count: u32,
    local_count: u32,
    root: OperationalOpV1,
    applications: Vec<CanonicalCheckedStaticApplicationV1>,
    kinded_parameters: Vec<u32>,
    static_constant_reads: Vec<StaticConstantSymbolV1>,
    dependencies: Vec<StaticSemanticDependencyV1>,
    result: StaticValueDomainV1,
    input_fingerprint: u64,
    module_binding_fingerprint: Option<u64>,
});

record!(StaticDefinitionRecordV1 {
    id: StableDeclarationRef,
    signature: StaticDefinitionSignatureV1,
    source: SpanV1,
    alias_info: Option<StableDeclarationRef>,
    canonical_program: CanonicalStaticProgramV1,
    diagnostics: Vec<CallableEvidenceDiagnosticV1>,
});

tagged!(StaticConstantOutcomeV1 {
    0 => Reduced { value: ArtifactStaticValueId }
});

record!(StaticConstantRecordV1 {
    symbol: StaticConstantSymbolV1,
    declaration_source: SpanV1,
    initializer_source: SpanV1,
    input_fingerprint: u64,
    dependencies: Vec<StaticSemanticDependencyV1>,
    static_constant_reads: Vec<StaticConstantSymbolV1>,
    record_key_static_constant_reads: Vec<StaticConstantSymbolV1>,
    result_type: Option<ArtifactTypeId>,
    outcome: StaticConstantOutcomeV1,
    lkg_value: Option<ArtifactStaticValueId>,
    diagnostics: Vec<CallableEvidenceDiagnosticV1>,
});

tagged!(ModuleStaticBindingPhaseV1 {
    0 => StaticConstant,
    1 => Runtime
});

record!(ModuleStaticBindingRecordV1 {
    symbol: StaticConstantSymbolV1,
    source_body: StableCallableRef,
    source_binding_ordinal: u32,
    phase: ModuleStaticBindingPhaseV1,
    declaration_source: SpanV1,
    initializer_source: Option<SpanV1>,
});

record!(ModuleStaticBindingVisibleNameV1 {
    name: Box<str>,
    symbol: StaticConstantSymbolV1,
});

record!(ModuleStaticBindingIndexV1 {
    module: StableModuleRef,
    bindings: Vec<ModuleStaticBindingRecordV1>,
    visible_names: Vec<ModuleStaticBindingVisibleNameV1>,
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

record!(SemanticWorldV3 {
    type_graph: TypeGraphV1,
    structural_contracts: StructuralPrimaryContractsV1,
    enum_requirements: Vec<EnumRequirementOwnerV1>,
    static_values: StaticValueGraphV1,
    static_definitions: Vec<StaticDefinitionRecordV1>,
    static_constants: Vec<StaticConstantRecordV1>,
    module_static_bindings: Vec<ModuleStaticBindingIndexV1>,
    witness_plans: Vec<WitnessPlanV1>,
    derived_conformance_rules: Vec<DerivedConformanceRuleRecordV1>,
    callable_signatures: Vec<CallableSignatureRecordV1>,
    callable_definitions: Vec<CallableDefinitionRecordV1>,
    callable_analyses: Vec<CallableAnalysisRecordV1>,
    trait_surfaces: Vec<TraitSurfaceRecordV1>,
    dispatch_surfaces: Vec<DeclarationSurfaceRecordV1>,
    conditional_inherent_sets: Vec<ConditionalInherentSetV1>,
    conformance_heads: Vec<ConformanceHeadRecordV1>,
    constructor_initializations: Vec<ConstructorInitializationRecordV1>,
    type_member_declarations: Vec<TypeMemberDeclarationRecordV1>,
    associated_static_domains: Vec<AssociatedStaticDomainRecordV1>,
    static_case_catalogs: Vec<StaticDomainCaseCatalogRecordV1>,
    static_case_signatures: Vec<StaticCaseSignatureRecordV1>,
    associated_surfaces: Vec<AssociatedSurfaceRecordV1>,
});

impl SemanticWorldV3 {
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        self.validate(&ValidationLimits::default())?;
        let mut out = BinaryWriter::new();
        out.write_u32(SEMANTIC_WORLD_SCHEMA_V3);
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
            if input.reader.read_u32()? != SEMANTIC_WORLD_SCHEMA_V3 {
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
    pub fn validate_with_allocation_budget(&self, limits: &ValidationLimits, transaction: &mut AllocationBudget) -> Result<(), ArtifactError> {
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
        if self.witness_plans.len() > limits.max_semantic_entries_per_family
            || self.derived_conformance_rules.len() > limits.max_semantic_entries_per_family
            || self.callable_signatures.len() > limits.max_semantic_entries_per_family
            || self.callable_definitions.len() > limits.max_semantic_entries_per_family
            || self.callable_analyses.len() > limits.max_semantic_entries_per_family
            || self.trait_surfaces.len() > limits.max_semantic_entries_per_family
            || self.dispatch_surfaces.len() > limits.max_semantic_entries_per_family
            || self.conditional_inherent_sets.len() > limits.max_semantic_entries_per_family
            || self.conformance_heads.len() > limits.max_semantic_entries_per_family
            || self.type_member_declarations.len() > limits.max_semantic_entries_per_family
            || self.associated_static_domains.len() > limits.max_semantic_entries_per_family
            || self.static_case_catalogs.len() > limits.max_semantic_entries_per_family
            || self.static_case_signatures.len() > limits.max_semantic_entries_per_family
            || self.static_definitions.len() > limits.max_semantic_entries_per_family
            || self.static_constants.len() > limits.max_semantic_entries_per_family
            || self.module_static_bindings.len() > limits.max_semantic_entries_per_family
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
        validate_static_definitions(self, limits, allocations)?;
        validate_static_constants(self, limits)?;
        if self.callable_definitions.windows(2).any(|pair| pair[0].callable >= pair[1].callable) {
            return Err(invalid("callable definition owners must be unique and sorted"));
        }
        for definition in &self.callable_definitions {
            validate_callable_definition(definition, &self.type_graph, limits)?;
            if let Ok(index) = self
                .callable_signatures
                .binary_search_by(|signature| signature.callable.cmp(&definition.callable))
                && self.callable_signatures[index] != definition.signature
            {
                return Err(invalid("callable definition signature disagrees with the canonical signature row"));
            }
        }
        if self.trait_surfaces.windows(2).any(|pair| pair[0].declaration >= pair[1].declaration) {
            return Err(invalid("trait surface owners must be unique and sorted"));
        }
        let mut trait_diagnostic_causes = std::collections::BTreeSet::new();
        for surface in &self.trait_surfaces {
            validate_trait_surface(surface, &self.callable_signatures, &self.type_graph, limits)?;
            for diagnostic in &surface.diagnostics {
                if let Some(cause) = diagnostic.root_cause {
                    trait_diagnostic_causes.insert(cause);
                }
            }
        }
        if trait_diagnostic_causes
            .iter()
            .copied()
            .enumerate()
            .any(|(expected, actual)| u32::try_from(expected).ok() != Some(actual))
        {
            return Err(invalid("trait diagnostic cause ordinals must be dense and owner-canonical"));
        }
        if self.witness_plans.windows(2).any(|pair| pair[0].impl_id >= pair[1].impl_id) {
            return Err(invalid("witness plans must be unique and sorted by implementation identity"));
        }
        for witness in &self.witness_plans {
            validate_witness_plan(witness, &self.trait_surfaces, &self.callable_signatures, &self.type_graph, limits, allocations)?;
        }
        if self
            .derived_conformance_rules
            .windows(2)
            .any(|pair| pair[0].rule.impl_id >= pair[1].rule.impl_id)
        {
            return Err(invalid("derived conformance rules must be unique and sorted by implementation identity"));
        }
        for record in &self.derived_conformance_rules {
            validate_derived_conformance_rule(record, &self.trait_surfaces, &self.callable_signatures, &self.type_graph, limits, allocations)?;
        }
        if self.dispatch_surfaces.windows(2).any(|pair| pair[0].declaration >= pair[1].declaration) {
            return Err(invalid("declaration surface owners must be unique and sorted"));
        }
        for surface in &self.dispatch_surfaces {
            validate_declaration_surface(surface, &self.callable_signatures, &self.type_graph, limits)?;
        }
        if self
            .conditional_inherent_sets
            .windows(2)
            .any(|pair| compare_inherent_targets(&pair[0].target, &pair[1].target).is_ge())
        {
            return Err(invalid("conditional inherent sets must be unique and target-sorted"));
        }
        for set in &self.conditional_inherent_sets {
            validate_conditional_inherent_set(set, &self.callable_signatures, &self.type_graph, limits)?;
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

            if owner.requirements.len() > limits.max_semantic_entries_per_family || owner.case_statuses.len() > limits.max_semantic_entries_per_family {
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
                if let Ok(index) = self
                    .callable_signatures
                    .binary_search_by(|signature| signature.callable.cmp(&requirement.requirement))
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
                        CaseRequirementStatusV1::Satisfied { implementation } | CaseRequirementStatusV1::Incompatible { implementation } => {
                            Some(implementation)
                        }
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
            for expression in &analysis.expressions {
                if let Some(selection) = &expression.conditional_dispatch {
                    self.validate_conditional_dispatch_selection(selection)?;
                }
                let Some(call) = &expression.call_specialization else {
                    continue;
                };
                if call.bindings.len() > limits.max_semantic_entries_per_family || call.bindings.windows(2).any(|pair| pair[0].parameter >= pair[1].parameter) {
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
                        (crate::semantic::type_store::ParameterOwnerV1::Callable { owner: parameter_owner }, _) => parameter_owner == &call.callable,
                        (
                            crate::semantic::type_store::ParameterOwnerV1::Declaration { owner: parameter_owner },
                            StableCallableOwnerRef::Declaration(call_owner),
                        ) => parameter_owner == call_owner,
                        (
                            crate::semantic::type_store::ParameterOwnerV1::Declaration { owner: parameter_owner },
                            StableCallableOwnerRef::Variant(call_owner),
                        ) => parameter_owner == &call_owner.owner,
                        (crate::semantic::type_store::ParameterOwnerV1::Impl { owner: parameter_owner }, StableCallableOwnerRef::Conformance(call_owner)) => {
                            parameter_owner == call_owner
                        }
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
            for binding in &analysis.bindings {
                if binding.range.start > binding.range.end {
                    return Err(invalid("callable binding range is inverted"));
                }
                if let Some(contract) = &binding.contract {
                    self.validate_denotation_type(contract.ty)?;
                    if contract.source.as_ref().is_some_and(|source| source.start > source.end) {
                        return Err(invalid("callable binding contract source range is inverted"));
                    }
                }
                if let TypeKnowledgeV1::Known { ty, provenance, .. } = &binding.current {
                    self.validate_denotation_type(*ty)?;
                    if provenance.ranges.iter().any(|range| range.start > range.end)
                        || provenance.ranges.len() > limits.max_semantic_entries_per_family
                        || provenance.descriptions.len() > limits.max_semantic_entries_per_family
                        || provenance.descriptions.iter().any(|description| description.len() > limits.max_string_bytes)
                    {
                        return Err(invalid("callable binding evidence provenance is invalid or exceeds its owner-local limit"));
                    }
                }
                match &binding.consistency {
                    CallableBindingConsistencyV1::Refuted { actual, expected, .. } => {
                        self.validate_denotation_type(*actual)?;
                        self.validate_denotation_type(*expected)?;
                    }
                    CallableBindingConsistencyV1::DynamicBoundary { reason } | CallableBindingConsistencyV1::InternalFailure { message: reason }
                        if reason.len() > limits.max_string_bytes =>
                    {
                        return Err(invalid("callable binding consistency text exceeds its string limit"));
                    }
                    CallableBindingConsistencyV1::Blocked { reason } => validate_callable_block_reason(reason, &self.type_graph, limits)?,
                    _ => {}
                }
                match &binding.authority {
                    CallableBindingAuthorityV1::BodyLocalSource { declaration_range, .. } if declaration_range.start > declaration_range.end => {
                        return Err(invalid("callable binding source declaration range is inverted"));
                    }
                    CallableBindingAuthorityV1::ModuleValue { name, .. } if name.is_empty() || name.len() > limits.max_string_bytes => {
                        return Err(invalid("module-value binding symbol name is invalid"));
                    }
                    _ => {}
                }
                if let Some(denotation) = &binding.denotation {
                    self.validate_callable_denotation(denotation, limits)?;
                }
            }
            for expression in &analysis.expressions {
                if expression.range.start > expression.range.end {
                    return Err(invalid("callable expression source range is inverted"));
                }
                for binding in [expression.binding_read, expression.binding_write].into_iter().flatten() {
                    if binding as usize >= analysis.bindings.len() {
                        return Err(invalid("callable expression references an absent binding ordinal"));
                    }
                }
                if let TypeKnowledgeV1::Known { ty, provenance, .. } = &expression.knowledge {
                    self.validate_denotation_type(*ty)?;
                    if provenance.ranges.iter().any(|range| range.start > range.end)
                        || provenance.ranges.len() > limits.max_semantic_entries_per_family
                        || provenance.descriptions.len() > limits.max_semantic_entries_per_family
                        || provenance.descriptions.iter().any(|description| description.len() > limits.max_string_bytes)
                    {
                        return Err(invalid("callable expression evidence provenance is invalid or exceeds its owner-local limit"));
                    }
                }
                if let CallableCausalInvalidityV1::One { cause } = &expression.causal_invalidity
                    && *cause as usize >= limits.max_semantic_entries_per_family
                {
                    return Err(invalid("callable expression causal cause exceeds its owner-local limit"));
                }
                match &expression.status {
                    CallableExpressionStatusV1::Invalid { cause }
                    | CallableExpressionStatusV1::Suppressed {
                        cause: CallableSuppressionCauseV1::One { cause },
                    } if *cause as usize >= limits.max_semantic_entries_per_family => {
                        return Err(invalid("callable expression cause ordinal exceeds its owner-local limit"));
                    }
                    CallableExpressionStatusV1::Blocked { reason } => validate_callable_block_reason(reason, &self.type_graph, limits)?,
                    _ => {}
                }
                if let Some(denotation) = &expression.denotation {
                    self.validate_callable_denotation(denotation, limits)?;
                }
            }
        }
        if self.callable_analyses.windows(2).any(|pair| pair[0].callable >= pair[1].callable) {
            return Err(invalid("callable analysis owners must be unique and sorted"));
        }
        validate_module_static_bindings(self, limits)?;
        validate_static_case_owners(self, limits, allocations)?;
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
                    crate::semantic::type_store::StaticDomainV1::Nominal { declaration } => self.validate_declaration_reference(declaration),
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
            SemanticDenotationV1::StaticApplication { .. } => Err(invalid("static-application denotation has no retained checked-body application row")),
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
                        if !matches!(&member.target, InvocationTargetV1::Behavioral { callable } if callable == &conditional.callable) {
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
            AssociatedMemberV1::Variant { variant } => self.structural_contracts.variant_contracts.iter().any(|contract| contract.variant == *variant),
            AssociatedMemberV1::DataConstructor { owner } => self.structural_contracts.data_contracts.iter().any(|contract| contract.owner == *owner),
            AssociatedMemberV1::DataComponent { owner, index } => self
                .structural_contracts
                .data_contracts
                .iter()
                .find(|contract| contract.owner == *owner)
                .is_some_and(|contract| (*index as usize) < contract.components.len()),
            AssociatedMemberV1::StaticDomain { declaration } => self.associated_static_domains.iter().any(|contract| contract.declaration == *declaration),
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
                    || self
                        .enum_requirements
                        .iter()
                        .any(|owner| owner.requirements.iter().any(|row| &row.requirement == callable))
                    || self
                        .trait_surfaces
                        .iter()
                        .any(|surface| surface.members.iter().any(|member| &member.callable == callable))
                    || self.dispatch_surfaces.iter().any(|surface| {
                        surface.instance_callables.iter().any(|fact| &fact.callable == callable)
                            || surface.class_callables.iter().any(|fact| &fact.callable == callable)
                    });
                if signature {
                    Ok(())
                } else {
                    Err(invalid("behavioral invocation target has no callable authority"))
                }
            }
            InvocationTargetV1::VariantConstructor { variant } => {
                if self
                    .structural_contracts
                    .variant_contracts
                    .iter()
                    .any(|row| &row.variant == variant && row.constructor.is_some())
                {
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
        if selection.side > 1
            || selection.callable.side
                != if selection.side == 0 {
                    StableDispatchSide::Instance
                } else {
                    StableDispatchSide::Class
                }
        {
            return Err(invalid("conditional dispatch selection has an invalid or inconsistent side"));
        }
        let target = match &selection.callable.owner {
            StableCallableOwnerRef::Declaration(declaration) if declaration == &selection.declaring_owner => InherentImplTargetV1::Declaration {
                declaration: declaration.clone(),
            },
            StableCallableOwnerRef::Variant(variant) if variant.owner == selection.declaring_owner => {
                InherentImplTargetV1::ExactEnumCase { variant: variant.clone() }
            }
            _ => return Err(invalid("conditional dispatch selection callable does not belong to its declaring owner")),
        };
        if self.conditional_inherent_sets.iter().any(|set| {
            set.target == target
                && set.members.iter().any(|member| {
                    member.impl_id == selection.impl_id
                        && member.callable == selection.callable
                        && member.signature.owner == selection.declaring_owner
                        && member.signature.side == selection.side
                })
        }) {
            Ok(())
        } else {
            Err(invalid("conditional dispatch selection has no serialized implementation-member authority"))
        }
    }
}

fn compare_static_case_domain(left: &StaticCaseDomainV1, right: &StaticCaseDomainV1) -> std::cmp::Ordering {
    use StaticCaseDomainV1 as Domain;
    match (left, right) {
        (Domain::Nominal { declaration: left }, Domain::Nominal { declaration: right }) => left.cmp(right),
        (Domain::Nominal { .. }, Domain::Associated { .. }) => std::cmp::Ordering::Less,
        (Domain::Associated { .. }, Domain::Nominal { .. }) => std::cmp::Ordering::Greater,
        (Domain::Associated { declaration: left }, Domain::Associated { declaration: right }) => left.cmp(right),
    }
}

fn validate_static_value_domain(domain: &StaticValueDomainV1, graph: &TypeGraphV1) -> Result<(), ArtifactError> {
    match domain {
        StaticValueDomainV1::Kind { kind } if usize::try_from(kind.0).map_or(true, |index| index >= graph.kinds.len()) => {
            Err(invalid("static definition domain references an absent kind"))
        }
        StaticValueDomainV1::ValueType { ty } if usize::try_from(ty.0).map_or(true, |index| index >= graph.types.len()) => {
            Err(invalid("static definition domain references an absent type"))
        }
        StaticValueDomainV1::Kind { .. } | StaticValueDomainV1::ValueType { .. } => Ok(()),
    }
}

fn validate_static_definition_signature(
    signature: &StaticDefinitionSignatureV1,
    owner: &StableDeclarationRef,
    graph: &TypeGraphV1,
    limits: &ValidationLimits,
) -> Result<(), ArtifactError> {
    if signature.parameters.len() > limits.max_parameters {
        return Err(invalid("static definition parameter limit exceeded"));
    }
    validate_static_value_domain(&signature.result_domain, graph)?;
    let mut kinded_count = 0usize;
    for (index, parameter) in signature.parameters.iter().enumerate() {
        if usize::try_from(parameter.index).ok() != Some(index)
            || parameter.name.is_empty()
            || parameter.name.len() > limits.max_string_bytes
            || parameter.source.start > parameter.source.end
        {
            return Err(invalid("static definition parameter identity or source range is invalid"));
        }
        validate_static_value_domain(&parameter.domain, graph)?;
        if let Some(type_parameter) = parameter.type_parameter {
            if usize::try_from(type_parameter.0).map_or(true, |position| position >= graph.parameters.len())
                || !matches!(parameter.domain, StaticValueDomainV1::Kind { .. })
            {
                return Err(invalid("static definition parameter references an invalid kinded parameter"));
            }
            if signature
                .generic_signature
                .as_ref()
                .map_or(true, |generic| generic.parameters.get(kinded_count) != Some(&type_parameter))
            {
                return Err(invalid("static definition generic signature parameter order disagrees with its signature"));
            }
            kinded_count += 1;
        }
    }
    if let Some(signature) = &signature.generic_signature {
        crate::semantic::aggregates::validate_signature(signature, limits)?;
        if signature.owner != (crate::semantic::type_store::ParameterOwnerV1::Declaration { owner: owner.clone() })
            || signature.parameters.len() != kinded_count
            || signature
                .parameter_kinds
                .iter()
                .any(|kind| usize::try_from(kind.0).map_or(true, |index| index >= graph.kinds.len()))
        {
            return Err(invalid("static definition generic signature disagrees with its kinded parameters"));
        }
    } else if kinded_count != 0 {
        return Err(invalid("kinded static definition parameters lack a generic signature"));
    }
    Ok(())
}

fn validate_static_definitions(
    world: &SemanticWorldV3,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<(), ArtifactError> {
    for pair in world.static_definitions.windows(2) {
        if pair[0].id >= pair[1].id {
            return Err(invalid("static definition owners must be unique and sorted"));
        }
    }
    for definition in &world.static_definitions {
        crate::semantic::aggregates::validate_owner(&definition.id, limits)?;
        if definition.source.start > definition.source.end
            || definition.alias_info.as_ref().is_some_and(|alias| alias != &definition.id)
            || definition.canonical_program.owner != definition.id
            || definition.canonical_program.result != definition.signature.result_domain
            || usize::try_from(definition.canonical_program.parameter_count).ok() != Some(definition.signature.parameters.len())
            || usize::try_from(definition.canonical_program.local_count).map_or(true, |count| count > limits.max_semantic_entries_per_family)
            || definition.diagnostics.len() > limits.max_semantic_entries_per_family
        {
            return Err(invalid("static definition owner fields disagree"));
        }
        validate_static_definition_signature(&definition.signature, &definition.id, &world.type_graph, limits)?;
        let program = &definition.canonical_program;
        crate::semantic::operational::validate_op(&program.root, program.parameter_count, program.local_count, &program.applications, 0, limits)?;
        if program.applications.len() > limits.max_semantic_entries_per_family
            || program.kinded_parameters.windows(2).any(|pair| pair[0] >= pair[1])
            || program.kinded_parameters.iter().any(|index| *index >= program.parameter_count)
            || program.static_constant_reads.windows(2).any(|pair| (&pair[0].module, &pair[0].name) >= (&pair[1].module, &pair[1].name))
            || program.dependencies.len() > limits.max_semantic_entries_per_family
        {
            return Err(invalid("canonical static program rows are non-canonical or exceed limits"));
        }
        let mut kinded_position = 0usize;
        let kinded_parameters_match = definition.signature.parameters.iter().filter(|parameter| parameter.type_parameter.is_some()).all(|parameter| {
            let matches = program.kinded_parameters.get(kinded_position) == Some(&parameter.index);
            kinded_position += 1;
            matches
        });
        if !kinded_parameters_match || kinded_position != program.kinded_parameters.len() {
            return Err(invalid("canonical static program kinded parameter indexes disagree with the definition signature"));
        }
        for (expected, application) in program.applications.iter().enumerate() {
            if usize::try_from(application.ordinal).ok() != Some(expected)
                || application.source.start > application.source.end
                || application.arguments.len() != application.signature.parameters.len()
            {
                return Err(invalid("canonical static application row is malformed"));
            }
            crate::semantic::aggregates::validate_owner(&application.target, limits)?;
            validate_static_definition_signature(&application.signature, &application.target, &world.type_graph, limits)?;
            let target = world.static_definitions.binary_search_by(|candidate| candidate.id.cmp(&application.target)).ok()
                .map(|index| &world.static_definitions[index])
                .ok_or_else(|| invalid("canonical static application target has no static definition owner"))?;
            if target.signature != application.signature {
                return Err(invalid("canonical static application signature disagrees with its target definition"));
            }
            for (index, argument) in application.arguments.iter().enumerate() {
                let parameter = &application.signature.parameters[index];
                if parameter.index as usize != index {
                    return Err(invalid("canonical static application parameters are not in index order"));
                }
                match argument {
                    CanonicalStaticApplicationArgumentV1::KindedTerm { parameter_index, term } => {
                        let expected_kind = match &parameter.domain {
                            StaticValueDomainV1::Kind { kind } => *kind,
                            StaticValueDomainV1::ValueType { .. } => {
                                return Err(invalid("canonical static application argument disagrees with its parameter domain"));
                            }
                        };
                        if *parameter_index as usize != index {
                            return Err(invalid("canonical static application argument parameter index is invalid"));
                        }
                        let term_index = usize::try_from(term.0).map_err(|_| invalid("canonical static application term index exceeds address space"))?;
                        let entry = world
                            .type_graph
                            .types
                            .get(term_index)
                            .ok_or_else(|| invalid("canonical static application term references an absent type"))?;
                        if entry.kind != expected_kind {
                            return Err(invalid("canonical static application term kind disagrees with its parameter"));
                        }
                    }
                    CanonicalStaticApplicationArgumentV1::KindedExpression {
                        parameter_index,
                        application_ordinal,
                    } => {
                        let expected_kind = match &parameter.domain {
                            StaticValueDomainV1::Kind { kind } => *kind,
                            StaticValueDomainV1::ValueType { .. } => {
                                return Err(invalid("canonical static application argument disagrees with its parameter domain"));
                            }
                        };
                        if *parameter_index as usize != index {
                            return Err(invalid("canonical static application argument parameter index is invalid"));
                        }
                        let child_index =
                            usize::try_from(*application_ordinal).map_err(|_| invalid("kinded-expression application ordinal exceeds address space"))?;
                        let child = program
                            .applications
                            .get(child_index)
                            .ok_or_else(|| invalid("kinded-expression application references an absent checked row"))?;
                        if child.signature.result_domain != (StaticValueDomainV1::Kind { kind: expected_kind }) {
                            return Err(invalid("kinded-expression result kind disagrees with its parameter"));
                        }
                    }
                    CanonicalStaticApplicationArgumentV1::Value { parameter_index }
                        if *parameter_index as usize == index && matches!(parameter.domain, StaticValueDomainV1::ValueType { .. }) => {}
                    CanonicalStaticApplicationArgumentV1::Value { .. } => {
                        return Err(invalid("canonical static application argument disagrees with its parameter domain"));
                    }
                }
            }
        }
        validate_static_application_graph(&program.applications, allocations)?;
        if definition.alias_info.is_some()
            && !world.structural_contracts.hierarchy_and_aliases.iter().any(|row| row.declaration == definition.id && row.aliased_type.is_some())
        {
            return Err(invalid("static definition alias projection lacks structural alias authority"));
        }
        for diagnostic in &definition.diagnostics {
            validate_witness_diagnostic(diagnostic, &world.callable_signatures, &world.type_graph, limits)?;
        }
    }
    Ok(())
}

fn validate_static_application_graph(applications: &[CanonicalCheckedStaticApplicationV1], allocations: &mut AllocationBudget) -> Result<(), ArtifactError> {
    let mut states = allocations.vector::<u8>(applications.len()).map_err(allocation_error)?;
    states.resize(applications.len(), 0);
    let mut stack = allocations.vector::<(usize, usize)>(applications.len()).map_err(allocation_error)?;

    for root in 0..applications.len() {
        if states[root] != 0 {
            continue;
        }
        states[root] = 1;
        stack.push((root, 0));
        while let Some((node, next_argument)) = stack.last().copied() {
            let arguments = &applications[node].arguments;
            if next_argument == arguments.len() {
                states[node] = 2;
                stack.pop();
                continue;
            }
            stack.last_mut().expect("the traversal stack is nonempty").1 += 1;
            let CanonicalStaticApplicationArgumentV1::KindedExpression { application_ordinal, .. } = arguments[next_argument] else {
                continue;
            };
            let child = usize::try_from(application_ordinal).map_err(|_| invalid("kinded-expression application ordinal exceeds address space"))?;
            let child_state = states
                .get(child)
                .copied()
                .ok_or_else(|| invalid("kinded-expression application references an absent checked row"))?;
            match child_state {
                0 => {
                    states[child] = 1;
                    stack.push((child, 0));
                }
                1 => return Err(invalid("canonical static application graph contains a cycle")),
                2 => {}
                _ => return Err(invalid("canonical static application graph has invalid traversal state")),
            }
        }
    }
    Ok(())
}

fn validate_static_constant_symbol(symbol: &StaticConstantSymbolV1, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    let module = &symbol.module;
    let valid_project = match &module.project {
        StableProjectRef::Builtin { namespace, version } => {
            !namespace.is_empty() && namespace.len() <= limits.max_string_bytes && !version.is_empty() && version.len() <= limits.max_string_bytes
        }
        StableProjectRef::Package { package, version, .. } => {
            !package.is_empty() && package.len() <= limits.max_string_bytes && !version.is_empty() && version.len() <= limits.max_string_bytes
        }
        StableProjectRef::SourceArtifact { logical_uri, .. } => !logical_uri.is_empty() && logical_uri.len() <= limits.max_string_bytes,
        StableProjectRef::Session { .. } => true,
    };
    if !valid_project
        || module.path.len() > limits.max_semantic_entries_per_family
        || module.path.iter().any(|part| part.is_empty() || part.len() > limits.max_string_bytes)
        || symbol.name.is_empty()
        || symbol.name.len() > limits.max_string_bytes
    {
        return Err(invalid("static constant identity is invalid or exceeds limits"));
    }
    Ok(())
}

fn validate_static_constants(world: &SemanticWorldV3, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    for pair in world.static_constants.windows(2) {
        let left = (&pair[0].symbol.module, &pair[0].symbol.name);
        let right = (&pair[1].symbol.module, &pair[1].symbol.name);
        if left >= right {
            return Err(invalid("static constant owners must be unique and sorted"));
        }
    }
    for constant in &world.static_constants {
        validate_static_constant_symbol(&constant.symbol, limits)?;
        if constant.declaration_source.start > constant.declaration_source.end
            || constant.initializer_source.start > constant.initializer_source.end
            || constant.dependencies.len() > limits.max_semantic_entries_per_family
            || constant.static_constant_reads.len() > limits.max_semantic_entries_per_family
            || constant.record_key_static_constant_reads.len() > limits.max_semantic_entries_per_family
            || constant.diagnostics.len() > limits.max_semantic_entries_per_family
        {
            return Err(invalid("static constant owner fields are malformed or exceed limits"));
        }
        if constant
            .result_type
            .is_some_and(|ty| usize::try_from(ty.0).map_or(true, |index| index >= world.type_graph.types.len()))
        {
            return Err(invalid("static constant result type references an absent type"));
        }
        let reduced = match &constant.outcome {
            StaticConstantOutcomeV1::Reduced { value } => *value,
        };
        if usize::try_from(reduced.0).map_or(true, |index| index >= world.static_values.values.len())
            || constant
                .lkg_value
                .is_some_and(|value| usize::try_from(value.0).map_or(true, |index| index >= world.static_values.values.len()))
        {
            return Err(invalid("static constant value references an absent static value"));
        }
        for symbol in constant.static_constant_reads.iter().chain(&constant.record_key_static_constant_reads) {
            validate_static_constant_symbol(symbol, limits)?;
        }
        for symbol in &constant.record_key_static_constant_reads {
            if !constant.static_constant_reads.contains(symbol) {
                return Err(invalid("record-key constant reads are absent from the initializer read set"));
            }
        }
        for dependency in &constant.dependencies {
            if let StaticSemanticDependencyV1::StaticConstant { symbol } = dependency {
                validate_static_constant_symbol(symbol, limits)?;
            }
        }
        for diagnostic in &constant.diagnostics {
            validate_witness_diagnostic(diagnostic, &world.callable_signatures, &world.type_graph, limits)?;
        }
    }
    for constant in &world.static_constants {
        for symbol in constant.static_constant_reads.iter() {
            if world
                .static_constants
                .binary_search_by(|candidate| (&candidate.symbol.module, &candidate.symbol.name).cmp(&(&symbol.module, &symbol.name)))
                .is_err()
            {
                return Err(invalid("static constant read references an absent canonical owner"));
            }
        }
    }
    Ok(())
}

fn validate_module_static_bindings(world: &SemanticWorldV3, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    for pair in world.module_static_bindings.windows(2) {
        if pair[0].module >= pair[1].module {
            return Err(invalid("module static-binding indexes must be unique and module-sorted"));
        }
    }
    for index in &world.module_static_bindings {
        if index.module.path.len() > limits.max_semantic_entries_per_family
            || index.module.path.iter().any(|part| part.is_empty() || part.len() > limits.max_string_bytes)
            || index.bindings.len() > limits.max_semantic_entries_per_family
            || index.visible_names.len() > limits.max_semantic_entries_per_family
        {
            return Err(invalid("module static-binding index exceeds limits"));
        }
        for pair in index.bindings.windows(2) {
            let left = (&pair[0].symbol.module, &pair[0].symbol.name);
            let right = (&pair[1].symbol.module, &pair[1].symbol.name);
            if left >= right {
                return Err(invalid("module static-binding symbols must be unique and sorted"));
            }
        }
        for binding in &index.bindings {
            validate_static_constant_symbol(&binding.symbol, limits)?;
            if binding.declaration_source.start > binding.declaration_source.end
                || binding.initializer_source.as_ref().is_some_and(|source| source.start > source.end)
            {
                return Err(invalid("module static-binding source range is inverted"));
            }
            let analysis_index = world
                .callable_analyses
                .binary_search_by(|analysis| analysis.callable.cmp(&binding.source_body))
                .ok()
                .map(|position| &world.callable_analyses[position])
                .ok_or_else(|| invalid("module static binding references an absent source body"))?;
            let StableCallableOwnerRef::Declaration(body_owner) = &binding.source_body.owner else {
                return Err(invalid("module static-binding source body is not a module-owned declaration body"));
            };
            if body_owner.module != binding.symbol.module || !matches!(analysis_index.status, CallableAnalysisStatusV1::Complete) {
                return Err(invalid("module static-binding body owner or status disagrees with its symbol"));
            }
            let binding_row = analysis_index
                .bindings
                .get(binding.source_binding_ordinal as usize)
                .filter(|row| row.ordinal == binding.source_binding_ordinal)
                .ok_or_else(|| invalid("module static-binding ordinal references an absent body-local binding"))?;
            if binding_row.name != binding.symbol.name.as_ref() {
                return Err(invalid("module static-binding symbol disagrees with its owner-local binding"));
            }
            match (&binding_row.authority, &binding.phase) {
                (
                    CallableBindingAuthorityV1::BodyLocalSource {
                        site_owner: CallableAnalysisSourceOwnerV1::Module { module },
                        declaration_range,
                        ..
                    },
                    ModuleStaticBindingPhaseV1::StaticConstant | ModuleStaticBindingPhaseV1::Runtime,
                ) if module == &body_owner.module && declaration_range == &binding.declaration_source => {}
                _ => return Err(invalid("module static-binding state is not owned by its module source binding")),
            }
            if matches!(binding.phase, ModuleStaticBindingPhaseV1::StaticConstant)
                && world.static_constants.binary_search_by(|constant| {
                    (&constant.symbol.module, &constant.symbol.name).cmp(&(&binding.symbol.module, &binding.symbol.name))
                }).is_err()
            {
                return Err(invalid("static-constant module binding has no reduced static-constant owner"));
            }
        }
        for pair in index.visible_names.windows(2) {
            if pair[0].name >= pair[1].name {
                return Err(invalid("module visible binding names must be unique and sorted"));
            }
        }
        for visible in &index.visible_names {
            if visible.name.is_empty()
                || visible.name.len() > limits.max_string_bytes
                || index.bindings.binary_search_by(|binding| {
                    (&binding.symbol.module, &binding.symbol.name).cmp(&(&visible.symbol.module, &visible.symbol.name))
                }).is_err()
            {
                return Err(invalid("module visible binding name references an invalid or absent symbol"));
            }
        }
    }
    Ok(())
}

fn selector_kind_order(kind: &SelectorKindV1) -> u8 {
    match kind {
        SelectorKindV1::Getter => 0,
        SelectorKindV1::Setter => 1,
        SelectorKindV1::Method => 2,
        SelectorKindV1::SubscriptGet => 3,
        SelectorKindV1::SubscriptSet => 4,
    }
}

fn compare_selector_slots(left: &[SelectorSlotV1], right: &[SelectorSlotV1]) -> std::cmp::Ordering {
    use SelectorSlotV1 as Slot;
    left.iter()
        .zip(right)
        .map(|(left, right)| match (left, right) {
            (Slot::Positional, Slot::Positional) => std::cmp::Ordering::Equal,
            (Slot::Positional, Slot::Label { .. }) => std::cmp::Ordering::Less,
            (Slot::Label { .. }, Slot::Positional) => std::cmp::Ordering::Greater,
            (Slot::Label { label: left }, Slot::Label { label: right }) => left.cmp(right),
        })
        .find(|ordering| !ordering.is_eq())
        .unwrap_or_else(|| left.len().cmp(&right.len()))
}

fn compare_exact_selectors(left: &ExactSelectorV1, right: &ExactSelectorV1) -> std::cmp::Ordering {
    use TypeMemberSelectorV1 as Base;
    let base_order = match (&left.base, &right.base) {
        (Base::Named { name: left }, Base::Named { name: right }) => left.cmp(right),
        (Base::Named { .. }, Base::Subscript) => std::cmp::Ordering::Less,
        (Base::Subscript, Base::Named { .. }) => std::cmp::Ordering::Greater,
        (Base::Subscript, Base::Subscript) => std::cmp::Ordering::Equal,
    };
    base_order
        .then_with(|| selector_kind_order(&left.kind).cmp(&selector_kind_order(&right.kind)))
        .then_with(|| compare_selector_slots(&left.slots, &right.slots))
}

fn validate_static_case_domain(world: &SemanticWorldV3, domain: &StaticCaseDomainV1) -> Result<(), ArtifactError> {
    use crate::semantic::type_store::StaticDomainV1;
    match domain {
        StaticCaseDomainV1::Nominal { declaration } => {
            if !world
                .type_graph
                .static_domains
                .iter()
                .any(|domain| matches!(domain, StaticDomainV1::Nominal { declaration: found } if found == declaration))
            {
                return Err(invalid("static-case catalog references a nominal domain absent from the type graph"));
            }
        }
        StaticCaseDomainV1::Associated { declaration } => {
            let record = world
                .associated_static_domains
                .iter()
                .find(|record| record.declaration == *declaration)
                .ok_or_else(|| invalid("static-case catalog references an absent associated static domain"))?;
            if !matches!(
                world.type_graph.static_domains.get(record.domain.0 as usize),
                Some(StaticDomainV1::Associated { declaration: found, .. }) if found == declaration
            ) {
                return Err(invalid("static-case catalog associated domain disagrees with its type-graph root"));
            }
        }
    }
    Ok(())
}

fn validate_static_case_selector(selector: &ExactSelectorV1, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    validate_selector_base(&selector.base, limits)?;
    validate_selector_slots(&selector.slots, limits)?;
    if !selector_kind_compatible(&selector.base, &selector.kind)
        || matches!(selector.kind, SelectorKindV1::Getter | SelectorKindV1::Setter) && !selector.slots.is_empty()
    {
        return Err(invalid("static-case selector has an incompatible kind or slot shape"));
    }
    Ok(())
}

fn validate_static_case_owners(world: &SemanticWorldV3, limits: &ValidationLimits, allocations: &mut AllocationBudget) -> Result<(), ArtifactError> {
    if world
        .static_case_catalogs
        .windows(2)
        .any(|pair| !compare_static_case_domain(&pair[0].domain, &pair[1].domain).is_lt())
    {
        return Err(invalid("static-case catalog domains must be unique and canonically ordered"));
    }
    if world.static_case_signatures.windows(2).any(|pair| {
        compare_static_case_domain(&pair[0].domain, &pair[1].domain)
            .then_with(|| compare_exact_selectors(&pair[0].selector, &pair[1].selector))
            .is_ge()
    }) {
        return Err(invalid("static-case signature owners must be unique and canonically ordered"));
    }
    let max_case_count = world.static_case_catalogs.iter().map(|catalog| catalog.cases.len()).max().unwrap_or(0);
    let mut selector_order = allocations.vector::<usize>(max_case_count).map_err(allocation_error)?;
    let mut signature_index = 0usize;
    for catalog in &world.static_case_catalogs {
        validate_static_case_domain(world, &catalog.domain)?;
        if catalog.cases.len() > limits.max_semantic_entries_per_family || catalog.cases.windows(2).any(|pair| pair[0].source.start >= pair[1].source.start) {
            return Err(invalid(
                "static-case catalog rows must preserve unique canonical source order and stay within limits",
            ));
        }
        selector_order.clear();
        for (case_index, case) in catalog.cases.iter().enumerate() {
            validate_static_case_selector(&case.selector, limits)?;
            if case.source.start > case.source.end {
                return Err(invalid("static-case source range is inverted"));
            }
            selector_order.push(case_index);
            match &case.shape {
                StaticCaseShapeV1::Singleton if !matches!(case.selector.kind, SelectorKindV1::Getter) || !case.selector.slots.is_empty() => {
                    return Err(invalid("singleton static case must use a slot-free getter selector"));
                }
                StaticCaseShapeV1::Constructor { slots } => {
                    validate_selector_slots(slots, limits)?;
                    if !matches!(case.selector.kind, SelectorKindV1::Method) || slots != &case.selector.slots {
                        return Err(invalid("constructor static-case shape disagrees with its method selector"));
                    }
                }
                StaticCaseShapeV1::Singleton => {}
            }
        }
        selector_order.sort_unstable_by(|left, right| compare_exact_selectors(&catalog.cases[*left].selector, &catalog.cases[*right].selector));
        if selector_order
            .windows(2)
            .any(|pair| compare_exact_selectors(&catalog.cases[pair[0]].selector, &catalog.cases[pair[1]].selector).is_eq())
        {
            return Err(invalid("static-case catalog contains a duplicate case identity"));
        }
        while let Some(signature) = world.static_case_signatures.get(signature_index) {
            match compare_static_case_domain(&signature.domain, &catalog.domain) {
                std::cmp::Ordering::Less => return Err(invalid("static-case signature has no matching catalog")),
                std::cmp::Ordering::Greater => break,
                std::cmp::Ordering::Equal => {}
            }
            validate_static_case_selector(&signature.selector, limits)?;
            if signature.source.start > signature.source.end || signature.parameter_kinds.len() > limits.max_semantic_entries_per_family {
                return Err(invalid("static-case signature source range or parameter count is invalid"));
            }
            let case_position = selector_order
                .binary_search_by(|case_index| compare_exact_selectors(&catalog.cases[*case_index].selector, &signature.selector))
                .map_err(|_| invalid("static-case signature references a case absent from its catalog"))?;
            let case = &catalog.cases[selector_order[case_position]];
            let StaticCaseShapeV1::Constructor { slots } = &case.shape else {
                return Err(invalid("singleton static case cannot own a constructor signature"));
            };
            if slots.len() != signature.parameter_kinds.len() {
                return Err(invalid("static-case constructor arity disagrees with its parameter-kind signature"));
            }
            for kind in &signature.parameter_kinds {
                if usize::try_from(kind.0).map_or(true, |index| index >= world.type_graph.kinds.len()) {
                    return Err(invalid("static-case signature references an absent kind"));
                }
            }
            signature_index += 1;
        }
    }
    if let Some(signature) = world.static_case_signatures.get(signature_index) {
        validate_static_case_domain(world, &signature.domain)?;
        return Err(invalid("static-case signature has no matching catalog"));
    }
    Ok(())
}

fn validate_callable_block_reason(
    reason: &CallableBlockReasonV1,
    type_graph: &crate::semantic::type_store::TypeGraphV1,
    limits: &ValidationLimits,
) -> Result<(), ArtifactError> {
    use crate::semantic::aggregates::TypeTermV1;

    match reason {
        CallableBlockReasonV1::UnknownType { .. }
        | CallableBlockReasonV1::RecursiveFixpoint
        | CallableBlockReasonV1::InferenceDidNotConverge
        | CallableBlockReasonV1::ReflectionBoundary
        | CallableBlockReasonV1::SuppressedDependency => Ok(()),
        CallableBlockReasonV1::UnresolvedDependency { project_source, module_path } => {
            if project_source.is_empty()
                || project_source.len() > limits.max_string_bytes
                || module_path.len() > limits.max_semantic_entries_per_family
                || module_path.iter().any(|part| part.is_empty() || part.len() > limits.max_string_bytes)
            {
                return Err(invalid("callable unresolved-dependency identity is invalid or exceeds its limits"));
            }
            Ok(())
        }
        CallableBlockReasonV1::InvalidAnnotation { diagnostic_code } if diagnostic_code.is_empty() || diagnostic_code.len() > limits.max_string_bytes => {
            Err(invalid("callable invalid-annotation diagnostic code is invalid"))
        }
        CallableBlockReasonV1::InvalidAnnotation { .. } => Ok(()),
        CallableBlockReasonV1::OpaqueNative { message } if message.len() > limits.max_string_bytes => {
            Err(invalid("callable opaque-native reason exceeds its string limit"))
        }
        CallableBlockReasonV1::OpaqueNative { .. } => Ok(()),
        CallableBlockReasonV1::BudgetExceeded { .. } => Ok(()),
        CallableBlockReasonV1::UnavailableSemanticOwner { owner } => match owner {
            CallableQueryOwnerV1::GenericConformance { subject, trait_ref } => {
                if let TypeTermV1::Canonical { ty } = subject
                    && ty.0 as usize >= type_graph.types.len()
                {
                    return Err(invalid("callable generic-conformance owner references an absent type"));
                }
                if trait_ref.arguments.iter().any(|argument| argument.0 as usize >= type_graph.types.len()) {
                    return Err(invalid("callable generic-conformance owner references an absent trait argument type"));
                }
                Ok(())
            }
            CallableQueryOwnerV1::TraitDispatchBucket { target_family, selector, .. } => {
                if let ConformanceTargetV1::ExactEnumCase { variant } = target_family
                    && !type_graph.variants.contains(variant)
                {
                    return Err(invalid("callable trait-dispatch owner references an absent exact-case identity"));
                }
                let base_too_long = matches!(
                    &selector.base,
                    crate::semantic::type_store::TypeMemberSelectorV1::Named { name } if name.len() > limits.max_string_bytes
                );
                if selector.slots.len() > limits.max_semantic_entries_per_family
                    || base_too_long
                    || selector
                        .slots
                        .iter()
                        .any(|slot| matches!(slot, SelectorSlotV1::Label { label } if label.len() > limits.max_string_bytes))
                {
                    return Err(invalid("callable trait-dispatch owner selector exceeds its limits"));
                }
                Ok(())
            }
            CallableQueryOwnerV1::CallableSignature { .. } | CallableQueryOwnerV1::FieldSignature { .. } => Ok(()),
        },
    }
}

fn associated_member_owner(member: &AssociatedMemberV1) -> &StableDeclarationRef {
    match member {
        AssociatedMemberV1::Variant { variant } => &variant.owner,
        AssociatedMemberV1::DataConstructor { owner } | AssociatedMemberV1::DataComponent { owner, .. } | AssociatedMemberV1::TypeMember { owner, .. } => owner,
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
        (
            TypeMemberSelectorV1::Named { .. },
            SelectorKindV1::Getter | SelectorKindV1::Setter | SelectorKindV1::Method
        ) | (TypeMemberSelectorV1::Subscript, SelectorKindV1::SubscriptGet | SelectorKindV1::SubscriptSet)
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
                    SelectorKindPatternV1::Exact {
                        kind: SelectorKindV1::Getter | SelectorKindV1::Setter
                    } | SelectorKindPatternV1::NamedAccessors
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
                SelectorKindPatternV1::AnyNamed => matches!(operation.kind, SelectorKindV1::Getter | SelectorKindV1::Setter | SelectorKindV1::Method),
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

fn validate_callable_definition(definition: &CallableDefinitionRecordV1, type_graph: &TypeGraphV1, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    if definition.signature.callable != definition.callable {
        return Err(invalid("callable definition key disagrees with its complete signature"));
    }
    validate_callable_signature(&definition.signature, limits)?;

    match (&definition.origin, &definition.inherent_body_context, &definition.body_provenance) {
        (CallableDefinitionOriginV1::PrimaryDeclaration, None, CallableDefinitionBodyProvenanceV1::SourceBody) => {
            if matches!(&definition.callable.owner, StableCallableOwnerRef::Conformance(_)) {
                return Err(invalid("primary callable definition has a conformance-owned callable identity"));
            }
        }
        (CallableDefinitionOriginV1::InherentImpl { impl_id }, Some(context), _) => {
            if matches!(&definition.callable.owner, StableCallableOwnerRef::Conformance(_)) {
                return Err(invalid("inherent callable definition has a conformance-owned callable identity"));
            }
            if context.source_target_head.0 as usize >= type_graph.types.len() {
                return Err(invalid("inherent callable body context references an absent source target head"));
            }
            if let Some(signature) = &context.generic_signature
                && signature.owner != (crate::semantic::type_store::ParameterOwnerV1::Impl { owner: impl_id.clone() })
            {
                return Err(invalid("inherent callable body generic signature has the wrong impl owner"));
            }
            if let Some(covering) = &context.covering {
                validate_covering_impl_substitution(covering, impl_id, &definition.signature.owner, type_graph, limits)?;
            }
        }
        (CallableDefinitionOriginV1::ConformanceWitness { impl_id }, None, CallableDefinitionBodyProvenanceV1::SourceBody) if matches!(&definition.callable.owner, StableCallableOwnerRef::Conformance(owner) if owner == impl_id) =>
            {}
        _ => return Err(invalid("callable definition origin, body provenance, and body context are inconsistent")),
    }

    if let CallableDefinitionBodyProvenanceV1::GeneratedDelegation { target_field } = &definition.body_provenance
        && (target_field.is_empty() || target_field.len() > limits.max_string_bytes)
    {
        return Err(invalid("generated delegation target field is empty or oversized"));
    }
    Ok(())
}

fn validate_covering_impl_substitution(
    covering: &CoveringImplSubstitutionV1,
    impl_id: &StableImplRef,
    declaration: &StableDeclarationRef,
    type_graph: &TypeGraphV1,
    limits: &ValidationLimits,
) -> Result<(), ArtifactError> {
    if covering.impl_to_decl.len() > limits.max_semantic_entries_per_family
        || covering.decl_to_impl.len() > limits.max_semantic_entries_per_family
        || covering.impl_to_decl.len() != covering.decl_to_impl.len()
    {
        return Err(invalid("covering impl substitution exceeds limits or is not a bijection"));
    }
    if covering.impl_to_decl.windows(2).any(|pair| pair[0].impl_parameter >= pair[1].impl_parameter)
        || covering
            .decl_to_impl
            .windows(2)
            .any(|pair| pair[0].declaration_parameter >= pair[1].declaration_parameter)
    {
        return Err(invalid("covering impl substitution rows must be strictly sorted and unique"));
    }

    for binding in &covering.impl_to_decl {
        let inverse = covering
            .decl_to_impl
            .binary_search_by_key(&binding.declaration_parameter, |entry| entry.declaration_parameter)
            .ok()
            .map(|index| &covering.decl_to_impl[index]);
        if !inverse.is_some_and(|inverse| inverse.impl_parameter == binding.impl_parameter) {
            return Err(invalid("covering impl substitution maps are not exact inverses"));
        }
        let impl_parameter = type_graph
            .parameters
            .get(binding.impl_parameter.0 as usize)
            .ok_or_else(|| invalid("covering impl substitution references an absent impl parameter"))?;
        let declaration_parameter = type_graph
            .parameters
            .get(binding.declaration_parameter.0 as usize)
            .ok_or_else(|| invalid("covering impl substitution references an absent declaration parameter"))?;
        if impl_parameter.owner != (crate::semantic::type_store::ParameterOwnerV1::Impl { owner: impl_id.clone() })
            || declaration_parameter.owner != (crate::semantic::type_store::ParameterOwnerV1::Declaration { owner: declaration.clone() })
            || impl_parameter.kind != declaration_parameter.kind
        {
            return Err(invalid("covering impl substitution parameter owner or kind is invalid"));
        }
    }
    for binding in &covering.decl_to_impl {
        let forward = covering
            .impl_to_decl
            .binary_search_by_key(&binding.impl_parameter, |entry| entry.impl_parameter)
            .ok()
            .map(|index| &covering.impl_to_decl[index]);
        if !forward.is_some_and(|forward| forward.declaration_parameter == binding.declaration_parameter) {
            return Err(invalid("covering impl substitution maps are not exact inverses"));
        }
    }
    Ok(())
}

fn validate_trait_surface(
    surface: &TraitSurfaceRecordV1,
    callable_signatures: &[CallableSignatureRecordV1],
    type_graph: &TypeGraphV1,
    limits: &ValidationLimits,
) -> Result<(), ArtifactError> {
    if surface.members.len() > limits.max_semantic_entries_per_family
        || surface.associated_types.len() > limits.max_semantic_entries_per_family
        || surface.diagnostics.len() > limits.max_semantic_entries_per_family
    {
        return Err(invalid("trait surface row budget exceeded"));
    }
    validate_trait_source(&surface.header_source, &surface.declaration)?;
    if let Some(signature) = &surface.generic_signature
        && signature.owner
            != (crate::semantic::type_store::ParameterOwnerV1::Declaration {
                owner: surface.declaration.clone(),
            })
    {
        return Err(invalid("trait header generic signature has the wrong declaration owner"));
    }

    if surface.members.windows(2).any(|pair| {
        let previous = &pair[0].requirement;
        let next = &pair[1].requirement;
        (&previous.owner, &previous.selector, previous.side) >= (&next.owner, &next.selector, next.side)
    }) {
        return Err(invalid("trait member requirements must be unique and sorted"));
    }
    for member in &surface.members {
        let expected_side = match member.callable.side {
            StableDispatchSide::Instance => 0,
            StableDispatchSide::Class => 1,
        };
        if member.requirement.owner != surface.declaration
            || member.requirement.selector.is_empty()
            || member.requirement.selector.len() > limits.max_string_bytes
            || member.requirement.side != expected_side
            || member.callable.owner != StableCallableOwnerRef::Declaration(surface.declaration.clone())
            || member.callable.selector != member.requirement.selector
            || member.signature.callable != member.callable
            || member.signature.owner != surface.declaration
            || member.signature.selector != member.requirement.selector
            || member.signature.side != expected_side
            || usize::try_from(member.source_member_index).map_or(true, |index| index >= limits.max_semantic_entries_per_family)
            || member.default_present != member.default_source.is_some()
        {
            return Err(invalid("trait member identity, signature, source ordinal, or default state is inconsistent"));
        }
        validate_callable_signature(&member.signature, limits)?;
        validate_trait_source(&member.source, &surface.declaration)?;
        if let Some(source) = &member.default_source {
            validate_trait_source(source, &surface.declaration)?;
        }
        if let Ok(index) = callable_signatures.binary_search_by(|signature| signature.callable.cmp(&member.callable))
            && callable_signatures[index] != member.signature
        {
            return Err(invalid("trait member signature disagrees with the canonical callable signature row"));
        }
    }

    if surface
        .associated_types
        .windows(2)
        .any(|pair| pair[0].requirement.index >= pair[1].requirement.index)
    {
        return Err(invalid("trait associated-type requirements must be unique and sorted"));
    }
    for (index, associated) in surface.associated_types.iter().enumerate() {
        if associated.requirement.owner != surface.declaration
            || usize::try_from(associated.requirement.index).ok() != Some(index)
            || associated.name.is_empty()
            || associated.name.len() > limits.max_string_bytes
        {
            return Err(invalid("trait associated-type identity or name is invalid"));
        }
        validate_trait_source(&associated.source, &surface.declaration)?;
        match associated.domain {
            TypeMemberDomainV1::Kind { kind } => {
                let index = usize::try_from(kind.0).map_err(|_| invalid("trait associated-type kind index exceeds address space"))?;
                if index >= type_graph.kinds.len() {
                    return Err(invalid("trait associated-type domain references an absent kind"));
                }
            }
            TypeMemberDomainV1::ValueType { ty } => {
                let index = usize::try_from(ty.0).map_err(|_| invalid("trait associated-type type index exceeds address space"))?;
                if index >= type_graph.types.len() {
                    return Err(invalid("trait associated-type domain references an absent type"));
                }
            }
        }
    }
    for diagnostic in &surface.diagnostics {
        validate_trait_surface_diagnostic(diagnostic, &surface.declaration, callable_signatures, type_graph, limits)?;
    }
    Ok(())
}

// `validate_trait_surface` establishes this key order before witness and
// derived-rule validation use the lookup.
fn find_trait_surface_member<'a>(surface: &'a TraitSurfaceRecordV1, requirement: &CallableRequirementIdentityV1) -> Option<&'a TraitSurfaceMemberV1> {
    surface
        .members
        .binary_search_by(|member| {
            (&member.requirement.owner, member.requirement.selector.as_ref(), member.requirement.side).cmp(&(
                &requirement.owner,
                requirement.selector.as_ref(),
                requirement.side,
            ))
        })
        .ok()
        .map(|index| &surface.members[index])
}

fn validate_witness_plan(
    witness: &WitnessPlanV1,
    trait_surfaces: &[TraitSurfaceRecordV1],
    callable_signatures: &[CallableSignatureRecordV1],
    type_graph: &TypeGraphV1,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<(), ArtifactError> {
    use CallableConformanceCompletenessV1 as Completeness;
    use CallableConformanceFailureV1 as Failure;
    use CallableRequirementSelectionV1 as Selection;

    let trait_surface = trait_surfaces
        .binary_search_by(|surface| surface.declaration.cmp(&witness.trait_ref_template.declaration))
        .ok()
        .map(|index| &trait_surfaces[index])
        .ok_or_else(|| invalid("witness plan references an absent trait surface"))?;
    let type_index = |ty: crate::semantic::type_store::ArtifactTypeId| -> Result<usize, ArtifactError> {
        let index = usize::try_from(ty.0).map_err(|_| invalid("witness type index exceeds address space"))?;
        if index >= type_graph.types.len() {
            return Err(invalid("witness references an absent type"));
        }
        Ok(index)
    };
    type_index(witness.target_template)?;
    for ty in &witness.trait_ref_template.arguments {
        type_index(*ty)?;
    }
    if let Some(signature) = &witness.generic_signature {
        if signature.parameters.len() > limits.max_semantic_entries_per_family
            || signature.constraints.len() > limits.max_semantic_entries_per_family
            || signature.parameters.len() != signature.parameter_modes.len()
            || signature.owner
                != (crate::semantic::type_store::ParameterOwnerV1::Impl {
                    owner: witness.impl_id.clone(),
                })
            || signature.parameters.iter().any(|parameter| {
                usize::try_from(parameter.0)
                    .ok()
                    .and_then(|index| type_graph.parameters.get(index))
                    .is_none_or(|parameter| parameter.owner != signature.owner)
            })
        {
            return Err(invalid("witness generic signature is malformed or exceeds its limits"));
        }
    }
    if witness.requirement_views.len() > limits.max_semantic_entries_per_family
        || witness.requirements.len() > limits.max_semantic_entries_per_family
        || witness.associated_type_plan.bindings.len() > limits.max_semantic_entries_per_family
        || witness.associated_type_plan.failures.len() > limits.max_semantic_entries_per_family
        || witness.associated_type_plan.diagnostics.len() > limits.max_semantic_entries_per_family
        || witness.diagnostics.len() > limits.max_semantic_entries_per_family
        || witness.conformance_evidence.len() > limits.max_semantic_entries_per_family
    {
        return Err(invalid("witness plan exceeds a semantic entry limit"));
    }
    if witness.requirement_views.windows(2).any(|pair| {
        (&pair[0].requirement.owner, &pair[0].requirement.selector, pair[0].requirement.side)
            >= (&pair[1].requirement.owner, &pair[1].requirement.selector, pair[1].requirement.side)
    }) || witness.requirements.windows(2).any(|pair| {
        (&pair[0].requirement.owner, &pair[0].requirement.selector, pair[0].requirement.side)
            >= (&pair[1].requirement.owner, &pair[1].requirement.selector, pair[1].requirement.side)
    }) {
        return Err(invalid("witness requirement views and selections must be unique and canonically ordered"));
    }

    let find_member = |requirement: &CallableRequirementIdentityV1| find_trait_surface_member(trait_surface, requirement);
    for view in &witness.requirement_views {
        let identity = &view.requirement;
        if identity.owner != trait_surface.declaration
            || identity.selector.is_empty()
            || identity.selector.len() > limits.max_string_bytes
            || identity.side > 1
            || !is_canonical_requirement_selector(&identity.selector)
        {
            return Err(invalid("witness requirement view identity is invalid"));
        }
        let member = find_member(identity).ok_or_else(|| invalid("witness requirement view is absent from the imported trait surface"))?;
        if view.callable != member.callable
            || view.signature.callable != view.callable
            || view.signature.owner != trait_surface.declaration
            || view.signature.selector != identity.selector
            || view.signature.side != identity.side
            || view.default_callable.is_some() != member.default_present
            || (view.default_callable.is_some() && view.default_callable.as_ref() != Some(&member.callable))
        {
            return Err(invalid("witness requirement view disagrees with its canonical trait member"));
        }
        validate_callable_signature(&view.signature, limits)?;
        if let Some(default) = &view.default_callable
            && callable_signatures.binary_search_by(|signature| signature.callable.cmp(default)).is_err()
        {
            return Err(invalid("witness requirement view references an absent default callable"));
        }
    }

    for entry in &witness.requirements {
        let identity = &entry.requirement;
        if identity.owner != trait_surface.declaration
            || identity.selector.is_empty()
            || identity.selector.len() > limits.max_string_bytes
            || identity.side > 1
            || !is_canonical_requirement_selector(&identity.selector)
        {
            return Err(invalid("witness requirement selection identity is invalid"));
        }
        let member = find_member(identity).ok_or_else(|| invalid("witness selection is absent from the imported trait surface"))?;
        validate_witness_selection(&entry.selection, identity, member, &witness.impl_id, callable_signatures, type_graph, limits, 0)?;
    }

    let associated_plan = &witness.associated_type_plan;
    if associated_plan.impl_id != witness.impl_id
        || associated_plan
            .bindings
            .windows(2)
            .any(|pair| (&pair[0].requirement.owner, pair[0].requirement.index) >= (&pair[1].requirement.owner, pair[1].requirement.index))
    {
        return Err(invalid("witness associated-type plan owner or binding order is invalid"));
    }
    for binding in &associated_plan.bindings {
        if binding.requirement.owner != trait_surface.declaration
            || binding.source_impl != witness.impl_id
            || binding.source.range.start > binding.source.range.end
            || usize::try_from(binding.requirement.index).map_or(true, |index| index >= trait_surface.associated_types.len())
        {
            return Err(invalid("witness associated-type binding has an invalid owner, source, or requirement"));
        }
        if let CallableAssociatedTypeBindingValueV1::Type { ty } = binding.value {
            type_index(ty)?;
        }
    }
    for failure in &associated_plan.failures {
        if failure.name.len() > limits.max_string_bytes
            || failure.reason.len() > limits.max_string_bytes
            || failure.source.range.start > failure.source.range.end
            || failure.requirement.as_ref().is_some_and(|requirement| {
                requirement.owner != trait_surface.declaration
                    || usize::try_from(requirement.index).map_or(true, |index| index >= trait_surface.associated_types.len())
            })
        {
            return Err(invalid("witness associated-type failure is malformed"));
        }
    }
    for diagnostic in associated_plan.diagnostics.iter().chain(&witness.diagnostics) {
        validate_witness_diagnostic(diagnostic, callable_signatures, type_graph, limits)?;
    }
    validate_witness_evidence_graph(witness, limits, allocations)?;

    match &witness.completeness {
        Completeness::Complete | Completeness::Cancelled => {}
        Completeness::Incomplete { failures } => {
            if failures.len() > limits.max_semantic_entries_per_family {
                return Err(invalid("witness completeness failure count exceeds its limit"));
            }
            for failure in failures {
                match failure {
                    Failure::Behavioral { requirement, reason } => {
                        if requirement.owner != trait_surface.declaration
                            || requirement.selector.is_empty()
                            || requirement.selector.len() > limits.max_string_bytes
                            || requirement.side > 1
                            || !is_canonical_requirement_selector(&requirement.selector)
                            || reason.len() > limits.max_string_bytes
                            || find_member(requirement).is_none()
                        {
                            return Err(invalid("behavioral witness failure has an invalid requirement"));
                        }
                    }
                    Failure::AssociatedType { failure } => {
                        if failure.name.len() > limits.max_string_bytes
                            || failure.reason.len() > limits.max_string_bytes
                            || failure.source.range.start > failure.source.range.end
                            || failure.requirement.as_ref().is_some_and(|requirement| {
                                requirement.owner != trait_surface.declaration
                                    || usize::try_from(requirement.index).map_or(true, |index| index >= trait_surface.associated_types.len())
                            })
                        {
                            return Err(invalid("associated-type witness failure is malformed"));
                        }
                    }
                }
            }
        }
        Completeness::Unknown { .. } => {}
        Completeness::Blocked { reason } => validate_callable_block_reason(reason, type_graph, limits)?,
        Completeness::Dynamic { reason } if reason.len() > limits.max_string_bytes => return Err(invalid("witness dynamic reason exceeds its string limit")),
        Completeness::Dynamic { .. } => {}
        Completeness::BudgetExceeded { .. } => {}
        Completeness::InternalFailure { message } if message.len() > limits.max_string_bytes => {
            return Err(invalid("witness internal-failure message exceeds its string limit"));
        }
        Completeness::InternalFailure { .. } => {}
    }

    let mut selected_index = 0usize;
    for entry in &witness.requirements {
        let callable = match &entry.selection {
            Selection::ConformanceCallable { callable } | Selection::InherentCallable { callable, .. } | Selection::TraitDefault { callable } => Some(callable),
            _ => None,
        };
        if let Some(callable) = callable {
            if witness.ordered_selected_callables.get(selected_index) != Some(callable) {
                return Err(invalid("witness selected-callable order does not match its requirement selections"));
            }
            selected_index += 1;
        }
    }
    if selected_index != witness.ordered_selected_callables.len()
        || compute_portable_witness_layout_fingerprint(&witness.ordered_selected_callables) != witness.slot_layout_fingerprint
    {
        return Err(invalid("witness selected-callable layout does not match its requirement selections"));
    }
    Ok(())
}

fn validate_derived_conformance_rule(
    record: &DerivedConformanceRuleRecordV1,
    trait_surfaces: &[TraitSurfaceRecordV1],
    callable_signatures: &[CallableSignatureRecordV1],
    type_graph: &TypeGraphV1,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<(), ArtifactError> {
    let rule = &record.rule;
    let trait_surface = trait_surfaces
        .binary_search_by(|surface| surface.declaration.cmp(&rule.trait_ref.declaration))
        .ok()
        .map(|index| &trait_surfaces[index])
        .ok_or_else(|| invalid("derived conformance rule references an absent trait surface"))?;
    let type_index = |ty: ArtifactTypeId| -> Result<usize, ArtifactError> {
        let index = usize::try_from(ty.0).map_err(|_| invalid("derived rule type index exceeds address space"))?;
        if index >= type_graph.types.len() {
            return Err(invalid("derived rule references an absent type"));
        }
        Ok(index)
    };
    if rule.source_module != rule.impl_id.module
        || rule.source.range.start > rule.source.range.end
        || rule.requirement_views.len() > limits.max_semantic_entries_per_family
        || rule.requirements.len() > limits.max_semantic_entries_per_family
        || rule.associated_type_plan.bindings.len() > limits.max_semantic_entries_per_family
        || rule.associated_type_plan.failures.len() > limits.max_semantic_entries_per_family
        || rule.associated_type_plan.diagnostics.len() > limits.max_semantic_entries_per_family
        || record.conformance_evidence.len() > limits.max_semantic_entries_per_family
    {
        return Err(invalid("derived conformance rule is malformed or exceeds its limits"));
    }
    type_index(rule.target_template)?;
    let parameter_index = usize::try_from(rule.target_parameter.0).map_err(|_| invalid("derived rule target parameter exceeds address space"))?;
    if type_graph
        .parameters
        .get(parameter_index)
        .is_none_or(|parameter| parameter.owner != crate::semantic::type_store::ParameterOwnerV1::Impl { owner: rule.impl_id.clone() })
    {
        return Err(invalid("derived rule target parameter is not owned by its implementation"));
    }
    for argument in &rule.trait_ref.arguments {
        type_index(*argument)?;
    }
    let signature = &rule.generic_signature;
    if signature.owner != (crate::semantic::type_store::ParameterOwnerV1::Impl { owner: rule.impl_id.clone() })
        || signature.parameters.len() > limits.max_semantic_entries_per_family
        || signature.constraints.len() > limits.max_semantic_entries_per_family
        || signature.parameters.len() != signature.parameter_modes.len()
        || signature.parameters.iter().any(|parameter| {
            usize::try_from(parameter.0)
                .ok()
                .and_then(|index| type_graph.parameters.get(index))
                .is_none_or(|parameter| parameter.owner != signature.owner)
        })
    {
        return Err(invalid("derived rule generic signature is malformed or exceeds its limits"));
    }

    if rule.requirement_views.windows(2).any(|pair| {
        (&pair[0].requirement.owner, &pair[0].requirement.selector, pair[0].requirement.side)
            >= (&pair[1].requirement.owner, &pair[1].requirement.selector, pair[1].requirement.side)
    }) || rule.requirements.windows(2).any(|pair| {
        (&pair[0].requirement.owner, &pair[0].requirement.selector, pair[0].requirement.side)
            >= (&pair[1].requirement.owner, &pair[1].requirement.selector, pair[1].requirement.side)
    }) {
        return Err(invalid("derived rule requirement owners must be unique and canonically ordered"));
    }
    let find_member = |requirement: &CallableRequirementIdentityV1| find_trait_surface_member(trait_surface, requirement);
    for view in &rule.requirement_views {
        let identity = &view.requirement;
        if identity.owner != trait_surface.declaration
            || identity.selector.is_empty()
            || identity.selector.len() > limits.max_string_bytes
            || identity.side > 1
            || !is_canonical_requirement_selector(&identity.selector)
        {
            return Err(invalid("derived rule requirement view identity is invalid"));
        }
        let member = find_member(identity).ok_or_else(|| invalid("derived rule requirement view is absent from its trait surface"))?;
        if view.callable != member.callable
            || view.signature.callable != view.callable
            || view.signature.owner != trait_surface.declaration
            || view.signature.selector != identity.selector
            || view.signature.side != identity.side
            || view.default_callable.is_some() != member.default_present
            || (view.default_callable.is_some() && view.default_callable.as_ref() != Some(&member.callable))
        {
            return Err(invalid("derived rule requirement view disagrees with its canonical trait member"));
        }
        validate_callable_signature(&view.signature, limits)?;
    }
    for entry in &rule.requirements {
        let identity = &entry.requirement;
        if identity.owner != trait_surface.declaration
            || identity.selector.is_empty()
            || identity.selector.len() > limits.max_string_bytes
            || identity.side > 1
            || !is_canonical_requirement_selector(&identity.selector)
        {
            return Err(invalid("derived rule requirement selection identity is invalid"));
        }
        let member = find_member(identity).ok_or_else(|| invalid("derived rule selection is absent from its trait surface"))?;
        validate_witness_selection(&entry.selection, identity, member, &rule.impl_id, callable_signatures, type_graph, limits, 0)?;
    }

    let associated = &rule.associated_type_plan;
    if associated.impl_id != rule.impl_id
        || associated
            .bindings
            .windows(2)
            .any(|pair| (&pair[0].requirement.owner, pair[0].requirement.index) >= (&pair[1].requirement.owner, pair[1].requirement.index))
    {
        return Err(invalid("derived rule associated-type plan owner or binding order is invalid"));
    }
    for binding in &associated.bindings {
        if binding.requirement.owner != trait_surface.declaration
            || binding.source_impl != rule.impl_id
            || binding.source.range.start > binding.source.range.end
            || usize::try_from(binding.requirement.index).map_or(true, |index| index >= trait_surface.associated_types.len())
        {
            return Err(invalid("derived rule associated-type binding has an invalid owner, source, or requirement"));
        }
        if let CallableAssociatedTypeBindingValueV1::Type { ty } = binding.value {
            type_index(ty)?;
        }
    }
    for failure in &associated.failures {
        if failure.name.len() > limits.max_string_bytes
            || failure.reason.len() > limits.max_string_bytes
            || failure.source.range.start > failure.source.range.end
            || failure.requirement.as_ref().is_some_and(|requirement| {
                requirement.owner != trait_surface.declaration
                    || usize::try_from(requirement.index).map_or(true, |index| index >= trait_surface.associated_types.len())
            })
        {
            return Err(invalid("derived rule associated-type failure is malformed"));
        }
    }
    for diagnostic in &associated.diagnostics {
        validate_witness_diagnostic(diagnostic, callable_signatures, type_graph, limits)?;
    }
    validate_callable_evidence_graph(&rule.requirements, &record.conformance_evidence, limits, allocations)
}

fn is_canonical_requirement_selector(selector: &str) -> bool {
    phalcom_common::selector::Selector::try_decode_exact(selector).is_ok_and(|decoded| decoded.encode() == selector)
}

fn validate_witness_selection(
    selection: &CallableRequirementSelectionV1,
    requirement: &CallableRequirementIdentityV1,
    member: &TraitSurfaceMemberV1,
    witness_impl: &StableImplRef,
    callable_signatures: &[CallableSignatureRecordV1],
    type_graph: &TypeGraphV1,
    limits: &ValidationLimits,
    depth: usize,
) -> Result<(), ArtifactError> {
    if depth > limits.max_recursion_depth {
        return Err(invalid("witness selection fallback exceeds its depth limit"));
    }
    let callable_valid = |callable: &StableCallableRef| {
        callable.selector == requirement.selector
            && callable.side == member.callable.side
            && callable_signatures.binary_search_by(|signature| signature.callable.cmp(callable)).is_ok()
    };
    match selection {
        CallableRequirementSelectionV1::ConformanceCallable { callable } => {
            if callable.owner != StableCallableOwnerRef::Conformance(witness_impl.clone()) || !callable_valid(callable) {
                return Err(invalid("conformance witness selection references a callable owned by another implementation"));
            }
        }
        CallableRequirementSelectionV1::InherentCallable { callable, .. } => {
            if !callable_valid(callable) {
                return Err(invalid("witness selection references an absent or mismatched callable"));
            }
            if let CallableRequirementSelectionV1::InherentCallable {
                conditional_impl,
                applicability,
                ..
            } = selection
                && conditional_impl
                    .as_ref()
                    .is_some_and(|impl_id| applicability.as_ref().is_some_and(|row| row.impl_id != *impl_id))
            {
                return Err(invalid("witness inherent applicability belongs to another implementation"));
            }
            if let CallableRequirementSelectionV1::InherentCallable { applicability: Some(row), .. } = selection {
                validate_witness_specialization(row, type_graph, limits)?;
            }
        }
        CallableRequirementSelectionV1::TraitDefault { callable } => {
            if !member.default_present || callable != &member.callable || !callable_valid(callable) {
                return Err(invalid("witness trait-default selection disagrees with the trait surface"));
            }
        }
        CallableRequirementSelectionV1::DataComponent { index, specialized_type, .. } => {
            if usize::try_from(specialized_type.0).map_or(true, |index| index >= type_graph.types.len())
                || *index >= limits.max_semantic_entries_per_family as u32
            {
                return Err(invalid("witness data-component selection has an invalid type or index"));
            }
        }
        CallableRequirementSelectionV1::ConditionalInherent { candidate, pending, fallback } => {
            if candidate.callable.selector != requirement.selector
                || candidate.callable.side != member.callable.side
                || candidate.signature.callable != candidate.callable
                || candidate.owner != candidate.signature.owner
                || callable_signatures
                    .binary_search_by(|signature| signature.callable.cmp(&candidate.callable))
                    .is_err()
            {
                return Err(invalid("conditional inherent witness candidate is malformed"));
            }
            validate_callable_signature(&candidate.signature, limits)?;
            if let Some(applicability) = &candidate.applicability {
                validate_witness_specialization(applicability, type_graph, limits)?;
            }
            if let Some(domain) = &candidate.conditional_domain {
                if candidate.conditional_impl.as_ref() != Some(&domain.impl_id)
                    || usize::try_from(domain.head_type.0).map_or(true, |index| index >= type_graph.types.len())
                    || domain.constraints.len() > limits.max_semantic_entries_per_family
                {
                    return Err(invalid("conditional inherent witness domain is inconsistent"));
                }
            }
            if let CallableConformanceCompletenessV1::Blocked { reason } = pending {
                validate_callable_block_reason(reason, type_graph, limits)?;
            }
            if let Some(fallback) = fallback {
                validate_witness_selection(fallback, requirement, member, witness_impl, callable_signatures, type_graph, limits, depth + 1)?;
            }
        }
    }
    Ok(())
}

fn compute_portable_witness_layout_fingerprint(callables: &[StableCallableRef]) -> [u8; 32] {
    let mut digest = [0u8; 32];
    for (index, callable) in callables.iter().enumerate() {
        for (byte_index, byte) in callable.selector.as_bytes().iter().enumerate() {
            digest[byte_index % 32] ^= byte.wrapping_add(index as u8);
        }
    }
    digest
}

fn witness_push_evidence_edge(
    nodes: &[CallableConformanceEvidenceNodeV1],
    edges: &mut Vec<(usize, usize)>,
    from: usize,
    ordinal: u32,
    expected_kind: Option<u8>,
) -> Result<(), ArtifactError> {
    let index = usize::try_from(ordinal).map_err(|_| invalid("witness evidence ordinal exceeds address space"))?;
    let node = nodes.get(index).ok_or_else(|| invalid("witness evidence reference is out of range"))?;
    if expected_kind.is_some_and(|expected| callable_evidence_kind(node) != expected) {
        return Err(invalid("witness evidence reference has the wrong node kind"));
    }
    edges.push((from, index));
    Ok(())
}

fn validate_witness_evidence_reference(nodes: &[CallableConformanceEvidenceNodeV1], ordinal: u32, expected_kind: Option<u8>) -> Result<(), ArtifactError> {
    let index = usize::try_from(ordinal).map_err(|_| invalid("witness evidence ordinal exceeds address space"))?;
    let node = nodes.get(index).ok_or_else(|| invalid("witness evidence reference is out of range"))?;
    if expected_kind.is_some_and(|expected| callable_evidence_kind(node) != expected) {
        return Err(invalid("witness evidence reference has the wrong node kind"));
    }
    Ok(())
}

fn validate_witness_specialization_references(
    nodes: &[CallableConformanceEvidenceNodeV1],
    specialization: &CallableInherentImplSpecializationV1,
) -> Result<(), ArtifactError> {
    if let Some(bindings) = &specialization.conformance_provenance {
        for binding in bindings {
            let (ordinal, expected) = match &binding.source {
                CallableGenericCallConformanceSourceV1::Exact { evidence } => (Some(*evidence), Some(0)),
                CallableGenericCallConformanceSourceV1::Forwarded { .. } => (None, None),
                CallableGenericCallConformanceSourceV1::Derived { evidence } => (Some(*evidence), Some(2)),
                CallableGenericCallConformanceSourceV1::CompilerDerived { evidence } => (Some(*evidence), Some(3)),
            };
            if let Some(ordinal) = ordinal {
                validate_witness_evidence_reference(nodes, ordinal, expected)?;
            }
        }
    }
    Ok(())
}

fn validate_witness_selection_references(
    nodes: &[CallableConformanceEvidenceNodeV1],
    selection: &CallableRequirementSelectionV1,
    depth: usize,
    limit: usize,
) -> Result<(), ArtifactError> {
    if depth > limit {
        return Err(invalid("witness selection nesting exceeds its limit"));
    }
    match selection {
        CallableRequirementSelectionV1::InherentCallable { applicability, .. } => {
            if let Some(applicability) = applicability {
                validate_witness_specialization_references(nodes, applicability)?;
            }
        }
        CallableRequirementSelectionV1::ConditionalInherent { candidate, fallback, .. } => {
            if let Some(applicability) = &candidate.applicability {
                validate_witness_specialization_references(nodes, applicability)?;
            }
            if let Some(fallback) = fallback {
                validate_witness_selection_references(nodes, fallback, depth + 1, limit)?;
            }
        }
        CallableRequirementSelectionV1::ConformanceCallable { .. }
        | CallableRequirementSelectionV1::DataComponent { .. }
        | CallableRequirementSelectionV1::TraitDefault { .. } => {}
    }
    Ok(())
}

fn collect_witness_selection_evidence_edges(
    nodes: &[CallableConformanceEvidenceNodeV1],
    edges: &mut Vec<(usize, usize)>,
    from: usize,
    selection: &CallableRequirementSelectionV1,
    depth: usize,
    limit: usize,
) -> Result<(), ArtifactError> {
    if depth > limit {
        return Err(invalid("witness selection nesting exceeds its limit"));
    }
    let mut collect_specialization = |specialization: &CallableInherentImplSpecializationV1| -> Result<(), ArtifactError> {
        if let Some(bindings) = &specialization.conformance_provenance {
            for binding in bindings {
                let (ordinal, expected) = match &binding.source {
                    CallableGenericCallConformanceSourceV1::Exact { evidence } => (Some(*evidence), Some(0)),
                    CallableGenericCallConformanceSourceV1::Forwarded { .. } => (None, None),
                    CallableGenericCallConformanceSourceV1::Derived { evidence } => (Some(*evidence), Some(2)),
                    CallableGenericCallConformanceSourceV1::CompilerDerived { evidence } => (Some(*evidence), Some(3)),
                };
                if let Some(ordinal) = ordinal {
                    witness_push_evidence_edge(nodes, edges, from, ordinal, expected)?;
                }
            }
        }
        Ok(())
    };
    match selection {
        CallableRequirementSelectionV1::InherentCallable { applicability, .. } => {
            if let Some(applicability) = applicability {
                collect_specialization(applicability)?;
            }
        }
        CallableRequirementSelectionV1::ConditionalInherent { candidate, fallback, .. } => {
            if let Some(applicability) = &candidate.applicability {
                collect_specialization(applicability)?;
            }
            if let Some(fallback) = fallback {
                collect_witness_selection_evidence_edges(nodes, edges, from, fallback, depth + 1, limit)?;
            }
        }
        CallableRequirementSelectionV1::ConformanceCallable { .. }
        | CallableRequirementSelectionV1::DataComponent { .. }
        | CallableRequirementSelectionV1::TraitDefault { .. } => {}
    }
    Ok(())
}

fn validate_witness_evidence_graph(witness: &WitnessPlanV1, limits: &ValidationLimits, allocations: &mut AllocationBudget) -> Result<(), ArtifactError> {
    validate_callable_evidence_graph(&witness.requirements, &witness.conformance_evidence, limits, allocations)
}

fn validate_callable_evidence_graph(
    requirements: &[CallableRequirementSelectionEntryV1],
    nodes: &[CallableConformanceEvidenceNodeV1],
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<(), ArtifactError> {
    let node_count = nodes.len();
    let mut edge_count = 0usize;
    for entry in requirements {
        validate_witness_selection_references(nodes, &entry.selection, 0, limits.max_recursion_depth)?;
    }
    for node in nodes {
        let additional = match node {
            CallableConformanceEvidenceNodeV1::Exact { evidence } => {
                let mut count = evidence.trait_header_conformance_proofs.as_ref().map_or(0, Vec::len);
                for entry in &evidence.requirements {
                    count = count
                        .checked_add(count_callable_requirement_selection_edges(&entry.selection, 0, limits.max_recursion_depth)?)
                        .ok_or_else(|| invalid("witness evidence edge count overflow"))?;
                }
                count
            }
            CallableConformanceEvidenceNodeV1::Derived { evidence } => {
                let origin = usize::from(matches!(&evidence.origin, CallableDerivedConformanceOriginV1::ConditionalSource { .. }));
                let nested = evidence
                    .nested
                    .iter()
                    .filter(|edge| matches!(edge, CallableNestedGenericEvidenceV1::Conformance { .. }))
                    .count();
                let rule = match &evidence.origin {
                    CallableDerivedConformanceOriginV1::GuardedRule { rule, .. } => {
                        let mut count = 0usize;
                        for entry in &rule.requirements {
                            count = count
                                .checked_add(count_callable_requirement_selection_edges(&entry.selection, 0, limits.max_recursion_depth)?)
                                .ok_or_else(|| invalid("witness evidence edge count overflow"))?;
                        }
                        count
                    }
                    CallableDerivedConformanceOriginV1::ConditionalSource { .. } => 0,
                };
                origin
                    .checked_add(nested)
                    .and_then(|count| count.checked_add(rule))
                    .ok_or_else(|| invalid("witness evidence edge count overflow"))?
            }
            CallableConformanceEvidenceNodeV1::Assumed { .. } | CallableConformanceEvidenceNodeV1::CompilerDerived { .. } => 0,
        };
        edge_count = edge_count
            .checked_add(additional)
            .ok_or_else(|| invalid("witness evidence edge count overflow"))?;
    }
    if edge_count > limits.max_semantic_entries_per_family {
        return Err(invalid("witness evidence edge count exceeds its limit"));
    }
    let mut edges = allocations.vector::<(usize, usize)>(edge_count).map_err(allocation_error)?;
    for (from, node) in nodes.iter().enumerate() {
        match node {
            CallableConformanceEvidenceNodeV1::Exact { evidence } => {
                if let Some(proofs) = &evidence.trait_header_conformance_proofs {
                    for proof in proofs {
                        witness_push_evidence_edge(nodes, &mut edges, from, proof.evidence, None)?;
                    }
                }
                for entry in &evidence.requirements {
                    collect_witness_selection_evidence_edges(nodes, &mut edges, from, &entry.selection, 0, limits.max_recursion_depth)?;
                }
            }
            CallableConformanceEvidenceNodeV1::Derived { evidence } => {
                if let CallableDerivedConformanceOriginV1::ConditionalSource { exact_source } = evidence.origin {
                    witness_push_evidence_edge(nodes, &mut edges, from, exact_source, Some(0))?;
                }
                if let CallableDerivedConformanceOriginV1::GuardedRule { rule, .. } = &evidence.origin {
                    for entry in &rule.requirements {
                        collect_witness_selection_evidence_edges(nodes, &mut edges, from, &entry.selection, 0, limits.max_recursion_depth)?;
                    }
                }
                for nested in &evidence.nested {
                    if let CallableNestedGenericEvidenceV1::Conformance { evidence } = nested {
                        witness_push_evidence_edge(nodes, &mut edges, from, *evidence, None)?;
                    }
                }
            }
            CallableConformanceEvidenceNodeV1::Assumed { .. } | CallableConformanceEvidenceNodeV1::CompilerDerived { .. } => {}
        }
    }
    if edges.len() != edge_count {
        return Err(invalid("witness evidence edge accounting mismatch"));
    }
    edges.sort_unstable();
    let mut offsets = allocations
        .vector::<usize>(node_count.checked_add(1).ok_or_else(|| invalid("witness evidence node count overflow"))?)
        .map_err(allocation_error)?;
    offsets.resize(node_count + 1, 0);
    let mut indegree = allocations.vector::<usize>(node_count).map_err(allocation_error)?;
    indegree.resize(node_count, 0);
    for &(from, to) in &edges {
        offsets[from + 1] = offsets[from + 1]
            .checked_add(1)
            .ok_or_else(|| invalid("witness evidence out-degree overflow"))?;
        indegree[to] = indegree[to].checked_add(1).ok_or_else(|| invalid("witness evidence in-degree overflow"))?;
    }
    for index in 1..offsets.len() {
        offsets[index] = offsets[index]
            .checked_add(offsets[index - 1])
            .ok_or_else(|| invalid("witness evidence adjacency overflow"))?;
    }
    let mut queue = allocations.vector::<usize>(node_count).map_err(allocation_error)?;
    let mut ready = allocations.vector::<usize>(node_count).map_err(allocation_error)?;
    for (node, degree) in indegree.iter().enumerate() {
        if *degree == 0 {
            ready.push(node);
        }
    }
    while let Some(node) = ready.pop() {
        queue.push(node);
        for &(_, target) in &edges[offsets[node]..offsets[node + 1]] {
            indegree[target] -= 1;
            if indegree[target] == 0 {
                ready.push(target);
            }
        }
    }
    if queue.len() != node_count {
        return Err(invalid("witness conformance evidence graph contains a cycle"));
    }
    Ok(())
}

fn validate_witness_specialization(
    specialization: &CallableInherentImplSpecializationV1,
    type_graph: &TypeGraphV1,
    limits: &ValidationLimits,
) -> Result<(), ArtifactError> {
    for ty in [specialization.receiver, specialization.owner_view] {
        if usize::try_from(ty.0).map_or(true, |index| index >= type_graph.types.len()) {
            return Err(invalid("inherent specialization references an absent type"));
        }
    }
    if specialization.bindings.windows(2).any(|pair| pair[0].parameter >= pair[1].parameter)
        || specialization
            .environment
            .bindings
            .windows(2)
            .any(|pair| pair[0].parameter >= pair[1].parameter)
    {
        return Err(invalid("inherent specialization bindings are not canonical"));
    }
    for binding in &specialization.bindings {
        if usize::try_from(binding.parameter.0).map_or(true, |index| index >= type_graph.parameters.len())
            || usize::try_from(binding.ty.0).map_or(true, |index| index >= type_graph.types.len())
        {
            return Err(invalid("inherent specialization binding references an absent identity"));
        }
    }
    for binding in &specialization.environment.bindings {
        if usize::try_from(binding.parameter.0).map_or(true, |index| index >= type_graph.parameters.len()) {
            return Err(invalid("inherent specialization environment references an absent parameter"));
        }
    }
    if let Some(provenance) = &specialization.conformance_provenance {
        if provenance.len() > limits.max_semantic_entries_per_family {
            return Err(invalid("inherent specialization conformance provenance exceeds its limit"));
        }
    }
    Ok(())
}

fn validate_witness_diagnostic(
    diagnostic: &CallableEvidenceDiagnosticV1,
    callable_signatures: &[CallableSignatureRecordV1],
    type_graph: &TypeGraphV1,
    limits: &ValidationLimits,
) -> Result<(), ArtifactError> {
    if diagnostic.code.is_empty()
        || diagnostic.code.len() > limits.max_string_bytes
        || diagnostic.message.len() > limits.max_string_bytes
        || diagnostic.primary.range != diagnostic.primary_range
        || diagnostic.primary_range.start > diagnostic.primary_range.end
        || diagnostic.labels.len() > limits.max_semantic_entries_per_family
        || diagnostic.notes.len() > limits.max_semantic_entries_per_family
        || diagnostic.helps.len() > limits.max_semantic_entries_per_family
        || diagnostic.guidance.len() > limits.max_semantic_entries_per_family
        || diagnostic.explanations.len() > limits.max_semantic_entries_per_family
        || diagnostic.fixes.len() > limits.max_semantic_entries_per_family
    {
        return Err(invalid("witness diagnostic is malformed or exceeds its limits"));
    }
    if diagnostic
        .notes
        .iter()
        .chain(&diagnostic.helps)
        .any(|text| text.len() > limits.max_string_bytes)
    {
        return Err(invalid("witness diagnostic text exceeds its string limit"));
    }
    for label in &diagnostic.labels {
        if label.message.len() > limits.max_string_bytes || label.source.range.start > label.source.range.end || label.range.start > label.range.end {
            return Err(invalid("witness diagnostic label is malformed"));
        }
    }
    for explanation in &diagnostic.explanations {
        if callable_signatures
            .binary_search_by(|signature| signature.callable.cmp(&explanation.callable))
            .is_err()
        {
            return Err(invalid("witness diagnostic explanation references an absent callable"));
        }
    }
    for diagnostic in [diagnostic] {
        for guidance in &diagnostic.guidance {
            match guidance {
                CallableEvidenceDiagnosticGuidanceV1::ChangeAnnotation { range, ty }
                | CallableEvidenceDiagnosticGuidanceV1::EstablishTypeEvidence { range, expected: Some(ty) } => {
                    if range.start > range.end || usize::try_from(ty.0).map_or(true, |index| index >= type_graph.types.len()) {
                        return Err(invalid("witness diagnostic guidance references an absent type or inverted range"));
                    }
                }
                CallableEvidenceDiagnosticGuidanceV1::SupplyAssignableValue { expected } => {
                    if usize::try_from(expected.0).map_or(true, |index| index >= type_graph.types.len()) {
                        return Err(invalid("witness diagnostic guidance references an absent type"));
                    }
                }
                CallableEvidenceDiagnosticGuidanceV1::UseCallableShape { callable } => {
                    if callable_signatures.binary_search_by(|signature| signature.callable.cmp(callable)).is_err() {
                        return Err(invalid("witness diagnostic guidance references an absent callable"));
                    }
                }
                CallableEvidenceDiagnosticGuidanceV1::EstablishTypeEvidence { range, expected: None } if range.start > range.end => {
                    return Err(invalid("witness diagnostic guidance range is inverted"));
                }
                CallableEvidenceDiagnosticGuidanceV1::EstablishTypeEvidence { .. } => {}
                CallableEvidenceDiagnosticGuidanceV1::ResolveGenericParameter { parameter }
                    if usize::try_from(parameter.0).map_or(true, |index| index >= type_graph.parameters.len()) =>
                {
                    return Err(invalid("witness diagnostic guidance references an absent parameter"));
                }
                CallableEvidenceDiagnosticGuidanceV1::ResolveGenericParameter { .. } => {}
            }
        }
    }
    for fix in &diagnostic.fixes {
        if fix.message.len() > limits.max_string_bytes
            || fix
                .replacement
                .as_ref()
                .is_some_and(|replacement| replacement.text.len() > limits.max_string_bytes || replacement.range.start > replacement.range.end)
        {
            return Err(invalid("witness diagnostic fix is malformed"));
        }
    }
    Ok(())
}

fn validate_trait_surface_diagnostic(
    diagnostic: &CallableEvidenceDiagnosticV1,
    declaration: &StableDeclarationRef,
    callable_signatures: &[CallableSignatureRecordV1],
    type_graph: &TypeGraphV1,
    limits: &ValidationLimits,
) -> Result<(), ArtifactError> {
    if diagnostic.code.is_empty()
        || diagnostic.code.len() > limits.max_string_bytes
        || diagnostic.message.len() > limits.max_string_bytes
        || diagnostic.notes.len() > limits.max_semantic_entries_per_family
        || diagnostic.helps.len() > limits.max_semantic_entries_per_family
        || diagnostic.labels.len() > limits.max_semantic_entries_per_family
        || diagnostic.guidance.len() > limits.max_semantic_entries_per_family
        || diagnostic.explanations.len() > limits.max_semantic_entries_per_family
        || diagnostic.fixes.len() > limits.max_semantic_entries_per_family
    {
        return Err(invalid("trait surface diagnostic exceeds limits"));
    }
    if diagnostic
        .notes
        .iter()
        .chain(&diagnostic.helps)
        .any(|text| text.len() > limits.max_string_bytes)
    {
        return Err(invalid("trait surface diagnostic text exceeds limits"));
    }
    validate_trait_source(&diagnostic.primary, declaration)?;
    if diagnostic.primary_range.start > diagnostic.primary_range.end || diagnostic.primary.range != diagnostic.primary_range {
        return Err(invalid("trait surface diagnostic primary range is inconsistent"));
    }
    for label in &diagnostic.labels {
        if label.message.len() > limits.max_string_bytes || label.source.range.start > label.source.range.end || label.range.start > label.range.end {
            return Err(invalid("trait surface diagnostic label is invalid"));
        }
    }
    for fix in &diagnostic.fixes {
        if fix.message.len() > limits.max_string_bytes
            || fix
                .replacement
                .as_ref()
                .is_some_and(|replacement| replacement.text.len() > limits.max_string_bytes || replacement.range.start > replacement.range.end)
        {
            return Err(invalid("trait surface diagnostic fix is invalid"));
        }
    }
    for guidance in &diagnostic.guidance {
        match guidance {
            CallableEvidenceDiagnosticGuidanceV1::ChangeAnnotation { range, ty }
            | CallableEvidenceDiagnosticGuidanceV1::EstablishTypeEvidence { range, expected: Some(ty) } => {
                if range.start > range.end || usize::try_from(ty.0).map_or(true, |index| index >= type_graph.types.len()) {
                    return Err(invalid("trait surface diagnostic guidance references an invalid type or range"));
                }
            }
            CallableEvidenceDiagnosticGuidanceV1::SupplyAssignableValue { expected: ty } => {
                if usize::try_from(ty.0).map_or(true, |index| index >= type_graph.types.len()) {
                    return Err(invalid("trait surface diagnostic guidance references an absent type"));
                }
            }
            CallableEvidenceDiagnosticGuidanceV1::EstablishTypeEvidence { range, expected: None } => {
                if range.start > range.end {
                    return Err(invalid("trait surface diagnostic guidance range is inverted"));
                }
            }
            CallableEvidenceDiagnosticGuidanceV1::UseCallableShape { callable } => {
                if callable_signatures.binary_search_by(|signature| signature.callable.cmp(callable)).is_err() {
                    return Err(invalid("trait surface diagnostic guidance references an absent callable signature"));
                }
            }
            CallableEvidenceDiagnosticGuidanceV1::ResolveGenericParameter { parameter } => {
                if usize::try_from(parameter.0).map_or(true, |index| index >= type_graph.parameters.len()) {
                    return Err(invalid("trait surface diagnostic guidance references an absent parameter"));
                }
            }
        }
    }
    for explanation in &diagnostic.explanations {
        if callable_signatures
            .binary_search_by(|signature| signature.callable.cmp(&explanation.callable))
            .is_err()
        {
            return Err(invalid("trait surface diagnostic explanation references an absent callable signature"));
        }
    }
    Ok(())
}

fn validate_declaration_surface(
    surface: &DeclarationSurfaceRecordV1,
    callable_signatures: &[CallableSignatureRecordV1],
    type_graph: &TypeGraphV1,
    limits: &ValidationLimits,
) -> Result<(), ArtifactError> {
    if surface.instance_callables.len() > limits.max_semantic_entries_per_family
        || surface.class_callables.len() > limits.max_semantic_entries_per_family
        || surface.fields.len() > limits.max_semantic_entries_per_family
    {
        return Err(invalid("declaration surface row budget exceeded"));
    }
    for (side, callables) in [(0, &surface.instance_callables), (1, &surface.class_callables)] {
        if callables.windows(2).any(|pair| pair[0].callable >= pair[1].callable) {
            return Err(invalid("declaration surface callable rows must be unique and sorted"));
        }
        for fact in callables {
            if fact.callable.owner != StableCallableOwnerRef::Declaration(surface.declaration.clone())
                || fact.callable.side != if side == 0 { StableDispatchSide::Instance } else { StableDispatchSide::Class }
                || callable_signatures
                    .binary_search_by(|signature| signature.callable.cmp(&fact.callable))
                    .is_err()
            {
                return Err(invalid("declaration surface callable identity has no matching canonical signature"));
            }
            validate_dispatch_knowledge(&fact.knowledge, type_graph, limits)?;
        }
    }
    if surface
        .fields
        .windows(2)
        .any(|pair| (pair[0].side, &pair[0].name) >= (pair[1].side, &pair[1].name))
    {
        return Err(invalid("declaration surface fields must be unique and ordered by side and name"));
    }
    for field in &surface.fields {
        if field.owner != surface.declaration || field.name.is_empty() || field.name.len() > limits.max_string_bytes || field.side > 1 {
            return Err(invalid("declaration surface field identity is invalid"));
        }
        match (&field.declared_type, field.mutable) {
            (Some(declared_type), Some(_)) => validate_declared_type_fact(declared_type, type_graph)?,
            (None, None) if field.source.is_none() => {}
            _ => return Err(invalid("declaration field surface has incomplete optional signature authority")),
        }
        validate_dispatch_knowledge(&field.knowledge, type_graph, limits)?;
        if field.source.as_ref().is_some_and(|source| source.start > source.end) {
            return Err(invalid("declaration surface field source range is inverted"));
        }
    }
    Ok(())
}

fn validate_dispatch_knowledge(knowledge: &TypeKnowledgeV1, type_graph: &TypeGraphV1, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    match knowledge {
        TypeKnowledgeV1::Known { ty, provenance, .. } => {
            validate_type_id(*ty, type_graph)?;
            if provenance.ranges.len() > limits.max_semantic_entries_per_family
                || provenance.descriptions.len() > limits.max_semantic_entries_per_family
                || provenance.ranges.iter().any(|range| range.start > range.end)
                || provenance.descriptions.iter().any(|description| description.len() > limits.max_string_bytes)
            {
                return Err(invalid("declaration surface evidence provenance is invalid"));
            }
        }
        TypeKnowledgeV1::Unknown {
            reason: UnknownReasonV1::UnresolvedName { name },
        } if name.len() > limits.max_string_bytes => {
            return Err(invalid("declaration surface unresolved-name reason exceeds its string limit"));
        }
        TypeKnowledgeV1::Unknown { .. } | TypeKnowledgeV1::Dynamic { .. } => {}
    }
    Ok(())
}

fn validate_declared_type_fact(fact: &DeclaredTypeFactV1, type_graph: &TypeGraphV1) -> Result<(), ArtifactError> {
    if let crate::semantic::aggregates::DeclaredTypeStateV1::Known { term } = &fact.state {
        validate_type_term(term, type_graph)?;
    }
    Ok(())
}

fn validate_type_term(term: &TypeTermV1, type_graph: &TypeGraphV1) -> Result<(), ArtifactError> {
    if let TypeTermV1::Canonical { ty } = term {
        validate_type_id(*ty, type_graph)?;
    }
    Ok(())
}

fn validate_type_id(ty: ArtifactTypeId, type_graph: &TypeGraphV1) -> Result<(), ArtifactError> {
    if usize::try_from(ty.0).map_or(true, |index| index >= type_graph.types.len()) {
        return Err(invalid("semantic fact references an absent canonical type"));
    }
    Ok(())
}

fn validate_inherent_target(target: &InherentImplTargetV1) -> Result<(), ArtifactError> {
    match target {
        InherentImplTargetV1::Declaration { .. } | InherentImplTargetV1::ExactEnumCase { .. } => Ok(()),
    }
}

fn compare_inherent_targets(left: &InherentImplTargetV1, right: &InherentImplTargetV1) -> std::cmp::Ordering {
    use InherentImplTargetV1 as Target;
    match (left, right) {
        (Target::Declaration { declaration: left }, Target::Declaration { declaration: right }) => left.cmp(right),
        (Target::ExactEnumCase { variant: left }, Target::ExactEnumCase { variant: right }) => left.cmp(right),
        (Target::Declaration { .. }, Target::ExactEnumCase { .. }) => std::cmp::Ordering::Less,
        (Target::ExactEnumCase { .. }, Target::Declaration { .. }) => std::cmp::Ordering::Greater,
    }
}

fn validate_conditional_inherent_set(
    set: &ConditionalInherentSetV1,
    callable_signatures: &[CallableSignatureRecordV1],
    type_graph: &TypeGraphV1,
    limits: &ValidationLimits,
) -> Result<(), ArtifactError> {
    if set.domains.len() > limits.max_semantic_entries_per_family || set.members.len() > limits.max_semantic_entries_per_family {
        return Err(invalid("conditional inherent set row budget exceeded"));
    }
    validate_inherent_target(&set.target)?;
    if set.domains.windows(2).any(|pair| pair[0].impl_id >= pair[1].impl_id) {
        return Err(invalid("conditional inherent domains must be unique and impl-sorted"));
    }
    for domain in &set.domains {
        if domain.target != set.target
            || usize::try_from(domain.head_type.0).map_or(true, |index| index >= type_graph.types.len())
            || domain.constraints.len() > limits.max_semantic_entries_per_family
        {
            return Err(invalid("conditional inherent domain identity, head, or constraints are invalid"));
        }
        if let Some(signature) = &domain.generic_signature {
            if signature.owner != (crate::semantic::type_store::ParameterOwnerV1::Impl { owner: domain.impl_id.clone() }) {
                return Err(invalid("conditional inherent domain generic signature has the wrong impl owner"));
            }
            if signature.parameters.iter().any(|parameter| {
                usize::try_from(parameter.0)
                    .ok()
                    .and_then(|index| type_graph.parameters.get(index))
                    .is_none_or(|parameter| parameter.owner != signature.owner)
            }) {
                return Err(invalid("conditional inherent domain generic signature references an invalid impl parameter"));
            }
        }
    }
    for (member_order, member) in set.members.iter().enumerate() {
        let domain_index = set.domains.binary_search_by(|domain| domain.impl_id.cmp(&member.impl_id));
        let Ok(domain_index) = domain_index else {
            return Err(invalid("conditional inherent member references an absent impl domain"));
        };
        if u32::try_from(member_order).ok() != Some(member.member_order)
            || set.domains[domain_index].target != set.target
            || member.signature.callable != member.callable
            || member.signature.owner
                != match &set.target {
                    InherentImplTargetV1::Declaration { declaration } => declaration.clone(),
                    InherentImplTargetV1::ExactEnumCase { variant } => variant.owner.clone(),
                }
            || member.signature.side
                != match member.callable.side {
                    StableDispatchSide::Instance => 0,
                    StableDispatchSide::Class => 1,
                }
            || member.source_member_index as usize >= limits.max_semantic_entries_per_family
        {
            return Err(invalid("conditional inherent member owner, signature, or source ordinal is invalid"));
        }
        let callable_owner_matches_target = match (&set.target, &member.callable.owner) {
            (InherentImplTargetV1::Declaration { declaration }, StableCallableOwnerRef::Declaration(owner)) => declaration == owner,
            (InherentImplTargetV1::ExactEnumCase { variant }, StableCallableOwnerRef::Variant(owner)) => variant == owner,
            _ => false,
        };
        if !callable_owner_matches_target
            || callable_signatures
                .binary_search_by(|signature| signature.callable.cmp(&member.callable))
                .is_err()
        {
            return Err(invalid("conditional inherent member callable owner has no canonical signature"));
        }
        validate_callable_signature(&member.signature, limits)?;
    }
    Ok(())
}

fn validate_trait_source(source: &EnumRequirementSourceV1, declaration: &StableDeclarationRef) -> Result<(), ArtifactError> {
    if source.module != declaration.module || source.range.start > source.range.end {
        return Err(invalid("trait surface source span is invalid"));
    }
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
        TypeExprSpecV1::Universe { owner } if owner.is_empty() || owner.len() > limits.max_string_bytes => {
            Err(invalid("native universe type owner is invalid"))
        }
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
                || tuple
                    .labeled
                    .iter()
                    .any(|parameter| parameter.label.is_empty() || parameter.label.len() > limits.max_string_bytes)
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

    fn has_flow_node(&self, ordinal: u32) -> bool {
        (ordinal as usize) < self.record.flow_graph.nodes.len()
    }

    fn validate(&self, limits: &ValidationLimits, allocations: &mut AllocationBudget) -> Result<(), ArtifactError> {
        let record = self.record;
        if record.expressions.len() > limits.max_semantic_entries_per_family || record.bindings.len() > limits.max_semantic_entries_per_family {
            return Err(invalid("callable expression or binding rows exceed their owner-local limit"));
        }
        validate_callable_conformance_evidence_graph(record, limits, allocations)?;
        if record.expressions.iter().enumerate().any(|(index, row)| row.ordinal as usize != index)
            || record.bindings.iter().enumerate().any(|(index, row)| row.ordinal as usize != index)
        {
            return Err(invalid("callable expression or binding rows are not dense and ordinal-sorted"));
        }
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

        let nested_rows = [
            record.expressions.len(),
            record.bindings.len(),
            record.flow_graph.nodes.len(),
            record.flow_graph.edges.len(),
            record.flow_graph.exits.len(),
        ]
        .into_iter()
        .try_fold(0usize, usize::checked_add)
        .ok_or_else(|| invalid("callable nested row count overflow"))?;
        let specialization_bindings = record
            .expressions
            .iter()
            .filter_map(|expression| expression.call_specialization.as_ref())
            .try_fold(0usize, |total, call| total.checked_add(call.bindings.len()))
            .ok_or_else(|| invalid("callable specialization binding count overflow"))?;
        let specialization_conformances = record
            .expressions
            .iter()
            .filter_map(|expression| expression.call_specialization.as_ref())
            .try_fold(0usize, |total, call| total.checked_add(call.conformances.len()))
            .ok_or_else(|| invalid("callable specialization conformance count overflow"))?;
        let nested_rows = nested_rows
            .checked_add(specialization_bindings)
            .and_then(|total| total.checked_add(specialization_conformances))
            .and_then(|total| total.checked_add(record.conformance_evidence.len()))
            .ok_or_else(|| invalid("callable nested row count overflow"))?;
        if nested_rows > limits.max_semantic_entries_per_family {
            return Err(invalid("callable nested row budget exceeded"));
        }
        Ok(())
    }
}

fn callable_evidence_kind(node: &CallableConformanceEvidenceNodeV1) -> u8 {
    match node {
        CallableConformanceEvidenceNodeV1::Exact { .. } => 0,
        CallableConformanceEvidenceNodeV1::Assumed { .. } => 1,
        CallableConformanceEvidenceNodeV1::Derived { .. } => 2,
        CallableConformanceEvidenceNodeV1::CompilerDerived { .. } => 3,
    }
}

fn validate_callable_evidence_reference(record: &CallableAnalysisRecordV1, ordinal: u32, expected_kind: Option<u8>) -> Result<usize, ArtifactError> {
    let index = usize::try_from(ordinal).map_err(|_| invalid("callable conformance evidence ordinal exceeds address space"))?;
    let node = record
        .conformance_evidence
        .get(index)
        .ok_or_else(|| invalid("callable conformance evidence reference is out of range"))?;
    if expected_kind.is_some_and(|expected| callable_evidence_kind(node) != expected) {
        return Err(invalid("callable conformance evidence reference has the wrong node kind"));
    }
    Ok(index)
}

fn count_callable_evidence_binding_edges(binding: &CallableGenericCallConformanceBindingV1) -> usize {
    usize::from(matches!(
        &binding.source,
        CallableGenericCallConformanceSourceV1::Exact { .. }
            | CallableGenericCallConformanceSourceV1::Derived { .. }
            | CallableGenericCallConformanceSourceV1::CompilerDerived { .. }
    ))
}

fn count_callable_specialization_edges(specialization: &CallableInherentImplSpecializationV1) -> Result<usize, ArtifactError> {
    let mut count = 0usize;
    if let Some(bindings) = &specialization.conformance_provenance {
        for binding in bindings {
            count = count
                .checked_add(count_callable_evidence_binding_edges(binding))
                .ok_or_else(|| invalid("callable evidence edge count overflow"))?;
        }
    }
    Ok(count)
}

fn count_callable_requirement_selection_edges(selection: &CallableRequirementSelectionV1, depth: usize, limit: usize) -> Result<usize, ArtifactError> {
    if depth > limit {
        return Err(invalid("callable requirement selection nesting limit exceeded"));
    }
    match selection {
        CallableRequirementSelectionV1::ConformanceCallable { .. }
        | CallableRequirementSelectionV1::DataComponent { .. }
        | CallableRequirementSelectionV1::TraitDefault { .. } => Ok(0),
        CallableRequirementSelectionV1::InherentCallable { applicability, .. } => applicability
            .as_ref()
            .map(count_callable_specialization_edges)
            .transpose()
            .map(|count| count.unwrap_or(0)),
        CallableRequirementSelectionV1::ConditionalInherent { candidate, fallback, .. } => {
            let own = candidate
                .applicability
                .as_ref()
                .map(count_callable_specialization_edges)
                .transpose()?
                .unwrap_or(0);
            let fallback = fallback
                .as_deref()
                .map(|nested| count_callable_requirement_selection_edges(nested, depth + 1, limit))
                .transpose()?
                .unwrap_or(0);
            own.checked_add(fallback).ok_or_else(|| invalid("callable evidence edge count overflow"))
        }
    }
}

fn count_callable_evidence_edges(record: &CallableAnalysisRecordV1, limit: usize) -> Result<usize, ArtifactError> {
    let mut count = 0usize;
    for node in &record.conformance_evidence {
        let edges = match node {
            CallableConformanceEvidenceNodeV1::Exact { evidence } => {
                let proofs = evidence.trait_header_conformance_proofs.as_ref().map_or(0, Vec::len);
                let mut requirement_edges = 0usize;
                for entry in &evidence.requirements {
                    requirement_edges = requirement_edges
                        .checked_add(count_callable_requirement_selection_edges(&entry.selection, 0, limit)?)
                        .ok_or_else(|| invalid("callable evidence edge count overflow"))?;
                }
                proofs.checked_add(requirement_edges)
            }
            CallableConformanceEvidenceNodeV1::Derived { evidence } => {
                let origin = usize::from(matches!(&evidence.origin, CallableDerivedConformanceOriginV1::ConditionalSource { .. }));
                let nested = evidence
                    .nested
                    .iter()
                    .filter(|item| matches!(item, CallableNestedGenericEvidenceV1::Conformance { .. }))
                    .count();
                let rule_requirements = match &evidence.origin {
                    CallableDerivedConformanceOriginV1::GuardedRule { rule, .. } => {
                        let mut count = 0usize;
                        for entry in &rule.requirements {
                            count = count
                                .checked_add(count_callable_requirement_selection_edges(&entry.selection, 0, limit)?)
                                .ok_or_else(|| invalid("callable evidence edge count overflow"))?;
                        }
                        count
                    }
                    CallableDerivedConformanceOriginV1::ConditionalSource { .. } => 0,
                };
                origin.checked_add(nested).and_then(|value| value.checked_add(rule_requirements))
            }
            CallableConformanceEvidenceNodeV1::Assumed { .. } | CallableConformanceEvidenceNodeV1::CompilerDerived { .. } => Some(0),
        }
        .ok_or_else(|| invalid("callable evidence edge count overflow"))?;
        count = count.checked_add(edges).ok_or_else(|| invalid("callable evidence edge count overflow"))?;
    }
    Ok(count)
}

fn push_callable_evidence_edge(
    record: &CallableAnalysisRecordV1,
    edges: &mut Vec<(usize, usize)>,
    from: usize,
    ordinal: u32,
    expected_kind: Option<u8>,
) -> Result<(), ArtifactError> {
    let to = validate_callable_evidence_reference(record, ordinal, expected_kind)?;
    edges.push((from, to));
    Ok(())
}

fn collect_callable_binding_edges(
    record: &CallableAnalysisRecordV1,
    edges: &mut Vec<(usize, usize)>,
    from: usize,
    binding: &CallableGenericCallConformanceBindingV1,
) -> Result<(), ArtifactError> {
    use CallableGenericCallConformanceSourceV1 as Source;
    let (ordinal, expected) = match &binding.source {
        Source::Exact { evidence } => (Some(*evidence), Some(0)),
        Source::Forwarded { .. } => (None, None),
        Source::Derived { evidence } => (Some(*evidence), Some(2)),
        Source::CompilerDerived { evidence } => (Some(*evidence), Some(3)),
    };
    if let Some(ordinal) = ordinal {
        push_callable_evidence_edge(record, edges, from, ordinal, expected)?;
    }
    Ok(())
}

fn collect_callable_specialization_edges(
    record: &CallableAnalysisRecordV1,
    edges: &mut Vec<(usize, usize)>,
    from: usize,
    specialization: &CallableInherentImplSpecializationV1,
) -> Result<(), ArtifactError> {
    if let Some(bindings) = &specialization.conformance_provenance {
        for binding in bindings {
            collect_callable_binding_edges(record, edges, from, binding)?;
        }
    }
    Ok(())
}

fn collect_callable_requirement_selection_edges(
    record: &CallableAnalysisRecordV1,
    edges: &mut Vec<(usize, usize)>,
    from: usize,
    selection: &CallableRequirementSelectionV1,
    depth: usize,
    limit: usize,
) -> Result<(), ArtifactError> {
    if depth > limit {
        return Err(invalid("callable requirement selection nesting limit exceeded"));
    }
    match selection {
        CallableRequirementSelectionV1::InherentCallable { applicability, .. } => {
            if let Some(applicability) = applicability {
                collect_callable_specialization_edges(record, edges, from, applicability)?;
            }
        }
        CallableRequirementSelectionV1::ConditionalInherent { candidate, fallback, .. } => {
            if let Some(applicability) = &candidate.applicability {
                collect_callable_specialization_edges(record, edges, from, applicability)?;
            }
            if let Some(fallback) = fallback {
                collect_callable_requirement_selection_edges(record, edges, from, fallback, depth + 1, limit)?;
            }
        }
        CallableRequirementSelectionV1::ConformanceCallable { .. }
        | CallableRequirementSelectionV1::DataComponent { .. }
        | CallableRequirementSelectionV1::TraitDefault { .. } => {}
    }
    Ok(())
}

fn collect_callable_evidence_edges(record: &CallableAnalysisRecordV1, edges: &mut Vec<(usize, usize)>, limit: usize) -> Result<(), ArtifactError> {
    for (from, node) in record.conformance_evidence.iter().enumerate() {
        match node {
            CallableConformanceEvidenceNodeV1::Exact { evidence } => {
                if let Some(proofs) = &evidence.trait_header_conformance_proofs {
                    for proof in proofs {
                        push_callable_evidence_edge(record, edges, from, proof.evidence, None)?;
                    }
                }
                for requirement in &evidence.requirements {
                    collect_callable_requirement_selection_edges(record, edges, from, &requirement.selection, 0, limit)?;
                }
            }
            CallableConformanceEvidenceNodeV1::Assumed { .. } | CallableConformanceEvidenceNodeV1::CompilerDerived { .. } => {}
            CallableConformanceEvidenceNodeV1::Derived { evidence } => {
                match &evidence.origin {
                    CallableDerivedConformanceOriginV1::ConditionalSource { exact_source } => {
                        push_callable_evidence_edge(record, edges, from, *exact_source, Some(0))?;
                    }
                    CallableDerivedConformanceOriginV1::GuardedRule { rule, .. } => {
                        for requirement in &rule.requirements {
                            collect_callable_requirement_selection_edges(record, edges, from, &requirement.selection, 0, limit)?;
                        }
                    }
                }
                for nested in &evidence.nested {
                    if let CallableNestedGenericEvidenceV1::Conformance { evidence } = nested {
                        push_callable_evidence_edge(record, edges, from, *evidence, None)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_callable_conformance_evidence_graph(
    record: &CallableAnalysisRecordV1,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<(), ArtifactError> {
    let node_count = record.conformance_evidence.len();
    if node_count > limits.max_semantic_entries_per_family {
        return Err(invalid("callable conformance evidence node count exceeds its owner-local limit"));
    }
    for expression in &record.expressions {
        let Some(specialization) = &expression.call_specialization else { continue };
        for binding in &specialization.conformances {
            use CallableGenericCallConformanceSourceV1 as Source;
            let expected = match &binding.source {
                Source::Exact { evidence } => Some((*evidence, 0)),
                Source::Forwarded { .. } => None,
                Source::Derived { evidence } => Some((*evidence, 2)),
                Source::CompilerDerived { evidence } => Some((*evidence, 3)),
            };
            if let Some((ordinal, kind)) = expected {
                validate_callable_evidence_reference(record, ordinal, Some(kind))?;
            }
        }
    }
    let edge_count = count_callable_evidence_edges(record, limits.max_recursion_depth)?;
    if edge_count > limits.max_semantic_entries_per_family {
        return Err(invalid("callable conformance evidence edge count exceeds its owner-local limit"));
    }
    let mut edges = allocations.vector::<(usize, usize)>(edge_count).map_err(allocation_error)?;
    collect_callable_evidence_edges(record, &mut edges, limits.max_recursion_depth)?;
    if edges.len() != edge_count {
        return Err(invalid("callable conformance evidence edge accounting mismatch"));
    }
    edges.sort_unstable();

    let mut offsets = allocations
        .vector::<usize>(node_count.checked_add(1).ok_or_else(|| invalid("callable evidence node count overflow"))?)
        .map_err(allocation_error)?;
    offsets.resize(node_count + 1, 0);
    let mut indegree = allocations.vector::<usize>(node_count).map_err(allocation_error)?;
    indegree.resize(node_count, 0);
    for &(from, to) in &edges {
        offsets[from + 1] = offsets[from + 1]
            .checked_add(1)
            .ok_or_else(|| invalid("callable evidence out-degree overflow"))?;
        indegree[to] = indegree[to].checked_add(1).ok_or_else(|| invalid("callable evidence in-degree overflow"))?;
    }
    for index in 1..offsets.len() {
        offsets[index] = offsets[index]
            .checked_add(offsets[index - 1])
            .ok_or_else(|| invalid("callable evidence adjacency overflow"))?;
    }
    let mut queue = allocations.vector::<usize>(node_count).map_err(allocation_error)?;
    let mut ready = allocations.vector::<usize>(node_count).map_err(allocation_error)?;
    for (node, degree) in indegree.iter().enumerate() {
        if *degree == 0 {
            ready.push(node);
        }
    }
    while let Some(node) = ready.pop() {
        queue.push(node);
        for &(_, target) in &edges[offsets[node]..offsets[node + 1]] {
            indegree[target] -= 1;
            if indegree[target] == 0 {
                ready.push(target);
            }
        }
    }
    if queue.len() != node_count {
        return Err(invalid("callable conformance evidence graph contains a cycle"));
    }
    Ok(())
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
