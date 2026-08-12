//! Deterministic syntax and local-flow inference.

#![allow(clippy::only_used_in_recursion, clippy::too_many_arguments)]

use std::collections::BTreeMap;
use std::sync::Arc;

use super::CallableSummary;
use super::callable::SolverResult;
use super::facts::{FieldFacts, InferredValue, LocalFacts, MAX_SHAPE_UNION, ParameterFacts, ValueShape};
use super::ids::{CORE_MODULE_URI, CallableId, ClassId, ModuleId};
use super::module_graph::ModuleGraph;
use super::query::SemanticGeneration;
use super::surface::{MemberSurface, ModuleSurface};
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
        Expr::GetProperty(property) => {
            let Expr::Var { value: binding, .. } = &property.object else {
                return InferredValue::exact(ValueShape::Unknown, range);
            };
            known_class(&format!("{binding}.{}", property.property))
                .map(ValueShape::ClassObject)
                .unwrap_or(ValueShape::Unknown)
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
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
) -> InferredValue {
    if let Expr::MethodCall(call) = expr {
        let receiver = infer_expr_with_returns(&call.object, module, current_class, environment, known_class, is_constructor, callable_return);
        let selector = call_selector(&call.method, &call.args);
        let shape = match receiver.shape {
            ValueShape::ClassObject(class) if is_constructor(&class, &selector) => ValueShape::Instance(class),
            ValueShape::Instance(class) => callable_return(&CallableId {
                owner: class,
                selector: selector.clone(),
                side: super::DispatchSide::Instance,
            })
            .map(|value| value.shape)
            .unwrap_or(ValueShape::Unknown),
            ValueShape::ClassObject(class) => callable_return(&CallableId {
                owner: class,
                selector: selector.clone(),
                side: super::DispatchSide::Class,
            })
            .map(|value| value.shape)
            .unwrap_or(ValueShape::Unknown),
            ValueShape::Union(shapes) => ValueShape::bounded_union(shapes.into_iter().filter_map(|shape| {
                match shape {
                    ValueShape::Instance(class) => callable_return(&CallableId {
                        owner: class,
                        selector: selector.clone(),
                        side: super::DispatchSide::Instance,
                    })
                    .map(|value| value.shape),
                    ValueShape::ClassObject(class) => callable_return(&CallableId {
                        owner: class,
                        selector: selector.clone(),
                        side: super::DispatchSide::Class,
                    })
                    .map(|value| value.shape),
                    _ => None,
                }
            })),
            _ => ValueShape::Unknown,
        };
        return InferredValue::flow(shape, expr.range());
    }
    if let Expr::UnqualifiedCall(call) = expr {
        let shape = current_class
            .and_then(|class| {
                callable_return(&CallableId {
                    owner: class.clone(),
                    selector: call_selector(&call.name, &call.args),
                    side: super::DispatchSide::Instance,
                })
            })
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
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
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
                ValueShape::Instance(class) => callable_return(&CallableId {
                    owner: class,
                    selector,
                    side: super::DispatchSide::Instance,
                })
                .map(|value| value.shape)
                .unwrap_or(ValueShape::Unknown),
                ValueShape::ClassObject(class) => callable_return(&CallableId {
                    owner: class,
                    selector,
                    side: super::DispatchSide::Class,
                })
                .map(|value| value.shape)
                .unwrap_or(ValueShape::Unknown),
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
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
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
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    parameter_fact: impl Fn(&CallableId, &str) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&ClassId, &str) -> Option<MemberSurface> + Copy,
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
        parameter_fact,
        resolve_member,
        &mut environment,
        &mut facts,
    );
    facts
}

