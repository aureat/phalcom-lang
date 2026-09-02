//! Incremental coverage for canonical open-record products.

use super::support::single_module_input;
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::checker::analysis::CallableAnalysisStatus;
use phalcom_semantic::db::{QueryKey, fingerprint::callable_signature_product_fingerprint};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::types::row::RecordRowTail;
use phalcom_semantic::types::store::TypeData;
use std::sync::Arc;

fn module(raw: u32) -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(raw),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").expect("valid module component")]),
    )
}

fn callable(module: &ModuleId, owner: &str, name: &str) -> CallableId {
    CallableId::new(
        DeclarationId::new(module.clone(), owner.into()),
        Selector::method(name, []).expect("valid method selector"),
        DispatchSide::Class,
    )
}

fn api_callable(module: &ModuleId) -> CallableId {
    CallableId::new(
        DeclarationId::new(module.clone(), "Api".into()),
        Selector::method("preserve", [SelectorSlot::Positional]).expect("valid preserve selector"),
        DispatchSide::Class,
    )
}

fn consumer_callable(module: &ModuleId) -> CallableId {
    callable(module, "Consumer", "read")
}

fn row_api_source(body: &str, prefix_type: &str, binder: &str) -> String {
    format!(
        r#"
class Api {{
  @class preserve<R: {binder}>(_ value: #{{ name: {prefix_type}, | R }}) -> #{{ name: {prefix_type}, | R }} {{
    {body}
  }}
}}

class Consumer {{
  @class read() -> #{{ name: String, age: Int }} {{
    Api.preserve(#{{name: "n", age: 1}})
  }}
}}
"#,
    )
}

fn diagnostic_codes(session: &SemanticWorkspaceSession, module: &ModuleId) -> Vec<DiagnosticCode> {
    let mut codes = session
        .last_snapshot()
        .and_then(|snapshot| snapshot.diagnostics.get(module).cloned())
        .map(|diagnostics| diagnostics.iter().map(|diagnostic| diagnostic.code).collect::<Vec<_>>())
        .unwrap_or_default();
    codes.sort_by_key(|code| code.as_str());
    codes
}

fn record_tail(session: &SemanticWorkspaceSession, module: &ModuleId) -> Option<RecordRowTail> {
    let snapshot = session.last_snapshot()?;
    let signature = snapshot.callable_signatures.get(&api_callable(module))?;
    let ty = signature.parameters.first()?.declared_type.canonical_type()?;
    let TypeData::Record(row_id) = snapshot.store.get(ty) else {
        return None;
    };
    Some(snapshot.store.record_row(*row_id).tail)
}

#[test]
fn row_body_only_edit_preserves_signature_semantics() {
    let module = module(301);
    let mut session = SemanticWorkspaceSession::new();
    let source_a = row_api_source("value", "String", "RecordRow");
    let update_a = session.update(single_module_input(module.clone(), &source_a, 1));
    assert!(!update_a.snapshot.has_errors(), "initial row source must be valid");
    let consumer_a = update_a
        .snapshot
        .callable_analyses
        .get(&consumer_callable(&module))
        .cloned()
        .expect("initial dependent analysis");
    let signature_a = update_a
        .snapshot
        .callable_signatures
        .get(&api_callable(&module))
        .expect("initial row signature");

    let source_b = row_api_source("value ", "String", "RecordRow");
    let update_b = session.update(single_module_input(module.clone(), &source_b, 2));
    assert!(!update_b.snapshot.has_errors(), "body-only row edit must remain valid");
    assert_eq!(update_b.stats.callables_recomputed, 1);
    assert_eq!(update_b.stats.callables_reused, 1);
    assert!(Arc::ptr_eq(
        &consumer_a,
        update_b
            .snapshot
            .callable_analyses
            .get(&consumer_callable(&module))
            .expect("reused dependent analysis")
    ));
    assert_eq!(
        record_tail(&session, &module),
        Some(RecordRowTail::Parameter(record_tail_parameter(&session, &module)))
    );
    let signature_b = update_b
        .snapshot
        .callable_signatures
        .get(&api_callable(&module))
        .expect("updated row signature");
    assert_eq!(
        callable_signature_product_fingerprint(signature_a),
        callable_signature_product_fingerprint(signature_b),
        "body-only edit must preserve canonical row signature product"
    );
    let signature_state = session
        .db()
        .query_state(&QueryKey::CallableSignature(api_callable(&module)))
        .expect("signature state");
    assert_eq!(signature_state.validated_revision(), Some(update_b.snapshot.id.revision()));
}

fn record_tail_parameter(session: &SemanticWorkspaceSession, module: &ModuleId) -> phalcom_semantic::types::id::TypeParameterId {
    let snapshot = session.last_snapshot().expect("latest snapshot");
    let signature = snapshot.callable_signatures.get(&api_callable(module)).expect("row API signature");
    let ty = signature
        .parameters
        .first()
        .and_then(|parameter| parameter.declared_type.canonical_type())
        .expect("record parameter");
    let TypeData::Record(row_id) = snapshot.store.get(ty) else {
        panic!("row API parameter must be a Record")
    };
    let row = snapshot.store.record_row(*row_id);
    let RecordRowTail::Parameter(parameter) = row.tail else {
        panic!("row API parameter must retain its stable row tail")
    };
    parameter
}

