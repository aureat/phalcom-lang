//! Construction products commit only after their contextual owners accept.

use crate::semantic::support::Fixture;
use phalcom_semantic::DispatchSide;
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::types::evidence::EvidenceStatus;

#[test]
fn late_generic_failure_discards_dependent_sequence_and_preserves_independent_sibling() {
    let f = Fixture::new(
        r#"
class Consumer {
  @class reject<T>(_ values: Vector<T, 1>) -> Int where T == String { 0 }
  @class run() {
    let independent = <Int, 1>[9]
    let rejected = Consumer.reject(<Int, 1>[1])
  }
}
"#,
    );
    let run = f.callable("Consumer", "run", DispatchSide::Class);
    let call = f.expression(run, "Consumer.reject(<Int, 1>[1])");
    assert!(matches!(call.status, AnalysisStatus::Invalid(_)), "{call:#?}");
    assert_eq!(call.knowledge.ty(), Some(f.ty("Int")));
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Established));
    let rejected = f.expression(run, "<Int, 1>[1]");
    let independent = f.expression(run, "<Int, 1>[9]");
    assert!(!run.sequence_literals.contains_key(&rejected.id), "rejected application retained executable argument facts");
    assert!(run.sequence_literals.contains_key(&independent.id), "independent sibling was discarded");
}

#[test]
fn successful_generic_and_partial_annotations_commit_sequences() {
    let f = Fixture::new(
        r#"
class Consumer {
  @class accept<T>(_ values: Vector<T, 1>) -> Int where T == Int { 0 }
  @class run() {
    let accepted = Consumer.accept(<Int, 1>[1])
    let canonical: Vector<Int, 1> = [2]
    let partial: Vector<_, _> = [3]
    let explicit = <_, 1>[4]
  }
}

"#,
    );
    f.assert_no_error_diagnostics();
    let run = f.callable("Consumer", "run", DispatchSide::Class);
    assert_eq!(run.sequence_literals.len(), 4);
    for source in ["<Int, 1>[1]", "[2]", "[3]", "<_, 1>[4]"] {
        assert!(run.sequence_literals.contains_key(&f.expression(run, source).id), "{source}");
    }
}