/// Solves source callable summaries and parameter facts without mutating the
/// published semantic database.
#[expect(dead_code, reason = "retained as a full-workspace solver reference for regression comparisons")]
pub(crate) fn solve_workspace_callables(
    inputs: &[(ModuleId, Arc<phalcom_ast::ast::Program>, ModuleSurface)],
    classes: &BTreeMap<ClassId, super::surface::ClassSurface>,
    graph: &ModuleGraph,
    generation: SemanticGeneration,
) -> SolverResult {
    let callable_count = inputs
        .iter()
        .map(|(_, _, surface)| surface.classes.values().map(|class| class.members.len()).sum::<usize>())
        .sum::<usize>();
    let slot_count = inputs
        .iter()
        .map(|(_, _, surface)| {
            surface
                .classes
                .values()
                .flat_map(|class| class.members.values())
                .map(|member| member.params.len())
                .sum::<usize>()
        })
        .sum::<usize>();
    let max_rounds = (callable_count + slot_count).max(1) * (MAX_SHAPE_UNION + 2);
    let mut summaries = BTreeMap::new();
    let mut parameter_facts = ParameterFacts::default();

    for _ in 0..max_rounds {
        let previous_summaries = summaries.clone();
        let previous_parameters = parameter_facts.clone();
        let mut next_parameters = ParameterFacts::default();

        for (module, program, surface) in inputs {
            let known_class = |name: &str| super::resolve_named_class(classes, graph, module, name);
            let is_constructor = |class: &ClassId, selector: &str| {
                super::resolve_member_surface(classes, class, selector).is_some_and(|member| member.is_constructor)
                    || (selector == "new()" && classes.contains_key(class))
            };
            let callable_return = |id: &CallableId| previous_summaries.get(id).map(|summary: &CallableSummary| summary.returns.clone());
            let parameter_fact = |id: &CallableId, name: &str| previous_parameters.get(id, name).cloned();
            let resolve_member = |class: &ClassId, selector: &str| super::resolve_member_surface(classes, class, selector);
            let facts = parameter_facts_for_program(
                program,
                surface,
                module,
                known_class,
                is_constructor,
                callable_return,
                parameter_fact,
                resolve_member,
            );
            next_parameters.merge_from(&facts);
        }

        let mut next_summaries = BTreeMap::new();
        for (module, _, surface) in inputs {
            let known_class = |name: &str| super::resolve_named_class(classes, graph, module, name);
            let is_constructor = |class: &ClassId, selector: &str| {
                super::resolve_member_surface(classes, class, selector).is_some_and(|member| member.is_constructor)
                    || (selector == "new()" && classes.contains_key(class))
            };
            let callable_return = |id: &CallableId| previous_summaries.get(id).map(|summary: &CallableSummary| summary.returns.clone());
            let parameter_fact = |id: &CallableId, name: &str| next_parameters.get(id, name).cloned();
            let resolve_member = |class: &ClassId, selector: &str| super::resolve_member_surface(classes, class, selector);
            for (summary, evidence) in summaries_for_surface_with_bottom(
                surface,
                module,
                known_class,
                is_constructor,
                callable_return,
                parameter_fact,
                resolve_member,
                generation,
            ) {
                if evidence.is_some() {
                    next_summaries.insert(summary.callable.clone(), summary);
                }
            }
        }

        let summaries_changed = next_summaries != previous_summaries;
        let parameters_changed = next_parameters != previous_parameters;
        summaries = next_summaries;
        parameter_facts = next_parameters;
        if !summaries_changed && !parameters_changed {
            complete_missing_summaries(inputs, classes, graph, generation, &parameter_facts, &mut summaries);
            return SolverResult { summaries, parameter_facts };
        }
    }

    if cfg!(debug_assertions) {
        panic!("callable solver failed to converge within derived budget");
    }

    // Release builds must still publish a coherent state. Widen all facts and
    // summaries together so no caller observes a partial round.
    parameter_facts.widen_all();
    for summary in summaries.values_mut() {
        summary.returns = InferredValue::flow(ValueShape::Unknown, Default::default());
        for value in &mut summary.params {
            *value = InferredValue::flow(ValueShape::Unknown, Default::default());
        }
    }
    SolverResult { summaries, parameter_facts }
}

