use phalcom_ast::parse_source;
use phalcom_common::range::SourceRange;
use phalcom_modules::identity::ModuleId;
use phalcom_native_meta::universe::UniverseKey;
use phalcom_semantic::checker::context::CheckingContext;
use phalcom_semantic::checker::expression::synthesize_typed_expr;
use phalcom_semantic::checker::statement::check_statement;
use phalcom_semantic::declarations::bootstrap_universe_declarations;
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::annotation::SimpleTypeResolver;
use phalcom_semantic::types::denotation::{SemanticDenotation, ValueSemanticFact};
use phalcom_semantic::types::evidence::TypeKnowledge;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;

fn test_universe_resolver(key: UniverseKey) -> DeclarationId {
    DeclarationId::new(ModuleId::universe_root(), key.name().into())
}

#[test]
fn class_name_has_class_object_value_type_and_type_form_denotation() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut resolver = SimpleTypeResolver::new();
    let declarations = bootstrap_universe_declarations(&mut store, &test_universe_resolver);
    let module = ModuleId::universe_root();

    let int_decl = test_universe_resolver(UniverseKey::Int);
    resolver.insert("Int", int_decl.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module);

    let program = parse_source("Int", 0).unwrap();
    let expr = match &program.statements[0] {
        phalcom_ast::ast::Statement::Expr { expr, .. } => expr,
        _ => panic!("expected expr"),
    };

    let typed = synthesize_typed_expr(&mut ctx, expr);

    let int_class_obj = declarations.class_object_type(&int_decl).unwrap();
    let int_form = declarations.form(&int_decl).unwrap();

    assert_eq!(typed.ty(), Some(int_class_obj));
    assert_eq!(typed.denotation, Some(SemanticDenotation::TypeForm(int_form)));
    assert_eq!(ctx.store.kind_of(int_form), KindId::TYPE);
}

#[test]
fn generic_class_name_denotes_constructor_kind() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut resolver = SimpleTypeResolver::new();
    let declarations = bootstrap_universe_declarations(&mut store, &test_universe_resolver);
    let module = ModuleId::universe_root();

    let list_decl = test_universe_resolver(UniverseKey::List);
    resolver.insert("List", list_decl.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module);

    let program = parse_source("List", 0).unwrap();
    let expr = match &program.statements[0] {
        phalcom_ast::ast::Statement::Expr { expr, .. } => expr,
        _ => panic!("expected expr"),
    };

    let typed = synthesize_typed_expr(&mut ctx, expr);

    let list_class_obj = declarations.class_object_type(&list_decl).unwrap();
    let list_form = declarations.form(&list_decl).unwrap();

    assert_eq!(typed.ty(), Some(list_class_obj));
    assert_eq!(typed.denotation, Some(SemanticDenotation::TypeForm(list_form)));
    assert_ne!(ctx.store.kind_of(list_form), KindId::TYPE);
}

#[test]
fn ordinary_literal_has_no_denotation() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut resolver = SimpleTypeResolver::new();
    let declarations = bootstrap_universe_declarations(&mut store, &test_universe_resolver);
    let module = ModuleId::universe_root();

    let int_decl = test_universe_resolver(UniverseKey::Int);
    resolver.insert("Int", int_decl);

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module);

    let program = parse_source("42", 0).unwrap();
    let expr = match &program.statements[0] {
        phalcom_ast::ast::Statement::Expr { expr, .. } => expr,
        _ => panic!("expected expr"),
    };

    let typed = synthesize_typed_expr(&mut ctx, expr);
    assert!(typed.knowledge.is_known());
    assert_eq!(typed.denotation, None);
}

#[test]
fn const_binding_preserves_denotation() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut resolver = SimpleTypeResolver::new();
    let declarations = bootstrap_universe_declarations(&mut store, &test_universe_resolver);
    let module = ModuleId::universe_root();

    let int_decl = test_universe_resolver(UniverseKey::Int);
    resolver.insert("Int", int_decl.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module);

    let program = parse_source("let t = Int", 0).unwrap();
    check_statement(&mut ctx, &program.statements[0]);

    let fact = ctx.lookup_local("t").unwrap();
    let int_form = declarations.form(&int_decl).unwrap();
    assert_eq!(fact.denotation, Some(SemanticDenotation::TypeForm(int_form)));
}

