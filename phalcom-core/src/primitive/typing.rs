//! Native boundary for explicit typing reflection.
//!
//! This module owns only runtime adaptation. Canonical type semantics remain in
//! `phalcom-semantic`/`phalcom-type-meta`; overlay nodes are validated handles,
//! never user-supplied numeric IDs.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{InstanceObject, ObjRef, Object, TypingPayload};
use crate::method::MethodObject;
use crate::primitive::{expect_tuple, primitive, primitive_static};
use crate::typing::capability::{TypingCapabilities, TypingCapability};
use crate::typing::context::TypingContextData;
use crate::typing::handle::{
    RuntimeCallableParameterRef, RuntimeCallableSignatureRef, RuntimeFieldSignatureRef, RuntimeGenericConstraintRef, RuntimeGenericSignatureRef,
    RuntimeKindRef, RuntimeSemanticHandle, RuntimeTypeParameterRef, RuntimeTypeRef, RuntimeTypeUseRef,
};
use crate::typing::inspect;
use crate::typing::overlay::{RuntimeOverlayKindNode, RuntimeOverlayTypeNode, RuntimeRecordField};
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

fn type_param_parts(receiver: &Value, vm: &VM) -> PhResult<(ObjRef, RuntimeTypeParameterRef)> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "TypeParameter",
        found: receiver.type_name(),
    })?;
    match vm.heap.as_typing_object(id).map(|object| &object.payload) {
        Some(TypingPayload::Descriptor {
            context,
            handle: RuntimeSemanticHandle::TypeParameter(handle),
        }) => Ok((*context, *handle)),
        _ => Err(RuntimeError::Type {
            expected: "TypeParameter",
            found: receiver.type_name(),
        }
        .into()),
    }
}

fn generic_sig_parts(receiver: &Value, vm: &VM) -> PhResult<(ObjRef, RuntimeGenericSignatureRef)> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "GenericSignature",
        found: receiver.type_name(),
    })?;
    match vm.heap.as_typing_object(id).map(|object| &object.payload) {
        Some(TypingPayload::Descriptor {
            context,
            handle: RuntimeSemanticHandle::GenericSignature(handle),
        }) => Ok((*context, *handle)),
        _ => Err(RuntimeError::Type {
            expected: "GenericSignature",
            found: receiver.type_name(),
        }
        .into()),
    }
}

fn generic_constraint_parts(receiver: &Value, vm: &VM) -> PhResult<(ObjRef, RuntimeGenericConstraintRef)> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "GenericConstraint",
        found: receiver.type_name(),
    })?;
    match vm.heap.as_typing_object(id).map(|object| &object.payload) {
        Some(TypingPayload::Descriptor {
            context,
            handle: RuntimeSemanticHandle::GenericConstraint(handle),
        }) => Ok((*context, *handle)),
        _ => Err(RuntimeError::Type {
            expected: "GenericConstraint",
            found: receiver.type_name(),
        }
        .into()),
    }
}

fn callable_sig_parts(receiver: &Value, vm: &VM) -> PhResult<(ObjRef, RuntimeCallableSignatureRef)> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "CallableSignature",
        found: receiver.type_name(),
    })?;
    match vm.heap.as_typing_object(id).map(|object| &object.payload) {
        Some(TypingPayload::Descriptor {
            context,
            handle: RuntimeSemanticHandle::CallableSignature(handle),
        }) => Ok((*context, *handle)),
        _ => Err(RuntimeError::Type {
            expected: "CallableSignature",
            found: receiver.type_name(),
        }
        .into()),
    }
}

fn callable_param_parts(receiver: &Value, vm: &VM) -> PhResult<(ObjRef, RuntimeCallableParameterRef)> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "CallableParameter",
        found: receiver.type_name(),
    })?;
    match vm.heap.as_typing_object(id).map(|object| &object.payload) {
        Some(TypingPayload::Descriptor {
            context,
            handle: RuntimeSemanticHandle::CallableParameter(handle),
        }) => Ok((*context, *handle)),
        _ => Err(RuntimeError::Type {
            expected: "CallableParameter",
            found: receiver.type_name(),
        }
        .into()),
    }
}

fn field_sig_parts(receiver: &Value, vm: &VM) -> PhResult<(ObjRef, RuntimeFieldSignatureRef)> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "FieldSignature",
        found: receiver.type_name(),
    })?;
    match vm.heap.as_typing_object(id).map(|object| &object.payload) {
        Some(TypingPayload::Descriptor {
            context,
            handle: RuntimeSemanticHandle::FieldSignature(handle),
        }) => Ok((*context, *handle)),
        _ => Err(RuntimeError::Type {
            expected: "FieldSignature",
            found: receiver.type_name(),
        }
        .into()),
    }
}

