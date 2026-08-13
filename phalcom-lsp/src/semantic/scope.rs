//! Lexical scopes and source-local binding identities.

use std::collections::BTreeMap;

use phalcom_ast::ast::{BindingKind, BlockExpr, ClassMember, Expr, ForStatement, LetBinding, Pattern, Program, Statement};
use phalcom_common::range::SourceRange;

use super::ids::{ClassId, ModuleId};

/// Identity of one lexical scope in a file snapshot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScopeId(u32);

/// Identity of one lexical binding in a file snapshot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BindingId(u32);

/// Semantic category of a source binding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticBindingKind {
    /// A module-level mutable binding.
    TopLevelLet,
    /// A module-level immutable binding.
    TopLevelConst,
    /// A local mutable binding.
    LocalLet,
    /// A local immutable binding.
    LocalConst,
    /// A method or constructor parameter.
    MethodParameter,
    /// A setter value parameter.
    SetterParameter,
    /// An index setter value parameter.
    IndexParameter,
    /// A closure parameter.
    ClosureParameter,
    /// A `for` loop binding.
    ForBinding,
    /// A name introduced by a destructuring pattern.
    Destructure,
    /// A module alias introduced by an import.
    Import,
}

/// Source identity and declaration metadata for one binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingInfo {
    /// Binding identity, stable for this file snapshot.
    pub id: BindingId,
    /// Scope containing the declaration.
    pub scope: ScopeId,
    /// Local spelling.
    pub name: String,
    /// Source-level binding category.
    pub kind: SemanticBindingKind,
    /// Exact declaration token range.
    pub declaration_range: SourceRange,
    /// Whether the source binding can be reassigned.
    pub mutable: bool,
}

/// One lexical scope and its direct bindings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeInfo {
    /// Scope identity.
    pub id: ScopeId,
    /// Lexical parent, if this is not the module scope.
    pub parent: Option<ScopeId>,
    /// Source region covered by this scope.
    pub range: SourceRange,
    /// Direct bindings keyed by spelling. Same-scope recovery keeps first declaration.
    pub bindings: BTreeMap<String, BindingId>,
}

/// Name identity resolved without inferring a runtime value shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NameResolution {
    /// A lexical binding.
    Binding(BindingId),
    /// A class declared in this module or the live core surface.
    Class(ClassId),
    /// A module namespace/import binding.
    Module(ModuleId),
    /// A name that may be handled by implicit-self dispatch.
    ImplicitSelf,
    /// A global name not represented by a local binding.
    Global(String),
    /// No semantic identity is known yet.
    Unresolved,
}

/// Per-file lexical graph used by source-oriented editor queries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeGraph {
    /// Module identity owning this graph.
    pub module: ModuleId,
    /// Root module scope.
    pub root: ScopeId,
    /// All scopes, including root.
    pub scopes: BTreeMap<ScopeId, ScopeInfo>,
    /// All bindings, including declarations in nested scopes.
    pub bindings: BTreeMap<BindingId, BindingInfo>,
    /// Scope IDs sorted by start offset for bounded interval lookup.
    scope_order: Vec<ScopeId>,
    /// Prefix maximum end offset for `scope_order`.
    scope_max_end_prefix: Vec<usize>,
    /// Direct declaration lookup by exact source range.
    declarations: BTreeMap<(usize, usize), BindingId>,
    classes: BTreeMap<String, ClassId>,
}

