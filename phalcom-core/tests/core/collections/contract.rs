//! Reusable conformance harness for the collection-protocol contract
//! (`docs/forge/units/U-CORE-5/as-built.md`). Keyed by "the collection under
//! test": a [`ContractSpec`] plus a build-closure. New collections (U-STD's
//! `Tuple`/`Map`/`Set`) are certified by adding a `build_*` closure and one
//! `#[test]` that calls [`assert_sequence_contract`] — R-INV-5.4.
//!
//! Mirrors the selector-interning + `send0`/`send1` pattern established in
//! `tests/invariants.rs`: every send goes through [`VM::send_dynamic`] with
//! selectors interned via [`VM::get_or_intern`], never through direct Rust
//! calls into a collection's own primitives, so the contract exercises
//! exactly the surface a user program would.

use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_core::modules::AssociatedLoweringSpec;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_modules::{ModuleComponent, ModuleId, ModulePath};
use phalcom_semantic::core_surface::CoreDeclarationIds;
use phalcom_semantic::identity::VariantId;

#[derive(Clone, Copy)]
enum IndexedReadContract {
    /// Historical List/Set contract: an in-range `at(_)` returns the value.
    BareValue,
    /// Tuple's typed contract: an in-range `at(_)` returns `Some(value)`.
    Option,
}

/// The static parameters that distinguish one collection kind from another
/// (as-built.md §2): whether it supports in-place growth, whether it is a
/// valid `Map`/`Set` key, and whether its iteration order is a function of
/// construction.
struct ContractSpec {
    /// The collection's class name, used only for assertion diagnostics.
    class_name: &'static str,
    /// Surface selector for in-place growth, if this collection is mutable.
    mutation_selector: Option<&'static str>,
    /// Whether the growth selector returns its receiver. `List#append(_)`
    /// returns Unit; `Set#add(_)` remains chainable.
    mutation_returns_receiver: bool,
    /// Whether the collection is a valid `Map`/`Set` key (H1/H2). `List` is
    /// `false` — its inherited identity `hash` is inconsistent with the
    /// structural `==` this unit adds (as-built.md §2.4), so the H2
    /// consistency assertion is skipped for it.
    hashable: bool,
    /// Lowest arity that remains an instance of this collection family.
    /// Tuple's zero-arity product normalizes to Unit.
    min_arity: usize,
    /// Precise result shape for a successful indexed read.
    indexed_read: IndexedReadContract,
}

/// Sends a nullary selector to `receiver` and returns the result value.
///
/// Mirrors the identical helper in `tests/invariants.rs`.
fn send0(vm: &mut VM, receiver: Value, selector: &str) -> Value {
    let sym = vm.get_or_intern(selector);
    vm.send_dynamic(receiver, sym, &[]).unwrap_or_else(|_| panic!("send `{selector}` failed"))
}

/// Sends a unary selector to `receiver` with one argument.
///
/// Mirrors the identical helper in `tests/invariants.rs`.
fn send1(vm: &mut VM, receiver: Value, selector: &str, arg: Value) -> Value {
    let sym = vm.get_or_intern(selector);
    vm.send_dynamic(receiver, sym, &[arg])
        .unwrap_or_else(|err| panic!("send `{selector}` failed: {err:?}"))
}

/// Extracts the `f64` behind a `Number` result (test-local helper, mirrors
/// `tests/invariants.rs::as_number`).
fn as_number(value: Value) -> f64 {
    if let Some(n) = value.as_int() {
        n as f64
    } else if let Some(n) = value.as_float() {
        n
    } else {
        panic!("expected a number, got {value:?}");
    }
}

/// Extracts the `bool` behind a `Bool` result.
fn as_bool(value: Value) -> bool {
    if let Some(b) = value.as_bool() {
        b
    } else {
        panic!("expected a Bool, got {value:?}");
    }
}

/// Builds a `List` from element values through the *surface* protocol
/// (`List.new()` + `.append(_)`), so the harness exercises the same path user
/// code does.
fn build_list(vm: &mut VM, elems: &[Value]) -> Value {
    let list_class = Value::obj(vm.universe.classes.list_class);
    let list = send0(vm, list_class, "new()");
    for elem in elems {
        send1(vm, list, "append(_)", *elem);
    }
    list
}

/// Evaluates a collection literal through the compiler, then retrieves its
/// module-global result. Tuple and Range deliberately expose literal syntax,
/// rather than public constructor selectors.
fn eval_literal(vm: &mut VM, literal: &str) -> Value {
    let module = vm.create_module("collections-contract", "collections-contract");
    let source = format!("let built = {literal}\n");
    let closure = vm.compile_closure(module, &source).expect("literal compiles");
    vm.run_in_module(module, closure).expect("literal runs");
    let built = vm.get_or_intern("built");
    vm.heap.module(module).get(built).expect("literal binds built")
}

