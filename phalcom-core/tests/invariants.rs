//! `verify_invariants()` harness for the object model / metaclass tower.
//!
//! `docs/spec/object-model.md` §6 step 7 calls for a `verify_invariants()`
//! that runs after bootstrap and asserts every rule in §5. That function now
//! lives on [`Universe`] and is called from [`VM::new`], `.expect()`-ed since
//! a malformed kernel cannot run any program correctly. This file exercises
//! it end-to-end via a real [`VM`] (which runs the actual bootstrap in
//! [`Universe::create_core_classes`]) and asserts, via **handle identity** on
//! the live class graph, the full §5 apex table plus the parallel rule
//! ([ADR-0002](../../docs/adr/accepted/0002-metaclass-tower-parallel-rule.md)) and the
//! `Behavior` kernel class ([ADR-0003](../../docs/adr/accepted/0003-introduce-behavior-kernel-class.md)).
//!
//! Since [ADR-0009](../../docs/adr/accepted/0009-handle-arena-heap.md) every class is a
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

use phalcom_core::error::{PhError, RuntimeError};
use phalcom_core::heap::ClassId;
use phalcom_core::heap::lookup_method_in_hierarchy;
use phalcom_core::interner::Symbol;
use phalcom_core::primitive::block::{block_arity, block_call, block_name};
use phalcom_core::primitive::boolean::bool_hash;
use phalcom_core::primitive::class::{behavior_methods, behavior_name};
use phalcom_core::primitive::method::method_bind;
use phalcom_core::primitive::nil::some_call;
use phalcom_core::primitive::number::number_hash;
use phalcom_core::primitive::object::{object_hash, object_method_for};
use phalcom_core::primitive::string::string_hash;
use phalcom_core::primitive::symbol::symbol_hash;
use phalcom_core::value::{Value, sentinel_to_option};
use phalcom_core::vm::{ClassKey, VM};
use std::collections::HashSet;

