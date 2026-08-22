//! Receiver-aware completion backed by live semantic facts.
//!
//! Recovery stays deliberately syntax-light so a dangling dot or incomplete
//! chained send remains useful while the editor buffer is not parseable.

use phalcom_ast::ast::{ClassMember, Expr, MethodDef, PackItem, Pattern, Program, Statement};
use phalcom_common::range::SourceRange;
use phalcom_native_surface::{NATIVE_MEMBERS, NativeDispatch, NativeMemberKind, NativeVisibility};
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat, Position, Url};

use crate::documents::{Document, DocumentSnapshot};
use crate::index::WorkspaceIndex;
use crate::semantic::{ClassId, CompletionMember, DispatchSide, MemberKind, MemberVisibility, SemanticSnapshot};

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

/// Recovers a completion target from an owned document snapshot.
pub(crate) fn target_at_snapshot(doc: &DocumentSnapshot, position: Position) -> Option<CompletionTarget> {
    target_at_offset(&doc.text, doc.line_index.offset(position))
}

/// Supplies immediate receiver completion from shallow source/index data
/// while the background semantic snapshot is still catching up.
/// Same bounded completion fallback using a request-local recovery parse for
/// buffers whose dangling member dot prevented the normal parse from
/// reaching later declarations.
#[cfg(test)]
pub(crate) fn shallow_receiver_completions_from_program(
    index: &WorkspaceIndex,
    uri: &Url,
    doc: &Document,
    program: &Program,
    position: Position,
) -> Option<Vec<CompletionItem>> {
    let target = target_at(doc, position)?;
    let receiver = doc.text.get(target.receiver_range.start..target.receiver_range.end)?.trim();
    if receiver.is_empty() {
        return None;
    }
    let classes = shallow_receiver_classes(program, receiver, target.receiver_range.end);
    if classes.is_empty() {
        return None;
    }
    let side = shallow_receiver_side(receiver);

    let module = crate::semantic::ModuleId::new(uri.to_string());
    let local_surface = crate::semantic::build_module_surface(module.clone(), program);
    let candidates = classes
        .iter()
        .map(|class| shallow_class_items(index, uri, &local_surface, &module, class, side))
        .collect::<Vec<_>>();
    Some(shallow_union_items(candidates))
}

/// Same bounded fallback for an owned request snapshot.
pub(crate) fn shallow_receiver_completions_from_snapshot(
    index: &WorkspaceIndex,
    uri: &Url,
    doc: &DocumentSnapshot,
    program: &Program,
    position: Position,
) -> Option<Vec<CompletionItem>> {
    let target = target_at_snapshot(doc, position)?;
    let receiver = doc.text.get(target.receiver_range.start..target.receiver_range.end)?.trim();
    if receiver.is_empty() {
        return None;
    }
    let classes = shallow_receiver_classes(program, receiver, target.receiver_range.end);
    if classes.is_empty() {
        return None;
    }
    let side = shallow_receiver_side(receiver);
    let module = crate::semantic::ModuleId::new(uri.to_string());
    let local_surface = crate::semantic::build_module_surface(module.clone(), program);
    let candidates = classes
        .iter()
        .map(|class| shallow_class_items(index, uri, &local_surface, &module, class, side))
        .collect::<Vec<_>>();
    Some(shallow_union_items(candidates))
}

/// Resolves only source-local constructor shapes needed to keep completion
/// useful before the worker publishes its first semantic generation. This is
/// intentionally bounded: it walks the current program, never the workspace.
fn shallow_receiver_classes(program: &Program, receiver: &str, offset: usize) -> Vec<String> {
    let mut classes = std::collections::BTreeSet::new();

    if receiver.contains('.') || receiver.contains('(') {
        if let Some(expr) = phalcom_ast::parser::parse(receiver, 0)
            .program
            .statements
            .iter()
            .find_map(|statement| match statement {
                Statement::Expr { expr, .. } => Some(expr),
                _ => None,
            })
        {
            collect_expression_classes(program, expr, &mut classes);
        }
    }

    if receiver == "self" {
        if let Some((class, _)) = enclosing_method(program, offset) {
            classes.insert(class.name.clone());
        }
    } else if receiver == "super" {
        if let Some((class, _)) = enclosing_method(program, offset) {
            if let Some(parent) = class.superclass_ref() {
                classes.insert(parent.leaf_name().to_string());
            }
        }
    } else if receiver.chars().next().is_some_and(char::is_uppercase)
        && receiver.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        && program
            .statements
            .iter()
            .any(|statement| matches!(statement, Statement::Class(class) if class.name == receiver))
    {
        classes.insert(receiver.to_string());
    }

    for statement in &program.statements {
        if let Statement::Let(binding) = statement {
            if let Pattern::Name { name, .. } = &binding.pattern {
                if name == receiver && binding.range.start < offset {
                    if let Some(class) = constructor_class(binding.value.as_ref()) {
                        classes.insert(class);
                    }
                }
            }
        }
    }

    let Some((class, method)) = enclosing_method(program, offset) else {
        return classes.into_iter().collect();
    };

    if receiver.starts_with('_') {
        for member in &class.members {
            if let Some(body) = member_body(member) {
                collect_field_constructor_assignments(body, receiver, &mut classes);
            }
        }
    }

    if let Some(method) = method {
        if method.params.iter().any(|param| param.name == receiver) {
            for statement in &program.statements {
                collect_argument_constructor_classes(statement, &method.name, receiver, &mut classes);
            }
        }
    }

    classes.into_iter().collect()
}