fn type_use_parts(receiver: &Value, vm: &VM) -> PhResult<(ObjRef, RuntimeTypeUseRef)> {
    let id = receiver.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "TypeUse",
        found: receiver.type_name(),
    })?;
    match vm.heap.as_typing_object(id).map(|object| &object.payload) {
        Some(TypingPayload::Descriptor {
            context,
            handle: RuntimeSemanticHandle::TypeUse(handle),
        }) => Ok((*context, *handle)),
        _ => Err(RuntimeError::Type {
            expected: "TypeUse",
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

fn alloc_context_with_capabilities(vm: &mut VM, profile: MetadataProfile, capabilities: TypingCapabilities) -> PhResult<Value> {
    let class = class(vm, "TypingContext")?;
    let data = TypingContextData::with_capabilities(Box::new([]), profile, capabilities);
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
        if !instance.slots.is_empty() {
            instance.slots[0] = value;
        }
    }
    Ok(Value::obj(vm.heap.alloc(Object::Instance(instance))))
}

fn alloc_unavailable(vm: &mut VM, reason: &str) -> PhResult<Value> {
    let sym = Value::symbol(vm.get_or_intern(reason));
    alloc_variant(vm, "TypingUnavailable", Some(sym))
}

fn alloc_invalid(vm: &mut VM, reason: &str) -> PhResult<Value> {
    let sym = Value::symbol(vm.get_or_intern(reason));
    alloc_variant(vm, "TypingInvalid", Some(sym))
}

fn alloc_dynamic_boundary(vm: &mut VM, reason: &str) -> PhResult<Value> {
    let sym = Value::symbol(vm.get_or_intern(reason));
    alloc_variant(vm, "RelationDynamicBoundary", Some(sym))
}

fn alloc_member_missing(vm: &mut VM, reason: &str) -> PhResult<Value> {
    let sym = Value::symbol(vm.get_or_intern(reason));
    alloc_variant(vm, "MemberMissing", Some(sym))
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
        RuntimeTypeRef::Overlay(id) => vm
            .heap
            .as_typing_object(context)
            .and_then(|object| match &object.payload {
                TypingPayload::Context(data) => data.overlay.type_node(id).map(|node| match node {
                    RuntimeOverlayTypeNode::Applied { .. } => "AppliedType",
                    RuntimeOverlayTypeNode::Union(_) => "UnionType",
                    RuntimeOverlayTypeNode::Tuple(_) => "TupleType",
                    RuntimeOverlayTypeNode::Record(_) => "RecordType",
                    RuntimeOverlayTypeNode::Callable { .. } => "CallableType",
                    RuntimeOverlayTypeNode::TypeLambda { .. } => "TypeLambda",
                    RuntimeOverlayTypeNode::Special(_) => "SpecialType",
                    RuntimeOverlayTypeNode::SelfType(_) => "SelfType",
                    RuntimeOverlayTypeNode::Nominal { .. } => "TypeDescriptor",
                }),
                TypingPayload::Descriptor { .. } => None,
            })
            .unwrap_or("TypeDescriptor"),
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

fn reify_kind(vm: &mut VM, context: ObjRef, handle: RuntimeKindRef) -> PhResult<Value> {
    let type_class = class(vm, "Type")?;
    let function_kind_class = class(vm, "FunctionKind")?;
    crate::typing::reify::reify_kind_form(context, handle, &vm.typing_registry, &mut vm.heap, type_class, function_kind_class)
}

fn reify_type_parameter_obj(vm: &mut VM, context: ObjRef, handle: RuntimeTypeParameterRef) -> PhResult<Value> {
    let type_param_class = class(vm, "TypeParameter")?;
    crate::typing::reify::reify_type_parameter(context, handle, &mut vm.heap, type_param_class)
}

fn reify_generic_signature_obj(vm: &mut VM, context: ObjRef, handle: RuntimeGenericSignatureRef) -> PhResult<Value> {
    let generic_sig_class = class(vm, "GenericSignature")?;
    crate::typing::reify::reify_generic_signature(context, handle, &mut vm.heap, generic_sig_class)
}

fn reify_generic_constraint_obj(vm: &mut VM, context: ObjRef, handle: RuntimeGenericConstraintRef) -> PhResult<Value> {
    let constraint_class = class(vm, "GenericConstraint")?;
    crate::typing::reify::reify_generic_constraint(context, handle, &mut vm.heap, constraint_class)
}

fn reify_callable_signature_obj(vm: &mut VM, context: ObjRef, handle: RuntimeCallableSignatureRef) -> PhResult<Value> {
    let callable_sig_class = class(vm, "CallableSignature")?;
    crate::typing::reify::reify_callable_signature(context, handle, &mut vm.heap, callable_sig_class)
}

fn reify_callable_parameter_obj(vm: &mut VM, context: ObjRef, handle: RuntimeCallableParameterRef) -> PhResult<Value> {
    let callable_param_class = class(vm, "CallableParameter")?;
    crate::typing::reify::reify_callable_parameter(context, handle, &mut vm.heap, callable_param_class)
}

pub fn install(vm: &mut VM) {
    let behavior = vm.universe.classes.behavior_class;
    primitive!(vm, behavior, "kind", crate::method::SignatureKind::Getter, behavior_kind);
    primitive!(vm, behavior, "display", crate::method::SignatureKind::Getter, behavior_display);
    primitive!(vm, behavior, "freeParameterCount", crate::method::SignatureKind::Getter, zero_count);
    primitive!(vm, behavior, "freeParameterAt", crate::method::SignatureKind::Method(1), zero_count);
    primitive!(vm, behavior, "freeParameters", crate::method::SignatureKind::Getter, empty_tuple);
    primitive!(
        vm,
        behavior,
        "remainingParameterCount",
        crate::method::SignatureKind::Getter,
        behavior_remaining_count
    );
    primitive!(
        vm,
        behavior,
        "remainingParameterAt",
        crate::method::SignatureKind::Method(1),
        behavior_remaining_parameter_at
    );
    primitive!(
        vm,
        behavior,
        "remainingParameters",
        crate::method::SignatureKind::Getter,
        behavior_remaining_parameters
    );
    primitive!(
        vm,
        behavior,
        "genericSignature",
        crate::method::SignatureKind::Getter,
        behavior_generic_signature
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
    primitive!(vm, descriptor, "freeParameterAt", crate::method::SignatureKind::Method(1), zero_count);
    primitive!(vm, descriptor, "freeParameters", crate::method::SignatureKind::Getter, empty_tuple);
    primitive!(
        vm,
        descriptor,
        "remainingParameterCount",
        crate::method::SignatureKind::Getter,
        descriptor_remaining_count
    );
    primitive!(
        vm,
        descriptor,
        "remainingParameterAt",
        crate::method::SignatureKind::Method(1),
        descriptor_remaining_parameter_at
    );
    primitive!(
        vm,
        descriptor,
        "remainingParameters",
        crate::method::SignatureKind::Getter,
        descriptor_remaining_parameters
    );
    primitive!(vm, descriptor, "argumentCount", crate::method::SignatureKind::Getter, descriptor_argument_count);
    primitive!(vm, descriptor, "argumentAt", crate::method::SignatureKind::Method(1), descriptor_argument_at);
    primitive!(vm, descriptor, "arguments", crate::method::SignatureKind::Getter, descriptor_arguments);
    primitive!(vm, descriptor, "origin", crate::method::SignatureKind::Getter, descriptor_origin);
    primitive!(
        vm,
        descriptor,
        "parameterCount",
        crate::method::SignatureKind::Getter,
        descriptor_parameter_count
    );
    primitive!(vm, descriptor, "parameterAt", crate::method::SignatureKind::Method(1), descriptor_parameter_at);
    primitive!(vm, descriptor, "parameters", crate::method::SignatureKind::Getter, descriptor_parameters);
    primitive!(vm, descriptor, "returnType", crate::method::SignatureKind::Getter, descriptor_return_type);
    primitive!(vm, descriptor, "fieldCount", crate::method::SignatureKind::Getter, descriptor_field_count);
    primitive!(vm, descriptor, "fieldNameAt", crate::method::SignatureKind::Method(1), descriptor_field_name_at);
    primitive!(vm, descriptor, "fieldTypeAt", crate::method::SignatureKind::Method(1), descriptor_field_type_at);
    primitive!(vm, descriptor, "fields", crate::method::SignatureKind::Getter, descriptor_fields);
    primitive!(vm, descriptor, "body", crate::method::SignatureKind::Getter, descriptor_body);
    primitive!(vm, descriptor, "equivalentTo", crate::method::SignatureKind::Method(1), descriptor_equivalent);
    primitive!(vm, descriptor, "subtypeOf", crate::method::SignatureKind::Method(1), descriptor_subtype);
    primitive!(vm, descriptor, "apply", crate::method::SignatureKind::Method(1), descriptor_apply);

    let type_param = vm.universe.typing_classes.get("TypeParameter").expect("TypeParameter");
    primitive!(vm, type_param, "owner", crate::method::SignatureKind::Getter, type_param_owner);
    primitive!(vm, type_param, "index", crate::method::SignatureKind::Getter, type_param_index);
    primitive!(vm, type_param, "name", crate::method::SignatureKind::Getter, type_param_name);
    primitive!(vm, type_param, "kind", crate::method::SignatureKind::Getter, type_param_kind);
    primitive!(vm, type_param, "variance", crate::method::SignatureKind::Getter, type_param_variance);
    primitive!(
        vm,
        type_param,
        "constraintCount",
        crate::method::SignatureKind::Getter,
        type_param_constraint_count
    );
    primitive!(
        vm,
        type_param,
        "constraintAt",
        crate::method::SignatureKind::Method(1),
        type_param_constraint_at
    );
    primitive!(vm, type_param, "constraints", crate::method::SignatureKind::Getter, type_param_constraints);

    let generic_sig = vm.universe.typing_classes.get("GenericSignature").expect("GenericSignature");
    primitive!(vm, generic_sig, "owner", crate::method::SignatureKind::Getter, generic_sig_owner);
    primitive!(
        vm,
        generic_sig,
        "parameterCount",
        crate::method::SignatureKind::Getter,
        generic_sig_parameter_count
    );
    primitive!(
        vm,
        generic_sig,
        "parameterAt",
        crate::method::SignatureKind::Method(1),
        generic_sig_parameter_at
    );
    primitive!(vm, generic_sig, "parameters", crate::method::SignatureKind::Getter, generic_sig_parameters);
    primitive!(
        vm,
        generic_sig,
        "constraintCount",
        crate::method::SignatureKind::Getter,
        generic_sig_constraint_count
    );
    primitive!(
        vm,
        generic_sig,
        "constraintAt",
        crate::method::SignatureKind::Method(1),
        generic_sig_constraint_at
    );
    primitive!(vm, generic_sig, "constraints", crate::method::SignatureKind::Getter, generic_sig_constraints);

    let constraint = vm.universe.typing_classes.get("GenericConstraint").expect("GenericConstraint");
    primitive!(vm, constraint, "relation", crate::method::SignatureKind::Getter, generic_constraint_relation);
    primitive!(vm, constraint, "left", crate::method::SignatureKind::Getter, generic_constraint_left);
    primitive!(vm, constraint, "right", crate::method::SignatureKind::Getter, generic_constraint_right);
    primitive!(vm, constraint, "source", crate::method::SignatureKind::Getter, generic_constraint_source);

    let callable_sig = vm.universe.typing_classes.get("CallableSignature").expect("CallableSignature");
    primitive!(vm, callable_sig, "owner", crate::method::SignatureKind::Getter, callable_sig_owner);
    primitive!(vm, callable_sig, "side", crate::method::SignatureKind::Getter, callable_sig_side);
    primitive!(vm, callable_sig, "selector", crate::method::SignatureKind::Getter, callable_sig_selector);
    primitive!(
        vm,
        callable_sig,
        "genericSignature",
        crate::method::SignatureKind::Getter,
        callable_sig_generic_signature
    );
    primitive!(
        vm,
        callable_sig,
        "parameterCount",
        crate::method::SignatureKind::Getter,
        callable_sig_parameter_count
    );
    primitive!(
        vm,
        callable_sig,
        "parameterAt",
        crate::method::SignatureKind::Method(1),
        callable_sig_parameter_at
    );
    primitive!(
        vm,
        callable_sig,
        "parameterTypeAt",
        crate::method::SignatureKind::Method(1),
        callable_sig_parameter_type_at
    );
    primitive!(vm, callable_sig, "parameters", crate::method::SignatureKind::Getter, callable_sig_parameters);
    primitive!(vm, callable_sig, "returnType", crate::method::SignatureKind::Getter, callable_sig_return_type);
    primitive!(vm, callable_sig, "source", crate::method::SignatureKind::Getter, callable_sig_source);
    primitive!(
        vm,
        callable_sig,
        "documentation",
        crate::method::SignatureKind::Getter,
        callable_sig_documentation
    );

    let callable_param = vm.universe.typing_classes.get("CallableParameter").expect("CallableParameter");
    primitive!(vm, callable_param, "index", crate::method::SignatureKind::Getter, callable_param_index);
    primitive!(vm, callable_param, "localName", crate::method::SignatureKind::Getter, callable_param_local_name);
    primitive!(
        vm,
        callable_param,
        "externalLabel",
        crate::method::SignatureKind::Getter,
        callable_param_external_label
    );
    primitive!(vm, callable_param, "restMode", crate::method::SignatureKind::Getter, callable_param_rest_mode);
    primitive!(vm, callable_param, "type", crate::method::SignatureKind::Getter, callable_param_type);
    primitive!(vm, callable_param, "source", crate::method::SignatureKind::Getter, callable_param_source);

    let field_sig = vm.universe.typing_classes.get("FieldSignature").expect("FieldSignature");
    primitive!(vm, field_sig, "owner", crate::method::SignatureKind::Getter, field_sig_owner);
    primitive!(vm, field_sig, "side", crate::method::SignatureKind::Getter, field_sig_side);
    primitive!(vm, field_sig, "name", crate::method::SignatureKind::Getter, field_sig_name);
    primitive!(vm, field_sig, "mutable", crate::method::SignatureKind::Getter, field_sig_mutable);
    primitive!(vm, field_sig, "type", crate::method::SignatureKind::Getter, field_sig_type);
    primitive!(vm, field_sig, "source", crate::method::SignatureKind::Getter, field_sig_source);

    let type_use = vm.universe.typing_classes.get("TypeUse").expect("TypeUse");
    primitive!(vm, type_use, "valueType", crate::method::SignatureKind::Getter, type_use_value_type);
    primitive!(vm, type_use, "denotation", crate::method::SignatureKind::Getter, type_use_denotation);
    primitive!(vm, type_use, "source", crate::method::SignatureKind::Getter, type_use_source);
    primitive!(vm, type_use, "spelling", crate::method::SignatureKind::Getter, type_use_spelling);
    primitive!(vm, type_use, "evidence", crate::method::SignatureKind::Getter, type_use_evidence);
    primitive!(vm, type_use, "inference", crate::method::SignatureKind::Getter, type_use_inference);
    primitive!(vm, type_use, "constant", crate::method::SignatureKind::Getter, type_use_constant);

    let typing = vm.universe.typing_classes.get("Typing").expect("Typing");
    primitive_static!(vm, typing, "current", crate::method::SignatureKind::Getter, typing_current);
    primitive_static!(vm, typing, "contextFor", crate::method::SignatureKind::Method(1), typing_context_for);

    let context = vm.universe.typing_classes.get("TypingContext").expect("TypingContext");
    primitive_static!(vm, context, "new", crate::method::SignatureKind::Method(0), typing_context_new);
    primitive!(vm, context, "profile", crate::method::SignatureKind::Getter, typing_context_profile);
    primitive!(vm, context, "capabilities", crate::method::SignatureKind::Getter, typing_context_capabilities);
    primitive!(vm, context, "restrictTo", crate::method::SignatureKind::Method(1), typing_context_restrict);
    primitive!(vm, context, "refresh", crate::method::SignatureKind::Getter, typing_context_refresh);
    primitive!(
        vm,
        context,
        "semanticModel",
        crate::method::SignatureKind::Getter,
        typing_context_semantic_model
    );
    primitive!(vm, context, "snapshot", crate::method::SignatureKind::Getter, typing_context_snapshot);
    primitive!(vm, context, "world", crate::method::SignatureKind::Getter, typing_context_world);
    primitive!(
        vm,
        context,
        "typeOfDeclaration",
        crate::method::SignatureKind::Method(1),
        typing_context_type_of_declaration
    );
    primitive!(
        vm,
        context,
        "genericSignatureOf",
        crate::method::SignatureKind::Method(1),
        typing_context_generic_signature_of
    );
    primitive!(
        vm,
        context,
        "declaredSupertypeOf",
        crate::method::SignatureKind::Method(1),
        typing_context_declared_supertype_of
    );
    primitive!(vm, context, "signatureOf", crate::method::SignatureKind::Method(1), typing_context_signature_of);
    primitive!(vm, context, "apply", crate::method::SignatureKind::Method(2), typing_context_apply);
    primitive!(vm, context, "unionOf", crate::method::SignatureKind::Method(1), typing_context_union_of);
    primitive!(vm, context, "tupleOf", crate::method::SignatureKind::Method(1), typing_context_tuple_of);
    primitive!(vm, context, "recordOf", crate::method::SignatureKind::Method(1), typing_context_record_of);
    primitive!(vm, context, "callable", crate::method::SignatureKind::Method(2), typing_context_callable);
    primitive!(vm, context, "equivalent", crate::method::SignatureKind::Method(2), typing_context_equivalent);
    primitive!(vm, context, "subtype", crate::method::SignatureKind::Method(2), typing_context_subtype);
    primitive!(vm, context, "assignable", crate::method::SignatureKind::Method(2), typing_context_assignable);
    primitive!(vm, context, "consistent", crate::method::SignatureKind::Method(2), typing_context_consistent);
    primitive!(vm, context, "conforms", crate::method::SignatureKind::Method(2), typing_context_conforms);
    primitive!(vm, context, "member", crate::method::SignatureKind::Method(4), typing_context_member);
    primitive!(vm, context, "typeUseAt", crate::method::SignatureKind::Method(2), typing_context_type_use_at);
    primitive!(vm, context, "typeUsesOf", crate::method::SignatureKind::Method(1), typing_context_type_uses_of);
    primitive!(vm, context, "matches", crate::method::SignatureKind::Method(2), typing_context_matches);
    primitive!(vm, context, "validate", crate::method::SignatureKind::Method(2), typing_context_validate);
    primitive!(vm, context, "construct", crate::method::SignatureKind::Method(2), typing_context_construct);
    primitive!(vm, context, "proofsOf", crate::method::SignatureKind::Method(1), typing_context_proofs_of);

    for name in [
        "TypingKnown",
        "TypingUnknown",
        "TypingInvalid",
        "TypingUnavailable",
        "TypingBudgetExceeded",
        "TypingInternalFailure",
        "RelationSatisfied",
        "RelationRejected",
        "RelationDynamicBoundary",
        "RelationBlocked",
        "RelationBudgetExceeded",
        "RelationInternalFailure",
        "MemberFound",
        "MemberMissing",
        "MemberDynamicBoundary",
        "MemberBlocked",
        "MemberBudgetExceeded",
        "MemberInternalFailure",
    ] {
        let result_class = vm.universe.typing_classes.get(name).expect("typing result class");
        primitive!(vm, result_class, "value", crate::method::SignatureKind::Getter, result_value);
    }

    for name in ["TypingCancelled", "RelationCancelled", "MemberCancelled"] {
        let result_class = vm.universe.typing_classes.get(name).expect("typing cancelled class");
        primitive!(vm, result_class, "value", crate::method::SignatureKind::Getter, result_cancelled);
    }
}

// ---------------------------------------------------------------------------
// Helpers & getters
// ---------------------------------------------------------------------------

fn empty_tuple(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::obj(vm.heap.alloc(Object::Tuple(crate::heap::TupleObject::positional(Vec::new())))))
}

fn zero_count(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::int(0))
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

fn result_cancelled(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::none())
}

// ---------------------------------------------------------------------------
// Behavior methods
// ---------------------------------------------------------------------------

fn behavior_generic_arity(vm: &VM, class_id: crate::heap::ClassId) -> usize {
    if let Some(signature) = inspect::generic_signature_of_declaration(&vm.typing_registry, class_id, &vm.heap) {
        return inspect::generic_sig_parameters(&vm.typing_registry, signature).len();
    }

    let class_name = vm.heap.class(class_id).name.as_str();
    for spec in phalcom_native_meta::universe::UNIVERSE_TYPE_FORMS {
        if spec.owner.name() == class_name {
            return spec.parameters.len();
        }
    }
    0
}

fn behavior_kind(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = receiver
        .as_obj()
        .filter(|id| vm.heap.as_class(*id).is_some())
        .ok_or_else(|| RuntimeError::Type {
            expected: "Class",
            found: receiver.type_name(),
        })?;
    let parameter_count = behavior_generic_arity(vm, class_id);
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

fn behavior_remaining_count(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = receiver
        .as_obj()
        .filter(|id| vm.heap.as_class(*id).is_some())
        .ok_or_else(|| RuntimeError::Type {
            expected: "Class",
            found: receiver.type_name(),
        })?;
    let count = behavior_generic_arity(vm, class_id);
    Ok(Value::int(count as i64))
}

fn behavior_remaining_parameter_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let class_id = receiver
        .as_obj()
        .filter(|id| vm.heap.as_class(*id).is_some())
        .ok_or_else(|| RuntimeError::Type {
            expected: "Class",
            found: receiver.type_name(),
        })?;
    let index = args.first().and_then(|v| v.as_int()).ok_or_else(|| RuntimeError::Type {
        expected: "Int",
        found: args.first().map_or("missing", Value::type_name),
    })?;
    let context = alloc_context(vm, MetadataProfile::RuntimePublic)?.as_obj().expect("context obj");
    let sig_opt = inspect::generic_signature_of_declaration(&vm.typing_registry, class_id, &vm.heap);
    if let Some(sig) = sig_opt {
        let params = inspect::generic_sig_parameters(&vm.typing_registry, sig);
        if let Some(&p) = params.get(index as usize) {
            return reify_type_parameter_obj(vm, context, p);
        }
    }
    Err(RuntimeError::Message("remaining parameter index out of range".into()).into())
}

fn behavior_remaining_parameters(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = receiver
        .as_obj()
        .filter(|id| vm.heap.as_class(*id).is_some())
        .ok_or_else(|| RuntimeError::Type {
            expected: "Class",
            found: receiver.type_name(),
        })?;
    let context = alloc_context(vm, MetadataProfile::RuntimePublic)?.as_obj().expect("context obj");
    let sig_opt = inspect::generic_signature_of_declaration(&vm.typing_registry, class_id, &vm.heap);
    let mut values = Vec::new();
    if let Some(sig) = sig_opt {
        let params = inspect::generic_sig_parameters(&vm.typing_registry, sig);
        for p in params {
            values.push(reify_type_parameter_obj(vm, context, p)?);
        }
    }
    Ok(Value::obj(vm.heap.alloc(Object::Tuple(crate::heap::TupleObject::positional(values)))))
}

fn behavior_generic_signature(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = receiver
        .as_obj()
        .filter(|id| vm.heap.as_class(*id).is_some())
        .ok_or_else(|| RuntimeError::Type {
            expected: "Class",
            found: receiver.type_name(),
        })?;
    let context = alloc_context(vm, MetadataProfile::RuntimePublic)?.as_obj().expect("context obj");
    if let Some(sig) = inspect::generic_signature_of_declaration(&vm.typing_registry, class_id, &vm.heap) {
        let val = reify_generic_signature_obj(vm, context, sig)?;
        Ok(val.wrap_some()?)
    } else {
        Ok(Value::none())
    }
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

// ---------------------------------------------------------------------------
// KindDescriptor methods
// ---------------------------------------------------------------------------

fn kind_display(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if receiver.as_obj() == Some(class(vm, "Type")?) {
        return Ok(vm.alloc_string_value("Type".to_string()));
    }
    let (context, handle) = context_kind_parts(receiver, vm)?;
    let rendered = {
        let context_data = match &vm.heap.typing_object(context).payload {
            TypingPayload::Context(data) => data,
            _ => unreachable!(),
        };
        inspect::kind_display(context_data, &vm.typing_registry, handle)
    };
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
        args.first().ok_or(RuntimeError::Arity {
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

// ---------------------------------------------------------------------------
// TypeDescriptor methods
// ---------------------------------------------------------------------------

fn descriptor_kind(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::obj(class(vm, "Type")?))
}

fn descriptor_display(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let rendered = {
        let context_data = match &vm.heap.typing_object(context).payload {
            TypingPayload::Context(data) => data,
            _ => unreachable!(),
        };
        inspect::display(context_data, &vm.typing_registry, &vm.heap, handle)
    };
    Ok(vm.alloc_string_value(rendered))
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

fn descriptor_remaining_parameter_at(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::Message("descriptor has no remaining parameters".into()).into())
}

fn descriptor_remaining_parameters(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::obj(vm.heap.alloc(Object::Tuple(crate::heap::TupleObject::positional(Vec::new())))))
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

fn descriptor_arguments(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let args_handles = {
        let context_data = match &vm.heap.typing_object(context).payload {
            TypingPayload::Context(data) => data,
            _ => unreachable!(),
        };
        inspect::arguments(context_data, &vm.typing_registry, handle).unwrap_or_default()
    };
    let mut values = Vec::with_capacity(args_handles.len());
    for arg_handle in args_handles {
        let desc_class = descriptor_class_for_handle(vm, context, arg_handle)?;
        values.push(crate::typing::reify::reify_type_form(
            context,
            arg_handle,
            &vm.typing_registry,
            &mut vm.heap,
            desc_class,
        )?);
    }
    Ok(Value::obj(vm.heap.alloc(Object::Tuple(crate::heap::TupleObject::positional(values)))))
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

fn descriptor_origin(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let origin_handle = match handle {
        RuntimeTypeRef::Overlay(id) => {
            let context_data = match &vm.heap.typing_object(context).payload {
                TypingPayload::Context(data) => data,
                _ => unreachable!(),
            };
            match context_data.overlay.type_node(id) {
                Some(RuntimeOverlayTypeNode::Applied { origin, .. }) => *origin,
                _ => return Err(RuntimeError::Message("not an applied type".into()).into()),
            }
        }
        RuntimeTypeRef::Base { pool, node } => {
            let loaded = vm
                .typing_registry
                .get_pool(pool)
                .ok_or_else(|| RuntimeError::Internal("pool not found".into()))?;
            let entry = loaded
                .bundle
                .types
                .get(node.0 as usize)
                .ok_or_else(|| RuntimeError::Internal("node not found".into()))?;
            match &entry.form {
                phalcom_type_meta::type_node::TypeNode::Applied { origin, .. } => RuntimeTypeRef::Base { pool, node: *origin },
                _ => return Err(RuntimeError::Message("not an applied type".into()).into()),
            }
        }
    };
    let desc_class = descriptor_class_for_handle(vm, context, origin_handle)?;
    crate::typing::reify::reify_type_form(context, origin_handle, &vm.typing_registry, &mut vm.heap, desc_class)
}

fn descriptor_parameter_count(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let count = match handle {
        RuntimeTypeRef::Overlay(id) => {
            let context_data = match &vm.heap.typing_object(context).payload {
                TypingPayload::Context(data) => data,
                _ => unreachable!(),
            };
            match context_data.overlay.type_node(id) {
                Some(RuntimeOverlayTypeNode::Callable { parameters, .. }) => parameters.len(),
                Some(RuntimeOverlayTypeNode::TypeLambda { parameters, .. }) => parameters.len(),
                _ => 0,
            }
        }
        RuntimeTypeRef::Base { pool, node } => {
            let loaded = vm
                .typing_registry
                .get_pool(pool)
                .ok_or_else(|| RuntimeError::Internal("pool not found".into()))?;
            let entry = loaded
                .bundle
                .types
                .get(node.0 as usize)
                .ok_or_else(|| RuntimeError::Internal("node not found".into()))?;
            match &entry.form {
                phalcom_type_meta::type_node::TypeNode::Callable(c) => c.parameters.len(),
                _ => 0,
            }
        }
    };
    Ok(Value::int(count as i64))
}

fn descriptor_parameter_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let index = args.first().and_then(|v| v.as_int()).ok_or_else(|| RuntimeError::Type {
        expected: "Int",
        found: args.first().map_or("missing", Value::type_name),
    })? as usize;

    match handle {
        RuntimeTypeRef::Overlay(id) => {
            let param_handle = {
                let context_data = match &vm.heap.typing_object(context).payload {
                    TypingPayload::Context(data) => data,
                    _ => unreachable!(),
                };
                match context_data.overlay.type_node(id) {
                    Some(RuntimeOverlayTypeNode::Callable { parameters, .. }) => parameters.get(index).map(|p| p.ty),
                    _ => None,
                }
            }
            .ok_or_else(|| RuntimeError::Message("parameter index out of range".into()))?;
            let desc_class = descriptor_class_for_handle(vm, context, param_handle)?;
            crate::typing::reify::reify_type_form(context, param_handle, &vm.typing_registry, &mut vm.heap, desc_class)
        }
        RuntimeTypeRef::Base { pool, node } => {
            let loaded = vm
                .typing_registry
                .get_pool(pool)
                .ok_or_else(|| RuntimeError::Internal("pool not found".into()))?;
            let entry = loaded
                .bundle
                .types
                .get(node.0 as usize)
                .ok_or_else(|| RuntimeError::Internal("node not found".into()))?;
            match &entry.form {
                phalcom_type_meta::type_node::TypeNode::Callable(c) => {
                    let p = c
                        .parameters
                        .get(index)
                        .ok_or_else(|| RuntimeError::Message("parameter index out of range".into()))?;
                    let param_handle = RuntimeTypeRef::Base { pool, node: p.ty };
                    let desc_class = descriptor_class_for_handle(vm, context, param_handle)?;
                    crate::typing::reify::reify_type_form(context, param_handle, &vm.typing_registry, &mut vm.heap, desc_class)
                }
                _ => Err(RuntimeError::Message("not a callable type".into()).into()),
            }
        }
    }
}

fn descriptor_parameters(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let param_handles: Vec<RuntimeTypeRef> = match handle {
        RuntimeTypeRef::Overlay(id) => {
            let context_data = match &vm.heap.typing_object(context).payload {
                TypingPayload::Context(data) => data,
                _ => unreachable!(),
            };
            match context_data.overlay.type_node(id) {
                Some(RuntimeOverlayTypeNode::Callable { parameters, .. }) => parameters.iter().map(|p| p.ty).collect(),
                _ => Vec::new(),
            }
        }
        RuntimeTypeRef::Base { pool, node } => {
            let loaded = vm
                .typing_registry
                .get_pool(pool)
                .ok_or_else(|| RuntimeError::Internal("pool not found".into()))?;
            let entry = loaded
                .bundle
                .types
                .get(node.0 as usize)
                .ok_or_else(|| RuntimeError::Internal("node not found".into()))?;
            match &entry.form {
                phalcom_type_meta::type_node::TypeNode::Callable(c) => c.parameters.iter().map(|p| RuntimeTypeRef::Base { pool, node: p.ty }).collect(),
                _ => Vec::new(),
            }
        }
    };
    let mut values = Vec::with_capacity(param_handles.len());
    for p in param_handles {
        let desc_class = descriptor_class_for_handle(vm, context, p)?;
        values.push(crate::typing::reify::reify_type_form(
            context,
            p,
            &vm.typing_registry,
            &mut vm.heap,
            desc_class,
        )?);
    }
    Ok(Value::obj(vm.heap.alloc(Object::Tuple(crate::heap::TupleObject::positional(values)))))
}

fn descriptor_return_type(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let ret_handle = match handle {
        RuntimeTypeRef::Overlay(id) => {
            let context_data = match &vm.heap.typing_object(context).payload {
                TypingPayload::Context(data) => data,
                _ => unreachable!(),
            };
            match context_data.overlay.type_node(id) {
                Some(RuntimeOverlayTypeNode::Callable { return_type, .. }) => *return_type,
                _ => return Err(RuntimeError::Message("not a callable type".into()).into()),
            }
        }
        RuntimeTypeRef::Base { pool, node } => {
            let loaded = vm
                .typing_registry
                .get_pool(pool)
                .ok_or_else(|| RuntimeError::Internal("pool not found".into()))?;
            let entry = loaded
                .bundle
                .types
                .get(node.0 as usize)
                .ok_or_else(|| RuntimeError::Internal("node not found".into()))?;
            match &entry.form {
                phalcom_type_meta::type_node::TypeNode::Callable(c) => RuntimeTypeRef::Base { pool, node: c.return_type },
                _ => return Err(RuntimeError::Message("not a callable type".into()).into()),
            }
        }
    };
    let desc_class = descriptor_class_for_handle(vm, context, ret_handle)?;
    crate::typing::reify::reify_type_form(context, ret_handle, &vm.typing_registry, &mut vm.heap, desc_class)
}

fn descriptor_field_count(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let count = match handle {
        RuntimeTypeRef::Overlay(id) => {
            let context_data = match &vm.heap.typing_object(context).payload {
                TypingPayload::Context(data) => data,
                _ => unreachable!(),
            };
            match context_data.overlay.type_node(id) {
                Some(RuntimeOverlayTypeNode::Record(fields)) => fields.len(),
                _ => 0,
            }
        }
        RuntimeTypeRef::Base { pool, node } => {
            let loaded = vm
                .typing_registry
                .get_pool(pool)
                .ok_or_else(|| RuntimeError::Internal("pool not found".into()))?;
            let entry = loaded
                .bundle
                .types
                .get(node.0 as usize)
                .ok_or_else(|| RuntimeError::Internal("node not found".into()))?;
            match &entry.form {
                phalcom_type_meta::type_node::TypeNode::Record(f) => f.len(),
                phalcom_type_meta::type_node::TypeNode::OpenRecord(o) => o.fields.len(),
                _ => 0,
            }
        }
    };
    Ok(Value::int(count as i64))
}

fn descriptor_field_name_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let index = args.first().and_then(|v| v.as_int()).ok_or_else(|| RuntimeError::Type {
        expected: "Int",
        found: args.first().map_or("missing", Value::type_name),
    })? as usize;

    let name = match handle {
        RuntimeTypeRef::Overlay(id) => {
            let context_data = match &vm.heap.typing_object(context).payload {
                TypingPayload::Context(data) => data,
                _ => unreachable!(),
            };
            match context_data.overlay.type_node(id) {
                Some(RuntimeOverlayTypeNode::Record(fields)) => fields.get(index).map(|f| f.name.to_string()),
                _ => None,
            }
        }
        RuntimeTypeRef::Base { pool, node } => {
            let loaded = vm
                .typing_registry
                .get_pool(pool)
                .ok_or_else(|| RuntimeError::Internal("pool not found".into()))?;
            let entry = loaded
                .bundle
                .types
                .get(node.0 as usize)
                .ok_or_else(|| RuntimeError::Internal("node not found".into()))?;
            match &entry.form {
                phalcom_type_meta::type_node::TypeNode::Record(f) => f.get(index).map(|f| f.name.to_string()),
                phalcom_type_meta::type_node::TypeNode::OpenRecord(o) => o.fields.get(index).map(|f| f.name.to_string()),
                _ => None,
            }
        }
    }
    .ok_or_else(|| RuntimeError::Message("field index out of range".into()))?;

    let sym = vm.get_or_intern(&name);
    Ok(Value::symbol(sym))
}

fn descriptor_field_type_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let index = args.first().and_then(|v| v.as_int()).ok_or_else(|| RuntimeError::Type {
        expected: "Int",
        found: args.first().map_or("missing", Value::type_name),
    })? as usize;

    let field_handle = match handle {
        RuntimeTypeRef::Overlay(id) => {
            let context_data = match &vm.heap.typing_object(context).payload {
                TypingPayload::Context(data) => data,
                _ => unreachable!(),
            };
            match context_data.overlay.type_node(id) {
                Some(RuntimeOverlayTypeNode::Record(fields)) => fields.get(index).map(|f| f.ty),
                _ => None,
            }
        }
        RuntimeTypeRef::Base { pool, node } => {
            let loaded = vm
                .typing_registry
                .get_pool(pool)
                .ok_or_else(|| RuntimeError::Internal("pool not found".into()))?;
            let entry = loaded
                .bundle
                .types
                .get(node.0 as usize)
                .ok_or_else(|| RuntimeError::Internal("node not found".into()))?;
            match &entry.form {
                phalcom_type_meta::type_node::TypeNode::Record(f) => f.get(index).map(|f| RuntimeTypeRef::Base { pool, node: f.ty }),
                phalcom_type_meta::type_node::TypeNode::OpenRecord(o) => o.fields.get(index).map(|f| RuntimeTypeRef::Base { pool, node: f.ty }),
                _ => None,
            }
        }
    }
    .ok_or_else(|| RuntimeError::Message("field index out of range".into()))?;

    let desc_class = descriptor_class_for_handle(vm, context, field_handle)?;
    crate::typing::reify::reify_type_form(context, field_handle, &vm.typing_registry, &mut vm.heap, desc_class)
}

fn descriptor_fields(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let field_handles: Vec<(String, RuntimeTypeRef)> = match handle {
        RuntimeTypeRef::Overlay(id) => {
            let context_data = match &vm.heap.typing_object(context).payload {
                TypingPayload::Context(data) => data,
                _ => unreachable!(),
            };
            match context_data.overlay.type_node(id) {
                Some(RuntimeOverlayTypeNode::Record(fields)) => fields.iter().map(|f| (f.name.to_string(), f.ty)).collect(),
                _ => Vec::new(),
            }
        }
        RuntimeTypeRef::Base { pool, node } => {
            let loaded = vm
                .typing_registry
                .get_pool(pool)
                .ok_or_else(|| RuntimeError::Internal("pool not found".into()))?;
            let entry = loaded
                .bundle
                .types
                .get(node.0 as usize)
                .ok_or_else(|| RuntimeError::Internal("node not found".into()))?;
            match &entry.form {
                phalcom_type_meta::type_node::TypeNode::Record(f) => {
                    f.iter().map(|f| (f.name.to_string(), RuntimeTypeRef::Base { pool, node: f.ty })).collect()
                }
                phalcom_type_meta::type_node::TypeNode::OpenRecord(o) => o
                    .fields
                    .iter()
                    .map(|f| (f.name.to_string(), RuntimeTypeRef::Base { pool, node: f.ty }))
                    .collect(),
                _ => Vec::new(),
            }
        }
    };
    let mut values = Vec::with_capacity(field_handles.len());
    for (name, ty_handle) in field_handles {
        let desc_class = descriptor_class_for_handle(vm, context, ty_handle)?;
        let ty_val = crate::typing::reify::reify_type_form(context, ty_handle, &vm.typing_registry, &mut vm.heap, desc_class)?;
        let name_sym = vm.get_or_intern(&name);
        let pair = vm
            .heap
            .alloc(Object::Tuple(crate::heap::TupleObject::positional(vec![Value::symbol(name_sym), ty_val])));
        values.push(Value::obj(pair));
    }
    Ok(Value::obj(vm.heap.alloc(Object::Tuple(crate::heap::TupleObject::positional(values)))))
}

fn descriptor_body(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let body_handle = match handle {
        RuntimeTypeRef::Overlay(id) => {
            let context_data = match &vm.heap.typing_object(context).payload {
                TypingPayload::Context(data) => data,
                _ => unreachable!(),
            };
            match context_data.overlay.type_node(id) {
                Some(RuntimeOverlayTypeNode::TypeLambda { body, .. }) => *body,
                _ => return Err(RuntimeError::Message("not a type lambda".into()).into()),
            }
        }
        RuntimeTypeRef::Base { .. } => return Err(RuntimeError::Message("not a type lambda".into()).into()),
    };
    let desc_class = descriptor_class_for_handle(vm, context, body_handle)?;
    crate::typing::reify::reify_type_form(context, body_handle, &vm.typing_registry, &mut vm.heap, desc_class)
}

fn descriptor_apply(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let (context, _handle) = descriptor_parts(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::ConstructTypeForms)?;
    let arguments = args.first().ok_or(RuntimeError::Arity {
        signature: "apply(_)",
        expected: 1,
        found: args.len(),
    })?;
    typing_context_apply(vm, &Value::obj(context), &[*receiver, *arguments])
}

fn descriptor_equivalent(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let (context, handle) = descriptor_parts(receiver, vm)?;
    let other_val = args.first().ok_or(RuntimeError::Arity {
        signature: "equivalentTo(_)",
        expected: 1,
        found: args.len(),
    })?;
    let other_handle = nominal_handle(vm, context, other_val)?;
    let context_data = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => data,
        _ => unreachable!(),
    };
    Ok(Value::bool(inspect::equivalent(context_data, &vm.typing_registry, handle, other_handle)))
}

fn descriptor_subtype(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let (context, left) = descriptor_parts(receiver, vm)?;
    let right_val = args.first().ok_or(RuntimeError::Arity {
        signature: "subtypeOf(_)",
        expected: 1,
        found: args.len(),
    })?;
    let right = nominal_handle(vm, context, right_val)?;
    let context_data = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => data,
        _ => unreachable!(),
    };
    let is_sub = inspect::subtype(context_data, &vm.typing_registry, &vm.heap, left, right);
    alloc_variant(vm, if is_sub { "RelationSatisfied" } else { "RelationRejected" }, Some(Value::bool(is_sub)))
}

// ---------------------------------------------------------------------------
// TypeParameter methods
// ---------------------------------------------------------------------------

fn type_param_owner(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = type_param_parts(receiver, vm)?;
    let owner_info = {
        let rec = inspect::type_param_record(&vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("type parameter not found".into()))?;
        rec.id.owner.clone()
    };
    match owner_info {
        phalcom_type_meta::generic::StableTypeParameterOwnerRef::Declaration(decl) => {
            if let Some(class_id) = vm.typing_registry.resolve_nominal(&decl) {
                Ok(Value::obj(class_id))
            } else {
                Ok(Value::none())
            }
        }
        phalcom_type_meta::generic::StableTypeParameterOwnerRef::Callable(callable) => {
            let call_handle_opt = {
                let loaded = vm
                    .typing_registry
                    .get_pool(handle.pool)
                    .ok_or_else(|| RuntimeError::Internal("pool not found".into()))?;
                loaded
                    .bundle
                    .callables
                    .iter()
                    .position(|c| c.callable == callable)
                    .map(|idx| RuntimeCallableSignatureRef {
                        pool: handle.pool,
                        record: phalcom_type_meta::declaration::CallableRecordId(idx as u32),
                        specialization_receiver: None,
                    })
            };
            if let Some(call_handle) = call_handle_opt {
                reify_callable_signature_obj(vm, context, call_handle)
            } else {
                Ok(Value::none())
            }
        }
    }
}

fn type_param_index(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = type_param_parts(receiver, _vm)?;
    Ok(Value::int(handle.index as i64))
}

fn type_param_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = type_param_parts(receiver, vm)?;
    let name = inspect::type_param_name(&vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("type parameter not found".into()))?;
    let sym = vm.get_or_intern(&name);
    Ok(Value::symbol(sym))
}

