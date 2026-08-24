//! Wave 3 Comprehensive Tests: Bidirectional Body/Call Semantics & Expression Analysis (E1–E6).

use phalcom_ast::parse_source;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::context::CheckingContext;
use phalcom_semantic::checker::expected::ExpectedType;
use phalcom_semantic::checker::expression::{analyze_expression, check_expr};
use phalcom_semantic::checker::statement::check_statement;
use phalcom_semantic::declarations::{DeclarationTypeTable, bootstrap_universe_declarations};
use phalcom_semantic::identity::{BodyId, CallableId, DeclarationId, DispatchSide, ExpressionId, LocalExpressionId};
use phalcom_semantic::types::annotation::{SimpleTypeResolver, TypeResolver};
use phalcom_semantic::types::evidence::{EvidenceAuthority, TypeKnowledge};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;

fn setup_wave3_test_env() -> (TypeStore, MapTypeHierarchy, SimpleTypeResolver, DeclarationTypeTable, ModuleId) {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let mut resolver = SimpleTypeResolver::new();
    let module = ModuleId::core();

    let declarations = bootstrap_universe_declarations(&mut store, &|k| DeclarationId::new(module.clone(), k.name().into()));

    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let string_decl = DeclarationId::new(module.clone(), "String".into());
    let bool_decl = DeclarationId::new(module.clone(), "Bool".into());
    let obj_decl = DeclarationId::new(module.clone(), "Object".into());
    let list_decl = DeclarationId::new(module.clone(), "List".into());
    let set_decl = DeclarationId::new(module.clone(), "Set".into());
    let map_decl = DeclarationId::new(module.clone(), "Map".into());

    hierarchy.insert(int_decl.clone(), obj_decl.clone());
    hierarchy.insert(string_decl.clone(), obj_decl.clone());
    hierarchy.insert(bool_decl.clone(), obj_decl.clone());

    resolver.insert("Int", int_decl);
    resolver.insert("String", string_decl);
    resolver.insert("Bool", bool_decl);
    resolver.insert("Object", obj_decl);
    resolver.insert("List", list_decl);
    resolver.insert("Set", set_decl);
    resolver.insert("Map", map_decl);

    (store, hierarchy, resolver, declarations, module)
}

#[test]
fn test_bidirectional_empty_collections() {
    let (mut store, hier, resolver, decls, module) = setup_wave3_test_env();
    let mut ctx = CheckingContext::new(&mut store, &hier, &resolver, &decls, module);

    let int_decl = ctx.resolver.resolve_type_name(&ctx.current_module, "Int", &[]).unwrap();
    let int_ty = ctx.nominal_type_of(&int_decl);

    let list_decl = ctx.resolver.resolve_type_name(&ctx.current_module, "List", &[]).unwrap();
    let list_k = ctx.store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let list_form = ctx.store.nominal_form(list_decl, list_k);
    let expected_list_int = ctx.store.list_of(list_form, int_ty).unwrap();

    // 1. Empty list literal [] with expected List<Int>
    let program = parse_source("[]", 0).unwrap();
    let expr = match &program.statements[0] {
        phalcom_ast::ast::Statement::Expr { expr, .. } => expr,
        _ => panic!("expected expr"),
    };

    let typed = analyze_expression(&mut ctx, expr, &ExpectedType::Proper(expected_list_int));
    assert_eq!(typed.ty(), Some(expected_list_int));

    // 2. Let binding with annotation: let xs: List<Int> = []
    let let_prog = parse_source("let xs: List<Int> = []", 0).unwrap();
    check_statement(&mut ctx, &let_prog.statements[0]);

    let fact = ctx.lookup_local("xs").expect("xs should be bound");
    assert_eq!(fact.knowledge.ty(), Some(expected_list_int));
}

#[test]
fn test_expression_analysis_index_population_and_stability() {
    let (mut store, hier, resolver, decls, module) = setup_wave3_test_env();
    let mut ctx = CheckingContext::new(&mut store, &hier, &resolver, &decls, module);
    ctx.body_id = BodyId(42);

    let program = parse_source("1 + 2", 0).unwrap();
    let expr = match &program.statements[0] {
        phalcom_ast::ast::Statement::Expr { expr, .. } => expr,
        _ => panic!("expected expr"),
    };

    analyze_expression(&mut ctx, expr, &ExpectedType::None);

    // Pre-order traversal records: 1, 2, and the outer BinaryExpr (1 + 2)
    assert!(ctx.expressions.len() >= 3);
    let first_id = ExpressionId::new(BodyId(42), LocalExpressionId(0));
    assert!(ctx.expressions.contains_key(&first_id));

    // Re-analyzing with a fresh context and same BodyId produces identical index keys
    let mut ctx2 = CheckingContext::new(ctx.store, &hier, &resolver, &decls, ctx.current_module.clone());
    ctx2.body_id = BodyId(42);
    analyze_expression(&mut ctx2, expr, &ExpectedType::None);

    assert_eq!(ctx.expressions.keys().collect::<Vec<_>>(), ctx2.expressions.keys().collect::<Vec<_>>());
}

