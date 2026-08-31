use super::environment::TypeView;
use super::evidence::TypeKnowledge;
use super::id::{RecordRowId, TypeId};
use super::outcome::{BlockReason, BudgetReport, CancellationToken, DynamicBoundaryObligation, QueryBudget, RelationFailure, RelationOutcome};
use super::row::{RecordAccess, RecordRowTail};
use super::store::{TypeData, TypeStore};
use super::variance::Variance;
use crate::core_surface::CoreDeclarationIds;
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
    fn supertype_template(&self, _declaration: &DeclarationId) -> Option<&GenericSupertypeTemplate> {
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
    DynamicBoundary(DynamicBoundaryObligation),
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
        matches!(self, Self::Assignable)
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefutationReason {
    IncompatibleNominal,
    TypeMismatch,
    UnionMemberMismatch,
}

/// Evaluates subtyping with explicit budgets, cancellation, and cycle detection.
pub fn check_subtype_bounded(
    store: &mut TypeStore,
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
    store: &mut TypeStore,
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

    let sub_data = store.get(sub).clone();
    let sup_data = store.get(sup).clone();

    let res = match (sub_data, sup_data) {
        (TypeData::Union(members), _) => {
            for &m in members.iter() {
                let outcome = check_subtype_impl(store, hierarchy, m, sup, budget, cancellation, visited);
                match outcome {
                    RelationOutcome::Proven { .. } => {}
                    RelationOutcome::Refuted(_) => {
                        return RelationOutcome::Refuted(RelationFailure::UnionMemberMismatch { actual: sub, expected: sup });
                    }
                    terminal => return terminal,
                }
            }
            RelationOutcome::proven(())
        }
        (_, TypeData::Union(members)) => {
            for &m in members.iter() {
                let outcome = check_subtype_impl(store, hierarchy, sub, m, budget, cancellation, visited);
                match outcome {
                    RelationOutcome::Proven { .. } => return RelationOutcome::proven(()),
                    RelationOutcome::Refuted(_) => {}
                    terminal => return terminal,
                }
            }
            RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
        }
        (TypeData::ClassObject { declaration: sub_decl }, TypeData::ClassObject { declaration: sup_decl }) => {
            if hierarchy.is_subclass(&sub_decl, &sup_decl) {
                RelationOutcome::proven(())
            } else {
                RelationOutcome::Refuted(RelationFailure::IncompatibleNominal {
                    actual: sub_decl,
                    expected: sup_decl,
                })
            }
        }
        (TypeData::Nominal { declaration: sub_decl }, TypeData::Nominal { declaration: sup_decl }) => {
            if hierarchy.is_subclass(&sub_decl, &sup_decl) || CoreDeclarationIds::default().is_object(&sup_decl) {
                RelationOutcome::proven(())
            } else {
                RelationOutcome::Refuted(RelationFailure::IncompatibleNominal {
                    actual: sub_decl,
                    expected: sup_decl,
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
                    let origin_decl = match store.get(sub_orig) {
                        TypeData::Nominal { declaration } => Some(declaration.clone()),
                        _ => None,
                    };

                    for (idx, (&a_sub, &a_sup)) in sub_args.iter().zip(sup_args.iter()).enumerate() {
                        let variance = origin_decl
                            .as_ref()
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

                        match arg_outcome {
                            RelationOutcome::Proven { .. } => {}
                            RelationOutcome::Refuted(_) => {
                                return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                            }
                            terminal => return terminal,
                        }
                    }
                    RelationOutcome::proven(())
                } else {
                    RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
                }
            } else {
                // Check generic supertype template if sub origin has one
                if let TypeData::Nominal { declaration: sub_decl } = store.get(sub_orig).clone() {
                    if let Some(template) = hierarchy.supertype_template(&sub_decl) {
                        let mut env = super::environment::TypeEnvironment::new();
                        // Search existing TypeParameterIds in store for this declaration
                        for (idx, &arg) in sub_args.iter().enumerate() {
                            if let Some(param_id) =
                                store.find_type_parameter_id(&super::parameter::TypeParameterOwner::Declaration(sub_decl.clone()), idx as u32)
                            {
                                env.bind_param(param_id, arg);
                            }
                        }
                        let specialized_super = TypeView::new(template.supertype, env).materialize(store);
                        check_subtype_impl(store, hierarchy, specialized_super, sup, budget, cancellation, visited)
                    } else {
                        RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
                    }
                } else {
                    RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
                }
            }
        }

        (
            TypeData::ExactCase {
                variant: sub_var,
                enum_type: sub_enum,
            },
            TypeData::ExactCase {
                variant: sup_var,
                enum_type: sup_enum,
            },
        ) => {
            if sub_var == sup_var {
                check_subtype_impl(store, hierarchy, sub_enum, sup_enum, budget, cancellation, visited)
            } else {
                RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
            }
        }
        (TypeData::ExactCase { enum_type, .. }, _) => check_subtype_impl(store, hierarchy, enum_type, sup, budget, cancellation, visited),

        (TypeData::Tuple(sub_elems), TypeData::Tuple(sup_elems)) => {
            if sub_elems.len() == sup_elems.len() {
                for (a, b) in sub_elems.iter().zip(sup_elems.iter()) {
                    if a.label != b.label {
                        return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                    }
                    let out = check_subtype_impl(store, hierarchy, a.ty, b.ty, budget, cancellation, visited);
                    match out {
                        RelationOutcome::Proven { .. } => {}
                        RelationOutcome::Refuted(_) => {
                            return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                        }
                        terminal => return terminal,
                    }
                }
                RelationOutcome::proven(())
            } else {
                RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
            }
        }
        (TypeData::Record(sub_row_id), TypeData::Record(sup_row_id)) => check_record_row_subtype(
            store,
            hierarchy,
            sub_row_id,
            sup_row_id,
            sub,
            sup,
            RecordAccess::ReadOnly,
            budget,
            cancellation,
            visited,
        ),
        (TypeData::Callable(sub_call), TypeData::Callable(sup_call)) => {
            if sub_call.parameters.len() == sup_call.parameters.len() {
                for (sub_p, sup_p) in sub_call.parameters.iter().zip(sup_call.parameters.iter()) {
                    if sub_p.label != sup_p.label || sub_p.rest != sup_p.rest {
                        return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                    }
                    // Contravariant parameters: sup_p.ty <: sub_p.ty
                    let out = check_subtype_impl(store, hierarchy, sup_p.ty, sub_p.ty, budget, cancellation, visited);
                    match out {
                        RelationOutcome::Proven { .. } => {}
                        RelationOutcome::Refuted(_) => {
                            return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                        }
                        terminal => return terminal,
                    }
                }
                // Covariant return type: sub_call.return_type <: sup_call.return_type
                check_subtype_impl(store, hierarchy, sub_call.return_type, sup_call.return_type, budget, cancellation, visited)
            } else {
                RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
            }
        }
        (TypeData::Callable(_), TypeData::Nominal { declaration: sup_decl }) => {
            let core_ids = CoreDeclarationIds::default();
            if core_ids.is_callable_supertype(&sup_decl) {
                RelationOutcome::proven(())
            } else {
                RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
            }
        }
        (TypeData::Family(sub_fid), TypeData::Family(sup_fid)) => {
            let sub_family = store.get_family(sub_fid).clone();
            let sup_family = store.get_family(sup_fid).clone();
            // Width subtyping: every required member in sup_family must be present in sub_family
            for sup_m in sup_family.members.iter() {
                let Some(sub_m) = sub_family.find_operation(&sup_m.operation) else {
                    return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                };
                if sub_m.member_kind != sup_m.member_kind {
                    return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                }
                // Member subtyping: sub_m.ty <: sup_m.ty
                let out = check_subtype_impl(store, hierarchy, sub_m.ty, sup_m.ty, budget, cancellation, visited);
                match out {
                    RelationOutcome::Proven { .. } => {}
                    RelationOutcome::Refuted(_) => {
                        return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                    }
                    terminal => return terminal,
                }
            }
            RelationOutcome::proven(())
        }
        (_, TypeData::Nominal { declaration: sup_decl }) if CoreDeclarationIds::default().is_object(&sup_decl) => RelationOutcome::proven(()),
        (TypeData::Unit, TypeData::Unit) => RelationOutcome::proven(()),
        _ => RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup }),
    };

    visited.remove(&(sub, sup));
    res
}

