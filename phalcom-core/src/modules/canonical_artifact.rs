//! Deterministic `universe.telosc` builder, atomic writer, and source-free loader (`PERF003.C2.P7.T10/T11/T12`).

use crate::modules::canonical_universe::CanonicalCompilerImageView;
use crate::modules::compile::{CompiledProgram, EntrySelection, ProgramCompilationOptions, ProgramCompileError, ProgramCompiler};
use phalcom_compiler_artifact::container::{TeloscContainerBuilder, TeloscContainerView};
use phalcom_compiler_artifact::digest::{
    ArtifactDigest, ArtifactHasher, DOMAIN_METADATA, DOMAIN_MODULE_WORLD, DOMAIN_QUERY_IMAGE, DOMAIN_SEMANTIC_WORLD, DOMAIN_SOURCE,
};
use phalcom_compiler_artifact::header::{TELOSC_CONTAINER_SCHEMA_V2, TELOSC_MIN_READER_SCHEMA_V2, TeloscHeaderV2};
use phalcom_compiler_artifact::identity::TeloscPackageIdentity;
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::metadata::{METADATA_SECTION_SCHEMA_V1, MetadataPreflightLimits, MetadataSectionError, MetadataSectionV1};
use phalcom_compiler_artifact::module_world::MODULE_WORLD_SCHEMA_V1;
use phalcom_compiler_artifact::query::{QUERY_IMAGE_SCHEMA_V1, QueryImageV1};
use phalcom_compiler_artifact::section::{SectionFlags, SectionKind};
use phalcom_compiler_artifact::semantic::world::{SEMANTIC_WORLD_SCHEMA_V3, SemanticWorldV3};
use phalcom_native_meta::NATIVE_SURFACE_SCHEMA_VERSION;
use phalcom_semantic::artifact::export::{export_query_image, export_semantic_world};
use phalcom_semantic::artifact::import::validate_and_import_semantic_world_with_allocation_budget;
use phalcom_semantic::artifact::query::reconstruct_query_image_with_allocation_budget;
use phalcom_semantic::canonical_image::{CanonicalImageError, CanonicalSemanticWorldId, CanonicalUniverseFormalImage, CanonicalWorldProvenance};
use phalcom_semantic::static_value::StaticExportContext;
use phalcom_semantic::types::store::artifact::TypeExportContext;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::{SEMANTIC_MODEL_VERSION, SemanticMetadataBundle, TYPE_METADATA_SCHEMA_VERSION};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TeloscCapabilities {
    pub module_world: bool,
    pub semantic_world: bool,
    pub query_image: bool,
    pub runtime_public_metadata: bool,
    pub portable_executable: bool,
}

