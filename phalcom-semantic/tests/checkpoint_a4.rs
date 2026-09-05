use std::sync::Arc;
use tempfile::tempdir;

use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
use phalcom_semantic::session::SemanticWorkspaceSession;

fn location(path: &std::path::Path) -> SourceLocation {
    SourceLocation {
        source_id: SourceId(path.to_string_lossy().into()),
        display_path: path.to_path_buf(),
    }
}

#[test]
fn test_previously_missing_public_name_appears_invalidates_consumer() {
    let root = tempdir().unwrap();
    std::fs::write(root.path().join("package.ph"), "").unwrap();
    let provider_path = root.path().join("provider.ph");
    let consumer_path = root.path().join("consumer.ph");
    let unrelated_path = root.path().join("unrelated.ph");

    let provider = location(&provider_path);
    let consumer = location(&consumer_path);
    let unrelated = location(&unrelated_path);

    let mut session = SemanticWorkspaceSession::new();

    // 1. Initial state: provider exports Foo, consumer references provider.Missing (absent)
    let _pub1 = session
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: provider,
                text: Arc::from("class Foo {}\nexport Foo\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: consumer,
                text: Arc::from("import .provider as Provider\nclass Consumer {\n  @class read() -> Int { let value: Provider.Missing }\n}\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: unrelated,
                text: Arc::from("class Unrelated {\n  @class work() -> Int { 100 }\n}\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("initial setup should succeed");

    // 2. Update provider to add Missing export
    let pub2 = session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: location(&provider_path),
            text: Arc::from("class Foo {}\nclass Missing {}\nexport Foo\nexport Missing\n"),
            revision: SourceRevision(2),
            recovered_program: None,
        }])
        .expect("update should succeed");

    // Consumer type-level read used PublicExport(provider, "Missing") which was Absent. Now Missing appears.
    // Consumer must recompute to pick up Missing.
    assert!(
        pub2.stats.callables_recomputed >= 1,
        "consumer body must recompute when previously missing name appears, stats: {:?}",
        pub2.stats
    );
}

#[test]
fn test_unrelated_export_added_reuses_exact_consumer() {
    let root = tempdir().unwrap();
    std::fs::write(root.path().join("package.ph"), "").unwrap();
    let provider_path = root.path().join("provider.ph");
    let consumer_path = root.path().join("consumer.ph");

    let provider = location(&provider_path);
    let consumer = location(&consumer_path);

    let mut session = SemanticWorkspaceSession::new();

    // 1. Initial state: consumer imports Foo from provider
    let _pub1 = session
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: provider,
                text: Arc::from("class Foo {}\nexport Foo\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: consumer,
                text: Arc::from("import .provider as Provider\nclass Consumer {\n  @class read() -> Int { let value: Provider.Foo }\n}\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("initial setup should succeed");

    // 2. Add Bar export (unrelated export) to provider
    let pub2 = session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: location(&provider_path),
            text: Arc::from("class Foo {}\nclass Bar {}\nexport Foo\nexport Bar\n"),
            revision: SourceRevision(2),
            recovered_program: None,
        }])
        .expect("update should succeed");

    // Consumer depends on PublicExport(provider, "Foo").
    // Adding Bar export does not change PublicExport(provider, "Foo") product fingerprint.
    // Consumer read() body should be reused without recomputation.
    assert_eq!(
        pub2.stats.callables_recomputed, 0,
        "consumer body must not recompute when an unrelated export is added, stats: {:?}",
        pub2.stats
    );
    assert!(
        pub2.stats.callables_reused >= 1,
        "consumer body must be reused, stats: {:?}",
        pub2.stats
    );
}

#[test]
fn test_reexport_retargeting_invalidates_exact_consumer() {
    let root = tempdir().unwrap();
    std::fs::write(root.path().join("package.ph"), "").unwrap();
    let a1_path = root.path().join("a1.ph");
    let a2_path = root.path().join("a2.ph");
    let b_path = root.path().join("b.ph");
    let c_path = root.path().join("c.ph");

    let a1 = location(&a1_path);
    let a2 = location(&a2_path);
    let b = location(&b_path);
    let c = location(&c_path);

    let mut session = SemanticWorkspaceSession::new();

    // 1. Initial state: B re-exports A1.Foo as TargetFoo
    let _pub1 = session
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: a1,
                text: Arc::from("class Foo {}\nexport Foo\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: a2,
                text: Arc::from("class Foo {}\nexport Foo\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: b,
                text: Arc::from("from .a1 import Foo as TargetFoo\nexport TargetFoo\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: c,
                text: Arc::from("import .b as B\nclass Consumer {\n  @class read() -> Int { let value: B.TargetFoo }\n}\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("initial setup should succeed");

    // 2. Change B to re-export A2.Foo as TargetFoo
    let pub2 = session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: location(&b_path),
            text: Arc::from("from .a2 import Foo as TargetFoo\nexport TargetFoo\n"),
            revision: SourceRevision(2),
            recovered_program: None,
        }])
        .expect("update should succeed");

    // Consumer in C depends on PublicExport(b, "TargetFoo").
    // Re-export target changed from A1.Foo to A2.Foo.
    // PublicExport product fingerprint changes, so consumer in C must recompute.
    assert!(
        pub2.stats.callables_recomputed >= 1,
        "consumer body must recompute when re-export target is retargeted, stats: {:?}",
        pub2.stats
    );
}
