//! Contextual realization of canonical basis values and native imaginary
//! literals through ordinary source-owned class-side trait requirements.

use crate::checker::analysis::{
    AlgebraicBasisAxis, ContextualRealization, ContextualRealizationOperation, RealizationCapabilitySelection, RealizationCoefficientSource,
    RealizationEvaluationPlan,
};
use crate::checker::context::CheckingContext;
use crate::checker::expected::ExpectedType;
use crate::checker::typed_expr::TypedExpression;
use crate::generic_evidence::{GenericConformanceEvidence, GenericConformanceProof};
use crate::identity::{DeclarationId, DispatchSide, ExpressionId};
use crate::trait_dispatch::{AssumedTraitDispatchSelection, TraitDispatchSelection};
use crate::traits::{TraitRef, TraitSurfaceMember};
use crate::types::environment::TypeEnvironment;
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge};
use crate::types::parameter::TypeTerm;
use phalcom_ast::ast::{BinaryExpr, BinaryOp, Expr, ImaginaryAxis, ImaginaryCoefficient};
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath};
use std::sync::Arc;

#[derive(Clone)]
enum RealizationSource {
    Basis {
        ty: crate::identity::TypeId,
        axis: AlgebraicBasisAxis,
    },
    Literal {
        scalar: crate::identity::TypeId,
        axis: AlgebraicBasisAxis,
        coefficient: RealizationCoefficientSource,
    },
}

#[derive(Default)]
struct ArithmeticRegionFacts {
    axes: u8,
    has_basis: bool,
    scalar_types: Vec<crate::identity::TypeId>,
    has_unresolved_coefficient_type: bool,
    algebra_types: Vec<crate::identity::TypeId>,
}

#[derive(Default)]
struct ArithmeticRegionSummary {
    facts: ArithmeticRegionFacts,
    known_value_type: Option<crate::identity::TypeId>,
    basis_axis: Option<AlgebraicBasisAxis>,
}

#[derive(Default)]
struct RegionNodeSummary {
    axes: u8,
    has_basis: bool,
    known_value_type: Option<crate::identity::TypeId>,
    basis_axis: Option<AlgebraicBasisAxis>,
}

