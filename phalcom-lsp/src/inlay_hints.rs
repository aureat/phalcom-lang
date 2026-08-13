//! Standard LSP runtime-value inlay hints.

use phalcom_common::range::SourceRange;
use tower_lsp::lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, InlayHintParams, InlayHintTooltip, MarkupContent, MarkupKind, Range, Url};

use crate::documents::Document;
use crate::semantic::{Confidence, SemanticDb, ValueShape, SemanticBindingKind};

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

/// Computes stable type-style hints for visible top-level binding declarations.
pub fn hints_for(db: &SemanticDb, uri: &Url, doc: &Document, visible: Range) -> Vec<InlayHint> {
    hints_for_with_policy(db, uri, doc, visible, HintPolicy::Stable, false)
}

/// Computes runtime-value hints under an explicit display policy.
pub fn hints_for_with_policy(db: &SemanticDb, uri: &Url, doc: &Document, visible: Range, policy: HintPolicy, suppress_obvious: bool) -> Vec<InlayHint> {
    if policy == HintPolicy::Off {
        return Vec::new();
    }
    let visible_start = doc.line_index.offset(visible.start);
    let visible_end = doc.line_index.offset(visible.end);
    let Some(snapshot) = db.file_snapshot(uri) else {
        return Vec::new();
    };
    let mut hints = Vec::new();
    for binding in snapshot.scopes.bindings.values() {
        if binding.kind == SemanticBindingKind::Import {
            continue;
        }
        let range = binding.declaration_range;
        if range.end < visible_start || range.start > visible_end {
            continue;
        }
        let Some(value) = snapshot.local_facts.value_before(binding.id, range.end.saturating_add(1)) else {
            continue;
        };
        if !should_render(policy, &value.confidence, &value.shape) || (suppress_obvious && obvious_initializer(doc, range)) {
            continue;
        }
        let rendered = render_shape(&value.shape);
        let position = doc.line_index.position(range.end);
        hints.push(InlayHint {
            position,
            label: InlayHintLabel::String(format!(": {rendered}")),
            kind: Some(InlayHintKind::TYPE),
            text_edits: None,
            tooltip: Some(InlayHintTooltip::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "Inferred runtime value: {rendered}\n\nConfidence: {}\n\nThis is editor inference, not a Phalcom type annotation.",
                    confidence_name(value.confidence)
                ),
            })),
            padding_left: Some(true),
            padding_right: None,
            data: None,
        });
    }
    hints.sort_by_key(|hint| (hint.position.line, hint.position.character));
    hints
}

/// Answers an inlay-hint request using an open document snapshot.
pub fn hints_for_params(db: &SemanticDb, uri: &Url, doc: &Document, params: &InlayHintParams) -> Vec<InlayHint> {
    hints_for(db, uri, doc, params.range)
}

/// Answers an inlay request under an explicit display policy.
pub fn hints_for_params_with_policy(
    db: &SemanticDb,
    uri: &Url,
    doc: &Document,
    params: &InlayHintParams,
    policy: HintPolicy,
    suppress_obvious: bool,
) -> Vec<InlayHint> {
    hints_for_with_policy(db, uri, doc, params.range, policy, suppress_obvious)
}

fn should_render(policy: HintPolicy, confidence: &Confidence, shape: &ValueShape) -> bool {
    !matches!(shape, ValueShape::Unknown) && (policy == HintPolicy::All || !matches!(confidence, Confidence::Heuristic))
}

fn obvious_initializer(doc: &Document, range: SourceRange) -> bool {
    let line_end = doc.text[range.end..].find('\n').map_or(doc.text.len(), |offset| range.end + offset);
    let tail = &doc.text[range.end..line_end];
    let Some(equal) = tail.find('=') else { return false };
    let value = tail[equal + 1..].trim_start();
    value.starts_with('"')
        || value.starts_with('\'')
        || value.chars().next().is_some_and(|character| character.is_ascii_digit() || character == '-')
        || value.starts_with("true")
        || value.starts_with("false")
}

fn render_shape(shape: &ValueShape) -> String {
    crate::semantic::render_value_shape(shape)
}

fn confidence_name(confidence: Confidence) -> &'static str {
    crate::semantic::confidence_name(confidence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::FileRevision;
    use tower_lsp::lsp_types::Position;

    #[test]
    fn literal_binding_gets_standard_type_hint() {
        let uri = Url::parse("file:///main.ph").unwrap();
        let doc = Document::new("let text = \"hello\"\n".to_string());
        let db = SemanticDb::new();
        db.update_file(&uri, FileRevision(1), &doc.parse.program);
        let hints = hints_for(
            &db,
            &uri,
            &doc,
            Range {
                start: Position::new(0, 0),
                end: Position::new(10, 0),
            },
        );
        assert_eq!(hints.len(), 1);
        assert!(matches!(&hints[0].label, InlayHintLabel::String(label) if label == ": String"));
        assert_eq!(hints[0].kind, Some(InlayHintKind::TYPE));
    }

    #[test]
    fn unknown_values_are_hidden() {
        let uri = Url::parse("file:///main.ph").unwrap();
        let doc = Document::new("let value = missing()\n".to_string());
        let db = SemanticDb::new();
        db.update_file(&uri, FileRevision(1), &doc.parse.program);
        let hints = hints_for(
            &db,
            &uri,
            &doc,
            Range {
                start: Position::new(0, 0),
                end: Position::new(10, 0),
            },
        );
        assert!(hints.is_empty());
    }

    #[test]
    fn method_local_binding_gets_standard_type_hint() {
        let uri = Url::parse("file:///main.ph").unwrap();
        let doc = Document::new("class Canvas { draw() { let width = 1 } }\n".to_string());
        let db = SemanticDb::new();
        db.update_file(&uri, FileRevision(1), &doc.parse.program);
        let hints = hints_for(
            &db,
            &uri,
            &doc,
            Range {
                start: Position::new(0, 0),
                end: Position::new(10, 0),
            },
        );
        assert_eq!(hints.len(), 1);
        assert!(matches!(&hints[0].label, InlayHintLabel::String(label) if label == ": Int"));
    }
}