#[test]
fn test_generic_method_call_inference() {
    let (mut store, hier, resolver, decls, module) = setup_wave3_test_env();
    let mut ctx = CheckingContext::new(&mut store, &hier, &resolver, &decls, module);

    let int_decl = ctx.resolver.resolve_type_name(&ctx.current_module, "Int", &[]).unwrap();
    let int_ty = ctx.nominal_type_of(&int_decl);

    // Register a class with a generic method: id<T>(x: T) -> T
    let box_decl = DeclarationId::new(ctx.current_module.clone(), "IdBox".into());
    let mut surface = phalcom_semantic::surface::DeclarationSurface::new(Some(box_decl.clone()));

    let callable_id = CallableId::new(
        box_decl.clone(),
        phalcom_common::selector::Selector::method("id", vec![phalcom_common::selector::SelectorSlot::Positional]).unwrap(),
        DispatchSide::Instance,
    );
    let param_t = ctx.store.intern_type_parameter(phalcom_semantic::types::parameter::TypeParameterData::new(
        phalcom_semantic::types::parameter::TypeParameterOwner::Callable(callable_id.clone()),
        0,
        "T",
        KindId::TYPE,
    ));
    let t_ty = ctx.store.parameter_form(param_t);

    let generic_sig = phalcom_semantic::types::parameter::GenericSignature::new(
        phalcom_semantic::types::parameter::TypeParameterOwner::Callable(callable_id.clone()),
        Box::new([param_t]),
    );

    let param = phalcom_semantic::dispatch::CallableParameter::new("x", TypeKnowledge::known(t_ty, EvidenceAuthority::Declared));
    let callable_sig = phalcom_semantic::dispatch::CallableSignature::new(
        callable_id.selector.clone(),
        vec![param],
        TypeKnowledge::known(t_ty, EvidenceAuthority::Declared),
    )
    .with_generics(generic_sig);

    surface.add_callable(DispatchSide::Instance, callable_sig);
    ctx.register_surface(box_decl.clone(), surface);

    // Bind `b` as instance of IdBox
    let box_ty = ctx.nominal_type_of(&box_decl);
    ctx.bind_local(
        "b",
        phalcom_semantic::types::denotation::ValueSemanticFact::new(TypeKnowledge::known(box_ty, EvidenceAuthority::Declared)),
        phalcom_common::range::SourceRange::default(),
    );

    // Call `b.id(42)`
    let program = parse_source("b.id(42)", 0).unwrap();
    let expr = match &program.statements[0] {
        phalcom_ast::ast::Statement::Expr { expr, .. } => expr,
        _ => panic!("expected expr"),
    };

    let ret_k = check_expr(&mut ctx, expr, &ExpectedType::None);
    assert_eq!(ret_k.ty(), Some(int_ty), "generic id<T>(42) should infer Int");
}

#[test]
fn test_flow_state_mutation_and_if_let_join() {
    let (mut store, hier, resolver, decls, module) = setup_wave3_test_env();
    let mut ctx = CheckingContext::new(&mut store, &hier, &resolver, &decls, module);

    let int_decl = ctx.resolver.resolve_type_name(&ctx.current_module, "Int", &[]).unwrap();
    let string_decl = ctx.resolver.resolve_type_name(&ctx.current_module, "String", &[]).unwrap();
    let int_ty = ctx.nominal_type_of(&int_decl);
    let str_ty = ctx.nominal_type_of(&string_decl);

    // Bind mutable local `x` with initial Int
    let x_binding = ctx.bind_local_var(
        "x",
        None,
        TypeKnowledge::known(int_ty, EvidenceAuthority::ExactSyntax),
        true,
        None,
        phalcom_common::range::SourceRange::default(),
    );
    assert_eq!(ctx.flow.get_current_type(x_binding).and_then(|k| k.ty()), Some(int_ty));

    // Reassign x = "hello"
    ctx.assign_existing(
        "x",
        phalcom_semantic::types::denotation::ValueSemanticFact::new(TypeKnowledge::known(str_ty, EvidenceAuthority::ExactSyntax)),
    );
    assert_eq!(ctx.flow.get_current_type(x_binding).and_then(|k| k.ty()), Some(str_ty));
    assert_eq!(ctx.flow.get_binding(x_binding).unwrap().version, 1);
}