/// Re-solves only callable surfaces owned by `inputs` while treating the
/// supplied summaries and parameter facts as read-only boundary values.
pub(crate) fn solve_affected_callables(
    inputs: &[(ModuleId, Arc<phalcom_ast::ast::Program>, ModuleSurface)],
    classes: &BTreeMap<ClassId, super::surface::ClassSurface>,
    graph: &ModuleGraph,
    generation: SemanticGeneration,
    seed_summaries: BTreeMap<CallableId, CallableSummary>,
    base_parameters: ParameterFacts,
) -> SolverResult {
    if inputs.is_empty() {
        return SolverResult {
            summaries: seed_summaries,
            parameter_facts: base_parameters,
        };
    }

    let callable_count = inputs
        .iter()
        .map(|(_, _, surface)| surface.classes.values().map(|class| class.members.len()).sum::<usize>())
        .sum::<usize>();
    let slot_count = inputs
        .iter()
        .map(|(_, _, surface)| {
            surface
                .classes
                .values()
                .flat_map(|class| class.members.values())
                .map(|member| member.params.len())
                .sum::<usize>()
        })
        .sum::<usize>();
    let max_rounds = (callable_count + slot_count).max(1) * (MAX_SHAPE_UNION + 2);
    let mut summaries = seed_summaries;
    let mut parameter_facts = base_parameters.clone();

    for _ in 0..max_rounds {
        let previous_summaries = summaries.clone();
        let previous_parameters = parameter_facts.clone();
        let mut next_parameters = base_parameters.clone();

        for (module, program, surface) in inputs {
            let known_class = |name: &str| super::resolve_named_class(classes, graph, module, name);
            let is_constructor = |class: &ClassId, selector: &str| {
                super::resolve_member_surface(classes, class, selector).is_some_and(|member| member.is_constructor)
                    || (selector == "new()" && classes.contains_key(class))
            };
            let callable_return = |id: &CallableId| previous_summaries.get(id).map(|summary: &CallableSummary| summary.returns.clone());
            let parameter_fact = |id: &CallableId, name: &str| previous_parameters.get(id, name).cloned();
            let resolve_member = |class: &ClassId, selector: &str| super::resolve_member_surface(classes, class, selector);
            let facts = parameter_facts_for_program(
                program,
                surface,
                module,
                known_class,
                is_constructor,
                callable_return,
                parameter_fact,
                resolve_member,
            );
            next_parameters.merge_from(&facts);
        }

        let mut next_summaries = previous_summaries.clone();
        for (module, _, surface) in inputs {
            let known_class = |name: &str| super::resolve_named_class(classes, graph, module, name);
            let is_constructor = |class: &ClassId, selector: &str| {
                super::resolve_member_surface(classes, class, selector).is_some_and(|member| member.is_constructor)
                    || (selector == "new()" && classes.contains_key(class))
            };
            let callable_return = |id: &CallableId| previous_summaries.get(id).map(|summary: &CallableSummary| summary.returns.clone());
            let parameter_fact = |id: &CallableId, name: &str| next_parameters.get(id, name).cloned();
            let resolve_member = |class: &ClassId, selector: &str| super::resolve_member_surface(classes, class, selector);
            for (summary, evidence) in summaries_for_surface_with_bottom(
                surface,
                module,
                known_class,
                is_constructor,
                callable_return,
                parameter_fact,
                resolve_member,
                generation,
            ) {
                if evidence.is_some() {
                    next_summaries.insert(summary.callable.clone(), summary);
                }
            }
        }

        let summaries_changed = next_summaries != previous_summaries;
        let parameters_changed = next_parameters != previous_parameters;
        summaries = next_summaries;
        parameter_facts = next_parameters;
        if !summaries_changed && !parameters_changed {
            complete_missing_summaries(inputs, classes, graph, generation, &parameter_facts, &mut summaries);
            return SolverResult { summaries, parameter_facts };
        }
    }

    if cfg!(debug_assertions) {
        panic!("affected callable solver failed to converge within derived budget");
    }

    parameter_facts.widen_all();
    for summary in summaries.values_mut() {
        if inputs.iter().any(|(module, _, _)| summary.callable.owner.module == *module) {
            summary.returns = InferredValue::flow(ValueShape::Unknown, Default::default());
            for value in &mut summary.params {
                *value = InferredValue::flow(ValueShape::Unknown, Default::default());
            }
        }
    }
    SolverResult { summaries, parameter_facts }
}

fn complete_missing_summaries(
    inputs: &[(ModuleId, Arc<phalcom_ast::ast::Program>, ModuleSurface)],
    classes: &BTreeMap<ClassId, super::surface::ClassSurface>,
    graph: &ModuleGraph,
    generation: SemanticGeneration,
    parameter_facts: &ParameterFacts,
    summaries: &mut BTreeMap<CallableId, CallableSummary>,
) {
    for (module, _, surface) in inputs {
        let known_class = |name: &str| super::resolve_named_class(classes, graph, module, name);
        let is_constructor = |class: &ClassId, selector: &str| {
            super::resolve_member_surface(classes, class, selector).is_some_and(|member| member.is_constructor)
                || (selector == "new()" && classes.contains_key(class))
        };
        let callable_return = |id: &CallableId| summaries.get(id).map(|summary| summary.returns.clone());
        let parameter_fact = |id: &CallableId, name: &str| parameter_facts.get(id, name).cloned();
        let resolve_member = |class: &ClassId, selector: &str| super::resolve_member_surface(classes, class, selector);
        let generated = summaries_for_surface(
            surface,
            module,
            known_class,
            is_constructor,
            callable_return,
            parameter_fact,
            resolve_member,
            generation,
        );
        for summary in generated {
            summaries.entry(summary.callable.clone()).or_insert(summary);
        }
    }
}

