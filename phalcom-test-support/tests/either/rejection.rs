use super::support::{Fixture, with_either};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::{TypeKnowledge, UnknownReason};

#[test]
fn conflicting_constructor_context_is_rejected_without_dynamic_escape() {
    let (f, probe_start) = invalid_fixture(include_str!("invalid/conflicting_constructor_context.ph"));
    assert_has_probe_generic_or_binding_conflict(&f, probe_start);
    assert_probe_call_invalid_not_dynamic(&f, "Either::Left(42)");
}

#[test]
fn map_result_contradiction_is_rejected() {
    let (f, probe_start) = invalid_fixture(include_str!("invalid/map_wrong_result.ph"));
    assert_has_probe_generic_or_binding_conflict(&f, probe_start);
    assert_probe_call_invalid_not_dynamic(&f, "source.map");
}

#[test]
fn map_left_result_contradiction_is_rejected() {
    let (f, probe_start) = invalid_fixture(include_str!("invalid/map_left_wrong_result.ph"));
    assert_has_probe_generic_or_binding_conflict(&f, probe_start);
    assert_probe_call_invalid_not_dynamic(&f, "source.mapLeft");
}

#[test]
fn repeated_generic_variable_conflict_is_rejected() {
    let (f, probe_start) = invalid_fixture(include_str!("invalid/repeated_variable_conflict.ph"));
    assert_has_probe_generic_or_binding_conflict(&f, probe_start);
    assert_probe_call_invalid_not_dynamic(&f, "Probe.merge(source)");
}

#[test]
fn flat_map_cannot_change_preserved_left_parameter() {
    let (f, probe_start) = invalid_fixture(include_str!("invalid/flat_map_wrong_left.ph"));
    assert_has_probe_generic_or_binding_conflict(&f, probe_start);
    assert_probe_call_invalid_not_dynamic(&f, "source.flatMap");
}

#[test]
fn nested_repeated_variable_conflict_is_rejected() {
    let (f, probe_start) = invalid_fixture(include_str!("invalid/nested_conflict.ph"));
    assert_has_probe_generic_or_binding_conflict(&f, probe_start);
    assert_probe_call_invalid_not_dynamic(&f, "Probe.flatten(outer)");
}

/// GEN-09/10/37: a missing family parameter is underconstrained, not Dynamic and not contradictory.
#[test]
fn uncontextualized_left_constructor_is_underconstrained_not_dynamic() {
    let (f, probe_start) = invalid_fixture(include_str!("invalid/underconstrained_left.ph"));
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let binding = f.binding(run, "value");
    assert!(
        matches!(binding.current, TypeKnowledge::Unknown(UnknownReason::UnderconstrainedTypeVariable)),
        "{binding:#?}"
    );
    assert!(
        f.diagnostics(DiagnosticCode::GenericInferenceUnderconstrained)
            .into_iter()
            .any(|diagnostic| diagnostic.primary_range.start >= probe_start),
        "underconstraint diagnostic must belong to the probe"
    );
}

fn invalid_fixture(extra: &str) -> (Fixture, usize) {
    let source = with_either(extra);
    let probe_start = source.len() - extra.len();
    (Fixture::new(&source), probe_start)
}

fn assert_probe_call_invalid_not_dynamic(f: &Fixture, needle: &str) {
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression_containing(run, needle);
    assert!(matches!(call.status, AnalysisStatus::Invalid(_)), "{call:#?}");
    assert!(
        !matches!(call.knowledge, TypeKnowledge::Dynamic(_)),
        "conflict must not escape through Dynamic: {call:#?}"
    );
}

fn assert_has_probe_generic_or_binding_conflict(f: &Fixture, probe_start: usize) {
    let accepted = [
        DiagnosticCode::GenericInferenceConflict,
        DiagnosticCode::BindingInitializerMismatch,
        DiagnosticCode::GenericConstraintUnsatisfied,
    ];
    let diagnostics = f
        .analysis
        .snapshot
        .all_diagnostics()
        .filter(|diagnostic| accepted.contains(&diagnostic.code) && diagnostic.primary_range.start >= probe_start)
        .collect::<Vec<_>>();
    assert!(
        !diagnostics.is_empty(),
        "expected probe-local generic/binding conflict, got diagnostics: {:#?}",
        f.analysis.snapshot.all_diagnostics().collect::<Vec<_>>()
    );
}
