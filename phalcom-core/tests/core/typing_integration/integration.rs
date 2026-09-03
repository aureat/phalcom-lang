use super::support::{Fixture, integration_semantic_source};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::EvidenceStatus;
use phalcom_semantic::types::parameter::TypeParameterOwner;
use phalcom_semantic::types::store::TypeData;

/// INT-00: direct Either application and MonadAlgorithms.bind must consume
/// one canonical Either declaration in one semantic analysis.
#[test]
fn direct_either_and_monad_paths_share_one_canonical_source() {
    let f = Fixture::new(&integration_semantic_source());
    f.assert_no_errors();

    let run = f.callable("UnifiedTypingProbe", "run", DispatchSide::Class);
    let mapped = f.binding(run, "mapped").current.ty().expect("mapped type");
    let bound = f.binding(run, "bound").current.ty().expect("bound type");
    f.assert_either(mapped, f.ty("String"), f.ty("Bool"));
    f.assert_either(bound, f.ty("String"), f.ty("Bool"));

    let direct_call = f.expression_containing(run, "source.map(");
    let direct_target = f.callable_id("Either", "map", DispatchSide::Instance);
    f.assert_expression_call(direct_call, &direct_target, mapped);

    let bind_call = f.expression_containing(run, "MonadAlgorithms.bind(");
    let bind_target = f.callable_id("MonadAlgorithms", "bind", DispatchSide::Class);
    f.assert_expression_call(bind_call, &bind_target, bound);

    let constructor_parameter = f.callable_generic_parameter("MonadAlgorithms", "bind", DispatchSide::Class, 0);
    let a = f.callable_generic_parameter("MonadAlgorithms", "bind", DispatchSide::Class, 1);
    let b = f.callable_generic_parameter("MonadAlgorithms", "bind", DispatchSide::Class, 2);
    let constructor = f.generic_solution_type_for(run, bind_call, constructor_parameter);
    f.assert_unary_constructor_kind(f.analysis.snapshot.store.kind_of(constructor));
    let TypeData::Lambda(lambda_id) = f.analysis.snapshot.store.get(constructor) else {
        panic!("F must remain a canonical unary type lambda");
    };
    let lambda = f.analysis.snapshot.store.arena().get_lambda(*lambda_id);
    let mut free = Vec::new();
    f.analysis.snapshot.store.arena().collect_free_types(lambda.body, &mut free);
    assert!(free.contains(&f.ty("String")), "F must capture String: {free:#?}");
    assert!(f.analysis.snapshot.store.arena().has_free_bound(lambda.body, 0), "F must retain its bound argument");
    f.assert_generic_solution_exact(run, bind_call, constructor_parameter, constructor, EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, bind_call, a, f.ty("Int"), EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, bind_call, b, f.ty("Bool"), EvidenceStatus::Established);
    assert!(matches!(
        f.analysis.snapshot.store.type_parameter(a).owner,
        TypeParameterOwner::Callable(ref owner) if owner == &bind_target
    ));
    assert!(matches!(
        f.analysis.snapshot.store.type_parameter(b).owner,
        TypeParameterOwner::Callable(ref owner) if owner == &bind_target
    ));

    assert_eq!(f.family_type(bound), f.family_type(mapped), "direct and Monad results must share canonical Either family");
    assert_eq!(f.decl("Either"), direct_target.owner, "direct call must target canonical Either declaration");
}
