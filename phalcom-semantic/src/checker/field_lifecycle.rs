//! Compiler-owned instance-field lifecycle proofs.

use crate::checker::analysis::CallableAnalysis;
use crate::checker::context::CheckingContext;
use crate::checker::flow::{FieldInitialization, FieldState, FlowState};
use crate::identity::{DeclarationId, DispatchSide, FieldId};
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::outcome::RelationOutcome;
use phalcom_ast::ast::{ClassDef, ClassMember};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldLifecycleFact {
    pub field: FieldId,
    pub contract: TypeKnowledge,
    pub read_knowledge: TypeKnowledge,
    pub initialization: FieldInitialization,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FieldLifecycleTable {
    pub fields: BTreeMap<FieldId, FieldLifecycleFact>,
}

impl FieldLifecycleTable {
    pub fn seed_flow_for_owner(&self, flow: &mut FlowState, owner: &DeclarationId, constructor: bool) {
        for fact in self
            .fields
            .values()
            .filter(|fact| fact.field.owner == *owner && fact.field.side == DispatchSide::Instance)
        {
            flow.seed_field(FieldState {
                field: fact.field.clone(),
                contract: fact.contract.clone(),
                current: if constructor || fact.initialization == FieldInitialization::DefinitelyInitialized {
                    fact.read_knowledge.clone()
                } else {
                    TypeKnowledge::Unknown(UnknownReason::MissingInitializer)
                },
                initialization: fact.initialization,
                version: 0,
            });
        }
    }

    pub fn extend(&mut self, other: Self) {
        self.fields.extend(other.fields);
    }
}

/// Checks default initializers and produces constructor-entry field seeds.
pub(crate) fn default_field_seeds(ctx: &mut CheckingContext<'_>, class_def: &ClassDef) -> FieldLifecycleTable {
    let owner = DeclarationId::new(ctx.current_module.clone(), class_def.name.clone().into());
    let mut table = FieldLifecycleTable::default();
    for member in &class_def.members {
        let ClassMember::Field(field) = member else { continue };
        if super::declaration::member_side(member) != DispatchSide::Instance {
            continue;
        }
        let Some((field_id, contract)) = ctx.resolve_field_contract(&owner, DispatchSide::Instance, &field.name) else {
            continue;
        };
        let (read_knowledge, initialization) = if let Some(default) = &field.default {
            let initializer = super::expression::synthesize_expr(ctx, default);
            let proven = contract
                .ty()
                .is_some_and(|expected| matches!(ctx.check_knowledge_against_type(&initializer, expected), RelationOutcome::Proven { .. }));
            if proven {
                (
                    TypeKnowledge::established(contract.ty().expect("proven contract type"), EvidenceOrigin::FieldLifecycle),
                    FieldInitialization::DefinitelyInitialized,
                )
            } else {
                (initializer, FieldInitialization::MaybeInitialized)
            }
        } else {
            (TypeKnowledge::Unknown(UnknownReason::MissingInitializer), FieldInitialization::Uninitialized)
        };
        table.fields.insert(
            field_id.clone(),
            FieldLifecycleFact {
                field: field_id,
                contract,
                read_knowledge,
                initialization,
            },
        );
    }
    table
}

pub(crate) fn finalize_instance_field_lifecycle<'a>(
    defaults: &FieldLifecycleTable,
    constructors: impl IntoIterator<Item = &'a CallableAnalysis>,
) -> FieldLifecycleTable {
    let constructors = constructors.into_iter().collect::<Vec<_>>();
    if constructors.is_empty() {
        return defaults.clone();
    }
    let mut result = defaults.clone();
    for fact in result.fields.values_mut() {
        let normal_exits = constructors.iter().flat_map(|constructor| constructor.exits.returns.iter()).collect::<Vec<_>>();
        if normal_exits.is_empty() {
            continue;
        }
        let definitely_initialized = normal_exits.iter().all(|exit| {
            exit.fields
                .get(&fact.field)
                .is_some_and(|state| state.initialization == FieldInitialization::DefinitelyInitialized)
        });
        fact.initialization = if definitely_initialized {
            FieldInitialization::DefinitelyInitialized
        } else {
            FieldInitialization::MaybeInitialized
        };
        fact.read_knowledge = match (definitely_initialized, fact.contract.ty()) {
            (true, Some(ty)) => TypeKnowledge::established(ty, EvidenceOrigin::FieldLifecycle),
            _ => TypeKnowledge::Unknown(UnknownReason::MissingInitializer),
        };
    }
    result
}
