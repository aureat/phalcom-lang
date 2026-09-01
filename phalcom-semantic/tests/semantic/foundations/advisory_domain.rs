use std::sync::Arc;

use phalcom_common::selector::{Selector, SelectorBase, SelectorKind, SelectorSlot};
use phalcom_semantic::{
    AdvisoryCallableSummary, AdvisoryConfidence, AdvisoryContributionSource, AdvisoryFact, AdvisoryOrigin, AdvisoryParameterContributions,
    AdvisoryParameterSlot, AdvisoryProductStatus, AdvisorySummaryEffects, CallableId, DeclarationId, DispatchSide, ModuleId, SourceOwner, SourceSiteId,
    SourceSiteLocalId, ValueShape,
};

fn declaration(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::universe_root(), name.into())
}

fn site(local: u32) -> SourceSiteId {
    SourceSiteId {
        owner: SourceOwner::Module(ModuleId::universe_root()),
        local: SourceSiteLocalId(local),
    }
}

#[test]
fn record_and_union_shapes_are_canonical() {
    let int = ValueShape::Instance(declaration("Int"));
    let string = ValueShape::Instance(declaration("String"));
    let left = ValueShape::record([("z", string.clone()), ("a", int.clone())]);
    let right = ValueShape::record([("a", int.clone()), ("z", string.clone())]);
    assert_eq!(left, right);

    let first = ValueShape::bounded_union([int.clone(), string.clone()]);
    let second = ValueShape::bounded_union([string, int]);
    assert_eq!(first, second);
}

#[test]
fn compatible_collections_join_and_incompatible_lists_widen() {
    let int = ValueShape::Instance(declaration("Int"));
    let string = ValueShape::Instance(declaration("String"));
    assert_eq!(
        ValueShape::ExactList(Arc::from(vec![int.clone(), string].into_boxed_slice()))
            .join(&ValueShape::ExactList(Arc::from(vec![int.clone(), int.clone()].into_boxed_slice()))),
        ValueShape::ExactList(Arc::from(
            vec![int.clone(), ValueShape::bounded_union([declaration_shape("String"), int.clone()])].into_boxed_slice()
        ))
    );
    assert!(matches!(
        ValueShape::ExactList(Arc::from(vec![int.clone()].into_boxed_slice()))
            .join(&ValueShape::ExactList(Arc::from(vec![int, declaration_shape("String")].into_boxed_slice()))),
        ValueShape::List(_)
    ));
}

fn declaration_shape(name: &str) -> ValueShape {
    ValueShape::Instance(declaration(name))
}

#[test]
fn advisory_fact_keeps_confidence_and_bounded_canonical_provenance() {
    let first = AdvisoryFact::exact(declaration_shape("Int"), AdvisoryOrigin::Syntax(site(2)));
    let second = AdvisoryFact::flow(declaration_shape("String"), AdvisoryOrigin::Binding(site(1)));
    let joined = first.join(&second);
    assert_eq!(joined.confidence, AdvisoryConfidence::Flow);
    assert_eq!(joined.provenance, vec![AdvisoryOrigin::Binding(site(1)), AdvisoryOrigin::Syntax(site(2))]);
}

#[test]
fn method_family_shape_uses_canonical_selector_identity() {
    let selector = Selector::new(SelectorBase::Named("value".into()), SelectorKind::Getter, Box::new([])).unwrap();
    let shape = ValueShape::Selector(selector.clone());
    assert_eq!(shape, ValueShape::Selector(selector));
    let _ = SelectorSlot::Positional;
}

#[test]
fn parameter_contributions_replace_and_remove_only_touched_slots() {
    let callable = CallableId::new(declaration("Worker"), Selector::getter("run").unwrap(), DispatchSide::Instance);
    let slot0 = AdvisoryParameterSlot::new(callable.clone(), 0);
    let slot1 = AdvisoryParameterSlot::new(callable.clone(), 1);
    let int = AdvisoryFact::flow(declaration_shape("Int"), AdvisoryOrigin::Binding(site(1)));
    let string = AdvisoryFact::flow(declaration_shape("String"), AdvisoryOrigin::Binding(site(2)));
    let source_a = AdvisoryContributionSource::Callable(callable.clone());
    let source_b = AdvisoryContributionSource::Module(ModuleId::universe_root());
    let mut contributions = AdvisoryParameterContributions::default();

    assert_eq!(
        contributions
            .replace_source(source_a.clone(), [(slot0.clone(), int.clone()), (slot1.clone(), string)].into())
            .len(),
        2
    );
    assert_eq!(
        contributions
            .replace_source(
                source_b.clone(),
                [(slot0.clone(), AdvisoryFact::flow(declaration_shape("Bool"), AdvisoryOrigin::Binding(site(3))))].into()
            )
            .len(),
        1
    );
    assert!(matches!(contributions.get(&slot0).map(|fact| &fact.shape), Some(ValueShape::Union(_))));
    assert_eq!(contributions.remove_source(&source_b).len(), 1);
    assert_eq!(contributions.get(&slot0), Some(&int));
    assert_eq!(contributions.remove_source(&source_a).len(), 2);
    assert!(contributions.get(&slot0).is_none());
    assert!(contributions.get(&slot1).is_none());
}

#[test]
fn callable_summary_fingerprint_is_deterministic_and_status_explicit() {
    let callable = CallableId::new(declaration("Worker"), Selector::getter("run").unwrap(), DispatchSide::Instance);
    let dependency = CallableId::new(declaration("Helper"), Selector::getter("value").unwrap(), DispatchSide::Instance);
    let parameter = AdvisoryParameterSlot::new(callable.clone(), 0);
    let fact = AdvisoryFact::exact(declaration_shape("Int"), AdvisoryOrigin::Syntax(site(4)));
    let left = AdvisoryCallableSummary::new(
        callable.clone(),
        vec![(parameter.clone(), fact.clone())],
        fact.clone(),
        vec![dependency.clone(), dependency.clone()],
        AdvisorySummaryEffects::default(),
        AdvisoryProductStatus::Complete,
    );
    let right = AdvisoryCallableSummary::new(
        callable,
        vec![(parameter, fact.clone())],
        fact,
        vec![dependency],
        AdvisorySummaryEffects::default(),
        AdvisoryProductStatus::Complete,
    );

    assert_eq!(left, right);
    assert_eq!(left.fingerprint, right.fingerprint);
}
