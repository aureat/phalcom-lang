//! Native primitives on `Object` — the tower root — plus the reflective-send
//! surface and the `Message` accessors.
//!
//! The reflective-dispatch primitives ([`object_perform_shape`],
//! [`object_responds_to`],
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
use crate::method::{ArgumentView, CallOutcome, SignatureKind, decode_selector};
use crate::value::Value;
use crate::vm::VM;

/// Signature: `Object::name` — returns the receiver's class name as a string.
#[phalcom_native_macros::primitive(
    Object,
    "name",
    params = [],
    returns = String,
    types = "() -> String",
    effects = pure
)]
pub fn object_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = receiver.class(vm);
    let name = vm.heap.class(class_id).name.clone();
    Ok(vm.alloc_string_value(name))
}

/// Signature: `Object::class` — returns the receiver's class.
#[phalcom_native_macros::primitive(
    Object,
    "class",
    params = [],
    returns = Class,
    types = "() -> Class",
    effects = pure
)]
pub fn object_class(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::obj(receiver.class(vm)))
}

/// Signature: `Object::toString` — the default display string (U-CORE-4,
/// [ADR-0015](../../../docs/adr/accepted/0015-object-default-tostring.md)).
#[phalcom_native_macros::primitive(
    Object,
    "toString",
    params = [],
    returns = String,
    types = "() -> String",
    effects = pure
)]
pub fn object_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let own_name = receiver.as_obj().and_then(|id| vm.heap.as_class(id).map(|c| c.name.clone()));
    if let Some(name) = own_name {
        return Ok(vm.alloc_string_value(name)); // class receiver -> own name (fixes F4)
    }
    let class_id = receiver.class(vm);
    let name = vm.heap.class(class_id).name.clone();
    Ok(vm.alloc_string_value(format!("<{name}>")))
}

/// Signature: `Object::hash` — a stable identity digest of the heap handle.
#[phalcom_native_macros::primitive(
    Object,
    "hash",
    params = [],
    returns = Int,
    types = "() -> Int",
    effects = pure
)]
pub fn object_hash(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let bits = if let Some(id) = receiver.as_obj() {
        id.to_opaque_u64()
    } else if let Some(b) = receiver.as_bool() {
        u64::from(b)
    } else if let Some(n) = receiver.as_int() {
        n as u64
    } else if let Some(n) = receiver.as_float() {
        n.to_bits()
    } else if let Some(s) = receiver.symbol_value() {
        u64::from(s.0)
    } else {
        0
    };
    Ok(crate::primitive::hash_code(bits))
}

/// Signature: `Object::class=(_)` — always an error; an object's class is fixed.
#[phalcom_native_macros::primitive(
    Object,
    "class=(put)",
    params = [Object],
    returns = Never,
    types = "(Object) -> Never",
    flow = never
)]
pub fn object_set_class(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::InvalidSetClass.into())
}

/// Signature: `Object::==(_)` — the base equality send (U5, control-flow.md
#[phalcom_native_macros::primitive(
    Object,
    "==(_)",
    params = [Object],
    returns = Bool,
    types = "(Object) -> Bool",
    effects = pure
)]
pub fn object_eq(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(receiver.value_eq(&args[0], &vm.heap)))
}

/// Signature: `Object::===(_)` — exact representation/identity sameness.
#[phalcom_native_macros::primitive(
    Object,
    "===(_)",
    params = [Object],
    returns = Bool,
    types = "(Object) -> Bool",
    effects = pure
)]
pub fn object_same(_vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(receiver.same_as(&args[0])))
}

/// Default value-pattern relation. The RHS is the receiver at the lowered
/// call site, so ordinary objects behave as equality patterns.
#[phalcom_native_macros::primitive(
    Object,
    "matches(_)",
    params = [Object],
    returns = Bool,
    types = "(Object) -> Bool",
    effects = pure
)]
pub fn object_matches(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(receiver.value_eq(&args[0], &vm.heap)))
}

/// Reports whether exact selector lookup would succeed, including an
/// accepting rest-family method. This intentionally bypasses DNU and never
/// invokes the selected method.
#[phalcom_native_macros::primitive(
    Object,
    "understands(_)",
    params = [Symbol],
    returns = Bool,
    types = "(Symbol) -> Bool",
    effects = pure
)]
pub fn object_understands(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let selector = args.first().and_then(|value| value.symbol_value()).ok_or_else(|| RuntimeError::Type {
        expected: "Symbol",
        found: args.first().map_or("missing", Value::type_name),
    })?;
    if receiver.lookup_method(vm, selector).is_some() {
        return Ok(Value::bool(true));
    }
    let (name, slots, kind) = decode_selector(vm.resolve_symbol(selector));
    let positional_count = slots.iter().filter(|slot| slot.is_none()).count();
    let labels = slots
        .iter()
        .filter_map(|slot| slot.as_ref())
        .map(|label| vm.interner.intern(label))
        .collect::<Vec<_>>();
    let base = vm.interner.intern(&name);
    let receiver_class = receiver.class(vm);
    let accepts_rest = matches!(kind, SignatureKind::Method(_)) && vm.lookup_rest_method(receiver_class, base, positional_count, &labels).is_some();
    Ok(Value::bool(accepts_rest))
}

/// Signature: `Object::!=(_)` — the base inequality send; the logical
#[phalcom_native_macros::primitive(
    Object,
    "!=(_)",
    params = [Object],
    returns = Bool,
    types = "(Object) -> Bool",
    effects = pure
)]
pub fn object_neq(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(!receiver.value_eq(&args[0], &vm.heap)))
}

