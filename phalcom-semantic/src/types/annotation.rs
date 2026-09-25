//! Source type annotation resolution.

use super::application::TypeApplicationError;
use super::evidence::{DynamicReason, EvidenceOrigin, TypeKnowledge, UnknownReason};
use super::id::{KindId, TypeId, TypeParameterId};
use super::kind::{KindApplicationError, KindData};
use super::outcome::{BlockReason, BudgetReport};
use super::parameter::{
    GenericBinderHeader, GenericConstraint, GenericParameterMode, GenericSignature, SelfRole, SelfTypeTerm, TypeParameterData, TypeParameterOwner, TypeTerm,
};
use super::row::RecordRowTail;
use super::store::{CallableParameterType, CallableType, RecordTypeField, StaticArgument, TypeData, TypeStore};
use super::type_lambda::{
    ScopedCallableParameter, ScopedCallableType, ScopedLabeledTypePackElement, ScopedOpenRecord, ScopedPackProjectionLane, ScopedRecordField, ScopedRecordTail,
    ScopedTupleElement, ScopedTypeData, TypeLambdaProvenance,
};
use super::type_member::{
    IntrinsicTypeOperation, ProjectionSubject, TypeLevelProjection, TypeMemberArgument, TypeMemberSelector, lookup_intrinsic_type_operation,
};
use super::variance::Variance;
use crate::declarations::{DeclarationSemanticForm, DeclarationTypeTable, IntrinsicTypeApplication};
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::identity::{DeclarationId, DispatchSide, ModuleId};
use crate::traits::{AssociatedTypeRequirementId, TraitRef};
use phalcom_ast::ast::{
    ExpansionMode, GenericConstraintSyntax, GenericParameterSyntax, KindSyntax, TypeAnnotation, TypeAnnotationExpr, VarianceSyntax, WhereClauseSyntax,
};
use std::collections::BTreeMap;

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
    pub self_type_override: Option<TypeId>,
    pub associated_type_context: Option<AssociatedTypeFormationContext>,
    pub generic_evidence: Option<crate::generic_evidence::GenericEvidenceEnvironment>,
}

/// Trait-owned context used to form symbolic associated-type projections.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssociatedTypeFormationContext {
    pub trait_ref: TraitRef,
    pub associated_types: BTreeMap<String, AssociatedTypeRequirementId>,
}

impl TypeFormationSite {
    pub fn module(module: ModuleId) -> Self {
        Self {
            module,
            self_term: None,
            self_type_override: None,
            associated_type_context: None,
            generic_evidence: None,
        }
    }

    pub fn member(module: ModuleId, owner: DeclarationId, side: DispatchSide) -> Self {
        Self {
            module,
            self_term: Some(SelfTypeTerm {
                owner,
                side,
                role: SelfRole::InstanceType,
            }),
            self_type_override: None,
            associated_type_context: None,
            generic_evidence: None,
        }
    }

    pub fn trait_member(
        module: ModuleId,
        owner: DeclarationId,
        side: DispatchSide,
        trait_ref: TraitRef,
        associated_types: BTreeMap<String, AssociatedTypeRequirementId>,
    ) -> Self {
        Self {
            module,
            self_term: Some(SelfTypeTerm {
                owner,
                side,
                role: SelfRole::InstanceType,
            }),
            self_type_override: None,
            associated_type_context: Some(AssociatedTypeFormationContext { trait_ref, associated_types }),
            generic_evidence: None,
        }
    }

    pub fn with_self_type_override(mut self, self_type: TypeId) -> Self {
        self.self_type_override = Some(self_type);
        self
    }

