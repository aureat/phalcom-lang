use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::enum_requirements::EnumRequirementId;
use phalcom_semantic::identity::DeclarationId;

#[test]
fn enum_requirement_identity_is_owner_and_selector_qualified() {
    let owner = DeclarationId::new(ModuleId::core(), "Shape".into());
    let describe = Selector::getter("describe").expect("describe selector");
    let other = Selector::getter("render").expect("render selector");

    let first = EnumRequirementId::new(owner.clone(), describe.clone());
    let same = EnumRequirementId::new(owner.clone(), describe);
    let different = EnumRequirementId::new(owner, other);

    assert_eq!(first, same);
    assert_ne!(first, different);
}
