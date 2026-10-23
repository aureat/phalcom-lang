//! Executable package artifacts: `slotmap.telosc` over `universe.telosc`.
//!
//! A full package artifact is the compiler-only package container
//! ([`super::package_artifact`]) plus one `EXECUTABLE` section holding the
//! portable bytecode of exactly the package-owned modules. Its lifecycle is:
//!
//! ```text
//! build:   PackageCompilerImage ─► compiler sections
//!          package program over Universe VM ─► ExecutableImageV1 (package scope)
//! load:    package container ─► prefix-admitted composition over Universe
//!          EXECUTABLE ─► package-scoped executable validation
//! install: Universe VM ─► package module objects from loaded linkage
//!          ─► initializers from portable prototypes ─► Universe → package order
//! ```
//!
//! Stable owners inside the executable resolve through an
//! [`ArtifactIdentityScope`] bound to the composed project catalog, so Universe
//! references land on generation-zero objects and package references on the
//! package generation. No package source is read after `build`.

use super::canonical_artifact::{LoadedUniverseArtifact, TeloscArtifactError, TeloscCapabilities};
use phalcom_compiler_artifact::header::TeloscCapabilities as HeaderCapabilities;
use phalcom_modules::ExternalModuleAuthority as _;
use super::compile::{CompiledModule, CompiledProgram, EntrySelection};
use super::package_artifact::{BuiltPackageTelosc, build_package_telosc};
use super::package_image::PackageCompilerImage;
use super::telosc_compiler_wire::{ComposedArtifactSemanticBase, StagedTeloscCompilerWire};
use crate::artifact::capture::{ExecutableImageCapture, PackageCaptureSource};
use crate::artifact::validate::ValidatedExecutableImage;
use crate::vm::VM;
use phalcom_compiler_artifact::container::{TeloscContainerBuilder, TeloscContainerView};
use phalcom_compiler_artifact::digest::ArtifactDigest;
use phalcom_compiler_artifact::executable::image::{ExecutableImageLimitsV1, ExecutableImageV1};
use phalcom_compiler_artifact::executable::staging::StagedExecutableSectionV1;
use phalcom_compiler_artifact::identity::TeloscPackageIdentity;
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::section::{SectionFlags, SectionKind};
use phalcom_modules::{ModuleComponent, ModuleId, ProjectUniverse, ResolvedProjectId, TrustedPackageResolver, ValidatedPackageAuthorityReceipt};
use phalcom_semantic::metadata::stable_identity::artifact_scope::ArtifactIdentityScope;
use phalcom_semantic::metadata::stable_identity::to_stable_module;
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::identity::{StableModuleRef, StableProjectRef};
use std::collections::BTreeSet;
use std::io::Write as _;
use std::path::Path;
use std::sync::Arc;

fn executable_error(error: impl std::fmt::Display) -> TeloscArtifactError {
    TeloscArtifactError::Executable(error.to_string())
}

fn total_budget(limits: &ValidationLimits) -> AllocationBudget {
    AllocationBudget::new(usize::try_from(limits.max_total_allocation_bytes).unwrap_or(usize::MAX))
}

