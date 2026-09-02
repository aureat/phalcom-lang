use crate::checker::analysis::AnalysisStatus;
use crate::checker::typed_expr::TypedExpression;
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::row::RecordRowTail;
use crate::types::store::{TypeData, TypeStore};

fn terminal_priority(status: &AnalysisStatus) -> u8 {
    match status {
        AnalysisStatus::InternalFailure(_) => 7,
        AnalysisStatus::Cancelled => 6,
        AnalysisStatus::BudgetExceeded(_) => 5,
        AnalysisStatus::Blocked(_) => 4,
        AnalysisStatus::Suppressed(_) => 3,
        AnalysisStatus::DynamicBoundary(_) => 2,
        AnalysisStatus::Invalid(_) => 1,
        AnalysisStatus::Ready => 0,
    }
}

fn push_unique<T: Eq + Copy>(items: &mut Vec<T>, value: T) {
    if !items.contains(&value) {
        items.push(value);
    }
}

fn select_terminal(selected: &mut Option<AnalysisStatus>, candidate: AnalysisStatus) {
    match selected {
        None => *selected = Some(candidate),
        Some(current) if terminal_priority(&candidate) > terminal_priority(current) => {
            *selected = Some(candidate);
        }
        Some(_) => {}
    }
}

pub(crate) fn propagate_required_dependencies(result: &mut TypedExpression, operands: &[TypedExpression]) {
    let mut selected_terminal: Option<AnalysisStatus> = None;

    for operand in operands {
        result.causal_invalidity = result.causal_invalidity.join(operand.causal_invalidity);
        for parent in &operand.explanation_parents {
            push_unique(&mut result.explanation_parents, *parent);
        }

        match &operand.status {
            AnalysisStatus::Invalid(_) if operand.knowledge.ty().is_some() => {}
            AnalysisStatus::Invalid(_) => {
                if let Some(cause) = result.causal_invalidity.suppression_cause() {
                    select_terminal(&mut selected_terminal, AnalysisStatus::Suppressed(cause));
                }
            }
            AnalysisStatus::Ready => {}
            status => select_terminal(&mut selected_terminal, status.clone()),
        }
    }

    if let Some(status) = selected_terminal {
        if terminal_priority(&status) >= terminal_priority(&result.status) {
            result.status = status;
        }
    }

    result.debug_assert_coherent();
}

pub(crate) fn project_applied_argument(store: &TypeStore, knowledge: &TypeKnowledge, expected_origin: TypeId, argument_index: usize) -> TypeKnowledge {
    match knowledge {
        TypeKnowledge::Known(_) => {
            let Some(source_ty) = knowledge.ty() else {
                unreachable!("Known knowledge has a type");
            };
            match store.get(source_ty) {
                TypeData::Applied { origin, arguments } if *origin == expected_origin => {
                    let Some(argument) = arguments.get(argument_index).copied() else {
                        return TypeKnowledge::Unknown(UnknownReason::UncheckedExpression);
                    };
                    knowledge.derive_known_type(argument, EvidenceOrigin::PatternDecomposition)
                }
                _ => TypeKnowledge::Unknown(UnknownReason::UncheckedExpression),
            }
        }
        TypeKnowledge::Unknown(reason) => TypeKnowledge::Unknown(reason.clone()),
        TypeKnowledge::Dynamic(reason) => TypeKnowledge::Dynamic(reason.clone()),
    }
}