fn shallow_receiver_side(receiver: &str) -> DispatchSide {
    if receiver.chars().next().is_some_and(char::is_uppercase) && receiver.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
        DispatchSide::Class
    } else {
        DispatchSide::Instance
    }
}

fn collect_expression_classes(program: &Program, expr: &Expr, classes: &mut std::collections::BTreeSet<String>) {
    match expr {
        Expr::MethodCall(call) if call.method == "new" => {
            if let Expr::Var { value, .. } = &call.object {
                classes.insert(value.clone());
            }
        }
        Expr::MethodCall(call) => {
            let mut receivers = std::collections::BTreeSet::new();
            collect_expression_classes(program, &call.object, &mut receivers);
            for receiver in receivers {
                if let Some(class) = find_method_return_class(program, &receiver, &call.method) {
                    classes.insert(class);
                }
            }
        }
        Expr::Var { value, .. } => {
            for statement in &program.statements {
                let Statement::Let(binding) = statement else { continue };
                let Pattern::Name { name, .. } = &binding.pattern else { continue };
                if name == value {
                    if let Some(value) = binding.value.as_ref() {
                        collect_expression_classes(program, value, classes);
                    }
                }
            }
        }
        _ => {}
    }
}

fn find_method_return_class(program: &Program, class_name: &str, method_name: &str) -> Option<String> {
    let class = program.statements.iter().find_map(|statement| match statement {
        Statement::Class(class) if class.name == class_name => Some(class),
        _ => None,
    })?;
    let body = class.members.iter().find_map(|member| match member {
        ClassMember::Method(method) if method.name == method_name => Some(method.body.as_slice()),
        _ => None,
    })?;
    body.iter().rev().find_map(|statement| match statement {
        Statement::Return(return_statement) => constructor_class(return_statement.value.as_ref()),
        Statement::Expr { expr, .. } => constructor_class(Some(expr)),
        _ => None,
    })
}

fn constructor_class(value: Option<&Expr>) -> Option<String> {
    let Expr::MethodCall(call) = value? else { return None };
    if call.method != "new" {
        return None;
    }
    let Expr::Var { value, .. } = &call.object else { return None };
    Some(value.clone())
}

fn enclosing_method(program: &Program, offset: usize) -> Option<(&phalcom_ast::ast::ClassDef, Option<&MethodDef>)> {
    program.statements.iter().find_map(|statement| {
        let Statement::Class(class) = statement else { return None };
        if !class.range.contains(offset) {
            return None;
        }
        let method = class.members.iter().find_map(|member| match member {
            ClassMember::Method(method) if method.range.contains(offset) => Some(method),
            _ => None,
        });
        Some((class, method))
    })
}

fn member_body(member: &ClassMember) -> Option<&[Statement]> {
    match member {
        ClassMember::Method(method) => Some(&method.body),
        ClassMember::Getter(getter) => Some(&getter.body),
        ClassMember::Setter(setter) => Some(&setter.body),
        ClassMember::Index(index) => Some(&index.body),
        ClassMember::Field(_) | ClassMember::Variant(_) => None,
    }
}

