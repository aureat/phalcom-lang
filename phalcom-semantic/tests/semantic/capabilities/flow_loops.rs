use crate::semantic::support::{Fixture, assert_source_contract};
use phalcom_semantic::checker::flow::graph::{FlowEdgeKind, FlowNodeKind};
use phalcom_semantic::identity::DispatchSide;

/// LAW: same-type loop writes preserve current type.
#[test]
fn loop_same_type_assignment_preserves_current_type() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = 1
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

/// LAW: loop join includes zero-iteration and body paths.
#[test]
fn loop_join_includes_preheader_and_body_types() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x: Number = 1
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

/// LAW: break/continue paths contribute correct loop exit and backedge facts.
#[test]
fn break_and_continue_preserve_loop_exit_and_backedge_facts() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ skip: Bool) {
    let x = 1
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

/// LAW: closure capture checks body writes without applying them at creation.
#[test]
fn captured_block_write_is_not_applied_until_execution_is_proven() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let x = 1
    let action = || {
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

/// COMPOSED: loop fixed points retain mutation facts across nested break/continue paths.
#[test]
fn loop_fixpoint_preserves_mutated_integer_and_abrupt_edges() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ limit: Int) -> Int {
    let total = 0
    let i = 0
    while (i < limit) {
      i = i + 1
      if (i == 2) { continue }
      total = total + i
    }
    for item in [1, 2, 3] {
      if (item == 2) { break }
      total = total + item
    }
    total
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);

    f.assert_binding_established(run, "total", int_ty);
    f.assert_normal_return(
        run,
        crate::semantic::support::known(int_ty)
            .established()
            .origin(phalcom_semantic::EvidenceOrigin::Flow),
    );
    assert!(
        run.flow_graph.nodes.values().any(|node| matches!(node.kind, FlowNodeKind::LoopHeader)),
        "composed loop must publish loop-header structure"
    );
    assert!(
        run.flow_graph.edges.values().any(|edge| matches!(edge.kind, FlowEdgeKind::BackEdge)),
        "while loop must publish a back edge"
    );
    assert!(
        run.flow_graph.edges.values().any(|edge| matches!(edge.kind, FlowEdgeKind::Continue))
            && run.flow_graph.edges.values().any(|edge| matches!(edge.kind, FlowEdgeKind::Break)),
        "nested abrupt loop paths must remain explicit in the flow graph"
    );
    f.assert_no_error_diagnostics();
}

/// LAW: continue state feeds next header, not direct post-loop exit.
#[test]
fn continue_state_feeds_header_not_direct_post_loop_exit() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ keepGoing: Bool, _ skip: Bool) {
    let x = 1
    while keepGoing {
      if skip {
        x = "continued"
        continue
      }
      x = 2.5
      break
    }
    let observed = x
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let float_ty = f.ty("Float");
    let observed = f.binding(run, "observed").current.ty().expect("post-loop knowledge");
    f.assert_union_members(observed, &[int_ty, string_ty, float_ty]);
    f.assert_continue_edges_target_loop_headers(run);
}

/// LAW: dead suffix after continue never contributes loop facts.
#[test]
fn statement_after_continue_never_contributes_loop_fact() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = 1
    while flag {
      x = "seen"
      continue
      x = true
    }
    let y = x
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let y = f.binding(run, "y").current.ty().expect("post-loop type");
    f.assert_union_members(y, &[f.ty("Int"), f.ty("String")]);
    assert!(!f.union_contains(y, f.ty("Bool")));
}

/// LAW: dead suffix after break never contributes loop facts.
#[test]
fn statement_after_break_never_contributes_loop_fact() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = 1
    while flag {
      x = "seen"
      break
      x = true
    }
    let y = x
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let y = f.binding(run, "y").current.ty().expect("post-loop type");
    f.assert_union_members(y, &[f.ty("Int"), f.ty("String")]);
    assert!(!f.union_contains(y, f.ty("Bool")));
}

/// LAW: while let evaluates iteratively and pattern bindings remain scoped to body.
#[test]
fn while_let_is_cyclic_with_scoped_pattern_bindings() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ opt: Int) {
    let x = 1
    while let n = opt {
      x = "seen"
    }
    let y = x
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let y = f.binding(run, "y").current.ty().expect("post-loop type");
    f.assert_union_members(y, &[f.ty("Int"), f.ty("String")]);
}

/// LAW: nested loops preserve distinct frame targets for break and continue.
#[test]
fn nested_loop_target_ownership() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ outer: Bool, _ inner: Bool, _ skip: Bool, _ keepInner: Bool) {
    let x: Object = 1
    while outer {
      while inner {
        x = "inner-break"
        break
      }
      if skip {
        x = 2.5
        continue
      }
      if keepInner {
        break
      }
      x = true
      break
    }
    let observed = x
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let int_ty = f.ty("Int");
    let str_ty = f.ty("String");
    let float_ty = f.ty("Float");
    let bool_ty = f.ty("Bool");
    let observed = f.binding(run, "observed").current.ty().expect("post-loop type");
    f.assert_union_members(observed, &[int_ty, str_ty, float_ty, bool_ty]);
    f.assert_no_error_diagnostics();
}

/// LAW: all-abrupt loop body without break only exits via false condition.
#[test]
fn all_abrupt_loop_body_yields_false_condition_exit() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) -> Object {
    let x: Object = 1
    while flag {
      x = "returning"
      return 42
    }
    x
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let int_ty = f.ty("Int");
    f.assert_binding_established(run, "x", int_ty);
    f.assert_no_error_diagnostics();
}

/// LAW: field mutation across loop cycles preserves field contract.
#[test]
fn field_mutation_through_loop_preserves_contract() {
    let f = Fixture::new(
        r#"
class Counter {
  _val: Int

  init() {
    self._val = 0
    let count = 0
    while (count < 3) {
      count = count + 1
      self._val = self._val + 1
    }
  }
}
"#,
    );
    let init = f.callable("Counter", "init", DispatchSide::Instance);
    let int_ty = f.ty("Int");
    f.assert_binding_established(init, "count", int_ty);
    f.assert_no_error_diagnostics();
}