fn collect_call_sites(
    statements: &[Statement],
    surface: &ModuleSurface,
    module: &ModuleId,
    current_class: Option<&ClassId>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    parameter_fact: impl Fn(&CallableId, &str) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&ClassId, &str) -> Option<MemberSurface> + Copy,
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
                        parameter_fact,
                        resolve_member,
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
                    parameter_fact,
                    resolve_member,
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
                        parameter_fact,
                        resolve_member,
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
                    let selector = crate::selectors::class_member_selector(member);
                    let Some(member_surface) = surface.classes.get(&id).and_then(|class| class.members.get(&selector)) else {
                        continue;
                    };
                    for param in &member_surface.params {
                        if let Some(value) = parameter_fact(&member_surface.callable, &param.name) {
                            member_environment.insert(param.name.clone(), value);
                        }
                    }
                    collect_call_sites(
                        body,
                        surface,
                        module,
                        Some(&id),
                        known_class,
                        is_constructor,
                        callable_return,
                        parameter_fact,
                        resolve_member,
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
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    parameter_fact: impl Fn(&CallableId, &str) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&ClassId, &str) -> Option<MemberSurface> + Copy,
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
                    parameter_fact,
                    resolve_member,
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
            if let Some(class) = target_class {
                if let Some(member) = resolve_member(&class, &selector) {
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
                parameter_fact,
                resolve_member,
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
                    parameter_fact,
                    resolve_member,
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
                parameter_fact,
                resolve_member,
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
                parameter_fact,
                resolve_member,
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
            parameter_fact,
            resolve_member,
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
                    parameter_fact,
                    resolve_member,
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
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    parameter_fact: impl Fn(&CallableId, &str) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&ClassId, &str) -> Option<MemberSurface> + Copy,
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
        parameter_fact,
        resolve_member,
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
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
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
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
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
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
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
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    parameter_fact: impl Fn(&CallableId, &str) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&ClassId, &str) -> Option<MemberSurface> + Copy,
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
            let dependencies = callable_dependencies(
                &member.body,
                module,
                Some(&class.id),
                &environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
            );
            CallableSummary {
                callable: member.callable.clone(),
                params,
                returns,
                dependencies,
                effects: Default::default(),
                revision,
            }
        })
        .collect()
}

type SummaryEvidence = Option<InferredValue>;

/// Computes a callable body while preserving solver-bottom for a source call
/// whose summary has not been established in the current iteration.
fn infer_summary_expr(
    expr: &Expr,
    module: &ModuleId,
    current_class: Option<&ClassId>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&ClassId, &str) -> Option<MemberSurface> + Copy,
) -> SummaryEvidence {
    let unknown = || InferredValue::flow(ValueShape::Unknown, expr.range());
    match expr {
        Expr::MethodCall(call) => {
            let receiver = infer_summary_expr(
                &call.object,
                module,
                current_class,
                environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
            )
            .unwrap_or_else(unknown);
            let selector = call_selector(&call.method, &call.args);
            let call_return = |class: ClassId| {
                let Some(member) = resolve_member(&class, &selector) else {
                    return Some(unknown());
                };
                if member.callable.owner.module.as_str() == CORE_MODULE_URI {
                    Some(unknown())
                } else {
                    callable_return(&member.callable)
                }
            };
            match receiver.shape {
                ValueShape::ClassObject(class) if is_constructor(&class, &selector) => Some(InferredValue::flow(ValueShape::Instance(class), expr.range())),
                ValueShape::Instance(class) => call_return(class),
                ValueShape::ClassObject(class) => call_return(class),
                ValueShape::Union(shapes) => {
                    let mut result = None;
                    for shape in shapes {
                        let evidence = match shape {
                            ValueShape::Instance(class) => call_return(class),
                            ValueShape::ClassObject(class) => call_return(class),
                            _ => Some(unknown()),
                        };
                        if let Some(value) = evidence {
                            result = Some(result.map_or(value.clone(), |old: InferredValue| old.join(&value)));
                        }
                    }
                    result
                }
                _ => Some(unknown()),
            }
        }
        Expr::UnqualifiedCall(call) => {
            let selector = call_selector(&call.name, &call.args);
            let Some(class) = current_class else { return Some(unknown()) };
            let Some(member) = resolve_member(class, &selector) else {
                return Some(unknown());
            };
            if member.callable.owner.module.as_str() == CORE_MODULE_URI {
                Some(unknown())
            } else {
                callable_return(&member.callable)
            }
        }
        _ => Some(infer_expr(expr, module, current_class, environment, known_class, is_constructor)),
    }
}