/// Check whether `sub` is a canonical subtype of `sup` (`sub <: sup`).
pub fn is_subtype(store: &mut TypeStore, hierarchy: &dyn TypeHierarchy, sub: TypeId, sup: TypeId) -> bool {
    let mut budget = QueryBudget::default();
    let cancellation = CancellationToken::new();
    check_subtype_bounded(store, hierarchy, sub, sup, &mut budget, &cancellation).is_proven()
}

/// Evaluates assignability with bounded queries, cancellation, and cycle tracking.
pub fn check_assignability_bounded(
    store: &mut TypeStore,
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

    match expected {
        TypeKnowledge::Known(expected_evidence) => check_knowledge_against_type_bounded(store, hierarchy, actual, expected_evidence.ty(), budget, cancellation),
        TypeKnowledge::Unknown(reason) => RelationOutcome::Blocked(BlockReason::UnknownType(reason.clone())),
        TypeKnowledge::Dynamic(_) => RelationOutcome::DynamicBoundary(DynamicBoundaryObligation {
            reason: "dynamic boundary".into(),
        }),
    }
}

/// Checks formal knowledge against a canonical contract type without first
/// manufacturing an expected `TypeKnowledge` value.
pub fn check_knowledge_against_type_bounded(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    actual: &TypeKnowledge,
    expected: TypeId,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
) -> RelationOutcome {
    if cancellation.is_cancelled() {
        return RelationOutcome::Cancelled;
    }
    if actual.is_dynamic() {
        return RelationOutcome::DynamicBoundary(DynamicBoundaryObligation {
            reason: "dynamic boundary".into(),
        });
    }
    let TypeKnowledge::Known(actual_evidence) = actual else {
        return match actual {
            TypeKnowledge::Unknown(reason) => RelationOutcome::Blocked(BlockReason::UnknownType(reason.clone())),
            TypeKnowledge::Dynamic(_) => RelationOutcome::DynamicBoundary(DynamicBoundaryObligation {
                reason: "dynamic boundary".into(),
            }),
            TypeKnowledge::Known(_) => unreachable!("known knowledge matched above"),
        };
    };

    if matches!(store.get(actual_evidence.ty()), TypeData::Parameter(_)) || matches!(store.get(expected), TypeData::Parameter(_)) {
        return RelationOutcome::Blocked(BlockReason::RecursiveFixpoint);
    }

    // Formal Known values are already restricted to Established/Assumed. Both
    // are valid static premises; advisory evidence must not enter this path.
    check_subtype_bounded(store, hierarchy, actual_evidence.ty(), expected, budget, cancellation)
}

