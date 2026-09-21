//! STDL002.C1.P1 — source-visible monotonic time type contracts.

use crate::semantic::support::{Fixture, assert_source_contract};
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_semantic::identity::{CallableId, DispatchSide};

#[test]
fn monotonic_time_surface_publishes_expected_types() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let elapsed: Duration = Duration.seconds(1)
    let timestamp: Timestamp = Timestamp.epoch
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    assert_source_contract(fixture.binding(run, "elapsed"), fixture.ty("Duration"));
    assert_source_contract(fixture.binding(run, "timestamp"), fixture.ty("Timestamp"));
}

#[test]
fn monotonic_time_types_are_source_owned_declarations() {
    let fixture = Fixture::new("class Probe {}\n");
    for (name, module_name) in [("Clock", "clock"), ("Instant", "clock"), ("Duration", "duration"), ("Timestamp", "timestamp")] {
        assert_eq!(
            fixture.decl(name).module,
            phalcom_modules::ModuleId::universe(phalcom_modules::ModulePath::from_components(vec![
                phalcom_modules::ModuleComponent::from_identifier("time").unwrap(),
                phalcom_modules::ModuleComponent::from_identifier(module_name).unwrap(),
            ]))
        );
    }
}

#[test]
fn utc_offset_is_source_owned_by_the_canonical_offset_module() {
    let fixture = Fixture::new("class Probe {}\n");
    assert_eq!(
        fixture.decl("UtcOffset").module,
        phalcom_modules::ModuleId::universe(phalcom_modules::ModulePath::from_components(vec![
            phalcom_modules::ModuleComponent::from_identifier("time").unwrap(),
            phalcom_modules::ModuleComponent::from_identifier("offset").unwrap(),
        ]))
    );
}

#[test]
fn canonical_data_class_side_methods_publish_argument_signatures() {
    let fixture = Fixture::new("class Probe {}\n");
    let duration = fixture.decl("Duration");
    let selector = Selector::method("seconds", [SelectorSlot::Positional]).expect("Duration.seconds selector");
    let signature = fixture
        .analysis
        .snapshot
        .dispatch
        .surface(&duration)
        .and_then(|surface| surface.get_callable(DispatchSide::Class, &selector))
        .expect("canonical data surface must publish Duration.seconds(_)");
    assert_eq!(
        signature.parameters[0].ty.ty(),
        Some(fixture.ty("Int")),
        "canonical data method must retain its Int parameter signature: {signature:#?}",
    );
    let callable = CallableId::new(duration, selector, DispatchSide::Class);
    assert!(
        fixture.analysis.snapshot.callable_signatures.get(&callable).is_some(),
        "canonical data class-side method must publish a reusable callable signature",
    );
}
