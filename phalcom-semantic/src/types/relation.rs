//! Canonical Subtype and Assignability Relations.

use super::evidence::TypeKnowledge;
use super::id::TypeId;
use super::store::{TypeData, TypeStore};
use crate::identity::DeclarationId;

/// Environment for querying class hierarchies and module declaration relations.
pub trait TypeHierarchy {
    /// Returns whether `sub` is identical to or inherits from `sup`.
    fn is_subclass(&self, sub: &DeclarationId, sup: &DeclarationId) -> bool;
}

/// A simple hierarchy based on direct parent maps.
#[derive(Clone, Debug, Default)]
pub struct MapTypeHierarchy {
    pub superclasses: std::collections::HashMap<DeclarationId, DeclarationId>,
}

impl MapTypeHierarchy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, class: DeclarationId, superclass: DeclarationId) {
        self.superclasses.insert(class, superclass);
    }
}

impl TypeHierarchy for MapTypeHierarchy {
    fn is_subclass(&self, sub: &DeclarationId, sup: &DeclarationId) -> bool {
        if sub == sup {
            return true;
        }
        let mut curr = sub;
        while let Some(parent) = self.superclasses.get(curr) {
            if parent == sup {
                return true;
            }
            curr = parent;
        }
        false
    }
}

/// The result of checking whether `actual` is assignable to `expected`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Assignability {
    /// Soundly proven to satisfy the relation.
    Assignable,
    /// Soundly refuted: provable type mismatch / contradiction.
    Refuted {
        actual: TypeId,
        expected: TypeId,
        reason: RefutationReason,
    },
    /// Epistemically uncertain (e.g. involving Unknown or dynamic unproven paths).
    Uncertain,
}

impl Assignability {
    #[inline]
    pub fn is_assignable(&self) -> bool {
        matches!(self, Self::Assignable)
    }

    #[inline]
    pub fn is_refuted(&self) -> bool {
        matches!(self, Self::Refuted { .. })
    }

    #[inline]
    pub fn is_uncertain(&self) -> bool {
        matches!(self, Self::Uncertain)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefutationReason {
    IncompatibleNominal,
    TypeMismatch,
    UnionMemberMismatch,
}

/// Check whether `sub` is a canonical subtype of `sup` (`sub <: sup`).
pub fn is_subtype(
    store: &TypeStore,
    hierarchy: &dyn TypeHierarchy,
    sub: TypeId,
    sup: TypeId,
) -> bool {
    // 1. Reflexivity
    if sub == sup {
        return true;
    }

    // 2. Never is bottom (Never <: T for any T)
    if sub == store.never() {
        return true;
    }

    let sub_data = store.get(sub);
    let sup_data = store.get(sup);

    match (sub_data, sup_data) {
        // Union on the left: A | B <: T iff A <: T and B <: T
        (TypeData::Union(members), _) => {
            members.iter().all(|&m| is_subtype(store, hierarchy, m, sup))
        }

        // Union on the right: T <: A | B if exists m in members where T <: m
        (_, TypeData::Union(members)) => {
            members.iter().any(|&m| is_subtype(store, hierarchy, sub, m))
        }

        // Nominal subtyping
        (
            TypeData::Nominal { declaration: sub_decl },
            TypeData::Nominal { declaration: sup_decl },
        ) => hierarchy.is_subclass(sub_decl, sup_decl),

        // Unit is a distinct nominal-like unit type
        (TypeData::Unit, TypeData::Unit) => true,

        _ => false,
    }
}

/// Checks assignability from an expression's type knowledge to an expected type knowledge.
pub fn check_assignability(
    store: &TypeStore,
    hierarchy: &dyn TypeHierarchy,
    actual: &TypeKnowledge,
    expected: &TypeKnowledge,
) -> Assignability {
    // Dynamic allows any assignment without static contradiction
    if actual.is_dynamic() || expected.is_dynamic() {
        return Assignability::Assignable;
    }

    match (actual, expected) {
        (TypeKnowledge::Known(act_ev), TypeKnowledge::Known(exp_ev)) => {
            if is_subtype(store, hierarchy, act_ev.ty, exp_ev.ty) {
                Assignability::Assignable
            } else if act_ev.authority.is_sound_for_rejection() && exp_ev.authority.is_sound_for_rejection() {
                Assignability::Refuted {
                    actual: act_ev.ty,
                    expected: exp_ev.ty,
                    reason: RefutationReason::TypeMismatch,
                }
            } else {
                Assignability::Uncertain
            }
        }
        (TypeKnowledge::Unknown(_), _) | (_, TypeKnowledge::Unknown(_)) => {
            Assignability::Uncertain
        }
        _ => Assignability::Assignable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::evidence::EvidenceAuthority;
    use phalcom_modules::identity::ModuleId;

    fn test_decl(name: &str) -> DeclarationId {
        let module = ModuleId::core();
        DeclarationId::new(module, name.into())
    }

    #[test]
    fn subtype_relations_hold() {
        let mut store = TypeStore::new();
        let mut hier = MapTypeHierarchy::new();

        let object = test_decl("Object");
        let number = test_decl("Number");
        let int = test_decl("Int");
        let string = test_decl("String");

        hier.insert(number.clone(), object.clone());
        hier.insert(int.clone(), number.clone());
        hier.insert(string.clone(), object.clone());

        let t_obj = store.nominal(object);
        let t_num = store.nominal(number);
        let t_int = store.nominal(int);
        let t_str = store.nominal(string);

        // Int <: Int
        assert!(is_subtype(&store, &hier, t_int, t_int));
        // Never <: Int
        assert!(is_subtype(&store, &hier, store.never(), t_int));
        // Int <: Number
        assert!(is_subtype(&store, &hier, t_int, t_num));
        // Int <: Object
        assert!(is_subtype(&store, &hier, t_int, t_obj));
        // String !<: Int
        assert!(!is_subtype(&store, &hier, t_str, t_int));

        // Int <: (Int | String)
        let int_or_str = store.union(&[t_int, t_str]);
        assert!(is_subtype(&store, &hier, t_int, int_or_str));
        // (Int | String) <: Object
        assert!(is_subtype(&store, &hier, int_or_str, t_obj));
        // (Int | String) !<: Number
        assert!(!is_subtype(&store, &hier, int_or_str, t_num));
    }

    #[test]
    fn assignability_rejects_sound_contradiction() {
        let mut store = TypeStore::new();
        let hier = MapTypeHierarchy::new();

        let t_int = store.nominal(test_decl("Int"));
        let t_str = store.nominal(test_decl("String"));

        let actual = TypeKnowledge::known(t_int, EvidenceAuthority::ExactSyntax);
        let expected = TypeKnowledge::known(t_str, EvidenceAuthority::Declared);

        let res = check_assignability(&store, &hier, &actual, &expected);
        assert!(res.is_refuted());
    }
}