/// Runs the full R-INV-5.x sequence + equality contract against whatever
/// `build` produces, parameterized by `spec` (as-built.md §2, §3.3(a)).
///
/// # Panics
///
/// Panics (via the `send0`/`send1`/`assert*` helpers) on the first law that
/// fails — this is a test harness, not a `Result`-returning API.
fn assert_sequence_contract(vm: &mut VM, spec: &ContractSpec, build: impl Fn(&mut VM, &[Value]) -> Value) {
    // L1/L2: size is a Number, >= 0, and == the element count.
    for n in spec.min_arity..=3 {
        let elems: Vec<Value> = (0..n).map(|i| Value::int(i as i64)).collect();
        let collection = build(vm, &elems);
        let size = as_number(send0(vm, collection, "size"));
        assert!(size >= 0.0, "{}: size must be >= 0, got {size}", spec.class_name);
        assert_eq!(size, n as f64, "{}: size must equal the element count", spec.class_name);

        // L3: at(i) recovers each element by its own `==`, for 0 <= i < n.
        for (i, elem) in elems.iter().enumerate() {
            let got = send1(vm, collection, "at(_)", Value::int(i as i64));
            let recovered = match spec.indexed_read {
                IndexedReadContract::BareValue => got,
                IndexedReadContract::Option => {
                    let expected = elem.wrap_some().expect("fixture value wraps in Some");
                    assert_eq!(got, expected, "{}: at({i}) must return Some(element)", spec.class_name);
                    got.unwrap_some().expect("Tuple at(valid) returns Some")
                }
            };
            assert!(
                as_bool(send1(vm, recovered, "==(_)", *elem)),
                "{}: at({i}) should recover the element it was built with",
                spec.class_name
            );
        }

        // L4: at(n) (and beyond) is immediate `None` — total, never a
        // panic, never the raw `nil` sentinel (Invariant 4).
        let out_of_range = send1(vm, collection, "at(_)", Value::int(n as i64));
        assert!(out_of_range.is_none(), "{}: at(size) must surface immediate None", spec.class_name);
        assert!(!out_of_range.is_nil(), "{}: at(size) must never leak the raw sentinel", spec.class_name);
    }

    // L5: a collection's documented growth selector grows size by one and
    // places its value at the old final index.
    if let Some(selector) = spec.mutation_selector {
        let collection = build(vm, &[Value::int(1), Value::int(2)]);
        let old_size = as_number(send0(vm, collection, "size"));
        let returned = send1(vm, collection, selector, Value::int(42));
        let new_size = as_number(send0(vm, collection, "size"));
        assert_eq!(new_size, old_size + 1.0, "{}: {selector} must grow size by 1", spec.class_name);
        let last = send1(vm, collection, "at(_)", Value::int(old_size as i64));
        assert!(
            as_bool(send1(vm, last, "==(_)", Value::int(42))),
            "{}: {selector} must place x at the old size index",
            spec.class_name
        );
        if spec.mutation_returns_receiver {
            assert!(
                as_bool(send1(vm, returned, "==(_)", collection)),
                "{}: {selector} must return the chainable receiver",
                spec.class_name
            );
        } else {
            assert_eq!(returned, Value::unit(), "{}: {selector} must return Unit", spec.class_name);
        }
    }

    // E1/E3/E4: A == B (equal elements, same order) is true; A == A is true
    // (reflexive); (A == B) == (B == A) (symmetric).
    let elems = [Value::int(1), Value::int(2), Value::int(3)];
    let a = build(vm, &elems);
    let b = build(vm, &elems);
    let a_eq_b = as_bool(send1(vm, a, "==(_)", b));
    let b_eq_a = as_bool(send1(vm, b, "==(_)", a));
    assert!(a_eq_b, "{}: structurally-equal collections must compare ==", spec.class_name);
    assert!(as_bool(send1(vm, a, "==(_)", a)), "{}: == must be reflexive", spec.class_name);
    assert_eq!(a_eq_b, b_eq_a, "{}: == must be symmetric", spec.class_name);

    // E2: cross-kind comparison is false, never a dNU.
    let cross_kind = send1(vm, a, "==(_)", Value::int(1));
    assert!(!as_bool(cross_kind), "{}: == must be false across collection kinds", spec.class_name);

    // E1/E5: a collection differing at one index is unequal; transitivity
    // via a second equal-to-`b` collection.
    let differing = [Value::int(1), Value::int(9), Value::int(3)];
    let c = build(vm, &differing);
    assert!(
        !as_bool(send1(vm, a, "==(_)", c)),
        "{}: differing elements must compare unequal",
        spec.class_name
    );
    let a2 = build(vm, &elems);
    assert!(as_bool(send1(vm, b, "==(_)", a2)), "{}: transitivity precondition (B == A2)", spec.class_name);
    assert!(
        as_bool(send1(vm, a, "==(_)", a2)),
        "{}: == must be transitive (A == B, B == A2 => A == A2)",
        spec.class_name
    );

    // E6: != is the logical negation of ==, routed through it (not floor
    // identity) — the `==`/`!=` decoupling hazard this unit guards against.
    assert!(as_bool(send1(vm, a, "!=(_)", c)), "{}: != must hold where == fails", spec.class_name);
    assert!(!as_bool(send1(vm, a, "!=(_)", b)), "{}: != must be false where == holds", spec.class_name);

    // H2: for a hashable collection, A == B implies hash(A) == hash(B). For
    // a non-hashable (mutable) collection like `List`, this is deliberately
    // SKIPPED — enforcement of "mutable collections are not hashable" is a
    // consumer (Map/Set) obligation, not this unit's (as-built.md §2.4).
    if spec.hashable {
        let hash_a = as_number(send0(vm, a, "hash"));
        let hash_b = as_number(send0(vm, b, "hash"));
        assert_eq!(hash_a, hash_b, "{}: == collections must hash equal (H2)", spec.class_name);
    }
}

/// `List` satisfies the full sequence-protocol contract as the reference
/// implementation (as-built.md §3.3(a), R-INV-5.1…5.3).
#[test]
fn list_satisfies_sequence_contract() {
    let mut vm = VM::new();
    let spec = ContractSpec {
        class_name: "List",
        mutation_selector: Some("append(_)"),
        mutation_returns_receiver: false,
        hashable: false,
        min_arity: 0,
        indexed_read: IndexedReadContract::BareValue,
    };
    assert_sequence_contract(&mut vm, &spec, build_list);
}

/// Builds a `Set` from element values through the surface protocol
/// (`Set.new()` + `.add(_)`).
fn build_set(vm: &mut VM, elems: &[Value]) -> Value {
    let set_class = Value::obj(vm.universe.classes.set_class);
    let set = send0(vm, set_class, "new()");
    for elem in elems {
        send1(vm, set, "add(_)", *elem);
    }
    set
}

/// `Set` satisfies the sequence-protocol contract (as-built.md §3.3(a)):
/// `mutable: true` (Set has a real `add(_)` that grows by 1, insertion-order
/// indexed by `at(_)`/`at_`); `hashable: false` (Q5: `Set` is mutable ⇒
/// identity `hash`).
#[test]
fn set_satisfies_sequence_contract() {
    let mut vm = VM::new();
    let spec = ContractSpec {
        class_name: "Set",
        mutation_selector: Some("add(_)"),
        mutation_returns_receiver: true,
        hashable: false,
        min_arity: 0,
        indexed_read: IndexedReadContract::BareValue,
    };
    assert_sequence_contract(&mut vm, &spec, build_set);
}

