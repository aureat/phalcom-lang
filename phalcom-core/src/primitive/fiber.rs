//! Native primitives for `Fiber` — the sole cooperative-concurrency primitive
//! ([ADR-0030](../../../docs/adr/0030-fibers-and-futures-cooperative-concurrency.md)).
//!
//! A fiber switch (`call`/`try`/`yield`) is an O(1) pointer-free handoff: the
//! parking fiber's live `VM::frames`/`VM::stack`/`VM::open_upvalues` are
//! moved into its [`crate::heap::FiberObject`], the resuming fiber's own
//! parked state is moved back in, and `VM::switch_pending` tells
//! `VM::call_method`'s `Primitive` arm to skip the ordinary post-call stack
//! reconciliation (D5). Every switch is legal only when
//! `VM::native_reentry_depth` is `0` (ADR-0030 §4, the restricted-yield
//! guard) — a nested re-entrant `run_until` (`block_call` and friends) has
//! its own `base_frames` computed against the *currently* running fiber,
//! which a switch underneath it would corrupt.

use crate::error::{PhResult, RuntimeError};
use crate::frame::CallContext;
use crate::heap::{FiberResumeMode, FiberStatus, Object, ObjRef};
use crate::heap::InstanceObject;
use crate::value::Value;
use crate::vm::VM;

/// Moves `vm`'s live stacks (`frames`/`stack`/`open_upvalues`) into the
/// parked [`FiberObject`] behind `fiber_ref` (ADR-0030 §3).
///
/// Called on the fiber giving up the CPU, just before [`VM::current`] is
/// repointed at the fiber taking over. `mem::take` leaves the VM's live
/// mirror empty — the counterpart [`load_live_from`] fills it back in for
/// whichever fiber runs next.
pub(crate) fn store_live_into(vm: &mut VM, fiber_ref: ObjRef) {
    let frames = std::mem::take(&mut vm.frames);
    let stack = std::mem::take(&mut vm.stack);
    let open_upvalues = std::mem::take(&mut vm.open_upvalues);
    // `checking` (ADR-0052 Fix 1, U-ANNOT-CONTRACTS) swaps alongside the
    // three fields above for the same reason: an `@invariant`-guarded call
    // can `yield` mid-body, so this fiber's in-flight guard bookkeeping must
    // park with it rather than leak into whichever fiber runs next.
    let checking = std::mem::take(&mut vm.checking);
    let fiber = vm.heap.fiber_mut(fiber_ref);
    fiber.frames = frames;
    fiber.stack = stack;
    fiber.open_upvalues = open_upvalues;
    fiber.checking = checking;
}

/// Moves the parked [`FiberObject`] behind `fiber_ref`'s stacks back into
/// `vm`'s live mirror (`frames`/`stack`/`open_upvalues`/`checking`) — the
/// reverse of [`store_live_into`], run on the fiber that is about to become
/// [`VM::current`].
pub(crate) fn load_live_from(vm: &mut VM, fiber_ref: ObjRef) {
    let fiber = vm.heap.fiber_mut(fiber_ref);
    let frames = std::mem::take(&mut fiber.frames);
    let stack = std::mem::take(&mut fiber.stack);
    let open_upvalues = std::mem::take(&mut fiber.open_upvalues);
    let checking = std::mem::take(&mut fiber.checking);
    vm.frames = frames;
    vm.stack = stack;
    vm.open_upvalues = open_upvalues;
    vm.checking = checking;
}

