//! Receiver-aware completion backed by live semantic facts.
//!
//! Recovery stays deliberately syntax-light so a dangling dot or incomplete
//! chained send remains useful while the editor buffer is not parseable.

use phalcom_ast::ast::{Pattern, Program, Statement};
use phalcom_common::range::SourceRange;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat, Position, Url};

use crate::documents::Document;
use crate::semantic::{ClassId, CompletionMember, DispatchSide, MemberKind, MemberVisibility, SemanticDb};

/// Whether a resolved receiver is an instance or a class object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverKind {
    /// Offer instance-side members.
    Instance,
    /// Offer class-side members.
    ClassObject,
}

/// Module-qualified receiver alternatives returned by semantic queries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticResolvedReceiver {
    /// Candidate class identities and dispatch sides.
    pub alternatives: Vec<(ClassId, ReceiverKind)>,
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

/// Completion context backed entirely by the live semantic database.
pub(crate) struct SemanticCompletionContext<'a> {
    pub resolved: Option<&'a SemanticResolvedReceiver>,
    pub lexical_class: Option<&'a ClassId>,
    pub privileged: bool,
    pub uri: &'a Url,
    pub program: &'a Program,
    pub text: &'a str,
    pub offset: usize,
}

/// Builds completion items from live source and native semantic surfaces.
pub(crate) fn semantic_contextual_completions(db: &SemanticDb, context: SemanticCompletionContext<'_>) -> Vec<CompletionItem> {
    let mut items = match context.resolved {
        Some(resolved) if resolved.alternatives.len() > 1 => semantic_union_completions(db, resolved, context.lexical_class, context.privileged),
        Some(resolved) => resolved
            .alternatives
            .first()
            .map(|(class, kind)| semantic_class_completions(db, class, *kind, context.lexical_class, context.privileged))
            .unwrap_or_default(),
        None => semantic_all_completions(db, context.lexical_class, context.privileged),
    };
    if target_at_offset(context.text, context.offset).is_none() {
        if let Some(class) = context.lexical_class {
            items.extend(semantic_class_completions(db, class, ReceiverKind::Instance, Some(class), context.privileged));
        }
        items.extend(visible_names_at(db, context.uri, context.program, context.offset).into_iter().map(|name| CompletionItem {
            label: name.clone(),
            kind: Some(CompletionItemKind::VARIABLE),
            insert_text: Some(name),
            ..CompletionItem::default()
        }));
    }
    items.sort_by(|left, right| left.label.cmp(&right.label));
    items.dedup_by(|left, right| left.label == right.label);
    items
}

fn semantic_union_completions(db: &SemanticDb, resolved: &SemanticResolvedReceiver, lexical_class: Option<&ClassId>, privileged: bool) -> Vec<CompletionItem> {
    let mut by_label = std::collections::BTreeMap::new();
    for (class, kind) in &resolved.alternatives {
        for item in semantic_class_completions(db, class, *kind, lexical_class, privileged) {
            by_label
                .entry(item.label.clone())
                .and_modify(|(_, coverage)| *coverage += 1)
                .or_insert((item, 1_usize));
        }
    }
    let total = resolved.alternatives.len();
    let mut items = by_label
        .into_values()
        .map(|(mut item, coverage)| {
            let owner = item.detail.unwrap_or_else(|| "semantic receiver".to_string());
            item.detail = Some(format!("{owner} — available on {coverage}/{total} candidates"));
            item.sort_text = Some(format!("{:02}:{}", total.saturating_sub(coverage), item.label));
            item
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.sort_text.cmp(&right.sort_text).then_with(|| left.label.cmp(&right.label)));
    items
}

fn semantic_class_completions(
    db: &SemanticDb,
    class: &ClassId,
    receiver_kind: ReceiverKind,
    lexical_class: Option<&ClassId>,
    privileged: bool,
) -> Vec<CompletionItem> {
    let side = match receiver_kind {
        ReceiverKind::Instance => DispatchSide::Instance,
        ReceiverKind::ClassObject => DispatchSide::Class,
    };
    let mut items = db
        .completion_members(class, side)
        .iter()
        .filter(|member| semantic_visibility_allowed(db, member, lexical_class, privileged))
        .map(semantic_to_completion_item)
        .collect::<Vec<_>>();
    if receiver_kind == ReceiverKind::ClassObject && !items.iter().any(|item| item.label == "new()") {
        items.push(CompletionItem {
            label: "new()".to_string(),
            kind: Some(CompletionItemKind::METHOD),
            insert_text: Some("new()".to_string()),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            ..CompletionItem::default()
        });
    }
    items.sort_by(|left, right| left.label.cmp(&right.label));
    items
}

fn semantic_all_completions(db: &SemanticDb, lexical_class: Option<&ClassId>, privileged: bool) -> Vec<CompletionItem> {
    let mut items = db
        .all_completion_members()
        .iter()
        .filter(|member| semantic_visibility_allowed(db, member, lexical_class, privileged))
        .map(semantic_to_completion_item)
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.label.cmp(&right.label));
    items
}

fn semantic_visibility_allowed(db: &SemanticDb, member: &CompletionMember, lexical_class: Option<&ClassId>, privileged: bool) -> bool {
    match member.visibility {
        MemberVisibility::Public => true,
        MemberVisibility::Private => lexical_class == Some(&member.owner),
        MemberVisibility::Protected => lexical_class.is_some_and(|caller| db.is_same_or_subclass(caller, &member.owner)),
        MemberVisibility::Internal => privileged,
    }
}

fn semantic_to_completion_item(member: &CompletionMember) -> CompletionItem {
    let (kind, insert_text, insert_text_format) = match member.kind {
        MemberKind::Getter | MemberKind::Field => (CompletionItemKind::PROPERTY, member.selector.clone(), InsertTextFormat::PLAIN_TEXT),
        MemberKind::Setter => (CompletionItemKind::PROPERTY, setter_snippet(&member.selector), InsertTextFormat::SNIPPET),
        MemberKind::Method | MemberKind::Index | MemberKind::Variant => {
            (CompletionItemKind::METHOD, method_snippet(&member.selector), InsertTextFormat::SNIPPET)
        }
    };
    CompletionItem {
        label: member.selector.clone(),
        detail: Some(member.owner.name.clone()),
        kind: Some(kind),
        insert_text: Some(insert_text),
        insert_text_format: Some(insert_text_format),
        ..CompletionItem::default()
    }
}

fn visible_names_at(db: &SemanticDb, uri: &Url, program: &Program, offset: usize) -> Vec<String> {
    let mut names = db.visible_bindings_at(uri, offset).into_iter().map(|binding| binding.name).collect::<Vec<_>>();
    for statement in &program.statements {
        match statement {
            Statement::Class(class) => names.push(class.name.clone()),
            Statement::Import(import) => names.push(import.binding.clone()),
            Statement::Let(binding) if binding.range.start < offset => collect_pattern_names(&binding.pattern, &mut names),
            Statement::For(for_statement) if for_statement.range.contains(offset) => names.push(for_statement.binding.clone()),
            _ => {}
        }
    }
    names.sort();
    names.dedup();
    names
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
