//! Conversions from semantic/module identities to durable metadata stable identities.

use crate::identity::{CallableId, CallableOwnerId, DeclarationId, DispatchSide, FieldId};
use phalcom_modules::{ModuleId, ProjectIdentity, ProjectUniverse};
use phalcom_type_meta::fingerprint::{Fingerprint128, FingerprintBuilder};
use phalcom_type_meta::identity::{
    StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide, StableFieldRef, StableImplRef, StableModuleRef, StableProjectRef,
    StableVariantRef,
};

#[derive(Clone, Copy)]
pub struct StableIdentityContext<'a> {
    pub projects: &'a ProjectUniverse,
    query_fingerprint: bool,
}

impl<'a> StableIdentityContext<'a> {
    pub fn new(projects: &'a ProjectUniverse) -> Self {
        Self { projects, query_fingerprint: false }
    }

    /// Uses logical project identity for in-process query fingerprints. Unlike
    /// artifact identities, it does not change for an unrelated source edit.
    pub fn for_query(projects: &'a ProjectUniverse) -> Self {
        Self { projects, query_fingerprint: true }
    }
}

pub fn to_stable_project(proj: &ProjectIdentity) -> StableProjectRef {
    match proj {
        ProjectIdentity::Universe => StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        },
        ProjectIdentity::Resolved(_) => panic!("resolved project requires StableIdentityContext"),
        ProjectIdentity::Synthetic(syn_id) => StableProjectRef::Session {
            session_fingerprint: Fingerprint128::from_u128(syn_id.raw() as u128),
        },
    }
}

pub fn to_stable_project_with_context(proj: &ProjectIdentity, context: &StableIdentityContext<'_>) -> StableProjectRef {
    match proj {
        ProjectIdentity::Resolved(id) => context.projects.get_project(*id).map_or_else(
            || to_stable_project(proj),
            |project| StableProjectRef::SourceArtifact {
                logical_uri: project.source_identity.0.to_string_lossy().into_owned().into_boxed_str(),
                source_fingerprint: if context.query_fingerprint {
                    let mut builder = FingerprintBuilder::new();
                    builder.write_str(&project.source_identity.0.to_string_lossy());
                    builder.finish()
                } else {
                    Fingerprint128(project.revision_fingerprint().0)
                },
            },
        ),
        _ => to_stable_project(proj),
    }
}

pub fn to_stable_module_with_context(module: &ModuleId, context: &StableIdentityContext<'_>) -> StableModuleRef {
    StableModuleRef {
        project: to_stable_project_with_context(&module.project, context),
        path: module
            .path
            .components()
            .iter()
            .map(|c| c.to_string().into_boxed_str())
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    }
}

pub fn to_stable_declaration_with_context(decl: &DeclarationId, context: &StableIdentityContext<'_>) -> StableDeclarationRef {
    StableDeclarationRef {
        module: to_stable_module_with_context(&decl.module, context),
        path: Box::new([decl.name.clone()]),
    }
}

pub fn to_stable_callable_with_context(callable: &CallableId, context: &StableIdentityContext<'_>) -> StableCallableRef {
    StableCallableRef {
        owner: stabilize_callable_owner(&callable.owner, |module| to_stable_module_with_context(module, context)),
        side: to_stable_dispatch_side(callable.side),
        selector: callable.selector.encode().into_boxed_str(),
    }
}

pub fn to_stable_field_with_context(field: &FieldId, context: &StableIdentityContext<'_>) -> StableFieldRef {
    StableFieldRef {
        owner: to_stable_declaration_with_context(&field.owner, context),
        side: to_stable_dispatch_side(field.side),
        name: field.name.clone(),
    }
}

pub fn to_stable_module(module: &ModuleId) -> StableModuleRef {
    let project = to_stable_project(&module.project);
    let path_segments: Vec<Box<str>> = module.path.components().iter().map(|c| c.to_string().into_boxed_str()).collect();
    StableModuleRef {
        project,
        path: path_segments.into_boxed_slice(),
    }
}

pub fn to_stable_declaration(decl: &DeclarationId) -> StableDeclarationRef {
    let module = to_stable_module(&decl.module);
    StableDeclarationRef {
        module,
        path: Box::new([decl.name.clone()]),
    }
}

