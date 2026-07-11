//! Native primitives on `Object` — the tower root — plus the reflective-send
//! surface and the `Message` accessors.
//!
//! The reflective-dispatch primitives ([`object_perform`],
//! [`object_perform_with`], [`object_responds_to`],
//! [`object_does_not_understand`]) realize messages-and-selectors.md §5 and
//! method-lookup.md §2 over the shared [`VM::send_dynamic`] workhorse
//! (ADR-0012). A missed send is reified as a `Message` instance whose slots
//! are read back through the [`message_selector`]/[`message_name`]/
//! [`message_labels`]/[`message_args`] accessors — see [`VM::new_message`] for
//! the slot layout.

use crate::error::PhResult;
use crate::error::RuntimeError;
use crate::expect_value;
use crate::heap::Object;
use crate::instance::InstanceObject;
use crate::primitive::{expect_class, expect_list};
use crate::value::Value;
use crate::vm::VM;

/// Signature: `Object::name` — returns the receiver's class name as a string.
pub fn object_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = receiver.class(vm);
    let name = vm.heap.class(class_id).name.clone();
    Ok(vm.alloc_string_value(name))
}

/// Signature: `Object::class` — returns the receiver's class.
pub fn object_class(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::Obj(receiver.class(vm)))
}

/// Signature: `Object::class=(_)` — always an error; an object's class is fixed.
///
/// # Errors
///
/// Always returns [`RuntimeError::InvalidSetClass`].
pub fn object_set_class(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::InvalidSetClass.into())
}

/// Signature: `Object.class::new` — allocates a bare instance of the receiver class.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a class.
pub fn object_class_new(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = expect_class(vm, receiver)?;
    let field_count = vm.heap.class(class_id).field_count;
    let instance = InstanceObject::new(class_id, field_count);
    Ok(Value::Obj(vm.heap.alloc(Object::Instance(instance))))
}

/// Signature: `Object::==(_)` — the base equality send (U5, control-flow.md
/// §1: `==`/`!=` are ordinary sends like every other operator). Delegates to
/// [`Value::value_eq`](crate::value::Value::value_eq) (content equality for
/// strings, identity for instances/classes/methods, by-value for
/// immediates), so it reproduces exactly today's `==` semantics — only the
/// *dispatch mechanism* changes. Any subclass (e.g. a user `==(other)`
/// override, per `person2.ph`) shadows this via ordinary method lookup.
pub fn object_eq(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::Bool(receiver.value_eq(&args[0], &vm.heap)))
}

/// Signature: `Object::!=(_)` — the base inequality send; the logical
/// negation of [`object_eq`].
pub fn object_neq(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::Bool(!receiver.value_eq(&args[0], &vm.heap)))
}

/// Signature: `Object::perform(_)` — reflectively send `args[0]` (a selector
/// [`Symbol`](crate::interner::Symbol)) to the receiver with no arguments
/// (messages-and-selectors.md §5).
///
/// The zero-argument case of [`object_perform_with`]; a thin wrapper over
/// [`VM::send_dynamic`], so it dispatches through the exact same lookup + dNU
/// path as a static send (reflective parity). A miss re-enters
/// `doesNotUnderstand(_:)` exactly once.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `args[0]` is not a `Symbol`, or propagates
/// any error raised by the dispatched method.
pub fn object_perform(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let selector = expect_value!(&args[0], Symbol);
    vm.send_dynamic(*receiver, *selector, &[])
}

/// Signature: `Object::perform(_,_)` — reflectively send selector `args[0]`
/// (a [`Symbol`](crate::interner::Symbol)) to the receiver with the arguments
/// packed in the [`List`](crate::list::ListObject) `args[1]`
/// (messages-and-selectors.md §5).
///
/// Because the selector is a *complete* selector symbol (built via
/// `Symbol.new("+(_:)")` until the `#`-literal lexer syntax lands in U-LEX),
/// its encoded arity must match the number of elements in the list; a mismatch
/// surfaces through ordinary lookup/arity checking. Delegates to
/// [`VM::send_dynamic`].
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `args[0]` is not a `Symbol` or `args[1]`
/// is not a `List`, or propagates any error raised by the dispatched method.
pub fn object_perform_with(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let selector = *expect_value!(&args[0], Symbol);
    let list_id = expect_list(vm, &args[1])?;
    let elements: Vec<Value> = vm.heap.list(list_id).elements().to_vec();
    vm.send_dynamic(*receiver, selector, &elements)
}

