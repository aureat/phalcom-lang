//! Layout and column calculation for diagnostic labels.

use crate::snippet::Label;

/// A formatted label line representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabelLine {
    pub col: usize,
    pub len: usize,
    pub text: String,
    pub is_primary: bool,
}

/// Computes label layouts for a source line.
pub fn layout_labels(_source: &str, labels: &[Label<'_>]) -> Vec<LabelLine> {
    labels
        .iter()
        .map(|l| LabelLine {
            col: l.span.start,
            len: l.span.end.saturating_sub(l.span.start).max(1),
            text: l.text.to_string(),
            is_primary: matches!(l.kind, crate::snippet::LabelKind::Primary),
        })
        .collect()
}
