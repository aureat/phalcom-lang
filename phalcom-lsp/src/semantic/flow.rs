//! Structured statement flow shared by local, summary, field, and call-site analysis.

use std::collections::{BTreeMap, BTreeSet};

use phalcom_ast::ast::{
    BinaryOp, BlockExpr, Expr, ListLiteralElement, MapLiteralEntry, MapLiteralKey, MethodRefKind, PackItem, PackLabel, Pattern, SetLiteralEntry, Statement,
    TupleLiteralEntry, UnaryOp,
};
use phalcom_common::range::SourceRange;

use super::analyzer::{AnalysisContext, analyze_expr};
use super::callable::CallableSummary;
use super::dispatch::{DispatchReceiver, ResolvedDispatch};
use super::facts::{FieldFacts, InferredValue, LocalFacts, MAX_SHAPE_UNION, ParameterFacts, ValueShape};
use super::ids::{CallableId, ClassId, DispatchSide, FieldId};
use super::query::SemanticGeneration;
use super::scope::{BindingId, ScopeGraph};
use super::snapshot::FileSourceSnapshot;
use super::source::{field_initializer, member_body};
use super::surface::{MemberSurface, ModuleSurface};
use crate::perf::PerfCounters;
use crate::selectors::{binary_selector_name, call_selector, index_selector_from_labels, setter_selector_from_name, unary_selector_name};

#[cfg(test)]
thread_local! {
    static TEST_FLOW_PASSES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static TEST_SURFACE_FLOW_PASSES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static TEST_CALLABLE_FLOW_PASSES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn test_flow_passes() -> (u64, u64, u64) {
    (
        TEST_FLOW_PASSES.with(std::cell::Cell::get),
        TEST_SURFACE_FLOW_PASSES.with(std::cell::Cell::get),
        TEST_CALLABLE_FLOW_PASSES.with(std::cell::Cell::get),
    )
}

/// Abstract lexical state at one reachable program point.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FlowState {
    /// Current value knowledge keyed by lexical binding identity.
    pub bindings: BTreeMap<BindingId, InferredValue>,
}

/// Evidence for one reachable return from a callable body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReturnEvidence {
    /// Callable receiving the return.
    pub target: CallableId,
    /// Inferred returned value.
    pub value: InferredValue,
    /// Exact return statement range.
    pub range: SourceRange,
}

/// Result of analyzing one statement sequence.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StatementFlow {
    /// State that continues after the sequence, if any.
    pub normal: Option<FlowState>,
    /// Reachable returns collected from the sequence.
    pub returns: Vec<ReturnEvidence>,
    /// Loop exits collected from the sequence.
    pub breaks: Vec<FlowState>,
    /// Loop back-edges collected from the sequence.
    pub continues: Vec<FlowState>,
    /// Whether a reachable throw was observed.
    pub throws: bool,
    /// Value of the last reachable expression statement, when present.
    pub tail_value: Option<InferredValue>,
}

/// One statically analyzed argument at a resolved call site.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalyzedArgument {
    /// Static keyword label, if present.
    pub label: Option<String>,
    /// Inferred argument value.
    pub value: InferredValue,
    /// Lexical binding identity when argument is a variable reference.
    pub binding: Option<BindingId>,
    /// Effects retained for a literal block argument.
    pub block_effect: Option<BlockEffects>,
    /// Exact argument range.
    pub range: SourceRange,
}

/// One resolved call emitted by structured flow.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedCall {
    /// Actual declaration owner selected by dispatch.
    pub target: CallableId,
    /// Exact call expression range.
    pub site: SourceRange,
    /// Arguments with value evidence.
    pub args: Vec<AnalyzedArgument>,
    /// Whether the call used a dynamic pack or otherwise conservative dispatch.
    pub dynamic: bool,
}

/// Event emitted while recursively analyzing one executable path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnalysisEvent {
    /// A statically resolved send.
    Call(ResolvedCall),
    /// A field write observed in flow order.
    FieldWrite {
        /// Defining class in the current source surface.
        field: FieldId,
        /// Written value.
        value: InferredValue,
        /// Exact assignment target range.
        site: SourceRange,
    },
}

/// Effects of constructing a block without assuming that it executes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BlockEffects {
    /// Values returned non-locally to the block's home callable when invoked.
    pub nonlocal_returns: Vec<ReturnEvidence>,
    /// Captured lexical writes observed on a normal block path.
    pub captured_writes: BTreeMap<BindingId, InferredValue>,
    /// Home-callable parameters invoked by this block when it runs.
    pub invokes_parameters: BTreeSet<usize>,
    /// Whether the block contains a dynamic send.
    pub dynamic_send: bool,
}

/// All semantic products emitted from one source surface pass.
#[derive(Clone, Debug, Default)]
pub struct SurfaceFlowAnalysis {
    /// Binding facts keyed by lexical identity.
    pub local_facts: LocalFacts,
    /// Field writes and declaration initializers.
    pub field_facts: FieldFacts,
    /// Call-site parameter evidence.
    pub parameter_facts: ParameterFacts,
    /// Callable summaries and whether each has reachable return evidence.
    pub summaries: Vec<(CallableSummary, bool)>,
}

/// Joins a sequence of facts while preserving bounded-shape semantics.
pub fn join_values(values: impl IntoIterator<Item = InferredValue>) -> InferredValue {
    let mut values = values.into_iter();
    let Some(mut joined) = values.next() else {
        return InferredValue::exact(ValueShape::Unknown, Default::default());
    };
    for value in values {
        joined = joined.join(&value);
    }
    joined
}

fn join_reachable_values(values: impl IntoIterator<Item = InferredValue>) -> InferredValue {
    let values = values.into_iter().collect::<Vec<_>>();
    let known = values
        .iter()
        .filter(|value| !matches!(value.shape, ValueShape::Unknown))
        .cloned()
        .collect::<Vec<_>>();
    if known.is_empty() { join_values(values) } else { join_values(known) }
}

/// Immutable inputs shared by one structured flow pass.
pub struct SolverContext<'ctx> {
    /// Resolves source-visible class names.
    pub known_class: &'ctx dyn Fn(&str) -> Option<ClassId>,
    /// Tests whether a class is part of the current semantic universe.
    pub contains_class: &'ctx dyn Fn(&ClassId) -> bool,
    /// Reads the latest callable return summary.
    pub callable_return: &'ctx dyn Fn(&CallableId) -> Option<InferredValue>,
    /// Reads the latest callable effects summary.
    pub callable_effects: &'ctx dyn Fn(&CallableId) -> Option<super::callable::SummaryEffects>,
    /// Reads the latest parameter fact.
    pub parameter_fact: &'ctx dyn Fn(&CallableId, &str) -> Option<InferredValue>,
    /// Reads a previously established field fact.
    pub field_value: &'ctx dyn Fn(&ClassId, &str, DispatchSide) -> Option<InferredValue>,
    /// Resolves a receiver send without allocating a copied surface.
    pub resolve_member: &'ctx dyn Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch>,
    /// Looks up member metadata by callable identity.
    pub member_surface: &'ctx dyn Fn(&CallableId) -> Option<MemberSurface>,
    /// Tests trusted nominal type-test membership against the current class universe.
    pub is_same_or_subclass: &'ctx dyn Fn(&ClassId, &ClassId) -> bool,
}

