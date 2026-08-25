//! AST-to-source-scope construction under compiler ownership.

use std::collections::BTreeMap;

use crate::identity::{CallableId, DeclarationId, DispatchSide, FieldId, ModuleId, SemanticTargetId, SourceOwner, SourceSiteId, SourceSiteLocalId};
use crate::source_index::scope::{SourceBindingInfo, SourceBindingKind, SourceScopeId, SourceScopeIndex};
use crate::source_index::site::{SourceSite, SourceSiteKind};
use phalcom_ast::ast::{BindingKind, BlockExpr, ClassDef, ClassMember, Expr, ForStatement, LetBinding, MemberBody, Pattern, Program, Statement};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorSlot};

/// Canonical linked targets available while building source identity.
#[derive(Clone, Debug, Default)]
pub struct SourceIndexContext {
    /// Linked module targets keyed by their canonical source path spelling.
    pub modules: BTreeMap<String, ModuleId>,
    /// Linked exported targets keyed by `(module, exported name)`.
    pub targets: BTreeMap<(ModuleId, String), SemanticTargetId>,
    /// Canonical linked import resolution keyed by importing module and the
    /// written logical path. This prevents source indexing from falling back
    /// to a disconnected default context.
    pub resolved_imports: BTreeMap<(ModuleId, String), ModuleId>,
}

impl SourceIndexContext {
    /// Adds one canonical module path mapping.
    pub fn with_module(mut self, path: impl Into<String>, module: ModuleId) -> Self {
        self.modules.insert(path.into(), module);
        self
    }

    /// Adds one canonical exported target mapping.
    pub fn with_target(mut self, module: ModuleId, name: impl Into<String>, target: SemanticTargetId) -> Self {
        self.targets.insert((module, name.into()), target);
        self
    }

    pub fn with_resolved_import(mut self, importer: ModuleId, path: impl Into<String>, module: ModuleId) -> Self {
        self.resolved_imports.insert((importer, path.into()), module);
        self
    }
}

/// Builds the compiler-owned lexical source index for one parsed module.
pub fn build_source_scope_index(module: ModuleId, program: &Program, context: &SourceIndexContext) -> SourceScopeIndex {
    let mut builder = SourceScopeBuilder {
        index: SourceScopeIndex::new(module.clone(), statements_range(&program.statements)),
        context,
        next_scope: 1,
        next_site: BTreeMap::new(),
        current_owner: SourceOwner::Module(module),
    };

    for statement in &program.statements {
        if let Statement::Class(class) = statement {
            let declaration = builder.declaration_id(class);
            builder.index.register_class(class.name.clone(), declaration);
        }
    }
    builder.visit_imports(program);
    builder.visit_statements(builder.index.root, &program.statements, true);
    builder.index.finish_scope_order();
    builder.index
}

struct SourceScopeBuilder<'a> {
    index: SourceScopeIndex,
    context: &'a SourceIndexContext,
    next_scope: u32,
    next_site: BTreeMap<SourceOwner, u32>,
    current_owner: SourceOwner,
}

