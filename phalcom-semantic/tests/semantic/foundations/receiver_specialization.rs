//! SC-2 owner-relative receiver specialization laws.

use crate::semantic::support::{Fixture, applied};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::explain::ExplanationStep;
use phalcom_semantic::identity::DispatchSide;

#[test]
fn direct_generic_receiver_specializes_member_contract() {
    let fixture = Fixture::new(
        r#"
class Box<T> {
  value(_ value: T) -> T { value }
}

class Probe {
  @class
  run(_ box: Box<Int>) {
    let result = box.value(1)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(fixture.binding(run, "result").current.ty(), Some(fixture.ty("Int")));
    assert_eq!(fixture.binding(run, "result").current.status(), Some(phalcom_semantic::EvidenceStatus::Assumed));
}

#[test]
fn transformed_single_hop_receiver_projects_owner_argument() {
    let fixture = Fixture::new(
        r#"
class Parent<T> {
  value() -> T { 1 }
}

class Child<T> is Parent<List<T>> {}

class Probe {
  @class
  run(_ child: Child<Int>) {
    let result = child.value()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let result = fixture.binding(run, "result");
    fixture.assert_type(result.current.ty().expect("projected result"), applied("List", [fixture.ty("Int").into()]));
    fixture.assert_trace_has(run, fixture.expression(run, "child.value()"), |step| {
        matches!(
            step,
            ExplanationStep::CallableSelection {
                declaring_owner,
                specialization_path,
                ..
            } if declaring_owner.name.as_ref() == "Parent"
                && specialization_path.len() == 2
                && specialization_path[0].name.as_ref() == "Child"
                && specialization_path[1].name.as_ref() == "Parent"
        )
    });
}

#[test]
fn class_and_method_generics_specialize_together() {
    let fixture = Fixture::new(
        r#"
class Pairer<T> {
  pair<U>(_ value: U) -> (T, U) { (1, value) }
}

class Probe {
  @class
  run(_ pairer: Pairer<Int>) {
    let result = pairer.pair("text")
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    fixture.assert_tuple_types(
        fixture.binding(run, "result").current.ty().expect("generic pair result"),
        &[fixture.ty("Int"), fixture.ty("String")],
    );
}

#[test]
fn multi_hop_receiver_projects_each_generic_supertype_template() {
    let fixture = Fixture::new(
        r#"
class Base<T> {
  value() -> T { 1 }
}

class Middle<T> is Base<Option<T>> {}
class Leaf<T> is Middle<T> {}

class Probe {
  @class
  run(_ leaf: Leaf<Int>) {
    let result = leaf.value()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let result = fixture.binding(run, "result");
    fixture.assert_type(result.current.ty().expect("projected result"), applied("Option", [fixture.ty("Int").into()]));
}

#[test]
fn receiver_specialization_carries_method_generic_constraints() {
    let fixture = Fixture::new(
        r#"
class Animal {}
class Cat is Animal { @constructor new() {} }

class Holder<T> {
  convert<U>(_ value: U) -> U where U <: T { value }
}

class Probe {
  @class
  run(_ holder: Holder<Animal>, _ cat: Cat) {
    let result = holder.convert(cat)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let result = fixture.binding(run, "result");
    assert_eq!(result.current.ty(), Some(fixture.ty("Cat")), "{result:#?}");
    assert!(matches!(fixture.expression(run, "holder.convert(cat)").status, AnalysisStatus::Ready));
}

#[test]
fn self_inherited_member_uses_actual_receiver_type() {
    let fixture = Fixture::new(
        r#"
class Box<T> {}
class Parent {
  wrap() -> Box<Self> { 1 }
}
class Child is Parent {}

class Probe {
  @class
  run(_ child: Child) {
    let result = child.wrap()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let result = fixture.binding(run, "result");
    fixture.assert_type(result.current.ty().expect("projected result"), applied("Box", [fixture.ty("Child").into()]));
}
