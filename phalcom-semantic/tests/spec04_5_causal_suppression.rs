use phalcom_common::range::SourceRange;
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::checker::context::CheckingContext;
use phalcom_semantic::checker::expected::ExpectedType;
use phalcom_semantic::checker::expression::analyze_expression;
use phalcom_semantic::declarations::DeclarationTypeTable;
use phalcom_semantic::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use phalcom_semantic::identity::ModuleId;
use phalcom_semantic::types::annotation::SimpleTypeResolver;
use phalcom_semantic::types::relation::MapTypeHierarchy;

use phalcom_ast::ast::Expr;
use phalcom_semantic::types::store::TypeStore;

#[test]
fn test_causal_suppression_and_marking() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let resolver = SimpleTypeResolver::new();
    let decls = DeclarationTypeTable::new();
    let module = ModuleId::core();

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &decls, module);

    let range = SourceRange { start: 10, end: 20 };
    let expr = Expr::Var {
        value: "unresolved_var".into(),
        range,
    };

    // Push an error diagnostic for this expression range
    ctx.diagnostics.push(SemanticDiagnostic::error_in(
        ctx.current_module.clone(),
        DiagnosticCode::TypeMismatch,
        "root error",
        range,
    ));

    let typed = analyze_expression(&mut ctx, &expr, &ExpectedType::None);
    assert!(typed.knowledge.is_unknown());

    // Verify expression has Invalid status with a cause ID
    let expr_analysis = ctx.expressions.values().last().unwrap();
    assert!(expr_analysis.status.is_invalid());
    if let AnalysisStatus::Invalid(cause) = expr_analysis.status {
        assert_eq!(ctx.suppression_cause(expr_analysis.id), Some(cause));
    } else {
        panic!("expected Invalid status");
    }
}
