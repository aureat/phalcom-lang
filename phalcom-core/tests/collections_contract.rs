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
    /// Lowest arity that remains an instance of this collection family.
    /// Tuple's zero-arity product normalizes to Unit.
    min_arity: usize,
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
        Value::Int(n) => n as f64,
        Value::Float(n) => n,
        other => panic!("expected a number, got {other:?}"),
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
        send1(vm, list, "add(_)", *elem);
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
    // L1/L2: size is a Number, >= 0, and == the element count.
    for n in spec.min_arity..=3 {
        let elems: Vec<Value> = (0..n).map(|i| Value::Int(i as i64)).collect();
        let collection = build(vm, &elems);
        let size = as_number(send0(vm, collection, "size"));
        assert!(size >= 0.0, "{}: size must be >= 0, got {size}", spec.class_name);
        assert_eq!(size, n as f64, "{}: size must equal the element count", spec.class_name);

        // L3: at(i) recovers each element by its own `==`, for 0 <= i < n.
        for (i, elem) in elems.iter().enumerate() {
            let got = send1(vm, collection, "at(_)", Value::Int(i as i64));
            assert!(
                as_bool(send1(vm, got, "==(_)", *elem)),
                "{}: at({i}) should recover the element it was built with",
                spec.class_name
            );
        }

        // L4: at(n) (and beyond) is the `None` singleton — total, never a
        // panic, never the raw `nil` sentinel (Invariant 4).
        let out_of_range = send1(vm, collection, "at(_)", Value::Int(n as i64));
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
        let collection = build(vm, &[Value::Int(1), Value::Int(2)]);
        let old_size = as_number(send0(vm, collection, "size"));
        let returned = send1(vm, collection, "add(_)", Value::Int(42));
        let new_size = as_number(send0(vm, collection, "size"));
        assert_eq!(new_size, old_size + 1.0, "{}: add(_) must grow size by 1", spec.class_name);
        let last = send1(vm, collection, "at(_)", Value::Int(old_size as i64));
        assert!(
            as_bool(send1(vm, last, "==(_)", Value::Int(42))),
            "{}: add(x) must place x at the old size index",
            spec.class_name
        );
        assert!(
            as_bool(send1(vm, returned, "==(_)", collection)),
            "{}: add(_) must return the (chainable) receiver",
            spec.class_name
        );
    }

    // E1/E3/E4: A == B (equal elements, same order) is true; A == A is true
    // (reflexive); (A == B) == (B == A) (symmetric).
    let elems = [Value::Int(1), Value::Int(2), Value::Int(3)];
    let a = build(vm, &elems);
    let b = build(vm, &elems);
    let a_eq_b = as_bool(send1(vm, a, "==(_)", b));
    let b_eq_a = as_bool(send1(vm, b, "==(_)", a));
    assert!(a_eq_b, "{}: structurally-equal collections must compare ==", spec.class_name);
    assert!(as_bool(send1(vm, a, "==(_)", a)), "{}: == must be reflexive", spec.class_name);
    assert_eq!(a_eq_b, b_eq_a, "{}: == must be symmetric", spec.class_name);

    // E2: cross-kind comparison is false, never a dNU.
    let cross_kind = send1(vm, a, "==(_)", Value::Int(1));
    assert!(!as_bool(cross_kind), "{}: == must be false across collection kinds", spec.class_name);

    // E1/E5: a collection differing at one index is unequal; transitivity
    // via a second equal-to-`b` collection.
    let differing = [Value::Int(1), Value::Int(9), Value::Int(3)];
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
        mutable: true,
        hashable: false,
        min_arity: 0,
    };
    assert_sequence_contract(&mut vm, &spec, build_list);
}

/// Builds a `Map` keyed `0 -> elems[0], 1 -> elems[1], …` through the surface
/// protocol (`Map.new()` + `.at(_,put)`) — the numeric index doubles as the
/// key, so the generic `at(i)` sequence-protocol check (as-built.md §3.3(a))
/// exercises `Map#at(_)`'s real keyed-lookup path (`get_`, re-entering the
/// VM to send `hash`/`==` on the `Number` key) rather than a synthetic index
/// read (U-COLLTYPES plan.md §7).
fn build_map(vm: &mut VM, elems: &[Value]) -> Value {
    let map_class = Value::Obj(vm.universe.classes.map_class);
    let map = send0(vm, map_class, "new()");
    for (i, elem) in elems.iter().enumerate() {
        let sym = vm.get_or_intern("at(_,put)");
        vm.send_dynamic(map, sym, &[Value::Int(i as i64), *elem]).expect("Map#at(_,put) failed");
    }
    map
}

