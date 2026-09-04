//! Incremental invalidation for callable-local and variant-local generic contracts.

use super::support::single_module_input;
use phalcom_common::selector::{SelectorBase, SelectorKind};
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::checker::analysis::{AnalysisStatus, ExpressionAnalysis};
use phalcom_semantic::db::QueryKey;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::match_semantics::{MatchResolution, PatternResolution, PatternUsefulness};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::snapshot::SemanticSnapshot;
use std::sync::Arc;

fn module(raw: u32) -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(raw),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").expect("valid module component")]),
    )
}

fn callable_id(snapshot: &SemanticSnapshot, module: &ModuleId, owner: &str, name: &str, side: DispatchSide) -> CallableId {
    let owner = DeclarationId::new(module.clone(), owner.into());
    snapshot
        .callable_signatures
        .iter()
        .map(|(callable, _)| callable)
        .find(|callable| callable.owner == owner && callable.side == side && matches!(&callable.selector.base, SelectorBase::Named(base) if base == name))
        .cloned()
        .unwrap_or_else(|| panic!("missing callable {owner:?}.{name} on {side:?}"))
}

fn subscript_setter_id(snapshot: &SemanticSnapshot, module: &ModuleId, owner: &str, side: DispatchSide) -> CallableId {
    let owner = DeclarationId::new(module.clone(), owner.into());
    snapshot
        .callable_signatures
        .iter()
        .map(|(callable, _)| callable)
        .find(|callable| callable.owner == owner && callable.side == side && callable.selector.kind == SelectorKind::SubscriptSet)
        .cloned()
        .unwrap_or_else(|| panic!("missing subscript setter on {owner:?}"))
}

fn diagnostic_codes(snapshot: &SemanticSnapshot, module: &ModuleId) -> Vec<DiagnosticCode> {
    let mut codes = snapshot
        .diagnostics
        .get(module)
        .map(|diagnostics| diagnostics.iter().map(|diagnostic| diagnostic.code).collect::<Vec<_>>())
        .unwrap_or_default();
    codes.sort_by_key(|code| code.as_str());
    codes
}

fn expression<'a>(snapshot: &'a SemanticSnapshot, module: &ModuleId, callable: &CallableId, needle: &str) -> &'a ExpressionAnalysis {
    let source = &snapshot.sources[module].text;
    snapshot
        .callable_analyses
        .get(callable)
        .expect("callable analysis")
        .expressions
        .values()
        .find(|expression| {
            source
                .get(expression.range.start..expression.range.end)
                .is_some_and(|text| text.contains(needle))
        })
        .unwrap_or_else(|| panic!("missing expression containing {needle:?}"))
}

fn first_match<'a>(snapshot: &'a SemanticSnapshot, callable: &CallableId) -> &'a MatchResolution {
    snapshot
        .callable_analyses
        .get(callable)
        .expect("match callable")
        .match_resolutions
        .values()
        .next()
        .expect("match resolution")
}

