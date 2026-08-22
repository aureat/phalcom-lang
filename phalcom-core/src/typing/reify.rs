//! Lazy descriptor materialization and reification rules.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{ClassId, Heap, ObjRef, Object, TypingObject, TypingPayload};
use crate::typing::handle::{RuntimeKindRef, RuntimeSemanticHandle, RuntimeTypeRef};
use crate::typing::registry::RuntimeTypingRegistry;
use crate::value::Value;
use phalcom_type_meta::type_node::TypeNode;

/// Reifies a runtime semantic type handle into a `Value`.
///
/// Follows Spec 02 invariant:
/// 1. Nominal forms resolve to the existing runtime `ClassObject` (`reify(Int) === Int`).
/// 2. Synthetic forms create or reuse a weakly cached descriptor object (`Object::Typing`).
pub fn reify_type_form(
    context_ref: ObjRef,
    handle: RuntimeTypeRef,
    registry: &RuntimeTypingRegistry,
    heap: &mut Heap,
    descriptor_class: ClassId,
) -> PhResult<Value> {
    // 1. Check if this is a nominal type form in a loaded base pool
    if let RuntimeTypeRef::Base { pool, node } = handle {
        if let Some(loaded) = registry.get_pool(pool) {
            if let Some(entry) = loaded.bundle.types.get(node.0 as usize) {
                if let TypeNode::Nominal { ref declaration } = entry.form {
                    if let Some(class_id) = registry.resolve_nominal(declaration) {
                        return Ok(Value::obj(class_id));
                    }
                }
            }
        }
    }

    let sem_handle = RuntimeSemanticHandle::Type(handle);

    // 2. Probe weak descriptor cache on context using Heap::try_get (NON-PANICKING)
    let context_obj = heap
        .as_typing_object(context_ref)
        .ok_or_else(|| RuntimeError::Internal(format!("ObjRef {context_ref:?} is not a TypingObject")))?;

    if let TypingPayload::Context(ref ctx_data) = context_obj.payload {
        if let Some(&cached_ref) = ctx_data.descriptor_cache.get(&sem_handle) {
            if let Some(cached_obj) = heap.try_get(cached_ref) {
                if let Object::Typing(t) = cached_obj {
                    if let TypingPayload::Descriptor { context, handle: h } = t.payload {
                        if t.class == descriptor_class && context == context_ref && h == sem_handle {
                            return Ok(Value::obj(cached_ref));
                        }
                    }
                }
            }
        }
    }

    // 3. Allocate fresh descriptor
    let descriptor = TypingObject {
        class: descriptor_class,
        payload: TypingPayload::Descriptor {
            context: context_ref,
            handle: sem_handle,
        },
    };
    let desc_ref = heap.alloc(Object::Typing(Box::new(descriptor)));

    // 4. Update weak cache in context
    let context_obj_mut = heap
        .as_typing_object_mut(context_ref)
        .ok_or_else(|| RuntimeError::Internal(format!("ObjRef {context_ref:?} is not a TypingObject")))?;
    if let TypingPayload::Context(ref mut ctx_data) = context_obj_mut.payload {
        ctx_data.descriptor_cache.insert(sem_handle, desc_ref);
    }

    Ok(Value::obj(desc_ref))
}

pub fn reify_kind_form(
    context_ref: ObjRef,
    handle: RuntimeKindRef,
    registry: &RuntimeTypingRegistry,
    heap: &mut Heap,
    type_class: ClassId,
    function_kind_class: ClassId,
) -> PhResult<Value> {
    let is_type = match handle {
        RuntimeKindRef::Overlay(id) => heap
            .as_typing_object(context_ref)
            .and_then(|object| match &object.payload {
                TypingPayload::Context(data) => data
                    .overlay
                    .kind_node(id)
                    .map(|node| matches!(node, crate::typing::overlay::RuntimeOverlayKindNode::Type)),
                TypingPayload::Descriptor { .. } => None,
            })
            .unwrap_or(false),
        RuntimeKindRef::Base { pool, node } => registry
            .get_pool(pool)
            .and_then(|loaded| loaded.bundle.kinds.get(node.0 as usize))
            .is_some_and(|entry| matches!(entry.node, phalcom_type_meta::kind::KindNode::Type)),
    };
    if is_type {
        return Ok(Value::obj(type_class));
    }

    let semantic_handle = RuntimeSemanticHandle::Kind(handle);
    if let Some(context_object) = heap.as_typing_object(context_ref) {
        if let TypingPayload::Context(data) = &context_object.payload {
            if let Some(&cached_ref) = data.descriptor_cache.get(&semantic_handle) {
                if let Some(Object::Typing(cached)) = heap.try_get(cached_ref) {
                    if let TypingPayload::Descriptor { context, handle } = cached.payload {
                        if context == context_ref && handle == semantic_handle {
                            return Ok(Value::obj(cached_ref));
                        }
                    }
                }
            }
        }
    }

    let descriptor = TypingObject {
        class: function_kind_class,
        payload: TypingPayload::Descriptor {
            context: context_ref,
            handle: semantic_handle,
        },
    };
    let descriptor_ref = heap.alloc(Object::Typing(Box::new(descriptor)));
    let context_object = heap
        .as_typing_object_mut(context_ref)
        .ok_or_else(|| RuntimeError::Internal(format!("ObjRef {context_ref:?} is not a TypingObject")))?;
    if let TypingPayload::Context(data) = &mut context_object.payload {
        data.descriptor_cache.insert(semantic_handle, descriptor_ref);
    }
    Ok(Value::obj(descriptor_ref))
}