/// Builds and raises a `CannotYieldAcrossNativeFrame` instance carrying
/// `rendered` as its message, mirroring
/// [`crate::primitive::object::object_does_not_understand`]'s build-then-raise
/// pattern. Shared by [`cannot_yield_across_native_frame`] and
/// [`cannot_resume_across_native_frame`], whose only difference is the
/// message text — the surface class (and thus what a `catch` clause matches
/// on) stays the same for both restricted-switch violations (D-FIB-1).
fn cannot_switch_across_native_frame(vm: &mut VM, rendered: String) -> crate::error::PhError {
    let class = vm.universe.classes.cannot_yield_across_native_frame_class;
    let field_count = vm.heap.class(class).field_count;
    let mut inst = InstanceObject::new(class, field_count);
    inst.slots[0] = vm.alloc_string_value(rendered.clone());
    let error = Value::Obj(vm.heap.alloc(Object::Instance(inst)));
    RuntimeError::Raise { error, rendered }.into()
}

/// Builds and raises the `CannotYieldAcrossNativeFrame` error (D-FIB-1) for a
/// `Fiber::yield` that finds `VM::native_reentry_depth` has grown past the
/// fiber's recorded `floor_depth` since it was last resumed — a *yield*
/// attempted underneath a native re-entrant `run_until` (a `block_call` and
/// friends) on the Rust call stack.
fn cannot_yield_across_native_frame(vm: &mut VM) -> crate::error::PhError {
    cannot_switch_across_native_frame(vm, "cannot switch fibers across a native call frame (e.g. inside an .on(_) handler or .ensure(_) cleanup)".to_string())
}

/// Builds and raises the `CannotYieldAcrossNativeFrame` error (D-FIB-1) for a
/// `Fiber#call`/`try` attempted while any native re-entrant `run_until` (a
/// `block_call` and friends) is on the Rust call stack
/// (`VM::native_reentry_depth != 0`). This is a *resume*, not a *yield* —
/// spec §6's restriction table only forecloses yielding underneath a native
/// frame, so this is a deliberately wider, sound over-restriction (a
/// nested `run_until`'s `base_frames` is computed against the currently
/// running fiber, which any switch underneath it — resume or yield alike —
/// would corrupt). The message names the actual violated action instead of
/// reusing [`cannot_yield_across_native_frame`]'s yield-specific wording.
fn cannot_resume_across_native_frame(vm: &mut VM) -> crate::error::PhError {
    cannot_switch_across_native_frame(vm, "cannot resume a fiber across a native call frame (e.g. inside an .on(_) handler or .ensure(_) cleanup)".to_string())
}

/// Resolves `receiver` to the [`FiberObject`] handle it refers to.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `receiver` is not a `Fiber`.
fn expect_fiber(vm: &VM, receiver: &Value) -> PhResult<ObjRef> {
    match receiver {
        Value::Obj(id) if vm.heap.as_fiber(*id).is_some() => Ok(*id),
        other => Err(RuntimeError::Type { expected: "Fiber", found: other.type_name() }.into()),
    }
}

/// Validates `entry` as a `Block`/`Closure`, builds a new, not-yet-started
/// [`crate::heap::FiberObject`] wrapping it
/// ([`crate::heap::FiberObject::new_entry`]), and allocates it on the heap.
///
/// Shared construction path behind both `Fiber.new(_)` ([`fiber_new`]) and
/// `System.schedule(_)` ([`crate::primitive::system::system_schedule`]) —
/// extracted so the ready-queue's enqueue reuses the exact same validation
/// and allocation as an ordinary `Fiber.new` (U-SCHED).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `entry` is not a `Block`/`Closure`.
pub(crate) fn new_fiber_ref(vm: &mut VM, entry: Value) -> PhResult<ObjRef> {
    match entry {
        Value::Obj(id) => match vm.heap.get(id) {
            Object::Block(_) | Object::Closure(_) => {}
            _ => return Err(RuntimeError::Type { expected: "Function", found: entry.type_name() }.into()),
        },
        other => return Err(RuntimeError::Type { expected: "Function", found: other.type_name() }.into()),
    }
    let entry_id = match entry {
        Value::Obj(id) => id,
        _ => unreachable!("checked above"),
    };
    #[cfg(feature = "fiber-pool")]
    let fiber = {
        let (stack, frames) = vm.fiber_pool.pop().unwrap_or_else(|| (Vec::new(), Vec::new()));
        crate::heap::FiberObject::new_entry_with_buffers(entry_id, stack, frames)
    };
    #[cfg(not(feature = "fiber-pool"))]
    let fiber = crate::heap::FiberObject::new_entry(entry_id);
    Ok(vm.heap.alloc(Object::Fiber(Box::new(fiber))))
}

