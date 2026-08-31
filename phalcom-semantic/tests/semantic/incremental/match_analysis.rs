//! Incremental match-query dependency ownership scenarios.

use super::support::single_module_input;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::db::QueryKey;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;

fn module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(506), ModulePath::root())
}

#[test]
fn adt_incr_match_analysis_records_enum_and_callable_dependencies() {
    let module = module();
    let mut session = SemanticWorkspaceSession::new();
    let source = "enum Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 _ => 0 } } }\n";
    let update = session.update(single_module_input(module.clone(), source, 1));
    assert!(
        update
            .snapshot
            .callable_analyses
            .values()
            .any(|callable| !callable.match_resolutions.is_empty())
    );

    let callable = CallableId::new(
        DeclarationId::new(module.clone(), "Test".into()),
        Selector::method("run", [phalcom_common::selector::SelectorSlot::Positional]).expect("run"),
        DispatchSide::Instance,
    );
    let dependencies = session
        .db()
        .index()
        .dependencies_of(&QueryKey::CallableBody(callable))
        .expect("match body dependencies");
    assert!(
        dependencies
            .iter()
            .any(|edge| edge.dependency == QueryKey::EnumDeclaration(DeclarationId::new(module.clone(), "Choice".into())))
    );
}

#[test]
fn adt_incr_match_analysis_uses_one_shared_query_index() {
    let module = module();
    let mut session = SemanticWorkspaceSession::new();
    let source = "enum Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 _ => 0 } } }\n";
    let _ = session.update(single_module_input(module.clone(), source, 1));
    let body = QueryKey::CallableBody(CallableId::new(
        DeclarationId::new(module, "Test".into()),
        Selector::method("run", [phalcom_common::selector::SelectorSlot::Positional]).expect("run"),
        DispatchSide::Instance,
    ));
    assert!(session.db().index().dependencies_of(&body).is_some());
}

#[test]
#[ignore = "GATED: source-level match dependency product does not yet expose alias edges"]
fn adt_incr_match_analysis_records_alias_union_dependency() {
    let module = module();
    let mut session = SemanticWorkspaceSession::new();
    let source = "enum Choice { @variant A @variant B }\ntype ChoiceAlias = Choice\nclass Test { run(_ value: ChoiceAlias) { match value { Choice::A => 1 _ => 0 } } }\n";
    let update = session.update(single_module_input(module.clone(), source, 1));
    assert!(
        update
            .snapshot
            .callable_analyses
            .values()
            .any(|callable| !callable.match_resolutions.is_empty())
    );
    let callable = CallableId::new(
        DeclarationId::new(module, "Test".into()),
        Selector::method("run", [phalcom_common::selector::SelectorSlot::Positional]).expect("run"),
        DispatchSide::Instance,
    );
    let dependencies = session
        .db()
        .index()
        .dependencies_of(&QueryKey::CallableBody(callable))
        .expect("match body dependencies");
    assert!(!dependencies.is_empty(), "alias-backed match must retain dependency edges");
}
