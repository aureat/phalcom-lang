//! Deterministic syntax and local-flow inference.

use std::collections::BTreeMap;

use super::CallableSummary;
use super::facts::{FieldFacts, InferredValue, LocalFacts, ParameterFacts, ValueShape};
use super::ids::{CORE_MODULE_URI, CallableId, ClassId, ModuleId};
use super::query::SemanticGeneration;
use super::surface::ModuleSurface;
use phalcom_ast::ast::{
    Expr, ListLiteralElement, MapLiteralEntry, MapLiteralKey, PackItem, PackLabel, Pattern, ProductLabel, RecordLiteralEntry, SetLiteralEntry, Statement,
    SymbolLiteralKind, TupleLiteralEntry,
};

/// Infers one expression using an existing local environment.
pub fn infer_expr(
    expr: &Expr,
    module: &ModuleId,
    current_class: Option<&ClassId>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
) -> InferredValue {
    let range = expr.range();
    let shape = match expr {
        Expr::Int { .. } => ValueShape::Instance(core_class("Int")),
        Expr::Float { .. } => ValueShape::Instance(core_class("Float")),
        Expr::String { .. } => ValueShape::Instance(core_class("String")),
        Expr::Boolean { .. } => ValueShape::Instance(core_class("Bool")),
        Expr::Symbol { .. } => ValueShape::Instance(core_class("Symbol")),
        Expr::Var { value, .. } => environment
            .get(value)
            .map(|value| value.shape.clone())
            .or_else(|| known_class(value).map(ValueShape::ClassObject))
            .unwrap_or(ValueShape::Unknown),
        Expr::SelfVar { .. } => current_class.map(|class| ValueShape::Instance(class.clone())).unwrap_or(ValueShape::Unknown),
        Expr::SuperVar { .. } => ValueShape::Unknown,
        Expr::TupleLiteral(tuple) => ValueShape::Tuple(
            tuple
                .entries
                .iter()
                .map(|entry| match entry {
                    TupleLiteralEntry::Positional { expr, .. } | TupleLiteralEntry::Labeled { value: expr, .. } | TupleLiteralEntry::Expand { expr, .. } => {
                        infer_expr(expr, module, current_class, environment, known_class, is_constructor).shape
                    }
                })
                .collect(),
        ),
        Expr::RecordLiteral(record) => ValueShape::Record(
            record
                .entries
                .iter()
                .filter_map(|entry| match entry {
                    RecordLiteralEntry::Field(field) => Some((
                        product_label(&field.label),
                        infer_expr(&field.value, module, current_class, environment, known_class, is_constructor).shape,
                    )),
                    RecordLiteralEntry::Expansion { .. } => None,
                })
                .collect(),
        ),
        Expr::ListLiteral(list) => ValueShape::List(Box::new(ValueShape::bounded_union(list.elements.iter().map(|element| match element {
            ListLiteralElement::Element { expr, .. } | ListLiteralElement::Expansion { expr, .. } => {
                infer_expr(expr, module, current_class, environment, known_class, is_constructor).shape
            }
        })))),
        Expr::SetLiteral(set) => ValueShape::Set(Box::new(ValueShape::bounded_union(set.entries.iter().map(|entry| match entry {
            SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => {
                infer_expr(expr, module, current_class, environment, known_class, is_constructor).shape
            }
        })))),
        Expr::MapLiteral(map) => {
            let mut keys = Vec::new();
            let mut values = Vec::new();
            for entry in &map.entries {
                if let MapLiteralEntry::Association { key, value, .. } = entry {
                    keys.push(match key {
                        MapLiteralKey::BareSymbol { .. } => ValueShape::Instance(core_class("Symbol")),
                        MapLiteralKey::Computed { expr, .. } => infer_expr(expr, module, current_class, environment, known_class, is_constructor).shape,
                    });
                    values.push(infer_expr(value, module, current_class, environment, known_class, is_constructor).shape);
                }
            }
            ValueShape::Map {
                key: Box::new(ValueShape::bounded_union(keys)),
                value: Box::new(ValueShape::bounded_union(values)),
            }
        }
        Expr::Range(range) => {
            let bounds = range
                .lower
                .iter()
                .chain(range.upper.iter())
                .map(|bound| infer_expr(bound, module, current_class, environment, known_class, is_constructor).shape);
            ValueShape::Range(Box::new(ValueShape::bounded_union(bounds)))
        }
        Expr::Assignment(assignment) => infer_expr(&assignment.value, module, current_class, environment, known_class, is_constructor).shape,
        Expr::MethodCall(call) => {
            let receiver = infer_expr(&call.object, module, current_class, environment, known_class, is_constructor);
            let selector = call_selector(&call.method, &call.args);
            match receiver.shape {
                ValueShape::ClassObject(class) if is_constructor(&class, &selector) => ValueShape::Instance(class),
                _ => ValueShape::Unknown,
            }
        }
        _ => ValueShape::Unknown,
    };
    InferredValue::exact(shape, range)
}

