use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("phalcom-lsp must be inside workspace")
        .to_path_buf()
}

fn production_sources() -> Vec<(PathBuf, String)> {
    fn visit(path: &Path, files: &mut Vec<(PathBuf, String)>) {
        let metadata = fs::metadata(path).expect("source path must be readable");
        if metadata.is_dir() {
            for entry in fs::read_dir(path).expect("source directory must be readable") {
                visit(&entry.expect("source entry must be readable").path(), files);
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push((path.to_path_buf(), fs::read_to_string(path).expect("Rust source must be readable")));
        }
    }

    let mut files = Vec::new();
    visit(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut files);
    files
}

#[test]
fn lsp_has_no_legacy_semantic_package_or_bridge() {
    let lsp = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(!lsp.join("src/semantic").exists());
    assert!(!lsp.join("src/index.rs").exists());

    let lib = fs::read_to_string(lsp.join("src/lib.rs")).expect("LSP lib source must be readable");
    assert!(!lib.contains("pub mod semantic;"));
    assert!(!lib.contains("pub mod index;"));

    let forbidden = [
        "struct SemanticDb",
        "struct SemanticEngine",
        "struct ScopeGraph",
        "struct ModuleGraph",
        "struct DispatchResolver",
        "struct InferredValue",
        "canonical_callables",
        "canonical_target_to_lsp",
        "class_for_canonical",
        "member_surface_for_canonical",
        "CompilerResolvedReceiver",
        "SemanticResolvedReceiver",
        "crate::semantic::",
        "apply_module_mutations_at_generation",
        "resolve_source_import",
    ];
    for (path, source) in production_sources() {
        for symbol in forbidden {
            assert!(!source.contains(symbol), "{symbol} remains in {}", path.display());
        }
    }

    let manifest = fs::read_to_string(lsp.join("Cargo.toml")).expect("LSP manifest must be readable");
    assert!(!manifest.contains("phalcom-native-surface"));
}

#[test]
fn request_features_do_not_read_or_canonicalize_filesystem_paths() {
    let lsp = Path::new(env!("CARGO_MANIFEST_DIR"));
    let request_features = [
        "backend.rs",
        "completion.rs",
        "hover.rs",
        "inlay_hints.rs",
        "request_context.rs",
        "semantic_tokens.rs",
        "signature_help.rs",
    ];
    for file in request_features {
        let path = lsp.join("src").join(file);
        let source = fs::read_to_string(&path).expect("request feature source must be readable");
        assert!(!source.contains("std::fs::"), "filesystem read remains in {}", path.display());
        assert!(!source.contains("canonicalize("), "filesystem canonicalization remains in {}", path.display());
    }
}

#[test]
fn worker_does_not_reimplement_import_resolution_or_generation_publication() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/analysis_service.rs");
    let source = fs::read_to_string(&path).expect("analysis service source must be readable");
    for symbol in [
        "resolve_source_import",
        "extend_import_closure_with_source",
        "apply_module_mutations_at_generation",
    ] {
        assert!(!source.contains(symbol), "{symbol} remains in {}", path.display());
    }
}

#[test]
fn semantic_crate_has_no_lsp_dependency() {
    let manifest = workspace_root().join("phalcom-semantic/Cargo.toml");
    let text = fs::read_to_string(manifest).expect("semantic manifest must be readable");
    assert!(!text.contains("tower-lsp"));
    assert!(!text.contains("phalcom-lsp"));
}
