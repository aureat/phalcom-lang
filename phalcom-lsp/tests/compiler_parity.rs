//! Assertion coverage for the compiler/LSP parity evidence channel.

use phalcom_lsp::parity::{ParityMismatch, ParitySurface, ShadowParityHarness};

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