/// Infers a source expression while consulting known callable return summaries.
pub fn infer_expr_with_returns(
    expr: &Expr,
    module: &ModuleId,
    current_class: Option<&ClassId>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&ClassId, &str) -> Option<InferredValue> + Copy,
) -> InferredValue {
    if let Expr::MethodCall(call) = expr {
        let receiver = infer_expr_with_returns(&call.object, module, current_class, environment, known_class, is_constructor, callable_return);
        let selector = call_selector(&call.method, &call.args);
        let shape = match receiver.shape {
            ValueShape::ClassObject(class) if is_constructor(&class, &selector) => ValueShape::Instance(class),
            ValueShape::Instance(class) => callable_return(&class, &selector).map(|value| value.shape).unwrap_or(ValueShape::Unknown),
            _ => ValueShape::Unknown,
        };
        return InferredValue::flow(shape, expr.range());
    }
    if let Expr::UnqualifiedCall(call) = expr {
        let shape = current_class
            .and_then(|class| callable_return(class, &call_selector(&call.name, &call.args)))
            .map(|value| value.shape)
            .unwrap_or(ValueShape::Unknown);
        return InferredValue::flow(shape, expr.range());
    }
    infer_expr(expr, module, current_class, environment, known_class, is_constructor)
}

/// Infers expressions while consulting callable summaries and constructor-assigned fields.
pub fn infer_expr_with_fields(
    expr: &Expr,
    module: &ModuleId,
    current_class: Option<&ClassId>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&ClassId, &str) -> Option<InferredValue> + Copy,
    field_value: impl Fn(&ClassId, &str) -> Option<InferredValue> + Copy,
) -> InferredValue {
    match expr {
        Expr::Field { value, .. } => current_class
            .and_then(|class| field_value(class, value))
            .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, expr.range())),
        Expr::MethodCall(call) => {
            let receiver = infer_expr_with_fields(
                &call.object,
                module,
                current_class,
                environment,
                known_class,
                is_constructor,
                callable_return,
                field_value,
            );
            let selector = call_selector(&call.method, &call.args);
            let shape = match receiver.shape {
                ValueShape::ClassObject(class) if is_constructor(&class, &selector) => ValueShape::Instance(class),
                ValueShape::Instance(class) => callable_return(&class, &selector).map(|value| value.shape).unwrap_or(ValueShape::Unknown),
                _ => ValueShape::Unknown,
            };
            InferredValue::flow(shape, expr.range())
        }
        _ => infer_expr_with_returns(expr, module, current_class, environment, known_class, is_constructor, callable_return),
    }
}

/// Collects constructor-assigned field facts from one module's source surface.
pub fn field_facts_for_surface(
    surface: &ModuleSurface,
    module: &ModuleId,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&ClassId, &str) -> Option<InferredValue> + Copy,
) -> FieldFacts {
    let mut facts = FieldFacts::default();
    for class in surface.classes.values() {
        for field in class.fields.values() {
            if let Some(initializer) = &field.initializer {
                let value = infer_expr_with_returns(
                    initializer,
                    module,
                    Some(&class.id),
                    &BTreeMap::new(),
                    known_class,
                    is_constructor,
                    callable_return,
                );
                facts.record(class.id.clone(), field.name.clone(), value);
            }
        }
        for member in class.members.values() {
            for statement in &member.body {
                if let Statement::Expr {
                    expr: Expr::Assignment(assignment),
                    ..
                } = statement
                {
                    let Expr::Field { value: name, range, .. } = assignment.name.as_ref() else {
                        continue;
                    };
                    let inferred = infer_expr_with_returns(
                        &assignment.value,
                        module,
                        Some(&class.id),
                        &BTreeMap::new(),
                        known_class,
                        is_constructor,
                        callable_return,
                    );
                    facts.record(class.id.clone(), name.clone(), InferredValue::flow(inferred.shape, *range));
                }
            }
        }
    }
    facts
}

