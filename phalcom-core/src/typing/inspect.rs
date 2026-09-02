//! Small, bounded projections over validated metadata and context overlays.

use crate::heap::{ClassId, Heap};
use crate::typing::context::TypingContextData;
use crate::typing::handle::{RuntimeKindRef, RuntimeOverlayTypeId, RuntimeTypeRef};
use crate::typing::overlay::{RuntimeOverlayKindNode, RuntimeOverlayTypeNode};
use crate::typing::registry::RuntimeTypingRegistry;
use crate::universe::Universe;
use phalcom_type_meta::type_node::TypeNode;

const MAX_DISPLAY_DEPTH: usize = 64;

pub fn overlay_node(context: &TypingContextData, handle: RuntimeTypeRef) -> Option<&RuntimeOverlayTypeNode> {
    match handle {
        RuntimeTypeRef::Overlay(id) => context.overlay.type_node(id),
        RuntimeTypeRef::Base { .. } => None,
    }
}

pub fn children(context: &TypingContextData, registry: &RuntimeTypingRegistry, handle: RuntimeTypeRef) -> Option<Vec<RuntimeTypeRef>> {
    match handle {
        RuntimeTypeRef::Overlay(id) => match context.overlay.type_node(id)? {
            RuntimeOverlayTypeNode::Nominal { .. } => Some(Vec::new()),
            RuntimeOverlayTypeNode::Applied { origin, arguments } => {
                let mut children = Vec::with_capacity(arguments.len() + 1);
                children.push(*origin);
                children.extend(arguments.iter().copied());
                Some(children)
            }
            RuntimeOverlayTypeNode::Union(members) => Some(members.to_vec()),
            RuntimeOverlayTypeNode::Tuple(elements) => Some(elements.iter().map(|element| element.ty).collect()),
            RuntimeOverlayTypeNode::Record(fields) => Some(fields.iter().map(|f| f.ty).collect()),
            RuntimeOverlayTypeNode::Callable { parameters, return_type } => {
                let mut children = parameters.iter().map(|parameter| parameter.ty).collect::<Vec<_>>();
                children.push(*return_type);
                Some(children)
            }
            RuntimeOverlayTypeNode::TypeLambda { body, .. } => Some(vec![*body]),
            RuntimeOverlayTypeNode::Special(_) | RuntimeOverlayTypeNode::SelfType(_) => Some(Vec::new()),
        },
        RuntimeTypeRef::Base { pool, node } => {
            let entry = &registry.get_pool(pool)?.bundle.types.get(node.0 as usize)?.form;
            match entry {
                TypeNode::Applied { origin, arguments } => {
                    let mut children = Vec::with_capacity(arguments.len() + 1);
                    children.push(RuntimeTypeRef::Base { pool, node: *origin });
                    children.extend(arguments.iter().map(|node| RuntimeTypeRef::Base { pool, node: *node }));
                    Some(children)
                }
                TypeNode::Union(members) => Some(members.iter().map(|node| RuntimeTypeRef::Base { pool, node: *node }).collect()),
                _ => Some(Vec::new()),
            }
        }
    }
}

pub fn arguments(context: &TypingContextData, registry: &RuntimeTypingRegistry, handle: RuntimeTypeRef) -> Option<Vec<RuntimeTypeRef>> {
    match handle {
        RuntimeTypeRef::Overlay(id) => match context.overlay.type_node(id)? {
            RuntimeOverlayTypeNode::Applied { arguments, .. } => Some(arguments.to_vec()),
            RuntimeOverlayTypeNode::Union(members) => Some(members.to_vec()),
            RuntimeOverlayTypeNode::Nominal { .. } => Some(Vec::new()),
            RuntimeOverlayTypeNode::Tuple(elements) => Some(elements.iter().map(|element| element.ty).collect()),
            RuntimeOverlayTypeNode::Record(fields) => Some(fields.iter().map(|f| f.ty).collect()),
            RuntimeOverlayTypeNode::Callable { parameters, .. } => Some(parameters.iter().map(|parameter| parameter.ty).collect()),
            RuntimeOverlayTypeNode::TypeLambda { .. } | RuntimeOverlayTypeNode::Special(_) | RuntimeOverlayTypeNode::SelfType(_) => Some(Vec::new()),
        },
        RuntimeTypeRef::Base { pool, node } => match &registry.get_pool(pool)?.bundle.types.get(node.0 as usize)?.form {
            TypeNode::Applied { arguments, .. } => Some(arguments.iter().map(|node| RuntimeTypeRef::Base { pool, node: *node }).collect()),
            TypeNode::Union(members) => Some(members.iter().map(|node| RuntimeTypeRef::Base { pool, node: *node }).collect()),
            _ => Some(Vec::new()),
        },
    }
}

fn class_constructor_arity(universe: &Universe, class: ClassId) -> usize {
    phalcom_native_meta::universe::UNIVERSE_TYPE_FORMS
        .iter()
        .find(|spec| universe.classes.resolve(spec.owner) == class)
        .map_or(0, |spec| spec.parameters.len())
}