enum ScanFrame<'a> {
    Enter(&'a Expr),
    ExitUnary,
    ExitBinary(&'a BinaryExpr),
}

/// Selects the bounded canonical target for an arithmetic region. This is a
/// source-semantic rule over canonical basis types and known scalar types; it
/// never searches imported declarations for a convenient algebra.
pub(crate) fn arithmetic_region_expected(ctx: &mut CheckingContext<'_>, expression: &Expr, expected: &ExpectedType) -> Option<ExpectedType> {
    let is_region = match expression {
        Expr::Imaginary { .. } => true,
        Expr::Unary(unary) => matches!(unary.op, phalcom_ast::ast::UnaryOp::Plus | phalcom_ast::ast::UnaryOp::Minus),
        Expr::Binary(binary) => is_algebraic_region_operator(&binary.op),
        _ => false,
    };
    if !is_region {
        return None;
    }

    let summary = summarize_arithmetic_region(ctx, expression);
    let facts = &summary.facts;
    if !facts.has_basis {
        return None;
    }

    if let Some(target) = expected.ty()
        && (expected_is_concrete(expected) || is_canonical_algebra_type(ctx, target))
    {
        return Some(expected.clone());
    }

    let target = if let Some(first) = facts.algebra_types.first().copied() {
        if facts.algebra_types.iter().any(|candidate| *candidate != first) || !is_canonical_algebra_type(ctx, first) {
            return None;
        }
        first
    } else {
        if facts.has_unresolved_coefficient_type {
            return None;
        }
        let scalar_type = match facts.scalar_types.first().copied() {
            Some(first) if facts.scalar_types.iter().all(|candidate| *candidate == first) => first,
            Some(_) => return None,
            None => {
                let int_decl = ctx.core_ids.int.clone();
                ctx.core_type(&int_decl)?
            }
        };
        let axis = if facts.axes & ((1 << axis_index(AlgebraicBasisAxis::J)) | (1 << axis_index(AlgebraicBasisAxis::K))) != 0 {
            AlgebraicBasisAxis::J
        } else {
            AlgebraicBasisAxis::I
        };
        canonical_algebra_type(ctx, scalar_type, axis)?
    };

    Some(ExpectedType::proper_from(
        target,
        crate::checker::expected::ExpectationOrigin::AlgebraicArithmetic,
    ))
}

/// Returns whether this operand contains a canonical basis source inside a
/// supported arithmetic region. Used only to decide where an already selected
/// target expectation should flow.
pub(crate) fn contains_arithmetic_basis(ctx: &mut CheckingContext<'_>, expression: &Expr) -> bool {
    summarize_arithmetic_region(ctx, expression).facts.has_basis
}

/// Detects a source basis binding (as opposed to a completed algebraic
/// expression) for scalar-by-basis realization.
pub(crate) fn is_basis_source(ctx: &mut CheckingContext<'_>, expression: &Expr) -> bool {
    basis_axis_for_expression(ctx, expression).is_some()
}

fn is_algebraic_region_operator(operator: &BinaryOp) -> bool {
    matches!(operator, BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply)
}

fn summarize_arithmetic_region(ctx: &mut CheckingContext<'_>, expression: &Expr) -> ArithmeticRegionSummary {
    let mut facts = ArithmeticRegionFacts::default();
    let mut frames = vec![ScanFrame::Enter(expression)];
    let mut nodes = Vec::<RegionNodeSummary>::new();

    while let Some(frame) = frames.pop() {
        match frame {
            ScanFrame::Enter(expression) => match expression {
                Expr::Binary(binary) if is_algebraic_region_operator(&binary.op) => {
                    frames.push(ScanFrame::ExitBinary(binary));
                    frames.push(ScanFrame::Enter(&binary.right));
                    frames.push(ScanFrame::Enter(&binary.left));
                }
                Expr::Unary(unary) if matches!(unary.op, phalcom_ast::ast::UnaryOp::Plus | phalcom_ast::ast::UnaryOp::Minus) => {
                    frames.push(ScanFrame::ExitUnary);
                    frames.push(ScanFrame::Enter(&unary.expr));
                }
                _ => nodes.push(summarize_leaf(ctx, expression, &mut facts)),
            },
            ScanFrame::ExitUnary => {
                let child = nodes.pop().expect("post-order unary child summary");
                let basis_axis = child.known_value_type.and_then(|ty| canonical_axis_for_type(ctx, ty));
                nodes.push(RegionNodeSummary {
                    axes: child.axes,
                    has_basis: child.has_basis,
                    known_value_type: child.known_value_type,
                    basis_axis,
                });
            }
            ScanFrame::ExitBinary(binary) => {
                let right = nodes.pop().expect("post-order right operand summary");
                let left = nodes.pop().expect("post-order left operand summary");
                match binary.op {
                    BinaryOp::Multiply => {
                        if left.basis_axis.is_some() ^ right.basis_axis.is_some() {
                            let coefficient = if left.basis_axis.is_some() {
                                right.known_value_type
                            } else {
                                left.known_value_type
                            };
                            if let Some(scalar) = coefficient {
                                facts.scalar_types.push(scalar);
                            } else {
                                facts.has_unresolved_coefficient_type = true;
                            }
                        }
                    }
                    BinaryOp::Add | BinaryOp::Subtract => {
                        for operand in [&left, &right] {
                            if let Some(ty) = operand.known_value_type {
                                if canonical_axis_for_type(ctx, ty).is_some() {
                                    continue;
                                }
                                if is_canonical_algebra_type(ctx, ty) {
                                    facts.algebra_types.push(ty);
                                } else {
                                    facts.scalar_types.push(ty);
                                }
                            } else if !operand.has_basis {
                                facts.has_unresolved_coefficient_type = true;
                            }
                        }
                    }
                    _ => unreachable!("only supported arithmetic operators enter the region summarizer"),
                }

                let known_value_type = combine_known_numeric_types(ctx, left.known_value_type, right.known_value_type);
                let basis_axis = known_value_type.and_then(|ty| canonical_axis_for_type(ctx, ty));
                nodes.push(RegionNodeSummary {
                    axes: left.axes | right.axes,
                    has_basis: left.has_basis || right.has_basis,
                    known_value_type,
                    basis_axis,
                });
            }
        }
    }

    let root = nodes.pop().unwrap_or_default();
    debug_assert!(nodes.is_empty(), "every post-order frame must produce exactly one summary");
    ArithmeticRegionSummary {
        facts,
        known_value_type: root.known_value_type,
        basis_axis: root.basis_axis,
    }
}

fn summarize_leaf(ctx: &mut CheckingContext<'_>, expression: &Expr, facts: &mut ArithmeticRegionFacts) -> RegionNodeSummary {
    let known_value_type = known_leaf_value_type(ctx, expression);
    let mut summary = RegionNodeSummary {
        known_value_type,
        ..RegionNodeSummary::default()
    };

    match expression {
        Expr::Imaginary { axis, .. } => {
            let axis = imaginary_axis(*axis);
            summary.axes |= 1 << axis_index(axis);
            summary.has_basis = true;
            facts.axes |= 1 << axis_index(axis);
            facts.has_basis = true;
            summary.basis_axis = Some(axis);
            if let Some(scalar) = known_value_type {
                facts.scalar_types.push(scalar);
            }
        }
        Expr::Var { .. } | Expr::GetProperty(_) => {
            if let Some(ty) = known_value_type {
                if let Some(axis) = canonical_axis_for_type(ctx, ty) {
                    summary.axes |= 1 << axis_index(axis);
                    summary.has_basis = true;
                    facts.axes |= 1 << axis_index(axis);
                    facts.has_basis = true;
                    summary.basis_axis = Some(axis);
                } else if is_canonical_algebra_type(ctx, ty) {
                    facts.algebra_types.push(ty);
                }
            }
        }
        _ => {}
    }
    summary
}

fn basis_axis_for_expression(ctx: &mut CheckingContext<'_>, expression: &Expr) -> Option<AlgebraicBasisAxis> {
    if let Expr::Imaginary { axis, .. } = expression {
        return Some(match axis {
            ImaginaryAxis::I => AlgebraicBasisAxis::I,
            ImaginaryAxis::J => AlgebraicBasisAxis::J,
            ImaginaryAxis::K => AlgebraicBasisAxis::K,
        });
    }
    summarize_arithmetic_region(ctx, expression).basis_axis
}

fn known_value_type(ctx: &mut CheckingContext<'_>, expression: &Expr) -> Option<crate::identity::TypeId> {
    summarize_arithmetic_region(ctx, expression).known_value_type
}

fn known_leaf_value_type(ctx: &mut CheckingContext<'_>, expression: &Expr) -> Option<crate::identity::TypeId> {
    match expression {
        Expr::Int { .. } => {
            let declaration = ctx.core_ids.int.clone();
            ctx.core_type(&declaration)
        }
        Expr::Float { .. } => {
            let declaration = ctx.core_ids.float.clone();
            ctx.core_type(&declaration)
        }
        Expr::Imaginary { coefficient, .. } => match coefficient {
            ImaginaryCoefficient::Int { .. } => {
                let declaration = ctx.core_ids.int.clone();
                ctx.core_type(&declaration)
            }
            ImaginaryCoefficient::Float(_) => {
                let declaration = ctx.core_ids.float.clone();
                ctx.core_type(&declaration)
            }
        },
        Expr::Var { value, .. } => ctx.lookup_local_knowledge(value).and_then(|knowledge| knowledge.ty()),
        Expr::GetProperty(get) => {
            let Expr::Var { value: module_alias, .. } = &get.object else {
                return None;
            };
            ctx.resolve_module_export_value(module_alias, &get.property)
                .and_then(|fact| fact.knowledge.ty())
        }
        _ => None,
    }
}

fn combine_known_numeric_types(
    ctx: &mut CheckingContext<'_>,
    left: Option<crate::identity::TypeId>,
    right: Option<crate::identity::TypeId>,
) -> Option<crate::identity::TypeId> {
    let (Some(left), Some(right)) = (left, right) else {
        return None;
    };
    (left == right && is_core_numeric_type(ctx, left)).then_some(left)
}

fn imaginary_axis(axis: ImaginaryAxis) -> AlgebraicBasisAxis {
    match axis {
        ImaginaryAxis::I => AlgebraicBasisAxis::I,
        ImaginaryAxis::J => AlgebraicBasisAxis::J,
        ImaginaryAxis::K => AlgebraicBasisAxis::K,
    }
}

fn is_core_numeric_type(ctx: &mut CheckingContext<'_>, ty: crate::identity::TypeId) -> bool {
    let int_decl = ctx.core_ids.int.clone();
    let float_decl = ctx.core_ids.float.clone();
    ctx.core_type(&int_decl) == Some(ty) || ctx.core_type(&float_decl) == Some(ty)
}

pub(crate) fn is_embeddable_scalar_source(ctx: &mut CheckingContext<'_>, expression: &Expr) -> bool {
    known_value_type(ctx, expression).is_some_and(|ty| canonical_axis_for_type(ctx, ty).is_none() && !is_canonical_algebra_type(ctx, ty))
}

pub(crate) fn is_canonical_algebra_type(ctx: &CheckingContext<'_>, ty: crate::identity::TypeId) -> bool {
    let Some((declaration, arguments)) = ctx.store.applied_nominal_parts(ty) else {
        return false;
    };
    let Some(complex) = standard_math_declaration(&["numbers", "complex"], "Complex") else {
        return false;
    };
    let Some(quaternion) = standard_math_declaration(&["numbers", "quaternion"], "Quaternion") else {
        return false;
    };
    arguments.len() == 1 && (declaration == complex || declaration == quaternion)
}

fn axis_index(axis: AlgebraicBasisAxis) -> usize {
    match axis {
        AlgebraicBasisAxis::I => 0,
        AlgebraicBasisAxis::J => 1,
        AlgebraicBasisAxis::K => 2,
    }
}

/// Replaces an expected basis value, suffix literal, or real scalar term with
/// its target type only when the exact class-side capability and parameter
/// contracts are available. Unresolved expectations are left to the enclosing
/// inference region.
pub(crate) fn apply_contextual_realization(
    ctx: &mut CheckingContext<'_>,
    expression_id: ExpressionId,
    expression: &Expr,
    expected: &ExpectedType,
    typed: &mut TypedExpression,
) {
    let Some(target_type) = expected.ty() else {
        return;
    };
    if !expected_is_concrete(expected) || !typed.status.is_ready() || typed.causal_invalidity != crate::checker::causal::CausalInvalidity::Clean {
        return;
    }

    let Some(candidate) = classify_realization_candidate(ctx, expression, expected, target_type, typed) else {
        return;
    };
    let scalar_type = match candidate {
        RealizationCandidate::RealScalar { scalar_type } => scalar_type,
        RealizationCandidate::Basis(source) => {
            let (axis, source_type, coefficient_source, evaluation) = match source {
                RealizationSource::Basis { ty, axis } => (axis, ty, None, RealizationEvaluationPlan::MultiplicativeIdentityThenSourceBasis),
                RealizationSource::Literal { scalar, axis, coefficient } => {
                    (axis, scalar, Some(coefficient), RealizationEvaluationPlan::LiteralCoefficientThenAxis)
                }
            };
            let Some(basis_type) = canonical_basis_type(ctx, axis) else {
                return;
            };

            // Keep an unelaborated basis type on failure so normal expected-type
            // checking reports a mismatch instead of silently accepting an unknown.
            if coefficient_source.is_some() {
                typed.knowledge = TypeKnowledge::established(basis_type, EvidenceOrigin::Syntax).with_range(expression.range());
            }

            let Some(realization) = resolve_capability(ctx, target_type, &[basis_type], "along", 2) else {
                return;
            };
            let Some(signature) = capability_signature(&realization) else {
                return;
            };
            if signature.parameters.len() != 2
                || signature.parameters[1].declared_type.canonical_type() != Some(basis_type)
                || signature.declared_return.canonical_type() != Some(target_type)
            {
                return;
            }
            let Some(coefficient_type) = signature.parameters[0].declared_type.canonical_type() else {
                return;
            };

            let coefficient = match coefficient_source {
                Some(source) => {
                    if source_type != coefficient_type {
                        return;
                    }
                    source
                }
                None => {
                    let Some(one) = resolve_capability(ctx, coefficient_type, &[], "one", 0) else {
                        return;
                    };
                    let Some(one_signature) = capability_signature(&one) else {
                        return;
                    };
                    if !one_signature.parameters.is_empty() || one_signature.declared_return.canonical_type() != Some(coefficient_type) {
                        return;
                    }
                    RealizationCoefficientSource::One {
                        scalar_type: coefficient_type,
                        one,
                    }
                }
            };

            let mut assumed = !selection_is_exact(&realization);
            if let RealizationCoefficientSource::One { one, .. } = &coefficient {
                assumed |= !selection_is_exact(one);
            }
            typed.knowledge = match assumed {
                true => TypeKnowledge::assumed(target_type, EvidenceOrigin::ContextualDerivation).with_range(expression.range()),
                false => TypeKnowledge::established(target_type, EvidenceOrigin::ContextualDerivation).with_range(expression.range()),
            };
            typed.contextual_realization = Some(Arc::new(ContextualRealization {
                source_expression: expression_id,
                source_range: expression.range(),
                source_type,
                target_type,
                operation: ContextualRealizationOperation::Basis {
                    basis_type,
                    axis,
                    coefficient,
                    evaluation,
                },
                realization,
            }));
            return;
        }
    };

    let Some(realization) = resolve_capability(ctx, target_type, &[], "fromReal", 1) else {
        return;
    };
    let Some(signature) = capability_signature(&realization) else {
        return;
    };
    if signature.parameters.len() != 1
        || signature.parameters[0].declared_type.canonical_type() != Some(scalar_type)
        || signature.declared_return.canonical_type() != Some(target_type)
    {
        return;
    }
    let assumed = !selection_is_exact(&realization);
    typed.knowledge = match assumed {
        true => TypeKnowledge::assumed(target_type, EvidenceOrigin::ContextualDerivation).with_range(expression.range()),
        false => TypeKnowledge::established(target_type, EvidenceOrigin::ContextualDerivation).with_range(expression.range()),
    };
    typed.contextual_realization = Some(Arc::new(ContextualRealization {
        source_expression: expression_id,
        source_range: expression.range(),
        source_type: scalar_type,
        target_type,
        operation: ContextualRealizationOperation::RealEmbedding {
            value: expression_id,
            scalar_type,
        },
        realization,
    }));
}

enum RealizationCandidate {
    Basis(RealizationSource),
    RealScalar { scalar_type: crate::identity::TypeId },
}

fn classify_realization_candidate(
    ctx: &mut CheckingContext<'_>,
    expression: &Expr,
    expected: &ExpectedType,
    target_type: crate::identity::TypeId,
    typed: &TypedExpression,
) -> Option<RealizationCandidate> {
    if let Some(source) = realization_source(ctx, expression, typed) {
        return (expected_is_concrete(expected) || realization_context_is_algebraic(ctx, expected, target_type)).then_some(RealizationCandidate::Basis(source));
    }

    if !is_embeddable_scalar_source(ctx, expression) {
        return None;
    }
    let scalar_type = typed.knowledge.ty()?;
    if scalar_type == target_type {
        return None;
    }
    if !expected_is_concrete(expected) && !realization_context_is_algebraic(ctx, expected, target_type) {
        return None;
    }
    (known_value_type(ctx, expression) == Some(scalar_type)).then_some(RealizationCandidate::RealScalar { scalar_type })
}

fn realization_context_is_algebraic(ctx: &mut CheckingContext<'_>, expected: &ExpectedType, target_type: crate::identity::TypeId) -> bool {
    is_canonical_algebra_type(ctx, target_type) || expected.origin() == Some(crate::checker::expected::ExpectationOrigin::AlgebraicArithmetic)
}

/// Selects the class-side basis operation for a source-ordered scalar-times-
/// basis expression. Either multiplication orientation is accepted. The
/// product is attached to the whole binary expression, so lowering evaluates
/// the original coefficient and basis operands once in source order.
pub(crate) fn realize_scalar_basis_product(
    ctx: &mut CheckingContext<'_>,
    binary: &BinaryExpr,
    left: &TypedExpression,
    right: &TypedExpression,
    expected: &ExpectedType,
) -> Option<TypedExpression> {
    if !matches!(binary.op, BinaryOp::Multiply)
        || !left.status.is_ready()
        || !right.status.is_ready()
        || left.causal_invalidity != crate::checker::causal::CausalInvalidity::Clean
        || right.causal_invalidity != crate::checker::causal::CausalInvalidity::Clean
    {
        return None;
    }
    let left_basis = realization_source(ctx, &binary.left, left);
    let right_basis = realization_source(ctx, &binary.right, right);
    let (scalar_type, axis, coefficient_typed, basis_typed) = match (left_basis, right_basis) {
        (None, Some(RealizationSource::Basis { axis, .. })) => (left.knowledge.ty()?, axis, left, right),
        (Some(RealizationSource::Basis { axis, .. }), None) => (right.knowledge.ty()?, axis, right, left),
        _ => return None,
    };
    let expression = ctx.current_expression_id()?;
    let coefficient_expression = coefficient_typed.expression_id?;
    let basis_expression = basis_typed.expression_id?;
    let basis_type = canonical_basis_type(ctx, axis)?;

    let (target_type, is_explicit) = if expected_is_concrete(expected) {
        (expected.ty()?, true)
    } else if expected.is_none() {
        (canonical_algebra_type(ctx, scalar_type, axis)?, false)
    } else {
        return None;
    };
    if !is_explicit {
        let (target_declaration, target_arguments) = ctx.store.applied_nominal_parts(target_type)?;
        let complex_declaration = standard_math_declaration(&["numbers", "complex"], "Complex")?;
        let quaternion_declaration = standard_math_declaration(&["numbers", "quaternion"], "Quaternion")?;
        if target_arguments.len() != 1
            || (target_declaration != complex_declaration && target_declaration != quaternion_declaration)
            || target_arguments[0] != scalar_type
            || (target_declaration == complex_declaration && axis != AlgebraicBasisAxis::I)
        {
            return None;
        }
    }

    let realization = resolve_capability(ctx, target_type, &[basis_type], "along", 2)?;
    let signature = capability_signature(&realization)?;
    if signature.parameters.len() != 2
        || signature.parameters[0].declared_type.canonical_type() != Some(scalar_type)
        || signature.parameters[1].declared_type.canonical_type() != Some(basis_type)
        || signature.declared_return.canonical_type() != Some(target_type)
    {
        return None;
    }

    let coefficient = RealizationCoefficientSource::Expression {
        expression: coefficient_expression,
        scalar_type,
    };
    let assumed = !selection_is_exact(&realization);
    let mut typed = TypedExpression::new(match assumed {
        true => TypeKnowledge::assumed(target_type, EvidenceOrigin::ContextualDerivation),
        false => TypeKnowledge::established(target_type, EvidenceOrigin::ContextualDerivation),
    });
    typed.knowledge = typed.knowledge.with_range(binary.range);
    typed.contextual_realization = Some(Arc::new(ContextualRealization {
        source_expression: expression,
        source_range: binary.range,
        source_type: scalar_type,
        target_type,
        operation: ContextualRealizationOperation::Basis {
            basis_type,
            axis,
            coefficient,
            evaluation: RealizationEvaluationPlan::SourceOperands {
                first: left.expression_id?,
                second: right.expression_id?,
                coefficient: coefficient_expression,
                basis: basis_expression,
            },
        },
        realization,
    }));
    Some(typed)
}

fn expected_is_concrete(expected: &ExpectedType) -> bool {
    match expected {
        ExpectedType::Proper { .. } => true,
        ExpectedType::Inference {
            term: crate::checker::inference::InferenceTerm::Canonical(_),
            ..
        } => true,
        ExpectedType::None | ExpectedType::Inference { .. } => false,
    }
}

fn realization_source(ctx: &mut CheckingContext<'_>, expression: &Expr, typed: &TypedExpression) -> Option<RealizationSource> {
    match expression {
        Expr::Imaginary { coefficient, axis, .. } => {
            let axis = match axis {
                ImaginaryAxis::I => AlgebraicBasisAxis::I,
                ImaginaryAxis::J => AlgebraicBasisAxis::J,
                ImaginaryAxis::K => AlgebraicBasisAxis::K,
            };
            let (scalar, coefficient) = match coefficient {
                ImaginaryCoefficient::Int { digits, radix } => (
                    ctx.core_type(&ctx.core_ids.int.clone())?,
                    RealizationCoefficientSource::IntegerLiteral {
                        digits: digits.clone().into_boxed_str(),
                        radix: *radix,
                    },
                ),
                ImaginaryCoefficient::Float(value) => (
                    ctx.core_type(&ctx.core_ids.float.clone())?,
                    RealizationCoefficientSource::FloatLiteral { bits: value.to_bits() },
                ),
            };
            Some(RealizationSource::Literal { scalar, axis, coefficient })
        }
        _ => {
            let source_type = typed.knowledge.ty()?;
            let (declaration, arguments) = ctx.store.applied_nominal_parts(source_type)?;
            if declaration != standard_math_declaration(&["numbers", "algebra"], "Basis")? || arguments.len() != 1 {
                return None;
            }
            let axis = canonical_axis_for_type(ctx, arguments[0])?;
            Some(RealizationSource::Basis { ty: source_type, axis })
        }
    }
}

fn canonical_basis_type(ctx: &mut CheckingContext<'_>, axis: AlgebraicBasisAxis) -> Option<crate::identity::TypeId> {
    let axis_decl = axis_declaration(axis)?;
    let axis_type = ctx.instance_type_of(&axis_decl)?;
    let basis_decl = standard_math_declaration(&["numbers", "algebra"], "Basis")?;
    let basis_form = ctx.nominal_type_of(&basis_decl)?;
    ctx.store.apply_type_form(basis_form, &[axis_type]).ok()
}

fn canonical_algebra_type(ctx: &mut CheckingContext<'_>, scalar: crate::identity::TypeId, axis: AlgebraicBasisAxis) -> Option<crate::identity::TypeId> {
    let (path, name) = if axis == AlgebraicBasisAxis::I {
        (&["numbers", "complex"][..], "Complex")
    } else {
        (&["numbers", "quaternion"][..], "Quaternion")
    };
    let declaration = standard_math_declaration(path, name)?;
    let form = ctx.nominal_type_of(&declaration)?;
    ctx.store.apply_type_form(form, &[scalar]).ok()
}

fn canonical_axis_for_type(ctx: &mut CheckingContext<'_>, ty: crate::identity::TypeId) -> Option<AlgebraicBasisAxis> {
    for axis in [AlgebraicBasisAxis::I, AlgebraicBasisAxis::J, AlgebraicBasisAxis::K] {
        let declaration = axis_declaration(axis)?;
        let axis_type = ctx.instance_type_of(&declaration)?;
        if ty == axis_type {
            return Some(axis);
        }
    }
    None
}

fn axis_declaration(axis: AlgebraicBasisAxis) -> Option<DeclarationId> {
    let name = match axis {
        AlgebraicBasisAxis::I => "I",
        AlgebraicBasisAxis::J => "J",
        AlgebraicBasisAxis::K => "K",
    };
    standard_math_declaration(&["numbers", "algebra"], name)
}

fn standard_math_declaration(path: &[&str], name: &str) -> Option<DeclarationId> {
    let components = path
        .iter()
        .map(|component| ModuleComponent::from_identifier(*component).ok())
        .collect::<Option<Vec<_>>>()?;
    Some(DeclarationId::new(ModuleId::universe(ModulePath::from_components(components)), name.into()))
}

fn resolve_capability(
    ctx: &mut CheckingContext<'_>,
    subject: crate::identity::TypeId,
    trait_arguments: &[crate::identity::TypeId],
    member_name: &str,
    positional_arguments: usize,
) -> Option<RealizationCapabilitySelection> {
    if let Some(declaration) = ctx.store.nominal_origin_declaration(subject).cloned() {
        ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::DeclarationShell(declaration));
    }
    let (trait_name, trait_path): (&str, &[&str]) = match member_name {
        "one" => ("One", &["numbers", "capabilities"]),
        "fromReal" => ("RealEmbeddable", &["numbers", "algebra"]),
        _ => ("BasisRealizable", &["numbers", "algebra"]),
    };
    let trait_declaration = standard_math_declaration(trait_path, trait_name)?;
    let view = ctx.conformance_semantics?;
    let surface = view.trait_surfaces.get(&trait_declaration)?;
    let selector = if positional_arguments == 0 {
        Selector::getter(member_name).ok()?
    } else {
        Selector::method(member_name, vec![SelectorSlot::Positional; positional_arguments]).ok()?
    };
    if let Some(target_family) = crate::trait_dispatch::target_family_for_receiver(ctx.store, subject) {
        ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitDispatchBucket(
            crate::trait_dispatch::TraitDispatchBucketKey {
                target_family,
                selector: selector.clone(),
                side: DispatchSide::Class,
            },
        ));
    }
    let mut matching = surface
        .iter()
        .filter(|(_, member)| member.requirement.selector == selector && member.requirement.side == DispatchSide::Class);
    let (requirement, member) = matching.next()?;
    if matching.next().is_some() {
        return None;
    }
    let trait_ref = TraitRef::new(trait_declaration.clone(), trait_arguments.to_vec().into_boxed_slice());
    let conformance_key = crate::generic_evidence::GenericConformanceKey::new(TypeTerm::Canonical(subject), trait_ref.clone());
    let proof = crate::generic_evidence::query_generic_conformance_proof(
        &TypeTerm::Canonical(subject),
        &trait_ref,
        Some(&ctx.generic_evidence),
        Some(view.conformance_index),
        Some(view.witness_plans),
        Some(view.trait_surfaces),
        Some(view.derived_conformance_rules),
        Some(ctx.declarations),
        Some(ctx.hierarchy.inner()),
        Some(&mut *ctx.store),
    );
    ctx.record_exact_conformance_query_dependency(&conformance_key, &proof);
    let selection = match proof {
        GenericConformanceProof::Proven(evidence @ (GenericConformanceEvidence::Exact(_) | GenericConformanceEvidence::Derived(_))) => {
            let selected = crate::trait_dispatch::selection_from_proven_requirement(evidence, subject, requirement, ctx.store)?;
            if selected.exact_trait_ref != trait_ref || selected.exact_target != subject || selected.callable.is_none() {
                return None;
            }
            let selection = RealizationCapabilitySelection::Evidenced(selected);
            ctx.record_contextual_realization_dependencies(&selection);
            selection
        }
        GenericConformanceProof::Proven(GenericConformanceEvidence::Assumed(evidence)) => {
            if evidence.key.subject != TypeTerm::Canonical(subject) || evidence.key.trait_ref != trait_ref {
                return None;
            }
            let signature = instantiate_assumed_signature(ctx, surface, member, &trait_ref, subject);
            let selection = RealizationCapabilitySelection::Assumed(AssumedTraitDispatchSelection {
                conformance: evidence.key,
                requirement: requirement.clone(),
                signature,
            });
            ctx.record_contextual_realization_dependencies(&selection);
            selection
        }
        _ => return None,
    };
    Some(selection)
}