/// Collects parameter facts from unambiguous call sites in one source module.
pub fn parameter_facts_for_program(
    program: &phalcom_ast::ast::Program,
    surface: &ModuleSurface,
    module: &ModuleId,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&ClassId, &str) -> Option<InferredValue> + Copy,
) -> ParameterFacts {
    let mut facts = ParameterFacts::default();
    let mut environment = BTreeMap::new();
    collect_call_sites(
        &program.statements,
        surface,
        module,
        None,
        known_class,
        is_constructor,
        callable_return,
        &mut environment,
        &mut facts,
    );
    facts
}

fn collect_call_sites(
    statements: &[Statement],
    surface: &ModuleSurface,
    module: &ModuleId,
    current_class: Option<&ClassId>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&ClassId, &str) -> Option<InferredValue> + Copy,
    environment: &mut BTreeMap<String, InferredValue>,
    facts: &mut ParameterFacts,
) {
    for statement in statements {
        match statement {
            Statement::Let(binding) => {
                if let Some(value) = &binding.value {
                    collect_call_sites_expr(
                        value,
                        surface,
                        module,
                        current_class,
                        known_class,
                        is_constructor,
                        callable_return,
                        environment,
                        facts,
                    );
                    if let Pattern::Name { name, .. } = &binding.pattern {
                        environment.insert(
                            name.clone(),
                            infer_expr_with_returns(value, module, current_class, environment, known_class, is_constructor, callable_return),
                        );
                    }
                }
            }
            Statement::Expr { expr, .. } => {
                collect_call_sites_expr(
                    expr,
                    surface,
                    module,
                    current_class,
                    known_class,
                    is_constructor,
                    callable_return,
                    environment,
                    facts,
                );
                if let Expr::Assignment(assignment) = expr {
                    if let Expr::Var { value: name, .. } = assignment.name.as_ref() {
                        let inferred = infer_expr_with_returns(
                            &assignment.value,
                            module,
                            current_class,
                            environment,
                            known_class,
                            is_constructor,
                            callable_return,
                        );
                        environment.insert(name.clone(), inferred);
                    }
                }
            }
            Statement::Return(return_statement) => {
                if let Some(value) = &return_statement.value {
                    collect_call_sites_expr(
                        value,
                        surface,
                        module,
                        current_class,
                        known_class,
                        is_constructor,
                        callable_return,
                        environment,
                        facts,
                    );
                }
            }
            Statement::Class(class) => {
                let id = ClassId::new(module.clone(), class.name.clone());
                for member in &class.members {
                    let body = match member {
                        phalcom_ast::ast::ClassMember::Method(item) => &item.body,
                        phalcom_ast::ast::ClassMember::Getter(item) => &item.body,
                        phalcom_ast::ast::ClassMember::Setter(item) => &item.body,
                        phalcom_ast::ast::ClassMember::Index(item) => &item.body,
                        phalcom_ast::ast::ClassMember::Field(_) | phalcom_ast::ast::ClassMember::Variant(_) => continue,
                    };
                    let mut member_environment = BTreeMap::new();
                    collect_call_sites(
                        body,
                        surface,
                        module,
                        Some(&id),
                        known_class,
                        is_constructor,
                        callable_return,
                        &mut member_environment,
                        facts,
                    );
                }
            }
            _ => {}
        }
    }
}