impl ScopeGraph {
    /// Finds the innermost scope containing `offset`.
    pub fn scope_at(&self, offset: usize) -> ScopeId {
        let mut low = 0;
        let mut high = self.scope_order.len();
        while low < high {
            let middle = (low + high) / 2;
            let scope = &self.scopes[&self.scope_order[middle]];
            if scope.range.start <= offset {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        let mut best = None;
        let mut index = low;
        while index > 0 {
            index -= 1;
            if self.scope_max_end_prefix.get(index).copied().unwrap_or(0) <= offset {
                break;
            }
            let scope = &self.scopes[&self.scope_order[index]];
            if scope.range.contains(offset) && best.is_none_or(|best: &ScopeInfo| scope.range.len() < best.range.len()) {
                best = Some(scope);
            }
        }
        best.map(|scope| scope.id).unwrap_or(self.root)
    }

    /// Resolves one spelling from the nearest lexical scope outward.
    pub fn resolve(&self, scope: ScopeId, name: &str, offset: usize) -> NameResolution {
        let mut current = Some(scope);
        while let Some(scope_id) = current {
            let Some(scope_info) = self.scopes.get(&scope_id) else { break };
            if let Some(binding_id) = scope_info.bindings.get(name) {
                if let Some(binding) = self.bindings.get(binding_id)
                    && binding.declaration_range.start <= offset
                {
                    return NameResolution::Binding(*binding_id);
                }
            }
            current = scope_info.parent;
        }
        if let Some(class) = self.classes.get(name) {
            return NameResolution::Class(class.clone());
        }
        if name == "self" {
            return NameResolution::ImplicitSelf;
        }
        NameResolution::Global(name.to_string())
    }

    /// Returns the binding declared at exactly `range`, if any.
    pub fn binding_for_declaration(&self, range: SourceRange) -> Option<BindingId> {
        self.declarations.get(&(range.start, range.end)).copied()
    }

    /// Returns bindings visible at a source offset, nearest scope first and
    /// with shadowed spellings removed.
    pub fn visible_bindings_at(&self, offset: usize) -> Vec<BindingInfo> {
        let mut visible = BTreeMap::new();
        let mut current = Some(self.scope_at(offset));
        while let Some(scope_id) = current {
            let Some(scope) = self.scopes.get(&scope_id) else { break };
            for (name, binding_id) in &scope.bindings {
                if visible.contains_key(name) {
                    continue;
                }
                let Some(binding) = self.bindings.get(binding_id) else { continue };
                if binding.declaration_range.start <= offset {
                    visible.insert(name.clone(), binding.clone());
                }
            }
            current = scope.parent;
        }
        visible.into_values().collect()
    }
}

/// Builds one lexical graph from the recovered source AST.
pub fn build_scope_graph(module: ModuleId, program: &Program) -> ScopeGraph {
    let root = ScopeId(0);
    let mut builder = ScopeBuilder {
        graph: ScopeGraph {
            module,
            root,
            scopes: BTreeMap::new(),
            bindings: BTreeMap::new(),
            scope_order: Vec::new(),
            scope_max_end_prefix: Vec::new(),
            declarations: BTreeMap::new(),
            classes: BTreeMap::new(),
        },
        next_scope: 1,
        next_binding: 0,
    };
    builder.graph.scopes.insert(
        root,
        ScopeInfo {
            id: root,
            parent: None,
            range: statements_range(&program.statements),
            bindings: BTreeMap::new(),
        },
    );
    for statement in &program.statements {
        if let Statement::Class(class) = statement {
            builder
                .graph
                .classes
                .insert(class.name.clone(), ClassId::new(builder.graph.module.clone(), class.name.clone()));
        }
    }
    builder.visit_statements(root, &program.statements, true);
    builder.graph.scope_order = builder.graph.scopes.keys().copied().collect();
    builder
        .graph
        .scope_order
        .sort_by_key(|id| (builder.graph.scopes[id].range.start, builder.graph.scopes[id].range.end));
    let mut max_end = 0;
    builder.graph.scope_max_end_prefix = builder
        .graph
        .scope_order
        .iter()
        .map(|id| {
            max_end = max_end.max(builder.graph.scopes[id].range.end);
            max_end
        })
        .collect();
    builder.graph
}

struct ScopeBuilder {
    graph: ScopeGraph,
    next_scope: u32,
    next_binding: u32,
}

impl ScopeBuilder {
    fn new_scope(&mut self, parent: ScopeId, range: SourceRange) -> ScopeId {
        let id = ScopeId(self.next_scope);
        self.next_scope += 1;
        self.graph.scopes.insert(
            id,
            ScopeInfo {
                id,
                parent: Some(parent),
                range,
                bindings: BTreeMap::new(),
            },
        );
        id
    }

    fn declare(&mut self, scope: ScopeId, name: String, kind: SemanticBindingKind, range: SourceRange, mutable: bool) -> BindingId {
        if let Some(existing) = self.graph.scopes.get(&scope).and_then(|scope| scope.bindings.get(&name)).copied() {
            return existing;
        }
        let id = BindingId(self.next_binding);
        self.next_binding += 1;
        self.graph.bindings.insert(
            id,
            BindingInfo {
                id,
                scope,
                name: name.clone(),
                kind,
                declaration_range: range,
                mutable,
            },
        );
        self.graph.declarations.insert((range.start, range.end), id);
        self.graph.scopes.get_mut(&scope).expect("scope exists").bindings.insert(name, id);
        id
    }

    fn visit_statements(&mut self, scope: ScopeId, statements: &[Statement], top_level: bool) {
        for statement in statements {
            match statement {
                Statement::Class(class) => {
                    for member in &class.members {
                        self.visit_member(scope, member);
                    }
                }
                Statement::Let(binding) => self.visit_let(scope, binding, top_level),
                Statement::Return(return_statement) => {
                    if let Some(value) = &return_statement.value {
                        self.visit_expr(scope, value);
                    }
                }
                Statement::Expr { expr, .. } => self.visit_expr(scope, expr),
                Statement::For(for_statement) => self.visit_for(scope, for_statement),
                Statement::Throw { expr, .. } => self.visit_expr(scope, expr),
                Statement::Import(import) => {
                    self.declare(scope, import.binding.clone(), SemanticBindingKind::Import, import.binding_range, false);
                }
                Statement::Break { .. } | Statement::Continue { .. } => {}
            }
        }
    }

    fn visit_member(&mut self, parent: ScopeId, member: &ClassMember) {
        let (range, params, body, parameter_kind) = match member {
            ClassMember::Method(method) => (
                method.range,
                method.params.as_slice(),
                method.body.as_slice(),
                SemanticBindingKind::MethodParameter,
            ),
            ClassMember::Getter(getter) => (getter.range, &[][..], getter.body.as_slice(), SemanticBindingKind::MethodParameter),
            ClassMember::Setter(setter) => (
                setter.range,
                std::slice::from_ref(&setter.param),
                setter.body.as_slice(),
                SemanticBindingKind::SetterParameter,
            ),
            ClassMember::Index(index) => {
                let mut all = index.params.clone();
                if let phalcom_ast::ast::IndexAccessor::Set { put } = &index.accessor {
                    all.push(put.clone());
                }
                let scope = self.new_scope(parent, index.range);
                for parameter in &all {
                    self.declare(scope, parameter.name.clone(), SemanticBindingKind::IndexParameter, parameter.name_range, true);
                }
                self.visit_statements(scope, &index.body, false);
                return;
            }
            ClassMember::Field(_) | ClassMember::Variant(_) => return,
        };
        let scope = self.new_scope(parent, range);
        for parameter in params {
            self.declare(scope, parameter.name.clone(), parameter_kind, parameter.name_range, true);
        }
        self.visit_statements(scope, body, false);
    }

    fn visit_let(&mut self, scope: ScopeId, binding: &LetBinding, top_level: bool) {
        if let Some(value) = &binding.value {
            self.visit_expr(scope, value);
        }
        let kind = match (top_level, binding.kind) {
            (true, BindingKind::Let) => SemanticBindingKind::TopLevelLet,
            (true, BindingKind::Const) => SemanticBindingKind::TopLevelConst,
            (false, BindingKind::Let) => SemanticBindingKind::LocalLet,
            (false, BindingKind::Const) => SemanticBindingKind::LocalConst,
        };
        self.declare_pattern(scope, &binding.pattern, kind, binding.kind == BindingKind::Let);
    }

    fn declare_pattern(&mut self, scope: ScopeId, pattern: &Pattern, kind: SemanticBindingKind, mutable: bool) {
        match pattern {
            Pattern::Name { name, range } => {
                self.declare(scope, name.clone(), kind, *range, mutable);
            }
            Pattern::Tuple { elements, .. } => {
                for element in elements {
                    self.declare_pattern(scope, element, SemanticBindingKind::Destructure, mutable);
                }
            }
            Pattern::List { elements, rest, .. } => {
                for element in elements {
                    self.declare_pattern(scope, element, SemanticBindingKind::Destructure, mutable);
                }
                if let Some(rest) = rest {
                    self.declare_pattern(scope, rest, SemanticBindingKind::Destructure, mutable);
                }
            }
        }
    }

    fn visit_for(&mut self, parent: ScopeId, statement: &ForStatement) {
        self.visit_expr(parent, &statement.iter);
        let body_range = statements_range(&statement.body);
        let scope = self.new_scope(parent, body_range);
        self.declare(scope, statement.binding.clone(), SemanticBindingKind::ForBinding, statement.binding_range, true);
        self.visit_statements(scope, &statement.body, false);
    }

    fn visit_expr(&mut self, scope: ScopeId, expr: &Expr) {
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
            | Expr::Symbol { .. } => {}
        }
    }

    fn visit_block(&mut self, parent: ScopeId, block: &BlockExpr) {
        let scope = self.new_scope(parent, block.range);
        for parameter in &block.params.fixed {
            self.declare(scope, parameter.name.clone(), SemanticBindingKind::ClosureParameter, parameter.range, true);
        }
        if let Some(parameter) = &block.params.positional_rest {
            self.declare(scope, parameter.name.clone(), SemanticBindingKind::ClosureParameter, parameter.range, true);
        }
        self.visit_statements(scope, &block.body, false);
    }

    fn visit_pack_items(&mut self, scope: ScopeId, items: &[phalcom_ast::ast::PackItem]) {
        for item in items {
            match item {
                phalcom_ast::ast::PackItem::Positional { expr, .. }
                | phalcom_ast::ast::PackItem::Expand { expr, .. }
                | phalcom_ast::ast::PackItem::Labeled { value: expr, .. } => self.visit_expr(scope, expr),
            }
        }
    }

    fn visit_product_label(&mut self, scope: ScopeId, label: &phalcom_ast::ast::ProductLabel) {
        if let phalcom_ast::ast::ProductLabel::Computed { expr, .. } = label {
            self.visit_expr(scope, expr);
        }
    }
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
        Statement::Import(import) => import.range,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::parser::parse;

    #[test]
    fn method_parameter_shadows_module_binding_by_identity() {
        let source = "let value = 1\nclass Sample {\n  method(value) { value }\n}\n";
        let parsed = parse(source, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        let module = ModuleId::new("file:///sample.ph");
        let graph = build_scope_graph(module, &parsed.program);
        let use_offset = source.rfind("value }").expect("method use") + 1;
        let method_scope = graph.scope_at(use_offset);
        let visible = graph.visible_bindings_at(use_offset);
        assert_eq!(visible[0].name, "value");
        assert_eq!(visible[0].scope, method_scope);
        assert_eq!(graph.bindings.len(), 2);
        assert_eq!(graph.resolve(method_scope, "value", use_offset), NameResolution::Binding(visible[0].id));
    }

    #[test]
    fn closure_parameter_uses_exact_ast_range() {
        let source = "let mapper = |value| value\n";
        let parsed = parse(source, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        let graph = build_scope_graph(ModuleId::new("file:///closure.ph"), &parsed.program);
        let parameter_start = source.find("value").expect("closure parameter");
        let parameter_range = (parameter_start..parameter_start + "value".len()).into();
        let binding = graph.binding_for_declaration(parameter_range).expect("closure binding");
        assert_eq!(graph.bindings[&binding].kind, SemanticBindingKind::ClosureParameter);
    }
}
