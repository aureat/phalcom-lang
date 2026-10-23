use phalcom_compiler_artifact::digest::ArtifactDigest;
use phalcom_compiler_artifact::header::{
    MODULE_SEMANTIC_ABI_V2, QUERY_FINGERPRINT_ABI_V3, SEMANTIC_ARTIFACT_ABI_V3, TELOSC_CONTAINER_SCHEMA_V2, TELOSC_MIN_READER_SCHEMA_V2, TeloscCapabilities,
    TeloscHeaderV2,
};
use phalcom_compiler_artifact::identity::{CanonicalCompatibilityIdentity, TeloscPackageIdentity};
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::metadata::{MetadataPreflightLimits, MetadataSectionV1};
use phalcom_compiler_artifact::module_world::{
    ArtifactModuleId, AvailabilityAuthorityV1, AvailabilityProductV1, AvailabilityRequirementProducerV1, AvailabilityTimingV1, ModuleKindV1, ModuleV2,
    ModuleWorldV2, ModuleWorldV3, RuntimeAvailabilityWitnessV1, RuntimeReasonV1, SpanV1, TargetV1,
};
use phalcom_compiler_artifact::payload::CompilerArtifactPayload;
use phalcom_compiler_artifact::query::{
    DependencyEvidenceEntryV4, PortableQueryKeyV1, PortableQueryKindV1, PortableQueryOwnerKeyV1, PortableQueryProductFamilyV1, QueryImageV4, QueryProductRefV3,
    QueryRootEntryV4,
};
use phalcom_compiler_artifact::semantic::aggregates::{DeclarationSemanticFormV1, DeclarationShellV2, DeclarationTypeBindingV1, StructuralPrimaryContractsV2};
use phalcom_compiler_artifact::semantic::static_value::{StaticValueGraphV1, StaticValueV1};
use phalcom_compiler_artifact::semantic::type_store::{ArtifactKindId, ArtifactTypeId, KindV1, TypeEntryV1, TypeGraphV1, TypeV1};
use phalcom_compiler_artifact::semantic::world::{ModuleExecutionAuthorityV1, ModuleImportPhaseProductV1, PhaseEvidenceCoverageV1, SemanticWorldV4};
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::{StableDeclarationRef, StableModuleRef, StableProjectRef};
use phalcom_type_meta::{
    ArtifactIdentityScheme, KindNode, KindNodeEntry, KindNodeId, MetadataFeatures, MetadataProfile, NATIVE_SURFACE_SCHEMA_VERSION, ProducerIdentity,
    SEMANTIC_MODEL_VERSION, SemanticMetadataBundle, SemanticMetadataHeader, TYPE_METADATA_SCHEMA_VERSION, TypeNode, TypeNodeEntry,
};

fn sample_bundle() -> SemanticMetadataBundle {
    SemanticMetadataBundle {
        header: SemanticMetadataHeader {
            schema_version: TYPE_METADATA_SCHEMA_VERSION,
            semantic_model_version: SEMANTIC_MODEL_VERSION,
            producer: ProducerIdentity("test".into()),
            producer_version: "1.0.0".into(),
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
        types: Box::new([TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        }]),
        scoped_types: Box::new([]),
        parameters: Box::new([]),
        generic_signatures: Box::new([]),
        declarations: Box::new([]),
        static_domains: Box::new([]),
        aliases: Box::new([]),
        callables: Box::new([]),
        fields: Box::new([]),
        static_definitions: Box::new([]),
        static_constants: Box::new([]),
        static_values: Box::new([]),
        module_roots: Box::new([]),
        runtime_roots: Box::new([]),
        occurrences: Box::new([]),
        extensions: Box::new([]),
    }
}

fn sample_metadata_section(pkg_identity: &TeloscPackageIdentity) -> MetadataSectionV1 {
    let bundle = sample_bundle();
    let limits = MetadataPreflightLimits {
        max_metadata_bytes: 65536,
        max_token_count: 10000,
        max_container_count: 10000,
        max_nesting_depth: 32,
    };
    MetadataSectionV1::from_bundle(&bundle, pkg_identity.digest().0, &limits, &phalcom_type_meta::ValidationLimits::default()).expect("create metadata section")
}

fn sample_module_world() -> ModuleWorldV2 {
    let universe_project = StableProjectRef::Builtin {
        namespace: "universe".into(),
        version: "0.1.0".into(),
    };
    let root_module = ModuleV2 {
        identity: StableModuleRef {
            project: universe_project.clone(),
            path: Box::new([]),
        },
        path: vec![],
        kind: ModuleKindV1::Package,
        declarations: vec![],
        exports: vec![],
        imports: vec![],
        exposed_children: vec!["core".into()],
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
        interface_fingerprint: 0,
        linked_interface_fingerprint: 0,
        dependency_fingerprint: 0,
    };

    let child_module = ModuleV2 {
        identity: StableModuleRef {
            project: universe_project,
            path: Box::new(["core".into()]),
        },
        path: vec!["core".into()],
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
        topology_parent: Some(ArtifactModuleId(0)),
        topology_children: vec![],
        interface_fingerprint: 0,
        linked_interface_fingerprint: 0,
        dependency_fingerprint: 0,
    };

    let mut module_world = ModuleWorldV2 {
        entry: ArtifactModuleId(0),
        initialization_order: vec![ArtifactModuleId(0), ArtifactModuleId(1)],
        modules: vec![root_module, child_module],
        semantic_rows: vec![],
        external_projects: vec![],
        topology_fingerprint: 0,
    };
    module_world.refresh_portable_topology_fingerprint().unwrap();
    module_world
}