#[test]
fn row_prefix_edit_invalidates_dependent_call() {
    let module = module(302);
    let mut session = SemanticWorkspaceSession::new();
    let source_a = row_api_source("value", "String", "RecordRow");
    let update_a = session.update(single_module_input(module.clone(), &source_a, 1));
    assert!(!update_a.snapshot.has_errors());
    let consumer_a = update_a
        .snapshot
        .callable_analyses
        .get(&consumer_callable(&module))
        .cloned()
        .expect("initial dependent analysis");

    let source_c = row_api_source("value", "Int", "RecordRow");
    let update_c = session.update(single_module_input(module.clone(), &source_c, 2));
    assert!(update_c.stats.callables_recomputed >= 2, "signature edit must recheck API and dependent caller");
    let consumer_c = update_c
        .snapshot
        .callable_analyses
        .get(&consumer_callable(&module))
        .cloned()
        .expect("updated dependent analysis");
    assert!(
        !Arc::ptr_eq(&consumer_a, &consumer_c),
        "known row prefix edit must invalidate dependent analysis"
    );
    let codes = diagnostic_codes(&session, &module);
    assert!(
        codes.contains(&DiagnosticCode::ArgumentMismatch)
            || codes.contains(&DiagnosticCode::GenericInferenceConflict)
            || codes.contains(&DiagnosticCode::RecordRowInferenceConflict),
        "prefix edit must leave an argument or row conflict diagnostic: {codes:?}"
    );
}

#[test]
fn row_tail_kind_edit_invalidates_open_record_signature() {
    let module = module(303);
    let mut session = SemanticWorkspaceSession::new();
    let source_a = row_api_source("value", "String", "RecordRow");
    let update_a = session.update(single_module_input(module.clone(), &source_a, 1));
    assert!(!update_a.snapshot.has_errors());
    let old_tail = record_tail(&session, &module);
    let api_a = update_a
        .snapshot
        .callable_analyses
        .get(&api_callable(&module))
        .cloned()
        .expect("initial analysis");

    let source_d = row_api_source("value", "String", "Type");
    let update_d = session.update(single_module_input(module.clone(), &source_d, 2));
    assert!(diagnostic_codes(&session, &module).contains(&DiagnosticCode::RecordRowTailKindMismatch));
    let api_d = update_d
        .snapshot
        .callable_analyses
        .get(&api_callable(&module))
        .cloned()
        .expect("invalid API analysis remains published");
    assert!(!Arc::ptr_eq(&api_a, &api_d), "wrong-kind row edit must not reuse stale callable analysis");
    assert_ne!(record_tail(&session, &module), old_tail);
}

#[test]
fn cold_and_incremental_row_semantics_match() {
    let module = module(304);
    let source_a = row_api_source("value", "String", "RecordRow");
    let source_c = row_api_source("value", "Int", "RecordRow");

    let mut incremental = SemanticWorkspaceSession::new();
    let _ = incremental.update(single_module_input(module.clone(), &source_a, 1));
    let incremental_result = incremental.update(single_module_input(module.clone(), &source_c, 2));
    let incremental_codes = diagnostic_codes(&incremental, &module);
    let incremental_status = incremental_result
        .snapshot
        .callable_analyses
        .get(&consumer_callable(&module))
        .map(|analysis| analysis.status);

    let mut cold = SemanticWorkspaceSession::new();
    let cold_result = cold.update(single_module_input(module.clone(), &source_c, 1));
    let cold_codes = diagnostic_codes(&cold, &module);
    let cold_status = cold_result
        .snapshot
        .callable_analyses
        .get(&consumer_callable(&module))
        .map(|analysis| analysis.status);

    assert_eq!(incremental_codes, cold_codes, "incremental and cold diagnostics must agree");
    assert_eq!(incremental_status, cold_status, "incremental and cold callable status must agree");
}

#[test]
fn retained_snapshot_preserves_previous_open_row_denotation() {
    let module = module(305);
    let mut session = SemanticWorkspaceSession::new();
    let source_a = row_api_source("value", "String", "RecordRow");
    let update_a = session.update(single_module_input(module.clone(), &source_a, 1));
    assert_eq!(
        record_tail(&session, &module),
        Some(RecordRowTail::Parameter(record_tail_parameter(&session, &module)))
    );
    let old_snapshot = update_a.snapshot.clone();

    let source_c = row_api_source("value", "Int", "RecordRow");
    let _ = session.update(single_module_input(module.clone(), &source_c, 2));

    let old_signature = old_snapshot.callable_signatures.get(&api_callable(&module)).expect("old row signature");
    let old_ty = old_signature
        .parameters
        .first()
        .and_then(|parameter| parameter.declared_type.canonical_type())
        .expect("old row parameter");
    let TypeData::Record(old_row_id) = old_snapshot.store.get(old_ty) else {
        panic!("old parameter must remain a Record")
    };
    assert!(matches!(old_snapshot.store.record_row(*old_row_id).tail, RecordRowTail::Parameter(_)));
    assert_eq!(old_snapshot.store.record_row(*old_row_id).fields[0].name.as_ref(), "name");
}

#[test]
fn solver_row_variables_never_appear_in_published_products() {
    let module = module(306);
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), &row_api_source("value", "String", "RecordRow"), 1));
    let debug = format!("{:?}", update.snapshot);
    assert!(!debug.contains("RecordRowVarId"), "solver-local row variables must not enter snapshots");
    assert!(!debug.contains("RecordRowSolver"), "solver sessions must not enter snapshots");
    assert!(
        update
            .snapshot
            .callable_analyses
            .values()
            .all(|analysis| !matches!(analysis.status, CallableAnalysisStatus::InternalFailure(_)))
    );
}
