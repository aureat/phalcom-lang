//! Source type annotation resolution.

use super::application::TypeApplicationError;
use super::evidence::{DynamicReason, EvidenceOrigin, TypeKnowledge, UnknownReason};
use super::id::{KindId, TypeId, TypeParameterId};
use super::kind::KindApplicationError;
use super::outcome::{BlockReason, BudgetReport};
use super::parameter::{GenericConstraint, GenericSignature, SelfRole, SelfTypeTerm, TypeParameterData, TypeParameterOwner, TypeTerm};
use super::store::{CallableParameterType, CallableType, RecordTypeField, TupleTypeElement, TypeStore};
use super::type_lambda::{ScopedCallableParameter, ScopedCallableType, ScopedRecordField, ScopedTupleElement, ScopedTypeData, TypeLambdaProvenance};
use super::variance::Variance;
use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::identity::{DeclarationId, DispatchSide, ModuleId};
use phalcom_ast::ast::{GenericConstraintSyntax, GenericParameterSyntax, KindSyntax, TypeAnnotation, TypeAnnotationExpr, VarianceSyntax, WhereClauseSyntax};

/// Lexical type-level binding domain. Record-row binders cannot be represented as value type forms.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypeLevelBinding {
    TypeForm(TypeId),
    RecordRow(TypeParameterId),
}

/// Context used while lowering source type forms.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeFormationSite {
    pub module: ModuleId,
    pub self_term: Option<SelfTypeTerm>,
}

impl TypeFormationSite {
    pub fn module(module: ModuleId) -> Self {
        Self { module, self_term: None }
    }

    pub fn member(module: ModuleId, owner: DeclarationId, side: DispatchSide) -> Self {
        Self {
            module,
            self_term: Some(SelfTypeTerm {
                owner,
                side,
                role: SelfRole::InstanceType,
            }),
        }
    }
}

/// Builds the binding used by a generic resolver without constructing `TypeData::Parameter` for rows.
pub fn type_level_binding_for_parameter(store: &mut TypeStore, parameter_id: TypeParameterId) -> TypeLevelBinding {
    let kind = store.type_parameter(parameter_id).kind;
    if kind == KindId::RECORD_ROW {
        TypeLevelBinding::RecordRow(parameter_id)
    } else {
        TypeLevelBinding::TypeForm(store.parameter_form(parameter_id))
    }
}

/// Resolves type names to declaration identities or builtins.
pub trait TypeResolver {
    /// Resolve an unqualified or qualified nominal type name to a DeclarationId.
    fn resolve_type_name(&self, current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId>;

    /// Resolve an in-scope type-level binding, including row-domain binders.
    fn resolve_type_level_binding(&self, _name: &str) -> Option<TypeLevelBinding> {
        None
    }

    /// Resolve a declaration-backed transparent alias form after name lookup.
    fn resolve_alias_form(&self, _declaration: &DeclarationId) -> Option<TypeId> {
        None
    }

    /// Compatibility lookup for expression/type-form consumers that accept only ordinary type forms.
    fn resolve_type_parameter(&self, name: &str) -> Option<TypeId> {
        match self.resolve_type_level_binding(name) {
            Some(TypeLevelBinding::TypeForm(form)) => Some(form),
            Some(TypeLevelBinding::RecordRow(_)) | None => None,
        }
    }
}

/// A scoped type resolver overlaying lexical type parameters on top of a parent resolver.
pub struct ScopedTypeResolver<'a> {
    pub parent: &'a dyn TypeResolver,
    pub type_parameters: std::collections::HashMap<String, TypeLevelBinding>,
}

impl<'a> TypeResolver for ScopedTypeResolver<'a> {
    fn resolve_type_name(&self, current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId> {
        self.parent.resolve_type_name(current_module, root, members)
    }

    fn resolve_type_level_binding(&self, name: &str) -> Option<TypeLevelBinding> {
        if let Some(&binding) = self.type_parameters.get(name) {
            Some(binding)
        } else {
            self.parent.resolve_type_level_binding(name)
        }
    }

    fn resolve_alias_form(&self, declaration: &DeclarationId) -> Option<TypeId> {
        self.parent.resolve_alias_form(declaration)
    }
}

/// A standard resolver holding local declarations, imported declarations, and builtins.
#[derive(Clone, Debug, Default)]
pub struct SimpleTypeResolver {
    pub declarations: std::collections::HashMap<String, DeclarationId>,
    pub type_parameters: std::collections::HashMap<String, TypeLevelBinding>,
}

impl SimpleTypeResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, decl: DeclarationId) {
        self.declarations.insert(name.into(), decl);
    }

    pub fn insert_parameter(&mut self, name: impl Into<String>, ty: crate::types::id::TypeId) {
        self.insert_type_form_binding(name, ty);
    }

    pub fn insert_type_form_binding(&mut self, name: impl Into<String>, form: TypeId) {
        self.type_parameters.insert(name.into(), TypeLevelBinding::TypeForm(form));
    }

    pub fn insert_record_row_binding(&mut self, name: impl Into<String>, parameter_id: TypeParameterId) {
        self.type_parameters.insert(name.into(), TypeLevelBinding::RecordRow(parameter_id));
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

    fn resolve_type_level_binding(&self, name: &str) -> Option<TypeLevelBinding> {
        self.type_parameters.get(name).copied()
    }
}

/// Reason a source type form is unavailable because its declaration product is absent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeFormationMissing {
    Annotation,
    DeclarationProduct(DeclarationId),
}

/// Reason source type-form resolution cannot identify a declaration yet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeFormationUnresolved {
    Name(Box<str>),
    SelfOutsideOwner,
}

/// Reason a source type form is malformed or violates formation rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeFormationInvalid {
    Syntax,
    InvalidKindSyntax,
    ExpectedProperType { actual: KindId },
    NotAConstructor,
    TooManyTypeArguments,
    TypeArgumentKindMismatch,
    MalformedTypeLambda,
    DuplicateRecordField(Box<str>),
    GenericConstraintOperandNotType,
    InvalidVariance,
    UnsupportedOpenRecordTail,
}

/// Result of resolving an AST type annotation into a type form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeFormationOutcome<T> {
    Ready(T),
    Dynamic,
    Missing(TypeFormationMissing),
    Unresolved(TypeFormationUnresolved),
    Invalid(TypeFormationInvalid),
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(String),
}

pub type TypeFormResolution = TypeFormationOutcome<TypeId>;
pub type KindResolution = TypeFormationOutcome<KindId>;

impl<T> TypeFormationOutcome<T> {
    pub fn ready(value: T) -> Self {
        Self::Ready(value)
    }

    pub fn as_ready(&self) -> Option<&T> {
        match self {
            Self::Ready(value) => Some(value),
            _ => None,
        }
    }

    pub fn into_ready(self) -> Option<T> {
        match self {
            Self::Ready(value) => Some(value),
            _ => None,
        }
    }

