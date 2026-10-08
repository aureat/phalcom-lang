//! Semantic family selection, element inference, and exact dimension proofs.
//!
//! Exact cardinality evidence always identifies its intrinsic execution lane.
//! Runtime-sized iteration never establishes a static Vector dimension.

use super::analysis::{SequenceConstructionPlan, SequenceExpansionResolution, SequenceLiteralFamily, SequenceLiteralResolution};
use super::annotation_elaboration::{AnnotationElaboration, elaborate_annotation};
use super::context::CheckingContext;
use super::expected::{ExpectationOrigin, ExpectedType, canonical_collection_arguments, inference_origin_matches};
use super::expression::analyze_expression;
use super::inference::{ConstraintOrigin, InferenceRelation, InferenceTerm};
use super::typed_expr::TypedExpression;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::store::{AnonymousTupleRemainder, TypeData};
use num_bigint::BigInt;
use phalcom_ast::ast::{BracketLiteralElement, BracketLiteralExpr, StaticSymbolRef, TypeAnnotation, TypeAnnotationExpr};
use phalcom_common::range::SourceRange;
use std::collections::BTreeMap;

fn error(ctx: &mut CheckingContext<'_>, code: DiagnosticCode, message: impl Into<String>, range: SourceRange) {
    ctx.emit_diagnostic(SemanticDiagnostic::error_in(ctx.current_module.clone(), code, message, range));
}

fn family(ctx: &CheckingContext<'_>, expected: &ExpectedType) -> Option<SequenceLiteralFamily> {
    [(SequenceLiteralFamily::List, &ctx.core_ids.list, 1), (SequenceLiteralFamily::Array, &ctx.core_ids.array, 1), (SequenceLiteralFamily::Vector, &ctx.core_ids.vector, 2)]
        .into_iter()
        .find_map(|(family, declaration, arity)| {
            (expected.ty().is_some_and(|ty| canonical_collection_arguments(ctx.store, ty, declaration, arity).is_some())
                || inference_origin_matches(ctx.store, expected, declaration)).then_some(family)
        })
}

fn argument(ctx: &CheckingContext<'_>, expected: &ExpectedType, family: SequenceLiteralFamily, index: usize) -> ExpectedType {
    let (declaration, arity) = match family {
        SequenceLiteralFamily::List => (&ctx.core_ids.list, 1),
        SequenceLiteralFamily::Array => (&ctx.core_ids.array, 1),
        SequenceLiteralFamily::Vector => (&ctx.core_ids.vector, 2),
    };
    if let Some(ty) = expected.ty() {
        return canonical_collection_arguments(ctx.store, ty, declaration, arity)
            .and_then(|arguments| arguments.get(index).copied())
            .map(|ty| ExpectedType::proper_from(ty, ExpectationOrigin::CollectionElement))
            .unwrap_or(ExpectedType::None);
    }
    if inference_origin_matches(ctx.store, expected, declaration)
        && let ExpectedType::Inference { context, term: InferenceTerm::Applied { arguments, .. }, .. } = expected
    {
        return arguments.get(index).and_then(|argument| argument.term()).cloned()
            .map(|term| ExpectedType::inference_from(*context, term, ExpectationOrigin::CollectionElement))
            .unwrap_or(ExpectedType::None);
    }
    ExpectedType::None
}

fn constrain(ctx: &mut CheckingContext<'_>, expected: &ExpectedType, actual: TypeId, exact: bool) -> bool {
    if let ExpectedType::Inference { context, term, .. } = expected {
        if let Some(session) = ctx.inference_session(*context) {
            let actual = InferenceTerm::Canonical(actual);
            let relation = if exact { InferenceRelation::Equivalent(actual, term.clone()) } else { InferenceRelation::Subtype(actual, term.clone()) };
            session.borrow_mut().add_constraint(relation, ConstraintOrigin::Explicit, None);
            return true;
        }
        return false;
    }
    !exact || expected.ty().is_none_or(|expected| expected == actual)
}