    pub fn with_generic_evidence(mut self, generic_evidence: crate::generic_evidence::GenericEvidenceEnvironment) -> Self {
        self.generic_evidence = Some(generic_evidence);
        self
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

    /// Resolve a trait name at a conformance-bound formation site.
    ///
    /// Implementations that track semantic dependencies may distinguish the
    /// trait-header contract from an ordinary nominal declaration shell.
    fn resolve_trait_name(&self, current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId> {
        self.resolve_type_name(current_module, root, members)
    }

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

    /// Resolve an imported module alias in the current module context.
    fn resolve_module_alias(&self, _current_module: &ModuleId, _alias: &str) -> Option<ModuleId> {
        None
    }

    /// Resolve an associated type requirement by trait declaration and member name.
    fn resolve_associated_type_requirement(&self, _trait_declaration: &DeclarationId, _name: &str) -> Option<AssociatedTypeRequirementId> {
        None
    }

    /// Selects a source type-member only through the published associated
    /// surface, then returns its checked declaration product by stable identity.
    fn resolve_associated_type_member(
        &self,
        _owner: &DeclarationId,
        _selector: &TypeMemberSelector,
    ) -> Option<std::sync::Arc<crate::db::product::TypeMemberDeclarationProduct>> {
        None
    }

    /// Retrieves the declaration-owned case set for an already resolved static domain.
    fn resolve_static_case_catalog(
        &self,
        _domain: &crate::identity::StaticDomainDeclarationId,
    ) -> Option<std::sync::Arc<crate::static_case::StaticDomainCaseCatalogProduct>> {
        None
    }

    /// Retrieves the checked constructor contract for one selected static case.
    fn resolve_static_case_signature(&self, _case: &crate::identity::StaticCaseId) -> Option<std::sync::Arc<crate::static_case::StaticCaseSignatureProduct>> {
        None
    }

    /// Selects an associated static domain only through the owner's namespace surface.
    fn resolve_associated_static_domain(
        &self,
        _owner: &DeclarationId,
        _name: &str,
    ) -> Option<crate::identity::AssociatedStaticDomainDeclarationId> {
        None
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

    fn resolve_trait_name(&self, current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId> {
        self.parent.resolve_trait_name(current_module, root, members)
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

    fn resolve_module_alias(&self, current_module: &ModuleId, alias: &str) -> Option<ModuleId> {
        self.parent.resolve_module_alias(current_module, alias)
    }

    fn resolve_associated_type_requirement(&self, trait_declaration: &DeclarationId, name: &str) -> Option<AssociatedTypeRequirementId> {
        self.parent.resolve_associated_type_requirement(trait_declaration, name)
    }

    fn resolve_associated_type_member(
        &self,
        owner: &DeclarationId,
        selector: &TypeMemberSelector,
    ) -> Option<std::sync::Arc<crate::db::product::TypeMemberDeclarationProduct>> {
        self.parent.resolve_associated_type_member(owner, selector)
    }

    fn resolve_static_case_catalog(
        &self,
        domain: &crate::identity::StaticDomainDeclarationId,
    ) -> Option<std::sync::Arc<crate::static_case::StaticDomainCaseCatalogProduct>> {
        self.parent.resolve_static_case_catalog(domain)
    }

    fn resolve_static_case_signature(&self, case: &crate::identity::StaticCaseId) -> Option<std::sync::Arc<crate::static_case::StaticCaseSignatureProduct>> {
        self.parent.resolve_static_case_signature(case)
    }

    fn resolve_associated_static_domain(
        &self,
        owner: &DeclarationId,
        name: &str,
    ) -> Option<crate::identity::AssociatedStaticDomainDeclarationId> {
        self.parent.resolve_associated_static_domain(owner, name)
    }
}

/// A standard resolver holding local declarations, imported declarations, and builtins.
#[derive(Clone, Debug, Default)]
pub struct SimpleTypeResolver {
    pub declarations: std::collections::HashMap<String, DeclarationId>,
    pub type_parameters: std::collections::HashMap<String, TypeLevelBinding>,
    pub associated_types: std::collections::HashMap<(DeclarationId, String), AssociatedTypeRequirementId>,
    pub associated_surfaces: std::collections::HashMap<DeclarationId, std::sync::Arc<crate::associated::AssociatedSurface>>,
    pub type_member_declarations:
        std::collections::HashMap<super::type_member::TypeMemberDeclarationId, std::sync::Arc<crate::db::product::TypeMemberDeclarationProduct>>,
}

impl SimpleTypeResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, decl: DeclarationId) {
        self.declarations.insert(name.into(), decl);
    }

    pub fn insert_associated_type(&mut self, trait_decl: DeclarationId, name: impl Into<String>, req: AssociatedTypeRequirementId) {
        self.associated_types.insert((trait_decl, name.into()), req);
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

    fn resolve_associated_type_requirement(&self, trait_declaration: &DeclarationId, name: &str) -> Option<AssociatedTypeRequirementId> {
        self.associated_types.get(&(trait_declaration.clone(), name.to_string())).cloned()
    }

    fn resolve_associated_type_member(
        &self,
        owner: &DeclarationId,
        selector: &TypeMemberSelector,
    ) -> Option<std::sync::Arc<crate::db::product::TypeMemberDeclarationProduct>> {
        let base = match selector {
            TypeMemberSelector::Named(name) => phalcom_common::selector::SelectorBase::Named(name.to_string()),
            TypeMemberSelector::Subscript => phalcom_common::selector::SelectorBase::Subscript,
        };
        let family = self.associated_surfaces.get(owner)?.families.get(&base)?;
        let crate::associated::AssociatedMemberId::TypeMember(id) = family.members.first()? else { return None };
        self.type_member_declarations.get(id).cloned()
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
    ExpectedStaticDomain { actual: KindId },
    NotAConstructor,
    TooManyTypeArguments,
    TypeArgumentKindMismatch,
    MalformedTypeLambda,
    DuplicateRecordField(Box<str>),
    GenericConstraintOperandNotType,
    InvalidVariance,
    RecordRowTailKindMismatch { actual: KindId },
    UnsupportedRecordRowBinderSite,
    UnsupportedTraitStaticDomainBinder,
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
        KindSyntax::RecordRow(_) | KindSyntax::RecordFields(_) => KindResolution::Ready(KindId::RECORD_ROW),
        KindSyntax::TypePack(_) => KindResolution::Ready(KindId::TYPE_PACK),
        KindSyntax::LabeledTypePack(_) => KindResolution::Ready(KindId::LABELED_TYPE_PACK),
        KindSyntax::CompleteTypePack(_) => KindResolution::Ready(KindId::COMPLETE_TYPE_PACK),
        KindSyntax::StaticDomain { .. } => KindResolution::Invalid(TypeFormationInvalid::InvalidKindSyntax),
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

pub(crate) fn resolve_contextual_kind_syntax(
    store: &mut TypeStore,
    declarations: Option<&DeclarationTypeTable>,
    resolver: Option<&dyn TypeResolver>,
    site: &TypeFormationSite,
    kind: &KindSyntax,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> KindResolution {
    match kind {
        KindSyntax::StaticDomain { form, .. } => {
            let (Some(declarations), Some(resolver)) = (declarations, resolver) else {
                return KindResolution::Invalid(TypeFormationInvalid::InvalidKindSyntax);
            };
            if let TypeAnnotationExpr::Reference(symbol) = &form.expr {
                let members = symbol.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
                if let Some(declaration) = resolver.resolve_type_name(&site.module, &symbol.root, &members)
                    && let Some(kind) = declarations.intrinsic_static_domain_kind(&declaration)
                {
                    return KindResolution::Ready(kind);
                }
            }
            let ty = match resolve_type_form(store, declarations, resolver, site, form, diagnostics) {
                TypeFormResolution::Ready(ty) => ty,
                TypeFormResolution::Dynamic => return KindResolution::Dynamic,
                TypeFormResolution::Missing(reason) => return KindResolution::Missing(reason),
                TypeFormResolution::Unresolved(reason) => return KindResolution::Unresolved(reason),
                TypeFormResolution::Invalid(reason) => return KindResolution::Invalid(reason),
                TypeFormResolution::Blocked(reason) => return KindResolution::Blocked(reason),
                TypeFormResolution::Cancelled => return KindResolution::Cancelled,
                TypeFormResolution::BudgetExceeded(report) => return KindResolution::BudgetExceeded(report),
                TypeFormResolution::InternalFailure(failure) => return KindResolution::InternalFailure(failure),
            };
            let kind = store.kind_of(ty);
            if matches!(store.get_kind(kind), KindData::StaticDomain(_)) {
                KindResolution::Ready(kind)
            } else {
                KindResolution::Invalid(TypeFormationInvalid::ExpectedStaticDomain { actual: kind })
            }
        }
        KindSyntax::Grouped { inner, .. } => resolve_contextual_kind_syntax(store, declarations, resolver, site, inner, diagnostics),
        KindSyntax::Arrow { parameter, result, .. } => resolve_contextual_kind_syntax(store, declarations, resolver, site, parameter, diagnostics).and_then(|parameter| {
            resolve_contextual_kind_syntax(store, declarations, resolver, site, result, diagnostics)
                .map_ready(|result| store.arrow_kind(Box::new([parameter]), result))
        }),
        _ => resolve_kind_syntax(store, kind),
    }
}

/// Classifies a resolved source form in the canonical type-level domain model.
///
/// `Int` is the source spelling for the compile-time integer domain; all other forms retain
/// their interned kind. This is shared by generic/static declarations and type-member
/// signatures so they cannot invent separate domain identities.
pub(crate) fn static_domain_kind_of_form(store: &mut TypeStore, form: TypeId) -> KindId {
    if let TypeData::Nominal { declaration } = store.get(form) {
        if declaration == &crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Int) {
            let domain = store.const_int_domain();
            return store.static_domain_kind(domain);
        }
    }
    store.kind_of(form)
}

#[allow(clippy::too_many_arguments)]
fn resolve_static_case_form(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    enum_target: &TypeAnnotation,
    variant_name: &str,
    generic_arguments: &[TypeAnnotation],
    payload_shape: Option<&phalcom_ast::ast::ExactCasePayloadSyntax>,
    range: phalcom_common::range::SourceRange,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> Option<TypeFormResolution> {
    // The parser deliberately keeps `A::B` ambiguous. Probe the semantic
    // kind without leaking diagnostics into a non-static projection path.
    let mut probe_diagnostics = Vec::new();
    let TypeFormResolution::Ready(owner_form) = resolve_type_form(
        store, declarations, resolver, site, enum_target, &mut probe_diagnostics,
    ) else {
        return None;
    };
    let KindData::StaticDomain(domain) = store.get_kind(store.kind_of(owner_form)) else {
        return None;
    };
    let domain = *domain;
    let identity = match store.static_domain(domain)? {
        crate::types::store::StaticDomainData::Nominal { declaration } => {
            crate::identity::StaticDomainDeclarationId::Nominal(declaration.clone())
        }
        crate::types::store::StaticDomainData::Associated { declaration, .. } => {
            crate::identity::StaticDomainDeclarationId::Associated(declaration.clone())
        }
        crate::types::store::StaticDomainData::ConstInt => return None,
    };
    let Some(catalog) = resolver.resolve_static_case_catalog(&identity) else {
        return Some(TypeFormResolution::Blocked(BlockReason::SuppressedDependency));
    };
    let selected = if payload_shape.is_none() && !generic_arguments.is_empty() {
        catalog.select_constructor(variant_name, generic_arguments.len()).ok_or(())
    } else {
        let selector = phalcom_ast::selector::selector_from_exact_case_target(variant_name, payload_shape);
        crate::static_case::select_static_case(&catalog, &selector).map_err(|_| ())
    };
    let case = match selected {
        Ok(case) => case,
        Err(()) => {
            let selector = phalcom_ast::selector::selector_from_exact_case_target(variant_name, payload_shape);
            let error = crate::static_case::select_static_case(&catalog, &selector).err();
            let message = match error {
                Some(crate::static_case::StaticCaseSelectionError::Missing) | None => format!("static case `{variant_name}` was not declared in this domain"),
                Some(crate::static_case::StaticCaseSelectionError::WrongShape) => {
                    format!("static case `{variant_name}` requires a different constructor shape")
                }
                Some(crate::static_case::StaticCaseSelectionError::Ambiguous) => {
                    format!("static case `{variant_name}` is declared more than once")
                }
            };
            diagnostics.push(SemanticDiagnostic::error_in(site.module.clone(), DiagnosticCode::AnnotationUnresolved, message, range));
            return Some(TypeFormResolution::Invalid(TypeFormationInvalid::Syntax));
        }
    };
    let term = match &case.shape {
        crate::static_case::StaticCaseShape::Singleton => {
            if !generic_arguments.is_empty() || payload_shape.is_some() {
                diagnostics.push(SemanticDiagnostic::error_in(
                    site.module.clone(),
                    DiagnosticCode::AnnotationUnresolved,
                    format!("static singleton `{variant_name}` does not accept constructor arguments"),
                    range,
                ));
                return Some(TypeFormResolution::Invalid(TypeFormationInvalid::Syntax));
            }
            crate::static_case::materialize_static_singleton(store, domain, case)
        }
        crate::static_case::StaticCaseShape::Constructor { .. } => {
            let Some(signature) = resolver.resolve_static_case_signature(&case.id) else {
                return Some(TypeFormResolution::Blocked(BlockReason::SuppressedDependency));
            };
            if signature.parameter_kinds.len() != generic_arguments.len() {
                diagnostics.push(SemanticDiagnostic::error_in(
                    site.module.clone(),
                    DiagnosticCode::AnnotationUnresolved,
                    format!(
                        "static case `{variant_name}` expects {} payload arguments, got {}",
                        signature.parameter_kinds.len(),
                        generic_arguments.len()
                    ),
                    range,
                ));
                return Some(TypeFormResolution::Invalid(TypeFormationInvalid::Syntax));
            }
            let mut arguments = Vec::with_capacity(generic_arguments.len());
            for (index, (annotation, expected)) in generic_arguments.iter().zip(signature.parameter_kinds.iter()).enumerate() {
                let TypeFormResolution::Ready(argument) = resolve_type_form(store, declarations, resolver, site, annotation, diagnostics) else {
                    return Some(TypeFormResolution::Invalid(TypeFormationInvalid::Syntax));
                };
                let static_argument = match store.get(argument) {
                    TypeData::Record(row) => StaticArgument::RecordRow(*row),
                    _ => StaticArgument::Term(argument),
                };
                let actual = match static_argument {
                    StaticArgument::Term(term) => store.kind_of(term),
                    StaticArgument::RecordRow(_) => KindId::RECORD_ROW,
                };
                if actual != *expected {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        site.module.clone(),
                        DiagnosticCode::KindExpectedType,
                        format!(
                            "payload argument {} for static case `{variant_name}` has kind {}, expected {}",
                            index + 1,
                            store.format_kind(actual),
                            store.format_kind(*expected)
                        ),
                        annotation.range,
                    ));
                    return Some(TypeFormResolution::Invalid(TypeFormationInvalid::TypeArgumentKindMismatch));
                }
                arguments.push(static_argument);
            }
            crate::static_case::materialize_static_constructor(store, domain, case, &signature, arguments.into_boxed_slice())
        }
    };
    let Some(term) = term else {
        diagnostics.push(SemanticDiagnostic::error_in(
            site.module.clone(), DiagnosticCode::AnnotationUnresolved,
            format!("could not materialize static case `{variant_name}`"), range,
        ));
        return Some(TypeFormResolution::Invalid(TypeFormationInvalid::Syntax));
    };
    Some(TypeFormResolution::Ready(term))
}

fn resolve_associated_static_domain_form(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    owner_annotation: &TypeAnnotation,
    name: &str,
) -> Option<TypeFormResolution> {
    let mut probe_diagnostics = Vec::new();
    let TypeFormResolution::Ready(owner_form) = resolve_type_form(
        store, declarations, resolver, site, owner_annotation, &mut probe_diagnostics,
    ) else {
        return None;
    };
    let (owner, arguments) = store.applied_nominal_static_parts(owner_form)?;
    let declaration = resolver.resolve_associated_static_domain(&owner, name)?;
    let domain = store.associated_static_domain(declaration, arguments.into_boxed_slice());
    Some(TypeFormResolution::Ready(store.associated_static_domain_form(domain)))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenericBinderSite {
    NominalDeclaration,
    Callable,
    TypeAlias,
    Impl,
    Trait,
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
        ScopedTypeData::Union(_)
        | ScopedTypeData::Tuple(_)
        | ScopedTypeData::Record(_)
        | ScopedTypeData::OpenRecord(_)
        | ScopedTypeData::Callable(_)
        | ScopedTypeData::TupleFromPacks { .. }
        | ScopedTypeData::CallableFromPacks { .. } => KindResolution::Ready(KindId::TYPE),
        ScopedTypeData::TypePack(_) | ScopedTypeData::TypePackConcat(_) => KindResolution::Ready(KindId::TYPE_PACK),
        ScopedTypeData::LabeledTypePack(_) | ScopedTypeData::LabeledTypePackConcat(_) => KindResolution::Ready(KindId::LABELED_TYPE_PACK),
        ScopedTypeData::CompleteTypePack { .. } => KindResolution::Ready(KindId::COMPLETE_TYPE_PACK),
        ScopedTypeData::PackProjection { lane, .. } => KindResolution::Ready(match lane {
            ScopedPackProjectionLane::Positionals => KindId::TYPE_PACK,
            ScopedPackProjectionLane::Labeled => KindId::LABELED_TYPE_PACK,
            ScopedPackProjectionLane::Complete => KindId::COMPLETE_TYPE_PACK,
        }),
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

fn scoped_pack_projection(
    store: &mut TypeStore,
    subject: crate::types::id::ScopedTypeId,
    binders: &ScopedBinderStack,
    lane: ScopedPackProjectionLane,
) -> TypeFormationOutcome<crate::types::id::ScopedTypeId> {
    let kind = scoped_ready_or_propagate!(scoped_kind(store, subject, binders));
    let identity = matches!(
        (kind, lane),
        (KindId::TYPE_PACK, ScopedPackProjectionLane::Positionals)
            | (KindId::LABELED_TYPE_PACK, ScopedPackProjectionLane::Labeled)
            | (KindId::COMPLETE_TYPE_PACK, ScopedPackProjectionLane::Complete)
    );
    if identity {
        return TypeFormationOutcome::Ready(subject);
    }
    if kind == KindId::COMPLETE_TYPE_PACK {
        return TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::PackProjection { subject, lane }));
    }
    TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax)
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
        TypeAnnotationExpr::AssociatedTypeProjection {
            subject,
            name,
            name_range,
            range,
        } => resolve_associated_type_projection(store, declarations, resolver, site, subject, name, *name_range, *range, diagnostics)
            .map_ready(|ty| intern_scoped_free(store, ty)),
        TypeAnnotationExpr::QualifiedAssociatedTypeProjection {
            subject,
            trait_ref,
            name,
            name_range,
            range,
        } => resolve_qualified_associated_projection(store, declarations, resolver, site, subject, trait_ref, name, *name_range, *range, diagnostics)
            .map_ready(|ty| intern_scoped_free(store, ty)),
        TypeAnnotationExpr::TypeMemberProjection {
            subject,
            selector,
            arguments,
            range,
        } => resolve_type_member_projection(store, declarations, resolver, site, subject, selector, arguments, *range, diagnostics)
            .map_ready(|ty| intern_scoped_free(store, ty)),
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
                    // `Some` and `None` are the runtime behavior classes of
                    // the canonical Option variants. They are not ordinary
                    // prelude value bindings, but remain valid nominal type
                    // forms for sealed-inheritance and class annotations.
                    "Some" | "None" => {
                        let key = if name == "Some" {
                            phalcom_native_meta::UniverseKey::Some
                        } else {
                            phalcom_native_meta::UniverseKey::None
                        };
                        let declaration = crate::core_surface::universe_declaration(key);
                        if let Some(form) = declarations.form(&declaration) {
                            return TypeFormationOutcome::Ready(intern_scoped_free(store, form));
                        }
                    }
                    "Dynamic" => return TypeFormationOutcome::Dynamic,
                    _ => {}
                }
            }

            let members: Vec<String> = sym_ref.members.iter().map(|member| member.name.clone()).collect();
            if let Some(declaration) = resolver.resolve_type_name(current_module, &sym_ref.root, &members) {
                let runtime_form = declarations
                    .intrinsic_runtime_owner(&declaration)
                    .and_then(|_| declarations.runtime_form(store, &declaration));
                if let Some(form) = runtime_form.or_else(|| declarations.form(&declaration)).or_else(|| resolver.resolve_alias_form(&declaration)) {
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
            let intrinsic_constructor = if let TypeAnnotationExpr::Reference(reference) = &origin.expr {
                let members = reference.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
                resolver
                    .resolve_type_name(current_module, &reference.root, &members)
                    .filter(|declaration| matches!(declarations.semantic_form(declaration), Some(DeclarationSemanticForm::Intrinsic(_))))
                    .and_then(|declaration| declarations.form(&declaration))
            } else {
                None
            };
            let scoped_origin = if let Some(form) = intrinsic_constructor {
                intern_scoped_free(store, form)
            } else {
                scoped_ready_or_propagate!(lower_scoped_type_form(store, declarations, resolver, site, binders, origin, diagnostics))
            };
            if intrinsic_type_application_for_scoped_form(store, declarations, scoped_origin) == Some(IntrinsicTypeApplication::Record)
                && arguments.len() == 1
            {
                if let phalcom_ast::ast::GenericArgumentSyntax::Single(argument) = &arguments[0] {
                    if let TypeAnnotationExpr::Reference(reference) = &argument.expr
                        && reference.members.is_empty()
                    {
                        if let Some((depth, index, KindId::RECORD_ROW)) = binders.resolve(reference.leaf_name()) {
                            return TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::OpenRecord(ScopedOpenRecord {
                                fields: Box::new([]),
                                tail: ScopedRecordTail::Bound { depth, index },
                            })));
                        }
                        if let Some(TypeLevelBinding::RecordRow(parameter)) = resolver.resolve_type_level_binding(reference.leaf_name()) {
                            return TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::OpenRecord(ScopedOpenRecord {
                                fields: Box::new([]),
                                tail: ScopedRecordTail::FreeParameter(parameter),
                            })));
                        }
                    }
                    let lowered = scoped_ready_or_propagate!(lower_scoped_type_form(
                        store, declarations, resolver, site, binders, argument, diagnostics,
                    ));
                    if matches!(store.arena().get_scoped(lowered), ScopedTypeData::Free(ty) if matches!(store.get(*ty), TypeData::Record(_))) {
                        return TypeFormationOutcome::Ready(lowered);
                    }
                    return TypeFormationOutcome::Invalid(TypeFormationInvalid::TypeArgumentKindMismatch);
                }
                return TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax);
            }
            if let Some(contract) = scoped_generic_application_contract(store, declarations, scoped_origin)
                && contract.has_variadic_binder()
            {
                debug_assert_eq!(contract.parameter_kinds.len(), contract.parameter_modes.len());
                let fixed_count = contract
                    .parameter_modes
                    .iter()
                    .take_while(|mode| **mode == GenericParameterMode::Single)
                    .count();
                let mut lowered = Vec::with_capacity(contract.parameter_modes.len());
                for argument in arguments.iter().take(fixed_count) {
                    let phalcom_ast::ast::GenericArgumentSyntax::Single(argument) = argument else {
                        return TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax);
                    };
                    lowered.push(scoped_ready_or_propagate!(lower_scoped_type_form(
                        store,
                        declarations,
                        resolver,
                        site,
                        binders,
                        argument,
                        diagnostics,
                    )));
                }
                if arguments.len() < fixed_count {
                    return TypeFormationOutcome::Invalid(TypeFormationInvalid::TooManyTypeArguments);
                }
                let mut positional_segments = Vec::new();
                let mut labeled_segments = Vec::new();
                for argument in arguments.iter().skip(fixed_count) {
                    match argument {
                        phalcom_ast::ast::GenericArgumentSyntax::Single(argument) => {
                            let range = argument.range;
                            let argument =
                                scoped_ready_or_propagate!(lower_scoped_type_form(store, declarations, resolver, site, binders, argument, diagnostics,));
                            scoped_ready_or_propagate!(require_scoped_proper(store, argument, binders, current_module, range, diagnostics,));
                            positional_segments.push(store.arena_mut().intern_scoped(ScopedTypeData::TypePack(vec![argument].into_boxed_slice())));
                        }
                        phalcom_ast::ast::GenericArgumentSyntax::Expansion(expansion) => {
                            let operand = scoped_ready_or_propagate!(lower_scoped_type_form(
                                store,
                                declarations,
                                resolver,
                                site,
                                binders,
                                &expansion.operand,
                                diagnostics,
                            ));
                            match expansion.mode {
                                ExpansionMode::Positional => positional_segments.push(scoped_ready_or_propagate!(scoped_pack_projection(
                                    store,
                                    operand,
                                    binders,
                                    ScopedPackProjectionLane::Positionals,
                                ))),
                                ExpansionMode::Labeled => labeled_segments.push(scoped_ready_or_propagate!(scoped_pack_projection(
                                    store,
                                    operand,
                                    binders,
                                    ScopedPackProjectionLane::Labeled,
                                ))),
                                ExpansionMode::Complete => {
                                    positional_segments.push(scoped_ready_or_propagate!(scoped_pack_projection(
                                        store,
                                        operand,
                                        binders,
                                        ScopedPackProjectionLane::Positionals,
                                    )));
                                    labeled_segments.push(scoped_ready_or_propagate!(scoped_pack_projection(
                                        store,
                                        operand,
                                        binders,
                                        ScopedPackProjectionLane::Labeled,
                                    )));
                                }
                            }
                        }
                    }
                }
                let positionals = store
                    .arena_mut()
                    .intern_scoped(ScopedTypeData::TypePackConcat(positional_segments.into_boxed_slice()));
                let labeled = store
                    .arena_mut()
                    .intern_scoped(ScopedTypeData::LabeledTypePackConcat(labeled_segments.into_boxed_slice()));
                for mode in contract.parameter_modes.iter().skip(fixed_count) {
                    lowered.push(match mode {
                        GenericParameterMode::PositionalPack => positionals,
                        GenericParameterMode::LabeledPack => labeled,
                        GenericParameterMode::CompletePack => store.arena_mut().intern_scoped(ScopedTypeData::CompleteTypePack { positionals, labeled }),
                        GenericParameterMode::Single => return TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax),
                    });
                }
                if intrinsic_type_application_for_scoped_form(store, declarations, scoped_origin) == Some(IntrinsicTypeApplication::Tuple)
                    && lowered.len() == 1
                {
                    if let ScopedTypeData::CompleteTypePack { positionals, labeled } = store.arena().get_scoped(lowered[0]).clone() {
                        return TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::TupleFromPacks { positionals, labeled }));
                    }
                    return TypeFormationOutcome::Invalid(TypeFormationInvalid::TypeArgumentKindMismatch);
                }
                let origin_kind = scoped_ready_or_propagate!(scoped_kind(store, scoped_origin, binders));
                let mut argument_kinds = Vec::with_capacity(lowered.len());
                for &argument in &lowered {
                    argument_kinds.push(scoped_ready_or_propagate!(scoped_kind(store, argument, binders)));
                }
                return match store.apply_kind(origin_kind, &argument_kinds) {
                    Ok(_) => TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::Applied {
                        origin: scoped_origin,
                        arguments: lowered.into_boxed_slice(),
                    })),
                    Err(KindApplicationError::NotApplicable { .. }) => TypeFormationOutcome::Invalid(TypeFormationInvalid::NotAConstructor),
                    Err(KindApplicationError::TooManyArguments { .. }) => TypeFormationOutcome::Invalid(TypeFormationInvalid::TooManyTypeArguments),
                    Err(KindApplicationError::ArgumentKindMismatch { .. }) => TypeFormationOutcome::Invalid(TypeFormationInvalid::TypeArgumentKindMismatch),
                };
            }
            let mut scoped_arguments = Vec::with_capacity(arguments.len());
            for argument in arguments {
                let phalcom_ast::ast::GenericArgumentSyntax::Single(argument) = argument else {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        current_module.clone(),
                        DiagnosticCode::AnnotationUnsupported,
                        "type-pack expansion is not available in this scoped type form",
                        argument.range(),
                    ));
                    return TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax);
                };
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
            let mut positional_segments = Vec::new();
            let mut labeled_segments = Vec::new();
            let mut fixed_elements = Vec::with_capacity(elements.len());
            let mut has_expansion = false;
            let mut labeled_phase = false;
            for element in elements {
                match element {
                    phalcom_ast::ast::TypeTupleItem::Element(element) => {
                        let ty = scoped_ready_or_propagate!(lower_scoped_type_form(store, declarations, resolver, site, binders, &element.ty, diagnostics,));
                        scoped_ready_or_propagate!(require_scoped_proper(store, ty, binders, current_module, element.range, diagnostics));
                        fixed_elements.push(ScopedTupleElement {
                            label: element.label.clone().map(Into::into),
                            ty,
                        });
                        if let Some(label) = &element.label {
                            labeled_phase = true;
                            labeled_segments.push(
                                store.arena_mut().intern_scoped(ScopedTypeData::LabeledTypePack(
                                    vec![ScopedLabeledTypePackElement {
                                        label: label.clone().into(),
                                        element: ty,
                                    }]
                                    .into_boxed_slice(),
                                )),
                            );
                        } else {
                            if labeled_phase {
                                return TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax);
                            }
                            positional_segments.push(store.arena_mut().intern_scoped(ScopedTypeData::TypePack(vec![ty].into_boxed_slice())));
                        }
                    }
                    phalcom_ast::ast::TypeTupleItem::Expansion(expansion) => {
                        has_expansion = true;
                        if labeled_phase && expansion.mode != ExpansionMode::Labeled {
                            return TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax);
                        }
                        let operand = scoped_ready_or_propagate!(lower_scoped_type_form(
                            store,
                            declarations,
                            resolver,
                            site,
                            binders,
                            &expansion.operand,
                            diagnostics,
                        ));
                        match expansion.mode {
                            ExpansionMode::Positional => positional_segments.push(scoped_ready_or_propagate!(scoped_pack_projection(
                                store,
                                operand,
                                binders,
                                ScopedPackProjectionLane::Positionals,
                            ))),
                            ExpansionMode::Labeled => {
                                labeled_phase = true;
                                labeled_segments.push(scoped_ready_or_propagate!(scoped_pack_projection(
                                    store,
                                    operand,
                                    binders,
                                    ScopedPackProjectionLane::Labeled,
                                )));
                            }
                            ExpansionMode::Complete => {
                                positional_segments.push(scoped_ready_or_propagate!(scoped_pack_projection(
                                    store,
                                    operand,
                                    binders,
                                    ScopedPackProjectionLane::Positionals,
                                )));
                                labeled_segments.push(scoped_ready_or_propagate!(scoped_pack_projection(
                                    store,
                                    operand,
                                    binders,
                                    ScopedPackProjectionLane::Labeled,
                                )));
                            }
                        }
                    }
                }
            }
            if !has_expansion {
                return TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::Tuple(fixed_elements.into_boxed_slice())));
            }
            let positionals = store
                .arena_mut()
                .intern_scoped(ScopedTypeData::TypePackConcat(positional_segments.into_boxed_slice()));
            let labeled = store
                .arena_mut()
                .intern_scoped(ScopedTypeData::LabeledTypePackConcat(labeled_segments.into_boxed_slice()));
            TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::TupleFromPacks { positionals, labeled }))
        }
        TypeAnnotationExpr::Record { fields, tail, .. } => {
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
            let scoped_fields = scoped_fields.into_boxed_slice();
            let Some(tail) = tail else {
                return TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::Record(scoped_fields)));
            };
            let scoped_tail = if let Some((depth, index, kind)) = binders.resolve(&tail.name) {
                if kind != KindId::RECORD_ROW {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        current_module.clone(),
                        DiagnosticCode::KindExpectedType,
                        format!("record row tail `{}` must have kind RecordRow", tail.name),
                        annotation.range,
                    ));
                    return TypeFormationOutcome::Invalid(TypeFormationInvalid::RecordRowTailKindMismatch { actual: kind });
                }
                ScopedRecordTail::Bound { depth, index }
            } else {
                match resolver.resolve_type_level_binding(&tail.name) {
                    Some(TypeLevelBinding::RecordRow(parameter)) => ScopedRecordTail::FreeParameter(parameter),
                    Some(TypeLevelBinding::TypeForm(form)) => {
                        let actual = store.kind_of(form);
                        diagnostics.push(SemanticDiagnostic::error_in(
                            current_module.clone(),
                            DiagnosticCode::RecordRowTailKindMismatch,
                            format!("record row tail `{}` must have kind RecordRow", tail.name),
                            annotation.range,
                        ));
                        return TypeFormationOutcome::Invalid(TypeFormationInvalid::RecordRowTailKindMismatch { actual });
                    }
                    None => {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            current_module.clone(),
                            DiagnosticCode::RecordRowTailUnresolved,
                            format!("unresolved record row tail `{}`", tail.name),
                            annotation.range,
                        ));
                        return TypeFormationOutcome::Unresolved(TypeFormationUnresolved::Name(tail.name.clone().into()));
                    }
                }
            };
            TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::OpenRecord(ScopedOpenRecord {
                fields: scoped_fields,
                tail: scoped_tail,
            })))
        }
        TypeAnnotationExpr::Callable { parameters, result, .. } => {
            let mut positional_segments = Vec::new();
            let mut labeled_segments = Vec::new();
            let mut fixed_parameters = Vec::with_capacity(parameters.len());
            let mut has_expansion = false;
            let mut labeled_phase = false;
            for parameter in parameters {
                match parameter {
                    phalcom_ast::ast::TypeCallableItem::Parameter(param) => {
                        let ty = scoped_ready_or_propagate!(lower_scoped_type_form(store, declarations, resolver, site, binders, &param.ty, diagnostics,));
                        scoped_ready_or_propagate!(require_scoped_proper(store, ty, binders, current_module, param.range, diagnostics));
                        fixed_parameters.push(ScopedCallableParameter {
                            label: param.label.clone().map(Into::into),
                            ty,
                            rest: param.rest,
                        });
                        if let Some(label) = &param.label {
                            labeled_phase = true;
                            labeled_segments.push(
                                store.arena_mut().intern_scoped(ScopedTypeData::LabeledTypePack(
                                    vec![ScopedLabeledTypePackElement {
                                        label: label.clone().into(),
                                        element: ty,
                                    }]
                                    .into_boxed_slice(),
                                )),
                            );
                        } else {
                            if labeled_phase {
                                diagnostics.push(SemanticDiagnostic::error_in(
                                    current_module.clone(),
                                    DiagnosticCode::AnnotationUnsupported,
                                    "positional callable parameter cannot follow labeled parameter",
                                    param.range,
                                ));
                                return TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax);
                            }
                            positional_segments.push(store.arena_mut().intern_scoped(ScopedTypeData::TypePack(vec![ty].into_boxed_slice())));
                        }
                    }
                    phalcom_ast::ast::TypeCallableItem::Expansion(expansion) => {
                        has_expansion = true;
                        if labeled_phase && expansion.mode != ExpansionMode::Labeled {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
                                DiagnosticCode::AnnotationUnsupported,
                                "positional type-pack expansion cannot follow labeled callable parameters",
                                expansion.range,
                            ));
                            return TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax);
                        }
                        let operand = scoped_ready_or_propagate!(lower_scoped_type_form(
                            store,
                            declarations,
                            resolver,
                            site,
                            binders,
                            &expansion.operand,
                            diagnostics,
                        ));
                        match expansion.mode {
                            ExpansionMode::Positional => positional_segments.push(scoped_ready_or_propagate!(scoped_pack_projection(
                                store,
                                operand,
                                binders,
                                ScopedPackProjectionLane::Positionals,
                            ))),
                            ExpansionMode::Labeled => {
                                labeled_phase = true;
                                labeled_segments.push(scoped_ready_or_propagate!(scoped_pack_projection(
                                    store,
                                    operand,
                                    binders,
                                    ScopedPackProjectionLane::Labeled,
                                )));
                            }
                            ExpansionMode::Complete => {
                                positional_segments.push(scoped_ready_or_propagate!(scoped_pack_projection(
                                    store,
                                    operand,
                                    binders,
                                    ScopedPackProjectionLane::Positionals,
                                )));
                                labeled_segments.push(scoped_ready_or_propagate!(scoped_pack_projection(
                                    store,
                                    operand,
                                    binders,
                                    ScopedPackProjectionLane::Labeled,
                                )));
                            }
                        }
                    }
                }
            }
            let scoped_return = scoped_ready_or_propagate!(lower_scoped_type_form(store, declarations, resolver, site, binders, result, diagnostics,));
            scoped_ready_or_propagate!(require_scoped_proper(store, scoped_return, binders, current_module, result.range, diagnostics));
            if !has_expansion {
                return TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::Callable(ScopedCallableType {
                    parameters: fixed_parameters.into_boxed_slice(),
                    return_type: scoped_return,
                })));
            }
            let positionals = store
                .arena_mut()
                .intern_scoped(ScopedTypeData::TypePackConcat(positional_segments.into_boxed_slice()));
            let labeled = store
                .arena_mut()
                .intern_scoped(ScopedTypeData::LabeledTypePackConcat(labeled_segments.into_boxed_slice()));
            TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type: scoped_return,
            }))
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
        TypeAnnotationExpr::ExactEnumCase {
            enum_target,
            variant_name,
            variant_name_range,
            generic_arguments,
            payload_shape,
            range,
        } => {
            if let Some(result) = resolve_static_case_form(
                store, declarations, resolver, site, enum_target, variant_name,
                generic_arguments, payload_shape.as_ref(), *range, diagnostics,
            ) {
                let ty = scoped_ready_or_propagate!(result);
                return TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::Free(ty)));
            }
            if generic_arguments.is_empty() && payload_shape.is_none() {
                if let Some(result) = resolve_associated_static_domain_form(
                    store, declarations, resolver, site, enum_target, variant_name,
                ) {
                    let ty = scoped_ready_or_propagate!(result);
                    return TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::Free(ty)));
                }
            }
            if generic_arguments.is_empty() && payload_shape.is_none() {
                let res = resolve_associated_type_projection(
                    store,
                    declarations,
                    resolver,
                    site,
                    enum_target,
                    variant_name,
                    *variant_name_range,
                    *range,
                    diagnostics,
                );
                let ty = scoped_ready_or_propagate!(res);
                return TypeFormationOutcome::Ready(store.arena_mut().intern_scoped(ScopedTypeData::Free(ty)));
            }
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnresolved,
                "exact enum case syntax is only valid as an inherent impl target",
                *range,
            ));
            TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax)
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
                let lambda_id = store.arena_mut().intern_lambda_with_modes(
                    parameter_kinds.into_boxed_slice(),
                    signature.parameter_modes.clone(),
                    scoped_body,
                    result_kind,
                    Some(provenance),
                );
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

