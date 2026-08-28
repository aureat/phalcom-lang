use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceMutation};
use phalcom_semantic::SemanticWorkspaceSession;
use std::fs;
use std::sync::Arc;
use tower_lsp::lsp_types::Url;

#[test]
fn compiler_session_keeps_module_identity_across_overlay_edits() {
    let root = std::env::temp_dir().join(format!("phalcom_lsp_single_world_{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let path = root.join("main.ph");
    fs::write(&path, "class Main {}\n").unwrap();
    let location = SourceLocation {
        source_id: SourceId(path.to_string_lossy().into()),
        display_path: path.clone(),
    };
    let mut session = SemanticWorkspaceSession::new();

    let first = session
        .apply_module_mutation(WorkspaceSourceMutation::SetOverlay {
            source: location.clone(),
            text: Arc::from("class Main {}\n"),
            revision: SourceRevision(1),
        })
        .unwrap();
    let module = session.module_session().module_for_source(&location.source_id).unwrap().clone();
    let first_snapshot = first.snapshot;

    let second = session
        .apply_module_mutation(WorkspaceSourceMutation::SetOverlay {
            source: location,
            text: Arc::from("class Main { run() {} }\n"),
            revision: SourceRevision(2),
        })
        .unwrap();
    let second_snapshot = second.snapshot;

    assert_eq!(
        session.module_session().module_for_source(&SourceId(path.to_string_lossy().into())),
        Some(&module)
    );
    assert!(second_snapshot.generation > first_snapshot.generation);
    assert_eq!(&*second_snapshot.sources.get(&module).unwrap().text, "class Main { run() {} }\n");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn worker_reuses_compiler_snapshot_store_across_edits() {
    let path = std::env::temp_dir().join(format!("phalcom_lsp_compiler_store_{}.ph", std::process::id()));
    let uri = Url::from_file_path(&path).unwrap();
    let (service, _events) = phalcom_lsp::analysis_service::AnalysisService::new();

    let first_text: Arc<str> = Arc::from("class Main { run() {} }\n");
    service.enqueue_file_update(
        uri.clone(),
        SourceRevision(1),
        first_text.clone(),
        Arc::new(phalcom_ast::parser::parse(&first_text, 0).program),
    );
    service.flush();
    let first = service.snapshot().expect("compiler publication");
    let module = first.sources.keys().next().cloned().expect("module publication");
    let store = first.store.id();

    let second_text: Arc<str> = Arc::from("class Main { run() { } edit() {} }\n");
    service.enqueue_file_update(
        uri,
        SourceRevision(2),
        second_text.clone(),
        Arc::new(phalcom_ast::parser::parse(&second_text, 0).program),
    );
    service.flush();
    let second = service.snapshot().expect("second compiler publication");

    assert_eq!(second.store.id(), store, "ordinary edits retain one compiler TypeStore");
    assert!(second.sources.contains_key(&module), "ordinary edits retain canonical module identity");
    assert_ne!(first.id, second.id, "edits publish a new immutable snapshot");
    service.shutdown();
    let _ = fs::remove_file(path);
}
