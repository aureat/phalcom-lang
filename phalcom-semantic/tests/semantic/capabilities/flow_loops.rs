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
