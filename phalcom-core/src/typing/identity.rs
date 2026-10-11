//! Exact graph comparison across metadata pools and instantiated overlays.
//!
//! Pool-local node numbers and structural fingerprints are representations, not
//! type identity. Project nodes into their complete shape plus children, then
//! compare those children recursively. No fingerprint is accepted as equality.

use crate::typing::context::TypingContextData;
use crate::typing::handle::{MetadataPoolId, RuntimeKindRef, RuntimeTypeRef};
use crate::typing::overlay::RuntimeOverlayTypeNode;
use crate::typing::registry::RuntimeTypingRegistry;
use phalcom_type_meta::kind::{KindNode, KindNodeId};
use phalcom_type_meta::scoped_type::{ScopedTypeNode, ScopedTypeNodeId};
use phalcom_type_meta::type_node::{CallableParamRef, CallableTypeRef, RecordFieldRef, RecordRowRef, StaticArgumentRef, TupleElementRef, TypeNode, TypeNodeId};
use std::collections::{HashMap, HashSet};

/// Exact identity of two type terms in the same runtime typing context.
pub fn equivalent(context: &TypingContextData, registry: &RuntimeTypingRegistry, left: RuntimeTypeRef, right: RuntimeTypeRef) -> bool {
    Comparison {
        context,
        registry,
        types: HashMap::new(),
    }
    .types(left, right)
}

struct Comparison<'a> {
    context: &'a TypingContextData,
    registry: &'a RuntimeTypingRegistry,
    types: HashMap<(RuntimeTypeRef, RuntimeTypeRef), bool>,
}

#[derive(PartialEq, Eq)]
enum Shape {
    Metadata(TypeNode),
    RuntimeNominal(crate::heap::ClassId),
    RuntimeSpecial(Box<str>),
    RuntimeSelf(Option<crate::heap::ClassId>),
    RuntimeLambda(Box<[Box<str>]>),
}

struct Projection {
    shape: Shape,
    children: Vec<RuntimeTypeRef>,
    kinds: Vec<RuntimeKindRef>,
    scoped: Option<(MetadataPoolId, ScopedTypeNodeId)>,
}

