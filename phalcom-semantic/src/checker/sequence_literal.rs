//! Semantic family selection, element inference, and exact dimension proofs.
//!
//! Exact cardinality evidence always identifies its intrinsic execution lane.
//! Runtime-sized iteration never establishes a static Vector dimension.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use super::analysis::{SequenceConstructionPlan, SequenceExpansionResolution, SequenceLiteralFamily, SequenceLiteralResolution};
use super::annotation_elaboration::{AnnotationElaboration, elaborate_annotation};
use super::context::{CheckingContext, EvaluationPhase};
use super::expected::{ExpectationOrigin, ExpectedType, canonical_collection_arguments, inference_origin_matches};
use super::expression::{analyze_expression, check_typed_expr_with_publication};
use super::inference::{ConstraintOrigin, InferenceRelation, InferenceTerm};
use super::typed_expr::TypedExpression;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::store::{AnonymousTupleRemainder, TypeData};
use num_bigint::BigInt;
use phalcom_ast::ast::{BracketLiteralElement, BracketLiteralExpr, StaticSymbolRef, TypeAnnotation, TypeAnnotationExpr};
use phalcom_common::range::SourceRange;

fn error(ctx: &mut CheckingContext<'_>, code: DiagnosticCode, message: impl Into<String>, range: SourceRange) {
    ctx.emit_diagnostic(SemanticDiagnostic::error_in(ctx.current_module.clone(), code, message, range));
}

fn family(ctx: &CheckingContext<'_>, expected: &ExpectedType) -> Option<SequenceLiteralFamily> {
    [
        (SequenceLiteralFamily::List, &ctx.core_ids.list, 1),
        (SequenceLiteralFamily::Array, &ctx.core_ids.array, 1),
        (SequenceLiteralFamily::Vector, &ctx.core_ids.vector, 2),
    ]
    .into_iter()
    .find_map(|(family, declaration, arity)| {
        (expected
            .ty()
            .is_some_and(|ty| canonical_collection_arguments(ctx.store, ty, declaration, arity).is_some())
            || inference_origin_matches(ctx.store, expected, declaration))
        .then_some(family)
    })
}

