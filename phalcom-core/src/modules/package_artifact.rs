//! Package-scoped compiler artifact projection and `.telosc` writing.

use super::canonical_artifact::{TeloscArtifactError, TeloscBuildSummary, TeloscCapabilities};
use super::compile::{AnalyzedProgram, ProgramCompilationOptions, ProgramCompiler};
use super::package_image::PackageCompilerImage;
use phalcom_compiler_artifact::container::{TeloscContainerBuilder, TeloscContainerView};
use phalcom_compiler_artifact::digest::{ArtifactDigest, DOMAIN_METADATA, DOMAIN_MODULE_WORLD, DOMAIN_QUERY_IMAGE, DOMAIN_SEMANTIC_WORLD};
use phalcom_compiler_artifact::header::{TELOSC_CONTAINER_SCHEMA_V2, TELOSC_MIN_READER_SCHEMA_V2};
use phalcom_compiler_artifact::identity::{PackageIdentityAbi, TeloscPackageIdentity, package_identity_fingerprint};
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::metadata::{METADATA_SECTION_SCHEMA_V1, MetadataPreflightLimits, MetadataSectionV1};
use phalcom_compiler_artifact::module_world_v5::{MODULE_WORLD_SCHEMA_V5, ModuleWorldV5};
use phalcom_compiler_artifact::query::QUERY_IMAGE_SCHEMA_V4;
use phalcom_compiler_artifact::section::{SectionFlags, SectionKind};
use phalcom_compiler_artifact::semantic::world::SEMANTIC_WORLD_SCHEMA_V4;
use phalcom_modules::compiler_artifact::project_package_module_world_v5;
use phalcom_semantic::artifact::export::{export_query_image_v4_scoped, export_semantic_world_v4_scoped};
use phalcom_semantic::artifact::projection::ArtifactSemanticProjection;
use phalcom_semantic::metadata::stable_identity::{StableIdentityContext, to_stable_module_with_context};
use phalcom_semantic::static_value::StaticExportContext;
use phalcom_semantic::types::store::artifact::TypeExportContext;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::StableProjectRef;
use std::collections::BTreeSet;

/// A compiler-only `.telosc` artifact published for one package identity.
#[derive(Clone, Debug)]
pub struct BuiltPackageTelosc {
    /// Encoded artifact bytes.
    pub bytes: Vec<u8>,
    /// Encoded section sizes and final artifact digest.
    pub summary: TeloscBuildSummary,
    /// Stable package identity embedded in the artifact header.
    pub identity: StableProjectRef,
}

/// Builds a deterministic package compiler artifact from a captured image.
///
/// The builder projects the module, semantic, query, and metadata sections
/// with one publishing identity context. If projection discovers additional
/// external owners, it retries once with the expanded identity input set.
pub fn build_package_telosc(image: &PackageCompilerImage, limits: &ValidationLimits) -> Result<BuiltPackageTelosc, TeloscArtifactError> {
    let (package, version) = match &image.identity {
        StableProjectRef::Package { package, version, .. } => (package.as_ref(), version.as_ref()),
        _ => return Err(incomplete("package compiler image has a non-package publishing identity")),
    };
    let mut identity_inputs = image.identity_external_projects.iter().cloned().collect::<BTreeSet<_>>();

    for pass in 0..2 {
        let fingerprint = package_identity_fingerprint(
            package,
            version,
            image.source_digest,
            &identity_inputs.iter().cloned().collect::<Vec<_>>(),
            &PackageIdentityAbi::from_header(image.base_header()),
        )?;
        let identity = StableProjectRef::Package {
            package: package.into(),
            version: version.into(),
            artifact_fingerprint: fingerprint,
        };
        let identity_context = StableIdentityContext::publishing(image.projects.as_ref(), image.project, &identity);
        let projection = project_package_sections(image, &identity_context, &identity_inputs, limits)?;
        let actual_external = collect_external_projects(
            &identity,
            &projection.module_world,
            &projection.semantic_project_refs,
            &projection.query_image,
            &projection.metadata,
        )?;
        let new_inputs = actual_external.difference(&identity_inputs).cloned().collect::<Vec<_>>();
        if new_inputs.is_empty() {
            return build_container(image, identity, projection, limits);
        }
        if pass == 1 {
            return Err(incomplete("package projection external-owner closure changed after the bounded identity retry"));
        }
        identity_inputs.extend(new_inputs);
    }

    Err(incomplete("package identity projection did not converge"))
}