/// Signature: `Object::respondsTo(_)` — returns whether the receiver's class
/// chain defines the selector `args[0]` (a [`Symbol`](crate::interner::Symbol)).
///
/// A **pure** exact-selector probe (method-lookup.md §2): it never triggers
/// `doesNotUnderstand(_:)`, so asking whether an object responds to an unknown
/// selector simply returns `false` rather than reifying a `Message`.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `args[0]` is not a `Symbol`.
pub fn object_responds_to(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let selector = *expect_value!(&args[0], Symbol);
    Ok(Value::Bool(receiver.lookup_method(vm, selector).is_some()))
}

/// Signature: `Object::doesNotUnderstand(_)` — the *default* miss handler
/// (method-lookup.md §2, ADR-0012): raises
/// [`RuntimeError::MessageNotUnderstood`] describing the missed send.
///
/// This is the terminal fallback the [`Bytecode::Invoke`](crate::bytecode::Bytecode::Invoke)
/// miss path forwards to. Because it is an ordinary (overridable) method on
/// `Object`, a subclass — e.g. a proxy — can override it to intercept and
/// re-forward sends via [`object_perform`]. `args[0]` is the reified `Message`
/// ([`VM::new_message`]); its selector slot supplies the diagnostic text.
///
/// # Errors
///
/// Always returns [`RuntimeError::MessageNotUnderstood`].
pub fn object_does_not_understand(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let selector = match message_slot(vm, &args[0], 0) {
        Some(Value::Symbol(sym)) => vm.resolve_symbol(sym).to_string(),
        _ => "<unknown>".to_string(),
    };
    let receiver_name = receiver.to_string(vm);
    Err(RuntimeError::MessageNotUnderstood { selector, receiver: receiver_name }.into())
}

/// Reads slot `index` of a `Message` instance `value`, or `None` if `value` is
/// not an [`InstanceObject`].
fn message_slot(vm: &VM, value: &Value, index: usize) -> Option<Value> {
    match value {
        Value::Obj(id) => vm.heap.as_instance(*id).map(|instance| instance.slots[index]),
        _ => None,
    }
}

/// Builds the "not a Message" [`RuntimeError::Type`] for the accessors.
fn not_a_message(value: &Value) -> crate::error::PhError {
    RuntimeError::Type { expected: "Message", found: value.type_name() }.into()
}

/// Signature: `Message::selector` — the interned selector
/// [`Symbol`](crate::interner::Symbol) exactly as sent (slot 0; see
/// [`VM::new_message`]).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Message` instance.
pub fn message_selector(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    message_slot(vm, receiver, 0).ok_or_else(|| not_a_message(receiver))
}

/// Signature: `Message::name` — the bare method-name [`String`] (slot 1),
/// e.g. `"+"` for a `+(_:)` send (encoder-inverse, see [`VM::new_message`]).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Message` instance.
pub fn message_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    message_slot(vm, receiver, 1).ok_or_else(|| not_a_message(receiver))
}

/// Signature: `Message::labels` — the [`List`](crate::list::ListObject) of
/// per-argument keyword labels (slot 2), one `String` per argument (`""` for a
/// positional argument), index-aligned with [`message_args`].
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Message` instance.
pub fn message_labels(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    message_slot(vm, receiver, 2).ok_or_else(|| not_a_message(receiver))
}

/// Signature: `Message::args` — the [`List`](crate::list::ListObject) of the
/// send's argument values (slot 3).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Message` instance.
pub fn message_args(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    message_slot(vm, receiver, 3).ok_or_else(|| not_a_message(receiver))
}
