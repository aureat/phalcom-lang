use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use std::path::PathBuf;

#[test]
fn telogue_project_compiles_from_declared_entry() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../packages/telogue");

    let compiled =
        ProgramCompiler::compile_entry_selection(EntrySelection::Project(project_root)).expect("Telogue and the modules reachable from prolog.main compile");
    assert_eq!(compiled.entry.path.to_string(), "main");
    for path in [
        "predicate",
        "term",
        "query",
        "variable",
        "rule",
        "base",
        "goal",
        "clause",
        "id",
        "identity_token",
        "relationship",
    ] {
        assert!(
            compiled
                .modules
                .keys()
                .any(|module| module.project == compiled.entry.project && module.path.to_string() == path),
            "reachable Telogue module {path} must be included in the compiled graph",
        );
    }
}
