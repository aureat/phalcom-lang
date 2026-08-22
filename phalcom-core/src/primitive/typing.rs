//! Native boundary for explicit typing reflection.
//!
//! This module owns only runtime adaptation. Canonical type semantics remain in
//! `phalcom-semantic`/`phalcom-type-meta`; overlay nodes are validated handles,
//! never user-supplied numeric IDs.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{InstanceObject, ObjRef, Object, TypingPayload};
use crate::method::MethodObject;
use crate::primitive::{expect_tuple, primitive, primitive_static};
use crate::typing::capability::TypingCapability;
use crate::typing::context::TypingContextData;
use crate::typing::handle::{RuntimeKindRef, RuntimeSemanticHandle, RuntimeTypeRef};
use crate::typing::inspect;
use crate::typing::overlay::{RuntimeOverlayKindNode, RuntimeOverlayTypeNode};
use crate::value::Value;
use crate::vm::VM;
use phalcom_type_meta::header::MetadataProfile;

fn class(vm: &VM, name: &str) -> PhResult<crate::heap::ClassId> {
    vm.universe
        .typing_classes
        .get(name)
        .ok_or_else(|| RuntimeError::Internal(format!("typing class `{name}` is not bootstrapped")).into())
}

fn context_ref(receiver: &Value, vm: &VM) -> PhResult<ObjRef> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "TypingContext",
        found: receiver.type_name(),
    })?;
    match vm.heap.as_typing_object(id).map(|object| &object.payload) {
        Some(TypingPayload::Context(_)) => Ok(id),
        _ => Err(RuntimeError::Type {
            expected: "TypingContext",
            found: receiver.type_name(),
        }
        .into()),
    }
}

fn descriptor_parts(receiver: &Value, vm: &VM) -> PhResult<(ObjRef, RuntimeTypeRef)> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "TypeForm",
        found: receiver.type_name(),
    })?;
    match vm.heap.as_typing_object(id).map(|object| &object.payload) {
        Some(TypingPayload::Descriptor {
            context,
            handle: RuntimeSemanticHandle::Type(handle),
        }) => Ok((*context, *handle)),
        _ => Err(RuntimeError::Type {
            expected: "TypeForm",
            found: receiver.type_name(),
        }
        .into()),
    }
}

fn alloc_context(vm: &mut VM, profile: MetadataProfile) -> PhResult<Value> {
    let class = class(vm, "TypingContext")?;
    let data = TypingContextData::with_profile(Box::new([]), profile);
    let object = crate::heap::TypingObject {
        class,
        payload: TypingPayload::Context(data),
    };
    Ok(Value::obj(vm.heap.alloc(Object::Typing(Box::new(object)))))
}

fn alloc_variant(vm: &mut VM, name: &str, payload: Option<Value>) -> PhResult<Value> {
    let variant = class(vm, name)?;
    let field_count = vm.heap.class(variant).field_count;
    let mut instance = InstanceObject::new(variant, field_count);
    if let Some(value) = payload {
        if instance.slots.is_empty() {
            return Err(RuntimeError::Internal(format!("typing result `{name}` has no payload slot")).into());
        }
        instance.slots[0] = value;
    }
    Ok(Value::obj(vm.heap.alloc(Object::Instance(instance))))
}

fn nominal_handle(vm: &mut VM, context: ObjRef, value: &Value) -> PhResult<RuntimeTypeRef> {
    if let Some(class) = value.as_obj() {
        if vm.heap.as_class(class).is_some() {
            let context_object = vm
                .heap
                .as_typing_object_mut(context)
                .ok_or_else(|| RuntimeError::Internal("typing context disappeared".to_string()))?;
            if let TypingPayload::Context(data) = &mut context_object.payload {
                return Ok(inspect::nominal_overlay(data, class));
            }
        }
        if let Some(TypingPayload::Descriptor {
            handle: RuntimeSemanticHandle::Type(handle),
            ..
        }) = vm.heap.as_typing_object(class).map(|object| &object.payload)
        {
            return Ok(*handle);
        }
    }
    Err(RuntimeError::Type {
        expected: "TypeForm",
        found: value.type_name(),
    }
    .into())
}