impl Comparison<'_> {
    fn types(&mut self, left: RuntimeTypeRef, right: RuntimeTypeRef) -> bool {
        if let Some(&result) = self.types.get(&(left, right)) {
            return result;
        }
        let result = self.compare_types(left, right);
        self.types.insert((left, right), result);
        result
    }

    fn compare_types(&mut self, left: RuntimeTypeRef, right: RuntimeTypeRef) -> bool {
        let mut pending = vec![(left, right)];
        let mut visited = HashSet::new();
        while let Some((left_type, right_type)) = pending.pop() {
            if !visited.insert((left_type, right_type)) {
                continue;
            }
            if let Some(&result) = self.types.get(&(left_type, right_type)) {
                if !result {
                    return false;
                }
                continue;
            }
            if left_type == right_type {
                let valid = match left_type {
                    RuntimeTypeRef::Base { pool, node } => self
                        .registry
                        .get_pool(pool)
                        .is_some_and(|loaded| loaded.bundle.types.get(node.0 as usize).is_some()),
                    RuntimeTypeRef::Overlay(id) => self.context.overlay.type_node(id).is_some(),
                };
                if !valid {
                    return false;
                }
                continue;
            }
            let Some(left) = self.project(left_type) else { return false };
            let Some(right) = self.project(right_type) else { return false };
            if matches!(left.shape, Shape::Metadata(TypeNode::Union(_))) || matches!(right.shape, Shape::Metadata(TypeNode::Union(_))) {
                let Some(left) = self.union_members(left_type) else { return false };
                let Some(right) = self.union_members(right_type) else { return false };
                if left.len() != right.len() || !left.iter().all(|&left| right.iter().any(|&right| self.types(left, right))) {
                    return false;
                }
                continue;
            }
            if left.shape != right.shape || left.children.len() != right.children.len() || left.kinds.len() != right.kinds.len() {
                return false;
            }
            pending.extend(left.children.into_iter().zip(right.children));
            if !left.kinds.iter().zip(&right.kinds).all(|(&left, &right)| self.kinds(left, right)) {
                return false;
            }
            match (left.scoped, right.scoped) {
                (None, None) => {}
                (Some(left), Some(right)) if self.scoped(left, right) => {}
                _ => return false,
            }
        }
        // Only complete successful comparisons are memoized. A visited pair
        // alone is not proof while its child work is still pending.
        self.types.extend(visited.into_iter().map(|pair| (pair, true)));
        true
    }

    fn union_members(&mut self, ty: RuntimeTypeRef) -> Option<Vec<RuntimeTypeRef>> {
        let mut members = Vec::new();
        let mut pending = vec![ty];
        let mut visited = HashSet::new();
        while let Some(ty) = pending.pop() {
            if !visited.insert(ty) {
                continue;
            }
            let node = self.project(ty)?;
            match node.shape {
                Shape::Metadata(TypeNode::Union(_)) => pending.extend(node.children),
                Shape::Metadata(TypeNode::Never) => {}
                _ => members.push(ty),
            }
        }
        let mut unique = Vec::new();
        for member in members {
            if !unique.iter().any(|&existing| self.types(member, existing)) {
                unique.push(member);
            }
        }
        Some(unique)
    }

    fn project(&self, ty: RuntimeTypeRef) -> Option<Projection> {
        let mut children = Vec::new();
        let mut kinds = Vec::new();
        let mut scoped = None;
        let form = match ty {
            RuntimeTypeRef::Base { pool, node } => {
                let mut form = self.registry.get_pool(pool)?.bundle.types.get(node.0 as usize)?.form.clone();
                if let TypeNode::TypeLambda(lambda) = &mut form {
                    for kind in &mut lambda.parameter_kinds {
                        kinds.push(RuntimeKindRef::Base { pool, node: *kind });
                        *kind = KindNodeId(kinds.len() as u32 - 1);
                    }
                    scoped = Some((pool, lambda.body));
                    lambda.body = ScopedTypeNodeId(0);
                }
                map_type_children(&mut form, |node| {
                    children.push(RuntimeTypeRef::Base { pool, node: *node });
                    *node = TypeNodeId(children.len() as u32 - 1);
                });
                form
            }
            RuntimeTypeRef::Overlay(id) => {
                let mut child = |ty| {
                    children.push(ty);
                    TypeNodeId(children.len() as u32 - 1)
                };
                match self.context.overlay.type_node(id)? {
                    RuntimeOverlayTypeNode::Nominal { class } => {
                        if let Some(declaration) = self.registry.declaration_for_nominal(*class) {
                            TypeNode::Nominal {
                                declaration: declaration.clone(),
                            }
                        } else {
                            return Some(Projection {
                                shape: Shape::RuntimeNominal(*class),
                                children,
                                kinds,
                                scoped,
                            });
                        }
                    }
                    RuntimeOverlayTypeNode::Applied { origin, arguments } => TypeNode::Applied {
                        origin: child(*origin),
                        arguments: arguments
                            .iter()
                            .map(|argument| match argument {
                                crate::typing::overlay::RuntimeStaticArgument::Term(ty) => Some(StaticArgumentRef::Term(child(*ty))),
                                crate::typing::overlay::RuntimeStaticArgument::RecordRow(row) => Some(StaticArgumentRef::RecordRow(RecordRowRef {
                                    fields: row
                                        .fields
                                        .iter()
                                        .map(|field| {
                                            Some(RecordFieldRef {
                                                name: field.key.as_symbol()?.into(),
                                                ty: child(field.ty),
                                                presence: Default::default(),
                                            })
                                        })
                                        .collect::<Option<_>>()?,
                                    tail: row.tail.clone(),
                                    remainder: None,
                                })),
                                crate::typing::overlay::RuntimeStaticArgument::DynamicType => Some(StaticArgumentRef::DynamicType),
                            })
                            .collect::<Option<_>>()?,
                    },
                    RuntimeOverlayTypeNode::Union(members) => TypeNode::Union(members.iter().map(|ty| child(*ty)).collect()),
                    RuntimeOverlayTypeNode::Tuple(elements) => TypeNode::Tuple(
                        elements
                            .iter()
                            .map(|element| TupleElementRef {
                                label: element.label.clone(),
                                ty: child(element.ty),
                                presence: Default::default(),
                            })
                            .collect(),
                    ),
                    RuntimeOverlayTypeNode::Record(fields) => TypeNode::Record(
                        fields
                            .iter()
                            .map(|field| {
                                Some(RecordFieldRef {
                                    name: field.key.as_symbol()?.into(),
                                    ty: child(field.ty),
                                    presence: Default::default(),
                                })
                            })
                            .collect::<Option<_>>()?,
                    ),
                    RuntimeOverlayTypeNode::Callable { parameters, return_type } => TypeNode::Callable(CallableTypeRef {
                        parameters: parameters
                            .iter()
                            .map(|parameter| CallableParamRef {
                                label: parameter.label.clone(),
                                ty: child(parameter.ty),
                                rest: parameter.rest,
                            })
                            .collect(),
                        return_type: child(*return_type),
                    }),
                    RuntimeOverlayTypeNode::TypeLambda { parameters, body } => {
                        // Runtime lambdas have their own binder representation; do
                        // not equate them with metadata's alpha-normalized graph.
                        // The overlay retains names as part of its existing shape.
                        let shape = Shape::RuntimeLambda(parameters.iter().map(|parameter| parameter.name.clone()).collect());
                        children.push(*body);
                        kinds.extend(parameters.iter().map(|parameter| parameter.kind));
                        return Some(Projection {
                            shape,
                            children,
                            kinds,
                            scoped: None,
                        });
                    }
                    RuntimeOverlayTypeNode::Special(name) => {
                        return Some(Projection {
                            shape: Shape::RuntimeSpecial(name.clone()),
                            children,
                            kinds,
                            scoped,
                        });
                    }
                    RuntimeOverlayTypeNode::SelfType(owner) => {
                        return Some(Projection {
                            shape: Shape::RuntimeSelf(*owner),
                            children,
                            kinds,
                            scoped,
                        });
                    }
                }
            }
        };
        // `form` preserves every non-edge field (including stable declaration,
        // parameter/requirement identity, labels, presence, and static values).
        Some(Projection {
            shape: Shape::Metadata(form),
            children,
            kinds,
            scoped,
        })
    }

    fn kinds(&mut self, left: RuntimeKindRef, right: RuntimeKindRef) -> bool {
        let project = |kind| -> Option<(KindNode, Vec<RuntimeKindRef>, Vec<RuntimeTypeRef>)> {
            let mut children = Vec::new();
            let mut types = Vec::new();
            let form = match kind {
                RuntimeKindRef::Base { pool, node } => {
                    let mut form = self.registry.get_pool(pool)?.bundle.kinds.get(node.0 as usize)?.node.clone();
                    if let KindNode::Arrow { parameters, result } = &mut form {
                        for kind in parameters.iter_mut().chain(std::iter::once(result)) {
                            children.push(RuntimeKindRef::Base { pool, node: *kind });
                            *kind = KindNodeId(children.len() as u32 - 1);
                        }
                    }
                    if let KindNode::StaticDomain(phalcom_type_meta::kind::StaticDomainRef::Associated { owner_environment, .. }) = &mut form {
                        map_static_arguments(owner_environment, &mut |node| {
                            types.push(RuntimeTypeRef::Base { pool, node: *node });
                            *node = TypeNodeId(types.len() as u32 - 1);
                        });
                    }
                    form
                }
                RuntimeKindRef::Overlay(id) => match self.context.overlay.kind_node(id)? {
                    crate::typing::overlay::RuntimeOverlayKindNode::Type => KindNode::Type,
                    crate::typing::overlay::RuntimeOverlayKindNode::Arrow { parameters, result } => {
                        children.extend(parameters.iter().copied());
                        children.push(**result);
                        KindNode::Arrow {
                            parameters: (0..parameters.len()).map(|index| KindNodeId(index as u32)).collect(),
                            result: KindNodeId(parameters.len() as u32),
                        }
                    }
                },
            };
            Some((form, children, types))
        };
        let mut pending = vec![(left, right)];
        let mut visited = HashSet::new();
        while let Some((left, right)) = pending.pop() {
            if !visited.insert((left, right)) {
                continue;
            }
            let Some((left, left_children, left_types)) = project(left) else {
                return false;
            };
            let Some((right, right_children, right_types)) = project(right) else {
                return false;
            };
            if left != right || left_children.len() != right_children.len() || left_types.len() != right_types.len() {
                return false;
            }
            // Finish the immutable projections before borrowing the comparison
            // mutably for associated static-domain environments.
            pending.extend(left_children.into_iter().zip(right_children));
            if !left_types.iter().zip(right_types).all(|(&left, right)| self.types(left, right)) {
                return false;
            }
        }
        true
    }

    fn scoped(&mut self, left: (MetadataPoolId, ScopedTypeNodeId), right: (MetadataPoolId, ScopedTypeNodeId)) -> bool {
        let project = |(pool, node): (MetadataPoolId, ScopedTypeNodeId)| {
            let mut form = self.registry.get_pool(pool)?.bundle.scoped_types.get(node.0 as usize)?.form.clone();
            let mut children = Vec::new();
            let mut free = Vec::new();
            let mut kinds = Vec::new();
            map_scoped_children(
                &mut form,
                |node| {
                    children.push((pool, *node));
                    *node = ScopedTypeNodeId(children.len() as u32 - 1);
                },
                |node| {
                    free.push(RuntimeTypeRef::Base { pool, node: *node });
                    *node = TypeNodeId(free.len() as u32 - 1);
                },
                |node| {
                    kinds.push(RuntimeKindRef::Base { pool, node: *node });
                    *node = KindNodeId(kinds.len() as u32 - 1);
                },
            );
            Some((form, children, free, kinds))
        };
        let mut pending = vec![(left, right)];
        let mut visited = HashSet::new();
        while let Some((left, right)) = pending.pop() {
            if !visited.insert((left, right)) {
                continue;
            }
            let Some((left, left_children, left_free, left_kinds)) = project(left) else {
                return false;
            };
            let Some((right, right_children, right_free, right_kinds)) = project(right) else {
                return false;
            };
            if left != right || left_children.len() != right_children.len() || left_free.len() != right_free.len() || left_kinds.len() != right_kinds.len() {
                return false;
            }
            if matches!(left, ScopedTypeNode::Union(_)) {
                if !left_children.iter().all(|&left| right_children.iter().any(|&right| self.scoped(left, right)))
                    || !right_children.iter().all(|&right| left_children.iter().any(|&left| self.scoped(left, right)))
                {
                    return false;
                }
            } else {
                pending.extend(left_children.into_iter().zip(right_children));
            }
            if !left_free.iter().zip(right_free).all(|(&left, right)| self.types(left, right))
                || !left_kinds.iter().zip(right_kinds).all(|(&left, right)| self.kinds(left, right))
            {
                return false;
            }
        }
        true
    }
}

