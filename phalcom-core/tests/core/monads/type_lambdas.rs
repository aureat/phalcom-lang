use super::support::{Fixture, monads_source, with_monads};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::parameter::TypeParameterOwner;
use phalcom_semantic::types::store::TypeData;
use phalcom_semantic::types::type_lambda::ScopedTypeData;

/// MON-LAMBDA-01/02: the Either constructor lambda is canonical, binds `X`, and
/// captures outer `E` as a free declaration parameter.
#[test]
fn either_monad_supertype_lambda_binds_x_and_captures_e() {
    let f = Fixture::new(monads_source());
    f.assert_no_errors();

    let template = f.info("EitherMonad").supertype_template.as_ref().expect("EitherMonad superclass template");
    let args = f.assert_applied(template.supertype, "Monad", 1);
    let TypeData::Lambda(lambda_id) = f.analysis.snapshot.store.get(args[0]) else {
        panic!("expected EitherMonad to pass a canonical type lambda")
    };
    let lambda = f.analysis.snapshot.store.arena().get_lambda(*lambda_id);
    assert_eq!(lambda.parameter_kinds.as_ref(), [KindId::TYPE]);
    assert_eq!(lambda.result_kind, KindId::TYPE);

    let ScopedTypeData::Applied { origin, arguments } = f.analysis.snapshot.store.arena().get_scoped(lambda.body) else {
        panic!("expected lambda body to be Either<E, X>")
    };
    assert_eq!(arguments.len(), 2);
    assert!(
        matches!(f.analysis.snapshot.store.arena().get_scoped(*origin), ScopedTypeData::Free(ty) if *ty == f.ty("Either")),
        "lambda origin must be the free Either constructor"
    );
    assert!(
        matches!(f.analysis.snapshot.store.arena().get_scoped(arguments[0]), ScopedTypeData::Free(ty) if *ty == f.type_parameter_form("EitherMonad", 0)),
        "outer E must remain free inside the lambda"
    );
    assert!(
        matches!(
            f.analysis.snapshot.store.arena().get_scoped(arguments[1]),
            ScopedTypeData::Bound { depth: 0, index: 0 }
        ),
        "X must be represented by the lambda binder"
    );
}

/// MON-LAMBDA-03: projecting `EitherMonad<String>` substitutes the captured
/// outer `E` without touching lambda-bound `X`.
#[test]
fn receiver_specialization_substitutes_free_outer_parameter_inside_lambda() {
    let f = Fixture::new(monads_source());
    f.assert_no_errors();

    let string = f.ty("String");
    let (mut store, specialization) = f.specialize_receiver("EitherMonad", &[string], "Monad");
    let monad_f = store
        .find_type_parameter_id(&TypeParameterOwner::Declaration(f.decl("Monad")), 0)
        .expect("Monad.F parameter");
    let constructor = specialization.environment.get_param(monad_f).expect("specialized Monad.F");
    let TypeData::Lambda(lambda_id) = store.get(constructor) else {
        panic!("expected specialized Monad.F to remain a lambda")
    };
    let lambda = store.arena().get_lambda(*lambda_id).clone();
    let mut free = Vec::new();
    store.arena().collect_free_types(lambda.body, &mut free);

    let outer_e = store.parameter_form(f.type_parameter("EitherMonad", 0));
    assert!(free.contains(&string), "specialized lambda must capture String: {free:#?}");
    assert!(!free.contains(&outer_e), "specialized lambda must not retain EitherMonad.E: {free:#?}");
    assert!(
        store.arena().has_free_bound(lambda.body, 0),
        "lambda-bound X must remain present after outer substitution"
    );
}

/// MON-LAMBDA-04: applying the constructor obtained from receiver specialization
/// beta-reduces to the proper concrete Either type.
#[test]
fn specialized_constructor_beta_reduces_to_either_string_int() {
    let f = Fixture::new(monads_source());
    f.assert_no_errors();

    let string = f.ty("String");
    let int = f.ty("Int");
    let (mut store, specialization) = f.specialize_receiver("EitherMonad", &[string], "Monad");
    let monad_f = store
        .find_type_parameter_id(&TypeParameterOwner::Declaration(f.decl("Monad")), 0)
        .expect("Monad.F parameter");
    let constructor = specialization.environment.get_param(monad_f).expect("specialized Monad.F");
    let applied = store.apply_type_form(constructor, &[int]).expect("beta reduction of specialized constructor");

    let TypeData::Applied { origin, arguments } = store.get(applied) else {
        panic!("expected Either<String, Int>, got {:?}", store.get(applied))
    };
    assert_eq!(*origin, f.ty("Either"));
    assert_eq!(arguments.as_ref(), [string, int]);
}

/// MON-LAMBDA-05: alpha-renaming a closed constructor lambda does not change
/// its canonical semantic identity.
#[test]
fn alpha_renamed_constructor_lambdas_canonicalize_identically() {
    let source = with_monads(
        r#"
class AlphaX is Monad<<X> =>> Either<String, X>> {}
class AlphaY is Monad<<Y> =>> Either<String, Y>> {}
"#,
    );
    let f = Fixture::new(&source);
    f.assert_no_errors();

    let x_template = f.info("AlphaX").supertype_template.as_ref().expect("AlphaX superclass template");
    let y_template = f.info("AlphaY").supertype_template.as_ref().expect("AlphaY superclass template");
    let x_constructor = f.assert_applied(x_template.supertype, "Monad", 1)[0];
    let y_constructor = f.assert_applied(y_template.supertype, "Monad", 1)[0];

    assert_eq!(
        x_constructor, y_constructor,
        "alpha-renaming must not allocate a distinct canonical constructor type"
    );
    assert!(matches!(f.analysis.snapshot.store.get(x_constructor), TypeData::Lambda(_)));
}
