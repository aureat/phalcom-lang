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
use crate::heap::InstanceObject;
use crate::heap::Object;
use crate::primitive::expect_list;
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
/// [ADR-0015](../../../docs/adr/accepted/0015-object-default-tostring.md)).
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
/// §8, [ADR-0023](../../../docs/adr/accepted/0023-amend-floor-admit-hash-and-kernel-reflection.md)):
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
        Value::Int(n) => *n as u64,
        Value::Float(n) => n.to_bits(),
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
/// packed in the [`List`](crate::heap::ListObject) `args[1]`
/// (messages-and-selectors.md §5).
///
/// Because the selector is a *complete* selector symbol (built via
/// `Symbol.new("+(_)")` until the `#`-literal lexer syntax lands in U-LEX),
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
    let responds = receiver
        .lookup_method(vm, selector)
        .is_some_and(|method| vm.authorize_method_access(method).is_ok());
    Ok(Value::Bool(responds))
}

/// Signature: `Object::methodFor(_)` — reifies the
/// [`MethodObject`](crate::method::MethodObject) that method lookup resolves
/// for selector `args[0]` (a [`Symbol`](crate::interner::Symbol)) on the
/// receiver, as a bare `Method` value; the shared `None` singleton
/// ([ADR-0007](../../../docs/adr/accepted/0007-option-some-none.md)) on a miss
/// (functions.md §3, U-CORE-3,
/// [ADR-0028](../../../docs/adr/accepted/0028-amend-floor-admit-method-reflection.md)).
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
        Some(method_id) if vm.authorize_method_access(method_id).is_ok() => Ok(Value::Obj(method_id)),
        None => Ok(vm.none_value()),
        Some(_) => Ok(vm.none_value()),
    }
}