/// Replaces only graph edges; all scalar and stable identity fields survive.
fn map_type_children(form: &mut TypeNode, mut map: impl FnMut(&mut TypeNodeId)) {
    use phalcom_type_meta::type_node::{AnonymousRecordRemainderRef as RecordRest, AnonymousTupleRemainderRef as TupleRest};
    match form {
        TypeNode::Never
        | TypeNode::Unit
        | TypeNode::ConstInt(_)
        | TypeNode::Nominal { .. }
        | TypeNode::Parameter(_)
        | TypeNode::SelfType(_)
        | TypeNode::TypeLambda(_) => {}
        TypeNode::StaticData(data) => {
            if let phalcom_type_meta::kind::StaticDomainRef::Associated { owner_environment, .. } = &mut data.domain {
                map_static_arguments(owner_environment, &mut map);
            }
            map_static_arguments(&mut data.arguments, &mut map);
        }
        TypeNode::Applied { origin, arguments } => {
            map(origin);
            map_static_arguments(arguments, &mut map);
        }
        TypeNode::Union(nodes) | TypeNode::TypePack(nodes) | TypeNode::TypePackConcat(nodes) | TypeNode::LabeledTypePackConcat(nodes) => {
            for node in nodes {
                map(node);
            }
        }
        TypeNode::Tuple(elements) => {
            for element in elements {
                map(&mut element.ty);
            }
        }
        TypeNode::Record(fields) => {
            for field in fields {
                map(&mut field.ty);
            }
        }
        TypeNode::Callable(callable) => {
            for parameter in &mut callable.parameters {
                map(&mut parameter.ty);
            }
            map(&mut callable.return_type);
        }
        TypeNode::OpenRecord(record) => {
            for field in &mut record.fields {
                map(&mut field.ty);
            }
        }
        TypeNode::AssociatedProjection {
            subject, trait_application, ..
        } => {
            map(subject);
            for argument in &mut trait_application.arguments {
                map(argument);
            }
        }
        TypeNode::LabeledTypePack(elements) => {
            for element in elements {
                map(&mut element.element);
            }
        }
        TypeNode::CompleteTypePack { positionals, labeled }
        | TypeNode::TupleFromPacks { positionals, labeled }
        | TypeNode::PackFromPacks { positionals, labeled } => {
            map(positionals);
            map(labeled);
        }
        TypeNode::CallableFromPacks {
            positionals,
            labeled,
            return_type,
        } => {
            map(positionals);
            map(labeled);
            map(return_type);
        }
        TypeNode::TypeMemberProjection(projection) => {
            map(&mut projection.subject);
            for argument in &mut projection.arguments {
                if let phalcom_type_meta::type_node::TypeMemberArgumentRef::Type(node) = argument {
                    map(node);
                }
            }
        }
        TypeNode::OpenTuple(tuple) => {
            for element in &mut tuple.elements {
                map(&mut element.ty);
            }
            for rest in [&mut tuple.positional_remainder, &mut tuple.labeled_remainder] {
                if let TupleRest::Typed(node) = rest {
                    map(node);
                }
            }
        }
        TypeNode::AnonymousRecord(record) => {
            for field in &mut record.fields {
                map(&mut field.ty);
            }
            if let RecordRest::Typed(node) = &mut record.remainder {
                map(node);
            }
        }
        TypeNode::Pack(pack) => {
            for element in &mut pack.elements {
                map(&mut element.ty);
            }
            for rest in [&mut pack.positional_remainder, &mut pack.labeled_remainder] {
                if let TupleRest::Typed(node) = rest {
                    map(node);
                }
            }
        }
    }
}

