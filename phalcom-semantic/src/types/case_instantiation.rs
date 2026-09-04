//! Branch-local opening of variant constructor binders.

use super::TypeStore;
use super::id::{RigidScopeId, RigidTypeVariableId, TypeParameterId};
use super::parameter::{GenericConstraint, TypeTerm};
use super::rigid::{LocalConstraint, LocalType, RigidArena, RigidOrigin};
use crate::enum_semantics::VariantInfo;
use crate::identity::VariantId;
use std::collections::{BTreeMap, HashMap};

/// One existential opening of one variant constructor.
///
/// The product is deliberately query-local. Its rigids may be used by branch
/// proof and payload analysis, but never become canonical `TypeId` metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseInstantiation {
    pub variant: VariantId,
    pub scope: RigidScopeId,
    pub local_rigids: BTreeMap<TypeParameterId, RigidTypeVariableId>,
    pub payload_types: Box<[LocalType]>,
    pub result_type: LocalType,
    pub constraints: Box<[LocalConstraint]>,
}

impl CaseInstantiation {
    pub fn open(store: &TypeStore, arena: &mut RigidArena, variant: &VariantInfo, parent: Option<RigidScopeId>) -> Self {
        let scope = arena.fresh_scope(parent);
        let mut local_rigids = BTreeMap::new();
        let signature = variant.constructor.as_ref().and_then(|constructor| constructor.generic_signature.as_ref());

        if let Some(signature) = signature {
            for &parameter in signature.parameters.iter() {
                let data = store.type_parameter(parameter);
                let rigid = arena.fresh(
                    scope,
                    data.kind,
                    RigidOrigin::VariantParameter {
                        variant: variant.id.clone(),
                        parameter,
                    },
                );
                local_rigids.insert(parameter, rigid);
            }
        }

        let replacements = local_rigids
            .iter()
            .map(|(&parameter, &rigid)| (parameter, LocalType::Rigid(rigid)))
            .collect::<HashMap<_, _>>();
        let payload_types = variant
            .fields
            .iter()
            .filter_map(|field| field.declared_type.canonical_type())
            .map(|ty| LocalType::from_canonical(store, ty, &replacements))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let result_type = LocalType::from_canonical(store, variant.result_type_template, &replacements);
        let constraints = signature
            .map(|signature| {
                signature
                    .constraints
                    .iter()
                    .filter_map(|constraint| localize_constraint(store, constraint, &replacements))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
            .into_boxed_slice();

        Self {
            variant: variant.id.clone(),
            scope,
            local_rigids,
            payload_types,
            result_type,
            constraints,
        }
    }

    pub fn payload_type(&self, index: usize) -> Option<&LocalType> {
        self.payload_types.get(index)
    }

    pub fn rigid_for(&self, parameter: TypeParameterId) -> Option<RigidTypeVariableId> {
        self.local_rigids.get(&parameter).copied()
    }

    pub fn is_local(&self) -> bool {
        !self.local_rigids.is_empty()
    }

    pub fn replacements(&self) -> HashMap<TypeParameterId, LocalType> {
        self.local_rigids
            .iter()
            .map(|(&parameter, &rigid)| (parameter, LocalType::Rigid(rigid)))
            .collect()
    }
}

fn localize_constraint(store: &TypeStore, constraint: &GenericConstraint, replacements: &HashMap<TypeParameterId, LocalType>) -> Option<LocalConstraint> {
    let localize = |term: &TypeTerm| match term {
        TypeTerm::Canonical(ty) => Some(LocalType::from_canonical(store, *ty, replacements)),
        // Self terms are already canonicalized by the declaration/signature
        // resolver before a variant-local constraint reaches this product.
        TypeTerm::SelfType(_) | TypeTerm::Infer(_) => None,
    };
    match constraint {
        GenericConstraint::Subtype { lower, upper } => Some(LocalConstraint::Subtype {
            lower: localize(lower)?,
            upper: localize(upper)?,
        }),
        GenericConstraint::Equivalent { left, right } => Some(LocalConstraint::Equivalent {
            left: localize(left)?,
            right: localize(right)?,
        }),
    }
}
