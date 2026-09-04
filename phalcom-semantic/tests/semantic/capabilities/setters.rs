//! Generic property setters use callable-local generic application.

use crate::semantic::support::Fixture;
use phalcom_common::selector::Selector;
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DispatchSide};
use phalcom_semantic::types::parameter::TypeParameterOwner;

#[test]
fn generic_setter_infers_rhs_and_returns_unit() {
    let fixture = Fixture::new(
        r#"
class Box {
  value<T>=(put next: T) { }
  run() {
    self.value = 1
    self.value = "text"
  }
}
"#,
    );
    let run = fixture.callable("Box", "run", DispatchSide::Instance);
    let first = fixture.expression(run, "self.value = 1");
    let second = fixture.expression(run, "self.value = \"text\"");
    let unit = fixture.analysis.snapshot.store.unit();
    assert_eq!(first.knowledge.ty(), Some(unit), "{first:#?}");
    assert_eq!(second.knowledge.ty(), Some(unit), "{second:#?}");
    assert!(matches!(first.status, AnalysisStatus::Ready), "{first:#?}");
    assert!(matches!(second.status, AnalysisStatus::Ready), "{second:#?}");

    let setter = CallableId::new(fixture.decl("Box"), Selector::setter("value").unwrap(), DispatchSide::Instance);
    assert_eq!(first.callable.as_ref(), Some(&setter));
    assert_eq!(second.callable.as_ref(), Some(&setter));
    let signature = fixture.analysis.snapshot.callable_signatures.get(&setter).expect("generic setter signature");
    let generic = signature.generics.as_ref().expect("setter-local generic signature");
    assert_eq!(generic.parameter_count(), 1);
    assert!(matches!(generic.owner, TypeParameterOwner::Callable(_)));
}

#[test]
fn generic_setter_where_bound_rejects_rhs_without_dynamic_escape() {
    let fixture = Fixture::new(
        r#"
class Number {}
class Box {
  value<T>=(put next: T) where T <: Number { }
  run() { self.value = "wrong" }
}
"#,
    );
    let run = fixture.callable("Box", "run", DispatchSide::Instance);
    let assignment = fixture.expression(run, "self.value = \"wrong\"");
    assert_eq!(assignment.knowledge.ty(), Some(fixture.analysis.snapshot.store.unit()), "{assignment:#?}");
    assert!(matches!(assignment.status, AnalysisStatus::Invalid(_)), "{assignment:#?}");
    fixture.assert_diagnostic(DiagnosticCode::GenericConstraintUnsatisfied, 1);
    assert!(!matches!(assignment.knowledge, phalcom_semantic::types::evidence::TypeKnowledge::Dynamic(_)));
}

#[test]
fn generic_setter_instantiations_keep_one_selector_identity() {
    let fixture = Fixture::new(
        r#"
class Box {
  value<T>=(put next: T) { }
  run() { self.value = 1; self.value = "text" }
}
"#,
    );
    let run = fixture.callable("Box", "run", DispatchSide::Instance);
    let first = fixture.expression(run, "self.value = 1");
    let second = fixture.expression(run, "self.value = \"text\"");
    assert_eq!(first.callable, second.callable);
    assert_eq!(first.callable.as_ref().map(|callable| callable.selector.clone()), Some(Selector::setter("value").unwrap()));
}
