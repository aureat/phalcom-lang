//! `verify_invariants()` harness for the object model / metaclass tower.
//!
//! `docs/spec/object-model.md` §6 step 7 calls for a `verify_invariants()`
//! that runs after bootstrap and asserts every rule in §5. That function now
//! lives on [`Universe`] and is called from [`VM::new`], `.expect()`-ed since
//! a malformed kernel cannot run any program correctly. This file exercises
//! it end-to-end via a real [`VM`] (which runs the actual bootstrap in
//! [`Universe::create_core_classes`]) and asserts, via **handle identity** on
//! the live class graph, the full §5 apex table plus the parallel rule
//! ([ADR-0002](../../docs/adr/0002-metaclass-tower-parallel-rule.md)) and the
//! `Behavior` kernel class ([ADR-0003](../../docs/adr/0003-introduce-behavior-kernel-class.md)).
//!
//! Since [ADR-0009](../../docs/adr/0009-handle-arena-heap.md) every class is a
//! [`ClassId`] into the [`VM`]'s [`Heap`], and its metaclass (`class`) and
//! superclass links are plain [`ClassId`] handles. Object identity is
//! therefore a `==` on the `Copy` handle. Links are read through
//! [`Heap::class`].
//!
//! [`VM`]: phalcom_core::vm::VM
//! [`VM::new`]: phalcom_core::vm::VM::new
//! [`Universe`]: phalcom_core::universe::Universe
//! [`Universe::create_core_classes`]: phalcom_core::universe::Universe
//! [`ClassId`]: phalcom_core::heap::ClassId
//! [`Heap`]: phalcom_core::heap::Heap
//! [`Heap::class`]: phalcom_core::heap::Heap::class

use phalcom_core::primitive::nil::some_new;
use phalcom_core::value::{sentinel_to_option, Value};
use phalcom_core::vm::VM;

#[test]
fn surface_nil_is_unreachable_from_user_code() {
    // U6 Invariant 4 (values-and-absence.md §3;
    // [ADR-0007](../../docs/adr/0007-option-some-none.md) /
    // [ADR-0010](../../docs/adr/0010-tagged-value-enum.md)): the private
    // `Value::Nil` sentinel has no surface syntax. U6 removes the `nil`
    // keyword, so `nil` is merely an undefined identifier — no user program can
    // name, print, or compare the sentinel; it fails to compile instead.
    let mut vm = VM::new();
    let module = vm.create_module("main", "surface_nil_is_unreachable_from_user_code");
    // Compile + run directly (rather than `interpret_source`, whose diagnostic
    // path renders source spans) so the assertion is on the result, not on
    // reporting. `nil` is an undefined identifier, so the program errors.
    let result = vm.compile_closure(module, "System.print(nil)\n").and_then(|closure| vm.run_in_module(module, closure));
    assert!(result.is_err(), "surface `nil` must not resolve to any value — it is an undefined identifier");
}

#[test]
fn sentinel_surfaces_to_none_and_never_survives_as_nil() {
    // The read-boundary surfacer (`sentinel_to_option`) converts the private
    // sentinel to the shared `None` singleton, one-directionally. An
    // uninitialized slot therefore reads as `None`, never the raw `Value::Nil`,
    // and a non-sentinel value passes through untouched.
    let vm = VM::new();
    let none = vm.universe.classes.none_singleton;

    let surfaced = sentinel_to_option(Value::Nil, none);
    assert!(!matches!(surfaced, Value::Nil), "the sentinel must not survive surfacing");
    assert!(matches!(surfaced, Value::Obj(id) if id == none), "the sentinel must surface to the None singleton");

    let passthrough = sentinel_to_option(Value::Number(1.0), none);
    assert!(matches!(passthrough, Value::Number(n) if n == 1.0), "non-sentinel values must pass through unchanged");
}

#[test]
#[should_panic(expected = "Invariant 4")]
fn some_construction_never_wraps_the_sentinel() {
    // `some_new` asserts its argument is never the private sentinel — `None`
    // can never end up inside a `Some`. The raw sentinel is only reachable from
    // a hypothetical internal surfacing bug; feeding it in must trip the guard.
    let mut vm = VM::new();
    let _ = some_new(&mut vm, &Value::Number(0.0), &[Value::Nil]);
}

#[test]
fn verify_invariants_holds_after_bootstrap() {
    // VM::new() already calls verify_invariants() and would have panicked;
    // this test asserts it also succeeds when called again directly.
    let vm = VM::new();
    assert!(vm.universe.verify_invariants(&vm.heap).is_ok());
}

