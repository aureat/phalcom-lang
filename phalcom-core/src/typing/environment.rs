//! Runtime type recipe and compact environment registry (LANG005.C1.P3 Task 21).

use crate::typing::handle::RuntimeTypeRef;
use phalcom_type_meta::StableTypeParameterRef;
use std::collections::HashMap;

/// A runtime type recipe representing either a closed type or a generic template type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RuntimeTypeRecipe {
    Closed(RuntimeTypeRef),
    Template(RuntimeTypeRef),
}

/// Compact identifier for an interned runtime type environment.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeTypeEnvironmentId(pub u32);

impl RuntimeTypeEnvironmentId {
    pub const EMPTY: Self = Self(0);
}

/// A compact, immutable generic substitution environment.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RuntimeTypeEnvironment {
    /// Canonical parameter -> runtime type bindings, kept sorted by parameter identity.
    pub bindings: Box<[(StableTypeParameterRef, RuntimeTypeRef)]>,
}

impl RuntimeTypeEnvironment {
    pub fn new(mut bindings: Vec<(StableTypeParameterRef, RuntimeTypeRef)>) -> Self {
        bindings.sort_by(|(a, _), (b, _)| a.cmp(b));
        Self {
            bindings: bindings.into_boxed_slice(),
        }
    }

    pub fn get(&self, param: &StableTypeParameterRef) -> Option<RuntimeTypeRef> {
        self.bindings
            .binary_search_by(|(p, _)| p.cmp(param))
            .ok()
            .map(|idx| self.bindings[idx].1)
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}

/// VM-owned registry for interned runtime type environments.
#[derive(Clone, Debug)]
pub struct RuntimeTypeEnvironmentRegistry {
    environments: Vec<RuntimeTypeEnvironment>,
    interner: HashMap<RuntimeTypeEnvironment, RuntimeTypeEnvironmentId>,
}

impl Default for RuntimeTypeEnvironmentRegistry {
    fn default() -> Self {
        let empty_env = RuntimeTypeEnvironment {
            bindings: Box::new([]),
        };
        let mut interner = HashMap::new();
        interner.insert(empty_env.clone(), RuntimeTypeEnvironmentId::EMPTY);
        Self {
            environments: vec![empty_env],
            interner,
        }
    }
}

impl RuntimeTypeEnvironmentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&mut self, env: RuntimeTypeEnvironment) -> RuntimeTypeEnvironmentId {
        if let Some(&id) = self.interner.get(&env) {
            return id;
        }
        let id = RuntimeTypeEnvironmentId(self.environments.len() as u32);
        self.environments.push(env.clone());
        self.interner.insert(env, id);
        id
    }

    pub fn get(&self, id: RuntimeTypeEnvironmentId) -> Option<&RuntimeTypeEnvironment> {
        self.environments.get(id.0 as usize)
    }
}

/// Instantiates a `RuntimeTypeRecipe` against a `RuntimeTypeEnvironment`, returning a closed `RuntimeTypeRef`.
pub fn instantiate_type_recipe(
    recipe: RuntimeTypeRecipe,
    env: &RuntimeTypeEnvironment,
    context: &mut crate::typing::context::TypingContextData,
    registry: &crate::typing::registry::RuntimeTypingRegistry,
) -> Option<RuntimeTypeRef> {
    match recipe {
        RuntimeTypeRecipe::Closed(ty) => Some(ty),
        RuntimeTypeRecipe::Template(ty) => {
            if env.is_empty() {
                Some(ty)
            } else {
                instantiate_type_ref(ty, env, context, registry, 0)
            }
        }
    }
}

