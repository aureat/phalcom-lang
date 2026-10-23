//! Executable package artifacts: `slotmap.telosc` over `universe.telosc`.
//!
//! A full package artifact is the compiler-only package container
//! ([`super::package_artifact`]) plus one `EXECUTABLE` section holding the
//! portable bytecode of exactly the package-owned modules. Its lifecycle is:
//!
//! ```text
//! build:   PackageCompilerImage ─► compiler sections
//!          package program over Universe + dependency VM ─► ExecutableImageV1 (package scope)
//! load:    dependency + package containers ─► prefix-admitted composition over Universe
//!          EXECUTABLE ─► package-scoped executable validation
//! install: Universe VM ─► package module objects from loaded linkage
//!          ─► initializers from portable prototypes ─► Universe → dependencies → package
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
/// `universe` and each dependency package from its artifact bytes in
/// `dependencies`, both for compilation and for the disposable capture VM. A
/// `project.toml` dependency binds to the dependency artifact of the same
/// package name, never to its source. The package's own captured bytecode is
/// never executed during the build; dependency initializers run once in the
/// capture VM so their runtime objects exist.
pub fn build_full_package_telosc(
    project_root: impl AsRef<Path>,
    universe: &LoadedUniverseArtifact,
    dependencies: &[Vec<u8>],
    limits: &ValidationLimits,
) -> Result<BuiltPackageTelosc, TeloscArtifactError> {
    let project_root = project_root.as_ref();
    let prefix = admit_package_set(dependencies, universe, limits)?;
    let image = PackageCompilerImage::capture_from_base(
        project_root,
        Arc::clone(&prefix.base),
        Arc::new(universe.header().clone()),
        Arc::clone(universe.availability()),
        limits,
    )
    .map_err(executable_error)?;
    let compiler = build_package_telosc(&image, limits)?;

    // Lowering bakes stable callable identities into bytecode recipes, so
    // compilation itself must already export the package's published identity.
    let compiled = {
        let _identity = ArtifactIdentityScope::enter(image.projects.as_ref(), Some((image.project, compiler.identity.clone())));
        // Always over the admitted base, even with no dependencies: its
        // project universe numbers the package by package identity.
        super::compile::ProgramCompiler::compile_entry_selection_with_options_artifact_base_and_availability(
            EntrySelection::Project(project_root.to_path_buf()),
            Arc::clone(&prefix.base),
            Arc::clone(universe.availability()),
            &super::compile::ProgramCompilationOptions::default(),
        )
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
        let mut vm = universe.new_vm().map_err(executable_error)?;
        prefix.install_into(&mut vm).map_err(executable_error)?;
        // Publishing scope: package-owned identities export as the exact
        // published package identity; upstream identities stay as published.
        let _identity = ArtifactIdentityScope::enter_with_aliases(compiled.project_universe.as_ref(), Some((project, compiler.identity.clone())), vec![image.project]);
        capture_package_executable(&mut vm, &compiled, scope, Arc::clone(&image.snapshot))?
    };

    let executable_limits = ExecutableImageLimitsV1::default();
    let mut budget = total_budget(limits);
    let decoded = Arc::new(ExecutableImageV1::decode(staged.payload(), limits, &executable_limits, &mut budget)?);
    let expected = decoded.modules.iter().map(|module| module.identity.clone()).collect::<BTreeSet<_>>();
    let prefixes = prefix.executable_prefixes(universe, None)?;
    crate::artifact::validate::validate_package_executable_image(decoded, &expected, &prefixes, &executable_limits).map_err(executable_error)?;

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

/// Compiles every package module closure in `vm` (Universe plus installed
/// dependencies) and records the emitted bytecode without running any
/// package initializer.
fn capture_package_executable(
    vm: &mut VM,
    compiled: &CompiledProgram,
    scope: BTreeSet<ModuleId>,
    snapshot: Arc<phalcom_semantic::snapshot::SemanticSnapshot>,
) -> Result<StagedExecutableSectionV1, TeloscArtifactError> {
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
        capture.capture_module(vm, &source, id, closure);
        capture.capture_class_layouts(vm, id);
    }
    capture.capture_detached_methods(vm);
    let image = capture.finish(vm, &source).map_err(executable_error)?;
    StagedExecutableSectionV1::from_image(&image).map_err(executable_error)
}

