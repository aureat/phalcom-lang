//! Crate-local tests for `ModuleWorldV5` (spec `03-spec-module-world-v5.md` §6).

use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::module_world::*;
use phalcom_compiler_artifact::module_world_v5::*;
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::{StableModuleRef, StableProjectRef};

fn universe() -> StableProjectRef {
    StableProjectRef::Builtin {
        namespace: "universe".into(),
        version: "0.1.0".into(),
    }
}

fn package() -> StableProjectRef {
    StableProjectRef::Package {
        package: "slotmap".into(),
        version: "0.1.1".into(),
        artifact_fingerprint: Fingerprint128([7; 16]),
    }
}

fn module_ref(project: &StableProjectRef, path: &[&str]) -> StableModuleRef {
    StableModuleRef {
        project: project.clone(),
        path: path.iter().map(|segment| (*segment).into()).collect::<Vec<Box<str>>>().into_boxed_slice(),
    }
}

fn span() -> SpanV1 {
    SpanV1 { start: 1, end: 2 }
}

fn row(identity: StableModuleRef, kind: ModuleKindV1) -> ModuleV3 {
    ModuleV3 {
        path: identity.path.iter().map(|segment| segment.to_string()).collect(),
        identity,
        kind,
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
    }
}

fn ext(index: u32) -> ModuleRefV1 {
    ModuleRefV1::External {
        module: ExternalModuleIndex(index),
    }
}

fn loc(index: u32) -> ModuleRefV1 {
    ModuleRefV1::Local {
        module: ArtifactModuleId(index),
    }
}

/// A package world with an external reference in every changed position.
fn package_world() -> ModuleWorldV5 {
    let package = package();
    let universe = universe();
    let mut root = row(module_ref(&package, &[]), ModuleKindV1::Package);
    let mut a = row(module_ref(&package, &["a"]), ModuleKindV1::Module);
    root.topology_children = vec![ArtifactModuleId(1)];
    a.topology_parent = Some(ArtifactModuleId(0));
    a.exports = vec![ExportV2 {
        exported_name: "Core".into(),
        internal_name: "Core".into(),
        target: UnlinkedTargetV2::CanonicalDeclaration {
            module: ext(0),
            name: "Core".into(),
        },
        span: span(),
    }];
    a.linked_exports = vec![LinkedExportV2 {
        name: "Core".into(),
        target: TargetV2::Module { module: ext(0) },
        span: span(),
    }];
    a.linked_reads = vec![
        TargetV2::Binding {
            module: ext(1),
            name: "List".into(),
        },
        TargetV2::Binding {
            module: ext(0),
            name: "x".into(),
        },
    ];
    a.runtime_dependencies = vec![ext(1)];
    a.runtime_edges = vec![RuntimeEdgeV2 {
        dependency: ext(1),
        reason: RuntimeReasonV1::SelectiveValueImport,
        span: span(),
    }];
    a.references = vec![ReferenceV2 {
        target: ext(0),
        kind: ReferenceKindV1::WholeModuleImport,
        span: span(),
    }];
    let mut world = ModuleWorldV5 {
        entry: ArtifactModuleId(0),
        initialization_order: vec![ArtifactModuleId(0), ArtifactModuleId(1)],
        modules: vec![root, a],
        semantic_rows: vec![SemanticRowV2 {
            node: SemanticNodeV1::Module { module: ArtifactModuleId(1) },
            edges: vec![SemanticEdgeV2 {
                target: SemanticNodeV2::Declaration {
                    module: ext(1),
                    name: "List".into(),
                },
                kind: SemanticEdgeKindV1::TypeReference,
                span: span(),
            }],
        }],
        external_projects: vec![universe.clone()],
        external_modules: vec![module_ref(&universe, &["core"]), module_ref(&universe, &["core", "list"])],
        topology_fingerprint: 0,
        availability_witnesses: vec![RuntimeAvailabilityWitnessV2 {
            importer: ArtifactModuleId(1),
            dependency: ext(0),
            range: span(),
            reason: RuntimeReasonV1::SelectiveValueImport,
            read_target: Some(TargetV2::Binding {
                module: ext(0),
                name: "x".into(),
            }),
            target: TargetV2::Binding {
                module: ext(0),
                name: "x".into(),
            },
            product: AvailabilityProductV1::RuntimeObject,
            producer: AvailabilityRequirementProducerV1::ImportRead,
            required_at: AvailabilityTimingV1::AfterBootstrap,
            authority: AvailabilityAuthorityV1::NativeBindingCatalog,
            available_at: AvailabilityTimingV1::BeforeSourceExecution,
        }],
        bootstrap_modules: vec![],
        bootstrap_availability: vec![],
    };
    world.refresh_portable_topology_fingerprint().unwrap();
    world
}

