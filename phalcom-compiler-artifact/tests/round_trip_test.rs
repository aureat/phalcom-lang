use phalcom_compiler_artifact::digest::ArtifactDigest;
use phalcom_compiler_artifact::header::{
    MODULE_SEMANTIC_ABI_V2, QUERY_FINGERPRINT_ABI_V2, SEMANTIC_ARTIFACT_ABI_V3, TELOSC_CONTAINER_SCHEMA_V2, TELOSC_MIN_READER_SCHEMA_V2,
    TeloscCapabilities, TeloscHeaderV2,
};
use phalcom_compiler_artifact::identity::{CanonicalCompatibilityIdentity, TeloscPackageIdentity};
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::metadata::{MetadataPreflightLimits, MetadataSectionV1};
use phalcom_compiler_artifact::module_world::{ArtifactModuleId, ModuleKindV1, ModuleV2, ModuleWorldV2};
use phalcom_compiler_artifact::payload::CompilerArtifactPayload;
use phalcom_compiler_artifact::query::{
    PortableQueryKeyV1, PortableQueryKindV1, PortableQueryOwnerKeyV1, QueryImageV3, QueryProductRefV2, QueryRootEntryV3,
};
use phalcom_compiler_artifact::semantic::aggregates::StructuralPrimaryContractsV2;
use phalcom_compiler_artifact::semantic::static_value::{StaticValueGraphV1, StaticValueV1};
use phalcom_compiler_artifact::semantic::type_store::{ArtifactKindId, KindV1, TypeEntryV1, TypeGraphV1, TypeV1};
use phalcom_compiler_artifact::semantic::world::{
    ModuleExecutionAuthorityV1, ModuleImportPhaseProductV1, PhaseEvidenceCoverageV1, SemanticWorldV4,
};
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

fn sample_query_image() -> QueryImageV3 {
    QueryImageV3 {
        supported_roots: vec![QueryRootEntryV3 {
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
            product: QueryProductRefV2 {
                kind: PortableQueryKindV1::LinkedInterface,
                row_index: 0,
            },
            input_fingerprint: 1,
            product_fingerprint: 2,
            dependency_indices: vec![],
        }],
        dependency_evidence: vec![],
    }
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
        query_fingerprint_abi: QUERY_FINGERPRINT_ABI_V2,
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
    use phalcom_compiler_artifact::module_world::SpanV1;
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
