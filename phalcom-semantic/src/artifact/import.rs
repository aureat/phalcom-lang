//! Semantic-owned artifact import for structural primary contracts (`PERF003.C2.P7.T7.A`).

use crate::data_semantics::{
    DataComponentSemantic, DataConstructorParameter, DataConstructorParameterContribution, DataConstructorSignature, DataConstructorTemplate, DataInfo,
    DataProductContribution, DataProductExpansionSemantic, DataProductShapeTemplate, DataSemanticTable, DataShape,
};
use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact, DeclaredTypeState};
use crate::declarations::{DeclarationTypeBinding, DeclarationTypeInfo, DeclarationTypeTable};
use crate::diagnostic::SemanticSourceSpan;
use crate::enum_requirements::{CaseRequirementResult, CaseRequirementStatus, EnumRequirement, EnumRequirementId, EnumRequirementRows, EnumRequirementTable};
use crate::enum_semantics::{
    AlgebraicCasePayloadKind, EnumInfo, EnumSemanticTable, VariantConstructorParameter, VariantConstructorSignature, VariantFieldSemantic, VariantInfo,
    VariantShape, VariantVisibility,
};
use crate::identity::{
    CallableId, CallableOwnerId, DataComponentId, DataConstructorId, DeclarationId, DispatchSide, VariantConstructorId, VariantFieldId, VariantId,
};
use crate::metadata::stable_identity::{
    from_stable_callable, from_stable_declaration, from_stable_dispatch_side, from_stable_impl, from_stable_module, from_stable_variant,
};
use crate::type_alias::{TypeAliasInfo, TypeAliasTable};
use crate::types::case_environment::CaseTypeEnvironment;
use crate::types::evidence::{DynamicReason, UnknownReason};
use crate::types::parameter::{
    GenericBinderHeader, GenericConstraint, GenericParameterMode, GenericSignature, SelfRole, SelfTypeTerm, TypeParameterOwner, TypeTerm,
};
use crate::types::relation::MapTypeHierarchy;
use crate::types::store::TypeStore;
use crate::types::store::artifact::TypeRelocation;
use crate::types::variance::Variance;
use phalcom_common::modifier::Visibility;
use phalcom_common::range::SourceRange;
use phalcom_compiler_artifact::semantic::aggregates::*;
use phalcom_compiler_artifact::semantic::type_store::{ParameterModeV1, ParameterOwnerV1, SelfRoleV1, VarianceV1};
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::identity::{StableCallableOwnerRef, StableCallableRef};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

fn referential(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}