fn map_static_arguments(arguments: &mut [StaticArgumentRef], map: &mut impl FnMut(&mut TypeNodeId)) {
    for argument in arguments {
        match argument {
            StaticArgumentRef::Term(node) => map(node),
            StaticArgumentRef::RecordRow(row) => {
                for field in &mut row.fields {
                    map(&mut field.ty);
                }
                if let Some(phalcom_type_meta::type_node::AnonymousRecordRemainderRef::Typed(node)) = &mut row.remainder {
                    map(node);
                }
            }
            StaticArgumentRef::DynamicType => {}
        }
    }
}

fn map_scoped_children(
    form: &mut ScopedTypeNode,
    mut map: impl FnMut(&mut ScopedTypeNodeId),
    mut free: impl FnMut(&mut TypeNodeId),
    mut kind: impl FnMut(&mut KindNodeId),
) {
    use phalcom_type_meta::scoped_type::{ScopedAnonymousRecordRemainderRef as RecordRest, ScopedAnonymousTupleRemainderRef as TupleRest};
    match form {
        ScopedTypeNode::Bound { .. } => {}
        ScopedTypeNode::Free(node) => free(node),
        ScopedTypeNode::Applied { origin, arguments } => {
            map(origin);
            for argument in arguments {
                map(argument);
            }
        }
        ScopedTypeNode::Union(nodes)
        | ScopedTypeNode::TypePack(nodes)
        | ScopedTypeNode::TypePackConcat(nodes)
        | ScopedTypeNode::LabeledTypePackConcat(nodes) => {
            for node in nodes {
                map(node);
            }
        }
        ScopedTypeNode::Tuple(elements) => {
            for element in elements {
                map(&mut element.ty);
            }
        }
        ScopedTypeNode::Record(fields) => {
            for field in fields {
                map(&mut field.ty);
            }
        }
        ScopedTypeNode::Callable(callable) => {
            for parameter in &mut callable.parameters {
                map(&mut parameter.ty);
            }
            map(&mut callable.return_type);
        }
        ScopedTypeNode::Lambda { parameter_kinds, body, .. } => {
            for node in parameter_kinds {
                kind(node);
            }
            map(body);
        }
        ScopedTypeNode::OpenRecord(record) => {
            for field in &mut record.fields {
                map(&mut field.ty);
            }
            if let phalcom_type_meta::scoped_type::ScopedRecordTailRef::Anonymous(RecordRest::Typed(node)) = &mut record.tail {
                map(node);
            }
        }
        ScopedTypeNode::RecordRow(row) => {
            for field in &mut row.fields {
                map(&mut field.ty);
            }
            match &mut row.tail {
                phalcom_type_meta::scoped_type::ScopedRecordRowTailRef::Projection(node) => free(node),
                phalcom_type_meta::scoped_type::ScopedRecordRowTailRef::Anonymous(RecordRest::Typed(node)) => map(node),
                _ => {}
            }
        }
        ScopedTypeNode::LabeledTypePack(elements) => {
            for element in elements {
                map(&mut element.element);
            }
        }
        ScopedTypeNode::CompleteTypePack { positionals, labeled }
        | ScopedTypeNode::TupleFromPacks { positionals, labeled }
        | ScopedTypeNode::PackFromPacks { positionals, labeled } => {
            map(positionals);
            map(labeled);
        }
        ScopedTypeNode::CallableFromPacks {
            positionals,
            labeled,
            return_type,
        } => {
            map(positionals);
            map(labeled);
            map(return_type);
        }
        ScopedTypeNode::PackProjection { subject, .. } => map(subject),
        ScopedTypeNode::OpenTuple(tuple) => {
            for element in &mut tuple.elements {
                map(&mut element.ty);
            }
            for rest in [&mut tuple.positional_remainder, &mut tuple.labeled_remainder] {
                if let TupleRest::Typed(node) = rest {
                    map(node);
                }
            }
        }
        ScopedTypeNode::AnonymousRecord(record) => {
            for field in &mut record.fields {
                map(&mut field.ty);
            }
            if let RecordRest::Typed(node) = &mut record.remainder {
                map(node);
            }
        }
        ScopedTypeNode::Pack(pack) => {
            for element in &mut pack.elements {
                map(&mut element.ty);
            }
            for rest in [&mut pack.positional_remainder, &mut pack.labeled_remainder] {
                if let TupleRest::Typed(node) = rest {
                    map(node);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typing::loader::LoadedSemanticMetadata;
    use crate::typing::overlay::RuntimeStaticArgument;
    use phalcom_type_meta::bundle::SemanticMetadataBundle;
    use phalcom_type_meta::fingerprint::Fingerprint128;
    use phalcom_type_meta::header::*;
    use phalcom_type_meta::identity::{StableDeclarationRef, StableModuleRef, StableProjectRef};
    use phalcom_type_meta::kind::KindNodeEntry;
    use phalcom_type_meta::type_node::{StructuralMemberPresenceRef, TypeNodeEntry};
    use std::sync::Arc;

    fn nominal(module: &str, name: &str) -> TypeNode {
        TypeNode::Nominal {
            declaration: StableDeclarationRef {
                module: StableModuleRef {
                    project: StableProjectRef::Builtin {
                        namespace: "identity-test".into(),
                        version: "1".into(),
                    },
                    path: Box::new([module.into()]),
                },
                path: Box::new([name.into()]),
            },
        }
    }

    fn pool(registry: &mut RuntimeTypingRegistry, types: Vec<TypeNode>) -> MetadataPoolId {
        let id = MetadataPoolId(registry.pool_count() as u32);
        let bundle = SemanticMetadataBundle {
            header: SemanticMetadataHeader {
                schema_version: TYPE_METADATA_SCHEMA_VERSION,
                semantic_model_version: SEMANTIC_MODEL_VERSION,
                producer: ProducerIdentity("identity-test".into()),
                producer_version: "1".into(),
                native_surface_schema_version: NATIVE_SURFACE_SCHEMA_VERSION,
                profile: MetadataProfile::RuntimePublic,
                features: MetadataFeatures::default(),
                identity_scheme: ArtifactIdentityScheme::V1Standard,
                source_fingerprint: Fingerprint128::ZERO,
                interface_fingerprint: Fingerprint128::ZERO,
            },
            kinds: Box::new([KindNodeEntry {
                node: KindNode::Type,
                structural_fingerprint: Fingerprint128::ZERO,
            }]),
            // Deliberately collide every fingerprint: equality must inspect the
            // full graph, including stable owners and static arguments.
            types: types
                .into_iter()
                .map(|form| TypeNodeEntry {
                    kind: KindNodeId(0),
                    form,
                    structural_fingerprint: Fingerprint128::ZERO,
                })
                .collect(),
            scoped_types: Box::new([]),
            parameters: Box::new([]),
            generic_signatures: Box::new([]),
            declarations: Box::new([]),
            static_domains: Box::new([]),
            aliases: Box::new([]),
            callables: Box::new([]),
            fields: Box::new([]),
            module_roots: Box::new([]),
            runtime_roots: Box::new([]),
            occurrences: Box::new([]),
            extensions: Box::new([]),
        };
        registry.register_pool(LoadedSemanticMetadata::new(id, Arc::new(bundle)))
    }

    fn base(pool: MetadataPoolId, node: u32) -> RuntimeTypeRef {
        RuntimeTypeRef::Base { pool, node: TypeNodeId(node) }
    }

    #[test]
    fn cross_pool_and_overlay_identity_compares_nested_static_arguments_despite_fingerprint_collisions() {
        let mut registry = RuntimeTypingRegistry::new();
        let left = pool(
            &mut registry,
            vec![
                nominal("left", "Box"),
                nominal("scalar", "Int"),
                TypeNode::Applied {
                    origin: TypeNodeId(0),
                    arguments: Box::new([StaticArgumentRef::Term(TypeNodeId(1))]),
                },
                nominal("right", "Box"),
                TypeNode::ConstInt(phalcom_type_meta::scalar::StaticInt::Small(1)),
                TypeNode::ConstInt(phalcom_type_meta::scalar::StaticInt::Small(2)),
            ],
        );
        let right = pool(
            &mut registry,
            vec![
                TypeNode::Applied {
                    origin: TypeNodeId(2),
                    arguments: Box::new([StaticArgumentRef::Term(TypeNodeId(1))]),
                },
                nominal("scalar", "Int"),
                nominal("left", "Box"),
            ],
        );
        let mut context = TypingContextData::new(Box::new([]));
        let overlay = context.overlay.type_ref(RuntimeOverlayTypeNode::Applied {
            origin: base(left, 0),
            arguments: Box::new([RuntimeStaticArgument::Term(base(right, 1))]),
        });
        assert!(equivalent(&context, &registry, base(left, 2), base(right, 0)));
        assert!(equivalent(&context, &registry, base(left, 2), overlay));
        assert!(equivalent(&context, &registry, overlay, base(right, 0)));
        assert!(!equivalent(&context, &registry, base(left, 0), base(left, 3)));
        assert!(!equivalent(&context, &registry, base(left, 4), base(left, 5)));
        assert!(!equivalent(&context, &registry, base(left, 999), base(right, 999)));
        assert!(!equivalent(&context, &registry, base(left, 999), base(left, 999)));
    }

    #[test]
    fn structural_argument_identity_preserves_labels_presence_and_callable_shape() {
        let mut registry = RuntimeTypingRegistry::new();
        let field = |label: Option<&str>, presence| TupleElementRef {
            label: label.map(Into::into),
            ty: TypeNodeId(0),
            presence,
        };
        let callable = |rest, result| {
            TypeNode::Callable(CallableTypeRef {
                parameters: Box::new([CallableParamRef {
                    label: Some("value".into()),
                    ty: TypeNodeId(0),
                    rest,
                }]),
                return_type: TypeNodeId(result),
            })
        };
        let pool = pool(
            &mut registry,
            vec![
                nominal("scalar", "Int"),
                nominal("scalar", "Float"),
                TypeNode::Tuple(Box::new([field(None, StructuralMemberPresenceRef::Required)])),
                TypeNode::Tuple(Box::new([field(Some("value"), StructuralMemberPresenceRef::Required)])),
                TypeNode::Tuple(Box::new([field(None, StructuralMemberPresenceRef::Optional)])),
                callable(false, 0),
                callable(true, 0),
                callable(false, 1),
            ],
        );
        let mut context = TypingContextData::new(Box::new([]));
        let tuple = context
            .overlay
            .type_ref(RuntimeOverlayTypeNode::Tuple(Box::new([crate::typing::overlay::RuntimeTupleElement {
                label: None,
                ty: base(pool, 0),
            }])));
        let function = context.overlay.type_ref(RuntimeOverlayTypeNode::Callable {
            parameters: Box::new([crate::typing::overlay::RuntimeCallableParameter {
                label: Some("value".into()),
                ty: base(pool, 0),
                rest: false,
            }]),
            return_type: base(pool, 0),
        });
        assert!(equivalent(&context, &registry, tuple, base(pool, 2)));
        assert!(!equivalent(&context, &registry, tuple, base(pool, 3)));
        assert!(!equivalent(&context, &registry, tuple, base(pool, 4)));
        assert!(equivalent(&context, &registry, function, base(pool, 5)));
        assert!(!equivalent(&context, &registry, function, base(pool, 6)));
        assert!(!equivalent(&context, &registry, function, base(pool, 7)));
    }

    #[test]
    fn union_identity_ignores_pool_local_order_and_instantiation_duplicates() {
        let mut registry = RuntimeTypingRegistry::new();
        let pool = pool(
            &mut registry,
            vec![
                nominal("scalar", "Int"),
                nominal("scalar", "Float"),
                TypeNode::Union(Box::new([TypeNodeId(0), TypeNodeId(1)])),
                TypeNode::Union(Box::new([TypeNodeId(1), TypeNodeId(0)])),
                TypeNode::Never,
            ],
        );
        let mut context = TypingContextData::new(Box::new([]));
        let duplicate = context
            .overlay
            .type_ref(RuntimeOverlayTypeNode::Union(Box::new([base(pool, 0), base(pool, 0)])));
        let nested = context
            .overlay
            .type_ref(RuntimeOverlayTypeNode::Union(Box::new([base(pool, 3), duplicate, base(pool, 4)])));
        assert!(equivalent(&context, &registry, base(pool, 2), base(pool, 3)));
        assert!(equivalent(&context, &registry, base(pool, 0), duplicate));
        assert!(equivalent(&context, &registry, base(pool, 2), nested));
        assert!(!equivalent(&context, &registry, base(pool, 2), duplicate));
    }

    #[test]
    fn exact_metadata_identity_is_not_limited_by_display_depth() {
        let mut registry = RuntimeTypingRegistry::new();
        let mut types = vec![nominal("owner", "Box"), nominal("scalar", "Int")];
        for child in 1..100 {
            types.push(TypeNode::Applied {
                origin: TypeNodeId(0),
                arguments: Box::new([StaticArgumentRef::Term(TypeNodeId(child))]),
            });
        }
        let left = pool(&mut registry, types.clone());
        let right = pool(&mut registry, types);
        let context = TypingContextData::new(Box::new([]));
        assert!(equivalent(&context, &registry, base(left, 100), base(right, 100)));
    }

    #[test]
    fn scoped_lambda_identity_preserves_binders_free_types_and_parameter_kinds() {
        use phalcom_type_meta::scoped_type::{ScopedTypeNodeEntry, TypeLambdaRef};
        let mut registry = RuntimeTypingRegistry::new();
        let lambda = |kind, body| {
            TypeNode::TypeLambda(TypeLambdaRef {
                parameter_kinds: Box::new([KindNodeId(kind)]),
                parameter_modes: None,
                body: ScopedTypeNodeId(body),
            })
        };
        let seed = pool(
            &mut registry,
            vec![nominal("scalar", "Int"), nominal("scalar", "Float"), lambda(0, 0), lambda(0, 1), lambda(1, 0)],
        );
        let mut bundle = (*registry.get_pool(seed).unwrap().bundle).clone();
        bundle.kinds = Box::new([
            KindNodeEntry {
                node: KindNode::Type,
                structural_fingerprint: Fingerprint128::ZERO,
            },
            KindNodeEntry {
                node: KindNode::RecordRow,
                structural_fingerprint: Fingerprint128::ZERO,
            },
        ]);
        let entry = |form| ScopedTypeNodeEntry {
            kind: KindNodeId(0),
            form,
            structural_fingerprint: Fingerprint128::ZERO,
        };
        bundle.scoped_types = Box::new([entry(ScopedTypeNode::Bound { depth: 0, index: 0 }), entry(ScopedTypeNode::Free(TypeNodeId(0)))]);
        let left = registry.register_pool(LoadedSemanticMetadata::new(
            MetadataPoolId(registry.pool_count() as u32),
            Arc::new(bundle.clone()),
        ));
        // Independently numbered global and scoped graphs still denote the same lambdas.
        bundle.types.swap(0, 1);
        bundle.types[2].form = lambda(0, 1);
        bundle.types[3].form = lambda(0, 2);
        bundle.scoped_types = Box::new([
            entry(ScopedTypeNode::Free(TypeNodeId(0))),
            entry(ScopedTypeNode::Bound { depth: 0, index: 0 }),
            entry(ScopedTypeNode::Free(TypeNodeId(1))),
        ]);
        let right = registry.register_pool(LoadedSemanticMetadata::new(MetadataPoolId(registry.pool_count() as u32), Arc::new(bundle)));
        let context = TypingContextData::new(Box::new([]));
        assert!(equivalent(&context, &registry, base(left, 2), base(right, 2)));
        assert!(equivalent(&context, &registry, base(left, 3), base(right, 3)));
        assert!(!equivalent(&context, &registry, base(left, 2), base(right, 3)));
        assert!(!equivalent(&context, &registry, base(left, 2), base(left, 4)));
    }
}
