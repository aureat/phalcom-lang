//! Semantic-owned artifact export for structural primary contracts (`PERF003.C2.P7.T7.A`).

use crate::data_semantics::{DataComponentSemantic, DataConstructorParameter, DataProductContribution, DataSemanticTable, DataShape};
use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact, DeclaredTypeState};
use crate::declarations::DeclarationTypeTable;
use crate::enum_semantics::{AlgebraicCasePayloadKind, EnumSemanticTable, VariantFieldSemantic, VariantShape, VariantVisibility};
use crate::metadata::stable_identity::{
    to_stable_callable, to_stable_declaration, to_stable_dispatch_side, to_stable_field, to_stable_impl, to_stable_module, to_stable_variant,
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
    CanonicalRootFamilyTag, CanonicalTypeRootKey, ParameterModeV1, ParameterOwnerV1, SelfRoleV1, SelfTypeV1, StableOwnerRef, VarianceV1,
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
        TypeParameterOwner::Impl(r#impl) => ParameterOwnerV1::Impl { owner: to_stable_impl(r#impl) },
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

pub fn export_constraint(constraint: &GenericConstraint, types: &TypeExportContext) -> Result<GenericConstraintV1, ArtifactError> {
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
                arguments: trait_ref.arguments.iter().map(|&arg| types.ty(arg)).collect::<Result<Vec<_>, _>>()?,
            },
        },
    })
}

pub fn export_generic_signature(sig: &GenericSignature, types: &TypeExportContext) -> Result<GenericSignatureV1, ArtifactError> {
    let owner = export_parameter_owner(&sig.owner)?;
    let parameters = sig.parameters.iter().map(|&p| types.parameter(p)).collect::<Result<Vec<_>, _>>()?;
    let parameter_kinds = sig.parameter_kinds.iter().map(|&k| types.kind(k)).collect::<Result<Vec<_>, _>>()?;
    let parameter_kind_shapes = sig.parameter_kind_shapes.to_vec();
    let parameter_variances = sig.parameter_variances.iter().map(|&v| export_variance(v)).collect();
    let parameter_modes = sig.parameter_modes.iter().map(|&m| export_mode(m)).collect();
    let constraint_shapes = sig.constraint_shapes.to_vec();
    let constraints = sig.constraints.iter().map(|c| export_constraint(c, types)).collect::<Result<Vec<_>, _>>()?;

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

pub fn export_declared_type_state(state: &DeclaredTypeState, types: &TypeExportContext) -> Result<DeclaredTypeStateV1, ArtifactError> {
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

pub fn export_declared_type_fact(fact: &DeclaredTypeFact, types: &TypeExportContext) -> Result<DeclaredTypeFactV1, ArtifactError> {
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
        constructors: static_data.constructors.iter().map(|&c| types.constructor(c)).collect::<Result<Vec<_>, _>>()?,
    })
}

pub fn export_case_environment(env: &CaseTypeEnvironment, types: &TypeExportContext) -> Result<CaseEnvironmentV1, ArtifactError> {
    let mut bindings = Vec::with_capacity(env.bindings.len());
    for (&param, &ty) in &env.bindings {
        bindings.push(CaseBindingV1 {
            parameter: types.parameter(param)?,
            ty: types.ty(ty)?,
        });
    }
    bindings.sort_by_key(|b| b.parameter.0);

    let equalities = env.equalities.iter().map(|eq| export_constraint(eq, types)).collect::<Result<Vec<_>, _>>()?;

    Ok(CaseEnvironmentV1 { bindings, equalities })
}

pub fn export_data_component(comp: &DataComponentSemantic, types: &TypeExportContext) -> Result<DataComponentSemanticV1, ArtifactError> {
    Ok(DataComponentSemanticV1 {
        local_name: comp.local_name.clone(),
        external_label: comp.external_label.clone(),
        declared_type: export_declared_type_fact(&comp.declared_type, types)?,
    })
}

pub fn export_data_constructor_parameter(param: &DataConstructorParameter, types: &TypeExportContext) -> Result<DataConstructorParameterV1, ArtifactError> {
    Ok(DataConstructorParameterV1 {
        external_label: param.external_label.clone(),
        local_name: param.local_name.clone(),
        declared_type: export_declared_type_fact(&param.declared_type, types)?,
    })
}

pub fn export_shape_contribution(contribution: &DataProductContribution, types: &TypeExportContext) -> Result<DataProductContributionV1, ArtifactError> {
    Ok(match contribution {
        DataProductContribution::Component(comp) => DataProductContributionV1::Component {
            component: export_data_component(comp, types)?,
        },
        DataProductContribution::Expansion(exp) => DataProductContributionV1::Expansion {
            mode: export_expansion_mode(exp.mode),
            operand: export_declared_type_fact(&exp.operand, types)?,
        },
    })
}

