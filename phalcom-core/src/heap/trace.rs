//! Precise tracing: the outgoing-edge enumeration the collector marks through.
//!
//! Realises [memory-management.md §2.3](../../../docs/spec/v0.2/memory-management.md)
//! (normative) per [ADR-0050](../../../docs/adr/accepted/0050-non-moving-mark-sweep-collector.md)
//! §3. One exhaustive `match` over [`Object`] yields every child handle an object
//! stores; [`Heap::collect`](super::Heap::collect) drives it from a worklist.
//!
//! Two disciplines this module exists to enforce:
//!
//! 1. **Exhaustive match, no wildcard.** A new [`Object`] variant must fail to
//!    compile until it declares its edges. There is deliberately no `_ => {}` arm.
//! 2. **`Value` children only through [`Value::as_obj`].** Never match `Value`'s
//!    arms here — that accessor is the seam that keeps this file independent of a
//!    future NaN-boxed `Value` ([ADR-0010](../../../docs/adr/accepted/0010-tagged-value-enum.md)).
//!
//! **The known gap:** the exhaustive match catches a new *variant*, never a new
//! *field* on an existing variant. Five edges (`Class.attributes`,
//! `Method.attributes`, `Method.contracts`, `Module.attributes`, `Fiber.checking`)
//! reached HEAD unnoticed exactly that way — see forge finding F6. When adding a
//! handle-bearing field to any payload struct below, add it here **and** to
//! memory-management.md §2.3 in the same change.

use super::{ObjRef, Object, Upvalue};
use crate::frame::{CallContext, CallFrame};
use crate::method::MethodKind;
use crate::value::Value;

/// Calls `push` once for every handle `frame` stores.
///
/// Shared by [`trace_object`]'s `Fiber` arm (a parked fiber's saved frames) and
/// the VM's root enumeration (the running fiber's live `frames` mirror), so the
/// two can never disagree about a frame's edges
/// ([memory-management.md §2.1/§2.3](../../../docs/spec/v0.2/memory-management.md)).
///
/// `home_frame_token` is **not** an edge: a [`crate::frame::FrameToken`] is an
/// index plus a generation counter, not a handle.
pub fn trace_frame(frame: &CallFrame, push: &mut impl FnMut(ObjRef)) {
    push(frame.closure);
    match frame.context {
        CallContext::Instance { instance } => push(instance),
        CallContext::Class { class } => push(class),
        CallContext::Module { module } => push(module),
        // An immediate receiver (`Bool`/`Number`/`Symbol`) carries no handle —
        // but it is still a `Value`, so it goes through the `as_obj` seam rather
        // than being assumed handle-free.
        CallContext::Immediate { value } => {
            if let Some(id) = value.as_obj() {
                push(id);
            }
        }
    }
}

/// Calls `push` once for every handle `value` points at (zero or one).
fn trace_value(value: Value, push: &mut impl FnMut(ObjRef)) {
    if let Some(id) = value.as_obj() {
        push(id);
    }
}

