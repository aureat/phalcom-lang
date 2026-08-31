//! Receiver-aware completion backed by live semantic facts.
//!
//! Recovery stays deliberately syntax-light so a dangling dot or incomplete
//! chained send remains useful while the editor buffer is not parseable.

use phalcom_ast::ast::{Pattern, Program, Statement};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::SelectorKind;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat, Position};

use crate::documents::{Document, DocumentSnapshot};
/// Inputs for compiler-owned completion presentation.
pub(crate) struct CompilerCompletionContext<'a> {
    pub resolved: Option<&'a phalcom_semantic::ResolvedReceiver>,
    pub lexical_class: Option<&'a phalcom_semantic::DeclarationId>,
    pub privileged: bool,
    pub module: &'a phalcom_modules::ModuleId,
    pub text: &'a str,
    pub offset: usize,
}

/// Recovered member-completion target from an incomplete editor buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionTarget {
    /// Receiver expression range, excluding member dot.
    pub receiver_range: SourceRange,
    /// Member text already typed after dot.
    pub partial_member: String,
}

/// Recovers a member target at an LSP position with delimiter-balanced scans.
pub fn target_at(doc: &Document, position: Position) -> Option<CompletionTarget> {
    target_at_offset(&doc.text, doc.line_index.offset(position))
}

/// Recovers a completion target from an owned document snapshot.
pub(crate) fn target_at_snapshot(doc: &DocumentSnapshot, position: Position) -> Option<CompletionTarget> {
    target_at_offset(&doc.text, doc.line_index.offset(position))
}

fn target_at_offset(text: &str, offset: usize) -> Option<CompletionTarget> {
    let end = offset.min(text.len());
    let bytes = text.as_bytes();
    let mut partial_start = end;
    while partial_start > 0 && is_identifier_byte(bytes[partial_start - 1]) {
        partial_start -= 1;
    }
    let dot = trim_left(text, partial_start);
    if dot == 0 || bytes[dot - 1] != b'.' {
        return None;
    }
    let receiver_end = dot - 1;
    let receiver_start = scan_expression_start(text, receiver_end)?;
    (receiver_start < receiver_end).then_some(CompletionTarget {
        receiver_range: SourceRange {
            start: receiver_start,
            end: receiver_end,
        },
        partial_member: text[partial_start..end].to_string(),
    })
}

fn scan_expression_start(text: &str, end: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let end = trim_left(text, end);
    if end == 0 {
        return None;
    }
    let mut start = match bytes[end - 1] {
        b')' | b']' | b'}' => {
            let open = matching_open(bytes, end - 1)?;
            let before = trim_left(text, open);
            if before > 0 && is_identifier_byte(bytes[before - 1]) {
                scan_identifier_start(bytes, before)
            } else {
                open
            }
        }
        byte if is_identifier_byte(byte) => scan_identifier_start(bytes, end),
        b'"' | b'\'' => scan_quoted_start(bytes, end)?,
        _ => return None,
    };
    loop {
        let before = trim_left(text, start);
        if before == 0 || bytes[before - 1] != b'.' {
            return Some(start);
        }
        start = scan_expression_start(text, before - 1)?;
    }
}