fn tuple_type_args(vm: &mut VM, context: ObjRef, value: &Value) -> PhResult<Box<[RuntimeTypeRef]>> {
    let tuple = expect_tuple(vm, value)?;
    let values = vm.heap.tuple(tuple).values().to_vec();
    values
        .iter()
        .map(|value| nominal_handle(vm, context, value))
        .collect::<PhResult<Vec<_>>>()
        .map(Vec::into_boxed_slice)
}

fn ensure_capability(vm: &VM, context: ObjRef, capability: TypingCapability) -> PhResult<()> {
    let allowed = match vm.heap.as_typing_object(context).map(|object| &object.payload) {
        Some(TypingPayload::Context(data)) => data.can(capability),
        _ => false,
    };
    if allowed {
        Ok(())
    } else {
        Err(RuntimeError::Message(format!("typing capability denied: {}", capability.display())).into())
    }
}

fn context_known(vm: &mut VM, context: ObjRef, handle: RuntimeTypeRef) -> PhResult<Value> {
    let descriptor_class = descriptor_class_for_handle(vm, context, handle)?;
    let value = crate::typing::reify::reify_type_form(context, handle, &vm.typing_registry, &mut vm.heap, descriptor_class)?;
    alloc_variant(vm, "TypingKnown", Some(value))
}

fn descriptor_class_for_handle(vm: &VM, context: ObjRef, handle: RuntimeTypeRef) -> PhResult<crate::heap::ClassId> {
    let name = match handle {
        RuntimeTypeRef::Overlay(id) => match vm.heap.as_typing_object(context).and_then(|object| match &object.payload {
            TypingPayload::Context(data) => data.overlay.type_node(id).map(|node| match node {
                RuntimeOverlayTypeNode::Applied { .. } => "AppliedType",
                RuntimeOverlayTypeNode::Union(_) => "UnionType",
                RuntimeOverlayTypeNode::Tuple(_) => "TupleType",
                RuntimeOverlayTypeNode::Callable { .. } => "CallableType",
                RuntimeOverlayTypeNode::Nominal { .. } => "TypeDescriptor",
            }),
            TypingPayload::Descriptor { .. } => None,
        }) {
            Some(name) => name,
            None => "TypeDescriptor",
        },
        RuntimeTypeRef::Base { pool, node } => match vm
            .typing_registry
            .get_pool(pool)
            .and_then(|pool| pool.bundle.types.get(node.0 as usize))
            .map(|entry| &entry.form)
        {
            Some(phalcom_type_meta::type_node::TypeNode::Applied { .. }) => "AppliedType",
            Some(phalcom_type_meta::type_node::TypeNode::Union(_)) => "UnionType",
            Some(phalcom_type_meta::type_node::TypeNode::Tuple(_)) => "TupleType",
            Some(phalcom_type_meta::type_node::TypeNode::Record(_)) => "RecordType",
            Some(phalcom_type_meta::type_node::TypeNode::Callable(_)) => "CallableType",
            Some(phalcom_type_meta::type_node::TypeNode::TypeLambda(_)) => "TypeLambda",
            Some(phalcom_type_meta::type_node::TypeNode::SelfType(_)) => "SelfType",
            _ => "TypeDescriptor",
        },
    };
    class(vm, name)
}

fn context_kind_parts(receiver: &Value, vm: &VM) -> PhResult<(ObjRef, RuntimeKindRef)> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "KindDescriptor",
        found: receiver.type_name(),
    })?;
    match vm.heap.as_typing_object(id).map(|object| &object.payload) {
        Some(TypingPayload::Descriptor {
            context,
            handle: RuntimeSemanticHandle::Kind(handle),
        }) => Ok((*context, *handle)),
        _ => Err(RuntimeError::Type {
            expected: "KindDescriptor",
            found: receiver.type_name(),
        }
        .into()),
    }
}

fn reify_kind(vm: &mut VM, context: ObjRef, handle: RuntimeKindRef) -> PhResult<Value> {
    let type_class = class(vm, "Type")?;
    let function_kind_class = class(vm, "FunctionKind")?;
    crate::typing::reify::reify_kind_form(context, handle, &vm.typing_registry, &mut vm.heap, type_class, function_kind_class)
}

