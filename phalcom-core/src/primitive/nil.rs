//! Absence primitives: `Some` construction and the `Option` `match` eliminator.
//!
//! U6 replaces surface `nil` with the `Option` type
//! ([ADR-0007](../../../docs/adr/0007-option-some-none.md)): `Option` is
//! abstract, `Some` wraps one value in its `_value` field, and `None` is a
//! single shared identity-comparable singleton. This module is the *substrate*
//! bootstrapped in Rust so U6 does not depend on U7's user-facing `construct`:
//!
//! - [`some_new`] — the `Some(_)` construction primitive.
//! - [`option_match`] — the `match(some:none:)` eliminator, the one primitive
//!   that leaves Option-world with a value (`values-and-absence.md` §3.2); every
//!   other extractor (`map`/`orElse`/… — U-STD's job) is defined over it.
//!
//! There is deliberately **no** surface `Nil` class or `nil` constructor here:
//! the private [`Value::Nil`] sentinel
//! ([ADR-0010](../../../docs/adr/0010-tagged-value-enum.md)) is surfaced to
//! `None` at read boundaries by
//! [`sentinel_to_option`](crate::value::sentinel_to_option) and can never be
//! produced by user code (Invariant 4).

use crate::error::{PhResult, RuntimeError};
use crate::heap::Object;
use crate::instance::InstanceObject;
use crate::primitive::block::block_call;
use crate::value::Value;
use crate::vm::VM;

/// Wraps `value` in a fresh `Some` instance — the shared core of `Some(_)`
/// construction ([ADR-0007](../../../docs/adr/0007-option-some-none.md)).
/// Allocates a fresh [`InstanceObject`] of `Some` and stores `value` in its
/// `_value` field.
///
/// Used both by [`some_new`] (the surface `Some(_)` primitive) and by the
/// sacred-selector inliner's [`crate::bytecode::Bytecode::WrapSome`] opcode,
/// which Some-lifts the taken arm of an inlined one-armed `ifTrue`/`ifFalse`
/// fast path to keep it observationally identical to the
/// `bool_if_true`/`bool_if_false` primitive fallback
/// ([ADR-0018](../../../docs/adr/0018-sacred-selector-inliner-and-override-guard.md)
/// amendment, U-CORE-2).
///
/// # Panics
///
/// Panics if `value` is the private [`Value::Nil`] sentinel: `None` is never
/// inside a `Some` (`values-and-absence.md` Invariant 4). User code cannot
/// produce the sentinel, so this only ever fires on an internal surfacing
/// bug.
pub(crate) fn wrap_some(vm: &mut VM, value: Value) -> Value {
    assert!(
        !matches!(value, Value::Nil),
        "Invariant 4: the private nil sentinel must never be wrapped in a Some"
    );
    let some_cls = vm.universe.classes.some_class;
    let field_count = vm.heap.class(some_cls).field_count;
    let mut instance = InstanceObject::new(some_cls, field_count);
    instance.slots[0] = value;
    let id = vm.heap.alloc(Object::Instance(instance));
    Value::Obj(id)
}

/// Constructs a `Some` wrapping `args[0]` — the `Some(_)` primitive.
///
/// Registered as the static `new(_)` on the kernel `Some` class
/// ([ADR-0007](../../../docs/adr/0007-option-some-none.md)); the surface
/// `Some(x)` desugar (compiler's job) lowers to this send. See `wrap_some`
/// (private to this module) for the allocation and the Invariant-4 panic
/// condition.
///
/// # Errors
///
/// Infallible in practice; returns [`PhResult`] to match the primitive ABI.
pub fn some_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(wrap_some(vm, args[0]))
}

/// Eliminates an `Option`: `receiver.match(some: onSome, none: onNone)`.
///
/// The sole primitive that leaves Option-world with a value
/// (`values-and-absence.md` §3.2,
/// [ADR-0007](../../../docs/adr/0007-option-some-none.md)). Registered on the
/// abstract `Option` class so both `Some` and `None` inherit it. If the
/// receiver is a `Some`, calls the `some:` block with the wrapped value; if it
/// is the `None` singleton, calls the `none:` block with no arguments.
///
/// `Option` never implements the boolean-branch protocol
/// ([`Bytecode::JumpIfFalse`](crate::bytecode::Bytecode::JumpIfFalse) requires a
/// real `Bool`), so this eliminator — not truthiness — is the only way to
/// branch on absence (BD-U6-1 Option A floor).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is neither a `Some` nor the
/// `None` singleton, or propagates any error raised by the invoked block.
pub fn option_match(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let some_cls = vm.universe.classes.some_class;
    let none_cls = vm.universe.classes.none_class;

    let class = match receiver {
        Value::Obj(id) => match vm.heap.as_instance(*id) {
            Some(instance) => instance.class,
            None => return Err(type_error(receiver)),
        },
        _ => return Err(type_error(receiver)),
    };

    if class == some_cls {
        let Value::Obj(id) = receiver else { unreachable!("checked above") };
        let wrapped = vm
            .heap
            .instance(*id)
            .slots[0];
        block_call(vm, &args[0], &[wrapped])
    } else if class == none_cls {
        block_call(vm, &args[1], &[])
    } else {
        Err(type_error(receiver))
    }
}

/// Builds the "not an Option" [`RuntimeError::Type`] for [`option_match`].
fn type_error(receiver: &Value) -> crate::error::PhError {
    RuntimeError::Type { expected: "Option", found: receiver.type_name() }.into()
}