    pub fn is_terminal_failure(&self) -> bool {
        matches!(
            self,
            Self::Missing(_) | Self::Unresolved(_) | Self::Invalid(_) | Self::Blocked(_) | Self::Cancelled | Self::BudgetExceeded(_) | Self::InternalFailure(_)
        )
    }

    pub fn map_ready<U>(self, map: impl FnOnce(T) -> U) -> TypeFormationOutcome<U> {
        match self {
            Self::Ready(value) => TypeFormationOutcome::Ready(map(value)),
            Self::Dynamic => TypeFormationOutcome::Dynamic,
            Self::Missing(reason) => TypeFormationOutcome::Missing(reason),
            Self::Unresolved(reason) => TypeFormationOutcome::Unresolved(reason),
            Self::Invalid(reason) => TypeFormationOutcome::Invalid(reason),
            Self::Blocked(reason) => TypeFormationOutcome::Blocked(reason),
            Self::Cancelled => TypeFormationOutcome::Cancelled,
            Self::BudgetExceeded(report) => TypeFormationOutcome::BudgetExceeded(report),
            Self::InternalFailure(failure) => TypeFormationOutcome::InternalFailure(failure),
        }
    }

    pub fn and_then<U>(self, next: impl FnOnce(T) -> TypeFormationOutcome<U>) -> TypeFormationOutcome<U> {
        match self {
            Self::Ready(value) => next(value),
            Self::Dynamic => TypeFormationOutcome::Dynamic,
            Self::Missing(reason) => TypeFormationOutcome::Missing(reason),
            Self::Unresolved(reason) => TypeFormationOutcome::Unresolved(reason),
            Self::Invalid(reason) => TypeFormationOutcome::Invalid(reason),
            Self::Blocked(reason) => TypeFormationOutcome::Blocked(reason),
            Self::Cancelled => TypeFormationOutcome::Cancelled,
            Self::BudgetExceeded(report) => TypeFormationOutcome::BudgetExceeded(report),
            Self::InternalFailure(failure) => TypeFormationOutcome::InternalFailure(failure),
        }
    }
}

