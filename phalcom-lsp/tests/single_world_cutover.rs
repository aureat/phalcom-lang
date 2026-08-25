use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceMutation};
use phalcom_semantic::SemanticWorkspaceSession;
use std::fs;
use std::sync::Arc;

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