/// Runs one structured pass over a module and all source members.
pub fn analyze_surface(source: &FileSourceSnapshot, context: &SolverContext<'_>, revision: SemanticGeneration, counters: &PerfCounters) -> SurfaceFlowAnalysis {
    #[cfg(test)]
    TEST_SURFACE_FLOW_PASSES.with(|count| count.set(count.get() + 1));
    analyze_surface_for_callable(source, context, revision, None, false, counters)
}

/// Runs one structured flow pass for one callable. Top-level and field work is
/// skipped so callable worklist rounds do not repeatedly traverse unrelated
/// member bodies.
pub fn analyze_callable(
    source: &FileSourceSnapshot,
    context: &SolverContext<'_>,
    revision: SemanticGeneration,
    callable: &CallableId,
    include_top_level: bool,
    counters: &PerfCounters,
) -> SurfaceFlowAnalysis {
    #[cfg(test)]
    TEST_CALLABLE_FLOW_PASSES.with(|count| count.set(count.get() + 1));
    analyze_surface_for_callable(source, context, revision, Some(callable), include_top_level, counters)
}

fn analyze_surface_for_callable(
    source: &FileSourceSnapshot,
    context: &SolverContext<'_>,
    revision: SemanticGeneration,
    target_callable: Option<&CallableId>,
    include_top_level: bool,
    counters: &PerfCounters,
) -> SurfaceFlowAnalysis {
    counters.flow_passes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    #[cfg(test)]
    TEST_FLOW_PASSES.with(|count| count.set(count.get() + 1));
    let program = source.program.as_ref();
    let surface = &source.surface;
    let scopes = &source.scopes;
    let mut analyzer = FlowAnalyzer {
        surface,
        scopes: &scopes,
        known_class: context.known_class,
        contains_class: context.contains_class,
        callable_return: context.callable_return,
        parameter_fact: context.parameter_fact,
        field_value: context.field_value,
        resolve_member: context.resolve_member,
        callable_effects: context.callable_effects,
        member_surface: context.member_surface,
        is_same_or_subclass: context.is_same_or_subclass,
        local_facts: LocalFacts::default(),
        field_facts: FieldFacts::default(),
        parameter_facts: ParameterFacts::default(),
        events: Vec::new(),
        dynamic_send: false,
        invoked_parameters: BTreeSet::new(),
        active_parameter_bindings: BTreeMap::new(),
        active_target: None,
        block_effects: BTreeMap::new(),
        pending_returns: Vec::new(),
        pending_writes: BTreeMap::new(),
        summaries: Vec::new(),
    };

    if target_callable.is_none() || include_top_level {
        let mut top_state = FlowState::default();
        let _ = analyzer.analyze_statements(&program.statements, &mut top_state, None, None, None);
        analyzer.parameter_facts_from_events();
    }

    for class in surface.classes.values() {
        if target_callable.is_none() {
            for field in class.fields.values() {
                if let Some(initializer) = field_initializer(source, field.ast) {
                    let state = FlowState::default();
                    let value = analyzer.value(initializer, &state, Some(&class.id), Some(field_side(field.is_class_side)));
                    analyzer.field_facts.record_evidence(
                        class.id.clone(),
                        field.name.clone(),
                        field_side(field.is_class_side),
                        super::facts::FieldEvidenceKind::DeclarationInitializer,
                        initializer.range(),
                        value,
                    );
                }
            }
        }

        for member in class.members_by_side.values() {
            if target_callable.is_some_and(|target| target != &member.callable) {
                continue;
            }
            counters.callables_analyzed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let body = member_body(source, member.ast);
            if body.is_empty() {
                continue;
            }
            analyzer.events.clear();
            analyzer.dynamic_send = false;
            analyzer.invoked_parameters.clear();
            analyzer.active_parameter_bindings.clear();
            analyzer.active_target = Some(member.callable.clone());
            analyzer.pending_returns.clear();
            analyzer.pending_writes.clear();
            let mut state = analyzer.seed_member(member);
            let flow = analyzer.analyze_statements(body, &mut state, Some(&class.id), Some(member.side), Some(&member.callable));
            let mut return_values = flow.returns.iter().map(|evidence| evidence.value.clone()).collect::<Vec<_>>();
            if let Some(tail) = flow.normal.as_ref().and_then(|_| flow.tail_value.clone()) {
                return_values.push(tail);
            }
            let has_evidence = !return_values.is_empty();
            let returns = if member.is_constructor {
                InferredValue::flow(ValueShape::Instance(class.id.clone()), member.source_range)
            } else if has_evidence {
                join_reachable_values(return_values)
            } else {
                InferredValue::flow(ValueShape::Unknown, member.source_range)
            };
            let dependencies = analyzer
                .events
                .iter()
                .filter_map(|event| match event {
                    AnalysisEvent::Call(call) if !call.dynamic => Some(call.target.clone()),
                    AnalysisEvent::Call(_) | AnalysisEvent::FieldWrite { .. } => None,
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            analyzer.parameter_facts_from_events();
            analyzer.summaries.push((
                CallableSummary {
                    callable: member.callable.clone(),
                    params: member
                        .params
                        .iter()
                        .map(|param| {
                            (analyzer.parameter_fact)(&member.callable, &param.name)
                                .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, param.source_range))
                        })
                        .collect(),
                    returns,
                    dependencies,
                    effects: super::callable::SummaryEffects {
                        dynamic_send: analyzer.dynamic_send,
                        invokes_parameters: analyzer.invoked_parameters.clone(),
                    },
                    revision,
                },
                has_evidence || member.is_constructor,
            ));
            analyzer.active_target = None;
        }
    }

    SurfaceFlowAnalysis {
        local_facts: analyzer.local_facts,
        field_facts: analyzer.field_facts,
        parameter_facts: analyzer.parameter_facts,
        summaries: analyzer.summaries,
    }
}

struct FlowAnalyzer<'ctx> {
    surface: &'ctx ModuleSurface,
    scopes: &'ctx ScopeGraph,
    known_class: &'ctx dyn Fn(&str) -> Option<ClassId>,
    contains_class: &'ctx dyn Fn(&ClassId) -> bool,
    callable_return: &'ctx dyn Fn(&CallableId) -> Option<InferredValue>,
    parameter_fact: &'ctx dyn Fn(&CallableId, &str) -> Option<InferredValue>,
    field_value: &'ctx dyn Fn(&ClassId, &str, DispatchSide) -> Option<InferredValue>,
    resolve_member: &'ctx dyn Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch>,
    callable_effects: &'ctx dyn Fn(&CallableId) -> Option<super::callable::SummaryEffects>,
    member_surface: &'ctx dyn Fn(&CallableId) -> Option<MemberSurface>,
    is_same_or_subclass: &'ctx dyn Fn(&ClassId, &ClassId) -> bool,
    local_facts: LocalFacts,
    field_facts: FieldFacts,
    parameter_facts: ParameterFacts,
    events: Vec<AnalysisEvent>,
    dynamic_send: bool,
    invoked_parameters: BTreeSet<usize>,
    active_parameter_bindings: BTreeMap<BindingId, usize>,
    active_target: Option<CallableId>,
    block_effects: BTreeMap<(usize, usize, Vec<(BindingId, ValueShape)>), BlockEffects>,
    pending_returns: Vec<ReturnEvidence>,
    pending_writes: BTreeMap<BindingId, InferredValue>,
    summaries: Vec<(CallableSummary, bool)>,
}

