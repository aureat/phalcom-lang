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
fn test_preexisting_diagnostic_does_not_claim_expression_ownership() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let resolver = SimpleTypeResolver::new();
    let decls = DeclarationTypeTable::new();
    let module = ModuleId::universe_root();

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &decls, module);

    let range = SourceRange { start: 10, end: 20 };
    let expr = Expr::Var {
        value: "unresolved_var".into(),
        range,
    };

    // A diagnostic emitted before this expression is not owned by it. Range
    // coincidence must not turn an otherwise analyzable expression invalid.
    ctx.emit_diagnostic(SemanticDiagnostic::error_in(
        ctx.current_module.clone(),
        DiagnosticCode::TypeMismatch,
        "root error",
        range,
    ));

    let typed = analyze_expression(&mut ctx, &expr, &ExpectedType::None);
    assert!(typed.knowledge.is_unknown());

    // Verify expression status is independent of pre-existing diagnostics.
    let expr_analysis = ctx.expressions.values().last().unwrap();
    assert_eq!(expr_analysis.status, AnalysisStatus::Ready);
    assert!(ctx.suppression_cause(expr_analysis.id).is_none());
}
