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
#[cfg(test)]
use std::cell::Cell;

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

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LoaderTestBoundary {
    AfterModuleImport,
    AfterSemanticImport,
    AfterQueryImport,
    AfterMetadataValidation,
}

#[cfg(test)]
impl LoaderTestBoundary {
    const fn error_message(self) -> &'static str {
        match self {
            Self::AfterModuleImport => "test failure after module import",
            Self::AfterSemanticImport => "test failure after semantic import",
            Self::AfterQueryImport => "test failure after query import",
            Self::AfterMetadataValidation => "test failure after metadata validation",
        }
    }
}

#[cfg(test)]
std::thread_local! {
    static LOADER_TEST_FAILURE_BOUNDARY: Cell<Option<LoaderTestBoundary>> = const { Cell::new(None) };
}

#[cfg(test)]
struct LoaderTestFailureGuard;

#[cfg(test)]
impl LoaderTestFailureGuard {
    fn after(boundary: LoaderTestBoundary) -> Self {
        LOADER_TEST_FAILURE_BOUNDARY.with(|slot| {
            assert!(slot.get().is_none(), "loader test failpoints cannot be nested");
            slot.set(Some(boundary));
        });
        Self
    }
}

#[cfg(test)]
impl Drop for LoaderTestFailureGuard {
    fn drop(&mut self) {
        LOADER_TEST_FAILURE_BOUNDARY.with(|slot| slot.set(None));
    }
}

