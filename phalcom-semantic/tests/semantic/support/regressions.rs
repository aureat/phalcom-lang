use super::{Fixture, tuple, union};
use phalcom_semantic::identity::DispatchSide;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// The assertion DSL is part of the semantic oracle. Complex expectations
/// inside unions must be matched structurally rather than acting as wildcards.
#[test]
fn union_expectation_rejects_wrong_structural_members() {
    let f = Fixture::new(
        r#"
class Cat { @constructor new() {} }
class Dog { @constructor new() {} }
class Probe {
  @class
  run(_ flag: Bool) {
    let value = if flag { Cat.new() } else { Dog.new() }
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let actual = f.binding(run, "value").current.ty().expect("branch union");

    let rejected = catch_unwind(AssertUnwindSafe(|| {
        f.assert_type(actual, union([tuple([int_ty.into()]), tuple([string_ty.into()])]));
    }));

    assert!(rejected.is_err(), "wrong structural union members were accepted by the test oracle");
}
