use std::{fs, path::Path};

use crate::support::{MarkedSource, fixture_path};

#[test]
fn all_non_incomplete_phalcom_fixtures_parse() {
    let root = fixture_path("");
    let mut files = Vec::new();
    collect_ph_files(&root, &mut files);

    for path in files {
        if path.components().any(|part| part.as_os_str() == "incomplete") {
            continue;
        }

        let raw = fs::read_to_string(&path).expect("read fixture");
        let source = MarkedSource::parse(&raw).text;
        let parsed = phalcom_ast::parse(&source, 0);

        assert!(
            parsed.errors.is_empty(),
            "fixture {} must track current Phalcom syntax; errors={:#?}\nsource:\n{}",
            path.display(),
            parsed.errors,
            source
        );
    }
}

fn collect_ph_files(dir: &Path, output: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).expect("read fixture directory") {
        let entry = entry.expect("fixture entry");
        let path = entry.path();

        if path.is_dir() {
            collect_ph_files(&path, output);
        } else if path.extension().is_some_and(|ext| ext == "ph") {
            output.push(path);
        }
    }
}
