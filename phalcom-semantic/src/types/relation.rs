use super::evidence::TypeKnowledge;
use super::id::{RecordRowId, TypeId};
use super::outcome::{BlockReason, BudgetReport, CancellationToken, DynamicBoundaryObligation, QueryBudget, RelationFailure, RelationOutcome};
use super::row::{AnonymousRecordRemainder, RecordRowTail, StructuralMemberPresence};
use super::specialization::project_direct_supertype;
use super::store::{TypeData, TypeStore};
use super::variance::Variance;
use crate::core_surface::CoreDeclarationIds;
use crate::declarations::GenericSupertypeTemplate;
use crate::field_key::FieldKey;
use crate::identity::{DeclarationId, ModuleId};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError, BudgetedSortedMap, SortedMapEntriesError};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

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
    parent: Option<Arc<MapTypeHierarchy>>,
    pub superclasses: std::collections::HashMap<DeclarationId, DeclarationId>,
    pub templates: std::collections::HashMap<DeclarationId, GenericSupertypeTemplate>,
    frozen_superclasses: Option<Arc<BudgetedSortedMap<DeclarationId, DeclarationId>>>,
    frozen_templates: Option<Arc<BudgetedSortedMap<DeclarationId, GenericSupertypeTemplate>>>,
    frozen_declaration_count: usize,
    module_declarations: HashMap<ModuleId, BTreeSet<DeclarationId>>,
}

impl MapTypeHierarchy {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an overlay hierarchy referencing an immutable parent authority.
    pub fn with_parent(parent: Arc<Self>) -> Self {
        Self {
            parent: Some(parent),
            superclasses: HashMap::new(),
            templates: HashMap::new(),
            frozen_superclasses: None,
            frozen_templates: None,
            frozen_declaration_count: 0,
            module_declarations: HashMap::new(),
        }
    }

    pub(crate) fn from_frozen_entries_with_allocation_budget(
        mut superclasses: Vec<(DeclarationId, DeclarationId)>,
        mut templates: Vec<(DeclarationId, GenericSupertypeTemplate)>,
        declaration_count: usize,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, FrozenTypeHierarchyError> {
        superclasses.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        templates.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        let superclasses = BudgetedSortedMap::from_sorted_entries(superclasses).map_err(|error| match error {
            SortedMapEntriesError::DuplicateKey => FrozenTypeHierarchyError::DuplicateSuperclass,
            SortedMapEntriesError::OutOfOrder => unreachable!("entries were sorted before frozen construction"),
        })?;
        let templates = BudgetedSortedMap::from_sorted_entries(templates).map_err(|error| match error {
            SortedMapEntriesError::DuplicateKey => FrozenTypeHierarchyError::DuplicateTemplate,
            SortedMapEntriesError::OutOfOrder => unreachable!("entries were sorted before frozen construction"),
        })?;
        Ok(Self {
            parent: None,
            superclasses: HashMap::new(),
            templates: HashMap::new(),
            frozen_superclasses: Some(allocations.arc(superclasses).map_err(FrozenTypeHierarchyError::Allocation)?),
            frozen_templates: Some(allocations.arc(templates).map_err(FrozenTypeHierarchyError::Allocation)?),
            frozen_declaration_count: declaration_count,
            module_declarations: HashMap::new(),
        })
    }

    pub fn parent(&self) -> Option<&Arc<Self>> {
        self.parent.as_ref()
    }

    pub fn local_len(&self) -> usize {
        self.frozen_declaration_count + self.module_declarations.values().map(BTreeSet::len).sum::<usize>()
    }

    pub fn len(&self) -> usize {
        self.local_len() + self.parent.as_ref().map_or(0, |parent| parent.len())
    }

    pub fn is_empty(&self) -> bool {
        self.local_len() == 0 && self.parent.as_ref().is_none_or(|parent| parent.is_empty())
    }

    pub fn superclass(&self, declaration: &DeclarationId) -> Option<&DeclarationId> {
        self.superclasses
            .get(declaration)
            .or_else(|| self.frozen_superclasses.as_ref().and_then(|entries| entries.get(declaration)))
            .or_else(|| self.parent.as_ref().and_then(|parent| parent.superclass(declaration)))
    }

    pub fn supertype_template(&self, declaration: &DeclarationId) -> Option<&GenericSupertypeTemplate> {
        self.templates
            .get(declaration)
            .or_else(|| self.frozen_templates.as_ref().and_then(|entries| entries.get(declaration)))
            .or_else(|| self.parent.as_ref().and_then(|parent| parent.supertype_template(declaration)))
    }

    pub fn iter_superclasses(&self) -> Box<dyn Iterator<Item = (&DeclarationId, &DeclarationId)> + '_> {
        let mut hierarchies = Vec::new();
        let mut current = Some(self);
        while let Some(hierarchy) = current {
            hierarchies.push(hierarchy);
            current = hierarchy.parent.as_deref();
        }
        Box::new(hierarchies.into_iter().rev().flat_map(|hierarchy| {
            let local = hierarchy.superclasses.iter();
            let frozen = hierarchy.frozen_superclasses.iter().flat_map(|entries| entries.iter());
            local.chain(frozen)
        }))
    }

    fn track_declaration(&mut self, declaration: &DeclarationId) {
        self.module_declarations
            .entry(declaration.module.clone())
            .or_default()
            .insert(declaration.clone());
    }

    fn maybe_untrack_declaration(&mut self, declaration: &DeclarationId) {
        if self.superclasses.contains_key(declaration) || self.templates.contains_key(declaration) {
            return;
        }

        if let Some(declarations) = self.module_declarations.get_mut(&declaration.module) {
            declarations.remove(declaration);
            if declarations.is_empty() {
                self.module_declarations.remove(&declaration.module);
            }
        }
    }

    /// Registers the direct superclass contribution for a declaration.
    pub fn insert(&mut self, class: DeclarationId, superclass: DeclarationId) {
        debug_assert!(
            self.parent.as_ref().map_or(true, |parent| parent.superclass(&class).is_none())
                && self.frozen_superclasses.as_ref().is_none_or(|entries| !entries.contains_key(&class)),
            "local mutation cannot overwrite canonical parent superclass: {class:?}"
        );
        self.track_declaration(&class);
        self.superclasses.insert(class, superclass);
    }

