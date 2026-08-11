//! Standard LSP runtime-value inlay hints.

use phalcom_ast::ast::{Pattern, Program, Statement};
use phalcom_common::range::SourceRange;
use tower_lsp::lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, InlayHintParams, InlayHintTooltip, MarkupContent, MarkupKind, Range, Url};

use crate::documents::Document;
use crate::semantic::{Confidence, SemanticDb, ValueShape};

/// Computes stable type-style hints for visible top-level binding declarations.
pub fn hints_for(db: &SemanticDb, uri: &Url, doc: &Document, visible: Range) -> Vec<InlayHint> {
    let visible_start = doc.line_index.offset(visible.start);
    let visible_end = doc.line_index.offset(visible.end);
    let mut bindings = Vec::new();
    collect_top_level_bindings(&doc.parse.program, &mut bindings);
    let mut hints = Vec::new();
    for (name, range) in bindings {
        if range.end < visible_start || range.start > visible_end {
            continue;
        }
        let Some(value) = db.binding_at(uri, &name, range.end.saturating_add(1)) else {
            continue;
        };
        if !should_render(&value.confidence, &value.shape) {
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

fn collect_top_level_bindings(program: &Program, out: &mut Vec<(String, SourceRange)>) {
    collect_bindings_in_statements(&program.statements, out);
}

fn collect_bindings_in_statements(statements: &[Statement], out: &mut Vec<(String, SourceRange)>) {
    for statement in statements {
        match statement {
            Statement::Let(binding) => collect_pattern_bindings(&binding.pattern, out),
            Statement::For(for_statement) => {
                out.push((for_statement.binding.clone(), for_statement.range));
                collect_bindings_in_statements(&for_statement.body, out);
            }
            Statement::Class(class) => {
                for member in &class.members {
                    let body = match member {
                        phalcom_ast::ast::ClassMember::Method(item) => &item.body,
                        phalcom_ast::ast::ClassMember::Getter(item) => &item.body,
                        phalcom_ast::ast::ClassMember::Setter(item) => &item.body,
                        phalcom_ast::ast::ClassMember::Index(item) => &item.body,
                        phalcom_ast::ast::ClassMember::Field(_) | phalcom_ast::ast::ClassMember::Variant(_) => continue,
                    };
                    collect_bindings_in_statements(body, out);
                }
            }
            _ => {}
        }
    }
}

fn collect_pattern_bindings(pattern: &Pattern, out: &mut Vec<(String, SourceRange)>) {
    match pattern {
        Pattern::Name { name, range } => out.push((name.clone(), *range)),
        Pattern::Tuple { elements, .. } => elements.iter().for_each(|element| collect_pattern_bindings(element, out)),
        Pattern::List { elements, rest, .. } => {
            elements.iter().for_each(|element| collect_pattern_bindings(element, out));
            if let Some(rest) = rest {
                collect_pattern_bindings(rest, out);
            }
        }
    }
}

fn should_render(confidence: &Confidence, shape: &ValueShape) -> bool {
    !matches!(confidence, Confidence::Heuristic) && !matches!(shape, ValueShape::Unknown)
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
