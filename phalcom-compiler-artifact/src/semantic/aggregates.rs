//! Versioned structural semantic contracts used to stage static ADT loading.
//!
//! This module owns only bounded wire DTOs. Declaration identity resolution,
//! type relocation, static-value factory calls, and publication remain owned by
//! `phalcom-semantic`.

use crate::semantic::static_value::ArtifactStaticValueId;
use crate::semantic::type_store::{ArtifactTypeId, ArtifactTypeParameterId, ArtifactVariantTypeId};
use crate::module_world::{Decode, Wire, invalid};
use crate::{ArtifactError, BinaryReader, BinaryWriter, ValidationError, ValidationLimits};
use phalcom_type_meta::identity::{StableDeclarationRef, StableVariantRef};

pub const STRUCTURAL_PRIMARY_CONTRACTS_SCHEMA_V1: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingStaticValueRefV1 {
    pub owner: StableDeclarationRef,
    pub target_value: ArtifactStaticValueId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConstraintKindV1 {
    Subtype,
    Equivalent,
    Conforms,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationShellV1 {
    pub declaration: StableDeclarationRef,
    pub self_type: ArtifactTypeId,
    pub parameters: Vec<ArtifactTypeParameterId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyAndAliasV1 {
    pub declaration: StableDeclarationRef,
    pub superclass: Option<StableDeclarationRef>,
    pub aliased_type: Option<ArtifactTypeId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedConstraintV1 {
    pub subject: ArtifactTypeId,
    pub bound: ArtifactTypeId,
    pub kind: ConstraintKindV1,
    pub pending_static: Option<PendingStaticValueRefV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericSignatureV1 {
    pub owner: StableDeclarationRef,
    pub parameters: Vec<ArtifactTypeParameterId>,
    pub constraints: Vec<NormalizedConstraintV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataComponentContractV1 {
    pub name: String,
    pub component_type: ArtifactTypeId,
    pub declared_index: u32,
    pub default_static: Option<PendingStaticValueRefV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataInfoV1 {
    pub owner: StableDeclarationRef,
    pub constructor_selector: String,
    pub result_type: ArtifactTypeId,
    pub components: Vec<DataComponentContractV1>,
    pub has_expansions: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantInfoV1 {
    pub variant: StableVariantRef,
    pub variant_type: ArtifactVariantTypeId,
    pub result_type: ArtifactTypeId,
    pub fields: Vec<DataComponentContractV1>,
    pub discriminant_static: Option<PendingStaticValueRefV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumInfoV1 {
    pub owner: StableDeclarationRef,
    pub enum_type: ArtifactTypeId,
    pub variants: Vec<VariantInfoV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct StructuralPrimaryContractsV1 {
    pub declaration_shells: Vec<DeclarationShellV1>,
    pub hierarchy_and_aliases: Vec<HierarchyAndAliasV1>,
    pub generic_signatures: Vec<GenericSignatureV1>,
    pub data_contracts: Vec<DataInfoV1>,
    pub enum_contracts: Vec<EnumInfoV1>,
}

macro_rules! wire_record {
    ($type:ty { $($field:ident),+ $(,)? }) => {
        impl Wire for $type {
            fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
                $(self.$field.write(out)?;)+
                Ok(())
            }
            fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
                input.nested(|input| Ok(Self { $($field: Wire::read(input)?),+ }))
            }
        }
    };
}

impl Wire for ConstraintKindV1 {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        out.write_u8(match self { Self::Subtype => 0, Self::Equivalent => 1, Self::Conforms => 2 });
        Ok(())
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        match input.reader.read_u8()? {
            0 => Ok(Self::Subtype),
            1 => Ok(Self::Equivalent),
            2 => Ok(Self::Conforms),
            _ => Err(invalid("invalid structural constraint kind")),
        }
    }
}

wire_record!(PendingStaticValueRefV1 { owner, target_value });
wire_record!(DeclarationShellV1 { declaration, self_type, parameters });
wire_record!(HierarchyAndAliasV1 { declaration, superclass, aliased_type });
wire_record!(NormalizedConstraintV1 { subject, bound, kind, pending_static });
wire_record!(GenericSignatureV1 { owner, parameters, constraints });
wire_record!(DataComponentContractV1 { name, component_type, declared_index, default_static });
wire_record!(DataInfoV1 { owner, constructor_selector, result_type, components, has_expansions });
wire_record!(VariantInfoV1 { variant, variant_type, result_type, fields, discriminant_static });
wire_record!(EnumInfoV1 { owner, enum_type, variants });
wire_record!(StructuralPrimaryContractsV1 { declaration_shells, hierarchy_and_aliases, generic_signatures, data_contracts, enum_contracts });

impl StructuralPrimaryContractsV1 {
    /// Encodes the structural subset with an explicit schema marker.
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

    /// Validates row-local ordering, owner identity and bounded cross references.
    /// Graph references are checked against the type/static graphs by the loader.
    pub fn validate(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        let total = self.declaration_shells.len()
            .checked_add(self.hierarchy_and_aliases.len())
            .and_then(|n| n.checked_add(self.generic_signatures.len()))
            .and_then(|n| n.checked_add(self.data_contracts.len()))
            .and_then(|n| n.checked_add(self.enum_contracts.len()))
            .ok_or_else(|| invalid("structural contract row count overflow"))?;
        if total > limits.max_semantic_entries_per_family || total > u32::MAX as usize {
            return Err(invalid("structural contract row budget exceeded"));
        }
        for shell in &self.declaration_shells {
            validate_owner(&shell.declaration, limits)?;
            validate_nonempty_index_list(&shell.parameters, limits)?;
        }
        for hierarchy in &self.hierarchy_and_aliases {
            validate_owner(&hierarchy.declaration, limits)?;
            if let Some(superclass) = &hierarchy.superclass { validate_owner(superclass, limits)?; }
        }
        for signature in &self.generic_signatures {
            validate_owner(&signature.owner, limits)?;
            validate_nonempty_index_list(&signature.parameters, limits)?;
            for constraint in &signature.constraints {
                if constraint.kind == ConstraintKindV1::Conforms && constraint.bound.0 == u32::MAX {
                    return Err(invalid("conformance constraint has an invalid bound"));
                }
                if let Some(pending) = &constraint.pending_static { validate_pending(pending, limits)?; }
            }
        }
        for data in &self.data_contracts {
            validate_owner(&data.owner, limits)?;
            validate_name(&data.constructor_selector, limits)?;
            validate_components(&data.components, limits)?;
        }
        for enum_info in &self.enum_contracts {
            validate_owner(&enum_info.owner, limits)?;
            for variant in &enum_info.variants {
                validate_owner(&variant.variant.owner, limits)?;
                validate_name(&variant.variant.selector, limits)?;
                validate_components(&variant.fields, limits)?;
                if let Some(pending) = &variant.discriminant_static { validate_pending(pending, limits)?; }
            }
        }
        Ok(())
    }
}

fn validate_pending(value: &PendingStaticValueRefV1, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    validate_owner(&value.owner, limits)
}

fn validate_components(components: &[DataComponentContractV1], limits: &ValidationLimits) -> Result<(), ArtifactError> {
    if components.len() > limits.max_semantic_entries_per_family {
        return Err(invalid("structural component budget exceeded"));
    }
    for (index, component) in components.iter().enumerate() {
        if component.declared_index as usize != index {
            return Err(invalid("structural component indexes must preserve declaration order"));
        }
        validate_name(&component.name, limits)?;
        if let Some(pending) = &component.default_static { validate_pending(pending, limits)?; }
    }
    Ok(())
}

fn validate_nonempty_index_list<T>(values: &[T], limits: &ValidationLimits) -> Result<(), ArtifactError> {
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
        _ => return Err(ValidationError::ReferentialIntegrity("structural owner is not a builtin Universe identity".into()).into()),
    }
    if value.module.path.is_empty() || value.path.is_empty() {
        return Err(invalid("structural owner identity has an empty path"));
    }
    validate_nonempty_index_list(&value.module.path, limits)?;
    validate_nonempty_index_list(&value.path, limits)?;
    for segment in value.module.path.iter().chain(value.path.iter()) {
        if segment.is_empty() || segment.len() > limits.max_string_bytes { return Err(invalid("empty or oversized structural owner path segment")); }
    }
    Ok(())
}