fn collect_field_constructor_assignments(statements: &[Statement], field: &str, classes: &mut std::collections::BTreeSet<String>) {
    for statement in statements {
        match statement {
            Statement::Expr { expr, .. } | Statement::Throw { expr, .. } => collect_field_constructor_expr(expr, field, classes),
            Statement::Return(return_statement) => {
                if let Some(expr) = &return_statement.value {
                    collect_field_constructor_expr(expr, field, classes);
                }
            }
            Statement::For(for_statement) => {
                for lane in &for_statement.lanes {
                    collect_field_constructor_expr(&lane.iter, field, classes);
                }
                collect_field_constructor_assignments(&for_statement.body, field, classes);
            }
            _ => {}
        }
    }
}

fn collect_field_constructor_expr(expr: &Expr, field: &str, classes: &mut std::collections::BTreeSet<String>) {
    match expr {
        Expr::Assignment(assignment) => {
            if let Expr::Field { value, .. } = assignment.name.as_ref() {
                if value == field {
                    if let Some(class) = constructor_class(Some(&assignment.value)) {
                        classes.insert(class);
                    }
                }
            }
            collect_field_constructor_expr(&assignment.value, field, classes);
        }
        Expr::Block(block) => collect_field_constructor_assignments(&block.body, field, classes),
        Expr::MethodCall(call) => {
            collect_field_constructor_expr(&call.object, field, classes);
            for item in &call.args {
                collect_field_constructor_pack(item, field, classes);
            }
        }
        _ => {}
    }
}

fn collect_field_constructor_pack(item: &PackItem, field: &str, classes: &mut std::collections::BTreeSet<String>) {
    match item {
        PackItem::Positional { expr, .. } | PackItem::Expand { expr, .. } => collect_field_constructor_expr(expr, field, classes),
        PackItem::Labeled { value, .. } => collect_field_constructor_expr(value, field, classes),
    }
}

fn collect_argument_constructor_classes(statement: &Statement, method_name: &str, parameter: &str, classes: &mut std::collections::BTreeSet<String>) {
    match statement {
        Statement::Expr { expr, .. } | Statement::Throw { expr, .. } => collect_argument_constructor_expr(expr, method_name, parameter, classes),
        Statement::Return(return_statement) => {
            if let Some(expr) = &return_statement.value {
                collect_argument_constructor_expr(expr, method_name, parameter, classes);
            }
        }
        Statement::For(for_statement) => {
            for lane in &for_statement.lanes {
                collect_argument_constructor_expr(&lane.iter, method_name, parameter, classes);
            }
            for nested in &for_statement.body {
                collect_argument_constructor_classes(nested, method_name, parameter, classes);
            }
        }
        _ => {}
    }
}

fn collect_argument_constructor_expr(expr: &Expr, method_name: &str, parameter: &str, classes: &mut std::collections::BTreeSet<String>) {
    match expr {
        Expr::UnqualifiedCall(call) if call.name == method_name => {
            for item in &call.args {
                let PackItem::Positional { expr, .. } = item else { continue };
                if let Some(class) = constructor_class(Some(expr)) {
                    classes.insert(class);
                }
                let _ = parameter;
            }
        }
        Expr::MethodCall(call) => {
            collect_argument_constructor_expr(&call.object, method_name, parameter, classes);
            for item in &call.args {
                let expr = match item {
                    PackItem::Positional { expr, .. } | PackItem::Expand { expr, .. } | PackItem::Labeled { value: expr, .. } => expr,
                };
                collect_argument_constructor_expr(expr, method_name, parameter, classes);
            }
        }
        Expr::Block(block) => {
            for statement in &block.body {
                collect_argument_constructor_classes(statement, method_name, parameter, classes);
            }
        }
        _ => {}
    }
}

fn shallow_class_items(
    index: &WorkspaceIndex,
    uri: &Url,
    local_surface: &crate::semantic::ModuleSurface,
    module: &crate::semantic::ModuleId,
    class: &str,
    side: DispatchSide,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    let mut current = Some(class.to_string());
    let mut visited = std::collections::BTreeSet::new();
    while let Some(class_name) = current.take() {
        if !visited.insert(class_name.clone()) {
            break;
        }
        if let Some(surface) = local_surface.classes.get(&crate::semantic::ClassId::new(module.clone(), class_name.clone())) {
            for member in surface.members_on(side) {
                items.push(semantic_to_completion_item(&CompletionMember {
                    selector: member.callable.selector.clone(),
                    kind: member.kind,
                    owner: member.callable.owner.clone(),
                    visibility: member.visibility,
                    side: member.side,
                }));
            }
            current = surface
                .superclass
                .as_ref()
                .and_then(|parent| (parent.module == *module).then(|| parent.name.clone()));
        } else {
            for member in index.class_members(uri, &class_name) {
                if (side == DispatchSide::Class) == member.is_class_side {
                    items.push(shallow_member_item(&member.selector, member.kind, &member.owner));
                }
            }
            current = index.class_parent(uri, &class_name);
        }
    }
    if side == DispatchSide::Instance && !visited.contains("Object") {
        items.extend(native_object_items());
    }
    items.sort_by(|left, right| left.label.cmp(&right.label));
    items.dedup_by(|left, right| left.label == right.label);
    items
}

