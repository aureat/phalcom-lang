//! Semantic analysis and product construction for enum declarations.

use crate::db::product::EnumDeclarationProduct;
use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact, DeclaredTypeState};
use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic, SemanticSourceSpan};
use crate::enum_semantics::{
    EnumInfo, VariantConstructorParameter, VariantConstructorSignature, VariantFieldSemantic, VariantInfo, VariantShape, VariantVisibility,
};
use crate::identity::{DeclarationId, ModuleId, VariantConstructorId, VariantFamilyId, VariantFieldId, VariantId};
use crate::resolver::LinkedTypeResolver;
use crate::surface::MemberVisibility;
use crate::types::annotation::{ScopedTypeResolver, resolve_type_annotation, resolve_type_form};
use crate::types::case_environment::{CaseEnvironmentError, CaseTypeEnvironment, derive_case_environment};
use crate::types::evidence::TypeKnowledge;
use crate::types::id::{KindId, TypeId, TypeParameterId};
use crate::types::store::TypeStore;
use phalcom_ast::ast::EnumDef;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Builds complete semantic metadata and variant products for an [`EnumDef`].
pub fn build_enum_semantics(
    owner: &DeclarationId,
    enum_def: &EnumDef,
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &LinkedTypeResolver,
    module_id: &ModuleId,
) -> EnumDeclarationProduct {
    let mut diagnostics = Vec::new();

    let generic_sig = declarations.generic_signature(owner).cloned();
    let generic_params: Vec<TypeParameterId> = generic_sig.as_ref().map(|sig| sig.parameters.to_vec()).unwrap_or_default();

    let root_form = declarations.form(owner).unwrap_or_else(|| store.nominal(owner.clone()));

    let default_result_type = if generic_params.is_empty() {
        root_form
    } else {
        let param_forms: Vec<TypeId> = generic_params.iter().map(|&p| store.parameter_form(p)).collect();
        store.apply_type_form(root_form, &param_forms).unwrap_or(root_form)
    };

    let mut type_params_map = HashMap::new();
    if let Some(ref sig) = generic_sig {
        for &p in sig.parameters.iter() {
            let p_name = store.type_parameter(p).name.to_string();
            let p_form = store.parameter_form(p);
            type_params_map.insert(p_name, p_form);
        }
    }

    let scoped_resolver = ScopedTypeResolver {
        parent: resolver,
        type_parameters: type_params_map,
    };

    let mut variant_ids = Vec::new();
    let mut variant_infos = Vec::new();
    let mut seen_selectors: HashSet<phalcom_common::selector::Selector> = HashSet::new();
    let mut variant_families = Vec::new();
    let mut seen_families: HashSet<VariantFamilyId> = HashSet::new();

    for member in &enum_def.members {
        let phalcom_ast::ast::EnumMember::Variant(variant) = member else {
            continue;
        };

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

        // Shape rule: Singleton iff payload is None, Constructor iff payload is Some
        let shape = match &variant.payload {
            None => VariantShape::Singleton,
            Some(_) => VariantShape::Constructor,
        };

        // Result type and GADT case environment
        let (result_type_template, case_env) = if let Some(ref ret_ann) = variant.result_annotation {
            let mut ann_diags = Vec::new();
            let form_res = resolve_type_form(store, declarations, &scoped_resolver, module_id, ret_ann, &mut ann_diags);
            diagnostics.extend(ann_diags);

            match form_res {
                crate::types::annotation::TypeFormResolution::Known(ret_ty) => {
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
                _ => {
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

        if let Some(ref payload) = variant.payload {
            let case_subst = case_env.to_substitution();

            for (idx, param) in payload.parameters.iter().enumerate() {
                let field_id = VariantFieldId::new(variant_id.clone(), idx as u32);
                let local_name: Box<str> = param.name.clone().into_boxed_str();
                let external_label: Option<Box<str>> = param.label.clone().map(String::into_boxed_str);

                let declared_type = if let Some(ref ann) = param.annotation {
                    let mut ann_diags = Vec::new();
                    let raw_knowledge = resolve_type_annotation(store, declarations, &scoped_resolver, module_id, ann, &mut ann_diags);
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

        let constructor = match shape {
            VariantShape::Singleton => None,
            VariantShape::Constructor => Some(VariantConstructorSignature {
                constructor: VariantConstructorId::new(variant_id.clone()),
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
                    format!("`@protected` is not supported on enum variant `{}`", variant.name),
                    attr.range,
                ));
            }
        }

        let variant_info = VariantInfo {
            id: variant_id.clone(),
            type_handle,
            family,
            shape,
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
        generic_signature: generic_sig,
        default_result_type,
        variants: variant_ids.into_boxed_slice(),
        variant_families: variant_families.into_boxed_slice(),
        source: Some(SemanticSourceSpan::new(module_id.clone(), enum_def.range)),
    };

    EnumDeclarationProduct {
        info: Arc::new(enum_info),
        variants: Arc::from(variant_infos.into_boxed_slice()),
        diagnostics: Arc::from(diagnostics.into_boxed_slice()),
    }
}
