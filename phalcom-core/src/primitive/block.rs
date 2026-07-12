//! Native primitives for `Function` and `Block`.
//!
//! The `Function` root is abstract and `Block` is the concrete first-class
//! callable object ([ADR-0006](../../docs/adr/0006-function-as-abstract-callable-root.md),
//! [ADR-0013](../../docs/adr/0013-block-closure-upvalues.md)). These
//! primitives expose the reflective surface (`arity`, `name`) and dispatch
//! `call`/`call(_:…)` by pushing a fresh [`CallFrame`](crate::frame::CallFrame)
//! for the block's closure and re-entering `VM::run_until` with the current
//! frame count as the base, so the call returns its result without draining
//! the caller's own frames (functions.md §1-2).

use crate::error::{PhResult, RuntimeError};
use crate::frame::{CallContext, FrameToken};
use crate::heap::Object;
use crate::value::Value;
use crate::vm::VM;

/// Resolves `receiver` to the [`crate::closure::ClosureObject`] handle it calls
/// through, together with the block's home-frame token (`None` when the receiver
/// is not a block).
///
/// A [`Object::Block`] unwraps to its wrapped closure and surfaces its
/// [`home_frame_token`](crate::block::BlockObject::home_frame_token) so
/// [`block_call`] can stamp the pushed frame for non-local return
/// ([ADR-0013](../../docs/adr/0013-block-closure-upvalues.md)); a bare
/// [`Object::Closure`] (e.g. a `Method`'s callable reflectively used as a
/// `Function`, functions.md) is its own target and has no lexical home frame, so
/// it yields `None` — its body compiles ordinary [`Bytecode::Return`](crate::bytecode::Bytecode::Return)
/// and can never issue a non-local return.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `receiver` is neither a block nor a closure.
fn resolve_callable(vm: &VM, receiver: &Value) -> PhResult<(crate::heap::ObjRef, Option<FrameToken>)> {
    match receiver {
        Value::Obj(id) => match vm.heap.get(*id) {
            Object::Block(block) => Ok((block.closure, Some(block.home_frame_token))),
            Object::Closure(_) => Ok((*id, None)),
            // `Method` `isA Function` and answers the *reflective* protocol
            // (`arity`/`name`/`selector`/`holder`/`bind`/`invokeOn`), but not
            // raw `call` while unbound — it has no receiver to run against
            // (U-CORE-3, [ADR-0028](../../docs/adr/0028-amend-floor-admit-method-reflection.md)).
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
/// (U-CORE-3) — neither has a [`ClosureObject`](crate::closure::ClosureObject)
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
/// [`ClosureObject`](crate::closure::ClosureObject) to resolve (a bound
/// *primitive* method has none at all), and this is what makes
/// `bound.call(args) ≡ method.invokeOn(recv, args)` hold by construction
/// (R-INV-3.3, U-CORE-3, [ADR-0028](../../docs/adr/0028-amend-floor-admit-method-reflection.md)).
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
    vm.frames.push(frame);
    // Re-entrant native frame — the crown-jewel hazard (ADR-0030 §4): a fiber
    // switch is forbidden while this recursive `run_until` is live on the
    // Rust call stack (`native_reentry_depth`, `vm.rs`). This is precisely
    // what makes `.each { Fiber.yield(x) }` raise `CannotYieldAcrossNativeFrame`
    // instead of corrupting the suspended position.
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
/// (Invariant 4, [ADR-0007](../../../docs/adr/0007-option-some-none.md)). This
/// is what the inliner's
/// `GuardBlock` deopt path sends to
/// ([ADR-0018](../../../docs/adr/0018-sacred-selector-inliner-and-override-guard.md)).
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