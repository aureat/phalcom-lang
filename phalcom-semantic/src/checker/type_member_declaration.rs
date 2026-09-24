//! Validation and semantic-product construction for source type-member declarations.

use crate::db::product::TypeMemberDeclarationProduct;
use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::identity::{DeclarationId, DispatchSide, ModuleId};
use crate::types::annotation::{
    KindResolution, TypeFormResolution, TypeFormationSite, resolve_contextual_kind_syntax, resolve_type_form, static_domain_kind_of_form,
};
use crate::types::type_member::{
    TypeMemberDeclarationId, TypeMemberDomain, TypeMemberParameter, TypeMemberSelector, TypeMemberSignature, lookup_intrinsic_type_operation,
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
            TypeMemberParameterDomainSyntax::Kind(syntax) => match resolve_contextual_kind_syntax(store, Some(declarations), Some(resolver), &site, syntax, &mut diagnostics) {
                KindResolution::Ready(kind) => Some(kind),
                other => {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        module.clone(),
                        DiagnosticCode::AnnotationUnresolved,
                        format!("could not resolve type-member parameter domain: {other:?}"),
                        parameter.domain.range(),
                    ));
                    None
                }
            },
            TypeMemberParameterDomainSyntax::ConstType(annotation) => {
                match resolve_type_form(store, declarations, resolver, &site, annotation, &mut diagnostics) {
                    TypeFormResolution::Ready(form) => Some(static_domain_kind_of_form(store, form)),
                    _ => None,
                }
            }
        };
        if let Some(kind) = domain {
            parameters.push(TypeMemberParameter {
                name: parameter.name.clone().into_boxed_str(),
                domain: TypeMemberDomain::Kind(kind),
            });
        } else {
            valid = false;
        }
    }

    let result_domain = match &definition.result {
        TypeMemberResultSyntax::Kind(syntax) => match resolve_contextual_kind_syntax(store, Some(declarations), Some(resolver), &site, syntax, &mut diagnostics) {
            KindResolution::Ready(kind) => Some(kind),
            other => {
                diagnostics.push(SemanticDiagnostic::error_in(
                    module.clone(),
                    DiagnosticCode::AnnotationUnresolved,
                    format!("could not resolve type-member result domain: {other:?}"),
                    definition.result.range(),
                ));
                None
            }
        },
        TypeMemberResultSyntax::Type(annotation) => match resolve_type_form(store, declarations, resolver, &site, annotation, &mut diagnostics) {
            TypeFormResolution::Ready(form) => Some(static_domain_kind_of_form(store, form)),
            _ => None,
        },
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

    if !valid {
        return (None, diagnostics);
    }
    let signature = TypeMemberSignature {
        selector,
        parameters: parameters.into_boxed_slice(),
        result_domain: TypeMemberDomain::Kind(result_domain.unwrap_or(KindId::TYPE)),
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
