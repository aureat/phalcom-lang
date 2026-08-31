use std::sync::Arc;

use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::diagnostic::DiagnosticCode;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

#[test]
fn exhaustive_match_proves_full_coverage() {
    let source = r#"
enum Option<T> {
    @variant Some(_ value: T) -> Option<T>
    @variant None -> Option<T>
}

class Test {
    check(_ opt: Option<Int>) {
        match opt {
            Option::Some(x) => x
            Option::None => 0
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module, Arc::from(source), Arc::new(parsed));

    assert!(analysis.snapshot.diagnostics.values().all(|d| d.is_empty()), "all diagnostics empty");
}

#[test]
fn non_exhaustive_match_reports_missing_cases() {
    let source = r#"
enum Option<T> {
    @variant Some(_ value: T) -> Option<T>
    @variant None -> Option<T>
}

class Test {
    check(_ opt: Option<Int>) {
        match opt {
            Option::Some(x) => x
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed));

    let diags = analysis.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(diags.iter().any(|d| d.code == DiagnosticCode::MatchNonExhaustive), "reports MatchNonExhaustive");
}

#[test]
fn unreachable_match_arm_reports_warning() {
    let source = r#"
enum Option<T> {
    @variant Some(_ value: T) -> Option<T>
    @variant None -> Option<T>
}

class Test {
    check(_ opt: Option<Int>) {
        match opt {
            _ => 1
            Option::None => 2
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed));

    let diags = analysis.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(diags.iter().any(|d| d.code == DiagnosticCode::MatchArmRedundant), "reports MatchArmRedundant warning");
}
