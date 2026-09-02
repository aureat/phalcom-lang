use super::support::{Fixture, monads_source};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::store::TypeData;

/// MON-KIND-01: `F: Type -> Type` publishes a constructor-kinded declaration parameter.
#[test]
fn functor_parameter_has_explicit_unary_constructor_kind() {
    let f = Fixture::new(monads_source());
    f.assert_no_errors();

    let kind = f.parameter_kind("Functor", 0);
    f.assert_unary_constructor_kind(kind);
    assert_ne!(kind, KindId::TYPE, "F must not collapse to a proper-type parameter");
}

/// MON-KIND-02: an ordinary unary nominal constructor can inhabit `F: Type -> Type`.
#[test]
fn unary_nominal_constructor_can_specialize_monad() {
    let f = Fixture::new(monads_source());
    f.assert_no_errors();

    let template = f.info("BoxMonad").supertype_template.as_ref().expect("BoxMonad superclass template");
    let args = f.assert_applied(template.supertype, "Monad", 1);
    f.assert_nominal(args[0], "Box");
    f.assert_unary_constructor_kind(f.analysis.snapshot.store.kind_of(args[0]));
}

/// MON-KIND-03: a type lambda of kind `Type -> Type` can inhabit the same constructor parameter.
#[test]
fn type_lambda_constructor_can_specialize_monad() {
    let f = Fixture::new(monads_source());
    f.assert_no_errors();

    let template = f.info("EitherMonad").supertype_template.as_ref().expect("EitherMonad superclass template");
    let args = f.assert_applied(template.supertype, "Monad", 1);
    assert!(matches!(f.analysis.snapshot.store.get(args[0]), TypeData::Lambda(_)), "expected type-lambda argument");
    f.assert_unary_constructor_kind(f.analysis.snapshot.store.kind_of(args[0]));
}
