//! Absence primitives: immediate `Some` construction and `Option.match`.
//!
//! U6 replaces surface `nil` with the `Option` type. `Some` and `None` are
//! immediate primitive variants; this module is the Rust bootstrap seam for
//! construction and the one native eliminator.

use crate::error::{PhResult, RuntimeError};
use crate::primitive::block::block_call;
use crate::value::{OptionCase, Value};
use crate::vm::VM;

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
    effects = pure
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
    effects = pure
)]
pub fn some_new(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
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
    types = "(some: Object, none: Object) -> Object"
)]
pub fn option_match(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    match receiver.option_case() {
        OptionCase::Some(value) => block_call(vm, &args[0], &[value]),
        OptionCase::None => block_call(vm, &args[1], &[]),
        OptionCase::NotOption => Err(type_error(receiver)),
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
