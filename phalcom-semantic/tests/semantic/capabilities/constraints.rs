//! Source-level generic constraint ownership and admissibility laws.

use crate::semantic::support::{Fixture, applied, nominal};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::parameter::{GenericConstraint, TypeParameterOwner, TypeTerm};
use phalcom_semantic::types::store::TypeData;

#[test]
fn source_constraints_preserve_class_and_callable_owners() {
    let fixture = Fixture::new(
        r#"
class Number {}
class Comparable<T> {}

class Probe<T> where T <: Number {
  @class
  same<A, B>(_ left: A, _ right: B) -> A where A == B { left }

  @class
  bounded<U>(_ value: U) -> U where U <: Comparable<U> { value }
}

class Container<T> {
  keep<U>(_ value: U) -> U where U <: T { value }
}
"#,
    );

    let probe = fixture.decl("Probe");
    let probe_info = fixture.analysis.snapshot.declarations.get(&probe).expect("Probe declaration");
    let class_signature = probe_info.generic_signature.as_ref().expect("class generic signature");
    assert!(matches!(class_signature.owner, TypeParameterOwner::Declaration(ref owner) if owner == &probe));
    assert_eq!(class_signature.parameters.len(), 1);
    assert_eq!(class_signature.constraints.len(), 1);
    let class_parameter = class_signature.parameters[0];
    assert!(matches!(
        &class_signature.constraints[0],
        GenericConstraint::Subtype {
            lower: TypeTerm::Canonical(lower),
            upper: TypeTerm::Canonical(upper),
        } if matches!(fixture.analysis.snapshot.store.get(*lower), TypeData::Parameter(parameter) if *parameter == class_parameter)
            && *upper == fixture.ty("Number")
    ));

    let same = fixture.callable("Probe", "same", DispatchSide::Class);
    let same_signature = fixture.analysis.snapshot.callable_signatures.get(&same.callable).expect("same signature");
    let same_generics = same_signature.generics.as_ref().expect("same generic signature");
    assert!(matches!(same_generics.owner, TypeParameterOwner::Callable(ref owner) if owner == &same.callable));
    assert_eq!(same_generics.parameters.len(), 2);
    assert_eq!(same_generics.constraints.len(), 1);
    let left_parameter = same_generics.parameters[0];
    let right_parameter = same_generics.parameters[1];
    assert!(matches!(
        &same_generics.constraints[0],
        GenericConstraint::Equivalent {
            left: TypeTerm::Canonical(actual_left),
            right: TypeTerm::Canonical(actual_right),
        } if matches!(fixture.analysis.snapshot.store.get(*actual_left), TypeData::Parameter(parameter) if *parameter == left_parameter)
            && matches!(fixture.analysis.snapshot.store.get(*actual_right), TypeData::Parameter(parameter) if *parameter == right_parameter)
    ));

    let bounded = fixture.callable("Probe", "bounded", DispatchSide::Class);
    let bounded_signature = fixture.analysis.snapshot.callable_signatures.get(&bounded.callable).expect("bounded signature");
    let bounded_generics = bounded_signature.generics.as_ref().expect("bounded generic signature");
    let bounded_parameter = bounded_generics.parameters[0];
    assert!(matches!(
        &bounded_generics.constraints[0],
        GenericConstraint::Subtype {
            lower: TypeTerm::Canonical(lower),
            upper: TypeTerm::Canonical(upper),
        } if matches!(fixture.analysis.snapshot.store.get(*lower), TypeData::Parameter(parameter) if *parameter == bounded_parameter)
            && matches!(fixture.analysis.snapshot.store.get(*upper), TypeData::Applied { .. })
    ));

    let container = fixture.callable("Container", "keep", DispatchSide::Instance);
    let container_signature = fixture
        .analysis
        .snapshot
        .callable_signatures
        .get(&container.callable)
        .expect("Container.keep signature");
    let container_generics = container_signature.generics.as_ref().expect("Container.keep generic signature");
    assert!(matches!(container_generics.owner, TypeParameterOwner::Callable(ref owner) if owner == &container.callable));
    let method_parameter = container_generics.parameters[0];
    let owner_parameter = fixture
        .analysis
        .snapshot
        .declarations
        .get(&fixture.decl("Container"))
        .expect("Container declaration")
        .generic_signature
        .as_ref()
        .expect("Container generic signature")
        .parameters[0];
    assert!(matches!(
        &container_generics.constraints[0],
        GenericConstraint::Subtype {
            lower: TypeTerm::Canonical(lower),
            upper: TypeTerm::Canonical(upper),
        } if matches!(fixture.analysis.snapshot.store.get(*lower), TypeData::Parameter(parameter) if *parameter == method_parameter)
            && matches!(fixture.analysis.snapshot.store.get(*upper), TypeData::Parameter(parameter) if *parameter == owner_parameter)
    ));
}

