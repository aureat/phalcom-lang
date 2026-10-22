//! Portable forms of the typed executable side tables attached to chunks.
//! Runtime pool and overlay coordinates are deliberately absent.

use crate::executable::constant::{SelectorPatternV1, StableClassRefV1};
use crate::module_world::{Decode, SpanV1, Wire, record, tagged};
use crate::{ArtifactError, BinaryWriter, ValidationLimits};
use phalcom_type_meta::identity::{StableCallableRef, StableDeclarationRef, StableDispatchSide, StableImplRef, StableVariantRef};

macro_rules! id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name(pub u32);
        impl Wire for $name {
            fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
                self.0.write(out)
            }
            fn read(input: &mut crate::module_world::Decode<'_>) -> Result<Self, ArtifactError> {
                Ok(Self(u32::read(input)?))
            }
        }
    };
}
id!(PortableTypeRecipeId);
id!(PortableKindRecipeId);
id!(PortableCallEnvironmentId);

tagged!(PortableTypeParameterOwnerV1 {
    0 => Declaration { owner: StableDeclarationRef },
    1 => Callable { owner: StableCallableRef },
    2 => Implementation { owner: StableImplRef },
});
record!(PortableTypeParameterRefV1 {
    owner: PortableTypeParameterOwnerV1,
    index: u32,
});

tagged!(PortableTypeRecipeV1 {
    0 => Canonical { digest: [u8; 32] },
    1 => Parameter { parameter: PortableTypeParameterRefV1 },
    2 => Applied { origin: PortableTypeRecipeId, arguments: Vec<PortableTypeRecipeId> },
    3 => Tuple { elements: Vec<PortableTypeRecipeId> },
    4 => Record { labels: Vec<String>, values: Vec<PortableTypeRecipeId> },
    5 => Pack { positionals: Vec<PortableTypeRecipeId>, labels: Vec<String>, labeled: Vec<PortableTypeRecipeId> },
    6 => Union { members: Vec<PortableTypeRecipeId> },
    7 => Dynamic,
    8 => Nominal { class: StableClassRefV1 },
    9 => TupleLabeled { elements: Vec<PortableTupleTypeElementV1> },
    10 => RecordKeys { fields: Vec<PortableRecordTypeFieldV1> },
    11 => Callable { parameters: Vec<PortableCallableTypeParameterV1>, return_type: PortableTypeRecipeId },
    12 => TypeLambda { parameters: Vec<PortableTypeLambdaParameterV1>, body: PortableTypeRecipeId },
    13 => Special { name: String },
    14 => SelfType { class: Option<StableClassRefV1> },
    15 => AppliedStatic { origin: PortableTypeRecipeId, arguments: Vec<PortableStaticTypeArgumentV1> },
});

tagged!(PortableKindRecipeV1 {
    0 => Canonical { digest: [u8; 32] },
    1 => Type,
    2 => Arrow { parameters: Vec<PortableKindRecipeId>, result: PortableKindRecipeId },
});
record!(PortableTypeLambdaParameterV1 {
    name: String,
    kind: PortableKindRecipeId,
});
record!(PortableTupleTypeElementV1 {
    label: Option<String>,
    ty: PortableTypeRecipeId,
});
record!(PortableCallableTypeParameterV1 {
    label: Option<String>,
    ty: PortableTypeRecipeId,
    rest: bool,
});
tagged!(PortableFieldKeyV1 {
    0 => Symbol { text: String },
    1 => String { text: String },
    2 => Integer { decimal: String },
    3 => Boolean { value: bool },
});
record!(PortableRecordTypeFieldV1 {
    key: PortableFieldKeyV1,
    ty: PortableTypeRecipeId,
});
tagged!(PortableStaticTypeArgumentV1 {
    0 => Term { ty: PortableTypeRecipeId },
    1 => RecordRow { fields: Vec<PortableRecordTypeFieldV1>, tail: Option<PortableTypeParameterRefV1> },
    2 => DynamicType,
});

tagged!(PortableRuntimeTypeRecipeV1 {
    0 => Closed { ty: PortableTypeRecipeId },
    1 => Template { ty: PortableTypeRecipeId },
});
record!(PortableTypeBindingV1 {
    parameter: PortableTypeParameterRefV1,
    recipe: PortableRuntimeTypeRecipeV1,
});

