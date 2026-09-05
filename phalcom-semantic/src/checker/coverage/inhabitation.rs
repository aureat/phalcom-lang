//! Productive inhabitation for closed recursive coverage domains.

use crate::checker::context::CheckerControl;
use crate::declarations::DeclarationTypeTable;
use crate::enum_semantics::EnumSemanticTable;
use crate::types::evidence::UnknownReason;
use crate::types::id::TypeId;
use crate::types::outcome::BlockReason;
use crate::types::relation::TypeHierarchy;
use crate::types::rigid::RigidArena;
use crate::types::store::{TypeData, TypeStore};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::domain::{decompose_domain, DomainDecomposition};
use super::subject::CoverageSubject;
use super::usefulness::CoverageMetrics;

/// Result of asking whether a type has a finite inhabitant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Inhabitation {
    Inhabited,
    Uninhabited,
    Unknown,
    Blocked(BlockReason),
}

#[derive(Clone, Debug, Default)]
struct DomainNode {
    constructors: Vec<Vec<TypeId>>,
    open: bool,
}

/// Computes the least productive fixed point of one closed coverage domain.
///
/// Recursive edges start uninhabited, so a cycle with no finite base path stays
/// uninhabited while a constructor with a finite payload path becomes inhabited.
/// All decomposition and fixed-point work remains query-local.
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_inhabitation(
    declarations: &DeclarationTypeTable,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    rigids: &mut RigidArena,
    enum_table: Option<&EnumSemanticTable>,
    subject: &CoverageSubject,
    control: &CheckerControl,
    metrics: &mut CoverageMetrics,
) -> Inhabitation {
    let mut nodes = BTreeSet::new();
    let mut pending = vec![subject.canonical];
    let mut domains = BTreeMap::<TypeId, DomainNode>::new();

    while let Some(ty) = pending.pop() {
        if control.is_cancelled() {
            return Inhabitation::Blocked(BlockReason::UnknownType(UnknownReason::InferenceCancelled));
        }
        if !nodes.insert(ty) {
            continue;
        }
        if let Err(report) = control.charge_step() {
            return Inhabitation::Blocked(BlockReason::BudgetExceeded(report));
        }
        let current = CoverageSubject::canonical(ty);
        metrics.constructor_decompositions += 1;
        let domain = match decompose_domain(declarations, store, hierarchy, rigids, enum_table, &current) {
            DomainDecomposition::Blocked(reason) => return Inhabitation::Blocked(reason),
            DomainDecomposition::Empty => DomainNode::default(),
            DomainDecomposition::Open => DomainNode {
                constructors: Vec::new(),
                open: true,
            },
            DomainDecomposition::Closed(cases) => {
                let mut node = DomainNode::default();
                for case in cases.iter() {
                    let fields = case.fields.iter().map(|field| field.canonical).collect::<Vec<_>>();
                    pending.extend(fields.iter().copied());
                    node.constructors.push(fields);
                }
                node
            }
        };
        domains.insert(ty, domain);
    }

    let mut states = nodes.iter().map(|ty| (*ty, Inhabitation::Uninhabited)).collect::<BTreeMap<_, _>>();
    let mut dependents = BTreeMap::<TypeId, Vec<TypeId>>::new();
    for (owner, domain) in &domains {
        for constructor in &domain.constructors {
            for field in constructor {
                let owners = dependents.entry(*field).or_default();
                if !owners.contains(owner) {
                    owners.push(*owner);
                }
            }
        }
    }

    // One worklist run is one fixed-point round. The old synchronous scan
    // spent one SCC charge per graph depth, so a finite recursive payload
    // chain could exhaust the round budget before reaching its base case.
    // State updates remain monotone (Uninhabited -> Unknown -> Inhabited), and
    // only affected constructors are revisited, preserving the least
    // productive fixed point while charging each unique domain decomposition
    // as one normal checker step.
    if control.is_cancelled() {
        return Inhabitation::Blocked(BlockReason::UnknownType(UnknownReason::InferenceCancelled));
    }
    if let Err(report) = control.charge_scc_iteration() {
        return Inhabitation::Blocked(BlockReason::BudgetExceeded(report));
    }
    metrics.inhabitation_iterations += 1;

    let mut queue = nodes.iter().copied().collect::<VecDeque<_>>();
    let mut queued = nodes.clone();
    while let Some(ty) = queue.pop_front() {
        queued.remove(&ty);
        if control.is_cancelled() {
            return Inhabitation::Blocked(BlockReason::UnknownType(UnknownReason::InferenceCancelled));
        }
        let Some(domain) = domains.get(&ty) else {
            continue;
        };
        let state = if domain.open {
            terminal_inhabitation(store, ty)
        } else {
            let mut saw_unknown = false;
            let mut productive = false;
            for constructor in &domain.constructors {
                let mut constructor_unknown = false;
                let mut constructor_productive = true;
                for field in constructor {
                    match states.get(field).unwrap_or(&Inhabitation::Unknown) {
                        Inhabitation::Inhabited => {}
                        Inhabitation::Unknown => constructor_unknown = true,
                        Inhabitation::Uninhabited => constructor_productive = false,
                        Inhabitation::Blocked(reason) => return Inhabitation::Blocked(reason.clone()),
                    }
                }
                if constructor_productive && !constructor_unknown {
                    productive = true;
                    break;
                }
                saw_unknown |= constructor_unknown;
            }
            if productive {
                Inhabitation::Inhabited
            } else if saw_unknown {
                Inhabitation::Unknown
            } else {
                Inhabitation::Uninhabited
            }
        };
        if states.get(&ty) != Some(&state) {
            states.insert(ty, state);
            if let Some(parents) = dependents.get(&ty) {
                for parent in parents {
                    if queued.insert(*parent) {
                        queue.push_back(*parent);
                    }
                }
            }
        }
    }

    states.get(&subject.canonical).cloned().unwrap_or(Inhabitation::Unknown)
}

fn terminal_inhabitation(store: &TypeStore, ty: TypeId) -> Inhabitation {
    match store.get(ty) {
        TypeData::Never => Inhabitation::Uninhabited,
        TypeData::Lambda(_) | TypeData::Family(_) | TypeData::SelfType(_) => Inhabitation::Unknown,
        _ => Inhabitation::Inhabited,
    }
}
