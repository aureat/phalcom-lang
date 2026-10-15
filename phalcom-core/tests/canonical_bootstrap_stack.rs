use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::vm::VM;
use std::fs;
use std::thread;
use tempfile::TempDir;

fn minimal_project() -> TempDir {
    let temp = TempDir::new().expect("temporary project");
    let source = temp.path().join("src");
    fs::create_dir_all(&source).expect("create source directory");
    fs::write(
        temp.path().join("project.toml"),
        "[project]\nname = \"minimal_compile\"\nnamespace = \"minimal_compile\"\nversion = \"0.1.0\"\nsource = \"src\"\nentry = \"minimal_compile.main\"\n",
    )
    .expect("project manifest");
    fs::write(source.join("package.ph"), "expose .main\n").expect("package source");
    fs::write(source.join("main.ph"), "let value = 1\nexport value\n").expect("entry source");
    temp
}

#[test]
fn dependency_free_project_compiles_then_bootstraps_and_runs() {
    let project = minimal_project();
    thread::Builder::new()
        .name("minimal-project-canonical-bootstrap".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            let program =
                ProgramCompiler::compile_entry_selection(EntrySelection::Project(project.path().to_path_buf())).expect("dependency-free project compiles");
            let mut vm = VM::new();
            vm.run_compiled(&program).expect("minimal project runs");
        })
        .expect("spawn bootstrap regression thread")
        .join()
        .expect("compile followed by VM bootstrap must not overflow the stack");
}
