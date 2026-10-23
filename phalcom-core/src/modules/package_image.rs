//! Source-backed capture of package-owned semantic products over a loaded artifact base.

use super::canonical_artifact::LoadedUniverseArtifact;
use super::compile::{
    AnalyzedProgram, ProgramCompilationOptions, ProgramCompileError, ProgramCompiler, ProgramSemanticDiagnostics, link_analyze_finalize_for_package_image,
};
use super::package_identity::package_source_digest;
use super::telosc_compiler_wire::{ComposedArtifactSemanticBase, StagedTeloscCompilerWire};
use crate::compiler::attributes::CompileMode;
use phalcom_compiler_artifact::header::TeloscHeaderV2;
use phalcom_compiler_artifact::identity::{PackageIdentityAbi, package_identity_fingerprint};
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_modules::{
    CanonicalExternalModuleWorld, ExternalModuleAuthority, FilesystemSourceProvider, ModuleId, ModuleLinker, ModulePath, ModuleResolutionError, ModuleResolver,
    ProjectUniverse, ResolvedProject, ResolverGeneration, SourceError, SourceId, SourceProvider, SourceUnit, TrustedPackageResolver,
};
use phalcom_semantic::artifact::projection::ArtifactProjectionScope;
use phalcom_semantic::canonical_image::{
    CanonicalDependencyFingerprints, CanonicalSemanticQueryImage, CanonicalSemanticWorldId, aggregate_dependency_fingerprints_scoped,
};
use phalcom_semantic::metadata::stable_identity::{StableIdentityContext, to_stable_project_with_context};
use phalcom_semantic::snapshot::SemanticSnapshot;
use phalcom_type_meta::SemanticMetadataBundle;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::StableProjectRef;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

/// A package's owned compiler products captured over one immutable artifact base.
pub struct PackageCompilerImage {
    /// Resolved project owner in `projects`.
    pub project: phalcom_modules::ResolvedProjectId,
    /// Stable, path-free identity used when this package is published.
    pub identity: StableProjectRef,
    /// Exact set of modules owned by the package artifact.
    pub scope: ArtifactProjectionScope,
    /// Project registry used to resolve stable module and project references.
    pub projects: Arc<ProjectUniverse>,
    /// Source-free linked module authority for the package-owned modules.
    pub modules: Arc<CanonicalExternalModuleWorld>,
    /// Executable linked package program.
    pub linked: Arc<phalcom_modules::ExecutableLinkedProgram>,
    /// Completed semantic snapshot for package compilation.
    pub snapshot: Arc<SemanticSnapshot>,
    /// Frozen package-owned semantic query products.
    pub queries: Arc<CanonicalSemanticQueryImage>,
    /// Owned query fingerprints and immutable external dependency evidence.
    pub dependency_fingerprints: Arc<CanonicalDependencyFingerprints>,
    /// Package-owned metadata projection with source-path-free identity inputs.
    pub metadata: Arc<SemanticMetadataBundle>,
    /// Immutable external module authority used during this capture.
    pub external: Arc<dyn ExternalModuleAuthority>,
    /// Path-free digest of all package-owned module paths and source text.
    pub source_digest: phalcom_compiler_artifact::ArtifactDigest,
    pub(crate) entry: ModuleId,
    pub(crate) sources: BTreeMap<ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
    pub(crate) identity_external_projects: Vec<StableProjectRef>,
    base_header: Arc<TeloscHeaderV2>,
    base: Arc<ComposedArtifactSemanticBase>,
    availability: Arc<phalcom_modules::AvailabilityContract>,
}

impl PackageCompilerImage {
    /// Captures a project over the already-loaded Universe artifact.
    ///
    /// Source reads are limited to `project_root`; Universe modules and semantic
    /// products come from `universe`. Package dependencies must already be part
    /// of the composed artifact base; this entry point composes the Universe-only
    /// base and therefore accepts projects whose external authority is Universe.
    pub fn capture_from_universe(
        project_root: impl AsRef<Path>,
        universe: &LoadedUniverseArtifact,
        limits: &ValidationLimits,
    ) -> Result<Self, PackageCompilerImageError> {
        let mut allocations = phalcom_type_meta::allocation::AllocationBudget::new(usize::try_from(limits.max_total_allocation_bytes).unwrap_or(usize::MAX));
        let resolver = NoPackageResolver;
        let base =
            StagedTeloscCompilerWire::compose_artifact_semantic_base(Vec::new(), Arc::clone(&universe.formal_image), &resolver, limits, &mut allocations)
                .map_err(|error| PackageCompilerImageError::ArtifactBase(error.to_string()))?;
        Self::capture_from_base(
            project_root.as_ref(),
            Arc::new(base),
            Arc::new(universe.header().clone()),
            Arc::clone(universe.availability()),
            limits,
        )
    }