fn shallow_union_items(candidates: Vec<Vec<CompletionItem>>) -> Vec<CompletionItem> {
    let total = candidates.len();
    let mut by_label = std::collections::BTreeMap::<String, (CompletionItem, usize)>::new();
    for items in candidates {
        for item in items {
            by_label
                .entry(item.label.to_string())
                .and_modify(|(_, coverage)| *coverage += 1)
                .or_insert((item, 1));
        }
    }
    let mut items = by_label
        .into_values()
        .map(|(mut item, coverage)| {
            let owner = item.detail.unwrap_or_else(|| "shallow receiver".to_string());
            item.detail = Some(format!("{owner} — available on {coverage}/{total} candidates"));
            item.sort_text = Some(format!("{:02}:{}", total.saturating_sub(coverage), item.label));
            item
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.sort_text.cmp(&right.sort_text).then_with(|| left.label.cmp(&right.label)));
    items
}

fn native_object_items() -> Vec<CompletionItem> {
    NATIVE_MEMBERS
        .iter()
        .filter(|member| member.class == "Object" && member.side == NativeDispatch::Instance && member.visibility == NativeVisibility::Public)
        .map(|member| {
            let kind = match member.kind {
                NativeMemberKind::Getter => crate::index::MemberKind::Getter,
                NativeMemberKind::Setter => crate::index::MemberKind::Setter,
                NativeMemberKind::Method => crate::index::MemberKind::Method,
            };
            shallow_member_item(member.selector, kind, "Object")
        })
        .collect()
}

fn shallow_member_item(selector: &str, kind: crate::index::MemberKind, owner: &str) -> CompletionItem {
    let (item_kind, insert_text, insert_text_format) = match kind {
        crate::index::MemberKind::Getter => (CompletionItemKind::PROPERTY, selector.to_string(), InsertTextFormat::PLAIN_TEXT),
        crate::index::MemberKind::Field => (CompletionItemKind::FIELD, selector.to_string(), InsertTextFormat::PLAIN_TEXT),
        crate::index::MemberKind::Setter => (CompletionItemKind::PROPERTY, setter_snippet(selector), InsertTextFormat::SNIPPET),
        crate::index::MemberKind::Method | crate::index::MemberKind::StaticMethod | crate::index::MemberKind::Construct => {
            (CompletionItemKind::METHOD, method_snippet(selector), InsertTextFormat::SNIPPET)
        }
    };
    CompletionItem {
        label: selector.to_string(),
        detail: Some(owner.to_string()),
        kind: Some(item_kind),
        insert_text: Some(insert_text),
        insert_text_format: Some(insert_text_format),
        ..CompletionItem::default()
    }
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
pub(crate) fn semantic_contextual_completions(db: &SemanticSnapshot, context: SemanticCompletionContext<'_>) -> Vec<CompletionItem> {
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
        items.extend(
            visible_names_at(db, context.uri, context.program, context.offset)
                .into_iter()
                .map(|name| CompletionItem {
                    label: name.clone(),
                    kind: Some(CompletionItemKind::VARIABLE),
                    insert_text: Some(name),
                    ..CompletionItem::default()
                }),
        );
    }
    items.sort_by(|left, right| left.label.cmp(&right.label));
    items.dedup_by(|left, right| left.label == right.label);
    items
}

fn semantic_union_completions(
    db: &SemanticSnapshot,
    resolved: &SemanticResolvedReceiver,
    lexical_class: Option<&ClassId>,
    privileged: bool,
) -> Vec<CompletionItem> {
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
    db: &SemanticSnapshot,
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

fn semantic_all_completions(db: &SemanticSnapshot, lexical_class: Option<&ClassId>, privileged: bool) -> Vec<CompletionItem> {
    let mut items = db
        .all_completion_members()
        .iter()
        .filter(|member| semantic_visibility_allowed(db, member, lexical_class, privileged))
        .map(semantic_to_completion_item)
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.label.cmp(&right.label));
    items
}

fn semantic_visibility_allowed(db: &SemanticSnapshot, member: &CompletionMember, lexical_class: Option<&ClassId>, privileged: bool) -> bool {
    match member.visibility {
        MemberVisibility::Public => true,
        MemberVisibility::Private => lexical_class == Some(&member.owner),
        MemberVisibility::Protected => lexical_class.is_some_and(|caller| db.is_same_or_subclass(caller, &member.owner)),
        MemberVisibility::Internal => privileged,
    }
}

fn semantic_to_completion_item(member: &CompletionMember) -> CompletionItem {
    let (kind, insert_text, insert_text_format) = match member.kind {
        MemberKind::Getter => (CompletionItemKind::PROPERTY, member.selector.clone(), InsertTextFormat::PLAIN_TEXT),
        MemberKind::Field => (CompletionItemKind::FIELD, member.selector.clone(), InsertTextFormat::PLAIN_TEXT),
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

fn visible_names_at(db: &SemanticSnapshot, uri: &Url, program: &Program, offset: usize) -> Vec<String> {
    let mut names = db.visible_bindings_at(uri, offset).into_iter().map(|binding| binding.name).collect::<Vec<_>>();
    for dep in &program.preamble.dependencies {
        match dep {
            phalcom_ast::ast::DependencyDecl::Import(imp) => match imp {
                phalcom_ast::ast::ImportDecl::Module(m) => {
                    let name = if let Some(alias) = &m.alias {
                        alias.name.clone()
                    } else if m.path.segments.is_empty() {
                        match &m.path.root {
                            phalcom_ast::ast::ImportRoot::Absolute(seg) => seg.name.clone(),
                            phalcom_ast::ast::ImportRoot::Relative { .. } => String::new(),
                        }
                    } else {
                        m.path.segments.last().unwrap().name.clone()
                    };
                    if !name.is_empty() {
                        names.push(name);
                    }
                }
                phalcom_ast::ast::ImportDecl::Selective(s) => {
                    for item in &s.items {
                        let name = if let Some(alias) = &item.alias {
                            alias.name.clone()
                        } else {
                            item.name.clone()
                        };
                        names.push(name);
                    }
                }
            },
            phalcom_ast::ast::DependencyDecl::ReExport(r) => {
                for item in &r.items {
                    names.push(item.local_or_remote_name.clone());
                }
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
        Pattern::Variant { arguments, .. } => arguments.iter().for_each(|argument| collect_pattern_names(argument, out)),
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

    #[test]
    fn shallow_completion_reads_live_constructor_surface_before_worker_publish() {
        let uri = Url::parse("file:///completion.ph").unwrap();
        let source = "class Animal { move() {} }\nclass Dog is Animal { bark() {} }\nconst dog = Dog.new()\ndog.bark()\n";
        let doc = Document::new(source.to_string());
        let index = WorkspaceIndex::new();
        index.update_file(uri.clone(), &doc.parse.program);
        let items = shallow_receiver_completions_from_program(&index, &uri, &doc, &doc.parse.program, Position { line: 3, character: 4 }).unwrap();
        let labels = items.into_iter().map(|item| item.label.to_string()).collect::<Vec<_>>();
        assert!(labels.contains(&"bark()".to_string()), "{labels:?}");
        assert!(labels.contains(&"move()".to_string()), "{labels:?}");
    }

    #[test]
    fn shallow_receiver_resolves_constructor_assigned_field() {
        let source =
            "class Client {\n  send() { }\n}\nclass Service {\n  @constructor new() { _client = Client.new() }\n  run() {\n    _client.send()\n  }\n}\n";
        let program = phalcom_ast::parser::parse(source, 0).program;
        let offset = source.find("_client.send").unwrap() + "_client.".len();
        assert_eq!(shallow_receiver_classes(&program, "_client", offset), vec!["Client"]);
    }

    #[test]
    fn shallow_receiver_resolves_parameter_constructor_union() {
        let source = "class Circle { stroke() { } }\nclass Rectangle { fill() { } }\nclass Canvas { draw(_ shape) {\n    shape.stroke()\n  }\n}\ndraw(Circle.new())\ndraw(Rectangle.new())\n";
        let program = phalcom_ast::parser::parse(source, 0).program;
        let offset = source.find("shape.stroke").unwrap() + "shape.".len();
        assert_eq!(shallow_receiver_classes(&program, "shape", offset), vec!["Circle", "Rectangle"]);
    }
}
