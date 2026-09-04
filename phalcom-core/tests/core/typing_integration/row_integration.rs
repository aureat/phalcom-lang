use super::support::{
    either, nominal, record, row_integration_invalid_source, row_integration_source, Fixture,
};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::TypeKnowledge;
use phalcom_semantic::types::store::TypeData;

/// INT-ROW-01: direct Either preserves row-specialized output.
#[test]
fn direct_either_preserves_row_specialized_output() {
    let f = Fixture::new(&row_integration_source());
    f.assert_no_errors();

    let run = f.callable("RowEitherIntegrationProbe", "mapRecord", DispatchSide::Class);
    let expected_record = record([
        ("cached", nominal("Bool")),
        ("mapped", nominal("Bool")),
        ("name", nominal("String")),
        ("value", nominal("Int")),
    ]);
    let expected_either = either(nominal("String"), expected_record);
    f.assert_known_generic_binding(run, "mapped", &expected_either);

    let map_call = f.expression_containing(run, "source.map");
    f.assert_expression_call(map_call, &f.callable_id("Either", "map", DispatchSide::Instance), f.binding(run, "mapped").current.ty().unwrap());

    let annotate_call = f.expression_containing(run, "RowCalculus.annotate");
    let inner_record_ty = annotate_call.knowledge.ty().expect("inner result Record type");
    f.assert_closed_record(
        inner_record_ty,
        &[
            ("cached", f.ty("Bool")),
            ("mapped", f.ty("Bool")),
            ("name", f.ty("String")),
            ("value", f.ty("Int")),
        ],
    );
    f.assert_expression_call(annotate_call, &f.callable_id("RowCalculus", "annotate", DispatchSide::Class), inner_record_ty);

    f.assert_generic_solution(run, annotate_call, "A", f.ty("Int"));
    f.assert_generic_solution(run, annotate_call, "B", f.ty("Bool"));
    f.assert_record_row_parameter(f.callable_generic_parameter("RowCalculus", "annotate", DispatchSide::Class, 2));

    f.assert_generic_solution(run, map_call, "R2", inner_record_ty);

    let mapped_ty = f.binding(run, "mapped").current.ty().unwrap();
    let applied_args = f.assert_applied(mapped_ty, "Either", 2);
    assert_eq!(applied_args[1], inner_record_ty, "outer Either right argument TypeId must match inner result Record TypeId");
}

/// INT-ROW-02: nested ADT can be an ordinary Record field generic.
#[test]
fn nested_adt_can_be_ordinary_record_field_generic() {
    let f = Fixture::new(&row_integration_source());
    f.assert_no_errors();

    let run = f.callable("RowNestedAdtProbe", "preserveNested", DispatchSide::Class);
    let expected_record = record([
        ("cached", nominal("Bool")),
        ("label", nominal("String")),
        ("value", either(nominal("String"), nominal("Int"))),
    ]);
    f.assert_known_generic_binding(run, "result", &expected_record);

    let call = f.expression_containing(run, "RowCalculus.preserveValue");
    let result_ty = f.binding(run, "result").current.ty().unwrap();
    f.assert_expression_call(call, &f.callable_id("RowCalculus", "preserveValue", DispatchSide::Class), result_ty);

    let solved_t = f.generic_solution_type(run, call, "T");
    f.assert_type(solved_t, &either(nominal("String"), nominal("Int")));

    let payload_param = f.binding(run, "payload").current.ty().unwrap();
    assert_eq!(solved_t, payload_param);

    f.assert_record_row_parameter(f.callable_generic_parameter("RowCalculus", "preserveValue", DispatchSide::Class, 1));
}

