//! Native primitives for `Function` and `Block`.
//!
//! The `Function` root is abstract and `Block` is the concrete first-class
//! callable object ([ADR-0006](../../docs/adr/accepted/0006-function-as-abstract-callable-root.md),
//! [ADR-0013](../../docs/adr/accepted/0013-block-closure-upvalues.md)). These
//! primitives expose the reflective surface (`arity`, `name`) and dispatch
//! `call`/`call(_:…)` by pushing a fresh [`CallFrame`](crate::frame::CallFrame)
//! for the block's closure and re-entering `VM::run_until` with the current
//! frame count as the base, so the call returns its result without draining
//! the caller's own frames (functions.md §1-2).

use crate::error::{PhError, PhResult, RuntimeError};
use crate::frame::{CallContext, FrameToken};
use crate::heap::Object;
use crate::value::Value;
use crate::vm::VM;

/// Resolves `receiver` to the [`crate::heap::ClosureObject`] handle it calls
/// through, together with the block's home-frame token (`None` when the receiver
/// is not a block).
///
/// A [`Object::Block`] unwraps to its wrapped closure and surfaces its
/// [`home_frame_token`](crate::heap::BlockObject::home_frame_token) so
/// [`block_call`] can stamp the pushed frame for non-local return
/// ([ADR-0013](../../docs/adr/accepted/0013-block-closure-upvalues.md)); a bare
/// [`Object::Closure`] (e.g. a `Method`'s callable reflectively used as a
/// `Function`, functions.md) is its own target and has no lexical home frame, so
/// it yields `None` — its body compiles ordinary [`Bytecode::Return`](crate::bytecode::Bytecode::Return)
/// and can never issue a non-local return.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `receiver` is neither a block nor a closure.
pub(crate) fn resolve_callable(vm: &VM, receiver: &Value) -> PhResult<(crate::heap::ObjRef, Option<FrameToken>)> {
    match receiver {
        Value::Obj(id) => match vm.heap.get(*id) {
            Object::Block(block) => Ok((block.closure, Some(block.home_frame_token))),
            Object::Closure(_) => Ok((*id, None)),
            // `Method` `isA Function` and answers the *reflective* protocol
            // (`arity`/`name`/`selector`/`holder`/`bind`/`invokeOn`), but not
            // raw `call` while unbound — it has no receiver to run against
            // (U-CORE-3, [ADR-0028](../../docs/adr/accepted/0028-amend-floor-admit-method-reflection.md)).
            // A dedicated arm ahead of the wildcard sharpens the error over
            // the generic "expected Function, found Method" it would
            // otherwise fall through to.
            Object::Method(_) => Err(RuntimeError::NotAllowed("unbound Method — use bind(_) or invokeOn(_,_)".to_string()).into()),
            _ => Err(RuntimeError::Type { expected: "Function", found: receiver.type_name() }.into()),
        },
        other => Err(RuntimeError::Type { expected: "Function", found: other.type_name() }.into()),
    }
}

/// Returns the callable's arity.
///
/// A [`Object::Method`] reports its signature's positional arity; an
/// [`Object::BoundMethod`] delegates to the arity of the method it wraps
/// (U-CORE-3) — neither has a [`ClosureObject`](crate::heap::ClosureObject)
/// to read `arity` off directly, unlike `Block`/`Closure`.
pub fn block_arity(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    match receiver {
        Value::Obj(id) => match vm.heap.get(*id) {
            Object::Closure(closure) => Ok(Value::Number(closure.callable.arity as f64)),
            Object::Block(block) => {
                let closure = vm.heap.closure(block.closure);
                Ok(Value::Number(closure.callable.arity as f64))
            }
            Object::Method(method) => Ok(Value::Number(method.signature.positional_arity as f64)),
            Object::BoundMethod(bound) => Ok(Value::Number(vm.heap.method(bound.method).signature.positional_arity as f64)),
            _ => Err(RuntimeError::Type { expected: "Function", found: receiver.type_name() }.into()),
        },
        other => Err(RuntimeError::Type { expected: "Function", found: other.type_name() }.into()),
    }
}

