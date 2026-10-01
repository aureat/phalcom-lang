//! Runtime type recipe and compact environment registry (LANG005.C1.P3 Task 21).

use crate::typing::handle::RuntimeTypeRef;
use phalcom_type_meta::StableTypeParameterRef;
use phalcom_type_meta::identity::StableCallableRef;
use std::collections::HashMap;

/// A runtime type recipe representing either a closed type or a generic template type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RuntimeTypeRecipe {
    Closed(RuntimeTypeRef),
    Template(RuntimeTypeRef),
}

/// Executable recipe for one generic conformance evidence slot at call entry.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RuntimeConformanceSlotRecipe {
    Selection {
        callable: phalcom_semantic::identity::CallableId,
        selector: phalcom_common::selector::Selector,
        nested_slots: Box<[(u16, RuntimeConformanceSlotRecipe)]>,
    },
    Forwarded(u16),
    DerivedConditional {
        callable: phalcom_semantic::identity::CallableId,
        selector: phalcom_common::selector::Selector,
        type_bindings: Box<[(StableTypeParameterRef, RuntimeTypeRecipe)]>,
        nested_slots: Box<[(u16, RuntimeConformanceSlotRecipe)]>,
    },
}

/// Immutable call-entry recipe projected from the semantic call
/// specialization product. The callable identity is retained so the VM can
/// verify that a recipe is used only for its intended callee.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RuntimeCallEnvironmentRecipe {
    pub callable: StableCallableRef,
    pub bindings: Box<[(StableTypeParameterRef, RuntimeTypeRecipe)]>,
    pub conformance_slots: Box<[RuntimeConformanceSlotRecipe]>,
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
        self.bindings.binary_search_by(|(p, _)| p.cmp(param)).ok().map(|idx| self.bindings[idx].1)
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
        let empty_env = RuntimeTypeEnvironment { bindings: Box::new([]) };
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
        RuntimeTypeRecipe::Template(ty) => instantiate_type_ref(ty, env, context, registry, 0),
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
                        new_args.push(match arg {
                            crate::typing::overlay::RuntimeStaticArgument::Term(arg) => {
                                crate::typing::overlay::RuntimeStaticArgument::Term(instantiate_type_ref(*arg, env, context, registry, depth + 1)?)
                            }
                            crate::typing::overlay::RuntimeStaticArgument::RecordRow(row) => {
                                crate::typing::overlay::RuntimeStaticArgument::RecordRow(instantiate_record_row(row, env, context, registry, depth + 1)?)
                            }
                            crate::typing::overlay::RuntimeStaticArgument::DynamicType => crate::typing::overlay::RuntimeStaticArgument::DynamicType,
                        });
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
                    Some(
                        context
                            .overlay
                            .type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Union(new_members.into_boxed_slice())),
                    )
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
                    Some(
                        context
                            .overlay
                            .type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Tuple(new_elements.into_boxed_slice())),
                    )
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
                    Some(
                        context
                            .overlay
                            .type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Record(new_fields.into_boxed_slice())),
                    )
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
                    // The parameter is bound when the caller carried a reified type
                    // environment (e.g. an InvokeSpecialized call threading T through).
                    // When the parameter is *absent* the type argument has been erased at
                    // this call site — e.g. a method on ListIterator<T> called via a plain
                    // Invoke from a native List frame that never carries reified T.
                    // Degrade to Dynamic: the correct erasure fallback. Dynamic dispatch is
                    // always by selector, never by the value of a type argument, so the
                    // missing binding does not affect correctness of the call itself.
                    Some(env.get(param).unwrap_or_else(|| {
                        context
                            .overlay
                            .type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Special("Dynamic".into()))
                    }))
                }
                phalcom_type_meta::type_node::TypeNode::Nominal { .. }
                | phalcom_type_meta::type_node::TypeNode::Never
                | phalcom_type_meta::type_node::TypeNode::Unit => Some(ty),
                phalcom_type_meta::type_node::TypeNode::Applied { origin, arguments } => {
                    if arguments
                        .iter()
                        .any(|argument| matches!(argument, phalcom_type_meta::type_node::StaticArgumentRef::RecordRow(row) if row.tail.is_some()))
                    {
                        // Runtime type environments bind proper type terms only.
                        // An open row cannot be made exact by erasing or closing its tail.
                        return None;
                    }
                    let new_origin = instantiate_type_ref(RuntimeTypeRef::Base { pool, node: *origin }, env, context, registry, depth + 1)?;
                    let mut new_args = Vec::with_capacity(arguments.len());
                    for arg in arguments.iter() {
                        new_args.push(match arg {
                            phalcom_type_meta::type_node::StaticArgumentRef::Term(arg) => crate::typing::overlay::RuntimeStaticArgument::Term(
                                instantiate_type_ref(RuntimeTypeRef::Base { pool, node: *arg }, env, context, registry, depth + 1)?,
                            ),
                            phalcom_type_meta::type_node::StaticArgumentRef::RecordRow(row) => {
                                let mut fields = Vec::with_capacity(row.fields.len());
                                for field in row.fields.iter() {
                                    fields.push(crate::typing::overlay::RuntimeRecordField {
                                        name: field.name.clone(),
                                        ty: instantiate_type_ref(RuntimeTypeRef::Base { pool, node: field.ty }, env, context, registry, depth + 1)?,
                                    });
                                }
                                crate::typing::overlay::RuntimeStaticArgument::RecordRow(crate::typing::overlay::RuntimeRecordRow {
                                    fields: fields.into_boxed_slice(),
                                    tail: row.tail.clone(),
                                })
                            }
                            phalcom_type_meta::type_node::StaticArgumentRef::DynamicType => crate::typing::overlay::RuntimeStaticArgument::DynamicType,
                        });
                    }
                    Some(context.overlay.type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Applied {
                        origin: new_origin,
                        arguments: new_args.into_boxed_slice(),
                    }))
                }
                phalcom_type_meta::type_node::TypeNode::Union(members) => {
                    let mut new_members = Vec::with_capacity(members.len());
                    for m in members.iter() {
                        new_members.push(instantiate_type_ref(
                            RuntimeTypeRef::Base { pool, node: *m },
                            env,
                            context,
                            registry,
                            depth + 1,
                        )?);
                    }
                    Some(
                        context
                            .overlay
                            .type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Union(new_members.into_boxed_slice())),
                    )
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
                    Some(
                        context
                            .overlay
                            .type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Tuple(new_elements.into_boxed_slice())),
                    )
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
                    Some(
                        context
                            .overlay
                            .type_ref(crate::typing::overlay::RuntimeOverlayTypeNode::Record(new_fields.into_boxed_slice())),
                    )
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
                    let new_return = instantiate_type_ref(
                        RuntimeTypeRef::Base {
                            pool,
                            node: callable.return_type,
                        },
                        env,
                        context,
                        registry,
                        depth + 1,
                    )?;
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