fn base_constructor_arity(
    registry: &RuntimeTypingRegistry,
    pool: crate::typing::handle::MetadataPoolId,
    node: phalcom_type_meta::type_node::TypeNodeId,
) -> usize {
    let Some(loaded) = registry.get_pool(pool) else { return 0 };
    let Some(entry) = loaded.bundle.types.get(node.0 as usize) else { return 0 };
    match &entry.form {
        TypeNode::Nominal { declaration } => loaded
            .bundle
            .declarations
            .iter()
            .find(|record| record.declaration == *declaration)
            .and_then(|record| record.generic_signature)
            .and_then(|id| loaded.bundle.generic_signatures.get(id.0 as usize))
            .map_or(0, |signature| signature.parameters.len()),
        TypeNode::Applied { origin, .. } => base_constructor_arity(registry, pool, *origin),
        _ => 0,
    }
}

pub fn remaining_parameter_count(context: &TypingContextData, registry: &RuntimeTypingRegistry, universe: &Universe, handle: RuntimeTypeRef) -> usize {
    match handle {
        RuntimeTypeRef::Overlay(id) => match context.overlay.type_node(id) {
            Some(RuntimeOverlayTypeNode::Nominal { class }) => class_constructor_arity(universe, *class),
            Some(RuntimeOverlayTypeNode::Applied { origin, arguments }) => {
                let arity = match origin {
                    RuntimeTypeRef::Overlay(origin_id) => match context.overlay.type_node(*origin_id) {
                        Some(RuntimeOverlayTypeNode::Nominal { class }) => class_constructor_arity(universe, *class),
                        Some(RuntimeOverlayTypeNode::Applied { .. }) => remaining_parameter_count(context, registry, universe, *origin) + arguments.len(),
                        _ => 0,
                    },
                    RuntimeTypeRef::Base { pool, node } => base_constructor_arity(registry, *pool, *node),
                };
                arity.saturating_sub(arguments.len())
            }
            Some(RuntimeOverlayTypeNode::Union(_))
            | Some(RuntimeOverlayTypeNode::Tuple(_))
            | Some(RuntimeOverlayTypeNode::Record(_))
            | Some(RuntimeOverlayTypeNode::Callable { .. })
            | Some(RuntimeOverlayTypeNode::TypeLambda { .. })
            | Some(RuntimeOverlayTypeNode::Special(_))
            | Some(RuntimeOverlayTypeNode::SelfType(_))
            | None => 0,
        },
        RuntimeTypeRef::Base { pool, node } => match registry.get_pool(pool).and_then(|loaded| loaded.bundle.types.get(node.0 as usize)) {
            Some(entry) => match &entry.form {
                TypeNode::Nominal { .. } => base_constructor_arity(registry, pool, node),
                TypeNode::Applied { origin, arguments } => base_constructor_arity(registry, pool, *origin).saturating_sub(arguments.len()),
                _ => 0,
            },
            None => 0,
        },
    }
}

pub fn display(context: &TypingContextData, registry: &RuntimeTypingRegistry, heap: &Heap, handle: RuntimeTypeRef) -> String {
    display_inner(context, registry, heap, handle, 0)
}

