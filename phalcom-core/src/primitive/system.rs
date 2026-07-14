//! Native primitives on `System`.

use crate::error::{PhResult, RuntimeError};
use crate::value::Value;
use crate::vm::VM;

/// Signature: `System.class::print(_)` — prints its arguments, then a newline.
///
/// Returns the `None` singleton (surface absence value): `print` is a
/// statement-like send whose result is user-reachable (e.g. `print(print(1))`),
/// so it must never yield the raw `nil` sentinel (Invariant 4,
/// [ADR-0007](../../../docs/adr/0007-option-some-none.md)).
///
/// Renders each argument via [`Value::to_display_string`], which sends
/// `toString` to any heap object with no bespoke native renderer, so this
/// stays in agreement with an explicit `.toString` send for user classes,
/// instances and metaclasses (U-ERR-FIX PRINT-TOSTRING).
///
/// # Errors
///
/// Propagates any error a `toString` send raises while rendering an argument.
pub fn system_class_print(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    for arg in args {
        let text = arg.to_display_string(vm)?;
        print!("{text}");
    }
    println!();
    Ok(vm.none_value())
}

/// Signature: `System.class::new()` — always an error; `System` is not instantiable.
///
/// # Errors
///
/// Always returns [`RuntimeError::NotAllowed`].
pub fn system_class_new(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::NotAllowed("System instances cannot be created".to_string()).into())
}

/// Signature: `System::schedule(_)` — wraps `args[0]` (a `Function`) as a
/// fresh, not-yet-started `Fiber` and enqueues it on `VM::ready_queue`
/// (private VM bookkeeping, `vm/mod.rs`). Returns the `Fiber`, so a caller
/// (e.g. `Future.async`) can hold a handle to poll it later via the
/// already-landed `isDone`/`error`
/// ([U-FIBER-REFLECT](../../../docs/forge/units/U-SCHED-FIBER/U-FIBER-REFLECT/plan.md)).
///
/// Does **not** run the fiber — it runs at "the next scheduler turn"
/// (`system.md` §2), i.e. whenever the queue is next drained (see
/// [`crate::vm::VM::run`]'s root-drive, or `.ph` `System.runScheduled`).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `args[0]` is not a `Function` (mirrors
/// `Fiber.new(_)`'s own check — reuses that validation via `fiber::new_fiber_ref`
/// rather than duplicating it).
pub fn system_schedule(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let fiber_ref = crate::primitive::fiber::new_fiber_ref(vm, args[0])?;
    vm.ready_queue.push_back(fiber_ref);
    Ok(Value::Obj(fiber_ref))
}

/// Signature: `System::nextScheduled` — pops and returns the next queued
/// fiber as `Option<Fiber>` (`None` once the queue is empty). The `.ph`-
/// visible drain seam every pump loop (native root-drive, `.ph`
/// `System.runScheduled`, and eventually `Future`'s Slice B await-pump)
/// bottoms out in.
pub fn system_next_scheduled(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    match vm.ready_queue.pop_front() {
        Some(fiber_ref) => Ok(crate::primitive::nil::wrap_some(vm, Value::Obj(fiber_ref))),
        None => Ok(vm.none_value()),
    }
}

/// Signature: `System.gc` — forces one full mark-sweep and returns `None`
/// (`system.md` §`gc`, [ADR-0050](../../../docs/adr/0050-non-moving-mark-sweep-collector.md) §8).
///
/// Runs **no finalizers**, performs **no compaction**, and changes **no handle**
/// (Invariant M1) — a surviving object keeps its `ObjRef`. Deterministic and safe
/// to call from `.ph` code: a primitive runs at a dispatch safepoint by
/// construction, where `VM::stack`/`frames` are the complete root truth
/// ([memory-management.md §4](../../../docs/spec/v0.2/memory-management.md)).
///
/// Returns `None` rather than the swept count because `system.md` §`gc` says so;
/// the count is available to Rust via [`VM::force_gc`].
///
/// # Errors
///
/// Infallible; returns [`PhResult`] to match the primitive ABI.
pub fn system_gc(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    vm.force_gc();
    Ok(vm.none_value())
}

