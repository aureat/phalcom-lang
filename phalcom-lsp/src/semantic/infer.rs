//! Deterministic syntax and local-flow inference.

#![allow(clippy::only_used_in_recursion, clippy::too_many_arguments)]

use std::collections::BTreeMap;
use std::sync::Arc;

use super::CallableSummary;
use super::analyzer::{AnalysisContext, analyze_expr};
use super::callable::SolverResult;
use super::dispatch::{DispatchReceiver, DispatchResolver, ResolvedDispatch};
use super::facts::{FieldFacts, InferredValue, LocalFacts, MAX_SHAPE_UNION, ParameterFacts, ValueShape};
#[cfg(test)]
use super::ids::CORE_MODULE_URI;
use super::ids::{CallableId, ClassId, DispatchSide, ModuleId};
use super::module_graph::ModuleGraph;
use super::query::SemanticGeneration;
use super::surface::ModuleSurface;
use crate::selectors::call_selector;
use phalcom_ast::ast::{
    Expr, ListLiteralElement, MapLiteralEntry, MapLiteralKey, MethodRefKind, PackItem, PackLabel, Pattern, SetLiteralEntry, Statement, TupleLiteralEntry,
};

fn infer_expr_with_dispatch(
    expr: &Expr,
    _module: &ModuleId,
    current_class: Option<&ClassId>,
    dispatch_side: Option<DispatchSide>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
) -> InferredValue {
    let range = expr.range();
    let returns = |targets: &[DispatchReceiver], selector: &str| {
        let shapes = targets.iter().map(|target| {
            if let DispatchReceiver::ClassObject(class) = target
                && is_constructor(class, selector)
            {
                return ValueShape::Instance(class.clone());
            }
            let Some(resolved) = resolve_member(target, selector) else {
                return ValueShape::Unknown;
            };
            if let DispatchReceiver::ClassObject(class) = target {
                if resolved.member.is_constructor {
                    return ValueShape::Instance(class.clone());
                }
            }
            callable_return(&resolved.member.callable)
                .map(|value| value.shape)
                .unwrap_or(ValueShape::Unknown)
        });
        ValueShape::bounded_union(shapes)
    };
    match expr {
        Expr::MethodCall(call) => {
            let receiver = infer_expr_with_dispatch(
                &call.object,
                _module,
                current_class,
                dispatch_side,
                environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
            );
            let selector = call_selector(&call.method, &call.args);
            let targets = dispatch_targets(&call.object, &receiver.shape, current_class, dispatch_side);
            InferredValue::flow(returns(&targets, &selector), range)
        }
        Expr::GetProperty(property) => {
            if let Expr::Var { value: binding, .. } = &property.object
                && !environment.contains_key(binding)
                && let Some(class) = known_class(&format!("{binding}.{}", property.property))
            {
                return InferredValue::exact(ValueShape::ClassObject(class), range);
            }
            let receiver = infer_expr_with_dispatch(
                &property.object,
                _module,
                current_class,
                dispatch_side,
                environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
            );
            let targets = dispatch_targets(&property.object, &receiver.shape, current_class, dispatch_side);
            InferredValue::flow(returns(&targets, &property.property), range)
        }
        Expr::UnqualifiedCall(call) => {
            if let Some(binding) = environment.get(&call.name) {
                match &binding.shape {
                    ValueShape::Callable(callable) => {
                        return callable_return(callable).unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, range));
                    }
                    ValueShape::Family { receiver, base } => {
                        let selector = call_selector(base, &call.args);
                        let targets = dispatch_targets_for_shape(receiver);
                        return InferredValue::flow(returns(&targets, &selector), range);
                    }
                    _ => return InferredValue::flow(ValueShape::Unknown, range),
                }
            }
            let Some(class) = current_class else {
                return InferredValue::flow(ValueShape::Unknown, range);
            };
            let target = match dispatch_side.unwrap_or(DispatchSide::Instance) {
                DispatchSide::Instance => DispatchReceiver::Instance(class.clone()),
                DispatchSide::Class => DispatchReceiver::ClassObject(class.clone()),
            };
            InferredValue::flow(returns(&[target], &call_selector(&call.name, &call.args)), range)
        }
        Expr::SelfVar { .. } => match (current_class, dispatch_side) {
            (Some(class), Some(DispatchSide::Class)) => InferredValue::exact(ValueShape::ClassObject(class.clone()), range),
            (Some(class), _) => InferredValue::exact(ValueShape::Instance(class.clone()), range),
            _ => InferredValue::flow(ValueShape::Unknown, range),
        },
        _ => {
            let no_field = |_: &ClassId, _: &str| None;
            let contains_class = |class: &ClassId| is_constructor(class, "new()");
            let context = AnalysisContext {
                current_class,
                dispatch_side,
                query_offset: 0,
                environment,
                local_facts: None,
                known_class: &known_class,
                callable_return: &callable_return,
                field_value: &no_field,
                resolver: &resolve_member,
                contains_class: &contains_class,
            };
            analyze_expr(expr, &context)
        }
    }
}

