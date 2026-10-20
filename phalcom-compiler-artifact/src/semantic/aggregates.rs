//! Versioned structural semantic contracts used to stage static ADT loading.
//!
//! This module owns only bounded wire DTOs. Declaration identity resolution,
//! type relocation, static-value factory calls, and publication remain owned by
//! `phalcom-semantic`. Rows mirror the live structural shapes exhaustively:
//! declared-type dynamic/unknown states, visibility, shape/constructor
//! templates, and case environments are all carried on the wire rather than
//! defaulted. Source spans are deliberately omitted (logical/capability-
//! separated per the DTO design rule).

use crate::module_world::{Decode, SpanV1, Wire, invalid, record, tagged};
use crate::semantic::type_store::{
    ArtifactKindId, ArtifactStaticConstructorId, ArtifactStaticDomainId, ArtifactTypeId, ArtifactTypeParameterId, ArtifactVariantTypeId, ParameterModeV1,
    ParameterOwnerV1, SelfTypeV1, VarianceV1,
};
use crate::{ArtifactError, BinaryReader, BinaryWriter, ValidationLimits};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::identity::{StableDeclarationRef, StableVariantRef};

pub const STRUCTURAL_PRIMARY_CONTRACTS_SCHEMA_V1: u32 = 1;

record!(DeclarationShellV1 {
    declaration: StableDeclarationRef,
    self_type: ArtifactTypeId,
    parameters: Vec<ArtifactTypeParameterId>,
});

record!(HierarchyAndAliasV1 {
    declaration: StableDeclarationRef,
    superclass: Option<StableDeclarationRef>,
    aliased_type: Option<ArtifactTypeId>,
});

tagged!(DeclarationSemanticFormV1 {
    0 => Ordinary,
    1 => IntrinsicTuple,
    2 => IntrinsicRecord,
    3 => IntrinsicPack,
});

tagged!(IntrinsicRuntimeOwnerV1 {
    0 => Tuple,
    1 => Record,
    2 => Pack,
});

tagged!(IntrinsicTypeOwnerV1 {
    0 => Tuple,
    1 => Record,
    2 => TypePack,
    3 => LabeledTypePack,
    4 => CompleteTypePack,
    5 => RecordRow,
    6 => UnorderedShape,
});

record!(DeclarationTypeBindingV1 {
    semantic_form: DeclarationSemanticFormV1,
    runtime_owner: Option<IntrinsicRuntimeOwnerV1>,
});

record!(GenericSupertypeTemplateV1 {
    declaration: StableDeclarationRef,
    supertype: ArtifactTypeId,
    structural_form: Option<Box<str>>,
});

record!(DeclarationShellV2 {
    declaration: StableDeclarationRef,
    self_type: ArtifactTypeId,
    class_object_type: ArtifactTypeId,
    kind: ArtifactKindId,
    binding: DeclarationTypeBindingV1,
    parameters: Vec<ArtifactTypeParameterId>,
    generic_signature: Option<GenericSignatureV1>,
    supertype_template: Option<GenericSupertypeTemplateV1>,
    intrinsic_type_owner: Option<IntrinsicRuntimeOwnerV1>,
});

record!(IntrinsicStaticDomainRecordV1 {
    declaration: StableDeclarationRef,
    kind: ArtifactKindId,
});

record!(IntrinsicTypeOwnerRecordV1 {
    declaration: StableDeclarationRef,
    owner: IntrinsicTypeOwnerV1,
});

record!(HierarchyEdgeRecordV1 {
    declaration: StableDeclarationRef,
    superclass: Option<StableDeclarationRef>,
    supertype_template: Option<GenericSupertypeTemplateV1>,
});

record!(TypeAliasRecordV1 {
    declaration: StableDeclarationRef,
    kind: ArtifactKindId,
    kind_shape: Box<str>,
    generic_signature: Option<GenericSignatureV1>,
    form: ArtifactTypeId,
    structural_form: Box<str>,
    dependencies: Vec<StableDeclarationRef>,
    source_span: SpanV1,
});

// Published type term. Inference-local variables are rejected by semantic
// publication and never appear on the wire.
tagged!(TypeTermV1 { 0 => Canonical { ty: ArtifactTypeId }, 1 => SelfType { term: SelfTypeV1 } });

record!(TraitRefV1 { declaration: StableDeclarationRef, arguments: Vec<ArtifactTypeId> });