/// The 21 named kernel classes (`CoreClasses` rows), paired with a stable name.
///
/// Used by the R-INV-0.x audit substrate to enumerate every class whose own —
/// or whose metaclass's own — method dictionary can carry a floor binding.
fn core_class_rows(vm: &VM) -> [(&'static str, ClassId); 35] {
    let c = vm.universe.classes;
    [
        ("Object", c.object_class),
        ("Behavior", c.behavior_class),
        ("Class", c.class_class),
        ("Metaclass", c.metaclass_class),
        ("Number", c.number_class),
        ("String", c.string_class),
        ("Nil", c.nil_class),
        ("Bool", c.bool_class),
        ("True", c.true_class),
        ("False", c.false_class),
        ("Method", c.method_class),
        ("Function", c.function_class),
        ("Closure", c.closure_class),
        ("BoundMethod", c.bound_method_class),
        ("MethodFamily", c.method_family_class),
        ("BoundMethodFamily", c.bound_method_family_class),
        ("Symbol", c.symbol_class),
        ("Module", c.module_class),
        ("System", c.system_class),
        ("Option", c.option_class),
        ("Some", c.some_class),
        ("None", c.none_class),
        ("Unit", c.unit_class),
        ("List", c.list_class),
        // U-COLLTYPES Phase 1 (ADR-0039): Map/Set join the audited census.
        ("Map", c.map_class),
        ("Set", c.set_class),
        // U-COLLTYPES Phase 2 (ADR-0039): Tuple joins the audited census.
        ("Tuple", c.tuple_class),
        ("Record", c.record_class),
        // U-COLLTYPES Phase 3 (ADR-0039): Range joins the audited census.
        ("Range", c.range_class),
        // U-BYTES (PDR-0011): Bytes joins the audited census at birth — the
        // CB-5 lesson: a class absent from the census is a class the
        // ADR-0019 freeze does not bind.
        ("Bytes", c.bytes_class),
        ("Message", c.message_class),
        ("Error", c.error_class),
        ("MessageNotUnderstood", c.message_not_understood_class),
        // `Family` (U16-Open, ADR-0047): the `::` method-reference call
        // router joins the audited census.
        ("Family", c.family_class),
        // `Fiber` (ADR-0030) joins the audited census 2026-07-15 (DEFERRED
        // CB-5). It shipped with 11 primitives but was never listed here, so
        // `install_primitives` bound a whole kernel class that R-INV-0.1 did
        // not audit and `floor-census.md` did not mention: the ADR-0019 freeze
        // did not bind `Fiber`, and adding or dropping a fiber primitive
        // changed the floor with no red test. The census and the test agreed
        // with each other (125 = 125, green), which is exactly why nothing
        // caught it — neither was ever compared against the install site.
        ("Fiber", c.fiber_class),
    ]
}

/// Extracts the `f64` behind a `Number` result (test-local helper).
fn as_number(value: Value) -> f64 {
    if let Some(n) = value.as_int() {
        n as f64
    } else if let Some(n) = value.as_float() {
        n
    } else {
        panic!("expected a number, got {value:?}");
    }
}

/// Sends a nullary selector to `receiver` and returns the result value.
fn send0(vm: &mut VM, receiver: Value, selector: &str) -> Value {
    let sym = vm.get_or_intern(selector);
    vm.send_dynamic(receiver, sym, &[]).unwrap_or_else(|_| panic!("send `{selector}` failed"))
}

/// Sends a unary selector to `receiver` with one argument.
fn send1(vm: &mut VM, receiver: Value, selector: &str, arg: Value) -> Value {
    let sym = vm.get_or_intern(selector);
    vm.send_dynamic(receiver, sym, &[arg]).unwrap_or_else(|_| panic!("send `{selector}` failed"))
}

#[test]
fn surface_nil_is_unreachable_from_user_code() {
    // U6 Invariant 4 (values-and-absence.md §3;
    // [ADR-0007](../../docs/adr/accepted/0007-option-some-none.md) /
    // [ADR-0010](../../docs/adr/accepted/0010-tagged-value-enum.md)): the private
    // `Value::nil()` sentinel has no surface syntax. U6 removes the `nil`
    // keyword, so `nil` is merely an undefined identifier — no user program can
    // name, print, or compare the sentinel; it fails to compile instead.
    let mut vm = VM::new();
    let module = vm.create_module("main", "surface_nil_is_unreachable_from_user_code");
    // Compile + run directly (rather than `interpret_source`, whose diagnostic
    // path renders source spans) so the assertion is on the result, not on
    // reporting. `nil` is an undefined identifier, so the program errors.
    let result = vm
        .compile_closure(module, "System.print(nil)\n")
        .and_then(|closure| vm.run_in_module(module, closure));
    assert!(result.is_err(), "surface `nil` must not resolve to any value — it is an undefined identifier");
}

#[test]
fn sentinel_surfaces_to_none_at_read_boundaries() {
    let surfaced = sentinel_to_option(phalcom_core::value::NIL);
    assert!(surfaced.is_none(), "the sentinel must surface to immediate None");

    let passthrough = sentinel_to_option(Value::int(1));
    assert_eq!(passthrough.as_int(), Some(1), "non-sentinel values must pass through unchanged");
}

#[test]
fn expression_result_absence_surfaces_to_none() {
    // U6 Invariant 4 (values-and-absence.md §3; ADR-0007/ADR-0010): an absent
    // *expression result* — one produced by a surface-reachable primitive whose
    // value flows into an arg or `print` without crossing a `Get*` read boundary
    // — must be the `None` singleton, never the raw `Value::nil()` sentinel. This
    // exercises the reviewer's leak set at the Value level by calling each fixed
    // primitive directly (`run_in_module` discards the top-level result, so it
    // cannot observe the value; the four programs' printed forms are pinned by
    // the `absence` golden corpus). The inliner's `Bytecode::Nil` result site
    // shares its target with these primitives — see those goldens.
    use phalcom_core::primitive::block::block_call;
    use phalcom_core::primitive::boolean::{bool_if_false, bool_if_true};
    use phalcom_core::primitive::class::class_superclass;
    use phalcom_core::primitive::system::system_class_print;

    let mut vm = VM::new();
    let is_none = |value: Value, label: &str| {
        assert!(!value.is_nil(), "{label} leaked the raw sentinel");
        assert!(value.is_none(), "{label} should yield immediate None");
    };

    let unused_block = Value::int(0);

    // `false.ifTrue { .. }` / `true.ifFalse { .. }` — branch not taken.
    is_none(bool_if_true(&mut vm, &Value::bool(false), &[unused_block]).expect("ifTrue"), "false.ifTrue");
    is_none(bool_if_false(&mut vm, &Value::bool(true), &[unused_block]).expect("ifFalse"), "true.ifFalse");

    // `System.print(_)` — surface-reachable send result.
    is_none(system_class_print(&mut vm, &Value::int(1), &[Value::int(1)]).expect("print"), "System.print");

    // `Object.superclass` — the root class has no superclass.
    let object = Value::obj(vm.universe.classes.object_class);
    is_none(class_superclass(&mut vm, &object, &[]).expect("superclass"), "Object.superclass");

    let module = vm.create_module("blk", "expression_result_absence_surfaces_to_none");
    vm.interpret_source(module, "let blk = || { }\n").expect("define an empty block global");
    let blk_sym = vm.interner.intern("blk");
    let empty_block = vm.heap.module(module).get(blk_sym).expect("the `blk` global must exist");
    is_none(block_call(&mut vm, &empty_block, &[]).expect("empty block call"), "{ }.call()");
}

#[test]
fn some_construction_never_wraps_the_sentinel() {
    let mut vm = VM::new();
    let some_class = Value::obj(vm.universe.classes.some_class);
    let error = some_call(&mut vm, &some_class, &[phalcom_core::value::NIL]).expect_err("Some must reject the private Nil sentinel");
    assert!(error.to_string().contains("private Nil"), "unexpected error: {error}");
}

#[test]
fn some_can_wrap_none() {
    let mut vm = VM::new();
    let none = Value::none();
    let some_class = Value::obj(vm.universe.classes.some_class);
    let result = some_call(&mut vm, &some_class, &[none]);
    assert!(result.is_ok(), "Some(None) must succeed — None is a legal immediate payload");
}

#[test]
fn verify_invariants_holds_after_bootstrap() {
    // VM::new() already calls verify_invariants() and would have panicked;
    // this test asserts it also succeeds when called again directly.
    let vm = VM::new();
    assert!(vm.universe.verify_invariants(&vm.heap).is_ok());
}

#[test]
fn native_rest_primitives_share_the_method_dictionary_and_rest_index() {
    // Callable-runtime follow-up to F.3: a rest Method is one direct class
    // definition represented in two indexes. `methods` owns the structural
    // selector used by reflection / `::`; `rest_methods` is only the
    // base-family fallback accelerator. Native rest gateways must obey the
    // same invariant as bytecode rest methods.
    let mut vm = VM::new();
    let c = vm.universe.classes;
    let rows = [
        (c.object_class, "perform(_,***)", "perform"),
        (c.method_class, "invokeOn(_,***)", "invokeOn"),
        (c.function_class, "call(***)", "call"),
    ];

    for (class, selector_text, base_text) in rows {
        let selector = vm.get_or_intern(selector_text);
        let base = vm.get_or_intern(base_text);
        let exact = vm
            .heap
            .class(class)
            .methods
            .get(&selector)
            .copied()
            .unwrap_or_else(|| panic!("{selector_text} is missing from ClassObject.methods"));

        assert_eq!(
            vm.heap.class(class).get_rest_method(base),
            Some(exact),
            "{selector_text} must name the same Method in methods and rest_methods"
        );
        assert!(
            vm.heap
                .class(class)
                .base_names
                .get(&base)
                .is_some_and(|selectors| selectors.contains(&selector)),
            "{selector_text} must participate in the finalized base-name family index"
        );
    }
}

#[test]
fn sealed_hierarchy_rejects_runtime_reparent_and_keeps_invariants() {
    // U13 / DEC-U13a=A ([ADR-0026](../../docs/adr/accepted/0026-class-hierarchy-mutability.md),
    // [ADR-0041](../../docs/adr/accepted/0041-hierarchy-stability-policy.md)): a class's
    // `superclass` is sealed at creation — `class_set_superclass` (the
    // `superclass=(_)` primitive, installed on `Behavior`) always errors and
    // never mutates the class graph, so `ClassId`-keyed dispatch and the
    // ADR-0011 fixed instance slot layout stay provably stable. This asserts
    // the reject is a clean, typed [`RuntimeError::InvalidSetSuper`] (never a
    // panic), that the attempted reparent leaves `Dog.superclass` untouched,
    // and that `verify_invariants()` is still green *after* the rejected
    // mutation — the tower cannot be silently corrupted by a reparent attempt.
    use phalcom_core::primitive::class::class_set_superclass;

    let mut vm = VM::new();
    let module = vm.create_module("main", "test");
    let object_class = vm.universe.classes.object_class;
    let animal = vm.create_class(module, "Animal", Some(object_class));
    let dog = vm.create_class(module, "Dog", Some(animal));
    let cat = vm.create_class(module, "Cat", Some(object_class));

    let dog_value = Value::obj(dog);
    let result = class_set_superclass(&mut vm, &dog_value, &[Value::obj(cat)]);

    match result {
        Err(PhError::Runtime(RuntimeError::InvalidSetSuper)) => {}
        other => panic!("expected RuntimeError::InvalidSetSuper, got {other:?}"),
    }

    assert_eq!(
        vm.heap.class(dog).superclass,
        Some(animal),
        "a rejected superclass= must not mutate the class graph — Dog.superclass should stay Animal"
    );

    assert!(
        vm.universe.verify_invariants(&vm.heap).is_ok(),
        "verify_invariants() must still pass after a rejected reparent attempt"
    );
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
fn user_subclass_metaclass_parallels_superclass() {
    // U-INH §3.3 / ADR-0002 rule 4 — the user-class analogue of
    // `metaclass_superclass_parallels_instance_superclass`. A class created
    // with an explicit superclass has BOTH `B.superclass == A` AND
    // `B.class.superclass == A.class`; the second link is what makes `static`
    // and `construct` members inherit across `extends`. `create_class` is the
    // single site that maintains rule 4 (DEC-INH-E), so both a surface
    // `class B is A` and a reflective creation stay parallel.
    let mut vm = VM::new();
    let module = vm.create_module("main", "test");
    let object_class = vm.universe.classes.object_class;
    let animal = vm.create_class(module, "Animal", Some(object_class));
    let dog = vm.create_class(module, "Dog", Some(animal));

    assert_eq!(vm.heap.class(dog).superclass, Some(animal), "Dog.superclass should be Animal");

    let dog_meta = vm.heap.class(dog).class;
    let animal_meta = vm.heap.class(animal).class;
    let dog_meta_super = vm.heap.class(dog_meta).superclass.expect("Dog.class should have a superclass");
    assert_eq!(dog_meta_super, animal_meta, "Dog.class.superclass should be Animal.class");
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

    assert_eq!(
        vm.heap.class(behavior_class).superclass,
        Some(object_class),
        "Behavior.superclass should be Object"
    );
    assert_eq!(
        vm.heap.class(class_class).superclass,
        Some(behavior_class),
        "Class.superclass should be Behavior"
    );
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
    use phalcom_core::heap::lookup_method_in_hierarchy;
    use phalcom_core::method::{SignatureKind, make_signature};

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

    assert_ne!(
        metaclass_meta, metaclass,
        "Metaclass.class should be a distinct 'Metaclass class' row, not itself"
    );
    assert_eq!(
        vm.heap.class(metaclass_meta).class,
        metaclass,
        "Metaclass.class.class should be Metaclass (closed loop)"
    );
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
    // Every direct-`Object` subclass created via `make_core_class` (Number,
    // String, Nil, Bool, Symbol, Module, System):
    //   - X.class.class == Metaclass (every metaclass is instance-of Metaclass)
    //   - X.superclass == Object
    //   - X.class.superclass == Object.class (the parallel rule)
    // `Method` is deliberately excluded: it remains a reified dispatch object
    // directly under `Object`; its shape is asserted by the dedicated Method
    // reflection invariant and the general parallel-rule check.
    let vm = VM::new();
    let metaclass = vm.universe.classes.metaclass_class;
    let object_class = vm.universe.classes.object_class;
    let object_meta = vm.heap.class(object_class).class;

    let core = [
        ("Number", vm.universe.classes.number_class),
        ("String", vm.universe.classes.string_class),
        ("Nil", vm.universe.classes.nil_class),
        ("Bool", vm.universe.classes.bool_class),
        ("Symbol", vm.universe.classes.symbol_class),
        ("Module", vm.universe.classes.module_class),
        ("System", vm.universe.classes.system_class),
    ];

    for (name, class) in core {
        let class_meta = vm.heap.class(class).class;
        assert_eq!(vm.heap.class(class_meta).class, metaclass, "{name}.class.class should be Metaclass");
        let sup = vm.heap.class(class).superclass.unwrap_or_else(|| panic!("{name}.superclass should be set"));
        assert_eq!(sup, object_class, "{name}.superclass should be Object");
        let meta_sup = vm
            .heap
            .class(class_meta)
            .superclass
            .unwrap_or_else(|| panic!("{name}.class.superclass should be set"));
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
    vm.interpret_source(module, "class SomeUserClass {\n}\n")
        .expect("class declaration should run without error");

    let name = vm.get_or_intern("SomeUserClass");
    let user_class = *vm
        .classes
        .get(&ClassKey { module, name })
        .expect("SomeUserClass should be registered as a named class");
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

#[test]
fn subclass_field_offset_stability() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "subclass_field_offset_stability");

    // 1. Compile and define Base
    let base_code = "
        class Base {
            @constructor
            new() {
                _name = \"Base\"
                _other = 42
            }
        }
    ";
    vm.interpret_source(module, base_code).expect("Base definition should succeed");

    // 2. Compile and run Subclass, declaring `extends Base` directly rather
    // than pre-registering a class object via `create_class` (U-CLASSCLOSE:
    // classes are closed, so there is no "reopen keeps the Rust stub's
    // established superclass" path outside the core module — see decision
    // 0065 ruling 4 — and `create_class` itself already wires field layout
    // from `vm.field_layouts` once it exists, so no manual copy is needed
    // either; `Bytecode::Class`'s own `create_class` call does it).
    let sub_code = "
        class Subclass is Base {
            @constructor
            new() {
                _name = \"Subclass\"
            }
        }
    ";
    let closure = vm.compile_closure(module, sub_code).expect("Subclass compilation should succeed");
    vm.run_in_module(module, closure).expect("Subclass execution should succeed");

    let name_sym = vm.get_or_intern("_name");
    let other_sym = vm.get_or_intern("_other");
    let base_sym = vm.get_or_intern("Base");
    let sub_sym = vm.get_or_intern("Subclass");

    let base_cls = *vm.classes.get(&ClassKey { module, name: base_sym }).unwrap();
    let sub_cls = *vm.classes.get(&ClassKey { module, name: sub_sym }).unwrap();

    let base_layout = vm.heap.class(base_cls);
    let sub_layout = vm.heap.class(sub_cls);

    assert_eq!(base_layout.field_slots.get(&name_sym).copied(), Some(0));
    assert_eq!(base_layout.field_slots.get(&other_sym).copied(), Some(1));
    assert_eq!(base_layout.field_count, 2);

    assert_eq!(sub_layout.field_slots.get(&name_sym).copied(), Some(2));
    assert_eq!(sub_layout.field_count, 3);
}

#[test]
fn subclass_static_field_offset_stability() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "subclass_static_field_offset_stability");

    // 1. Compile and define Base
    let base_code = "
        class Base {
            @class _count = 10
        }
    ";
    vm.interpret_source(module, base_code).expect("Base definition should succeed");

    // 2. Compile and run Subclass, declaring `extends Base` directly rather
    // than pre-registering a class object via `create_class` (U-CLASSCLOSE:
    // classes are closed, so there is no "reopen keeps the Rust stub's
    // established superclass" path outside the core module — see decision
    // 0065 ruling 4 — and `create_class` itself already wires field layout
    // from `vm.field_layouts` once it exists, so no manual copy is needed
    // either; `Bytecode::Class`'s own `create_class` call does it).
    let sub_code = "
        class Subclass is Base {
            @class _count = 20
        }
    ";
    let closure = vm.compile_closure(module, sub_code).expect("Subclass compilation should succeed");
    vm.run_in_module(module, closure).expect("Subclass execution should succeed");

    let count_sym = vm.get_or_intern("_count");
    let base_sym = vm.get_or_intern("Base");
    let sub_sym = vm.get_or_intern("Subclass");

    let base_cls = *vm.classes.get(&ClassKey { module, name: base_sym }).unwrap();
    let sub_cls = *vm.classes.get(&ClassKey { module, name: sub_sym }).unwrap();

    let base_meta = vm.heap.class(base_cls).class;
    let sub_meta = vm.heap.class(sub_cls).class;

    let base_meta_layout = vm.heap.class(base_meta);
    let sub_meta_layout = vm.heap.class(sub_meta);

    assert_eq!(base_meta_layout.field_slots.get(&count_sym).copied(), Some(0));
    assert_eq!(base_meta_layout.field_count, 1);

    assert_eq!(sub_meta_layout.field_slots.get(&count_sym).copied(), Some(1));
    assert_eq!(sub_meta_layout.field_count, 2);
}

