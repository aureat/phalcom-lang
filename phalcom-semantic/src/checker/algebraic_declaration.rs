//! Semantic analysis and product construction for unified algebraic declarations.

use crate::algebraic_semantics::{AlgebraicDeclarationInfo, AlgebraicRepresentation, AlgebraicSumInfo, StaticAlgebraicDeclarationInfo};
use crate::data_semantics::{
    DataComponentSemantic, DataConstructorParameter, DataConstructorParameterContribution, DataConstructorSignature, DataConstructorTemplate, DataInfo,
    DataProductContribution, DataProductExpansionSemantic, DataProductShapeTemplate, DataShape,
};
use crate::db::product::AlgebraicDeclarationProduct;
use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact, DeclaredTypeState};
use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic, SemanticSourceSpan};
use crate::enum_semantics::{
    AlgebraicCasePayloadKind, EnumInfo, VariantConstructorParameter, VariantConstructorSignature, VariantFieldSemantic, VariantInfo, VariantShape,
    VariantVisibility,
};
use crate::identity::{
    AssociatedStaticDomainDeclarationId, CallableId, DataComponentId, DataConstructorId, DeclarationId, DispatchSide, ModuleId, VariantConstructorId,
    VariantFamilyId, VariantFieldId, VariantId,
};
use crate::resolver::LinkedTypeResolver;
use crate::surface::MemberVisibility;
use crate::types::annotation::{ScopedTypeResolver, TypeFormationSite, TypeLevelBinding, TypeResolver, resolve_type_annotation, resolve_type_form, type_level_binding_for_parameter};
use crate::types::case_environment::{CaseEnvironmentError, CaseTypeEnvironment, derive_case_environment};
use crate::types::evidence::TypeKnowledge;
use crate::types::id::{KindId, TypeId, TypeParameterId};
use crate::types::kind::KindData;
use crate::types::store::TypeStore;
use phalcom_ast::ast::{AlgebraicDef, AlgebraicPhase, CasePayloadSyntax, DataProductContributionSyntax, ProductShapeSyntax};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

/// Builds complete semantic metadata and products for an [`AlgebraicDef`].
pub fn build_algebraic_semantics(
    owner: &DeclarationId,
    def: &AlgebraicDef,
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &LinkedTypeResolver,
    module_id: &ModuleId,
) -> Option<AlgebraicDeclarationProduct> {
    build_algebraic_semantics_with_static_products(owner, def, store, declarations, resolver, module_id, None, &BTreeMap::new())
}

