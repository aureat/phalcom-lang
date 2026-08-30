//! Compiler-owned semantic source occurrences and target reverse index.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::identity::{SemanticTargetId, SourceOwner, SourceSiteId, SourceSiteLocalId};
use crate::source_index::builder::SourceIndexContext;
use crate::source_index::interval::{RangeEntry, RangeIndex};
use crate::source_index::scope::{SourceNameResolution, SourceReceiverKind, SourceScopeIndex};
use crate::source_index::site::{SourceSite, SourceSiteKind};
use phalcom_ast::ast::{BlockExpr, Expr, PackItem, Program, Statement};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorSlot};

/// Broad syntax category for one source occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OccurrenceKind {
    Binding,
    Parameter,
    Declaration,
    Module,
    Member,
    Field,
    Operator,
}

/// Role of an occurrence in source semantics.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OccurrenceRole {
    Declaration,
    Read,
    Write,
    Call,
    Reference,
}

/// Non-authoritative information retained when exact target resolution is not
/// available. Hints never become [`SemanticTargetId`] values.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OccurrenceHint {
    MemberSelector(Selector),
    Operator(Box<str>),
    Name(Box<str>),
}

/// Syntax-owned occurrence record.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticOccurrence {
    pub site: SourceSiteId,
    pub range: SourceRange,
    pub kind: OccurrenceKind,
    pub role: OccurrenceRole,
    pub owner: SourceOwner,
    pub hint: Option<OccurrenceHint>,
}

/// Occurrence plus optional exact canonical target attachment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceView<'a> {
    pub occurrence: &'a SemanticOccurrence,
    pub target: Option<&'a SemanticTargetId>,
}

/// Immutable occurrence index with bounded interval and target reverse lookup.
#[derive(Clone, Debug)]
pub struct OccurrenceIndex {
    occurrences: Arc<[SemanticOccurrence]>,
    intervals: RangeIndex<usize>,
    exact_targets: BTreeMap<SourceSiteId, SemanticTargetId>,
    target_occurrences: BTreeMap<SemanticTargetId, Arc<[SourceSiteId]>>,
}

impl OccurrenceIndex {
    /// Builds deterministic source-order occurrence and target indexes.
    pub fn new(mut occurrences: Vec<SemanticOccurrence>, exact_targets: BTreeMap<SourceSiteId, SemanticTargetId>) -> Self {
        occurrences.sort_by_key(|occurrence| {
            (
                occurrence.range.start,
                occurrence.range.len(),
                occurrence_kind_priority(occurrence.kind),
                occurrence_role_priority(occurrence.role),
                occurrence.site.clone(),
            )
        });
        let intervals = RangeIndex::new(
            occurrences
                .iter()
                .enumerate()
                .map(|(index, occurrence)| RangeEntry::new(occurrence.range, index, occurrence_kind_priority(occurrence.kind))),
        );
        let mut reverse = BTreeMap::<SemanticTargetId, Vec<SourceSiteId>>::new();
        for occurrence in &occurrences {
            if let Some(target) = exact_targets.get(&occurrence.site) {
                reverse.entry(target.clone()).or_default().push(occurrence.site.clone());
            }
        }
        let target_occurrences = reverse.into_iter().map(|(target, sites)| (target, Arc::from(sites))).collect();
        Self {
            occurrences: Arc::from(occurrences),
            intervals,
            exact_targets,
            target_occurrences,
        }
    }

    /// Number of occurrences in this source index.
    pub fn len(&self) -> usize {
        self.occurrences.len()
    }

    /// Whether this source index has no occurrences.
    pub fn is_empty(&self) -> bool {
        self.occurrences.is_empty()
    }

    /// Returns occurrences in deterministic source order.
    pub fn all(&self) -> &[SemanticOccurrence] {
        &self.occurrences
    }

