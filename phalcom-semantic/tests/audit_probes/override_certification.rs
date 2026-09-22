//! Adversarial override certification and static proof invariant probes.

use crate::semantic::support::Fixture;

#[test]
fn incompatible_override_must_not_certify_int() {
    let f = Fixture::new(
        r#"
class Base { value() -> Int { return 1 } }
class Child is Base {
  @constructor
  new() {}
  value() -> String { return "wrong" }
}
class Probe {
  @class
  consume(_ x: Base) -> Int { return x.value() }
}
"#,
    );
    assert!(
        f.analysis.snapshot.has_errors(),
        "an incompatible String override must not satisfy a Base Int contract"
    );
}

#[test]
fn narrowing_override_parameter_is_rejected() {
    let f = Fixture::new(
        r#"
class Base { accept(_ value: Object) -> Int { return 1 } }
class Child is Base { accept(_ value: Int) -> Int { return value } }
"#,
    );
    assert!(f.analysis.snapshot.has_errors(), "an override must not narrow an inherited parameter");
}

#[test]
fn widening_override_parameter_is_accepted() {
    let f = Fixture::new(
        r#"
class Base { accept(_ value: Int) -> Object { return value } }
class Child is Base { accept(_ value: Object) -> Int { return 1 } }
"#,
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn unannotated_override_does_not_reject_body_derived_return() {
    let f = Fixture::new(
        r#"
class Base { toString() -> String { return "base" } }
class Child is Base {
  _value
  @constructor new(_ value) { _value = value }
  toString { "Child(" + _value + ")" }
}
"#,
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn alpha_equivalent_generic_hkt_override_is_accepted() {
    let f = Fixture::new(
        r#"
class Functor<F: Type -> Type> {
  map<A, B>(_ value: F<A>, _ transform: (A) -> B) -> F<B> {
    throw Error.new("contract stub")
  }
}
class Either<E, A> {}
class EitherFunctor<E> is Functor<<X> =>> Either<E, X>> {
  map<X, Y>(_ value: Either<E, X>, _ transform: (X) -> Y) -> Either<E, Y> {
    throw Error.new("implementation stub")
  }
}
"#,
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn generic_hkt_override_with_incompatible_result_is_rejected() {
    let f = Fixture::new(
        r#"
class Functor<F: Type -> Type> {
  map<A, B>(_ value: F<A>, _ transform: (A) -> B) -> F<B> {
    throw Error.new("contract stub")
  }
}
class Either<E, A> {}
class IncompatibleEitherFunctor<E> is Functor<<X> =>> Either<E, X>> {
  map<X, Y>(_ value: Either<E, X>, _ transform: (X) -> Y) -> String {
    throw Error.new("implementation stub")
  }
}
"#,
    );
    assert!(
        f.analysis.snapshot.has_errors(),
        "a generic override must not widen the inherited result contract"
    );
}

#[test]
fn captured_write_must_invalidate_previous_int_fact() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() -> Int {
    let x: Int | String = 1
    let action = || { x = "changed" }
    action()
    return x
  }
}
"#,
    );
    assert!(
        f.analysis.snapshot.has_errors(),
        "executing the captured String write must prevent certifying an Int return"
    );
}

#[test]
fn applied_constructor_has_precise_static_type() {
    let f = Fixture::new(
        r#"
class Box<T> { @constructor new() {} }
class Probe { @class run() { let result = Box<Int>.new() } }
"#,
    );
    f.assert_no_error_diagnostics();
    let run = f.callable("Probe", "run", phalcom_semantic::identity::DispatchSide::Class);
    let result = f.binding(run, "result").current.ty().unwrap();
    let phalcom_semantic::types::store::TypeData::Applied { arguments, .. } = f.analysis.snapshot.store.get(result) else {
        panic!("expected applied result")
    };
    assert_eq!(arguments.as_ref(), &[phalcom_semantic::types::store::StaticArgument::Term(f.ty("Int"))]);
}

#[test]
fn generic_inheritance_source_accepts_base_parameter() {
    let f = Fixture::new(
        r#"
class Base {}
class Child<T> is Base { @constructor new() {} }
class Probe {
  @class consume(_ value: Base) -> Int { return 1 }
  @class run() { let result = Probe.consume(Child<Int>.new()) }
}
"#,
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn tuple_union_source_accepts_both_components() {
    let f = Fixture::new(
        r#"
class Probe {
  @class run() { let value: (Int | String, Int | String) = (1, 2) }
}
"#,
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn tuple_union_source_accepts_heterogeneous_components() {
    let f = Fixture::new(
        r#"
class Probe {
  @class run() { let value: (Int | String, Int | String) = (1, "two") }
}
"#,
    );
    f.assert_no_error_diagnostics();
}

#[test]
/// AUD-SEM-006: `()` is the canonical Unit value, including under either Unit
/// annotation spelling.
fn unit_spellings_have_one_source_identity() {
    let f = Fixture::new(
        r#"
class Probe { @class run() { let a: Unit = (); let b: () = () } }
"#,
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn unit_literal_default_and_union_share_canonical_identity() {
    let f = Fixture::new(
        r#"
class Probe {
  @class explicit() -> Unit { () }
  @class inferred() { () }
  @class run() {
    let default: Unit = ()
    let alternative: Unit | Int = ()
  }
}
"#,
    );
    f.assert_no_error_diagnostics();
    let run = f.callable("Probe", "run", phalcom_semantic::identity::DispatchSide::Class);
    let default = f.binding(run, "default");
    let alternative = f.binding(run, "alternative");
    let unit = f.analysis.snapshot.store.unit();
    assert_eq!(default.current.ty(), Some(unit));
    assert_eq!(alternative.current.ty(), Some(unit));
}
