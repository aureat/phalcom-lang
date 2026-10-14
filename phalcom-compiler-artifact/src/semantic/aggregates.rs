//! Versioned structural semantic contracts used to stage static ADT loading.
//!
//! This module owns only bounded wire DTOs. Declaration identity resolution,
//! type relocation, static-value factory calls, and publication remain owned by
//! `phalcom-semantic`. Rows mirror the live structural shapes exhaustively:
//! declared-type dynamic/unknown states, visibility, shape/constructor
//! templates, and case environments are all carried on the wire rather than
//! defaulted. Source spans are deliberately omitted (logical/capability-
//! separated per the DTO design rule).

use crate::module_world::{Decode, Wire, invalid, record, tagged};
use crate::semantic::type_store::{
    ArtifactKindId, ArtifactStaticConstructorId, ArtifactStaticDomainId, ArtifactTypeParameterId, ArtifactTypeId, ArtifactVariantTypeId,
    ParameterModeV1, ParameterOwnerV1, SelfTypeV1, VarianceV1,
};
use crate::{ArtifactError, BinaryReader, BinaryWriter, ValidationError, ValidationLimits};
use phalcom_type_meta::identity::{StableDeclarationRef, StableVariantRef};
use std::collections::BTreeSet;

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

record!(DeclaredTypeFactV1 { state: DeclaredTypeStateV1, basis: DeclaredTypeBasisV1 });

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

record!(CaseBindingV1 { parameter: ArtifactTypeParameterId, ty: ArtifactTypeId });

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
        let mut input = Decode {
            reader: BinaryReader::new(bytes),
            limits: *limits,
            allocations_left: limits.max_decoded_section_bytes,
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
        let total = self.declaration_shells.len()
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
        let mut variant_rows = BTreeSet::new();
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
            if !variant_rows.insert(variant.variant.clone()) {
                return Err(invalid("duplicate structural variant row"));
            }
        }
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
                if !variant_rows.remove(variant) {
                    return Err(invalid("enum variant has no structural variant row"));
                }
            }
        }
        if !variant_rows.is_empty() {
            return Err(invalid("structural variant row belongs to no enum contract"));
        }
        Ok(())
    }
}

fn validate_signature(signature: &GenericSignatureV1, limits: &ValidationLimits) -> Result<(), ArtifactError> {
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

fn validate_owner(value: &StableDeclarationRef, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    use phalcom_type_meta::identity::StableProjectRef;
    match &value.module.project {
        StableProjectRef::Builtin { namespace, version } if !namespace.is_empty() && !version.is_empty() => {
            if namespace.len() > limits.max_string_bytes || version.len() > limits.max_string_bytes {
                return Err(invalid("structural owner identity exceeds string budget"));
            }
        }
        _ => {
            return Err(ValidationError::ReferentialIntegrity("structural owner is not a builtin Universe identity".into()).into());
        }
    }
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