/// Unbounded convenience wrapper for knowledge-to-contract checking.
pub fn check_knowledge_against_type(store: &mut TypeStore, hierarchy: &dyn TypeHierarchy, actual: &TypeKnowledge, expected: TypeId) -> Assignability {
    let mut budget = QueryBudget::default();
    let cancellation = CancellationToken::new();
    let outcome = check_knowledge_against_type_bounded(store, hierarchy, actual, expected, &mut budget, &cancellation);
    relation_to_assignability(outcome, actual.ty(), Some(expected))
}

fn relation_to_assignability(outcome: RelationOutcome, actual: Option<TypeId>, expected: Option<TypeId>) -> Assignability {
    match outcome {
        RelationOutcome::Proven { .. } => Assignability::Assignable,
        RelationOutcome::Refuted(failure) => {
            let (failure_actual, failure_expected, reason) = match failure {
                RelationFailure::IncompatibleNominal { .. } => (actual, expected, RefutationReason::IncompatibleNominal),
                RelationFailure::UnionMemberMismatch { actual, expected } => (Some(actual), Some(expected), RefutationReason::UnionMemberMismatch),
                RelationFailure::TypeMismatch { actual, expected } => (Some(actual), Some(expected), RefutationReason::TypeMismatch),
                RelationFailure::CycleDetected { sub, sup } => (Some(sub), Some(sup), RefutationReason::TypeMismatch),
                RelationFailure::DepthExceeded | RelationFailure::Custom(_) => (actual, expected, RefutationReason::TypeMismatch),
            };
            let (Some(actual), Some(expected)) = (failure_actual, failure_expected) else {
                return Assignability::Blocked(BlockReason::RecursiveFixpoint);
            };
            Assignability::Refuted { actual, expected, reason }
        }
        RelationOutcome::DynamicBoundary(obligation) => Assignability::DynamicBoundary(obligation),
        RelationOutcome::Blocked(reason) => Assignability::Blocked(reason),
        RelationOutcome::Cancelled => Assignability::Cancelled,
        RelationOutcome::BudgetExceeded(report) => Assignability::BudgetExceeded(report),
        RelationOutcome::InternalFailure(message) => Assignability::InternalFailure(message),
    }
}

