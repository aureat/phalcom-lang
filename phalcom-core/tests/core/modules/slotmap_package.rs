use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::vm::{BufferedOutput, VM};
use std::path::PathBuf;

fn slotmap_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../packages/slotmap")
}

fn run_project(project_root: PathBuf) -> Vec<u8> {
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Project(project_root)).expect("slotmap project compiles");
    let sink = BufferedOutput::new();
    let output = sink.handle();
    let mut vm = VM::new_with_output(Box::new(sink));
    vm.run_compiled(&program).expect("slotmap project executes");
    output.bytes()
}

#[test]
fn slotmap_package_entry_compiles_and_executes() {
    let output = run_project(slotmap_root());

    assert_eq!(
        output,
        b"Socrates\nPlato\nSocrates key is stale\nSlotKey(index: 0, generation: 0): All humans are mortal\nSlotKey(index: 1, generation: 0): Knowledge is recollection\n"
    );
}

#[test]
fn slotmap_runtime_test_project_checks_sparse_and_dense_invariants() {
    let output = run_project(slotmap_root().join("tests/runtime"));

    assert_eq!(output, b"ok: slotmap-basic\nok: dense-slotmap-basic\nslotmap runtime tests complete\n");
}
