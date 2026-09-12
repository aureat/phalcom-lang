//! Native primitives for `Function` and `Block`.
//!
//! The `Function` root is abstract and `Block` is the concrete first-class
//! callable object ([ADR-0006](../../docs/adr/accepted/0006-function-as-abstract-callable-root.md),
//! [ADR-0013](../../docs/adr/accepted/0013-block-closure-upvalues.md)). These
//! primitives expose the reflective surface (`arity`, `name`) and route
//! ordinary `call(***)`/`callWith(_)` through the VM's flat Function gateway.
//! The legacy [`block_call`] helper still re-enters `VM::run_until` for
//! explicitly synchronous native combinators such as `on` and `ensure`
//! (functions.md §1-2).

use crate::error::{PhResult, RuntimeError};
use crate::frame::{CallContext, FrameToken};
use crate::heap::Object;
use crate::method::{ArgumentView, CallOutcome, InvocationLayout};
use crate::parameters::{ArgumentShape, RestKind};
use crate::value::Value;
use crate::vm::control::{ControlDestination, ControlPhase};
use crate::vm::VM;
use phalcom_common::range::SourceRange;

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
    if let Some(id) = receiver.as_obj() {
        match vm.heap.get(id) {
            Object::Block(block) => Ok((block.closure, Some(block.home_frame_token))),
            Object::Closure(_) => Ok((id, None)),
            Object::Method(_) => Err(RuntimeError::NotAllowed("unbound Method — use bind(_) or invokeOn(_,***)".to_string()).into()),
            _ => Err(RuntimeError::Type {
                expected: "Function",
                found: receiver.type_name(),
            }
            .into()),
        }
    } else {
        Err(RuntimeError::Type {
            expected: "Function",
            found: receiver.type_name(),
        }
        .into())
    }
}

/// Returns the callable's arity.
///
/// A [`Object::Method`] reports its signature's positional arity; an
/// [`Object::BoundMethod`] delegates to the arity of the method it wraps
/// (U-CORE-3) — neither has a [`ClosureObject`](crate::heap::ClosureObject)
/// to read `arity` off directly, unlike `Block`/`Closure`.
#[phalcom_native_macros::primitive(Function, "arity")]
pub fn block_arity(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if let Some(id) = receiver.as_obj() {
        match vm.heap.get(id) {
            Object::Closure(closure) => Ok(Value::int(closure.callable.arity as i64)),
            Object::Block(block) => {
                let closure = vm.heap.closure(block.closure);
                Ok(Value::int(closure.callable.arity as i64))
            }
            Object::Method(method) => Ok(Value::int(method.signature.positional_arity as i64)),
            Object::BoundMethod(bound) => Ok(Value::int(vm.heap.method(bound.method).signature.positional_arity as i64)),
            _ => Err(RuntimeError::Type {
                expected: "Function",
                found: receiver.type_name(),
            }
            .into()),
        }
    } else {
        Err(RuntimeError::Type {
            expected: "Function",
            found: receiver.type_name(),
        }
        .into())
    }
}

#[phalcom_native_macros::primitive(Closure, "arity")]
pub fn closure_arity(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    block_arity(vm, receiver, args)
}

/// Returns the callable's display name.
///
/// A [`Object::Method`] renders its encoded selector text; an
/// [`Object::BoundMethod`] delegates to the name of the method it wraps
/// (U-CORE-3).
#[phalcom_native_macros::primitive(Function, "name")]
pub fn block_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let name = if let Some(id) = receiver.as_obj() {
        match vm.heap.get(id) {
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
            _ => {
                return Err(RuntimeError::Type {
                    expected: "Function",
                    found: receiver.type_name(),
                }
                .into());
            }
        }
    } else {
        return Err(RuntimeError::Type {
            expected: "Function",
            found: receiver.type_name(),
        }
        .into());
    };
    Ok(vm.alloc_string_value(name))
}

#[phalcom_native_macros::primitive(Closure, "name")]
pub fn closure_name(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    block_name(vm, receiver, args)
}

