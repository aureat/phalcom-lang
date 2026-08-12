//! Exact source occurrences shared by hover, navigation, and semantic tokens.

use phalcom_ast::ast::{
    BinaryOp, ClassMember, Expr, IndexAccessor, MethodRefKind, Pattern, Program, Statement, UnaryOp,
};
use phalcom_common::range::SourceRange;

use super::ids::{CallableId, ClassId, ModuleId};
use super::scope::{BindingId, ScopeGraph};
use super::surface::{MemberSurface, ModuleSurface};

/// Kind of semantic source token under the cursor.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticOccurrenceKind {
    /// A local/module binding.
    Binding,
    /// A callable parameter declaration/reference.
    Parameter,
    /// A class declaration/reference.
    Class,
    /// A member selector or property reference.
    Member,
    /// A field declaration/reference.
    Field,
    /// A written operator token.
    Operator,
}

/// Role of one semantic occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OccurrenceRole {
    /// Binding/member/class declaration.
    Declaration,
    /// Value read or member getter/call.
    Read,
    /// Assignment target.
    Write,
    /// Callable/member call.
    Call,
    /// Method reference.
    Reference,
}

/// Semantic identity attached to one exact source range.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SemanticTarget {
    /// Lexical binding identity.
    Binding(BindingId),
    /// Module-qualified class identity.
    Class(ClassId),
    /// Resolved callable declaration.
    Callable(CallableId),
    /// Field identity, ready for later field facts.
    Field {
        /// Class owning field.
        owner: ClassId,
        /// Field spelling.
        name: String,
    },
    /// Member name awaiting Spec 2 dispatch enrichment.
    Member {
        /// Unresolved selector spelling.
        name: String,
    },
    /// Written operator spelling.
    Operator(String),
}

/// One exact semantic token occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticOccurrence {
    /// Half-open source byte range of the semantic token.
    pub range: SourceRange,
    /// Broad semantic category.
    pub kind: SemanticOccurrenceKind,
    /// Syntactic role at this location.
    pub role: OccurrenceRole,
    /// Identity consumed by editor adapters.
    pub target: SemanticTarget,
}

/// Sorted exact occurrence index for one source file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OccurrenceIndex {
    occurrences: Vec<SemanticOccurrence>,
}

impl OccurrenceIndex {
    /// Returns all occurrences in source order.
    pub fn all(&self) -> &[SemanticOccurrence] {
        &self.occurrences
    }

    /// Selects the shortest semantic range containing `offset`.
    pub fn occurrence_at(&self, offset: usize) -> Option<&SemanticOccurrence> {
        self.occurrences
            .iter()
            .filter(|occurrence| occurrence.range.contains(offset))
            .min_by(|left, right| {
                left.range
                    .len()
                    .cmp(&right.range.len())
                    .then_with(|| occurrence_priority(left.kind).cmp(&occurrence_priority(right.kind)))
                    .then_with(|| left.range.start.cmp(&right.range.start))
            })
    }
}

/// Builds exact semantic occurrences from source AST and lexical identities.
pub fn build_occurrence_index(module: ModuleId, program: &Program, surface: &ModuleSurface, scopes: &ScopeGraph) -> OccurrenceIndex {
    let mut builder = OccurrenceBuilder {
        module,
        surface,
        scopes,
        occurrences: Vec::new(),
    };
    builder.visit_statements(&program.statements, scopes.root);
    builder.occurrences.sort_by(|left, right| {
        left.range
            .start
            .cmp(&right.range.start)
            .then_with(|| left.range.len().cmp(&right.range.len()))
            .then_with(|| occurrence_priority(left.kind).cmp(&occurrence_priority(right.kind)))
    });
    OccurrenceIndex {
        occurrences: builder.occurrences,
    }
}

struct OccurrenceBuilder<'a> {
    module: ModuleId,
    surface: &'a ModuleSurface,
    scopes: &'a ScopeGraph,
    occurrences: Vec<SemanticOccurrence>,
}