    pub(crate) fn capture_from_base(
        project_root: &Path,
        base: Arc<ComposedArtifactSemanticBase>,
        base_header: Arc<TeloscHeaderV2>,
        availability: Arc<phalcom_modules::AvailabilityContract>,
        limits: &ValidationLimits,
    ) -> Result<Self, PackageCompilerImageError> {
        let manifest_path = if project_root.file_name().is_some_and(|name| name == "project.toml") {
            project_root.to_path_buf()
        } else {
            project_root.join("project.toml")
        };
        let mut project_universe = base.projects().as_ref().clone();
        let project = project_universe.load_root(&manifest_path).map_err(ProgramCompileError::Project)?;
        let resolved_project = project_universe
            .get_project(project)
            .ok_or_else(|| PackageCompilerImageError::Project("loaded package project was not registered".into()))?;
        let manifest = resolved_project
            .manifest
            .as_ref()
            .ok_or_else(|| PackageCompilerImageError::Project("package publication requires a validated project manifest".into()))?;
        let package_name = manifest.name.clone();
        let package_version = manifest
            .version
            .as_deref()
            .ok_or_else(|| PackageCompilerImageError::Project("package publication requires a version".into()))?
            .to_owned();
        let entry_path = resolved_project
            .entry
            .clone()
            .ok_or_else(|| PackageCompilerImageError::Project("package publication requires an entry module".into()))?;
        let package_source_root = resolved_project.source_root.clone();
        let entry = ModuleId::resolved(project, entry_path);
        let projects = Arc::new(project_universe);

        let external_modules: Arc<dyn ExternalModuleAuthority> = base.modules().clone();
        let provider = PackageOwnedSourceProvider {
            project,
            source_root: package_source_root,
            filesystem: FilesystemSourceProvider::new(),
        };
        let mut module_resolver = ModuleResolver::with_external_modules(projects.as_ref(), &provider, Arc::clone(&external_modules));
        let mut pending = vec![entry.clone(), ModuleId::resolved(project, ModulePath::root())];
        pending.sort_unstable();
        pending.dedup();
        let discovered = super::compile::discover_reachable_dependencies(
            &mut module_resolver,
            projects.as_ref(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            pending,
            None,
        )?;
        let runtime_modules = discovered.interfaces.keys().cloned().collect::<BTreeSet<_>>();
        if runtime_modules.is_empty()
            || runtime_modules
                .iter()
                .any(|module| module.project != phalcom_modules::ProjectIdentity::Resolved(project))
        {
            return Err(PackageCompilerImageError::Project(
                "package discovery did not produce a nonempty package-owned module set".into(),
            ));
        }
        let scope = ArtifactProjectionScope::new(runtime_modules.iter().cloned());

        let linker = ModuleLinker::new(Arc::clone(&projects), discovered.interfaces.clone()).with_external_modules(external_modules.clone());
        if let Some(universe_snapshot) = base.universe_snapshot() {
            phalcom_semantic::prelude::PreludeTypeMap::initialize_from_snapshot(universe_snapshot);
        } else {
            return Err(PackageCompilerImageError::ArtifactBase(
                "composed package base has no Universe semantic generation".into(),
            ));
        }
        let mut semantic_session = base
            .semantic_session(phalcom_semantic::identity::WorkspaceId::from_raw(1))
            .map_err(|error| PackageCompilerImageError::SemanticImage(error.to_string()))?
            .with_product_request(phalcom_semantic::request::SemanticProductRequest::canonical_compiler_image());
        let (linked, snapshot) = link_analyze_finalize_for_package_image(
            &linker,
            entry.clone(),
            &discovered.resolved,
            &discovered.sources,
            &runtime_modules,
            &mut semantic_session,
            &availability,
        )?;

        let source_digest = package_source_digest(project, discovered.sources.iter().map(|(module, source)| (module, source.text.as_ref())));
        let identity_external_projects = discovered
            .resolved
            .values()
            .filter(|module| module.project != phalcom_modules::ProjectIdentity::Resolved(project))
            .map(|module| module.project)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|project| to_stable_project_with_context(&project, &StableIdentityContext::new(projects.as_ref())))
            .collect::<Vec<_>>();
        let package_fingerprint = package_identity_fingerprint(
            &package_name,
            &package_version,
            source_digest,
            &identity_external_projects,
            &PackageIdentityAbi::from_header(&base_header),
        )
        .map_err(|error| PackageCompilerImageError::Identity(error.to_string()))?;
        let identity = StableProjectRef::Package {
            package: package_name.into(),
            version: package_version.into(),
            artifact_fingerprint: package_fingerprint,
        };
        let identity_context = StableIdentityContext::publishing(projects.as_ref(), project, &identity);
        let source_fingerprint = Fingerprint128(source_digest.as_bytes()[..16].try_into().expect("digest prefix is 16 bytes"));

        let linked_modules = linked.as_program().modules.keys().cloned().collect::<BTreeSet<_>>();
        if linked_modules != runtime_modules {
            return Err(PackageCompilerImageError::ModuleWorld(
                "package linked module set differs from discovered package interfaces".into(),
            ));
        }
        let source_locations = discovered
            .sources
            .iter()
            .filter_map(|(module, source)| source.source.clone().map(|location| (module.clone(), location)))
            .collect::<BTreeMap<_, _>>();
        let topology = Arc::new(phalcom_modules::ModuleTopology::from_parts(
            ResolverGeneration(0),
            &projects,
            &discovered.interfaces,
            &source_locations,
        ));
        let interfaces = discovered
            .interfaces
            .iter()
            .map(|(module, interface)| (module.clone(), Arc::new(interface.clone())))
            .collect::<BTreeMap<_, _>>();
        let modules = Arc::new(
            CanonicalExternalModuleWorld::from_parts(Arc::new(linked.as_program().clone()), interfaces, topology)
                .map_err(PackageCompilerImageError::ModuleWorld)?,
        );
        let world_sources = discovered
            .sources
            .iter()
            .filter(|(module, _)| scope.owns_module(module))
            .map(|(module, source)| (module.clone(), Arc::<str>::from(source.text.as_ref())))
            .collect::<BTreeMap<_, _>>();
        let world_fingerprint = u64::from_le_bytes(source_digest.as_bytes()[..8].try_into().expect("digest prefix is 8 bytes"));
        let world = CanonicalSemanticWorldId::for_source_world(world_fingerprint, Arc::new(world_sources));
        let queries = Arc::new(
            CanonicalSemanticQueryImage::freeze_scoped(semantic_session.db(), world, &snapshot, &modules, &scope, base.query_catalogue().as_ref())
                .map_err(|error| PackageCompilerImageError::SemanticImage(error.to_string()))?,
        );
        reject_owned_query_terminals(&queries)?;
        let dependency_fingerprints = Arc::new(
            aggregate_dependency_fingerprints_scoped(semantic_session.db(), &snapshot, &queries, &scope, base.query_catalogue().as_ref())
                .map_err(|error| PackageCompilerImageError::SemanticImage(error.to_string()))?,
        );

        reject_owned_errors(&snapshot, &scope)?;
        let sources = discovered.sources.clone();
        let analyzed = AnalyzedProgram {
            compile_mode: CompileMode::Debug,
            project_universe: Arc::clone(&projects),
            linked: Arc::clone(&linked),
            semantic: Arc::clone(&snapshot),
            sources: discovered.sources,
            runtime_modules,
            entry: entry.clone(),
        };
        let compiled = ProgramCompiler::project_analyzed_for_package_image(
            &analyzed,
            &ProgramCompilationOptions::default(),
            &scope,
            identity_context,
            source_fingerprint,
        )?;
        let metadata = compiled
            .semantic_metadata
            .ok_or_else(|| PackageCompilerImageError::Metadata("package projection produced no metadata bundle".into()))?;

        Ok(Self {
            project,
            identity,
            scope,
            projects,
            modules,
            linked,
            snapshot,
            queries,
            dependency_fingerprints,
            metadata,
            external: external_modules,
            source_digest,
            entry,
            sources,
            identity_external_projects,
            base_header,
            base,
            availability,
        })
    }