/// Builds an algebraic declaration using the canonical static-case catalog and
/// checked constructor-signature products when the declaration is type-only.
pub fn build_algebraic_semantics_with_static_products(
    owner: &DeclarationId,
    def: &AlgebraicDef,
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &LinkedTypeResolver,
    module_id: &ModuleId,
    static_catalog: Option<&crate::static_case::StaticDomainCaseCatalogProduct>,
    static_signatures: &BTreeMap<crate::identity::StaticCaseId, Arc<crate::static_case::StaticCaseSignatureProduct>>,
) -> Option<AlgebraicDeclarationProduct> {
    if matches!(&def.shape, phalcom_ast::ast::AlgebraicShapeSyntax::IntrinsicFacade(_)) {
        return None;
    }
    let mut diagnostics = Vec::new();

    let generic_sig = declarations.generic_signature(owner).cloned();
    let generic_params: Vec<TypeParameterId> = generic_sig.as_ref().map(|sig| sig.parameters.to_vec()).unwrap_or_default();

    let root_form = declarations.form(owner)?;

    let mut type_params_map = HashMap::new();
    if let Some(ref sig) = generic_sig {
        for &p in sig.parameters.iter() {
            let p_name = store.type_parameter(p).name.to_string();
            let binding = type_level_binding_for_parameter(store, p);
            type_params_map.insert(p_name, binding);
        }
    }

    let scoped_resolver = ScopedTypeResolver {
        parent: resolver,
        type_parameters: type_params_map,
    };
    let formation_site = TypeFormationSite::member(module_id.clone(), owner.clone(), DispatchSide::Instance);

    let representation = if let Some(product_shape) = def.product_shape() {
        let result_type_template = if generic_params.is_empty() {
            root_form
        } else {
            let static_args: Vec<crate::types::store::StaticArgument> = generic_params
                .iter()
                .map(|&p| {
                    let kind = store.type_parameter(p).kind;
                    if kind == KindId::RECORD_ROW {
                        let row = store
                            .record_row_checked(Vec::new(), crate::types::row::RecordRowTail::Parameter(p))
                            .expect("a parameter-only record row is canonical");
                        crate::types::store::StaticArgument::RecordRow(row)
                    } else {
                        crate::types::store::StaticArgument::Term(store.parameter_form(p))
                    }
                })
                .collect();
            store.apply_static_form(root_form, &static_args).unwrap_or(root_form)
        };

        let shape = match product_shape {
            ProductShapeSyntax::Tuple { .. } => DataShape::Tuple,
            ProductShapeSyntax::Record { .. } => DataShape::Record,
        };

        let mut seen_names: HashSet<String> = HashSet::new();
        let mut seen_labels: HashSet<String> = HashSet::new();
        let mut component_semantics = Vec::new();
        let mut constructor_params = Vec::new();
        let mut shape_contributions = Vec::new();
        let mut constructor_param_contributions = Vec::new();
        let mut next_comp_idx = 0u32;

        for contrib in product_shape.contributions() {
            match contrib {
                DataProductContributionSyntax::Component(comp) => {
                    let comp_id = DataComponentId::new(owner.clone(), next_comp_idx);
                    next_comp_idx += 1;
                    let local_name: Box<str> = comp.local_name.clone().into_boxed_str();
                    let external_label: Option<Box<str>> = comp.external_label.clone().map(String::into_boxed_str);

                    match shape {
                        DataShape::Record => {
                            if !seen_names.insert(comp.local_name.clone()) {
                                diagnostics.push(SemanticDiagnostic::error_in(
                                    module_id.clone(),
                                    DiagnosticCode::DataDuplicateComponent,
                                    format!("duplicate component `{}` in data `{}`", comp.local_name, owner.name),
                                    comp.range,
                                ));
                            }
                        }
                        DataShape::Tuple => {
                            if let Some(ref label) = comp.external_label {
                                if !seen_labels.insert(label.clone()) {
                                    diagnostics.push(SemanticDiagnostic::error_in(
                                        module_id.clone(),
                                        DiagnosticCode::DataDuplicateComponent,
                                        format!("duplicate component label `{}` in data `{}`", label, owner.name),
                                        comp.range,
                                    ));
                                }
                            }
                        }
                    }

                    let mut ann_diags = Vec::new();
                    let raw_knowledge = resolve_algebraic_payload_annotation(
                        store, declarations, &scoped_resolver, &formation_site, &comp.annotation, def.phase == AlgebraicPhase::TypeOnly, &mut ann_diags,
                    );
                    diagnostics.extend(ann_diags);

                    let declared_type = match raw_knowledge {
                        TypeKnowledge::Known(evidence) => DeclaredTypeFact {
                            state: DeclaredTypeState::Known(crate::types::parameter::TypeTerm::Canonical(evidence.ty())),
                            basis: DeclaredTypeBasis::SourceAnnotation,
                        },
                        _ => DeclaredTypeFact::from_knowledge(&raw_knowledge),
                    };

                    let source = Some(SemanticSourceSpan::new(module_id.clone(), comp.range));

                    let comp_sem = DataComponentSemantic {
                        id: comp_id.clone(),
                        local_name: local_name.clone(),
                        external_label: external_label.clone(),
                        declared_type: declared_type.clone(),
                        source,
                    };
                    component_semantics.push(comp_sem.clone());
                    shape_contributions.push(DataProductContribution::Component(comp_sem));

                    let param = DataConstructorParameter {
                        component: comp_id,
                        external_label,
                        local_name,
                        declared_type,
                    };
                    constructor_params.push(param.clone());
                    constructor_param_contributions.push(DataConstructorParameterContribution::Parameter(param));
                }
                DataProductContributionSyntax::Expansion(exp) => {
                    let operand_name = match &exp.operand.expr {
                        phalcom_ast::ast::TypeAnnotationExpr::Reference(sym) if sym.is_bare() => Some(sym.root.as_str()),
                        _ => None,
                    };
                    let raw_knowledge = if let Some(name) = operand_name {
                        if let Some(TypeLevelBinding::RecordRow(parameter)) = scoped_resolver.resolve_type_level_binding(name) {
                            let row_ty = store
                                .record_row_type_checked(Vec::new(), crate::types::row::RecordRowTail::Parameter(parameter))
                                .expect("open row type");
                            TypeKnowledge::established(row_ty, crate::types::evidence::EvidenceOrigin::DeclarationSemantics)
                        } else if let Some(TypeLevelBinding::TypeForm(form)) = scoped_resolver.resolve_type_level_binding(name) {
                            let kind = store.kind_of(form);
                            if kind == KindId::COMPLETE_TYPE_PACK || kind == KindId::TYPE_PACK || kind == KindId::LABELED_TYPE_PACK {
                                TypeKnowledge::established(form, crate::types::evidence::EvidenceOrigin::DeclarationSemantics)
                            } else {
                                let mut ann_diags = Vec::new();
                                let rk = resolve_algebraic_payload_annotation(
                                    store, declarations, &scoped_resolver, &formation_site, &exp.operand, true, &mut ann_diags,
                                );
                                diagnostics.extend(ann_diags);
                                rk
                            }
                        } else {
                            let mut ann_diags = Vec::new();
                            let rk = resolve_algebraic_payload_annotation(
                                store, declarations, &scoped_resolver, &formation_site, &exp.operand, true, &mut ann_diags,
                            );
                            diagnostics.extend(ann_diags);
                            rk
                        }
                    } else {
                        let mut ann_diags = Vec::new();
                        let rk = resolve_algebraic_payload_annotation(
                            store, declarations, &scoped_resolver, &formation_site, &exp.operand, true, &mut ann_diags,
                        );
                        diagnostics.extend(ann_diags);
                        rk
                    };

                    let declared_type = match raw_knowledge {
                        TypeKnowledge::Known(evidence) => DeclaredTypeFact {
                            state: DeclaredTypeState::Known(crate::types::parameter::TypeTerm::Canonical(evidence.ty())),
                            basis: DeclaredTypeBasis::SourceAnnotation,
                        },
                        _ => DeclaredTypeFact::from_knowledge(&raw_knowledge),
                    };

                    let exp_source = Some(SemanticSourceSpan::new(module_id.clone(), exp.range));
                    let exp_sem = DataProductExpansionSemantic {
                        mode: exp.mode,
                        operand: declared_type,
                        source: exp_source,
                    };
                    shape_contributions.push(DataProductContribution::Expansion(exp_sem.clone()));
                    constructor_param_contributions.push(DataConstructorParameterContribution::Expansion(exp_sem));
                }
            }
        }

        let constructor = DataConstructorSignature {
            constructor: DataConstructorId::new(owner.clone()),
            visibility: def.constructor_visibility(),
            parameters: constructor_params.into_boxed_slice(),
            result_type_template,
            source: Some(SemanticSourceSpan::new(module_id.clone(), def.name_range)),
        };

        let shape_template = DataProductShapeTemplate {
            shape,
            contributions: shape_contributions.into_boxed_slice(),
        };

        let constructor_template = if shape_template.has_expansions() {
            Some(DataConstructorTemplate {
                constructor: DataConstructorId::new(owner.clone()),
                visibility: def.constructor_visibility(),
                parameters: constructor_param_contributions.into_boxed_slice(),
                result_type_template,
                source: Some(SemanticSourceSpan::new(module_id.clone(), def.name_range)),
            })
        } else {
            None
        };

        let data_info = DataInfo {
            owner: owner.clone(),
            root_form,
            generic_signature: generic_sig.clone(),
            static_data: None,
            shape,
            components: component_semantics.into_boxed_slice(),
            constructor,
            shape_template,
            constructor_template,
            source: Some(SemanticSourceSpan::new(module_id.clone(), def.range)),
        };

        AlgebraicRepresentation::Product(Arc::new(data_info))
    } else if let Some(sum_shape) = def.sum_shape() {
        let default_result_type = if generic_params.is_empty() {
            root_form
        } else {
            let static_args: Vec<crate::types::store::StaticArgument> = generic_params
                .iter()
                .map(|&p| {
                    let kind = store.type_parameter(p).kind;
                    if kind == KindId::RECORD_ROW {
                        let row = store
                            .record_row_checked(Vec::new(), crate::types::row::RecordRowTail::Parameter(p))
                            .expect("a parameter-only record row is canonical");
                        crate::types::store::StaticArgument::RecordRow(row)
                    } else {
                        crate::types::store::StaticArgument::Term(store.parameter_form(p))
                    }
                })
                .collect();
            store.apply_static_form(root_form, &static_args).unwrap_or(root_form)
        };

        let mut variant_ids = Vec::new();
        let mut variant_infos = Vec::new();
        let mut seen_selectors: HashSet<phalcom_common::selector::Selector> = HashSet::new();
        let mut variant_families = Vec::new();
        let mut seen_families: HashSet<VariantFamilyId> = HashSet::new();

        for variant in &sum_shape.cases {
            let selector = phalcom_ast::selector::selector_from_variant(variant);
            let variant_id = VariantId::new(owner.clone(), selector.clone());

            if !seen_selectors.insert(selector.clone()) {
                diagnostics.push(SemanticDiagnostic::error_in(
                    module_id.clone(),
                    DiagnosticCode::EnumVariantDuplicate,
                    format!("duplicate variant `{}` in enum `{}`", variant.name, owner.name),
                    variant.range,
                ));
            }

            let family = variant_id.family();
            if let Some(ref fam) = family {
                if seen_families.insert(fam.clone()) {
                    variant_families.push(fam.clone());
                }
            }

            let (shape, payload_kind) = match &variant.payload {
                CasePayloadSyntax::Singleton => (VariantShape::Singleton, AlgebraicCasePayloadKind::Singleton),
                CasePayloadSyntax::Tuple { explicit_empty, parameters, .. } => {
                    if *explicit_empty && parameters.is_empty() {
                        (VariantShape::Constructor, AlgebraicCasePayloadKind::ExplicitEmpty)
                    } else {
                        (VariantShape::Constructor, AlgebraicCasePayloadKind::Tuple)
                    }
                }
                CasePayloadSyntax::Record { .. } => (VariantShape::Constructor, AlgebraicCasePayloadKind::Record),
            };

            let constructor_callable = CallableId::variant_constructor(variant_id.clone());
            let variant_generic_signature = if variant.generic_parameters.is_empty() {
                None
            } else {
                match crate::types::annotation::resolve_generic_signature(
                    store,
                    declarations,
                    &scoped_resolver,
                    &formation_site,
                    crate::types::parameter::TypeParameterOwner::Callable(constructor_callable),
                    crate::types::annotation::GenericBinderSite::Callable,
                    &variant.generic_parameters,
                    variant.where_clause.as_ref(),
                    &mut diagnostics,
                ) {
                    crate::types::annotation::TypeFormationOutcome::Ready(signature) => Some(signature),
                    crate::types::annotation::TypeFormationOutcome::Dynamic
                    | crate::types::annotation::TypeFormationOutcome::Missing(_)
                    | crate::types::annotation::TypeFormationOutcome::Unresolved(_)
                    | crate::types::annotation::TypeFormationOutcome::Invalid(_)
                    | crate::types::annotation::TypeFormationOutcome::Blocked(_)
                    | crate::types::annotation::TypeFormationOutcome::Cancelled
                    | crate::types::annotation::TypeFormationOutcome::BudgetExceeded(_)
                    | crate::types::annotation::TypeFormationOutcome::InternalFailure(_) => None,
                }
            };
            let variant_type_parameters = variant_generic_signature
                .as_ref()
                .map(|signature| signature.parameters.to_vec())
                .unwrap_or_default()
                .into_iter()
                .map(|parameter_id| {
                    let name = store.type_parameter(parameter_id).name.to_string();
                    let binding = type_level_binding_for_parameter(store, parameter_id);
                    (name, binding)
                })
                .collect();
            let variant_resolver = ScopedTypeResolver {
                parent: &scoped_resolver,
                type_parameters: variant_type_parameters,
            };

            // Result type and GADT case environment
            let (result_type_template, case_env) = if let Some(ref ret_ann) = variant.result_annotation {
                let mut ann_diags = Vec::new();
                let form_res = resolve_type_form(store, declarations, &variant_resolver, &formation_site, ret_ann, &mut ann_diags);
                diagnostics.extend(ann_diags);

                match form_res {
                    crate::types::annotation::TypeFormResolution::Ready(ret_ty) => {
                        if store.kind_of(ret_ty) != KindId::TYPE {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                module_id.clone(),
                                DiagnosticCode::EnumVariantResultInvalid,
                                format!("return type of variant `{}` must have kind Type", variant.name),
                                ret_ann.range,
                            ));
                            (default_result_type, CaseTypeEnvironment::default())
                        } else if store.nominal_origin_declaration(ret_ty) != Some(owner) {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                module_id.clone(),
                                DiagnosticCode::EnumVariantResultWrongOwner,
                                format!(
                                    "return type of variant `{}` must be an instance of enclosing enum `{}`",
                                    variant.name, owner.name
                                ),
                                ret_ann.range,
                            ));
                            (default_result_type, CaseTypeEnvironment::default())
                        } else {
                            match derive_case_environment(store, owner, &generic_params, Some(ret_ty)) {
                                Ok(env) => (ret_ty, env),
                                Err(CaseEnvironmentError::ResultUnsaturated { expected_arity, got_arity }) => {
                                    diagnostics.push(SemanticDiagnostic::error_in(
                                        module_id.clone(),
                                        DiagnosticCode::EnumVariantResultUnsaturated,
                                        format!(
                                            "return type of variant `{}` expects {} type arguments, got {}",
                                            variant.name, expected_arity, got_arity
                                        ),
                                        ret_ann.range,
                                    ));
                                    (ret_ty, CaseTypeEnvironment::default())
                                }
                                Err(CaseEnvironmentError::CyclicEquality { .. }) => {
                                    diagnostics.push(SemanticDiagnostic::error_in(
                                        module_id.clone(),
                                        DiagnosticCode::EnumVariantGadtCyclicEquality,
                                        format!("GADT case equality cycle detected in return type of variant `{}`", variant.name),
                                        ret_ann.range,
                                    ));
                                    (ret_ty, CaseTypeEnvironment::default())
                                }
                                Err(CaseEnvironmentError::ResultNotProper) => {
                                    diagnostics.push(SemanticDiagnostic::error_in(
                                        module_id.clone(),
                                        DiagnosticCode::EnumVariantResultInvalid,
                                        format!("return type of variant `{}` is not a proper type", variant.name),
                                        ret_ann.range,
                                    ));
                                    (ret_ty, CaseTypeEnvironment::default())
                                }
                                Err(CaseEnvironmentError::ResultWrongOwner { .. }) => {
                                    diagnostics.push(SemanticDiagnostic::error_in(
                                        module_id.clone(),
                                        DiagnosticCode::EnumVariantResultWrongOwner,
                                        format!(
                                            "return type of variant `{}` must be an instance of enclosing enum `{}`",
                                            variant.name, owner.name
                                        ),
                                        ret_ann.range,
                                    ));
                                    (ret_ty, CaseTypeEnvironment::default())
                                }
                            }
                        }
                    }
                    crate::types::annotation::TypeFormResolution::Dynamic
                    | crate::types::annotation::TypeFormResolution::Missing(_)
                    | crate::types::annotation::TypeFormResolution::Unresolved(_)
                    | crate::types::annotation::TypeFormResolution::Invalid(_)
                    | crate::types::annotation::TypeFormResolution::Blocked(_)
                    | crate::types::annotation::TypeFormResolution::Cancelled
                    | crate::types::annotation::TypeFormResolution::BudgetExceeded(_)
                    | crate::types::annotation::TypeFormResolution::InternalFailure(_) => {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            module_id.clone(),
                            DiagnosticCode::EnumVariantResultInvalid,
                            format!("unresolved return type for variant `{}`", variant.name),
                            ret_ann.range,
                        ));
                        (default_result_type, CaseTypeEnvironment::default())
                    }
                }
            } else {
                (default_result_type, CaseTypeEnvironment::default())
            };

            let type_handle = store.intern_variant_identity(variant_id.clone());
            let exact_case_template = store.exact_case_type(&variant_id, result_type_template).unwrap_or_else(|_| {
                store.intern_with_kind(
                    crate::types::store::TypeData::ExactCase {
                        variant: type_handle,
                        enum_type: default_result_type,
                    },
                    KindId::TYPE,
                )
            });

            // Resolve payload fields
            let mut fields = Vec::new();
            let mut constructor_params = Vec::new();

            match &variant.payload {
                CasePayloadSyntax::Singleton => {}
                CasePayloadSyntax::Tuple { parameters, .. } => {
                    let case_subst = case_env.to_substitution();

                    for (idx, param) in parameters.iter().enumerate() {
                        let field_id = VariantFieldId::new(variant_id.clone(), idx as u32);
                        let local_name: Box<str> = param.name.clone().into_boxed_str();
                        let external_label: Option<Box<str>> = param.label.clone().map(String::into_boxed_str);

                        let declared_type = if let Some(ref ann) = param.annotation {
                            let mut ann_diags = Vec::new();
                            let raw_knowledge = resolve_algebraic_payload_annotation(
                                store, declarations, &variant_resolver, &formation_site, ann, def.phase == AlgebraicPhase::TypeOnly, &mut ann_diags,
                            );
                            diagnostics.extend(ann_diags);

                            match raw_knowledge {
                                TypeKnowledge::Known(evidence) => {
                                    let specialized_ty = case_subst.apply(store, evidence.ty());
                                    DeclaredTypeFact {
                                        state: DeclaredTypeState::Known(crate::types::parameter::TypeTerm::Canonical(specialized_ty)),
                                        basis: DeclaredTypeBasis::SourceAnnotation,
                                    }
                                }
                                _ => DeclaredTypeFact::from_knowledge(&raw_knowledge),
                            }
                        } else {
                            DeclaredTypeFact::unknown(crate::types::evidence::UnknownReason::UnannotatedDeclaration)
                        };

                        let field_source = Some(SemanticSourceSpan::new(module_id.clone(), param.range));

                        fields.push(VariantFieldSemantic {
                            id: field_id.clone(),
                            local_name: local_name.clone(),
                            external_label: external_label.clone(),
                            declared_type: declared_type.clone(),
                            source: field_source,
                        });

                        constructor_params.push(VariantConstructorParameter {
                            field: field_id,
                            external_label,
                            local_name,
                            declared_type,
                        });
                    }
                }
                CasePayloadSyntax::Record { components, .. } => {
                    let case_subst = case_env.to_substitution();

                    for (idx, comp) in components.iter().enumerate() {
                        let field_id = VariantFieldId::new(variant_id.clone(), idx as u32);
                        let local_name: Box<str> = comp.local_name.clone().into_boxed_str();
                        let external_label: Option<Box<str>> = comp.external_label.clone().map(String::into_boxed_str).or_else(|| Some(local_name.clone()));

                        let mut ann_diags = Vec::new();
                        let raw_knowledge = resolve_algebraic_payload_annotation(
                            store,
                            declarations,
                            &variant_resolver,
                            &formation_site,
                            &comp.annotation,
                            def.phase == AlgebraicPhase::TypeOnly,
                            &mut ann_diags,
                        );
                        diagnostics.extend(ann_diags);

                        let declared_type = match raw_knowledge {
                            TypeKnowledge::Known(evidence) => {
                                let specialized_ty = case_subst.apply(store, evidence.ty());
                                DeclaredTypeFact {
                                    state: DeclaredTypeState::Known(crate::types::parameter::TypeTerm::Canonical(specialized_ty)),
                                    basis: DeclaredTypeBasis::SourceAnnotation,
                                }
                            }
                            _ => DeclaredTypeFact::from_knowledge(&raw_knowledge),
                        };

                        let field_source = Some(SemanticSourceSpan::new(module_id.clone(), comp.range));

                        fields.push(VariantFieldSemantic {
                            id: field_id.clone(),
                            local_name: local_name.clone(),
                            external_label: external_label.clone(),
                            declared_type: declared_type.clone(),
                            source: field_source,
                        });

                        constructor_params.push(VariantConstructorParameter {
                            field: field_id,
                            external_label,
                            local_name,
                            declared_type,
                        });
                    }
                }
            }

            let constructor = match shape {
                VariantShape::Singleton => None,
                VariantShape::Constructor => Some(VariantConstructorSignature {
                    constructor: VariantConstructorId::new(variant_id.clone()),
                    generic_signature: variant_generic_signature,
                    parameters: constructor_params.into_boxed_slice(),
                    result_type_template,
                    exact_case_template,
                    source: Some(SemanticSourceSpan::new(module_id.clone(), variant.range)),
                }),
            };

            // Visibility parsing
            let mut visibility = VariantVisibility::default();
            for attr in &variant.attributes {
                if attr.name == "private" {
                    visibility.construct = MemberVisibility::Private;
                } else if attr.name == "protected" {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        module_id.clone(),
                        DiagnosticCode::EnumVariantVisibilityInvalid,
                        format!("`protected` is not supported on enum variant `{}`", variant.name),
                        attr.range,
                    ));
                }
            }

            let variant_info = VariantInfo {
                id: variant_id.clone(),
                type_handle,
                family,
                shape,
                payload_kind,
                fields: fields.into_boxed_slice(),
                result_type_template,
                exact_case_template,
                case_environment: case_env,
                constructor,
                visibility,
                source: Some(SemanticSourceSpan::new(module_id.clone(), variant.range)),
            };

            variant_ids.push(variant_id);
            variant_infos.push(variant_info);
        }

        let enum_info = EnumInfo {
            owner: owner.clone(),
            root_form,
            generic_signature: generic_sig.clone(),
            static_data: None,
            default_result_type,
            variants: variant_ids.into_boxed_slice(),
            variant_families: variant_families.into_boxed_slice(),
            source: Some(SemanticSourceSpan::new(module_id.clone(), def.range)),
        };

        AlgebraicRepresentation::Sum(Arc::new(AlgebraicSumInfo {
            enum_info: Arc::new(enum_info),
            variants: Arc::from(variant_infos.into_boxed_slice()),
        }))
    } else {
        return None;
    };

    let static_data = if def.phase == AlgebraicPhase::TypeOnly {
        Some(register_static_declaration(owner, &representation, def, store, module_id, &mut diagnostics, static_catalog, static_signatures))
    } else {
        None
    };

    let decl_info = AlgebraicDeclarationInfo {
        owner: owner.clone(),
        phase: def.phase,
        implementation: def.implementation,
        generic_signature: generic_sig,
        representation,
        static_data,
        source: Some(SemanticSourceSpan::new(module_id.clone(), def.range)),
    };

    Some(AlgebraicDeclarationProduct {
        info: Arc::new(decl_info),
        diagnostics: Arc::from(diagnostics.into_boxed_slice()),
    })
}