// ---------------------------------------------------------------------------
// R-INV-0.x — the audit substrate (U-CORE-1 spec §5.1). These are the shared
// invariants every later U-CORE unit extends: the census set/count audit
// (0.1), plus the corpus half of the parallel-rule / absence / fixed-slot
// checks whose boot half rides `Universe::verify_invariants` (0.2/0.3/0.4).
// ---------------------------------------------------------------------------

#[test]
fn floor_census_matches_installed_bindings() {
    // R-INV-0.1 — reconstruct the installed `(class, selector)` floor from a
    // live `VM::new()` and assert it equals the census in
    // `docs/spec/core/floor-census.md` (count = 88 after ADR-0023's +7,
    // ADR-0028's +5, U-CORE-4's +1, and U-CORE-6's own +2). Turns silent
    // floor drift — an accidental extra primitive, or a dropped one — into a
    // red test. The baseline is 73; the first +7 (marked NEW below) is the
    // ADR-0023 kernel-reflection amendment (`hash` ×5 + `Behavior#name` +
    // `Behavior#methods`); the second +5 (marked NEW_METHOD_REFLECTION) is
    // ADR-0028's `Method` reflection surface (U-CORE-3):
    // `Object#methodFor(_)`, `Method#invokeOn(_,***)`, `Method#bind(_)`,
    // `Method#selector`, `Method#holder`; the third +1 (marked
    // NEW_VALUE_TOSTRING) is U-CORE-4's amendment (`Number#toString`); the
    // fourth +2 (marked NEW_ERROR) is U-CORE-6's amendment (ADR-0037):
    // `Error#message`, `Error#raise`.
    // U-COLLTYPES Phase 1 (ADR-0039): Map (8: new/size_/get_/put_/
    // has_/remove_/keyAt_/valueAt_) + Set (6: new/size_/add_/
    // has_/remove_/at_) = +14 (88 -> 102).
    // U-COLLTYPES Phase 2 (ADR-0039): Tuple (3: fromList/size_/at_) = +3
    // (102 -> 105). No mutation primitive — immutability is structural.
    // U-COLLTYPES Phase 3 (ADR-0039): Range originally contributed four
    // bindings. C.2 superseded that construction surface with direct
    // `BuildRange` bytecode and three raw observers.
    // U-ERR (ADR-0038, this unit's own amendment): `Block#on(_,_)` +
    // `Block#ensure(_)` = +2 (109 -> 111) — the catch protocol `try`/`on`/
    // `catch`/`ensure` (ADR-0031) desugar to. The WHOLE remaining error
    // surface — `throw`, the `try` statement, `Result`/`Ok`/`Err`, `attempt` —
    // is `.ph`/parser sugar over these two plus the pre-existing `Error#raise`
    // (U-CORE-6/ADR-0037), so this is the floor's final word on error
    // handling.
    // U15 (ADR-0045, this unit's own amendment): `Module#doesNotUnderstand(_)`
    // = +1 (111 -> 112) — member access (`math.pi`, `math.distance(1, 2)`) as
    // an ordinary send over the module's own globals table; the only way to
    // reach it from a message send (floor-census.md §2.12).
    // U16-Open's former Family dNU router (ADR-0047) is superseded by the
    // shared shape-aware Function gateway. Family's six reflective protocol
    // bindings remain native; its call activation rebuilds the target selector
    // directly.
    // U-SCHED (floor-census.md amendment, ADR-0030 §Consequences): the
    // native ready-queue scheduler seam admits **+2** bindings (113 -> 115):
    // `System::schedule(_)` (`system_schedule`) and `System::nextScheduled`
    // (`system_next_scheduled`), both `primitive/system.rs`.
    // U-ANNOT-CONTRACTS (ADR-0052 Fix 1, this unit's own amendment): the
    // `@invariant` re-entrancy guard admits **+2** bindings (115 -> 117):
    // `Object::__invariantEnter()` and `Object::__invariantExit()`, both
    // `primitive/object.rs` — the native pair the woven prologue/epilogue
    // call, never `.ph`-authored.
    // M-ATTR-ROOT (attribute-classes.md, this unit's own amendment): the
    // attribute-retention mechanism admits **+3** bindings (117 -> 120):
    // `Object#__attach(_)`, `Object#__attributes`, `Object#__freezeAttributes()`,
    // all `primitive/attribute.rs` — the native pair (triple) the compiler's
    // `@Name(args)` desugar (`compiler::attributes`/`compiler::lib::class_decl`)
    // calls, never `.ph`-authored.
    // U-GC Step 3: System.gc = +1 (120 -> 121)
    // U-STRING (ADR-0049 amendment): raw byte-level string accessors +
    // raw stdout write = +4 (121 -> 125):
    // `String#byteCount_`, `String#byteAt_(_)`, `String#slice_(_,_)`,
    // `System.write_(_)` — all `primitive/string.rs`/`primitive/system.rs`.
    // `Fiber` (ADR-0030) — admitted to the census 2026-07-15 (DEFERRED CB-5).
    // The first 11 were NOT a floor amendment: they had been installed since the
    // fiber work landed. What changed is that they became *audited*. Before
    // this, `Fiber` was absent from `core_class_rows` and from
    // `floor-census.md` entirely, so the ADR-0019 freeze did not bind it
    // (125 -> 136): `Fiber.new(_)`, `#call()`/`#call(_)`, `#try()`/`#try(_)`,
    // `Fiber.yield()`/`Fiber.yield(_)`, `Fiber.current`, `Fiber.abort(_)`,
    // `#isDone`, `#error` — all `primitive/fiber.rs`, bound at
    // `universe/primitives.rs`. `call`/`try`/`yield` are each shared across two
    // arities.
    //
    // The 12th, `#isRoot`, **is** a floor amendment (136 -> 137, 9 distinct
    // native fns): the predicate form of `Fiber::yield`'s root refusal, added
    // 2026-07-19 to fix E004. `Future#await` had no way to ask whether it was
    // allowed to suspend, so it attempted a yield and inspected the failure —
    // but the `{ … }.attempt()` wrapper it used to catch that failure is itself
    // two native re-entrant frames, so the probe tripped the restricted-yield
    // guard it was probing for and `await` could never suspend any fiber.
    // Justified against the ADR-0019 freeze on the ground that attempt-and-
    // inspect is unfixable in `.ph` when the attempt changes the answer: no
    // arrangement of library code can observe root-ness without a predicate.
    // U-BYTES (PDR-0011 ruling 3, this unit's own amendment): the kernel
    // octet buffer admits **+10** bindings (137 -> 147): `Bytes.new(_)`,
    // `Bytes.fromString_(_)`, `size_`, `at_(_)`, `set_(_,_)`, `fill_(_)`,
    // `slice_(_,_)`, `copyInto_(_,_)`, `utf8_`, `equalsConstantTime_(_)` —
    // all `primitive/bytes.rs`, under the container bulk-op posture (bulk
    // no-user-code ops native; block-taking selectors stay `.ph` so
    // `Fiber#yield` works mid-iteration, bytes.md §3.1).
    // PDR-0013 ruling 4 (ships with U-BYTES, censused to its own record):
    // `utf8Lossy_` (147 -> 148) — total lossy decode for `Path#toString`;
    // display-only, never round-tripped into data.
    // Spec A product foundations replace the public Tuple.fromList(_) floor
    // entry with internal __fromList(_), add four lane observers, and admit
    // three immutable Record observers: +7 (151 -> 158).
    // C.2 retires `Range class#new(_,_,_)`; C.3 admits two underivable
    // operations: Tuple#slice_ reconstructs a label-preserving immutable
    // product, and List#replaceSlice_ mutates a variable-length span while
    // preserving receiver identity. Combined delta: +1 (158 -> 159).

    let mut vm = VM::new();
    let c = vm.universe.classes;

    // `(class, is_static, human-canonical selector)`. `is_static` bindings are
    // installed on the class's metaclass. Selectors are the interned `_:` form
    // (floor-census §1.2).
    let bindings: Vec<(ClassId, bool, &str)> = vec![
        // §2.1 Object
        (c.object_class, false, "name"),
        (c.object_class, false, "class"),
        (c.object_class, false, "class=(put)"),
        (c.object_class, false, "toString"),
        (c.object_class, false, "hash"), // NEW (ADR-0023)
        (c.object_class, false, "==(_)"),
        (c.object_class, false, "!=(_)"),
        (c.object_class, false, "perform(_,***)"),
        (c.object_class, false, "respondsTo(_)"),
        (c.object_class, false, "doesNotUnderstand(_)"),
        (c.object_class, false, "methodFor(_)"),         // NEW (ADR-0028)
        (c.object_class, false, "_$invariantEnter()"),   // NEW_INVARIANT_GUARD (ADR-0052)
        (c.object_class, false, "_$invariantExit()"),    // NEW_INVARIANT_GUARD (ADR-0052)
        (c.object_class, false, "_$attach(_)"),          // NEW_ATTR_ROOT (M-ATTR-ROOT)
        (c.object_class, false, "_$attributes"),         // NEW_ATTR_ROOT (M-ATTR-ROOT)
        (c.object_class, false, "_$freezeAttributes()"), // NEW_ATTR_ROOT (M-ATTR-ROOT)
        // §2.2 Behavior
        (c.behavior_class, false, "superclass"),
        (c.behavior_class, false, "superclass=(put)"),
        (c.behavior_class, false, "name"),    // NEW (ADR-0023)
        (c.behavior_class, false, "methods"), // NEW (ADR-0023)
        (c.behavior_class, false, ">>(_)"),   // selector-pattern reflection
        // §2.3 Class
        (c.class_class, false, "+(_)"),
        (c.class_class, false, "_$new()"),
        // §2.4 Number
        (c.number_class, false, "+(_)"),
        (c.number_class, false, "-(_)"),
        (c.number_class, false, "*(_)"),
        (c.number_class, false, "/(_)"),
        (c.number_class, false, "%(_)"),
        (c.number_class, false, "~/(_)"),
        (c.number_class, false, "**(_)"),
        (c.number_class, false, "<(_)"),
        (c.number_class, false, "<=(_)"),
        (c.number_class, false, ">(_)"),
        (c.number_class, false, ">=(_)"),
        (c.number_class, false, "negated()"),
        (c.number_class, false, "hash"),     // NEW (ADR-0023)
        (c.number_class, false, "toString"), // NEW_VALUE_TOSTRING (U-CORE-4)
        (c.number_class, true, "new()"),
        (c.number_class, true, "new(_)"),
        // §2.5 String
        (c.string_class, false, "+(_)"),
        (c.string_class, false, "hash"), // NEW (ADR-0023)
        (c.string_class, true, "new()"),
        (c.string_class, true, "new(_)"),
        // U-STRING raw byte accessors (ADR-0049 amendment)
        (c.string_class, false, "_$byteCount"),  // NEW (ADR-0049)
        (c.string_class, false, "_$byteAt(_)"),  // NEW (ADR-0049)
        (c.string_class, false, "_$slice(_,_)"), // NEW (ADR-0049)
        // §2.6 Bool
        (c.bool_class, true, "new()"),
        (c.bool_class, true, "new(_)"),
        (c.bool_class, false, "and(_)"),
        (c.bool_class, false, "or(_)"),
        (c.bool_class, false, "not()"),
        (c.bool_class, false, "ifTrue(_)"),
        (c.bool_class, false, "ifFalse(_)"),
        (c.bool_class, false, "ifTrue(_,ifFalse)"),
        (c.bool_class, false, "hash"), // NEW (ADR-0023)
        // §2.7 Symbol
        (c.symbol_class, false, "toString"),
        (c.symbol_class, false, "hash"), // NEW (ADR-0023)
        (c.symbol_class, true, "new(_)"),
        // §2.8 Absence
        (c.some_class, true, "call(_)"),
        (c.some_class, true, "new(_)"),
        (c.option_class, false, "match(some,none)"),
        // §2.9 Method
        (c.method_class, true, "new(_)"),
        (c.method_class, false, "arity"),
        (c.method_class, false, "name"),
        (c.method_class, false, "invokeOn(_,***)"), // callable rest gateway
        (c.method_class, false, "bind(_)"),         // NEW (ADR-0028)
        (c.method_class, false, "selector"),        // NEW (ADR-0028)
        (c.method_class, false, "holder"),          // NEW (ADR-0028)
        // §2.9 MethodFamily
        (c.method_family_class, false, "selectors"),
        (c.method_family_class, false, "size"),
        (c.method_family_class, false, "methodFor(_)"),
        (c.method_family_class, false, "bind(_)"),
        // §2.10 Family
        (c.family_class, false, "receiver"),
        (c.family_class, false, "selector"),
        (c.family_class, false, "pattern"),
        (c.family_class, false, "isExact"),
        (c.family_class, false, "get()"),
        (c.family_class, false, "set(_)"),
        // §2.10 Function
        (c.function_class, false, "arity"),
        (c.function_class, false, "name"),
        (c.function_class, false, "callWith(_)"),
        (c.function_class, false, "call(***)"),
        // §2.10 Closure
        (c.closure_class, false, "arity"),
        (c.closure_class, false, "name"),
        (c.closure_class, false, "whileTrue(_)"),
        // U-ERR error-handling catch protocol (ADR-0038) — NEW_ON_ENSURE
        (c.closure_class, false, "on(_,_)"),
        (c.closure_class, false, "ensure(_)"),
        // §2.11 System
        (c.system_class, true, "print(_)"),
        (c.system_class, true, "new()"),
        // Native ready-queue scheduler seam (U-SCHED) — NEW_SCHED
        (c.system_class, true, "schedule(_)"),
        (c.system_class, true, "nextScheduled"),
        (c.system_class, true, "gc"),
        // U-STRING raw I/O seam (ADR-0049 amendment)
        (c.system_class, true, "_$write(_)"), // NEW (ADR-0049)
        // §2.12 Module (U15, ADR-0045) — NEW_IMPORTS
        (c.module_class, true, "new()"),
        (c.module_class, false, "doesNotUnderstand(_)"),
        (c.module_class, false, "name"),
        (c.module_class, false, "namespace"),
        (c.module_class, false, "package"),
        (c.module_class, false, "rootPackage"),
        (c.module_class, false, "packageInfo"),
        (c.module_class, false, "exports"),
        (c.module_class, false, "metadata"),
        (c.module_class, false, "dependencies"),
        (c.module_class, false, "uri"),
        (c.module_class, false, "identity"),
        (c.module_class, false, "__exports__"),
        (c.module_class, false, "__export__(_)"),
        (c.module_class, false, "__understands__(_)"),
        (c.module_class, false, "__metadata__"),
        (c.module_class, false, "__dependencies__"),
        (c.module_class, false, "__uri__"),
        (c.module_class, false, "__name__"),
        (c.module_class, false, "__id__"),
        (c.module_class, false, "__path__"),
        (c.module_class, false, "toString"),
        // §2.13 List
        (c.list_class, true, "new()"),
        (c.list_class, false, "_$length"),
        (c.list_class, false, "_$at(_)"),
        (c.list_class, false, "_$set(_,_)"),
        (c.list_class, false, "_$push(_)"),
        (c.list_class, false, "_$replaceSlice(_,_,_)"),
        (c.list_class, false, "toString"),
        // §2.x Bytes (PDR-0011 ruling 3 + PDR-0013 ruling 4, U-BYTES)
        (c.bytes_class, true, "new(_)"),
        (c.bytes_class, true, "_$fromString(_)"),
        (c.bytes_class, false, "_$size"),
        (c.bytes_class, false, "_$at(_)"),
        (c.bytes_class, false, "_$set(_,_)"),
        (c.bytes_class, false, "_$fill(_)"),
        (c.bytes_class, false, "_$slice(_,_)"),
        (c.bytes_class, false, "_$copyInto(_,_)"),
        (c.bytes_class, false, "_$utf8"),
        (c.bytes_class, false, "_$utf8Lossy"),
        (c.bytes_class, false, "_$equalsConstantTime(_)"),
        // §2.14 Message
        (c.message_class, false, "selector"),
        (c.message_class, false, "name"),
        (c.message_class, false, "labels"),
        (c.message_class, false, "args"),
        // §2.15 Error (U-CORE-6, ADR-0037) — NEW_ERROR
        (c.error_class, false, "message"),
        (c.error_class, false, "raise()"),
        // Map/Set (U-COLLTYPES Phase 1, ADR-0039) — NEW_MAP_SET
        (c.map_class, true, "new()"),
        (c.map_class, false, "_$size"),
        (c.map_class, false, "_$get(_)"),
        (c.map_class, false, "_$put(_,_)"),
        (c.map_class, false, "_$has(_)"),
        (c.map_class, false, "_$remove(_)"),
        (c.map_class, false, "_$keyAt(_)"),
        (c.map_class, false, "_$valueAt(_)"),
        (c.set_class, true, "new()"),
        (c.set_class, false, "_$size"),
        (c.set_class, false, "_$add(_)"),
        (c.set_class, false, "_$has(_)"),
        (c.set_class, false, "_$remove(_)"),
        (c.set_class, false, "_$at(_)"),
        // Tuple (U-COLLTYPES Phase 2, ADR-0039) — NEW_TUPLE
        (c.tuple_class, true, "_$fromList(_)"),
        (c.tuple_class, false, "_$size"),
        (c.tuple_class, false, "_$at(_)"),
        (c.tuple_class, false, "_$positionalSize"),
        (c.tuple_class, false, "_$labelAt(_)"),
        (c.tuple_class, false, "_$positionals"),
        (c.tuple_class, false, "_$labeled"),
        (c.tuple_class, false, "_$slice(_,_)"),
        // Record (Spec A product foundations) — NEW_RECORD
        (c.record_class, false, "_$size"),
        (c.record_class, false, "_$labelAt(_)"),
        (c.record_class, false, "_$valueAt(_)"),
        // Range construction is direct `BuildRange` bytecode; only these
        // native observers belong to the floor.
        (c.range_class, false, "_$lower"),
        (c.range_class, false, "_$upper"),
        (c.range_class, false, "_$upperInclusive"),
        // Fiber (ADR-0030) — NEW_FIBER. Installed since the fiber work landed;
        // audited only from 2026-07-15 (DEFERRED CB-5). `universe/primitives.rs`
        // L362-374, `primitive/fiber.rs`.
        (c.fiber_class, true, "new(_)"),
        (c.fiber_class, false, "call()"),
        (c.fiber_class, false, "call(_)"),
        (c.fiber_class, false, "try()"),
        (c.fiber_class, false, "try(_)"),
        (c.fiber_class, true, "yield()"),
        (c.fiber_class, true, "yield(_)"),
        (c.fiber_class, true, "current"),
        (c.fiber_class, true, "abort(_)"),
        (c.fiber_class, false, "isDone"),
        (c.fiber_class, false, "isRoot"),
        (c.fiber_class, false, "error"),
        // System (Resource tracking primitives)
        (c.system_class, true, "_$leakReport"),
        (c.system_class, true, "_$strictResources(_)"),
    ];

    // Resolve each binding to its owning class (metaclass for statics).
    let targets: Vec<(ClassId, String)> = bindings
        .iter()
        .map(|&(cls, is_static, sel)| {
            let owner = if is_static { vm.heap.class(cls).class } else { cls };
            (owner, sel.to_string())
        })
        .collect();

    let mut expected: HashSet<(ClassId, Symbol)> = HashSet::new();
    for (owner, sel) in targets {
        let sym = vm.get_or_intern(&sel);
        assert!(expected.insert((owner, sym)), "duplicate census entry for `{sel}`");
    }

    // Reconstruct the live floor from the heap: every core row's own methods
    // (instance side) plus its metaclass's own methods (static side). Only
    // native `Primitive` bindings count — `core.ph`-defined closures (e.g.
    // `Object#isA(_)`, the `List`/`Option` protocol) are *derived* surface, not
    // floor, and must be excluded (floor-census §1/§3).
    let mut live: HashSet<(ClassId, Symbol)> = HashSet::new();
    for (_, class_id) in core_class_rows(&vm) {
        for owner in [class_id, vm.heap.class(class_id).class] {
            let primitives: Vec<Symbol> = vm
                .heap
                .class(owner)
                .methods
                .iter()
                .filter(|(_, method_id)| vm.heap.method(**method_id).is_primitive())
                .map(|(sym, _)| *sym)
                .collect();
            for sym in primitives {
                live.insert((owner, sym));
            }
            let rest_primitives: Vec<Symbol> = vm
                .heap
                .class(owner)
                .rest_methods
                .values()
                .filter(|method_id| vm.heap.method(**method_id).is_primitive())
                .map(|method_id| vm.heap.method(*method_id).signature.selector)
                .collect();
            for sym in rest_primitives {
                live.insert((owner, sym));
            }
        }
    }

    let describe = |pairs: HashSet<(ClassId, Symbol)>| -> Vec<String> {
        let mut out: Vec<String> = pairs
            .iter()
            .map(|(id, sym)| format!("{}#{}", vm.heap.class(*id).name, vm.interner.lookup(*sym)))
            .collect();
        out.sort();
        out
    };

    let missing: HashSet<_> = expected.difference(&live).copied().collect();
    let extra: HashSet<_> = live.difference(&expected).copied().collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "floor census drift:\n  in census but NOT installed: {:?}\n  installed but NOT in census: {:?}",
        describe(missing),
        describe(extra),
    );

    assert_eq!(expected.len(), 181, "census must enumerate exactly 181 bindings after Module reflection");
    assert_eq!(live.len(), 181, "the live floor must be exactly 181 bindings");
}

