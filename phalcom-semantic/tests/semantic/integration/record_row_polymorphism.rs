use crate::semantic::support::Fixture;
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::TypeKnowledge;
use phalcom_semantic::types::row::RecordRowTail;
use phalcom_semantic::types::store::TypeData;

#[test]
fn row_polymorphic_call_preserves_remainder_in_return() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  preserve<R: RecordRow>(_ value: #{ item: Int, | R }) -> #{ item: Int, | R } {
    value
  }

  @class
  run() {
    let result = Probe.preserve(#{item: 1, name: "x"})
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.preserve(#{item: 1, name: \"x\"})");
    let result = call.knowledge.ty().expect("row-polymorphic call result");
    let TypeData::Record(row_id) = fixture.analysis.snapshot.store.get(result) else {
        panic!("expected Record result, got {:?}", fixture.analysis.snapshot.store.get(result));
    };
    let row = fixture.analysis.snapshot.store.record_row(*row_id);
    assert_eq!(row.tail, RecordRowTail::Closed);
    assert_eq!(row.fields.len(), 2);
    assert!(matches!(call.status, AnalysisStatus::Ready));
    fixture.assert_no_diagnostic(DiagnosticCode::GenericInferenceConflict);
}

#[test]
fn type_and_row_variables_infer_together() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  preserve<T, R: RecordRow>(_ value: #{ item: T, | R }) -> #{ item: T, | R } {
    value
  }

  @class
  run() {
    let result = Probe.preserve(#{item: 1, name: "x"})
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.preserve(#{item: 1, name: \"x\"})");
    let result = call.knowledge.ty().expect("combined generic result");
    let TypeData::Record(row_id) = fixture.analysis.snapshot.store.get(result) else {
        panic!("expected Record result, got {:?}", fixture.analysis.snapshot.store.get(result));
    };
    let row = fixture.analysis.snapshot.store.record_row(*row_id);
    assert_eq!(row.tail, RecordRowTail::Closed);
    assert_eq!(row.fields.iter().map(|field| field.name.as_ref()).collect::<Vec<_>>(), vec!["item", "name"]);
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
    fixture.assert_no_diagnostic(DiagnosticCode::GenericInferenceConflict);
}

#[test]
fn repeated_row_parameter_conflict_is_rejected() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  same<R: RecordRow>(_ left: #{ left: Int, | R }, _ right: #{ right: Int, | R }) -> #{ left: Int, | R } {
    left
  }

  @class
  run() {
    let result = Probe.same(#{left: 1, extra: "x"}, #{right: 2, other: "y"})
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.same(#{left: 1, extra: \"x\"}, #{right: 2, other: \"y\"})");
    assert!(call.knowledge.ty().is_none(), "conflicting row should not publish a type: {call:#?}");
    assert!(matches!(call.status, AnalysisStatus::Invalid(_)), "{call:#?}");
    fixture.assert_diagnostic(DiagnosticCode::RecordRowInferenceConflict, 1);
}

#[test]
fn expected_result_can_select_row() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  make<R: RecordRow>() -> #{ item: Int, | R } {
    #{item: 1}
  }

  @class
  run() {
    let result: #{ item: Int, name: String } = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    let result = call.knowledge.ty().expect("contextual row result");
    let TypeData::Record(row_id) = fixture.analysis.snapshot.store.get(result) else {
        panic!("expected Record result, got {:?}", fixture.analysis.snapshot.store.get(result));
    };
    let row = fixture.analysis.snapshot.store.record_row(*row_id);
    assert_eq!(row.tail, RecordRowTail::Closed);
    assert_eq!(row.fields.iter().map(|field| field.name.as_ref()).collect::<Vec<_>>(), vec!["item", "name"]);
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
}

#[test]
fn row_only_return_is_underconstrained_without_context() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  make<R: RecordRow>() -> #{ item: Int, | R } {
    #{item: 1}
  }

  @class
  run() {
    let result = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    assert!(call.knowledge.ty().is_none(), "underconstrained row must not close: {call:#?}");
    assert!(matches!(call.status, AnalysisStatus::Blocked(_)), "{call:#?}");
    fixture.assert_diagnostic(DiagnosticCode::RecordRowInferenceUnderconstrained, 1);
}

#[test]
fn open_record_pattern_binds_known_prefix_without_fabricating_tail_fields() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  inspect<R: RecordRow>(_ value: #{ known: Int, | R }) {
    match value {
      #{ known: item } => item
      #{ missing: other } => other
      _ => 0
    }
  }
}
"#,
    );
    let inspect = fixture.callable("Probe", "inspect", DispatchSide::Class);
    let resolution = inspect.match_resolutions.values().next().expect("record match resolution");
    assert_eq!(resolution.arms[0].bindings[0].knowledge.ty(), Some(fixture.ty("Int")));
    assert_ne!(resolution.arms[1].bindings[0].knowledge.ty(), Some(fixture.ty("Int")));
    fixture.assert_no_diagnostic(DiagnosticCode::MatchPatternFieldMismatch);
}

#[test]
fn record_annotation_reports_unresolved_row_tail() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let value: #{ known: Int, | MissingRow } = #{known: 1}
  }
}
"#,
    );
    fixture.assert_diagnostic(DiagnosticCode::RecordRowTailUnresolved, 1);
}

#[test]
fn record_annotation_reports_duplicate_known_field() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let value: #{ known: Int, known: String } = #{known: 1}
  }
}
"#,
    );
    fixture.assert_diagnostic(DiagnosticCode::RecordDuplicateField, 1);
}

#[test]
fn row_conflict_message_does_not_expose_solver_variables() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  same<R: RecordRow>(_ left: #{ left: Int, | R }, _ right: #{ right: Int, | R }) -> #{ left: Int, | R } {
    left
  }

  @class
  run() {
    Probe.same(#{left: 1, extra: "x"}, #{right: 2, other: "y"})
  }
}
"#,
    );
    let diagnostic = fixture
        .diagnostics(DiagnosticCode::RecordRowInferenceConflict)
        .into_iter()
        .next()
        .expect("row conflict diagnostic");
    assert!(!diagnostic.message.contains("RecordRowVarId"));
    assert!(!diagnostic.message.contains("InferenceVar"));
    assert_eq!(fixture.diagnostics(DiagnosticCode::RecordRowInferenceConflict).len(), 1);
}

#[test]
fn return_record_prefix_contributes_lacks_constraint_to_row_inference() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  tagged<R: RecordRow>(
      _ value: #{ name: String, | R }
  ) -> #{ name: String, tag: String, | R } {
      #{ **value, tag: "entity" }
  }

  @class
  run() {
      let result = Probe.tagged(#{ name: "Phalcom", tag: "existing" })
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.tagged(#{ name: \"Phalcom\", tag: \"existing\" })");

    assert!(call.knowledge.ty().is_none(), "rejected row call must not publish a type: {call:#?}");
    assert!(
        matches!(call.knowledge, TypeKnowledge::Unknown(_)),
        "rejected row call must not become Dynamic: {call:#?}"
    );
    assert!(
        matches!(call.status, AnalysisStatus::Invalid(_)),
        "return-only row lack must invalidate call: {call:#?}"
    );
    fixture.assert_diagnostic(DiagnosticCode::RecordRowLacksViolation, 1);
    let diagnostic = fixture
        .diagnostics(DiagnosticCode::RecordRowLacksViolation)
        .into_iter()
        .next()
        .expect("row lacks diagnostic");
    assert!(!diagnostic.message.contains("RecordRowVarId"));
    assert!(!diagnostic.message.contains("InferVarId"));
}
