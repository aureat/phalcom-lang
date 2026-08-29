use crate::semantic::support::Fixture;
use phalcom_semantic::identity::DispatchSide;

/// LAW: nested return in branch is recorded exactly once as a callable exit fact.
#[test]
fn nested_return_is_recorded_once_as_callable_exit() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) -> Int {
    if flag {
      return 1
    }
    return 2
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(run.exits.normal_returns.len(), 2, "each source return path must be recorded exactly once");
    assert!(run.exits.normal_returns.iter().all(|exit| exit.knowledge.ty() == Some(f.ty("Int"))));
    f.assert_no_error_diagnostics();
}

/// LAW: nested throw is recorded as throw exit and not as a normal return.
#[test]
fn nested_throw_is_not_misclassified_as_normal_return() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) -> Int {
    if flag {
      throw "bad"
    }
    return 1
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(run.exits.throws.len(), 1);
    assert_eq!(run.exits.normal_returns.len(), 1);
    f.assert_no_error_diagnostics();
}

/// LAW: abrupt region contributes no normal value to enclosing branch expression.
#[test]
fn abrupt_region_has_no_normal_value() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = if flag {
      return 1
    } else {
      "ok"
    }
    let result = x
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "x", f.ty("String"));
    f.assert_binding_established(run, "result", f.ty("String"));
}

/// LAW: mutations inside executable branch survive into branch flow and post-join.
#[test]
fn executable_branch_mutation_survives_region_scope() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = 1
    if flag {
      x = "changed"
    } else {
      x = 2
    }
    let observed = x
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_union_members(
        f.binding(run, "observed").current.ty().expect("joined mutation"),
        &[f.ty("Int"), f.ty("String")],
    );
}

/// LAW: if let success region executes as control region and preserves outer mutation.
#[test]
fn if_let_success_region_preserves_outer_mutation() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ input: Int) {
    let observed = 0
    if let value = input {
      observed = "matched"
    } else {
      observed = 2
    }
    let result = observed
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_union_members(
        f.binding(run, "result").current.ty().expect("if-let outer mutation join"),
        &[f.ty("Int"), f.ty("String")],
    );
}

/// LAW: if let pattern binding is lexically scoped to the success region.
#[test]
fn if_let_pattern_binding_is_lexically_scoped() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let input = 10
    if let value = input {
      let inside = value
    }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "inside", f.ty("Int"));
    f.assert_no_error_diagnostics();
}

/// LAW: if let branch values join as executed control regions.
#[test]
fn if_let_branch_values_join_as_executed_regions() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ input: Int) {
    let x = if let value = input {
      1
    } else {
      "none"
    }
    let result = x
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_union_members(
        f.binding(run, "result").current.ty().expect("if-let branch value join"),
        &[f.ty("Int"), f.ty("String")],
    );
}

/// LAW: composed branch return and field mutation retain precise exit field facts.
#[test]
fn branch_return_and_field_mutation_keep_separate_exit_facts() {
    let f = Fixture::new(
        r#"
class Box {
  _value: Int = 0

  update(_ flag: Bool) -> Int {
    if flag {
      _value = 1
      return _value
    } else {
      _value = 2
    }
    _value
  }
}
"#,
    );
    let update = f.callable("Box", "update", DispatchSide::Instance);
    assert_eq!(update.exits.normal_returns.len(), 2);
    assert!(update.exits.normal_returns.iter().all(|exit| exit.knowledge.ty() == Some(f.ty("Int"))));
    assert!(
        update
            .exits
            .normal_returns
            .iter()
            .all(|exit| { exit.flow.fields.values().any(|field| field.current.ty() == Some(f.ty("Int"))) })
    );
}

/// LAW: nested control and shadowing remain cleanly isolated.
#[test]
fn nested_control_and_shadowing_composition() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ outer: Bool, _ inner: Bool) -> Int {
    let x = 0
    if outer {
      let x = "shadow"
      if inner {
        return 1
      }
    } else {
      x = 2
    }
    let y = x
    return y
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(run.exits.normal_returns.len(), 2);
    f.assert_binding_established(run, "y", f.ty("Int"));
    f.assert_no_error_diagnostics();
}

/// LAW: unreachable trailing statements after return do not publish or execute.
#[test]
fn unreachable_trailing_statements_do_not_publish_or_leak() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) -> Int {
    if flag {
      return 1
      let impossible = 999
    }
    return 2
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    assert!(
        !run.bindings.values().any(|b| b.name == "impossible"),
        "unreachable binding must not be published"
    );
    f.assert_no_error_diagnostics();
}
