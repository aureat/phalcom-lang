//! Family identity remains independent from individual selector identity.

use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::identity::{DeclarationId, VariantFamilyId, VariantId};

#[test]
fn overloaded_variant_selectors_share_one_variant_family() {
    let owner = DeclarationId::new(ModuleId::universe_root(), "Animal".into());
    let short = VariantId::new(owner.clone(), Selector::method("Dog", [SelectorSlot::Positional]).expect("selector"));
    let long = VariantId::new(
        owner,
        Selector::method("Dog", [SelectorSlot::Positional, SelectorSlot::Positional]).expect("selector"),
    );
    assert_eq!(short.family(), long.family());
    assert_eq!(short.family(), Some(VariantFamilyId::new(short.owner.clone(), "Dog")));
    assert_ne!(short, long);
}