/// Computes summaries with absent return evidence left at solver-bottom.
fn summaries_for_surface_with_bottom(
    surface: &ModuleSurface,
    module: &ModuleId,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    parameter_fact: impl Fn(&CallableId, &str) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&ClassId, &str) -> Option<MemberSurface> + Copy,
    revision: SemanticGeneration,
) -> Vec<(CallableSummary, SummaryEvidence)> {
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
            let evidence = body_summary_value(
                &member.body,
                module,
                Some(&class.id),
                &environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
            );
            let returns = evidence
                .clone()
                .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, member.source_range));
            let dependencies = callable_dependencies(
                &member.body,
                module,
                Some(&class.id),
                &environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
            );
            (
                CallableSummary {
                    callable: member.callable.clone(),
                    params,
                    returns,
                    dependencies,
                    effects: Default::default(),
                    revision,
                },
                evidence,
            )
        })
        .collect()
}

fn body_summary_value(
    body: &[Statement],
    module: &ModuleId,
    current_class: Option<&ClassId>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&ClassId, &str) -> Option<MemberSurface> + Copy,
) -> SummaryEvidence {
    let last = body.last()?;
    let mut result = None;
    for statement in body {
        if let Statement::Return(return_statement) = statement {
            let evidence = return_statement.value.as_ref().and_then(|expr| {
                infer_summary_expr(
                    expr,
                    module,
                    current_class,
                    environment,
                    known_class,
                    is_constructor,
                    callable_return,
                    resolve_member,
                )
            });
            if let Some(value) = evidence {
                result = Some(result.map_or(value.clone(), |old: InferredValue| old.join(&value)));
            }
        }
    }
    if result.is_some() {
        return result;
    }
    match last {
        Statement::Expr { expr, .. } => infer_summary_expr(
            expr,
            module,
            current_class,
            environment,
            known_class,
            is_constructor,
            callable_return,
            resolve_member,
        ),
        Statement::Let(binding) => binding.value.as_ref().map(|expr| {
            infer_summary_expr(
                expr,
                module,
                current_class,
                environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
            )
            .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, expr.range()))
        }),
        _ => Some(InferredValue::flow(ValueShape::Unknown, last_range(last))),
    }
}

fn callable_dependencies(
    body: &[Statement],
    module: &ModuleId,
    current_class: Option<&ClassId>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&ClassId, &str) -> Option<MemberSurface> + Copy,
) -> Vec<CallableId> {
    let mut dependencies = std::collections::BTreeSet::new();
    for statement in body {
        collect_dependency_statement(
            statement,
            module,
            current_class,
            environment,
            known_class,
            is_constructor,
            callable_return,
            resolve_member,
            &mut dependencies,
        );
    }
    dependencies.into_iter().collect()
}