tagged!(GenericConstraintV1 {
    0 => Subtype { lower: TypeTermV1, upper: TypeTermV1 },
    1 => Equivalent { left: TypeTermV1, right: TypeTermV1 },
    2 => Conforms { subject: TypeTermV1, trait_ref: TraitRefV1 },
});

record!(GenericSignatureV1 {
    owner: ParameterOwnerV1,
    parameters: Vec<ArtifactTypeParameterId>,
    parameter_kinds: Vec<ArtifactKindId>,
    parameter_kind_shapes: Vec<Box<str>>,
    parameter_variances: Vec<VarianceV1>,
    parameter_modes: Vec<ParameterModeV1>,
    constraint_shapes: Vec<Box<str>>,
    constraints: Vec<GenericConstraintV1>,
});

tagged!(DynamicReasonV1 { 0 => ExplicitEscape, 1 => DynamicRestPack, 2 => RuntimeReflection });

tagged!(UnknownReasonV1 {
    0 => NoTypeEvidence,
    1 => MissingInitializer,
    2 => UnannotatedDeclaration,
    3 => UnresolvedName { name: Box<str> },
    4 => DynamicMessageSend,
    5 => OpaqueNative,
    6 => RecursiveFixpoint,
    7 => UncheckedExpression,
    8 => SyntaxError,
    9 => UnderconstrainedTypeVariable,
    10 => InferenceConflict,
    11 => InferenceAmbiguous,
    12 => InferenceBlocked,
    13 => InferenceCancelled,
    14 => InferenceBudgetExceeded,
    15 => SuppressedByInvalidCause,
    16 => ExistentialEscape,
});

tagged!(DeclaredTypeBasisV1 {
    0 => Unspecified,
    1 => SourceAnnotation,
    2 => NativeSignature,
    3 => DeclarationSemantics,
    4 => ConstructorSemantics,
    5 => InitializerInference,
    6 => BodyInference,
    7 => ContextualTyping,
    8 => PatternDecomposition,
});

tagged!(DeclaredTypeStateV1 {
    0 => Known { term: TypeTermV1 },
    1 => Dynamic { reason: DynamicReasonV1 },
    2 => Unknown { reason: UnknownReasonV1 },
});

record!(DeclaredTypeFactV1 {
    state: DeclaredTypeStateV1,
    basis: DeclaredTypeBasisV1
});

tagged!(VisibilityV1 { 0 => Public, 1 => Private, 2 => Protected, 3 => Module, 4 => Package, 5 => Internal });

tagged!(DataShapeV1 { 0 => Tuple, 1 => Record });

tagged!(VariantShapeV1 { 0 => Singleton, 1 => Constructor });

tagged!(PayloadKindV1 { 0 => Singleton, 1 => Tuple, 2 => Record, 3 => ExplicitEmpty });

tagged!(ExpansionModeV1 { 0 => Positional, 1 => Labeled, 2 => Complete });

record!(DataComponentSemanticV1 {
    local_name: Box<str>,
    external_label: Option<Box<str>>,
    declared_type: DeclaredTypeFactV1,
});

record!(DataConstructorParameterV1 {
    external_label: Option<Box<str>>,
    local_name: Box<str>,
    declared_type: DeclaredTypeFactV1,
});

tagged!(DataProductContributionV1 {
    0 => Component { component: DataComponentSemanticV1 },
    1 => Expansion { mode: ExpansionModeV1, operand: DeclaredTypeFactV1 },
});

record!(StaticAlgebraicDeclarationV1 {
    domain: ArtifactStaticDomainId,
    constructors: Vec<ArtifactStaticConstructorId>,
});

record!(DataInfoV1 {
    owner: StableDeclarationRef,
    root_form: ArtifactTypeId,
    generic_signature: Option<GenericSignatureV1>,
    static_data: Option<StaticAlgebraicDeclarationV1>,
    shape: DataShapeV1,
    components: Vec<DataComponentSemanticV1>,
    constructor_visibility: VisibilityV1,
    constructor_parameters: Vec<DataConstructorParameterV1>,
    constructor_result_type: ArtifactTypeId,
    shape_contributions: Vec<DataProductContributionV1>,
});

record!(VariantFieldV1 {
    local_name: Box<str>,
    external_label: Option<Box<str>>,
    declared_type: DeclaredTypeFactV1,
});

record!(VariantConstructorParameterV1 {
    external_label: Option<Box<str>>,
    local_name: Box<str>,
    declared_type: DeclaredTypeFactV1,
});