fn type_param_kind(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = type_param_parts(receiver, vm)?;
    let kind_handle = inspect::type_param_kind(&vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("type parameter not found".into()))?;
    reify_kind(vm, context, kind_handle)
}

fn type_param_variance(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = type_param_parts(receiver, vm)?;
    let variance_opt = inspect::type_param_variance(&vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("type parameter not found".into()))?;
    match variance_opt {
        Some(v) => {
            let sym = vm.get_or_intern(v);
            Ok(Value::symbol(sym).wrap_some()?)
        }
        None => Ok(Value::none()),
    }
}

fn type_param_constraint_count(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = type_param_parts(receiver, vm)?;
    let constraints = inspect::type_param_constraints(&vm.typing_registry, handle);
    Ok(Value::int(constraints.len() as i64))
}

fn type_param_constraint_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let (context, handle) = type_param_parts(receiver, vm)?;
    let index = args.first().and_then(|v| v.as_int()).ok_or_else(|| RuntimeError::Type {
        expected: "Int",
        found: args.first().map_or("missing", Value::type_name),
    })? as usize;
    let constraints = inspect::type_param_constraints(&vm.typing_registry, handle);
    let c = constraints
        .get(index)
        .ok_or_else(|| RuntimeError::Message("constraint index out of range".into()))?;
    reify_generic_constraint_obj(vm, context, *c)
}

