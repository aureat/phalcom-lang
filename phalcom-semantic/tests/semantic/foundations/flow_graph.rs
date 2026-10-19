use phalcom_ast::parse_source;
use phalcom_common::range::SourceRange;
use phalcom_modules::DeclarationId;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::check_program;
use phalcom_semantic::checker::flow::graph::{FlowEdgeKind, FlowGraph, FlowNodeKind};
use phalcom_semantic::checker::flow::predicate::{FlowPredicate, extract_predicate_shape};
use phalcom_semantic::checker::flow::state::FlowState;
use phalcom_semantic::checker::flow::transfer::apply_predicate;
use phalcom_semantic::declarations::{DeclarationTypeTable, bootstrap_universe_declarations};
use phalcom_semantic::identity::BindingId;
use phalcom_semantic::types::annotation::SimpleTypeResolver;
use phalcom_semantic::types::denotation::SemanticDenotation;
use phalcom_semantic::types::evidence::{DynamicReason, EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;

fn setup_test_env() -> (TypeStore, MapTypeHierarchy, SimpleTypeResolver, DeclarationTypeTable, ModuleId) {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let mut resolver = SimpleTypeResolver::new();
    let module = ModuleId::universe_root();

    let declarations = bootstrap_universe_declarations(&mut store, &|k| DeclarationId::new(module.clone(), k.name().into()));

    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let float_decl = DeclarationId::new(module.clone(), "Float".into());
    let string_decl = DeclarationId::new(module.clone(), "String".into());
    let bool_decl = DeclarationId::new(module.clone(), "Bool".into());
    let list_decl = DeclarationId::new(module.clone(), "List".into());
    let map_decl = DeclarationId::new(module.clone(), "Map".into());
    let set_decl = DeclarationId::new(module.clone(), "Set".into());
    let symbol_decl = DeclarationId::new(module.clone(), "Symbol".into());
    let obj_decl = DeclarationId::new(module.clone(), "Object".into());
    let num_decl = DeclarationId::new(module.clone(), "Number".into());

    hierarchy.insert(num_decl.clone(), obj_decl.clone());
    hierarchy.insert(int_decl.clone(), num_decl.clone());
    hierarchy.insert(float_decl.clone(), num_decl.clone());
    hierarchy.insert(string_decl.clone(), obj_decl.clone());
    hierarchy.insert(bool_decl.clone(), obj_decl.clone());
    hierarchy.insert(list_decl.clone(), obj_decl.clone());
    hierarchy.insert(map_decl.clone(), obj_decl.clone());
    hierarchy.insert(set_decl.clone(), obj_decl.clone());
    hierarchy.insert(symbol_decl.clone(), obj_decl.clone());

    resolver.insert("Int", int_decl);
    resolver.insert("Float", float_decl);
    resolver.insert("String", string_decl);
    resolver.insert("Bool", bool_decl);
    resolver.insert("List", list_decl);
    resolver.insert("Map", map_decl);
    resolver.insert("Set", set_decl);
    resolver.insert("Symbol", symbol_decl);
    resolver.insert("Object", obj_decl);
    resolver.insert("Number", num_decl);

    (store, hierarchy, resolver, declarations, module)
}

#[test]
fn test_flow_graph_construction() {
    let mut graph = FlowGraph::new();
    let r0 = SourceRange { start: 0, end: 10 };
    let n1 = graph.add_node(FlowNodeKind::Entry, r0).unwrap();
    let n2 = graph.add_node(FlowNodeKind::BranchCondition, r0).unwrap();
    let n3 = graph.add_node(FlowNodeKind::Exit, r0).unwrap();

    let e1 = graph.add_edge(n1, n2, None).unwrap();
    let e2 = graph.add_edge(n2, n3, None).unwrap();

    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.edge_count(), 2);
    assert_eq!(graph.node(&n1).unwrap().successors, vec![e1]);
    assert_eq!(graph.node(&n2).unwrap().predecessors, vec![e1]);
    assert_eq!(graph.node(&n2).unwrap().successors, vec![e2]);
}