struct PackageProjection {
    module_world: ModuleWorldV5,
    module_world_bytes: Vec<u8>,
    semantic_world_bytes: Vec<u8>,
    semantic_project_refs: BTreeSet<StableProjectRef>,
    query_image: phalcom_compiler_artifact::query::QueryImageV4,
    query_image_bytes: Vec<u8>,
    metadata: std::sync::Arc<phalcom_type_meta::SemanticMetadataBundle>,
    metadata_bytes: Vec<u8>,
}

fn project_package_sections(
    image: &PackageCompilerImage,
    identity_context: &StableIdentityContext<'_>,
    identity_inputs: &BTreeSet<StableProjectRef>,
    limits: &ValidationLimits,
) -> Result<PackageProjection, TeloscArtifactError> {
    let stable_module = |module: &phalcom_modules::ModuleId| to_stable_module_with_context(module, identity_context);
    let mut module_world = project_package_module_world_v5(
        &image.modules,
        &image.linked,
        image.external.as_ref(),
        &stable_module,
        image.availability(),
        limits,
    )
    .map_err(|error| incomplete(error.to_string()))?;
    module_world.external_projects.extend(identity_inputs.iter().cloned());
    module_world.external_projects.sort_unstable();
    module_world.external_projects.dedup();
    module_world.refresh_portable_topology_fingerprint()?;
    module_world.validate(limits)?;
    let module_world_bytes = module_world.encode()?;

    let projection = ArtifactSemanticProjection::new(&image.snapshot, &image.scope, *identity_context);
    let mut types = TypeExportContext::new(&image.snapshot.store, *identity_context);
    let mut statics = StaticExportContext::new(image.snapshot.static_values.as_ref());
    phalcom_semantic::artifact::roots::register_semantic_world_roots_scoped(projection, &mut types, &mut statics)?;
    statics.register_type_roots(&image.snapshot.static_values, &mut types, limits)?;
    types.close_graph(&image.snapshot.store, limits)?;
    statics
        .close_graph(image.snapshot.static_values.as_ref(), &types, limits)
        .map_err(|error| incomplete(error.to_string()))?;
    let semantic_world = export_semantic_world_v4_scoped(projection, &types, &statics, &module_world, image.linked.runtime_requirements(), limits)
        .map_err(|error| incomplete(error.to_string()))?;
    let (semantic_world_bytes, semantic_project_refs) = semantic_world.encode_with_project_inventory()?;

    let query_image = export_query_image_v4_scoped(
        &image.scope,
        &image.queries,
        &image.dependency_fingerprints,
        &module_world,
        &semantic_world,
        types.identity_context(),
        limits,
    )
    .map_err(|error| incomplete(error.to_string()))?;
    let query_image_bytes = query_image.encode()?;

    let analyzed = AnalyzedProgram {
        compile_mode: crate::compiler::attributes::CompileMode::Debug,
        project_universe: image.projects.clone(),
        linked: image.linked.clone(),
        semantic: image.snapshot.clone(),
        sources: image.sources.clone(),
        runtime_modules: image
            .scope
            .owned_modules()
            .ok_or_else(|| incomplete("package image has no explicit module scope"))?
            .clone(),
        entry: image.entry.clone(),
    };
    let source_fingerprint = Fingerprint128(image.source_digest.as_bytes()[..16].try_into().expect("digest prefix is 16 bytes"));
    let compiled = ProgramCompiler::project_analyzed_for_package_image(
        &analyzed,
        &ProgramCompilationOptions::default(),
        &image.scope,
        *identity_context,
        source_fingerprint,
    )
    .map_err(|error| incomplete(error.to_string()))?;
    let metadata = compiled
        .semantic_metadata
        .ok_or_else(|| incomplete("package metadata projection returned no bundle"))?;
    let preflight_limits = MetadataPreflightLimits {
        max_metadata_bytes: usize::try_from(limits.max_metadata_bytes).unwrap_or(usize::MAX),
        max_token_count: u32::try_from(limits.max_semantic_entries_per_family).unwrap_or(u32::MAX),
        max_container_count: u32::try_from(limits.max_semantic_entries_per_family).unwrap_or(u32::MAX),
        max_nesting_depth: u16::try_from(limits.max_recursion_depth).unwrap_or(u16::MAX),
    };
    let metadata_section = MetadataSectionV1::from_bundle(
        &metadata,
        package_identity_digest(image)?,
        &preflight_limits,
        &phalcom_type_meta::ValidationLimits::default(),
    )?;
    let metadata_bytes = metadata_section.encode()?;

    Ok(PackageProjection {
        module_world,
        module_world_bytes,
        semantic_world_bytes,
        semantic_project_refs,
        query_image,
        query_image_bytes,
        metadata,
        metadata_bytes,
    })
}

