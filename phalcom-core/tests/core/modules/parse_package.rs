use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn package_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../packages/parse")
}

/// The library project compiles from its declared entry.
#[test]
fn parse_project_compiles_from_declared_entry() {
    let compiled = ProgramCompiler::compile_entry_selection(EntrySelection::Project(package_root()))
        .expect("parse and the modules reachable from parse.main compile");
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
            compiled.modules.keys().any(|module| {
                module.project == compiled.entry.project && module.path.to_string() == path
            }),
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
        } else if path.extension().is_some_and(|ext| ext == "ph")
            && path
                .file_stem()
                .is_some_and(|stem| stem.to_string_lossy().ends_with("-test"))
        {
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
    assert!(
        !fixtures.is_empty(),
        "runtime fixture corpus must exist under {}",
        fixture_root.display()
    );

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
                        .map(|fixture| run_fixture(&binary, &fixture))
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
        assert_eq!(
            passed.len(),
            fixtures.len(),
            "every discovered fixture must report a PASS marker"
        );
    });
}

fn partition_into_chunks(items: Vec<PathBuf>, groups: usize) -> Vec<Vec<PathBuf>> {
    let mut chunks: Vec<Vec<PathBuf>> = vec![Vec::new(); groups.max(1)];
    for (index, item) in items.into_iter().enumerate() {
        chunks[index % groups.max(1)].push(item);
    }
    chunks
}

fn run_fixture(binary: &Path, fixture: &Path) -> Result<String, String> {
    let output = Command::new(binary)
        .arg(fixture)
        .output()
        .unwrap_or_else(|error| panic!("spawn phalcom for {}: {error}", fixture.display()));
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() || !stdout.lines().any(|line| line.starts_with("PASS ")) {
        Err(format!(
            "fixture {} failed\nstdout:\n{stdout}\nstderr:\n{stderr}",
            fixture.display()
        ))
    } else {
        Ok(fixture.display().to_string())
    }
}
