use phalcom_semantic::explain::{ExplanationArena, ExplanationStep, PredicateKind, causal_slice};
use phalcom_semantic::identity::{BindingId, BodyId, ExpressionId, LocalExpressionId, TypeId};
use phalcom_semantic::types::evidence::{EvidenceOrigin, EvidenceStatus};

#[test]
fn test_explanation_arena_and_causal_slice() {
    let mut arena = ExplanationArena::new();

    let lit_expr = ExpressionId::new(BodyId(1), LocalExpressionId(0));
    let n1 = arena.alloc(
        ExplanationStep::Literal {
            expression: lit_expr,
            ty: TypeId(1),
        },
        EvidenceStatus::Established,
        EvidenceOrigin::Syntax,
        Vec::new(),
    );

    let n2 = arena.alloc(
        ExplanationStep::FlowRefinement {
            binding: BindingId(1),
            predicate: PredicateKind::IsInstance,
            prior: phalcom_semantic::types::evidence::TypeKnowledge::established(TypeId(1), EvidenceOrigin::Syntax),
            refined: phalcom_semantic::types::evidence::TypeKnowledge::established(TypeId(1), EvidenceOrigin::Flow),
        },
        EvidenceStatus::Established,
        EvidenceOrigin::Flow,
        vec![n1],
    );

    let n3 = arena.alloc(
        ExplanationStep::Subtyping {
            actual: TypeId(1),
            expected: TypeId(1),
            proven: true,
        },
        EvidenceStatus::Established,
        EvidenceOrigin::Flow,
        vec![n2],
    );

    assert_eq!(arena.len(), 3);

    let slice = causal_slice(&arena, n3);
    assert_eq!(slice.len(), 3);
    assert_eq!(slice[0].id, n3);
    assert_eq!(slice[1].id, n2);
    assert_eq!(slice[2].id, n1);
}