fn instantiate_assumed_signature(
    ctx: &mut CheckingContext<'_>,
    surface: &crate::traits::TraitSurface,
    member: &TraitSurfaceMember,
    trait_ref: &TraitRef,
    subject: crate::identity::TypeId,
) -> crate::signature::CallableSemanticSignature {
    let mut environment = TypeEnvironment::new();
    if let Some(generic) = &surface.generic_signature {
        for (&parameter, &argument) in generic.parameters.iter().zip(trait_ref.arguments.iter()) {
            environment.bind_param(parameter, argument).expect("realization environment is source mutable");
        }
    }
    environment.bind_self(subject).expect("realization environment is source mutable");
    member.instantiate(ctx.store, &environment).signature
}

fn capability_signature(selection: &RealizationCapabilitySelection) -> Option<&crate::signature::CallableSemanticSignature> {
    match selection {
        RealizationCapabilitySelection::Evidenced(selection) => Some(&selection.signature),
        RealizationCapabilitySelection::Assumed(selection) => Some(&selection.signature),
    }
}

fn selection_is_exact(selection: &RealizationCapabilitySelection) -> bool {
    matches!(
        selection,
        RealizationCapabilitySelection::Evidenced(TraitDispatchSelection {
            evidence: GenericConformanceEvidence::Exact(_),
            ..
        })
    )
}

