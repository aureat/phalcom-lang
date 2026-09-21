use std::sync::Arc;

use phalcom_core::modules::compile::{EntrySelection, ProgramAnalyzer};
use phalcom_core::modules::{ProgramCompileError, ProgramCompiler};
use phalcom_semantic::DiagnosticCode;

#[test]
fn analyzer_preserves_semantic_errors_in_snapshot() {
    let source = Arc::<str>::from("const count: String = 1");

    let analyzed = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Inline(source)).expect("semantic errors should not prevent analysis");

    assert!(analyzed.semantic.has_errors());

    let diagnostics = analyzed
        .semantic
        .diagnostics
        .get(&analyzed.entry)
        .expect("entry module should have diagnostics");

    assert_eq!(diagnostics.len(), 1);

    assert_eq!(diagnostics[0].code, DiagnosticCode::BindingInitializerMismatch);
    assert_eq!(analyzed.sources[&analyzed.entry].text.as_ref(), "const count: String = 1");
    assert_eq!(analyzed.semantic.sources[&analyzed.entry].text.as_ref(), "const count: String = 1");
}

#[test]
fn compiler_accepts_valid_analyzed_program() {
    let source = Arc::<str>::from("const count: Int = 1");

    let analyzed = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Inline(source)).expect("valid program should analyze");
    assert!(!analyzed.semantic.has_errors());

    ProgramCompiler::compile_analyzed(&analyzed).expect("valid analyzed program should compile");
}

#[test]
fn compiler_rejects_program_with_semantic_errors() {
    let source = Arc::<str>::from("const count: String = 1");

    let error = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(source)).expect_err("compiler must reject semantic errors");

    let ProgramCompileError::Semantic(diagnostics) = error else {
        panic!("expected semantic compilation error");
    };

    assert!(diagnostics.has_errors());
}

#[test]
fn standalone_module_projects_canonical_for_iteration_lowering() {
    let directory = tempfile::tempdir().expect("temporary source directory");
    let source_path = directory.path().join("iteration.ph");
    std::fs::write(
        &source_path,
        r#"
for value in [1, 2] {
  System.print(value)
}
"#,
    )
    .expect("standalone source");

    let analyzed = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Module(source_path)).expect("standalone module analyzes");
    assert!(
        !analyzed.semantic.has_errors(),
        "canonical List iteration must analyze without diagnostics: {:#?}",
        analyzed.semantic.diagnostics
    );

    let program = ProgramCompiler::compile_analyzed(&analyzed).expect("standalone module projects");
    assert_eq!(
        program.modules[&program.entry].lowering.for_iteration_lanes.len(),
        1,
        "standalone List iteration must retain its semantic lowering specification"
    );
}
