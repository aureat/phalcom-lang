use crate::integration::checker::setup_test_env;
use phalcom_ast::ast::Statement;
use phalcom_ast::parse_source;
use phalcom_semantic::{CheckingContext, EvidenceOrigin, TypeKnowledge, TypeResolver, synthesize_typed_expr};

#[test]
fn integer_literal_synthesizes_int() {
    let (mut store, hier, resolver, decls, module) = setup_test_env();
    let mut ctx = CheckingContext::new(&mut store, &hier, &resolver, &decls, module);

    let source = "let x = 42";
    let program = parse_source(source, 0).expect("valid parse");

    let Statement::Let(let_stmt) = &program.statements[0] else {
        panic!("Expected a let statement");
    };

    let expr = let_stmt.value.as_ref().expect("Let statement should have an initializer");

    let typed = synthesize_typed_expr(&mut ctx, expr);

    let TypeKnowledge::Known(evidence) = &typed.knowledge else {
        panic!("integer literal should have known type, got {:?}", typed.knowledge);
    };

    let int_decl = ctx
        .resolver
        .resolve_type_name(&ctx.current_module, "Int", &[])
        .expect("Int type should be resolvable");
    let int_ty = decls.form(&int_decl).expect("Int should exist in bootstraped universe declarations");

    assert_eq!(evidence.ty, int_ty);
    assert_eq!(evidence.status, phalcom_semantic::EvidenceStatus::Established);
    assert_eq!(evidence.origin, EvidenceOrigin::Syntax);
    assert_eq!(evidence.provenance.ranges.len(), 1);
    assert_eq!(evidence.provenance.ranges.as_slice(), &[expr.range()]);
    assert!(
        ctx.diagnostics.is_empty(),
        "literal synthesis should not emit diagnostics: {:?}",
        ctx.diagnostics
    );
}

#[test]
fn just_testing() {
    let (mut store, hier, resolver, decls, module) = setup_test_env();
    let mut ctx = CheckingContext::new(&mut store, &hier, &resolver, &decls, module);

    let source = "let x = \"hello\"";
    let program = parse_source(source, 0).expect("valid parse");
    let Statement::Let(let_stmt) = &program.statements[0] else {
        panic!("Expected a let statement");
    };
    let expr = &let_stmt.value.as_ref().expect("Let statement should have an initializer");

    let typed = synthesize_typed_expr(&mut ctx, expr);
    println!("Typed expression: {:?}", typed);
}