fn collect_dependency_statement(
    statement: &Statement,
    module: &ModuleId,
    current_class: Option<&ClassId>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&ClassId, &str) -> Option<MemberSurface> + Copy,
    dependencies: &mut std::collections::BTreeSet<CallableId>,
) {
    let expression = match statement {
        Statement::Let(binding) => binding.value.as_ref(),
        Statement::Return(return_statement) => return_statement.value.as_ref(),
        Statement::Expr { expr, .. } => Some(expr),
        Statement::Throw { expr, .. } => Some(expr),
        Statement::For(for_statement) => {
            collect_dependency_expr(
                &for_statement.iter,
                module,
                current_class,
                environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
                dependencies,
            );
            for nested in &for_statement.body {
                collect_dependency_statement(
                    nested,
                    module,
                    current_class,
                    environment,
                    known_class,
                    is_constructor,
                    callable_return,
                    resolve_member,
                    dependencies,
                );
            }
            None
        }
        Statement::Class(class) => {
            let class_id = ClassId::new(module.clone(), class.name.clone());
            for member in &class.members {
                let body = match member {
                    phalcom_ast::ast::ClassMember::Method(item) => &item.body,
                    phalcom_ast::ast::ClassMember::Getter(item) => &item.body,
                    phalcom_ast::ast::ClassMember::Setter(item) => &item.body,
                    phalcom_ast::ast::ClassMember::Index(item) => &item.body,
                    phalcom_ast::ast::ClassMember::Field(_) | phalcom_ast::ast::ClassMember::Variant(_) => continue,
                };
                for nested in body {
                    collect_dependency_statement(
                        nested,
                        module,
                        Some(&class_id),
                        environment,
                        known_class,
                        is_constructor,
                        callable_return,
                        resolve_member,
                        dependencies,
                    );
                }
            }
            None
        }
        Statement::Break { .. } | Statement::Continue { .. } | Statement::Import(_) => None,
    };
    if let Some(expression) = expression {
        collect_dependency_expr(
            expression,
            module,
            current_class,
            environment,
            known_class,
            is_constructor,
            callable_return,
            resolve_member,
            dependencies,
        );
    }
}

fn collect_dependency_expr(
    expr: &Expr,
    module: &ModuleId,
    current_class: Option<&ClassId>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&ClassId, &str) -> Option<MemberSurface> + Copy,
    dependencies: &mut std::collections::BTreeSet<CallableId>,
) {
    match expr {
        Expr::MethodCall(call) => {
            let receiver = infer_expr_with_returns(&call.object, module, current_class, environment, known_class, is_constructor, callable_return);
            let selector = call_selector(&call.method, &call.args);
            let side = match receiver.shape {
                ValueShape::Instance(class) => resolve_member(&class, &selector).map(|member| member.callable),
                ValueShape::ClassObject(class) => resolve_member(&class, &selector).map(|member| member.callable),
                ValueShape::Union(shapes) => {
                    let ids = shapes.into_iter().filter_map(|shape| match shape {
                        ValueShape::Instance(class) | ValueShape::ClassObject(class) => resolve_member(&class, &selector).map(|member| member.callable),
                        _ => None,
                    });
                    for id in ids {
                        dependencies.insert(id);
                    }
                    None
                }
                _ => None,
            };
            if let Some(id) = side {
                dependencies.insert(id);
            }
            collect_dependency_expr(
                &call.object,
                module,
                current_class,
                environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
                dependencies,
            );
            for arg in &call.args {
                collect_dependency_pack(
                    arg,
                    module,
                    current_class,
                    environment,
                    known_class,
                    is_constructor,
                    callable_return,
                    resolve_member,
                    dependencies,
                );
            }
        }
        Expr::UnqualifiedCall(call) => {
            if let Some(class) = current_class {
                if let Some(member) = resolve_member(class, &call_selector(&call.name, &call.args)) {
                    dependencies.insert(member.callable);
                }
            }
            for arg in &call.args {
                collect_dependency_pack(
                    arg,
                    module,
                    current_class,
                    environment,
                    known_class,
                    is_constructor,
                    callable_return,
                    resolve_member,
                    dependencies,
                );
            }
        }
        Expr::Assignment(assignment) => {
            collect_dependency_expr(
                &assignment.name,
                module,
                current_class,
                environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
                dependencies,
            );
            collect_dependency_expr(
                &assignment.value,
                module,
                current_class,
                environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
                dependencies,
            );
        }
        Expr::GetProperty(property) => collect_dependency_expr(
            &property.object,
            module,
            current_class,
            environment,
            known_class,
            is_constructor,
            callable_return,
            resolve_member,
            dependencies,
        ),
        Expr::SetProperty(property) => {
            collect_dependency_expr(
                &property.object,
                module,
                current_class,
                environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
                dependencies,
            );
            collect_dependency_expr(
                &property.value,
                module,
                current_class,
                environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
                dependencies,
            );
        }
        Expr::TupleLiteral(tuple) => {
            for entry in &tuple.entries {
                let value = match entry {
                    TupleLiteralEntry::Positional { expr, .. } | TupleLiteralEntry::Labeled { value: expr, .. } | TupleLiteralEntry::Expand { expr, .. } => {
                        expr
                    }
                };
                collect_dependency_expr(
                    value,
                    module,
                    current_class,
                    environment,
                    known_class,
                    is_constructor,
                    callable_return,
                    resolve_member,
                    dependencies,
                );
            }
        }
        Expr::ListLiteral(list) => {
            for entry in &list.elements {
                let value = match entry {
                    ListLiteralElement::Element { expr, .. } | ListLiteralElement::Expansion { expr, .. } => expr,
                };
                collect_dependency_expr(
                    value,
                    module,
                    current_class,
                    environment,
                    known_class,
                    is_constructor,
                    callable_return,
                    resolve_member,
                    dependencies,
                );
            }
        }
        Expr::SetLiteral(set) => {
            for entry in &set.entries {
                let value = match entry {
                    SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => expr,
                };
                collect_dependency_expr(
                    value,
                    module,
                    current_class,
                    environment,
                    known_class,
                    is_constructor,
                    callable_return,
                    resolve_member,
                    dependencies,
                );
            }
        }
        Expr::MapLiteral(map) => {
            for entry in &map.entries {
                match entry {
                    MapLiteralEntry::Association { key, value, .. } => {
                        if let MapLiteralKey::Computed { expr, .. } = key {
                            collect_dependency_expr(
                                expr,
                                module,
                                current_class,
                                environment,
                                known_class,
                                is_constructor,
                                callable_return,
                                resolve_member,
                                dependencies,
                            );
                        }
                        collect_dependency_expr(
                            value,
                            module,
                            current_class,
                            environment,
                            known_class,
                            is_constructor,
                            callable_return,
                            resolve_member,
                            dependencies,
                        );
                    }
                    MapLiteralEntry::Expansion { expr, .. } => collect_dependency_expr(
                        expr,
                        module,
                        current_class,
                        environment,
                        known_class,
                        is_constructor,
                        callable_return,
                        resolve_member,
                        dependencies,
                    ),
                }
            }
        }
        Expr::Range(range) => {
            if let Some(lower) = &range.lower {
                collect_dependency_expr(
                    lower,
                    module,
                    current_class,
                    environment,
                    known_class,
                    is_constructor,
                    callable_return,
                    resolve_member,
                    dependencies,
                );
            }
            if let Some(upper) = &range.upper {
                collect_dependency_expr(
                    upper,
                    module,
                    current_class,
                    environment,
                    known_class,
                    is_constructor,
                    callable_return,
                    resolve_member,
                    dependencies,
                );
            }
        }
        _ => {}
    }
}