    /// Builds declaration occurrences from compiler-owned source sites.
    pub fn from_scope_index(scopes: &crate::source_index::scope::SourceScopeIndex) -> Self {
        let occurrences = scopes
            .sites
            .values()
            .filter_map(|site| {
                let (kind, role) = match site.kind {
                    crate::source_index::site::SourceSiteKind::BindingDeclaration => (OccurrenceKind::Binding, OccurrenceRole::Declaration),
                    crate::source_index::site::SourceSiteKind::Declaration(_) => (OccurrenceKind::Declaration, OccurrenceRole::Declaration),
                    crate::source_index::site::SourceSiteKind::Callable(_) => (OccurrenceKind::Member, OccurrenceRole::Declaration),
                    crate::source_index::site::SourceSiteKind::Field(_) => (OccurrenceKind::Field, OccurrenceRole::Declaration),
                    crate::source_index::site::SourceSiteKind::Module
                    | crate::source_index::site::SourceSiteKind::Expression
                    | crate::source_index::site::SourceSiteKind::Occurrence => return None,
                };
                Some(SemanticOccurrence {
                    site: site.id.clone(),
                    range: site.range,
                    kind,
                    role,
                    owner: site.id.owner.clone(),
                    hint: None,
                })
            })
            .collect();
        Self::new(occurrences, scopes.targets.clone())
    }

    /// Builds declaration plus AST-wide read/write/call occurrences. Exact
    /// targets are attached only when compiler lexical resolution proves them;
    /// unresolved names remain hints and never become semantic identities.
    pub fn from_program(scopes: &mut SourceScopeIndex, program: &Program) -> Self {
        Self::from_program_with_context(scopes, program, None)
    }

    /// Builds AST occurrences while attaching compiler-owned targets for
    /// qualified module members. The context is optional for standalone source
    /// index tests that only exercise lexical occurrences.
    pub fn from_program_with_context(scopes: &mut SourceScopeIndex, program: &Program, context: Option<&SourceIndexContext>) -> Self {
        let mut result = Self::from_scope_index(scopes);
        let next_site = scopes.sites.values().fold(BTreeMap::<SourceOwner, u32>::new(), |mut next, site| {
            next.entry(site.id.owner.clone())
                .and_modify(|value| *value = (*value).max(site.id.local.0.saturating_add(1)))
                .or_insert(site.id.local.0.saturating_add(1));
            next
        });
        let targets = scopes.targets.clone();
        let mut visitor = OccurrenceBuilder {
            scopes,
            next_site,
            occurrences: result.all().to_vec(),
            targets,
            context,
        };
        for statement in &program.statements {
            visitor.statement(statement);
        }
        if let Some(context) = context {
            let module = visitor.scopes.module.clone();
            let references = context
                .type_reference_targets
                .iter()
                .filter(|((owner, _), _)| owner == &module)
                .map(|((_, range), declaration)| (*range, declaration.clone()))
                .collect::<Vec<_>>();
            for (range, declaration) in references {
                visitor.record_targeted(
                    range,
                    OccurrenceKind::Declaration,
                    OccurrenceRole::Reference,
                    None,
                    Some(SemanticTargetId::Declaration(declaration)),
                );
            }
        }
        result = Self::new(visitor.occurrences, visitor.targets);
        result
    }

    /// Selects shortest containing occurrence with deterministic tie breaks.
    pub fn occurrence_at(&self, offset: usize) -> Option<OccurrenceView<'_>> {
        let index = self.intervals.index_at(offset)?;
        let occurrence = &self.occurrences[index];
        Some(OccurrenceView {
            occurrence,
            target: self.exact_targets.get(&occurrence.site),
        })
    }

    /// Returns all exact occurrence sites for one canonical target.
    pub fn occurrences_for_target(&self, target: &SemanticTargetId) -> Option<&[SourceSiteId]> {
        self.target_occurrences.get(target).map(AsRef::as_ref)
    }

    /// Returns canonical target attached to one occurrence site.
    pub fn target_for(&self, site: &SourceSiteId) -> Option<&SemanticTargetId> {
        self.exact_targets.get(site)
    }

    /// Returns exact occurrence metadata for one indexed source site.
    pub fn occurrence_for_site(&self, site: &SourceSiteId) -> Option<&SemanticOccurrence> {
        self.occurrences.iter().find(|occurrence| &occurrence.site == site)
    }
}

