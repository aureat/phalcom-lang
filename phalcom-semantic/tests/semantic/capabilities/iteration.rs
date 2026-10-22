use crate::semantic::support::{Fixture, assert_source_contract};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::{EvidenceStatus, TypeKnowledge, UnknownReason};
use phalcom_semantic::types::store::TypeData;

/// Legacy cursor-shaped members are not a substitute for the canonical traits.
#[test]
fn custom_iterable_element_type_comes_from_protocol_not_first_generic_argument() {
    let f = Fixture::new(
        r#"
class Weird<A, B> {
  iteratorValue(_ cursor: Int) -> B {
    mystery()
  }
}
class Probe {
  @class
  run(_ weird: Weird<String, Int>) {
    for value in weird {
      let observed = value
    }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(f.binding(run, "value").current, TypeKnowledge::Unknown(UnknownReason::DynamicMessageSend));
    assert_eq!(f.binding(run, "observed").current, TypeKnowledge::Unknown(UnknownReason::DynamicMessageSend));
    assert!(run.for_iteration_lanes.is_empty(), "legacy cursor protocol must not publish a canonical lane");
}
/// LAW: branch joins compose nested collection element precision.
#[test]
fn constructor_branch_nested_inside_collection_preserves_composed_specific_type() {
    let f = Fixture::new(
        r#"
class Animal {}
class Cat is Animal { @constructor new() {} }
class Dog is Animal { @constructor new() {} }
class Probe {
  @class
  run(_ flag: Bool) {
    let xs = if flag {
      [Cat.new()]
    } else {
      [Dog.new()]
    }
  }
}
"#,
    );
    let cat = f.ty("Cat");
    let dog = f.ty("Dog");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let xs_ty = f.binding(run, "xs").current.ty().expect("branch collection type");
    match f.analysis.snapshot.store.get(xs_ty) {
        phalcom_semantic::types::store::TypeData::Applied { arguments, .. } => {
            assert_eq!(arguments.len(), 1);
            f.assert_union_members(arguments[0].as_type_id().expect("type-only collection argument"), &[cat, dog]);
        }
        phalcom_semantic::types::store::TypeData::Union(members) => {
            assert_eq!(members.len(), 2, "alternative normalization should retain both collection branches");
        }
        other => panic!("expected joined collection type, got {other:?}"),
    }
}

/// LAW: unknown branch evidence weakens a declared result instead of becoming established.
#[test]
fn formal_unknown_branch_with_declared_contract_remains_assumed_not_established() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x: Number = if flag {
      42
    } else {
      mystery()
    }
  }
}
"#,
    );
    let number = f.ty("Number");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let x = f.binding(run, "x");
    assert_source_contract(x, number);
    assert_ne!(
        x.current.status(),
        Some(EvidenceStatus::Established),
        "reachable unknown evidence must not be laundered into an established Number fact: {x:#?}"
    );
    if x.current.ty() == Some(number) {
        assert_eq!(x.current.status(), Some(EvidenceStatus::Assumed));
    }
    f.assert_no_error_diagnostics();
}