fn collect_dependency_pack(
    item: &PackItem,
    module: &ModuleId,
    current_class: Option<&ClassId>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&ClassId, &str) -> Option<MemberSurface> + Copy,
    dependencies: &mut std::collections::BTreeSet<CallableId>,
) {
    let expr = match item {
        PackItem::Positional { expr, .. } | PackItem::Expand { expr, .. } | PackItem::Labeled { value: expr, .. } => expr,
    };
    collect_dependency_expr(
        expr,
        module,
        current_class,
        environment,
        known_class,
        is_constructor,
        callable_return,
        resolve_member,
        dependencies,
    );
}

fn body_value(
    body: &[Statement],
    module: &ModuleId,
    current_class: Option<&ClassId>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
) -> InferredValue {
    let Some(last) = body.last() else {
        return InferredValue::flow(ValueShape::Unknown, Default::default());
    };
    let explicit_returns = body.iter().filter_map(|statement| match statement {
        Statement::Return(return_statement) => Some(
            return_statement
                .value
                .as_ref()
                .map(|expr| infer_expr_with_returns(expr, module, current_class, environment, known_class, is_constructor, callable_return))
                .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, return_statement.range)),
        ),
        _ => None,
    });
    let mut value = None;
    for returned in explicit_returns {
        value = Some(value.map_or(returned.clone(), |old: InferredValue| old.join(&returned)));
    }
    if let Some(value) = value {
        return value;
    }
    match last {
        Statement::Expr { expr, .. } => infer_expr_with_returns(expr, module, current_class, environment, known_class, is_constructor, callable_return),
        Statement::Let(binding) => binding
            .value
            .as_ref()
            .map(|expr| infer_expr_with_returns(expr, module, current_class, environment, known_class, is_constructor, callable_return))
            .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, binding.range)),
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
        let facts = collect_local_facts_with_returns(&program, &module, no_classes, |_, _| false, |_| None);
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
