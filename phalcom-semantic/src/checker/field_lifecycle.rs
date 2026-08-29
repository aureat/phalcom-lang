//! Compiler-owned instance-field lifecycle proofs.

use crate::checker::analysis::CallableAnalysis;
use crate::checker::causal::CausalInvalidity;
use crate::checker::context::CheckingContext;
use crate::checker::flow::{FieldContractValidity, FieldInitialization, FieldState, FlowState};
use crate::identity::{DeclarationId, DispatchSide, FieldId};
use crate::types::evidence::{EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use crate::types::outcome::RelationOutcome;
use phalcom_ast::ast::{ClassDef, ClassMember};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FieldWriteReconciliation {
    pub current: TypeKnowledge,
    pub validity: FieldContractValidity,
}

pub(crate) fn reconcile_field_write(_contract: &TypeKnowledge, actual: &TypeKnowledge, relation: &RelationOutcome) -> FieldWriteReconciliation {
    let validity = match relation {
        RelationOutcome::Proven { .. } => match actual.status() {
            Some(EvidenceStatus::Established) => FieldContractValidity::Validated,
            Some(EvidenceStatus::Assumed) => FieldContractValidity::Assumed,
            None => FieldContractValidity::Unchecked,
        },
        RelationOutcome::Refuted(_) => FieldContractValidity::Refuted,
        RelationOutcome::Blocked(reason) => FieldContractValidity::Blocked(reason.clone()),
        RelationOutcome::DynamicBoundary(obligation) => FieldContractValidity::DynamicBoundary(obligation.clone()),
        RelationOutcome::Cancelled | RelationOutcome::BudgetExceeded(_) | RelationOutcome::InternalFailure(_) => FieldContractValidity::Unchecked,
    };
    FieldWriteReconciliation {
        current: actual.clone(),
        validity,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldLifecycleFact {
    pub field: FieldId,
    pub contract: TypeKnowledge,
    pub read_knowledge: TypeKnowledge,
    pub initialization: FieldInitialization,
    pub validity: FieldContractValidity,
    pub causal_invalidity: CausalInvalidity,
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
                validity: fact.validity.clone(),
                causal_invalidity: fact.causal_invalidity,
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
        let (read_knowledge, initialization, validity, causal_invalidity) = if let Some(default) = &field.default {
            let initializer = super::expression::synthesize_typed_expr(ctx, default);
            let application = ctx.apply_assignability(
                &initializer.knowledge,
                &contract,
                crate::diagnostic::DiagnosticCode::FieldMismatch,
                format!("default initializer does not match field `{}` type", field.name),
                field.range,
            );
            let reconciliation = reconcile_field_write(&contract, &initializer.knowledge, &application.outcome);
            let relation_causal = application.cause.map(CausalInvalidity::One).unwrap_or(CausalInvalidity::Clean);
            (
                reconciliation.current,
                FieldInitialization::DefinitelyInitialized,
                reconciliation.validity,
                initializer.causal_invalidity.join(relation_causal),
            )
        } else {
            (
                TypeKnowledge::Unknown(UnknownReason::MissingInitializer),
                FieldInitialization::Uninitialized,
                FieldContractValidity::Unchecked,
                CausalInvalidity::Clean,
            )
        };
        table.fields.insert(
            field_id.clone(),
            FieldLifecycleFact {
                field: field_id,
                contract,
                read_knowledge,
                initialization,
                validity,
                causal_invalidity,
            },
        );
    }
    table
}

pub(crate) fn lifecycle_read_knowledge(
    contract: &TypeKnowledge,
    initialization: FieldInitialization,
    validity: &FieldContractValidity,
    causal_invalidity: CausalInvalidity,
) -> TypeKnowledge {
    if initialization != FieldInitialization::DefinitelyInitialized {
        return TypeKnowledge::Unknown(UnknownReason::MissingInitializer);
    }
    if causal_invalidity != CausalInvalidity::Clean {
        return TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause);
    }
    match validity {
        FieldContractValidity::Validated => match contract.ty() {
            Some(ty) => TypeKnowledge::established(ty, EvidenceOrigin::FieldLifecycle),
            None => contract.clone(),
        },
        FieldContractValidity::Assumed => match contract.ty() {
            Some(ty) => TypeKnowledge::assumed(ty, EvidenceOrigin::FieldLifecycle),
            None => contract.clone(),
        },
        FieldContractValidity::Refuted => TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause),
        FieldContractValidity::Blocked(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
        FieldContractValidity::DynamicBoundary(_) => TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::ExplicitEscape),
        FieldContractValidity::Unchecked => TypeKnowledge::Unknown(UnknownReason::MissingInitializer),
    }
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
        let normal_exits = constructors
            .iter()
            .flat_map(|constructor| constructor.exits.normal_returns.iter())
            .collect::<Vec<_>>();
        if normal_exits.is_empty() {
            continue;
        }
        let definitely_initialized = normal_exits.iter().all(|exit| {
            exit.flow
                .fields
                .get(&fact.field)
                .is_some_and(|state| state.initialization == FieldInitialization::DefinitelyInitialized)
        });
        let uninitialized = normal_exits.iter().all(|exit| {
            exit.flow
                .fields
                .get(&fact.field)
                .is_some_and(|state| state.initialization == FieldInitialization::Uninitialized)
        });
        fact.initialization = if definitely_initialized {
            FieldInitialization::DefinitelyInitialized
        } else if uninitialized {
            FieldInitialization::Uninitialized
        } else {
            FieldInitialization::MaybeInitialized
        };
        fact.validity = crate::checker::flow::join_field_validity(normal_exits.iter().map(|exit| {
            exit.flow
                .fields
                .get(&fact.field)
                .map(|state| state.validity.clone())
                .unwrap_or(FieldContractValidity::Unchecked)
        }));
        fact.causal_invalidity = normal_exits
            .iter()
            .map(|exit| {
                let field_causal = exit
                    .flow
                    .fields
                    .get(&fact.field)
                    .map(|state| state.causal_invalidity)
                    .unwrap_or(CausalInvalidity::Clean);
                field_causal.join(exit.causal_invalidity)
            })
            .fold(CausalInvalidity::Clean, CausalInvalidity::join);
        fact.read_knowledge = lifecycle_read_knowledge(&fact.contract, fact.initialization, &fact.validity, fact.causal_invalidity);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::ModuleId;
    use crate::types::outcome::{BlockReason, DynamicBoundaryObligation, RelationEvidence, RelationFailure};
    use crate::types::store::TypeStore;

    #[test]
    fn reconcile_field_write_preserves_actual_and_derives_correct_validity() {
        let mut store = TypeStore::new();
        let int_ty = store.nominal_type(DeclarationId::new(ModuleId::core(), "Int".into()));
        let string_ty = store.nominal_type(DeclarationId::new(ModuleId::core(), "String".into()));
        let contract = TypeKnowledge::assumed(int_ty, EvidenceOrigin::DeveloperAnnotation);

        let established_int = TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax);
        let rec = reconcile_field_write(
            &contract,
            &established_int,
            &RelationOutcome::Proven {
                value: (),
                evidence: RelationEvidence::default(),
            },
        );
        assert_eq!(rec.current, established_int);
        assert_eq!(rec.validity, FieldContractValidity::Validated);

        let assumed_int = TypeKnowledge::assumed(int_ty, EvidenceOrigin::DeveloperAnnotation);
        let rec = reconcile_field_write(
            &contract,
            &assumed_int,
            &RelationOutcome::Proven {
                value: (),
                evidence: RelationEvidence::default(),
            },
        );
        assert_eq!(rec.current, assumed_int);
        assert_eq!(rec.validity, FieldContractValidity::Assumed);

        let established_string = TypeKnowledge::established(string_ty, EvidenceOrigin::Syntax);
        let rec = reconcile_field_write(
            &contract,
            &established_string,
            &RelationOutcome::Refuted(RelationFailure::TypeMismatch {
                actual: string_ty,
                expected: int_ty,
            }),
        );
        assert_eq!(rec.current, established_string);
        assert_eq!(rec.validity, FieldContractValidity::Refuted);

        let unknown = TypeKnowledge::Unknown(UnknownReason::MissingInitializer);
        let block_reason = BlockReason::UnknownType(UnknownReason::MissingInitializer);
        let rec = reconcile_field_write(&contract, &unknown, &RelationOutcome::Blocked(block_reason.clone()));
        assert_eq!(rec.current, unknown);
        assert_eq!(rec.validity, FieldContractValidity::Blocked(block_reason));

        let dynamic = TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::ExplicitEscape);
        let obligation = DynamicBoundaryObligation {
            reason: "dynamic write".into(),
        };
        let rec = reconcile_field_write(&contract, &dynamic, &RelationOutcome::DynamicBoundary(obligation.clone()));
        assert_eq!(rec.current, dynamic);
        assert_eq!(rec.validity, FieldContractValidity::DynamicBoundary(obligation));
    }
}
