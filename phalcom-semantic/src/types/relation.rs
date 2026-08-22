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
pub fn is_subtype(store: &TypeStore, hierarchy: &dyn TypeHierarchy, sub: TypeId, sup: TypeId) -> bool {
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
        (TypeData::Union(members), _) => members.iter().all(|&m| is_subtype(store, hierarchy, m, sup)),

        // Union on the right: T <: A | B if exists m in members where T <: m
        (_, TypeData::Union(members)) => members.iter().any(|&m| is_subtype(store, hierarchy, sub, m)),

        // Nominal subtyping
        (TypeData::Nominal { declaration: sub_decl }, TypeData::Nominal { declaration: sup_decl }) => hierarchy.is_subclass(sub_decl, sup_decl),

        // Generic applied subtyping
        (
            TypeData::Applied {
                origin: sub_orig,
                arguments: sub_args,
            },
            TypeData::Applied {
                origin: sup_orig,
                arguments: sup_args,
            },
        ) => {
            if is_subtype(store, hierarchy, *sub_orig, *sup_orig) && sub_args.len() == sup_args.len() {
                sub_args.iter().zip(sup_args.iter()).all(|(&a, &b)| is_subtype(store, hierarchy, a, b))
            } else {
                false
            }
        }

        // Tuple subtyping (width and depth subtyping)
        (TypeData::Tuple(sub_elems), TypeData::Tuple(sup_elems)) => {
            if sub_elems.len() == sup_elems.len() {
                sub_elems
                    .iter()
                    .zip(sup_elems.iter())
                    .all(|(a, b)| a.label == b.label && is_subtype(store, hierarchy, a.ty, b.ty))
            } else {
                false
            }
        }

        // Record subtyping (width and depth subtyping: sub has all fields of sup)
        (TypeData::Record(sub_fields), TypeData::Record(sup_fields)) => sup_fields.iter().all(|sup_f| {
            sub_fields
                .iter()
                .any(|sub_f| sub_f.name == sup_f.name && is_subtype(store, hierarchy, sub_f.ty, sup_f.ty))
        }),

        // Callable subtyping: contravariant parameter types, covariant return type
        (TypeData::Callable(sub_call), TypeData::Callable(sup_call)) => {
            if sub_call.parameters.len() == sup_call.parameters.len() {
                let params_ok = sub_call
                    .parameters
                    .iter()
                    .zip(sup_call.parameters.iter())
                    .all(|(sub_p, sup_p)| sub_p.label == sup_p.label && sub_p.rest == sup_p.rest && is_subtype(store, hierarchy, sup_p.ty, sub_p.ty));
                params_ok && is_subtype(store, hierarchy, sub_call.return_type, sup_call.return_type)
            } else {
                false
            }
        }

        // Unit is a distinct nominal-like unit type
        (TypeData::Unit, TypeData::Unit) => true,

        // Infer variables match reflexively (handled at start of function)
        (TypeData::Infer(a), TypeData::Infer(b)) => a == b,

        _ => false,
    }
}

/// Checks assignability from an expression's type knowledge to an expected type knowledge.
pub fn check_assignability(store: &TypeStore, hierarchy: &dyn TypeHierarchy, actual: &TypeKnowledge, expected: &TypeKnowledge) -> Assignability {
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
        (TypeKnowledge::Unknown(_), _) | (_, TypeKnowledge::Unknown(_)) => Assignability::Uncertain,
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