/// Signature: `Fiber.new(_)` — builds a new, not-yet-started fiber wrapping
/// `args[0]` as its entry callable ([`crate::heap::FiberObject::new_entry`]).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `args[0]` is not a `Block`/`Closure`.
pub fn fiber_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::Obj(new_fiber_ref(vm, args[0])?))
}

/// Signature: `Fiber::current` — the currently-running fiber (`VM::current`).
pub fn fiber_current(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::Obj(vm.current))
}

/// Signature: `Fiber#isDone` — `true` once the receiver is `Done` or `Failed`.
///
/// A pure read over [`crate::heap::FiberObject::status`] — no scheduler or
/// suspension dependency, callable from anywhere (including under a native
/// re-entrant frame, unlike `call`/`try`/`yield`) ([U-FIBER-REFLECT]).
///
/// [U-FIBER-REFLECT]: ../../../docs/forge/units/U-SCHED-FIBER/U-FIBER-REFLECT/plan.md
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `receiver` is not a `Fiber`.
pub fn fiber_is_done(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let fiber_ref = expect_fiber(vm, receiver)?;
    let status = vm.heap.fiber(fiber_ref).status;
    Ok(Value::Bool(matches!(status, FiberStatus::Done | FiberStatus::Failed)))
}

/// Signature: `Fiber#isRoot` — `true` if the receiver is the root fiber, i.e.
/// it has no resumer to hand control back to.
///
/// This is the *predicate form* of [`fiber_yield`]'s first refusal: a root
/// fiber cannot yield, because there is nowhere to yield to. Exposing it as a
/// getter lets `.ph` code ask the question **before** attempting a switch.
///
/// That distinction is the whole reason this primitive exists. `Future#await`
/// (`core/core.ph`) previously discovered its own root-ness by wrapping a
/// `Fiber.yield` in `{ … }.attempt()` and inspecting the failure — but
/// `.attempt()` is itself two nested native re-entrant frames (`block_on` +
/// `block_call`, each bumping [`crate::vm::VM::native_reentry_depth`]), so the
/// probe tripped the restricted-yield guard it was probing for and `await`
/// could never suspend any fiber at all ([E004]). Attempt-and-inspect cannot
/// work when the attempt changes the answer; a predicate can.
///
/// Like `isDone`/`error`, a pure read with no scheduler or suspension
/// dependency — callable from anywhere, including under a native re-entrant
/// frame ([U-FIBER-REFLECT]).
///
/// [U-FIBER-REFLECT]: ../../../docs/forge/units/U-SCHED-FIBER/U-FIBER-REFLECT/plan.md
/// [E004]: ../../../docs/errors/E004-await-cannot-suspend.md
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `receiver` is not a `Fiber`.
pub fn fiber_is_root(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let fiber_ref = expect_fiber(vm, receiver)?;
    Ok(Value::Bool(vm.heap.fiber(fiber_ref).resumer.is_none()))
}

/// Signature: `Fiber#error` — the captured `Error` as `Option`, if the
/// receiver is `Failed`; `None` otherwise (including `Done`, where
/// [`crate::heap::FiberObject::result`] holds the return value, not an
/// `Error`) ([U-FIBER-REFLECT]).
///
/// [U-FIBER-REFLECT]: ../../../docs/forge/units/U-SCHED-FIBER/U-FIBER-REFLECT/plan.md
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `receiver` is not a `Fiber`.
pub fn fiber_error(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let fiber_ref = expect_fiber(vm, receiver)?;
    let fiber = vm.heap.fiber(fiber_ref);
    if fiber.status == FiberStatus::Failed {
        let error = fiber.result;
        Ok(crate::primitive::nil::wrap_some(vm, error))
    } else {
        Ok(vm.none_value())
    }
}

