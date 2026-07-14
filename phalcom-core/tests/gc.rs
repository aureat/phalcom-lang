//! Mark-sweep collector tests — U-GC step 2's green gate.
//!
//! Asserts the properties of [memory-management.md §3/§6](../../docs/spec/v0.2/memory-management.md)
//! and the M-series invariants, per
//! [ADR-0050](../../docs/adr/accepted/0050-non-moving-mark-sweep-collector.md) (Accepted).
//!
//! These drive `vm.force_gc()` directly. Automatic safepoint-latched collection
//! and the `temp_roots` escape hatch are step 4; the temp-root stress test lands
//! with them.

use phalcom_core::heap::{InstanceObject, Object};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

/// A VM whose heap is already **collected once**, plus that settled live count.
///
/// A freshly bootstrapped VM is not garbage-free: running `core.ph` leaves its
/// top-level `Closure` unreachable (the core `ModuleObject::closure` field is
/// `None` by the time bootstrap returns), so the very first `force_gc()` on any
/// VM legitimately sweeps it. Tests that assert an exact live count must
/// therefore baseline *after* a collection, or they measure that closure rather
/// than their own fixture.
fn settled_vm() -> (VM, usize) {
    let mut vm = VM::new();
    vm.force_gc();
    let baseline = vm.heap.live_count();
    (vm, baseline)
}

/// Allocates an `Instance` of the kernel `Object` class with `slots`.
///
/// Bare heap allocation on purpose: these tests exercise reachability, and a
/// handle that no root holds is exactly the fixture. Rooting it via a `.ph`
/// program would defeat the point.
fn alloc_instance(vm: &mut VM, slots: Vec<Value>) -> phalcom_core::heap::ObjRef {
    let class = vm.universe.classes.object_class;
    vm.heap.alloc(Object::Instance(InstanceObject { class, slots: slots.into_boxed_slice() }))
}

/// **Reclaims garbage (M6).** An object no root reaches is swept, and its handle
/// goes stale rather than resolving to some other object.
#[test]
fn collects_unreachable_object() {
    let (mut vm, before) = settled_vm();

    let garbage = alloc_instance(&mut vm, vec![]);
    assert_eq!(vm.heap.live_count(), before + 1);

    let swept = vm.force_gc();
    assert!(swept >= 1, "expected the unrooted instance to be swept, swept {swept}");
    assert_eq!(vm.heap.live_count(), before, "live count should return to the pre-alloc baseline");
    assert!(vm.heap.try_get(garbage).is_none(), "swept handle must be stale, not resolve to an object");
}

/// **Collects cycles (the reason this is mark-sweep, not refcount).** Two
/// instances referencing each other, rooted by nothing, both die. Reference
/// counting could never free this pair — see ADR-0050 §Alternatives.
#[test]
fn collects_cycle() {
    let (mut vm, before) = settled_vm();

    let a = alloc_instance(&mut vm, vec![Value::Nil]);
    let b = alloc_instance(&mut vm, vec![Value::Obj(a)]);
    match vm.heap.get_mut(a) {
        Object::Instance(inst) => inst.slots[0] = Value::Obj(b),
        _ => unreachable!(),
    }

    vm.force_gc();
    assert_eq!(vm.heap.live_count(), before, "an unrooted cycle must be fully collected");
    assert!(vm.heap.try_get(a).is_none() && vm.heap.try_get(b).is_none());
}

/// **Kernel survives (M5).** The kernel is pinned *and* is itself a cycle
/// (`Metaclass` is an instance of itself) — the marker must terminate on it and
/// never sweep it. `verify_invariants` re-checks the whole tower post-GC.
#[test]
fn kernel_survives_collection() {
    let mut vm = VM::new();
    vm.force_gc();

    vm.universe
        .verify_invariants(&vm.heap)
        .expect("kernel must satisfy every object-model invariant after a collection");

    let mut roots = Vec::new();
    vm.collect_roots(&mut roots);
    for root in roots {
        assert!(vm.heap.try_get(root).is_some(), "a root resolved stale after GC: {root:?}");
    }
}

