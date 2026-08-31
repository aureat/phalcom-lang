use std::sync::Arc;

use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::diagnostic::{DiagnosticCode, DiagnosticSeverity};

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

fn diagnostics_for(source: &str) -> Vec<phalcom_semantic::diagnostic::SemanticDiagnostic> {
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("source should parse cleanly");
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed));
    analysis.snapshot.diagnostics.get(&module).cloned().unwrap_or_default().to_vec()
}

#[test]
fn generic_gadt_match_keeps_cases_reachable_until_case_equalities_are_introduced() {
    let source = r#"
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}

class Eval {
    eval<T>(_ e: Expr<T>) -> T {
        match e {
            Expr::Int(x) => x
            Expr::Bool(x) => x
        }
    }
}
"#;

    let diagnostics = diagnostics_for(source);
    assert!(
        diagnostics.is_empty(),
        "generic GADT cases must be reachable and establish branch equalities: {diagnostics:#?}"
    );
}

#[test]
fn or_pattern_requires_the_same_binding_names() {
    let source = r#"
enum Either {
    @variant Left(_ value: Int) -> Either
    @variant Right(_ value: String) -> Either
}

class Test {
    inspect(_ value: Either) {
        match value {
            Either::Left(x) | Either::Right(y) => 1
        }
    }
}
"#;

    let diagnostics = diagnostics_for(source);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.as_str() == "match.pattern.or_binding_mismatch"),
        "or-patterns with different binding sets must be rejected: {diagnostics:#?}"
    );
}

#[test]
fn duplicate_binding_in_one_pattern_has_match_specific_diagnostic() {
    let source = r#"
enum Pair {
    @variant Both(_ left: Int, right: Int) -> Pair
}

class Test {
    inspect(_ value: Pair) {
        match value {
            Pair::Both(x, right: x) => x
        }
    }
}
"#;

    let diagnostics = diagnostics_for(source);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.as_str() == "match.pattern.duplicate_binding"),
        "duplicate pattern bindings need a match-specific diagnostic: {diagnostics:#?}"
    );
}

#[test]
fn redundant_match_arm_is_a_compile_error() {
    let source = r#"
enum Option<T> {
    @variant Some(_ value: T) -> Option<T>
    @variant None -> Option<T>
}

class Test {
    inspect(_ value: Option<Int>) {
        match value {
            _ => 1
            Option::None => 2
        }
    }
}
"#;

    let diagnostics = diagnostics_for(source);
    let redundant = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == DiagnosticCode::MatchArmRedundant)
        .expect("redundant arm diagnostic");
    assert_eq!(
        redundant.severity,
        DiagnosticSeverity::Error,
        "redundant patterns are compile errors, not advisory warnings"
    );
}