#[test]
fn test_flow_graph_from_statements_branches_and_loops() {
    let source = r#"
class CfgSubject {
  testFlow(_ cond: Bool, _ count: Int) {
    let x = 10
    if (cond) {
      return x
    } else {
      for i in [1, 2, 3] {
        if (i == 2) {
          break
        }
      }
    }
    throw "done"
  }
}

"#;
    let program = parse_source(source, 0).expect("valid parse");
    let class_def = match &program.statements[0] {
        phalcom_ast::ast::Statement::Class(c) => c,
        _ => panic!("expected class"),
    };
    let member_body = match &class_def.members[0] {
        phalcom_ast::ast::ClassMember::Method(m) => m.body.statements().expect("statements block"),
        _ => panic!("expected method"),
    };

    let cfg = FlowGraph::from_statements(member_body);
    assert!(cfg.entry().is_some(), "CFG must have an entry node");
    assert!(!cfg.exits().is_empty(), "CFG must record exit nodes");
    assert!(cfg.node_count() > 0, "CFG must have statement/branch nodes");

    let reachable = cfg.reachable_nodes();
    assert!(reachable.contains(&cfg.entry().unwrap()));

    let has_loop = cfg.nodes().any(|n| matches!(n.kind, FlowNodeKind::LoopHeader));
    assert!(has_loop, "CFG for loop construct must contain a LoopHeader node");

    let has_back_edge = cfg.edges().any(|e| matches!(e.kind, FlowEdgeKind::BackEdge));
    assert!(has_back_edge, "CFG for loop construct must contain a BackEdge");
}

#[test]
fn test_flow_graph_for_explicit_while_keeps_condition_and_backedge() {
    let source = r#"
class CfgWhile {
  run(_ cond: Bool) {
    while cond {
      let bodyLocal = 1
    }
  }
}
"#;
    let program = parse_source(source, 0).expect("valid parse");
    let class_def = match &program.statements[0] {
        phalcom_ast::ast::Statement::Class(c) => c,
        _ => panic!("expected class"),
    };
    let member_body = match &class_def.members[0] {
        phalcom_ast::ast::ClassMember::Method(m) => m.body.statements().expect("statements block"),
        _ => panic!("expected method"),
    };
    let while_expr = match &member_body[0] {
        phalcom_ast::ast::Statement::Expr {
            expr: phalcom_ast::ast::Expr::While(while_expr),
            ..
        } => while_expr,
        other => panic!("expected explicit While expression, got {other:?}"),
    };

    let cfg = FlowGraph::from_statements(member_body);
    let header = cfg
        .nodes()
        .find(|node| matches!(node.kind, FlowNodeKind::LoopHeader))
        .expect("while loop header");
    let condition = cfg
        .nodes()
        .find(|node| node.kind == FlowNodeKind::BranchCondition && node.range == while_expr.condition.range())
        .expect("condition branch uses the source condition range");
    assert_eq!(header.range, while_expr.range);
    assert!(cfg.edges().any(|edge| edge.kind == FlowEdgeKind::BackEdge && edge.target == header.id));
    assert!(cfg.edges().any(|edge| edge.kind == FlowEdgeKind::FalseBranch && edge.source == condition.id));
}

fn parse_expr_helper(src: &str) -> phalcom_ast::ast::Expr {
    let p = parse_source(src, 0).expect("valid parse");
    match p.statements.into_iter().next().unwrap() {
        phalcom_ast::ast::Statement::Expr { expr, .. } => expr,
        other => panic!("expected expr statement, got {:?}", other),
    }
}