/// Atomically writes a full package artifact after re-validating its bytes
/// over `universe` and the same `dependencies` it was built against.
pub fn write_full_package_telosc(
    path: &Path,
    universe: &LoadedUniverseArtifact,
    dependencies: &[Vec<u8>],
    built: &BuiltPackageTelosc,
) -> Result<(), TeloscArtifactError> {
    load_package_telosc_from_bytes(&built.bytes, universe, dependencies, &ValidationLimits::default())?;
    let parent = path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(&built.bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}

/// One package of an admitted package set.
struct AdmittedPackage {
    identity: StableProjectRef,
    project: ResolvedProjectId,
    metadata: Arc<phalcom_type_meta::SemanticMetadataBundle>,
    executable: Option<ValidatedExecutableImage>,
}

/// Package artifacts composed over one loaded Universe as a single immutable
/// semantic base. `packages` is in dependency (installation) order.
struct AdmittedPackageSet {
    base: Arc<ComposedArtifactSemanticBase>,
    packages: Vec<AdmittedPackage>,
}

impl AdmittedPackageSet {
    /// Universe's executable image followed by every package image except
    /// `excluded`, the prefix a package executable resolves upstream owners in.
    fn executable_prefixes<'a>(
        &'a self,
        universe: &'a LoadedUniverseArtifact,
        excluded: Option<ResolvedProjectId>,
    ) -> Result<Vec<&'a ExecutableImageV1>, TeloscArtifactError> {
        let root = universe
            .executable()
            .ok_or(TeloscArtifactError::Capability("package executable requires a Universe artifact with a portable executable"))?;
        Ok(std::iter::once(root.image())
            .chain(
                self.packages
                    .iter()
                    .filter(|package| Some(package.project) != excluded)
                    .filter_map(|package| package.executable.as_ref().map(ValidatedExecutableImage::image)),
            )
            .collect())
    }

    /// Installs and initializes every package in dependency order.
    fn install_into(&self, vm: &mut VM) -> crate::error::PhResult<()> {
        for package in &self.packages {
            install_package(&self.base, package, vm)?;
        }
        Ok(())
    }
}

/// Receipt issuer for artifacts whose bytes this process just authenticated
/// by digest; each binds exactly one stable identity to one project.
struct ArtifactDigestResolver {
    claims: Vec<(StableProjectRef, ArtifactDigest, ResolvedProjectId)>,
}

impl TrustedPackageResolver for ArtifactDigestResolver {
    fn package_authority_receipt(&self, stable_project: &StableProjectRef) -> Option<ValidatedPackageAuthorityReceipt> {
        self.claims
            .iter()
            .find(|(identity, ..)| identity == stable_project)
            .and_then(|(identity, digest, project)| ValidatedPackageAuthorityReceipt::from_trusted_resolver(identity.clone(), *digest, *project))
    }
}

