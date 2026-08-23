//! Standard LSP runtime-value inlay hints.

use std::collections::HashSet;

use phalcom_ast::ast::{
    ClassMember, Expr, IndexAccessor, ListLiteralElement, MapLiteralEntry, MapLiteralKey, PackItem, PackLabel, Pattern, ProductLabel, Program,
    RecordLiteralEntry, SetLiteralEntry, Statement, TupleLiteralEntry,
};
use phalcom_common::range::SourceRange;
use tower_lsp::lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, InlayHintParams, InlayHintTooltip, MarkupContent, MarkupKind, Range, Url};

use crate::documents::{Document, DocumentSnapshot};
use crate::line_index::LineIndex;
use crate::request_context::RequestContext;
use crate::semantic::{Confidence, FileSemanticSnapshot, InferredValue, SemanticBindingKind, SemanticDb, SemanticSnapshot, ValueShape};

type SourceRangeKey = (usize, usize);

fn source_range_key(range: SourceRange) -> SourceRangeKey {
    (range.start, range.end)
}

/// Source-owned annotation facts used only to suppress duplicate advisory hints.
///
/// This index deliberately contains no inferred or formal semantic state. It is
/// rebuilt from the pinned program so explicit source annotations remain the
/// sole reason an advisory hint is suppressed.
#[derive(Default)]
struct ExplicitAnnotationIndex {
    binding_names: HashSet<SourceRangeKey>,
    parameter_names: HashSet<SourceRangeKey>,
    field_names: HashSet<SourceRangeKey>,
    return_members: HashSet<(usize, usize)>,
}

impl ExplicitAnnotationIndex {
    fn from_program(program: &Program) -> Self {
        let mut index = Self::default();
        for (statement_idx, statement) in program.statements.iter().enumerate() {
            collect_statement_annotations(statement, &mut index, statement_idx);
        }
        index
    }

    fn has_binding(&self, range: SourceRange) -> bool {
        self.binding_names.contains(&source_range_key(range))
    }

    fn has_parameter(&self, range: SourceRange) -> bool {
        self.parameter_names.contains(&source_range_key(range))
    }

    fn has_field(&self, range: SourceRange) -> bool {
        self.field_names.contains(&source_range_key(range))
    }

    fn has_return(&self, class_stmt_idx: usize, member_idx: usize) -> bool {
        self.return_members.contains(&(class_stmt_idx, member_idx))
    }
}

fn collect_pattern_names(pattern: &Pattern, names: &mut HashSet<SourceRangeKey>) {
    match pattern {
        Pattern::Name { range, .. } => {
            names.insert(source_range_key(*range));
        }
        Pattern::Tuple { elements, .. } => {
            for element in elements {
                collect_pattern_names(element, names);
            }
        }
        Pattern::List { elements, rest, .. } => {
            for element in elements {
                collect_pattern_names(element, names);
            }
            if let Some(rest) = rest {
                collect_pattern_names(rest, names);
            }
        }
        Pattern::Variant { arguments, .. } => {
            for argument in arguments {
                collect_pattern_names(argument, names);
            }
        }
        Pattern::Record { entries, .. } => {
            for entry in entries {
                collect_pattern_names(&entry.pattern, names);
            }
        }
        Pattern::Map { entries, .. } => {
            for entry in entries {
                collect_pattern_names(&entry.pattern, names);
            }
        }
    }
}