record!(CaseBindingV1 {
    parameter: ArtifactTypeParameterId,
    ty: ArtifactTypeId
});

record!(CaseEnvironmentV1 {
    bindings: Vec<CaseBindingV1>,
    equalities: Vec<GenericConstraintV1>,
});

record!(VariantConstructorV1 {
    generic_signature: Option<GenericSignatureV1>,
    parameters: Vec<VariantConstructorParameterV1>,
    result_type: ArtifactTypeId,
    exact_case_type: ArtifactTypeId,
});

record!(VariantVisibilityV1 {
    name: VisibilityV1,
    construct: VisibilityV1,
    payload: VisibilityV1,
});

record!(VariantInfoV1 {
    variant: StableVariantRef,
    variant_type: ArtifactVariantTypeId,
    shape: VariantShapeV1,
    payload_kind: PayloadKindV1,
    fields: Vec<VariantFieldV1>,
    result_type: ArtifactTypeId,
    exact_case_type: ArtifactTypeId,
    case_environment: CaseEnvironmentV1,
    constructor: Option<VariantConstructorV1>,
    visibility: VariantVisibilityV1,
});

record!(EnumInfoV1 {
    owner: StableDeclarationRef,
    root_form: ArtifactTypeId,
    generic_signature: Option<GenericSignatureV1>,
    static_data: Option<StaticAlgebraicDeclarationV1>,
    default_result_type: ArtifactTypeId,
    variants: Vec<StableVariantRef>,
});

record!(StructuralPrimaryContractsV1 {
    declaration_shells: Vec<DeclarationShellV1>,
    hierarchy_and_aliases: Vec<HierarchyAndAliasV1>,
    generic_signatures: Vec<GenericSignatureV1>,
    data_contracts: Vec<DataInfoV1>,
    enum_contracts: Vec<EnumInfoV1>,
    variant_contracts: Vec<VariantInfoV1>,
});