pub fn install(vm: &mut VM) {
    let behavior = vm.universe.classes.behavior_class;
    primitive!(vm, behavior, "kind", crate::method::SignatureKind::Getter, behavior_kind);
    primitive!(vm, behavior, "display", crate::method::SignatureKind::Getter, behavior_display);
    primitive!(vm, behavior, "freeParameterCount", crate::method::SignatureKind::Getter, zero_count);
    primitive!(
        vm,
        behavior,
        "remainingParameterCount",
        crate::method::SignatureKind::Getter,
        behavior_remaining_count
    );
    primitive!(vm, behavior, "equivalentTo", crate::method::SignatureKind::Method(1), behavior_equivalent);
    primitive!(vm, behavior, "subtypeOf", crate::method::SignatureKind::Method(1), behavior_subtype);

    let kind = vm.universe.typing_classes.get("KindDescriptor").expect("KindDescriptor");
    primitive!(vm, kind, "display", crate::method::SignatureKind::Getter, kind_display);
    primitive!(vm, kind, "argumentCount", crate::method::SignatureKind::Getter, kind_argument_count);
    primitive!(vm, kind, "argumentAt", crate::method::SignatureKind::Method(1), kind_argument_at);
    primitive!(vm, kind, "arguments", crate::method::SignatureKind::Getter, kind_arguments);
    primitive!(vm, kind, "result", crate::method::SignatureKind::Getter, kind_result);
    primitive!(vm, kind, "equivalentTo", crate::method::SignatureKind::Method(1), kind_equivalent);

    let descriptor = vm.universe.typing_classes.get("TypeDescriptor").expect("TypeDescriptor");
    primitive!(vm, descriptor, "kind", crate::method::SignatureKind::Getter, descriptor_kind);
    primitive!(vm, descriptor, "display", crate::method::SignatureKind::Getter, descriptor_display);
    primitive!(vm, descriptor, "freeParameterCount", crate::method::SignatureKind::Getter, zero_count);
    primitive!(
        vm,
        descriptor,
        "remainingParameterCount",
        crate::method::SignatureKind::Getter,
        descriptor_remaining_count
    );
    primitive!(vm, descriptor, "argumentCount", crate::method::SignatureKind::Getter, descriptor_argument_count);
    primitive!(vm, descriptor, "argumentAt", crate::method::SignatureKind::Method(1), descriptor_argument_at);
    primitive!(vm, descriptor, "equivalentTo", crate::method::SignatureKind::Method(1), descriptor_equivalent);
    primitive!(vm, descriptor, "subtypeOf", crate::method::SignatureKind::Method(1), descriptor_subtype);

    let typing = vm.universe.typing_classes.get("Typing").expect("Typing");
    primitive_static!(vm, typing, "current", crate::method::SignatureKind::Getter, typing_current);

    let context = vm.universe.typing_classes.get("TypingContext").expect("TypingContext");
    primitive_static!(vm, context, "new", crate::method::SignatureKind::Method(0), typing_context_new);
    primitive!(vm, context, "profile", crate::method::SignatureKind::Getter, typing_context_profile);
    primitive!(vm, context, "capabilities", crate::method::SignatureKind::Getter, typing_context_capabilities);
    primitive!(vm, context, "restrictTo", crate::method::SignatureKind::Method(1), typing_context_restrict);
    primitive!(vm, context, "refresh", crate::method::SignatureKind::Getter, typing_context_refresh);
    primitive!(vm, context, "apply", crate::method::SignatureKind::Method(2), typing_context_apply);
    primitive!(vm, context, "unionOf", crate::method::SignatureKind::Method(1), typing_context_union_of);
    primitive!(vm, context, "tupleOf", crate::method::SignatureKind::Method(1), typing_context_tuple_of);
    primitive!(vm, context, "callable", crate::method::SignatureKind::Method(2), typing_context_callable);

    for (name, getter) in [
        ("TypingKnown", result_value),
        ("TypingUnknown", result_value),
        ("TypingInvalid", result_value),
        ("TypingUnavailable", result_value),
        ("TypingBudgetExceeded", result_value),
        ("TypingInternalFailure", result_value),
        ("RelationSatisfied", result_value),
        ("RelationRejected", result_value),
        ("RelationDynamicBoundary", result_value),
        ("RelationBlocked", result_value),
        ("RelationBudgetExceeded", result_value),
        ("RelationInternalFailure", result_value),
        ("MemberFound", result_value),
        ("MemberMissing", result_value),
        ("MemberDynamicBoundary", result_value),
        ("MemberBlocked", result_value),
        ("MemberBudgetExceeded", result_value),
        ("MemberInternalFailure", result_value),
    ] {
        let result_class = vm.universe.typing_classes.get(name).expect("typing result class");
        primitive!(vm, result_class, "value", crate::method::SignatureKind::Getter, getter);
    }
}