fn type_param_constraints(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = type_param_parts(receiver, vm)?;
    let constraints = inspect::type_param_constraints(&vm.typing_registry, handle);
    let mut values = Vec::with_capacity(constraints.len());
    for c in constraints {
        values.push(reify_generic_constraint_obj(vm, context, c)?);
    }
    Ok(Value::obj(vm.heap.alloc(Object::Tuple(crate::heap::TupleObject::positional(values)))))
}

// ---------------------------------------------------------------------------
// GenericSignature methods
// ---------------------------------------------------------------------------

fn generic_sig_owner(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = generic_sig_parts(receiver, vm)?;
    let owner_info = {
        let rec = inspect::generic_sig_record(&vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("generic signature not found".into()))?;
        rec.owner.clone()
    };
    match owner_info {
        phalcom_type_meta::generic::StableTypeParameterOwnerRef::Declaration(decl) => {
            if let Some(class_id) = vm.typing_registry.resolve_nominal(&decl) {
                Ok(Value::obj(class_id))
            } else {
                Ok(Value::none())
            }
        }
        phalcom_type_meta::generic::StableTypeParameterOwnerRef::Callable(callable) => {
            let call_handle_opt = {
                let loaded = vm
                    .typing_registry
                    .get_pool(handle.pool)
                    .ok_or_else(|| RuntimeError::Internal("pool not found".into()))?;
                loaded
                    .bundle
                    .callables
                    .iter()
                    .position(|c| c.callable == callable)
                    .map(|idx| RuntimeCallableSignatureRef {
                        pool: handle.pool,
                        record: phalcom_type_meta::declaration::CallableRecordId(idx as u32),
                        specialization_receiver: None,
                    })
            };
            if let Some(call_handle) = call_handle_opt {
                reify_callable_signature_obj(vm, context, call_handle)
            } else {
                Ok(Value::none())
            }
        }
    }
}