pub fn to_stable_dispatch_side(side: DispatchSide) -> StableDispatchSide {
    match side {
        DispatchSide::Instance => StableDispatchSide::Instance,
        DispatchSide::Class => StableDispatchSide::Class,
    }
}

pub fn to_stable_callable(callable: &CallableId) -> StableCallableRef {
    StableCallableRef {
        owner: stabilize_callable_owner(&callable.owner, to_stable_module),
        side: to_stable_dispatch_side(callable.side),
        selector: callable.selector.encode().into_boxed_str(),
    }
}

pub fn to_stable_field(field: &FieldId) -> StableFieldRef {
    StableFieldRef {
        owner: to_stable_declaration(&field.owner),
        side: to_stable_dispatch_side(field.side),
        name: field.name.clone(),
    }
}

fn stabilize_callable_owner(owner: &CallableOwnerId, module: impl Fn(&ModuleId) -> StableModuleRef) -> StableCallableOwnerRef {
    let declaration = |decl: &DeclarationId| StableDeclarationRef {
        module: module(&decl.module),
        path: Box::new([decl.name.clone()]),
    };
    match owner {
        CallableOwnerId::Declaration(decl) => StableCallableOwnerRef::Declaration(declaration(decl)),
        CallableOwnerId::Variant(variant) => StableCallableOwnerRef::Variant(StableVariantRef {
            owner: declaration(&variant.owner),
            selector: variant.selector.encode().into_boxed_str(),
        }),
        CallableOwnerId::Conformance(implementation) => StableCallableOwnerRef::Conformance(StableImplRef {
            module: module(&implementation.module),
            local_id: implementation.local.0,
        }),
    }
}