#[test]
fn parallel_rule_holds_for_all_ordinary_rows() {
    // R-INV-0.2 (corpus half) — `X.class.superclass == X.superclass.class` for
    // every ordinary (non-apex) core row, including the U11 `True`/`False`
    // rows (both resolve to `Bool class`) and the absence / collection / message
    // rows. The boot half of this rides `Universe::verify_invariants`.
    let vm = VM::new();
    let c = &vm.universe.classes;
    let rows: [(&str, ClassId); 22] = [
        ("Number", c.number_class),
        ("Int", c.int_class),
        ("Float", c.float_class),
        ("String", c.string_class),
        ("Nil", c.nil_class),
        ("Bool", c.bool_class),
        ("True", c.true_class),
        ("False", c.false_class),
        ("Method", c.method_class),
        ("Function", c.function_class),
        ("Closure", c.closure_class),
        ("BoundMethod", c.bound_method_class),
        ("Symbol", c.symbol_class),
        ("Module", c.module_class),
        ("System", c.system_class),
        ("Option", c.option_class),
        ("Some", c.some_class),
        ("None", c.none_class),
        ("List", c.list_class),
        ("Message", c.message_class),
        ("Error", c.error_class),
        ("MessageNotUnderstood", c.message_not_understood_class),
    ];
    for (name, class_id) in rows {
        let meta = vm.heap.class(class_id).class;
        let superclass = vm.heap.class(class_id).superclass.unwrap_or_else(|| panic!("{name}.superclass should be set"));
        let expected = vm.heap.class(superclass).class;
        assert_eq!(
            vm.heap.class(meta).superclass,
            Some(expected),
            "{name}.class.superclass should equal {name}.superclass.class (parallel rule)"
        );
    }
}

