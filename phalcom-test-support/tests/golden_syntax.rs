use std::{
    fs, io,
    path::{Path, PathBuf},
};

use phalcom_test_support::{GoldenWorkspace, MarkedSource};

#[test]
fn every_golden_source_parses_without_errors() {
    let workspace = GoldenWorkspace::repository_fixture();
    let mut paths = Vec::new();
    collect_ph_files(workspace.root(), &mut paths).unwrap();
    paths.sort();

    // This source intentionally contains competing, not-yet-ratified enum
    // spellings for syntax-highlighting coverage; it is not a parser fixture.
    let parse_paths = paths
        .into_iter()
        .filter(|path| path.file_name().is_none_or(|name| name != "phalcom-enum-highlighting-showcase.ph"))
        .collect::<Vec<_>>();
    assert_eq!(parse_paths.len(), 28, "golden source count changed: {parse_paths:?}");

    for path in parse_paths {
        let raw = fs::read_to_string(&path).unwrap();
        let source = MarkedSource::parse(&raw);
        let parsed = phalcom_ast::parse(&source.text, 0);
        assert!(
            parsed.errors.is_empty(),
            "golden source failed to parse: {}\nerrors: {:#?}\nsource:\n{}",
            path.display(),
            parsed.errors,
            source.text
        );
    }
}

fn collect_ph_files(dir: &Path, output: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_ph_files(&path, output)?;
        } else if path.extension().is_some_and(|extension| extension == "ph") {
            output.push(path);
        }
    }
    Ok(())
}