fn sample_semantic_world() -> SemanticWorldV4 {
    let type_graph = TypeGraphV1 {
        types: vec![TypeEntryV1 {
            kind: ArtifactKindId(0),
            form: TypeV1::Unit,
        }],
        kinds: vec![KindV1::Type],
        parameters: vec![],
        current_parameters: vec![],
        rows: vec![],
        families: vec![],
        variants: vec![],
        static_domains: vec![],
        static_constructors: vec![],
        scoped: vec![],
        lambdas: vec![],
        variance_overrides: vec![],
    };

    let static_values = StaticValueGraphV1 {
        values: vec![StaticValueV1::Unit],
        strings: vec![],
        symbols: vec![],
        tuples: vec![],
        sequences: vec![],
        records: vec![],
        data_values: vec![],
        variant_values: vec![],
        pack_backings: vec![],
        packs: vec![],
    };

    let universe_project = StableProjectRef::Builtin {
        namespace: "universe".into(),
        version: "0.1.0".into(),
    };

    SemanticWorldV4 {
        type_graph,
        structural_contracts: StructuralPrimaryContractsV2 {
            declaration_shells: vec![],
            intrinsic_static_domains: vec![],
            intrinsic_type_owners: vec![],
            hierarchy_edges: vec![],
            type_aliases: vec![],
            generic_signatures: vec![],
            data_contracts: vec![],
            enum_contracts: vec![],
            variant_contracts: vec![],
        },
        enum_requirements: vec![],
        static_values,
        static_definitions: vec![],
        static_constants: vec![],
        module_static_bindings: vec![],
        witness_plans: vec![],
        derived_conformance_rules: vec![],
        callable_signatures: vec![],
        callable_definitions: vec![],
        callable_analyses: vec![],
        trait_surfaces: vec![],
        dispatch_surfaces: vec![],
        conditional_inherent_sets: vec![],
        conformance_heads: vec![],
        constructor_initializations: vec![],
        type_member_declarations: vec![],
        associated_static_domains: vec![],
        static_case_catalogs: vec![],
        static_case_signatures: vec![],
        associated_surfaces: vec![],
        module_execution_authority: ModuleExecutionAuthorityV1 {
            import_phase_products: vec![
                ModuleImportPhaseProductV1 {
                    module: StableModuleRef {
                        project: universe_project.clone(),
                        path: Box::new([]),
                    },
                    bindings: vec![],
                    coverage: PhaseEvidenceCoverageV1::Full,
                },
                ModuleImportPhaseProductV1 {
                    module: StableModuleRef {
                        project: universe_project,
                        path: Box::new(["core".into()]),
                    },
                    bindings: vec![],
                    coverage: PhaseEvidenceCoverageV1::Full,
                },
            ],
            runtime_requirements: vec![],
        },
    }
}

#[test]
fn semantic_world_project_inventory_matches_wire_bytes_and_finds_nested_foreign_owner() {
    let mut world = sample_semantic_world();
    let foreign = StableProjectRef::Package {
        package: "dependency-only-owner".into(),
        version: "2.3.4".into(),
        artifact_fingerprint: Fingerprint128([0x5a; 16]),
    };
    world
        .module_execution_authority
        .import_phase_products
        .push(phalcom_compiler_artifact::semantic::world::ModuleImportPhaseProductV1 {
            module: StableModuleRef {
                project: foreign.clone(),
                path: vec!["nested".into(), "owner".into()].into_boxed_slice(),
            },
            bindings: vec![],
            coverage: phalcom_compiler_artifact::semantic::world::PhaseEvidenceCoverageV1::Full,
        });
    world.module_execution_authority.import_phase_products.sort_by(|left, right| left.module.cmp(&right.module));

    let ordinary = world.encode().expect("ordinary semantic world encoding");
    let (with_inventory, projects) = world.encode_with_project_inventory().expect("semantic world encoding with project inventory");

    assert_eq!(with_inventory, ordinary, "project collection must not change canonical bytes");
    assert!(projects.contains(&foreign), "nested semantic-only project owner must be inventoried");
}

fn sample_query_image() -> QueryImageV4 {
    QueryImageV4 {
        supported_roots: vec![QueryRootEntryV4 {
            key: PortableQueryKeyV1 {
                kind: PortableQueryKindV1::LinkedInterface,
                owner: PortableQueryOwnerKeyV1::Module(StableModuleRef {
                    project: StableProjectRef::Builtin {
                        namespace: "universe".into(),
                        version: "0.1.0".into(),
                    },
                    path: Box::new([]),
                }),
            },
            product: QueryProductRefV3 {
                family: PortableQueryProductFamilyV1::Module,
                row_index: 0,
            },
            input_fingerprint: 1,
            product_fingerprint: 2,
            dependency_indices: vec![],
        }],
        dependency_evidence: vec![],
    }
}

fn nominal_shell(owner: StableDeclarationRef) -> DeclarationShellV2 {
    DeclarationShellV2 {
        declaration: owner,
        self_type: ArtifactTypeId(0),
        class_object_type: ArtifactTypeId(0),
        kind: ArtifactKindId(0),
        binding: DeclarationTypeBindingV1 {
            semantic_form: DeclarationSemanticFormV1::Ordinary,
            runtime_owner: None,
        },
        parameters: vec![],
        generic_signature: None,
        supertype_template: None,
        intrinsic_type_owner: None,
    }
}

fn declaration_shell_image(owner: StableDeclarationRef, family: PortableQueryProductFamilyV1) -> QueryImageV4 {
    QueryImageV4 {
        supported_roots: vec![QueryRootEntryV4 {
            key: PortableQueryKeyV1 {
                kind: PortableQueryKindV1::DeclarationShell,
                owner: PortableQueryOwnerKeyV1::Declaration(owner),
            },
            product: QueryProductRefV3 { family, row_index: 0 },
            input_fingerprint: 0,
            product_fingerprint: 0,
            dependency_indices: vec![],
        }],
        dependency_evidence: vec![],
    }
}

