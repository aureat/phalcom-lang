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

use phalcom_core::value::Value;
use phalcom_core::vm::VM;

/// The static parameters that distinguish one collection kind from another
/// (as-built.md §2): whether it supports in-place growth, whether it is a
/// valid `Map`/`Set` key, and whether its iteration order is a function of
/// construction.
struct ContractSpec {
    /// The collection's class name, used only for assertion diagnostics.
    class_name: &'static str,
    /// Whether the collection supports in-place growth via `add(_)` (L5).
    /// `List` is `true`; an eventual `Tuple` would be `false`.
    mutable: bool,
    /// Whether the collection is a valid `Map`/`Set` key (H1/H2). `List` is
    /// `false` — its inherited identity `hash` is inconsistent with the
    /// structural `==` this unit adds (as-built.md §2.4), so the H2
    /// consistency assertion is skipped for it.
    hashable: bool,
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
    vm.send_dynamic(receiver, sym, &[arg]).unwrap_or_else(|_| panic!("send `{selector}` failed"))
}

/// Extracts the `f64` behind a `Number` result (test-local helper, mirrors
/// `tests/invariants.rs::as_number`).
fn as_number(value: Value) -> f64 {
    match value {
        Value::Number(n) => n,
        other => panic!("expected a Number, got {other:?}"),
    }
}

/// Extracts the `bool` behind a `Bool` result.
fn as_bool(value: Value) -> bool {
    match value {
        Value::Bool(b) => b,
        other => panic!("expected a Bool, got {other:?}"),
    }
}

/// Builds a `List` from element values through the *surface* protocol
/// (`List.new()` + `.add(_)`), so the harness exercises the same path user
/// code does.
fn build_list(vm: &mut VM, elems: &[Value]) -> Value {
    let list_class = Value::Obj(vm.universe.classes.list_class);
    let list = send0(vm, list_class, "new()");
    for elem in elems {
        send1(vm, list, "add(_:)", *elem);
    }
    list
}

/// Runs the full R-INV-5.x sequence + equality contract against whatever
/// `build` produces, parameterized by `spec` (as-built.md §2, §3.3(a)).
///
/// # Panics
///
/// Panics (via the `send0`/`send1`/`assert*` helpers) on the first law that
/// fails — this is a test harness, not a `Result`-returning API.
fn assert_sequence_contract(vm: &mut VM, spec: &ContractSpec, build: impl Fn(&mut VM, &[Value]) -> Value) {
    // L1/L2: size is a Number, >= 0, and == the element count, for n = 0..3.
    for n in 0..=3 {
        let elems: Vec<Value> = (0..n).map(|i| Value::Number(i as f64)).collect();
        let collection = build(vm, &elems);
        let size = as_number(send0(vm, collection, "size"));
        assert!(size >= 0.0, "{}: size must be >= 0, got {size}", spec.class_name);
        assert_eq!(size, n as f64, "{}: size must equal the element count", spec.class_name);

        // L3: at(i) recovers each element by its own `==`, for 0 <= i < n.
        for (i, elem) in elems.iter().enumerate() {
            let got = send1(vm, collection, "at(_:)", Value::Number(i as f64));
            assert!(
                as_bool(send1(vm, got, "==(_:)", *elem)),
                "{}: at({i}) should recover the element it was built with",
                spec.class_name
            );
        }

        // L4: at(n) (and beyond) is the `None` singleton — total, never a
        // panic, never the raw `nil` sentinel (Invariant 4).
        let out_of_range = send1(vm, collection, "at(_:)", Value::Number(n as f64));
        assert!(
            matches!(out_of_range, Value::Obj(id) if id == vm.universe.classes.none_singleton),
            "{}: at(size) must surface the None singleton",
            spec.class_name
        );
        assert_ne!(out_of_range, Value::Nil, "{}: at(size) must never leak the raw sentinel", spec.class_name);
    }

    // L5: add(x) grows size by 1 and at(oldSize) == x (mutable collections
    // only); add returns the (chainable) receiver.
    if spec.mutable {
        let collection = build(vm, &[Value::Number(1.0), Value::Number(2.0)]);
        let old_size = as_number(send0(vm, collection, "size"));
        let returned = send1(vm, collection, "add(_:)", Value::Number(42.0));
        let new_size = as_number(send0(vm, collection, "size"));
        assert_eq!(new_size, old_size + 1.0, "{}: add(_) must grow size by 1", spec.class_name);
        let last = send1(vm, collection, "at(_:)", Value::Number(old_size));
        assert!(
            as_bool(send1(vm, last, "==(_:)", Value::Number(42.0))),
            "{}: add(x) must place x at the old size index",
            spec.class_name
        );
        assert!(
            as_bool(send1(vm, returned, "==(_:)", collection)),
            "{}: add(_) must return the (chainable) receiver",
            spec.class_name
        );
    }

    // E1/E3/E4: A == B (equal elements, same order) is true; A == A is true
    // (reflexive); (A == B) == (B == A) (symmetric).
    let elems = [Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
    let a = build(vm, &elems);
    let b = build(vm, &elems);
    let a_eq_b = as_bool(send1(vm, a, "==(_:)", b));
    let b_eq_a = as_bool(send1(vm, b, "==(_:)", a));
    assert!(a_eq_b, "{}: structurally-equal collections must compare ==", spec.class_name);
    assert!(as_bool(send1(vm, a, "==(_:)", a)), "{}: == must be reflexive", spec.class_name);
    assert_eq!(a_eq_b, b_eq_a, "{}: == must be symmetric", spec.class_name);

    // E2: cross-kind comparison is false, never a dNU.
    let cross_kind = send1(vm, a, "==(_:)", Value::Number(1.0));
    assert!(!as_bool(cross_kind), "{}: == must be false across collection kinds", spec.class_name);

    // E1/E5: a collection differing at one index is unequal; transitivity
    // via a second equal-to-`b` collection.
    let differing = [Value::Number(1.0), Value::Number(9.0), Value::Number(3.0)];
    let c = build(vm, &differing);
    assert!(!as_bool(send1(vm, a, "==(_:)", c)), "{}: differing elements must compare unequal", spec.class_name);
    let a2 = build(vm, &elems);
    assert!(as_bool(send1(vm, b, "==(_:)", a2)), "{}: transitivity precondition (B == A2)", spec.class_name);
    assert!(as_bool(send1(vm, a, "==(_:)", a2)), "{}: == must be transitive (A == B, B == A2 => A == A2)", spec.class_name);

    // E6: != is the logical negation of ==, routed through it (not floor
    // identity) — the `==`/`!=` decoupling hazard this unit guards against.
    assert!(as_bool(send1(vm, a, "!=(_:)", c)), "{}: != must hold where == fails", spec.class_name);
    assert!(!as_bool(send1(vm, a, "!=(_:)", b)), "{}: != must be false where == holds", spec.class_name);

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
    let spec = ContractSpec { class_name: "List", mutable: true, hashable: false };
    assert_sequence_contract(&mut vm, &spec, build_list);
}