pub(crate) fn project_tuple_elements(store: &TypeStore, knowledge: &TypeKnowledge) -> Result<Vec<TypeKnowledge>, TypeKnowledge> {
    match knowledge {
        TypeKnowledge::Known(_) => {
            let Some(source_ty) = knowledge.ty() else {
                unreachable!("Known knowledge has a type");
            };
            match store.get(source_ty) {
                TypeData::Tuple(elements) => Ok(elements
                    .iter()
                    .map(|element| knowledge.derive_known_type(element.ty, EvidenceOrigin::PatternDecomposition))
                    .collect()),
                _ => Err(TypeKnowledge::Unknown(UnknownReason::UncheckedExpression)),
            }
        }
        TypeKnowledge::Unknown(reason) => Err(TypeKnowledge::Unknown(reason.clone())),
        TypeKnowledge::Dynamic(reason) => Err(TypeKnowledge::Dynamic(reason.clone())),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecordProjection {
    pub fields: Vec<(Box<str>, TypeKnowledge)>,
    pub tail: RecordRowTail,
}

/// Projects statically known Record fields without pretending an open row is
/// complete. The returned tail is owned so callers may continue analysis
/// without holding a borrow into `TypeStore`.
pub(crate) fn project_record_shape(store: &TypeStore, knowledge: &TypeKnowledge) -> Result<RecordProjection, TypeKnowledge> {
    match knowledge {
        TypeKnowledge::Known(_) => {
            let Some(source_ty) = knowledge.ty() else {
                unreachable!("Known knowledge has a type");
            };
            match store.get(source_ty) {
                TypeData::Record(row_id) => {
                    let row = store.record_row(*row_id);
                    if matches!(row.tail, RecordRowTail::Closed) {
                        return project_complete_record_fields(store, knowledge).map(|fields| RecordProjection {
                            fields,
                            tail: RecordRowTail::Closed,
                        });
                    }
                    Ok(RecordProjection {
                        fields: row
                            .fields
                            .iter()
                            .map(|field| (field.name.clone(), knowledge.derive_known_type(field.ty, EvidenceOrigin::PatternDecomposition)))
                            .collect(),
                        tail: row.tail,
                    })
                }
                _ => Err(TypeKnowledge::Unknown(UnknownReason::UncheckedExpression)),
            }
        }
        TypeKnowledge::Unknown(reason) => Err(TypeKnowledge::Unknown(reason.clone())),
        TypeKnowledge::Dynamic(reason) => Err(TypeKnowledge::Dynamic(reason.clone())),
    }
}

/// Complete field enumeration is only sound for a closed Record row.
pub(crate) fn project_complete_record_fields(store: &TypeStore, knowledge: &TypeKnowledge) -> Result<Vec<(Box<str>, TypeKnowledge)>, TypeKnowledge> {
    match knowledge {
        TypeKnowledge::Known(_) => {
            let Some(source_ty) = knowledge.ty() else {
                unreachable!("Known knowledge has a type");
            };
            match store.get(source_ty) {
                TypeData::Record(row_id) => {
                    let row = store.record_row(*row_id);
                    if !matches!(row.tail, RecordRowTail::Closed) {
                        return Err(TypeKnowledge::Unknown(UnknownReason::UncheckedExpression));
                    }
                    Ok(row
                        .fields
                        .iter()
                        .map(|field| (field.name.clone(), knowledge.derive_known_type(field.ty, EvidenceOrigin::PatternDecomposition)))
                        .collect())
                }
                _ => Err(TypeKnowledge::Unknown(UnknownReason::UncheckedExpression)),
            }
        }
        TypeKnowledge::Unknown(reason) => Err(TypeKnowledge::Unknown(reason.clone())),
        TypeKnowledge::Dynamic(reason) => Err(TypeKnowledge::Dynamic(reason.clone())),
    }
}

/// Projects one known prefix field. Open rows are safe here because a known
/// prefix is guaranteed; absence from the prefix remains unknown.
pub(crate) fn lookup_record_field(store: &TypeStore, knowledge: &TypeKnowledge, name: &str) -> Result<TypeKnowledge, TypeKnowledge> {
    match knowledge {
        TypeKnowledge::Known(_) => {
            let Some(source_ty) = knowledge.ty() else {
                unreachable!("Known knowledge has a type");
            };
            match store.get(source_ty) {
                TypeData::Record(row_id) => {
                    let row = store.record_row(*row_id);
                    match row.find_field(name) {
                        Some(field_ty) => Ok(knowledge.derive_known_type(field_ty, EvidenceOrigin::PatternDecomposition)),
                        None => Err(TypeKnowledge::Unknown(UnknownReason::UncheckedExpression)),
                    }
                }
                _ => Err(TypeKnowledge::Unknown(UnknownReason::UncheckedExpression)),
            }
        }
        TypeKnowledge::Unknown(reason) => Err(TypeKnowledge::Unknown(reason.clone())),
        TypeKnowledge::Dynamic(reason) => Err(TypeKnowledge::Dynamic(reason.clone())),
    }
}

pub(crate) fn decompose_tuple_component(store: &TypeStore, parent: &TypeKnowledge, index: usize, expected_len: usize) -> TypeKnowledge {
    match parent {
        TypeKnowledge::Known(_) => {
            let Some(parent_ty) = parent.ty() else {
                unreachable!("Known knowledge has a type");
            };
            match store.get(parent_ty) {
                TypeData::Tuple(elements) if elements.len() == expected_len => {
                    let Some(element) = elements.get(index) else {
                        return TypeKnowledge::Unknown(UnknownReason::UncheckedExpression);
                    };
                    parent.derive_known_type(element.ty, EvidenceOrigin::PatternDecomposition)
                }
                _ => TypeKnowledge::Unknown(UnknownReason::UncheckedExpression),
            }
        }
        TypeKnowledge::Unknown(reason) => TypeKnowledge::Unknown(reason.clone()),
        TypeKnowledge::Dynamic(reason) => TypeKnowledge::Dynamic(reason.clone()),
    }
}

/// Projects the homogeneous element knowledge from a formal `List<T>` value.
///
/// Unavailable and dynamic parents retain their exact epistemic reason; a
/// malformed known shape does not manufacture an element type.
pub(crate) fn decompose_list_element(store: &TypeStore, parent: &TypeKnowledge, list_origin: TypeId) -> TypeKnowledge {
    project_applied_argument(store, parent, list_origin, 0)
}

/// Projects the rest binding from a formal `List<T>` value.
///
/// The rest value has the same `List<T>` shape and authority as its parent.
/// Only a known application of the canonical List origin is decomposed.
pub(crate) fn decompose_list_rest(store: &TypeStore, parent: &TypeKnowledge, list_origin: TypeId) -> TypeKnowledge {
    match parent {
        TypeKnowledge::Known(_) => {
            let Some(parent_ty) = parent.ty() else {
                unreachable!("Known knowledge has a type");
            };
            match store.get(parent_ty) {
                TypeData::Applied { origin, arguments } if *origin == list_origin && arguments.len() == 1 => {
                    parent.derive_known_type(parent_ty, EvidenceOrigin::PatternDecomposition)
                }
                _ => TypeKnowledge::Unknown(UnknownReason::UncheckedExpression),
            }
        }
        TypeKnowledge::Unknown(reason) => TypeKnowledge::Unknown(reason.clone()),
        TypeKnowledge::Dynamic(reason) => TypeKnowledge::Dynamic(reason.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::causal::{CausalInvalidity, SuppressionCause};
    use crate::identity::{DiagnosticCauseId, InternalSemanticIncidentId};
    use crate::types::evidence::{DynamicReason, EvidenceOrigin, UnknownReason};
    use crate::types::id::TypeId;
    use crate::types::outcome::{BlockReason, BudgetKind, BudgetReport};
    use phalcom_common::range::SourceRange;

    fn known() -> TypedExpression {
        TypedExpression::established(TypeId::DUMMY, EvidenceOrigin::Syntax, SourceRange::default())
    }

    fn result() -> TypedExpression {
        TypedExpression::new(crate::types::evidence::TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence))
    }

    #[test]
    fn invalid_known_dependency_keeps_parent_ready_and_causal() {
        let cause = DiagnosticCauseId(1);
        let mut operand = known();
        operand.invalidate(cause);
        let mut parent = result();

        propagate_required_dependencies(&mut parent, &[operand]);

        assert!(matches!(parent.status, AnalysisStatus::Ready));
        assert!(parent.causal_invalidity.contains(cause));
    }

    #[test]
    fn invalid_unavailable_dependency_suppresses_parent() {
        let cause = DiagnosticCauseId(1);
        let mut operand = TypedExpression::unknown(UnknownReason::UnresolvedName("missing".into()));
        operand.invalidate(cause);
        let mut parent = result();

        propagate_required_dependencies(&mut parent, &[operand]);

        assert!(matches!(parent.status, AnalysisStatus::Suppressed(SuppressionCause::One(actual)) if actual == cause));
        assert!(parent.causal_invalidity.contains(cause));
    }

    #[test]
    fn suppressed_dependency_suppresses_parent() {
        let cause = DiagnosticCauseId(1);
        let mut operand = TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause);
        operand.status = AnalysisStatus::Suppressed(SuppressionCause::One(cause));
        operand.causal_invalidity = CausalInvalidity::One(cause);
        let mut parent = result();

        propagate_required_dependencies(&mut parent, &[operand]);

        assert!(matches!(parent.status, AnalysisStatus::Suppressed(_)));
    }

    #[test]
    fn higher_priority_terminal_status_wins() {
        let mut cancelled = TypedExpression::new(crate::types::evidence::TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape));
        cancelled.status = AnalysisStatus::Cancelled;
        let mut internal = known();
        internal.status = AnalysisStatus::InternalFailure(InternalSemanticIncidentId(2));
        let mut parent = result();

        propagate_required_dependencies(&mut parent, &[cancelled, internal]);

        assert!(matches!(parent.status, AnalysisStatus::InternalFailure(InternalSemanticIncidentId(2))));
    }

    #[test]
    fn budget_and_cancelled_statuses_propagate() {
        let mut budget = known();
        budget.status = AnalysisStatus::BudgetExceeded(BudgetReport::new(BudgetKind::Steps, 1, 2));
        let mut parent = result();
        propagate_required_dependencies(&mut parent, &[budget]);
        assert!(matches!(parent.status, AnalysisStatus::BudgetExceeded(_)));

        let mut cancelled = known();
        cancelled.status = AnalysisStatus::Cancelled;
        let mut parent = result();
        propagate_required_dependencies(&mut parent, &[cancelled]);
        assert!(matches!(parent.status, AnalysisStatus::Cancelled));

        let mut blocked = known();
        blocked.status = AnalysisStatus::Blocked(BlockReason::RecursiveFixpoint);
        let mut parent = result();
        propagate_required_dependencies(&mut parent, &[blocked]);
        assert!(matches!(parent.status, AnalysisStatus::Blocked(BlockReason::RecursiveFixpoint)));
    }

    #[test]
    fn tuple_decomposition_preserves_unknown_and_dynamic_reasons() {
        let store = TypeStore::new();
        let unknown = TypeKnowledge::Unknown(UnknownReason::UnresolvedName("missing".into()));
        assert_eq!(decompose_tuple_component(&store, &unknown, 0, 2), unknown);

        let dynamic = TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape);
        assert_eq!(decompose_tuple_component(&store, &dynamic, 0, 2), dynamic);
    }

    #[test]
    fn list_decomposition_preserves_unknown_and_dynamic_reasons() {
        let store = TypeStore::new();
        let unknown = TypeKnowledge::Unknown(UnknownReason::UnresolvedName("missing".into()));
        assert_eq!(decompose_list_element(&store, &unknown, TypeId::DUMMY), unknown);
        assert_eq!(decompose_list_rest(&store, &unknown, TypeId::DUMMY), unknown);

        let dynamic = TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape);
        assert_eq!(decompose_list_element(&store, &dynamic, TypeId::DUMMY), dynamic);
        assert_eq!(decompose_list_rest(&store, &dynamic, TypeId::DUMMY), dynamic);
    }

    #[test]
    fn list_decomposition_preserves_known_parent_authority() {
        let mut store = TypeStore::new();
        let declarations = crate::declarations::bootstrap_universe_declarations(&mut store, &|key| {
            crate::identity::DeclarationId::new(crate::identity::ModuleId::universe_root(), key.name().into())
        });
        let list_origin = declarations
            .form(&crate::identity::DeclarationId::new(crate::identity::ModuleId::universe_root(), "List".into()))
            .expect("List form");
        let int_ty = declarations
            .form(&crate::identity::DeclarationId::new(crate::identity::ModuleId::universe_root(), "Int".into()))
            .expect("Int form");
        let list_ty = store.apply_type_form(list_origin, &[int_ty]).expect("List<Int>");
        let parent = TypeKnowledge::established(list_ty, EvidenceOrigin::Syntax);

        let element = decompose_list_element(&store, &parent, list_origin);
        assert_eq!(element.ty(), Some(int_ty));
        assert_eq!(element.status(), parent.status());
        assert_eq!(element.origin(), Some(EvidenceOrigin::PatternDecomposition));

        let rest = decompose_list_rest(&store, &parent, list_origin);
        assert_eq!(rest.ty(), Some(list_ty));
        assert_eq!(rest.status(), parent.status());
        assert_eq!(rest.origin(), Some(EvidenceOrigin::PatternDecomposition));
    }
}
