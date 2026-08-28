use std::sync::Arc;

use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
use phalcom_semantic::SemanticWorkspaceSession;
use tempfile::tempdir;

fn location(path: &std::path::Path) -> SourceLocation {
    SourceLocation {
        source_id: SourceId(path.to_string_lossy().into()),
        display_path: path.to_path_buf(),
    }
}

#[test]
fn semantic_snapshot_publishes_relative_import_alias_path_and_provenance() {
    let root = tempdir().unwrap();
    let main_path = root.path().join("main.ph");
    let shapes_path = root.path().join("shapes.ph");
    let main = location(&main_path);
    let shapes = location(&shapes_path);

    let mut session = SemanticWorkspaceSession::new();
    let publication = session
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: main,
                text: Arc::from("import .shapes as shapes\nfrom .shapes import Circle\nlet c = Circle.new()\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: shapes,
                text: Arc::from("class Circle {}\nexport Circle\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("workspace publication should succeed");

    let queries = publication.snapshot.module_queries();
    let main_module = queries
        .module_for_display_path(&main_path)
        .cloned()
        .expect("main source must map to its canonical module");
    let shapes_module = queries
        .module_for_display_path(&shapes_path)
        .cloned()
        .expect("shapes source must map to its canonical module");

    assert_eq!(
        queries.resolved_import_target(&main_module, "shapes"),
        Some(&shapes_module),
        "whole-module alias must resolve through canonical module products"
    );
    assert_eq!(
        queries.resolved_import_target(&main_module, ".shapes"),
        Some(&shapes_module),
        "written relative import path must resolve through canonical module products"
    );
    assert_eq!(
        queries.definition_source(&shapes_module).map(|source| source.display_path.as_path()),
        Some(shapes_path.as_path()),
        "resolved module target must retain source provenance"
    );
}