fn generic_sig_parameter_count(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = generic_sig_parts(receiver, vm)?;
    let params = inspect::generic_sig_parameters(&vm.typing_registry, handle);
    Ok(Value::int(params.len() as i64))
}

fn generic_sig_parameter_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let (context, handle) = generic_sig_parts(receiver, vm)?;
    let index = args.first().and_then(|v| v.as_int()).ok_or_else(|| RuntimeError::Type {
        expected: "Int",
        found: args.first().map_or("missing", Value::type_name),
    })? as usize;
    let params = inspect::generic_sig_parameters(&vm.typing_registry, handle);
    let p = params.get(index).ok_or_else(|| RuntimeError::Message("parameter index out of range".into()))?;
    reify_type_parameter_obj(vm, context, *p)
}

fn generic_sig_parameters(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = generic_sig_parts(receiver, vm)?;
    let params = inspect::generic_sig_parameters(&vm.typing_registry, handle);
    let mut values = Vec::with_capacity(params.len());
    for p in params {
        values.push(reify_type_parameter_obj(vm, context, p)?);
    }
    Ok(Value::obj(vm.heap.alloc(Object::Tuple(crate::heap::TupleObject::positional(values)))))
}

fn generic_sig_constraint_count(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = generic_sig_parts(receiver, vm)?;
    let constraints = inspect::generic_sig_constraints(&vm.typing_registry, handle);
    Ok(Value::int(constraints.len() as i64))
}

fn generic_sig_constraint_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let (context, handle) = generic_sig_parts(receiver, vm)?;
    let index = args.first().and_then(|v| v.as_int()).ok_or_else(|| RuntimeError::Type {
        expected: "Int",
        found: args.first().map_or("missing", Value::type_name),
    })? as usize;
    let constraints = inspect::generic_sig_constraints(&vm.typing_registry, handle);
    let c = constraints
        .get(index)
        .ok_or_else(|| RuntimeError::Message("constraint index out of range".into()))?;
    reify_generic_constraint_obj(vm, context, *c)
}