#[test]
fn declaration_shell_query_round_trips_nominal_and_alias_product_families() {
    let module = StableModuleRef {
        project: StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        },
        path: vec!["core".into()].into_boxed_slice(),
    };
    let nominal_owner = StableDeclarationRef {
        module: module.clone(),
        path: vec!["Nominal".into()].into_boxed_slice(),
    };
    let mut nominal_world = sample_semantic_world();
    nominal_world.structural_contracts.declaration_shells.push(nominal_shell(nominal_owner.clone()));
    let mut nominal_image = declaration_shell_image(nominal_owner, PortableQueryProductFamilyV1::NominalDeclarationShell);
    nominal_image
        .refresh_portable_structural_fingerprints_v4(&nominal_world)
        .expect("nominal shell portable fingerprints");
    nominal_image
        .validate_product_references(&phalcom_compiler_artifact::module_world_v5::ModuleWorldV5::from(sample_module_world()), &nominal_world)
        .expect("nominal shell row owner");
    nominal_image
        .validate_portable_structural_fingerprints_v4(&nominal_world)
        .expect("nominal shell portable fingerprint validation");
    let nominal_encoded = nominal_image.encode().expect("encode V4 nominal query image");
    assert_eq!(
        QueryImageV4::decode(&nominal_encoded, &ValidationLimits::default()).expect("decode V4 nominal query image"),
        nominal_image
    );
    let mut nominal_wrong_family = nominal_image.clone();
    nominal_wrong_family.supported_roots[0].product.family = PortableQueryProductFamilyV1::TypeAlias;
    assert!(
        nominal_wrong_family
            .validate_product_references(&phalcom_compiler_artifact::module_world_v5::ModuleWorldV5::from(sample_module_world()), &nominal_world)
            .is_err()
    );

    let (alias_world, alias_owner) = semantic_world_with_static_alias(Some("Alias"), false);
    let alias_index = alias_world
        .structural_contracts
        .type_aliases
        .iter()
        .position(|row| row.declaration == alias_owner)
        .expect("alias row");
    let mut alias_image = declaration_shell_image(alias_owner.clone(), PortableQueryProductFamilyV1::TypeAlias);
    alias_image.supported_roots[0].product.row_index = alias_index as u32;
    alias_image.dependency_evidence.push(DependencyEvidenceEntryV4 {
        key: PortableQueryKeyV1 {
            kind: PortableQueryKindV1::DeclarationShell,
            owner: PortableQueryOwnerKeyV1::Declaration(alias_owner.clone()),
        },
        stable_aggregate_owner: PortableQueryOwnerKeyV1::Declaration(alias_owner),
        product_fingerprint: 0,
    });
    alias_image
        .refresh_portable_structural_fingerprints_v4(&alias_world)
        .expect("alias shell and dependency portable fingerprints");
    alias_image
        .validate_product_references(&phalcom_compiler_artifact::module_world_v5::ModuleWorldV5::from(sample_module_world()), &alias_world)
        .expect("alias shell row owner");
    alias_image
        .validate_portable_structural_fingerprints_v4(&alias_world)
        .expect("alias shell portable fingerprint validation");

    let limits = ValidationLimits::default();
    let encoded = alias_image.encode().expect("encode V4 alias query image");
    let decoded = QueryImageV4::decode(&encoded, &limits).expect("decode V4 alias query image");
    assert_eq!(decoded, alias_image);
    decoded
        .validate_portable_structural_fingerprints_v4(&alias_world)
        .expect("decoded alias evidence remains valid");

    let mut mutated_alias = alias_world.clone();
    mutated_alias.structural_contracts.type_aliases[alias_index].structural_form = "Int".into();
    assert!(alias_image.validate_portable_structural_fingerprints_v4(&mutated_alias).is_err());

    let mut bad_input = alias_image.clone();
    bad_input.supported_roots[0].input_fingerprint ^= 1;
    assert!(bad_input.validate_portable_structural_fingerprints_v4(&alias_world).is_err());
    let mut bad_product = alias_image.clone();
    bad_product.supported_roots[0].product_fingerprint ^= 1;
    assert!(bad_product.validate_portable_structural_fingerprints_v4(&alias_world).is_err());
    let mut bad_dependency = alias_image.clone();
    bad_dependency.dependency_evidence[0].product_fingerprint ^= 1;
    assert!(bad_dependency.validate_portable_structural_fingerprints_v4(&alias_world).is_err());

    let mut wrong_index = alias_image.clone();
    wrong_index.supported_roots[0].product.row_index = u32::MAX;
    assert!(wrong_index.validate_product_references(&phalcom_compiler_artifact::module_world_v5::ModuleWorldV5::from(sample_module_world()), &alias_world).is_err());
    let mut wrong_owner = alias_image;
    wrong_owner.supported_roots[0].key.owner = PortableQueryOwnerKeyV1::Declaration(StableDeclarationRef {
        module: alias_world.structural_contracts.type_aliases[alias_index].declaration.module.clone(),
        path: vec!["Different".into()].into_boxed_slice(),
    });
    assert!(wrong_owner.validate_product_references(&phalcom_compiler_artifact::module_world_v5::ModuleWorldV5::from(sample_module_world()), &alias_world).is_err());

    // A valid logical query may not select the nominal table when only an
    // alias row exists for its owner.
    let wrong_family = declaration_shell_image(
        alias_world.structural_contracts.type_aliases[alias_index].declaration.clone(),
        PortableQueryProductFamilyV1::NominalDeclarationShell,
    );
    assert!(wrong_family.validate_product_references(&phalcom_compiler_artifact::module_world_v5::ModuleWorldV5::from(sample_module_world()), &alias_world).is_err());

    let mut ambiguous_world = alias_world;
    let ambiguous_owner = ambiguous_world.structural_contracts.type_aliases[alias_index].declaration.clone();
    ambiguous_world.structural_contracts.declaration_shells.push(nominal_shell(ambiguous_owner));
    assert!(ambiguous_world.validate(&limits).is_err(), "one owner cannot have nominal and alias authority");
    let ambiguous_query = declaration_shell_image(
        ambiguous_world.structural_contracts.type_aliases[alias_index].declaration.clone(),
        PortableQueryProductFamilyV1::TypeAlias,
    );
    assert!(ambiguous_query.validate_product_references(&phalcom_compiler_artifact::module_world_v5::ModuleWorldV5::from(sample_module_world()), &ambiguous_world).is_err());
}

fn sample_payload() -> CompilerArtifactPayload {
    let pkg_id = TeloscPackageIdentity::universe();
    let header = TeloscHeaderV2 {
        container_schema: TELOSC_CONTAINER_SCHEMA_V2,
        minimum_reader_schema: TELOSC_MIN_READER_SCHEMA_V2,
        package_identity: pkg_id.clone(),
        semantic_model_version: 1,
        stable_identity_schema: 1,
        module_semantic_abi: MODULE_SEMANTIC_ABI_V2,
        semantic_artifact_abi: SEMANTIC_ARTIFACT_ABI_V3,
        query_fingerprint_abi: QUERY_FINGERPRINT_ABI_V3,
        native_surface_schema_version: 1,
        native_catalog_fingerprint: [5; 16],
        type_metadata_schema_version: 13,
        capabilities: TeloscCapabilities::CANONICAL_COMPILER_IMAGE_V1 | TeloscCapabilities::RUNTIME_PUBLIC_METADATA,
        source_digest: ArtifactDigest::ZERO,
        interface_digest: ArtifactDigest::ZERO,
        semantic_digest: ArtifactDigest::ZERO,
        query_digest: ArtifactDigest::ZERO,
        metadata_digest: ArtifactDigest::ZERO,
        artifact_digest: ArtifactDigest::ZERO,
        section_count: 4,
    };

    CompilerArtifactPayload {
        header,
        module_world: sample_module_world(),
        semantic_world: sample_semantic_world(),
        query_image: sample_query_image(),
        metadata: sample_metadata_section(&pkg_id),
    }
}

