use phalcom_common::selector::Selector;
use phalcom_compiler_artifact::module_world::{ArtifactModuleId, ModuleKindV1, ModuleV2, ModuleWorldV2};
use phalcom_compiler_artifact::query::{PortableQueryKindV1 as Kind, PortableQueryOwnerKeyV1 as Owner};
use phalcom_modules::ProjectUniverse;
use phalcom_modules::identity::ModulePath;
use phalcom_semantic::artifact::{StableOwnerCatalog, query_root_is_reusable};
use phalcom_semantic::db::QueryKey;
use phalcom_semantic::db::key::QueryKind;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, FieldId, ModuleId};
use phalcom_semantic::types::type_member::{TypeMemberDeclarationId, TypeMemberSelector};
use phalcom_type_meta::identity::{
    StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide, StableFieldRef, StableModuleRef, StableProjectRef, StableTypeMemberRef,
};

fn projects() -> ProjectUniverse {
    ProjectUniverse::new()
}

fn module_world(identity: StableModuleRef) -> ModuleWorldV2 {
    let path = identity.path.iter().map(|segment| segment.to_string()).collect();
    ModuleWorldV2 {
        entry: ArtifactModuleId(0),
        initialization_order: vec![ArtifactModuleId(0)],
        modules: vec![ModuleV2 {
            identity,
            path,
            kind: ModuleKindV1::Module,
            declarations: vec![],
            exports: vec![],
            imports: vec![],
            exposed_children: vec![],
            metadata: vec![],
            linked_exports: vec![],
            globals: vec![],
            import_bindings: vec![],
            linked_reads: vec![],
            runtime_dependencies: vec![],
            references: vec![],
            runtime_edges: vec![],
            topology_parent: None,
            topology_children: vec![],
            interface_fingerprint: 0,
            linked_interface_fingerprint: 0,
            dependency_fingerprint: 0,
        }],
        semantic_rows: vec![],
        external_projects: vec![],
        topology_fingerprint: 0,
    }
}

fn stable_module() -> StableModuleRef {
    StableModuleRef {
        project: StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        },
        path: Box::new([]),
    }
}

fn stable_declaration() -> StableDeclarationRef {
    StableDeclarationRef {
        module: stable_module(),
        path: vec!["Widget".into()].into_boxed_slice(),
    }
}

fn declaration() -> DeclarationId {
    DeclarationId::new(ModuleId::universe(ModulePath::root()), "Widget".into())
}

#[test]
fn typed_query_key_reconstructs_project_qualified_owner() {
    let projects = projects();
    let stable_module = stable_module();
    let module_world = module_world(stable_module.clone());
    let owners = StableOwnerCatalog::new(&module_world, &projects).unwrap();
    let stable_declaration = stable_declaration();
    let selector = Selector::method("run", Vec::new()).unwrap();
    let stable_callable = StableCallableRef {
        owner: StableCallableOwnerRef::Declaration(stable_declaration.clone()),
        side: StableDispatchSide::Instance,
        selector: selector.encode().into_boxed_str(),
    };

    assert_eq!(
        QueryKey::from_portable_owner(Kind::ModuleStaticBindings, &Owner::Module(stable_module.clone()), &owners).unwrap(),
        QueryKey::ModuleStaticBindings(ModuleId::universe(ModulePath::root())),
    );
    assert_eq!(
        QueryKey::from_portable_owner(Kind::DataDeclaration, &Owner::Declaration(stable_declaration.clone()), &owners).unwrap(),
        QueryKey::DataDeclaration(declaration()),
    );
    assert_eq!(
        QueryKey::from_portable_owner(Kind::CallableDefinition, &Owner::Callable(stable_callable), &owners).unwrap(),
        QueryKey::CallableDefinition(CallableId::new(declaration(), selector, DispatchSide::Instance)),
    );
    assert_eq!(
        QueryKey::from_portable_owner(
            Kind::FieldSignature,
            &Owner::Field(StableFieldRef {
                owner: stable_declaration.clone(),
                side: StableDispatchSide::Class,
                name: "count".into(),
            }),
            &owners,
        )
        .unwrap(),
        QueryKey::FieldSignature(FieldId::new(declaration(), "count", DispatchSide::Class)),
    );
    assert_eq!(
        QueryKey::from_portable_owner(
            Kind::TypeMemberDeclaration,
            &Owner::TypeMember(StableTypeMemberRef {
                owner: stable_declaration,
                selector: "[]".into(),
            }),
            &owners,
        )
        .unwrap(),
        QueryKey::TypeMemberDeclaration(TypeMemberDeclarationId::new(declaration(), TypeMemberSelector::Subscript)),
    );
}

#[test]
fn mismatched_and_lossy_portable_owners_fail_closed() {
    let projects = projects();
    let module_world = module_world(stable_module());
    let owners = StableOwnerCatalog::new(&module_world, &projects).unwrap();

    assert!(QueryKey::from_portable_owner(Kind::DataDeclaration, &Owner::Module(stable_module()), &owners).is_err());
    assert!(
        QueryKey::from_portable_owner(
            Kind::GenericConformance,
            &Owner::GenericConformance {
                implementation: phalcom_type_meta::identity::StableImplRef {
                    module: stable_module(),
                    local_id: 4,
                },
                context_ordinal: 0,
            },
            &owners,
        )
        .is_err()
    );
}

#[test]
fn internal_owner_must_resolve_to_the_exact_module_row() {
    let projects = projects();
    let module_world = module_world(stable_module());
    let owners = StableOwnerCatalog::new(&module_world, &projects).unwrap();
    let missing_module = StableModuleRef {
        project: StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        },
        path: vec!["missing".into()].into_boxed_slice(),
    };
    assert!(QueryKey::from_portable_owner(Kind::ModuleStaticBindings, &Owner::Module(missing_module), &owners).is_err());
}

#[test]
fn reusable_query_roots_require_both_policy_and_a_portable_product_location() {
    assert!(query_root_is_reusable(QueryKind::DataDeclaration));
    assert!(query_root_is_reusable(QueryKind::ModuleStaticBindings));
    assert!(!query_root_is_reusable(QueryKind::ParsedModule));
    assert!(!query_root_is_reusable(QueryKind::HierarchyEdge));
    assert!(!query_root_is_reusable(QueryKind::GenericConformance));
    assert!(!query_root_is_reusable(QueryKind::AttributeArgument));
}