#[test]
fn test_predicate_extraction_from_ast() {
    let (mut store, hier, resolver, decls, module) = setup_test_env();
    let mut ctx = phalcom_semantic::checker::context::CheckingContext::new(&mut store, &hier, &resolver, &decls, module);

    let b1 = match ctx.bind_pattern_binding(
        "x",
        phalcom_semantic::types::denotation::ValueSemanticFact::new(TypeKnowledge::assumed(ctx.store.unit(), EvidenceOrigin::DeveloperAnnotation)),
        phalcom_common::range::SourceRange::default(),
    ) {
        phalcom_semantic::checker::binding::BindingDeclarationResult::Inserted(binding) => binding,
        phalcom_semantic::checker::binding::BindingDeclarationResult::Redeclared(binding) => binding,
    };

    // 1. is test
    let is_expr = parse_expr_helper("x.is(Int)");
    let pred_is = extract_predicate_shape(&mut ctx, &is_expr, true);
    assert!(matches!(pred_is, Some(FlowPredicate::IsInstance { binding, .. }) if binding == b1));

    // 2. != None test
    let not_nil_expr = parse_expr_helper("x != None");
    let pred_not_nil = extract_predicate_shape(&mut ctx, &not_nil_expr, true);
    assert_eq!(pred_not_nil, Some(FlowPredicate::NotNil { binding: b1 }));

    // 3. > 0 comparison test
    let ord_expr = parse_expr_helper("x > 0");
    let pred_ord = extract_predicate_shape(&mut ctx, &ord_expr, true);
    assert!(matches!(pred_ord, Some(FlowPredicate::OrderedPredicate { binding, ref op, threshold: 0 }) if binding == b1 && op == ">"));
}

#[test]
fn type_test_refinements_require_supported_proper_targets_and_respect_exactness() {
    let (mut store, mut hierarchy, resolver, decls, module) = setup_test_env();
    let string_decl = DeclarationId::new(module.clone(), "String".into());
    let subclass_decl = DeclarationId::new(module.clone(), "StringSubclassForTypeTest".into());
    hierarchy.insert(subclass_decl.clone(), string_decl.clone());
    let mut ctx = phalcom_semantic::checker::context::CheckingContext::new(&mut store, &hierarchy, &resolver, &decls, module.clone());
    let binding = match ctx.bind_pattern_binding(
        "x",
        phalcom_semantic::types::denotation::ValueSemanticFact::new(TypeKnowledge::Unknown(UnknownReason::UncheckedExpression)),
        phalcom_common::range::SourceRange::default(),
    ) {
        phalcom_semantic::checker::binding::BindingDeclarationResult::Inserted(binding)
        | phalcom_semantic::checker::binding::BindingDeclarationResult::Redeclared(binding) => binding,
    };

    let string_ty = ctx.nominal_type_of(&string_decl).expect("String declaration type");
    assert_eq!(ctx.store.kind_of(string_ty), KindId::TYPE);

    let is_string = parse_expr_helper("x is String");
    assert_eq!(
        extract_predicate_shape(&mut ctx, &is_string, true),
        Some(FlowPredicate::IsInstance { binding, target: string_ty }),
    );
    assert_eq!(
        extract_predicate_shape(&mut ctx, &is_string, false),
        Some(FlowPredicate::IsNotInstance { binding, target: string_ty }),
    );

    let strict_string = parse_expr_helper("x is! String");
    assert_eq!(
        extract_predicate_shape(&mut ctx, &strict_string, true),
        Some(FlowPredicate::IsInstance { binding, target: string_ty }),
        "exact-class success may provide the weaker membership fact",
    );
    assert_eq!(
        extract_predicate_shape(&mut ctx, &strict_string, false),
        None,
        "failing an exact-class test does not exclude subclasses",
    );

    let list_decl = DeclarationId::new(module, "List".into());
    let list_form = ctx.nominal_type_of(&list_decl).expect("List declaration form");
    assert_ne!(
        ctx.store.kind_of(list_form),
        KindId::TYPE,
        "bare generic List is a constructor, not a proper target"
    );
    let int_decl = DeclarationId::new(string_decl.module.clone(), "Int".into());
    let int_ty = ctx.nominal_type_of(&int_decl).expect("Int declaration type");
    let applied_list = ctx.store.apply_type_form(list_form, &[int_ty]).expect("List<Int> remains a valid applied type");
    assert_eq!(
        ctx.store.kind_of(applied_list),
        KindId::TYPE,
        "the unsupported bare-family check must not damage proper applications"
    );
    let before = ctx.flow.get_current_type(binding).cloned();
    let bare_list_positive = parse_expr_helper("x is List");
    let bare_list_negative = parse_expr_helper("x is not List");
    for expression in [&bare_list_positive, &bare_list_negative] {
        for truth in [true, false] {
            assert_eq!(
                extract_predicate_shape(&mut ctx, expression, truth),
                None,
                "unsupported generic-family classifier must not produce a static refinement on either edge"
            );
        }
    }
    assert_eq!(
        ctx.flow.get_current_type(binding).cloned(),
        before,
        "unsupported target must not publish flow knowledge"
    );

    let applied_list_knowledge = TypeKnowledge::established(applied_list, EvidenceOrigin::Syntax);
    ctx.flow.assign(binding, applied_list_knowledge.clone());
    for expression in [&bare_list_positive, &bare_list_negative] {
        for truth in [true, false] {
            assert_eq!(extract_predicate_shape(&mut ctx, expression, truth), None);
            assert_eq!(
                ctx.flow.get_current_type(binding),
                Some(&applied_list_knowledge),
                "an unsupported test must preserve a previously established List<Int> fact"
            );
        }
    }

    let applied_generic_target = parse_expr_helper("x is List<Int>");
    assert_eq!(
        extract_predicate_shape(&mut ctx, &applied_generic_target, true),
        None,
        "applied generic classifiers need checked runtime reification, not only a proper type",
    );

    let subclass_ty = ctx.store.nominal(subclass_decl);
    let subclass_knowledge = TypeKnowledge::established(subclass_ty, EvidenceOrigin::Syntax);
    ctx.flow.assign(binding, subclass_knowledge.clone());
    assert_eq!(extract_predicate_shape(&mut ctx, &strict_string, false), None);
    assert_eq!(
        ctx.flow.get_current_type(binding),
        Some(&subclass_knowledge),
        "false `is! String` must preserve a String subclass fact",
    );
}