#[derive(Clone, Debug)]
struct GenericApplicationContract {
    parameter_kinds: Box<[KindId]>,
    parameter_modes: Box<[GenericParameterMode]>,
}

impl GenericApplicationContract {
    fn from_binder_header(header: &GenericBinderHeader, consumed: usize) -> Option<Self> {
        (consumed <= header.parameters.len()).then(|| Self {
            parameter_kinds: header.parameter_kinds[consumed..].to_vec().into_boxed_slice(),
            parameter_modes: header.parameter_modes[consumed..].to_vec().into_boxed_slice(),
        })
    }

    fn from_type_lambda(lambda: &super::type_lambda::TypeLambdaData) -> Self {
        Self {
            parameter_kinds: lambda.parameter_kinds.clone(),
            parameter_modes: lambda.parameter_modes.clone(),
        }
    }

    fn has_variadic_binder(&self) -> bool {
        self.parameter_modes.iter().any(|mode| *mode != GenericParameterMode::Single)
    }
}

fn generic_application_contract(store: &TypeStore, declarations: &DeclarationTypeTable, origin: TypeId) -> Option<GenericApplicationContract> {
    match store.get(origin) {
        TypeData::Nominal { declaration } => declarations
            .generic_binder_header(declaration)
            .and_then(|header| GenericApplicationContract::from_binder_header(header, 0)),
        TypeData::Applied { origin, arguments } => {
            let declaration = store.nominal_origin_declaration(*origin)?;
            declarations
                .generic_binder_header(declaration)
                .and_then(|header| GenericApplicationContract::from_binder_header(header, arguments.len()))
        }
        TypeData::Lambda(lambda) => Some(GenericApplicationContract::from_type_lambda(store.arena().get_lambda(*lambda))),
        _ => None,
    }
}

