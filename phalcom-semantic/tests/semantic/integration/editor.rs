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
            receiver_type: None,
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
fn associated_type_declaration_and_binding_lhs_share_canonical_source_target() {
    let source = "trait Iterable { type Item }\nclass Box {}\nimpl Iterable for Box { type Item = Int }\n";
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "parser errors: {:?}", parsed.errors);
    let module = ModuleId::universe_root();
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed.program));
    assert!(!analysis.snapshot.has_errors(), "semantic diagnostics: {:?}", analysis.snapshot.diagnostics);
    let trait_declaration = DeclarationId::new(module.clone(), "Iterable".into());
    let requirement = analysis
        .snapshot
        .trait_surfaces
        .get(&trait_declaration)
        .expect("trait surface")
        .associated_type_by_name("Item")
        .expect("Item requirement")
        .requirement
        .clone();
    let declaration_offset = source.find("type Item").expect("associated declaration") + "type ".len();
    let binding_offset = source.rfind("type Item").expect("associated binding") + "type ".len();
    let target = SemanticTargetId::AssociatedType(requirement);
    assert_eq!(analysis.snapshot.editor().target_at(&module, declaration_offset), Some(target.clone()));
    assert_eq!(analysis.snapshot.editor().target_at(&module, binding_offset), Some(target.clone()));
    assert!(!analysis.snapshot.editor().definition_sites(&target).is_empty());
}

#[test]
fn editor_facade_projects_exact_trait_members_without_re_solving_them() {
    let source = r#"
trait Tagged { tag -> String }
class Value<T> {}
impl<T> Tagged for Value<T> { tag -> String { "generic" } }
class Caller { run(_ value: Value<Int>) -> String { value.tag } }
"#;
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "parser errors: {:?}", parsed.errors);
    let module = ModuleId::universe_root();
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed.program));
    assert!(!analysis.snapshot.has_errors(), "semantic diagnostics: {:#?}", analysis.snapshot.diagnostics);
    let caller = DeclarationId::new(module.clone(), "Caller".into());
    let run = CallableId::new(
        caller,
        Selector::method("run", [phalcom_common::selector::SelectorSlot::Positional]).unwrap(),
        DispatchSide::Instance,
    );
    let receiver_type = analysis
        .snapshot
        .callable_signatures
        .get(&run)
        .and_then(|signature| signature.parameters.first())
        .and_then(|parameter| parameter.declared_type.canonical_type())
        .expect("exact Value<Int> receiver type");
    let value = DeclarationId::new(module, "Value".into());
    let members = analysis.snapshot.editor().members_for_receiver(
        &ResolvedReceiver {
            alternatives: Arc::from([ReceiverAlternative {
                declaration: value,
                mode: ReceiverMode::Instance,
                receiver_type: Some(receiver_type),
            }]),
        },
        &AccessContext {
            enclosing_declaration: None,
            enclosing_callable: None,
        },
    );
    let tag = members
        .iter()
        .find_map(|member| match &member.target {
            EditorMemberTarget::Callable(callable) if callable.selector == Selector::getter("tag").unwrap() => Some(callable),
            _ => None,
        })
        .expect("trait-only tag member should be visible to editor queries");
    assert!(
        tag.conformance_owner().is_some(),
        "editor must retain the selected conformance callable: {tag:?}"
    );
}

#[test]
fn editor_facade_projects_data_component_trait_witness() {
    let source = r#"
trait Named { name -> String }
data Person(name: String)
impl Named for Person {}
class Caller { run(_ person: Person) -> String { person.name } }
"#;
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "parser errors: {:?}", parsed.errors);
    let module = ModuleId::universe_root();
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed.program));
    assert!(!analysis.snapshot.has_errors(), "semantic diagnostics: {:#?}", analysis.snapshot.diagnostics);
    let caller = DeclarationId::new(module.clone(), "Caller".into());
    let run = CallableId::new(
        caller,
        Selector::method("run", [phalcom_common::selector::SelectorSlot::Positional]).unwrap(),
        DispatchSide::Instance,
    );
    let receiver_type = analysis
        .snapshot
        .callable_signatures
        .get(&run)
        .and_then(|signature| signature.parameters.first())
        .and_then(|parameter| parameter.declared_type.canonical_type())
        .expect("exact Person receiver type");
    let person = DeclarationId::new(module, "Person".into());
    let members = analysis.snapshot.editor().members_for_receiver(
        &ResolvedReceiver {
            alternatives: Arc::from([ReceiverAlternative {
                declaration: person.clone(),
                mode: ReceiverMode::Instance,
                receiver_type: Some(receiver_type),
            }]),
        },
        &AccessContext {
            enclosing_declaration: None,
            enclosing_callable: None,
        },
    );
    assert!(
        members
            .iter()
            .any(|member| matches!(&member.target, EditorMemberTarget::DataComponent(component) if component.owner == person))
    );
}

#[test]
fn exact_case_trait_dispatch_expression_retains_selection() {
    let source = r#"
trait Tagged { tag -> String }
enum Result<T> {
  Ok(_ value: T)
  Error(_ message: String)
}
impl Tagged for Result<Int>::Ok(_) { tag -> String { "ok" } }
let result = Result<Int>::Ok(42).tag
"#;
    let location = SourceLocation {
        source_id: SourceId("/tmp/phalcom-exact-case-trait.ph".into()),
        display_path: "/tmp/phalcom-exact-case-trait.ph".into(),
    };
    let mut session = SemanticWorkspaceSession::new();
    let publication = session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: location.clone(),
            text: Arc::from(source),
            revision: SourceRevision(1),
            recovered_program: None,
        }])
        .expect("semantic publication");
    let snapshot = publication.snapshot;
    assert!(!snapshot.has_errors(), "semantic diagnostics: {:#?}", snapshot.diagnostics);
    let selected = snapshot.callable_analyses.values().any(|callable| {
        callable.expressions.values().any(|expression| {
            source
                .get(expression.range.start..expression.range.end)
                .is_some_and(|text| text == "Result<Int>::Ok(42).tag")
                && expression.trait_dispatch.is_some()
        })
    });
    assert!(selected, "exact enum-case getter must retain semantic trait selection");
}

#[test]
fn incomplete_trait_proof_does_not_become_runtime_dynamic() {
    let source = "trait Tagged { tag -> String }\nclass User {}\nimpl Tagged for User { tag -> Int { 1 } }\nclass Caller { read(_ user: User) -> String { user.tag } }\n";
    let location = SourceLocation {
        source_id: SourceId("/tmp/phalcom-incomplete-trait.ph".into()),
        display_path: "/tmp/phalcom-incomplete-trait.ph".into(),
    };
    let mut session = SemanticWorkspaceSession::new();
    let publication = session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: location,
            text: Arc::from(source),
            revision: SourceRevision(1),
            recovered_program: None,
        }])
        .expect("semantic publication");
    let snapshot = publication.snapshot;
    let expression = snapshot
        .callable_analyses
        .values()
        .flat_map(|callable| callable.expressions.values())
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("user.tag"))
        .expect("incomplete trait getter expression");
    assert!(matches!(
        expression.knowledge,
        phalcom_semantic::types::evidence::TypeKnowledge::Unknown(phalcom_semantic::types::evidence::UnknownReason::UncheckedExpression)
    ));
    assert!(!matches!(expression.knowledge, phalcom_semantic::types::evidence::TypeKnowledge::Dynamic(_)));
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
