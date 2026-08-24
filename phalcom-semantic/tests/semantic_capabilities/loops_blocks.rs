use crate::support::{Fixture, assert_source_contract};
use phalcom_semantic::identity::DispatchSide;

#[test]
fn loop_same_type_assignment_preserves_current_type() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    var x = 1
    while flag {
      x = 2
    }
    let y = x
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "x", int_ty);
    f.assert_binding_established(run, "y", int_ty);
}

#[test]
fn loop_join_includes_preheader_and_body_types() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    var x: Number = 1
    while flag {
      x = 2.5
    }
    let y = x
  }
}
"#,
    );
    let number = f.ty("Number");
    let int_ty = f.ty("Int");
    let float_ty = f.ty("Float");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let x = f.binding(run, "x");
    assert_source_contract(x, number);
    let joined = x.current.ty().expect("loop fixed point should retain a type");
    f.assert_union_members(joined, &[int_ty, float_ty]);
    f.assert_subtype(joined, number);
    f.assert_union_members(f.binding(run, "y").current.ty().expect("post-loop read"), &[int_ty, float_ty]);
}

#[test]
fn break_and_continue_preserve_loop_exit_and_backedge_facts() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ skip: Bool) {
    var x = 1
    for n in [1, 2, 3] {
      if skip {
        x = "continued"
        continue
      }
      x = 2.5
      break
    }
    let y = x
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let float_ty = f.ty("Float");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let post = f.binding(run, "y").current.ty().expect("loop exit join");
    f.assert_union_members(post, &[int_ty, string_ty, float_ty]);
}

#[test]
fn captured_block_write_is_not_applied_until_execution_is_proven() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    var x = 1
    let action = {
      x = "changed"
    }
    let y = x
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "y", int_ty);
}
