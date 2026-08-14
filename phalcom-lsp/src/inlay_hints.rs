//! Standard LSP runtime-value inlay hints.

use phalcom_ast::ast::{Expr, Pattern, Statement};
use phalcom_common::range::SourceRange;
use tower_lsp::lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, InlayHintParams, InlayHintTooltip, MarkupContent, MarkupKind, Range, Url};

use crate::documents::{Document, DocumentSnapshot};
use crate::request_context::RequestContext;
use crate::semantic::{Confidence, SemanticBindingKind, SemanticDb, ValueShape};

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
        return shallow_hints(doc, uri, visible_start, visible_end, policy, suppress_obvious);
    };
    if snapshot.revision != doc.revision {
        return Vec::new();
    }
    let mut hints = Vec::new();
    for binding in snapshot.source.scopes.bindings.values() {
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

/// Computes inlay hints from one pinned request context.
pub fn hints_for_request(request: &RequestContext, visible: Range, policy: HintPolicy, suppress_obvious: bool) -> Vec<InlayHint> {
    if policy == HintPolicy::Off {
        return Vec::new();
    }
    let visible_start = request.document.line_index.offset(visible.start);
    let visible_end = request.document.line_index.offset(visible.end);
    let Some(module) = request.module.as_ref() else {
        return shallow_hints_snapshot(&request.document, request.module.as_ref(), visible_start, visible_end, policy, suppress_obvious);
    };
    let Some(snapshot) = request.semantic.file(module) else {
        return shallow_hints_snapshot(&request.document, request.module.as_ref(), visible_start, visible_end, policy, suppress_obvious);
    };
    if snapshot.revision != request.document.revision {
        return Vec::new();
    };
    let mut hints = Vec::new();
    for binding in snapshot.source.scopes.bindings.values() {
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
        if !should_render(policy, &value.confidence, &value.shape)
            || (suppress_obvious && obvious_initializer_text(&request.document.text, range))
        {
            continue;
        }
        let rendered = render_shape(&value.shape);
        hints.push(InlayHint {
            position: request.document.line_index.position(range.end),
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

fn shallow_hints_snapshot(
    doc: &DocumentSnapshot,
    module: Option<&crate::semantic::ModuleId>,
    visible_start: usize,
    visible_end: usize,
    policy: HintPolicy,
    suppress_obvious: bool,
) -> Vec<InlayHint> {
    let module = module.cloned().unwrap_or_else(|| crate::semantic::ModuleId::new("phalcom://request"));
    let mut hints = Vec::new();
    for statement in &doc.parse.program.statements {
        let Statement::Let(binding) = statement else { continue };
        let Pattern::Name { .. } = &binding.pattern else { continue };
        let Some(value) = binding.value.as_ref() else { continue };
        let Some(shape) = shallow_expression_shape(value, &module) else { continue };
        if binding.range.end < visible_start || binding.range.start > visible_end || !should_render(policy, &Confidence::Exact, &shape) {
            continue;
        }
        if suppress_obvious && obvious_initializer_text(&doc.text, binding.range) {
            continue;
        }
        let rendered = render_shape(&shape);
        hints.push(InlayHint {
            position: doc.line_index.position(binding.range.end),
            label: InlayHintLabel::String(format!(": {rendered}")),
            kind: Some(InlayHintKind::TYPE),
            text_edits: None,
            tooltip: Some(InlayHintTooltip::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("Inferred runtime value: {rendered}\n\nConfidence: exact\n\nThis is editor inference, not a Phalcom type annotation."),
            })),
            padding_left: Some(true),
            padding_right: None,
            data: None,
        });
    }
    hints.sort_by_key(|hint| (hint.position.line, hint.position.character));
    hints
}

/// Produces exact source-local facts while the worker has not published the
/// first semantic file snapshot. Once a snapshot exists, callers must use its
/// revision-matched facts or receive no hints.
fn shallow_hints(doc: &Document, uri: &Url, visible_start: usize, visible_end: usize, policy: HintPolicy, suppress_obvious: bool) -> Vec<InlayHint> {
    let module = crate::semantic::ModuleId::from_uri(uri);
    let mut hints = Vec::new();
    for statement in &doc.parse.program.statements {
        let Statement::Let(binding) = statement else { continue };
        let Pattern::Name { .. } = &binding.pattern else { continue };
        let Some(value) = binding.value.as_ref() else { continue };
        let Some(shape) = shallow_expression_shape(value, &module) else { continue };
        if binding.range.end < visible_start || binding.range.start > visible_end || !should_render(policy, &Confidence::Exact, &shape) {
            continue;
        }
        if suppress_obvious && obvious_initializer(doc, binding.range) {
            continue;
        }
        let rendered = render_shape(&shape);
        hints.push(InlayHint {
            position: doc.line_index.position(binding.range.end),
            label: InlayHintLabel::String(format!(": {rendered}")),
            kind: Some(InlayHintKind::TYPE),
            text_edits: None,
            tooltip: Some(InlayHintTooltip::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("Inferred runtime value: {rendered}\n\nConfidence: exact\n\nThis is editor inference, not a Phalcom type annotation."),
            })),
            padding_left: Some(true),
            padding_right: None,
            data: None,
        });
    }
    hints.sort_by_key(|hint| (hint.position.line, hint.position.character));
    hints
}

fn shallow_expression_shape(expr: &Expr, module: &crate::semantic::ModuleId) -> Option<ValueShape> {
    let core = |name| {
        ValueShape::Instance(crate::semantic::ClassId::new(
            crate::semantic::ModuleId::new(crate::semantic::CORE_MODULE_URI),
            name,
        ))
    };
    match expr {
        Expr::Int { .. } => Some(core("Int")),
        Expr::Float { .. } => Some(core("Float")),
        Expr::String { .. } => Some(core("String")),
        Expr::Boolean { .. } => Some(core("Bool")),
        Expr::MethodCall(call) if call.method == "new" => match &call.object {
            Expr::Var { value, .. } => Some(ValueShape::Instance(crate::semantic::ClassId::new(module.clone(), value.clone()))),
            _ => None,
        },
        _ => None,
    }
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
    obvious_initializer_text(&doc.text, range)
}

fn obvious_initializer_text(text: &str, range: SourceRange) -> bool {
    let line_end = text[range.end..].find('\n').map_or(text.len(), |offset| range.end + offset);
    let tail = &text[range.end..line_end];
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
    fn revision_mismatch_rejects_semantic_hints_for_stale_document_range() {
        let uri = Url::parse("file:///stale-range.ph").unwrap();
        let db = SemanticDb::new();
        let published = Document::new_with_revision("let value = 1\n".to_string(), FileRevision(1));
        db.update_file(&uri, published.revision, &published.parse.program);
        let current = Document::new_with_revision("let value = 1\nlet newer = 2\n".to_string(), FileRevision(2));

        let hints = hints_for(
            &db,
            &uri,
            &current,
            Range {
                start: Position::new(0, 0),
                end: Position::new(2, 0),
            },
        );

        assert!(hints.is_empty(), "stale semantic facts must not produce hints in current range");
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

    #[test]
    fn stale_document_revision_returns_no_semantic_hints() {
        let uri = Url::parse("file:///main.ph").unwrap();
        let published = Document::new_with_revision("let text = \"hello\"\n".to_string(), FileRevision(1));
        let live = Document::new_with_revision("let text = \"changed\"\n".to_string(), FileRevision(2));
        let db = SemanticDb::new();
        db.update_file(&uri, FileRevision(1), &published.parse.program);

        let hints = hints_for(
            &db,
            &uri,
            &live,
            Range {
                start: Position::new(0, 0),
                end: Position::new(10, 0),
            },
        );

        assert!(hints.is_empty(), "stale semantic facts must not leak into a newer document");
    }

    #[test]
    fn out_of_range_request_returns_no_hints() {
        let uri = Url::parse("file:///main.ph").unwrap();
        let doc = Document::new("let text = \"hello\"\n".to_string());
        let db = SemanticDb::new();
        db.update_file(&uri, FileRevision(1), &doc.parse.program);

        let hints = hints_for(
            &db,
            &uri,
            &doc,
            Range {
                start: Position::new(4, 0),
                end: Position::new(5, 0),
            },
        );

        assert!(hints.is_empty(), "hints outside requested range must be rejected");
    }
}
