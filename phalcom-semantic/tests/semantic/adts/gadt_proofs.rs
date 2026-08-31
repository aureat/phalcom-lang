use std::sync::Arc;

use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::analyze_single_module;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

#[test]
fn gadt_branch_proof_refines_type_parameter_and_omits_refuted_cases() {
    let source = r#"
enum Expr<T> {
    @variant LitInt(_ value: Int) -> Expr<Int>
    @variant LitBool(_ value: Bool) -> Expr<Bool>
}

class Test {
    evalInt(_ e: Expr<Int>) {
        match e {
            Expr::LitInt(n) => n
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module, Arc::from(source), Arc::new(parsed));

    assert!(analysis.snapshot.diagnostics.values().all(|d| d.is_empty()), "no errors: LitBool is refuted for Expr<Int>");
}