impl OccurrenceBuilder<'_> {
    fn push(&mut self, range: SourceRange, kind: SemanticOccurrenceKind, role: OccurrenceRole, target: SemanticTarget) {
        if range.is_empty() {
            return;
        }
        self.occurrences.push(SemanticOccurrence { range, kind, role, target });
    }

    fn visit_statements(&mut self, statements: &[Statement], scope: super::scope::ScopeId) {
        for statement in statements {
            match statement {
                Statement::Class(class) => {
                    let class_id = ClassId::new(self.module.clone(), class.name.clone());
                    self.push(class.name_range, SemanticOccurrenceKind::Class, OccurrenceRole::Declaration, SemanticTarget::Class(class_id.clone()));
                    for member in &class.members {
                        self.visit_member(&class_id, member);
                    }
                }
                Statement::Let(binding) => {
                    if let Some(value) = &binding.value {
                        self.visit_expr(value, scope);
                    }
                    self.visit_pattern_declarations(&binding.pattern, scope);
                }
                Statement::Return(return_statement) => {
                    if let Some(value) = &return_statement.value {
                        self.visit_expr(value, scope);
                    }
                }
                Statement::Expr { expr, .. } => self.visit_expr(expr, scope),
                Statement::For(statement) => {
                    self.visit_expr(&statement.iter, scope);
                    let body_scope = statement
                        .body
                        .first()
                        .map(statement_range)
                        .map(|range| self.scopes.scope_at(range.start))
                        .unwrap_or(scope);
                    if let Some(binding) = self.scopes.binding_for_declaration(statement.binding_range) {
                        self.push(
                            statement.binding_range,
                            SemanticOccurrenceKind::Binding,
                            OccurrenceRole::Declaration,
                            SemanticTarget::Binding(binding),
                        );
                    }
                    self.visit_statements(&statement.body, body_scope);
                }
                Statement::Throw { expr, .. } => self.visit_expr(expr, scope),
                Statement::Import(import) => {
                    if let Some(binding) = self.scopes.binding_for_declaration(import.binding_range) {
                        self.push(
                            import.binding_range,
                            SemanticOccurrenceKind::Binding,
                            OccurrenceRole::Declaration,
                            SemanticTarget::Binding(binding),
                        );
                    }
                }
                Statement::Break { .. } | Statement::Continue { .. } => {}
            }
        }
    }

    fn visit_member(&mut self, class: &ClassId, member: &ClassMember) {
        let Some(member_surface) = self.member_surface(class, member).cloned() else { return };
        let scope = self.scopes.scope_at(member_surface.name_range.start);
        match member {
            ClassMember::Method(method) => {
                self.push_callable_declaration(&member_surface);
                self.visit_parameters(&method.params, scope);
                self.visit_statements(&method.body, scope);
            }
            ClassMember::Getter(getter) => {
                self.push_callable_declaration(&member_surface);
                self.visit_statements(&getter.body, scope);
            }
            ClassMember::Setter(setter) => {
                self.push_callable_declaration(&member_surface);
                self.visit_parameters(std::slice::from_ref(&setter.param), scope);
                self.visit_statements(&setter.body, scope);
            }
            ClassMember::Index(index) => {
                self.push_callable_declaration(&member_surface);
                self.visit_parameters(&index.params, scope);
                if let IndexAccessor::Set { put } = &index.accessor {
                    self.visit_parameters(std::slice::from_ref(put), scope);
                }
                self.visit_statements(&index.body, scope);
            }
            ClassMember::Field(field) => {
                self.push(
                    field.name_range,
                    SemanticOccurrenceKind::Field,
                    OccurrenceRole::Declaration,
                    SemanticTarget::Field {
                        owner: class.clone(),
                        name: field.name.clone(),
                    },
                );
                if let Some(default) = &field.default {
                    self.visit_expr(default, scope);
                }
            }
            ClassMember::Variant(variant) => {
                self.push(
                    variant.name_range,
                    SemanticOccurrenceKind::Class,
                    OccurrenceRole::Declaration,
                    SemanticTarget::Class(ClassId::new(self.module.clone(), variant.name.clone())),
                );
            }
        }
    }

    fn member_surface(&self, class: &ClassId, member: &ClassMember) -> Option<&MemberSurface> {
        let name_range = match member {
            ClassMember::Method(item) => item.name_range,
            ClassMember::Getter(item) => item.name_range,
            ClassMember::Setter(item) => item.name_range,
            ClassMember::Field(item) => item.name_range,
            ClassMember::Variant(item) => item.name_range,
            ClassMember::Index(item) => item.name_range,
        };
        self.surface
            .classes
            .get(class)?
            .members_by_side
            .values()
            .find(|member| member.name_range == name_range)
    }

    fn push_callable_declaration(&mut self, member: &MemberSurface) {
        self.push(
            member.name_range,
            SemanticOccurrenceKind::Member,
            OccurrenceRole::Declaration,
            SemanticTarget::Callable(member.callable.clone()),
        );
    }

    fn visit_parameters(&mut self, parameters: &[phalcom_ast::ast::ParameterDef], scope: super::scope::ScopeId) {
        for parameter in parameters {
            if let Some(binding) = self.scopes.binding_for_declaration(parameter.name_range) {
                self.push(
                    parameter.name_range,
                    SemanticOccurrenceKind::Parameter,
                    OccurrenceRole::Declaration,
                    SemanticTarget::Binding(binding),
                );
            }
            let _ = scope;
        }
    }

    fn visit_pattern_declarations(&mut self, pattern: &Pattern, _scope: super::scope::ScopeId) {
        match pattern {
            Pattern::Name { range, .. } => {
                if let Some(binding) = self.scopes.binding_for_declaration(*range) {
                    self.push(*range, SemanticOccurrenceKind::Binding, OccurrenceRole::Declaration, SemanticTarget::Binding(binding));
                }
            }
            Pattern::Tuple { elements, .. } => {
                for element in elements {
                    self.visit_pattern_declarations(element, _scope);
                }
            }
            Pattern::List { elements, rest, .. } => {
                for element in elements {
                    self.visit_pattern_declarations(element, _scope);
                }
                if let Some(rest) = rest {
                    self.visit_pattern_declarations(rest, _scope);
                }
            }
        }
    }

    fn visit_expr(&mut self, expr: &Expr, scope: super::scope::ScopeId) {
        match expr {
            Expr::Var { value, range } => {
                if let Some(target) = self.name_target(value, *range, scope) {
                    self.push(*range, target_kind(&target), OccurrenceRole::Read, target);
                }
            }
            Expr::Field { value, range, kind } => {
                if let Some(class) = self.enclosing_class(*range) {
                    self.push(
                        *range,
                        SemanticOccurrenceKind::Field,
                        OccurrenceRole::Read,
                        SemanticTarget::Field {
                            owner: class,
                            name: value.clone(),
                        },
                    );
                } else {
                    let _ = kind;
                }
            }
            Expr::Assignment(assignment) => {
                self.visit_assignment_target(&assignment.name, scope);
                self.visit_expr(&assignment.value, scope);
            }
            Expr::Range(range) => {
                if let Some(lower) = &range.lower {
                    self.visit_expr(lower, scope);
                }
                if let Some(upper) = &range.upper {
                    self.visit_expr(upper, scope);
                }
            }
            Expr::Unary(unary) => {
                if let Some(range) = unary.op_range {
                    self.push(range, SemanticOccurrenceKind::Operator, OccurrenceRole::Reference, SemanticTarget::Operator(unary_name(&unary.op)));
                }
                self.visit_expr(&unary.expr, scope);
            }
            Expr::Binary(binary) => {
                if let Some(range) = binary.op_range {
                    self.push(range, SemanticOccurrenceKind::Operator, OccurrenceRole::Reference, SemanticTarget::Operator(binary_name(&binary.op)));
                }
                self.visit_expr(&binary.left, scope);
                self.visit_expr(&binary.right, scope);
            }
            Expr::UnqualifiedCall(call) => {
                if let Some(range) = call.name_range {
                    let target = match self.scopes.resolve(self.scopes.scope_at(range.start), &call.name, range.start) {
                        super::scope::NameResolution::Binding(binding) => SemanticTarget::Binding(binding),
                        _ => SemanticTarget::Member { name: call.name.clone() },
                    };
                    self.push(range, target_kind(&target), OccurrenceRole::Call, target);
                }
                self.visit_pack_items(&call.args, scope);
            }
            Expr::MethodCall(call) => {
                if let Some(range) = call.method_range {
                    self.push(
                        range,
                        SemanticOccurrenceKind::Member,
                        OccurrenceRole::Call,
                        SemanticTarget::Member { name: call.method.clone() },
                    );
                }
                self.visit_expr(&call.object, scope);
                self.visit_pack_items(&call.args, scope);
            }
            Expr::GetProperty(property) => {
                if let Some(range) = property.property_range {
                    self.push(
                        range,
                        SemanticOccurrenceKind::Member,
                        OccurrenceRole::Read,
                        SemanticTarget::Member { name: property.property.clone() },
                    );
                }
                self.visit_expr(&property.object, scope);
            }
            Expr::SetProperty(property) => {
                if let Some(range) = property.property_range {
                    self.push(
                        range,
                        SemanticOccurrenceKind::Member,
                        OccurrenceRole::Write,
                        SemanticTarget::Member { name: property.property.clone() },
                    );
                }
                self.visit_expr(&property.object, scope);
                self.visit_expr(&property.value, scope);
            }
            Expr::Index(index) => {
                if let Some(range) = index.selector_range {
                    self.push(
                        range,
                        SemanticOccurrenceKind::Member,
                        OccurrenceRole::Read,
                        SemanticTarget::Member { name: "[]".to_string() },
                    );
                }
                self.visit_expr(&index.object, scope);
                self.visit_pack_items(&index.args, scope);
            }
            Expr::SetIndex(index) => {
                if let Some(range) = index.selector_range {
                    self.push(
                        range,
                        SemanticOccurrenceKind::Member,
                        OccurrenceRole::Write,
                        SemanticTarget::Member { name: "[]=".to_string() },
                    );
                }
                self.visit_expr(&index.object, scope);
                self.visit_pack_items(&index.args, scope);
                self.visit_expr(&index.value, scope);
            }
            Expr::Block(block) => {
                let block_scope = self.scopes.scope_at(block.range.start);
                for parameter in &block.params.fixed {
                    if let Some(binding) = self.scopes.binding_for_declaration(parameter.range) {
                        self.push(parameter.range, SemanticOccurrenceKind::Parameter, OccurrenceRole::Declaration, SemanticTarget::Binding(binding));
                    }
                }
                if let Some(parameter) = &block.params.positional_rest
                    && let Some(binding) = self.scopes.binding_for_declaration(parameter.range)
                {
                    self.push(parameter.range, SemanticOccurrenceKind::Parameter, OccurrenceRole::Declaration, SemanticTarget::Binding(binding));
                }
                self.visit_statements(&block.body, block_scope);
            }
            Expr::MethodRef(reference) => {
                if let Some(range) = reference.selector_range {
                    let name = match &reference.kind {
                        MethodRefKind::Open { name } | MethodRefKind::Pinned { name, .. } => name.clone(),
                    };
                    self.push(range, SemanticOccurrenceKind::Member, OccurrenceRole::Reference, SemanticTarget::Member { name });
                }
                self.visit_expr(&reference.receiver, scope);
            }
            Expr::SetLiteral(set) => {
                for entry in &set.entries {
                    match entry {
                        phalcom_ast::ast::SetLiteralEntry::Element { expr, .. }
                        | phalcom_ast::ast::SetLiteralEntry::Expansion { expr, .. } => self.visit_expr(expr, scope),
                    }
                }
            }
            Expr::ListLiteral(list) => {
                for element in &list.elements {
                    match element {
                        phalcom_ast::ast::ListLiteralElement::Element { expr, .. }
                        | phalcom_ast::ast::ListLiteralElement::Expansion { expr, .. } => self.visit_expr(expr, scope),
                    }
                }
            }
            Expr::TupleLiteral(tuple) => {
                for entry in &tuple.entries {
                    match entry {
                        phalcom_ast::ast::TupleLiteralEntry::Positional { expr, .. }
                        | phalcom_ast::ast::TupleLiteralEntry::Expand { expr, .. } => self.visit_expr(expr, scope),
                        phalcom_ast::ast::TupleLiteralEntry::Labeled { value, .. } => self.visit_expr(value, scope),
                    }
                }
            }
            Expr::RecordLiteral(record) => {
                for entry in &record.entries {
                    match entry {
                        phalcom_ast::ast::RecordLiteralEntry::Field(field) => self.visit_expr(&field.value, scope),
                        phalcom_ast::ast::RecordLiteralEntry::Expansion { expr, .. } => self.visit_expr(expr, scope),
                    }
                }
            }
            Expr::MapLiteral(map) => {
                for entry in &map.entries {
                    match entry {
                        phalcom_ast::ast::MapLiteralEntry::Association { key, value, .. } => {
                            if let phalcom_ast::ast::MapLiteralKey::Computed { expr, .. } = key {
                                self.visit_expr(expr, scope);
                            }
                            self.visit_expr(value, scope);
                        }
                        phalcom_ast::ast::MapLiteralEntry::Expansion { expr, .. } => self.visit_expr(expr, scope),
                    }
                }
            }
            Expr::Int { .. }
            | Expr::Float { .. }
            | Expr::String { .. }
            | Expr::Boolean { .. }
            | Expr::SelfVar { .. }
            | Expr::SuperVar { .. }
            | Expr::ImplementationSelector { .. }
            | Expr::Symbol { .. } => {}
        }
    }

    fn visit_assignment_target(&mut self, expr: &Expr, scope: super::scope::ScopeId) {
        if let Expr::Var { value, range } = expr {
            if let Some(target) = self.name_target(value, *range, scope) {
                self.push(*range, target_kind(&target), OccurrenceRole::Write, target);
            }
        } else {
            self.visit_expr(expr, scope);
        }
    }

    fn name_target(&self, name: &str, range: SourceRange, scope: super::scope::ScopeId) -> Option<SemanticTarget> {
        match self.scopes.resolve(scope, name, range.start) {
            super::scope::NameResolution::Binding(binding) => Some(SemanticTarget::Binding(binding)),
            super::scope::NameResolution::Class(class) => Some(SemanticTarget::Class(class)),
            super::scope::NameResolution::Global(_) | super::scope::NameResolution::Module(_) | super::scope::NameResolution::ImplicitSelf | super::scope::NameResolution::Unresolved => None,
        }
    }

    fn enclosing_class(&self, range: SourceRange) -> Option<ClassId> {
        self.surface
            .classes
            .values()
            .find(|class| class.source_range.contains(range.start))
            .map(|class| class.id.clone())
    }

    fn visit_pack_items(&mut self, items: &[phalcom_ast::ast::PackItem], scope: super::scope::ScopeId) {
        for item in items {
            match item {
                phalcom_ast::ast::PackItem::Positional { expr, .. }
                | phalcom_ast::ast::PackItem::Expand { expr, .. }
                | phalcom_ast::ast::PackItem::Labeled { value: expr, .. } => self.visit_expr(expr, scope),
            }
        }
    }
}

