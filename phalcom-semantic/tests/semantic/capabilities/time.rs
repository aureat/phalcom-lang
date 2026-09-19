//! STDL002.C1.P1 — source-visible monotonic time type contracts.

use crate::semantic::support::{Fixture, assert_source_contract};
use phalcom_semantic::identity::DispatchSide;

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
