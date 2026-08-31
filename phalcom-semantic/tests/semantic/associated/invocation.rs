use std::sync::Arc;

use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::AssociatedMemberId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::checker::AssociatedResolutionKind;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, InvocationTargetId, VariantId};

#[test]
fn associated_variant_forms_keep_value_and_constructor_invocation_distinct() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
enum Weird {
  @variant Marker
  @variant Marker()
  @variant Marker(_ value: Int)
}

class Probe {
  @class singleton() { Weird::Marker }
  @class nullary() { Weird::Marker() }
  @class unary() { Weird::Marker(1) }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let probe = DeclarationId::new(module.clone(), "Probe".into());
    let weird = DeclarationId::new(module, "Weird".into());

    let singleton = find_resolution(&analysis, &source, &probe, "Weird::Marker", "singleton");
    assert!(
        matches!(&singleton.kind, AssociatedResolutionKind::ExactValue { member: AssociatedMemberId::Variant(VariantId { selector, .. }), .. } if selector == &Selector::getter("Marker").expect("getter"))
    );

    let nullary = find_resolution(&analysis, &source, &probe, "Weird::Marker()", "nullary");
    assert!(
        matches!(&nullary.kind, AssociatedResolutionKind::StaticInvoke { member: AssociatedMemberId::Variant(VariantId { selector, owner }), target: InvocationTargetId::VariantConstructor(constructor), .. } if owner == &weird && selector == &Selector::method("Marker", []).expect("nullary") && &constructor.variant.selector == selector)
    );

    let unary = find_resolution(&analysis, &source, &probe, "Weird::Marker(1)", "unary");
    assert!(
        matches!(&unary.kind, AssociatedResolutionKind::StaticInvoke { member: AssociatedMemberId::Variant(VariantId { selector, .. }), target: InvocationTargetId::VariantConstructor(constructor), .. } if selector == &Selector::method("Marker", [SelectorSlot::Positional]).expect("unary") && &constructor.variant.selector == selector)
    );
}

fn find_resolution<'a>(
    analysis: &'a phalcom_semantic::workspace::SemanticAnalysis,
    source: &str,
    probe: &DeclarationId,
    expression: &str,
    callable: &str,
) -> &'a phalcom_semantic::checker::AssociatedResolution {
    let callable_id = CallableId::new(probe.clone(), Selector::method(callable, []).expect("callable selector"), DispatchSide::Class);
    let callable_analysis = analysis.snapshot.callable_analyses.get(&callable_id).expect("callable analysis");
    let expression = callable_analysis
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some(expression))
        .expect("associated expression");
    callable_analysis.associated_resolutions.get(&expression.id).expect("associated resolution")
}
