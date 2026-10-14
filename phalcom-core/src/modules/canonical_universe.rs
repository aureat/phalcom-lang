//! Process-shared compiler product for the canonical source-authored Universe.
//!
//! This module owns only immutable, VM-independent products. Runtime
//! materialization, bytecode compilation, and source execution remain local to
//! each [`crate::vm::VM`].

use crate::modules::{AnalyzedProgram, CompiledProgram, ProgramCompiler};
use crate::native::{NativeSourceIndex, PRIMITIVES, verify_native_contracts};
use phalcom_common::range::SourceRange;
use phalcom_modules::{FilesystemSourceProvider, ImportSurface, ModuleId, ModuleLinker, ModuleResolver, ProjectUniverse};
use phalcom_semantic::identity::{CallableId, ExpressionId};
use phalcom_semantic::match_semantics::ExhaustivenessResult;
use phalcom_semantic::snapshot::SemanticSnapshot;
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
    /// Canonical semantic analysis failed without publishing a snapshot.
    #[error("canonical Universe semantic analysis failed: {0}")]
    SemanticAnalysisFailed(#[from] phalcom_semantic::SemanticUpdateFailure),
    /// A semantic match product lacks the proof required for executable lowering.
    #[error("canonical Universe semantic product is not executable: match in {module} ({callable:?}) at {range:?} is {reason}")]
    NonExecutableMatch {
        module: Box<ModuleId>,
        callable: Box<CallableId>,
        expression: ExpressionId,
        range: SourceRange,
        reason: Box<str>,
    },
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
    /// A finalized runtime dependency would initialize after its importer.
    #[error("canonical Universe bootstrap order initializes {importer} before runtime dependency {dependency}")]
    BootstrapRuntimeDependencyOrder { importer: ModuleId, dependency: ModuleId },
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
    #[cfg(any(test, feature = "test-timings"))]
    crate::instrumentation::record_canonical_universe_product_build();
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
    let linker = ModuleLinker::new(project_universe.clone(), interfaces);
    let symbolic = Arc::new(
        linker.link_semantic(ModuleId::universe_root(), &resolved)
            .map_err(|error| CanonicalUniverseBuildError::Linking(error.to_string()))?,
    );
    let linked_for_semantics = symbolic.to_unfinalized_program();
    let sources: BTreeMap<ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>> =
        source_index.units.iter().map(|unit| (unit.id.clone(), unit.clone())).collect();
    #[cfg(test)]
    CANONICAL_SEMANTIC_ANALYSES.fetch_add(1, Ordering::Relaxed);
    let semantic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        phalcom_semantic::try_analyze_workspace(
            phalcom_semantic::SemanticWorkspaceInput::new(linked_for_semantics.clone(), sources.clone(), 0)
                .with_semantic_linked(symbolic.clone()),
        )
    }))
    .map_err(|_| CanonicalUniverseBuildError::SemanticAnalysisPanicked)?;
    let semantic = semantic?;

    if let Some(diagnostic) = semantic
        .snapshot
        .diagnostics
        .values()
        .flat_map(|diagnostics| diagnostics.iter())
        .find(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::NativeSignatureMismatch)
    {
        return Err(CanonicalUniverseBuildError::NativeContracts(diagnostic.message.clone()));
    }
    ensure_executable_match_coverage(&semantic.snapshot)?;

    let runtime_modules = symbolic.modules.keys().cloned().collect();
    let mut requirements = semantic.snapshot.generated_runtime_requirements.to_vec();
    let semantic_requirement_count = requirements.len();
    requirements.extend(crate::modules::compile::runtime_metadata_type_requirements(&semantic.snapshot, &sources, &runtime_modules));
    let linked = Arc::new(
        linker.finalize_executable_with_requirements((*symbolic).clone(), &semantic.snapshot.import_phase_products, &requirements)
            .map_err(|error| {
                let detail = canonical_runtime_cycle_requirements(
                    &error,
                    &requirements[..semantic_requirement_count],
                    &requirements[semantic_requirement_count..],
                );
                CanonicalUniverseBuildError::Linking(format!("{error}{detail}"))
            })?,
    );

    let analyzed = AnalyzedProgram {
        project_universe,
        runtime_modules: linked.modules.keys().cloned().collect(),
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

/// Reports generated requirements within a rejected runtime SCC, grouped by
/// the producing stage. The linker still owns edge inclusion and DAG rejection;
/// this diagnostic neither modifies the edge set nor guesses native-floor
/// availability. Authored imports are represented by the original cycle error
/// but their individual ranges are not available here.
fn canonical_runtime_cycle_requirements(
    error: &phalcom_modules::LinkError,
    semantic_requirements: &[phalcom_modules::RuntimeDependencyRequirement],
    metadata_requirements: &[phalcom_modules::RuntimeDependencyRequirement],
) -> String {
    let phalcom_modules::LinkError::RuntimeCycle(
        phalcom_modules::ModuleGraphError::RuntimeCycle { cycle },
    ) = error else {
        return String::new();
    };
    let members = cycle.iter().collect::<std::collections::BTreeSet<_>>();
    let mut edges = [
        ("semantic-generated", semantic_requirements),
        ("metadata-type-owner", metadata_requirements),
    ]
    .into_iter()
    .flat_map(|(origin, requirements)| {
        requirements
            .iter()
            .filter(|requirement| members.contains(&requirement.importer) && members.contains(&requirement.dependency))
            .map(move |requirement| {
                format!(
                    "  [{origin}] {} -> {} at {:?} ({:?}, linked read: {:?})",
                    requirement.importer,
                    requirement.dependency,
                    requirement.range,
                    requirement.reason,
                    requirement.read_target
                )
            })
    })
    .collect::<Vec<_>>();
    edges.sort();
    edges.dedup();
    if edges.is_empty() {
        return "\nno generated requirement edges inside the reported cycle; inspect authored imports".into();
    }
    const MAX_DISPLAY_EDGES: usize = 128;
    let omitted = edges.len().saturating_sub(MAX_DISPLAY_EDGES);
    edges.truncate(MAX_DISPLAY_EDGES);
    let omitted_note = if omitted == 0 {
        String::new()
    } else {
        format!("\n  ... {omitted} additional generated requirement edges omitted")
    };
    format!(
        "\ngenerated requirements inside rejected runtime SCC:\n{}{}",
        edges.join("\n"),
        omitted_note
    )
}

/// Ensures the canonical product never asks executable lowering to interpret a
/// match whose semantic coverage proof is absent or blocked.
///
/// Canonical bootstrap deliberately retains a narrow diagnostics baseline for
/// source/native migration work. That tolerance must not extend to executable
/// semantic products: backend match lowering consumes only a proven coverage
/// product and remains fail-closed as a defense in depth.
fn ensure_executable_match_coverage(snapshot: &SemanticSnapshot) -> Result<(), CanonicalUniverseBuildError> {
    let mut failures = Vec::new();

    for analysis in snapshot.callable_analyses.values() {
        for resolution in analysis.match_resolutions.values() {
            let reason = match &resolution.exhaustiveness {
                ExhaustivenessResult::Proven => continue,
                ExhaustivenessResult::Missing(witnesses) => {
                    format!("missing coverage proof ({} witness(es))", witnesses.len())
                }
                ExhaustivenessResult::Blocked(reason) => format!("blocked coverage proof: {reason:?}"),
            };
            let range = analysis
                .expressions
                .get(&resolution.expression)
                .map(|expression| expression.range)
                .unwrap_or(analysis.body_range);
            failures.push((
                analysis.callable.module().clone(),
                analysis.callable.clone(),
                resolution.expression,
                range,
                reason,
            ));
        }
    }

    failures.sort_by(|left, right| (&left.0, &left.1, left.3.start, left.3.end, &left.4).cmp(&(&right.0, &right.1, right.3.start, right.3.end, &right.4)));
    if let Some((module, callable, expression, range, reason)) = failures.into_iter().next() {
        return Err(CanonicalUniverseBuildError::NonExecutableMatch {
            module: Box::new(module),
            callable: Box::new(callable),
            expression,
            range,
            reason: reason.into_boxed_str(),
        });
    }

    Ok(())
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

    // Only actual initializers participate in this order check: dependency
    // targets outside this bootstrap closure are validated by the appropriate
    // materialization or native-floor owner, not by this execution schedule.
    validate_bootstrap_runtime_order(&program.linked.as_program().graphs.runtime, bootstrap_order)?;

    Ok(())
}

/// Confirms the source-index schedule respects all finalized runtime edges
/// between source modules whose initializers this VM will execute.
fn validate_bootstrap_runtime_order(
    graph: &phalcom_modules::RuntimeDependencyGraph,
    bootstrap_order: &[ModuleId],
) -> Result<(), CanonicalUniverseBuildError> {
    let positions = bootstrap_order
        .iter()
        .enumerate()
        .map(|(index, module)| (module.clone(), index))
        .collect::<BTreeMap<_, _>>();
    for importer in bootstrap_order {
        for dependency in graph.dependencies(importer) {
            if let Some(&dependency_position) = positions.get(&dependency) {
                if dependency_position >= positions[importer] {
                    return Err(CanonicalUniverseBuildError::BootstrapRuntimeDependencyOrder {
                        importer: importer.clone(),
                        dependency,
                    });
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CanonicalUniverseBuildError, canonical_universe_program};
    use phalcom_modules::{ModuleComponent, ModuleId, ModulePath};
    use phalcom_native_meta::UniverseKey;

    #[test]
    fn finalized_bootstrap_order_rejects_reversed_generated_dependency() {
        use phalcom_modules::{RuntimeDependencyEdge, RuntimeDependencyGraph, RuntimeDependencyReason};

        let importer = ModuleId::universe_root();
        let dependency = ModuleId::universe(ModulePath::from_components(vec![
            ModuleComponent::from_identifier("errors").expect("module component"),
        ]));
        let mut graph = RuntimeDependencyGraph::default();
        graph.add(RuntimeDependencyEdge {
            importer: importer.clone(),
            dependency: dependency.clone(),
            range: Default::default(),
            reason: RuntimeDependencyReason::RuntimeDeclarationReference,
        });

        assert!(matches!(
            super::validate_bootstrap_runtime_order(&graph, &[importer.clone(), dependency.clone()]),
            Err(CanonicalUniverseBuildError::BootstrapRuntimeDependencyOrder {
                importer: actual_importer, dependency: actual_dependency
            }) if actual_importer == importer && actual_dependency == dependency
        ));
        super::validate_bootstrap_runtime_order(&graph, &[dependency, importer])
            .expect("dependency-first order must pass");
    }

    #[test]
    fn finalized_bootstrap_order_ignores_nonexecuted_dependency_targets() {
        use phalcom_modules::{RuntimeDependencyEdge, RuntimeDependencyGraph, RuntimeDependencyReason};

        let importer = ModuleId::universe_root();
        let external = ModuleId::universe(ModulePath::from_components(vec![
            ModuleComponent::from_identifier("errors").expect("module component"),
        ]));
        let mut graph = RuntimeDependencyGraph::default();
        graph.add(RuntimeDependencyEdge {
            importer: importer.clone(),
            dependency: external,
            range: Default::default(),
            reason: RuntimeDependencyReason::RuntimeDeclarationReference,
        });
        super::validate_bootstrap_runtime_order(&graph, &[importer])
            .expect("this check covers only the executed source initializer schedule");
    }

    #[test]
    fn canonical_cycle_diagnostic_preserves_generated_requirement_provenance() {
        use phalcom_modules::{LinkError, LinkedReadSpec, ModuleGraphError, RuntimeDependencyReason, RuntimeDependencyRequirement, SymbolId};

        let a = ModuleId::universe_root();
        let b = ModuleId::universe(ModulePath::from_components(vec![
            ModuleComponent::from_identifier("errors").expect("module component"),
        ]));
        let requirement = RuntimeDependencyRequirement {
            importer: a.clone(),
            dependency: b.clone(),
            range: Default::default(),
            reason: RuntimeDependencyReason::RuntimeDeclarationReference,
            read_target: Some(LinkedReadSpec::Binding(SymbolId {
                module: b.clone(),
                name: "Error".into(),
            })),
        };
        let error = LinkError::RuntimeCycle(ModuleGraphError::RuntimeCycle {
            cycle: vec![a, b],
        });
        let detail = super::canonical_runtime_cycle_requirements(&error, &[requirement], &[]);
        assert!(detail.contains("generated requirements inside rejected runtime SCC"));
        assert!(detail.contains("Error"));
        assert!(detail.contains("RuntimeDeclarationReference"));
        assert!(detail.contains("[semantic-generated]"));
        assert!(detail.contains("linked read: Some("));
        assert!(super::canonical_runtime_cycle_requirements(
            &LinkError::RuntimeCycle(ModuleGraphError::RuntimeCycle { cycle: vec![] }),
            &[],
            &[]
        ).contains("no generated requirement edges"));
    }

    #[test]
    fn canonical_cycle_diagnostic_distinguishes_metadata_owner_requirements() {
        use phalcom_modules::{LinkError, ModuleGraphError, RuntimeDependencyReason, RuntimeDependencyRequirement};

        let importer = ModuleId::universe_root();
        let dependency = ModuleId::universe(ModulePath::from_components(vec![
            ModuleComponent::from_identifier("errors").expect("module component"),
        ]));
        let requirement = RuntimeDependencyRequirement {
            importer: importer.clone(),
            dependency: dependency.clone(),
            range: Default::default(),
            reason: RuntimeDependencyReason::RuntimeDeclarationReference,
            read_target: None,
        };
        let error = LinkError::RuntimeCycle(ModuleGraphError::RuntimeCycle {
            cycle: vec![importer, dependency],
        });
        let detail = super::canonical_runtime_cycle_requirements(&error, &[], &[requirement]);
        assert!(detail.contains("[metadata-type-owner]"));
        assert!(detail.contains("linked read: None"));
    }

    #[test]
    fn canonical_semantic_failure_is_distinct_from_panic() {
        let error: CanonicalUniverseBuildError = phalcom_semantic::SemanticUpdateFailure::Failed("injected update failure".into()).into();
        assert!(
            matches!(error, CanonicalUniverseBuildError::SemanticAnalysisFailed(phalcom_semantic::SemanticUpdateFailure::Failed(message)) if message == "injected update failure")
        );
    }

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
