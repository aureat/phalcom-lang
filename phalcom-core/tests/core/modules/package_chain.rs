//! Package artifacts compiled over other package artifacts.
//!
//! A diamond (`top` over `left` and `right`, both over `keys`) covers a
//! prefix chain beyond Universe and a dependency set in which a package
//! is admitted next to a sibling it was not built with. Every package is
//! compiled once to `.telosc`; `top` then loads and runs from artifacts
//! alone, whatever order its dependencies are given in.

use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_core::modules::canonical_artifact::{
    LoadedUniverseArtifact, build_full_universe_telosc, load_universe_telosc, load_universe_telosc_from_bytes, write_full_universe_telosc,
};
use phalcom_core::modules::canonical_universe::canonical_universe_program;
use phalcom_core::modules::{build_full_package_telosc, load_package_telosc_from_bytes};
use std::path::Path;
use std::sync::Arc;

fn full_universe(limits: &ValidationLimits, scratch: &Path) -> Arc<LoadedUniverseArtifact> {
    let path = std::env::var_os("PHALCOM_TEST_FULL_TELOSC_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| scratch.join("universe.telosc"));
    if path.is_file() {
        return load_universe_telosc(&path, limits).expect("load reusable full Universe artifact");
    }
    let program = canonical_universe_program().expect("canonical universe program");
    let built = build_full_universe_telosc(program, limits).expect("build full Universe artifact");
    write_full_universe_telosc(&path, &built).expect("write reusable full Universe artifact");
    load_universe_telosc_from_bytes(&built.bytes, limits).expect("load full Universe artifact")
}

/// Writes package `name` with `files` under `src/` and path dependencies on
/// the sibling packages `dependencies`.
fn package(root: &Path, name: &str, dependencies: &[&str], files: &[(&str, &str)]) -> std::path::PathBuf {
    let directory = root.join(name);
    std::fs::create_dir_all(directory.join("src")).expect("create package source root");
    let mut manifest = format!("[project]\nname = \"{name}\"\nnamespace = \"{name}\"\nversion = \"0.1.0\"\nentry = \"{name}.main\"\n\n[dependencies]\n");
    for dependency in dependencies {
        manifest.push_str(&format!("{dependency} = {{ path = \"{}\" }}\n", root.join(dependency).display()));
    }
    std::fs::write(directory.join("project.toml"), manifest).expect("write package manifest");
    for (file, text) in files {
        std::fs::write(directory.join("src").join(file), text).expect("write package source");
    }
    directory
}

#[test]
fn diamond_of_package_artifacts_runs_from_artifacts_in_any_dependency_order() {
    let limits = ValidationLimits::default();
    let scratch = tempfile::tempdir().expect("scratch directory");
    let universe = full_universe(&limits, scratch.path());
    let root = scratch.path().join("packages");

    let keys = package(
        &root,
        "keys",
        &[],
        &[
            ("package.ph", "expose .key\nexpose .main\n"),
            (
                "key.ph",
                "class Key {\n  mut _n: Int\n\n  @constructor\n  call(_ n: Int) {\n    _n = n\n  }\n\n  value -> Int { _n }\n}\n\nexport Key\n",
            ),
            ("main.ph", "System.print(\"keys loaded\")\n"),
        ],
    );
    // Inside a method a bare `Key(..)` is a send to `self`, so the side
    // packages construct through `Key.call`.
    let side = |name: &str, class: &str, value: u32| {
        let source = format!(
            "from keys.key import Key\n\nclass {class} {{\n  @class\n  make -> Key {{ Key.call({value}) }}\n}}\n\nexport {class}\n"
        );
        package(
            &root,
            name,
            &["keys"],
            &[
                ("package.ph", &format!("expose .{name}\nexpose .main\n")),
                (&format!("{name}.ph"), &source),
                ("main.ph", "System.print(\"side loaded\")\n"),
            ],
        )
    };
    let left = side("left", "Left", 2);
    let right = side("right", "Right", 5);
    let top = package(
        &root,
        "top",
        &["left", "right", "keys"],
        &[
            ("package.ph", "expose .main\n"),
            (
                "main.ph",
                "from left.left import Left\nfrom right.right import Right\n\nlet total = Left.make.value + Right.make.value\nif total != 7 {\n  throw Error.new(\"diamond total \\(total)\")\n}\nSystem.print(\"diamond total \\(total)\")\n",
            ),
        ],
    );

    let build = |path: &Path, dependencies: &[&Vec<u8>]| {
        let dependencies = dependencies.iter().map(|bytes| (*bytes).clone()).collect::<Vec<_>>();
        build_full_package_telosc(path, &universe, &dependencies, &limits)
            .unwrap_or_else(|error| panic!("build {}: {error}", path.display()))
            .bytes
    };
    let keys = build(&keys, &[]);
    let left = build(&left, &[&keys]);
    let right = build(&right, &[&keys]);
    // `left` and `right` were each built beside `keys` alone; `top` admits
    // them together, in an order unrelated to how either was built.
    let top = build(&top, &[&right, &keys, &left]);

    for dependencies in [vec![&keys, &left, &right], vec![&right, &left, &keys]] {
        let dependencies = dependencies.into_iter().cloned().collect::<Vec<_>>();
        let loaded = load_package_telosc_from_bytes(&top, &universe, &dependencies, &limits).expect("load top over its dependency artifacts");
        assert!(loaded.is_executable(), "top artifact carries a portable executable");
        assert_eq!(
            loaded.project,
            phalcom_modules::package_project_id("top", "0.1.0"),
            "a loaded package is numbered by its package identity"
        );
        let mut vm = universe.new_vm().expect("source-free Universe VM");
        loaded.install_into(&mut vm).expect("dependencies and top initialize from artifacts");
    }
}
