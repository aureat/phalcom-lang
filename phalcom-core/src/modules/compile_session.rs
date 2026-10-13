//! Incremental compilation session for standalone modules and language corpus fixtures.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;
#[cfg(any(test, feature = "test-timings"))]
use std::time::{Duration, Instant};

use phalcom_modules::{
    ModuleId, ModulePath, SourceId, SourceLocation, SourceRevision, SyntheticProjectIdAllocator, WorkspaceModuleSessionError, WorkspaceSourceBatchMutation,
};
use phalcom_semantic::SemanticWorkspaceSession;

use super::compile::{AnalyzedProgram, CompiledProgram, ProgramCompileError, ProgramCompiler};

/// A persistent compilation session that replaces one fixture module at a time.
///
/// The module ID and source ID are allocated once per session. Each fixture
/// mutation updates the source location and text under those retained identities,
/// allowing the module and semantic sessions to invalidate prior fixture facts.
pub struct ProgramCompilationSession {
    semantic_session: SemanticWorkspaceSession,
    generation: u64,
    fixture_module_id: ModuleId,
    fixture_source_id: SourceId,
    #[cfg(any(test, feature = "test-timings"))]
    last_fixture_timing: crate::instrumentation::FixtureTiming,
    #[cfg(any(test, feature = "test-timings"))]
    pending_session_init: Duration,
}

impl ProgramCompilationSession {
    /// Creates a standalone session backed by the canonical semantic baseline.
    pub fn standalone() -> Result<Self, ProgramCompileError> {
        crate::perf_span_result!(
            telos_observability::PerfPhase::SemanticSessionInit,
            || {
                #[cfg(any(test, feature = "test-timings"))]
                let init_started = Instant::now();
                let semantic_session = SemanticWorkspaceSession::new();
                #[cfg(any(test, feature = "test-timings"))]
                let pending_session_init = init_started.elapsed();
                let mut synthetic_ids = SyntheticProjectIdAllocator;
                let fixture_module_id = ModuleId::synthetic(synthetic_ids.allocate(), ModulePath::root());
                let fixture_source_id = SourceId(format!("phalcom-perf003-fixture:{fixture_module_id}").into());
                Ok(Self {
                    semantic_session,
                    generation: 0,
                    fixture_module_id,
                    fixture_source_id,
                    #[cfg(any(test, feature = "test-timings"))]
                    last_fixture_timing: crate::instrumentation::FixtureTiming::default(),
                    #[cfg(any(test, feature = "test-timings"))]
                    pending_session_init,
                })
            },
            super::compile::classify_program_compile_error,
        )
    }