// ---------------------------------------------------------------------------
// R-INV-1.x — U-CORE-1 unit invariants (spec §5.2).
// ---------------------------------------------------------------------------

#[test]
fn isa_is_reflexive_and_superclass_closed() {
    // R-INV-1.2 — `x.isA(x.class)` and `x.isA(Object)` hold; `x.isA(C)` iff `C`
    // is on `x.class`'s superclass chain. Exercised on an immediate, a class
    // receiver, and a user instance.
    let mut vm = VM::new();
    let number = Value::obj(vm.universe.classes.number_class);
    let string = Value::obj(vm.universe.classes.string_class);
    let object = Value::obj(vm.universe.classes.object_class);
    let class = Value::obj(vm.universe.classes.class_class);

    // Immediate receiver `3`.
    assert_eq!(send1(&mut vm, Value::int(3), "isA(_)", number).as_bool(), Some(true), "3.isA(Number)");
    assert_eq!(
        send1(&mut vm, Value::int(3), "isA(_)", object).as_bool(),
        Some(true),
        "3.isA(Object) — reflexive-to-root"
    );
    assert_eq!(send1(&mut vm, Value::int(3), "isA(_)", string).as_bool(), Some(false), "!3.isA(String)");

    // Class receiver `Number` (walks the metaclass chain, Smalltalk isKindOf:).
    assert_eq!(send1(&mut vm, number, "isA(_)", class).as_bool(), Some(true), "Number.isA(Class)");
    assert_eq!(send1(&mut vm, number, "isA(_)", object).as_bool(), Some(true), "Number.isA(Object)");

    // User instance.
    let module = vm.create_module("main", "isa_user_instance");
    vm.interpret_source(module, "class IsaFoo {}\n").expect("class decl should run");
    let foo_sym = vm.get_or_intern("IsaFoo");
    let foo_cls = *vm.classes.get(&ClassKey { module, name: foo_sym }).expect("IsaFoo registered");
    let foo_val = Value::obj(foo_cls);
    let instance = send0(&mut vm, foo_val, "new()");
    assert_eq!(send1(&mut vm, instance, "isA(_)", foo_val).as_bool(), Some(true), "aFoo.isA(IsaFoo)");
    assert_eq!(send1(&mut vm, instance, "isA(_)", object).as_bool(), Some(true), "aFoo.isA(Object)");
    assert_eq!(send1(&mut vm, instance, "isA(_)", number).as_bool(), Some(false), "!aFoo.isA(Number)");
}

