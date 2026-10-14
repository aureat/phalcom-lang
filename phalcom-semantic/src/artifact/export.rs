//! Semantic-owned artifact export for structural primary contracts (`PERF003.C2.P7.T7.A`).

use crate::data_semantics::{DataComponentSemantic, DataConstructorParameter, DataProductContribution, DataSemanticTable, DataShape};
use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact, DeclaredTypeState};
use crate::declarations::DeclarationTypeTable;
use crate::enum_semantics::{AlgebraicCasePayloadKind, EnumSemanticTable, VariantFieldSemantic, VariantShape, VariantVisibility};
use crate::metadata::stable_identity::{
    to_stable_callable, to_stable_declaration, to_stable_dispatch_side, to_stable_impl, to_stable_variant,
};
use crate::type_alias::TypeAliasTable;
use crate::types::case_environment::CaseTypeEnvironment;
use crate::types::evidence::{DynamicReason, UnknownReason};
use crate::types::parameter::{GenericConstraint, GenericParameterMode, GenericSignature, SelfRole, TypeParameterOwner, TypeTerm};
use crate::types::relation::MapTypeHierarchy;
use crate::types::store::artifact::TypeExportContext;
use crate::types::variance::Variance;
use phalcom_common::modifier::Visibility;
use phalcom_compiler_artifact::semantic::aggregates::*;
use phalcom_compiler_artifact::semantic::type_store::{
    CanonicalRootFamilyTag, CanonicalTypeRootKey, ParameterModeV1, ParameterOwnerV1, SelfRoleV1, SelfTypeV1,
    StableOwnerRef, VarianceV1,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use std::collections::BTreeSet;

fn referential(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}

pub fn export_parameter_owner(owner: &TypeParameterOwner) -> Result<ParameterOwnerV1, ArtifactError> {
    Ok(match owner {
        TypeParameterOwner::Declaration(decl) => ParameterOwnerV1::Declaration {
            owner: to_stable_declaration(decl),
        },
        TypeParameterOwner::Callable(callable) => ParameterOwnerV1::Callable {
            owner: to_stable_callable(callable),
        },
        TypeParameterOwner::Impl(r#impl) => ParameterOwnerV1::Impl {
            owner: to_stable_impl(r#impl),
        },
    })
}

pub fn export_variance(variance: Variance) -> VarianceV1 {
    match variance {
        Variance::Covariant => VarianceV1::Covariant,
        Variance::Contravariant => VarianceV1::Contravariant,
        Variance::Invariant => VarianceV1::Invariant,
    }
}

pub fn export_mode(mode: GenericParameterMode) -> ParameterModeV1 {
    match mode {
        GenericParameterMode::Single => ParameterModeV1::Single,
        GenericParameterMode::PositionalPack => ParameterModeV1::PositionalPack,
        GenericParameterMode::LabeledPack => ParameterModeV1::LabeledPack,
        GenericParameterMode::CompletePack => ParameterModeV1::CompletePack,
    }
}

pub fn export_self_role(role: SelfRole) -> SelfRoleV1 {
    match role {
        SelfRole::InstanceType => SelfRoleV1::InstanceType,
        SelfRole::ReceiverValue => SelfRoleV1::ReceiverValue,
    }
}

pub fn export_type_term(term: &TypeTerm, types: &TypeExportContext) -> Result<TypeTermV1, ArtifactError> {
    match term {
        TypeTerm::Canonical(ty) => Ok(TypeTermV1::Canonical { ty: types.ty(*ty)? }),
        TypeTerm::SelfType(term) => Ok(TypeTermV1::SelfType {
            term: SelfTypeV1 {
                owner: to_stable_declaration(&term.owner),
                side: to_stable_dispatch_side(term.side),
                role: export_self_role(term.role),
            },
        }),
        TypeTerm::Infer(_) => Err(referential("inference variables cannot be published to artifact")),
    }
}

pub fn export_constraint(
    constraint: &GenericConstraint,
    types: &TypeExportContext,
) -> Result<GenericConstraintV1, ArtifactError> {
    Ok(match constraint {
        GenericConstraint::Subtype { lower, upper } => GenericConstraintV1::Subtype {
            lower: export_type_term(lower, types)?,
            upper: export_type_term(upper, types)?,
        },
        GenericConstraint::Equivalent { left, right } => GenericConstraintV1::Equivalent {
            left: export_type_term(left, types)?,
            right: export_type_term(right, types)?,
        },
        GenericConstraint::Conforms { subject, trait_ref } => GenericConstraintV1::Conforms {
            subject: export_type_term(subject, types)?,
            trait_ref: TraitRefV1 {
                declaration: to_stable_declaration(&trait_ref.declaration),
                arguments: trait_ref
                    .arguments
                    .iter()
                    .map(|&arg| types.ty(arg))
                    .collect::<Result<Vec<_>, _>>()?,
            },
        },
    })
}

pub fn export_generic_signature(
    sig: &GenericSignature,
    types: &TypeExportContext,
) -> Result<GenericSignatureV1, ArtifactError> {
    let owner = export_parameter_owner(&sig.owner)?;
    let parameters = sig
        .parameters
        .iter()
        .map(|&p| types.parameter(p))
        .collect::<Result<Vec<_>, _>>()?;
    let parameter_kinds = sig
        .parameter_kinds
        .iter()
        .map(|&k| types.kind(k))
        .collect::<Result<Vec<_>, _>>()?;
    let parameter_kind_shapes = sig.parameter_kind_shapes.to_vec();
    let parameter_variances = sig.parameter_variances.iter().map(|&v| export_variance(v)).collect();
    let parameter_modes = sig.parameter_modes.iter().map(|&m| export_mode(m)).collect();
    let constraint_shapes = sig.constraint_shapes.to_vec();
    let constraints = sig
        .constraints
        .iter()
        .map(|c| export_constraint(c, types))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(GenericSignatureV1 {
        owner,
        parameters,
        parameter_kinds,
        parameter_kind_shapes,
        parameter_variances,
        parameter_modes,
        constraint_shapes,
        constraints,
    })
}

pub fn export_declared_type_state(
    state: &DeclaredTypeState,
    types: &TypeExportContext,
) -> Result<DeclaredTypeStateV1, ArtifactError> {
    Ok(match state {
        DeclaredTypeState::Known(term) => DeclaredTypeStateV1::Known {
            term: export_type_term(term, types)?,
        },
        DeclaredTypeState::Dynamic(reason) => DeclaredTypeStateV1::Dynamic {
            reason: match reason {
                DynamicReason::ExplicitEscape => DynamicReasonV1::ExplicitEscape,
                DynamicReason::DynamicRestPack => DynamicReasonV1::DynamicRestPack,
                DynamicReason::RuntimeReflection => DynamicReasonV1::RuntimeReflection,
            },
        },
        DeclaredTypeState::Unknown(reason) => DeclaredTypeStateV1::Unknown {
            reason: match reason {
                UnknownReason::NoTypeEvidence => UnknownReasonV1::NoTypeEvidence,
                UnknownReason::MissingInitializer => UnknownReasonV1::MissingInitializer,
                UnknownReason::UnannotatedDeclaration => UnknownReasonV1::UnannotatedDeclaration,
                UnknownReason::UnresolvedName(name) => UnknownReasonV1::UnresolvedName { name: name.clone() },
                UnknownReason::DynamicMessageSend => UnknownReasonV1::DynamicMessageSend,
                UnknownReason::OpaqueNative => UnknownReasonV1::OpaqueNative,
                UnknownReason::RecursiveFixpoint => UnknownReasonV1::RecursiveFixpoint,
                UnknownReason::UncheckedExpression => UnknownReasonV1::UncheckedExpression,
                UnknownReason::SyntaxError => UnknownReasonV1::SyntaxError,
                UnknownReason::UnderconstrainedTypeVariable => UnknownReasonV1::UnderconstrainedTypeVariable,
                UnknownReason::InferenceConflict => UnknownReasonV1::InferenceConflict,
                UnknownReason::InferenceAmbiguous => UnknownReasonV1::InferenceAmbiguous,
                UnknownReason::InferenceBlocked => UnknownReasonV1::InferenceBlocked,
                UnknownReason::InferenceCancelled => UnknownReasonV1::InferenceCancelled,
                UnknownReason::InferenceBudgetExceeded => UnknownReasonV1::InferenceBudgetExceeded,
                UnknownReason::SuppressedByInvalidCause => UnknownReasonV1::SuppressedByInvalidCause,
                UnknownReason::ExistentialEscape => UnknownReasonV1::ExistentialEscape,
            },
        },
    })
}

pub fn export_declared_type_basis(basis: DeclaredTypeBasis) -> DeclaredTypeBasisV1 {
    match basis {
        DeclaredTypeBasis::Unspecified => DeclaredTypeBasisV1::Unspecified,
        DeclaredTypeBasis::SourceAnnotation => DeclaredTypeBasisV1::SourceAnnotation,
        DeclaredTypeBasis::NativeSignature => DeclaredTypeBasisV1::NativeSignature,
        DeclaredTypeBasis::DeclarationSemantics => DeclaredTypeBasisV1::DeclarationSemantics,
        DeclaredTypeBasis::ConstructorSemantics => DeclaredTypeBasisV1::ConstructorSemantics,
        DeclaredTypeBasis::InitializerInference => DeclaredTypeBasisV1::InitializerInference,
        DeclaredTypeBasis::BodyInference => DeclaredTypeBasisV1::BodyInference,
        DeclaredTypeBasis::ContextualTyping => DeclaredTypeBasisV1::ContextualTyping,
        DeclaredTypeBasis::PatternDecomposition => DeclaredTypeBasisV1::PatternDecomposition,
    }
}

pub fn export_declared_type_fact(
    fact: &DeclaredTypeFact,
    types: &TypeExportContext,
) -> Result<DeclaredTypeFactV1, ArtifactError> {
    Ok(DeclaredTypeFactV1 {
        state: export_declared_type_state(&fact.state, types)?,
        basis: export_declared_type_basis(fact.basis),
    })
}

pub fn export_visibility(visibility: Visibility) -> VisibilityV1 {
    match visibility {
        Visibility::Public => VisibilityV1::Public,
        Visibility::Private => VisibilityV1::Private,
        Visibility::Protected => VisibilityV1::Protected,
        Visibility::Module => VisibilityV1::Module,
        Visibility::Package => VisibilityV1::Package,
        Visibility::Internal => VisibilityV1::Internal,
    }
}

pub fn export_data_shape(shape: DataShape) -> DataShapeV1 {
    match shape {
        DataShape::Tuple => DataShapeV1::Tuple,
        DataShape::Record => DataShapeV1::Record,
    }
}

pub fn export_variant_shape(shape: VariantShape) -> VariantShapeV1 {
    match shape {
        VariantShape::Singleton => VariantShapeV1::Singleton,
        VariantShape::Constructor => VariantShapeV1::Constructor,
    }
}

pub fn export_payload_kind(kind: AlgebraicCasePayloadKind) -> PayloadKindV1 {
    match kind {
        AlgebraicCasePayloadKind::Singleton => PayloadKindV1::Singleton,
        AlgebraicCasePayloadKind::Tuple => PayloadKindV1::Tuple,
        AlgebraicCasePayloadKind::Record => PayloadKindV1::Record,
        AlgebraicCasePayloadKind::ExplicitEmpty => PayloadKindV1::ExplicitEmpty,
    }
}

pub fn export_expansion_mode(mode: phalcom_ast::ast::ExpansionMode) -> ExpansionModeV1 {
    match mode {
        phalcom_ast::ast::ExpansionMode::Positional => ExpansionModeV1::Positional,
        phalcom_ast::ast::ExpansionMode::Labeled => ExpansionModeV1::Labeled,
        phalcom_ast::ast::ExpansionMode::Complete => ExpansionModeV1::Complete,
    }
}

pub fn export_static_data(
    static_data: &crate::algebraic_semantics::StaticAlgebraicDeclarationInfo,
    types: &TypeExportContext,
) -> Result<StaticAlgebraicDeclarationV1, ArtifactError> {
    Ok(StaticAlgebraicDeclarationV1 {
        domain: types.domain(static_data.domain)?,
        constructors: static_data
            .constructors
            .iter()
            .map(|&c| types.constructor(c))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

pub fn export_case_environment(
    env: &CaseTypeEnvironment,
    types: &TypeExportContext,
) -> Result<CaseEnvironmentV1, ArtifactError> {
    let mut bindings = Vec::with_capacity(env.bindings.len());
    for (&param, &ty) in &env.bindings {
        bindings.push(CaseBindingV1 {
            parameter: types.parameter(param)?,
            ty: types.ty(ty)?,
        });
    }
    bindings.sort_by_key(|b| b.parameter.0);

    let equalities = env
        .equalities
        .iter()
        .map(|eq| export_constraint(eq, types))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CaseEnvironmentV1 { bindings, equalities })
}

pub fn export_data_component(
    comp: &DataComponentSemantic,
    types: &TypeExportContext,
) -> Result<DataComponentSemanticV1, ArtifactError> {
    Ok(DataComponentSemanticV1 {
        local_name: comp.local_name.clone(),
        external_label: comp.external_label.clone(),
        declared_type: export_declared_type_fact(&comp.declared_type, types)?,
    })
}

pub fn export_data_constructor_parameter(
    param: &DataConstructorParameter,
    types: &TypeExportContext,
) -> Result<DataConstructorParameterV1, ArtifactError> {
    Ok(DataConstructorParameterV1 {
        external_label: param.external_label.clone(),
        local_name: param.local_name.clone(),
        declared_type: export_declared_type_fact(&param.declared_type, types)?,
    })
}

pub fn export_shape_contribution(
    contribution: &DataProductContribution,
    types: &TypeExportContext,
) -> Result<DataProductContributionV1, ArtifactError> {
    Ok(match contribution {
        DataProductContribution::Component(comp) => {
            DataProductContributionV1::Component {
                component: export_data_component(comp, types)?,
            }
        }
        DataProductContribution::Expansion(exp) => {
            DataProductContributionV1::Expansion {
                mode: export_expansion_mode(exp.mode),
                operand: export_declared_type_fact(&exp.operand, types)?,
            }
        }
    })
}

pub fn export_variant_field(
    field: &VariantFieldSemantic,
    types: &TypeExportContext,
) -> Result<VariantFieldV1, ArtifactError> {
    Ok(VariantFieldV1 {
        local_name: field.local_name.clone(),
        external_label: field.external_label.clone(),
        declared_type: export_declared_type_fact(&field.declared_type, types)?,
    })
}

pub fn export_variant_constructor_parameter(
    param: &crate::enum_semantics::VariantConstructorParameter,
    types: &TypeExportContext,
) -> Result<VariantConstructorParameterV1, ArtifactError> {
    Ok(VariantConstructorParameterV1 {
        external_label: param.external_label.clone(),
        local_name: param.local_name.clone(),
        declared_type: export_declared_type_fact(&param.declared_type, types)?,
    })
}

pub fn export_variant_visibility(v: &VariantVisibility) -> VariantVisibilityV1 {
    VariantVisibilityV1 {
        name: export_visibility(v.name),
        construct: export_visibility(v.construct),
        payload: export_visibility(v.payload),
    }
}

/// Registers roots required by structural primary contracts into `TypeExportContext`
/// prior to graph closure.
pub fn register_structural_primary_roots(
    types: &mut TypeExportContext,
    data_table: &DataSemanticTable,
    enum_table: &EnumSemanticTable,
    declarations: &DeclarationTypeTable,
    hierarchy: &MapTypeHierarchy,
    type_aliases: &TypeAliasTable,
) -> Result<(), ArtifactError> {
    let lineage = types.expected_lineage();

    for (decl_id, info) in declarations.iter() {
        let key = CanonicalTypeRootKey {
            family_tag: CanonicalRootFamilyTag::DeclarationShell,
            owner: StableOwnerRef::Declaration(to_stable_declaration(decl_id)),
            product_subkey: 0,
        };
        types.register_type_root(lineage, key, info.form)?;
    }

    for (decl_id, info) in type_aliases.iter() {
        let key = CanonicalTypeRootKey {
            family_tag: CanonicalRootFamilyTag::HierarchyAndAlias,
            owner: StableOwnerRef::Declaration(to_stable_declaration(decl_id)),
            product_subkey: 0,
        };
        types.register_type_root(lineage, key, info.form)?;
    }

    for (decl_id, _) in hierarchy.iter_superclasses() {
        if let Some(template) = hierarchy.supertype_template(decl_id) {
            let key = CanonicalTypeRootKey {
                family_tag: CanonicalRootFamilyTag::HierarchyAndAlias,
                owner: StableOwnerRef::Declaration(to_stable_declaration(decl_id)),
                product_subkey: 1,
            };
            types.register_type_root(lineage, key, template.supertype)?;
        }
    }

    for (decl_id, info) in data_table.data_decls.iter() {
        let owner = StableOwnerRef::Declaration(to_stable_declaration(decl_id));
        let key = CanonicalTypeRootKey {
            family_tag: CanonicalRootFamilyTag::DataContract,
            owner,
            product_subkey: 0,
        };
        types.register_type_root(lineage, key, info.root_form)?;
    }

    for (decl_id, info) in enum_table.enums.iter() {
        let owner = StableOwnerRef::Declaration(to_stable_declaration(decl_id));
        let key = CanonicalTypeRootKey {
            family_tag: CanonicalRootFamilyTag::EnumContract,
            owner,
            product_subkey: 0,
        };
        types.register_type_root(lineage, key, info.root_form)?;
    }

    for (var_id, info) in enum_table.variants.iter() {
        let owner = StableOwnerRef::Variant(to_stable_variant(var_id));
        let key = CanonicalTypeRootKey {
            family_tag: CanonicalRootFamilyTag::VariantContract,
            owner,
            product_subkey: 0,
        };
        types.register_variant_root(lineage, key, info.type_handle)?;
    }

    Ok(())
}

/// Projects all structural primary contracts into a bounded, deterministic wire DTO.
pub fn export_structural_primary_contracts(
    types: &TypeExportContext,
    data_table: &DataSemanticTable,
    enum_table: &EnumSemanticTable,
    declarations: &DeclarationTypeTable,
    hierarchy: &MapTypeHierarchy,
    type_aliases: &TypeAliasTable,
    limits: &ValidationLimits,
) -> Result<StructuralPrimaryContractsV1, ArtifactError> {
    let mut declaration_shells = Vec::new();
    let mut sorted_declarations = declarations.iter().collect::<Vec<_>>();
    sorted_declarations.sort_by(|a, b| a.0.cmp(b.0));

    let mut generic_signatures = Vec::new();

    for (decl_id, info) in sorted_declarations {
        let declaration = to_stable_declaration(decl_id);
        let self_type = types.ty(info.form)?;
        let parameters = if let Some(sig) = &info.generic_signature {
            sig.parameters
                .iter()
                .map(|&p| types.parameter(p))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };
        declaration_shells.push(DeclarationShellV1 {
            declaration,
            self_type,
            parameters,
        });

        if let Some(sig) = &info.generic_signature {
            generic_signatures.push(export_generic_signature(sig, types)?);
        }
    }

    let mut hierarchy_and_aliases = Vec::new();
    let mut seen_hierarchy = BTreeSet::new();

    let mut superclasses = hierarchy.iter_superclasses().collect::<Vec<_>>();
    superclasses.sort_by(|a, b| a.0.cmp(b.0));

    for (class_id, super_id) in superclasses {
        seen_hierarchy.insert(class_id.clone());
        let declaration = to_stable_declaration(class_id);
        let superclass = Some(to_stable_declaration(super_id));
        let aliased_type = type_aliases.get(class_id).map(|a| types.ty(a.form)).transpose()?;
        hierarchy_and_aliases.push(HierarchyAndAliasV1 {
            declaration,
            superclass,
            aliased_type,
        });
    }

    let mut aliases = type_aliases.iter().collect::<Vec<_>>();
    aliases.sort_by(|a, b| a.0.cmp(b.0));

    for (alias_id, info) in aliases {
        if seen_hierarchy.contains(alias_id) {
            continue;
        }
        let declaration = to_stable_declaration(alias_id);
        let aliased_type = Some(types.ty(info.form)?);
        hierarchy_and_aliases.push(HierarchyAndAliasV1 {
            declaration,
            superclass: None,
            aliased_type,
        });
    }

    let mut data_contracts = Vec::new();
    let mut data_decls = data_table.data_decls.iter().collect::<Vec<_>>();
    data_decls.sort_by(|a, b| a.0.cmp(b.0));

    for (_, info) in data_decls {
        let owner = to_stable_declaration(&info.owner);
        let root_form = types.ty(info.root_form)?;
        let generic_signature = info
            .generic_signature
            .as_ref()
            .map(|s| export_generic_signature(s, types))
            .transpose()?;
        let static_data = info.static_data.as_ref().map(|s| export_static_data(s, types)).transpose()?;
        let shape = export_data_shape(info.shape);
        let components = info
            .components
            .iter()
            .map(|c| export_data_component(c, types))
            .collect::<Result<Vec<_>, _>>()?;
        let constructor_visibility = export_visibility(info.constructor.visibility);
        let constructor_parameters = info
            .constructor
            .parameters
            .iter()
            .map(|p| export_data_constructor_parameter(p, types))
            .collect::<Result<Vec<_>, _>>()?;
        let constructor_result_type = types.ty(info.constructor.result_type_template)?;
        let shape_contributions = info
            .shape_template
            .contributions
            .iter()
            .map(|c| export_shape_contribution(c, types))
            .collect::<Result<Vec<_>, _>>()?;

        data_contracts.push(DataInfoV1 {
            owner,
            root_form,
            generic_signature,
            static_data,
            shape,
            components,
            constructor_visibility,
            constructor_parameters,
            constructor_result_type,
            shape_contributions,
        });
    }

    let mut enum_contracts = Vec::new();
    let mut enums = enum_table.enums.iter().collect::<Vec<_>>();
    enums.sort_by(|a, b| a.0.cmp(b.0));

    let mut variant_contracts = Vec::new();

    for (_, info) in enums {
        let owner = to_stable_declaration(&info.owner);
        let root_form = types.ty(info.root_form)?;
        let generic_signature = info
            .generic_signature
            .as_ref()
            .map(|s| export_generic_signature(s, types))
            .transpose()?;
        let static_data = info.static_data.as_ref().map(|s| export_static_data(s, types)).transpose()?;
        let default_result_type = types.ty(info.default_result_type)?;
        let variants = info.variants.iter().map(to_stable_variant).collect();

        for variant_id in info.variants.iter() {
            let Some(var_info) = enum_table.variant_info(variant_id) else {
                return Err(referential(format!("missing variant info for {:?}", variant_id)));
            };
            let variant = to_stable_variant(&var_info.id);
            let variant_type = types.variant(var_info.type_handle)?;
            let shape = export_variant_shape(var_info.shape);
            let payload_kind = export_payload_kind(var_info.payload_kind);
            let fields = var_info
                .fields
                .iter()
                .map(|f| export_variant_field(f, types))
                .collect::<Result<Vec<_>, _>>()?;
            let result_type = types.ty(var_info.result_type_template)?;
            let exact_case_type = types.ty(var_info.exact_case_template)?;
            let case_environment = export_case_environment(&var_info.case_environment, types)?;
            let constructor = var_info
                .constructor
                .as_ref()
                .map(|c| -> Result<VariantConstructorV1, ArtifactError> {
                    let c_sig = c
                        .generic_signature
                        .as_ref()
                        .map(|s| export_generic_signature(s, types))
                        .transpose()?;
                    let params = c
                        .parameters
                        .iter()
                        .map(|p| export_variant_constructor_parameter(p, types))
                        .collect::<Result<Vec<_>, _>>()?;
                    let res_ty = types.ty(c.result_type_template)?;
                    let case_ty = types.ty(c.exact_case_template)?;
                    Ok(VariantConstructorV1 {
                        generic_signature: c_sig,
                        parameters: params,
                        result_type: res_ty,
                        exact_case_type: case_ty,
                    })
                })
                .transpose()?;
            let visibility = export_variant_visibility(&var_info.visibility);

            variant_contracts.push(VariantInfoV1 {
                variant,
                variant_type,
                shape,
                payload_kind,
                fields,
                result_type,
                exact_case_type,
                case_environment,
                constructor,
                visibility,
            });
        }

        enum_contracts.push(EnumInfoV1 {
            owner,
            root_form,
            generic_signature,
            static_data,
            default_result_type,
            variants,
        });
    }

    let contracts = StructuralPrimaryContractsV1 {
        declaration_shells,
        hierarchy_and_aliases,
        generic_signatures,
        data_contracts,
        enum_contracts,
        variant_contracts,
    };
    contracts.validate(limits)?;
    Ok(contracts)
}