fn display_inner(context: &TypingContextData, registry: &RuntimeTypingRegistry, heap: &Heap, handle: RuntimeTypeRef, depth: usize) -> String {
    if depth >= MAX_DISPLAY_DEPTH {
        return "…".to_string();
    }
    match handle {
        RuntimeTypeRef::Overlay(id) => match context.overlay.type_node(id) {
            Some(RuntimeOverlayTypeNode::Nominal { class }) => heap.class(*class).name.clone(),
            Some(RuntimeOverlayTypeNode::Applied { origin, arguments }) => {
                let args = arguments
                    .iter()
                    .map(|arg| display_inner(context, registry, heap, *arg, depth + 1))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", display_inner(context, registry, heap, *origin, depth + 1), args)
            }
            Some(RuntimeOverlayTypeNode::Union(members)) => members
                .iter()
                .map(|member| display_inner(context, registry, heap, *member, depth + 1))
                .collect::<Vec<_>>()
                .join(" | "),
            Some(RuntimeOverlayTypeNode::Tuple(elements)) => format!(
                "({})",
                elements
                    .iter()
                    .map(|element| display_inner(context, registry, heap, element.ty, depth + 1))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Some(RuntimeOverlayTypeNode::Record(fields)) => format!(
                "{{{}}}",
                fields
                    .iter()
                    .map(|f| format!("{}: {}", f.name, display_inner(context, registry, heap, f.ty, depth + 1)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Some(RuntimeOverlayTypeNode::Callable { parameters, return_type }) => format!(
                "({}) -> {}",
                parameters
                    .iter()
                    .map(|parameter| display_inner(context, registry, heap, parameter.ty, depth + 1))
                    .collect::<Vec<_>>()
                    .join(", "),
                display_inner(context, registry, heap, *return_type, depth + 1)
            ),
            Some(RuntimeOverlayTypeNode::TypeLambda { parameters, body }) => {
                let params = parameters.iter().map(|p| p.name.as_ref()).collect::<Vec<_>>().join(", ");
                format!("<{}> =>> {}", params, display_inner(context, registry, heap, *body, depth + 1))
            }
            Some(RuntimeOverlayTypeNode::Special(name)) => name.to_string(),
            Some(RuntimeOverlayTypeNode::SelfType(_)) => "Self".to_string(),
            None => "<invalid-type>".to_string(),
        },
        RuntimeTypeRef::Base { pool, node } => match registry.get_pool(pool).and_then(|pool| pool.bundle.types.get(node.0 as usize)) {
            Some(entry) => match &entry.form {
                TypeNode::Never => "Never".to_string(),
                TypeNode::Unit => "()".to_string(),
                TypeNode::Nominal { declaration } => declaration.path.last().map_or_else(|| "<nominal>".to_string(), |name| name.to_string()),
                TypeNode::Applied { origin, arguments } => {
                    let origin = RuntimeTypeRef::Base { pool, node: *origin };
                    let args = arguments
                        .iter()
                        .map(|arg| display_inner(context, registry, heap, RuntimeTypeRef::Base { pool, node: *arg }, depth + 1))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}<{}>", display_inner(context, registry, heap, origin, depth + 1), args)
                }
                TypeNode::Union(members) => members
                    .iter()
                    .map(|member| display_inner(context, registry, heap, RuntimeTypeRef::Base { pool, node: *member }, depth + 1))
                    .collect::<Vec<_>>()
                    .join(" | "),
                TypeNode::Tuple(elements) => format!(
                    "({})",
                    elements
                        .iter()
                        .map(|element| display_inner(context, registry, heap, RuntimeTypeRef::Base { pool, node: element.ty }, depth + 1))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                TypeNode::Record(fields) => format!(
                    "{{{}}}",
                    fields
                        .iter()
                        .map(|field| format!(
                            "{}: {}",
                            field.name,
                            display_inner(context, registry, heap, RuntimeTypeRef::Base { pool, node: field.ty }, depth + 1)
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                TypeNode::OpenRecord(open_rec) => format!(
                    "#{{{}, | R{}}}",
                    open_rec
                        .fields
                        .iter()
                        .map(|field| format!(
                            "{}: {}",
                            field.name,
                            display_inner(context, registry, heap, RuntimeTypeRef::Base { pool, node: field.ty }, depth + 1)
                        ))
                        .collect::<Vec<_>>()
                        .join(", "),
                    open_rec.tail.index
                ),
                TypeNode::Callable(callable) => format!(
                    "({}) -> {}",
                    callable
                        .parameters
                        .iter()
                        .map(|param| display_inner(context, registry, heap, RuntimeTypeRef::Base { pool, node: param.ty }, depth + 1))
                        .collect::<Vec<_>>()
                        .join(", "),
                    display_inner(
                        context,
                        registry,
                        heap,
                        RuntimeTypeRef::Base {
                            pool,
                            node: callable.return_type
                        },
                        depth + 1
                    )
                ),
                TypeNode::Parameter(parameter) => format!("T{}", parameter.index),
                TypeNode::SelfType(_) => "Self".to_string(),
                TypeNode::TypeLambda(_) => "<type-lambda>".to_string(),
            },
            None => "<invalid-type>".to_string(),
        },
    }
}

pub fn equivalent(context: &TypingContextData, registry: &RuntimeTypingRegistry, left: RuntimeTypeRef, right: RuntimeTypeRef) -> bool {
    match (left, right) {
        (RuntimeTypeRef::Overlay(left), RuntimeTypeRef::Overlay(right)) => left == right,
        (
            RuntimeTypeRef::Base {
                pool: left_pool,
                node: left_node,
            },
            RuntimeTypeRef::Base {
                pool: right_pool,
                node: right_node,
            },
        ) => {
            let Some(left_pool) = registry.get_pool(left_pool) else { return false };
            let Some(right_pool) = registry.get_pool(right_pool) else { return false };
            left_pool.bundle.types.get(left_node.0 as usize).map(|entry| entry.structural_fingerprint)
                == right_pool.bundle.types.get(right_node.0 as usize).map(|entry| entry.structural_fingerprint)
        }
        _ => false,
    }
}

pub fn nominal_class(context: &TypingContextData, registry: &RuntimeTypingRegistry, handle: RuntimeTypeRef) -> Option<ClassId> {
    match handle {
        RuntimeTypeRef::Overlay(id) => match context.overlay.type_node(id)? {
            RuntimeOverlayTypeNode::Nominal { class } => Some(*class),
            _ => None,
        },
        RuntimeTypeRef::Base { pool, node } => {
            let loaded = registry.get_pool(pool)?;
            let entry = loaded.bundle.types.get(node.0 as usize)?;
            if let TypeNode::Nominal { declaration } = &entry.form {
                registry.resolve_nominal(declaration)
            } else {
                None
            }
        }
    }
}

pub fn subtype(context: &TypingContextData, registry: &RuntimeTypingRegistry, heap: &Heap, left: RuntimeTypeRef, right: RuntimeTypeRef) -> bool {
    if equivalent(context, registry, left, right) {
        return true;
    }
    if let (Some(left_class), Some(right_class)) = (nominal_class(context, registry, left), nominal_class(context, registry, right)) {
        let mut current = heap.class(left_class).superclass;
        while let Some(sup) = current {
            if sup == right_class {
                return true;
            }
            current = heap.class(sup).superclass;
        }
    }
    false
}

pub fn nominal_overlay(context: &mut TypingContextData, class: ClassId) -> RuntimeTypeRef {
    context.overlay.type_ref(RuntimeOverlayTypeNode::Nominal { class })
}

pub fn overlay_id(handle: RuntimeTypeRef) -> Option<RuntimeOverlayTypeId> {
    match handle {
        RuntimeTypeRef::Overlay(id) => Some(id),
        RuntimeTypeRef::Base { .. } => None,
    }
}

pub fn kind_children(context: &TypingContextData, registry: &RuntimeTypingRegistry, handle: RuntimeKindRef) -> Option<Vec<RuntimeKindRef>> {
    match handle {
        RuntimeKindRef::Overlay(id) => match context.overlay.kind_node(id)? {
            RuntimeOverlayKindNode::Type => Some(Vec::new()),
            RuntimeOverlayKindNode::Arrow { parameters, .. } => Some(parameters.to_vec()),
        },
        RuntimeKindRef::Base { pool, node } => match &registry.get_pool(pool)?.bundle.kinds.get(node.0 as usize)?.node {
            phalcom_type_meta::kind::KindNode::Type | phalcom_type_meta::kind::KindNode::RecordRow => Some(Vec::new()),
            phalcom_type_meta::kind::KindNode::Arrow { parameters, .. } => {
                Some(parameters.iter().map(|node| RuntimeKindRef::Base { pool, node: *node }).collect())
            }
        },
    }
}

pub fn kind_result(context: &TypingContextData, registry: &RuntimeTypingRegistry, handle: RuntimeKindRef) -> Option<RuntimeKindRef> {
    match handle {
        RuntimeKindRef::Overlay(id) => match context.overlay.kind_node(id)? {
            RuntimeOverlayKindNode::Type => None,
            RuntimeOverlayKindNode::Arrow { result, .. } => Some(**result),
        },
        RuntimeKindRef::Base { pool, node } => match &registry.get_pool(pool)?.bundle.kinds.get(node.0 as usize)?.node {
            phalcom_type_meta::kind::KindNode::Type | phalcom_type_meta::kind::KindNode::RecordRow => None,
            phalcom_type_meta::kind::KindNode::Arrow { result, .. } => Some(RuntimeKindRef::Base { pool, node: *result }),
        },
    }
}

pub fn kind_display(context: &TypingContextData, registry: &RuntimeTypingRegistry, handle: RuntimeKindRef) -> String {
    match handle {
        RuntimeKindRef::Overlay(id) => match context.overlay.kind_node(id) {
            Some(RuntimeOverlayKindNode::Type) => "Type".to_string(),
            Some(RuntimeOverlayKindNode::Arrow { parameters, result }) => {
                let parameters = parameters
                    .iter()
                    .map(|kind| kind_display(context, registry, *kind))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} -> {}", parameters, kind_display(context, registry, **result))
            }
            None => "<invalid-kind>".to_string(),
        },
        RuntimeKindRef::Base { pool, node } => match registry.get_pool(pool).and_then(|pool| pool.bundle.kinds.get(node.0 as usize)) {
            Some(entry) => match &entry.node {
                phalcom_type_meta::kind::KindNode::Type => "Type".to_string(),
                phalcom_type_meta::kind::KindNode::RecordRow => "RecordRow".to_string(),
                phalcom_type_meta::kind::KindNode::Arrow { parameters, result } => {
                    let parameters = parameters
                        .iter()
                        .map(|node| kind_display(context, registry, RuntimeKindRef::Base { pool, node: *node }))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(
                        "{} -> {}",
                        parameters,
                        kind_display(context, registry, RuntimeKindRef::Base { pool, node: *result })
                    )
                }
            },
            None => "<invalid-kind>".to_string(),
        },
    }
}

pub fn kind_equivalent(_context: &TypingContextData, registry: &RuntimeTypingRegistry, left: RuntimeKindRef, right: RuntimeKindRef) -> bool {
    match (left, right) {
        (RuntimeKindRef::Overlay(left), RuntimeKindRef::Overlay(right)) => left == right,
        (
            RuntimeKindRef::Base {
                pool: left_pool,
                node: left_node,
            },
            RuntimeKindRef::Base {
                pool: right_pool,
                node: right_node,
            },
        ) => {
            let Some(left_pool) = registry.get_pool(left_pool) else { return false };
            let Some(right_pool) = registry.get_pool(right_pool) else { return false };
            left_pool.bundle.kinds.get(left_node.0 as usize).map(|entry| entry.structural_fingerprint)
                == right_pool.bundle.kinds.get(right_node.0 as usize).map(|entry| entry.structural_fingerprint)
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// TypeParameter inspection
// ---------------------------------------------------------------------------

pub fn type_param_record(
    registry: &RuntimeTypingRegistry,
    handle: crate::typing::handle::RuntimeTypeParameterRef,
) -> Option<&phalcom_type_meta::generic::TypeParameterRecord> {
    registry
        .get_pool(handle.pool)
        .and_then(|loaded| loaded.bundle.parameters.get(handle.index as usize))
}

pub fn type_param_name(registry: &RuntimeTypingRegistry, handle: crate::typing::handle::RuntimeTypeParameterRef) -> Option<String> {
    type_param_record(registry, handle).map(|r| r.name.to_string())
}

pub fn type_param_kind(registry: &RuntimeTypingRegistry, handle: crate::typing::handle::RuntimeTypeParameterRef) -> Option<RuntimeKindRef> {
    type_param_record(registry, handle).map(|r| RuntimeKindRef::Base {
        pool: handle.pool,
        node: r.kind,
    })
}

pub fn type_param_variance(registry: &RuntimeTypingRegistry, handle: crate::typing::handle::RuntimeTypeParameterRef) -> Option<Option<&'static str>> {
    let rec = type_param_record(registry, handle)?;
    match rec.id.owner {
        phalcom_type_meta::generic::StableTypeParameterOwnerRef::Declaration(_) => {
            let v = match rec.variance {
                phalcom_type_meta::generic::VarianceRef::Covariant => "covariant",
                phalcom_type_meta::generic::VarianceRef::Contravariant => "contravariant",
                phalcom_type_meta::generic::VarianceRef::Invariant => "invariant",
            };
            Some(Some(v))
        }
        phalcom_type_meta::generic::StableTypeParameterOwnerRef::Callable(_) => Some(None),
    }
}

pub fn type_param_constraints(
    registry: &RuntimeTypingRegistry,
    handle: crate::typing::handle::RuntimeTypeParameterRef,
) -> Vec<crate::typing::handle::RuntimeGenericConstraintRef> {
    let Some(loaded) = registry.get_pool(handle.pool) else { return Vec::new() };
    let Some(param_record) = loaded.bundle.parameters.get(handle.index as usize) else {
        return Vec::new();
    };
    let mut results = Vec::new();
    for (sig_idx, sig) in loaded.bundle.generic_signatures.iter().enumerate() {
        if sig.owner == param_record.id.owner {
            for (c_idx, _c) in sig.constraints.iter().enumerate() {
                results.push(crate::typing::handle::RuntimeGenericConstraintRef {
                    pool: handle.pool,
                    signature: phalcom_type_meta::generic::GenericSignatureRecordId(sig_idx as u32),
                    index: c_idx as u32,
                });
            }
        }
    }
    results
}

// ---------------------------------------------------------------------------
// GenericSignature inspection
// ---------------------------------------------------------------------------

pub fn generic_sig_record(
    registry: &RuntimeTypingRegistry,
    handle: crate::typing::handle::RuntimeGenericSignatureRef,
) -> Option<&phalcom_type_meta::generic::GenericSignatureRecord> {
    registry
        .get_pool(handle.pool)
        .and_then(|loaded| loaded.bundle.generic_signatures.get(handle.id.0 as usize))
}

pub fn generic_sig_parameters(
    registry: &RuntimeTypingRegistry,
    handle: crate::typing::handle::RuntimeGenericSignatureRef,
) -> Vec<crate::typing::handle::RuntimeTypeParameterRef> {
    let Some(loaded) = registry.get_pool(handle.pool) else { return Vec::new() };
    let Some(sig) = loaded.bundle.generic_signatures.get(handle.id.0 as usize) else {
        return Vec::new();
    };
    sig.parameters
        .iter()
        .filter_map(|p_ref| {
            loaded
                .bundle
                .parameters
                .iter()
                .position(|rec| rec.id == *p_ref)
                .map(|idx| crate::typing::handle::RuntimeTypeParameterRef {
                    pool: handle.pool,
                    index: idx as u32,
                })
        })
        .collect()
}

pub fn generic_sig_constraints(
    registry: &RuntimeTypingRegistry,
    handle: crate::typing::handle::RuntimeGenericSignatureRef,
) -> Vec<crate::typing::handle::RuntimeGenericConstraintRef> {
    let Some(sig) = generic_sig_record(registry, handle) else { return Vec::new() };
    (0..sig.constraints.len())
        .map(|idx| crate::typing::handle::RuntimeGenericConstraintRef {
            pool: handle.pool,
            signature: handle.id,
            index: idx as u32,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// GenericConstraint inspection
// ---------------------------------------------------------------------------

pub fn generic_constraint_ref(
    registry: &RuntimeTypingRegistry,
    handle: crate::typing::handle::RuntimeGenericConstraintRef,
) -> Option<&phalcom_type_meta::generic::GenericConstraintRef> {
    let sig = generic_sig_record(
        registry,
        crate::typing::handle::RuntimeGenericSignatureRef {
            pool: handle.pool,
            id: handle.signature,
        },
    )?;
    sig.constraints.get(handle.index as usize)
}

pub fn generic_constraint_relation(registry: &RuntimeTypingRegistry, handle: crate::typing::handle::RuntimeGenericConstraintRef) -> Option<&'static str> {
    match generic_constraint_ref(registry, handle)? {
        phalcom_type_meta::generic::GenericConstraintRef::Subtype { .. } => Some("subtype"),
        phalcom_type_meta::generic::GenericConstraintRef::Equivalent { .. } => Some("equivalent"),
    }
}

pub fn generic_constraint_left(registry: &RuntimeTypingRegistry, handle: crate::typing::handle::RuntimeGenericConstraintRef) -> Option<RuntimeTypeRef> {
    match generic_constraint_ref(registry, handle)? {
        phalcom_type_meta::generic::GenericConstraintRef::Subtype { lower, .. } => Some(RuntimeTypeRef::Base {
            pool: handle.pool,
            node: *lower,
        }),
        phalcom_type_meta::generic::GenericConstraintRef::Equivalent { left, .. } => Some(RuntimeTypeRef::Base {
            pool: handle.pool,
            node: *left,
        }),
    }
}

pub fn generic_constraint_right(registry: &RuntimeTypingRegistry, handle: crate::typing::handle::RuntimeGenericConstraintRef) -> Option<RuntimeTypeRef> {
    match generic_constraint_ref(registry, handle)? {
        phalcom_type_meta::generic::GenericConstraintRef::Subtype { upper, .. } => Some(RuntimeTypeRef::Base {
            pool: handle.pool,
            node: *upper,
        }),
        phalcom_type_meta::generic::GenericConstraintRef::Equivalent { right, .. } => Some(RuntimeTypeRef::Base {
            pool: handle.pool,
            node: *right,
        }),
    }
}

// ---------------------------------------------------------------------------
// CallableSignature inspection
// ---------------------------------------------------------------------------

pub fn callable_record(
    registry: &RuntimeTypingRegistry,
    handle: crate::typing::handle::RuntimeCallableSignatureRef,
) -> Option<&phalcom_type_meta::declaration::CallableSemanticRecord> {
    registry
        .get_pool(handle.pool)
        .and_then(|loaded| loaded.bundle.callables.get(handle.record.0 as usize))
}

pub fn callable_generic_signature(
    registry: &RuntimeTypingRegistry,
    handle: crate::typing::handle::RuntimeCallableSignatureRef,
) -> Option<crate::typing::handle::RuntimeGenericSignatureRef> {
    let rec = callable_record(registry, handle)?;
    rec.generic_signature
        .map(|id| crate::typing::handle::RuntimeGenericSignatureRef { pool: handle.pool, id })
}

pub fn callable_parameters(
    registry: &RuntimeTypingRegistry,
    handle: crate::typing::handle::RuntimeCallableSignatureRef,
) -> Vec<crate::typing::handle::RuntimeCallableParameterRef> {
    let Some(rec) = callable_record(registry, handle) else { return Vec::new() };
    (0..rec.parameters.len())
        .map(|idx| crate::typing::handle::RuntimeCallableParameterRef {
            callable: handle,
            index: idx as u32,
        })
        .collect()
}

pub fn specialize_type(
    context: &TypingContextData,
    registry: &RuntimeTypingRegistry,
    raw_type: RuntimeTypeRef,
    receiver: Option<RuntimeTypeRef>,
) -> RuntimeTypeRef {
    let Some(receiver) = receiver else { return raw_type };
    let receiver_args = arguments(context, registry, receiver).unwrap_or_default();
    if receiver_args.is_empty() {
        return raw_type;
    }

    match raw_type {
        RuntimeTypeRef::Base { pool, node } => {
            if let Some(loaded) = registry.get_pool(pool) {
                if let Some(entry) = loaded.bundle.types.get(node.0 as usize) {
                    if let TypeNode::Parameter(param) = &entry.form {
                        if let Some(arg) = receiver_args.get(param.index as usize) {
                            return *arg;
                        }
                    }
                }
            }
            raw_type
        }
        RuntimeTypeRef::Overlay(_) => raw_type,
    }
}

pub fn callable_parameter_type_at(
    context: &TypingContextData,
    registry: &RuntimeTypingRegistry,
    handle: crate::typing::handle::RuntimeCallableSignatureRef,
    index: usize,
) -> Option<RuntimeTypeRef> {
    let rec = callable_record(registry, handle)?;
    let param = rec.parameters.get(index)?;
    let raw = match &param.ty {
        phalcom_type_meta::declaration::PublishedTypeSlot::Known { form, .. } => RuntimeTypeRef::Base {
            pool: handle.pool,
            node: *form,
        },
        _ => return None,
    };
    Some(specialize_type(context, registry, raw, handle.specialization_receiver))
}

pub fn callable_return_type(
    context: &TypingContextData,
    registry: &RuntimeTypingRegistry,
    handle: crate::typing::handle::RuntimeCallableSignatureRef,
) -> Option<RuntimeTypeRef> {
    let rec = callable_record(registry, handle)?;
    let raw = match &rec.return_type {
        phalcom_type_meta::declaration::PublishedTypeSlot::Known { form, .. } => RuntimeTypeRef::Base {
            pool: handle.pool,
            node: *form,
        },
        _ => return None,
    };
    Some(specialize_type(context, registry, raw, handle.specialization_receiver))
}

// ---------------------------------------------------------------------------
// CallableParameter inspection
// ---------------------------------------------------------------------------

pub fn callable_param_record(
    registry: &RuntimeTypingRegistry,
    handle: crate::typing::handle::RuntimeCallableParameterRef,
) -> Option<&phalcom_type_meta::declaration::CallableParameterRecord> {
    let callable = callable_record(registry, handle.callable)?;
    callable.parameters.get(handle.index as usize)
}

pub fn callable_param_local_name(registry: &RuntimeTypingRegistry, handle: crate::typing::handle::RuntimeCallableParameterRef) -> Option<String> {
    callable_param_record(registry, handle).map(|p| p.local_name.to_string())
}

pub fn callable_param_external_label(registry: &RuntimeTypingRegistry, handle: crate::typing::handle::RuntimeCallableParameterRef) -> Option<Option<String>> {
    callable_param_record(registry, handle).map(|p| p.external_label.as_ref().map(|s| s.to_string()))
}

pub fn callable_param_rest_mode(registry: &RuntimeTypingRegistry, handle: crate::typing::handle::RuntimeCallableParameterRef) -> &'static str {
    match callable_param_record(registry, handle).map(|p| p.rest) {
        Some(phalcom_type_meta::declaration::RestModeRef::None) => "none",
        Some(phalcom_type_meta::declaration::RestModeRef::Anonymous) => "anonymous",
        Some(phalcom_type_meta::declaration::RestModeRef::Named) => "named",
        None => "none",
    }
}

pub fn callable_param_type(
    context: &TypingContextData,
    registry: &RuntimeTypingRegistry,
    handle: crate::typing::handle::RuntimeCallableParameterRef,
) -> Option<RuntimeTypeRef> {
    callable_parameter_type_at(context, registry, handle.callable, handle.index as usize)
}

// ---------------------------------------------------------------------------
// FieldSignature inspection
// ---------------------------------------------------------------------------

pub fn field_record(
    registry: &RuntimeTypingRegistry,
    handle: crate::typing::handle::RuntimeFieldSignatureRef,
) -> Option<&phalcom_type_meta::declaration::FieldSemanticRecord> {
    registry
        .get_pool(handle.pool)
        .and_then(|loaded| loaded.bundle.fields.get(handle.record.0 as usize))
}

pub fn field_name(registry: &RuntimeTypingRegistry, handle: crate::typing::handle::RuntimeFieldSignatureRef) -> Option<String> {
    field_record(registry, handle).map(|f| f.field.name.to_string())
}

pub fn field_mutable(registry: &RuntimeTypingRegistry, handle: crate::typing::handle::RuntimeFieldSignatureRef) -> bool {
    field_record(registry, handle).is_some_and(|f| matches!(f.mutability, phalcom_type_meta::declaration::FieldMutabilityRef::Mutable))
}

pub fn field_type(
    context: &TypingContextData,
    registry: &RuntimeTypingRegistry,
    handle: crate::typing::handle::RuntimeFieldSignatureRef,
) -> Option<RuntimeTypeRef> {
    let rec = field_record(registry, handle)?;
    let raw = match &rec.ty {
        phalcom_type_meta::declaration::PublishedTypeSlot::Known { form, .. } => RuntimeTypeRef::Base {
            pool: handle.pool,
            node: *form,
        },
        _ => return None,
    };
    Some(specialize_type(context, registry, raw, handle.specialization_receiver))
}

// ---------------------------------------------------------------------------
// TypeUse inspection
// ---------------------------------------------------------------------------

pub fn type_use_record(
    registry: &RuntimeTypingRegistry,
    handle: crate::typing::handle::RuntimeTypeUseRef,
) -> Option<&phalcom_type_meta::bundle::TypeUseRecord> {
    registry
        .get_pool(handle.pool)
        .and_then(|loaded| loaded.bundle.occurrences.get(handle.index as usize))
}

pub fn type_use_denotation(registry: &RuntimeTypingRegistry, handle: crate::typing::handle::RuntimeTypeUseRef) -> Option<RuntimeTypeRef> {
    let rec = type_use_record(registry, handle)?;
    match rec.status {
        phalcom_type_meta::bundle::TypeUseStatusRef::Known(node) => Some(RuntimeTypeRef::Base { pool: handle.pool, node }),
        _ => None,
    }
}

pub fn type_use_written(registry: &RuntimeTypingRegistry, handle: crate::typing::handle::RuntimeTypeUseRef) -> Option<String> {
    type_use_record(registry, handle).and_then(|rec| rec.written.as_ref().map(|s| s.to_string()))
}

// ---------------------------------------------------------------------------
// Declaration queries
// ---------------------------------------------------------------------------

pub fn declaration_record_for_class<'a>(
    registry: &'a RuntimeTypingRegistry,
    class_id: ClassId,
) -> Option<(crate::typing::handle::MetadataPoolId, &'a phalcom_type_meta::declaration::DeclarationTypeRecord)> {
    let declaration = registry.declaration_for_nominal(class_id)?;
    for (pool_idx, loaded) in registry.pools().iter().enumerate() {
        for decl in loaded.bundle.declarations.iter() {
            if decl.declaration == *declaration {
                return Some((crate::typing::handle::MetadataPoolId(pool_idx as u32), decl));
            }
        }
    }
    None
}

pub fn generic_signature_of_declaration(
    registry: &RuntimeTypingRegistry,
    class_id: ClassId,
    _heap: &Heap,
) -> Option<crate::typing::handle::RuntimeGenericSignatureRef> {
    let (pool, decl) = declaration_record_for_class(registry, class_id)?;
    decl.generic_signature.map(|id| crate::typing::handle::RuntimeGenericSignatureRef { pool, id })
}

pub fn declared_supertype_of_declaration(registry: &RuntimeTypingRegistry, class_id: ClassId, _heap: &Heap) -> Option<RuntimeTypeRef> {
    let (pool, decl) = declaration_record_for_class(registry, class_id)?;
    decl.superclass_template.map(|node| RuntimeTypeRef::Base { pool, node })
}

// ---------------------------------------------------------------------------
// Member lookup
// ---------------------------------------------------------------------------

pub fn lookup_member(
    context: &TypingContextData,
    registry: &RuntimeTypingRegistry,
    heap: &Heap,
    receiver: RuntimeTypeRef,
    selector: &str,
    is_class_side: bool,
) -> Option<crate::typing::handle::RuntimeCallableSignatureRef> {
    let (nominal_class, specialization_receiver) = match receiver {
        RuntimeTypeRef::Overlay(id) => match context.overlay.type_node(id)? {
            RuntimeOverlayTypeNode::Nominal { class } => (*class, None),
            RuntimeOverlayTypeNode::Applied { origin, .. } => {
                let origin_class = match origin {
                    RuntimeTypeRef::Overlay(origin_id) => match context.overlay.type_node(*origin_id)? {
                        RuntimeOverlayTypeNode::Nominal { class } => *class,
                        _ => return None,
                    },
                    RuntimeTypeRef::Base { pool, node } => {
                        let loaded = registry.get_pool(*pool)?;
                        let entry = loaded.bundle.types.get(node.0 as usize)?;
                        if let TypeNode::Nominal { declaration } = &entry.form {
                            registry.resolve_nominal(declaration)?
                        } else {
                            return None;
                        }
                    }
                };
                (origin_class, Some(receiver))
            }
            _ => return None,
        },
        RuntimeTypeRef::Base { pool, node } => {
            let loaded = registry.get_pool(pool)?;
            let entry = loaded.bundle.types.get(node.0 as usize)?;
            match &entry.form {
                TypeNode::Nominal { declaration } => (registry.resolve_nominal(declaration)?, None),
                TypeNode::Applied { origin, .. } => {
                    let origin_entry = loaded.bundle.types.get(origin.0 as usize)?;
                    if let TypeNode::Nominal { declaration } = &origin_entry.form {
                        (registry.resolve_nominal(declaration)?, Some(receiver))
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }
    };

    let (pool, decl) = declaration_record_for_class(registry, nominal_class)?;
    let callables = if is_class_side { &decl.class_callables } else { &decl.instance_callables };

    let loaded = registry.get_pool(pool)?;
    for (rec_idx, callable_rec) in loaded.bundle.callables.iter().enumerate() {
        if callables.contains(&callable_rec.callable) && callable_rec.callable.selector.as_ref() == selector {
            return Some(crate::typing::handle::RuntimeCallableSignatureRef {
                pool,
                record: phalcom_type_meta::declaration::CallableRecordId(rec_idx as u32),
                specialization_receiver,
            });
        }
    }

    None
}