#[test]
fn constructor_generic_bound_edit_invalidates_dependent_call_and_matches_cold() {
    let module = module(601);
    let source_without_bound = r#"
class Number {}
class Box<T> {
  @constructor new<U>(_ value: T, _ metadata: U) {}
}
class Consumer {
  @class use() {
    Box.new(1, "metadata")
  }
}
"#;
    let source_with_bound = r#"
class Number {}
class Box<T> {
  @constructor new<U>(_ value: T, _ metadata: U) where U <: Number {}
}
class Consumer {
  @class use() {
    Box.new(1, "metadata")
  }
}
"#;

    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental.update(single_module_input(module.clone(), source_without_bound, 1));
    assert!(
        !first.snapshot.has_errors(),
        "unbounded constructor call must be valid: {:?}",
        first.snapshot.diagnostics
    );

    let constructor = callable_id(&first.snapshot, &module, "Box", "new", DispatchSide::Class);
    let consumer = callable_id(&first.snapshot, &module, "Consumer", "use", DispatchSide::Class);
    let signature_key = QueryKey::CallableSignature(constructor.clone());
    let constructor_fp1 = incremental
        .db()
        .ready_product_fingerprint(&signature_key)
        .expect("constructor signature fingerprint");
    let consumer_v1 = first.snapshot.callable_analyses.get(&consumer).cloned().expect("consumer analysis v1");

    let second = incremental.update(single_module_input(module.clone(), source_with_bound, 2));
    let constructor_fp2 = incremental
        .db()
        .ready_product_fingerprint(&signature_key)
        .expect("updated constructor signature fingerprint");
    assert_ne!(
        constructor_fp1, constructor_fp2,
        "constructor local bound edit must change its product fingerprint"
    );
    assert!(second.recomputed.contains(&QueryKey::CallableBody(consumer.clone())));
    assert!(diagnostic_codes(&second.snapshot, &module).contains(&DiagnosticCode::GenericConstraintUnsatisfied));
    let consumer_v2 = second.snapshot.callable_analyses.get(&consumer).expect("consumer analysis v2");
    assert!(
        !Arc::ptr_eq(&consumer_v1, consumer_v2),
        "dependent constructor call must not reuse its body analysis"
    );
    let call_v2 = expression(&second.snapshot, &module, &consumer, "Box.new(1, \"metadata\")");
    assert!(matches!(call_v2.status, AnalysisStatus::Invalid(_)));

    let mut cold = SemanticWorkspaceSession::new();
    let cold_update = cold.update(single_module_input(module.clone(), source_with_bound, 1));
    let cold_consumer = callable_id(&cold_update.snapshot, &module, "Consumer", "use", DispatchSide::Class);
    let cold_call = expression(&cold_update.snapshot, &module, &cold_consumer, "Box.new(1, \"metadata\")");
    assert_eq!(diagnostic_codes(&second.snapshot, &module), diagnostic_codes(&cold_update.snapshot, &module));
    assert_eq!(call_v2.callable, cold_call.callable, "cold and incremental calls must retain CallableId");
    assert_eq!(call_v2.knowledge.status(), cold_call.knowledge.status());
    assert_eq!(
        matches!(call_v2.status, AnalysisStatus::Invalid(_)),
        matches!(cold_call.status, AnalysisStatus::Invalid(_))
    );
}