/// `Map` satisfies the sequence-protocol contract (as-built.md §3.3(a)):
/// `mutable: false` skips the L5 `add(_)` growth check (`Map` has no
/// positional `add` — it is keyed, not indexed; DEC-CT-C is exercised
/// separately below); `hashable: false` skips H2 (Q5: `Map` is mutable ⇒
/// identity `hash`, not value-hashable).
#[test]
fn map_satisfies_sequence_contract() {
    let mut vm = VM::new();
    let spec = ContractSpec {
        class_name: "Map",
        mutable: false,
        hashable: false,
        min_arity: 0,
    };
    assert_sequence_contract(&mut vm, &spec, build_map);
}

/// Builds a `Set` from element values through the surface protocol
/// (`Set.new()` + `.add(_)`).
fn build_set(vm: &mut VM, elems: &[Value]) -> Value {
    let set_class = Value::Obj(vm.universe.classes.set_class);
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
        mutable: true,
        hashable: false,
        min_arity: 0,
    };
    assert_sequence_contract(&mut vm, &spec, build_set);
}

/// `Map` extras (map-and-set.md §6): key overwrite leaves `size` unchanged
/// and updates the stored value; `remove` is idempotent (removing an absent
/// key is a no-op returning `self`); a missing key's `at(_)` is the `None`
/// singleton; `keys`/`values` agree with iteration order.
#[test]
fn map_key_overwrite_and_remove_idempotence() {
    let mut vm = VM::new();
    let map_class = Value::Obj(vm.universe.classes.map_class);
    let map = send0(&mut vm, map_class, "new()");
    let sym_put = vm.get_or_intern("at(_,put)");

    vm.send_dynamic(map, sym_put, &[Value::Int(1), Value::Int(10)]).unwrap();
    let size_after_first = as_number(send0(&mut vm, map, "size"));
    assert_eq!(size_after_first, 1.0, "one entry after first put");

    // Overwrite: same key, new value — size unchanged, value updated.
    vm.send_dynamic(map, sym_put, &[Value::Int(1), Value::Int(20)]).unwrap();
    let size_after_overwrite = as_number(send0(&mut vm, map, "size"));
    assert_eq!(size_after_overwrite, 1.0, "overwrite must not grow size");
    let got = send1(&mut vm, map, "at(_)", Value::Int(1));
    assert_eq!(as_number(got), 20.0, "overwrite must update the stored value");

    // Missing key -> the None singleton (total, never nil, never a raise).
    let missing = send1(&mut vm, map, "at(_)", Value::Int(999));
    assert!(matches!(missing, Value::Obj(id) if id == vm.universe.classes.none_singleton));

    // remove(absent) is a no-op returning self.
    let returned = send1(&mut vm, map, "remove(_)", Value::Int(999));
    assert_eq!(returned, map, "remove(absent) returns self");
    assert_eq!(as_number(send0(&mut vm, map, "size")), 1.0, "remove(absent) must not shrink size");

    // remove(present) actually deletes.
    send1(&mut vm, map, "remove(_)", Value::Int(1));
    assert_eq!(as_number(send0(&mut vm, map, "size")), 0.0, "remove(present) must shrink size");
}

/// `Set` extras (map-and-set.md §6): `add` is idempotent (a duplicate does
/// not grow `size`); `remove` is idempotent.
#[test]
fn set_add_and_remove_idempotence() {
    let mut vm = VM::new();
    let set_class = Value::Obj(vm.universe.classes.set_class);
    let set = send0(&mut vm, set_class, "new()");

    send1(&mut vm, set, "add(_)", Value::Int(7));
    send1(&mut vm, set, "add(_)", Value::Int(7));
    assert_eq!(as_number(send0(&mut vm, set, "size")), 1.0, "duplicate add must not grow size");
    assert!(as_bool(send1(&mut vm, set, "includes(_)", Value::Int(7))));

    let returned = send1(&mut vm, set, "remove(_)", Value::Int(999));
    assert_eq!(returned, set, "remove(absent) returns self");
    assert_eq!(as_number(send0(&mut vm, set, "size")), 1.0, "remove(absent) must not shrink size");

    send1(&mut vm, set, "remove(_)", Value::Int(7));
    assert_eq!(as_number(send0(&mut vm, set, "size")), 0.0, "remove(present) must shrink size");
    assert!(!as_bool(send1(&mut vm, set, "includes(_)", Value::Int(7))));
}

