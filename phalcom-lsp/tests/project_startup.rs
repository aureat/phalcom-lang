//! Project-aware startup coverage for manifest and dependency-root discovery.

use phalcom_lsp::analysis_service::{AnalysisService, WorkspaceScanRequest};
use phalcom_lsp::workspace_scan::AnalysisMode;
use phalcom_modules::identity::ProjectIdentity;
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
    fs::write(dependency.join("src/package.ph"), "expose .provider\n").expect("write dependency package");
    fs::write(app.join("src/main.ph"), "from dep.provider import Provider\nexport Provider\n").expect("write app source");
    fs::write(dependency.join("src/provider.ph"), "class Provider {}\nexport Provider\n").expect("write dependency source");
    let app = fs::canonicalize(app).expect("canonicalize app root");
    let dependency = fs::canonicalize(dependency).expect("canonicalize dependency root");

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
        snapshot
            .sources
            .values()
            .any(|source| source.text.as_ref() == "from dep.provider import Provider\nexport Provider\n"),
        "root project source must be indexed"
    );
    assert!(
        snapshot
            .sources
            .values()
            .any(|source| source.text.as_ref() == "class Provider {}\nexport Provider\n"),
        "path dependency source root must be indexed even when it is outside the advertised workspace root"
    );

    let root_project = snapshot
        .module_products
        .universe
        .projects()
        .iter()
        .find(|project| project.root_dir == app)
        .expect("app project must retain its resolved identity");
    let dependency_project = snapshot
        .module_products
        .universe
        .projects()
        .iter()
        .find(|project| project.root_dir == dependency)
        .expect("dependency project must retain its resolved identity");
    assert_ne!(root_project.id, dependency_project.id);

    let main_module = snapshot
        .module_for_display_path(&app.join("src/main.ph"))
        .expect("app source must map to a canonical module");
    let provider_module = snapshot
        .module_for_display_path(&dependency.join("src/provider.ph"))
        .expect("dependency source must map to a canonical module");
    assert_eq!(main_module.project, ProjectIdentity::Resolved(root_project.id));
    assert_eq!(provider_module.project, ProjectIdentity::Resolved(dependency_project.id));
    assert_ne!(main_module.project, provider_module.project);

    let resolved = snapshot
        .module_products
        .resolved_imports
        .iter()
        .find(|((importer, _), _)| importer == main_module)
        .map(|(_, target)| target);
    assert_eq!(
        resolved,
        Some(provider_module),
        "the canonical import product must resolve the dependency alias to its provider module: {:?}",
        snapshot.module_products.resolved_imports
    );

    service.shutdown();
    let _ = fs::remove_dir_all(root);
}
