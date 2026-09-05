use super::super::support::{Fixture, expression_semantic_source as semantic_source};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::EvidenceStatus;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::parameter::TypeParameterOwner;
use phalcom_semantic::types::store::{TypeData, TypeStore};
use phalcom_semantic::types::type_lambda::ScopedTypeData;

fn collect_transitive_free_types(store: &TypeStore, body: phalcom_semantic::types::id::ScopedTypeId) -> Vec<phalcom_semantic::types::id::TypeId> {
    let mut free = Vec::new();
    store.arena().collect_free_types(body, &mut free);
    let mut index = 0;
    while index < free.len() {
        let ty = free[index];
        if let TypeData::Lambda(lambda_id) = store.get(ty) {
            let nested_body = store.arena().get_lambda(*lambda_id).body;
            let mut nested = Vec::new();
            store.arena().collect_free_types(nested_body, &mut nested);
            for nested_ty in nested {
                if !free.contains(&nested_ty) {
                    free.push(nested_ty);
                }
            }
        }
        index += 1;
    }
    free
}

/// GEX-06/GEX-07: ExpressionMonad exposes a canonical unary constructor,
/// captures its outer F, and projects through the existing Monad hierarchy.
#[test]
fn expression_monad_captures_effect_constructor_and_projects_hierarchy() {
    let f = Fixture::new(&semantic_source());
    f.assert_no_errors();

    let run = f.callable("ExpressionMonadSemanticProbe", "constructorOperations", DispatchSide::Class);
    let pure = f.binding(run, "pure").current.ty().expect("pure expression type");
    let pure_args = f.assert_applied(pure, "Expression", 2).to_vec();
    assert_eq!(pure_args[1], f.ty("Int"));
    assert!(matches!(f.analysis.snapshot.store.get(pure_args[0]), TypeData::Lambda(_)));

    let mapped = f.binding(run, "mapped").current.ty().expect("mapped expression type");
    let mapped_args = f.assert_applied(mapped, "Expression", 2);
    assert_eq!(mapped_args[0], pure_args[0]);
    assert_eq!(mapped_args[1], f.ty("Bool"));

    let bound = f.binding(run, "bound").current.ty().expect("bound expression type");
    let bound_args = f.assert_applied(bound, "Expression", 2);
    assert_eq!(bound_args[0], pure_args[0]);
    assert_eq!(bound_args[1], f.ty("Bool"));

    let bind_call = f.expression_containing(run, "MonadAlgorithms.bind(");
    let bind_target = f.callable_id("MonadAlgorithms", "bind", DispatchSide::Class);
    f.assert_expression_call(bind_call, &bind_target, bound);

    let constructor_parameter = f.callable_generic_parameter("MonadAlgorithms", "bind", DispatchSide::Class, 0);
    let constructor = f.generic_solution_type_for(run, bind_call, constructor_parameter);
    f.assert_unary_constructor_kind(f.analysis.snapshot.store.kind_of(constructor));
    let TypeData::Lambda(lambda_id) = f.analysis.snapshot.store.get(constructor) else {
        panic!("Expression constructor must solve to a type lambda");
    };
    let lambda = f.analysis.snapshot.store.arena().get_lambda(*lambda_id);
    assert_eq!(lambda.parameter_kinds.as_ref(), [KindId::TYPE]);
    assert_eq!(lambda.result_kind, KindId::TYPE);
    let free = collect_transitive_free_types(&f.analysis.snapshot.store, lambda.body);
    assert!(free.contains(&f.ty("Expression")), "constructor must capture Expression: {free:#?}");
    assert!(free.contains(&f.ty("Either")), "nested constructor must retain Either: {free:#?}");
    assert!(f.analysis.snapshot.store.arena().has_free_bound(lambda.body, 0));
    f.assert_generic_solution_exact(run, bind_call, constructor_parameter, constructor, EvidenceStatus::Assumed);

    let (_full_store, full_specialization) = f.specialize_receiver("StringEitherExpressionMonad", &[], "Functor");
    let full_owners = full_specialization.path.iter().map(|step| step.owner.clone()).collect::<Vec<_>>();
    assert_eq!(
        full_owners,
        [
            f.decl("StringEitherExpressionMonad"),
            f.decl("ExpressionMonad"),
            f.decl("Monad"),
            f.decl("Applicative"),
            f.decl("Functor")
        ]
    );

    let (mut store, specialization) = f.specialize_receiver("StringEitherExpressionMonad", &[], "Monad");
    let owners = specialization.path.iter().map(|step| step.owner.clone()).collect::<Vec<_>>();
    assert_eq!(owners, [f.decl("StringEitherExpressionMonad"), f.decl("ExpressionMonad"), f.decl("Monad")]);
    let monad_f = store
        .find_type_parameter_id(&TypeParameterOwner::Declaration(f.decl("Monad")), 0)
        .expect("Monad.F parameter");
    let expression_constructor = specialization.environment.get_param(monad_f).expect("specialized Expression constructor");
    f.assert_unary_constructor_kind(store.kind_of(expression_constructor));
    let TypeData::Lambda(lambda_id) = store.get(expression_constructor) else {
        panic!("specialized Expression constructor must remain a lambda");
    };
    let lambda = store.arena().get_lambda(*lambda_id);
    let ScopedTypeData::Applied { origin, arguments } = store.arena().get_scoped(lambda.body) else {
        panic!("specialized lambda body must be Expression<F, X>");
    };
    assert!(matches!(store.arena().get_scoped(*origin), ScopedTypeData::Free(ty) if *ty == f.ty("Expression")));
    assert!(matches!(store.arena().get_scoped(arguments[1]), ScopedTypeData::Bound { depth: 0, index: 0 }));

    let applied = store
        .apply_type_form(expression_constructor, &[f.ty("Int")])
        .expect("Expression constructor must beta-reduce");
    assert_eq!(applied, pure, "Monad F must be nested Expression over supplied Either family");
    let TypeData::Applied { origin, arguments } = store.get(applied) else {
        panic!("expected Expression<..., Int>, got {}", store.format_type(applied));
    };
    assert_eq!(*origin, f.ty("Expression"));
    assert_eq!(arguments[1], f.ty("Int"));
}