    /// Registers the declaration-shell-owned generic supertype template.
    pub fn insert_template(&mut self, template: GenericSupertypeTemplate) {
        debug_assert!(
            self.parent
                .as_ref()
                .map_or(true, |parent| parent.supertype_template(&template.declaration).is_none())
                && self
                    .frozen_templates
                    .as_ref()
                    .is_none_or(|entries| !entries.contains_key(&template.declaration)),
            "local mutation cannot overwrite canonical parent supertype template: {:?}",
            template.declaration
        );
        self.track_declaration(&template.declaration);
        self.templates.insert(template.declaration.clone(), template);
    }

    /// Removes direct hierarchy contributions owned by one source module.
    pub fn remove_module(&mut self, module: &phalcom_modules::identity::ModuleId) {
        let declarations = self.module_declarations.remove(module).unwrap_or_default();
        for declaration in declarations {
            self.remove(&declaration);
        }
    }

    /// Retracts only the direct-superclass contribution for a declaration.
    pub fn remove_superclass(&mut self, declaration: &DeclarationId) {
        self.superclasses.remove(declaration);
        self.maybe_untrack_declaration(declaration);
    }

    /// Retracts only the generic-supertype-template contribution for a declaration.
    pub fn remove_template(&mut self, declaration: &DeclarationId) {
        self.templates.remove(declaration);
        self.maybe_untrack_declaration(declaration);
    }