/// INT-ROW-07: row collision inside Either map fails closed.
#[test]
fn row_collision_inside_either_map_fails_closed() {
    let f = Fixture::new(&row_integration_invalid_source());
    let run = f.callable("RowEitherInvalidProbe", "collision", DispatchSide::Class);

    let tagged_call = f.expression_containing(run, "RowCalculus.tagged");
    assert!(
        matches!(tagged_call.status, AnalysisStatus::Invalid(_) | AnalysisStatus::Blocked(_)),
        "inner tagged call must be rejected: {tagged_call:#?}"
    );
    assert!(
        !matches!(tagged_call.knowledge, TypeKnowledge::Dynamic(_)),
        "inner tagged call must not escape through Dynamic: {tagged_call:#?}"
    );
    assert_eq!(f.diagnostics(DiagnosticCode::RecordRowLacksViolation).len(), 1);

    let binding = f.binding(run, "result");
    assert!(
        !matches!(binding.current, TypeKnowledge::Known(_)),
        "outer binding must not produce a known type: {binding:#?}"
    );
    assert!(
        !matches!(binding.current, TypeKnowledge::Dynamic(_)),
        "outer binding must not escape through Dynamic: {binding:#?}"
    );
}

/// INT-ROW-03: Monad bind solves HKT F and Record A/B simultaneously.
#[test]
fn int_row_03_monad_bind_solves_hkt_and_records() {
    let f = Fixture::new(&row_integration_source());
    f.assert_no_errors();

    let run = f.callable("RowMonadIntegrationProbe", "bindRecord", DispatchSide::Class);
    let expected_record = record([
        ("cached", nominal("Bool")),
        ("mapped", nominal("Bool")),
        ("name", nominal("String")),
        ("value", nominal("Int")),
    ]);
    let expected_either = either(nominal("String"), expected_record);
    f.assert_known_generic_binding(run, "result", &expected_either);
    let result_ty = f.binding(run, "result").current.ty().unwrap();
    let bind_target = f.callable_id("MonadAlgorithms", "bind", DispatchSide::Class);
    f.assert_expression_call(bind_call, &bind_target, result_ty);

    let constructor_param = f.callable_generic_parameter("MonadAlgorithms", "bind", DispatchSide::Class, 0);
    let a_param = f.callable_generic_parameter("MonadAlgorithms", "bind", DispatchSide::Class, 1);
    let b_param = f.callable_generic_parameter("MonadAlgorithms", "bind", DispatchSide::Class, 2);

    let constructor = f.generic_solution_type_for(run, bind_call, constructor_param);
    f.assert_unary_constructor_kind(f.analysis.snapshot.store.kind_of(constructor));
    let TypeData::Lambda(lambda_id) = f.analysis.snapshot.store.get(constructor) else {
        panic!("F must remain a canonical unary type lambda");
    };
    let lambda = f.analysis.snapshot.store.arena().get_lambda(*lambda_id);
    let mut free = Vec::new();
    f.analysis.snapshot.store.arena().collect_free_types(lambda.body, &mut free);
    assert!(free.contains(&f.ty("String")), "F must capture String: {free:#?}");
    assert!(f.analysis.snapshot.store.arena().has_free_bound(lambda.body, 0), "F must retain its bound argument");

    let source_param = f.binding(run, "source").current.ty().unwrap();
    let source_applied = f.assert_applied(source_param, "Either", 2);
    let input_record = source_applied[1];
    f.assert_closed_record(
        input_record,
        &[
            ("cached", f.ty("Bool")),
            ("name", f.ty("String")),
            ("value", f.ty("Int")),
        ],
    );

    let result_applied = f.assert_applied(result_ty, "Either", 2);
    let output_record = result_applied[1];
    f.assert_closed_record(
        output_record,
        &[
            ("cached", f.ty("Bool")),
            ("mapped", f.ty("Bool")),
            ("name", f.ty("String")),
            ("value", f.ty("Int")),
        ],
    );

    f.assert_generic_solution_exact(run, bind_call, constructor_param, constructor, phalcom_semantic::types::evidence::EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, bind_call, a_param, input_record, phalcom_semantic::types::evidence::EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, bind_call, b_param, output_record, phalcom_semantic::types::evidence::EvidenceStatus::Established);
}