#[test]
fn hash_is_consistent_with_equality() {
    let mut vm = VM::new();

    // Number: 3 == 3.
    let n3a = as_number(number_hash(&mut vm, &Value::int(3), &[]).unwrap());
    let n3b = as_number(number_hash(&mut vm, &Value::int(3), &[]).unwrap());
    let n4 = as_number(number_hash(&mut vm, &Value::int(4), &[]).unwrap());
    assert_eq!(n3a, n3b, "3.hash == 3.hash");
    assert_ne!(n3a, n4, "3.hash != 4.hash");

    // String: two distinct-handle, equal-content strings.
    let s1 = vm.alloc_string_value("ab".to_string());
    let s2 = vm.alloc_string_value("ab".to_string());
    assert!(
        s1.as_obj().is_some() && s2.as_obj().is_some() && s1.as_obj() != s2.as_obj(),
        "the two strings must have distinct handles"
    );
    let h1 = as_number(string_hash(&mut vm, &s1, &[]).unwrap());
    let h2 = as_number(string_hash(&mut vm, &s2, &[]).unwrap());
    assert_eq!(h1, h2, "equal string content ⇒ equal hash");

    // Bool: distinct codes.
    let ht = as_number(bool_hash(&mut vm, &Value::bool(true), &[]).unwrap());
    let hf = as_number(bool_hash(&mut vm, &Value::bool(false), &[]).unwrap());
    assert_ne!(ht, hf, "true.hash != false.hash");

    // Identity object: same handle ⇒ equal hash.
    let obj = Value::obj(vm.universe.classes.none_class);
    let o1 = as_number(object_hash(&mut vm, &obj, &[]).unwrap());
    let o2 = as_number(object_hash(&mut vm, &obj, &[]).unwrap());
    assert_eq!(o1, o2, "same-handle identity hash is equal");

    // Symbol: stability across two references to the same interned symbol.
    let sym = Value::symbol(vm.get_or_intern("someSelector"));
    let y1 = as_number(symbol_hash(&mut vm, &sym, &[]).unwrap());
    let y2 = as_number(symbol_hash(&mut vm, &sym, &[]).unwrap());
    assert_eq!(y1, y2, "the same interned symbol hashes stably");
}

#[test]
fn hash_is_stable_across_repeated_calls() {
    let mut vm = VM::new();
    let obj = Value::obj(vm.universe.classes.none_class);
    let cases: [Value; 4] = [Value::int(42), Value::bool(true), Value::symbol(vm.get_or_intern("stable")), obj];
    for value in cases {
        let first = send0(&mut vm, value, "hash");
        let second = send0(&mut vm, value, "hash");
        assert_eq!(as_number(first), as_number(second), "hash of {value:?} must be stable across calls");
    }
    // String separately (needs a heap allocation, not a bare immediate).
    let s = vm.alloc_string_value("stableString".to_string());
    let a = send0(&mut vm, s, "hash");
    let b = send0(&mut vm, s, "hash");
    assert_eq!(as_number(a), as_number(b), "String#hash must be stable across calls");
}

#[test]
fn method_is_a_reified_object_outside_function_hierarchy() {
    // `Method` is a reified dispatch object, not an executable Function
    // descendant. It retains reflection metadata without inheriting call.
    let mut vm = VM::new();
    let method_class = vm.universe.classes.method_class;
    let object_class = vm.universe.classes.object_class;
    assert_eq!(vm.heap.class(method_class).superclass, Some(object_class), "Method.superclass should be Object");

    // Parallel rule for Method.
    let method_meta = vm.heap.class(method_class).class;
    let object_meta = vm.heap.class(object_class).class;
    assert_eq!(
        vm.heap.class(method_meta).superclass,
        Some(object_meta),
        "Method.class.superclass should be Object.class"
    );

    for selector in ["arity", "name"] {
        let sym = vm.get_or_intern(selector);
        assert!(
            lookup_method_in_hierarchy(&vm.heap, method_class, sym).is_some(),
            "Method should expose `{selector}` reflection"
        );
    }
    for selector in ["call()", "call(_)"] {
        let sym = vm.get_or_intern(selector);
        assert!(
            lookup_method_in_hierarchy(&vm.heap, method_class, sym).is_none(),
            "Method must not expose `{selector}`"
        );
    }
}

#[test]
fn callable_surface_is_vm_owned_and_sealed() {
    let vm = VM::new();
    let c = vm.universe.classes;
    assert!(vm.heap.class(c.function_class).is_abstract);
    assert_eq!(vm.heap.class(c.closure_class).superclass, Some(c.function_class));
    assert_eq!(vm.heap.class(c.bound_method_class).superclass, Some(c.function_class));
    assert_eq!(vm.heap.class(c.family_class).superclass, Some(c.function_class));
    assert_eq!(vm.heap.class(c.method_class).superclass, Some(c.object_class));

    let mut vm = vm;
    let module = vm.create_module("main", "callable_surface_is_vm_owned_and_sealed");
    for (index, superclass) in ["Function", "Closure", "BoundMethod", "Family", "Method"].into_iter().enumerate() {
        let source = format!("class UserCallable{index} is {superclass} {{}}\n");
        let error = vm.interpret_source(module, &source).expect_err("user subclass of callable surface must fail");
        assert!(error.to_string().contains("attr.sealed_violation"), "unexpected error: {error}");
    }
}

#[test]
fn reflection_is_side_effect_free() {
    // R-INV-1.6 — `Behavior#name` / `Behavior#methods` read the class's data
    // without mutating it: two calls agree and the method dictionary is
    // unchanged afterward.
    let mut vm = VM::new();
    let number_class = vm.universe.classes.number_class;
    let receiver = Value::obj(number_class);
    let before = vm.heap.class(number_class).methods.len();

    let n1 = behavior_name(&mut vm, &receiver, &[]).unwrap();
    let n2 = behavior_name(&mut vm, &receiver, &[]).unwrap();
    let name1 = match n1.as_obj() {
        Some(id) => vm.heap.string(id).as_str().to_string(),
        None => panic!("Behavior#name should return a String, got {n1:?}"),
    };
    let name2 = match n2.as_obj() {
        Some(id) => vm.heap.string(id).as_str().to_string(),
        None => panic!("Behavior#name should return a String, got {n2:?}"),
    };
    assert_eq!(name1, "Number");
    assert_eq!(name1, name2, "Behavior#name is deterministic");

    let m1 = match behavior_methods(&mut vm, &receiver, &[]).unwrap().as_obj() {
        Some(id) => vm.heap.list(id).len(),
        None => panic!("Behavior#methods should return a List"),
    };
    let m2 = match behavior_methods(&mut vm, &receiver, &[]).unwrap().as_obj() {
        Some(id) => vm.heap.list(id).len(),
        None => panic!("Behavior#methods should return a List"),
    };
    assert_eq!(m1, m2, "Behavior#methods returns the same count on repeat");

    let after = vm.heap.class(number_class).methods.len();
    assert_eq!(before, after, "reflection must not mutate the method dictionary");
}

// ---------------------------------------------------------------------------
// R-INV-3.x — U-CORE-3 unit invariants (the `Method` reflection surface,
// ADR-0028).
// ---------------------------------------------------------------------------

