//! LANG007.C1 incremental and cold-parity tests for type-level structural operations.

use super::support::single_module_input;
use phalcom_common::selector::Selector;
use phalcom_modules::ModuleId;
use phalcom_modules::identity::{ModuleComponent, ModulePath, ResolvedProjectId};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;

fn module() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(807),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("type_members").unwrap()]),
    )
}

fn getter_callable(module: &ModuleId, owner: &str, name: &str) -> CallableId {
    CallableId::new(
        DeclarationId::new(module.clone(), owner.into()),
        Selector::getter(name).expect("valid getter selector"),
        DispatchSide::Instance,
    )
}

#[test]
fn test_incremental_tuple_at_edit_matches_cold_session() {
    let module = module();
    let source_a = r#"
class Worker {
  first_element -> (Int, String)[0] {
    42
  }
}
"#;
    let source_b = r#"
class Worker {
  first_element -> (Int, String)[1] {
    "hello"
  }
}
"#;

    // 1. Initial incremental run
    let mut incremental = SemanticWorkspaceSession::new();
    let res_a = incremental.update(single_module_input(module.clone(), source_a, 1));
    assert!(!res_a.snapshot.has_errors(), "diagnostics: {:?}", res_a.snapshot.diagnostics);

    let callable = getter_callable(&module, "Worker", "first_element");
    let sig_a = res_a.snapshot.callable_signatures.get(&callable).expect("Worker.first_element signature A");
    let ret_a = sig_a.declared_return.canonical_type().expect("ret A");
    assert_eq!(res_a.snapshot.store.format_type(ret_a), "Int");

    // 2. Incremental update
    let res_b_inc = incremental.update(single_module_input(module.clone(), source_b, 2));
    assert!(!res_b_inc.snapshot.has_errors(), "diagnostics B inc: {:?}", res_b_inc.snapshot.diagnostics);
    let sig_b_inc = res_b_inc
        .snapshot
        .callable_signatures
        .get(&callable)
        .expect("Worker.first_element signature B inc");
    let ret_b_inc = sig_b_inc.declared_return.canonical_type().expect("ret B inc");
    assert_eq!(res_b_inc.snapshot.store.format_type(ret_b_inc), "String");

    // 3. Cold run on source_b
    let mut cold = SemanticWorkspaceSession::new();
    let res_b_cold = cold.update(single_module_input(module.clone(), source_b, 1));
    assert!(!res_b_cold.snapshot.has_errors(), "diagnostics B cold: {:?}", res_b_cold.snapshot.diagnostics);
    let sig_b_cold = res_b_cold
        .snapshot
        .callable_signatures
        .get(&callable)
        .expect("Worker.first_element signature B cold");
    let ret_b_cold = sig_b_cold.declared_return.canonical_type().expect("ret B cold");

    // 4. Parity assertion: semantic type format and diagnostics match
    assert_eq!(
        res_b_inc.snapshot.store.format_type(ret_b_inc),
        res_b_cold.snapshot.store.format_type(ret_b_cold),
        "Incremental and cold return types must have identical formatted types"
    );
    assert_eq!(
        res_b_inc.snapshot.has_errors(),
        res_b_cold.snapshot.has_errors(),
        "Diagnostic error presence must match"
    );
}

#[test]
fn test_incremental_tuple_map_edit_matches_cold_session() {
    let module = module();
    let source_a = r#"
class Transformer {
  mapped_tuple -> (Int, name: String)::Map<List> {
    ([1, 2], name: ["a", "b"])
  }
}
"#;
    let source_b = r#"
class Transformer {
  mapped_tuple -> (Int, name: String)::Map<Option> {
    (Some(1), name: Some("a"))
  }
}
"#;

    // 1. Initial incremental run
    let mut incremental = SemanticWorkspaceSession::new();
    let res_a = incremental.update(single_module_input(module.clone(), source_a, 1));
    assert!(!res_a.snapshot.has_errors(), "diagnostics A: {:?}", res_a.snapshot.diagnostics);

    // 2. Incremental update
    let res_b_inc = incremental.update(single_module_input(module.clone(), source_b, 2));
    assert!(!res_b_inc.snapshot.has_errors(), "diagnostics B inc: {:?}", res_b_inc.snapshot.diagnostics);

    // 3. Cold run
    let mut cold = SemanticWorkspaceSession::new();
    let res_b_cold = cold.update(single_module_input(module.clone(), source_b, 1));
    assert!(!res_b_cold.snapshot.has_errors(), "diagnostics B cold: {:?}", res_b_cold.snapshot.diagnostics);

    let callable = getter_callable(&module, "Transformer", "mapped_tuple");
    let sig_inc = res_b_inc.snapshot.callable_signatures.get(&callable).expect("sig inc");
    let sig_cold = res_b_cold.snapshot.callable_signatures.get(&callable).expect("sig cold");

    let ret_inc = sig_inc.declared_return.canonical_type().expect("ret inc");
    let ret_cold = sig_cold.declared_return.canonical_type().expect("ret cold");

    assert_eq!(
        res_b_inc.snapshot.store.format_type(ret_inc),
        res_b_cold.snapshot.store.format_type(ret_cold),
        "Incremental and cold mapped return types must match"
    );
    assert_eq!(res_b_inc.snapshot.store.format_type(ret_inc), "(Option<Int>, name: Option<String>)");
}