#[cfg(test)]
mod tests {
    use super::contains_arithmetic_basis;
    use crate::checker::context::CheckingContext;
    use crate::core_surface::universe_declaration;
    use crate::declarations::bootstrap_universe_declarations;
    use crate::types::annotation::SimpleTypeResolver;
    use crate::types::relation::MapTypeHierarchy;
    use crate::types::store::TypeStore;
    use phalcom_ast::ast::{BinaryExpr, BinaryOp, Expr, ImaginaryAxis, ImaginaryCoefficient};
    use phalcom_common::range::SourceRange;
    use phalcom_modules::ModuleId;

    #[test]
    fn deep_arithmetic_region_summary_is_stack_safe() {
        let module = ModuleId::universe_root();
        let mut store = TypeStore::new();
        let declarations = bootstrap_universe_declarations(&mut store, &universe_declaration);
        let resolver = SimpleTypeResolver::new();
        let hierarchy = MapTypeHierarchy::new();
        let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module);

        let range = SourceRange { start: 0, end: 1 };
        let mut expression = Expr::Imaginary {
            coefficient: ImaginaryCoefficient::Int {
                digits: "1".to_owned(),
                radix: 10,
            },
            axis: ImaginaryAxis::I,
            range,
        };
        for _ in 0..25_000 {
            expression = Expr::Binary(Box::new(BinaryExpr {
                op: BinaryOp::Add,
                op_range: None,
                left: expression,
                right: Expr::Int {
                    digits: "1".to_owned(),
                    radix: 10,
                    range,
                },
                range,
            }));
        }

