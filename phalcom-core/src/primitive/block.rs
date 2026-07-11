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
use crate::frame::CallContext;
use crate::heap::Object;
use crate::value::Value;
use crate::vm::VM;

/// Resolves `receiver` to the [`crate::closure::ClosureObject`] handle it calls
/// through — a [`Object::Block`] unwraps to its wrapped closure, and a bare
/// [`Object::Closure`] (e.g. a `Method`'s callable used as a `Function`) is its
/// own target.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `receiver` is neither.
fn resolve_callable(vm: &VM, receiver: &Value) -> PhResult<crate::heap::ObjRef> {
    match receiver {
        Value::Obj(id) => match vm.heap.get(*id) {
            Object::Block(block) => Ok(block.closure),
            Object::Closure(_) => Ok(*id),
            _ => Err(RuntimeError::Type { expected: "Function", found: receiver.type_name() }.into()),
        },
        other => Err(RuntimeError::Type { expected: "Function", found: other.type_name() }.into()),
    }
}

/// Returns the callable's arity.
pub fn block_arity(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    match receiver {
        Value::Obj(id) => match vm.heap.get(*id) {
            Object::Closure(closure) => Ok(Value::Number(closure.callable.arity as f64)),
            Object::Block(block) => {
                let closure = vm.heap.closure(block.closure);
                Ok(Value::Number(closure.callable.arity as f64))
            }
            _ => Err(RuntimeError::Type { expected: "Function", found: receiver.type_name() }.into()),
        },
        other => Err(RuntimeError::Type { expected: "Function", found: other.type_name() }.into()),
    }
}

/// Returns the callable's display name.
pub fn block_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let name = match receiver {
        Value::Obj(id) => match vm.heap.get(*id) {
            Object::Closure(closure) => vm.resolve_symbol(closure.callable.name_sym).to_string(),
            Object::Block(block) => {
                let closure = vm.heap.closure(block.closure);
                vm.resolve_symbol(closure.callable.name_sym).to_string()
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
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `receiver` is not callable,
/// [`RuntimeError::Arity`] on an argument-count mismatch, or any
/// [`RuntimeError`] raised while running the block body.
pub fn block_call(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let closure_id = resolve_callable(vm, receiver)?;
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
    let frame = vm.new_call_frame(closure_id, context, 0, stack_offset, None);
    vm.frames.push(frame);
    vm.run_until(base_frames)
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