#[test]
fn complete_compiler_artifact_round_trips_through_binary_container() {
    let payload = sample_payload();
    let encoded = payload.encode().expect("encode compiler artifact");
    assert!(!encoded.is_empty());
    assert_eq!(&encoded[0..6], b"TELOSC");

    let limits = ValidationLimits::default();
    let mut budget = AllocationBudget::new(32 * 1024 * 1024);
    let restored = CompilerArtifactPayload::decode_and_validate(&encoded, &limits, None, &mut budget).expect("decode and validate compiler artifact");

    assert_eq!(restored.header.package_identity, payload.header.package_identity);
    assert_eq!(restored.module_world.modules.len(), payload.module_world.modules.len());
    assert_eq!(restored.module_world.modules[0].path, payload.module_world.modules[0].path);
    assert_eq!(restored.module_world.modules[1].path, payload.module_world.modules[1].path);
    assert_eq!(restored.semantic_world.type_graph.types.len(), payload.semantic_world.type_graph.types.len());
    assert_eq!(restored.query_image.supported_roots.len(), payload.query_image.supported_roots.len());
    assert_eq!(restored.metadata.package_digest, payload.metadata.package_digest);
    assert!(budget.remaining() < 32 * 1024 * 1024);
}

fn runtime_requirement() -> phalcom_compiler_artifact::semantic::world::RuntimeDependencyRequirementV1 {
    use phalcom_compiler_artifact::semantic::world::*;
    use phalcom_type_meta::identity::StableDeclarationRef;
    let world = sample_module_world();
    let importer = world.modules[0].identity.clone();
    let dependency = world.modules[1].identity.clone();
    RuntimeDependencyRequirementV1 {
        importer: importer.clone(),
        dependency: dependency.clone(),
        range: SpanV1 { start: 12, end: 18 },
        reason: RuntimeDependencyReasonV1::RuntimeDeclarationReference,
        read_target: Some(LinkedReadSpecV1::Binding {
            symbol: StableDeclarationRef {
                module: dependency.clone(),
                path: vec!["First".into()].into_boxed_slice(),
            },
        }),
        provenance: RuntimeRequirementProvenanceV1 {
            consumer: importer,
            producer: RuntimeRequirementProducerV1::DeclarationRead,
            target: RuntimeRequirementTargetV1::Binding {
                module: dependency,
                name: "First".into(),
            },
            product: RuntimeRequirementProductV1::RuntimeObject,
            timing: RuntimeRequirementTimingV1::DuringModuleInitializer,
        },
    }
}

#[test]
fn metadata_type_owner_availability_requires_its_persisted_runtime_requirement() {
    use phalcom_compiler_artifact::semantic::world::{
        RuntimeDependencyReasonV1, RuntimeDependencyRequirementV1, RuntimeRequirementProducerV1, RuntimeRequirementProductV1,
        RuntimeRequirementProvenanceV1, RuntimeRequirementTargetV1, RuntimeRequirementTimingV1,
    };

    let modules = sample_module_world();
    let importer = modules.modules[0].identity.clone();
    let dependency = modules.modules[1].identity.clone();
    let requirement = RuntimeDependencyRequirementV1 {
        importer: importer.clone(),
        dependency: dependency.clone(),
        range: SpanV1 { start: 1181, end: 1216 },
        reason: RuntimeDependencyReasonV1::RuntimeDeclarationReference,
        read_target: None,
        provenance: RuntimeRequirementProvenanceV1 {
            consumer: importer,
            producer: RuntimeRequirementProducerV1::MetadataTypeOwner,
            target: RuntimeRequirementTargetV1::Module { module: dependency },
            product: RuntimeRequirementProductV1::CanonicalIdentity,
            timing: RuntimeRequirementTimingV1::BeforeSourceExecution,
        },
    };
    let mut semantic = sample_semantic_world();
    semantic.module_execution_authority.runtime_requirements = vec![requirement.clone()];

    let semantic_bytes = semantic.encode().expect("semantic execution authority encodes");
    let restored_semantic = SemanticWorldV4::decode(&semantic_bytes, &ValidationLimits::default()).expect("semantic execution authority round-trips");
    assert_eq!(restored_semantic.module_execution_authority.runtime_requirements, vec![requirement.clone()]);

    let module_world = ModuleWorldV3::new(
        modules,
        vec![RuntimeAvailabilityWitnessV1 {
            importer: ArtifactModuleId(0),
            dependency: ArtifactModuleId(1),
            range: SpanV1 { start: 1181, end: 1216 },
            reason: RuntimeReasonV1::RuntimeDeclarationReference,
            read_target: None,
            target: TargetV1::Module { module: ArtifactModuleId(1) },
            product: AvailabilityProductV1::CanonicalIdentity,
            producer: AvailabilityRequirementProducerV1::MetadataTypeOwner,
            required_at: AvailabilityTimingV1::BeforeSourceExecution,
            authority: AvailabilityAuthorityV1::SemanticIdentityCatalog,
            available_at: AvailabilityTimingV1::BeforeSourceExecution,
        }],
    );
    let module_bytes = module_world.encode().expect("finalized module world encodes");
    let mut budget = AllocationBudget::new(1024 * 1024);
    let restored_module = ModuleWorldV3::decode_with_allocation_budget(&module_bytes, &ValidationLimits::default(), &mut budget)
        .expect("finalized module world round-trips");
    assert_eq!(restored_module, module_world);
    restored_module
        .validate_execution_authority(&restored_semantic.module_execution_authority)
        .expect("persisted compiler-derived requirement authorizes its exact omitted edge");

    let mut missing_authority = restored_semantic.module_execution_authority.clone();
    missing_authority.runtime_requirements.clear();
    let error = restored_module.validate_execution_authority(&missing_authority).unwrap_err();
    assert!(error.to_string().contains("availability witness has no matching runtime requirement"), "{error}");
}

fn assert_v4_validation_matches_projected_v3(world: &SemanticWorldV4, case: &str, expected_valid: bool) {
    let limits = ValidationLimits::default();
    let projected = world.to_v3_for_validation().validate(&limits);
    let actual = world.validate(&limits);
    assert_eq!(
        actual.is_ok(),
        projected.is_ok(),
        "V4 validation outcome diverged from the projected-V3 oracle for {case}: V4={actual:?}, projected V3={projected:?}"
    );
    assert_eq!(
        actual.is_ok(),
        expected_valid,
        "unexpected validation outcome for {case}: V4={actual:?}, projected V3={projected:?}"
    );
}

