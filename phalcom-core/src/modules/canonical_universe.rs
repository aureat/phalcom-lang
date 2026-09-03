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

/// Immutable compiler-side representation of the canonical source Universe.
///
/// The product may be shared between VMs because it contains no heap handles,
/// runtime classes, closures, chunks, or mutable runtime registries.
#[derive(Clone, Debug)]
pub struct CanonicalUniverseProgram {
    /// VM-independent linked and lowered module program.
    pub program: Arc<CompiledProgram>,
    /// Canonical parsed source units used by each VM's local compiler.
    pub source_index: Arc<NativeSourceIndex>,
    /// Modules reachable from the canonical Universe root.
    pub root_reachable: Arc<[ModuleId]>,
    /// Existing eager source-bootstrap order, retained without recomputation.
    pub bootstrap_order: Arc<[ModuleId]>,
}

/// Failure while deriving the process-shared canonical compiler product.
#[derive(Debug, Error)]
pub enum CanonicalUniverseBuildError {
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
}

static CANONICAL_UNIVERSE_PROGRAM: OnceLock<Result<CanonicalUniverseProgram, CanonicalUniverseBuildError>> = OnceLock::new();

/// Returns the one immutable canonical Universe compiler product for this process.
pub fn canonical_universe_program() -> Result<&'static CanonicalUniverseProgram, &'static CanonicalUniverseBuildError> {
    match CANONICAL_UNIVERSE_PROGRAM.get_or_init(build_canonical_universe_program) {
        Ok(program) => Ok(program),
        Err(error) => Err(error),
    }
}

fn build_canonical_universe_program() -> Result<CanonicalUniverseProgram, CanonicalUniverseBuildError> {
    let source_index = Arc::new(NativeSourceIndex::build().map_err(CanonicalUniverseBuildError::SourceIndex)?);
    let descriptors = PRIMITIVES.iter().collect::<Vec<_>>();
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

    let linked = ModuleLinker::new(project_universe.clone(), interfaces)
        .link_all(ModuleId::universe_root(), &resolved)
        .map_err(|error| CanonicalUniverseBuildError::Linking(error.to_string()))?;
    let linked = Arc::new(linked);
    let sources: BTreeMap<ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>> =
        source_index.units.iter().map(|unit| (unit.id.clone(), unit.clone())).collect();
    let semantic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        phalcom_semantic::analyze_workspace(phalcom_semantic::SemanticWorkspaceInput {
            linked: linked.clone(),
            sources: sources.clone(),
            generation: 0,
        })
    }))
    .map_err(|_| CanonicalUniverseBuildError::SemanticAnalysisPanicked)?;

    let analyzed = AnalyzedProgram {
        project_universe,
        linked,
        semantic: semantic.snapshot,
        sources,
        entry: ModuleId::universe_root(),
    };
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

    Ok(CanonicalUniverseProgram {
        program: Arc::new(program),
        source_index,
        root_reachable: Arc::from(root_reachable.into_boxed_slice()),
        bootstrap_order: Arc::from(bootstrap_order.into_boxed_slice()),
    })
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