fn build_container(
    image: &PackageCompilerImage,
    identity: StableProjectRef,
    projection: PackageProjection,
    limits: &ValidationLimits,
) -> Result<BuiltPackageTelosc, TeloscArtifactError> {
    let package_identity = match &identity {
        StableProjectRef::Package { package, version, .. } => TeloscPackageIdentity::Package {
            package: package.clone(),
            version: version.clone(),
        },
        _ => return Err(incomplete("final package identity is not persistent")),
    };
    let mut header = image.base_header().as_ref().clone();
    header.container_schema = TELOSC_CONTAINER_SCHEMA_V2;
    header.minimum_reader_schema = TELOSC_MIN_READER_SCHEMA_V2;
    header.package_identity = package_identity.clone();
    header.capabilities = phalcom_compiler_artifact::header::TeloscCapabilities::CANONICAL_COMPILER_IMAGE_V1
        | phalcom_compiler_artifact::header::TeloscCapabilities::RUNTIME_PUBLIC_METADATA;
    header.source_digest = image.source_digest;
    header.interface_digest = ArtifactDigest::digest_domain(DOMAIN_MODULE_WORLD, &projection.module_world_bytes);
    header.semantic_digest = ArtifactDigest::digest_domain(DOMAIN_SEMANTIC_WORLD, &projection.semantic_world_bytes);
    header.query_digest = ArtifactDigest::digest_domain(DOMAIN_QUERY_IMAGE, &projection.query_image_bytes);
    header.metadata_digest = ArtifactDigest::digest_domain(DOMAIN_METADATA, &projection.metadata_bytes);
    header.artifact_digest = ArtifactDigest::ZERO;
    header.section_count = 4;

    let module_world_bytes = projection.module_world_bytes.len();
    let semantic_world_bytes = projection.semantic_world_bytes.len();
    let query_image_bytes = projection.query_image_bytes.len();
    let metadata_bytes = projection.metadata_bytes.len();

    let mut builder = TeloscContainerBuilder::new(header);
    builder.add_section(
        SectionKind::ModuleWorld,
        MODULE_WORLD_SCHEMA_V5,
        SectionFlags::REQUIRED,
        projection.module_world_bytes,
    );
    builder.add_section(
        SectionKind::SemanticWorld,
        SEMANTIC_WORLD_SCHEMA_V4,
        SectionFlags::REQUIRED,
        projection.semantic_world_bytes,
    );
    builder.add_section(
        SectionKind::QueryImage,
        QUERY_IMAGE_SCHEMA_V4,
        SectionFlags::REQUIRED,
        projection.query_image_bytes,
    );
    builder.add_section(
        SectionKind::Metadata,
        METADATA_SECTION_SCHEMA_V1,
        SectionFlags::REQUIRED,
        projection.metadata_bytes,
    );
    let bytes = builder.build().map_err(|error| incomplete(error.to_string()))?;
    let container = TeloscContainerView::parse(&bytes, limits).map_err(|error| incomplete(error.to_string()))?;
    let summary = TeloscBuildSummary {
        capabilities: TeloscCapabilities::compiler_only_v1(),
        module_world_bytes,
        semantic_world_bytes,
        query_image_bytes,
        metadata_bytes,
        executable_bytes: 0,
        artifact_digest: *container.header().artifact_digest.as_bytes(),
    };
    Ok(BuiltPackageTelosc { bytes, summary, identity })
}