impl StructuralPrimaryContractsV1 {
    /// Encodes the structural contracts with an explicit schema marker.
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        self.validate(&ValidationLimits::default())?;
        let mut out = BinaryWriter::new();
        out.write_u32(STRUCTURAL_PRIMARY_CONTRACTS_SCHEMA_V1);
        self.write(&mut out)?;
        Ok(out.into_vec())
    }

    /// Decodes and validates structural rows before a semantic store sees them.
    pub fn decode(bytes: &[u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        if bytes.len() as u64 > limits.max_decoded_section_bytes {
            return Err(invalid("structural contract section exceeds decoded byte limit"));
        }
        let allocation_limit =
            usize::try_from(limits.max_decoded_section_bytes).map_err(|_| invalid("structural contract allocation limit exceeds address space"))?;
        let mut allocations = AllocationBudget::new(allocation_limit);
        let mut input = Decode {
            reader: BinaryReader::new(bytes),
            limits: *limits,
            allocations: &mut allocations,
            items_left: limits.max_semantic_entries_per_family,
            depth: 0,
        };
        if input.reader.read_u32()? != STRUCTURAL_PRIMARY_CONTRACTS_SCHEMA_V1 {
            return Err(invalid("unsupported structural contract schema"));
        }
        let contracts = Self::read(&mut input)?;
        if input.reader.has_remaining() {
            return Err(invalid("trailing structural contract bytes"));
        }
        contracts.validate(limits)?;
        Ok(contracts)
    }

    /// Validates row-local ordering, owner identity, declared-order preservation,
    /// and bounded cross references. Graph references into the type/static
    /// graphs are checked against those graphs by the semantic loader.
    pub fn validate(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        let allocation_limit =
            usize::try_from(limits.max_decoded_section_bytes).map_err(|_| invalid("structural contract allocation limit exceeds address space"))?;
        let mut allocations = AllocationBudget::new(allocation_limit);
        self.validate_with_allocation_budget(limits, &mut allocations)
    }

    /// Validates structural rows while charging duplicate-detection scratch to
    /// the caller's semantic import transaction.
    pub fn validate_with_allocation_budget(&self, limits: &ValidationLimits, allocations: &mut AllocationBudget) -> Result<(), ArtifactError> {
        let total = self
            .declaration_shells
            .len()
            .checked_add(self.hierarchy_and_aliases.len())
            .and_then(|n| n.checked_add(self.generic_signatures.len()))
            .and_then(|n| n.checked_add(self.data_contracts.len()))
            .and_then(|n| n.checked_add(self.enum_contracts.len()))
            .and_then(|n| n.checked_add(self.variant_contracts.len()))
            .ok_or_else(|| invalid("structural contract row count overflow"))?;
        if total > limits.max_semantic_entries_per_family || total > u32::MAX as usize {
            return Err(invalid("structural contract row budget exceeded"));
        }
        for shell in &self.declaration_shells {
            validate_owner(&shell.declaration, limits)?;
            validate_index_list(&shell.parameters, limits)?;
        }
        for hierarchy in &self.hierarchy_and_aliases {
            validate_owner(&hierarchy.declaration, limits)?;
            if let Some(superclass) = &hierarchy.superclass {
                validate_owner(superclass, limits)?;
            }
        }
        for signature in &self.generic_signatures {
            validate_signature(signature, limits)?;
        }
        for data in &self.data_contracts {
            validate_owner(&data.owner, limits)?;
            validate_components(&data.components, limits)?;
            if data.constructor_parameters.len() != data.components.len() {
                return Err(invalid("data constructor parameters do not zip with components"));
            }
            for parameter in &data.constructor_parameters {
                validate_name(&parameter.local_name, limits)?;
            }
            validate_contributions(&data.shape_contributions, &data.components, limits)?;
            if let Some(static_data) = &data.static_data {
                validate_static_data(static_data, limits)?;
            }
            if let Some(signature) = &data.generic_signature {
                validate_signature(signature, limits)?;
            }
        }
        let mut variant_rows = allocations
            .vector::<&StableVariantRef>(self.variant_contracts.len())
            .map_err(structural_allocation_error)?;
        for variant in &self.variant_contracts {
            validate_owner(&variant.variant.owner, limits)?;
            validate_name(&variant.variant.selector, limits)?;
            if validate_index_list(&variant.fields, limits).is_err() {
                return Err(invalid("structural variant field budget exceeded"));
            }
            for field in &variant.fields {
                validate_name(&field.local_name, limits)?;
            }
            if let Some(constructor) = &variant.constructor {
                if variant.shape == VariantShapeV1::Singleton {
                    return Err(invalid("singleton variant carries a constructor"));
                }
                if constructor.parameters.len() != variant.fields.len() {
                    return Err(invalid("variant constructor parameters do not zip with fields"));
                }
                if let Some(signature) = &constructor.generic_signature {
                    validate_signature(signature, limits)?;
                }
            } else if variant.shape == VariantShapeV1::Constructor {
                return Err(invalid("constructor variant has no constructor signature"));
            }
            let mut previous_parameter = None;
            for binding in &variant.case_environment.bindings {
                if previous_parameter.is_some_and(|previous| binding.parameter.0 <= previous) {
                    return Err(invalid("case environment bindings are not sorted by parameter"));
                }
                previous_parameter = Some(binding.parameter.0);
            }
            validate_constraint_list(&variant.case_environment.equalities, limits)?;
            variant_rows.push(&variant.variant);
        }
        variant_rows.sort_unstable();
        if variant_rows.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(invalid("duplicate structural variant row"));
        }
        let membership_count = self
            .enum_contracts
            .iter()
            .try_fold(0usize, |count, enum_info| count.checked_add(enum_info.variants.len()))
            .ok_or_else(|| invalid("enum variant membership count overflow"))?;
        let mut variant_memberships = allocations.vector::<&StableVariantRef>(membership_count).map_err(structural_allocation_error)?;
        for enum_info in &self.enum_contracts {
            validate_owner(&enum_info.owner, limits)?;
            if let Some(signature) = &enum_info.generic_signature {
                validate_signature(signature, limits)?;
            }
            if let Some(static_data) = &enum_info.static_data {
                validate_static_data(static_data, limits)?;
            }
            for variant in &enum_info.variants {
                if variant.owner != enum_info.owner {
                    return Err(invalid("enum variant membership names a foreign owner"));
                }
                validate_name(&variant.selector, limits)?;
                if variant_rows.binary_search(&variant).is_err() {
                    return Err(invalid("enum variant has no structural variant row"));
                }
                variant_memberships.push(variant);
            }
        }
        if self.enum_contracts.windows(2).any(|pair| pair[0].owner >= pair[1].owner) {
            return Err(invalid("enum contracts must be unique and sorted by owner"));
        }
        variant_memberships.sort_unstable();
        if variant_memberships.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(invalid("enum variant membership is duplicated"));
        }
        if variant_rows != variant_memberships {
            return Err(invalid("structural variant row belongs to no enum contract"));
        }
        Ok(())
    }
}