fn generic_sig_constraints(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = generic_sig_parts(receiver, vm)?;
    let constraints = inspect::generic_sig_constraints(&vm.typing_registry, handle);
    let mut values = Vec::with_capacity(constraints.len());
    for c in constraints {
        values.push(reify_generic_constraint_obj(vm, context, c)?);
    }
    Ok(Value::obj(vm.heap.alloc(Object::Tuple(crate::heap::TupleObject::positional(values)))))
}

// ---------------------------------------------------------------------------
// GenericConstraint methods
// ---------------------------------------------------------------------------

fn generic_constraint_relation(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = generic_constraint_parts(receiver, vm)?;
    let relation =
        inspect::generic_constraint_relation(&vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("constraint relation not found".into()))?;
    let sym = vm.get_or_intern(relation);
    Ok(Value::symbol(sym))
}

fn generic_constraint_left(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = generic_constraint_parts(receiver, vm)?;
    let left_handle = inspect::generic_constraint_left(&vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("constraint left not found".into()))?;
    let desc_class = descriptor_class_for_handle(vm, context, left_handle)?;
    crate::typing::reify::reify_type_form(context, left_handle, &vm.typing_registry, &mut vm.heap, desc_class)
}

fn generic_constraint_right(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = generic_constraint_parts(receiver, vm)?;
    let right_handle =
        inspect::generic_constraint_right(&vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("constraint right not found".into()))?;
    let desc_class = descriptor_class_for_handle(vm, context, right_handle)?;
    crate::typing::reify::reify_type_form(context, right_handle, &vm.typing_registry, &mut vm.heap, desc_class)
}

fn generic_constraint_source(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::none())
}

// ---------------------------------------------------------------------------
// CallableSignature methods
// ---------------------------------------------------------------------------

fn callable_sig_owner(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = callable_sig_parts(receiver, vm)?;
    let decl = {
        let rec = inspect::callable_record(&vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("callable signature not found".into()))?;
        rec.callable.owner.clone()
    };
    if let Some(class_id) = vm.typing_registry.resolve_nominal(&decl) {
        Ok(Value::obj(class_id))
    } else {
        Ok(Value::none())
    }
}

fn callable_sig_side(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = callable_sig_parts(receiver, vm)?;
    let side_str = {
        let rec = inspect::callable_record(&vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("callable signature not found".into()))?;
        match rec.callable.side {
            phalcom_type_meta::identity::StableDispatchSide::Instance => "instance",
            phalcom_type_meta::identity::StableDispatchSide::Class => "class",
        }
    };
    let sym = vm.get_or_intern(side_str);
    Ok(Value::symbol(sym))
}

fn callable_sig_selector(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = callable_sig_parts(receiver, vm)?;
    let selector = {
        let rec = inspect::callable_record(&vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("callable signature not found".into()))?;
        rec.callable.selector.to_string()
    };
    let sym = vm.get_or_intern(&selector);
    Ok(Value::symbol(sym))
}

fn callable_sig_generic_signature(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = callable_sig_parts(receiver, vm)?;
    if let Some(sig) = inspect::callable_generic_signature(&vm.typing_registry, handle) {
        let val = reify_generic_signature_obj(vm, context, sig)?;
        Ok(val.wrap_some()?)
    } else {
        Ok(Value::none())
    }
}

fn callable_sig_parameter_count(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = callable_sig_parts(receiver, vm)?;
    let params = inspect::callable_parameters(&vm.typing_registry, handle);
    Ok(Value::int(params.len() as i64))
}

fn callable_sig_parameter_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let (context, handle) = callable_sig_parts(receiver, vm)?;
    let index = args.first().and_then(|v| v.as_int()).ok_or_else(|| RuntimeError::Type {
        expected: "Int",
        found: args.first().map_or("missing", Value::type_name),
    })? as usize;
    let params = inspect::callable_parameters(&vm.typing_registry, handle);
    let p = params.get(index).ok_or_else(|| RuntimeError::Message("parameter index out of range".into()))?;
    reify_callable_parameter_obj(vm, context, *p)
}

fn callable_sig_parameter_type_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let (context, handle) = callable_sig_parts(receiver, vm)?;
    let index = args.first().and_then(|v| v.as_int()).ok_or_else(|| RuntimeError::Type {
        expected: "Int",
        found: args.first().map_or("missing", Value::type_name),
    })? as usize;
    let ty_handle = {
        let context_data = match &vm.heap.typing_object(context).payload {
            TypingPayload::Context(data) => data,
            _ => unreachable!(),
        };
        inspect::callable_parameter_type_at(context_data, &vm.typing_registry, handle, index)
            .ok_or_else(|| RuntimeError::Message("parameter type not found".into()))?
    };
    let desc_class = descriptor_class_for_handle(vm, context, ty_handle)?;
    crate::typing::reify::reify_type_form(context, ty_handle, &vm.typing_registry, &mut vm.heap, desc_class)
}

fn callable_sig_parameters(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = callable_sig_parts(receiver, vm)?;
    let params = inspect::callable_parameters(&vm.typing_registry, handle);
    let mut values = Vec::with_capacity(params.len());
    for p in params {
        values.push(reify_callable_parameter_obj(vm, context, p)?);
    }
    Ok(Value::obj(vm.heap.alloc(Object::Tuple(crate::heap::TupleObject::positional(values)))))
}

fn callable_sig_return_type(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = callable_sig_parts(receiver, vm)?;
    let ty_handle = {
        let context_data = match &vm.heap.typing_object(context).payload {
            TypingPayload::Context(data) => data,
            _ => unreachable!(),
        };
        inspect::callable_return_type(context_data, &vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("return type not found".into()))?
    };
    let desc_class = descriptor_class_for_handle(vm, context, ty_handle)?;
    crate::typing::reify::reify_type_form(context, ty_handle, &vm.typing_registry, &mut vm.heap, desc_class)
}

fn callable_sig_source(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::none())
}

fn callable_sig_documentation(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::none())
}

// ---------------------------------------------------------------------------
// CallableParameter methods
// ---------------------------------------------------------------------------

fn callable_param_index(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = callable_param_parts(receiver, _vm)?;
    Ok(Value::int(handle.index as i64))
}

fn callable_param_local_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = callable_param_parts(receiver, vm)?;
    let name = inspect::callable_param_local_name(&vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("parameter not found".into()))?;
    let sym = vm.get_or_intern(&name);
    Ok(Value::symbol(sym))
}

fn callable_param_external_label(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = callable_param_parts(receiver, vm)?;
    let label = inspect::callable_param_external_label(&vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("parameter not found".into()))?;
    match label {
        Some(l) => {
            let sym = vm.get_or_intern(&l);
            Ok(Value::symbol(sym).wrap_some()?)
        }
        None => Ok(Value::none()),
    }
}

fn callable_param_rest_mode(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = callable_param_parts(receiver, vm)?;
    let mode = inspect::callable_param_rest_mode(&vm.typing_registry, handle);
    let sym = vm.get_or_intern(mode);
    Ok(Value::symbol(sym))
}

fn callable_param_type(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = callable_param_parts(receiver, vm)?;
    let ty_handle = {
        let context_data = match &vm.heap.typing_object(context).payload {
            TypingPayload::Context(data) => data,
            _ => unreachable!(),
        };
        inspect::callable_param_type(context_data, &vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("parameter type not found".into()))?
    };
    let desc_class = descriptor_class_for_handle(vm, context, ty_handle)?;
    crate::typing::reify::reify_type_form(context, ty_handle, &vm.typing_registry, &mut vm.heap, desc_class)
}

fn callable_param_source(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::none())
}

// ---------------------------------------------------------------------------
// FieldSignature methods
// ---------------------------------------------------------------------------

fn field_sig_owner(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = field_sig_parts(receiver, vm)?;
    let decl = {
        let rec = inspect::field_record(&vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("field signature not found".into()))?;
        rec.field.owner.clone()
    };
    if let Some(class_id) = vm.typing_registry.resolve_nominal(&decl) {
        Ok(Value::obj(class_id))
    } else {
        Ok(Value::none())
    }
}

fn field_sig_side(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = field_sig_parts(receiver, vm)?;
    let side_str = {
        let rec = inspect::field_record(&vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("field signature not found".into()))?;
        match rec.field.side {
            phalcom_type_meta::identity::StableDispatchSide::Instance => "instance",
            phalcom_type_meta::identity::StableDispatchSide::Class => "class",
        }
    };
    let sym = vm.get_or_intern(side_str);
    Ok(Value::symbol(sym))
}

fn field_sig_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = field_sig_parts(receiver, vm)?;
    let name = inspect::field_name(&vm.typing_registry, handle).ok_or_else(|| RuntimeError::Message("field signature not found".into()))?;
    let sym = vm.get_or_intern(&name);
    Ok(Value::symbol(sym))
}

fn field_sig_mutable(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = field_sig_parts(receiver, vm)?;
    let mutable = inspect::field_mutable(&vm.typing_registry, handle);
    Ok(Value::bool(mutable))
}

fn field_sig_type(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = field_sig_parts(receiver, vm)?;
    let ty_handle_opt = {
        let context_data = match &vm.heap.typing_object(context).payload {
            TypingPayload::Context(data) => data,
            _ => unreachable!(),
        };
        inspect::field_type(context_data, &vm.typing_registry, handle)
    };
    if let Some(ty_handle) = ty_handle_opt {
        let desc_class = descriptor_class_for_handle(vm, context, ty_handle)?;
        let val = crate::typing::reify::reify_type_form(context, ty_handle, &vm.typing_registry, &mut vm.heap, desc_class)?;
        alloc_variant(vm, "TypingKnown", Some(val))
    } else {
        alloc_unavailable(vm, "unannotated_field")
    }
}

fn field_sig_source(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::none())
}

// ---------------------------------------------------------------------------
// TypeUse methods
// ---------------------------------------------------------------------------

fn type_use_value_type(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = type_use_parts(receiver, vm)?;
    if let Some(ty_handle) = inspect::type_use_denotation(&vm.typing_registry, handle) {
        let desc_class = descriptor_class_for_handle(vm, context, ty_handle)?;
        let val = crate::typing::reify::reify_type_form(context, ty_handle, &vm.typing_registry, &mut vm.heap, desc_class)?;
        alloc_variant(vm, "TypingKnown", Some(val))
    } else {
        alloc_unavailable(vm, "no_value_type")
    }
}

