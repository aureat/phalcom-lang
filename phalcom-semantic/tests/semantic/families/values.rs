use std::sync::Arc;

use phalcom_common::selector::{Selector, SelectorKind, SelectorSlot};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::AssociatedMemberId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::checker::AssociatedResolutionKind;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, VariantId};
use phalcom_semantic::types::denotation::{AssociatedValueDenotation, SemanticDenotation};

#[test]
fn captured_variant_family_preserves_exact_authorized_member_shapes() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
enum Weird {
  @variant Marker
  @variant Marker()
  @variant Marker(_ value: Int)
}
class Probe {
  @class run() { Weird::Marker::* }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let probe = DeclarationId::new(module.clone(), "Probe".into());
    let run = CallableId::new(probe, Selector::method("run", []).expect("run selector"), DispatchSide::Class);
    let callable = analysis.snapshot.callable_analyses.get(&run).expect("Probe.run analysis");
    let expression = callable
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some("Weird::Marker::*"))
        .expect("family capture expression");
    let resolution = callable.associated_resolutions.get(&expression.id).expect("family resolution");

    let AssociatedResolutionKind::Family { members, .. } = &resolution.kind else {
        panic!("expected family resolution, got {:?}", resolution.kind);
    };
    assert_eq!(members.len(), 3);
    assert!(members.iter().any(|member| member.operation.kind == SelectorKind::Getter));
    assert!(
        members
            .iter()
            .any(|member| member.operation.kind == SelectorKind::Method && member.operation.slots.is_empty())
    );
    assert!(
        members
            .iter()
            .any(|member| member.operation.kind == SelectorKind::Method && member.operation.slots.as_ref() == [SelectorSlot::Positional])
    );

    let SemanticDenotation::AssociatedValue(denotation) = expression.denotation.as_ref().expect("captured denotation") else {
        panic!("expected captured family denotation, got {:?}", expression.denotation);
    };
    let AssociatedValueDenotation::Family { members: captured, .. } = &**denotation else {
        panic!("expected captured family denotation, got {:?}", expression.denotation);
    };
    assert_eq!(captured.len(), 3);
    assert!(
        captured
            .iter()
            .all(|member| matches!(member.member, AssociatedMemberId::Variant(VariantId { .. })))
    );
}