#[test]
fn setter_and_index_generic_contract_edits_invalidate_dependent_calls() {
    let setter_module = module(602);
    let setter_a = r#"
class Number {}
class Box {
  value<T>=(put next: T) { }
  run() { self.value = "text" }
}
"#;
    let setter_b = r#"
class Number {}
class Box {
  value<T>=(put next: T) where T <: Number { }
  run() { self.value = "text" }
}
"#;
    let mut setter_session = SemanticWorkspaceSession::new();
    let setter_first = setter_session.update(single_module_input(setter_module.clone(), setter_a, 1));
    assert!(
        !setter_first.snapshot.has_errors(),
        "unbounded setter call must be valid: {:?}",
        setter_first.snapshot.diagnostics
    );
    let setter = callable_id(&setter_first.snapshot, &setter_module, "Box", "value", DispatchSide::Instance);
    let setter_run = callable_id(&setter_first.snapshot, &setter_module, "Box", "run", DispatchSide::Instance);
    let setter_fp1 = setter_session
        .db()
        .ready_product_fingerprint(&QueryKey::CallableSignature(setter.clone()))
        .expect("setter fingerprint");
    let setter_body_v1 = setter_first.snapshot.callable_analyses.get(&setter_run).cloned().expect("setter caller v1");
    let setter_second = setter_session.update(single_module_input(setter_module.clone(), setter_b, 2));
    let setter_fp2 = setter_session
        .db()
        .ready_product_fingerprint(&QueryKey::CallableSignature(setter.clone()))
        .expect("updated setter fingerprint");
    assert_ne!(setter_fp1, setter_fp2, "setter local bound edit must change its product fingerprint");
    assert!(!Arc::ptr_eq(
        &setter_body_v1,
        setter_second.snapshot.callable_analyses.get(&setter_run).expect("setter caller v2")
    ));
    assert!(diagnostic_codes(&setter_second.snapshot, &setter_module).contains(&DiagnosticCode::GenericConstraintUnsatisfied));

    let index_module = module(603);
    let index_a = r#"
class Number {}
class Store {
  [_ key: U]<U>=(put value: U) { }
  run(_ store: Store) { store["text"] = "text" }
}
"#;
    let index_b = r#"
class Number {}
class Store {
  [_ key: U]<U>=(put value: U) where U <: Number { }
  run(_ store: Store) { store["text"] = "text" }
}
"#;
    let mut index_session = SemanticWorkspaceSession::new();
    let index_first = index_session.update(single_module_input(index_module.clone(), index_a, 1));
    assert!(
        !index_first.snapshot.has_errors(),
        "unbounded index setter call must be valid: {:?}",
        index_first.snapshot.diagnostics
    );
    let index_setter = subscript_setter_id(&index_first.snapshot, &index_module, "Store", DispatchSide::Instance);
    let index_run = callable_id(&index_first.snapshot, &index_module, "Store", "run", DispatchSide::Instance);
    let index_fp1 = index_session
        .db()
        .ready_product_fingerprint(&QueryKey::CallableSignature(index_setter.clone()))
        .expect("index setter fingerprint");
    let index_body_v1 = index_first.snapshot.callable_analyses.get(&index_run).cloned().expect("index caller v1");
    let index_second = index_session.update(single_module_input(index_module.clone(), index_b, 2));
    let index_fp2 = index_session
        .db()
        .ready_product_fingerprint(&QueryKey::CallableSignature(index_setter))
        .expect("updated index setter fingerprint");
    assert_ne!(index_fp1, index_fp2, "index local bound edit must change its product fingerprint");
    assert!(!Arc::ptr_eq(
        &index_body_v1,
        index_second.snapshot.callable_analyses.get(&index_run).expect("index caller v2")
    ));
    assert!(diagnostic_codes(&index_second.snapshot, &index_module).contains(&DiagnosticCode::GenericConstraintUnsatisfied));
}

#[test]
fn variant_generic_payload_and_result_edits_invalidate_construction_and_match() {
    let module = module(604);
    let source_a = r#"
enum Expr<T> {
  @variant Wrap<U>(_ value: U) -> Expr<Int>
}
class Producer {
  @class build() { Expr::Wrap(1) }
}
class Evaluator {
  eval(_ value: Expr<Int>) {
    match value { Expr::Wrap(x) => 1 }
  }
}
"#;
    let source_b = r#"
enum Expr<T> {
  @variant Wrap<U>(_ value: List<U>) -> Expr<List<U>> where U <: Object
}
class Producer {
  @class build() { Expr::Wrap(1) }
}
class Evaluator {
  eval(_ value: Expr<Int>) {
    match value { Expr::Wrap(x) => 1 }
  }
}
"#;

    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental.update(single_module_input(module.clone(), source_a, 1));
    assert!(
        !first.snapshot.has_errors(),
        "initial variant construction/match must be valid: {:?}",
        first.snapshot.diagnostics
    );
    let producer = callable_id(&first.snapshot, &module, "Producer", "build", DispatchSide::Class);
    let evaluator = callable_id(&first.snapshot, &module, "Evaluator", "eval", DispatchSide::Instance);
    let enum_owner = DeclarationId::new(module.clone(), "Expr".into());
    let enum_key = QueryKey::EnumDeclaration(enum_owner);
    let enum_fp1 = incremental.db().ready_product_fingerprint(&enum_key).expect("initial enum fingerprint");
    let producer_v1 = first.snapshot.callable_analyses.get(&producer).cloned().expect("producer v1");
    let evaluator_v1 = first.snapshot.callable_analyses.get(&evaluator).cloned().expect("evaluator v1");
    let match_v1 = first_match(&first.snapshot, &evaluator);
    assert_eq!(match_v1.arms[0].usefulness, PatternUsefulness::Useful);

    let second = incremental.update(single_module_input(module.clone(), source_b, 2));
    let enum_fp2 = incremental.db().ready_product_fingerprint(&enum_key).expect("updated enum fingerprint");
    assert_ne!(enum_fp1, enum_fp2, "variant payload/result/bound edit must change enum product fingerprint");
    assert!(!Arc::ptr_eq(
        &producer_v1,
        second.snapshot.callable_analyses.get(&producer).expect("producer v2")
    ));
    assert!(!Arc::ptr_eq(
        &evaluator_v1,
        second.snapshot.callable_analyses.get(&evaluator).expect("evaluator v2")
    ));
    assert!(
        !diagnostic_codes(&second.snapshot, &module).is_empty(),
        "invalid construction/match must publish diagnostics"
    );
    let match_v2 = first_match(&second.snapshot, &evaluator);
    assert_ne!(match_v1.arms[0].usefulness, match_v2.arms[0].usefulness);
    assert!(matches!(&match_v2.arms[0].pattern, PatternResolution::Variant(_)));

    let mut cold = SemanticWorkspaceSession::new();
    let cold_update = cold.update(single_module_input(module.clone(), source_b, 1));
    let cold_evaluator = callable_id(&cold_update.snapshot, &module, "Evaluator", "eval", DispatchSide::Instance);
    let cold_match = first_match(&cold_update.snapshot, &cold_evaluator);
    assert_eq!(
        match_v2.arms[0].usefulness, cold_match.arms[0].usefulness,
        "cold and incremental variant usefulness must agree"
    );
    assert_eq!(diagnostic_codes(&second.snapshot, &module), diagnostic_codes(&cold_update.snapshot, &module));
}