fn typing_current(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    alloc_context(vm, MetadataProfile::RuntimePublic)
}

fn typing_context_new(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    alloc_context(vm, MetadataProfile::RuntimePublic)
}

fn typing_context_profile(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    let profile = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => data.profile,
        _ => unreachable!(),
    };
    Ok(Value::symbol(vm.get_or_intern(match profile {
        MetadataProfile::RuntimeMinimal => "RuntimeMinimal",
        MetadataProfile::RuntimePublic => "RuntimePublic",
        MetadataProfile::ToolingDebug => "ToolingDebug",
        MetadataProfile::Proof => "Proof",
    })))
}

fn typing_context_capabilities(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    let names = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => data.capabilities.iter().map(|capability| capability.display()).collect::<Vec<_>>(),
        _ => unreachable!(),
    }
    .into_iter()
    .map(|name| Value::symbol(vm.get_or_intern(name)))
    .collect();
    Ok(Value::obj(vm.heap.alloc(Object::Tuple(crate::heap::TupleObject::positional(names)))))
}

fn typing_context_restrict(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    let requested = expect_tuple(
        vm,
        args.first().ok_or_else(|| RuntimeError::Arity {
            signature: "restrictTo(_)",
            expected: 1,
            found: args.len(),
        })?,
    )?;
    let mut requested_set = crate::typing::capability::TypingCapabilities::empty();
    for value in vm.heap.tuple(requested).values() {
        let Some(symbol) = value.symbol_value() else { continue };
        let name = vm.resolve_symbol(symbol);
        if let Some(capability) = TypingCapability::ALL.into_iter().find(|capability| capability.display() == name) {
            requested_set = requested_set.with(capability);
        }
    }
    let data = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => {
            let mut data = data.clone();
            data.capabilities = data.capabilities.restricted_to(requested_set);
            data.descriptor_cache.clear();
            data
        }
        _ => unreachable!(),
    };
    let class = class(vm, "TypingContext")?;
    let object = crate::heap::TypingObject {
        class,
        payload: TypingPayload::Context(data),
    };
    Ok(Value::obj(vm.heap.alloc(Object::Typing(Box::new(object)))))
}

fn typing_context_refresh(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    let data = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => {
            let mut data = data.clone();
            data.descriptor_cache.clear();
            data
        }
        _ => unreachable!(),
    };
    let class = class(vm, "TypingContext")?;
    let object = crate::heap::TypingObject {
        class,
        payload: TypingPayload::Context(data),
    };
    Ok(Value::obj(vm.heap.alloc(Object::Typing(Box::new(object)))))
}

fn typing_context_apply(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::ConstructTypeForms)?;
    let origin = args.first().ok_or_else(|| RuntimeError::Arity {
        signature: "apply(_,_)",
        expected: 2,
        found: args.len(),
    })?;
    let arguments = args.get(1).ok_or_else(|| RuntimeError::Arity {
        signature: "apply(_,_)",
        expected: 2,
        found: args.len(),
    })?;
    let origin = nominal_handle(vm, context, origin)?;
    let arguments = tuple_type_args(vm, context, arguments)?;
    let handle = {
        let object = vm
            .heap
            .as_typing_object_mut(context)
            .ok_or_else(|| RuntimeError::Internal("typing context disappeared".to_string()))?;
        let TypingPayload::Context(data) = &mut object.payload else { unreachable!() };
        data.overlay.type_ref(RuntimeOverlayTypeNode::Applied { origin, arguments })
    };
    context_known(vm, context, handle)
}