fn intrinsic_type_application_for_form(
    store: &TypeStore,
    declarations: &DeclarationTypeTable,
    origin: TypeId,
) -> Option<IntrinsicTypeApplication> {
    let declaration = store.nominal_origin_declaration(origin)?;
    match declarations.semantic_form(declaration)? {
        DeclarationSemanticForm::Ordinary => None,
        DeclarationSemanticForm::Intrinsic(application) => Some(application),
    }
}

fn scoped_generic_application_contract(
    store: &TypeStore,
    declarations: &DeclarationTypeTable,
    origin: crate::types::id::ScopedTypeId,
) -> Option<GenericApplicationContract> {
    match store.arena().get_scoped(origin) {
        ScopedTypeData::Free(origin) => generic_application_contract(store, declarations, *origin),
        ScopedTypeData::Lambda(lambda) => Some(GenericApplicationContract::from_type_lambda(store.arena().get_lambda(*lambda))),
        _ => None,
    }
}

fn intrinsic_type_application_for_scoped_form(
    store: &TypeStore,
    declarations: &DeclarationTypeTable,
    origin: crate::types::id::ScopedTypeId,
) -> Option<IntrinsicTypeApplication> {
    let ScopedTypeData::Free(origin) = store.arena().get_scoped(origin) else {
        return None;
    };
    intrinsic_type_application_for_form(store, declarations, *origin)
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
        let kind = scoped_ready_or_propagate!(kind);
        if kind == KindId::RECORD_ROW {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnsupported,
                "RecordRow parameters are supported only on callable generic binders",
                parameter.range,
            ));
            return TypeFormationOutcome::Invalid(TypeFormationInvalid::UnsupportedRecordRowBinderSite);
        }
        parameter_kinds.push(kind);
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

fn resolve_associated_type_projection(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    subject: &TypeAnnotation,
    name: &str,
    name_range: phalcom_common::range::SourceRange,
    range: phalcom_common::range::SourceRange,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormResolution {
    let current_module = &site.module;
    let subject_ty = match resolve_type_form(store, declarations, resolver, site, subject, diagnostics) {
        TypeFormResolution::Ready(subject) => subject,
        TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
        TypeFormResolution::Missing(reason) => return TypeFormResolution::Missing(reason),
        TypeFormResolution::Unresolved(reason) => return TypeFormResolution::Unresolved(reason),
        TypeFormResolution::Invalid(reason) => return TypeFormResolution::Invalid(reason),
        TypeFormResolution::Blocked(reason) => return TypeFormResolution::Blocked(reason),
        TypeFormResolution::Cancelled => return TypeFormResolution::Cancelled,
        TypeFormResolution::BudgetExceeded(report) => return TypeFormResolution::BudgetExceeded(report),
        TypeFormResolution::InternalFailure(failure) => return TypeFormResolution::InternalFailure(failure),
    };

    if let Some(context) = site.associated_type_context.as_ref() {
        if matches!(subject.expr, TypeAnnotationExpr::SelfType { .. }) {
            let Some(requirement) = context.associated_types.get(name).cloned() else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    current_module.clone(),
                    DiagnosticCode::AssociatedTypeUnknown,
                    format!("unknown associated type `{name}` in trait `{}`", context.trait_ref.declaration.name),
                    name_range,
                ));
                return TypeFormResolution::Unresolved(TypeFormationUnresolved::Name(name.into()));
            };
            return TypeFormResolution::Ready(store.associated_projection(subject_ty, context.trait_ref.clone(), requirement));
        }
    }

    if let Some(evidence) = site.generic_evidence.as_ref() {
        let subject_term = TypeTerm::Canonical(subject_ty);
        let mut matching_traits = Vec::new();

        for conformance_key in evidence.conformances().keys() {
            if conformance_key.subject == subject_term
                || evidence.are_equivalent(&conformance_key.subject, &subject_term) == crate::generic_evidence::GenericEquivalenceProof::Proven
            {
                if let Some(context) = site.associated_type_context.as_ref() {
                    if context.trait_ref.declaration == conformance_key.trait_ref.declaration {
                        if let Some(req_id) = context.associated_types.get(name).cloned() {
                            matching_traits.push((conformance_key.trait_ref.clone(), req_id));
                            continue;
                        }
                    }
                }
                let req_id = AssociatedTypeRequirementId::new(conformance_key.trait_ref.declaration.clone(), 0);
                matching_traits.push((conformance_key.trait_ref.clone(), req_id));
            }
        }

        matching_traits.dedup_by(|a, b| a.1 == b.1);

        if matching_traits.len() == 1 {
            let (trait_ref, req_id) = matching_traits.into_iter().next().unwrap();
            return TypeFormResolution::Ready(store.associated_projection(subject_ty, trait_ref, req_id));
        } else if matching_traits.len() > 1 {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnresolved,
                format!("ambiguous contextual associated type projection `{name}` for subject"),
                range,
            ));
            return TypeFormResolution::Unresolved(TypeFormationUnresolved::Name(name.into()));
        }
    }

    diagnostics.push(SemanticDiagnostic::error_in(
        current_module.clone(),
        DiagnosticCode::AnnotationUnresolved,
        "contextual associated type projection subject must have conformance evidence",
        range,
    ));
    TypeFormResolution::Unresolved(TypeFormationUnresolved::Name(name.into()))
}