    pub(crate) fn base_header(&self) -> &Arc<TeloscHeaderV2> {
        &self.base_header
    }

    pub(crate) fn base(&self) -> &Arc<ComposedArtifactSemanticBase> {
        &self.base
    }

    pub(crate) fn availability(&self) -> &Arc<phalcom_modules::AvailabilityContract> {
        &self.availability
    }
}

fn reject_owned_errors(snapshot: &SemanticSnapshot, scope: &ArtifactProjectionScope) -> Result<(), PackageCompilerImageError> {
    let by_module = snapshot
        .diagnostics
        .iter()
        .filter(|(module, _)| scope.owns_module(module))
        .map(|(module, diagnostics)| (module.clone(), diagnostics.to_vec()))
        .collect::<BTreeMap<_, _>>();
    let has_errors = by_module
        .values()
        .flatten()
        .any(|diagnostic| diagnostic.severity == phalcom_semantic::DiagnosticSeverity::Error);
    if has_errors {
        return Err(ProgramCompileError::Semantic(ProgramSemanticDiagnostics { by_module, snapshot: None }).into());
    }
    Ok(())
}

fn reject_owned_query_terminals(queries: &CanonicalSemanticQueryImage) -> Result<(), PackageCompilerImageError> {
    let terminals = &queries.census().terminal_diagnostics;
    if !terminals.is_empty() {
        let details = terminals
            .iter()
            .map(|(key, reason)| format!("{key:?}: {reason}"))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(PackageCompilerImageError::SemanticImage(format!(
            "package capture rejected owned Failed/Blocked query products: {details}"
        )));
    }
    Ok(())
}

