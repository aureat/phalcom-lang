//! Canonical callable-application regressions.

use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DispatchSide};
use phalcom_semantic::types::evidence::{DynamicReason, EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};

use crate::semantic::support::Fixture;

#[test]
fn canonical_call_diagnostic_codes_are_stable() {
    assert_eq!(DiagnosticCode::CallShapeMismatch.as_str(), "type.call.shape_mismatch");
    assert_eq!(DiagnosticCode::NotCallable.as_str(), "type.call.not_callable");
}

#[test]
fn fixed_method_return_survives_argument_mismatch() {
    let fixture = Fixture::new(
        r#"
class Probe {
  accept(_ value: Int) -> Int {
    1
  }

  run() {
    self.accept("wrong")
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let call = fixture.expression(run, r#"self.accept("wrong")"#);
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("Int")));
    assert!(matches!(call.status, AnalysisStatus::Invalid(cause) if call.causal_invalidity.contains(cause)));
    assert_eq!(call.callable.as_ref(), Some(&fixture.callable_id("Probe", "accept", DispatchSide::Instance)));
    fixture.assert_diagnostic(DiagnosticCode::ArgumentMismatch, 1);
}

#[test]
fn explicit_method_call_publishes_exact_callable_identity() {
    let fixture = Fixture::new(
        r#"
class Probe {
  value() -> Int { 1 }
  run() { self.value() }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let call = fixture.expression(run, "self.value()");
    assert_eq!(call.callable.as_ref(), Some(&fixture.callable_id("Probe", "value", DispatchSide::Instance)));
}

#[test]
fn getter_call_publishes_exact_callable_identity() {
    let fixture = Fixture::new(
        r#"
class Box {
  value -> Int { 1 }
  run() { self.value }
}
"#,
    );
    let run = fixture.callable("Box", "run", DispatchSide::Instance);
    let getter = fixture.expression(run, "self.value");
    assert_eq!(getter.callable.as_ref(), Some(&fixture.callable_id("Box", "value", DispatchSide::Instance)));
}

#[test]
fn assumed_receiver_caps_fixed_method_result() {
    let fixture = Fixture::new(
        r#"
class Worker { value() -> Int { 1 } }
class Probe { run(worker: Worker) { worker.value() } }
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let call = fixture.expression(run, "worker.value()");
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("Int")));
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Assumed));
}

#[test]
fn dispatch_miss_still_analyzes_argument_expressions() {
    let fixture = Fixture::new(
        r#"
class Probe { run() { self.noSuchMethod(missing) } }
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let argument = fixture.expression(run, "missing");
    assert_eq!(argument.knowledge, TypeKnowledge::Unknown(UnknownReason::UnresolvedName("missing".into())));
    assert!(fixture.expression(run, "self.noSuchMethod(missing)").knowledge.is_unknown());
}

#[test]
fn spread_call_shape_is_dynamic_not_one_positional_slot() {
    let fixture = Fixture::new(
        r#"
class Receiver { target(_ value: Int) -> Int { value } }
class Probe { run(receiver: Receiver, values) { receiver.target(*values) } }
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let call = fixture.expression(run, "receiver.target(*values)");
    assert_eq!(call.knowledge, TypeKnowledge::Dynamic(DynamicReason::DynamicRestPack));
    assert_eq!(call.status, AnalysisStatus::DynamicBoundary(DynamicReason::DynamicRestPack));
}

#[test]
fn invoking_known_non_callable_is_invalid_not_a_value_read() {
    let fixture = Fixture::new(
        r#"
class Probe { run() { let value = 1; value() } }
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let call = fixture.expression(run, "value()");
    assert_ne!(call.knowledge.ty(), Some(fixture.ty("Int")));
    assert!(matches!(call.status, AnalysisStatus::Invalid(_)));
    fixture.assert_diagnostic(DiagnosticCode::NotCallable, 1);
}

#[test]
fn lexical_unknown_callee_does_not_fall_back_to_self_method() {
    let fixture = Fixture::new(
        r#"
class Probe {
  helper() -> Int { 1 }
  run(value) { let helper = value; helper() }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    assert!(fixture.expression(run, "helper()").callable.is_none());
}

#[test]
fn binary_operator_checks_rhs_parameter_relation() {
    let fixture = Fixture::new(
        r#"
class Probe { run() { 1 + "wrong" } }
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let expr = fixture.expression(run, r#"1 + "wrong""#);
    assert_eq!(expr.knowledge.ty(), Some(fixture.ty("Int")));
    assert!(matches!(expr.status, AnalysisStatus::Invalid(cause) if expr.causal_invalidity.contains(cause)));
    assert_eq!(expr.callable.as_ref().map(|_| true), Some(true));
    fixture.assert_diagnostic(DiagnosticCode::ArgumentMismatch, 1);
}

#[test]
fn bilateral_operator_keeps_direct_priority_when_both_targets_are_viable() {
    let fixture = Fixture::new(
        r#"
class DirectResult {}
class ReflectedResult {}

class Operand {
  @constructor new() {}
  +(_ other: Operand) -> DirectResult { DirectResult.new() }
  +(from other: Operand) -> ReflectedResult { ReflectedResult.new() }
}

class Probe {
  run() {
    let left = Operand.new()
    let right = Operand.new()
    left + right
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let expression = fixture.expression(run, "left + right");
    let direct = CallableId::new(
        fixture.decl("Operand"),
        Selector::method("+", vec![SelectorSlot::Positional]).unwrap(),
        DispatchSide::Instance,
    );

    assert_eq!(expression.knowledge.ty(), Some(fixture.ty("DirectResult")), "{expression:#?}");
    assert_eq!(expression.callable.as_ref(), Some(&direct), "{expression:#?}");
    assert!(matches!(expression.status, AnalysisStatus::Ready), "{expression:#?}");
}

#[test]
fn bilateral_operator_uses_reflected_target_when_direct_contract_is_impossible() {
    let fixture = Fixture::new(
        r#"
class ReflectedResult {}

class Operand {
  @constructor new() {}
  +(from other: Int) -> ReflectedResult { ReflectedResult.new() }
}

class Probe {
  run() {
    let right = Operand.new()
    1 + right
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let expression = fixture.expression(run, "1 + right");
    let reflected = CallableId::new(
        fixture.decl("Operand"),
        Selector::method("+", vec![SelectorSlot::Label("from".to_string())]).unwrap(),
        DispatchSide::Instance,
    );

    assert_eq!(expression.knowledge.ty(), Some(fixture.ty("ReflectedResult")), "{expression:#?}");
    assert_eq!(expression.callable.as_ref(), Some(&reflected), "{expression:#?}");
    assert!(matches!(expression.status, AnalysisStatus::Ready), "{expression:#?}");
    fixture.assert_diagnostic(DiagnosticCode::ArgumentMismatch, 0);
    for source_text in ["1", "right"] {
        let analyzed = run
            .expressions
            .values()
            .filter(|candidate| fixture.source.get(candidate.range.start..candidate.range.end) == Some(source_text))
            .count();
        assert_eq!(analyzed, 1, "binary operand `{source_text}` must be analyzed exactly once");
    }
}

#[test]
fn bilateral_operator_prefers_rhs_strict_subtype_reflected_override() {
    let fixture = Fixture::new(
        r#"
class DirectResult {}
class ReflectedResult {}

class BaseOperand {
  @constructor new() {}
  +(_ other: BaseOperand) -> DirectResult { DirectResult.new() }
}

class SubOperand is BaseOperand {
  @constructor new() { super.new() }
  +(from other: BaseOperand) -> ReflectedResult { ReflectedResult.new() }
}

class Probe {
  run() {
    let left = BaseOperand.new()
    let right = SubOperand.new()
    left + right
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let expression = fixture.expression(run, "left + right");
    let reflected = CallableId::new(
        fixture.decl("SubOperand"),
        Selector::method("+", vec![SelectorSlot::Label("from".to_string())]).unwrap(),
        DispatchSide::Instance,
    );

    assert_eq!(expression.knowledge.ty(), Some(fixture.ty("ReflectedResult")), "{expression:#?}");
    assert_eq!(expression.callable.as_ref(), Some(&reflected), "{expression:#?}");
    assert!(matches!(expression.status, AnalysisStatus::Ready), "{expression:#?}");
}

#[test]
fn reflected_generic_operator_constrains_preanalyzed_lhs_against_parameter() {
    let fixture = Fixture::new(
        r#"
class Operand {
  @constructor new() {}
  +<T>(from other: T) -> T { other }
}

class Probe {
  run() {
    let right = Operand.new()
    1 + right
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let expression = fixture.expression(run, "1 + right");

    assert_eq!(expression.knowledge.ty(), Some(fixture.ty("Int")), "{expression:#?}");
    assert!(matches!(expression.status, AnalysisStatus::Ready), "{expression:#?}");
    fixture.assert_diagnostic(DiagnosticCode::ArgumentMismatch, 0);
}

#[test]
fn unary_operator_respects_receiver_authority() {
    let fixture = Fixture::new(
        r#"
class Probe { run(value: Int) { -value } }
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    assert_eq!(fixture.expression(run, "-value").knowledge.status(), Some(EvidenceStatus::Assumed));
}

#[test]
fn direct_field_assignment_expression_is_unit() {
    let fixture = Fixture::new(
        r#"
class Box {
  _value: Int = 0

  run() {
    let result = (self._value = 1)
  }
}
"#,
    );
    let run = fixture.callable("Box", "run", DispatchSide::Instance);
    assert_eq!(
        fixture.expression(run, "self._value = 1").knowledge.ty(),
        Some(fixture.analysis.snapshot.store.unit())
    );
}

#[test]
fn direct_field_assignment_keeps_field_mismatch_status() {
    let fixture = Fixture::new(
        r#"
class Box {
  _value: Int = 0
  run() { self._value = "wrong" }
}
"#,
    );
    let run = fixture.callable("Box", "run", DispatchSide::Instance);
    let assignment = fixture.expression(run, r#"self._value = "wrong""#);
    assert_eq!(assignment.knowledge.ty(), Some(fixture.analysis.snapshot.store.unit()));
    assert!(
        matches!(assignment.status, AnalysisStatus::Invalid(cause) if assignment.causal_invalidity.contains(cause)),
        "{assignment:#?}"
    );
    fixture.assert_diagnostic(DiagnosticCode::FieldMismatch, 1);
}

#[test]
fn setter_assignment_expression_is_unit_and_keeps_callable() {
    let fixture = Fixture::new(
        r#"
class Box {
  _value: Int = 0
  value { _value }
  value=(put next: Int) { _value = next }
  run() { let result = (self.value = 1) }
}
"#,
    );
    let run = fixture.callable("Box", "run", DispatchSide::Instance);
    let assignment = fixture.expression(run, "self.value = 1");
    assert_eq!(assignment.knowledge.ty(), Some(fixture.analysis.snapshot.store.unit()));
    let setter = CallableId::new(fixture.decl("Box"), Selector::setter("value").unwrap(), DispatchSide::Instance);
    assert_eq!(assignment.callable.as_ref(), Some(&setter));
}

#[test]
fn setter_assignment_checks_value_and_keeps_unit() {
    let fixture = Fixture::new(
        r#"
class Box {
  _value: Int = 0
  value=(put next: Int) { _value = next }
  run() { self.value = "wrong" }
}
"#,
    );
    let run = fixture.callable("Box", "run", DispatchSide::Instance);
    let assignment = fixture.expression(run, r#"self.value = "wrong""#);
    assert_eq!(assignment.knowledge.ty(), Some(fixture.analysis.snapshot.store.unit()));
    assert!(matches!(assignment.status, AnalysisStatus::Invalid(_)));
    fixture.assert_diagnostic(DiagnosticCode::ArgumentMismatch, 1);
}

#[test]
fn list_subscript_checks_index_contract() {
    let fixture = Fixture::new(
        r#"
class Probe { run() { let values: List<Int> = [1]; values["wrong"] } }
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let index = fixture.expression(run, r#"values["wrong"]"#);
    assert_eq!(index.knowledge.ty(), Some(fixture.ty("Int")));
    assert!(!matches!(index.status, AnalysisStatus::Ready));
    fixture.assert_diagnostic(DiagnosticCode::ArgumentMismatch, 1);
}

#[test]
fn map_subscript_checks_key_contract() {
    let fixture = Fixture::new(
        r#"
class Probe { run() { let values: Map<String, Int> = { key: 1 }; values[1] } }
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let index = fixture.expression(run, "values[1]");
    assert_eq!(index.knowledge.ty(), Some(fixture.ty("Int")));
    assert!(!matches!(index.status, AnalysisStatus::Ready));
    fixture.assert_diagnostic(DiagnosticCode::ArgumentMismatch, 1);
}

#[test]
fn assumed_list_receiver_caps_structural_index_result() {
    let fixture = Fixture::new(
        r#"
class Probe { run(values: List<Int>) { values[0] } }
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    assert_eq!(fixture.expression(run, "values[0]").knowledge.status(), Some(EvidenceStatus::Assumed));
}

#[test]
fn list_subscript_set_checks_index_and_value_and_returns_unit() {
    let fixture = Fixture::new(
        r#"
class Probe { run() { let values: List<Int> = [1]; values["wrong"] = "wrong" } }
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let assignment = fixture.expression(run, r#"values["wrong"] = "wrong""#);
    assert_eq!(assignment.knowledge.ty(), Some(fixture.analysis.snapshot.store.unit()));
    assert!(!matches!(assignment.status, AnalysisStatus::Ready));
    assert_eq!(fixture.diagnostics(DiagnosticCode::ArgumentMismatch).len(), 2);
}

#[test]
fn user_defined_subscript_set_keeps_exact_callable_identity() {
    let fixture = Fixture::new(
        r#"
class Table {
  [_ key: Int]=(put value: String) -> String {
    value
  }

  run() { self[1] = "value" }
}
"#,
    );
    let run = fixture.callable("Table", "run", DispatchSide::Instance);
    let assignment = fixture.expression(run, r#"self[1] = "value""#);
    let selector = Selector::subscript_set(vec![SelectorSlot::Positional]).unwrap();
    let expected = CallableId::new(fixture.decl("Table"), selector, DispatchSide::Instance);
    assert_eq!(assignment.knowledge.ty(), Some(fixture.analysis.snapshot.store.unit()));
    assert_eq!(assignment.callable.as_ref(), Some(&expected));
}

#[test]
fn callable_local_uses_canonical_application_and_assumed_authority() {
    let fixture = Fixture::new(
        r#"
class Probe {
  run(worker: (Int) -> String) { worker(1) }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let call = fixture.expression(run, "worker(1)");
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("String")));
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Assumed));
    assert!(call.callable.is_none());
}

#[test]
fn known_invalid_receiver_remains_analyzable_and_causal() {
    let fixture = Fixture::new(
        r#"
class Probe {
  accept(_ value: Int) -> Int { 1 }
  run() {
    let bad = self.accept("wrong")
    bad + 1
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let call = fixture.expression(run, "bad + 1");
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("Int")));
    assert!(!matches!(call.causal_invalidity, phalcom_semantic::checker::causal::CausalInvalidity::Clean));
}

#[test]
fn generic_call_on_assumed_receiver_is_capped() {
    let fixture = Fixture::new(
        r#"
class Box {
  echo<T>(_ value: T) -> T { value }
}
class Probe {
  run(box: Box) { box.echo(1) }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let call = fixture.expression(run, "box.echo(1)");
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("Int")));
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Assumed));
}

#[test]
fn zero_argument_iteration_getter_uses_callable_application() {
    let fixture = Fixture::new(
        r#"
class Stream {
  iteratorValue -> String { "item" }
}
class Probe {
  run(stream: Stream) {
    for item in stream {
      const copy: String = item
    }
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let item = fixture.binding(run, "item");
    assert_eq!(item.current.ty(), Some(fixture.ty("String")));
}

#[test]
fn parameterized_iteration_protocol_fails_closed_without_cursor() {
    let fixture = Fixture::new(
        r#"
class Stream {
  iteratorValue(_ cursor: Int) -> String { "item" }
}
class Probe {
  run(stream: Stream) {
    for item in stream {
      const copy: String = item
    }
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let item = fixture.binding(run, "item");
    assert_eq!(item.current, TypeKnowledge::Unknown(UnknownReason::UncheckedExpression));
}

#[test]
fn constructor_keeps_instance_result_when_argument_relation_is_invalid() {
    let fixture = Fixture::new(
        r#"
class CellNum { @constructor new(_ value: Int) {} }
class Probe { run() { CellNum.new("wrong") } }
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let call = fixture.expression(run, r#"CellNum.new("wrong")"#);
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("CellNum")));
    assert_eq!(call.knowledge.origin(), Some(EvidenceOrigin::ConstructorSemantics));
    assert!(matches!(call.status, AnalysisStatus::Invalid(_)));
}

#[test]
fn constructor_result_keeps_constructor_semantics_origin() {
    let fixture = Fixture::new(
        r#"
class CellNum {
  @constructor
  new() {}
}
class Probe { run() { CellNum.new() } }
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let call = fixture.expression(run, "CellNum.new()");
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("CellNum")));
    assert_eq!(call.knowledge.origin(), Some(EvidenceOrigin::ConstructorSemantics));
}

#[test]
fn positional_rest_parameter_binds_multiple_positional_arguments() {
    let fixture = Fixture::new(
        r#"
class Probe {
  run(sum: (...Int) -> Int) {
    sum()
    sum(1)
    sum(1, 2, 3)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    assert_eq!(fixture.expression(run, "sum()").knowledge.ty(), Some(fixture.ty("Int")));
    assert_eq!(fixture.expression(run, "sum(1)").knowledge.ty(), Some(fixture.ty("Int")));
    assert_eq!(fixture.expression(run, "sum(1, 2, 3)").knowledge.ty(), Some(fixture.ty("Int")));
}

#[test]
fn labeled_rest_parameter_binds_arbitrary_labeled_arguments() {
    let fixture = Fixture::new(
        r#"
class Probe {
  run(accept: (...options: Int) -> Int) {
    accept()
    accept(options: 1)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    assert_eq!(fixture.expression(run, "accept()").knowledge.ty(), Some(fixture.ty("Int")));
    assert_eq!(fixture.expression(run, "accept(options: 1)").knowledge.ty(), Some(fixture.ty("Int")));
}

#[test]
fn exact_fixed_parameter_takes_priority_over_rest() {
    let fixture = Fixture::new(
        r#"
class Probe {
  run(format: (String, ...Int) -> String) {
    format("hello")
    format("hello", 1, 2)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    assert_eq!(fixture.expression(run, r#"format("hello")"#).knowledge.ty(), Some(fixture.ty("String")));
    assert_eq!(fixture.expression(run, r#"format("hello", 1, 2)"#).knowledge.ty(), Some(fixture.ty("String")));
}

#[test]
fn missing_fixed_parameter_fails_even_with_rest_parameter() {
    let fixture = Fixture::new(
        r#"
class Probe {
  run(format: (String, ...Int) -> String) {
    format()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Instance);
    let call = fixture.expression(run, "format()");
    assert!(matches!(call.status, AnalysisStatus::Invalid(_)));
    fixture.assert_diagnostic(DiagnosticCode::CallShapeMismatch, 1);
}
