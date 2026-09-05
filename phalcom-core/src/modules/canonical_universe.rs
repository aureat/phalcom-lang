//! Process-shared compiler product for the canonical source-authored Universe.
//!
//! This module owns only immutable, VM-independent products. Runtime
//! materialization, bytecode compilation, and source execution remain local to
//! each [`crate::vm::VM`].

use crate::modules::{AnalyzedProgram, CompiledProgram, ProgramCompiler};
use crate::native::{NativeSourceIndex, PRIMITIVES, verify_native_contracts};
use phalcom_modules::{FilesystemSourceProvider, ImportSurface, ModuleId, ModuleLinker, ModuleResolver, ProjectUniverse};
use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};
use thiserror::Error;

#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

/// Immutable compiler-side representation of the canonical source Universe.
///
/// The product may be shared between VMs because it contains no heap handles,
/// runtime classes, closures, chunks, or mutable runtime registries.
#[derive(Clone, Debug)]
pub(crate) struct CanonicalUniverseProgram {
    /// VM-independent linked and lowered module program.
    program: Arc<CompiledProgram>,
    /// Canonical parsed source units used by each VM's local compiler.
    source_index: Arc<NativeSourceIndex>,
    /// Modules reachable from the canonical Universe root.
    root_reachable: Arc<[ModuleId]>,
    /// Existing eager source-bootstrap order, retained without recomputation.
    bootstrap_order: Arc<[ModuleId]>,
}

impl CanonicalUniverseProgram {
    pub(crate) fn program(&self) -> &CompiledProgram {
        &self.program
    }

    pub(crate) fn source_index(&self) -> &NativeSourceIndex {
        &self.source_index
    }

    pub(crate) fn root_reachable(&self) -> &[ModuleId] {
        &self.root_reachable
    }

    pub(crate) fn bootstrap_order(&self) -> &[ModuleId] {
        &self.bootstrap_order
    }
}

/// Failure while deriving the process-shared canonical compiler product.
#[derive(Debug, Error)]
pub(crate) enum CanonicalUniverseBuildError {
    /// Canonical source discovery or parsing failed.
    #[error("canonical Universe source index failed: {0}")]
    SourceIndex(String),
    /// Native/source conformance failed.
    #[error("canonical native source contracts failed: {0}")]
    NativeContracts(String),
    /// A canonical interface could not be loaded.
    #[error("canonical Universe interface load failed for {module}: {error}")]
    InterfaceLoad { module: ModuleId, error: String },
    /// A canonical import could not be resolved.
    #[error("canonical Universe import resolution failed for {module} and {path}: {error}")]
    ImportResolution { module: ModuleId, path: String, error: String },
    /// Canonical source linking failed.
    #[error("canonical Universe linking failed: {0}")]
    Linking(String),
    /// Canonical semantic analysis panicked at its existing boundary.
    #[error("canonical Universe semantic analysis panicked")]
    SemanticAnalysisPanicked,
    /// Canonical semantic-to-runtime projection or compilation failed.
    #[error("canonical Universe compilation failed: {0}")]
    Compilation(String),
    /// Canonical root reachability/order derivation failed.
    #[error("canonical Universe bootstrap planning failed: {0}")]
    BootstrapPlanning(String),
    /// A root-reachable module is absent from the canonical parsed source index.
    #[error("canonical Universe root-reachable module {0} has no parsed source")]
    MissingReachableSource(ModuleId),
    /// A root-reachable module is absent from the compiled projection.
    #[error("canonical Universe root-reachable module {0} has no compiled module")]
    MissingReachableCompiledModule(ModuleId),
    /// A root-reachable module is absent from the linked program.
    #[error("canonical Universe root-reachable module {0} has no linked module")]
    MissingReachableLinkedModule(ModuleId),
    /// A bootstrap module is absent from the canonical parsed source index.
    #[error("canonical Universe bootstrap module {0} has no parsed source")]
    MissingBootstrapSource(ModuleId),
    /// A bootstrap module is absent from the compiled projection.
    #[error("canonical Universe bootstrap module {0} has no compiled module")]
    MissingBootstrapCompiledModule(ModuleId),
    /// A bootstrap module is absent from the linked program.
    #[error("canonical Universe bootstrap module {0} has no linked module")]
    MissingBootstrapLinkedModule(ModuleId),
    /// The explicit eager bootstrap order must be a set-like ordering.
    #[error("canonical Universe bootstrap order contains duplicate module {0}")]
    DuplicateBootstrapModule(ModuleId),
}

static CANONICAL_UNIVERSE_PROGRAM: OnceLock<Result<CanonicalUniverseProgram, CanonicalUniverseBuildError>> = OnceLock::new();