#[test]
fn late_failure_applies_to_list_and_array_families() {
    for (family, literal) in [("List", "[1]"), ("Array", "<Int>[1]")] {
        let source = format!(r#"
class Consumer {{
  @class reject<T>(_ values: {family}<T>) -> Int where T == String {{ 0 }}
  @class run() {{ let rejected = Consumer.reject({literal}) }}
}}
"#);
        let f = Fixture::new(&source);
        let run = f.callable("Consumer", "run", DispatchSide::Class);
        assert!(matches!(f.expression(run, &format!("Consumer.reject({literal})")).status, AnalysisStatus::Invalid(_)));
        assert!(run.sequence_literals.is_empty(), "{family} retained rejected construction");
    }
}

#[test]
fn canonical_generic_parameter_expectation_preserves_application_gate() {
    let f = Fixture::new(r#"
class Consumer<T> {
  reject<U>(_ values: Vector<T, 1>, _ marker: U) -> Int where U == String { 0 }
  @class run(_ consumer: Consumer<Int>) {
    let rejected = consumer.reject(<Int, 1>[1], 42)
  }
}
"#);
    let run = f.callable("Consumer", "run", DispatchSide::Class);
    assert!(matches!(f.expression(run, "consumer.reject(<Int, 1>[1], 42)").status, AnalysisStatus::Invalid(_)));
    assert!(run.sequence_literals.is_empty());
}

#[test]
fn locally_solved_child_waits_for_parent_acceptance() {
    for second in ["<Int, 1>[2]", "<String, 1>[\"bad\"]"] {
        let valid = second.starts_with("<Int");
        let source = format!(r#"
class Consumer {{
  @class identity<T>(_ values: Vector<T, 1>) -> Vector<T, 1> {{ values }}
  @class both<T>(_ first: Vector<T, 1>, _ second: Vector<T, 1>) -> Int {{ 0 }}
  @class run() {{
    let result = Consumer.both(Consumer.identity(<Int, 1>[1]), {second})
  }}
}}
"#);
        let f = Fixture::new(&source);
        let run = f.callable("Consumer", "run", DispatchSide::Class);
        let child = f.expression(run, "Consumer.identity(<Int, 1>[1])");
        assert!(child.status.is_ready() && child.knowledge.ty().is_some(), "{child:#?}");
        let literal = f.expression(run, "<Int, 1>[1]");
        assert_eq!(run.sequence_literals.contains_key(&literal.id), valid);
        if valid { f.assert_no_error_diagnostics(); }
    }
}

#[test]
fn rejected_context_preserves_independent_declaration_inside_argument_body() {
    let f = Fixture::new(r#"
class Consumer {
  @class apply<T>(_ f: (Int) -> Vector<T, 1>, _ second: Vector<T, 1>) -> Int { 0 }
  @class run() {
    let result = Consumer.apply(|value| {
      let independent = <Int, 1>[9]
      <Int, 1>[1]
    }, <String, 1>["bad"])
  }
}
"#);
    let run = f.callable("Consumer", "run", DispatchSide::Class);
    assert!(f.analysis.snapshot.has_errors());
    assert!(run.sequence_literals.contains_key(&f.expression(run, "<Int, 1>[9]").id));
    assert!(!run.sequence_literals.contains_key(&f.expression(run, "<Int, 1>[1]").id));
}
#[test]
fn deferred_child_commits_only_after_parent_supplies_result_evidence() {
    let f = Fixture::new(r#"
class Consumer {
  @class choose<T, U>(_ values: Vector<T, 1>) -> U { 0 }
  @class use<U>(_ value: U, _ witness: U) -> U { value }
  @class run() {
    let result = Consumer.use(Consumer.choose(<Int, 1>[1]), 42)
  }
}
"#);
    f.assert_no_error_diagnostics();
    let run = f.callable("Consumer", "run", DispatchSide::Class);
    assert_eq!(f.binding(run, "result").current.ty(), Some(f.ty("Int")));
    assert!(run.sequence_literals.contains_key(&f.expression(run, "<Int, 1>[1]").id));
}

#[test]
fn failed_parent_discards_deferred_child_products() {
    let f = Fixture::new(r#"
class Consumer {
  @class choose<T, U>(_ values: Vector<T, 1>) -> U { 0 }
  @class use<U>(_ value: U, _ witness: U) -> U { value }
  @class run() {
    let independent = <Int, 1>[9]
    let result = Consumer.use(Consumer.choose(<Int, 1>[1]), missing)
  }
}
"#);
    let run = f.callable("Consumer", "run", DispatchSide::Class);
    assert!(matches!(f.expression(run, "Consumer.use(Consumer.choose(<Int, 1>[1]), missing)").status, AnalysisStatus::Blocked(_)));
    assert!(!run.sequence_literals.contains_key(&f.expression(run, "<Int, 1>[1]").id));
    assert!(run.sequence_literals.contains_key(&f.expression(run, "<Int, 1>[9]").id));
}

#[test]
fn row_generic_application_commits_only_after_row_and_type_checks() {
    for witness in ["1", "\"bad\""] {
        let valid = witness == "1";
        let source = format!(r#"
class Consumer {{
  @class use<T, **R>(_ values: Vector<T, 1>, _ record: {{ value: T, **R }}) -> {{ value: T, **R }} {{ record }}
  @class run() {{
    let result = Consumer.use(<Int, 1>[1], {{ value: {witness}, label: "kept" }})
  }}
}}
"#);
        let f = Fixture::new(&source);
        let run = f.callable("Consumer", "run", DispatchSide::Class);
        assert_eq!(run.sequence_literals.contains_key(&f.expression(run, "<Int, 1>[1]").id), valid);
        if valid { f.assert_no_error_diagnostics(); }
        else { assert!(f.analysis.snapshot.has_errors()); }
    }
}

#[test]
fn static_partial_annotation_extracts_committed_sequence_after_solving() {
    let f = Fixture::new("const values: List<_> = [1]\n");
    f.assert_no_error_diagnostics();
    let constant = f.analysis.snapshot.static_constants.iter().find(|(id, _)| id.name() == "values").expect("checked constant").1;
    assert_eq!(constant.checked.expression.sequence_literals.len(), 1);
    let resolution = constant.checked.expression.sequence_literals.values().next().unwrap();
    assert_eq!(constant.checked.expression.result_type, Some(resolution.result_type));
}

#[test]
fn static_partial_vector_annotation_extracts_committed_sequence_after_solving() {
    let f = Fixture::new("const values: Vector<_, _> = [1]\n");
    f.assert_no_error_diagnostics();
    let constant = f.analysis.snapshot.static_constants.iter().find(|(id, _)| id.name() == "values").expect("checked constant").1;
    assert_eq!(constant.checked.expression.sequence_literals.len(), 1);
    let resolution = constant.checked.expression.sequence_literals.values().next().unwrap();
    assert_eq!(resolution.family, phalcom_semantic::checker::SequenceLiteralFamily::Vector);
    assert_eq!(constant.checked.expression.result_type, Some(resolution.result_type));
}

#[test]
fn static_incompatible_vector_annotation_publishes_no_sequence() {
    let f = Fixture::new_allowing_internal_incidents("const values: Vector<Int, 2> = [1]\n");
    assert!(f.analysis.snapshot.has_errors());
    let constant = f.analysis.snapshot.static_constants.iter().find(|(id, _)| id.name() == "values").expect("checked constant").1;
    assert!(constant.checked.expression.sequence_literals.is_empty());
}

#[test]
fn static_empty_partial_vector_annotation_stays_unresolved() {
    let f = Fixture::new_allowing_internal_incidents("const values: Vector<_, _> = []\n");
    assert!(f.analysis.snapshot.has_errors());
    let constant = f.analysis.snapshot.static_constants.iter().find(|(id, _)| id.name() == "values").expect("checked constant").1;
    assert!(constant.checked.expression.sequence_literals.is_empty());
}

#[test]
fn static_partial_annotation_diagnostics_survive_initializer_checking() {
    let f = Fixture::new_allowing_internal_incidents("const values: Vector<Missing, _> = [1]\n");
    let constant = f.analysis.snapshot.static_constants.iter().find(|(id, _)| id.name() == "values").expect("checked constant").1;
    assert!(constant.checked.expression.diagnostics.iter().any(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::AnnotationUnresolved));
    assert!(constant.checked.expression.sequence_literals.is_empty());
}