tagged!(PortableConformanceSlotV1 {
    0 => Selection {
        callable: StableCallableRef,
        selector: SelectorPatternV1,
        nested: Vec<PortableConformanceSlotEntryV1>
    },
    1 => Forwarded { slot: u16 },
    2 => DerivedConditional {
        callable: StableCallableRef,
        selector: SelectorPatternV1,
        bindings: Vec<PortableTypeBindingV1>,
        nested: Vec<PortableConformanceSlotEntryV1>
    },
});
record!(PortableConformanceSlotEntryV1 {
    slot: u16,
    recipe: PortableConformanceSlotV1,
});
record!(PortableCallEnvironmentV1 {
    callable: StableCallableRef,
    bindings: Vec<PortableTypeBindingV1>,
    conformance_slots: Vec<PortableConformanceSlotV1>,
});

tagged!(PortableProductSlotReprV1 {
    0 => Int64,
    1 => Float64,
    2 => Bool,
    3 => Symbol,
    4 => Value,
});
record!(PortableProductComponentV1 {
    logical_index: u32,
    repr: PortableProductSlotReprV1,
});
record!(PortableProductLayoutV1 {
    components: Vec<PortableProductComponentV1>,
});

tagged!(PortableAdtRepresentationV1 {
    0 => OptionImmediate,
    1 => TaggedHeap,
});
tagged!(PortableVariantShapeV1 {
    0 => Singleton,
    1 => Positional { arity: u32 },
    2 => Named { fields: Vec<String> },
});
record!(PortableVariantFieldV1 {
    index: u32,
    local_name: String,
    slot: u16,
});
record!(PortableVariantSpecV1 {
    variant: StableVariantRef,
    shape: PortableVariantShapeV1,
    fields: Vec<PortableVariantFieldV1>,
    layout: Option<PortableProductLayoutV1>,
});
record!(PortableEnumSpecV1 {
    owner: StableDeclarationRef,
    representation: PortableAdtRepresentationV1,
    variants: Vec<PortableVariantSpecV1>,
});

record!(PortableDataComponentV1 {
    index: u32,
    local_name: String,
    logical_index: u32,
});
record!(PortableDataSpecV1 {
    owner: StableDeclarationRef,
    layout: Option<PortableProductLayoutV1>,
    components: Vec<PortableDataComponentV1>,
});
record!(PortableDataConstructionV1 {
    constructor: StableDeclarationRef,
    exact_type_digest: [u8; 32],
    type_recipe: PortableRuntimeTypeRecipeV1,
    layout: PortableProductLayoutV1,
    argument_to_component: Vec<u32>,
    component_to_logical: Vec<u32>,
});

tagged!(PortableAnonymousProductKindV1 {
    0 => Tuple { positional_len: u32, labels: Vec<String> },
    1 => Record { logical_keys: Vec<PortableFieldKeyV1>, source_to_logical: Vec<u32> },
    2 => Pack { positional_len: u32, labels: Vec<String> },
});
record!(PortableAnonymousProductV1 {
    kind: PortableAnonymousProductKindV1,
    layout: PortableProductLayoutV1,
    type_recipe: PortableRuntimeTypeRecipeV1,
});