/// Signature: `Fiber::abort(_)` — raises `args[0]` at the fiber floor, caught
/// by `VM::run_until`'s fiber-floor capture exactly like any other raise.
///
/// # Errors
///
/// Returns [`RuntimeError::NotAllowed`] if the current fiber is the root
/// (has no resumer) — the root fiber has nowhere to propagate a fiber-floor
/// capture to, so aborting it is illegal (spec §2 rule 7, §6). Otherwise
/// returns [`RuntimeError::Raise`] wrapping `args[0]`.
pub fn fiber_abort(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let me = vm.current;
    if vm.heap.fiber(me).resumer.is_none() {
        return Err(RuntimeError::NotAllowed("cannot abort the root fiber".to_string()).into());
    }
    let error = args[0];
    let rendered = error.to_string(vm);
    Err(RuntimeError::Raise { error, rendered }.into())
}

/// Signature: `Fiber#call`/`call(_)` — resumes the receiver fiber, re-raising
/// an uncaught failure into the resumer ([`FiberResumeMode::Call`]).
pub fn fiber_call(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    fiber_resume(vm, receiver, args, FiberResumeMode::Call)
}

/// Signature: `Fiber#try`/`try(_)` — resumes the receiver fiber, capturing an
/// uncaught failure as the delivered `Error` value ([`FiberResumeMode::Try`]).
pub fn fiber_try(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    fiber_resume(vm, receiver, args, FiberResumeMode::Try)
}

