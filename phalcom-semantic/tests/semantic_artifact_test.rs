use phalcom_compiler_artifact::ArtifactError;
use phalcom_compiler_artifact::digest::ArtifactDigest;
use phalcom_compiler_artifact::header::{TELOSC_CONTAINER_SCHEMA_V2, TELOSC_MIN_READER_SCHEMA_V2, TeloscCapabilities, TeloscHeaderV2};
use phalcom_compiler_artifact::identity::TeloscPackageIdentity;
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::metadata::{MetadataPreflightLimits, MetadataSectionV1};
use phalcom_compiler_artifact::query::{PortableQueryKeyV1, PortableQueryKindV1, QueryImageV2, QueryProductRefV2, QueryRootEntryV2};
use phalcom_compiler_artifact::semantic::aggregates::StructuralPrimaryContractsV1;
use phalcom_compiler_artifact::semantic::static_value::{StaticValueGraphV1, StaticValueV1};
use phalcom_compiler_artifact::semantic::type_store::{ArtifactKindId, KindV1, TypeEntryV1, TypeGraphV1, TypeV1};
use phalcom_compiler_artifact::semantic::world::SemanticWorldV3;
use phalcom_semantic::artifact::{CompilerArtifactImage, decode_and_validate_artifact, encode_artifact, import_compiler_image};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::{
    ArtifactIdentityScheme, KindNode, KindNodeEntry, KindNodeId, MetadataFeatures, MetadataProfile, NATIVE_SURFACE_SCHEMA_VERSION, ProducerIdentity,
    SEMANTIC_MODEL_VERSION, SemanticMetadataBundle, SemanticMetadataHeader, TYPE_METADATA_SCHEMA_VERSION, TypeNode, TypeNodeEntry,
};
use std::sync::Arc;

