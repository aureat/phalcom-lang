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
use crate::types::evidence::{DynamicReason, EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
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

fn export_unknown_reason(reason: &UnknownReason) -> UnknownReasonV1 {
    match reason {
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
    }
}

fn export_dynamic_reason(reason: &DynamicReason) -> DynamicReasonV1 {
    match reason {
        DynamicReason::ExplicitEscape => DynamicReasonV1::ExplicitEscape,
        DynamicReason::DynamicRestPack => DynamicReasonV1::DynamicRestPack,
        DynamicReason::RuntimeReflection => DynamicReasonV1::RuntimeReflection,
    }
}

fn export_source_span(source: &crate::diagnostic::SemanticSourceSpan) -> Result<EnumRequirementSourceV1, ArtifactError> {
    Ok(EnumRequirementSourceV1 {
        module: to_stable_module(&source.module),
        range: phalcom_compiler_artifact::module_world::SpanV1 {
            start: u64::try_from(source.range.start).map_err(|_| referential("callable source start exceeds portable range"))?,
            end: u64::try_from(source.range.end).map_err(|_| referential("callable source end exceeds portable range"))?,
        },
    })
}

fn export_type_knowledge(knowledge: &TypeKnowledge, types: &TypeExportContext) -> Result<TypeKnowledgeV1, ArtifactError> {
    Ok(match knowledge {
        TypeKnowledge::Known(evidence) => TypeKnowledgeV1::Known {
            ty: types.ty(evidence.ty())?,
            status: match evidence.status() {
                EvidenceStatus::Established => EvidenceStatusV1::Established,
                EvidenceStatus::Assumed => EvidenceStatusV1::Assumed,
            },
            origin: match evidence.origin() {
                EvidenceOrigin::Syntax => EvidenceOriginV1::Syntax,
                EvidenceOrigin::DeclarationSemantics => EvidenceOriginV1::DeclarationSemantics,
                EvidenceOrigin::ConstructorSemantics => EvidenceOriginV1::ConstructorSemantics,
                EvidenceOrigin::CallableSignature => EvidenceOriginV1::CallableSignature,
                EvidenceOrigin::NativeSignature => EvidenceOriginV1::NativeSignature,
                EvidenceOrigin::DeveloperAnnotation => EvidenceOriginV1::DeveloperAnnotation,
                EvidenceOrigin::GenericInference => EvidenceOriginV1::GenericInference,
                EvidenceOrigin::Flow => EvidenceOriginV1::Flow,
                EvidenceOrigin::FieldLifecycle => EvidenceOriginV1::FieldLifecycle,
                EvidenceOrigin::ContextualDerivation => EvidenceOriginV1::ContextualDerivation,
                EvidenceOrigin::PatternDecomposition => EvidenceOriginV1::PatternDecomposition,
            },
            provenance: EvidenceSetV1 {
                ranges: evidence
                    .provenance()
                    .ranges
                    .iter()
                    .map(|range| {
                        Ok(phalcom_compiler_artifact::module_world::SpanV1 {
                            start: u64::try_from(range.start).map_err(|_| referential("evidence range start exceeds portable range"))?,
                            end: u64::try_from(range.end).map_err(|_| referential("evidence range end exceeds portable range"))?,
                        })
                    })
                    .collect::<Result<Vec<_>, ArtifactError>>()?,
                descriptions: evidence
                    .provenance()
                    .descriptions
                    .iter()
                    .map(|description| description.clone().into_boxed_str())
                    .collect(),
            },
        },
        TypeKnowledge::Unknown(reason) => TypeKnowledgeV1::Unknown {
            reason: export_unknown_reason(reason),
        },
        TypeKnowledge::Dynamic(reason) => TypeKnowledgeV1::Dynamic {
            reason: match export_dynamic_reason(reason) {
                DynamicReasonV1::ExplicitEscape => DynamicReasonV1::ExplicitEscape,
                DynamicReasonV1::DynamicRestPack => DynamicReasonV1::DynamicRestPack,
                DynamicReasonV1::RuntimeReflection => DynamicReasonV1::RuntimeReflection,
            },
        },
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
use crate::artifact::query::validate_query_image_closure_with_allocation_budget;
use crate::artifact::validate::compute_witness_layout_fingerprint;
use crate::canonical_image::CanonicalSemanticQueryImage;
use crate::enum_requirements::{CaseRequirementStatus, EnumRequirement, EnumRequirementTable};
use crate::snapshot::SemanticSnapshot;
use crate::static_value::StaticExportContext;
use phalcom_compiler_artifact::query::{DependencyEvidenceEntryV1, PortableQueryKeyV1, PortableQueryKindV1, QueryImageV1, QueryRootEntryV1};
use phalcom_compiler_artifact::semantic::type_store::TypeMemberSelectorV1;
use phalcom_compiler_artifact::semantic::world::{
    AssociatedBindingEntryV1, AssociatedBindingValueV1, AssociatedFamilyIdentityV1, AssociatedFamilyKindV1, AssociatedFamilyRecordV1, AssociatedMemberV1,
    AssociatedStaticDomainRecordV1, AssociatedSurfaceRecordV1, AssociatedValueDenotationV1, BehavioralFamilyDenotationV1, BehavioralFamilySpecV1,
    CallableAnalysisRecordV1, CallableAnalysisSourceOwnerV1, CallableAnalysisStatusV1, CallableBindingOriginV1, CallableBindingRowV1,
    CallableBindingSourceAttachmentV1, CallableCallSpecializationV1, CallableExpressionRowV1, CallableParameterV1 as PortableCallableParameter,
    CallableSignatureRecordV1, CallableStaticArgumentBindingV1, CapturedAssociatedMemberV1, CapturedBehavioralMemberV1, CaseRequirementResultV1,
    CaseRequirementStatusV1, ConditionalDispatchSelectionV1, ConformanceHeadRecordV1, ConformanceTargetV1, ConstructorInitializationRecordV1,
    ConstructorParamFieldV1, DeclarationSurfaceRecordV1, EffectSpecV1, EnumRequirementOwnerV1, EnumRequirementRecordV1, EnumRequirementSourceV1,
    EvidenceOriginV1, EvidenceSetV1, EvidenceStatusV1, ExactSelectorV1, FamilyOperationShapeV1, FieldSignatureV1, ImplementationKindV1,
    IntrinsicTypeOperationV1, InvocationTargetV1, LabeledParameterSpecV1, NativeEffectV1, NativeLifecycleSpecV1, NativeSurfaceIdV1, ParameterContractV1,
    ParameterTupleSpecV1, RaisesSpecV1, RequirementEntryV1, RequirementSelectionV1, RestParameterSpecV1, RestValueContractV1, ReturnContractValidationV1,
    ReturnFlowV1, SelectorKindPatternV1, SelectorKindV1, SelectorPatternSpecV1, SelectorSlotV1, SemanticDenotationV1, SemanticWorldV2, TraitSurfaceRecordV1,
    TypeExprSpecV1, TypeKnowledgeV1, TypeMemberDeclarationRecordV1, TypeMemberDomainV1, TypeMemberImplementationV1, TypeMemberParameterV1, WitnessPlanV1,
};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::identity::{StableAssociatedStaticDomainDeclarationRef, StableCallableRef};
use std::collections::BTreeMap;

fn export_callable_analysis_status(status: crate::checker::analysis::CallableAnalysisStatus) -> Result<CallableAnalysisStatusV1, ArtifactError> {
    Ok(match status {
        crate::checker::analysis::CallableAnalysisStatus::Complete => CallableAnalysisStatusV1::Complete,
        crate::checker::analysis::CallableAnalysisStatus::Partial => CallableAnalysisStatusV1::Partial,
        crate::checker::analysis::CallableAnalysisStatus::Blocked => CallableAnalysisStatusV1::Blocked,
        crate::checker::analysis::CallableAnalysisStatus::Cancelled => CallableAnalysisStatusV1::Cancelled,
        crate::checker::analysis::CallableAnalysisStatus::BudgetExceeded => CallableAnalysisStatusV1::BudgetExceeded,
        crate::checker::analysis::CallableAnalysisStatus::InternalFailure(_) => {
            return Err(referential("internal callable-analysis incidents have no portable artifact identity"));
        }
    })
}

/// Deterministic, callable-qualified coordinates for every existing body-local
/// identity family. These maps are also the closure boundary used by the
/// eventual wire projection; raw numeric IDs never cross a callable owner.
struct CallableAnalysisOrdinalMap {
    owner: StableCallableRef,
    expressions: BTreeMap<crate::identity::ExpressionId, u32>,
    bindings: BTreeMap<crate::identity::BindingId, u32>,
    flow_nodes: BTreeMap<crate::identity::FlowNodeId, u32>,
    flow_edges: BTreeMap<crate::identity::FlowEdgeId, u32>,
    explanations: BTreeMap<crate::identity::ExplanationId, u32>,
    diagnostic_causes: BTreeMap<crate::identity::DiagnosticCauseId, u32>,
    pattern_sites: BTreeMap<crate::match_semantics::PatternSiteKey, u32>,
}

fn assign_local_ordinals<T: Copy + Ord>(ids: impl IntoIterator<Item = T>) -> Result<BTreeMap<T, u32>, ArtifactError> {
    let mut output = BTreeMap::new();
    for (ordinal, id) in ids.into_iter().enumerate() {
        let ordinal = u32::try_from(ordinal).map_err(|_| referential("callable-local artifact ordinal overflow"))?;
        output.insert(id, ordinal);
    }
    Ok(output)
}

impl CallableAnalysisOrdinalMap {
    fn local_counts(&self) -> [usize; 7] {
        [
            self.expressions.len(),
            self.bindings.len(),
            self.flow_nodes.len(),
            self.flow_edges.len(),
            self.explanations.len(),
            self.diagnostic_causes.len(),
            self.pattern_sites.len(),
        ]
    }

    fn new(callable: &crate::identity::CallableId, analysis: &crate::checker::analysis::CallableAnalysis) -> Result<Self, ArtifactError> {
        use crate::checker::analysis::{AnalysisStatus, CallableAnalysisStatus};
        use crate::checker::causal::CausalInvalidity;
        use crate::explain::node::EvidenceRef;

        if analysis.callable != *callable {
            return Err(referential("callable analysis table key does not match its owner"));
        }
        if !analysis.internal_incidents.is_empty() || matches!(analysis.status, CallableAnalysisStatus::InternalFailure(_)) {
            return Err(referential("canonical callable analysis cannot contain internal incidents"));
        }

        let body_ids = analysis
            .expressions
            .keys()
            .map(|expression| expression.owner)
            .collect::<std::collections::BTreeSet<_>>();
        if body_ids.len() > 1 {
            return Err(referential("callable analysis contains expression identities from multiple bodies"));
        }
        for (id, expression) in &analysis.expressions {
            if id != &expression.id {
                return Err(referential("callable expression map key disagrees with its owner record"));
            }
            if expression.call.is_some() {
                return Err(referential("call-resolution handle has no canonical artifact owner"));
            }
        }
        for (id, binding) in &analysis.bindings {
            if id != &binding.binding {
                return Err(referential("callable binding map key disagrees with its owner record"));
            }
        }
        for (id, node) in analysis.flow_graph.node_entries() {
            if id != &node.id {
                return Err(referential("callable flow-node map key disagrees with its owner record"));
            }
        }
        for (id, edge) in analysis.flow_graph.edge_entries() {
            if id != &edge.id {
                return Err(referential("callable flow-edge map key disagrees with its owner record"));
            }
            if edge.predicate.is_some() {
                return Err(referential("flow predicate handle has no canonical artifact owner"));
            }
        }

        let expressions = assign_local_ordinals(analysis.expressions.keys().copied())?;
        let bindings = assign_local_ordinals(analysis.bindings.keys().copied())?;
        let flow_nodes = assign_local_ordinals(analysis.flow_graph.node_ids().copied())?;
        let flow_edges = assign_local_ordinals(analysis.flow_graph.edge_ids().copied())?;
        let explanations = assign_local_ordinals(analysis.explanations.iter().map(|(id, _)| *id))?;
        let pattern_sites = assign_local_ordinals(analysis.pattern_site_resolutions.keys().copied())?;

        fn add_cause(causes: &mut std::collections::BTreeSet<crate::identity::DiagnosticCauseId>, cause: crate::identity::DiagnosticCauseId) {
            causes.insert(cause);
        }
        fn add_causal(causes: &mut std::collections::BTreeSet<crate::identity::DiagnosticCauseId>, causal: CausalInvalidity) {
            if let CausalInvalidity::One(cause) = causal {
                causes.insert(cause);
            }
        }
        let mut causes = std::collections::BTreeSet::new();
        for expression in analysis.expressions.values() {
            match &expression.status {
                AnalysisStatus::Invalid(cause) => add_cause(&mut causes, *cause),
                AnalysisStatus::Suppressed(crate::checker::causal::SuppressionCause::One(cause)) => add_cause(&mut causes, *cause),
                AnalysisStatus::InternalFailure(_) => return Err(referential("expression internal incident has no canonical artifact owner")),
                _ => {}
            }
            add_causal(&mut causes, expression.causal_invalidity);
        }
        for binding in analysis.bindings.values() {
            add_causal(&mut causes, binding.causal_invalidity);
        }
        for exit in &analysis.exits.normal_returns {
            match &exit.status {
                AnalysisStatus::Invalid(cause) => add_cause(&mut causes, *cause),
                AnalysisStatus::Suppressed(crate::checker::causal::SuppressionCause::One(cause)) => add_cause(&mut causes, *cause),
                AnalysisStatus::InternalFailure(_) => return Err(referential("exit internal incident has no canonical artifact owner")),
                _ => {}
            }
            add_causal(&mut causes, exit.causal_invalidity);
        }
        for field in analysis
            .entry_flow
            .fields
            .values()
            .chain(analysis.exits.normal_returns.iter().flat_map(|exit| exit.flow.fields.values()))
            .chain(analysis.exits.throws.iter().flat_map(|flow| flow.fields.values()))
        {
            add_causal(&mut causes, field.causal_invalidity);
        }
        for diagnostic in analysis.diagnostics.iter() {
            if let Some(cause) = diagnostic.root_cause {
                add_cause(&mut causes, cause);
            }
        }
        for (_, node) in analysis.explanations.iter() {
            for evidence in &node.evidence {
                if let EvidenceRef::Suppressed { cause } = evidence {
                    add_cause(&mut causes, *cause);
                }
            }
        }
        let diagnostic_causes = assign_local_ordinals(causes)?;

        let has_binding = |id: &crate::identity::BindingId| bindings.contains_key(id);
        let has_explanation = |id: &crate::identity::ExplanationId| explanations.contains_key(id);
        let has_cause = |id: &crate::identity::DiagnosticCauseId| diagnostic_causes.contains_key(id);

        for expression in analysis.expressions.values() {
            if expression.binding_read.is_some_and(|id| !has_binding(&id))
                || expression.binding_write.is_some_and(|id| !has_binding(&id))
                || expression.explanation.is_some_and(|id| !has_explanation(&id))
            {
                return Err(referential("callable expression references an absent binding or explanation"));
            }
            match expression.status {
                AnalysisStatus::Invalid(cause) | AnalysisStatus::Suppressed(crate::checker::causal::SuppressionCause::One(cause)) if !has_cause(&cause) => {
                    return Err(referential("callable expression references an absent diagnostic cause"));
                }
                _ => {}
            }
            if let CausalInvalidity::One(cause) = expression.causal_invalidity
                && !has_cause(&cause)
            {
                return Err(referential("callable expression causal state references an absent diagnostic cause"));
            }
        }
        for binding in analysis.bindings.values() {
            if binding.explanation.is_some_and(|id| !has_explanation(&id)) {
                return Err(referential("callable binding references an absent explanation"));
            }
            if let CausalInvalidity::One(cause) = binding.causal_invalidity
                && !has_cause(&cause)
            {
                return Err(referential("callable binding causal state references an absent diagnostic cause"));
            }
        }
        for edge in analysis.flow_graph.edges() {
            if !flow_nodes.contains_key(&edge.source) || !flow_nodes.contains_key(&edge.target) {
                return Err(referential("callable flow edge references an absent node"));
            }
            let source = analysis.flow_graph.node(&edge.source).expect("validated source node");
            let target = analysis.flow_graph.node(&edge.target).expect("validated target node");
            if !source.successors.contains(&edge.id) || !target.predecessors.contains(&edge.id) {
                return Err(referential("callable flow edge is missing from its endpoint adjacency index"));
            }
        }
        for node in analysis.flow_graph.nodes() {
            if node.predecessors.windows(2).any(|pair| pair[0] >= pair[1])
                || node.successors.windows(2).any(|pair| pair[0] >= pair[1])
                || node
                    .predecessors
                    .iter()
                    .any(|edge| !flow_edges.contains_key(edge) || analysis.flow_graph.edge(edge).is_none_or(|flow_edge| flow_edge.target != node.id))
                || node
                    .successors
                    .iter()
                    .any(|edge| !flow_edges.contains_key(edge) || analysis.flow_graph.edge(edge).is_none_or(|flow_edge| flow_edge.source != node.id))
            {
                return Err(referential("callable flow node references an absent edge"));
            }
        }
        if analysis.flow_graph.entry().is_some_and(|entry| !flow_nodes.contains_key(&entry))
            || analysis.flow_graph.exits().iter().any(|exit| !flow_nodes.contains_key(exit))
        {
            return Err(referential("callable flow graph entry or exit references an absent node"));
        }
        for (_, node) in analysis.explanations.iter() {
            if node.parents.iter().any(|parent| !has_explanation(parent)) {
                return Err(referential("callable explanation references an absent parent"));
            }
            for evidence in &node.evidence {
                match evidence {
                    EvidenceRef::CallResolution(_) => return Err(referential("explanation call-resolution handle has no canonical artifact owner")),
                    EvidenceRef::BindingVersion { binding, .. } if !has_binding(binding) => {
                        return Err(referential("callable explanation references an absent binding"));
                    }
                    EvidenceRef::Suppressed { cause } if !has_cause(cause) => {
                        return Err(referential("callable explanation references an absent diagnostic cause"));
                    }
                    _ => {}
                }
            }
        }
        if analysis.return_explanation.is_some_and(|id| !has_explanation(&id)) {
            return Err(referential("callable return references an absent explanation"));
        }
        for diagnostic in analysis.diagnostics.iter() {
            if diagnostic.root_cause.is_some_and(|id| !has_cause(&id))
                || diagnostic
                    .explanations
                    .iter()
                    .any(|reference| &reference.callable != callable || !has_explanation(&reference.explanation))
            {
                return Err(referential("callable diagnostic references an absent owner-local cause or explanation"));
            }
        }
        for exit in &analysis.exits.normal_returns {
            if exit
                .direct_result_dependency
                .as_ref()
                .is_some_and(|dependency| !analysis.dependencies.contains(dependency))
            {
                return Err(referential("callable return references an absent direct-result dependency"));
            }
        }
        for flow in std::iter::once(&analysis.entry_flow)
            .chain(analysis.exits.normal_returns.iter().map(|exit| &exit.flow))
            .chain(analysis.exits.throws.iter())
        {
            if flow.bindings.keys().any(|binding| !has_binding(binding)) {
                return Err(referential("callable flow state references an absent binding"));
            }
        }

        Ok(Self {
            owner: to_stable_callable(callable),
            expressions,
            bindings,
            flow_nodes,
            flow_edges,
            explanations,
            diagnostic_causes,
            pattern_sites,
        })
    }
}

fn export_callable_analysis_record(
    callable_id: &crate::identity::CallableId,
    analysis: &crate::checker::analysis::CallableAnalysis,
    binding_source_ranges: &std::collections::BTreeMap<crate::identity::BindingId, phalcom_common::range::SourceRange>,
    types: &TypeExportContext,
) -> Result<CallableAnalysisRecordV1, ArtifactError> {
    if analysis.callable != *callable_id {
        return Err(referential("callable analysis table key does not match its owner"));
    }
    let ordinals = CallableAnalysisOrdinalMap::new(callable_id, analysis)?;
    if analysis.binding_origins.len() != analysis.bindings.len()
        || analysis.binding_origins.keys().any(|binding| !ordinals.bindings.contains_key(binding))
        || ordinals.bindings.keys().any(|binding| !analysis.binding_origins.contains_key(binding))
    {
        return Err(referential("callable binding storage-origin map is incomplete or contains a foreign binding"));
    }
    if binding_source_ranges.keys().any(|binding| !ordinals.bindings.contains_key(binding)) {
        return Err(referential("callable formal binding attachment references an absent owner-local binding"));
    }
    let flow_nodes = analysis
        .flow_graph
        .nodes()
        .map(|node| {
            use phalcom_compiler_artifact::semantic::world::{CallableFlowNodeKindV1, CallableFlowNodeV1};
            let kind = match node.kind {
                crate::checker::flow::graph::FlowNodeKind::Entry => CallableFlowNodeKindV1::Entry,
                crate::checker::flow::graph::FlowNodeKind::Exit => CallableFlowNodeKindV1::Exit,
                crate::checker::flow::graph::FlowNodeKind::Statement(index) => CallableFlowNodeKindV1::Statement {
                    index: u32::try_from(index).map_err(|_| referential("flow statement index exceeds portable range"))?,
                },
                crate::checker::flow::graph::FlowNodeKind::BranchCondition => CallableFlowNodeKindV1::BranchCondition,
                crate::checker::flow::graph::FlowNodeKind::LoopHeader => CallableFlowNodeKindV1::LoopHeader,
                crate::checker::flow::graph::FlowNodeKind::Join => CallableFlowNodeKindV1::Join,
                crate::checker::flow::graph::FlowNodeKind::Throw => CallableFlowNodeKindV1::Throw,
                crate::checker::flow::graph::FlowNodeKind::Unreachable => CallableFlowNodeKindV1::Unreachable,
            };
            Ok(CallableFlowNodeV1 {
                kind,
                range: phalcom_compiler_artifact::module_world::SpanV1 {
                    start: u64::try_from(node.range.start).map_err(|_| referential("flow node range start exceeds portable range"))?,
                    end: u64::try_from(node.range.end).map_err(|_| referential("flow node range end exceeds portable range"))?,
                },
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    let flow_edges = analysis
        .flow_graph
        .edges()
        .map(|edge| {
            use phalcom_compiler_artifact::semantic::world::CallableFlowEdgeKindV1;
            let kind = match edge.kind {
                crate::checker::flow::graph::FlowEdgeKind::Normal => CallableFlowEdgeKindV1::Normal,
                crate::checker::flow::graph::FlowEdgeKind::TrueBranch => CallableFlowEdgeKindV1::TrueBranch,
                crate::checker::flow::graph::FlowEdgeKind::FalseBranch => CallableFlowEdgeKindV1::FalseBranch,
                crate::checker::flow::graph::FlowEdgeKind::BackEdge => CallableFlowEdgeKindV1::BackEdge,
                crate::checker::flow::graph::FlowEdgeKind::Break => CallableFlowEdgeKindV1::Break,
                crate::checker::flow::graph::FlowEdgeKind::Continue => CallableFlowEdgeKindV1::Continue,
                crate::checker::flow::graph::FlowEdgeKind::Return => CallableFlowEdgeKindV1::Return,
                crate::checker::flow::graph::FlowEdgeKind::Throw => CallableFlowEdgeKindV1::Throw,
                crate::checker::flow::graph::FlowEdgeKind::Unreachable => CallableFlowEdgeKindV1::Unreachable,
            };
            Ok(phalcom_compiler_artifact::semantic::world::CallableFlowEdgeV1 {
                source: ordinals.flow_nodes[&edge.source],
                target: ordinals.flow_nodes[&edge.target],
                kind,
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    let flow_graph = phalcom_compiler_artifact::semantic::world::CallableFlowGraphV1 {
        nodes: flow_nodes,
        edges: flow_edges,
        entry: analysis.flow_graph.entry().map(|entry| ordinals.flow_nodes[&entry]),
        exits: analysis.flow_graph.exits().iter().map(|exit| ordinals.flow_nodes[exit]).collect(),
    };
    let mut expressions = Vec::with_capacity(analysis.expressions.len());
    for (expression_id, expression) in analysis.expressions.iter() {
        let denotation = expression
            .denotation
            .as_ref()
            .map(|denotation| export_semantic_denotation(denotation, types))
            .transpose()?;
        let call_specialization = expression
            .call_specialization
            .as_ref()
            .map(|specialization| -> Result<_, ArtifactError> {
                if expression.callable.as_ref().is_some_and(|callable| callable != &specialization.callable) {
                    return Err(referential("call specialization target disagrees with its expression callable"));
                }
                if !specialization.conformances.is_empty() {
                    return Err(referential(
                        "call specialization conformance bindings lack a complete portable owner representation",
                    ));
                }
                let mut bindings = specialization
                    .bindings
                    .iter()
                    .map(|(parameter, argument)| {
                        let argument = match argument {
                            crate::types::store::StaticArgument::Term(term) => {
                                phalcom_compiler_artifact::semantic::type_store::StaticArgumentV1::Term { term: types.ty(*term)? }
                            }
                            crate::types::store::StaticArgument::RecordRow(row) => {
                                phalcom_compiler_artifact::semantic::type_store::StaticArgumentV1::RecordRow { row: types.row(*row)? }
                            }
                            crate::types::store::StaticArgument::DynamicType => phalcom_compiler_artifact::semantic::type_store::StaticArgumentV1::DynamicType,
                        };
                        Ok(CallableStaticArgumentBindingV1 {
                            parameter: types.parameter(*parameter)?,
                            argument,
                        })
                    })
                    .collect::<Result<Vec<_>, ArtifactError>>()?;
                bindings.sort_unstable_by_key(|binding| binding.parameter);
                if bindings.windows(2).any(|pair| pair[0].parameter == pair[1].parameter) {
                    return Err(referential("call specialization has duplicate static-argument parameters"));
                }
                Ok(CallableCallSpecializationV1 {
                    callable: to_stable_callable(&specialization.callable),
                    bindings,
                })
            })
            .transpose()?;
        expressions.push(CallableExpressionRowV1 {
            ordinal: ordinals.expressions[expression_id],
            denotation,
            call_specialization,
        });
    }
    expressions.sort_unstable_by_key(|expression| expression.ordinal);

    let mut bindings = Vec::with_capacity(analysis.bindings.len());
    for (binding_id, state) in analysis.bindings.iter() {
        let origin = match analysis.binding_origins.get(binding_id) {
            Some(crate::checker::analysis::CallableBindingOrigin::BodyLocal) => CallableBindingOriginV1::BodyLocal,
            Some(crate::checker::analysis::CallableBindingOrigin::ModuleValue { symbol, import }) => CallableBindingOriginV1::ModuleValue {
                module: to_stable_module(&symbol.module),
                name: symbol.name.to_string(),
                import_binding: import.map(|id| id.0),
            },
            None => return Err(referential("callable binding is missing its storage origin")),
        };
        let formal_attachment = binding_source_ranges
            .get(binding_id)
            .map(|range| -> Result<_, ArtifactError> {
                if !matches!(&origin, CallableBindingOriginV1::BodyLocal) {
                    return Err(referential("module-value binding must not acquire a callable-local source declaration"));
                }
                Ok(CallableBindingSourceAttachmentV1 {
                    range: phalcom_compiler_artifact::module_world::SpanV1 {
                        start: u64::try_from(range.start).map_err(|_| referential("formal binding source start exceeds portable range"))?,
                        end: u64::try_from(range.end).map_err(|_| referential("formal binding source end exceeds portable range"))?,
                    },
                })
            })
            .transpose()?;
        let denotation = state
            .denotation
            .as_ref()
            .map(|denotation| export_semantic_denotation(denotation, types))
            .transpose()?;
        bindings.push(CallableBindingRowV1 {
            ordinal: ordinals.bindings[binding_id],
            denotation,
            formal_attachment,
            origin,
        });
    }
    bindings.sort_unstable_by_key(|binding| binding.ordinal);
    let constructor_super_initializer = analysis
        .constructor_super_initializer
        .as_ref()
        .map(|resolution| -> Result<_, ArtifactError> {
            Ok(phalcom_compiler_artifact::semantic::world::ConstructorSuperInitializerV1 {
                call_range: phalcom_compiler_artifact::module_world::SpanV1 {
                    start: u64::try_from(resolution.call_range.start).map_err(|_| referential("constructor super-call start exceeds portable range"))?,
                    end: u64::try_from(resolution.call_range.end).map_err(|_| referential("constructor super-call end exceeds portable range"))?,
                },
                initializer: to_stable_callable(&resolution.initializer),
            })
        })
        .transpose()?;
    Ok(CallableAnalysisRecordV1 {
        callable: to_stable_callable(callable_id),
        body_source_owner: analysis.body_source_owner.as_ref().map(|owner| match owner {
            crate::identity::SourceOwner::Module(module) => CallableAnalysisSourceOwnerV1::Module {
                module: to_stable_module(module),
            },
            crate::identity::SourceOwner::Callable(callable) => CallableAnalysisSourceOwnerV1::Callable {
                callable: to_stable_callable(callable),
            },
        }),
        body_range: phalcom_compiler_artifact::module_world::SpanV1 {
            start: u64::try_from(analysis.body_range.start).map_err(|_| referential("callable body range start exceeds portable range"))?,
            end: u64::try_from(analysis.body_range.end).map_err(|_| referential("callable body range end exceeds portable range"))?,
        },
        status: export_callable_analysis_status(analysis.status)?,
        exits_unreachable: analysis.exits.unreachable,
        flow_graph,
        constructor_super_initializer,
        expressions,
        bindings,
    })
}

fn enum_requirement_callable(requirement: &EnumRequirement, owner: &crate::identity::DeclarationId) -> Result<StableCallableRef, ArtifactError> {
    let expected = crate::identity::CallableId::new(
        crate::identity::CallableOwnerId::Declaration(owner.clone()),
        requirement.id.selector.clone(),
        crate::identity::DispatchSide::Instance,
    );
    if requirement.id.owner != *owner
        || requirement.signature.owner != *owner
        || requirement.signature.callable != expected
        || requirement.signature.side != crate::identity::DispatchSide::Instance
        || requirement.signature.selector != requirement.id.selector
    {
        return Err(referential("enum requirement identity disagrees with its canonical callable signature"));
    }
    Ok(to_stable_callable(&requirement.signature.callable))
}

fn export_rest_mode(mode: phalcom_ast::ast::RestMode) -> u8 {
    match mode {
        phalcom_ast::ast::RestMode::None => 0,
        phalcom_ast::ast::RestMode::Positional => 1,
        phalcom_ast::ast::RestMode::Labeled => 2,
        phalcom_ast::ast::RestMode::Complete => 3,
    }
}

pub(super) fn export_native_type_expr(ty: &phalcom_native_meta::TypeExprSpec) -> TypeExprSpecV1 {
    use phalcom_native_meta::TypeExprSpec as Native;
    match ty {
        Native::Unknown => TypeExprSpecV1::Unknown,
        Native::Never => TypeExprSpecV1::Never,
        Native::SelfType => TypeExprSpecV1::SelfType,
        Native::Universe(owner) => TypeExprSpecV1::Universe { owner: owner.name().into() },
        Native::Parameter(name) => TypeExprSpecV1::Parameter { name: (*name).into() },
        Native::Applied { origin, arguments } => TypeExprSpecV1::Applied {
            origin: Box::new(export_native_type_expr(origin)),
            arguments: arguments.iter().map(export_native_type_expr).collect(),
        },
        Native::Union(members) => TypeExprSpecV1::Union {
            members: members.iter().map(export_native_type_expr).collect(),
        },
        Native::Tuple(tuple) => TypeExprSpecV1::Tuple {
            tuple: ParameterTupleSpecV1 {
                positional: tuple.positional.iter().map(export_native_type_expr).collect(),
                labeled: tuple
                    .labeled
                    .iter()
                    .map(|parameter| LabeledParameterSpecV1 {
                        label: parameter.label.into(),
                        ty: Box::new(export_native_type_expr(parameter.ty)),
                    })
                    .collect(),
                rest: tuple.rest.map(|rest| RestParameterSpecV1 {
                    ty: rest.ty.map(|ty| Box::new(export_native_type_expr(ty))),
                }),
            },
        },
    }
}

pub(super) fn export_native_effects(effects: phalcom_native_meta::EffectSpec) -> EffectSpecV1 {
    match effects {
        phalcom_native_meta::EffectSpec::Unknown => EffectSpecV1::Unknown,
        phalcom_native_meta::EffectSpec::Pure => EffectSpecV1::Pure,
        phalcom_native_meta::EffectSpec::Known(effects) => EffectSpecV1::Known {
            effects: effects
                .iter()
                .map(|effect| match effect {
                    phalcom_native_meta::NativeEffect::Mutation => NativeEffectV1::Mutation,
                    phalcom_native_meta::NativeEffect::Io => NativeEffectV1::Io,
                    phalcom_native_meta::NativeEffect::Scheduling => NativeEffectV1::Scheduling,
                    phalcom_native_meta::NativeEffect::Reflection => NativeEffectV1::Reflection,
                    phalcom_native_meta::NativeEffect::Nondeterminism => NativeEffectV1::Nondeterminism,
                    phalcom_native_meta::NativeEffect::Blocking => NativeEffectV1::Blocking,
                })
                .collect(),
        },
    }
}

pub(super) fn export_native_raises(raises: phalcom_native_meta::RaisesSpec) -> RaisesSpecV1 {
    match raises {
        phalcom_native_meta::RaisesSpec::Unknown => RaisesSpecV1::Unknown,
        phalcom_native_meta::RaisesSpec::Known(types) => RaisesSpecV1::Known {
            types: types.iter().map(export_native_type_expr).collect(),
        },
    }
}

pub(super) fn export_return_flow(flow: phalcom_native_meta::ReturnFlowSpec) -> Result<ReturnFlowV1, ArtifactError> {
    Ok(match flow {
        phalcom_native_meta::ReturnFlowSpec::Value => ReturnFlowV1::Value,
        phalcom_native_meta::ReturnFlowSpec::Receiver => ReturnFlowV1::Receiver,
        phalcom_native_meta::ReturnFlowSpec::Argument(index) => ReturnFlowV1::Argument {
            index: u64::try_from(index).map_err(|_| referential("callable return flow index exceeds portable range"))?,
        },
        phalcom_native_meta::ReturnFlowSpec::Never => ReturnFlowV1::Never,
        phalcom_native_meta::ReturnFlowSpec::Unknown => ReturnFlowV1::Unknown,
    })
}

pub(super) fn export_native_lifecycle(lifecycle: phalcom_native_meta::NativeLifecycleSpec) -> NativeLifecycleSpecV1 {
    NativeLifecycleSpecV1 {
        since: lifecycle.since.map(Into::into),
        deprecated_since: lifecycle.deprecated_since.map(Into::into),
        replacement: lifecycle.replacement.map(Into::into),
    }
}

fn export_return_validation(validation: crate::signature::ReturnContractValidation) -> ReturnContractValidationV1 {
    match validation {
        crate::signature::ReturnContractValidation::NotApplicable => ReturnContractValidationV1::NotApplicable,
        crate::signature::ReturnContractValidation::Unchecked => ReturnContractValidationV1::Unchecked,
        crate::signature::ReturnContractValidation::Satisfied(EvidenceStatus::Established) => ReturnContractValidationV1::Satisfied {
            status: EvidenceStatusV1::Established,
        },
        crate::signature::ReturnContractValidation::Satisfied(EvidenceStatus::Assumed) => ReturnContractValidationV1::Satisfied {
            status: EvidenceStatusV1::Assumed,
        },
        crate::signature::ReturnContractValidation::Refuted => ReturnContractValidationV1::Refuted,
        crate::signature::ReturnContractValidation::Blocked => ReturnContractValidationV1::Blocked,
        crate::signature::ReturnContractValidation::DynamicBoundary => ReturnContractValidationV1::DynamicBoundary,
    }
}

pub(super) fn export_callable_signature(
    signature: &crate::signature::CallableSemanticSignature,
    types: &TypeExportContext,
) -> Result<CallableSignatureRecordV1, ArtifactError> {
    if signature.callable.owner.try_declaration().is_some_and(|owner| owner != &signature.owner) {
        return Err(referential("callable signature owner disagrees with its callable identity"));
    }
    let side = match signature.side {
        crate::identity::DispatchSide::Instance => 0,
        crate::identity::DispatchSide::Class => 1,
    };
    if signature.callable.side != signature.side || signature.callable.selector != signature.selector {
        return Err(referential("callable signature dispatch identity disagrees with its callable key"));
    }
    let generics = signature
        .generics
        .as_ref()
        .map(|generics| export_generic_signature(generics, types))
        .transpose()?;
    let mut parameters = Vec::with_capacity(signature.parameters.len());
    for (index, parameter) in signature.parameters.iter().enumerate() {
        let index = u32::try_from(index).map_err(|_| referential("callable parameter index exceeds portable range"))?;
        if parameter.id.callable != signature.callable || parameter.id.index != index {
            return Err(referential("callable parameter identity disagrees with its signature"));
        }
        let rest_mode = export_rest_mode(parameter.rest);
        let contract = match &parameter.contract {
            crate::signature::ParameterContract::Fixed(fact) => ParameterContractV1::Fixed {
                fact: export_declared_type_fact(fact, types)?,
            },
            crate::signature::ParameterContract::Rest { mode, contract } => ParameterContractV1::Rest {
                mode: export_rest_mode(*mode),
                contract: match contract {
                    crate::signature::RestValueContract::Unconstrained => RestValueContractV1::Unconstrained,
                    crate::signature::RestValueContract::Element(fact) => RestValueContractV1::Element {
                        fact: export_declared_type_fact(fact, types)?,
                    },
                    crate::signature::RestValueContract::Lane(ty) => RestValueContractV1::Lane { ty: types.ty(*ty)? },
                    crate::signature::RestValueContract::Invalid(reason) => RestValueContractV1::Invalid {
                        reason: export_unknown_reason(reason),
                    },
                },
            },
        };
        parameters.push(PortableCallableParameter {
            callable: to_stable_callable(&parameter.id.callable),
            index,
            local_name: parameter.local_name.clone(),
            external_label: parameter.external_label.clone(),
            declared_type: export_declared_type_fact(&parameter.declared_type, types)?,
            rest_mode,
            contract,
            source: parameter.source.as_ref().map(export_source_span).transpose()?,
        });
    }
    let inferred_return = signature
        .inferred_return
        .as_ref()
        .map(|knowledge| export_type_knowledge(knowledge, types))
        .transpose()?;
    let native_id = signature.native_id.map(|native_id| {
        let key = native_id.0;
        NativeSurfaceIdV1 {
            owner: key.owner.name().into(),
            side: match key.side {
                phalcom_native_meta::NativeDispatch::Instance => 0,
                phalcom_native_meta::NativeDispatch::Class => 1,
            },
            selector: key.selector.into(),
        }
    });
    let effects = export_native_effects(signature.effects);
    let raises = export_native_raises(signature.raises);
    let flow = export_return_flow(signature.flow)?;
    let lifecycle = export_native_lifecycle(signature.lifecycle);
    if signature.native_id.is_none()
        && (matches!(&effects, EffectSpecV1::Known { effects } if !effects.is_empty())
            || matches!(&raises, RaisesSpecV1::Known { types } if !types.is_empty())
            || lifecycle.since.is_some()
            || lifecycle.deprecated_since.is_some()
            || lifecycle.replacement.is_some())
    {
        return Err(referential(
            "non-catalog static native metadata cannot be reconstructed without an owned metadata representation",
        ));
    }
    Ok(CallableSignatureRecordV1 {
        callable: to_stable_callable(&signature.callable),
        owner: to_stable_declaration(&signature.owner),
        side,
        selector: signature.selector.to_string().into_boxed_str(),
        generics,
        parameters,
        declared_return: export_declared_type_fact(&signature.declared_return, types)?,
        return_validation: export_return_validation(signature.return_validation),
        inferred_return,
        source: signature.source.as_ref().map(export_source_span).transpose()?,
        implementation: match signature.implementation {
            phalcom_common::modifier::ImplementationKind::Phalcom => ImplementationKindV1::Phalcom,
            phalcom_common::modifier::ImplementationKind::Native => ImplementationKindV1::Native,
            phalcom_common::modifier::ImplementationKind::Foreign => ImplementationKindV1::Foreign,
            phalcom_common::modifier::ImplementationKind::Intrinsic => ImplementationKindV1::Intrinsic,
        },
        native_id,
        effects,
        raises,
        flow,
        lifecycle,
    })
}

pub(super) fn export_enum_requirements(
    requirements: &EnumRequirementTable,
    enum_semantics: &EnumSemanticTable,
    types: &TypeExportContext,
) -> Result<Vec<EnumRequirementOwnerV1>, ArtifactError> {
    if requirements.requirements.len() != enum_semantics.enums.len() || requirements.case_statuses.len() != requirements.requirements.len() {
        return Err(referential("enum requirement table does not cover the published enum owner set"));
    }

    let mut owners = Vec::with_capacity(requirements.requirements.len());
    for (owner, owner_requirements) in requirements.requirements.iter() {
        let enum_info = enum_semantics
            .enum_info(owner)
            .ok_or_else(|| referential("enum requirement owner has no structural enum authority"))?;
        let statuses = requirements
            .case_statuses
            .get(owner)
            .ok_or_else(|| referential("enum requirement owner has no case-status table entry"))?;
        let stable_owner = to_stable_declaration(owner);

        let mut requirement_records = Vec::with_capacity(owner_requirements.len());
        for (source_order, requirement) in owner_requirements.iter().enumerate() {
            let source_order = u32::try_from(source_order).map_err(|_| referential("enum requirement source order exceeds portable range"))?;
            let source = requirement
                .source
                .as_ref()
                .map(|source| -> Result<_, ArtifactError> {
                    if source.module != owner.module || source.range.start > source.range.end {
                        return Err(referential("enum requirement source span disagrees with its owner"));
                    }
                    Ok(EnumRequirementSourceV1 {
                        module: to_stable_module(&source.module),
                        range: phalcom_compiler_artifact::module_world::SpanV1 {
                            start: u64::try_from(source.range.start).map_err(|_| referential("enum requirement span start exceeds portable range"))?,
                            end: u64::try_from(source.range.end).map_err(|_| referential("enum requirement span end exceeds portable range"))?,
                        },
                    })
                })
                .transpose()?;
            requirement_records.push(EnumRequirementRecordV1 {
                requirement: enum_requirement_callable(requirement, owner)?,
                signature: export_callable_signature(&requirement.signature, types)?,
                source_order,
                source,
            });
        }
        requirement_records.sort_unstable_by(|left, right| left.requirement.cmp(&right.requirement));

        let expected_status_count = owner_requirements
            .len()
            .checked_mul(enum_info.variants.len())
            .ok_or_else(|| referential("enum requirement case status count overflow"))?;
        if statuses.len() != expected_status_count {
            return Err(referential("enum requirement case statuses do not cover requirements and variants"));
        }
        let mut status_records = Vec::with_capacity(statuses.len());
        if !enum_info.variants.is_empty() {
            for (index, status) in statuses.iter().enumerate() {
                let requirement_order = index / enum_info.variants.len();
                let variant_order = index % enum_info.variants.len();
                let requirement = &owner_requirements[requirement_order];
                let variant = &enum_info.variants[variant_order];
                if status.requirement != requirement.id || status.variant != *variant {
                    return Err(referential("enum requirement case status order disagrees with semantic owner order"));
                }
                let portable_status = match &status.status {
                    CaseRequirementStatus::Satisfied { implementation } => CaseRequirementStatusV1::Satisfied {
                        implementation: to_stable_callable(implementation),
                    },
                    CaseRequirementStatus::Missing => CaseRequirementStatusV1::Missing,
                    CaseRequirementStatus::Incompatible { implementation } => CaseRequirementStatusV1::Incompatible {
                        implementation: to_stable_callable(implementation),
                    },
                    CaseRequirementStatus::Blocked => CaseRequirementStatusV1::Blocked,
                };
                status_records.push(CaseRequirementResultV1 {
                    variant: to_stable_variant(variant),
                    requirement: enum_requirement_callable(requirement, owner)?,
                    requirement_order: u32::try_from(requirement_order).map_err(|_| referential("enum requirement order exceeds portable range"))?,
                    variant_order: u32::try_from(variant_order).map_err(|_| referential("enum variant order exceeds portable range"))?,
                    status: portable_status,
                });
            }
        }

        owners.push(EnumRequirementOwnerV1 {
            owner: stable_owner,
            requirements: requirement_records,
            case_statuses: status_records,
        });
    }
    owners.sort_unstable_by(|left, right| left.owner.cmp(&right.owner));

    for (owner, _) in requirements.case_statuses.iter() {
        if !requirements.requirements.contains_key(owner) {
            return Err(referential("enum case-status owner has no requirement owner row"));
        }
    }
    Ok(owners)
}

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

fn export_selector_kind(kind: phalcom_common::selector::SelectorKind) -> SelectorKindV1 {
    use phalcom_common::selector::SelectorKind as Native;
    match kind {
        Native::Getter => SelectorKindV1::Getter,
        Native::Setter => SelectorKindV1::Setter,
        Native::Method => SelectorKindV1::Method,
        Native::SubscriptGet => SelectorKindV1::SubscriptGet,
        Native::SubscriptSet => SelectorKindV1::SubscriptSet,
    }
}

fn export_selector_slot(slot: &phalcom_common::selector::SelectorSlot) -> SelectorSlotV1 {
    match slot {
        phalcom_common::selector::SelectorSlot::Positional => SelectorSlotV1::Positional,
        phalcom_common::selector::SelectorSlot::Label(label) => SelectorSlotV1::Label {
            label: label.clone().into_boxed_str(),
        },
    }
}

fn export_family_operation_shape(shape: &crate::types::family::FamilyOperationShape) -> FamilyOperationShapeV1 {
    FamilyOperationShapeV1 {
        kind: export_selector_kind(shape.kind),
        slots: shape.slots.iter().map(export_selector_slot).collect(),
    }
}

fn export_exact_selector(selector: &phalcom_common::selector::Selector) -> ExactSelectorV1 {
    ExactSelectorV1 {
        base: export_associated_selector(&selector.base),
        kind: export_selector_kind(selector.kind),
        slots: selector.slots.iter().map(export_selector_slot).collect(),
    }
}

fn export_selector_pattern(pattern: &phalcom_common::selector::SelectorPattern) -> SelectorPatternSpecV1 {
    use phalcom_common::selector::SelectorKindPattern as Native;
    SelectorPatternSpecV1 {
        base: export_associated_selector(&pattern.base),
        kind: match pattern.kind {
            Native::AnyNamed => SelectorKindPatternV1::AnyNamed,
            Native::NamedAccessors => SelectorKindPatternV1::NamedAccessors,
            Native::AnySubscript => SelectorKindPatternV1::AnySubscript,
            Native::Exact(kind) => SelectorKindPatternV1::Exact {
                kind: export_selector_kind(kind),
            },
        },
        prefix: pattern.prefix.iter().map(export_selector_slot).collect(),
        suffix: pattern.suffix.iter().map(export_selector_slot).collect(),
        has_gap: pattern.has_gap,
    }
}

fn export_invocation_target(target: &crate::identity::InvocationTargetId) -> InvocationTargetV1 {
    match target {
        crate::identity::InvocationTargetId::Behavioral(callable) => InvocationTargetV1::Behavioral {
            callable: to_stable_callable(callable),
        },
        crate::identity::InvocationTargetId::VariantConstructor(constructor) => InvocationTargetV1::VariantConstructor {
            variant: to_stable_variant(&constructor.variant),
        },
        crate::identity::InvocationTargetId::DataConstructor(constructor) => InvocationTargetV1::DataConstructor {
            owner: to_stable_declaration(&constructor.owner),
        },
    }
}

fn export_conditional_dispatch_selection(selection: &crate::dispatch::ConditionalDispatchSelection) -> ConditionalDispatchSelectionV1 {
    ConditionalDispatchSelectionV1 {
        impl_id: to_stable_impl(&selection.impl_id),
        callable: to_stable_callable(&selection.callable),
        declaring_owner: to_stable_declaration(&selection.declaring_owner),
        side: match selection.side {
            crate::identity::DispatchSide::Instance => 0,
            crate::identity::DispatchSide::Class => 1,
        },
    }
}

pub(super) fn export_semantic_denotation(
    denotation: &crate::types::denotation::SemanticDenotation,
    types: &TypeExportContext,
) -> Result<SemanticDenotationV1, ArtifactError> {
    use crate::types::denotation::{AssociatedValueDenotation, SemanticDenotation};
    Ok(match denotation {
        SemanticDenotation::TypeForm(ty) => SemanticDenotationV1::TypeForm { ty: types.ty(*ty)? },
        SemanticDenotation::Kind(kind) => SemanticDenotationV1::Kind { kind: types.kind(*kind)? },
        SemanticDenotation::StaticDomain(domain) => SemanticDenotationV1::StaticDomain {
            domain: types.domain(*domain)?,
        },
        SemanticDenotation::StaticTerm(term) => SemanticDenotationV1::StaticTerm { term: types.ty(*term)? },
        SemanticDenotation::StaticApplication(_) => {
            return Err(referential("checked static-application rows are not retained by a portable callable body"));
        }
        SemanticDenotation::AssociatedValue(value) => SemanticDenotationV1::AssociatedValue {
            value: match &**value {
                AssociatedValueDenotation::Exact {
                    owner_form,
                    lookup_owner,
                    member,
                    target,
                } => AssociatedValueDenotationV1::Exact {
                    owner_form: types.ty(*owner_form)?,
                    lookup_owner: to_stable_declaration(lookup_owner),
                    member: export_associated_member(member),
                    target: target.as_ref().as_ref().map(export_invocation_target),
                },
                AssociatedValueDenotation::Family {
                    owner_form,
                    lookup_owner,
                    family,
                    members,
                } => AssociatedValueDenotationV1::Family {
                    owner_form: types.ty(*owner_form)?,
                    lookup_owner: to_stable_declaration(lookup_owner),
                    family: AssociatedFamilyIdentityV1 {
                        owner: to_stable_declaration(&family.owner),
                        selector: export_associated_selector(&family.base),
                    },
                    members: members
                        .iter()
                        .map(|member| CapturedAssociatedMemberV1 {
                            operation: export_family_operation_shape(&member.operation),
                            member: export_associated_member(&member.member),
                            target: member.target.as_ref().map(export_invocation_target),
                        })
                        .collect(),
                },
            },
        },
        SemanticDenotation::BehavioralFamily(family) => SemanticDenotationV1::BehavioralFamily {
            family: BehavioralFamilyDenotationV1 {
                receiver_type: types.ty(family.receiver_type)?,
                receiver_application: family.receiver_application.map(|ty| types.ty(ty)).transpose()?,
                spec: match &family.spec {
                    crate::checker::associated::BehavioralFamilySpec::Exact(selector) => BehavioralFamilySpecV1::Exact {
                        selector: export_exact_selector(selector),
                    },
                    crate::checker::associated::BehavioralFamilySpec::Pattern(pattern) => BehavioralFamilySpecV1::Pattern {
                        pattern: export_selector_pattern(pattern),
                    },
                },
                members: family
                    .members
                    .iter()
                    .map(|member| CapturedBehavioralMemberV1 {
                        operation: export_family_operation_shape(&member.operation),
                        target: export_invocation_target(&member.target),
                        conditional: member.conditional.as_ref().map(export_conditional_dispatch_selection),
                    })
                    .collect(),
            },
        },
    })
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
) -> Result<SemanticWorldV2, ArtifactError> {
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
        callable_signatures.push(export_callable_signature(sig, types)?);
    }
    callable_signatures.sort_unstable_by(|left, right| left.callable.cmp(&right.callable));

    let mut callable_analyses = Vec::new();
    for (callable_id, analysis) in snapshot.callable_analyses.iter() {
        let source_callable = snapshot.callable_signatures.id_for_body(callable_id).unwrap_or_else(|| callable_id.clone());
        let mut binding_source_ranges = std::collections::BTreeMap::new();
        for (binding, state) in &analysis.bindings {
            match analysis.binding_origins.get(binding) {
                Some(crate::checker::analysis::CallableBindingOrigin::ModuleValue { .. }) if state.parameter.is_none() => continue,
                Some(crate::checker::analysis::CallableBindingOrigin::ModuleValue { .. }) => {
                    return Err(referential("callable parameter cannot originate from module storage"));
                }
                Some(crate::checker::analysis::CallableBindingOrigin::BodyLocal) => {}
                None => return Err(referential("callable binding lacks its explicit storage origin")),
            }
            if let Some(site) = snapshot.source_index.source_site_for_binding(callable_id, *binding) {
                binding_source_ranges.insert(*binding, site.range);
            } else if state
                .parameter
                .as_ref()
                .is_some_and(|parameter| !snapshot.source_index.has_parameter_source_site(&source_callable, parameter))
            {
                // Implicit receiver facts have no source declaration token;
                // retain their semantic binding row without inventing a site.
            } else {
                return Err(referential("callable source binding lacks its canonical formal attachment"));
            }
        }
        callable_analyses.push(export_callable_analysis_record(callable_id, analysis, &binding_source_ranges, types)?);
    }
    callable_analyses.sort_unstable_by(|left, right| left.callable.cmp(&right.callable));

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
    let enum_requirements = export_enum_requirements(&snapshot.enum_requirements, &snapshot.enum_semantics, types)?;

    let world = SemanticWorldV2 {
        type_graph,
        structural_contracts,
        enum_requirements,
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
    for (callable, analysis) in snapshot.callable_analyses.iter() {
        let local_ids = CallableAnalysisOrdinalMap::new(callable, analysis)?;
        if local_ids.owner != to_stable_callable(callable) {
            return Err(referential("callable analysis ordinal owner is not canonical"));
        }
        let _local_counts = local_ids.local_counts();
    }
    if !snapshot.callable_analyses.is_empty() {
        return Err(referential(
            "semantic artifact export does not yet encode complete callable analysis facts and cannot publish a complete image",
        ));
    }
    if !snapshot.static_case_catalogs.is_empty()
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
    let scratch_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| ArtifactError::Allocation(AllocationError::Overflow))?;
    let mut scratch_budget = AllocationBudget::new(scratch_limit);
    validate_query_image_closure_with_allocation_budget(&image, limits, &mut scratch_budget)?;
    Ok(image)
}

#[cfg(test)]
mod callable_analysis_status_tests {
    use super::*;

    #[test]
    fn callable_analysis_status_requires_exact_portable_disposition() {
        use crate::checker::analysis::CallableAnalysisStatus as Status;
        use phalcom_compiler_artifact::semantic::world::CallableAnalysisStatusV1 as PortableStatus;

        assert_eq!(export_callable_analysis_status(Status::Complete).unwrap(), PortableStatus::Complete);
        assert_eq!(export_callable_analysis_status(Status::Partial).unwrap(), PortableStatus::Partial);
        assert_eq!(export_callable_analysis_status(Status::Blocked).unwrap(), PortableStatus::Blocked);
        assert_eq!(export_callable_analysis_status(Status::Cancelled).unwrap(), PortableStatus::Cancelled);
        assert_eq!(export_callable_analysis_status(Status::BudgetExceeded).unwrap(), PortableStatus::BudgetExceeded);
        assert!(export_callable_analysis_status(Status::InternalFailure(crate::identity::InternalSemanticIncidentId(7))).is_err());
    }

    #[test]
    fn callable_analysis_record_preserves_status_termination_and_constructor_target() {
        use crate::checker::analysis::{BodyExitFacts, CallableAnalysis, CallableAnalysisStatus, ConstructorSuperCallResolution, FlowStateSummary};
        use crate::identity::{CallableId, DispatchSide};
        use crate::signature::ReturnContractValidation;
        use phalcom_common::range::SourceRange;
        use phalcom_common::selector::Selector;
        use phalcom_compiler_artifact::semantic::world::CallableAnalysisStatusV1 as PortableStatus;

        let owner = crate::identity::DeclarationId::new(
            crate::identity::ModuleId::universe(phalcom_modules::ModulePath::from_components(vec![
                phalcom_modules::ModuleComponent::from_identifier("artifact_callable").unwrap(),
            ])),
            "Owner".into(),
        );
        let callable = CallableId::method(owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let initializer = CallableId::method(owner.clone(), Selector::getter("initialize").unwrap(), DispatchSide::Instance);
        let mut analysis = CallableAnalysis {
            callable: callable.clone(),
            body_source_owner: None,
            body_range: SourceRange::default(),
            expressions: Default::default(),
            bindings: Default::default(),
            binding_origins: Default::default(),
            for_iteration_lanes: std::sync::Arc::new(Default::default()),
            positional_expansions: std::sync::Arc::new(Default::default()),
            known_invalid_positional_expansions: std::sync::Arc::new(Default::default()),
            sequence_literals: std::sync::Arc::new(Default::default()),
            associated_resolutions: std::sync::Arc::new(Default::default()),
            callable_reference_resolutions: std::sync::Arc::new(Default::default()),
            family_applications: std::sync::Arc::new(Default::default()),
            match_resolutions: std::sync::Arc::new(Default::default()),
            pattern_site_resolutions: std::sync::Arc::new(Default::default()),
            flow_graph: std::sync::Arc::new(crate::checker::flow::graph::FlowGraph::default()),
            entry_flow: FlowStateSummary::default(),
            exits: BodyExitFacts {
                normal_returns: Vec::new(),
                throws: Vec::new(),
                unreachable: true,
            },
            return_validation: ReturnContractValidation::NotApplicable,
            diagnostics: std::sync::Arc::from([]),
            internal_incidents: std::sync::Arc::from([]),
            explanations: std::sync::Arc::new(crate::explain::ExplanationArena::default()),
            return_explanation: None,
            dependencies: std::sync::Arc::from([]),
            semantic_dependencies: std::sync::Arc::from([]),
            constructor_super_initializer: Some(ConstructorSuperCallResolution {
                call_range: SourceRange { start: 7, end: 9 },
                initializer: initializer.clone(),
            }),
            dependency_fingerprint: crate::db::ProductFingerprint::new(123),
            status: CallableAnalysisStatus::Blocked,
        };
        analysis.body_source_owner = Some(crate::identity::SourceOwner::Module(callable.module().clone()));

        let expression_a = crate::identity::ExpressionId::new(crate::identity::BodyId(8), crate::identity::LocalExpressionId(7));
        let expression_b = crate::identity::ExpressionId::new(crate::identity::BodyId(8), crate::identity::LocalExpressionId(2));
        analysis
            .expressions
            .insert_source(
                expression_a,
                crate::checker::analysis::ExpressionAnalysis::ready(
                    expression_a,
                    SourceRange { start: 20, end: 21 },
                    crate::types::evidence::TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::NoTypeEvidence),
                ),
            )
            .unwrap();
        analysis
            .expressions
            .insert_source(
                expression_b,
                crate::checker::analysis::ExpressionAnalysis::ready(
                    expression_b,
                    SourceRange { start: 20, end: 21 },
                    crate::types::evidence::TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::NoTypeEvidence),
                ),
            )
            .unwrap();
        let binding = crate::identity::BindingId(19);
        analysis
            .bindings
            .insert_source(
                binding,
                crate::checker::analysis::BindingState::new(
                    binding,
                    "local",
                    SourceRange { start: 30, end: 35 },
                    None,
                    crate::types::evidence::TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::NoTypeEvidence),
                    true,
                ),
            )
            .unwrap();
        analysis
            .binding_origins
            .insert_source(binding, crate::checker::analysis::CallableBindingOrigin::BodyLocal)
            .unwrap();
        let pattern_site_a = crate::match_semantics::PatternSiteKey::new(crate::match_semantics::PatternSiteKind::IfLet, SourceRange { start: 40, end: 45 });
        let pattern_site_b = crate::match_semantics::PatternSiteKey::new(crate::match_semantics::PatternSiteKind::IfLet, SourceRange { start: 50, end: 55 });
        let mut pattern_sites = crate::match_semantics::PatternSiteResolutionIndex::default();
        for (key, range) in [
            (pattern_site_b, SourceRange { start: 50, end: 55 }),
            (pattern_site_a, SourceRange { start: 40, end: 45 }),
        ] {
            pattern_sites
                .insert_source(
                    key,
                    crate::match_semantics::PatternSiteResolution {
                        kind: key.kind,
                        range,
                        subject: crate::types::evidence::TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::NoTypeEvidence),
                        pattern: crate::match_semantics::PatternResolution::Wildcard,
                        bindings: Box::default(),
                        usefulness: crate::match_semantics::PatternUsefulness::Useful,
                    },
                )
                .unwrap();
        }
        analysis.pattern_site_resolutions = std::sync::Arc::new(pattern_sites);

        let mut store = crate::types::store::TypeStore::new();
        let parameter = store.intern_type_parameter(crate::types::parameter::TypeParameterData::new(
            crate::types::parameter::TypeParameterOwner::Callable(callable.clone()),
            0,
            "T",
            crate::types::id::KindId::TYPE,
        ));
        let call_expression = analysis.expressions.get_source_mut(&expression_a).unwrap().unwrap();
        call_expression.callable = Some(callable.clone());
        call_expression.call_specialization = Some(crate::checker::analysis::CallSpecialization {
            callable: callable.clone(),
            bindings: Box::new([(parameter, crate::types::store::StaticArgument::DynamicType)]),
            conformances: Box::default(),
        });
        let mut types = crate::types::store::artifact::TypeExportContext::new(&store);
        super::super::roots::register_callable_analysis_roots(&mut types, &analysis).unwrap();
        types.close_graph(&store, &ValidationLimits::default()).unwrap();
        let mut parameter_only = crate::types::store::artifact::TypeExportContext::new(&store);
        let parameter_only_lineage = parameter_only.expected_lineage();
        parameter_only
            .register_parameter_root(
                parameter_only_lineage,
                phalcom_compiler_artifact::semantic::type_store::CanonicalTypeRootKey {
                    family_tag: phalcom_compiler_artifact::semantic::type_store::CanonicalRootFamilyTag::CallableAnalysis,
                    owner: phalcom_compiler_artifact::semantic::type_store::StableOwnerRef::Callable(to_stable_callable(&callable)),
                    product_subkey: 100,
                },
                parameter,
            )
            .unwrap();
        parameter_only.close_graph(&store, &ValidationLimits::default()).unwrap();
        assert_eq!(
            types.project_graph(&store, &ValidationLimits::default()).unwrap(),
            parameter_only.project_graph(&store, &ValidationLimits::default()).unwrap(),
            "Dynamic adds no proper-type root beyond the parameter's own canonical form"
        );

        let ordinals = CallableAnalysisOrdinalMap::new(&callable, &analysis).expect("well-formed body owner map");
        assert_eq!(ordinals.owner, to_stable_callable(&callable));
        assert_eq!(ordinals.expressions.get(&expression_b), Some(&0));
        assert_eq!(ordinals.expressions.get(&expression_a), Some(&1));
        assert_eq!(ordinals.bindings.get(&binding), Some(&0));
        assert_eq!(ordinals.pattern_sites.get(&pattern_site_a), Some(&0));
        assert_eq!(ordinals.pattern_sites.get(&pattern_site_b), Some(&1));

        let mut allocation_history_permuted = analysis.clone();
        allocation_history_permuted.expressions = analysis.expressions.iter().rev().map(|(id, expression)| (*id, expression.clone())).collect();
        allocation_history_permuted.pattern_site_resolutions = std::sync::Arc::new(
            analysis
                .pattern_site_resolutions
                .iter()
                .rev()
                .map(|(key, resolution)| (*key, resolution.clone()))
                .collect(),
        );
        let permuted_ordinals = CallableAnalysisOrdinalMap::new(&callable, &allocation_history_permuted)
            .expect("reordered insertion history keeps the same callable-local coordinate map");
        assert_eq!(permuted_ordinals.expressions, ordinals.expressions);
        assert_eq!(permuted_ordinals.bindings, ordinals.bindings);
        assert_eq!(permuted_ordinals.flow_nodes, ordinals.flow_nodes);
        assert_eq!(permuted_ordinals.flow_edges, ordinals.flow_edges);
        assert_eq!(permuted_ordinals.explanations, ordinals.explanations);
        assert_eq!(permuted_ordinals.diagnostic_causes, ordinals.diagnostic_causes);
        assert_eq!(permuted_ordinals.pattern_sites, ordinals.pattern_sites);

        let other_callable = CallableId::method(owner.clone(), Selector::getter("other").unwrap(), DispatchSide::Class);
        assert!(
            CallableAnalysisOrdinalMap::new(&other_callable, &analysis).is_err(),
            "an owner cannot reuse another callable's ordinal context"
        );
        let mut other_analysis = analysis.clone();
        other_analysis.callable = other_callable.clone();
        let other_ordinals = CallableAnalysisOrdinalMap::new(&other_callable, &other_analysis).expect("other callable has its own local coordinate space");
        assert_eq!(other_ordinals.expressions.get(&expression_b), Some(&0));
        assert_ne!(ordinals.owner, other_ordinals.owner);

        let mut unowned_call = analysis.clone();
        unowned_call.expressions.get_source_mut(&expression_a).unwrap().unwrap().call = Some(crate::identity::CallResolutionId(4));
        assert!(CallableAnalysisOrdinalMap::new(&callable, &unowned_call).is_err());

        let mut unowned_predicate = analysis.clone();
        let mut flow = crate::checker::flow::graph::FlowGraph::new();
        let entry = flow.add_node(crate::checker::flow::graph::FlowNodeKind::Entry, SourceRange::default()).unwrap();
        let exit = flow.add_node(crate::checker::flow::graph::FlowNodeKind::Exit, SourceRange::default()).unwrap();
        flow.try_set_entry(Some(entry)).unwrap();
        flow.try_push_exit(exit).unwrap();
        flow.add_edge_with_kind(
            entry,
            exit,
            crate::checker::flow::graph::FlowEdgeKind::Normal,
            Some(crate::identity::PredicateId(4)),
        )
        .unwrap();
        unowned_predicate.flow_graph = std::sync::Arc::new(flow);
        assert!(CallableAnalysisOrdinalMap::new(&callable, &unowned_predicate).is_err());

        let binding_ranges = std::collections::BTreeMap::from([(binding, SourceRange { start: 3, end: 7 })]);
        let record = export_callable_analysis_record(&callable, &analysis, &binding_ranges, &types).unwrap();
        assert_eq!(record.status, PortableStatus::Blocked);
        assert_eq!(
            &record.body_source_owner,
            &Some(CallableAnalysisSourceOwnerV1::Module {
                module: to_stable_module(callable.module()),
            })
        );
        assert!(record.exits_unreachable);
        let constructor_call = record.constructor_super_initializer.unwrap();
        assert_eq!(constructor_call.call_range.start, 7);
        assert_eq!(constructor_call.call_range.end, 9);
        assert_eq!(constructor_call.initializer, to_stable_callable(&initializer));
        assert!(matches!(
            record.bindings.first().map(|entry| &entry.origin),
            Some(phalcom_compiler_artifact::semantic::world::CallableBindingOriginV1::BodyLocal)
        ));
        assert_eq!(record.expressions.len(), 2);
        let call = record.expressions[ordinals.expressions[&expression_a] as usize]
            .call_specialization
            .as_ref()
            .expect("expression row owns its call specialization");
        assert_eq!(call.bindings[0].parameter, types.parameter(parameter).unwrap());
        assert!(matches!(
            call.bindings[0].argument,
            phalcom_compiler_artifact::semantic::type_store::StaticArgumentV1::DynamicType
        ));
        let wrong_key = other_callable;
        assert!(export_callable_analysis_record(&wrong_key, &analysis, &binding_ranges, &types).is_err());
    }

    fn empty_callable_analysis(callable: crate::identity::CallableId) -> crate::checker::analysis::CallableAnalysis {
        use crate::checker::analysis::{BodyExitFacts, CallableAnalysis, CallableAnalysisStatus, FlowStateSummary};
        use crate::signature::ReturnContractValidation;
        use phalcom_common::range::SourceRange;
        use std::sync::Arc;

        CallableAnalysis {
            callable,
            body_range: SourceRange::default(),
            body_source_owner: None,
            expressions: Default::default(),
            bindings: Default::default(),
            binding_origins: Default::default(),
            for_iteration_lanes: Arc::new(Default::default()),
            positional_expansions: Arc::new(Default::default()),
            known_invalid_positional_expansions: Arc::new(Default::default()),
            sequence_literals: Arc::new(Default::default()),
            associated_resolutions: Arc::new(Default::default()),
            callable_reference_resolutions: Arc::new(Default::default()),
            family_applications: Arc::new(Default::default()),
            match_resolutions: Arc::new(Default::default()),
            pattern_site_resolutions: Arc::new(Default::default()),
            flow_graph: Arc::new(crate::checker::flow::graph::FlowGraph::default()),
            entry_flow: FlowStateSummary::default(),
            exits: BodyExitFacts {
                normal_returns: Vec::new(),
                throws: Vec::new(),
                unreachable: false,
            },
            return_validation: ReturnContractValidation::NotApplicable,
            diagnostics: Arc::from([]),
            internal_incidents: Arc::from([]),
            explanations: Arc::new(crate::explain::ExplanationArena::default()),
            return_explanation: None,
            dependencies: Arc::from([]),
            semantic_dependencies: Arc::from([]),
            constructor_super_initializer: None,
            dependency_fingerprint: crate::db::ProductFingerprint::new(0),
            status: CallableAnalysisStatus::Complete,
        }
    }

    #[test]
    fn callable_analysis_record_preserves_expression_denotations() {
        use crate::checker::analysis::ExpressionAnalysis;
        use crate::identity::{BodyId, CallableId, DeclarationId, DispatchSide, ExpressionId, LocalExpressionId, ModuleId};
        use crate::types::denotation::SemanticDenotation;
        use crate::types::evidence::{TypeKnowledge, UnknownReason};
        use crate::types::store::TypeStore;
        use crate::types::store::artifact::TypeExportContext;
        use phalcom_common::range::SourceRange;
        use phalcom_common::selector::Selector;

        let owner = DeclarationId::new(ModuleId::universe_root(), "DenotationOwner".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("factory").unwrap(), DispatchSide::Class);
        let expression_id = ExpressionId::new(BodyId(11), LocalExpressionId(4));
        let mut source = empty_callable_analysis(callable.clone());
        source
            .expressions
            .insert_source(
                expression_id,
                ExpressionAnalysis::ready(
                    expression_id,
                    SourceRange { start: 9, end: 13 },
                    TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                ),
            )
            .unwrap();
        let mut denoted = source.clone();
        let denotation_owner = DeclarationId::new(owner.module.clone(), "DenotationOnly".into());
        let mut store = TypeStore::new();
        let denotation_type = store.nominal_type(denotation_owner);
        denoted.expressions.get_source_mut(&expression_id).unwrap().unwrap().denotation = Some(SemanticDenotation::TypeForm(denotation_type));

        let mut types = TypeExportContext::new(&store);
        crate::artifact::roots::register_callable_analysis_roots(&mut types, &denoted).unwrap();
        types.close_graph(&store, &ValidationLimits::default()).unwrap();
        let empty_record = export_callable_analysis_record(&callable, &source, &BTreeMap::new(), &types).unwrap();
        let denoted_record = export_callable_analysis_record(&callable, &denoted, &BTreeMap::new(), &types).unwrap();

        assert_ne!(
            empty_record, denoted_record,
            "a denotation is owned expression data and must be represented in the callable-analysis record"
        );
    }
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