pub fn import_parameter_owner(owner: &ParameterOwnerV1) -> Result<TypeParameterOwner, ArtifactError> {
    Ok(match owner {
        ParameterOwnerV1::Declaration { owner } => TypeParameterOwner::Declaration(from_stable_declaration(owner).map_err(referential)?),
        ParameterOwnerV1::Callable { owner } => TypeParameterOwner::Callable(from_stable_callable(owner).map_err(referential)?),
        ParameterOwnerV1::Impl { owner } => TypeParameterOwner::Impl(from_stable_impl(owner).map_err(referential)?),
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

pub fn import_constraint(constraint: &GenericConstraintV1, relocation: &TypeRelocation) -> Result<GenericConstraint, ArtifactError> {
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

pub fn import_generic_signature(sig: &GenericSignatureV1, relocation: &TypeRelocation, store: &TypeStore) -> Result<GenericSignature, ArtifactError> {
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

pub fn import_declared_type_state(state: &DeclaredTypeStateV1, relocation: &TypeRelocation) -> Result<DeclaredTypeState, ArtifactError> {
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

pub fn import_declared_type_fact(fact: &DeclaredTypeFactV1, relocation: &TypeRelocation) -> Result<DeclaredTypeFact, ArtifactError> {
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

pub fn import_case_environment(env: &CaseEnvironmentV1, relocation: &TypeRelocation) -> Result<CaseTypeEnvironment, ArtifactError> {
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
        let static_data = data_wire.static_data.as_ref().map(|s| import_static_data(s, type_relocation)).transpose()?;
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
        let static_data = enum_wire.static_data.as_ref().map(|s| import_static_data(s, type_relocation)).transpose()?;
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

use crate::artifact::operational::import_checked_static_definitions;
use crate::artifact::validate::{compute_witness_layout_fingerprint, validate_pending_static_references};
use crate::static_definition::OperationalTemplate;
use crate::static_value::{StaticRelocation, StaticValueStore, import_static_value_graph};
use phalcom_compiler_artifact::semantic::type_store::TypeMemberSelectorV1;
use phalcom_compiler_artifact::semantic::world::{
    AssociatedBindingValueV1, AssociatedFamilyKindV1, AssociatedMemberV1, AssociatedStaticDomainRecordV1, AssociatedSurfaceRecordV1, ConformanceTargetV1,
    IntrinsicTypeOperationV1, RequirementSelectionV1, SemanticWorldV2, TypeMemberDeclarationRecordV1, TypeMemberDomainV1, TypeMemberImplementationV1,
};
use phalcom_type_meta::identity::StableDeclarationRef;

fn import_type_member_selector(selector: &TypeMemberSelectorV1) -> crate::types::type_member::TypeMemberSelector {
    match selector {
        TypeMemberSelectorV1::Named { name } => crate::types::type_member::TypeMemberSelector::Named(name.clone().into_boxed_str()),
        TypeMemberSelectorV1::Subscript => crate::types::type_member::TypeMemberSelector::Subscript,
    }
}

fn import_associated_selector(selector: &TypeMemberSelectorV1) -> phalcom_common::selector::SelectorBase {
    match selector {
        TypeMemberSelectorV1::Named { name } => phalcom_common::selector::SelectorBase::Named(name.clone()),
        TypeMemberSelectorV1::Subscript => phalcom_common::selector::SelectorBase::Subscript,
    }
}

fn import_associated_selector_with_allocation_budget(
    selector: &TypeMemberSelectorV1,
    allocations: &mut AllocationBudget,
) -> Result<phalcom_common::selector::SelectorBase, ArtifactError> {
    import_selector_base_with_allocation_budget(selector, allocations)
}

fn ensure_complete_callable_analysis_payload(analyses: &[phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV1]) -> Result<(), ArtifactError> {
    if analyses.is_empty() {
        return Ok(());
    }
    Err(referential(
        "callable analysis body facts are not completely represented; refusing semantic image import",
    ))
}

/// Reconstructs a callable flow graph into immutable, explicitly budgeted
/// storage. Edge endpoint ordinals determine adjacency, and each node's edge
/// lists follow ascending edge ordinal as in source graphs.
pub fn import_callable_flow_graph_with_allocation_budget(
    flow: &phalcom_compiler_artifact::semantic::world::CallableFlowGraphV1,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<Arc<crate::checker::flow::graph::FlowGraph>, ArtifactError> {
    use crate::checker::flow::graph::{FlowEdge, FlowEdgeKind, FlowGraph, FlowNode, FlowNodeKind};
    use phalcom_compiler_artifact::semantic::world::{CallableFlowEdgeKindV1, CallableFlowNodeKindV1};
    use phalcom_type_meta::allocation::BudgetedSortedMap;

    let node_count = flow.nodes.len();
    let edge_count = flow.edges.len();
    if node_count > limits.max_semantic_entries_per_family || edge_count > limits.max_semantic_entries_per_family {
        return Err(referential("callable flow graph exceeds its owner-local entry limit"));
    }

    // Work in a child ledger so a failed graph import publishes no partial
    // allocation charge to the enclosing semantic-image transaction.
    let mut child = allocations.child(allocations.remaining());
    let budget = &mut child;
    let mut degrees = budget.vector::<(usize, usize)>(node_count).map_err(allocation_error)?;
    degrees.resize(node_count, (0, 0));

    for edge in &flow.edges {
        let source = usize::try_from(edge.source).map_err(|_| referential("flow edge source ordinal exceeds address space"))?;
        let target = usize::try_from(edge.target).map_err(|_| referential("flow edge target ordinal exceeds address space"))?;
        if source >= node_count || target >= node_count {
            return Err(referential("callable flow edge references an absent node ordinal"));
        }
        degrees[source].1 = degrees[source].1.checked_add(1).ok_or_else(|| referential("flow out-degree overflow"))?;
        degrees[target].0 = degrees[target].0.checked_add(1).ok_or_else(|| referential("flow in-degree overflow"))?;
    }

    let entry = flow
        .entry
        .map(|ordinal| {
            let index = usize::try_from(ordinal).map_err(|_| referential("flow entry ordinal exceeds address space"))?;
            if index >= node_count {
                return Err(referential("callable flow entry references an absent node ordinal"));
            }
            if !matches!(&flow.nodes[index].kind, CallableFlowNodeKindV1::Entry) {
                return Err(referential("callable flow entry ordinal does not identify an entry node"));
            }
            Ok(crate::identity::FlowNodeId(ordinal))
        })
        .transpose()?;

    if !flow.exits.is_empty() {
        let mut seen_exits = budget.vector::<u8>(node_count).map_err(allocation_error)?;
        seen_exits.resize(node_count, 0);
        for exit in &flow.exits {
            let index = usize::try_from(*exit).map_err(|_| referential("flow exit ordinal exceeds address space"))?;
            if index >= node_count {
                return Err(referential("callable flow exit references an absent node ordinal"));
            }
            if !matches!(&flow.nodes[index].kind, CallableFlowNodeKindV1::Exit | CallableFlowNodeKindV1::Throw) {
                return Err(referential("callable flow exit ordinal does not identify an exit or throw node"));
            }
            if seen_exits[index] != 0 {
                return Err(referential("callable flow graph contains a duplicate exit"));
            }
            seen_exits[index] = 1;
        }
    }

    let mut nodes = budget.vector::<(crate::identity::FlowNodeId, FlowNode)>(node_count).map_err(allocation_error)?;
    for (index, record) in flow.nodes.iter().enumerate() {
        let ordinal = u32::try_from(index).map_err(|_| referential("flow node count exceeds portable ordinal range"))?;
        if record.range.start > record.range.end {
            return Err(referential("flow node source range is inverted"));
        }
        let range = SourceRange {
            start: usize::try_from(record.range.start).map_err(|_| referential("flow node range start exceeds address space"))?,
            end: usize::try_from(record.range.end).map_err(|_| referential("flow node range end exceeds address space"))?,
        };
        let kind = match &record.kind {
            CallableFlowNodeKindV1::Entry => FlowNodeKind::Entry,
            CallableFlowNodeKindV1::Exit => FlowNodeKind::Exit,
            CallableFlowNodeKindV1::Statement { index } => {
                FlowNodeKind::Statement(usize::try_from(*index).map_err(|_| referential("flow statement index exceeds address space"))?)
            }
            CallableFlowNodeKindV1::BranchCondition => FlowNodeKind::BranchCondition,
            CallableFlowNodeKindV1::LoopHeader => FlowNodeKind::LoopHeader,
            CallableFlowNodeKindV1::Join => FlowNodeKind::Join,
            CallableFlowNodeKindV1::Throw => FlowNodeKind::Throw,
            CallableFlowNodeKindV1::Unreachable => FlowNodeKind::Unreachable,
        };
        let predecessors = budget.vector::<crate::identity::FlowEdgeId>(degrees[index].0).map_err(allocation_error)?;
        let successors = budget.vector::<crate::identity::FlowEdgeId>(degrees[index].1).map_err(allocation_error)?;
        nodes.push((
            crate::identity::FlowNodeId(ordinal),
            FlowNode {
                id: crate::identity::FlowNodeId(ordinal),
                kind,
                range,
                predecessors,
                successors,
            },
        ));
    }

    let mut edges = budget.vector::<(crate::identity::FlowEdgeId, FlowEdge)>(edge_count).map_err(allocation_error)?;
    for (index, record) in flow.edges.iter().enumerate() {
        let ordinal = u32::try_from(index).map_err(|_| referential("flow edge count exceeds portable ordinal range"))?;
        let source = crate::identity::FlowNodeId(record.source);
        let target = crate::identity::FlowNodeId(record.target);
        let id = crate::identity::FlowEdgeId(ordinal);
        let source_index = usize::try_from(record.source).map_err(|_| referential("flow edge source ordinal exceeds address space"))?;
        let target_index = usize::try_from(record.target).map_err(|_| referential("flow edge target ordinal exceeds address space"))?;
        nodes[source_index].1.successors.push(id);
        nodes[target_index].1.predecessors.push(id);
        let kind = match &record.kind {
            CallableFlowEdgeKindV1::Normal => FlowEdgeKind::Normal,
            CallableFlowEdgeKindV1::TrueBranch => FlowEdgeKind::TrueBranch,
            CallableFlowEdgeKindV1::FalseBranch => FlowEdgeKind::FalseBranch,
            CallableFlowEdgeKindV1::BackEdge => FlowEdgeKind::BackEdge,
            CallableFlowEdgeKindV1::Break => FlowEdgeKind::Break,
            CallableFlowEdgeKindV1::Continue => FlowEdgeKind::Continue,
            CallableFlowEdgeKindV1::Return => FlowEdgeKind::Return,
            CallableFlowEdgeKindV1::Throw => FlowEdgeKind::Throw,
            CallableFlowEdgeKindV1::Unreachable => FlowEdgeKind::Unreachable,
        };
        edges.push((
            id,
            FlowEdge {
                id,
                source,
                target,
                kind,
                predicate: None,
            },
        ));
    }

    let mut exits = budget.vector::<crate::identity::FlowNodeId>(flow.exits.len()).map_err(allocation_error)?;
    for exit in &flow.exits {
        exits.push(crate::identity::FlowNodeId(*exit));
    }
    let nodes = BudgetedSortedMap::from_sorted_entries(nodes).map_err(|_| referential("reconstructed flow nodes are not strictly ordered"))?;
    let edges = BudgetedSortedMap::from_sorted_entries(edges).map_err(|_| referential("reconstructed flow edges are not strictly ordered"))?;
    let graph = FlowGraph::from_budgeted(budget, nodes, edges, entry, exits).map_err(allocation_error)?;
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(graph)
}

fn import_exact_callable(callable: &StableCallableRef) -> Result<CallableId, ArtifactError> {
    let selector =
        phalcom_common::selector::Selector::try_decode_exact(&callable.selector).map_err(|_| referential("artifact callable selector is not canonical"))?;
    if selector.encode() != callable.selector.as_ref() {
        return Err(referential("artifact callable selector is not in canonical encoding"));
    }
    let owner = match &callable.owner {
        StableCallableOwnerRef::Declaration(declaration) => CallableOwnerId::Declaration(from_stable_declaration(declaration).map_err(referential)?),
        StableCallableOwnerRef::Variant(variant) => CallableOwnerId::Variant(from_stable_variant(variant).map_err(referential)?),
        StableCallableOwnerRef::Conformance(implementation) => CallableOwnerId::Conformance(from_stable_impl(implementation).map_err(referential)?),
    };
    Ok(CallableId::new(
        owner,
        selector,
        crate::metadata::stable_identity::from_stable_dispatch_side(callable.side),
    ))
}

/// Relocates the currently supported static-argument portion of a callable's
/// selected call specializations. The complete callable-analysis importer must
/// attach these owner-local ordinals to its reconstructed expression rows.
pub(crate) fn import_callable_call_specializations(
    expression_count: u32,
    records: &[phalcom_compiler_artifact::semantic::world::CallableCallSpecializationV1],
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<Vec<(u32, crate::checker::analysis::CallSpecialization)>, ArtifactError> {
    use phalcom_compiler_artifact::semantic::type_store::StaticArgumentV1;
    let mut output = allocations
        .vector::<(u32, crate::checker::analysis::CallSpecialization)>(records.len())
        .map_err(allocation_error)?;
    let mut previous_ordinal = None;
    for record in records {
        if record.expression_ordinal >= expression_count || previous_ordinal.is_some_and(|previous| previous >= record.expression_ordinal) {
            return Err(referential("call specialization expression ordinals are absent, duplicated, or unsorted"));
        }
        previous_ordinal = Some(record.expression_ordinal);
        let callable = import_exact_callable(&record.callable)?;
        let mut bindings = allocations
            .vector::<(crate::types::id::TypeParameterId, crate::types::store::StaticArgument)>(record.bindings.len())
            .map_err(allocation_error)?;
        for binding in &record.bindings {
            let parameter = relocation.parameter(binding.parameter)?;
            if bindings.iter().any(|(existing, _)| *existing == parameter) {
                return Err(referential("call specialization contains duplicate relocated parameter bindings"));
            }
            let argument = match &binding.argument {
                StaticArgumentV1::Term { term } => crate::types::store::StaticArgument::Term(relocation.ty(*term)?),
                StaticArgumentV1::RecordRow { row } => crate::types::store::StaticArgument::RecordRow(relocation.row(*row)?),
                StaticArgumentV1::DynamicType => crate::types::store::StaticArgument::DynamicType,
            };
            bindings.push((parameter, argument));
        }
        output.push((
            record.expression_ordinal,
            crate::checker::analysis::CallSpecialization {
                callable,
                bindings: bindings.into_boxed_slice(),
                conformances: Box::default(),
            },
        ));
    }
    Ok(output)
}

fn import_unknown_reason_with_allocation_budget(reason: &UnknownReasonV1, allocations: &mut AllocationBudget) -> Result<UnknownReason, ArtifactError> {
    Ok(match reason {
        UnknownReasonV1::NoTypeEvidence => UnknownReason::NoTypeEvidence,
        UnknownReasonV1::MissingInitializer => UnknownReason::MissingInitializer,
        UnknownReasonV1::UnannotatedDeclaration => UnknownReason::UnannotatedDeclaration,
        UnknownReasonV1::UnresolvedName { name } => UnknownReason::UnresolvedName(allocations.boxed_str(name).map_err(allocation_error)?),
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
    })
}

fn import_source_span(source: &phalcom_compiler_artifact::semantic::world::EnumRequirementSourceV1) -> Result<SemanticSourceSpan, ArtifactError> {
    let module = from_stable_module(&source.module).map_err(referential)?;
    if source.range.start > source.range.end {
        return Err(referential("callable source span is inverted"));
    }
    Ok(SemanticSourceSpan::new(
        module,
        phalcom_common::range::SourceRange {
            start: usize::try_from(source.range.start).map_err(|_| referential("callable source start exceeds address space"))?,
            end: usize::try_from(source.range.end).map_err(|_| referential("callable source end exceeds address space"))?,
        },
    ))
}

fn import_return_flow(flow: &phalcom_compiler_artifact::semantic::world::ReturnFlowV1) -> Result<phalcom_native_meta::ReturnFlowSpec, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::ReturnFlowV1 as Portable;
    Ok(match flow {
        Portable::Value => phalcom_native_meta::ReturnFlowSpec::Value,
        Portable::Receiver => phalcom_native_meta::ReturnFlowSpec::Receiver,
        Portable::Argument { index } => {
            phalcom_native_meta::ReturnFlowSpec::Argument(usize::try_from(*index).map_err(|_| referential("callable return flow index exceeds address space"))?)
        }
        Portable::Never => phalcom_native_meta::ReturnFlowSpec::Never,
        Portable::Unknown => phalcom_native_meta::ReturnFlowSpec::Unknown,
    })
}

fn import_native_metadata(
    signature: &phalcom_compiler_artifact::semantic::world::CallableSignatureRecordV1,
) -> Result<
    (
        Option<phalcom_native_surface::NativeSurfaceId>,
        phalcom_native_meta::EffectSpec,
        phalcom_native_meta::RaisesSpec,
        phalcom_native_meta::ReturnFlowSpec,
        phalcom_native_meta::NativeLifecycleSpec,
    ),
    ArtifactError,
> {
    use phalcom_compiler_artifact::semantic::world::{EffectSpecV1, RaisesSpecV1};
    if let Some(native_id) = &signature.native_id {
        let owner = phalcom_native_meta::UniverseKey::from_name(&native_id.owner)
            .ok_or_else(|| referential("native surface owner is absent from the trusted catalog"))?;
        let side = match native_id.side {
            0 => phalcom_native_meta::NativeDispatch::Instance,
            1 => phalcom_native_meta::NativeDispatch::Class,
            _ => return Err(referential("native surface dispatch side is invalid")),
        };
        let record = phalcom_native_surface::find_native_surface(owner, side, &native_id.selector)
            .ok_or_else(|| referential("native surface identity is absent from the trusted catalog"))?;
        let expected_key = record.id().0;
        if expected_key.owner != owner || expected_key.side != side || expected_key.selector != native_id.selector.as_ref() {
            return Err(referential("native surface identity does not match its trusted catalog row"));
        }
        if crate::artifact::export::export_native_effects(record.effects()) != signature.effects
            || crate::artifact::export::export_native_raises(record.raises()) != signature.raises
            || crate::artifact::export::export_return_flow(record.flow())? != signature.flow
            || crate::artifact::export::export_native_lifecycle(record.lifecycle()) != signature.lifecycle
        {
            return Err(referential("native signature metadata disagrees with its trusted catalog row"));
        }
        return Ok((Some(record.id()), record.effects(), record.raises(), record.flow(), record.lifecycle()));
    }

    let effects = match &signature.effects {
        EffectSpecV1::Unknown => phalcom_native_meta::EffectSpec::Unknown,
        EffectSpecV1::Pure => phalcom_native_meta::EffectSpec::Pure,
        EffectSpecV1::Known { effects } if effects.is_empty() => phalcom_native_meta::EffectSpec::Known(&[]),
        EffectSpecV1::Known { .. } => return Err(referential("non-catalog native effects require an owned metadata representation")),
    };
    let raises = match &signature.raises {
        RaisesSpecV1::Unknown => phalcom_native_meta::RaisesSpec::Unknown,
        RaisesSpecV1::Known { types } if types.is_empty() => phalcom_native_meta::RaisesSpec::Known(&[]),
        RaisesSpecV1::Known { .. } => return Err(referential("non-catalog raised-type metadata requires an owned representation")),
    };
    if signature.lifecycle.since.is_some() || signature.lifecycle.deprecated_since.is_some() || signature.lifecycle.replacement.is_some() {
        return Err(referential("non-catalog native lifecycle metadata requires an owned representation"));
    }
    Ok((
        None,
        effects,
        raises,
        import_return_flow(&signature.flow)?,
        phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
    ))
}

fn import_type_knowledge_with_allocation_budget(
    knowledge: &phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::types::evidence::TypeKnowledge, ArtifactError> {
    use crate::types::evidence::{EvidenceOrigin, EvidenceSet, EvidenceStatus, TypeKnowledge};
    use phalcom_compiler_artifact::semantic::world::{EvidenceOriginV1, EvidenceStatusV1, TypeKnowledgeV1};

    // Keep a nested import atomic: the caller only pays for evidence that is
    // fully reconstructed, and allocation failure leaves its ledger intact.
    let mut child = allocations.child(allocations.remaining());
    let imported = match knowledge {
        TypeKnowledgeV1::Known {
            ty,
            status,
            origin,
            provenance,
        } => {
            let ty = relocation.ty(*ty)?;
            let status = match status {
                EvidenceStatusV1::Established => EvidenceStatus::Established,
                EvidenceStatusV1::Assumed => EvidenceStatus::Assumed,
            };
            let origin = match origin {
                EvidenceOriginV1::Syntax => EvidenceOrigin::Syntax,
                EvidenceOriginV1::DeclarationSemantics => EvidenceOrigin::DeclarationSemantics,
                EvidenceOriginV1::ConstructorSemantics => EvidenceOrigin::ConstructorSemantics,
                EvidenceOriginV1::CallableSignature => EvidenceOrigin::CallableSignature,
                EvidenceOriginV1::NativeSignature => EvidenceOrigin::NativeSignature,
                EvidenceOriginV1::DeveloperAnnotation => EvidenceOrigin::DeveloperAnnotation,
                EvidenceOriginV1::GenericInference => EvidenceOrigin::GenericInference,
                EvidenceOriginV1::Flow => EvidenceOrigin::Flow,
                EvidenceOriginV1::FieldLifecycle => EvidenceOrigin::FieldLifecycle,
                EvidenceOriginV1::ContextualDerivation => EvidenceOrigin::ContextualDerivation,
                EvidenceOriginV1::PatternDecomposition => EvidenceOrigin::PatternDecomposition,
            };
            let mut ranges = child
                .vector::<phalcom_common::range::SourceRange>(provenance.ranges.len())
                .map_err(allocation_error)?;
            for range in &provenance.ranges {
                if range.start > range.end {
                    return Err(referential("inferred type evidence span is inverted"));
                }
                ranges.push(phalcom_common::range::SourceRange {
                    start: usize::try_from(range.start).map_err(|_| referential("inferred type evidence start exceeds address space"))?,
                    end: usize::try_from(range.end).map_err(|_| referential("inferred type evidence end exceeds address space"))?,
                });
            }
            let mut descriptions = child.vector::<String>(provenance.descriptions.len()).map_err(allocation_error)?;
            for description in &provenance.descriptions {
                descriptions.push(child.string(description).map_err(allocation_error)?);
            }
            let provenance = EvidenceSet { ranges, descriptions };
            TypeKnowledge::known_with_evidence(ty, status, origin, provenance)
        }
        TypeKnowledgeV1::Unknown { reason } => TypeKnowledge::Unknown(import_unknown_reason_with_allocation_budget(reason, &mut child)?),
        TypeKnowledgeV1::Dynamic { reason } => TypeKnowledge::Dynamic(match reason {
            DynamicReasonV1::ExplicitEscape => DynamicReason::ExplicitEscape,
            DynamicReasonV1::DynamicRestPack => DynamicReason::DynamicRestPack,
            DynamicReasonV1::RuntimeReflection => DynamicReason::RuntimeReflection,
        }),
    };
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(imported)
}

fn import_callable_signature(
    record: &phalcom_compiler_artifact::semantic::world::CallableSignatureRecordV1,
    relocation: &TypeRelocation,
    store: &TypeStore,
    allocations: &mut AllocationBudget,
) -> Result<crate::signature::CallableSemanticSignature, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::{ImplementationKindV1, ParameterContractV1, RestValueContractV1, ReturnContractValidationV1};
    let callable = import_exact_callable(&record.callable)?;
    let owner = from_stable_declaration(&record.owner).map_err(referential)?;
    let side = match record.side {
        0 => DispatchSide::Instance,
        1 => DispatchSide::Class,
        _ => return Err(referential("callable signature dispatch side is invalid")),
    };
    let generics = record
        .generics
        .as_ref()
        .map(|generics| import_generic_signature(generics, relocation, store))
        .transpose()?;
    let mut parameters = Vec::with_capacity(record.parameters.len());
    for parameter in &record.parameters {
        let parameter_callable = import_exact_callable(&parameter.callable)?;
        let declared_type = import_declared_type_fact(&parameter.declared_type, relocation)?;
        let rest = match parameter.rest_mode {
            0 => phalcom_ast::ast::RestMode::None,
            1 => phalcom_ast::ast::RestMode::Positional,
            2 => phalcom_ast::ast::RestMode::Labeled,
            3 => phalcom_ast::ast::RestMode::Complete,
            _ => return Err(referential("callable parameter rest mode is invalid")),
        };
        let contract = match &parameter.contract {
            ParameterContractV1::Fixed { fact } => crate::signature::ParameterContract::Fixed(import_declared_type_fact(fact, relocation)?),
            ParameterContractV1::Rest { mode, contract } => {
                let mode = match mode {
                    1 => phalcom_ast::ast::RestMode::Positional,
                    2 => phalcom_ast::ast::RestMode::Labeled,
                    3 => phalcom_ast::ast::RestMode::Complete,
                    _ => return Err(referential("callable rest contract mode is invalid")),
                };
                crate::signature::ParameterContract::Rest {
                    mode,
                    contract: match contract {
                        RestValueContractV1::Unconstrained => crate::signature::RestValueContract::Unconstrained,
                        RestValueContractV1::Element { fact } => crate::signature::RestValueContract::Element(import_declared_type_fact(fact, relocation)?),
                        RestValueContractV1::Lane { ty } => crate::signature::RestValueContract::Lane(relocation.ty(*ty)?),
                        RestValueContractV1::Invalid { reason } => {
                            crate::signature::RestValueContract::Invalid(import_unknown_reason_with_allocation_budget(reason, allocations)?)
                        }
                    },
                }
            }
        };
        let source = parameter.source.as_ref().map(import_source_span).transpose()?;
        parameters.push(crate::signature::CallableParameterSemantic {
            id: crate::identity::CallableParameterId::new(parameter_callable, parameter.index),
            local_name: parameter.local_name.clone(),
            external_label: parameter.external_label.clone(),
            rest,
            declared_type,
            contract,
            source,
        });
    }
    let declared_return = import_declared_type_fact(&record.declared_return, relocation)?;
    let return_validation = match &record.return_validation {
        ReturnContractValidationV1::NotApplicable => crate::signature::ReturnContractValidation::NotApplicable,
        ReturnContractValidationV1::Unchecked => crate::signature::ReturnContractValidation::Unchecked,
        ReturnContractValidationV1::Satisfied { status } => crate::signature::ReturnContractValidation::Satisfied(match status {
            phalcom_compiler_artifact::semantic::world::EvidenceStatusV1::Established => crate::types::evidence::EvidenceStatus::Established,
            phalcom_compiler_artifact::semantic::world::EvidenceStatusV1::Assumed => crate::types::evidence::EvidenceStatus::Assumed,
        }),
        ReturnContractValidationV1::Refuted => crate::signature::ReturnContractValidation::Refuted,
        ReturnContractValidationV1::Blocked => crate::signature::ReturnContractValidation::Blocked,
        ReturnContractValidationV1::DynamicBoundary => crate::signature::ReturnContractValidation::DynamicBoundary,
    };
    let inferred_return = record
        .inferred_return
        .as_ref()
        .map(|knowledge| import_type_knowledge_with_allocation_budget(knowledge, relocation, allocations))
        .transpose()?;
    let source = record.source.as_ref().map(import_source_span).transpose()?;
    let implementation = match record.implementation {
        ImplementationKindV1::Phalcom => phalcom_common::modifier::ImplementationKind::Phalcom,
        ImplementationKindV1::Native => phalcom_common::modifier::ImplementationKind::Native,
        ImplementationKindV1::Foreign => phalcom_common::modifier::ImplementationKind::Foreign,
        ImplementationKindV1::Intrinsic => phalcom_common::modifier::ImplementationKind::Intrinsic,
    };
    let (native_id, effects, raises, flow, lifecycle) = import_native_metadata(record)?;
    Ok(crate::signature::CallableSemanticSignature {
        callable,
        owner,
        side,
        selector: import_exact_callable(&record.callable)?.selector,
        generics,
        parameters: parameters.into_boxed_slice(),
        declared_return,
        return_validation,
        inferred_return,
        source,
        implementation,
        native_id,
        effects,
        raises,
        flow,
        lifecycle,
    })
}

fn import_enum_requirements(
    records: &[phalcom_compiler_artifact::semantic::world::EnumRequirementOwnerV1],
    enum_semantics: &EnumSemanticTable,
    relocation: &TypeRelocation,
    store: &TypeStore,
    allocations: &mut AllocationBudget,
) -> Result<EnumRequirementTable, ArtifactError> {
    let mut requirement_rows = allocations
        .vector::<(DeclarationId, EnumRequirementRows<EnumRequirement>)>(records.len())
        .map_err(allocation_error)?;
    let mut status_rows = allocations
        .vector::<(DeclarationId, EnumRequirementRows<CaseRequirementResult>)>(records.len())
        .map_err(allocation_error)?;
    for owner_record in records {
        let owner = from_stable_declaration(&owner_record.owner).map_err(referential)?;
        let enum_info = enum_semantics
            .enum_info(&owner)
            .ok_or_else(|| referential("enum requirement owner has no reconstructed enum authority"))?;
        let mut ordered_requirements = allocations
            .vector::<Option<EnumRequirement>>(owner_record.requirements.len())
            .map_err(allocation_error)?;
        ordered_requirements.resize_with(owner_record.requirements.len(), || None);

        for record in &owner_record.requirements {
            let source_order = usize::try_from(record.source_order).map_err(|_| referential("enum requirement source order exceeds address space"))?;
            let slot = ordered_requirements
                .get_mut(source_order)
                .ok_or_else(|| referential("enum requirement source order is outside its owner table"))?;
            if slot.is_some() {
                return Err(referential("enum requirement source order is duplicated"));
            }
            let callable = import_exact_callable(&record.requirement)?;
            let crate::identity::CallableOwnerId::Declaration(callable_owner) = &callable.owner else {
                return Err(referential("enum root requirement is not declaration-owned"));
            };
            if callable_owner != &owner || callable.side != DispatchSide::Instance {
                return Err(referential("enum root requirement callable identity disagrees with its enum owner"));
            }
            let signature = import_callable_signature(&record.signature, relocation, store, allocations)?;
            if signature.callable != callable || signature.owner != owner || signature.side != DispatchSide::Instance || signature.selector != callable.selector
            {
                return Err(referential("inline enum requirement signature disagrees with its exact owner"));
            }
            let source = record
                .source
                .as_ref()
                .map(|source| -> Result<_, ArtifactError> {
                    let module = from_stable_module(&source.module).map_err(referential)?;
                    if module != owner.module || source.range.start > source.range.end {
                        return Err(referential("enum requirement source span disagrees with its enum owner"));
                    }
                    Ok(SemanticSourceSpan::new(
                        module,
                        phalcom_common::range::SourceRange {
                            start: usize::try_from(source.range.start).map_err(|_| referential("enum requirement source start exceeds address space"))?,
                            end: usize::try_from(source.range.end).map_err(|_| referential("enum requirement source end exceeds address space"))?,
                        },
                    ))
                })
                .transpose()?;
            *slot = Some(EnumRequirement {
                id: EnumRequirementId::new(owner.clone(), callable.selector.clone()),
                signature,
                source,
            });
        }

        let mut requirement_values = allocations
            .vector::<EnumRequirement>(owner_record.requirements.len())
            .map_err(allocation_error)?;
        for requirement in ordered_requirements {
            requirement_values.push(requirement.ok_or_else(|| referential("enum requirement source order has a gap"))?);
        }

        let expected_status_count = requirement_values
            .len()
            .checked_mul(enum_info.variants.len())
            .ok_or_else(|| referential("enum requirement case status count overflow"))?;
        if owner_record.case_statuses.len() != expected_status_count {
            return Err(referential("enum requirement case status count disagrees with owner products"));
        }
        let mut statuses = allocations
            .vector::<CaseRequirementResult>(owner_record.case_statuses.len())
            .map_err(allocation_error)?;
        if !enum_info.variants.is_empty() {
            for (index, record) in owner_record.case_statuses.iter().enumerate() {
                let requirement_order = index / enum_info.variants.len();
                let variant_order = index % enum_info.variants.len();
                let requirement = requirement_values
                    .get(requirement_order)
                    .ok_or_else(|| referential("enum case status requirement order is missing"))?;
                let variant = from_stable_variant(&record.variant).map_err(referential)?;
                if record.requirement_order as usize != requirement_order
                    || record.variant_order as usize != variant_order
                    || record.requirement != crate::metadata::stable_identity::to_stable_callable(&requirement.signature.callable)
                    || variant != enum_info.variants[variant_order]
                {
                    return Err(referential("enum case status order or owner identity disagrees with enum semantics"));
                }
                let status = match &record.status {
                    phalcom_compiler_artifact::semantic::world::CaseRequirementStatusV1::Satisfied { implementation } => {
                        let implementation = import_exact_callable(implementation)?;
                        if implementation.owner != CallableOwnerId::Variant(variant.clone())
                            || implementation.side != DispatchSide::Instance
                            || implementation.selector != requirement.id.selector
                        {
                            return Err(referential("satisfied enum requirement implementation has the wrong case or selector"));
                        }
                        CaseRequirementStatus::Satisfied { implementation }
                    }
                    phalcom_compiler_artifact::semantic::world::CaseRequirementStatusV1::Missing => CaseRequirementStatus::Missing,
                    phalcom_compiler_artifact::semantic::world::CaseRequirementStatusV1::Incompatible { implementation } => {
                        let implementation = import_exact_callable(implementation)?;
                        if implementation.owner != CallableOwnerId::Variant(variant.clone())
                            || implementation.side != DispatchSide::Instance
                            || implementation.selector != requirement.id.selector
                        {
                            return Err(referential("incompatible enum requirement implementation has the wrong case or selector"));
                        }
                        CaseRequirementStatus::Incompatible { implementation }
                    }
                    phalcom_compiler_artifact::semantic::world::CaseRequirementStatusV1::Blocked => CaseRequirementStatus::Blocked,
                };
                statuses.push(CaseRequirementResult {
                    variant,
                    requirement: requirement.id.clone(),
                    status,
                });
            }
        }

        let requirement_values = EnumRequirementRows::from_budgeted(allocations.arc(requirement_values).map_err(allocation_error)?);
        let statuses = EnumRequirementRows::from_budgeted(allocations.arc(statuses).map_err(allocation_error)?);
        requirement_rows.push((owner.clone(), requirement_values));
        status_rows.push((owner, statuses));
    }

    let requirements_by_owner = {
        let mut builder = allocations
            .sorted_map_builder::<DeclarationId, EnumRequirementRows<EnumRequirement>>(records.len())
            .map_err(allocation_error)?;
        for (owner, requirements) in requirement_rows {
            builder.push(owner, requirements).map_err(allocation_error)?;
        }
        builder.finish().map_err(|_| referential("duplicate enum requirement owner row"))?
    };
    let requirements = allocations.arc(requirements_by_owner).map_err(allocation_error)?;

    let case_statuses_by_owner = {
        let mut builder = allocations
            .sorted_map_builder::<DeclarationId, EnumRequirementRows<CaseRequirementResult>>(records.len())
            .map_err(allocation_error)?;
        for (owner, statuses) in status_rows {
            builder.push(owner, statuses).map_err(allocation_error)?;
        }
        builder.finish().map_err(|_| referential("duplicate enum requirement owner row"))?
    };
    let case_statuses = allocations.arc(case_statuses_by_owner).map_err(allocation_error)?;
    EnumRequirementTable::from_frozen(requirements, case_statuses, allocations).map_err(allocation_error)
}

fn import_type_member_domain(domain: &TypeMemberDomainV1, relocation: &TypeRelocation) -> Result<crate::types::type_member::TypeMemberDomain, ArtifactError> {
    Ok(match domain {
        TypeMemberDomainV1::Kind { kind } => crate::types::type_member::TypeMemberDomain::Kind(relocation.kind(*kind)?),
        TypeMemberDomainV1::ValueType { ty } => crate::types::type_member::TypeMemberDomain::ValueType(relocation.ty(*ty)?),
    })
}

fn import_intrinsic_type_operation(operation: &IntrinsicTypeOperationV1) -> crate::types::type_member::IntrinsicTypeOperation {
    use crate::types::type_member::IntrinsicTypeOperation as Native;
    match operation {
        IntrinsicTypeOperationV1::TuplePositionals => Native::TuplePositionals,
        IntrinsicTypeOperationV1::TupleLabeled => Native::TupleLabeled,
        IntrinsicTypeOperationV1::TupleEntries => Native::TupleEntries,
        IntrinsicTypeOperationV1::TupleElement => Native::TupleElement,
        IntrinsicTypeOperationV1::TupleAt => Native::TupleAt,
        IntrinsicTypeOperationV1::TupleMap => Native::TupleMap,
        IntrinsicTypeOperationV1::RecordFields => Native::RecordFields,
        IntrinsicTypeOperationV1::RecordValue => Native::RecordValue,
        IntrinsicTypeOperationV1::RecordMap => Native::RecordMap,
        IntrinsicTypeOperationV1::TypePackMap => Native::TypePackMap,
        IntrinsicTypeOperationV1::LabeledTypePackMap => Native::LabeledTypePackMap,
        IntrinsicTypeOperationV1::RecordRowMap => Native::RecordRowMap,
        IntrinsicTypeOperationV1::CompleteTypePackPositionals => Native::CompleteTypePackPositionals,
        IntrinsicTypeOperationV1::CompleteTypePackLabeled => Native::CompleteTypePackLabeled,
        IntrinsicTypeOperationV1::CompleteTypePackMap => Native::CompleteTypePackMap,
        IntrinsicTypeOperationV1::UnorderedShapeLookup => Native::UnorderedShapeLookup,
        IntrinsicTypeOperationV1::UnorderedShapeView => Native::UnorderedShapeView,
        IntrinsicTypeOperationV1::UnorderedShapeTail => Native::UnorderedShapeTail,
    }
}

fn import_type_member_implementation(implementation: &TypeMemberImplementationV1) -> phalcom_common::modifier::ImplementationKind {
    match implementation {
        TypeMemberImplementationV1::Phalcom => phalcom_common::modifier::ImplementationKind::Phalcom,
        TypeMemberImplementationV1::Native => phalcom_common::modifier::ImplementationKind::Native,
        TypeMemberImplementationV1::Foreign => phalcom_common::modifier::ImplementationKind::Foreign,
        TypeMemberImplementationV1::Intrinsic => phalcom_common::modifier::ImplementationKind::Intrinsic,
    }
}

fn import_associated_family_kind(kind: &AssociatedFamilyKindV1) -> crate::associated::AssociatedFamilyKind {
    match kind {
        AssociatedFamilyKindV1::Variant => crate::associated::AssociatedFamilyKind::Variant,
        AssociatedFamilyKindV1::DataConstructor => crate::associated::AssociatedFamilyKind::DataConstructor,
        AssociatedFamilyKindV1::StaticDomain => crate::associated::AssociatedFamilyKind::StaticDomain,
        AssociatedFamilyKindV1::TypeMember => crate::associated::AssociatedFamilyKind::TypeMember,
        AssociatedFamilyKindV1::IntrinsicTypeMember => crate::associated::AssociatedFamilyKind::IntrinsicTypeMember,
    }
}

fn import_associated_static_domain_declaration(
    declaration: &phalcom_type_meta::identity::StableAssociatedStaticDomainDeclarationRef,
) -> Result<crate::identity::AssociatedStaticDomainDeclarationId, ArtifactError> {
    Ok(crate::identity::AssociatedStaticDomainDeclarationId::new(
        from_stable_declaration(&declaration.owner).map_err(referential)?,
        declaration.name.clone(),
    ))
}

fn import_associated_member(member: &AssociatedMemberV1) -> Result<crate::associated::AssociatedMemberId, ArtifactError> {
    Ok(match member {
        AssociatedMemberV1::Variant { variant } => crate::associated::AssociatedMemberId::Variant(from_stable_variant(variant).map_err(referential)?),
        AssociatedMemberV1::DataConstructor { owner } => crate::associated::AssociatedMemberId::DataConstructor(crate::identity::DataConstructorId::new(
            from_stable_declaration(owner).map_err(referential)?,
        )),
        AssociatedMemberV1::DataComponent { owner, index } => crate::associated::AssociatedMemberId::DataComponent(crate::identity::DataComponentId::new(
            from_stable_declaration(owner).map_err(referential)?,
            *index,
        )),
        AssociatedMemberV1::StaticDomain { declaration } => {
            crate::associated::AssociatedMemberId::StaticDomain(import_associated_static_domain_declaration(declaration)?)
        }
        AssociatedMemberV1::TypeMember { owner, selector } => {
            crate::associated::AssociatedMemberId::TypeMember(crate::types::type_member::TypeMemberDeclarationId::new(
                from_stable_declaration(owner).map_err(referential)?,
                import_type_member_selector(selector),
            ))
        }
    })
}

fn import_selector_kind(kind: &phalcom_compiler_artifact::semantic::world::SelectorKindV1) -> phalcom_common::selector::SelectorKind {
    use phalcom_compiler_artifact::semantic::world::SelectorKindV1 as Portable;
    match kind {
        Portable::Getter => phalcom_common::selector::SelectorKind::Getter,
        Portable::Setter => phalcom_common::selector::SelectorKind::Setter,
        Portable::Method => phalcom_common::selector::SelectorKind::Method,
        Portable::SubscriptGet => phalcom_common::selector::SelectorKind::SubscriptGet,
        Portable::SubscriptSet => phalcom_common::selector::SelectorKind::SubscriptSet,
    }
}

fn import_selector_slot_with_allocation_budget(
    slot: &phalcom_compiler_artifact::semantic::world::SelectorSlotV1,
    allocations: &mut AllocationBudget,
) -> Result<phalcom_common::selector::SelectorSlot, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::SelectorSlotV1 as Portable;
    Ok(match slot {
        Portable::Positional => phalcom_common::selector::SelectorSlot::Positional,
        Portable::Label { label } => phalcom_common::selector::SelectorSlot::Label(allocations.string(label).map_err(allocation_error)?),
    })
}

fn import_selector_base_with_allocation_budget(
    base: &TypeMemberSelectorV1,
    allocations: &mut AllocationBudget,
) -> Result<phalcom_common::selector::SelectorBase, ArtifactError> {
    Ok(match base {
        TypeMemberSelectorV1::Named { name } => phalcom_common::selector::SelectorBase::Named(allocations.string(name).map_err(allocation_error)?),
        TypeMemberSelectorV1::Subscript => phalcom_common::selector::SelectorBase::Subscript,
    })
}

fn import_type_member_selector_with_allocation_budget(
    selector: &TypeMemberSelectorV1,
    allocations: &mut AllocationBudget,
) -> Result<crate::types::type_member::TypeMemberSelector, ArtifactError> {
    Ok(match selector {
        TypeMemberSelectorV1::Named { name } => crate::types::type_member::TypeMemberSelector::Named(allocations.boxed_str(name).map_err(allocation_error)?),
        TypeMemberSelectorV1::Subscript => crate::types::type_member::TypeMemberSelector::Subscript,
    })
}

fn import_family_operation_shape_with_allocation_budget(
    shape: &phalcom_compiler_artifact::semantic::world::FamilyOperationShapeV1,
    allocations: &mut AllocationBudget,
) -> Result<crate::types::family::FamilyOperationShape, ArtifactError> {
    let mut slots = allocations
        .vector::<phalcom_common::selector::SelectorSlot>(shape.slots.len())
        .map_err(allocation_error)?;
    for slot in &shape.slots {
        slots.push(import_selector_slot_with_allocation_budget(slot, allocations)?);
    }
    Ok(crate::types::family::FamilyOperationShape::new(
        import_selector_kind(&shape.kind),
        slots.into_boxed_slice(),
    ))
}

fn import_behavioral_family_spec_with_allocation_budget(
    spec: &phalcom_compiler_artifact::semantic::world::BehavioralFamilySpecV1,
    allocations: &mut AllocationBudget,
) -> Result<crate::checker::associated::BehavioralFamilySpec, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::{BehavioralFamilySpecV1, SelectorKindPatternV1};
    Ok(match spec {
        BehavioralFamilySpecV1::Exact { selector } => {
            let base = import_selector_base_with_allocation_budget(&selector.base, allocations)?;
            let mut slots = allocations
                .vector::<phalcom_common::selector::SelectorSlot>(selector.slots.len())
                .map_err(allocation_error)?;
            for slot in &selector.slots {
                slots.push(import_selector_slot_with_allocation_budget(slot, allocations)?);
            }
            let selector = phalcom_common::selector::Selector::new(base, import_selector_kind(&selector.kind), slots.into_boxed_slice())
                .map_err(|_| referential("behavioral exact selector is invalid"))?;
            crate::checker::associated::BehavioralFamilySpec::Exact(selector)
        }
        BehavioralFamilySpecV1::Pattern { pattern } => {
            let base = import_selector_base_with_allocation_budget(&pattern.base, allocations)?;
            let kind = match &pattern.kind {
                SelectorKindPatternV1::AnyNamed => phalcom_common::selector::SelectorKindPattern::AnyNamed,
                SelectorKindPatternV1::NamedAccessors => phalcom_common::selector::SelectorKindPattern::NamedAccessors,
                SelectorKindPatternV1::AnySubscript => phalcom_common::selector::SelectorKindPattern::AnySubscript,
                SelectorKindPatternV1::Exact { kind } => phalcom_common::selector::SelectorKindPattern::Exact(import_selector_kind(kind)),
            };
            let mut prefix = allocations
                .vector::<phalcom_common::selector::SelectorSlot>(pattern.prefix.len())
                .map_err(allocation_error)?;
            for slot in &pattern.prefix {
                prefix.push(import_selector_slot_with_allocation_budget(slot, allocations)?);
            }
            let mut suffix = allocations
                .vector::<phalcom_common::selector::SelectorSlot>(pattern.suffix.len())
                .map_err(allocation_error)?;
            for slot in &pattern.suffix {
                suffix.push(import_selector_slot_with_allocation_budget(slot, allocations)?);
            }
            let pattern = phalcom_common::selector::SelectorPattern::new(base, kind, prefix.into_boxed_slice(), suffix.into_boxed_slice(), pattern.has_gap)
                .map_err(|_| referential("behavioral selector pattern is invalid"))?;
            crate::checker::associated::BehavioralFamilySpec::Pattern(pattern)
        }
    })
}

fn import_invocation_target(
    target: &phalcom_compiler_artifact::semantic::world::InvocationTargetV1,
) -> Result<crate::identity::InvocationTargetId, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::InvocationTargetV1 as Portable;
    Ok(match target {
        Portable::Behavioral { callable } => crate::identity::InvocationTargetId::Behavioral(from_stable_callable(callable).map_err(referential)?),
        Portable::VariantConstructor { variant } => {
            crate::identity::InvocationTargetId::VariantConstructor(VariantConstructorId::new(from_stable_variant(variant).map_err(referential)?))
        }
        Portable::DataConstructor { owner } => {
            crate::identity::InvocationTargetId::DataConstructor(DataConstructorId::new(from_stable_declaration(owner).map_err(referential)?))
        }
    })
}

fn import_conditional_dispatch_selection(
    selection: &phalcom_compiler_artifact::semantic::world::ConditionalDispatchSelectionV1,
) -> Result<crate::dispatch::ConditionalDispatchSelection, ArtifactError> {
    if selection.side > 1 {
        return Err(referential("conditional dispatch selection side is invalid"));
    }
    Ok(crate::dispatch::ConditionalDispatchSelection {
        impl_id: from_stable_impl(&selection.impl_id).map_err(referential)?,
        callable: from_stable_callable(&selection.callable).map_err(referential)?,
        declaring_owner: from_stable_declaration(&selection.declaring_owner).map_err(referential)?,
        side: if selection.side == 0 { DispatchSide::Instance } else { DispatchSide::Class },
    })
}

fn import_associated_member_with_allocation_budget(
    member: &AssociatedMemberV1,
    allocations: &mut AllocationBudget,
) -> Result<crate::associated::AssociatedMemberId, ArtifactError> {
    use AssociatedMemberV1 as Portable;
    Ok(match member {
        Portable::Variant { variant } => crate::associated::AssociatedMemberId::Variant(from_stable_variant(variant).map_err(referential)?),
        Portable::DataConstructor { owner } => {
            crate::associated::AssociatedMemberId::DataConstructor(DataConstructorId::new(from_stable_declaration(owner).map_err(referential)?))
        }
        Portable::DataComponent { owner, index } => {
            crate::associated::AssociatedMemberId::DataComponent(DataComponentId::new(from_stable_declaration(owner).map_err(referential)?, *index))
        }
        Portable::StaticDomain { declaration } => {
            crate::associated::AssociatedMemberId::StaticDomain(crate::identity::AssociatedStaticDomainDeclarationId::new(
                from_stable_declaration(&declaration.owner).map_err(referential)?,
                allocations.boxed_str(&declaration.name).map_err(allocation_error)?,
            ))
        }
        Portable::TypeMember { owner, selector } => crate::associated::AssociatedMemberId::TypeMember(crate::types::type_member::TypeMemberDeclarationId::new(
            from_stable_declaration(owner).map_err(referential)?,
            import_type_member_selector_with_allocation_budget(selector, allocations)?,
        )),
    })
}

fn budgeted_arc_slice<T>(values: Vec<T>, allocations: &mut AllocationBudget) -> Result<Arc<[T]>, ArtifactError> {
    use std::alloc::Layout;
    use std::sync::atomic::AtomicUsize;

    if values.capacity() > values.len() {
        allocations.reserve_array::<T>(values.len()).map_err(allocation_error)?;
    }
    let payload = Layout::array::<T>(values.len()).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let layout = Layout::new::<[AtomicUsize; 2]>()
        .extend(payload)
        .map_err(|_| allocation_error(AllocationError::Overflow))?
        .0
        .pad_to_align();
    allocations.reserve_layout(layout).map_err(allocation_error)?;
    Ok(Arc::from(values.into_boxed_slice()))
}

fn import_semantic_denotation_inner(
    denotation: &phalcom_compiler_artifact::semantic::world::SemanticDenotationV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::types::denotation::SemanticDenotation, ArtifactError> {
    use crate::types::denotation::{
        AssociatedValueDenotation, BehavioralFamilyDenotation, CapturedAssociatedMember, CapturedBehavioralMember, SemanticDenotation,
    };
    use phalcom_compiler_artifact::semantic::world::{AssociatedValueDenotationV1, SemanticDenotationV1};
    Ok(match denotation {
        SemanticDenotationV1::TypeForm { ty } => SemanticDenotation::TypeForm(relocation.ty(*ty)?),
        SemanticDenotationV1::Kind { kind } => SemanticDenotation::Kind(relocation.kind(*kind)?),
        SemanticDenotationV1::StaticDomain { domain } => SemanticDenotation::StaticDomain(relocation.domain(*domain)?),
        SemanticDenotationV1::StaticTerm { term } => SemanticDenotation::StaticTerm(relocation.ty(*term)?),
        SemanticDenotationV1::StaticApplication { .. } => {
            return Err(referential("static-application denotation has no retained checked-body application row"));
        }
        SemanticDenotationV1::AssociatedValue { value } => {
            let value = match value {
                AssociatedValueDenotationV1::Exact {
                    owner_form,
                    lookup_owner,
                    member,
                    target,
                } => {
                    let target = target.as_ref().map(import_invocation_target).transpose()?;
                    AssociatedValueDenotation::Exact {
                        owner_form: relocation.ty(*owner_form)?,
                        lookup_owner: from_stable_declaration(lookup_owner).map_err(referential)?,
                        member: import_associated_member_with_allocation_budget(member, allocations)?,
                        target: allocations.boxed(target).map_err(allocation_error)?,
                    }
                }
                AssociatedValueDenotationV1::Family {
                    owner_form,
                    lookup_owner,
                    family,
                    members,
                } => {
                    let mut captured = allocations.vector::<CapturedAssociatedMember>(members.len()).map_err(allocation_error)?;
                    for member in members {
                        captured.push(CapturedAssociatedMember {
                            operation: import_family_operation_shape_with_allocation_budget(&member.operation, allocations)?,
                            member: import_associated_member_with_allocation_budget(&member.member, allocations)?,
                            target: member.target.as_ref().map(import_invocation_target).transpose()?,
                        });
                    }
                    let family_owner = from_stable_declaration(&family.owner).map_err(referential)?;
                    let family_base = import_associated_selector_with_allocation_budget(&family.selector, allocations)?;
                    AssociatedValueDenotation::Family {
                        owner_form: relocation.ty(*owner_form)?,
                        lookup_owner: from_stable_declaration(lookup_owner).map_err(referential)?,
                        family: crate::identity::AssociatedFamilyId::new(family_owner, family_base),
                        members: budgeted_arc_slice(captured, allocations)?,
                    }
                }
            };
            SemanticDenotation::AssociatedValue(allocations.boxed(value).map_err(allocation_error)?)
        }
        SemanticDenotationV1::BehavioralFamily { family } => {
            let mut members = allocations.vector::<CapturedBehavioralMember>(family.members.len()).map_err(allocation_error)?;
            for member in &family.members {
                members.push(CapturedBehavioralMember {
                    operation: import_family_operation_shape_with_allocation_budget(&member.operation, allocations)?,
                    target: import_invocation_target(&member.target)?,
                    conditional: member.conditional.as_ref().map(import_conditional_dispatch_selection).transpose()?,
                });
            }
            let family = BehavioralFamilyDenotation {
                receiver_type: relocation.ty(family.receiver_type)?,
                receiver_application: family.receiver_application.map(|application| relocation.ty(application)).transpose()?,
                spec: import_behavioral_family_spec_with_allocation_budget(&family.spec, allocations)?,
                members: budgeted_arc_slice(members, allocations)?,
            };
            SemanticDenotation::BehavioralFamily(allocations.boxed(family).map_err(allocation_error)?)
        }
    })
}

/// Imports one denotation using the shared artifact type relocation.
///
/// The caller's ledger is committed only if the full denotation is reconstructed.
pub fn import_semantic_denotation(
    denotation: &phalcom_compiler_artifact::semantic::world::SemanticDenotationV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::types::denotation::SemanticDenotation, ArtifactError> {
    let mut child = allocations.child(allocations.remaining());
    let imported = import_semantic_denotation_inner(denotation, relocation, &mut child)?;
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(imported)
}

/// Reconstructed denotations keyed by callable-body-local ordinals.
#[derive(Debug, Eq, PartialEq)]
pub struct ImportedCallableDenotations {
    /// Denotations owned by expression records, in ascending expression ordinal.
    pub expressions: Vec<(u32, crate::types::denotation::SemanticDenotation)>,
    /// Denotations owned by binding records, in ascending binding ordinal.
    pub bindings: Vec<(u32, crate::types::denotation::SemanticDenotation)>,
}

/// Imports expression/binding denotations using the shared type relocation.
/// Static applications remain rejected until their checked-body rows are retained.
pub fn import_callable_denotations(
    record: &phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<ImportedCallableDenotations, ArtifactError> {
    let mut child = allocations.child(allocations.remaining());
    let mut expressions = child
        .vector::<(u32, crate::types::denotation::SemanticDenotation)>(record.expression_denotations.len())
        .map_err(allocation_error)?;
    for denotation in &record.expression_denotations {
        expressions.push((
            denotation.expression_ordinal,
            import_semantic_denotation_inner(&denotation.denotation, relocation, &mut child)?,
        ));
    }
    let mut bindings = child
        .vector::<(u32, crate::types::denotation::SemanticDenotation)>(record.binding_denotations.len())
        .map_err(allocation_error)?;
    for denotation in &record.binding_denotations {
        bindings.push((
            denotation.binding_ordinal,
            import_semantic_denotation_inner(&denotation.denotation, relocation, &mut child)?,
        ));
    }
    let imported = ImportedCallableDenotations { expressions, bindings };
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(imported)
}

pub(super) struct ImportedAssociatedAuthority {
    pub associated_surfaces: crate::associated::AssociatedFamilyTable,
    pub type_member_declarations:
        crate::parent_map::ParentMap<crate::types::type_member::TypeMemberDeclarationId, Arc<crate::db::product::TypeMemberDeclarationProduct>>,
    pub associated_static_domains:
        BTreeMap<crate::identity::AssociatedStaticDomainDeclarationId, Arc<crate::db::product::AssociatedStaticDomainDeclarationProduct>>,
}

pub(super) fn import_associated_authority(
    type_member_records: &[TypeMemberDeclarationRecordV1],
    static_domain_records: &[AssociatedStaticDomainRecordV1],
    surface_records: &[AssociatedSurfaceRecordV1],
    type_relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<ImportedAssociatedAuthority, ArtifactError> {
    let mut type_member_declarations = crate::parent_map::ParentMap::default();
    for rec in type_member_records {
        let owner = from_stable_declaration(&rec.owner).map_err(referential)?;
        let selector = import_type_member_selector(&rec.selector);
        let id = crate::types::type_member::TypeMemberDeclarationId::new(owner, selector.clone());
        let mut parameters = allocations
            .vector::<crate::types::type_member::TypeMemberParameter>(rec.parameters.len())
            .map_err(allocation_error)?;
        for parameter in &rec.parameters {
            parameters.push(crate::types::type_member::TypeMemberParameter {
                name: parameter.name.clone(),
                domain: import_type_member_domain(&parameter.domain, type_relocation)?,
            });
        }
        let product = crate::db::product::TypeMemberDeclarationProduct {
            id: id.clone(),
            signature: crate::types::type_member::TypeMemberSignature {
                selector,
                parameters: parameters.into_boxed_slice(),
                result_domain: import_type_member_domain(&rec.result_domain, type_relocation)?,
                intrinsic: rec.intrinsic.as_ref().map(import_intrinsic_type_operation),
                implementation: import_type_member_implementation(&rec.implementation),
            },
            source_range: phalcom_common::range::SourceRange::default(),
        };
        let product = allocations.arc(product).map_err(allocation_error)?;
        if type_member_declarations.insert(id, product).is_some() {
            return Err(referential("duplicate type-member declaration in semantic artifact"));
        }
    }

    let mut associated_static_domains = BTreeMap::new();
    for rec in static_domain_records {
        let id = import_associated_static_domain_declaration(&rec.declaration)?;
        let domain = type_relocation.domain(rec.domain)?;
        let mut constructors = allocations
            .vector::<crate::types::id::AlgebraicStaticConstructorId>(rec.constructors.len())
            .map_err(allocation_error)?;
        for &constructor in &rec.constructors {
            constructors.push(type_relocation.constructor(constructor)?);
        }
        let product = crate::db::product::AssociatedStaticDomainDeclarationProduct {
            id: id.clone(),
            domain,
            constructors: constructors.into_boxed_slice(),
            source_range: phalcom_common::range::SourceRange::default(),
        };
        let product = allocations.arc(product).map_err(allocation_error)?;
        if associated_static_domains.insert(id, product).is_some() {
            return Err(referential("duplicate associated static-domain declaration in semantic artifact"));
        }
    }

    let mut associated_surfaces = crate::associated::AssociatedFamilyTable::new();
    for rec in surface_records {
        let owner = from_stable_declaration(&rec.owner).map_err(referential)?;
        let mut surface = crate::associated::AssociatedSurface::new(owner.clone());
        for family in &rec.families {
            let base = import_associated_selector(&family.selector);
            let mut members = allocations
                .vector::<crate::associated::AssociatedMemberId>(family.members.len())
                .map_err(allocation_error)?;
            for member in &family.members {
                members.push(import_associated_member(member)?);
            }
            let family_info = crate::associated::AssociatedFamilyInfo {
                id: crate::identity::AssociatedFamilyId::new(owner.clone(), base.clone()),
                kind: import_associated_family_kind(&family.kind),
                members: members.into_boxed_slice(),
            };
            if surface.families.insert(base, family_info).is_some() {
                return Err(referential("duplicate associated family in semantic artifact"));
            }
        }
        if associated_surfaces.surfaces.contains_key(&owner) {
            return Err(referential("duplicate associated surface in semantic artifact"));
        }
        let surface = allocations.arc(surface).map_err(allocation_error)?;
        associated_surfaces.insert(owner, surface);
    }

    Ok(ImportedAssociatedAuthority {
        associated_surfaces,
        type_member_declarations,
        associated_static_domains,
    })
}

#[derive(Debug, Default)]
pub struct ReconstructedModuleOwnerIndexes {
    pub declarations_by_module: BTreeMap<String, Vec<StableDeclarationRef>>,
    pub data_by_module: BTreeMap<String, Vec<StableDeclarationRef>>,
    pub enums_by_module: BTreeMap<String, Vec<StableDeclarationRef>>,
}

pub struct ValidatedSemanticParts {
    pub structural_contracts: LoadedStructuralContracts,
    pub type_store: TypeStore,
    pub type_relocation: TypeRelocation,
    pub static_values: StaticValueStore,
    pub static_relocation: StaticRelocation,
    pub operational_templates: Vec<OperationalTemplate>,
    pub owner_indexes: ReconstructedModuleOwnerIndexes,
    pub snapshot: Arc<crate::snapshot::SemanticSnapshot>,
}

/// Executes stages 3–7 of the P7 loader construction order (`T4/T5` -> `T7.A` -> `T6` -> `T7.B` -> `T8`).
pub fn validate_and_import_semantic_world(world: SemanticWorldV2, limits: &ValidationLimits) -> Result<ValidatedSemanticParts, ArtifactError> {
    let budget_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = AllocationBudget::new(budget_limit);
    validate_and_import_semantic_world_with_allocation_budget(world, limits, &mut allocations)
}

/// Imports semantic authority using the caller's cumulative load transaction.
/// It currently fails closed while validation and materialization still have
/// allocation requests outside that ledger.
pub fn validate_and_import_semantic_world_with_allocation_budget(
    world: SemanticWorldV2,
    limits: &ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<ValidatedSemanticParts, ArtifactError> {
    // TypeStore relocation/interning and the primary/static owner imports do
    // not yet expose every allocator request to this transaction. Reject
    // before validation or materialization can allocate outside its ledger.
    require_semantic_import_allocation_accounting()?;
    let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = transaction.child(section_limit);
    let parts = validate_and_import_semantic_world_inner(world, limits, &mut allocations)?;
    transaction.commit_child(allocations).map_err(allocation_error)?;
    Ok(parts)
}

fn require_semantic_import_allocation_accounting() -> Result<(), ArtifactError> {
    Err(AllocationError::UnaccountedBoundary.into())
}

fn allocation_error(error: AllocationError) -> ArtifactError {
    // Keep budget failure reporting allocation-free. `LimitExceeded(String)`
    // would request a new owned string precisely when the budget may already
    // be exhausted; the typed allocation variant carries the cause directly.
    ArtifactError::Allocation(error)
}

fn validate_and_import_semantic_world_inner(
    world: SemanticWorldV2,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ValidatedSemanticParts, ArtifactError> {
    world.validate_with_allocation_budget(limits, allocations)?;
    ensure_complete_callable_analysis_payload(&world.callable_analyses)?;

    // 3. Fresh TypeStore with checked TypeRelocation
    let (mut type_store, type_relocation) = TypeStore::import_type_graph(&world.type_graph, limits)?;

    // 4. Structural primary contracts (shells, hierarchy, aliases, data/enum contracts, signatures)
    let structural_contracts = import_structural_primary_contracts(&world.structural_contracts, &mut type_store, &type_relocation, limits)?;

    // 5. Fresh StaticValueStore through validated factories
    let (static_values, static_relocation) = import_static_value_graph(
        &world.static_values,
        &type_store,
        &type_relocation,
        &structural_contracts.data_table,
        &structural_contracts.enum_table,
        &|decl_ref| from_stable_declaration(decl_ref).map_err(referential),
        &|var_ref| from_stable_variant(var_ref).map_err(referential),
        limits,
    )?;

    // Cross-validate pending static references
    validate_pending_static_references(&world.structural_contracts, &static_relocation)?;

    // 6. Checked operational static definitions
    let operational_templates = import_checked_static_definitions(&world.checked_definitions, &static_relocation, &type_relocation, limits)?;

    // 7. Verify witness plans and layout fingerprints (P7.T8)
    for witness in &world.witness_plans {
        let actual = compute_witness_layout_fingerprint(&witness.ordered_selected_callables);
        if actual != witness.slot_layout_fingerprint {
            return Err(referential("witness plan slot layout fingerprint mismatch"));
        }
    }

    // 8. Reconstruct callable signatures and definitions (P7.T7.B)
    let mut callable_signatures = crate::signature::CallableSignatureTable::new();
    let mut callable_definitions_map = std::collections::HashMap::new();
    for rec in &world.callable_signatures {
        let sig = import_callable_signature(rec, &type_relocation, &type_store, allocations)?;
        let callable = sig.callable.clone();
        callable_definitions_map.insert(
            callable.clone(),
            crate::impls::EffectiveCallableDefinition {
                callable: callable.clone(),
                origin: crate::impls::CallableDefinitionOrigin::PrimaryDeclaration,
                source_member_index: 0,
                signature: sig.clone(),
                inherent_body_context: None,
            },
        );
        callable_signatures.insert(sig);
    }

    // Callable analysis imports are rejected before materialization until their
    // complete body-fact payload and fresh dependency fingerprint are present.
    let callable_analyses = crate::checker::analysis::CallableAnalysisTable::new();
    let enum_requirements = import_enum_requirements(
        &world.enum_requirements,
        &structural_contracts.enum_table,
        &type_relocation,
        &type_store,
        allocations,
    )?;

    // 10. Reconstruct trait surfaces and trait headers (P7.T7.B)
    let mut trait_surfaces = crate::traits::TraitSurfaceTable::new();
    let mut trait_headers = crate::traits::TraitHeaderTable::new();
    for rec in &world.trait_surfaces {
        let decl = from_stable_declaration(&rec.declaration).map_err(referential)?;
        let generic_signature = if let Some(g) = &rec.generic_signature {
            Some(import_generic_signature(g, &type_relocation, &mut type_store)?)
        } else {
            None
        };
        let mut surface = crate::traits::TraitSurface::new(decl.clone(), generic_signature.clone());
        for (idx, name) in rec.associated_types.iter().enumerate() {
            let req_id = crate::traits::AssociatedTypeRequirementId::new(decl.clone(), idx as u32);
            surface.associated_types.insert(
                req_id.clone(),
                crate::traits::TraitAssociatedTypeRequirement {
                    requirement: req_id,
                    name: name.clone(),
                    domain: crate::types::type_member::TypeMemberDomain::Kind(crate::types::id::KindId(0)),
                    source: crate::diagnostic::SemanticSourceSpan::new(decl.module.clone(), phalcom_common::range::SourceRange::default()),
                },
            );
        }
        for call_ref in &rec.required_callables {
            let cid = from_stable_callable(call_ref).map_err(referential)?;
            let req_id = crate::traits::TraitRequirementId {
                owner: decl.clone(),
                selector: cid.selector.clone(),
                side: cid.side,
            };
            if let Some(sig) = callable_signatures.get(&cid) {
                surface.members.insert(
                    req_id.clone(),
                    crate::traits::TraitSurfaceMember {
                        requirement: req_id,
                        callable: cid,
                        signature: sig.clone(),
                        visibility: Visibility::Public,
                        source: crate::diagnostic::SemanticSourceSpan::new(decl.module.clone(), phalcom_common::range::SourceRange::default()),
                        default_present: false,
                        default_source: None,
                    },
                );
            }
        }
        for call_ref in &rec.default_callables {
            let cid = from_stable_callable(call_ref).map_err(referential)?;
            let req_id = crate::traits::TraitRequirementId {
                owner: decl.clone(),
                selector: cid.selector.clone(),
                side: cid.side,
            };
            if let Some(sig) = callable_signatures.get(&cid) {
                surface.members.insert(
                    req_id.clone(),
                    crate::traits::TraitSurfaceMember {
                        requirement: req_id,
                        callable: cid,
                        signature: sig.clone(),
                        visibility: Visibility::Public,
                        source: crate::diagnostic::SemanticSourceSpan::new(decl.module.clone(), phalcom_common::range::SourceRange::default()),
                        default_present: true,
                        default_source: None,
                    },
                );
            }
        }
        surface.finalize_requirement_layout();
        trait_surfaces.insert(Arc::new(surface));
        trait_headers.insert(Arc::new(crate::traits::TraitHeader {
            declaration: decl.clone(),
            generic_signature,
            source: crate::diagnostic::SemanticSourceSpan::new(decl.module.clone(), phalcom_common::range::SourceRange::default()),
        }));
    }

    // 11. Reconstruct surfaces, dispatch resolver, and field signatures (P7.T7.B)
    let mut dispatch_resolver = crate::dispatch::SurfaceDispatchResolver::new();
    let mut field_signatures = crate::signature::FieldSignatureTable::new();
    for rec in &world.dispatch_surfaces {
        let decl = from_stable_declaration(&rec.declaration).map_err(referential)?;
        let mut surface = crate::surface::DeclarationSurface::new(Some(decl.clone()));
        for call_ref in &rec.instance_callables {
            let cid = from_stable_callable(call_ref).map_err(referential)?;
            surface.instance.callables.insert(
                cid.clone(),
                crate::types::evidence::TypeKnowledge::assumed(crate::types::id::TypeId(0), crate::types::evidence::EvidenceOrigin::Flow),
            );
            surface.instance.callables_by_selector.insert(cid.selector.clone(), cid);
        }
        for call_ref in &rec.class_callables {
            let cid = from_stable_callable(call_ref).map_err(referential)?;
            surface.class.callables.insert(
                cid.clone(),
                crate::types::evidence::TypeKnowledge::assumed(crate::types::id::TypeId(0), crate::types::evidence::EvidenceOrigin::Flow),
            );
            surface.class.callables_by_selector.insert(cid.selector.clone(), cid);
        }
        for f in &rec.fields {
            let f_owner = from_stable_declaration(&f.owner).map_err(referential)?;
            let f_side = match f.side {
                0 => crate::identity::DispatchSide::Instance,
                _ => crate::identity::DispatchSide::Class,
            };
            let declared_type = import_declared_type_fact(&f.declared_type, &type_relocation)?;
            let field_id = crate::identity::FieldId::new(f_owner.clone(), f.name.clone(), f_side);
            field_signatures.insert(crate::signature::FieldSemanticSignature {
                field: field_id,
                owner: f_owner,
                side: f_side,
                name: f.name.clone(),
                mutable: f.mutable,
                declared_type,
                source: None,
            });
        }
        dispatch_resolver.register_surface(decl, surface);
    }

    // 12. Reconstruct conformance index (P7.T7.B)
    let mut conformance_index = crate::impls::ConformanceIndex::new();
    for rec in &world.conformance_heads {
        let impl_id = from_stable_impl(&rec.impl_id).map_err(referential)?;
        let source_module = from_stable_module(&rec.source_module).map_err(referential)?;
        let target = match &rec.target {
            ConformanceTargetV1::Declaration { declaration } => {
                crate::impls::ConformanceTarget::Declaration(from_stable_declaration(declaration).map_err(referential)?)
            }
            ConformanceTargetV1::ExactEnumCase { variant } => {
                crate::impls::ConformanceTarget::ExactEnumCase(from_stable_variant(variant).map_err(referential)?)
            }
        };
        let target_head = type_relocation.ty(rec.target_type)?;
        let trait_decl = from_stable_declaration(&rec.trait_ref.declaration).map_err(referential)?;
        let mut args = Vec::new();
        for &a in &rec.trait_ref.arguments {
            args.push(type_relocation.ty(a)?);
        }
        let generic_signature = if let Some(g) = &rec.generic_signature {
            Some(import_generic_signature(g, &type_relocation, &mut type_store)?)
        } else {
            None
        };
        conformance_index.insert(crate::impls::ConformanceContribution {
            impl_id: impl_id.clone(),
            source_module,
            trait_ref: crate::traits::TraitRef::new(trait_decl, args),
            target,
            target_head,
            generic_signature,
            authorized: rec.authorized,
            eligible: rec.eligible,
            source: crate::diagnostic::SemanticSourceSpan::new(impl_id.module.clone(), phalcom_common::range::SourceRange::default()),
            diagnostics: Box::new([]),
        });
    }

    let mut witness_plans_map = BTreeMap::new();
    for witness in &world.witness_plans {
        let impl_id = from_stable_impl(&witness.impl_id).map_err(referential)?;
        let source_module = from_stable_module(&witness.source_module).map_err(referential)?;
        let trait_decl = from_stable_declaration(&witness.trait_ref_template.declaration).map_err(referential)?;
        let mut trait_args = Vec::with_capacity(witness.trait_ref_template.arguments.len());
        for &a in &witness.trait_ref_template.arguments {
            trait_args.push(type_relocation.ty(a)?);
        }
        let trait_ref_template = crate::traits::TraitRef::new(trait_decl.clone(), trait_args);
        let target_template = type_relocation.ty(witness.target_template)?;
        let generic_signature = if let Some(g) = &witness.generic_signature {
            Some(import_generic_signature(g, &type_relocation, &mut type_store)?)
        } else {
            None
        };
        let mut requirements = BTreeMap::new();
        for entry in &witness.requirements {
            let side = match entry.side {
                0 => crate::identity::DispatchSide::Instance,
                _ => crate::identity::DispatchSide::Class,
            };
            let selector = phalcom_common::selector::Selector::decode(&entry.selector);
            let req_id = crate::traits::TraitRequirementId {
                owner: trait_decl.clone(),
                selector,
                side,
            };
            let selection = match &entry.selection {
                RequirementSelectionV1::ConformanceCallable { callable } => {
                    let c = from_stable_callable(callable).map_err(referential)?;
                    crate::impls::RequirementSelectionTemplate::ConformanceCallable { callable: c }
                }
                RequirementSelectionV1::InherentCallable { callable } => {
                    let c = from_stable_callable(callable).map_err(referential)?;
                    crate::impls::RequirementSelectionTemplate::InherentCallable {
                        callable: c,
                        conditional_impl: None,
                        applicability: None,
                    }
                }
                RequirementSelectionV1::TraitDefault { callable } => {
                    let c = from_stable_callable(callable).map_err(referential)?;
                    crate::impls::RequirementSelectionTemplate::TraitDefault { callable: c }
                }
                RequirementSelectionV1::DataComponent {
                    owner,
                    index,
                    specialized_type,
                } => {
                    let d = from_stable_declaration(owner).map_err(referential)?;
                    let specialized_type = type_relocation.ty(*specialized_type)?;
                    crate::impls::RequirementSelectionTemplate::DataComponent {
                        component: crate::identity::DataComponentId::new(d, *index),
                        specialized_type,
                    }
                }
            };
            requirements.insert(req_id, selection);
        }
        let mut bindings = BTreeMap::new();
        for b in &witness.associated_types {
            let req_id = crate::traits::AssociatedTypeRequirementId::new(trait_decl.clone(), b.index);
            let value_template = match b.value {
                AssociatedBindingValueV1::Type { ty } => crate::impls::AssociatedTypeBindingValue::Type(type_relocation.ty(ty)?),
                AssociatedBindingValueV1::Dynamic => crate::impls::AssociatedTypeBindingValue::Dynamic,
            };
            bindings.insert(
                req_id.clone(),
                crate::impls::AssociatedTypeBindingTemplate {
                    requirement: req_id,
                    value_template,
                    source_impl: impl_id.clone(),
                    source: crate::diagnostic::SemanticSourceSpan::new(impl_id.module.clone(), phalcom_common::range::SourceRange::default()),
                },
            );
        }
        let completeness = match witness.completeness {
            0 => crate::impls::ConformanceCompleteness::Complete,
            1 => crate::impls::ConformanceCompleteness::Incomplete { failures: Box::new([]) },
            2 => crate::impls::ConformanceCompleteness::Unknown(crate::types::evidence::UnknownReason::NoTypeEvidence),
            3 => crate::impls::ConformanceCompleteness::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint),
            4 => crate::impls::ConformanceCompleteness::Dynamic(crate::types::outcome::DynamicBoundaryObligation { reason: "dynamic".into() }),
            5 => crate::impls::ConformanceCompleteness::Cancelled,
            6 => {
                crate::impls::ConformanceCompleteness::BudgetExceeded(crate::types::outcome::BudgetReport::new(crate::types::outcome::BudgetKind::Steps, 0, 0))
            }
            _ => crate::impls::ConformanceCompleteness::InternalFailure("restored from artifact".into()),
        };
        let mut associated_type_plan = crate::impls::ConformanceAssociatedTypePlan::empty(impl_id.clone());
        associated_type_plan.bindings = bindings;
        witness_plans_map.insert(
            impl_id.clone(),
            Arc::new(crate::impls::ConformanceWitnessPlan {
                impl_id: impl_id.clone(),
                source_module,
                trait_ref_template,
                target_template,
                generic_signature,
                requirement_views: BTreeMap::new(),
                requirements,
                associated_type_plan,
                fingerprint: crate::db::ProductFingerprint(witness.fingerprint),
                invalid_explicit_members: false,
                completeness,
                diagnostics: Box::new([]),
            }),
        );
    }

    let mut constructor_initializations = crate::signature::ConstructorInitializationTable::default();
    for rec in &world.constructor_initializations {
        let factory = from_stable_callable(&rec.factory).map_err(referential)?;
        let initializer = from_stable_callable(&rec.initializer).map_err(referential)?;
        let origin = match rec.origin {
            0 => crate::signature::ConstructorOrigin::Source,
            1 => crate::signature::ConstructorOrigin::Construct,
            2 => crate::signature::ConstructorOrigin::Data,
            _ => crate::signature::ConstructorOrigin::DefaultOnly,
        };
        let field_input_fingerprint = crate::db::ProductFingerprint(rec.field_input_fingerprint);
        let default_fields: Box<[crate::identity::FieldId]> = rec
            .default_fields
            .iter()
            .map(|name| crate::identity::FieldId::new(initializer.owner.declaration().clone(), name.clone(), crate::identity::DispatchSide::Instance))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let parameter_fields: Box<[crate::signature::ConstructorParameterField]> = rec
            .parameter_fields
            .iter()
            .map(|pf| {
                let side = match pf.field_side {
                    0 => crate::identity::DispatchSide::Instance,
                    _ => crate::identity::DispatchSide::Class,
                };
                crate::signature::ConstructorParameterField {
                    parameter: crate::identity::CallableParameterId::new(factory.clone(), pf.parameter_index),
                    field: crate::identity::FieldId::new(initializer.owner.declaration().clone(), pf.field_name.clone(), side),
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        constructor_initializations.insert(crate::signature::ConstructorInitializationProduct {
            factory,
            initializer,
            origin,
            field_input_fingerprint,
            default_fields,
            parameter_fields,
        });
    }

    let ImportedAssociatedAuthority {
        associated_surfaces,
        type_member_declarations,
        associated_static_domains,
    } = import_associated_authority(
        &world.type_member_declarations,
        &world.associated_static_domains,
        &world.associated_surfaces,
        &type_relocation,
        allocations,
    )?;

    let mut callable_definitions_by_module: BTreeMap<crate::identity::ModuleId, Vec<crate::identity::CallableId>> = BTreeMap::new();
    for (_, sig) in callable_signatures.iter() {
        callable_definitions_by_module
            .entry(sig.callable.owner.module().clone())
            .or_default()
            .push(sig.callable.clone());
    }
    let callable_definitions_by_module: BTreeMap<crate::identity::ModuleId, Box<[crate::identity::CallableId]>> =
        callable_definitions_by_module.into_iter().map(|(m, v)| (m, v.into_boxed_slice())).collect();

    let surfaces = dispatch_resolver.surfaces().clone();
    let mut snapshot = crate::snapshot::SemanticSnapshot {
        id: crate::identity::SnapshotId::new(crate::identity::WorkspaceId(1), crate::identity::SemanticRevision(1), type_store.id()),
        generation: 1,
        store: Arc::new(type_store.clone()),
        sources: Arc::new(BTreeMap::new()),
        presentation_sources: Arc::new(BTreeMap::new()),
        surfaces,
        dispatch: Arc::new(dispatch_resolver),
        callable_signatures: Arc::new(callable_signatures),
        constructor_initializations: Arc::new(constructor_initializations),
        callable_definitions: Arc::new(crate::parent_map::ParentMap::from(&callable_definitions_map)),
        callable_definitions_by_module: Arc::new(callable_definitions_by_module),
        conformance_index: Arc::new(conformance_index),
        conformance_witness_plans: Arc::new(witness_plans_map),
        derived_conformance_rules: Arc::new(crate::derived_conformance::DerivedConformanceRuleIndex::new()),
        trait_dispatch_index: Arc::new(crate::trait_dispatch::TraitDispatchIndex::default()),
        field_signatures: Arc::new(field_signatures),
        declarations: Arc::new(structural_contracts.declaration_table.clone()),
        type_aliases: Arc::new(structural_contracts.type_aliases.clone()),
        trait_headers: Arc::new(trait_headers),
        trait_surfaces: Arc::new(trait_surfaces),
        hierarchy: Arc::new(structural_contracts.hierarchy.clone()),
        diagnostics: Arc::new(BTreeMap::new()),
        semantic_graph: Arc::new(phalcom_modules::graph::SemanticGraph::default()),
        callable_analyses: Arc::new(callable_analyses),
        internal_incidents: Arc::from([]),
        formal_projection: Arc::new(crate::presentation::FormalSemanticProjection::default()),
        source_index: Arc::new(crate::source_index::SourceSemanticIndex::default()),
        advisory: Arc::new(crate::advisory::AdvisoryWorkspace::default()),
        module_products: Arc::new(crate::snapshot::ModuleQueryProducts::empty()),
        data_semantics: Arc::new(structural_contracts.data_table.clone()),
        enum_semantics: Arc::new(structural_contracts.enum_table.clone()),
        enum_requirements: Arc::new(enum_requirements),
        associated_surfaces: Arc::new(associated_surfaces),
        type_member_declarations: Arc::new(type_member_declarations),
        associated_static_domains: Arc::new(associated_static_domains),
        static_case_catalogs: Arc::new(BTreeMap::new()),
        static_case_signatures: Arc::new(BTreeMap::new()),
        static_definitions: Arc::new(crate::static_definition::StaticDefinitionTable::new()),
        static_constants: Arc::new(crate::static_constant::StaticConstantTable::new()),
        static_values: Arc::new(static_values.clone()),
        module_static_bindings: Arc::new(crate::parent_map::ParentMap::default()),
        semantic_structure_shards: Arc::new(BTreeMap::new()),
        // Imported rows are not promoted to Complete until every canonical
        // source-free owner family passes the final completeness check below.
        status: crate::snapshot::SnapshotStatus::Partial { blocked_modules: 0 },
    };

    if !snapshot.has_source_free_artifact_contents() {
        return Err(referential("reconstructed snapshot is not source-free complete"));
    }
    snapshot.status = crate::snapshot::SnapshotStatus::Complete;
    let snapshot = Arc::new(snapshot);

    // Build secondary owner indexes in a single O(N) pass (P7.T8)
    let mut owner_indexes = ReconstructedModuleOwnerIndexes::default();
    for shell in &world.structural_contracts.declaration_shells {
        let key = format!("{:?}", shell.declaration.module);
        owner_indexes.declarations_by_module.entry(key).or_default().push(shell.declaration.clone());
    }
    for data in &world.structural_contracts.data_contracts {
        let key = format!("{:?}", data.owner.module);
        owner_indexes.data_by_module.entry(key).or_default().push(data.owner.clone());
    }
    for enm in &world.structural_contracts.enum_contracts {
        let key = format!("{:?}", enm.owner.module);
        owner_indexes.enums_by_module.entry(key).or_default().push(enm.owner.clone());
    }

    Ok(ValidatedSemanticParts {
        structural_contracts,
        type_store,
        type_relocation,
        static_values,
        static_relocation,
        operational_templates,
        owner_indexes,
        snapshot,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::associated::{AssociatedFamilyInfo, AssociatedFamilyKind, AssociatedFamilyTable, AssociatedMemberId, AssociatedSurface};
    use crate::db::product::{AssociatedStaticDomainDeclarationProduct, TypeMemberDeclarationProduct};
    use crate::identity::{AssociatedFamilyId, AssociatedStaticDomainDeclarationId, CallableOwnerId, DataConstructorId, VariantId};
    use crate::signature::{CallableSemanticSignature, ReturnContractValidation};
    use crate::types::store::artifact::TypeExportContext;
    use crate::types::type_member::{
        IntrinsicTypeOperation, TypeMemberDeclarationId, TypeMemberDomain, TypeMemberParameter, TypeMemberSelector, TypeMemberSignature,
    };
    use phalcom_common::modifier::ImplementationKind;
    use phalcom_common::range::SourceRange;
    use phalcom_common::selector::{Selector, SelectorBase};
    use phalcom_modules::{DeclarationId, ModuleComponent, ModuleId, ModulePath};
    use phalcom_native_meta::{EffectSpec, NativeLifecycleSpec, RaisesSpec, ReturnFlowSpec};
    use phalcom_type_meta::identity::{StableCallableOwnerRef, StableCallableRef, StableDispatchSide, StableModuleRef, StableProjectRef};

    fn round_trip_denotation(
        denotation: &crate::types::denotation::SemanticDenotation,
        store: &TypeStore,
        type_roots: &[crate::types::id::TypeId],
    ) -> (crate::types::denotation::SemanticDenotation, usize, Vec<crate::types::id::TypeId>) {
        use phalcom_compiler_artifact::semantic::type_store::{CanonicalRootFamilyTag, CanonicalTypeRootKey, StableOwnerRef};

        let root_callable = crate::identity::CallableId::method(owner(), Selector::getter("artifact_denotation_root").unwrap(), DispatchSide::Class);
        let mut types = TypeExportContext::new(store);
        let lineage = types.expected_lineage();
        for (index, ty) in type_roots.iter().copied().enumerate() {
            types
                .register_type_root(
                    lineage,
                    CanonicalTypeRootKey {
                        family_tag: CanonicalRootFamilyTag::CallableAnalysis,
                        owner: StableOwnerRef::Callable(crate::metadata::stable_identity::to_stable_callable(&root_callable)),
                        product_subkey: index as u32,
                    },
                    ty,
                )
                .unwrap();
        }
        types.close_graph(store, &ValidationLimits::default()).unwrap();
        let portable = crate::artifact::export::export_semantic_denotation(denotation, &types).unwrap();
        let graph = types.project_graph(store, &ValidationLimits::default()).unwrap();
        let (_loaded_store, relocation) = TypeStore::import_type_graph(&graph, &ValidationLimits::default()).unwrap();
        let relocated_roots = type_roots.iter().map(|&ty| relocation.ty(types.ty(ty).unwrap()).unwrap()).collect::<Vec<_>>();
        let mut allocations = AllocationBudget::new(1 << 20);
        let imported = import_semantic_denotation(&portable, &relocation, &mut allocations).unwrap();
        (imported, allocations.requested(), relocated_roots)
    }

    #[test]
    fn callable_type_knowledge_import_budgets_owned_names_and_rolls_back() {
        let source_name = "unresolved-type-name";
        let portable = phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
            reason: UnknownReasonV1::UnresolvedName { name: source_name.into() },
        };
        let store = TypeStore::new();
        let relocation = TypeRelocation {
            store_id: store.id(),
            types: Vec::new(),
            kinds: Vec::new(),
            parameters: Vec::new(),
            rows: Vec::new(),
            families: Vec::new(),
            variants: Vec::new(),
            domains: Vec::new(),
            constructors: Vec::new(),
            scoped: Vec::new(),
            lambdas: Vec::new(),
        };

        let mut exact = AllocationBudget::new(source_name.len());
        let imported = import_type_knowledge_with_allocation_budget(&portable, &relocation, &mut exact).unwrap();
        assert_eq!(exact.requested(), source_name.len());
        assert_eq!(
            imported,
            crate::types::evidence::TypeKnowledge::Unknown(UnknownReason::UnresolvedName(source_name.into()))
        );

        let mut insufficient = AllocationBudget::new(source_name.len() - 1);
        let error = import_type_knowledge_with_allocation_budget(&portable, &relocation, &mut insufficient).unwrap_err();
        assert!(matches!(error, ArtifactError::Allocation(AllocationError::Exhausted)));
        assert_eq!(insufficient.requested(), 0, "failed nested reconstruction must not debit the caller");
    }

    #[test]
    fn associated_denotations_round_trip_optional_targets_and_captured_member_sequence() {
        use crate::types::denotation::{AssociatedValueDenotation, CapturedAssociatedMember, SemanticDenotation};
        use crate::types::family::FamilyOperationShape;
        use phalcom_common::selector::{SelectorBase, SelectorSlot};
        use std::sync::Arc;

        let owner = owner();
        let mut store = TypeStore::new();
        let owner_form = store.nominal(owner.clone());
        let variant = VariantId::new(owner.clone(), Selector::getter("Ready").unwrap());
        let exact_without_target = SemanticDenotation::AssociatedValue(Box::new(AssociatedValueDenotation::exact(
            owner_form,
            owner.clone(),
            AssociatedMemberId::Variant(variant.clone()),
            None,
        )));
        let (restored, requested, relocated_roots) = round_trip_denotation(&exact_without_target, &store, &[owner_form]);
        let SemanticDenotation::AssociatedValue(restored) = restored else {
            panic!("associated exact denotation")
        };
        let AssociatedValueDenotation::Exact {
            owner_form: restored_owner,
            member,
            target,
            ..
        } = &*restored
        else {
            panic!("associated exact payload")
        };
        assert_eq!(*restored_owner, relocated_roots[0]);
        assert_eq!(member, &AssociatedMemberId::Variant(variant.clone()));
        assert_eq!(target.as_ref(), &None);
        assert!(requested > 0, "the owned exact-target box is charged");

        let exact_with_target = SemanticDenotation::AssociatedValue(Box::new(AssociatedValueDenotation::exact(
            owner_form,
            owner.clone(),
            AssociatedMemberId::DataConstructor(DataConstructorId::new(owner.clone())),
            Some(crate::identity::InvocationTargetId::DataConstructor(DataConstructorId::new(owner.clone()))),
        )));
        let (restored, _, relocated_roots) = round_trip_denotation(&exact_with_target, &store, &[owner_form]);
        let SemanticDenotation::AssociatedValue(restored) = restored else {
            panic!("associated exact denotation")
        };
        let AssociatedValueDenotation::Exact {
            owner_form: restored_owner,
            target,
            ..
        } = &*restored
        else {
            panic!("associated exact payload")
        };
        assert_eq!(*restored_owner, relocated_roots[0]);
        assert_eq!(
            target.as_ref(),
            &Some(crate::identity::InvocationTargetId::DataConstructor(DataConstructorId::new(owner.clone())))
        );

        let captured = vec![
            CapturedAssociatedMember {
                operation: FamilyOperationShape::getter(),
                member: AssociatedMemberId::Variant(variant.clone()),
                target: None,
            },
            CapturedAssociatedMember {
                operation: FamilyOperationShape::method(vec![SelectorSlot::Label("payload".into())].into_boxed_slice()),
                member: AssociatedMemberId::DataConstructor(DataConstructorId::new(owner.clone())),
                target: Some(crate::identity::InvocationTargetId::DataConstructor(DataConstructorId::new(owner.clone()))),
            },
        ];
        let family_id = AssociatedFamilyId::new(owner.clone(), SelectorBase::Named("selected".into()));
        let subset = SemanticDenotation::AssociatedValue(Box::new(AssociatedValueDenotation::Family {
            owner_form,
            lookup_owner: owner.clone(),
            family: family_id,
            members: Arc::from([captured[1].clone()]),
        }));
        let (restored, requested, relocated_roots) = round_trip_denotation(&subset, &store, &[owner_form]);
        let SemanticDenotation::AssociatedValue(restored) = restored else {
            panic!("associated family denotation")
        };
        let AssociatedValueDenotation::Family {
            owner_form: restored_owner,
            members,
            ..
        } = &*restored
        else {
            panic!("associated family payload")
        };
        assert_eq!(*restored_owner, relocated_roots[0]);
        assert_eq!(
            members.as_ref(),
            &[captured[1].clone()],
            "the selected subset and its source order are denotation-owned"
        );
        assert!(requested > 0, "captured member buffers are charged");
    }

    #[test]
    fn behavioral_family_round_trip_preserves_specs_targets_and_optional_selections() {
        use crate::checker::associated::BehavioralFamilySpec;
        use crate::types::denotation::{BehavioralFamilyDenotation, CapturedBehavioralMember, SemanticDenotation};
        use crate::types::family::FamilyOperationShape;
        use phalcom_common::selector::{SelectorKindPattern, SelectorSlot};
        use std::sync::Arc;

        let owner = owner();
        let callable = crate::identity::CallableId::method(
            owner.clone(),
            Selector::method("run", [SelectorSlot::Positional]).unwrap(),
            DispatchSide::Instance,
        );
        let impl_id = crate::identity::ImplId::new(owner.module.clone(), crate::identity::ImplLocalId(9));
        let conditional = crate::dispatch::ConditionalDispatchSelection {
            impl_id,
            callable: callable.clone(),
            declaring_owner: owner.clone(),
            side: DispatchSide::Instance,
        };
        let mut store = TypeStore::new();
        let receiver_type = store.nominal_type(owner.clone());
        let constructor_kind = store.arrow_kind(vec![crate::types::id::KindId::TYPE].into_boxed_slice(), crate::types::id::KindId::TYPE);
        let receiver_form = store.nominal_form(owner.clone(), constructor_kind);
        let receiver_application = store
            .apply_static_form(receiver_form, &[crate::types::store::StaticArgument::Term(receiver_type)])
            .unwrap();
        let spec = BehavioralFamilySpec::Pattern(
            phalcom_common::selector::SelectorPattern::named(
                "run",
                SelectorKindPattern::Exact(phalcom_common::selector::SelectorKind::Method),
                Vec::<SelectorSlot>::new().into_boxed_slice(),
                Vec::<SelectorSlot>::new().into_boxed_slice(),
                true,
            )
            .unwrap(),
        );
        let source = SemanticDenotation::BehavioralFamily(Box::new(BehavioralFamilyDenotation {
            receiver_type,
            receiver_application: Some(receiver_application),
            spec,
            members: Arc::from([
                CapturedBehavioralMember {
                    operation: FamilyOperationShape::method(vec![SelectorSlot::Positional].into_boxed_slice()),
                    target: crate::identity::InvocationTargetId::Behavioral(callable.clone()),
                    conditional: None,
                },
                CapturedBehavioralMember {
                    operation: FamilyOperationShape::method(vec![SelectorSlot::Positional].into_boxed_slice()),
                    target: crate::identity::InvocationTargetId::Behavioral(callable.clone()),
                    conditional: Some(conditional.clone()),
                },
            ]),
        }));
        let (restored, requested, relocated_roots) = round_trip_denotation(&source, &store, &[receiver_type, receiver_application]);
        let SemanticDenotation::BehavioralFamily(restored) = restored else {
            panic!("behavioral-family denotation")
        };
        assert_eq!(restored.receiver_type, relocated_roots[0]);
        assert_eq!(restored.receiver_application, Some(relocated_roots[1]));
        assert_eq!(
            restored.spec,
            match &source {
                SemanticDenotation::BehavioralFamily(family) => family.spec.clone(),
                _ => unreachable!(),
            }
        );
        assert_eq!(restored.members[0].conditional, None);
        assert_eq!(restored.members[1].conditional.as_ref(), Some(&conditional));
        assert!(requested > 0, "captured behavioral members and publication storage are charged");
    }

    #[test]
    fn callable_denotation_import_preserves_ordinals_and_commits_only_complete_records() {
        use crate::types::denotation::{AssociatedValueDenotation, SemanticDenotation};
        use crate::types::id::{TypeId, TypeStoreId};
        use phalcom_compiler_artifact::module_world::SpanV1;
        use phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId;
        use phalcom_compiler_artifact::semantic::world::{
            AssociatedFamilyIdentityV1, AssociatedMemberV1, AssociatedValueDenotationV1, CallableAnalysisRecordV1, CallableAnalysisStatusV1,
            CallableBindingDenotationV1, CallableBindingOriginEntryV1, CallableBindingOriginV1, CallableExpressionDenotationV1, CallableFlowGraphV1,
            FamilyOperationShapeV1, InvocationTargetV1, SelectorKindV1, SelectorSlotV1, SemanticDenotationV1,
        };

        let owner = owner();
        let callable = crate::identity::CallableId::method(owner.clone(), Selector::method("analyze", []).unwrap(), DispatchSide::Instance);
        let variant = VariantId::new(owner.clone(), Selector::getter("Ready").unwrap());
        let stable_owner = crate::metadata::stable_identity::to_stable_declaration(&owner);
        let mut record = CallableAnalysisRecordV1 {
            callable: crate::metadata::stable_identity::to_stable_callable(&callable),
            body_source_owner: None,
            body_range: SpanV1 { start: 0, end: 3 },
            status: CallableAnalysisStatusV1::Complete,
            exits_unreachable: false,
            flow_graph: CallableFlowGraphV1 {
                nodes: Vec::new(),
                edges: Vec::new(),
                entry: None,
                exits: Vec::new(),
            },
            constructor_super_initializer: None,
            expression_count: 3,
            expression_denotations: vec![CallableExpressionDenotationV1 {
                expression_ordinal: 2,
                denotation: SemanticDenotationV1::TypeForm { ty: ArtifactTypeId(0) },
            }],
            binding_denotations: vec![
                CallableBindingDenotationV1 {
                    binding_ordinal: 0,
                    denotation: SemanticDenotationV1::AssociatedValue {
                        value: AssociatedValueDenotationV1::Exact {
                            owner_form: ArtifactTypeId(0),
                            lookup_owner: stable_owner.clone(),
                            member: AssociatedMemberV1::Variant {
                                variant: crate::metadata::stable_identity::to_stable_variant(&variant),
                            },
                            target: None,
                        },
                    },
                },
                CallableBindingDenotationV1 {
                    binding_ordinal: 1,
                    denotation: SemanticDenotationV1::AssociatedValue {
                        value: AssociatedValueDenotationV1::Family {
                            owner_form: ArtifactTypeId(0),
                            lookup_owner: stable_owner.clone(),
                            family: AssociatedFamilyIdentityV1 {
                                owner: stable_owner.clone(),
                                selector: TypeMemberSelectorV1::Named { name: "selected".into() },
                            },
                            members: vec![phalcom_compiler_artifact::semantic::world::CapturedAssociatedMemberV1 {
                                operation: FamilyOperationShapeV1 {
                                    kind: SelectorKindV1::Method,
                                    slots: vec![SelectorSlotV1::Label { label: "payload".into() }],
                                },
                                member: AssociatedMemberV1::DataConstructor { owner: stable_owner.clone() },
                                target: Some(InvocationTargetV1::DataConstructor { owner: stable_owner.clone() }),
                            }],
                        },
                    },
                },
            ],
            call_specializations: Vec::new(),
            formal_bindings: Vec::new(),
            binding_origins: vec![
                CallableBindingOriginEntryV1 {
                    binding_ordinal: 0,
                    origin: CallableBindingOriginV1::BodyLocal,
                },
                CallableBindingOriginEntryV1 {
                    binding_ordinal: 1,
                    origin: CallableBindingOriginV1::BodyLocal,
                },
            ],
        };
        let relocation = TypeRelocation {
            store_id: TypeStoreId(0),
            types: vec![TypeId(81)],
            kinds: Vec::new(),
            parameters: Vec::new(),
            rows: Vec::new(),
            families: Vec::new(),
            variants: Vec::new(),
            domains: Vec::new(),
            constructors: Vec::new(),
            scoped: Vec::new(),
            lambdas: Vec::new(),
        };

        let expected = crate::types::denotation::SemanticDenotation::AssociatedValue(Box::new(AssociatedValueDenotation::exact(
            TypeId(81),
            owner.clone(),
            crate::associated::AssociatedMemberId::Variant(variant),
            None,
        )));
        let expected_family = crate::types::denotation::SemanticDenotation::AssociatedValue(Box::new(AssociatedValueDenotation::Family {
            owner_form: TypeId(81),
            lookup_owner: owner.clone(),
            family: crate::identity::AssociatedFamilyId::new(owner.clone(), phalcom_common::selector::SelectorBase::Named("selected".into())),
            members: Arc::from([crate::types::denotation::CapturedAssociatedMember {
                operation: crate::types::family::FamilyOperationShape::method(
                    vec![phalcom_common::selector::SelectorSlot::Label("payload".into())].into_boxed_slice(),
                ),
                member: crate::associated::AssociatedMemberId::DataConstructor(crate::identity::DataConstructorId::new(owner.clone())),
                target: Some(crate::identity::InvocationTargetId::DataConstructor(crate::identity::DataConstructorId::new(
                    owner.clone(),
                ))),
            }]),
        }));
        let mut measured = AllocationBudget::new(1 << 20);
        let imported = import_callable_denotations(&record, &relocation, &mut measured).unwrap();
        let required = measured.requested();
        assert!(required > 0, "owner vectors and the captured associated value are charged");
        assert_eq!(imported.expressions, vec![(2, SemanticDenotation::TypeForm(TypeId(81)))]);
        assert_eq!(imported.bindings, vec![(0, expected), (1, expected_family)]);

        let mut exact = AllocationBudget::new(required);
        import_callable_denotations(&record, &relocation, &mut exact).unwrap();
        assert_eq!(exact.requested(), required);

        let mut insufficient = AllocationBudget::new(required - 1);
        let error = import_callable_denotations(&record, &relocation, &mut insufficient).unwrap_err();
        assert!(matches!(error, ArtifactError::Allocation(AllocationError::Exhausted)));
        assert_eq!(insufficient.requested(), 0, "a rejected owner graph does not debit the caller");

        record.binding_origins.push(CallableBindingOriginEntryV1 {
            binding_ordinal: 2,
            origin: CallableBindingOriginV1::BodyLocal,
        });
        record.binding_denotations.push(CallableBindingDenotationV1 {
            binding_ordinal: 2,
            denotation: SemanticDenotationV1::StaticApplication {
                body: stable_owner,
                ordinal: 0,
            },
        });
        let mut rejected = AllocationBudget::new(1 << 20);
        assert!(import_callable_denotations(&record, &relocation, &mut rejected).is_err());
        assert_eq!(rejected.requested(), 0, "a later fail-closed denotation rolls back earlier owner allocations");
    }

    fn owner() -> DeclarationId {
        let path = ModulePath::from_components(vec![ModuleComponent::from_identifier("artifact_parity").unwrap()]);
        DeclarationId::new(ModuleId::universe(path), "Packet".into())
    }

    fn signature(callable: crate::identity::CallableId, owner: &DeclarationId) -> CallableSemanticSignature {
        CallableSemanticSignature {
            owner: owner.clone(),
            side: callable.side,
            selector: callable.selector.clone(),
            callable,
            generics: None,
            parameters: Box::new([]),
            declared_return: crate::declaration_type::DeclaredTypeFact::unknown(UnknownReason::NoTypeEvidence),
            return_validation: ReturnContractValidation::NotApplicable,
            inferred_return: None,
            source: None,
            implementation: ImplementationKind::Phalcom,
            native_id: None,
            effects: EffectSpec::Unknown,
            raises: RaisesSpec::Unknown,
            flow: ReturnFlowSpec::Value,
            lifecycle: NativeLifecycleSpec::UNKNOWN,
        }
    }

    #[test]
    fn typed_call_specialization_arguments_survive_relocation() {
        use crate::types::store::artifact::TypeRelocation;
        use phalcom_compiler_artifact::semantic::type_store::{ArtifactRecordRowId, ArtifactTypeId, ArtifactTypeParameterId, StaticArgumentV1};
        use phalcom_compiler_artifact::semantic::world::{CallableCallSpecializationV1, CallableStaticArgumentBindingV1};

        let owner = owner();
        let callable = crate::identity::CallableId::new(
            CallableOwnerId::Declaration(owner),
            Selector::method("ping", []).unwrap(),
            crate::identity::DispatchSide::Instance,
        );
        let record = CallableCallSpecializationV1 {
            expression_ordinal: 3,
            callable: crate::metadata::stable_identity::to_stable_callable(&callable),
            bindings: vec![
                CallableStaticArgumentBindingV1 {
                    parameter: ArtifactTypeParameterId(0),
                    argument: StaticArgumentV1::DynamicType,
                },
                CallableStaticArgumentBindingV1 {
                    parameter: ArtifactTypeParameterId(1),
                    argument: StaticArgumentV1::Term { term: ArtifactTypeId(0) },
                },
                CallableStaticArgumentBindingV1 {
                    parameter: ArtifactTypeParameterId(2),
                    argument: StaticArgumentV1::RecordRow { row: ArtifactRecordRowId(0) },
                },
            ],
        };
        let relocated_parameters = [
            crate::types::id::TypeParameterId(71),
            crate::types::id::TypeParameterId(72),
            crate::types::id::TypeParameterId(73),
        ];
        let relocated_type = crate::types::id::TypeId(81);
        let relocated_row = crate::types::id::RecordRowId(91);
        let relocation = TypeRelocation {
            store_id: crate::types::id::TypeStoreId(0),
            types: vec![relocated_type],
            kinds: vec![],
            parameters: relocated_parameters.into(),
            rows: vec![relocated_row],
            families: vec![],
            variants: vec![],
            domains: vec![],
            constructors: vec![],
            scoped: vec![],
            lambdas: vec![],
        };
        let mut allocations = AllocationBudget::new(1024);
        let restored = import_callable_call_specializations(4, &[record], &relocation, &mut allocations).unwrap();
        assert_eq!(restored[0].0, 3);
        assert_eq!(restored[0].1.callable, callable);
        assert_eq!(
            restored[0].1.bindings.as_ref(),
            &[
                (relocated_parameters[0], crate::types::store::StaticArgument::DynamicType),
                (relocated_parameters[1], crate::types::store::StaticArgument::Term(relocated_type)),
                (relocated_parameters[2], crate::types::store::StaticArgument::RecordRow(relocated_row)),
            ]
        );
        assert!(import_callable_call_specializations(3, &[], &relocation, &mut allocations).is_ok());
    }

    #[test]
    fn enum_requirements_round_trip_source_order_and_all_case_statuses() {
        let owner = owner();
        let mut store = TypeStore::new();
        let owner_type = store.nominal(owner.clone());
        let lane = store.type_pack(vec![owner_type].into_boxed_slice()).unwrap();
        let area = crate::identity::CallableId::new(
            CallableOwnerId::Declaration(owner.clone()),
            Selector::method("area", []).unwrap(),
            crate::identity::DispatchSide::Instance,
        );
        let width = crate::identity::CallableId::new(
            CallableOwnerId::Declaration(owner.clone()),
            Selector::method("width", []).unwrap(),
            crate::identity::DispatchSide::Instance,
        );
        let circle = VariantId::new(owner.clone(), Selector::getter("Circle").unwrap());
        let square = VariantId::new(owner.clone(), Selector::getter("Square").unwrap());
        let circle_area = crate::identity::CallableId::new(
            CallableOwnerId::Variant(circle.clone()),
            Selector::method("area", []).unwrap(),
            crate::identity::DispatchSide::Instance,
        );
        let circle_width = crate::identity::CallableId::new(
            CallableOwnerId::Variant(circle.clone()),
            Selector::method("width", []).unwrap(),
            crate::identity::DispatchSide::Instance,
        );

        let area_signature = signature(area.clone(), &owner);
        let fixed_fact = crate::declaration_type::DeclaredTypeFact::known(
            crate::types::parameter::TypeTerm::Canonical(owner_type),
            crate::declaration_type::DeclaredTypeBasis::SourceAnnotation,
        );
        let lane_declared = crate::declaration_type::DeclaredTypeFact::unknown(UnknownReason::NoTypeEvidence);
        let invalid_reason = UnknownReason::InferenceBlocked;
        let parameter_source = |start, end| Some(SemanticSourceSpan::new(owner.module.clone(), SourceRange { start, end }));
        let width_signature_source = SemanticSourceSpan::new(owner.module.clone(), SourceRange { start: 3, end: 10 });
        let mut width_signature = signature(width.clone(), &owner);
        width_signature.parameters = vec![
            crate::signature::CallableParameterSemantic {
                id: crate::identity::CallableParameterId::new(width.clone(), 0),
                local_name: "value".into(),
                external_label: Some("value".into()),
                rest: phalcom_ast::ast::RestMode::None,
                declared_type: fixed_fact.clone(),
                contract: crate::signature::ParameterContract::Fixed(fixed_fact.clone()),
                source: parameter_source(11, 16),
            },
            crate::signature::CallableParameterSemantic {
                id: crate::identity::CallableParameterId::new(width.clone(), 1),
                local_name: "items".into(),
                external_label: None,
                rest: phalcom_ast::ast::RestMode::Positional,
                declared_type: fixed_fact.clone(),
                contract: crate::signature::ParameterContract::Rest {
                    mode: phalcom_ast::ast::RestMode::Positional,
                    contract: crate::signature::RestValueContract::Element(fixed_fact.clone()),
                },
                source: parameter_source(17, 23),
            },
            crate::signature::CallableParameterSemantic {
                id: crate::identity::CallableParameterId::new(width.clone(), 2),
                local_name: "named".into(),
                external_label: Some("named".into()),
                rest: phalcom_ast::ast::RestMode::Labeled,
                declared_type: lane_declared.clone(),
                contract: crate::signature::ParameterContract::Rest {
                    mode: phalcom_ast::ast::RestMode::Labeled,
                    contract: crate::signature::RestValueContract::Lane(lane),
                },
                source: parameter_source(24, 30),
            },
            crate::signature::CallableParameterSemantic {
                id: crate::identity::CallableParameterId::new(width.clone(), 3),
                local_name: "all".into(),
                external_label: None,
                rest: phalcom_ast::ast::RestMode::Complete,
                declared_type: crate::declaration_type::DeclaredTypeFact::unknown(invalid_reason.clone()),
                contract: crate::signature::ParameterContract::Rest {
                    mode: phalcom_ast::ast::RestMode::Complete,
                    contract: crate::signature::RestValueContract::Invalid(invalid_reason),
                },
                source: parameter_source(31, 35),
            },
        ]
        .into_boxed_slice();
        width_signature.declared_return = fixed_fact;
        width_signature.return_validation = ReturnContractValidation::Satisfied(crate::types::evidence::EvidenceStatus::Assumed);
        width_signature.inferred_return = Some(
            crate::types::evidence::TypeKnowledge::established(owner_type, crate::types::evidence::EvidenceOrigin::Flow)
                .with_range(SourceRange { start: 36, end: 41 }),
        );
        width_signature.source = Some(width_signature_source);
        width_signature.flow = ReturnFlowSpec::Argument(1);

        let source_range = SourceRange { start: 11, end: 19 };
        let requirements: Arc<[EnumRequirement]> = Arc::from(
            vec![
                EnumRequirement {
                    id: crate::enum_requirements::EnumRequirementId::new(owner.clone(), width.selector.clone()),
                    signature: width_signature,
                    source: Some(SemanticSourceSpan::new(owner.module.clone(), source_range)),
                },
                EnumRequirement {
                    id: crate::enum_requirements::EnumRequirementId::new(owner.clone(), area.selector.clone()),
                    signature: area_signature,
                    source: None,
                },
            ]
            .into_boxed_slice(),
        );
        let case_statuses: Arc<[CaseRequirementResult]> = Arc::from(
            vec![
                CaseRequirementResult {
                    variant: circle.clone(),
                    requirement: requirements[0].id.clone(),
                    status: CaseRequirementStatus::Satisfied {
                        implementation: circle_width.clone(),
                    },
                },
                CaseRequirementResult {
                    variant: square.clone(),
                    requirement: requirements[0].id.clone(),
                    status: CaseRequirementStatus::Missing,
                },
                CaseRequirementResult {
                    variant: circle.clone(),
                    requirement: requirements[1].id.clone(),
                    status: CaseRequirementStatus::Incompatible {
                        implementation: circle_area.clone(),
                    },
                },
                CaseRequirementResult {
                    variant: square.clone(),
                    requirement: requirements[1].id.clone(),
                    status: CaseRequirementStatus::Blocked,
                },
            ]
            .into_boxed_slice(),
        );
        let mut source_table = EnumRequirementTable::new();
        source_table.insert(owner.clone(), requirements.clone(), case_statuses.clone());

        let unit = store.unit();
        let mut enum_semantics = EnumSemanticTable::new();
        enum_semantics.insert_enum(Arc::new(crate::enum_semantics::EnumInfo {
            owner: owner.clone(),
            root_form: unit,
            generic_signature: None,
            static_data: None,
            default_result_type: unit,
            variants: vec![circle.clone(), square.clone()].into_boxed_slice(),
            variant_families: Box::new([]),
            source: None,
        }));

        let mut export_types = TypeExportContext::new(&store);
        crate::artifact::roots::register_callable_signature_roots(&mut export_types, &requirements[0].signature).unwrap();
        crate::artifact::roots::register_callable_signature_roots(&mut export_types, &requirements[1].signature).unwrap();
        export_types.close_graph(&store, &ValidationLimits::default()).unwrap();
        let type_graph = export_types.project_graph(&store, &ValidationLimits::default()).unwrap();
        let portable = crate::artifact::export::export_enum_requirements(&source_table, &enum_semantics, &export_types).unwrap();
        assert_eq!(portable[0].requirements[0].requirement.selector.as_ref(), "area()");
        assert_eq!(portable[0].requirements[0].source_order, 1);
        assert_eq!(portable[0].requirements[1].source_order, 0);
        let (restored_store, relocation) = TypeStore::import_type_graph(&type_graph, &ValidationLimits::default()).unwrap();
        let mut allocation_budget = AllocationBudget::new(usize::MAX);
        let restored = import_enum_requirements(&portable, &enum_semantics, &relocation, &restored_store, &mut allocation_budget).unwrap();
        let required = allocation_budget.requested();
        assert!(required > 0);

        let mut exact_budget = AllocationBudget::new(required);
        import_enum_requirements(&portable, &enum_semantics, &relocation, &restored_store, &mut exact_budget).unwrap();
        assert_eq!(
            exact_budget.requested(),
            required,
            "budgeted vector rows and Arc owners fit at the measured boundary"
        );

        let mut short_budget = AllocationBudget::new(required - 1);
        let error = import_enum_requirements(&portable, &enum_semantics, &relocation, &restored_store, &mut short_budget).unwrap_err();
        assert!(matches!(error, ArtifactError::Allocation(AllocationError::Exhausted)));

        let mut restored_types = TypeExportContext::new(&restored_store);
        for requirement in restored.requirements.get(&owner).unwrap().iter() {
            crate::artifact::roots::register_callable_signature_roots(&mut restored_types, &requirement.signature).unwrap();
        }
        restored_types.close_graph(&restored_store, &ValidationLimits::default()).unwrap();
        let restored_portable = crate::artifact::export::export_enum_requirements(&restored, &enum_semantics, &restored_types).unwrap();
        assert_eq!(restored_portable, portable, "every owner-held callable signature field survives relocation");
        assert_eq!(restored.case_statuses.get(&owner).unwrap().as_ref(), case_statuses.as_ref());
        assert_eq!(allocation_budget.requested(), required);
    }

    #[test]
    fn callable_native_metadata_must_match_the_trusted_catalog_row() {
        let owner = owner();
        let callable = crate::identity::CallableId::new(
            CallableOwnerId::Declaration(owner.clone()),
            Selector::method("native", []).unwrap(),
            crate::identity::DispatchSide::Instance,
        );
        let native = phalcom_native_surface::NATIVE_SURFACES.first().expect("native catalog has a row");
        let mut signature = signature(callable, &owner);
        signature.implementation = ImplementationKind::Native;
        signature.native_id = Some(native.id());
        signature.effects = native.effects();
        signature.raises = native.raises();
        signature.flow = native.flow();
        signature.lifecycle = native.lifecycle();

        let store = TypeStore::new();
        let types = TypeExportContext::new(&store);
        let portable = crate::artifact::export::export_callable_signature(&signature, &types).unwrap();
        assert!(import_native_metadata(&portable).is_ok());

        let mut tampered = portable;
        tampered.effects = if matches!(tampered.effects, phalcom_compiler_artifact::semantic::world::EffectSpecV1::Unknown) {
            phalcom_compiler_artifact::semantic::world::EffectSpecV1::Pure
        } else {
            phalcom_compiler_artifact::semantic::world::EffectSpecV1::Unknown
        };
        assert!(
            import_native_metadata(&tampered).is_err(),
            "catalog identity cannot authorize mismatched native metadata"
        );
    }

    #[test]
    fn callable_analysis_import_rejects_records_without_complete_body_facts() {
        let callable = StableCallableRef {
            owner: StableCallableOwnerRef::Declaration(StableDeclarationRef {
                module: StableModuleRef {
                    project: StableProjectRef::Builtin {
                        namespace: "universe".into(),
                        version: "0.1.0".into(),
                    },
                    path: vec!["test".into()].into_boxed_slice(),
                },
                path: vec!["Owner".into()].into_boxed_slice(),
            }),
            side: StableDispatchSide::Instance,
            selector: "method".into(),
        };
        let record = phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV1 {
            callable,
            body_source_owner: None,
            body_range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 8 },
            status: phalcom_compiler_artifact::semantic::world::CallableAnalysisStatusV1::Complete,
            exits_unreachable: false,
            flow_graph: phalcom_compiler_artifact::semantic::world::CallableFlowGraphV1 {
                nodes: Vec::new(),
                edges: Vec::new(),
                entry: None,
                exits: Vec::new(),
            },
            constructor_super_initializer: None,
            expression_count: 0,
            expression_denotations: Vec::new(),
            binding_denotations: Vec::new(),
            call_specializations: Vec::new(),
            formal_bindings: Vec::new(),
            binding_origins: Vec::new(),
        };

        let error = ensure_complete_callable_analysis_payload(&[record]).expect_err("partial analysis payload must fail closed");
        assert!(error.to_string().contains("callable analysis body facts are not completely represented"));
        assert!(ensure_complete_callable_analysis_payload(&[]).is_ok());
    }

    fn insert_family(
        surface: &mut AssociatedSurface,
        owner: &DeclarationId,
        base: SelectorBase,
        kind: AssociatedFamilyKind,
        members: Box<[AssociatedMemberId]>,
    ) {
        surface.families.insert(
            base.clone(),
            AssociatedFamilyInfo {
                id: AssociatedFamilyId::new(owner.clone(), base),
                kind,
                members,
            },
        );
    }

    #[test]
    fn associated_authority_round_trips_through_type_relocation() {
        let owner = owner();
        let mut store = TypeStore::new();
        let unit = store.unit();

        let member_id = TypeMemberDeclarationId::new(owner.clone(), TypeMemberSelector::named("Map"));
        let member_product = Arc::new(TypeMemberDeclarationProduct {
            id: member_id.clone(),
            signature: TypeMemberSignature {
                selector: member_id.selector.clone(),
                parameters: Box::new([
                    TypeMemberParameter {
                        name: "mapper".into(),
                        domain: TypeMemberDomain::Kind(crate::types::id::KindId::TYPE),
                    },
                    TypeMemberParameter {
                        name: "fallback".into(),
                        domain: TypeMemberDomain::ValueType(unit),
                    },
                ]),
                result_domain: TypeMemberDomain::Kind(crate::types::id::KindId::TYPE),
                intrinsic: Some(IntrinsicTypeOperation::RecordMap),
                implementation: ImplementationKind::Intrinsic,
            },
            source_range: SourceRange::default(),
        });
        let type_members = [(member_id.clone(), member_product)]
            .into_iter()
            .collect::<crate::parent_map::ParentMap<_, _>>();

        let static_id = AssociatedStaticDomainDeclarationId::new(owner.clone(), "State");
        let static_domain = store.associated_static_domain(static_id.clone(), Box::new([]));
        let static_constructor = store.intern_static_constructor(static_domain, owner.clone(), "Ready", Box::new([]));
        let static_product = Arc::new(AssociatedStaticDomainDeclarationProduct {
            id: static_id.clone(),
            domain: static_domain,
            constructors: Box::new([static_constructor]),
            source_range: SourceRange::default(),
        });
        let static_domains = [(static_id.clone(), static_product)].into_iter().collect::<BTreeMap<_, _>>();

        let mut surface = AssociatedSurface::new(owner.clone());
        let variant = VariantId::new(owner.clone(), Selector::getter("Ready").unwrap());
        insert_family(
            &mut surface,
            &owner,
            SelectorBase::Named("Ready".into()),
            AssociatedFamilyKind::Variant,
            Box::new([AssociatedMemberId::Variant(variant)]),
        );
        insert_family(
            &mut surface,
            &owner,
            SelectorBase::Named("Construct".into()),
            AssociatedFamilyKind::DataConstructor,
            Box::new([AssociatedMemberId::DataConstructor(DataConstructorId::new(owner.clone()))]),
        );
        insert_family(
            &mut surface,
            &owner,
            SelectorBase::Named("State".into()),
            AssociatedFamilyKind::StaticDomain,
            Box::new([AssociatedMemberId::StaticDomain(static_id.clone())]),
        );
        insert_family(
            &mut surface,
            &owner,
            SelectorBase::Named("Map".into()),
            AssociatedFamilyKind::IntrinsicTypeMember,
            Box::new([AssociatedMemberId::TypeMember(member_id.clone())]),
        );
        let mut surfaces = AssociatedFamilyTable::new();
        surfaces.insert(owner.clone(), Arc::new(surface));

        let type_export = TypeExportContext::new_full(&store).unwrap();
        let exported = crate::artifact::export::export_associated_authority(&type_members, &static_domains, &surfaces, &type_export).unwrap();
        let graph = type_export.project_graph(&store, &ValidationLimits::default()).unwrap();
        let (_imported_store, relocation) = TypeStore::import_type_graph(&graph, &ValidationLimits::default()).unwrap();

        let imported = import_associated_authority(
            &exported.type_member_declarations,
            &exported.associated_static_domains,
            &exported.associated_surfaces,
            &relocation,
            &mut AllocationBudget::new(usize::MAX),
        )
        .unwrap();

        let imported_member = imported.type_member_declarations.get(&member_id).unwrap();
        assert_eq!(imported_member.id, member_id);
        assert_eq!(imported_member.signature.selector, TypeMemberSelector::named("Map"));
        assert_eq!(imported_member.signature.intrinsic, Some(IntrinsicTypeOperation::RecordMap));
        assert_eq!(imported_member.signature.implementation, ImplementationKind::Intrinsic);
        assert_eq!(
            imported_member.signature.parameters[1].domain,
            TypeMemberDomain::ValueType(relocation.ty(type_export.ty(unit).unwrap()).unwrap())
        );

        let imported_static = imported.associated_static_domains.get(&static_id).unwrap();
        assert_eq!(imported_static.id, static_id);
        assert_eq!(imported_static.domain, relocation.domain(type_export.domain(static_domain).unwrap()).unwrap());
        assert_eq!(
            imported_static.constructors.as_ref(),
            [relocation.constructor(type_export.constructor(static_constructor).unwrap()).unwrap()]
        );
        assert_eq!(imported.associated_surfaces, surfaces);
    }
}
