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

/// Signature: `Object::toString` — the default display string (U-CORE-4,
/// [ADR-0015](../../../docs/adr/0015-object-default-tostring.md)).
///
/// A **class** receiver renders as its own name (`"Number"`), fixing
/// DEFERRED F4 (the old binding, [`object_name`], returned the *metaclass*
/// name for a class receiver — see `universe.rs`'s `install_primitives`
/// Object block, where this fn replaces `object_name` as the `toString`
/// target while `object_name` itself stays bound to `Object#name`). A plain
/// instance renders as `"<{ClassName}>"` (e.g. `"<Point>"`). User classes are
/// free to override `toString` for a richer form; this default only
/// guarantees the class is identifiable.
pub fn object_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    // Borrow-model care: bind the cloned name to its own `let` so the
    // immutable `vm.heap` borrow is released before the `&mut vm` alloc below
    // (the `object_name` idiom, this file, L25-26). Do NOT inline the clone
    // into the alloc call.
    let own_name = match receiver {
        Value::Obj(id) => vm.heap.as_class(*id).map(|c| c.name.clone()),
        _ => None,
    };
    if let Some(name) = own_name {
        return Ok(vm.alloc_string_value(name)); // class receiver -> own name (fixes F4)
    }
    let class_id = receiver.class(vm);
    let name = vm.heap.class(class_id).name.clone();
    Ok(vm.alloc_string_value(format!("<{name}>")))
}

/// Signature: `Object::hash` — a stable identity digest of the heap handle.
///
/// The universal-protocol `hash` ([`object-model.md`](../../../docs/spec/object-model.md)
/// §8, [ADR-0023](../../../docs/adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)):
/// underivable because it reads the receiver's [`ObjRef`](crate::heap::ObjRef)
/// handle, which no `.ph`-visible primitive exposes. Immediates
/// ([`Value::Number`], [`Value::Bool`], [`Value::Symbol`]) override this with a
/// value digest; every heap object inherits this identity digest, so
/// `a == b ⇒ a.hash == b.hash` holds for identity-`==` classes (R-INV-1.3). The
/// non-[`Value::Obj`] arm is a defensive catch-all (kept total so a future
/// [`Value`] arm — e.g. `Fiber`, forward-compat §1 — does not silently break
/// this fn), routing through a single `Value`-level hash.
pub fn object_hash(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    use slotmap::Key;
    let bits = match receiver {
        Value::Obj(id) => id.data().as_ffi(),
        // Defensive: every immediate overrides `hash`, so this arm is
        // effectively unreachable. The inner `_` catch-all keeps it total
        // *without* a closed `Value` match — a future `Value` arm (e.g.
        // `Fiber`, forward-compat §1) still compiles here and simply inherits
        // this identity digest until it installs its own override.
        Value::Bool(b) => u64::from(*b),
        Value::Number(n) => n.to_bits(),
        Value::Symbol(s) => u64::from(s.0),
        _ => 0,
    };
    Ok(crate::primitive::hash_code(bits))
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

/// Signature: `Object::methodFor(_)` — reifies the
/// [`MethodObject`](crate::method::MethodObject) that method lookup resolves
/// for selector `args[0]` (a [`Symbol`](crate::interner::Symbol)) on the
/// receiver, as a bare `Method` value; the shared `None` singleton
/// ([ADR-0007](../../../docs/adr/0007-option-some-none.md)) on a miss
/// (functions.md §3, U-CORE-3,
/// [ADR-0028](../../../docs/adr/0028-amend-floor-admit-method-reflection.md)).
///
/// A **pure** probe, like [`object_responds_to`]: a miss never triggers
/// `doesNotUnderstand(_:)`.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `args[0]` is not a `Symbol`.
pub fn object_method_for(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let selector = *expect_value!(&args[0], Symbol);
    match receiver.lookup_method(vm, selector) {
        Some(method_id) => Ok(Value::Obj(method_id)),
        None => Ok(vm.none_value()),
    }
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