/// Checks one neutral bracket literal and publishes only valid construction facts.
pub(crate) fn synthesize(ctx: &mut CheckingContext<'_>, literal: &BracketLiteralExpr, expected: &ExpectedType) -> TypedExpression {
    let contextual_family = family(ctx, expected);
    let selected = match literal.head.as_ref().map(|head| head.arguments.len()) {
        Some(1) => SequenceLiteralFamily::Array,
        Some(2) => SequenceLiteralFamily::Vector,
        Some(_) => {
            error(ctx, DiagnosticCode::ApplicationTooManyArguments, "bracket literal requires one Array argument or two Vector arguments", literal.range);
            return TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause);
        }
        None => contextual_family.unwrap_or(SequenceLiteralFamily::List),
    };
    if literal.head.is_some() && contextual_family.is_some_and(|contextual| contextual != selected) {
        error(ctx, DiagnosticCode::ApplicationArgumentKindMismatch, "explicit sequence family conflicts with contextual collection family", literal.range);
        return TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause);
    }
    let declaration = match selected {
        SequenceLiteralFamily::List => ctx.core_ids.list.clone(),
        SequenceLiteralFamily::Array => ctx.core_ids.array.clone(),
        SequenceLiteralFamily::Vector => ctx.core_ids.vector.clone(),
    };
    let Some(form) = ctx.declarations.form(&declaration) else {
        return TypedExpression::unknown(UnknownReason::NoTypeEvidence);
    };
    let resolver = ctx.resolver.clone();
    let elaboration = if let Some(head) = &literal.head {
        let origin = TypeAnnotation { expr: TypeAnnotationExpr::Reference(StaticSymbolRef {
            root: declaration.name.to_string(), root_range: head.range, members: Vec::new(), range: head.range,
        }), range: head.range };
        let annotation = TypeAnnotation { expr: TypeAnnotationExpr::Application {
            origin: Box::new(origin), arguments: head.arguments.clone(), range: head.range,
        }, range: head.range };
        match elaborate_annotation(ctx, &resolver, &annotation) {
            Ok(elaboration) => Some(elaboration),
            Err(diagnostics) => {
                for diagnostic in diagnostics { ctx.emit_diagnostic(diagnostic); }
                return TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause);
            }
        }
    } else { None };
    let literal_expected = match &elaboration {
        Some(AnnotationElaboration::Canonical(ty)) => ExpectedType::proper(*ty),
        Some(AnnotationElaboration::Inference { context, term, .. }) => ExpectedType::inference_from(*context, term.clone(), ExpectationOrigin::ExplicitCheck),
        None => expected.clone(),
    };
    let element_expected = argument(ctx, &literal_expected, selected, 0);
    let extent_expected = argument(ctx, &literal_expected, selected, 1);
    let mut operands = Vec::new();
    let mut element_types = Vec::new();
    let mut exact_expansions = BTreeMap::new();
    let mut concrete_count = BigInt::from(0);
    let mut symbolic_extent = None;
    let mut valid = true;
    let mut has_expansion = false;

    for element in &literal.elements {
        match element {
            BracketLiteralElement::Element { expr, .. } => {
                let typed = analyze_expression(ctx, expr, &element_expected);
                if let Some(ty) = typed.knowledge.ty() { element_types.push(ty); } else { valid = false; }
                valid &= matches!(typed.causal_invalidity, super::causal::CausalInvalidity::Clean);
                concrete_count += 1;
                operands.push(typed);
            }
            BracketLiteralElement::Expansion { expr, range } => {
                has_expansion = true;
                let typed = analyze_expression(ctx, expr, &ExpectedType::None);
                let source_ty = typed.knowledge.ty();
                let mut exact = None;
                let mut items = Vec::new();
                if let Some(source_ty) = source_ty {
                    match ctx.store.get(source_ty).clone() {
                        TypeData::Unit => exact = Some(SequenceExpansionResolution::TuplePositionals { extent: ctx.store.const_int(0) }),
                        TypeData::Tuple(tuple) if tuple.positional_remainder == AnonymousTupleRemainder::None => {
                            items.extend(tuple.elements.iter().filter(|entry| entry.label.is_none()).map(|entry| entry.ty));
                            exact = Some(SequenceExpansionResolution::TuplePositionals { extent: ctx.store.const_int(BigInt::from(items.len())) });
                        }
                        _ => {
                            if let Some(arguments) = canonical_collection_arguments(ctx.store, source_ty, &ctx.core_ids.vector, 2) {
                                items.push(arguments[0]);
                                exact = Some(SequenceExpansionResolution::VectorStorage { extent: arguments[1] });
                            }
                        }
                    }
                    if exact.is_none() && selected != SequenceLiteralFamily::Vector {
                        let (resolution, _) = super::iteration::resolve_positional_expansion(ctx, typed.expression_id, source_ty, *range);
                        if let Some(resolution) = resolution {
                            items.push(resolution.item_type);
                            ctx.positional_expansions.insert(resolution.source_expression, resolution);
                        } else if let Some(arguments) = canonical_collection_arguments(ctx.store, source_ty, &ctx.core_ids.array, 1)
                            .or_else(|| canonical_collection_arguments(ctx.store, source_ty, &ctx.core_ids.list, 1))
                        {
                            // Canonical runtime-sized sequence declarations
                            // establish their element type even when a reduced
                            // fixture does not load protocol evidence.
                            items.push(arguments[0]);
                        } else {
                            super::iteration::record_known_invalid_positional_expansion(ctx, typed.expression_id, source_ty, *range);
                            valid = false;
                        }
                    }
                }
                if selected == SequenceLiteralFamily::Vector && exact.is_none() {
                    error(ctx, DiagnosticCode::GenericInferenceUnderconstrained, "Vector spread cardinality is not statically established; use a closed positional product or Vector", *range);
                    valid = false;
                }
                if let Some(exact) = exact {
                    let extent = match exact { SequenceExpansionResolution::TuplePositionals { extent } | SequenceExpansionResolution::VectorStorage { extent } => extent };
                    match ctx.store.get(extent) {
                        TypeData::ConstInt(value) => concrete_count += value.to_bigint(),
                        _ if symbolic_extent.is_none() => symbolic_extent = Some(extent),
                        _ => valid = false,
                    }
                    exact_expansions.insert(*range, exact);
                }
                if !items.is_empty() {
                    let item = ctx.store.union(&items);
                    constrain(ctx, &element_expected, item, false);
                    if let Some(expected) = element_expected.ty()
                        && !crate::types::relation::is_subtype(ctx.store, &ctx.hierarchy, item, expected)
                    {
                        error(ctx, DiagnosticCode::BindingInitializerMismatch, "sequence spread element is incompatible with the expected element type", *range);
                        valid = false;
                    }
                    element_types.push(item);
                }
                valid &= matches!(typed.causal_invalidity, super::causal::CausalInvalidity::Clean);
                operands.push(typed);
            }
        }
    }
    let extent = if selected == SequenceLiteralFamily::Vector {
        let extent = if let Some(symbolic) = symbolic_extent {
            if concrete_count != BigInt::from(0) {
                error(ctx, DiagnosticCode::GenericInferenceUnderconstrained, "symbolic Vector cardinality arithmetic is not established", literal.range);
                valid = false;
            }
            symbolic
        } else { ctx.store.const_int(concrete_count) };
        if !constrain(ctx, &extent_expected, extent, true) {
            error(ctx, DiagnosticCode::GenericInferenceConflict, "Vector extent mismatch: constructed cardinality differs from the declared extent", literal.range);
            valid = false;
        }
        Some(extent)
    } else { None };
    let inferred_element = if element_types.is_empty() { None } else { Some(ctx.store.union(&element_types)) };
    let mut element_type = element_expected.ty().or(inferred_element);

    if let Some(AnnotationElaboration::Inference { context, frame, term, holes }) = elaboration {
        let outcome = ctx.inference_session(context).map(|session| ctx.solve_inference_in_frame(&mut session.borrow_mut(), frame));
        let solved = if outcome.as_ref().is_some_and(|outcome| outcome.is_solved()) {
            ctx.inference_session(context).and_then(|session| session.borrow().materialize(&term, ctx.store).ok())
        } else { None };
        ctx.pop_inference_frame(context, frame);
        ctx.finish_inference_context(context);
        if let Some(solved) = solved {
            element_type = canonical_collection_arguments(ctx.store, solved, &declaration, if extent.is_some() { 2 } else { 1 }).and_then(|args| args.first().copied());
        } else {
            for hole in holes {
                error(ctx, DiagnosticCode::GenericInferenceUnderconstrained, "sequence inference hole is underconstrained or conflicting", hole.range);
            }
            valid = false;
        }
    }
    let Some(element_type) = element_type else {
        return TypedExpression::unknown(UnknownReason::NoTypeEvidence);
    };
    let mut arguments = vec![element_type];
    arguments.extend(extent);
    let Ok(result_type) = ctx.store.apply_type_form(form, &arguments) else {
        return TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause);
    };
    let mut result = TypedExpression::established(result_type, EvidenceOrigin::Syntax, literal.range);
    super::composition::propagate_required_dependencies(&mut result, &operands);
    if valid && let Some(expression) = ctx.current_expression_id() {
        ctx.sequence_literals.insert(expression, SequenceLiteralResolution {
            source_expression: expression, source_range: literal.range, family: selected, result_type, element_type, extent,
            construction: if has_expansion { SequenceConstructionPlan::Expanded } else { SequenceConstructionPlan::Fixed { element_count: literal.elements.len() as u32 } },
            exact_expansions,
        });
    }
    if !valid { result.knowledge = TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause); }
    result
}
