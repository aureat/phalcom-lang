//! Process-shared compiler product for the canonical source-authored Universe.
//!
//! This module owns only immutable, VM-independent products. Runtime
//! materialization, bytecode compilation, and source execution remain local to
//! each [`crate::vm::VM`].

use crate::modules::{AnalyzedProgram, CompiledProgram, ProgramCompiler};
use crate::native::{NativeSourceIndex, PRIMITIVES, verify_native_contracts};
use phalcom_common::range::SourceRange;
use phalcom_modules::{
    AvailabilityAuthority, AvailabilityContract, AvailabilityProof, FilesystemSourceProvider, ImportSurface, ModuleComponent, ModuleId, ModuleLinker,
    ModulePath, ModuleResolver, ProjectUniverse, RuntimeRequirementProduct, RuntimeRequirementTarget, RuntimeRequirementTiming,
};
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
    /// Availability guaranteed after the native installation floor.
    #[cfg(test)]
    native_floor_availability: AvailabilityContract,
    /// Exact products guaranteed after canonical Universe bootstrap.
    availability_after_bootstrap: AvailabilityContract,
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

    pub(crate) fn availability_after_bootstrap(&self) -> &AvailabilityContract {
        &self.availability_after_bootstrap
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
    /// Exact availability proof disagreed with the canonical installation path.
    #[error("canonical availability contract failed: {0}")]
    AvailabilityContract(String),
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
        linker
            .link_semantic(ModuleId::universe_root(), &resolved)
            .map_err(|error| CanonicalUniverseBuildError::Linking(error.to_string()))?,
    );
    let linked_for_semantics = symbolic.to_unfinalized_program();
    let sources: BTreeMap<ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>> =
        source_index.units.iter().map(|unit| (unit.id.clone(), unit.clone())).collect();
    #[cfg(test)]
    CANONICAL_SEMANTIC_ANALYSES.fetch_add(1, Ordering::Relaxed);
    let semantic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        phalcom_semantic::try_analyze_workspace(
            phalcom_semantic::SemanticWorkspaceInput::new(linked_for_semantics.clone(), sources.clone(), 0).with_semantic_linked(symbolic.clone()),
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
    requirements.extend(crate::modules::compile::runtime_metadata_type_requirements(
        &semantic.snapshot,
        &sources,
        &runtime_modules,
    ));
    let native_floor_availability = native_floor_availability(&symbolic)?;
    let linked = Arc::new(
        linker
            .finalize_executable_with_availability(
                (*symbolic).clone(),
                &semantic.snapshot.import_phase_products,
                &requirements,
                &native_floor_availability,
            )
            .map_err(|error| CanonicalUniverseBuildError::Linking(error.to_string()))?,
    );

    let analyzed = AnalyzedProgram {
        compile_mode: crate::compiler::attributes::CompileMode::Debug,
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
    let authored_bootstrap_order = source_index
        .initialization_order_from_roots(&bootstrap_roots)
        .map_err(CanonicalUniverseBuildError::BootstrapPlanning)?
        .into_iter()
        .map(|unit| unit.id.clone())
        .collect::<Vec<_>>();
    // Module discovery owns the source initializer closure, while the
    // finalized runtime DAG owns dependency-first execution constraints.
    // Preserve the old authored order wherever runtime dependencies permit.
    let bootstrap_order = align_bootstrap_order_with_runtime_dependencies(&program.linked.as_program().graphs.runtime, &authored_bootstrap_order)?;

    validate_canonical_product(&source_index, &program, &root_reachable, &bootstrap_order)?;

    // Only modules whose source initializers execute can publish source-value
    // availability, even when other module identities are linked/catalogued.
    let availability_after_bootstrap = fully_initialized_universe_availability(&native_floor_availability, &program, &bootstrap_order);

    Ok(CanonicalUniverseProgram {
        program: Arc::new(program),
        source_index,
        root_reachable: Arc::from(root_reachable.into_boxed_slice()),
        bootstrap_order: Arc::from(bootstrap_order.into_boxed_slice()),
        #[cfg(test)]
        native_floor_availability,
        availability_after_bootstrap,
    })
}

/// The native materializer creates every canonical Universe module before any
/// source initializer. Only bindings listed by `UNIVERSE_BINDINGS` receive a
/// native-floor runtime-value proof; semantic declaration identities are
/// recorded separately and do not assert that reserved source-only slots are
/// populated.
fn native_floor_availability(symbolic: &phalcom_modules::SemanticLinkedProgram) -> Result<AvailabilityContract, CanonicalUniverseBuildError> {
    let before_source = RuntimeRequirementTiming::BeforeSourceExecution;
    let mut proofs = Vec::new();
    for (module, linked) in &symbolic.modules {
        proofs.push(AvailabilityProof {
            target: RuntimeRequirementTarget::Module(module.clone()),
            product: RuntimeRequirementProduct::RuntimeObject,
            authority: AvailabilityAuthority::NativeModuleMaterialization,
            available_at: before_source,
        });
        proofs.push(AvailabilityProof {
            target: RuntimeRequirementTarget::Module(module.clone()),
            product: RuntimeRequirementProduct::CanonicalIdentity,
            authority: AvailabilityAuthority::SemanticIdentityCatalog,
            available_at: before_source,
        });
        for name in linked.local_globals.keys() {
            proofs.push(AvailabilityProof {
                target: RuntimeRequirementTarget::Binding {
                    module: module.clone(),
                    name: name.clone(),
                },
                product: RuntimeRequirementProduct::CanonicalIdentity,
                authority: AvailabilityAuthority::SemanticIdentityCatalog,
                available_at: before_source,
            });
        }
    }
    for binding in phalcom_native_meta::UNIVERSE_BINDINGS {
        let module = universe_module_id(binding.key)?;
        proofs.push(AvailabilityProof {
            target: RuntimeRequirementTarget::Binding {
                module,
                name: binding.name.into(),
            },
            product: RuntimeRequirementProduct::RuntimeObject,
            authority: AvailabilityAuthority::NativeBindingCatalog,
            available_at: before_source,
        });
    }
    Ok(AvailabilityContract::from_proofs(proofs))
}

/// Source-initialized values are available only for modules whose source
/// initializers the canonical bootstrap actually executes. Discovery/linking
/// alone cannot certify that a runtime binding was populated.
fn fully_initialized_universe_availability(
    native_floor: &AvailabilityContract,
    program: &crate::modules::CompiledProgram,
    executed_modules: &[ModuleId],
) -> AvailabilityContract {
    let mut proofs = native_floor.proofs().cloned().collect::<Vec<_>>();
    let executed = executed_modules.iter().cloned().collect::<std::collections::BTreeSet<_>>();
    for (module, linked) in &program.linked.modules {
        if !module.project.is_universe() || !executed.contains(module) {
            continue;
        }
        proofs.push(AvailabilityProof {
            target: RuntimeRequirementTarget::Module(module.clone()),
            product: RuntimeRequirementProduct::RuntimeObject,
            authority: AvailabilityAuthority::FullyInitializedUniverse,
            available_at: RuntimeRequirementTiming::AfterBootstrap,
        });
        proofs.push(AvailabilityProof {
            target: RuntimeRequirementTarget::Module(module.clone()),
            product: RuntimeRequirementProduct::SourceInitializedValue,
            authority: AvailabilityAuthority::FullyInitializedUniverse,
            available_at: RuntimeRequirementTiming::AfterBootstrap,
        });
        for name in linked.bindings.local_globals.keys() {
            proofs.push(AvailabilityProof {
                target: RuntimeRequirementTarget::Binding {
                    module: module.clone(),
                    name: name.clone(),
                },
                product: RuntimeRequirementProduct::SourceInitializedValue,
                authority: AvailabilityAuthority::FullyInitializedUniverse,
                available_at: RuntimeRequirementTiming::AfterBootstrap,
            });
        }
    }
    if let Some(bundle) = &program.semantic_metadata {
        for root in bundle.module_roots.iter() {
            if let Some(module) = runtime_module_from_stable_ref(&root.module) {
                proofs.push(AvailabilityProof {
                    target: RuntimeRequirementTarget::Module(module),
                    product: RuntimeRequirementProduct::MetadataDescriptor,
                    authority: AvailabilityAuthority::FullyInitializedUniverse,
                    available_at: RuntimeRequirementTiming::AfterBootstrap,
                });
            }
        }
    }
    AvailabilityContract::from_proofs(proofs)
}

fn universe_module_id(key: phalcom_native_meta::UniverseKey) -> Result<ModuleId, CanonicalUniverseBuildError> {
    let components = key
        .source_path()
        .iter()
        .map(|component| ModuleComponent::from_identifier(component).map_err(|error| CanonicalUniverseBuildError::AvailabilityContract(error.to_string())))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ModuleId::universe(ModulePath::from_components(components)))
}

