use crate::semantic::support::Fixture;

use phalcom_semantic::explain::{ExplanationStep, causal_slice};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::presentation::TypePresenter;
use phalcom_semantic::types::evidence::{EvidenceOrigin, EvidenceStatus};

#[test]
fn prints_constructor_type_derivation() {
    let f = Fixture::new(
        r#"
class CellNum {
  @constructor new(_ value: Int) {}
}

class Probe {
  @class
  run() {
    CellNum.new(42)
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "CellNum.new(42)");

    let explanation_id = call.explanation.expect("constructor call should have an explanation");

    let root = run.explanations.get(explanation_id).expect("explanation node should exist");

    assert_eq!(root.status, EvidenceStatus::Established);
    assert_eq!(root.origin, EvidenceOrigin::ConstructorSemantics);

    let expected_return_ty = call.knowledge.ty().expect("constructor call should have a known return type");

    let presenter = TypePresenter::new(&f.analysis.snapshot.store);
    let return_type = presenter.present_type(expected_return_ty);

    println!();
    println!("Type derivation");
    println!();
    println!("CellNum.new(42)");
    println!("- `CellNum.new` is an @constructor");
    println!("- constructors return `Self`");
    println!("- Self = {return_type}");
    println!("- therefore");
    println!("  CellNum.new(42) : {return_type}");

    println!();
    println!("Raw causal slice:");
    for node in causal_slice(&run.explanations, explanation_id) {
        println!("{node:#?}");
    }

    match &root.step {
        ExplanationStep::MethodCall { callable, return_ty, .. } => {
            assert_eq!(callable.owner.name.as_ref(), "CellNum");
            assert_eq!(*return_ty, expected_return_ty);
        }

        other => panic!("expected MethodCall explanation, got {other:#?}"),
    }
}