fn target_kind(target: &SemanticTarget) -> SemanticOccurrenceKind {
    match target {
        SemanticTarget::Binding(_) => SemanticOccurrenceKind::Binding,
        SemanticTarget::Class(_) => SemanticOccurrenceKind::Class,
        SemanticTarget::Callable(_) | SemanticTarget::Member { .. } => SemanticOccurrenceKind::Member,
        SemanticTarget::Field { .. } => SemanticOccurrenceKind::Field,
        SemanticTarget::Operator(_) => SemanticOccurrenceKind::Operator,
    }
}

fn occurrence_priority(kind: SemanticOccurrenceKind) -> u8 {
    match kind {
        SemanticOccurrenceKind::Binding | SemanticOccurrenceKind::Parameter => 0,
        SemanticOccurrenceKind::Member => 1,
        SemanticOccurrenceKind::Class => 2,
        SemanticOccurrenceKind::Field => 1,
        SemanticOccurrenceKind::Operator => 3,
    }
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

fn binary_name(op: &BinaryOp) -> String {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::IntegerDivide => "~/",
        BinaryOp::Power => "**",
        BinaryOp::Modulo => "%",
        BinaryOp::ShiftLeft => "<<",
        BinaryOp::ShiftRight => ">>",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitXor => "^",
        BinaryOp::BitOr => "|",
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::LessThan => "<",
        BinaryOp::LessThanOrEqual => "<=",
        BinaryOp::GreaterThan => ">",
        BinaryOp::GreaterThanOrEqual => ">=",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
    }
    .to_string()
}