fn runtime_module_from_stable_ref(reference: &phalcom_type_meta::identity::StableModuleRef) -> Option<ModuleId> {
    match &reference.project {
        phalcom_type_meta::identity::StableProjectRef::Builtin { namespace, .. } if namespace.as_ref() == "universe" => {
            let components = reference
                .path
                .iter()
                .map(|component| ModuleComponent::from_identifier(component).ok())
                .collect::<Option<Vec<_>>>()?;
            Some(ModuleId::universe(ModulePath::from_components(components)))
        }
        _ => None,
    }
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

    // Only executed source initializers participate in this schedule check.
    // Dependencies satisfied by prior native-floor availability are absent
    // from the finalized graph, and nonexecuted targets have separate owners.
    validate_bootstrap_runtime_order(&program.linked.as_program().graphs.runtime, bootstrap_order)?;

    Ok(())
}

/// Stably topologically order the selected source initializer closure
/// against *effective*, finalized runtime dependencies. The source index
/// determines which modules execute; its authored-import-only order is not
/// authoritative when semantic-generated reads introduce additional edges.
/// Use the authored order as a deterministic tie-breaker to avoid reordering
/// unrelated source initializers.
fn align_bootstrap_order_with_runtime_dependencies(
    graph: &phalcom_modules::RuntimeDependencyGraph,
    authored_order: &[ModuleId],
) -> Result<Vec<ModuleId>, CanonicalUniverseBuildError> {
    let positions = authored_order
        .iter()
        .enumerate()
        .map(|(index, module)| (module.clone(), index))
        .collect::<BTreeMap<_, _>>();
    if positions.len() != authored_order.len() {
        let mut seen = std::collections::BTreeSet::new();
        let repeated = authored_order
            .iter()
            .find(|module| !seen.insert((*module).clone()))
            .expect("non-unique source order has a duplicate");
        return Err(CanonicalUniverseBuildError::DuplicateBootstrapModule(repeated.clone()));
    }

    let mut remaining = positions.keys().cloned().map(|module| (module, 0usize)).collect::<BTreeMap<_, _>>();
    let mut dependents = BTreeMap::<ModuleId, std::collections::BTreeSet<ModuleId>>::new();
    for importer in authored_order {
        for dependency in graph.dependencies(importer) {
            if !positions.contains_key(&dependency) {
                // Native-floor and other nonexecuted targets have a separate
                // availability owner, rather than a source initializer slot.
                continue;
            }
            *remaining.get_mut(importer).expect("source importer is in closure") += 1;
            dependents.entry(dependency).or_default().insert(importer.clone());
        }
    }

    let mut ready = remaining
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(module, _)| (positions[module], module.clone()))
        .collect::<std::collections::BTreeSet<_>>();
    let mut result = Vec::with_capacity(authored_order.len());
    while let Some((_, module)) = ready.pop_first() {
        if let Some(affected) = dependents.get(&module) {
            for importer in affected {
                let count = remaining.get_mut(importer).expect("dependent importer is in closure");
                *count -= 1;
                if *count == 0 {
                    ready.insert((positions[importer], importer.clone()));
                }
            }
        }
        result.push(module);
    }

    if result.len() != authored_order.len() {
        return Err(CanonicalUniverseBuildError::BootstrapPlanning(
            "effective finalized runtime dependencies are cyclic within canonical source execution closure".into(),
        ));
    }

    Ok(result)
}