#[test]
fn metaclass_superclass_parallels_instance_superclass() {
    // object-model.md §5 rule 4 / ADR-0002: (X class).superclass ==
    // (X.superclass) class.
    let vm = VM::new();
    let number_class = vm.universe.classes.number_class;
    let object_class = vm.universe.classes.object_class;

    let number_meta = vm.heap.class(number_class).class;
    let object_meta = vm.heap.class(object_class).class;
    let number_meta_super = vm.heap.class(number_meta).superclass.expect("Number.class should have a superclass");

    assert_eq!(number_meta_super, object_meta, "Number.class.superclass should be Object.class");
}

#[test]
fn behavior_class_exists_in_tower() {
    // object-model.md §5 diagram / ADR-0003: Behavior is the shared abstract
    // superclass of Class and Metaclass, itself a subclass of Object.
    let vm = VM::new();
    let behavior_class = vm.universe.classes.behavior_class;
    let object_class = vm.universe.classes.object_class;
    let class_class = vm.universe.classes.class_class;
    let metaclass_class = vm.universe.classes.metaclass_class;

    assert_eq!(vm.heap.class(behavior_class).superclass, Some(object_class), "Behavior.superclass should be Object");
    assert_eq!(vm.heap.class(class_class).superclass, Some(behavior_class), "Class.superclass should be Behavior");
    assert_eq!(
        vm.heap.class(metaclass_class).superclass,
        Some(behavior_class),
        "Metaclass.superclass should be Behavior"
    );
}

#[test]
fn metaclass_responds_to_superclass_via_behavior() {
    // Behavior inheritance: `superclass` is installed once on Behavior
    // (universe.rs::install_primitives) and both Class and Metaclass inherit
    // it through the superclass chain, rather than each metaclass
    // special-casing its own accessor.
    use phalcom_core::class::lookup_method_in_hierarchy;
    use phalcom_core::method::{make_signature, SignatureKind};

    let mut vm = VM::new();
    let behavior_class = vm.universe.classes.behavior_class;
    let class_class = vm.universe.classes.class_class;
    let metaclass_class = vm.universe.classes.metaclass_class;

    let getter_sym = vm.get_or_intern(&make_signature("superclass", SignatureKind::Getter));

    assert!(
        vm.heap.class(behavior_class).get_method(getter_sym).is_some(),
        "Behavior should define superclass directly"
    );
    assert!(
        vm.heap.class(class_class).get_method(getter_sym).is_none(),
        "Class should not redefine superclass directly"
    );
    assert!(
        vm.heap.class(metaclass_class).get_method(getter_sym).is_none(),
        "Metaclass should not redefine superclass directly"
    );
    assert!(
        lookup_method_in_hierarchy(&vm.heap, class_class, getter_sym).is_some(),
        "Class should inherit superclass from Behavior"
    );
    assert!(
        lookup_method_in_hierarchy(&vm.heap, metaclass_class, getter_sym).is_some(),
        "Metaclass should inherit superclass from Behavior"
    );
}

#[test]
fn metaclass_is_instance_of_metaclass_class_closing_the_loop() {
    // object-model.md §5 apex table: Metaclass.class == Metaclass class, and
    // (Metaclass class).class == Metaclass — the loop closes through a
    // distinct row rather than Metaclass folding into itself (F6).
    let vm = VM::new();
    let metaclass = vm.universe.classes.metaclass_class;
    let metaclass_meta = vm.heap.class(metaclass).class;

    assert_ne!(metaclass_meta, metaclass, "Metaclass.class should be a distinct 'Metaclass class' row, not itself");
    assert_eq!(vm.heap.class(metaclass_meta).class, metaclass, "Metaclass.class.class should be Metaclass (closed loop)");
}

#[test]
fn class_is_instance_of_class_class_not_metaclass_directly() {
    // object-model.md §5 apex table: Class.class == Class class (a distinct
    // row), and (Class class).class == Metaclass — not Class.class ==
    // Metaclass directly (F6: the apex must not be 3-way collapsed).
    let vm = VM::new();
    let class_class = vm.universe.classes.class_class;
    let metaclass_class = vm.universe.classes.metaclass_class;
    let class_meta = vm.heap.class(class_class).class;

    assert_ne!(class_meta, metaclass_class, "Class.class should be 'Class class', not Metaclass directly");
    assert_eq!(vm.heap.class(class_meta).class, metaclass_class, "Class.class.class should be Metaclass");
}

