use phalcom_common::range::SourceRange;
use phalcom_semantic::explain::{DerivationRule, EvidenceRef, ExplanationArena, ExplanationStep, PredicateKind, causal_slice, causal_trace};
use phalcom_semantic::identity::{BindingId, BodyId, ExpressionId, LocalExpressionId, TypeId};
use phalcom_semantic::types::evidence::{EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use phalcom_semantic::types::outcome::{BlockReason, RelationFailure, RelationOutcome};

const RANGE: SourceRange = SourceRange { start: 0, end: 10 };

fn known(ty: TypeId) -> TypeKnowledge {
    TypeKnowledge::established(ty, EvidenceOrigin::Syntax)
}

#[test]
fn causal_trace_is_deterministic_parent_first_and_deduplicated() {
    let mut arena = ExplanationArena::new();
    let expr = ExpressionId::new(BodyId(1), LocalExpressionId(0));

    let shared = arena.alloc_full(
        ExplanationStep::Literal {
            expression: expr,
            ty: TypeId(1),
        },
        DerivationRule::LiteralSynthesis,
        EvidenceStatus::Established,
        EvidenceOrigin::Syntax,
        vec![EvidenceRef::SourceSpan(RANGE), EvidenceRef::TypeId(TypeId(1))],
        Vec::new(),
    );
    let left = arena.alloc(
        ExplanationStep::Declared {
            binding: Some(BindingId(1)),
            range: RANGE,
            ty: TypeId(1),
        },
        EvidenceStatus::Established,
        EvidenceOrigin::DeveloperAnnotation,
        vec![shared],
    );
    let right = arena.alloc(
        ExplanationStep::TypeRequirement {
            expected: TypeId(2),
            origin: phalcom_semantic::checker::expected::ExpectationOrigin::ExplicitCheck,
            source: None,
        },
        EvidenceStatus::Established,
        EvidenceOrigin::DeveloperAnnotation,
        vec![shared],
    );
    let root = arena.alloc(
        ExplanationStep::TypeRelation {
            actual: known(TypeId(1)),
            expected: TypeId(2),
            outcome: RelationOutcome::Refuted(RelationFailure::TypeMismatch {
                actual: TypeId(1),
                expected: TypeId(2),
            }),
        },
        EvidenceStatus::Established,
        EvidenceOrigin::Flow,
        vec![left, right],
    );

    let trace = causal_trace(&arena, root);
    assert_eq!(trace.iter().map(|node| node.id).collect::<Vec<_>>(), vec![shared, left, right, root]);
    assert_eq!(trace.iter().filter(|node| node.id == shared).count(), 1);
    assert_eq!(trace.last().unwrap().id, root);

    // Compatibility slice keeps the historical root-first view.
    assert_eq!(causal_slice(&arena, root).first().unwrap().id, root);
}

#[test]
fn type_relation_preserves_refuted_and_blocked_outcomes() {
    let mut arena = ExplanationArena::new();
    let refuted = arena.alloc(
        ExplanationStep::TypeRelation {
            actual: known(TypeId(1)),
            expected: TypeId(2),
            outcome: RelationOutcome::Refuted(RelationFailure::TypeMismatch {
                actual: TypeId(1),
                expected: TypeId(2),
            }),
        },
        EvidenceStatus::Established,
        EvidenceOrigin::Flow,
        Vec::new(),
    );
    let blocked = arena.alloc(
        ExplanationStep::TypeRelation {
            actual: TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration),
            expected: TypeId(2),
            outcome: RelationOutcome::Blocked(BlockReason::UnknownType(UnknownReason::UnannotatedDeclaration)),
        },
        EvidenceStatus::Assumed,
        EvidenceOrigin::Flow,
        Vec::new(),
    );

    assert!(matches!(
        &arena.get(refuted).unwrap().step,
        ExplanationStep::TypeRelation {
            outcome: RelationOutcome::Refuted(_),
            ..
        }
    ));
    assert!(matches!(
        &arena.get(blocked).unwrap().step,
        ExplanationStep::TypeRelation {
            outcome: RelationOutcome::Blocked(_),
            ..
        }
    ));
}

#[test]
fn flow_refinement_keeps_actual_predicate_kind() {
    let mut arena = ExplanationArena::new();
    let node = arena.alloc(
        ExplanationStep::FlowRefinement {
            binding: BindingId(1),
            predicate: PredicateKind::Falsy,
            prior: known(TypeId(1)),
            refined: known(TypeId(2)),
        },
        EvidenceStatus::Established,
        EvidenceOrigin::Flow,
        Vec::new(),
    );

    assert_eq!(
        arena.get(node).unwrap().rule,
        DerivationRule::FlowRefinement {
            predicate_kind: PredicateKind::Falsy
        }
    );
}