#[test]
fn test_predicate_refinement_and_inversion() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let module = ModuleId::universe_root();
    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let str_decl = DeclarationId::new(module.clone(), "String".into());

    let int_ty = store.nominal(int_decl);
    let str_ty = store.nominal(str_decl);
    let union_ty = store.union(&[int_ty, str_ty]);

    let b1 = BindingId(1);
    let mut state = FlowState::new();
    state.declare(
        b1,
        "b1",
        SourceRange::default(),
        None,
        TypeKnowledge::established(union_ty, EvidenceOrigin::Syntax),
        true,
    );

    // 1. Filter by IsInstance { target: Int }
    let pred = FlowPredicate::IsInstance { binding: b1, target: int_ty };
    apply_predicate(&mut state, &pred.clone().authoritative(), &mut store, &hierarchy);
    assert_eq!(state.get_current_type(b1).and_then(|k| k.ty()), Some(int_ty));
    assert!(state.facts.contains(&pred));

    // 2. Test Inversion
    let inv = pred.invert().unwrap();
    assert_eq!(inv, FlowPredicate::IsNotInstance { binding: b1, target: int_ty });

    // 3. Test IsNotInstance on fresh union state
    let mut state2 = FlowState::new();
    state2.declare(
        b1,
        "b1",
        SourceRange::default(),
        None,
        TypeKnowledge::established(union_ty, EvidenceOrigin::Syntax),
        true,
    );
    apply_predicate(&mut state2, &inv.clone().authoritative(), &mut store, &hierarchy);
    assert_eq!(state2.get_current_type(b1).and_then(|k| k.ty()), Some(str_ty));
    assert!(state2.facts.contains(&inv));
}

