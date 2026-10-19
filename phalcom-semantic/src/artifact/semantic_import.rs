//! Reconstructs structural declaration products into current-main semantic tables.

use crate::algebraic_semantics::StaticAlgebraicDeclarationInfo;
use crate::artifact::identity::StableOwnerCatalog;
use crate::data_semantics::{
    DataComponentSemantic, DataConstructorParameter, DataConstructorParameterContribution, DataConstructorSignature, DataInfo, DataProductContribution,
    DataProductExpansionSemantic, DataProductShapeTemplate, DataSemanticTable, DataShape,
};
use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact, DeclaredTypeState};
use crate::enum_semantics::{
    AlgebraicCasePayloadKind, EnumInfo, EnumSemanticTable, VariantConstructorParameter, VariantConstructorSignature, VariantFieldSemantic, VariantInfo,
    VariantShape, VariantVisibility,
};
use crate::identity::{DataComponentId, DataConstructorId, DeclarationId, VariantConstructorId, VariantFamilyId, VariantFieldId, VariantId};
use crate::types::TypeStore;
use crate::types::case_environment::CaseTypeEnvironment;
use crate::types::evidence::{DynamicReason, UnknownReason};
use crate::types::id::{AlgebraicStaticConstructorId, KindId, StaticDomainId, TypeId, TypeParameterId};
use crate::types::parameter::{GenericConstraint, GenericParameterMode, GenericSignature, SelfRole, SelfTypeTerm, TypeParameterOwner, TypeTerm};
use crate::types::store::{StaticDomainData, TypeData};
use crate::types::variance::Variance;
use phalcom_ast::ast::ExpansionMode;
use phalcom_common::modifier::Visibility;
use phalcom_compiler_artifact::semantic::aggregates as wire;
use phalcom_compiler_artifact::semantic::aggregates::{
    DataProductContributionV1, DeclaredTypeBasisV1, DeclaredTypeStateV1, DynamicReasonV1, ExpansionModeV1, GenericConstraintV1, PayloadKindV1, TypeTermV1,
    UnknownReasonV1, VariantShapeV1, VisibilityV1,
};
use phalcom_compiler_artifact::semantic::type_store::{ArtifactStaticConstructorId, ArtifactStaticDomainId, ArtifactTypeId, ParameterOwnerV1, SelfRoleV1};
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::identity::StableDeclarationRef;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

/// Reconstructed native structural products carried by an imported image.
#[derive(Clone, Debug)]
pub struct ImportedSemanticAggregates {
    /// Current-main product declarations.
    pub data_semantics: Arc<DataSemanticTable>,
    /// Current-main enum roots and variant declarations.
    pub enum_semantics: Arc<EnumSemanticTable>,
}