fn resolve_qualified_associated_projection(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    subject: &TypeAnnotation,
    trait_requirement: &phalcom_ast::ast::TraitRequirementSyntax,
    name: &str,
    name_range: phalcom_common::range::SourceRange,
    _range: phalcom_common::range::SourceRange,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormResolution {
    let current_module = &site.module;
    let subject_ty = match resolve_type_form(store, declarations, resolver, site, subject, diagnostics) {
        TypeFormResolution::Ready(s) => s,
        TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
        TypeFormResolution::Missing(reason) => return TypeFormResolution::Missing(reason),
        TypeFormResolution::Unresolved(reason) => return TypeFormResolution::Unresolved(reason),
        TypeFormResolution::Invalid(reason) => return TypeFormResolution::Invalid(reason),
        TypeFormResolution::Blocked(reason) => return TypeFormResolution::Blocked(reason),
        TypeFormResolution::Cancelled => return TypeFormResolution::Cancelled,
        TypeFormResolution::BudgetExceeded(report) => return TypeFormResolution::BudgetExceeded(report),
        TypeFormResolution::InternalFailure(failure) => return TypeFormResolution::InternalFailure(failure),
    };

    let trait_res = resolve_trait_ref_for_conformance(store, declarations, resolver, site, &trait_requirement.trait_type, diagnostics);
    let trait_ref = match trait_res {
        TypeFormationOutcome::Ready(t) => t,
        TypeFormationOutcome::Dynamic => return TypeFormResolution::Dynamic,
        TypeFormationOutcome::Missing(reason) => return TypeFormResolution::Missing(reason),
        TypeFormationOutcome::Unresolved(reason) => return TypeFormResolution::Unresolved(reason),
        TypeFormationOutcome::Invalid(reason) => return TypeFormResolution::Invalid(reason),
        TypeFormationOutcome::Blocked(reason) => return TypeFormResolution::Blocked(reason),
        TypeFormationOutcome::Cancelled => return TypeFormResolution::Cancelled,
        TypeFormationOutcome::BudgetExceeded(report) => return TypeFormResolution::BudgetExceeded(report),
        TypeFormationOutcome::InternalFailure(failure) => return TypeFormResolution::InternalFailure(failure),
    };

    // If trait context is available for this trait declaration, look up the requirement
    if let Some(context) = site.associated_type_context.as_ref() {
        if context.trait_ref.declaration == trait_ref.declaration {
            if let Some(requirement) = context.associated_types.get(name).cloned() {
                return TypeFormResolution::Ready(store.associated_projection(subject_ty, trait_ref, requirement));
            } else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    current_module.clone(),
                    DiagnosticCode::AssociatedTypeUnknown,
                    format!("unknown associated type `{name}` in trait `{}`", trait_ref.declaration.name),
                    name_range,
                ));
                return TypeFormResolution::Unresolved(TypeFormationUnresolved::Name(name.into()));
            }
        }
    }

    if let Some(requirement) = resolver.resolve_associated_type_requirement(&trait_ref.declaration, name) {
        return TypeFormResolution::Ready(store.associated_projection(subject_ty, trait_ref, requirement));
    }

    let requirement = AssociatedTypeRequirementId::new(trait_ref.declaration.clone(), 0);
    TypeFormResolution::Ready(store.associated_projection(subject_ty, trait_ref, requirement))
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
    macro_rules! type_form_ready_or_return {
        ($result:expr) => {
            match $result {
                TypeFormResolution::Ready(ty) => ty,
                TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
                TypeFormResolution::Unresolved(reason) => return TypeFormResolution::Unresolved(reason),
                TypeFormResolution::Missing(reason) => return TypeFormResolution::Missing(reason),
                TypeFormResolution::Invalid(reason) => return TypeFormResolution::Invalid(reason),
                TypeFormResolution::Blocked(reason) => return TypeFormResolution::Blocked(reason),
                TypeFormResolution::Cancelled => return TypeFormResolution::Cancelled,
                TypeFormResolution::BudgetExceeded(report) => return TypeFormResolution::BudgetExceeded(report),
                TypeFormResolution::InternalFailure(failure) => return TypeFormResolution::InternalFailure(failure),
            }
        };
    }
    match &annotation.expr {
        TypeAnnotationExpr::Unit { .. } => TypeFormResolution::Ready(store.unit()),
        TypeAnnotationExpr::Dynamic { .. } => TypeFormResolution::Dynamic,
        TypeAnnotationExpr::Never { .. } => TypeFormResolution::Ready(store.never()),
        TypeAnnotationExpr::SelfType { range } => {
            if let Some(self_type) = site.self_type_override {
                TypeFormResolution::Ready(self_type)
            } else if let Some(term) = site.self_term.clone() {
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
        TypeAnnotationExpr::AssociatedTypeProjection {
            subject,
            name,
            name_range,
            range,
        } => resolve_associated_type_projection(store, declarations, resolver, site, subject, name, *name_range, *range, diagnostics),
        TypeAnnotationExpr::QualifiedAssociatedTypeProjection {
            subject,
            trait_ref,
            name,
            name_range,
            range,
        } => resolve_qualified_associated_projection(store, declarations, resolver, site, subject, trait_ref, name, *name_range, *range, diagnostics),
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
                    "Some" | "None" => {
                        let key = if name == "Some" {
                            phalcom_native_meta::UniverseKey::Some
                        } else {
                            phalcom_native_meta::UniverseKey::None
                        };
                        let declaration = crate::core_surface::universe_declaration(key);
                        if let Some(form) = declarations.form(&declaration) {
                            return TypeFormResolution::Ready(form);
                        }
                    }
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
            let origin_res = if let TypeAnnotationExpr::Reference(reference) = &origin.expr {
                let members = reference.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
                resolver
                    .resolve_type_name(current_module, &reference.root, &members)
                    .filter(|declaration| matches!(declarations.semantic_form(declaration), Some(DeclarationSemanticForm::Intrinsic(_))))
                    .and_then(|declaration| declarations.form(&declaration))
                    .map(TypeFormResolution::Ready)
                    .unwrap_or_else(|| resolve_type_form(store, declarations, resolver, site, origin, diagnostics))
            } else {
                resolve_type_form(store, declarations, resolver, site, origin, diagnostics)
            };
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

            // RecordRow is a separate semantic identity, not a TypeId generic
            // argument. Consume a bound row directly and form the existing
            // structural Record value at this declaration-owned boundary.
            if intrinsic_type_application_for_form(store, declarations, origin_ty) == Some(IntrinsicTypeApplication::Record) {
                if arguments.len() != 1 {
                    return TypeFormResolution::Invalid(TypeFormationInvalid::TooManyTypeArguments);
                }
                let phalcom_ast::ast::GenericArgumentSyntax::Single(argument) = &arguments[0] else {
                    return TypeFormResolution::Invalid(TypeFormationInvalid::Syntax);
                };
                let row_parameter = match &argument.expr {
                    TypeAnnotationExpr::Reference(reference) if reference.members.is_empty() => {
                        match resolver.resolve_type_level_binding(reference.leaf_name()) {
                            Some(TypeLevelBinding::RecordRow(parameter)) => Some(parameter),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                if let Some(parameter) = row_parameter {
                    let row = match store.record_row_checked(Vec::new(), RecordRowTail::Parameter(parameter)) {
                        Ok(row) => row,
                        Err(error) => return TypeFormResolution::InternalFailure(error.to_string().into()),
                    };
                    return TypeFormResolution::Ready(store.record_type(row));
                }
                let applied_argument = type_form_ready_or_return!(resolve_type_form(store, declarations, resolver, site, argument, diagnostics));
                if let TypeData::Record(row) = store.get(applied_argument) {
                    return TypeFormResolution::Ready(store.record_type(*row));
                }
                diagnostics.push(SemanticDiagnostic::error_in(
                    current_module.clone(),
                    DiagnosticCode::KindExpectedType,
                    "Record application requires a RecordRow argument",
                    argument.range,
                ));
                return TypeFormResolution::Invalid(TypeFormationInvalid::TypeArgumentKindMismatch);
            }

            // Canonical origins retain application mode independently from
            // their arrow kind. This also covers residual alias lambdas.
            if let Some(contract) = generic_application_contract(store, declarations, origin_ty)
                && contract.has_variadic_binder()
            {
                debug_assert_eq!(contract.parameter_kinds.len(), contract.parameter_modes.len());
                enum Contribution {
                    Single(TypeId),
                    Positional(TypeId),
                    Labeled(TypeId),
                    Complete(TypeId),
                }

                macro_rules! ready_or_return {
                    ($result:expr) => {
                        match $result {
                            TypeFormResolution::Ready(ty) => ty,
                            TypeFormResolution::Dynamic => return TypeFormResolution::Dynamic,
                            TypeFormResolution::Unresolved(reason) => return TypeFormResolution::Unresolved(reason),
                            TypeFormResolution::Missing(reason) => return TypeFormResolution::Missing(reason),
                            TypeFormResolution::Invalid(reason) => return TypeFormResolution::Invalid(reason),
                            TypeFormResolution::Blocked(reason) => return TypeFormResolution::Blocked(reason),
                            TypeFormResolution::Cancelled => return TypeFormResolution::Cancelled,
                            TypeFormResolution::BudgetExceeded(report) => return TypeFormResolution::BudgetExceeded(report),
                            TypeFormResolution::InternalFailure(failure) => return TypeFormResolution::InternalFailure(failure),
                        }
                    };
                }

                let mut contributions = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    let (mode, operand, range) = match argument {
                        phalcom_ast::ast::GenericArgumentSyntax::Single(operand) => {
                            contributions.push(Contribution::Single(ready_or_return!(resolve_type_form(
                                store,
                                declarations,
                                resolver,
                                site,
                                operand,
                                diagnostics,
                            ))));
                            continue;
                        }
                        phalcom_ast::ast::GenericArgumentSyntax::Expansion(expansion) => (expansion.mode, &expansion.operand, expansion.range),
                    };
                    let operand = ready_or_return!(resolve_type_form(store, declarations, resolver, site, operand, diagnostics));
                    let projected = match mode {
                        ExpansionMode::Positional => store.project_positionals(operand).map(Contribution::Positional),
                        ExpansionMode::Labeled => store.project_labeled(operand).map(Contribution::Labeled),
                        ExpansionMode::Complete => store.project_complete(operand).map(Contribution::Complete),
                    };
                    match projected {
                        Ok(contribution) => contributions.push(contribution),
                        Err(error) => {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
                                DiagnosticCode::AnnotationUnsupported,
                                format!("invalid type-pack expansion: {error}"),
                                range,
                            ));
                            return TypeFormResolution::Invalid(TypeFormationInvalid::Syntax);
                        }
                    }
                }

                let fixed_count = contract
                    .parameter_modes
                    .iter()
                    .take_while(|mode| **mode == GenericParameterMode::Single)
                    .count();
                if contributions.len() < fixed_count {
                    if contributions.is_empty() {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            current_module.clone(),
                            DiagnosticCode::ApplicationTooManyArguments,
                            "required generic arguments are missing",
                            annotation.range,
                        ));
                        return TypeFormResolution::Invalid(TypeFormationInvalid::Syntax);
                    }
                    if contributions.iter().any(|contribution| !matches!(contribution, Contribution::Single(_))) {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            current_module.clone(),
                            DiagnosticCode::AnnotationUnsupported,
                            "a symbolic pack cannot satisfy a preceding fixed generic binder without pack decomposition",
                            annotation.range,
                        ));
                        return TypeFormResolution::Invalid(TypeFormationInvalid::Syntax);
                    }
                } else {
                    let mut canonical_arguments = Vec::with_capacity(contract.parameter_modes.len());
                    for contribution in contributions.iter().take(fixed_count) {
                        let Contribution::Single(ty) = contribution else {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
                                DiagnosticCode::AnnotationUnsupported,
                                "a pack expansion cannot satisfy a fixed generic binder without decomposition",
                                annotation.range,
                            ));
                            return TypeFormResolution::Invalid(TypeFormationInvalid::Syntax);
                        };
                        canonical_arguments.push(*ty);
                    }

                    let mut positional_segments = Vec::new();
                    let mut labeled_segments = Vec::new();
                    for contribution in contributions.into_iter().skip(fixed_count) {
                        match contribution {
                            Contribution::Single(ty) => {
                                if store.kind_of(ty) != KindId::TYPE {
                                    diagnostics.push(SemanticDiagnostic::error_in(
                                        current_module.clone(),
                                        DiagnosticCode::KindExpectedType,
                                        "a positional generic pack can contain only proper types",
                                        annotation.range,
                                    ));
                                    return TypeFormResolution::Invalid(TypeFormationInvalid::ExpectedProperType { actual: store.kind_of(ty) });
                                }
                                let pack = match store.type_pack(vec![ty].into_boxed_slice()) {
                                    Ok(pack) => pack,
                                    Err(error) => return TypeFormResolution::InternalFailure(error.to_string().into()),
                                };
                                positional_segments.push(pack);
                            }
                            Contribution::Positional(pack) => positional_segments.push(pack),
                            Contribution::Labeled(pack) => labeled_segments.push(pack),
                            Contribution::Complete(pack) => {
                                let positionals = match store.project_positionals(pack) {
                                    Ok(positionals) => positionals,
                                    Err(error) => return TypeFormResolution::InternalFailure(error.to_string().into()),
                                };
                                let labeled = match store.project_labeled(pack) {
                                    Ok(labeled) => labeled,
                                    Err(error) => return TypeFormResolution::InternalFailure(error.to_string().into()),
                                };
                                positional_segments.push(positionals);
                                labeled_segments.push(labeled);
                            }
                        }
                    }
                    let positionals = match store.type_pack_concat(&positional_segments) {
                        Ok(pack) => pack,
                        Err(error) => return TypeFormResolution::InternalFailure(error.to_string().into()),
                    };
                    let labeled = match store.labeled_type_pack_concat(&labeled_segments) {
                        Ok(pack) => pack,
                        Err(error) => return TypeFormResolution::InternalFailure(error.to_string().into()),
                    };
                    for mode in contract.parameter_modes.iter().skip(fixed_count) {
                        match mode {
                            GenericParameterMode::PositionalPack => canonical_arguments.push(positionals),
                            GenericParameterMode::LabeledPack => canonical_arguments.push(labeled),
                            GenericParameterMode::CompletePack => match store.complete_type_pack(positionals, labeled) {
                                Ok(pack) => canonical_arguments.push(pack),
                                Err(error) => return TypeFormResolution::InternalFailure(error.to_string().into()),
                            },
                            GenericParameterMode::Single => {
                                return TypeFormResolution::InternalFailure("non-prefix fixed generic binder escaped signature validation".into());
                            }
                        }
                    }
                    if intrinsic_type_application_for_form(store, declarations, origin_ty) == Some(IntrinsicTypeApplication::Tuple)
                        && canonical_arguments.len() == 1
                    {
                        let complete = canonical_arguments[0];
                        let positionals = match store.project_positionals(complete) {
                            Ok(pack) => pack,
                            Err(_error) => return TypeFormResolution::Invalid(TypeFormationInvalid::TypeArgumentKindMismatch),
                        };
                        let labeled = match store.project_labeled(complete) {
                            Ok(pack) => pack,
                            Err(_error) => return TypeFormResolution::Invalid(TypeFormationInvalid::TypeArgumentKindMismatch),
                        };
                        return match store.tuple_from_packs(positionals, labeled) {
                            Ok(tuple) => TypeFormResolution::Ready(tuple),
                            Err(error) => TypeFormResolution::InternalFailure(error.to_string().into()),
                        };
                    }
                    return match store.apply_type_form(origin_ty, &canonical_arguments) {
                        Ok(applied) => TypeFormResolution::Ready(applied),
                        Err(error) => TypeFormResolution::Invalid(match error {
                            TypeApplicationError::NotAConstructor { .. } => TypeFormationInvalid::NotAConstructor,
                            TypeApplicationError::TooManyArguments { .. } => TypeFormationInvalid::TooManyTypeArguments,
                            TypeApplicationError::ArgumentKindMismatch { .. } => TypeFormationInvalid::TypeArgumentKindMismatch,
                            TypeApplicationError::MalformedLambda => TypeFormationInvalid::MalformedTypeLambda,
                        }),
                    };
                }
            }

            let mut arg_tys = Vec::with_capacity(arguments.len());
            for arg in arguments {
                let phalcom_ast::ast::GenericArgumentSyntax::Single(arg) = arg else {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        current_module.clone(),
                        DiagnosticCode::AnnotationUnsupported,
                        "type-pack expansion is not available in generic applications yet",
                        arg.range(),
                    ));
                    return TypeFormResolution::Invalid(TypeFormationInvalid::Syntax);
                };
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
            let mut positional_segments = Vec::new();
            let mut labeled_segments = Vec::new();
            let mut labeled_phase = false;
            for elem in elements {
                match elem {
                    phalcom_ast::ast::TypeTupleItem::Element(elem) => {
                        let ty = type_form_ready_or_return!(resolve_type_form(store, declarations, resolver, site, &elem.ty, diagnostics));
                        if store.kind_of(ty) != KindId::TYPE {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
                                DiagnosticCode::KindExpectedType,
                                "tuple element must be a proper type",
                                elem.range,
                            ));
                            return TypeFormResolution::Invalid(TypeFormationInvalid::ExpectedProperType { actual: store.kind_of(ty) });
                        }
                        if let Some(label) = &elem.label {
                            labeled_phase = true;
                            match store.labeled_type_pack(
                                vec![super::store::LabeledTypePackElement {
                                    label: label.clone().into(),
                                    element: ty,
                                }]
                                .into_boxed_slice(),
                            ) {
                                Ok(pack) => labeled_segments.push(pack),
                                Err(error) => return TypeFormResolution::InternalFailure(error.to_string().into()),
                            }
                        } else {
                            if labeled_phase {
                                diagnostics.push(SemanticDiagnostic::error_in(
                                    current_module.clone(),
                                    DiagnosticCode::AnnotationUnsupported,
                                    "positional Tuple entries cannot follow labeled entries",
                                    elem.range,
                                ));
                                return TypeFormResolution::Invalid(TypeFormationInvalid::Syntax);
                            }
                            match store.type_pack(vec![ty].into_boxed_slice()) {
                                Ok(pack) => positional_segments.push(pack),
                                Err(error) => return TypeFormResolution::InternalFailure(error.to_string().into()),
                            }
                        }
                    }
                    phalcom_ast::ast::TypeTupleItem::Expansion(expansion) => {
                        if labeled_phase && expansion.mode != ExpansionMode::Labeled {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
                                DiagnosticCode::AnnotationUnsupported,
                                "positional type-pack expansion cannot follow labeled Tuple entries",
                                expansion.range,
                            ));
                            return TypeFormResolution::Invalid(TypeFormationInvalid::Syntax);
                        }
                        let operand = type_form_ready_or_return!(resolve_type_form(store, declarations, resolver, site, &expansion.operand, diagnostics,));
                        let result = match expansion.mode {
                            ExpansionMode::Positional => store.project_positionals(operand).map(|pack| {
                                positional_segments.push(pack);
                            }),
                            ExpansionMode::Labeled => store.project_labeled(operand).map(|pack| {
                                labeled_phase = true;
                                labeled_segments.push(pack);
                            }),
                            ExpansionMode::Complete => store.project_complete(operand).and_then(|pack| {
                                positional_segments.push(store.project_positionals(pack)?);
                                labeled_segments.push(store.project_labeled(pack)?);
                                Ok(())
                            }),
                        };
                        if let Err(error) = result {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
                                DiagnosticCode::AnnotationUnsupported,
                                format!("invalid Tuple type-pack expansion: {error}"),
                                expansion.range,
                            ));
                            return TypeFormResolution::Invalid(TypeFormationInvalid::Syntax);
                        }
                    }
                }
            }
            let positionals = match store.type_pack_concat(&positional_segments) {
                Ok(pack) => pack,
                Err(error) => return TypeFormResolution::InternalFailure(error.to_string().into()),
            };
            let labeled = match store.labeled_type_pack_concat(&labeled_segments) {
                Ok(pack) => pack,
                Err(error) => return TypeFormResolution::InternalFailure(error.to_string().into()),
            };
            match store.tuple_from_packs(positionals, labeled) {
                Ok(tuple) => TypeFormResolution::Ready(tuple),
                Err(error) => TypeFormResolution::InternalFailure(error.to_string().into()),
            }
        }
        TypeAnnotationExpr::Record { fields, tail, range: _ } => {
            let record_tail = match tail {
                None => RecordRowTail::Closed,
                Some(tail) => match resolver.resolve_type_level_binding(&tail.name) {
                    Some(TypeLevelBinding::RecordRow(parameter)) => RecordRowTail::Parameter(parameter),
                    Some(TypeLevelBinding::TypeForm(form)) => {
                        let actual = store.kind_of(form);
                        diagnostics.push(SemanticDiagnostic::error_in(
                            current_module.clone(),
                            DiagnosticCode::RecordRowTailKindMismatch,
                            format!("record row tail `{}` must have kind RecordRow", tail.name),
                            annotation.range,
                        ));
                        return TypeFormResolution::Invalid(TypeFormationInvalid::RecordRowTailKindMismatch { actual });
                    }
                    None => {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            current_module.clone(),
                            DiagnosticCode::RecordRowTailUnresolved,
                            format!("unresolved record row tail `{}`", tail.name),
                            annotation.range,
                        ));
                        return TypeFormResolution::Unresolved(TypeFormationUnresolved::Name(tail.name.clone().into()));
                    }
                },
            };
            let mut record_fields = Vec::with_capacity(fields.len());
            let mut seen_names = std::collections::HashSet::new();
            for field in fields {
                if !seen_names.insert(field.name.clone()) {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        current_module.clone(),
                        DiagnosticCode::RecordDuplicateField,
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
            match store.record_row_type_checked(record_fields, record_tail) {
                Ok(record_ty) => TypeFormResolution::Ready(record_ty),
                Err(crate::types::row::RecordRowFormationError::DuplicateField(field)) => {
                    TypeFormResolution::Invalid(TypeFormationInvalid::DuplicateRecordField(field))
                }
                Err(crate::types::row::RecordRowFormationError::TailParameterWrongKind { actual, .. }) => {
                    TypeFormResolution::Invalid(TypeFormationInvalid::RecordRowTailKindMismatch { actual })
                }
                Err(crate::types::row::RecordRowFormationError::TailParameterMissing(parameter)) => {
                    TypeFormResolution::InternalFailure(format!("missing record row tail parameter {parameter:?}"))
                }
                Err(crate::types::row::RecordRowFormationError::FieldNotProperType { .. }) => {
                    TypeFormResolution::InternalFailure("record field escaped proper-type validation".into())
                }
            }
        }
        TypeAnnotationExpr::Callable { parameters, result, range: _ } => {
            let mut positional_segments = Vec::new();
            let mut labeled_segments = Vec::new();
            let mut fixed_parameters = Vec::with_capacity(parameters.len());
            let mut has_expansion = false;
            let mut labeled_phase = false;
            for param in parameters {
                match param {
                    phalcom_ast::ast::TypeCallableItem::Parameter(param) => {
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
                        fixed_parameters.push(CallableParameterType {
                            label: param.label.clone().map(Into::into),
                            ty,
                            rest: if param.rest {
                                phalcom_ast::ast::RestMode::Positional
                            } else {
                                phalcom_ast::ast::RestMode::None
                            },
                        });
                        if let Some(label) = &param.label {
                            labeled_phase = true;
                            match store.labeled_type_pack(
                                vec![crate::types::store::LabeledTypePackElement {
                                    label: label.clone().into(),
                                    element: ty,
                                }]
                                .into_boxed_slice(),
                            ) {
                                Ok(pack) => labeled_segments.push(pack),
                                Err(error) => return TypeFormResolution::InternalFailure(error.to_string().into()),
                            }
                        } else {
                            if labeled_phase {
                                diagnostics.push(SemanticDiagnostic::error_in(
                                    current_module.clone(),
                                    DiagnosticCode::AnnotationUnsupported,
                                    "positional callable parameter cannot follow labeled parameter",
                                    param.range,
                                ));
                                return TypeFormResolution::Invalid(TypeFormationInvalid::Syntax);
                            }
                            match store.type_pack(vec![ty].into_boxed_slice()) {
                                Ok(pack) => positional_segments.push(pack),
                                Err(error) => return TypeFormResolution::InternalFailure(error.to_string().into()),
                            }
                        }
                    }
                    phalcom_ast::ast::TypeCallableItem::Expansion(expansion) => {
                        has_expansion = true;
                        if labeled_phase && expansion.mode != ExpansionMode::Labeled {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
                                DiagnosticCode::AnnotationUnsupported,
                                "positional type-pack expansion cannot follow labeled callable parameters",
                                expansion.range,
                            ));
                            return TypeFormResolution::Invalid(TypeFormationInvalid::Syntax);
                        }
                        let operand = type_form_ready_or_return!(resolve_type_form(store, declarations, resolver, site, &expansion.operand, diagnostics,));
                        let result = match expansion.mode {
                            ExpansionMode::Positional => store.project_positionals(operand).map(|pack| {
                                positional_segments.push(pack);
                            }),
                            ExpansionMode::Labeled => store.project_labeled(operand).map(|pack| {
                                labeled_phase = true;
                                labeled_segments.push(pack);
                            }),
                            ExpansionMode::Complete => store.project_complete(operand).and_then(|pack| {
                                positional_segments.push(store.project_positionals(pack)?);
                                labeled_segments.push(store.project_labeled(pack)?);
                                Ok(())
                            }),
                        };
                        if let Err(error) = result {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                current_module.clone(),
                                DiagnosticCode::AnnotationUnsupported,
                                format!("invalid callable type-pack expansion: {error}"),
                                expansion.range,
                            ));
                            return TypeFormResolution::Invalid(TypeFormationInvalid::Syntax);
                        }
                    }
                }
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

            if !has_expansion {
                let callable_ty = store.callable(CallableType {
                    parameters: fixed_parameters.into_boxed_slice(),
                    return_type,
                });
                return TypeFormResolution::Ready(callable_ty);
            }

            let positionals = match store.type_pack_concat(&positional_segments) {
                Ok(pack) => pack,
                Err(error) => return TypeFormResolution::InternalFailure(error.to_string().into()),
            };
            let labeled = match store.labeled_type_pack_concat(&labeled_segments) {
                Ok(pack) => pack,
                Err(error) => return TypeFormResolution::InternalFailure(error.to_string().into()),
            };
            match store.callable_from_packs(positionals, labeled, return_type) {
                Ok(callable) => TypeFormResolution::Ready(callable),
                Err(error) => TypeFormResolution::InternalFailure(error.to_string().into()),
            }
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
        TypeAnnotationExpr::ExactEnumCase {
            enum_target,
            variant_name,
            variant_name_range,
            generic_arguments,
            payload_shape,
            range,
        } => {
            if let Some(result) = resolve_static_case_form(
                store, declarations, resolver, site, enum_target, variant_name,
                generic_arguments, payload_shape.as_ref(), *range, diagnostics,
            ) {
                return result;
            }
            if generic_arguments.is_empty() && payload_shape.is_none() {
                if let Some(result) = resolve_associated_static_domain_form(
                    store, declarations, resolver, site, enum_target, variant_name,
                ) {
                    return result;
                }
            }
            if generic_arguments.is_empty() && payload_shape.is_none() {
                return resolve_associated_type_projection(
                    store,
                    declarations,
                    resolver,
                    site,
                    enum_target,
                    variant_name,
                    *variant_name_range,
                    *range,
                    diagnostics,
                );
            }
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnresolved,
                "exact enum case syntax is only valid as an inherent impl target",
                *range,
            ));
            TypeFormResolution::Invalid(TypeFormationInvalid::Syntax)
        }
        TypeAnnotationExpr::TypeMemberProjection {
            subject,
            selector,
            arguments,
            range,
        } => resolve_type_member_projection(store, declarations, resolver, site, subject, selector, arguments, *range, diagnostics),
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