fn dispatch_targets(object: &Expr, shape: &ValueShape, current_class: Option<&ClassId>, dispatch_side: Option<DispatchSide>) -> Vec<DispatchReceiver> {
    if matches!(object, Expr::SuperVar { .. }) {
        return current_class
            .map(|class| {
                vec![DispatchReceiver::Super {
                    lexical_class: class.clone(),
                    side: dispatch_side.unwrap_or(DispatchSide::Instance),
                }]
            })
            .unwrap_or_default();
    }
    dispatch_targets_for_shape(shape)
}

fn dispatch_targets_for_shape(shape: &ValueShape) -> Vec<DispatchReceiver> {
    match shape {
        ValueShape::Instance(class) => vec![DispatchReceiver::Instance(class.clone())],
        ValueShape::ClassObject(class) => vec![DispatchReceiver::ClassObject(class.clone())],
        ValueShape::Union(shapes) => shapes.iter().flat_map(dispatch_targets_for_shape).collect(),
        _ => Vec::new(),
    }
}

/// Collects constructor-assigned field facts from one module's source surface.
pub fn field_facts_for_surface(
    surface: &ModuleSurface,
    _module: &ModuleId,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    _is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolver: DispatchResolver<'_>,
) -> FieldFacts {
    let mut facts = FieldFacts::default();
    for class in surface.classes.values() {
        for field in class.fields.values() {
            if let Some(initializer) = &field.initializer {
                let value = analyze_with_resolver(
                    initializer,
                    Some(&class.id),
                    Some(if field.is_class_side { DispatchSide::Class } else { DispatchSide::Instance }),
                    &BTreeMap::new(),
                    &known_class,
                    &callable_return,
                    &|_, _| None,
                    resolver,
                );
                facts.record(class.id.clone(), field.name.clone(), value);
            }
        }
        for member in class.members_by_side.values() {
            for statement in &member.body {
                if let Statement::Expr {
                    expr: Expr::Assignment(assignment),
                    ..
                } = statement
                {
                    let Expr::Field { value: name, range, .. } = assignment.name.as_ref() else {
                        continue;
                    };
                    let inferred = analyze_with_resolver(
                        &assignment.value,
                        Some(&class.id),
                        Some(member.side),
                        &BTreeMap::new(),
                        &known_class,
                        &callable_return,
                        &|_, _| None,
                        resolver,
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
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
) -> ParameterFacts {
    let mut facts = ParameterFacts::default();
    let mut environment = BTreeMap::new();
    collect_call_sites(
        &program.statements,
        surface,
        module,
        None,
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
        .map(|(_, _, surface)| surface.classes.values().map(|class| class.members_by_side.len()).sum::<usize>())
        .sum::<usize>();
    let slot_count = inputs
        .iter()
        .map(|(_, _, surface)| {
            surface
                .classes
                .values()
                .flat_map(|class| class.members_by_side.values())
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
            let resolver = DispatchResolver::new(classes);
            let is_constructor = |class: &ClassId, selector: &str| {
                resolver
                    .resolve(&DispatchReceiver::ClassObject(class.clone()), selector)
                    .is_some_and(|resolved| resolved.member.is_constructor)
                    || (selector == "new()" && classes.contains_key(class))
            };
            let callable_return = |id: &CallableId| super::return_for_callable(classes, &previous_summaries, id);
            let parameter_fact = |id: &CallableId, name: &str| previous_parameters.get(id, name).cloned();
            let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
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
            let resolver = DispatchResolver::new(classes);
            let is_constructor = |class: &ClassId, selector: &str| {
                resolver
                    .resolve(&DispatchReceiver::ClassObject(class.clone()), selector)
                    .is_some_and(|resolved| resolved.member.is_constructor)
                    || (selector == "new()" && classes.contains_key(class))
            };
            let callable_return = |id: &CallableId| super::return_for_callable(classes, &previous_summaries, id);
            let parameter_fact = |id: &CallableId, name: &str| next_parameters.get(id, name).cloned();
            let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
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
        .map(|(_, _, surface)| surface.classes.values().map(|class| class.members_by_side.len()).sum::<usize>())
        .sum::<usize>();
    let slot_count = inputs
        .iter()
        .map(|(_, _, surface)| {
            surface
                .classes
                .values()
                .flat_map(|class| class.members_by_side.values())
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
            let resolver = DispatchResolver::new(classes);
            let is_constructor = |class: &ClassId, selector: &str| {
                resolver
                    .resolve(&DispatchReceiver::ClassObject(class.clone()), selector)
                    .is_some_and(|resolved| resolved.member.is_constructor)
                    || (selector == "new()" && classes.contains_key(class))
            };
            let callable_return = |id: &CallableId| super::return_for_callable(classes, &previous_summaries, id);
            let parameter_fact = |id: &CallableId, name: &str| previous_parameters.get(id, name).cloned();
            let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
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
            let resolver = DispatchResolver::new(classes);
            let is_constructor = |class: &ClassId, selector: &str| {
                resolver
                    .resolve(&DispatchReceiver::ClassObject(class.clone()), selector)
                    .is_some_and(|resolved| resolved.member.is_constructor)
                    || (selector == "new()" && classes.contains_key(class))
            };
            let callable_return = |id: &CallableId| super::return_for_callable(classes, &previous_summaries, id);
            let parameter_fact = |id: &CallableId, name: &str| next_parameters.get(id, name).cloned();
            let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
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
        let resolver = DispatchResolver::new(classes);
        let is_constructor = |class: &ClassId, selector: &str| {
            resolver
                .resolve(&DispatchReceiver::ClassObject(class.clone()), selector)
                .is_some_and(|resolved| resolved.member.is_constructor)
                || (selector == "new()" && classes.contains_key(class))
        };
        let callable_return = |id: &CallableId| super::return_for_callable(classes, summaries, id);
        let parameter_fact = |id: &CallableId, name: &str| parameter_facts.get(id, name).cloned();
        let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
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
    dispatch_side: Option<DispatchSide>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    parameter_fact: impl Fn(&CallableId, &str) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
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
                        dispatch_side,
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
                            infer_expr_with_dispatch(
                                value,
                                module,
                                current_class,
                                dispatch_side,
                                environment,
                                known_class,
                                is_constructor,
                                callable_return,
                                resolve_member,
                            ),
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
                    dispatch_side,
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
                        let inferred = infer_expr_with_dispatch(
                            &assignment.value,
                            module,
                            current_class,
                            dispatch_side,
                            environment,
                            known_class,
                            is_constructor,
                            callable_return,
                            resolve_member,
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
                        dispatch_side,
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
                    let member_end = match member {
                        phalcom_ast::ast::ClassMember::Method(item) => item.range.end,
                        phalcom_ast::ast::ClassMember::Getter(item) => item.range.end,
                        phalcom_ast::ast::ClassMember::Setter(item) => item.range.end,
                        phalcom_ast::ast::ClassMember::Index(item) => item.range.end,
                        phalcom_ast::ast::ClassMember::Field(item) => item.range.end,
                        phalcom_ast::ast::ClassMember::Variant(item) => item.range.end,
                    };
                    let Some(member_surface) = surface.classes.get(&id).and_then(|class| {
                        class
                            .members_by_side
                            .values()
                            .find(|member| member.callable.selector == selector && member.source_range.end == member_end)
                    }) else {
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
                        Some(member_surface.side),
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
    dispatch_side: Option<DispatchSide>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    parameter_fact: impl Fn(&CallableId, &str) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
    environment: &BTreeMap<String, InferredValue>,
    facts: &mut ParameterFacts,
) {
    match expr {
        Expr::UnqualifiedCall(call) => {
            let selector = call_selector(&call.name, &call.args);
            let target = current_class
                .and_then(|class| {
                    let receiver = match dispatch_side.unwrap_or(DispatchSide::Instance) {
                        DispatchSide::Instance => DispatchReceiver::Instance(class.clone()),
                        DispatchSide::Class => DispatchReceiver::ClassObject(class.clone()),
                    };
                    resolve_member(&receiver, &selector).map(|resolved| resolved.member)
                })
                .or_else(|| {
                    let mut matches = surface
                        .classes
                        .values()
                        .flat_map(|class| class.members_by_side.values())
                        .filter(|member| member.callable.selector == selector);
                    let first = matches.next()?.clone();
                    matches.next().is_none().then_some(first)
                });
            if let Some(member) = target {
                record_call_arguments(
                    &member.callable,
                    &call.args,
                    &member.params,
                    module,
                    current_class,
                    dispatch_side,
                    environment,
                    known_class,
                    is_constructor,
                    callable_return,
                    resolve_member,
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
                    dispatch_side,
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
            let receiver = infer_expr_with_dispatch(
                &call.object,
                module,
                current_class,
                dispatch_side,
                environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
            );
            let selector = call_selector(&call.method, &call.args);
            let target = match receiver.shape {
                ValueShape::Instance(class) => Some(DispatchReceiver::Instance(class)),
                ValueShape::ClassObject(class) => Some(DispatchReceiver::ClassObject(class)),
                _ => None,
            };
            if let Some(target) = target {
                if let Some(resolved) = resolve_member(&target, &selector) {
                    let member = resolved.member;
                    record_call_arguments(
                        &member.callable,
                        &call.args,
                        &member.params,
                        module,
                        current_class,
                        dispatch_side,
                        environment,
                        known_class,
                        is_constructor,
                        callable_return,
                        resolve_member,
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
                dispatch_side,
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
                    dispatch_side,
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
                dispatch_side,
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
                dispatch_side,
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
            dispatch_side,
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
                    dispatch_side,
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
    dispatch_side: Option<DispatchSide>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    parameter_fact: impl Fn(&CallableId, &str) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
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
        dispatch_side,
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
    dispatch_side: Option<DispatchSide>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
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
        let inferred = infer_expr_with_dispatch(
            expr,
            module,
            current_class,
            dispatch_side,
            environment,
            known_class,
            is_constructor,
            callable_return,
            resolve_member,
        );
        if !matches!(inferred.shape, ValueShape::Unknown) {
            facts.record(callable.clone(), param.name.clone(), InferredValue::interprocedural(inferred.shape, call_range));
        }
    }
}

/// Collects local facts with source-callable summaries enabled.
pub fn collect_local_facts_with_returns(
    program: &phalcom_ast::ast::Program,
    surface: &ModuleSurface,
    module: &ModuleId,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
) -> LocalFacts {
    let mut facts = LocalFacts::default();
    let mut environment = BTreeMap::new();
    collect_statements_with_returns(
        &program.statements,
        module,
        None,
        None,
        known_class,
        is_constructor,
        callable_return,
        resolve_member,
        surface,
        &mut environment,
        &mut facts,
    );
    facts
}

fn collect_statements_with_returns(
    statements: &[Statement],
    module: &ModuleId,
    current_class: Option<&ClassId>,
    dispatch_side: Option<DispatchSide>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
    surface: &ModuleSurface,
    environment: &mut BTreeMap<String, InferredValue>,
    facts: &mut LocalFacts,
) {
    for statement in statements {
        match statement {
            Statement::Let(binding) => {
                let value = binding
                    .value
                    .as_ref()
                    .map(|expr| {
                        infer_expr_with_dispatch(
                            expr,
                            module,
                            current_class,
                            dispatch_side,
                            environment,
                            known_class,
                            is_constructor,
                            callable_return,
                            resolve_member,
                        )
                    })
                    .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, binding.range));
                bind_pattern(&binding.pattern, &value, module, current_class, known_class, environment, facts);
            }
            Statement::Expr {
                expr: Expr::Assignment(assignment),
                ..
            } => {
                let value = infer_expr_with_dispatch(
                    &assignment.value,
                    module,
                    current_class,
                    dispatch_side,
                    environment,
                    known_class,
                    is_constructor,
                    callable_return,
                    resolve_member,
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
                    let selector = crate::selectors::class_member_selector(member);
                    let member_end = match member {
                        phalcom_ast::ast::ClassMember::Method(item) => item.range.end,
                        phalcom_ast::ast::ClassMember::Getter(item) => item.range.end,
                        phalcom_ast::ast::ClassMember::Setter(item) => item.range.end,
                        phalcom_ast::ast::ClassMember::Index(item) => item.range.end,
                        phalcom_ast::ast::ClassMember::Field(item) => item.range.end,
                        phalcom_ast::ast::ClassMember::Variant(item) => item.range.end,
                    };
                    let member_side = surface
                        .classes
                        .get(&id)
                        .and_then(|class| {
                            class
                                .members_by_side
                                .values()
                                .find(|candidate| candidate.callable.selector == selector && candidate.source_range.end == member_end)
                        })
                        .map(|member| member.side)
                        .unwrap_or(DispatchSide::Instance);
                    collect_statements_with_returns(
                        body,
                        module,
                        Some(&id),
                        Some(member_side),
                        known_class,
                        is_constructor,
                        callable_return,
                        resolve_member,
                        surface,
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
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
    revision: SemanticGeneration,
) -> Vec<CallableSummary> {
    surface
        .classes
        .values()
        .flat_map(|class| class.members_by_side.values().map(move |member| (class, member)))
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
                Some(member.side),
                &environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
            );
            let dependencies = callable_dependencies(
                &member.body,
                module,
                Some(&class.id),
                Some(member.side),
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
    dispatch_side: Option<DispatchSide>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
) -> SummaryEvidence {
    let unknown = || InferredValue::flow(ValueShape::Unknown, expr.range());
    match expr {
        Expr::MethodCall(call) => {
            let receiver = infer_summary_expr(
                &call.object,
                module,
                current_class,
                dispatch_side,
                environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
            )
            .unwrap_or_else(unknown);
            let selector = call_selector(&call.method, &call.args);
            let call_return = |target: DispatchReceiver| {
                let Some(resolved) = resolve_member(&target, &selector) else {
                    return Some(unknown());
                };
                callable_return(&resolved.member.callable).map(|value| InferredValue::flow(value.shape, expr.range()))
            };
            match receiver.shape {
                ValueShape::ClassObject(class) => {
                    let target = DispatchReceiver::ClassObject(class.clone());
                    if is_constructor(&class, &selector) || resolve_member(&target, &selector).is_some_and(|resolved| resolved.member.is_constructor) {
                        Some(InferredValue::flow(ValueShape::Instance(class), expr.range()))
                    } else {
                        call_return(target)
                    }
                }
                ValueShape::Instance(class) => call_return(DispatchReceiver::Instance(class)),
                ValueShape::Union(shapes) => {
                    let mut result = None;
                    for shape in shapes {
                        let evidence = match shape {
                            ValueShape::Instance(class) => call_return(DispatchReceiver::Instance(class)),
                            ValueShape::ClassObject(class) => call_return(DispatchReceiver::ClassObject(class)),
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
            if let Some(binding) = environment.get(&call.name) {
                return match &binding.shape {
                    ValueShape::Callable(callable) => callable_return(callable).map(|value| InferredValue::flow(value.shape, expr.range())),
                    ValueShape::Family { receiver, base } => {
                        let selector = call_selector(base, &call.args);
                        let targets = dispatch_targets_for_shape(receiver);
                        let mut result = None;
                        for target in targets {
                            let Some(resolved) = resolve_member(&target, &selector) else {
                                continue;
                            };
                            let Some(value) = callable_return(&resolved.member.callable) else {
                                continue;
                            };
                            let value = InferredValue::flow(value.shape, expr.range());
                            result = Some(result.map_or(value.clone(), |old: InferredValue| old.join(&value)));
                        }
                        result.or_else(|| Some(unknown()))
                    }
                    _ => Some(unknown()),
                };
            }
            let selector = call_selector(&call.name, &call.args);
            let Some(class) = current_class else { return Some(unknown()) };
            let Some(resolved) = resolve_member(&DispatchReceiver::Instance(class.clone()), &selector) else {
                return Some(unknown());
            };
            callable_return(&resolved.member.callable).map(|value| InferredValue::flow(value.shape, expr.range()))
        }
        _ => Some(infer_expr_with_dispatch(
            expr,
            module,
            current_class,
            dispatch_side,
            environment,
            known_class,
            is_constructor,
            callable_return,
            resolve_member,
        )),
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
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
    revision: SemanticGeneration,
) -> Vec<(CallableSummary, SummaryEvidence)> {
    surface
        .classes
        .values()
        .flat_map(|class| class.members_by_side.values().map(move |member| (class, member)))
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
                Some(member.side),
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
                Some(member.side),
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
    dispatch_side: Option<DispatchSide>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
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
                    dispatch_side,
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
            dispatch_side,
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
                dispatch_side,
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
    dispatch_side: Option<DispatchSide>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
) -> Vec<CallableId> {
    let mut dependencies = std::collections::BTreeSet::new();
    for statement in body {
        collect_dependency_statement(
            statement,
            module,
            current_class,
            dispatch_side,
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
    dispatch_side: Option<DispatchSide>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
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
                dispatch_side,
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
                    dispatch_side,
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
                        dispatch_side,
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
            dispatch_side,
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
    dispatch_side: Option<DispatchSide>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
    dependencies: &mut std::collections::BTreeSet<CallableId>,
) {
    match expr {
        Expr::MethodCall(call) => {
            let receiver = infer_expr_with_dispatch(
                &call.object,
                module,
                current_class,
                dispatch_side,
                environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
            );
            let selector = call_selector(&call.method, &call.args);
            let side = match receiver.shape {
                ValueShape::Instance(class) => resolve_member(&DispatchReceiver::Instance(class), &selector).map(|resolved| resolved.member.callable),
                ValueShape::ClassObject(class) => resolve_member(&DispatchReceiver::ClassObject(class), &selector).map(|resolved| resolved.member.callable),
                ValueShape::Union(shapes) => {
                    let ids = shapes.into_iter().filter_map(|shape| match shape {
                        ValueShape::Instance(class) => resolve_member(&DispatchReceiver::Instance(class), &selector).map(|resolved| resolved.member.callable),
                        ValueShape::ClassObject(class) => {
                            resolve_member(&DispatchReceiver::ClassObject(class), &selector).map(|resolved| resolved.member.callable)
                        }
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
                dispatch_side,
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
                    dispatch_side,
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
            if let Some(binding) = environment.get(&call.name) {
                match &binding.shape {
                    ValueShape::Callable(callable) => {
                        dependencies.insert(callable.clone());
                    }
                    ValueShape::Family { receiver, base } => {
                        let selector = call_selector(base, &call.args);
                        for target in dispatch_targets_for_shape(receiver) {
                            if let Some(resolved) = resolve_member(&target, &selector) {
                                dependencies.insert(resolved.member.callable);
                            }
                        }
                    }
                    _ => {}
                }
            } else if let Some(class) = current_class {
                let target = match dispatch_side.unwrap_or(DispatchSide::Instance) {
                    DispatchSide::Instance => DispatchReceiver::Instance(class.clone()),
                    DispatchSide::Class => DispatchReceiver::ClassObject(class.clone()),
                };
                if let Some(resolved) = resolve_member(&target, &call_selector(&call.name, &call.args)) {
                    dependencies.insert(resolved.member.callable);
                }
            }
            for arg in &call.args {
                collect_dependency_pack(
                    arg,
                    module,
                    current_class,
                    dispatch_side,
                    environment,
                    known_class,
                    is_constructor,
                    callable_return,
                    resolve_member,
                    dependencies,
                );
            }
        }
        Expr::MethodRef(reference) => {
            collect_dependency_expr(
                &reference.receiver,
                module,
                current_class,
                dispatch_side,
                environment,
                known_class,
                is_constructor,
                callable_return,
                resolve_member,
                dependencies,
            );
            if let MethodRefKind::Pinned { name, labels } = &reference.kind {
                let receiver = infer_expr_with_dispatch(
                    &reference.receiver,
                    module,
                    current_class,
                    dispatch_side,
                    environment,
                    known_class,
                    is_constructor,
                    callable_return,
                    resolve_member,
                );
                let selector = crate::selectors::comma_form_from_labels(name, labels);
                for target in dispatch_targets(&reference.receiver, &receiver.shape, current_class, dispatch_side) {
                    if let Some(resolved) = resolve_member(&target, &selector) {
                        dependencies.insert(resolved.member.callable);
                    }
                }
            }
        }
        Expr::Assignment(assignment) => {
            collect_dependency_expr(
                &assignment.name,
                module,
                current_class,
                dispatch_side,
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
                dispatch_side,
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
            dispatch_side,
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
                dispatch_side,
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
                dispatch_side,
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
                    dispatch_side,
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
                    dispatch_side,
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
                    dispatch_side,
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
                                dispatch_side,
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
                            dispatch_side,
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
                        dispatch_side,
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
                    dispatch_side,
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
                    dispatch_side,
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
    dispatch_side: Option<DispatchSide>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
    dependencies: &mut std::collections::BTreeSet<CallableId>,
) {
    let expr = match item {
        PackItem::Positional { expr, .. } | PackItem::Expand { expr, .. } | PackItem::Labeled { value: expr, .. } => expr,
    };
    collect_dependency_expr(
        expr,
        module,
        current_class,
        dispatch_side,
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
    dispatch_side: Option<DispatchSide>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
) -> InferredValue {
    let Some(last) = body.last() else {
        return InferredValue::flow(ValueShape::Unknown, Default::default());
    };
    let explicit_returns = body.iter().filter_map(|statement| match statement {
        Statement::Return(return_statement) => Some(
            return_statement
                .value
                .as_ref()
                .map(|expr| {
                    infer_expr_with_dispatch(
                        expr,
                        module,
                        current_class,
                        dispatch_side,
                        environment,
                        known_class,
                        is_constructor,
                        callable_return,
                        resolve_member,
                    )
                })
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
        Statement::Expr { expr, .. } => infer_expr_with_dispatch(
            expr,
            module,
            current_class,
            dispatch_side,
            environment,
            known_class,
            is_constructor,
            callable_return,
            resolve_member,
        ),
        Statement::Let(binding) => binding
            .value
            .as_ref()
            .map(|expr| {
                infer_expr_with_dispatch(
                    expr,
                    module,
                    current_class,
                    dispatch_side,
                    environment,
                    known_class,
                    is_constructor,
                    callable_return,
                    resolve_member,
                )
            })
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

#[cfg(test)]
fn core_class(name: &str) -> ClassId {
    ClassId::new(ModuleId::new(CORE_MODULE_URI), name)
}

fn analyze_with_resolver(
    expr: &Expr,
    current_class: Option<&ClassId>,
    dispatch_side: Option<DispatchSide>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: &dyn Fn(&str) -> Option<ClassId>,
    callable_return: &dyn Fn(&CallableId) -> Option<InferredValue>,
    field_value: &dyn Fn(&ClassId, &str) -> Option<InferredValue>,
    resolver: DispatchResolver<'_>,
) -> InferredValue {
    let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
    let contains_class = |class: &ClassId| resolver.contains_class(class);
    let context = AnalysisContext {
        current_class,
        dispatch_side,
        query_offset: 0,
        environment,
        local_facts: None,
        known_class,
        callable_return,
        field_value,
        resolver: &resolve_member,
        contains_class: &contains_class,
    };
    analyze_expr(expr, &context)
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
        let surface = super::super::surface::build_module_surface(module.clone(), &program);
        let facts = collect_local_facts_with_returns(&program, &surface, &module, no_classes, |_, _| false, |_| None, |_, _| None);
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