pub(crate) fn reconstruct_structural_aggregates(
    contracts: &wire::StructuralPrimaryContractsV1,
    types: &super::type_import::ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ImportedSemanticAggregates, ArtifactError> {
    contracts.validate_with_allocation_budget(limits, allocations)?;
    let mut data_semantics = DataSemanticTable::new();
    for row in &contracts.data_contracts {
        let info = reconstruct_data_info(row, types, owners, allocations)?;
        if data_semantics.get(&info.owner).is_some() {
            return Err(referential("duplicate reconstructed data declaration owner"));
        }
        data_semantics.insert_data(Arc::new(info));
    }

    let mut enum_semantics = EnumSemanticTable::new();
    for row in &contracts.variant_contracts {
        let info = reconstruct_variant_info(row, types, owners, allocations)?;
        if enum_semantics.variant_info(&info.id).is_some() {
            return Err(referential("duplicate reconstructed enum variant owner"));
        }
        enum_semantics.insert_variant(Arc::new(info));
    }
    for row in &contracts.enum_contracts {
        let info = reconstruct_enum_info(row, &enum_semantics, types, owners, allocations)?;
        if enum_semantics.enum_info(&info.owner).is_some() {
            return Err(referential("duplicate reconstructed enum declaration owner"));
        }
        enum_semantics.insert_enum(Arc::new(info));
    }

    Ok(ImportedSemanticAggregates {
        data_semantics: Arc::new(data_semantics),
        enum_semantics: Arc::new(enum_semantics),
    })
}

fn reconstruct_data_info(
    row: &wire::DataInfoV1,
    types: &super::type_import::ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<DataInfo, ArtifactError> {
    let owner = declaration(&row.owner, owners)?;
    let root_form = type_id(row.root_form, types)?;
    validate_declaration_form(root_form, &owner, &types.store)?;
    let generic_signature = row
        .generic_signature
        .as_ref()
        .map(|signature| reconstruct_signature(signature, types, owners, allocations))
        .transpose()?;
    if generic_signature
        .as_ref()
        .is_some_and(|signature| signature.owner != TypeParameterOwner::Declaration(owner.clone()))
    {
        return Err(referential("data generic signature has a different declaration owner"));
    }
    let static_data = row
        .static_data
        .as_ref()
        .map(|static_data| reconstruct_static_data(static_data.domain, &static_data.constructors, &owner, types))
        .transpose()?;

    let mut components = allocations
        .vector::<DataComponentSemantic>(row.components.len())
        .map_err(ArtifactError::Allocation)?;
    let mut constructor_parameters = allocations
        .vector::<DataConstructorParameter>(row.constructor_parameters.len())
        .map_err(ArtifactError::Allocation)?;
    for (index, component) in row.components.iter().enumerate() {
        let component_index = u32::try_from(index).map_err(|_| referential("data component index exceeds u32"))?;
        let id = DataComponentId::new(owner.clone(), component_index);
        components.push(DataComponentSemantic {
            id: id.clone(),
            local_name: component.local_name.clone(),
            external_label: component.external_label.clone(),
            declared_type: reconstruct_declared_type(&component.declared_type, types, owners)?,
            source: None,
        });
        let parameter = row
            .constructor_parameters
            .get(index)
            .ok_or_else(|| referential("data constructor parameter row is missing"))?;
        constructor_parameters.push(DataConstructorParameter {
            component: id,
            external_label: parameter.external_label.clone(),
            local_name: parameter.local_name.clone(),
            declared_type: reconstruct_declared_type(&parameter.declared_type, types, owners)?,
        });
    }

    let shape = match row.shape {
        wire::DataShapeV1::Tuple => DataShape::Tuple,
        wire::DataShapeV1::Record => DataShape::Record,
    };
    let mut contributions = allocations
        .vector::<DataProductContribution>(row.shape_contributions.len())
        .map_err(ArtifactError::Allocation)?;
    let mut constructor_contributions = allocations
        .vector::<DataConstructorParameterContribution>(row.shape_contributions.len())
        .map_err(ArtifactError::Allocation)?;
    let mut component_index = 0usize;
    for contribution in &row.shape_contributions {
        match contribution {
            DataProductContributionV1::Component { component: _ } => {
                let component = components
                    .get(component_index)
                    .ok_or_else(|| referential("data component contribution has no declared component"))?
                    .clone();
                let parameter = constructor_parameters
                    .get(component_index)
                    .ok_or_else(|| referential("data component has no constructor parameter"))?
                    .clone();
                contributions.push(DataProductContribution::Component(component));
                constructor_contributions.push(DataConstructorParameterContribution::Parameter(parameter));
                component_index += 1;
            }
            DataProductContributionV1::Expansion { mode, operand } => {
                let expansion = DataProductExpansionSemantic {
                    mode: map_expansion_mode(mode.clone()),
                    operand: reconstruct_declared_type(operand, types, owners)?,
                    source: None,
                };
                contributions.push(DataProductContribution::Expansion(expansion.clone()));
                constructor_contributions.push(DataConstructorParameterContribution::Expansion(expansion));
            }
        }
    }
    if component_index != components.len() {
        return Err(referential("data shape contributions do not cover every component"));
    }
    let shape_template = DataProductShapeTemplate {
        shape,
        contributions: allocations.boxed_slice(contributions).map_err(ArtifactError::Allocation)?,
    };
    let constructor_template = if shape_template.has_expansions() {
        Some(crate::data_semantics::DataConstructorTemplate {
            constructor: DataConstructorId::new(owner.clone()),
            visibility: map_visibility(row.constructor_visibility.clone()),
            parameters: allocations.boxed_slice(constructor_contributions).map_err(ArtifactError::Allocation)?,
            result_type_template: type_id(row.constructor_result_type, types)?,
            source: None,
        })
    } else {
        None
    };
    let result_type_template = type_id(row.constructor_result_type, types)?;
    if types.store.kind_of(result_type_template) != KindId::TYPE || types.store.nominal_origin_declaration(result_type_template) != Some(&owner) {
        return Err(referential("data constructor result type does not belong to its declaration"));
    }
    Ok(DataInfo {
        owner: owner.clone(),
        root_form,
        generic_signature,
        static_data,
        shape,
        components: allocations.boxed_slice(components).map_err(ArtifactError::Allocation)?,
        constructor: DataConstructorSignature {
            constructor: DataConstructorId::new(owner),
            visibility: map_visibility(row.constructor_visibility.clone()),
            parameters: allocations.boxed_slice(constructor_parameters).map_err(ArtifactError::Allocation)?,
            result_type_template,
            source: None,
        },
        shape_template,
        constructor_template,
        source: None,
    })
}

fn reconstruct_enum_info(
    row: &wire::EnumInfoV1,
    variants: &EnumSemanticTable,
    types: &super::type_import::ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<EnumInfo, ArtifactError> {
    let owner = declaration(&row.owner, owners)?;
    let root_form = type_id(row.root_form, types)?;
    validate_declaration_form(root_form, &owner, &types.store)?;
    let generic_signature = row
        .generic_signature
        .as_ref()
        .map(|signature| reconstruct_signature(signature, types, owners, allocations))
        .transpose()?;
    if generic_signature
        .as_ref()
        .is_some_and(|signature| signature.owner != TypeParameterOwner::Declaration(owner.clone()))
    {
        return Err(referential("enum generic signature has a different declaration owner"));
    }
    let static_data = row
        .static_data
        .as_ref()
        .map(|static_data| reconstruct_static_data(static_data.domain, &static_data.constructors, &owner, types))
        .transpose()?;
    let default_result_type = type_id(row.default_result_type, types)?;
    if types.store.kind_of(default_result_type) != KindId::TYPE || types.store.nominal_origin_declaration(default_result_type) != Some(&owner) {
        return Err(referential("enum default result type does not belong to its declaration"));
    }
    let mut variant_ids = allocations.vector::<VariantId>(row.variants.len()).map_err(ArtifactError::Allocation)?;
    let mut variant_families = allocations.vector::<VariantFamilyId>(row.variants.len()).map_err(ArtifactError::Allocation)?;
    let mut seen_families = HashSet::new();
    for stable_variant in &row.variants {
        let variant = super::type_import::relocate_stable_variant(stable_variant, owners)?;
        if variant.owner != owner {
            return Err(referential("enum membership contains a foreign variant owner"));
        }
        if variants.variant_info(&variant).is_none() {
            return Err(referential("enum membership has no reconstructed variant contract"));
        }
        if let Some(family) = variant.family()
            && seen_families.insert(family.clone())
        {
            variant_families.push(family);
        }
        variant_ids.push(variant);
    }
    Ok(EnumInfo {
        owner,
        root_form,
        generic_signature,
        static_data,
        default_result_type,
        variants: allocations.boxed_slice(variant_ids).map_err(ArtifactError::Allocation)?,
        variant_families: allocations.boxed_slice(variant_families).map_err(ArtifactError::Allocation)?,
        source: None,
    })
}

fn reconstruct_variant_info(
    row: &wire::VariantInfoV1,
    types: &super::type_import::ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<VariantInfo, ArtifactError> {
    let id = super::type_import::relocate_stable_variant(&row.variant, owners)?;
    let type_handle = mapped(&types.variants, row.variant_type.0, "variant type")?;
    if types.store.variant_identity(type_handle) != &id {
        return Err(referential("variant type row disagrees with its stable identity"));
    }
    let result_type_template = type_id(row.result_type, types)?;
    let exact_case_template = type_id(row.exact_case_type, types)?;
    if types.store.kind_of(result_type_template) != KindId::TYPE || types.store.nominal_origin_declaration(result_type_template) != Some(&id.owner) {
        return Err(referential("variant result type does not belong to its enum declaration"));
    }
    match types.store.get(exact_case_template) {
        TypeData::ExactCase { variant, .. } if types.store.variant_identity(*variant) == &id => {}
        _ => return Err(referential("variant exact-case template disagrees with its stable identity")),
    }

    let mut fields = allocations
        .vector::<VariantFieldSemantic>(row.fields.len())
        .map_err(ArtifactError::Allocation)?;
    for (index, field) in row.fields.iter().enumerate() {
        fields.push(VariantFieldSemantic {
            id: VariantFieldId::new(id.clone(), u32::try_from(index).map_err(|_| referential("variant field index exceeds u32"))?),
            local_name: field.local_name.clone(),
            external_label: field.external_label.clone(),
            declared_type: reconstruct_declared_type(&field.declared_type, types, owners)?,
            source: None,
        });
    }
    let mut bindings = BTreeMap::new();
    for binding in &row.case_environment.bindings {
        let parameter = mapped(&types.parameters, binding.parameter.0, "case environment parameter")?;
        let ty = type_id(binding.ty, types)?;
        if bindings.insert(parameter, ty).is_some() {
            return Err(referential("duplicate relocated case environment parameter"));
        }
    }
    let mut equalities = allocations
        .vector::<GenericConstraint>(row.case_environment.equalities.len())
        .map_err(ArtifactError::Allocation)?;
    for equality in &row.case_environment.equalities {
        equalities.push(reconstruct_constraint(equality, types, owners, allocations)?);
    }

    let constructor = row
        .constructor
        .as_ref()
        .map(|constructor| reconstruct_variant_constructor(constructor, &id, &fields, types, owners, allocations))
        .transpose()?;
    let family = id.family();
    Ok(VariantInfo {
        id,
        type_handle,
        family,
        shape: match row.shape {
            VariantShapeV1::Singleton => VariantShape::Singleton,
            VariantShapeV1::Constructor => VariantShape::Constructor,
        },
        payload_kind: match row.payload_kind {
            PayloadKindV1::Singleton => AlgebraicCasePayloadKind::Singleton,
            PayloadKindV1::Tuple => AlgebraicCasePayloadKind::Tuple,
            PayloadKindV1::Record => AlgebraicCasePayloadKind::Record,
            PayloadKindV1::ExplicitEmpty => AlgebraicCasePayloadKind::ExplicitEmpty,
        },
        fields: allocations.boxed_slice(fields).map_err(ArtifactError::Allocation)?,
        result_type_template,
        exact_case_template,
        case_environment: CaseTypeEnvironment::new(bindings, allocations.boxed_slice(equalities).map_err(ArtifactError::Allocation)?),
        constructor,
        visibility: VariantVisibility {
            name: map_visibility(row.visibility.name.clone()),
            construct: map_visibility(row.visibility.construct.clone()),
            payload: map_visibility(row.visibility.payload.clone()),
        },
        source: None,
    })
}

fn reconstruct_variant_constructor(
    row: &wire::VariantConstructorV1,
    variant: &VariantId,
    fields: &[VariantFieldSemantic],
    types: &super::type_import::ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<VariantConstructorSignature, ArtifactError> {
    let constructor_id = VariantConstructorId::new(variant.clone());
    let callable = crate::identity::CallableId::variant_constructor(variant.clone());
    let generic_signature = row
        .generic_signature
        .as_ref()
        .map(|signature| reconstruct_signature(signature, types, owners, allocations))
        .transpose()?;
    if generic_signature
        .as_ref()
        .is_some_and(|signature| signature.owner != TypeParameterOwner::Callable(callable))
    {
        return Err(referential("variant constructor generic signature has a different callable owner"));
    }
    if row.parameters.len() != fields.len() {
        return Err(referential("variant constructor parameter count disagrees with its fields"));
    }
    let mut parameters = allocations
        .vector::<VariantConstructorParameter>(row.parameters.len())
        .map_err(ArtifactError::Allocation)?;
    for (index, parameter) in row.parameters.iter().enumerate() {
        let field = fields.get(index).ok_or_else(|| referential("variant constructor parameter has no field"))?;
        parameters.push(VariantConstructorParameter {
            field: field.id.clone(),
            external_label: parameter.external_label.clone(),
            local_name: parameter.local_name.clone(),
            declared_type: reconstruct_declared_type(&parameter.declared_type, types, owners)?,
        });
    }
    let result_type_template = type_id(row.result_type, types)?;
    let exact_case_template = type_id(row.exact_case_type, types)?;
    if types.store.kind_of(result_type_template) != KindId::TYPE || types.store.nominal_origin_declaration(result_type_template) != Some(&variant.owner) {
        return Err(referential("variant constructor result type does not belong to its enum declaration"));
    }
    match types.store.get(exact_case_template) {
        TypeData::ExactCase { variant: exact, .. } if types.store.variant_identity(*exact) == variant => {}
        _ => return Err(referential("variant constructor exact-case result disagrees with its stable identity")),
    }
    Ok(VariantConstructorSignature {
        constructor: constructor_id,
        generic_signature,
        parameters: allocations.boxed_slice(parameters).map_err(ArtifactError::Allocation)?,
        result_type_template,
        exact_case_template,
        source: None,
    })
}

fn reconstruct_static_data(
    domain_id: ArtifactStaticDomainId,
    constructors: &[ArtifactStaticConstructorId],
    owner: &DeclarationId,
    types: &super::type_import::ReconstructedTypeGraph,
) -> Result<StaticAlgebraicDeclarationInfo, ArtifactError> {
    let domain: StaticDomainId = mapped(&types.static_domains, domain_id.0, "static domain")?;
    match types.store.static_domain(domain) {
        Some(StaticDomainData::Nominal { declaration }) if declaration == owner => {}
        _ => return Err(referential("static algebraic domain does not belong to its declaration")),
    }
    let mut relocated = Vec::with_capacity(constructors.len());
    let mut seen = HashSet::new();
    for constructor in constructors {
        let id: AlgebraicStaticConstructorId = mapped(&types.static_constructors, constructor.0, "static constructor")?;
        let Some(data) = types.store.static_constructor(id) else {
            return Err(referential("static algebraic constructor is missing from the live type store"));
        };
        if data.domain != domain || &data.declaration != owner || !seen.insert(id) {
            return Err(referential("static algebraic constructor identity or domain mismatch"));
        }
        relocated.push(id);
    }
    Ok(StaticAlgebraicDeclarationInfo {
        domain,
        constructors: relocated.into_boxed_slice(),
    })
}

pub(crate) fn reconstruct_signature(
    row: &wire::GenericSignatureV1,
    types: &super::type_import::ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<GenericSignature, ArtifactError> {
    let owner = match &row.owner {
        ParameterOwnerV1::Declaration { owner } => TypeParameterOwner::Declaration(declaration(owner, owners)?),
        ParameterOwnerV1::Callable { owner } => TypeParameterOwner::Callable(super::type_import::relocate_stable_callable(owner, owners)?),
        ParameterOwnerV1::Impl { owner } => TypeParameterOwner::Impl(super::type_import::relocate_stable_impl(owner, owners)?),
    };
    let mut parameters = allocations.vector::<TypeParameterId>(row.parameters.len()).map_err(ArtifactError::Allocation)?;
    for parameter in &row.parameters {
        parameters.push(mapped(&types.parameters, parameter.0, "generic signature parameter")?);
    }
    let mut parameter_kinds = allocations.vector::<KindId>(row.parameter_kinds.len()).map_err(ArtifactError::Allocation)?;
    for kind in &row.parameter_kinds {
        parameter_kinds.push(mapped(&types.kinds, kind.0, "generic signature parameter kind")?);
    }
    let mut parameter_variances = allocations
        .vector::<Variance>(row.parameter_variances.len())
        .map_err(ArtifactError::Allocation)?;
    for variance in &row.parameter_variances {
        parameter_variances.push(match variance {
            phalcom_compiler_artifact::semantic::type_store::VarianceV1::Covariant => Variance::Covariant,
            phalcom_compiler_artifact::semantic::type_store::VarianceV1::Contravariant => Variance::Contravariant,
            phalcom_compiler_artifact::semantic::type_store::VarianceV1::Invariant => Variance::Invariant,
        });
    }
    let mut parameter_modes = allocations
        .vector::<GenericParameterMode>(row.parameter_modes.len())
        .map_err(ArtifactError::Allocation)?;
    for mode in &row.parameter_modes {
        parameter_modes.push(match mode {
            phalcom_compiler_artifact::semantic::type_store::ParameterModeV1::Single => GenericParameterMode::Single,
            phalcom_compiler_artifact::semantic::type_store::ParameterModeV1::PositionalPack => GenericParameterMode::PositionalPack,
            phalcom_compiler_artifact::semantic::type_store::ParameterModeV1::LabeledPack => GenericParameterMode::LabeledPack,
            phalcom_compiler_artifact::semantic::type_store::ParameterModeV1::CompletePack => GenericParameterMode::CompletePack,
        });
    }
    let mut parameter_kind_shapes = allocations
        .vector::<Box<str>>(row.parameter_kind_shapes.len())
        .map_err(ArtifactError::Allocation)?;
    for shape in &row.parameter_kind_shapes {
        parameter_kind_shapes.push(allocations.boxed_str(shape).map_err(ArtifactError::Allocation)?);
    }
    let mut constraint_shapes = allocations
        .vector::<Box<str>>(row.constraint_shapes.len())
        .map_err(ArtifactError::Allocation)?;
    for shape in &row.constraint_shapes {
        constraint_shapes.push(allocations.boxed_str(shape).map_err(ArtifactError::Allocation)?);
    }
    let mut constraints = allocations
        .vector::<GenericConstraint>(row.constraints.len())
        .map_err(ArtifactError::Allocation)?;
    for constraint in &row.constraints {
        constraints.push(reconstruct_constraint(constraint, types, owners, allocations)?);
    }
    let signature = GenericSignature {
        owner,
        parameters: allocations.boxed_slice(parameters).map_err(ArtifactError::Allocation)?,
        parameter_kinds: allocations.boxed_slice(parameter_kinds).map_err(ArtifactError::Allocation)?,
        parameter_kind_shapes: allocations.boxed_slice(parameter_kind_shapes).map_err(ArtifactError::Allocation)?,
        parameter_variances: allocations.boxed_slice(parameter_variances).map_err(ArtifactError::Allocation)?,
        parameter_modes: allocations.boxed_slice(parameter_modes).map_err(ArtifactError::Allocation)?,
        constraint_shapes: allocations.boxed_slice(constraint_shapes).map_err(ArtifactError::Allocation)?,
        constraints: allocations.boxed_slice(constraints).map_err(ArtifactError::Allocation)?,
    };
    for (index, parameter) in signature.parameters.iter().enumerate() {
        let data = types.store.type_parameter(*parameter);
        if data.owner != signature.owner || data.index != index as u32 {
            return Err(referential("generic signature parameter has a different owner or order"));
        }
        if signature.parameter_kinds.get(index).is_some_and(|kind| *kind != data.kind)
            || signature.parameter_variances.get(index).is_some_and(|variance| *variance != data.variance)
            || signature.parameter_modes.get(index).is_some_and(|mode| *mode != data.mode)
        {
            return Err(referential("generic signature parameter metadata disagrees with the relocated type parameter"));
        }
    }
    signature
        .validate_publishable(&types.store)
        .map_err(|error| referential(format!("generic signature is not publishable: {error:?}")))?;
    Ok(signature)
}

fn reconstruct_constraint(
    row: &GenericConstraintV1,
    types: &super::type_import::ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<GenericConstraint, ArtifactError> {
    Ok(match row {
        GenericConstraintV1::Subtype { lower, upper } => GenericConstraint::Subtype {
            lower: reconstruct_term(lower, types, owners)?,
            upper: reconstruct_term(upper, types, owners)?,
        },
        GenericConstraintV1::Equivalent { left, right } => GenericConstraint::Equivalent {
            left: reconstruct_term(left, types, owners)?,
            right: reconstruct_term(right, types, owners)?,
        },
        GenericConstraintV1::Conforms { subject, trait_ref } => {
            let mut arguments = allocations.vector::<TypeId>(trait_ref.arguments.len()).map_err(ArtifactError::Allocation)?;
            for argument in &trait_ref.arguments {
                arguments.push(type_id(*argument, types)?);
            }
            GenericConstraint::Conforms {
                subject: reconstruct_term(subject, types, owners)?,
                trait_ref: crate::traits::TraitRef::new(
                    declaration(&trait_ref.declaration, owners)?,
                    allocations.boxed_slice(arguments).map_err(ArtifactError::Allocation)?,
                ),
            }
        }
    })
}

fn reconstruct_term(row: &TypeTermV1, types: &super::type_import::ReconstructedTypeGraph, owners: &StableOwnerCatalog<'_>) -> Result<TypeTerm, ArtifactError> {
    Ok(match row {
        TypeTermV1::Canonical { ty } => TypeTerm::Canonical(type_id(*ty, types)?),
        TypeTermV1::SelfType { term } => TypeTerm::SelfType(SelfTypeTerm {
            owner: declaration(&term.owner, owners)?,
            side: super::identity::dispatch_side(term.side),
            role: match term.role {
                SelfRoleV1::InstanceType => SelfRole::InstanceType,
                SelfRoleV1::ReceiverValue => SelfRole::ReceiverValue,
            },
        }),
    })
}

fn reconstruct_declared_type(
    row: &wire::DeclaredTypeFactV1,
    types: &super::type_import::ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
) -> Result<DeclaredTypeFact, ArtifactError> {
    let state = match &row.state {
        DeclaredTypeStateV1::Known { term } => DeclaredTypeState::Known(reconstruct_term(term, types, owners)?),
        DeclaredTypeStateV1::Dynamic { reason } => DeclaredTypeState::Dynamic(match reason {
            DynamicReasonV1::ExplicitEscape => DynamicReason::ExplicitEscape,
            DynamicReasonV1::DynamicRestPack => DynamicReason::DynamicRestPack,
            DynamicReasonV1::RuntimeReflection => DynamicReason::RuntimeReflection,
        }),
        DeclaredTypeStateV1::Unknown { reason } => DeclaredTypeState::Unknown(match reason {
            UnknownReasonV1::NoTypeEvidence => UnknownReason::NoTypeEvidence,
            UnknownReasonV1::MissingInitializer => UnknownReason::MissingInitializer,
            UnknownReasonV1::UnannotatedDeclaration => UnknownReason::UnannotatedDeclaration,
            UnknownReasonV1::UnresolvedName { name } => UnknownReason::UnresolvedName(name.clone()),
            UnknownReasonV1::DynamicMessageSend => UnknownReason::DynamicMessageSend,
            UnknownReasonV1::OpaqueNative => UnknownReason::OpaqueNative,
            UnknownReasonV1::RecursiveFixpoint => UnknownReason::RecursiveFixpoint,
            UnknownReasonV1::UncheckedExpression => UnknownReason::UncheckedExpression,
            UnknownReasonV1::SyntaxError => UnknownReason::SyntaxError,
            UnknownReasonV1::UnderconstrainedTypeVariable => UnknownReason::UnderconstrainedTypeVariable,
            UnknownReasonV1::InferenceConflict => UnknownReason::InferenceConflict,
            UnknownReasonV1::InferenceAmbiguous => UnknownReason::InferenceAmbiguous,
            UnknownReasonV1::InferenceBlocked => UnknownReason::InferenceBlocked,
            UnknownReasonV1::InferenceCancelled => UnknownReason::InferenceCancelled,
            UnknownReasonV1::InferenceBudgetExceeded => UnknownReason::InferenceBudgetExceeded,
            UnknownReasonV1::SuppressedByInvalidCause => UnknownReason::SuppressedByInvalidCause,
            UnknownReasonV1::ExistentialEscape => UnknownReason::ExistentialEscape,
        }),
    };
    Ok(DeclaredTypeFact {
        state,
        basis: match row.basis {
            DeclaredTypeBasisV1::Unspecified => DeclaredTypeBasis::Unspecified,
            DeclaredTypeBasisV1::SourceAnnotation => DeclaredTypeBasis::SourceAnnotation,
            DeclaredTypeBasisV1::NativeSignature => DeclaredTypeBasis::NativeSignature,
            DeclaredTypeBasisV1::DeclarationSemantics => DeclaredTypeBasis::DeclarationSemantics,
            DeclaredTypeBasisV1::ConstructorSemantics => DeclaredTypeBasis::ConstructorSemantics,
            DeclaredTypeBasisV1::InitializerInference => DeclaredTypeBasis::InitializerInference,
            DeclaredTypeBasisV1::BodyInference => DeclaredTypeBasis::BodyInference,
            DeclaredTypeBasisV1::ContextualTyping => DeclaredTypeBasis::ContextualTyping,
            DeclaredTypeBasisV1::PatternDecomposition => DeclaredTypeBasis::PatternDecomposition,
        },
    })
}

fn validate_declaration_form(form: TypeId, owner: &DeclarationId, store: &TypeStore) -> Result<(), ArtifactError> {
    let matches_owner = match store.get(form) {
        TypeData::StaticDomainForm { domain } => {
            matches!(store.static_domain(*domain), Some(StaticDomainData::Nominal { declaration }) if declaration == owner)
        }
        _ => store.nominal_origin_declaration(form) == Some(owner),
    };
    if matches_owner {
        Ok(())
    } else {
        Err(referential("structural declaration root form belongs to another owner"))
    }
}

fn type_id(id: ArtifactTypeId, types: &super::type_import::ReconstructedTypeGraph) -> Result<TypeId, ArtifactError> {
    mapped(&types.types, id.0, "type")
}

fn declaration(id: &StableDeclarationRef, owners: &StableOwnerCatalog<'_>) -> Result<DeclarationId, ArtifactError> {
    owners.declaration(id)
}

fn mapped<T: Copy>(values: &[T], raw: u32, family: &str) -> Result<T, ArtifactError> {
    values
        .get(raw as usize)
        .copied()
        .ok_or_else(|| referential(format!("invalid {family} reference {raw}")))
}

fn map_expansion_mode(mode: ExpansionModeV1) -> ExpansionMode {
    match mode {
        ExpansionModeV1::Positional => ExpansionMode::Positional,
        ExpansionModeV1::Labeled => ExpansionMode::Labeled,
        ExpansionModeV1::Complete => ExpansionMode::Complete,
    }
}

fn map_visibility(visibility: VisibilityV1) -> Visibility {
    match visibility {
        VisibilityV1::Public => Visibility::Public,
        VisibilityV1::Private => Visibility::Private,
        VisibilityV1::Protected => Visibility::Protected,
        VisibilityV1::Module => Visibility::Module,
        VisibilityV1::Package => Visibility::Package,
        VisibilityV1::Internal => Visibility::Internal,
    }
}

fn referential(message: impl Into<String>) -> ArtifactError {
    ArtifactError::Validation(ValidationError::ReferentialIntegrity(message.into()))
}
