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
  preserve<**R>(_ value: { item: Int, **R }) -> { item: Int, **R } {
    value
  }

  @class
  run() {
    let result = Probe.preserve({item: 1, name: "x"})
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.preserve({item: 1, name: \"x\"})");
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
  preserve<T, **R>(_ value: { item: T, **R }) -> { item: T, **R } {
    value
  }

  @class
  run() {
    let result = Probe.preserve({item: 1, name: "x"})
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.preserve({item: 1, name: \"x\"})");
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
  same<**R>(_ left: { left: Int, **R }, _ right: { right: Int, **R }) -> { left: Int, **R } {
    left
  }

  @class
  run() {
    let result = Probe.same({left: 1, extra: "x"}, {right: 2, other: "y"})
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.same({left: 1, extra: \"x\"}, {right: 2, other: \"y\"})");
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
  make<**R>() -> { item: Int, **R } {
    {item: 1}
  }

  @class
  run() {
    let result: { item: Int, name: String } = Probe.make()
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
    assert_eq!(call.knowledge.status(), Some(phalcom_semantic::types::evidence::EvidenceStatus::Assumed));
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
}

#[test]
fn source_established_row_stays_established_under_compatible_expected_result() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  preserve<**R>(_ value: { item: Int, **R }) -> { item: Int, **R } {
    value
  }

  @class
  run() {
    let result: { item: Int, name: String } = Probe.preserve({item: 1, name: "x"})
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.preserve({item: 1, name: \"x\"})");
    assert!(call.knowledge.ty().is_some(), "compatible source and context should publish the row: {call:#?}");
    assert_eq!(call.knowledge.status(), Some(phalcom_semantic::types::evidence::EvidenceStatus::Established));
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
}

#[test]
fn return_irrelevant_dynamic_row_does_not_weaken_exact_int_result() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  ignore<**S>(_ ignored: { tag: Int, **S }) -> Int {
    42
  }

  @class
  run(_ ignored: Dynamic) {
    Probe.ignore(ignored)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.ignore(ignored)");
    assert!(
        call.knowledge.ty().is_some(),
        "an unrelated weak row must not weaken the unrelated Int return: {call:#?}"
    );
    assert_eq!(call.knowledge.status(), Some(phalcom_semantic::types::evidence::EvidenceStatus::Established));
}

#[test]
fn row_only_return_is_underconstrained_without_context() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  make<**R>() -> { item: Int, **R } {
    {item: 1}
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
  inspect<**R>(_ value: { known: Int, **R }) {
    match value {
      { known: item } => item
      { missing: other } => other
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
    let value: { known: Int, **MissingRow } = {known: 1}
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
    let value: { known: Int, known: String } = {known: 1}
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
  same<**R>(_ left: { left: Int, **R }, _ right: { right: Int, **R }) -> { left: Int, **R } {
    left
  }

  @class
  run() {
    Probe.same({left: 1, extra: "x"}, {right: 2, other: "y"})
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
  tagged<**R>(
      _ value: { name: String, **R }
  ) -> { name: String, tag: String, **R } {
      { **value, tag: "entity" }
  }

  @class
  run() {
      let result = Probe.tagged({ name: "Phalcom", tag: "existing" })
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.tagged({ name: \"Phalcom\", tag: \"existing\" })");

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

#[test]
fn nested_row_polymorphic_call_in_generic_closure_preserves_specialized_record() {
    let fixture = Fixture::new(
        r#"
class Mapping {
  @class
  apply<T, U>(_ value: T, _ f: (T) -> U) -> U {
    f.call(value)
  }
}

class Probe {
  @class
  annotate<A, B, **R>(_ record: { value: A, **R }, _ f: (A) -> B) -> { mapped: B, value: A, **R } {
    { **record, mapped: f.call(record.value) }
  }

  @class
  run() {
    let source: { cached: Bool, name: String, value: Int } = { cached: false, name: "test", value: 42 }
    let result = Mapping.apply(source, |record| {
      Probe.annotate(record, |value| { value == 0 })
    })
  }
}
"#,
    );
    fixture.assert_no_error_diagnostics();
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let result_binding = fixture.binding(run, "result");
    let result_ty = result_binding.current.ty().expect("result type should be known");
    let TypeData::Record(row_id) = fixture.analysis.snapshot.store.get(result_ty) else {
        panic!("expected Record result, got {:?}", fixture.analysis.snapshot.store.get(result_ty));
    };
    let row = fixture.analysis.snapshot.store.record_row(*row_id);
    assert_eq!(row.tail, RecordRowTail::Closed);
    let field_names = row.fields.iter().map(|f| f.name.as_ref()).collect::<Vec<_>>();
    assert_eq!(field_names, vec!["cached", "mapped", "name", "value"]);
}

#[test]
fn record_spread_preserves_symbolic_row_and_composes_closed_specialization() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  spread<**R>(_ source: { name: String, **R }) -> { id: Int, name: String, **R } {
    { id: 7, **source }
  }

  @class
  run() {
    let result = Probe.spread({ name: "Ada", age: 42 })
  }
}
"#,
    );
    let spread = fixture.callable("Probe", "spread", DispatchSide::Class);
    let body = fixture.expression(spread, "{ id: 7, **source }");
    let body_ty = body.knowledge.ty().expect("symbolic Record spread result");
    let TypeData::Record(body_row_id) = fixture.analysis.snapshot.store.get(body_ty) else {
        panic!("expected Record body result, got {:?}", fixture.analysis.snapshot.store.get(body_ty));
    };
    let body_tail = fixture.analysis.snapshot.store.record_row(*body_row_id).tail;
    assert!(matches!(body_tail, RecordRowTail::Parameter(_)));
    let source = fixture.expression(spread, "source");
    let source_ty = source.knowledge.ty().expect("generic Record source type");
    let TypeData::Record(source_row_id) = fixture.analysis.snapshot.store.get(source_ty) else {
        panic!("expected symbolic Record source, got {:?}", fixture.analysis.snapshot.store.get(source_ty));
    };
    assert_eq!(
        body_tail,
        fixture.analysis.snapshot.store.record_row(*source_row_id).tail,
        "spread must retain the canonical row parameter identity"
    );

    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.spread({ name: \"Ada\", age: 42 })");
    let call_ty = call.knowledge.ty().expect("specialized Record spread result");
    let TypeData::Record(call_row_id) = fixture.analysis.snapshot.store.get(call_ty) else {
        panic!("expected specialized Record, got {:?}", fixture.analysis.snapshot.store.get(call_ty));
    };
    let row = fixture.analysis.snapshot.store.record_row(*call_row_id);
    assert_eq!(row.tail, RecordRowTail::Closed);
    assert_eq!(
        row.fields
            .iter()
            .map(|field| (field.name.as_ref(), fixture.analysis.snapshot.store.format_type(field.ty)))
            .collect::<Vec<_>>(),
        vec![("age", "Int".into()), ("id", "Int".into()), ("name", "String".into())]
    );
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
    fixture.assert_no_error_diagnostics();
}

#[test]
fn mixed_labeled_tuple_spread_projects_only_its_labeled_lane_to_record() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let source = (1, "ignored", name: "Ada", age: 42)
    let result = { **source }
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let result = fixture.expression(run, "{ **source }");
    let ty = result.knowledge.ty().expect("labeled Tuple lane has a RecordFields projection");
    let TypeData::Record(row_id) = fixture.analysis.snapshot.store.get(ty) else {
        panic!("expected Record projection, got {:?}", fixture.analysis.snapshot.store.get(ty));
    };
    let row = fixture.analysis.snapshot.store.record_row(*row_id);
    assert_eq!(row.tail, RecordRowTail::Closed);
    assert_eq!(
        row.fields
            .iter()
            .map(|field| (field.name.as_ref(), fixture.analysis.snapshot.store.format_type(field.ty)))
            .collect::<Vec<_>>(),
        vec![("age", "Int".into()), ("name", "String".into())]
    );
    assert!(matches!(result.status, AnalysisStatus::Ready), "{result:#?}");
    fixture.assert_no_error_diagnostics();
}

#[test]
fn record_spread_rejects_map_binding_and_literal_with_the_same_diagnostic() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let source = Map { name: "Ada" }
    let fromBinding = { **source }
    let fromLiteral = { **Map<Symbol, String> { name: "Ada" } }
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    for source in ["{ **source }", "{ **Map<Symbol, String> { name: \"Ada\" } }"] {
        let spread = fixture.expression(run, source);
        assert!(spread.knowledge.ty().is_none(), "Map spread must not publish a Record: {spread:#?}");
        assert!(matches!(spread.status, AnalysisStatus::Invalid(_)), "Map spread must be rejected: {spread:#?}");
    }
    fixture.assert_diagnostic(DiagnosticCode::RecordSpreadSourceUnsupported, 2);
    let messages = fixture
        .diagnostics(DiagnosticCode::RecordSpreadSourceUnsupported)
        .into_iter()
        .map(|diagnostic| diagnostic.message.clone())
        .collect::<Vec<_>>();
    assert!(messages.iter().all(|message| message.contains("runtime-dependent key set")));
}

#[test]
fn record_spread_keeps_duplicate_and_open_row_lacks_checks() {
    let duplicate = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let source = { name: "spread" }
    let result = { name: "literal", **source }
  }
}
"#,
    );
    let run = duplicate.callable("Probe", "run", DispatchSide::Class);
    let spread = duplicate.expression(run, "{ name: \"literal\", **source }");
    assert!(spread.knowledge.ty().is_none(), "duplicate spread field must not publish a Record: {spread:#?}");
    assert!(
        matches!(spread.status, AnalysisStatus::Invalid(_)),
        "duplicate spread field must invalidate: {spread:#?}"
    );
    duplicate.assert_diagnostic(DiagnosticCode::RecordDuplicateField, 1);

    let open_row = Fixture::new(
        r#"
class Probe {
  @class
  extend<**R>(_ source: { name: String, **R }) {
    { **source, tag: 1 }
  }

  @class
  run() {
    Probe.extend({ name: "spread" })
  }
}
"#,
    );
    let extend = open_row.callable("Probe", "extend", DispatchSide::Class);
    let spread = open_row.expression(extend, "{ **source, tag: 1 }");
    assert!(spread.knowledge.ty().is_none(), "unproven row lacks must not publish a Record: {spread:#?}");
    assert!(
        matches!(spread.status, AnalysisStatus::Invalid(_)),
        "unproven row extension must invalidate: {spread:#?}"
    );
    open_row.assert_diagnostic(DiagnosticCode::RecordRowLacksUnproven, 1);
}

#[test]
fn record_pattern_remainder_subtracts_known_fields_and_preserves_open_tail() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  closed() {
    let source = { name: "Ada", age: 42, city: "Istanbul" }
    let { name, **rest } = source
    let observed_name = name
    let observed = rest
  }

  @class
  open<**R>(_ source: { name: String, age: Int, **R }) {
    let { name, **rest } = source
    let observed = rest
  }
}
"#,
    );

    let closed = fixture.callable("Probe", "closed", DispatchSide::Class);
    let closed_name = fixture.expression(closed, "name").knowledge.ty().expect("closed field binding type");
    assert_eq!(fixture.analysis.snapshot.store.format_type(closed_name), "String");
    let closed_rest = fixture.expression(closed, "rest").knowledge.ty().expect("closed residual Record type");
    let TypeData::Record(closed_row_id) = fixture.analysis.snapshot.store.get(closed_rest) else {
        panic!("expected closed residual Record, got {:?}", fixture.analysis.snapshot.store.get(closed_rest));
    };
    let closed_row = fixture.analysis.snapshot.store.record_row(*closed_row_id);
    assert_eq!(closed_row.tail, RecordRowTail::Closed);
    assert_eq!(
        closed_row
            .fields
            .iter()
            .map(|field| (field.name.as_ref(), fixture.analysis.snapshot.store.format_type(field.ty)))
            .collect::<Vec<_>>(),
        vec![("age", "Int".into()), ("city", "String".into())]
    );

    let open = fixture.callable("Probe", "open", DispatchSide::Class);
    let source = fixture.binding(open, "source").current.ty().expect("open source Record type");
    let rest = fixture.expression(open, "rest").knowledge.ty().expect("open residual Record type");
    let TypeData::Record(source_row_id) = fixture.analysis.snapshot.store.get(source) else {
        panic!("expected open source Record, got {:?}", fixture.analysis.snapshot.store.get(source));
    };
    let TypeData::Record(rest_row_id) = fixture.analysis.snapshot.store.get(rest) else {
        panic!("expected open residual Record, got {:?}", fixture.analysis.snapshot.store.get(rest));
    };
    let source_row = fixture.analysis.snapshot.store.record_row(*source_row_id);
    let rest_row = fixture.analysis.snapshot.store.record_row(*rest_row_id);
    assert_eq!(rest_row.tail, source_row.tail, "the residual keeps the source's canonical open tail");
    assert_eq!(
        rest_row
            .fields
            .iter()
            .map(|field| (field.name.as_ref(), fixture.analysis.snapshot.store.format_type(field.ty)))
            .collect::<Vec<_>>(),
        vec![("age", "Int".into())]
    );
    fixture.assert_no_error_diagnostics();
}
