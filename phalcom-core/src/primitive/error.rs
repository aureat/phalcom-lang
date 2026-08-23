//! Native primitives on `Error` — the raisable root of the surface error
//! hierarchy (object-model.md §4 "Errors",
//! [ADR-0008](../../../docs/adr/accepted/0008-layered-exceptions-and-result.md)).
//!
//! `raise` is the surface half of the unified unwind (the `Raise` sibling of
//! U10's `Return`/[`Bytecode::ReturnNonLocal`](crate::bytecode::Bytecode::ReturnNonLocal));
//! `message` reads the error's `_message` slot, stamped in [`VM::new`]'s
//! Phase E exactly like [`crate::universe::CoreClasses::message_class`]. Both
//! are ADR-0019 floor additions (+2 bindings), pre-cleared in principle by
//! [ADR-0023](../../../docs/adr/accepted/0023-amend-floor-admit-hash-and-kernel-reflection.md)
//! and landed by this unit's own amendment
//! ([ADR-0037](../../../docs/adr/accepted/0037-amend-floor-admit-error-root.md)).
//!
//! [`VM::new`]: crate::vm::VM::new

use crate::error::{PhResult, RuntimeError};
use crate::value::Value;
use crate::vm::VM;

/// Signature: `Error::message` — the error's message string (slot 0),
/// surfaced as `None` if unset (though every reified
/// [`MessageNotUnderstood`](crate::universe::CoreClasses::message_not_understood_class)
/// sets it at construction time). Native slot accessor, mirroring
/// [`crate::primitive::object::message_selector`] — avoids the
/// read-before-write hazard a `.ph` getter over this field would trip
/// ([U-CORE-6 §2](../../../docs/forge/units/U-CORE-6/as-built.md)).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not an `Error` instance.
#[phalcom_native_macros::primitive(Error, "message")]
pub fn error_message(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let slot0 = receiver
        .as_obj()
        .and_then(|id| vm.heap.as_instance(id).map(|inst| inst.slots[0]))
        .ok_or_else(|| RuntimeError::Type {
            expected: "Error",
            found: receiver.type_name(),
        })?;
    Ok(vm.surface_absence(slot0))
}

/// Signature: `Error::raise()` — unwinds the stack with `self` as a surface
/// `Error` ([ADR-0008](../../../docs/adr/accepted/0008-layered-exceptions-and-result.md);
/// `throw expr` desugars to this send per
/// [ADR-0031](../../../docs/adr/accepted/0031-error-handling-surface-syntax.md) §1).
/// Only `Error` and its subclasses respond to `raise` — the primitive is
/// installed on `Error` only, so a non-`Error` receiver misses (dNU →
/// `MessageNotUnderstood`), realizing the runtime half of R-INV-6.3.
///
/// Renders `rendered` via the receiver's own `message` send (rather than
/// reading slot 0 directly) so a future computed override on a user `Error`
/// subclass is honored; re-entering the VM here is safe (the stack is healthy
/// — this runs inside a live primitive, before any unwind), the same pattern
/// [`object_perform_shape`](crate::primitive::object::object_perform_shape) uses.
///
/// # Errors
///
/// Always returns [`RuntimeError::Raise`] carrying `self` as the raised
/// `error` value — the `Raise` payload of the unified unwind. A future
/// `on(_)`/`ensure` (ADR-0031) or a fiber's result-slot capture intercepts it;
/// uncaught, it renders and exits exactly like the retired native
/// `MessageNotUnderstood` did.
#[phalcom_native_macros::primitive(Error, "raise()")]
pub fn error_raise(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let message_sym = vm.get_or_intern("message");
    let rendered = vm.send_dynamic(*receiver, message_sym, &[])?.to_string(vm);
    Err(RuntimeError::Raise {
        error: *receiver,
        rendered,
        traceback: None,
        help: None,
    }
    .into())
}
