from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if old in text:
        return text.replace(old, new, 1)
    if new in text:
        return text
    raise SystemExit(f"{label} shape changed")


# 1. Compiler-owned source binding metadata records explicit annotation truth.
path = Path("phalcom-semantic/src/source_index/scope.rs")
text = path.read_text()
text = replace_once(
    text,
    "    pub declaration_range: SourceRange,\n    pub mutable: bool,\n    pub redeclaration_of: Option<SourceSiteId>,\n",
    "    pub declaration_range: SourceRange,\n    pub has_explicit_annotation: bool,\n    pub mutable: bool,\n    pub redeclaration_of: Option<SourceSiteId>,\n",
    "source binding annotation metadata",
)
path.write_text(text)

# 2. Source index construction supplies that fact for lets and callable parameters.
path = Path("phalcom-semantic/src/source_index/builder.rs")
text = path.read_text()
old = '''    fn declare(&mut self, scope: SourceScopeId, name: impl Into<Box<str>>, kind: SourceBindingKind, range: SourceRange, mutable: bool) -> SourceSiteId {
        let name = name.into();
        let first = self.index.scopes.get(&scope).and_then(|scope| scope.bindings.get(&name)).cloned();
        let site = self.allocate_site(self.current_owner.clone(), range, SourceSiteKind::BindingDeclaration);
        let primary = first.clone().unwrap_or_else(|| site.clone());
        self.index.register_target(site.clone(), SemanticTargetId::Binding(primary.clone()));
        self.index.register_binding(SourceBindingInfo {
            declaration_site: site.clone(),
            scope,
            name: name.clone(),
            kind,
            declaration_range: range,
            mutable,
            redeclaration_of: first,
        });
        if let Some(scope_info) = self.index.scopes.get_mut(&scope)
            && !scope_info.bindings.contains_key(&name)
        {
            scope_info.bindings.insert(name, site.clone());
        }
        site
    }
'''
new = '''    fn declare(&mut self, scope: SourceScopeId, name: impl Into<Box<str>>, kind: SourceBindingKind, range: SourceRange, mutable: bool) -> SourceSiteId {
        self.declare_with_annotation(scope, name, kind, range, mutable, false)
    }

    fn declare_with_annotation(
        &mut self,
        scope: SourceScopeId,
        name: impl Into<Box<str>>,
        kind: SourceBindingKind,
        range: SourceRange,
        mutable: bool,
        has_explicit_annotation: bool,
    ) -> SourceSiteId {
        let name = name.into();
        let first = self.index.scopes.get(&scope).and_then(|scope| scope.bindings.get(&name)).cloned();
        let site = self.allocate_site(self.current_owner.clone(), range, SourceSiteKind::BindingDeclaration);
        let primary = first.clone().unwrap_or_else(|| site.clone());
        self.index.register_target(site.clone(), SemanticTargetId::Binding(primary.clone()));
        self.index.register_binding(SourceBindingInfo {
            declaration_site: site.clone(),
            scope,
            name: name.clone(),
            kind,
            declaration_range: range,
            has_explicit_annotation,
            mutable,
            redeclaration_of: first,
        });
        if let Some(scope_info) = self.index.scopes.get_mut(&scope)
            && !scope_info.bindings.contains_key(&name)
        {
            scope_info.bindings.insert(name, site.clone());
        }
        site
    }
'''
text = replace_once(text, old, new, "annotated binding declaration helper")
text = replace_once(
    text,
    "            let site = self.declare(scope, parameter.name.clone(), parameter_kind, parameter.name_range, true);\n",
    "            let site = self.declare_with_annotation(\n                scope,\n                parameter.name.clone(),\n                parameter_kind,\n                parameter.name_range,\n                true,\n                parameter.annotation.is_some(),\n            );\n",
    "callable parameter annotation truth",
)
text = replace_once(
    text,
    "        self.declare_pattern(scope, &binding.pattern, kind, binding.kind == BindingKind::Let);\n",
    "        self.declare_pattern_with_annotation(\n            scope,\n            &binding.pattern,\n            kind,\n            binding.kind == BindingKind::Let,\n            binding.annotation.is_some(),\n        );\n",
    "let annotation truth",
)
old = '''    fn declare_pattern(&mut self, scope: SourceScopeId, pattern: &Pattern, kind: SourceBindingKind, mutable: bool) {
        match pattern {
            Pattern::Name { name, range } => {
                self.declare(scope, name.clone(), kind, *range, mutable);
            }
            Pattern::Tuple { elements, .. } => {
                for element in elements {
                    self.declare_pattern(scope, element, SourceBindingKind::Destructure, mutable);
                }
            }
            Pattern::List { elements, rest, .. } => {
                for element in elements {
                    self.declare_pattern(scope, element, SourceBindingKind::Destructure, mutable);
                }
                if let Some(rest) = rest {
                    self.declare_pattern(scope, rest, SourceBindingKind::Destructure, mutable);
                }
            }
            Pattern::Variant { arguments, .. } => {
                for argument in arguments {
                    self.declare_pattern(scope, argument, SourceBindingKind::Destructure, mutable);
                }
            }
            Pattern::Record { entries, .. } => {
                for entry in entries {
                    self.declare_pattern(scope, &entry.pattern, SourceBindingKind::Destructure, mutable);
                }
            }
            Pattern::Map { entries, .. } => {
                for entry in entries {
                    self.declare_pattern(scope, &entry.pattern, SourceBindingKind::Destructure, mutable);
                }
            }
        }
    }
'''
new = '''    fn declare_pattern(&mut self, scope: SourceScopeId, pattern: &Pattern, kind: SourceBindingKind, mutable: bool) {
        self.declare_pattern_with_annotation(scope, pattern, kind, mutable, false);
    }

    fn declare_pattern_with_annotation(
        &mut self,
        scope: SourceScopeId,
        pattern: &Pattern,
        kind: SourceBindingKind,
        mutable: bool,
        has_explicit_annotation: bool,
    ) {
        match pattern {
            Pattern::Name { name, range } => {
                self.declare_with_annotation(scope, name.clone(), kind, *range, mutable, has_explicit_annotation);
            }
            Pattern::Tuple { elements, .. } => {
                for element in elements {
                    self.declare_pattern_with_annotation(scope, element, SourceBindingKind::Destructure, mutable, has_explicit_annotation);
                }
            }
            Pattern::List { elements, rest, .. } => {
                for element in elements {
                    self.declare_pattern_with_annotation(scope, element, SourceBindingKind::Destructure, mutable, has_explicit_annotation);
                }
                if let Some(rest) = rest {
                    self.declare_pattern_with_annotation(scope, rest, SourceBindingKind::Destructure, mutable, has_explicit_annotation);
                }
            }
            Pattern::Variant { arguments, .. } => {
                for argument in arguments {
                    self.declare_pattern_with_annotation(scope, argument, SourceBindingKind::Destructure, mutable, has_explicit_annotation);
                }
            }
            Pattern::Record { entries, .. } => {
                for entry in entries {
                    self.declare_pattern_with_annotation(scope, &entry.pattern, SourceBindingKind::Destructure, mutable, has_explicit_annotation);
                }
            }
            Pattern::Map { entries, .. } => {
                for entry in entries {
                    self.declare_pattern_with_annotation(scope, &entry.pattern, SourceBindingKind::Destructure, mutable, has_explicit_annotation);
                }
            }
        }
    }
'''
text = replace_once(text, old, new, "pattern annotation propagation")
path.write_text(text)