fn typing_context_union_of(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::ConstructTypeForms)?;
    let members = tuple_type_args(
        vm,
        context,
        args.first().ok_or_else(|| RuntimeError::Arity {
            signature: "unionOf(_)",
            expected: 1,
            found: args.len(),
        })?,
    )?;
    let handle = {
        let object = vm
            .heap
            .as_typing_object_mut(context)
            .ok_or_else(|| RuntimeError::Internal("typing context disappeared".to_string()))?;
        let TypingPayload::Context(data) = &mut object.payload else { unreachable!() };
        data.overlay.type_ref(RuntimeOverlayTypeNode::Union(members))
    };
    context_known(vm, context, handle)
}

fn typing_context_tuple_of(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::ConstructTypeForms)?;
    let types = tuple_type_args(
        vm,
        context,
        args.first().ok_or_else(|| RuntimeError::Arity {
            signature: "tupleOf(_)",
            expected: 1,
            found: args.len(),
        })?,
    )?;
    let elements = types
        .iter()
        .map(|ty| crate::typing::overlay::RuntimeTupleElement { label: None, ty: *ty })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let handle = {
        let object = vm
            .heap
            .as_typing_object_mut(context)
            .ok_or_else(|| RuntimeError::Internal("typing context disappeared".to_string()))?;
        let TypingPayload::Context(data) = &mut object.payload else { unreachable!() };
        data.overlay.type_ref(RuntimeOverlayTypeNode::Tuple(elements))
    };
    context_known(vm, context, handle)
}

fn typing_context_callable(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::ConstructTypeForms)?;
    let parameters = tuple_type_args(
        vm,
        context,
        args.first().ok_or_else(|| RuntimeError::Arity {
            signature: "callable(_,_)",
            expected: 2,
            found: args.len(),
        })?,
    )?;
    let returns = nominal_handle(
        vm,
        context,
        args.get(1).ok_or_else(|| RuntimeError::Arity {
            signature: "callable(_,_)",
            expected: 2,
            found: args.len(),
        })?,
    )?;
    let parameters = parameters
        .iter()
        .map(|ty| crate::typing::overlay::RuntimeCallableParameter {
            label: None,
            ty: *ty,
            rest: false,
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let handle = {
        let object = vm
            .heap
            .as_typing_object_mut(context)
            .ok_or_else(|| RuntimeError::Internal("typing context disappeared".to_string()))?;
        let TypingPayload::Context(data) = &mut object.payload else { unreachable!() };
        data.overlay.type_ref(RuntimeOverlayTypeNode::Callable {
            parameters,
            return_type: returns,
        })
    };
    context_known(vm, context, handle)
}

fn behavior_kind(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = receiver
        .as_obj()
        .filter(|id| vm.heap.as_class(*id).is_some())
        .ok_or_else(|| RuntimeError::Type {
            expected: "Class",
            found: receiver.type_name(),
        })?;
    let parameter_count = match vm.heap.class(class_id).name.as_str() {
        "List" | "Set" => 1,
        "Map" => 2,
        _ => 0,
    };
    if parameter_count == 0 {
        return Ok(Value::obj(class(vm, "Type")?));
    }

    let context = alloc_context(vm, MetadataProfile::RuntimeMinimal)?.as_obj().expect("typing context object");
    let handle = {
        let object = vm
            .heap
            .as_typing_object_mut(context)
            .ok_or_else(|| RuntimeError::Internal("typing context disappeared".to_string()))?;
        let TypingPayload::Context(data) = &mut object.payload else { unreachable!() };
        let type_kind = data.overlay.kind_ref(RuntimeOverlayKindNode::Type);
        data.overlay.kind_ref(RuntimeOverlayKindNode::Arrow {
            parameters: vec![type_kind; parameter_count].into_boxed_slice(),
            result: Box::new(type_kind),
        })
    };
    reify_kind(vm, context, handle)
}

fn behavior_display(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Class",
        found: receiver.type_name(),
    })?;
    let name = vm.heap.class(id).name.clone();
    Ok(vm.alloc_string_value(name))
}