#[test]
fn reassignment_replaces_or_clears_denotation() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut resolver = SimpleTypeResolver::new();
    let declarations = bootstrap_universe_declarations(&mut store, &test_universe_resolver);
    let module = ModuleId::universe_root();

    let int_decl = test_universe_resolver(UniverseKey::Int);
    let string_decl = test_universe_resolver(UniverseKey::String);
    resolver.insert("Int", int_decl.clone());
    resolver.insert("String", string_decl.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module);

    let p1 = parse_source("let t = Int", 0).unwrap();
    check_statement(&mut ctx, &p1.statements[0]);

    let int_form = declarations.form(&int_decl).unwrap();
    assert_eq!(ctx.lookup_local("t").unwrap().denotation, Some(SemanticDenotation::TypeForm(int_form)));

    // t = String
    let p2 = parse_source("t = String", 0).unwrap();
    let expr2 = match &p2.statements[0] {
        phalcom_ast::ast::Statement::Expr { expr, .. } => expr,
        _ => panic!("expected expr"),
    };
    synthesize_typed_expr(&mut ctx, expr2);

    let string_form = declarations.form(&string_decl).unwrap();
    assert_eq!(ctx.lookup_local("t").unwrap().denotation, Some(SemanticDenotation::TypeForm(string_form)));

    // t = 42 (clears denotation)
    let p3 = parse_source("t = 42", 0).unwrap();
    let expr3 = match &p3.statements[0] {
        phalcom_ast::ast::Statement::Expr { expr, .. } => expr,
        _ => panic!("expected expr"),
    };
    synthesize_typed_expr(&mut ctx, expr3);

    assert_eq!(ctx.lookup_local("t").unwrap().denotation, None);
}

#[test]
fn flow_join_preserves_only_identical_denotation() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut resolver = SimpleTypeResolver::new();
    let declarations = bootstrap_universe_declarations(&mut store, &test_universe_resolver);
    let module = ModuleId::universe_root();

    let int_decl = test_universe_resolver(UniverseKey::Int);
    let string_decl = test_universe_resolver(UniverseKey::String);
    resolver.insert("Int", int_decl.clone());
    resolver.insert("String", string_decl.clone());

    let _ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module);

    let int_form = declarations.form(&int_decl).unwrap();

    let fact_int1 = ValueSemanticFact::new(TypeKnowledge::established(
        declarations.class_object_type(&int_decl).unwrap(),
        phalcom_semantic::types::evidence::EvidenceOrigin::DeveloperAnnotation,
    ))
    .with_denotation(SemanticDenotation::TypeForm(int_form));

    let fact_int2 = fact_int1.clone();

    let fact_string = ValueSemanticFact::new(TypeKnowledge::established(
        declarations.class_object_type(&string_decl).unwrap(),
        phalcom_semantic::types::evidence::EvidenceOrigin::DeveloperAnnotation,
    ))
    .with_denotation(SemanticDenotation::TypeForm(declarations.form(&string_decl).unwrap()));

    // Joining identical denotation preserves it
    let merged_same = ValueSemanticFact::merge(&fact_int1, &fact_int2, fact_int1.knowledge.clone());
    assert_eq!(merged_same.denotation, Some(SemanticDenotation::TypeForm(int_form)));

    // Joining different denotations clears it
    let merged_diff = ValueSemanticFact::merge(&fact_int1, &fact_string, fact_int1.knowledge.clone());
    assert_eq!(merged_diff.denotation, None);
}

#[test]
fn type_form_expression_synthesizes_class_object_and_applied_denotation() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut resolver = SimpleTypeResolver::new();
    let declarations = bootstrap_universe_declarations(&mut store, &test_universe_resolver);
    let module = ModuleId::universe_root();

    let int_decl = test_universe_resolver(UniverseKey::Int);
    let list_decl = test_universe_resolver(UniverseKey::List);
    resolver.insert("Int", int_decl.clone());
    resolver.insert("List", list_decl.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module);

    let annotation = phalcom_ast::ast::TypeAnnotation {
        expr: phalcom_ast::ast::TypeAnnotationExpr::Application {
            origin: Box::new(phalcom_ast::ast::TypeAnnotation {
                expr: phalcom_ast::ast::TypeAnnotationExpr::Reference(phalcom_ast::ast::StaticSymbolRef {
                    root: "List".into(),
                    root_range: SourceRange { start: 0, end: 4 },
                    members: Vec::new(),
                    range: SourceRange { start: 0, end: 4 },
                }),
                range: SourceRange { start: 0, end: 4 },
            }),
            arguments: vec![phalcom_ast::ast::TypeAnnotation {
                expr: phalcom_ast::ast::TypeAnnotationExpr::Reference(phalcom_ast::ast::StaticSymbolRef {
                    root: "Int".into(),
                    root_range: SourceRange { start: 5, end: 8 },
                    members: Vec::new(),
                    range: SourceRange { start: 5, end: 8 },
                }),
                range: SourceRange { start: 5, end: 8 },
            }],
            range: SourceRange { start: 0, end: 9 },
        },
        range: SourceRange { start: 0, end: 9 },
    };
    let expr = phalcom_ast::ast::Expr::TypeForm(Box::new(annotation));

    let typed = synthesize_typed_expr(&mut ctx, &expr);

    let list_class_obj = declarations.class_object_type(&list_decl).unwrap();
    let list_form = declarations.form(&list_decl).unwrap();
    let int_form = declarations.form(&int_decl).unwrap();
    let applied_form = ctx.store.apply_type_form(list_form, &[int_form]).unwrap();

    assert_eq!(typed.ty(), Some(list_class_obj));
    assert_eq!(typed.denotation, Some(SemanticDenotation::TypeForm(applied_form)));
    assert_eq!(ctx.store.kind_of(applied_form), KindId::TYPE);
}