#[allow(clippy::too_many_arguments)]
fn resolve_type_member_projection(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    subject: &TypeAnnotation,
    selector: &phalcom_ast::ast::TypeMemberSelectorSyntax,
    arguments: &[phalcom_ast::ast::TypeMemberArgumentSyntax],
    range: phalcom_common::range::SourceRange,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormResolution {
    let current_module = &site.module;
    let subject_res = resolve_type_form(store, declarations, resolver, site, subject, diagnostics);
    let subject_ty = match subject_res {
        TypeFormResolution::Ready(ty) => ty,
        other => return other,
    };

    let canonical_selector = TypeMemberSelector::from_syntax(selector);
    let mut canonical_args = Vec::with_capacity(arguments.len());

    for arg in arguments {
        match arg {
            phalcom_ast::ast::TypeMemberArgumentSyntax::TypeForm(ann) => {
                let arg_res = resolve_type_form(store, declarations, resolver, site, ann, diagnostics);
                let arg_ty = match arg_res {
                    TypeFormResolution::Ready(ty) => ty,
                    other => return other,
                };
                canonical_args.push(TypeMemberArgument::Type(arg_ty));
            }
            phalcom_ast::ast::TypeMemberArgumentSyntax::IntLiteral { value, .. } => {
                canonical_args.push(TypeMemberArgument::Type(store.const_int(*value)));
            }
            phalcom_ast::ast::TypeMemberArgumentSyntax::Name { name, range: arg_range } => {
                let dummy_ann = TypeAnnotation {
                    expr: TypeAnnotationExpr::Reference(phalcom_ast::ast::StaticSymbolRef {
                        root: name.clone(),
                        root_range: *arg_range,
                        members: Vec::new(),
                        range: *arg_range,
                    }),
                    range: *arg_range,
                };
                let arg_res = resolve_type_form(store, declarations, resolver, site, &dummy_ann, diagnostics);
                let arg_ty = match arg_res {
                    TypeFormResolution::Ready(ty) => ty,
                    other => return other,
                };
                canonical_args.push(TypeMemberArgument::Type(arg_ty));
            }
        }
    }

    let intrinsic_owner = match store.get(subject_ty) {
        TypeData::Tuple(_) | TypeData::TupleFromPacks { .. } | TypeData::Unit => Some(crate::types::type_member::IntrinsicTypeOwner::Tuple),
        TypeData::Record(_) => Some(crate::types::type_member::IntrinsicTypeOwner::Record),
        TypeData::TypePack(_) => Some(crate::types::type_member::IntrinsicTypeOwner::TypePack),
        TypeData::LabeledTypePack(_) => Some(crate::types::type_member::IntrinsicTypeOwner::LabeledTypePack),
        TypeData::Nominal { declaration } => declarations.intrinsic_type_owner(declaration),
        _ => None,
    };
    let intrinsic = intrinsic_owner.and_then(|owner| lookup_intrinsic_type_operation(owner, &canonical_selector));

    let mut kind = match intrinsic {
        Some(IntrinsicTypeOperation::TuplePositionals) | Some(IntrinsicTypeOperation::TypePackMap) => KindId::TYPE_PACK,
        Some(IntrinsicTypeOperation::TupleLabeled) | Some(IntrinsicTypeOperation::LabeledTypePackMap) => KindId::LABELED_TYPE_PACK,
        Some(IntrinsicTypeOperation::TupleEntries) => KindId::COMPLETE_TYPE_PACK,
        _ => KindId::TYPE,
    };
    let map_operation = matches!(
        intrinsic,
        Some(
            IntrinsicTypeOperation::TupleMap
                | IntrinsicTypeOperation::RecordMap
                | IntrinsicTypeOperation::TypePackMap
                | IntrinsicTypeOperation::LabeledTypePackMap
                | IntrinsicTypeOperation::RecordRowMap,
        )
    );
    let at_operation = matches!(intrinsic, Some(IntrinsicTypeOperation::TupleAt));
    let valid_arguments = if map_operation {
        let expected_kind = store.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
        matches!(canonical_args.as_slice(), [TypeMemberArgument::Type(argument)] if store.kind_of(*argument) == expected_kind)
    } else if at_operation {
        matches!(canonical_args.as_slice(), [TypeMemberArgument::Type(argument)] if matches!(store.get(*argument), TypeData::ConstInt(_)))
    } else {
        canonical_args.is_empty()
    };
    if intrinsic.is_some() && !valid_arguments {
        diagnostics.push(SemanticDiagnostic::error_in(
            current_module.clone(),
            DiagnosticCode::AnnotationUnresolved,
            "type-member arguments do not match the intrinsic member signature",
            range,
        ));
        return TypeFormResolution::Invalid(TypeFormationInvalid::TypeArgumentKindMismatch);
    }

    let selected_source = if intrinsic.is_none() {
        store
            .applied_nominal_static_parts(subject_ty)
            .and_then(|(owner, _)| resolver.resolve_associated_type_member(&owner, &canonical_selector))
    } else {
        None
    };
    let source_declaration = if let Some(product) = selected_source {
        if let Err(error) = super::type_member::validate_type_member_arguments(store, &product.signature, &canonical_args) {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnresolved,
                format!("type-member arguments do not match the selected declaration: {error:?}"),
                range,
            ));
            return TypeFormResolution::Invalid(TypeFormationInvalid::TypeArgumentKindMismatch);
        }
        let super::type_member::TypeMemberDomain::Kind(result_kind) = product.signature.result_domain;
        kind = result_kind;
        Some(product.id.clone())
    } else {
        None
    };

    let projection = TypeLevelProjection {
        subject: ProjectionSubject::Type(subject_ty),
        selector: canonical_selector,
        arguments: canonical_args.into_boxed_slice(),
        source_declaration,
    };

    let proj_ty = store.type_member_projection(projection, kind);

    let mut budget = crate::types::outcome::QueryBudget::new(10000);
    let cancel = crate::types::outcome::CancellationToken::new();
    let empty_hier = crate::types::relation::MapTypeHierarchy::new();
    let conformance_index = crate::impls::ConformanceIndex::new();
    let witness_plans = std::collections::BTreeMap::new();
    let trait_surfaces = crate::traits::TraitSurfaceTable::new();
    let mode = crate::types::projection::ProjectionNormalizationMode::Exact {
        conformance_index: &conformance_index,
        witness_plans: &witness_plans,
        trait_surfaces: &trait_surfaces,
        derived_rules: None,
        declarations,
        hierarchy: &empty_hier,
    };
    let mut context = crate::types::projection::ProjectionNormalizationContext::new(mode, &mut budget, &cancel);
    let norm_res = crate::types::projection::normalize_type(store, proj_ty, &mut context);
    match norm_res {
        crate::types::projection::ProjectionNormalizationResult::Normalized(ty) | crate::types::projection::ProjectionNormalizationResult::Symbolic(ty) => {
            TypeFormResolution::Ready(ty)
        }
        crate::types::projection::ProjectionNormalizationResult::InternalFailure(err) => {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnresolved,
                err.to_string(),
                range,
            ));
            TypeFormResolution::Invalid(TypeFormationInvalid::Syntax)
        }
        _ => TypeFormResolution::Ready(proj_ty),
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
        GenericConstraint::Conforms { subject, trait_ref } => {
            let args_str = if trait_ref.arguments.is_empty() {
                String::new()
            } else {
                format!(
                    "<{}>",
                    trait_ref.arguments.iter().map(|ty| store.format_type(*ty)).collect::<Vec<_>>().join(", ")
                )
            };
            format!("Conforms({}, {}{})", term_shape(store, subject), trait_ref.declaration.name, args_str)
        }
    }
    .into_boxed_str()
}

