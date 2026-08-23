use super::environment::TypeView;
use super::evidence::TypeKnowledge;
use super::id::TypeId;
use super::outcome::{BlockReason, BudgetReport, CancellationToken, DynamicBoundaryObligation, QueryBudget, RelationFailure, RelationOutcome};
use super::store::{TypeData, TypeStore};
use super::variance::Variance;
use crate::declarations::GenericSupertypeTemplate;
use crate::identity::DeclarationId;
use std::collections::HashSet;

/// Environment for querying class hierarchies and module declaration relations.
pub trait TypeHierarchy {
    /// Returns the immediate superclass of a class declaration, if any.
    fn superclass(&self, declaration: &DeclarationId) -> Option<&DeclarationId>;

    /// Returns whether `sub` is identical to or inherits from `sup`.
    fn is_subclass(&self, sub: &DeclarationId, sup: &DeclarationId) -> bool;

    /// Returns the generic supertype template for a class, if registered.
    fn supertype_template(&self, declaration: &DeclarationId) -> Option<&GenericSupertypeTemplate> {
        None
    }
}

/// A simple hierarchy based on direct parent maps and supertype templates.
#[derive(Clone, Debug, Default)]
pub struct MapTypeHierarchy {
    pub superclasses: std::collections::HashMap<DeclarationId, DeclarationId>,
    pub templates: std::collections::HashMap<DeclarationId, GenericSupertypeTemplate>,
}

impl MapTypeHierarchy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, class: DeclarationId, superclass: DeclarationId) {
        self.superclasses.insert(class, superclass);
    }

    pub fn insert_template(&mut self, template: GenericSupertypeTemplate) {
        self.superclasses.insert(
            template.declaration.clone(),
            DeclarationId::new(phalcom_modules::identity::ModuleId::core(), "generic_super".into()),
        );
        self.templates.insert(template.declaration.clone(), template);
    }
}

impl TypeHierarchy for MapTypeHierarchy {
    fn superclass(&self, declaration: &DeclarationId) -> Option<&DeclarationId> {
        self.superclasses.get(declaration)
    }

    fn supertype_template(&self, declaration: &DeclarationId) -> Option<&GenericSupertypeTemplate> {
        self.templates.get(declaration)
    }

