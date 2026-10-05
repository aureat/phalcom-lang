//! Shared semantic static domain model distinguishing kinded terms from ordinary value types.

use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::SemanticDiagnostic;
use crate::types::annotation::{TypeFormResolution, TypeFormationInvalid, TypeFormationOutcome, TypeFormationSite, TypeResolver, resolve_type_form};
use crate::types::id::{KindId, ProperTypeId, TypeId};
use crate::types::kind::KindData;
use crate::types::store::TypeStore;
use crate::types::type_member::TypeMemberDomain;
use phalcom_ast::ast::{StaticDomainSyntax, TypeAnnotation, TypeAnnotationExpr, TypeCallableItem};

/// Semantic domain distinction for a static definition parameter or result.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StaticValueDomain {
    Kind(KindId),
    ValueType(TypeId),
}

impl StaticValueDomain {
    #[inline]
    pub fn is_kind(&self) -> bool {
        matches!(self, Self::Kind(_))
    }

    #[inline]
    pub fn is_value_type(&self) -> bool {
        matches!(self, Self::ValueType(_))
    }

    #[inline]
    pub fn as_kind(&self) -> Option<KindId> {
        match self {
            Self::Kind(k) => Some(*k),
            Self::ValueType(_) => None,
        }
    }

    #[inline]
    pub fn as_value_type(&self) -> Option<TypeId> {
        match self {
            Self::ValueType(t) => Some(*t),
            Self::Kind(_) => None,
        }
    }

    /// Validates whether this domain represents a valid kind or a valid proper value type.
    pub fn validate(&self, store: &TypeStore) -> bool {
        match self {
            Self::Kind(_) => true,
            Self::ValueType(ty) => store.kind_of(*ty) == KindId::TYPE,
        }
    }

    /// Adapter to existing type-member domain representation if this domain is a kind.
    pub fn to_type_member_domain(&self) -> Option<TypeMemberDomain> {
        match self {
            Self::Kind(k) => Some(TypeMemberDomain::Kind(*k)),
            Self::ValueType(_) => None,
        }
    }
}

impl From<KindId> for StaticValueDomain {
    #[inline]
    fn from(kind: KindId) -> Self {
        Self::Kind(kind)
    }
}

impl TryFrom<StaticValueDomain> for TypeMemberDomain {
    type Error = StaticValueDomain;

    fn try_from(domain: StaticValueDomain) -> Result<Self, Self::Error> {
        match domain {
            StaticValueDomain::Kind(k) => Ok(TypeMemberDomain::Kind(k)),
            other => Err(other),
        }
    }
}

/// Validates whether a [`TypeId`] is a proper value type (`kind == KindId::TYPE`).
#[inline]
pub fn validate_proper_type(store: &TypeStore, type_id: TypeId) -> Result<ProperTypeId, KindId> {
    store.proper_type(type_id)
}

/// Checks whether a [`TypeId`] is a proper value type (`kind == KindId::TYPE`).
#[inline]
pub fn is_proper_type(store: &TypeStore, type_id: TypeId) -> bool {
    store.kind_of(type_id) == KindId::TYPE
}

pub type StaticDomainResolution = TypeFormationOutcome<StaticValueDomain>;

/// Resolves an AST [`StaticDomainSyntax`] into a [`StaticValueDomain`].
pub fn resolve_static_domain(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    syntax: &StaticDomainSyntax,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> StaticDomainResolution {
    if let Some(kind) = resolve_form_as_kind(store, declarations, resolver, site, &syntax.form) {
        return TypeFormationOutcome::Ready(StaticValueDomain::Kind(kind));
    }

    let form_res = resolve_type_form(store, declarations, resolver, site, &syntax.form, diagnostics);
    match form_res {
        TypeFormResolution::Ready(ty) => {
            let kind = store.kind_of(ty);
            if matches!(store.get_kind(kind), KindData::StaticDomain(_)) {
                TypeFormationOutcome::Ready(StaticValueDomain::Kind(kind))
            } else if kind == KindId::TYPE {
                TypeFormationOutcome::Ready(StaticValueDomain::ValueType(ty))
            } else {
                TypeFormationOutcome::Invalid(TypeFormationInvalid::ExpectedProperType { actual: kind })
            }
        }
        TypeFormResolution::Dynamic => TypeFormationOutcome::Dynamic,
        TypeFormResolution::Missing(reason) => TypeFormationOutcome::Missing(reason),
        TypeFormResolution::Unresolved(reason) => TypeFormationOutcome::Unresolved(reason),
        TypeFormResolution::Invalid(reason) => TypeFormationOutcome::Invalid(reason),
        TypeFormResolution::Blocked(reason) => TypeFormationOutcome::Blocked(reason),
        TypeFormResolution::Cancelled => TypeFormationOutcome::Cancelled,
        TypeFormResolution::BudgetExceeded(report) => TypeFormationOutcome::BudgetExceeded(report),
        TypeFormResolution::InternalFailure(failure) => TypeFormationOutcome::InternalFailure(failure),
    }
}

fn resolve_form_as_kind(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    site: &TypeFormationSite,
    annotation: &TypeAnnotation,
) -> Option<KindId> {
    match &annotation.expr {
        TypeAnnotationExpr::Reference(symbol) => {
            if symbol.members.is_empty() {
                match symbol.root.as_str() {
                    "Type" => return Some(KindId::TYPE),
                    "TypePack" => return Some(KindId::TYPE_PACK),
                    "RecordFields" | "RecordRow" => return Some(KindId::RECORD_ROW),
                    "LabeledTypePack" => return Some(KindId::LABELED_TYPE_PACK),
                    "CompleteTypePack" => return Some(KindId::COMPLETE_TYPE_PACK),
                    _ => {}
                }
            }
            let members = symbol.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
            if let Some(declaration) = resolver.resolve_type_name(&site.module, &symbol.root, &members)
                && let Some(kind) = declarations.intrinsic_static_domain_kind(&declaration)
            {
                return Some(kind);
            }
            None
        }
        TypeAnnotationExpr::Callable { parameters, result, .. } => {
            let mut param_kinds = Vec::with_capacity(parameters.len());
            for item in parameters {
                match item {
                    TypeCallableItem::Parameter(param) if !param.optional && !param.rest => {
                        let pk = resolve_form_as_kind(store, declarations, resolver, site, &param.ty)?;
                        param_kinds.push(pk);
                    }
                    _ => return None,
                }
            }
            let rk = resolve_form_as_kind(store, declarations, resolver, site, result)?;
            Some(store.arrow_kind(param_kinds.into_boxed_slice(), rk))
        }
        _ => None,
    }
}