/// Returns the callable's display name.
///
/// A [`Object::Method`] renders its encoded selector text; an
/// [`Object::BoundMethod`] delegates to the name of the method it wraps
/// (U-CORE-3).
pub fn block_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let name = match receiver {
        Value::Obj(id) => match vm.heap.get(*id) {
            Object::Closure(closure) => vm.resolve_symbol(closure.callable.name_sym).to_string(),
            Object::Block(block) => {
                let closure = vm.heap.closure(block.closure);
                vm.resolve_symbol(closure.callable.name_sym).to_string()
            }
            Object::Method(method) => vm.resolve_symbol(method.signature.selector).to_string(),
            Object::BoundMethod(bound) => {
                let selector = vm.heap.method(bound.method).signature.selector;
                vm.resolve_symbol(selector).to_string()
            }
            _ => return Err(RuntimeError::Type { expected: "Function", found: receiver.type_name() }.into()),
        },
        other => return Err(RuntimeError::Type { expected: "Function", found: other.type_name() }.into()),
    };
    Ok(vm.alloc_string_value(name))
}

/// Calls the callable receiver with `args`, running its closure to completion
/// and returning its result (functions.md §1-2, `f(a, b)` desugars to
/// `f.call(a, b)`).
///
/// A [`Object::BoundMethod`] receiver (`Method#bind(_)`'s result) is
/// intercepted **before** `resolve_callable` and funnelled through
/// [`VM::invoke_method_object`] instead — it has no
/// [`ClosureObject`](crate::heap::ClosureObject) to resolve (a bound
/// *primitive* method has none at all), and this is what makes
/// `bound.call(args) ≡ method.invokeOn(recv, args)` hold by construction
/// (R-INV-3.3, U-CORE-3, [ADR-0028](../../docs/adr/accepted/0028-amend-floor-admit-method-reflection.md)).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `receiver` is not callable,
/// [`RuntimeError::Arity`] on an argument-count mismatch, or any
/// [`RuntimeError`] raised while running the block body.
pub fn block_call(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if let Value::Obj(id) = receiver {
        if let Object::BoundMethod(bound) = vm.heap.get(*id) {
            let (method_id, target) = (bound.method, bound.receiver);
            return vm.invoke_method_object(method_id, target, args);
        }
    }

    let (closure_id, home_frame_token) = resolve_callable(vm, receiver)?;
    let arity = vm.heap.closure(closure_id).callable.arity;
    if args.len() != arity {
        return Err(RuntimeError::Arity { signature: "call", expected: arity, found: args.len() }.into());
    }

    // Slot 0 of the callee's stack window is a dummy receiver slot (blocks
    // reach `self` through a captured upvalue, not this slot — see
    // `compile_block`); push it followed by the arguments.
    let stack_offset = vm.stack.len();
    vm.stack.push(*receiver);
    vm.stack.extend_from_slice(args);

    let base_frames = vm.frames.len();
    let context = CallContext::Instance { instance: match receiver {
        Value::Obj(id) => *id,
        _ => unreachable!("resolve_callable only accepts Value::Obj"),
    } };
    let mut frame = vm.new_call_frame(closure_id, context, 0, stack_offset, None);
    // Stamp the block activation with its lexical home frame so a `return` in
    // the block body (compiled to `Bytecode::ReturnNonLocal`) can unwind to the
    // enclosing method rather than just this block frame (ADR-0013, blocks.md
    // §5). `None` for a bare closure receiver, which has no home frame. Setting
    // the field post-construction (rather than threading a `new_call_frame`
    // parameter) keeps `new_call_frame`'s signature stable and is sound because
    // `CallFrame` is `Copy`.
    frame.home_frame_token = home_frame_token;
    vm.push_frame(frame)?;
    // Re-entrant native frame — the crown-jewel hazard (ADR-0030 §4): a fiber
    // switch is forbidden while this recursive `run_until` is live on the
    // Rust call stack (`native_reentry_depth`, `vm.rs`). This is precisely
    // what makes `.each { Fiber.yield(x) }` raise `CannotYieldAcrossNativeFrame`
    // instead of corrupting the suspended position.
    vm.check_native_reentry()?;
    vm.native_reentry_depth += 1;
    let result = vm.run_until(base_frames);
    vm.native_reentry_depth -= 1;
    result
}

/// Calls the callable receiver with a single packed-argument value.
///
/// `List` is not yet part of the kernel, so packed multi-argument calls are
/// deferred (see `docs/forge/DEFERRED.md`); a single value is forwarded as a
/// one-argument call, matching the common `callWith(_:)` case exercised
/// against a non-list value.
///
/// # Errors
///
/// See [`block_call`].
pub fn block_call_with(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    block_call(vm, receiver, args)
}

