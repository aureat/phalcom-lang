//! Standard LSP type-hint rendering over compiler-owned editor semantics.

use phalcom_common::range::SourceRange;
use phalcom_semantic::{AdvisoryConfidence, AdvisoryPresenter, EditorTypeHint, EditorTypeHintKind, FormalPresentation, ValueShape};
use tower_lsp::lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, InlayHintTooltip, MarkupContent, MarkupKind, Range};

use crate::line_index::LineIndex;
use crate::request_context::RequestContext;

/// Server policy for runtime-value inlay hints.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HintPolicy {
    /// Do not render hints.
    Off,
    /// Render stable facts and suppress heuristic facts.
    Stable,
    /// Render all known facts, including heuristic facts.
    All,
}

/// Computes inlay hints from one pinned request context.
pub fn hints_for_request(request: &RequestContext, visible: Range, policy: HintPolicy, suppress_obvious: bool) -> Vec<InlayHint> {
    if policy == HintPolicy::Off {
        return Vec::new();
    }
    let visible_start = request.document.line_index.offset(visible.start);
    let visible_end = request.document.line_index.offset(visible.end);
    let Some(module) = request.compiler_module() else { return Vec::new() };
    let Some(snapshot) = request.compiler.as_deref() else { return Vec::new() };
    if !matches!(request.source_match, crate::request_context::SourceMatch::Exact) {
        return Vec::new();
    }

    let mut hints = snapshot
        .editor()
        .type_hints(module, SourceRange::new(visible_start, visible_end))
        .into_iter()
        .filter_map(|hint| {
            if suppress_obvious
                && hint.kind == EditorTypeHintKind::Binding
                && !matches!(hint.formal.as_ref(), Some(FormalPresentation::Known(_) | FormalPresentation::Dynamic))
                && obvious_initializer_text(&request.document.text, hint.source_range)
            {
                return None;
            }
            render_hint(&request.document.line_index, hint, policy)
        })
        .collect::<Vec<_>>();
    hints.sort_by_key(|hint| (hint.position.line, hint.position.character));
    hints
}

fn obvious_initializer_text(text: &str, range: SourceRange) -> bool {
    let Some(after_range) = text.get(range.end..) else { return false };
    let line_end = after_range.find('\n').map_or(text.len(), |offset| range.end + offset);
    let Some(tail) = text.get(range.end..line_end) else { return false };
    let Some(equal) = tail.find('=') else { return false };
    let Some(after_equal) = tail.get(equal + 1..) else { return false };
    let value = after_equal.trim_start();
    value.starts_with('"')
        || value.starts_with('\'')
        || value.chars().next().is_some_and(|character| character.is_ascii_digit() || character == '-')
        || value.starts_with("true")
        || value.starts_with("false")
}

fn render_hint(line_index: &LineIndex, hint: EditorTypeHint, policy: HintPolicy) -> Option<InlayHint> {
    let return_hint = hint.kind == EditorTypeHintKind::Return;
    let formal_text = hint.formal.as_ref().and_then(|presentation| match presentation {
        FormalPresentation::Known(_) | FormalPresentation::Dynamic => Some(presentation.text()),
        _ => None,
    });

    let (label, tooltip) = if let Some(text) = formal_text {
        (crate::presentation::inlay_type_label(&text, return_hint), None)
    } else {
        if hint.formal.is_some() && !matches!(hint.formal.as_ref(), Some(FormalPresentation::Unknown)) {
            return None;
        }
        let fact = hint.advisory.as_ref()?;
        if matches!(fact.shape, ValueShape::Unknown) || (policy == HintPolicy::Stable && matches!(fact.confidence, AdvisoryConfidence::Heuristic)) {
            return None;
        }
        let rendered = AdvisoryPresenter::present_shape(&fact.shape);
        (
            crate::presentation::inlay_type_label(&rendered, return_hint),
            Some(crate::presentation::advisory_tooltip(
                &rendered,
                if return_hint { "return value" } else { "runtime value" },
            )),
        )
    };

    Some(InlayHint {
        position: line_index.position(hint.insertion_offset),
        label: InlayHintLabel::String(label),
        kind: Some(InlayHintKind::TYPE),
        text_edits: None,
        tooltip: tooltip.map(|value| {
            InlayHintTooltip::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value,
            })
        }),
        padding_left: Some(true),
        padding_right: None,
        data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::obvious_initializer_text;
    use phalcom_common::range::SourceRange;

    #[test]
    fn review_m1_01_ascii_initializer_is_detected_after_range() {
        assert!(obvious_initializer_text("let count = 42\n", SourceRange::new(0, 9)));
    }

    #[test]
    fn review_m1_02_utf8_before_initializer_keeps_byte_range_safe() {
        assert!(obvious_initializer_text("let café = 42\n", SourceRange::new(0, 10)));
    }

    #[test]
    fn review_m1_03_interior_utf8_offset_never_panics() {
        let result = std::panic::catch_unwind(|| obvious_initializer_text("let café = 42\n", SourceRange::new(0, 9)));
        assert!(result.is_ok(), "initializer suppression must not panic on an invalid UTF-8 boundary");
    }

    #[test]
    fn review_m1_04_out_of_bounds_range_is_safe() {
        let result = std::panic::catch_unwind(|| obvious_initializer_text("let value = 1", SourceRange::new(0, 100)));
        assert!(result.is_ok());
        assert!(!result.expect("closure result"));
    }

    #[test]
    fn review_m1_05_non_literal_initializer_is_not_obvious() {
        assert!(!obvious_initializer_text("let value = other\n", SourceRange::new(0, 10)));
    }
}
