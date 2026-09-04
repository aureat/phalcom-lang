use crate::semantic::support::Fixture;
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::checker::causal::CausalInvalidity;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::{EvidenceStatus, TypeKnowledge, UnknownReason};
use phalcom_semantic::types::row::RecordRowTail;
use phalcom_semantic::types::store::TypeData;

#[test]
fn invalid_initializer_expression_contains_its_own_cause() {
    let fixture = Fixture::new(
        r#"
class CellNum {
  @constructor
  new() {}
}

class Probe {
  @class
  run() {
    let x: Int = CellNum.new()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let initializer = fixture.expression(run, "CellNum.new()");
    let AnalysisStatus::Invalid(cause) = initializer.status else {
        panic!("initializer mismatch must own Invalid status: {initializer:#?}");
    };
    assert!(initializer.causal_invalidity.contains(cause));
}

#[test]
fn list_unknown_element_does_not_disappear() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let xs = [1, missing]
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let expected = TypeKnowledge::Unknown(UnknownReason::UnresolvedName("missing".into()));
    assert_eq!(fixture.expression(run, "[1, missing]").knowledge, expected);
    assert_eq!(fixture.binding(run, "xs").current, expected);
}

#[test]
fn assumed_list_element_weakens_aggregate() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run(_ value: Int) {
    let xs = [1, value]
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(fixture.binding(run, "xs").current.status(), Some(EvidenceStatus::Assumed));
}

#[test]
fn map_unknown_key_and_value_do_not_disappear() {
    let value_fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let values = { key: missing }
  }
}
"#,
    );
    let run = value_fixture.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(
        value_fixture.expression(run, "{ key: missing }").knowledge,
        TypeKnowledge::Unknown(UnknownReason::UnresolvedName("missing".into()))
    );

    let key_fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let values = { [missing]: 1 }
  }
}
"#,
    );
    let run = key_fixture.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(
        key_fixture.expression(run, "{ [missing]: 1 }").knowledge,
        TypeKnowledge::Unknown(UnknownReason::UnresolvedName("missing".into()))
    );
}

#[test]
fn assumed_map_value_weakens_map_evidence() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run(_ value: Int) {
    let values = { key: value }
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(fixture.expression(run, "{ key: value }").knowledge.status(), Some(EvidenceStatus::Assumed));
}

#[test]
fn tuple_and_record_members_preserve_unknown_and_assumed_state() {
    let unknown = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let pair = (1, missing)
    let record = #{field: missing}
  }
}
"#,
    );
    let run = unknown.callable("Probe", "run", DispatchSide::Class);
    let expected = TypeKnowledge::Unknown(UnknownReason::UnresolvedName("missing".into()));
    assert_eq!(unknown.expression(run, "(1, missing)").knowledge, expected);
    assert_eq!(unknown.expression(run, "#{field: missing}").knowledge, expected);

    let assumed = Fixture::new(
        r#"
class Probe {
  @class
  run(_ value: Int) {
    let pair = (1, value)
    let record = #{field: value}
  }
}
"#,
    );
    let run = assumed.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(assumed.expression(run, "(1, value)").knowledge.status(), Some(EvidenceStatus::Assumed));
    assert_eq!(assumed.expression(run, "#{field: value}").knowledge.status(), Some(EvidenceStatus::Assumed));
}

#[test]
fn tuple_pattern_preserves_unknown_source_reason() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let (left, right) = missing
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let expected = TypeKnowledge::Unknown(UnknownReason::UnresolvedName("missing".into()));
    assert_eq!(fixture.binding(run, "left").current, expected);
    assert_eq!(fixture.binding(run, "right").current, expected);
}

