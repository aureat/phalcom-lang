//! Assertion coverage for the compiler/LSP parity evidence channel.

use phalcom_lsp::parity::{ParityMismatch, ParitySurface, ShadowParityHarness};
use tower_lsp::lsp_types::Position;

mod support;

#[test]
fn parity_harness_exposes_structured_mismatch_evidence() {
    let harness = ShadowParityHarness::new();
    harness.record_hover_parity("value", Some("Int"), Some("String"));

    assert_eq!(
        harness.mismatches(),
        vec![ParityMismatch {
            surface: ParitySurface::Hover,
            target_name: "value".into(),
            formal: Some("Int".into()),
            advisory: Some("String".into()),
        }]
    );
}

#[tokio::test]
async fn live_hover_feeds_formal_and_advisory_parity_observation() {
    let root = std::env::temp_dir().join(format!("phalcom-lsp-live-parity-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("create parity workspace");
    std::fs::write(root.join("package.ph"), "").expect("write parity package marker");
    let root_uri = tower_lsp::lsp_types::Url::from_directory_path(&root).expect("parity workspace URI").to_string();
    let uri = tower_lsp::lsp_types::Url::from_file_path(root.join("main.ph"))
        .expect("parity source URI")
        .to_string();
    let mut lsp = support::TestLsp::start().await;
    lsp.initialize(Some(&root_uri)).await;
    lsp.open_and_wait(&uri, "class Point {\n  value() -> Int { 1 }\n}\n").await;

    let response = lsp.hover(&uri, Position::new(1, 3)).await;
    assert!(!response.is_null(), "the live compiler hover query should resolve the callable: {response:#?}");

    let observations = lsp.parity_observations();
    assert!(
        observations
            .iter()
            .any(|observation| observation.surface == ParitySurface::Hover && observation.target_name.contains("value")),
        "the production hover path must feed the parity harness: {observations:#?}"
    );

    lsp.finish().await;
    let _ = std::fs::remove_dir_all(root);
}