fn default_family(ctx: &CheckingContext<'_>) -> SequenceLiteralFamily {
    match ctx.phase {
        EvaluationPhase::Runtime => SequenceLiteralFamily::Array,
        EvaluationPhase::Static => SequenceLiteralFamily::Vector,
    }
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
        && let ExpectedType::Inference {
            context,
            term: InferenceTerm::Applied { arguments, .. },
            ..
        } = expected
    {
        return arguments
            .get(index)
            .and_then(|argument| argument.term())
            .cloned()
            .map(|term| ExpectedType::inference_from(*context, term, ExpectationOrigin::CollectionElement))
            .unwrap_or(ExpectedType::None);
    }
    ExpectedType::None
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConstraintApplication {
    ConcreteSatisfied,
    ConcreteMismatch,
    /// The concrete subtype relation is checked by the caller's canonical relation query.
    ConcreteRelationDeferred,
    /// The canonical inference owner accepted this relation for later solving.
    /// Registration is not proof that the relation will hold.
    InferenceRegistered,
    MissingInferenceOwner,
}

fn register_inference_constraint(
    session: Option<Rc<RefCell<super::inference::InferenceCoordinator>>>,
    term: &InferenceTerm,
    actual: TypeId,
    exact: bool,
) -> ConstraintApplication {
    let Some(session) = session else {
        return ConstraintApplication::MissingInferenceOwner;
    };
    let actual = InferenceTerm::Canonical(actual);
    let relation = if exact {
        InferenceRelation::Equivalent(actual, term.clone())
    } else {
        InferenceRelation::Subtype(actual, term.clone())
    };
    session.borrow_mut().add_constraint(relation, ConstraintOrigin::Explicit, None);
    ConstraintApplication::InferenceRegistered
}

fn constrain(ctx: &mut CheckingContext<'_>, expected: &ExpectedType, actual: TypeId, exact: bool) -> ConstraintApplication {
    if let ExpectedType::Inference { context, term, .. } = expected {
        return register_inference_constraint(ctx.inference_session(*context), term, actual, exact);
    }
    if !exact && expected.ty().is_some() {
        ConstraintApplication::ConcreteRelationDeferred
    } else if expected.ty().is_none_or(|expected| expected == actual) {
        ConstraintApplication::ConcreteSatisfied
    } else {
        ConstraintApplication::ConcreteMismatch
    }
}

fn report_missing_inference_owner(ctx: &mut CheckingContext<'_>, range: SourceRange, role: &'static str) {
    ctx.publish_analysis_incident(format!("sequence {role} constraint has no active query-local inference owner"));
    let _ = ctx.emit_diagnostic(SemanticDiagnostic::error_in(
        ctx.current_module.clone(),
        DiagnosticCode::AnalysisInternalFailure,
        format!("sequence {role} constraint has no active inference owner"),
        range,
    ));
}

/// Checks one neutral bracket literal and publishes only valid construction facts.
pub(crate) fn synthesize(ctx: &mut CheckingContext<'_>, literal: &BracketLiteralExpr, expected: &ExpectedType) -> TypedExpression {
    let contextual_family = family(ctx, expected);
    let selected = match literal.head.as_ref().map(|head| head.arguments.len()) {
        Some(1) => SequenceLiteralFamily::Array,
        Some(2) => SequenceLiteralFamily::Vector,
        Some(_) => {
            error(
                ctx,
                DiagnosticCode::ApplicationTooManyArguments,
                "bracket literal requires one Array argument or two Vector arguments",
                literal.range,
            );
            return TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause);
        }
        None => contextual_family.unwrap_or_else(|| default_family(ctx)),
    };
    if literal.head.is_some() && contextual_family.is_some_and(|contextual| contextual != selected) {
        error(
            ctx,
            DiagnosticCode::ApplicationArgumentKindMismatch,
            "explicit sequence family conflicts with contextual collection family",
            literal.range,
        );
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
        let origin = TypeAnnotation {
            expr: TypeAnnotationExpr::Reference(StaticSymbolRef {
                root: declaration.name.to_string(),
                root_range: head.range,
                members: Vec::new(),
                range: head.range,
            }),
            range: head.range,
        };
        let annotation = TypeAnnotation {
            expr: TypeAnnotationExpr::Application {
                origin: Box::new(origin),
                arguments: head.arguments.clone(),
                range: head.range,
            },
            range: head.range,
        };
        match elaborate_annotation(ctx, &resolver, &annotation) {
            Ok(elaboration) => Some(elaboration),
            Err(diagnostics) => {
                for diagnostic in diagnostics {
                    ctx.emit_diagnostic(diagnostic);
                }
                return TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause);
            }
        }
    } else {
        None
    };
    let head_owner = match &elaboration {
        Some(AnnotationElaboration::Inference { context, frame, .. }) => Some(super::sequence_publication::PublicationOwner(*context, *frame)),
        _ => None,
    };
    if let Some(owner) = head_owner {
        ctx.record_sequence_dependencies(&[owner].into_iter().collect());
    }
    let mut result = (|| {
        let literal_expected = match &elaboration {
            Some(AnnotationElaboration::Canonical(ty)) => ExpectedType::proper(*ty),
            Some(AnnotationElaboration::Inference { context, term, .. }) => {
                ExpectedType::inference_from(*context, term.clone(), ExpectationOrigin::ExplicitCheck)
            }
            None => expected.clone(),
        };
        let element_expected = argument(ctx, &literal_expected, selected, 0);
        let extent_expected = argument(ctx, &literal_expected, selected, 1);
        let element_dependencies = if literal.head.is_none() {
            ctx.sequence_dependencies()
        } else {
            Default::default()
        };
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
                    let typed = check_typed_expr_with_publication(ctx, expr, &element_expected, element_dependencies.clone());
                    if let Some(ty) = typed.knowledge.ty() {
                        element_types.push(ty);
                    } else {
                        valid = false;
                    }
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
                            TypeData::Unit => {
                                exact = Some(SequenceExpansionResolution::TuplePositionals {
                                    extent: ctx.store.const_int(0),
                                })
                            }
                            TypeData::Tuple(tuple) if tuple.positional_remainder == AnonymousTupleRemainder::None => {
                                items.extend(tuple.elements.iter().filter(|entry| entry.label.is_none()).map(|entry| entry.ty));
                                exact = Some(SequenceExpansionResolution::TuplePositionals {
                                    extent: ctx.store.const_int(BigInt::from(items.len())),
                                });
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
                                ctx.positional_expansions
                                    .insert_source(resolution.source_expression, resolution)
                                    .expect("checking context positional expansions are source mutable");
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
                        error(
                            ctx,
                            DiagnosticCode::GenericInferenceUnderconstrained,
                            "Vector spread cardinality is not statically established; use a closed positional product or Vector",
                            *range,
                        );
                        valid = false;
                    }
                    if let Some(exact) = exact {
                        let extent = match exact {
                            SequenceExpansionResolution::TuplePositionals { extent } | SequenceExpansionResolution::VectorStorage { extent } => extent,
                        };
                        match ctx.store.get(extent) {
                            TypeData::ConstInt(value) => concrete_count += value.to_bigint(),
                            _ if symbolic_extent.is_none() => symbolic_extent = Some(extent),
                            _ => valid = false,
                        }
                        exact_expansions.insert(*range, exact);
                    }
                    if !items.is_empty() {
                        let item = ctx.store.union(&items);
                        match constrain(ctx, &element_expected, item, false) {
                            ConstraintApplication::ConcreteSatisfied
                            | ConstraintApplication::ConcreteRelationDeferred
                            | ConstraintApplication::InferenceRegistered => {}
                            ConstraintApplication::ConcreteMismatch => {
                                error(
                                    ctx,
                                    DiagnosticCode::BindingInitializerMismatch,
                                    "sequence spread element is incompatible with the expected element type",
                                    *range,
                                );
                                valid = false;
                            }
                            ConstraintApplication::MissingInferenceOwner => {
                                report_missing_inference_owner(ctx, *range, "spread element");
                                valid = false;
                            }
                        }
                        if let Some(expected) = element_expected.ty()
                            && !crate::types::relation::is_subtype(ctx.store, &ctx.hierarchy, item, expected)
                        {
                            error(
                                ctx,
                                DiagnosticCode::BindingInitializerMismatch,
                                "sequence spread element is incompatible with the expected element type",
                                *range,
                            );
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
                    error(
                        ctx,
                        DiagnosticCode::GenericInferenceUnderconstrained,
                        "symbolic Vector cardinality arithmetic is not established",
                        literal.range,
                    );
                    valid = false;
                }
                symbolic
            } else {
                ctx.store.const_int(concrete_count)
            };
            match constrain(ctx, &extent_expected, extent, true) {
                ConstraintApplication::ConcreteSatisfied | ConstraintApplication::InferenceRegistered => {}
                ConstraintApplication::ConcreteMismatch => {
                    error(
                        ctx,
                        DiagnosticCode::GenericInferenceConflict,
                        "Vector extent mismatch: constructed cardinality differs from the declared extent",
                        literal.range,
                    );
                    valid = false;
                }
                ConstraintApplication::ConcreteRelationDeferred => {
                    ctx.publish_analysis_incident("exact Vector extent constraint was deferred as a subtype relation");
                    let _ = ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::AnalysisInternalFailure,
                        "exact Vector extent constraint was not checked",
                        literal.range,
                    ));
                    valid = false;
                }
                ConstraintApplication::MissingInferenceOwner => {
                    report_missing_inference_owner(ctx, literal.range, "Vector extent");
                    valid = false;
                }
            }
            Some(extent)
        } else {
            None
        };
        let identity_spread = if selected == SequenceLiteralFamily::Vector && symbolic_extent.is_some() {
            match literal.elements.as_slice() {
                [BracketLiteralElement::Expansion { range, .. }] => match exact_expansions.get(range) {
                    Some(SequenceExpansionResolution::VectorStorage { extent: source_extent }) if Some(*source_extent) == extent => {
                        Some((*range, *source_extent))
                    }
                    _ => None,
                },
                _ => None,
            }
        } else {
            None
        };
        if symbolic_extent.is_some() && selected == SequenceLiteralFamily::Vector && identity_spread.is_none() {
            error(
                ctx,
                DiagnosticCode::GenericInferenceUnderconstrained,
                "symbolic Vector construction requires a lone identity storage spread",
                literal.range,
            );
            valid = false;
        }
        let inferred_element = if element_types.is_empty() {
            None
        } else {
            Some(ctx.store.union(&element_types))
        };
        let mut element_type = element_expected.ty().or(inferred_element);

        if let Some(AnnotationElaboration::Inference { context, frame, term, holes }) = elaboration {
            let outcome = ctx
                .inference_session(context)
                .map(|session| ctx.solve_inference_in_frame(&mut session.borrow_mut(), frame));
            let solved = if outcome.as_ref().is_some_and(|outcome| outcome.is_solved()) {
                ctx.inference_session(context)
                    .and_then(|session| session.borrow().materialize(&term, ctx.store).ok())
            } else {
                None
            };
            if let Some(solved) = solved {
                element_type = canonical_collection_arguments(ctx.store, solved, &declaration, if extent.is_some() { 2 } else { 1 })
                    .and_then(|args| args.first().copied());
            } else {
                for hole in holes {
                    error(
                        ctx,
                        DiagnosticCode::GenericInferenceUnderconstrained,
                        "sequence inference hole is underconstrained or conflicting",
                        hole.range,
                    );
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
        // Fixed slots in a partial outer annotation are contracts too. The
        // enclosing inference owner still solves its holes; do not finish it here.
        if literal.head.is_some() && expected.ty().is_none() {
            if let (Some(expected_element), Some(owner)) = (argument(ctx, expected, selected, 0).ty(), ctx.current_expression_id()) {
                let application = ctx.apply_knowledge_against_type_owned(
                    &TypeKnowledge::established(element_type, EvidenceOrigin::Syntax),
                    expected_element,
                    DiagnosticCode::BindingInitializerMismatch,
                    "sequence element type conflicts with its contextual type",
                    literal.range,
                    owner,
                );
                valid &= application.outcome.is_proven();
            }
            if let (Some(expected_extent), Some(actual_extent)) = (argument(ctx, expected, selected, 1).ty(), extent)
                && expected_extent != actual_extent
            {
                error(
                    ctx,
                    DiagnosticCode::GenericInferenceConflict,
                    "Vector extent conflicts with its contextual static argument",
                    literal.range,
                );
                valid = false;
            }
        }
        // An explicit head refines construction but does not erase the surrounding
        // contract. Establish that relation before publishing executable facts.
        if let (Some(expected_type), Some(owner)) = (expected.ty(), ctx.current_expression_id()) {
            let application = ctx.apply_knowledge_against_type_owned(
                &result.knowledge,
                expected_type,
                DiagnosticCode::BindingInitializerMismatch,
                "sequence literal is incompatible with its contextual type",
                literal.range,
                owner,
            );
            valid &= application.outcome.is_proven();
        }
        if valid && let Some(expression) = ctx.current_expression_id() {
            ctx.stage_sequence(
                SequenceLiteralResolution {
                    source_expression: expression,
                    source_range: literal.range,
                    family: selected,
                    result_type,
                    element_type,
                    extent,
                    construction: if let Some((source_range, extent)) = identity_spread {
                        SequenceConstructionPlan::VectorIdentitySpread { source_range, extent }
                    } else if has_expansion {
                        SequenceConstructionPlan::Expanded
                    } else {
                        SequenceConstructionPlan::Fixed {
                            element_count: literal.elements.len(),
                        }
                    },
                    exact_expansions: exact_expansions.into(),
                },
                result.publication_dependencies.clone(),
            );
        }
        if !valid {
            result.knowledge = TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause);
        }
        result
    })();
    if let Some(owner) = head_owner {
        if result.knowledge.ty().is_some()
            && result.status.is_ready()
            && matches!(result.causal_invalidity, super::causal::CausalInvalidity::Clean)
            && ctx.owning_cause_for_current_expression().is_none()
        {
            if let Err(status) = ctx.accept_sequence_owner(owner) {
                result.status = status;
            }
        } else {
            ctx.reject_sequence_owner(owner);
        }
        ctx.pop_inference_frame(owner.0, owner.1);
        ctx.finish_inference_context(owner.0);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{ConstraintApplication, register_inference_constraint};
    use crate::checker::analysis::SequenceConstructionPlan;
    use crate::checker::inference::{InferenceCoordinator, InferenceTerm};
    use crate::types::id::TypeId;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn missing_inference_session_is_not_treated_as_registered_or_proven() {
        let result = register_inference_constraint(None, &InferenceTerm::Canonical(TypeId(0)), TypeId(1), false);
        assert_eq!(result, ConstraintApplication::MissingInferenceOwner);
    }

    #[test]
    fn live_inference_session_reports_registration_without_claiming_proof() {
        let session = Rc::new(RefCell::new(InferenceCoordinator::new()));
        let result = register_inference_constraint(Some(session), &InferenceTerm::Canonical(TypeId(0)), TypeId(1), false);
        assert_eq!(result, ConstraintApplication::InferenceRegistered);
    }

    #[test]
    fn fixed_sequence_count_preserves_host_lengths_above_u32() {
        let Some(count) = usize::try_from(u64::from(u32::MAX) + 1).ok() else {
            return;
        };
        let construction = SequenceConstructionPlan::Fixed { element_count: count };
        assert!(matches!(construction, SequenceConstructionPlan::Fixed { element_count } if element_count == count));
    }
}