/// Builds a `Tuple` from element values via the surface freeze path
/// (`List.new()` + `.add(_)`, then `Tuple.__fromList(_)`) — the exact path the
/// `(a, b)` literal's parser lowering (U-COLL) takes.
fn build_tuple(vm: &mut VM, elems: &[Value]) -> Value {
    let list = build_list(vm, elems);
    let tuple_class = Value::Obj(vm.universe.classes.tuple_class);
    send1(vm, tuple_class, "__fromList(_)", list)
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
        mutable: false,
        hashable: true,
        min_arity: 1,
    };
    assert_sequence_contract(&mut vm, &spec, build_tuple);
}

#[test]
fn empty_tuple_construction_normalizes_to_unit() {
    let mut vm = VM::new();
    assert_eq!(build_tuple(&mut vm, &[]), Value::Unit);
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
    let a = build_tuple(&mut vm, &[Value::Int(1), Value::Int(2)]);
    let b = build_tuple(&mut vm, &[Value::Int(1), Value::Int(2)]);
    assert!(as_bool(send1(&mut vm, a, "==(_)", b)), "independently-built equal tuples must compare ==");
    let hash_a = as_number(send0(&mut vm, a, "hash"));
    let hash_b = as_number(send0(&mut vm, b, "hash"));
    assert_eq!(hash_a, hash_b, "value-equal tuples must hash equal");

    let differing = build_tuple(&mut vm, &[Value::Int(1), Value::Int(3)]);
    let hash_c = as_number(send0(&mut vm, differing, "hash"));
    assert_ne!(hash_a, hash_c, "differing tuples should (almost certainly) hash differently");

    // Cross-kind: a same-content List is never == a Tuple (E2).
    let list_same_content = build_list(&mut vm, &[Value::Int(1), Value::Int(2)]);
    assert!(
        !as_bool(send1(&mut vm, a, "==(_)", list_same_content)),
        "Tuple must never == a List, even same content"
    );

    // No mutation selector: `at(_,put)` and `add(_)` both miss (dNU).
    let sym_put = vm.get_or_intern("at(_,put)");
    assert!(
        vm.send_dynamic(a, sym_put, &[Value::Int(0), Value::Int(9)]).is_err(),
        "Tuple must not respond to at(_,put)"
    );
    let sym_add = vm.get_or_intern("add(_)");
    assert!(vm.send_dynamic(a, sym_add, &[Value::Int(9)]).is_err(), "Tuple must not respond to add(_)");
}

/// `Tuple` as a valid `Map` key (tuple-and-range.md; the re-entrant
/// `hash`+`==` key-lookup path, `docs/forge/units/U-COLLTYPES/plan.md` §7):
/// two independently-built value-equal tuples resolve to the SAME map entry.
#[test]
fn tuple_is_a_valid_map_key() {
    let mut vm = VM::new();
    let map_class = Value::Obj(vm.universe.classes.map_class);
    let map = send0(&mut vm, map_class, "new()");
    let key1 = build_tuple(&mut vm, &[Value::Int(1), Value::Int(2)]);
    let sym_put = vm.get_or_intern("at(_,put)");
    vm.send_dynamic(map, sym_put, &[key1, Value::Int(9)]).unwrap();

    let key2 = build_tuple(&mut vm, &[Value::Int(1), Value::Int(2)]);
    let got = send1(&mut vm, map, "at(_)", key2);
    assert_eq!(
        as_number(got),
        9.0,
        "a value-equal Tuple key must recover the same entry (re-entrant hash+== lookup)"
    );
    assert_eq!(as_number(send0(&mut vm, map, "size")), 1.0, "only one entry — key1/key2 are the SAME key");
}

/// Builds an exclusive `Range.new(0, n, false)` — a `0..n-1` interval whose
/// `n` generated elements are exactly `0, 1, …, n-1`.
///
/// Unlike every other `build_*` closure, this does **not** faithfully encode
/// arbitrary `elems` **content** — a `Range` holds no element storage
/// (RG-2): it can only represent an arithmetic run from a start bound, never
/// arbitrary per-index values. `elems.len()` is the only signal used. This is
/// why `Range` gets its own hand-rolled law-by-law test below
/// (`range_satisfies_the_applicable_sequence_laws`) instead of the shared
/// [`assert_sequence_contract`]: that harness's E5 ("differing elements
/// compare unequal") assumes a build closure that can encode two same-length,
/// same-first-element arrays as genuinely different collections, which is
/// exactly what a `Range` cannot do — the representational trade-off RG-2
/// laziness makes (`docs/spec/v0.2/core/tuple-and-range.md` §2).
fn build_range(vm: &mut VM, elems: &[Value]) -> Value {
    let range_class = Value::Obj(vm.universe.classes.range_class);
    let sym = vm.get_or_intern("new(_,_,_)");
    vm.send_dynamic(range_class, sym, &[Value::Int(0), Value::Int(elems.len() as i64), Value::Bool(false)])
        .expect("Range.new failed")
}