/// Signature: `Block::whileTrue(_)` — sacred loop fallback (control-flow.md
/// §1/§3: `while (c) { B }` desugars to `{ c }.whileTrue { B }`). Calls the
/// receiver block each iteration as the condition; if its result is not a
/// `Bool`, raises a type error (this is Phalcom's "no truthiness" floor —
/// there is no generic coercion, only `Bool` may drive a branch). Loops
/// while the condition is `true`, calling `args[0]` (the body) each pass and
/// discarding its result; returns the `None` singleton (surface absence value)
/// on normal exit, matching the sacred inliner's `Bytecode::Nil` result site
/// (Invariant 4, [ADR-0007](../../../docs/adr/accepted/0007-option-some-none.md)). This
/// is what the inliner's
/// `GuardBlock` deopt path sends to
/// ([ADR-0018](../../../docs/adr/accepted/0018-sacred-selector-inliner-and-override-guard.md)).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if a condition evaluation is not `Bool`,
/// or any error raised calling the condition/body blocks.
pub fn block_while_true(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    loop {
        let cond = block_call(vm, receiver, &[])?;
        let Value::Bool(cond) = cond else {
            return Err(RuntimeError::Type { expected: "Bool", found: cond.type_name() }.into());
        };
        if !cond {
            return Ok(vm.none_value());
        }
        block_call(vm, &args[0], &[])?;
    }
}

/// Signature: `Block::on(_)(_)` — the typed catch primitive `try`/`on`/`catch`
/// desugar to (error-handling.md §2, [ADR-0008](../../../docs/adr/accepted/0008-layered-exceptions-and-result.md),
/// [ADR-0038](../../../docs/adr/accepted/0038-amend-floor-admit-block-on-ensure.md)).
///
/// Runs the receiver block (`args[0]` unused — the receiver *is* the
/// protected block); if it completes with a caught `throw` whose `Error`
/// `isA(args[0])` (a `Class`), restores the VM to its pre-run snapshot and
/// runs the handler block `args[1]` with the caught `Error`, returning its
/// result. Any other outcome passes straight through:
///
/// - **Normal completion**, or an **`Ok` with a shrunk frame stack** (a
///   non-local `return` unwound *through* the protected block, U10) — `on`
///   does not catch a `return`; it is not a `throw` (ADR-0008 §4.2). Returned
///   unchanged.
/// - **A `Raise` whose `Error` does not match `args[0]`** — the original
///   `Err` is re-propagated, but only *after* `unwind_to` has torn the
///   protected block's frames down to this `on`'s own snapshot (the unwind
///   moved before the probe for PDR-0007 §2 — see the comment in the body).
///   An outer `on` still matches correctly because it records its own
///   snapshot at entry; a traceback rendered *above* this point sees the
///   stack only down to this boundary (first-match-wins, error-handling.md
///   §2; capture-at-boundary is PDR-0010 §3's job).
/// - **Any other `Err`** (`DeadFrameError`, a future fiber `abort` payload,
///   …) — **wrapped into a synthetic base `Error` instance** carrying the
///   rendered message, then run through the *same* `isA` probe as a real
///   `Raise`: a catch-all `on(Error)` catches it, a narrower class does not.
///   The native variant's identity is currently discarded in the wrap;
///   PDR-0010 §2's `kind` Symbol is the ruled fix. (This doc previously
///   claimed non-`Raise` errors were "re-propagated unchanged" — wrong on
///   both counts; error-handling-followups.md §3.)
///
/// The snapshot/restore is **length-relative** (`vm.stack.len()`/
/// `vm.frames.len()`, never an absolute index), so this stays fiber-local by
/// construction once a fiber owns its own frame/stack buffers (ADR-0030 D7,
/// forward-compat.md §7 D7) — never hardcode the main stack.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `args[0]` is not a `Class`. Propagates a
/// non-matching `Err`/an `isA` dispatch failure/any error raised running the
/// protected or handler block.
pub fn block_on(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let class_arg = args[0];
    let is_class = matches!(class_arg, Value::Obj(id) if matches!(vm.heap.get(id), Object::Class(_)));
    if !is_class {
        return Err(RuntimeError::Type { expected: "Class", found: class_arg.type_name() }.into());
    }
    let handler = args[1];

    let stack_len = vm.stack.len();
    let frames_len = vm.frames.len();
    let outcome = block_call(vm, receiver, &[]);

    match outcome {
        Ok(v) => Ok(v),
        Err(err) => {
            let error = match &err {
                PhError::Runtime(RuntimeError::Raise { error, .. }) => *error,
                _ => {
                    let error_class = vm.universe.classes.error_class;
                    let field_count = vm.heap.class(error_class).field_count;
                    let mut inst = crate::heap::InstanceObject::new(error_class, field_count);
                    inst.slots[0] = vm.alloc_string_value(err.to_string());
                    Value::Obj(vm.heap.alloc(crate::heap::Object::Instance(inst)))
                }
            };
            // `isA(_)` is an ordinary 1-arg `.ph` `Method` (`core.ph`'s
            // `Object#isA`), so its dispatch selector is the *encoded*
            // form `isA(_:)`, not the bare name — mirror the ordinary
            // `.ph` call-site encoding rather than hand-rolling the walk
            // in Rust (error-handling.md §2: "`on` catches typed by
            // `isA`", U-ERR plan §2.3).
            // Unwind *before* probing. The protected block's frames are dead the
            // moment the error escaped it, and the probe below is a full dynamic
            // send that needs frames of its own. Probing first meant running
            // recovery logic on top of an abandoned stack — which is merely untidy
            // until the stack is the resource that ran out, and then it is fatal:
            // a `RuntimeError::DepthExceeded` raised at the call-depth ceiling could
            // never be caught, because `isA` could not get a frame to run in
            // (PDR-0007 §2 requires the depth error be catchable).
            //
            // Unwinding here also covers the non-matching branch below, which
            // previously returned `Err` without unwinding at all.
            vm.unwind_to(stack_len, frames_len);

            let isa_sig = crate::method::encode_selector("isA", &[None], crate::method::SignatureKind::Method(1));
            let isa_sym = vm.get_or_intern(&isa_sig);
            let matched = vm.send_dynamic(error, isa_sym, &[class_arg])?;
            if matches!(matched, Value::Bool(true)) {
                block_call(vm, &handler, &[error])
            } else {
                Err(err)
            }
        }
    }
}

