//! Native primitives on `Bool`.

use crate::error::{PhResult, RuntimeError};
use crate::method::{ArgumentView, CallOutcome, InvocationLayout};
use crate::primitive::block::block_call;
use crate::primitive::expect_class;
use crate::primitive::option::wrap_some;
use crate::value::Value;
use crate::value::{FALSE, TRUE};
use crate::vm::control::{BoolBranchKind, ControlDestination, ControlPhase};
use crate::vm::VM;
use phalcom_common::range::SourceRange;

/// Signature: `Bool.class::new(_)` — coerces its argument to a boolean.
#[phalcom_native_macros::primitive(
    Bool,
    "new(_)",
    params = [Object],
    returns = Bool,
    types = "(Object) -> Bool",
    side = class
)]
pub fn bool_class_new(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    expect_class(vm, receiver)?;
    let Some(arg) = args.first() else {
        return Err(RuntimeError::AbstractClass { class: "Bool" }.into());
    };
    if let Some(b) = arg.as_bool() {
        Ok(if b { TRUE } else { FALSE })
    } else if arg.is_nil() {
        Ok(FALSE)
    } else if let Some(n) = arg.as_int() {
        Ok(if n != 0 { TRUE } else { FALSE })
    } else if let Some(n) = arg.as_float() {
        Ok(if n != 0.0 { TRUE } else { FALSE })
    } else {
        Ok(TRUE)
    }
}

#[phalcom_native_macros::primitive(Bool, "new()", side = class)]
pub fn bool_class_new_default(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    bool_class_new(vm, receiver, &[])
}

/// Signature: `Bool::hash` — `1` for `true`, `0` for `false`.
#[phalcom_native_macros::primitive(
    Bool,
    "hash",
    params = [],
    returns = Int,
    types = "() -> Int",
    effects = pure
)]
pub fn bool_hash(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let bit = u64::from(receiver.as_bool() == Some(true));
    Ok(crate::primitive::hash_code(bit))
}

/// Extracts the `bool` payload of a `Bool` receiver.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a `Bool`.
fn expect_bool(value: &Value) -> PhResult<bool> {
    value.as_bool().ok_or_else(|| {
        RuntimeError::Type {
            expected: "Bool",
            found: value.type_name(),
        }
        .into()
    })
}

/// Signature: `Bool::and(_)` — sacred, lazy logical conjunction
#[phalcom_native_macros::primitive(
    Bool,
    "and(_)",
    params = [Object],
    returns = Object,
    types = "(Object) -> Object",
    intrinsic = BoolAnd,
    abi = shape
)]
pub fn bool_and_shape(vm: &mut VM, receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    if !expect_bool(&receiver)? {
        return Ok(CallOutcome::Returned(FALSE));
    }
    let block = args.positional(vm, 0).ok_or_else(|| RuntimeError::Arity {
        signature: "and",
        expected: 1,
        found: args.positional_count(),
    })?;
    let receiver_idx = args.receiver_index();
    vm.stack.truncate(receiver_idx);
    vm.stack.push(block);
    let layout = InvocationLayout::ordinary(0, Box::new([]));
    let view = ArgumentView::from_layout(receiver_idx, layout, vm.current_access_class(), vm.current_has_internal_privilege());
    vm.activate_function(block, view, SourceRange::default())
}

pub fn bool_and(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if !expect_bool(receiver)? {
        return Ok(FALSE);
    }
    block_call(vm, &args[0], &[])
}

/// Signature: `Bool::or(_)` — sacred, lazy logical disjunction
#[phalcom_native_macros::primitive(
    Bool,
    "or(_)",
    params = [Object],
    returns = Object,
    types = "(Object) -> Object",
    intrinsic = BoolOr,
    abi = shape
)]
pub fn bool_or_shape(vm: &mut VM, receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    if expect_bool(&receiver)? {
        return Ok(CallOutcome::Returned(TRUE));
    }
    let block = args.positional(vm, 0).ok_or_else(|| RuntimeError::Arity {
        signature: "or",
        expected: 1,
        found: args.positional_count(),
    })?;
    let receiver_idx = args.receiver_index();
    vm.stack.truncate(receiver_idx);
    vm.stack.push(block);
    let layout = InvocationLayout::ordinary(0, Box::new([]));
    let view = ArgumentView::from_layout(receiver_idx, layout, vm.current_access_class(), vm.current_has_internal_privilege());
    vm.activate_function(block, view, SourceRange::default())
}

pub fn bool_or(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if expect_bool(receiver)? {
        return Ok(TRUE);
    }
    block_call(vm, &args[0], &[])
}

/// Signature: `Bool::not` — sacred logical negation.
#[phalcom_native_macros::primitive(
    Bool,
    "not",
    params = [],
    returns = Bool,
    types = "() -> Bool",
    intrinsic = BoolNot,
    effects = pure
)]
pub fn bool_not(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(if expect_bool(receiver)? { FALSE } else { TRUE })
}

