use crate::semantic::support::Fixture;
use phalcom_semantic::checker::SequenceLiteralFamily;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::static_constant::StaticConstantId;
use phalcom_semantic::static_value::StaticValueData;

#[test]
fn spread_constraints_solve_partial_sequence_types_and_reject_contradictions() {
    let accepted = Fixture::new(
        r#"
const source: Vector<Int, 2> = <Int, 2>[1, 2]
const copied: Vector<_, _> = [*source]
"#,
    );
    accepted.assert_no_error_diagnostics();
    let copied_id = StaticConstantId::new(accepted.module.clone(), "copied");
    let copied = accepted.analysis.snapshot.static_constant(&copied_id).expect("copied static constant");
    let copied_value = copied.value().expect("solved spread value");
    assert!(matches!(
        accepted.analysis.snapshot.static_values().get(copied_value),
        Ok(StaticValueData::Sequence(_))
    ));
    let resolution = copied.checked.sequence_literals.values().next().expect("committed spread resolution");
    assert_eq!(resolution.family, SequenceLiteralFamily::Vector);
    assert_eq!(accepted.analysis.snapshot.store.format_type(resolution.result_type), "Vector<Int, 2#const>");

    let rejected = Fixture::new(
        r#"
const source: Vector<Int, 2> = <Int, 2>[1, 2]
const mismatch: Vector<String, 2> = [*source]
"#,
    );
    assert!(rejected.analysis.snapshot.has_errors(), "incompatible spread type must be rejected");
    assert!(rejected.diagnostics(DiagnosticCode::BindingInitializerMismatch).len() > 0);
    let mismatch_id = StaticConstantId::new(rejected.module.clone(), "mismatch");
    let mismatch = rejected.analysis.snapshot.static_constant(&mismatch_id).expect("failed constant product");
    assert!(mismatch.value().is_none(), "a rejected spread has no static value");
    assert!(mismatch.materialization_descriptor(rejected.analysis.snapshot.static_values()).is_none());
    assert!(mismatch.checked.sequence_literals.is_empty(), "a rejected spread has no committed construction");
}

#[test]
fn spread_constraint_owner_rejection_discards_only_dependent_sequence() {
    let fixture = Fixture::new(
        r#"
class Consumer {
  @class reject<T>(_ values: Vector<T, 1>) -> Int where T == String { 0 }
  @class run() {
    let source = <Int, 1>[1]
    let independent = <Int, 1>[9]
    let rejected = Consumer.reject([*source])
  }
}
"#,
    );
    let run = fixture.callable("Consumer", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Consumer.reject([*source])");
    assert!(
        matches!(call.status, phalcom_semantic::checker::analysis::AnalysisStatus::Invalid(_)),
        "{call:#?}"
    );
    assert!(!run.sequence_literals.contains_key(&fixture.expression(run, "[*source]").id));
    assert!(
        run.sequence_literals.contains_key(&fixture.expression(run, "<Int, 1>[9]").id),
        "independent owner stays publishable"
    );
}