/// Resolves a kind syntax node to a canonical [`KindId`] without recovering invalid syntax.
pub fn resolve_kind_syntax(store: &mut TypeStore, kind: &KindSyntax) -> KindResolution {
    match kind {
        KindSyntax::Type(_) => KindResolution::Ready(KindId::TYPE),
        KindSyntax::RecordRow(_) => KindResolution::Ready(KindId::RECORD_ROW),
        KindSyntax::Grouped { inner, .. } => resolve_kind_syntax(store, inner),
        KindSyntax::Arrow { parameter, result, .. } => {
            let p_kind = match resolve_kind_syntax(store, parameter) {
                KindResolution::Ready(kind) => kind,
                KindResolution::Dynamic => return KindResolution::Dynamic,
                KindResolution::Missing(reason) => return KindResolution::Missing(reason),
                KindResolution::Unresolved(reason) => return KindResolution::Unresolved(reason),
                KindResolution::Invalid(reason) => return KindResolution::Invalid(reason),
                KindResolution::Blocked(reason) => return KindResolution::Blocked(reason),
                KindResolution::Cancelled => return KindResolution::Cancelled,
                KindResolution::BudgetExceeded(report) => return KindResolution::BudgetExceeded(report),
                KindResolution::InternalFailure(failure) => return KindResolution::InternalFailure(failure),
            };
            let r_kind = match resolve_kind_syntax(store, result) {
                KindResolution::Ready(kind) => kind,
                KindResolution::Dynamic => return KindResolution::Dynamic,
                KindResolution::Missing(reason) => return KindResolution::Missing(reason),
                KindResolution::Unresolved(reason) => return KindResolution::Unresolved(reason),
                KindResolution::Invalid(reason) => return KindResolution::Invalid(reason),
                KindResolution::Blocked(reason) => return KindResolution::Blocked(reason),
                KindResolution::Cancelled => return KindResolution::Cancelled,
                KindResolution::BudgetExceeded(report) => return KindResolution::BudgetExceeded(report),
                KindResolution::InternalFailure(failure) => return KindResolution::InternalFailure(failure),
            };
            KindResolution::Ready(store.arrow_kind(Box::new([p_kind]), r_kind))
        }
        KindSyntax::Invalid { .. } => KindResolution::Invalid(TypeFormationInvalid::InvalidKindSyntax),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenericBinderSite {
    NominalDeclaration,
    Callable,
    TypeAlias,
}

#[derive(Clone, Debug)]
struct ScopedBinder {
    name: Box<str>,
    kind: KindId,
}

#[derive(Default)]
struct ScopedBinderStack {
    layers: Vec<Box<[ScopedBinder]>>,
}

impl ScopedBinderStack {
    fn push(&mut self, binders: Box<[ScopedBinder]>) {
        self.layers.push(binders);
    }

    fn pop(&mut self) {
        self.layers.pop();
    }

    fn resolve(&self, name: &str) -> Option<(u32, u32, KindId)> {
        for (depth, layer) in self.layers.iter().rev().enumerate() {
            if let Some((index, binder)) = layer.iter().enumerate().find(|(_, binder)| binder.name.as_ref() == name) {
                return Some((depth as u32, index as u32, binder.kind));
            }
        }
        None
    }
}

macro_rules! scoped_ready_or_propagate {
    ($expr:expr) => {
        match $expr {
            TypeFormationOutcome::Ready(value) => value,
            TypeFormationOutcome::Dynamic => return TypeFormationOutcome::Dynamic,
            TypeFormationOutcome::Missing(reason) => return TypeFormationOutcome::Missing(reason),
            TypeFormationOutcome::Unresolved(reason) => return TypeFormationOutcome::Unresolved(reason),
            TypeFormationOutcome::Invalid(reason) => return TypeFormationOutcome::Invalid(reason),
            TypeFormationOutcome::Blocked(reason) => return TypeFormationOutcome::Blocked(reason),
            TypeFormationOutcome::Cancelled => return TypeFormationOutcome::Cancelled,
            TypeFormationOutcome::BudgetExceeded(report) => return TypeFormationOutcome::BudgetExceeded(report),
            TypeFormationOutcome::InternalFailure(failure) => return TypeFormationOutcome::InternalFailure(failure),
        }
    };
}

fn intern_scoped_free(store: &mut TypeStore, ty: TypeId) -> crate::types::id::ScopedTypeId {
    store.arena_mut().intern_scoped(ScopedTypeData::Free(ty))
}

fn scoped_kind(store: &mut TypeStore, scoped: crate::types::id::ScopedTypeId, binders: &ScopedBinderStack) -> KindResolution {
    match store.arena().get_scoped(scoped).clone() {
        ScopedTypeData::Bound { depth, index } => binders
            .layers
            .get(binders.layers.len().saturating_sub(1).saturating_sub(depth as usize))
            .and_then(|layer| layer.get(index as usize))
            .map_or(KindResolution::Invalid(TypeFormationInvalid::MalformedTypeLambda), |binder| {
                KindResolution::Ready(binder.kind)
            }),
        ScopedTypeData::Free(ty) => KindResolution::Ready(store.kind_of(ty)),
        ScopedTypeData::Applied { origin, arguments } => {
            let origin_kind = scoped_ready_or_propagate!(scoped_kind(store, origin, binders));
            let mut argument_kinds = Vec::with_capacity(arguments.len());
            for argument in arguments {
                argument_kinds.push(scoped_ready_or_propagate!(scoped_kind(store, argument, binders)));
            }
            match store.apply_kind(origin_kind, &argument_kinds) {
                Ok(kind) => KindResolution::Ready(kind),
                Err(KindApplicationError::NotApplicable { .. }) => KindResolution::Invalid(TypeFormationInvalid::NotAConstructor),
                Err(KindApplicationError::TooManyArguments { .. }) => KindResolution::Invalid(TypeFormationInvalid::TooManyTypeArguments),
                Err(KindApplicationError::ArgumentKindMismatch { .. }) => KindResolution::Invalid(TypeFormationInvalid::TypeArgumentKindMismatch),
            }
        }
        ScopedTypeData::Union(_) | ScopedTypeData::Tuple(_) | ScopedTypeData::Record(_) | ScopedTypeData::Callable(_) => KindResolution::Ready(KindId::TYPE),
        ScopedTypeData::Lambda(lambda_id) => {
            let lambda = store.arena().get_lambda(lambda_id).clone();
            if lambda.parameter_kinds.is_empty() {
                KindResolution::Ready(lambda.result_kind)
            } else {
                KindResolution::Ready(store.arrow_kind(lambda.parameter_kinds, lambda.result_kind))
            }
        }
    }
}

fn require_scoped_proper(
    store: &mut TypeStore,
    scoped: crate::types::id::ScopedTypeId,
    binders: &ScopedBinderStack,
    current_module: &ModuleId,
    range: phalcom_common::range::SourceRange,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormationOutcome<()> {
    let kind = scoped_kind(store, scoped, binders);
    let actual = scoped_ready_or_propagate!(kind);
    if actual != KindId::TYPE {
        diagnostics.push(SemanticDiagnostic::error_in(
            current_module.clone(),
            DiagnosticCode::KindExpectedType,
            "type expression must have kind Type",
            range,
        ));
        TypeFormationOutcome::Invalid(TypeFormationInvalid::ExpectedProperType { actual })
    } else {
        TypeFormationOutcome::Ready(())
    }
}

#[allow(clippy::too_many_arguments)]
fn lower_scoped_type_form(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    binders: &mut ScopedBinderStack,
    annotation: &TypeAnnotation,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormationOutcome<crate::types::id::ScopedTypeId> {
    let current_module = &site.module;
    match &annotation.expr {
        TypeAnnotationExpr::Unit { .. } => {
            let unit = store.unit();
            TypeFormationOutcome::Ready(intern_scoped_free(store, unit))
        }
        TypeAnnotationExpr::Never { .. } => {
            let never = store.never();
            TypeFormationOutcome::Ready(intern_scoped_free(store, never))
        }
        TypeAnnotationExpr::Dynamic { .. } => TypeFormationOutcome::Dynamic,
        TypeAnnotationExpr::SelfType { .. } => {
            let form = resolve_type_form(store, declarations, resolver, site, annotation, diagnostics);
            form.map_ready(|ty| intern_scoped_free(store, ty))
        }
        TypeAnnotationExpr::Reference(sym_ref) => {
            let name = sym_ref.leaf_name();
            if sym_ref.members.is_empty() {
                if let Some((depth, index, _kind)) = binders.resolve(name) {
                    return TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::Bound { depth, index }));
                }
                if let Some(binding) = resolver.resolve_type_level_binding(name) {
                    return match binding {
                        TypeLevelBinding::TypeForm(form) => TypeFormationOutcome::Ready(intern_scoped_free(store, form)),
                        TypeLevelBinding::RecordRow(_) => {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
                                DiagnosticCode::KindExpectedType,
                                "record-row generic binding cannot be used as an ordinary type",
                                annotation.range,
                            ));
                            TypeFormationOutcome::Invalid(TypeFormationInvalid::ExpectedProperType { actual: KindId::RECORD_ROW })
                        }
                    };
                }
                match name {
                    "Never" => {
                        let never = store.never();
                        return TypeFormationOutcome::Ready(intern_scoped_free(store, never));
                    }
                    "Unit" => {
                        let unit = store.unit();
                        return TypeFormationOutcome::Ready(intern_scoped_free(store, unit));
                    }
                    "Dynamic" => return TypeFormationOutcome::Dynamic,
                    _ => {}
                }
            }

            let members: Vec<String> = sym_ref.members.iter().map(|member| member.name.clone()).collect();
            if let Some(declaration) = resolver.resolve_type_name(current_module, &sym_ref.root, &members) {
                if let Some(form) = declarations.form(&declaration).or_else(|| resolver.resolve_alias_form(&declaration)) {
                    TypeFormationOutcome::Ready(intern_scoped_free(store, form))
                } else {
                    TypeFormationOutcome::Missing(TypeFormationMissing::DeclarationProduct(declaration))
                }
            } else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    current_module.clone(),
                    DiagnosticCode::AnnotationUnresolved,
                    format!("unresolved type `{}`", sym_ref.root),
                    annotation.range,
                ));
                TypeFormationOutcome::Unresolved(TypeFormationUnresolved::Name(name.into()))
            }
        }
        TypeAnnotationExpr::Application { origin, arguments, .. } => {
            let scoped_origin = scoped_ready_or_propagate!(lower_scoped_type_form(store, declarations, resolver, site, binders, origin, diagnostics));
            let mut scoped_arguments = Vec::with_capacity(arguments.len());
            for argument in arguments {
                scoped_arguments.push(scoped_ready_or_propagate!(lower_scoped_type_form(
                    store,
                    declarations,
                    resolver,
                    site,
                    binders,
                    argument,
                    diagnostics,
                )));
            }
            let origin_kind = scoped_ready_or_propagate!(scoped_kind(store, scoped_origin, binders));
            let mut argument_kinds = Vec::with_capacity(scoped_arguments.len());
            for &argument in &scoped_arguments {
                argument_kinds.push(scoped_ready_or_propagate!(scoped_kind(store, argument, binders)));
            }
            match store.apply_kind(origin_kind, &argument_kinds) {
                Ok(_) => TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::Applied {
                    origin: scoped_origin,
                    arguments: scoped_arguments.into_boxed_slice(),
                })),
                Err(KindApplicationError::NotApplicable { .. }) => TypeFormationOutcome::Invalid(TypeFormationInvalid::NotAConstructor),
                Err(KindApplicationError::TooManyArguments { .. }) => TypeFormationOutcome::Invalid(TypeFormationInvalid::TooManyTypeArguments),
                Err(KindApplicationError::ArgumentKindMismatch { .. }) => TypeFormationOutcome::Invalid(TypeFormationInvalid::TypeArgumentKindMismatch),
            }
        }
        TypeAnnotationExpr::Tuple { elements, .. } => {
            let mut scoped_elements = Vec::with_capacity(elements.len());
            for element in elements {
                let ty = scoped_ready_or_propagate!(lower_scoped_type_form(store, declarations, resolver, site, binders, &element.ty, diagnostics,));
                scoped_ready_or_propagate!(require_scoped_proper(store, ty, binders, current_module, element.range, diagnostics));
                scoped_elements.push(ScopedTupleElement {
                    label: element.label.clone().map(Into::into),
                    ty,
                });
            }
            TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::Tuple(scoped_elements.into_boxed_slice())))
        }
        TypeAnnotationExpr::Record { fields, tail, .. } => {
            if let Some(tail) = tail {
                diagnostics.push(SemanticDiagnostic::error_in(
                    current_module.clone(),
                    DiagnosticCode::AnnotationUnsupported,
                    format!("open record type tail `{}` is not available in scoped type formation", tail.name),
                    annotation.range,
                ));
                return TypeFormationOutcome::Invalid(TypeFormationInvalid::UnsupportedOpenRecordTail);
            }
            let mut names = std::collections::HashSet::new();
            let mut scoped_fields = Vec::with_capacity(fields.len());
            for field in fields {
                if !names.insert(field.name.clone()) {
                    return TypeFormationOutcome::Invalid(TypeFormationInvalid::DuplicateRecordField(field.name.clone().into()));
                }
                let ty = scoped_ready_or_propagate!(lower_scoped_type_form(store, declarations, resolver, site, binders, &field.ty, diagnostics,));
                scoped_ready_or_propagate!(require_scoped_proper(store, ty, binders, current_module, field.range, diagnostics));
                scoped_fields.push(ScopedRecordField {
                    name: field.name.clone().into(),
                    ty,
                });
            }
            TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::Record(scoped_fields.into_boxed_slice())))
        }
        TypeAnnotationExpr::Callable { parameters, result, .. } => {
            let mut scoped_parameters = Vec::with_capacity(parameters.len());
            for parameter in parameters {
                let ty = scoped_ready_or_propagate!(lower_scoped_type_form(store, declarations, resolver, site, binders, &parameter.ty, diagnostics,));
                scoped_ready_or_propagate!(require_scoped_proper(store, ty, binders, current_module, parameter.range, diagnostics));
                scoped_parameters.push(ScopedCallableParameter {
                    label: parameter.label.clone().map(Into::into),
                    ty,
                    rest: parameter.rest,
                });
            }
            let scoped_return = scoped_ready_or_propagate!(lower_scoped_type_form(store, declarations, resolver, site, binders, result, diagnostics,));
            scoped_ready_or_propagate!(require_scoped_proper(store, scoped_return, binders, current_module, result.range, diagnostics));
            TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::Callable(ScopedCallableType {
                parameters: scoped_parameters.into_boxed_slice(),
                return_type: scoped_return,
            })))
        }
        TypeAnnotationExpr::Union { members, .. } => {
            let mut scoped_members = Vec::with_capacity(members.len());
            for member in members {
                let scoped = scoped_ready_or_propagate!(lower_scoped_type_form(store, declarations, resolver, site, binders, member, diagnostics,));
                scoped_ready_or_propagate!(require_scoped_proper(store, scoped, binders, current_module, member.range, diagnostics));
                scoped_members.push(scoped);
            }
            TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::Union(scoped_members.into_boxed_slice())))
        }
        TypeAnnotationExpr::TypeLambda { parameters, body, range } => {
            let lambda_id = scoped_ready_or_propagate!(lower_scoped_type_lambda(
                store,
                declarations,
                resolver,
                site,
                binders,
                parameters,
                body,
                *range,
                diagnostics,
            ));
            TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::Lambda(lambda_id)))
        }
        TypeAnnotationExpr::Invalid { message, range } => {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnresolved,
                message.clone(),
                *range,
            ));
            TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax)
        }
    }
}