/// Shape-aware `Function#call(***)` gateway. The VM has already selected the
/// concrete `call` rest method, so this activation only unwraps the sealed
/// callable representation and reuses the current stack window.
#[phalcom_native_macros::primitive(Function, "call(***)", abi = shape)]
pub fn block_call_shape(vm: &mut VM, receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    vm.activate_function(receiver, args, phalcom_common::range::SourceRange::default())
}

/// Shape-aware `Function#callWith(_)` gateway. A complete pack is copied into
/// the existing argument window and forwarded through the same `call` gateway
/// as ordinary invocation. Unit represents an empty pack; Tuple is the only
/// heap-backed complete-pack representation.
#[phalcom_native_macros::primitive(Function, "callWith(_)", abi = shape)]
pub fn block_call_with_shape(vm: &mut VM, receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    let packed = args.positional(vm, 0).ok_or_else(|| RuntimeError::Arity {
        signature: "callWith",
        expected: 1,
        found: args.positional_count(),
    })?;
    let (positionals, labeled): (Vec<Value>, Vec<(crate::interner::Symbol, Value)>) = if packed.is_unit() {
        (Vec::new(), Vec::new())
    } else if let Some(id) = packed.as_obj() {
        if matches!(vm.heap.get(id), Object::Tuple(_)) {
            let tuple = vm.heap.tuple(id);
            (tuple.positionals().to_vec(), tuple.labeled_entries().collect())
        } else {
            return Err(RuntimeError::Type {
                expected: "Tuple",
                found: packed.type_name(),
            }
            .into());
        }
    } else {
        return Err(RuntimeError::Type {
            expected: "Tuple",
            found: packed.type_name(),
        }
        .into());
    };

    let receiver_index = args.receiver_index();
    vm.stack.truncate(receiver_index + 1);
    vm.stack.extend_from_slice(&positionals);
    vm.stack.extend(labeled.iter().map(|(_, value)| *value));

    let label_syms: Box<[crate::interner::Symbol]> = labeled.iter().map(|(label, _)| *label).collect();
    let shaped = args.with_layout(InvocationLayout::ordinary(positionals.len(), label_syms));
    vm.activate_function(receiver, shaped, phalcom_common::range::SourceRange::default())
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
/// `bound.call(***args) ≡ method.invokeOn(recv, ***args)` hold by construction
/// (R-INV-3.3, U-CORE-3, [ADR-0028](../../docs/adr/accepted/0028-amend-floor-admit-method-reflection.md)).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `receiver` is not callable,
/// [`RuntimeError::Arity`] on an argument-count mismatch, or any
/// [`RuntimeError`] raised while running the block body.
pub fn block_call(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if let Some(id) = receiver.as_obj() {
        if let Object::BoundMethod(bound) = vm.heap.get(id) {
            let (method_id, target) = (bound.method, bound.receiver);
            return vm.invoke_method_object(method_id, target, args);
        }
    }

    let (closure_id, home_frame_token) = resolve_callable(vm, receiver)?;
    let shape = vm.heap.closure(closure_id).callable.parameter_shape.clone();
    let argument_shape = ArgumentShape::positional(args.len());
    if !shape.accepts(&argument_shape) {
        return Err(RuntimeError::Arity {
            signature: "call",
            expected: shape.fixed_positionals,
            found: args.len(),
        }
        .into());
    }

    let mut bound_args = Vec::with_capacity(shape.fixed_positionals + usize::from(shape.rest.is_some()));
    bound_args.extend_from_slice(&args[..shape.fixed_positionals]);
    if matches!(shape.rest, Some(RestKind::Positional)) {
        let rest = crate::product::finish_tuple(vm, args[shape.fixed_positionals..].to_vec(), Vec::new())
            .map_err(|error| crate::product::runtime_error(vm, "Tuple label", error))?;
        bound_args.push(rest);
    }

    // Slot 0 of the callee's stack window is a dummy receiver slot (blocks
    // reach `self` through a captured upvalue, not this slot — see
    // `compile_block`); push it followed by the arguments.
    let stack_offset = vm.stack.len();
    vm.stack.push(*receiver);
    vm.stack.extend_from_slice(&bound_args);

    let base_frames = vm.frames.len();
    let context = CallContext::Instance {
        instance: receiver.as_obj().expect("resolve_callable only accepts Value::Obj"),
    };
    let mut frame = vm.new_call_frame(closure_id, context, 0, stack_offset, None);
    // Stamp the block activation with its lexical home frame so a `return` in
    // the block body (compiled to `Bytecode::ReturnNonLocal`) can unwind to the
    // enclosing method rather than just this block frame (ADR-0013, blocks.md
    // §5). `None` for a bare closure receiver, which has no home frame. Setting
    // the field post-construction (rather than threading a `new_call_frame`
    // parameter) keeps `new_call_frame`'s signature stable and is sound because
    // `CallFrame` is `Copy`.
    frame.home_frame_token = home_frame_token;
    frame.foreign_receiver_guard = vm.heap.closure(closure_id).foreign_receiver_guard;
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

/// Signature: `Block::whileTrue(_)` — sacred loop fallback (control-flow.md
/// §1/§3: `while (c) { B }` desugars to `{ c }.whileTrue { B }`). Calls the
/// receiver block each iteration as the condition; if its result is not a
/// `Bool`, raises a type error (this is Phalcom's "no truthiness" floor —
/// there is no generic coercion, only `Bool` may drive a branch). Loops
/// while the condition is `true`, calling `args[0]` (the body) each pass and
/// discarding its result; returns immediate `None` (surface absence value)
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
/// Signature: `Block::whileTrue(_)` — sacred loop fallback (control-flow.md
/// §1/§3: `while (c) { B }` desugars to `{ c }.whileTrue { B }`). Calls the
/// receiver block each iteration as the condition; if its result is not a
/// `Bool`, raises a type error.
#[phalcom_native_macros::primitive(Closure, "whileTrue(_)", abi = shape)]
pub fn block_while_true_shape(vm: &mut VM, receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    let body = args.positional(vm, 0).ok_or_else(|| RuntimeError::Arity {
        signature: "whileTrue",
        expected: 1,
        found: args.positional_count(),
    })?;

    let receiver_idx = args.receiver_index();
    let callback_floor = vm.frames.len();
    let owner = vm.frames.last().and_then(|f| f.home_frame_token);
    let caller_auth = (vm.current_access_class(), vm.current_has_internal_privilege());
    let source_range = SourceRange::default();

    vm.control_stack.push(
        owner,
        receiver_idx,
        callback_floor,
        caller_auth,
        source_range,
        ControlDestination::StackOperand { target_index: receiver_idx },
        ControlPhase::WhileCondition { condition: receiver, body },
    );

    let layout = InvocationLayout::ordinary(0, Box::new([]));
    let view = ArgumentView::from_layout(receiver_idx, layout, caller_auth.0, caller_auth.1);
    vm.activate_function(receiver, view, source_range)
}

pub fn block_while_true(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let receiver_idx = vm.stack.len();
    vm.stack.push(*receiver);
    vm.stack.extend_from_slice(args);
    let layout = InvocationLayout::ordinary(args.len(), Box::new([]));
    let view = ArgumentView::from_layout(receiver_idx, layout, vm.current_access_class(), vm.current_has_internal_privilege());
    match block_while_true_shape(vm, *receiver, view)? {
        CallOutcome::Returned(v) => Ok(v),
        CallOutcome::EnteredFrame | CallOutcome::EnteredControl => {
            vm.check_native_reentry()?;
            vm.native_reentry_depth += 1;
            let res = vm.run_until(receiver_idx);
            vm.native_reentry_depth -= 1;
            res
        }
        CallOutcome::SwitchedFiber => Ok(Value::nil()),
    }
}

/// Signature: `Block::on(_)(_)` — the typed catch primitive `try`/`on`/`catch`
/// desugar to (error-handling.md §2, [ADR-0008](../../../docs/adr/accepted/0008-layered-exceptions-and-result.md),
/// [ADR-0038](../../../docs/adr/accepted/0038-amend-floor-admit-block-on-ensure.md)).
#[phalcom_native_macros::primitive(Closure, "on(_,_)", abi = shape)]
pub fn block_on_shape(vm: &mut VM, receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    let class_arg = args.positional(vm, 0).ok_or_else(|| RuntimeError::Arity {
        signature: "on",
        expected: 2,
        found: args.positional_count(),
    })?;
    let is_class = class_arg.as_obj().is_some_and(|id| matches!(vm.heap.get(id), Object::Class(_)));
    if !is_class {
        return Err(RuntimeError::Type {
            expected: "Class",
            found: class_arg.type_name(),
        }
        .into());
    }
    let handler = args.positional(vm, 1).ok_or_else(|| RuntimeError::Arity {
        signature: "on",
        expected: 2,
        found: args.positional_count(),
    })?;

    let receiver_idx = args.receiver_index();
    let callback_floor = vm.frames.len();
    let owner = vm.frames.last().and_then(|f| f.home_frame_token);
    let caller_auth = (vm.current_access_class(), vm.current_has_internal_privilege());
    let source_range = SourceRange::default();

    vm.control_stack.push(
        owner,
        receiver_idx,
        callback_floor,
        caller_auth,
        source_range,
        ControlDestination::StackOperand { target_index: receiver_idx },
        ControlPhase::OnBody { class: class_arg, handler },
    );

    let layout = InvocationLayout::ordinary(0, Box::new([]));
    let view = ArgumentView::from_layout(receiver_idx, layout, caller_auth.0, caller_auth.1);
    vm.activate_function(receiver, view, source_range)
}

pub fn block_on(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let receiver_idx = vm.stack.len();
    vm.stack.push(*receiver);
    vm.stack.extend_from_slice(args);
    let layout = InvocationLayout::ordinary(args.len(), Box::new([]));
    let view = ArgumentView::from_layout(receiver_idx, layout, vm.current_access_class(), vm.current_has_internal_privilege());
    match block_on_shape(vm, *receiver, view)? {
        CallOutcome::Returned(v) => Ok(v),
        CallOutcome::EnteredFrame | CallOutcome::EnteredControl => {
            vm.check_native_reentry()?;
            vm.native_reentry_depth += 1;
            let res = vm.run_until(receiver_idx);
            vm.native_reentry_depth -= 1;
            res
        }
        CallOutcome::SwitchedFiber => Ok(Value::nil()),
    }
}

/// Signature: `Block::ensure(_)` — the always-runs cleanup primitive `try`/
/// `ensure` desugars to (error-handling.md §4, ADR-0008 §4.1,
/// [ADR-0038](../../../docs/adr/accepted/0038-amend-floor-admit-block-on-ensure.md)).
#[phalcom_native_macros::primitive(Closure, "ensure(_)", abi = shape)]
pub fn block_ensure_shape(vm: &mut VM, receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    let cleanup = args.positional(vm, 0).ok_or_else(|| RuntimeError::Arity {
        signature: "ensure",
        expected: 1,
        found: args.positional_count(),
    })?;

    let receiver_idx = args.receiver_index();
    let callback_floor = vm.frames.len();
    let owner = vm.frames.last().and_then(|f| f.home_frame_token);
    let caller_auth = (vm.current_access_class(), vm.current_has_internal_privilege());
    let source_range = SourceRange::default();

    vm.control_stack.push(
        owner,
        receiver_idx,
        callback_floor,
        caller_auth,
        source_range,
        ControlDestination::StackOperand { target_index: receiver_idx },
        ControlPhase::EnsureBody { cleanup },
    );

    let layout = InvocationLayout::ordinary(0, Box::new([]));
    let view = ArgumentView::from_layout(receiver_idx, layout, caller_auth.0, caller_auth.1);
    vm.activate_function(receiver, view, source_range)
}

pub fn block_ensure(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let receiver_idx = vm.stack.len();
    vm.stack.push(*receiver);
    vm.stack.extend_from_slice(args);
    let layout = InvocationLayout::ordinary(args.len(), Box::new([]));
    let view = ArgumentView::from_layout(receiver_idx, layout, vm.current_access_class(), vm.current_has_internal_privilege());
    match block_ensure_shape(vm, *receiver, view)? {
        CallOutcome::Returned(v) => Ok(v),
        CallOutcome::EnteredFrame | CallOutcome::EnteredControl => {
            vm.check_native_reentry()?;
            vm.native_reentry_depth += 1;
            let res = vm.run_until(receiver_idx);
            vm.native_reentry_depth -= 1;
            res
        }
        CallOutcome::SwitchedFiber => Ok(Value::nil()),
    }
}