#[test]
fn test_flow_join_is_epistemically_conservative_and_deterministic() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal(DeclarationId::new(ModuleId::universe_root(), "Int".into()));
    let string_ty = store.nominal(DeclarationId::new(ModuleId::universe_root(), "String".into()));
    let binding = BindingId(7);

    let mut established = FlowState::new();
    established.declare(
        binding,
        "value",
        SourceRange::default(),
        None,
        TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
        true,
    );
    established.bindings.get_mut(&binding).unwrap().denotation = Some(SemanticDenotation::Kind(KindId::TYPE));
    established.bindings.get_mut(&binding).unwrap().causal_invalidity =
        phalcom_semantic::checker::causal::CausalInvalidity::One(phalcom_semantic::identity::DiagnosticCauseId(1));

    let mut assumed = established.clone();
    assumed.bindings.get_mut(&binding).unwrap().current = TypeKnowledge::assumed(int_ty, EvidenceOrigin::DeveloperAnnotation);
    assumed.bindings.get_mut(&binding).unwrap().denotation = Some(SemanticDenotation::Kind(KindId::RECORD_ROW));
    assumed.bindings.get_mut(&binding).unwrap().causal_invalidity =
        phalcom_semantic::checker::causal::CausalInvalidity::One(phalcom_semantic::identity::DiagnosticCauseId(2));

    let joined = FlowState::join(&[established.clone(), assumed], &mut store);
    let joined_binding = joined.get_binding(binding).expect("binding survives reachable join");
    assert_eq!(joined_binding.current.ty(), Some(int_ty));
    assert_eq!(joined_binding.current.status(), Some(EvidenceStatus::Assumed));
    assert_eq!(joined_binding.denotation, None, "disagreeing denotations must not survive join");
    assert!(matches!(
        joined_binding.causal_invalidity,
        phalcom_semantic::checker::causal::CausalInvalidity::Multiple
    ));

    let mut unknown = established.clone();
    unknown.bindings.get_mut(&binding).unwrap().current = TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence);
    let unknown_join = FlowState::join(&[established.clone(), unknown], &mut store);
    assert!(matches!(
        unknown_join.get_current_type(binding),
        Some(TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence))
    ));

    let mut dynamic = established.clone();
    dynamic.bindings.get_mut(&binding).unwrap().current = TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape);
    let dynamic_join = FlowState::join(&[established.clone(), dynamic], &mut store);
    assert!(matches!(
        dynamic_join.get_current_type(binding),
        Some(TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape))
    ));

    let mut divergent = established;
    divergent.bindings.get_mut(&binding).unwrap().current = TypeKnowledge::established(string_ty, EvidenceOrigin::Syntax);
    let divergent_join = FlowState::join(&[divergent, joined], &mut store);
    assert!(matches!(divergent_join.get_current_type(binding), Some(TypeKnowledge::Known(_))));
    assert_ne!(divergent_join.get_current_type(binding).and_then(TypeKnowledge::ty), Some(string_ty));
}

#[test]
fn divergent_branch_contracts_fail_closed_without_first_branch_metadata() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal(DeclarationId::new(ModuleId::universe_root(), "Int".into()));
    let string_ty = store.nominal(DeclarationId::new(ModuleId::universe_root(), "String".into()));
    let binding = BindingId(11);

    let mut int_branch = FlowState::new();
    int_branch.declare(
        binding,
        "value",
        SourceRange::default(),
        Some(int_ty),
        TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
        true,
    );
    let mut string_branch = int_branch.clone();
    let string_state = string_branch.bindings.get_mut(&binding).expect("shared binding");
    string_state.contract.as_mut().expect("source contract").ty = string_ty;
    string_state.current = TypeKnowledge::established(string_ty, EvidenceOrigin::Syntax);

    let failure = FlowState::join_with_hierarchy(&[int_branch, string_branch], &mut store, &MapTypeHierarchy::new())
        .expect_err("divergent contracts are a flow invariant failure");
    assert!(matches!(failure, phalcom_semantic::checker::flow::state::FlowInvariantFailure::DivergentBindingContract { binding: id, .. } if id == binding));
}