#[test]
fn tuple_pattern_preserves_causal_invalidity_without_suppressing_known_components() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let pair: Int = (1, 2)
    let (left, right) = pair
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let left = fixture.binding(run, "left");
    let right = fixture.binding(run, "right");
    assert_eq!(left.current.ty(), Some(fixture.ty("Int")));
    assert_eq!(right.current.ty(), Some(fixture.ty("Int")));
    assert!(!matches!(left.causal_invalidity, CausalInvalidity::Clean));
    assert!(!matches!(right.causal_invalidity, CausalInvalidity::Clean));
    assert!(!matches!(left.current, TypeKnowledge::Unknown(_)));
    assert!(!matches!(right.current, TypeKnowledge::Unknown(_)));
}

#[test]
fn exact_expansions_contribute_to_aggregate_shapes() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let values = [1, 2]
    let list = [*values, 3]
    let tuple_values = (1, 2)
    let tuple = (*tuple_values, 3)
    let mapping = { base: 1 }
    let map = { **mapping, key: 2 }
    let source = #{base: 1}
    let record = #{**source, name: 2}
  }
}

"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    assert!(fixture.binding(run, "list").current.ty().is_some());
    assert!(fixture.binding(run, "tuple").current.ty().is_some());
    assert!(fixture.binding(run, "map").current.ty().is_some());
    assert!(fixture.binding(run, "record").current.ty().is_some());
}

#[test]
fn expected_record_field_guides_nested_empty_collection() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let result: #{ values: List<Int> } = #{values: []}
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let result = fixture.binding(run, "result").current.ty().expect("record result");
    let TypeData::Record(row_id) = fixture.analysis.snapshot.store.get(result) else {
        panic!("expected Record result");
    };
    let values = fixture.analysis.snapshot.store.record_row(*row_id).find_field("values").expect("values field");
    assert_eq!(fixture.analysis.snapshot.store.format_type(values), "List<Int>");
}

#[test]
fn open_record_expansion_preserves_tail_and_allows_disjoint_extension() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  spread<R: RecordRow>(_ value: #{ known: Int, | R }) -> #{ known: Int, extra: Bool, | R } {
    let result = #{ **value, extra: true }
    result
  }
}
"#,
    );
    let spread = fixture.callable("Probe", "spread", DispatchSide::Class);
    let result = fixture.binding(spread, "result").current.ty().expect("expanded record result");
    let TypeData::Record(row_id) = fixture.analysis.snapshot.store.get(result) else {
        panic!("expected Record result");
    };
    let row = fixture.analysis.snapshot.store.record_row(*row_id);
    assert!(matches!(row.tail, RecordRowTail::Parameter(_)));
    assert_eq!(row.fields.iter().map(|field| field.name.as_ref()).collect::<Vec<_>>(), vec!["extra", "known"]);
}

#[test]
fn concrete_record_annotation_proves_open_extension_disjointness() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  extend<R: RecordRow>(_ value: #{ known: Int, | R }) {
    let result: #{ known: Int, extra: Bool, | R } = #{ **value, extra: true }
  }
}
"#,
    );
    let extend = fixture.callable("Probe", "extend", DispatchSide::Class);
    let result = fixture.binding(extend, "result").current.clone();

    assert!(result.ty().is_some(), "concrete Record annotation should prove extension: {result:#?}");
    assert!(fixture.diagnostics(DiagnosticCode::RecordRowLacksUnproven).is_empty());
}

#[test]
fn open_record_extension_requires_proven_tail_absence() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  extend<R: RecordRow>(_ value: #{ known: Int, | R }) {
    let result = #{ **value, extra: true }
  }
}
"#,
    );
    let extend = fixture.callable("Probe", "extend", DispatchSide::Class);
    let literal = fixture.expression(extend, "#{ **value, extra: true }");

    assert!(literal.knowledge.ty().is_none(), "unproved extension must not publish a type: {literal:#?}");
    assert!(
        matches!(literal.status, AnalysisStatus::Invalid(_)),
        "unproved extension must be invalid: {literal:#?}"
    );
    fixture.assert_diagnostic(DiagnosticCode::RecordRowLacksUnproven, 1);
    let diagnostic = fixture
        .diagnostics(DiagnosticCode::RecordRowLacksUnproven)
        .into_iter()
        .next()
        .expect("unproven row lack diagnostic");
    assert!(diagnostic.message.contains("extra"));
    assert!(!diagnostic.message.contains("RecordRowVarId"));
}