#[cfg(test)]
fn maybe_fail_after(boundary: LoaderTestBoundary) -> Result<(), TeloscArtifactError> {
    let should_fail = LOADER_TEST_FAILURE_BOUNDARY.with(|slot| slot.get() == Some(boundary));
    if should_fail {
        Err(TeloscArtifactError::Capability(boundary.error_message()))
    } else {
        Ok(())
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
    #[cfg(test)]
    maybe_fail_after(LoaderTestBoundary::AfterModuleImport)?;

    let world_v2 = SemanticWorldV3::decode_with_allocation_budget(semantic_world_sec, limits, allocations)
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;
    let parts = validate_and_import_semantic_world_with_allocation_budget(world_v2, limits, allocations)
        .map_err(|e| TeloscArtifactError::CanonicalImage(CanonicalImageError::IncompletePublication(e.to_string())))?;
    #[cfg(test)]
    maybe_fail_after(LoaderTestBoundary::AfterSemanticImport)?;

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
    #[cfg(test)]
    maybe_fail_after(LoaderTestBoundary::AfterQueryImport)?;

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
    #[cfg(test)]
    maybe_fail_after(LoaderTestBoundary::AfterMetadataValidation)?;

    let transaction_limit = usize::try_from(limits.max_total_allocation_bytes).unwrap_or(usize::MAX);
    let modules = allocations.arc(modules).map_err(|error| allocation_error(error, transaction_limit))?;
    let formal_image = CanonicalUniverseFormalImage::from_loaded_artifact(world_id, modules, parts.snapshot, query_image, fingerprints, allocations)?;

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
    if !header.package_identity.is_exact_universe() {
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

#[cfg(test)]
mod temporary_loaded_publication_allocation_census {
    use super::*;
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    struct CountingAllocator;

    static MEASURING: AtomicBool = AtomicBool::new(false);
    static REQUESTED_BYTES: AtomicUsize = AtomicUsize::new(0);

    fn record_request(bytes: usize) {
        if MEASURING.load(Ordering::Relaxed) {
            REQUESTED_BYTES.fetch_add(bytes, Ordering::Relaxed);
        }
    }

    fn observed_requested_bytes() -> usize {
        REQUESTED_BYTES.load(Ordering::Relaxed)
    }

    // SAFETY: The wrapper forwards every original allocator operation unchanged
    // and records only the requested layout size while the census is enabled.
    unsafe impl GlobalAlloc for CountingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            record_request(layout.size());
            unsafe { System.alloc(layout) }
        }

        unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
            unsafe { System.dealloc(pointer, layout) }
        }

        unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            record_request(new_size);
            unsafe { System.realloc(pointer, layout, new_size) }
        }
    }

    #[global_allocator]
    static GLOBAL: CountingAllocator = CountingAllocator;

    fn measure_stage<T>(
        name: &'static str,
        allocations: &mut AllocationBudget,
        operation: impl FnOnce(&mut AllocationBudget) -> T,
    ) -> (T, usize, isize) {
        let ledger_before = allocations.requested();
        REQUESTED_BYTES.store(0, Ordering::Relaxed);
        MEASURING.store(true, Ordering::Relaxed);
        let output = operation(allocations);
        MEASURING.store(false, Ordering::Relaxed);
        let observed = observed_requested_bytes();
        let ledger_delta = allocations.requested() - ledger_before;
        let signed_delta = observed as isize - ledger_delta as isize;
        eprintln!(
            "CENSUS {name}: ledger_before={ledger_before} ledger_after={} ledger_delta={ledger_delta} allocator_before=0 allocator_after={observed} allocator_delta={observed} signed_delta={signed_delta} output_retained=yes",
            allocations.requested()
        );
        (output, observed, signed_delta)
    }

    fn print_query_row(name: &str, row: phalcom_semantic::artifact::query::QueryReconstructionCensusRow) -> isize {
        let signed = row.observed_bytes as isize - row.ledger_bytes as isize;
        eprintln!(
            "  CENSUS {name}: ledger_before={} ledger_after={} ledger_delta={} allocator_before={} allocator_after={} allocator_delta={} signed_delta={signed}",
            row.ledger_before,
            row.ledger_after,
            row.ledger_bytes,
            row.allocator_before,
            row.allocator_after,
            row.observed_bytes,
        );
        signed
    }

    #[test]
    fn temporary_current_loaded_publication_allocation_census() {
        let limits = ValidationLimits::default();
        let artifact_bytes = if let Some(path) = std::env::var_os("PHALCOM_UNIVERSE_TELOSC_FOR_TEST") {
            std::fs::read(path).expect("read prebuilt canonical Universe artifact")
        } else {
            let program = crate::modules::canonical_universe::canonical_universe_program()
                .expect("canonical Universe program must compile");
            let view = program.compiler_image_view().expect("compiler image view must be present");
            build_universe_telosc(&view, &limits)
                .expect("build canonical Universe artifact")
                .bytes
        };
        assert!(!artifact_bytes.is_empty());
        let budget_limit = usize::try_from(limits.max_total_allocation_bytes).unwrap_or(usize::MAX);

        let mut allocations = AllocationBudget::new(budget_limit);
        let mut independent_total = 0usize;
        let mut signed_stage_total = 0isize;
        let mut container = None;

        let (container_result, observed, signed) = measure_stage("A container/root + header/provenance", &mut allocations, |budget| {
            let parsed = TeloscContainerView::parse_with_allocation_budget(&artifact_bytes, &limits, budget)?;
            validate_reader_header(parsed.header())?;
            let module = parsed.require_section(SectionKind::ModuleWorld).map_err(|_| TeloscArtifactError::Capability("missing MODULE_WORLD section"))?;
            let semantic = parsed.require_section(SectionKind::SemanticWorld).map_err(|_| TeloscArtifactError::Capability("missing SEMANTIC_WORLD section"))?;
            let query = parsed.require_section(SectionKind::QueryImage).map_err(|_| TeloscArtifactError::Capability("missing QUERY_IMAGE section"))?;
            let metadata = parsed.require_section(SectionKind::Metadata).map_err(|_| TeloscArtifactError::Capability("missing METADATA section"))?;
            validate_section_provenance(parsed.header(), module, semantic, query, metadata)?;
            container = Some(parsed);
            Ok::<_, TeloscArtifactError>(())
        });
        independent_total += observed;
        signed_stage_total += signed;
        container_result.expect("container parse and provenance validation must succeed");
        let parsed = container.as_ref().expect("container retained for census");
        let module_bytes = parsed.require_section(SectionKind::ModuleWorld).expect("MODULE_WORLD section");
        let semantic_bytes = parsed.require_section(SectionKind::SemanticWorld).expect("SEMANTIC_WORLD section");
        let query_bytes = parsed.require_section(SectionKind::QueryImage).expect("QUERY_IMAGE section");
        let metadata_bytes = parsed.require_section(SectionKind::Metadata).expect("METADATA section");

        let (module_dto_result, observed, signed) = measure_stage("B MODULE_WORLD DTO decode", &mut allocations, |budget| {
            phalcom_compiler_artifact::module_world::ModuleWorldV1::decode_with_allocation_budget(module_bytes, &limits, budget)
        });
        independent_total += observed;
        signed_stage_total += signed;
        let module_dto = module_dto_result.expect("MODULE_WORLD DTO decode must succeed");

        let (module_result, observed, signed) = measure_stage("C module native materialization", &mut allocations, |budget| {
            phalcom_modules::compiler_artifact::load_module_world_with_allocation_budget(&module_dto, &limits, budget)
        });
        independent_total += observed;
        signed_stage_total += signed;
        let modules = module_result.expect("MODULE_WORLD materialization must succeed");

        let (semantic_dto_result, observed, signed) = measure_stage("D SEMANTIC_WORLD DTO decode", &mut allocations, |budget| {
            SemanticWorldV3::decode_with_allocation_budget(semantic_bytes, &limits, budget)
        });
        independent_total += observed;
        signed_stage_total += signed;
        let semantic_dto = semantic_dto_result.expect("SEMANTIC_WORLD DTO decode must succeed");

        let (semantic_result, observed, signed) = measure_stage("E semantic native import", &mut allocations, |budget| {
            validate_and_import_semantic_world_with_allocation_budget(semantic_dto, &limits, budget)
        });
        independent_total += observed;
        signed_stage_total += signed;
        let parts = semantic_result.expect("SEMANTIC_WORLD import must succeed");

        let (query_dto_result, observed, signed) = measure_stage("F QUERY_IMAGE DTO decode", &mut allocations, |budget| {
            QueryImageV1::decode_with_allocation_budget(query_bytes, &limits, budget)
        });
        independent_total += observed;
        signed_stage_total += signed;
        let query_dto = query_dto_result.expect("QUERY_IMAGE DTO decode must succeed");

        let (metadata_dto_result, observed, signed) = measure_stage("G METADATA DTO decode", &mut allocations, |budget| {
            MetadataSectionV1::decode_with_allocation_budget(metadata_bytes, &limits, budget)
        });
        independent_total += observed;
        signed_stage_total += signed;
        let metadata_dto = metadata_dto_result.expect("METADATA DTO decode must succeed");

        phalcom_type_meta::allocation::register_allocation_probe(observed_requested_bytes);
        phalcom_semantic::artifact::query::reset_query_reconstruction_census();
        let (world_result, observed, signed) = measure_stage("H0 query semantic-world identity setup", &mut allocations, |_budget| {
            let header = parsed.header();
            let compatibility = phalcom_compiler_artifact::CanonicalCompatibilityIdentity {
                package: header.package_identity.clone(),
                source_digest: header.source_digest,
                interface_digest: header.interface_digest,
                semantic_digest: header.semantic_digest,
                semantic_model_version: header.semantic_model_version,
                stable_identity_schema: header.stable_identity_schema,
                module_semantic_abi: header.module_semantic_abi,
                semantic_artifact_abi: header.semantic_artifact_abi,
                native_surface_schema_version: header.native_surface_schema_version,
                native_catalog_fingerprint: header.native_catalog_fingerprint,
                type_metadata_schema_version: header.type_metadata_schema_version,
            };
            Ok::<_, TeloscArtifactError>(CanonicalSemanticWorldId::for_artifact_world(1, header.source_digest, compatibility))
        });
        independent_total += observed;
        signed_stage_total += signed;
        let world = world_result.expect("artifact semantic world identity must be constructed");
        let query_ledger_before = allocations.requested();
        let (query_result, observed, signed) = measure_stage("H-K query reconstruction", &mut allocations, |budget| {
            let reconstructed = reconstruct_query_image_with_allocation_budget(
                &query_dto,
                &parts.snapshot,
                &world,
                &modules,
                &limits,
                budget,
            );
            reconstructed
        });
        independent_total += observed;
        signed_stage_total += signed;
        let (query_image, fingerprints) = query_result.expect("query reconstruction must succeed");
        let query_ledger_delta = allocations.requested() - query_ledger_before;
        let query_census = phalcom_semantic::artifact::query::take_query_reconstruction_census();
        signed_stage_total -= signed;
        let mut query_rows_observed = 0usize;
        let mut query_rows_ledger = 0usize;
        for (name, row) in [
            ("Q0 closure/root validation", query_census.closure_and_roots),
            ("Q1 owner decode + QueryKey reconstruction", query_census.owner_key_resolution),
            ("Q2 product_and_fingerprints", query_census.product_reconstruction),
            ("Q3 dependency/entry/fingerprint assembly", query_census.entry_assembly),
            ("Q4 CanonicalSemanticQueryImage loaded validation", query_census.loaded_validation),
            ("Q5 query image Arc publication", query_census.query_image_publication),
        ] {
            query_rows_observed += row.observed_bytes;
            query_rows_ledger += row.ledger_bytes;
            signed_stage_total += print_query_row(name, row);
        }
        for row in &query_census.product_by_kind {
            eprintln!(
                "  CENSUS Q2 {:?}: ledger={} observed={} signed_delta={}",
                row.kind,
                row.ledger_bytes,
                row.observed_bytes,
                row.observed_bytes as isize - row.ledger_bytes as isize
            );
        }
        let query_x_observed = observed - query_rows_observed;
        let query_x_ledger = query_ledger_delta - query_rows_ledger;
        let query_x_signed = query_x_observed as isize - query_x_ledger as isize;
        eprintln!("  CENSUS QX boundary/unclassified: ledger_delta={query_x_ledger} allocator_delta={query_x_observed} signed_delta={query_x_signed}");
        assert_eq!(0, query_x_observed, "query census allocator boundary must be zero");
        assert_eq!(0, query_x_ledger, "query census ledger boundary must be zero");
        assert_eq!(0, query_x_signed, "query census signed boundary must be zero");

        let preflight_limits = MetadataPreflightLimits {
            max_metadata_bytes: limits.max_metadata_bytes as usize,
            max_token_count: u32::try_from(limits.max_semantic_entries_per_family).unwrap_or(u32::MAX),
            max_container_count: u32::try_from(limits.max_semantic_entries_per_family).unwrap_or(u32::MAX),
            max_nesting_depth: u16::try_from(limits.max_recursion_depth).unwrap_or(u16::MAX),
        };
        let bundle_limits = phalcom_type_meta::ValidationLimits::default();
        let expected_package_digest = *parsed.header().package_identity.digest().as_bytes();
        let (bundle_result, observed, signed) = measure_stage("M metadata bundle decode/materialization", &mut allocations, |budget| {
            metadata_dto.decode_bundle_with_allocation_budget(&preflight_limits, &bundle_limits, &expected_package_digest, budget)
        });
        independent_total += observed;
        signed_stage_total += signed;
        let metadata_bundle = bundle_result.expect("metadata bundle decode must succeed");

        phalcom_semantic::canonical_image::reset_loaded_publication_census();
        let mut module_arc_observed = 0usize;
        let formal_ledger_before = allocations.requested();
        let (formal_result, observed, signed) = measure_stage("N-P formal image/module-query publication", &mut allocations, |budget| {
            let before_arc = observed_requested_bytes();
            let modules = budget.arc(modules).map_err(|error| allocation_error(error, budget_limit))?;
            module_arc_observed = observed_requested_bytes().saturating_sub(before_arc);
            CanonicalUniverseFormalImage::from_loaded_artifact(
                world,
                modules,
                parts.snapshot,
                query_image,
                fingerprints,
                budget,
            )
            .map_err(TeloscArtifactError::from)
        });
        independent_total += observed;
        signed_stage_total += signed;
        let formal_image = formal_result.expect("formal image publication must succeed");
        let formal_ledger_delta = allocations.requested() - formal_ledger_before;
        let publication = phalcom_semantic::canonical_image::take_loaded_publication_census();
        let publication_rows = [
            ("P1 snapshot clone", publication.snapshot_clone, publication.snapshot_clone_ledger),
            ("P2 status/content promotion", publication.status_promotion, publication.status_promotion_ledger),
            ("P3 ModuleQueryProducts reconstruction", publication.module_query_products, publication.module_query_products_ledger),
            (
                "P4 ModuleQueryProducts Arc/publication",
                publication.module_query_products_publication,
                publication.module_query_products_publication_ledger,
            ),
            ("P5 source-free completeness", publication.source_free_completeness, publication.source_free_completeness_ledger),
            ("P6 advisory validation", publication.advisory_validation, publication.advisory_validation_ledger),
            ("P7 query reader validation", publication.query_reader_validation, publication.query_reader_validation_ledger),
            ("P8 fingerprint validation", publication.fingerprint_validation, publication.fingerprint_validation_ledger),
            ("P9 final Arc<SemanticSnapshot>", publication.snapshot_arc, publication.snapshot_arc_ledger),
            ("P10 Arc<CanonicalUniverseFormalImage>", publication.formal_image_arc, publication.formal_image_arc_ledger),
        ];
        let publication_observed = publication_rows.iter().map(|(_, observed, _)| *observed).sum::<usize>();
        let publication_ledger = publication_rows.iter().map(|(_, _, ledger)| *ledger).sum::<usize>();
        assert_eq!(publication_observed + module_arc_observed, observed, "formal publication sub-census must match its outer stage");
        assert_eq!(publication_ledger + 64, formal_ledger_delta, "formal publication sub-ledger must match its outer stage");
        let mut module_query_signed = 0isize;
        for (name, observed, ledger) in publication_rows {
            let signed = observed as isize - ledger as isize;
            eprintln!("  CENSUS {name}: ledger={ledger} observed={observed} signed_delta={signed}");
            if name.starts_with("P3 ") || name.starts_with("P4 ") {
                module_query_signed += signed;
            }
        }
        eprintln!(
            "  CENSUS formal image module Arc: ledger=64 observed={module_arc_observed} signed_delta={}",
            module_arc_observed as isize - 64
        );
        let formal_module_query_signed = module_query_signed;
        let formal_remaining_signed = signed - formal_module_query_signed;
        eprintln!("  CENSUS M ModuleQueryProducts reconstruction/publication: signed_delta={formal_module_query_signed}");
        eprintln!("  CENSUS N remaining formal publication: signed_delta={formal_remaining_signed}");
        signed_stage_total -= signed;
        signed_stage_total += formal_module_query_signed + formal_remaining_signed;

        let (loaded_result, observed, signed) = measure_stage("O final LoadedUniverseArtifact publication", &mut allocations, |budget| {
            let metadata = budget.arc(metadata_bundle).map_err(|error| allocation_error(error, budget_limit))?;
            finish_loaded_universe_telosc(
                TeloscCapabilities::compiler_only_v1(),
                &metadata_dto,
                &preflight_limits,
                &expected_package_digest,
                budget,
                formal_image,
                metadata,
                TeloscLoadTimings::default(),
            )
        });
        independent_total += observed;
        signed_stage_total += signed;
        let loaded = loaded_result.expect("final artifact publication must succeed");

        let stage_ledger_total = allocations.requested();
        eprintln!(
            "CURRENT STAGE CENSUS TOTAL: ledger={stage_ledger_total} observed={independent_total} signed_delta={}",
            independent_total as isize - stage_ledger_total as isize
        );
        drop(loaded);

        REQUESTED_BYTES.store(0, Ordering::Relaxed);
        MEASURING.store(true, Ordering::Relaxed);
        let (production_result, production_ledger) = load_universe_telosc_from_bytes_with_allocation_summary(&artifact_bytes, &limits);
        MEASURING.store(false, Ordering::Relaxed);
        let production_observed = observed_requested_bytes();
        drop(production_result.expect("production whole load must succeed"));
        let whole_signed = production_observed as isize - production_ledger as isize;
        let boundary_signed = whole_signed - signed_stage_total;
        eprintln!(
            "WHOLE LOAD: ledger={production_ledger} observed={production_observed} signed_delta={whole_signed}\nSTAGE SIGNED DELTA SUM={signed_stage_total}\nO UNCLASSIFIED/BOUNDARY={boundary_signed}\nRECONCILED={}",
            signed_stage_total + boundary_signed
        );
        assert_eq!(stage_ledger_total, production_ledger, "manual stage ledger must equal production ledger");
        assert_eq!(signed_stage_total + boundary_signed, whole_signed, "stage deltas plus boundary must exactly reconcile");
    }

    #[test]
    fn loaded_artifact_prepublication_failures_are_retryable() {
        use crate::modules::canonical_universe::canonical_universe_program;

        let limits = ValidationLimits::default();
        let program = canonical_universe_program().expect("canonical universe program must build");
        let view = program.compiler_image_view().expect("canonical compiler image view must exist");
        let artifact = build_universe_telosc(&view, &limits).expect("canonical artifact must build");

        let control = load_universe_telosc_from_bytes(&artifact.bytes, &limits).expect("control artifact load must succeed");
        let control_world = control.formal_image.world().clone();
        let control_query_identity = control
            .formal_image
            .query_image()
            .entries()
            .map(|(key, entry)| (key.clone(), entry.product_fingerprint))
            .collect::<Vec<_>>();
        let control_dependency_identity = control
            .formal_image
            .dependency_fingerprints()
            .iter()
            .map(|(key, fingerprint)| (key.clone(), *fingerprint))
            .collect::<Vec<_>>();
        assert!(control.formal_image.sources().is_empty());
        assert!(control.formal_image.snapshot().is_source_free_artifact_complete());
        assert!(control.formal_image.reader().is_ok());

        for boundary in [
            LoaderTestBoundary::AfterModuleImport,
            LoaderTestBoundary::AfterSemanticImport,
            LoaderTestBoundary::AfterQueryImport,
            LoaderTestBoundary::AfterMetadataValidation,
        ] {
            let failure_guard = LoaderTestFailureGuard::after(boundary);
            let failed = load_universe_telosc_from_bytes(&artifact.bytes, &limits);
            drop(failure_guard);
            match failed {
                Err(TeloscArtifactError::Capability(message)) => assert_eq!(message, boundary.error_message()),
                Err(error) => panic!("injected {boundary:?} failure returned the wrong error: {error}"),
                Ok(_) => panic!("injected {boundary:?} failure returned a LoadedUniverseArtifact"),
            }

            let retry = load_universe_telosc_from_bytes(&artifact.bytes, &limits)
                .unwrap_or_else(|error| panic!("clean retry after {boundary:?} must succeed: {error}"));
            assert_eq!(retry.formal_image.world(), &control_world, "retry must retain the artifact's canonical world identity");
            assert_eq!(
                retry
                    .formal_image
                    .query_image()
                    .entries()
                    .map(|(key, entry)| (key.clone(), entry.product_fingerprint))
                    .collect::<Vec<_>>(),
                control_query_identity,
                "retry must reconstruct the same semantic query identities"
            );
            assert_eq!(
                retry
                    .formal_image
                    .dependency_fingerprints()
                    .iter()
                    .map(|(key, fingerprint)| (key.clone(), *fingerprint))
                    .collect::<Vec<_>>(),
                control_dependency_identity,
                "retry must reconstruct the same dependency fingerprints"
            );
            assert!(retry.formal_image.sources().is_empty());
            assert!(retry.formal_image.snapshot().is_source_free_artifact_complete());
            assert!(retry.formal_image.reader().is_ok());
        }
    }
}