/// Lowers a generic transparent alias body into one canonical type lambda.
pub(crate) fn lower_scoped_type_alias_form(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    signature: &GenericSignature,
    body: &TypeAnnotation,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormationOutcome<TypeId> {
    let mut binders = ScopedBinderStack::default();
    let parameters = signature
        .parameters
        .iter()
        .map(|&parameter| {
            let data = store.type_parameter(parameter);
            ScopedBinder {
                name: data.name.clone(),
                kind: data.kind,
            }
        })
        .collect::<Vec<_>>();
    let parameter_kinds = parameters.iter().map(|parameter| parameter.kind).collect::<Vec<_>>();
    binders.push(parameters.into_boxed_slice());
    let lowered = lower_scoped_type_form(store, declarations, resolver, site, &mut binders, body, diagnostics);
    let result = match lowered {
        TypeFormationOutcome::Ready(scoped_body) => match scoped_kind(store, scoped_body, &binders) {
            KindResolution::Ready(result_kind) => {
                let provenance = TypeLambdaProvenance {
                    parameter_names: signature
                        .parameters
                        .iter()
                        .map(|&parameter| store.type_parameter(parameter).name.clone())
                        .collect(),
                    parameter_sources: signature
                        .parameters
                        .iter()
                        .filter_map(|&parameter| store.type_parameter(parameter).source.clone())
                        .collect(),
                    lambda_source: None,
                };
                let lambda_id = store
                    .arena_mut()
                    .intern_lambda(parameter_kinds.into_boxed_slice(), scoped_body, result_kind, Some(provenance));
                TypeFormationOutcome::Ready(store.type_lambda(lambda_id))
            }
            KindResolution::Dynamic => TypeFormationOutcome::Dynamic,
            KindResolution::Missing(reason) => TypeFormationOutcome::Missing(reason),
            KindResolution::Unresolved(reason) => TypeFormationOutcome::Unresolved(reason),
            KindResolution::Invalid(reason) => TypeFormationOutcome::Invalid(reason),
            KindResolution::Blocked(reason) => TypeFormationOutcome::Blocked(reason),
            KindResolution::Cancelled => TypeFormationOutcome::Cancelled,
            KindResolution::BudgetExceeded(report) => TypeFormationOutcome::BudgetExceeded(report),
            KindResolution::InternalFailure(failure) => TypeFormationOutcome::InternalFailure(failure),
        },
        TypeFormationOutcome::Dynamic => TypeFormationOutcome::Dynamic,
        TypeFormationOutcome::Missing(reason) => TypeFormationOutcome::Missing(reason),
        TypeFormationOutcome::Unresolved(reason) => TypeFormationOutcome::Unresolved(reason),
        TypeFormationOutcome::Invalid(reason) => TypeFormationOutcome::Invalid(reason),
        TypeFormationOutcome::Blocked(reason) => TypeFormationOutcome::Blocked(reason),
        TypeFormationOutcome::Cancelled => TypeFormationOutcome::Cancelled,
        TypeFormationOutcome::BudgetExceeded(report) => TypeFormationOutcome::BudgetExceeded(report),
        TypeFormationOutcome::InternalFailure(failure) => TypeFormationOutcome::InternalFailure(failure),
    };
    binders.pop();
    result
}

#[allow(clippy::too_many_arguments)]
fn lower_scoped_type_lambda(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    binders: &mut ScopedBinderStack,
    parameters: &[phalcom_ast::ast::TypeLambdaParameter],
    body: &TypeAnnotation,
    range: phalcom_common::range::SourceRange,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormationOutcome<crate::types::id::TypeLambdaId> {
    let current_module = &site.module;
    let mut parameter_kinds = Vec::with_capacity(parameters.len());
    for parameter in parameters {
        let kind = match parameter.kind.as_ref() {
            None => KindResolution::Ready(KindId::TYPE),
            Some(kind) => resolve_kind_syntax(store, kind),
        };
        parameter_kinds.push(scoped_ready_or_propagate!(kind));
    }
    let scoped_binders: Vec<ScopedBinder> = parameters
        .iter()
        .zip(parameter_kinds.iter().copied())
        .map(|(parameter, kind)| ScopedBinder {
            name: parameter.name.clone().into_boxed_str(),
            kind,
        })
        .collect();
    binders.push(scoped_binders.into_boxed_slice());
    let lowered_body = lower_scoped_type_form(store, declarations, resolver, site, binders, body, diagnostics);
    let result = match lowered_body {
        TypeFormationOutcome::Ready(scoped_body) => match scoped_kind(store, scoped_body, binders) {
            TypeFormationOutcome::Ready(result_kind) => {
                let provenance = TypeLambdaProvenance {
                    parameter_names: parameters.iter().map(|parameter| parameter.name.clone().into_boxed_str()).collect(),
                    parameter_sources: parameters
                        .iter()
                        .map(|parameter| crate::diagnostic::SemanticSourceSpan::new(current_module.clone(), parameter.range))
                        .collect(),
                    lambda_source: Some(crate::diagnostic::SemanticSourceSpan::new(current_module.clone(), range)),
                };
                let lambda_id = store
                    .arena_mut()
                    .intern_lambda(parameter_kinds.into_boxed_slice(), scoped_body, result_kind, Some(provenance));
                TypeFormationOutcome::Ready(lambda_id)
            }
            TypeFormationOutcome::Dynamic => TypeFormationOutcome::Dynamic,
            TypeFormationOutcome::Missing(reason) => TypeFormationOutcome::Missing(reason),
            TypeFormationOutcome::Unresolved(reason) => TypeFormationOutcome::Unresolved(reason),
            TypeFormationOutcome::Invalid(reason) => TypeFormationOutcome::Invalid(reason),
            TypeFormationOutcome::Blocked(reason) => TypeFormationOutcome::Blocked(reason),
            TypeFormationOutcome::Cancelled => TypeFormationOutcome::Cancelled,
            TypeFormationOutcome::BudgetExceeded(report) => TypeFormationOutcome::BudgetExceeded(report),
            TypeFormationOutcome::InternalFailure(failure) => TypeFormationOutcome::InternalFailure(failure),
        },
        TypeFormationOutcome::Dynamic => TypeFormationOutcome::Dynamic,
        TypeFormationOutcome::Missing(reason) => TypeFormationOutcome::Missing(reason),
        TypeFormationOutcome::Unresolved(reason) => TypeFormationOutcome::Unresolved(reason),
        TypeFormationOutcome::Invalid(reason) => TypeFormationOutcome::Invalid(reason),
        TypeFormationOutcome::Blocked(reason) => TypeFormationOutcome::Blocked(reason),
        TypeFormationOutcome::Cancelled => TypeFormationOutcome::Cancelled,
        TypeFormationOutcome::BudgetExceeded(report) => TypeFormationOutcome::BudgetExceeded(report),
        TypeFormationOutcome::InternalFailure(failure) => TypeFormationOutcome::InternalFailure(failure),
    };
    binders.pop();
    result
}

/// Resolves an AST [`TypeAnnotation`] into a type constructor or proper type form.
pub fn resolve_type_form(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    annotation: &TypeAnnotation,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormResolution {
    let current_module = &site.module;
    match &annotation.expr {
        TypeAnnotationExpr::Unit { .. } => TypeFormResolution::Ready(store.unit()),
        TypeAnnotationExpr::Dynamic { .. } => TypeFormResolution::Dynamic,
        TypeAnnotationExpr::Never { .. } => TypeFormResolution::Ready(store.never()),
        TypeAnnotationExpr::SelfType { range } => {
            if let Some(term) = site.self_term.clone() {
                TypeFormResolution::Ready(store.self_type(term))
            } else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    current_module.clone(),
                    DiagnosticCode::AnnotationUnresolved,
                    "Self type is only valid within a class declaration or method context",
                    *range,
                ));
                TypeFormResolution::Unresolved(TypeFormationUnresolved::SelfOutsideOwner)
            }
        }
        TypeAnnotationExpr::Reference(sym_ref) => {
            let name = sym_ref.leaf_name();
            if sym_ref.members.is_empty() {
                if let Some(binding) = resolver.resolve_type_level_binding(name) {
                    return match binding {
                        TypeLevelBinding::TypeForm(form) => TypeFormResolution::Ready(form),
                        TypeLevelBinding::RecordRow(_) => {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
                                DiagnosticCode::KindExpectedType,
                                "record-row generic binding cannot be used as an ordinary type",
                                annotation.range,
                            ));
                            TypeFormResolution::Invalid(TypeFormationInvalid::ExpectedProperType { actual: KindId::RECORD_ROW })
                        }
                    };
                }
                match name {
                    "Never" => return TypeFormResolution::Ready(store.never()),
                    "Unit" => return TypeFormResolution::Ready(store.unit()),
                    "Dynamic" => return TypeFormResolution::Dynamic,
                    _ => {}
                }
            }

            let members: Vec<String> = sym_ref.members.iter().map(|m| m.name.clone()).collect();
            if let Some(decl) = resolver.resolve_type_name(current_module, &sym_ref.root, &members) {
                if let Some(form) = declarations.form(&decl).or_else(|| resolver.resolve_alias_form(&decl)) {
                    TypeFormResolution::Ready(form)
                } else {
                    TypeFormResolution::Missing(TypeFormationMissing::DeclarationProduct(decl))
                }
            } else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    current_module.clone(),
                    DiagnosticCode::AnnotationUnresolved,
                    format!("unresolved type `{}`", sym_ref.root),
                    annotation.range,
                ));
                TypeFormResolution::Unresolved(TypeFormationUnresolved::Name(name.into()))
            }
        }
        TypeAnnotationExpr::Application { origin, arguments, range: _ } => {
            let origin_res = resolve_type_form(store, declarations, resolver, site, origin, diagnostics);
            let origin_ty = match origin_res {
                TypeFormResolution::Ready(ty) => ty,
                TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
                TypeFormResolution::Unresolved(reason) => return TypeFormResolution::Unresolved(reason),
                TypeFormResolution::Missing(reason) => return TypeFormResolution::Missing(reason),
                TypeFormResolution::Invalid(reason) => return TypeFormResolution::Invalid(reason),
                TypeFormResolution::Blocked(reason) => return TypeFormResolution::Blocked(reason),
                TypeFormResolution::Cancelled => return TypeFormResolution::Cancelled,
                TypeFormResolution::BudgetExceeded(report) => return TypeFormResolution::BudgetExceeded(report),
                TypeFormResolution::InternalFailure(failure) => return TypeFormResolution::InternalFailure(failure),
            };

            let mut arg_tys = Vec::with_capacity(arguments.len());
            for arg in arguments {
                let arg_res = resolve_type_form(store, declarations, resolver, site, arg, diagnostics);
                match arg_res {
                    TypeFormResolution::Ready(ty) => arg_tys.push(ty),
                    TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
                    TypeFormResolution::Unresolved(reason) => return TypeFormResolution::Unresolved(reason),
                    TypeFormResolution::Missing(reason) => return TypeFormResolution::Missing(reason),
                    TypeFormResolution::Invalid(reason) => return TypeFormResolution::Invalid(reason),
                    TypeFormResolution::Blocked(reason) => return TypeFormResolution::Blocked(reason),
                    TypeFormResolution::Cancelled => return TypeFormResolution::Cancelled,
                    TypeFormResolution::BudgetExceeded(report) => return TypeFormResolution::BudgetExceeded(report),
                    TypeFormResolution::InternalFailure(failure) => return TypeFormResolution::InternalFailure(failure),
                }
            }

            match store.apply_type_form(origin_ty, &arg_tys) {
                Ok(applied) => TypeFormResolution::Ready(applied),
                Err(err) => {
                    let code = match &err {
                        TypeApplicationError::NotAConstructor { .. } => DiagnosticCode::ApplicationNotConstructor,
                        TypeApplicationError::TooManyArguments { .. } => DiagnosticCode::ApplicationTooManyArguments,
                        TypeApplicationError::ArgumentKindMismatch { .. } => DiagnosticCode::ApplicationArgumentKindMismatch,
                        TypeApplicationError::MalformedLambda => DiagnosticCode::ApplicationNotConstructor,
                    };
                    diagnostics.push(SemanticDiagnostic::error_in(current_module.clone(), code, format!("{err}"), annotation.range));
                    TypeFormResolution::Invalid(match err {
                        TypeApplicationError::NotAConstructor { .. } => TypeFormationInvalid::NotAConstructor,
                        TypeApplicationError::TooManyArguments { .. } => TypeFormationInvalid::TooManyTypeArguments,
                        TypeApplicationError::ArgumentKindMismatch { .. } => TypeFormationInvalid::TypeArgumentKindMismatch,
                        TypeApplicationError::MalformedLambda => TypeFormationInvalid::MalformedTypeLambda,
                    })
                }
            }
        }
        TypeAnnotationExpr::Tuple { elements, range: _ } => {
            let mut tuple_elements = Vec::with_capacity(elements.len());
            for elem in elements {
                let elem_res = resolve_type_form(store, declarations, resolver, site, &elem.ty, diagnostics);
                let ty = match elem_res {
                    TypeFormResolution::Ready(ty) => {
                        if store.kind_of(ty) != KindId::TYPE {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
                                DiagnosticCode::KindExpectedType,
                                "tuple element must be a proper type",
                                elem.range,
                            ));
                            return TypeFormResolution::Invalid(TypeFormationInvalid::ExpectedProperType { actual: store.kind_of(ty) });
                        }
                        ty
                    }
                    TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
                    TypeFormResolution::Unresolved(reason) => return TypeFormResolution::Unresolved(reason),
                    TypeFormResolution::Missing(reason) => return TypeFormResolution::Missing(reason),
                    TypeFormResolution::Invalid(reason) => return TypeFormResolution::Invalid(reason),
                    TypeFormResolution::Blocked(reason) => return TypeFormResolution::Blocked(reason),
                    TypeFormResolution::Cancelled => return TypeFormResolution::Cancelled,
                    TypeFormResolution::BudgetExceeded(report) => return TypeFormResolution::BudgetExceeded(report),
                    TypeFormResolution::InternalFailure(failure) => return TypeFormResolution::InternalFailure(failure),
                };
                tuple_elements.push(TupleTypeElement {
                    label: elem.label.clone().map(Into::into),
                    ty,
                });
            }
            let tuple_ty = store.tuple(tuple_elements.into_boxed_slice());
            TypeFormResolution::Ready(tuple_ty)
        }
        TypeAnnotationExpr::Record { fields, tail, range: _ } => {
            if let Some(tail) = tail {
                diagnostics.push(SemanticDiagnostic::error_in(
                    current_module.clone(),
                    DiagnosticCode::AnnotationUnsupported,
                    format!("open record type tail `{}` is not available in type formation", tail.name),
                    annotation.range,
                ));
                return TypeFormResolution::Invalid(TypeFormationInvalid::UnsupportedOpenRecordTail);
            }
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
                    return TypeFormResolution::Invalid(TypeFormationInvalid::DuplicateRecordField(field.name.clone().into()));
                }
                let f_res = resolve_type_form(store, declarations, resolver, site, &field.ty, diagnostics);
                let ty = match f_res {
                    TypeFormResolution::Ready(ty) => {
                        if store.kind_of(ty) != KindId::TYPE {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
                                DiagnosticCode::KindExpectedType,
                                "record field must be a proper type",
                                field.range,
                            ));
                            return TypeFormResolution::Invalid(TypeFormationInvalid::ExpectedProperType { actual: store.kind_of(ty) });
                        }
                        ty
                    }
                    TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
                    TypeFormResolution::Unresolved(reason) => return TypeFormResolution::Unresolved(reason),
                    TypeFormResolution::Missing(reason) => return TypeFormResolution::Missing(reason),
                    TypeFormResolution::Invalid(reason) => return TypeFormResolution::Invalid(reason),
                    TypeFormResolution::Blocked(reason) => return TypeFormResolution::Blocked(reason),
                    TypeFormResolution::Cancelled => return TypeFormResolution::Cancelled,
                    TypeFormResolution::BudgetExceeded(report) => return TypeFormResolution::BudgetExceeded(report),
                    TypeFormResolution::InternalFailure(failure) => return TypeFormResolution::InternalFailure(failure),
                };
                record_fields.push(RecordTypeField {
                    name: field.name.clone().into(),
                    ty,
                });
            }
            let rec_ty = store.record(record_fields.into_boxed_slice());
            TypeFormResolution::Ready(rec_ty)
        }
        TypeAnnotationExpr::Callable { parameters, result, range: _ } => {
            let mut param_types = Vec::with_capacity(parameters.len());
            for param in parameters {
                let param_res = resolve_type_form(store, declarations, resolver, site, &param.ty, diagnostics);
                let ty = match param_res {
                    TypeFormResolution::Ready(ty) => {
                        if store.kind_of(ty) != KindId::TYPE {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
                                DiagnosticCode::KindExpectedType,
                                "callable parameter must be a proper type",
                                param.range,
                            ));
                            return TypeFormResolution::Invalid(TypeFormationInvalid::ExpectedProperType { actual: store.kind_of(ty) });
                        }
                        ty
                    }
                    TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
                    TypeFormResolution::Unresolved(reason) => return TypeFormResolution::Unresolved(reason),
                    TypeFormResolution::Missing(reason) => return TypeFormResolution::Missing(reason),
                    TypeFormResolution::Invalid(reason) => return TypeFormResolution::Invalid(reason),
                    TypeFormResolution::Blocked(reason) => return TypeFormResolution::Blocked(reason),
                    TypeFormResolution::Cancelled => return TypeFormResolution::Cancelled,
                    TypeFormResolution::BudgetExceeded(report) => return TypeFormResolution::BudgetExceeded(report),
                    TypeFormResolution::InternalFailure(failure) => return TypeFormResolution::InternalFailure(failure),
                };
                param_types.push(CallableParameterType {
                    label: param.label.clone().map(Into::into),
                    ty,
                    rest: if param.rest {
                        phalcom_ast::ast::RestMode::Positional
                    } else {
                        phalcom_ast::ast::RestMode::None
                    },
                });
            }

            let result_res = resolve_type_form(store, declarations, resolver, site, result, diagnostics);
            let return_type = match result_res {
                TypeFormResolution::Ready(ty) => {
                    if store.kind_of(ty) != KindId::TYPE {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            current_module.clone(),
                            DiagnosticCode::KindExpectedType,
                            "callable return type must be a proper type",
                            result.range,
                        ));
                        return TypeFormResolution::Invalid(TypeFormationInvalid::ExpectedProperType { actual: store.kind_of(ty) });
                    }
                    ty
                }
                TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
                TypeFormResolution::Unresolved(reason) => return TypeFormResolution::Unresolved(reason),
                TypeFormResolution::Missing(reason) => return TypeFormResolution::Missing(reason),
                TypeFormResolution::Invalid(reason) => return TypeFormResolution::Invalid(reason),
                TypeFormResolution::Blocked(reason) => return TypeFormResolution::Blocked(reason),
                TypeFormResolution::Cancelled => return TypeFormResolution::Cancelled,
                TypeFormResolution::BudgetExceeded(report) => return TypeFormResolution::BudgetExceeded(report),
                TypeFormResolution::InternalFailure(failure) => return TypeFormResolution::InternalFailure(failure),
            };

            let callable_ty = store.callable(CallableType {
                parameters: param_types.into_boxed_slice(),
                return_type,
            });
            TypeFormResolution::Ready(callable_ty)
        }
        TypeAnnotationExpr::Union { members, .. } => {
            let mut resolved_tys = Vec::new();
            for m in members {
                let k = resolve_type_form(store, declarations, resolver, site, m, diagnostics);
                match k {
                    TypeFormResolution::Ready(ty) => {
                        if store.kind_of(ty) != KindId::TYPE {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
                                DiagnosticCode::KindExpectedType,
                                "union member must be a proper type",
                                m.range,
                            ));
                            return TypeFormResolution::Invalid(TypeFormationInvalid::ExpectedProperType { actual: store.kind_of(ty) });
                        }
                        resolved_tys.push(ty);
                    }
                    TypeFormResolution::Dynamic => {
                        return TypeFormResolution::Dynamic;
                    }
                    TypeFormResolution::Unresolved(reason) => {
                        return TypeFormResolution::Unresolved(reason);
                    }
                    TypeFormResolution::Missing(reason) => return TypeFormResolution::Missing(reason),
                    TypeFormResolution::Invalid(reason) => return TypeFormResolution::Invalid(reason),
                    TypeFormResolution::Blocked(reason) => return TypeFormResolution::Blocked(reason),
                    TypeFormResolution::Cancelled => return TypeFormResolution::Cancelled,
                    TypeFormResolution::BudgetExceeded(report) => return TypeFormResolution::BudgetExceeded(report),
                    TypeFormResolution::InternalFailure(failure) => return TypeFormResolution::InternalFailure(failure),
                }
            }
            let union_ty = store.union(&resolved_tys);
            TypeFormResolution::Ready(union_ty)
        }
        TypeAnnotationExpr::TypeLambda { parameters, body, range: _ } => {
            let mut binders = ScopedBinderStack::default();
            lower_scoped_type_lambda(
                store,
                declarations,
                resolver,
                site,
                &mut binders,
                parameters,
                body,
                annotation.range,
                diagnostics,
            )
            .map_ready(|lambda_id| store.type_lambda(lambda_id))
        }
        TypeAnnotationExpr::Invalid { message, range } => {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnresolved,
                message.clone(),
                *range,
            ));
            TypeFormResolution::Invalid(TypeFormationInvalid::Syntax)
        }
    }
}