/// Admits package artifacts over `universe` without reading package source.
///
/// Every artifact receives a project in one catalog extending Universe's and
/// a digest-bound authority receipt; the compiler sections are then composed
/// in dependency order, each resolving external owners only against the
/// generations before it. Each `EXECUTABLE` section is validated against its
/// exact module scope and against Universe plus the other packages' images.
fn admit_package_set(blobs: &[Vec<u8>], universe: &LoadedUniverseArtifact, limits: &ValidationLimits) -> Result<AdmittedPackageSet, TeloscArtifactError> {
    struct Claim {
        identity: StableProjectRef,
        project: ResolvedProjectId,
        scope: BTreeSet<StableModuleRef>,
    }
    let mut budget = total_budget(limits);
    let mut projects: ProjectUniverse = universe.formal_image.modules().projects().as_ref().clone();
    // Package projects, admitted here or compiled later over this base, are
    // numbered by package identity so fingerprints match their builds.
    projects.use_package_identity_ids();
    let decoded_wires = blobs
        .iter()
        .map(|bytes| StagedTeloscCompilerWire::decode(bytes, limits, &mut budget).map_err(executable_error))
        .collect::<Result<Vec<_>, _>>()?;
    let order = admission_order(&decoded_wires)?;
    let publishing_index = order.iter().position(|index| *index + 1 == blobs.len());
    let blobs = order.iter().map(|index| &blobs[*index]).collect::<Vec<_>>();
    let mut decoded_wires = decoded_wires.into_iter().map(Some).collect::<Vec<_>>();
    let mut wires = Vec::with_capacity(blobs.len());
    let mut claims = Vec::with_capacity(blobs.len());
    let mut resolver = ArtifactDigestResolver { claims: Vec::with_capacity(blobs.len()) };
    for index in &order {
        let wire = decoded_wires[*index].take().ok_or(TeloscArtifactError::Capability("package artifact admitted twice"))?;
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
        let namespace = ModuleComponent::from_identifier(&package).map_err(|_| TeloscArtifactError::Capability("package name is not a valid namespace"))?;
        let uri = format!("telos-artifact://{package}@{version}/{}", digest.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>());
        let project = projects
            .register_artifact_package(&package, &version, namespace, &uri)
            .map_err(|error| executable_error(format!("package artifact registration rejected: {error}")))?;
        if claims.iter().any(|claim: &Claim| claim.project == project || claim.identity == identity) {
            return Err(TeloscArtifactError::Capability("package artifact set admits one artifact twice"));
        }
        resolver.claims.push((identity.clone(), digest, project));
        claims.push(Claim { identity, project, scope });
        wires.push(wire);
    }
    for (identity, digest, _) in &resolver.claims {
        projects
            .register_trusted_package_authority(identity, Some(*digest), &resolver)
            .map_err(|error| executable_error(format!("package authority rejected: {error}")))?;
    }

    // The last blob is the publishing package (the run target, or the
    // nearest dependency of a build); admission order may have moved it.
    let publishing = publishing_index.and_then(|index| claims.get(index)).map(|claim| (claim.project, claim.identity.clone()));
    let _identity = ArtifactIdentityScope::enter(&projects, publishing);
    let base = StagedTeloscCompilerWire::compose_artifact_semantic_base_over_projects(
        wires,
        Arc::clone(&universe.formal_image),
        &projects,
        &resolver,
        limits,
        &mut budget,
    )
    .map_err(executable_error)?;

    let mut decoded = Vec::with_capacity(blobs.len());
    for bytes in blobs.iter().copied() {
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
            Some(Arc::new(ExecutableImageV1::decode(payload, limits, &ExecutableImageLimitsV1::default(), &mut budget)?))
        } else {
            None
        };
        decoded.push((metadata, executable));
    }

    let root = universe
        .executable()
        .ok_or(TeloscArtifactError::Capability("package executable requires a Universe artifact with a portable executable"))?;
    let mut packages = Vec::with_capacity(blobs.len());
    for (index, (claim, (metadata, executable))) in claims.iter().zip(&decoded).enumerate() {
        let executable = match executable {
            Some(image) => {
                let prefixes = std::iter::once(root.image())
                    .chain(decoded.iter().enumerate().filter(|(other, _)| *other != index).filter_map(|(_, (_, image))| image.as_deref()))
                    .collect::<Vec<_>>();
                Some(
                    crate::artifact::validate::validate_package_executable_image(Arc::clone(image), &claim.scope, &prefixes, &ExecutableImageLimitsV1::default())
                        .map_err(executable_error)?,
                )
            }
            None => None,
        };
        packages.push(AdmittedPackage {
            identity: claim.identity.clone(),
            project: claim.project,
            metadata: Arc::clone(metadata),
            executable,
        });
    }

    let generation_of = |project: ResolvedProjectId| {
        base.generations()
            .iter()
            .position(|generation| generation.modules.interfaces().keys().any(|id| id.project == phalcom_modules::ProjectIdentity::Resolved(project)))
    };
    let mut ordered = Vec::with_capacity(packages.len());
    for package in packages {
        let generation = generation_of(package.project).ok_or(TeloscArtifactError::Capability("composed base has no package generation"))?;
        ordered.push((generation, package));
    }
    ordered.sort_by_key(|(generation, _)| *generation);
    Ok(AdmittedPackageSet {
        base: Arc::new(base),
        packages: ordered.into_iter().map(|(_, package)| package).collect(),
    })
}