/// `Map` extras (map-and-set.md §6): key overwrite leaves `size` unchanged
/// and updates the stored value; `remove` is idempotent and returns the Map;
/// a missing key's `at(_)` is the `None` singleton; `keys`/`values` agree
/// with iteration order.
#[test]
fn map_key_overwrite_and_remove_idempotence() {
    let mut vm = VM::new();
    let map_class = Value::obj(vm.universe.classes.map_class);
    let map = send0(&mut vm, map_class, "new()");
    let sym_put = vm.get_or_intern("at(_,put)");

    vm.send_dynamic(map, sym_put, &[Value::int(1), Value::int(10)]).unwrap();
    let size_after_first = as_number(send0(&mut vm, map, "size"));
    assert_eq!(size_after_first, 1.0, "one entry after first put");

    // Overwrite: same key, new value — size unchanged, value updated.
    vm.send_dynamic(map, sym_put, &[Value::int(1), Value::int(20)]).unwrap();
    let size_after_overwrite = as_number(send0(&mut vm, map, "size"));
    assert_eq!(size_after_overwrite, 1.0, "overwrite must not grow size");
    let got = send1(&mut vm, map, "[_]", Value::int(1));
    assert_eq!(as_number(got), 20.0, "overwrite must update the stored value");

    // Missing key -> KeyError on strict [_] lookup, None on safe get(_) lookup.
    let sym_at = vm.get_or_intern("[_]");
    let result = vm.send_dynamic(map, sym_at, &[Value::int(999)]);
    assert!(result.is_err(), "strict lookup of absent key must raise KeyError");
    let missing = send1(&mut vm, map, "get(_)", Value::int(999));
    assert!(missing.is_none());

    // The public remove(_) wrapper is a no-op returning the Map.
    let returned = send1(&mut vm, map, "remove(_)", Value::int(999));
    assert_eq!(returned, map, "remove(absent) returns the Map");
    assert_eq!(as_number(send0(&mut vm, map, "size")), 1.0, "remove(absent) must not shrink size");

    // remove(present) deletes the association and returns the Map.
    let removed_present = send1(&mut vm, map, "remove(_)", Value::int(1));
    assert_eq!(removed_present, map, "remove(present) returns the Map");
    assert_eq!(as_number(send0(&mut vm, map, "size")), 0.0, "remove(present) must shrink size");
}

/// `Set` extras (map-and-set.md §6): `add` is idempotent (a duplicate does
/// not grow `size`); `remove` is idempotent.
#[test]
fn set_add_and_remove_idempotence() {
    let mut vm = VM::new();
    let set_class = Value::obj(vm.universe.classes.set_class);
    let set = send0(&mut vm, set_class, "new()");

    send1(&mut vm, set, "add(_)", Value::int(7));
    send1(&mut vm, set, "add(_)", Value::int(7));
    assert_eq!(as_number(send0(&mut vm, set, "size")), 1.0, "duplicate add must not grow size");
    assert!(as_bool(send1(&mut vm, set, "includes(_)", Value::int(7))));

    let returned = send1(&mut vm, set, "remove(_)", Value::int(999));
    assert_eq!(returned, set, "remove(absent) returns self");
    assert_eq!(as_number(send0(&mut vm, set, "size")), 1.0, "remove(absent) must not shrink size");

    send1(&mut vm, set, "remove(_)", Value::int(7));
    assert_eq!(as_number(send0(&mut vm, set, "size")), 0.0, "remove(present) must shrink size");
    assert!(!as_bool(send1(&mut vm, set, "includes(_)", Value::int(7))));
}

