use phalcom_core::heap::Object;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn package_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../packages/parse")
}

/// The library project compiles from its declared entry.
#[test]
fn parse_project_compiles_from_declared_entry() {
    let compiled =
        ProgramCompiler::compile_entry_selection(EntrySelection::Project(package_root())).expect("parse and the modules reachable from parse.main compile");
    assert_eq!(compiled.entry.path.to_string(), "main");
    for path in [
        "core.source",
        "core.cursor",
        "core.error",
        "core.result",
        "core.rule",
        "core.pair",
        "core.span",
        "core.parser",
        "source.text_source",
        "source.list_source",
        "text",
        "run",
    ] {
        assert!(
            compiled
                .modules
                .keys()
                .any(|module| { module.project == compiled.entry.project && module.path.to_string() == path }),
            "reachable parse module {path} must be included in the compiled graph",
        );
    }
}

fn discover_fixtures(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            discover_fixtures(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "ph") && path.file_stem().is_some_and(|stem| stem.to_string_lossy().ends_with("-test")) {
            out.push(path);
        }
    }
}

/// Every `*-test.ph` fixture under the runtime test project must run to
/// completion and print its `PASS <name>` marker. Fixture compilation is
/// expensive (the generic library is instantiated per fixture), so the
/// runner fans fixture chunks out over a small worker pool.
#[test]
fn parse_runtime_fixtures_pass() {
    let fixture_root = package_root().join("tests/runtime/src");
    let mut fixtures = Vec::new();
    discover_fixtures(&fixture_root, &mut fixtures);
    fixtures.sort();
    assert!(!fixtures.is_empty(), "runtime fixture corpus must exist under {}", fixture_root.display());

    let binary = PathBuf::from(env!("CARGO_BIN_EXE_phalcom"));
    let workers = fixtures.len().min(4);
    let chunks: Vec<Vec<PathBuf>> = partition_into_chunks(fixtures.clone(), workers);

    std::thread::scope(|scope| {
        let handles: Vec<_> = chunks
            .into_iter()
            .map(|chunk| {
                let binary = &binary;
                scope.spawn(move || {
                    chunk
                        .into_iter()
                        .map(|fixture| run_fixture(binary, &fixture))
                        .collect::<Vec<Result<String, String>>>()
                })
            })
            .collect();
        let mut passed: BTreeMap<String, String> = BTreeMap::new();
        let mut failures = Vec::new();
        for handle in handles {
            for result in handle.join().expect("fixture worker must not panic") {
                match result {
                    Ok(fixture) => {
                        let name = Path::new(&fixture)
                            .file_stem()
                            .map(|stem| stem.to_string_lossy().to_string())
                            .unwrap_or_else(|| fixture.clone());
                        passed.insert(name, fixture);
                    }
                    Err(message) => failures.push(message),
                }
            }
        }
        assert!(
            failures.is_empty(),
            "{} of {} parse runtime fixtures failed:\n{}",
            failures.len(),
            fixtures.len(),
            failures.join("\n---\n")
        );
        assert_eq!(passed.len(), fixtures.len(), "every discovered fixture must report a PASS marker");
    });
}

/// The P5 parser snapshot contract must be executable without importing the
/// unrelated TextSource migration surface.
#[test]
fn parse_list_source_snapshot_fixture_passes() {
    let fixture = package_root().join("tests/runtime/src/core/list-source-snapshot-test.ph");
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_phalcom"));

    run_fixture_with_marker(&binary, &fixture, Some("PASS core-list-source-snapshot")).expect("ListSource snapshot fixture must pass");
}

