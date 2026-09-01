use phalcom_common::selector::{Selector, SelectorBase};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::identity::{AssociatedFamilyId, DeclarationId, InvocationTargetId, VariantId};
use phalcom_semantic::types::denotation::{AssociatedValueDenotation, CapturedAssociatedMember, SemanticDenotation, ValueSemanticFact};
use phalcom_semantic::types::evidence::{EvidenceOrigin, TypeKnowledge};
use phalcom_semantic::types::family::FamilyOperationShape;
use phalcom_semantic::types::store::TypeStore;

#[test]
fn flow_merge_preserves_identical_family_capability_denotation() {
    let mut store = TypeStore::new();
    let owner = DeclarationId::new(ModuleId::universe_root(), "Option".into());
    let family = AssociatedFamilyId::new(owner.clone(), SelectorBase::Named("make".into()));
    let variant = VariantId::new(owner.clone(), Selector::getter("None").expect("None"));
    let captured = CapturedAssociatedMember {
        operation: FamilyOperationShape::getter(),
        member: phalcom_semantic::AssociatedMemberId::Variant(variant),
        target: Some(InvocationTargetId::variant_constructor(VariantId::new(
            owner,
            Selector::getter("None").expect("None"),
        ))),
    };
    let denotation = SemanticDenotation::AssociatedValue(Box::new(AssociatedValueDenotation::family(
        store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "Option".into())),
        DeclarationId::new(ModuleId::universe_root(), "Option".into()),
        family,
        vec![captured],
    )));
    let fact = ValueSemanticFact {
        knowledge: TypeKnowledge::established(store.unit(), EvidenceOrigin::DeclarationSemantics),
        denotation: Some(denotation),
    };

    let merged = ValueSemanticFact::merge(&fact, &fact, fact.knowledge.clone());

    assert_eq!(merged.denotation, fact.denotation);
}

#[test]
fn flow_merge_drops_different_family_capabilities() {
    let mut store = TypeStore::new();
    let owner = DeclarationId::new(ModuleId::universe_root(), "Option".into());
    let family = AssociatedFamilyId::new(owner.clone(), SelectorBase::Named("make".into()));
    let owner_form = store.nominal_type(owner.clone());
    let left = SemanticDenotation::AssociatedValue(Box::new(AssociatedValueDenotation::family(
        owner_form,
        owner.clone(),
        family.clone(),
        vec![CapturedAssociatedMember {
            operation: FamilyOperationShape::getter(),
            member: phalcom_semantic::AssociatedMemberId::Variant(VariantId::new(owner.clone(), Selector::getter("None").expect("None"))),
            target: None,
        }],
    )));
    let right = SemanticDenotation::AssociatedValue(Box::new(AssociatedValueDenotation::family(
        owner_form,
        owner.clone(),
        family,
        vec![CapturedAssociatedMember {
            operation: FamilyOperationShape::method(Vec::new().into_boxed_slice()),
            member: phalcom_semantic::AssociatedMemberId::Variant(VariantId::new(owner, Selector::method("None", []).expect("None()"))),
            target: None,
        }],
    )));
    let knowledge = TypeKnowledge::established(store.unit(), EvidenceOrigin::DeclarationSemantics);
    let left_fact = ValueSemanticFact {
        knowledge: knowledge.clone(),
        denotation: Some(left),
    };
    let right_fact = ValueSemanticFact {
        knowledge: knowledge.clone(),
        denotation: Some(right),
    };

    assert_eq!(ValueSemanticFact::merge(&left_fact, &right_fact, knowledge).denotation, None);
}
