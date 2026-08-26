//! Message send and callable argument verification (Spec 04.5 / E5).

use super::context::CheckingContext;
use super::expected::{ExpectationOrigin, ExpectedType};
use super::expression::analyze_expression;
use super::inference::{ConstraintOrigin, InferenceRelation, InferenceSession, InferenceSupport, InferenceTerm};
use crate::checker::analysis::AnalysisStatus;
use crate::checker::causal::CausalInvalidity;
use crate::checker::typed_expr::TypedExpression;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::dispatch::{CallableParameter, CallableSemanticKind, CallableSignature};
use crate::identity::{CallableId, ExplanationId};
use crate::types::evidence::DynamicReason;
use crate::types::evidence::{EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::parameter::{GenericConstraint, TypeParameterOwner};
use crate::types::store::TypeData;
use crate::types::substitution::TypeSubstitution;
use phalcom_ast::ast::{Expr, PackItem, PackLabel};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorSlot};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallTargetAuthority {
    ExactDispatch,
    CallableValue(EvidenceStatus),
    StructuralBuiltin,
}

#[derive(Clone, Debug)]
pub(crate) struct CallableApplicationTarget {
    pub signature: CallableSignature,
    pub callable: Option<CallableId>,
    pub authority: CallTargetAuthority,
}

impl CallableApplicationTarget {
    pub(crate) fn exact(callable: CallableId, signature: CallableSignature) -> Self {
        Self {
            signature,
            callable: Some(callable),
            authority: CallTargetAuthority::ExactDispatch,
        }
    }

    pub(crate) fn from_dispatch(resolved: crate::dispatch::ResolvedDispatch) -> Self {
        Self::exact(resolved.callable, resolved.signature)
    }

    pub(crate) fn callable_value(signature: CallableSignature, status: EvidenceStatus) -> Self {
        Self {
            signature,
            callable: None,
            authority: CallTargetAuthority::CallableValue(status),
        }
    }

