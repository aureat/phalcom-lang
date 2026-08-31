use std::sync::Arc;

use phalcom_common::selector::{Selector, SelectorBase};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::checker::AssociatedResolutionKind;
use phalcom_semantic::identity::{AssociatedFamilyId, CallableId, DeclarationId, DispatchSide};

#[test]
fn weird_variant_forms_compose_into_one_associated_family_with_three_resolutions() {
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
  @class payload() { Weird::Marker(1) }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let probe = DeclarationId::new(module.clone(), "Probe".into());
    let family = AssociatedFamilyId::new(DeclarationId::new(module, "Weird".into()), SelectorBase::Named("Marker".into()));

    for (callable_name, expression_text, expected_kind) in [
        ("singleton", "Weird::Marker", "value"),
        ("nullary", "Weird::Marker()", "invoke"),
        ("payload", "Weird::Marker(1)", "invoke"),
    ] {
        let callable = CallableId::new(
            probe.clone(),
            Selector::method(callable_name, []).expect("callable selector"),
            DispatchSide::Class,
        );
        let callable_analysis = analysis.snapshot.callable_analyses.get(&callable).expect("callable analysis");
        let expression = callable_analysis
            .expressions
            .values()
            .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some(expression_text))
            .expect("associated expression");
        let resolution = callable_analysis.associated_resolutions.get(&expression.id).expect("associated resolution");
        assert_eq!(resolution.family.as_ref(), Some(&family));
        assert!(matches!(
            (expected_kind, &resolution.kind),
            ("value", AssociatedResolutionKind::ExactValue { .. }) | ("invoke", AssociatedResolutionKind::StaticInvoke { .. })
        ));
    }
}