/// Builds the declaration-owned static-domain product for associated `type data`.
/// The owner declaration and its canonical generic environment define identity;
/// the source declaration name is not promoted to a synthetic top-level owner.
pub fn build_associated_static_domain_declaration(
    owner: &DeclarationId,
    def: &AlgebraicDef,
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &LinkedTypeResolver,
    module_id: &ModuleId,
) -> (Option<crate::db::product::AssociatedStaticDomainDeclarationProduct>, Vec<SemanticDiagnostic>) {
    build_associated_static_domain_declaration_with_static_products(owner, def, store, declarations, resolver, module_id, None, &BTreeMap::new())
}

pub fn build_associated_static_domain_declaration_with_static_products(
    owner: &DeclarationId,
    def: &AlgebraicDef,
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &LinkedTypeResolver,
    module_id: &ModuleId,
    catalog: Option<&crate::static_case::StaticDomainCaseCatalogProduct>,
    signatures: &BTreeMap<crate::identity::StaticCaseId, Arc<crate::static_case::StaticCaseSignatureProduct>>,
) -> (Option<crate::db::product::AssociatedStaticDomainDeclarationProduct>, Vec<SemanticDiagnostic>) {
    let mut diagnostics = Vec::new();
    if def.phase != AlgebraicPhase::TypeOnly || !def.generic_parameters.is_empty() || def.where_clause.is_some() {
        diagnostics.push(SemanticDiagnostic::error_in(
            module_id.clone(),
            DiagnosticCode::KindExpectedType,
            "associated type data must be type-only and cannot declare member-local generic parameters",
            def.range,
        ));
        return (None, diagnostics);
    }

    let id = AssociatedStaticDomainDeclarationId::new(owner.clone(), def.name.clone());
    let owner_signature = declarations.generic_signature(owner).cloned();
    let mut type_parameters = HashMap::new();
    if let Some(signature) = &owner_signature {
        for parameter in signature.parameters.iter().copied() {
            let name = store.type_parameter(parameter).name.to_string();
            type_parameters.insert(name, type_level_binding_for_parameter(store, parameter));
        }
    }
    let scoped_resolver = ScopedTypeResolver { parent: resolver, type_parameters };
    let formation_site = TypeFormationSite::member(module_id.clone(), owner.clone(), DispatchSide::Instance);
    let owner_arguments = declarations
        .generic_signature(owner)
        .map(|signature| {
            signature
                .parameters
                .iter()
                .map(|parameter| {
                    let kind = store.type_parameter(*parameter).kind;
                    if kind == KindId::RECORD_ROW {
                        let row = store
                            .record_row_checked(Vec::new(), crate::types::row::RecordRowTail::Parameter(*parameter))
                            .expect("a parameter-only record row is canonical");
                        crate::types::store::StaticArgument::RecordRow(row)
                    } else {
                        crate::types::store::StaticArgument::Term(store.parameter_form(*parameter))
                    }
                })
                .collect::<Vec<_>>()
                .into_boxed_slice()
        })
        .unwrap_or_else(|| Box::new([]));
    let domain = store.associated_static_domain(id.clone(), owner_arguments);

    let mut constructors = Vec::new();
    if let Some(catalog) = catalog {
        for case in catalog.cases.iter() {
            let signature = signatures.get(&case.id).map(Arc::as_ref);
            if let Some(constructor) = crate::static_case::intern_static_case_constructor(store, domain, case, signature) {
                constructors.push(constructor);
            }
        }
    } else {
    match &def.shape {
        phalcom_ast::ast::AlgebraicShapeSyntax::Sum(sum) => {
            for case in &sum.cases {
                if !case.generic_parameters.is_empty() || case.where_clause.is_some() || case.result_annotation.is_some() {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        module_id.clone(),
                        DiagnosticCode::KindExpectedType,
                        "associated type-data cases cannot declare local generics, constraints, or specialized results",
                        case.range,
                    ));
                    continue;
                }
                let mut parameter_kinds = Vec::new();
                let mut valid_case = true;
                match &case.payload {
                    phalcom_ast::ast::CasePayloadSyntax::Singleton => {}
                    phalcom_ast::ast::CasePayloadSyntax::Tuple { parameters, .. } => {
                        for parameter in parameters {
                            let Some(annotation) = parameter.annotation.as_ref() else {
                                valid_case = false;
                                diagnostics.push(SemanticDiagnostic::error_in(
                                    module_id.clone(),
                                    DiagnosticCode::KindExpectedType,
                                    "associated static-domain constructor parameters require explicit static-domain types",
                                    parameter.range,
                                ));
                                continue;
                            };
                            if let Some(kind) = associated_static_payload_kind(
                                store, declarations, &scoped_resolver, &formation_site, annotation, module_id, parameter.range, &mut diagnostics,
                            ) {
                                parameter_kinds.push(kind);
                            } else {
                                valid_case = false;
                            }
                        }
                    }
                    phalcom_ast::ast::CasePayloadSyntax::Record { components, .. } => {
                        for component in components {
                            if let Some(kind) = associated_static_payload_kind(
                                store,
                                declarations,
                                &scoped_resolver,
                                &formation_site,
                                &component.annotation,
                                module_id,
                                component.range,
                                &mut diagnostics,
                            ) {
                                parameter_kinds.push(kind);
                            } else {
                                valid_case = false;
                            }
                        }
                    }
                };
                if !valid_case {
                    continue;
                }
                constructors.push(store.intern_static_constructor(
                    domain,
                    owner.clone(),
                    phalcom_ast::selector::selector_from_variant(case).encode(),
                    parameter_kinds.into_boxed_slice(),
                ));
            }
        }
        phalcom_ast::ast::AlgebraicShapeSyntax::Product(_) => {
            diagnostics.push(SemanticDiagnostic::error_in(
                module_id.clone(),
                DiagnosticCode::KindExpectedType,
                "associated type data must declare a closed set of static cases",
                def.range,
            ));
        }
        phalcom_ast::ast::AlgebraicShapeSyntax::IntrinsicFacade(_) => {}
    }
    }

    let product = crate::db::product::AssociatedStaticDomainDeclarationProduct {
        id,
        domain,
        constructors: constructors.into_boxed_slice(),
        source_range: def.range,
    };
    (Some(product), diagnostics)
}

