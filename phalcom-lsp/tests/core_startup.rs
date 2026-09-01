//! Universe startup depth and demand-driven analysis coverage.

use phalcom_lsp::analysis_service::{AnalysisEvent, AnalysisService, WorkspaceScanRequest};
use phalcom_lsp::analysis_status::AnalysisPhase;
use phalcom_lsp::workspace_scan::AnalysisMode;
use std::fs;

#[test]
fn startup_does_not_solve_callable_bodies_for_empty_workspace() {
    let root = std::env::temp_dir().join(format!("phalcom_lsp_core_startup_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create empty workspace root");

    let (service, mut events) = AnalysisService::new();
    let _ = events.blocking_recv().expect("initial status event");

    service.configure_workspace(WorkspaceScanRequest {
        roots: vec![root.clone()],
        mode: AnalysisMode::Local,
        excludes: Vec::new(),
    });
    service.flush();

    let mut saw_ready = false;
    while let Ok(event) = events.try_recv() {
        if let AnalysisEvent::Status(status) = event {
            saw_ready |= status.phase == AnalysisPhase::Ready;
        }
    }

    assert!(saw_ready, "startup must reach Ready");
    assert!(service.snapshot().is_none(), "empty workspace must not publish a source snapshot");
    assert_eq!(
        service.perf_counters().snapshot().callables_analyzed,
        0,
        "startup must not solve core callable bodies"
    );

    service.shutdown();
    let _ = fs::remove_dir_all(root);
}