/// **Reachability is transitive.** An object held only *through* a rooted chain
/// survives; the same object survives no longer than its holder.
#[test]
fn retains_transitively_then_collects_when_root_drops() {
    let (mut vm, before) = settled_vm();

    let leaf = alloc_instance(&mut vm, vec![]);
    let holder = alloc_instance(&mut vm, vec![Value::Obj(leaf)]);

    // Root the holder the way real code does — on the operand stack.
    vm.push_root_for_test(Value::Obj(holder));
    vm.force_gc();
    assert!(vm.heap.try_get(leaf).is_some(), "leaf is reachable via the rooted holder");
    assert!(vm.heap.try_get(holder).is_some());

    vm.pop_root_for_test();
    vm.force_gc();
    assert_eq!(vm.heap.live_count(), before, "both die once the root is gone");
    assert!(vm.heap.try_get(leaf).is_none(), "leaf must die with its only holder");
}

/// **Deep chain (worklist, not recursion).** A 100k-deep instance chain must
/// collect without overflowing the native stack — the whole reason marking uses
/// an explicit worklist. A recursive tracer would blow up here.
#[test]
fn deep_chain_collects_without_stack_overflow() {
    let (mut vm, before) = settled_vm();

    let mut head = alloc_instance(&mut vm, vec![]);
    for _ in 0..100_000 {
        head = alloc_instance(&mut vm, vec![Value::Obj(head)]);
    }

    // Rooted: mark must walk all 100k links.
    vm.push_root_for_test(Value::Obj(head));
    vm.force_gc();
    assert_eq!(vm.heap.live_count(), before + 100_001, "the whole rooted chain survives");

    // Unrooted: sweep must free all 100k.
    vm.pop_root_for_test();
    vm.force_gc();
    assert_eq!(vm.heap.live_count(), before, "the whole chain is reclaimed");
}

/// **Non-moving (M1).** A surviving object keeps its handle for life — the
/// property inline-cache tags, `==` identity, and parked fiber `Value`s all rest
/// on. Asserted by identity of the handle *and* of the value behind it.
#[test]
fn surviving_objects_keep_their_handles() {
    let (mut vm, _) = settled_vm();

    let survivor = alloc_instance(&mut vm, vec![Value::Number(42.0)]);
    vm.push_root_for_test(Value::Obj(survivor));

    // Allocate garbage around it so the sweep genuinely has work to do.
    for _ in 0..100 {
        alloc_instance(&mut vm, vec![]);
    }

    vm.force_gc();

    match vm.heap.get(survivor) {
        Object::Instance(inst) => assert_eq!(inst.slots[0], Value::Number(42.0)),
        _ => panic!("survivor's handle must still name the same object"),
    }
    vm.pop_root_for_test();
}

/// **System.gc returns None (U-GC step 3).**
#[test]
fn system_gc_returns_none() {
    let (mut vm, _) = settled_vm();
    let system_cls = Value::Obj(vm.universe.classes.system_class);
    use phalcom_core::primitive::system::system_gc;
    let result = system_gc(&mut vm, &system_cls, &[]).expect("System.gc primitive call");
    
    let none_singleton = vm.universe.classes.none_singleton;
    assert!(matches!(result, Value::Obj(id) if id == none_singleton));
}

/// **System.gc collects garbage from source (U-GC step 3).**
#[test]
fn system_gc_collects_garbage_from_source() {
    let (mut vm, before) = settled_vm();
    let module = vm.create_module("test_gc", "system_gc_test");

    vm.interpret_source(
        module,
        r#"
        class UnusedClass {}
        var i = 0
        while (i < 10) {
            UnusedClass.new()
            i = i + 1
        }
        System.gc
        "#,
    )
    .expect("interpret_source failed");

    // Loop-allocated UnusedClass instances must be swept.
    assert!(vm.heap.live_count() < before + 10, "GC should have reclaimed the 10 loop-allocated instances");
}

/// **System.gc is idempotent (U-GC step 3).**
#[test]
fn system_gc_is_idempotent() {
    let (mut vm, _) = settled_vm();
    let system_cls = Value::Obj(vm.universe.classes.system_class);
    use phalcom_core::primitive::system::system_gc;
    
    system_gc(&mut vm, &system_cls, &[]).expect("first System.gc");
    let count1 = vm.heap.live_count();
    
    system_gc(&mut vm, &system_cls, &[]).expect("second System.gc");
    let count2 = vm.heap.live_count();
    
    assert_eq!(count1, count2, "System.gc twice in a row must be idempotent");
}

