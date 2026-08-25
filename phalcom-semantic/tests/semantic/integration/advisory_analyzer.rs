use std::collections::BTreeMap;

use phalcom_ast::ast::Statement;
use phalcom_ast::parser::parse;
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_semantic::advisory::{AdvisoryBuiltins, AdvisoryExpressionContext, AdvisoryFlowContext, AdvisoryOrigin, analyze_expr, analyze_statements};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, FieldId, SourceOwner, SourceSiteId, SourceSiteLocalId};
use phalcom_semantic::source_index::build_source_scope_index;
use phalcom_semantic::{AdvisoryConfidence, AdvisoryFact, ModuleId, SourceIndexContext, ValueShape};

fn declaration(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::core(), name.into())
}

fn context<'a>(
    source: &'a phalcom_semantic::source_index::SourceScopeIndex,
    bindings: &'a BTreeMap<SourceSiteId, AdvisoryFact>,
    fields: &'a BTreeMap<FieldId, AdvisoryFact>,
    returns: &'a BTreeMap<CallableId, AdvisoryFact>,
    builtins: &'a AdvisoryBuiltins,
    site_for_range: &'a dyn Fn(SourceRange) -> Option<SourceSiteId>,
    resolved_callable: &'a dyn Fn(SourceRange) -> Option<CallableId>,
) -> AdvisoryExpressionContext<'a> {
    AdvisoryExpressionContext {
        scope_index: source,
        scope: source.root,
        bindings,
        fields,
        callable_returns: returns,
        builtins,
        current_owner: None,
        dispatch_side: DispatchSide::Instance,
        source_site_for_range: site_for_range,
        resolved_callable_for_range: resolved_callable,
        resolve_callable_for_shape: None,
        resolve_method_family: None,
        call_observer: None,
        expression_observer: None,
    }
}

#[test]
fn analyzer_preserves_exact_collection_shapes_with_canonical_builtin_ids() {
    let parsed = parse("[1, 2]", 0);
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let source = build_source_scope_index(ModuleId::core(), &parsed.program, &SourceIndexContext::default());
    let builtins = AdvisoryBuiltins {
        int: Some(declaration("Int")),
        ..AdvisoryBuiltins::default()
    };
    let bindings = BTreeMap::new();
    let fields = BTreeMap::new();
    let returns = BTreeMap::new();
    let site_for_range = |range: SourceRange| {
        Some(SourceSiteId {
            owner: SourceOwner::Module(ModuleId::core()),
            local: SourceSiteLocalId(range.start as u32),
        })
    };
    let no_call = |_range: SourceRange| None;
    let Statement::Expr { expr, .. } = &parsed.program.statements[0] else {
        panic!("expected expression")
    };

    let fact = analyze_expr(expr, &context(&source, &bindings, &fields, &returns, &builtins, &site_for_range, &no_call));

    assert_eq!(fact.confidence, AdvisoryConfidence::Exact);
    assert!(matches!(fact.shape, ValueShape::ExactList(elements) if elements.len() == 2));
    assert!(matches!(fact.provenance.as_slice(), [AdvisoryOrigin::Syntax(_)]));
}

#[test]
fn analyzer_resolves_locals_through_compiler_scope_identity() {
    let parsed = parse("let value = [1, 2]\nvalue", 0);
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let source = build_source_scope_index(ModuleId::core(), &parsed.program, &SourceIndexContext::default());
    let binding = source
        .bindings
        .values()
        .find(|binding| binding.name.as_ref() == "value")
        .expect("value binding");
    let int = ValueShape::Instance(declaration("Int"));
    let list = ValueShape::ExactList(vec![int.clone(), int].into());
    let bindings = BTreeMap::from([(
        binding.declaration_site.clone(),
        AdvisoryFact::exact(list.clone(), AdvisoryOrigin::Binding(binding.declaration_site.clone())),
    )]);
    let returns = BTreeMap::new();
    let fields = BTreeMap::new();
    let builtins = AdvisoryBuiltins::default();
    let site_for_range = |range: SourceRange| {
        Some(SourceSiteId {
            owner: SourceOwner::Module(ModuleId::core()),
            local: SourceSiteLocalId(range.start as u32),
        })
    };
    let no_call = |_range: SourceRange| None;
    let Statement::Expr { expr, .. } = &parsed.program.statements[1] else {
        panic!("expected expression")
    };

    let fact = analyze_expr(expr, &context(&source, &bindings, &fields, &returns, &builtins, &site_for_range, &no_call));

    assert_eq!(fact.shape, list);
    assert_eq!(fact.confidence, AdvisoryConfidence::Flow);
    assert!(
        fact.provenance
            .iter()
            .any(|origin| matches!(origin, AdvisoryOrigin::Binding(site) if site == &binding.declaration_site))
    );
}