    /// Replaces the current fixture source and compiles it through the retained
    /// module and semantic sessions.
    pub fn compile_fixture(&mut self, path: &Path, source_text: &str) -> Result<CompiledProgram, ProgramCompileError> {
        crate::perf_span_result!(
            telos_observability::PerfPhase::Compilation,
            || {
                #[cfg(any(test, feature = "test-timings"))]
                {
                    self.last_fixture_timing = crate::instrumentation::FixtureTiming {
                        semantic_session_init: std::mem::take(&mut self.pending_session_init),
                        ..crate::instrumentation::FixtureTiming::default()
                    };
                }
                #[cfg(any(test, feature = "test-timings"))]
                let parse_started = Instant::now();
                let parsed = phalcom_ast::parse(source_text, 0);
                #[cfg(any(test, feature = "test-timings"))]
                {
                    self.last_fixture_timing.compile_parse = parse_started.elapsed();
                }
                if let Some(error) = parsed.errors.first() {
                    return Err(ProgramCompileError::Parse(error.clone()));
                }

                for dependency in &parsed.program.preamble.dependencies {
                    let import_path = match dependency {
                        phalcom_ast::ast::DependencyDecl::Import(phalcom_ast::ast::ImportDecl::Module(decl)) => &decl.path,
                        phalcom_ast::ast::DependencyDecl::Import(phalcom_ast::ast::ImportDecl::Selective(decl)) => &decl.path,
                        phalcom_ast::ast::DependencyDecl::ReExport(decl) => &decl.path,
                        phalcom_ast::ast::DependencyDecl::Expose(decl) => {
                            return Err(ProgramCompileError::StandaloneImportRequiresPackageContext {
                                import_name: decl.child.name.clone().into(),
                            });
                        }
                    };
                    if !matches!(&import_path.root, phalcom_ast::ast::ImportRoot::Absolute(root) if root.name == "universe") {
                        return Err(ProgramCompileError::StandaloneImportRequiresPackageContext {
                            import_name: import_path.to_string().into(),
                        });
                    }
                }

                self.generation = self.generation.saturating_add(1);
                let location = SourceLocation {
                    source_id: self.fixture_source_id.clone(),
                    display_path: path.to_path_buf(),
                };
                let mutation = WorkspaceSourceBatchMutation::SetOverlayForModule {
                    module: self.fixture_module_id.clone(),
                    source: location,
                    text: Arc::from(source_text),
                    revision: SourceRevision(self.generation),
                    recovered_program: Some(Arc::new(parsed.program)),
                };

                #[cfg(any(test, feature = "test-timings"))]
                let link_started = Instant::now();
                let module_update = match self.semantic_session.module_session_mut().apply_batch([mutation]) {
                    Ok(update) => update,
                    Err(error) => {
                        #[cfg(any(test, feature = "test-timings"))]
                        {
                            self.last_fixture_timing.compile_link = link_started.elapsed();
                        }
                        return Err(map_workspace_error(error));
                    }
                };
                #[cfg(any(test, feature = "test-timings"))]
                {
                    self.last_fixture_timing.compile_link = link_started.elapsed();
                }
                #[cfg(any(test, feature = "test-timings"))]
                let semantic_started = Instant::now();
                let publication = self
                    .semantic_session
                    .update_module_workspace_with_cancel(module_update, &phalcom_semantic::db::CancellationToken::new())
                    .map_err(phalcom_semantic::SemanticUpdateFailure::from)?;
                #[cfg(any(test, feature = "test-timings"))]
                {
                    self.last_fixture_timing.semantic_update = semantic_started.elapsed();
                }
                let module_session = self.semantic_session.module_session();
                let linked = module_session
                    .linked()
                    .cloned()
                    .ok_or_else(|| ProgramCompileError::Io("module session published no linked program".to_owned()))?;
                let entry = self.fixture_module_id.clone();
                let analyzed = AnalyzedProgram {
                    project_universe: Arc::new(module_session.universe().clone()),
                    linked,
                    sources: publication.snapshot.sources().as_ref().clone(),
                    semantic: publication.snapshot,
                    runtime_modules: BTreeSet::from([entry.clone()]),
                    entry,
                };

                #[cfg(any(test, feature = "test-timings"))]
                let projection_started = Instant::now();
                let compiled = ProgramCompiler::compile_analyzed(&analyzed);
                #[cfg(any(test, feature = "test-timings"))]
                {
                    self.last_fixture_timing.compile_projection = projection_started.elapsed();
                }
                compiled
            },
            super::compile::classify_program_compile_error,
        )
    }

    /// Returns the latest fixture timing breakdown when test timing support is enabled.
    #[cfg(any(test, feature = "test-timings"))]
    pub fn last_fixture_timing(&self) -> &crate::instrumentation::FixtureTiming {
        &self.last_fixture_timing
    }
}

