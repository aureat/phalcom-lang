//! Incremental ADT declaration, candidate, and match-product scenarios.

use super::support::single_module_input;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::db::QueryKey;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::match_semantics::{MatchResolution, PatternResolution};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::snapshot::SemanticSnapshot;
use std::sync::Arc;

fn module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(505), ModulePath::root())
}

fn first_match(snapshot: &SemanticSnapshot) -> &MatchResolution {
    snapshot
        .callable_analyses
        .values()
        .flat_map(|callable| callable.match_resolutions.values())
        .next()
        .expect("fixture contains a match")
}

#[test]
fn adt_incr_01_adding_enum_case_invalidates_match_and_reports_new_residual() {
    let module = module();
    let mut session = SemanticWorkspaceSession::new();
    let source_a = "enum Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 Choice::B => 2 } } }\n";
    let source_b = "enum Choice { @variant A @variant B @variant C }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 Choice::B => 2 } } }\n";
    let _ = session.update(single_module_input(module.clone(), source_a, 1));
    let update = session.update(single_module_input(module.clone(), source_b, 2));
    assert!(
        update
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::MatchNonExhaustive)
    );
    assert!(!matches!(
        first_match(&update.snapshot).exhaustiveness,
        phalcom_semantic::match_semantics::ExhaustivenessResult::Proven
    ));
}

#[test]
fn adt_incr_02_removing_enum_case_changes_candidate_universe() {
    let module = module();
    let mut session = SemanticWorkspaceSession::new();
    let source_a = "enum Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 _ => 0 } } }\n";
    let source_b = "enum Choice { @variant A }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 } } }\n";
    let _ = session.update(single_module_input(module.clone(), source_a, 1));
    let update = session.update(single_module_input(module.clone(), source_b, 2));
    let owner = DeclarationId::new(module.clone(), "Choice".into());
    assert_eq!(update.snapshot.enum_semantics.enum_info(&owner).expect("Choice metadata").variants.len(), 1);
}

#[test]
fn adt_incr_03_adding_family_member_changes_family_candidate_set() {
    let module = module();
    let mut session = SemanticWorkspaceSession::new();
    let source_a = "enum Animal { @variant Dog @variant Cat }\nclass Test { run(_ value: Animal) { match value { Dog* => 1 Cat => 2 } } }\n";
    let source_b = "enum Animal { @variant Dog @variant Dog() @variant Cat }\nclass Test { run(_ value: Animal) { match value { Dog* => 1 Cat => 2 } } }\n";
    let first = session.update(single_module_input(module.clone(), source_a, 1));
    let second = session.update(single_module_input(module, source_b, 2));
    let first_pattern = &first_match(&first.snapshot).arms[0].pattern;
    let second_pattern = &first_match(&second.snapshot).arms[0].pattern;
    let first_count = match first_pattern {
        PatternResolution::Variant(pattern) => pattern.candidates.len(),
        _ => 0,
    };
    let second_count = match second_pattern {
        PatternResolution::Variant(pattern) => pattern.candidates.len(),
        _ => 0,
    };
    assert!(second_count > first_count);
}

#[test]
#[ignore = "RED: selector-family callable candidate invalidation remains incomplete"]
fn adt_incr_04_adding_callable_family_member_changes_gap_candidates() {
    let module = module();
    let mut session = SemanticWorkspaceSession::new();
    let source_a = "enum Animal { @variant Dog(_ age: Int) @variant Cat }\nclass Test { run(_ value: Animal) { match value { Dog* => 1 _ => 0 } } }\n";
    let source_b = "enum Animal { @variant Dog(_ age: Int) @variant Dog(_ age: Int, breed: String) @variant Cat }\nclass Test { run(_ value: Animal) { match value { Dog* => 1 _ => 0 } } }\n";
    let first = session.update(single_module_input(module.clone(), source_a, 1));
    let second = session.update(single_module_input(module, source_b, 2));
    assert_ne!(format!("{:?}", first_match(&first.snapshot)), format!("{:?}", first_match(&second.snapshot)));
}

#[test]
fn adt_incr_05_payload_type_edit_invalidates_binding_product() {
    let module = module();
    let mut session = SemanticWorkspaceSession::new();
    let source_a = "enum Boxed { @variant Value(_ value: Int) }\nclass Test { run(_ value: Boxed) { match value { Boxed::Value(x) => x } } }\n";
    let source_b = "enum Boxed { @variant Value(_ value: String) }\nclass Test { run(_ value: Boxed) { match value { Boxed::Value(x) => x } } }\n";
    let first = session.update(single_module_input(module.clone(), source_a, 1));
    let second = session.update(single_module_input(module.clone(), source_b, 2));
    let callable = CallableId::new(
        DeclarationId::new(module, "Test".into()),
        Selector::method("run", [phalcom_common::selector::SelectorSlot::Positional]).expect("run"),
        DispatchSide::Instance,
    );
    let first_analysis = first.snapshot.callable_analyses.get(&callable).expect("first match analysis");
    let second_analysis = second.snapshot.callable_analyses.get(&callable).expect("second match analysis");
    assert!(!Arc::ptr_eq(first_analysis, second_analysis));
}