#[test]
fn analyzer_reuses_formal_resolved_callable_for_return_shape() {
    let parsed = parse("answer()", 0);
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let source = build_source_scope_index(ModuleId::core(), &parsed.program, &SourceIndexContext::default());
    let target = CallableId::new(declaration("Answer"), Selector::getter("answer").unwrap(), DispatchSide::Instance);
    let returns = BTreeMap::from([(
        target.clone(),
        AdvisoryFact::flow(ValueShape::Instance(declaration("String")), AdvisoryOrigin::Callable(target.clone())),
    )]);
    let bindings = BTreeMap::new();
    let fields = BTreeMap::new();
    let builtins = AdvisoryBuiltins::default();
    let site_for_range = |_range: SourceRange| None;
    let resolved = move |_range: SourceRange| Some(target.clone());
    let Statement::Expr { expr, .. } = &parsed.program.statements[0] else {
        panic!("expected expression")
    };

    let fact = analyze_expr(expr, &context(&source, &bindings, &fields, &returns, &builtins, &site_for_range, &resolved));

    assert_eq!(fact.shape, ValueShape::Instance(declaration("String")));
    assert_eq!(fact.confidence, AdvisoryConfidence::Interprocedural);
}

#[test]
fn analyzer_does_not_fabricate_missing_builtin_identity() {
    let parsed = parse("1", 0);
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let source = build_source_scope_index(ModuleId::core(), &parsed.program, &SourceIndexContext::default());
    let bindings = BTreeMap::new();
    let returns = BTreeMap::new();
    let fields = BTreeMap::new();
    let builtins = AdvisoryBuiltins::default();
    let site_for_range = |_range: SourceRange| None;
    let no_call = |_range: SourceRange| None;
    let Statement::Expr { expr, .. } = &parsed.program.statements[0] else {
        panic!("expected expression")
    };

    let fact = analyze_expr(expr, &context(&source, &bindings, &fields, &returns, &builtins, &site_for_range, &no_call));

    assert_eq!(fact.shape, ValueShape::Unknown);
}

#[test]
fn flow_product_shares_binding_environment_across_initializer_and_use() {
    let parsed = parse("let values = [1, 2]\nvalues", 0);
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let source = build_source_scope_index(ModuleId::core(), &parsed.program, &SourceIndexContext::default());
    let fields = BTreeMap::new();
    let returns = BTreeMap::new();
    let builtins = AdvisoryBuiltins {
        int: Some(declaration("Int")),
        ..AdvisoryBuiltins::default()
    };
    let site_for_range = |range: SourceRange| {
        Some(SourceSiteId {
            owner: SourceOwner::Module(ModuleId::core()),
            local: SourceSiteLocalId(range.start as u32),
        })
    };
    let no_call = |_range: SourceRange| None;
    let context = AdvisoryFlowContext {
        scope_index: &source,
        fields: &fields,
        callable_returns: &returns,
        builtins: &builtins,
        current_owner: None,
        dispatch_side: DispatchSide::Instance,
        source_site_for_range: &site_for_range,
        resolved_callable_for_range: &no_call,
        resolve_callable_for_shape: None,
        resolve_method_family: None,
    };

    let product = analyze_statements(&parsed.program.statements, &context, BTreeMap::new());

    let binding = source
        .bindings
        .values()
        .find(|binding| binding.name.as_ref() == "values")
        .expect("values binding");
    assert_eq!(
        product.bindings.get(&binding.declaration_site).map(|fact| &fact.shape),
        Some(&ValueShape::ExactList(
            vec![ValueShape::Instance(declaration("Int")), ValueShape::Instance(declaration("Int"))].into(),
        ))
    );
    assert!(product.expressions.values().any(|fact| matches!(fact.shape, ValueShape::ExactList(_))));
}