#[test]
fn parse_list_source_snapshot_owner_survives_gc() {
    let fixture = package_root().join("tests/runtime/src/core/list-source-snapshot-test.ph");
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Module(fixture)).expect("independent ListSource fixture compiles");
    let mut vm = VM::new();
    vm.run_compiled(&program).expect("independent ListSource fixture runs");
    let source_module = program
        .modules
        .keys()
        .find(|id| id.path.to_string() == "source.list_source")
        .expect("real ListSource module");
    let module = vm.module_registry.get(source_module).expect("loaded source module").object;
    let list_source = vm
        .heap
        .module(module)
        .get(vm.interner.find("ListSource").expect("ListSource symbol"))
        .expect("ListSource class");

    let first = vm.alloc_string_value("first heap token".to_owned());
    vm.push_root_for_test(first);
    let second = vm.alloc_string_value("second heap token".to_owned());
    let caller = vm.heap.alloc_list(vec![first, second]);
    vm.pop_root_for_test();
    vm.push_root_for_test(Value::obj(caller));
    let new = vm.get_or_intern("new(_)");
    let source = vm
        .send_dynamic(list_source, new, &[Value::obj(caller)])
        .expect("real ListSource constructor snapshots the caller List");
    vm.pop_root_for_test();
    vm.push_root_for_test(source);

    let Object::Instance(instance) = vm.heap.get(source.as_obj().expect("ListSource instance")) else {
        panic!("ListSource must be a source instance");
    };
    assert_eq!(instance.slots.len(), 1, "no independently rooted backing Array field");
    let slice_id = instance.slots[0].as_obj().expect("source field is heap-backed");
    let Object::Slice(slice) = vm.heap.get(slice_id) else {
        panic!("ListSource must retain the canonical Slice");
    };
    let owner = slice.owner();
    assert!(matches!(vm.heap.get(owner), Object::Array(_)), "Slice owner is Array, not List or Vector");
    assert_ne!(owner, caller, "snapshot has independent storage");
    assert_eq!(slice.start(), 0);
    assert_eq!(slice.len(), 2);

    // ObjRef/Value locals are not roots. Only the ListSource root retains this
    // freshly allocated Slice -> Array -> String graph during collection.
    vm.force_gc();
    assert!(vm.heap.try_get(caller).is_none(), "caller List is unreachable and collected");
    assert!(vm.heap.try_get(slice_id).is_some(), "source retains Slice");
    assert!(vm.heap.try_get(owner).is_some(), "Slice retains private Array");
    for token in [first, second] {
        assert!(vm.heap.try_get(token.as_obj().expect("heap token")).is_some(), "Array retains heap token");
    }
    let entry = vm.module_registry.get(&program.entry).expect("fixture entry").object;
    let factory = vm
        .heap
        .module(entry)
        .get(vm.interner.find("SourceFactory").expect("factory symbol"))
        .expect("fixture factory");
    let verify = vm.get_or_intern("verifyStrings(_)");
    assert_eq!(
        vm.send_dynamic(factory, verify, &[source]).expect("post-GC Cursor and parser reads"),
        Value::bool(true)
    );
    vm.pop_root_for_test();
}

fn partition_into_chunks(items: Vec<PathBuf>, groups: usize) -> Vec<Vec<PathBuf>> {
    let mut chunks: Vec<Vec<PathBuf>> = vec![Vec::new(); groups.max(1)];
    for (index, item) in items.into_iter().enumerate() {
        chunks[index % groups.max(1)].push(item);
    }
    chunks
}

fn run_fixture(binary: &Path, fixture: &Path) -> Result<String, String> {
    run_fixture_with_marker(binary, fixture, None)
}

fn run_fixture_with_marker(binary: &Path, fixture: &Path, expected_marker: Option<&str>) -> Result<String, String> {
    let output = Command::new(binary)
        .arg(fixture)
        .output()
        .unwrap_or_else(|error| panic!("spawn phalcom for {}: {error}", fixture.display()));
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let passed = stdout.lines().any(|line| match expected_marker {
        Some(marker) => line == marker,
        None => line.starts_with("PASS "),
    });
    if !output.status.success() || !passed {
        Err(format!("fixture {} failed\nstdout:\n{stdout}\nstderr:\n{stderr}", fixture.display()))
    } else {
        Ok(fixture.display().to_string())
    }
}
