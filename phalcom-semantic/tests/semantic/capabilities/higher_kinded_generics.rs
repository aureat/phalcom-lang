//! SC-4 higher-kinded inference characterization.

use crate::semantic::support::{Fixture, applied, nominal};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::explain::{ExplanationStep, GenericConstraintOrigin};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::EvidenceStatus;

#[test]
fn expected_result_selects_hkt_constructor_without_value_evidence() {
    let fixture = Fixture::new(
        r#"
enum Either<L, R> {
  @variant Left(_ value: L)
  @variant Right(_ value: R)
}

class Probe {
  @class
  make<F: Type -> Type, A>() -> F<A> { Either::Left(0) }

  @class
  run() {
    let result: Either<String, Int> = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    fixture.assert_type(
        call.knowledge.ty().expect("contextual HKT result"),
        applied("Either", [nominal("String"), nominal("Int")]),
    );
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Assumed));
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
    fixture.assert_trace_has(run, call, |step| {
        matches!(
            step,
            ExplanationStep::GenericConstraint {
                origin: GenericConstraintOrigin::ExpectedResult,
                ..
            }
        )
    });
}

#[test]
fn nested_generic_result_uses_outer_expected_context() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  make<T>() -> T { 0 }

  @class
  apply<T, U>(_ value: T, _ transform: (T) -> U) -> U { transform(value) }

  @class
  run() {
    let result: Int = Probe.apply(1, |value| { Probe.make() })
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.apply(1, |value| { Probe.make() })");
    fixture.assert_type(call.knowledge.ty().expect("nested generic result"), nominal("Int"));
    fixture.assert_no_error_diagnostics();
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
}

#[test]
fn argument_selected_hkt_is_retained_when_expected_constructor_conflicts() {
    let fixture = Fixture::new(
        r#"
class List<T> {}
class Option<T> {}

class Probe {
  @class
  keep<F: Type -> Type, A>(_ value: F<A>) -> F<A> { value }

  @class
  run(_ value: List<Int>) {
    let result: Option<Int> = Probe.keep(value)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.keep(value)");
    fixture.assert_type(call.knowledge.ty().expect("argument-selected result"), applied("List", [nominal("Int")]));
    assert!(matches!(call.status, AnalysisStatus::Invalid(_)), "{call:#?}");
    fixture.assert_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::GenericInferenceConflict, 1);
    fixture.assert_trace_has(run, call, |step| {
        matches!(
            step,
            ExplanationStep::GenericConstraint {
                origin: GenericConstraintOrigin::ExpectedResult,
                ..
            }
        )
    });
}

#[test]
fn hkt_argument_satisfies_declaration_restriction() {
    let fixture = Fixture::new(
        r#"
class Number {}
class Good is Number {}
class Box<T> {}

class Probe {
  @class
  keep<F: Type -> Type, A>(_ value: F<A>) -> F<A> where A <: Number { value }

  @class
  run(_ value: Box<Good>) {
    let result = Probe.keep(value)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.keep(value)");
    fixture.assert_type(call.knowledge.ty().expect("HKT constrained result"), applied("Box", [nominal("Good")]));
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
}

#[test]
fn hkt_argument_rejects_declaration_restriction() {
    let fixture = Fixture::new(
        r#"
class Comparable<T> {}
class Other {}
class Box<T> {}

class Probe {
  @class
  keep<F: Type -> Type, A>(_ value: F<A>) -> F<A> where A <: Comparable<A> { value }

  @class
  run(_ value: Box<Other>) {
    let result = Probe.keep(value)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.keep(value)");
    assert!(call.knowledge.ty().is_none(), "invalid F-bound must not publish result: {call:#?}");
    assert!(matches!(call.status, AnalysisStatus::Invalid(_)), "{call:#?}");
    fixture.assert_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::GenericConstraintUnsatisfied, 1);
}

#[test]
fn hkt_f_bound_accepts_selected_argument() {
    let fixture = Fixture::new(
        r#"
class Comparable<T> {}
class Good is Comparable<Good> {}
class Box<T> {}

class Probe {
  @class
  keep<F: Type -> Type, A>(_ value: F<A>) -> F<A> where A <: Comparable<A> { value }

  @class
  run(_ value: Box<Good>) {
    let result = Probe.keep(value)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.keep(value)");
    fixture.assert_type(call.knowledge.ty().expect("HKT F-bound result"), applied("Box", [nominal("Good")]));
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
}

#[test]
fn hkt_bound_only_remains_underconstrained() {
    let fixture = Fixture::new(
        r#"
class Number {}

class Probe {
  @class
  make<F: Type -> Type, A>() -> F<A> where A <: Number { 0 }

  @class
  run() {
    let result = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    assert!(call.knowledge.ty().is_none(), "declaration bounds must not select HKT variables: {call:#?}");
    assert!(matches!(call.status, AnalysisStatus::Blocked(_)), "{call:#?}");
    fixture.assert_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::GenericInferenceUnderconstrained, 1);
}

#[test]
fn hkt_constraint_projects_transformed_generic_supertype() {
    let fixture = Fixture::new(
        r#"
class List<T> {}
class Parent<T> {}
class Child<T> is Parent<List<T>> {}

class Probe {
  @class
  keep<F: Type -> Type, A>(_ value: F<A>) -> F<A> where F<A> <: Parent<List<A>> { value }

  @class
  run(_ value: Child<Int>) {
    let result = Probe.keep(value)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.keep(value)");
    fixture.assert_type(
        call.knowledge.ty().expect("transformed generic supertype result"),
        applied("Child", [nominal("Int")]),
    );
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
}

#[test]
fn hkt_self_specializes_to_direct_receiver() {
    let fixture = Fixture::new(
        r#"
class List<T> {}
class Parent<F: Type -> Type> {
  wrap() -> F<Self> { 0 }
}
class Child is Parent<List> {}

class Probe {
  @class
  run(_ child: Child) {
    let result = child.wrap()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let result = fixture.binding(run, "result");
    fixture.assert_type(result.current.ty().expect("direct HKT Self result"), applied("List", [nominal("Child")]));
}

#[test]
fn hkt_self_specializes_through_multi_hop_transformed_inheritance() {
    let fixture = Fixture::new(
        r#"
class List<T> {}
class Base<F: Type -> Type> {
  wrap() -> F<Self> { 0 }
}
class Middle<F: Type -> Type> is Base<F> {}
class Leaf is Middle<List> {}

class Probe {
  @class
  run(_ leaf: Leaf) {
    let result = leaf.wrap()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let result = fixture.binding(run, "result");
    fixture.assert_type(result.current.ty().expect("multi-hop HKT Self result"), applied("List", [nominal("Leaf")]));
}