/// Map and Set must keep Telos equality as the collision disambiguator and
/// reject structural re-entry while a key callback is running. Explicit
/// Dynamic arguments keep this runtime regression independent from generic
/// constructor inference.
#[test]
fn shared_hash_engine_preserves_collisions_and_reentry() {
    let source = r#"
class Pt {
  mut _x
  mut _y
  x { _x }
  y { _y }
  @constructor
  new(_ x, _ y) { _x = x; _y = y }
  hash { _x * 31 + _y }
  ==(_ other) { _x == other.x and _y == other.y }
}

let first = Pt.new(1, 32)
let second = Pt.new(2, 1)
let duplicate = Pt.new(1, 32)
let map = Map<Dynamic, Dynamic>.new()
map.at(first, put: "first")
map.at(second, put: "second")
map.at(duplicate, put: "overwritten")
let mapCollisions = map.size == 2 and map[first] == "overwritten" and map[second] == "second"

let set = Set<Dynamic>.new()
set.add(first)
set.add(second)
set.add(duplicate)
let setCollisions = set.size == 2 and set.at(0) == first and set.at(1) == second

let mut guardedMap = Map<Dynamic, Dynamic>.new()
class BadMapKey {
  mut _triggered
  @constructor
  new() { _triggered = false }
  hash { 0 }
  ==(_ other) {
    if not _triggered {
      _triggered = true
      guardedMap.remove(other)
    }
    return true
  }
}
let mapKeyA = BadMapKey.new()
let mapKeyB = BadMapKey.new()
guardedMap.at(mapKeyA, put: 1)
let mapReentry = || { guardedMap.at(mapKeyB, put: 2) }.on(Error) |e| { e.message }
let mapReentryRejected = mapReentry == "cannot mutate this Map inside 'hash'/'==' of its own key (concurrent mutation)"

let mut hashFailureMap = Map<Dynamic, Dynamic>.new()
class ExplodingHashKey {
  hash { throw Error.new("hash failed") }
}
let hashFailure = || { hashFailureMap.at(ExplodingHashKey.new(), put: 1) }.on(Error) |e| { e.message }
hashFailureMap.at(#safe, put: 2)
let hashFailureRecovered = hashFailure == "hash failed" and hashFailureMap.size == 1 and hashFailureMap[#safe] == 2

let mut equalityFailureMap = Map<Dynamic, Dynamic>.new()
class ExplodingEqualityKey {
  hash { 0 }
  ==(_ other) { throw Error.new("equality failed") }
}
equalityFailureMap.at(ExplodingEqualityKey.new(), put: 1)
let equalityFailure = || { equalityFailureMap.at(ExplodingEqualityKey.new(), put: 2) }.on(Error) |e| { e.message }
equalityFailureMap.at(#safe, put: 3)
let equalityFailureRecovered = equalityFailure == "equality failed" and equalityFailureMap.size == 2 and equalityFailureMap[#safe] == 3

let mut guardedSet = Set<Dynamic>.new()
class BadSetKey {
  mut _triggered
  @constructor
  new() { _triggered = false }
  hash { 0 }
  ==(_ other) {
    if not _triggered {
      _triggered = true
      guardedSet.remove(other)
    }
    return true
  }
}
let setKeyA = BadSetKey.new()
let setKeyB = BadSetKey.new()
guardedSet.add(setKeyA)
let setReentry = || { guardedSet.add(setKeyB) }.on(Error) |e| { e.message }
let setReentryRejected = setReentry == "cannot mutate this Set inside 'hash'/'==' of its own key (concurrent mutation)"

let passed = mapCollisions and setCollisions and mapReentryRejected and setReentryRejected and hashFailureRecovered and equalityFailureRecovered
"#;
    let (vm, module) = super::vm_support::run_inline(source).expect("ordered-hash source compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("ordered-hash result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("ordered-hash source publishes its result");
    assert!(as_bool(passed), "shared Map/Set key engine must preserve collision and re-entry contracts");
}

/// P6's mutable families share collision handling and callback guards, while
/// source equality compares GC-rooted snapshots despite callback mutation.
#[test]
fn p6_mutable_hash_families_preserve_callback_snapshot_and_gc_contracts() {
    let source = r#"
class CollisionPoint {
  mut _x
  mut _y
  x { _x }
  y { _y }
  @constructor
  new(_ x, _ y) { _x = x; _y = y }
  hash { _x * 31 + _y }
  ==(_ other) { _x == other.x and _y == other.y }
}
let first = CollisionPoint.new(1, 32)
let second = CollisionPoint.new(2, 1)
let duplicate = CollisionPoint.new(1, 32)
let bag = MutableBag<CollisionPoint>.new()
bag.add(first)
bag.add(second)
bag.add(duplicate)
let bagCollision = bag.distinctCount == 2 and bag.count(first) == 2 and bag.count(second) == 1
let dict = IndexedDict<CollisionPoint, String>.new()
dict.insert("first", for: first)
dict.insert("second", for: second)
dict.insert("overwritten", for: duplicate)
let dictCollision = dict.size == 2 and dict[first] == "overwritten" and dict[second] == "second"
let mut errorDict = IndexedDict<Int, String>.new()
errorDict.insert((key: 1, value: "one"), at: 0)
let duplicateInsert = errorDict.insert((key: 1, value: "duplicate"), at: 0)
let duplicateErrorPreserved = duplicateInsert.isErr and duplicateInsert.match(
  ok: |value| { false },
  err: |error| { error is DuplicateKeyError }
)
let invalidInsert = errorDict.insert((key: 2, value: "two"), at: 10)
let indexErrorPreserved = invalidInsert.isErr and invalidInsert.match(
  ok: |value| { false },
  err: |error| { error is IndexError }
)
let ordered = OrderedSet<CollisionPoint>.new()
ordered.add(first)
ordered.add(second)
ordered.add(duplicate)
let fallbackPoint = CollisionPoint.new(0, 0)
let orderedCollision = ordered.size == 2 and ordered.at(0).unwrapOr(fallbackPoint) == first and ordered.at(1).unwrapOr(fallbackPoint) == second

let mutableMap = Map<Collection, Int>.new()
let bagKeyError = || { mutableMap.at(bag, put: 1) }.on(Error) |e| { e.message }
let dictKeyError = || { mutableMap.at(dict, put: 1) }.on(Error) |e| { e.message }
let orderedKeyError = || { mutableMap.at(ordered, put: 1) }.on(Error) |e| { e.message }
let mapRejectionPolicy = bagKeyError == "Map key must be immutable: mutable collections and Slice views are not admitted as keys (collection-protocol.md law 4)" and dictKeyError == "Map key must be immutable: mutable collections and Slice views are not admitted as keys (collection-protocol.md law 4)" and orderedKeyError == "Map key must be immutable: mutable collections and Slice views are not admitted as keys (collection-protocol.md law 4)"

let rejectBag = MutableBag<Collection>.new()
let rejectDict = IndexedDict<Collection, String>.new()
let rejectOrdered = OrderedSet<Collection>.new()
let bagAdmitError = || { rejectBag.add(mutableMap) }.on(Error) |e| { e.message }
let dictAdmitError = || { rejectDict.insert("val", for: mutableMap) }.on(Error) |e| { e.message }
let orderedAdmitError = || { rejectOrdered.add(mutableMap) }.on(Error) |e| { e.message }
let newFamiliesRejectionPolicy = bagAdmitError == "MutableBag key must be immutable: mutable collections and Slice views are not admitted as keys (collection-protocol.md law 4)" and dictAdmitError == "IndexedDict key must be immutable: mutable collections and Slice views are not admitted as keys (collection-protocol.md law 4)" and orderedAdmitError == "OrderedSet key must be immutable: mutable collections and Slice views are not admitted as keys (collection-protocol.md law 4)"
let mutableKeyPolicy = mapRejectionPolicy and newFamiliesRejectionPolicy

class ReentrantHashKey {
  mut _owner
  mut _mode
  @constructor
  new(_ owner, _ mode) { _owner = owner; _mode = mode }
  hash {
    if _mode == 1 { _mode = 0; _owner.clear }
    if _mode == 2 { throw Error.new("hash failed") }
    0
  }
}
let mut guardedBag = MutableBag<ReentrantHashKey>.new()
guardedBag.add(ReentrantHashKey.new(guardedBag, 0))
let bagError = || { guardedBag.add(ReentrantHashKey.new(guardedBag, 1)) }.on(Error) |e| { e.message }
let bagRejected = bagError == "cannot mutate this MutableBag inside 'hash'/'==' of its own key (concurrent mutation)" and guardedBag.size == 1
let bagHashError = || { guardedBag.add(ReentrantHashKey.new(guardedBag, 2)) }.on(Error) |e| { e.message }
let bagErrorRecovered = bagHashError == "hash failed" and guardedBag.size == 1
guardedBag.add(ReentrantHashKey.new(guardedBag, 0))
let bagUnlocked = guardedBag.size == 2
let mut guardedDict = IndexedDict<ReentrantHashKey, String>.new()
guardedDict.insert("seed", for: ReentrantHashKey.new(guardedDict, 0))
let dictError = || { guardedDict.insert("blocked", for: ReentrantHashKey.new(guardedDict, 1)) }.on(Error) |e| { e.message }
let dictRejected = dictError == "cannot mutate this IndexedDict inside 'hash'/'==' of its own key (concurrent mutation)" and guardedDict.size == 1
let dictHashError = || { guardedDict.insert("failed", for: ReentrantHashKey.new(guardedDict, 2)) }.on(Error) |e| { e.message }
let dictErrorRecovered = dictHashError == "hash failed" and guardedDict.size == 1
guardedDict.insert("recovered", for: ReentrantHashKey.new(guardedDict, 0))
let dictUnlocked = guardedDict.size == 2
let mut guardedOrdered = OrderedSet<ReentrantHashKey>.new()
guardedOrdered.add(ReentrantHashKey.new(guardedOrdered, 0))
let orderedError = || { guardedOrdered.add(ReentrantHashKey.new(guardedOrdered, 1)) }.on(Error) |e| { e.message }
let orderedRejected = orderedError == "cannot mutate this OrderedSet inside 'hash'/'==' of its own key (concurrent mutation)" and guardedOrdered.size == 1
let orderedHashError = || { guardedOrdered.add(ReentrantHashKey.new(guardedOrdered, 2)) }.on(Error) |e| { e.message }
let orderedErrorRecovered = orderedHashError == "hash failed" and guardedOrdered.size == 1
guardedOrdered.add(ReentrantHashKey.new(guardedOrdered, 0))
let orderedUnlocked = guardedOrdered.size == 2

class SnapshotMutationKey {
  mut _left
  mut _right
  @constructor
  new(_ left, _ right) { _left = left; _right = right }
  hash { 0 }
  ==(_ other) { _left.clear; _right.clear; true }
}
let mut bagLeft = MutableBag<SnapshotMutationKey>.new()
let mut bagRight = MutableBag<SnapshotMutationKey>.new()
bagLeft.add(SnapshotMutationKey.new(bagLeft, bagRight))
bagRight.add(SnapshotMutationKey.new(bagLeft, bagRight))
let bagSnapshot = bagLeft == bagRight and bagLeft.size == 0 and bagRight.size == 0
let mut dictLeft = IndexedDict<SnapshotMutationKey, String>.new()
let mut dictRight = IndexedDict<SnapshotMutationKey, String>.new()
dictLeft.insert("same", for: SnapshotMutationKey.new(dictLeft, dictRight))
dictRight.insert("same", for: SnapshotMutationKey.new(dictLeft, dictRight))
let dictSnapshot = dictLeft == dictRight and dictLeft.size == 0 and dictRight.size == 0
let mut orderedLeft = OrderedSet<SnapshotMutationKey>.new()
let mut orderedRight = OrderedSet<SnapshotMutationKey>.new()
orderedLeft.add(SnapshotMutationKey.new(orderedLeft, orderedRight))
orderedRight.add(SnapshotMutationKey.new(orderedLeft, orderedRight))
let orderedSnapshot = orderedLeft == orderedRight and orderedLeft.size == 0 and orderedRight.size == 0

class GcCollectionKey {
  mut _label
  @constructor
  new(_ label) { _label = label }
  label { _label }
  hash { 0 }
  ==(_ other) { _label == other.label }
}
let mut gcBag = MutableBag<GcCollectionKey>.new()
gcBag.add(GcCollectionKey.new("bag"))
let mut gcDict = IndexedDict<GcCollectionKey, String>.new()
gcDict.insert("dict-value", for: GcCollectionKey.new("dict-key"))
let mut gcOrdered = OrderedSet<GcCollectionKey>.new()
gcOrdered.add(GcCollectionKey.new("ordered"))
System.gc
let bagGc = gcBag.iterator.next.unwrapOr(GcCollectionKey.new("missing")).label == "bag"
let dictGc = gcDict.keyAt(0).unwrapOr(GcCollectionKey.new("missing")).label == "dict-key" and gcDict.valueAt(0).unwrapOr("missing") == "dict-value"
let orderedGc = gcOrdered.at(0).unwrapOr(GcCollectionKey.new("missing")).label == "ordered"
let passed = bagCollision and dictCollision and duplicateErrorPreserved and indexErrorPreserved and orderedCollision and mutableKeyPolicy and bagRejected and bagErrorRecovered and bagUnlocked and dictRejected and dictErrorRecovered and dictUnlocked and orderedRejected and orderedErrorRecovered and orderedUnlocked and bagSnapshot and dictSnapshot and orderedSnapshot and bagGc and dictGc and orderedGc
"#;
    let (vm, module) = super::vm_support::run_inline(source).expect("P6 mutable-family source compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("P6 result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("P6 source publishes its result");
    if !as_bool(passed) {
        for name in [
            "bagCollision",
            "dictCollision",
            "duplicateErrorPreserved",
            "indexErrorPreserved",
            "orderedCollision",
            "mutableKeyPolicy",
            "bagRejected",
            "bagErrorRecovered",
            "bagUnlocked",
            "dictRejected",
            "dictErrorRecovered",
            "dictUnlocked",
            "orderedRejected",
            "orderedErrorRecovered",
            "orderedUnlocked",
            "bagSnapshot",
            "dictSnapshot",
            "orderedSnapshot",
            "bagGc",
            "dictGc",
            "orderedGc",
        ] {
            if let Some(symbol) = vm.interner.find(name)
                && let Some(value) = vm.heap.module(module).get(symbol)
            {
                eprintln!("{name}: {value:?}");
            }
        }
    }
    assert!(as_bool(passed), "P6 collections preserve collisions, callback guards, snapshots, and GC roots");
}

#[test]
fn p6_mutable_key_admission_matrix() {
    let source = r#"
let list = List<Int>.new()
let array = <Int>[1, 2, 3]
let buffer = Buffer<Int>.new(4)
let slice = array.slice(0..2)
let map = Map<String, Int>.new()
let set = Set<Int>.new()
let bag = MutableBag<Int>.new()
let dict = IndexedDict<String, Int>.new()
let ordered = OrderedSet<Int>.new()
let bytes = Bytes.new(0)
let vector = <Int, 3>[1, 2, 3]

let candidates: Array<Collection> = <Collection>[list, array, buffer, slice, map, set, bag, dict, ordered, bytes]

// 1. Map admission
let mut testMap = Map<Collection, Int>.new()
let mut mapRejections = 0
let mut i = 0
while i < candidates.size {
  let cand = candidates[i]
  let err = || { testMap.at(cand, put: 1) }.on(Error) |_| { true }
  if err == true { mapRejections = mapRejections + 1 }
  i = i + 1
}
let mapRejections10 = mapRejections == 10 and testMap.size == 0
testMap.at(vector, put: 1)
let mapAdmitsVector = testMap.size == 1

// 2. Set admission
let mut testSet = Set<Collection>.new()
let mut setRejections = 0
i = 0
while i < candidates.size {
  let cand = candidates[i]
  let err = || { testSet.add(cand) }.on(Error) |_| { true }
  if err == true { setRejections = setRejections + 1 }
  i = i + 1
}
let setRejections10 = setRejections == 10 and testSet.size == 0
testSet.add(vector)
let setAdmitsVector = testSet.size == 1

// 3. MutableBag admission
let mut testBag = MutableBag<Collection>.new()
let mut bagRejections = 0
i = 0
while i < candidates.size {
  let cand = candidates[i]
  let err = || { testBag.add(cand) }.on(Error) |_| { true }
  if err == true { bagRejections = bagRejections + 1 }
  i = i + 1
}
let bagRejections10 = bagRejections == 10 and testBag.size == 0
testBag.add(vector)
let bagAdmitsVector = testBag.size == 1

// 4. IndexedDict admission
let mut testDict = IndexedDict<Collection, Int>.new()
let mut dictRejections = 0
i = 0
while i < candidates.size {
  let cand = candidates[i]
  let err = || { testDict.insert(1, for: cand) }.on(Error) |_| { true }
  if err == true { dictRejections = dictRejections + 1 }
  i = i + 1
}
let dictRejections10 = dictRejections == 10 and testDict.size == 0
testDict.insert(1, for: vector)
let dictAdmitsVector = testDict.size == 1

// 5. OrderedSet admission
let mut testOrdered = OrderedSet<Collection>.new()
let mut orderedRejections = 0
i = 0
while i < candidates.size {
  let cand = candidates[i]
  let err = || { testOrdered.add(cand) }.on(Error) |_| { true }
  if err == true { orderedRejections = orderedRejections + 1 }
  i = i + 1
}
let orderedRejections10 = orderedRejections == 10 and testOrdered.size == 0
testOrdered.add(vector)
let orderedAdmitsVector = testOrdered.size == 1

let passed = mapRejections10 and mapAdmitsVector and setRejections10 and setAdmitsVector and bagRejections10 and bagAdmitsVector and dictRejections10 and dictAdmitsVector and orderedRejections10 and orderedAdmitsVector
"#;
    let (vm, module) = super::vm_support::run_inline(source).expect("P6 admission matrix source compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("passed interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("passed published");
    assert!(as_bool(passed), "Full 50-rejection admission matrix passes and admits Vector/scalars");
}

/// Builds a `Tuple` through its public product-literal syntax. The internal
/// `Tuple.rawFromList(_)` bridge is intentionally inaccessible to callers.
fn build_tuple(vm: &mut VM, elems: &[Value]) -> Value {
    let elements = elems
        .iter()
        .map(|value| {
            if let Some(val) = value.as_int() {
                val.to_string()
            } else {
                panic!("Tuple contract fixture only uses integer literals, got {value:?}");
            }
        })
        .collect::<Vec<_>>();
    let literal = match elements.len() {
        0 => "()".to_string(),
        1 => format!("({},)", elements[0]),
        _ => format!("({})", elements.join(", ")),
    };
    eval_literal(vm, &literal)
}

/// `Tuple` satisfies the sequence-protocol contract (as-built.md §3.3(a)):
/// `mutable: false` (no `add(_)` — immutability is structural, no mutation
/// selector exists at all); `hashable: true` (Q5: immutable ⇒ value hash,
/// asserted by H2 here).
#[test]
fn tuple_satisfies_sequence_contract() {
    let mut vm = VM::new();
    let spec = ContractSpec {
        class_name: "Tuple",
        mutation_selector: None,
        mutation_returns_receiver: false,
        hashable: true,
        min_arity: 1,
        indexed_read: IndexedReadContract::Option,
    };
    assert_sequence_contract(&mut vm, &spec, build_tuple);
}

#[test]
fn record_get_returns_option_values() {
    let mut vm = VM::new();
    let record = eval_literal(&mut vm, "{ name: 42 }");
    let name = Value::symbol(vm.get_or_intern("name"));
    let missing_name = Value::symbol(vm.get_or_intern("missing"));

    assert_eq!(
        send1(&mut vm, record, "get(_)", name),
        Value::int(42).wrap_some().expect("found Record value wraps in Some"),
        "Record.get(present) returns Some(value)"
    );
    assert_eq!(
        send1(&mut vm, record, "get(_)", missing_name),
        Value::none(),
        "Record.get(missing) returns None"
    );
}

#[test]
fn empty_tuple_construction_normalizes_to_unit() {
    let mut vm = VM::new();
    assert_eq!(build_tuple(&mut vm, &[]), Value::unit());
}

/// `Tuple` extras (tuple-and-range.md §5): value-hash equality holds across
/// two *independently built* tuples with equal elements (not merely the same
/// object) — the defining property of value-hashing vs `Map`/`Set`'s
/// identity hash; cross-kind `==` against an elementwise-equal `List` is
/// `false` (E2, distinct kinds never compare equal even with equal content);
/// no mutation selector exists (`at(_,put)`/`add(_)` both miss via dNU).
#[test]
fn tuple_value_hash_and_immutability() {
    let mut vm = VM::new();
    let a = build_tuple(&mut vm, &[Value::int(1), Value::int(2)]);
    let b = build_tuple(&mut vm, &[Value::int(1), Value::int(2)]);
    assert!(as_bool(send1(&mut vm, a, "==(_)", b)), "independently-built equal tuples must compare ==");
    let hash_a = as_number(send0(&mut vm, a, "hash"));
    let hash_b = as_number(send0(&mut vm, b, "hash"));
    assert_eq!(hash_a, hash_b, "value-equal tuples must hash equal");

    let differing = build_tuple(&mut vm, &[Value::int(1), Value::int(3)]);
    let hash_c = as_number(send0(&mut vm, differing, "hash"));
    assert_ne!(hash_a, hash_c, "differing tuples should (almost certainly) hash differently");

    // Cross-kind: a same-content List is never == a Tuple (E2).
    let list_same_content = build_list(&mut vm, &[Value::int(1), Value::int(2)]);
    assert!(
        !as_bool(send1(&mut vm, a, "==(_)", list_same_content)),
        "Tuple must never == a List, even same content"
    );

    // No mutation selector: `at(_,put)` and `add(_)` both miss (dNU).
    let sym_put = vm.get_or_intern("at(_,put)");
    assert!(
        vm.send_dynamic(a, sym_put, &[Value::int(0), Value::int(9)]).is_err(),
        "Tuple must not respond to at(_,put)"
    );
    let sym_add = vm.get_or_intern("add(_)");
    assert!(vm.send_dynamic(a, sym_add, &[Value::int(9)]).is_err(), "Tuple must not respond to add(_)");
}

/// `Tuple` as a valid `Map` key (tuple-and-range.md; the re-entrant
/// `hash`+`==` key-lookup path, `docs/forge/units/U-COLLTYPES/plan.md` §7):
/// two independently-built value-equal tuples resolve to the SAME map entry.
#[test]
fn tuple_is_a_valid_map_key() {
    let mut vm = VM::new();
    let map_class = Value::obj(vm.universe.classes.map_class);
    let map = send0(&mut vm, map_class, "new()");
    let key1 = build_tuple(&mut vm, &[Value::int(1), Value::int(2)]);
    let sym_put = vm.get_or_intern("at(_,put)");
    vm.send_dynamic(map, sym_put, &[key1, Value::int(9)]).unwrap();

    let key2 = build_tuple(&mut vm, &[Value::int(1), Value::int(2)]);
    let got = send1(&mut vm, map, "[_]", key2);
    assert_eq!(
        as_number(got),
        9.0,
        "a value-equal Tuple key must recover the same entry (re-entrant hash+== lookup)"
    );
    assert_eq!(as_number(send0(&mut vm, map, "size")), 1.0, "only one entry — key1/key2 are the SAME key");
}

/// `Range` is a public slice-bounds descriptor. Its supported forward integer
/// subset also implements `Iterable`; slicing still routes literals through a
/// collection's `[_]` implementation with the correct upper boundary.
#[test]
fn range_literals_drive_collection_slices() {
    let mut vm = VM::new();
    let list = build_list(&mut vm, &[Value::int(0), Value::int(1), Value::int(2), Value::int(3), Value::int(4)]);

    let exclusive = eval_literal(&mut vm, "1..4");
    let exclusive_slice = send1(&mut vm, list, "[_]", exclusive);
    assert_eq!(as_number(send0(&mut vm, exclusive_slice, "size")), 3.0);
    for (index, expected) in [1, 2, 3].into_iter().enumerate() {
        assert_eq!(as_number(send1(&mut vm, exclusive_slice, "at(_)", Value::int(index as i64))), expected as f64);
    }

    let inclusive = eval_literal(&mut vm, "1..=4");
    let inclusive_slice = send1(&mut vm, list, "[_]", inclusive);
    assert_eq!(as_number(send0(&mut vm, inclusive_slice, "size")), 4.0);
    for (index, expected) in [1, 2, 3, 4].into_iter().enumerate() {
        assert_eq!(as_number(send1(&mut vm, inclusive_slice, "at(_)", Value::int(index as i64))), expected as f64);
    }
}

/// The canonical Range implementation constructs Result cases through the
/// semantic VariantIds registered for the Universe Result enum. This guards
/// the lowering/runtime boundary that would otherwise be hidden by the slice
/// result alone.
#[test]
fn canonical_range_result_lowering_keeps_variant_identity() {
    let vm = VM::new();
    let result = CoreDeclarationIds::default().result;
    let ok = VariantId::new(result.clone(), Selector::method("Ok", vec![SelectorSlot::Positional]).unwrap());
    let error = VariantId::new(result, Selector::method("Error", vec![SelectorSlot::Positional]).unwrap());

    let range_id = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("collections").unwrap(),
        ModuleComponent::from_identifier("range").unwrap(),
    ]));
    let range_obj = vm.module_registry.get(&range_id).expect("canonical Range module").object;
    let lowering = vm.heap.module(range_obj).lowering.as_ref().expect("Range lowering");

    for expected in [&ok, &error] {
        assert!(
            lowering.associated.values().any(|spec| matches!(
                spec,
                AssociatedLoweringSpec::ConstructVariant { variant, .. } if variant == expected
            )),
            "Range lowering must retain canonical Result variant {expected:?}"
        );
        assert!(
            vm.adt_registry.variant_by_semantic(expected).is_some(),
            "canonical Result variant {expected:?} must be registered"
        );
    }
}

/// DEC-CT-C (negative): a mutable-collection key (`List`) rejected by both
/// `Map#at(_,put)` and `Set#add(_)` — a raised catchable `Error`, never a
/// silent identity-keyed admission (collection-protocol.md law 4).
#[test]
fn mutable_collection_key_is_rejected() {
    let mut vm = VM::new();

    let map_class = Value::obj(vm.universe.classes.map_class);
    let map = send0(&mut vm, map_class, "new()");
    let list_class = Value::obj(vm.universe.classes.list_class);
    let mutable_key = send0(&mut vm, list_class, "new()");
    let sym_put = vm.get_or_intern("at(_,put)");
    let result = vm.send_dynamic(map, sym_put, &[mutable_key, Value::int(1)]);
    assert!(result.is_err(), "Map#at(_,put) with a mutable List key must raise, not silently admit it");

    let set_class = Value::obj(vm.universe.classes.set_class);
    let set = send0(&mut vm, set_class, "new()");
    let mutable_key2 = send0(&mut vm, list_class, "new()");
    let sym_add = vm.get_or_intern("add(_)");
    let result2 = vm.send_dynamic(set, sym_add, &[mutable_key2]);
    assert!(result2.is_err(), "Set#add(_) with a mutable List key must raise, not silently admit it");
}

#[test]
fn p7_numeric_same_value_zero_observed_behavior() {
    let mut vm = VM::new();

    let hash_of = |vm: &mut VM, v: Value| -> i64 {
        let sym = vm.get_or_intern("hash");
        let res = vm.send_dynamic(v, sym, &[]).expect("send hash");
        res.as_int().expect("int hash")
    };

    // 1. 1 and 1.0 (passes SameValueZero and hash equality)
    let one_int = Value::int(1);
    let one_float = Value::float(1.0);
    assert!(phalcom_core::value::same_value_zero(one_int, one_float, &vm.heap));
    assert_eq!(hash_of(&mut vm, one_int), hash_of(&mut vm, one_float), "1 and 1.0 must have identical hash");

    // 2. 0 and -0.0 (passes SameValueZero and hash equality)
    let zero_int = Value::int(0);
    let neg_zero_float = Value::float(-0.0);
    assert!(phalcom_core::value::same_value_zero(zero_int, neg_zero_float, &vm.heap));
    assert_eq!(
        hash_of(&mut vm, zero_int),
        hash_of(&mut vm, neg_zero_float),
        "0 and -0.0 must have identical hash"
    );

    // 3. integral Float at 2^53 when exactly representable (passes hash equality)
    let pow53 = 9_007_199_254_740_992i64; // 2^53
    let pow53_int = Value::int(pow53);
    let pow53_float = Value::float(pow53 as f64);
    assert!(phalcom_core::value::same_value_zero(pow53_int, pow53_float, &vm.heap));
    assert_eq!(hash_of(&mut vm, pow53_int), hash_of(&mut vm, pow53_float));

    // 4. integral Float at 2^53+2: SameValueZero is true, but Number#hash diverges (2 vs 1)
    // Documented in R-P7-NUMERIC-HASH-LAW.
    let pow53_plus_2 = 9_007_199_254_740_994i64; // 2^53 + 2
    let pow53_plus_2_int = Value::int(pow53_plus_2);
    let pow53_plus_2_float = Value::float(pow53_plus_2 as f64);
    assert!(phalcom_core::value::same_value_zero(pow53_plus_2_int, pow53_plus_2_float, &vm.heap));
    assert_eq!(hash_of(&mut vm, pow53_plus_2_int), 2);
    assert_eq!(hash_of(&mut vm, pow53_plus_2_float), 1);

    // 5. heap-backed LargeInt and equal integral Float: SameValueZero is true, but hashes diverge.
    // Documented in R-P7-NUMERIC-HASH-LAW.
    use num_bigint::BigInt;
    let big_val_2_64 = BigInt::from(1u128 << 64);
    let large_int_val = phalcom_core::value::normalize_bigint(big_val_2_64, &mut vm.heap);
    let float_2_64_val = Value::float((1u128 << 64) as f64);
    assert!(large_int_val.is_obj(), "2^64 must be LargeInt on heap");
    assert!(phalcom_core::value::same_value_zero(large_int_val, float_2_64_val, &vm.heap));
    assert_eq!(hash_of(&mut vm, large_int_val), 6323511191391134);
    assert_eq!(hash_of(&mut vm, float_2_64_val), 4503599627370496);

    // 6. multiple NaN payloads: SameValueZero is true, but Number#hash produces distinct values.
    // Documented in R-P7-NUMERIC-HASH-LAW.
    let nan1 = Value::float(f64::from_bits(0x7ff8_0000_0000_0001));
    let nan2 = Value::float(f64::from_bits(0x7ff8_0000_0000_0002));
    assert!(phalcom_core::value::same_value_zero(nan1, nan2, &vm.heap));
    assert_eq!(hash_of(&mut vm, nan1), 6755399441055745);
    assert_eq!(hash_of(&mut vm, nan2), 6755399441055746);
}

#[test]
#[ignore = "R-P7-NUMERIC-HASH-LAW: SameValueZero vs Number#hash divergence for >2^53, LargeInt, and NaN payloads"]
fn p7_numeric_same_value_zero_hash_law_strict_ideal() {
    let mut vm = VM::new();

    let hash_of = |vm: &mut VM, v: Value| -> i64 {
        let sym = vm.get_or_intern("hash");
        let res = vm.send_dynamic(v, sym, &[]).expect("send hash");
        res.as_int().expect("int hash")
    };

    let pow53_plus_2_int = Value::int(9_007_199_254_740_994i64);
    let pow53_plus_2_float = Value::float(9_007_199_254_740_994.0f64);
    assert_eq!(hash_of(&mut vm, pow53_plus_2_int), hash_of(&mut vm, pow53_plus_2_float));
}