fn collect_external_projects(
    identity: &StableProjectRef,
    module_world: &ModuleWorldV5,
    semantic_project_refs: &BTreeSet<StableProjectRef>,
    query_image: &phalcom_compiler_artifact::query::QueryImageV4,
    metadata: &phalcom_type_meta::SemanticMetadataBundle,
) -> Result<BTreeSet<StableProjectRef>, TeloscArtifactError> {
    let mut projects = module_world.external_projects.iter().cloned().collect::<BTreeSet<_>>();
    projects.extend(module_world.external_modules.iter().map(|module| module.project.clone()));
    projects.extend(semantic_project_refs.iter().cloned());
    for root in &query_image.supported_roots {
        collect_query_owner_project(&root.key.owner, &mut projects);
    }
    for evidence in &query_image.dependency_evidence {
        collect_query_owner_project(&evidence.key.owner, &mut projects);
        collect_query_owner_project(&evidence.stable_aggregate_owner, &mut projects);
    }
    let metadata_json = serde_json::to_value(metadata).map_err(|error| incomplete(error.to_string()))?;
    collect_metadata_projects(&metadata_json, &mut projects);
    projects.remove(identity);
    if projects
        .iter()
        .any(|project| matches!(project, StableProjectRef::SourceArtifact { .. } | StableProjectRef::Session { .. }))
    {
        return Err(incomplete("package projection contains a non-persistent external project identity"));
    }
    Ok(projects)
}

fn collect_metadata_projects(value: &serde_json::Value, projects: &mut BTreeSet<StableProjectRef>) {
    match value {
        serde_json::Value::Object(object) => {
            if let Some(project) = object.get("project")
                && let Ok(project) = serde_json::from_value::<StableProjectRef>(project.clone())
            {
                projects.insert(project);
            }
            for child in object.values() {
                collect_metadata_projects(child, projects);
            }
        }
        serde_json::Value::Array(items) => {
            for child in items {
                collect_metadata_projects(child, projects);
            }
        }
        _ => {}
    }
}

fn collect_query_owner_project(owner: &phalcom_compiler_artifact::query::PortableQueryOwnerKeyV1, projects: &mut BTreeSet<StableProjectRef>) {
    projects.insert(owner.project().clone());
}

fn package_identity_digest(image: &PackageCompilerImage) -> Result<[u8; 32], TeloscArtifactError> {
    match &image.identity {
        StableProjectRef::Package { package, version, .. } => Ok(TeloscPackageIdentity::Package {
            package: package.clone(),
            version: version.clone(),
        }
        .digest()
        .as_bytes()
        .to_owned()),
        _ => Err(incomplete("package compiler image has a non-package publishing identity")),
    }
}

fn incomplete(message: impl Into<String>) -> TeloscArtifactError {
    TeloscArtifactError::CanonicalImage(phalcom_semantic::canonical_image::CanonicalImageError::IncompletePublication(message.into()))
}
