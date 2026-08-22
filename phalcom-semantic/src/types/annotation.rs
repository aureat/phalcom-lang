//! Source type annotation resolution.

use super::application::TypeApplicationError;
use super::evidence::{DynamicReason, EvidenceAuthority, TypeEvidence, TypeKnowledge, UnknownReason};
use super::id::KindId;
use super::store::{CallableParameterType, CallableType, TupleTypeElement, TypeStore};
use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::identity::{DeclarationId, ModuleId};
use phalcom_ast::ast::{TypeAnnotation, TypeAnnotationExpr};

/// Resolves type names to declaration identities or builtins.
pub trait TypeResolver {
    /// Resolve an unqualified or qualified nominal type name to a DeclarationId.
    fn resolve_type_name(&self, current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId>;
}

/// A standard resolver holding local declarations, imported declarations, and builtins.
#[derive(Clone, Debug, Default)]
pub struct SimpleTypeResolver {
    pub declarations: std::collections::HashMap<String, DeclarationId>,
}

impl SimpleTypeResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, decl: DeclarationId) {
        self.declarations.insert(name.into(), decl);
    }
}

impl TypeResolver for SimpleTypeResolver {
    fn resolve_type_name(&self, _current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId> {
        if members.is_empty() {
            self.declarations.get(root).cloned()
        } else {
            let full = format!("{}.{}", root, members.join("."));
            self.declarations.get(&full).cloned()
        }
    }
}

/// Result of resolving an AST type annotation into a type form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeFormResolution {
    Known(crate::types::id::TypeId),
    Dynamic,
    Unknown(UnknownReason),
}