fn instantiate_record_row(
    row: &crate::typing::overlay::RuntimeRecordRow,
    env: &RuntimeTypeEnvironment,
    context: &mut crate::typing::context::TypingContextData,
    registry: &crate::typing::registry::RuntimeTypingRegistry,
    depth: usize,
) -> Option<crate::typing::overlay::RuntimeRecordRow> {
    if row.tail.is_some() || depth > 64 {
        return None;
    }
    let mut fields = Vec::with_capacity(row.fields.len());
    for field in row.fields.iter() {
        fields.push(crate::typing::overlay::RuntimeRecordField {
            name: field.name.clone(),
            ty: instantiate_type_ref(field.ty, env, context, registry, depth + 1)?,
        });
    }
    Some(crate::typing::overlay::RuntimeRecordRow {
        fields: fields.into_boxed_slice(),
        tail: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heap::ObjRef;
    use crate::typing::context::TypingContextData;
    use crate::typing::handle::{MetadataPoolId, RuntimeTypeRef};
    use crate::typing::loader::LoadedSemanticMetadata;
    use crate::typing::overlay::{RuntimeOverlayTypeNode, RuntimeRecordField, RuntimeRecordRow, RuntimeStaticArgument, RuntimeTupleElement};
    use crate::typing::registry::RuntimeTypingRegistry;
    use phalcom_type_meta::StableTypeParameterRef;
    use phalcom_type_meta::bundle::SemanticMetadataBundle;
    use phalcom_type_meta::fingerprint::Fingerprint128;
    use phalcom_type_meta::generic::StableTypeParameterOwnerRef;
    use phalcom_type_meta::header::{
        ArtifactIdentityScheme, MetadataFeatures, MetadataProfile, NATIVE_SURFACE_SCHEMA_VERSION, ProducerIdentity, SEMANTIC_MODEL_VERSION,
        TYPE_METADATA_SCHEMA_VERSION,
    };
    use phalcom_type_meta::identity::{StableDeclarationRef, StableModuleRef, StableProjectRef};
    use phalcom_type_meta::kind::{KindNode, KindNodeEntry, KindNodeId};
    use phalcom_type_meta::type_node::{RecordFieldRef, StaticArgumentRef, StructuralMemberPresenceRef, TypeNode, TypeNodeEntry, TypeNodeId};
    use std::sync::Arc;

    fn row_test_bundle() -> (SemanticMetadataBundle, StableTypeParameterRef, StableTypeParameterRef) {
        let module = StableModuleRef {
            project: StableProjectRef::Builtin {
                namespace: "core".into(),
                version: "1.0.0".into(),
            },
            path: Box::new(["runtime-row-test".into()]),
        };
        let owner = StableDeclarationRef {
            module: module.clone(),
            path: Box::new(["MyRecord".into()]),
        };
        let parameter = StableTypeParameterRef {
            owner: StableTypeParameterOwnerRef::Declaration(owner.clone()),
            index: 0,
        };
        let open_tail = StableTypeParameterRef {
            owner: StableTypeParameterOwnerRef::Declaration(owner.clone()),
            index: 1,
        };
        let type_entry = |form| TypeNodeEntry {
            kind: KindNodeId(0),
            form,
            structural_fingerprint: Fingerprint128::ZERO,
        };
        let types = vec![
            type_entry(TypeNode::Nominal {
                declaration: StableDeclarationRef {
                    module: module.clone(),
                    path: Box::new(["Int".into()]),
                },
            }),
            type_entry(TypeNode::Nominal {
                declaration: StableDeclarationRef {
                    module: module.clone(),
                    path: Box::new(["String".into()]),
                },
            }),
            type_entry(TypeNode::Parameter(parameter.clone())),
            type_entry(TypeNode::Nominal { declaration: owner }),
            type_entry(TypeNode::Applied {
                origin: TypeNodeId(3),
                arguments: vec![StaticArgumentRef::RecordRow(phalcom_type_meta::type_node::RecordRowRef {
                    fields: vec![RecordFieldRef {
                        name: "payload".into(),
                        ty: TypeNodeId(2),
                        presence: StructuralMemberPresenceRef::Required,
                    }]
                    .into_boxed_slice(),
                    tail: None,
                    remainder: None,
                })]
                .into_boxed_slice(),
            }),
            type_entry(TypeNode::Applied {
                origin: TypeNodeId(3),
                arguments: vec![StaticArgumentRef::RecordRow(phalcom_type_meta::type_node::RecordRowRef {
                    fields: vec![RecordFieldRef {
                        name: "payload".into(),
                        ty: TypeNodeId(2),
                        presence: StructuralMemberPresenceRef::Required,
                    }]
                    .into_boxed_slice(),
                    tail: Some(open_tail.clone()),
                    remainder: None,
                })]
                .into_boxed_slice(),
            }),
            type_entry(TypeNode::Applied {
                origin: TypeNodeId(3),
                arguments: vec![StaticArgumentRef::Term(TypeNodeId(0))].into_boxed_slice(),
            }),
        ];
        let bundle = SemanticMetadataBundle {
            header: phalcom_type_meta::header::SemanticMetadataHeader {
                schema_version: TYPE_METADATA_SCHEMA_VERSION,
                semantic_model_version: SEMANTIC_MODEL_VERSION,
                producer: ProducerIdentity("typing-test".into()),
                producer_version: "1".into(),
                native_surface_schema_version: NATIVE_SURFACE_SCHEMA_VERSION,
                profile: MetadataProfile::RuntimePublic,
                features: MetadataFeatures::default(),
                identity_scheme: ArtifactIdentityScheme::V1Standard,
                source_fingerprint: Fingerprint128::ZERO,
                interface_fingerprint: Fingerprint128::ZERO,
            },
            kinds: vec![KindNodeEntry {
                node: KindNode::Type,
                structural_fingerprint: Fingerprint128::ZERO,
            }]
            .into_boxed_slice(),
            types: types.into_boxed_slice(),
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
        (bundle, parameter, open_tail)
    }

    fn row_test_registry(bundle: SemanticMetadataBundle) -> RuntimeTypingRegistry {
        let mut registry = RuntimeTypingRegistry::new();
        registry.register_pool(LoadedSemanticMetadata::new(MetadataPoolId(0), Arc::new(bundle)));
        registry
    }

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

        assert_eq!(instantiate_type_recipe(closed_recipe, &empty_env, &mut context, &registry), Some(int_ty));

        // Template with overlay tuple containing int_ty
        let tuple_ty = context
            .overlay
            .type_ref(RuntimeOverlayTypeNode::Tuple(Box::new([RuntimeTupleElement { label: None, ty: int_ty }])));
        let template_recipe = RuntimeTypeRecipe::Template(tuple_ty);

        assert_eq!(instantiate_type_recipe(template_recipe, &empty_env, &mut context, &registry), Some(tuple_ty));
    }

    #[test]
    fn applied_record_row_arguments_preserve_identity_and_substitute_closed_fields() {
        let (bundle, parameter, _) = row_test_bundle();
        let registry = row_test_registry(bundle);
        let mut context = TypingContextData::new(Box::new([]));
        let int_ty = context.overlay.type_ref(RuntimeOverlayTypeNode::Nominal {
            class: ObjRef::from_opaque_u64(101),
        });
        let string_ty = context.overlay.type_ref(RuntimeOverlayTypeNode::Nominal {
            class: ObjRef::from_opaque_u64(102),
        });
        let int_env = RuntimeTypeEnvironment::new(vec![(parameter.clone(), int_ty)]);
        let string_env = RuntimeTypeEnvironment::new(vec![(parameter.clone(), string_ty)]);
        let int_recipe = RuntimeTypeRecipe::Template(RuntimeTypeRef::Base {
            pool: MetadataPoolId(0),
            node: TypeNodeId(4),
        });
        let base_application = RuntimeTypeRef::Base {
            pool: MetadataPoolId(0),
            node: TypeNodeId(4),
        };
        assert!(matches!(
            crate::typing::inspect::static_arguments(&context, &registry, base_application)
                .expect("metadata row argument is inspectable")
                .as_slice(),
            [RuntimeStaticArgument::RecordRow(row)]
                if row.fields[0].ty == (RuntimeTypeRef::Base { pool: MetadataPoolId(0), node: TypeNodeId(2) })
        ));
        assert_eq!(
            crate::typing::inspect::arguments(&context, &registry, base_application),
            None,
            "metadata row arguments are not projected as proper type arguments"
        );

        let int_application = instantiate_type_recipe(int_recipe, &int_env, &mut context, &registry).expect("closed row specializes");
        let int_application_again = instantiate_type_recipe(int_recipe, &int_env, &mut context, &registry).expect("repeat specializes");
        let string_application = instantiate_type_recipe(int_recipe, &string_env, &mut context, &registry).expect("string row specializes");
        assert_eq!(
            int_application, int_application_again,
            "the overlay interner reuses an identical exact row type"
        );
        assert_ne!(
            int_application, string_application,
            "different closed row field types retain distinct exact identities"
        );

        let RuntimeTypeRef::Overlay(int_id) = int_application else {
            panic!("instantiated applied type is overlaid")
        };
        let RuntimeTypeRef::Overlay(string_id) = string_application else {
            panic!("instantiated applied type is overlaid")
        };
        let RuntimeOverlayTypeNode::Applied { arguments: int_arguments, .. } = context.overlay.type_node(int_id).expect("interned int application") else {
            panic!("applied runtime type")
        };
        let RuntimeOverlayTypeNode::Applied {
            arguments: string_arguments, ..
        } = context.overlay.type_node(string_id).expect("interned string application")
        else {
            panic!("applied runtime type")
        };
        let RuntimeStaticArgument::RecordRow(int_row) = &int_arguments[0] else {
            panic!("row remains a distinct static argument")
        };
        let RuntimeStaticArgument::RecordRow(string_row) = &string_arguments[0] else {
            panic!("row remains a distinct static argument")
        };
        assert_eq!(
            int_row.fields.as_ref(),
            &[RuntimeRecordField {
                name: "payload".into(),
                ty: int_ty
            }]
        );
        assert_eq!(
            string_row.fields.as_ref(),
            &[RuntimeRecordField {
                name: "payload".into(),
                ty: string_ty
            }]
        );
        assert_eq!(int_row.tail, None);
        assert_eq!(string_row.tail, None);
        assert_eq!(
            crate::typing::inspect::arguments(&context, &registry, int_application),
            None,
            "the legacy proper-type argument projection cannot flatten a row argument"
        );
        assert_eq!(
            crate::typing::inspect::static_arguments(&context, &registry, int_application)
                .expect("heterogeneous arguments remain inspectable")
                .len(),
            1
        );

        let overlay_template = context.overlay.type_ref(RuntimeOverlayTypeNode::Applied {
            origin: int_ty,
            arguments: vec![RuntimeStaticArgument::RecordRow(RuntimeRecordRow {
                fields: Box::new([RuntimeRecordField {
                    name: "payload".into(),
                    ty: RuntimeTypeRef::Base {
                        pool: MetadataPoolId(0),
                        node: TypeNodeId(2),
                    },
                }]),
                tail: None,
            })]
            .into_boxed_slice(),
        });
        let overlay_instantiated = instantiate_type_recipe(RuntimeTypeRecipe::Template(overlay_template), &int_env, &mut context, &registry)
            .expect("overlay row types use the same substitution path");
        let RuntimeTypeRef::Overlay(overlay_id) = overlay_instantiated else {
            panic!("instantiated overlay application")
        };
        let RuntimeOverlayTypeNode::Applied { arguments, .. } = context.overlay.type_node(overlay_id).expect("overlay application") else {
            panic!("applied overlay")
        };
        let RuntimeStaticArgument::RecordRow(overlay_row) = &arguments[0] else {
            panic!("overlay row remains a row")
        };
        assert_eq!(
            overlay_row.fields[0].ty, int_ty,
            "overlay row parameter is substituted from the runtime environment"
        );

        let proper_record = context.overlay.type_ref(RuntimeOverlayTypeNode::Record(Box::new([RuntimeRecordField {
            name: "payload".into(),
            ty: int_ty,
        }])));
        let row_applied = context.overlay.type_ref(RuntimeOverlayTypeNode::Applied {
            origin: int_ty,
            arguments: vec![RuntimeStaticArgument::RecordRow(RuntimeRecordRow {
                fields: Box::new([RuntimeRecordField {
                    name: "payload".into(),
                    ty: int_ty,
                }]),
                tail: None,
            })]
            .into_boxed_slice(),
        });
        assert_ne!(proper_record, row_applied, "a row argument is not a proper structural Record type");

        let ordinary_type_application = instantiate_type_recipe(
            RuntimeTypeRecipe::Template(RuntimeTypeRef::Base {
                pool: MetadataPoolId(0),
                node: TypeNodeId(6),
            }),
            &int_env,
            &mut context,
            &registry,
        )
        .expect("ordinary type-only Applied remains supported");
        let RuntimeTypeRef::Overlay(ordinary_id) = ordinary_type_application else {
            panic!("ordinary applied type is overlaid")
        };
        let RuntimeOverlayTypeNode::Applied { arguments, .. } = context.overlay.type_node(ordinary_id).expect("ordinary applied node") else {
            panic!("ordinary applied type")
        };
        assert!(matches!(arguments.as_ref(), [RuntimeStaticArgument::Term(_)]));
    }

    #[test]
    fn open_record_row_tail_is_not_erased_during_exact_instantiation() {
        let (bundle, parameter, open_tail) = row_test_bundle();
        let registry = row_test_registry(bundle);
        let mut context = TypingContextData::new(Box::new([]));
        let int_ty = context.overlay.type_ref(RuntimeOverlayTypeNode::Nominal {
            class: ObjRef::from_opaque_u64(103),
        });
        let env = RuntimeTypeEnvironment::new(vec![(parameter, int_ty)]);
        let open_recipe = RuntimeTypeRecipe::Template(RuntimeTypeRef::Base {
            pool: MetadataPoolId(0),
            node: TypeNodeId(5),
        });

        assert!(open_tail.index > 0);
        assert_eq!(instantiate_type_recipe(open_recipe, &env, &mut context, &registry), None);
    }
}