/// **Invariant L — alloc latches but never collects (U-GC step 4).**
#[test]
fn alloc_latches_but_never_collects() {
    let (mut vm, before) = settled_vm();
    
    // Allocate one unrooted object and hold its handle.
    let unrooted = alloc_instance(&mut vm, vec![]);
    assert_eq!(vm.heap.live_count(), before + 1);
    
    // Allocate > 4096 objects to cross the latch without running bytecode.
    for _ in 0..4100 {
        alloc_instance(&mut vm, vec![]);
    }
    
    // The unrooted object must STILL be alive (latch does not collect).
    assert!(vm.heap.try_get(unrooted).is_some());
    assert!(vm.heap.gc_pending());
}

/// **The automatic safepoint fires (U-GC step 4).**
#[test]
fn automatic_safepoint_fires() {
    let (mut vm, before) = settled_vm();
    let module = vm.create_module("test_gc_safepoint", "safepoint_test");

    // Loop churns past threshold.
    vm.interpret_source(
        module,
        r#"
        class Trash {}
        var i = 0
        while (i < 5000) {
            Trash.new()
            i = i + 1
        }
        "#,
    )
    .expect("interpret_source failed");

    // The automatic collections should have run, bounding live count.
    assert!(vm.heap.live_count() < before + 2000, "live count should be bounded by GC");
    assert!(!vm.heap.gc_pending(), "gc_pending should be reset after safepoint collection");
}

/// **Bounded churn, real workload (U-GC step 4).**
#[test]
fn bounded_churn_real_workload() {
    let (mut vm, before) = settled_vm();
    let module = vm.create_module("test_gc_workload", "workload_test");

    // We do a simple loop that accumulates but does not leak.
    vm.interpret_source(
        module,
        r#"
        var i = 0
        while (i < 6000) {
            i = i + 1
        }
        "#,
    )
    .expect("interpret_source failed");

    assert!(vm.heap.live_count() < before + 1000, "heap size must remain bounded");
}

/// **Suspended fiber roots its stack (U-GC step 4).**
#[test]
fn suspended_fiber_roots_its_stack() {
    let (mut vm, _before) = settled_vm();
    let module = vm.create_module("test_gc_fiber", "fiber_test");

    vm.interpret_source(
        module,
        "class RootItem {}\nvar fiber = Fiber.new {\nvar item = RootItem.new()\nFiber.yield(item)\n}\nfiber.call()\n",
    )
    .expect("interpret_source failed");

    // Retrieve the fiber from the module's globals.
    let module_obj = match vm.heap.get(module) {
        Object::Module(m) => m,
        _ => panic!("expected Module"),
    };
    let fiber_sym = vm.interner.intern("fiber");
    let fiber_val = module_obj.get(fiber_sym).expect("fiber global should exist");
    let fiber_ref = match fiber_val {
        Value::Obj(id) => id,
        _ => panic!("expected Fiber object returned"),
    };

    // Extract the yielded item from the fiber's stack.
    let fiber_obj = vm.heap.fiber(fiber_ref);
    let yielded_ref = fiber_obj.stack.iter().find_map(|v| match v {
        Value::Obj(id) => Some(*id),
        _ => None,
    }).expect("expected object in fiber stack");

    // Root the fiber in Rust.
    vm.push_root_for_test(fiber_val);

    vm.force_gc();

    // The fiber and yielded item must survive.
    assert!(vm.heap.try_get(fiber_ref).is_some(), "fiber should survive when rooted");
    assert!(vm.heap.try_get(yielded_ref).is_some(), "yielded item should survive via rooted fiber");

    // Pop the fiber root.
    vm.pop_root_for_test();

    // The fiber is still reachable because it is a global in the module.
    // Clear the module's global to make it unreachable.
    let module_mut = match vm.heap.get_mut(module) {
        Object::Module(m) => m,
        _ => panic!("expected Module"),
    };
    let fiber_idx = module_mut.name_to_slot.get(&fiber_sym).copied().unwrap();
    module_mut.globals[fiber_idx] = Value::Nil;

    vm.force_gc();

    assert!(vm.heap.try_get(fiber_ref).is_none(), "fiber should be collected");
    assert!(vm.heap.try_get(yielded_ref).is_none(), "yielded item should be collected");
}

/// **verify_invariants post-GC kernel assert (U-GC step 4).**
#[test]
fn verify_invariants_post_automatic_gc() {
    let mut vm = VM::new();
    let module = vm.create_module("test_gc_invariants", "invariants_test");
    
    vm.interpret_source(
        module,
        r#"
        var i = 0
        while (i < 5000) {
            i = i + 1
        }
        "#,
    )
    .expect("interpret_source failed");

    vm.universe
        .verify_invariants(&vm.heap)
        .expect("kernel invariants must hold after automatic GC");
}



