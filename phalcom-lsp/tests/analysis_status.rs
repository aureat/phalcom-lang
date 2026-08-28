//! Integration tests for LSP `phalcom/analysisStatus` status notifications.

use phalcom_lsp::analysis_service::{AnalysisEvent, AnalysisService, WorkspaceScanRequest};
use phalcom_lsp::analysis_status::AnalysisPhase;
use phalcom_lsp::workspace_scan::AnalysisMode;
use phalcom_modules::SourceRevision;
use std::fs;
use std::sync::Arc;

#[test]
fn analysis_status_transitions_and_session_increment() {
    let root = std::env::temp_dir().join(format!("phalcom_lsp_status_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create temp dir");
    fs::write(root.join("main.ph"), "class Main { main() {} }\n").expect("write main file");

    let (service, mut rx) = AnalysisService::new();

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

#[test]
fn edit_only_batch_returns_to_ready_after_publication() {
    let root = std::env::temp_dir().join(format!("phalcom_lsp_status_edit_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create temp dir");
    let file_path = root.join("main.ph");
    fs::write(&file_path, "class Main { main() {} }\n").expect("write main file");

    let (service, mut rx) = AnalysisService::new();

    service.configure_workspace(WorkspaceScanRequest {
        roots: vec![root.clone()],
        mode: AnalysisMode::Local,
        excludes: Vec::new(),
        core_source_path: None,
    });
    service.flush();

    // Drain until initial Ready
    loop {
        let event = rx.blocking_recv().expect("expected event");
        if let AnalysisEvent::Status(status) = event {
            if status.phase == AnalysisPhase::Ready && status.session == 2 {
                break;
            }
        }
    }

    // Now enqueue an edit-only file update
    let uri = tower_lsp::lsp_types::Url::from_file_path(&file_path).unwrap();
    let source = "class Main { main() { let x = 42; } }\n";
    let program = phalcom_ast::parse(source, 0).program;
    service.mark_open(uri.clone());
    let text: Arc<str> = Arc::from(source);
    service.enqueue_file_update(uri, SourceRevision(2), text, Arc::new(program));
    service.flush();

    let mut statuses = Vec::new();
    let mut saw_publishing = false;
    let mut final_status = None;

    // Collect status events until flush finishes
    while let Ok(event) = rx.try_recv() {
        if let AnalysisEvent::Status(status) = event {
            if status.phase == AnalysisPhase::Publishing {
                saw_publishing = true;
            }
            final_status = Some(status.clone());
            statuses.push(status);
        }
    }

    assert!(saw_publishing, "expected to observe Publishing phase during edit batch");
    let last = final_status.expect("expected at least one status update after edit");
    assert_eq!(
        last.phase,
        AnalysisPhase::Ready,
        "edit-only batch must finish in Ready, not stuck in Publishing: last was {:?}",
        last
    );
    assert!(last.complete, "final status must have complete == true");

    service.shutdown();
    let _ = fs::remove_dir_all(root);
}
