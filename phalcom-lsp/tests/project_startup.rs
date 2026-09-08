//! Project-aware startup coverage for manifest and dependency-root discovery.

use phalcom_lsp::analysis_service::{AnalysisService, WorkspaceScanRequest};
use phalcom_lsp::workspace_scan::AnalysisMode;
use std::fs;

#[test]
fn startup_ingests_manifest_and_scans_dependency_source_roots() {
    let root = std::env::temp_dir().join(format!("phalcom_lsp_project_startup_{}", std::process::id()));
    let app = root.join("app");
    let dependency = root.join("dependency");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(app.join("src")).expect("create app source root");
    fs::create_dir_all(dependency.join("src")).expect("create dependency source root");

    fs::write(
        app.join("project.toml"),
        "[project]\nname = \"startup-app\"\nnamespace = \"startup_app\"\nversion = \"0.1.0\"\n[dependencies]\ndep = { path = \"../dependency\" }\n",
    )
    .expect("write app manifest");
    fs::write(
        dependency.join("project.toml"),
        "[project]\nname = \"startup-dependency\"\nnamespace = \"startup_dependency\"\nversion = \"0.1.0\"\n",
    )
    .expect("write dependency manifest");
    fs::write(app.join("src/package.ph"), "").expect("write app package");
    fs::write(dependency.join("src/package.ph"), "").expect("write dependency package");
    fs::write(app.join("src/main.ph"), "class Main {}\nexport Main\n").expect("write app source");
    fs::write(dependency.join("src/provider.ph"), "class Provider {}\nexport Provider\n").expect("write dependency source");

    let (service, _events) = AnalysisService::new();
    service.configure_workspace(WorkspaceScanRequest {
        roots: vec![app.clone()],
        mode: AnalysisMode::Local,
        excludes: Vec::new(),
    });
    service.flush();

    let snapshot = service.snapshot().expect("project scan must publish a snapshot");
    assert_eq!(
        snapshot.module_products.universe.projects().len(),
        2,
        "root and path dependency must be loaded before scanning"
    );
    assert!(
        snapshot.sources.values().any(|source| source.text.as_ref() == "class Main {}\nexport Main\n"),
        "root project source must be indexed"
    );
    assert!(
        snapshot
            .sources
            .values()
            .any(|source| source.text.as_ref() == "class Provider {}\nexport Provider\n"),
        "path dependency source root must be indexed even when it is outside the advertised workspace root"
    );

    service.shutdown();
    let _ = fs::remove_dir_all(root);
}
