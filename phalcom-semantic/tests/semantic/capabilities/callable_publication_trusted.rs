//! Formal fixed-return regressions for canonical native callables.

use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::advisory::{AdvisoryAgreement, AdvisoryConfidence, AdvisoryFact, ValueShape, compare_expression};
use phalcom_semantic::checker::analysis::CallableAnalysis;
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::{EvidenceOrigin, EvidenceStatus, TypeKnowledge};
use phalcom_semantic::workspace::analyze_single_module;
use std::sync::Arc;

fn module_id() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").expect("module component")]),
    )
}

fn demo_run(analyses: &std::collections::HashMap<phalcom_semantic::CallableId, Arc<CallableAnalysis>>) -> &CallableAnalysis {
    let selector = Selector::method("run", []).expect("run selector");
    analyses
        .iter()
        .find(|(callable, _)| callable.owner.name.as_ref() == "Demo" && callable.selector == selector)
        .map(|(_, analysis)| analysis.as_ref())
        .expect("Demo.run analysis")
}

#[test]
fn trusted_native_fixed_returns_are_table_driven() {
    struct Case {
        source: &'static str,
        expression: &'static str,
        expected_type: &'static str,
    }

    let cases = [
        Case {
            source: r#"
class Demo {
  run() {
    System.print("hello")
  }
}
"#,
            expression: "System.print(\"hello\")",
            expected_type: "Unit",
        },
        Case {
            source: r#"
class Demo {
  run() {
    System.gc
  }
}
"#,
            expression: "System.gc",
            expected_type: "Unit",
        },
    ];

    for case in cases {
        let module = module_id();
        let source: Arc<str> = Arc::from(case.source);
        let program = Arc::new(phalcom_ast::parse(&source, 0).program);
        let snapshot = analyze_single_module(module, source.clone(), program).snapshot;
        let analysis = demo_run(&snapshot.callable_analyses);
        let call = analysis
            .expressions
            .values()
            .find(|expression| source.get(expression.range.start..expression.range.end).map(str::trim) == Some(case.expression))
            .unwrap_or_else(|| panic!("{} call expression", case.expression));

        assert!(matches!(call.knowledge, TypeKnowledge::Known(ref evidence)
            if snapshot.store.format_type(evidence.ty()) == case.expected_type
                && evidence.status() == EvidenceStatus::Established
                && evidence.origin() == EvidenceOrigin::NativeSignature));

        assert_eq!(analysis.exits.normal_return_values.len(), 1);
        assert!(matches!(&analysis.exits.normal_return_values[0], TypeKnowledge::Known(evidence)
            if snapshot.store.format_type(evidence.ty()) == case.expected_type
                && evidence.status() == EvidenceStatus::Established));
    }
}

#[test]
fn advisory_shape_cannot_replace_trusted_native_fixed_return() {
    let module = module_id();
    let source: Arc<str> = Arc::from(
        r#"
class Demo {
  run() {
    System.print("hello")
  }
}
"#,
    );
    let program = Arc::new(phalcom_ast::parse(&source, 0).program);
    let snapshot = analyze_single_module(module, source.clone(), program).snapshot;
    let analysis = demo_run(&snapshot.callable_analyses);
    let call = analysis
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end).map(str::trim) == Some("System.print(\"hello\")"))
        .expect("System.print call expression");

    let advisory = AdvisoryFact::new(
        ValueShape::Instance(DeclarationId::new(ModuleId::core(), "Int".into())),
        AdvisoryConfidence::Exact,
    );
    assert_eq!(compare_expression(&snapshot.store, call, &advisory), AdvisoryAgreement::Incomparable);
    assert!(matches!(call.knowledge, TypeKnowledge::Known(ref evidence)
        if snapshot.store.format_type(evidence.ty()) == "Unit"
            && evidence.status() == EvidenceStatus::Established
            && evidence.origin() == EvidenceOrigin::NativeSignature));
}
