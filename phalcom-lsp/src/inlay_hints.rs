//! Standard LSP runtime-value inlay hints.

use std::collections::HashSet;

use phalcom_ast::ast::{
    ClassMember, Expr, IndexAccessor, ListLiteralElement, MapLiteralEntry, MapLiteralKey, PackItem, PackLabel, Pattern, ProductLabel, Program,
    RecordLiteralEntry, SetLiteralEntry, Statement, TupleLiteralEntry,
};
use phalcom_common::range::SourceRange;
use tower_lsp::lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, InlayHintTooltip, MarkupContent, MarkupKind, Range};

use crate::line_index::LineIndex;
use crate::request_context::RequestContext;
use phalcom_semantic::SemanticSnapshot;

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
    canonical_hints_for_request(
        snapshot,
        module,
        &request.document.parse.program,
        &request.document.text,
        &request.document.line_index,
        visible_start,
        visible_end,
        policy,
        suppress_obvious,
    )
}

/// Builds request-facing hints exclusively from one canonical compiler
/// snapshot. AST data supplies annotation suppression and source placement;
/// every displayed type/value comes from formal or advisory products.
#[allow(clippy::too_many_arguments)]
fn canonical_hints_for_request(
    snapshot: &phalcom_semantic::SemanticSnapshot,
    module: &phalcom_modules::ModuleId,
    program: &Program,
    text: &str,
    line_index: &LineIndex,
    visible_start: usize,
    visible_end: usize,
    policy: HintPolicy,
    suppress_obvious: bool,
) -> Vec<InlayHint> {
    let Some(source) = snapshot.source_index().module(module) else {
        return Vec::new();
    };
    let annotations = ExplicitAnnotationIndex::from_program(program);
    let presenter = phalcom_semantic::TypePresenter::new(&snapshot.store);
    let mut hints = Vec::new();

    for binding in source.structure.bindings.values() {
        if binding.kind == phalcom_semantic::source_index::SourceBindingKind::Import
            || binding.declaration_range.end < visible_start
            || binding.declaration_range.start > visible_end
            || annotations.has_binding(binding.declaration_range)
        {
            continue;
        }
        let formal = snapshot
            .formal_fact_at(module, binding.declaration_range.start)
            .and_then(|fact| canonical_formal_for_binding(snapshot, fact, &presenter));
        let advisory = snapshot.advisory_fact(&binding.declaration_site);
        if suppress_obvious && formal.is_none() && obvious_initializer_text(text, binding.declaration_range) {
            continue;
        }
        push_canonical_hint(&mut hints, line_index, binding.declaration_range.end, formal, advisory, policy, false);
    }

    for field in source.structure.field_sources.values() {
        if field.declaration_range.end < visible_start || field.declaration_range.start > visible_end || field.has_explicit_annotation {
            continue;
        }
        push_canonical_hint(
            &mut hints,
            line_index,
            field.name_range.end,
            None,
            snapshot.advisory().field(&field.id),
            policy,
            false,
        );
    }

    for callable in source.structure.callable_sources.values() {
        let Some(signature) = snapshot.callable_signatures.get(&callable.id) else {
            continue;
        };
        let advisory = snapshot.advisory_callable(&callable.id);
        for parameter in signature.parameters.iter() {
            let Some(name_range) = callable.parameter_name_ranges.get(parameter.index as usize).copied() else {
                continue;
            };
            if name_range.end < visible_start || name_range.start > visible_end || annotations.has_parameter(name_range) {
                continue;
            }
            let formal = canonical_formal_for_term(&parameter.ty, &presenter);
            let advisory = advisory.and_then(|summary| summary.parameters.iter().find(|(slot, _)| slot.index == parameter.index).map(|(_, fact)| fact));
            push_canonical_hint(&mut hints, line_index, name_range.end, formal, advisory, policy, false);
        }

        if !callable.has_explicit_return_annotation {
            let formal = canonical_formal_for_term(&signature.return_type, &presenter);
            let advisory = advisory.map(|summary| &summary.return_fact);
            let offset = source
                .structure
                .callable_body_ranges
                .get(&callable.id)
                .map_or(callable.declaration_range.end, |range| range.end);
            if offset >= visible_start && offset <= visible_end {
                push_canonical_hint(&mut hints, line_index, offset, formal, advisory, policy, true);
            }
        }
    }

    hints.sort_by_key(|hint| (hint.position.line, hint.position.character));
    hints
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

fn canonical_formal_for_binding(
    snapshot: &SemanticSnapshot,
    fact: &phalcom_semantic::FormalFactSite,
    presenter: &phalcom_semantic::TypePresenter<'_>,
) -> Option<phalcom_semantic::FormalPresentation> {
    let knowledge = match &fact.fact {
        phalcom_semantic::FormalFactRef::Binding { callable, binding } => snapshot.formal_binding(callable, *binding)?.current.clone(),
        _ => return None,
    };
    Some(match fact.status {
        phalcom_semantic::FormalFactStatus::Ready => presenter.present_knowledge(&knowledge),
        phalcom_semantic::FormalFactStatus::Dynamic => phalcom_semantic::FormalPresentation::Dynamic,
        phalcom_semantic::FormalFactStatus::Invalid | phalcom_semantic::FormalFactStatus::InvalidMultiple => phalcom_semantic::FormalPresentation::Invalid,
        phalcom_semantic::FormalFactStatus::Blocked => phalcom_semantic::FormalPresentation::Blocked,
        phalcom_semantic::FormalFactStatus::Cancelled => phalcom_semantic::FormalPresentation::Cancelled,
        phalcom_semantic::FormalFactStatus::BudgetExceeded => phalcom_semantic::FormalPresentation::BudgetExceeded,
        phalcom_semantic::FormalFactStatus::InternalFailure => phalcom_semantic::FormalPresentation::InternalFailure,
        phalcom_semantic::FormalFactStatus::Partial => phalcom_semantic::FormalPresentation::Partial,
        phalcom_semantic::FormalFactStatus::Unknown => phalcom_semantic::FormalPresentation::Unknown,
    })
}

fn canonical_formal_for_term(
    term: &phalcom_semantic::types::TypeTerm,
    presenter: &phalcom_semantic::TypePresenter<'_>,
) -> Option<phalcom_semantic::FormalPresentation> {
    Some(match term {
        phalcom_semantic::types::TypeTerm::Canonical(ty) => phalcom_semantic::FormalPresentation::Known(presenter.present_type(*ty)),
        phalcom_semantic::types::TypeTerm::SelfType(_) | phalcom_semantic::types::TypeTerm::Infer(_) => phalcom_semantic::FormalPresentation::Unknown,
    })
}

fn push_canonical_hint(
    hints: &mut Vec<InlayHint>,
    line_index: &LineIndex,
    offset: usize,
    formal: Option<phalcom_semantic::FormalPresentation>,
    advisory: Option<&phalcom_semantic::AdvisoryFact>,
    policy: HintPolicy,
    return_hint: bool,
) {
    let formal_text = formal.and_then(|presentation| match &presentation {
        phalcom_semantic::FormalPresentation::Known(_) | phalcom_semantic::FormalPresentation::Dynamic => Some(presentation.text()),
        _ => None,
    });
    let (label, tooltip) = if let Some(text) = formal_text {
        (format!(": {text}"), None)
    } else {
        let Some(fact) = advisory else { return };
        if matches!(fact.shape, phalcom_semantic::ValueShape::Unknown)
            || (policy == HintPolicy::Stable && matches!(fact.confidence, phalcom_semantic::AdvisoryConfidence::Heuristic))
        {
            return;
        }
        let rendered = phalcom_semantic::AdvisoryPresenter::present_shape(&fact.shape);
        (
            crate::presentation::inlay_type_label(&rendered, return_hint),
            Some(crate::presentation::advisory_tooltip(
                &rendered,
                if return_hint { "return value" } else { "runtime value" },
            )),
        )
    };
    hints.push(InlayHint {
        position: line_index.position(offset),
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
    });
}

// These inputs are intentionally explicit: each is independently sourced from the pinned request snapshot.
