use phalcom_modules::{DunderPolicy, DunderPolicyError, DunderRole};

#[test]
fn runtime_spelling_is_not_the_policy_but_user_source_roles_are_reserved() {
    let policy = DunderPolicy::default();
    assert!(DunderPolicy::is_dunder("__module__"));
    assert!(matches!(
        policy.validate_user_declaration("__module__", DunderRole::Binding),
        Err(DunderPolicyError::Reserved { .. })
    ));
    assert!(matches!(
        policy.validate_user_declaration("__whatever__", DunderRole::Method),
        Err(DunderPolicyError::Unknown { .. })
    ));
    assert!(policy.validate_user_declaration("ordinary_name", DunderRole::Binding).is_ok());
}

#[test]
fn standardized_hook_can_be_opened_for_exactly_one_role() {
    static METHOD_ONLY: &[DunderRole] = &[DunderRole::Method];
    let policy = DunderPolicy::default().with_hook("__test_hook__", METHOD_ONLY);
    assert!(policy.validate_user_declaration("__test_hook__", DunderRole::Method).is_ok());
    assert!(policy.validate_user_declaration("__test_hook__", DunderRole::Field).is_err());
    assert!(policy.validate_user_declaration("__test_hook__", DunderRole::Export).is_err());
}