/// Resolves generic parameters and where constraints into a [`GenericSignature`].
// Each argument is a separate scope/type-resolution input, so grouping them would obscure ownership.
fn generic_constraint_shape(store: &TypeStore, constraint: &GenericConstraint) -> Box<str> {
    fn term_shape(store: &TypeStore, term: &TypeTerm) -> String {
        match term {
            TypeTerm::Canonical(ty) => store.format_type(*ty),
            TypeTerm::SelfType(self_term) => format!("Self<{:?}:{:?}:{:?}>", self_term.owner, self_term.side, self_term.role),
            TypeTerm::Infer(variable) => format!("Infer<{variable:?}>"),
        }
    }
    match constraint {
        GenericConstraint::Subtype { lower, upper } => format!("Subtype({}, {})", term_shape(store, lower), term_shape(store, upper)),
        GenericConstraint::Equivalent { left, right } => format!("Equivalent({}, {})", term_shape(store, left), term_shape(store, right)),
    }
    .into_boxed_str()
}

#[allow(clippy::too_many_arguments)]
pub fn resolve_generic_signature(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    owner: TypeParameterOwner,
    binder_site: GenericBinderSite,
    params: &[GenericParameterSyntax],
    where_clause: Option<&WhereClauseSyntax>,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormationOutcome<GenericSignature> {
    let current_module = &site.module;
    let mut pending = Vec::with_capacity(params.len());
    for p in params {
        let kind = match p.kind.as_ref() {
            None => KindResolution::Ready(KindId::TYPE),
            Some(syntax) => resolve_kind_syntax(store, syntax),
        };
        if let KindResolution::Invalid(reason) = &kind {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnresolved,
                format!("invalid generic parameter kind: {reason:?}"),
                p.range,
            ));
        }
        let kind = scoped_ready_or_propagate!(kind);
        let variance = lower_variance(p.variance);
        if binder_site != GenericBinderSite::NominalDeclaration && variance != Variance::Invariant {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnsupported,
                "variance annotations are only valid on nominal declaration parameters",
                p.range,
            ));
            return TypeFormationOutcome::Invalid(TypeFormationInvalid::InvalidVariance);
        }
        pending.push((p, kind, variance));
    }

    let mut param_ids = Vec::with_capacity(params.len());
    for (idx, (p, kind, variance)) in pending.iter().enumerate() {
        let data = TypeParameterData::new(owner.clone(), idx as u32, p.name.clone(), *kind)
            .with_variance(*variance)
            .with_source(crate::diagnostic::SemanticSourceSpan::new(current_module.clone(), p.range));
        let param_id = store.intern_type_parameter(data);
        param_ids.push(param_id);
    }

    let mut param_map = std::collections::HashMap::new();
    for (p, &param_id) in params.iter().zip(param_ids.iter()) {
        let binding = type_level_binding_for_parameter(store, param_id);
        param_map.insert(p.name.clone(), binding);
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
                    let l_res = resolve_type_form(store, declarations, &scoped_resolver, site, lower, diagnostics);
                    let u_res = resolve_type_form(store, declarations, &scoped_resolver, site, upper, diagnostics);
                    let l_ty = scoped_ready_or_propagate!(l_res);
                    if store.kind_of(l_ty) != KindId::TYPE {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            current_module.clone(),
                            DiagnosticCode::KindExpectedType,
                            "generic constraint operand must be a proper type",
                            lower.range,
                        ));
                        return TypeFormationOutcome::Invalid(TypeFormationInvalid::GenericConstraintOperandNotType);
                    }
                    let u_ty = scoped_ready_or_propagate!(u_res);
                    if store.kind_of(u_ty) != KindId::TYPE {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            current_module.clone(),
                            DiagnosticCode::KindExpectedType,
                            "generic constraint operand must be a proper type",
                            upper.range,
                        ));
                        return TypeFormationOutcome::Invalid(TypeFormationInvalid::GenericConstraintOperandNotType);
                    }
                    constraints.push(GenericConstraint::Subtype {
                        lower: TypeTerm::Canonical(l_ty),
                        upper: TypeTerm::Canonical(u_ty),
                    });
                }
                GenericConstraintSyntax::Equivalent { left, right, range: _ } => {
                    let l_res = resolve_type_form(store, declarations, &scoped_resolver, site, left, diagnostics);
                    let r_res = resolve_type_form(store, declarations, &scoped_resolver, site, right, diagnostics);
                    let l_ty = scoped_ready_or_propagate!(l_res);
                    if store.kind_of(l_ty) != KindId::TYPE {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            current_module.clone(),
                            DiagnosticCode::KindExpectedType,
                            "generic constraint operand must be a proper type",
                            left.range,
                        ));
                        return TypeFormationOutcome::Invalid(TypeFormationInvalid::GenericConstraintOperandNotType);
                    }
                    let r_ty = scoped_ready_or_propagate!(r_res);
                    if store.kind_of(r_ty) != KindId::TYPE {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            current_module.clone(),
                            DiagnosticCode::KindExpectedType,
                            "generic constraint operand must be a proper type",
                            right.range,
                        ));
                        return TypeFormationOutcome::Invalid(TypeFormationInvalid::GenericConstraintOperandNotType);
                    }
                    constraints.push(GenericConstraint::Equivalent {
                        left: TypeTerm::Canonical(l_ty),
                        right: TypeTerm::Canonical(r_ty),
                    });
                }
                GenericConstraintSyntax::Invalid { message, range } => {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        current_module.clone(),
                        DiagnosticCode::AnnotationUnresolved,
                        message.clone(),
                        *range,
                    ));
                    return TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax);
                }
            }
        }
    }

    let constraint_shapes = constraints
        .iter()
        .map(|constraint| generic_constraint_shape(store, constraint))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let signature = GenericSignature::with_constraints(owner, param_ids.into_boxed_slice(), constraints.into_boxed_slice())
        .with_parameter_metadata(
            pending.iter().map(|(_, kind, _)| *kind).collect::<Vec<_>>().into_boxed_slice(),
            pending.iter().map(|(_, _, variance)| *variance).collect::<Vec<_>>().into_boxed_slice(),
        )
        .with_constraint_shapes(constraint_shapes);
    let signature = signature.with_parameter_kind_shapes(
        pending
            .iter()
            .map(|(_, kind, _)| store.format_kind(*kind).into_boxed_str())
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    if let Err(error) = signature.validate_publishable(store) {
        diagnostics.push(SemanticDiagnostic::error_in(
            current_module.clone(),
            DiagnosticCode::AnnotationUnresolved,
            format!("generic signature is not publishable: {error:?}"),
            params
                .first()
                .map_or(phalcom_common::range::SourceRange::default(), |parameter| parameter.range),
        ));
        return TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax);
    }
    TypeFormationOutcome::Ready(signature)
}

