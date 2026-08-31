use std::sync::Arc;

use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::analyze_single_module;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

#[test]
fn match_matrix_simple_and_generic_enums() {
    let source = r#"
enum Status {
    @variant Active -> Status
    @variant Inactive -> Status
    @variant Pending -> Status
}

enum Result<T, E> {
    @variant Ok(_ value: T) -> Result<T, E>
    @variant Err(_ error: E) -> Result<T, E>
}

class Test {
    statusWeight(_ s: Status) {
        match s {
            Status::Active => 1
            Status::Inactive => 2
            Status::Pending => 3
        }
    }

    unwrapOr(_ res: Result<Int, String>, _ defaultVal: Int) {
        match res {
            Result::Ok(val) => val
            Result::Err(err) => defaultVal
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module, Arc::from(source), Arc::new(parsed));

    assert!(analysis.snapshot.diagnostics.values().all(|d| d.is_empty()), "matrix simple and generic enums pass without diagnostics");
}

#[test]
fn match_matrix_multi_type_parameter_gadts() {
    let source = r#"
enum Equal<A, B> {
    @variant Refl -> Equal<A, A>
}

enum Expr<T> {
    @variant LitInt(_ v: Int) -> Expr<Int>
    @variant LitBool(_ b: Bool) -> Expr<Bool>
    @variant Add(_ left: Expr<Int>, right: Expr<Int>) -> Expr<Int>
}

class Test {
    evalInt(_ e: Expr<Int>) {
        match e {
            Expr::LitInt(v) => v
            Expr::Add(left, right: r) => 42
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module, Arc::from(source), Arc::new(parsed));

    assert!(analysis.snapshot.diagnostics.values().all(|d| d.is_empty()), "multi-parameter GADTs omit refuted cases correctly");
}

#[test]
fn match_matrix_tuples_nested_or_and_wildcard_catch_alls() {
    let source = r#"
enum Color {
    @variant Red -> Color
    @variant Green -> Color
    @variant Blue -> Color
}

class Test {
    blend(_ pair: (Color, Color)) {
        match pair {
            (Color::Red, Color::Red) => 1
            (Color::Green | Color::Blue, Color::Red) => 2
            (Color::Red, Color::Green | Color::Blue) => 3
            _ => 4
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module, Arc::from(source), Arc::new(parsed));

    assert!(analysis.snapshot.diagnostics.values().all(|d| d.is_empty()), "tuples, nested or-patterns, and wildcards pass cleanly");
}

#[test]
fn match_matrix_selector_patterns_with_gaps_and_labels() {
    let source = r#"
enum Command {
    @variant Move(_ x: Int, y: Int, speed: Int) -> Command
    @variant Stop -> Command
}

class Test {
    inspect(_ cmd: Command) {
        match cmd {
            Command::Stop => 0
            Command::Move(x, ..., speed: s) => x
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module, Arc::from(source), Arc::new(parsed));

    assert!(analysis.snapshot.diagnostics.values().all(|d| d.is_empty()), "selector patterns with gaps and labels pass cleanly");
}