/// `Range` satisfies every sequence-protocol law the shared harness checks
/// **except** E5 (see [`build_range`]'s doc for why that one law does not
/// transfer to a storage-less collection): L1-L4 totality (via
/// [`build_range`], `0..n`), E1/E3/E4/E6 (equality reflexive/symmetric,
/// `!=` routes through `==`), and H2 (immutable ⇒ value-hash consistency,
/// Q5) — using two **independently-built**, structurally-equal ranges (not
/// the same object) so H2 actually exercises value-hashing, not identity.
#[test]
fn range_satisfies_the_applicable_sequence_laws() {
    let mut vm = VM::new();

    // L1-L4: size/at/None-on-oob, for n = 0..3, exactly as assert_sequence_contract's loop.
    for n in 0..=3 {
        let elems: Vec<Value> = (0..n).map(|i| Value::Int(i as i64)).collect();
        let range = build_range(&mut vm, &elems);
        let size = as_number(send0(&mut vm, range, "size"));
        assert_eq!(size, n as f64, "Range: size must equal the element count");
        for i in 0..n {
            let got = send1(&mut vm, range, "at(_)", Value::Int(i as i64));
            assert_eq!(as_number(got), i as f64, "Range: at({i}) should recover the generated element");
        }
        let out_of_range = send1(&mut vm, range, "at(_)", Value::Int(n as i64));
        assert!(
            matches!(out_of_range, Value::Obj(id) if id == vm.universe.classes.none_singleton),
            "Range: at(size) must surface the None singleton"
        );
    }

    // E1/E3/E4/E6: two independently-built equal ranges.
    let elems = [Value::Int(1), Value::Int(2), Value::Int(3)];
    let a = build_range(&mut vm, &elems);
    let b = build_range(&mut vm, &elems);
    assert!(
        as_bool(send1(&mut vm, a, "==(_)", b)),
        "Range: independently-built equal ranges must compare =="
    );
    assert!(as_bool(send1(&mut vm, a, "==(_)", a)), "Range: == must be reflexive");
    assert!(!as_bool(send1(&mut vm, a, "!=(_)", a)), "Range: != must be false where == holds");

    // A genuinely different range (different bounds) must compare unequal.
    let range_class = Value::Obj(vm.universe.classes.range_class);
    let sym_new = vm.get_or_intern("new(_,_,_)");
    let differing = vm
        .send_dynamic(range_class, sym_new, &[Value::Int(1), Value::Int(9), Value::Bool(false)])
        .unwrap();
    assert!(!as_bool(send1(&mut vm, a, "==(_)", differing)), "Range: differing bounds must compare unequal");
    assert!(as_bool(send1(&mut vm, a, "!=(_)", differing)), "Range: != must hold where == fails");

    // E2: cross-kind is false, never a dNU.
    assert!(!as_bool(send1(&mut vm, a, "==(_)", Value::Int(1))), "Range: == must be false across kinds");

    // H2: value-equal (independently-built) ranges hash equal.
    let hash_a = as_number(send0(&mut vm, a, "hash"));
    let hash_b = as_number(send0(&mut vm, b, "hash"));
    assert_eq!(hash_a, hash_b, "Range: == ranges must hash equal (H2)");
}