/// Signature: `Object::doesNotUnderstand(_)` — the *default* miss handler
/// (method-lookup.md §2, ADR-0012): builds a surface
/// [`MessageNotUnderstood`](crate::universe::CoreClasses::message_not_understood_class)
/// carrying the reified `Message` and raises it through the unified unwind
/// ([`RuntimeError::Raise`], ADR-0008, U-CORE-6) — rather than the retired
/// native `RuntimeError::MessageNotUnderstood`.
///
/// This is the terminal fallback the [`Bytecode::Invoke`](crate::bytecode::Bytecode::Invoke)
/// miss path forwards to. Because it is an ordinary (overridable) method on
/// `Object`, a subclass — e.g. a proxy — can override it to intercept and
/// re-forward sends via [`object_perform`] *before* this default ever runs.
/// `args[0]` is the reified `Message` ([`VM::new_message`]); its selector slot
/// supplies the diagnostic text. The built `MessageNotUnderstood` instance has
/// two slots: slot 0 the rendered message string (`Error#message`), slot 1 the
/// reified `Message` itself (`args[0]`), stamped by [`VM::new`]'s Phase E,
/// mirroring how `Message` itself is built directly in Rust rather than via a
/// `.ph` `construct` (U-CORE-6 §2, avoids the read-before-write hazard a
/// `.ph` getter over this field would trip).
///
/// # Errors
///
/// Always returns [`RuntimeError::Raise`] carrying a
/// [`MessageNotUnderstood`](crate::universe::CoreClasses::message_not_understood_class)
/// instance.
pub fn object_does_not_understand(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let selector = match message_slot(vm, &args[0], 0) {
        Some(Value::Symbol(sym)) => vm.resolve_symbol(sym).to_string(),
        _ => "<unknown>".to_string(),
    };
    let mut receiver_name = receiver.to_string(vm);
    if receiver_name.chars().count() > 40 {
        receiver_name = receiver_name.chars().take(40).collect::<String>();
    }
    let rendered = format!("{receiver_name} does not understand '{selector}'");

    // Collect candidates from receiver's class and ancestors
    let mut cand_strings = Vec::new();
    let mut current_class_id = Some(receiver.class(vm));
    while let Some(cls_id) = current_class_id {
        let class_obj = vm.heap.class(cls_id);
        for &sym in class_obj.methods.keys() {
            cand_strings.push(vm.resolve_symbol(sym).to_string());
        }
        current_class_id = class_obj.superclass;
    }
    cand_strings.sort();
    cand_strings.dedup();

    let help = crate::diagnostics::suggest::suggest_selector(&selector, cand_strings.into_iter());

    // Reify the surface MessageNotUnderstood: slot 0 = message string, slot 1
    // = the reified Message (`args[0]`, floor-census §2.14). Built directly in
    // Rust — the `Message` precedent (`VM::new_message`), no `.ph` construct.
    let mnu_class = vm.universe.classes.message_not_understood_class;
    let field_count = vm.heap.class(mnu_class).field_count; // == 2 (Phase E)
    let mut inst = InstanceObject::new(mnu_class, field_count);
    inst.slots[0] = vm.alloc_string_value(rendered.clone());
    inst.slots[1] = args[0]; // the reified Message
    let mnu = Value::Obj(vm.heap.alloc(Object::Instance(inst)));

    // Raise it through the unified unwind (NOT the retired native
    // RuntimeError::MessageNotUnderstood variant).
    Err(RuntimeError::Raise {
        error: mnu,
        rendered,
        traceback: None,
        help,
    }
    .into())
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
    RuntimeError::Type {
        expected: "Message",
        found: value.type_name(),
    }
    .into()
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

/// Signature: `Message::labels` — the [`List`](crate::heap::ListObject) of
/// per-argument keyword labels (slot 2), one `String` per argument (`""` for a
/// positional argument), index-aligned with [`message_args`].
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Message` instance.
pub fn message_labels(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    message_slot(vm, receiver, 2).ok_or_else(|| not_a_message(receiver))
}

/// Signature: `Message::args` — the [`List`](crate::heap::ListObject) of the
/// send's argument values (slot 3).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Message` instance.
pub fn message_args(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    message_slot(vm, receiver, 3).ok_or_else(|| not_a_message(receiver))
}

/// Signature: `Object::__invariantEnter()` — the entry half of the
/// `@invariant` re-entrancy guard
/// ([ADR-0052](../../../docs/adr/accepted/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)
/// Fix 1, U-ANNOT-CONTRACTS). Woven by
/// `crate::compiler::attributes::weave_invariant_checks` into every public
/// method/getter/setter of an `@invariant`-bearing class.
///
/// Inserts the receiver into `VM::checking` and returns `true` **iff** the
/// receiver was not already present — i.e. this call is the *outermost*
/// guarded call on `self` (own-object nesting, Eiffel's rule). The caller
/// binds this return value to a local (`__invariant_owner`) and gates both
/// the entry check and the paired [`object_invariant_exit`] on it, rather than
/// re-checking `checking` membership at exit time — membership alone cannot
/// distinguish the owning call from a nested one once a nested call exists,
/// only a locally-captured boolean can (ADR-0052's own pseudocode does not
/// capture this correctly; this primitive pair is the corrected mechanism).
///
/// A non-heap receiver (an immediate — `Number`/`Bool`/`Symbol`/`None`) has no
/// [`crate::heap::ObjRef`] identity to key `checking` on; `@invariant` is only
/// ever woven onto a user class's own instance methods, so this is not
/// expected to fire on an immediate in practice, but returns `true`
/// unconditionally rather than panicking if it ever does (every call is then
/// treated as its own outermost call, the safe default).
pub fn object_invariant_enter(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let Value::Obj(id) = receiver else {
        return Ok(Value::Bool(true));
    };
    let is_owner = vm.checking.insert(*id);
    Ok(Value::Bool(is_owner))
}

/// Signature: `Object::__invariantExit()` — the exit half of the `@invariant`
/// re-entrancy guard (see [`object_invariant_enter`]).
///
/// Unconditionally removes the receiver from `VM::checking`. Only ever
/// called from the woven `Block#ensure(_)` cleanup, itself gated on the
/// caller's own `__invariant_owner` local — so in practice this only fires
/// once per outermost guarded call, but removal is idempotent regardless.
///
/// Returns `VM::none_value` — **never** a raw `Value::Nil` — matching every
/// other unit-returning native primitive: the one-armed `ifTrue` inliner's
/// Some-wrap expects the surface `None` singleton, not the bare tag.
pub fn object_invariant_exit(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if let Value::Obj(id) = receiver {
        vm.checking.remove(id);
    }
    Ok(vm.none_value())
}