#[cfg(test)]
pub(crate) static SOURCE_INDEX_BUILDS: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
pub(crate) static NATIVE_CONTRACT_VERIFICATIONS: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
pub(crate) static CANONICAL_LINKS: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
pub(crate) static CANONICAL_SEMANTIC_ANALYSES: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
pub(crate) static CANONICAL_PROGRAM_PROJECTIONS: AtomicUsize = AtomicUsize::new(0);

/// Returns the one immutable canonical Universe compiler product for this process.
pub(crate) fn canonical_universe_program() -> Result<&'static CanonicalUniverseProgram, &'static CanonicalUniverseBuildError> {
    match CANONICAL_UNIVERSE_PROGRAM.get_or_init(build_canonical_universe_program) {
        Ok(program) => Ok(program),
        Err(error) => Err(error),
    }
}

fn build_canonical_universe_program() -> Result<CanonicalUniverseProgram, CanonicalUniverseBuildError> {
    #[cfg(test)]
    SOURCE_INDEX_BUILDS.fetch_add(1, Ordering::Relaxed);
    let source_index = Arc::new(NativeSourceIndex::build().map_err(CanonicalUniverseBuildError::SourceIndex)?);
    let descriptors = PRIMITIVES.iter().collect::<Vec<_>>();
    #[cfg(test)]
    NATIVE_CONTRACT_VERIFICATIONS.fetch_add(1, Ordering::Relaxed);
    verify_native_contracts(&source_index, &descriptors).map_err(|error| CanonicalUniverseBuildError::NativeContracts(error.to_string()))?;

    let project_universe = Arc::new(ProjectUniverse::new());
    let filesystem = FilesystemSourceProvider::new();
    let mut resolver = ModuleResolver::new(&project_universe, &filesystem);
    let mut interfaces = BTreeMap::new();

    for unit in &source_index.units {
        let interface = resolver.load_interface(&unit.id).map_err(|error| CanonicalUniverseBuildError::InterfaceLoad {
            module: unit.id.clone(),
            error: error.to_string(),
        })?;
        interfaces.insert(unit.id.clone(), interface);
    }

    let mut resolved = BTreeMap::new();
    for (module, interface) in &interfaces {
        for import in &interface.imports {
            let path = match import {
                ImportSurface::Module(decl) => &decl.path,
                ImportSurface::Selective(decl) => &decl.path,
                ImportSurface::ReExport(decl) => &decl.path,
            };
            let target = resolver
                .resolve_import(module, path)
                .map_err(|error| CanonicalUniverseBuildError::ImportResolution {
                    module: module.clone(),
                    path: path.to_string(),
                    error: error.to_string(),
                })?;
            resolved.insert((module.clone(), path.to_string()), target.id);
        }
    }

    #[cfg(test)]
    CANONICAL_LINKS.fetch_add(1, Ordering::Relaxed);
    let linked = ModuleLinker::new(project_universe.clone(), interfaces)
        .link_all(ModuleId::universe_root(), &resolved)
        .map_err(|error| CanonicalUniverseBuildError::Linking(error.to_string()))?;
    let linked = Arc::new(linked);
    let sources: BTreeMap<ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>> =
        source_index.units.iter().map(|unit| (unit.id.clone(), unit.clone())).collect();
    #[cfg(test)]
    CANONICAL_SEMANTIC_ANALYSES.fetch_add(1, Ordering::Relaxed);
    let semantic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        phalcom_semantic::analyze_workspace(phalcom_semantic::SemanticWorkspaceInput::new(
            linked.clone(),
            sources.clone(),
            0,
        ))
    }))
    .map_err(|_| CanonicalUniverseBuildError::SemanticAnalysisPanicked)?;

    let analyzed = AnalyzedProgram {
        project_universe,
        linked,
        semantic: semantic.snapshot,
        sources,
        entry: ModuleId::universe_root(),
    };
    #[cfg(test)]
    CANONICAL_PROGRAM_PROJECTIONS.fetch_add(1, Ordering::Relaxed);
    let program =
        ProgramCompiler::compile_analyzed_for_canonical_bootstrap(&analyzed).map_err(|error| CanonicalUniverseBuildError::Compilation(error.to_string()))?;

    let root = ModuleId::universe_root();
    let root_reachable = source_index
        .reachable_units_from_roots(std::slice::from_ref(&root))
        .map_err(CanonicalUniverseBuildError::BootstrapPlanning)?;
    let bootstrap_roots = source_index.bootstrap_roots();
    let bootstrap_order = source_index
        .initialization_order_from_roots(&bootstrap_roots)
        .map_err(CanonicalUniverseBuildError::BootstrapPlanning)?
        .into_iter()
        .map(|unit| unit.id.clone())
        .collect::<Vec<_>>();

    validate_canonical_product(&source_index, &program, &root_reachable, &bootstrap_order)?;

    Ok(CanonicalUniverseProgram {
        program: Arc::new(program),
        source_index,
        root_reachable: Arc::from(root_reachable.into_boxed_slice()),
        bootstrap_order: Arc::from(bootstrap_order.into_boxed_slice()),
    })
}