#[test]
fn divergent_branch_mutability_is_a_flow_invariant_failure() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal(DeclarationId::new(ModuleId::universe_root(), "Int".into()));
    let binding = BindingId(12);
    let mut mutable_branch = FlowState::new();
    mutable_branch.declare(
        binding,
        "value",
        SourceRange::default(),
        Some(int_ty),
        TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
        true,
    );
    let mut immutable_branch = mutable_branch.clone();
    immutable_branch.bindings.get_mut(&binding).expect("shared binding").mutable = false;

    let failure = FlowState::join_with_hierarchy(&[mutable_branch, immutable_branch], &mut store, &MapTypeHierarchy::new())
        .expect_err("divergent mutability is a flow invariant failure");
    assert!(matches!(failure, phalcom_semantic::checker::flow::state::FlowInvariantFailure::DivergentMutability { binding: id, .. } if id == binding));
}

#[test]
fn current_value_disagreement_is_a_normal_flow_join() {
    let mut store = TypeStore::new();
    let module = ModuleId::universe_root();
    let int_ty = store.nominal(DeclarationId::new(module.clone(), "Int".into()));
    let string_ty = store.nominal(DeclarationId::new(module, "String".into()));
    let binding = BindingId(13);

    let mut left = FlowState::new();
    left.declare(
        binding,
        "value",
        SourceRange::default(),
        Some(int_ty),
        TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
        true,
    );
    let mut right = left.clone();
    right.bindings.get_mut(&binding).expect("shared binding").current = TypeKnowledge::established(string_ty, EvidenceOrigin::Flow);

    let joined = FlowState::join_with_hierarchy(&[left, right], &mut store, &MapTypeHierarchy::new()).expect("current facts may differ");
    assert_eq!(
        joined
            .get_binding(binding)
            .and_then(|state| state.contract.as_ref())
            .map(|contract| contract.ty),
        Some(int_ty)
    );
    assert_eq!(
        joined.get_current_type(binding).and_then(TypeKnowledge::ty),
        Some(store.union(&[int_ty, string_ty]))
    );
}

#[test]
fn loop_widening_rejects_divergent_persistent_contracts() {
    let mut store = TypeStore::new();
    let module = ModuleId::universe_root();
    let int_ty = store.nominal(DeclarationId::new(module.clone(), "Int".into()));
    let string_ty = store.nominal(DeclarationId::new(module, "String".into()));
    let binding = BindingId(14);

    let mut header = FlowState::new();
    header.declare(
        binding,
        "value",
        SourceRange::default(),
        Some(int_ty),
        TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
        true,
    );
    let mut next_header = header.clone();
    next_header
        .bindings
        .get_mut(&binding)
        .expect("shared binding")
        .contract
        .as_mut()
        .expect("persistent contract")
        .ty = string_ty;

    let failure = FlowState::widen_loop_state_with_hierarchy(&header, &next_header, &mut store, &MapTypeHierarchy::new())
        .expect_err("loop widening must reject divergent persistent contracts");
    assert!(matches!(
        failure,
        phalcom_semantic::checker::flow::state::FlowInvariantFailure::DivergentBindingContract { binding: id, .. } if id == binding
    ));
}

#[test]
fn test_mutation_invalidation_kills_dependent_facts() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let module = ModuleId::universe_root();
    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let str_decl = DeclarationId::new(module.clone(), "String".into());

    let int_ty = store.nominal(int_decl);
    let str_ty = store.nominal(str_decl);

    let b_x = BindingId(1);
    let b_y = BindingId(2);

    let mut state = FlowState::new();
    state.declare(
        b_x,
        "b_x",
        SourceRange::default(),
        None,
        TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
        true,
    );
    state.declare(
        b_y,
        "b_y",
        SourceRange::default(),
        None,
        TypeKnowledge::established(str_ty, EvidenceOrigin::Syntax),
        true,
    );

    let pred_x = FlowPredicate::EqualLiteral {
        binding: b_x,
        literal: "42".into(),
    };
    let pred_y = FlowPredicate::EqualLiteral {
        binding: b_y,
        literal: "\"hello\"".into(),
    };

    apply_predicate(&mut state, &pred_x.clone().derived(), &mut store, &hierarchy);
    apply_predicate(&mut state, &pred_y.clone().derived(), &mut store, &hierarchy);

    assert!(state.facts.contains(&pred_x));
    assert!(state.facts.contains(&pred_y));

    // Mutate b_x: assign new knowledge
    state.assign(b_x, TypeKnowledge::established(int_ty, EvidenceOrigin::Flow));

    // b_x facts must be invalidated, b_y facts must survive
    assert!(!state.facts.contains(&pred_x), "assigning b_x must invalidate facts about b_x");
    assert!(state.facts.contains(&pred_y), "assigning b_x must not invalidate facts about b_y");
}