/// Calls `push` once for every handle `obj` stores — its outgoing edges.
///
/// Normative edge set: [memory-management.md §2.3](../../../docs/spec/v0.2/memory-management.md).
/// `push` may be called with the same handle more than once; the caller
/// de-duplicates via the mark set.
///
/// Tracing a **currently-running** fiber's `FiberObject` is harmless but
/// pointless: its `stack`/`frames`/`open_upvalues`/`checking` buffers are empty
/// while the VM mirror holds the authoritative state (§2.3). The VM roots the
/// mirror.
pub fn trace_object(obj: &Object, push: &mut impl FnMut(ObjRef)) {
    match obj {
        Object::Instance(inst) => {
            push(inst.class);
            for slot in inst.slots.iter() {
                trace_value(*slot, push);
            }
        }
        Object::Class(class) => {
            push(class.class);
            if let Some(sup) = class.superclass {
                push(sup);
            }
            // Methods are heap objects; the map's keys are `Symbol`s (non-roots).
            for method in class.methods.values() {
                push(*method);
            }
            for method in class.rest_methods.values() {
                push(*method);
            }
            for slot in class.static_slots.iter() {
                trace_value(*slot, push);
            }
            // Retained `@attribute` instances (M-ATTR-ROOT). `name` is a Rust
            // `String`, `field_slots`/`base_names` hold only `Symbol`s — none are edges.
            for attr in &class.attributes {
                trace_value(*attr, push);
            }
        }
        Object::Method(method) => {
            // A primitive holds a Rust fn pointer — no Phalcom handle. A bytecode
            // method's constants live in its `ClosureObject`, not here.
            match method.kind {
                MethodKind::Closure(closure) => push(closure),
                MethodKind::Primitive(_) => {}
            }
            if let Some(holder) = method.holder {
                push(holder);
            }
            // `@requires`/`@ensures`/`@invariant` metadata (U-ANNOT-CONTRACTS):
            // the `.0` is a `Symbol`, the `.1` is the predicate `Value`.
            if let Some(contracts) = &method.contracts {
                for (_sym, value) in contracts {
                    trace_value(*value, push);
                }
            }
            for attr in &method.attributes {
                trace_value(*attr, push);
            }
        }
        Object::Module(module) => {
            if let Some(closure) = module.closure {
                push(closure);
            }
            for global in &module.globals {
                trace_value(*global, push);
            }
            for attr in &module.attributes {
                trace_value(*attr, push);
            }
        }
        Object::Closure(closure) => {
            push(closure.module);
            for upvalue in &closure.upvalues {
                push(*upvalue);
            }
            // The constant pool holds string literals and selector symbols.
            for constant in &closure.callable.chunk.constants {
                trace_value(*constant, push);
            }
        }
        // A leaf: `value: String` + `hash: u32`, no handles.
        Object::Str(_) => {}
        Object::Block(block) => {
            // The block *is* the only retainer of its closure once passed around.
            // `home_frame_token` is an index+generation, not a handle.
            push(block.closure);
        }
        Object::BoundMethod(bound) => {
            push(bound.method);
            trace_value(bound.receiver, push);
        }
        Object::Upvalue(upvalue) => match upvalue {
            // NOT merely a stack index: the aliased slot lives on `fiber`'s stack,
            // which is the VM mirror only while that fiber is *current* — otherwise
            // it is parked inside the `FiberObject`, reachable only via this handle.
            Upvalue::Open { fiber, slot: _ } => push(*fiber),
            Upvalue::Closed(value) => trace_value(*value, push),
        },
        Object::List(list) => {
            for element in list.elements() {
                trace_value(*element, push);
            }
        }
        Object::Fiber(fiber) => {
            for value in &fiber.stack {
                trace_value(*value, push);
            }
            for frame in &fiber.frames {
                trace_frame(frame, push);
            }
            for cell in fiber.open_upvalues.values() {
                push(*cell);
            }
            if let Some(resumer) = fiber.resumer {
                push(resumer);
            }
            trace_value(fiber.result, push);
            if let Some(entry) = fiber.entry {
                push(entry);
            }
            // Receivers under `@invariant` re-entrancy checking (U-ANNOT-CONTRACTS).
            for receiver in &fiber.checking {
                push(*receiver);
            }
        }
        // `Set` shares `MapObject`; its `.1` slots are always `Value::Nil`, which
        // `as_obj` filters out — so one arm body is correct for both.
        Object::Map(map) | Object::Set(map) => {
            for (key, value) in map.entries() {
                trace_value(key, push);
                trace_value(value, push);
            }
        }
        // `Bytes` holds raw octets, never a `Value` — nothing to visit. An
        // explicit arm (not a `_` wildcard) so the match stays exhaustive and
        // the next variant's author is forced to decide (impl/bytes.md §2.3).
        Object::Bytes(_) => {}
        Object::Tuple(tuple) => {
            for element in tuple.values() {
                trace_value(*element, push);
            }
        }
        Object::Record(record) => {
            for value in record.values() {
                trace_value(*value, push);
            }
        }
        Object::Range(range) => {
            if let Some(lower) = range.lower() {
                trace_value(lower, push);
            }
            if let Some(upper) = range.upper() {
                trace_value(upper, push);
            }
        }
        Object::Family(family) => {
            // `selector` is a `Symbol`, `open` a `bool` — neither is an edge.
            trace_value(family.recv, push);
        }
        // `LargeInt` holds an arbitrary-precision integer, no `Value` handles.
        Object::LargeInt(_) => {}
        Object::PackBuilder(builder) => {
            for value in builder.positionals().iter().chain(builder.labeled_values()) {
                trace_value(*value, push);
            }
        }
        Object::RecordLiteralBuilder(builder) => {
            for (_, value) in builder.entries() {
                trace_value(*value, push);
            }
        }
    }
}