/// Source access restricted to one package owner during artifact capture.
struct PackageOwnedSourceProvider {
    project: phalcom_modules::ResolvedProjectId,
    source_root: std::path::PathBuf,
    filesystem: FilesystemSourceProvider,
}

impl SourceProvider for PackageOwnedSourceProvider {
    fn locate(&self, project: &ResolvedProject, path: &ModulePath) -> Result<SourceUnit, ModuleResolutionError> {
        if project.id != self.project {
            return Err(SourceError::CapabilityUnavailable {
                module: ModuleId::resolved(project.id, path.clone()),
            }
            .into());
        }
        self.filesystem.locate(project, path)
    }

    fn read(&self, source: &SourceId) -> Result<Arc<str>, SourceError> {
        let path = Path::new(&*source.0);
        let source_root = phalcom_modules::source::canonicalize_path(&self.source_root);
        if !path.starts_with(&source_root) {
            return Err(SourceError::NotFound(format!(
                "package capture denied source outside its owner: {}",
                path.display()
            )));
        }
        self.filesystem.read(source)
    }

    fn read_uncached(&self, source: &SourceId) -> Result<Arc<str>, SourceError> {
        let path = Path::new(&*source.0);
        let source_root = phalcom_modules::source::canonicalize_path(&self.source_root);
        if !path.starts_with(&source_root) {
            return Err(SourceError::NotFound(format!(
                "package capture denied source outside its owner: {}",
                path.display()
            )));
        }
        self.filesystem.read_uncached(source)
    }
}

struct NoPackageResolver;

impl TrustedPackageResolver for NoPackageResolver {
    fn package_authority_receipt(&self, _stable_project: &StableProjectRef) -> Option<phalcom_modules::ValidatedPackageAuthorityReceipt> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_capture_rejects_foreign_dependency_source_before_parsing() {
        let temp = tempfile::tempdir().expect("temporary package roots");
        let own_root = write_project(temp.path(), "owner");
        let dependency_root = write_project(temp.path(), "dependency");
        let mut projects = ProjectUniverse::new();
        let owner = projects.load_root(own_root.join("project.toml")).expect("load owner project");
        let dependency = projects.load_root(dependency_root.join("project.toml")).expect("load local dependency project");
        let owner_source_root = projects.get_project(owner).expect("owner project").source_root.clone();
        let provider = PackageOwnedSourceProvider {
            project: owner,
            source_root: owner_source_root,
            filesystem: FilesystemSourceProvider::new(),
        };
        let mut resolver = ModuleResolver::new(&projects, &provider);

        let error = resolver
            .load_parsed(&ModuleId::resolved(dependency, ModulePath::root()))
            .expect_err("a missing external artifact must not fall back to dependency source");
        assert!(matches!(
            error,
            phalcom_modules::ModuleLoadError::Resolution(ModuleResolutionError::Source(SourceError::CapabilityUnavailable { .. }))
        ));
    }

    fn write_project(parent: &Path, name: &str) -> std::path::PathBuf {
        let root = parent.join(name);
        let source = root.join("src");
        std::fs::create_dir_all(&source).expect("create package source root");
        std::fs::write(
            root.join("project.toml"),
            format!("[project]\nname = \"{name}\"\nnamespace = \"{name}\"\nversion = \"0.1.0\"\nsource = \"src\"\nentry = \"{name}.main\"\n"),
        )
        .expect("write project manifest");
        std::fs::write(source.join("package.ph"), "expose .main\n").expect("write package descriptor");
        std::fs::write(source.join("main.ph"), "export value\nlet value = 1\n").expect("write dependency source");
        root
    }
}

/// Failures while capturing an owned package image over immutable artifacts.
#[derive(Debug, Error)]
pub enum PackageCompilerImageError {
    /// Source compilation or semantic linking failed.
    #[error(transparent)]
    Compile(#[from] ProgramCompileError),
    /// The composed external artifact base was invalid or incomplete.
    #[error("artifact base error: {0}")]
    ArtifactBase(String),
    /// Package identity inputs could not be computed.
    #[error("package identity error: {0}")]
    Identity(String),
    /// Package discovery or ownership was inconsistent.
    #[error("package image project error: {0}")]
    Project(String),
    /// Linked, interface, and topology module sets disagreed.
    #[error("package module world error: {0}")]
    ModuleWorld(String),
    /// Canonical semantic query or fingerprint publication failed.
    #[error("package semantic image error: {0}")]
    SemanticImage(String),
    /// Scoped metadata projection failed.
    #[error("package metadata error: {0}")]
    Metadata(String),
}