        let has_basis = contains_arithmetic_basis(&mut ctx, &expression);
        // The nested AST's default destructor is recursive too; keep this test
        // focused on the analyzer's traversal rather than recursive drop glue.
        std::mem::forget(expression);
        assert!(has_basis, "the iterative summary must retain the I-axis basis fact");
    }

    #[test]
    fn ordinary_int_expression_is_not_a_realization_candidate() {
        use super::classify_realization_candidate;
        use crate::checker::expected::ExpectedType;
        use crate::checker::typed_expr::TypedExpression;
        use crate::types::evidence::{EvidenceOrigin, TypeKnowledge};

        let module = ModuleId::universe_root();
        let mut store = TypeStore::new();
        let declarations = bootstrap_universe_declarations(&mut store, &universe_declaration);
        let resolver = SimpleTypeResolver::new();
        let hierarchy = MapTypeHierarchy::new();
        let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module);
        let int_declaration = ctx.core_ids.int.clone();
        let int_type = ctx.core_type(&int_declaration).expect("canonical Int type");
        let range = SourceRange { start: 0, end: 1 };
        let expression = Expr::Int {
            digits: "1".to_owned(),
            radix: 10,
            range,
        };
        let typed = TypedExpression::new(TypeKnowledge::established(int_type, EvidenceOrigin::Syntax).with_range(range));

        let candidate = classify_realization_candidate(&mut ctx, &expression, &ExpectedType::proper(int_type), int_type, &typed);
        assert!(candidate.is_none(), "ordinary `Int` checking must stop before capability resolution");
    }

    #[test]
    fn small_arithmetic_regions_keep_axis_and_scalar_facts() {
        use super::summarize_arithmetic_region;

        let module = ModuleId::universe_root();
        let mut store = TypeStore::new();
        let declarations = bootstrap_universe_declarations(&mut store, &universe_declaration);
        let resolver = SimpleTypeResolver::new();
        let hierarchy = MapTypeHierarchy::new();
        let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module);
        let range = SourceRange { start: 0, end: 1 };
        let int = || Expr::Int {
            digits: "1".to_owned(),
            radix: 10,
            range,
        };
        let imaginary = |axis| Expr::Imaginary {
            coefficient: ImaginaryCoefficient::Int {
                digits: "2".to_owned(),
                radix: 10,
            },
            axis,
            range,
        };
        let binary = |op, left, right| {
            Expr::Binary(Box::new(BinaryExpr {
                op,
                op_range: None,
                left,
                right,
                range,
            }))
        };

        let expressions = [
            ("2i", imaginary(ImaginaryAxis::I), 1),
            ("2*i", binary(BinaryOp::Multiply, int(), imaginary(ImaginaryAxis::I)), 1),
            ("i*2", binary(BinaryOp::Multiply, imaginary(ImaginaryAxis::I), int()), 1),
            ("1+2i", binary(BinaryOp::Add, int(), imaginary(ImaginaryAxis::I)), 1),
            (
                "1+2i+3j",
                binary(
                    BinaryOp::Add,
                    binary(BinaryOp::Add, int(), imaginary(ImaginaryAxis::I)),
                    imaginary(ImaginaryAxis::J),
                ),
                0b011,
            ),
            (
                "1+2i+3j+4k",
                binary(
                    BinaryOp::Add,
                    binary(
                        BinaryOp::Add,
                        binary(BinaryOp::Add, int(), imaginary(ImaginaryAxis::I)),
                        imaginary(ImaginaryAxis::J),
                    ),
                    imaginary(ImaginaryAxis::K),
                ),
                0b111,
            ),
        ];

        for (source, expression, expected_axes) in expressions {
            let summary = summarize_arithmetic_region(&mut ctx, &expression);
            assert!(summary.facts.has_basis, "{source}");
            assert_eq!(summary.facts.axes, expected_axes, "{source}");
            assert!(!summary.facts.has_unresolved_coefficient_type, "{source}");
            let scalar_types = &summary.facts.scalar_types;
            assert!(!scalar_types.is_empty(), "{source} retains at least one scalar identity");
            assert!(scalar_types.iter().all(|scalar| *scalar == scalar_types[0]), "{source} scalar identities agree");
        }
    }
}