fn resolve_trait_ref_for_conformance(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    annotation: &TypeAnnotation,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormationOutcome<TraitRef> {
    let current_module = &site.module;
    let (trait_declaration, type_arguments_syntax) = match &annotation.expr {
        TypeAnnotationExpr::Reference(sym_ref) => {
            let members: Vec<String> = sym_ref.members.iter().map(|m| m.name.clone()).collect();
            let decl = resolver.resolve_trait_name(current_module, &sym_ref.root, &members);
            let Some(decl) = decl else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    current_module.clone(),
                    DiagnosticCode::AnnotationUnresolved,
                    format!("unresolved trait `{}`", sym_ref.root),
                    annotation.range,
                ));
                return TypeFormationOutcome::Unresolved(TypeFormationUnresolved::Name(sym_ref.root.clone().into()));
            };
            (decl, Vec::new())
        }
        TypeAnnotationExpr::Application { origin, arguments, .. } => {
            let TypeAnnotationExpr::Reference(sym_ref) = &origin.expr else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    current_module.clone(),
                    DiagnosticCode::AnnotationUnresolved,
                    "conformance trait reference origin must be a trait name",
                    origin.range,
                ));
                return TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax);
            };
            let members: Vec<String> = sym_ref.members.iter().map(|m| m.name.clone()).collect();
            let decl = resolver.resolve_trait_name(current_module, &sym_ref.root, &members);
            let Some(decl) = decl else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    current_module.clone(),
                    DiagnosticCode::AnnotationUnresolved,
                    format!("unresolved trait `{}`", sym_ref.root),
                    origin.range,
                ));
                return TypeFormationOutcome::Unresolved(TypeFormationUnresolved::Name(sym_ref.root.clone().into()));
            };
            let mut type_arguments = Vec::with_capacity(arguments.len());
            for argument in arguments {
                let phalcom_ast::ast::GenericArgumentSyntax::Single(argument) = argument else {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        current_module.clone(),
                        DiagnosticCode::AnnotationUnsupported,
                        "type-pack expansion is not available in conformance bounds",
                        argument.range(),
                    ));
                    return TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax);
                };
                type_arguments.push(argument.clone());
            }
            (decl, type_arguments)
        }
        _ => {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnresolved,
                "conformance constraint bound must be a trait reference",
                annotation.range,
            ));
            return TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax);
        }
    };

    let mut arg_tys = Vec::with_capacity(type_arguments_syntax.len());
    for arg_ann in &type_arguments_syntax {
        let arg_res = resolve_type_form(store, declarations, resolver, site, arg_ann, diagnostics);
        let arg_ty = scoped_ready_or_propagate!(arg_res);
        arg_tys.push(arg_ty);
    }

    TypeFormationOutcome::Ready(TraitRef::new(trait_declaration, arg_tys.into_boxed_slice()))
}