fn zero_count(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::int(0))
}

fn behavior_remaining_count(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = receiver
        .as_obj()
        .filter(|id| vm.heap.as_class(*id).is_some())
        .ok_or_else(|| RuntimeError::Type {
            expected: "Class",
            found: receiver.type_name(),
        })?;
    let count = match vm.heap.class(class_id).name.as_str() {
        "List" | "Set" => 1,
        "Map" => 2,
        _ => 0,
    };
    Ok(Value::int(count))
}

fn behavior_equivalent(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(
        receiver.as_obj().is_some() && receiver.as_obj() == args.first().and_then(Value::as_obj),
    ))
}

fn behavior_subtype(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let left = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Class",
        found: receiver.type_name(),
    })?;
    let right = args.first().and_then(Value::as_obj).ok_or_else(|| RuntimeError::Type {
        expected: "Class",
        found: args.first().map_or("missing", Value::type_name),
    })?;
    let mut current = Some(left);
    let mut satisfied = false;
    while let Some(class_id) = current {
        if class_id == right {
            satisfied = true;
            break;
        }
        current = vm.heap.class(class_id).superclass;
    }
    alloc_variant(
        vm,
        if satisfied { "RelationSatisfied" } else { "RelationRejected" },
        Some(Value::bool(satisfied)),
    )
}

fn kind_display(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if receiver.as_obj() == Some(class(vm, "Type")?) {
        return Ok(vm.alloc_string_value("Type".to_string()));
    }
    let (context, handle) = context_kind_parts(receiver, vm)?;
    let context_data = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => data,
        _ => unreachable!(),
    };
    let rendered = inspect::kind_display(context_data, &vm.typing_registry, handle);
    Ok(vm.alloc_string_value(rendered))
}

fn kind_argument_count(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if receiver.as_obj() == Some(class(vm, "Type")?) {
        return Ok(Value::int(0));
    }
    let (context, handle) = context_kind_parts(receiver, vm)?;
    let context_data = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => data,
        _ => unreachable!(),
    };
    Ok(Value::int(
        inspect::kind_children(context_data, &vm.typing_registry, handle).map_or(0, |children| children.len() as i64),
    ))
}

fn kind_arguments(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if receiver.as_obj() == Some(class(vm, "Type")?) {
        return Ok(Value::obj(vm.heap.alloc(Object::Tuple(crate::heap::TupleObject::positional(Vec::new())))));
    }
    let (context, handle) = context_kind_parts(receiver, vm)?;
    let children = {
        let context_data = match &vm.heap.typing_object(context).payload {
            TypingPayload::Context(data) => data,
            _ => unreachable!(),
        };
        inspect::kind_children(context_data, &vm.typing_registry, handle).unwrap_or_default()
    };
    let mut values = Vec::with_capacity(children.len());
    for child in children {
        values.push(reify_kind(vm, context, child)?);
    }
    Ok(Value::obj(vm.heap.alloc(Object::Tuple(crate::heap::TupleObject::positional(values)))))
}

fn kind_result(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if receiver.as_obj() == Some(class(vm, "Type")?) {
        return Ok(Value::none());
    }
    let (context, handle) = context_kind_parts(receiver, vm)?;
    let result = {
        let context_data = match &vm.heap.typing_object(context).payload {
            TypingPayload::Context(data) => data,
            _ => unreachable!(),
        };
        inspect::kind_result(context_data, &vm.typing_registry, handle)
    };
    match result {
        Some(result) => Ok(reify_kind(vm, context, result)?.wrap_some()?),
        None => Ok(Value::none()),
    }
}