fn sample_bundle() -> SemanticMetadataBundle {
    SemanticMetadataBundle {
        header: SemanticMetadataHeader {
            schema_version: TYPE_METADATA_SCHEMA_VERSION,
            semantic_model_version: SEMANTIC_MODEL_VERSION,
            producer: ProducerIdentity("phalcom-compiler".into()),
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

fn sample_image() -> CompilerArtifactImage {
    let pkg_id = TeloscPackageIdentity::universe();
    let header = TeloscHeaderV2 {
        container_schema: TELOSC_CONTAINER_SCHEMA_V2,
        minimum_reader_schema: TELOSC_MIN_READER_SCHEMA_V2,
        package_identity: pkg_id.clone(),
        semantic_model_version: 1,
        stable_identity_schema: 1,
        module_semantic_abi: 1,
        semantic_artifact_abi: 1,
        query_fingerprint_abi: 1,
        native_surface_schema_version: 1,
        native_catalog_fingerprint: [7; 16],
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

    let universe = Arc::new(phalcom_modules::ProjectUniverse::new());
    let root = phalcom_modules::ModuleId::universe_root();
    let iface = phalcom_modules::UnlinkedModuleInterface {
        id: root.clone(),
        kind: phalcom_modules::ModuleKind::Package,
        declarations: std::collections::BTreeMap::new(),
        exports: std::collections::BTreeMap::new(),
        imports: vec![],
        exposed_children: std::collections::BTreeSet::new(),
        metadata: phalcom_modules::ModuleMetadata::default(),
    };
    let mut interfaces = std::collections::BTreeMap::new();
    interfaces.insert(root.clone(), Arc::new(iface.clone()));
    let linked_iface = phalcom_modules::LinkedModuleInterface {
        module: root.clone(),
        kind: phalcom_modules::ModuleKind::Package,
        exports: std::collections::BTreeMap::new(),
        metadata: phalcom_modules::ModuleMetadata::default(),
    };
    let mut modules = std::collections::BTreeMap::new();
    modules.insert(
        root.clone(),
        phalcom_modules::LinkedModule {
            interface: linked_iface,
            bindings: phalcom_modules::ModuleBindingLayout::default(),
            linked_reads: vec![],
            runtime_dependencies: vec![],
        },
    );
    let linked = Arc::new(phalcom_modules::LinkedProgram {
        universe: universe.clone(),
        modules,
        graphs: phalcom_modules::ModuleGraphs::default(),
        entry: root.clone(),
        initialization_order: vec![root.clone()],
    });
    let mut unlinked_map = std::collections::BTreeMap::new();
    unlinked_map.insert(root.clone(), iface);
    let topology = Arc::new(phalcom_modules::ModuleTopology::from_parts(
        phalcom_modules::ResolverGeneration(0),
        &universe,
        &unlinked_map,
        &std::collections::BTreeMap::new(),
    ));
    let external_world = phalcom_modules::canonical_products::CanonicalExternalModuleWorld::from_parts(linked, interfaces, topology).unwrap();
    let module_world = phalcom_modules::compiler_artifact::project_module_world(&external_world, &ValidationLimits::default()).unwrap();

    let semantic_world = SemanticWorldV3 {
        type_graph: TypeGraphV1 {
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
        },
        structural_contracts: StructuralPrimaryContractsV1 {
            declaration_shells: vec![],
            hierarchy_and_aliases: vec![],
            generic_signatures: vec![],
            data_contracts: vec![],
            enum_contracts: vec![],
            variant_contracts: vec![],
        },
        enum_requirements: vec![],
        static_values: StaticValueGraphV1 {
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
        },
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
    };

    let query_image = QueryImageV2 {
        supported_roots: vec![QueryRootEntryV2 {
            key: PortableQueryKeyV1 {
                kind: PortableQueryKindV1::LinkedInterface,
                stable_owner_key: "".into(),
            },
            product: QueryProductRefV2 {
                kind: PortableQueryKindV1::LinkedInterface,
                row_index: 0,
            },
            input_fingerprint: 0,
            product_fingerprint: 0,
            dependency_indices: vec![],
        }],
        dependency_evidence: vec![],
    };

    let preflight_limits = MetadataPreflightLimits {
        max_metadata_bytes: 65536,
        max_token_count: 10000,
        max_container_count: 10000,
        max_nesting_depth: 32,
    };
    let metadata = MetadataSectionV1::from_bundle(
        &sample_bundle(),
        pkg_id.digest().0,
        &preflight_limits,
        &phalcom_type_meta::ValidationLimits::default(),
    )
    .unwrap();

    CompilerArtifactImage {
        header,
        module_world,
        semantic_world,
        query_image,
        metadata,
    }
}

#[test]
fn transactional_semantic_export_and_import_round_trip() {
    let image = sample_image();
    let bytes = encode_artifact(&image).expect("encode artifact");
    assert!(!bytes.is_empty());

    let limits = ValidationLimits::default();
    let mut decode_budget = AllocationBudget::new(32 * 1024 * 1024);
    let artifact = decode_and_validate_artifact(&bytes, &limits, None, &mut decode_budget).expect("decode and validate artifact");

    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let imported = import_compiler_image(artifact, &limits, &mut import_budget).expect("import compiler image");

    assert_eq!(imported.external_world.linked().modules.len(), 1);
    assert_eq!(imported.metadata_bundle.header.producer.0.as_ref(), "phalcom-compiler");
    assert_eq!(imported.semantic_world.type_graph.types.len(), 1);
    assert_eq!(imported.query_image.supported_roots.len(), 1);
}

#[test]
fn exact_allocation_accounting_n_succeeds_n_minus_1_fails_and_retry_succeeds() {
    let image = sample_image();
    let bytes = encode_artifact(&image).unwrap();
    let limits = ValidationLimits::default();

    // 1. Measure exact allocation N required for import
    let mut measure_budget = AllocationBudget::new(64 * 1024 * 1024);
    let artifact = decode_and_validate_artifact(&bytes, &limits, None, &mut measure_budget).unwrap();

    let mut import_measure_budget = AllocationBudget::new(64 * 1024 * 1024);
    let measure_artifact = artifact.clone();
    let _imported = import_compiler_image(measure_artifact, &limits, &mut import_measure_budget).unwrap();
    let n = import_measure_budget.requested();
    assert!(n > 0, "import must require non-zero allocation budget");

    // 2. Exact N succeeds leaving exactly 0 remaining
    let mut exact_budget = AllocationBudget::new(n);
    let exact_artifact = artifact.clone();
    let imported = import_compiler_image(exact_artifact, &limits, &mut exact_budget);
    assert!(imported.is_ok(), "exact budget N must succeed");
    assert_eq!(exact_budget.remaining(), 0, "exact budget N must leave 0 remaining");

    // 3. N - 1 fails with typed AllocationError::Exhausted
    let mut insufficient_budget = AllocationBudget::new(n - 1);
    let insufficient_artifact = artifact.clone();
    let failed = import_compiler_image(insufficient_artifact, &limits, &mut insufficient_budget);
    assert!(failed.is_err(), "budget N - 1 must fail");
    let err = failed.unwrap_err();
    assert!(
        matches!(err, ArtifactError::Allocation(AllocationError::Exhausted)),
        "failure must be typed AllocationError::Exhausted, got {err:?}"
    );
    // Crucial: verify fail-closed atomicity: insufficient_budget left uncommitted
    assert_eq!(insufficient_budget.requested(), 0, "failed import must not commit allocations to caller budget");

    // 4. Retry with sufficient budget succeeds
    let mut retry_budget = AllocationBudget::new(n);
    let retry_artifact = artifact.clone();
    let retried = import_compiler_image(retry_artifact, &limits, &mut retry_budget);
    assert!(retried.is_ok(), "retry with N must succeed");
    assert_eq!(retry_budget.remaining(), 0);
}

#[test]
fn hostile_corrupted_metadata_fails_closed_without_publishing() {
    let image = sample_image();
    let bytes = encode_artifact(&image).unwrap();
    let limits = ValidationLimits::default();

    let mut budget = AllocationBudget::new(32 * 1024 * 1024);
    let mut artifact = decode_and_validate_artifact(&bytes, &limits, None, &mut budget).unwrap();

    // Corrupt the metadata section package digest
    artifact.metadata.package_digest = [0xff; 32];

    let mut import_budget = AllocationBudget::new(32 * 1024 * 1024);
    let result = import_compiler_image(artifact, &limits, &mut import_budget);
    assert!(result.is_err(), "corrupted metadata digest must fail import");
    assert_eq!(import_budget.requested(), 0, "uncommitted import must debit 0 bytes");
}