fn collect_call_sites_expr(
    expr: &Expr,
    surface: &ModuleSurface,
    module: &ModuleId,
    current_class: Option<&ClassId>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&ClassId, &str) -> Option<InferredValue> + Copy,
    environment: &BTreeMap<String, InferredValue>,
    facts: &mut ParameterFacts,
) {
    match expr {
        Expr::UnqualifiedCall(call) => {
            let selector = call_selector(&call.name, &call.args);
            let target = current_class
                .and_then(|class| surface.classes.get(class))
                .and_then(|class| class.members.get(&selector))
                .map(|member| (member.callable.clone(), member.params.as_slice()))
                .or_else(|| {
                    let mut matches = surface
                        .classes
                        .values()
                        .filter_map(|class| class.members.get(&selector).map(|member| (member.callable.clone(), member.params.as_slice())));
                    let first = matches.next()?;
                    matches.next().is_none().then_some(first)
                });
            if let Some((callable, params)) = target {
                record_call_arguments(
                    &callable,
                    &call.args,
                    params,
                    module,
                    current_class,
                    environment,
                    known_class,
                    is_constructor,
                    callable_return,
                    call.range,
                    facts,
                );
            }
            for arg in &call.args {
                collect_call_sites_pack(
                    arg,
                    surface,
                    module,
                    current_class,
                    known_class,
                    is_constructor,
                    callable_return,
                    environment,
                    facts,
                );
            }
        }
        Expr::MethodCall(call) => {
            let receiver = infer_expr_with_returns(&call.object, module, current_class, environment, known_class, is_constructor, callable_return);
            let selector = call_selector(&call.method, &call.args);
            let target_class = match receiver.shape {
                ValueShape::Instance(class) | ValueShape::ClassObject(class) => Some(class),
                _ => None,
            };
            if let Some(class) = target_class.and_then(|id| surface.classes.get(&id)) {
                if let Some(member) = class.members.get(&selector) {
                    record_call_arguments(
                        &member.callable,
                        &call.args,
                        &member.params,
                        module,
                        current_class,
                        environment,
                        known_class,
                        is_constructor,
                        callable_return,
                        call.range,
                        facts,
                    );
                }
            }
            collect_call_sites_expr(
                &call.object,
                surface,
                module,
                current_class,
                known_class,
                is_constructor,
                callable_return,
                environment,
                facts,
            );
            for arg in &call.args {
                collect_call_sites_pack(
                    arg,
                    surface,
                    module,
                    current_class,
                    known_class,
                    is_constructor,
                    callable_return,
                    environment,
                    facts,
                );
            }
        }
        Expr::Assignment(assignment) => {
            collect_call_sites_expr(
                &assignment.name,
                surface,
                module,
                current_class,
                known_class,
                is_constructor,
                callable_return,
                environment,
                facts,
            );
            collect_call_sites_expr(
                &assignment.value,
                surface,
                module,
                current_class,
                known_class,
                is_constructor,
                callable_return,
                environment,
                facts,
            );
        }
        Expr::GetProperty(property) => collect_call_sites_expr(
            &property.object,
            surface,
            module,
            current_class,
            known_class,
            is_constructor,
            callable_return,
            environment,
            facts,
        ),
        Expr::TupleLiteral(tuple) => {
            for entry in &tuple.entries {
                let value = match entry {
                    TupleLiteralEntry::Positional { expr, .. } | TupleLiteralEntry::Labeled { value: expr, .. } | TupleLiteralEntry::Expand { expr, .. } => {
                        expr
                    }
                };
                collect_call_sites_expr(
                    value,
                    surface,
                    module,
                    current_class,
                    known_class,
                    is_constructor,
                    callable_return,
                    environment,
                    facts,
                );
            }
        }
        _ => {}
    }
}

fn collect_call_sites_pack(
    item: &PackItem,
    surface: &ModuleSurface,
    module: &ModuleId,
    current_class: Option<&ClassId>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&ClassId, &str) -> Option<InferredValue> + Copy,
    environment: &BTreeMap<String, InferredValue>,
    facts: &mut ParameterFacts,
) {
    let expr = match item {
        PackItem::Positional { expr, .. } | PackItem::Expand { expr, .. } | PackItem::Labeled { value: expr, .. } => expr,
    };
    collect_call_sites_expr(
        expr,
        surface,
        module,
        current_class,
        known_class,
        is_constructor,
        callable_return,
        environment,
        facts,
    );
}

fn record_call_arguments(
    callable: &CallableId,
    args: &[PackItem],
    params: &[super::surface::ParamSurface],
    module: &ModuleId,
    current_class: Option<&ClassId>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&ClassId, &str) -> Option<InferredValue> + Copy,
    call_range: phalcom_common::range::SourceRange,
    facts: &mut ParameterFacts,
) {
    let mut positional = 0;
    for arg in args {
        let (label, expr) = match arg {
            PackItem::Positional { expr, .. } | PackItem::Expand { expr, .. } => (None, expr),
            PackItem::Labeled { label, value, .. } => (
                match label {
                    PackLabel::Static { text, .. } => Some(text.as_str()),
                    PackLabel::Computed { .. } => None,
                },
                value,
            ),
        };
        let param = label
            .and_then(|label| params.iter().find(|param| param.label.as_deref() == Some(label) || param.name == label))
            .or_else(|| {
                let value = params.get(positional);
                positional += 1;
                value
            });
        let Some(param) = param else { continue };
        let inferred = infer_expr_with_returns(expr, module, current_class, environment, known_class, is_constructor, callable_return);
        if !matches!(inferred.shape, ValueShape::Unknown) {
            facts.record(callable.clone(), param.name.clone(), InferredValue::interprocedural(inferred.shape, call_range));
        }
    }
}

