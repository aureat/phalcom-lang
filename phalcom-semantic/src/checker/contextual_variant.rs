//! Context-provided algebraic owner and case discovery shared by expressions
//! and patterns. Candidate identity retains the specialization, never just its
//! nominal declaration. This module does not apply constructors or open fields.

use super::{context::CheckingContext, expected::ExpectedType, inference::InferenceTerm};
use crate::{
    identity::{DeclarationId, VariantId},
    types::{
        id::TypeId,
        store::{TypeData, TypeStore},
    },
};
use std::collections::BTreeSet;

/// One independently established owner specialization and its checking context.
#[derive(Clone, Debug)]
pub(crate) struct ContextualOwner {
    pub owner: DeclarationId,
    pub owner_type: TypeId,
    pub expected: ExpectedType,
}

/// An exact declaration-backed case within one owner specialization.
#[derive(Clone, Debug)]
pub(crate) struct ContextualVariantCandidate {
    pub owner: ContextualOwner,
    pub variant: VariantId,
}

pub(crate) fn nominal_owner_types(store: &TypeStore, ty: TypeId) -> Vec<(DeclarationId, TypeId)> {
    fn collect(store: &TypeStore, ty: TypeId, seen: &mut BTreeSet<TypeId>, owners: &mut Vec<(DeclarationId, TypeId)>) {
        if !seen.insert(ty) {
            return;
        }
        match store.get(ty) {
            TypeData::Union(members) => {
                for member in members.iter().copied() {
                    collect(store, member, seen, owners);
                }
            }
            _ => {
                if let Some(owner) = store.nominal_origin_declaration(ty).cloned() {
                    if !owners.iter().any(|(known, known_ty)| known == &owner && known_ty == &ty) {
                        owners.push((owner, ty));
                    }
                }
            }
        }
    }

    let mut owners = Vec::new();
    collect(store, ty, &mut BTreeSet::new(), &mut owners);
    owners
}

/// Enumerates owner branches without materializing away symbolic arguments.
pub(crate) fn expected_owners(ctx: &mut CheckingContext<'_>, expected: &ExpectedType) -> Vec<ContextualOwner> {
    fn collect(ctx: &mut CheckingContext<'_>, expected: &ExpectedType, out: &mut Vec<ContextualOwner>) {
        if let ExpectedType::Inference {
            context,
            term: InferenceTerm::Union(members),
            origin,
        } = expected
        {
            for term in members.iter() {
                collect(ctx, &ExpectedType::inference_from(*context, term.clone(), *origin), out);
            }
            return;
        }
        if let Some(ty) = expected.ty() {
            for (owner, owner_type) in nominal_owner_types(ctx.store, ty) {
                let origin = expected.origin().unwrap_or(super::expected::ExpectationOrigin::ExplicitCheck);
                let branch = match expected {
                    ExpectedType::Inference { context, .. } => ExpectedType::inference_from(*context, InferenceTerm::Canonical(owner_type), origin),
                    _ => ExpectedType::proper_from(owner_type, origin),
                };
                out.push(ContextualOwner {
                    owner,
                    owner_type,
                    expected: branch,
                });
            }
            return;
        }
        let ExpectedType::Inference {
            context,
            term,
            origin: expectation_origin,
        } = expected
        else {
            return;
        };
        let materialized = ctx
            .inference_session(*context)
            .and_then(|session| session.borrow().materialize(term, ctx.store).ok());
        if let Some(ty) = materialized {
            for (owner, owner_type) in nominal_owner_types(ctx.store, ty) {
                out.push(ContextualOwner {
                    owner,
                    owner_type,
                    expected: ExpectedType::inference_from(*context, InferenceTerm::Canonical(owner_type), *expectation_origin),
                });
            }
            return;
        }
        fn origin(term: &InferenceTerm) -> Option<TypeId> {
            match term {
                InferenceTerm::Canonical(ty) => Some(*ty),
                InferenceTerm::Applied { origin: base, .. } => origin(base),
                InferenceTerm::ExactCase { enum_type, .. } => origin(enum_type),
                _ => None,
            }
        }
        if let Some(ty) = origin(term) {
            for (owner, owner_type) in nominal_owner_types(ctx.store, ty) {
                out.push(ContextualOwner {
                    owner,
                    owner_type,
                    expected: expected.clone(),
                });
            }
        }
    }
    let mut owners = Vec::new();
    collect(ctx, expected, &mut owners);
    owners
}

/// Discovers exact cases only inside contextual owners. Canonical GADT
/// contradictions exclude candidates; missing or blocked proof retains them.
/// Every eligible enum owner is a dependency, including failed lookups.
pub(crate) fn discover(
    ctx: &mut CheckingContext<'_>,
    owners: &[ContextualOwner],
    matches: impl Fn(&crate::enum_semantics::VariantInfo) -> bool,
) -> (Vec<ContextualVariantCandidate>, bool) {
    let Some(table) = ctx.enum_table else {
        return (Vec::new(), false);
    };
    let mut candidates = Vec::new();
    let mut refuted = false;
    for owner in owners {
        let Some(info) = table.enums.get(&owner.owner) else {
            continue;
        };
        ctx.record_semantic_dependency(super::analysis::SemanticDependency::EnumDeclaration(owner.owner.clone()));
        for variant in &info.variants {
            let Some(info) = table.variants.get(variant) else {
                continue;
            };
            if !matches(info) {
                continue;
            }
            if super::associated::contextual_variant_is_refuted(ctx, variant, owner.owner_type) {
                refuted = true;
            } else {
                candidates.push(ContextualVariantCandidate {
                    owner: owner.clone(),
                    variant: variant.clone(),
                });
            }
        }
    }
    (candidates, refuted)
}