pub const STRUCTURAL_PRIMARY_CONTRACTS_SCHEMA_V2: u32 = 2;

record!(StructuralPrimaryContractsV2 {
    declaration_shells: Vec<DeclarationShellV2>,
    intrinsic_static_domains: Vec<IntrinsicStaticDomainRecordV1>,
    intrinsic_type_owners: Vec<IntrinsicTypeOwnerRecordV1>,
    hierarchy_edges: Vec<HierarchyEdgeRecordV1>,
    type_aliases: Vec<TypeAliasRecordV1>,
    generic_signatures: Vec<GenericSignatureV1>,
    data_contracts: Vec<DataInfoV1>,
    enum_contracts: Vec<EnumInfoV1>,
    variant_contracts: Vec<VariantInfoV1>,
});

impl StructuralPrimaryContractsV2 {
    /// Encodes the structural contracts with an explicit schema marker.
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        self.validate(&ValidationLimits::default())?;
        let mut out = BinaryWriter::new();
        out.write_u32(STRUCTURAL_PRIMARY_CONTRACTS_SCHEMA_V2);
        self.write(&mut out)?;
        Ok(out.into_vec())
    }

    /// Decodes and validates structural rows before a semantic store sees them.
    pub fn decode(bytes: &[u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        if bytes.len() as u64 > limits.max_decoded_section_bytes {
            return Err(invalid("structural contract section exceeds decoded byte limit"));
        }
        let allocation_limit =
            usize::try_from(limits.max_decoded_section_bytes).map_err(|_| invalid("structural contract allocation limit exceeds address space"))?;
        let mut allocations = AllocationBudget::new(allocation_limit);
        let mut input = Decode {
            reader: BinaryReader::new(bytes),
            limits: *limits,
            allocations: &mut allocations,
            items_left: limits.max_semantic_entries_per_family,
            depth: 0,
        };
        if input.reader.read_u32()? != STRUCTURAL_PRIMARY_CONTRACTS_SCHEMA_V2 {
            return Err(invalid("unsupported structural contract schema"));
        }
        let contracts = Self::read(&mut input)?;
        if input.reader.has_remaining() {
            return Err(invalid("trailing structural contract bytes"));
        }
        contracts.validate(limits)?;
        Ok(contracts)
    }

    /// Validates row-local ordering, owner identity, declared-order preservation,
    /// and bounded cross references. Graph references into the type/static
    /// graphs are checked against those graphs by the semantic loader.
    pub fn validate(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        let allocation_limit =
            usize::try_from(limits.max_decoded_section_bytes).map_err(|_| invalid("structural contract allocation limit exceeds address space"))?;
        let mut allocations = AllocationBudget::new(allocation_limit);
        self.validate_with_allocation_budget(limits, &mut allocations)
    }

    /// Validates structural rows while charging duplicate-detection scratch to
    /// the caller's semantic import transaction.
    pub fn validate_with_allocation_budget(&self, limits: &ValidationLimits, allocations: &mut AllocationBudget) -> Result<(), ArtifactError> {
        let total = self
            .declaration_shells
            .len()
            .checked_add(self.intrinsic_static_domains.len())
            .and_then(|n| n.checked_add(self.intrinsic_type_owners.len()))
            .and_then(|n| n.checked_add(self.hierarchy_edges.len()))
            .and_then(|n| n.checked_add(self.type_aliases.len()))
            .and_then(|n| n.checked_add(self.generic_signatures.len()))
            .and_then(|n| n.checked_add(self.data_contracts.len()))
            .and_then(|n| n.checked_add(self.enum_contracts.len()))
            .and_then(|n| n.checked_add(self.variant_contracts.len()))
            .ok_or_else(|| invalid("structural contract row count overflow"))?;
        if total > limits.max_semantic_entries_per_family || total > u32::MAX as usize {
            return Err(invalid("structural contract row budget exceeded"));
        }
        for shell in &self.declaration_shells {
            validate_owner(&shell.declaration, limits)?;
            validate_index_list(&shell.parameters, limits)?;
            if shell.intrinsic_type_owner != shell.binding.runtime_owner {
                return Err(invalid("declaration shell intrinsic runtime owner disagrees with its binding"));
            }
            if let Some(signature) = &shell.generic_signature {
                validate_signature(signature, limits)?;
            }
            if let Some(template) = &shell.supertype_template {
                validate_owner(&template.declaration, limits)?;
            }
        }
        if self.declaration_shells.windows(2).any(|pair| pair[0].declaration >= pair[1].declaration) {
            return Err(invalid("declaration shells must be unique and sorted by owner"));
        }
        for record in &self.intrinsic_static_domains {
            validate_owner(&record.declaration, limits)?;
        }
        if self
            .intrinsic_static_domains
            .windows(2)
            .any(|pair| pair[0].declaration >= pair[1].declaration)
        {
            return Err(invalid("intrinsic static domains must be unique and sorted by owner"));
        }
        for record in &self.intrinsic_type_owners {
            validate_owner(&record.declaration, limits)?;
        }
        if self
            .intrinsic_type_owners
            .windows(2)
            .any(|pair| pair[0].declaration >= pair[1].declaration)
        {
            return Err(invalid("intrinsic type owners must be unique and sorted by owner"));
        }
        for hierarchy in &self.hierarchy_edges {
            validate_owner(&hierarchy.declaration, limits)?;
            if let Some(superclass) = &hierarchy.superclass {
                validate_owner(superclass, limits)?;
            }
            if let Some(template) = &hierarchy.supertype_template {
                validate_owner(&template.declaration, limits)?;
            }
        }
        for alias in &self.type_aliases {
            validate_owner(&alias.declaration, limits)?;
            if let Some(signature) = &alias.generic_signature {
                validate_signature(signature, limits)?;
            }
            for dep in &alias.dependencies {
                validate_owner(dep, limits)?;
            }
            if alias.source_span.start > alias.source_span.end {
                return Err(invalid("inverted source span in type alias"));
            }
        }
        for signature in &self.generic_signatures {
            validate_signature(signature, limits)?;
        }
        for data in &self.data_contracts {
            validate_owner(&data.owner, limits)?;
            validate_components(&data.components, limits)?;
            if data.constructor_parameters.len() != data.components.len() {
                return Err(invalid("data constructor parameters do not zip with components"));
            }
            for parameter in &data.constructor_parameters {
                validate_name(&parameter.local_name, limits)?;
            }
            validate_contributions(&data.shape_contributions, &data.components, limits)?;
            if let Some(static_data) = &data.static_data {
                validate_static_data(static_data, limits)?;
            }
            if let Some(signature) = &data.generic_signature {
                validate_signature(signature, limits)?;
            }
        }
        let mut variant_rows = allocations
            .vector::<&StableVariantRef>(self.variant_contracts.len())
            .map_err(structural_allocation_error)?;
        for variant in &self.variant_contracts {
            validate_owner(&variant.variant.owner, limits)?;
            validate_name(&variant.variant.selector, limits)?;
            if validate_index_list(&variant.fields, limits).is_err() {
                return Err(invalid("structural variant field budget exceeded"));
            }
            for field in &variant.fields {
                validate_name(&field.local_name, limits)?;
            }
            if let Some(constructor) = &variant.constructor {
                if variant.shape == VariantShapeV1::Singleton {
                    return Err(invalid("singleton variant carries a constructor"));
                }
                if constructor.parameters.len() != variant.fields.len() {
                    return Err(invalid("variant constructor parameters do not zip with fields"));
                }
                if let Some(signature) = &constructor.generic_signature {
                    validate_signature(signature, limits)?;
                }
            } else if variant.shape == VariantShapeV1::Constructor {
                return Err(invalid("constructor variant has no constructor signature"));
            }
            let mut previous_parameter = None;
            for binding in &variant.case_environment.bindings {
                if previous_parameter.is_some_and(|previous| binding.parameter.0 <= previous) {
                    return Err(invalid("case environment bindings are not sorted by parameter"));
                }
                previous_parameter = Some(binding.parameter.0);
            }
            validate_constraint_list(&variant.case_environment.equalities, limits)?;
            variant_rows.push(&variant.variant);
        }
        variant_rows.sort_unstable();
        if variant_rows.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(invalid("duplicate structural variant row"));
        }
        let membership_count = self
            .enum_contracts
            .iter()
            .try_fold(0usize, |count, enum_info| count.checked_add(enum_info.variants.len()))
            .ok_or_else(|| invalid("enum variant membership count overflow"))?;
        let mut variant_memberships = allocations.vector::<&StableVariantRef>(membership_count).map_err(structural_allocation_error)?;
        for enum_info in &self.enum_contracts {
            validate_owner(&enum_info.owner, limits)?;
            if let Some(signature) = &enum_info.generic_signature {
                validate_signature(signature, limits)?;
            }
            if let Some(static_data) = &enum_info.static_data {
                validate_static_data(static_data, limits)?;
            }
            for variant in &enum_info.variants {
                if variant.owner != enum_info.owner {
                    return Err(invalid("enum variant membership names a foreign owner"));
                }
                validate_name(&variant.selector, limits)?;
                if variant_rows.binary_search(&variant).is_err() {
                    return Err(invalid("enum variant has no structural variant row"));
                }
                variant_memberships.push(variant);
            }
        }
        if self.enum_contracts.windows(2).any(|pair| pair[0].owner >= pair[1].owner) {
            return Err(invalid("enum contracts must be unique and sorted by owner"));
        }
        variant_memberships.sort_unstable();
        if variant_memberships.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(invalid("enum variant membership is duplicated"));
        }
        if variant_rows != variant_memberships {
            return Err(invalid("structural variant row belongs to no enum contract"));
        }
        Ok(())
    }
}

