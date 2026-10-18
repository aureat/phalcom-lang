//! Semantic-owned artifact export for structural primary contracts (`PERF003.C2.P7.T7.A`).

mod proof;
mod explanation;

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

fn export_trait_ref_identity(trait_ref: &crate::traits::TraitRef, types: &TypeExportContext) -> Result<TraitRefV1, ArtifactError> {
    Ok(TraitRefV1 {
        declaration: to_stable_declaration(&trait_ref.declaration),
        arguments: trait_ref.arguments.iter().map(|&argument| types.ty(argument)).collect::<Result<Vec<_>, _>>()?,
    })
}

pub fn export_callable_query_owner(
    key: &crate::db::QueryKey,
    types: &TypeExportContext,
) -> Result<phalcom_compiler_artifact::semantic::world::CallableQueryOwnerV1, ArtifactError> {
    use crate::db::QueryKey;
    use phalcom_compiler_artifact::semantic::world::{CallableFieldRefV1, CallableQueryOwnerV1 as Portable, ConformanceTargetV1};

    Ok(match key {
        QueryKey::CallableSignature(callable) => Portable::CallableSignature {
            callable: to_stable_callable(callable),
        },
        QueryKey::FieldSignature(field) => {
            let field = to_stable_field(field);
            Portable::FieldSignature {
                field: CallableFieldRefV1 {
                    owner: field.owner,
                    side: field.side,
                    name: field.name,
                },
            }
        }
        QueryKey::GenericConformance(key) => Portable::GenericConformance {
            subject: export_type_term(&key.subject, types)?,
            trait_ref: export_trait_ref_identity(&key.trait_ref, types)?,
        },
        QueryKey::TraitDispatchBucket(key) => Portable::TraitDispatchBucket {
            target_family: match &key.target_family {
                crate::trait_dispatch::TraitDispatchTargetFamily::Declaration(declaration) => ConformanceTargetV1::Declaration {
                    declaration: to_stable_declaration(declaration),
                },
                crate::trait_dispatch::TraitDispatchTargetFamily::ExactEnumCase(variant) => ConformanceTargetV1::ExactEnumCase {
                    variant: to_stable_variant(variant),
                },
            },
            selector: export_exact_selector(&key.selector),
            side: to_stable_dispatch_side(key.side),
        },
        unsupported => {
            return Err(referential(format!(
                "query owner kind {} is not admitted in callable semantic artifacts",
                unsupported.kind().as_str()
            )));
        }
    })
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
                        Ok::<_, ArtifactError>(phalcom_compiler_artifact::module_world::SpanV1 {
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

        // DeclarationTypeInfo retains the canonical class-object form beside
        // the source-facing declaration form. Preserve it in the portable
        // type prefix so import can relocate it without extending the frozen
        // store through an unbudgeted local interning mutation.
        let key = CanonicalTypeRootKey {
            family_tag: CanonicalRootFamilyTag::DeclarationShell,
            owner: StableOwnerRef::Declaration(to_stable_declaration(decl_id)),
            product_subkey: 1,
        };
        types.register_type_root(lineage, key, info.class_object_type)?;
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

#[cfg(test)]
mod structural_type_roots_tests {
    use super::*;
    use crate::declarations::NominalDeclarationHeader;
    use crate::identity::{DeclarationId, ModuleId};
    use crate::types::store::TypeStore;

    #[test]
    fn declaration_shell_roots_retain_class_object_types_in_the_frozen_prefix() {
        let limits = ValidationLimits::default();
        let declaration = DeclarationId::new(ModuleId::universe_root(), "ArtifactShellRoot".into());
        let mut source_store = TypeStore::new();
        let header = NominalDeclarationHeader::from_signature(&mut source_store, declaration.clone(), None);
        let class_object_type = header.class_object_type;
        let mut declarations = DeclarationTypeTable::new();
        declarations.insert(header.into_type_info(None));

        let mut roots = TypeExportContext::new(&source_store);
        register_structural_primary_roots(
            &mut roots,
            &DataSemanticTable::new(),
            &EnumSemanticTable::new(),
            &declarations,
            &MapTypeHierarchy::new(),
            &TypeAliasTable::new(),
        )
        .expect("register structural roots");
        roots.close_graph(&source_store, &limits).expect("close type roots");
        roots.ty(class_object_type).expect("class-object type is part of the portable graph");

        let graph = roots.project_graph(&source_store, &limits).expect("project type graph");
        let (imported_store, _) = TypeStore::import_type_graph(&graph, &limits).expect("import frozen type prefix");
        assert!(
            imported_store.class_object_type_id(&declaration).is_some(),
            "the frozen prefix must contain the class-object form used by declaration-shell import"
        );
    }
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

use crate::artifact::query::validate_query_image_closure_with_allocation_budget;
use crate::artifact::validate::compute_witness_layout_fingerprint;
use crate::canonical_image::CanonicalSemanticQueryImage;
use crate::enum_requirements::{CaseRequirementStatus, EnumRequirement, EnumRequirementTable};
use crate::snapshot::SemanticSnapshot;
use crate::static_value::StaticExportContext;
use phalcom_compiler_artifact::query::{DependencyEvidenceEntryV1, PortableQueryKeyV1, PortableQueryKindV1, QueryImageV1, QueryRootEntryV1};
use phalcom_compiler_artifact::semantic::type_store::TypeMemberSelectorV1;
use phalcom_compiler_artifact::semantic::world::{
    AssociatedFamilyIdentityV1, AssociatedFamilyKindV1, AssociatedFamilyRecordV1, AssociatedMemberV1, AssociatedStaticDomainRecordV1,
    AssociatedSurfaceRecordV1, AssociatedValueDenotationV1, BehavioralFamilyDenotationV1, BehavioralFamilySpecV1, CallableAnalysisRecordV1,
    CallableAnalysisSourceOwnerV1, CallableAnalysisStatusV1, CallableAssociatedTypeRequirementIdentityV1, CallableBindingAssumptionBasisV1,
    CallableBindingAuthorityV1, CallableBindingConsistencyV1, CallableBindingContractOriginV1, CallableBindingRowV1, CallableBlockReasonV1,
    CallableBudgetKindV1, CallableBudgetReportV1, CallableCallSpecializationV1, CallableCausalInvalidityV1, CallableDefinitionBodyProvenanceV1,
    CallableDefinitionOriginV1, CallableDefinitionRecordV1, CallableExpressionRowV1, CallableExpressionStatusV1, CallableGeneratedBindingReasonV1,
    CallableInitializationStateV1, CallableParameterIdentityV1, CallableParameterV1 as PortableCallableParameter, CallableRefutationReasonV1,
    CallableRequirementIdentityV1, CallableSignatureRecordV1, CallableStaticArgumentBindingV1, CallableSuppressionCauseV1, CapturedAssociatedMemberV1,
    CapturedBehavioralMemberV1, CaseRequirementResultV1, CaseRequirementStatusV1, ConditionalDispatchSelectionV1, ConformanceHeadRecordV1, ConformanceTargetV1,
    ConstructorInitializationRecordV1, ConstructorParamFieldV1, CoveringImplParameterBindingV1, CoveringImplSubstitutionV1, DeclarationSurfaceRecordV1,
    EffectSpecV1, EnumRequirementOwnerV1, EnumRequirementRecordV1, EnumRequirementSourceV1, EvidenceOriginV1, EvidenceSetV1, EvidenceStatusV1, ExactSelectorV1,
    FamilyOperationShapeV1, FieldSignatureV1, ImplementationKindV1, InherentBodyContextV1, IntrinsicTypeOperationV1, InvocationTargetV1,
    LabeledParameterSpecV1, NativeEffectV1, NativeLifecycleSpecV1, NativeSurfaceIdV1, ParameterContractV1, ParameterTupleSpecV1, RaisesSpecV1,
    RestParameterSpecV1, RestValueContractV1, ReturnContractValidationV1, ReturnFlowV1, SelectorKindPatternV1, SelectorKindV1, SelectorPatternSpecV1,
    SelectorSlotV1, SemanticDenotationV1, SemanticWorldV3, TraitAssociatedTypeRequirementV1, TraitSurfaceMemberV1, TraitSurfaceRecordV1, TypeExprSpecV1,
    TypeKnowledgeV1, TypeMemberDeclarationRecordV1, TypeMemberDomainV1, TypeMemberImplementationV1, TypeMemberParameterV1, WitnessPlanV1,
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

#[derive(Clone, Debug)]
enum CallableBindingSourceAuthority {
    Source {
        site: crate::identity::SourceSiteId,
        range: phalcom_common::range::SourceRange,
    },
    GeneratedReceiver,
}

fn export_binding_authority(
    origin: &crate::checker::analysis::CallableBindingOrigin,
    source: Option<&CallableBindingSourceAuthority>,
) -> Result<CallableBindingAuthorityV1, ArtifactError> {
    use crate::identity::SourceOwner;

    match (origin, source) {
        (crate::checker::analysis::CallableBindingOrigin::BodyLocal, Some(CallableBindingSourceAuthority::Source { site, range })) => {
            let site_owner = match &site.owner {
                SourceOwner::Module(module) => CallableAnalysisSourceOwnerV1::Module {
                    module: to_stable_module(module),
                },
                SourceOwner::Callable(callable) => CallableAnalysisSourceOwnerV1::Callable {
                    callable: to_stable_callable(callable),
                },
            };
            Ok(CallableBindingAuthorityV1::BodyLocalSource {
                site_owner,
                site_local: site.local.0,
                declaration_range: phalcom_compiler_artifact::module_world::SpanV1 {
                    start: u64::try_from(range.start).map_err(|_| referential("binding declaration start exceeds portable range"))?,
                    end: u64::try_from(range.end).map_err(|_| referential("binding declaration end exceeds portable range"))?,
                },
            })
        }
        (crate::checker::analysis::CallableBindingOrigin::BodyLocal, Some(CallableBindingSourceAuthority::GeneratedReceiver)) => {
            Ok(CallableBindingAuthorityV1::BodyLocalGenerated {
                reason: CallableGeneratedBindingReasonV1::ImplicitReceiver,
            })
        }
        (crate::checker::analysis::CallableBindingOrigin::BodyLocal, None) => {
            Err(referential("callable body-local binding lacks exact source or generated authority"))
        }
        (crate::checker::analysis::CallableBindingOrigin::ModuleValue { symbol, import }, None) => Ok(CallableBindingAuthorityV1::ModuleValue {
            module: to_stable_module(&symbol.module),
            name: symbol.name.to_string(),
            import_binding: import.map(|id| id.0),
        }),
        (crate::checker::analysis::CallableBindingOrigin::ModuleValue { .. }, Some(_)) => {
            Err(referential("module-value binding cannot acquire callable-local source authority"))
        }
    }
}

fn export_binding_budget(report: &crate::types::outcome::BudgetReport) -> CallableBudgetReportV1 {
    CallableBudgetReportV1 {
        kind: match report.kind {
            crate::types::outcome::BudgetKind::Steps => CallableBudgetKindV1::Steps,
            crate::types::outcome::BudgetKind::RelationPairs => CallableBudgetKindV1::RelationPairs,
            crate::types::outcome::BudgetKind::SccIterations => CallableBudgetKindV1::SccIterations,
            crate::types::outcome::BudgetKind::TypeDepth => CallableBudgetKindV1::TypeDepth,
            crate::types::outcome::BudgetKind::DiagnosticNotes => CallableBudgetKindV1::DiagnosticNotes,
            crate::types::outcome::BudgetKind::ResidualNodes => CallableBudgetKindV1::ResidualNodes,
        },
        limit: report.limit,
        used: report.used,
    }
}

fn export_callable_block_reason(reason: &crate::types::outcome::BlockReason, types: &TypeExportContext) -> Result<CallableBlockReasonV1, ArtifactError> {
    use crate::types::outcome::BlockReason;
    Ok(match reason {
        BlockReason::UnknownType(reason) => CallableBlockReasonV1::UnknownType {
            reason: export_unknown_reason(reason),
        },
        BlockReason::UnresolvedDependency(module) => CallableBlockReasonV1::UnresolvedDependency {
            project_source: module
                .project
                .source
                .0
                .to_str()
                .ok_or_else(|| referential("unresolved dependency project path is not portable UTF-8"))?
                .into(),
            module_path: module.path.components().iter().map(|part| part.to_string().into_boxed_str()).collect(),
        },
        BlockReason::InvalidAnnotation(code) => CallableBlockReasonV1::InvalidAnnotation {
            diagnostic_code: code.as_str().into(),
        },
        BlockReason::RecursiveFixpoint => CallableBlockReasonV1::RecursiveFixpoint,
        BlockReason::InferenceDidNotConverge => CallableBlockReasonV1::InferenceDidNotConverge,
        BlockReason::OpaqueNative(message) => CallableBlockReasonV1::OpaqueNative { message: message.clone() },
        BlockReason::ReflectionBoundary => CallableBlockReasonV1::ReflectionBoundary,
        BlockReason::UnavailableSemanticOwner(owner) => CallableBlockReasonV1::UnavailableSemanticOwner {
            owner: export_callable_query_owner(owner, types)?,
        },
        BlockReason::BudgetExceeded(report) => CallableBlockReasonV1::BudgetExceeded {
            report: export_binding_budget(report),
        },
        BlockReason::SuppressedDependency => CallableBlockReasonV1::SuppressedDependency,
    })
}

fn export_callable_expression_status(
    status: &crate::checker::analysis::AnalysisStatus,
    ordinals: &CallableAnalysisOrdinalMap,
    types: &TypeExportContext,
) -> Result<CallableExpressionStatusV1, ArtifactError> {
    use crate::checker::analysis::AnalysisStatus;

    Ok(match status {
        AnalysisStatus::Ready => CallableExpressionStatusV1::Ready,
        AnalysisStatus::Invalid(cause) => CallableExpressionStatusV1::Invalid {
            cause: *ordinals
                .diagnostic_causes
                .get(cause)
                .ok_or_else(|| referential("callable expression status references an absent diagnostic cause"))?,
        },
        AnalysisStatus::Suppressed(crate::checker::causal::SuppressionCause::One(cause)) => CallableExpressionStatusV1::Suppressed {
            cause: CallableSuppressionCauseV1::One {
                cause: *ordinals
                    .diagnostic_causes
                    .get(cause)
                    .ok_or_else(|| referential("callable expression suppression references an absent diagnostic cause"))?,
            },
        },
        AnalysisStatus::Suppressed(crate::checker::causal::SuppressionCause::Multiple) => CallableExpressionStatusV1::Suppressed {
            cause: CallableSuppressionCauseV1::Multiple,
        },
        AnalysisStatus::Blocked(reason) => CallableExpressionStatusV1::Blocked {
            reason: export_callable_block_reason(reason, types)?,
        },
        AnalysisStatus::DynamicBoundary(reason) => CallableExpressionStatusV1::DynamicBoundary {
            reason: export_dynamic_reason(reason),
        },
        AnalysisStatus::Cancelled => CallableExpressionStatusV1::Cancelled,
        AnalysisStatus::BudgetExceeded(report) => CallableExpressionStatusV1::BudgetExceeded {
            report: export_binding_budget(report),
        },
        AnalysisStatus::InternalFailure(_) => return Err(referential("expression internal incident has no canonical artifact owner")),
    })
}

fn export_generic_conformance_key(
    key: &crate::generic_evidence::GenericConformanceKey,
    types: &TypeExportContext,
) -> Result<phalcom_compiler_artifact::semantic::world::GenericConformanceKeyV1, ArtifactError> {
    Ok(phalcom_compiler_artifact::semantic::world::GenericConformanceKeyV1 {
        subject: export_type_term(&key.subject, types)?,
        trait_ref: export_trait_ref_identity(&key.trait_ref, types)?,
    })
}

fn export_callable_requirement_identity(
    requirement: &crate::traits::TraitRequirementId,
) -> phalcom_compiler_artifact::semantic::world::CallableRequirementIdentityV1 {
    phalcom_compiler_artifact::semantic::world::CallableRequirementIdentityV1 {
        owner: to_stable_declaration(&requirement.owner),
        selector: requirement.selector.encode().into_boxed_str(),
        side: match requirement.side {
            crate::identity::DispatchSide::Instance => 0,
            crate::identity::DispatchSide::Class => 1,
        },
    }
}

fn export_callable_associated_type_requirement_identity(
    requirement: &crate::traits::AssociatedTypeRequirementId,
) -> phalcom_compiler_artifact::semantic::world::CallableAssociatedTypeRequirementIdentityV1 {
    phalcom_compiler_artifact::semantic::world::CallableAssociatedTypeRequirementIdentityV1 {
        owner: to_stable_declaration(&requirement.owner),
        index: requirement.index,
    }
}

fn export_callable_type_environment(
    environment: &crate::types::environment::TypeEnvironment,
    types: &TypeExportContext,
) -> Result<phalcom_compiler_artifact::semantic::world::CallableTypeEnvironmentV1, ArtifactError> {
    use phalcom_compiler_artifact::semantic::type_store::StaticArgumentV1;
    use phalcom_compiler_artifact::semantic::world::CallableStaticArgumentBindingV1;

    let mut bindings = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for (&parameter, &ty) in &environment.bindings {
        if !seen.insert(parameter) {
            return Err(referential("callable conformance environment binds one parameter more than once"));
        }
        bindings.push(CallableStaticArgumentBindingV1 {
            parameter: types.parameter(parameter)?,
            argument: StaticArgumentV1::Term { term: types.ty(ty)? },
        });
    }
    for &parameter in &environment.dynamic_bindings {
        if !seen.insert(parameter) {
            return Err(referential("callable conformance environment binds one parameter more than once"));
        }
        bindings.push(CallableStaticArgumentBindingV1 {
            parameter: types.parameter(parameter)?,
            argument: StaticArgumentV1::DynamicType,
        });
    }
    for (&parameter, &row) in &environment.row_bindings {
        if !seen.insert(parameter) {
            return Err(referential("callable conformance environment binds one parameter more than once"));
        }
        bindings.push(CallableStaticArgumentBindingV1 {
            parameter: types.parameter(parameter)?,
            argument: StaticArgumentV1::RecordRow { row: types.row(row)? },
        });
    }
    bindings.sort_unstable_by_key(|binding| binding.parameter);
    Ok(phalcom_compiler_artifact::semantic::world::CallableTypeEnvironmentV1 {
        bindings,
        self_binding: environment.self_binding.map(|ty| types.ty(ty)).transpose()?,
    })
}

fn export_callable_type_bindings(
    bindings: impl IntoIterator<Item = (crate::types::id::TypeParameterId, crate::types::id::TypeId)>,
    types: &TypeExportContext,
) -> Result<Vec<phalcom_compiler_artifact::semantic::world::CallableTypeBindingV1>, ArtifactError> {
    let mut rows = bindings
        .into_iter()
        .map(|(parameter, ty)| {
            Ok(phalcom_compiler_artifact::semantic::world::CallableTypeBindingV1 {
                parameter: types.parameter(parameter)?,
                ty: types.ty(ty)?,
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    rows.sort_unstable_by_key(|row| row.parameter);
    if rows.windows(2).any(|pair| pair[0].parameter == pair[1].parameter) {
        return Err(referential("callable evidence substitution has duplicate parameters"));
    }
    Ok(rows)
}

fn export_callable_requirement_views<'a>(
    views: impl IntoIterator<Item = (&'a crate::traits::TraitRequirementId, &'a crate::traits::InstantiatedTraitRequirement)>,
    types: &TypeExportContext,
) -> Result<Vec<phalcom_compiler_artifact::semantic::world::CallableRequirementViewV1>, ArtifactError> {
    views
        .into_iter()
        .map(|(requirement, view)| {
            if &view.requirement != requirement {
                return Err(referential("callable requirement view key disagrees with its value"));
            }
            Ok(phalcom_compiler_artifact::semantic::world::CallableRequirementViewV1 {
                requirement: export_callable_requirement_identity(requirement),
                callable: to_stable_callable(&view.callable),
                signature: export_callable_signature(&view.signature, types)?,
                visibility: export_visibility(view.visibility),
                default_callable: view.default_callable.as_ref().map(to_stable_callable),
            })
        })
        .collect()
}

fn export_semantic_diagnostic(
    diagnostic: &crate::diagnostic::SemanticDiagnostic,
    ordinals: Option<&CallableAnalysisOrdinalMap>,
    portable_explanations: Option<&BTreeMap<crate::identity::CallableId, BTreeMap<crate::identity::ExplanationId, u32>>>,
    portable_causes: Option<&BTreeMap<crate::identity::DiagnosticCauseId, u32>>,
    types: &TypeExportContext,
) -> Result<phalcom_compiler_artifact::semantic::world::CallableEvidenceDiagnosticV1, ArtifactError> {
    use crate::diagnostic::{DiagnosticGuidance, DiagnosticSeverity};
    use phalcom_compiler_artifact::semantic::world as wire;

    let span = |range: phalcom_common::range::SourceRange| -> Result<phalcom_compiler_artifact::module_world::SpanV1, ArtifactError> {
        Ok(phalcom_compiler_artifact::module_world::SpanV1 {
            start: u64::try_from(range.start).map_err(|_| referential("callable diagnostic range start exceeds portable range"))?,
            end: u64::try_from(range.end).map_err(|_| referential("callable diagnostic range end exceeds portable range"))?,
        })
    };
    let labels = diagnostic
        .labels
        .iter()
        .map(|label| {
            Ok(wire::CallableEvidenceDiagnosticLabelV1 {
                source: export_source_span(&label.span)?,
                range: span(label.range)?,
                message: label.message.clone().into_boxed_str(),
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    let guidance = diagnostic
        .guidance
        .iter()
        .map(|guidance| {
            Ok(match guidance {
                DiagnosticGuidance::ChangeAnnotation { range, ty } => wire::CallableEvidenceDiagnosticGuidanceV1::ChangeAnnotation {
                    range: span(*range)?,
                    ty: types.ty(*ty)?,
                },
                DiagnosticGuidance::SupplyAssignableValue { expected } => wire::CallableEvidenceDiagnosticGuidanceV1::SupplyAssignableValue {
                    expected: types.ty(*expected)?,
                },
                DiagnosticGuidance::UseCallableShape { callable } => wire::CallableEvidenceDiagnosticGuidanceV1::UseCallableShape {
                    callable: to_stable_callable(callable),
                },
                DiagnosticGuidance::EstablishTypeEvidence { range, expected } => wire::CallableEvidenceDiagnosticGuidanceV1::EstablishTypeEvidence {
                    range: span(*range)?,
                    expected: expected.map(|ty| types.ty(ty)).transpose()?,
                },
                DiagnosticGuidance::ResolveGenericParameter { parameter } => wire::CallableEvidenceDiagnosticGuidanceV1::ResolveGenericParameter {
                    parameter: types.parameter(*parameter)?,
                },
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    let explanations = diagnostic
        .explanations
        .iter()
        .map(|reference| {
            let callable = to_stable_callable(&reference.callable);
            let ordinal = match ordinals {
                Some(ordinals) => {
                    if callable != ordinals.owner {
                        return Err(referential("evidence diagnostic references an explanation owned by another callable"));
                    }
                    ordinals
                        .explanations
                        .get(&reference.explanation)
                        .copied()
                        .ok_or_else(|| referential("evidence diagnostic references an absent callable explanation"))?
                }
                None => portable_explanations
                    .and_then(|by_callable| by_callable.get(&reference.callable))
                    .and_then(|ordinals| ordinals.get(&reference.explanation))
                    .copied()
                    .ok_or_else(|| referential("trait diagnostic references an absent callable explanation"))?,
            };
            Ok(wire::CallableEvidenceExplanationRefV1 { callable, ordinal })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    let fixes = diagnostic
        .fixes
        .iter()
        .map(|fix| {
            Ok(wire::CallableEvidenceDiagnosticFixV1 {
                message: fix.message.clone().into_boxed_str(),
                replacement: fix
                    .replacement
                    .as_ref()
                    .map(|(range, text)| {
                        Ok::<_, ArtifactError>(wire::CallableEvidenceDiagnosticReplacementV1 {
                            range: span(*range)?,
                            text: text.clone().into_boxed_str(),
                        })
                    })
                    .transpose()?,
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    Ok(wire::CallableEvidenceDiagnosticV1 {
        code: diagnostic.code.as_str().into(),
        severity: match diagnostic.severity {
            DiagnosticSeverity::Error => wire::CallableDiagnosticSeverityV1::Error,
            DiagnosticSeverity::Warning => wire::CallableDiagnosticSeverityV1::Warning,
            DiagnosticSeverity::Information => wire::CallableDiagnosticSeverityV1::Information,
            DiagnosticSeverity::Hint => wire::CallableDiagnosticSeverityV1::Hint,
        },
        message: diagnostic.message.clone().into_boxed_str(),
        primary: export_source_span(&diagnostic.primary)?,
        primary_range: span(diagnostic.primary_range)?,
        labels,
        notes: diagnostic.notes.iter().cloned().map(String::into_boxed_str).collect(),
        helps: diagnostic.helps.iter().cloned().map(String::into_boxed_str).collect(),
        guidance,
        explanations,
        fixes,
        root_cause: match (ordinals, diagnostic.root_cause) {
            (Some(ordinals), Some(cause)) => Some(
                ordinals
                    .diagnostic_causes
                    .get(&cause)
                    .copied()
                    .ok_or_else(|| referential("evidence diagnostic references an absent callable diagnostic cause"))?,
            ),
            (None, Some(cause)) => Some(
                portable_causes
                    .and_then(|causes| causes.get(&cause))
                    .copied()
                    .ok_or_else(|| referential("trait diagnostic references an unregistered diagnostic cause"))?,
            ),
            (_, None) => None,
        },
    })
}

struct CallableConformanceEvidenceExport<'a> {
    types: &'a TypeExportContext,
    ordinals: Option<&'a CallableAnalysisOrdinalMap>,
    portable_explanations: Option<&'a BTreeMap<crate::identity::CallableId, BTreeMap<crate::identity::ExplanationId, u32>>>,
    portable_causes: Option<&'a BTreeMap<crate::identity::DiagnosticCauseId, u32>>,
    max_nodes: usize,
    max_depth: usize,
    active_arc_addresses: std::collections::HashSet<usize>,
    nodes: Vec<phalcom_compiler_artifact::semantic::world::CallableConformanceEvidenceNodeV1>,
}

impl CallableConformanceEvidenceExport<'_> {
    fn push_node(&mut self, node: phalcom_compiler_artifact::semantic::world::CallableConformanceEvidenceNodeV1) -> Result<u32, ArtifactError> {
        if self.nodes.len() >= self.max_nodes {
            return Err(referential("callable conformance evidence node limit exceeded during export"));
        }
        let ordinal = u32::try_from(self.nodes.len()).map_err(|_| referential("callable evidence ordinal exceeds portable range"))?;
        self.nodes.push(node);
        Ok(ordinal)
    }

    fn binding(
        &mut self,
        binding: &crate::checker::analysis::GenericCallConformanceBinding,
        depth: usize,
    ) -> Result<phalcom_compiler_artifact::semantic::world::CallableGenericCallConformanceBindingV1, ArtifactError> {
        use crate::checker::analysis::GenericCallConformanceSource as Source;
        use phalcom_compiler_artifact::semantic::world as wire;
        if depth > self.max_depth {
            return Err(referential("callable conformance evidence nesting limit exceeded during export"));
        }
        let source = match &binding.source {
            Source::Exact { evidence } => wire::CallableGenericCallConformanceSourceV1::Exact {
                evidence: self.evidence(&crate::generic_evidence::GenericConformanceEvidence::Exact(evidence.clone()), depth + 1)?,
            },
            Source::Forwarded { caller_conformance } => wire::CallableGenericCallConformanceSourceV1::Forwarded {
                caller_conformance: export_generic_conformance_key(caller_conformance, self.types)?,
            },
            Source::Derived { evidence } => wire::CallableGenericCallConformanceSourceV1::Derived {
                evidence: self.evidence(&crate::generic_evidence::GenericConformanceEvidence::Derived(evidence.clone()), depth + 1)?,
            },
            Source::CompilerDerived { evidence } => wire::CallableGenericCallConformanceSourceV1::CompilerDerived {
                evidence: self.evidence(
                    &crate::generic_evidence::GenericConformanceEvidence::CompilerDerived(evidence.clone()),
                    depth + 1,
                )?,
            },
        };
        Ok(wire::CallableGenericCallConformanceBindingV1 {
            callee_conformance: export_generic_conformance_key(&binding.callee_conformance, self.types)?,
            source,
        })
    }

    fn evidence(&mut self, evidence: &crate::generic_evidence::GenericConformanceEvidence, depth: usize) -> Result<u32, ArtifactError> {
        use crate::generic_evidence::{DerivedConformanceOrigin, GenericConformanceEvidence, NestedGenericEvidence};
        use phalcom_compiler_artifact::semantic::world as wire;
        if depth > self.max_depth {
            return Err(referential("callable conformance evidence nesting limit exceeded during export"));
        }
        match evidence {
            GenericConformanceEvidence::Exact(exact) => {
                let address = std::sync::Arc::as_ptr(exact) as usize;
                if !self.active_arc_addresses.insert(address) {
                    return Err(referential("callable conformance evidence contains a recursive Arc cycle"));
                }
                let result = (|| {
                    let proofs = exact
                        .trait_header_conformance_proofs
                        .as_ref()
                        .map(|proofs| {
                            proofs
                                .iter()
                                .map(|(key, evidence)| {
                                    Ok(wire::CallableGenericConformanceProofRefV1 {
                                        key: export_generic_conformance_key(key, self.types)?,
                                        evidence: self.evidence(evidence, depth + 1)?,
                                    })
                                })
                                .collect::<Result<Vec<_>, ArtifactError>>()
                        })
                        .transpose()?;
                    let requirements = exact
                        .requirements
                        .iter()
                        .map(|(requirement, selection)| {
                            Ok(wire::CallableRequirementSelectionEntryV1 {
                                requirement: export_callable_requirement_identity(requirement),
                                selection: self.selection(selection, depth + 1)?,
                            })
                        })
                        .collect::<Result<Vec<_>, ArtifactError>>()?;
                    let mut associated_types = Vec::with_capacity(exact.associated_types.len());
                    for (requirement, binding) in &exact.associated_types {
                        if &binding.requirement != requirement {
                            return Err(referential("exact conformance associated-type key disagrees with its binding"));
                        }
                        associated_types.push(wire::CallableExactAssociatedTypeBindingV1 {
                            requirement: export_callable_associated_type_requirement_identity(requirement),
                            value: match binding.value {
                                crate::impls::AssociatedTypeBindingValue::Type(ty) => {
                                    wire::CallableAssociatedTypeBindingValueV1::Type { ty: self.types.ty(ty)? }
                                }
                                crate::impls::AssociatedTypeBindingValue::Dynamic => wire::CallableAssociatedTypeBindingValueV1::Dynamic,
                            },
                            source_impl: to_stable_impl(&binding.source_impl),
                            source: export_source_span(&binding.source)?,
                        });
                    }
                    let exact_wire = wire::CallableExactConformanceEvidenceV1 {
                        source_impl: to_stable_impl(&exact.source_impl),
                        exact_target: self.types.ty(exact.exact_target)?,
                        exact_trait_ref: export_trait_ref_identity(&exact.exact_trait_ref, self.types)?,
                        impl_environment: export_callable_type_environment(&exact.impl_environment, self.types)?,
                        requirement_views: export_callable_requirement_views(exact.requirement_views.iter(), self.types)?,
                        requirements,
                        associated_types,
                        trait_header_contract: exact
                            .trait_header_contract
                            .iter()
                            .map(|constraint| export_constraint(constraint, self.types))
                            .collect::<Result<Vec<_>, _>>()?,
                        trait_header_conformance_proofs: proofs,
                    };
                    self.push_node(wire::CallableConformanceEvidenceNodeV1::Exact { evidence: exact_wire })
                })();
                self.active_arc_addresses.remove(&address);
                result
            }
            GenericConformanceEvidence::Assumed(assumed) => self.push_node(wire::CallableConformanceEvidenceNodeV1::Assumed {
                key: export_generic_conformance_key(&assumed.key, self.types)?,
                owner: export_parameter_owner(&assumed.owner)?,
            }),
            GenericConformanceEvidence::Derived(derived) => {
                let address = std::sync::Arc::as_ptr(derived) as usize;
                if !self.active_arc_addresses.insert(address) {
                    return Err(referential("callable derived evidence contains a recursive Arc cycle"));
                }
                let result = (|| {
                    let origin = match &derived.origin {
                        DerivedConformanceOrigin::ConditionalSource { exact_source } => wire::CallableDerivedConformanceOriginV1::ConditionalSource {
                            exact_source: self.evidence(&GenericConformanceEvidence::Exact(exact_source.clone()), depth + 1)?,
                        },
                        DerivedConformanceOrigin::GuardedRule {
                            rule,
                            exact_subject,
                            exact_trait_ref,
                        } => {
                            let rule_wire = self.derived_rule(rule, depth + 1)?;
                            wire::CallableDerivedConformanceOriginV1::GuardedRule {
                                rule: rule_wire,
                                exact_subject: self.types.ty(*exact_subject)?,
                                exact_trait_ref: export_trait_ref_identity(exact_trait_ref, self.types)?,
                            }
                        }
                    };
                    let mut bindings = derived
                        .impl_bindings
                        .bindings()
                        .iter()
                        .map(|(&parameter, &ty)| (parameter, ty))
                        .collect::<Vec<_>>();
                    bindings.sort_unstable_by_key(|(parameter, _)| *parameter);
                    let mut nested = Vec::with_capacity(derived.nested.len());
                    for item in &derived.nested {
                        nested.push(match item {
                            NestedGenericEvidence::Conformance(evidence) => wire::CallableNestedGenericEvidenceV1::Conformance {
                                evidence: self.evidence(evidence, depth + 1)?,
                            },
                            NestedGenericEvidence::Equivalent => wire::CallableNestedGenericEvidenceV1::Equivalent,
                            NestedGenericEvidence::Subtype => wire::CallableNestedGenericEvidenceV1::Subtype,
                        });
                    }
                    self.push_node(wire::CallableConformanceEvidenceNodeV1::Derived {
                        evidence: wire::CallableDerivedConformanceEvidenceV1 {
                            origin,
                            impl_bindings: export_callable_type_bindings(bindings, self.types)?,
                            nested,
                        },
                    })
                })();
                self.active_arc_addresses.remove(&address);
                result
            }
            GenericConformanceEvidence::CompilerDerived(derived) => {
                let classification = match &derived.classification {
                    crate::structural::StructuralClassification::Unsupported => wire::CallableStructuralClassificationV1::Unsupported,
                    crate::structural::StructuralClassification::OrderedProduct(form) => wire::CallableStructuralClassificationV1::OrderedProduct {
                        form: match form {
                            crate::structural::OrderedProductForm::Unit => wire::CallableOrderedProductFormV1::Unit,
                            crate::structural::OrderedProductForm::Tuple => wire::CallableOrderedProductFormV1::Tuple,
                            crate::structural::OrderedProductForm::TupleFromPacks => wire::CallableOrderedProductFormV1::TupleFromPacks,
                            crate::structural::OrderedProductForm::Pack => wire::CallableOrderedProductFormV1::Pack,
                            crate::structural::OrderedProductForm::PackFromPacks => wire::CallableOrderedProductFormV1::PackFromPacks,
                            crate::structural::OrderedProductForm::Data { owner } => wire::CallableOrderedProductFormV1::Data {
                                owner: to_stable_declaration(owner),
                            },
                        },
                    },
                    crate::structural::StructuralClassification::UnorderedProduct(form) => wire::CallableStructuralClassificationV1::UnorderedProduct {
                        form: match form {
                            crate::structural::UnorderedProductForm::Record => wire::CallableUnorderedProductFormV1::Record,
                            crate::structural::UnorderedProductForm::Data { owner } => wire::CallableUnorderedProductFormV1::Data {
                                owner: to_stable_declaration(owner),
                            },
                        },
                    },
                    crate::structural::StructuralClassification::Sum { owner } => wire::CallableStructuralClassificationV1::Sum {
                        owner: to_stable_declaration(owner),
                    },
                };
                let kind = match derived.kind {
                    crate::generic_evidence::CompilerDerivedConformanceKind::Product => wire::CallableCompilerDerivedConformanceKindV1::Product,
                    crate::generic_evidence::CompilerDerivedConformanceKind::OrderedProduct => wire::CallableCompilerDerivedConformanceKindV1::OrderedProduct,
                    crate::generic_evidence::CompilerDerivedConformanceKind::UnorderedProduct => {
                        wire::CallableCompilerDerivedConformanceKindV1::UnorderedProduct
                    }
                    crate::generic_evidence::CompilerDerivedConformanceKind::Sum => wire::CallableCompilerDerivedConformanceKindV1::Sum,
                };
                self.push_node(wire::CallableConformanceEvidenceNodeV1::CompilerDerived {
                    evidence: wire::CallableCompilerDerivedConformanceEvidenceV1 {
                        key: export_generic_conformance_key(&derived.key, self.types)?,
                        kind,
                        classification,
                    },
                })
            }
        }
    }

    fn selection(
        &mut self,
        selection: &crate::impls::RequirementSelectionTemplate,
        depth: usize,
    ) -> Result<phalcom_compiler_artifact::semantic::world::CallableRequirementSelectionV1, ArtifactError> {
        use crate::impls::RequirementSelectionTemplate as Native;
        use phalcom_compiler_artifact::semantic::world as wire;
        if depth > self.max_depth {
            return Err(referential("callable requirement-selection nesting limit exceeded during export"));
        }
        Ok(match selection {
            Native::ConformanceCallable { callable } => wire::CallableRequirementSelectionV1::ConformanceCallable {
                callable: to_stable_callable(callable),
            },
            Native::InherentCallable {
                callable,
                conditional_impl,
                applicability,
            } => wire::CallableRequirementSelectionV1::InherentCallable {
                callable: to_stable_callable(callable),
                conditional_impl: conditional_impl.as_ref().map(to_stable_impl),
                applicability: applicability.as_ref().map(|value| self.inherent_specialization(value, depth + 1)).transpose()?,
            },
            Native::ConditionalInherent { candidate, pending, fallback } => wire::CallableRequirementSelectionV1::ConditionalInherent {
                candidate: self.effective_inherent_witness(candidate, depth + 1)?,
                pending: self.completeness(pending)?,
                fallback: fallback
                    .as_deref()
                    .map(|selection| self.selection(selection, depth + 1).map(Box::new))
                    .transpose()?,
            },
            Native::DataComponent { component, specialized_type } => wire::CallableRequirementSelectionV1::DataComponent {
                owner: to_stable_declaration(&component.owner),
                index: component.index,
                specialized_type: self.types.ty(*specialized_type)?,
            },
            Native::TraitDefault { callable } => wire::CallableRequirementSelectionV1::TraitDefault {
                callable: to_stable_callable(callable),
            },
        })
    }

    fn derived_rule(
        &mut self,
        rule: &crate::derived_conformance::DerivedConformanceRule,
        depth: usize,
    ) -> Result<phalcom_compiler_artifact::semantic::world::CallableDerivedConformanceRuleV1, ArtifactError> {
        use phalcom_compiler_artifact::semantic::world as wire;
        if depth > self.max_depth {
            return Err(referential("derived conformance rule nesting limit exceeded during export"));
        }
        if rule.associated_type_plan.impl_id != rule.impl_id {
            return Err(referential("derived conformance associated-type plan disagrees with its rule owner"));
        }
        let mut requirements = Vec::with_capacity(rule.requirements.len());
        for (requirement, selection) in &rule.requirements {
            if requirement.owner != rule.trait_ref.declaration {
                return Err(referential("derived rule selection belongs to another trait"));
            }
            requirements.push(wire::CallableRequirementSelectionEntryV1 {
                requirement: export_callable_requirement_identity(requirement),
                selection: self.selection(selection, depth + 1)?,
            });
        }
        Ok(wire::CallableDerivedConformanceRuleV1 {
            impl_id: to_stable_impl(&rule.impl_id),
            source_module: to_stable_module(&rule.source_module),
            target_parameter: self.types.parameter(rule.target_parameter)?,
            target_template: self.types.ty(rule.target_template)?,
            trait_ref: export_trait_ref_identity(&rule.trait_ref, self.types)?,
            generic_signature: export_generic_signature(&rule.generic_signature, self.types)?,
            associated_type_plan: self.associated_type_plan(&rule.associated_type_plan)?,
            requirement_views: export_callable_requirement_views(rule.requirement_views.iter(), self.types)?,
            requirements,
            source: export_source_span(&rule.source)?,
        })
    }

    fn inherent_specialization(
        &mut self,
        specialization: &crate::impls::InherentImplSpecialization,
        depth: usize,
    ) -> Result<phalcom_compiler_artifact::semantic::world::CallableInherentImplSpecializationV1, ArtifactError> {
        let mut bindings = specialization.bindings.iter().map(|(&parameter, &ty)| (parameter, ty)).collect::<Vec<_>>();
        bindings.sort_unstable_by_key(|(parameter, _)| *parameter);
        Ok(phalcom_compiler_artifact::semantic::world::CallableInherentImplSpecializationV1 {
            impl_id: to_stable_impl(&specialization.impl_id),
            receiver: self.types.ty(specialization.receiver)?,
            owner_view: self.types.ty(specialization.owner_view)?,
            bindings: export_callable_type_bindings(bindings, self.types)?,
            environment: export_callable_type_environment(&specialization.environment, self.types)?,
            conformance_provenance: specialization
                .conformance_provenance
                .as_ref()
                .map(|bindings| bindings.iter().map(|binding| self.binding(binding, depth + 1)).collect())
                .transpose()?,
        })
    }

    fn conditional_capability(
        &mut self,
        capability: &crate::dispatch::ConditionalSelectedCallCapability,
        depth: usize,
    ) -> Result<phalcom_compiler_artifact::semantic::world::CallableConditionalSelectedCallCapabilityV1, ArtifactError> {
        if capability.selection.impl_id != capability.applicability.impl_id {
            return Err(referential("conditional call capability applicability belongs to another implementation"));
        }
        Ok(phalcom_compiler_artifact::semantic::world::CallableConditionalSelectedCallCapabilityV1 {
            selection: export_conditional_dispatch_selection(&capability.selection),
            applicability: self.inherent_specialization(&capability.applicability, depth + 1)?,
        })
    }

    fn trait_dispatch_site(
        &mut self,
        site: &crate::trait_dispatch::TraitDispatchSite,
        depth: usize,
    ) -> Result<phalcom_compiler_artifact::semantic::world::CallableTraitDispatchSiteV1, ArtifactError> {
        use crate::trait_dispatch::TraitDispatchSite;
        use phalcom_compiler_artifact::semantic::world as wire;

        Ok(match site {
            TraitDispatchSite::AbstractRequirement { requirement } => wire::CallableTraitDispatchSiteV1::AbstractRequirement {
                requirement: export_callable_requirement_identity(requirement),
            },
            TraitDispatchSite::Assumed(selection) => wire::CallableTraitDispatchSiteV1::Assumed {
                conformance: export_generic_conformance_key(&selection.conformance, self.types)?,
                requirement: export_callable_requirement_identity(&selection.requirement),
                signature: export_callable_signature(&selection.signature, self.types)?,
            },
            TraitDispatchSite::Evidenced(selection) => wire::CallableTraitDispatchSiteV1::Evidenced {
                selection: self.trait_dispatch_selection(selection, depth + 1)?,
            },
        })
    }

    fn trait_dispatch_selection(
        &mut self,
        selection: &crate::trait_dispatch::TraitDispatchSelection,
        depth: usize,
    ) -> Result<phalcom_compiler_artifact::semantic::world::CallableTraitDispatchSelectionV1, ArtifactError> {
        use phalcom_compiler_artifact::semantic::world as wire;
        let expected_fingerprint = crate::db::fingerprint::generic_conformance_evidence_fingerprint(&selection.evidence);
        if selection.evidence_fingerprint != expected_fingerprint {
            return Err(referential("trait dispatch selection carries a stale evidence fingerprint"));
        }
        let mut requirement_targets = Vec::with_capacity(selection.requirement_targets.len());
        for (requirement, target) in &selection.requirement_targets {
            requirement_targets.push(wire::CallableRequirementSelectionEntryV1 {
                requirement: export_callable_requirement_identity(requirement),
                selection: self.selection(target, depth + 1)?,
            });
        }
        Ok(wire::CallableTraitDispatchSelectionV1 {
            source_impl: to_stable_impl(&selection.source_impl),
            exact_target: self.types.ty(selection.exact_target)?,
            exact_trait_ref: export_trait_ref_identity(&selection.exact_trait_ref, self.types)?,
            requirement: export_callable_requirement_identity(&selection.requirement),
            callable: selection.callable.as_ref().map(to_stable_callable),
            signature: export_callable_signature(&selection.signature, self.types)?,
            evidence: self.evidence(&selection.evidence, depth + 1)?,
            selection: self.selection(&selection.selection, depth + 1)?,
            requirement_targets,
        })
    }

    fn for_iteration_requirement(
        &mut self,
        requirement: &crate::checker::analysis::ForIterationRequirementResolution,
        depth: usize,
    ) -> Result<phalcom_compiler_artifact::semantic::world::CallableForIterationRequirementV1, ArtifactError> {
        use crate::checker::analysis::ForIterationRequirementResolution as Native;
        use phalcom_compiler_artifact::semantic::world::CallableForIterationRequirementV1 as Portable;
        Ok(match requirement {
            Native::Evidence {
                conformance,
                evidence,
                requirement,
                signature,
            } => Portable::Evidence {
                conformance: export_generic_conformance_key(conformance, self.types)?,
                evidence: self.evidence(evidence, depth + 1)?,
                requirement: export_callable_requirement_identity(requirement),
                signature: export_callable_signature(signature, self.types)?,
            },
            Native::AbstractRequirement { requirement, signature } => Portable::AbstractRequirement {
                requirement: export_callable_requirement_identity(requirement),
                signature: export_callable_signature(signature, self.types)?,
            },
        })
    }

    fn effective_inherent_witness(
        &mut self,
        witness: &crate::impls::EffectiveInherentWitness,
        depth: usize,
    ) -> Result<phalcom_compiler_artifact::semantic::world::CallableEffectiveInherentWitnessV1, ArtifactError> {
        Ok(phalcom_compiler_artifact::semantic::world::CallableEffectiveInherentWitnessV1 {
            callable: to_stable_callable(&witness.callable),
            signature: export_callable_signature(&witness.signature, self.types)?,
            visibility: export_visibility(witness.visibility),
            owner: to_stable_declaration(&witness.owner),
            conditional_impl: witness.conditional_impl.as_ref().map(to_stable_impl),
            applicability: witness
                .applicability
                .as_ref()
                .map(|value| self.inherent_specialization(value, depth + 1))
                .transpose()?,
            conditional_domain: witness.conditional_domain.as_ref().map(|domain| self.inherent_domain(domain)).transpose()?,
        })
    }

    fn inherent_domain(
        &self,
        domain: &crate::impls::InherentImplDomain,
    ) -> Result<phalcom_compiler_artifact::semantic::world::CallableInherentImplDomainV1, ArtifactError> {
        use phalcom_compiler_artifact::semantic::world::CallableInherentImplTargetV1 as Target;
        Ok(phalcom_compiler_artifact::semantic::world::CallableInherentImplDomainV1 {
            impl_id: to_stable_impl(&domain.impl_id),
            target: match &domain.target {
                crate::impls::InherentImplTarget::Declaration(declaration) => Target::Declaration {
                    declaration: to_stable_declaration(declaration),
                },
                crate::impls::InherentImplTarget::ExactEnumCase(variant) => Target::ExactEnumCase {
                    variant: to_stable_variant(variant),
                },
            },
            head_type: self.types.ty(domain.head_type)?,
            generic_signature: domain
                .generic_signature
                .as_ref()
                .map(|signature| export_generic_signature(signature, self.types))
                .transpose()?,
            constraints: domain
                .constraints
                .iter()
                .map(|constraint| export_constraint(constraint, self.types))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    fn associated_type_plan(
        &self,
        plan: &crate::impls::ConformanceAssociatedTypePlan,
    ) -> Result<phalcom_compiler_artifact::semantic::world::CallableConformanceAssociatedTypePlanV1, ArtifactError> {
        use phalcom_compiler_artifact::semantic::world as wire;
        let mut bindings = Vec::with_capacity(plan.bindings.len());
        for (requirement, binding) in &plan.bindings {
            if &binding.requirement != requirement {
                return Err(referential("associated-type plan key disagrees with its binding"));
            }
            bindings.push(wire::CallableAssociatedTypeBindingTemplateV1 {
                requirement: export_callable_associated_type_requirement_identity(requirement),
                value: match binding.value_template {
                    crate::impls::AssociatedTypeBindingValue::Type(ty) => wire::CallableAssociatedTypeBindingValueV1::Type { ty: self.types.ty(ty)? },
                    crate::impls::AssociatedTypeBindingValue::Dynamic => wire::CallableAssociatedTypeBindingValueV1::Dynamic,
                },
                source_impl: to_stable_impl(&binding.source_impl),
                source: export_source_span(&binding.source)?,
            });
        }
        let failures = plan
            .failures
            .iter()
            .map(|failure| {
                Ok(wire::CallableAssociatedTypeBindingFailureV1 {
                    kind: match failure.kind {
                        crate::impls::AssociatedTypeBindingFailureKind::Missing => wire::CallableAssociatedTypeBindingFailureKindV1::Missing,
                        crate::impls::AssociatedTypeBindingFailureKind::Duplicate => wire::CallableAssociatedTypeBindingFailureKindV1::Duplicate,
                        crate::impls::AssociatedTypeBindingFailureKind::Unknown => wire::CallableAssociatedTypeBindingFailureKindV1::Unknown,
                        crate::impls::AssociatedTypeBindingFailureKind::Invalid => wire::CallableAssociatedTypeBindingFailureKindV1::Invalid,
                    },
                    name: failure.name.clone(),
                    reason: failure.reason.clone(),
                    source: export_source_span(&failure.source)?,
                    requirement: failure.requirement.as_ref().map(export_callable_associated_type_requirement_identity),
                })
            })
            .collect::<Result<Vec<_>, ArtifactError>>()?;
        Ok(wire::CallableConformanceAssociatedTypePlanV1 {
            impl_id: to_stable_impl(&plan.impl_id),
            bindings,
            failures,
            diagnostics: plan
                .diagnostics
                .iter()
                .map(|diagnostic| export_semantic_diagnostic(diagnostic, self.ordinals, self.portable_explanations, self.portable_causes, self.types))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    fn completeness(
        &self,
        state: &crate::impls::ConformanceCompleteness,
    ) -> Result<phalcom_compiler_artifact::semantic::world::CallableConformanceCompletenessV1, ArtifactError> {
        use crate::impls::ConformanceCompleteness as Native;
        use phalcom_compiler_artifact::semantic::world as wire;
        Ok(match state {
            Native::Complete => wire::CallableConformanceCompletenessV1::Complete,
            Native::Incomplete { failures } => wire::CallableConformanceCompletenessV1::Incomplete {
                failures: failures
                    .iter()
                    .map(|failure| -> Result<wire::CallableConformanceFailureV1, ArtifactError> {
                        Ok(match failure {
                            crate::impls::ConformanceFailure::Behavioral(failure) => wire::CallableConformanceFailureV1::Behavioral {
                                requirement: export_callable_requirement_identity(&failure.requirement),
                                reason: failure.reason.clone(),
                            },
                            crate::impls::ConformanceFailure::AssociatedType(failure) => wire::CallableConformanceFailureV1::AssociatedType {
                                failure: wire::CallableAssociatedTypeBindingFailureV1 {
                                    kind: match failure.kind {
                                        crate::impls::AssociatedTypeBindingFailureKind::Missing => wire::CallableAssociatedTypeBindingFailureKindV1::Missing,
                                        crate::impls::AssociatedTypeBindingFailureKind::Duplicate => {
                                            wire::CallableAssociatedTypeBindingFailureKindV1::Duplicate
                                        }
                                        crate::impls::AssociatedTypeBindingFailureKind::Unknown => wire::CallableAssociatedTypeBindingFailureKindV1::Unknown,
                                        crate::impls::AssociatedTypeBindingFailureKind::Invalid => wire::CallableAssociatedTypeBindingFailureKindV1::Invalid,
                                    },
                                    name: failure.name.clone(),
                                    reason: failure.reason.clone(),
                                    source: export_source_span(&failure.source)?,
                                    requirement: failure.requirement.as_ref().map(export_callable_associated_type_requirement_identity),
                                },
                            },
                        })
                    })
                    .collect::<Result<Vec<_>, ArtifactError>>()?,
            },
            Native::Unknown(reason) => wire::CallableConformanceCompletenessV1::Unknown {
                reason: export_unknown_reason(reason),
            },
            Native::Blocked(reason) => wire::CallableConformanceCompletenessV1::Blocked {
                reason: export_callable_block_reason(reason, self.types)?,
            },
            Native::Dynamic(obligation) => wire::CallableConformanceCompletenessV1::Dynamic {
                reason: obligation.reason.clone().into_boxed_str(),
            },
            Native::Cancelled => wire::CallableConformanceCompletenessV1::Cancelled,
            Native::BudgetExceeded(report) => wire::CallableConformanceCompletenessV1::BudgetExceeded {
                report: export_binding_budget(report),
            },
            Native::InternalFailure(message) => wire::CallableConformanceCompletenessV1::InternalFailure { message: message.clone() },
        })
    }
}

fn export_binding_consistency(
    consistency: &crate::checker::binding::BindingConsistency,
    types: &TypeExportContext,
) -> Result<CallableBindingConsistencyV1, ArtifactError> {
    use crate::checker::binding::{AssumptionBasis, BindingConsistency};
    Ok(match consistency {
        BindingConsistency::Unconstrained => CallableBindingConsistencyV1::Unconstrained,
        BindingConsistency::Validated => CallableBindingConsistencyV1::Validated,
        BindingConsistency::Assumed { basis } => CallableBindingConsistencyV1::Assumed {
            basis: match basis {
                AssumptionBasis::MissingValueEvidence(reason) => CallableBindingAssumptionBasisV1::MissingValueEvidence {
                    reason: export_unknown_reason(reason),
                },
                AssumptionBasis::CallableParameterContract => CallableBindingAssumptionBasisV1::CallableParameterContract,
                AssumptionBasis::ContextualParameterContract => CallableBindingAssumptionBasisV1::ContextualParameterContract,
                AssumptionBasis::DerivedEvidence(origin) => CallableBindingAssumptionBasisV1::DerivedEvidence {
                    origin: match origin {
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
                },
            },
        },
        BindingConsistency::Refuted { actual, expected, reason } => CallableBindingConsistencyV1::Refuted {
            actual: types.ty(*actual)?,
            expected: types.ty(*expected)?,
            reason: match reason {
                crate::types::relation::RefutationReason::IncompatibleNominal => CallableRefutationReasonV1::IncompatibleNominal,
                crate::types::relation::RefutationReason::TypeMismatch => CallableRefutationReasonV1::TypeMismatch,
                crate::types::relation::RefutationReason::UnionMemberMismatch => CallableRefutationReasonV1::UnionMemberMismatch,
            },
        },
        BindingConsistency::DynamicBoundary { obligation } => CallableBindingConsistencyV1::DynamicBoundary {
            reason: obligation.reason.clone().into_boxed_str(),
        },
        BindingConsistency::Blocked(reason) => CallableBindingConsistencyV1::Blocked {
            reason: export_callable_block_reason(reason, types)?,
        },
        BindingConsistency::Cancelled => CallableBindingConsistencyV1::Cancelled,
        BindingConsistency::BudgetExceeded(report) => CallableBindingConsistencyV1::BudgetExceeded {
            report: export_binding_budget(report),
        },
        BindingConsistency::InternalFailure(message) => CallableBindingConsistencyV1::InternalFailure { message: message.clone() },
    })
}

fn export_callable_binding_contract(
    contract: &crate::checker::binding::BindingContract,
    types: &TypeExportContext,
) -> Result<phalcom_compiler_artifact::semantic::world::CallableBindingContractV1, ArtifactError> {
    use crate::checker::binding::BindingContractOrigin;
    use phalcom_compiler_artifact::semantic::world::CallableBindingContractOriginV1 as PortableOrigin;
    Ok(phalcom_compiler_artifact::semantic::world::CallableBindingContractV1 {
        ty: types.ty(contract.ty)?,
        origin: match contract.origin {
            BindingContractOrigin::SourceAnnotation => PortableOrigin::SourceAnnotation,
            BindingContractOrigin::InferredInitializer => PortableOrigin::InferredInitializer,
            BindingContractOrigin::CallableParameter => PortableOrigin::CallableParameter,
            BindingContractOrigin::ContextualBlockParameter => PortableOrigin::ContextualBlockParameter,
            BindingContractOrigin::PatternBinding => PortableOrigin::PatternBinding,
            BindingContractOrigin::StaticParameter => PortableOrigin::StaticParameter,
        },
        source: contract.source.map(export_source_range).transpose()?,
    })
}

fn export_callable_causal_invalidity(
    invalidity: crate::checker::causal::CausalInvalidity,
    ordinals: &CallableAnalysisOrdinalMap,
) -> Result<CallableCausalInvalidityV1, ArtifactError> {
    Ok(match invalidity {
        crate::checker::causal::CausalInvalidity::Clean => CallableCausalInvalidityV1::Clean,
        crate::checker::causal::CausalInvalidity::One(cause) => CallableCausalInvalidityV1::One {
            cause: *ordinals
                .diagnostic_causes
                .get(&cause)
                .ok_or_else(|| referential("callable fact references an absent diagnostic cause"))?,
        },
        crate::checker::causal::CausalInvalidity::Multiple => CallableCausalInvalidityV1::Multiple,
    })
}

fn export_callable_flow_state(
    flow: &crate::checker::analysis::FlowStateSummary,
    ordinals: &CallableAnalysisOrdinalMap,
    types: &TypeExportContext,
) -> Result<phalcom_compiler_artifact::semantic::world::CallableFlowStateSummaryV1, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world as wire;
    let mut bindings = Vec::with_capacity(flow.bindings.len());
    for (binding, summary) in &flow.bindings {
        bindings.push(wire::CallableFlowBindingSummaryV1 {
            binding: *ordinals
                .bindings
                .get(binding)
                .ok_or_else(|| referential("callable flow summary references an absent binding"))?,
            knowledge: export_type_knowledge(&summary.knowledge, types)?,
            contract: summary
                .contract
                .as_ref()
                .map(|contract| export_callable_binding_contract(contract, types))
                .transpose()?,
            consistency: export_binding_consistency(&summary.consistency, types)?,
            mutable: summary.mutable,
            initialization: match summary.initialization {
                crate::checker::flow::InitializationState::Uninitialized => wire::CallableInitializationStateV1::Uninitialized,
                crate::checker::flow::InitializationState::MaybeInitialized => wire::CallableInitializationStateV1::MaybeInitialized,
                crate::checker::flow::InitializationState::DefinitelyInitialized => wire::CallableInitializationStateV1::DefinitelyInitialized,
            },
        });
    }
    bindings.sort_unstable_by_key(|binding| binding.binding);
    let mut fields = Vec::with_capacity(flow.fields.len());
    for (field, summary) in &flow.fields {
        fields.push(wire::CallableFlowFieldSummaryV1 {
            field: wire::StaticFieldDependencyV1 {
                owner: to_stable_declaration(&field.owner),
                side: to_stable_dispatch_side(field.side),
                name: field.name.clone(),
            },
            contract: export_type_knowledge(&summary.contract, types)?,
            current: export_type_knowledge(&summary.current, types)?,
            initialization: match summary.initialization {
                crate::checker::flow::InitializationState::Uninitialized => wire::CallableInitializationStateV1::Uninitialized,
                crate::checker::flow::InitializationState::MaybeInitialized => wire::CallableInitializationStateV1::MaybeInitialized,
                crate::checker::flow::InitializationState::DefinitelyInitialized => wire::CallableInitializationStateV1::DefinitelyInitialized,
            },
            validity: match &summary.validity {
                crate::checker::flow::FieldContractValidity::Unchecked => wire::CallableFieldContractValidityV1::Unchecked,
                crate::checker::flow::FieldContractValidity::Validated => wire::CallableFieldContractValidityV1::Validated,
                crate::checker::flow::FieldContractValidity::Assumed => wire::CallableFieldContractValidityV1::Assumed,
                crate::checker::flow::FieldContractValidity::Refuted => wire::CallableFieldContractValidityV1::Refuted,
                crate::checker::flow::FieldContractValidity::Blocked(reason) => wire::CallableFieldContractValidityV1::Blocked {
                    reason: export_callable_block_reason(reason, types)?,
                },
                crate::checker::flow::FieldContractValidity::DynamicBoundary(obligation) => wire::CallableFieldContractValidityV1::DynamicBoundary {
                    reason: obligation.reason.clone().into_boxed_str(),
                },
            },
            causal_invalidity: export_callable_causal_invalidity(summary.causal_invalidity, ordinals)?,
        });
    }
    fields.sort_unstable_by(|left, right| {
        (&left.field.owner, left.field.side, &left.field.name).cmp(&(&right.field.owner, right.field.side, &right.field.name))
    });
    Ok(wire::CallableFlowStateSummaryV1 {
        bindings,
        fields,
        fact_count: u64::try_from(flow.fact_count).map_err(|_| referential("callable flow fact count exceeds portable range"))?,
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
        for (id, node) in analysis.explanations.iter() {
            if id != &node.id {
                return Err(referential("callable explanation map key disagrees with its node identity"));
            }
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
    binding_source_authorities: &std::collections::BTreeMap<crate::identity::BindingId, CallableBindingSourceAuthority>,
    types: &TypeExportContext,
    limits: &ValidationLimits,
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
    if binding_source_authorities.keys().any(|binding| !ordinals.bindings.contains_key(binding)) {
        return Err(referential("callable source authority references an absent owner-local binding"));
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
    let mut conformance_export = CallableConformanceEvidenceExport {
        types,
        ordinals: Some(&ordinals),
        portable_explanations: None,
        portable_causes: None,
        max_nodes: limits.max_semantic_entries_per_family,
        max_depth: limits.max_recursion_depth,
        active_arc_addresses: std::collections::HashSet::new(),
        nodes: Vec::new(),
    };
    let mut expressions = Vec::with_capacity(analysis.expressions.len());
    let mut ordered_expressions = analysis.expressions.iter().collect::<Vec<_>>();
    ordered_expressions.sort_unstable_by_key(|(expression_id, _)| ordinals.expressions[expression_id]);
    for (expression_id, expression) in ordered_expressions {
        if expression.call.is_some() {
            return Err(referential("call resolution ids have no admitted portable callable owner"));
        }
        let trait_dispatch_candidates = expression
            .trait_dispatch_candidates
            .as_ref()
            .map(|candidates| {
                candidates
                    .iter()
                    .map(|candidate| export_callable_trait_dispatch_candidate(candidate, &mut conformance_export, 0))
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?;
        let contextual_realization = expression
            .contextual_realization
            .as_deref()
            .map(|realization| export_callable_contextual_realization(realization, &ordinals, &mut conformance_export))
            .transpose()?
            .map(Box::new);
        let status = export_callable_expression_status(&expression.status, &ordinals, types)?;
        let range = phalcom_compiler_artifact::module_world::SpanV1 {
            start: u64::try_from(expression.range.start).map_err(|_| referential("callable expression range start exceeds portable range"))?,
            end: u64::try_from(expression.range.end).map_err(|_| referential("callable expression range end exceeds portable range"))?,
        };
        let knowledge = export_type_knowledge(&expression.knowledge, types)?;
        let causal_invalidity = match expression.causal_invalidity {
            crate::checker::causal::CausalInvalidity::Clean => CallableCausalInvalidityV1::Clean,
            crate::checker::causal::CausalInvalidity::One(cause) => CallableCausalInvalidityV1::One {
                cause: *ordinals
                    .diagnostic_causes
                    .get(&cause)
                    .ok_or_else(|| referential("callable expression causal state references an absent diagnostic cause"))?,
            },
            crate::checker::causal::CausalInvalidity::Multiple => CallableCausalInvalidityV1::Multiple,
        };
        let binding_read_safety = expression.binding_read_safety.map(|safety| match safety {
            crate::checker::flow::BindingReadSafety::ProvenInitialized => {
                phalcom_compiler_artifact::semantic::world::CallableBindingReadSafetyV1::ProvenInitialized
            }
            crate::checker::flow::BindingReadSafety::DefinitelyUninitialized => {
                phalcom_compiler_artifact::semantic::world::CallableBindingReadSafetyV1::DefinitelyUninitialized
            }
            crate::checker::flow::BindingReadSafety::RuntimeCheckRequired => {
                phalcom_compiler_artifact::semantic::world::CallableBindingReadSafetyV1::RuntimeCheckRequired
            }
        });
        let binding_read = expression
            .binding_read
            .map(|binding| {
                ordinals
                    .bindings
                    .get(&binding)
                    .copied()
                    .ok_or_else(|| referential("callable expression reads an absent owner-local binding"))
            })
            .transpose()?;
        let binding_write = expression
            .binding_write
            .map(|binding| {
                ordinals
                    .bindings
                    .get(&binding)
                    .copied()
                    .ok_or_else(|| referential("callable expression writes an absent owner-local binding"))
            })
            .transpose()?;
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
                let conformances = specialization
                    .conformances
                    .iter()
                    .map(|binding| conformance_export.binding(binding, 0))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(CallableCallSpecializationV1 {
                    callable: to_stable_callable(&specialization.callable),
                    bindings,
                    conformances,
                })
            })
            .transpose()?;
        let conditional_capability = expression
            .conditional_capability
            .as_ref()
            .map(|capability| {
                if expression.conditional_dispatch.as_ref() != Some(&capability.selection) {
                    return Err(referential("conditional call capability disagrees with its expression dispatch selection"));
                }
                conformance_export.conditional_capability(capability, 0)
            })
            .transpose()?;
        let trait_dispatch = expression
            .trait_dispatch
            .as_ref()
            .map(|site| conformance_export.trait_dispatch_site(site, 0))
            .transpose()?
            .map(Box::new);
        expressions.push(CallableExpressionRowV1 {
            ordinal: ordinals.expressions[expression_id],
            range,
            knowledge,
            callable: expression.callable.as_ref().map(to_stable_callable),
            conditional_dispatch: expression.conditional_dispatch.as_ref().map(export_conditional_dispatch_selection),
            conditional_capability,
            trait_dispatch,
            trait_dispatch_candidates,
            contextual_realization,
            status,
            causal_invalidity,
            explanation: expression
                .explanation
                .map(|explanation| {
                    ordinals
                        .explanations
                        .get(&explanation)
                        .copied()
                        .ok_or_else(|| referential("callable expression references an absent explanation"))
                })
                .transpose()?,
            binding_read_safety,
            binding_read,
            binding_write,
            denotation,
            call_specialization,
        });
    }
    expressions.sort_unstable_by_key(|expression| expression.ordinal);

    let mut bindings = Vec::with_capacity(analysis.bindings.len());
    for (binding_id, state) in analysis.bindings.iter() {
        let origin = match analysis.binding_origins.get(binding_id) {
            Some(origin) => origin,
            None => return Err(referential("callable binding is missing its storage origin")),
        };
        let denotation = state
            .denotation
            .as_ref()
            .map(|denotation| export_semantic_denotation(denotation, types))
            .transpose()?;
        let range = phalcom_compiler_artifact::module_world::SpanV1 {
            start: u64::try_from(state.range.start).map_err(|_| referential("callable binding range start exceeds portable range"))?,
            end: u64::try_from(state.range.end).map_err(|_| referential("callable binding range end exceeds portable range"))?,
        };
        let contract = state
            .contract
            .as_ref()
            .map(|contract| -> Result<_, ArtifactError> {
                let source = contract
                    .source
                    .map(|source| -> Result<_, ArtifactError> {
                        Ok(phalcom_compiler_artifact::module_world::SpanV1 {
                            start: u64::try_from(source.start).map_err(|_| referential("callable binding contract source start exceeds portable range"))?,
                            end: u64::try_from(source.end).map_err(|_| referential("callable binding contract source end exceeds portable range"))?,
                        })
                    })
                    .transpose()?;
                Ok(phalcom_compiler_artifact::semantic::world::CallableBindingContractV1 {
                    ty: types.ty(contract.ty)?,
                    origin: match contract.origin {
                        crate::checker::binding::BindingContractOrigin::SourceAnnotation => CallableBindingContractOriginV1::SourceAnnotation,
                        crate::checker::binding::BindingContractOrigin::InferredInitializer => CallableBindingContractOriginV1::InferredInitializer,
                        crate::checker::binding::BindingContractOrigin::CallableParameter => CallableBindingContractOriginV1::CallableParameter,
                        crate::checker::binding::BindingContractOrigin::ContextualBlockParameter => CallableBindingContractOriginV1::ContextualBlockParameter,
                        crate::checker::binding::BindingContractOrigin::PatternBinding => CallableBindingContractOriginV1::PatternBinding,
                        crate::checker::binding::BindingContractOrigin::StaticParameter => CallableBindingContractOriginV1::StaticParameter,
                    },
                    source,
                })
            })
            .transpose()?;
        let causal_invalidity = match state.causal_invalidity {
            crate::checker::causal::CausalInvalidity::Clean => CallableCausalInvalidityV1::Clean,
            crate::checker::causal::CausalInvalidity::One(cause) => CallableCausalInvalidityV1::One {
                cause: *ordinals
                    .diagnostic_causes
                    .get(&cause)
                    .ok_or_else(|| referential("callable binding causal state references an absent diagnostic cause"))?,
            },
            crate::checker::causal::CausalInvalidity::Multiple => CallableCausalInvalidityV1::Multiple,
        };
        let authority = export_binding_authority(origin, binding_source_authorities.get(binding_id))?;
        bindings.push(CallableBindingRowV1 {
            ordinal: ordinals.bindings[binding_id],
            name: state.name.clone(),
            parameter: state.parameter.as_ref().map(|parameter| CallableParameterIdentityV1 {
                callable: to_stable_callable(&parameter.callable),
                index: parameter.index,
            }),
            range,
            contract,
            current: export_type_knowledge(&state.current, types)?,
            denotation,
            consistency: export_binding_consistency(&state.consistency, types)?,
            causal_invalidity,
            mutable: state.mutable,
            initialization: match state.initialization {
                crate::checker::flow::InitializationState::Uninitialized => CallableInitializationStateV1::Uninitialized,
                crate::checker::flow::InitializationState::MaybeInitialized => CallableInitializationStateV1::MaybeInitialized,
                crate::checker::flow::InitializationState::DefinitelyInitialized => CallableInitializationStateV1::DefinitelyInitialized,
            },
            version: state.version,
            explanation: state
                .explanation
                .map(|explanation| {
                    ordinals
                        .explanations
                        .get(&explanation)
                        .copied()
                        .ok_or_else(|| referential("callable binding references an absent explanation"))
                })
                .transpose()?,
            authority,
        });
    }
    bindings.sort_unstable_by_key(|binding| binding.ordinal);

    use phalcom_compiler_artifact::semantic::world as wire;
    let mut for_iteration_lanes = Vec::with_capacity(analysis.for_iteration_lanes.len());
    for (expression, lane) in analysis.for_iteration_lanes.iter() {
        if expression != &lane.source_expression {
            return Err(referential("for-iteration lane key disagrees with its source expression"));
        }
        let expression = *ordinals
            .expressions
            .get(expression)
            .ok_or_else(|| referential("for-iteration lane references an absent owner-local expression"))?;
        for_iteration_lanes.push(wire::CallableForIterationLaneV1 {
            expression,
            source_range: export_source_range(lane.source_range)?,
            source_type: types.ty(lane.source_type)?,
            item_type: types.ty(lane.item_type)?,
            iterator_type: types.ty(lane.iterator_type)?,
            iterator: conformance_export.for_iteration_requirement(&lane.iterator, 0)?,
            next: conformance_export.for_iteration_requirement(&lane.next, 0)?,
        });
    }
    for_iteration_lanes.sort_unstable_by_key(|lane| lane.expression);

    let mut positional_expansions = Vec::with_capacity(analysis.positional_expansions.len());
    for (expression, expansion) in analysis.positional_expansions.iter() {
        if expression != &expansion.source_expression {
            return Err(referential("positional expansion key disagrees with its source expression"));
        }
        let expression = *ordinals
            .expressions
            .get(expression)
            .ok_or_else(|| referential("positional expansion references an absent owner-local expression"))?;
        positional_expansions.push(wire::CallablePositionalExpansionV1 {
            expression,
            source_range: export_source_range(expansion.source_range)?,
            source_type: types.ty(expansion.source_type)?,
            item_type: types.ty(expansion.item_type)?,
            iterator_type: types.ty(expansion.iterator_type)?,
            iterator: conformance_export.for_iteration_requirement(&expansion.iterator, 0)?,
            next: conformance_export.for_iteration_requirement(&expansion.next, 0)?,
        });
    }
    positional_expansions.sort_unstable_by_key(|expansion| expansion.expression);
    let mut known_invalid_positional_expansions = Vec::with_capacity(analysis.known_invalid_positional_expansions.len());
    for (expression, expansion) in analysis.known_invalid_positional_expansions.iter() {
        if expression != &expansion.source_expression {
            return Err(referential("known-invalid positional expansion key disagrees with its source expression"));
        }
        let expression = *ordinals
            .expressions
            .get(expression)
            .ok_or_else(|| referential("known-invalid positional expansion references an absent owner-local expression"))?;
        known_invalid_positional_expansions.push(wire::CallableKnownInvalidPositionalExpansionV1 {
            expression,
            source_range: export_source_range(expansion.source_range)?,
            source_type: types.ty(expansion.source_type)?,
        });
    }
    known_invalid_positional_expansions.sort_unstable_by_key(|expansion| expansion.expression);

    let mut sequence_literals = Vec::with_capacity(analysis.sequence_literals.len());
    for (expression, literal) in analysis.sequence_literals.iter() {
        if expression != &literal.source_expression {
            return Err(referential("sequence literal key disagrees with its source expression"));
        }
        let expression = *ordinals
            .expressions
            .get(expression)
            .ok_or_else(|| referential("sequence literal references an absent owner-local expression"))?;
        let construction = match literal.construction {
            crate::checker::analysis::SequenceConstructionPlan::Fixed { element_count } => wire::CallableSequenceConstructionV1::Fixed {
                element_count: u64::try_from(element_count).map_err(|_| referential("sequence element count exceeds portable range"))?,
            },
            crate::checker::analysis::SequenceConstructionPlan::Expanded => wire::CallableSequenceConstructionV1::Expanded,
            crate::checker::analysis::SequenceConstructionPlan::VectorIdentitySpread { source_range, extent } => {
                wire::CallableSequenceConstructionV1::VectorIdentitySpread {
                    source_range: export_source_range(source_range)?,
                    extent: types.ty(extent)?,
                }
            }
        };
        let exact_expansions = literal
            .exact_expansions
            .iter()
            .map(|(range, expansion)| {
                Ok(wire::CallableSequenceExpansionEntryV1 {
                    source_range: export_source_range(*range)?,
                    expansion: match expansion {
                        crate::checker::analysis::SequenceExpansionResolution::TuplePositionals { extent } => {
                            wire::CallableSequenceExpansionV1::TuplePositionals { extent: types.ty(*extent)? }
                        }
                        crate::checker::analysis::SequenceExpansionResolution::VectorStorage { extent } => {
                            wire::CallableSequenceExpansionV1::VectorStorage { extent: types.ty(*extent)? }
                        }
                    },
                })
            })
            .collect::<Result<Vec<_>, ArtifactError>>()?;
        sequence_literals.push(wire::CallableSequenceLiteralV1 {
            expression,
            source_range: export_source_range(literal.source_range)?,
            family: match literal.family {
                crate::checker::analysis::SequenceLiteralFamily::List => wire::CallableSequenceFamilyV1::List,
                crate::checker::analysis::SequenceLiteralFamily::Array => wire::CallableSequenceFamilyV1::Array,
                crate::checker::analysis::SequenceLiteralFamily::Vector => wire::CallableSequenceFamilyV1::Vector,
            },
            result_type: types.ty(literal.result_type)?,
            element_type: types.ty(literal.element_type)?,
            extent: literal.extent.map(|extent| types.ty(extent)).transpose()?,
            construction,
            exact_expansions,
        });
    }
    sequence_literals.sort_unstable_by_key(|literal| literal.expression);

    let mut associated_resolutions = Vec::with_capacity(analysis.associated_resolutions.len());
    for (expression, resolution) in analysis.associated_resolutions.iter() {
        let expression = *ordinals
            .expressions
            .get(expression)
            .ok_or_else(|| referential("associated resolution references an absent owner-local expression"))?;
        let family = resolution.family.as_ref().map(|family| wire::AssociatedFamilyIdentityV1 {
            owner: to_stable_declaration(&family.owner),
            selector: export_associated_selector(&family.base),
        });
        let kind = match &resolution.kind {
            crate::checker::associated::AssociatedResolutionKind::ExactValue { member, value_type } => {
                wire::CallableAssociatedResolutionKindV1::ExactValue {
                    member: export_associated_member(member),
                    value_type: types.ty(*value_type)?,
                }
            }
            crate::checker::associated::AssociatedResolutionKind::StaticInvoke {
                member,
                target,
                result_type,
                argument_mapping,
            } => wire::CallableAssociatedResolutionKindV1::StaticInvoke {
                member: export_associated_member(member),
                target: export_invocation_target(target),
                result_type: types.ty(*result_type)?,
                argument_mapping: argument_mapping.as_ref().map(|mapping| mapping.to_vec()),
            },
            _ => return Err(referential("associated resolution variant is outside the admitted portable profile")),
        };
        associated_resolutions.push(wire::CallableAssociatedResolutionV1 {
            expression,
            owner_form: types.ty(resolution.owner_form)?,
            lookup_owner: to_stable_declaration(&resolution.lookup_owner),
            family,
            kind,
        });
    }
    associated_resolutions.sort_unstable_by_key(|resolution| resolution.expression);

    let mut callable_reference_resolutions = Vec::with_capacity(analysis.callable_reference_resolutions.len());
    for (expression, resolution) in analysis.callable_reference_resolutions.iter() {
        let expression = *ordinals
            .expressions
            .get(expression)
            .ok_or_else(|| referential("callable reference resolution references an absent owner-local expression"))?;
        let crate::checker::associated::CallableReferenceResolutionKind::BoundFamily {
            family_type,
            spec,
            members,
        } = &resolution.kind
        else {
            return Err(referential("associated callable-reference resolution is outside the admitted portable profile"));
        };
        let spec = match spec {
            crate::checker::associated::BehavioralFamilySpec::Exact(selector) => wire::BehavioralFamilySpecV1::Exact {
                selector: export_exact_selector(selector),
            },
            crate::checker::associated::BehavioralFamilySpec::Pattern(pattern) => wire::BehavioralFamilySpecV1::Pattern {
                pattern: export_selector_pattern(pattern),
            },
        };
        let mut portable_members = Vec::with_capacity(members.len());
        for member in members.iter() {
            let conditional_capability = member
                .conditional_capability
                .as_ref()
                .map(|capability| {
                    if member.conditional.as_ref() != Some(&capability.selection) {
                        return Err(referential("bound behavioral capability disagrees with its conditional selection"));
                    }
                    conformance_export.conditional_capability(capability, 0)
                })
                .transpose()?;
            portable_members.push(wire::CallableBoundBehavioralMemberV1 {
                operation: export_family_operation_shape(&member.operation),
                member_kind: match member.member_kind {
                    crate::types::family::FamilyMemberTypeKind::Value => wire::CallableFamilyMemberKindV1::Value,
                    crate::types::family::FamilyMemberTypeKind::Callable => wire::CallableFamilyMemberKindV1::Callable,
                },
                target: export_invocation_target(&member.target),
                callable_type: types.ty(member.callable_type)?,
                conditional: member.conditional.as_ref().map(export_conditional_dispatch_selection),
                conditional_capability,
                trait_dispatch: member
                    .trait_dispatch
                    .as_ref()
                    .map(|selection| conformance_export.trait_dispatch_selection(selection, 0))
                    .transpose()?,
            });
        }
        callable_reference_resolutions.push(wire::CallableReferenceResolutionV1 {
            expression,
            family_type: types.ty(*family_type)?,
            spec,
            members: portable_members,
        });
    }
    callable_reference_resolutions.sort_unstable_by_key(|resolution| resolution.expression);

    let mut family_applications = Vec::with_capacity(analysis.family_applications.len());
    for (expression, application) in analysis.family_applications.iter() {
        use crate::checker::associated::{FamilyApplicationKind, FamilyApplicationSelection};
        let expression = *ordinals
            .expressions
            .get(expression)
            .ok_or_else(|| referential("family application references an absent owner-local expression"))?;
        let selection = match &application.selection {
            FamilyApplicationSelection::Static {
                operation,
                target,
                callable_type,
                result_type,
            } => wire::CallableFamilyApplicationSelectionV1::Static {
                operation: export_family_operation_shape(operation),
                target: target.as_ref().map(export_invocation_target),
                callable_type: types.ty(*callable_type)?,
                result_type: types.ty(*result_type)?,
            },
            FamilyApplicationSelection::Dynamic { candidates, result_type } => wire::CallableFamilyApplicationSelectionV1::Dynamic {
                candidates: candidates
                    .iter()
                    .map(|candidate| {
                        Ok(wire::CallableFamilyApplicationCandidateV1 {
                            operation: export_family_operation_shape(&candidate.operation),
                            target: candidate.target.as_ref().map(export_invocation_target),
                            callable_type: types.ty(candidate.callable_type)?,
                        })
                    })
                    .collect::<Result<Vec<_>, ArtifactError>>()?,
                result_type: result_type.map(|ty| types.ty(ty)).transpose()?,
            },
        };
        family_applications.push(wire::CallableFamilyApplicationV1 {
            expression,
            family_type: types.ty(application.family_type)?,
            kind: match application.kind {
                FamilyApplicationKind::Associated => wire::CallableFamilyApplicationKindV1::Associated,
                FamilyApplicationKind::Behavioral => wire::CallableFamilyApplicationKindV1::Behavioral,
            },
            selection,
        });
    }
    family_applications.sort_unstable_by_key(|application| application.expression);

    let entry_flow = export_callable_flow_state(&analysis.entry_flow, &ordinals, types)?;
    let normal_returns = analysis
        .exits
        .normal_returns
        .iter()
        .map(|exit| {
            Ok(wire::CallableNormalReturnV1 {
                knowledge: export_type_knowledge(&exit.knowledge, types)?,
                flow: export_callable_flow_state(&exit.flow, &ordinals, types)?,
                status: export_callable_expression_status(&exit.status, &ordinals, types)?,
                causal_invalidity: export_callable_causal_invalidity(exit.causal_invalidity, &ordinals)?,
                direct_result_dependency: exit.direct_result_dependency.as_ref().map(to_stable_callable),
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    let throws = analysis
        .exits
        .throws
        .iter()
        .map(|flow| export_callable_flow_state(flow, &ordinals, types))
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    let return_validation = export_return_validation(analysis.return_validation);

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
    let (local_proof_graph, match_resolutions, pattern_site_resolutions) = proof::export_callable_proofs(analysis, &ordinals, types)?;
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
        conformance_evidence: conformance_export.nodes,
        bindings,
        for_iteration_lanes,
        positional_expansions,
        known_invalid_positional_expansions,
        sequence_literals,
        associated_resolutions,
        callable_reference_resolutions,
        family_applications,
        local_proof_graph,
        match_resolutions,
        pattern_site_resolutions,
        entry_flow,
        normal_returns,
        throws,
        return_validation,
        explanations: explanation::export_explanations(analysis, &ordinals, types)?,
        return_explanation: analysis
            .return_explanation
            .map(|id| {
                ordinals
                    .explanations
                    .get(&id)
                    .copied()
                    .ok_or_else(|| referential("return explanation is absent"))
            })
            .transpose()?,
        diagnostic_cause_count: u32::try_from(ordinals.diagnostic_causes.len()).map_err(|_| referential("too many diagnostic causes"))?,
        diagnostics: analysis
            .diagnostics
            .iter()
            .map(|diagnostic| export_semantic_diagnostic(diagnostic, Some(&ordinals), None, None, types))
            .collect::<Result<_, _>>()?,
        dependencies: analysis.dependencies.iter().map(to_stable_callable).collect(),
        semantic_dependencies: analysis
            .semantic_dependencies
            .iter()
            .map(|dependency| export_static_semantic_dependency(dependency, types))
            .collect::<Result<_, _>>()?,
    })
}

fn export_callable_trait_dispatch_candidate(
    candidate: &crate::trait_dispatch::TraitEvidencedMemberCandidate,
    conformance_export: &mut CallableConformanceEvidenceExport<'_>,
    depth: usize,
) -> Result<phalcom_compiler_artifact::semantic::world::CallableTraitDispatchCandidateV1, ArtifactError> {
    use crate::trait_dispatch::TraitDispatchConvergenceKey as Native;
    use phalcom_compiler_artifact::semantic::world as wire;

    let expected_fingerprint = crate::db::fingerprint::generic_conformance_evidence_fingerprint(&candidate.evidence);
    if candidate.evidence_fingerprint != expected_fingerprint {
        return Err(referential("trait dispatch candidate carries a stale evidence fingerprint"));
    }
    let mut requirement_targets = Vec::with_capacity(candidate.requirement_targets.len());
    for (requirement, target) in candidate.requirement_targets.iter() {
        requirement_targets.push(wire::CallableRequirementSelectionEntryV1 {
            requirement: export_callable_requirement_identity(requirement),
            selection: conformance_export.selection(target, depth + 1)?,
        });
    }
    let convergence = match &candidate.convergence {
        Native::Inherent(callable) => wire::CallableTraitDispatchConvergenceV1::Inherent {
            callable: to_stable_callable(callable),
        },
        Native::ConditionalInherent {
            callable,
            impl_id,
            applicability_fingerprint,
        } => wire::CallableTraitDispatchConvergenceV1::ConditionalInherent {
            callable: to_stable_callable(callable),
            implementation: to_stable_impl(impl_id),
            applicability_fingerprint: applicability_fingerprint.raw(),
        },
        Native::DataComponent(component) => wire::CallableTraitDispatchConvergenceV1::DataComponent {
            owner: to_stable_declaration(&component.owner),
            index: component.index,
        },
        Native::ConformanceCallable(callable) => wire::CallableTraitDispatchConvergenceV1::ConformanceCallable {
            callable: to_stable_callable(callable),
        },
        Native::TraitDefault {
            callable,
            evidence_fingerprint,
        } => wire::CallableTraitDispatchConvergenceV1::TraitDefault {
            callable: to_stable_callable(callable),
            evidence_fingerprint: evidence_fingerprint.raw(),
        },
    };
    Ok(wire::CallableTraitDispatchCandidateV1 {
        source_impl: to_stable_impl(&candidate.source_impl),
        exact_target: conformance_export.types.ty(candidate.exact_target)?,
        exact_trait_ref: export_trait_ref_identity(&candidate.exact_trait_ref, conformance_export.types)?,
        requirement: export_callable_requirement_identity(&candidate.requirement),
        signature: export_callable_signature(&candidate.signature, conformance_export.types)?,
        evidence: conformance_export.evidence(&candidate.evidence, depth + 1)?,
        selection: conformance_export.selection(&candidate.selection, depth + 1)?,
        requirement_targets,
        convergence,
    })
}

fn export_callable_contextual_realization(
    realization: &crate::checker::analysis::ContextualRealization,
    ordinals: &CallableAnalysisOrdinalMap,
    conformance_export: &mut CallableConformanceEvidenceExport<'_>,
) -> Result<phalcom_compiler_artifact::semantic::world::CallableContextualRealizationV1, ArtifactError> {
    use crate::checker::analysis::{
        AlgebraicBasisAxis, ContextualRealizationOperation, RealizationCapabilitySelection, RealizationCoefficientSource, RealizationEvaluationPlan,
    };
    use phalcom_compiler_artifact::semantic::world as wire;

    let expression = |id| {
        ordinals
            .expressions
            .get(&id)
            .copied()
            .ok_or_else(|| referential("contextual realization references an absent owner-local expression"))
    };
    let capability = |selection: &RealizationCapabilitySelection,
                      conformance_export: &mut CallableConformanceEvidenceExport<'_>|
     -> Result<wire::CallableRealizationCapabilityV1, ArtifactError> {
        Ok(match selection {
            RealizationCapabilitySelection::Evidenced(selection) => wire::CallableRealizationCapabilityV1::Evidenced {
                selection: conformance_export.trait_dispatch_selection(selection, 0)?,
            },
            RealizationCapabilitySelection::Assumed(selection) => wire::CallableRealizationCapabilityV1::Assumed {
                conformance: export_generic_conformance_key(&selection.conformance, conformance_export.types)?,
                requirement: export_callable_requirement_identity(&selection.requirement),
                signature: export_callable_signature(&selection.signature, conformance_export.types)?,
            },
        })
    };
    let axis = |axis| match axis {
        AlgebraicBasisAxis::I => wire::CallableAlgebraicBasisAxisV1::I,
        AlgebraicBasisAxis::J => wire::CallableAlgebraicBasisAxisV1::J,
        AlgebraicBasisAxis::K => wire::CallableAlgebraicBasisAxisV1::K,
    };
    let operation = match &realization.operation {
        ContextualRealizationOperation::Basis {
            basis_type,
            axis: basis_axis,
            coefficient,
            evaluation,
        } => {
            let coefficient = match coefficient {
                RealizationCoefficientSource::One { scalar_type, one } => wire::CallableRealizationCoefficientV1::One {
                    scalar_type: conformance_export.types.ty(*scalar_type)?,
                    capability: capability(one, conformance_export)?,
                },
                RealizationCoefficientSource::IntegerLiteral { digits, radix } => wire::CallableRealizationCoefficientV1::IntegerLiteral {
                    digits: digits.clone(),
                    radix: *radix,
                },
                RealizationCoefficientSource::FloatLiteral { bits } => wire::CallableRealizationCoefficientV1::FloatLiteral { bits: *bits },
                RealizationCoefficientSource::Expression {
                    expression: coefficient_expression,
                    scalar_type,
                } => wire::CallableRealizationCoefficientV1::Expression {
                    expression: expression(*coefficient_expression)?,
                    scalar_type: conformance_export.types.ty(*scalar_type)?,
                },
            };
            let evaluation = match evaluation {
                RealizationEvaluationPlan::LiteralCoefficientThenAxis => wire::CallableRealizationEvaluationV1::LiteralCoefficientThenAxis,
                RealizationEvaluationPlan::MultiplicativeIdentityThenSourceBasis => {
                    wire::CallableRealizationEvaluationV1::MultiplicativeIdentityThenSourceBasis
                }
                RealizationEvaluationPlan::SourceOperands {
                    first,
                    second,
                    coefficient,
                    basis,
                } => wire::CallableRealizationEvaluationV1::SourceOperands {
                    first: expression(*first)?,
                    second: expression(*second)?,
                    coefficient: expression(*coefficient)?,
                    basis: expression(*basis)?,
                },
            };
            wire::CallableContextualRealizationOperationV1::Basis {
                basis_type: conformance_export.types.ty(*basis_type)?,
                axis: axis(*basis_axis),
                coefficient,
                evaluation,
            }
        }
        ContextualRealizationOperation::RealEmbedding { value, scalar_type } => wire::CallableContextualRealizationOperationV1::RealEmbedding {
            value: expression(*value)?,
            scalar_type: conformance_export.types.ty(*scalar_type)?,
        },
    };
    Ok(wire::CallableContextualRealizationV1 {
        source_expression: expression(realization.source_expression)?,
        source_range: export_source_range(realization.source_range)?,
        source_type: conformance_export.types.ty(realization.source_type)?,
        target_type: conformance_export.types.ty(realization.target_type)?,
        operation,
        realization: capability(&realization.realization, conformance_export)?,
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

fn export_callable_definition(
    definition: &crate::impls::EffectiveCallableDefinition,
    store: &crate::types::store::TypeStore,
    types: &TypeExportContext,
) -> Result<CallableDefinitionRecordV1, ArtifactError> {
    use crate::impls::{CallableDefinitionBodyProvenance, CallableDefinitionOrigin};

    if definition.callable != definition.signature.callable {
        return Err(referential("accepted callable definition key disagrees with its complete signature"));
    }
    let origin = match &definition.origin {
        CallableDefinitionOrigin::PrimaryDeclaration => CallableDefinitionOriginV1::PrimaryDeclaration,
        CallableDefinitionOrigin::InherentImpl(impl_id) => CallableDefinitionOriginV1::InherentImpl {
            impl_id: to_stable_impl(impl_id),
        },
        CallableDefinitionOrigin::ConformanceWitness(impl_id) => CallableDefinitionOriginV1::ConformanceWitness {
            impl_id: to_stable_impl(impl_id),
        },
    };
    let body_provenance = match &definition.body_provenance {
        CallableDefinitionBodyProvenance::SourceBody => CallableDefinitionBodyProvenanceV1::SourceBody,
        CallableDefinitionBodyProvenance::GeneratedDelegation { target_field } => CallableDefinitionBodyProvenanceV1::GeneratedDelegation {
            target_field: target_field.clone(),
        },
    };
    let inherent_body_context = definition
        .inherent_body_context
        .as_ref()
        .map(|context| -> Result<_, ArtifactError> {
            let covering = context
                .covering
                .as_ref()
                .map(|covering| -> Result<_, ArtifactError> {
                    if covering.impl_to_decl.len() != covering.decl_to_impl.len() {
                        return Err(referential("covering impl substitution is not a bijection"));
                    }
                    let mut forward_native = covering
                        .impl_to_decl
                        .iter()
                        .map(|(&implementation, &declaration)| (implementation, declaration))
                        .collect::<Vec<_>>();
                    let mut inverse_native = covering
                        .decl_to_impl
                        .iter()
                        .map(|(&declaration, &implementation)| (implementation, declaration))
                        .collect::<Vec<_>>();
                    forward_native.sort_unstable_by_key(|(implementation, _)| *implementation);
                    inverse_native.sort_unstable_by_key(|(_, declaration)| *declaration);
                    if forward_native.windows(2).any(|pair| pair[0].0 >= pair[1].0) || inverse_native.windows(2).any(|pair| pair[0].1 >= pair[1].1) {
                        return Err(referential("covering impl substitution contains duplicate parameter identities"));
                    }
                    let expected_impl = match &definition.origin {
                        CallableDefinitionOrigin::InherentImpl(impl_id) => impl_id,
                        _ => return Err(referential("covering substitution is attached to a non-inherent definition")),
                    };
                    for &(impl_parameter, declaration_parameter) in &forward_native {
                        if covering.decl_to_impl.get(&declaration_parameter) != Some(&impl_parameter) {
                            return Err(referential("covering impl substitution maps are not exact inverses"));
                        }
                        let impl_info = store
                            .try_type_parameter(impl_parameter)
                            .ok_or_else(|| referential("covering impl substitution references an absent implementation parameter"))?;
                        let declaration_info = store
                            .try_type_parameter(declaration_parameter)
                            .ok_or_else(|| referential("covering impl substitution references an absent declaration parameter"))?;
                        if impl_info.owner != TypeParameterOwner::Impl(expected_impl.clone())
                            || declaration_info.owner != TypeParameterOwner::Declaration(definition.signature.owner.clone())
                            || impl_info.kind != declaration_info.kind
                        {
                            return Err(referential("covering impl substitution is not an owner-correct, kind-preserving bijection"));
                        }
                    }
                    let impl_to_decl = forward_native
                        .into_iter()
                        .map(|(impl_parameter, declaration_parameter)| {
                            Ok(CoveringImplParameterBindingV1 {
                                impl_parameter: types.parameter(impl_parameter)?,
                                declaration_parameter: types.parameter(declaration_parameter)?,
                            })
                        })
                        .collect::<Result<Vec<_>, ArtifactError>>()?;
                    let decl_to_impl = inverse_native
                        .into_iter()
                        .map(|(impl_parameter, declaration_parameter)| {
                            Ok(CoveringImplParameterBindingV1 {
                                impl_parameter: types.parameter(impl_parameter)?,
                                declaration_parameter: types.parameter(declaration_parameter)?,
                            })
                        })
                        .collect::<Result<Vec<_>, ArtifactError>>()?;
                    Ok(CoveringImplSubstitutionV1 { impl_to_decl, decl_to_impl })
                })
                .transpose()?;
            Ok(InherentBodyContextV1 {
                generic_signature: context
                    .generic_signature
                    .as_ref()
                    .map(|signature| export_generic_signature(signature, types))
                    .transpose()?,
                covering,
                source_target_head: types.ty(context.source_target_head)?,
            })
        })
        .transpose()?;
    let is_inherent = matches!(&definition.origin, CallableDefinitionOrigin::InherentImpl(_));
    if is_inherent != inherent_body_context.is_some()
        || (matches!(&definition.origin, CallableDefinitionOrigin::ConformanceWitness(_))
            && !matches!(&definition.body_provenance, CallableDefinitionBodyProvenance::SourceBody))
    {
        return Err(referential(
            "accepted callable definition origin, context, and body provenance are inconsistent",
        ));
    }

    Ok(CallableDefinitionRecordV1 {
        callable: to_stable_callable(&definition.callable),
        origin,
        source_member_index: u32::try_from(definition.source_member_index).map_err(|_| referential("callable source member index exceeds portable range"))?,
        signature: export_callable_signature(&definition.signature, types)?,
        body_provenance,
        inherent_body_context,
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

fn export_static_case_domain(domain: &crate::identity::StaticDomainDeclarationId) -> phalcom_compiler_artifact::semantic::world::StaticCaseDomainV1 {
    use phalcom_compiler_artifact::semantic::world::StaticCaseDomainV1;
    match domain {
        crate::identity::StaticDomainDeclarationId::Nominal(declaration) => StaticCaseDomainV1::Nominal {
            declaration: to_stable_declaration(declaration),
        },
        crate::identity::StaticDomainDeclarationId::Associated(declaration) => StaticCaseDomainV1::Associated {
            declaration: export_associated_static_domain_declaration(declaration),
        },
    }
}

fn export_static_semantic_dependency(
    dependency: &crate::checker::analysis::SemanticDependency,
    types: &TypeExportContext,
) -> Result<phalcom_compiler_artifact::semantic::world::StaticSemanticDependencyV1, ArtifactError> {
    use crate::checker::analysis::SemanticDependency as Dependency;
    use phalcom_compiler_artifact::semantic::world::{
        ConformanceTargetV1, StaticConstantSymbolV1, StaticSemanticDependencyV1 as Portable,
    };
    Ok(match dependency {
        Dependency::DeclarationShell(owner) => Portable::DeclarationShell { owner: to_stable_declaration(owner) },
        Dependency::TraitHeader(owner) => Portable::TraitHeader { owner: to_stable_declaration(owner) },
        Dependency::CallableSignature(callable) => Portable::CallableSignature { callable: to_stable_callable(callable) },
        Dependency::FieldSignature(field) => Portable::FieldSignature {
            field: phalcom_compiler_artifact::semantic::world::StaticFieldDependencyV1 {
                owner: to_stable_declaration(&field.owner),
                side: to_stable_dispatch_side(field.side),
                name: field.name.clone(),
            },
        },
        Dependency::DeclarationSurface(owner) => Portable::DeclarationSurface { owner: to_stable_declaration(owner) },
        Dependency::TraitSurface(owner) => Portable::TraitSurface { owner: to_stable_declaration(owner) },
        Dependency::ConformanceDispatch(implementation) => Portable::ConformanceDispatch { implementation: to_stable_impl(implementation) },
        Dependency::TraitDispatchBucket(bucket) => Portable::TraitDispatchBucket {
            target: match &bucket.target_family {
                crate::trait_dispatch::TraitDispatchTargetFamily::Declaration(declaration) => ConformanceTargetV1::Declaration {
                    declaration: to_stable_declaration(declaration),
                },
                crate::trait_dispatch::TraitDispatchTargetFamily::ExactEnumCase(variant) => ConformanceTargetV1::ExactEnumCase {
                    variant: to_stable_variant(variant),
                },
            },
            selector: export_exact_selector(&bucket.selector),
            side: to_stable_dispatch_side(bucket.side),
        },
        Dependency::GenericConformance(key) => Portable::GenericConformance {
            subject: export_type_term(&key.subject, types)?,
            trait_ref: export_trait_ref_identity(&key.trait_ref, types)?,
        },
        Dependency::HierarchyEdge(owner) => Portable::HierarchyEdge { owner: to_stable_declaration(owner) },
        Dependency::LinkedInterface(module) => Portable::LinkedInterface { module: to_stable_module(module) },
        Dependency::DataDeclaration(owner) => Portable::DataDeclaration { owner: to_stable_declaration(owner) },
        Dependency::EnumDeclaration(owner) => Portable::EnumDeclaration { owner: to_stable_declaration(owner) },
        Dependency::AssociatedSurface(owner) => Portable::AssociatedSurface { owner: to_stable_declaration(owner) },
        Dependency::TypeMemberDeclaration(member) => Portable::TypeMemberDeclaration {
            owner: to_stable_declaration(&member.owner),
            selector: export_type_member_selector(&member.selector),
        },
        Dependency::StaticDomainCaseCatalog(domain) => Portable::StaticDomainCaseCatalog {
            domain: export_static_case_domain(domain),
        },
        Dependency::StaticCaseSignature(case) => Portable::StaticCaseSignature {
            domain: export_static_case_domain(&case.domain),
            selector: export_exact_selector(&case.selector),
        },
        Dependency::StaticDefinition(owner) => Portable::StaticDefinition { owner: to_stable_declaration(owner) },
        Dependency::StaticConstant(id) => Portable::StaticConstant {
            symbol: StaticConstantSymbolV1 {
                module: to_stable_module(&id.0.module),
                name: id.0.name.clone(),
            },
        },
        Dependency::ModuleStaticBindingCatalog(module) => Portable::ModuleStaticBindingCatalog { module: to_stable_module(module) },
        Dependency::ConstructorInitialization(owner) => Portable::ConstructorInitialization { owner: to_stable_declaration(owner) },
        Dependency::ResolvedImport(site) => Portable::ResolvedImport {
            module: to_stable_module(&site.importer),
            local_id: site.local.0,
        },
        Dependency::LinkedName(module, name) => Portable::LinkedName {
            module: to_stable_module(module),
            name: name.clone().into_boxed_str(),
        },
        Dependency::PublicExport(module, name) => Portable::PublicExport {
            module: to_stable_module(module),
            name: name.clone().into_boxed_str(),
        },
        Dependency::ModuleStaticBindings(module) => Portable::ModuleStaticBindings { module: to_stable_module(module) },
    })
}

fn export_static_constant_symbol(id: &crate::static_constant::StaticConstantId) -> phalcom_compiler_artifact::semantic::world::StaticConstantSymbolV1 {
    phalcom_compiler_artifact::semantic::world::StaticConstantSymbolV1 {
        module: to_stable_module(&id.0.module),
        name: id.0.name.clone(),
    }
}

fn export_source_range(range: phalcom_common::range::SourceRange) -> Result<phalcom_compiler_artifact::module_world::SpanV1, ArtifactError> {
    Ok(phalcom_compiler_artifact::module_world::SpanV1 {
        start: u64::try_from(range.start).map_err(|_| referential("static-case source start exceeds portable range"))?,
        end: u64::try_from(range.end).map_err(|_| referential("static-case source end exceeds portable range"))?,
    })
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
) -> Result<SemanticWorldV3, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world as wire;

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

    let mut ordered_analyses = snapshot.callable_analyses.iter().collect::<Vec<_>>();
    ordered_analyses.sort_unstable_by(|(left, _), (right, _)| to_stable_callable(left).cmp(&to_stable_callable(right)));
    let mut portable_explanations = BTreeMap::new();
    let mut portable_causes = BTreeMap::new();
    for (callable, analysis) in ordered_analyses {
        let ordinals = CallableAnalysisOrdinalMap::new(callable, analysis)?;
        portable_explanations.insert(callable.clone(), ordinals.explanations);
        for cause in ordinals.diagnostic_causes.keys() {
            if !portable_causes.contains_key(cause) {
                let ordinal = u32::try_from(portable_causes.len()).map_err(|_| referential("witness diagnostic cause ordinal overflow"))?;
                portable_causes.insert(*cause, ordinal);
            }
        }
    }

    let mut ordered_witness_plans = snapshot.conformance_witness_plans.iter().collect::<Vec<_>>();
    ordered_witness_plans.sort_unstable_by(|(left, _), (right, _)| to_stable_impl(left).cmp(&to_stable_impl(right)));
    for (_, plan) in &ordered_witness_plans {
        for diagnostic in plan.diagnostics.iter().chain(plan.associated_type_plan.diagnostics.iter()) {
            if let Some(cause) = diagnostic.root_cause
                && !portable_causes.contains_key(&cause)
            {
                let ordinal = u32::try_from(portable_causes.len()).map_err(|_| referential("witness diagnostic cause ordinal overflow"))?;
                portable_causes.insert(cause, ordinal);
            }
        }
    }
    for rule in snapshot.derived_conformance_rules.iter() {
        for diagnostic in &rule.associated_type_plan.diagnostics {
            if let Some(cause) = diagnostic.root_cause
                && !portable_causes.contains_key(&cause)
            {
                let ordinal = u32::try_from(portable_causes.len()).map_err(|_| referential("derived-rule diagnostic cause ordinal overflow"))?;
                portable_causes.insert(cause, ordinal);
            }
        }
    }

    let mut ordered_static_definitions = snapshot.static_definitions.iter().collect::<Vec<_>>();
    ordered_static_definitions.sort_unstable_by(|(left, _), (right, _)| to_stable_declaration(left).cmp(&to_stable_declaration(right)));
    for (_, product) in &ordered_static_definitions {
        for diagnostic in product.diagnostics.iter() {
            if let Some(cause) = diagnostic.root_cause
                && !portable_causes.contains_key(&cause)
            {
                let ordinal = u32::try_from(portable_causes.len()).map_err(|_| referential("static diagnostic cause ordinal overflow"))?;
                portable_causes.insert(cause, ordinal);
            }
        }
    }
    let mut ordered_static_constants = snapshot.static_constants.iter().collect::<Vec<_>>();
    ordered_static_constants.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
    for (_, product) in &ordered_static_constants {
        for diagnostic in product.diagnostics.iter() {
            if let Some(cause) = diagnostic.root_cause
                && !portable_causes.contains_key(&cause)
            {
                let ordinal = u32::try_from(portable_causes.len()).map_err(|_| referential("static constant diagnostic cause ordinal overflow"))?;
                portable_causes.insert(cause, ordinal);
            }
        }
    }
    let mut static_definitions = Vec::with_capacity(ordered_static_definitions.len());
    for (owner, product) in ordered_static_definitions {
        if &product.definition.id != owner {
            return Err(referential("static definition table key disagrees with its product owner"));
        }
        let program = product
            .canonical_program
            .as_ref()
            .ok_or_else(|| referential("static definition has no canonical static program"))?;
        if program.owner.0 != *owner {
            return Err(referential("canonical static program belongs to a different definition"));
        }
        let alias_info = if let Some(alias) = &product.alias_info {
            if alias.declaration != *owner || snapshot.type_aliases.get(owner) != Some(alias.as_ref()) {
                return Err(referential("static definition alias projection disagrees with structural alias authority"));
            }
            Some(to_stable_declaration(owner))
        } else {
            None
        };
        let dependencies = program
            .dependencies
            .iter()
            .map(|dependency| export_static_semantic_dependency(dependency, types))
            .collect::<Result<Vec<_>, _>>()?;
        let canonical_program = crate::artifact::operational::export_canonical_static_program(program, statics, types, dependencies)?;
        let signature = crate::artifact::operational::export_static_definition_signature(&product.definition.signature, types)?;
        let diagnostics = product
            .diagnostics
            .iter()
            .map(|diagnostic| export_semantic_diagnostic(diagnostic, None, Some(&portable_explanations), Some(&portable_causes), types))
            .collect::<Result<Vec<_>, _>>()?;
        static_definitions.push(wire::StaticDefinitionRecordV1 {
            id: to_stable_declaration(owner),
            signature,
            source: export_source_range(product.definition.source_range)?,
            alias_info,
            canonical_program,
            diagnostics,
        });
    }

    let mut static_constants = Vec::with_capacity(ordered_static_constants.len());
    for (owner, product) in ordered_static_constants {
        if owner != &product.id {
            return Err(referential("static constant table key disagrees with its product owner"));
        }
        let crate::static_eval::StaticEvalOutcome::Reduced(value) = &product.outcome else {
            return Err(referential("canonical static constants require Reduced outcomes"));
        };
        if product.value != Some(*value)
            || product.checked.as_ref().is_some_and(|checked| crate::static_constant::StaticConstantAuthority::from_checked(checked) != product.authority)
        {
            return Err(referential("static constant outcome or authority disagrees with its source product"));
        }
        let dependencies = product
            .authority
            .dependencies
            .iter()
            .map(|dependency| export_static_semantic_dependency(dependency, types))
            .collect::<Result<Vec<_>, _>>()?;
        let static_constant_reads = product
            .authority
            .static_constant_reads
            .iter()
            .map(|symbol| export_static_constant_symbol(&crate::static_constant::StaticConstantId(symbol.clone())))
            .collect();
        let record_key_static_constant_reads = product
            .authority
            .record_key_static_constant_reads
            .iter()
            .map(|symbol| export_static_constant_symbol(&crate::static_constant::StaticConstantId(symbol.clone())))
            .collect();
        let diagnostics = product
            .diagnostics
            .iter()
            .map(|diagnostic| export_semantic_diagnostic(diagnostic, None, Some(&portable_explanations), Some(&portable_causes), types))
            .collect::<Result<Vec<_>, _>>()?;
        static_constants.push(wire::StaticConstantRecordV1 {
            symbol: export_static_constant_symbol(owner),
            declaration_source: export_source_range(product.declaration_range)?,
            initializer_source: export_source_range(product.initializer_range)?,
            input_fingerprint: product.authority.input_fingerprint.raw(),
            dependencies,
            static_constant_reads,
            record_key_static_constant_reads,
            result_type: product.authority.result_type.map(|ty| types.ty(ty)).transpose()?,
            outcome: wire::StaticConstantOutcomeV1::Reduced { value: statics.value(*value)? },
            lkg_value: product.lkg_value.map(|value| statics.value(value)).transpose()?,
            diagnostics,
        });
    }

    let mut ordered_module_bindings = snapshot.module_static_bindings.iter().collect::<Vec<_>>();
    ordered_module_bindings.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
    let mut module_static_bindings = Vec::with_capacity(ordered_module_bindings.len());
    for (module, index) in ordered_module_bindings {
        if index.module() != module {
            return Err(referential("module static-binding table key disagrees with its index owner"));
        }
        let mut bindings = Vec::with_capacity(index.len());
        for (symbol, fact) in index.iter() {
            if symbol != &fact.symbol {
                return Err(referential("module static-binding key disagrees with its fact symbol"));
            }
            let analysis = snapshot
                .callable_analyses
                .get(&fact.source_body)
                .ok_or_else(|| referential("module static binding references an absent callable analysis"))?;
            if analysis.bindings.get(&fact.source_binding) != Some(&fact.state) {
                return Err(referential("module static binding state disagrees with its callable owner row"));
            }
            let ordinals = CallableAnalysisOrdinalMap::new(&fact.source_body, analysis)?;
            let source_binding_ordinal = ordinals
                .bindings
                .get(&fact.source_binding)
                .copied()
                .ok_or_else(|| referential("module static binding references an absent owner-local binding ordinal"))?;
            bindings.push(wire::ModuleStaticBindingRecordV1 {
                symbol: export_static_constant_symbol(&crate::static_constant::StaticConstantId(symbol.clone())),
                source_body: to_stable_callable(&fact.source_body),
                source_binding_ordinal,
                phase: match fact.phase {
                    crate::module_bindings::ModuleBindingPhase::StaticConstant => wire::ModuleStaticBindingPhaseV1::StaticConstant,
                    crate::module_bindings::ModuleBindingPhase::Runtime => wire::ModuleStaticBindingPhaseV1::Runtime,
                },
                declaration_source: export_source_range(fact.declaration_range)?,
                initializer_source: fact.initializer_range.map(export_source_range).transpose()?,
            });
        }
        let visible_names = index
            .visible_symbol_bindings()
            .map(|(name, symbol)| wire::ModuleStaticBindingVisibleNameV1 {
                name: name.into(),
                symbol: export_static_constant_symbol(&crate::static_constant::StaticConstantId(symbol.clone())),
            })
            .collect();
        module_static_bindings.push(wire::ModuleStaticBindingIndexV1 {
            module: to_stable_module(module),
            bindings,
            visible_names,
        });
    }

    let mut witness_plans = Vec::new();
    for (owner, plan) in ordered_witness_plans {
        if owner != &plan.impl_id || plan.associated_type_plan.impl_id != plan.impl_id {
            return Err(referential("conformance witness plan identity disagrees with its owner"));
        }
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
        let mut conformance_export = CallableConformanceEvidenceExport {
            types,
            ordinals: None,
            portable_explanations: Some(&portable_explanations),
            portable_causes: Some(&portable_causes),
            max_nodes: limits.max_semantic_entries_per_family,
            max_depth: limits.max_recursion_depth,
            active_arc_addresses: std::collections::HashSet::new(),
            nodes: Vec::new(),
        };
        let requirement_views = export_callable_requirement_views(plan.requirement_views.iter(), types)?;
        let mut requirements = Vec::with_capacity(plan.requirements.len());
        for (req_id, selection) in &plan.requirements {
            if req_id.owner != plan.trait_ref_template.declaration {
                return Err(referential("witness requirement selection belongs to another trait"));
            }
            requirements.push(wire::CallableRequirementSelectionEntryV1 {
                requirement: export_callable_requirement_identity(req_id),
                selection: conformance_export.selection(selection, 0)?,
            });
        }
        let associated_type_plan = conformance_export.associated_type_plan(&plan.associated_type_plan)?;
        let completeness = conformance_export.completeness(&plan.completeness)?;
        let diagnostics = plan
            .diagnostics
            .iter()
            .map(|diagnostic| export_semantic_diagnostic(diagnostic, None, Some(&portable_explanations), Some(&portable_causes), types))
            .collect::<Result<Vec<_>, _>>()?;
        if plan.fingerprint != crate::impls::conformance_witness_plan_fingerprint(plan) {
            return Err(referential("conformance witness plan fingerprint is stale"));
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
        witness_plans.push(WitnessPlanV1 {
            impl_id,
            source_module,
            trait_ref_template,
            target_template,
            generic_signature,
            requirement_views,
            requirements,
            associated_type_plan,
            invalid_explicit_members: plan.invalid_explicit_members,
            completeness,
            diagnostics,
            conformance_evidence: conformance_export.nodes,
            fingerprint,
            ordered_selected_callables,
            slot_layout_fingerprint,
        });
    }

    let mut ordered_derived_rules = snapshot.derived_conformance_rules.iter().collect::<Vec<_>>();
    ordered_derived_rules.sort_unstable_by(|left, right| to_stable_impl(&left.impl_id).cmp(&to_stable_impl(&right.impl_id)));
    let mut derived_conformance_rules = Vec::with_capacity(ordered_derived_rules.len());
    for rule in ordered_derived_rules {
        if rule.associated_type_plan.impl_id != rule.impl_id {
            return Err(referential("derived conformance rule identity disagrees with its owner"));
        }
        if rule.fingerprint != crate::derived_conformance::derived_conformance_rule_fingerprint(rule) {
            return Err(referential("derived conformance rule fingerprint is stale"));
        }
        let mut conformance_export = CallableConformanceEvidenceExport {
            types,
            ordinals: None,
            portable_explanations: Some(&portable_explanations),
            portable_causes: Some(&portable_causes),
            max_nodes: limits.max_semantic_entries_per_family,
            max_depth: limits.max_recursion_depth,
            active_arc_addresses: std::collections::HashSet::new(),
            nodes: Vec::new(),
        };
        let rule = conformance_export.derived_rule(rule, 0)?;
        derived_conformance_rules.push(wire::DerivedConformanceRuleRecordV1 {
            rule,
            conformance_evidence: conformance_export.nodes,
        });
    }

    let mut callable_signatures = Vec::new();
    for (_, sig) in snapshot.callable_signatures.iter() {
        callable_signatures.push(export_callable_signature(sig, types)?);
    }
    callable_signatures.sort_unstable_by(|left, right| left.callable.cmp(&right.callable));

    let mut callable_definitions = Vec::new();
    for (callable, definition) in snapshot.callable_definitions.iter() {
        if callable != &definition.callable {
            return Err(referential("callable definition table key disagrees with its owner"));
        }
        let canonical_sig = snapshot
            .callable_signatures
            .get(callable)
            .ok_or_else(|| referential("callable definition has no canonical signature row"))?;
        let mut synchronized = definition.clone();
        synchronized.signature = canonical_sig.clone();
        callable_definitions.push(export_callable_definition(&synchronized, &snapshot.store, types)?);
    }
    callable_definitions.sort_unstable_by(|left, right| left.callable.cmp(&right.callable));

    let mut callable_analyses = Vec::new();
    for (callable_id, analysis) in snapshot.callable_analyses.iter() {
        let source_callable = snapshot.callable_signatures.id_for_body(callable_id).unwrap_or_else(|| callable_id.clone());
        let mut binding_source_authorities = std::collections::BTreeMap::new();
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
                binding_source_authorities.insert(
                    *binding,
                    CallableBindingSourceAuthority::Source {
                        site: site.id.clone(),
                        range: site.range,
                    },
                );
            } else if let Some(parameter) = state.parameter.as_ref()
                && snapshot.source_index.parameter_source_site_is_indexed(&source_callable, parameter) == Some(false)
            {
                binding_source_authorities.insert(*binding, CallableBindingSourceAuthority::GeneratedReceiver);
            } else {
                return Err(referential("callable source binding lacks its canonical formal attachment"));
            }
        }
        callable_analyses.push(export_callable_analysis_record(
            callable_id,
            analysis,
            &binding_source_authorities,
            types,
            limits,
        )?);
    }
    callable_analyses.sort_unstable_by(|left, right| left.callable.cmp(&right.callable));

    let mut trait_owners = snapshot.trait_surfaces.iter().collect::<Vec<_>>();
    trait_owners.sort_unstable_by(|(left, _), (right, _)| to_stable_declaration(left).cmp(&to_stable_declaration(right)));
    let mut portable_explanations = BTreeMap::new();
    for (callable, analysis) in snapshot.callable_analyses.iter() {
        portable_explanations.insert(callable.clone(), assign_local_ordinals(analysis.explanations.iter().map(|(id, _)| *id))?);
    }
    let mut portable_causes = BTreeMap::new();
    for (_, surface) in &trait_owners {
        for diagnostic in &surface.diagnostics {
            let Some(cause) = diagnostic.root_cause else {
                continue;
            };
            if !portable_causes.contains_key(&cause) {
                let ordinal = u32::try_from(portable_causes.len()).map_err(|_| referential("trait diagnostic cause ordinal overflow"))?;
                portable_causes.insert(cause, ordinal);
            }
        }
    }

    let mut trait_surfaces = Vec::new();
    for (decl_id, surface) in trait_owners {
        let declaration = to_stable_declaration(decl_id);
        let header = snapshot
            .trait_headers
            .get(decl_id)
            .ok_or_else(|| referential("trait surface has no canonical trait-header owner"))?;
        if &header.declaration != decl_id || header.generic_signature != surface.generic_signature {
            return Err(referential("trait surface generic signature disagrees with its header owner"));
        }
        let generic_signature = header
            .generic_signature
            .as_ref()
            .map(|signature| export_generic_signature(signature, types))
            .transpose()?;
        let members = surface
            .iter()
            .map(|(requirement, member)| -> Result<_, ArtifactError> {
                if &member.requirement != requirement
                    || &requirement.owner != decl_id
                    || member.signature.callable != member.callable
                    || member.signature.owner != *decl_id
                    || member.signature.selector != requirement.selector
                    || member.signature.side != requirement.side
                    || member.default_present != member.default_source.is_some()
                {
                    return Err(referential("trait surface member disagrees with its complete callable owner"));
                }
                Ok(TraitSurfaceMemberV1 {
                    requirement: CallableRequirementIdentityV1 {
                        owner: declaration.clone(),
                        selector: requirement.selector.to_string().into_boxed_str(),
                        side: match requirement.side {
                            crate::identity::DispatchSide::Instance => 0,
                            crate::identity::DispatchSide::Class => 1,
                        },
                    },
                    callable: to_stable_callable(&member.callable),
                    signature: export_callable_signature(
                        snapshot
                            .callable_signatures
                            .get(&member.callable)
                            .unwrap_or(&member.signature),
                        types,
                    )?,
                    visibility: export_visibility(member.visibility),
                    source: export_source_span(&member.source)?,
                    source_member_index: u32::try_from(member.source_member_index)
                        .map_err(|_| referential("trait member source ordinal exceeds portable range"))?,
                    default_present: member.default_present,
                    default_source: member.default_source.as_ref().map(export_source_span).transpose()?,
                })
            })
            .collect::<Result<Vec<_>, ArtifactError>>()?;
        let associated_types = surface
            .associated_types
            .iter()
            .map(|(requirement, associated)| -> Result<_, ArtifactError> {
                if &associated.requirement != requirement || &requirement.owner != decl_id {
                    return Err(referential("trait associated-type owner disagrees with its surface key"));
                }
                let domain = match associated.domain {
                    crate::types::type_member::TypeMemberDomain::Kind(kind) => TypeMemberDomainV1::Kind { kind: types.kind(kind)? },
                    crate::types::type_member::TypeMemberDomain::ValueType(ty) => TypeMemberDomainV1::ValueType { ty: types.ty(ty)? },
                };
                Ok(TraitAssociatedTypeRequirementV1 {
                    requirement: CallableAssociatedTypeRequirementIdentityV1 {
                        owner: declaration.clone(),
                        index: requirement.index,
                    },
                    name: associated.name.clone(),
                    domain,
                    source: export_source_span(&associated.source)?,
                })
            })
            .collect::<Result<Vec<_>, ArtifactError>>()?;
        let diagnostics = surface
            .diagnostics
            .iter()
            .map(|diagnostic| export_semantic_diagnostic(diagnostic, None, Some(&portable_explanations), Some(&portable_causes), types))
            .collect::<Result<Vec<_>, _>>()?;
        trait_surfaces.push(TraitSurfaceRecordV1 {
            declaration,
            generic_signature,
            header_source: export_source_span(&header.source)?,
            members,
            associated_types,
            diagnostics,
        });
    }
    trait_surfaces.sort_unstable_by(|left, right| left.declaration.cmp(&right.declaration));

    let mut dispatch_surfaces = Vec::new();
    for (decl_id, surface) in snapshot.surfaces.iter() {
        let declaration = to_stable_declaration(decl_id);
        let dispatch_callables =
            |side: crate::identity::DispatchSide, member_surface: &crate::surface::MemberSurface| -> Result<Vec<wire::DispatchCallableFactV1>, ArtifactError> {
                let mut callables = Vec::with_capacity(member_surface.callables.len());
                for (callable, knowledge) in &member_surface.callables {
                    if callable.owner != crate::identity::CallableOwnerId::Declaration(decl_id.clone()) || callable.side != side {
                        return Err(referential("declaration dispatch callable owner or side disagrees with its surface"));
                    }
                    let semantic = snapshot
                        .callable_signatures
                        .get(callable)
                        .ok_or_else(|| referential(format!("declaration dispatch callable has no canonical semantic signature: {callable:?}")))?;
                    let projected = crate::checker::declaration_signature::project_semantic_signature(semantic);
                    let retained = member_surface
                        .callable_signatures
                        .get(&callable.selector)
                        .ok_or_else(|| referential("declaration dispatch callable has no final projected signature"))?;
                    if retained.selector != callable.selector || &retained.return_type != knowledge {
                        return Err(referential("declaration dispatch callable knowledge disagrees with its final projection"));
                    }
                    let mut projected_with_final_return = projected;
                    projected_with_final_return.return_type = retained.return_type.clone();
                    if projected_with_final_return != *retained {
                        return Err(referential("declaration dispatch signature differs from its canonical semantic projection"));
                    }
                    let visibility = member_surface
                        .callable_visibility
                        .get(&callable.selector)
                        .copied()
                        .ok_or_else(|| referential("declaration dispatch callable has no retained visibility"))?;
                    callables.push(wire::DispatchCallableFactV1 {
                        callable: to_stable_callable(callable),
                        knowledge: export_type_knowledge(knowledge, types)?,
                        visibility: export_visibility(visibility),
                    });
                }
                callables.sort_unstable_by(|left, right| left.callable.cmp(&right.callable));
                Ok(callables)
            };
        let instance_callables = dispatch_callables(crate::identity::DispatchSide::Instance, &surface.instance)?;
        let class_callables = dispatch_callables(crate::identity::DispatchSide::Class, &surface.class)?;
        let mut fields = Vec::new();
        for (field_side, member_surface) in [
            (crate::identity::DispatchSide::Instance, &surface.instance),
            (crate::identity::DispatchSide::Class, &surface.class),
        ] {
            for (name, knowledge) in &member_surface.fields {
                let field_id = member_surface
                    .fields_by_name
                    .get(name)
                    .ok_or_else(|| referential("declaration field has no stable field identity"))?;
                if field_id.owner != *decl_id || field_id.side != field_side || field_id.name.as_ref() != name {
                    return Err(referential("declaration field identity disagrees with its dispatch surface"));
                }
                let field_signature = snapshot.field_signatures.get(field_id);
                if field_signature.is_some_and(|signature| {
                    signature.field != *field_id || signature.owner != *decl_id || signature.side != field_side || signature.name.as_ref() != name
                }) {
                    return Err(referential("declaration field signature disagrees with its stable field identity"));
                }
                let visibility = member_surface
                    .field_visibility
                    .get(name)
                    .copied()
                    .ok_or_else(|| referential("declaration field has no retained visibility"))?;
                let (declared_type, mutable, source) = match field_signature {
                    Some(signature) => {
                        if signature.source.as_ref().is_some_and(|source| source.module != decl_id.module) {
                            return Err(referential("declaration field source belongs to a different module"));
                        }
                        let source = signature
                            .source
                            .as_ref()
                            .map(|source| {
                                Ok::<_, ArtifactError>(phalcom_compiler_artifact::module_world::SpanV1 {
                                    start: u64::try_from(source.range.start).map_err(|_| referential("field source start exceeds portable range"))?,
                                    end: u64::try_from(source.range.end).map_err(|_| referential("field source end exceeds portable range"))?,
                                })
                            })
                            .transpose()?;
                        (
                            Some(export_declared_type_fact(&signature.declared_type, types)?),
                            Some(signature.mutable),
                            source,
                        )
                    }
                    None => (None, None, None),
                };
                fields.push(FieldSignatureV1 {
                    owner: to_stable_declaration(decl_id),
                    name: name.clone().into_boxed_str(),
                    declared_type,
                    knowledge: export_type_knowledge(knowledge, types)?,
                    mutable,
                    side: match field_side {
                        crate::identity::DispatchSide::Instance => 0,
                        crate::identity::DispatchSide::Class => 1,
                    },
                    visibility: export_visibility(visibility),
                    source,
                });
            }
        }
        for (field_id, field_signature) in snapshot.field_signatures.iter_owner(decl_id) {
            let member_surface = surface.surface(field_signature.side);
            if field_signature.owner != *decl_id || field_signature.field != *field_id || !member_surface.fields.contains_key(field_signature.name.as_ref()) {
                return Err(referential("declaration field signature has no final dispatch surface field"));
            }
        }
        fields.sort_unstable_by(|left, right| (left.side, &left.name).cmp(&(right.side, &right.name)));
        dispatch_surfaces.push(DeclarationSurfaceRecordV1 {
            declaration,
            instance_callables,
            class_callables,
            fields,
        });
    }

    let mut conditional_inherent_sets = Vec::new();
    for (target, set) in snapshot.dispatch.conditional_members().iter() {
        let portable_target = match target {
            crate::impls::InherentImplTarget::Declaration(declaration) => wire::InherentImplTargetV1::Declaration {
                declaration: to_stable_declaration(declaration),
            },
            crate::impls::InherentImplTarget::ExactEnumCase(variant) => wire::InherentImplTargetV1::ExactEnumCase {
                variant: to_stable_variant(variant),
            },
        };
        if set.target.as_ref() != Some(target) {
            return Err(referential("conditional inherent member set target disagrees with its resolver owner"));
        }
        let mut domains = BTreeMap::new();
        for member in &set.members {
            if let Some(existing) = domains.insert(member.impl_id.clone(), member.domain.clone())
                && existing.as_ref() != member.domain.as_ref()
            {
                return Err(referential("conditional inherent members disagree about their shared impl domain"));
            }
        }
        let domains = domains
            .into_iter()
            .map(|(impl_id, domain)| -> Result<_, ArtifactError> {
                if domain.impl_id != impl_id || &domain.target != target {
                    return Err(referential("conditional inherent domain identity disagrees with its set"));
                }
                Ok(wire::InherentImplDomainV1 {
                    impl_id: to_stable_impl(&impl_id),
                    target: portable_target.clone(),
                    head_type: types.ty(domain.head_type)?,
                    generic_signature: domain
                        .generic_signature
                        .as_ref()
                        .map(|signature| export_generic_signature(signature, types))
                        .transpose()?,
                    constraints: domain
                        .constraints
                        .iter()
                        .map(|constraint| export_constraint(constraint, types))
                        .collect::<Result<Vec<_>, _>>()?,
                })
            })
            .collect::<Result<Vec<_>, ArtifactError>>()?;
        let members = set
            .members
            .iter()
            .enumerate()
            .map(|(member_order, member)| -> Result<_, ArtifactError> {
                let member_order = u32::try_from(member_order).map_err(|_| referential("conditional inherent member order exceeds portable range"))?;
                Ok(wire::ConditionalInherentMemberV1 {
                    impl_id: to_stable_impl(&member.impl_id),
                    member_order,
                    is_conditionally_applicable: member.is_conditionally_applicable,
                    callable: to_stable_callable(&member.callable),
                    signature: export_callable_signature(&member.signature_template, types)?,
                    visibility: export_visibility(member.visibility),
                    source_member_index: u32::try_from(member.source_member)
                        .map_err(|_| referential("conditional inherent source ordinal exceeds portable range"))?,
                    is_requirement: member.is_requirement,
                })
            })
            .collect::<Result<Vec<_>, ArtifactError>>()?;
        conditional_inherent_sets.push(wire::ConditionalInherentSetV1 {
            target: portable_target,
            domains,
            members,
        });
    }
    conditional_inherent_sets.sort_unstable_by(|left, right| match (&left.target, &right.target) {
        (wire::InherentImplTargetV1::Declaration { declaration: left }, wire::InherentImplTargetV1::Declaration { declaration: right }) => left.cmp(right),
        (wire::InherentImplTargetV1::ExactEnumCase { variant: left }, wire::InherentImplTargetV1::ExactEnumCase { variant: right }) => left.cmp(right),
        (wire::InherentImplTargetV1::Declaration { .. }, wire::InherentImplTargetV1::ExactEnumCase { .. }) => std::cmp::Ordering::Less,
        (wire::InherentImplTargetV1::ExactEnumCase { .. }, wire::InherentImplTargetV1::Declaration { .. }) => std::cmp::Ordering::Greater,
    });

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

    let mut static_case_catalogs = Vec::with_capacity(snapshot.static_case_catalogs.len());
    for (domain, catalog) in snapshot.static_case_catalogs.iter() {
        if domain != &catalog.domain {
            return Err(referential("static-case catalog table key disagrees with its owner"));
        }
        let mut cases = Vec::with_capacity(catalog.cases.len());
        for case in catalog.cases.iter() {
            if case.id.domain != *domain {
                return Err(referential("static-case identity belongs to another catalog domain"));
            }
            let shape = match &case.shape {
                crate::static_case::StaticCaseShape::Singleton => wire::StaticCaseShapeV1::Singleton,
                crate::static_case::StaticCaseShape::Constructor { slots } => wire::StaticCaseShapeV1::Constructor {
                    slots: slots.iter().map(export_selector_slot).collect(),
                },
            };
            cases.push(wire::StaticCaseHeaderV1 {
                selector: export_exact_selector(&case.id.selector),
                shape,
                source: export_source_range(case.source_range)?,
            });
        }
        static_case_catalogs.push(wire::StaticDomainCaseCatalogRecordV1 {
            domain: export_static_case_domain(domain),
            cases,
        });
    }
    let mut static_case_signatures = Vec::with_capacity(snapshot.static_case_signatures.len());
    for (case_id, signature) in snapshot.static_case_signatures.iter() {
        if case_id != &signature.id {
            return Err(referential("static-case signature table key disagrees with its owner"));
        }
        static_case_signatures.push(wire::StaticCaseSignatureRecordV1 {
            domain: export_static_case_domain(&case_id.domain),
            selector: export_exact_selector(&case_id.selector),
            parameter_kinds: signature.parameter_kinds.iter().map(|&kind| types.kind(kind)).collect::<Result<Vec<_>, _>>()?,
            source: export_source_range(signature.source_range)?,
        });
    }

    let world = SemanticWorldV3 {
        type_graph,
        structural_contracts,
        enum_requirements,
        static_values,
        static_definitions,
        static_constants,
        module_static_bindings,
        witness_plans,
        derived_conformance_rules,
        callable_signatures,
        callable_definitions,
        callable_analyses,
        trait_surfaces,
        dispatch_surfaces,
        conditional_inherent_sets,
        conformance_heads,
        constructor_initializations,
        type_member_declarations,
        associated_static_domains,
        static_case_catalogs,
        static_case_signatures,
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
    for (owner, plan) in snapshot.conformance_witness_plans.iter() {
        if owner != &plan.impl_id || plan.associated_type_plan.impl_id != plan.impl_id {
            return Err(referential("conformance witness plan identity disagrees with its owner"));
        }
        if plan.fingerprint != crate::impls::conformance_witness_plan_fingerprint(plan) {
            return Err(referential("conformance witness plan fingerprint is stale"));
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
        use phalcom_compiler_artifact::semantic::world::{CallableAnalysisStatusV1 as PortableStatus, CallableExpressionStatusV1 as ExpressionStatus};

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
            local_proof_graph: std::sync::Arc::new(Default::default()),
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
        let blocked_subject = store.nominal_type(crate::identity::DeclarationId::new(callable.module().clone(), "BlockedSubject".into()));
        let blocked_argument = store.nominal_type(crate::identity::DeclarationId::new(callable.module().clone(), "BlockedArgument".into()));
        let call_expression = analysis.expressions.get_source_mut(&expression_a).unwrap().unwrap();
        call_expression.callable = Some(callable.clone());
        call_expression.knowledge = crate::types::evidence::TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::ExplicitEscape);
        call_expression.binding_read = Some(binding);
        call_expression.binding_write = Some(binding);
        call_expression.binding_read_safety = Some(crate::checker::flow::BindingReadSafety::RuntimeCheckRequired);
        let conditional_impl = crate::identity::ImplId::new(callable.module().clone(), crate::identity::ImplLocalId(4));
        call_expression.conditional_dispatch = Some(crate::dispatch::ConditionalDispatchSelection {
            impl_id: conditional_impl.clone(),
            callable: callable.clone(),
            declaring_owner: owner.clone(),
            side: DispatchSide::Class,
        });
        call_expression.status = crate::checker::analysis::AnalysisStatus::Blocked(crate::types::outcome::BlockReason::UnavailableSemanticOwner(
            crate::db::QueryKey::GenericConformance(crate::generic_evidence::GenericConformanceKey::new(
                crate::types::parameter::TypeTerm::Canonical(blocked_subject),
                crate::traits::TraitRef::new(
                    crate::identity::DeclarationId::new(callable.module().clone(), "BlockedTrait".into()),
                    vec![blocked_argument].into_boxed_slice(),
                ),
            )),
        ));
        call_expression.call_specialization = Some(crate::checker::analysis::CallSpecialization {
            callable: callable.clone(),
            bindings: Box::new([(parameter, crate::types::store::StaticArgument::DynamicType)]),
            conformances: Box::default(),
        });
        let mut types = crate::types::store::artifact::TypeExportContext::new(&store);
        super::super::roots::register_callable_analysis_roots(&mut types, &analysis, &store).unwrap();
        types.close_graph(&store, &ValidationLimits::default()).unwrap();
        let mut parameter_only = crate::types::store::artifact::TypeExportContext::new(&store);
        let parameter_only_lineage = parameter_only.expected_lineage();
        for (product_subkey, ty) in [(100, blocked_subject), (101, blocked_argument)] {
            parameter_only
                .register_type_root(
                    parameter_only_lineage,
                    phalcom_compiler_artifact::semantic::type_store::CanonicalTypeRootKey {
                        family_tag: phalcom_compiler_artifact::semantic::type_store::CanonicalRootFamilyTag::CallableAnalysis,
                        owner: phalcom_compiler_artifact::semantic::type_store::StableOwnerRef::Callable(to_stable_callable(&callable)),
                        product_subkey,
                    },
                    ty,
                )
                .unwrap();
        }
        parameter_only
            .register_parameter_root(
                parameter_only_lineage,
                phalcom_compiler_artifact::semantic::type_store::CanonicalTypeRootKey {
                    family_tag: phalcom_compiler_artifact::semantic::type_store::CanonicalRootFamilyTag::CallableAnalysis,
                    owner: phalcom_compiler_artifact::semantic::type_store::StableOwnerRef::Callable(to_stable_callable(&callable)),
                    product_subkey: 102,
                },
                parameter,
            )
            .unwrap();
        parameter_only.close_graph(&store, &ValidationLimits::default()).unwrap();
        assert_eq!(
            types.project_graph(&store, &ValidationLimits::default()).unwrap(),
            parameter_only.project_graph(&store, &ValidationLimits::default()).unwrap(),
            "the blocked query owner roots its types, while Dynamic adds no further proper-type root"
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

        let binding_source_authorities = std::collections::BTreeMap::from([(
            binding,
            CallableBindingSourceAuthority::Source {
                site: crate::identity::SourceSiteId {
                    owner: crate::identity::SourceOwner::Module(callable.module().clone()),
                    local: crate::identity::SourceSiteLocalId(3),
                },
                range: SourceRange { start: 30, end: 35 },
            },
        )]);
        let compiler_key = crate::generic_evidence::GenericConformanceKey::new(
            crate::types::parameter::TypeTerm::Canonical(blocked_subject),
            crate::traits::TraitRef::new(
                crate::identity::DeclarationId::new(callable.module().clone(), "CompilerTrait".into()),
                vec![blocked_argument].into_boxed_slice(),
            ),
        );
        let forwarded_callee_key = crate::generic_evidence::GenericConformanceKey::new(
            crate::types::parameter::TypeTerm::Canonical(blocked_argument),
            crate::traits::TraitRef::new(
                crate::identity::DeclarationId::new(callable.module().clone(), "ForwardedTrait".into()),
                Vec::new(),
            ),
        );
        let forwarded_caller_key = crate::generic_evidence::GenericConformanceKey::new(
            crate::types::parameter::TypeTerm::Canonical(blocked_subject),
            crate::traits::TraitRef::new(crate::identity::DeclarationId::new(callable.module().clone(), "CallerTrait".into()), Vec::new()),
        );
        let call = analysis
            .expressions
            .get_source_mut(&expression_a)
            .unwrap()
            .unwrap()
            .call_specialization
            .as_mut()
            .unwrap();
        call.conformances = vec![
            crate::checker::analysis::GenericCallConformanceBinding {
                callee_conformance: compiler_key.clone(),
                source: crate::checker::analysis::GenericCallConformanceSource::CompilerDerived {
                    evidence: std::sync::Arc::new(crate::generic_evidence::CompilerDerivedConformanceEvidence {
                        key: compiler_key,
                        kind: crate::generic_evidence::CompilerDerivedConformanceKind::OrderedProduct,
                        classification: crate::structural::StructuralClassification::OrderedProduct(crate::structural::OrderedProductForm::Unit),
                    }),
                },
            },
            crate::checker::analysis::GenericCallConformanceBinding {
                callee_conformance: forwarded_callee_key,
                source: crate::checker::analysis::GenericCallConformanceSource::Forwarded {
                    caller_conformance: forwarded_caller_key,
                },
            },
        ]
        .into_boxed_slice();
        let mut types = crate::types::store::artifact::TypeExportContext::new(&store);
        super::super::roots::register_callable_analysis_roots(&mut types, &analysis, &store).unwrap();
        types.close_graph(&store, &ValidationLimits::default()).unwrap();
        let record = export_callable_analysis_record(&callable, &analysis, &binding_source_authorities, &types, &ValidationLimits::default()).unwrap();
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
        let binding_row = record.bindings.first().expect("portable binding row");
        assert_eq!(binding_row.name, "local");
        assert_eq!(binding_row.range.start, 30);
        assert_eq!(binding_row.range.end, 35);
        assert!(matches!(
            &binding_row.authority,
            CallableBindingAuthorityV1::BodyLocalSource {
                site_owner: CallableAnalysisSourceOwnerV1::Module { module },
                site_local: 3,
                declaration_range: phalcom_compiler_artifact::module_world::SpanV1 { start: 30, end: 35 },
            } if module == &to_stable_module(callable.module())
        ));
        assert_eq!(record.expressions.len(), 2);
        let expression_row = &record.expressions[ordinals.expressions[&expression_a] as usize];
        assert_eq!(expression_row.range.start, 20);
        assert_eq!(expression_row.range.end, 21);
        assert!(matches!(
            &expression_row.knowledge,
            phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Dynamic {
                reason: phalcom_compiler_artifact::semantic::aggregates::DynamicReasonV1::ExplicitEscape,
            }
        ));
        assert_eq!(expression_row.callable, Some(to_stable_callable(&callable)));
        assert_eq!(
            expression_row.conditional_dispatch,
            Some(phalcom_compiler_artifact::semantic::world::ConditionalDispatchSelectionV1 {
                impl_id: to_stable_impl(&conditional_impl),
                callable: to_stable_callable(&callable),
                declaring_owner: to_stable_declaration(&owner),
                side: 1,
            })
        );
        assert_eq!(
            expression_row.binding_read_safety,
            Some(phalcom_compiler_artifact::semantic::world::CallableBindingReadSafetyV1::RuntimeCheckRequired)
        );
        assert_eq!(expression_row.binding_read, Some(0));
        assert_eq!(expression_row.binding_write, Some(0));
        assert!(matches!(
            &expression_row.status,
            ExpressionStatus::Blocked {
                reason: CallableBlockReasonV1::UnavailableSemanticOwner {
                    owner: phalcom_compiler_artifact::semantic::world::CallableQueryOwnerV1::GenericConformance {
                        subject: phalcom_compiler_artifact::semantic::aggregates::TypeTermV1::Canonical { .. },
                        trait_ref: _,
                    }
                }
            }
        ));
        let call = expression_row
            .call_specialization
            .as_ref()
            .expect("expression row owns its call specialization");
        assert_eq!(call.bindings[0].parameter, types.parameter(parameter).unwrap());
        assert!(matches!(
            call.bindings[0].argument,
            phalcom_compiler_artifact::semantic::type_store::StaticArgumentV1::DynamicType
        ));
        assert_eq!(call.conformances.len(), 2, "conformance binding order and count are preserved");
        assert!(matches!(
            call.conformances[0].source,
            phalcom_compiler_artifact::semantic::world::CallableGenericCallConformanceSourceV1::CompilerDerived { evidence: 0 }
        ));
        assert!(matches!(
            call.conformances[1].source,
            phalcom_compiler_artifact::semantic::world::CallableGenericCallConformanceSourceV1::Forwarded { .. }
        ));
        assert_eq!(record.conformance_evidence.len(), 1);
        assert!(matches!(
            record.conformance_evidence[0],
            phalcom_compiler_artifact::semantic::world::CallableConformanceEvidenceNodeV1::CompilerDerived { .. }
        ));
        let wrong_key = other_callable;
        assert!(export_callable_analysis_record(&wrong_key, &analysis, &binding_source_authorities, &types, &ValidationLimits::default()).is_err());
    }

    pub(super) fn empty_callable_analysis(callable: crate::identity::CallableId) -> crate::checker::analysis::CallableAnalysis {
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
            local_proof_graph: Arc::new(Default::default()),
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
    fn callable_owner_graph_round_trips_with_local_relocation() {
        use crate::checker::analysis::{ExpressionAnalysis, SemanticDependency};
        use crate::diagnostic::{DiagnosticCode, ExplanationRef, SemanticDiagnostic};
        use crate::explain::node::{DerivationRule, EvidenceRef, ExplanationStep};
        use crate::identity::{BodyId, CallableId, DeclarationId, DiagnosticCauseId, DispatchSide, ExpressionId, LocalExpressionId, ModuleId};
        use phalcom_common::{range::SourceRange, selector::Selector};
        use std::sync::Arc;
        let limits = ValidationLimits::default();
        let owner = DeclarationId::new(
            ModuleId::universe(phalcom_modules::ModulePath::from_components(vec![
                phalcom_modules::ModuleComponent::from_identifier("owner_graph").unwrap(),
            ])),
            "Owner".into(),
        );
        let callable = CallableId::method(owner.clone(), Selector::getter("value").unwrap(), DispatchSide::Instance);
        let mut source = empty_callable_analysis(callable.clone());
        let mut store = crate::types::store::TypeStore::new();
        let ty = store.nominal_type(owner.clone());
        let expression = ExpressionId::new(BodyId(73), LocalExpressionId(19));
        source
            .expressions
            .insert_source(
                expression,
                ExpressionAnalysis::ready(expression, SourceRange::default(), TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)),
            )
            .unwrap();
        let mut arena = crate::explain::ExplanationArena::new();
        let cause = DiagnosticCauseId(91);
        let first = arena
            .alloc_full(
                ExplanationStep::Literal { expression, ty },
                DerivationRule::LiteralSynthesis,
                EvidenceStatus::Established,
                EvidenceOrigin::Syntax,
                vec![EvidenceRef::TypeId(ty), EvidenceRef::Suppressed { cause }],
                vec![],
            )
            .unwrap();
        let last = arena
            .alloc_full(
                ExplanationStep::ProductComponent {
                    source: first,
                    index: 3,
                    result: TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                },
                DerivationRule::ProductDecomposition,
                EvidenceStatus::Assumed,
                EvidenceOrigin::PatternDecomposition,
                vec![],
                vec![first],
            )
            .unwrap();
        source.expressions.get_source_mut(&expression).unwrap().unwrap().explanation = Some(first);
        source.explanations = Arc::new(arena);
        source.return_explanation = Some(last);
        source.dependencies = Arc::from([callable.clone()]);
        source.semantic_dependencies = Arc::from([SemanticDependency::DeclarationShell(owner)]);
        source.diagnostics = Arc::from([SemanticDiagnostic::error_in(
            callable.module().clone(),
            DiagnosticCode::TypeMismatch,
            "graph diagnostic",
            SourceRange::default(),
        )
        .with_root_cause(cause)
        .with_explanation(ExplanationRef::new(callable.clone(), first))]);
        let mut types = TypeExportContext::new(&store);
        crate::artifact::roots::register_callable_analysis_roots(&mut types, &source, &store).unwrap();
        types.close_graph(&store, &limits).unwrap();
        let record = export_callable_analysis_record(&callable, &source, &BTreeMap::new(), &types, &limits).unwrap();
        assert_eq!(record.diagnostic_cause_count, 1);
        assert_eq!(record.diagnostics[0].root_cause, Some(0));
        let graph = types.project_graph(&store, &limits).unwrap();
        let (imported_store, relocation) = crate::types::store::TypeStore::import_type_graph(&graph, &limits).unwrap();
        let mut budget = phalcom_type_meta::allocation::AllocationBudget::new(1 << 20);
        let (table, _) = crate::artifact::import::import_callable_analysis_records(
            std::slice::from_ref(&record),
            &relocation,
            &imported_store,
            &BTreeMap::new(),
            &limits,
            &mut budget,
        )
        .unwrap();
        let imported = table.get(&callable).unwrap();
        assert!(imported.explanations.is_frozen());
        assert_eq!(imported.explanations.len(), 2);
        assert_eq!(imported.return_explanation, Some(crate::identity::ExplanationId(1)));
        assert_eq!(imported.diagnostics[0].root_cause, Some(DiagnosticCauseId(0)));
        assert_eq!(imported.dependencies, source.dependencies);
        assert_eq!(imported.semantic_dependencies, source.semantic_dependencies);
        assert_eq!(
            imported.dependency_fingerprint,
            crate::db::fingerprint::callable_body_product_fingerprint(imported)
        );
        let mut imported_types = TypeExportContext::new(&imported_store);
        crate::artifact::roots::register_callable_analysis_roots(&mut imported_types, imported, &imported_store).unwrap();
        imported_types.close_graph(&imported_store, &limits).unwrap();
        let reexported = export_callable_analysis_record(&callable, imported, &BTreeMap::new(), &imported_types, &limits).unwrap();
        assert_eq!(
            record, reexported,
            "complete portable authority must survive fresh-store and local-ID relocation"
        );
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
        crate::artifact::roots::register_callable_analysis_roots(&mut types, &denoted, &store).unwrap();
        types.close_graph(&store, &ValidationLimits::default()).unwrap();
        let empty_record = export_callable_analysis_record(&callable, &source, &BTreeMap::new(), &types, &ValidationLimits::default()).unwrap();
        let denoted_record = export_callable_analysis_record(&callable, &denoted, &BTreeMap::new(), &types, &ValidationLimits::default()).unwrap();

        assert_ne!(
            empty_record, denoted_record,
            "a denotation is owned expression data and must be represented in the callable-analysis record"
        );
    }

    #[test]
    fn callable_analysis_record_preserves_static_and_dynamic_family_applications() {
        use crate::checker::analysis::ExpressionAnalysis;
        use crate::checker::associated::{FamilyApplicationCandidate, FamilyApplicationKind, FamilyApplicationResolution, FamilyApplicationSelection};
        use crate::identity::{BodyId, CallableId, DeclarationId, DispatchSide, ExpressionId, InvocationTargetId, LocalExpressionId, ModuleId};
        use crate::types::evidence::{TypeKnowledge, UnknownReason};
        use crate::types::family::FamilyOperationShape;
        use phalcom_common::selector::{Selector, SelectorSlot};
        use std::sync::Arc;

        let owner = DeclarationId::new(ModuleId::universe_root(), "FamilyApplicationOwner".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("apply").unwrap(), DispatchSide::Instance);
        let mut source = empty_callable_analysis(callable.clone());
        let mut store = crate::types::store::TypeStore::new();
        let family_type = store.nominal_type(owner);
        let result_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "FamilyResult".into()));
        let target = InvocationTargetId::behavioral(callable.clone());
        let selections = [
            FamilyApplicationSelection::Static {
                operation: FamilyOperationShape::getter(),
                target: Some(target.clone()),
                callable_type: family_type,
                result_type,
            },
            FamilyApplicationSelection::Static {
                operation: FamilyOperationShape::setter(),
                target: None,
                callable_type: family_type,
                result_type,
            },
            FamilyApplicationSelection::Dynamic {
                candidates: vec![
                    FamilyApplicationCandidate {
                        operation: FamilyOperationShape::method(vec![SelectorSlot::Positional]),
                        target: None,
                        callable_type: family_type,
                    },
                    FamilyApplicationCandidate {
                        operation: FamilyOperationShape::getter(),
                        target: Some(target),
                        callable_type: result_type,
                    },
                ]
                .into_boxed_slice(),
                result_type: Some(result_type),
            },
            FamilyApplicationSelection::Dynamic {
                candidates: Box::default(),
                result_type: None,
            },
        ];
        let mut families = crate::checker::associated::FamilyApplicationResolutionIndex::new();
        for (index, selection) in selections.into_iter().enumerate() {
            let id = ExpressionId::new(BodyId(71), LocalExpressionId((index * 3 + 5) as u32));
            source
                .expressions
                .insert_source(
                    id,
                    ExpressionAnalysis::ready(
                        id,
                        phalcom_common::range::SourceRange { start: index, end: index + 1 },
                        TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                    ),
                )
                .unwrap();
            families
                .insert_source(
                    id,
                    FamilyApplicationResolution {
                        family_type,
                        kind: if index % 2 == 0 {
                            FamilyApplicationKind::Associated
                        } else {
                            FamilyApplicationKind::Behavioral
                        },
                        selection,
                    },
                )
                .unwrap();
        }
        source.family_applications = Arc::new(families);
        let mut types = crate::types::store::artifact::TypeExportContext::new(&store);
        crate::artifact::roots::register_callable_analysis_roots(&mut types, &source, &store).unwrap();
        types.close_graph(&store, &ValidationLimits::default()).unwrap();
        let record = export_callable_analysis_record(&callable, &source, &BTreeMap::new(), &types, &ValidationLimits::default())
            .expect("checked family applications must have portable owner rows");
        let graph = types.project_graph(&store, &ValidationLimits::default()).unwrap();
        let (_fresh_store, relocation) = crate::types::store::TypeStore::import_type_graph(&graph, &ValidationLimits::default()).unwrap();
        let mut budget = phalcom_type_meta::allocation::AllocationBudget::new(1 << 20);
        let restored = crate::artifact::import::import_callable_family_applications(&record, BodyId(0), &relocation, &mut budget).unwrap();
        assert!(restored.is_frozen());
        assert_eq!(restored.len(), source.family_applications.len());
        for (index, ((id, restored), source)) in restored.iter().zip(source.family_applications.values()).enumerate() {
            assert_eq!(*id, ExpressionId::new(BodyId(0), LocalExpressionId(index as u32)));
            assert_eq!(restored.family_type, relocation.ty(types.ty(source.family_type).unwrap()).unwrap());
            assert_eq!(restored.kind, source.kind);
            match (&restored.selection, &source.selection) {
                (
                    FamilyApplicationSelection::Static {
                        operation,
                        target,
                        callable_type,
                        result_type,
                    },
                    FamilyApplicationSelection::Static {
                        operation: expected_operation,
                        target: expected_target,
                        callable_type: expected_callable,
                        result_type: expected_result,
                    },
                ) => {
                    assert_eq!(operation, expected_operation);
                    assert_eq!(target, expected_target);
                    assert_eq!(*callable_type, relocation.ty(types.ty(*expected_callable).unwrap()).unwrap());
                    assert_eq!(*result_type, relocation.ty(types.ty(*expected_result).unwrap()).unwrap());
                }
                (
                    FamilyApplicationSelection::Dynamic { candidates, result_type },
                    FamilyApplicationSelection::Dynamic {
                        candidates: expected_candidates,
                        result_type: expected_result,
                    },
                ) => {
                    assert_eq!(*result_type, expected_result.map(|ty| relocation.ty(types.ty(ty).unwrap()).unwrap()));
                    assert_eq!(candidates.len(), expected_candidates.len());
                    for (candidate, expected) in candidates.iter().zip(expected_candidates.iter()) {
                        assert_eq!(candidate.operation, expected.operation);
                        assert_eq!(candidate.target, expected.target);
                        assert_eq!(candidate.callable_type, relocation.ty(types.ty(expected.callable_type).unwrap()).unwrap());
                    }
                }
                _ => panic!("family application changed selection kind"),
            }
        }
        let requested = budget.requested();
        for capacity in [requested, requested - 1] {
            let mut limited = phalcom_type_meta::allocation::AllocationBudget::new(capacity);
            let result = crate::artifact::import::import_callable_family_applications(&record, BodyId(0), &relocation, &mut limited);
            assert_eq!(result.is_ok(), capacity == requested);
            assert_eq!(limited.requested(), if capacity == requested { requested } else { 0 });
        }
        let mut malformed = record;
        malformed.family_applications.last_mut().unwrap().expression = u32::MAX;
        let mut rejected = phalcom_type_meta::allocation::AllocationBudget::new(1 << 20);
        assert!(crate::artifact::import::import_callable_family_applications(&malformed, BodyId(0), &relocation, &mut rejected).is_err());
        assert_eq!(rejected.requested(), 0);
    }

    #[test]
    fn callable_analysis_record_preserves_positional_expansion_authority() {
        use crate::checker::analysis::{
            ExpressionAnalysis, ForIterationRequirementResolution, KnownInvalidPositionalExpansion, PositionalExpansionIterationResolution,
        };
        use crate::identity::{BodyId, CallableId, DeclarationId, DispatchSide, ExpressionId, LocalExpressionId, ModuleId};
        use crate::types::evidence::{TypeKnowledge, UnknownReason};
        use phalcom_common::range::SourceRange;
        use phalcom_common::selector::Selector;
        use std::sync::Arc;

        let owner = DeclarationId::new(ModuleId::universe_root(), "ExpansionOwner".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("expand").unwrap(), DispatchSide::Instance);
        let requirement = crate::traits::TraitRequirementId {
            owner: owner.clone(),
            selector: callable.selector.clone(),
            side: callable.side,
        };
        let signature = crate::signature::CallableSemanticSignature {
            callable: callable.clone(),
            owner: owner.clone(),
            side: callable.side,
            selector: callable.selector.clone(),
            generics: None,
            parameters: Box::default(),
            declared_return: crate::declaration_type::DeclaredTypeFact::unknown(UnknownReason::NoTypeEvidence),
            return_validation: crate::signature::ReturnContractValidation::NotApplicable,
            inferred_return: None,
            source: None,
            implementation: phalcom_common::modifier::ImplementationKind::Phalcom,
            native_id: None,
            effects: phalcom_native_meta::EffectSpec::Unknown,
            raises: phalcom_native_meta::RaisesSpec::Unknown,
            flow: phalcom_native_meta::ReturnFlowSpec::Value,
            lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
        };
        let mut source = empty_callable_analysis(callable.clone());
        let mut store = crate::types::store::TypeStore::new();
        let source_type = store.nominal_type(owner);
        let item_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "ExpansionItem".into()));
        let iterator_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "ExpansionIterator".into()));
        let conformance = crate::generic_evidence::GenericConformanceKey::new(
            crate::types::parameter::TypeTerm::Canonical(iterator_type),
            crate::traits::TraitRef::new(requirement.owner.clone(), Box::default()),
        );
        let evidence =
            crate::generic_evidence::GenericConformanceEvidence::CompilerDerived(Arc::new(crate::generic_evidence::CompilerDerivedConformanceEvidence {
                key: conformance.clone(),
                kind: crate::generic_evidence::CompilerDerivedConformanceKind::OrderedProduct,
                classification: crate::structural::StructuralClassification::OrderedProduct(crate::structural::OrderedProductForm::Tuple),
            }));
        let ids = [
            ExpressionId::new(BodyId(91), LocalExpressionId(7)),
            ExpressionId::new(BodyId(91), LocalExpressionId(13)),
        ];
        for (index, id) in ids.into_iter().enumerate() {
            source
                .expressions
                .insert_source(
                    id,
                    ExpressionAnalysis::ready(
                        id,
                        SourceRange { start: index, end: index + 1 },
                        TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                    ),
                )
                .unwrap();
        }
        let mut expansions = crate::checker::analysis::PositionalExpansionIterationResolutionIndex::new();
        expansions
            .insert_source(
                ids[0],
                PositionalExpansionIterationResolution {
                    source_expression: ids[0],
                    source_range: SourceRange { start: 1, end: 4 },
                    source_type,
                    item_type,
                    iterator_type,
                    iterator: ForIterationRequirementResolution::AbstractRequirement {
                        requirement: requirement.clone(),
                        signature: signature.clone(),
                    },
                    next: ForIterationRequirementResolution::Evidence {
                        conformance,
                        evidence,
                        requirement,
                        signature,
                    },
                },
            )
            .unwrap();
        source.positional_expansions = Arc::new(expansions);
        let mut invalid = crate::checker::analysis::KnownInvalidPositionalExpansionIndex::new();
        invalid
            .insert_source(
                ids[1],
                KnownInvalidPositionalExpansion {
                    source_expression: ids[1],
                    source_range: SourceRange { start: 4, end: 7 },
                    source_type,
                },
            )
            .unwrap();
        source.known_invalid_positional_expansions = Arc::new(invalid);
        let mut types = crate::types::store::artifact::TypeExportContext::new(&store);
        crate::artifact::roots::register_callable_analysis_roots(&mut types, &source, &store).unwrap();
        types.close_graph(&store, &ValidationLimits::default()).unwrap();
        let mut record = export_callable_analysis_record(&callable, &source, &BTreeMap::new(), &types, &ValidationLimits::default())
            .expect("both executable and known-invalid expansion authority must be transported");
        let limits = ValidationLimits::default();
        let graph = types.project_graph(&store, &limits).unwrap();
        let (fresh_store, relocation) = crate::types::store::TypeStore::import_type_graph(&graph, &limits).unwrap();
        let mut budget = phalcom_type_meta::allocation::AllocationBudget::new(1 << 20);
        let restored = crate::artifact::import::import_callable_resolution_authority(
            &record,
            BodyId(0),
            &relocation,
            &fresh_store,
            &BTreeMap::new(),
            &limits,
            &mut budget,
        )
        .unwrap();
        assert!(restored.positional_expansions.is_frozen());
        assert!(restored.known_invalid_positional_expansions.is_frozen());
        let expansion = restored.positional_expansions.get(&ExpressionId::new(BodyId(0), LocalExpressionId(0))).unwrap();
        assert_eq!(expansion.source_expression, ExpressionId::new(BodyId(0), LocalExpressionId(0)));
        assert_eq!(expansion.source_range, SourceRange { start: 1, end: 4 });
        assert_eq!(expansion.source_type, relocation.ty(types.ty(source_type).unwrap()).unwrap());
        assert_eq!(expansion.item_type, relocation.ty(types.ty(item_type).unwrap()).unwrap());
        assert_eq!(expansion.iterator_type, relocation.ty(types.ty(iterator_type).unwrap()).unwrap());
        let expected = source.positional_expansions.get(&ids[0]).unwrap();
        assert_eq!(expansion.iterator, expected.iterator);
        let ForIterationRequirementResolution::Evidence {
            conformance,
            evidence,
            requirement,
            signature,
        } = &expansion.next
        else {
            panic!("expansion lost checked conformance evidence")
        };
        assert_eq!(conformance.subject, crate::types::parameter::TypeTerm::Canonical(expansion.iterator_type));
        assert_eq!(conformance.trait_ref.declaration, requirement.owner);
        assert_eq!(signature.callable, callable);
        let crate::generic_evidence::GenericConformanceEvidence::CompilerDerived(evidence) = evidence else {
            panic!("expansion evidence changed kind")
        };
        assert_eq!(&evidence.key, conformance);
        let invalid = restored
            .known_invalid_positional_expansions
            .get(&ExpressionId::new(BodyId(0), LocalExpressionId(1)))
            .unwrap();
        assert_eq!(invalid.source_range, SourceRange { start: 4, end: 7 });
        assert_eq!(invalid.source_type, expansion.source_type);
        let requested = budget.requested();
        for capacity in [requested, requested - 1] {
            let mut limited = phalcom_type_meta::allocation::AllocationBudget::new(capacity);
            let result = crate::artifact::import::import_callable_resolution_authority(
                &record,
                BodyId(0),
                &relocation,
                &fresh_store,
                &BTreeMap::new(),
                &limits,
                &mut limited,
            );
            assert_eq!(result.is_ok(), capacity == requested);
            assert_eq!(limited.requested(), if capacity == requested { requested } else { 0 });
        }
        record.known_invalid_positional_expansions[0].expression = u32::MAX;
        let mut rejected = phalcom_type_meta::allocation::AllocationBudget::new(1 << 20);
        assert!(
            crate::artifact::import::import_callable_resolution_authority(
                &record,
                BodyId(0),
                &relocation,
                &fresh_store,
                &BTreeMap::new(),
                &limits,
                &mut rejected,
            )
            .is_err()
        );
        assert_eq!(rejected.requested(), 0);
    }

    #[test]
    fn callable_analysis_record_preserves_expression_dispatch_authority() {
        use crate::checker::analysis::{ExpressionAnalysis, GenericCallConformanceBinding, GenericCallConformanceSource};
        use crate::generic_evidence::{CompilerDerivedConformanceEvidence, CompilerDerivedConformanceKind, GenericConformanceEvidence, GenericConformanceKey};
        use crate::identity::{BodyId, CallableId, DeclarationId, DispatchSide, ExpressionId, ImplId, ImplLocalId, LocalExpressionId, ModuleId};
        use crate::trait_dispatch::{TraitDispatchSelection, TraitDispatchSite};
        use crate::types::evidence::{TypeKnowledge, UnknownReason};
        use crate::types::parameter::TypeTerm;
        use crate::types::store::TypeStore;
        use crate::types::store::artifact::TypeExportContext;
        use phalcom_common::range::SourceRange;
        use phalcom_common::selector::Selector;

        let owner = DeclarationId::new(ModuleId::universe_root(), "DispatchOwner".into());
        let trait_owner = DeclarationId::new(ModuleId::universe_root(), "DispatchTrait".into());
        let callable = CallableId::method(owner.clone(), Selector::getter("dispatch").unwrap(), DispatchSide::Class);
        let requirement = crate::traits::TraitRequirementId {
            owner: trait_owner.clone(),
            selector: callable.selector.clone(),
            side: DispatchSide::Class,
        };
        let expression_id = ExpressionId::new(BodyId(12), LocalExpressionId(4));
        let mut source = empty_callable_analysis(callable.clone());
        source
            .expressions
            .insert_source(
                expression_id,
                ExpressionAnalysis::ready(
                    expression_id,
                    SourceRange { start: 5, end: 8 },
                    TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                ),
            )
            .unwrap();

        let mut store = TypeStore::new();
        let receiver = store.nominal_type(DeclarationId::new(owner.module.clone(), "Receiver".into()));
        let owner_view = store.nominal_type(owner.clone());
        let trait_argument = store.nominal_type(DeclarationId::new(owner.module.clone(), "TraitArgument".into()));
        let result_type = store.nominal_type(DeclarationId::new(owner.module.clone(), "Result".into()));
        let trait_ref = crate::traits::TraitRef::new(trait_owner, vec![trait_argument].into_boxed_slice());
        let evidence_key = GenericConformanceKey::new(TypeTerm::Canonical(receiver), trait_ref.clone());
        let evidence = GenericConformanceEvidence::CompilerDerived(std::sync::Arc::new(CompilerDerivedConformanceEvidence {
            key: evidence_key.clone(),
            kind: CompilerDerivedConformanceKind::OrderedProduct,
            classification: crate::structural::StructuralClassification::OrderedProduct(crate::structural::OrderedProductForm::Tuple),
        }));
        let impl_id = ImplId::new(owner.module.clone(), ImplLocalId(1));
        let signature = crate::signature::CallableSemanticSignature {
            callable: callable.clone(),
            owner: owner.clone(),
            side: DispatchSide::Class,
            selector: callable.selector.clone(),
            generics: None,
            parameters: Box::default(),
            declared_return: crate::declaration_type::DeclaredTypeFact::known(
                TypeTerm::Canonical(result_type),
                crate::declaration_type::DeclaredTypeBasis::SourceAnnotation,
            ),
            return_validation: crate::signature::ReturnContractValidation::NotApplicable,
            inferred_return: None,
            source: None,
            implementation: phalcom_common::modifier::ImplementationKind::Phalcom,
            native_id: None,
            effects: phalcom_native_meta::EffectSpec::Unknown,
            raises: phalcom_native_meta::RaisesSpec::Unknown,
            flow: phalcom_native_meta::ReturnFlowSpec::Value,
            lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
        };
        let selection = crate::dispatch::ConditionalDispatchSelection {
            impl_id: impl_id.clone(),
            callable: callable.clone(),
            declaring_owner: owner.clone(),
            side: DispatchSide::Class,
        };

        let mut enriched = source.clone();
        source.expressions.get_source_mut(&expression_id).unwrap().unwrap().conditional_dispatch = Some(selection.clone());
        let expression = enriched.expressions.get_source_mut(&expression_id).unwrap().unwrap();
        expression.conditional_dispatch = Some(selection.clone());
        expression.conditional_capability = Some(crate::dispatch::ConditionalSelectedCallCapability {
            selection,
            applicability: crate::impls::InherentImplSpecialization {
                impl_id: impl_id.clone(),
                receiver,
                owner_view,
                bindings: Default::default(),
                environment: Default::default(),
                conformance_provenance: Some(
                    vec![GenericCallConformanceBinding {
                        callee_conformance: evidence_key,
                        source: GenericCallConformanceSource::CompilerDerived {
                            evidence: match &evidence {
                                GenericConformanceEvidence::CompilerDerived(evidence) => evidence.clone(),
                                _ => unreachable!(),
                            },
                        },
                    }]
                    .into_boxed_slice(),
                ),
            },
        });
        expression.trait_dispatch = Some(TraitDispatchSite::Evidenced(TraitDispatchSelection {
            source_impl: impl_id,
            exact_target: receiver,
            exact_trait_ref: trait_ref,
            requirement: requirement.clone(),
            callable: Some(callable.clone()),
            signature,
            evidence_fingerprint: crate::db::fingerprint::generic_conformance_evidence_fingerprint(&evidence),
            evidence,
            selection: crate::impls::RequirementSelectionTemplate::ConformanceCallable {
                callable: callable.clone(),
            },
            requirement_targets: Default::default(),
        }));

        let mut types = TypeExportContext::new(&store);
        crate::artifact::roots::register_callable_analysis_roots(&mut types, &enriched, &store).unwrap();
        types.close_graph(&store, &ValidationLimits::default()).unwrap();
        let source_record = export_callable_analysis_record(&callable, &source, &BTreeMap::new(), &types, &ValidationLimits::default()).unwrap();
        let enriched_record = export_callable_analysis_record(&callable, &enriched, &BTreeMap::new(), &types, &ValidationLimits::default()).unwrap();

        assert_ne!(
            source_record, enriched_record,
            "conditional execution capability and trait dispatch are owned expression authority"
        );

        let graph = types.project_graph(&store, &ValidationLimits::default()).unwrap();
        let (imported_store, relocation) = TypeStore::import_type_graph(&graph, &ValidationLimits::default()).unwrap();
        let mut allocations = phalcom_type_meta::allocation::AllocationBudget::new(1024 * 1024);
        let imported = crate::artifact::import::import_callable_expression_dispatch_authority(
            &enriched_record,
            crate::identity::BodyId(0),
            &relocation,
            &imported_store,
            &BTreeMap::new(),
            &mut allocations,
            ValidationLimits::default().max_recursion_depth,
        )
        .unwrap();
        let imported = &imported[0];
        let capability = imported.conditional_capability.as_ref().expect("conditional capability");
        assert_eq!(capability.selection.impl_id, capability.applicability.impl_id);
        assert_eq!(capability.applicability.receiver, relocation.ty(types.ty(receiver).unwrap()).unwrap());
        let crate::trait_dispatch::TraitDispatchSite::Evidenced(selection) = imported.trait_dispatch.as_ref().expect("trait dispatch") else {
            panic!("expected evidenced trait dispatch")
        };
        assert_eq!(selection.requirement, requirement);
        assert_eq!(
            selection.evidence_fingerprint,
            crate::db::fingerprint::generic_conformance_evidence_fingerprint(&selection.evidence)
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