# 3. Reuse the canonical type presenter for declaration facts from editor queries.
path = Path("phalcom-semantic/src/presentation.rs")
text = path.read_text()
text = replace_once(
    text,
    "fn present_declared_type(fact: &crate::declaration_type::DeclaredTypeFact, presenter: &TypePresenter<'_>) -> FormalPresentation {\n",
    "pub(crate) fn present_declared_type(fact: &crate::declaration_type::DeclaredTypeFact, presenter: &TypePresenter<'_>) -> FormalPresentation {\n",
    "shared declared type presentation",
)
path.write_text(text)

# Exact source-site lookup is internal compiler plumbing used by editor projection.
path = Path("phalcom-semantic/src/snapshot.rs")
text = path.read_text()
text = replace_once(
    text,
    "    fn formal_fact_for_site(&self, site: &SourceSiteId) -> Option<FormalFactRef> {\n",
    "    pub(crate) fn formal_fact_for_site(&self, site: &SourceSiteId) -> Option<FormalFactRef> {\n",
    "formal fact source-site visibility",
)
path.write_text(text)

# 4. EditorSemanticQuery owns type-hint eligibility and formal/advisory separation.
path = Path("phalcom-semantic/src/editor.rs")
text = path.read_text()
text = replace_once(
    text,
    "use crate::advisory::ValueShape;\nuse crate::advisory::advisory_shape_from_formal;\n",
    "use crate::advisory::{AdvisoryFact, ValueShape, advisory_shape_from_formal};\n",
    "editor advisory imports",
)
text = replace_once(
    text,
    "use crate::snapshot::SemanticSnapshot;\nuse crate::source_index::{OccurrenceHint, OccurrenceRole, SourceBindingInfo};\n",
    "use crate::presentation::{FormalFactStatus, FormalPresentation, TypePresenter, present_declared_type};\nuse crate::snapshot::SemanticSnapshot;\nuse crate::source_index::{OccurrenceHint, OccurrenceRole, SourceBindingInfo, SourceBindingKind};\n",
    "editor hint imports",
)
anchor = '''/// Read-only editor query facade over one immutable semantic snapshot.
#[derive(Clone, Copy, Debug)]
pub struct EditorSemanticQuery<'a> {
'''
insert = '''/// Compiler-owned category for a source type hint.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EditorTypeHintKind {
    Binding,
    Parameter,
    Field,
    Return,
}

/// Protocol-neutral type hint projection.
///
/// Formal and advisory channels remain separate. An advisory shape may explain
/// an otherwise unknown formal position, but it never replaces formal truth.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorTypeHint {
    pub kind: EditorTypeHintKind,
    pub source_range: SourceRange,
    pub insertion_offset: usize,
    pub target: Option<SemanticTargetId>,
    pub formal: Option<FormalPresentation>,
    pub advisory: Option<AdvisoryFact>,
}

''' + anchor
text = replace_once(text, anchor, insert, "editor type hint models")
anchor = '''    /// Returns compiler-owned lexical bindings visible at a source position.
    pub fn visible_symbols_at(&self, module: &ModuleId, offset: usize) -> Vec<VisibleSymbol> {
'''
method = '''    /// Returns compiler-owned type hints within one source range.
    ///
    /// Annotation truth, canonical declaration/binding identity, and the formal
    /// versus advisory distinction are decided here. Protocol adapters may
    /// suppress hints for presentation preferences but must not reconstruct
    /// semantic eligibility from the AST.
    pub fn type_hints(&self, module: &ModuleId, visible: SourceRange) -> Vec<EditorTypeHint> {
        let Some(source) = self.snapshot.source_index.module(module) else {
            return Vec::new();
        };
        let presenter = TypePresenter::new(&self.snapshot.store);
        let mut hints = Vec::new();

        for binding in source.structure.bindings.values() {
            if matches!(
                binding.kind,
                SourceBindingKind::Import | SourceBindingKind::MethodParameter | SourceBindingKind::SetterParameter | SourceBindingKind::IndexParameter
            ) || binding.has_explicit_annotation
                || !ranges_overlap(binding.declaration_range, visible)
            {
                continue;
            }
            let formal = self.formal_binding_presentation(&binding.declaration_site, &presenter);
            let advisory = self.snapshot.advisory_fact(&binding.declaration_site).cloned();
            if !type_hint_has_usable_evidence(formal.as_ref(), advisory.as_ref()) {
                continue;
            }
            hints.push(EditorTypeHint {
                kind: EditorTypeHintKind::Binding,
                source_range: binding.declaration_range,
                insertion_offset: binding.declaration_range.end,
                target: source.structure.target_for(&binding.declaration_site).cloned(),
                formal,
                advisory,
            });
        }

        for field in source.structure.field_sources.values() {
            if field.has_explicit_annotation || !ranges_overlap(field.name_range, visible) {
                continue;
            }
            let formal = self
                .snapshot
                .field_signatures
                .get(&field.id)
                .map(|signature| present_declared_type(&signature.declared_type, &presenter));
            let advisory = self.snapshot.advisory.field(&field.id).cloned();
            if !type_hint_has_usable_evidence(formal.as_ref(), advisory.as_ref()) {
                continue;
            }
            hints.push(EditorTypeHint {
                kind: EditorTypeHintKind::Field,
                source_range: field.name_range,
                insertion_offset: field.name_range.end,
                target: Some(SemanticTargetId::Field(field.id.clone())),
                formal,
                advisory,
            });
        }

        for callable in source.structure.callable_sources.values() {
            let Some(signature) = self.snapshot.callable_signatures.get(&callable.id) else {
                continue;
            };
            let advisory = self.snapshot.advisory_callable(&callable.id);

            for parameter in signature.parameters.iter() {
                let Some(site) = callable.parameter_sites.get(&parameter.id) else {
                    continue;
                };
                let Some(binding) = source.structure.bindings.get(site) else {
                    continue;
                };
                if binding.has_explicit_annotation || !ranges_overlap(binding.declaration_range, visible) {
                    continue;
                }
                let formal = Some(present_declared_type(&parameter.declared_type, &presenter));
                let advisory = advisory
                    .and_then(|summary| summary.parameters.iter().find(|(slot, _)| slot == &parameter.id).map(|(_, fact)| fact))
                    .cloned();
                if !type_hint_has_usable_evidence(formal.as_ref(), advisory.as_ref()) {
                    continue;
                }
                hints.push(EditorTypeHint {
                    kind: EditorTypeHintKind::Parameter,
                    source_range: binding.declaration_range,
                    insertion_offset: binding.declaration_range.end,
                    target: source.structure.target_for(site).cloned(),
                    formal,
                    advisory,
                });
            }

            if callable.has_explicit_return_annotation {
                continue;
            }
            let insertion_offset = source
                .structure
                .callable_body_ranges
                .get(&callable.id)
                .map_or(callable.declaration_range.end, |range| range.end);
            if insertion_offset < visible.start || insertion_offset > visible.end {
                continue;
            }
            let formal = Some(
                signature
                    .inferred_return
                    .as_ref()
                    .filter(|knowledge| knowledge.is_known() || knowledge.is_dynamic())
                    .map(|knowledge| presenter.present_knowledge(knowledge))
                    .unwrap_or_else(|| present_declared_type(&signature.declared_return, &presenter)),
            );
            let advisory = advisory.map(|summary| summary.return_fact.clone());
            if !type_hint_has_usable_evidence(formal.as_ref(), advisory.as_ref()) {
                continue;
            }
            hints.push(EditorTypeHint {
                kind: EditorTypeHintKind::Return,
                source_range: callable.declaration_range,
                insertion_offset,
                target: Some(SemanticTargetId::Callable(callable.id.clone())),
                formal,
                advisory,
            });
        }

        hints.sort_by_key(|hint| (hint.insertion_offset, hint.kind));
        hints
    }

    fn formal_binding_presentation(&self, site: &SourceSiteId, presenter: &TypePresenter<'_>) -> Option<FormalPresentation> {
        let fact_ref = self.snapshot.formal_fact_for_site(site)?;
        let fact_site = self.snapshot.formal_fact(&fact_ref)?;
        let knowledge = match &fact_ref {
            crate::presentation::FormalFactRef::Binding { callable, binding } => self.snapshot.formal_binding(callable, *binding)?.current.clone(),
            _ => return None,
        };
        Some(match fact_site.status {
            FormalFactStatus::Ready => presenter.present_knowledge(&knowledge),
            FormalFactStatus::Unknown => FormalPresentation::Unknown,
            FormalFactStatus::Dynamic => FormalPresentation::Dynamic,
            FormalFactStatus::Invalid | FormalFactStatus::InvalidMultiple => FormalPresentation::Invalid,
            FormalFactStatus::Blocked => FormalPresentation::Blocked,
            FormalFactStatus::Cancelled => FormalPresentation::Cancelled,
            FormalFactStatus::BudgetExceeded => FormalPresentation::BudgetExceeded,
            FormalFactStatus::InternalFailure => FormalPresentation::InternalFailure,
            FormalFactStatus::Partial => FormalPresentation::Partial,
        })
    }

''' + anchor
text = replace_once(text, anchor, method, "editor type hint query")
anchor = '''fn collect_receiver_alternatives(shape: &ValueShape, alternatives: &mut Vec<ReceiverAlternative>) {
'''
helpers = '''fn ranges_overlap(left: SourceRange, right: SourceRange) -> bool {
    left.start <= right.end && right.start <= left.end
}

fn type_hint_has_usable_evidence(formal: Option<&FormalPresentation>, advisory: Option<&AdvisoryFact>) -> bool {
    if matches!(formal, Some(FormalPresentation::Known(_) | FormalPresentation::Dynamic)) {
        return true;
    }
    if formal.is_some() && !matches!(formal, Some(FormalPresentation::Unknown)) {
        return false;
    }
    advisory.is_some_and(|fact| !matches!(fact.shape, ValueShape::Unknown))
}

''' + anchor
text = replace_once(text, anchor, helpers, "editor type hint helpers")
path.write_text(text)

