//! Bidirectional checking type expectations (Spec 04.5).

use crate::checker::inference::InferenceTerm;
use crate::identity::DeclarationId;
use crate::types::id::TypeId;

/// Why a checker expectation is being propagated downward. This is
/// contextual control information, never value evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExpectationOrigin {
    DeclarationContract,
    ReturnContract,
    CallableSignature,
    AssignmentContract,
    ContextualBlockParameter,
    GenericArgument,
    GenericResult,
    CollectionElement,
    ProductComponent,
    ExplicitCheck,
}

/// Contextual type expectation propagated downward into an expression during bidirectional checking.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ExpectedType {
    /// No contextual expectation (synthesis mode).
    #[default]
    None,
    /// A known canonical proper type expectation (checking mode).
    Proper { ty: TypeId, origin: ExpectationOrigin },
    /// A solver-local inference term expectation.
    Inference {
        context: crate::checker::inference::InferenceContextId,
        term: InferenceTerm,
        origin: ExpectationOrigin,
    },
}

impl ExpectedType {
    pub fn none() -> Self {
        Self::None
    }

    pub fn proper(ty: TypeId) -> Self {
        Self::Proper {
            ty,
            origin: ExpectationOrigin::ExplicitCheck,
        }
    }

    pub fn proper_from(ty: TypeId, origin: ExpectationOrigin) -> Self {
        Self::Proper { ty, origin }
    }

    pub(crate) fn inference_from(context: crate::checker::inference::InferenceContextId, term: InferenceTerm, origin: ExpectationOrigin) -> Self {
        Self::Inference { context, term, origin }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn ty(&self) -> Option<TypeId> {
        match self {
            Self::Proper { ty, .. } => Some(*ty),
            Self::Inference {
                term: InferenceTerm::Canonical(ty),
                ..
            } => Some(*ty),
            _ => None,
        }
    }

    pub fn origin(&self) -> Option<ExpectationOrigin> {
        match self {
            Self::None => None,
            Self::Proper { origin, .. } | Self::Inference { origin, .. } => Some(*origin),
        }
    }

    /// Returns element expectation only for the canonical collection named by
    /// `collection_decl`. An arbitrary applied user type is not evidence that
    /// its first argument is an element type.
    pub fn collection_element_type(&self, store: &crate::types::store::TypeStore, collection_decl: &DeclarationId) -> ExpectedType {
        if let Some(ty) = self.ty() {
            if let Some(arguments) = canonical_collection_arguments(store, ty, collection_decl, 1) {
                if let Some(&first) = arguments.first() {
                    return ExpectedType::proper_from(first, ExpectationOrigin::CollectionElement);
                }
            }
        } else if let Self::Inference {
            context,
            term: InferenceTerm::Applied { arguments, .. },
            ..
        } = self
        {
            if arguments.len() == 1 && inference_origin_matches(store, self, collection_decl) {
                if let Some(first) = arguments.first() {
                    return ExpectedType::inference_from(*context, first.clone(), ExpectationOrigin::CollectionElement);
                }
            }
        }
        ExpectedType::None
    }

    /// Returns map key/value expectations only for canonical `Map<K, V>`.
    pub fn map_key_val_types(&self, store: &crate::types::store::TypeStore, map_decl: &DeclarationId) -> (ExpectedType, ExpectedType) {
        if let Some(ty) = self.ty() {
            if let Some(arguments) = canonical_collection_arguments(store, ty, map_decl, 2) {
                if arguments.len() >= 2 {
                    return (
                        ExpectedType::proper_from(arguments[0], ExpectationOrigin::ProductComponent),
                        ExpectedType::proper_from(arguments[1], ExpectationOrigin::ProductComponent),
                    );
                }
            }
        } else if let Self::Inference {
            context,
            term: InferenceTerm::Applied { arguments, .. },
            ..
        } = self
        {
            if arguments.len() == 2 && inference_origin_matches(store, self, map_decl) {
                return (
                    ExpectedType::inference_from(*context, arguments[0].clone(), ExpectationOrigin::ProductComponent),
                    ExpectedType::inference_from(*context, arguments[1].clone(), ExpectationOrigin::ProductComponent),
                );
            }
        }
        (ExpectedType::None, ExpectedType::None)
    }

    pub fn callable_signature(&self, store: &crate::types::store::TypeStore) -> Option<(Vec<ExpectedType>, ExpectedType)> {
        if let Some(ty) = self.ty() {
            if let crate::types::store::TypeData::Callable(c) = store.get(ty) {
                let params = c
                    .parameters
                    .iter()
                    .map(|p| ExpectedType::proper_from(p.ty, ExpectationOrigin::CallableSignature))
                    .collect();
                let ret = ExpectedType::proper_from(c.return_type, ExpectationOrigin::CallableSignature);
                return Some((params, ret));
            }
        } else if let Self::Inference {
            context,
            term: InferenceTerm::Callable(c),
            ..
        } = self
        {
            let params = c
                .parameters
                .iter()
                .map(|p| ExpectedType::inference_from(*context, p.term.clone(), ExpectationOrigin::CallableSignature))
                .collect();
            let ret = ExpectedType::inference_from(*context, (*c.return_type).clone(), ExpectationOrigin::CallableSignature);
            return Some((params, ret));
        }
        None
    }

    pub fn contextual_knowledge(&self, ty: TypeId) -> Option<crate::types::evidence::TypeKnowledge> {
        match self {
            Self::Proper { ty: expected_ty, origin } => {
                if *expected_ty == ty {
                    let status = match origin {
                        ExpectationOrigin::DeclarationContract
                        | ExpectationOrigin::ReturnContract
                        | ExpectationOrigin::AssignmentContract
                        | ExpectationOrigin::ContextualBlockParameter => crate::types::evidence::EvidenceStatus::Assumed,
                        ExpectationOrigin::ExplicitCheck => crate::types::evidence::EvidenceStatus::Established,
                        _ => crate::types::evidence::EvidenceStatus::Assumed,
                    };
                    Some(match status {
                        crate::types::evidence::EvidenceStatus::Established => {
                            crate::types::evidence::TypeKnowledge::established(ty, crate::types::evidence::EvidenceOrigin::ContextualDerivation)
                        }
                        crate::types::evidence::EvidenceStatus::Assumed => {
                            crate::types::evidence::TypeKnowledge::assumed(ty, crate::types::evidence::EvidenceOrigin::ContextualDerivation)
                        }
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

fn canonical_collection_arguments<'a>(
    store: &'a crate::types::store::TypeStore,
    ty: TypeId,
    collection_decl: &DeclarationId,
    arity: usize,
) -> Option<&'a [TypeId]> {
    let (declaration, _) = store.applied_nominal_parts(ty)?;
    if &declaration != collection_decl {
        return None;
    }
    match store.get(ty) {
        crate::types::store::TypeData::Applied { arguments, .. } if arguments.len() == arity => Some(arguments),
        _ => None,
    }
}

fn inference_origin_matches(store: &crate::types::store::TypeStore, expected: &ExpectedType, collection_decl: &DeclarationId) -> bool {
    let ExpectedType::Inference {
        term: InferenceTerm::Applied { origin, .. },
        ..
    } = expected
    else {
        return false;
    };
    fn matches(store: &crate::types::store::TypeStore, origin: &InferenceTerm, collection_decl: &DeclarationId) -> bool {
        match origin {
            InferenceTerm::Canonical(ty) => store.nominal_origin_declaration(*ty) == Some(collection_decl),
            InferenceTerm::Applied { origin, .. } => matches(store, origin, collection_decl),
            _ => false,
        }
    }
    matches(store, origin, collection_decl)
}