fn instantiate_type_ref(
    ty: RuntimeTypeRef,
    env: &RuntimeTypeEnvironment,
    context: &mut crate::typing::context::TypingContextData,
    registry: &crate::typing::registry::RuntimeTypingRegistry,
    depth: usize,
) -> Option<RuntimeTypeRef> {
    if depth > 64 {
        return None;
    }
    match ty {
        RuntimeTypeRef::Overlay(id) => {
            let node = context.overlay.type_node(id)?.clone();
            match node {
                crate::typing::overlay::RuntimeOverlayTypeNode::Nominal { .. } => Some(ty),
                crate::typing::overlay::RuntimeOverlayTypeNode::Applied { origin, arguments } => {
                    let new_origin = instantiate_type_ref(origin, env, context, registry, depth + 1)?;
                    let mut new_args = Vec::with_capacity(arguments.len());
                    for arg in arguments.iter() {
                        new_args.push(instantiate_type_ref(*arg, env, context, registry, depth + 1)?);
                    }
                    Some(context.overlay.type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Applied {
                        origin: new_origin,
                        arguments: new_args.into_boxed_slice(),
                    }))
                }
                crate::typing::overlay::RuntimeOverlayTypeNode::Union(members) => {
                    let mut new_members = Vec::with_capacity(members.len());
                    for m in members.iter() {
                        new_members.push(instantiate_type_ref(*m, env, context, registry, depth + 1)?);
                    }
                    Some(context.overlay.type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Union(
                        new_members.into_boxed_slice(),
                    )))
                }
                crate::typing::overlay::RuntimeOverlayTypeNode::Tuple(elements) => {
                    let mut new_elements = Vec::with_capacity(elements.len());
                    for el in elements.iter() {
                        let new_ty = instantiate_type_ref(el.ty, env, context, registry, depth + 1)?;
                        new_elements.push(crate::typing::overlay::RuntimeTupleElement {
                            label: el.label.clone(),
                            ty: new_ty,
                        });
                    }
                    Some(context.overlay.type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Tuple(
                        new_elements.into_boxed_slice(),
                    )))
                }
                crate::typing::overlay::RuntimeOverlayTypeNode::Record(fields) => {
                    let mut new_fields = Vec::with_capacity(fields.len());
                    for f in fields.iter() {
                        let new_ty = instantiate_type_ref(f.ty, env, context, registry, depth + 1)?;
                        new_fields.push(crate::typing::overlay::RuntimeRecordField {
                            name: f.name.clone(),
                            ty: new_ty,
                        });
                    }
                    Some(context.overlay.type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Record(
                        new_fields.into_boxed_slice(),
                    )))
                }
                crate::typing::overlay::RuntimeOverlayTypeNode::Callable { parameters, return_type } => {
                    let mut new_params = Vec::with_capacity(parameters.len());
                    for p in parameters.iter() {
                        let new_ty = instantiate_type_ref(p.ty, env, context, registry, depth + 1)?;
                        new_params.push(crate::typing::overlay::RuntimeCallableParameter {
                            label: p.label.clone(),
                            ty: new_ty,
                            rest: p.rest,
                        });
                    }
                    let new_return = instantiate_type_ref(return_type, env, context, registry, depth + 1)?;
                    Some(context.overlay.type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Callable {
                        parameters: new_params.into_boxed_slice(),
                        return_type: new_return,
                    }))
                }
                crate::typing::overlay::RuntimeOverlayTypeNode::TypeLambda { .. }
                | crate::typing::overlay::RuntimeOverlayTypeNode::Special(_)
                | crate::typing::overlay::RuntimeOverlayTypeNode::SelfType(_) => Some(ty),
            }
        }
        RuntimeTypeRef::Base { pool, node } => {
            let loaded = registry.get_pool(pool)?;
            let entry = loaded.bundle.types.get(node.0 as usize)?;
            match &entry.form {
                phalcom_type_meta::type_node::TypeNode::Parameter(param) => {
                    if let Some(subst) = env.get(param) {
                        Some(subst)
                    } else {
                        Some(ty)
                    }
                }
                phalcom_type_meta::type_node::TypeNode::Nominal { .. }
                | phalcom_type_meta::type_node::TypeNode::Never
                | phalcom_type_meta::type_node::TypeNode::Unit => Some(ty),
                phalcom_type_meta::type_node::TypeNode::Applied { origin, arguments } => {
                    let new_origin = instantiate_type_ref(RuntimeTypeRef::Base { pool, node: *origin }, env, context, registry, depth + 1)?;
                    let mut new_args = Vec::with_capacity(arguments.len());
                    for arg in arguments.iter() {
                        new_args.push(instantiate_type_ref(RuntimeTypeRef::Base { pool, node: *arg }, env, context, registry, depth + 1)?);
                    }
                    Some(context.overlay.type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Applied {
                        origin: new_origin,
                        arguments: new_args.into_boxed_slice(),
                    }))
                }
                phalcom_type_meta::type_node::TypeNode::Union(members) => {
                    let mut new_members = Vec::with_capacity(members.len());
                    for m in members.iter() {
                        new_members.push(instantiate_type_ref(RuntimeTypeRef::Base { pool, node: *m }, env, context, registry, depth + 1)?);
                    }
                    Some(context.overlay.type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Union(
                        new_members.into_boxed_slice(),
                    )))
                }
                phalcom_type_meta::type_node::TypeNode::Tuple(elements) => {
                    let mut new_elements = Vec::with_capacity(elements.len());
                    for el in elements.iter() {
                        let new_ty = instantiate_type_ref(RuntimeTypeRef::Base { pool, node: el.ty }, env, context, registry, depth + 1)?;
                        new_elements.push(crate::typing::overlay::RuntimeTupleElement {
                            label: el.label.clone(),
                            ty: new_ty,
                        });
                    }
                    Some(context.overlay.type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Tuple(
                        new_elements.into_boxed_slice(),
                    )))
                }
                phalcom_type_meta::type_node::TypeNode::Record(fields) => {
                    let mut new_fields = Vec::with_capacity(fields.len());
                    for f in fields.iter() {
                        let new_ty = instantiate_type_ref(RuntimeTypeRef::Base { pool, node: f.ty }, env, context, registry, depth + 1)?;
                        new_fields.push(crate::typing::overlay::RuntimeRecordField {
                            name: f.name.clone(),
                            ty: new_ty,
                        });
                    }
                    Some(context.overlay.type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Record(
                        new_fields.into_boxed_slice(),
                    )))
                }
                phalcom_type_meta::type_node::TypeNode::Callable(callable) => {
                    let mut new_params = Vec::with_capacity(callable.parameters.len());
                    for p in callable.parameters.iter() {
                        let new_ty = instantiate_type_ref(RuntimeTypeRef::Base { pool, node: p.ty }, env, context, registry, depth + 1)?;
                        new_params.push(crate::typing::overlay::RuntimeCallableParameter {
                            label: p.label.clone(),
                            ty: new_ty,
                            rest: p.rest,
                        });
                    }
                    let new_return = instantiate_type_ref(RuntimeTypeRef::Base { pool, node: callable.return_type }, env, context, registry, depth + 1)?;
                    Some(context.overlay.type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Callable {
                        parameters: new_params.into_boxed_slice(),
                        return_type: new_return,
                    }))
                }
                _ => Some(ty),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heap::ObjRef;
    use crate::typing::context::TypingContextData;
    use crate::typing::handle::RuntimeTypeRef;
    use crate::typing::overlay::{RuntimeOverlayTypeNode, RuntimeTupleElement};
    use crate::typing::registry::RuntimeTypingRegistry;
    use phalcom_type_meta::StableTypeParameterRef;
    use phalcom_type_meta::generic::StableTypeParameterOwnerRef;
    use phalcom_type_meta::identity::StableDeclarationRef;

    #[test]
    fn test_type_environment_interning_and_lookup() {
        let mut registry = RuntimeTypeEnvironmentRegistry::new();
        assert_eq!(RuntimeTypeEnvironmentId::EMPTY.0, 0);
        assert!(registry.get(RuntimeTypeEnvironmentId::EMPTY).unwrap().is_empty());

        let param = StableTypeParameterRef {
            owner: StableTypeParameterOwnerRef::Declaration(StableDeclarationRef {
                module: phalcom_type_meta::identity::StableModuleRef {
                    project: phalcom_type_meta::identity::StableProjectRef::Builtin {
                        namespace: "core".into(),
                        version: "1.0.0".into(),
                    },
                    path: Box::new(["test".into()]),
                },
                path: Box::new(["Foo".into()]),
            }),
            index: 0,
        };
        let int_ty = RuntimeTypeRef::Overlay(crate::typing::handle::RuntimeOverlayTypeId(1));
        let env = RuntimeTypeEnvironment::new(vec![(param.clone(), int_ty)]);

        let id = registry.intern(env.clone());
        assert_eq!(registry.intern(env), id);
        assert_eq!(registry.get(id).unwrap().get(&param), Some(int_ty));
    }

    #[test]
    fn test_instantiate_type_recipe_closed_and_template() {
        let mut context = TypingContextData::new(Box::new([]));
        let registry = RuntimeTypingRegistry::new();

        let dummy_class = ObjRef::from_opaque_u64(1);
        let int_ty = context.overlay.type_ref(RuntimeOverlayTypeNode::Nominal { class: dummy_class });
        let closed_recipe = RuntimeTypeRecipe::Closed(int_ty);
        let empty_env = RuntimeTypeEnvironment::new(vec![]);

        assert_eq!(
            instantiate_type_recipe(closed_recipe, &empty_env, &mut context, &registry),
            Some(int_ty)
        );

        // Template with overlay tuple containing int_ty
        let tuple_ty = context.overlay.type_ref(RuntimeOverlayTypeNode::Tuple(Box::new([
            RuntimeTupleElement { label: None, ty: int_ty },
        ])));
        let template_recipe = RuntimeTypeRecipe::Template(tuple_ty);

        assert_eq!(
            instantiate_type_recipe(template_recipe, &empty_env, &mut context, &registry),
            Some(tuple_ty)
        );
    }
}
