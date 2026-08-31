use std::sync::Arc;

use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::analyze_single_module;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

#[test]
fn resolve_exact_and_labeled_variant_patterns() {
    let source = r#"
enum Result<T, E> {
    @variant Ok(_ value: T) -> Result<T, E>
    @variant Error(_ code: Int, reason: E) -> Result<T, E>
}

class Test {
    check(res: Result<Int, String>) {
        match res {
            Result::Ok(x) => x
            Result::Error(_, reason: msg) => 0
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module, Arc::from(source), Arc::new(parsed));

    assert!(analysis.snapshot.diagnostics.values().all(|d| d.is_empty()), "no semantic errors");
}

#[test]
fn resolve_family_and_gap_patterns() {
    let source = r#"
enum Animal {
    @variant Dog(_ name: String) -> Animal
    @variant Dog(_ name: String, age: Int) -> Animal
    @variant Dog(_ name: String, age: Int, breed: String) -> Animal
    @variant Cat -> Animal
}

class Test {
    check(a: Animal) {
        match a {
            Animal::Dog* => 1
            Animal::Dog(name, ..., breed: b) => 2
            Animal::Cat => 3
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module, Arc::from(source), Arc::new(parsed));

    assert!(analysis.snapshot.diagnostics.values().all(|d| d.is_empty()), "no semantic errors");
}