    pub(crate) fn structural(signature: CallableSignature) -> Self {
        Self {
            signature,
            callable: None,
            authority: CallTargetAuthority::StructuralBuiltin,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CallPremise {
    pub knowledge: TypeKnowledge,
    pub status: AnalysisStatus,
    pub causal_invalidity: CausalInvalidity,
    pub explanation: Option<ExplanationId>,
}

impl CallPremise {
    pub(crate) fn from_typed(ctx: &CheckingContext<'_>, expression: &crate::checker::typed_expr::TypedExpression) -> Self {
        Self {
            knowledge: expression.knowledge.clone(),
            status: expression.status.clone(),
            causal_invalidity: expression.causal_invalidity,
            explanation: expression.expression_id.and_then(|id| ctx.explanation_for_expression(id)),
        }
    }

    pub(crate) fn established(knowledge: TypeKnowledge) -> Self {
        Self {
            knowledge,
            status: AnalysisStatus::Ready,
            causal_invalidity: CausalInvalidity::Clean,
            explanation: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ApplicationArgument<'a> {
    Positional {
        expression: &'a Expr,
        range: SourceRange,
    },
    Labeled {
        label: &'a str,
        expression: &'a Expr,
        range: SourceRange,
    },
    DynamicLabel {
        expression: &'a Expr,
        range: SourceRange,
    },
    Expansion {
        expression: &'a Expr,
        range: SourceRange,
    },
    /// Source operand analyzed before target selection because bilateral
    /// dispatch needs both operand facts. Canonical application records the
    /// existing child product in its call frame instead of traversing it again.
    PreAnalyzed {
        label: Option<&'a str>,
        typed: &'a TypedExpression,
        range: SourceRange,
    },
}

impl<'a> ApplicationArgument<'a> {
    pub(crate) fn expression(self) -> Option<&'a Expr> {
        match self {
            Self::Positional { expression, .. }
            | Self::Labeled { expression, .. }
            | Self::DynamicLabel { expression, .. }
            | Self::Expansion { expression, .. } => Some(expression),
            Self::PreAnalyzed { .. } => None,
        }
    }

    pub(crate) fn range(self) -> SourceRange {
        match self {
            Self::Positional { range, .. }
            | Self::Labeled { range, .. }
            | Self::DynamicLabel { range, .. }
            | Self::Expansion { range, .. }
            | Self::PreAnalyzed { range, .. } => range,
        }
    }
}

fn analyze_application_argument(ctx: &mut CheckingContext<'_>, argument: ApplicationArgument<'_>, expected: &ExpectedType) -> TypedExpression {
    match argument {
        ApplicationArgument::PreAnalyzed { typed, .. } => {
            ctx.record_call_dependency(typed.causal_invalidity, typed.expression_id.and_then(|id| ctx.explanation_for_expression(id)));
            (*typed).clone()
        }
        _ => analyze_expression(ctx, argument.expression().expect("ordinary application argument has an expression"), expected),
    }
}

pub(crate) fn application_arguments(args: &[PackItem]) -> Vec<ApplicationArgument<'_>> {
    args.iter()
        .map(|item| match item {
            PackItem::Positional { expr, range } => ApplicationArgument::Positional {
                expression: expr,
                range: *range,
            },
            PackItem::Labeled {
                label: PackLabel::Static { text, .. },
                value,
                range,
            } => ApplicationArgument::Labeled {
                label: text.as_str(),
                expression: value,
                range: *range,
            },
            PackItem::Labeled { value, range, .. } => ApplicationArgument::DynamicLabel {
                expression: value,
                range: *range,
            },
            PackItem::Expand { expr, range, .. } => ApplicationArgument::Expansion {
                expression: expr,
                range: *range,
            },
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StaticCallShape {
    Exact(Vec<SelectorSlot>),
    Dynamic(DynamicReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArgumentBinding {
    pub argument_index: usize,
    pub parameter_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArgumentBindingPlan {
    pub bindings: Vec<ArgumentBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ArgumentShapeFailure {
    MissingRequiredParameter { parameter_index: usize },
    UnexpectedPositional { argument_index: usize },
    UnknownLabel { argument_index: usize, label: String },
    DuplicateParameterBinding { parameter_index: usize },
    UnsupportedRestShape,
    DynamicShape,
}

pub(crate) fn static_call_shape(arguments: &[ApplicationArgument<'_>]) -> StaticCallShape {
    let mut slots = Vec::with_capacity(arguments.len());
    for argument in arguments {
        match argument {
            ApplicationArgument::Positional { .. } => slots.push(SelectorSlot::Positional),
            ApplicationArgument::Labeled { label, .. } => slots.push(SelectorSlot::Label((*label).to_string())),
            ApplicationArgument::PreAnalyzed { label: Some(label), .. } => slots.push(SelectorSlot::Label((*label).to_string())),
            ApplicationArgument::PreAnalyzed { label: None, .. } => slots.push(SelectorSlot::Positional),
            ApplicationArgument::DynamicLabel { .. } | ApplicationArgument::Expansion { .. } => {
                return StaticCallShape::Dynamic(DynamicReason::DynamicRestPack);
            }
        }
    }
    StaticCallShape::Exact(slots)
}

pub(crate) fn bind_static_arguments(
    arguments: &[ApplicationArgument<'_>],
    parameters: &[CallableParameter],
) -> Result<ArgumentBindingPlan, Vec<ArgumentShapeFailure>> {
    if parameters.iter().any(|parameter| parameter.rest) {
        return Err(vec![ArgumentShapeFailure::UnsupportedRestShape]);
    }
    if arguments
        .iter()
        .any(|argument| matches!(argument, ApplicationArgument::DynamicLabel { .. } | ApplicationArgument::Expansion { .. }))
    {
        return Err(vec![ArgumentShapeFailure::DynamicShape]);
    }

    let mut bindings = Vec::new();
    let mut bound = vec![false; parameters.len()];
    let mut failures = Vec::new();
    let mut positional_cursor = 0;
    for (argument_index, argument) in arguments.iter().enumerate() {
        let parameter_index = match argument {
            ApplicationArgument::Positional { .. } => {
                let mut found = None;
                while positional_cursor < parameters.len() {
                    let index = positional_cursor;
                    positional_cursor += 1;
                    if parameters[index].external_label.is_none() && !bound[index] {
                        found = Some(index);
                        break;
                    }
                }
                found
            }
            ApplicationArgument::PreAnalyzed { label: Some(label), .. } => parameters
                .iter()
                .enumerate()
                .find_map(|(index, parameter)| (parameter.external_label.as_deref() == Some(*label)).then_some(index)),
            ApplicationArgument::PreAnalyzed { label: None, .. } => {
                let mut found = None;
                while positional_cursor < parameters.len() {
                    let index = positional_cursor;
                    positional_cursor += 1;
                    if parameters[index].external_label.is_none() && !bound[index] {
                        found = Some(index);
                        break;
                    }
                }
                found
            }
            ApplicationArgument::Labeled { label, .. } => parameters
                .iter()
                .enumerate()
                .find_map(|(index, parameter)| (parameter.external_label.as_deref() == Some(*label)).then_some(index)),
            ApplicationArgument::DynamicLabel { .. } | ApplicationArgument::Expansion { .. } => None,
        };
        let Some(parameter_index) = parameter_index else {
            failures.push(match argument {
                ApplicationArgument::Positional { .. } | ApplicationArgument::PreAnalyzed { label: None, .. } => {
                    ArgumentShapeFailure::UnexpectedPositional { argument_index }
                }
                ApplicationArgument::PreAnalyzed { label: Some(label), .. } => ArgumentShapeFailure::UnknownLabel {
                    argument_index,
                    label: (*label).to_string(),
                },
                ApplicationArgument::Labeled { label, .. } => ArgumentShapeFailure::UnknownLabel {
                    argument_index,
                    label: (*label).to_string(),
                },
                ApplicationArgument::DynamicLabel { .. } | ApplicationArgument::Expansion { .. } => ArgumentShapeFailure::DynamicShape,
            });
            continue;
        };
        if bound[parameter_index] {
            failures.push(ArgumentShapeFailure::DuplicateParameterBinding { parameter_index });
            continue;
        }
        bound[parameter_index] = true;
        bindings.push(ArgumentBinding {
            argument_index,
            parameter_index,
        });
    }
    for (parameter_index, parameter) in parameters.iter().enumerate() {
        if !parameter.rest && !bound[parameter_index] {
            failures.push(ArgumentShapeFailure::MissingRequiredParameter { parameter_index });
        }
    }
    if failures.is_empty() {
        Ok(ArgumentBindingPlan { bindings })
    } else {
        Err(failures)
    }
}

fn minimum_evidence_status(left: EvidenceStatus, right: EvidenceStatus) -> EvidenceStatus {
    if left == EvidenceStatus::Assumed || right == EvidenceStatus::Assumed {
        EvidenceStatus::Assumed
    } else {
        EvidenceStatus::Established
    }
}

fn target_base_authority(target: &CallableApplicationTarget) -> EvidenceStatus {
    match target.authority {
        CallTargetAuthority::ExactDispatch | CallTargetAuthority::StructuralBuiltin => EvidenceStatus::Established,
        CallTargetAuthority::CallableValue(status) => status,
    }
}

fn target_fixed_return_origin(target: &CallableApplicationTarget) -> EvidenceOrigin {
    match target.authority {
        CallTargetAuthority::ExactDispatch => match target.signature.kind {
            CallableSemanticKind::Ordinary => EvidenceOrigin::CallableSignature,
            CallableSemanticKind::Constructor => EvidenceOrigin::ConstructorSemantics,
            CallableSemanticKind::Native => EvidenceOrigin::NativeSignature,
        },
        CallTargetAuthority::CallableValue(_) => EvidenceOrigin::CallableSignature,
        CallTargetAuthority::StructuralBuiltin => EvidenceOrigin::DeclarationSemantics,
    }
}

fn weaken_known_to_status(knowledge: TypeKnowledge, maximum: EvidenceStatus, origin: EvidenceOrigin, range: SourceRange) -> TypeKnowledge {
    match knowledge {
        TypeKnowledge::Known(evidence) => {
            let status = minimum_evidence_status(evidence.status(), maximum);
            TypeKnowledge::Known(evidence).with_status_and_origin(status, origin, range)
        }
        other => other.with_range(range),
    }
}

fn derive_fixed_return(target: &CallableApplicationTarget, premise: &CallPremise, range: SourceRange) -> TypeKnowledge {
    let Some(premise_status) = premise.knowledge.status() else {
        return premise.knowledge.clone();
    };
    let authority = minimum_evidence_status(target_base_authority(target), premise_status);
    let origin = target_fixed_return_origin(target);
    let return_type = match &target.signature.return_type {
        TypeKnowledge::Known(evidence) => TypeKnowledge::established(evidence.ty(), origin),
        other => other.clone(),
    };
    weaken_known_to_status(return_type, authority, origin, range)
}

fn parameter_for_argument<'a>(plan: &ArgumentBindingPlan, argument_index: usize, parameters: &'a [CallableParameter]) -> Option<&'a CallableParameter> {
    let parameter_index = plan.bindings.iter().find(|binding| binding.argument_index == argument_index)?.parameter_index;
    parameters.get(parameter_index)
}

fn shape_failure_message(failure: &ArgumentShapeFailure) -> Option<String> {
    match failure {
        ArgumentShapeFailure::MissingRequiredParameter { parameter_index } => Some(format!("missing required parameter at position {}", parameter_index + 1)),
        ArgumentShapeFailure::UnexpectedPositional { argument_index } => Some(format!("unexpected positional argument at position {}", argument_index + 1)),
        ArgumentShapeFailure::UnknownLabel { label, .. } => Some(format!("unknown argument label `{label}:`")),
        ArgumentShapeFailure::DuplicateParameterBinding { parameter_index } => {
            Some(format!("parameter at position {} is bound more than once", parameter_index + 1))
        }
        ArgumentShapeFailure::UnsupportedRestShape | ArgumentShapeFailure::DynamicShape => None,
    }
}

fn emit_shape_failures(ctx: &mut CheckingContext<'_>, failures: &[ArgumentShapeFailure], range: SourceRange) {
    for failure in failures {
        let Some(message) = shape_failure_message(failure) else {
            continue;
        };
        if let Some(cause) = ctx.emit_diagnostic(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::CallShapeMismatch,
            message,
            range,
        )) {
            ctx.record_call_status(AnalysisStatus::Invalid(cause));
        }
    }
    if failures.iter().any(|failure| matches!(failure, ArgumentShapeFailure::UnsupportedRestShape)) {
        ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::OpaqueNative(
            "callable rest shape is not modeled".into(),
        )));
    }
}

fn argument_relation_message(argument_index: usize, argument: ApplicationArgument<'_>, parameter: &CallableParameter) -> String {
    match argument {
        ApplicationArgument::Labeled { label, .. } => format!(
            "argument for label `{label}:` does not match expected parameter type `{}`",
            parameter.local_name
        ),
        _ => format!(
            "argument at position {} does not match expected parameter type `{}`",
            argument_index + 1,
            parameter.local_name
        ),
    }
}

fn debug_assert_call_result_coherent(result: &CallCheckResult) {
    if let AnalysisStatus::Invalid(cause) = result.status {
        debug_assert!(result.causal_invalidity.contains(cause));
    }
    if matches!(result.status, AnalysisStatus::Suppressed(_)) {
        debug_assert!(!matches!(result.causal_invalidity, CausalInvalidity::Clean));
    }
}

fn cap_result_to_premise_authority(target: &CallableApplicationTarget, premise: &CallPremise, result: TypeKnowledge, range: SourceRange) -> TypeKnowledge {
    let Some(premise_status) = premise.knowledge.status() else {
        return result;
    };
    let maximum = minimum_evidence_status(target_base_authority(target), premise_status);
    let origin = result.origin().unwrap_or(EvidenceOrigin::GenericInference);
    weaken_known_to_status(result, maximum, origin, range)
}

fn apply_non_generic_callable(
    ctx: &mut CheckingContext<'_>,
    target: &CallableApplicationTarget,
    premise: &CallPremise,
    arguments: &[ApplicationArgument<'_>],
    call_range: SourceRange,
) -> TypeKnowledge {
    let plan = match bind_static_arguments(arguments, &target.signature.parameters) {
        Ok(plan) => Some(plan),
        Err(failures) => {
            emit_shape_failures(ctx, &failures, call_range);
            None
        }
    };

    for (argument_index, argument) in arguments.iter().copied().enumerate() {
        let parameter = plan
            .as_ref()
            .and_then(|plan| parameter_for_argument(plan, argument_index, &target.signature.parameters));
        let expected = parameter
            .and_then(|parameter| parameter.ty.ty())
            .map(|ty| ExpectedType::proper_from(ty, ExpectationOrigin::CallableSignature))
            .unwrap_or_default();
        let typed = analyze_application_argument(ctx, argument, &expected);
        if !typed.status.is_ready() {
            ctx.record_call_status(typed.status.clone());
        }
        if let Some(parameter) = parameter {
            ctx.apply_assignability(
                &typed.knowledge,
                &parameter.ty,
                DiagnosticCode::ArgumentMismatch,
                argument_relation_message(argument_index, argument, parameter),
                argument.range(),
            );
        }
    }

    derive_fixed_return(target, premise, call_range)
}

/// Transitional semantic fallback.
/// Delete when canonical universe dispatch surfaces publish this indexer.
/// This helper may construct a target only; it must never construct the
/// final subscript expression result.
pub(crate) fn structural_list_index_get_target(ctx: &mut CheckingContext<'_>, receiver_ty: TypeId) -> Option<CallableApplicationTarget> {
    let TypeData::Applied { origin, arguments } = ctx.store.get(receiver_ty).clone() else {
        return None;
    };
    let TypeData::Nominal { declaration } = ctx.store.get(origin) else {
        return None;
    };
    if declaration.name.as_ref() != "List" || arguments.len() != 1 {
        return None;
    }
    let int_decl = ctx.resolve_type_name("Int")?;
    let int_ty = ctx.nominal_type_of(&int_decl);
    let selector = Selector::subscript_get(vec![SelectorSlot::Positional]).ok()?;
    let signature = CallableSignature::new(
        selector,
        vec![CallableParameter::new(
            "index",
            TypeKnowledge::established(int_ty, EvidenceOrigin::DeclarationSemantics),
        )],
        TypeKnowledge::established(arguments[0], EvidenceOrigin::DeclarationSemantics),
    );
    Some(CallableApplicationTarget::structural(signature))
}

/// Transitional semantic fallback for the current structural Map indexer.
/// This helper may construct a target only; it must never construct the final
/// subscript expression result.
pub(crate) fn structural_map_index_get_target(ctx: &mut CheckingContext<'_>, receiver_ty: TypeId) -> Option<CallableApplicationTarget> {
    let TypeData::Applied { origin, arguments } = ctx.store.get(receiver_ty).clone() else {
        return None;
    };
    let TypeData::Nominal { declaration } = ctx.store.get(origin) else {
        return None;
    };
    if declaration.name.as_ref() != "Map" || arguments.len() != 2 {
        return None;
    }
    let selector = Selector::subscript_get(vec![SelectorSlot::Positional]).ok()?;
    let signature = CallableSignature::new(
        selector,
        vec![CallableParameter::new(
            "key",
            TypeKnowledge::established(arguments[0], EvidenceOrigin::DeclarationSemantics),
        )],
        TypeKnowledge::established(arguments[1], EvidenceOrigin::DeclarationSemantics),
    );
    Some(CallableApplicationTarget::structural(signature))
}

/// Transitional semantic fallback for List subscript assignment.
/// This helper may construct a target only; it must never construct the final
/// assignment expression result.
pub(crate) fn structural_list_index_set_target(ctx: &mut CheckingContext<'_>, receiver_ty: TypeId) -> Option<CallableApplicationTarget> {
    let TypeData::Applied { origin, arguments } = ctx.store.get(receiver_ty).clone() else {
        return None;
    };
    let TypeData::Nominal { declaration } = ctx.store.get(origin) else {
        return None;
    };
    if declaration.name.as_ref() != "List" || arguments.len() != 1 {
        return None;
    }
    let int_decl = ctx.resolve_type_name("Int")?;
    let int_ty = ctx.nominal_type_of(&int_decl);
    let element_ty = arguments[0];
    let selector = Selector::subscript_set(vec![SelectorSlot::Positional]).ok()?;
    let signature = CallableSignature::new(
        selector,
        vec![
            CallableParameter::new("index", TypeKnowledge::established(int_ty, EvidenceOrigin::DeclarationSemantics)),
            CallableParameter::new("put", TypeKnowledge::established(element_ty, EvidenceOrigin::DeclarationSemantics)),
        ],
        TypeKnowledge::established(element_ty, EvidenceOrigin::DeclarationSemantics),
    );
    Some(CallableApplicationTarget::structural(signature))
}

pub(crate) fn assignment_result_from_call(
    ctx: &mut CheckingContext<'_>,
    operation: CallCheckResult,
    range: SourceRange,
) -> crate::checker::typed_expr::TypedExpression {
    let mut typed = crate::checker::typed_expr::TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Syntax, range);
    typed.status = operation.status;
    typed.causal_invalidity = operation.causal_invalidity;
    typed.callable = operation.callable;
    typed.explanation_parents = operation.explanation_parents;
    typed.debug_assert_coherent();
    typed
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallCheckResult {
    pub knowledge: TypeKnowledge,
    pub status: AnalysisStatus,
    pub causal_invalidity: crate::checker::causal::CausalInvalidity,
    pub explanation_parents: Vec<crate::identity::ExplanationId>,
    pub callable: Option<crate::identity::CallableId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExactReturnOrigin {
    CallableSignature,
    ConstructorSemantics,
    NativeSignature,
    GenericInference,
}

fn exact_return_origin(kind: CallableSemanticKind) -> ExactReturnOrigin {
    match kind {
        CallableSemanticKind::Ordinary => ExactReturnOrigin::CallableSignature,
        CallableSemanticKind::Constructor => ExactReturnOrigin::ConstructorSemantics,
        CallableSemanticKind::Native => ExactReturnOrigin::NativeSignature,
    }
}

/// Promotes a complete callable return contract to call-site knowledge.
/// Unknown and dynamic contracts remain unknown/dynamic; only a concrete
/// exact-dispatch return receives established call-site status.
fn promote_exact_return(return_type: &TypeKnowledge, origin: ExactReturnOrigin, range: SourceRange) -> TypeKnowledge {
    match return_type {
        TypeKnowledge::Known(evidence) => TypeKnowledge::established(
            evidence.ty(),
            match origin {
                ExactReturnOrigin::CallableSignature => EvidenceOrigin::CallableSignature,
                ExactReturnOrigin::ConstructorSemantics => EvidenceOrigin::ConstructorSemantics,
                ExactReturnOrigin::NativeSignature => EvidenceOrigin::NativeSignature,
                ExactReturnOrigin::GenericInference => EvidenceOrigin::GenericInference,
            },
        )
        .with_range(range),
        other => other.clone().with_range(range),
    }
}

fn apply_generic_callable_inner(
    ctx: &mut CheckingContext<'_>,
    signature: &CallableSignature,
    args: &[ApplicationArgument<'_>],
    expected: &ExpectedType,
    call_range: SourceRange,
) -> TypeKnowledge {
    // 1. Generic Callable Resolution via InferenceSession
    if let Some(ref generic_sig) = signature.generics {
        if !generic_sig.parameters.is_empty() {
            let mut session = InferenceSession::new();
            let var_map = session.instantiate_generic_signature(generic_sig, ctx.store);

            let Some(call_id) = ctx.current_expression_id() else {
                return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
            };
            let generic_callable = match &generic_sig.owner {
                TypeParameterOwner::Callable(callable) => callable.clone(),
                TypeParameterOwner::Declaration(declaration) => {
                    crate::identity::CallableId::new(declaration.clone(), signature.selector.clone(), ctx.current_side)
                }
            };
            for (constraint_index, constraint) in generic_sig.constraints.iter().enumerate() {
                let relation = match constraint {
                    GenericConstraint::Subtype { lower, upper } => {
                        let lower = session.type_term_to_inference(lower, &var_map, ctx.store);
                        let upper = session.type_term_to_inference(upper, &var_map, ctx.store);
                        match (lower, upper) {
                            (Ok(lower), Ok(upper)) => InferenceRelation::Subtype(lower, upper),
                            _ => return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
                        }
                    }
                    GenericConstraint::Equivalent { left, right } => {
                        let left = session.type_term_to_inference(left, &var_map, ctx.store);
                        let right = session.type_term_to_inference(right, &var_map, ctx.store);
                        match (left, right) {
                            (Ok(left), Ok(right)) => InferenceRelation::Equivalent(left, right),
                            _ => return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
                        }
                    }
                };
                session.add_constraint(
                    relation,
                    ConstraintOrigin::GenericWhere {
                        callable: generic_callable.clone(),
                        constraint_index: constraint_index as u16,
                    },
                    None,
                );
            }
            let mut positional_idx = 0;

            for arg in args.iter().copied() {
                if matches!(arg, ApplicationArgument::DynamicLabel { .. } | ApplicationArgument::Expansion { .. }) {
                    analyze_application_argument(ctx, arg, &ExpectedType::None);
                    return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
                }

                let parameter_index = match arg {
                    ApplicationArgument::Positional { .. } | ApplicationArgument::PreAnalyzed { label: None, .. } => {
                        let mut found = None;
                        while positional_idx < signature.parameters.len() {
                            let index = positional_idx;
                            positional_idx += 1;
                            if signature.parameters[index].external_label.is_none() {
                                found = Some(index);
                                break;
                            }
                        }
                        found
                    }
                    ApplicationArgument::Labeled { label, .. } | ApplicationArgument::PreAnalyzed { label: Some(label), .. } => signature
                        .parameters
                        .iter()
                        .position(|parameter| parameter.external_label.as_deref() == Some(label)),
                    ApplicationArgument::DynamicLabel { .. } | ApplicationArgument::Expansion { .. } => {
                        unreachable!("dynamic arguments return before generic binding")
                    }
                };

                let Some(parameter_index) = parameter_index else {
                    analyze_application_argument(ctx, arg, &ExpectedType::None);
                    continue;
                };
                let parameter = &signature.parameters[parameter_index];
                let Some(parameter_ty) = parameter.ty.ty() else {
                    return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
                };
                let parameter_term = session.type_id_to_inference(parameter_ty, &var_map, ctx.store);
                let argument_expected = ExpectedType::inference_from(parameter_term.clone(), ExpectationOrigin::GenericArgument);
                let argument_typed = analyze_application_argument(ctx, arg, &argument_expected);
                if let Some(argument_ty) = argument_typed.knowledge.ty()
                    && let Some(support) = inference_support(&argument_typed.knowledge)
                {
                    let explanation = argument_typed.expression_id.and_then(|id| ctx.explanation_for_expression(id));
                    session.add_constraint_with_support(
                        InferenceRelation::Subtype(InferenceTerm::Canonical(argument_ty), parameter_term),
                        ConstraintOrigin::Argument {
                            call: call_id,
                            argument: argument_typed.expression_id.expect("analyzed argument has expression identity"),
                            parameter_index: parameter_index as u16,
                        },
                        explanation,
                        support,
                    );
                }
            }

            let return_term = signature
                .return_type
                .ty()
                .map(|ret_ty| session.type_id_to_inference(ret_ty, &var_map, ctx.store));
            let fixed_return = return_term.as_ref().and_then(|term| {
                if session.term_has_variables(term) {
                    None
                } else {
                    Some(promote_exact_return(&signature.return_type, ExactReturnOrigin::GenericInference, call_range))
                }
            });

            // Solve value-derived constraints before applying contextual result
            // constraints. If the context contradicts a value-supported
            // instantiation, preserve that call fact so the enclosing binding
            // can report the actual-vs-declared mismatch instead of erasing it.
            let argument_outcome = session.solve(ctx.store, &ctx.hierarchy);
            let argument_return = if let crate::checker::inference::InferenceOutcome::Solved(solution) = &argument_outcome {
                if let Some(ret_ty) = signature.return_type.ty() {
                    let mut subst = TypeSubstitution::new();
                    for (&param_id, var_term) in &var_map {
                        if let InferenceTerm::Var(v) = var_term {
                            if let Some(&solved_ty) = solution.substitutions.get(v) {
                                subst.bind(param_id, solved_ty);
                            }
                        }
                    }
                    let specialized_ret = subst.apply(ctx.store, ret_ty);
                    let support = return_term.as_ref().and_then(|term| session.term_support(term));
                    match (return_term.as_ref().is_some_and(|term| session.term_has_variables(term)), support) {
                        (true, Some(InferenceSupport::Established)) => {
                            Some(TypeKnowledge::established(specialized_ret, EvidenceOrigin::GenericInference).with_range(call_range))
                        }
                        (true, Some(InferenceSupport::Assumed)) => {
                            Some(TypeKnowledge::assumed(specialized_ret, EvidenceOrigin::GenericInference).with_range(call_range))
                        }
                        (false, _) => fixed_return.clone(),
                        (true, None) => None,
                    }
                } else {
                    fixed_return.clone()
                }
            } else {
                None
            };

            let outcome = if matches!(
                &argument_outcome,
                crate::checker::inference::InferenceOutcome::Solved(_) | crate::checker::inference::InferenceOutcome::Underconstrained(_)
            ) {
                if let Some(ret_term) = return_term.as_ref() {
                    if let Some(exp_ty) = expected.ty() {
                        session.add_constraint(
                            InferenceRelation::Subtype(ret_term.clone(), InferenceTerm::Canonical(exp_ty)),
                            ConstraintOrigin::ExpectedResult { expression: call_id },
                            None,
                        );
                    } else if let ExpectedType::Inference { term: exp_term, .. } = expected {
                        session.add_constraint(
                            InferenceRelation::Subtype(ret_term.clone(), exp_term.clone()),
                            ConstraintOrigin::ExpectedResult { expression: call_id },
                            None,
                        );
                    }
                }
                if expected.is_none() {
                    argument_outcome
                } else {
                    session.solve(ctx.store, &ctx.hierarchy)
                }
            } else {
                argument_outcome
            };
            match &outcome {
                crate::checker::inference::InferenceOutcome::Blocked(reason) => {
                    ctx.record_call_status(AnalysisStatus::Blocked(reason.clone()));
                }
                crate::checker::inference::InferenceOutcome::Cancelled => ctx.record_call_status(AnalysisStatus::Cancelled),
                crate::checker::inference::InferenceOutcome::BudgetExceeded(report) => {
                    ctx.record_call_status(AnalysisStatus::BudgetExceeded(report.clone()));
                }
                _ => {}
            }
            return match &outcome {
                crate::checker::inference::InferenceOutcome::Solved(solution) => {
                    if let Some(ret_ty) = signature.return_type.ty() {
                        let mut subst = TypeSubstitution::new();
                        for (&param_id, var_term) in &var_map {
                            if let InferenceTerm::Var(v) = var_term {
                                if let Some(&solved_ty) = solution.substitutions.get(v) {
                                    subst.bind(param_id, solved_ty);
                                }
                            }
                        }
                        let specialized_ret = subst.apply(ctx.store, ret_ty);
                        let support = return_term.as_ref().and_then(|term| session.term_support(term));
                        match (session.term_has_variables(return_term.as_ref().expect("generic return term")), support) {
                            (true, Some(InferenceSupport::Established)) => {
                                TypeKnowledge::established(specialized_ret, EvidenceOrigin::GenericInference).with_range(call_range)
                            }
                            (true, Some(InferenceSupport::Assumed)) => {
                                TypeKnowledge::assumed(specialized_ret, EvidenceOrigin::GenericInference).with_range(call_range)
                            }
                            (true, None) => TypeKnowledge::Unknown(UnknownReason::UnderconstrainedTypeVariable),
                            (false, _) => fixed_return.expect("fixed generic return must be available"),
                        }
                    } else {
                        promote_exact_return(&signature.return_type, exact_return_origin(signature.kind), call_range)
                    }
                }
                crate::checker::inference::InferenceOutcome::Underconstrained(_) => terminal_generic_return(&outcome, fixed_return),
                crate::checker::inference::InferenceOutcome::Conflicting(_) => {
                    ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ArgumentMismatch,
                        "generic argument does not satisfy type constraints",
                        call_range,
                    ));
                    terminal_generic_return(&outcome, argument_return.or(fixed_return))
                }
                crate::checker::inference::InferenceOutcome::Blocked(_)
                | crate::checker::inference::InferenceOutcome::Cancelled
                | crate::checker::inference::InferenceOutcome::BudgetExceeded(_) => terminal_generic_return(&outcome, fixed_return),
            };
        }
    }

    // 2. Non-generic Callable Resolution
    let mut positional_idx = 0;
    for arg in args {
        match arg {
            ApplicationArgument::Positional { range, .. } => {
                let mut matched_param = None;
                while positional_idx < signature.parameters.len() {
                    let param = &signature.parameters[positional_idx];
                    positional_idx += 1;
                    if param.external_label.is_none() {
                        matched_param = Some(param);
                        break;
                    }
                }
                let expected_arg = matched_param
                    .and_then(|p| p.ty.ty())
                    .map(|ty| ExpectedType::proper_from(ty, ExpectationOrigin::CallableSignature))
                    .unwrap_or_default();
                let arg_typed = analyze_application_argument(ctx, *arg, &expected_arg);
                if let Some(param) = matched_param {
                    ctx.apply_assignability(
                        &arg_typed.knowledge,
                        &param.ty,
                        DiagnosticCode::ArgumentMismatch,
                        format!("positional argument `{}` does not match expected parameter type", param.local_name),
                        *range,
                    );
                }
            }
            ApplicationArgument::Labeled { label: text, range, .. } => {
                let mut matched_param = None;
                for param in &signature.parameters {
                    if let Some(ref ext_label) = param.external_label {
                        if ext_label == text {
                            matched_param = Some(param);
                            break;
                        }
                    }
                }
                let expected_arg = matched_param
                    .and_then(|p| p.ty.ty())
                    .map(|ty| ExpectedType::proper_from(ty, ExpectationOrigin::CallableSignature))
                    .unwrap_or_default();
                let arg_typed = analyze_application_argument(ctx, *arg, &expected_arg);
                if let Some(param) = matched_param {
                    ctx.apply_assignability(
                        &arg_typed.knowledge,
                        &param.ty,
                        DiagnosticCode::ArgumentMismatch,
                        format!("argument for label `{}:` does not match expected parameter type", text),
                        *range,
                    );
                }
            }
            ApplicationArgument::DynamicLabel { .. } | ApplicationArgument::Expansion { .. } => {
                analyze_application_argument(ctx, *arg, &ExpectedType::None);
                return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
            }
            ApplicationArgument::PreAnalyzed { .. } => {
                let arg_typed = analyze_application_argument(ctx, *arg, &ExpectedType::None);
                if !arg_typed.status.is_ready() {
                    ctx.record_call_status(arg_typed.status);
                }
            }
        }
    }

    // Successful non-generic dispatch establishes that this call reached a
    // concrete callable contract. Keep the contract's type, but upgrade the
    // call-site evidence to `Proven`; the declaration remains `Declared` in
    // the published surface and can still be checked independently against
    // the body.
    promote_exact_return(&signature.return_type, exact_return_origin(signature.kind), call_range)
}

fn apply_generic_callable(
    ctx: &mut CheckingContext<'_>,
    target: &CallableApplicationTarget,
    _premise: &CallPremise,
    arguments: &[ApplicationArgument<'_>],
    expected: &ExpectedType,
    call_range: SourceRange,
) -> TypeKnowledge {
    apply_generic_callable_inner(ctx, &target.signature, arguments, expected, call_range)
}

pub(crate) fn apply_resolved_callable(
    ctx: &mut CheckingContext<'_>,
    target: &CallableApplicationTarget,
    premise: &CallPremise,
    arguments: &[ApplicationArgument<'_>],
    expected: &ExpectedType,
    call_range: SourceRange,
) -> CallCheckResult {
    ctx.begin_call_causal_capture();
    let knowledge = if target.signature.generics.as_ref().is_some_and(|generics| !generics.parameters.is_empty()) {
        let result = apply_generic_callable(ctx, target, premise, arguments, expected, call_range);
        cap_result_to_premise_authority(target, premise, result, call_range)
    } else {
        apply_non_generic_callable(ctx, target, premise, arguments, call_range)
    };
    let (captured_causal_invalidity, mut explanation_parents, captured_status) = ctx.end_call_causal_capture();
    let owning_cause = ctx.owning_cause_for_current_expression();
    let mut causal_invalidity = premise.causal_invalidity.join(captured_causal_invalidity);
    if let AnalysisStatus::Invalid(cause) = &premise.status {
        causal_invalidity = causal_invalidity.join(CausalInvalidity::One(*cause));
    }
    if let Some(cause) = owning_cause {
        causal_invalidity = causal_invalidity.join(CausalInvalidity::One(cause));
    }
    if let Some(explanation) = premise.explanation {
        if !explanation_parents.contains(&explanation) {
            explanation_parents.push(explanation);
        }
    }
    let status = captured_status
        .or_else(|| (!premise.status.is_ready()).then(|| premise.status.clone()))
        .or_else(|| owning_cause.map(AnalysisStatus::Invalid))
        .unwrap_or_else(|| match &knowledge {
            TypeKnowledge::Dynamic(reason) => AnalysisStatus::DynamicBoundary(reason.clone()),
            _ => AnalysisStatus::Ready,
        });
    let result = CallCheckResult {
        knowledge,
        status,
        causal_invalidity,
        explanation_parents,
        callable: target.callable.clone(),
    };
    debug_assert_call_result_coherent(&result);
    result
}

fn analyze_unbound_arguments(
    ctx: &mut CheckingContext<'_>,
    arguments: &[ApplicationArgument<'_>],
) -> (CausalInvalidity, Vec<ExplanationId>, Option<AnalysisStatus>) {
    ctx.begin_call_causal_capture();
    for argument in arguments.iter().copied() {
        let typed = analyze_application_argument(ctx, argument, &ExpectedType::None);
        if !typed.status.is_ready() {
            ctx.record_call_status(typed.status);
        }
    }
    ctx.end_call_causal_capture()
}

#[derive(Clone, Debug)]
pub(crate) enum UnresolvedApplicationReason {
    PremiseUnknown,
    PremiseInvalidUnavailable,
    PremiseDynamic(DynamicReason),
    DispatchMissing,
    DispatchAmbiguous,
    DynamicShape(DynamicReason),
    IterationArgumentUnavailable,
}

pub(crate) fn analyze_unresolved_application(
    ctx: &mut CheckingContext<'_>,
    premise: &CallPremise,
    arguments: &[ApplicationArgument<'_>],
    reason: UnresolvedApplicationReason,
) -> CallCheckResult {
    let (argument_invalidity, mut explanation_parents, argument_status) = analyze_unbound_arguments(ctx, arguments);
    if let Some(explanation) = premise.explanation {
        if !explanation_parents.contains(&explanation) {
            explanation_parents.push(explanation);
        }
    }
    let causal_invalidity = premise.causal_invalidity.join(argument_invalidity);
    let knowledge = match &reason {
        UnresolvedApplicationReason::PremiseUnknown => premise.knowledge.clone(),
        UnresolvedApplicationReason::PremiseInvalidUnavailable => TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause),
        UnresolvedApplicationReason::PremiseDynamic(reason) | UnresolvedApplicationReason::DynamicShape(reason) => TypeKnowledge::Dynamic(reason.clone()),
        UnresolvedApplicationReason::DispatchMissing | UnresolvedApplicationReason::DispatchAmbiguous => {
            TypeKnowledge::Unknown(UnknownReason::DynamicMessageSend)
        }
        UnresolvedApplicationReason::IterationArgumentUnavailable => TypeKnowledge::Unknown(UnknownReason::UncheckedExpression),
    };
    let status = argument_status.unwrap_or_else(|| match &reason {
        UnresolvedApplicationReason::PremiseInvalidUnavailable => premise
            .causal_invalidity
            .suppression_cause()
            .map(AnalysisStatus::Suppressed)
            .unwrap_or(AnalysisStatus::Ready),
        UnresolvedApplicationReason::PremiseDynamic(reason) | UnresolvedApplicationReason::DynamicShape(reason) => {
            AnalysisStatus::DynamicBoundary(reason.clone())
        }
        UnresolvedApplicationReason::IterationArgumentUnavailable => AnalysisStatus::Ready,
        _ => AnalysisStatus::Ready,
    });
    let result = CallCheckResult {
        knowledge,
        status,
        causal_invalidity,
        explanation_parents,
        callable: None,
    };
    debug_assert_call_result_coherent(&result);
    result
}

pub(crate) fn analyze_non_callable_invocation(
    ctx: &mut CheckingContext<'_>,
    premise: &CallPremise,
    args: &[PackItem],
    call_range: SourceRange,
) -> CallCheckResult {
    let mut result = analyze_unresolved_application(ctx, premise, &application_arguments(args), UnresolvedApplicationReason::DispatchMissing);
    let cause = ctx.emit_diagnostic(SemanticDiagnostic::error_in(
        ctx.current_module.clone(),
        DiagnosticCode::NotCallable,
        "value is not callable",
        call_range,
    ));
    if let Some(cause) = cause {
        result.status = AnalysisStatus::Invalid(cause);
        result.causal_invalidity = result.causal_invalidity.join(CausalInvalidity::One(cause));
    }
    debug_assert_call_result_coherent(&result);
    result
}

fn terminal_generic_return(outcome: &crate::checker::inference::InferenceOutcome, fixed_return: Option<TypeKnowledge>) -> TypeKnowledge {
    if let Some(fixed_return) = fixed_return {
        return fixed_return;
    }
    match outcome {
        crate::checker::inference::InferenceOutcome::Underconstrained(_) => TypeKnowledge::Unknown(UnknownReason::UnderconstrainedTypeVariable),
        crate::checker::inference::InferenceOutcome::Conflicting(_) => TypeKnowledge::Unknown(UnknownReason::InferenceConflict),
        crate::checker::inference::InferenceOutcome::Blocked(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
        crate::checker::inference::InferenceOutcome::Cancelled => TypeKnowledge::Unknown(UnknownReason::InferenceCancelled),
        crate::checker::inference::InferenceOutcome::BudgetExceeded(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBudgetExceeded),
        crate::checker::inference::InferenceOutcome::Solved(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
    }
}

fn inference_support(knowledge: &TypeKnowledge) -> Option<InferenceSupport> {
    match knowledge.status() {
        Some(EvidenceStatus::Established) => Some(InferenceSupport::Established),
        Some(EvidenceStatus::Assumed) => Some(InferenceSupport::Assumed),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ApplicationArgument, ArgumentShapeFailure, StaticCallShape, application_arguments, bind_static_arguments, static_call_shape, terminal_generic_return,
    };
    use crate::checker::inference::{InferenceConflict, InferenceFailureReason, InferenceOutcome, InferenceTerm, UnderconstrainedInference};
    use crate::dispatch::CallableParameter;
    use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
    use crate::types::id::TypeId;
    use crate::types::outcome::{BlockReason, BudgetKind, BudgetReport};
    use phalcom_ast::ast::{Expr, Statement};
    use phalcom_ast::parse_source;
    use phalcom_common::range::SourceRange;

    fn terminal_outcomes() -> [(InferenceOutcome, UnknownReason); 5] {
        [
            (
                InferenceOutcome::Underconstrained(UnderconstrainedInference { unsolved_vars: Vec::new() }),
                UnknownReason::UnderconstrainedTypeVariable,
            ),
            (
                InferenceOutcome::Conflicting(InferenceConflict {
                    constraint_index: Some(2),
                    origin: None,
                    failure: InferenceFailureReason::StructuralMismatch {
                        left: Box::new(InferenceTerm::Canonical(TypeId(1))),
                        right: Box::new(InferenceTerm::Canonical(TypeId(2))),
                    },
                }),
                UnknownReason::InferenceConflict,
            ),
            (InferenceOutcome::Blocked(BlockReason::RecursiveFixpoint), UnknownReason::InferenceBlocked),
            (InferenceOutcome::Cancelled, UnknownReason::InferenceCancelled),
            (
                InferenceOutcome::BudgetExceeded(BudgetReport::new(BudgetKind::Steps, 0, 1)),
                UnknownReason::InferenceBudgetExceeded,
            ),
        ]
    }

    #[test]
    fn every_generic_terminal_outcome_keeps_its_reason_without_fixed_return() {
        for (outcome, reason) in terminal_outcomes() {
            assert_eq!(terminal_generic_return(&outcome, None), TypeKnowledge::Unknown(reason));
        }
    }

    #[test]
    fn every_generic_terminal_outcome_preserves_only_independent_fixed_return() {
        let fixed = TypeKnowledge::established(TypeId(99), EvidenceOrigin::CallableSignature);
        for (outcome, _) in terminal_outcomes() {
            assert_eq!(terminal_generic_return(&outcome, Some(fixed.clone())), fixed);
        }
    }

    #[test]
    fn static_shape_preserves_positional_and_labeled_slots() {
        let parsed = parse_source("call(1, named: 2)", 0).expect("valid call");
        let Expr::UnqualifiedCall(call) = (match &parsed.statements[0] {
            Statement::Expr { expr, .. } => expr,
            _ => panic!("expected expression"),
        }) else {
            panic!("expected unqualified call");
        };
        let arguments = application_arguments(&call.args);
        assert_eq!(
            static_call_shape(&arguments),
            StaticCallShape::Exact(vec![
                phalcom_common::selector::SelectorSlot::Positional,
                phalcom_common::selector::SelectorSlot::Label("named".into()),
            ])
        );
    }

    #[test]
    fn static_shape_rejects_expansion_and_dynamic_label() {
        let expr = Expr::Ellipsis { range: SourceRange::default() };
        let expansion = [ApplicationArgument::Expansion {
            expression: &expr,
            range: SourceRange::default(),
        }];
        assert_eq!(
            static_call_shape(&expansion),
            StaticCallShape::Dynamic(crate::types::evidence::DynamicReason::DynamicRestPack)
        );

        let dynamic_label = [ApplicationArgument::DynamicLabel {
            expression: &expr,
            range: SourceRange::default(),
        }];
        assert_eq!(
            static_call_shape(&dynamic_label),
            StaticCallShape::Dynamic(crate::types::evidence::DynamicReason::DynamicRestPack)
        );
    }

    #[test]
    fn static_binding_reports_shape_failures_and_exact_bindings() {
        let expr = Expr::Ellipsis { range: SourceRange::default() };
        let range = SourceRange::default();
        let arguments = vec![
            ApplicationArgument::Positional { expression: &expr, range },
            ApplicationArgument::Labeled {
                label: "named",
                expression: &expr,
                range,
            },
        ];
        let parameters = vec![
            CallableParameter::new("first", TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)),
            CallableParameter::new("second", TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)).with_label("named"),
        ];
        let plan = bind_static_arguments(&arguments, &parameters).expect("arguments bind");
        assert_eq!(
            plan.bindings,
            vec![
                super::ArgumentBinding {
                    argument_index: 0,
                    parameter_index: 0,
                },
                super::ArgumentBinding {
                    argument_index: 1,
                    parameter_index: 1,
                },
            ]
        );

        let missing = bind_static_arguments(&arguments[..1], &parameters).expect_err("missing parameter");
        assert!(missing.contains(&ArgumentShapeFailure::MissingRequiredParameter { parameter_index: 1 }));

        let extra_args = vec![arguments[0], arguments[0], arguments[1]];
        let extra = bind_static_arguments(&extra_args, &parameters).expect_err("duplicate/extra argument");
        assert!(extra.iter().any(|failure| matches!(failure, ArgumentShapeFailure::UnexpectedPositional { .. })));

        let unknown_label = [ApplicationArgument::Labeled {
            label: "other",
            expression: &expr,
            range,
        }];
        let unknown = bind_static_arguments(&unknown_label, &parameters).expect_err("unknown label");
        assert!(unknown.contains(&ArgumentShapeFailure::UnknownLabel {
            argument_index: 0,
            label: "other".into(),
        }));

        let duplicate = [arguments[1], arguments[1]];
        let duplicate = bind_static_arguments(&duplicate, &parameters).expect_err("duplicate label");
        assert!(duplicate.contains(&ArgumentShapeFailure::DuplicateParameterBinding { parameter_index: 1 }));

        let rest = [CallableParameter::new("rest", TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)).with_rest(true)];
        assert_eq!(bind_static_arguments(&[], &rest), Err(vec![ArgumentShapeFailure::UnsupportedRestShape]));
    }
}
