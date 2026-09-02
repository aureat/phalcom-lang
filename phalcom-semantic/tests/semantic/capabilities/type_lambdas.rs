use crate::semantic::support::{applied, nominal, Fixture};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::explain::ExplanationStep;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::store::TypeData;

#[test]
fn generic_constructor_application_publishes_canonical_lambda_candidate() {
    let f = Fixture::new(
        r#"
class Either<L, R> {}

class Probe {
  @class
  keep<F: Type -> Type, A>(_ value: F<A>) -> F<A> { value }

  @class
  run(_ value: Either<String, Int>) {
    let result = Probe.keep(value)
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let result = f.binding(run, "result").current.ty().expect("inferred result");
    f.assert_type(result, applied("Either", [nominal("String"), nominal("Int")]));
    let call = f.expression(run, "Probe.keep(value)");
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
    let trace = f.explanation_trace(run, call);
    assert!(
        trace.iter().any(|node| match node.step {
            ExplanationStep::GenericSolution { ty, .. } => matches!(f.analysis.snapshot.store.get(ty), TypeData::Lambda(_)),
            _ => false,
        }),
        "expected canonical type-lambda candidate in generic solutions: {trace:#?}"
    );
    f.assert_no_error_diagnostics();
}