impl SourceScopeBuilder<'_> {
    fn declaration_id(&self, class: &ClassDef) -> DeclarationId {
        DeclarationId::new(self.index.module.clone(), class.name.clone().into())
    }

    fn new_scope(&mut self, parent: SourceScopeId, range: SourceRange) -> SourceScopeId {
        let id = SourceScopeId(self.next_scope);
        self.next_scope += 1;
        self.index.add_scope(id, parent, range);
        id
    }

    fn allocate_site(&mut self, owner: SourceOwner, range: SourceRange, kind: SourceSiteKind) -> SourceSiteId {
        let next = self.next_site.entry(owner.clone()).or_default();
        let site = SourceSite::new(owner, SourceSiteLocalId(*next), range, kind);
        *next += 1;
        let id = site.id.clone();
        self.index.register_site(site);
        id
    }

    fn declare(&mut self, scope: SourceScopeId, name: impl Into<Box<str>>, kind: SourceBindingKind, range: SourceRange, mutable: bool) -> SourceSiteId {
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

    fn visit_imports(&mut self, program: &Program) {
        for dependency in &program.preamble.dependencies {
            let phalcom_ast::ast::DependencyDecl::Import(import) = dependency else {
                continue;
            };
            match import {
                phalcom_ast::ast::ImportDecl::Module(module_import) => {
                    let name = module_import
                        .alias
                        .as_ref()
                        .map(|alias| alias.name.clone())
                        .or_else(|| module_import.path.segments.last().map(|segment| segment.name.clone()))
                        .or_else(|| match &module_import.path.root {
                            phalcom_ast::ast::ImportRoot::Absolute(segment) => Some(segment.name.clone()),
                            phalcom_ast::ast::ImportRoot::Relative { .. } => None,
                        });
                    let Some(name) = name else { continue };
                    let range = module_import.alias.as_ref().map_or(module_import.range, |alias| alias.range);
                    let site = self.declare(self.index.root, name.clone(), SourceBindingKind::Import, range, false);
                    if let Some(module) = self
                        .context
                        .resolved_imports
                        .get(&(self.index.module.clone(), module_import.path.to_string()))
                        .or_else(|| self.context.modules.get(&module_import.path.to_string()))
                    {
                        self.index.register_module(name, module.clone());
                        self.index.register_target(site, SemanticTargetId::Module(module.clone()));
                    }
                }
                phalcom_ast::ast::ImportDecl::Selective(selective_import) => {
                    let module = self
                        .context
                        .resolved_imports
                        .get(&(self.index.module.clone(), selective_import.path.to_string()))
                        .or_else(|| self.context.modules.get(&selective_import.path.to_string()));
                    for item in &selective_import.items {
                        let name = item.alias.as_ref().map_or_else(|| item.name.clone(), |alias| alias.name.clone());
                        let range = item.alias.as_ref().map_or(item.name_range, |alias| alias.range);
                        let site = self.declare(self.index.root, name, SourceBindingKind::Import, range, false);
                        if let Some(module) = module
                            && let Some(target) = self.context.targets.get(&(module.clone(), item.name.clone()))
                        {
                            self.index.register_target(site, target.clone());
                        }
                    }
                }
            }
        }
    }

    fn visit_statements(&mut self, scope: SourceScopeId, statements: &[Statement], top_level: bool) {
        for statement in statements {
            match statement {
                Statement::Class(class) => self.visit_class(scope, class),
                Statement::Let(binding) => self.visit_let(scope, binding, top_level),
                Statement::Return(return_statement) => {
                    if let Some(value) = &return_statement.value {
                        self.visit_expr(scope, value);
                    }
                }
                Statement::Expr { expr, .. } => self.visit_expr(scope, expr),
                Statement::For(for_statement) => self.visit_for(scope, for_statement),
                Statement::Throw { expr, .. } => self.visit_expr(scope, expr),
                Statement::Export(_) | Statement::Break { .. } | Statement::Continue { .. } | Statement::TypeAlias(_) => {}
            }
        }
    }

    fn visit_class(&mut self, parent: SourceScopeId, class: &ClassDef) {
        let declaration = self.declaration_id(class);
        let site = self.allocate_site(
            SourceOwner::Module(self.index.module.clone()),
            class.name_range,
            SourceSiteKind::Declaration(declaration.clone()),
        );
        self.index.register_class(class.name.clone(), declaration.clone());
        self.index.register_target(site, SemanticTargetId::Declaration(declaration.clone()));
        for member in &class.members {
            self.visit_member(parent, &declaration, member);
        }
    }

    fn visit_member(&mut self, parent: SourceScopeId, declaration: &DeclarationId, member: &ClassMember) {
        match member {
            ClassMember::Method(method) => {
                let Some(callable) = self.method_callable(declaration, method) else { return };
                self.visit_callable(
                    parent,
                    callable,
                    method.name_range,
                    method.range,
                    &method.params,
                    &method.body,
                    SourceBindingKind::MethodParameter,
                );
            }
            ClassMember::Getter(getter) => {
                let Some(callable) = Selector::getter(&getter.name)
                    .ok()
                    .map(|selector| CallableId::new(declaration.clone(), selector, side(getter.is_static)))
                else {
                    return;
                };
                self.visit_callable(
                    parent,
                    callable,
                    getter.name_range,
                    getter.range,
                    &[],
                    &getter.body,
                    SourceBindingKind::MethodParameter,
                );
            }
            ClassMember::Setter(setter) => {
                let Some(callable) = Selector::setter(&setter.name)
                    .ok()
                    .map(|selector| CallableId::new(declaration.clone(), selector, side(setter.is_static)))
                else {
                    return;
                };
                self.visit_callable(
                    parent,
                    callable,
                    setter.name_range,
                    setter.range,
                    std::slice::from_ref(&setter.param),
                    &setter.body,
                    SourceBindingKind::SetterParameter,
                );
            }
            ClassMember::Index(index) => {
                let slots = index.params.iter().map(parameter_slot).collect::<Vec<_>>();
                let selector = match index.accessor {
                    phalcom_ast::ast::IndexAccessor::Get => Selector::subscript_get(slots),
                    phalcom_ast::ast::IndexAccessor::Set { .. } => Selector::subscript_set(slots),
                };
                let Some(selector) = selector.ok() else { return };
                let callable = CallableId::new(declaration.clone(), selector, DispatchSide::Instance);
                let mut parameters = index.params.clone();
                if let phalcom_ast::ast::IndexAccessor::Set { put } = &index.accessor {
                    parameters.push((**put).clone());
                }
                self.visit_callable(
                    parent,
                    callable,
                    index.name_range,
                    index.range,
                    &parameters,
                    &MemberBody::Block(index.body.clone()),
                    SourceBindingKind::IndexParameter,
                );
            }
            ClassMember::Field(field) => {
                let field_id = FieldId::new(declaration.clone(), field.name.clone(), side(field.is_static));
                let site = self.allocate_site(
                    SourceOwner::Module(self.index.module.clone()),
                    field.name_range,
                    SourceSiteKind::Field(field_id.clone()),
                );
                self.index.register_target(site, SemanticTargetId::Field(field_id));
                if let Some(default) = &field.default {
                    self.visit_expr(parent, default);
                }
            }
            ClassMember::Variant(_) => {}
        }
    }

    fn method_callable(&self, declaration: &DeclarationId, method: &phalcom_ast::ast::MethodDef) -> Option<CallableId> {
        let slots = method.params.iter().map(parameter_slot).collect::<Vec<_>>();
        Selector::method(&method.name, slots).ok().map(|selector| {
            CallableId::new(
                declaration.clone(),
                selector,
                if method.is_constructor { DispatchSide::Class } else { side(method.is_static) },
            )
        })
    }

    fn visit_callable(
        &mut self,
        parent: SourceScopeId,
        callable: CallableId,
        name_range: SourceRange,
        body_range: SourceRange,
        parameters: &[phalcom_ast::ast::ParameterDef],
        body: &MemberBody,
        parameter_kind: SourceBindingKind,
    ) {
        let declaration_site = self.allocate_site(
            SourceOwner::Module(self.index.module.clone()),
            name_range,
            SourceSiteKind::Callable(callable.clone()),
        );
        self.index.register_target(declaration_site, SemanticTargetId::Callable(callable.clone()));
        let previous_owner = std::mem::replace(&mut self.current_owner, SourceOwner::Callable(callable));
        let scope = self.new_scope(parent, body_range);
        for parameter in parameters {
            self.declare(scope, parameter.name.clone(), parameter_kind, parameter.name_range, true);
        }
        if let Some(statements) = body.statements() {
            self.visit_statements(scope, statements, false);
        }
        self.current_owner = previous_owner;
    }

    fn visit_let(&mut self, scope: SourceScopeId, binding: &LetBinding, top_level: bool) {
        if let Some(value) = &binding.value {
            self.visit_expr(scope, value);
        }
        let kind = match (top_level, binding.kind) {
            (true, BindingKind::Let) => SourceBindingKind::TopLevelLet,
            (true, BindingKind::Const) => SourceBindingKind::TopLevelConst,
            (false, BindingKind::Let) => SourceBindingKind::LocalLet,
            (false, BindingKind::Const) => SourceBindingKind::LocalConst,
        };
        self.declare_pattern(scope, &binding.pattern, kind, binding.kind == BindingKind::Let);
    }

    fn declare_pattern(&mut self, scope: SourceScopeId, pattern: &Pattern, kind: SourceBindingKind, mutable: bool) {
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

    fn visit_for(&mut self, parent: SourceScopeId, statement: &ForStatement) {
        for lane in &statement.lanes {
            self.visit_expr(parent, &lane.iter);
        }
        let scope = self.new_scope(parent, statements_range(&statement.body));
        for lane in &statement.lanes {
            self.declare_pattern(scope, &lane.pattern, SourceBindingKind::ForBinding, true);
            if let Some(index) = &lane.index {
                self.declare(scope, index.name.clone(), SourceBindingKind::ForBinding, index.range, true);
            }
        }
        self.visit_statements(scope, &statement.body, false);
    }

    fn visit_expr(&mut self, scope: SourceScopeId, expr: &Expr) {
        match expr {
            Expr::Assignment(assignment) => {
                self.visit_expr(scope, &assignment.name);
                self.visit_expr(scope, &assignment.value);
            }
            Expr::Range(range) => {
                if let Some(lower) = &range.lower {
                    self.visit_expr(scope, lower);
                }
                if let Some(upper) = &range.upper {
                    self.visit_expr(scope, upper);
                }
            }
            Expr::Unary(unary) => self.visit_expr(scope, &unary.expr),
            Expr::Binary(binary) => {
                self.visit_expr(scope, &binary.left);
                self.visit_expr(scope, &binary.right);
            }
            Expr::ComparisonChain(chain) => {
                for operand in &chain.operands {
                    self.visit_expr(scope, operand);
                }
            }
            Expr::IfLet(if_let) => {
                self.visit_expr(scope, &if_let.value);
                let then_scope = self.new_scope(scope, if_let.then_body.range);
                self.declare_pattern(then_scope, &if_let.pattern, SourceBindingKind::Destructure, true);
                self.visit_block_contents(then_scope, &if_let.then_body);
                if let Some(else_body) = &if_let.else_body {
                    self.visit_block_contents(scope, else_body);
                }
            }
            Expr::WhileLet(while_let) => {
                let loop_scope = self.new_scope(scope, while_let.range);
                self.visit_expr(loop_scope, &while_let.value);
                self.declare_pattern(loop_scope, &while_let.pattern, SourceBindingKind::Destructure, true);
                self.visit_statements(loop_scope, &while_let.body, false);
            }
            Expr::Membership(membership) => {
                self.visit_expr(scope, &membership.left);
                self.visit_expr(scope, &membership.right);
            }
            Expr::IsMembership(membership) => {
                self.visit_expr(scope, &membership.left);
                self.visit_expr(scope, &membership.candidates);
            }
            Expr::UnqualifiedCall(call) => self.visit_pack_items(scope, &call.args),
            Expr::MethodCall(call) => {
                self.visit_expr(scope, &call.object);
                self.visit_pack_items(scope, &call.args);
            }
            Expr::GetProperty(property) => self.visit_expr(scope, &property.object),
            Expr::SetProperty(property) => {
                self.visit_expr(scope, &property.object);
                self.visit_expr(scope, &property.value);
            }
            Expr::Index(index) => {
                self.visit_expr(scope, &index.object);
                self.visit_pack_items(scope, &index.args);
            }
            Expr::SetIndex(index) => {
                self.visit_expr(scope, &index.object);
                self.visit_pack_items(scope, &index.args);
                self.visit_expr(scope, &index.value);
            }
            Expr::Block(block) => self.visit_block(scope, block),
            Expr::MethodRef(reference) => self.visit_expr(scope, &reference.receiver),
            Expr::TupleLiteral(tuple) => {
                for entry in &tuple.entries {
                    match entry {
                        phalcom_ast::ast::TupleLiteralEntry::Positional { expr, .. } | phalcom_ast::ast::TupleLiteralEntry::Expand { expr, .. } => {
                            self.visit_expr(scope, expr)
                        }
                        phalcom_ast::ast::TupleLiteralEntry::Labeled { label, value, .. } => {
                            self.visit_product_label(scope, label);
                            self.visit_expr(scope, value);
                        }
                    }
                }
            }
            Expr::RecordLiteral(record) => {
                for entry in &record.entries {
                    match entry {
                        phalcom_ast::ast::RecordLiteralEntry::Field(field) => {
                            self.visit_product_label(scope, &field.label);
                            self.visit_expr(scope, &field.value);
                        }
                        phalcom_ast::ast::RecordLiteralEntry::Expansion { expr, .. } => self.visit_expr(scope, expr),
                    }
                }
            }
            Expr::MapLiteral(map) => {
                for entry in &map.entries {
                    match entry {
                        phalcom_ast::ast::MapLiteralEntry::Association { key, value, .. } => {
                            if let phalcom_ast::ast::MapLiteralKey::Computed { expr, .. } = key {
                                self.visit_expr(scope, expr);
                            }
                            self.visit_expr(scope, value);
                        }
                        phalcom_ast::ast::MapLiteralEntry::Expansion { expr, .. } => self.visit_expr(scope, expr),
                    }
                }
            }
            Expr::SetLiteral(set) => {
                for entry in &set.entries {
                    match entry {
                        phalcom_ast::ast::SetLiteralEntry::Element { expr, .. } | phalcom_ast::ast::SetLiteralEntry::Expansion { expr, .. } => {
                            self.visit_expr(scope, expr)
                        }
                    }
                }
            }
            Expr::ListLiteral(list) => {
                for element in &list.elements {
                    match element {
                        phalcom_ast::ast::ListLiteralElement::Element { expr, .. } | phalcom_ast::ast::ListLiteralElement::Expansion { expr, .. } => {
                            self.visit_expr(scope, expr)
                        }
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

    fn visit_block(&mut self, parent: SourceScopeId, block: &BlockExpr) {
        let scope = self.new_scope(parent, block.range);
        self.visit_block_contents(scope, block);
    }

    fn visit_block_contents(&mut self, scope: SourceScopeId, block: &BlockExpr) {
        for parameter in &block.params.fixed {
            self.declare(scope, parameter.name.clone(), SourceBindingKind::ClosureParameter, parameter.range, true);
        }
        if let Some(parameter) = &block.params.positional_rest {
            self.declare(scope, parameter.name.clone(), SourceBindingKind::ClosureParameter, parameter.range, true);
        }
        self.visit_statements(scope, &block.body, false);
    }

    fn visit_pack_items(&mut self, scope: SourceScopeId, items: &[phalcom_ast::ast::PackItem]) {
        for item in items {
            match item {
                phalcom_ast::ast::PackItem::Positional { expr, .. }
                | phalcom_ast::ast::PackItem::Expand { expr, .. }
                | phalcom_ast::ast::PackItem::Labeled { value: expr, .. } => self.visit_expr(scope, expr),
            }
        }
    }

    fn visit_product_label(&mut self, scope: SourceScopeId, label: &phalcom_ast::ast::ProductLabel) {
        if let phalcom_ast::ast::ProductLabel::Computed { expr, .. } = label {
            self.visit_expr(scope, expr);
        }
    }
}

fn parameter_slot(parameter: &phalcom_ast::ast::ParameterDef) -> SelectorSlot {
    parameter
        .label
        .as_ref()
        .map_or(SelectorSlot::Positional, |label| SelectorSlot::Label(label.clone()))
}

fn side(is_static: bool) -> DispatchSide {
    if is_static { DispatchSide::Class } else { DispatchSide::Instance }
}

fn statements_range(statements: &[Statement]) -> SourceRange {
    statements
        .iter()
        .map(statement_range)
        .reduce(|left, right| left.merge(&right))
        .unwrap_or_default()
}

fn statement_range(statement: &Statement) -> SourceRange {
    match statement {
        Statement::Class(class) => class.range,
        Statement::Let(binding) => binding.range,
        Statement::Return(return_statement) => return_statement.range,
        Statement::Expr { range, .. } => *range,
        Statement::For(for_statement) => for_statement.range,
        Statement::Break { range } | Statement::Continue { range } => *range,
        Statement::Throw { range, .. } => *range,
        Statement::Export(export_decl) => export_decl.range,
        Statement::TypeAlias(type_alias) => type_alias.range,
    }
}