struct OccurrenceBuilder<'a> {
    scopes: &'a mut SourceScopeIndex,
    next_site: BTreeMap<SourceOwner, u32>,
    occurrences: Vec<SemanticOccurrence>,
    targets: BTreeMap<SourceSiteId, SemanticTargetId>,
    context: Option<&'a SourceIndexContext>,
}

impl OccurrenceBuilder<'_> {
    fn statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Class(class) => {
                for member in &class.members {
                    match member {
                        phalcom_ast::ast::ClassMember::Method(method) => {
                            if let Some(body) = method.body.statements() {
                                self.statements(body);
                            }
                        }
                        phalcom_ast::ast::ClassMember::Getter(getter) => {
                            if let Some(body) = getter.body.statements() {
                                self.statements(body);
                            }
                        }
                        phalcom_ast::ast::ClassMember::Setter(setter) => {
                            if let Some(body) = setter.body.statements() {
                                self.statements(body);
                            }
                        }
                        phalcom_ast::ast::ClassMember::Index(index) => self.statements(&index.body),
                        phalcom_ast::ast::ClassMember::Field(field) => {
                            if let Some(default) = &field.default {
                                self.expr(default, OccurrenceRole::Read);
                            }
                        }
                        phalcom_ast::ast::ClassMember::Variant(_) => {}
                    }
                }
            }
            Statement::Let(binding) => {
                if let Some(value) = &binding.value {
                    self.expr(value, OccurrenceRole::Read);
                }
            }
            Statement::Return(return_statement) => {
                if let Some(value) = &return_statement.value {
                    self.expr(value, OccurrenceRole::Read);
                }
            }
            Statement::Expr { expr, .. } | Statement::Throw { expr, .. } => self.expr(expr, OccurrenceRole::Read),
            Statement::For(for_statement) => {
                for lane in &for_statement.lanes {
                    self.expr(&lane.iter, OccurrenceRole::Read);
                }
                self.statements(&for_statement.body);
            }
            Statement::Export(_) | Statement::Break { .. } | Statement::Continue { .. } | Statement::TypeAlias(_) => {}
            Statement::Enum(enum_def) => {
                for member in &enum_def.members {
                    match member {
                        phalcom_ast::ast::EnumMember::Variant(v) => {
                            if let Some(body) = &v.body {
                                for b_member in &body.members {
                                    match b_member {
                                        phalcom_ast::ast::EnumBehaviorMember::Method(m) => {
                                            if let Some(body) = m.body.statements() {
                                                self.statements(body);
                                            }
                                        }
                                        phalcom_ast::ast::EnumBehaviorMember::Getter(g) => {
                                            if let Some(body) = g.body.statements() {
                                                self.statements(body);
                                            }
                                        }
                                        phalcom_ast::ast::EnumBehaviorMember::Setter(s) => {
                                            if let Some(body) = s.body.statements() {
                                                self.statements(body);
                                            }
                                        }
                                        phalcom_ast::ast::EnumBehaviorMember::Index(i) => {
                                            self.statements(&i.body);
                                        }
                                    }
                                }
                            }
                        }
                        phalcom_ast::ast::EnumMember::Behavior(b) => {
                            match b {
                                phalcom_ast::ast::EnumBehaviorMember::Method(m) => {
                                    if let Some(body) = m.body.statements() {
                                        self.statements(body);
                                    }
                                }
                                phalcom_ast::ast::EnumBehaviorMember::Getter(g) => {
                                    if let Some(body) = g.body.statements() {
                                        self.statements(body);
                                    }
                                }
                                phalcom_ast::ast::EnumBehaviorMember::Setter(s) => {
                                    if let Some(body) = s.body.statements() {
                                        self.statements(body);
                                    }
                                }
                                phalcom_ast::ast::EnumBehaviorMember::Index(i) => {
                                    self.statements(&i.body);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn statements(&mut self, statements: &[Statement]) {
        for statement in statements {
            self.statement(statement);
        }
    }

    fn expr(&mut self, expr: &Expr, role: OccurrenceRole) {
        match expr {
            Expr::Var { value, range } => {
                self.record(
                    *range,
                    OccurrenceKind::Binding,
                    role,
                    Some(OccurrenceHint::Name(value.clone().into())),
                    Some(value),
                );
            }
            Expr::Field { value, range, .. } => {
                self.record(*range, OccurrenceKind::Field, role, Some(OccurrenceHint::Name(value.clone().into())), None);
            }
            Expr::SelfVar { range } => {
                let site = self.record(*range, OccurrenceKind::Binding, role, None, None);
                self.scopes.register_receiver_kind(site, SourceReceiverKind::SelfValue);
            }
            Expr::SuperVar { range } => {
                let site = self.record(*range, OccurrenceKind::Binding, role, None, None);
                self.scopes.register_receiver_kind(site, SourceReceiverKind::SuperValue);
            }
            Expr::Assignment(assignment) => {
                self.expr(&assignment.name, OccurrenceRole::Write);
                self.expr(&assignment.value, OccurrenceRole::Read);
            }
            Expr::Range(range) => {
                if let Some(lower) = &range.lower {
                    self.expr(lower, OccurrenceRole::Read);
                }
                if let Some(upper) = &range.upper {
                    self.expr(upper, OccurrenceRole::Read);
                }
            }
            Expr::Unary(unary) => self.expr(&unary.expr, role),
            Expr::Binary(binary) => {
                self.expr(&binary.left, OccurrenceRole::Read);
                self.expr(&binary.right, OccurrenceRole::Read);
            }
            Expr::ComparisonChain(chain) => {
                for operand in &chain.operands {
                    self.expr(operand, OccurrenceRole::Read);
                }
            }
            Expr::IfLet(if_let) => {
                self.expr(&if_let.value, OccurrenceRole::Read);
                self.block(&if_let.then_body);
                if let Some(else_body) = &if_let.else_body {
                    self.block(else_body);
                }
            }
            Expr::WhileLet(while_let) => {
                self.expr(&while_let.value, OccurrenceRole::Read);
                self.statements(&while_let.body);
            }
            Expr::UnqualifiedCall(call) => {
                if let Some(range) = call.name_range {
                    self.record_targeted(
                        range,
                        OccurrenceKind::Member,
                        OccurrenceRole::Call,
                        Some(OccurrenceHint::Name(call.name.clone().into())),
                        None,
                    );
                }
                self.pack(&call.args);
            }
            Expr::MethodCall(call) => {
                if let Some(range) = call.method_range {
                    let target = self.member_target(&call.object, &call.method, &call.args);
                    self.record_targeted(
                        range,
                        OccurrenceKind::Member,
                        OccurrenceRole::Call,
                        Some(OccurrenceHint::Name(call.method.clone().into())),
                        target,
                    );
                }
                self.expr(&call.object, OccurrenceRole::Read);
                self.pack(&call.args);
            }
            Expr::GetProperty(property) => {
                if let Some(range) = property.property_range {
                    let target = self.member_target(&property.object, &property.property, &[]);
                    self.record_targeted(
                        range,
                        OccurrenceKind::Field,
                        OccurrenceRole::Read,
                        Some(OccurrenceHint::Name(property.property.clone().into())),
                        target,
                    );
                }
                self.expr(&property.object, OccurrenceRole::Read);
            }
            Expr::SetProperty(property) => {
                if let Some(range) = property.property_range {
                    self.record(
                        range,
                        OccurrenceKind::Field,
                        OccurrenceRole::Write,
                        Some(OccurrenceHint::Name(property.property.clone().into())),
                        None,
                    );
                }
                self.expr(&property.object, OccurrenceRole::Read);
                self.expr(&property.value, OccurrenceRole::Read);
            }
            Expr::Index(index) => {
                self.expr(&index.object, OccurrenceRole::Read);
                self.pack(&index.args);
            }
            Expr::SetIndex(index) => {
                self.expr(&index.object, OccurrenceRole::Read);
                self.pack(&index.args);
                self.expr(&index.value, OccurrenceRole::Read);
            }
            Expr::Block(block) => self.block(block),
            Expr::AssociatedLookup(lookup) => self.expr(&lookup.receiver, OccurrenceRole::Reference),
            Expr::AssociatedInvoke(invoke) => {
                self.expr(&invoke.receiver, OccurrenceRole::Reference);
                self.pack(&invoke.args);
            }
            Expr::TupleLiteral(tuple) => {
                for entry in &tuple.entries {
                    match entry {
                        phalcom_ast::ast::TupleLiteralEntry::Positional { expr, .. } | phalcom_ast::ast::TupleLiteralEntry::Expand { expr, .. } => {
                            self.expr(expr, OccurrenceRole::Read)
                        }
                        phalcom_ast::ast::TupleLiteralEntry::Labeled { value, .. } => self.expr(value, OccurrenceRole::Read),
                    }
                }
            }
            Expr::RecordLiteral(record) => {
                for entry in &record.entries {
                    match entry {
                        phalcom_ast::ast::RecordLiteralEntry::Field(field) => self.expr(&field.value, OccurrenceRole::Read),
                        phalcom_ast::ast::RecordLiteralEntry::Expansion { expr, .. } => self.expr(expr, OccurrenceRole::Read),
                    }
                }
            }
            Expr::MapLiteral(map) => {
                for entry in &map.entries {
                    match entry {
                        phalcom_ast::ast::MapLiteralEntry::Association { key, value, .. } => {
                            if let phalcom_ast::ast::MapLiteralKey::Computed { expr, .. } = key {
                                self.expr(expr, OccurrenceRole::Read);
                            }
                            self.expr(value, OccurrenceRole::Read);
                        }
                        phalcom_ast::ast::MapLiteralEntry::Expansion { expr, .. } => self.expr(expr, OccurrenceRole::Read),
                    }
                }
            }
            Expr::SetLiteral(set) => {
                for entry in &set.entries {
                    match entry {
                        phalcom_ast::ast::SetLiteralEntry::Element { expr, .. } | phalcom_ast::ast::SetLiteralEntry::Expansion { expr, .. } => {
                            self.expr(expr, OccurrenceRole::Read)
                        }
                    }
                }
            }
            Expr::ListLiteral(list) => {
                for element in &list.elements {
                    match element {
                        phalcom_ast::ast::ListLiteralElement::Element { expr, .. } | phalcom_ast::ast::ListLiteralElement::Expansion { expr, .. } => {
                            self.expr(expr, OccurrenceRole::Read)
                        }
                    }
                }
            }
            Expr::Membership(membership) => {
                self.expr(&membership.left, OccurrenceRole::Read);
                self.expr(&membership.right, OccurrenceRole::Read);
            }
            Expr::IsMembership(membership) => {
                self.expr(&membership.left, OccurrenceRole::Read);
                self.expr(&membership.candidates, OccurrenceRole::Read);
            }
            Expr::Int { .. }
            | Expr::Float { .. }
            | Expr::String { .. }
            | Expr::Boolean { .. }
            | Expr::Ellipsis { .. }
            | Expr::ImplementationSelector { .. }
            | Expr::Symbol { .. }
            | Expr::TypeForm(_) => {}
        }
    }

    fn block(&mut self, block: &BlockExpr) {
        self.statements(&block.body);
    }

    fn pack(&mut self, items: &[PackItem]) {
        for item in items {
            match item {
                PackItem::Positional { expr, .. } | PackItem::Expand { expr, .. } => self.expr(expr, OccurrenceRole::Read),
                PackItem::Labeled { value, .. } => self.expr(value, OccurrenceRole::Read),
            }
        }
    }

    fn record(
        &mut self,
        range: phalcom_common::range::SourceRange,
        kind: OccurrenceKind,
        role: OccurrenceRole,
        hint: Option<OccurrenceHint>,
        name: Option<&str>,
    ) -> SourceSiteId {
        let scope = self.scopes.scope_at(range.start);
        let target = name.and_then(|name| match self.scopes.resolve_name(scope, name, range.start) {
            SourceNameResolution::Binding(binding) => Some(SemanticTargetId::Binding(binding)),
            SourceNameResolution::Target(target) => Some(target),
            SourceNameResolution::ImplicitSelf | SourceNameResolution::Unresolved => None,
        });
        self.record_targeted(range, kind, role, hint, target)
    }

    fn record_targeted(
        &mut self,
        range: phalcom_common::range::SourceRange,
        kind: OccurrenceKind,
        role: OccurrenceRole,
        hint: Option<OccurrenceHint>,
        target: Option<SemanticTargetId>,
    ) -> SourceSiteId {
        let scope = self.scopes.scope_at(range.start);
        let owner = owner_for_scope(self.scopes, scope, range.start);
        let next = self.next_site.entry(owner.clone()).or_default();
        let site = SourceSite::new(owner.clone(), SourceSiteLocalId(*next), range, SourceSiteKind::Occurrence);
        *next = next.saturating_add(1);
        self.scopes.register_site(site.clone());
        if let Some(target) = &target {
            self.targets.insert(site.id.clone(), target.clone());
        }
        let site_id = site.id.clone();
        self.occurrences.push(SemanticOccurrence {
            site: site_id.clone(),
            range,
            kind,
            role,
            owner,
            hint,
        });
        site_id
    }

    fn member_target(&self, object: &Expr, member: &str, args: &[PackItem]) -> Option<SemanticTargetId> {
        let context = self.context?;
        let receiver = self.expression_target(object)?;
        match receiver {
            SemanticTargetId::Module(module) => context.targets.get(&(module, member.to_owned())).cloned(),
            SemanticTargetId::Declaration(declaration) => {
                let slots = args
                    .iter()
                    .map(|arg| match arg {
                        PackItem::Positional { .. } | PackItem::Expand { .. } => Some(SelectorSlot::Positional),
                        PackItem::Labeled {
                            label: phalcom_ast::ast::PackLabel::Static { text, .. },
                            ..
                        } => Some(SelectorSlot::Label(text.clone())),
                        PackItem::Labeled {
                            label: phalcom_ast::ast::PackLabel::Computed { .. },
                            ..
                        } => None,
                    })
                    .collect::<Option<Vec<_>>>()?;
                let selector = Selector::method(member, slots).ok()?;
                context.callable_targets.get(&(declaration, selector)).cloned().map(SemanticTargetId::Callable)
            }
            _ => None,
        }
    }

    fn expression_target(&self, expr: &Expr) -> Option<SemanticTargetId> {
        match expr {
            Expr::Var { value, range } => match self.scopes.resolve_name(self.scopes.scope_at(range.start), value, range.start) {
                SourceNameResolution::Binding(binding) => self.scopes.target_for(&binding).cloned().or(Some(SemanticTargetId::Binding(binding))),
                SourceNameResolution::Target(target) => Some(target),
                SourceNameResolution::ImplicitSelf | SourceNameResolution::Unresolved => None,
            },
            Expr::GetProperty(property) => self.member_target(&property.object, &property.property, &[]),
            _ => None,
        }
    }
}

fn owner_for_scope(scopes: &SourceScopeIndex, mut scope: crate::source_index::SourceScopeId, offset: usize) -> SourceOwner {
    if let Some(owner) = scopes
        .callable_body_ranges
        .iter()
        .filter(|(_, range)| range.contains(offset))
        .min_by_key(|(_, range)| range.len())
        .map(|(callable, _)| SourceOwner::Callable(callable.clone()))
    {
        return owner;
    }
    loop {
        if let Some(owner) = scopes
            .bindings
            .values()
            .find(|binding| binding.scope == scope && matches!(binding.declaration_site.owner, SourceOwner::Callable(_)))
            .map(|binding| binding.declaration_site.owner.clone())
        {
            return owner;
        }
        let Some(parent) = scopes.scopes.get(&scope).and_then(|scope| scope.parent) else {
            break;
        };
        scope = parent;
    }
    SourceOwner::Module(scopes.module.clone())
}

fn occurrence_kind_priority(kind: OccurrenceKind) -> u8 {
    match kind {
        OccurrenceKind::Binding => 0,
        OccurrenceKind::Parameter => 1,
        OccurrenceKind::Declaration => 2,
        OccurrenceKind::Module => 3,
        OccurrenceKind::Member => 4,
        OccurrenceKind::Field => 5,
        OccurrenceKind::Operator => 6,
    }
}

fn occurrence_role_priority(role: OccurrenceRole) -> u8 {
    match role {
        OccurrenceRole::Declaration => 0,
        OccurrenceRole::Write => 1,
        OccurrenceRole::Call => 2,
        OccurrenceRole::Reference => 3,
        OccurrenceRole::Read => 4,
    }
}
