//! Native primitives for the kernel `Family` class (`::` method references,
//! selectors.md §3, U16-Open, U16-Pinned,
//! [ADR-0047](../../../../docs/adr/0047-amend-floor-admit-family-call-router.md)).
//!
//! `Family` defines exactly one native primitive — [`family_does_not_understand`]
//! — and no other methods: every bare-call selector shape a call site can
//! produce (`call()`, `call(_:)`, `call(to:duration:)`, …) therefore misses
//! `Family`'s own method table by construction and lands here, which is what
//! makes it the uniform call router (selectors.md §3 "A family call *is* a
//! send"). There is no second dispatch mechanism and no reflective surface on
//! `Family` in this unit (Q14 ruling, `open-questions.md`).

use crate::error::{PhResult, RuntimeError};
use crate::heap::FamilyObject;
use crate::method::{decode_selector, encode_selector};
use crate::primitive::object::object_does_not_understand;
use crate::value::Value;
use crate::vm::VM;

/// Reads slot `index` of a `Message` instance `value`, or `None` if `value`
/// is not an [`InstanceObject`](crate::instance::InstanceObject). Mirrors
/// [`crate::primitive::module`]'s private helper of the same shape (kept
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

/// `Family#doesNotUnderstand(_:)` — the Open/Pinned-family **call router**
/// (selectors.md §3, U16-Open, U16-Pinned).
///
/// A call site on a `Family` value (`f(to: p, duration: 2)`) always compiles
/// to a `call(...)` send whose argument labels are exactly the call site's
/// (`phalcom-ast::parser::Parser::parse_call`'s bare-`(args)` postfix). Since
/// `Family` never defines a `call(...)` method, that send always misses and
/// reaches this handler, which branches on [`FamilyObject::open`]:
///
/// - **Open**: decodes the missed selector back into its `(labels, kind)`
///   shape ([`decode_selector`] — the exact inverse of the encoder that
///   built it), rebuilds the *real* target selector from the family's own
///   base name ([`FamilyObject::selector`]) plus those same labels
///   ([`encode_selector`] — selectors.md §3 "the selector is built from
///   `family.name` + the call's label suffix"), then re-dispatches.
/// - **Pinned**: ignores the call site's own labels entirely — only its
///   *argument count* is checked against the pinned selector's arity (a
///   mismatch is a hard error, not a silent wrong dispatch) — then
///   re-dispatches [`FamilyObject::selector`] verbatim, unchanged from
///   reference time.
///
/// Either way the re-dispatch is an ordinary send to the family's bound
/// [`recv`](FamilyObject::recv) via [`VM::send_dynamic`] — not a second
/// dispatch mechanism. An Open target-selector miss (the labels supplied at
/// the call site match no member of the family) falls straight through
/// `send_dynamic`'s own miss path to `recv`'s `doesNotUnderstand(_:)` — the
/// ordinary one, per selectors.md §3's error table.
///
/// # Errors
///
/// Falls back to [`object_does_not_understand`] (surfacing the ordinary
/// `MessageNotUnderstood`) if `args[0]` is not a reified `Message` or
/// `receiver` is not a `Family`. For a Pinned family, raises
/// [`RuntimeError::Message`] if the call site's argument count does not
/// match the pinned selector's declared arity. Otherwise propagates any
/// error raised by the re-dispatched send.
pub fn family_does_not_understand(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let message = args[0];
    let Some(Value::Symbol(missed_selector)) = message_slot(vm, &message, 0) else {
        return object_does_not_understand(vm, receiver, args);
    };
    let family: FamilyObject = match receiver {
        Value::Obj(id) => match vm.heap.as_family(*id) {
            Some(family) => *family,
            None => return object_does_not_understand(vm, receiver, args),
        },
        _ => return object_does_not_understand(vm, receiver, args),
    };

    let call_args = message_args(vm, &message);

    if family.open {
        let missed_str = vm.resolve_symbol(missed_selector).to_string();
        let (_missed_name, labels, kind) = decode_selector(&missed_str);
        let base_name = vm.resolve_symbol(family.selector).to_string();
        let target_selector = encode_selector(&base_name, &labels, kind);
        let target_sym = vm.get_or_intern(&target_selector);
        vm.send_dynamic(family.recv, target_sym, &call_args)
    } else {
        let pinned_str = vm.resolve_symbol(family.selector).to_string();
        let (pinned_name, pinned_labels, _kind) = decode_selector(&pinned_str);
        if call_args.len() != pinned_labels.len() {
            return Err(RuntimeError::Message(format!(
                "pinned method reference `{pinned_name}` expects {} argument(s), got {}",
                pinned_labels.len(),
                call_args.len()
            ))
            .into());
        }
        vm.send_dynamic(family.recv, family.selector, &call_args)
    }
}
