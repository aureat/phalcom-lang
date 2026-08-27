//! Dynamic and reflective boundary capability probes.

use crate::semantic::support::Fixture;
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::{DynamicReason, TypeKnowledge};

/// COMPOSED: an unresolved spread call stays dynamic without weakening an independent sibling fact.
#[test]
fn dynamic_spread_preserves_independent_known_fact() {
    let f = Fixture::new(
        r#"
class Receiver {
  target(_ value: Int) -> Int { value }
}

class Probe {
  @class
  run(_ receiver: Receiver, values) {
    let known = 42
    let spread = receiver.target(*values)
    known
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let spread = f.expression(run, "receiver.target(*values)");

    assert_eq!(spread.knowledge, TypeKnowledge::Dynamic(DynamicReason::DynamicRestPack));
    assert_eq!(spread.status, AnalysisStatus::DynamicBoundary(DynamicReason::DynamicRestPack));
    f.assert_binding_established(run, "known", int_ty);
    f.assert_normal_return(
        run,
        crate::semantic::support::known(int_ty)
            .established()
            .origin(phalcom_semantic::EvidenceOrigin::Syntax),
    );
    f.assert_no_error_diagnostics();
}

/// COMPOSED: reflective dispatch with a dynamic outgoing pack must stay opaque.
#[test]
fn reflective_dynamic_pack_stays_conservative_but_keeps_known_fact() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ target: Object, args) {
    let known = 42
    let reflected = target.perform(Symbol.new("add(_,_)"), ***args)
    known
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let reflected = f.expression(run, "target.perform(Symbol.new(\"add(_,_)\"), ***args)");

    assert!(
        matches!(
            reflected.knowledge,
            TypeKnowledge::Dynamic(DynamicReason::RuntimeReflection | DynamicReason::DynamicRestPack)
        ),
        "reflection must not fabricate a concrete return: {reflected:#?}"
    );
    assert!(matches!(reflected.status, AnalysisStatus::DynamicBoundary(_)));
    f.assert_binding_established(run, "known", int_ty);
    f.assert_no_error_diagnostics();
}
