use phalcom_ast::parse;
use phalcom_common::selector::Selector;
use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, ModuleId, SemanticTargetId};
use phalcom_semantic::{
    AccessContext, EditorMemberTarget, ReceiverAlternative, ReceiverMode, ResolvedReceiver, SemanticWorkspaceSession, analyze_single_module,
};
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

#[test]
fn editor_visible_symbols_use_prelude_policy_and_preserve_local_shadowing() {
    let location = SourceLocation {
        source_id: SourceId("/tmp/phalcom-editor-prelude.ph".into()),
        display_path: "/tmp/phalcom-editor-prelude.ph".into(),
    };
    let source = "class Probe { run() { let Int = 1\n Int } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let publication = session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: location.clone(),
            text: Arc::from(source),
            revision: SourceRevision(1),
            recovered_program: None,
        }])
        .expect("semantic publication");
    let module = publication.snapshot.module_for_source(&location.source_id).cloned().expect("source module");
    let offset = source.rfind("Int").expect("local Int read") + 1;
    let symbols = publication.snapshot.editor().visible_symbols_at(&module, offset);

    for name in ["Int", "Option", "Result", "List", "Map", "Unit"] {
        assert!(
            symbols.iter().any(|symbol| symbol.name.as_ref() == name),
            "missing prelude symbol {name}: {symbols:#?}"
        );
    }
    for name in ["Nil", "Some", "None", "Behavior", "Metaclass", "Method", "Family"] {
        assert!(
            symbols.iter().all(|symbol| symbol.name.as_ref() != name),
            "non-prelude symbol leaked into editor visibility: {name}: {symbols:#?}"
        );
    }

    let canonical_int = SemanticTargetId::Declaration(phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Int));
    let visible_ints = symbols.iter().filter(|symbol| symbol.name.as_ref() == "Int").collect::<Vec<_>>();
    assert_eq!(visible_ints.len(), 1, "local binding must suppress the prelude candidate");
    assert_ne!(visible_ints[0].target, canonical_int, "local Int must win over prelude Int");
}
