//! `verify_invariants()` harness for the object model / metaclass tower.
//!
//! `docs/spec/object-model.md` §6 step 7 calls for a `verify_invariants()`
//! that runs after bootstrap and asserts every rule in §5. No such function
//! exists in `phalcom-core/src` today (adding one is a core-source change,
//! out of scope for this stabilization pass — flagged for the architect).
//! This file is the test-side stand-in: it builds a real `VM` (which runs
//! the actual bootstrap in `Universe::create_core_classes`) and asserts, via
//! pointer identity on the live `PhRef<ClassObject>` graph, exactly the
//! invariants the *current* implementation satisfies.
//!
//! Spec invariants the current bootstrap does **not** satisfy are present as
//! `#[ignore]`d tests below, each citing the relevant spec section. Per ADR
//! 0002, the metaclass tower's "superclass parallels instance superclass"
//! rule is a known, deliberate deferral — do not fix it here.

use phalcom_core::class::ClassObject;
use phalcom_core::vm::VM;
use phalcom_common::PhRef;
use std::rc::Rc;

fn ptr_eq(a: &PhRef<ClassObject>, b: &PhRef<ClassObject>) -> bool {
    Rc::ptr_eq(a, b)
}

// --- Invariants that hold today ------------------------------------------

#[test]
fn metaclass_is_its_own_class_closing_the_loop() {
    // Metaclass.class == Metaclass (object-model.md §5 sanity check:
    // "Metaclass.class.class == Metaclass").
    let vm = VM::new();
    let metaclass = vm.universe.classes.metaclass_class.clone();
    let metaclass_class = metaclass.borrow().class();
    assert!(ptr_eq(&metaclass_class, &metaclass), "Metaclass.class should be Metaclass itself");
}

#[test]
fn class_class_is_metaclass() {
    // Class.class == Metaclass.
    let vm = VM::new();
    let class_class = vm.universe.classes.class_class.clone();
    let metaclass = vm.universe.classes.metaclass_class.clone();
    assert!(ptr_eq(&class_class.borrow().class(), &metaclass), "Class.class should be Metaclass");
}

#[test]
fn object_class_class_is_metaclass() {
    // object-model.md §5 sanity check: "Object.class.class == Metaclass".
    let vm = VM::new();
    let object_class = vm.universe.classes.object_class.clone();
    let metaclass = vm.universe.classes.metaclass_class.clone();
    let object_metaclass = object_class.borrow().class();
    assert!(
        ptr_eq(&object_metaclass.borrow().class(), &metaclass),
        "Object.class.class should be Metaclass"
    );
}

#[test]
fn object_has_no_superclass() {
    let vm = VM::new();
    let object_class = vm.universe.classes.object_class.clone();
    assert!(object_class.borrow().superclass().is_none(), "Object.superclass should be None");
}

#[test]
fn metaclass_superclass_is_class() {
    // Current bootstrap wiring (universe.rs): Metaclass.superclass == Class.
    let vm = VM::new();
    let metaclass = vm.universe.classes.metaclass_class.clone();
    let class_class = vm.universe.classes.class_class.clone();
    let sup = metaclass.borrow().superclass().expect("Metaclass should have a superclass");
    assert!(ptr_eq(&sup, &class_class), "Metaclass.superclass should be Class");
}

#[test]
fn core_classes_have_correct_metaclass_and_superclass() {
    // Every class created via `create_core_class` (Number, String, Nil,
    // Bool, Method, Symbol, Module, System):
    //   - X.class.class == Metaclass (every metaclass is instance-of Metaclass)
    //   - X.superclass == Object (current bootstrap gives them all Object directly)
    let vm = VM::new();
    let metaclass = vm.universe.classes.metaclass_class.clone();
    let object_class = vm.universe.classes.object_class.clone();

    let core = [
        ("Number", vm.universe.classes.number_class.clone()),
        ("String", vm.universe.classes.string_class.clone()),
        ("Nil", vm.universe.classes.nil_class.clone()),
        ("Bool", vm.universe.classes.bool_class.clone()),
        ("Method", vm.universe.classes.method_class.clone()),
        ("Symbol", vm.universe.classes.symbol_class.clone()),
        ("Module", vm.universe.classes.module_class.clone()),
        ("System", vm.universe.classes.system_class.clone()),
    ];

    for (name, class) in core {
        let class_meta = class.borrow().class();
        assert!(
            ptr_eq(&class_meta.borrow().class(), &metaclass),
            "{name}.class.class should be Metaclass"
        );
        let sup = class.borrow().superclass().unwrap_or_else(|| panic!("{name}.superclass should be set"));
        assert!(ptr_eq(&sup, &object_class), "{name}.superclass should be Object");
    }
}

#[test]
fn walking_metaclass_superclass_chain_terminates() {
    // object-model.md §5 sanity check: walking any metaclass's superclass
    // chain terminates (does not loop forever / dangle). Bounded walk guards
    // against an infinite loop turning into a hang instead of a failure.
    let vm = VM::new();
    let number_class = vm.universe.classes.number_class.clone();
    let mut current = number_class.borrow().class();
    let mut steps = 0;
    loop {
        let next = current.borrow().superclass();
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
    let number_class = vm.universe.classes.number_class.clone();
    let object_class = vm.universe.classes.object_class.clone();

    let number_meta = number_class.borrow().class();
    let object_meta = object_class.borrow().class();
    let number_meta_super = number_meta.borrow().superclass().expect("Number.class should have a superclass");

    assert!(
        ptr_eq(&number_meta_super, &object_meta),
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