#[test]
fn source_equivalence_and_lower_bound_constraints_control_calls() {
    let fixture = Fixture::new(
        r#"
class Base {}
class Derived is Base {
  @constructor
  new() {}
}
class Other {
  @constructor
  new() {}
}

class Probe {
  @class
  same<A, B>(_ left: A, _ right: B) -> A where A == B { left }

  @class
  widen<T>(_ value: T) -> T where Base <: T, T <: Object { value }

  @class
  narrow<T>(_ value: T) -> T where T <: Base { value }

  @class
  run(_ value: Base) {
    let equal = Probe.same(1, 1)
    let unequal = Probe.same(1, "wrong")
    let lower_ok = Probe.widen(value)
    let upper_bad = Probe.narrow(Other.new())
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let widen = fixture.callable("Probe", "widen", DispatchSide::Class);
    let widen_signature = fixture.analysis.snapshot.callable_signatures.get(&widen.callable).expect("widen signature");
    let widen_generics = widen_signature.generics.as_ref().expect("widen generic signature");
    assert_eq!(widen_generics.constraints.len(), 2);
    assert!(matches!(
        &widen_generics.constraints[0],
        GenericConstraint::Subtype {
            lower: TypeTerm::Canonical(lower),
            upper: TypeTerm::Canonical(upper),
        } if matches!(fixture.analysis.snapshot.store.get(*lower), TypeData::Nominal { declaration } if declaration.name.as_ref() == "Base")
            && matches!(fixture.analysis.snapshot.store.get(*upper), TypeData::Parameter(parameter) if *parameter == widen_generics.parameters[0])
    ));
    assert!(matches!(
        &widen_generics.constraints[1],
        GenericConstraint::Subtype {
            lower: TypeTerm::Canonical(lower),
            upper: TypeTerm::Canonical(upper),
        } if matches!(fixture.analysis.snapshot.store.get(*lower), TypeData::Parameter(parameter) if *parameter == widen_generics.parameters[0])
            && *upper == fixture.ty("Object")
    ));
    let equal = fixture.expression(run, "Probe.same(1, 1)");
    assert!(matches!(equal.status, AnalysisStatus::Ready), "equal call: {equal:#?}");
    assert_eq!(equal.knowledge.ty(), Some(fixture.ty("Int")));

    let unequal = fixture.expression(run, "Probe.same(1, \"wrong\")");
    assert!(matches!(unequal.status, AnalysisStatus::Invalid(_)), "unequal call: {unequal:#?}");
    assert!(unequal.knowledge.ty().is_none());

    let lower_ok = fixture.expression(run, "Probe.widen(value)");
    assert!(matches!(lower_ok.status, AnalysisStatus::Ready), "lower-bound success: {lower_ok:#?}");
    assert_eq!(lower_ok.knowledge.ty(), Some(fixture.ty("Base")));

    let upper_bad = fixture.expression(run, "Probe.narrow(Other.new())");
    assert!(matches!(upper_bad.status, AnalysisStatus::Invalid(_)), "upper-bound failure: {upper_bad:#?}");
    assert!(upper_bad.knowledge.ty().is_none());
    assert_eq!(fixture.diagnostics(DiagnosticCode::GenericConstraintUnsatisfied).len(), 2);
}

#[test]
fn source_generic_superclass_constraint_substitutes_owner_parameter() {
    let fixture = Fixture::new(
        r#"
class Parent<T> {}
class Child<T> is Parent<T> where T <: Number {}

class Probe {
  @class
  keep<F: Type -> Type, A>(_ value: F<A>) -> F<A> where F<A> <: Parent<A> { value }

  @class
  run(_ value: Child<Int>) {
    let result = Probe.keep(value)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let result = fixture.expression(run, "Probe.keep(value)");
    assert!(matches!(result.status, AnalysisStatus::Ready), "superclass projection: {result:#?}");
    fixture.assert_type(result.knowledge.ty().expect("Child<Int> result"), applied("Child", [nominal("Int")]));
}
