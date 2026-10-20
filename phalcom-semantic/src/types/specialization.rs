//! Owner-relative specialization of nominal receiver types.
//!
//! This module owns the projection from an actual receiver through generic
//! superclass templates.  Dispatch and associated-member lookup consume the
//! same result so inherited generic members cannot acquire feature-specific
//! substitutions.

use super::environment::{TypeEnvironment, TypeView};
use super::id::TypeId;
use super::outcome::{BlockReason, BudgetReport};
use super::relation::TypeHierarchy;
use super::store::{StaticArgument, TypeData, TypeStore};
use crate::identity::DeclarationId;
use phalcom_native_meta::UniverseKey;
use std::collections::HashSet;

/// Control shared with the checker for bounded, cancellable specialization.
pub trait SpecializationControl {
    /// Charges one projection step.
    fn charge_step(&self) -> Result<(), BudgetReport>;

    /// Reports whether the enclosing query was cancelled.
    fn is_cancelled(&self) -> bool;
}

/// One owner reached while projecting a receiver through its inheritance path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiverSpecializationStep {
    pub owner: DeclarationId,
    pub specialized_form: TypeId,
}

/// Canonical owner-relative specialization view for one selected member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiverSpecialization {
    pub receiver: TypeId,
    pub receiver_owner: DeclarationId,
    pub target_owner: DeclarationId,
    pub environment: TypeEnvironment,
    pub path: Box<[ReceiverSpecializationStep]>,
}

/// Why receiver projection could not produce a canonical owner environment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReceiverSpecializationFailure {
    UnsupportedReceiver,
    TargetNotReachable,
    InvalidSupertypeTemplate,
    InheritanceCycle,
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(String),
}

fn receiver_parts(store: &TypeStore, receiver: TypeId) -> Option<(DeclarationId, Vec<StaticArgument>)> {
    match store.get(receiver) {
        TypeData::Nominal { declaration } | TypeData::ClassObject { declaration } => Some((declaration.clone(), Vec::new())),
        TypeData::Applied { .. } | TypeData::ExactCase { .. } => store.applied_nominal_static_parts(receiver),
        _ => None,
    }
}

fn bind_owner_parameters(store: &TypeStore, owner: &DeclarationId, args: &[StaticArgument], environment: &mut TypeEnvironment) -> bool {
    for (index, arg) in args.iter().enumerate() {
        let Some(parameter) = store.find_type_parameter_id(&super::parameter::TypeParameterOwner::Declaration(owner.clone()), index as u32) else {
            return false;
        };
        match arg {
            StaticArgument::Term(ty) => environment.bind_param(parameter, *ty).expect("specialization environment is source mutable"),
            StaticArgument::RecordRow(row) => environment.bind_row(parameter, *row).expect("specialization environment is source mutable"),
            StaticArgument::DynamicType => environment.bind_dynamic(parameter).expect("specialization environment is source mutable"),
        }
    }
    true
}

/// Projects one proper nominal receiver to its immediately declared ancestor.
///
/// This is the shared generic-supertype-template materialization contract for
/// subtype relation checking and member specialization. `None` means the
/// receiver has no declared ancestor or its owner arguments cannot form a
/// valid template environment.
pub fn project_direct_supertype(store: &mut TypeStore, hierarchy: &dyn TypeHierarchy, receiver: TypeId) -> Option<TypeId> {
    let (owner, arguments) = receiver_parts(store, receiver)?;
    if let Some(template) = hierarchy.supertype_template(&owner) {
        let mut environment = TypeEnvironment::new();
        if !bind_owner_parameters(store, &owner, &arguments, &mut environment) {
            return None;
        }
        Some(TypeView::new(template.supertype, environment).materialize(store))
    } else {
        hierarchy.superclass(&owner).cloned().map(|superclass| store.nominal_type(superclass))
    }
}

/// Projects `receiver` to `target_owner` and returns its declaration bindings.
///
/// Every generic superclass template is materialized only as an intermediate
/// canonical view.  The returned environment binds parameters belonging to
/// the selected owner and binds `Self` to the original receiver value.
pub fn specialize_receiver_to_owner<C: SpecializationControl>(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    receiver: TypeId,
    target_owner: &DeclarationId,
    control: &C,
) -> Result<ReceiverSpecialization, ReceiverSpecializationFailure> {
    let (receiver_owner, mut current_args) = receiver_parts(store, receiver).ok_or(ReceiverSpecializationFailure::UnsupportedReceiver)?;
    // A declaration form used as a class-side receiver is intentionally
    // unsaturated.  Keep that distinction at use sites, but give the
    // owner-relative projector symbolic parameter forms so template
    // signatures can still be specialized and later inference can saturate
    // them.  Ordinary applied receivers already arrive with concrete args.
    if current_args.is_empty() && matches!(store.get(receiver), TypeData::Nominal { .. } | TypeData::ClassObject { .. }) {
        let mut index = 0;
        while let Some(parameter) = store.find_type_parameter_id(&super::parameter::TypeParameterOwner::Declaration(receiver_owner.clone()), index) {
            current_args.push(StaticArgument::Term(store.parameter_form(parameter)));
            index += 1;
        }
    }
    let mut current_owner = receiver_owner.clone();
    let mut current_form = receiver;
    let mut path = Vec::new();
    let mut visited = HashSet::new();

    loop {
        if control.is_cancelled() {
            return Err(ReceiverSpecializationFailure::Cancelled);
        }
        control.charge_step().map_err(ReceiverSpecializationFailure::BudgetExceeded)?;
        if !visited.insert(current_owner.clone()) {
            return Err(ReceiverSpecializationFailure::InheritanceCycle);
        }

        let mut owner_environment = TypeEnvironment::new();
        if !bind_owner_parameters(store, &current_owner, &current_args, &mut owner_environment) {
            return Err(ReceiverSpecializationFailure::InvalidSupertypeTemplate);
        }
        path.push(ReceiverSpecializationStep {
            owner: current_owner.clone(),
            specialized_form: current_form,
        });

        if &current_owner == target_owner {
            owner_environment
                .bind_self(receiver)
                .expect("receiver specialization environment is source mutable");
            return Ok(ReceiverSpecialization {
                receiver,
                receiver_owner,
                target_owner: target_owner.clone(),
                environment: owner_environment,
                path: path.into_boxed_slice(),
            });
        }

        let next_form = if let Some(supertype) = project_direct_supertype(store, hierarchy, current_form) {
            supertype
        } else if matches!(store.get(receiver), TypeData::ClassObject { .. }) && target_owner == &crate::core_surface::universe_declaration(UniverseKey::Class)
        {
            store.nominal_type(target_owner.clone())
        } else {
            return Err(ReceiverSpecializationFailure::TargetNotReachable);
        };

        let Some((next_owner, next_args)) = store.applied_nominal_static_parts(next_form) else {
            return Err(ReceiverSpecializationFailure::InvalidSupertypeTemplate);
        };
        current_owner = next_owner;
        current_args = next_args;
        current_form = next_form;
    }
}
