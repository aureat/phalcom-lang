//! SC-4 generic getter applications through canonical callable signatures.

use crate::semantic::support::{Fixture, applied, nominal};
use phalcom_common::selector::Selector;
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DispatchSide};
use phalcom_semantic::types::parameter::TypeParameterOwner;

#[test]
fn contextual_generic_getter_uses_expected_result() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  value<T> -> T { 0 }

  @class
  run() {
    let result: Int = Probe.value
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let access = fixture.expression(run, "Probe.value");
    assert_eq!(access.knowledge.ty(), Some(fixture.ty("Int")), "{access:#?}");
    assert!(matches!(access.status, AnalysisStatus::Ready), "{access:#?}");
}

#[test]
fn generic_getter_without_context_remains_underconstrained() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  value<T> -> T { 0 }

  @class
  run() {
    let result = Probe.value
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let access = fixture.expression(run, "Probe.value");
    assert!(access.knowledge.ty().is_none(), "{access:#?}");
    assert!(matches!(access.status, AnalysisStatus::Blocked(_)), "{access:#?}");
    fixture.assert_diagnostic(DiagnosticCode::GenericInferenceUnderconstrained, 1);
}

#[test]
fn generic_getter_where_bound_accepts_expected_subtype() {
    let fixture = Fixture::new(
        r#"
class Number {}
class Allowed is Number {}
class Probe {
  @class
  value<T> -> T where T <: Number { 0 }

  @class
  run() {
    let result: Allowed = Probe.value
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let access = fixture.expression(run, "Probe.value");
    assert_eq!(access.knowledge.ty(), Some(fixture.ty("Allowed")), "{access:#?}");
    assert!(matches!(access.status, AnalysisStatus::Ready), "{access:#?}");
}

#[test]
fn generic_getter_where_bound_rejects_incompatible_expected_type() {
    let fixture = Fixture::new(
        r#"
class Number {}
class Probe {
  @class
  value<T> -> T where T <: Number { 0 }

  @class
  run() {
    let result: String = Probe.value
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let access = fixture.expression(run, "Probe.value");
    assert!(access.knowledge.ty().is_none(), "invalid getter bound must not publish result: {access:#?}");
    assert!(matches!(access.status, AnalysisStatus::Invalid(_)), "{access:#?}");
    fixture.assert_diagnostic(DiagnosticCode::GenericConstraintUnsatisfied, 1);
}

#[test]
fn inherited_generic_getter_specializes_transformed_receiver() {
    let fixture = Fixture::new(
        r#"
class Pair<T, U> {}
class Parent<T> {
  value<U> -> Pair<T, U> { 0 }
}
class Child is Parent<Int> {}

class Probe {
  @class
  run(_ child: Child) {
    let result: Pair<Int, String> = child.value
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let access = fixture.expression(run, "child.value");
    fixture.assert_type(
        access.knowledge.ty().expect("inherited getter result"),
        applied("Pair", [nominal("Int"), nominal("String")]),
    );
    assert!(matches!(access.status, AnalysisStatus::Ready), "{access:#?}");
}

#[test]
fn generic_getter_self_result_specializes_actual_receiver() {
    let fixture = Fixture::new(
        r#"
class List<T> {}
class Parent<F: Type -> Type> {
  value<T> -> F<Self> { 0 }
}
class Child is Parent<List> {}

class Probe {
  @class
  run(_ child: Child) {
    let result: List<Child> = child.value
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let access = fixture.expression(run, "child.value");
    fixture.assert_type(access.knowledge.ty().expect("getter F<Self> result"), applied("List", [nominal("Child")]));
}

#[test]
fn enum_generic_getter_uses_canonical_application_path() {
    let fixture = Fixture::new(
        r#"
enum Box {
  @class
  value<T> -> T { 0 }
}

class Probe {
  @class
  run() {
    let result: Int = Box.value
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let access = fixture.expression(run, "Box.value");
    assert_eq!(access.knowledge.ty(), Some(fixture.ty("Int")), "{access:#?}");
    assert!(matches!(access.status, AnalysisStatus::Ready), "{access:#?}");
}

#[test]
fn generic_getter_signature_owns_callable_parameters_and_keeps_getter_selector() {
    let fixture = Fixture::new(
        r#"
class Probe {
  value<T> -> T { 0 }
}
"#,
    );
    let owner = fixture.decl("Probe");
    let selector = Selector::getter("value").expect("getter selector");
    let callable = CallableId::new(owner, selector, DispatchSide::Instance);
    let signature = fixture
        .analysis
        .snapshot
        .callable_signatures
        .get(&callable)
        .expect("canonical generic getter signature");
    let generics = signature.generics.as_ref().expect("getter generic signature");
    assert_eq!(generics.parameters.len(), 1);
    assert!(matches!(generics.owner, TypeParameterOwner::Callable(ref owner) if owner == &callable));
    assert_eq!(signature.selector, callable.selector);
}

#[test]
fn class_generic_getter_uses_local_binder_without_ambient_instance_parameter() {
    let fixture = Fixture::new(
        r#"
class Box<T> {
  @class
  value<U> -> U { 0 }
}

class Probe {
  @class
  run() {
    let result: String = Box.value
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let access = fixture.expression(run, "Box.value");
    assert_eq!(access.knowledge.ty(), Some(fixture.ty("String")), "{access:#?}");
    assert!(matches!(access.status, AnalysisStatus::Ready), "{access:#?}");
}