    fn is_subclass(&self, sub: &DeclarationId, sup: &DeclarationId) -> bool {
        if sub == sup {
            return true;
        }
        let mut curr = sub;
        let mut visited = HashSet::new();
        visited.insert(curr);
        while let Some(parent) = self.superclasses.get(curr) {
            if parent == sup {
                return true;
            }
            if !visited.insert(parent) {
                // Inheritance cycle detected
                return false;
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
    /// Dynamic boundary requiring runtime check/coercion.
    DynamicBoundary,
    /// Query is blocked awaiting resolution or cycle breaking.
    Blocked(BlockReason),
    /// Request was cancelled.
    Cancelled,
    /// Query budget was exceeded.
    BudgetExceeded(BudgetReport),
    /// Internal failure occurred.
    InternalFailure(String),
    /// Epistemically uncertain.
    Uncertain,
}

impl Assignability {
    #[inline]
    pub fn is_assignable(&self) -> bool {
        matches!(self, Self::Assignable | Self::DynamicBoundary)
    }

    #[inline]
    pub fn is_refuted(&self) -> bool {
        matches!(self, Self::Refuted { .. })
    }

    #[inline]
    pub fn is_uncertain(&self) -> bool {
        matches!(self, Self::Uncertain | Self::Blocked(_) | Self::Cancelled | Self::BudgetExceeded(_))
    }

    #[inline]
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }

    #[inline]
    pub fn is_budget_exceeded(&self) -> bool {
        matches!(self, Self::BudgetExceeded(_))
    }
}

impl From<RelationOutcome> for Assignability {
    fn from(outcome: RelationOutcome) -> Self {
        match outcome {
            RelationOutcome::Proven { .. } => Self::Assignable,
            RelationOutcome::Refuted(failure) => match failure {
                RelationFailure::IncompatibleNominal { actual: _, expected: _ } => Self::Refuted {
                    actual: TypeId::DUMMY,
                    expected: TypeId::DUMMY,
                    reason: RefutationReason::IncompatibleNominal,
                },
                RelationFailure::UnionMemberMismatch { actual, expected } => Self::Refuted {
                    actual,
                    expected,
                    reason: RefutationReason::UnionMemberMismatch,
                },
                _ => Self::Refuted {
                    actual: TypeId::DUMMY,
                    expected: TypeId::DUMMY,
                    reason: RefutationReason::TypeMismatch,
                },
            },
            RelationOutcome::DynamicBoundary(_) => Self::DynamicBoundary,
            RelationOutcome::Blocked(reason) => Self::Blocked(reason),
            RelationOutcome::Cancelled => Self::Cancelled,
            RelationOutcome::BudgetExceeded(report) => Self::BudgetExceeded(report),
            RelationOutcome::InternalFailure(msg) => Self::InternalFailure(msg),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefutationReason {
    IncompatibleNominal,
    TypeMismatch,
    UnionMemberMismatch,
}

/// Evaluates subtyping with explicit budgets, cancellation, and cycle detection.
pub fn check_subtype_bounded(
    store: &TypeStore,
    hierarchy: &dyn TypeHierarchy,
    sub: TypeId,
    sup: TypeId,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
) -> RelationOutcome {
    if cancellation.is_cancelled() {
        return RelationOutcome::Cancelled;
    }
    if let Err(report) = budget.charge_step() {
        return RelationOutcome::BudgetExceeded(report);
    }
    if let Err(report) = budget.charge_pair() {
        return RelationOutcome::BudgetExceeded(report);
    }

    let mut visited = HashSet::new();
    check_subtype_impl(store, hierarchy, sub, sup, budget, cancellation, &mut visited)
}

fn check_subtype_impl(
    store: &TypeStore,
    hierarchy: &dyn TypeHierarchy,
    sub: TypeId,
    sup: TypeId,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
    visited: &mut HashSet<(TypeId, TypeId)>,
) -> RelationOutcome {
    if cancellation.is_cancelled() {
        return RelationOutcome::Cancelled;
    }
    if let Err(report) = budget.charge_step() {
        return RelationOutcome::BudgetExceeded(report);
    }

    if sub == sup || sub == store.never() {
        return RelationOutcome::proven(());
    }

    if !visited.insert((sub, sup)) {
        return RelationOutcome::Refuted(RelationFailure::CycleDetected { sub, sup });
    }

    let sub_data = store.get(sub);
    let sup_data = store.get(sup);

    let res = match (sub_data, sup_data) {
        (TypeData::Union(members), _) => {
            let mut all_ok = true;
            for &m in members.iter() {
                let outcome = check_subtype_impl(store, hierarchy, m, sup, budget, cancellation, visited);
                if !outcome.is_proven() {
                    all_ok = false;
                    break;
                }
            }
            if all_ok {
                RelationOutcome::proven(())
            } else {
                RelationOutcome::Refuted(RelationFailure::UnionMemberMismatch { actual: sub, expected: sup })
            }
        }
        (_, TypeData::Union(members)) => {
            let mut any_ok = false;
            for &m in members.iter() {
                let outcome = check_subtype_impl(store, hierarchy, sub, m, budget, cancellation, visited);
                if outcome.is_proven() {
                    any_ok = true;
                    break;
                }
            }
            if any_ok {
                RelationOutcome::proven(())
            } else {
                RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
            }
        }
        (TypeData::ClassObject { declaration: sub_decl }, TypeData::ClassObject { declaration: sup_decl }) => {
            if hierarchy.is_subclass(sub_decl, sup_decl) {
                RelationOutcome::proven(())
            } else {
                RelationOutcome::Refuted(RelationFailure::IncompatibleNominal {
                    actual: sub_decl.clone(),
                    expected: sup_decl.clone(),
                })
            }
        }
        (TypeData::Nominal { declaration: sub_decl }, TypeData::Nominal { declaration: sup_decl }) => {
            if hierarchy.is_subclass(sub_decl, sup_decl) {
                RelationOutcome::proven(())
            } else {
                RelationOutcome::Refuted(RelationFailure::IncompatibleNominal {
                    actual: sub_decl.clone(),
                    expected: sup_decl.clone(),
                })
            }
        }
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
            if sub_orig == sup_orig {
                // If origins match, check arguments against declaration variance
                if sub_args.len() == sup_args.len() {
                    let origin_decl = match store.get(*sub_orig) {
                        TypeData::Nominal { declaration } => Some(declaration),
                        _ => None,
                    };

                    let mut all_args_ok = true;
                    for (idx, (&a_sub, &a_sup)) in sub_args.iter().zip(sup_args.iter()).enumerate() {
                        let variance = origin_decl
                            .and_then(|decl| store.get_parameter_variance(decl, idx as u32))
                            .unwrap_or(Variance::Invariant);

                        let arg_outcome = match variance {
                            Variance::Covariant => {
                                // Covariant: a_sub <: a_sup
                                check_subtype_impl(store, hierarchy, a_sub, a_sup, budget, cancellation, visited)
                            }
                            Variance::Contravariant => {
                                // Contravariant: a_sup <: a_sub
                                check_subtype_impl(store, hierarchy, a_sup, a_sub, budget, cancellation, visited)
                            }
                            Variance::Invariant => {
                                // Invariant: a_sub == a_sup
                                if a_sub == a_sup {
                                    RelationOutcome::proven(())
                                } else {
                                    RelationOutcome::Refuted(RelationFailure::TypeMismatch {
                                        actual: a_sub,
                                        expected: a_sup,
                                    })
                                }
                            }
                        };

                        if !arg_outcome.is_proven() {
                            all_args_ok = false;
                            break;
                        }
                    }

                    if all_args_ok {
                        RelationOutcome::proven(())
                    } else {
                        RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
                    }
                } else {
                    RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
                }
            } else {
                // Check generic supertype template if sub origin has one
                if let TypeData::Nominal { declaration: sub_decl } = store.get(*sub_orig) {
                    if let Some(template) = hierarchy.supertype_template(sub_decl) {
                        let mut env = super::environment::TypeEnvironment::new();
                        // Search existing TypeParameterIds in store for this declaration
                        for (idx, &arg) in sub_args.iter().enumerate() {
                            if let Some(param_id) =
                                store.find_type_parameter_id(&super::parameter::TypeParameterOwner::Declaration(sub_decl.clone()), idx as u32)
                            {
                                env.bind_param(param_id, arg);
                            }
                        }
                        let specialized_super = TypeView::new(template.supertype, env).materialize(&mut store.clone());
                        check_subtype_impl(store, hierarchy, specialized_super, sup, budget, cancellation, visited)
                    } else {
                        RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
                    }
                } else {
                    RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
                }
            }
        }

        (TypeData::Tuple(sub_elems), TypeData::Tuple(sup_elems)) => {
            if sub_elems.len() == sup_elems.len() {
                let mut all_ok = true;
                for (a, b) in sub_elems.iter().zip(sup_elems.iter()) {
                    if a.label != b.label {
                        all_ok = false;
                        break;
                    }
                    let out = check_subtype_impl(store, hierarchy, a.ty, b.ty, budget, cancellation, visited);
                    if !out.is_proven() {
                        all_ok = false;
                        break;
                    }
                }
                if all_ok {
                    RelationOutcome::proven(())
                } else {
                    RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
                }
            } else {
                RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
            }
        }
        (TypeData::Record(sub_fields), TypeData::Record(sup_fields)) => {
            let mut all_ok = true;
            for sup_f in sup_fields.iter() {
                let matching = sub_fields.iter().find(|sub_f| sub_f.name == sup_f.name);
                if let Some(sub_f) = matching {
                    let out = check_subtype_impl(store, hierarchy, sub_f.ty, sup_f.ty, budget, cancellation, visited);
                    if !out.is_proven() {
                        all_ok = false;
                        break;
                    }
                } else {
                    all_ok = false;
                    break;
                }
            }
            if all_ok {
                RelationOutcome::proven(())
            } else {
                RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
            }
        }
        (TypeData::Callable(sub_call), TypeData::Callable(sup_call)) => {
            if sub_call.parameters.len() == sup_call.parameters.len() {
                let mut params_ok = true;
                for (sub_p, sup_p) in sub_call.parameters.iter().zip(sup_call.parameters.iter()) {
                    if sub_p.label != sup_p.label || sub_p.rest != sup_p.rest {
                        params_ok = false;
                        break;
                    }
                    // Contravariant parameters: sup_p.ty <: sub_p.ty
                    let out = check_subtype_impl(store, hierarchy, sup_p.ty, sub_p.ty, budget, cancellation, visited);
                    if !out.is_proven() {
                        params_ok = false;
                        break;
                    }
                }
                if params_ok {
                    // Covariant return type: sub_call.return_type <: sup_call.return_type
                    check_subtype_impl(store, hierarchy, sub_call.return_type, sup_call.return_type, budget, cancellation, visited)
                } else {
                    RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
                }
            } else {
                RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
            }
        }
        (TypeData::Callable(_), TypeData::Nominal { declaration: sup_decl }) => {
            if sup_decl.name.as_ref() == "Function" || sup_decl.name.as_ref() == "Closure" || sup_decl.name.as_ref() == "Object" {
                RelationOutcome::proven(())
            } else {
                RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
            }
        }
        (_, TypeData::Nominal { declaration: sup_decl }) if sup_decl.name.as_ref() == "Object" => RelationOutcome::proven(()),
        (TypeData::Unit, TypeData::Unit) => RelationOutcome::proven(()),
        _ => RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup }),
    };

    visited.remove(&(sub, sup));
    res
}

/// Check whether `sub` is a canonical subtype of `sup` (`sub <: sup`).
pub fn is_subtype(store: &TypeStore, hierarchy: &dyn TypeHierarchy, sub: TypeId, sup: TypeId) -> bool {
    let mut budget = QueryBudget::default();
    let cancellation = CancellationToken::new();
    check_subtype_bounded(store, hierarchy, sub, sup, &mut budget, &cancellation).is_proven()
}

/// Evaluates assignability with bounded queries, cancellation, and cycle tracking.
pub fn check_assignability_bounded(
    store: &TypeStore,
    hierarchy: &dyn TypeHierarchy,
    actual: &TypeKnowledge,
    expected: &TypeKnowledge,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
) -> RelationOutcome {
    if cancellation.is_cancelled() {
        return RelationOutcome::Cancelled;
    }

    if actual.is_dynamic() || expected.is_dynamic() {
        return RelationOutcome::DynamicBoundary(DynamicBoundaryObligation {
            reason: "dynamic boundary".into(),
        });
    }

    match (actual, expected) {
        (TypeKnowledge::Known(act_ev), TypeKnowledge::Known(exp_ev)) => {
            if matches!(store.get(act_ev.ty), TypeData::Parameter(_)) || matches!(store.get(exp_ev.ty), TypeData::Parameter(_)) {
                return RelationOutcome::Blocked(BlockReason::RecursiveFixpoint);
            }

            let sub_res = check_subtype_bounded(store, hierarchy, act_ev.ty, exp_ev.ty, budget, cancellation);
            if sub_res.is_proven() {
                RelationOutcome::proven(())
            } else if sub_res.is_refuted() && act_ev.authority.is_sound_for_rejection() && exp_ev.authority.is_sound_for_rejection() {
                sub_res
            } else if sub_res.is_cancelled() || sub_res.is_budget_exceeded() {
                sub_res
            } else {
                RelationOutcome::Blocked(BlockReason::RecursiveFixpoint)
            }
        }
        (TypeKnowledge::Unknown(reason), _) | (_, TypeKnowledge::Unknown(reason)) => RelationOutcome::Blocked(BlockReason::UnknownType(reason.clone())),
        _ => RelationOutcome::DynamicBoundary(DynamicBoundaryObligation {
            reason: "dynamic boundary".into(),
        }),
    }
}

/// Checks assignability from an expression's type knowledge to an expected type knowledge.
pub fn check_assignability(store: &TypeStore, hierarchy: &dyn TypeHierarchy, actual: &TypeKnowledge, expected: &TypeKnowledge) -> Assignability {
    let mut budget = QueryBudget::default();
    let cancellation = CancellationToken::new();
    check_assignability_bounded(store, hierarchy, actual, expected, &mut budget, &cancellation).into()
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

    #[test]
    fn bounded_relation_handles_budget_and_cancellation() {
        let mut store = TypeStore::new();
        let hier = MapTypeHierarchy::new();
        let t_int = store.nominal(test_decl("Int"));
        let t_str = store.nominal(test_decl("String"));

        let token = CancellationToken::new();
        token.cancel();
        let mut budget = QueryBudget::new(10);
        let res = check_subtype_bounded(&store, &hier, t_int, t_str, &mut budget, &token);
        assert!(res.is_cancelled());

        let uncancelled = CancellationToken::new();
        let mut tiny_budget = QueryBudget::new(0);
        let res = check_subtype_bounded(&store, &hier, t_int, t_str, &mut tiny_budget, &uncancelled);
        assert!(res.is_budget_exceeded());
    }
}