fn associated_static_payload_kind(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &ScopedTypeResolver<'_>,
    formation_site: &TypeFormationSite,
    annotation: &phalcom_ast::ast::TypeAnnotation,
    module_id: &ModuleId,
    range: phalcom_ast::ast::SourceRange,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> Option<KindId> {
    match resolve_type_annotation(store, declarations, resolver, formation_site, annotation, diagnostics) {
        TypeKnowledge::Known(evidence) => {
            let ty = evidence.ty();
            let kind = store.kind_of(ty);
            if kind == KindId::TYPE || matches!(store.get_kind(kind), KindData::StaticDomain(_)) {
                Some(kind)
            } else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    module_id.clone(),
                    DiagnosticCode::KindExpectedType,
                    "associated type-data payload must have a supported static kind",
                    range,
                ));
                None
            }
        }
        TypeKnowledge::Unknown(_) | TypeKnowledge::Dynamic(_) => None,
    }
}

/// Type-only algebraic payloads may name a static domain header directly.
/// Runtime `data` payloads retain the ordinary proper-type checker.
fn resolve_algebraic_payload_annotation(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn crate::types::annotation::TypeResolver,
    site: &TypeFormationSite,
    annotation: &phalcom_ast::ast::TypeAnnotation,
    allow_static_domain: bool,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeKnowledge {
    if !allow_static_domain {
        return resolve_type_annotation(store, declarations, resolver, site, annotation, diagnostics);
    }
    match resolve_type_form(store, declarations, resolver, site, annotation, diagnostics) {
        crate::types::annotation::TypeFormResolution::Ready(form) => {
            let kind = store.kind_of(form);
            if kind == KindId::TYPE || matches!(store.get_kind(kind), KindData::StaticDomain(_)) {
                TypeKnowledge::assumed(form, crate::types::evidence::EvidenceOrigin::DeveloperAnnotation).with_range(annotation.range)
            } else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    site.module.clone(),
                    DiagnosticCode::KindExpectedType,
                    "type-data payload must be a type or static-domain form",
                    annotation.range,
                ));
                TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::SuppressedByInvalidCause)
            }
        }
        crate::types::annotation::TypeFormResolution::Dynamic => TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::ExplicitEscape),
        crate::types::annotation::TypeFormResolution::Unresolved(crate::types::annotation::TypeFormationUnresolved::Name(name)) => {
            TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::UnresolvedName(name))
        }
        crate::types::annotation::TypeFormResolution::Unresolved(crate::types::annotation::TypeFormationUnresolved::SelfOutsideOwner) => {
            TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::UnresolvedName("Self".into()))
        }
        crate::types::annotation::TypeFormResolution::Blocked(_) => TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::InferenceBlocked),
        crate::types::annotation::TypeFormResolution::Cancelled => TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::InferenceCancelled),
        crate::types::annotation::TypeFormResolution::BudgetExceeded(_) => {
            TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::InferenceBudgetExceeded)
        }
        crate::types::annotation::TypeFormResolution::Invalid(_)
        | crate::types::annotation::TypeFormResolution::Missing(_)
        | crate::types::annotation::TypeFormResolution::InternalFailure(_) => {
            TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::SuppressedByInvalidCause)
        }
    }
}