/// Builds a full compiler + executable package artifact for `project_root`.
///
/// Package source is read here and only here. Universe authority comes from
/// `universe`, both for compilation and for the disposable capture VM; the
/// captured bytecode is never executed during the build.
pub fn build_full_package_telosc(
    project_root: impl AsRef<Path>,
    universe: &LoadedUniverseArtifact,
    limits: &ValidationLimits,
) -> Result<BuiltPackageTelosc, TeloscArtifactError> {
    let project_root = project_root.as_ref();
    let image = PackageCompilerImage::capture_from_universe(project_root, universe, limits).map_err(executable_error)?;
    let compiler = build_package_telosc(&image, limits)?;

    // Lowering bakes stable callable identities into bytecode recipes, so
    // compilation itself must already export the package's published identity.
    let compiled = {
        let _identity = ArtifactIdentityScope::enter(image.projects.as_ref(), Some((image.project, compiler.identity.clone())));
        universe
            .compile_user_entry(EntrySelection::Project(project_root.to_path_buf()))
            .map_err(executable_error)?
    };
    let project = match &compiled.entry.project {
        phalcom_modules::ProjectIdentity::Resolved(project) => *project,
        _ => return Err(TeloscArtifactError::Capability("package executable entry is not a resolved package project")),
    };
    let scope = compiled
        .modules
        .keys()
        .filter(|id| id.project == compiled.entry.project)
        .cloned()
        .collect::<BTreeSet<_>>();
    let staged = {
        // Publishing scope: package-owned identities export as the exact
        // published package identity; Universe identities stay builtin.
        let _identity = ArtifactIdentityScope::enter_with_aliases(compiled.project_universe.as_ref(), Some((project, compiler.identity.clone())), vec![image.project]);
        capture_package_executable(universe, &compiled, scope, Arc::clone(&image.snapshot))?
    };

    let executable_limits = ExecutableImageLimitsV1::default();
    let mut budget = total_budget(limits);
    let decoded = Arc::new(ExecutableImageV1::decode(staged.payload(), limits, &executable_limits, &mut budget)?);
    let expected = decoded.modules.iter().map(|module| module.identity.clone()).collect::<BTreeSet<_>>();
    crate::artifact::validate::validate_package_executable_image(decoded, &expected, &executable_limits).map_err(executable_error)?;

    let view = TeloscContainerView::parse(&compiler.bytes, limits)?;
    let mut header = view.header().clone();
    header.capabilities |= HeaderCapabilities::PORTABLE_EXECUTABLE;
    header.artifact_digest = ArtifactDigest::ZERO;
    header.section_count = header.section_count.checked_add(1).ok_or(TeloscArtifactError::Capability("section count overflow"))?;
    let mut builder = TeloscContainerBuilder::new(header);
    for entry in view.directory() {
        let payload = view.require_section(entry.kind).map_err(phalcom_compiler_artifact::ArtifactError::from)?;
        builder.add_section(entry.kind, entry.schema_version, entry.flags, payload.to_vec());
    }
    builder.add_section(SectionKind::Executable, staged.schema_version(), SectionFlags::REQUIRED, staged.payload().to_vec());
    let bytes = builder.build()?;
    let parsed = TeloscContainerView::parse(&bytes, limits)?;
    let mut summary = compiler.summary.clone();
    summary.capabilities = TeloscCapabilities::full_v1();
    summary.executable_bytes = staged.payload().len();
    summary.artifact_digest = *parsed.header().artifact_digest.as_bytes();
    Ok(BuiltPackageTelosc {
        bytes,
        summary,
        identity: compiler.identity,
    })
}

/// Compiles every package module closure in a disposable Universe VM and
/// records the emitted bytecode without running any initializer.
fn capture_package_executable(
    universe: &LoadedUniverseArtifact,
    compiled: &CompiledProgram,
    scope: BTreeSet<ModuleId>,
    snapshot: Arc<phalcom_semantic::snapshot::SemanticSnapshot>,
) -> Result<StagedExecutableSectionV1, TeloscArtifactError> {
    let mut vm = universe.new_vm().map_err(executable_error)?;
    vm.materialize_program(compiled).map_err(executable_error)?;
    let source = PackageCaptureSource {
        program: compiled,
        scope: scope.clone(),
        snapshot,
    };
    let mut capture = ExecutableImageCapture::new(&source);
    for id in compiled.initialization_order.iter().filter(|id| scope.contains(*id)) {
        let text = compiled
            .modules
            .get(id)
            .and_then(|module| module.source_text.clone())
            .ok_or(TeloscArtifactError::Capability("package module has no source text to compile"))?;
        let closure = vm.compile_program_module_closure(id, &text, compiled).map_err(executable_error)?;
        capture.capture_module(&mut vm, &source, id, closure);
        capture.capture_class_layouts(&vm, id);
    }
    capture.capture_detached_methods(&mut vm);
    let image = capture.finish(&mut vm, &source).map_err(executable_error)?;
    StagedExecutableSectionV1::from_image(&image).map_err(executable_error)
}

