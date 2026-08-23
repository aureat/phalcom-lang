use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_semantic::explain::{DerivationRule, EvidenceRef, ExplanationArena, ExplanationStep, PredicateKind, causal_slice};
use phalcom_semantic::identity::{
    BindingId, BodyId, CallableId, DeclarationId, DispatchSide, ExplanationId, ExpressionId, LocalExpressionId, ModuleId, TypeId,
};
use phalcom_semantic::types::evidence::EvidenceAuthority;

const RANGE: SourceRange = SourceRange { start: 0, end: 10 };

#[test]
fn test_explanation_graph_derivation_rules() {
    let mut arena = ExplanationArena::new();

    let lit_expr = ExpressionId::new(BodyId(1), LocalExpressionId(0));
    let lit_step = ExplanationStep::Literal {
        expression: lit_expr,
        ty: TypeId(1),
    };
    let n1 = arena.alloc_full(
        lit_step.clone(),
        DerivationRule::LiteralSynthesis,
        EvidenceAuthority::ExactSyntax,
        vec![EvidenceRef::SourceSpan(RANGE), EvidenceRef::TypeId(TypeId(1))],
        Vec::new(),
    );

    let call_step = ExplanationStep::MethodCall {
        call: ExpressionId::new(BodyId(1), LocalExpressionId(1)),
        callable: CallableId::new(
            DeclarationId::new(ModuleId::core(), "Number".into()),
            Selector::getter("plus").unwrap(),
            DispatchSide::Instance,
        ),
        return_ty: TypeId(2),
    };
    let n2 = arena.alloc_full(
        call_step,
        DerivationRule::MethodCallReturn { selector: "plus".into() },
        EvidenceAuthority::Proven,
        vec![EvidenceRef::TypeId(TypeId(2))],
        vec![n1],
    );

    let flow_step = ExplanationStep::FlowRefinement {
        binding: BindingId(1),
        prior: phalcom_semantic::types::evidence::TypeKnowledge::known(TypeId(2), EvidenceAuthority::ExactSyntax),
        refined: phalcom_semantic::types::evidence::TypeKnowledge::known(TypeId(2), EvidenceAuthority::Proven),
    };
    let n3 = arena.alloc_full(
        flow_step,
        DerivationRule::FlowRefinement {
            predicate_kind: PredicateKind::IsInstance,
        },
        EvidenceAuthority::Proven,
        vec![EvidenceRef::BindingVersion {
            binding: BindingId(1),
            version: 1,
        }],
        vec![n2],
    );

    assert_eq!(arena.len(), 3);

    let node3 = arena.get(n3).unwrap();
    assert_eq!(
        node3.rule,
        DerivationRule::FlowRefinement {
            predicate_kind: PredicateKind::IsInstance
        }
    );
    assert_eq!(node3.parents, vec![n2]);

    let slice = causal_slice(&arena, n3);
    assert_eq!(slice.len(), 3);
    assert_eq!(slice[0].id, n3);
    assert_eq!(slice[1].id, n2);
    assert_eq!(slice[2].id, n1);
}