pub fn export_variant_field(field: &VariantFieldSemantic, types: &TypeExportContext) -> Result<VariantFieldV1, ArtifactError> {
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
            sig.parameters.iter().map(|&p| types.parameter(p)).collect::<Result<Vec<_>, _>>()?
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
        let generic_signature = info.generic_signature.as_ref().map(|s| export_generic_signature(s, types)).transpose()?;
        let static_data = info.static_data.as_ref().map(|s| export_static_data(s, types)).transpose()?;
        let shape = export_data_shape(info.shape);
        let components = info.components.iter().map(|c| export_data_component(c, types)).collect::<Result<Vec<_>, _>>()?;
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
        let generic_signature = info.generic_signature.as_ref().map(|s| export_generic_signature(s, types)).transpose()?;
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
            let fields = var_info.fields.iter().map(|f| export_variant_field(f, types)).collect::<Result<Vec<_>, _>>()?;
            let result_type = types.ty(var_info.result_type_template)?;
            let exact_case_type = types.ty(var_info.exact_case_template)?;
            let case_environment = export_case_environment(&var_info.case_environment, types)?;
            let constructor = var_info
                .constructor
                .as_ref()
                .map(|c| -> Result<VariantConstructorV1, ArtifactError> {
                    let c_sig = c.generic_signature.as_ref().map(|s| export_generic_signature(s, types)).transpose()?;
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

use crate::artifact::operational::export_checked_static_definitions;
use crate::artifact::query::validate_query_image_closure;
use crate::artifact::validate::compute_witness_layout_fingerprint;
use crate::canonical_image::CanonicalSemanticQueryImage;
use crate::snapshot::SemanticSnapshot;
use crate::static_value::StaticExportContext;
use phalcom_compiler_artifact::query::{DependencyEvidenceEntryV1, PortableQueryKeyV1, PortableQueryKindV1, QueryImageV1, QueryRootEntryV1};
use phalcom_compiler_artifact::semantic::type_store::TypeMemberSelectorV1;
use phalcom_compiler_artifact::semantic::world::{
    AssociatedBindingEntryV1, AssociatedBindingValueV1, AssociatedFamilyKindV1, AssociatedFamilyRecordV1, AssociatedMemberV1, AssociatedStaticDomainRecordV1,
    AssociatedSurfaceRecordV1, CallableAnalysisRecordV1, CallableParameterV1, CallableSignatureRecordV1, ConformanceHeadRecordV1, ConformanceTargetV1,
    ConstructorInitializationRecordV1, ConstructorParamFieldV1, DeclarationSurfaceRecordV1, FieldSignatureV1, IntrinsicTypeOperationV1, RequirementEntryV1,
    RequirementSelectionV1, SemanticWorldV1, TraitSurfaceRecordV1, TypeMemberDeclarationRecordV1, TypeMemberDomainV1, TypeMemberImplementationV1,
    TypeMemberParameterV1, WitnessPlanV1,
};
use phalcom_type_meta::identity::{StableAssociatedStaticDomainDeclarationRef, StableCallableRef};
use std::collections::BTreeMap;

fn export_type_member_selector(selector: &crate::types::type_member::TypeMemberSelector) -> TypeMemberSelectorV1 {
    match selector {
        crate::types::type_member::TypeMemberSelector::Named(name) => TypeMemberSelectorV1::Named { name: name.to_string() },
        crate::types::type_member::TypeMemberSelector::Subscript => TypeMemberSelectorV1::Subscript,
    }
}

fn export_associated_selector(base: &phalcom_common::selector::SelectorBase) -> TypeMemberSelectorV1 {
    match base {
        phalcom_common::selector::SelectorBase::Named(name) => TypeMemberSelectorV1::Named { name: name.clone() },
        phalcom_common::selector::SelectorBase::Subscript => TypeMemberSelectorV1::Subscript,
    }
}

fn export_type_member_domain(domain: crate::types::type_member::TypeMemberDomain, types: &TypeExportContext) -> Result<TypeMemberDomainV1, ArtifactError> {
    Ok(match domain {
        crate::types::type_member::TypeMemberDomain::Kind(kind) => TypeMemberDomainV1::Kind { kind: types.kind(kind)? },
        crate::types::type_member::TypeMemberDomain::ValueType(ty) => TypeMemberDomainV1::ValueType { ty: types.ty(ty)? },
    })
}

fn export_intrinsic_type_operation(operation: crate::types::type_member::IntrinsicTypeOperation) -> IntrinsicTypeOperationV1 {
    use crate::types::type_member::IntrinsicTypeOperation as Native;
    match operation {
        Native::TuplePositionals => IntrinsicTypeOperationV1::TuplePositionals,
        Native::TupleLabeled => IntrinsicTypeOperationV1::TupleLabeled,
        Native::TupleEntries => IntrinsicTypeOperationV1::TupleEntries,
        Native::TupleElement => IntrinsicTypeOperationV1::TupleElement,
        Native::TupleAt => IntrinsicTypeOperationV1::TupleAt,
        Native::TupleMap => IntrinsicTypeOperationV1::TupleMap,
        Native::RecordFields => IntrinsicTypeOperationV1::RecordFields,
        Native::RecordValue => IntrinsicTypeOperationV1::RecordValue,
        Native::RecordMap => IntrinsicTypeOperationV1::RecordMap,
        Native::TypePackMap => IntrinsicTypeOperationV1::TypePackMap,
        Native::LabeledTypePackMap => IntrinsicTypeOperationV1::LabeledTypePackMap,
        Native::RecordRowMap => IntrinsicTypeOperationV1::RecordRowMap,
        Native::CompleteTypePackPositionals => IntrinsicTypeOperationV1::CompleteTypePackPositionals,
        Native::CompleteTypePackLabeled => IntrinsicTypeOperationV1::CompleteTypePackLabeled,
        Native::CompleteTypePackMap => IntrinsicTypeOperationV1::CompleteTypePackMap,
        Native::UnorderedShapeLookup => IntrinsicTypeOperationV1::UnorderedShapeLookup,
        Native::UnorderedShapeView => IntrinsicTypeOperationV1::UnorderedShapeView,
        Native::UnorderedShapeTail => IntrinsicTypeOperationV1::UnorderedShapeTail,
    }
}

fn export_type_member_implementation(implementation: phalcom_common::modifier::ImplementationKind) -> TypeMemberImplementationV1 {
    match implementation {
        phalcom_common::modifier::ImplementationKind::Phalcom => TypeMemberImplementationV1::Phalcom,
        phalcom_common::modifier::ImplementationKind::Native => TypeMemberImplementationV1::Native,
        phalcom_common::modifier::ImplementationKind::Foreign => TypeMemberImplementationV1::Foreign,
        phalcom_common::modifier::ImplementationKind::Intrinsic => TypeMemberImplementationV1::Intrinsic,
    }
}

fn export_associated_family_kind(kind: crate::associated::AssociatedFamilyKind) -> AssociatedFamilyKindV1 {
    match kind {
        crate::associated::AssociatedFamilyKind::Variant => AssociatedFamilyKindV1::Variant,
        crate::associated::AssociatedFamilyKind::DataConstructor => AssociatedFamilyKindV1::DataConstructor,
        crate::associated::AssociatedFamilyKind::StaticDomain => AssociatedFamilyKindV1::StaticDomain,
        crate::associated::AssociatedFamilyKind::TypeMember => AssociatedFamilyKindV1::TypeMember,
        crate::associated::AssociatedFamilyKind::IntrinsicTypeMember => AssociatedFamilyKindV1::IntrinsicTypeMember,
    }
}

fn export_associated_static_domain_declaration(
    declaration: &crate::identity::AssociatedStaticDomainDeclarationId,
) -> StableAssociatedStaticDomainDeclarationRef {
    StableAssociatedStaticDomainDeclarationRef {
        owner: to_stable_declaration(&declaration.owner),
        name: declaration.name.clone(),
    }
}

fn export_associated_member(member: &crate::associated::AssociatedMemberId) -> AssociatedMemberV1 {
    match member {
        crate::associated::AssociatedMemberId::Variant(variant) => AssociatedMemberV1::Variant {
            variant: to_stable_variant(variant),
        },
        crate::associated::AssociatedMemberId::DataConstructor(constructor) => AssociatedMemberV1::DataConstructor {
            owner: to_stable_declaration(&constructor.owner),
        },
        crate::associated::AssociatedMemberId::DataComponent(component) => AssociatedMemberV1::DataComponent {
            owner: to_stable_declaration(&component.owner),
            index: component.index,
        },
        crate::associated::AssociatedMemberId::StaticDomain(declaration) => AssociatedMemberV1::StaticDomain {
            declaration: export_associated_static_domain_declaration(declaration),
        },
        crate::associated::AssociatedMemberId::TypeMember(member) => AssociatedMemberV1::TypeMember {
            owner: to_stable_declaration(&member.owner),
            selector: export_type_member_selector(&member.selector),
        },
    }
}

pub(super) struct ExportedAssociatedAuthority {
    pub type_member_declarations: Vec<TypeMemberDeclarationRecordV1>,
    pub associated_static_domains: Vec<AssociatedStaticDomainRecordV1>,
    pub associated_surfaces: Vec<AssociatedSurfaceRecordV1>,
}

pub(super) fn export_associated_authority(
    type_member_products: &crate::parent_map::ParentMap<
        crate::types::type_member::TypeMemberDeclarationId,
        std::sync::Arc<crate::db::product::TypeMemberDeclarationProduct>,
    >,
    static_domain_products: &BTreeMap<
        crate::identity::AssociatedStaticDomainDeclarationId,
        std::sync::Arc<crate::db::product::AssociatedStaticDomainDeclarationProduct>,
    >,
    surface_products: &crate::associated::AssociatedFamilyTable,
    types: &TypeExportContext,
) -> Result<ExportedAssociatedAuthority, ArtifactError> {
    let mut type_member_products = type_member_products.iter().collect::<Vec<_>>();
    type_member_products.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
    let mut type_member_declarations = Vec::with_capacity(type_member_products.len());
    for (id, product) in type_member_products {
        if id != &product.id || product.signature.selector != id.selector {
            return Err(referential("type-member declaration identity does not match its checked product"));
        }
        let parameters = product
            .signature
            .parameters
            .iter()
            .map(|parameter| {
                Ok(TypeMemberParameterV1 {
                    name: parameter.name.clone(),
                    domain: export_type_member_domain(parameter.domain, types)?,
                })
            })
            .collect::<Result<Vec<_>, ArtifactError>>()?;
        type_member_declarations.push(TypeMemberDeclarationRecordV1 {
            owner: to_stable_declaration(&id.owner),
            selector: export_type_member_selector(&id.selector),
            parameters,
            result_domain: export_type_member_domain(product.signature.result_domain, types)?,
            intrinsic: product.signature.intrinsic.map(export_intrinsic_type_operation),
            implementation: export_type_member_implementation(product.signature.implementation),
        });
    }

    let mut associated_static_domains = Vec::with_capacity(static_domain_products.len());
    for (id, product) in static_domain_products {
        if id != &product.id {
            return Err(referential("associated static-domain identity does not match its checked product"));
        }
        associated_static_domains.push(AssociatedStaticDomainRecordV1 {
            declaration: export_associated_static_domain_declaration(id),
            domain: types.domain(product.domain)?,
            constructors: product
                .constructors
                .iter()
                .map(|&constructor| types.constructor(constructor))
                .collect::<Result<Vec<_>, _>>()?,
        });
    }

    let mut associated_surface_products = surface_products.surfaces.iter().collect::<Vec<_>>();
    associated_surface_products.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
    let mut associated_surfaces = Vec::with_capacity(associated_surface_products.len());
    for (owner, surface) in associated_surface_products {
        if owner != &surface.owner {
            return Err(referential("associated surface owner does not match its table identity"));
        }
        let mut families = Vec::with_capacity(surface.families.len());
        for (base, family) in &surface.families {
            if family.id.owner != *owner || family.id.base != *base {
                return Err(referential("associated family identity does not match its surface key"));
            }
            families.push(AssociatedFamilyRecordV1 {
                selector: export_associated_selector(base),
                kind: export_associated_family_kind(family.kind),
                members: family.members.iter().map(export_associated_member).collect(),
            });
        }
        associated_surfaces.push(AssociatedSurfaceRecordV1 {
            owner: to_stable_declaration(owner),
            families,
        });
    }

    Ok(ExportedAssociatedAuthority {
        type_member_declarations,
        associated_static_domains,
        associated_surfaces,
    })
}

pub fn export_semantic_world(
    snapshot: &SemanticSnapshot,
    types: &TypeExportContext,
    statics: &StaticExportContext,
    limits: &ValidationLimits,
) -> Result<SemanticWorldV1, ArtifactError> {
    validate_semantic_world_export_coverage(snapshot)?;

    let type_graph = types.project_graph(&snapshot.store, limits)?;
    let structural_contracts = export_structural_primary_contracts(
        types,
        &snapshot.data_semantics,
        &snapshot.enum_semantics,
        &snapshot.declarations,
        &snapshot.hierarchy,
        &snapshot.type_aliases,
        limits,
    )?;
    let static_values = statics.project_graph(snapshot.static_values.as_ref(), types, limits)?;

    let mut templates = Vec::new();
    for (_, product) in snapshot.static_definitions.iter() {
        if let Some(checked_body) = &product.checked_body {
            if let crate::static_definition::StaticDefinitionBodyRepr::Operational(template) = &checked_body.root {
                templates.push((**template).clone());
            }
        }
    }
    let checked_definitions = export_checked_static_definitions(&templates, statics, types, limits)?;

    let mut witness_plans = Vec::new();
    for (_, plan) in snapshot.conformance_witness_plans.iter() {
        let impl_id = to_stable_impl(&plan.impl_id);
        let source_module = to_stable_module(&plan.source_module);
        let mut trait_args = Vec::with_capacity(plan.trait_ref_template.arguments.len());
        for a in &plan.trait_ref_template.arguments {
            trait_args.push(types.ty(*a)?);
        }
        let trait_ref_template = TraitRefV1 {
            declaration: to_stable_declaration(&plan.trait_ref_template.declaration),
            arguments: trait_args,
        };
        let target_template = types.ty(plan.target_template)?;
        let generic_signature = if let Some(g) = &plan.generic_signature {
            Some(export_generic_signature(g, types)?)
        } else {
            None
        };
        let mut requirements = Vec::with_capacity(plan.requirements.len());
        for (req_id, selection) in &plan.requirements {
            let side = match req_id.side {
                crate::identity::DispatchSide::Instance => 0,
                crate::identity::DispatchSide::Class => 1,
            };
            let sel_v1 = match selection {
                crate::impls::RequirementSelectionTemplate::ConformanceCallable { callable } => RequirementSelectionV1::ConformanceCallable {
                    callable: to_stable_callable(callable),
                },
                crate::impls::RequirementSelectionTemplate::InherentCallable { callable, .. } => RequirementSelectionV1::InherentCallable {
                    callable: to_stable_callable(callable),
                },
                crate::impls::RequirementSelectionTemplate::TraitDefault { callable } => RequirementSelectionV1::TraitDefault {
                    callable: to_stable_callable(callable),
                },
                crate::impls::RequirementSelectionTemplate::DataComponent { component, specialized_type } => RequirementSelectionV1::DataComponent {
                    owner: to_stable_declaration(&component.owner),
                    index: component.index,
                    specialized_type: types.ty(*specialized_type)?,
                },
                crate::impls::RequirementSelectionTemplate::ConditionalInherent { .. } => {
                    return Err(referential("unresolved conditional inherent cannot be exported to artifact"));
                }
            };
            requirements.push(RequirementEntryV1 {
                selector: req_id.selector.to_string().into_boxed_str(),
                side,
                selection: sel_v1,
            });
        }
        let mut associated_types = Vec::with_capacity(plan.associated_type_plan.bindings.len());
        for (req_id, template) in &plan.associated_type_plan.bindings {
            let value = match template.value_template {
                crate::impls::AssociatedTypeBindingValue::Type(ty) => AssociatedBindingValueV1::Type { ty: types.ty(ty)? },
                crate::impls::AssociatedTypeBindingValue::Dynamic => AssociatedBindingValueV1::Dynamic,
            };
            associated_types.push(AssociatedBindingEntryV1 { index: req_id.index, value });
        }
        let fingerprint = plan.fingerprint.0;
        let ordered_selected_callables: Vec<StableCallableRef> = plan
            .requirements
            .values()
            .filter_map(|req| match req {
                crate::impls::RequirementSelectionTemplate::ConformanceCallable { callable }
                | crate::impls::RequirementSelectionTemplate::InherentCallable { callable, .. }
                | crate::impls::RequirementSelectionTemplate::TraitDefault { callable } => Some(to_stable_callable(callable)),
                _ => None,
            })
            .collect();
        let slot_layout_fingerprint = compute_witness_layout_fingerprint(&ordered_selected_callables);
        let completeness = match plan.completeness {
            crate::impls::ConformanceCompleteness::Complete => 0,
            crate::impls::ConformanceCompleteness::Incomplete { .. } => 1,
            crate::impls::ConformanceCompleteness::Unknown(_) => 2,
            crate::impls::ConformanceCompleteness::Blocked(_) => 3,
            crate::impls::ConformanceCompleteness::Dynamic(_) => 4,
            crate::impls::ConformanceCompleteness::Cancelled => 5,
            crate::impls::ConformanceCompleteness::BudgetExceeded(_) => 6,
            crate::impls::ConformanceCompleteness::InternalFailure(_) => 7,
        };
        witness_plans.push(WitnessPlanV1 {
            impl_id,
            source_module,
            trait_ref_template,
            target_template,
            generic_signature,
            requirements,
            associated_types,
            fingerprint,
            ordered_selected_callables,
            slot_layout_fingerprint,
            completeness,
        });
    }

    let mut callable_signatures = Vec::new();
    for (_, sig) in snapshot.callable_signatures.iter() {
        let callable = to_stable_callable(&sig.callable);
        let owner = to_stable_declaration(&sig.owner);
        let side = match sig.side {
            crate::identity::DispatchSide::Instance => 0,
            crate::identity::DispatchSide::Class => 1,
        };
        let selector = sig.selector.to_string().into_boxed_str();
        let generics = if let Some(g) = &sig.generics {
            Some(export_generic_signature(g, types)?)
        } else {
            None
        };
        let mut parameters = Vec::with_capacity(sig.parameters.len());
        for p in sig.parameters.iter() {
            let declared_type = export_declared_type_fact(&p.contract.declared_type(), types)?;
            let rest_mode = match p.contract.rest_mode() {
                phalcom_ast::ast::RestMode::None => 0,
                phalcom_ast::ast::RestMode::Positional => 1,
                phalcom_ast::ast::RestMode::Labeled => 2,
                phalcom_ast::ast::RestMode::Complete => 3,
            };
            parameters.push(CallableParameterV1 {
                local_name: p.local_name.clone(),
                external_label: p.external_label.clone(),
                declared_type,
                rest_mode,
            });
        }
        let declared_return = export_declared_type_fact(&sig.declared_return, types)?;
        let return_flow = match sig.flow {
            phalcom_native_meta::ReturnFlowSpec::Value => 0,
            phalcom_native_meta::ReturnFlowSpec::Receiver => 1,
            phalcom_native_meta::ReturnFlowSpec::Argument(i) => (2 + i) as u8,
            phalcom_native_meta::ReturnFlowSpec::Never => 254,
            phalcom_native_meta::ReturnFlowSpec::Unknown => 255,
        };
        callable_signatures.push(CallableSignatureRecordV1 {
            callable,
            owner,
            side,
            selector,
            generics,
            parameters,
            declared_return,
            return_flow,
        });
    }

    let mut callable_analyses = Vec::new();
    for (callable_id, analysis) in snapshot.callable_analyses.iter() {
        let callable = to_stable_callable(callable_id);
        let effects = 0;
        let is_terminating = analysis.exits.unreachable;
        callable_analyses.push(CallableAnalysisRecordV1 {
            callable,
            effects,
            is_terminating,
        });
    }

    let mut trait_surfaces = Vec::new();
    for (decl_id, surface) in snapshot.trait_surfaces.iter() {
        let declaration = to_stable_declaration(decl_id);
        let generic_signature = if let Some(g) = &surface.generic_signature {
            Some(export_generic_signature(g, types)?)
        } else {
            None
        };
        let mut super_traits = Vec::new();
        if let Some(generics) = &surface.generic_signature {
            for constraint in &generics.constraints {
                if let GenericConstraint::Conforms { subject, trait_ref } = constraint {
                    if matches!(subject, TypeTerm::SelfType(_)) {
                        let mut args = Vec::with_capacity(trait_ref.arguments.len());
                        for a in &trait_ref.arguments {
                            args.push(types.ty(*a)?);
                        }
                        super_traits.push(TraitRefV1 {
                            declaration: to_stable_declaration(&trait_ref.declaration),
                            arguments: args,
                        });
                    }
                }
            }
        }
        let mut required_callables = Vec::new();
        let mut default_callables = Vec::new();
        for (_, member) in surface.iter() {
            let stable = to_stable_callable(&member.callable);
            if member.default_present {
                default_callables.push(stable);
            } else {
                required_callables.push(stable);
            }
        }
        let associated_types = surface.associated_types.values().map(|req| req.name.clone()).collect();
        trait_surfaces.push(TraitSurfaceRecordV1 {
            declaration,
            generic_signature,
            super_traits,
            required_callables,
            default_callables,
            associated_types,
        });
    }

    let mut dispatch_surfaces = Vec::new();
    for (decl_id, surface) in snapshot.surfaces.iter() {
        let declaration = to_stable_declaration(decl_id);
        let instance_callables = surface.instance.callables.keys().map(to_stable_callable).collect();
        let class_callables = surface.class.callables.keys().map(to_stable_callable).collect();
        let mut fields = Vec::new();
        for (_, field_sig) in snapshot.field_signatures.iter_owner(decl_id) {
            let owner = to_stable_declaration(&field_sig.owner);
            let name = field_sig.name.clone();
            let declared_type = export_declared_type_fact(&field_sig.declared_type, types)?;
            let mutable = field_sig.mutable;
            let side = match field_sig.side {
                crate::identity::DispatchSide::Instance => 0,
                crate::identity::DispatchSide::Class => 1,
            };
            fields.push(FieldSignatureV1 {
                owner,
                name,
                declared_type,
                mutable,
                side,
            });
        }
        dispatch_surfaces.push(DeclarationSurfaceRecordV1 {
            declaration,
            instance_callables,
            class_callables,
            fields,
        });
    }

    let mut conformance_heads = Vec::new();
    for (impl_id, contribution) in snapshot.conformance_index.iter() {
        let stable_impl = to_stable_impl(impl_id);
        let source_module = to_stable_module(&contribution.source_module);
        let target = match &contribution.target {
            crate::impls::ConformanceTarget::Declaration(decl) => ConformanceTargetV1::Declaration {
                declaration: to_stable_declaration(decl),
            },
            crate::impls::ConformanceTarget::ExactEnumCase(variant) => ConformanceTargetV1::ExactEnumCase {
                variant: to_stable_variant(variant),
            },
        };
        let target_type = types.ty(contribution.target_head)?;
        let trait_decl = to_stable_declaration(&contribution.trait_ref.declaration);
        let mut args = Vec::with_capacity(contribution.trait_ref.arguments.len());
        for a in &contribution.trait_ref.arguments {
            args.push(types.ty(*a)?);
        }
        let generic_signature = if let Some(g) = &contribution.generic_signature {
            Some(export_generic_signature(g, types)?)
        } else {
            None
        };
        conformance_heads.push(ConformanceHeadRecordV1 {
            impl_id: stable_impl,
            source_module,
            target,
            target_type,
            trait_ref: TraitRefV1 {
                declaration: trait_decl,
                arguments: args,
            },
            generic_signature,
            authorized: contribution.authorized,
            eligible: contribution.eligible,
        });
    }

    let mut constructor_initializations = Vec::new();
    for (_, product) in snapshot.constructor_initializations.iter() {
        let factory = to_stable_callable(&product.factory);
        let initializer = to_stable_callable(&product.initializer);
        let origin = match product.origin {
            crate::signature::ConstructorOrigin::Source => 0,
            crate::signature::ConstructorOrigin::Construct => 1,
            crate::signature::ConstructorOrigin::Data => 2,
            crate::signature::ConstructorOrigin::DefaultOnly => 3,
        };
        let field_input_fingerprint = product.field_input_fingerprint.0;
        let default_fields = product.default_fields.iter().map(|f| f.name.clone()).collect();
        let parameter_fields = product
            .parameter_fields
            .iter()
            .map(|pf| ConstructorParamFieldV1 {
                parameter_index: pf.parameter.index,
                field_name: pf.field.name.clone(),
                field_side: match pf.field.side {
                    crate::identity::DispatchSide::Instance => 0,
                    crate::identity::DispatchSide::Class => 1,
                },
            })
            .collect();
        constructor_initializations.push(ConstructorInitializationRecordV1 {
            factory,
            initializer,
            origin,
            field_input_fingerprint,
            default_fields,
            parameter_fields,
        });
    }

    let ExportedAssociatedAuthority {
        type_member_declarations,
        associated_static_domains,
        associated_surfaces,
    } = export_associated_authority(
        &snapshot.type_member_declarations,
        &snapshot.associated_static_domains,
        &snapshot.associated_surfaces,
        types,
    )?;

    let world = SemanticWorldV1 {
        type_graph,
        structural_contracts,
        static_values,
        checked_definitions,
        witness_plans,
        callable_signatures,
        callable_analyses,
        trait_surfaces,
        dispatch_surfaces,
        conformance_heads,
        constructor_initializations,
        type_member_declarations,
        associated_static_domains,
        associated_surfaces,
    };
    world.validate(limits)?;
    Ok(world)
}

/// Stops the writer before it emits an image whose importer would have to
/// fabricate or discard required compiler authority. As each omitted family
/// gains an owner-controlled wire adapter, remove its corresponding blocker.
fn validate_semantic_world_export_coverage(snapshot: &SemanticSnapshot) -> Result<(), ArtifactError> {
    if !snapshot.callable_analyses.is_empty() {
        return Err(referential(
            "semantic artifact export does not yet encode callable analysis facts and cannot publish a complete image",
        ));
    }
    if !snapshot.enum_requirements.requirements.is_empty()
        || !snapshot.static_case_catalogs.is_empty()
        || !snapshot.static_case_signatures.is_empty()
        || snapshot.static_definitions.iter().next().is_some()
        || !snapshot.static_constants.is_empty()
        || snapshot.module_static_bindings.iter().next().is_some()
        || !snapshot.derived_conformance_rules.is_empty()
        || !snapshot.trait_dispatch_index.is_empty()
    {
        return Err(referential(
            "semantic artifact export omits required primary owner families and cannot publish a complete image",
        ));
    }
    for (_, plan) in snapshot.conformance_witness_plans.iter() {
        if !plan.requirement_views.is_empty()
            || plan.invalid_explicit_members
            || !matches!(plan.completeness, crate::impls::ConformanceCompleteness::Complete)
            || !plan.associated_type_plan.failures.is_empty()
            || plan.requirements.values().any(|selection| {
                matches!(
                    selection,
                    crate::impls::RequirementSelectionTemplate::InherentCallable { conditional_impl: Some(_), .. }
                        | crate::impls::RequirementSelectionTemplate::InherentCallable { applicability: Some(_), .. }
                        | crate::impls::RequirementSelectionTemplate::ConditionalInherent { .. }
                )
            })
        {
            return Err(referential("semantic artifact export cannot losslessly encode this conformance witness plan"));
        }
    }
    Ok(())
}

pub fn export_query_image(
    queries: &CanonicalSemanticQueryImage,
    dependency_fingerprints: &BTreeMap<crate::db::QueryKey, crate::db::ProductFingerprint>,
    limits: &ValidationLimits,
) -> Result<QueryImageV1, ArtifactError> {
    let mut candidate_roots = Vec::new();

    for (key, entry) in queries.entries() {
        if matches!(key, crate::db::QueryKey::ParsedModule(_)) {
            continue;
        }
        let Some(kind) = portable_query_kind(key) else {
            continue;
        };
        if kind.profile_support() != phalcom_compiler_artifact::query::QueryProfileSupport::SupportedReusableRoot {
            continue;
        }
        let stable_owner_key = portable_query_owner_key(key)?;
        let portable_key = PortableQueryKeyV1 { kind, stable_owner_key };
        candidate_roots.push((key.clone(), portable_key, entry));
    }

    // Reverse pruning: only retain roots whose dependencies are ALL retained in candidate set!
    let mut retained_keys: BTreeSet<crate::db::QueryKey> = candidate_roots.iter().map(|(k, _, _)| k.clone()).collect();
    let mut changed = true;
    while changed {
        changed = false;
        let mut to_remove = Vec::new();
        for key in &retained_keys {
            if let Some(entry) = queries.entry(key) {
                if !entry.dependencies.iter().all(|dep| retained_keys.contains(&dep.dependency)) {
                    to_remove.push(key.clone());
                }
            } else {
                to_remove.push(key.clone());
            }
        }
        if !to_remove.is_empty() {
            changed = true;
            for key in to_remove {
                retained_keys.remove(&key);
            }
        }
    }

    let mut supported_roots = Vec::new();
    let mut key_to_final_index = BTreeMap::new();
    for (key, port_key, entry) in candidate_roots {
        if retained_keys.contains(&key) {
            let idx = supported_roots.len() as u32;
            key_to_final_index.insert(key, idx);
            supported_roots.push((port_key, entry));
        }
    }

    let mut roots = Vec::with_capacity(supported_roots.len());
    for (port_key, entry) in supported_roots {
        let mut dependency_indices = Vec::new();
        for dep in entry.dependencies.iter() {
            if let Some(&dep_idx) = key_to_final_index.get(&dep.dependency) {
                dependency_indices.push(dep_idx);
            }
        }
        roots.push(QueryRootEntryV1 {
            key: port_key,
            dependency_indices,
        });
    }

    let mut dependency_evidence = Vec::new();
    for (key, _) in dependency_fingerprints {
        if matches!(key, crate::db::QueryKey::ParsedModule(_)) {
            continue;
        }
        let Some(kind) = portable_query_kind(key) else {
            continue;
        };
        let stable_owner_key = portable_query_owner_key(key)?;
        let stable_aggregate_owner = stable_owner_key.clone();
        dependency_evidence.push(DependencyEvidenceEntryV1 {
            key: PortableQueryKeyV1 { kind, stable_owner_key },
            stable_aggregate_owner,
        });
    }

    let image = QueryImageV1 {
        supported_roots: roots,
        dependency_evidence,
    };
    image.validate(limits)?;
    validate_query_image_closure(&image)?;
    Ok(image)
}

pub(crate) fn portable_query_owner_key(key: &crate::db::QueryKey) -> Result<String, ArtifactError> {
    fn typed_json<T: serde::Serialize>(tag: &str, value: &T) -> Result<String, ArtifactError> {
        let encoded = serde_json::to_string(value).map_err(|error| referential(format!("cannot encode stable query owner: {error}")))?;
        Ok(format!("{tag}:{encoded}"))
    }

    match key {
        crate::db::QueryKey::UnlinkedInterface(m)
        | crate::db::QueryKey::ResolvedImports(m)
        | crate::db::QueryKey::LinkedInterface(m)
        | crate::db::QueryKey::SemanticComponent(m)
        | crate::db::QueryKey::SourceStructure(m)
        | crate::db::QueryKey::AdvisoryModule(m)
        | crate::db::QueryKey::ModuleDiagnostics(m)
        | crate::db::QueryKey::ModuleMetadata(m)
        | crate::db::QueryKey::ModuleStaticBindingCatalog(m)
        | crate::db::QueryKey::ModuleStaticBindings(m) => typed_json("module", &to_stable_module(m)),
        crate::db::QueryKey::DeclarationShell(d)
        | crate::db::QueryKey::DeclarationSurface(d)
        | crate::db::QueryKey::TraitHeader(d)
        | crate::db::QueryKey::TraitSurface(d)
        | crate::db::QueryKey::HierarchyEdge(d)
        | crate::db::QueryKey::ConstructorInitialization(d)
        | crate::db::QueryKey::DataDeclaration(d)
        | crate::db::QueryKey::EnumDeclaration(d)
        | crate::db::QueryKey::EnumRequirements(d)
        | crate::db::QueryKey::AssociatedSurface(d)
        | crate::db::QueryKey::StaticDefinition(d) => typed_json("declaration", &to_stable_declaration(d)),
        crate::db::QueryKey::CallableDefinition(c)
        | crate::db::QueryKey::CallableSignature(c)
        | crate::db::QueryKey::CallableBody(c)
        | crate::db::QueryKey::CallableEffects(c)
        | crate::db::QueryKey::CallableControl(c)
        | crate::db::QueryKey::CallableTermination(c)
        | crate::db::QueryKey::CallableContracts(c)
        | crate::db::QueryKey::VerificationConditions(c)
        | crate::db::QueryKey::SourceFormalAttachment(c)
        | crate::db::QueryKey::AdvisoryCallable(c) => typed_json("callable", &to_stable_callable(c)),
        crate::db::QueryKey::FieldSignature(f) => typed_json("field", &to_stable_field(f)),
        crate::db::QueryKey::TypeMemberDeclaration(id) => {
            let selector = match &id.selector {
                crate::types::type_member::TypeMemberSelector::Named(name) => (0_u8, name.as_ref()),
                crate::types::type_member::TypeMemberSelector::Subscript => (1_u8, ""),
            };
            typed_json("type-member", &(to_stable_declaration(&id.owner), selector))
        }
        crate::db::QueryKey::ConformanceDispatch(impl_id) => typed_json("impl", &to_stable_impl(impl_id)),
        _ => Err(referential("query kind has no portable stable owner-key encoding")),
    }
}

fn portable_query_kind(key: &crate::db::QueryKey) -> Option<PortableQueryKindV1> {
    match key {
        crate::db::QueryKey::ParsedModule(_) => Some(PortableQueryKindV1::ParsedModule),
        crate::db::QueryKey::UnlinkedInterface(_) => Some(PortableQueryKindV1::UnlinkedInterface),
        crate::db::QueryKey::ResolvedImports(_) => Some(PortableQueryKindV1::ResolvedImports),
        crate::db::QueryKey::LinkedInterface(_) => Some(PortableQueryKindV1::LinkedInterface),
        crate::db::QueryKey::DeclarationShell(_) => Some(PortableQueryKindV1::DeclarationShell),
        crate::db::QueryKey::HierarchyEdge(_) => Some(PortableQueryKindV1::HierarchyEdge),
        crate::db::QueryKey::CallableDefinition(_) => Some(PortableQueryKindV1::CallableDefinition),
        crate::db::QueryKey::CallableSignature(_) => Some(PortableQueryKindV1::CallableSignature),
        crate::db::QueryKey::FieldSignature(_) => Some(PortableQueryKindV1::FieldSignature),
        crate::db::QueryKey::DeclarationSurface(_) => Some(PortableQueryKindV1::DeclarationSurface),
        crate::db::QueryKey::TraitHeader(_) => Some(PortableQueryKindV1::TraitHeader),
        crate::db::QueryKey::TraitSurface(_) => Some(PortableQueryKindV1::TraitSurface),
        crate::db::QueryKey::DataDeclaration(_) => Some(PortableQueryKindV1::DataDeclaration),
        crate::db::QueryKey::EnumDeclaration(_) => Some(PortableQueryKindV1::EnumDeclaration),
        crate::db::QueryKey::AssociatedSurface(_) => Some(PortableQueryKindV1::AssociatedSurface),
        crate::db::QueryKey::TypeMemberDeclaration(_) => Some(PortableQueryKindV1::TypeMemberDeclaration),
        crate::db::QueryKey::ModuleStaticBindingCatalog(_) => Some(PortableQueryKindV1::ModuleStaticBindingCatalog),
        crate::db::QueryKey::ModuleStaticBindings(_) => Some(PortableQueryKindV1::ModuleStaticBindings),
        _ => None,
    }
}
