use std::sync::Arc;

use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::analyze_single_module;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

#[test]
fn match_arm_bindings_are_scoped_and_distinct_per_arm() {
    let source = r#"
enum Either<L, R> {
    @variant Left(_ val: L) -> Either<L, R>
    @variant Right(_ val: R) -> Either<L, R>
}

class Test {
    process(_ e: Either<Int, String>) {
        match e {
            Either::Left(x) => x
            Either::Right(x) => 0
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module, Arc::from(source), Arc::new(parsed));

    assert!(analysis.snapshot.diagnostics.values().all(|d| d.is_empty()), "no scope leakage between match arms");
}

#[test]
fn nested_pattern_bindings_receive_exact_component_types() {
    let source = r#"
enum Tree<T> {
    @variant Leaf(_ value: T) -> Tree<T>
    @variant Node(_ left: Tree<T>, right: Tree<T>) -> Tree<T>
}

class Test {
    depth(_ t: Tree<Int>) {
        match t {
            Tree::Leaf(v) => v
            Tree::Node(Tree::Leaf(l), right: Tree::Leaf(r)) => l
            Tree::Node* => 0
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module, Arc::from(source), Arc::new(parsed));
    assert!(analysis.snapshot.diagnostics.values().all(|d| d.is_empty()), "nested pattern bindings succeed without error");
}
