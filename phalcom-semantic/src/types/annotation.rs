//! Source type annotation resolution.

use super::evidence::{DynamicReason, EvidenceAuthority, TypeEvidence, TypeKnowledge, UnknownReason};
use super::store::TypeStore;
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

/// Resolves an AST [`TypeAnnotation`] into semantic [`TypeKnowledge`].
pub fn resolve_type_annotation(
    store: &mut TypeStore,
    resolver: &dyn TypeResolver,
    current_module: &ModuleId,
    annotation: &TypeAnnotation,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeKnowledge {
    match &annotation.expr {
        TypeAnnotationExpr::Reference(sym_ref) => {
            let name = sym_ref.leaf_name();
            // 1. Builtins and special types
            match name {
                "Never" => {
                    return TypeKnowledge::Known(TypeEvidence {
                        ty: store.never(),
                        authority: EvidenceAuthority::Declared,
                        provenance: Default::default(),
                    })
                    .with_range(annotation.range);
                }
                "Unit" => {
                    return TypeKnowledge::Known(TypeEvidence {
                        ty: store.unit(),
                        authority: EvidenceAuthority::Declared,
                        provenance: Default::default(),
                    })
                    .with_range(annotation.range);
                }
                "Dynamic" => {
                    return TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape);
                }
                _ => {}
            }

            let members: Vec<String> = sym_ref.members.iter().map(|m| m.name.clone()).collect();
            if let Some(decl) = resolver.resolve_type_name(current_module, &sym_ref.root, &members) {
                let ty = store.nominal(decl);
                TypeKnowledge::Known(TypeEvidence {
                    ty,
                    authority: EvidenceAuthority::Declared,
                    provenance: Default::default(),
                })
                .with_range(annotation.range)
            } else {
                diagnostics.push(SemanticDiagnostic::error(
                    DiagnosticCode::AnnotationUnresolved,
                    format!("unresolved type `{}`", sym_ref.root),
                    annotation.range,
                ));
                TypeKnowledge::Unknown(UnknownReason::UnresolvedName(name.into()))
            }
        }
        TypeAnnotationExpr::Union { members, .. } => {
            let mut resolved_tys = Vec::new();
            for m in members {
                let k = resolve_type_annotation(store, resolver, current_module, m, diagnostics);
                if let Some(ty) = k.ty() {
                    resolved_tys.push(ty);
                } else if k.is_dynamic() {
                    return TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape);
                } else {
                    return TypeKnowledge::Unknown(UnknownReason::UnresolvedName("union member".into()));
                }
            }
            let union_ty = store.union(&resolved_tys);
            TypeKnowledge::Known(TypeEvidence {
                ty: union_ty,
                authority: EvidenceAuthority::Declared,
                provenance: Default::default(),
            })
            .with_range(annotation.range)
        }
        _ => {
            diagnostics.push(SemanticDiagnostic::error(
                DiagnosticCode::AnnotationUnsupported,
                "generic applications, tuples, and callable type annotations are deferred in this milestone",
                annotation.range,
            ));
            TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
        }
    }
}
