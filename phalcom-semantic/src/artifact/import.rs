//! Semantic-owned artifact import for structural primary contracts (`PERF003.C2.P7.T7.A`).

use crate::data_semantics::{
    DataComponentSemantic, DataConstructorParameter, DataConstructorParameterContribution, DataConstructorSignature, DataConstructorTemplate, DataInfo,
    DataProductContribution, DataProductExpansionSemantic, DataProductShapeTemplate, DataSemanticTable, DataShape,
};
use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact, DeclaredTypeState};
use crate::declarations::{DeclarationTypeBinding, DeclarationTypeInfo, DeclarationTypeTable};
use crate::diagnostic::SemanticSourceSpan;
use crate::enum_semantics::{
    AlgebraicCasePayloadKind, EnumInfo, EnumSemanticTable, VariantConstructorParameter, VariantConstructorSignature, VariantFieldSemantic, VariantInfo,
    VariantShape, VariantVisibility,
};
use crate::identity::{DataComponentId, DataConstructorId, VariantConstructorId, VariantFieldId, VariantId};
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
use phalcom_compiler_artifact::semantic::aggregates::*;
use phalcom_compiler_artifact::semantic::type_store::{ParameterModeV1, ParameterOwnerV1, SelfRoleV1, VarianceV1};
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
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
    IntrinsicTypeOperationV1, RequirementSelectionV1, SemanticWorldV1, TypeMemberDeclarationRecordV1, TypeMemberDomainV1, TypeMemberImplementationV1,
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
pub fn validate_and_import_semantic_world(world: SemanticWorldV1, limits: &ValidationLimits) -> Result<ValidatedSemanticParts, ArtifactError> {
    let budget_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = AllocationBudget::new(budget_limit);
    validate_and_import_semantic_world_with_allocation_budget(world, limits, &mut allocations)
}