# 5. Export the editor-neutral result types.
path = Path("phalcom-semantic/src/lib.rs")
text = path.read_text()
text = replace_once(
    text,
    "    AccessContext, EditorMember, EditorMemberTarget, EditorSemanticQuery, NativeCallablePresentation, PartialCallPattern, ReceiverAlternative, ReceiverMode,\n    ResolvedReceiver, VisibleSymbol,\n",
    "    AccessContext, EditorMember, EditorMemberTarget, EditorSemanticQuery, EditorTypeHint, EditorTypeHintKind, NativeCallablePresentation, PartialCallPattern,\n    ReceiverAlternative, ReceiverMode, ResolvedReceiver, VisibleSymbol,\n",
    "editor type hint exports",
)
path.write_text(text)

# 6. LSP becomes a pure protocol/presentation adapter over EditorSemanticQuery.
path = Path("phalcom-lsp/src/inlay_hints.rs")
path.write_text(r'''//! Standard LSP type-hint rendering over compiler-owned editor semantics.

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
                && !matches!(hint.formal, Some(FormalPresentation::Known(_) | FormalPresentation::Dynamic))
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

fn render_hint(line_index: &LineIndex, hint: EditorTypeHint, policy: HintPolicy) -> Option<InlayHint> {
    let return_hint = hint.kind == EditorTypeHintKind::Return;
    let formal_text = hint.formal.as_ref().and_then(|presentation| match presentation {
        FormalPresentation::Known(_) | FormalPresentation::Dynamic => Some(presentation.text()),
        _ => None,
    });

    let (label, tooltip) = if let Some(text) = formal_text {
        (crate::presentation::inlay_type_label(&text, return_hint), None)
    } else {
        if hint.formal.is_some() && !matches!(hint.formal, Some(FormalPresentation::Unknown)) {
            return None;
        }
        let fact = hint.advisory.as_ref()?;
        if matches!(fact.shape, ValueShape::Unknown)
            || (policy == HintPolicy::Stable && matches!(fact.confidence, AdvisoryConfidence::Heuristic))
        {
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
''')

# Architecture assertions so a shape change fails before commit.
scope = Path("phalcom-semantic/src/source_index/scope.rs").read_text()
editor = Path("phalcom-semantic/src/editor.rs").read_text()
lsp = Path("phalcom-lsp/src/inlay_hints.rs").read_text()
assert "pub has_explicit_annotation: bool" in scope
assert "pub fn type_hints(" in editor
assert "ExplicitAnnotationIndex" not in lsp
assert ".type_hints(" in lsp
