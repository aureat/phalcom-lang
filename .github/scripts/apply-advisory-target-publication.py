from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"anchor not found in {path}: {old[:100]!r}")
    p.write_text(text.replace(old, new, 1))


analyzer = "phalcom-semantic/src/advisory/analyzer.rs"
replace_once(
    analyzer,
    "    /// Exact call expression range.\n    pub range: SourceRange,",
    "    /// Exact source range of the written selector/name token, when present.\n    pub target_range: Option<SourceRange>,\n    /// Exact call expression range.\n    pub range: SourceRange,",
)
replace_once(
    analyzer,
    "            resolved_call_or_unknown_with_shape(call.range, &receiver.shape, &call.method, &call.args, &arguments, context)",
    "            resolved_call_or_unknown_with_shape(\n                call.range,\n                call.method_range,\n                &receiver.shape,\n                &call.method,\n                &call.args,\n                &arguments,\n                context,\n            )",
)
replace_once(
    analyzer,
    "            resolved_call_or_unknown_with_arguments(call.range, &arguments, context)",
    "            resolved_call_or_unknown_with_arguments_at(call.range, call.name_range, &arguments, context)",
)
replace_once(
    analyzer,
    """fn resolved_call_or_unknown_with_arguments(range: SourceRange, arguments: &[AdvisoryCallArgument], context: &AdvisoryExpressionContext<'_>) -> AdvisoryFact {
    let Some(callable) = (context.resolved_callable_for_range)(range) else {
        return unknown_at(context, range);
    };
    resolved_callable_fact(callable, None, range, arguments, context)
}
""",
    """fn resolved_call_or_unknown_with_arguments(range: SourceRange, arguments: &[AdvisoryCallArgument], context: &AdvisoryExpressionContext<'_>) -> AdvisoryFact {
    resolved_call_or_unknown_with_arguments_at(range, None, arguments, context)
}

fn resolved_call_or_unknown_with_arguments_at(
    range: SourceRange,
    target_range: Option<SourceRange>,
    arguments: &[AdvisoryCallArgument],
    context: &AdvisoryExpressionContext<'_>,
) -> AdvisoryFact {
    let Some(callable) = (context.resolved_callable_for_range)(range) else {
        return unknown_at(context, range);
    };
    resolved_callable_fact(callable, None, range, target_range, arguments, context)
}
""",
)
replace_once(
    analyzer,
    """fn resolved_call_or_unknown_with_shape(
    range: SourceRange,
    receiver: &ValueShape,""",
    """fn resolved_call_or_unknown_with_shape(
    range: SourceRange,
    target_range: Option<SourceRange>,
    receiver: &ValueShape,""",
)
replace_once(
    analyzer,
    "    resolved_callable_fact(callable, Some(receiver), range, arguments, context)",
    "    resolved_callable_fact(callable, Some(receiver), range, target_range, arguments, context)",
)
replace_once(
    analyzer,
    """fn resolved_callable_fact(
    callable: CallableId,
    receiver: Option<&ValueShape>,
    range: SourceRange,
    arguments: &[AdvisoryCallArgument],""",
    """fn resolved_callable_fact(
    callable: CallableId,
    receiver: Option<&ValueShape>,
    range: SourceRange,
    target_range: Option<SourceRange>,
    arguments: &[AdvisoryCallArgument],""",
)
replace_once(
    analyzer,
    "    observe_call(callable.clone(), transfer_target.clone(), range, arguments, context);",
    "    observe_call(callable.clone(), transfer_target.clone(), range, target_range, arguments, context);",
)
replace_once(
    analyzer,
    """fn observe_call(
    target: CallableId,
    transfer_target: CallableId,
    range: SourceRange,
    arguments: &[AdvisoryCallArgument],""",
    """fn observe_call(
    target: CallableId,
    transfer_target: CallableId,
    range: SourceRange,
    target_range: Option<SourceRange>,
    arguments: &[AdvisoryCallArgument],""",
)
replace_once(
    analyzer,
    """        observer(AdvisoryCallObservation {
            target,
            transfer_target,
            range,""",
    """        observer(AdvisoryCallObservation {
            target,
            transfer_target,
            target_range,
            range,""",
)