/// Imports semantic authority using the caller's cumulative load transaction.
pub fn validate_and_import_semantic_world_with_allocation_budget(
    world: SemanticWorldV1,
    limits: &ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<ValidatedSemanticParts, ArtifactError> {
    let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = transaction.child(section_limit);
    let parts = validate_and_import_semantic_world_inner(world, limits, &mut allocations)?;
    transaction.commit_child(allocations).map_err(allocation_error)?;
    Ok(parts)
}

fn allocation_error(error: AllocationError) -> ArtifactError {
    ArtifactError::LimitExceeded(
        match error {
            AllocationError::Exhausted => "semantic materialization allocation budget exceeded",
            AllocationError::Overflow => "semantic materialization allocation layout overflow",
            AllocationError::Failed => "semantic materialization allocation failed",
        }
        .into(),
    )
}

fn validate_and_import_semantic_world_inner(
    world: SemanticWorldV1,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ValidatedSemanticParts, ArtifactError> {
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
        let callable = from_stable_callable(&rec.callable).map_err(referential)?;
        let owner = from_stable_declaration(&rec.owner).map_err(referential)?;
        let side = if rec.side == 0 {
            crate::identity::DispatchSide::Instance
        } else {
            crate::identity::DispatchSide::Class
        };
        let selector = callable.selector.clone();
        let generics = if let Some(g) = &rec.generics {
            Some(import_generic_signature(g, &type_relocation, &mut type_store)?)
        } else {
            None
        };
        let mut parameters = Vec::with_capacity(rec.parameters.len());
        for (idx, p) in rec.parameters.iter().enumerate() {
            let declared_type = import_declared_type_fact(&p.declared_type, &type_relocation)?;
            let rest_mode = match p.rest_mode {
                1 => phalcom_ast::ast::RestMode::Positional,
                2 => phalcom_ast::ast::RestMode::Labeled,
                3 => phalcom_ast::ast::RestMode::Complete,
                _ => phalcom_ast::ast::RestMode::None,
            };
            parameters.push(
                crate::signature::CallableParameterSemantic::new(
                    crate::identity::CallableParameterId::new(callable.clone(), idx as u32),
                    p.local_name.clone(),
                    declared_type,
                )
                .with_rest(rest_mode),
            );
        }
        let declared_return = import_declared_type_fact(&rec.declared_return, &type_relocation)?;
        let flow = match rec.return_flow {
            1 => phalcom_native_meta::ReturnFlowSpec::Receiver,
            254 => phalcom_native_meta::ReturnFlowSpec::Never,
            255 => phalcom_native_meta::ReturnFlowSpec::Unknown,
            n if n >= 2 => phalcom_native_meta::ReturnFlowSpec::Argument((n - 2) as usize),
            _ => phalcom_native_meta::ReturnFlowSpec::Value,
        };
        let sig = crate::signature::CallableSemanticSignature {
            callable: callable.clone(),
            owner,
            side,
            selector,
            generics,
            parameters: parameters.into_boxed_slice(),
            declared_return,
            return_validation: crate::signature::ReturnContractValidation::NotApplicable,
            inferred_return: None,
            source: None,
            implementation: phalcom_common::modifier::ImplementationKind::Phalcom,
            native_id: None,
            effects: phalcom_native_meta::EffectSpec::Unknown,
            raises: phalcom_native_meta::RaisesSpec::Unknown,
            flow,
            lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
        };
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

    // 9. Reconstruct callable analyses (P7.T7.B)
    let mut callable_analyses = crate::checker::analysis::CallableAnalysisTable::new();
    for rec in &world.callable_analyses {
        let callable = from_stable_callable(&rec.callable).map_err(referential)?;
        callable_analyses.insert(
            callable.clone(),
            Arc::new(crate::checker::analysis::CallableAnalysis {
                callable,
                body_range: phalcom_common::range::SourceRange::default(),
                expressions: BTreeMap::new(),
                bindings: BTreeMap::new(),
                for_iteration_lanes: Arc::new(BTreeMap::new()),
                positional_expansions: Arc::new(BTreeMap::new()),
                known_invalid_positional_expansions: Arc::new(BTreeMap::new()),
                sequence_literals: Arc::new(BTreeMap::new()),
                associated_resolutions: Arc::new(BTreeMap::new()),
                callable_reference_resolutions: Arc::new(BTreeMap::new()),
                family_applications: Arc::new(BTreeMap::new()),
                match_resolutions: Arc::new(BTreeMap::new()),
                pattern_site_resolutions: Arc::new(BTreeMap::new()),
                flow_graph: Arc::new(crate::checker::flow::graph::FlowGraph::default()),
                entry_flow: crate::checker::analysis::FlowStateSummary::default(),
                exits: crate::checker::analysis::BodyExitFacts {
                    normal_returns: Vec::new(),
                    throws: Vec::new(),
                    unreachable: rec.is_terminating,
                },
                return_validation: crate::signature::ReturnContractValidation::NotApplicable,
                diagnostics: Arc::from([]),
                internal_incidents: Arc::from([]),
                explanations: Arc::new(crate::explain::ExplanationArena::default()),
                return_explanation: None,
                dependencies: Arc::from([]),
                semantic_dependencies: Arc::from([]),
                constructor_super_initializer: None,
                dependency_fingerprint: crate::db::ProductFingerprint::new(0),
                status: crate::checker::analysis::CallableAnalysisStatus::Complete,
            }),
        );
    }

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
        enum_requirements: Arc::new(crate::enum_requirements::EnumRequirementTable::new()),
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
    use crate::identity::{AssociatedFamilyId, AssociatedStaticDomainDeclarationId, DataConstructorId, VariantId};
    use crate::types::store::artifact::TypeExportContext;
    use crate::types::type_member::{
        IntrinsicTypeOperation, TypeMemberDeclarationId, TypeMemberDomain, TypeMemberParameter, TypeMemberSelector, TypeMemberSignature,
    };
    use phalcom_common::modifier::ImplementationKind;
    use phalcom_common::range::SourceRange;
    use phalcom_common::selector::{Selector, SelectorBase};
    use phalcom_modules::{DeclarationId, ModuleComponent, ModuleId, ModulePath};

    fn owner() -> DeclarationId {
        let path = ModulePath::from_components(vec![ModuleComponent::from_identifier("artifact_parity").unwrap()]);
        DeclarationId::new(ModuleId::universe(path), "Packet".into())
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
