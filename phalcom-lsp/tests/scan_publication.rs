use phalcom_lsp::analysis_service::{AnalysisEvent, AnalysisService, WorkspaceScanRequest};
use phalcom_lsp::workspace_scan::AnalysisMode;
use std::fs;

#[test]
fn workspace_scan_publishes_its_snapshot_to_request_readers() {
    let root = std::env::temp_dir().join(format!("phalcom_lsp_scan_publication_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create workspace root");
    fs::write(root.join("main.ph"), "class Main {}\nexport Main\n").expect("write source file");

    let (service, mut events) = AnalysisService::new();
    let _ = events.blocking_recv().expect("initial status event");

    service.configure_workspace(WorkspaceScanRequest {
        roots: vec![root.clone()],
        mode: AnalysisMode::Local,
        excludes: Vec::new(),
    });
    service.flush();

    let snapshot = service.snapshot().expect("scan must publish a request-visible snapshot");
    assert!(
        snapshot.sources.values().any(|source| source.text.as_ref() == "class Main {}\nexport Main\n"),
        "published snapshot must contain the discovered source"
    );
    assert!(service.perf_counters().snapshot().scan_batches_published > 0);

    let mut published_generations = Vec::new();
    while let Ok(event) = events.try_recv() {
        if let AnalysisEvent::Published { generation, .. } = event {
            published_generations.push(generation);
        }
    }
    assert!(
        published_generations.contains(&snapshot.generation),
        "scan event must name the visible snapshot generation"
    );
    service.shutdown();
    let _ = fs::remove_dir_all(root);
}
