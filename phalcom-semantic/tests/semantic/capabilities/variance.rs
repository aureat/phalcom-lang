//! Source-level declaration variance and nested `Self` laws.

use crate::semantic::support::{Fixture, applied, nominal};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::parameter::TypeTerm;
use phalcom_semantic::types::store::{TypeData, TypeStore};
use phalcom_semantic::types::variance::{Variance, VarianceStep, compute_variance_occurrence};
use phalcom_semantic::{DeclaredTypeState, is_subtype};

fn applied_type(fixture: &Fixture, store: &mut TypeStore, name: &str, arguments: &[phalcom_semantic::TypeId]) -> phalcom_semantic::TypeId {
    let form = fixture
        .analysis
        .snapshot
        .declarations
        .form(&fixture.decl(name))
        .expect("generic declaration form");
    store.apply_type_form(form, arguments).expect("well-kinded type application")
}

fn relation(
    store: &mut TypeStore,
    hierarchy: &dyn phalcom_semantic::types::TypeHierarchy,
    sub: phalcom_semantic::TypeId,
    sup: phalcom_semantic::TypeId,
) -> bool {
    is_subtype(store, hierarchy, sub, sup)
}

#[test]
fn source_variance_reaches_canonical_relation_and_superclass_projection() {
    let fixture = Fixture::new(
        r#"
class Animal {}
class Cat is Animal {}
class Producer<+T> {}
class Consumer<-T> {}
class Invariant<T> {}
class Parent<+T> {}
class Child<T> is Parent<T> {}
class Probe {
  @class
  accept(_ value: Producer<Animal>) -> Unit { ... }

  @class
  run(_ value: Producer<Cat>) {
    let accepted = Probe.accept(value)
  }
}
"#,
    );
    let cat = fixture.ty("Cat");
    let animal = fixture.ty("Animal");
    let mut store = (*fixture.analysis.snapshot.store).clone();
    let hierarchy = fixture.analysis.snapshot.hierarchy.as_ref();
    assert!(relation(&mut store, hierarchy, cat, animal), "source superclass relation missing");
    assert!(!relation(&mut store, hierarchy, animal, cat), "source superclass relation reversed");

    let producer_cat = applied_type(&fixture, &mut store, "Producer", &[cat]);
    let producer_animal = applied_type(&fixture, &mut store, "Producer", &[animal]);
    assert!(matches!(
        store.get(producer_cat),
        TypeData::Applied { arguments, .. } if arguments.as_ref() == [cat]
    ));
    assert!(matches!(
        store.get(producer_animal),
        TypeData::Applied { arguments, .. } if arguments.as_ref() == [animal]
    ));
    assert_eq!(store.get_parameter_variance(&fixture.decl("Producer"), 0), Some(Variance::Covariant));
    assert!(relation(&mut store, hierarchy, producer_cat, producer_animal));
    assert!(!relation(&mut store, hierarchy, producer_animal, producer_cat));

    let consumer_cat = applied_type(&fixture, &mut store, "Consumer", &[cat]);
    let consumer_animal = applied_type(&fixture, &mut store, "Consumer", &[animal]);
    assert!(relation(&mut store, hierarchy, consumer_animal, consumer_cat));
    assert!(!relation(&mut store, hierarchy, consumer_cat, consumer_animal));

    let invariant_cat = applied_type(&fixture, &mut store, "Invariant", &[cat]);
    let invariant_animal = applied_type(&fixture, &mut store, "Invariant", &[animal]);
    assert!(!relation(&mut store, hierarchy, invariant_cat, invariant_animal));
    assert!(!relation(&mut store, hierarchy, invariant_animal, invariant_cat));

    let child_cat = applied_type(&fixture, &mut store, "Child", &[cat]);
    let parent_animal = applied_type(&fixture, &mut store, "Parent", &[animal]);
    assert!(
        relation(&mut store, hierarchy, child_cat, parent_animal),
        "transformed superclass must retain Parent covariance"
    );

    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let accepted = fixture.binding(run, "accepted").current.ty().expect("covariant callable argument result");
    assert!(matches!(fixture.analysis.snapshot.store.get(accepted), TypeData::Unit));

    for (name, expected) in [
        ("Producer", Variance::Covariant),
        ("Consumer", Variance::Contravariant),
        ("Invariant", Variance::Invariant),
    ] {
        let info = fixture.analysis.snapshot.declarations.get(&fixture.decl(name)).expect("variance declaration");
        let signature = info.generic_signature.as_ref().expect("generic signature");
        assert_eq!(signature.parameter_variances.as_ref(), [expected]);
        let parameter = signature.parameters[0];
        assert_eq!(fixture.analysis.snapshot.store.type_parameter(parameter).variance, expected);
    }
}