fn unary_name(op: &UnaryOp) -> String {
    match op {
        UnaryOp::Negate => "-",
        UnaryOp::Not => "not",
        UnaryOp::BitNot => "~",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::surface::build_module_surface;
    use phalcom_ast::parser::parse;

    #[test]
    fn occurrence_index_resolves_parameter_declaration_and_use_to_same_binding() {
        let source = "class Sample {\n  method(value) { value }\n}\n";
        let parsed = parse(source, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        let module = ModuleId::new("file:///sample.ph");
        let surface = build_module_surface(module.clone(), &parsed.program);
        let scopes = super::super::scope::build_scope_graph(module.clone(), &parsed.program);
        let index = build_occurrence_index(module, &parsed.program, &surface, &scopes);
        let parameter_offset = source.find("value").expect("parameter");
        let use_offset = source.rfind("value }").expect("use") + 1;
        let declaration = index.occurrence_at(parameter_offset).expect("parameter occurrence");
        let usage = index.occurrence_at(use_offset).expect("usage occurrence");
        assert!(matches!(declaration.kind, SemanticOccurrenceKind::Parameter));
        assert_eq!(declaration.target, usage.target);
        assert_eq!(&source[declaration.range.start..declaration.range.end], "value");
        assert_eq!(&source[usage.range.start..usage.range.end], "value");
    }

    #[test]
    fn keyword_and_literal_offsets_have_no_occurrence() {
        let source = "let value = 1\n";
        let parsed = parse(source, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        let module = ModuleId::new("file:///literal.ph");
        let surface = build_module_surface(module.clone(), &parsed.program);
        let scopes = super::super::scope::build_scope_graph(module.clone(), &parsed.program);
        let index = build_occurrence_index(module, &parsed.program, &surface, &scopes);
        assert!(index.occurrence_at(source.find("let").unwrap()).is_none());
        assert!(index.occurrence_at(source.find('1').unwrap()).is_none());
    }
}
