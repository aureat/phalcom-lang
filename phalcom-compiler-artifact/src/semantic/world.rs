//! Combined semantic world section payload (`PERF003.C2.P7.T7.B/T8`).

use crate::module_world::{Decode, Wire, invalid, record, tagged};
use crate::semantic::aggregates::{DeclaredTypeFactV1, GenericSignatureV1, StructuralPrimaryContractsV1, TraitRefV1};
use crate::semantic::operational::CheckedStaticDefinitionsV1;
use crate::semantic::static_value::StaticValueGraphV1;
use crate::semantic::type_store::{ArtifactTypeId, TypeGraphV1};
use crate::{ArtifactError, BinaryReader, BinaryWriter, ValidationLimits};
use phalcom_type_meta::identity::{StableCallableRef, StableDeclarationRef, StableImplRef, StableModuleRef, StableVariantRef};

pub const SEMANTIC_WORLD_SCHEMA_V1: u32 = 1;

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
    local_name: Box<str>,
    external_label: Option<Box<str>>,
    declared_type: DeclaredTypeFactV1,
    rest_mode: u8,
});

record!(CallableSignatureRecordV1 {
    callable: StableCallableRef,
    owner: StableDeclarationRef,
    side: u8,
    selector: Box<str>,
    generics: Option<GenericSignatureV1>,
    parameters: Vec<CallableParameterV1>,
    declared_return: DeclaredTypeFactV1,
    return_flow: u8,
});

record!(CallableAnalysisRecordV1 {
    callable: StableCallableRef,
    effects: u32,
    is_terminating: bool,
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

record!(SemanticWorldV1 {
    type_graph: TypeGraphV1,
    structural_contracts: StructuralPrimaryContractsV1,
    static_values: StaticValueGraphV1,
    checked_definitions: CheckedStaticDefinitionsV1,
    witness_plans: Vec<WitnessPlanV1>,
    callable_signatures: Vec<CallableSignatureRecordV1>,
    callable_analyses: Vec<CallableAnalysisRecordV1>,
    trait_surfaces: Vec<TraitSurfaceRecordV1>,
    dispatch_surfaces: Vec<DeclarationSurfaceRecordV1>,
    conformance_heads: Vec<ConformanceHeadRecordV1>,
    constructor_initializations: Vec<ConstructorInitializationRecordV1>,
});

impl SemanticWorldV1 {
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        self.validate(&ValidationLimits::default())?;
        let mut out = BinaryWriter::new();
        out.write_u32(SEMANTIC_WORLD_SCHEMA_V1);
        self.write(&mut out)?;
        Ok(out.into_vec())
    }

    pub fn decode_with_budget(bytes: &[u8], limits: &ValidationLimits, remaining_budget: &mut u64) -> Result<Self, ArtifactError> {
        if bytes.len() as u64 > *remaining_budget {
            return Err(invalid("semantic world section byte budget exceeded"));
        }
        let mut input = Decode {
            reader: BinaryReader::new(bytes),
            limits: *limits,
            allocations_left: *remaining_budget,
            items_left: limits.max_semantic_entries_per_family,
            depth: 0,
        };
        if input.reader.read_u32()? != SEMANTIC_WORLD_SCHEMA_V1 {
            return Err(invalid("unsupported semantic world schema"));
        }
        let world = Self::read(&mut input)?;
        if input.reader.has_remaining() {
            return Err(invalid("trailing semantic world bytes"));
        }
        world.validate(limits)?;
        *remaining_budget = input.allocations_left;
        Ok(world)
    }

    pub fn decode(bytes: &[u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        let mut budget = limits.max_decoded_section_bytes;
        Self::decode_with_budget(bytes, limits, &mut budget)
    }

    pub fn validate(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
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
        {
            return Err(invalid("primary family count budget exceeded"));
        }
        Ok(())
    }
}