impl From<StructuralPrimaryContractsV1> for StructuralPrimaryContractsV2 {
    fn from(v1: StructuralPrimaryContractsV1) -> Self {
        let declaration_shells = v1
            .declaration_shells
            .into_iter()
            .map(|s| DeclarationShellV2 {
                declaration: s.declaration,
                self_type: s.self_type,
                class_object_type: s.self_type,
                kind: ArtifactKindId(0),
                binding: DeclarationTypeBindingV1 {
                    semantic_form: DeclarationSemanticFormV1::Ordinary,
                    runtime_owner: None,
                },
                parameters: s.parameters,
                generic_signature: None,
                supertype_template: None,
                intrinsic_type_owner: None,
            })
            .collect();
        let hierarchy_edges = v1
            .hierarchy_and_aliases
            .into_iter()
            .map(|h| HierarchyEdgeRecordV1 {
                declaration: h.declaration,
                superclass: h.superclass,
                supertype_template: None,
            })
            .collect();
        Self {
            declaration_shells,
            intrinsic_static_domains: Vec::new(),
            intrinsic_type_owners: Vec::new(),
            hierarchy_edges,
            type_aliases: Vec::new(),
            generic_signatures: v1.generic_signatures,
            data_contracts: v1.data_contracts,
            enum_contracts: v1.enum_contracts,
            variant_contracts: v1.variant_contracts,
        }
    }
}