/// Collects local facts with source-callable summaries enabled.
pub fn collect_local_facts_with_returns(
    program: &phalcom_ast::ast::Program,
    module: &ModuleId,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&ClassId, &str) -> Option<InferredValue> + Copy,
) -> LocalFacts {
    let mut facts = LocalFacts::default();
    let mut environment = BTreeMap::new();
    collect_statements_with_returns(
        &program.statements,
        module,
        None,
        known_class,
        is_constructor,
        callable_return,
        &mut environment,
        &mut facts,
    );
    facts
}

fn collect_statements_with_returns(
    statements: &[Statement],
    module: &ModuleId,
    current_class: Option<&ClassId>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&ClassId, &str) -> Option<InferredValue> + Copy,
    environment: &mut BTreeMap<String, InferredValue>,
    facts: &mut LocalFacts,
) {
    for statement in statements {
        match statement {
            Statement::Let(binding) => {
                let value = binding
                    .value
                    .as_ref()
                    .map(|expr| infer_expr_with_returns(expr, module, current_class, environment, known_class, is_constructor, callable_return))
                    .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, binding.range));
                bind_pattern(&binding.pattern, &value, module, current_class, known_class, environment, facts);
            }
            Statement::Expr {
                expr: Expr::Assignment(assignment),
                ..
            } => {
                let value = infer_expr_with_returns(
                    &assignment.value,
                    module,
                    current_class,
                    environment,
                    known_class,
                    is_constructor,
                    callable_return,
                );
                if let Expr::Var { value: name, range } = assignment.name.as_ref() {
                    let value = InferredValue::flow(value.shape.clone(), *range);
                    facts.record(name.clone(), *range, value.clone());
                    environment.insert(name.clone(), value);
                }
            }
            Statement::Class(class) => {
                let id = ClassId::new(module.clone(), class.name.clone());
                for member in &class.members {
                    let body = match member {
                        phalcom_ast::ast::ClassMember::Method(item) => &item.body,
                        phalcom_ast::ast::ClassMember::Getter(item) => &item.body,
                        phalcom_ast::ast::ClassMember::Setter(item) => &item.body,
                        phalcom_ast::ast::ClassMember::Index(item) => &item.body,
                        phalcom_ast::ast::ClassMember::Field(_) | phalcom_ast::ast::ClassMember::Variant(_) => continue,
                    };
                    let mut member_environment = BTreeMap::new();
                    collect_statements_with_returns(
                        body,
                        module,
                        Some(&id),
                        known_class,
                        is_constructor,
                        callable_return,
                        &mut member_environment,
                        facts,
                    );
                }
            }
            _ => {}
        }
    }
}

/// Computes one-pass callable return summaries for a source module surface.
pub fn summaries_for_surface(
    surface: &ModuleSurface,
    module: &ModuleId,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&ClassId, &str) -> Option<InferredValue> + Copy,
    parameter_fact: impl Fn(&CallableId, &str) -> Option<InferredValue> + Copy,
    revision: SemanticGeneration,
) -> Vec<CallableSummary> {
    surface
        .classes
        .values()
        .flat_map(|class| class.members.values().map(move |member| (class, member)))
        .filter(|(_, member)| !member.body.is_empty())
        .map(|(class, member)| {
            let mut environment = BTreeMap::new();
            let params = member
                .params
                .iter()
                .map(|param| {
                    let value = parameter_fact(&member.callable, &param.name).unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, param.source_range));
                    environment.insert(param.name.clone(), value.clone());
                    value
                })
                .collect();
            let returns = body_value(
                &member.body,
                module,
                Some(&class.id),
                &environment,
                known_class,
                is_constructor,
                callable_return,
            );
            CallableSummary {
                callable: member.callable.clone(),
                params,
                returns,
                dependencies: Vec::new(),
                effects: Default::default(),
                revision,
            }
        })
        .collect()
}

fn body_value(
    body: &[Statement],
    module: &ModuleId,
    current_class: Option<&ClassId>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&ClassId, &str) -> Option<InferredValue> + Copy,
) -> InferredValue {
    let Some(last) = body.last() else {
        return InferredValue::flow(ValueShape::Unknown, Default::default());
    };
    match last {
        Statement::Expr { expr, .. } => infer_expr_with_returns(expr, module, current_class, environment, known_class, is_constructor, callable_return),
        Statement::Let(binding) => binding
            .value
            .as_ref()
            .map(|expr| infer_expr_with_returns(expr, module, current_class, environment, known_class, is_constructor, callable_return))
            .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, binding.range)),
        Statement::Return(return_statement) => return_statement
            .value
            .as_ref()
            .map(|expr| infer_expr_with_returns(expr, module, current_class, environment, known_class, is_constructor, callable_return))
            .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, return_statement.range)),
        _ => InferredValue::flow(ValueShape::Unknown, last_range(last)),
    }
}