/// Checks assignability from an expression's type knowledge to an expected type knowledge.
pub fn check_assignability(store: &mut TypeStore, hierarchy: &dyn TypeHierarchy, actual: &TypeKnowledge, expected: &TypeKnowledge) -> Assignability {
    let mut budget = QueryBudget::default();
    let cancellation = CancellationToken::new();
    let outcome = check_assignability_bounded(store, hierarchy, actual, expected, &mut budget, &cancellation);
    relation_to_assignability(outcome, actual.ty(), expected.ty())
}

// Recursive record checking carries shared query state explicitly for budget and cycle control.
#[allow(clippy::too_many_arguments)]
pub fn check_record_row_subtype(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    sub_row_id: RecordRowId,
    sup_row_id: RecordRowId,
    sub_ty: TypeId,
    sup_ty: TypeId,
    access: RecordAccess,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
    visited: &mut HashSet<(TypeId, TypeId)>,
) -> RelationOutcome<()> {
    let sub_row = store.record_row(sub_row_id).clone();
    let sup_row = store.record_row(sup_row_id).clone();

    match access {
        RecordAccess::ReadOnly => {
            if sub_row.tail != sup_row.tail && (sub_row.tail != RecordRowTail::Closed || sup_row.tail != RecordRowTail::Closed) {
                return RelationOutcome::Blocked(BlockReason::RecursiveFixpoint);
            }
            for sup_f in sup_row.fields.iter() {
                if let Some(sub_f_ty) = sub_row.find_field(&sup_f.name) {
                    let out = check_subtype_impl(store, hierarchy, sub_f_ty, sup_f.ty, budget, cancellation, visited);
                    if !out.is_proven() {
                        return out;
                    }
                } else {
                    return RelationOutcome::Refuted(RelationFailure::TypeMismatch {
                        actual: sub_ty,
                        expected: sup_ty,
                    });
                }
            }
            RelationOutcome::proven(())
        }
        RecordAccess::ReadWrite => {
            if sub_row.tail != sup_row.tail {
                return RelationOutcome::Refuted(RelationFailure::TypeMismatch {
                    actual: sub_ty,
                    expected: sup_ty,
                });
            }
            if sub_row.fields.len() != sup_row.fields.len() {
                return RelationOutcome::Refuted(RelationFailure::TypeMismatch {
                    actual: sub_ty,
                    expected: sup_ty,
                });
            }
            for (sub_f, sup_f) in sub_row.fields.iter().zip(sup_row.fields.iter()) {
                if sub_f.name != sup_f.name {
                    return RelationOutcome::Refuted(RelationFailure::TypeMismatch {
                        actual: sub_ty,
                        expected: sup_ty,
                    });
                }
                let cov = check_subtype_impl(store, hierarchy, sub_f.ty, sup_f.ty, budget, cancellation, visited);
                if !cov.is_proven() {
                    return cov;
                }
                let contra = check_subtype_impl(store, hierarchy, sup_f.ty, sub_f.ty, budget, cancellation, visited);
                if !contra.is_proven() {
                    return contra;
                }
            }
            RelationOutcome::proven(())
        }
        RecordAccess::WriteOnly => {
            if sub_row.tail != sup_row.tail {
                return RelationOutcome::Refuted(RelationFailure::TypeMismatch {
                    actual: sub_ty,
                    expected: sup_ty,
                });
            }
            if sub_row.fields.len() != sup_row.fields.len() {
                return RelationOutcome::Refuted(RelationFailure::TypeMismatch {
                    actual: sub_ty,
                    expected: sup_ty,
                });
            }
            for (sub_f, sup_f) in sub_row.fields.iter().zip(sup_row.fields.iter()) {
                if sub_f.name != sup_f.name {
                    return RelationOutcome::Refuted(RelationFailure::TypeMismatch {
                        actual: sub_ty,
                        expected: sup_ty,
                    });
                }
                let contra = check_subtype_impl(store, hierarchy, sup_f.ty, sub_f.ty, budget, cancellation, visited);
                if !contra.is_proven() {
                    return contra;
                }
            }
            RelationOutcome::proven(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::evidence::EvidenceOrigin;
    use crate::types::store::TupleTypeElement;
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
        assert!(is_subtype(&mut store, &hier, t_int, t_int));
        // Never <: Int
        let never = store.never();
        assert!(is_subtype(&mut store, &hier, never, t_int));
        // Int <: Number

        assert!(is_subtype(&mut store, &hier, t_int, t_num));
        // Int <: Object
        assert!(is_subtype(&mut store, &hier, t_int, t_obj));
        // String !<: Int
        assert!(!is_subtype(&mut store, &hier, t_str, t_int));

        // Int <: (Int | String)
        let int_or_str = store.union(&[t_int, t_str]);
        assert!(is_subtype(&mut store, &hier, t_int, int_or_str));
        // (Int | String) <: Object
        assert!(is_subtype(&mut store, &hier, int_or_str, t_obj));
        // (Int | String) !<: Number
        assert!(!is_subtype(&mut store, &hier, int_or_str, t_num));
    }

    #[test]
    fn assignability_rejects_sound_contradiction() {
        let mut store = TypeStore::new();
        let hier = MapTypeHierarchy::new();

        let t_int = store.nominal(test_decl("Int"));
        let t_str = store.nominal(test_decl("String"));

        let actual = TypeKnowledge::established(t_int, EvidenceOrigin::Syntax);
        let expected = TypeKnowledge::assumed(t_str, EvidenceOrigin::DeveloperAnnotation);

        let res = check_assignability(&mut store, &hier, &actual, &expected);
        assert!(res.is_refuted());
    }

    #[test]
    fn knowledge_to_contract_relation_preserves_real_operands() {
        let mut store = TypeStore::new();
        let hier = MapTypeHierarchy::new();
        let actual_ty = store.nominal(test_decl("Int"));
        let expected_ty = store.nominal(test_decl("String"));
        let actual = TypeKnowledge::established(actual_ty, super::super::evidence::EvidenceOrigin::Syntax);

        let result = check_knowledge_against_type(&mut store, &hier, &actual, expected_ty);
        assert_eq!(
            result,
            Assignability::Refuted {
                actual: actual_ty,
                expected: expected_ty,
                reason: RefutationReason::IncompatibleNominal,
            }
        );
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
        let res = check_subtype_bounded(&mut store, &hier, t_int, t_str, &mut budget, &token);
        assert!(res.is_cancelled());

        let uncancelled = CancellationToken::new();
        let mut tiny_budget = QueryBudget::new(0);
        let res = check_subtype_bounded(&mut store, &hier, t_int, t_str, &mut tiny_budget, &uncancelled);
        assert!(res.is_budget_exceeded());
    }

    #[test]
    fn nested_structural_relation_preserves_terminal_outcomes() {
        let mut store = TypeStore::new();
        let mut hier = MapTypeHierarchy::new();
        let int_ty = store.nominal(test_decl("Int"));
        let number_ty = store.nominal(test_decl("Number"));
        let string_ty = store.nominal(test_decl("String"));
        hier.insert(test_decl("Int"), test_decl("Number"));
        let actual = store.tuple(vec![TupleTypeElement { label: None, ty: int_ty }, TupleTypeElement { label: None, ty: string_ty }].into_boxed_slice());
        let expected = store.tuple(vec![TupleTypeElement { label: None, ty: number_ty }, TupleTypeElement { label: None, ty: string_ty }].into_boxed_slice());

        let token = CancellationToken::new();
        let mut tiny_budget = QueryBudget::new(2);
        let exhausted = check_subtype_bounded(&mut store, &hier, actual, expected, &mut tiny_budget, &token);
        assert!(
            matches!(exhausted, RelationOutcome::BudgetExceeded(_)),
            "nested exhaustion was flattened: {exhausted:#?}"
        );

        token.cancel();
        let mut budget = QueryBudget::default();
        let cancelled = check_subtype_bounded(&mut store, &hier, actual, expected, &mut budget, &token);
        assert_eq!(cancelled, RelationOutcome::Cancelled);
    }

    #[test]
    fn dynamic_boundary_obligation_survives_assignability_projection() {
        let mut store = TypeStore::new();
        let hier = MapTypeHierarchy::new();
        let int_ty = store.nominal(test_decl("Int"));
        let actual = TypeKnowledge::Dynamic(super::super::evidence::DynamicReason::RuntimeReflection);
        let expected = TypeKnowledge::assumed(int_ty, EvidenceOrigin::DeveloperAnnotation);

        let mut budget = QueryBudget::default();
        let token = CancellationToken::new();
        let bounded = check_assignability_bounded(&mut store, &hier, &actual, &expected, &mut budget, &token);
        let RelationOutcome::DynamicBoundary(obligation) = bounded else {
            panic!("expected dynamic boundary, got {bounded:#?}");
        };
        assert_eq!(obligation.reason, "dynamic boundary");

        let projected = check_assignability(&mut store, &hier, &actual, &expected);
        assert!(matches!(projected, Assignability::DynamicBoundary(ref obligation) if obligation.reason == "dynamic boundary"));
        assert!(!projected.is_assignable(), "dynamic boundary is not static proof");
    }
}
