//! Integration tests for LSP `phalcom/analysisLog` structured notifications.

use phalcom_lsp::analysis_service::{AnalysisEvent, AnalysisService, WorkspaceScanRequest};
use phalcom_lsp::analysis_status::AnalysisPhase;
use phalcom_lsp::workspace_scan::AnalysisMode;
use phalcom_modules::SourceRevision;
use std::fs;
use std::sync::Arc;

#[test]
fn structured_analysis_log_events_emitted_with_session_sequence() {
    let root = std::env::temp_dir().join(format!("phalcom_lsp_log_test_{}", std::process::id()));
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

    let mut logs = Vec::new();
    loop {
        let event = rx.blocking_recv().expect("expected event");
        match event {
            AnalysisEvent::Log(log) => {
                logs.push(log);
            }
            AnalysisEvent::Status(status) if status.phase == AnalysisPhase::Ready && status.session == 2 => {
                break;
            }
            _ => {}
        }
    }

    assert!(!logs.is_empty(), "expected structured log events");
    assert!(
        logs.iter().any(|l| l.event == "workspace.session.started" && l.session == 2),
        "expected workspace.session.started log event"
    );
    assert!(logs.iter().any(|l| l.event == "core.surface.loaded"), "expected core.surface.loaded log event");

    // Now edit file
    let uri = tower_lsp::lsp_types::Url::from_file_path(&file_path).unwrap();
    let source = "class Main { main() { let y = 100; } }\n";
    let program = phalcom_ast::parse(source, 0).program;
    service.mark_open(uri.clone());
    let text: Arc<str> = Arc::from(source);
    service.enqueue_file_update(uri, SourceRevision(2), text, Arc::new(program));
    service.flush();

    while let Ok(event) = rx.try_recv() {
        if let AnalysisEvent::Log(log) = event {
            logs.push(log);
        }
    }

    assert!(
        logs.iter().any(|l| l.event == "semantic.batch.started"),
        "expected semantic.batch.started log event"
    );
    assert!(logs.iter().any(|l| l.event == "snapshot.published"), "expected snapshot.published log event");

    for log in &logs {
        assert!(log.session >= 1, "session must be >= 1");
        assert!(log.sequence >= 1, "sequence must be >= 1");
    }

    service.shutdown();
    let _ = fs::remove_dir_all(root);
}