#[test]
fn duplicate_record_literal_field_has_explicit_diagnostic() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let result = #{ known: 1, known: 2 }
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let literal = fixture.expression(run, "#{ known: 1, known: 2 }");

    assert!(literal.knowledge.ty().is_none());
    assert!(
        matches!(literal.status, AnalysisStatus::Invalid(_)),
        "duplicate literal must be invalid: {literal:#?}"
    );
    fixture.assert_diagnostic(DiagnosticCode::RecordDuplicateField, 1);
}

#[test]
fn duplicate_record_literal_field_from_expansion_has_explicit_diagnostic() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let source = #{ known: 1 }
    let result = #{ **source, known: 2 }
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let literal = fixture.expression(run, "#{ **source, known: 2 }");

    assert!(literal.knowledge.ty().is_none());
    assert!(
        matches!(literal.status, AnalysisStatus::Invalid(_)),
        "expanded duplicate must be invalid: {literal:#?}"
    );
    fixture.assert_diagnostic(DiagnosticCode::RecordDuplicateField, 1);
}

#[test]
fn incompatible_open_record_expansions_have_explicit_diagnostic() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  merge<R: RecordRow, S: RecordRow>(
      _ left: #{ left: Int, | R },
      _ right: #{ right: Int, | S }
  ) {
    let result = #{ **left, **right }
  }
}
"#,
    );
    let merge = fixture.callable("Probe", "merge", DispatchSide::Class);
    let literal = fixture.expression(merge, "#{ **left, **right }");

    assert!(literal.knowledge.ty().is_none());
    assert!(
        matches!(literal.status, AnalysisStatus::Invalid(_)),
        "incompatible open tails must be invalid: {literal:#?}"
    );
    fixture.assert_diagnostic(DiagnosticCode::RecordRowInferenceConflict, 1);
}

#[test]
fn map_expansion_does_not_fabricate_record_row() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let source = { base: 1 }
    let result = #{ **source, name: 2 }
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    assert!(matches!(fixture.binding(run, "result").current, TypeKnowledge::Unknown(_)));
}

#[test]
fn assumed_expansion_weakens_aggregate() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run(_ value: Int) {
    let values = [value]
    let list = [*values, 1]
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(fixture.binding(run, "list").current.status(), Some(EvidenceStatus::Assumed));
}

#[test]
fn unknown_expansions_do_not_disappear() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let list = [*missing, 1]
    let tuple = (*missing, 1)
    let map = { **missing, key: 1 }
    let record = #{**missing, key: 1}
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let expected = TypeKnowledge::Unknown(UnknownReason::UnresolvedName("missing".into()));
    for expression in ["[*missing, 1]", "(*missing, 1)", "{ **missing, key: 1 }", "#{**missing, key: 1}"] {
        assert_eq!(fixture.expression(run, expression).knowledge, expected, "{expression}");
    }
}

#[test]
fn for_loop_preserves_iterable_unknown_reason() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    for value in missing {
      let copy = value
    }
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let values = fixture.bindings_named(run, "value");
    assert!(!values.is_empty());
    assert_eq!(values[0].current, TypeKnowledge::Unknown(UnknownReason::UnresolvedName("missing".into())));
}

#[test]
fn invalid_known_dependency_remains_analyzable_with_causal_state() {
    let fixture = Fixture::new(
        r#"
class CellNum {
  @constructor
  new() {}
  value() -> Int { 1 }
}

class Probe {
  @class
  run(_ assumed: Int) {
    let bad: Int = CellNum.new()
    let known = bad.value()
    let list = [1, assumed]
    let unresolved = [1, missing]
  }
}
"#,
    );
    fixture.assert_expression_product_invariants();
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let known = fixture.expression(run, "bad.value()");
    assert!(matches!(known.status, AnalysisStatus::Ready));
    assert!(!matches!(known.causal_invalidity, CausalInvalidity::Clean));
}
