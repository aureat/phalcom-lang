use std::collections::BTreeMap;

use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_semantic::{
    AdvisoryConfidence, AdvisoryContributionSource, AdvisoryFact, AdvisoryParameterContributions, CallableId, CallableParameterId, DeclarationId, DispatchSide,
    ModuleId, ValueShape,
};

#[test]
fn advisory_parameter_contributions_use_canonical_parameter_identity() {
    let callable = CallableId::new(
        DeclarationId::new(ModuleId::core(), "Probe".into()),
        Selector::method("consume", [SelectorSlot::Positional]).unwrap(),
        DispatchSide::Instance,
    );
    let parameter = CallableParameterId::new(callable.clone(), 0);
    let fact = AdvisoryFact::new(ValueShape::Unknown, AdvisoryConfidence::Interprocedural);
    let mut contributions = AdvisoryParameterContributions::default();

    let deltas = contributions.replace_source(
        AdvisoryContributionSource::Callable(callable),
        BTreeMap::from([(parameter.clone(), fact.clone())]),
    );

    assert_eq!(contributions.get(&parameter), Some(&fact));
    assert_eq!(deltas.len(), 1);
    assert_eq!(deltas[0].slot, parameter);
}
