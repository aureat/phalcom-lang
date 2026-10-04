use crate::semantic::incremental::support::single_module_input;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::db::{CancellationToken, QueryBudget, QueryOutcome};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::static_constant::StaticConstantId;
use phalcom_semantic::static_eval::StaticEvalOutcome;
use phalcom_semantic::static_value::{StaticInt, StaticValueData};
use std::sync::Arc;

fn test_module() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    )
}

#[test]
fn test_literal_const() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), "const A: Int = 42\n", 1));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let id = StaticConstantId::new(module.clone(), "A");
    let product = update.snapshot.static_constant(&id).expect("static constant A");
    assert!(product.is_successful());
    let val_ref = product.value().expect("value");
    let val = update.snapshot.static_values().get(val_ref).expect("value data");
    assert_eq!(val, &StaticValueData::Int(StaticInt::Small(42)));
}

#[test]
fn test_computed_const() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), "const A: Int = 10 + 20\n", 1));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let id = StaticConstantId::new(module.clone(), "A");
    let product = update.snapshot.static_constant(&id).expect("static constant A");
    assert!(product.is_successful());
    let val_ref = product.value().expect("value");
    let val = update.snapshot.static_values().get(val_ref).expect("value data");
    assert_eq!(val, &StaticValueData::Int(StaticInt::Small(30)));
}

#[test]
fn test_const_to_const_dependency() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();
    let source = "const A: Int = 10\nconst B: Int = A + 5\n";
    let update = session.update(single_module_input(module.clone(), source, 1));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let id_a = StaticConstantId::new(module.clone(), "A");
    let id_b = StaticConstantId::new(module.clone(), "B");

    let prod_a = update.snapshot.static_constant(&id_a).expect("static constant A");
    assert!(prod_a.is_successful());

    let prod_b = update.snapshot.static_constant(&id_b).expect("static constant B");
    assert!(prod_b.is_successful());
    assert!(
        prod_b.checked.static_constant_reads.contains(id_a.symbol()),
        "B should record dependency on A"
    );

    let val_b = update.snapshot.static_values().get(prod_b.value().unwrap()).unwrap();
    assert_eq!(val_b, &StaticValueData::Int(StaticInt::Small(15)));
}

#[test]
fn test_key_discriminator_same_module_isolation() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();

    let src1 = "const A: Int = 10\nconst B: Int = 20\n";
    let update1 = session.update(single_module_input(module.clone(), src1, 1));
    assert!(!update1.snapshot.has_errors());

    let id_a = StaticConstantId::new(module.clone(), "A");
    let id_b = StaticConstantId::new(module.clone(), "B");

    let prod_a1 = update1.snapshot.static_constant(&id_a).cloned().expect("A rev 1");
    let prod_b1 = update1.snapshot.static_constant(&id_b).cloned().expect("B rev 1");

    // Rev 2: Edit A to 15, leave B = 20 unchanged.
    let src2 = "const A: Int = 15\nconst B: Int = 20\n";
    let update2 = session.update(single_module_input(module.clone(), src2, 2));
    assert!(!update2.snapshot.has_errors());

    let prod_a2 = update2.snapshot.static_constant(&id_a).cloned().expect("A rev 2");
    let prod_b2 = update2.snapshot.static_constant(&id_b).cloned().expect("B rev 2");

    // A must recompute to 15
    assert!(!Arc::ptr_eq(&prod_a1, &prod_a2));
    let val_a2 = update2.snapshot.static_values().get(prod_a2.value().unwrap()).unwrap();
    assert_eq!(val_a2, &StaticValueData::Int(StaticInt::Small(15)));

    // B must be a cache hit (Arc::ptr_eq preserved)
    assert!(
        Arc::ptr_eq(&prod_b1, &prod_b2),
        "B should be reused from cache without recomputation"
    );
    let val_b2 = update2.snapshot.static_values().get(prod_b2.value().unwrap()).unwrap();
    assert_eq!(val_b2, &StaticValueData::Int(StaticInt::Small(20)));
}

#[test]
fn test_transitive_invalidation_same_type() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();

    let src1 = "const A: Int = 10\nconst B: Int = A + 1\n";
    let update1 = session.update(single_module_input(module.clone(), src1, 1));
    assert!(!update1.snapshot.has_errors());

    let _id_a = StaticConstantId::new(module.clone(), "A");
    let id_b = StaticConstantId::new(module.clone(), "B");

    let prod_b1 = update1.snapshot.static_constant(&id_b).cloned().expect("B rev 1");
    let val_b1 = update1.snapshot.static_values().get(prod_b1.value().unwrap()).unwrap();
    assert_eq!(val_b1, &StaticValueData::Int(StaticInt::Small(11)));

    // Rev 2: Edit A to 20. A's type is still Int, but its value changed.
    let src2 = "const A: Int = 20\nconst B: Int = A + 1\n";
    let update2 = session.update(single_module_input(module.clone(), src2, 2));
    assert!(!update2.snapshot.has_errors());

    let prod_b2 = update2.snapshot.static_constant(&id_b).cloned().expect("B rev 2");
    assert!(!Arc::ptr_eq(&prod_b1, &prod_b2), "B must be invalidated when A changes value");
    let val_b2 = update2.snapshot.static_values().get(prod_b2.value().unwrap()).unwrap();
    assert_eq!(val_b2, &StaticValueData::Int(StaticInt::Small(21)));
}

