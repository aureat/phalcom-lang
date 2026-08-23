//! Source type annotation resolution.

use super::application::TypeApplicationError;
use super::evidence::{DynamicReason, EvidenceAuthority, TypeEvidence, TypeKnowledge, UnknownReason};
use super::id::KindId;
use super::parameter::{GenericConstraint, GenericSignature, SelfRole, SelfTypeTerm, TypeParameterData, TypeParameterOwner, TypeTerm};
use super::store::{CallableParameterType, CallableType, RecordTypeField, TupleTypeElement, TypeStore};
use super::type_lambda::ScopedTypeData;
use super::variance::Variance;
use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::identity::{DeclarationId, DispatchSide, ModuleId};
use phalcom_ast::ast::{GenericConstraintSyntax, GenericParameterSyntax, KindSyntax, TypeAnnotation, TypeAnnotationExpr, VarianceSyntax, WhereClauseSyntax};

/// Resolves type names to declaration identities or builtins.
pub trait TypeResolver {
    /// Resolve an unqualified or qualified nominal type name to a DeclarationId.
    fn resolve_type_name(&self, current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId>;

    /// Resolve an in-scope type parameter name to its interned parameter TypeId.
    fn resolve_type_parameter(&self, _name: &str) -> Option<crate::types::id::TypeId> {
        None
    }

    /// The declaration currently enclosing the type resolution context, if any.
    fn current_declaration(&self) -> Option<DeclarationId> {
        None
    }
}

/// A scoped type resolver overlaying lexical type parameters on top of a parent resolver.
pub struct ScopedTypeResolver<'a> {
    pub parent: &'a dyn TypeResolver,
    pub type_parameters: std::collections::HashMap<String, crate::types::id::TypeId>,
}

impl<'a> TypeResolver for ScopedTypeResolver<'a> {
    fn resolve_type_name(&self, current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId> {
        self.parent.resolve_type_name(current_module, root, members)
    }

    fn resolve_type_parameter(&self, name: &str) -> Option<crate::types::id::TypeId> {
        if let Some(&ty) = self.type_parameters.get(name) {
            Some(ty)
        } else {
            self.parent.resolve_type_parameter(name)
        }
    }

    fn current_declaration(&self) -> Option<DeclarationId> {
        self.parent.current_declaration()
    }
}

/// A standard resolver holding local declarations, imported declarations, and builtins.
#[derive(Clone, Debug, Default)]
pub struct SimpleTypeResolver {
    pub declarations: std::collections::HashMap<String, DeclarationId>,
    pub type_parameters: std::collections::HashMap<String, crate::types::id::TypeId>,
    pub enclosing_declaration: Option<DeclarationId>,
}

impl SimpleTypeResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, decl: DeclarationId) {
        self.declarations.insert(name.into(), decl);
    }

    pub fn insert_parameter(&mut self, name: impl Into<String>, ty: crate::types::id::TypeId) {
        self.type_parameters.insert(name.into(), ty);
    }

    pub fn with_enclosing_declaration(mut self, decl: DeclarationId) -> Self {
        self.enclosing_declaration = Some(decl);
        self
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

    fn resolve_type_parameter(&self, name: &str) -> Option<crate::types::id::TypeId> {
        self.type_parameters.get(name).copied()
    }

    fn current_declaration(&self) -> Option<DeclarationId> {
        self.enclosing_declaration.clone()
    }
}

/// Result of resolving an AST type annotation into a type form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeFormResolution {
    Known(crate::types::id::TypeId),
    Dynamic,
    Unknown(UnknownReason),
}

/// Resolves a kind syntax node to a canonical [`KindId`].
pub fn resolve_kind_syntax(store: &mut TypeStore, kind: &KindSyntax) -> KindId {
    match kind {
        KindSyntax::Type(_) => KindId::TYPE,
        KindSyntax::RecordRow(_) => KindId::RECORD_ROW,
        KindSyntax::Grouped { inner, .. } => resolve_kind_syntax(store, inner),
        KindSyntax::Arrow { parameter, result, .. } => {
            let p_kind = resolve_kind_syntax(store, parameter);
            let r_kind = resolve_kind_syntax(store, result);
            store.arrow_kind(Box::new([p_kind]), r_kind)
        }
        KindSyntax::Invalid { .. } => KindId::TYPE,
    }
}