/// Orders decoded package wires so every package is admitted after the
/// packages it references, ties broken by stable project identity.
///
/// Project ids come from package identity, not from this order; the order
/// only makes admission deterministic and independent of `--dep` order.
fn admission_order(wires: &[StagedTeloscCompilerWire]) -> Result<Vec<usize>, TeloscArtifactError> {
    let identities = wires
        .iter()
        .map(|wire| {
            let world = wire.module_world();
            world
                .modules
                .get(world.entry.0 as usize)
                .map(|root| root.identity.project.clone())
                .ok_or(TeloscArtifactError::Capability("package artifact has no root module"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let dependencies = wires
        .iter()
        .map(|wire| {
            let external = &wire.module_world().external_projects;
            (0..identities.len()).filter(|other| external.contains(&identities[*other])).collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut order = Vec::with_capacity(wires.len());
    let mut admitted = vec![false; wires.len()];
    while order.len() < wires.len() {
        let next = (0..wires.len())
            .filter(|index| !admitted[*index] && dependencies[*index].iter().all(|dependency| admitted[*dependency]))
            .min_by(|left, right| identities[*left].cmp(&identities[*right]))
            .ok_or(TeloscArtifactError::Capability("package artifact set has a dependency cycle"))?;
        admitted[next] = true;
        order.push(next);
    }
    Ok(order)
}

/// Installs and initializes one admitted package generation in `vm`.
///
/// Package module objects are rebuilt from the loaded linked authority;
/// initializers come only from the portable executable image.
fn install_package(base: &Arc<ComposedArtifactSemanticBase>, package: &AdmittedPackage, vm: &mut VM) -> crate::error::PhResult<()> {
    let executable = package
        .executable
        .as_ref()
        .ok_or_else(|| crate::error::RuntimeError::Internal("package artifact has no executable section".into()))?;
    let generation = base
        .generations()
        .iter()
        .find(|generation| generation.modules.interfaces().keys().any(|id| id.project == phalcom_modules::ProjectIdentity::Resolved(package.project)))
        .ok_or_else(|| crate::error::RuntimeError::Internal("composed base has no package generation".into()))?;
    let linked = generation.modules.linked();
    let program = CompiledProgram {
        compile_mode: crate::compiler::attributes::CompileMode::Debug,
        project_universe: Arc::clone(base.projects()),
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
        semantic_metadata: Some(Arc::clone(&package.metadata)),
    };
    let _identity = ArtifactIdentityScope::enter(base.projects().as_ref(), Some((package.project, package.identity.clone())));
    vm.materialize_program(&program)?;
    crate::artifact::materialize::install_package_executable(vm, executable)
}

/// One package artifact admitted over a loaded Universe artifact and the
/// dependency package artifacts it was compiled against.
pub struct LoadedPackageArtifact {
    /// Exact stable package identity bound by the artifact digest.
    pub identity: StableProjectRef,
    /// Native project allocated for the package in the composed catalog.
    pub project: ResolvedProjectId,
    set: AdmittedPackageSet,
}

/// Reads and admits a package artifact from disk; see [`load_package_telosc_from_bytes`].
pub fn load_package_telosc(
    path: &Path,
    universe: &LoadedUniverseArtifact,
    dependencies: &[std::path::PathBuf],
    limits: &ValidationLimits,
) -> Result<LoadedPackageArtifact, TeloscArtifactError> {
    let dependencies = dependencies.iter().map(std::fs::read).collect::<Result<Vec<_>, _>>()?;
    load_package_telosc_from_bytes(&std::fs::read(path)?, universe, &dependencies, limits)
}

/// Admits a package artifact over `universe` and its `dependencies` without
/// reading package source.
///
/// The compiler sections of all artifacts are composed with Universe through
/// prefix authority; a present `EXECUTABLE` section is validated against the
/// package's exact module scope and its upstream images.
pub fn load_package_telosc_from_bytes(
    bytes: &[u8],
    universe: &LoadedUniverseArtifact,
    dependencies: &[Vec<u8>],
    limits: &ValidationLimits,
) -> Result<LoadedPackageArtifact, TeloscArtifactError> {
    let mut blobs = dependencies.to_vec();
    blobs.push(bytes.to_vec());
    let set = admit_package_set(&blobs, universe, limits)?;
    let target = {
        let mut budget = total_budget(limits);
        let wire = StagedTeloscCompilerWire::decode(bytes, limits, &mut budget).map_err(executable_error)?;
        let world = wire.module_world();
        world
            .modules
            .get(world.entry.0 as usize)
            .ok_or(TeloscArtifactError::Capability("package artifact has no root module"))?
            .identity
            .project
            .clone()
    };
    let project = set
        .packages
        .iter()
        .find(|package| package.identity == target)
        .map(|package| package.project)
        .ok_or(TeloscArtifactError::Capability("admitted package set lost its target artifact"))?;
    Ok(LoadedPackageArtifact { identity: target, project, set })
}

impl LoadedPackageArtifact {
    /// Whether the artifact carries installable package bytecode.
    pub fn is_executable(&self) -> bool {
        self.target().executable.is_some()
    }

    fn target(&self) -> &AdmittedPackage {
        self.set
            .packages
            .iter()
            .find(|package| package.project == self.project)
            .expect("admitted package set contains its target")
    }

    /// Installs and initializes the dependency packages and then this
    /// package in `vm`.
    ///
    /// `vm` must come from [`LoadedUniverseArtifact::new_vm`]: Universe is
    /// already initialized, so every package initializes strictly after its
    /// dependencies.
    pub fn install_into(&self, vm: &mut VM) -> crate::error::PhResult<()> {
        self.set.install_into(vm)
    }

    /// Stable identities of the package modules this artifact owns.
    pub fn modules(&self) -> Vec<StableModuleRef> {
        let target = self.target();
        target
            .executable
            .as_ref()
            .map(|executable| executable.image().modules.iter().map(|module| module.identity.clone()).collect())
            .unwrap_or_else(|| {
                let _identity = ArtifactIdentityScope::enter(self.set.base.projects().as_ref(), Some((self.project, self.identity.clone())));
                self.set
                    .base
                    .generations()
                    .iter()
                    .flat_map(|generation| {
                        generation
                            .modules
                            .interfaces()
                            .keys()
                            .filter(|id| id.project == phalcom_modules::ProjectIdentity::Resolved(self.project))
                            .map(to_stable_module)
                            .collect::<Vec<_>>()
                    })
                    .collect()
            })
    }
}
