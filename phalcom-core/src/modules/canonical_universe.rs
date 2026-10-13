//! Process-shared compiler product for the canonical source-authored Universe.
//!
//! This module owns only immutable, VM-independent products. Runtime
//! materialization, bytecode compilation, and source execution remain local to
//! each [`crate::vm::VM`].

use crate::modules::{AnalyzedProgram, CompiledProgram, ProgramCompiler};
use crate::native::{NativeSourceIndex, PRIMITIVES, verify_native_contracts};
use phalcom_common::range::SourceRange;
use phalcom_modules::ModuleId;
use phalcom_semantic::identity::{CallableId, ExpressionId};
use phalcom_semantic::match_semantics::ExhaustivenessResult;
use phalcom_semantic::snapshot::SemanticSnapshot;
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
    /// Canonical semantic analysis panicked at its existing boundary.
    #[error("canonical Universe semantic analysis panicked")]
    SemanticAnalysisPanicked,
    /// Canonical semantic analysis failed without publishing a snapshot.
    #[error("canonical Universe semantic analysis failed: {0}")]
    SemanticAnalysisFailed(#[from] phalcom_semantic::SemanticUpdateFailure),
    /// Semantic-owned immutable image acquisition failed.
    #[error("canonical Universe formal image failed: {0}")]
    FormalImage(#[from] phalcom_semantic::canonical_image::CanonicalImageError),
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
    #[cfg(feature = "perf-observability")]
    if CANONICAL_UNIVERSE_PROGRAM.get().is_some() {
        crate::perf_counter!(telos_observability::PerfCounter::CanonicalUniverseCacheHits, 1);
    }

    crate::perf_span_result!(
        telos_observability::PerfPhase::CanonicalUniverse,
        || {
            match CANONICAL_UNIVERSE_PROGRAM.get_or_init(|| {
                crate::perf_counter!(telos_observability::PerfCounter::CanonicalUniverseProductBuilds, 1);
                crate::perf_span_result!(telos_observability::PerfPhase::CanonicalUniverseBuild, build_canonical_universe_program, |_| {
                    telos_observability::PerfOutcome::BootstrapError
                },)
            }) {
                Ok(program) => Ok(program),
                Err(error) => Err(error),
            }
        },
        |_| telos_observability::PerfOutcome::BootstrapError,
    )
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

    // Formal acquisition has one semantic owner. Native descriptor validation
    // above and executable projection below remain core responsibilities.
    let image = crate::perf_span!(telos_observability::PerfPhase::CanonicalImageCoreConsumer, || {
        phalcom_semantic::canonical_image::canonical_universe_formal_image()
    })
    .map_err(|error| match error {
        phalcom_semantic::canonical_image::CanonicalImageError::SemanticUpdate(error) => CanonicalUniverseBuildError::SemanticAnalysisFailed(error),
        phalcom_semantic::canonical_image::CanonicalImageError::BootstrapPanicked => CanonicalUniverseBuildError::SemanticAnalysisPanicked,
        error => CanonicalUniverseBuildError::FormalImage(error),
    })?;
    let linked = image.linked().clone();
    let sources = image.sources().clone();
    let semantic = image.snapshot().clone();
    let project_universe = linked.universe.clone();

    if let Some(diagnostic) = semantic
        .diagnostics
        .values()
        .flat_map(|diagnostics| diagnostics.iter())
        .find(|diagnostic| diagnostic.code == phalcom_semantic::DiagnosticCode::NativeSignatureMismatch)
    {
        return Err(CanonicalUniverseBuildError::NativeContracts(diagnostic.message.clone()));
    }
    ensure_executable_match_coverage(&semantic)?;

    let analyzed = AnalyzedProgram {
        project_universe,
        runtime_modules: linked.modules.keys().cloned().collect(),
        linked,
        semantic,
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CanonicalUniverseBuildError, canonical_universe_program};
    use phalcom_modules::{ModuleComponent, ModuleId, ModulePath};
    use phalcom_native_meta::UniverseKey;

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
