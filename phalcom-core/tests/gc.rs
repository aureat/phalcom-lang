//! Mark-sweep collector tests — U-GC step 2's green gate.
//!
//! Asserts the properties of [memory-management.md §3/§6](../../docs/spec/v0.2/memory-management.md)
//! and the M-series invariants, per
//! [ADR-0050](../../docs/adr/0050-non-moving-mark-sweep-collector.md) (Accepted).
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