#[test]
fn test_runtime_dependency_rejection() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();

    let source = "let runtime_x = 10\nconst C = runtime_x\n";
    let update = session.update(single_module_input(module.clone(), source, 1));

    let diags = update.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::TypeOnlyRuntimeValue),
        "expected TypeOnlyRuntimeValue diagnostic, got: {:?}",
        diags
    );

    let id_c = StaticConstantId::new(module.clone(), "C");
    if let Some(prod_c) = update.snapshot.static_constant(&id_c) {
        assert!(!prod_c.is_successful(), "C must not reduce successfully with runtime dependency");
    }
}

#[test]
fn test_self_cycle() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();

    let source = "const A: Int = A\n";
    let update = session.update(single_module_input(module.clone(), source, 1));

    let diags = update.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::StaticConstantCycle),
        "expected StaticConstantCycle diagnostic, got: {:?}",
        diags
    );

    let id_a = StaticConstantId::new(module.clone(), "A");
    let prod_a = update.snapshot.static_constant(&id_a).expect("constant product");
    assert_eq!(prod_a.outcome, StaticEvalOutcome::Recursive);
    assert!(!prod_a.is_successful());
}

#[test]
fn test_mutual_cycle() {
    let module = test_module();
    let mut session = SemanticWorkspaceSession::new();

    let source = "const A: Int = B\nconst B: Int = A\n";
    let update = session.update(single_module_input(module.clone(), source, 1));

    let diags = update.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::StaticConstantCycle),
        "expected StaticConstantCycle diagnostic, got: {:?}",
        diags
    );

    let id_a = StaticConstantId::new(module.clone(), "A");
    let id_b = StaticConstantId::new(module.clone(), "B");

    let prod_a = update.snapshot.static_constant(&id_a).expect("constant product A");
    let prod_b = update.snapshot.static_constant(&id_b).expect("constant product B");

    assert_eq!(prod_a.outcome, StaticEvalOutcome::Recursive);
    assert_eq!(prod_b.outcome, StaticEvalOutcome::Recursive);
}

#[test]
fn test_evaluation_controls_and_lkg() {
    let module = test_module();

    // 1. Cancelled token
    {
        let mut session = SemanticWorkspaceSession::new();
        let cancel = CancellationToken::new();
        cancel.cancel();
        let res = session.update_with_budget_and_cancel(
            single_module_input(module.clone(), "const A: Int = 42\n", 1),
            QueryBudget::default(),
            &cancel,
        );
        assert!(matches!(res, Err(QueryOutcome::Cancelled)));
    }

    // 2. Zero budget
    {
        let mut session = SemanticWorkspaceSession::new();
        let cancel = CancellationToken::new();
        let zero_budget = QueryBudget {
            max_steps: 0,
            ..QueryBudget::default()
        };
        let res = session.update_with_budget_and_cancel(
            single_module_input(module.clone(), "const A: Int = 42\n", 1),
            zero_budget,
            &cancel,
        );
        assert!(matches!(res, Err(QueryOutcome::BudgetExceeded(_))));
    }

    // 3. LKG persistence
    {
        let mut session = SemanticWorkspaceSession::new();

        // Rev 1: Valid constant
        let update1 = session.update(single_module_input(module.clone(), "const A: Int = 42\n", 1));
        assert!(!update1.snapshot.has_errors());
        let id_a = StaticConstantId::new(module.clone(), "A");
        let prod_a1 = update1.snapshot.static_constant(&id_a).expect("A rev 1");
        assert!(prod_a1.is_successful());
        let val1 = prod_a1.value().unwrap();
        assert_eq!(prod_a1.lkg_value, Some(val1));

        // Rev 2: Introduce cycle (eval fails)
        let update2 = session.update(single_module_input(module.clone(), "const A: Int = A\n", 2));
        let prod_a2 = update2.snapshot.static_constant(&id_a).expect("A rev 2");
        assert!(!prod_a2.is_successful());
        assert_eq!(prod_a2.value, None);
        assert_eq!(
            prod_a2.lkg_value,
            Some(val1),
            "LKG value must persist across failed revision"
        );
    }
}
