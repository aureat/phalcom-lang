use std::sync::Arc;

use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::diagnostic::DiagnosticCode;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

#[test]
fn reports_match_pattern_unresolved_for_nonexistent_variant() {
    let source = r#"
enum Option<T> {
    @variant Some(_ value: T) -> Option<T>
    @variant None -> Option<T>
}

class Test {
    check(_ opt: Option<Int>) {
        match opt {
            Option::NonExistent => 1
            _ => 2
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed));

    let diags = analysis.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(diags.iter().any(|d| d.code == DiagnosticCode::MatchPatternUnresolved), "reports MatchPatternUnresolved");
}

#[test]
fn reports_match_pattern_arity_mismatch() {
    let source = r#"
enum Option<T> {
    @variant Some(_ value: T) -> Option<T>
    @variant None -> Option<T>
}

class Test {
    check(_ opt: Option<Int>) {
        match opt {
            Option::Some(a, b) => 1
            _ => 2
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed));

    let diags = analysis.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(diags.iter().any(|d| d.code == DiagnosticCode::MatchPatternArityMismatch), "reports MatchPatternArityMismatch");
}

#[test]
fn reports_match_pattern_field_mismatch_for_invalid_label() {
    let source = r#"
enum Tree<T> {
    @variant Leaf(_ value: T) -> Tree<T>
    @variant Node(_ left: Tree<T>, right: Tree<T>) -> Tree<T>
}

class Test {
    check(_ t: Tree<Int>) {
        match t {
            Tree::Leaf(v) => 1
            Tree::Node(left: l, wrongLabel: r) => 2
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed));

    let diags = analysis.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(diags.iter().any(|d| d.code == DiagnosticCode::MatchPatternFieldMismatch), "reports MatchPatternFieldMismatch");
}

#[test]
fn reports_match_pattern_contradictory_for_wrong_nominal_enum() {
    let source = r#"
enum Option<T> {
    @variant Some(_ value: T) -> Option<T>
    @variant None -> Option<T>
}

enum Tree<T> {
    @variant Leaf(_ value: T) -> Tree<T>
}

class Test {
    check(_ opt: Option<Int>) {
        match opt {
            Tree::Leaf(x) => 1
            _ => 2
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed));

    let diags = analysis.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(diags.iter().any(|d| d.code == DiagnosticCode::MatchPatternContradictory), "reports MatchPatternContradictory");
}

#[test]
fn reports_match_pattern_contradictory_for_refuted_gadt_arm() {
    let source = r#"
enum Expr<T> {
    @variant LitInt(_ value: Int) -> Expr<Int>
    @variant LitBool(_ value: Bool) -> Expr<Bool>
}

class Test {
    evalInt(_ e: Expr<Int>) {
        match e {
            Expr::LitInt(n) => n
            Expr::LitBool(b) => 0
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed));

    let diags = analysis.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(diags.iter().any(|d| d.code == DiagnosticCode::MatchPatternContradictory), "reports MatchPatternContradictory for refuted GADT variant");
}