fn sample_callable_analysis() -> phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV2 {
    use phalcom_compiler_artifact::module_world::SpanV1;
    use phalcom_compiler_artifact::semantic::world::*;
    use phalcom_type_meta::identity::{StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide};

    let module = sample_module_world().modules[1].identity.clone();
    let callable = StableCallableRef {
        owner: StableCallableOwnerRef::Declaration(StableDeclarationRef {
            module: module.clone(),
            path: vec!["Body".into()].into_boxed_slice(),
        }),
        side: StableDispatchSide::Instance,
        selector: "body".into(),
    };
    let explanation = CallableExplanationNodeV1 {
        step: CallableExplanationStepV1::UnknownBoundary {
            reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
            source: None,
        },
        rule: CallableDerivationRuleV1::UnknownPropagation,
        status: EvidenceStatusV1::Established,
        origin: EvidenceOriginV1::Syntax,
        evidence: vec![],
        parents: vec![],
    };
    let source = CallableAnalysisRecordV1 {
        callable: callable.clone(),
        body_source_owner: None,
        body_range: SpanV1 { start: 0, end: 8 },
        status: CallableAnalysisStatusV1::Complete,
        exits_unreachable: true,
        flow_graph: CallableFlowGraphV1 {
            nodes: vec![],
            edges: vec![],
            entry: None,
            exits: vec![],
        },
        constructor_super_initializer: None,
        expressions: vec![],
        conformance_evidence: vec![],
        bindings: vec![CallableBindingRowV1 {
            ordinal: 0,
            name: "value".into(),
            parameter: None,
            range: SpanV1 { start: 0, end: 1 },
            contract: None,
            current: TypeKnowledgeV1::Unknown {
                reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
            },
            denotation: None,
            consistency: CallableBindingConsistencyV1::Unconstrained,
            causal_invalidity: CallableCausalInvalidityV1::Clean,
            mutable: false,
            initialization: CallableInitializationStateV1::DefinitelyInitialized,
            version: 0,
            explanation: None,
            authority: CallableBindingAuthorityV1::ModuleValue {
                module: module.clone(),
                name: "value".into(),
                import_binding: None,
            },
        }],
        for_iteration_lanes: vec![],
        positional_expansions: vec![],
        known_invalid_positional_expansions: vec![],
        sequence_literals: vec![],
        associated_resolutions: vec![],
        callable_reference_resolutions: vec![],
        family_applications: vec![],
        local_proof_graph: Some(CallableProofGraphV1 {
            owner: callable,
            scopes: vec![CallableProofScopeV1 { parent: None }],
            rigids: vec![],
        }),
        match_resolutions: vec![],
        pattern_site_resolutions: vec![],
        entry_flow: CallableFlowStateSummaryV1 {
            bindings: vec![],
            fields: vec![],
            fact_count: 0,
        },
        normal_returns: vec![],
        throws: vec![],
        return_validation: ReturnContractValidationV1::NotApplicable,
        explanations: vec![explanation],
        return_explanation: Some(0),
        diagnostic_cause_count: 0,
        diagnostics: vec![],
        dependencies: vec![],
        semantic_dependencies: vec![],
    };

    CallableAnalysisRecordV2::from_v1_with_observed_import_phases(source, vec![])
}

#[test]
fn semantic_world_v4_validation_matches_projected_v3_for_shared_hostile_cases() {
    use phalcom_compiler_artifact::module_world::SpanV1;
    use phalcom_compiler_artifact::semantic::aggregates::TraitRefV1;
    use phalcom_compiler_artifact::semantic::static_value::ArtifactStaticValueId;
    use phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId;
    use phalcom_compiler_artifact::semantic::world::*;
    use phalcom_type_meta::identity::{StableDeclarationRef, StableImplRef};

    assert_v4_validation_matches_projected_v3(&sample_semantic_world(), "empty valid V4 world", true);

    let (structural, _) = semantic_world_with_static_alias(Some("Alias"), true);
    assert_v4_validation_matches_projected_v3(&structural, "alias and hierarchy rows", true);

    let (missing_alias, _) = semantic_world_with_static_alias(None, false);
    assert_v4_validation_matches_projected_v3(&missing_alias, "static definition lacks alias authority", false);

    let mut callable = sample_semantic_world();
    callable.callable_analyses.push(sample_callable_analysis());
    assert_v4_validation_matches_projected_v3(&callable, "callable proof and explanation records", true);
    let mut cyclic_proof = callable.clone();
    cyclic_proof.callable_analyses[0]
        .local_proof_graph
        .as_mut()
        .expect("sample callable proof graph")
        .scopes[0]
        .parent = Some(0);
    assert_v4_validation_matches_projected_v3(&cyclic_proof, "callable proof scope cycle", false);
    let mut dangling_explanation = callable.clone();
    dangling_explanation.callable_analyses[0].return_explanation = Some(1);
    assert_v4_validation_matches_projected_v3(&dangling_explanation, "callable return explanation is dangling", false);

    let module = sample_module_world().modules[1].identity.clone();
    let source_body = callable.callable_analyses[0].callable.clone();
    let symbol = StaticConstantSymbolV1 {
        module: module.clone(),
        name: "value".into(),
    };
    let mut module_bindings = callable.clone();
    module_bindings.module_static_bindings.push(ModuleStaticBindingIndexV1 {
        module: module.clone(),
        bindings: vec![ModuleStaticBindingRecordV1 {
            symbol: symbol.clone(),
            source_body,
            source_binding_ordinal: 0,
            phase: ModuleStaticBindingPhaseV1::Runtime,
            declaration_source: SpanV1 { start: 0, end: 8 },
            initializer_source: None,
        }],
        visible_names: vec![ModuleStaticBindingVisibleNameV1 {
            name: "value".into(),
            symbol: symbol.clone(),
        }],
    });
    assert_v4_validation_matches_projected_v3(&module_bindings, "module static binding authority", true);
    let mut imported_binding = module_bindings.clone();
    imported_binding.callable_analyses[0].bindings[0].authority = CallableBindingAuthorityV1::ModuleValue {
        module: module.clone(),
        name: "value".into(),
        import_binding: Some(1),
    };
    assert_v4_validation_matches_projected_v3(&imported_binding, "module static binding uses an imported alias", false);

    let missing_read = StaticConstantSymbolV1 {
        module: module.clone(),
        name: "missing".into(),
    };
    let mut bad_constant = sample_semantic_world();
    bad_constant.static_constants.push(StaticConstantRecordV1 {
        symbol,
        declaration_source: SpanV1 { start: 0, end: 1 },
        initializer_source: SpanV1 { start: 2, end: 3 },
        input_fingerprint: 0,
        dependencies: vec![],
        static_constant_reads: vec![missing_read],
        record_key_static_constant_reads: vec![],
        result_type: Some(ArtifactTypeId(0)),
        outcome: StaticConstantOutcomeV1::Reduced {
            value: ArtifactStaticValueId(0),
        },
        lkg_value: None,
        diagnostics: vec![],
    });
    assert_v4_validation_matches_projected_v3(&bad_constant, "static constant read lacks a canonical owner", false);

    let trait_owner = StableDeclarationRef {
        module: module.clone(),
        path: vec!["Trait".into()].into_boxed_slice(),
    };
    let mut bad_trait = sample_semantic_world();
    bad_trait.trait_surfaces.push(TraitSurfaceRecordV1 {
        declaration: trait_owner.clone(),
        generic_signature: None,
        header_source: EnumRequirementSourceV1 {
            module: module.clone(),
            range: SpanV1 { start: 2, end: 1 },
        },
        members: vec![],
        associated_types: vec![],
        diagnostics: vec![],
    });
    assert_v4_validation_matches_projected_v3(&bad_trait, "trait surface has an inverted source range", false);

    let mut bad_witness = sample_semantic_world();
    let implementation = StableImplRef {
        module: module.clone(),
        local_id: 7,
    };
    bad_witness.witness_plans.push(WitnessPlanV1 {
        impl_id: implementation.clone(),
        source_module: module.clone(),
        trait_ref_template: TraitRefV1 {
            declaration: trait_owner,
            arguments: vec![],
        },
        target_template: ArtifactTypeId(0),
        generic_signature: None,
        requirement_views: vec![],
        requirements: vec![],
        associated_type_plan: CallableConformanceAssociatedTypePlanV1 {
            impl_id: implementation,
            bindings: vec![],
            failures: vec![],
            diagnostics: vec![],
        },
        invalid_explicit_members: false,
        completeness: CallableConformanceCompletenessV1::Complete,
        diagnostics: vec![],
        conformance_evidence: vec![],
        fingerprint: 0,
        ordered_selected_callables: vec![],
        slot_layout_fingerprint: [0; 32],
    });
    assert_v4_validation_matches_projected_v3(&bad_witness, "witness plan references an absent trait surface", false);

    let surface_owner = StableDeclarationRef {
        module: module.clone(),
        path: vec!["Surface".into()].into_boxed_slice(),
    };
    let mut bad_dispatch = sample_semantic_world();
    bad_dispatch.dispatch_surfaces.push(DeclarationSurfaceRecordV1 {
        declaration: surface_owner.clone(),
        instance_callables: vec![],
        class_callables: vec![],
        fields: vec![FieldSignatureV1 {
            owner: surface_owner.clone(),
            name: "".into(),
            declared_type: None,
            knowledge: TypeKnowledgeV1::Unknown {
                reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
            },
            mutable: None,
            side: 0,
            visibility: phalcom_compiler_artifact::semantic::aggregates::VisibilityV1::Public,
            source: None,
        }],
    });
    assert_v4_validation_matches_projected_v3(&bad_dispatch, "dispatch surface has an empty field name", false);

    let mut bad_static_cases = sample_semantic_world();
    bad_static_cases.static_case_catalogs.push(StaticDomainCaseCatalogRecordV1 {
        domain: StaticCaseDomainV1::Nominal { declaration: surface_owner },
        cases: vec![],
    });
    assert_v4_validation_matches_projected_v3(&bad_static_cases, "static-case catalog references an absent domain", false);
}