fn map_workspace_error(error: WorkspaceModuleSessionError) -> ProgramCompileError {
    match error {
        WorkspaceModuleSessionError::Project(error) => error.into(),
        WorkspaceModuleSessionError::Module(error) => error.into(),
        WorkspaceModuleSessionError::Resolution(error) => error.into(),
        WorkspaceModuleSessionError::Interface(error) => error.into(),
        WorkspaceModuleSessionError::Link(error) => error.into(),
        WorkspaceModuleSessionError::Source(error) => error.into(),
        WorkspaceModuleSessionError::Parse { error, .. } => ProgramCompileError::Parse(error),
        WorkspaceModuleSessionError::UnknownSource(source) => ProgramCompileError::Io(format!("unknown source {source}")),
        WorkspaceModuleSessionError::InvalidSourcePath(path) => ProgramCompileError::Io(format!("invalid source path {}", path.display())),
        other => ProgramCompileError::Io(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::compile::{EntrySelection, ProgramCompileError};

    fn semantic_error(result: Result<CompiledProgram, ProgramCompileError>) -> crate::modules::compile::ProgramSemanticDiagnostics {
        match result {
            Err(ProgramCompileError::Semantic(diagnostics)) => diagnostics,
            Ok(_) => panic!("expected semantic failure, compilation succeeded"),
            Err(error) => panic!("expected semantic failure, got {error}"),
        }
    }

    fn write_fixture(path: &Path, source: &str) {
        std::fs::write(path, source).unwrap();
    }

    fn signature_shape(module: &crate::modules::compile::CompiledModule) -> Vec<(String, String, String, usize, String)> {
        let mut shape = module
            .lowering
            .callable_signatures
            .iter()
            .filter(|(_, signature)| signature.owner.module == module.id)
            .map(|(_, signature)| {
                (
                    signature.owner.name.to_string(),
                    signature.selector.encode(),
                    format!("{:?}", signature.side),
                    signature.parameters.len(),
                    format!("{:?}", signature.declared_return),
                )
            })
            .collect::<Vec<_>>();
        shape.sort();
        shape
    }

    fn declaration_shape(module: &crate::modules::compile::CompiledModule) -> Vec<String> {
        let mut shape = module
            .plan
            .declarations
            .iter()
            .map(|declaration| match declaration {
                crate::modules::artifact::RuntimeDeclarationBlueprint::Class(class) => format!(
                    "class:{}:{:?}:{}:{}",
                    class.symbol.name,
                    class.superclass.as_ref().map(|symbol| symbol.name.as_ref()),
                    class.fields.iter().map(AsRef::as_ref).collect::<Vec<_>>().join(","),
                    class.methods.iter().map(AsRef::as_ref).collect::<Vec<_>>().join(",")
                ),
                crate::modules::artifact::RuntimeDeclarationBlueprint::Enum(enumeration) => format!(
                    "enum:{}:{}",
                    enumeration.symbol.name,
                    enumeration
                        .variants
                        .iter()
                        .map(|variant| format!("{}:{:?}:{}", variant.name, variant.shape, variant.payload_arity))
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                crate::modules::artifact::RuntimeDeclarationBlueprint::Global { symbol, mutable } => {
                    format!("global:{}:{mutable}", symbol.name)
                }
            })
            .collect::<Vec<_>>();
        shape.sort();
        shape
    }

    #[test]
    fn replacement_keeps_module_identity_and_publishes_fixture_path() {
        let directory = tempfile::tempdir().unwrap();
        let first_path = directory.path().join("first.ph");
        let second_path = directory.path().join("second.ph");
        let mut session = ProgramCompilationSession::standalone().unwrap();

        let first = session.compile_fixture(&first_path, "const first: Int = 1\n").unwrap();
        let second = session.compile_fixture(&second_path, "const second: Int = 2\n").unwrap();

        assert_eq!(first.entry, second.entry, "fixture replacement retains its module identity");
        let first_source = first.modules[&first.entry].source.as_ref().unwrap();
        let second_source = second.modules[&second.entry].source.as_ref().unwrap();
        assert_eq!(first_source.source_id, second_source.source_id);
        assert_eq!(first_source.display_path, phalcom_modules::source::canonicalize_path(&first_path));
        assert_eq!(second_source.display_path, phalcom_modules::source::canonicalize_path(&second_path));
        assert_eq!(second.modules.len(), 1, "standalone result contains only the current fixture module");
    }

    #[test]
    fn retained_random_fixture_replacement_publishes_current_source() {
        let mut session = ProgramCompilationSession::standalone().unwrap();
        let path = Path::new("random-retained.ph");
        let first = "from universe.random import Xoshiro256PlusPlus\nlet result = Xoshiro256PlusPlus.seed(42).nextU64\n";
        let second = "from universe.random import Xoshiro256PlusPlus\nlet result = Xoshiro256PlusPlus.seed(7).nextU64\n";
        for source in [
            first,
            second,
            "const result: Int = 17\n",
            second,
            include_str!("../../tests/fixtures/language/random/random_foundation.ph"),
            include_str!("../../tests/fixtures/language/random-syntax/random_syntax.ph"),
            first,
        ] {
            let program = session.compile_fixture(path, source).unwrap();
            assert_eq!(program.modules[&program.entry].source_text.as_deref(), Some(source));
            let cold = ProgramCompilationSession::standalone().unwrap().compile_fixture(path, source).unwrap();
            assert_eq!(
                program.modules[&program.entry].lowering.associated.len(),
                cold.modules[&cold.entry].lowering.associated.len()
            );
            assert_eq!(signature_shape(&program.modules[&program.entry]), signature_shape(&cold.modules[&cold.entry]));
        }
    }

    #[test]
    fn replaced_fixture_does_not_leak_declarations_or_bindings() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("fixture.ph");
        let mut session = ProgramCompilationSession::standalone().unwrap();

        session.compile_fixture(&path, "class FixtureOnly {}\n").unwrap();
        let declaration = session.compile_fixture(&path, "class UsesFixtureOnly is FixtureOnly {}\n");
        assert!(declaration.is_err(), "a later fixture must not inherit the prior class declaration");
        assert!(declaration.unwrap_err().to_string().contains("FixtureOnly"));

        session.compile_fixture(&path, "const fixture_binding: Int = 1\n").unwrap();
        let binding = session.compile_fixture(&path, "fixture_binding\n").unwrap();
        let current = session.semantic_session.last_snapshot().unwrap();
        let current_structure = &current.source_index.module(&binding.entry).unwrap().structure;
        assert!(
            current_structure.bindings.values().all(|binding| binding.name.as_ref() != "fixture_binding"),
            "an unresolved dynamic read must not be rebound to a declaration from the preceding fixture"
        );
        assert!(
            binding.modules[&binding.entry].source_text.as_deref().unwrap().contains("fixture_binding"),
            "the fixture must actually reference the prior binding name"
        );
    }

    #[test]
    fn replacement_removes_imports_and_recovers_with_cold_compile_parity() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("fixture.ph");
        let source_a = "import universe.reflection.selector\nclass FixtureShape { value { 42 } }\n";
        let mut session = ProgramCompilationSession::standalone().unwrap();

        write_fixture(&path, source_a);
        let first = session.compile_fixture(&path, source_a).unwrap();
        let module_id = first.entry.clone();
        let first_module = &first.modules[&module_id];
        assert!(!session.semantic_session.module_session().interface(&module_id).unwrap().imports.is_empty());

        let source_b = "const broken: String = 1\n";
        write_fixture(&path, source_b);
        let reused_error = semantic_error(session.compile_fixture(&path, source_b));
        assert!(
            session.semantic_session.module_session().interface(&module_id).unwrap().imports.is_empty(),
            "replacement must remove imports that disappeared from the fixture"
        );
        let cold_error = semantic_error(ProgramCompiler::compile_entry_selection(EntrySelection::Module(path.clone())));
        let reused_diagnostics = reused_error.for_module(&module_id);
        let cold_snapshot = cold_error.snapshot.as_ref().unwrap();
        let cold_module = cold_snapshot
            .sources
            .iter()
            .find(|(_, unit)| {
                unit.source
                    .as_ref()
                    .is_some_and(|source| source.display_path == phalcom_modules::source::canonicalize_path(&path))
            })
            .map(|(module, _)| module)
            .expect("cold compile source path");
        let cold_diagnostics = cold_error.for_module(cold_module);
        assert_eq!(reused_diagnostics.len(), 1);
        assert_eq!(cold_diagnostics.len(), 1);
        assert_eq!(reused_diagnostics[0].code, cold_diagnostics[0].code);
        assert_eq!(reused_diagnostics[0].message, cold_diagnostics[0].message);
        assert_eq!(reused_diagnostics[0].primary_range, cold_diagnostics[0].primary_range);
        assert_eq!(
            reused_error.snapshot.as_ref().unwrap().sources[&module_id]
                .source
                .as_ref()
                .unwrap()
                .display_path,
            phalcom_modules::source::canonicalize_path(&path)
        );

        let source_c = "import universe.reflection.selector\nclass FixtureShape { value { 42 } }\n";
        write_fixture(&path, source_c);
        let last = session.compile_fixture(&path, source_c).unwrap();
        let cold = ProgramCompiler::compile_entry_selection(EntrySelection::Module(path.clone())).unwrap();

        assert_eq!(first.modules[&module_id].plan.declarations, last.modules[&module_id].plan.declarations);
        assert_eq!(first_module.lowering.callable_signatures, last.modules[&module_id].lowering.callable_signatures);
        assert_eq!(
            signature_shape(&last.modules[&module_id]),
            signature_shape(&cold.modules[&cold.entry]),
            "reused and cold compilation preserve callable declaration signatures"
        );
        assert_eq!(
            declaration_shape(&last.modules[&module_id]),
            declaration_shape(&cold.modules[&cold.entry]),
            "reused and cold compilation preserve declaration shapes"
        );
    }
}