fn last_range(statement: &Statement) -> phalcom_common::range::SourceRange {
    match statement {
        Statement::Class(class) => class.range,
        Statement::Let(binding) => binding.range,
        Statement::Return(return_statement) => return_statement.range,
        Statement::Expr { range, .. } => *range,
        Statement::For(statement) => statement.range,
        Statement::Break { range } | Statement::Continue { range } => *range,
        Statement::Throw { range, .. } | Statement::Import(phalcom_ast::ast::ImportStatement { range, .. }) => *range,
    }
}

fn bind_pattern(
    pattern: &Pattern,
    value: &InferredValue,
    module: &ModuleId,
    current_class: Option<&ClassId>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    environment: &mut BTreeMap<String, InferredValue>,
    facts: &mut LocalFacts,
) {
    match pattern {
        Pattern::Name { name, range } => {
            let fact = InferredValue::flow(value.shape.clone(), *range);
            facts.record(name.clone(), *range, fact.clone());
            environment.insert(name.clone(), fact);
        }
        Pattern::Tuple { elements, .. } | Pattern::List { elements, .. } => {
            for (index, element) in elements.iter().enumerate() {
                let projected = match &value.shape {
                    ValueShape::Tuple(values) => values.get(index).cloned().unwrap_or(ValueShape::Unknown),
                    ValueShape::List(element) => (**element).clone(),
                    _ => ValueShape::Unknown,
                };
                bind_pattern(
                    element,
                    &InferredValue::flow(projected, element.range()),
                    module,
                    current_class,
                    known_class,
                    environment,
                    facts,
                );
            }
            if let Pattern::List { rest: Some(rest), .. } = pattern {
                bind_pattern(
                    rest,
                    &InferredValue::flow(ValueShape::List(Box::new(ValueShape::Unknown)), rest.range()),
                    module,
                    current_class,
                    known_class,
                    environment,
                    facts,
                );
            }
        }
    }
}

fn core_class(name: &str) -> ClassId {
    ClassId::new(ModuleId::new(CORE_MODULE_URI), name)
}

fn call_selector(method: &str, args: &[PackItem]) -> String {
    let labels = args
        .iter()
        .map(|arg| match arg {
            PackItem::Positional { .. } | PackItem::Expand { .. } => None,
            PackItem::Labeled {
                label: PackLabel::Static { text, .. },
                ..
            } => Some(text.clone()),
            PackItem::Labeled { .. } => None,
        })
        .collect::<Vec<_>>();
    crate::selectors::comma_form_from_labels(method, &labels)
}

fn product_label(label: &ProductLabel) -> String {
    match label {
        ProductLabel::Static {
            symbol: SymbolLiteralKind::Name(name),
            ..
        } => name.clone(),
        ProductLabel::Static {
            symbol: SymbolLiteralKind::Selector { name, .. },
            ..
        } => name.clone(),
        ProductLabel::Computed { .. } => "?".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::parser::parse;

    fn no_classes(_: &str) -> Option<ClassId> {
        None
    }

    #[test]
    fn literals_and_reassignment_are_queryable() {
        let program = parse("let value = 1\nvalue = \"ok\"\n", 0).program;
        let module = ModuleId::new("file:///main.ph");
        let facts = collect_local_facts_with_returns(&program, &module, no_classes, |_, _| false, |_, _| None);
        assert_eq!(facts.binding_at("value", 10).unwrap().shape, ValueShape::Instance(core_class("Int")));
        assert_eq!(facts.binding_at("value", 30).unwrap().shape, ValueShape::Instance(core_class("String")));
    }

    #[test]
    fn union_widens_after_nine_distinct_shapes() {
        let module = ModuleId::new("file:///main.ph");
        let mut shape = ValueShape::Instance(ClassId::new(module.clone(), "C0"));
        for index in 1..=7 {
            shape = shape.join(&ValueShape::Instance(ClassId::new(module.clone(), format!("C{index}"))));
        }
        assert!(matches!(shape, ValueShape::Union(_)));
        shape = shape.join(&ValueShape::Instance(ClassId::new(module, "C8")));
        assert_eq!(shape, ValueShape::Unknown);
    }
}