/// Signature: `Block::ensure(_)` — the always-runs cleanup primitive `try`/
/// `ensure` desugars to (error-handling.md §4, ADR-0008 §4.1,
/// [ADR-0038](../../../docs/adr/accepted/0038-amend-floor-admit-block-on-ensure.md)).
///
/// Runs the receiver (protected) block, then runs the cleanup block `args[0]`
/// on **every** exit path — normal completion, a non-local `return` unwinding
/// *through* the protected block, or an uncaught `throw` — and re-propagates
/// the protected block's original outcome unchanged. `ensure` never catches a
/// `Raise` (unlike [`block_on`]): an uncaught error's frames are left exactly
/// as `run_until` produced them, so an enclosing `on`/the top-level trace
/// renderer still sees the full stack.
///
/// **Cleanup-supersedes** (ADR-0008 §4.2): if the cleanup block itself
/// diverges — raises, or non-locally returns — that new outcome **replaces**
/// the pending one instead of being merely run for effect.
///
/// # Errors
///
/// Propagates whichever of the protected/cleanup outcomes wins per the
/// cleanup-supersedes rule above.
pub fn block_ensure(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let cleanup = args[0];
    let outcome = block_call(vm, receiver, &[]);

    // The pending outcome lives only in this Rust local while the cleanup block
    // runs, and that block re-enters the interpreter — so its back-edge
    // safepoint can collect. Neither `vm.stack` nor `vm.frames` describes
    // `outcome`, so without a temp root the collector frees it and `ensure`
    // returns a dangling handle (ADR-0050 §7; memory-management.md §4).
    //
    // Both arms carry a handle: `Ok` is the protected block's value, and a
    // `Raise` error is the surface `Error` instance an enclosing `on` will
    // receive.
    let roots = vm.temp_root_depth();
    match &outcome {
        Ok(value) => vm.push_temp_root(*value),
        Err(PhError::Runtime(RuntimeError::Raise { error, .. })) => vm.push_temp_root(*error),
        Err(_) => {}
    }

    let frames_before_cleanup = vm.frames.len();
    let cleanup_outcome = block_call(vm, &cleanup, &[]);
    vm.truncate_temp_roots(roots);

    match cleanup_outcome {
        // The cleanup block itself diverged (raised) — supersedes.
        Err(cleanup_err) => Err(cleanup_err),
        Ok(cleanup_value) => {
            if vm.frames.len() < frames_before_cleanup {
                // The cleanup block itself non-locally returned — supersedes.
                Ok(cleanup_value)
            } else {
                // Cleanup ran to ordinary completion for effect only; its
                // value is discarded and the original outcome resumes.
                outcome
            }
        }
    }
}