fn validate_canonical_product(
    source_index: &NativeSourceIndex,
    program: &CompiledProgram,
    root_reachable: &[ModuleId],
    bootstrap_order: &[ModuleId],
) -> Result<(), CanonicalUniverseBuildError> {
    for id in root_reachable {
        if source_index.unit(id).is_none() {
            return Err(CanonicalUniverseBuildError::MissingReachableSource(id.clone()));
        }
        if !program.modules.contains_key(id) {
            return Err(CanonicalUniverseBuildError::MissingReachableCompiledModule(id.clone()));
        }
        if !program.linked.modules.contains_key(id) {
            return Err(CanonicalUniverseBuildError::MissingReachableLinkedModule(id.clone()));
        }
    }

    let mut seen = std::collections::BTreeSet::new();
    for id in bootstrap_order {
        if !seen.insert(id.clone()) {
            return Err(CanonicalUniverseBuildError::DuplicateBootstrapModule(id.clone()));
        }
        if source_index.unit(id).is_none() {
            return Err(CanonicalUniverseBuildError::MissingBootstrapSource(id.clone()));
        }
        if !program.modules.contains_key(id) {
            return Err(CanonicalUniverseBuildError::MissingBootstrapCompiledModule(id.clone()));
        }
        if !program.linked.modules.contains_key(id) {
            return Err(CanonicalUniverseBuildError::MissingBootstrapLinkedModule(id.clone()));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::canonical_universe_program;
    use phalcom_modules::{ModuleComponent, ModuleId, ModulePath};
    use phalcom_native_meta::UniverseKey;

    #[test]
    fn canonical_universe_program_is_process_singleton() {
        let first = canonical_universe_program().expect("canonical Universe compiler product");
        let second = canonical_universe_program().expect("canonical Universe compiler product");

        assert!(std::ptr::eq(first, second));
        assert!(std::sync::Arc::ptr_eq(&first.program, &second.program));
    }

    #[test]
    fn canonical_product_covers_every_bootstrap_module() {
        let canonical = canonical_universe_program().expect("canonical Universe compiler product");

        let bootstrap = canonical.bootstrap_order().iter().collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            bootstrap.len(),
            canonical.bootstrap_order().len(),
            "bootstrap order must not contain duplicates"
        );

        for id in canonical.bootstrap_order() {
            assert!(
                canonical.source_index().unit(id).is_some(),
                "bootstrap module {id} must have canonical parsed source"
            );
            assert!(
                canonical.program().modules.contains_key(id),
                "bootstrap module {id} must have compiled projection"
            );
            assert!(
                canonical.program().linked.modules.contains_key(id),
                "bootstrap module {id} must exist in linked program"
            );
        }
    }

    #[test]
    fn canonical_product_covers_every_root_reachable_module() {
        let canonical = canonical_universe_program().expect("canonical Universe compiler product");

        for id in canonical.root_reachable() {
            assert!(
                canonical.source_index().unit(id).is_some(),
                "root-reachable module {id} must have canonical parsed source"
            );
            assert!(
                canonical.program().modules.contains_key(id),
                "root-reachable module {id} must have compiled projection"
            );
            assert!(
                canonical.program().linked.modules.contains_key(id),
                "root-reachable module {id} must exist in linked program"
            );
        }
    }

    #[test]
    fn canonical_universe_program_preserves_result_identity() {
        let canonical = canonical_universe_program().expect("canonical Universe compiler product");
        let result_module = ModuleId::universe(ModulePath::from_components(
            UniverseKey::Result
                .source_path()
                .iter()
                .map(|component| ModuleComponent::from_identifier(component).expect("canonical module component"))
                .collect::<Vec<_>>(),
        ));
        let result_owner = phalcom_semantic::core_surface::universe_declaration(UniverseKey::Result);
        let result_lowering = &canonical.program.modules[&result_module].lowering;

        let result_enum = result_lowering
            .enums
            .iter()
            .find(|enum_lowering| enum_lowering.owner == result_owner)
            .expect("canonical Result enum lowering");
        assert!(result_enum.variants.iter().all(|variant| variant.id.owner == result_owner));
    }

    #[test]
    fn canonical_universe_program_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<super::CanonicalUniverseProgram>();
    }
}