flow = "phalcom-semantic/src/advisory/flow.rs"
replace_once(
    flow,
    "    /// Parameter facts contributed by resolved call sites in this traversal.\n    pub parameter_contributions: BTreeMap<AdvisoryParameterSlot, AdvisoryFact>,",
    "    /// Canonical call targets keyed by the exact selector/name source range.\n    pub call_targets: Vec<(SourceRange, CallableId)>,\n    /// Parameter facts contributed by resolved call sites in this traversal.\n    pub parameter_contributions: BTreeMap<AdvisoryParameterSlot, AdvisoryFact>,",
)
replace_once(
    flow,
    """fn record_call_contributions(product: &mut AdvisoryFlowProduct, context: &AdvisoryFlowContext<'_>, call: AdvisoryCallObservation) {
    let mut positional = 0;""",
    """fn record_call_contributions(product: &mut AdvisoryFlowProduct, context: &AdvisoryFlowContext<'_>, call: AdvisoryCallObservation) {
    if let Some(range) = call.target_range {
        product.call_targets.push((range, call.target.clone()));
    }
    let mut positional = 0;""",
)

session = "phalcom-semantic/src/session.rs"
replace_once(
    session,
    "            let mut member_bodies = BTreeMap::new();",
    """            let target_site_for_range = |range: SourceRange| {
                let candidates = module_index
                    .occurrences
                    .all()
                    .iter()
                    .filter(|occurrence| occurrence.range == range)
                    .map(|occurrence| occurrence.site.clone())
                    .collect::<Vec<_>>();
                (candidates.len() == 1).then(|| candidates[0].clone())
            };

            let mut member_bodies = BTreeMap::new();""",
)
replace_once(
    session,
    """                expressions.extend(flow.expressions);
                bindings.extend(flow.bindings);""",
    """                for (range, callable) in &flow.call_targets {
                    if let Some(site) = target_site_for_range(*range) {
                        let target = SemanticTargetId::Callable(callable.clone());
                        targets
                            .entry(site.clone())
                            .or_insert_with(|| advisory_target_resolution(&site, &target));
                    }
                }
                expressions.extend(flow.expressions);
                bindings.extend(flow.bindings);""",
)
replace_once(
    session,
    """            expressions.extend(top_level.expressions);
            bindings.extend(top_level.bindings);""",
    """            for (range, callable) in &top_level.call_targets {
                if let Some(site) = target_site_for_range(*range) {
                    let target = SemanticTargetId::Callable(callable.clone());
                    targets
                        .entry(site.clone())
                        .or_insert_with(|| advisory_target_resolution(&site, &target));
                }
            }
            expressions.extend(top_level.expressions);
            bindings.extend(top_level.bindings);""",
)

test = Path("phalcom-semantic/tests/constructor_factory_probe.rs")
text = test.read_text()
marker = "\nfn builtin_annotation_snapshot()"
if marker not in text:
    raise SystemExit("constructor probe insertion marker missing")
new_test = r'''

#[test]
fn advisory_inherited_call_publishes_defining_callable_target() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
class Animal {
  speak() {}
}

class Dog is Animal {}

const dog = Dog.new()
dog.speak()
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let snapshot = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program)).snapshot;

    let animal = DeclarationId::new(module.clone(), "Animal".into());
    let speak = CallableId::new(
        animal,
        Selector::method("speak", Vec::new()).unwrap(),
        DispatchSide::Instance,
    );
    let selector_offset = source.rfind("speak()").expect("call-site speak") + 1;
    assert_eq!(
        snapshot.editor().target_at(&module, selector_offset),
        Some(SemanticTargetId::Callable(speak)),
        "advisory dispatch must publish the canonical defining callable at the selector occurrence"
    );
}
'''
test.write_text(text.replace(marker, new_test + marker, 1))

print("advisory target publication candidate applied")
