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
    let clock: Clock = System.clock
    let instant: Instant = clock.now
    let elapsed: Duration = instant.elapsed
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    assert_source_contract(&fixture.binding(run, "clock"), fixture.ty("Clock"));
    assert_source_contract(&fixture.binding(run, "instant"), fixture.ty("Instant"));
    assert_source_contract(&fixture.binding(run, "elapsed"), fixture.ty("Duration"));
}

#[test]
fn monotonic_time_types_are_source_owned_prelude_declarations() {
    let fixture = Fixture::new("class Probe {}\n");
    for name in ["Clock", "Instant", "Duration"] {
        assert_eq!(
            fixture.decl(name).module,
            phalcom_modules::ModuleId::universe(phalcom_modules::ModulePath::from_components(vec![
                phalcom_modules::ModuleComponent::from_identifier("time").unwrap(),
                phalcom_modules::ModuleComponent::from_identifier("clock").unwrap(),
            ]))
        );
    }
}