#[test]
fn callable_tower_and_reflection_protocol() {
    // R-INV-3.1 — `Closure`/`BoundMethod`/`Family` are `Function` descendants
    // (the boot half rides `Universe::verify_invariants`); a reified `Method`
    // and its bound counterpart both expose `arity`/`name` through the
    // reflection primitives' receiver-shape handling.
    let mut vm = VM::new();
    let closure_class = vm.universe.classes.closure_class;
    let function_class = vm.universe.classes.function_class;
    assert_eq!(
        vm.heap.class(closure_class).superclass,
        Some(function_class),
        "Closure.superclass should be Function"
    );

    let module = vm.create_module("main", "callable_tower_and_reflection_protocol");
    vm.interpret_source(
        module,
        "class Greeter {\n  greet(_ name) { return \"Hello, \" + name }\n}\nlet g = Greeter.new()\n",
    )
    .expect("class + instance should compile and run");
    let g_sym = vm.interner.intern("g");
    let g = vm.heap.module(module).get(g_sym).expect("`g` global should exist");

    let selector_sym = vm.get_or_intern("greet(_)");
    let method_value = object_method_for(&mut vm, &g, &[Value::symbol(selector_sym)]).expect("methodFor should succeed");
    assert!(!method_value.is_none(), "methodFor should hit for a defined selector");

    // `arity`/`name` on the bare `Method`.
    assert_eq!(block_arity(&mut vm, &method_value, &[]).unwrap().as_int(), Some(1), "Method#arity should be 1");
    match method_value.as_obj() {
        Some(_) => {
            let res = block_name(&mut vm, &method_value, &[]).unwrap();
            let name_id = res.as_obj().expect("Method#name should return String");
            assert_eq!(vm.heap.string(name_id).as_str(), "greet(_)", "Method#name should be the encoded selector");
        }
        None => panic!("Method#name should return a String"),
    }

    // `arity`/`name` on the `BoundMethod` produced by `bind(_)`.
    let bound = method_bind(&mut vm, &method_value, &[g]).expect("bind should succeed");
    assert_eq!(block_arity(&mut vm, &bound, &[]).unwrap().as_int(), Some(1), "BoundMethod#arity should be 1");
    let bound_res = block_name(&mut vm, &bound, &[]).unwrap();
    let bound_name_id = bound_res.as_obj().expect("BoundMethod#name should return String");
    assert_eq!(
        vm.heap.string(bound_name_id).as_str(),
        "greet(_)",
        "BoundMethod#name should be the wrapped method's name"
    );
}

#[test]
fn invoke_on_preserves_dead_frame_fencing_for_escaping_blocks() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "invoke_on_preserves_dead_frame_fencing_for_escaping_blocks");
    vm.interpret_source(module, "class Maker {\n  make() { return || { return 1 } }\n}\nlet maker = Maker.new()\n")
        .expect("class + instance should compile and run");

    let maker_sym = vm.interner.intern("maker");
    let maker = vm.heap.module(module).get(maker_sym).expect("`maker` global should exist");

    let make_sym = vm.get_or_intern("make()");
    let method_id = maker.lookup_method(&vm, make_sym).expect("Maker should define make()");

    let escaped = vm.invoke_method_object(method_id, maker, &[]).expect("invokeOn `make()` should succeed");

    let call_sym = vm.get_or_intern("call()");
    let result = vm.send_dynamic(escaped, call_sym, &[]);
    assert!(
        matches!(result, Err(PhError::Runtime(RuntimeError::DeadFrameError))),
        "calling the escaped block after its home invokeOn activation is gone should raise DeadFrameError, got {result:?}"
    );
}

#[test]
fn cross_fiber_non_local_return_raises_dead_frame_error() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "cross_fiber_non_local_return_raises_dead_frame_error");
    vm.interpret_source(
        module,
        "class Maker {\n  make() { return || { return 1 } }\n}\nlet escaped = Maker.new().make()\n",
    )
    .expect("class + escaping block should compile and run");

    let result = vm.interpret_source(module, "let f = Fiber.new(escaped)\nf.call()\n");
    match result {
        Err(PhError::Runtime(RuntimeError::Raise { error, .. })) => {
            let msg_sym = vm.get_or_intern("message");
            let msg = vm.send_dynamic(error, msg_sym, &[]).unwrap().to_string(&vm);
            assert!(msg.contains("DeadFrameError"), "expected message to contain DeadFrameError, got {msg}");
        }
        other => panic!("expected RuntimeError::Raise, got {other:?}"),
    }
}

#[test]
fn invoke_on_and_bind_call_are_equivalent() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "invoke_on_and_bind_call_are_equivalent");
    vm.interpret_source(
        module,
        "class Greeter {\n  greet(_ name) { return \"Hello, \" + name }\n}\nlet g = Greeter.new()\n",
    )
    .expect("class + instance should compile and run");
    let g_sym = vm.interner.intern("g");
    let g = vm.heap.module(module).get(g_sym).expect("`g` global should exist");

    let selector_sym = vm.get_or_intern("greet(_)");
    let method_value = object_method_for(&mut vm, &g, &[Value::symbol(selector_sym)]).expect("methodFor should succeed");

    let arg = vm.alloc_string_value("World".to_string());
    let method_id = method_value.as_obj().expect("methodFor should return Method");
    let via_invoke_on = vm.invoke_method_object(method_id, g, &[arg]).expect("synchronous host invoke should succeed");
    let bound = method_bind(&mut vm, &method_value, &[g]).expect("bind should succeed");
    let via_bind_call = block_call(&mut vm, &bound, &[arg]).expect("bound.call should succeed");

    assert!(
        via_invoke_on.value_eq(&via_bind_call, &vm.heap),
        "invokeOn(recv, ***args) and bind(recv).call(***args) should produce equal results"
    );
}

#[test]
fn invoke_on_and_bind_call_reject_arity_mismatch() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "invoke_on_and_bind_call_reject_arity_mismatch");
    vm.interpret_source(
        module,
        "class Greeter {\n  greet(_ name) { return \"Hello, \" + name }\n}\nlet g = Greeter.new()\n",
    )
    .expect("class + instance should compile and run");
    let g_sym = vm.interner.intern("g");
    let g = vm.heap.module(module).get(g_sym).expect("`g` global should exist");

    let selector_sym = vm.get_or_intern("greet(_)");
    let method_value = object_method_for(&mut vm, &g, &[Value::symbol(selector_sym)]).expect("methodFor should succeed");

    let method_id = method_value.as_obj().expect("methodFor should return Method");
    let result = vm.invoke_method_object(method_id, g, &[]);
    assert!(
        matches!(result, Err(PhError::Runtime(RuntimeError::Arity { .. }))),
        "invokeOn with the wrong argument count should raise RuntimeError::Arity, got {result:?}"
    );

    let bound = method_bind(&mut vm, &method_value, &[g]).expect("bind should succeed");
    let bound_result = block_call(&mut vm, &bound, &[]);
    assert!(
        matches!(bound_result, Err(PhError::Runtime(RuntimeError::Arity { .. }))),
        "bound.call with the wrong argument count should raise RuntimeError::Arity, got {bound_result:?}"
    );
}

#[test]
fn bind_allows_foreign_receiver_for_representation_independent_method() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "bind_allows_foreign_receiver_for_representation_independent_method");
    vm.interpret_source(module, "class Greeter { greet(_ name) { name } }\nlet g = Greeter.new()\n")
        .expect("class + instance should run");
    let g = vm.heap.module(module).get(vm.interner.intern("g")).expect("`g` global should exist");
    let selector = vm.get_or_intern("greet(_)");
    let method = object_method_for(&mut vm, &g, &[Value::symbol(selector)]).expect("methodFor should return Greeter#greet(_)");
    let bound = method_bind(&mut vm, &method, &[Value::int(3)]).expect("foreign receiver should bind");
    let result = block_call(&mut vm, &bound, &[Value::int(3)]).expect("representation-independent method should activate");
    assert_eq!(result, Value::int(3));
}

fn value_type_sweep(vm: &mut VM) -> Vec<(&'static str, Value)> {
    let string = vm.alloc_string_value("hi".to_string());
    let symbol = Value::symbol(vm.get_or_intern("foo"));
    let list = Value::obj(vm.heap.alloc_list(vec![Value::int(1), Value::int(2)]));
    let none = Value::none();
    let some_class = Value::obj(vm.universe.classes.some_class);
    let some = some_call(vm, &some_class, &[Value::int(42)]).expect("Some(42) should succeed");
    vec![
        ("Number", Value::int(42)),
        ("String", string),
        ("Symbol", symbol),
        ("Bool", Value::bool(true)),
        ("None", none),
        ("Some(_)", some),
        ("List", list),
    ]
}

#[test]
fn value_tostring_message_agrees_with_print_path() {
    let mut vm = VM::new();
    for (label, value) in value_type_sweep(&mut vm) {
        let message = send0(&mut vm, value, "toString");
        let message_text = message.to_string(&vm);
        let print_text = value.to_string(&vm);
        assert_eq!(message_text, print_text, "{label}: `toString` message disagrees with the print path");
    }
}

#[test]
fn value_object_default_tostring_is_angle_bracket_class_name() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "value_object_default_tostring_is_angle_bracket_class_name");
    vm.interpret_source(module, "class Foo {}\nlet f = Foo.new()\n")
        .expect("class + instance should compile and run");
    let f_sym = vm.interner.intern("f");
    let f = vm.heap.module(module).get(f_sym).expect("`f` global should exist");

    let rendered = send0(&mut vm, f, "toString");
    assert_eq!(rendered.to_string(&vm), "<Foo>", "a bare user instance should render as `<ClassName>`");

    let number_rendered = send0(&mut vm, Value::int(42), "toString");
    assert_eq!(
        number_rendered.to_string(&vm),
        "42",
        "Number's own toString override must still read the numeric value"
    );
}