fn matching_open(bytes: &[u8], close: usize) -> Option<usize> {
    let (open, close_byte) = match bytes.get(close)? {
        b')' => (b'(', b')'),
        b']' => (b'[', b']'),
        b'}' => (b'{', b'}'),
        _ => return None,
    };
    let mut depth = 0;
    for index in (0..=close).rev() {
        match bytes[index] {
            byte if byte == close_byte => depth += 1,
            byte if byte == open => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn scan_identifier_start(bytes: &[u8], end: usize) -> usize {
    let mut start = end;
    while start > 0 && is_identifier_byte(bytes[start - 1]) {
        start -= 1;
    }
    start
}

fn scan_quoted_start(bytes: &[u8], end: usize) -> Option<usize> {
    let quote = *bytes.get(end - 1)?;
    (0..end - 1)
        .rev()
        .find(|&index| bytes[index] == quote && (index == 0 || bytes[index - 1] != b'\\'))
}

fn trim_left(text: &str, mut offset: usize) -> usize {
    while offset > 0 && text.as_bytes()[offset - 1].is_ascii_whitespace() {
        offset -= 1;
    }
    offset
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Builds completion items from one pinned compiler snapshot.
///
/// Receiver completion never reconstructs a source module surface or asks the
/// legacy LSP semantic database to infer a class. Unknown receiver targets
/// therefore produce no fabricated member surface. Non-member completion is
/// deliberately bounded to names visible in the current compiler source shard.
pub(crate) fn compiler_contextual_completions(db: &phalcom_semantic::SemanticSnapshot, context: CompilerCompletionContext<'_>) -> Vec<CompletionItem> {
    let mut items = if target_at_offset(context.text, context.offset).is_some() {
        match context.resolved {
            Some(resolved) if resolved.alternatives.len() > 1 => compiler_union_completions(db, resolved, context.lexical_class, context.privileged),
            Some(resolved) => resolved
                .alternatives
                .first()
                .map(|alternative| compiler_class_completions(db, &alternative.declaration, alternative.mode, context.lexical_class, context.privileged))
                .unwrap_or_default(),
            None => Vec::new(),
        }
    } else {
        compiler_visible_completions(db, context.module, context.offset)
    };
    items.sort_by(|left, right| left.label.cmp(&right.label));
    items.dedup_by(|left, right| left.label == right.label);
    items
}

fn compiler_union_completions(
    db: &phalcom_semantic::SemanticSnapshot,
    resolved: &phalcom_semantic::ResolvedReceiver,
    lexical_class: Option<&phalcom_semantic::DeclarationId>,
    privileged: bool,
) -> Vec<CompletionItem> {
    let mut by_label = std::collections::BTreeMap::<String, (CompletionItem, usize)>::new();
    for alternative in resolved.alternatives.iter() {
        for item in compiler_class_completions(db, &alternative.declaration, alternative.mode, lexical_class, privileged) {
            by_label
                .entry(item.label.clone())
                .and_modify(|(_, coverage)| *coverage += 1)
                .or_insert((item, 1));
        }
    }
    let total = resolved.alternatives.len();
    by_label
        .into_values()
        .map(|(mut item, coverage)| {
            let owner = item.detail.take().unwrap_or_else(|| "semantic receiver".to_string());
            item.detail = Some(format!("{owner} — available on {coverage}/{total} candidates"));
            item.sort_text = Some(format!("{:02}:{}", total.saturating_sub(coverage), item.label));
            item
        })
        .collect()
}

fn compiler_class_completions(
    db: &phalcom_semantic::SemanticSnapshot,
    declaration: &phalcom_semantic::DeclarationId,
    receiver_mode: phalcom_semantic::ReceiverMode,
    lexical_class: Option<&phalcom_semantic::DeclarationId>,
    _privileged: bool,
) -> Vec<CompletionItem> {
    let receiver = phalcom_semantic::ResolvedReceiver {
        alternatives: std::sync::Arc::from([phalcom_semantic::ReceiverAlternative {
            declaration: declaration.clone(),
            mode: receiver_mode,
        }]),
    };
    let access = phalcom_semantic::AccessContext {
        enclosing_declaration: lexical_class.cloned(),
        enclosing_callable: None,
    };
    let mut items = Vec::new();
    for member in db.editor().members_for_receiver(&receiver, &access) {
        match member.target {
            phalcom_semantic::EditorMemberTarget::Field(field) => items.push(CompletionItem {
                label: field.name.to_string(),
                kind: Some(CompletionItemKind::FIELD),
                insert_text: Some(field.name.to_string()),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                detail: Some(member.owner.name.to_string()),
                ..CompletionItem::default()
            }),
            phalcom_semantic::EditorMemberTarget::Callable(callable) => {
                let label = callable.selector.encode();
                let (kind, insert_text, insert_text_format) = match callable.selector.kind {
                    SelectorKind::Getter => (CompletionItemKind::PROPERTY, label.clone(), InsertTextFormat::PLAIN_TEXT),
                    SelectorKind::Setter => (CompletionItemKind::PROPERTY, setter_snippet(&label), InsertTextFormat::SNIPPET),
                    SelectorKind::Method | SelectorKind::SubscriptGet | SelectorKind::SubscriptSet => {
                        (CompletionItemKind::METHOD, method_snippet(&label), InsertTextFormat::SNIPPET)
                    }
                };
                items.push(CompletionItem {
                    label,
                    kind: Some(kind),
                    insert_text: Some(insert_text),
                    insert_text_format: Some(insert_text_format),
                    detail: Some(member.owner.name.to_string()),
                    ..CompletionItem::default()
                });
            }
        }
    }
    items.sort_by(|left, right| left.label.cmp(&right.label));
    items.dedup_by(|left, right| left.label == right.label);
    items
}

fn compiler_visible_completions(db: &phalcom_semantic::SemanticSnapshot, module: &phalcom_modules::ModuleId, offset: usize) -> Vec<CompletionItem> {
    let names = db
        .editor()
        .visible_symbols_at(module, offset)
        .into_iter()
        .map(|symbol| symbol.name.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    names
        .into_iter()
        .map(|name| CompletionItem {
            label: name.clone(),
            kind: Some(CompletionItemKind::VARIABLE),
            insert_text: Some(name),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            ..CompletionItem::default()
        })
        .collect()
}

/// Returns syntax-only names while the pinned compiler publication is stale or
/// has not mapped the document yet. This deliberately never infers a receiver
/// class or reconstructs a member surface.
pub(crate) fn syntax_visible_completions(program: &Program, text: &str, offset: usize) -> Vec<CompletionItem> {
    if target_at_offset(text, offset).is_some() {
        return Vec::new();
    }

    let mut names = Vec::new();
    for dependency in &program.preamble.dependencies {
        match dependency {
            phalcom_ast::ast::DependencyDecl::Import(import) => match import {
                phalcom_ast::ast::ImportDecl::Module(import) => {
                    if let Some(alias) = &import.alias {
                        names.push(alias.name.clone());
                    } else if let Some(segment) = import.path.segments.last() {
                        names.push(segment.name.clone());
                    }
                }
                phalcom_ast::ast::ImportDecl::Selective(import) => {
                    names.extend(
                        import
                            .items
                            .iter()
                            .map(|item| item.alias.as_ref().map_or_else(|| item.name.clone(), |alias| alias.name.clone())),
                    );
                }
            },
            phalcom_ast::ast::DependencyDecl::ReExport(reexport) => {
                names.extend(reexport.items.iter().map(|item| item.local_or_remote_name.clone()));
            }
            phalcom_ast::ast::DependencyDecl::Expose(_) => {}
        }
    }
    for statement in &program.statements {
        match statement {
            Statement::Class(class) => names.push(class.name.clone()),
            Statement::Let(binding) if binding.range.start < offset => collect_pattern_names(&binding.pattern, &mut names),
            Statement::For(for_statement) if for_statement.range.contains(offset) => {
                for lane in &for_statement.lanes {
                    collect_pattern_names(&lane.pattern, &mut names);
                    if let Some(index) = &lane.index {
                        names.push(index.name.clone());
                    }
                }
            }
            _ => {}
        }
    }
    names.sort();
    names.dedup();
    names
        .into_iter()
        .map(|name| CompletionItem {
            label: name.clone(),
            kind: Some(CompletionItemKind::VARIABLE),
            insert_text: Some(name),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            ..CompletionItem::default()
        })
        .collect()
}

fn collect_pattern_names(pattern: &Pattern, out: &mut Vec<String>) {
    match pattern {
        Pattern::Name { name, .. } => out.push(name.clone()),
        Pattern::Tuple { elements, .. } => elements.iter().for_each(|element| collect_pattern_names(element, out)),
        Pattern::List { elements, rest, .. } => {
            elements.iter().for_each(|element| collect_pattern_names(element, out));
            if let Some(rest) = rest {
                collect_pattern_names(rest, out);
            }
        }
        Pattern::Variant(variant_pat) => match &variant_pat.mode {
            phalcom_ast::ast::VariantPatternMode::ExactCall { arguments } => {
                arguments.iter().for_each(|arg| collect_pattern_names(&arg.pattern, out));
            }
            phalcom_ast::ast::VariantPatternMode::CallablePattern { prefix, suffix, .. } => {
                prefix.iter().for_each(|arg| collect_pattern_names(&arg.pattern, out));
                suffix.iter().for_each(|arg| collect_pattern_names(&arg.pattern, out));
            }
            _ => {}
        },
        Pattern::Or { alternatives, .. } => alternatives.iter().for_each(|pat| collect_pattern_names(pat, out)),
        Pattern::Wildcard { .. } => {}
        Pattern::Record { entries, .. } => entries.iter().for_each(|entry| collect_pattern_names(&entry.pattern, out)),
        Pattern::Map { entries, .. } => entries.iter().for_each(|entry| collect_pattern_names(&entry.pattern, out)),
    }
}

fn method_snippet(selector: &str) -> String {
    let Some(open) = selector.find('(') else { return selector.to_string() };
    let name = &selector[..open];
    let inner = selector[open + 1..].strip_suffix(')').unwrap_or(&selector[open + 1..]);
    if inner.is_empty() {
        return format!("{name}()");
    }
    let slots = inner
        .split(',')
        .enumerate()
        .map(|(index, slot)| {
            let number = index + 1;
            if slot == "_" {
                format!("${{{number}:_}}")
            } else {
                format!("{slot}: ${{{number}:_}}")
            }
        })
        .collect::<Vec<_>>();
    format!("{name}({})", slots.join(", "))
}

fn setter_snippet(selector: &str) -> String {
    selector
        .strip_suffix("=(put)")
        .map_or_else(|| selector.to_string(), |base| format!("{base} = ${{1:value}}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_recovery_handles_chained_and_balanced_receivers() {
        for (source, receiver) in [("p.", "p"), ("factory.make().", "factory.make()"), ("users[0].", "users[0]")] {
            let target = target_at_offset(source, source.len()).unwrap();
            assert_eq!(&source[target.receiver_range.start..target.receiver_range.end], receiver);
        }
    }

    #[test]
    fn snippets_preserve_selector_shape() {
        assert_eq!(method_snippet("move(_,to,duration)"), "move(${1:_}, to: ${2:_}, duration: ${3:_})");
        assert_eq!(setter_snippet("x=(put)"), "x = ${1:value}");
    }
}