/// Signature: `Bool::ifTrue(_)` — sacred one-armed conditional.
/// Executes block if receiver is true.
#[phalcom_native_macros::primitive(
    Bool,
    "ifTrue(_)",
    params = [Object],
    returns = Option,
    types = "(Object) -> Option",
    abi = shape
)]
pub fn bool_if_true_shape(vm: &mut VM, receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    if !expect_bool(&receiver)? {
        return Ok(CallOutcome::Returned(vm.none_value()));
    }
    let block = args.positional(vm, 0).ok_or_else(|| RuntimeError::Arity {
        signature: "ifTrue",
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
        ControlPhase::BoolBranch {
            kind: BoolBranchKind::IfTrueOption,
            branch: block,
        },
    );

    let layout = InvocationLayout::ordinary(0, Box::new([]));
    let view = ArgumentView::from_layout(receiver_idx, layout, caller_auth.0, caller_auth.1);
    vm.activate_function(block, view, source_range)
}

pub fn bool_if_true(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if expect_bool(receiver)? {
        let result = block_call(vm, &args[0], &[])?;
        Ok(wrap_some(vm, result)?)
    } else {
        Ok(vm.none_value())
    }
}

/// Signature: `Bool::ifFalse(_)` — sacred one-armed conditional, mirror of
#[phalcom_native_macros::primitive(
    Bool,
    "ifFalse(_)",
    params = [Object],
    returns = Option,
    types = "(Object) -> Option",
    abi = shape
)]
pub fn bool_if_false_shape(vm: &mut VM, receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    if expect_bool(&receiver)? {
        return Ok(CallOutcome::Returned(vm.none_value()));
    }
    let block = args.positional(vm, 0).ok_or_else(|| RuntimeError::Arity {
        signature: "ifFalse",
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
        ControlPhase::BoolBranch {
            kind: BoolBranchKind::IfFalseOption,
            branch: block,
        },
    );

    let layout = InvocationLayout::ordinary(0, Box::new([]));
    let view = ArgumentView::from_layout(receiver_idx, layout, caller_auth.0, caller_auth.1);
    vm.activate_function(block, view, source_range)
}

pub fn bool_if_false(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if expect_bool(receiver)? {
        Ok(vm.none_value())
    } else {
        let result = block_call(vm, &args[0], &[])?;
        Ok(wrap_some(vm, result)?)
    }
}

/// Signature: `Bool::ifTrue(_)ifFalse(_)` — sacred paired conditional
#[phalcom_native_macros::primitive(
    Bool,
    "ifTrue(_,ifFalse)",
    params = [Object, ifFalse: Object],
    returns = Object,
    types = "(Object, ifFalse: Object) -> Object",
    abi = shape
)]
pub fn bool_if_true_if_false_shape(vm: &mut VM, receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    let branch = if expect_bool(&receiver)? {
        args.positional(vm, 0).ok_or_else(|| RuntimeError::Arity {
            signature: "ifTrue:ifFalse:",
            expected: 2,
            found: args.positional_count(),
        })?
    } else {
        args.positional(vm, 1).ok_or_else(|| RuntimeError::Arity {
            signature: "ifTrue:ifFalse:",
            expected: 2,
            found: args.positional_count(),
        })?
    };

    let receiver_idx = args.receiver_index();
    vm.stack.truncate(receiver_idx);
    vm.stack.push(branch);
    let layout = InvocationLayout::ordinary(0, Box::new([]));
    let view = ArgumentView::from_layout(receiver_idx, layout, vm.current_access_class(), vm.current_has_internal_privilege());
    vm.activate_function(branch, view, SourceRange::default())
}

pub fn bool_if_true_if_false(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let branch = if expect_bool(receiver)? { &args[0] } else { &args[1] };
    block_call(vm, branch, &[])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::{BufferedOutput, VM};

    #[test]
    fn bool_constructor_does_not_write_debug_output() {
        let sink = BufferedOutput::new();
        let handle = sink.handle();
        let mut vm = VM::new_native_with_output(Box::new(sink));
        let receiver = Value::obj(vm.universe.classes.bool_class);

        assert_eq!(bool_class_new(&mut vm, &receiver, &[Value::int(1)]).expect("bool coercion succeeds"), TRUE);
        assert!(handle.bytes().is_empty());
    }

    #[test]
    fn bool_zero_argument_constructor_returns_catchable_error() {
        let mut vm = VM::new_native();
        let receiver = Value::obj(vm.universe.classes.bool_class);

        let error = bool_class_new(&mut vm, &receiver, &[]).expect_err("Bool.new() primitive must fail cleanly");
        assert!(matches!(error, crate::error::PhError::Runtime(RuntimeError::AbstractClass { class: "Bool" })));

        let error = bool_class_new_default(&mut vm, &receiver, &[]).expect_err("Bool.new() must fail cleanly");
        assert!(matches!(error, crate::error::PhError::Runtime(RuntimeError::AbstractClass { class: "Bool" })));
    }
}