/// INT-02: the existing generic traverse algorithm operates over the
/// Expression constructor without an Expression-specific algorithm copy.
#[test]
fn existing_traverse_infers_expression_constructor_and_exact_list_result() {
    let f = Fixture::new(&semantic_source());
    f.assert_no_errors();

    let run = f.callable("ExpressionMonadSemanticProbe", "traverseConstruction", DispatchSide::Class);
    let expression = f.binding(run, "expression").current.ty().expect("traverse expression type");
    let expression_args = f.assert_applied(expression, "Expression", 2);
    let list = f.assert_applied(expression_args[1], "List", 1);
    assert_eq!(list, [f.ty("Int")]);

    let call = f.expression_containing(run, "MonadAlgorithms.traverse(");
    let target = f.callable_id("MonadAlgorithms", "traverse", DispatchSide::Class);
    f.assert_expression_call(call, &target, expression);

    let constructor_parameter = f.callable_generic_parameter("MonadAlgorithms", "traverse", DispatchSide::Class, 0);
    let a = f.callable_generic_parameter("MonadAlgorithms", "traverse", DispatchSide::Class, 1);
    let b = f.callable_generic_parameter("MonadAlgorithms", "traverse", DispatchSide::Class, 2);
    let constructor = f.generic_solution_type_for(run, call, constructor_parameter);
    f.assert_unary_constructor_kind(f.analysis.snapshot.store.kind_of(constructor));
    f.assert_generic_solution_exact(run, call, constructor_parameter, constructor, EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, a, f.ty("Int"), EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, b, f.ty("Int"), EvidenceStatus::Assumed);
}
