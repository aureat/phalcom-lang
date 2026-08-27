use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("phalcom-lsp must be inside workspace")
        .to_path_buf()
}

#[test]
#[ignore = "enabled when legacy LSP semantic package is physically deleted"]
fn lsp_has_no_legacy_semantic_package() {
    let lsp = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(!lsp.join("src/semantic").exists());
}

#[test]
fn semantic_crate_has_no_lsp_dependency() {
    let manifest = workspace_root().join("phalcom-semantic/Cargo.toml");
    let text = fs::read_to_string(manifest).expect("semantic manifest must be readable");
    assert!(!text.contains("tower-lsp"));
    assert!(!text.contains("phalcom-lsp"));
}