fn collect_statement_annotations(statement: &Statement, index: &mut ExplicitAnnotationIndex, statement_idx: usize) {
    match statement {
        Statement::Class(class_def) => {
            for (member_idx, member) in class_def.members.iter().enumerate() {
                match member {
                    ClassMember::Method(method) => {
                        for parameter in &method.params {
                            if parameter.annotation.is_some() {
                                index.parameter_names.insert(source_range_key(parameter.name_range));
                            }
                        }
                        if method.return_annotation.is_some() {
                            index.return_members.insert((statement_idx, member_idx));
                        }
                        for nested in method.body.statements().unwrap_or_default() {
                            collect_statement_annotations(nested, index, statement_idx);
                        }
                    }
                    ClassMember::Getter(getter) => {
                        if getter.return_annotation.is_some() {
                            index.return_members.insert((statement_idx, member_idx));
                        }
                        for nested in getter.body.statements().unwrap_or_default() {
                            collect_statement_annotations(nested, index, statement_idx);
                        }
                    }
                    ClassMember::Setter(setter) => {
                        if setter.param.annotation.is_some() {
                            index.parameter_names.insert(source_range_key(setter.param.name_range));
                        }
                        if setter.return_annotation.is_some() {
                            index.return_members.insert((statement_idx, member_idx));
                        }
                        for nested in setter.body.statements().unwrap_or_default() {
                            collect_statement_annotations(nested, index, statement_idx);
                        }
                    }
                    ClassMember::Field(field) => {
                        if field.annotation.is_some() {
                            index.field_names.insert(source_range_key(field.name_range));
                        }
                        if let Some(default) = &field.default {
                            collect_expr_annotations(default, index);
                        }
                    }
                    ClassMember::Variant(_) => {}
                    ClassMember::Index(index_method) => {
                        for parameter in &index_method.params {
                            if parameter.annotation.is_some() {
                                index.parameter_names.insert(source_range_key(parameter.name_range));
                            }
                        }
                        if let IndexAccessor::Set { put } = &index_method.accessor {
                            if put.annotation.is_some() {
                                index.parameter_names.insert(source_range_key(put.name_range));
                            }
                        }
                        if index_method.return_annotation.is_some() {
                            index.return_members.insert((statement_idx, member_idx));
                        }
                        for nested in &index_method.body {
                            collect_statement_annotations(nested, index, statement_idx);
                        }
                    }
                }
            }
            for (expr, _) in &class_def.invariants {
                collect_expr_annotations(expr, index);
            }
        }
        Statement::Let(binding) => {
            if binding.annotation.is_some() {
                collect_pattern_names(&binding.pattern, &mut index.binding_names);
            }
            if let Some(value) = &binding.value {
                collect_expr_annotations(value, index);
            }
        }
        Statement::Return(return_statement) => {
            if let Some(value) = &return_statement.value {
                collect_expr_annotations(value, index);
            }
        }
        Statement::Expr { expr, .. } | Statement::Throw { expr, .. } => collect_expr_annotations(expr, index),
        Statement::For(for_statement) => {
            for lane in &for_statement.lanes {
                collect_expr_annotations(&lane.iter, index);
            }
            for nested in &for_statement.body {
                collect_statement_annotations(nested, index, statement_idx);
            }
        }
        Statement::Break { .. } | Statement::Continue { .. } | Statement::Export(_) | Statement::TypeAlias(_) => {}
    }
}

fn collect_expr_annotations(expr: &Expr, index: &mut ExplicitAnnotationIndex) {
    match expr {
        Expr::Assignment(assignment) => {
            collect_expr_annotations(&assignment.name, index);
            collect_expr_annotations(&assignment.value, index);
        }
        Expr::Range(range) => {
            if let Some(lower) = &range.lower {
                collect_expr_annotations(lower, index);
            }
            if let Some(upper) = &range.upper {
                collect_expr_annotations(upper, index);
            }
        }
        Expr::Unary(unary) => collect_expr_annotations(&unary.expr, index),
        Expr::Binary(binary) => {
            collect_expr_annotations(&binary.left, index);
            collect_expr_annotations(&binary.right, index);
        }
        Expr::ComparisonChain(chain) => {
            for operand in &chain.operands {
                collect_expr_annotations(operand, index);
            }
        }
        Expr::Membership(membership) => {
            collect_expr_annotations(&membership.left, index);
            collect_expr_annotations(&membership.right, index);
        }
        Expr::IsMembership(membership) => {
            collect_expr_annotations(&membership.left, index);
            collect_expr_annotations(&membership.candidates, index);
        }
        Expr::IfLet(if_let) => {
            collect_expr_annotations(&if_let.value, index);
            for nested in &if_let.then_body.body {
                collect_statement_annotations(nested, index, usize::MAX);
            }
            if let Some(else_body) = &if_let.else_body {
                for nested in &else_body.body {
                    collect_statement_annotations(nested, index, usize::MAX);
                }
            }
        }
        Expr::WhileLet(while_let) => {
            collect_expr_annotations(&while_let.value, index);
            for nested in &while_let.body {
                collect_statement_annotations(nested, index, usize::MAX);
            }
        }
        Expr::UnqualifiedCall(call) => collect_pack_annotations(&call.args, index),
        Expr::MethodCall(call) => {
            collect_expr_annotations(&call.object, index);
            collect_pack_annotations(&call.args, index);
        }
        Expr::GetProperty(property) => collect_expr_annotations(&property.object, index),
        Expr::SetProperty(property) => {
            collect_expr_annotations(&property.object, index);
            collect_expr_annotations(&property.value, index);
        }
        Expr::Index(index_expr) => {
            collect_expr_annotations(&index_expr.object, index);
            collect_pack_annotations(&index_expr.args, index);
        }
        Expr::SetIndex(index_expr) => {
            collect_expr_annotations(&index_expr.object, index);
            collect_pack_annotations(&index_expr.args, index);
            collect_expr_annotations(&index_expr.value, index);
        }
        Expr::Block(block) => {
            for nested in &block.body {
                collect_statement_annotations(nested, index, usize::MAX);
            }
        }
        Expr::MethodRef(method_ref) => collect_expr_annotations(&method_ref.receiver, index),
        Expr::TupleLiteral(tuple) => {
            for entry in &tuple.entries {
                match entry {
                    TupleLiteralEntry::Positional { expr, .. } | TupleLiteralEntry::Expand { expr, .. } => collect_expr_annotations(expr, index),
                    TupleLiteralEntry::Labeled { label, value, .. } => {
                        collect_product_label_annotations(label, index);
                        collect_expr_annotations(value, index);
                    }
                }
            }
        }
        Expr::RecordLiteral(record) => {
            for entry in &record.entries {
                match entry {
                    RecordLiteralEntry::Field(field) => {
                        collect_product_label_annotations(&field.label, index);
                        collect_expr_annotations(&field.value, index);
                    }
                    RecordLiteralEntry::Expansion { expr, .. } => collect_expr_annotations(expr, index),
                }
            }
        }
        Expr::MapLiteral(map) => {
            for entry in &map.entries {
                match entry {
                    MapLiteralEntry::Association { key, value, .. } => {
                        if let MapLiteralKey::Computed { expr, .. } = key {
                            collect_expr_annotations(expr, index);
                        }
                        collect_expr_annotations(value, index);
                    }
                    MapLiteralEntry::Expansion { expr, .. } => collect_expr_annotations(expr, index),
                }
            }
        }
        Expr::SetLiteral(set) => {
            for entry in &set.entries {
                match entry {
                    SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => collect_expr_annotations(expr, index),
                }
            }
        }
        Expr::ListLiteral(list) => {
            for element in &list.elements {
                match element {
                    ListLiteralElement::Element { expr, .. } | ListLiteralElement::Expansion { expr, .. } => collect_expr_annotations(expr, index),
                }
            }
        }
        Expr::Int { .. }
        | Expr::Float { .. }
        | Expr::String { .. }
        | Expr::Boolean { .. }
        | Expr::Var { .. }
        | Expr::Field { .. }
        | Expr::SelfVar { .. }
        | Expr::SuperVar { .. }
        | Expr::ImplementationSelector { .. }
        | Expr::Symbol { .. }
        | Expr::Ellipsis { .. }
        | Expr::TypeForm(_) => {}
    }
}

