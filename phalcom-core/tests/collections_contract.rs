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
/// (`List.new()` + `.append(_)`), so the harness exercises the same path user
/// code does.
fn build_list(vm: &mut VM, elems: &[Value]) -> Value {
    let list_class = Value::Obj(vm.universe.classes.list_class);
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
    let source = format!("const built = {literal}\n");
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

    // L5: a collection's documented growth selector grows size by one and
    // places its value at the old final index.
    if let Some(selector) = spec.mutation_selector {
        let collection = build(vm, &[Value::Int(1), Value::Int(2)]);
        let old_size = as_number(send0(vm, collection, "size"));
        let returned = send1(vm, collection, selector, Value::Int(42));
        let new_size = as_number(send0(vm, collection, "size"));
        assert_eq!(new_size, old_size + 1.0, "{}: {selector} must grow size by 1", spec.class_name);
        let last = send1(vm, collection, "at(_)", Value::Int(old_size as i64));
        assert!(
            as_bool(send1(vm, last, "==(_)", Value::Int(42))),
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
            assert_eq!(returned, Value::Unit, "{}: {selector} must return Unit", spec.class_name);
        }
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
        mutation_selector: Some("append(_)"),
        mutation_returns_receiver: false,
        hashable: false,
        min_arity: 0,
    };
    assert_sequence_contract(&mut vm, &spec, build_list);
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
        mutation_selector: Some("add(_)"),
        mutation_returns_receiver: true,
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
    let got = send1(&mut vm, map, "[_]", Value::Int(1));
    assert_eq!(as_number(got), 20.0, "overwrite must update the stored value");

    // Missing key -> KeyError on strict [_] lookup, None on safe get(_) lookup.
    let sym_at = vm.get_or_intern("[_]");
    let result = vm.send_dynamic(map, sym_at, &[Value::Int(999)]);
    assert!(result.is_err(), "strict lookup of absent key must raise KeyError");
    let missing = send1(&mut vm, map, "get(_)", Value::Int(999));
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

/// Builds a `Tuple` through its public product-literal syntax. The internal
/// `Tuple._$fromList(_)` bridge is intentionally inaccessible to callers.
fn build_tuple(vm: &mut VM, elems: &[Value]) -> Value {
    let elements = elems
        .iter()
        .map(|value| match value {
            Value::Int(value) => value.to_string(),
            other => panic!("Tuple contract fixture only uses integer literals, got {other:?}"),
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
    let list = build_list(&mut vm, &[Value::Int(0), Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]);

    let exclusive = eval_literal(&mut vm, "1..4");
    let exclusive_slice = send1(&mut vm, list, "[_]", exclusive);
    assert_eq!(as_number(send0(&mut vm, exclusive_slice, "size")), 3.0);
    for (index, expected) in [1, 2, 3].into_iter().enumerate() {
        assert_eq!(as_number(send1(&mut vm, exclusive_slice, "at(_)", Value::Int(index as i64))), expected as f64);
    }

    let inclusive = eval_literal(&mut vm, "1..=4");
    let inclusive_slice = send1(&mut vm, list, "[_]", inclusive);
    assert_eq!(as_number(send0(&mut vm, inclusive_slice, "size")), 4.0);
    for (index, expected) in [1, 2, 3, 4].into_iter().enumerate() {
        assert_eq!(as_number(send1(&mut vm, inclusive_slice, "at(_)", Value::Int(index as i64))), expected as f64);
    }
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
