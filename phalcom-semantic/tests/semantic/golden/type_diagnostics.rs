use crate::semantic::support::Fixture;
use phalcom_semantic::dispatch::CallableSemanticKind;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::{DiagnosticCode, DiagnosticDetail, DiagnosticPresenter, ExplanationStep, RelationOutcome};

fn trace_has(fixture: &Fixture, code: DiagnosticCode, predicate: impl Fn(&ExplanationStep) -> bool) {
    let diagnostic = fixture
        .diagnostics(code)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing {code:?} diagnostic"));
    let trace = fixture.diagnostic_trace(diagnostic);
    assert!(
        trace.iter().any(|node| predicate(&node.step)),
        "missing expected trace step for {code:?}: {trace:#?}"
    );
}

#[test]
fn rich_diagnostic_a_constructor_binding_mismatch_keeps_constructor_self_chain() {
    let fixture = Fixture::new(
        r#"
class CellNum { @constructor new() {} }
class Probe {
  @class
  run() {
    let value: Int = CellNum.new()
  }
}
"#,
    );
    let diagnostic = fixture.diagnostics(DiagnosticCode::BindingInitializerMismatch)[0];
    let trace = fixture.diagnostic_trace(diagnostic);
    assert!(trace.iter().any(|node| matches!(
        node.step,
        ExplanationStep::CallableKind {
            kind: CallableSemanticKind::Constructor,
            ..
        }
    )));
    assert!(trace.iter().any(|node| matches!(node.step, ExplanationStep::CallableReturn { .. })));
    assert!(trace.iter().any(|node| matches!(node.step, ExplanationStep::SelfTypeSpecialization { .. })));
    assert!(trace.iter().any(|node| matches!(
        &node.step,
        ExplanationStep::TypeRelation {
            outcome: RelationOutcome::Refuted(_),
            ..
        }
    )));
}

#[test]
fn rich_diagnostic_b_argument_mismatch_keeps_argument_and_relation_edges() {
    let fixture = Fixture::new(
        r#"
class CellNum {
  @constructor new() {}
  @class
  fromInt(_ value: Int) -> CellNum { CellNum.new() }
}
class Probe {
  @class
  run() {
    let value = CellNum.fromInt("bad")
  }
}
"#,
    );
    trace_has(&fixture, DiagnosticCode::ArgumentMismatch, |step| {
        matches!(step, ExplanationStep::ArgumentCheck { parameter_index: 0, .. })
    });
    trace_has(&fixture, DiagnosticCode::ArgumentMismatch, |step| {
        matches!(
            step,
            ExplanationStep::TypeRelation {
                outcome: RelationOutcome::Refuted(_),
                ..
            }
        )
    });
}

#[test]
fn rich_diagnostic_c_generic_conflict_keeps_constraint_set_and_argument_range() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  constrained<T>(_ value: T) -> T where T == Int { value }

  @class
  run() {
    let result = Probe.constrained("bad")
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let diagnostic = fixture.diagnostics(DiagnosticCode::GenericConstraintUnsatisfied)[0];
    assert_eq!(diagnostic.primary_range, fixture.expression(run, "\"bad\"").range);
    let trace = fixture.diagnostic_trace(diagnostic);
    assert!(trace.iter().any(|node| matches!(node.step, ExplanationStep::GenericConstraint { .. })));
    assert!(trace.iter().any(|node| matches!(node.step, ExplanationStep::GenericConflict { .. })));
}

#[test]
fn rich_diagnostic_d_refined_branch_return_mismatch_keeps_refinement_and_return_check() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run(_ value: Object) -> String {
    if (value.is(Int)) {
      return value
    } else {
      return "ok"
    }
  }
}
"#,
    );
    trace_has(&fixture, DiagnosticCode::ReturnMismatch, |step| {
        matches!(step, ExplanationStep::FlowRefinement { .. })
    });
    trace_has(&fixture, DiagnosticCode::ReturnMismatch, |step| {
        matches!(step, ExplanationStep::ReturnCheck { .. })
    });
}

#[test]
fn rich_diagnostic_e_expected_context_selection_is_not_underconstrained() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  make<T>() -> T { 42 }

  @class
  run() {
    let result: Int = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("Int")));
    assert_eq!(call.knowledge.status(), Some(phalcom_semantic::types::evidence::EvidenceStatus::Assumed));
    assert!(fixture.diagnostics(DiagnosticCode::GenericInferenceUnderconstrained).is_empty());
}

#[test]
fn rich_diagnostic_f_type_constructor_error_has_stable_specialized_presentation() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let value: Int<String> = 1
  }
}
"#,
    );
    let diagnostic = fixture.diagnostics(DiagnosticCode::ApplicationNotConstructor)[0];
    let presented = DiagnosticPresenter::new(&fixture.analysis.snapshot).present(diagnostic, DiagnosticDetail::Explain);
    assert_eq!(presented.code, DiagnosticCode::ApplicationNotConstructor);
    assert!(!presented.headline.is_empty());
}

#[test]
fn rich_diagnostic_g_structural_product_failure_retains_product_fact_and_refuted_relation() {
    let fixture = Fixture::new(
        r#"
class Choice {}
class Present is Choice { @constructor new() {} }
class Probe {
  @class
  run() {
    let value: (Number, Choice) = ("bad", Present.new())
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let binding = fixture.binding(run, "value");
    let actual = binding.current.ty().expect("actual tuple type");
    fixture.assert_tuple_types(actual, &[fixture.ty("String"), fixture.ty("Present")]);
    trace_has(&fixture, DiagnosticCode::BindingInitializerMismatch, |step| {
        matches!(
            step,
            ExplanationStep::TypeRelation {
                outcome: RelationOutcome::Refuted(_),
                ..
            }
        )
    });
}

#[test]
fn rich_diagnostic_h_return_mismatch_has_explicit_return_check() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() -> Int {
    return "bad"
  }
}
"#,
    );
    trace_has(&fixture, DiagnosticCode::ReturnMismatch, |step| {
        matches!(step, ExplanationStep::ReturnCheck { .. })
    });
    trace_has(&fixture, DiagnosticCode::ReturnMismatch, |step| {
        matches!(
            step,
            ExplanationStep::TypeRelation {
                outcome: RelationOutcome::Refuted(_),
                ..
            }
        )
    });
}
