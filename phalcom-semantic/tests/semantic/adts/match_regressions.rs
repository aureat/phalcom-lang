use std::sync::Arc;

use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::diagnostic::{DiagnosticCode, DiagnosticSeverity};
use phalcom_semantic::match_semantics::PatternUsefulness;

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

/// LAW: ordered usefulness precedes branch typing, so an arm eliminated by an
/// earlier pattern cannot contribute value evidence to the match result.
#[test]
fn redundant_arm_does_not_contribute_to_match_result() {
    let source = r#"
enum Choice {
    @variant A -> Choice
    @variant B -> Choice
}

class Test {
    inspect(_ value: Choice) {
        match value {
            _ => 1
            Choice::A => "unreachable"
        }
    }
}
"#;

    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("source should parse cleanly");
    let analysis = analyze_single_module(module, Arc::from(source), Arc::new(parsed));
    let resolution = analysis
        .snapshot
        .callable_analyses
        .values()
        .flat_map(|callable| callable.match_resolutions.values())
        .next()
        .expect("match resolution");

    assert_eq!(resolution.arms.len(), 2);
    assert_eq!(resolution.arms[0].usefulness, PatternUsefulness::Useful);
    assert_eq!(resolution.arms[1].usefulness, PatternUsefulness::Redundant);
    assert_eq!(
        resolution.result, resolution.arms[0].branch_result,
        "a statically unreachable arm must not widen or weaken the match expression result"
    );
}

/// LAW: if every reachable arm exits abruptly, the match expression itself
/// cannot complete normally and therefore has the bottom type `Never`.
#[test]
fn all_abrupt_match_has_never_result() {
    let source = r#"
enum Choice {
    @variant A -> Choice
    @variant B -> Choice
}

class Test {
    inspect(_ value: Choice) -> Int {
        match value {
            Choice::A => { return 1 }
            Choice::B => { return 2 }
        }
    }
}
"#;

    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("source should parse cleanly");
    let analysis = analyze_single_module(module, Arc::from(source), Arc::new(parsed));
    let resolution = analysis
        .snapshot
        .callable_analyses
        .values()
        .flat_map(|callable| callable.match_resolutions.values())
        .next()
        .expect("match resolution");
    let result_ty = resolution.result.ty().expect("all-abrupt match should still establish Never");

    assert!(
        matches!(analysis.snapshot.store.get(result_ty), phalcom_semantic::types::store::TypeData::Never),
        "an exhaustive match with no normally completing arm must have Never result, got {:?}",
        analysis.snapshot.store.get(result_ty)
    );
}