/// Atomically writes a full package artifact after re-validating its bytes.
pub fn write_full_package_telosc(path: &Path, universe: &LoadedUniverseArtifact, built: &BuiltPackageTelosc) -> Result<(), TeloscArtifactError> {
    load_package_telosc_from_bytes(&built.bytes, universe, &ValidationLimits::default())?;
    let parent = path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(&built.bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}

/// One package artifact admitted over a loaded Universe artifact.
pub struct LoadedPackageArtifact {
    /// Exact stable package identity bound by the artifact digest.
    pub identity: StableProjectRef,
    /// Native project allocated for the package in the composed catalog.
    pub project: ResolvedProjectId,
    base: Arc<ComposedArtifactSemanticBase>,
    metadata: Arc<phalcom_type_meta::SemanticMetadataBundle>,
    executable: Option<ValidatedExecutableImage>,
}

/// Receipt issuer for an artifact whose bytes this process just authenticated
/// by digest; it binds exactly one stable identity to one project.
struct ArtifactDigestResolver {
    identity: StableProjectRef,
    digest: ArtifactDigest,
    project: ResolvedProjectId,
}

impl TrustedPackageResolver for ArtifactDigestResolver {
    fn package_authority_receipt(&self, stable_project: &StableProjectRef) -> Option<ValidatedPackageAuthorityReceipt> {
        (stable_project == &self.identity)
            .then(|| ValidatedPackageAuthorityReceipt::from_trusted_resolver(self.identity.clone(), self.digest, self.project))
            .flatten()
    }
}

/// Reads and admits a package artifact from disk; see [`load_package_telosc_from_bytes`].
pub fn load_package_telosc(path: &Path, universe: &LoadedUniverseArtifact, limits: &ValidationLimits) -> Result<LoadedPackageArtifact, TeloscArtifactError> {
    load_package_telosc_from_bytes(&std::fs::read(path)?, universe, limits)
}

/// Admits a package artifact over `universe` without reading package source.
///
/// The compiler sections are composed with Universe through prefix authority;
/// a present `EXECUTABLE` section is validated against the package's exact
/// module scope.
pub fn load_package_telosc_from_bytes(
    bytes: &[u8],
    universe: &LoadedUniverseArtifact,
    limits: &ValidationLimits,
) -> Result<LoadedPackageArtifact, TeloscArtifactError> {
    let mut budget = total_budget(limits);
    let wire = StagedTeloscCompilerWire::decode(bytes, limits, &mut budget).map_err(executable_error)?;
    let (package, version) = match wire.package_identity() {
        TeloscPackageIdentity::Package { package, version } => (package.to_string(), version.to_string()),
        _ => return Err(TeloscArtifactError::Capability("package artifact must carry a package identity")),
    };
    let world = wire.module_world();
    let root = world
        .modules
        .get(world.entry.0 as usize)
        .ok_or(TeloscArtifactError::Capability("package artifact has no root module"))?;
    let identity = root.identity.project.clone();
    let scope = world.modules.iter().map(|module| module.identity.clone()).collect::<BTreeSet<StableModuleRef>>();
    let digest = wire.artifact_digest();

    let mut projects: ProjectUniverse = universe.formal_image.modules().projects().as_ref().clone();
    let namespace = ModuleComponent::from_identifier(&package).map_err(|_| TeloscArtifactError::Capability("package name is not a valid namespace"))?;
    let project = projects.register_artifact_package(&package, namespace, &format!("telos-artifact://{package}@{version}/{}", digest.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>()));
    let _identity = ArtifactIdentityScope::enter(&projects, Some((project, identity.clone())));
    let resolver = ArtifactDigestResolver {
        identity: identity.clone(),
        digest,
        project,
    };
    let base = StagedTeloscCompilerWire::compose_artifact_semantic_base_over_projects(
        vec![wire],
        Arc::clone(&universe.formal_image),
        &projects,
        &resolver,
        limits,
        &mut budget,
    )
    .map_err(executable_error)?;

    let container = TeloscContainerView::parse(bytes, limits)?;
    let metadata = {
        use phalcom_compiler_artifact::metadata::{MetadataPreflightLimits, MetadataSectionV1};
        let payload = container.require_section(SectionKind::Metadata).map_err(phalcom_compiler_artifact::ArtifactError::from)?;
        let section = MetadataSectionV1::decode_with_allocation_budget(payload, limits, &mut budget)?;
        let preflight = MetadataPreflightLimits {
            max_metadata_bytes: usize::try_from(limits.max_metadata_bytes).unwrap_or(usize::MAX),
            max_token_count: u32::try_from(limits.max_semantic_entries_per_family).unwrap_or(u32::MAX),
            max_container_count: u32::try_from(limits.max_semantic_entries_per_family).unwrap_or(u32::MAX),
            max_nesting_depth: u16::try_from(limits.max_recursion_depth).unwrap_or(u16::MAX),
        };
        let digest = *container.header().package_identity.digest().as_bytes();
        Arc::new(section.decode_bundle_with_allocation_budget(&preflight, &phalcom_type_meta::ValidationLimits::default(), &digest, &mut budget)?)
    };
    let executable = if container.header().capabilities.contains(HeaderCapabilities::PORTABLE_EXECUTABLE) {
        let payload = container.require_section(SectionKind::Executable).map_err(phalcom_compiler_artifact::ArtifactError::from)?;
        let executable_limits = ExecutableImageLimitsV1::default();
        let image = Arc::new(ExecutableImageV1::decode(payload, limits, &executable_limits, &mut budget)?);
        Some(crate::artifact::validate::validate_package_executable_image(image, &scope, &executable_limits).map_err(executable_error)?)
    } else {
        None
    };
    Ok(LoadedPackageArtifact {
        identity,
        project,
        base: Arc::new(base),
        metadata,
        executable,
    })
}

impl LoadedPackageArtifact {
    /// Whether the artifact carries installable package bytecode.
    pub fn is_executable(&self) -> bool {
        self.executable.is_some()
    }

    /// Installs and initializes this package generation in `vm`.
    ///
    /// `vm` must come from [`LoadedUniverseArtifact::new_vm`]: Universe is
    /// already initialized, so the package initializes strictly after its
    /// dependency. Package module objects are rebuilt from the loaded linked
    /// authority; initializers come only from the portable executable image.
    pub fn install_into(&self, vm: &mut VM) -> crate::error::PhResult<()> {
        let executable = self
            .executable
            .as_ref()
            .ok_or_else(|| crate::error::RuntimeError::Internal("package artifact has no executable section".into()))?;
        let generation = self
            .base
            .generations()
            .iter()
            .find(|generation| generation.modules.interfaces().keys().any(|id| id.project == phalcom_modules::ProjectIdentity::Resolved(self.project)))
            .ok_or_else(|| crate::error::RuntimeError::Internal("composed base has no package generation".into()))?;
        let linked = generation.modules.linked();
        let program = CompiledProgram {
            compile_mode: crate::compiler::attributes::CompileMode::Debug,
            project_universe: Arc::clone(self.base.projects()),
            linked: Arc::new(phalcom_modules::ExecutableLinkedProgram::from_artifact_generation(linked.as_ref().clone())),
            modules: linked
                .modules
                .iter()
                .map(|(id, module)| {
                    let lowering = Arc::new(super::semantic_lowering::ModuleLoweringSemantics::new(id.clone()));
                    (id.clone(), super::compile::compile_module(id.clone(), module, None, None, lowering))
                })
                .collect::<std::collections::BTreeMap<ModuleId, CompiledModule>>(),
            entry: linked.entry.clone(),
            initialization_order: linked.initialization_order.clone(),
            semantic_metadata: Some(Arc::clone(&self.metadata)),
        };
        let _identity = ArtifactIdentityScope::enter(self.base.projects().as_ref(), Some((self.project, self.identity.clone())));
        vm.materialize_program(&program)?;
        crate::artifact::materialize::install_package_executable(vm, executable)
    }

    /// Stable identities of the package modules this artifact owns.
    pub fn modules(&self) -> Vec<StableModuleRef> {
        self.executable
            .as_ref()
            .map(|executable| executable.image().modules.iter().map(|module| module.identity.clone()).collect())
            .unwrap_or_else(|| {
                let _identity = ArtifactIdentityScope::enter(self.base.projects().as_ref(), Some((self.project, self.identity.clone())));
                self.base
                    .generations()
                    .iter()
                    .flat_map(|generation| generation.modules.interfaces().keys().filter(|id| !id.project.is_universe()).map(to_stable_module).collect::<Vec<_>>())
                    .collect()
            })
    }
}