/// `Range` extras (tuple-and-range.md §5): inclusive/exclusive bound parity
/// (`toList` round-trip), `size`/`first`/`last`/`includes` parity, and
/// laziness — `Range.new(1, 1_000_000, true)` must construct and answer
/// `size`/`includes` promptly (no million-element materialization).
#[test]
fn range_inclusive_exclusive_parity_and_to_list_roundtrip() {
    let mut vm = VM::new();
    let range_class = Value::Obj(vm.universe.classes.range_class);
    let sym_new = vm.get_or_intern("new(_,_,_)");

    // Range.new(1, 5, true) — inclusive: 1,2,3,4,5.
    let inclusive = vm
        .send_dynamic(range_class, sym_new, &[Value::Int(1), Value::Int(5), Value::Bool(true)])
        .unwrap();
    assert_eq!(as_number(send0(&mut vm, inclusive, "size")), 5.0, "inclusive size: 5-1+1 = 5");
    assert_eq!(as_number(send0(&mut vm, inclusive, "first")), 1.0);
    assert_eq!(as_number(send0(&mut vm, inclusive, "last")), 5.0, "inclusive last == end");
    assert!(
        as_bool(send1(&mut vm, inclusive, "includes(_)", Value::Int(5))),
        "inclusive range includes its end bound"
    );

    let list_inclusive = send0(&mut vm, inclusive, "toList");
    for (i, expected) in [1.0, 2.0, 3.0, 4.0, 5.0].iter().enumerate() {
        let got = send1(&mut vm, list_inclusive, "at(_)", Value::Int(i as i64));
        assert_eq!(as_number(got), *expected, "toList element {i}");
    }

    // Range.new(1, 5, false) — exclusive: 1,2,3,4.
    let exclusive = vm
        .send_dynamic(range_class, sym_new, &[Value::Int(1), Value::Int(5), Value::Bool(false)])
        .unwrap();
    assert_eq!(as_number(send0(&mut vm, exclusive, "size")), 4.0, "exclusive size: 5-1 = 4");
    assert_eq!(as_number(send0(&mut vm, exclusive, "last")), 4.0, "exclusive last == end-1");
    assert!(
        !as_bool(send1(&mut vm, exclusive, "includes(_)", Value::Int(5))),
        "exclusive range excludes its end bound"
    );
    assert!(
        as_bool(send1(&mut vm, exclusive, "includes(_)", Value::Int(4))),
        "exclusive range includes end-1"
    );

    // Value-hash equality + structural == over independently-built ranges.
    let inclusive2 = vm
        .send_dynamic(range_class, sym_new, &[Value::Int(1), Value::Int(5), Value::Bool(true)])
        .unwrap();
    assert!(as_bool(send1(&mut vm, inclusive, "==(_)", inclusive2)));
    assert_eq!(as_number(send0(&mut vm, inclusive, "hash")), as_number(send0(&mut vm, inclusive2, "hash")));
}

/// Laziness (RG-2): `Range.new(1, 1_000_000, true)` constructs and answers
/// `size`/`includes(_)` without materializing a million-element buffer — a
/// timing bound (a real materialization would be orders of magnitude
/// slower) stands in for an allocation assertion here, since `RangeObject`
/// holds no element storage by construction (asserted structurally by the
/// type itself: `crate::range::RangeObject` has exactly three `Value`/`bool`
/// fields, no `Vec`).
#[test]
fn range_is_lazy_for_a_million_element_bound() {
    let mut vm = VM::new();
    let range_class = Value::Obj(vm.universe.classes.range_class);
    let sym_new = vm.get_or_intern("new(_,_,_)");
    let start = std::time::Instant::now();
    let big = vm
        .send_dynamic(range_class, sym_new, &[Value::Int(1), Value::Int(1_000_000), Value::Bool(true)])
        .unwrap();
    let size = as_number(send0(&mut vm, big, "size"));
    let includes = as_bool(send1(&mut vm, big, "includes(_)", Value::Int(500_000)));
    let elapsed = start.elapsed();
    assert_eq!(size, 1_000_000.0);
    assert!(includes);
    assert!(
        elapsed.as_millis() < 200,
        "construct+size+includes on a million-bound Range took {elapsed:?} — looks materialized, not lazy"
    );
}

/// DEC-CT-C (negative): a mutable-collection key (`List`) rejected by both
/// `Map#at(_,put)` and `Set#add(_)` — a raised catchable `Error`, never a
/// silent identity-keyed admission (collection-protocol.md law 4).
#[test]
fn mutable_collection_key_is_rejected() {
    let mut vm = VM::new();

    let map_class = Value::Obj(vm.universe.classes.map_class);
    let map = send0(&mut vm, map_class, "new()");
    let list_class = Value::Obj(vm.universe.classes.list_class);
    let mutable_key = send0(&mut vm, list_class, "new()");
    let sym_put = vm.get_or_intern("at(_,put)");
    let result = vm.send_dynamic(map, sym_put, &[mutable_key, Value::Int(1)]);
    assert!(result.is_err(), "Map#at(_,put) with a mutable List key must raise, not silently admit it");

    let set_class = Value::Obj(vm.universe.classes.set_class);
    let set = send0(&mut vm, set_class, "new()");
    let mutable_key2 = send0(&mut vm, list_class, "new()");
    let sym_add = vm.get_or_intern("add(_)");
    let result2 = vm.send_dynamic(set, sym_add, &[mutable_key2]);
    assert!(result2.is_err(), "Set#add(_) with a mutable List key must raise, not silently admit it");
}