/// Lowers an AST variance marker into semantic [`Variance`].
pub fn lower_variance(variance: VarianceSyntax) -> Variance {
    match variance {
        VarianceSyntax::Invariant => Variance::Invariant,
        VarianceSyntax::Covariant => Variance::Covariant,
        VarianceSyntax::Contravariant => Variance::Contravariant,
    }
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
        TypeAnnotationExpr::Unit { .. } => TypeFormResolution::Known(store.unit()),
        TypeAnnotationExpr::Dynamic { .. } => TypeFormResolution::Dynamic,
        TypeAnnotationExpr::Never { .. } => TypeFormResolution::Known(store.never()),
        TypeAnnotationExpr::SelfType { range } => {
            if let Some(decl) = resolver.current_declaration() {
                let term = SelfTypeTerm {
                    owner: decl,
                    side: DispatchSide::Instance,
                    role: SelfRole::InstanceType,
                };
                TypeFormResolution::Known(store.self_type(term))
            } else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    current_module.clone(),
                    DiagnosticCode::AnnotationUnresolved,
                    "Self type is only valid within a class declaration or method context",
                    *range,
                ));
                TypeFormResolution::Unknown(UnknownReason::UnresolvedName("Self".into()))
            }
        }
        TypeAnnotationExpr::Reference(sym_ref) => {
            let name = sym_ref.leaf_name();
            if sym_ref.members.is_empty() {
                if let Some(param_ty) = resolver.resolve_type_parameter(name) {
                    return TypeFormResolution::Known(param_ty);
                }
                match name {
                    "Never" => return TypeFormResolution::Known(store.never()),
                    "Unit" => return TypeFormResolution::Known(store.unit()),
                    "Dynamic" => return TypeFormResolution::Dynamic,
                    _ => {}
                }
            }

            let members: Vec<String> = sym_ref.members.iter().map(|m| m.name.clone()).collect();
            if let Some(decl) = resolver.resolve_type_name(current_module, &sym_ref.root, &members) {
                let form = declarations.form(&decl).unwrap_or_else(|| store.nominal_type(decl));
                TypeFormResolution::Known(form)
            } else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    current_module.clone(),
                    DiagnosticCode::AnnotationUnresolved,
                    format!("unresolved type `{}`", sym_ref.root),
                    annotation.range,
                ));
                TypeFormResolution::Unknown(UnknownReason::UnresolvedName(name.into()))
            }
        }
        TypeAnnotationExpr::Application { origin, arguments, range: _ } => {
            let origin_res = resolve_type_form(store, declarations, resolver, current_module, origin, diagnostics);
            let origin_ty = match origin_res {
                TypeFormResolution::Known(ty) => ty,
                TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
                TypeFormResolution::Unknown(reason) => return TypeFormResolution::Unknown(reason),
            };

            let mut arg_tys = Vec::with_capacity(arguments.len());
            for arg in arguments {
                let arg_res = resolve_type_form(store, declarations, resolver, current_module, arg, diagnostics);
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
                        TypeApplicationError::NotAConstructor { .. } => DiagnosticCode::ApplicationNotConstructor,
                        TypeApplicationError::TooManyArguments { .. } => DiagnosticCode::ApplicationTooManyArguments,
                        TypeApplicationError::ArgumentKindMismatch { .. } => DiagnosticCode::ApplicationArgumentKindMismatch,
                    };
                    diagnostics.push(SemanticDiagnostic::error_in(current_module.clone(), code, format!("{err}"), annotation.range));
                    TypeFormResolution::Unknown(UnknownReason::UnannotatedDeclaration)
                }
            }
        }
        TypeAnnotationExpr::Tuple { elements, range: _ } => {
            let mut tuple_elements = Vec::with_capacity(elements.len());
            for elem in elements {
                let elem_res = resolve_type_form(store, declarations, resolver, current_module, &elem.ty, diagnostics);
                let ty = match elem_res {
                    TypeFormResolution::Known(ty) => {
                        if store.kind_of(ty) != KindId::TYPE {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
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
        TypeAnnotationExpr::Record { fields, tail: _, range: _ } => {
            let mut record_fields = Vec::with_capacity(fields.len());
            let mut seen_names = std::collections::HashSet::new();
            for field in fields {
                if !seen_names.insert(field.name.clone()) {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        current_module.clone(),
                        DiagnosticCode::KindExpectedType,
                        format!("duplicate field `{}` in record type annotation", field.name),
                        field.range,
                    ));
                    return TypeFormResolution::Unknown(UnknownReason::UnannotatedDeclaration);
                }
                let f_res = resolve_type_form(store, declarations, resolver, current_module, &field.ty, diagnostics);
                let ty = match f_res {
                    TypeFormResolution::Known(ty) => {
                        if store.kind_of(ty) != KindId::TYPE {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
                                DiagnosticCode::KindExpectedType,
                                "record field must be a proper type",
                                field.range,
                            ));
                            return TypeFormResolution::Unknown(UnknownReason::UnannotatedDeclaration);
                        }
                        ty
                    }
                    TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
                    TypeFormResolution::Unknown(reason) => return TypeFormResolution::Unknown(reason),
                };
                record_fields.push(RecordTypeField {
                    name: field.name.clone().into(),
                    ty,
                });
            }
            let rec_ty = store.record(record_fields.into_boxed_slice());
            TypeFormResolution::Known(rec_ty)
        }
        TypeAnnotationExpr::Callable { parameters, result, range: _ } => {
            let mut param_types = Vec::with_capacity(parameters.len());
            for param in parameters {
                let param_res = resolve_type_form(store, declarations, resolver, current_module, &param.ty, diagnostics);
                let ty = match param_res {
                    TypeFormResolution::Known(ty) => {
                        if store.kind_of(ty) != KindId::TYPE {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
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

            let result_res = resolve_type_form(store, declarations, resolver, current_module, result, diagnostics);
            let return_type = match result_res {
                TypeFormResolution::Known(ty) => {
                    if store.kind_of(ty) != KindId::TYPE {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            current_module.clone(),
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
                let k = resolve_type_form(store, declarations, resolver, current_module, m, diagnostics);
                match k {
                    TypeFormResolution::Known(ty) => {
                        if store.kind_of(ty) != KindId::TYPE {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
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
        TypeAnnotationExpr::TypeLambda { parameters, body, range: _ } => {
            let mut param_kinds = Vec::with_capacity(parameters.len());
            for p in parameters {
                let kind = p.kind.as_ref().map_or(KindId::TYPE, |k| resolve_kind_syntax(store, k));
                param_kinds.push(kind);
            }

            let body_res = resolve_type_form(store, declarations, resolver, current_module, body, diagnostics);
            let body_ty = match body_res {
                TypeFormResolution::Known(ty) => ty,
                TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
                TypeFormResolution::Unknown(reason) => return TypeFormResolution::Unknown(reason),
            };

            let scoped_body = store.arena_mut().intern_scoped(ScopedTypeData::Free(body_ty));
            let result_kind = store.kind_of(body_ty);
            let lambda_ty = store.lambda(param_kinds.into_boxed_slice(), scoped_body, result_kind);
            TypeFormResolution::Known(lambda_ty)
        }
        TypeAnnotationExpr::Invalid { message, range } => {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnresolved,
                message.clone(),
                *range,
            ));
            TypeFormResolution::Unknown(UnknownReason::SyntaxError)
        }
    }
}

/// Resolves generic parameters and where constraints into a [`GenericSignature`].
// Each argument is a separate scope/type-resolution input, so grouping them would obscure ownership.
#[allow(clippy::too_many_arguments)]
pub fn resolve_generic_signature(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    current_module: &ModuleId,
    owner: TypeParameterOwner,
    params: &[GenericParameterSyntax],
    where_clause: Option<&WhereClauseSyntax>,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> GenericSignature {
    let mut param_ids = Vec::with_capacity(params.len());
    for (idx, p) in params.iter().enumerate() {
        let kind = p.kind.as_ref().map_or(KindId::TYPE, |k| resolve_kind_syntax(store, k));
        let variance = lower_variance(p.variance);
        let data = TypeParameterData::new(owner.clone(), idx as u32, p.name.clone(), kind)
            .with_variance(variance)
            .with_source(crate::diagnostic::SemanticSourceSpan::new(current_module.clone(), p.range));
        let param_id = store.intern_type_parameter(data);
        param_ids.push(param_id);
    }

    let mut param_map = std::collections::HashMap::new();
    for (p, &param_id) in params.iter().zip(param_ids.iter()) {
        let param_form = store.parameter_form(param_id);
        param_map.insert(p.name.clone(), param_form);
    }
    let scoped_resolver = ScopedTypeResolver {
        parent: resolver,
        type_parameters: param_map,
    };

    let mut constraints = Vec::new();
    if let Some(clause) = where_clause {
        for c in &clause.constraints {
            match c {
                GenericConstraintSyntax::Subtype { lower, upper, range: _ } => {
                    let l_res = resolve_type_form(store, declarations, &scoped_resolver, current_module, lower, diagnostics);
                    let u_res = resolve_type_form(store, declarations, &scoped_resolver, current_module, upper, diagnostics);
                    if let (TypeFormResolution::Known(l_ty), TypeFormResolution::Known(u_ty)) = (l_res, u_res) {
                        constraints.push(GenericConstraint::Subtype {
                            lower: TypeTerm::Canonical(l_ty),
                            upper: TypeTerm::Canonical(u_ty),
                        });
                    }
                }
                GenericConstraintSyntax::Equivalent { left, right, range: _ } => {
                    let l_res = resolve_type_form(store, declarations, &scoped_resolver, current_module, left, diagnostics);
                    let r_res = resolve_type_form(store, declarations, &scoped_resolver, current_module, right, diagnostics);
                    if let (TypeFormResolution::Known(l_ty), TypeFormResolution::Known(r_ty)) = (l_res, r_res) {
                        constraints.push(GenericConstraint::Equivalent {
                            left: TypeTerm::Canonical(l_ty),
                            right: TypeTerm::Canonical(r_ty),
                        });
                    }
                }
                GenericConstraintSyntax::Invalid { message, range } => {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        current_module.clone(),
                        DiagnosticCode::AnnotationUnresolved,
                        message.clone(),
                        *range,
                    ));
                }
            }
        }
    }

    GenericSignature::with_constraints(owner, param_ids.into_boxed_slice(), constraints.into_boxed_slice())
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
    let form_res = resolve_type_form(store, declarations, resolver, current_module, annotation, diagnostics);
    match form_res {
        TypeFormResolution::Known(ty) => {
            if store.kind_of(ty) != KindId::TYPE {
                diagnostics.push(SemanticDiagnostic::error_in(
                    current_module.clone(),
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
