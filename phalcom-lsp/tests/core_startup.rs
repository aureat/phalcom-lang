//! Core startup depth and demand-driven analysis coverage.

use std::fs;
use std::sync::Arc;

use phalcom_lsp::analysis_service::{AnalysisEvent, AnalysisService, WorkspaceScanRequest};
use phalcom_lsp::analysis_status::AnalysisPhase;
use phalcom_lsp::semantic::SemanticDb;
use phalcom_lsp::workspace_scan::AnalysisMode;
use tower_lsp::lsp_types::Url;

#[test]
fn startup_publishes_core_surface_without_solving_callable_bodies() {
    let root = std::env::temp_dir().join(format!("phalcom_lsp_core_startup_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create empty workspace root");

    let db = Arc::new(SemanticDb::new());
    let (service, mut events) = AnalysisService::new(db.clone());
    let _ = events.blocking_recv().expect("initial status event");

    service.configure_workspace(WorkspaceScanRequest {
        roots: vec![root.clone()],
        mode: AnalysisMode::Local,
        excludes: Vec::new(),
        core_source_path: None,
    });
    service.flush();

    let mut saw_ready = false;
    while let Ok(event) = events.try_recv() {
        if let AnalysisEvent::Status(status) = event {
            saw_ready |= status.phase == AnalysisPhase::Ready;
        }
    }

    assert!(saw_ready, "startup must reach Ready");
    let snapshot = db.snapshot();
    assert!(!snapshot.classes.is_empty(), "startup must publish core declarations");
    assert!(snapshot.summaries.is_empty(), "startup must not publish eager core callable summaries");
    assert_eq!(
        service.perf_counters().snapshot().callables_analyzed,
        0,
        "startup must not solve core callable bodies"
    );

    service.shutdown();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn opening_selected_core_source_allows_explicit_deep_analysis() {
    let root = std::env::temp_dir().join(format!("phalcom_lsp_core_deep_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create workspace root");
    let core_path = root.join("core.ph");
    let source = "class Core { value() { 1 } }\n";
    fs::write(&core_path, source).expect("write core source");

    let db = Arc::new(SemanticDb::new());
    let (service, mut events) = AnalysisService::new(db.clone());
    let _ = events.blocking_recv().expect("initial status event");
    service.configure_workspace(WorkspaceScanRequest {
        roots: vec![root.clone()],
        mode: AnalysisMode::Local,
        excludes: Vec::new(),
        core_source_path: Some(core_path.clone()),
    });
    service.flush();
    assert_eq!(service.perf_counters().snapshot().callables_analyzed, 0, "startup must remain surface-only");

    let core_uri = Url::from_file_path(&core_path).expect("core URI");
    service.mark_open(core_uri);
    service.enqueue_core_update(phalcom_lsp::semantic::FileRevision(2), phalcom_ast::parse(source, 0).program);
    service.flush();

    assert!(
        service.perf_counters().snapshot().callables_analyzed > 0,
        "opening core source must permit deep analysis: {:?}",
        service.perf_counters().snapshot()
    );

    service.shutdown();
    let _ = fs::remove_dir_all(root);
}