tagged!(PortableSelectorKindV1 {
    0 => Getter,
    1 => Setter,
    2 => Method,
    3 => SubscriptGet,
    4 => SubscriptSet,
});
tagged!(PortableSelectorSlotV1 {
    0 => Positional,
    1 => Label { label: String },
});
record!(PortableFamilyOperationV1 {
    kind: PortableSelectorKindV1,
    slots: Vec<PortableSelectorSlotV1>,
});
tagged!(PortableRestModeV1 {
    0 => None,
    1 => Positional,
    2 => Labeled,
    3 => Complete,
});
tagged!(PortableInvocationTargetV1 {
    0 => Behavioral {
        lookup_owner: StableDeclarationRef,
        callable: StableCallableRef,
        operation: PortableFamilyOperationV1,
        rest_mode: PortableRestModeV1
    },
    1 => VariantConstructor { variant: StableVariantRef },
    2 => DataConstructor { constructor: StableDeclarationRef, construction: u32 },
});
tagged!(PortableFamilyTargetV1 {
    0 => Singleton { variant: StableVariantRef },
    1 => Behavioral { target: PortableInvocationTargetV1 },
    2 => VariantConstructor { variant: StableVariantRef },
    3 => DataConstructor { constructor: StableDeclarationRef, construction: u32 },
});
tagged!(PortableFamilyMemberKindV1 {
    0 => Value,
    1 => Callable,
});
record!(PortableFamilyEntryV1 {
    operation: PortableFamilyOperationV1,
    member_kind: PortableFamilyMemberKindV1,
    target: PortableFamilyTargetV1,
});
record!(PortableFamilyDescriptorV1 {
    entries: Vec<PortableFamilyEntryV1>,
});
record!(PortableFamilyCandidateV1 {
    operation: PortableFamilyOperationV1,
    target: Option<PortableInvocationTargetV1>,
});
record!(PortableFamilyCandidateSetV1 {
    candidates: Vec<PortableFamilyCandidateV1>,
});
record!(PortableConditionalFamilyEntryV1 {
    operation: PortableFamilyOperationV1,
    selector: String,
    declaring_owner: StableDeclarationRef,
    side: StableDispatchSide,
    fallback_method: crate::executable::prototype::ConstantId,
    call_environment: Option<PortableCallEnvironmentId>,
});
record!(PortableConditionalFamilyV1 {
    entries: Vec<PortableConditionalFamilyEntryV1>,
});

record!(PortableTraitRequirementV1 {
    owner: StableDeclarationRef,
    selector: String,
    side: StableDispatchSide,
});
tagged!(PortableTraitSelectionTargetV1 {
    0 => Callable { callable: StableCallableRef },
    1 => DataComponent { component: PortableDataComponentRefV1, specialized_type: PortableTypeRecipeId },
});
record!(PortableDataComponentRefV1 {
    owner: StableDeclarationRef,
    index: u32,
});
record!(PortableTraitAntecedentSlotV1 {
    slot: u16,
    target: PortableTraitSelectionTargetV1,
    selector: String,
});
record!(PortableTraitRequirementTargetV1 {
    requirement: PortableTraitRequirementV1,
    target: PortableTraitSelectionTargetV1,
});
record!(PortableTraitInvocationV1 {
    source_impl: StableImplRef,
    selection: PortableTraitSelectionTargetV1,
    selection_is_trait_default: bool,
    requirement: PortableTraitRequirementV1,
    requirement_slot: u16,
    requirement_slots: Vec<PortableTraitRequirementSlotV1>,
    requirement_targets: Vec<PortableTraitRequirementTargetV1>,
    antecedent_slots: Vec<PortableTraitAntecedentSlotV1>,
    type_bindings: Vec<PortableTypeBindingV1>,
    call_environment: Option<PortableCallEnvironmentId>,
});
record!(PortableTraitRequirementSlotV1 {
    requirement: PortableTraitRequirementV1,
    slot: u16,
});
record!(PortableTraitRequirementInvocationV1 {
    requirement: PortableTraitRequirementV1,
    requirement_slot: u16,
});

record!(PortableSemanticPoolV1 {
    type_recipes: Vec<PortableTypeRecipeV1>,
    kind_recipes: Vec<PortableKindRecipeV1>,
    enum_specs: Vec<PortableEnumSpecV1>,
    variant_targets: Vec<StableVariantRef>,
    data_specs: Vec<PortableDataSpecV1>,
    data_targets: Vec<StableDeclarationRef>,
    data_constructions: Vec<PortableDataConstructionV1>,
    anonymous_products: Vec<PortableAnonymousProductV1>,
    call_environments: Vec<PortableCallEnvironmentV1>,
    associated_targets: Vec<PortableInvocationTargetV1>,
    family_descriptors: Vec<PortableFamilyDescriptorV1>,
    conditional_families: Vec<PortableConditionalFamilyV1>,
    family_operations: Vec<PortableFamilyOperationV1>,
    family_candidate_sets: Vec<PortableFamilyCandidateSetV1>,
    trait_invocations: Vec<PortableTraitInvocationV1>,
    trait_requirement_invocations: Vec<PortableTraitRequirementInvocationV1>,
    type_parameter_class_receivers: Vec<PortableTypeParameterRefV1>,
    provenance: Option<SpanV1>,
});
