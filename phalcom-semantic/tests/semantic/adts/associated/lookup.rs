//! Associated family lookup identity tests.

use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::associated::{AssociatedFamilyKind, AssociatedMemberId, build_associated_surface};
use phalcom_semantic::identity::{DeclarationId, VariantId};
use std::collections::HashSet;

#[test]
fn associated_lookup_returns_exact_variant_members_in_declaration_order() {
    let module = ModuleId::core();
    let owner = DeclarationId::new(module.clone(), "Animal".into());
    let variants = vec![
        VariantId::new(owner.clone(), Selector::method("Dog", [SelectorSlot::Positional]).expect("dog selector")),
        VariantId::new(
            owner.clone(),
            Selector::method("Dog", [SelectorSlot::Positional, SelectorSlot::Positional]).expect("dog selector"),
        ),
        VariantId::new(owner.clone(), Selector::getter("Cat").expect("cat selector")),
    ];

    let (surface, diagnostics) = build_associated_surface(&owner, Some(&variants), &[], &HashSet::new(), &module, None);
    assert!(diagnostics.is_empty(), "unexpected associated lookup diagnostics: {diagnostics:#?}");
    let dog = surface
        .families
        .get(&phalcom_common::selector::SelectorBase::Named("Dog".into()))
        .expect("Dog family");
    assert_eq!(dog.kind, AssociatedFamilyKind::Variant);
    assert_eq!(
        dog.members.as_ref(),
        &[
            AssociatedMemberId::Variant(variants[0].clone()),
            AssociatedMemberId::Variant(variants[1].clone()),
        ]
    );
    assert_eq!(dog.id, variants[0].family().expect("family").associated());
}