fn register_static_declaration(
    owner: &DeclarationId,
    representation: &AlgebraicRepresentation,
    def: &AlgebraicDef,
    store: &mut TypeStore,
    module_id: &ModuleId,
    diagnostics: &mut Vec<SemanticDiagnostic>,
    catalog: Option<&crate::static_case::StaticDomainCaseCatalogProduct>,
    signatures: &BTreeMap<crate::identity::StaticCaseId, Arc<crate::static_case::StaticCaseSignatureProduct>>,
) -> StaticAlgebraicDeclarationInfo {
    let domain = store.nominal_static_domain(owner.clone());
    if let Some(catalog) = catalog {
        let constructors = catalog
            .cases
            .iter()
            .filter_map(|case| {
                let signature = signatures.get(&case.id).map(Arc::as_ref);
                crate::static_case::intern_static_case_constructor(store, domain, case, signature)
            })
            .collect::<Vec<_>>();
        return StaticAlgebraicDeclarationInfo {
            domain,
            constructors: constructors.into_boxed_slice(),
        };
    }
    let constructors = match representation {
        AlgebraicRepresentation::Product(info) => {
            let kinds = info
                .components
                .iter()
                .filter_map(|component| static_payload_kind(store, component.declared_type.canonical_type(), diagnostics, module_id, def.range))
                .collect::<Vec<_>>();
            if kinds.len() == info.components.len() {
                vec![store.intern_static_constructor(
                    domain,
                    owner.clone(),
                    owner.name.clone(),
                    kinds.into_boxed_slice(),
                )]
            } else {
                Vec::new()
            }
        }
        AlgebraicRepresentation::Sum(info) => info
            .variants
            .iter()
            .filter_map(|variant| {
                let mut kinds = Vec::new();
                for field in variant.fields.iter() {
                    let Some(kind) = static_payload_kind(store, field.declared_type.canonical_type(), diagnostics, module_id, def.range) else {
                        return None;
                    };
                    kinds.push(kind);
                }
                Some(store.intern_static_constructor(
                    domain,
                    owner.clone(),
                    variant.id.selector.encode(),
                    kinds.into_boxed_slice(),
                ))
            })
            .collect(),
    };
    StaticAlgebraicDeclarationInfo {
        domain,
        constructors: constructors.into_boxed_slice(),
    }
}

fn static_payload_kind(
    store: &mut TypeStore,
    payload: Option<TypeId>,
    diagnostics: &mut Vec<SemanticDiagnostic>,
    module_id: &ModuleId,
    range: phalcom_ast::ast::SourceRange,
) -> Option<KindId> {
    let Some(payload) = payload else {
        diagnostics.push(SemanticDiagnostic::error_in(
            module_id.clone(),
            DiagnosticCode::KindExpectedType,
            "type-data payload must have a supported static domain",
            range,
        ));
        return None;
    };
    let kind = crate::types::annotation::static_domain_kind_of_form(store, payload);
    if kind == KindId::TYPE || matches!(store.get_kind(kind), KindData::StaticDomain(_)) {
        Some(kind)
    } else {
        diagnostics.push(SemanticDiagnostic::error_in(
            module_id.clone(),
            DiagnosticCode::KindExpectedType,
            "type-data payload must have kind Type or a supported static domain",
            range,
        ));
        None
    }
}