/// Resolves an AST [`TypeAnnotation`] into a type constructor or proper type form.
pub fn resolve_type_form(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    current_module: &ModuleId,
    annotation: &TypeAnnotation,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormResolution {
    match &annotation.expr {
        TypeAnnotationExpr::Reference(sym_ref) => {
            let name = sym_ref.leaf_name();
            if sym_ref.members.is_empty() {
                match name {
                    "Never" => return TypeFormResolution::Known(store.never()),
                    "Unit" => return TypeFormResolution::Known(store.unit()),
                    "Dynamic" => return TypeFormResolution::Dynamic,
                    _ => {}
                }
            }

            let members: Vec<String> = sym_ref.members.iter().map(|m| m.name.clone()).collect();
            if let Some(decl) = resolver.resolve_type_name(current_module, &sym_ref.root, &members) {
                let form = declarations
                    .form(&decl)
                    .unwrap_or_else(|| store.nominal_type(decl));
                TypeFormResolution::Known(form)
            } else {
                diagnostics.push(SemanticDiagnostic::error(
                    DiagnosticCode::AnnotationUnresolved,
                    format!("unresolved type `{}`", sym_ref.root),
                    annotation.range,
                ));
                TypeFormResolution::Unknown(UnknownReason::UnresolvedName(name.into()))
            }
        }
        TypeAnnotationExpr::Application {
            origin,
            arguments,
            range: _,
        } => {
            let origin_res = resolve_type_form(
                store,
                declarations,
                resolver,
                current_module,
                origin,
                diagnostics,
            );
            let origin_ty = match origin_res {
                TypeFormResolution::Known(ty) => ty,
                TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
                TypeFormResolution::Unknown(reason) => return TypeFormResolution::Unknown(reason),
            };

            let mut arg_tys = Vec::with_capacity(arguments.len());
            for arg in arguments {
                let arg_res = resolve_type_form(
                    store,
                    declarations,
                    resolver,
                    current_module,
                    arg,
                    diagnostics,
                );
                match arg_res {
                    TypeFormResolution::Known(ty) => arg_tys.push(ty),
                    TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
                    TypeFormResolution::Unknown(reason) => return TypeFormResolution::Unknown(reason),
                }
            }

            match store.apply_type_form(origin_ty, &arg_tys) {
                Ok(applied) => TypeFormResolution::Known(applied),
                Err(err) => {
                    let code = match &err {
                        TypeApplicationError::NotAConstructor { .. } => {
                            DiagnosticCode::ApplicationNotConstructor
                        }
                        TypeApplicationError::TooManyArguments { .. } => {
                            DiagnosticCode::ApplicationTooManyArguments
                        }
                        TypeApplicationError::ArgumentKindMismatch { .. } => {
                            DiagnosticCode::ApplicationArgumentKindMismatch
                        }
                    };
                    diagnostics.push(SemanticDiagnostic::error(
                        code,
                        format!("{err}"),
                        annotation.range,
                    ));
                    TypeFormResolution::Unknown(UnknownReason::UnannotatedDeclaration)
                }
            }
        }
        TypeAnnotationExpr::Tuple {
            elements,
            range: _,
        } => {
            let mut tuple_elements = Vec::with_capacity(elements.len());
            for elem in elements {
                let elem_res = resolve_type_form(
                    store,
                    declarations,
                    resolver,
                    current_module,
                    &elem.ty,
                    diagnostics,
                );
                let ty = match elem_res {
                    TypeFormResolution::Known(ty) => {
                        if store.kind_of(ty) != KindId::TYPE {
                            diagnostics.push(SemanticDiagnostic::error(
                                DiagnosticCode::KindExpectedType,
                                "tuple element must be a proper type",
                                elem.range,
                            ));
                            return TypeFormResolution::Unknown(UnknownReason::UnannotatedDeclaration);
                        }
                        ty
                    }
                    TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
                    TypeFormResolution::Unknown(reason) => return TypeFormResolution::Unknown(reason),
                };
                tuple_elements.push(TupleTypeElement {
                    label: elem.label.clone().map(Into::into),
                    ty,
                });
            }
            let tuple_ty = store.tuple(tuple_elements.into_boxed_slice());
            TypeFormResolution::Known(tuple_ty)
        }
        TypeAnnotationExpr::Callable {
            parameters,
            result,
            range: _,
        } => {
            let mut param_types = Vec::with_capacity(parameters.len());
            for param in parameters {
                let param_res = resolve_type_form(
                    store,
                    declarations,
                    resolver,
                    current_module,
                    &param.ty,
                    diagnostics,
                );
                let ty = match param_res {
                    TypeFormResolution::Known(ty) => {
                        if store.kind_of(ty) != KindId::TYPE {
                            diagnostics.push(SemanticDiagnostic::error(
                                DiagnosticCode::KindExpectedType,
                                "callable parameter must be a proper type",
                                param.range,
                            ));
                            return TypeFormResolution::Unknown(UnknownReason::UnannotatedDeclaration);
                        }
                        ty
                    }
                    TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
                    TypeFormResolution::Unknown(reason) => return TypeFormResolution::Unknown(reason),
                };
                param_types.push(CallableParameterType {
                    label: param.label.clone().map(Into::into),
                    ty,
                    rest: param.rest,
                });
            }

            let result_res = resolve_type_form(
                store,
                declarations,
                resolver,
                current_module,
                result,
                diagnostics,
            );
            let return_type = match result_res {
                TypeFormResolution::Known(ty) => {
                    if store.kind_of(ty) != KindId::TYPE {
                        diagnostics.push(SemanticDiagnostic::error(
                            DiagnosticCode::KindExpectedType,
                            "callable return type must be a proper type",
                            result.range,
                        ));
                        return TypeFormResolution::Unknown(UnknownReason::UnannotatedDeclaration);
                    }
                    ty
                }
                TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
                TypeFormResolution::Unknown(reason) => return TypeFormResolution::Unknown(reason),
            };

            let callable_ty = store.callable(CallableType {
                parameters: param_types.into_boxed_slice(),
                return_type,
            });
            TypeFormResolution::Known(callable_ty)
        }
        TypeAnnotationExpr::Union { members, .. } => {
            let mut resolved_tys = Vec::new();
            for m in members {
                let k = resolve_type_form(
                    store,
                    declarations,
                    resolver,
                    current_module,
                    m,
                    diagnostics,
                );
                match k {
                    TypeFormResolution::Known(ty) => {
                        if store.kind_of(ty) != KindId::TYPE {
                            diagnostics.push(SemanticDiagnostic::error(
                                DiagnosticCode::KindExpectedType,
                                "union member must be a proper type",
                                m.range,
                            ));
                            return TypeFormResolution::Unknown(UnknownReason::UnannotatedDeclaration);
                        }
                        resolved_tys.push(ty);
                    }
                    TypeFormResolution::Dynamic => {
                        return TypeFormResolution::Dynamic;
                    }
                    TypeFormResolution::Unknown(reason) => {
                        return TypeFormResolution::Unknown(reason);
                    }
                }
            }
            let union_ty = store.union(&resolved_tys);
            TypeFormResolution::Known(union_ty)
        }
    }
}

/// Resolves an AST [`TypeAnnotation`] into semantic [`TypeKnowledge`] representing a proper value type.
pub fn resolve_type_annotation(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    current_module: &ModuleId,
    annotation: &TypeAnnotation,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeKnowledge {
    let form_res = resolve_type_form(
        store,
        declarations,
        resolver,
        current_module,
        annotation,
        diagnostics,
    );
    match form_res {
        TypeFormResolution::Known(ty) => {
            if store.kind_of(ty) != KindId::TYPE {
                diagnostics.push(SemanticDiagnostic::error(
                    DiagnosticCode::AnnotationUnsaturatedConstructor,
                    "type constructor requires type arguments and cannot be used directly as a value type",
                    annotation.range,
                ));
                TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
            } else {
                TypeKnowledge::Known(TypeEvidence {
                    ty,
                    authority: EvidenceAuthority::Declared,
                    provenance: Default::default(),
                })
                .with_range(annotation.range)
            }
        }
        TypeFormResolution::Dynamic => TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape),
        TypeFormResolution::Unknown(reason) => TypeKnowledge::Unknown(reason),
    }
}