fn kind_argument_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let index = args.first().and_then(|value| value.as_int()).ok_or_else(|| RuntimeError::Type {
        expected: "Int",
        found: args.first().map_or("missing", Value::type_name),
    })?;
    let (context, handle) = context_kind_parts(receiver, vm)?;
    let child = {
        let context_data = match &vm.heap.typing_object(context).payload {
            TypingPayload::Context(data) => data,
            _ => unreachable!(),
        };
        inspect::kind_children(context_data, &vm.typing_registry, handle).and_then(|children| children.get(index as usize).copied())
    }
    .ok_or_else(|| RuntimeError::Message("kind argument index out of range".into()))?;
    reify_kind(vm, context, child)
}

fn kind_equivalent(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let type_class = class(vm, "Type")?;
    let left_is_type = receiver.as_obj() == Some(type_class);
    let right_is_type = args.first().and_then(Value::as_obj) == Some(type_class);
    if left_is_type || right_is_type {
        return Ok(Value::bool(left_is_type && right_is_type));
    }
    let (left_context, left) = context_kind_parts(receiver, vm)?;
    let (_right_context, right) = context_kind_parts(
        args.first().ok_or_else(|| RuntimeError::Arity {
            signature: "equivalentTo(_)",
            expected: 1,
            found: args.len(),
        })?,
        vm,
    )?;
    let context_data = match &vm.heap.typing_object(left_context).payload {
        TypingPayload::Context(data) => data,
        _ => unreachable!(),
    };
    Ok(Value::bool(inspect::kind_equivalent(context_data, &vm.typing_registry, left, right)))
}

fn descriptor_kind(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::obj(class(vm, "Type")?))
}

fn descriptor_display(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let context_data = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => data,
        _ => unreachable!(),
    };
    let rendered = inspect::display(context_data, &vm.typing_registry, &vm.heap, handle);
    Ok(vm.alloc_string_value(rendered))
}

fn descriptor_argument_count(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let context_data = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => data,
        _ => unreachable!(),
    };
    Ok(Value::int(
        inspect::arguments(context_data, &vm.typing_registry, handle).map_or(0, |arguments| arguments.len() as i64),
    ))
}

fn descriptor_remaining_count(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let context_data = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => data,
        _ => unreachable!(),
    };
    Ok(Value::int(
        inspect::remaining_parameter_count(context_data, &vm.typing_registry, &vm.heap, handle) as i64,
    ))
}

fn descriptor_argument_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let index = args.first().and_then(|value| value.as_int()).ok_or_else(|| RuntimeError::Type {
        expected: "Int",
        found: args.first().map_or("missing", Value::type_name),
    })?;
    let child = {
        let context_data = match &vm.heap.typing_object(context).payload {
            TypingPayload::Context(data) => data,
            _ => unreachable!(),
        };
        inspect::arguments(context_data, &vm.typing_registry, handle)
            .and_then(|arguments| arguments.get(index as usize).copied())
            .ok_or_else(|| RuntimeError::Message("type argument index out of range".into()))?
    };
    let descriptor_class = descriptor_class_for_handle(vm, context, child)?;
    crate::typing::reify::reify_type_form(context, child, &vm.typing_registry, &mut vm.heap, descriptor_class)
}

fn descriptor_equivalent(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let (_other_context, other) = descriptor_parts(
        args.first().ok_or_else(|| RuntimeError::Arity {
            signature: "equivalentTo(_)",
            expected: 1,
            found: args.len(),
        })?,
        vm,
    )?;
    let context_data = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => data,
        _ => unreachable!(),
    };
    Ok(Value::bool(inspect::equivalent(context_data, &vm.typing_registry, handle, other)))
}

fn descriptor_subtype(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let equivalent = descriptor_equivalent(vm, receiver, args)?.as_bool().unwrap_or(false);
    alloc_variant(
        vm,
        if equivalent { "RelationSatisfied" } else { "RelationRejected" },
        Some(Value::bool(equivalent)),
    )
}

fn result_value(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "TypingResult",
        found: receiver.type_name(),
    })?;
    vm.heap
        .as_instance(id)
        .and_then(|instance| instance.slots.first().copied())
        .ok_or_else(|| RuntimeError::Message("result variant has no value payload".into()).into())
}