/// Resolves the constraint-free portion of a generic declaration. Nominal
/// shells publish this result before alias lowering; full signatures later add
/// constraints from the same canonical parameter identities.
pub fn resolve_generic_binder_header(
    store: &mut TypeStore,
    site: &TypeFormationSite,
    owner: TypeParameterOwner,
    binder_site: GenericBinderSite,
    params: &[GenericParameterSyntax],
    where_clause: Option<&WhereClauseSyntax>,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormationOutcome<GenericBinderHeader> {
    resolve_generic_binder_header_with_context(store, site, owner, binder_site, params, where_clause, diagnostics, None, None)
}

/// Resolves generic binders with declaration-aware static-domain kinds.
///
/// The legacy wrapper above remains useful during predeclaration, where the
/// declaration table is not available yet. Full signature formation supplies
/// both the resolver and declaration table so `N: Int`/`S: Shape` can name a
/// canonical static domain rather than being mistaken for `Type`.
pub fn resolve_generic_binder_header_with_context(
    store: &mut TypeStore,
    site: &TypeFormationSite,
    owner: TypeParameterOwner,
    binder_site: GenericBinderSite,
    params: &[GenericParameterSyntax],
    where_clause: Option<&WhereClauseSyntax>,
    diagnostics: &mut Vec<SemanticDiagnostic>,
    declarations: Option<&DeclarationTypeTable>,
    resolver: Option<&dyn TypeResolver>,
) -> TypeFormationOutcome<GenericBinderHeader> {
    let current_module = &site.module;

    // Phase 1: Unify parameter kind facts from binder and where-clause KindFacts
    let mut where_kind_facts = std::collections::HashMap::new();
    if let Some(clause) = where_clause {
        for c in &clause.constraints {
            if let GenericConstraintSyntax::KindFact { subject, kind, range, .. } = c {
                let resolved_kind = resolve_contextual_kind_syntax(store, declarations, resolver, site, kind, diagnostics);
                if let KindResolution::Invalid(reason) = &resolved_kind {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        current_module.clone(),
                        DiagnosticCode::AnnotationUnresolved,
                        format!("invalid generic parameter kind in where clause: {reason:?}"),
                        *range,
                    ));
                }
                let k = scoped_ready_or_propagate!(resolved_kind);
                where_kind_facts.insert(subject.clone(), (k, *range));
            }
        }
    }

    let mut pending = Vec::with_capacity(params.len());
    let mut positional_captured = false;
    let mut labeled_captured = false;
    let mut complete_captured = false;
    for p in params {
        let mode = match p.mode {
            None => GenericParameterMode::Single,
            Some(ExpansionMode::Positional) => GenericParameterMode::PositionalPack,
            Some(ExpansionMode::Labeled) => GenericParameterMode::LabeledPack,
            Some(ExpansionMode::Complete) => GenericParameterMode::CompletePack,
        };
        let invalid_order = match mode {
            GenericParameterMode::Single => positional_captured || labeled_captured || complete_captured,
            GenericParameterMode::PositionalPack => positional_captured || labeled_captured || complete_captured,
            GenericParameterMode::LabeledPack => labeled_captured || complete_captured,
            GenericParameterMode::CompletePack => positional_captured || labeled_captured || complete_captured,
        };
        if invalid_order {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnsupported,
                "generic pack binders must have deterministic, non-overlapping lane ownership",
                p.mode_range.unwrap_or(p.range),
            ));
            return TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax);
        }
        if mode != GenericParameterMode::Single && p.variance != VarianceSyntax::Invariant {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnsupported,
                "variance on variadic generic pack binders is unsupported",
                p.mode_range.unwrap_or(p.range),
            ));
            return TypeFormationOutcome::Invalid(TypeFormationInvalid::InvalidVariance);
        }
        match mode {
            GenericParameterMode::PositionalPack => positional_captured = true,
            GenericParameterMode::LabeledPack => labeled_captured = true,
            GenericParameterMode::CompletePack => complete_captured = true,
            GenericParameterMode::Single => {}
        }
        let binder_kind = match p.kind.as_ref() {
            None => None,
            Some(syntax) => {
                let res = resolve_contextual_kind_syntax(store, declarations, resolver, site, syntax, diagnostics);
                if let KindResolution::Invalid(reason) = &res {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        current_module.clone(),
                        DiagnosticCode::AnnotationUnresolved,
                        format!("invalid generic parameter kind: {reason:?}"),
                        p.range,
                    ));
                }
                Some(scoped_ready_or_propagate!(res))
            }
        };

        let default_kind = match mode {
            GenericParameterMode::Single => KindId::TYPE,
            GenericParameterMode::PositionalPack => KindId::TYPE_PACK,
            GenericParameterMode::LabeledPack => KindId::LABELED_TYPE_PACK,
            GenericParameterMode::CompletePack => KindId::COMPLETE_TYPE_PACK,
        };
        let declared_kind = binder_kind.unwrap_or(default_kind);

        if !super::parameter::expansion_mode_accepts_kind(mode, declared_kind) {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnsupported,
                "generic parameter expansion mode is incompatible with its declared kind",
                p.kind.as_ref().map_or(p.range, KindSyntax::range),
            ));
            return TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax);
        }

        let kind = match (Some(declared_kind), where_kind_facts.get(&p.name)) {
            (Some(k), None) => k,
            (Some(k1), Some((k2, where_range))) => {
                if k1 != *k2 {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        current_module.clone(),
                        DiagnosticCode::AnnotationUnresolved,
                        format!("conflicting generic parameter kind for `{}` between binder and where clause", p.name),
                        *where_range,
                    ));
                    return TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax);
                }
                k1
            }
            _ => declared_kind,
        };

        let variance = lower_variance(p.variance);
        if kind == KindId::RECORD_ROW && binder_site == GenericBinderSite::Trait {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnsupported,
                "RecordFields generic binders on traits are unsupported until trait evidence is generalized",
                p.range,
            ));
            return TypeFormationOutcome::Invalid(TypeFormationInvalid::UnsupportedRecordRowBinderSite);
        }
        let uses_user_static_domain = matches!(store.get_kind(kind), KindData::StaticDomain(domain)
            if matches!(store.static_domain(*domain),
                Some(super::store::StaticDomainData::Nominal { .. } | super::store::StaticDomainData::Associated { .. })));
        if binder_site == GenericBinderSite::Trait && uses_user_static_domain {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnsupported,
                "trait generic parameters over user-defined static domains are unsupported until trait evidence is generalized",
                p.range,
            ));
            return TypeFormationOutcome::Invalid(TypeFormationInvalid::UnsupportedTraitStaticDomainBinder);
        }
        if !matches!(binder_site, GenericBinderSite::NominalDeclaration | GenericBinderSite::Trait) && variance != Variance::Invariant {
            diagnostics.push(SemanticDiagnostic::error_in(
                current_module.clone(),
                DiagnosticCode::AnnotationUnsupported,
                "variance annotations are only valid on nominal declaration parameters",
                p.range,
            ));
            return TypeFormationOutcome::Invalid(TypeFormationInvalid::InvalidVariance);
        }
        pending.push((p, kind, variance, mode));
    }

    // Phase 2: Intern parameter identities.
    let mut param_ids = Vec::with_capacity(params.len());
    for (idx, (p, kind, variance, mode)) in pending.iter().enumerate() {
        let data = TypeParameterData::new(owner.clone(), idx as u32, p.name.clone(), *kind)
            .with_variance(*variance)
            .with_mode(*mode)
            .with_source(crate::diagnostic::SemanticSourceSpan::new(current_module.clone(), p.range));
        let param_id = store.intern_type_parameter(data);
        param_ids.push(param_id);
    }

    TypeFormationOutcome::Ready(GenericBinderHeader {
        owner,
        parameters: param_ids.into_boxed_slice(),
        parameter_kinds: pending.iter().map(|(_, kind, _, _)| *kind).collect::<Vec<_>>().into_boxed_slice(),
        parameter_variances: pending.iter().map(|(_, _, variance, _)| *variance).collect::<Vec<_>>().into_boxed_slice(),
        parameter_modes: pending.iter().map(|(_, _, _, mode)| *mode).collect::<Vec<_>>().into_boxed_slice(),
    })
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
    let binders = scoped_ready_or_propagate!(resolve_generic_binder_header_with_context(
        store,
        site,
        owner.clone(),
        binder_site,
        params,
        where_clause,
        diagnostics,
        Some(declarations),
        Some(resolver),
    ));

    let mut param_map = std::collections::HashMap::new();
    for (p, &param_id) in params.iter().zip(binders.parameters.iter()) {
        let binding = type_level_binding_for_parameter(store, param_id);
        param_map.insert(p.name.clone(), binding);
    }
    let scoped_resolver = ScopedTypeResolver {
        parent: resolver,
        type_parameters: param_map,
    };

    let where_site = if let Some(context) = &site.associated_type_context {
        let mut updated_context = context.clone();
        if updated_context.trait_ref.arguments.is_empty() && !binders.parameters.is_empty() {
            updated_context.trait_ref.arguments = binders
                .parameters
                .iter()
                .map(|&p| store.parameter_form(p))
                .collect::<Vec<_>>()
                .into_boxed_slice();
        }
        let mut site_clone = site.clone();
        site_clone.associated_type_context = Some(updated_context);
        site_clone
    } else {
        site.clone()
    };

    // Phase 3: Lower direct conformance bounds and extract specializations
    let mut constraints = Vec::new();
    let mut pending_specializations = Vec::new();

    if let Some(clause) = where_clause {
        for c in &clause.constraints {
            match c {
                GenericConstraintSyntax::Subtype { lower, upper, range: _ } => {
                    let l_res = resolve_type_form(store, declarations, &scoped_resolver, &where_site, lower, diagnostics);
                    let u_res = resolve_type_form(store, declarations, &scoped_resolver, &where_site, upper, diagnostics);
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
                    let l_res = resolve_type_form(store, declarations, &scoped_resolver, &where_site, left, diagnostics);
                    let r_res = resolve_type_form(store, declarations, &scoped_resolver, &where_site, right, diagnostics);
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
                    let left_term = TypeTerm::Canonical(l_ty);
                    let right_term = TypeTerm::Canonical(r_ty);
                    let (left, right) = if left_term <= right_term {
                        (left_term, right_term)
                    } else {
                        (right_term, left_term)
                    };
                    constraints.push(GenericConstraint::Equivalent { left, right });
                }
                GenericConstraintSyntax::Conforms { subject, bounds, range: _ } => {
                    let s_res = resolve_type_form(store, declarations, &scoped_resolver, &where_site, subject, diagnostics);
                    let s_ty = scoped_ready_or_propagate!(s_res);
                    if store.kind_of(s_ty) != KindId::TYPE {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            current_module.clone(),
                            DiagnosticCode::KindExpectedType,
                            "generic conformance subject must be a proper type",
                            subject.range,
                        ));
                        return TypeFormationOutcome::Invalid(TypeFormationInvalid::GenericConstraintOperandNotType);
                    }
                    for bound in bounds {
                        let trait_res = resolve_trait_ref_for_conformance(store, declarations, &scoped_resolver, &where_site, &bound.trait_type, diagnostics);
                        let trait_ref = scoped_ready_or_propagate!(trait_res);
                        constraints.push(GenericConstraint::Conforms {
                            subject: TypeTerm::Canonical(s_ty),
                            trait_ref: trait_ref.clone(),
                        });
                        for spec in &bound.associated_specializations {
                            pending_specializations.push((s_ty, trait_ref.clone(), spec));
                        }
                    }
                }
                GenericConstraintSyntax::KindFact { .. } => {}
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

    // Phase 4: Elaborate associated specializations into canonical Equivalent constraints
    for (s_ty, trait_ref, spec) in pending_specializations {
        let val_res = resolve_type_form(store, declarations, &scoped_resolver, &where_site, &spec.value, diagnostics);
        let val_ty = scoped_ready_or_propagate!(val_res);
        let req_id = resolver
            .resolve_associated_type_requirement(&trait_ref.declaration, &spec.name)
            .unwrap_or_else(|| AssociatedTypeRequirementId::new(trait_ref.declaration.clone(), 0));
        let proj_ty = store.associated_projection(s_ty, trait_ref, req_id);
        let left_term = TypeTerm::Canonical(proj_ty);
        let right_term = TypeTerm::Canonical(val_ty);
        let (left, right) = if left_term <= right_term {
            (left_term, right_term)
        } else {
            (right_term, left_term)
        };
        constraints.push(GenericConstraint::Equivalent { left, right });
    }

    // Phase 5: Canonical sort and deduplication
    constraints.sort();
    constraints.dedup();

    let constraint_shapes = constraints
        .iter()
        .map(|constraint| generic_constraint_shape(store, constraint))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let signature = GenericSignature::with_constraints(owner, binders.parameters.clone(), constraints.into_boxed_slice())
        .with_parameter_metadata(binders.parameter_kinds.clone(), binders.parameter_variances.clone())
        .with_parameter_modes(binders.parameter_modes.clone())
        .with_constraint_shapes(constraint_shapes);
    let signature = signature.with_parameter_kind_shapes(
        binders
            .parameter_kinds
            .iter()
            .map(|kind| store.format_kind(*kind).into_boxed_str())
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    assert!(
        binders.matches_signature(&signature),
        "generic signature must complete its published binder header"
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
