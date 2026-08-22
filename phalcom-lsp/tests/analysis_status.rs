//! Integration tests for LSP `phalcom/analysisStatus` status notifications.

use std::fs;
use std::sync::Arc;

use phalcom_lsp::analysis_service::{AnalysisEvent, AnalysisService, WorkspaceScanRequest};
use phalcom_lsp::analysis_status::AnalysisPhase;
use phalcom_lsp::semantic::SemanticDb;
use phalcom_lsp::workspace_scan::AnalysisMode;

#[test]
fn analysis_status_transitions_and_session_increment() {
    let root = std::env::temp_dir().join(format!("phalcom_lsp_status_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create temp dir");
    fs::write(root.join("main.ph"), "class Main { main() {} }\n").expect("write main file");

    let db = Arc::new(SemanticDb::new());
    let (service, mut rx) = AnalysisService::new(db);

    // Initial status event on worker spawn
    let first = rx.blocking_recv().expect("starting status");
    if let AnalysisEvent::Status(status) = first {
        assert_eq!(status.session, 1);
        assert_eq!(status.phase, AnalysisPhase::Starting);
    } else {
        panic!("expected Status event");
    }

    // Configure workspace for first time -> increments session to 2
    service.configure_workspace(WorkspaceScanRequest {
        roots: vec![root.clone()],
        mode: AnalysisMode::Local,
        excludes: Vec::new(),
        core_source_path: None,
    });

    service.flush();

    let mut session2_statuses = Vec::new();
    loop {
        let event = rx.blocking_recv().expect("expected event");
        if let AnalysisEvent::Status(status) = event {
            let is_ready = status.phase == AnalysisPhase::Ready;
            session2_statuses.push(status);
            if is_ready {
                break;
            }
        }
    }

    assert!(!session2_statuses.is_empty());
    assert!(session2_statuses.iter().any(|s| s.session == 2));

    // Configure workspace again -> increments session to 3
    service.configure_workspace(WorkspaceScanRequest {
        roots: vec![root.clone()],
        mode: AnalysisMode::Workspace,
        excludes: Vec::new(),
        core_source_path: None,
    });

    service.flush();

    let mut session3_statuses = Vec::new();
    loop {
        let event = rx.blocking_recv().expect("expected event");
        if let AnalysisEvent::Status(status) = event {
            let is_ready = status.phase == AnalysisPhase::Ready && status.session == 3;
            let session = status.session;
            session3_statuses.push(status);
            if is_ready || session == 3 {
                break;
            }
        }
    }

    assert!(
        session3_statuses.iter().any(|s| s.session == 3),
        "workspace scan request must increment session to 3"
    );

    service.shutdown();
    let _ = fs::remove_dir_all(root);
}