impl TeloscCapabilities {
    pub const fn compiler_only_v1() -> Self {
        Self {
            module_world: true,
            semantic_world: true,
            query_image: true,
            runtime_public_metadata: true,
            portable_executable: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TeloscBuildSummary {
    pub capabilities: TeloscCapabilities,
    pub module_world_bytes: usize,
    pub semantic_world_bytes: usize,
    pub query_image_bytes: usize,
    pub metadata_bytes: usize,
    pub artifact_digest: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct BuiltUniverseTelosc {
    pub bytes: Vec<u8>,
    pub summary: TeloscBuildSummary,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TeloscLoadTimings {
    pub file_read: Duration,
    pub container_validation: Duration,
    pub section_validation: Duration,
    pub relocation: Duration,
    pub publication: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum TeloscArtifactError {
    #[error("artifact validation error: {0}")]
    Artifact(#[from] phalcom_compiler_artifact::ArtifactError),
    #[error("artifact io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("metadata transport error: {0}")]
    Metadata(#[from] phalcom_compiler_artifact::metadata::MetadataTransportError),
    #[error("canonical image error: {0}")]
    CanonicalImage(#[from] CanonicalImageError),
    #[error("capability error: {0}")]
    Capability(&'static str),
    #[error("allocation budget exceeded: limit {limit_bytes} bytes")]
    BudgetExceeded { limit_bytes: usize },
}

fn allocation_error(error: AllocationError, limit_bytes: usize) -> TeloscArtifactError {
    match error {
        AllocationError::Exhausted | AllocationError::Overflow | AllocationError::Failed => TeloscArtifactError::BudgetExceeded { limit_bytes },
        AllocationError::UnaccountedBoundary => TeloscArtifactError::Artifact(phalcom_compiler_artifact::ArtifactError::Allocation(error)),
    }
}

impl From<MetadataSectionError> for TeloscArtifactError {
    fn from(err: MetadataSectionError) -> Self {
        Self::Metadata(phalcom_compiler_artifact::metadata::MetadataTransportError::Envelope(err))
    }
}

/// Immutable loaded Universe compiler artifact (`PERF003.C2.P7.T12`).
pub struct LoadedUniverseArtifact {
    pub capabilities: TeloscCapabilities,
    pub formal_image: Arc<CanonicalUniverseFormalImage>,
    pub metadata: Arc<SemanticMetadataBundle>,
    pub timings: TeloscLoadTimings,
    pub executable_reserved_p8: Option<()>,
}

impl LoadedUniverseArtifact {
    pub fn compile_user_entry(&self, entry: EntrySelection) -> Result<CompiledProgram, ProgramCompileError> {
        ProgramCompiler::compile_entry_selection_with_image(entry, self.formal_image.clone())
    }

    pub fn compile_user_entry_with_options(&self, entry: EntrySelection, options: &ProgramCompilationOptions) -> Result<CompiledProgram, ProgramCompileError> {
        ProgramCompiler::compile_entry_selection_with_options_and_image(entry, self.formal_image.clone(), options)
    }
}

/// Builds a deterministic `universe.telosc` compilation artifact from a canonical compiler image view (`PERF003.C2.P7.T11`).
pub fn build_universe_telosc(view: &CanonicalCompilerImageView<'_>, limits: &ValidationLimits) -> Result<BuiltUniverseTelosc, TeloscArtifactError> {
    let capabilities = TeloscCapabilities::compiler_only_v1();

    // 1. MODULE_WORLD
    let module_world_bytes = view
        .formal
        .modules()
        .encode_artifact(limits)
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;

    // 2. SEMANTIC_WORLD
    let mut types = TypeExportContext::new(&view.formal.snapshot().store);
    let mut statics = StaticExportContext::new(view.formal.snapshot().static_values.as_ref());
    phalcom_semantic::artifact::roots::register_semantic_world_roots(view.formal.snapshot(), &mut types, &mut statics)?;
    statics.register_type_roots(&view.formal.snapshot().static_values, &mut types, limits)?;
    types.close_graph(&view.formal.snapshot().store, limits)?;
    statics
        .close_graph(view.formal.snapshot().static_values.as_ref(), &types, limits)
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;
    let semantic_world = export_semantic_world(view.formal.snapshot(), &types, &statics, limits)
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;
    let semantic_world_bytes = semantic_world
        .encode()
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;

    // 3. QUERY_IMAGE
    let query_image = export_query_image(view.formal.query_image(), view.formal.dependency_fingerprints(), limits)
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;
    let query_image_bytes = query_image
        .encode()
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;

    // 4. METADATA
    let preflight_limits = MetadataPreflightLimits {
        max_metadata_bytes: limits.max_metadata_bytes as usize,
        max_token_count: u32::try_from(limits.max_semantic_entries_per_family).unwrap_or(u32::MAX),
        max_container_count: u32::try_from(limits.max_semantic_entries_per_family).unwrap_or(u32::MAX),
        max_nesting_depth: u16::try_from(limits.max_recursion_depth).unwrap_or(u16::MAX),
    };
    let bundle_limits = phalcom_type_meta::ValidationLimits::default();
    let package_identity = TeloscPackageIdentity::universe();
    let package_digest = *package_identity.digest().as_bytes();
    let metadata_section =
        MetadataSectionV1::from_bundle(view.metadata, package_digest, &preflight_limits, &bundle_limits).map_err(TeloscArtifactError::Metadata)?;
    let metadata_bytes = metadata_section
        .encode()
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;

    // Header:
    let source_digest = canonical_source_digest(view.formal)?;
    let header = TeloscHeaderV2 {
        container_schema: TELOSC_CONTAINER_SCHEMA_V2,
        minimum_reader_schema: TELOSC_MIN_READER_SCHEMA_V2,
        package_identity,
        semantic_model_version: SEMANTIC_MODEL_VERSION,
        stable_identity_schema: 1,
        module_semantic_abi: 1,
        semantic_artifact_abi: 1,
        query_fingerprint_abi: 1,
        native_surface_schema_version: NATIVE_SURFACE_SCHEMA_VERSION,
        native_catalog_fingerprint: view.compatibility.native_catalog_fingerprint,
        type_metadata_schema_version: TYPE_METADATA_SCHEMA_VERSION,
        capabilities: phalcom_compiler_artifact::header::TeloscCapabilities::CANONICAL_COMPILER_IMAGE_V1
            | phalcom_compiler_artifact::header::TeloscCapabilities::RUNTIME_PUBLIC_METADATA,
        source_digest,
        interface_digest: ArtifactDigest::digest_domain(DOMAIN_MODULE_WORLD, &module_world_bytes),
        semantic_digest: ArtifactDigest::digest_domain(DOMAIN_SEMANTIC_WORLD, &semantic_world_bytes),
        query_digest: ArtifactDigest::digest_domain(DOMAIN_QUERY_IMAGE, &query_image_bytes),
        metadata_digest: ArtifactDigest::digest_domain(DOMAIN_METADATA, &metadata_bytes),
        artifact_digest: ArtifactDigest::ZERO,
        section_count: 4,
    };

    let mut builder = TeloscContainerBuilder::new(header);
    builder.add_section(
        SectionKind::ModuleWorld,
        MODULE_WORLD_SCHEMA_V1,
        SectionFlags::REQUIRED,
        module_world_bytes.clone(),
    );
    builder.add_section(
        SectionKind::SemanticWorld,
        SEMANTIC_WORLD_SCHEMA_V3,
        SectionFlags::REQUIRED,
        semantic_world_bytes.clone(),
    );
    builder.add_section(
        SectionKind::QueryImage,
        QUERY_IMAGE_SCHEMA_V1,
        SectionFlags::REQUIRED,
        query_image_bytes.clone(),
    );
    builder.add_section(
        SectionKind::Metadata,
        METADATA_SECTION_SCHEMA_V1,
        SectionFlags::REQUIRED,
        metadata_bytes.clone(),
    );

    let built_bytes = builder
        .build()
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;

    // Self-validate output bytes
    let container_view = TeloscContainerView::parse(&built_bytes, limits)
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;

    let summary = TeloscBuildSummary {
        capabilities,
        module_world_bytes: module_world_bytes.len(),
        semantic_world_bytes: semantic_world_bytes.len(),
        query_image_bytes: query_image_bytes.len(),
        metadata_bytes: metadata_bytes.len(),
        artifact_digest: *container_view.header().artifact_digest.as_bytes(),
    };

    Ok(BuiltUniverseTelosc { bytes: built_bytes, summary })
}

/// Atomically writes `universe.telosc` via a same-directory temporary file (`PERF003.C2.P7.T11`).
pub fn write_universe_telosc(path: &Path, built: &BuiltUniverseTelosc) -> Result<(), TeloscArtifactError> {
    if built.summary.capabilities.portable_executable {
        return Err(TeloscArtifactError::Capability(
            "P7 compiler-only artifact must not declare PORTABLE_EXECUTABLE",
        ));
    }
    // The public build product is mutable: validate the actual bytes, including
    // every owner section, before touching the previous destination.
    load_universe_telosc_from_bytes(&built.bytes, &ValidationLimits::default())?;
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(&built.bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}

/// Loads a compiled `universe.telosc` artifact from disk (`PERF003.C2.P7.T12`).
pub fn load_universe_telosc(path: &Path, limits: &ValidationLimits) -> Result<Arc<LoadedUniverseArtifact>, TeloscArtifactError> {
    let t0 = Instant::now();
    let transaction_limit = usize::try_from(limits.max_total_allocation_bytes).map_err(|_| TeloscArtifactError::BudgetExceeded { limit_bytes: usize::MAX })?;
    let mut allocations = AllocationBudget::new(transaction_limit);
    let mut file = File::open(path)?;
    let file_len = file.metadata()?.len();
    if file_len > limits.max_artifact_bytes {
        return Err(TeloscArtifactError::BudgetExceeded {
            limit_bytes: limits.max_artifact_bytes as usize,
        });
    }
    let capacity = usize::try_from(file_len).map_err(|_| TeloscArtifactError::BudgetExceeded { limit_bytes: usize::MAX })?;
    let mut bytes = allocations.vector::<u8>(capacity).map_err(|error| allocation_error(error, transaction_limit))?;
    let mut chunk = [0u8; 8192];
    loop {
        let read = file.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        let next = bytes
            .len()
            .checked_add(read)
            .filter(|len| *len as u64 <= limits.max_artifact_bytes)
            .ok_or(TeloscArtifactError::BudgetExceeded {
                limit_bytes: limits.max_artifact_bytes as usize,
            })?;
        let additional = next - bytes.len();
        allocations
            .grow_vector_exact(&mut bytes, additional)
            .map_err(|error| allocation_error(error, transaction_limit))?;
        bytes.extend_from_slice(&chunk[..read]);
    }
    let mut timings = TeloscLoadTimings {
        file_read: t0.elapsed(),
        ..Default::default()
    };
    load_universe_telosc_from_bytes_with_timings_and_budget(&bytes, limits, &mut timings, &mut allocations)
}

/// Loads a compiled `universe.telosc` artifact from an in-memory buffer (`PERF003.C2.P7.T12`).
pub fn load_universe_telosc_from_bytes(bytes: &[u8], limits: &ValidationLimits) -> Result<Arc<LoadedUniverseArtifact>, TeloscArtifactError> {
    let mut timings = TeloscLoadTimings::default();
    let transaction_limit = usize::try_from(limits.max_total_allocation_bytes).map_err(|_| TeloscArtifactError::BudgetExceeded { limit_bytes: usize::MAX })?;
    let mut allocations = AllocationBudget::new(transaction_limit);
    load_universe_telosc_from_bytes_with_timings_and_budget(bytes, limits, &mut timings, &mut allocations)
}

/// Test-only/internal helper reporting load outcome and cumulative requested budget bytes.
#[doc(hidden)]
pub fn load_universe_telosc_from_bytes_with_allocation_summary(
    bytes: &[u8],
    limits: &ValidationLimits,
) -> (Result<Arc<LoadedUniverseArtifact>, TeloscArtifactError>, usize) {
    let mut timings = TeloscLoadTimings::default();
    let transaction_limit = usize::try_from(limits.max_total_allocation_bytes).unwrap_or(usize::MAX);
    let mut allocations = AllocationBudget::new(transaction_limit);
    let result = load_universe_telosc_from_bytes_with_timings_and_budget(bytes, limits, &mut timings, &mut allocations);
    let requested = allocations.requested();
    (result, requested)
}

fn load_universe_telosc_from_bytes_with_timings_and_budget(
    bytes: &[u8],
    limits: &ValidationLimits,
    timings: &mut TeloscLoadTimings,
    allocations: &mut AllocationBudget,
) -> Result<Arc<LoadedUniverseArtifact>, TeloscArtifactError> {
    let t_container = Instant::now();
    let container = TeloscContainerView::parse_with_allocation_budget(bytes, limits, allocations)
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;
    validate_reader_header(container.header())?;
    timings.container_validation = t_container.elapsed();

    let t_section = Instant::now();
    let module_world_sec = container
        .require_section(SectionKind::ModuleWorld)
        .map_err(|_| TeloscArtifactError::Capability("missing MODULE_WORLD section"))?;
    let semantic_world_sec = container
        .require_section(SectionKind::SemanticWorld)
        .map_err(|_| TeloscArtifactError::Capability("missing SEMANTIC_WORLD section"))?;
    let query_image_sec = container
        .require_section(SectionKind::QueryImage)
        .map_err(|_| TeloscArtifactError::Capability("missing QUERY_IMAGE section"))?;
    let metadata_sec = container
        .require_section(SectionKind::Metadata)
        .map_err(|_| TeloscArtifactError::Capability("missing METADATA section"))?;
    validate_section_provenance(container.header(), module_world_sec, semantic_world_sec, query_image_sec, metadata_sec)?;

    let module_world_v1 = phalcom_compiler_artifact::module_world::ModuleWorldV1::decode_with_allocation_budget(module_world_sec, limits, allocations)
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;
    let modules = phalcom_modules::compiler_artifact::load_module_world_with_allocation_budget(&module_world_v1, limits, allocations)
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;

    let world_v2 = SemanticWorldV3::decode_with_allocation_budget(semantic_world_sec, limits, allocations)
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;
    let parts = validate_and_import_semantic_world_with_allocation_budget(world_v2, limits, allocations)
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;

    let query_v1 = QueryImageV1::decode_with_allocation_budget(query_image_sec, limits, allocations)
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;

    let meta_v1 = MetadataSectionV1::decode_with_allocation_budget(metadata_sec, limits, allocations)
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;
    timings.section_validation = t_section.elapsed();

    let t_reloc = Instant::now();
    let compatibility = phalcom_compiler_artifact::CanonicalCompatibilityIdentity {
        package: container.header().package_identity.clone(),
        source_digest: container.header().source_digest,
        interface_digest: container.header().interface_digest,
        semantic_digest: container.header().semantic_digest,
        semantic_model_version: container.header().semantic_model_version,
        stable_identity_schema: container.header().stable_identity_schema,
        module_semantic_abi: container.header().module_semantic_abi,
        semantic_artifact_abi: container.header().semantic_artifact_abi,
        native_surface_schema_version: container.header().native_surface_schema_version,
        native_catalog_fingerprint: container.header().native_catalog_fingerprint,
        type_metadata_schema_version: container.header().type_metadata_schema_version,
    };
    let world_id = CanonicalSemanticWorldId::for_artifact_world(1, container.header().source_digest, compatibility);

    let (query_image, fingerprints) = reconstruct_query_image_with_allocation_budget(&query_v1, &parts.snapshot, &world_id, &modules, limits, allocations)?;

    let preflight_limits = MetadataPreflightLimits {
        max_metadata_bytes: limits.max_metadata_bytes as usize,
        max_token_count: u32::try_from(limits.max_semantic_entries_per_family).unwrap_or(u32::MAX),
        max_container_count: u32::try_from(limits.max_semantic_entries_per_family).unwrap_or(u32::MAX),
        max_nesting_depth: u16::try_from(limits.max_recursion_depth).unwrap_or(u16::MAX),
    };
    let bundle_limits = phalcom_type_meta::ValidationLimits::default();
    let expected_package_digest = *container.header().package_identity.digest().as_bytes();
    let metadata_bundle = meta_v1
        .decode_bundle_with_allocation_budget(&preflight_limits, &bundle_limits, &expected_package_digest, allocations)
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;

    let transaction_limit = usize::try_from(limits.max_total_allocation_bytes).unwrap_or(usize::MAX);
    let modules = allocations.arc(modules).map_err(|error| allocation_error(error, transaction_limit))?;
    let formal_image = CanonicalUniverseFormalImage::from_loaded_artifact(world_id, modules, parts.snapshot, query_image, fingerprints)?;

    timings.relocation = t_reloc.elapsed();

    let metadata_bundle = allocations.arc(metadata_bundle).map_err(|error| allocation_error(error, transaction_limit))?;
    finish_loaded_universe_telosc(
        TeloscCapabilities::compiler_only_v1(),
        &meta_v1,
        &preflight_limits,
        &expected_package_digest,
        allocations,
        formal_image,
        metadata_bundle,
        *timings,
    )
}

/// Validates metadata section preflight and constructs `Arc<LoadedUniverseArtifact>` (`PERF003.C2.P7.T10/T12`).
pub fn finish_loaded_universe_telosc(
    capabilities: TeloscCapabilities,
    _metadata_section: &MetadataSectionV1,
    _metadata_limits: &MetadataPreflightLimits,
    _expected_package_digest: &[u8; 32],
    allocation_budget: &mut AllocationBudget,
    formal_image: Arc<CanonicalUniverseFormalImage>,
    metadata: Arc<SemanticMetadataBundle>,
    mut timings: TeloscLoadTimings,
) -> Result<Arc<LoadedUniverseArtifact>, TeloscArtifactError> {
    if !capabilities.module_world || !capabilities.semantic_world || !capabilities.query_image || !capabilities.runtime_public_metadata {
        return Err(TeloscArtifactError::Capability("missing required compiler-only capability"));
    }
    let t_pub = Instant::now();
    let transaction_limit = allocation_budget.remaining().saturating_add(allocation_budget.requested());
    let artifact = allocation_budget
        .arc(LoadedUniverseArtifact {
            capabilities,
            formal_image,
            metadata,
            timings: {
                timings.publication = t_pub.elapsed();
                timings
            },
            executable_reserved_p8: None,
        })
        .map_err(|error| allocation_error(error, transaction_limit))?;
    Ok(artifact)
}

/// Validate reader compatibility independently of digest integrity.
fn validate_reader_header(header: &TeloscHeaderV2) -> Result<(), TeloscArtifactError> {
    use phalcom_compiler_artifact::ValidationError;
    use phalcom_compiler_artifact::header::TeloscCapabilities as WireCapabilities;
    if header.package_identity != TeloscPackageIdentity::universe() {
        return Err(phalcom_compiler_artifact::ArtifactError::from(ValidationError::IncompatibleWorld("unsupported Universe package identity".into())).into());
    }
    for (kind, found, expected) in [
        ("semantic_model_version", header.semantic_model_version, SEMANTIC_MODEL_VERSION),
        ("stable_identity_schema", header.stable_identity_schema, 1),
        ("module_semantic_abi", header.module_semantic_abi, 1),
        ("semantic_artifact_abi", header.semantic_artifact_abi, 1),
        ("query_fingerprint_abi", header.query_fingerprint_abi, 1),
        (
            "native_surface_schema_version",
            header.native_surface_schema_version,
            NATIVE_SURFACE_SCHEMA_VERSION,
        ),
        (
            "type_metadata_schema_version",
            header.type_metadata_schema_version,
            TYPE_METADATA_SCHEMA_VERSION,
        ),
    ] {
        if found != expected {
            return Err(phalcom_compiler_artifact::ArtifactError::from(ValidationError::IncompatibleAbi { kind, found, expected }).into());
        }
    }
    let expected_catalog = phalcom_native_surface::catalog_fingerprint().0;
    if header.native_catalog_fingerprint != expected_catalog {
        return Err(
            phalcom_compiler_artifact::ArtifactError::from(ValidationError::NativeCatalogFingerprintMismatch {
                found: header.native_catalog_fingerprint,
                expected: expected_catalog,
            })
            .into(),
        );
    }
    let required = WireCapabilities::CANONICAL_COMPILER_IMAGE_V1 | WireCapabilities::RUNTIME_PUBLIC_METADATA;
    if header.capabilities != required {
        return Err(TeloscArtifactError::Capability("unsupported compiler artifact capability set"));
    }
    Ok(())
}

fn validate_section_provenance(
    header: &TeloscHeaderV2,
    module_world: &[u8],
    semantic_world: &[u8],
    query_image: &[u8],
    metadata: &[u8],
) -> Result<(), TeloscArtifactError> {
    // Source text is intentionally absent from the portable sections, so the
    // reader cannot recompute its provenance digest. It still rejects the old
    // zero placeholder; the container digest binds this asserted identity to
    // the exact serialized header.
    if header.source_digest == ArtifactDigest::ZERO {
        return Err(
            phalcom_compiler_artifact::ArtifactError::from(phalcom_compiler_artifact::ValidationError::IncompatibleWorld(
                "source provenance digest is missing".into(),
            ))
            .into(),
        );
    }
    let expected = [
        (
            "interface",
            header.interface_digest,
            ArtifactDigest::digest_domain(DOMAIN_MODULE_WORLD, module_world),
        ),
        (
            "semantic",
            header.semantic_digest,
            ArtifactDigest::digest_domain(DOMAIN_SEMANTIC_WORLD, semantic_world),
        ),
        ("query", header.query_digest, ArtifactDigest::digest_domain(DOMAIN_QUERY_IMAGE, query_image)),
        ("metadata", header.metadata_digest, ArtifactDigest::digest_domain(DOMAIN_METADATA, metadata)),
    ];
    for (name, found, actual) in expected {
        if found != actual {
            return Err(
                phalcom_compiler_artifact::ArtifactError::from(phalcom_compiler_artifact::ValidationError::IncompatibleWorld(format!(
                    "{name} provenance digest mismatch"
                )))
                .into(),
            );
        }
    }
    Ok(())
}

/// Computes source provenance from the exact source-authority map retained by
/// a source-built canonical image. Artifact-loaded images preserve the original
/// digest in their provenance identity and can therefore be republished without
/// access to source text.
fn canonical_source_digest(formal: &CanonicalUniverseFormalImage) -> Result<ArtifactDigest, TeloscArtifactError> {
    match formal.world().provenance() {
        CanonicalWorldProvenance::Source(sources) => {
            let mut hasher = ArtifactHasher::new_domain(DOMAIN_SOURCE);
            for (module, source) in sources.iter() {
                let module = module.to_string();
                hasher.write_u64(module.len() as u64);
                hasher.write(module.as_bytes());
                hasher.write_u64(source.len() as u64);
                hasher.write(source.as_bytes());
            }
            Ok(hasher.finalize())
        }
        CanonicalWorldProvenance::Artifact { source_digest, .. } => Ok(*source_digest),
    }
}