/// A Universe-shaped closed V2 world (all local references).
fn universe_world_v4() -> ModuleWorldV4 {
    let universe = universe();
    let mut root = ModuleV2 {
        identity: module_ref(&universe, &[]),
        path: vec![],
        kind: ModuleKindV1::Package,
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
        topology_children: vec![ArtifactModuleId(1)],
        interface_fingerprint: 1,
        linked_interface_fingerprint: 2,
        dependency_fingerprint: 3,
    };
    let mut core = ModuleV2 {
        identity: module_ref(&universe, &["core"]),
        path: vec!["core".into()],
        topology_parent: Some(ArtifactModuleId(0)),
        topology_children: vec![],
        ..root.clone()
    };
    core.linked_reads = vec![TargetV1::Binding {
        module: ArtifactModuleId(0),
        name: "Object".into(),
    }];
    core.runtime_dependencies = vec![ArtifactModuleId(0)];
    core.runtime_edges = vec![RuntimeEdgeV1 {
        dependency: ArtifactModuleId(0),
        reason: RuntimeReasonV1::WholeModuleImport,
        span: span(),
    }];
    core.references = vec![ReferenceV1 {
        target: ArtifactModuleId(0),
        kind: ReferenceKindV1::InterfaceOnly,
        span: span(),
    }];
    root.exports = vec![ExportV1 {
        exported_name: "Core".into(),
        internal_name: "Core".into(),
        target: UnlinkedTargetV1::CanonicalDeclaration {
            module: ArtifactModuleId(1),
            name: "Core".into(),
        },
        span: span(),
    }];
    let mut v2 = ModuleWorldV2 {
        entry: ArtifactModuleId(0),
        initialization_order: vec![ArtifactModuleId(0), ArtifactModuleId(1)],
        modules: vec![root, core],
        semantic_rows: vec![SemanticRowV1 {
            node: SemanticNodeV1::Module { module: ArtifactModuleId(1) },
            edges: vec![SemanticEdgeV1 {
                target: SemanticNodeV1::Declaration {
                    module: ArtifactModuleId(0),
                    name: "Object".into(),
                },
                kind: SemanticEdgeKindV1::TypeReference,
                span: span(),
            }],
        }],
        external_projects: vec![],
        topology_fingerprint: 0,
    };
    v2.refresh_portable_topology_fingerprint().unwrap();
    let witness = RuntimeAvailabilityWitnessV1 {
        importer: ArtifactModuleId(1),
        dependency: ArtifactModuleId(0),
        range: span(),
        reason: RuntimeReasonV1::WholeModuleImport,
        read_target: None,
        target: TargetV1::Module { module: ArtifactModuleId(0) },
        product: AvailabilityProductV1::CanonicalIdentity,
        producer: AvailabilityRequirementProducerV1::DeclarationRead,
        required_at: AvailabilityTimingV1::AfterBootstrap,
        authority: AvailabilityAuthorityV1::SemanticIdentityCatalog,
        available_at: AvailabilityTimingV1::BeforeSourceExecution,
    };
    ModuleWorldV4::new(
        ModuleWorldV3::new(v2, vec![witness]),
        vec![ArtifactModuleId(0)],
        vec![AvailabilityProofV1 {
            target: TargetV1::Module { module: ArtifactModuleId(0) },
            product: AvailabilityProductV1::CanonicalIdentity,
            authority: AvailabilityAuthorityV1::SemanticIdentityCatalog,
            available_at: AvailabilityTimingV1::BeforeSourceExecution,
        }],
    )
}

fn limits() -> ValidationLimits {
    ValidationLimits::default()
}

/// Applies `mutate`, refreshes the fingerprint so it is not the failure, and
/// asserts validation fails with `needle`.
fn expect_rejection(needle: &str, mutate: impl FnOnce(&mut ModuleWorldV5)) {
    let mut world = package_world();
    world.validate(&limits()).expect("fixture is valid");
    mutate(&mut world);
    // A dangling reference makes the fingerprint uncomputable; validation then
    // fails on the specific rule before it reaches the fingerprint check.
    let _ = world.refresh_portable_topology_fingerprint();
    let error = world.validate(&limits()).expect_err("mutated world must be rejected");
    assert!(error.to_string().contains(needle), "error `{error}` lacks `{needle}`");
}

#[test]
fn mw5_rt_round_trip_with_external_references_in_every_position() {
    let world = package_world();
    let bytes = world.encode().unwrap();
    assert_eq!(ModuleWorldV5::decode(&bytes, &limits()).unwrap(), world);
    let mut budget = AllocationBudget::new(usize::MAX);
    assert_eq!(ModuleWorldV5::decode_any_with_allocation_budget(&bytes, &limits(), &mut budget).unwrap(), world);
}

#[test]
fn mw5_lift_round_trips_the_universe_shaped_fixture() {
    let v4 = universe_world_v4();
    v4.validate(&limits()).unwrap();
    let lifted = ModuleWorldV5::from(v4.clone());
    lifted.validate(&limits()).unwrap();
    assert!(lifted.external_modules.is_empty());
    assert_eq!(lifted.try_into_v4().expect("lifted world lowers"), v4);
    // The wire dispatch lifts a V4 section byte-for-byte equal to `from`.
    let mut budget = AllocationBudget::new(usize::MAX);
    let decoded = ModuleWorldV5::decode_any_with_allocation_budget(&v4.encode().unwrap(), &limits(), &mut budget).unwrap();
    assert_eq!(decoded, lifted);
    // Any external reference blocks lowering.
    assert!(package_world().try_into_v4().is_none());
}