pub fn to_stable_impl_with_context(r#impl: &crate::identity::ImplId, context: &StableIdentityContext<'_>) -> phalcom_type_meta::identity::StableImplRef {
    phalcom_type_meta::identity::StableImplRef {
        module: to_stable_module_with_context(&r#impl.module, context),
        local_id: r#impl.local.0,
    }
}

pub fn to_stable_impl(r#impl: &crate::identity::ImplId) -> phalcom_type_meta::identity::StableImplRef {
    phalcom_type_meta::identity::StableImplRef {
        module: to_stable_module(&r#impl.module),
        local_id: r#impl.local.0,
    }
}

pub fn to_stable_static_constant_with_context(
    id: &crate::static_constant::StaticConstantId,
    context: &StableIdentityContext<'_>,
) -> phalcom_type_meta::identity::StableStaticConstantRef {
    phalcom_type_meta::identity::StableStaticConstantRef {
        module: to_stable_module_with_context(&id.0.module, context),
        name: id.0.name.to_string().into_boxed_str(),
    }
}

pub fn to_stable_static_constant(
    id: &crate::static_constant::StaticConstantId,
) -> phalcom_type_meta::identity::StableStaticConstantRef {
    phalcom_type_meta::identity::StableStaticConstantRef {
        module: to_stable_module(&id.0.module),
        name: id.0.name.to_string().into_boxed_str(),
    }
}

pub fn to_stable_variant_with_context(
    variant: &crate::identity::VariantId,
    context: &StableIdentityContext<'_>,
) -> phalcom_type_meta::identity::StableVariantRef {
    phalcom_type_meta::identity::StableVariantRef {
        owner: to_stable_declaration_with_context(&variant.owner, context),
        selector: variant.selector.encode().into_boxed_str(),
    }
}

pub fn to_stable_variant(
    variant: &crate::identity::VariantId,
) -> phalcom_type_meta::identity::StableVariantRef {
    phalcom_type_meta::identity::StableVariantRef {
        owner: to_stable_declaration(&variant.owner),
        selector: variant.selector.encode().into_boxed_str(),
    }
}

pub fn to_stable_variant_field_with_context(
    field: &crate::identity::VariantFieldId,
    context: &StableIdentityContext<'_>,
) -> phalcom_type_meta::identity::StableVariantFieldRef {
    phalcom_type_meta::identity::StableVariantFieldRef {
        variant: to_stable_variant_with_context(&field.variant, context),
        index: field.index,
    }
}

pub fn to_stable_variant_field(
    field: &crate::identity::VariantFieldId,
) -> phalcom_type_meta::identity::StableVariantFieldRef {
    phalcom_type_meta::identity::StableVariantFieldRef {
        variant: to_stable_variant(&field.variant),
        index: field.index,
    }
}

pub fn to_stable_data_component_with_context(
    comp: &crate::identity::DataComponentId,
    context: &StableIdentityContext<'_>,
) -> phalcom_type_meta::identity::StableDataComponentRef {
    phalcom_type_meta::identity::StableDataComponentRef {
        owner: to_stable_declaration_with_context(&comp.owner, context),
        index: comp.index,
    }
}

pub fn to_stable_data_component(
    comp: &crate::identity::DataComponentId,
) -> phalcom_type_meta::identity::StableDataComponentRef {
    phalcom_type_meta::identity::StableDataComponentRef {
        owner: to_stable_declaration(&comp.owner),
        index: comp.index,
    }
}

pub fn to_field_key_ref(key: &crate::field_key::FieldKey) -> phalcom_type_meta::field_key::FieldKeyRef {
    match key {
        crate::field_key::FieldKey::Symbol(s) => phalcom_type_meta::field_key::FieldKeyRef::Symbol(s.as_ref().into()),
        crate::field_key::FieldKey::String(s) => phalcom_type_meta::field_key::FieldKeyRef::String(s.as_ref().into()),
        crate::field_key::FieldKey::Int(i) => phalcom_type_meta::field_key::FieldKeyRef::Int(i.clone()),
        crate::field_key::FieldKey::Bool(b) => phalcom_type_meta::field_key::FieldKeyRef::Bool(*b),
    }
}

pub fn from_field_key_ref(key: &phalcom_type_meta::field_key::FieldKeyRef) -> crate::field_key::FieldKey {
    match key {
        phalcom_type_meta::field_key::FieldKeyRef::Symbol(s) => crate::field_key::FieldKey::symbol(s.as_ref()),
        phalcom_type_meta::field_key::FieldKeyRef::String(s) => crate::field_key::FieldKey::string(s.as_ref()),
        phalcom_type_meta::field_key::FieldKeyRef::Int(i) => crate::field_key::FieldKey::Int(i.clone()),
        phalcom_type_meta::field_key::FieldKeyRef::Bool(b) => crate::field_key::FieldKey::Bool(*b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{ImplId, ImplLocalId, VariantId};
    use phalcom_common::selector::Selector;

    #[test]
    fn stable_callable_owners_preserve_all_source_identity_categories() {
        let module = ModuleId {
            project: ProjectIdentity::Universe,
            path: Default::default(),
        };
        let declaration = DeclarationId::new(module.clone(), "Box".into());
        let owners = [
            CallableOwnerId::Declaration(declaration.clone()),
            CallableOwnerId::Variant(VariantId::new(declaration.clone(), Selector::getter("A").unwrap())),
            CallableOwnerId::Variant(VariantId::new(declaration, Selector::getter("B").unwrap())),
            CallableOwnerId::Conformance(ImplId::new(module.clone(), ImplLocalId(1))),
            CallableOwnerId::Conformance(ImplId::new(module, ImplLocalId(2))),
        ];
        let projects = ProjectUniverse::new();
        let context = StableIdentityContext::new(&projects);
        let projected: Vec<_> = owners
            .into_iter()
            .map(|owner| {
                let callable = CallableId {
                    owner,
                    side: DispatchSide::Instance,
                    selector: Selector::getter("foo").unwrap(),
                };
                let stable = to_stable_callable(&callable);
                assert_eq!(stable, to_stable_callable_with_context(&callable, &context));
                assert_eq!(stable.selector.as_ref(), callable.selector.encode());
                stable
            })
            .collect();
        assert_eq!(projected.iter().collect::<std::collections::BTreeSet<_>>().len(), projected.len());
        assert!(projected[0].owner.try_declaration().is_some());
        assert!(projected[1].owner.try_declaration().is_none());
        assert!(projected[1].owner.enclosing_declaration().is_some());
        assert!(projected[3].owner.enclosing_declaration().is_none());
    }

    #[test]
    fn p8_t1_stable_identities_and_field_keys() {
        use crate::identity::{DataComponentId, VariantFieldId};
        use crate::scalar::StaticInt;
        use crate::static_constant::StaticConstantId;
        use phalcom_common::selector::SelectorSlot;
        use phalcom_modules::identity::ModuleComponent;
        use phalcom_type_meta::field_key::FieldKeyRef;

        let mod1 = ModuleId {
            project: ProjectIdentity::Universe,
            path: Default::default(),
        };
        let mod2 = ModuleId {
            project: ProjectIdentity::Universe,
            path: phalcom_modules::ModulePath::from_components(vec![ModuleComponent::from_identifier("sub").unwrap()]),
        };

        // Same static constant name in two modules does not collide:
        let sc1 = StaticConstantId::new(mod1.clone(), "C");
        let sc2 = StaticConstantId::new(mod2.clone(), "C");
        let stable_sc1 = to_stable_static_constant(&sc1);
        let stable_sc2 = to_stable_static_constant(&sc2);
        assert_ne!(stable_sc1, stable_sc2);
        assert_eq!(stable_sc1.name.as_ref(), "C");
        assert_eq!(stable_sc2.name.as_ref(), "C");

        // Data components with same index but different owner do not collide:
        let decl1 = DeclarationId::new(mod1.clone(), "Point".into());
        let decl2 = DeclarationId::new(mod1.clone(), "Line".into());
        let comp1 = DataComponentId::new(decl1, 0);
        let comp2 = DataComponentId::new(decl2, 0);
        let stable_comp1 = to_stable_data_component(&comp1);
        let stable_comp2 = to_stable_data_component(&comp2);
        assert_ne!(stable_comp1, stable_comp2);
        assert_eq!(stable_comp1.index, 0);
        assert_eq!(stable_comp2.index, 0);

        // Variant fields with same index but different variant do not collide:
        let var_decl = DeclarationId::new(mod1.clone(), "Result".into());
        let var1 = VariantId::new(var_decl.clone(), Selector::method("Ok", [SelectorSlot::Positional]).unwrap());
        let var2 = VariantId::new(var_decl, Selector::method("Err", [SelectorSlot::Positional]).unwrap());
        let field1 = VariantFieldId::new(var1, 0);
        let field2 = VariantFieldId::new(var2, 0);
        let stable_field1 = to_stable_variant_field(&field1);
        let stable_field2 = to_stable_variant_field(&field2);
        assert_ne!(stable_field1, stable_field2);
        assert_eq!(stable_field1.index, 0);
        assert_eq!(stable_field2.index, 0);

        // FieldKeyRef distinctions and round-trip:
        let fk_sym = crate::field_key::FieldKey::symbol("x");
        let fk_str = crate::field_key::FieldKey::string("x");
        let ref_sym = to_field_key_ref(&fk_sym);
        let ref_str = to_field_key_ref(&fk_str);
        assert_ne!(ref_sym, ref_str);
        assert_eq!(ref_sym, FieldKeyRef::Symbol("x".into()));
        assert_eq!(ref_str, FieldKeyRef::String("x".into()));
        assert_eq!(from_field_key_ref(&ref_sym), fk_sym);
        assert_eq!(from_field_key_ref(&ref_str), fk_str);

        // Large / negative exact Int round-trip:
        let bigint = num_bigint::BigInt::parse_bytes(b"-999999999999999999999999999999999999999", 10).unwrap();
        let large_neg = StaticInt::new(bigint);
        let fk_int = crate::field_key::FieldKey::int(large_neg.clone());
        let ref_int = to_field_key_ref(&fk_int);
        assert_eq!(ref_int, FieldKeyRef::Int(large_neg));
        assert_eq!(from_field_key_ref(&ref_int), fk_int);

        // Bool round-trip:
        let fk_bool = crate::field_key::FieldKey::bool(true);
        let ref_bool = to_field_key_ref(&fk_bool);
        assert_eq!(ref_bool, FieldKeyRef::Bool(true));
        assert_eq!(from_field_key_ref(&ref_bool), fk_bool);

        // Ordering equals P4 ordering: Symbol < String < Int < Bool:
        assert!(ref_sym < ref_str);
        assert!(ref_str < ref_int);
        assert!(ref_int < ref_bool);
    }
}
