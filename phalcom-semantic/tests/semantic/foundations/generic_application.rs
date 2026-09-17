//! SC-2 generic application and expected-context laws.

use crate::semantic::support::{Fixture, applied, nominal};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::{EvidenceOrigin, EvidenceStatus, UnknownReason};

#[test]
fn result_only_generic_uses_expected_context_as_assumed_selection() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  make<T>() -> T { 42 }

  @class
  run() {
    let value: Int = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("Int")));
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Assumed));
    assert_eq!(call.knowledge.origin(), Some(EvidenceOrigin::GenericInference));
    assert!(matches!(call.status, AnalysisStatus::Ready));
    fixture.assert_no_diagnostic(DiagnosticCode::GenericInferenceUnderconstrained);
}

#[test]
fn solved_generic_call_publishes_canonical_call_specialization() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  make<T>() -> T { 42 }

  @class
  run() {
    let value: Int = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    let specialization = call.call_specialization.as_ref().expect("solved generic call product");
    assert_eq!(specialization.callable, call.callable.clone().expect("callable identity"));
    assert_eq!(specialization.bindings.len(), 1);
    assert_eq!(specialization.bindings[0].1, fixture.ty("Int"));
}

#[test]
fn result_only_generic_without_context_remains_underconstrained() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  make<T>() -> T { 42 }

  @class
  run() {
    let value = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    assert_eq!(
        call.knowledge,
        phalcom_semantic::types::evidence::TypeKnowledge::Unknown(UnknownReason::UnderconstrainedTypeVariable)
    );
    assert!(matches!(call.status, AnalysisStatus::Blocked(_)));
    fixture.assert_diagnostic(DiagnosticCode::GenericInferenceUnderconstrained, 1);
}

#[test]
fn declaration_upper_bound_does_not_default_result_only_generic() {
    let fixture = Fixture::new(
        r#"
class Number {}
class Probe {
  @class
  make<T>() -> T where T <: Number { 42 }

  @class
  run() {
    let value = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    assert!(call.knowledge.ty().is_none());
    assert!(matches!(call.status, AnalysisStatus::Blocked(_)));
    fixture.assert_diagnostic(DiagnosticCode::GenericInferenceUnderconstrained, 1);
}

#[test]
fn candidate_is_checked_against_declaration_upper_bound() {
    let fixture = Fixture::new(
        r#"
class Number {}
class Allowed is Number { @constructor new() {} }
class Probe {
  @class
  keep<T>(_ value: T) -> T where T <: Number { value }

  @class
  run(_ value: Allowed) {
    let result = Probe.keep(value)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.keep(value)");
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("Allowed")));
    assert!(matches!(call.status, AnalysisStatus::Ready));
}

#[test]
fn f_bound_is_checked_after_argument_selects_candidate() {
    let fixture = Fixture::new(
        r#"
class Comparable<T> {}
class User is Comparable<User> {}
class Probe {
  @class
  keep<T>(_ value: T) -> T where T <: Comparable<T> { value }

  @class
  run(_ value: User) {
    let result = Probe.keep(value)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.keep(value)");
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("User")));
    assert!(matches!(call.status, AnalysisStatus::Ready));
    fixture.assert_no_diagnostic(DiagnosticCode::GenericConstraintUnsatisfied);
}

#[test]
fn f_bound_refutation_is_reported_as_constraint_failure() {
    let fixture = Fixture::new(
        r#"
class Comparable<T> {}
class Other {}
class Probe {
  @class
  keep<T>(_ value: T) -> T where T <: Comparable<T> { value }

  @class
  run(_ value: Other) {
    let result = Probe.keep(value)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.keep(value)");
    assert!(call.knowledge.ty().is_none());
    assert!(matches!(call.status, AnalysisStatus::Invalid(_)));
    fixture.assert_diagnostic(DiagnosticCode::GenericConstraintUnsatisfied, 1);
}

#[test]
fn expected_context_solves_nested_generic_result() {
    let fixture = Fixture::new(
        r#"
class Box<T> {
  @constructor new(_ value: T) {}
}

class Probe {
  @class
  make<T>() -> Box<T> { Box<Int>.new(1) }

  @class
  run() {
    let result: Box<Int> = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    fixture.assert_type(call.knowledge.ty().expect("nested generic result"), applied("Box", [nominal("Int")]));
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Assumed));
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
}

#[test]
fn generic_constructor_composes_declaration_and_callable_domains() {
    let fixture = Fixture::new(
        r#"
class Box<T> {
  @constructor new<U>(_ value: T, _ metadata: U) {}
}

class Probe {
  @class
  run() {
    let result: Box<Int> = Box.new(1, "metadata")
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Box.new(1, \"metadata\")");
    fixture.assert_type(call.knowledge.ty().expect("generic constructor result"), applied("Box", [nominal("Int")]));
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");

    let constructor = fixture
        .analysis
        .snapshot
        .callable_signatures
        .iter()
        .map(|(_, signature)| signature)
        .find(|signature| signature.owner.name.as_ref() == "Box" && signature.side == DispatchSide::Class)
        .expect("Box constructor signature");
    let generic_signature = constructor.generics.as_ref().expect("constructor-local generic signature");
    assert_eq!(generic_signature.parameter_count(), 1);
    assert!(matches!(
        generic_signature.owner,
        phalcom_semantic::types::parameter::TypeParameterOwner::Callable(_)
    ));
}

#[test]
fn instance_method_on_generic_class_publishes_owner_call_specialization() {
    let fixture = Fixture::new(
        r#"
class Box<T> {
  @constructor new() {}
  fetch(_ x: Int) -> T { 42 }
}

class Probe {
  @class
  run() {
    let b: Box<Int> = Box<Int>.new()
    let value = b.fetch(1)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let diags = fixture.analysis.snapshot.diagnostics();
    println!("DIAGNOSTICS: {diags:?}");
    let call = fixture.expression(run, "b.fetch(1)");
    println!("CALL: {call:#?}");
    let specialization = call.call_specialization.as_ref().expect("instance method call on generic class should have call_specialization");
    assert_eq!(specialization.bindings.len(), 1);
    assert_eq!(specialization.bindings[0].1, fixture.ty("Int"));
}

#[test]
fn range_like_option_unwrap_or_call_specialization() {
    let fixture = Fixture::new(
        r#"
class Range {
  _$lower -> Option<Number> { Some(0) }
}

class RangeIterator {
  _range: Range
  @constructor new(_ r: Range) { _range = r }
  next() {
    let lowerOpt = _range._$lower
    let lower = lowerOpt.unwrapOr(0)
  }
}
"#,
    );
    let run = fixture.callable("RangeIterator", "next", DispatchSide::Instance);
    let call = fixture.expression(run, "lowerOpt.unwrapOr(0)");
    println!("RANGE CALL: {call:#?}");
    let specialization = call.call_specialization.as_ref().expect("lowerOpt.unwrapOr(0) should have call_specialization");
    println!("SPECIALIZATION: {specialization:#?}");
}

#[test]
fn range_iterator_next_is_some_call_specialization() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let exclusive = (1..4).iterator
    let directFirst = exclusive.next.isSome
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let exclusive = fixture.expression(run, "(1..4).iterator");
    println!("EXCLUSIVE: {exclusive:#?}");
    let exclusive_next = fixture.expression(run, "exclusive.next");
    println!("EXCLUSIVE_NEXT: {exclusive_next:#?}");
    let is_some = fixture.expression(run, "exclusive.next.isSome");
    println!("IS_SOME: {is_some:#?}");
}