#[test]
fn mw5_fp_lifted_fingerprint_equals_v4_fingerprint_and_tracks_external_paths() {
    let v4 = universe_world_v4();
    let lifted = ModuleWorldV5::from(v4.clone());
    assert_eq!(lifted.portable_topology_fingerprint().unwrap(), v4.topology_fingerprint);

    let world = package_world();
    let mut changed = world.clone();
    changed.external_modules[1] = module_ref(&universe(), &["core", "other"]);
    assert_ne!(
        world.portable_topology_fingerprint().unwrap(),
        changed.portable_topology_fingerprint().unwrap()
    );
}

#[test]
fn mw5_v1_external_modules_must_be_strictly_sorted() {
    expect_rejection("MW5-V1", |world| world.external_modules.reverse());
    expect_rejection("MW5-V1", |world| {
        let duplicate = world.external_modules[0].clone();
        world.external_modules.insert(1, duplicate);
    });
}

#[test]
fn mw5_v2_external_module_cannot_have_a_session_project() {
    expect_rejection("not persistent", |world| {
        world.external_modules[1].project = StableProjectRef::Session {
            session_fingerprint: Fingerprint128([1; 16]),
        };
    });
}

#[test]
fn mw5_v3_external_project_must_be_declared() {
    expect_rejection("MW5-V3", |world| world.external_projects.clear());
}

#[test]
fn mw5_v4_external_module_cannot_live_in_an_owned_project() {
    expect_rejection("MW5-V4", |world| {
        let own = package();
        world.external_projects.push(own.clone());
        world.external_projects.sort();
        world.external_modules.push(module_ref(&own, &["zz"]));
        world.external_modules.sort();
    });
}

#[test]
fn mw5_v5_every_external_module_must_be_referenced() {
    expect_rejection("MW5-V5", |world| {
        world.external_modules.push(module_ref(&universe(), &["core", "zz"]));
    });
}

#[test]
fn mw5_v6_local_only_positions_are_range_checked_as_local() {
    expect_rejection("topology parent of module 1", |world| world.modules[1].topology_parent = Some(ArtifactModuleId(9)));
    expect_rejection("entry module", |world| world.entry = ArtifactModuleId(9));
    expect_rejection("initialization order", |world| world.initialization_order[1] = ArtifactModuleId(9));
    expect_rejection("witness importer", |world| world.availability_witnesses[0].importer = ArtifactModuleId(9));
}

#[test]
fn mw5_v7_dependent_world_cannot_carry_a_bootstrap_contract() {
    expect_rejection("MW5-V7", |world| world.bootstrap_modules = vec![ArtifactModuleId(0)]);
}

#[test]
fn mw5_v8_runtime_dependencies_and_edges_must_agree_across_arms() {
    expect_rejection("MW5-V8", |world| world.modules[1].runtime_edges[0].dependency = ext(0));
}

#[test]
fn mw5_v9_witness_rules_hold_for_external_dependencies() {
    expect_rejection("target and dependency disagree", |world| world.availability_witnesses[0].dependency = ext(1));
    expect_rejection("MW5-V9", |world| world.availability_witnesses[0].authority = AvailabilityAuthorityV1::CanonicalSourceInitializer);
    expect_rejection("does not match a finalized runtime read", |world| {
        world.availability_witnesses[0].read_target = Some(TargetV2::Module { module: ext(0) });
    });
}

#[test]
fn mw5_v10_unwitnessed_external_read_is_rejected() {
    expect_rejection("MW5-V10", |world| world.availability_witnesses.clear());
}

#[test]
fn mw5_range_errors_name_the_reference_kind() {
    expect_rejection("external module reference 5 out of bounds (2 entries)", |world| {
        world.modules[1].references[0].target = ext(5);
    });
    expect_rejection("local module 5 out of bounds", |world| {
        world.modules[1].references[0].target = loc(5);
    });
}

#[test]
fn mw5_trail_trailing_bytes_and_wrong_schema_are_rejected() {
    let world = package_world();
    let mut bytes = world.encode().unwrap();
    bytes.push(0);
    assert!(ModuleWorldV5::decode(&bytes, &limits()).unwrap_err().to_string().contains("trailing"));

    let mut wrong = world.encode().unwrap();
    wrong[..4].copy_from_slice(&MODULE_WORLD_SCHEMA_V4.to_le_bytes());
    assert!(ModuleWorldV5::decode(&wrong, &limits()).is_err());
}

#[test]
fn mw5_fingerprint_mismatch_is_rejected() {
    let mut world = package_world();
    world.topology_fingerprint ^= 1;
    assert!(world.validate(&limits()).unwrap_err().to_string().contains("fingerprint mismatch"));
}
