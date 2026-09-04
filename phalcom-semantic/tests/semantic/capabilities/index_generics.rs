//! Generic index getter/setter application regressions.

use crate::semantic::support::{Fixture, applied, nominal};
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DispatchSide};
use phalcom_semantic::types::parameter::TypeParameterOwner;

fn index_getter_selector() -> Selector {
    Selector::subscript_get(vec![SelectorSlot::Positional]).unwrap()
}

fn index_setter_selector() -> Selector {
    Selector::subscript_set(vec![SelectorSlot::Positional]).unwrap()
}

#[test]
fn generic_index_getter_infers_key_and_result() {
    let fixture = Fixture::new(
        r#"
class Store {
  [_ key: U]<U> -> Pair<Int, U> { 0 }
  run(store: Store) {
    let value: Pair<Int, String> = store["text"]
  }
}
class Pair<A, B> {}
"#,
    );
    let run = fixture.callable("Store", "run", DispatchSide::Instance);
    let index = fixture.expression(run, "store[\"text\"]");
    fixture.assert_type(
        index.knowledge.ty().expect("generic index result"),
        applied("Pair", [nominal("Int"), nominal("String")]),
    );
    assert!(matches!(index.status, AnalysisStatus::Ready), "{index:#?}");

    let callable = CallableId::new(fixture.decl("Store"), index_getter_selector(), DispatchSide::Instance);
    let signature = fixture.analysis.snapshot.callable_signatures.get(&callable).expect("index getter signature");
    let generic = signature.generics.as_ref().expect("index getter-local generic signature");
    assert_eq!(generic.parameter_count(), 1);
    assert!(matches!(generic.owner, TypeParameterOwner::Callable(_)));
}

#[test]
fn generic_index_setter_uses_key_and_put_value_as_arguments() {
    let fixture = Fixture::new(
        r#"
class Store {
  [_ key: U]<U>=(put value: U) { }
  run(store: Store) { store[1] = "wrong" }
}
"#,
    );
    let run = fixture.callable("Store", "run", DispatchSide::Instance);
    let assignment = fixture.expression(run, "store[1] = \"wrong\"");
    assert_eq!(assignment.knowledge.ty(), Some(fixture.analysis.snapshot.store.unit()), "{assignment:#?}");
    assert!(matches!(assignment.status, AnalysisStatus::Invalid(_)), "{assignment:#?}");
    fixture.assert_diagnostic(DiagnosticCode::GenericInferenceConflict, 1);

    let callable = CallableId::new(fixture.decl("Store"), index_setter_selector(), DispatchSide::Instance);
    let signature = fixture.analysis.snapshot.callable_signatures.get(&callable).expect("index setter signature");
    let generic = signature.generics.as_ref().expect("index setter-local generic signature");
    assert_eq!(generic.parameter_count(), 1);
    assert!(matches!(generic.owner, TypeParameterOwner::Callable(_)));
}

#[test]
fn generic_index_instantiations_keep_subscript_selector_shape() {
    let fixture = Fixture::new(
        r#"
class Store {
  [_ key: U]<U> -> U { key }
  run(store: Store) {
    store[1]
    store["text"]
  }
}
"#,
    );
    let run = fixture.callable("Store", "run", DispatchSide::Instance);
    let first = fixture.expression(run, "store[1]");
    let second = fixture.expression(run, "store[\"text\"]");
    assert_eq!(first.callable, second.callable);
    assert_eq!(first.callable.as_ref().map(|callable| callable.selector.clone()), Some(index_getter_selector()));
}