#[test]
fn adt_incr_06_gadt_specialization_edit_changes_branch_product() {
    let module = module();
    let mut session = SemanticWorkspaceSession::new();
    let source_a = "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Test { run(_ value: Expr<Int>) { match value { Expr::Int(x) => x } } }\n";
    let source_b = "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Test { run(_ value: Expr<Bool>) { match value { Expr::Bool(x) => x } } }\n";
    let first = session.update(single_module_input(module.clone(), source_a, 1));
    let second = session.update(single_module_input(module, source_b, 2));
    assert_ne!(format!("{:?}", first_match(&first.snapshot)), format!("{:?}", first_match(&second.snapshot)));
}

#[test]
#[ignore = "GATED: alias declaration fixture is required"]
fn adt_incr_07_alias_union_expansion_invalidates_exhaustiveness() {
    let module = module();
    let mut session = SemanticWorkspaceSession::new();
    let source_a = "enum Choice { @variant A @variant B }\ntype ChoiceAlias = Choice\nclass Test { run(_ value: ChoiceAlias) { match value { Choice::A => 1 Choice::B => 2 } } }\n";
    let source_b = "enum Choice { @variant A @variant B @variant C }\ntype ChoiceAlias = Choice\nclass Test { run(_ value: ChoiceAlias) { match value { Choice::A => 1 Choice::B => 2 } } }\n";
    let _ = session.update(single_module_input(module.clone(), source_a, 1));
    let update = session.update(single_module_input(module, source_b, 2));
    assert!(
        update
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::MatchNonExhaustive)
    );
}

#[test]
#[ignore = "GATED: alias declaration fixture is required"]
fn adt_incr_08_alias_union_contraction_updates_residual_witness() {
    let module = module();
    let mut session = SemanticWorkspaceSession::new();
    let source_a = "enum Choice { @variant A @variant B @variant C }\ntype ChoiceAlias = Choice\nclass Test { run(_ value: ChoiceAlias) { match value { Choice::A => 1 Choice::B => 2 } } }\n";
    let source_b = "enum Choice { @variant A @variant B }\ntype ChoiceAlias = Choice\nclass Test { run(_ value: ChoiceAlias) { match value { Choice::A => 1 Choice::B => 2 } } }\n";
    let _ = session.update(single_module_input(module.clone(), source_a, 1));
    let update = session.update(single_module_input(module, source_b, 2));
    assert!(matches!(
        first_match(&update.snapshot).exhaustiveness,
        phalcom_semantic::match_semantics::ExhaustivenessResult::Proven
    ));
}

#[test]
#[ignore = "GATED: cross-module visibility fixture is required"]
fn adt_incr_09_visibility_edit_invalidates_acquisition_without_shrinking_match_universe() {
    let module = module();
    let mut session = SemanticWorkspaceSession::new();
    let source_a = "enum Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 _ => 0 } } }\n";
    let source_b = "enum Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 2 _ => 0 } } }\n";
    let first = session.update(single_module_input(module.clone(), source_a, 1));
    let second = session.update(single_module_input(module, source_b, 2));
    assert_eq!(first_match(&first.snapshot).arms.len(), first_match(&second.snapshot).arms.len());
    assert_ne!(format!("{:?}", first_match(&first.snapshot)), format!("{:?}", first_match(&second.snapshot)));
}

#[test]
fn adt_incr_10_unrelated_method_edit_reuses_match_analysis() {
    let module = module();
    let mut session = SemanticWorkspaceSession::new();
    let source_a = "enum Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 _ => 0 } } keep() { 1 } }\n";
    let source_b = "enum Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 _ => 0 } } keep() { 2 } }\n";
    let first = session.update(single_module_input(module.clone(), source_a, 1));
    let callable = CallableId::new(
        DeclarationId::new(module.clone(), "Test".into()),
        Selector::method("run", [phalcom_common::selector::SelectorSlot::Positional]).expect("run"),
        DispatchSide::Instance,
    );
    let first_analysis = first.snapshot.callable_analyses.get(&callable).expect("run analysis");
    let second = session.update(single_module_input(module, source_b, 2));
    let second_analysis = second.snapshot.callable_analyses.get(&callable).expect("run analysis");
    assert!(Arc::ptr_eq(first_analysis, second_analysis));
}

#[test]
fn adt_incr_11_whitespace_edit_does_not_change_enum_product_fingerprint() {
    let module = module();
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(module.clone(), "enum Choice { @variant A @variant B }\n", 1));
    let owner = DeclarationId::new(module.clone(), "Choice".into());
    let first_fp = session
        .db()
        .ready_product_fingerprint(&QueryKey::EnumDeclaration(owner.clone()))
        .expect("enum fingerprint");
    let _ = session.update(single_module_input(module, "enum Choice {\n\n  @variant A\n  @variant B\n}\n", 2));
    let second_fp = session
        .db()
        .ready_product_fingerprint(&QueryKey::EnumDeclaration(owner))
        .expect("enum fingerprint");
    assert_eq!(first_fp, second_fp);
    assert_eq!(first.snapshot.store.id(), session.last_snapshot().expect("snapshot").store.id());
}

#[test]
fn adt_incr_12_candidate_semantic_change_changes_match_product() {
    let module = module();
    let mut session = SemanticWorkspaceSession::new();
    let source_a = "enum Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 _ => 0 } } }\n";
    let source_b = "enum Choice { @variant A @variant B @variant C }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 _ => 0 } } }\n";
    let first = session.update(single_module_input(module.clone(), source_a, 1));
    let second = session.update(single_module_input(module, source_b, 2));
    assert_ne!(format!("{:?}", first_match(&first.snapshot)), format!("{:?}", first_match(&second.snapshot)));
}