fn collect_pack_annotations(items: &[PackItem], index: &mut ExplicitAnnotationIndex) {
    for item in items {
        match item {
            PackItem::Positional { expr, .. } | PackItem::Expand { expr, .. } => collect_expr_annotations(expr, index),
            PackItem::Labeled { label, value, .. } => {
                if let PackLabel::Computed { expr, .. } = label {
                    collect_expr_annotations(expr, index);
                }
                collect_expr_annotations(value, index);
            }
        }
    }
}

fn collect_product_label_annotations(label: &ProductLabel, index: &mut ExplicitAnnotationIndex) {
    if let ProductLabel::Computed { expr, .. } = label {
        collect_expr_annotations(expr, index);
    }
}

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

/// Computes stable type-style hints for visible declarations.
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
    let global_snapshot = db.snapshot();
    collect_file_semantic_hints(
        &snapshot,
        Some(&global_snapshot),
        &doc.text,
        &doc.line_index,
        visible_start,
        visible_end,
        policy,
        suppress_obvious,
        &mut hints,
    );
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
    collect_file_semantic_hints(
        snapshot,
        Some(&request.semantic),
        &request.document.text,
        &request.document.line_index,
        visible_start,
        visible_end,
        policy,
        suppress_obvious,
        &mut hints,
    );
    hints.sort_by_key(|hint| (hint.position.line, hint.position.character));
    hints
}