fn structural_allocation_error(error: AllocationError) -> ArtifactError {
    ArtifactError::Allocation(error)
}

pub(crate) fn validate_signature(signature: &GenericSignatureV1, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    validate_index_list(&signature.parameters, limits)?;
    if signature.parameter_modes.len() != signature.parameters.len() {
        return Err(invalid("generic signature parameter modes disagree with parameters"));
    }
    if !signature.parameter_kinds.is_empty() && signature.parameter_kinds.len() != signature.parameters.len() {
        return Err(invalid("generic signature parameter kinds disagree with parameters"));
    }
    if !signature.parameter_variances.is_empty() && signature.parameter_variances.len() != signature.parameters.len() {
        return Err(invalid("generic signature parameter variances disagree with parameters"));
    }
    if !signature.parameter_kind_shapes.is_empty() && signature.parameter_kind_shapes.len() != signature.parameters.len() {
        return Err(invalid("generic signature kind shapes disagree with parameters"));
    }
    validate_constraint_list(&signature.constraints, limits)?;
    Ok(())
}

fn validate_constraint_list(constraints: &[GenericConstraintV1], limits: &ValidationLimits) -> Result<(), ArtifactError> {
    if constraints.len() > limits.max_semantic_entries_per_family {
        return Err(invalid("structural constraint budget exceeded"));
    }
    for constraint in constraints {
        let mut terms = 0usize;
        match constraint {
            GenericConstraintV1::Subtype { lower, upper } => {
                terms += lower.terms() + upper.terms();
            }
            GenericConstraintV1::Equivalent { left, right } => {
                terms += left.terms() + right.terms();
            }
            GenericConstraintV1::Conforms { subject, trait_ref } => {
                terms += subject.terms();
                validate_owner(&trait_ref.declaration, limits)?;
                validate_index_list(&trait_ref.arguments, limits)?;
            }
        }
        if terms > limits.max_semantic_entries_per_family {
            return Err(invalid("structural constraint term budget exceeded"));
        }
    }
    Ok(())
}

