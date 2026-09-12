//! Absence primitives: immediate `Some` construction and `Option.match`.
//!
//! U6 replaces surface `nil` with the `Option` type. `Some` and `None` are
//! immediate primitive variants; this module is the Rust bootstrap seam for
//! construction and the one native eliminator.

use crate::error::{PhResult, RuntimeError};
use crate::method::{ArgumentView, CallOutcome, InvocationLayout};
use crate::value::{OptionCase, Value};
use crate::vm::VM;
use phalcom_common::range::SourceRange;

/// Adds one immediate `Some` layer without allocating an Option wrapper.
///
/// The private `Value::Nil` sentinel is rejected by the representation helper
/// so it cannot enter a surface `Some` value.
pub(crate) fn wrap_some(_vm: &mut VM, value: Value) -> Result<Value, RuntimeError> {
    value.wrap_some()
}

/// Constructs a `Some` wrapping `args[0]` — the canonical `Some(_)` primitive.
///
/// Registered as `call(_)` on the `Some` class object. Existing unqualified-call
/// lowering makes `Some(x)` an ordinary `Some.call(x)` send.
#[phalcom_native_macros::primitive(
    Some,
    "call(_)",
    params = [Object],
    returns = Option,
    types = "(Object) -> Option",
    side = class,
    effects = pure,
    anchor = hidden
)]
pub fn some_call(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(wrap_some(_vm, args[0])?)
}

/// Compatibility alias for the historical `Some.new(_)` construction surface.
#[phalcom_native_macros::primitive(
    Some,
    "new(_)",
    params = [Object],
    returns = Option,
    types = "(Object) -> Option",
    side = class,
    effects = pure,
    anchor = hidden
)]
pub fn some_new(_vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(wrap_some(_vm, args[0])?)
}

/// Eliminates an `Option`: `receiver.match(some: onSome, none: onNone)`.
///
/// A `Some` peels exactly one layer before invoking the `some:` block. Immediate
/// `None` invokes `none:` with no arguments. The primitive never inspects class
/// IDs or heap slots, so nested values remain distinct.
#[phalcom_native_macros::primitive(
    Option,
    "match(some,none)",
    params = [some: Object, none: Object],
    returns = Object,
    types = "(some: Object, none: Object) -> Object",
    abi = shape
)]
pub fn option_match_shape(vm: &mut VM, receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    if args.physical_arity() < 2 {
        return Err(RuntimeError::Arity {
            signature: "match",
            expected: 2,
            found: args.physical_arity(),
        }
        .into());
    }
    let some_branch = vm.stack[args.receiver_index() + 1];
    let none_branch = vm.stack[args.receiver_index() + 2];

    match receiver.option_case() {
        OptionCase::Some(value) => {
            let receiver_idx = args.receiver_index();
            let source_range = SourceRange::default();
            vm.stack[receiver_idx] = some_branch;
            vm.stack[receiver_idx + 1] = value;
            vm.stack.truncate(receiver_idx + 2);
            let layout = InvocationLayout::ordinary(1, Box::new([]));
            let view = ArgumentView::from_layout(receiver_idx, layout, args.caller_authority().0, args.caller_authority().1);
            vm.activate_function(some_branch, view, source_range)
        }
        OptionCase::None => {
            let receiver_idx = args.receiver_index();
            let source_range = SourceRange::default();
            vm.stack[receiver_idx] = none_branch;
            vm.stack.truncate(receiver_idx + 1);
            let layout = InvocationLayout::ordinary(0, Box::new([]));
            let view = ArgumentView::from_layout(receiver_idx, layout, args.caller_authority().0, args.caller_authority().1);
            vm.activate_function(none_branch, view, source_range)
        }
        OptionCase::NotOption => Err(type_error(&receiver)),
    }
}

pub fn option_match(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let receiver_idx = vm.stack.len();
    vm.stack.push(*receiver);
    vm.stack.extend_from_slice(args);
    let layout = InvocationLayout::ordinary(args.len(), Box::new([]));
    let view = ArgumentView::from_layout(receiver_idx, layout, vm.current_access_class(), vm.current_has_internal_privilege());
    match option_match_shape(vm, *receiver, view)? {
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

/// Builds the "not an Option" error for [`option_match`].
fn type_error(receiver: &Value) -> crate::error::PhError {
    RuntimeError::Type {
        expected: "Option",
        found: receiver.type_name(),
    }
    .into()
}