#[test]
fn cold_and_incremental_variant_openings_are_alpha_equivalent() {
    let module = module(605);
    let source_a = r#"
enum Expr<T> {
  @variant Wrap<U>(_ value: U) -> Expr<List<U>> where U <: Object
}
class Evaluator {
  eval<T>(_ value: Expr<T>) {
    match value { Expr::Wrap(x) => x }
  }
}
"#;
    let source_b = r#"
enum Expr<T> {
  @variant Wrap<U>(_ value: U) -> Expr<List<U>> where U <: Object
}
class Evaluator {
  keep() { 1 }
  eval<T>(_ value: Expr<T>) {
    match value { Expr::Wrap(x) => x }
  }
}
"#;

    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental.update(single_module_input(module.clone(), source_a, 1));
    let evaluator = callable_id(&first.snapshot, &module, "Evaluator", "eval", DispatchSide::Instance);
    let _ = incremental.update(single_module_input(module.clone(), source_b, 2));
    let incremental_match = first_match(incremental.last_snapshot().expect("incremental snapshot"), &evaluator);
    let PatternResolution::Variant(incremental_pattern) = &incremental_match.arms[0].pattern else {
        panic!("expected variant pattern in incremental analysis");
    };
    let incremental_case = incremental_pattern.candidates[0].case_instantiation.as_ref().expect("incremental case opening");

    let mut cold = SemanticWorkspaceSession::new();
    let cold_update = cold.update(single_module_input(module.clone(), source_b, 1));
    let cold_evaluator = callable_id(&cold_update.snapshot, &module, "Evaluator", "eval", DispatchSide::Instance);
    let cold_match = first_match(&cold_update.snapshot, &cold_evaluator);
    let PatternResolution::Variant(cold_pattern) = &cold_match.arms[0].pattern else {
        panic!("expected variant pattern in cold analysis");
    };
    let cold_case = cold_pattern.candidates[0].case_instantiation.as_ref().expect("cold case opening");

    assert!(incremental_case.result_type.alpha_equivalent(&cold_case.result_type));
    assert_eq!(incremental_case.payload_types.len(), cold_case.payload_types.len());
    for (incremental_payload, cold_payload) in incremental_case.payload_types.iter().zip(cold_case.payload_types.iter()) {
        assert!(incremental_payload.alpha_equivalent(cold_payload));
    }
}
