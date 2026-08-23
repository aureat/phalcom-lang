use phalcom_common::range::SourceRange;
use phalcom_modules::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::types::id::TypeId;
use phalcom_semantic::{DiagnosticCode, DiagnosticSeverity, SemanticDiagnostic, SemanticRevision, SnapshotId, SnapshotTypeRef, TypeStoreId, WorkspaceId};

fn module(name: &str) -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]),
    )
}

#[test]
fn snapshot_identity_keeps_workspace_revision_and_store_domains_distinct() {
    let workspace = WorkspaceId::from_raw(4);
    let revision = SemanticRevision::from_raw(9);
    let store = TypeStoreId::from_raw(12);
    let snapshot = SnapshotId::new(workspace, revision, store);
    let type_ref = SnapshotTypeRef::new(store, TypeId(3));

    assert_eq!(snapshot.workspace(), workspace);
    assert_eq!(snapshot.revision(), revision);
    assert_eq!(snapshot.store(), store);
    assert_eq!(type_ref.store(), store);
    assert_eq!(type_ref.id(), TypeId(3));
    assert_ne!(snapshot, SnapshotId::new(workspace, SemanticRevision::from_raw(10), store));
}

#[test]
fn diagnostic_primary_and_related_spans_own_their_modules() {
    let first = module("first");
    let second = module("second");
    let diagnostic = SemanticDiagnostic::error_in(
        first.clone(),
        DiagnosticCode::ModuleImportUnresolved,
        "import target is unresolved",
        SourceRange { start: 2, end: 4 },
    )
    .with_label_in(second.clone(), SourceRange { start: 8, end: 10 }, "declared in another module");

    assert_eq!(diagnostic.primary.module, first);
    assert_eq!(diagnostic.primary.range, diagnostic.primary_range);
    assert_eq!(diagnostic.labels[0].span.module, second);
    assert_eq!(diagnostic.labels[0].span.range, diagnostic.labels[0].range);
}

#[test]
fn explicit_diagnostic_constructor_keeps_range_and_severity_behavior() {
    let diagnostic = SemanticDiagnostic::error_in(
        module("source"),
        DiagnosticCode::BindingInitializerMismatch,
        "incompatible initializer",
        (3..6).into(),
    )
    .with_label((10..12).into(), "declared type");

    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert_eq!(diagnostic.primary_range, (3..6).into());
    assert_eq!(diagnostic.labels[0].range, (10..12).into());
}

#[test]
fn foundation_codes_have_stable_wire_names() {
    assert_eq!(DiagnosticCode::ProjectLoadFailed.as_str(), "project.load.failed");
    assert_eq!(DiagnosticCode::ModuleLinkFailed.as_str(), "module.link.failed");
    assert_eq!(DiagnosticCode::AnalysisBlocked.as_str(), "analysis.blocked");
    assert_eq!(DiagnosticCode::AnalysisBudgetExceeded.as_str(), "analysis.budget_exceeded");
    assert_eq!(DiagnosticCode::TypeDynamicBoundary.as_str(), "type.dynamic_boundary");
}