#[test]
fn object_class_class_is_metaclass() {
    // object-model.md §5 sanity check: "Object.class.class == Metaclass".
    let vm = VM::new();
    let object_class = vm.universe.classes.object_class;
    let metaclass = vm.universe.classes.metaclass_class;
    let object_metaclass = vm.heap.class(object_class).class;
    assert_eq!(vm.heap.class(object_metaclass).class, metaclass, "Object.class.class should be Metaclass");
}

#[test]
fn object_has_no_superclass() {
    let vm = VM::new();
    let object_class = vm.universe.classes.object_class;
    assert!(vm.heap.class(object_class).superclass.is_none(), "Object.superclass should be None");
}

#[test]
fn core_classes_have_correct_metaclass_and_superclass() {
    // Every class created via `make_core_class` (Number, String, Nil, Bool,
    // Method, Symbol, Module, System):
    //   - X.class.class == Metaclass (every metaclass is instance-of Metaclass)
    //   - X.superclass == Object
    //   - X.class.superclass == Object.class (the parallel rule)
    let vm = VM::new();
    let metaclass = vm.universe.classes.metaclass_class;
    let object_class = vm.universe.classes.object_class;
    let object_meta = vm.heap.class(object_class).class;

    let core = [
        ("Number", vm.universe.classes.number_class),
        ("String", vm.universe.classes.string_class),
        ("Nil", vm.universe.classes.nil_class),
        ("Bool", vm.universe.classes.bool_class),
        ("Method", vm.universe.classes.method_class),
        ("Symbol", vm.universe.classes.symbol_class),
        ("Module", vm.universe.classes.module_class),
        ("System", vm.universe.classes.system_class),
    ];

    for (name, class) in core {
        let class_meta = vm.heap.class(class).class;
        assert_eq!(vm.heap.class(class_meta).class, metaclass, "{name}.class.class should be Metaclass");
        let sup = vm.heap.class(class).superclass.unwrap_or_else(|| panic!("{name}.superclass should be set"));
        assert_eq!(sup, object_class, "{name}.superclass should be Object");
        let meta_sup = vm.heap.class(class_meta).superclass.unwrap_or_else(|| panic!("{name}.class.superclass should be set"));
        assert_eq!(meta_sup, object_meta, "{name}.class.superclass should be Object.class (parallel rule)");
    }
}

#[test]
fn user_class_metaclass_superclass_parallels_instance_superclass() {
    // object-model.md §5 rule 4 / ADR-0002, exercised on a user-defined class
    // rather than a core class: (SomeUserClass.class).superclass ==
    // (SomeUserClass.superclass).class. Runs real source through the
    // compiler/VM (rather than constructing the class via the Rust API) so
    // this proves the parallel rule holds for classes built by `class { }`
    // syntax, not just `make_core_class`.
    let mut vm = VM::new();
    let module = vm.create_module("main", "user_class_metaclass_superclass_parallels_instance_superclass");
    vm.interpret_source(module, "class SomeUserClass {\n}\n").expect("class declaration should run without error");

    let name = vm.get_or_intern("SomeUserClass");
    let user_class = *vm.classes.get(&name).expect("SomeUserClass should be registered as a named class");
    let object_class = vm.universe.classes.object_class;

    let user_meta = vm.heap.class(user_class).class;
    let user_super = vm.heap.class(user_class).superclass.expect("SomeUserClass.superclass should be set");
    assert_eq!(user_super, object_class, "SomeUserClass.superclass should default to Object");

    let user_super_meta = vm.heap.class(user_super).class;
    let user_meta_super = vm.heap.class(user_meta).superclass.expect("SomeUserClass.class.superclass should be set");

    assert_eq!(
        user_meta_super, user_super_meta,
        "SomeUserClass.class.superclass should equal SomeUserClass.superclass.class (parallel rule)"
    );
}

#[test]
fn walking_metaclass_superclass_chain_terminates() {
    // object-model.md §5 sanity check: walking any metaclass's superclass
    // chain terminates (does not loop forever / dangle). Bounded walk guards
    // against an infinite loop turning into a hang instead of a failure.
    let vm = VM::new();
    let number_class = vm.universe.classes.number_class;
    let mut current = vm.heap.class(number_class).class;
    let mut steps = 0;
    loop {
        let next = vm.heap.class(current).superclass;
        steps += 1;
        assert!(steps < 64, "metaclass superclass chain did not terminate within 64 steps");
        match next {
            Some(n) => current = n,
            None => break,
        }
    }
}