#[test]
fn source_nested_callable_occurrence_has_contravariant_polarity() {
    let fixture = Fixture::new(
        r#"
class Consumer<-T> {
  invoke(_ callback: (T) -> Int) -> Unit { ... }
}
"#,
    );
    let info = fixture
        .analysis
        .snapshot
        .declarations
        .get(&fixture.decl("Consumer"))
        .expect("Consumer declaration");
    let parameter = info.generic_signature.as_ref().expect("Consumer generic signature").parameters[0];
    let invoke = fixture.callable("Consumer", "invoke", DispatchSide::Instance);
    let signature = fixture.analysis.snapshot.callable_signatures.get(&invoke.callable).expect("invoke signature");
    let callback = signature
        .parameter_declared_type_at(0)
        .expect("callback parameter")
        .canonical_type()
        .expect("canonical callback type");
    let (actual, path) = compute_variance_occurrence(&fixture.analysis.snapshot.store, parameter, callback, Variance::Covariant, &mut Vec::new())
        .expect("T occurs in callback parameter");
    assert_eq!(actual, Variance::Contravariant);
    assert!(path.iter().any(|step| matches!(step, VarianceStep::CallableParameter { index: 0, .. })));
}

#[test]
fn source_nested_and_higher_kinded_self_specialize_to_receiver() {
    let fixture = Fixture::new(
        r#"
class Box<T> {}
class List<T> {}
class Base<F: Type -> Type> {
  nested() -> Box<Self> { 0 }
  higher() -> F<Self> { 0 }
}
class Child is Base<List> {}

class Probe {
  @class
  run(_ child: Child) {
    let nested = child.nested()
    let higher = child.higher()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    fixture.assert_type(
        fixture.binding(run, "nested").current.ty().expect("nested Self result"),
        applied("Box", [nominal("Child")]),
    );
    fixture.assert_type(
        fixture.binding(run, "higher").current.ty().expect("higher-kinded Self result"),
        applied("List", [nominal("Child")]),
    );
}

#[test]
fn source_class_side_self_is_distinct_from_ambient_class_generic() {
    let fixture = Fixture::new(
        r#"
class Box<T> {}
class Generic<T> {
  @class
  class_self() -> Box<Self> { 0 }

  @class
  class_generic() -> T { 0 }
}
"#,
    );
    let class_self = fixture.callable("Generic", "class_self", DispatchSide::Class);
    let class_self_signature = fixture
        .analysis
        .snapshot
        .callable_signatures
        .get(&class_self.callable)
        .expect("class Self signature");
    let DeclaredTypeState::Known(TypeTerm::Canonical(result)) = &class_self_signature.declared_return.state else {
        panic!("class-side nested Self must publish a canonical type");
    };
    let TypeData::Applied { arguments, .. } = fixture.analysis.snapshot.store.get(*result) else {
        panic!("class-side Self must remain nested in Box");
    };
    assert!(matches!(fixture.analysis.snapshot.store.get(arguments[0]), TypeData::SelfType(term) if term.side == DispatchSide::Class));

    let class_generic = fixture
        .analysis
        .snapshot
        .callable_signatures
        .get(&fixture.callable_id("Generic", "class_generic", DispatchSide::Class))
        .expect("class generic signature");
    let DeclaredTypeState::Known(TypeTerm::Canonical(result)) = &class_generic.declared_return.state else {
        panic!("class-side declaration generic must publish its template");
    };
    assert!(matches!(fixture.analysis.snapshot.store.get(*result), TypeData::Parameter(_)));
}
