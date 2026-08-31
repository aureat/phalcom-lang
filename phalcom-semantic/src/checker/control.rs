//! Control-flow outcomes and executable region analysis.

use crate::checker::causal::CausalInvalidity;
use crate::checker::context::CheckingContext;
use crate::checker::expected::ExpectedType;
use crate::checker::expression::analyze_expression;
use crate::checker::flow::FlowState;
use crate::checker::statement::check_statement;
use crate::checker::typed_expr::TypedExpression;
use crate::types::evidence::{EvidenceOrigin, EvidenceStatus, TypeKnowledge, join_type_knowledge};
use phalcom_ast::ast::{Expr, Statement};
use phalcom_common::range::SourceRange;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatementControl {
    FallsThrough,
    Return,
    Throw,
    Break,
    Continue,
}

impl StatementControl {
    pub const fn is_abrupt(self) -> bool {
        !matches!(self, Self::FallsThrough)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ExecutableRegionResult {
    /// Present only when the region has a reachable normal completion.
    pub value: Option<TypedExpression>,
    /// Current region flow after execution. Unreachable for abrupt-only completion.
    pub flow: FlowState,
    pub causal_invalidity: CausalInvalidity,
}

impl ExecutableRegionResult {
    pub fn completes_normally(&self) -> bool {
        self.value.is_some() && self.flow.is_reachable()
    }
}

pub(crate) fn analyze_executable_region(
    ctx: &mut CheckingContext<'_>,
    statements: &[Statement],
    range: SourceRange,
    expected: &ExpectedType,
) -> ExecutableRegionResult {
    analyze_executable_region_with_prelude(ctx, statements, range, expected, |_| {})
}

pub(crate) fn analyze_executable_region_with_prelude(
    ctx: &mut CheckingContext<'_>,
    statements: &[Statement],
    range: SourceRange,
    expected: &ExpectedType,
    prelude: impl FnOnce(&mut CheckingContext<'_>),
) -> ExecutableRegionResult {
    ctx.push_scope();
    prelude(ctx);

    let mut last = None;
    let mut causal = CausalInvalidity::Clean;

    for (index, statement) in statements.iter().enumerate() {
        if !ctx.flow.is_reachable() {
            break;
        }

        let is_tail = index + 1 == statements.len();
        match statement {
            Statement::Expr { expr, .. } if is_tail => {
                let typed = analyze_expression(ctx, expr, expected);
                causal = causal.join(typed.causal_invalidity);
                if ctx.flow.is_reachable() && typed.knowledge.ty() != Some(ctx.store.never()) {
                    last = Some(typed);
                }
            }
            _ => {
                let control = check_statement(ctx, statement);
                if control.is_abrupt() || !ctx.flow.is_reachable() {
                    last = None;
                    break;
                }
                if is_tail {
                    last = Some(TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Flow, range));
                }
            }
        }
    }

    if statements.is_empty() && ctx.flow.is_reachable() {
        last = Some(TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Flow, range));
    }

    let flow = ctx.flow.clone();
    ctx.pop_scope();
    ExecutableRegionResult {
        value: last.filter(|_| flow.is_reachable()),
        flow,
        causal_invalidity: causal,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConditionTruth {
    AlwaysTrue,
    AlwaysFalse,
    Unknown,
}

pub(crate) fn condition_truth(expr: &Expr) -> ConditionTruth {
    match expr {
        Expr::Boolean { value: true, .. } => ConditionTruth::AlwaysTrue,
        Expr::Boolean { value: false, .. } => ConditionTruth::AlwaysFalse,
        Expr::Unary(unary) if matches!(unary.op, phalcom_ast::ast::UnaryOp::Not) => match condition_truth(&unary.expr) {
            ConditionTruth::AlwaysTrue => ConditionTruth::AlwaysFalse,
            ConditionTruth::AlwaysFalse => ConditionTruth::AlwaysTrue,
            ConditionTruth::Unknown => ConditionTruth::Unknown,
        },
        _ => ConditionTruth::Unknown,
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct ConditionFlowSplit {
    pub condition: TypedExpression,
    pub when_true: FlowState,
    pub when_false: FlowState,
    pub causal_invalidity: CausalInvalidity,
}

#[allow(dead_code)]
pub(crate) fn analyze_condition_split(ctx: &mut CheckingContext<'_>, condition: &Expr) -> ConditionFlowSplit {
    let expected_bool = ctx
        .core_type(&ctx.core_ids.bool_.clone())
        .map(|ty| ExpectedType::proper_from(ty, crate::checker::expected::ExpectationOrigin::ExplicitCheck))
        .unwrap_or_default();

    let condition_typed = analyze_expression(ctx, condition, &expected_bool);
    analyze_condition_split_with_typed(ctx, condition, condition_typed)
}

pub(crate) fn analyze_condition_split_with_typed(ctx: &mut CheckingContext<'_>, condition: &Expr, condition_typed: TypedExpression) -> ConditionFlowSplit {
    let truth = condition_truth(condition);
    let before = ctx.flow.clone();

    let when_true = match truth {
        ConditionTruth::AlwaysFalse => FlowState::unreachable(),
        _ => {
            ctx.flow = before.clone();
            if let Some(predicate) = crate::checker::flow::extract_trusted_predicate(ctx, condition, &condition_typed, true) {
                ctx.apply_flow_predicate(&predicate);
            }
            ctx.flow.clone()
        }
    };

    let when_false = match truth {
        ConditionTruth::AlwaysTrue => FlowState::unreachable(),
        _ => {
            ctx.flow = before.clone();
            if let Some(predicate) = crate::checker::flow::extract_trusted_predicate(ctx, condition, &condition_typed, false) {
                ctx.apply_flow_predicate(&predicate);
            }
            ctx.flow.clone()
        }
    };

    ctx.flow = before;

    ConditionFlowSplit {
        causal_invalidity: condition_typed.causal_invalidity,
        condition: condition_typed,
        when_true,
        when_false,
    }
}

pub(crate) struct BranchPairResult {
    pub typed: TypedExpression,
    #[allow(dead_code)]
    pub then_flow: FlowState,
    #[allow(dead_code)]
    pub else_flow: FlowState,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn analyze_branch_pair(
    ctx: &mut CheckingContext<'_>,
    condition: &Expr,
    condition_typed: &TypedExpression,
    then_body: &[Statement],
    then_range: SourceRange,
    else_body: Option<(&[Statement], SourceRange)>,
    whole_range: SourceRange,
    expected: &ExpectedType,
) -> BranchPairResult {
    let split = analyze_condition_split_with_typed(ctx, condition, condition_typed.clone());

    // 1. Then branch
    let then_result = if split.when_true.is_reachable() {
        ctx.flow = split.when_true.clone();
        analyze_executable_region(ctx, then_body, then_range, expected)
    } else {
        ExecutableRegionResult {
            value: None,
            flow: FlowState::unreachable(),
            causal_invalidity: CausalInvalidity::Clean,
        }
    };
    let then_flow = ctx.flow.clone();

    // 2. Else branch
    let else_result = if split.when_false.is_reachable() {
        ctx.flow = split.when_false.clone();
        if let Some((else_stmts, else_range)) = else_body {
            analyze_executable_region(ctx, else_stmts, else_range, expected)
        } else {
            ExecutableRegionResult {
                value: Some(TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Flow, whole_range)),
                flow: ctx.flow.clone(),
                causal_invalidity: CausalInvalidity::Clean,
            }
        }
    } else {
        ExecutableRegionResult {
            value: None,
            flow: FlowState::unreachable(),
            causal_invalidity: CausalInvalidity::Clean,
        }
    };
    let else_flow = ctx.flow.clone();

    let typed = join_branch_results(ctx, condition_typed.causal_invalidity, &then_result, &else_result, whole_range);

    BranchPairResult { typed, then_flow, else_flow }
}

pub(crate) fn join_branch_results(
    ctx: &mut CheckingContext<'_>,
    premise_causal: CausalInvalidity,
    then_result: &ExecutableRegionResult,
    else_result: &ExecutableRegionResult,
    whole_range: SourceRange,
) -> TypedExpression {
    let then_normal = then_result.completes_normally();
    let else_normal = else_result.completes_normally();

    let mut normal_flows = Vec::new();
    if then_normal {
        normal_flows.push(then_result.flow.clone());
    }
    if else_normal {
        normal_flows.push(else_result.flow.clone());
    }

    let join_status = if normal_flows.is_empty() {
        ctx.flow = FlowState::unreachable();
        None
    } else if normal_flows.len() == 1 {
        ctx.flow = normal_flows.remove(0);
        None
    } else {
        match ctx.join_flow_states(&normal_flows) {
            Ok(f) => {
                ctx.flow = f;
                None
            }
            Err(failure) => Some(ctx.publish_flow_join_failure(failure, whole_range)),
        }
    };

    let mut normal_values = Vec::new();
    if let Some(val) = &then_result.value {
        if then_normal {
            normal_values.push(val.knowledge.clone());
        }
    }
    if let Some(val) = &else_result.value {
        if else_normal {
            normal_values.push(val.knowledge.clone());
        }
    }

    let knowledge = if normal_values.is_empty() {
        TypeKnowledge::established(ctx.store.never(), EvidenceOrigin::Flow)
    } else {
        join_type_knowledge(ctx.store, normal_values)
    };

    let mut typed = TypedExpression::new(knowledge.clone());
    if let Some(status) = join_status {
        typed.status = status;
    }
    typed.causal_invalidity = premise_causal.join(then_result.causal_invalidity).join(else_result.causal_invalidity);

    let mut explanation_parents = Vec::new();
    if let Some(val) = &then_result.value {
        explanation_parents.extend(val.explanation_parents.iter().copied());
    }
    if let Some(val) = &else_result.value {
        explanation_parents.extend(val.explanation_parents.iter().copied());
    }
    typed.explanation_parents = explanation_parents.clone();

    let branch_values = [
        then_result
            .value
            .as_ref()
            .map(|v| v.knowledge.clone())
            .unwrap_or_else(|| TypeKnowledge::established(ctx.store.never(), EvidenceOrigin::Flow)),
        else_result
            .value
            .as_ref()
            .map(|v| v.knowledge.clone())
            .unwrap_or_else(|| TypeKnowledge::established(ctx.store.never(), EvidenceOrigin::Flow)),
    ];

    let join_derivation = ctx.record_derivation(
        crate::explain::ExplanationStep::BranchJoin {
            binding: None,
            branches: branch_values.into(),
            reachable: Box::new([then_normal, else_normal]),
            joined: knowledge.clone(),
        },
        crate::explain::DerivationRule::BranchJoin { branch_count: 2 },
        knowledge.status().unwrap_or(EvidenceStatus::Assumed),
        EvidenceOrigin::Flow,
        Vec::new(),
        explanation_parents,
    );
    typed.explanation_parents.push(join_derivation);

    if let (Some(d1), Some(d2)) = (
        then_result.value.as_ref().and_then(|v| v.denotation.as_ref()),
        else_result.value.as_ref().and_then(|v| v.denotation.as_ref()),
    ) {
        if d1 == d2 {
            typed.denotation = Some(d1.clone());
        }
    }

    typed
}
