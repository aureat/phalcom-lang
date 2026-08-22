//! Small, bounded projections over validated metadata and context overlays.

use crate::heap::{ClassId, Heap};
use crate::typing::context::TypingContextData;
use crate::typing::handle::{RuntimeTypeRef, RuntimeOverlayTypeId};
use crate::typing::overlay::RuntimeOverlayTypeNode;
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

pub fn display(
    context: &TypingContextData,
    registry: &RuntimeTypingRegistry,
    heap: &Heap,
    handle: RuntimeTypeRef,
) -> String {
    display_inner(context, registry, heap, handle, 0)
}

fn display_inner(
    context: &TypingContextData,
    registry: &RuntimeTypingRegistry,
    heap: &Heap,
    handle: RuntimeTypeRef,
    depth: usize,
) -> String {
    if depth >= MAX_DISPLAY_DEPTH {
        return "…".to_string();
    }
    match handle {
        RuntimeTypeRef::Overlay(id) => match context.overlay.type_node(id) {
            Some(RuntimeOverlayTypeNode::Nominal { class }) => heap.class(*class).name.clone(),
            Some(RuntimeOverlayTypeNode::Applied { origin, arguments }) => {
                let args = arguments.iter().map(|arg| display_inner(context, registry, heap, *arg, depth + 1)).collect::<Vec<_>>().join(", ");
                format!("{}<{}>", display_inner(context, registry, heap, *origin, depth + 1), args)
            }
            Some(RuntimeOverlayTypeNode::Union(members)) => members
                .iter()
                .map(|member| display_inner(context, registry, heap, *member, depth + 1))
                .collect::<Vec<_>>()
                .join(" | "),
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
                TypeNode::Tuple(elements) => format!("({})", elements.iter().map(|element| display_inner(context, registry, heap, RuntimeTypeRef::Base { pool, node: element.ty }, depth + 1)).collect::<Vec<_>>().join(", ")),
                TypeNode::Record(fields) => format!("{{{}}}", fields.iter().map(|field| format!("{}: {}", field.name, display_inner(context, registry, heap, RuntimeTypeRef::Base { pool, node: field.ty }, depth + 1))).collect::<Vec<_>>().join(", ")),
                TypeNode::Callable(callable) => format!("({}) -> {}", callable.parameters.iter().map(|param| display_inner(context, registry, heap, RuntimeTypeRef::Base { pool, node: param.ty }, depth + 1)).collect::<Vec<_>>().join(", "), display_inner(context, registry, heap, RuntimeTypeRef::Base { pool, node: callable.return_type }, depth + 1)),
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
        (RuntimeTypeRef::Base { pool: left_pool, node: left_node }, RuntimeTypeRef::Base { pool: right_pool, node: right_node }) => {
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