impl TypeTermV1 {
    fn terms(&self) -> usize {
        match self {
            Self::Canonical { .. } | Self::SelfType { .. } => 1,
        }
    }
}

fn validate_components(components: &[DataComponentSemanticV1], limits: &ValidationLimits) -> Result<(), ArtifactError> {
    if components.len() > limits.max_semantic_entries_per_family {
        return Err(invalid("structural component budget exceeded"));
    }
    for component in components {
        validate_name(&component.local_name, limits)?;
    }
    Ok(())
}

fn validate_contributions(
    contributions: &[DataProductContributionV1],
    components: &[DataComponentSemanticV1],
    limits: &ValidationLimits,
) -> Result<(), ArtifactError> {
    if contributions.len() > limits.max_semantic_entries_per_family {
        return Err(invalid("structural shape contribution budget exceeded"));
    }
    let mut component_index = 0usize;
    for contribution in contributions {
        match contribution {
            DataProductContributionV1::Component { component } => {
                let Some(expected) = components.get(component_index) else {
                    return Err(invalid("shape contributions enumerate more components than declared"));
                };
                if expected != component {
                    return Err(invalid("shape contribution component disagrees with declared component order"));
                }
                component_index += 1;
            }
            DataProductContributionV1::Expansion { .. } => {}
        }
    }
    if component_index != components.len() {
        return Err(invalid("shape contributions do not cover every declared component in order"));
    }
    Ok(())
}

fn validate_static_data(static_data: &StaticAlgebraicDeclarationV1, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    validate_index_list(&static_data.constructors, limits)
}

fn validate_index_list<T>(values: &[T], limits: &ValidationLimits) -> Result<(), ArtifactError> {
    if values.len() > limits.max_semantic_entries_per_family {
        return Err(invalid("structural index list budget exceeded"));
    }
    Ok(())
}

fn validate_name(value: &str, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    if value.is_empty() || value.len() > limits.max_string_bytes {
        return Err(invalid("invalid structural semantic name"));
    }
    if !value.is_char_boundary(value.len()) {
        return Err(invalid("structural semantic name is not UTF-8"));
    }
    Ok(())
}

pub(crate) fn validate_owner(value: &StableDeclarationRef, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    crate::identity::validate_persistent_project_ref(&value.module.project, limits)?;
    if value.module.path.is_empty() || value.path.is_empty() {
        return Err(invalid("structural owner identity has an empty path"));
    }
    validate_index_list(&value.module.path, limits)?;
    validate_index_list(&value.path, limits)?;
    for segment in value.module.path.iter().chain(value.path.iter()) {
        if segment.is_empty() || segment.len() > limits.max_string_bytes {
            return Err(invalid("empty or oversized structural owner path segment"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod owner_validation_tests {
    use super::*;
    use crate::ValidationError;
    use phalcom_type_meta::fingerprint::Fingerprint128;
    use phalcom_type_meta::identity::{StableModuleRef, StableProjectRef};

    fn contracts(project: StableProjectRef) -> StructuralPrimaryContractsV1 {
        StructuralPrimaryContractsV1 {
            declaration_shells: vec![DeclarationShellV1 {
                declaration: StableDeclarationRef {
                    module: StableModuleRef {
                        project,
                        path: Box::new(["library".into()]),
                    },
                    path: Box::new(["Widget".into()]),
                },
                self_type: ArtifactTypeId(0),
                parameters: vec![],
            }],
            hierarchy_and_aliases: vec![],
            generic_signatures: vec![],
            data_contracts: vec![],
            enum_contracts: vec![],
            variant_contracts: vec![],
        }
    }

    #[test]
    fn structural_owner_syntax_accepts_persistent_package_identity() {
        let package = StableProjectRef::Package {
            package: "example.library".into(),
            version: "1.2.3".into(),
            artifact_fingerprint: Fingerprint128([9; 16]),
        };
        assert!(contracts(package).validate(&ValidationLimits::default()).is_ok());
    }

    #[test]
    fn structural_owner_syntax_rejects_session_identity() {
        let session = StableProjectRef::Session {
            session_fingerprint: Fingerprint128([9; 16]),
        };
        assert!(matches!(
            contracts(session).validate(&ValidationLimits::default()),
            Err(ArtifactError::Validation(ValidationError::ReferentialIntegrity(message))) if message.contains("not persistent")
        ));
    }
}