#[test]
fn semantic_world_v4_only_authority_rejections_are_not_projected_to_v3() {
    use phalcom_compiler_artifact::semantic::world::{CallableObservedImportPhaseV1, DependencyPhaseV1, LinkedReadSpecV1};

    let (mut invalid_alias, _) = semantic_world_with_static_alias(Some("Alias"), true);
    invalid_alias.structural_contracts.type_aliases[0].source_span.start = 2;
    invalid_alias.structural_contracts.type_aliases[0].source_span.end = 1;
    let projected = invalid_alias.to_v3_for_validation().validate(&ValidationLimits::default());
    let actual = invalid_alias.validate(&ValidationLimits::default());
    assert!(projected.is_ok(), "the V3 projection drops the V2 alias source span: {projected:?}");
    assert!(actual.is_err(), "V4 must reject an inverted V2 alias source span");

    let mut duplicate_observation = sample_semantic_world();
    let mut analysis = sample_callable_analysis();
    analysis.observed_import_phases = vec![
        CallableObservedImportPhaseV1 {
            binding: 0,
            phase: DependencyPhaseV1::Runtime,
        },
        CallableObservedImportPhaseV1 {
            binding: 0,
            phase: DependencyPhaseV1::InterfaceOnly,
        },
    ];
    duplicate_observation.callable_analyses.push(analysis);
    let projected = duplicate_observation.to_v3_for_validation().validate(&ValidationLimits::default());
    let actual = duplicate_observation.validate(&ValidationLimits::default());
    assert!(projected.is_ok(), "the V3 projection intentionally omits observed import phases: {projected:?}");
    assert!(actual.is_err(), "V4 must reject duplicate observed import-phase bindings");

    let mut invalid_runtime_requirement = sample_semantic_world();
    let mut requirement = runtime_requirement();
    if let Some(LinkedReadSpecV1::Binding { symbol }) = &mut requirement.read_target {
        symbol.path = vec!["different".into()].into_boxed_slice();
    }
    invalid_runtime_requirement.module_execution_authority.runtime_requirements.push(requirement);
    let projected = invalid_runtime_requirement.to_v3_for_validation().validate(&ValidationLimits::default());
    let actual = invalid_runtime_requirement.validate(&ValidationLimits::default());
    assert!(projected.is_ok(), "the V3 projection intentionally omits runtime requirements: {projected:?}");
    assert!(actual.is_err(), "V4 must reject a runtime requirement with inconsistent read authority");
}