    /// Fully retires all hierarchy contributions for a declaration.
    pub fn remove(&mut self, declaration: &DeclarationId) {
        self.superclasses.remove(declaration);
        self.templates.remove(declaration);
        if let Some(declarations) = self.module_declarations.get_mut(&declaration.module) {
            declarations.remove(declaration);
            if declarations.is_empty() {
                self.module_declarations.remove(&declaration.module);
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum FrozenTypeHierarchyError {
    Allocation(AllocationError),
    DuplicateSuperclass,
    DuplicateTemplate,
}

impl TypeHierarchy for MapTypeHierarchy {
    fn superclass(&self, declaration: &DeclarationId) -> Option<&DeclarationId> {
        MapTypeHierarchy::superclass(self, declaration)
    }

    fn supertype_template(&self, declaration: &DeclarationId) -> Option<&GenericSupertypeTemplate> {
        MapTypeHierarchy::supertype_template(self, declaration)
    }

    fn is_subclass(&self, sub: &DeclarationId, sup: &DeclarationId) -> bool {
        if sub == sup {
            return true;
        }
        let mut curr = sub;
        let mut visited = HashSet::new();
        visited.insert(curr);
        while let Some(parent) = self.superclass(curr) {
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
    #[cfg(feature = "perf-observability")]
    telos_observability::PerfContext::current_thread().record_counter(telos_observability::PerfCounter::TypeRelationSubtypeChecks, 1);
    let mut visited = HashSet::new();
    check_subtype_impl(store, hierarchy, sub, sup, budget, cancellation, &mut visited, 0)
}

fn check_subtype_impl(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    sub: TypeId,
    sup: TypeId,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
    visited: &mut HashSet<(TypeId, TypeId)>,
    depth: u16,
) -> RelationOutcome {
    #[cfg(feature = "perf-observability")]
    telos_observability::PerfContext::current_thread().record_counter(telos_observability::PerfCounter::TypeRelationSteps, 1);
    if cancellation.is_cancelled() {
        return RelationOutcome::Cancelled;
    }
    if let Err(report) = budget.charge_step() {
        return RelationOutcome::BudgetExceeded(report);
    }
    if let Err(report) = budget.charge_pair() {
        return RelationOutcome::BudgetExceeded(report);
    }
    if depth > budget.max_type_depth {
        return RelationOutcome::BudgetExceeded(BudgetReport::new(
            crate::types::outcome::BudgetKind::TypeDepth,
            budget.max_type_depth.into(),
            depth.into(),
        ));
    }

    if sub == sup {
        return RelationOutcome::proven(());
    }

    // Only proper `Type` forms participate in ordinary value subtyping. An
    // unsaturated constructor or type lambda may be equal to itself, but it
    // must not inherit the blanket `Object` relation or bottom relation just
    // because its representation lives in the canonical type store.
    if store.kind_of(sub) != super::id::KindId::TYPE || store.kind_of(sup) != super::id::KindId::TYPE {
        return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
    }

    if sub == store.never() {
        return RelationOutcome::proven(());
    }

    if !visited.insert((sub, sup)) {
        return RelationOutcome::Refuted(RelationFailure::CycleDetected { sub, sup });
    }

    let res = (|| {
        let sub_data = store.get(sub).clone();
        let sup_data = store.get(sup).clone();

        match (sub_data, sup_data) {
            (TypeData::Union(members), _) => {
                for &m in members.iter() {
                    let outcome = check_subtype_impl(store, hierarchy, m, sup, budget, cancellation, visited, depth + 1);
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
                    let outcome = check_subtype_impl(store, hierarchy, sub, m, budget, cancellation, visited, depth + 1);
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
            (TypeData::Nominal { declaration: sub_decl }, TypeData::Applied { .. }) => {
                if let Some(template) = hierarchy.supertype_template(&sub_decl) {
                    check_subtype_impl(store, hierarchy, template.supertype, sup, budget, cancellation, visited, depth + 1)
                } else {
                    RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
                }
            }
            (TypeData::Applied { .. }, TypeData::Nominal { .. }) => match project_direct_supertype(store, hierarchy, sub) {
                Some(supertype) => check_subtype_impl(store, hierarchy, supertype, sup, budget, cancellation, visited, depth + 1),
                None => RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup }),
            },

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
                            let (Some(a_sub), Some(a_sup)) = (a_sub.as_type_id(), a_sup.as_type_id()) else {
                                if a_sub == a_sup {
                                    continue;
                                }
                                return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                            };
                            let variance = origin_decl
                                .as_ref()
                                .and_then(|decl| store.get_parameter_variance(decl, idx as u32))
                                .unwrap_or(Variance::Invariant);

                            let arg_outcome = match variance {
                                Variance::Covariant => {
                                    // Covariant: a_sub <: a_sup
                                    check_subtype_impl(store, hierarchy, a_sub, a_sup, budget, cancellation, visited, depth + 1)
                                }
                                Variance::Contravariant => {
                                    // Contravariant: a_sup <: a_sub
                                    check_subtype_impl(store, hierarchy, a_sup, a_sub, budget, cancellation, visited, depth + 1)
                                }
                                Variance::Invariant => {
                                    // Invariant: a_sub == a_sup (fast-path TypeId identity or bounded semantic equality)
                                    if a_sub == a_sup {
                                        RelationOutcome::proven(())
                                    } else {
                                        let forward = check_subtype_impl(store, hierarchy, a_sub, a_sup, budget, cancellation, visited, depth + 1);
                                        match forward {
                                            RelationOutcome::Proven { .. } => {
                                                check_subtype_impl(store, hierarchy, a_sup, a_sub, budget, cancellation, visited, depth + 1)
                                            }
                                            other => other,
                                        }
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
                    match project_direct_supertype(store, hierarchy, sub) {
                        Some(supertype) => check_subtype_impl(store, hierarchy, supertype, sup, budget, cancellation, visited, depth + 1),
                        None => RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup }),
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
                    check_subtype_impl(store, hierarchy, sub_enum, sup_enum, budget, cancellation, visited, depth + 1)
                } else {
                    RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
                }
            }
            (TypeData::ExactCase { enum_type, .. }, _) => check_subtype_impl(store, hierarchy, enum_type, sup, budget, cancellation, visited, depth + 1),

            (TypeData::AssociatedProjection(sub_projection), TypeData::AssociatedProjection(sup_projection)) => {
                if sub_projection == sup_projection {
                    RelationOutcome::proven(())
                } else {
                    RelationOutcome::Blocked(BlockReason::RecursiveFixpoint)
                }
            }
            (TypeData::AssociatedProjection(_), _) | (_, TypeData::AssociatedProjection(_)) => RelationOutcome::Blocked(BlockReason::RecursiveFixpoint),

            (TypeData::TypeMemberProjection(sub_projection), TypeData::TypeMemberProjection(sup_projection)) => {
                if sub_projection == sup_projection {
                    RelationOutcome::proven(())
                } else {
                    RelationOutcome::Blocked(BlockReason::RecursiveFixpoint)
                }
            }
            (TypeData::TypeMemberProjection(_), _) | (_, TypeData::TypeMemberProjection(_)) => RelationOutcome::Blocked(BlockReason::RecursiveFixpoint),

            (TypeData::Tuple(sub_tuple), TypeData::Tuple(sup_tuple)) => {
                let sub_tuple = sub_tuple.clone();
                let sup_tuple = sup_tuple.clone();

                let sub_pos: Vec<_> = sub_tuple.elements.iter().filter(|e| e.label.is_none()).collect();
                let sup_pos: Vec<_> = sup_tuple.elements.iter().filter(|e| e.label.is_none()).collect();
                let sub_lab: Vec<_> = sub_tuple.elements.iter().filter(|e| e.label.is_some()).collect();
                let sup_lab: Vec<_> = sup_tuple.elements.iter().filter(|e| e.label.is_some()).collect();

                // 1. Positional lane
                if sub_pos.len() < sup_pos.len() {
                    return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                }
                for (a, b) in sub_pos.iter().take(sup_pos.len()).zip(sup_pos.iter()) {
                    let out = check_subtype_impl(store, hierarchy, a.ty, b.ty, budget, cancellation, visited, depth + 1);
                    if !out.is_proven() {
                        return out;
                    }
                }
                if sub_pos.len() > sup_pos.len() {
                    match sup_tuple.positional_remainder {
                        super::store::AnonymousTupleRemainder::None => {
                            return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                        }
                        super::store::AnonymousTupleRemainder::Unconstrained => {}
                        super::store::AnonymousTupleRemainder::Typed(target_elem) => {
                            for extra in &sub_pos[sup_pos.len()..] {
                                let out = check_subtype_impl(store, hierarchy, extra.ty, target_elem, budget, cancellation, visited, depth + 1);
                                if !out.is_proven() {
                                    return out;
                                }
                            }
                        }
                    }
                }
                match sup_tuple.positional_remainder {
                    super::store::AnonymousTupleRemainder::None => {
                        if sub_tuple.positional_remainder != super::store::AnonymousTupleRemainder::None {
                            return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                        }
                    }
                    super::store::AnonymousTupleRemainder::Unconstrained => {}
                    super::store::AnonymousTupleRemainder::Typed(sup_elem) => match sub_tuple.positional_remainder {
                        super::store::AnonymousTupleRemainder::None => {}
                        super::store::AnonymousTupleRemainder::Typed(sub_elem) => {
                            let out = check_subtype_impl(store, hierarchy, sub_elem, sup_elem, budget, cancellation, visited, depth + 1);
                            if !out.is_proven() {
                                return out;
                            }
                        }
                        super::store::AnonymousTupleRemainder::Unconstrained => {
                            return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                        }
                    },
                }

                // 2. Labeled lane
                for target_field in sup_lab.iter() {
                    let target_name = target_field.label.as_ref().unwrap();
                    if let Some(source_field) = sub_lab.iter().find(|e| e.label.as_ref().unwrap() == target_name) {
                        let out = check_subtype_impl(store, hierarchy, source_field.ty, target_field.ty, budget, cancellation, visited, depth + 1);
                        if !out.is_proven() {
                            return out;
                        }
                    } else {
                        // Tuple fixed elements are always required — a missing labeled element is a failure
                        // unless the source has a compatible typed remainder.
                        match sub_tuple.labeled_remainder {
                            super::store::AnonymousTupleRemainder::None => {
                                return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                            }
                            super::store::AnonymousTupleRemainder::Typed(elem_ty) => {
                                let out = check_subtype_impl(store, hierarchy, elem_ty, target_field.ty, budget, cancellation, visited, depth + 1);
                                if !out.is_proven() {
                                    return out;
                                }
                            }
                            super::store::AnonymousTupleRemainder::Unconstrained => {
                                return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                            }
                        }
                    }
                }
                for source_field in sub_lab.iter() {
                    let source_name = source_field.label.as_ref().unwrap();
                    if !sup_lab.iter().any(|e| e.label.as_ref().unwrap() == source_name) {
                        match sup_tuple.labeled_remainder {
                            super::store::AnonymousTupleRemainder::None => {
                                return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                            }
                            super::store::AnonymousTupleRemainder::Unconstrained => {}
                            super::store::AnonymousTupleRemainder::Typed(target_elem) => {
                                let out = check_subtype_impl(store, hierarchy, source_field.ty, target_elem, budget, cancellation, visited, depth + 1);
                                if !out.is_proven() {
                                    return out;
                                }
                            }
                        }
                    }
                }
                match sup_tuple.labeled_remainder {
                    super::store::AnonymousTupleRemainder::None => {
                        if sub_tuple.labeled_remainder != super::store::AnonymousTupleRemainder::None {
                            return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                        }
                    }
                    super::store::AnonymousTupleRemainder::Unconstrained => {}
                    super::store::AnonymousTupleRemainder::Typed(sup_elem) => match sub_tuple.labeled_remainder {
                        super::store::AnonymousTupleRemainder::None => {}
                        super::store::AnonymousTupleRemainder::Typed(sub_elem) => {
                            let out = check_subtype_impl(store, hierarchy, sub_elem, sup_elem, budget, cancellation, visited, depth + 1);
                            if !out.is_proven() {
                                return out;
                            }
                        }
                        super::store::AnonymousTupleRemainder::Unconstrained => {
                            return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                        }
                    },
                }

                RelationOutcome::proven(())
            }
            (TypeData::Record(sub_row_id), TypeData::Record(sup_row_id)) => {
                check_record_row_subtype(store, hierarchy, sub_row_id, sup_row_id, sub, sup, budget, cancellation, visited, depth)
            }
            (TypeData::TypePack(sub_elems), TypeData::TypePack(sup_elems)) => {
                if sub_elems == sup_elems {
                    RelationOutcome::proven(())
                } else {
                    RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
                }
            }
            (TypeData::LabeledTypePack(sub_elems), TypeData::LabeledTypePack(sup_elems)) => {
                if sub_elems == sup_elems {
                    RelationOutcome::proven(())
                } else {
                    RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup })
                }
            }
            (TypeData::Pack(sub_pack), TypeData::Pack(sup_pack)) => {
                let sub_pack = sub_pack.clone();
                let sup_pack = sup_pack.clone();

                let sub_pos: Vec<_> = sub_pack.elements.iter().filter(|e| e.label.is_none()).collect();
                let sup_pos: Vec<_> = sup_pack.elements.iter().filter(|e| e.label.is_none()).collect();
                let sub_lab: Vec<_> = sub_pack.elements.iter().filter(|e| e.label.is_some()).collect();
                let sup_lab: Vec<_> = sup_pack.elements.iter().filter(|e| e.label.is_some()).collect();

                // 1. Positional lane
                if sub_pos.len() < sup_pos.len() {
                    return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                }
                for (a, b) in sub_pos.iter().take(sup_pos.len()).zip(sup_pos.iter()) {
                    let out = check_subtype_impl(store, hierarchy, a.ty, b.ty, budget, cancellation, visited, depth + 1);
                    if !out.is_proven() {
                        return out;
                    }
                }
                if sub_pos.len() > sup_pos.len() {
                    match sup_pack.positional_remainder {
                        super::store::AnonymousTupleRemainder::None => {
                            return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                        }
                        super::store::AnonymousTupleRemainder::Unconstrained => {}
                        super::store::AnonymousTupleRemainder::Typed(target_elem) => {
                            for extra in &sub_pos[sup_pos.len()..] {
                                let out = check_subtype_impl(store, hierarchy, extra.ty, target_elem, budget, cancellation, visited, depth + 1);
                                if !out.is_proven() {
                                    return out;
                                }
                            }
                        }
                    }
                }
                match sup_pack.positional_remainder {
                    super::store::AnonymousTupleRemainder::None => {
                        if sub_pack.positional_remainder != super::store::AnonymousTupleRemainder::None {
                            return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                        }
                    }
                    super::store::AnonymousTupleRemainder::Unconstrained => {}
                    super::store::AnonymousTupleRemainder::Typed(sup_elem) => match sub_pack.positional_remainder {
                        super::store::AnonymousTupleRemainder::None => {}
                        super::store::AnonymousTupleRemainder::Typed(sub_elem) => {
                            let out = check_subtype_impl(store, hierarchy, sub_elem, sup_elem, budget, cancellation, visited, depth + 1);
                            if !out.is_proven() {
                                return out;
                            }
                        }
                        super::store::AnonymousTupleRemainder::Unconstrained => {
                            return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                        }
                    },
                }

                // 2. Labeled lane
                for target_field in sup_lab.iter() {
                    let target_name = target_field.label.as_ref().unwrap();
                    if let Some(source_field) = sub_lab.iter().find(|e| e.label.as_ref().unwrap() == target_name) {
                        let out = check_subtype_impl(store, hierarchy, source_field.ty, target_field.ty, budget, cancellation, visited, depth + 1);
                        if !out.is_proven() {
                            return out;
                        }
                    } else {
                        match sub_pack.labeled_remainder {
                            super::store::AnonymousTupleRemainder::None => {
                                return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                            }
                            super::store::AnonymousTupleRemainder::Typed(elem_ty) => {
                                let out = check_subtype_impl(store, hierarchy, elem_ty, target_field.ty, budget, cancellation, visited, depth + 1);
                                if !out.is_proven() {
                                    return out;
                                }
                            }
                            super::store::AnonymousTupleRemainder::Unconstrained => {
                                return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                            }
                        }
                    }
                }
                for source_field in sub_lab.iter() {
                    let source_name = source_field.label.as_ref().unwrap();
                    if !sup_lab.iter().any(|e| e.label.as_ref().unwrap() == source_name) {
                        match sup_pack.labeled_remainder {
                            super::store::AnonymousTupleRemainder::None => {
                                return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                            }
                            super::store::AnonymousTupleRemainder::Unconstrained => {}
                            super::store::AnonymousTupleRemainder::Typed(target_elem) => {
                                let out = check_subtype_impl(store, hierarchy, source_field.ty, target_elem, budget, cancellation, visited, depth + 1);
                                if !out.is_proven() {
                                    return out;
                                }
                            }
                        }
                    }
                }
                match sup_pack.labeled_remainder {
                    super::store::AnonymousTupleRemainder::None => {
                        if sub_pack.labeled_remainder != super::store::AnonymousTupleRemainder::None {
                            return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                        }
                    }
                    super::store::AnonymousTupleRemainder::Unconstrained => {}
                    super::store::AnonymousTupleRemainder::Typed(sup_elem) => match sub_pack.labeled_remainder {
                        super::store::AnonymousTupleRemainder::None => {}
                        super::store::AnonymousTupleRemainder::Typed(sub_elem) => {
                            let out = check_subtype_impl(store, hierarchy, sub_elem, sup_elem, budget, cancellation, visited, depth + 1);
                            if !out.is_proven() {
                                return out;
                            }
                        }
                        super::store::AnonymousTupleRemainder::Unconstrained => {
                            return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                        }
                    },
                }

                RelationOutcome::proven(())
            }
            (
                TypeData::PackFromPacks {
                    positionals: sub_p,
                    labeled: sub_l,
                },
                TypeData::PackFromPacks {
                    positionals: sup_p,
                    labeled: sup_l,
                },
            ) => {
                if sub_p == sup_p && sub_l == sup_l {
                    RelationOutcome::proven(())
                } else {
                    RelationOutcome::Blocked(BlockReason::RecursiveFixpoint)
                }
            }
            (
                TypeData::TupleFromPacks {
                    positionals: sub_p,
                    labeled: sub_l,
                },
                TypeData::TupleFromPacks {
                    positionals: sup_p,
                    labeled: sup_l,
                },
            ) => {
                if sub_p == sup_p && sub_l == sup_l {
                    RelationOutcome::proven(())
                } else {
                    RelationOutcome::Blocked(BlockReason::RecursiveFixpoint)
                }
            }
            (
                TypeData::CallableFromPacks {
                    positionals: sub_p,
                    labeled: sub_l,
                    return_type: sub_r,
                },
                TypeData::CallableFromPacks {
                    positionals: sup_p,
                    labeled: sup_l,
                    return_type: sup_r,
                },
            ) => {
                if sub_p == sup_p && sub_l == sup_l && sub_r == sup_r {
                    RelationOutcome::proven(())
                } else {
                    RelationOutcome::Blocked(BlockReason::RecursiveFixpoint)
                }
            }
            (
                TypeData::CompleteTypePack {
                    positionals: sub_p,
                    labeled: sub_l,
                },
                TypeData::CompleteTypePack {
                    positionals: sup_p,
                    labeled: sup_l,
                },
            ) => {
                if sub_p == sup_p && sub_l == sup_l {
                    RelationOutcome::proven(())
                } else {
                    RelationOutcome::Blocked(BlockReason::RecursiveFixpoint)
                }
            }
            (TypeData::Callable(sub_call), TypeData::Callable(sup_call)) => {
                if sub_call.parameters.len() == sup_call.parameters.len() {
                    for (sub_p, sup_p) in sub_call.parameters.iter().zip(sup_call.parameters.iter()) {
                        if sub_p.label != sup_p.label || sub_p.rest != sup_p.rest {
                            return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                        }
                        // Contravariant parameters: sup_p.ty <: sub_p.ty
                        let out = check_subtype_impl(store, hierarchy, sup_p.ty, sub_p.ty, budget, cancellation, visited, depth + 1);
                        match out {
                            RelationOutcome::Proven { .. } => {}
                            RelationOutcome::Refuted(_) => {
                                return RelationOutcome::Refuted(RelationFailure::TypeMismatch { actual: sub, expected: sup });
                            }
                            terminal => return terminal,
                        }
                    }
                    // Covariant return type: sub_call.return_type <: sup_call.return_type
                    check_subtype_impl(
                        store,
                        hierarchy,
                        sub_call.return_type,
                        sup_call.return_type,
                        budget,
                        cancellation,
                        visited,
                        depth + 1,
                    )
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
                    let out = check_subtype_impl(store, hierarchy, sub_m.ty, sup_m.ty, budget, cancellation, visited, depth + 1);
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
        }
    })();

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
    #[cfg(feature = "perf-observability")]
    telos_observability::PerfContext::current_thread().record_counter(telos_observability::PerfCounter::TypeRelationAssignabilityChecks, 1);
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

    if actual_evidence.ty() == expected {
        return RelationOutcome::proven(());
    }

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
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
    visited: &mut HashSet<(TypeId, TypeId)>,
    depth: u16,
) -> RelationOutcome<()> {
    let sub_row = store.record_row(sub_row_id).clone();
    let sup_row = store.record_row(sup_row_id).clone();

    // Both fixed prefixes are stored in canonical FieldKey order. A single
    // merge walk classifies equal, source-only, and target-only keys.
    let (mut sub_index, mut sup_index) = (0, 0);
    while sub_index < sub_row.fields.len() && sup_index < sup_row.fields.len() {
        let source_field = &sub_row.fields[sub_index];
        let target_field = &sup_row.fields[sup_index];
        match source_field.key.canonical_cmp(&target_field.key) {
            std::cmp::Ordering::Less => {
                let outcome = check_fixed_field_against_tail(
                    store,
                    hierarchy,
                    &source_field.key,
                    source_field.ty,
                    sup_row.tail,
                    sub_ty,
                    sup_ty,
                    budget,
                    cancellation,
                    visited,
                    depth + 1,
                );
                if !outcome.is_proven() {
                    return outcome;
                }
                sub_index += 1;
            }
            std::cmp::Ordering::Greater => {
                let outcome = check_absent_optional_field(
                    store,
                    hierarchy,
                    &target_field.key,
                    target_field.ty,
                    target_field.presence,
                    sub_row.tail,
                    sub_ty,
                    sup_ty,
                    budget,
                    cancellation,
                    visited,
                    depth + 1,
                );
                if !outcome.is_proven() {
                    return outcome;
                }
                sup_index += 1;
            }
            std::cmp::Ordering::Equal => {
                if target_field.presence.is_required() && source_field.presence.is_optional() {
                    return record_row_mismatch(sub_ty, sup_ty);
                }
                let outcome = check_subtype_impl(store, hierarchy, source_field.ty, target_field.ty, budget, cancellation, visited, depth + 1);
                if !outcome.is_proven() {
                    return outcome;
                }
                sub_index += 1;
                sup_index += 1;
            }
        }
    }

    for source_field in sub_row.fields[sub_index..].iter() {
        let outcome = check_fixed_field_against_tail(
            store,
            hierarchy,
            &source_field.key,
            source_field.ty,
            sup_row.tail,
            sub_ty,
            sup_ty,
            budget,
            cancellation,
            visited,
            depth + 1,
        );
        if !outcome.is_proven() {
            return outcome;
        }
    }
    for target_field in sup_row.fields[sup_index..].iter() {
        let outcome = check_absent_optional_field(
            store,
            hierarchy,
            &target_field.key,
            target_field.ty,
            target_field.presence,
            sub_row.tail,
            sub_ty,
            sup_ty,
            budget,
            cancellation,
            visited,
            depth + 1,
        );
        if !outcome.is_proven() {
            return outcome;
        }
    }

    check_record_tail_subtype(
        store,
        hierarchy,
        sub_row.tail,
        sup_row.tail,
        sub_ty,
        sup_ty,
        budget,
        cancellation,
        visited,
        depth + 1,
    )
}

fn record_row_mismatch(sub_ty: TypeId, sup_ty: TypeId) -> RelationOutcome {
    RelationOutcome::Refuted(RelationFailure::TypeMismatch {
        actual: sub_ty,
        expected: sup_ty,
    })
}

fn field_key_type(store: &mut TypeStore, key: &FieldKey) -> TypeId {
    let core = CoreDeclarationIds::default();
    let declaration = match key {
        FieldKey::Symbol(_) => core.symbol,
        FieldKey::String(_) => core.string,
        FieldKey::Int(_) => core.int,
        FieldKey::Bool(_) => core.bool_,
    };
    store.nominal_type(declaration)
}

fn key_domain_contains_unresolved_parameter(store: &TypeStore, key_type: TypeId) -> bool {
    match store.get(key_type) {
        TypeData::Parameter(_) => true,
        TypeData::Union(members) => members.iter().any(|&member| key_domain_contains_unresolved_parameter(store, member)),
        _ => false,
    }
}

fn preserve_unknown_key_membership(store: &TypeStore, key_type: TypeId, outcome: RelationOutcome) -> RelationOutcome {
    match outcome {
        RelationOutcome::Refuted(_) if key_domain_contains_unresolved_parameter(store, key_type) => {
            RelationOutcome::Blocked(BlockReason::UnknownType(super::evidence::UnknownReason::InferenceBlocked))
        }
        other => other,
    }
}

/// Checks a concrete Record key against a key-domain type using the ordinary
/// relation engine. Every non-refutation outcome is retained for callers.
pub fn check_field_key_domain_membership(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    key: &FieldKey,
    key_type: TypeId,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
) -> RelationOutcome {
    let concrete_type = field_key_type(store, key);
    let outcome = check_subtype_bounded(store, hierarchy, concrete_type, key_type, budget, cancellation);
    preserve_unknown_key_membership(store, key_type, outcome)
}

fn check_field_key_domain_membership_impl(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    key: &FieldKey,
    key_type: TypeId,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
    visited: &mut HashSet<(TypeId, TypeId)>,
    depth: u16,
) -> RelationOutcome {
    let concrete_type = field_key_type(store, key);
    let outcome = check_subtype_impl(store, hierarchy, concrete_type, key_type, budget, cancellation, visited, depth);
    preserve_unknown_key_membership(store, key_type, outcome)
}

#[allow(clippy::too_many_arguments)]
fn check_fixed_field_against_tail(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    key: &FieldKey,
    value_type: TypeId,
    target_tail: RecordRowTail,
    sub_ty: TypeId,
    sup_ty: TypeId,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
    visited: &mut HashSet<(TypeId, TypeId)>,
    depth: u16,
) -> RelationOutcome {
    match target_tail {
        RecordRowTail::Closed => record_row_mismatch(sub_ty, sup_ty),
        RecordRowTail::Anonymous(remainder) => {
            let key_outcome = check_field_key_domain_membership_impl(store, hierarchy, key, remainder.key_type(), budget, cancellation, visited, depth);
            if !key_outcome.is_proven() {
                return key_outcome;
            }
            match remainder {
                AnonymousRecordRemainder::Unconstrained { .. } => RelationOutcome::proven(()),
                AnonymousRecordRemainder::Typed { value_type: expected, .. } => {
                    check_subtype_impl(store, hierarchy, value_type, expected, budget, cancellation, visited, depth + 1)
                }
            }
        }
        RecordRowTail::Parameter(_) | RecordRowTail::Projection(_) => RelationOutcome::Blocked(BlockReason::RecursiveFixpoint),
    }
}

#[allow(clippy::too_many_arguments)]
fn check_absent_optional_field(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    key: &FieldKey,
    target_type: TypeId,
    presence: StructuralMemberPresence,
    source_tail: RecordRowTail,
    sub_ty: TypeId,
    sup_ty: TypeId,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
    visited: &mut HashSet<(TypeId, TypeId)>,
    depth: u16,
) -> RelationOutcome {
    if presence.is_required() {
        return record_row_mismatch(sub_ty, sup_ty);
    }
    match source_tail {
        RecordRowTail::Closed => RelationOutcome::proven(()),
        RecordRowTail::Anonymous(remainder) => {
            let membership = check_field_key_domain_membership_impl(store, hierarchy, key, remainder.key_type(), budget, cancellation, visited, depth);
            match membership {
                RelationOutcome::Refuted(_) => RelationOutcome::proven(()),
                RelationOutcome::Proven { .. } => match remainder {
                    AnonymousRecordRemainder::Typed { value_type, .. } => {
                        check_subtype_impl(store, hierarchy, value_type, target_type, budget, cancellation, visited, depth + 1)
                    }
                    AnonymousRecordRemainder::Unconstrained { .. } => record_row_mismatch(sub_ty, sup_ty),
                },
                other => other,
            }
        }
        RecordRowTail::Parameter(_) | RecordRowTail::Projection(_) => RelationOutcome::Blocked(BlockReason::RecursiveFixpoint),
    }
}

#[allow(clippy::too_many_arguments)]
fn check_record_tail_subtype(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    sub_tail: RecordRowTail,
    sup_tail: RecordRowTail,
    sub_ty: TypeId,
    sup_ty: TypeId,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
    visited: &mut HashSet<(TypeId, TypeId)>,
    depth: u16,
) -> RelationOutcome {
    match sup_tail {
        RecordRowTail::Closed => match sub_tail {
            RecordRowTail::Closed => RelationOutcome::proven(()),
            _ => record_row_mismatch(sub_ty, sup_ty),
        },
        RecordRowTail::Anonymous(sup_remainder) => match sub_tail {
            RecordRowTail::Closed => RelationOutcome::proven(()),
            RecordRowTail::Anonymous(sub_remainder) => {
                let keys = check_subtype_impl(
                    store,
                    hierarchy,
                    sub_remainder.key_type(),
                    sup_remainder.key_type(),
                    budget,
                    cancellation,
                    visited,
                    depth,
                );
                if !keys.is_proven() {
                    return keys;
                }
                match (sub_remainder, sup_remainder) {
                    (AnonymousRecordRemainder::Typed { value_type: sub_value, .. }, AnonymousRecordRemainder::Typed { value_type: sup_value, .. }) => {
                        check_subtype_impl(store, hierarchy, sub_value, sup_value, budget, cancellation, visited, depth + 1)
                    }
                    (AnonymousRecordRemainder::Typed { .. }, AnonymousRecordRemainder::Unconstrained { .. })
                    | (AnonymousRecordRemainder::Unconstrained { .. }, AnonymousRecordRemainder::Unconstrained { .. }) => RelationOutcome::proven(()),
                    (AnonymousRecordRemainder::Unconstrained { .. }, AnonymousRecordRemainder::Typed { .. }) => record_row_mismatch(sub_ty, sup_ty),
                }
            }
            RecordRowTail::Parameter(_) | RecordRowTail::Projection(_) => RelationOutcome::Blocked(BlockReason::RecursiveFixpoint),
        },
        RecordRowTail::Parameter(required_tail) => match sub_tail {
            RecordRowTail::Parameter(actual_tail) if actual_tail == required_tail => RelationOutcome::proven(()),
            _ => RelationOutcome::Blocked(BlockReason::RecursiveFixpoint),
        },
        RecordRowTail::Projection(required_projection) => match sub_tail {
            RecordRowTail::Projection(actual_projection) if actual_projection == required_projection => RelationOutcome::proven(()),
            _ => RelationOutcome::Blocked(BlockReason::RecursiveFixpoint),
        },
    }
}

/// Result of looking up one exact key in a canonical unordered Record row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnorderedFieldLookup {
    Required(TypeId),
    Optional(TypeId),
    Absent,
    Unknown(RelationOutcome),
}

/// Looks up one FieldKey using canonical row fields and anonymous-tail key
/// domains. Unresolved captured tails remain explicitly unknown.
pub fn lookup_unordered_field(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    row_id: RecordRowId,
    key: &FieldKey,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
) -> UnorderedFieldLookup {
    let row = store.record_row(row_id).clone();
    if let Some(field) = row.find_field_entry(key) {
        return match field.presence {
            StructuralMemberPresence::Required => UnorderedFieldLookup::Required(field.ty),
            StructuralMemberPresence::Optional => UnorderedFieldLookup::Optional(field.ty),
        };
    }
    match row.tail {
        RecordRowTail::Closed => UnorderedFieldLookup::Absent,
        RecordRowTail::Anonymous(remainder) => match check_field_key_domain_membership(store, hierarchy, key, remainder.key_type(), budget, cancellation) {
            RelationOutcome::Refuted(_) => UnorderedFieldLookup::Absent,
            RelationOutcome::Proven { .. } => match remainder {
                AnonymousRecordRemainder::Typed { value_type, .. } => UnorderedFieldLookup::Optional(value_type),
                AnonymousRecordRemainder::Unconstrained { .. } => UnorderedFieldLookup::Unknown(RelationOutcome::Blocked(BlockReason::UnknownType(
                    super::evidence::UnknownReason::NoTypeEvidence,
                ))),
            },
            other => UnorderedFieldLookup::Unknown(other),
        },
        RecordRowTail::Parameter(_) | RecordRowTail::Projection(_) => UnorderedFieldLookup::Unknown(RelationOutcome::Blocked(BlockReason::RecursiveFixpoint)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field_key::FieldKey;
    use crate::types::evidence::EvidenceOrigin;
    use crate::types::id::KindId;
    use crate::types::store::TupleTypeElement;
    use phalcom_modules::identity::ModuleId;

    fn test_decl(name: &str) -> DeclarationId {
        let module = ModuleId::universe_root();
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
        let actual = store.tuple(vec![TupleTypeElement::unlabeled(int_ty), TupleTypeElement::unlabeled(string_ty)].into_boxed_slice());
        let expected = store.tuple(vec![TupleTypeElement::unlabeled(number_ty), TupleTypeElement::unlabeled(string_ty)].into_boxed_slice());

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

    fn template(declaration: DeclarationId, supertype: TypeId) -> GenericSupertypeTemplate {
        GenericSupertypeTemplate {
            declaration,
            supertype,
            structural_form: Some("Base".into()),
        }
    }

    #[test]
    fn direct_superclass_retraction_preserves_generic_template() {
        let mut store = TypeStore::new();
        let child = test_decl("Child");
        let base = test_decl("Base");
        let template = template(child.clone(), store.nominal(base.clone()));
        let mut hierarchy = MapTypeHierarchy::new();

        hierarchy.insert(child.clone(), base.clone());
        hierarchy.insert_template(template.clone());
        hierarchy.remove_superclass(&child);

        assert_eq!(hierarchy.superclass(&child), None);
        assert_eq!(hierarchy.supertype_template(&child), Some(&template));
    }

    #[test]
    fn template_retraction_preserves_direct_superclass() {
        let mut store = TypeStore::new();
        let child = test_decl("Child");
        let base = test_decl("Base");
        let template = template(child.clone(), store.nominal(base.clone()));
        let mut hierarchy = MapTypeHierarchy::new();

        hierarchy.insert(child.clone(), base.clone());
        hierarchy.insert_template(template);
        hierarchy.remove_template(&child);

        assert_eq!(hierarchy.superclass(&child), Some(&base));
        assert_eq!(hierarchy.supertype_template(&child), None);
    }

    #[test]
    fn template_insertion_does_not_manufacture_direct_superclass() {
        let mut store = TypeStore::new();
        let child = test_decl("Child");
        let base = test_decl("Base");
        let template = template(child.clone(), store.nominal(base));
        let mut hierarchy = MapTypeHierarchy::new();

        hierarchy.insert_template(template.clone());

        assert_eq!(hierarchy.superclass(&child), None);
        assert_eq!(hierarchy.supertype_template(&child), Some(&template));
    }

    #[test]
    fn active_pair_is_removed_after_terminal_nested_relation() {
        let mut store = TypeStore::new();
        let mut hierarchy = MapTypeHierarchy::new();
        let int = store.nominal(test_decl("Int"));
        let number = store.nominal(test_decl("Number"));
        let string = store.nominal(test_decl("String"));
        hierarchy.insert(test_decl("Int"), test_decl("Number"));
        let expected = store.union(&[number, string]);
        let actual = store.tuple(Box::new([TupleTypeElement::unlabeled(int), TupleTypeElement::unlabeled(int)]));
        let required = store.tuple(Box::new([TupleTypeElement::unlabeled(expected), TupleTypeElement::unlabeled(expected)]));

        let mut budget = QueryBudget::default();
        let token = CancellationToken::new();
        assert!(check_subtype_bounded(&mut store, &hierarchy, actual, required, &mut budget, &token).is_proven());
    }

    #[test]
    fn cv07_tuple_lane_independence() {
        let mut store = TypeStore::new();
        let hier = MapTypeHierarchy::new();
        let int = store.nominal(test_decl("Int"));
        let float = store.nominal(test_decl("Float"));
        let string = store.nominal(test_decl("String"));

        // Open positional only: (Int, *: ..., name: String)
        let open_pos_target = store.intern_with_kind(
            TypeData::Tuple(super::super::store::TupleType {
                elements: Box::new([TupleTypeElement::unlabeled(int), TupleTypeElement::labeled("name".into(), string)]),
                positional_remainder: super::super::store::AnonymousTupleRemainder::Unconstrained,
                labeled_remainder: super::super::store::AnonymousTupleRemainder::None,
            }),
            KindId::TYPE,
        );

        // Source has extra positional: (Int, Float, name: String)
        let src_extra_pos = store.intern_with_kind(
            TypeData::Tuple(super::super::store::TupleType {
                elements: Box::new([
                    TupleTypeElement::unlabeled(int),
                    TupleTypeElement::unlabeled(float),
                    TupleTypeElement::labeled("name".into(), string),
                ]),
                positional_remainder: super::super::store::AnonymousTupleRemainder::None,
                labeled_remainder: super::super::store::AnonymousTupleRemainder::None,
            }),
            KindId::TYPE,
        );
        assert!(
            is_subtype(&mut store, &hier, src_extra_pos, open_pos_target),
            "Extra positional accepted by open positional lane"
        );

        // Source has extra labeled: (Int, name: String, age: Int) -> Rejected because labeled lane is closed!
        let src_extra_lab = store.intern_with_kind(
            TypeData::Tuple(super::super::store::TupleType {
                elements: Box::new([
                    TupleTypeElement::unlabeled(int),
                    TupleTypeElement::labeled("name".into(), string),
                    TupleTypeElement::labeled("age".into(), int),
                ]),
                positional_remainder: super::super::store::AnonymousTupleRemainder::None,
                labeled_remainder: super::super::store::AnonymousTupleRemainder::None,
            }),
            KindId::TYPE,
        );
        assert!(
            !is_subtype(&mut store, &hier, src_extra_lab, open_pos_target),
            "Extra labeled rejected when labeled lane is closed"
        );

        // Open labeled only: (Int, name: String, **: ...)
        let open_lab_target = store.intern_with_kind(
            TypeData::Tuple(super::super::store::TupleType {
                elements: Box::new([TupleTypeElement::unlabeled(int), TupleTypeElement::labeled("name".into(), string)]),
                positional_remainder: super::super::store::AnonymousTupleRemainder::None,
                labeled_remainder: super::super::store::AnonymousTupleRemainder::Unconstrained,
            }),
            KindId::TYPE,
        );
        assert!(
            is_subtype(&mut store, &hier, src_extra_lab, open_lab_target),
            "Extra labeled accepted by open labeled lane"
        );
        assert!(
            !is_subtype(&mut store, &hier, src_extra_pos, open_lab_target),
            "Extra positional rejected when positional lane is closed"
        );
    }

    #[test]
    fn cv08_structural_canonicalization() {
        let mut store = TypeStore::new();
        let hier = MapTypeHierarchy::new();
        let int = store.nominal(test_decl("Int"));
        let string = store.nominal(test_decl("String"));

        let r1 = store.record(Box::new([
            crate::types::row::RecordRowField::required(FieldKey::symbol("a"), int),
            crate::types::row::RecordRowField::required(FieldKey::symbol("b"), string),
        ]));
        let r2 = store.record(Box::new([
            crate::types::row::RecordRowField::required(FieldKey::symbol("b"), string),
            crate::types::row::RecordRowField::required(FieldKey::symbol("a"), int),
        ]));
        assert_eq!(r1, r2, "Unordered record fields canonicalize to identical TypeId");
        assert!(is_subtype(&mut store, &hier, r1, r2));
        assert!(is_subtype(&mut store, &hier, r2, r1));
    }

    #[test]
    fn cv09_proper_type_boundary_packs() {
        let mut store = TypeStore::new();
        let hier = MapTypeHierarchy::new();
        let int = store.nominal(test_decl("Int"));
        let string = store.nominal(test_decl("String"));

        let p1 = store.type_pack(vec![int].into_boxed_slice()).unwrap();
        let p2 = store.type_pack(vec![string].into_boxed_slice()).unwrap();
        assert!(!is_subtype(&mut store, &hier, p1, p2), "Unequal TypePacks do not have subtype relation");
    }
}
