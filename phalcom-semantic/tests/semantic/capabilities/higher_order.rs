//! Higher-order callable capability probes.

use crate::semantic::support::Fixture;
use phalcom_semantic::identity::DispatchSide;

/// COMPOSED: a callback invocation should publish its body result, not only its closure object type.
#[test]
#[ignore = "GATED: source-level closure invocation and callable-result publication are not formal yet"]
fn higher_order_block_call_propagates_captured_result() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  apply(_ value: Int) {
    const increment = || { value + 1 }
    increment.call()
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let apply = f.callable("Probe", "apply", DispatchSide::Class);
    f.assert_expression_established(f.expression(apply, "increment.call()"), int_ty);
    f.assert_normal_return(
        apply,
        crate::semantic::support::known(int_ty)
            .established()
            .origin(phalcom_semantic::EvidenceOrigin::Flow),
    );
    f.assert_no_error_diagnostics();
}