fn type_use_denotation(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (context, handle) = type_use_parts(receiver, vm)?;
    if let Some(ty_handle) = inspect::type_use_denotation(&vm.typing_registry, handle) {
        let desc_class = descriptor_class_for_handle(vm, context, ty_handle)?;
        let val = crate::typing::reify::reify_type_form(context, ty_handle, &vm.typing_registry, &mut vm.heap, desc_class)?;
        alloc_variant(vm, "TypingKnown", Some(val))
    } else {
        alloc_unavailable(vm, "no_denotation")
    }
}

fn type_use_source(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::none())
}

fn type_use_spelling(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let (_context, handle) = type_use_parts(receiver, vm)?;
    match inspect::type_use_written(&vm.typing_registry, handle) {
        Some(s) => Ok(vm.alloc_string_value(s).wrap_some()?),
        None => Ok(Value::none()),
    }
}

fn type_use_evidence(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::none())
}

fn type_use_inference(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::none())
}

fn type_use_constant(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::none())
}

// ---------------------------------------------------------------------------
// Typing & TypingContext methods
// ---------------------------------------------------------------------------

fn typing_current(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    alloc_context(vm, MetadataProfile::RuntimePublic)
}

fn typing_context_for(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if let Some(arg) = args.first() {
        if let Some(sym) = arg.symbol_value() {
            let sym_str = vm.resolve_symbol(sym).to_ascii_lowercase();
            let profile = match sym_str.as_str() {
                "minimal" | "runtimeminimal" => MetadataProfile::RuntimeMinimal,
                "public" | "runtimepublic" => MetadataProfile::RuntimePublic,
                "tooling" | "debug" | "toolingdebug" => MetadataProfile::ToolingDebug,
                "proof" => MetadataProfile::Proof,
                _ => MetadataProfile::RuntimePublic,
            };
            let mut caps = TypingCapabilities::for_profile(profile);
            if profile == MetadataProfile::ToolingDebug || profile == MetadataProfile::Proof {
                caps = caps.with(TypingCapability::ValidateRuntimeValues).with(TypingCapability::InvokeReflectively);
            }
            let ctx = alloc_context_with_capabilities(vm, profile, caps)?;
            return alloc_variant(vm, "TypingKnown", Some(ctx));
        } else if let Some(tuple_id) = arg.as_obj().and_then(|obj| vm.heap.as_tuple(obj).map(|_| obj)) {
            let values = vm.heap.tuple(tuple_id).values().to_vec();
            let mut caps = TypingCapabilities::empty();
            for v in values {
                if let Some(s) = v.symbol_value() {
                    let s_str = vm.resolve_symbol(s);
                    for cap in TypingCapability::ALL {
                        if cap.display().eq_ignore_ascii_case(s_str) {
                            caps = caps.with(cap);
                        }
                    }
                }
            }
            let ctx = alloc_context_with_capabilities(vm, MetadataProfile::RuntimePublic, caps)?;
            return alloc_variant(vm, "TypingKnown", Some(ctx));
        }
    }
    let ctx = alloc_context(vm, MetadataProfile::RuntimePublic)?;
    alloc_variant(vm, "TypingKnown", Some(ctx))
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
    let prof_str = match profile {
        MetadataProfile::RuntimeMinimal => "RuntimeMinimal",
        MetadataProfile::RuntimePublic => "RuntimePublic",
        MetadataProfile::ToolingDebug => "ToolingDebug",
        MetadataProfile::Proof => "Proof",
    };
    let sym = vm.get_or_intern(prof_str);
    Ok(Value::symbol(sym))
}

fn typing_context_capabilities(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    let names = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => data.capabilities.iter().map(|capability| capability.display()).collect::<Vec<_>>(),
        _ => unreachable!(),
    };
    let mut symbols = Vec::with_capacity(names.len());
    for name in names {
        symbols.push(Value::symbol(vm.get_or_intern(name)));
    }
    Ok(Value::obj(vm.heap.alloc(Object::Tuple(crate::heap::TupleObject::positional(symbols)))))
}

fn typing_context_restrict(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    let requested = expect_tuple(
        vm,
        args.first().ok_or(RuntimeError::Arity {
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
    let new_ctx = Value::obj(vm.heap.alloc(Object::Typing(Box::new(object))));
    alloc_variant(vm, "TypingKnown", Some(new_ctx))
}

fn typing_context_semantic_model(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(vm.alloc_string_value("1.0".to_string()))
}

fn typing_context_snapshot(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::int(1))
}

fn typing_context_world(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::int(vm.world_version as i64))
}

fn typing_context_type_of_declaration(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::ObservePublicTypes)?;
    let decl_val = args.first().ok_or(RuntimeError::Arity {
        signature: "typeOfDeclaration(_)",
        expected: 1,
        found: args.len(),
    })?;
    let handle = nominal_handle(vm, context, decl_val)?;
    context_known(vm, context, handle)
}

fn typing_context_generic_signature_of(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::ObserveSignatures)?;
    let decl_val = args.first().ok_or(RuntimeError::Arity {
        signature: "genericSignatureOf(_)",
        expected: 1,
        found: args.len(),
    })?;
    let Some(class_id) = decl_val.as_obj().filter(|id| vm.heap.as_class(*id).is_some()) else {
        return alloc_invalid(vm, "not_a_declaration");
    };

    if let Some(sig_ref) = inspect::generic_signature_of_declaration(&vm.typing_registry, class_id, &vm.heap) {
        let sig_obj = reify_generic_signature_obj(vm, context, sig_ref)?;
        alloc_variant(vm, "TypingKnown", Some(sig_obj))
    } else {
        alloc_unavailable(vm, "no_generic_signature")
    }
}

fn typing_context_declared_supertype_of(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::ObservePublicTypes)?;
    let decl_val = args.first().ok_or(RuntimeError::Arity {
        signature: "declaredSupertypeOf(_)",
        expected: 1,
        found: args.len(),
    })?;
    let Some(class_id) = decl_val.as_obj().filter(|id| vm.heap.as_class(*id).is_some()) else {
        return alloc_invalid(vm, "not_a_declaration");
    };

    if let Some(super_handle) = inspect::declared_supertype_of_declaration(&vm.typing_registry, class_id, &vm.heap) {
        context_known(vm, context, super_handle)
    } else if let Some(superclass_id) = vm.heap.class(class_id).superclass {
        alloc_variant(vm, "TypingKnown", Some(Value::obj(superclass_id)))
    } else {
        alloc_unavailable(vm, "no_superclass")
    }
}

fn typing_context_signature_of(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::ObserveSignatures)?;
    let method_val = args.first().ok_or(RuntimeError::Arity {
        signature: "signatureOf(_)",
        expected: 1,
        found: args.len(),
    })?;
    let Some(method_ref) = method_val.as_obj() else {
        return alloc_invalid(vm, "not_a_method");
    };

    if let Some(call_ref) = vm.typing_registry.method_semantics.get(method_ref) {
        let sig_ref = RuntimeCallableSignatureRef {
            pool: call_ref.pool,
            record: call_ref.record,
            specialization_receiver: None,
        };
        let sig_obj = reify_callable_signature_obj(vm, context, sig_ref)?;
        alloc_variant(vm, "TypingKnown", Some(sig_obj))
    } else {
        alloc_unavailable(vm, "unannotated_or_dynamic_method")
    }
}

fn typing_context_apply(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::ConstructTypeForms)?;
    let origin = args.first().ok_or(RuntimeError::Arity {
        signature: "apply(_,_)",
        expected: 2,
        found: args.len(),
    })?;
    let arguments = args.get(1).ok_or(RuntimeError::Arity {
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
        args.first().ok_or(RuntimeError::Arity {
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
        args.first().ok_or(RuntimeError::Arity {
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

fn typing_context_record_of(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::ConstructTypeForms)?;
    let fields_arg = args.first().ok_or(RuntimeError::Arity {
        signature: "recordOf(_)",
        expected: 1,
        found: args.len(),
    })?;

    let mut fields = Vec::new();
    if let Some(tuple_id) = fields_arg.as_obj().filter(|id| vm.heap.as_tuple(*id).is_some()) {
        let items = vm.heap.tuple(tuple_id).values().to_vec();
        for item in items {
            let pair_id = expect_tuple(vm, &item)?;
            let pair = vm.heap.tuple(pair_id).values().to_vec();
            if pair.len() == 2 {
                let name = pair[0].symbol_value().map(|s| vm.resolve_symbol(s).to_string()).unwrap_or_default();
                let ty = nominal_handle(vm, context, &pair[1])?;
                fields.push(RuntimeRecordField {
                    name: name.into_boxed_str(),
                    ty,
                });
            }
        }
    }
    let handle = {
        let object = vm
            .heap
            .as_typing_object_mut(context)
            .ok_or_else(|| RuntimeError::Internal("typing context disappeared".to_string()))?;
        let TypingPayload::Context(data) = &mut object.payload else { unreachable!() };
        data.overlay.type_ref(RuntimeOverlayTypeNode::Record(fields.into_boxed_slice()))
    };
    context_known(vm, context, handle)
}

fn typing_context_callable(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::ConstructTypeForms)?;
    let parameters = tuple_type_args(
        vm,
        context,
        args.first().ok_or(RuntimeError::Arity {
            signature: "callable(_,_)",
            expected: 2,
            found: args.len(),
        })?,
    )?;
    let returns = nominal_handle(
        vm,
        context,
        args.get(1).ok_or(RuntimeError::Arity {
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

fn typing_context_equivalent(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::EvaluateRelations)?;
    let left = nominal_handle(
        vm,
        context,
        args.first().ok_or(RuntimeError::Arity {
            signature: "equivalent(_,_)",
            expected: 2,
            found: args.len(),
        })?,
    )?;
    let right = nominal_handle(
        vm,
        context,
        args.get(1).ok_or(RuntimeError::Arity {
            signature: "equivalent(_,_)",
            expected: 2,
            found: args.len(),
        })?,
    )?;
    let context_data = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => data,
        _ => unreachable!(),
    };
    let eq = inspect::equivalent(context_data, &vm.typing_registry, left, right);
    alloc_variant(vm, if eq { "RelationSatisfied" } else { "RelationRejected" }, Some(Value::bool(eq)))
}

fn typing_context_subtype(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::EvaluateRelations)?;
    let left = nominal_handle(
        vm,
        context,
        args.first().ok_or(RuntimeError::Arity {
            signature: "subtype(_,_)",
            expected: 2,
            found: args.len(),
        })?,
    )?;
    let right = nominal_handle(
        vm,
        context,
        args.get(1).ok_or(RuntimeError::Arity {
            signature: "subtype(_,_)",
            expected: 2,
            found: args.len(),
        })?,
    )?;
    let context_data = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => data,
        _ => unreachable!(),
    };
    let is_sub = inspect::subtype(context_data, &vm.typing_registry, &vm.heap, left, right);
    alloc_variant(vm, if is_sub { "RelationSatisfied" } else { "RelationRejected" }, Some(Value::bool(is_sub)))
}

fn typing_context_assignable(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::EvaluateRelations)?;
    let _left = nominal_handle(
        vm,
        context,
        args.first().ok_or(RuntimeError::Arity {
            signature: "assignable(_,_)",
            expected: 2,
            found: args.len(),
        })?,
    )?;
    let _right = nominal_handle(
        vm,
        context,
        args.get(1).ok_or(RuntimeError::Arity {
            signature: "assignable(_,_)",
            expected: 2,
            found: args.len(),
        })?,
    )?;
    alloc_unavailable(vm, "assignability_unavailable")
}

fn typing_context_consistent(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::EvaluateRelations)?;
    let _left = nominal_handle(
        vm,
        context,
        args.first().ok_or(RuntimeError::Arity {
            signature: "consistent(_,_)",
            expected: 2,
            found: args.len(),
        })?,
    )?;
    let _right = nominal_handle(
        vm,
        context,
        args.get(1).ok_or(RuntimeError::Arity {
            signature: "consistent(_,_)",
            expected: 2,
            found: args.len(),
        })?,
    )?;
    alloc_unavailable(vm, "consistency_unavailable")
}

fn typing_context_conforms(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::EvaluateRelations)?;
    let _left = nominal_handle(
        vm,
        context,
        args.first().ok_or(RuntimeError::Arity {
            signature: "conforms(_,_)",
            expected: 2,
            found: args.len(),
        })?,
    )?;
    let _right = nominal_handle(
        vm,
        context,
        args.get(1).ok_or(RuntimeError::Arity {
            signature: "conforms(_,_)",
            expected: 2,
            found: args.len(),
        })?,
    )?;
    alloc_unavailable(vm, "conformance_unavailable")
}

fn typing_context_member(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::ObserveSignatures)?;
    let recv_ty = nominal_handle(
        vm,
        context,
        args.first().ok_or(RuntimeError::Arity {
            signature: "member(_,_,_,_)",
            expected: 4,
            found: args.len(),
        })?,
    )?;
    let selector_val = args.get(1).ok_or(RuntimeError::Arity {
        signature: "member(_,_,_,_)",
        expected: 4,
        found: args.len(),
    })?;
    let selector_str = selector_val.symbol_value().map(|s| vm.resolve_symbol(s).to_string()).unwrap_or_default();
    let side_val = args.get(2).and_then(|v| v.symbol_value()).map(|s| vm.resolve_symbol(s).to_string());
    let is_class_side = side_val.as_deref() == Some("class");

    let call_ref_opt = {
        let context_data = match &vm.heap.typing_object(context).payload {
            TypingPayload::Context(data) => data,
            _ => unreachable!(),
        };
        inspect::lookup_member(context_data, &vm.typing_registry, &vm.heap, recv_ty, &selector_str, is_class_side)
    };

    if let Some(call_ref) = call_ref_opt {
        let sig_obj = reify_callable_signature_obj(vm, context, call_ref)?;
        alloc_variant(vm, "MemberFound", Some(sig_obj))
    } else {
        alloc_member_missing(vm, "member_not_found")
    }
}