#[test]
fn test_flow_state_conservative_join_and_loop_widening() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let module = ModuleId::universe_root();
    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let str_decl = DeclarationId::new(module.clone(), "String".into());

    let int_ty = store.nominal(int_decl);
    let str_ty = store.nominal(str_decl);

    let b1 = BindingId(1);

    let mut branch_a = FlowState::new();
    branch_a.declare(
        b1,
        "b1",
        SourceRange::default(),
        None,
        TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
        true,
    );
    let fact_shared = FlowPredicate::OrderedPredicate {
        binding: b1,
        op: ">".into(),
        threshold: 0,
    };
    let fact_a_only = FlowPredicate::EqualLiteral {
        binding: b1,
        literal: "10".into(),
    };
    apply_predicate(&mut branch_a, &fact_shared.clone().derived(), &mut store, &hierarchy);
    apply_predicate(&mut branch_a, &fact_a_only.clone().derived(), &mut store, &hierarchy);

    let mut branch_b = FlowState::new();
    branch_b.declare(
        b1,
        "b1",
        SourceRange::default(),
        None,
        TypeKnowledge::established(str_ty, EvidenceOrigin::Flow),
        true,
    );
    apply_predicate(&mut branch_b, &fact_shared.clone().derived(), &mut store, &hierarchy);

    let joined = FlowState::join(&[branch_a, branch_b], &mut store);
    assert!(joined.is_reachable());

    // Types join to Union(Int, String)
    let joined_ty = joined.get_current_type(b1).and_then(|k| k.ty()).unwrap();
    assert_eq!(joined_ty, store.union(&[int_ty, str_ty]));

    // Shared facts survive, branch-specific facts are killed
    assert!(joined.facts.contains(&fact_shared), "shared facts must survive join");
    assert!(!joined.facts.contains(&fact_a_only), "branch-only facts must not survive join");

    // Loop widening
    let mut next_header = joined.clone();
    next_header.assign(b1, TypeKnowledge::established(int_ty, EvidenceOrigin::Flow));
    let widened = FlowState::widen_loop_state(&joined, &next_header, &mut store).expect("matching loop invariants");
    assert!(widened.is_reachable());
}

#[test]
fn test_protocol_derived_iteration_typing_no_name_matching() {
    let (mut store, hier, resolver, decls, module) = setup_test_env();

    // 1. Generic List<Int>
    let list_decl = DeclarationId::new(module.clone(), "List".into());
    let list_form = decls.form(&list_decl).expect("generic list form");
    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let int_ty = store.nominal_type(int_decl);
    let list_int = store.apply_type_form(list_form, &[int_ty]).expect("valid application");

    let ctx = phalcom_semantic::checker::context::CheckingContext::new(&mut store, &hier, &resolver, &decls, module.clone());

    assert!(matches!(
        ctx.store.get(list_int),
        phalcom_semantic::types::store::TypeData::Applied { arguments, .. }
            if arguments.as_ref() == [phalcom_semantic::types::store::StaticArgument::Term(int_ty)]
    ));

    // 2. Program with for loop over List<String>
    let source = r#"
class IterationSubject {
  process(items: List<String>) {
    for s in items {
      const copy: String = s
    }
  }
}
"#;
    let program = parse_source(source, 0).expect("valid parse");
    let report = check_program(ctx.store, &hier, &resolver, &decls, module, &program);
    assert!(
        !report.has_errors(),
        "unknown iteration element must fail closed without inventing a mismatch: {:?}",
        report.diagnostics
    );
}
