//! Release-facing semantic presentation policy tests.

use phalcom_lsp::presentation::{advisory_hover, advisory_tooltip, inlay_type_label};

#[test]
fn formal_and_advisory_types_use_plain_language_labels() {
    let rendered = [
        inlay_type_label("User", false),
        inlay_type_label("Result<Data, Error>", true),
        advisory_hover("Observed return", "User"),
    ]
    .join("\n");
    assert!(rendered.contains(": User"));
    assert!(rendered.contains(" -> Result<Data, Error>"));
    assert!(rendered.contains("`User`"));
    assert!(!rendered.contains('≈'));
    assert!(!rendered.contains("Confidence:"));
    assert!(!rendered.contains("Observed type: ≈"));
}

#[test]
fn advisory_evidence_remains_available_as_contextual_tooltip() {
    let tooltip = advisory_tooltip("User", "runtime value");
    assert!(tooltip.contains("`User`"));
    assert!(tooltip.contains("Inferred from local flow."));
    assert!(!tooltip.contains('≈'));
    assert!(!tooltip.contains("Confidence:"));
}
