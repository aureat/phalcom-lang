use super::super::support::analyze_adt;
use phalcom_semantic::match_semantics::PatternUsefulness;

#[test]
fn recursive_binary_adt_outer_patterns_terminate_and_are_exhaustive() {
    let case = analyze_adt(
        r#"
enum Tree<T> {
    @variant Leaf(_ value: T) -> Tree<T>
    @variant Node(_ left: Tree<T>, _ right: Tree<T>) -> Tree<T>
}

class Eval {
    eval<T>(_ tree: Tree<T>) {
        match tree {
            Tree::Leaf(x) => 0
            Tree::Node(l, r) => 1
        }
    }
}
"#,
    );
    let handle = case.only_match();
    handle.assert_exhaustive();
    handle.arm(0).assert_usefulness(PatternUsefulness::Useful);
    handle.arm(1).assert_usefulness(PatternUsefulness::Useful);
}

#[test]
fn indexed_recursive_apply_outer_match() {
    let case = analyze_adt(
        r#"
enum Expr<T> {
    @variant IntLit(_ value: Int) -> Expr<Int>
    @variant Apply<A, B>(
        _ function: Expr<(A) -> B>,
        _ argument: Expr<A>
    ) -> Expr<B>
}

class Eval {
    eval<T>(_ expr: Expr<T>) {
        match expr {
            Expr::IntLit(x) => 0
            Expr::Apply(f, a) => 1
        }
    }
}
"#,
    );
    let handle = case.only_match();
    handle.assert_exhaustive();
    handle.arm(0).assert_usefulness(PatternUsefulness::Useful);
    handle.arm(1).assert_usefulness(PatternUsefulness::Useful);
}

#[test]
fn explicit_nested_recursive_pattern_requires_only_source_depth() {
    let case = analyze_adt(
        r#"
enum Tree<T> {
    @variant Leaf(_ value: T) -> Tree<T>
    @variant Node(_ left: Tree<T>, _ right: Tree<T>) -> Tree<T>
}

class Eval {
    eval<T>(_ tree: Tree<T>) {
        match tree {
            Tree::Leaf(x) => 0
            Tree::Node(Tree::Leaf(l), r) => 1
            Tree::Node(Tree::Node(nl, nr), r) => 2
        }
    }
}
"#,
    );
    let handle = case.only_match();
    handle.assert_exhaustive();
    handle.arm(0).assert_usefulness(PatternUsefulness::Useful);
    handle.arm(1).assert_usefulness(PatternUsefulness::Useful);
    handle.arm(2).assert_usefulness(PatternUsefulness::Useful);
}

#[test]
fn recursive_uninhabited_family_is_vacuously_exhaustive() {
    let case = analyze_adt(
        r#"
enum Loop {
    @variant Next(_ next: Loop) -> Loop
}

class Check {
    run(_ value: Loop) {
        match value { }
    }
}
"#,
    );
    case.only_match().assert_exhaustive();
}

#[test]
fn recursive_family_with_base_constructor_is_inhabited() {
    let case = analyze_adt(
        r#"
enum Tree {
    @variant Leaf -> Tree
    @variant Next(_ next: Tree) -> Tree
}

class Check {
    run(_ value: Tree) {
        match value { }
    }
}
"#,
    );
    case.only_match().assert_not_exhaustive();
}
