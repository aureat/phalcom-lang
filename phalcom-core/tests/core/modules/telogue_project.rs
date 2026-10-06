use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use std::path::PathBuf;

#[test]
fn telogue_project_compiles_from_declared_entry() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../packages/telogue");

    ProgramCompiler::compile_entry_selection(EntrySelection::Project(project_root))
        .expect("Telogue and the modules reachable from prolog.main compile");
}