fn semantic_world_with_static_alias(structural_alias_name: Option<&str>, with_hierarchy: bool) -> (SemanticWorldV4, StableDeclarationRef) {
    use phalcom_compiler_artifact::module_world::SpanV1;
    use phalcom_compiler_artifact::semantic::aggregates::{HierarchyEdgeRecordV1, TypeAliasRecordV1};
    use phalcom_compiler_artifact::semantic::operational::OperationalOpV1;
    use phalcom_compiler_artifact::semantic::static_value::ArtifactStaticValueId;
    use phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId;
    use phalcom_compiler_artifact::semantic::world::{CanonicalStaticProgramV1, StaticDefinitionRecordV1, StaticDefinitionSignatureV1, StaticValueDomainV1};

    let mut world = sample_semantic_world();
    let module = world.module_execution_authority.import_phase_products[1].module.clone();
    let alias_owner = StableDeclarationRef {
        module: module.clone(),
        path: vec!["Alias".into()].into_boxed_slice(),
    };
    if let Some(alias_name) = structural_alias_name {
        let declaration = StableDeclarationRef {
            module: module.clone(),
            path: vec![alias_name.into()].into_boxed_slice(),
        };
        world.structural_contracts.type_aliases.push(TypeAliasRecordV1 {
            declaration,
            kind: ArtifactKindId(0),
            kind_shape: "Type".into(),
            generic_signature: None,
            form: ArtifactTypeId(0),
            structural_form: "Unit".into(),
            dependencies: vec![],
            source_span: SpanV1 { start: 0, end: 0 },
        });
    }
    if with_hierarchy {
        world.structural_contracts.hierarchy_edges.push(HierarchyEdgeRecordV1 {
            declaration: alias_owner.clone(),
            superclass: Some(StableDeclarationRef {
                module,
                path: vec!["Base".into()].into_boxed_slice(),
            }),
            supertype_template: None,
        });
    }

    let result_domain = StaticValueDomainV1::Kind { kind: ArtifactKindId(0) };
    world.static_definitions.push(StaticDefinitionRecordV1 {
        id: alias_owner.clone(),
        signature: StaticDefinitionSignatureV1 {
            parameters: vec![],
            result_domain: result_domain.clone(),
            generic_signature: None,
        },
        source: SpanV1 { start: 0, end: 0 },
        alias_info: Some(alias_owner.clone()),
        canonical_program: CanonicalStaticProgramV1 {
            owner: alias_owner.clone(),
            parameter_count: 0,
            local_count: 0,
            root: OperationalOpV1::Constant {
                value: ArtifactStaticValueId(0),
            },
            applications: vec![],
            kinded_parameters: vec![],
            static_constant_reads: vec![],
            dependencies: vec![],
            result: result_domain,
            input_fingerprint: 0,
            module_binding_fingerprint: None,
        },
        diagnostics: vec![],
    });
    (world, alias_owner)
}

#[test]
fn runtime_obligation_duplicates_reject_exact_and_producer_only_repetitions() {
    use phalcom_compiler_artifact::semantic::world::*;
    let original = runtime_requirement();
    let mut other_producer = original.clone();
    other_producer.provenance.producer = RuntimeRequirementProducerV1::CompilerGeneratedRead;
    for duplicate in [original.clone(), other_producer] {
        let mut world = sample_semantic_world();
        world.module_execution_authority.runtime_requirements = vec![original.clone(), duplicate];
        let error = world.validate(&ValidationLimits::default()).unwrap_err();
        assert!(error.to_string().contains("duplicate runtime dependency requirement"), "{error}");
    }
}

#[test]
fn runtime_obligations_at_one_span_preserve_each_distinct_capability() {
    use phalcom_compiler_artifact::semantic::world::*;
    let original = runtime_requirement();
    let mut other_binding = original.clone();
    if let Some(LinkedReadSpecV1::Binding { symbol }) = &mut other_binding.read_target {
        symbol.path = vec!["Second".into()].into_boxed_slice();
    }
    if let RuntimeRequirementTargetV1::Binding { name, .. } = &mut other_binding.provenance.target {
        *name = "Second".into();
    }
    let mut other_product = original.clone();
    other_product.provenance.product = RuntimeRequirementProductV1::MetadataDescriptor;
    let mut other_timing = original.clone();
    other_timing.provenance.timing = RuntimeRequirementTimingV1::AfterBootstrap;
    let mut other_reason = original.clone();
    other_reason.reason = RuntimeDependencyReasonV1::SelectiveValueImport;
    let mut graph_only = original.clone();
    graph_only.read_target = None;
    let variants = vec![original, other_binding, other_product, other_timing, other_reason, graph_only];
    let mut world = sample_semantic_world();
    world.module_execution_authority.runtime_requirements = variants.clone();
    let bytes = world.encode().expect("distinct obligations at one span must encode");
    let decoded = SemanticWorldV4::decode(&bytes, &ValidationLimits::default()).expect("distinct obligations must decode");
    assert_eq!(decoded.module_execution_authority.runtime_requirements, variants);

    world
        .module_execution_authority
        .runtime_requirements
        .sort_unstable_by(RuntimeDependencyRequirementV1::canonical_cmp);
    let expected = world.encode().unwrap();
    world.module_execution_authority.runtime_requirements.reverse();
    world
        .module_execution_authority
        .runtime_requirements
        .sort_unstable_by(RuntimeDependencyRequirementV1::canonical_cmp);
    assert_eq!(world.encode().unwrap(), expected, "full-record ordering must ignore input order");
}

#[test]
fn runtime_obligations_reject_inconsistent_read_and_provenance_targets() {
    use phalcom_compiler_artifact::semantic::world::*;
    let original = runtime_requirement();
    let mut wrong_read_module = original.clone();
    if let Some(LinkedReadSpecV1::Binding { symbol }) = &mut wrong_read_module.read_target {
        symbol.module = original.importer.clone();
    }
    let mut wrong_target_module = original.clone();
    if let RuntimeRequirementTargetV1::Binding { module, .. } = &mut wrong_target_module.provenance.target {
        *module = original.importer.clone();
    }
    let mut wrong_target_name = original.clone();
    if let RuntimeRequirementTargetV1::Binding { name, .. } = &mut wrong_target_name.provenance.target {
        *name = "Different".into();
    }
    let mut wrong_target_kind = original.clone();
    wrong_target_kind.provenance.target = RuntimeRequirementTargetV1::Module {
        module: original.dependency.clone(),
    };
    let mut wrong_consumer = original.clone();
    wrong_consumer.provenance.consumer = original.dependency;
    for (requirement, expected) in [
        (wrong_read_module, "read target symbol module mismatch"),
        (wrong_target_module, "provenance target module mismatch"),
        (wrong_target_name, "read target disagrees with provenance target"),
        (wrong_target_kind, "read target disagrees with provenance target"),
        (wrong_consumer, "provenance consumer must match importer"),
    ] {
        let mut world = sample_semantic_world();
        world.module_execution_authority.runtime_requirements = vec![requirement];
        let error = world.validate(&ValidationLimits::default()).unwrap_err();
        assert!(error.to_string().contains(expected), "expected {expected}: {error}");
    }
}

