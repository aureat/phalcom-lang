//! `verify_invariants()` harness for the object model / metaclass tower.
//!
//! `docs/spec/object-model.md` §6 step 7 calls for a `verify_invariants()`
//! that runs after bootstrap and asserts every rule in §5. No such function
//! exists in `phalcom-core/src` today (adding one is a core-source change,
//! out of scope for this stabilization pass — flagged for the architect).
//! This file is the test-side stand-in: it builds a real [`VM`] (which runs
//! the actual bootstrap in [`Universe::create_core_classes`]) and asserts, via
//! **handle identity** on the live class graph, exactly the invariants the
//! *current* implementation satisfies.
//!
//! Since [ADR-0009](../../docs/adr/0009-handle-arena-heap.md) every class is a
//! [`ClassId`] into the [`VM`]'s [`Heap`], and its metaclass (`class`) and
//! superclass links are plain [`ClassId`] handles. Object identity is therefore
//! a `==` on the `Copy` handle — the old `Rc::ptr_eq` on `PhRef<ClassObject>`
//! is gone with the `Rc<RefCell<T>>` graph it tested. Links are read through
//! [`Heap::class`].
//!
//! Spec invariants the current bootstrap does **not** satisfy are present as
//! `#[ignore]`d tests below, each citing the relevant spec section. Per ADR
//! 0002, the metaclass tower's "superclass parallels instance superclass"
//! rule is a known, deliberate deferral — do not fix it here.
//!
//! [`VM`]: phalcom_core::vm::VM
//! [`Universe::create_core_classes`]: phalcom_core::universe::Universe
//! [`ClassId`]: phalcom_core::heap::ClassId
//! [`Heap`]: phalcom_core::heap::Heap
//! [`Heap::class`]: phalcom_core::heap::Heap::class

use phalcom_core::heap::ClassId;
use phalcom_core::vm::VM;

// --- Invariants that hold today ------------------------------------------

#[test]
fn metaclass_is_its_own_class_closing_the_loop() {
    // Metaclass.class == Metaclass (object-model.md §5 sanity check:
    // "Metaclass.class.class == Metaclass").
    let vm = VM::new();
    let metaclass: ClassId = vm.universe.classes.metaclass_class;
    let metaclass_class = vm.heap.class(metaclass).class;
    assert_eq!(metaclass_class, metaclass, "Metaclass.class should be Metaclass itself");
}

#[test]
fn class_class_is_metaclass() {
    // Class.class == Metaclass.
    let vm = VM::new();
    let class_class = vm.universe.classes.class_class;
    let metaclass = vm.universe.classes.metaclass_class;
    assert_eq!(vm.heap.class(class_class).class, metaclass, "Class.class should be Metaclass");
}

#[test]
fn object_class_class_is_metaclass() {
    // object-model.md §5 sanity check: "Object.class.class == Metaclass".
    let vm = VM::new();
    let object_class = vm.universe.classes.object_class;
    let metaclass = vm.universe.classes.metaclass_class;
    let object_metaclass = vm.heap.class(object_class).class;
    assert_eq!(
        vm.heap.class(object_metaclass).class,
        metaclass,
        "Object.class.class should be Metaclass"
    );
}

#[test]
fn object_has_no_superclass() {
    let vm = VM::new();
    let object_class = vm.universe.classes.object_class;
    assert!(vm.heap.class(object_class).superclass.is_none(), "Object.superclass should be None");
}

#[test]
fn metaclass_superclass_is_class() {
    // Current bootstrap wiring (universe.rs): Metaclass.superclass == Class.
    let vm = VM::new();
    let metaclass = vm.universe.classes.metaclass_class;
    let class_class = vm.universe.classes.class_class;
    let sup = vm.heap.class(metaclass).superclass.expect("Metaclass should have a superclass");
    assert_eq!(sup, class_class, "Metaclass.superclass should be Class");
}

#[test]
fn core_classes_have_correct_metaclass_and_superclass() {
    // Every class created via `create_core_class` (Number, String, Nil,
    // Bool, Method, Symbol, Module, System):
    //   - X.class.class == Metaclass (every metaclass is instance-of Metaclass)
    //   - X.superclass == Object (current bootstrap gives them all Object directly)
    let vm = VM::new();
    let metaclass = vm.universe.classes.metaclass_class;
    let object_class = vm.universe.classes.object_class;

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
        assert_eq!(
            vm.heap.class(class_meta).class,
            metaclass,
            "{name}.class.class should be Metaclass"
        );
        let sup = vm
            .heap
            .class(class)
            .superclass
            .unwrap_or_else(|| panic!("{name}.superclass should be set"));
        assert_eq!(sup, object_class, "{name}.superclass should be Object");
    }
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

// --- Spec invariants the current bootstrap does NOT satisfy ---------------
// (targets for a later phase; do not "fix" these in a stabilization pass)

#[test]
#[ignore = "object-model.md §5 rule 4: (X class).superclass == (X.superclass) class. \
Current bootstrap (universe.rs::create_core_class) wires every core class's metaclass \
superclass flatly to Class, not to the metaclass of X's own superclass. Known bug, \
see ADR 0002 (parallel metaclass hierarchy)."]
fn metaclass_superclass_parallels_instance_superclass() {
    let vm = VM::new();
    let number_class = vm.universe.classes.number_class;
    let object_class = vm.universe.classes.object_class;

    let number_meta = vm.heap.class(number_class).class;
    let object_meta = vm.heap.class(object_class).class;
    let number_meta_super =
        vm.heap.class(number_meta).superclass.expect("Number.class should have a superclass");

    assert_eq!(
        number_meta_super, object_meta,
        "Number.class.superclass should be Object.class (it is currently Class)"
    );
}

#[test]
#[ignore = "object-model.md §5 diagram: the tower should include a `Behavior` class \
(Object <- Behavior <- Class/Metaclass) so Class and Metaclass share a common non-Object \
ancestor. `Universe::CoreClasses` has no `behavior_class` field today — introducing one \
is a core-source (universe.rs) change, out of scope for this stabilization pass. \
Flagged for the architect alongside ADR 0002."]
fn behavior_class_exists_in_tower() {
    unimplemented!(
        "CoreClasses does not expose a `behavior_class`; see object-model.md §5 diagram \
         and ADR 0002 for the target shape."
    );
}