impl FlowAnalyzer<'_> {
    fn seed_member(&mut self, member: &MemberSurface) -> FlowState {
        let mut state = FlowState::default();
        for (index, param) in member.params.iter().enumerate() {
            let value = (self.parameter_fact)(&member.callable, &param.name)
                .or_else(|| self.parameter_facts.get(&member.callable, &param.name).cloned())
                .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, param.source_range));
            if let Some(binding) = self.scopes.binding_for_declaration(param.name_range) {
                state.bindings.insert(binding, value.clone());
                self.local_facts.record(binding, param.name_range, value);
                self.active_parameter_bindings.insert(binding, index);
            }
        }
        state
    }

    fn value(&self, expr: &Expr, state: &FlowState, current_class: Option<&ClassId>, side: Option<DispatchSide>) -> InferredValue {
        let environment = BTreeMap::new();
        let contains_class = |class: &ClassId| (self.contains_class)(class);
        let resolve_member = |receiver: &DispatchReceiver, selector: &str| (self.resolve_member)(receiver, selector);
        let field_value = |class: &ClassId, name: &str, field_side: DispatchSide| {
            self.field_facts
                .get(class, name, field_side)
                .cloned()
                .or_else(|| (self.field_value)(class, name, field_side))
        };
        let context = AnalysisContext {
            current_class,
            dispatch_side: side,
            query_offset: 0,
            environment: &environment,
            local_facts: None,
            binding_values: Some(&state.bindings),
            scopes: Some(self.scopes),
            known_class: self.known_class,
            callable_return: self.callable_return,
            field_value: &field_value,
            resolver: &resolve_member,
            member_surface: self.member_surface,
            contains_class: &contains_class,
            is_same_or_subclass: self.is_same_or_subclass,
        };
        analyze_expr(expr, &context)
    }

    fn eval(&mut self, expr: &Expr, state: &FlowState, current_class: Option<&ClassId>, side: Option<DispatchSide>) -> InferredValue {
        let value = self.value(expr, state, current_class, side);
        self.collect_events(expr, state, current_class, side);
        value
    }

    fn analyze_statements(
        &mut self,
        statements: &[Statement],
        state: &mut FlowState,
        current_class: Option<&ClassId>,
        side: Option<DispatchSide>,
        target: Option<&CallableId>,
    ) -> StatementFlow {
        let mut result = StatementFlow {
            normal: Some(state.clone()),
            ..StatementFlow::default()
        };
        for statement in statements {
            let Some(mut current) = result.normal.take() else {
                // Keep collecting explicit return evidence for summary joins
                // even when the preceding statement terminated one path.
                let mut unreachable = FlowState::default();
                let step = self.analyze_statement(statement, &mut unreachable, current_class, side, target);
                result.returns.extend(step.returns);
                result.returns.append(&mut self.pending_returns);
                self.pending_writes.clear();
                result.throws |= step.throws;
                continue;
            };
            let step = self.analyze_statement(statement, &mut current, current_class, side, target);
            result.returns.extend(step.returns);
            result.returns.append(&mut self.pending_returns);
            let mut normal = step.normal;
            if let Some(normal) = &mut normal {
                self.apply_pending_writes(normal);
            } else {
                self.pending_writes.clear();
            }
            result.breaks.extend(step.breaks);
            result.continues.extend(step.continues);
            result.throws |= step.throws;
            result.tail_value = step.tail_value;
            result.normal = normal;
        }
        if let Some(normal) = &result.normal {
            *state = normal.clone();
        }
        result
    }

    fn analyze_statement(
        &mut self,
        statement: &Statement,
        state: &mut FlowState,
        current_class: Option<&ClassId>,
        side: Option<DispatchSide>,
        target: Option<&CallableId>,
    ) -> StatementFlow {
        match statement {
            Statement::Let(binding) => {
                let value = binding
                    .value
                    .as_ref()
                    .map(|expr| self.eval(expr, state, current_class, side))
                    .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, binding.range));
                self.bind_pattern(&binding.pattern, &value, state);
                StatementFlow {
                    normal: Some(state.clone()),
                    ..StatementFlow::default()
                }
            }
            Statement::Expr { expr, .. } => {
                if let Some(flow) = self.analyze_control_expression(expr, state, current_class, side, target) {
                    return flow;
                }
                let value = self.eval(expr, state, current_class, side);
                self.apply_assignment(expr, state, current_class, side, target);
                StatementFlow {
                    normal: Some(state.clone()),
                    tail_value: Some(value),
                    ..StatementFlow::default()
                }
            }
            Statement::Return(return_statement) => {
                let value = return_statement
                    .value
                    .as_ref()
                    .map(|expr| self.eval(expr, state, current_class, side))
                    .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, return_statement.range));
                let returns = target
                    .cloned()
                    .map(|target| {
                        vec![ReturnEvidence {
                            target,
                            value,
                            range: return_statement.range,
                        }]
                    })
                    .unwrap_or_default();
                StatementFlow {
                    normal: None,
                    returns,
                    ..StatementFlow::default()
                }
            }
            Statement::For(for_statement) => self.analyze_for(for_statement, state, current_class, side, target),
            Statement::Throw { expr, .. } => {
                self.eval(expr, state, current_class, side);
                StatementFlow {
                    normal: None,
                    throws: true,
                    ..StatementFlow::default()
                }
            }
            Statement::Break { .. } => StatementFlow {
                normal: None,
                breaks: vec![state.clone()],
                ..StatementFlow::default()
            },
            Statement::Continue { .. } => StatementFlow {
                normal: None,
                continues: vec![state.clone()],
                ..StatementFlow::default()
            },
            Statement::Class(_) | Statement::Import(_) => StatementFlow {
                normal: Some(state.clone()),
                ..StatementFlow::default()
            },
        }
    }

    fn analyze_control_expression(
        &mut self,
        expr: &Expr,
        state: &mut FlowState,
        current_class: Option<&ClassId>,
        side: Option<DispatchSide>,
        target: Option<&CallableId>,
    ) -> Option<StatementFlow> {
        match expr {
            Expr::MethodCall(call) => {
                let selector = call_selector(&call.method, &call.args);
                let receiver = self.value(&call.object, state, current_class, side);
                let arguments = self.arguments(&call.args, state, current_class, side);
                self.emit_send(
                    &call.object,
                    &receiver,
                    &selector,
                    arguments,
                    has_dynamic_pack(&call.args),
                    call.range,
                    state,
                    current_class,
                    side,
                );
                self.collect_events(&call.object, state, current_class, side);

                match selector.as_str() {
                    "ifTrue(_)" => {
                        let block = positional_block(&call.args, 0)?;
                        Some(self.analyze_if(
                            state,
                            Some(&call.object),
                            receiver.known_boolean,
                            Some(block),
                            None,
                            current_class,
                            side,
                            target,
                        ))
                    }
                    "ifFalse(_)" => {
                        let block = positional_block(&call.args, 0)?;
                        Some(self.analyze_if(
                            state,
                            Some(&call.object),
                            receiver.known_boolean,
                            None,
                            Some(block),
                            current_class,
                            side,
                            target,
                        ))
                    }
                    "ifTrue(_:ifFalse:)" => {
                        let then_block = positional_block(&call.args, 0)?;
                        let else_block = labeled_block(&call.args, "ifFalse")?;
                        Some(self.analyze_if(
                            state,
                            Some(&call.object),
                            receiver.known_boolean,
                            Some(then_block),
                            Some(else_block),
                            current_class,
                            side,
                            target,
                        ))
                    }
                    "whileTrue(_)" => {
                        let Expr::Block(condition) = &call.object else { return None };
                        let body = positional_block(&call.args, 0)?;
                        Some(self.analyze_while(state, condition, body, current_class, side, target))
                    }
                    _ => None,
                }
            }
            Expr::Binary(binary) if matches!(binary.op, BinaryOp::And | BinaryOp::Or) => {
                let Expr::Block(block) = &binary.right else { return None };
                let left = self.value(&binary.left, state, current_class, side);
                let block_effect = self.ensure_block_effect(block, state, current_class, side);
                let argument = vec![AnalyzedArgument {
                    label: None,
                    value: self.value(&binary.right, state, current_class, side),
                    binding: None,
                    block_effect: Some(block_effect),
                    range: binary.right.range(),
                }];
                let selector = if matches!(binary.op, BinaryOp::And) { "and(_)" } else { "or(_)" };
                self.emit_send(&binary.left, &left, selector, argument, false, binary.range, state, current_class, side);
                self.collect_events(&binary.left, state, current_class, side);
                Some(self.analyze_if(state, None, None, None, Some(block), current_class, side, target))
            }
            _ => None,
        }
    }

    fn analyze_if(
        &mut self,
        state: &mut FlowState,
        condition: Option<&Expr>,
        condition_truth: Option<bool>,
        then_block: Option<&BlockExpr>,
        else_block: Option<&BlockExpr>,
        current_class: Option<&ClassId>,
        side: Option<DispatchSide>,
        target: Option<&CallableId>,
    ) -> StatementFlow {
        let then_state = condition
            .map(|condition| self.refine_condition_state(condition, state, true, current_class, side))
            .unwrap_or_else(|| Some(state.clone()));
        let else_state = condition
            .map(|condition| self.refine_condition_state(condition, state, false, current_class, side))
            .unwrap_or_else(|| Some(state.clone()));
        let then_flow = if condition_truth != Some(false) {
            then_state
                .as_ref()
                .and_then(|state| then_block.map(|block| self.analyze_invoked_block(block, state, current_class, side, target)))
        } else {
            None
        };
        let else_flow = if condition_truth != Some(true) {
            else_state
                .as_ref()
                .and_then(|state| else_block.map(|block| self.analyze_invoked_block(block, state, current_class, side, target)))
        } else {
            None
        };
        let normal = match condition_truth {
            Some(true) => match then_block {
                Some(_) => then_flow.as_ref().and_then(|flow| {
                    then_state
                        .as_ref()
                        .and_then(|entry| flow.normal.as_ref().map(|state| project_outer_state(entry, state)))
                }),
                None => Some(state.clone()),
            },
            Some(false) => match else_block {
                Some(_) => else_flow.as_ref().and_then(|flow| {
                    else_state
                        .as_ref()
                        .and_then(|entry| flow.normal.as_ref().map(|state| project_outer_state(entry, state)))
                }),
                None => Some(state.clone()),
            },
            None => match (
                then_flow.as_ref().and_then(|flow| flow.normal.as_ref()),
                else_flow.as_ref().and_then(|flow| flow.normal.as_ref()),
            ) {
                (Some(then_state), Some(else_state)) => Some(join_states(&project_outer_state(state, then_state), &project_outer_state(state, else_state))),
                (Some(then_state), None) => Some(join_states(state, &project_outer_state(state, then_state))),
                (None, Some(else_state)) => Some(join_states(&project_outer_state(state, else_state), state)),
                (None, None) => Some(state.clone()),
            },
        };
        let mut returns = Vec::new();
        let mut breaks = Vec::new();
        let mut continues = Vec::new();
        let mut throws = false;
        let mut tails = Vec::new();
        for flow in then_flow.into_iter().chain(else_flow) {
            returns.extend(flow.returns);
            breaks.extend(flow.breaks);
            continues.extend(flow.continues);
            throws |= flow.throws;
            if let Some(tail) = flow.tail_value {
                tails.push(tail);
            }
        }
        if condition_truth.is_none() && else_block.is_none() {
            tails.push(InferredValue::flow(ValueShape::Unknown, Default::default()));
        }
        StatementFlow {
            normal,
            returns,
            breaks,
            continues,
            throws,
            tail_value: (!tails.is_empty()).then(|| join_values(tails)),
        }
    }

    fn refine_condition_state(
        &self,
        condition: &Expr,
        state: &FlowState,
        truth: bool,
        current_class: Option<&ClassId>,
        side: Option<DispatchSide>,
    ) -> Option<FlowState> {
        if let Expr::Unary(unary) = condition
            && matches!(unary.op, UnaryOp::Not)
        {
            return self.refine_condition_state(&unary.expr, state, !truth, current_class, side);
        }

        let Expr::MethodCall(call) = condition else { return Some(state.clone()) };
        let is_exact = match call.method.as_str() {
            "is" => false,
            "isExactly" => true,
            _ => return Some(state.clone()),
        };
        if call.args.len() != 1 {
            return Some(state.clone());
        }
        let Expr::Var {
            value: name,
            range: name_range,
        } = &call.object
        else {
            return Some(state.clone());
        };
        let Some(binding) = self.binding_for_name(name, *name_range, state) else {
            return Some(state.clone());
        };
        let Some(PackItem::Positional { expr: rhs, .. }) = call.args.first() else {
            return Some(state.clone());
        };
        let ValueShape::ClassObject(target) = self.value(rhs, state, current_class, side).shape else {
            return Some(state.clone());
        };
        let Some(current) = state.bindings.get(&binding) else {
            return Some(state.clone());
        };
        let shape = narrow_type_test_shape(&current.shape, &target, is_exact, truth, self.is_same_or_subclass)?;
        let mut refined = state.clone();
        refined.bindings.insert(binding, InferredValue::flow(shape, condition.range()));
        Some(refined)
    }

    fn analyze_while(
        &mut self,
        state: &mut FlowState,
        condition: &BlockExpr,
        body: &BlockExpr,
        current_class: Option<&ClassId>,
        side: Option<DispatchSide>,
        target: Option<&CallableId>,
    ) -> StatementFlow {
        let entry = state.clone();
        let mut header = entry.clone();
        let mut exits = vec![entry.clone()];
        let mut returns = Vec::new();
        let mut throws = false;
        let mut tail_values = Vec::new();

        for iteration in 0..MAX_FLOW_FIXPOINT_ITERS {
            let condition_flow = self.analyze_invoked_block(condition, &header, current_class, side, target);
            returns.extend(condition_flow.returns);
            throws |= condition_flow.throws;
            let Some(condition_state) = condition_flow.normal else { break };
            let condition_state = project_outer_state(&entry, &condition_state);
            exits.push(condition_state.clone());

            let body_flow = self.analyze_invoked_block(body, &condition_state, current_class, side, target);
            returns.extend(body_flow.returns);
            throws |= body_flow.throws;
            if let Some(tail) = body_flow.tail_value {
                tail_values.push(tail);
            }
            for break_state in body_flow.breaks {
                exits.push(project_outer_state(&entry, &break_state));
            }
            let mut back_edges = Vec::new();
            if let Some(normal) = body_flow.normal {
                let normal = project_outer_state(&entry, &normal);
                exits.push(normal.clone());
                back_edges.push(normal);
            }
            back_edges.extend(body_flow.continues.into_iter().map(|state| project_outer_state(&entry, &state)));
            let Some(next_header) = join_states_many(back_edges) else { break };
            if next_header == header {
                exits.push(header.clone());
                break;
            }
            if iteration + 1 == MAX_FLOW_FIXPOINT_ITERS {
                header = widen_loop_state(&entry, &header, &next_header);
                exits.push(header.clone());
                break;
            }
            header = next_header;
        }

        let normal = join_states_many(exits).or_else(|| Some(entry.clone()));
        let mut tails = vec![InferredValue::flow(ValueShape::Unknown, Default::default())];
        tails.extend(tail_values);
        StatementFlow {
            normal,
            returns,
            throws,
            tail_value: Some(join_values(tails)),
            ..StatementFlow::default()
        }
    }

    fn analyze_invoked_block(
        &mut self,
        block: &BlockExpr,
        state: &FlowState,
        current_class: Option<&ClassId>,
        side: Option<DispatchSide>,
        target: Option<&CallableId>,
    ) -> StatementFlow {
        let mut block_state = state.clone();
        self.seed_block_parameters(block, &mut block_state);
        self.analyze_statements(&block.body, &mut block_state, current_class, side, target)
    }

    fn seed_block_parameters(&mut self, block: &BlockExpr, state: &mut FlowState) {
        for parameter in block.params.fixed.iter().chain(block.params.positional_rest.iter()) {
            if let Some(binding) = self.scopes.binding_for_declaration(parameter.range) {
                let value = InferredValue::flow(ValueShape::Unknown, parameter.range);
                state.bindings.insert(binding, value.clone());
                self.local_facts.record(binding, parameter.range, value);
            }
        }
    }

    fn analyze_for(
        &mut self,
        statement: &phalcom_ast::ast::ForStatement,
        state: &mut FlowState,
        current_class: Option<&ClassId>,
        side: Option<DispatchSide>,
        target: Option<&CallableId>,
    ) -> StatementFlow {
        let iterable = self.eval(&statement.iter, state, current_class, side);
        let entry = state.clone();
        let mut header = entry.clone();
        let mut exits = vec![entry.clone()];
        let mut returns = Vec::new();
        let mut throws = false;
        let binding = self.scopes.binding_for_declaration(statement.binding_range);
        let element = InferredValue::flow(iterable.shape.element_shape(), statement.binding_range);

        for iteration in 0..MAX_FLOW_FIXPOINT_ITERS {
            let mut loop_state = header.clone();
            if let Some(binding) = binding {
                loop_state.bindings.insert(binding, element.clone());
                self.local_facts.record(binding, statement.binding_range, element.clone());
            }
            let body = self.analyze_statements(&statement.body, &mut loop_state, current_class, side, target);
            returns.extend(body.returns);
            throws |= body.throws;
            let mut back_edges = Vec::new();
            if let Some(normal) = body.normal {
                let normal = project_outer_state(&entry, &normal);
                exits.push(normal.clone());
                back_edges.push(normal);
            }
            back_edges.extend(body.continues.into_iter().map(|state| project_outer_state(&entry, &state)));
            for break_state in body.breaks {
                exits.push(project_outer_state(&entry, &break_state));
            }
            let Some(next_header) = join_states_many(back_edges) else { break };
            if next_header == header {
                exits.push(header.clone());
                break;
            }
            if iteration + 1 == MAX_FLOW_FIXPOINT_ITERS {
                header = widen_loop_state(&entry, &header, &next_header);
                exits.push(header.clone());
                break;
            }
            header = next_header;
        }

        StatementFlow {
            normal: join_states_many(exits).or_else(|| Some(entry)),
            returns,
            throws,
            ..StatementFlow::default()
        }
    }

    fn bind_pattern(&mut self, pattern: &Pattern, value: &InferredValue, state: &mut FlowState) {
        match pattern {
            Pattern::Name { range, .. } => {
                if let Some(binding) = self.scopes.binding_for_declaration(*range) {
                    let fact = InferredValue::flow(value.shape.clone(), *range);
                    state.bindings.insert(binding, fact.clone());
                    self.local_facts.record(binding, *range, fact);
                }
            }
            Pattern::Tuple { elements, .. } | Pattern::List { elements, .. } => {
                for (index, element) in elements.iter().enumerate() {
                    let shape = match &value.shape {
                        ValueShape::Tuple(values) => values.get(index).cloned().unwrap_or(ValueShape::Unknown),
                        ValueShape::List(element) => (**element).clone(),
                        _ => ValueShape::Unknown,
                    };
                    self.bind_pattern(element, &InferredValue::flow(shape, element.range()), state);
                }
                if let Pattern::List { rest: Some(rest), .. } = pattern {
                    self.bind_pattern(rest, &InferredValue::flow(ValueShape::List(Box::new(ValueShape::Unknown)), rest.range()), state);
                }
            }
        }
    }

    fn apply_assignment(
        &mut self,
        expr: &Expr,
        state: &mut FlowState,
        current_class: Option<&ClassId>,
        side: Option<DispatchSide>,
        target: Option<&CallableId>,
    ) {
        let Expr::Assignment(assignment) = expr else { return };
        let value = self.value(&assignment.value, state, current_class, side);
        match assignment.name.as_ref() {
            Expr::Var { value: name, range } => {
                let binding = self.scopes.resolve(self.scopes.scope_at(range.start), name, range.start);
                let super::scope::NameResolution::Binding(binding) = binding else { return };
                if self.scopes.bindings.get(&binding).is_some_and(|info| info.mutable) {
                    let fact = InferredValue::flow(value.shape, *range);
                    state.bindings.insert(binding, fact.clone());
                    self.local_facts.record(binding, *range, fact);
                }
            }
            Expr::Field { value: name, range, .. } => {
                let Some(class) = current_class else { return };
                let field_side = self
                    .surface
                    .classes
                    .get(class)
                    .and_then(|surface| surface.fields.get(name))
                    .map(|field| field_side(field.is_class_side))
                    .unwrap_or(DispatchSide::Instance);
                let fact = InferredValue::flow(value.shape, *range);
                let field = FieldId {
                    owner: class.clone(),
                    name: name.clone(),
                    side: field_side,
                };
                let kind = target
                    .and_then(|callable| {
                        self.surface
                            .classes
                            .get(class)
                            .and_then(|surface| surface.members_by_side.get(&(callable.selector.clone(), callable.side)))
                    })
                    .filter(|member| member.is_constructor)
                    .map_or(super::facts::FieldEvidenceKind::GeneralWrite, |_| {
                        super::facts::FieldEvidenceKind::ConstructorInitialization
                    });
                self.field_facts
                    .record_evidence(class.clone(), name.clone(), field_side, kind, *range, fact.clone());
                self.events.push(AnalysisEvent::FieldWrite {
                    field,
                    value: fact,
                    site: *range,
                });
            }
            _ => {}
        }
    }

    fn context_value(&self, expr: &Expr, state: &FlowState, current_class: Option<&ClassId>, side: Option<DispatchSide>) -> InferredValue {
        self.value(expr, state, current_class, side)
    }

    fn collect_events(&mut self, expr: &Expr, state: &FlowState, current_class: Option<&ClassId>, side: Option<DispatchSide>) {
        match expr {
            Expr::Unary(unary) => {
                let value = self.context_value(&unary.expr, state, current_class, side);
                let selector = format!("{}()", unary_selector_name(&unary.op));
                self.emit_send(&unary.expr, &value, &selector, Vec::new(), false, unary.range, state, current_class, side);
                self.collect_events(&unary.expr, state, current_class, side);
            }
            Expr::Binary(binary) => {
                let left = self.context_value(&binary.left, state, current_class, side);
                let right = self.context_value(&binary.right, state, current_class, side);
                let selector = match binary.op {
                    BinaryOp::And => Some("and(_)".to_string()),
                    BinaryOp::Or => Some("or(_)".to_string()),
                    _ => binary_selector_name(&binary.op).map(|name| format!("{name}(_)")),
                };
                if let Some(selector) = selector {
                    self.emit_send(
                        &binary.left,
                        &left,
                        &selector,
                        vec![AnalyzedArgument {
                            label: None,
                            value: right,
                            binding: None,
                            block_effect: None,
                            range: binary.right.range(),
                        }],
                        matches!(binary.op, BinaryOp::And | BinaryOp::Or),
                        binary.range,
                        state,
                        current_class,
                        side,
                    );
                }
                self.collect_events(&binary.left, state, current_class, side);
                self.collect_events(&binary.right, state, current_class, side);
            }
            Expr::UnqualifiedCall(call) => {
                let args = self.arguments(&call.args, state, current_class, side);
                let selector = call_selector(&call.name, &call.args);
                if has_dynamic_pack(&call.args) {
                    self.collect_pack_events(&call.args, state, current_class, side);
                    return;
                }

                let binding_id = call.name_range.and_then(|range| self.binding_for_name(&call.name, range, state));
                if let Some(binding) = binding_id
                    && let Some(parameter) = self.active_parameter_bindings.get(&binding)
                {
                    self.invoked_parameters.insert(*parameter);
                }
                let binding_value = binding_id.and_then(|binding| state.bindings.get(&binding));
                match binding_value.map(|value| &value.shape) {
                    Some(ValueShape::Callable(target)) => {
                        self.record_call(target.clone(), call.range, args, state);
                    }
                    Some(ValueShape::Family { receiver, .. }) => {
                        for target in receiver_targets(expr, receiver, current_class, side) {
                            let Some(resolved) = (self.resolve_member)(&target, &selector) else {
                                continue;
                            };
                            self.record_call(resolved.callable, call.range, args.clone(), state);
                        }
                    }
                    Some(_) => {}
                    None => {
                        let target = current_class.map(|class| match side.unwrap_or(DispatchSide::Instance) {
                            DispatchSide::Instance => DispatchReceiver::Instance(class.clone()),
                            DispatchSide::Class => DispatchReceiver::ClassObject(class.clone()),
                        });
                        if let Some(target) = target {
                            if let Some(resolved) = (self.resolve_member)(&target, &selector) {
                                self.record_call(resolved.callable, call.range, args, state);
                            }
                        } else {
                            let mut candidates = self
                                .surface
                                .classes
                                .values()
                                .flat_map(|class| class.members_by_side.values())
                                .filter(|member| member.callable.selector == selector);
                            if let Some(member) = candidates.next()
                                && candidates.next().is_none()
                            {
                                self.record_call(member.callable.clone(), call.range, args, state);
                            }
                        }
                    }
                }
                self.collect_pack_events(&call.args, state, current_class, side);
            }
            Expr::MethodCall(call) => {
                let receiver = self.context_value(&call.object, state, current_class, side);
                let args = self.arguments(&call.args, state, current_class, side);
                let selector = call_selector(&call.method, &call.args);
                if let Expr::Var { value, range } = &call.object
                    && let Some(binding) = self.binding_for_name(value, *range, state)
                    && let Some(parameter) = self.active_parameter_bindings.get(&binding)
                    && selector.starts_with("call(")
                {
                    self.invoked_parameters.insert(*parameter);
                }
                self.emit_send(
                    &call.object,
                    &receiver,
                    &selector,
                    args,
                    has_dynamic_pack(&call.args),
                    call.range,
                    state,
                    current_class,
                    side,
                );
                self.collect_events(&call.object, state, current_class, side);
                self.collect_pack_events(&call.args, state, current_class, side);
            }
            Expr::GetProperty(property) => {
                let receiver = self.context_value(&property.object, state, current_class, side);
                self.emit_send(
                    &property.object,
                    &receiver,
                    &property.property,
                    Vec::new(),
                    false,
                    property.range,
                    state,
                    current_class,
                    side,
                );
                self.collect_events(&property.object, state, current_class, side);
            }
            Expr::SetProperty(property) => {
                let receiver = self.context_value(&property.object, state, current_class, side);
                let argument = self.analyzed_argument(None, &property.value, property.value.range(), state, current_class, side);
                self.emit_send(
                    &property.object,
                    &receiver,
                    &setter_selector_from_name(&property.property),
                    vec![argument],
                    false,
                    property.range,
                    state,
                    current_class,
                    side,
                );
                self.collect_events(&property.object, state, current_class, side);
                self.collect_events(&property.value, state, current_class, side);
            }
            Expr::Index(index) => {
                let receiver = self.context_value(&index.object, state, current_class, side);
                let args = self.arguments(&index.args, state, current_class, side);
                let selector = index_selector_from_labels(&static_labels(&index.args), false);
                self.emit_send(
                    &index.object,
                    &receiver,
                    &selector,
                    args,
                    has_dynamic_pack(&index.args),
                    index.range,
                    state,
                    current_class,
                    side,
                );
                self.collect_events(&index.object, state, current_class, side);
                self.collect_pack_events(&index.args, state, current_class, side);
            }
            Expr::SetIndex(index) => {
                let receiver = self.context_value(&index.object, state, current_class, side);
                let mut args = self.arguments(&index.args, state, current_class, side);
                args.push(self.analyzed_argument(None, &index.value, index.value.range(), state, current_class, side));
                let selector = index_selector_from_labels(&static_labels(&index.args), true);
                self.emit_send(
                    &index.object,
                    &receiver,
                    &selector,
                    args,
                    has_dynamic_pack(&index.args),
                    index.range,
                    state,
                    current_class,
                    side,
                );
                self.collect_events(&index.object, state, current_class, side);
                self.collect_pack_events(&index.args, state, current_class, side);
                self.collect_events(&index.value, state, current_class, side);
            }
            Expr::Assignment(assignment) => {
                self.collect_events(&assignment.name, state, current_class, side);
                self.collect_events(&assignment.value, state, current_class, side);
            }
            Expr::Block(block) => self.collect_block_facts(block, state, current_class, side),
            Expr::MethodRef(reference) => {
                self.collect_events(&reference.receiver, state, current_class, side);
                if let MethodRefKind::Pinned { name, labels } = &reference.kind {
                    let receiver = self.value(&reference.receiver, state, current_class, side);
                    let selector = crate::selectors::comma_form_from_labels(name, labels);
                    for target in receiver_targets(&reference.receiver, &receiver.shape, current_class, side) {
                        if let Some(resolved) = (self.resolve_member)(&target, &selector) {
                            self.events.push(AnalysisEvent::Call(ResolvedCall {
                                target: resolved.callable,
                                site: reference.range,
                                args: Vec::new(),
                                dynamic: false,
                            }));
                        }
                    }
                }
            }
            Expr::Range(range) => {
                if let Some(lower) = &range.lower {
                    self.collect_events(lower, state, current_class, side);
                }
                if let Some(upper) = &range.upper {
                    self.collect_events(upper, state, current_class, side);
                }
            }
            Expr::TupleLiteral(tuple) => {
                for entry in &tuple.entries {
                    let expr = match entry {
                        TupleLiteralEntry::Positional { expr, .. }
                        | TupleLiteralEntry::Labeled { value: expr, .. }
                        | TupleLiteralEntry::Expand { expr, .. } => expr,
                    };
                    self.collect_events(expr, state, current_class, side);
                }
            }
            Expr::RecordLiteral(record) => {
                for entry in &record.entries {
                    match entry {
                        phalcom_ast::ast::RecordLiteralEntry::Field(field) => self.collect_events(&field.value, state, current_class, side),
                        phalcom_ast::ast::RecordLiteralEntry::Expansion { expr, .. } => self.collect_events(expr, state, current_class, side),
                    }
                }
            }
            Expr::MapLiteral(map) => {
                for entry in &map.entries {
                    match entry {
                        MapLiteralEntry::Association { key, value, .. } => {
                            if let MapLiteralKey::Computed { expr, .. } = key {
                                self.collect_events(expr, state, current_class, side);
                            }
                            self.collect_events(value, state, current_class, side);
                        }
                        MapLiteralEntry::Expansion { expr, .. } => self.collect_events(expr, state, current_class, side),
                    }
                }
            }
            Expr::SetLiteral(set) => {
                for entry in &set.entries {
                    let expr = match entry {
                        SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => expr,
                    };
                    self.collect_events(expr, state, current_class, side);
                }
            }
            Expr::ListLiteral(list) => {
                for entry in &list.elements {
                    let expr = match entry {
                        ListLiteralElement::Element { expr, .. } | ListLiteralElement::Expansion { expr, .. } => expr,
                    };
                    self.collect_events(expr, state, current_class, side);
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

    fn collect_block_facts(&mut self, block: &BlockExpr, state: &FlowState, current_class: Option<&ClassId>, side: Option<DispatchSide>) {
        let _ = self.ensure_block_effect(block, state, current_class, side);
    }

    fn ensure_block_effect(&mut self, block: &BlockExpr, state: &FlowState, current_class: Option<&ClassId>, side: Option<DispatchSide>) -> BlockEffects {
        let key = (
            block.range.start,
            block.range.end,
            state.bindings.iter().map(|(binding, value)| (*binding, value.shape.clone())).collect(),
        );
        if let Some(effect) = self.block_effects.get(&key) {
            return effect.clone();
        }
        let event_len = self.events.len();
        let local_facts = self.local_facts.checkpoint();
        let field_facts = self.field_facts.clone();
        let parameter_facts = self.parameter_facts.clone();
        let dynamic_send = self.dynamic_send;
        let invoked_parameters = self.invoked_parameters.clone();
        let pending_returns = self.pending_returns.clone();
        let pending_writes = self.pending_writes.clone();
        let mut block_state = state.clone();
        self.seed_block_parameters(block, &mut block_state);
        let target = self.active_target.clone();
        let flow = self.analyze_statements(&block.body, &mut block_state, current_class, side, target.as_ref());
        let captured_writes = state
            .bindings
            .iter()
            .filter_map(|(binding, before)| {
                let after = block_state.bindings.get(binding)?;
                (after != before).then_some((*binding, after.clone()))
            })
            .collect();
        let effect = BlockEffects {
            nonlocal_returns: flow.returns,
            captured_writes,
            invokes_parameters: self.invoked_parameters.difference(&invoked_parameters).copied().collect(),
            dynamic_send: self.dynamic_send && !dynamic_send,
        };
        self.events.truncate(event_len);
        self.local_facts.rollback(&local_facts);
        self.field_facts = field_facts;
        self.parameter_facts = parameter_facts;
        self.dynamic_send = dynamic_send;
        self.invoked_parameters = invoked_parameters;
        self.pending_returns = pending_returns;
        self.pending_writes = pending_writes;
        self.block_effects.insert(key, effect.clone());
        effect
    }

    fn arguments(&mut self, args: &[PackItem], state: &FlowState, current_class: Option<&ClassId>, side: Option<DispatchSide>) -> Vec<AnalyzedArgument> {
        args.iter()
            .map(|arg| match arg {
                PackItem::Positional { expr, range } => self.analyzed_argument(None, expr, *range, state, current_class, side),
                PackItem::Labeled { label, value, range } => self.analyzed_argument(
                    match label {
                        PackLabel::Static { text, .. } => Some(text.clone()),
                        PackLabel::Computed { .. } => None,
                    },
                    value,
                    *range,
                    state,
                    current_class,
                    side,
                ),
                PackItem::Expand { expr, range, .. } => self.analyzed_argument(None, expr, *range, state, current_class, side),
            })
            .collect()
    }

    fn analyzed_argument(
        &mut self,
        label: Option<String>,
        expr: &Expr,
        range: SourceRange,
        state: &FlowState,
        current_class: Option<&ClassId>,
        side: Option<DispatchSide>,
    ) -> AnalyzedArgument {
        let block_effect = match expr {
            Expr::Block(block) => Some(self.ensure_block_effect(block, state, current_class, side)),
            _ => None,
        };
        let binding = match expr {
            Expr::Var { value, range } => self.binding_for_name(value, *range, state),
            _ => None,
        };
        AnalyzedArgument {
            label,
            value: self.value(expr, state, current_class, side),
            binding,
            block_effect,
            range,
        }
    }

    fn binding_for_name(&self, name: &str, range: SourceRange, _state: &FlowState) -> Option<BindingId> {
        match self.scopes.resolve(self.scopes.scope_at(range.start), name, range.start) {
            super::scope::NameResolution::Binding(binding) => Some(binding),
            _ => None,
        }
    }

    fn collect_pack_events(&mut self, args: &[PackItem], state: &FlowState, current_class: Option<&ClassId>, side: Option<DispatchSide>) {
        for arg in args {
            let expr = match arg {
                PackItem::Positional { expr, .. } | PackItem::Expand { expr, .. } | PackItem::Labeled { value: expr, .. } => expr,
            };
            self.collect_events(expr, state, current_class, side);
        }
    }

    fn emit_send(
        &mut self,
        object: &Expr,
        receiver: &InferredValue,
        selector: &str,
        args: Vec<AnalyzedArgument>,
        dynamic: bool,
        site: SourceRange,
        state: &FlowState,
        current_class: Option<&ClassId>,
        side: Option<DispatchSide>,
    ) {
        if dynamic {
            self.dynamic_send = true;
            return;
        }
        for target in receiver_targets(object, &receiver.shape, current_class, side) {
            let Some(resolved) = (self.resolve_member)(&target, selector) else { continue };
            self.record_call(resolved.callable, site, args.clone(), state);
        }
    }

    fn record_call(&mut self, target: CallableId, site: SourceRange, args: Vec<AnalyzedArgument>, _state: &FlowState) {
        self.events.push(AnalysisEvent::Call(ResolvedCall {
            target: target.clone(),
            site,
            args: args
                .iter()
                .cloned()
                .map(|mut argument| {
                    argument.block_effect = None;
                    argument
                })
                .collect(),
            dynamic: false,
        }));
        let Some(effects) = (self.callable_effects)(&target) else { return };
        self.dynamic_send |= effects.dynamic_send;
        for parameter in effects.invokes_parameters {
            let Some(argument) = args.get(parameter) else { continue };
            if let Some(effect) = &argument.block_effect {
                self.apply_block_effect(effect);
            } else if let Some(binding) = argument.binding
                && let Some(source_parameter) = self.active_parameter_bindings.get(&binding)
            {
                self.invoked_parameters.insert(*source_parameter);
            }
        }
    }

    fn apply_block_effect(&mut self, effect: &BlockEffects) {
        for return_evidence in &effect.nonlocal_returns {
            self.pending_returns.push(return_evidence.clone());
        }
        for (binding, written) in &effect.captured_writes {
            self.pending_writes.insert(*binding, written.clone());
        }
        self.invoked_parameters.extend(effect.invokes_parameters.iter().copied());
        self.dynamic_send |= effect.dynamic_send;
    }

    fn apply_pending_writes(&mut self, state: &mut FlowState) {
        for (binding, value) in std::mem::take(&mut self.pending_writes) {
            state.bindings.insert(binding, value);
        }
    }

    fn parameter_facts_from_events(&mut self) {
        for event in &self.events {
            let AnalysisEvent::Call(call) = event else { continue };
            let receiver = match call.target.side {
                DispatchSide::Instance => DispatchReceiver::Instance(call.target.owner.clone()),
                DispatchSide::Class => DispatchReceiver::ClassObject(call.target.owner.clone()),
            };
            let Some(resolved) = (self.resolve_member)(&receiver, &call.target.selector) else {
                continue;
            };
            let Some(member) = (self.member_surface)(&resolved.callable) else { continue };
            record_parameter_arguments(&mut self.parameter_facts, &member, call);
        }
    }
}

fn record_parameter_arguments(facts: &mut ParameterFacts, member: &MemberSurface, call: &ResolvedCall) {
    let mut positional = 0;
    for argument in &call.args {
        let Some(param) = argument
            .label
            .as_deref()
            .and_then(|label| member.params.iter().find(|param| param.label.as_deref() == Some(label) || param.name == label))
            .or_else(|| {
                let param = member.params.get(positional);
                positional += 1;
                param
            })
        else {
            continue;
        };
        if !matches!(argument.value.shape, ValueShape::Unknown) {
            facts.record(
                member.callable.clone(),
                param.name.clone(),
                InferredValue::interprocedural(argument.value.shape.clone(), call.site),
            );
        }
    }
}

fn positional_block(args: &[PackItem], index: usize) -> Option<&BlockExpr> {
    match args.get(index) {
        Some(PackItem::Positional { expr: Expr::Block(block), .. }) => Some(block.as_ref()),
        _ => None,
    }
}

fn labeled_block<'a>(args: &'a [PackItem], label: &str) -> Option<&'a BlockExpr> {
    args.iter().find_map(|argument| match argument {
        PackItem::Labeled {
            label: PackLabel::Static { text, .. },
            value: Expr::Block(block),
            ..
        } if text == label => Some(block.as_ref()),
        _ => None,
    })
}

fn field_side(class_side: bool) -> DispatchSide {
    if class_side { DispatchSide::Class } else { DispatchSide::Instance }
}

fn narrow_type_test_shape(
    shape: &ValueShape,
    target: &ClassId,
    is_exact: bool,
    truth: bool,
    is_same_or_subclass: &dyn Fn(&ClassId, &ClassId) -> bool,
) -> Option<ValueShape> {
    let matches = |class: &ClassId| {
        if is_exact { class == target } else { is_same_or_subclass(class, target) }
    };
    match shape {
        ValueShape::Unknown if truth => Some(ValueShape::Instance(target.clone())),
        ValueShape::Unknown => Some(ValueShape::Unknown),
        ValueShape::Instance(class) if matches(class) == truth => Some(shape.clone()),
        ValueShape::Instance(_) => None,
        ValueShape::Union(alternatives) => {
            let retained = alternatives
                .iter()
                .filter(|alternative| match alternative {
                    ValueShape::Instance(class) => matches(class) == truth,
                    _ => true,
                })
                .cloned()
                .collect::<Vec<_>>();
            (!retained.is_empty()).then(|| ValueShape::bounded_union(retained))
        }
        _ if truth => None,
        _ => Some(shape.clone()),
    }
}

fn receiver_targets(expr: &Expr, shape: &ValueShape, current_class: Option<&ClassId>, side: Option<DispatchSide>) -> Vec<DispatchReceiver> {
    if matches!(expr, Expr::SuperVar { .. }) {
        return current_class
            .map(|class| {
                vec![DispatchReceiver::Super {
                    lexical_class: class.clone(),
                    side: side.unwrap_or(DispatchSide::Instance),
                }]
            })
            .unwrap_or_default();
    }
    match shape {
        ValueShape::Instance(class) => vec![DispatchReceiver::Instance(class.clone())],
        ValueShape::ClassObject(class) => vec![DispatchReceiver::ClassObject(class.clone())],
        ValueShape::Union(shapes) => shapes.iter().flat_map(|shape| receiver_targets(expr, shape, current_class, side)).collect(),
        _ => Vec::new(),
    }
}

fn join_states(left: &FlowState, right: &FlowState) -> FlowState {
    let mut bindings = BTreeMap::new();
    let keys = left.bindings.keys().chain(right.bindings.keys()).copied().collect::<BTreeSet<_>>();
    for binding in keys {
        let value = match (left.bindings.get(&binding), right.bindings.get(&binding)) {
            (Some(left), Some(right)) => left.join(right),
            (Some(value), None) | (None, Some(value)) => InferredValue::flow(
                ValueShape::Unknown,
                value.provenance.first().map_or(Default::default(), |origin| match origin {
                    super::facts::FactOrigin::Syntax(range)
                    | super::facts::FactOrigin::Binding(range)
                    | super::facts::FactOrigin::CallSite(range)
                    | super::facts::FactOrigin::Constraint(range) => *range,
                    super::facts::FactOrigin::Callable(_) => Default::default(),
                }),
            ),
            (None, None) => InferredValue::flow(ValueShape::Unknown, Default::default()),
        };
        bindings.insert(binding, value);
    }
    FlowState { bindings }
}

const MAX_FLOW_FIXPOINT_ITERS: usize = MAX_SHAPE_UNION + 2;

fn join_states_many(states: impl IntoIterator<Item = FlowState>) -> Option<FlowState> {
    states.into_iter().reduce(|left, right| join_states(&left, &right))
}

fn widen_loop_state(entry: &FlowState, previous: &FlowState, next: &FlowState) -> FlowState {
    let mut widened = join_states(entry, next);
    let keys = previous.bindings.keys().chain(next.bindings.keys()).copied().collect::<BTreeSet<_>>();
    for binding in keys {
        if previous.bindings.get(&binding) != next.bindings.get(&binding) {
            widened.bindings.insert(binding, InferredValue::flow(ValueShape::Unknown, Default::default()));
        }
    }
    widened
}

fn project_outer_state(outer: &FlowState, inner: &FlowState) -> FlowState {
    let bindings = outer
        .bindings
        .keys()
        .map(|binding| {
            (
                *binding,
                inner.bindings.get(binding).cloned().unwrap_or_else(|| outer.bindings[binding].clone()),
            )
        })
        .collect();
    FlowState { bindings }
}

fn has_dynamic_pack(args: &[PackItem]) -> bool {
    args.iter().any(|argument| match argument {
        PackItem::Expand { .. } => true,
        PackItem::Labeled {
            label: PackLabel::Computed { .. },
            ..
        } => true,
        PackItem::Positional { .. }
        | PackItem::Labeled {
            label: PackLabel::Static { .. },
            ..
        } => false,
    })
}

fn static_labels(args: &[PackItem]) -> Vec<Option<String>> {
    args.iter()
        .map(|argument| match argument {
            PackItem::Positional { .. } => None,
            PackItem::Labeled {
                label: PackLabel::Static { text, .. },
                ..
            } => Some(text.clone()),
            PackItem::Expand { .. }
            | PackItem::Labeled {
                label: PackLabel::Computed { .. },
                ..
            } => None,
        })
        .collect()
}