fn typing_context_type_use_at(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::ObserveSourceUses)?;
    let profile = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => data.profile,
        _ => unreachable!(),
    };
    if profile != MetadataProfile::ToolingDebug {
        return alloc_unavailable(vm, "omitted_in_current_profile");
    }
    let sym = Value::symbol(vm.get_or_intern("no_occurrence_data"));
    alloc_variant(vm, "TypingUnknown", Some(sym))
}

fn typing_context_type_uses_of(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::ObserveSourceUses)?;
    let profile = match &vm.heap.typing_object(context).payload {
        TypingPayload::Context(data) => data.profile,
        _ => unreachable!(),
    };
    if profile != MetadataProfile::ToolingDebug {
        return alloc_unavailable(vm, "omitted_in_current_profile");
    }
    let tuple_obj = vm.heap.alloc(Object::Tuple(crate::heap::TupleObject::positional(Vec::new())));
    alloc_variant(vm, "TypingKnown", Some(Value::obj(tuple_obj)))
}

fn typing_context_matches(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    let val = args.first().ok_or(RuntimeError::Arity {
        signature: "matches(_,_)",
        expected: 2,
        found: args.len(),
    })?;
    let form_val = args.get(1).ok_or(RuntimeError::Arity {
        signature: "matches(_,_)",
        expected: 2,
        found: args.len(),
    })?;

    if let Some(target_class) = form_val.as_obj().filter(|id| vm.heap.as_class(*id).is_some()) {
        let val_class = val.class(vm);
        let mut curr = Some(val_class);
        let mut satisfied = false;
        while let Some(c) = curr {
            if c == target_class {
                satisfied = true;
                break;
            }
            curr = vm.heap.class(c).superclass;
        }
        return alloc_variant(
            vm,
            if satisfied { "RelationSatisfied" } else { "RelationRejected" },
            Some(Value::bool(satisfied)),
        );
    }

    let form_handle = nominal_handle(vm, context, form_val)?;
    match form_handle {
        RuntimeTypeRef::Overlay(id) => {
            let is_applied = {
                let context_data = match &vm.heap.typing_object(context).payload {
                    TypingPayload::Context(data) => data,
                    _ => unreachable!(),
                };
                matches!(context_data.overlay.type_node(id), Some(RuntimeOverlayTypeNode::Applied { .. }))
            };
            if is_applied {
                alloc_dynamic_boundary(vm, "erased_generic_boundary")
            } else {
                alloc_variant(vm, "RelationRejected", Some(Value::bool(false)))
            }
        }
        RuntimeTypeRef::Base { pool, node } => {
            let is_applied = {
                let loaded = vm
                    .typing_registry
                    .get_pool(pool)
                    .ok_or_else(|| RuntimeError::Internal("pool not found".into()))?;
                let entry = loaded
                    .bundle
                    .types
                    .get(node.0 as usize)
                    .ok_or_else(|| RuntimeError::Internal("node not found".into()))?;
                matches!(&entry.form, phalcom_type_meta::type_node::TypeNode::Applied { .. })
            };
            if is_applied {
                alloc_dynamic_boundary(vm, "erased_generic_boundary")
            } else {
                alloc_variant(vm, "RelationRejected", Some(Value::bool(false)))
            }
        }
    }
}

fn typing_context_validate(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::ValidateRuntimeValues)?;
    let val = args.first().ok_or(RuntimeError::Arity {
        signature: "validate(_,_)",
        expected: 2,
        found: args.len(),
    })?;
    let form_val = args.get(1).ok_or(RuntimeError::Arity {
        signature: "validate(_,_)",
        expected: 2,
        found: args.len(),
    })?;

    if let Some(target_class) = form_val.as_obj().filter(|id| vm.heap.as_class(*id).is_some()) {
        let val_class = val.class(vm);
        let mut curr = Some(val_class);
        let mut satisfied = false;
        while let Some(c) = curr {
            if c == target_class {
                satisfied = true;
                break;
            }
            curr = vm.heap.class(c).superclass;
        }
        return alloc_variant(
            vm,
            if satisfied { "RelationSatisfied" } else { "RelationRejected" },
            Some(Value::bool(satisfied)),
        );
    }

    if let Some(list_id) = val.as_obj().filter(|id| vm.heap.as_list(*id).is_some()) {
        let elements = vm.heap.list(list_id).elements().to_vec();
        let int_class = vm.universe.classes.int_class;
        let all_int = elements.iter().all(|e| e.class(vm) == int_class);
        return alloc_variant(vm, if all_int { "RelationSatisfied" } else { "RelationRejected" }, Some(Value::bool(all_int)));
    }

    alloc_dynamic_boundary(vm, "unsupported_container_validation")
}

fn typing_context_construct(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::InvokeReflectively)?;
    let form_val = args.first().ok_or(RuntimeError::Arity {
        signature: "construct(_,_)",
        expected: 2,
        found: args.len(),
    })?;
    let args_val = args.get(1).ok_or(RuntimeError::Arity {
        signature: "construct(_,_)",
        expected: 2,
        found: args.len(),
    })?;

    let class_id = if let Some(c) = form_val.as_obj().filter(|id| vm.heap.as_class(*id).is_some()) {
        c
    } else {
        let handle = nominal_handle(vm, context, form_val)?;
        match handle {
            RuntimeTypeRef::Overlay(id) => {
                let context_data = match &vm.heap.typing_object(context).payload {
                    TypingPayload::Context(data) => data,
                    _ => unreachable!(),
                };
                match context_data.overlay.type_node(id) {
                    Some(RuntimeOverlayTypeNode::Nominal { class }) => *class,
                    Some(RuntimeOverlayTypeNode::Applied {
                        origin: RuntimeTypeRef::Overlay(o_id),
                        ..
                    }) => match context_data.overlay.type_node(*o_id) {
                        Some(RuntimeOverlayTypeNode::Nominal { class }) => *class,
                        _ => return Err(RuntimeError::Message("invalid construct target".into()).into()),
                    },
                    _ => return Err(RuntimeError::Message("invalid construct target".into()).into()),
                }
            }
            RuntimeTypeRef::Base { pool, node } => {
                let loaded = vm
                    .typing_registry
                    .get_pool(pool)
                    .ok_or_else(|| RuntimeError::Internal("pool not found".into()))?;
                let entry = loaded
                    .bundle
                    .types
                    .get(node.0 as usize)
                    .ok_or_else(|| RuntimeError::Internal("node not found".into()))?;
                match &entry.form {
                    phalcom_type_meta::type_node::TypeNode::Nominal { declaration } => vm
                        .typing_registry
                        .resolve_nominal(declaration)
                        .ok_or_else(|| RuntimeError::Message("unresolved nominal".into()))?,
                    _ => return Err(RuntimeError::Message("invalid construct target".into()).into()),
                }
            }
        }
    };

    let tuple_id = expect_tuple(vm, args_val)?;
    let ctor_args = vm.heap.tuple(tuple_id).values().to_vec();
    let new_selector = vm.get_or_intern(match ctor_args.len() {
        0 => "new()",
        1 => "new(_)",
        2 => "new(_,_)",
        _ => "new()",
    });

    let result = vm.send_dynamic(Value::obj(class_id), new_selector, &ctor_args)?;
    alloc_variant(vm, "TypingKnown", Some(result))
}

fn typing_context_proofs_of(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let context = context_ref(receiver, vm)?;
    ensure_capability(vm, context, TypingCapability::InspectProofs)?;
    alloc_unavailable(vm, "proofs_deferred_to_spec05")
}