/// Shared engine behind [`fiber_call`]/[`fiber_try`] (ADR-0030 §3/§4).
///
/// Parks the current fiber, switches `VM::current` to the callee, and
/// either pushes its entry frame fresh (first resume) or restores its parked
/// stacks and delivers `args[0]` at its own recorded `yield` site (a
/// subsequent resume). Sets `VM::switch_pending` so `VM::call_method`
/// skips ordinary post-call stack reconciliation — the callee's own
/// eventual completion/failure is delivered later, by
/// `VM::run_until`'s fiber-floor capture.
///
/// # Errors
///
/// Returns [`RuntimeError::NotAllowed`] if the callee is `Done`/`Failed`/
/// already `Running`, [`RuntimeError::Arity`] on a first-resume argument
/// mismatch, or the `CannotYieldAcrossNativeFrame` error (with a
/// resume-specific message, see [`cannot_resume_across_native_frame`]) if
/// `VM::native_reentry_depth` is nonzero.
fn fiber_resume(vm: &mut VM, receiver: &Value, args: &[Value], mode: FiberResumeMode) -> PhResult<Value> {
    if vm.native_reentry_depth != 0 {
        return Err(cannot_resume_across_native_frame(vm));
    }
    let callee_ref = expect_fiber(vm, receiver)?;
    match vm.heap.fiber(callee_ref).status {
        FiberStatus::Done | FiberStatus::Failed => {
            return Err(RuntimeError::NotAllowed("cannot resume a finished fiber".to_string()).into());
        }
        FiberStatus::Running => {
            return Err(RuntimeError::NotAllowed("fiber is already running".to_string()).into());
        }
        FiberStatus::Suspended => {}
    }

    // Resolve and validate the entry callable *before* any state mutation
    // (the resumer steal below): an early return here must leave the calling
    // fiber's live stacks and `vm.current` completely untouched, since this
    // check can fail for an ordinary usage error (wrong arity) that has
    // nothing to do with the callee having actually started running. Doing
    // this after `store_live_into` was a real bug — see the regression
    // golden `fiber_first_resume_arity_mismatch_does_not_corrupt_resumer.ph`.
    let started = vm.heap.fiber(callee_ref).started;
    let entry_call = if !started {
        let entry = vm.heap.fiber(callee_ref).entry.expect("non-root fiber always has an entry");
        let (closure_id, home_frame_token) = match vm.heap.get(entry) {
            Object::Block(block) => (block.closure, Some(block.home_frame_token)),
            Object::Closure(_) => (entry, None),
            _ => unreachable!("fiber_new only accepts Block/Closure entries"),
        };
        let arity = vm.heap.closure(closure_id).callable.arity;
        if args.len() != arity {
            let signature = match mode {
                FiberResumeMode::Call => "call",
                FiberResumeMode::Try => "try",
            };
            return Err(RuntimeError::Arity { signature, expected: arity, found: args.len() }.into());
        }
        Some((entry, closure_id, home_frame_token))
    } else {
        None
    };

    let receiver_idx = vm.stack.len() - 1 - args.len();
    let resumer_ref = vm.current;
    vm.heap.fiber_mut(resumer_ref).resume_slot = receiver_idx;
    store_live_into(vm, resumer_ref);

    vm.heap.fiber_mut(callee_ref).resumer = Some(resumer_ref);
    vm.heap.fiber_mut(callee_ref).resume_mode = mode;

    if let Some((entry, closure_id, home_frame_token)) = entry_call {
        // `vm.stack`/`vm.frames` are empty here (just taken by
        // `store_live_into` above), so the callee's fresh window starts at 0.
        let stack_offset = vm.stack.len();
        vm.stack.push(Value::Obj(entry));
        vm.stack.extend_from_slice(args);
        let mut frame = vm.new_call_frame(closure_id, CallContext::Instance { instance: entry }, 0, stack_offset, None);
        frame.home_frame_token = home_frame_token;
        vm.push_frame(frame)?;
        vm.heap.fiber_mut(callee_ref).started = true;
    } else {
        load_live_from(vm, callee_ref);
        let delivered = args.first().copied().unwrap_or_else(|| vm.none_value());
        let slot = vm.heap.fiber(callee_ref).resume_slot;
        vm.stack.truncate(slot);
        vm.stack.push(delivered);
    }

    vm.heap.fiber_mut(callee_ref).status = FiberStatus::Running;
    vm.heap.fiber_mut(callee_ref).floor_depth = vm.native_reentry_depth;
    vm.current = callee_ref;
    vm.switch_pending = true;
    Ok(Value::Nil)
}

/// Signature: `Fiber::yield`/`yield(_)` — suspends the current fiber and
/// hands control back to its resumer, delivering `args[0]` (or `None`) as the
/// resumer's `call`/`try` result.
///
/// # Errors
///
/// Returns [`RuntimeError::NotAllowed`] if the current fiber is the root
/// (has no resumer), or the `CannotYieldAcrossNativeFrame` error if
/// `VM::native_reentry_depth` has grown past the fiber's recorded
/// `floor_depth` since it was last resumed (ADR-0030 §4).
pub fn fiber_yield(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let me = vm.current;
    let Some(resumer) = vm.heap.fiber(me).resumer else {
        return Err(RuntimeError::NotAllowed("cannot yield the root fiber".to_string()).into());
    };
    if vm.native_reentry_depth != vm.heap.fiber(me).floor_depth {
        return Err(cannot_yield_across_native_frame(vm));
    }

    let receiver_idx = vm.stack.len() - 1 - args.len();
    let value = args.first().copied().unwrap_or_else(|| vm.none_value());

    vm.heap.fiber_mut(me).resume_slot = receiver_idx;
    vm.heap.fiber_mut(me).status = FiberStatus::Suspended;
    store_live_into(vm, me);

    vm.switch_to_fiber_and_deliver(resumer, value);
    vm.switch_pending = true;
    Ok(Value::Nil)
}
