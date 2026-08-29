use crate::semantic::support::{Fixture, assert_source_contract, assert_validated, binding, known, union};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::{EvidenceStatus, TypeKnowledge, UnknownReason};

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

/// COMPOSED: narrowing plus an abrupt arm publishes only reachable normal values.
#[test]
fn refined_branch_with_abrupt_else_publishes_only_normal_value() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ value: Object) {
    if (value.is(Int)) {
      return value
    } else {
      throw value
    }
  }
}
"#,
    );

    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);

    f.assert_expression_established(f.expression_n(run, "value", 1), int_ty);
    f.assert_normal_return(run, known(int_ty).established().origin(phalcom_semantic::EvidenceOrigin::Flow));
    assert_eq!(run.exits.throws.len(), 1, "throwing arm must remain recorded as abrupt");
    f.assert_no_error_diagnostics();
}

#[test]
fn overridden_is_method_does_not_gain_builtin_refinement_authority() {
    let f = Fixture::new(
        r#"
class Liar {
  is(_ cls) -> Bool { true }
}

class Probe {
  @class
  run(_ value: Liar) {
    if (value.is(Int)) {
      return value
    }
    0
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let liar = f.ty("Liar");
    let branch_value = f.expression_n(run, "value", 1);
    assert_eq!(branch_value.knowledge.ty(), Some(liar));
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

/// L05: an unknown nested branch remains visible after a known outer join.
#[test]
fn nested_unknown_branch_weakens_transitive_join() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ outer: Bool, _ inner: Bool) {
    let value = if outer {
      if inner { 1 } else { mystery() }
    } else {
      2
    }
    let observed = value
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let value = f.binding(run, "value");
    assert!(value.current.is_unknown() || value.current.is_dynamic());
    assert!(f.binding(run, "observed").current.is_unknown() || f.binding(run, "observed").current.is_dynamic());
}

/// L03: a return nested inside one arm contributes no continuing value.
#[test]
fn nested_return_arm_does_not_pollute_outer_value_join() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ outer: Bool, _ inner: Bool) {
    let value = if outer {
      if inner { return 1 } else { 2 }
    } else {
      3
    }
    let observed = value
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "value", int_ty);
    f.assert_binding_established(run, "observed", int_ty);
}

/// L02: branch products preserve a union at the outer collection boundary.
#[test]
fn branch_collections_join_without_collapsing_to_object() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let values = if flag { [1] } else { ["text"] }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let values = f.binding(run, "values").current.ty().expect("joined list");
    assert!(
        !matches!(f.analysis.snapshot.store.get(values), phalcom_semantic::types::store::TypeData::Nominal { declaration } if declaration.name.as_ref() == "Object")
    );
}

/// LAW P3: canonical type test on an assumed parameter establishes the exact target.
#[test]
fn canonical_type_test_can_establish_exact_target_from_assumed_parameter() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ value: Object) {
    if value.is(Int) {
      let narrowed = value
    }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "narrowed", f.ty("Int"));
    f.assert_no_error_diagnostics();
}

/// LAW P4: a broad observation on an assumed narrow type does not strengthen the assumption.
#[test]
fn broad_test_on_narrow_assumed_union_preserves_assumed_status() {
    let f = Fixture::new(
        r#"
class Animal {}
class Cat is Animal {}
class Dog is Animal {}

class Probe {
  @class
  run(_ value: Cat) {
    if value.is(Animal) {
      let narrowed = value
    }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let cat_ty = f.ty("Cat");
    let narrowed = f.binding(run, "narrowed");
    assert_eq!(narrowed.current.ty(), Some(cat_ty));
    assert_eq!(narrowed.current.status(), Some(EvidenceStatus::Assumed));
}

/// LAW P6: trusted type test prunes branch contradicting established literal type.
#[test]
fn trusted_type_test_prunes_branch_contradicting_established_literal_type() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let value = "hello"
    let result = if value.is(Int) {
      1
    } else {
      "ok"
    }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "result", f.ty("String"));
    f.assert_no_error_diagnostics();
}

/// LAW P10: contradictory branch does not publish bindings or error diagnostics.
#[test]
fn contradictory_branch_does_not_publish_bindings_or_diagnostics() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let value = "hello"
    if value.is(Int) {
      let impossible = mystery()
    }
    let observed = value
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    assert!(f.bindings_named(run, "impossible").is_empty());
    f.assert_no_error_diagnostics();
    f.assert_binding_established(run, "observed", f.ty("String"));
}

/// LAW P7: assumed type contradicted by runtime observation degrades to InferenceConflict and stays reachable.
#[test]
fn assumed_type_contradicted_by_runtime_test_degrades_to_inference_conflict_and_stays_reachable() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ value: Int) {
    if not value.is(Int) {
      let residual = value
    }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let residual = f.binding(run, "residual");
    assert_eq!(residual.current, TypeKnowledge::Unknown(UnknownReason::InferenceConflict));
}

/// LAW P9: user-overloaded equality does not gain formal refinement authority.
#[test]
fn overloaded_equality_does_not_gain_formal_refinement_authority() {
    let f = Fixture::new(
        r#"
class Liar {
  ==(_ other: Object) -> Bool {
    true
  }
}

class Probe {
  @class
  run(_ liar: Liar) {
    if liar == None {
      let unchecked = liar
    }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let liar_ty = f.ty("Liar");
    let unchecked = f.binding(run, "unchecked");
    assert_eq!(unchecked.current.ty(), Some(liar_ty));
    assert_eq!(unchecked.current.status(), Some(EvidenceStatus::Assumed));
}

/// LAW P11/P12: if true uses only true branch value and prunes dead false branch.
#[test]
fn if_true_uses_only_true_branch_value() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let value = if true { 1 } else { mystery() }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "value", f.ty("Int"));
    f.assert_no_error_diagnostics();
}

/// LAW P11/P12: if false uses only false branch value and prunes dead true branch.
#[test]
fn if_false_uses_only_false_branch_value() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let value = if false { mystery() } else { "ok" }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "value", f.ty("String"));
    f.assert_no_error_diagnostics();
}

/// LAW P11: unary not on boolean constants inverts truth.
#[test]
fn if_not_true_and_if_not_false_prune_correct_branches() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let not_true = if not true { mystery() } else { 1 }
    let not_false = if not false { "ok" } else { mystery() }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "not_true", f.ty("Int"));
    f.assert_binding_established(run, "not_false", f.ty("String"));
    f.assert_no_error_diagnostics();
}

/// LAW P12: constant false branch does not record callable normal return.
#[test]
fn constant_false_branch_does_not_record_callable_return() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() -> String {
    if false {
      return 1
    }
    return "live"
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(run.exits.normal_returns.len(), 1);
    assert_eq!(run.exits.normal_returns[0].knowledge.ty(), Some(f.ty("String")));
    f.assert_no_error_diagnostics();
}

/// LAW P1/P3: trusted narrowing certifies declared return without laundering assumptions.
#[test]
fn trusted_narrowing_can_certify_declared_return_without_laundering_assumptions() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  normalize(_ value: Object) -> Int {
    if value.is(Int) {
      return value
    }
    0
  }
}
"#,
    );
    let normalize = f.callable("Probe", "normalize", DispatchSide::Class);
    assert_eq!(normalize.exits.normal_returns.len(), 2);
    assert!(normalize.exits.normal_returns.iter().all(|exit| exit.knowledge.ty() == Some(f.ty("Int"))));
    f.assert_no_error_diagnostics();
}