/// Validate actual runtime edges against the VM's source-execution schedule.
/// Linker acyclicity alone does not prove that the source-index order honors
/// a generated, unsatisfied runtime requirement.
fn validate_bootstrap_runtime_order(graph: &phalcom_modules::RuntimeDependencyGraph, bootstrap_order: &[ModuleId]) -> Result<(), CanonicalUniverseBuildError> {
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
    use phalcom_modules::{
        AvailabilityAuthority, ModuleComponent, ModuleId, ModulePath, RuntimeRequirementProduct, RuntimeRequirementTarget, RuntimeRequirementTiming,
    };
    use phalcom_native_meta::UniverseKey;

    #[test]
    fn finalized_bootstrap_order_respects_generated_edges_and_stable_authored_ties() {
        use phalcom_modules::{RuntimeDependencyEdge, RuntimeDependencyGraph, RuntimeDependencyReason};

        let importer = ModuleId::universe_root();
        let dependency = ModuleId::universe(ModulePath::from_components(vec![
            ModuleComponent::from_identifier("errors").expect("module component"),
        ]));
        let independent = ModuleId::universe(ModulePath::from_components(vec![
            ModuleComponent::from_identifier("scalar").expect("module component"),
        ]));
        let mut graph = RuntimeDependencyGraph::default();
        graph.add(RuntimeDependencyEdge {
            importer: importer.clone(),
            dependency: dependency.clone(),
            range: Default::default(),
            reason: RuntimeDependencyReason::RuntimeDeclarationReference,
            provenance: None,
        });

        let aligned = super::align_bootstrap_order_with_runtime_dependencies(&graph, &[independent.clone(), importer.clone(), dependency.clone()])
            .expect("acyclic finalized dependencies can be honored by reordering");
        assert_eq!(aligned, [independent, dependency, importer]);
        super::validate_bootstrap_runtime_order(&graph, &aligned).expect("reconciled schedule honors finalized dependencies");
    }

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
            provenance: None,
        });

        assert!(matches!(
            super::validate_bootstrap_runtime_order(&graph, &[importer.clone(), dependency.clone()]),
            Err(CanonicalUniverseBuildError::BootstrapRuntimeDependencyOrder {
                importer: actual_importer, dependency: actual_dependency
            }) if actual_importer == importer && actual_dependency == dependency
        ));
        super::validate_bootstrap_runtime_order(&graph, &[dependency, importer]).expect("dependency-first order must pass");
    }

    #[test]
    fn finalized_bootstrap_order_ignores_nonexecuted_dependency_targets() {
        use phalcom_modules::{RuntimeDependencyEdge, RuntimeDependencyGraph, RuntimeDependencyReason};

        let importer = ModuleId::universe_root();
        let dependency = ModuleId::universe(ModulePath::from_components(vec![
            ModuleComponent::from_identifier("errors").expect("module component"),
        ]));
        let mut graph = RuntimeDependencyGraph::default();
        graph.add(RuntimeDependencyEdge {
            importer: importer.clone(),
            dependency,
            range: Default::default(),
            reason: RuntimeDependencyReason::RuntimeDeclarationReference,
            provenance: None,
        });

        super::validate_bootstrap_runtime_order(&graph, &[importer]).expect("nonexecuted target is not part of this source execution schedule");
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
    fn unexecuted_universe_modules_cannot_publish_source_initialized_value_proofs() {
        let canonical = canonical_universe_program().expect("canonical Universe compiler product");

        let empty_schedule = super::fully_initialized_universe_availability(&canonical.native_floor_availability, canonical.program(), &[]);
        assert!(
            !empty_schedule.proofs().any(|proof| {
                proof.authority == AvailabilityAuthority::FullyInitializedUniverse && proof.product == RuntimeRequirementProduct::SourceInitializedValue
            }),
            "linking a module must not assert that its source initializer executed"
        );

        let executed = canonical.bootstrap_order().iter().cloned().collect::<std::collections::BTreeSet<_>>();
        let source_values = canonical
            .availability_after_bootstrap()
            .proofs()
            .filter(|proof| {
                proof.authority == AvailabilityAuthority::FullyInitializedUniverse && proof.product == RuntimeRequirementProduct::SourceInitializedValue
            })
            .collect::<Vec<_>>();
        assert!(!source_values.is_empty(), "executed canonical modules publish source values");
        for proof in source_values {
            assert!(
                executed.contains(proof.target.module()),
                "module {} has a source-value proof without an executed initializer",
                proof.target.module()
            );
        }
    }

    #[test]
    fn native_floor_contract_matches_fresh_vm_values_and_excludes_source_only_slots() {
        let canonical = canonical_universe_program().expect("canonical Universe compiler product");
        let mut vm = crate::vm::VM::new_native();

        for binding in phalcom_native_meta::UNIVERSE_BINDINGS {
            let module = super::universe_module_id(binding.key).expect("catalog module path");
            let module_object = vm.module_registry.get(&module).expect("native floor module materialized").object;
            let name = vm.interner.intern(binding.name);
            let value = vm
                .heap
                .module(module_object)
                .get(name)
                .expect("catalog binding populated before source execution");
            if binding.key == UniverseKey::None {
                assert_eq!(value, crate::value::Value::none());
            } else {
                let expected = crate::value::Value::obj(vm.universe.classes.resolve(binding.key));
                assert!(value.same_as(&expected), "{} must retain its canonical class identity", binding.name);
            }
            assert!(canonical.native_floor_availability.proofs().any(|proof| {
                proof.target
                    == RuntimeRequirementTarget::Binding {
                        module: module.clone(),
                        name: binding.name.into(),
                    }
                    && proof.product == RuntimeRequirementProduct::RuntimeObject
                    && proof.authority == AvailabilityAuthority::NativeBindingCatalog
                    && proof.available_at == RuntimeRequirementTiming::BeforeSourceExecution
            }));
        }

        for class in canonical.source_index().census.classes.iter().filter(|class| class.universe_key.is_none()) {
            let module_object = vm
                .module_registry
                .get(&class.module)
                .expect("source-only module object exists at native floor")
                .object;
            let name = vm.interner.intern(&class.name);
            assert!(
                vm.heap.module(module_object).get(name).is_none(),
                "reserved source-only binding {}::{} is not an initialized native-floor value",
                class.module,
                class.name
            );
            assert!(!canonical.native_floor_availability.proofs().any(|proof| {
                proof.target
                    == RuntimeRequirementTarget::Binding {
                        module: class.module.clone(),
                        name: class.name.clone().into_boxed_str(),
                    }
                    && matches!(
                        proof.product,
                        RuntimeRequirementProduct::RuntimeObject | RuntimeRequirementProduct::SourceInitializedValue
                    )
            }));
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
