//! Native primitives for the kernel `Module` class (U15, DEC-U15 A+A).

use crate::error::{PhResult, RuntimeError};
use crate::method::{SignatureKind, decode_selector, encode_selector};
use crate::primitive::object::object_does_not_understand;
use crate::value::Value;
use crate::vm::VM;

/// `Module.class::new()`
///
/// # Errors
///
/// Always returns [`RuntimeError::NotAllowed`] — a `Module` is only ever
/// produced by [`VM::import_module`](crate::vm::VM::import_module), never by
/// a surface `construct`.
pub fn module_class_new(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::NotAllowed("Module instances cannot be created directly".to_string()).into())
}

/// Reads slot `index` of a `Message` instance `value`, or `None` if `value`
/// is not an [`InstanceObject`](crate::heap::InstanceObject). Mirrors
/// [`crate::primitive::object`]'s private helper of the same shape (kept
/// local rather than shared, since it is a one-line, non-generic accessor).
fn message_slot(vm: &VM, value: &Value, index: usize) -> Option<Value> {
    match value {
        Value::Obj(id) => vm.heap.as_instance(*id).map(|instance| instance.slots[index]),
        _ => None,
    }
}

/// Extracts a reified `Message`'s `args` (slot 3) as an owned `Vec<Value>`.
fn message_args(vm: &VM, message: &Value) -> Vec<Value> {
    match message_slot(vm, message, 3) {
        Some(Value::Obj(id)) => vm.heap.as_list(id).map(|list| list.elements().to_vec()).unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// `Module#doesNotUnderstand(_:)` — member access is an ordinary send
/// (object-model.md §4, U15 DEC-U15): a message the kernel `Module` class
/// itself does not define is checked against the module's own global table
/// (its member table, [`ModuleObject::get`](crate::heap::ModuleObject::get))
/// before falling through to the [`Object`](crate::universe::CoreClasses::object_class)
/// default's `MessageNotUnderstood` raise.
///
/// A zero-arg getter selector (`math.pi`) returns the bound value directly.
/// Any other selector shape (`math.distance(1, 2)`) whose bound value is
/// itself callable is forwarded to it via the matching-arity `call(...)`
/// selector — "the `distance` member, called with two arguments" falls out
/// of "everything is a message" rather than needing a bespoke static-method
/// dispatch path; a non-callable member sent with arguments simply forwards
/// into that value's own (likely also-missing) `call` and reports *its*
/// `doesNotUnderstand`, which is the expected behavior for "not a function".
///
/// # Errors
///
/// Propagates [`object_does_not_understand`]'s [`RuntimeError::Raise`] when
/// `args[0]` is not a `Message`, the receiver is not a `Module`, or no
/// member matches the selector's bare name; propagates any error raised by
/// a forwarded `call(...)` send.
pub fn module_does_not_understand(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let message = args[0];
    let selector_sym = match message_slot(vm, &message, 0) {
        Some(Value::Symbol(sym)) => sym,
        _ => return object_does_not_understand(vm, receiver, args),
    };
    let module_id = match receiver {
        Value::Obj(id) => *id,
        _ => return object_does_not_understand(vm, receiver, args),
    };

    let selector_str = vm.resolve_symbol(selector_sym).to_string();
    let (name, _labels, _kind) = decode_selector(&selector_str);
    let name_sym = vm.interner.intern(&name);

    let Some(value) = vm.heap.module(module_id).get(name_sym) else {
        return object_does_not_understand(vm, receiver, args);
    };

    let call_args = message_args(vm, &message);
    if call_args.is_empty() {
        return Ok(value);
    }

    let call_selector = encode_selector("call", &vec![None; call_args.len()], SignatureKind::Method(call_args.len() as u8));
    let call_sym = vm.get_or_intern(&call_selector);
    vm.send_dynamic(value, call_sym, &call_args)
}