/// Resolves an AST [`TypeAnnotation`] into semantic [`TypeKnowledge`] representing a proper value type.
pub fn resolve_type_annotation(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    annotation: &TypeAnnotation,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeKnowledge {
    let current_module = &site.module;
    let form_res = resolve_type_form(store, declarations, resolver, site, annotation, diagnostics);
    match form_res {
        TypeFormResolution::Ready(ty) => {
            if store.kind_of(ty) != KindId::TYPE {
                diagnostics.push(SemanticDiagnostic::error_in(
                    current_module.clone(),
                    DiagnosticCode::AnnotationUnsaturatedConstructor,
                    "type constructor requires type arguments and cannot be used directly as a value type",
                    annotation.range,
                ));
                TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause)
            } else {
                TypeKnowledge::assumed(ty, EvidenceOrigin::DeveloperAnnotation).with_range(annotation.range)
            }
        }
        TypeFormResolution::Dynamic => TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape),
        TypeFormResolution::Unresolved(TypeFormationUnresolved::Name(name)) => TypeKnowledge::Unknown(UnknownReason::UnresolvedName(name)),
        TypeFormResolution::Unresolved(TypeFormationUnresolved::SelfOutsideOwner) => TypeKnowledge::Unknown(UnknownReason::UnresolvedName("Self".into())),
        TypeFormResolution::Invalid(_) | TypeFormResolution::Missing(_) => TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause),
        TypeFormResolution::Blocked(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
        TypeFormResolution::Cancelled => TypeKnowledge::Unknown(UnknownReason::InferenceCancelled),
        TypeFormResolution::BudgetExceeded(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBudgetExceeded),
        TypeFormResolution::InternalFailure(_) => TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause),
    }
}
