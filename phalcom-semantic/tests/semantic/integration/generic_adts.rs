use std::sync::Arc;

use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::checker::AssociatedResolutionKind;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};

#[test]
fn explicit_generic_enum_owner_specializes_direct_and_family_resolution() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
enum Option<T> {
  @variant Some(_ value: T)
  @variant None
}
class Probe {
  @class direct() { Option<Int>::Some(42) }
  @class family() { Option<Int>::Some::* }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let probe = DeclarationId::new(module, "Probe".into());

    let direct = callable_resolution(&analysis, &source, &probe, "direct", "Option<Int>::Some(42)");
    assert!(matches!(direct.kind, AssociatedResolutionKind::StaticInvoke { .. }));
    assert!(direct.owner_form != phalcom_semantic::TypeId::DUMMY);

    let family = callable_resolution(&analysis, &source, &probe, "family", "Option<Int>::Some::*");
    assert!(matches!(&family.kind, AssociatedResolutionKind::Family { members, .. } if members.len() == 1));
}

fn callable_resolution<'a>(
    analysis: &'a phalcom_semantic::workspace::SemanticAnalysis,
    source: &str,
    probe: &DeclarationId,
    callable_name: &str,
    text: &str,
) -> &'a phalcom_semantic::checker::AssociatedResolution {
    let callable = CallableId::new(
        probe.clone(),
        Selector::method(callable_name, []).expect("callable selector"),
        DispatchSide::Class,
    );
    let callable_analysis = analysis.snapshot.callable_analyses.get(&callable).expect("callable analysis");
    let expression = callable_analysis
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some(text))
        .expect("associated expression");
    callable_analysis.associated_resolutions.get(&expression.id).expect("associated resolution")
}
