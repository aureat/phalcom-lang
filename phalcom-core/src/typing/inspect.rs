//! Small, bounded projections over validated metadata and context overlays.

use crate::heap::{ClassId, Heap};
use crate::typing::context::TypingContextData;
use crate::typing::handle::{RuntimeKindRef, RuntimeOverlayTypeId, RuntimeTypeRef};
use crate::typing::overlay::{RuntimeOverlayKindNode, RuntimeOverlayTypeNode};
use crate::typing::registry::RuntimeTypingRegistry;
use phalcom_type_meta::type_node::TypeNode;

const MAX_DISPLAY_DEPTH: usize = 64;

pub fn overlay_node<'a>(context: &'a TypingContextData, handle: RuntimeTypeRef) -> Option<&'a RuntimeOverlayTypeNode> {
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
            RuntimeOverlayTypeNode::Callable { parameters, return_type } => {
                let mut children = parameters.iter().map(|parameter| parameter.ty).collect::<Vec<_>>();
                children.push(*return_type);
                Some(children)
            }
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
            RuntimeOverlayTypeNode::Callable { parameters, .. } => Some(parameters.iter().map(|parameter| parameter.ty).collect()),
        },
        RuntimeTypeRef::Base { pool, node } => match &registry.get_pool(pool)?.bundle.types.get(node.0 as usize)?.form {
            TypeNode::Applied { arguments, .. } => Some(arguments.iter().map(|node| RuntimeTypeRef::Base { pool, node: *node }).collect()),
            TypeNode::Union(members) => Some(members.iter().map(|node| RuntimeTypeRef::Base { pool, node: *node }).collect()),
            _ => Some(Vec::new()),
        },
    }
}

fn class_constructor_arity(heap: &Heap, class: ClassId) -> usize {
    match heap.class(class).name.as_str() {
        "List" | "Set" => 1,
        "Map" => 2,
        _ => 0,
    }
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

pub fn remaining_parameter_count(context: &TypingContextData, registry: &RuntimeTypingRegistry, heap: &Heap, handle: RuntimeTypeRef) -> usize {
    match handle {
        RuntimeTypeRef::Overlay(id) => match context.overlay.type_node(id) {
            Some(RuntimeOverlayTypeNode::Nominal { class }) => class_constructor_arity(heap, *class),
            Some(RuntimeOverlayTypeNode::Applied { origin, arguments }) => {
                let arity = match origin {
                    RuntimeTypeRef::Overlay(origin_id) => match context.overlay.type_node(*origin_id) {
                        Some(RuntimeOverlayTypeNode::Nominal { class }) => class_constructor_arity(heap, *class),
                        Some(RuntimeOverlayTypeNode::Applied { .. }) => remaining_parameter_count(context, registry, heap, *origin) + arguments.len(),
                        _ => 0,
                    },
                    RuntimeTypeRef::Base { pool, node } => base_constructor_arity(registry, *pool, *node),
                };
                arity.saturating_sub(arguments.len())
            }
            Some(RuntimeOverlayTypeNode::Union(_)) | Some(RuntimeOverlayTypeNode::Tuple(_)) | Some(RuntimeOverlayTypeNode::Callable { .. }) | None => 0,
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
            Some(RuntimeOverlayTypeNode::Callable { parameters, return_type }) => format!(
                "({}) -> {}",
                parameters
                    .iter()
                    .map(|parameter| display_inner(context, registry, heap, parameter.ty, depth + 1))
                    .collect::<Vec<_>>()
                    .join(", "),
                display_inner(context, registry, heap, *return_type, depth + 1)
            ),
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
            phalcom_type_meta::kind::KindNode::Type => Some(Vec::new()),
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
            phalcom_type_meta::kind::KindNode::Type => None,
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