/// Shape-aware `Object#perform(_,***)` gateway. The first positional value is
/// the complete selector; all remaining values retain their canonical
/// positional/labeled lanes and enter ordinary dispatch directly.
#[phalcom_native_macros::primitive(Object, "perform(_,***)", abi = shape)]
pub fn object_perform_shape(vm: &mut VM, receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    let selector_value = args.positional(vm, 0).ok_or_else(|| RuntimeError::Arity {
        signature: "perform",
        expected: 1,
        found: args.positional_count(),
    })?;
    let selector = expect_value!(&selector_value, Symbol);
    let positional_count = args.positional_count().checked_sub(1).ok_or_else(|| RuntimeError::Arity {
        signature: "perform",
        expected: 1,
        found: args.positional_count(),
    })?;
    let labels = args.labels();
    let receiver_index = args.receiver_index();
    let residual = vm.stack[receiver_index + 2..].to_vec();
    vm.stack[receiver_index] = receiver;
    vm.stack.truncate(receiver_index + 1);
    vm.stack.extend_from_slice(&residual);
    vm.dispatch_shape_at_as(
        receiver_index,
        selector,
        positional_count,
        labels,
        phalcom_common::range::SourceRange::default(),
        args.caller_authority(),
    )
}

/// Signature: `Object::respondsTo(_)` — returns whether the receiver's class
#[phalcom_native_macros::primitive(
    Object,
    "respondsTo(_)",
    params = [Symbol],
    returns = Bool,
    types = "(Symbol) -> Bool",
    effects = pure
)]
pub fn object_responds_to(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let selector = expect_value!(&args[0], Symbol);
    let responds = receiver
        .lookup_method(vm, selector)
        .is_some_and(|method| vm.authorize_method_access(method).is_ok());
    Ok(Value::bool(responds))
}

/// Signature: `Object::methodFor(_)` — reifies the
#[phalcom_native_macros::primitive(
    Object,
    "methodFor(_)",
    params = [Symbol],
    returns = "Option<Method>",
    types = "(Symbol) -> Option<Method>",
    effects = pure
)]
pub fn object_method_for(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let selector = expect_value!(&args[0], Symbol);
    match receiver.lookup_method(vm, selector) {
        Some(method_id) if vm.authorize_method_access(method_id).is_ok() => Ok(Value::obj(method_id)),
        None => Ok(vm.none_value()),
        Some(_) => Ok(vm.none_value()),
    }
}

/// Signature: `Object::doesNotUnderstand(_)` — the *default* miss handler
#[phalcom_native_macros::primitive(
    Object,
    "doesNotUnderstand(_)",
    params = [Message],
    returns = Never,
    types = "(Message) -> Never",
    raises = [MessageNotUnderstood],
    flow = never
)]
pub fn object_does_not_understand(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let selector = match message_slot(vm, &args[0], 0).and_then(|v| v.symbol_value()) {
        Some(sym) => vm.resolve_symbol(sym).to_string(),
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
    let mnu = Value::obj(vm.heap.alloc(Object::Instance(inst)));

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
    value.as_obj().and_then(|id| vm.heap.as_instance(id).map(|instance| instance.slots[index]))
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
#[phalcom_native_macros::primitive(
    Message,
    "selector",
    params = [],
    returns = Symbol,
    types = "() -> Symbol",
    effects = pure
)]
pub fn message_selector(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    message_slot(vm, receiver, 0).ok_or_else(|| not_a_message(receiver))
}

/// Signature: `Message::name` — the bare method-name [`String`] (slot 1),
#[phalcom_native_macros::primitive(
    Message,
    "name",
    params = [],
    returns = String,
    types = "() -> String",
    effects = pure
)]
pub fn message_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    message_slot(vm, receiver, 1).ok_or_else(|| not_a_message(receiver))
}

/// Signature: `Message::labels` — the [`List`](crate::heap::ListObject) of
#[phalcom_native_macros::primitive(
    Message,
    "labels",
    params = [],
    returns = List,
    types = "() -> List",
    effects = pure
)]
pub fn message_labels(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    message_slot(vm, receiver, 2).ok_or_else(|| not_a_message(receiver))
}

/// Signature: `Message::args` — the [`List`](crate::heap::ListObject) of the
#[phalcom_native_macros::primitive(
    Message,
    "args",
    params = [],
    returns = List,
    types = "() -> List",
    effects = pure
)]
pub fn message_args(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    message_slot(vm, receiver, 3).ok_or_else(|| not_a_message(receiver))
}

/// Signature: `Object::__invariantEnter()` — the entry half of the
#[phalcom_native_macros::primitive(
    Object,
    "_$invariantEnter()",
    params = [],
    returns = Bool,
    types = "() -> Bool",
    visibility = internal
)]
pub fn object_invariant_enter(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let Some(id) = receiver.as_obj() else {
        return Ok(Value::bool(true));
    };
    let is_owner = vm.checking.insert(id);
    Ok(Value::bool(is_owner))
}

/// Signature: `Object::__invariantExit()` — the exit half of the `@invariant`
#[phalcom_native_macros::primitive(
    Object,
    "_$invariantExit()",
    params = [],
    returns = Option,
    types = "() -> Option",
    visibility = internal
)]
pub fn object_invariant_exit(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if let Some(id) = receiver.as_obj() {
        vm.checking.remove(&id);
    }
    Ok(vm.none_value())
}
