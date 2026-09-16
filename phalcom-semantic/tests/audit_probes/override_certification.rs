//! Adversarial override certification and static proof invariant probes.

use crate::semantic::support::Fixture;

#[test]
#[ignore = "AUD-SEM-001: incompatible override defect (analysis must reject incompatible override)"]
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
#[ignore = "AUD-SEM-002: captured write stale fact defect (captured mutation must invalidate flow fact)"]
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
    assert_eq!(arguments.as_ref(), &[f.ty("Int")]);
}

#[test]
#[ignore = "AUD-SEM-005: generic inheritance parameter acceptance (generic child applied form subtyping)"]
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
#[ignore = "AUD-SEM-006: tuple union source assignment"]
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
fn unit_spellings_have_one_source_identity() {
    let f = Fixture::new(
        r#"
class Probe { @class run() { let a: Unit = (); let b: () = () } }
"#,
    );
    f.assert_no_error_diagnostics();
}