#[test]
fn v4_static_aliases_survive_validation_projection_with_or_without_hierarchy() {
    use phalcom_compiler_artifact::module_world::SpanV1;
    use phalcom_compiler_artifact::semantic::aggregates::HierarchyAndAliasV1;
    use phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId;

    let limits = ValidationLimits::default();
    let (alias_only, alias_owner) = semantic_world_with_static_alias(Some("Alias"), false);
    let bytes = alias_only.encode().expect("alias-only V4 world must validate and encode");
    let decoded = SemanticWorldV4::decode(&bytes, &limits).expect("alias-only V4 world must decode");
    assert_eq!(decoded, alias_only);
    assert_eq!(
        decoded.to_v3_for_validation().structural_contracts.hierarchy_and_aliases,
        vec![HierarchyAndAliasV1 {
            declaration: alias_owner.clone(),
            superclass: None,
            aliased_type: Some(ArtifactTypeId(0)),
        }],
        "an alias-only owner must be represented in the legacy validation projection"
    );

    let (combined, combined_owner) = semantic_world_with_static_alias(Some("Alias"), true);
    let bytes = combined.encode().expect("hierarchy plus alias V4 world must encode");
    let decoded = SemanticWorldV4::decode(&bytes, &limits).expect("hierarchy plus alias V4 world must decode");
    assert_eq!(decoded, combined);
    assert_eq!(
        decoded.to_v3_for_validation().structural_contracts.hierarchy_and_aliases,
        vec![HierarchyAndAliasV1 {
            declaration: combined_owner,
            superclass: Some(StableDeclarationRef {
                module: decoded.module_execution_authority.import_phase_products[1].module.clone(),
                path: vec!["Base".into()].into_boxed_slice(),
            }),
            aliased_type: Some(ArtifactTypeId(0)),
        }],
        "the alias form must merge into its existing hierarchy row without dropping the superclass"
    );
    assert_eq!(decoded.structural_contracts.type_aliases[0].source_span, SpanV1 { start: 0, end: 0 });
}

#[test]
fn v4_static_definition_rejects_missing_or_foreign_alias_authority() {
    let limits = ValidationLimits::default();
    let (missing, _) = semantic_world_with_static_alias(None, false);
    let error = missing.validate(&limits).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("static definition alias projection lacks structural alias authority"),
        "{error}"
    );

    let (foreign, _) = semantic_world_with_static_alias(Some("OtherAlias"), false);
    let error = foreign.validate(&limits).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("static definition alias projection lacks structural alias authority"),
        "{error}"
    );
}

#[test]
fn v4_static_alias_projection_handles_out_of_order_and_duplicate_rows() {
    use phalcom_compiler_artifact::module_world::SpanV1;
    use phalcom_compiler_artifact::semantic::aggregates::{HierarchyAndAliasV1, HierarchyEdgeRecordV1, TypeAliasRecordV1};
    use phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId;

    let limits = ValidationLimits::default();
    let (mut world, alias_owner) = semantic_world_with_static_alias(Some("Alias"), true);
    let module = alias_owner.module.clone();
    let earlier_owner = StableDeclarationRef {
        module: module.clone(),
        path: vec!["Aardvark".into()].into_boxed_slice(),
    };
    world.structural_contracts.hierarchy_edges.push(HierarchyEdgeRecordV1 {
        declaration: earlier_owner.clone(),
        superclass: Some(StableDeclarationRef {
            module: module.clone(),
            path: vec!["EarlierBase".into()].into_boxed_slice(),
        }),
        supertype_template: None,
    });
    let earlier_alias = TypeAliasRecordV1 {
        declaration: earlier_owner.clone(),
        kind: ArtifactKindId(0),
        kind_shape: "Type".into(),
        generic_signature: None,
        form: ArtifactTypeId(0),
        structural_form: "Unit".into(),
        dependencies: vec![],
        source_span: SpanV1 { start: 0, end: 0 },
    };
    world.structural_contracts.type_aliases.push(earlier_alias);
    world.structural_contracts.type_aliases.push(TypeAliasRecordV1 {
        declaration: alias_owner.clone(),
        kind: ArtifactKindId(0),
        kind_shape: "Type".into(),
        generic_signature: None,
        form: ArtifactTypeId(0),
        structural_form: "Unit".into(),
        dependencies: vec![],
        source_span: SpanV1 { start: 0, end: 0 },
    });

    let bytes = world.encode().expect("unsorted aliases and hierarchy rows must validate");
    let decoded = SemanticWorldV4::decode(&bytes, &limits).expect("unsorted alias rows must round trip");
    let rows = decoded.to_v3_for_validation().structural_contracts.hierarchy_and_aliases;
    assert_eq!(
        rows,
        vec![
            HierarchyAndAliasV1 {
                declaration: earlier_owner.clone(),
                superclass: Some(StableDeclarationRef {
                    module: module.clone(),
                    path: vec!["EarlierBase".into()].into_boxed_slice(),
                }),
                aliased_type: Some(ArtifactTypeId(0)),
            },
            HierarchyAndAliasV1 {
                declaration: alias_owner.clone(),
                superclass: Some(StableDeclarationRef {
                    module: module.clone(),
                    path: vec!["Base".into()].into_boxed_slice(),
                }),
                aliased_type: Some(ArtifactTypeId(0)),
            },
            HierarchyAndAliasV1 {
                declaration: alias_owner,
                superclass: None,
                aliased_type: Some(ArtifactTypeId(0)),
            },
        ],
        "projection must sort by owner while retaining every hierarchy and alias row"
    );
}

#[test]
fn compatibility_identity_verifies_matching_and_rejects_foreign() {
    let payload = sample_payload();
    let encoded = payload.encode().expect("encode compiler artifact");

    let limits = ValidationLimits::default();
    let mut initial_budget = AllocationBudget::new(32 * 1024 * 1024);
    let initial = CompilerArtifactPayload::decode_and_validate(&encoded, &limits, None, &mut initial_budget).unwrap();

    let matching_identity = CanonicalCompatibilityIdentity {
        package: initial.header.package_identity.clone(),
        source_digest: initial.header.source_digest,
        interface_digest: initial.header.interface_digest,
        semantic_digest: initial.header.semantic_digest,
        semantic_model_version: initial.header.semantic_model_version,
        stable_identity_schema: initial.header.stable_identity_schema,
        module_semantic_abi: initial.header.module_semantic_abi,
        semantic_artifact_abi: initial.header.semantic_artifact_abi,
        query_fingerprint_abi: initial.header.query_fingerprint_abi,
        native_surface_schema_version: initial.header.native_surface_schema_version,
        native_catalog_fingerprint: initial.header.native_catalog_fingerprint,
        type_metadata_schema_version: initial.header.type_metadata_schema_version,
    };

    let mut budget = AllocationBudget::new(32 * 1024 * 1024);
    assert!(CompilerArtifactPayload::decode_and_validate(&encoded, &limits, Some(&matching_identity), &mut budget).is_ok());

    let mut foreign_identity = matching_identity;
    foreign_identity.native_catalog_fingerprint = [99; 16];
    let mut budget2 = AllocationBudget::new(32 * 1024 * 1024);
    let err = CompilerArtifactPayload::decode_and_validate(&encoded, &limits, Some(&foreign_identity), &mut budget2);
    assert!(err.is_err());
}
