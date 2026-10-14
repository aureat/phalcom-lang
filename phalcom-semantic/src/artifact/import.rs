//! Semantic-owned artifact import for structural primary contracts (`PERF003.C2.P7.T7.A`).

use crate::data_semantics::{
    DataComponentSemantic, DataConstructorParameter, DataConstructorParameterContribution,
    DataConstructorSignature, DataConstructorTemplate, DataInfo, DataProductContribution, DataProductExpansionSemantic,
    DataProductShapeTemplate, DataSemanticTable, DataShape,
};
use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact, DeclaredTypeState};
use crate::declarations::{DeclarationTypeBinding, DeclarationTypeInfo, DeclarationTypeTable};
use crate::diagnostic::SemanticSourceSpan;
use crate::enum_semantics::{
    AlgebraicCasePayloadKind, EnumInfo, EnumSemanticTable, VariantConstructorParameter, VariantConstructorSignature,
    VariantFieldSemantic, VariantInfo, VariantShape, VariantVisibility,
};
use crate::identity::{DataComponentId, DataConstructorId, VariantConstructorId, VariantFieldId, VariantId};
use crate::metadata::stable_identity::{
    from_stable_callable, from_stable_declaration, from_stable_dispatch_side, from_stable_impl, from_stable_variant,
};
use crate::type_alias::{TypeAliasInfo, TypeAliasTable};
use crate::types::case_environment::CaseTypeEnvironment;
use crate::types::evidence::{DynamicReason, UnknownReason};
use crate::types::parameter::{GenericBinderHeader, GenericConstraint, GenericParameterMode, GenericSignature, SelfRole, SelfTypeTerm, TypeParameterOwner, TypeTerm};
use crate::types::relation::MapTypeHierarchy;
use crate::types::store::artifact::TypeRelocation;
use crate::types::store::TypeStore;
use crate::types::variance::Variance;
use phalcom_common::modifier::Visibility;
use phalcom_compiler_artifact::semantic::aggregates::*;
use phalcom_compiler_artifact::semantic::type_store::{
    ParameterModeV1, ParameterOwnerV1, SelfRoleV1, VarianceV1,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

fn referential(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}

pub fn import_parameter_owner(owner: &ParameterOwnerV1) -> Result<TypeParameterOwner, ArtifactError> {
    Ok(match owner {
        ParameterOwnerV1::Declaration { owner } => {
            TypeParameterOwner::Declaration(from_stable_declaration(owner).map_err(referential)?)
        }
        ParameterOwnerV1::Callable { owner } => {
            TypeParameterOwner::Callable(from_stable_callable(owner).map_err(referential)?)
        }
        ParameterOwnerV1::Impl { owner } => {
            TypeParameterOwner::Impl(from_stable_impl(owner).map_err(referential)?)
        }
    })
}

pub fn import_variance(variance: &VarianceV1) -> Variance {
    match variance {
        VarianceV1::Covariant => Variance::Covariant,
        VarianceV1::Contravariant => Variance::Contravariant,
        VarianceV1::Invariant => Variance::Invariant,
    }
}

pub fn import_mode(mode: &ParameterModeV1) -> GenericParameterMode {
    match mode {
        ParameterModeV1::Single => GenericParameterMode::Single,
        ParameterModeV1::PositionalPack => GenericParameterMode::PositionalPack,
        ParameterModeV1::LabeledPack => GenericParameterMode::LabeledPack,
        ParameterModeV1::CompletePack => GenericParameterMode::CompletePack,
    }
}

pub fn import_self_role(role: &SelfRoleV1) -> SelfRole {
    match role {
        SelfRoleV1::InstanceType => SelfRole::InstanceType,
        SelfRoleV1::ReceiverValue => SelfRole::ReceiverValue,
    }
}

pub fn import_type_term(term: &TypeTermV1, relocation: &TypeRelocation) -> Result<TypeTerm, ArtifactError> {
    Ok(match term {
        TypeTermV1::Canonical { ty } => TypeTerm::Canonical(relocation.ty(*ty)?),
        TypeTermV1::SelfType { term } => TypeTerm::SelfType(SelfTypeTerm {
            owner: from_stable_declaration(&term.owner).map_err(referential)?,
            side: from_stable_dispatch_side(term.side),
            role: import_self_role(&term.role),
        }),
    })
}

pub fn import_constraint(
    constraint: &GenericConstraintV1,
    relocation: &TypeRelocation,
) -> Result<GenericConstraint, ArtifactError> {
    Ok(match constraint {
        GenericConstraintV1::Subtype { lower, upper } => GenericConstraint::Subtype {
            lower: import_type_term(lower, relocation)?,
            upper: import_type_term(upper, relocation)?,
        },
        GenericConstraintV1::Equivalent { left, right } => GenericConstraint::Equivalent {
            left: import_type_term(left, relocation)?,
            right: import_type_term(right, relocation)?,
        },
        GenericConstraintV1::Conforms { subject, trait_ref } => GenericConstraint::Conforms {
            subject: import_type_term(subject, relocation)?,
            trait_ref: crate::traits::TraitRef {
                declaration: from_stable_declaration(&trait_ref.declaration).map_err(referential)?,
                arguments: trait_ref
                    .arguments
                    .iter()
                    .map(|&arg| relocation.ty(arg))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
            },
        },
    })
}

pub fn import_generic_signature(
    sig: &GenericSignatureV1,
    relocation: &TypeRelocation,
    store: &TypeStore,
) -> Result<GenericSignature, ArtifactError> {
    let owner = import_parameter_owner(&sig.owner)?;
    let parameters = sig
        .parameters
        .iter()
        .map(|&p| relocation.parameter(p))
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    let parameter_kinds = sig
        .parameter_kinds
        .iter()
        .map(|&k| relocation.kind(k))
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    let parameter_kind_shapes = sig.parameter_kind_shapes.to_vec().into_boxed_slice();
    let parameter_variances = sig.parameter_variances.iter().map(import_variance).collect::<Vec<_>>().into_boxed_slice();
    let parameter_modes = sig.parameter_modes.iter().map(import_mode).collect::<Vec<_>>().into_boxed_slice();
    let constraint_shapes = sig.constraint_shapes.to_vec().into_boxed_slice();
    let constraints = sig
        .constraints
        .iter()
        .map(|c| import_constraint(c, relocation))
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();

    let reconstructed = GenericSignature {
        owner,
        parameters,
        parameter_kinds,
        parameter_kind_shapes,
        parameter_variances,
        parameter_modes,
        constraint_shapes,
        constraints,
    };
    reconstructed.validate_publishable(store).map_err(|e| referential(format!("{e:?}")))?;
    Ok(reconstructed)
}

pub fn import_declared_type_state(
    state: &DeclaredTypeStateV1,
    relocation: &TypeRelocation,
) -> Result<DeclaredTypeState, ArtifactError> {
    Ok(match state {
        DeclaredTypeStateV1::Known { term } => DeclaredTypeState::Known(import_type_term(term, relocation)?),
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
    })
}

pub fn import_declared_type_basis(basis: &DeclaredTypeBasisV1) -> DeclaredTypeBasis {
    match basis {
        DeclaredTypeBasisV1::Unspecified => DeclaredTypeBasis::Unspecified,
        DeclaredTypeBasisV1::SourceAnnotation => DeclaredTypeBasis::SourceAnnotation,
        DeclaredTypeBasisV1::NativeSignature => DeclaredTypeBasis::NativeSignature,
        DeclaredTypeBasisV1::DeclarationSemantics => DeclaredTypeBasis::DeclarationSemantics,
        DeclaredTypeBasisV1::ConstructorSemantics => DeclaredTypeBasis::ConstructorSemantics,
        DeclaredTypeBasisV1::InitializerInference => DeclaredTypeBasis::InitializerInference,
        DeclaredTypeBasisV1::BodyInference => DeclaredTypeBasis::BodyInference,
        DeclaredTypeBasisV1::ContextualTyping => DeclaredTypeBasis::ContextualTyping,
        DeclaredTypeBasisV1::PatternDecomposition => DeclaredTypeBasis::PatternDecomposition,
    }
}

pub fn import_declared_type_fact(
    fact: &DeclaredTypeFactV1,
    relocation: &TypeRelocation,
) -> Result<DeclaredTypeFact, ArtifactError> {
    Ok(DeclaredTypeFact {
        state: import_declared_type_state(&fact.state, relocation)?,
        basis: import_declared_type_basis(&fact.basis),
    })
}

pub fn import_visibility(visibility: &VisibilityV1) -> Visibility {
    match visibility {
        VisibilityV1::Public => Visibility::Public,
        VisibilityV1::Private => Visibility::Private,
        VisibilityV1::Protected => Visibility::Protected,
        VisibilityV1::Module => Visibility::Module,
        VisibilityV1::Package => Visibility::Package,
        VisibilityV1::Internal => Visibility::Internal,
    }
}

pub fn import_data_shape(shape: &DataShapeV1) -> DataShape {
    match shape {
        DataShapeV1::Tuple => DataShape::Tuple,
        DataShapeV1::Record => DataShape::Record,
    }
}

pub fn import_variant_shape(shape: &VariantShapeV1) -> VariantShape {
    match shape {
        VariantShapeV1::Singleton => VariantShape::Singleton,
        VariantShapeV1::Constructor => VariantShape::Constructor,
    }
}

pub fn import_payload_kind(kind: &PayloadKindV1) -> AlgebraicCasePayloadKind {
    match kind {
        PayloadKindV1::Singleton => AlgebraicCasePayloadKind::Singleton,
        PayloadKindV1::Tuple => AlgebraicCasePayloadKind::Tuple,
        PayloadKindV1::Record => AlgebraicCasePayloadKind::Record,
        PayloadKindV1::ExplicitEmpty => AlgebraicCasePayloadKind::ExplicitEmpty,
    }
}

pub fn import_expansion_mode(mode: &ExpansionModeV1) -> phalcom_ast::ast::ExpansionMode {
    match mode {
        ExpansionModeV1::Positional => phalcom_ast::ast::ExpansionMode::Positional,
        ExpansionModeV1::Labeled => phalcom_ast::ast::ExpansionMode::Labeled,
        ExpansionModeV1::Complete => phalcom_ast::ast::ExpansionMode::Complete,
    }
}

pub fn import_static_data(
    wire: &StaticAlgebraicDeclarationV1,
    relocation: &TypeRelocation,
) -> Result<crate::algebraic_semantics::StaticAlgebraicDeclarationInfo, ArtifactError> {
    Ok(crate::algebraic_semantics::StaticAlgebraicDeclarationInfo {
        domain: relocation.domain(wire.domain)?,
        constructors: wire
            .constructors
            .iter()
            .map(|&c| relocation.constructor(c))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
    })
}

pub fn import_case_environment(
    env: &CaseEnvironmentV1,
    relocation: &TypeRelocation,
) -> Result<CaseTypeEnvironment, ArtifactError> {
    let mut bindings = BTreeMap::new();
    for b in &env.bindings {
        let param = relocation.parameter(b.parameter)?;
        let ty = relocation.ty(b.ty)?;
        bindings.insert(param, ty);
    }
    let equalities = env
        .equalities
        .iter()
        .map(|eq| import_constraint(eq, relocation))
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();

    Ok(CaseTypeEnvironment { bindings, equalities })
}

/// Loaded authority populated from structural primary contracts.
#[derive(Clone, Debug)]
pub struct LoadedStructuralContracts {
    pub declaration_table: DeclarationTypeTable,
    pub hierarchy: MapTypeHierarchy,
    pub type_aliases: TypeAliasTable,
    pub data_table: DataSemanticTable,
    pub enum_table: EnumSemanticTable,
    pub generic_signatures: BTreeMap<TypeParameterOwner, GenericSignature>,
}

/// Reconstructs all structural primary contracts into a fresh set of semantic tables.
pub fn import_structural_primary_contracts(
    contracts: &StructuralPrimaryContractsV1,
    store: &mut TypeStore,
    type_relocation: &TypeRelocation,
    limits: &ValidationLimits,
) -> Result<LoadedStructuralContracts, ArtifactError> {
    contracts.validate(limits)?;
    type_relocation.verify_store(store)?;

    let mut generic_signatures = BTreeMap::new();
    for sig_wire in &contracts.generic_signatures {
        let sig = import_generic_signature(sig_wire, type_relocation, store)?;
        generic_signatures.insert(sig.owner.clone(), sig);
    }

    let mut data_table = DataSemanticTable::new();
    for data_wire in &contracts.data_contracts {
        let owner = from_stable_declaration(&data_wire.owner).map_err(referential)?;
        let root_form = type_relocation.ty(data_wire.root_form)?;
        let generic_signature = data_wire
            .generic_signature
            .as_ref()
            .map(|s| import_generic_signature(s, type_relocation, store))
            .transpose()?;
        let static_data = data_wire
            .static_data
            .as_ref()
            .map(|s| import_static_data(s, type_relocation))
            .transpose()?;
        let shape = import_data_shape(&data_wire.shape);

        let mut components = Vec::with_capacity(data_wire.components.len());
        for (idx, c) in data_wire.components.iter().enumerate() {
            let id = DataComponentId::new(owner.clone(), idx as u32);
            let declared_type = import_declared_type_fact(&c.declared_type, type_relocation)?;
            components.push(DataComponentSemantic {
                id,
                local_name: c.local_name.clone(),
                external_label: c.external_label.clone(),
                declared_type,
                source: None,
            });
        }

        let constructor_id = DataConstructorId::new(owner.clone());
        let constructor_visibility = import_visibility(&data_wire.constructor_visibility);
        let mut constructor_parameters = Vec::with_capacity(data_wire.constructor_parameters.len());
        for (idx, p) in data_wire.constructor_parameters.iter().enumerate() {
            let component = DataComponentId::new(owner.clone(), idx as u32);
            let declared_type = import_declared_type_fact(&p.declared_type, type_relocation)?;
            constructor_parameters.push(DataConstructorParameter {
                component,
                external_label: p.external_label.clone(),
                local_name: p.local_name.clone(),
                declared_type,
            });
        }
        let constructor_result_type = type_relocation.ty(data_wire.constructor_result_type)?;
        let constructor = DataConstructorSignature {
            constructor: constructor_id.clone(),
            visibility: constructor_visibility,
            parameters: constructor_parameters.into_boxed_slice(),
            result_type_template: constructor_result_type,
            source: None,
        };

        let mut shape_contributions = Vec::with_capacity(data_wire.shape_contributions.len());
        let mut constructor_param_contributions = Vec::with_capacity(data_wire.shape_contributions.len());
        let mut component_iter = components.iter();

        for contrib in &data_wire.shape_contributions {
            match contrib {
                DataProductContributionV1::Component { .. } => {
                    let comp = component_iter.next().ok_or_else(|| referential("shape contribution component index error"))?;
                    shape_contributions.push(DataProductContribution::Component(comp.clone()));
                    constructor_param_contributions.push(DataConstructorParameterContribution::Parameter(DataConstructorParameter {
                        component: comp.id.clone(),
                        external_label: comp.external_label.clone(),
                        local_name: comp.local_name.clone(),
                        declared_type: comp.declared_type.clone(),
                    }));
                }
                DataProductContributionV1::Expansion { mode, operand } => {
                    let operand = import_declared_type_fact(operand, type_relocation)?;
                    let exp = DataProductExpansionSemantic {
                        mode: import_expansion_mode(mode),
                        operand,
                        source: None,
                    };
                    shape_contributions.push(DataProductContribution::Expansion(exp.clone()));
                    constructor_param_contributions.push(DataConstructorParameterContribution::Expansion(exp));
                }
            }
        }

        let shape_template = DataProductShapeTemplate {
            shape,
            contributions: shape_contributions.into_boxed_slice(),
        };

        let constructor_template = if shape_template.has_expansions() {
            Some(DataConstructorTemplate {
                constructor: constructor_id,
                visibility: constructor_visibility,
                parameters: constructor_param_contributions.into_boxed_slice(),
                result_type_template: constructor_result_type,
                source: None,
            })
        } else {
            None
        };

        let data_info = DataInfo {
            owner,
            root_form,
            generic_signature,
            static_data,
            shape,
            components: components.into_boxed_slice(),
            constructor,
            shape_template,
            constructor_template,
            source: None,
        };
        data_table.insert_data(Arc::new(data_info));
    }

    let mut enum_table = EnumSemanticTable::new();
    let mut loaded_variants: BTreeMap<VariantId, Arc<VariantInfo>> = BTreeMap::new();

    for variant_wire in &contracts.variant_contracts {
        let variant_id = from_stable_variant(&variant_wire.variant).map_err(referential)?;
        let type_handle = type_relocation.variant(variant_wire.variant_type)?;
        if store.variant_identity(type_handle) != &variant_id {
            return Err(referential("relocated variant type handle does not match variant identity in TypeStore"));
        }

        let family = variant_id.family();
        let shape = import_variant_shape(&variant_wire.shape);
        let payload_kind = import_payload_kind(&variant_wire.payload_kind);

        let mut fields = Vec::with_capacity(variant_wire.fields.len());
        for (idx, f) in variant_wire.fields.iter().enumerate() {
            let id = VariantFieldId::new(variant_id.clone(), idx as u32);
            let declared_type = import_declared_type_fact(&f.declared_type, type_relocation)?;
            fields.push(VariantFieldSemantic {
                id,
                local_name: f.local_name.clone(),
                external_label: f.external_label.clone(),
                declared_type,
                source: None,
            });
        }

        let result_type_template = type_relocation.ty(variant_wire.result_type)?;
        let exact_case_template = type_relocation.ty(variant_wire.exact_case_type)?;
        let case_environment = import_case_environment(&variant_wire.case_environment, type_relocation)?;

        let constructor = if let Some(cw) = &variant_wire.constructor {
            let generic_signature = cw
                .generic_signature
                .as_ref()
                .map(|s| import_generic_signature(s, type_relocation, store))
                .transpose()?;
            let mut parameters = Vec::with_capacity(cw.parameters.len());
            for (idx, p) in cw.parameters.iter().enumerate() {
                let field = VariantFieldId::new(variant_id.clone(), idx as u32);
                let declared_type = import_declared_type_fact(&p.declared_type, type_relocation)?;
                parameters.push(VariantConstructorParameter {
                    field,
                    external_label: p.external_label.clone(),
                    local_name: p.local_name.clone(),
                    declared_type,
                });
            }
            Some(VariantConstructorSignature {
                constructor: VariantConstructorId::new(variant_id.clone()),
                generic_signature,
                parameters: parameters.into_boxed_slice(),
                result_type_template: type_relocation.ty(cw.result_type)?,
                exact_case_template: type_relocation.ty(cw.exact_case_type)?,
                source: None,
            })
        } else {
            None
        };

        let visibility = VariantVisibility {
            name: import_visibility(&variant_wire.visibility.name),
            construct: import_visibility(&variant_wire.visibility.construct),
            payload: import_visibility(&variant_wire.visibility.payload),
        };

        let variant_info = Arc::new(VariantInfo {
            id: variant_id.clone(),
            type_handle,
            family,
            shape,
            payload_kind,
            fields: fields.into_boxed_slice(),
            result_type_template,
            exact_case_template,
            case_environment,
            constructor,
            visibility,
            source: None,
        });
        enum_table.insert_variant(variant_info.clone());
        loaded_variants.insert(variant_id, variant_info);
    }

    for enum_wire in &contracts.enum_contracts {
        let owner = from_stable_declaration(&enum_wire.owner).map_err(referential)?;
        let root_form = type_relocation.ty(enum_wire.root_form)?;
        let generic_signature = enum_wire
            .generic_signature
            .as_ref()
            .map(|s| import_generic_signature(s, type_relocation, store))
            .transpose()?;
        let static_data = enum_wire
            .static_data
            .as_ref()
            .map(|s| import_static_data(s, type_relocation))
            .transpose()?;
        let default_result_type = type_relocation.ty(enum_wire.default_result_type)?;

        let mut variant_ids = Vec::with_capacity(enum_wire.variants.len());
        let mut variant_families = Vec::new();
        let mut seen_families = BTreeSet::new();

        for v_stable in &enum_wire.variants {
            let v_id = from_stable_variant(v_stable).map_err(referential)?;
            if !loaded_variants.contains_key(&v_id) {
                return Err(referential(format!("missing variant {:?} referenced by enum {:?}", v_id, owner)));
            }
            if let Some(fam) = v_id.family() {
                if seen_families.insert(fam.clone()) {
                    variant_families.push(fam);
                }
            }
            variant_ids.push(v_id);
        }

        let enum_info = Arc::new(EnumInfo {
            owner,
            root_form,
            generic_signature,
            static_data,
            default_result_type,
            variants: variant_ids.into_boxed_slice(),
            variant_families: variant_families.into_boxed_slice(),
            source: None,
        });
        enum_table.insert_enum(enum_info);
    }

    let mut hierarchy = MapTypeHierarchy::new();
    let mut type_aliases = TypeAliasTable::new();

    for entry in &contracts.hierarchy_and_aliases {
        let decl = from_stable_declaration(&entry.declaration).map_err(referential)?;
        if let Some(superclass_ref) = &entry.superclass {
            let super_decl = from_stable_declaration(superclass_ref).map_err(referential)?;
            hierarchy.insert(decl.clone(), super_decl);
        }
        if let Some(alias_wire) = entry.aliased_type {
            let form = type_relocation.ty(alias_wire)?;
            let kind = store.kind_of(form);
            let kind_shape = format!("{kind:?}").into_boxed_str();
            let structural_form = format!("{form:?}").into_boxed_str();
            let alias_info = TypeAliasInfo {
                declaration: decl.clone(),
                kind,
                kind_shape,
                generic_signature: generic_signatures.get(&TypeParameterOwner::Declaration(decl.clone())).cloned(),
                form,
                structural_form,
                dependencies: Box::new([]),
                source: SemanticSourceSpan::new(decl.module.clone(), phalcom_common::range::SourceRange::default()),
            };
            type_aliases.insert(alias_info);
        }
    }

    let mut declaration_table = DeclarationTypeTable::new();
    for shell in &contracts.declaration_shells {
        let decl = from_stable_declaration(&shell.declaration).map_err(referential)?;
        let form = type_relocation.ty(shell.self_type)?;
        let class_object_type = store.class_object_type(decl.clone());
        let kind = store.kind_of(form);
        let generic_sig = generic_signatures.get(&TypeParameterOwner::Declaration(decl.clone())).cloned();
        let generic_binder_header = generic_sig.as_ref().map(|s| GenericBinderHeader::from_signature(store, s));
        let supertype_template = hierarchy.supertype_template(&decl).cloned();

        let info = DeclarationTypeInfo {
            declaration: decl,
            form,
            class_object_type,
            kind,
            binding: DeclarationTypeBinding::ordinary(),
            generic_binder_header,
            generic_signature: generic_sig,
            supertype_template,
        };
        declaration_table.insert(info);
    }

    Ok(LoadedStructuralContracts {
        declaration_table,
        hierarchy,
        type_aliases,
        data_table,
        enum_table,
        generic_signatures,
    })
}