#[test]
fn option_tostring_matches_none_and_wraps_some() {
    let mut vm = VM::new();
    let none = Value::none();
    let none_rendered = send0(&mut vm, none, "toString");
    assert_eq!(none_rendered.to_string(&vm), "None");

    let some_class = Value::obj(vm.universe.classes.some_class);
    let some = some_call(&mut vm, &some_class, &[Value::int(42)]).expect("Some(42) should succeed");
    let some_rendered = send0(&mut vm, some, "toString");
    assert_eq!(some_rendered.to_string(&vm), "Some(42)");

    let some_none = some_call(&mut vm, &some_class, &[none]).expect("Some(None) should succeed");
    let some_none_rendered = send0(&mut vm, some_none, "toString");
    assert_eq!(some_none_rendered.to_string(&vm), "Some(None)");

    let inner_some = some_call(&mut vm, &some_class, &[Value::int(1)]).expect("Some(1) should succeed");
    let nested_some = some_call(&mut vm, &some_class, &[inner_some]).expect("Some(Some(1)) should succeed");
    let nested_rendered = send0(&mut vm, nested_some, "toString");
    assert_eq!(nested_rendered.to_string(&vm), "Some(Some(1))");
}

#[test]
fn value_tostring_is_total_and_never_leaks_nil() {
    let mut vm = VM::new();
    for (label, value) in value_type_sweep(&mut vm) {
        let sym = vm.get_or_intern("toString");
        let result = vm.send_dynamic(value, sym, &[]);
        let rendered = result.unwrap_or_else(|err| panic!("{label}: toString should never raise, got {err:?}"));
        let text = rendered.to_string(&vm);
        assert!(!text.contains("nil"), "{label}: toString rendered the raw `nil` sentinel: {text:?}");
    }

    let module = vm.create_module("main", "value_tostring_is_total_and_never_leaks_nil");
    vm.interpret_source(module, "let r = true.ifTrue || { }\n")
        .expect("empty ifTrue should compile and run");
    let r_sym = vm.interner.intern("r");
    let result = vm.heap.module(module).get(r_sym).expect("`r` global should exist");
    let rendered = send0(&mut vm, result, "toString");
    assert_eq!(rendered.to_string(&vm), "Some(None)");
}

fn is_a(vm: &VM, mut class_id: ClassId, target: ClassId) -> bool {
    loop {
        if class_id == target {
            return true;
        }
        match vm.heap.class(class_id).superclass {
            Some(next) => class_id = next,
            None => return false,
        }
    }
}

#[test]
fn genuine_miss_raises_surface_message_not_understood() {
    let mut vm = VM::new();
    let bogus = vm.get_or_intern("frobnicate");
    let err = vm.send_dynamic(Value::int(3), bogus, &[]).unwrap_err();

    let raised = match err {
        PhError::Runtime(RuntimeError::Raise { error, .. }) => error,
        other => panic!("expected RuntimeError::Raise, got {other:?}"),
    };

    let cls = raised.class(&vm);
    assert_eq!(
        cls, vm.universe.classes.message_not_understood_class,
        "the raised object's class should be MessageNotUnderstood"
    );
    assert!(
        is_a(&vm, cls, vm.universe.classes.error_class),
        "the raised MessageNotUnderstood must be isA(Error)"
    );

    let message = send0(&mut vm, raised, "message");
    assert_eq!(
        message.to_string(&vm),
        "3 does not understand 'frobnicate'",
        "Error#message should read the rendered miss string"
    );

    let reified_message = match raised.as_obj() {
        Some(id) => vm.heap.as_instance(id).expect("MessageNotUnderstood should be an InstanceObject").slots[1],
        None => panic!("expected an Obj"),
    };
    assert_eq!(
        reified_message.class(&vm),
        vm.universe.classes.message_class,
        "slot 1 should hold a reified Message"
    );
    let reified_selector = send0(&mut vm, reified_message, "selector");
    assert_eq!(
        reified_selector.as_symbol().ok(),
        Some(bogus),
        "the reified Message's selector should be the missed selector"
    );
}

#[test]
fn only_error_subclasses_respond_to_raise() {
    let mut vm = VM::new();
    let raise_sym = Value::symbol(vm.get_or_intern("raise()"));

    let responds_to_number = send1(&mut vm, Value::int(3), "respondsTo(_)", raise_sym);
    assert_eq!(responds_to_number.as_bool(), Some(false), "a Number should not respond to `raise`");

    let error_instance = {
        let error_class = vm.universe.classes.error_class;
        let field_count = vm.heap.class(error_class).field_count;
        let inst = phalcom_core::heap::InstanceObject::new(error_class, field_count);
        Value::obj(vm.heap.alloc(phalcom_core::heap::Object::Instance(inst)))
    };
    let responds_to_error = send1(&mut vm, error_instance, "respondsTo(_)", raise_sym);
    assert_eq!(responds_to_error.as_bool(), Some(true), "an Error instance should respond to `raise`");
}

#[test]
fn error_raise_unwinds_through_the_shared_raise_payload() {
    let mut vm = VM::new();
    let error_class = vm.universe.classes.error_class;
    let field_count = vm.heap.class(error_class).field_count;
    let mut inst = phalcom_core::heap::InstanceObject::new(error_class, field_count);
    let text = vm.alloc_string_value("boom".to_string());
    inst.slots[0] = text;
    let error_instance = Value::obj(vm.heap.alloc(phalcom_core::heap::Object::Instance(inst)));

    let raise_sym = vm.get_or_intern("raise()");
    let err = vm.send_dynamic(error_instance, raise_sym, &[]).unwrap_err();
    match err {
        PhError::Runtime(RuntimeError::Raise { error, rendered, .. }) => {
            assert!(
                error.as_obj().is_some() && error.as_obj() == error_instance.as_obj(),
                "raise() should carry `self` as the raised error"
            );
            assert_eq!(rendered, "boom", "raise() should render via the `message` send");
        }
        other => panic!("expected RuntimeError::Raise, got {other:?}"),
    }
}

#[test]
fn overriding_does_not_understand_still_intercepts_before_the_default_raise() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "overriding_does_not_understand_still_intercepts_before_the_default_raise");
    vm.interpret_source(
        module,
        "class Proxy {\n  doesNotUnderstand(_ msg) { return \"intercepted: \" + msg.name }\n}\nlet p = Proxy.new()\n",
    )
    .expect("class + instance should compile and run");
    let p_sym = vm.interner.intern("p");
    let p = vm.heap.module(module).get(p_sym).expect("`p` global should exist");

    let bogus = vm.get_or_intern("frobnicate");
    let result = vm.send_dynamic(p, bogus, &[]).expect("the override should intercept, not raise");
    assert_eq!(
        result.to_string(&vm),
        "intercepted: frobnicate",
        "the user override should run instead of the default raise"
    );
}

#[test]
fn on_catch_restore_survives_a_deep_throw_and_the_vm_stays_healthy() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "on_catch_restore_survives_a_deep_throw_and_the_vm_stays_healthy");
    vm.interpret_source(
        module,
        r#"
class DeepErr is Error {
  @constructor
  new(msg) { super.new(msg) }
}
class M {
  deep(_ n) {
    (n <= 0).ifTrue || { return self.boom() }
    return self.deep(n - 1)
  }
  boom() {
    throw DeepErr.new("deep")
  }
}
let r = || { M.new().deep(30) }.on(Error) |e| {
  let y = "handled:" + e.message
  y
}
let after = 1 + 2
"#,
    )
    .expect("a deep throw caught by `on` should run to completion");

    let r_sym = vm.interner.intern("r");
    let r_value = vm.heap.module(module).get(r_sym).expect("`r` global should exist");
    assert_eq!(
        r_value.to_string(&vm),
        "handled:deep",
        "the handler's post-restore allocation should survive as the `on` result"
    );

    let after_sym = vm.interner.intern("after");
    let after_value = vm.heap.module(module).get(after_sym).expect("`after` global should exist");
    assert!(
        as_number(after_value) == 3.0,
        "the VM must stay healthy enough to keep running top-level code after the deep catch, got {after_value:?}"
    );
}

#[test]
fn on_isa_match_walks_the_superclass_chain() {
    // U-ERR isA-match invariant (error-handling.md §2: "`on T` catches `T`
    // and its subclasses"): `on(BaseErr)` must catch a thrown `SubErr`
    // instance by walking `SubErr`'s superclass chain up to `BaseErr` (mirrors
    // `core.ph`'s `Object#isA`), not just an exact class-identity check.
    let mut vm = VM::new();
    let module = vm.create_module("main", "on_isa_match_walks_the_superclass_chain");
    vm.interpret_source(
        module,
        r#"
class BaseErr is Error {
  @constructor
  new(msg) { super.new(msg) }
}
class SubErr is BaseErr {
  @constructor
  new(msg) { super.new(msg) }
}
let r = || { throw SubErr.new("leaf") }.on(BaseErr) |e| { "caught:" + e.message }
"#,
    )
    .expect("on(Super) should catch a thrown Sub instance");

    let r_sym = vm.interner.intern("r");
    let r_value = vm.heap.module(module).get(r_sym).expect("`r` global should exist");
    assert_eq!(
        r_value.to_string(&vm),
        "caught:leaf",
        "on(BaseErr) should catch a SubErr throw via the superclass walk"
    );
}

#[test]
fn obj_ref_opaque_round_trip() {
    let mut vm = VM::new();
    let id1 = vm.heap.alloc_string("hello".to_string());
    let v1 = Value::obj(id1);
    assert!(v1.is_obj());
    assert_eq!(v1.as_obj(), Some(id1));
    // A second allocation must produce a distinct handle.
    let id2 = vm.heap.alloc_string("world".to_string());
    let v2 = Value::obj(id2);
    assert_ne!(v1.as_obj(), v2.as_obj());
}
