//! Validation and semantic-product construction for source type-member declarations.

use crate::db::product::TypeMemberDeclarationProduct;
use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::identity::{DeclarationId, DispatchSide, ModuleId};
use crate::types::annotation::{
    KindResolution, TypeFormationSite, resolve_contextual_kind_syntax,
};
use crate::types::type_member::{
    IntrinsicTypeOperation, TypeMemberDeclarationId, TypeMemberDomain, TypeMemberParameter, TypeMemberSelector, TypeMemberSignature,
    lookup_intrinsic_type_operation,
};
use crate::types::{KindId, TypeResolver, TypeStore};
use phalcom_ast::ast::{TypeMemberDef, TypeMemberParameterDomainSyntax, TypeMemberResultSyntax};
use phalcom_common::modifier::ImplementationKind;

/// Checks one source type-member declaration and returns its canonical product when valid.
///
/// This function validates domains but does not perform `::` lookup. The stable product
/// identity is owner-plus-canonical-selector; `AssociatedSurface` separately decides
/// whether that identity is admitted into the owner's namespace.
pub fn build_type_member_declaration(
    store: &mut TypeStore,
    owner: &DeclarationId,
    definition: &TypeMemberDef,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    module: &ModuleId,
) -> (Option<TypeMemberDeclarationProduct>, Vec<SemanticDiagnostic>) {
    let selector = TypeMemberSelector::from_syntax(&definition.selector);
    let id = TypeMemberDeclarationId::new(owner.clone(), selector.clone());
    let site = TypeFormationSite::member(module.clone(), owner.clone(), DispatchSide::Class);
    let mut diagnostics = Vec::new();
    let mut parameters = Vec::with_capacity(definition.parameters.len());
    let mut seen_parameters = std::collections::HashSet::new();
    let mut valid = true;

    for parameter in &definition.parameters {
        if !seen_parameters.insert(parameter.name.as_str()) {
            diagnostics.push(SemanticDiagnostic::error_in(
                module.clone(),
                DiagnosticCode::AnnotationUnresolved,
                format!("duplicate type-member parameter `{}`", parameter.name),
                parameter.name_range,
            ));
            valid = false;
            continue;
        }
        let domain = match &parameter.domain {
            TypeMemberParameterDomainSyntax::Kind(syntax) => {
                match resolve_contextual_kind_syntax(store, Some(declarations), Some(resolver), &site, syntax, &mut diagnostics) {
                    KindResolution::Ready(kind) => Some(TypeMemberDomain::Kind(kind)),
                    other => {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            module.clone(),
                            DiagnosticCode::AnnotationUnresolved,
                            format!("could not resolve type-member parameter domain: {other:?}"),
                            parameter.domain.range(),
                        ));
                        None
                    }
                }
            }
            TypeMemberParameterDomainSyntax::ConstType(annotation) => {
                let syntax = phalcom_ast::ast::StaticDomainSyntax::new(annotation.clone(), annotation.range);
                match crate::static_domain::resolve_static_domain(store, declarations, resolver, &site, &syntax, &mut diagnostics) {
                    crate::types::annotation::TypeFormationOutcome::Ready(domain) => {
                        let member_domain = match domain {
                            crate::static_domain::StaticValueDomain::ValueType(ty) => {
                                if let crate::types::store::TypeData::Nominal { declaration } = store.get(ty)
                                    && declaration == &crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Int)
                                {
                                    TypeMemberDomain::Kind(store.static_domain_kind(store.const_int_domain()))
                                } else {
                                    TypeMemberDomain::ValueType(ty)
                                }
                            }
                            crate::static_domain::StaticValueDomain::Kind(k) => TypeMemberDomain::Kind(k),
                        };
                        Some(member_domain)
                    }
                    other => {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            module.clone(),
                            DiagnosticCode::AnnotationUnresolved,
                            format!("could not resolve type-member parameter domain: {other:?}"),
                            parameter.domain.range(),
                        ));
                        None
                    }
                }
            }
        };
        if let Some(domain) = domain {
            parameters.push(TypeMemberParameter {
                name: parameter.name.clone().into_boxed_str(),
                domain,
            });
        } else {
            valid = false;
        }
    }

    let result_domain = match &definition.result {
        TypeMemberResultSyntax::Kind(syntax) => {
            match resolve_contextual_kind_syntax(store, Some(declarations), Some(resolver), &site, syntax, &mut diagnostics) {
                KindResolution::Ready(kind) => Some(TypeMemberDomain::Kind(kind)),
                other => {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        module.clone(),
                        DiagnosticCode::AnnotationUnresolved,
                        format!("could not resolve type-member result domain: {other:?}"),
                        definition.result.range(),
                    ));
                    None
                }
            }
        }
        TypeMemberResultSyntax::Type(annotation) => {
            let syntax = phalcom_ast::ast::StaticDomainSyntax::new(annotation.clone(), annotation.range);
            match crate::static_domain::resolve_static_domain(store, declarations, resolver, &site, &syntax, &mut diagnostics) {
                crate::types::annotation::TypeFormationOutcome::Ready(domain) => Some(TypeMemberDomain::from(domain)),
                other => {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        module.clone(),
                        DiagnosticCode::AnnotationUnresolved,
                        format!("could not resolve type-member result domain: {other:?}"),
                        definition.result.range(),
                    ));
                    None
                }
            }
        }
    };
    if result_domain.is_none() {
        valid = false;
    }

    if definition.where_clause.as_ref().is_some_and(|clause| !clause.constraints.is_empty()) {
        diagnostics.push(SemanticDiagnostic::error_in(
            module.clone(),
            DiagnosticCode::AnnotationUnsupported,
            "where constraints on type-member declarations are not yet supported",
            definition.where_clause.as_ref().map_or(definition.range, |clause| clause.range),
        ));
        valid = false;
    }

    let intrinsic = if definition.implementation == ImplementationKind::Intrinsic {
        let operation = declarations
            .intrinsic_type_owner(owner)
            .and_then(|owner| lookup_intrinsic_type_operation(owner, &selector));
        if operation.is_none() {
            diagnostics.push(SemanticDiagnostic::error_in(
                module.clone(),
                DiagnosticCode::AnnotationUnresolved,
                format!("no intrinsic type-member operation is defined for `{}`::`{}`", owner.name, selector.name()),
                definition.selector.range(),
            ));
            valid = false;
        }
        operation
    } else {
        None
    };

    if let Some(operation) = intrinsic {
        match operation {
            IntrinsicTypeOperation::UnorderedShapeLookup => {
                let param_ok = parameters.len() == 1 && matches!(parameters[0].domain, TypeMemberDomain::ValueType(_));
                let result_ok = matches!(result_domain, Some(TypeMemberDomain::ValueType(_)));
                if !param_ok || !result_ok {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        module.clone(),
                        DiagnosticCode::AnnotationUnresolved,
                        format!(
                            "intrinsic type member `{}` has parameters/result {:?} -> {:?}; expected 1 value parameter and value result",
                            selector.name(),
                            parameters,
                            result_domain,
                        ),
                        definition.range,
                    ));
                    valid = false;
                }
            }
            IntrinsicTypeOperation::UnorderedShapeView | IntrinsicTypeOperation::UnorderedShapeTail => {
                let param_ok = parameters.is_empty();
                let result_ok = matches!(result_domain, Some(TypeMemberDomain::ValueType(_)));
                if !param_ok || !result_ok {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        module.clone(),
                        DiagnosticCode::AnnotationUnresolved,
                        format!(
                            "intrinsic type member `{}` has parameters/result {:?} -> {:?}; expected 0 parameters and value result",
                            selector.name(),
                            parameters,
                            result_domain,
                        ),
                        definition.range,
                    ));
                    valid = false;
                }
            }
            _ => {
                let (expected_parameters, expected_result) = operation.signature_kinds(store);
                let actual_parameters = parameters
                    .iter()
                    .map(|parameter| match &parameter.domain {
                        TypeMemberDomain::Kind(kind) => Some(*kind),
                        TypeMemberDomain::ValueType(_) => None,
                    })
                    .collect::<Vec<_>>();
                let actual_result = match result_domain {
                    Some(TypeMemberDomain::Kind(k)) => Some(k),
                    Some(TypeMemberDomain::ValueType(_)) => None,
                    None => Some(KindId::TYPE),
                };
                let matches_kinds = actual_parameters.iter().all(|p| p.is_some())
                    && actual_parameters.iter().map(|p| p.unwrap()).collect::<Vec<_>>().as_slice() == expected_parameters.as_ref()
                    && actual_result == Some(expected_result);
                if !matches_kinds {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        module.clone(),
                        DiagnosticCode::AnnotationUnresolved,
                        format!(
                            "intrinsic type member `{}` has parameter/result kinds {:?} -> {:?}; its evaluator requires {:?} -> {:?}",
                            selector.name(),
                            actual_parameters,
                            actual_result,
                            expected_parameters,
                            expected_result,
                        ),
                        definition.range,
                    ));
                    valid = false;
                }
            }
        }
    }

    if !valid {
        return (None, diagnostics);
    }
    let signature = TypeMemberSignature {
        selector,
        parameters: parameters.into_boxed_slice(),
        result_domain: result_domain.unwrap_or(TypeMemberDomain::Kind(KindId::TYPE)),
        intrinsic,
        implementation: definition.implementation,
    };
    (
        Some(TypeMemberDeclarationProduct {
            id,
            signature,
            source_range: definition.range,
        }),
        diagnostics,
    )
}