/// I02: a legacy cursor receiver does not publish a canonical iteration lane.
#[test]
fn generic_iterator_receiver_substitution_selects_second_parameter() {
    let f = Fixture::new(
        r#"
class Weird<A, B> {
  iteratorValue(_ cursor: Int) -> B { mystery() }
}
class Probe {
  @class
  run(_ weird: Weird<Int, String>) {
    for value in weird {
      let observed = value
    }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(f.binding(run, "value").current, TypeKnowledge::Unknown(UnknownReason::DynamicMessageSend));
    assert_eq!(f.binding(run, "observed").current, TypeKnowledge::Unknown(UnknownReason::DynamicMessageSend));
    assert!(run.for_iteration_lanes.is_empty(), "legacy cursor protocol must not publish a canonical lane");
}

/// I03: structured legacy cursor iteration remains outside the canonical lane.
#[test]
fn for_tuple_pattern_decomposes_protocol_element_type() {
    let f = Fixture::new(
        r#"
class Entries {
  iteratorValue(_ cursor: Int) -> (String, Int) { mystery() }
}
class Probe {
  @class
  run(_ entries: Entries) {
    for (key, value) in entries {
      let observedKey = key
      let observedValue = value
    }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let unknown = TypeKnowledge::Unknown(UnknownReason::DynamicMessageSend);
    assert_eq!(f.binding(run, "key").current, unknown);
    assert_eq!(f.binding(run, "value").current, unknown);
    assert_eq!(f.binding(run, "observedKey").current, unknown);
    assert_eq!(f.binding(run, "observedValue").current, unknown);
    assert!(run.for_iteration_lanes.is_empty(), "legacy cursor protocol must not publish a canonical lane");
}

/// I05: a collection protocol without formal element evidence stays incomplete.
#[test]
fn heterogeneous_collection_iteration_preserves_element_union() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let values = [1, "text"]
    for value in values {
      let observed = value
    }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let value = f.binding(run, "value");
    assert!(value.current.is_unknown() || value.current.is_dynamic());
    let observed = f.binding(run, "observed");
    assert!(observed.current.is_unknown() || observed.current.is_dynamic());
}

#[test]
fn iterator_adapter_fixture_preserves_item_projections_and_chain_equality() {
    let source = r##"
trait Iterator {
  type Item
  next -> Option<Self::Item>
  flatMap<S>(_ f: (Self::Item) -> S) -> FlatMapIterator<Self, S> where S |= Iterable { FlatMapIterator.new(self, f) }
  flatten -> FlattenIterator<Self> where Self::Item |= Iterable { FlattenIterator.new(self) }
  chain<J>(_ other: J) -> ChainIterator<Self, J> where J |= Iterator, <J as Iterator>::Item == Self::Item { ChainIterator.new(self, other) }
  zip<J>(_ other: J) -> StrictZipIterator<Self, J> where J |= Iterator { StrictZipIterator.new(self, other) }
}
trait Iterable {
  type Item
  type Iterator
  iterator -> Self::Iterator
  indexed -> IndexedIterator<Self::Iterator> { IndexedIterator.new(self.iterator) }
}
class NumberValue {}
class TextValue {}
class IntIterator { @constructor new() {} }
impl Iterator for IntIterator { type Item = NumberValue
  next -> Option<NumberValue> { None }
}
class OtherIntIterator { @constructor new() {} }
impl Iterator for OtherIntIterator { type Item = NumberValue
  next -> Option<NumberValue> { None }
}
class TextIterator { @constructor new() {} }
impl Iterator for TextIterator { type Item = TextValue
  next -> Option<TextValue> { None }
}
class TextValues { @constructor new() {} }
impl Iterable for TextValues { type Item = TextValue
  type Iterator = TextIterator
  iterator -> TextIterator { TextIterator.new }
}
class FlatMapIterator<I, S> {
  @constructor
  new(_ upstream: I, _ mapper: (<I as Iterator>::Item) -> S) where I |= Iterator, S |= Iterable {}
  next -> Option<<S as Iterable>::Item> { None }
}
impl<I, S> Iterator for FlatMapIterator<I, S> where I |= Iterator, S |= Iterable {
  type Item = <S as Iterable>::Item
}
class FlattenIterator<I> {
  @constructor
  new(_ upstream: I) where I |= Iterator, <I as Iterator>::Item |= Iterable {}
  next -> Option<< <I as Iterator>::Item as Iterable>::Item> { None }
}
impl<I> Iterator for FlattenIterator<I> where I |= Iterator, <I as Iterator>::Item |= Iterable {
  type Item = < <I as Iterator>::Item as Iterable>::Item
}
class ChainIterator<I, J> {
  @constructor
  new(_ first: I, _ second: J) where I |= Iterator, J |= Iterator, <J as Iterator>::Item == <I as Iterator>::Item {}
  next -> Option<<I as Iterator>::Item> { None }
}
impl<I, J> Iterator for ChainIterator<I, J> where I |= Iterator, J |= Iterator, <J as Iterator>::Item == <I as Iterator>::Item {
  type Item = <I as Iterator>::Item
}
class StrictZipIterator<I, J> {
  @constructor
  new(_ left: I, _ right: J) where I |= Iterator, J |= Iterator {}
  next -> Option<(<I as Iterator>::Item, <J as Iterator>::Item)> { None }
}
impl<I, J> Iterator for StrictZipIterator<I, J> where I |= Iterator, J |= Iterator {
  type Item = (<I as Iterator>::Item, <J as Iterator>::Item)
}
class IndexedIterator<I> {
  @constructor
  new(_ upstream: I) where I |= Iterator {}
  next -> Option<(Int, <I as Iterator>::Item)> { None }
}
impl<I> Iterator for IndexedIterator<I> where I |= Iterator {
  type Item = (Int, <I as Iterator>::Item)
}
class Probe {
  flat(_ source: FlatMapIterator<IntIterator, TextValues>) { source.next }
  flattened(_ source: FlattenIterator<IntIterator>) { source.next }
  zipped(_ source: StrictZipIterator<IntIterator, TextIterator>) { source.next }
  indexed(_ source: IndexedIterator<IntIterator>) { source.next }
  chained(_ left: IntIterator, _ right: OtherIntIterator) -> ChainIterator<IntIterator, OtherIntIterator> { left.chain(right) }
}
"##;
    let fixture = Fixture::new(source);
    let return_type = |name: &str| {
        fixture.callable("Probe", name, DispatchSide::Instance).exits.normal_returns[0]
            .knowledge
            .ty()
            .expect("adapter expression has a known return type")
    };
    let option_payload = |ty| {
        let TypeData::Applied { origin, arguments } = fixture.analysis.snapshot.store.get(ty) else {
            panic!("expected Option application, got {:?}", fixture.analysis.snapshot.store.get(ty));
        };
        assert_eq!(*origin, fixture.ty("Option"));
        arguments[0].as_type_id().expect("Option argument is a type")
    };
    let assert_projection = |ty, subject, trait_name: &str| {
        let TypeData::AssociatedProjection(projection) = fixture.analysis.snapshot.store.get(ty) else {
            panic!("expected canonical associated projection, got {:?}", fixture.analysis.snapshot.store.get(ty));
        };
        let owner = fixture.decl(trait_name);
        assert_eq!(projection.subject, subject);
        assert_eq!(projection.trait_ref.declaration, owner);
        assert_eq!(projection.requirement.owner, owner);
        assert_eq!(projection.requirement.index, 0);
    };

    // These source-level fixture declarations validate generic projection
    // formation/substitution independently of canonical Universe bootstrap.
    assert_projection(option_payload(return_type("flat")), fixture.ty("TextValues"), "Iterable");

    let flattened_item = option_payload(return_type("flattened"));
    let TypeData::AssociatedProjection(flattened) = fixture.analysis.snapshot.store.get(flattened_item) else {
        panic!("expected outer Iterable::Item projection");
    };
    assert_eq!(flattened.trait_ref.declaration, fixture.decl("Iterable"));
    assert_eq!(flattened.requirement.owner, fixture.decl("Iterable"));
    let TypeData::AssociatedProjection(upstream_item) = fixture.analysis.snapshot.store.get(flattened.subject) else {
        panic!("expected nested Iterator::Item projection");
    };
    assert_eq!(upstream_item.subject, fixture.ty("IntIterator"));
    assert_eq!(upstream_item.trait_ref.declaration, fixture.decl("Iterator"));
    assert_eq!(upstream_item.requirement.owner, fixture.decl("Iterator"));

    let zipped_item = option_payload(return_type("zipped"));
    let TypeData::Tuple(zipped) = fixture.analysis.snapshot.store.get(zipped_item) else {
        panic!("expected precise zip item tuple");
    };
    assert_eq!(zipped.elements.len(), 2);
    assert_projection(zipped.elements[0].ty, fixture.ty("IntIterator"), "Iterator");
    assert_projection(zipped.elements[1].ty, fixture.ty("TextIterator"), "Iterator");

    let indexed_item = option_payload(return_type("indexed"));
    let TypeData::Tuple(indexed) = fixture.analysis.snapshot.store.get(indexed_item) else {
        panic!("expected precise indexed item tuple");
    };
    assert_eq!(indexed.elements.len(), 2);
    assert_eq!(indexed.elements[0].ty, fixture.ty("Int"));
    assert_projection(indexed.elements[1].ty, fixture.ty("IntIterator"), "Iterator");

    let invalid = Fixture::new_allowing_internal_incidents(
        r#"
trait Iterator {
  type Item
  next -> Option<Self::Item>
  chain<J>(_ other: J) -> ChainIterator<Self, J> where J |= Iterator, <J as Iterator>::Item == Self::Item { ChainIterator.new(self, other) }
}
class NumberValue {}
class TextValue {}
class Left { @constructor new() {} }
impl Iterator for Left { type Item = NumberValue
  next -> Option<NumberValue> { None }
}
class Right { @constructor new() {} }
impl Iterator for Right { type Item = TextValue
  next -> Option<TextValue> { None }
}
class ChainIterator<I, J> {
  @constructor
  new(_ first: I, _ second: J) where I |= Iterator, J |= Iterator, <J as Iterator>::Item == <I as Iterator>::Item {}
}
class Probe {
  invalid(_ left: Left, _ right: Right) { left.chain(right) }
}
"#,
    );
    assert!(invalid.analysis.snapshot.has_errors(), "heterogeneous Item types must fail the chain contract");
    let chain_diagnostic = invalid
        .analysis
        .snapshot
        .diagnostics
        .values()
        .flat_map(|diagnostics| diagnostics.iter())
        .find(|diagnostic| invalid.source.get(diagnostic.primary_range.start..diagnostic.primary_range.end) == Some("left.chain(right)"))
        .expect("heterogeneous chain must be rejected at the call site");
    assert_eq!(
        chain_diagnostic.code,
        phalcom_semantic::diagnostic::DiagnosticCode::GenericConstraintUnsatisfied,
        "unexpected chain rejection: {chain_diagnostic:#?}"
    );
}
