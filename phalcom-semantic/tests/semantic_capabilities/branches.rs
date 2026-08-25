use crate::semantic::support::{Fixture, assert_source_contract, assert_validated, binding, known, union};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::EvidenceStatus;

/// LAW: both reachable branch literals establish one precise joined result.
#[test]
fn same_type_branch_results_establish_single_result_type() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = if flag {
      1
    } else {
      2
    }
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_expression_knowledge(
        f.expression_n(run, "1", 0),
        known(int_ty).established().origin(phalcom_semantic::EvidenceOrigin::Syntax),
    );
    f.assert_expression_knowledge(
        f.expression_n(run, "2", 0),
        known(int_ty).established().origin(phalcom_semantic::EvidenceOrigin::Syntax),
    );
    f.assert_expression_established(f.expression_n(run, "1", 0), int_ty);
    f.assert_expression_established(f.expression_n(run, "2", 0), int_ty);
    f.assert_binding_established(run, "x", int_ty);
}

/// LAW: heterogeneous reachable branch values join to a precise union.
#[test]
fn heterogeneous_branch_results_join_into_union() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = if flag {
      1
    } else {
      "hello"
    }
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_expression_knowledge(
        f.expression_n(run, "1", 0),
        known(int_ty).established().origin(phalcom_semantic::EvidenceOrigin::Syntax),
    );
    f.assert_expression_knowledge(
        f.expression_n(run, "\"hello\"", 0),
        known(string_ty).established().origin(phalcom_semantic::EvidenceOrigin::Syntax),
    );
    let x = f.binding(run, "x");
    let joined = x.current.ty().expect("branch result should have a formal type");
    f.assert_union_members(joined, &[int_ty, string_ty]);
    assert_eq!(x.current.status(), Some(EvidenceStatus::Established));
    f.assert_binding_expectation(
        run,
        "x",
        binding().current(
            known(union([int_ty.into(), string_ty.into()]))
                .established()
                .origin(phalcom_semantic::EvidenceOrigin::Flow),
        ),
    );
    f.assert_no_error_diagnostics();
}

/// LAW: a broad contract validates, but does not replace, a narrower branch union.
#[test]
fn branch_union_validates_common_supertype_without_widening_current_fact() {
    let f = Fixture::new(
        r#"
class Animal {}
class Cat is Animal { @constructor new() {} }
class Dog is Animal { @constructor new() {} }
class Probe {
  @class
  run(_ flag: Bool) {
    let x: Animal = if flag {
      Cat.new()
    } else {
      Dog.new()
    }
  }
}
"#,
    );
    let animal = f.ty("Animal");
    let cat = f.ty("Cat");
    let dog = f.ty("Dog");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let x = f.binding(run, "x");
    assert_source_contract(x, animal);
    let current = x.current.ty().expect("branch union should be retained");
    f.assert_union_members(current, &[cat, dog]);
    f.assert_subtype(current, animal);
    assert_validated(x);
    f.assert_binding_expectation(
        run,
        "x",
        binding()
            .declared(animal)
            .current(
                known(union([cat.into(), dog.into()]))
                    .established()
                    .origin(phalcom_semantic::EvidenceOrigin::Flow),
            )
            .validated(),
    );
    f.assert_no_error_diagnostics();
}

/// LAW: an abrupt branch contributes no value to the continuing join.
#[test]
fn returning_branch_does_not_contribute_value_to_continuing_join() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = if flag {
      return 1
    } else {
      2
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

/// LAW: a throwing branch is excluded from reachable value knowledge.
#[test]
fn throwing_branch_is_excluded_from_reachable_value_join() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = if flag {
      throw "bad"
    } else {
      42
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
fn same_type_writes_in_both_branches_preserve_flow_type() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = 1
    if flag {
      x = 2
    } else {
      x = 3
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
fn divergent_branch_assignments_join_current_binding_types() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = 1
    if flag {
      x = 2
    } else {
      x = "three"
    }
    let y = x
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    for name in ["x", "y"] {
        let ty = f.binding(run, name).current.ty().expect("joined flow type");
        f.assert_union_members(ty, &[int_ty, string_ty]);
    }
}

#[test]
fn branch_join_preserves_narrow_flow_under_broad_declared_contract() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x: Number = 1
    if flag {
      x = 2
    } else {
      x = 3.5
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
    let current = x.current.ty().expect("joined current type");
    f.assert_union_members(current, &[int_ty, float_ty]);
    f.assert_subtype(current, number);
    let y = f.binding(run, "y");
    f.assert_union_members(y.current.ty().expect("y type"), &[int_ty, float_ty]);
}

#[test]
fn refuted_branch_assignment_does_not_fabricate_declared_flow_fact() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x: Number = 1
    if flag {
      x = "bad"
    } else {
      x = 2
    }
    let y = x
  }
}
"#,
    );
    let number = f.ty("Number");
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let x = f.binding(run, "x");
    assert_source_contract(x, number);
    let current = x.current.ty().expect("recovery should retain concrete branch facts");
    f.assert_union_members(current, &[int_ty, string_ty]);
    assert_ne!(current, number, "invalid write must not be laundered into declared Number");
    let y = f.binding(run, "y");
    f.assert_union_members(y.current.ty().expect("recovery flow type"), &[int_ty, string_ty]);
}

#[test]
fn branch_local_shadow_does_not_mutate_outer_binding_flow() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = 1
    if flag {
      let x = "shadow"
      let inside = x
    }
    let outside = x
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let xs = f.bindings_named(run, "x");
    assert_eq!(xs.len(), 2, "shadowing should create distinct bindings: {xs:#?}");
    assert_ne!(xs[0].binding, xs[1].binding);
    assert_eq!(xs[0].current.ty(), Some(int_ty));
    assert_eq!(xs[1].current.ty(), Some(string_ty));
    f.assert_binding_type(run, "inside", string_ty);
    f.assert_binding_type(run, "outside", int_ty);
}

/// LAW: nested branch joins flatten transitively without widening.
#[test]
fn nested_branch_results_compose_transitively() {
    let f = Fixture::new(
        r#"
class Animal {}
class Cat is Animal { @constructor new() {} }
class Dog is Animal { @constructor new() {} }
class Bird is Animal { @constructor new() {} }
class Probe {
  @class
  run(_ a: Bool, _ b: Bool) {
    let x = if a {
      if b { Cat.new() } else { Dog.new() }
    } else {
      Bird.new()
    }
  }
}
"#,
    );
    let cat = f.ty("Cat");
    let dog = f.ty("Dog");
    let bird = f.ty("Bird");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_union_members(f.binding(run, "x").current.ty().expect("nested join"), &[cat, dog, bird]);
}

/// LAW: reachable unknown evidence remains incomplete; known arm cannot launder it.
#[test]
fn known_branch_does_not_hide_reachable_unknown_branch_in_formal_analysis() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = if flag {
      42
    } else {
      mystery()
    }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let x = f.binding(run, "x");
    assert!(
        x.current.is_unknown() || x.current.is_dynamic(),
        "reachable unknown branch must not be discarded in formal analysis: {x:#?}"
    );
}