// These inputs are intentionally explicit: each is independently sourced from the pinned request snapshot.
#[allow(clippy::too_many_arguments)]
fn collect_file_semantic_hints(
    file_snapshot: &FileSemanticSnapshot,
    global_snapshot: Option<&SemanticSnapshot>,
    text: &str,
    line_index: &LineIndex,
    visible_start: usize,
    visible_end: usize,
    policy: HintPolicy,
    suppress_obvious: bool,
    hints: &mut Vec<InlayHint>,
) {
    let annotations = ExplicitAnnotationIndex::from_program(&file_snapshot.source.program);

    // 1. Local bindings (let/const)
    for binding in file_snapshot.source.scopes.bindings.values() {
        if binding.kind == SemanticBindingKind::Import {
            continue;
        }
        let range = binding.declaration_range;
        if range.end < visible_start || range.start > visible_end {
            continue;
        }
        if annotations.has_binding(range) {
            continue;
        }
        let Some(value) = file_snapshot.local_facts.value_before(binding.id, range.end.saturating_add(1)) else {
            continue;
        };
        let formal = global_snapshot.and_then(|snap| {
            let uri = snap.documents.uri_for_lsp(&file_snapshot.module)?;
            snap.formal_binding_presentation_at(&uri, &binding.name, range.end)
        });
        if formal.is_none() && (!should_render(policy, &value.confidence, &value.shape) || (suppress_obvious && obvious_initializer_text(text, range))) {
            continue;
        }
        let formal_text = formal.as_ref().map(|presentation| presentation.text());
        crate::parity::ShadowParityHarness::new().record_inlay_hint_parity(&binding.name, formal_text.as_deref(), Some(&render_shape(&value.shape)));

        let (label, tooltip) = if let Some(formal) = formal.as_ref() {
            (
                format!(": {}", formal.text()),
                format!(
                    "Formal semantic result: `{}`\n\nThis result is compiler-owned and authoritative.",
                    formal.text()
                ),
            )
        } else {
            let rendered = render_shape(&value.shape);
            (
                format!("≈ {rendered}"),
                format!(
                    "Observed runtime value: {rendered}\n\nConfidence: {}\n\nThis is editor inference, not a Phalcom type annotation.",
                    confidence_name(value.confidence)
                ),
            )
        };
        hints.push(InlayHint {
            position: line_index.position(range.end),
            label: InlayHintLabel::String(label),
            kind: Some(InlayHintKind::TYPE),
            text_edits: None,
            tooltip: Some(InlayHintTooltip::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: tooltip,
            })),
            padding_left: Some(true),
            padding_right: None,
            data: None,
        });
    }

    // 2. Class fields
    for class in file_snapshot.source.surface.classes.values() {
        for (field_name, field_sides) in &class.fields {
            for (side, field_surface) in [
                (crate::semantic::DispatchSide::Instance, field_sides.instance.as_ref()),
                (crate::semantic::DispatchSide::Class, field_sides.class.as_ref()),
            ] {
                let Some(f) = field_surface else { continue };
                if f.source_range.end < visible_start || f.source_range.start > visible_end {
                    continue;
                }
                if annotations.has_field(f.name_range) {
                    continue;
                }
                let field_value = global_snapshot.and_then(|db| db.field_value(&class.id, field_name, side)).or_else(|| {
                    let stmt = file_snapshot.source.program.statements.get(f.ast.class_stmt_idx)?;
                    let Statement::Class(class_def) = stmt else { return None };
                    let ClassMember::Field(field_def) = class_def.members.get(f.ast.member_idx)? else {
                        return None;
                    };
                    field_def
                        .default
                        .as_ref()
                        .and_then(|def| shallow_expression_shape(def, &class.id.module))
                        .map(|shape| InferredValue::exact(shape, field_def.name_range))
                });
                if let Some(val) = field_value {
                    if should_render(policy, &val.confidence, &val.shape) {
                        let rendered = render_shape(&val.shape);
                        hints.push(InlayHint {
                            position: line_index.position(f.name_range.end),
                            label: InlayHintLabel::String(format!("≈ {rendered}")),
                            kind: Some(InlayHintKind::TYPE),
                            text_edits: None,
                            tooltip: Some(InlayHintTooltip::MarkupContent(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: format!(
                                    "Inferred runtime value: {rendered}\n\nConfidence: {}\n\nThis is editor inference, not a Phalcom type annotation.",
                                    confidence_name(val.confidence)
                                ),
                            })),
                            padding_left: Some(true),
                            padding_right: None,
                            data: None,
                        });
                    }
                }
            }
        }

        // 3. Class members (Parameters, Returns)
        for member in class.all_members() {
            if member.kind == crate::semantic::MemberKind::Field {
                continue;
            }
            // Callable parameters
            for param in &member.params {
                if param.name_range.end < visible_start || param.name_range.start > visible_end {
                    continue;
                }
                if annotations.has_parameter(param.name_range) {
                    continue;
                }
                let param_val = global_snapshot.and_then(|db| db.parameter_at(&member.callable, &param.name));
                if let Some(val) = param_val {
                    if should_render(policy, &val.confidence, &val.shape) {
                        let rendered = render_shape(&val.shape);
                        hints.push(InlayHint {
                            position: line_index.position(param.name_range.end),
                            label: InlayHintLabel::String(format!("≈ {rendered}")),
                            kind: Some(InlayHintKind::TYPE),
                            text_edits: None,
                            tooltip: Some(InlayHintTooltip::MarkupContent(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: format!(
                                    "Inferred runtime value: {rendered}\n\nConfidence: {}\n\nThis is editor inference, not a Phalcom type annotation.",
                                    confidence_name(val.confidence)
                                ),
                            })),
                            padding_left: Some(true),
                            padding_right: None,
                            data: None,
                        });
                    }
                }
            }

            // Callable return type
            let return_val = global_snapshot.and_then(|db| db.return_for_callable(&member.callable));
            if let Some(ret) = return_val {
                if should_render(policy, &ret.confidence, &ret.shape) {
                    let ret_pos = member.ast.and_then(|ast| {
                        if annotations.has_return(ast.class_stmt_idx, ast.member_idx) {
                            return None;
                        }
                        find_return_hint_offset(
                            &file_snapshot.source.program,
                            ast.class_stmt_idx,
                            ast.member_idx,
                            member.kind,
                            member.name_range,
                            text,
                        )
                    });
                    if let Some(offset) = ret_pos {
                        if offset >= visible_start && offset <= visible_end {
                            let rendered = render_shape(&ret.shape);
                            hints.push(InlayHint {
                                position: line_index.position(offset),
                                label: InlayHintLabel::String(format!(" ≈ {rendered}")),
                                kind: Some(InlayHintKind::TYPE),
                                text_edits: None,
                                tooltip: Some(InlayHintTooltip::MarkupContent(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value: format!(
                                        "Inferred return value: {rendered}\n\nConfidence: {}\n\nThis is editor inference, not a Phalcom type annotation.",
                                        confidence_name(ret.confidence)
                                    ),
                                })),
                                padding_left: Some(true),
                                padding_right: None,
                                data: None,
                            });
                        }
                    }
                }
            }
        }
    }

    // 4. Closure expression parameters in statements
    for stmt in &file_snapshot.source.program.statements {
        collect_statement_closure_hints(stmt, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_statement_closure_hints(
    stmt: &Statement,
    file_snapshot: &FileSemanticSnapshot,
    text: &str,
    line_index: &LineIndex,
    visible_start: usize,
    visible_end: usize,
    policy: HintPolicy,
    hints: &mut Vec<InlayHint>,
) {
    match stmt {
        Statement::Let(binding) => {
            if let Some(expr) = &binding.value {
                collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            }
        }
        Statement::Expr { expr, .. } => {
            collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
        }
        Statement::Class(class_def) => {
            for member in &class_def.members {
                match member {
                    ClassMember::Method(m) => {
                        for s in m.body.statements().unwrap_or_default() {
                            collect_statement_closure_hints(s, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                        }
                    }
                    ClassMember::Getter(g) => {
                        for s in g.body.statements().unwrap_or_default() {
                            collect_statement_closure_hints(s, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                        }
                    }
                    ClassMember::Setter(s) => {
                        for st in s.body.statements().unwrap_or_default() {
                            collect_statement_closure_hints(st, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                        }
                    }
                    ClassMember::Field(f) => {
                        if let Some(default_expr) = &f.default {
                            collect_expr_closure_hints(default_expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                        }
                    }
                    ClassMember::Index(ix) => {
                        for s in &ix.body {
                            collect_statement_closure_hints(s, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                        }
                    }
                    ClassMember::Variant(_) => {}
                }
            }
        }
        Statement::Return(r) => {
            if let Some(expr) = &r.value {
                collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            }
        }
        Statement::For(for_stmt) => {
            for lane in &for_stmt.lanes {
                collect_expr_closure_hints(&lane.iter, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            }
            for s in &for_stmt.body {
                collect_statement_closure_hints(s, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            }
        }
        Statement::Throw { expr, .. } => {
            collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_expr_closure_hints(
    expr: &Expr,
    file_snapshot: &FileSemanticSnapshot,
    text: &str,
    line_index: &LineIndex,
    visible_start: usize,
    visible_end: usize,
    policy: HintPolicy,
    hints: &mut Vec<InlayHint>,
) {
    if expr.range().end < visible_start || expr.range().start > visible_end {
        return;
    }
    match expr {
        Expr::Block(block) => {
            for param in &block.params.fixed {
                if param.range.end >= visible_start && param.range.start <= visible_end {
                    let binding_fact = file_snapshot
                        .source
                        .scopes
                        .bindings
                        .values()
                        .find(|b| b.name == param.name && b.declaration_range == param.range)
                        .and_then(|b| file_snapshot.local_facts.value_before(b.id, param.range.end.saturating_add(1)));
                    if let Some(val) = binding_fact {
                        if should_render(policy, &val.confidence, &val.shape) {
                            let rendered = render_shape(&val.shape);
                            hints.push(InlayHint {
                                position: line_index.position(param.range.end),
                                label: InlayHintLabel::String(format!("≈ {rendered}")),
                                kind: Some(InlayHintKind::TYPE),
                                text_edits: None,
                                tooltip: Some(InlayHintTooltip::MarkupContent(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value: format!(
                                        "Inferred runtime value: {rendered}\n\nConfidence: {}\n\nThis is editor inference, not a Phalcom type annotation.",
                                        confidence_name(val.confidence)
                                    ),
                                })),
                                padding_left: Some(true),
                                padding_right: None,
                                data: None,
                            });
                        }
                    }
                }
            }
            if let Some(param) = &block.params.positional_rest {
                if param.range.end >= visible_start && param.range.start <= visible_end {
                    let binding_fact = file_snapshot
                        .source
                        .scopes
                        .bindings
                        .values()
                        .find(|b| b.name == param.name && b.declaration_range == param.range)
                        .and_then(|b| file_snapshot.local_facts.value_before(b.id, param.range.end.saturating_add(1)));
                    if let Some(val) = binding_fact {
                        if should_render(policy, &val.confidence, &val.shape) {
                            let rendered = render_shape(&val.shape);
                            hints.push(InlayHint {
                                position: line_index.position(param.range.end),
                                label: InlayHintLabel::String(format!("≈ {rendered}")),
                                kind: Some(InlayHintKind::TYPE),
                                text_edits: None,
                                tooltip: Some(InlayHintTooltip::MarkupContent(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value: format!(
                                        "Inferred runtime value: {rendered}\n\nConfidence: {}\n\nThis is editor inference, not a Phalcom type annotation.",
                                        confidence_name(val.confidence)
                                    ),
                                })),
                                padding_left: Some(true),
                                padding_right: None,
                                data: None,
                            });
                        }
                    }
                }
            }
            for s in &block.body {
                collect_statement_closure_hints(s, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            }
        }
        Expr::MethodCall(call) => {
            collect_expr_closure_hints(&call.object, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            for item in &call.args {
                match item {
                    PackItem::Positional { expr, .. } => {
                        collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                    }
                    PackItem::Labeled { value, .. } => {
                        collect_expr_closure_hints(value, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                    }
                    PackItem::Expand { expr, .. } => {
                        collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                    }
                }
            }
        }
        Expr::UnqualifiedCall(call) => {
            for item in &call.args {
                match item {
                    PackItem::Positional { expr, .. } => {
                        collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                    }
                    PackItem::Labeled { value, .. } => {
                        collect_expr_closure_hints(value, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                    }
                    PackItem::Expand { expr, .. } => {
                        collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                    }
                }
            }
        }
        Expr::Assignment(a) => {
            collect_expr_closure_hints(&a.name, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            collect_expr_closure_hints(&a.value, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
        }
        Expr::Unary(u) => collect_expr_closure_hints(&u.expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints),
        Expr::Binary(b) => {
            collect_expr_closure_hints(&b.left, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            collect_expr_closure_hints(&b.right, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
        }
        Expr::ComparisonChain(c) => {
            for operand in &c.operands {
                collect_expr_closure_hints(operand, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            }
        }
        Expr::Range(r) => {
            if let Some(e) = &r.lower {
                collect_expr_closure_hints(e, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            }
            if let Some(e) = &r.upper {
                collect_expr_closure_hints(e, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            }
        }
        Expr::Index(ix) => {
            collect_expr_closure_hints(&ix.object, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            for item in &ix.args {
                match item {
                    PackItem::Positional { expr, .. } => {
                        collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                    }
                    PackItem::Labeled { value, .. } => {
                        collect_expr_closure_hints(value, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                    }
                    PackItem::Expand { expr, .. } => {
                        collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                    }
                }
            }
        }
        Expr::SetIndex(ix) => {
            collect_expr_closure_hints(&ix.object, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            for item in &ix.args {
                match item {
                    PackItem::Positional { expr, .. } => {
                        collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                    }
                    PackItem::Labeled { value, .. } => {
                        collect_expr_closure_hints(value, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                    }
                    PackItem::Expand { expr, .. } => {
                        collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                    }
                }
            }
            collect_expr_closure_hints(&ix.value, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
        }
        Expr::TupleLiteral(t) => {
            for entry in &t.entries {
                match entry {
                    TupleLiteralEntry::Positional { expr, .. } => {
                        collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints)
                    }
                    TupleLiteralEntry::Labeled { value, .. } => {
                        collect_expr_closure_hints(value, file_snapshot, text, line_index, visible_start, visible_end, policy, hints)
                    }
                    TupleLiteralEntry::Expand { expr, .. } => {
                        collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints)
                    }
                }
            }
        }
        Expr::RecordLiteral(r) => {
            for entry in &r.entries {
                match entry {
                    RecordLiteralEntry::Field(f) => {
                        collect_expr_closure_hints(&f.value, file_snapshot, text, line_index, visible_start, visible_end, policy, hints)
                    }
                    RecordLiteralEntry::Expansion { expr, .. } => {
                        collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints)
                    }
                }
            }
        }
        Expr::ListLiteral(l) => {
            for elem in &l.elements {
                match elem {
                    ListLiteralElement::Element { expr, .. } | ListLiteralElement::Expansion { expr, .. } => {
                        collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                    }
                }
            }
        }
        Expr::SetLiteral(s) => {
            for entry in &s.entries {
                match entry {
                    SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => {
                        collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                    }
                }
            }
        }
        Expr::MapLiteral(m) => {
            for entry in &m.entries {
                match entry {
                    MapLiteralEntry::Association { key, value, .. } => {
                        if let MapLiteralKey::Computed { expr, .. } = key {
                            collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                        }
                        collect_expr_closure_hints(value, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                    }
                    MapLiteralEntry::Expansion { expr, .. } => {
                        collect_expr_closure_hints(expr, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                    }
                }
            }
        }
        Expr::IfLet(if_let) => {
            collect_expr_closure_hints(&if_let.value, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            for statement in &if_let.then_body.body {
                collect_statement_closure_hints(statement, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            }
            if let Some(else_body) = &if_let.else_body {
                for statement in &else_body.body {
                    collect_statement_closure_hints(statement, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
                }
            }
        }
        Expr::WhileLet(while_let) => {
            collect_expr_closure_hints(&while_let.value, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            for statement in &while_let.body {
                collect_statement_closure_hints(statement, file_snapshot, text, line_index, visible_start, visible_end, policy, hints);
            }
        }
        _ => {}
    }
}

fn find_return_hint_offset(
    program: &Program,
    class_stmt_idx: usize,
    member_idx: usize,
    _member_kind: crate::semantic::MemberKind,
    _name_range: SourceRange,
    text: &str,
) -> Option<usize> {
    let stmt = program.statements.get(class_stmt_idx)?;
    let Statement::Class(class_def) = stmt else { return None };
    let member = class_def.members.get(member_idx)?;
    match member {
        ClassMember::Method(m) => {
            // Find closing ')' after params
            let search_start = m.params.last().map(|p| p.range.end).unwrap_or(m.name_range.end);
            text[search_start..m.range.end].find(')').map(|offset| search_start + offset + 1)
        }
        ClassMember::Getter(g) => Some(g.name_range.end),
        ClassMember::Setter(_) => None,
        ClassMember::Field(_) => None,
        ClassMember::Variant(_) => None,
        ClassMember::Index(ix) => match &ix.accessor {
            IndexAccessor::Get => {
                let search_start = ix.params.last().map(|p| p.range.end).unwrap_or(ix.name_range.start);
                text[search_start..ix.range.end].find(']').map(|offset| search_start + offset + 1)
            }
            IndexAccessor::Set { .. } => None,
        },
    }
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
    shallow_hints_internal(
        &doc.parse.program,
        &doc.text,
        &doc.line_index,
        &module,
        visible_start,
        visible_end,
        policy,
        suppress_obvious,
    )
}

/// Produces exact source-local facts while the worker has not published the
/// first semantic file snapshot. Once a snapshot exists, callers must use its
/// revision-matched facts or receive no hints.
fn shallow_hints(doc: &Document, uri: &Url, visible_start: usize, visible_end: usize, policy: HintPolicy, suppress_obvious: bool) -> Vec<InlayHint> {
    let module = crate::semantic::ModuleId::new(uri.to_string());
    shallow_hints_internal(
        &doc.parse.program,
        &doc.text,
        &doc.line_index,
        &module,
        visible_start,
        visible_end,
        policy,
        suppress_obvious,
    )
}

#[allow(clippy::too_many_arguments)]
fn shallow_hints_internal(
    program: &Program,
    text: &str,
    line_index: &LineIndex,
    module: &crate::semantic::ModuleId,
    visible_start: usize,
    visible_end: usize,
    policy: HintPolicy,
    suppress_obvious: bool,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    for (class_stmt_idx, statement) in program.statements.iter().enumerate() {
        match statement {
            Statement::Let(binding) => {
                let Pattern::Name { .. } = &binding.pattern else { continue };
                if binding.annotation.is_some() {
                    continue;
                }
                let Some(value) = binding.value.as_ref() else { continue };
                let Some(shape) = shallow_expression_shape(value, module) else { continue };
                if binding.range.end < visible_start || binding.range.start > visible_end || !should_render(policy, &Confidence::Exact, &shape) {
                    continue;
                }
                if suppress_obvious && obvious_initializer_text(text, binding.range) {
                    continue;
                }
                let rendered = render_shape(&shape);
                hints.push(InlayHint {
                    position: line_index.position(binding.range.end),
                    label: InlayHintLabel::String(format!("≈ {rendered}")),
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
            Statement::Class(class_def) => {
                for (member_idx, member) in class_def.members.iter().enumerate() {
                    match member {
                        ClassMember::Field(f) => {
                            if f.range.end < visible_start || f.range.start > visible_end {
                                continue;
                            }
                            if f.annotation.is_some() {
                                continue;
                            }
                            let shape = f.default.as_ref().and_then(|def| shallow_expression_shape(def, module));
                            if let Some(shape) = shape {
                                if should_render(policy, &Confidence::Exact, &shape) {
                                    let rendered = render_shape(&shape);
                                    hints.push(InlayHint {
                                        position: line_index.position(f.name_range.end),
                                        label: InlayHintLabel::String(format!("≈ {rendered}")),
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
                            }
                        }
                        ClassMember::Method(m)
                            // Constructor return hint in shallow mode
                            if m.is_constructor && m.return_annotation.is_none() => {
                                let shape = ValueShape::Instance(crate::semantic::ClassId::new(module.clone(), class_def.name.clone()));
                                if should_render(policy, &Confidence::Exact, &shape) {
                                    let offset =
                                        find_return_hint_offset(program, class_stmt_idx, member_idx, crate::semantic::MemberKind::Method, m.name_range, text);
                                    if let Some(offset) = offset {
                                        if offset >= visible_start && offset <= visible_end {
                                            let rendered = render_shape(&shape);
                                            hints.push(InlayHint {
                                                position: line_index.position(offset),
                                                label: InlayHintLabel::String(format!(" ≈ {rendered}")),
                                                kind: Some(InlayHintKind::TYPE),
                                                text_edits: None,
                                                tooltip: Some(InlayHintTooltip::MarkupContent(MarkupContent {
                                                    kind: MarkupKind::Markdown,
                                                    value: format!("Inferred return value: {rendered}\n\nConfidence: exact\n\nThis is editor inference, not a Phalcom type annotation."),
                                                })),
                                                padding_left: Some(true),
                                                padding_right: None,
                                                data: None,
                                            });
                                        }
                                    }
                                }
                            }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
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
        assert!(matches!(&hints[0].label, InlayHintLabel::String(label) if label == "≈ String"));
        assert_eq!(hints[0].kind, Some(InlayHintKind::TYPE));
    }

    #[test]
    fn explicit_binding_annotation_suppresses_duplicate_hint() {
        let uri = Url::parse("file:///annotated.ph").unwrap();
        let doc = Document::new("let annotated: Int = 1\nlet inferred = 2\n".to_string());
        assert!(doc.parse.errors.is_empty(), "parse errors: {:?}", doc.parse.errors);
        let db = SemanticDb::new();
        db.update_file(&uri, FileRevision(1), &doc.parse.program);

        let hints = hints_for(
            &db,
            &uri,
            &doc,
            Range {
                start: Position::new(0, 0),
                end: Position::new(5, 0),
            },
        );

        let labels = hints
            .iter()
            .filter_map(|hint| match &hint.label {
                InlayHintLabel::String(label) => Some(label.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(labels, vec!["≈ Int"], "only unannotated binding should receive a hint: {hints:?}");
    }

    #[test]
    fn explicit_member_annotations_suppress_field_parameter_and_return_hints() {
        let uri = Url::parse("file:///annotated-members.ph").unwrap();
        let source = "class Service {\n  const _annotated: Int = 1\n  const _inferred = 2\n  compute(value: Int) -> Int { 42 }\n  infer(value) { 42 }\n}\n";
        let doc = Document::new(source.to_string());
        assert!(doc.parse.errors.is_empty(), "parse errors: {:?}", doc.parse.errors);
        let db = SemanticDb::new();
        db.update_core(FileRevision(1), &crate::semantic::core_source::bundled_parse().program);
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
        let labels = hints
            .iter()
            .filter_map(|hint| match &hint.label {
                InlayHintLabel::String(label) => Some(label.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            labels.iter().filter(|label| **label == "≈ Int").count(),
            1,
            "only _inferred should get ≈ Int: {labels:?}"
        );
        assert_eq!(
            labels.iter().filter(|label| **label == " ≈ Int").count(),
            1,
            "only infer should get ≈ Int: {labels:?}"
        );
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
        assert!(matches!(&hints[0].label, InlayHintLabel::String(label) if label == "≈ Int"));
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

    #[test]
    fn field_type_hint_annotated_on_declaration() {
        let uri = Url::parse("file:///main.ph").unwrap();
        let doc = Document::new("class Point {\n  _x = 1\n  _y = \"a\"\n}\n".to_string());
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

        let x_hint = hints.iter().find(|h| matches!(&h.label, InlayHintLabel::String(s) if s == "≈ Int"));
        assert!(x_hint.is_some(), "Field _x should have ≈ Int hint, got: {hints:?}");
        let y_hint = hints.iter().find(|h| matches!(&h.label, InlayHintLabel::String(s) if s == "≈ String"));
        assert!(y_hint.is_some(), "Field _y should have ≈ String hint, got: {hints:?}");
    }

    #[test]
    fn parameter_and_return_inlay_hints() {
        let uri = Url::parse("file:///main.ph").unwrap();
        let source = "class Service {\n  compute(_ x, *rest, label y) {\n    42\n  }\n  name {\n    \"srv\"\n  }\n  [idx] {\n    true\n  }\n}\n";
        let doc = Document::new(source.to_string());
        let db = SemanticDb::new();
        let bundled = crate::semantic::core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
        db.update_file(&uri, FileRevision(1), &doc.parse.program);

        let hints = hints_for(
            &db,
            &uri,
            &doc,
            Range {
                start: Position::new(0, 0),
                end: Position::new(20, 100),
            },
        );

        let method_ret = hints.iter().find(|h| matches!(&h.label, InlayHintLabel::String(s) if s == " ≈ Int"));
        assert!(method_ret.is_some(), "Method compute should have ≈ Int return hint, got: {hints:?}");

        let getter_ret = hints.iter().find(|h| matches!(&h.label, InlayHintLabel::String(s) if s == " ≈ String"));
        assert!(getter_ret.is_some(), "Getter name should have ≈ String return hint, got: {hints:?}");

        let index_ret = hints.iter().find(|h| matches!(&h.label, InlayHintLabel::String(s) if s == " ≈ Bool"));
        assert!(index_ret.is_some(), "Index getter should have ≈ Bool return hint, got: {hints:?}");
    }
}
