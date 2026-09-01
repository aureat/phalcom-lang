use phalcom_ast::parse;
use phalcom_common::selector::Selector;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, ModuleId, SemanticTargetId};
use phalcom_semantic::{AccessContext, EditorMemberTarget, ReceiverAlternative, ReceiverMode, ResolvedReceiver, analyze_single_module};
use std::sync::Arc;

#[test]
fn editor_facade_returns_canonical_members_and_targets() {
    let source = "class Box { @constructor new() {} value() -> Int { 1 } }\n";
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "parser errors: {:?}", parsed.errors);
    let module = ModuleId::universe_root();
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed.program));
    let snapshot = analysis.snapshot;
    let owner = DeclarationId::new(module.clone(), "Box".into());
    let receiver = ResolvedReceiver {
        alternatives: Arc::from([ReceiverAlternative {
            declaration: owner.clone(),
            mode: ReceiverMode::Instance,
        }]),
    };

    let members = snapshot.editor().members_for_receiver(
        &receiver,
        &AccessContext {
            enclosing_declaration: None,
            enclosing_callable: None,
        },
    );
    let value = CallableId::new(owner.clone(), Selector::method("value", []).unwrap(), DispatchSide::Instance);
    assert!(members.iter().any(|member| member.target == EditorMemberTarget::Callable(value.clone())));

    let box_offset = source.find("Box").expect("class declaration");
    assert_eq!(
        snapshot.editor().target_at(&module, box_offset),
        Some(SemanticTargetId::Declaration(owner.clone()))
    );
    assert!(
        snapshot
            .editor()
            .definition_sites(&SemanticTargetId::Callable(value))
            .iter()
            .any(|site| snapshot.source_site(site).is_some())
    );
}

#[test]
fn editor_facade_fails_closed_for_unknown_receiver() {
    let source = "class Box {}\n";
    let parsed = parse(source, 0);
    let analysis = analyze_single_module(ModuleId::universe_root(), Arc::from(source), Arc::new(parsed.program));
    let snapshot = analysis.snapshot;
    let receiver = ResolvedReceiver { alternatives: Arc::from([]) };
    assert!(
        snapshot
            .editor()
            .members_for_receiver(
                &receiver,
                &AccessContext {
                    enclosing_declaration: None,
                    enclosing_callable: None,
                }
            )
            .is_empty()
    );
}
