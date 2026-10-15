//! Ordinary semantic checking and evaluator layout for static definition bodies.

use crate::checker::analysis::{AnalysisStatus, BindingAnalysisIndex, ExpressionAnalysisIndex, SemanticDependency};
use crate::checker::binding::{BindingContract, BindingContractOrigin, BindingSeed};
use crate::checker::causal::CausalInvalidity;
use crate::checker::context::CheckingContext;
use crate::checker::control::analyze_executable_region;
use crate::checker::expected::{ExpectationOrigin, ExpectedType};
use crate::checker::expression::analyze_expression;
use crate::checker::flow::InitializationState;
use crate::checker::typed_expr::TypedExpression;
use crate::db::key::QueryKey;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::identity::{BindingId, ExpressionId};
use crate::member_projection::{MemberDeclarationIdentity, MemberProjection, MemberProjectionArgument, MemberProjectionSubject};
use crate::static_definition::{CheckedStaticBody, StaticBodyId, StaticDefinition, StaticDefinitionBodyRepr};
use crate::static_domain::StaticValueDomain;
use crate::static_intrinsic::StaticIntrinsicRegistry;
use crate::types::annotation::TypeResolver;
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::store::TypeData;
use crate::types::type_member::{ProjectionSubject, TypeMemberArgument};
use phalcom_ast::ast::{Expr, TypeAnnotationExpr};
use phalcom_common::range::SourceRange;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Checks a static body using the caller's ordinary checker context.
///
/// The context must be fresh for this body, with canonical dispatch/type
/// products attached. Any pre-seeded outer bindings must be listed in
/// `statically_readable_bindings`; other outer binding reads become phase
/// errors. Kinded parameters must already be present in the context resolver's
/// type-parameter scope.
pub fn analyze_static_body(
    context: CheckingContext<'_>,
    definition: &StaticDefinition,
    statically_readable_bindings: &BTreeSet<BindingId>,
    intrinsics: &StaticIntrinsicRegistry,
) -> CheckedStaticBody {
    analyze_static_body_inner(context, definition, statically_readable_bindings, &BTreeMap::new(), None, intrinsics)
}

/// Checks a static body after seeding its lexical scope from checked module
/// binding facts. The stable symbol identities remain paired with the newly
/// allocated body-local binding IDs used by ordinary expression analysis.
pub fn analyze_static_body_with_module_bindings(
    mut context: CheckingContext<'_>,
    definition: &StaticDefinition,
    module_bindings: &crate::module_bindings::ModuleStaticBindingIndex,
    intrinsics: &StaticIntrinsicRegistry,
) -> CheckedStaticBody {
    let mut readable = BTreeSet::new();
    let mut stable_bindings = BTreeMap::new();
    context.push_scope();
    for (visible_name, fact) in module_bindings.visible_bindings() {
        let inserted = context.declare_checked_binding(visible_name.to_owned(), &fact.state, fact.phase);
        if let crate::checker::binding::BindingDeclarationResult::Inserted(binding) = inserted {
            if fact.phase == crate::module_bindings::ModuleBindingPhase::StaticConstant {
                readable.insert(binding);
                stable_bindings.insert(binding, fact.symbol.clone());
            }
        }
    }
    let mut checked = analyze_static_body_inner(
        context,
        definition,
        &readable,
        &stable_bindings,
        Some(module_bindings.product_fingerprint()),
        intrinsics,
    );
    let mut dependencies = checked.dependencies.to_vec();
    dependencies.push(crate::checker::analysis::SemanticDependency::ModuleStaticBindings(
        module_bindings.module().clone(),
    ));
    dependencies.sort();
    dependencies.dedup();
    checked.dependencies = dependencies.into();
    checked
}

/// Checks a static constant's initializer expression using the caller's ordinary
/// checker context and the module's visible static bindings.
pub fn analyze_static_constant_initializer(
    context: CheckingContext<'_>,
    constant_id: &crate::static_constant::StaticConstantId,
    declaration_range: SourceRange,
    initializer_range: SourceRange,
    initializer: &Expr,
    annotation: Option<&phalcom_ast::ast::TypeAnnotation>,
    expected_type: Option<crate::types::id::TypeId>,
    catalog: &crate::module_bindings::ModuleStaticBindingCatalog,
    binding_states: &BTreeMap<phalcom_modules::linker::SymbolId, crate::checker::analysis::BindingState>,
    intrinsics: &StaticIntrinsicRegistry,
) -> crate::static_constant::CheckedStaticConstantInitializer {
    let diagnostic_start = context.diagnostics.len();
    let expected = match expected_type {
        Some(ty) => ExpectedType::proper_from(ty, ExpectationOrigin::DeclarationContract),
        None => ExpectedType::None,
    };
    analyze_static_constant_initializer_with_expected_and_diagnostic_start(
        context,
        constant_id,
        declaration_range,
        initializer_range,
        initializer,
        annotation,
        expected,
        catalog,
        binding_states,
        intrinsics,
        diagnostic_start,
    )
}

/// Checks a static constant's initializer expression using an explicit ExpectedType.
pub fn analyze_static_constant_initializer_with_expected(
    context: CheckingContext<'_>,
    constant_id: &crate::static_constant::StaticConstantId,
    declaration_range: SourceRange,
    initializer_range: SourceRange,
    initializer: &Expr,
    annotation: Option<&phalcom_ast::ast::TypeAnnotation>,
    expected: ExpectedType,
    catalog: &crate::module_bindings::ModuleStaticBindingCatalog,
    binding_states: &BTreeMap<phalcom_modules::linker::SymbolId, crate::checker::analysis::BindingState>,
    intrinsics: &StaticIntrinsicRegistry,
) -> crate::static_constant::CheckedStaticConstantInitializer {
    let diagnostic_start = context.diagnostics.len();
    analyze_static_constant_initializer_with_expected_and_diagnostic_start(
        context,
        constant_id,
        declaration_range,
        initializer_range,
        initializer,
        annotation,
        expected,
        catalog,
        binding_states,
        intrinsics,
        diagnostic_start,
    )
}

/// Variant used when the caller elaborates a partial declaration annotation in
/// this same checker before checking the initializer. The earlier diagnostics
/// belong to the resulting static owner and must remain in its product.
pub fn analyze_static_constant_initializer_with_expected_and_diagnostic_start(
    context: CheckingContext<'_>,
    constant_id: &crate::static_constant::StaticConstantId,
    declaration_range: SourceRange,
    initializer_range: SourceRange,
    initializer: &Expr,
    annotation: Option<&phalcom_ast::ast::TypeAnnotation>,
    expected: ExpectedType,
    catalog: &crate::module_bindings::ModuleStaticBindingCatalog,
    binding_states: &BTreeMap<phalcom_modules::linker::SymbolId, crate::checker::analysis::BindingState>,
    intrinsics: &StaticIntrinsicRegistry,
    diagnostic_start: usize,
) -> crate::static_constant::CheckedStaticConstantInitializer {
    let expected_type = expected.ty();
    let input_fingerprint = crate::db::fingerprint::static_constant_input_fingerprint(constant_id, initializer, annotation, expected_type);
    let expression = analyze_static_expression_with_demands_at(context, initializer, expected, catalog, binding_states, intrinsics, diagnostic_start).0;
    crate::static_constant::CheckedStaticConstantInitializer {
        id: constant_id.clone(),
        declaration_range,
        initializer_range,
        expression,
        record_key_static_constant_reads: Arc::from([]),
        input_fingerprint,
    }
}

/// Checks a static expression through the ordinary checker and captures the
/// reusable expression/evaluator product used by constant initializers and
/// computed Record-key projection.
pub fn analyze_static_expression(
    context: CheckingContext<'_>,
    initializer: &Expr,
    expected_type: Option<crate::types::id::TypeId>,
    catalog: &crate::module_bindings::ModuleStaticBindingCatalog,
    binding_states: &BTreeMap<phalcom_modules::linker::SymbolId, crate::checker::analysis::BindingState>,
    intrinsics: &StaticIntrinsicRegistry,
) -> crate::static_constant::CheckedStaticExpression {
    let expected = match expected_type {
        Some(ty) => ExpectedType::proper_from(ty, ExpectationOrigin::DeclarationContract),
        None => ExpectedType::None,
    };
    analyze_static_expression_with_demands(context, initializer, expected, catalog, binding_states, intrinsics).0
}

/// Result of checking a static expression against the current semantic world.
#[derive(Clone, Debug)]
pub enum StaticExpressionCheckOutcome {
    /// The expression checked without diagnostics or unavailable prerequisites.
    Ready(crate::static_constant::CheckedStaticExpression),
    /// Exact semantic owner products must become current before checking can
    /// be classified as valid or invalid.
    Awaiting(BTreeSet<QueryKey>),
    /// The expression was checked against current inputs and produced source
    /// diagnostics; no owner product may be published from it.
    Invalid(crate::static_constant::CheckedStaticExpression),
}

/// Checks a static expression and preserves semantic-owner readiness as
/// scheduler control instead of converting it into a source diagnostic.
pub fn check_static_expression(
    context: CheckingContext<'_>,
    initializer: &Expr,
    expected_type: Option<crate::types::id::TypeId>,
    catalog: &crate::module_bindings::ModuleStaticBindingCatalog,
    binding_states: &BTreeMap<phalcom_modules::linker::SymbolId, crate::checker::analysis::BindingState>,
    intrinsics: &StaticIntrinsicRegistry,
) -> StaticExpressionCheckOutcome {
    let expected = match expected_type {
        Some(ty) => ExpectedType::proper_from(ty, ExpectationOrigin::DeclarationContract),
        None => ExpectedType::None,
    };
    check_static_expression_with_expected(context, initializer, expected, catalog, binding_states, intrinsics)
}

pub fn check_static_expression_with_expected(
    context: CheckingContext<'_>,
    initializer: &Expr,
    expected: ExpectedType,
    catalog: &crate::module_bindings::ModuleStaticBindingCatalog,
    binding_states: &BTreeMap<phalcom_modules::linker::SymbolId, crate::checker::analysis::BindingState>,
    intrinsics: &StaticIntrinsicRegistry,
) -> StaticExpressionCheckOutcome {
    let diagnostic_start = context.diagnostics.len();
    check_static_expression_with_expected_and_diagnostic_start(context, initializer, expected, catalog, binding_states, intrinsics, diagnostic_start)
}

pub fn check_static_expression_with_expected_and_diagnostic_start(
    context: CheckingContext<'_>,
    initializer: &Expr,
    expected: ExpectedType,
    catalog: &crate::module_bindings::ModuleStaticBindingCatalog,
    binding_states: &BTreeMap<phalcom_modules::linker::SymbolId, crate::checker::analysis::BindingState>,
    intrinsics: &StaticIntrinsicRegistry,
    diagnostic_start: usize,
) -> StaticExpressionCheckOutcome {
    let (checked, demands) = analyze_static_expression_with_demands_at(context, initializer, expected, catalog, binding_states, intrinsics, diagnostic_start);
    if !demands.is_empty() {
        return StaticExpressionCheckOutcome::Awaiting(demands);
    }
    if checked.diagnostics.is_empty() {
        StaticExpressionCheckOutcome::Ready(checked)
    } else {
        StaticExpressionCheckOutcome::Invalid(checked)
    }
}

fn analyze_static_expression_with_demands(
    context: CheckingContext<'_>,
    initializer: &Expr,
    expected: ExpectedType,
    catalog: &crate::module_bindings::ModuleStaticBindingCatalog,
    binding_states: &BTreeMap<phalcom_modules::linker::SymbolId, crate::checker::analysis::BindingState>,
    intrinsics: &StaticIntrinsicRegistry,
) -> (crate::static_constant::CheckedStaticExpression, BTreeSet<QueryKey>) {
    let diagnostic_start = context.diagnostics.len();
    analyze_static_expression_with_demands_at(context, initializer, expected, catalog, binding_states, intrinsics, diagnostic_start)
}

fn analyze_static_expression_with_demands_at(
    mut context: CheckingContext<'_>,
    initializer: &Expr,
    expected: ExpectedType,
    catalog: &crate::module_bindings::ModuleStaticBindingCatalog,
    binding_states: &BTreeMap<phalcom_modules::linker::SymbolId, crate::checker::analysis::BindingState>,
    intrinsics: &StaticIntrinsicRegistry,
    diagnostic_start: usize,
) -> (crate::static_constant::CheckedStaticExpression, BTreeSet<QueryKey>) {
    context.phase = crate::checker::context::EvaluationPhase::Static;
    let existing_expressions = context.expressions.keys().copied().collect::<BTreeSet<_>>();
    let expression_start = context.next_local_expr_id;
    let body_id = context.body_id;
    let runtime_outer_bindings = context
        .scopes
        .iter()
        .flat_map(|scope| scope.values().map(|binding| binding.id))
        .collect::<BTreeSet<_>>();

    context.push_scope();
    let mut readable = BTreeSet::new();
    let mut stable_bindings = BTreeMap::new();
    let mut runtime_module_bindings = BTreeSet::new();
    for (visible_name, seed) in catalog.visible_bindings() {
        let Some(state) = binding_states.get(&seed.symbol) else {
            continue;
        };
        let inserted = context.declare_checked_binding(visible_name.to_owned(), state, seed.phase);
        if let crate::checker::binding::BindingDeclarationResult::Inserted(binding) = inserted {
            if seed.phase == crate::module_bindings::ModuleBindingPhase::StaticConstant {
                readable.insert(binding);
                stable_bindings.insert(binding, seed.symbol.clone());
            } else {
                runtime_module_bindings.insert(binding);
            }
        }
    }
    let first_body_binding = context.next_binding_id;

    let mut typed_result = analyze_expression(&mut context, initializer, &expected);
    context.sync_expression_outcome(&typed_result);

    let runtime_accesses = context
        .expressions
        .values()
        .filter(|expression| !existing_expressions.contains(&expression.id) && expression.id.owner == body_id && expression.id.local.0 >= expression_start)
        .filter_map(|expression| {
            let binding = expression.binding_read.or(expression.binding_write)?;
            ((runtime_module_bindings.contains(&binding) || runtime_outer_bindings.contains(&binding)) && !readable.contains(&binding)).then_some((
                expression.id,
                expression.range,
                binding,
            ))
        })
        .collect::<Vec<(ExpressionId, SourceRange, BindingId)>>();
    for (expression_id, range, binding) in runtime_accesses {
        let message = format!("runtime binding is not accessible in a static constant initializer (binding {binding:?})");
        let diagnostic = SemanticDiagnostic::error_in(context.current_module.clone(), DiagnosticCode::TypeOnlyRuntimeValue, message, range);
        if let Some(cause) = context.emit_diagnostic(diagnostic)
            && let Some(expression) = context.expressions.get_mut(&expression_id)
        {
            expression.status = AnalysisStatus::Invalid(cause);
            expression.causal_invalidity = expression.causal_invalidity.join(CausalInvalidity::One(cause));
        }
    }
    context.pop_scope();

    let source_invalid = context.diagnostics[diagnostic_start..]
        .iter()
        .any(|diagnostic| diagnostic.severity == crate::diagnostic::DiagnosticSeverity::Error);
    if source_invalid {
        context.reject_all_sequence_products();
    }

    let solved_expected_type = if source_invalid {
        None
    } else if let ExpectedType::Inference {
        context: inf_ctx, ref term, ..
    } = expected
    {
        if let Some(session) = context.inference_session(inf_ctx) {
            let frame = context
                .current_inference_frame(inf_ctx)
                .unwrap_or(crate::checker::inference::InferenceFrameId(0));
            let outcome = context.solve_inference_in_frame(&mut session.borrow_mut(), frame);
            let solved = if outcome.is_solved() {
                session.borrow().materialize(term, context.store).ok()
            } else {
                None
            };
            let owner = super::sequence_publication::PublicationOwner(inf_ctx, frame);
            let source_valid = typed_result.status.is_ready()
                && matches!(typed_result.causal_invalidity, CausalInvalidity::Clean)
                && !context.diagnostics[diagnostic_start..]
                    .iter()
                    .any(|diagnostic| diagnostic.severity == crate::diagnostic::DiagnosticSeverity::Error);
            if solved.is_some() && source_valid {
                if let Err(status) = context.accept_sequence_owner(owner) {
                    typed_result.status = status.clone();
                    context.sync_expression_outcome(&typed_result);
                    if !context.diagnostics[diagnostic_start..]
                        .iter()
                        .any(|diagnostic| diagnostic.severity == crate::diagnostic::DiagnosticSeverity::Error)
                    {
                        let (code, message) = match status {
                            AnalysisStatus::Blocked(reason) => {
                                (DiagnosticCode::AnalysisBlocked, format!("static initializer inference is blocked: {reason:?}"))
                            }
                            AnalysisStatus::BudgetExceeded(report) => (
                                DiagnosticCode::AnalysisBudgetExceeded,
                                format!("static initializer inference exceeded budget: {report:?}"),
                            ),
                            AnalysisStatus::InternalFailure(incident) => (
                                DiagnosticCode::AnalysisInternalFailure,
                                format!("static initializer inference failed: {incident:?}"),
                            ),
                            AnalysisStatus::Cancelled => (DiagnosticCode::AnalysisBlocked, "static initializer inference was cancelled".to_string()),
                            _ => (
                                DiagnosticCode::GenericInferenceUnderconstrained,
                                "static initializer inference was not accepted".to_string(),
                            ),
                        };
                        context.emit_diagnostic(SemanticDiagnostic::error_in(context.current_module.clone(), code, message, initializer.range()));
                    }
                }
            } else {
                context.reject_sequence_owner(owner);
                if source_valid {
                    typed_result.status = AnalysisStatus::Blocked(crate::types::outcome::BlockReason::UnknownType(
                        crate::types::evidence::UnknownReason::InferenceBlocked,
                    ));
                    context.sync_expression_outcome(&typed_result);
                    context.emit_diagnostic(SemanticDiagnostic::error_in(
                        context.current_module.clone(),
                        DiagnosticCode::GenericInferenceUnderconstrained,
                        "static initializer inference could not be completed",
                        initializer.range(),
                    ));
                }
            }
            context.pop_inference_frame(inf_ctx, frame);
            context.finish_inference_context(inf_ctx);
            solved
        } else {
            context.publish_analysis_incident("static initializer inference owner is no longer active");
            context.emit_diagnostic(SemanticDiagnostic::error_in(
                context.current_module.clone(),
                DiagnosticCode::AnalysisInternalFailure,
                "static initializer inference owner is no longer active",
                initializer.range(),
            ));
            None
        }
    } else {
        expected.ty()
    };

    // A bare empty bracket literal has a known collection family but supplies
    // no evidence for its element type. Keep family selection in
    // `sequence_literal::synthesize`; once this static initializer owner has
    // had its ordinary expected-type/inference opportunity, report the
    // unresolved element type here instead of silently leaving a closed
    // constant without a usable type or value.
    let unresolved_empty_bracket = matches!(initializer, Expr::BracketLiteral(literal) if literal.elements.is_empty())
        && matches!(&expected, ExpectedType::None)
        && matches!(typed_result.knowledge, TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence));
    if unresolved_empty_bracket
        && !context.diagnostics[diagnostic_start..]
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::GenericInferenceUnderconstrained)
        && let Some(cause) = context.emit_diagnostic(SemanticDiagnostic::error_in(
            context.current_module.clone(),
            DiagnosticCode::GenericInferenceUnderconstrained,
            "empty static sequence literal has no element type evidence",
            initializer.range(),
        ))
    {
        typed_result.invalidate(cause);
        context.sync_expression_outcome(&typed_result);
    }

    context.ensure_sequence_publication_settled(initializer.range());

    let expressions = context
        .expressions
        .iter()
        .filter(|(id, _)| !existing_expressions.contains(id) && id.owner == body_id && id.local.0 >= expression_start)
        .map(|(id, analysis)| (*id, analysis.clone()))
        .collect::<ExpressionAnalysisIndex>();
    let static_operations = intrinsics.resolve_ready_expressions(&expressions);
    let static_constant_reads = expressions
        .values()
        .filter_map(|expression| expression.binding_read.and_then(|binding| stable_bindings.get(&binding).cloned()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let static_applications = std::mem::take(&mut context.checked_static_applications).into_boxed_slice();
    let static_constructions = std::mem::take(&mut context.checked_static_constructions);
    let associated_resolutions = context
        .associated_resolutions
        .iter()
        .filter(|(expression, _)| expressions.contains_key(expression))
        .map(|(expr, res)| (*expr, res.clone()))
        .collect();
    let sequence_literals: crate::checker::analysis::SequenceLiteralResolutionIndex = context
        .sequence_literals
        .iter()
        .filter(|(expression, _)| expressions.contains_key(expression))
        .map(|(expr, res)| (*expr, res.clone()))
        .collect();
    let match_resolutions: crate::match_semantics::MatchResolutionIndex = context
        .match_resolutions
        .iter()
        .filter(|(expression, _)| expressions.contains_key(expression))
        .map(|(expr, res)| (*expr, res.clone()))
        .collect();
    let initializer_range = initializer.range();
    let pattern_site_resolutions: crate::match_semantics::PatternSiteResolutionIndex = context
        .pattern_site_resolutions
        .iter()
        .filter(|(key, _)| initializer_range.contains_range(&key.range))
        .map(|(key, res)| (*key, res.clone()))
        .collect();
    let proof_predicates: crate::checker::proof_predicate::ProofPredicateResolutionIndex = context
        .proof_predicates
        .iter()
        .filter(|(expression, _)| expressions.contains_key(expression))
        .map(|(expr, res)| (*expr, res.clone()))
        .collect();

    let mut bindings: BindingAnalysisIndex = context.binding_history.clone();
    bindings.extend(context.flow.bindings.clone());
    bindings.retain(|binding, _| binding.0 >= first_body_binding || readable.contains(binding));
    let mut local_ids = bindings
        .keys()
        .copied()
        .filter(|binding| binding.0 >= first_body_binding && !stable_bindings.contains_key(binding))
        .collect::<Vec<_>>();
    local_ids.sort_unstable();
    let local_slots = local_ids
        .into_iter()
        .enumerate()
        .map(|(slot, binding)| (binding, slot as u32))
        .collect::<BTreeMap<_, _>>();

    let mut dependencies = context
        .semantic_dependencies_snapshot()
        .into_iter()
        .filter(|dep| !matches!(dep, SemanticDependency::ModuleStaticBindings(_)))
        .collect::<Vec<_>>();
    dependencies.push(SemanticDependency::ModuleStaticBindingCatalog(catalog.module().clone()));
    for read_sym in &static_constant_reads {
        dependencies.push(SemanticDependency::StaticConstant(crate::static_constant::StaticConstantId(read_sym.clone())));
    }
    dependencies.sort();
    dependencies.dedup();

    let diagnostics = Arc::from(context.diagnostics[diagnostic_start..].to_vec().into_boxed_slice());
    let checked = crate::static_constant::CheckedStaticExpression {
        root: initializer.clone(),
        expressions: Arc::new(expressions),
        bindings: Arc::new(bindings),
        static_operations: Arc::new(static_operations),
        static_applications: Arc::from(static_applications),
        static_constructions: Arc::new(static_constructions),
        associated_resolutions: Arc::new(associated_resolutions),
        sequence_literals: Arc::new(sequence_literals),
        match_resolutions: Arc::new(match_resolutions),
        pattern_site_resolutions: Arc::new(pattern_site_resolutions),
        proof_predicates: Arc::new(proof_predicates),
        local_slots: Arc::new(local_slots),
        static_constant_bindings: Arc::new(stable_bindings),
        static_constant_reads: static_constant_reads.into(),
        dependencies: dependencies.into(),
        result_type: if matches!(expected, ExpectedType::Inference { .. }) && !typed_result.status.is_ready() {
            None
        } else {
            solved_expected_type.or_else(|| typed_result.knowledge.ty())
        },
        diagnostics,
    };
    (checked, context.semantic_query_demands_snapshot())
}

fn analyze_static_body_inner(
    mut context: CheckingContext<'_>,
    definition: &StaticDefinition,
    statically_readable_bindings: &BTreeSet<BindingId>,
    static_binding_symbols: &BTreeMap<BindingId, phalcom_modules::linker::SymbolId>,
    module_binding_fingerprint: Option<crate::db::key::ProductFingerprint>,
    intrinsics: &StaticIntrinsicRegistry,
) -> CheckedStaticBody {
    context.phase = crate::checker::context::EvaluationPhase::Static;
    context.current_static_body_owner = Some(StaticBodyId(definition.id.clone()));
    let diagnostic_start = context.diagnostics.len();
    let existing_expressions = context.expressions.keys().copied().collect::<BTreeSet<_>>();
    let expression_start = context.next_local_expr_id;
    let body_id = context.body_id;
    let first_body_binding = context.next_binding_id;
    let runtime_outer_bindings = context
        .scopes
        .iter()
        .flat_map(|scope| scope.values().map(|binding| binding.id))
        .collect::<BTreeSet<_>>();

    context.push_scope();
    let mut parameter_bindings = BTreeMap::new();
    let kinded_parameters = definition
        .signature
        .parameters
        .iter()
        .filter_map(|parameter| parameter.type_parameter_id.map(|id| (parameter.index, id)))
        .collect::<BTreeMap<_, _>>();
    let mut static_parameter_bindings = BTreeSet::new();
    for parameter in definition.signature.parameters.iter() {
        let StaticValueDomain::ValueType(elem_ty) = parameter.domain else {
            continue;
        };
        let bound_ty = match parameter.mode {
            crate::types::parameter::GenericParameterMode::Single => elem_ty,
            crate::types::parameter::GenericParameterMode::PositionalPack => {
                let remainder = crate::types::store::AnonymousTupleRemainder::Typed(elem_ty);
                context
                    .store
                    .open_pack(Box::new([]), remainder, crate::types::store::AnonymousTupleRemainder::None)
                    .unwrap_or(elem_ty)
            }
            crate::types::parameter::GenericParameterMode::LabeledPack => {
                let remainder = crate::types::store::AnonymousTupleRemainder::Typed(elem_ty);
                context
                    .store
                    .open_pack(Box::new([]), crate::types::store::AnonymousTupleRemainder::None, remainder)
                    .unwrap_or(elem_ty)
            }
            crate::types::parameter::GenericParameterMode::CompletePack => {
                let remainder = crate::types::store::AnonymousTupleRemainder::Typed(elem_ty);
                context.store.open_pack(Box::new([]), remainder, remainder).unwrap_or(elem_ty)
            }
        };
        let current = TypeKnowledge::assumed(bound_ty, EvidenceOrigin::CallableSignature);
        let inserted = context.declare_binding(BindingSeed {
            name: parameter.name.clone(),
            parameter: None,
            range: parameter.range,
            contract: Some(BindingContract {
                ty: bound_ty,
                origin: BindingContractOrigin::StaticParameter,
                source: Some(parameter.range),
            }),
            current,
            denotation: None,
            causal_invalidity: CausalInvalidity::Clean,
            mutable: false,
            initialization: InitializationState::DefinitelyInitialized,
        });
        if let crate::checker::binding::BindingDeclarationResult::Inserted(binding) = inserted {
            parameter_bindings.insert(parameter.index, binding);
            static_parameter_bindings.insert(binding);
        }
    }

    let expected = match definition.signature.result_domain {
        StaticValueDomain::Kind(_) => ExpectedType::None,
        StaticValueDomain::ValueType(ty) => ExpectedType::proper_from(ty, ExpectationOrigin::DeclarationContract),
    };
    context.expected_return = match definition.signature.result_domain {
        StaticValueDomain::Kind(_) => None,
        StaticValueDomain::ValueType(ty) => Some(crate::checker::context::CallableReturnContract {
            ty,
            basis: crate::declaration_type::DeclaredTypeBasis::DeclarationSemantics,
            origin: crate::types::evidence::EvidenceOrigin::DeclarationSemantics,
            is_dynamic: false,
            source: None,
        }),
    };
    context.expected_kinded_return = definition.signature.result_domain.as_kind();
    let return_exit_start = context.normal_return_exits.len();
    let typed_result = match &definition.body {
        StaticDefinitionBodyRepr::TypeForm(annotation) => {
            // The parser keeps a neutral type-form bridge for an ambiguous
            // bare identifier. In a value-result definition, resolve that
            // single name through the already-seeded ordinary checker scope;
            // only an actual checked binding makes it a value read.
            let value_binding = matches!(definition.signature.result_domain, StaticValueDomain::ValueType(_))
                .then(|| match &annotation.expr {
                    TypeAnnotationExpr::Reference(reference) if reference.members.is_empty() => {
                        context.lookup_binding_info(&reference.root).map(|_| reference.root.clone())
                    }
                    _ => None,
                })
                .flatten();
            if let Some(value) = value_binding {
                Some(analyze_expression(
                    &mut context,
                    &Expr::Var {
                        value,
                        range: annotation.range,
                    },
                    &expected,
                ))
            } else {
                let expr = Expr::TypeForm(Box::new(annotation.clone()));
                Some(analyze_expression(&mut context, &expr, &expected))
            }
        }
        StaticDefinitionBodyRepr::Expression(expression) => Some(analyze_expression(&mut context, expression, &expected)),
        StaticDefinitionBodyRepr::Block(block) => analyze_executable_region(&mut context, &block.body, block.range, &expected).value,
        StaticDefinitionBodyRepr::Operational(_) => None,
    };

    let result_range = definition.body.range();
    let mut typed_result = typed_result;
    if let Some(typed) = typed_result.as_mut() {
        match definition.signature.result_domain {
            StaticValueDomain::Kind(expected_kind) => {
                let actual_kind = match typed.denotation.as_ref() {
                    Some(crate::types::denotation::SemanticDenotation::TypeForm(form)) => match context.store.get(*form) {
                        TypeData::Lambda(lambda) => Some(context.store.arena().get_lambda(*lambda).result_kind),
                        _ => Some(context.store.kind_of(*form)),
                    },
                    Some(crate::types::denotation::SemanticDenotation::StaticApplication(application)) => context
                        .checked_static_applications
                        .iter()
                        .find(|candidate| candidate.id == *application)
                        .and_then(|candidate| candidate.signature.result_domain.as_kind()),
                    _ => None,
                };
                if actual_kind != Some(expected_kind) {
                    let actual = actual_kind.map_or_else(|| "a non-type value".to_string(), |kind| context.store.format_kind(kind));
                    let expected = context.store.format_kind(expected_kind);
                    invalidate_with_diagnostic(
                        &mut context,
                        typed,
                        DiagnosticCode::TypeMismatch,
                        format!("static definition body has kind {actual}, expected {expected}"),
                        result_range,
                    );
                }
            }
            StaticValueDomain::ValueType(expected_type) if typed.expression_id.is_none() => {
                let application = context.apply_knowledge_against_type(
                    &typed.knowledge,
                    expected_type,
                    DiagnosticCode::TypeMismatch,
                    "static definition body result does not match its declared value type",
                    result_range,
                );
                apply_relation_status(typed, application.status);
            }
            StaticValueDomain::ValueType(_) => {}
        }
        context.sync_expression_outcome(typed);
    } else if let StaticValueDomain::Kind(expected_kind) = definition.signature.result_domain
        && context.normal_return_exits.len() == return_exit_start
    {
        let diagnostic = SemanticDiagnostic::error_in(
            context.current_module.clone(),
            DiagnosticCode::TypeMismatch,
            format!(
                "static definition body has no type-form result of kind {}",
                context.store.format_kind(expected_kind)
            ),
            result_range,
        );
        context.emit_diagnostic(diagnostic);
    }

    let runtime_accesses = context
        .expressions
        .values()
        .filter(|expression| !existing_expressions.contains(&expression.id) && expression.id.owner == body_id && expression.id.local.0 >= expression_start)
        .filter_map(|expression| {
            let binding = expression.binding_read.or(expression.binding_write)?;
            (runtime_outer_bindings.contains(&binding) && !statically_readable_bindings.contains(&binding)).then_some((
                expression.id,
                expression.range,
                binding,
            ))
        })
        .collect::<Vec<(ExpressionId, SourceRange, BindingId)>>();
    for (expression_id, range, binding) in runtime_accesses {
        let message = format!("runtime binding is not accessible in a static definition body (binding {binding:?})");
        let diagnostic = SemanticDiagnostic::error_in(context.current_module.clone(), DiagnosticCode::TypeOnlyRuntimeValue, message, range);
        if let Some(cause) = context.emit_diagnostic(diagnostic)
            && let Some(expression) = context.expressions.get_mut(&expression_id)
        {
            expression.status = AnalysisStatus::Invalid(cause);
            expression.causal_invalidity = expression.causal_invalidity.join(CausalInvalidity::One(cause));
        }
    }
    context.pop_scope();

    context.ensure_sequence_publication_settled(definition.source_range);

    let expressions = context
        .expressions
        .iter()
        .filter(|(id, _)| !existing_expressions.contains(id) && id.owner == body_id && id.local.0 >= expression_start)
        .map(|(id, analysis)| (*id, analysis.clone()))
        .collect::<ExpressionAnalysisIndex>();
    let static_operations = intrinsics.resolve_ready_expressions(&expressions);
    let static_constant_reads = expressions
        .values()
        .filter_map(|expression| expression.binding_read.and_then(|binding| static_binding_symbols.get(&binding).cloned()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let static_applications = std::mem::take(&mut context.checked_static_applications).into_boxed_slice();
    let static_constructions = std::mem::take(&mut context.checked_static_constructions);
    let member_projections = checked_member_projection_index(&context, &expressions);
    let dependencies: Arc<[SemanticDependency]> = context.semantic_dependencies_snapshot().into_iter().collect::<Vec<_>>().into();
    let associated_resolutions = context
        .associated_resolutions
        .into_iter()
        .filter(|(expression, _)| expressions.contains_key(expression))
        .collect();
    let sequence_literals: crate::checker::analysis::SequenceLiteralResolutionIndex = context
        .sequence_literals
        .into_iter()
        .filter(|(expression, _)| expressions.contains_key(expression))
        .collect();
    let match_resolutions: crate::match_semantics::MatchResolutionIndex = context
        .match_resolutions
        .into_iter()
        .filter(|(expression, _)| expressions.contains_key(expression))
        .collect();
    let body_range = definition.body.range();
    let pattern_site_resolutions: crate::match_semantics::PatternSiteResolutionIndex = context
        .pattern_site_resolutions
        .into_iter()
        .filter(|(key, _)| body_range.contains_range(&key.range))
        .collect();
    let static_iteration_lanes: crate::checker::static_iteration::StaticIterationLaneResolutionIndex = context
        .static_iteration_lanes
        .into_iter()
        .filter(|(expression, _)| expressions.contains_key(expression))
        .collect();
    let proof_predicates: crate::checker::proof_predicate::ProofPredicateResolutionIndex = context
        .proof_predicates
        .into_iter()
        .filter(|(expression, _)| expressions.contains_key(expression))
        .collect();

    let mut bindings: BindingAnalysisIndex = context.binding_history.clone();
    bindings.extend(context.flow.bindings.clone());
    bindings.retain(|binding, _| binding.0 >= first_body_binding || statically_readable_bindings.contains(binding));
    let mut local_ids = bindings
        .keys()
        .copied()
        .filter(|binding| binding.0 >= first_body_binding && !static_parameter_bindings.contains(binding))
        .collect::<Vec<_>>();
    local_ids.sort_unstable();
    let local_slots = local_ids
        .into_iter()
        .enumerate()
        .map(|(slot, binding)| (binding, slot as u32))
        .collect::<BTreeMap<_, _>>();

    let diagnostics = Arc::from(context.diagnostics[diagnostic_start..].to_vec().into_boxed_slice());
    CheckedStaticBody {
        owner: StaticBodyId(definition.id.clone()),
        root: definition.body.clone(),
        expressions: Arc::new(expressions),
        bindings: Arc::new(bindings),
        static_operations: Arc::new(static_operations),
        static_applications: Arc::from(static_applications),
        static_constructions: Arc::new(static_constructions),
        member_projections: Arc::new(member_projections),
        associated_resolutions: Arc::new(associated_resolutions),
        sequence_literals: Arc::new(sequence_literals),
        match_resolutions: Arc::new(match_resolutions),
        pattern_site_resolutions: Arc::new(pattern_site_resolutions),
        static_iteration_lanes: Arc::new(static_iteration_lanes),
        proof_predicates: Arc::new(proof_predicates),
        parameter_bindings: Arc::new(parameter_bindings),
        kinded_parameters: Arc::new(kinded_parameters),
        local_slots: Arc::new(local_slots),
        static_constant_bindings: Arc::new(static_binding_symbols.clone()),
        static_constant_reads: static_constant_reads.into(),
        module_binding_fingerprint,
        input_fingerprint: crate::db::fingerprint::checked_static_body_source_input_fingerprint(definition, module_binding_fingerprint),
        dependencies,
        result: definition.signature.result_domain,
        diagnostics,
    }
}

/// Derives evaluator projection requests from already checked canonical type forms.
/// This only publishes sites supported by the structural reflection projector;
/// member identity and result domains come from the resolved form and signature.
fn checked_member_projection_index(
    context: &CheckingContext<'_>,
    expressions: &ExpressionAnalysisIndex,
) -> crate::static_definition::CheckedStaticMemberProjectionIndex {
    let structural_ids = crate::core_surface::identity::StructuralUniverseIds::new();
    let mut projections = BTreeMap::new();
    for (expression, analysis) in expressions {
        if !analysis.status.is_ready() {
            continue;
        }
        let Some(form) = analysis.denotation.as_ref().and_then(|denotation| match denotation {
            crate::types::denotation::SemanticDenotation::TypeForm(form) | crate::types::denotation::SemanticDenotation::StaticTerm(form) => Some(*form),
            _ => None,
        }) else {
            continue;
        };
        let projection = match context.store.get(form) {
            TypeData::AssociatedProjection(projection) if structural_ids.is_structural_capability(&projection.trait_ref.declaration) => {
                let Some(surface) = context
                    .conformance_semantics
                    .and_then(|view| view.trait_surfaces.get(&projection.trait_ref.declaration))
                else {
                    continue;
                };
                let Some(requirement) = surface.associated_type(&projection.requirement) else {
                    continue;
                };
                context.record_semantic_dependency(SemanticDependency::TraitSurface(projection.trait_ref.declaration.clone()));
                let conformance_key = crate::generic_evidence::GenericConformanceKey::new(
                    crate::types::parameter::TypeTerm::Canonical(projection.subject),
                    projection.trait_ref.clone(),
                );
                let evidence = context
                    .generic_evidence
                    .conformances()
                    .contains_key(&conformance_key)
                    .then_some(conformance_key);
                Some(MemberProjection {
                    subject: MemberProjectionSubject::Type(projection.subject),
                    member: MemberDeclarationIdentity::TraitAssociated {
                        trait_ref: projection.trait_ref.clone(),
                        requirement: projection.requirement.clone(),
                    },
                    arguments: Box::new([]),
                    evidence,
                    result_domain: requirement.domain.into(),
                })
            }
            TypeData::TypeMemberProjection(projection)
                if projection
                    .source_declaration
                    .as_ref()
                    .is_some_and(|member| member.owner == structural_ids.unordered_shape) =>
            {
                let Some(member) = projection.source_declaration.as_ref() else {
                    continue;
                };
                let Some(product) = context.resolver.resolve_associated_type_member(&member.owner, &member.selector) else {
                    continue;
                };
                context.record_semantic_dependency(SemanticDependency::AssociatedSurface(member.owner.clone()));
                let subject = match projection.subject {
                    ProjectionSubject::Type(ty) => MemberProjectionSubject::Type(ty),
                    ProjectionSubject::RecordRow(row) => MemberProjectionSubject::RecordRow(row),
                };
                Some(MemberProjection {
                    subject,
                    member: MemberDeclarationIdentity::TypeMember(member.clone()),
                    arguments: projection
                        .arguments
                        .iter()
                        .cloned()
                        .map(|argument| {
                            MemberProjectionArgument::Kinded(match argument {
                                TypeMemberArgument::Type(ty) => TypeMemberArgument::Type(ty),
                                TypeMemberArgument::Const(term) => TypeMemberArgument::Const(term),
                                TypeMemberArgument::RecordRow(row) => TypeMemberArgument::RecordRow(row),
                            })
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    evidence: None,
                    result_domain: product.signature.result_domain.into(),
                })
            }
            _ => None,
        };
        if let Some(projection) = projection {
            projections.insert(*expression, projection);
        }
    }
    projections
}

fn invalidate_with_diagnostic(context: &mut CheckingContext<'_>, typed: &mut TypedExpression, code: DiagnosticCode, message: String, range: SourceRange) {
    let diagnostic = SemanticDiagnostic::error_in(context.current_module.clone(), code, message, range);
    if let Some(cause) = context.emit_diagnostic(diagnostic) {
        typed.invalidate(cause);
    }
}

fn apply_relation_status(typed: &mut TypedExpression, status: Option<AnalysisStatus>) {
    match status {
        Some(AnalysisStatus::Invalid(cause)) => typed.invalidate(cause),
        Some(other) => typed.status = other,
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::context::CheckingContext;
    use crate::declarations::DeclarationTypeTable;
    use crate::dispatch::SurfaceDispatchResolver;
    use crate::identity::{DeclarationId, ModuleId};
    use crate::static_definition::{StaticDefinitionBodyRepr, StaticDefinitionSignature, StaticParameter};
    use crate::static_eval::{StaticArgument, StaticEvalError, StaticEvalOutcome, StaticEvaluator};
    use crate::static_value::StaticValueStore;
    use crate::types::annotation::TypeResolver;
    use crate::types::outcome::{CancellationToken, QueryBudget};
    use crate::types::relation::MapTypeHierarchy;
    use crate::types::store::TypeStore;
    use phalcom_ast::ast::Expr;
    use phalcom_common::range::SourceRange;

    struct EmptyResolver;
    impl TypeResolver for EmptyResolver {
        fn resolve_type_name(&self, _current_module: &ModuleId, _root: &str, _members: &[String]) -> Option<DeclarationId> {
            None
        }
    }

    #[test]
    fn checked_static_body_publishes_parameter_binding_and_local_read() {
        let module = ModuleId::universe_root();
        let mut store = TypeStore::new();
        let declarations = DeclarationTypeTable::new();
        let hierarchy = MapTypeHierarchy::new();
        let resolver = EmptyResolver;
        let dispatch = SurfaceDispatchResolver::new();
        let context = CheckingContext::new_with_dispatch_ref(&mut store, &hierarchy, &resolver, &declarations, &dispatch, module.clone());
        let unit = context.store.unit();
        let id = DeclarationId::new(module, "StaticUnit".into());
        let parameter = StaticParameter {
            name: "value".into(),
            domain: StaticValueDomain::ValueType(unit),
            mode: crate::types::parameter::GenericParameterMode::Single,
            index: 0,
            type_parameter_id: None,
            range: SourceRange::default(),
        };
        let definition = StaticDefinition {
            id,
            signature: StaticDefinitionSignature {
                parameters: vec![parameter].into_boxed_slice(),
                result_domain: StaticValueDomain::ValueType(unit),
                generic_signature: None,
                where_clause: None,
            },
            body: StaticDefinitionBodyRepr::Expression(Expr::Var {
                value: "value".into(),
                range: SourceRange { start: 1, end: 6 },
            }),
            source_range: SourceRange::default(),
        };
        let intrinsics = StaticIntrinsicRegistry::from_canonical_core().unwrap();
        let checked = analyze_static_body(context, &definition, &BTreeSet::new(), &intrinsics);
        assert_eq!(checked.parameter_bindings.len(), 1);
        let parameter_binding = checked.parameter_bindings[&0];
        assert!(checked.expressions.values().any(|analysis| analysis.binding_read == Some(parameter_binding)));
        assert!(checked.local_slots.is_empty());
        assert!(checked.diagnostics.is_empty());

        let mut values = StaticValueStore::new();
        let argument = values.intern_unit();
        let mut budget = QueryBudget::default();
        let cancellation = CancellationToken::new();
        let outcome = StaticEvaluator::new(&checked, &mut values, &mut budget, &cancellation, &[StaticArgument::Value(argument)]).evaluate();
        assert_eq!(outcome, StaticEvalOutcome::Reduced(argument));

        let wrong_count = StaticEvaluator::new(&checked, &mut StaticValueStore::new(), &mut QueryBudget::default(), &cancellation, &[]).evaluate();
        assert_eq!(
            wrong_count,
            StaticEvalOutcome::Invalid(StaticEvalError::InvalidArgumentCount { expected: 1, actual: 0 })
        );

        let wrong_kind = StaticEvaluator::new(
            &checked,
            &mut StaticValueStore::new(),
            &mut QueryBudget::default(),
            &cancellation,
            &[StaticArgument::Kinded(unit)],
        )
        .evaluate();
        assert_eq!(wrong_kind, StaticEvalOutcome::Invalid(StaticEvalError::InvalidArgumentKind { index: 0 }));

        let mut limited_values = StaticValueStore::new();
        let mut limited_budget = QueryBudget::new(0);
        let limited = StaticEvaluator::new(
            &checked,
            &mut limited_values,
            &mut limited_budget,
            &cancellation,
            &[StaticArgument::Value(argument)],
        )
        .evaluate();
        assert!(matches!(limited, StaticEvalOutcome::BudgetExceeded(_)));

        let mut cancelled_values = StaticValueStore::new();
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        let cancelled_outcome = StaticEvaluator::new(
            &checked,
            &mut cancelled_values,
            &mut QueryBudget::default(),
            &cancelled,
            &[StaticArgument::Value(argument)],
        )
        .evaluate();
        assert_eq!(cancelled_outcome, StaticEvalOutcome::Cancelled);
    }

    #[test]
    fn checked_body_projection_index_uses_canonical_structural_member_and_domain() {
        let module = ModuleId::universe_root();
        let structural_ids = crate::core_surface::identity::StructuralUniverseIds::new();
        let mut store = TypeStore::new();
        let owner = DeclarationId::new(module.clone(), "ProjectionFixture".into());
        let parameter = store.intern_type_parameter(crate::types::parameter::TypeParameterData::new(
            crate::types::parameter::TypeParameterOwner::Declaration(owner),
            0,
            "T",
            crate::types::id::KindId::TYPE,
        ));
        let subject = store.parameter_form(parameter);
        let product_shape = store.nominal(structural_ids.product_shape.clone());
        let requirement = crate::traits::AssociatedTypeRequirementId::new(structural_ids.product.clone(), 1);
        let projection = store.associated_projection(
            subject,
            crate::traits::TraitRef::new(structural_ids.product.clone(), Vec::new()),
            requirement.clone(),
        );

        let mut surface = crate::traits::TraitSurface::new(structural_ids.product.clone(), None);
        surface.associated_types.insert(
            requirement.clone(),
            crate::traits::TraitAssociatedTypeRequirement {
                requirement: requirement.clone(),
                name: "Shape".into(),
                domain: crate::types::type_member::TypeMemberDomain::ValueType(product_shape),
                source: crate::diagnostic::SemanticSourceSpan::new(module.clone(), SourceRange::default()),
            },
        );
        let mut trait_surfaces = crate::traits::TraitSurfaceTable::new();
        trait_surfaces.insert(Arc::new(surface));
        let trait_dispatch = crate::trait_dispatch::TraitDispatchIndex::default();
        let conformance_index = crate::impls::ConformanceIndex::new();
        let witness_plans = BTreeMap::new();
        let derived_conformance_rules = crate::derived_conformance::DerivedConformanceRuleIndex::new();
        let conformance_semantics = crate::trait_dispatch::ConformanceSemanticView {
            trait_dispatch: &trait_dispatch,
            conformance_index: &conformance_index,
            witness_plans: &witness_plans,
            trait_surfaces: &trait_surfaces,
            derived_conformance_rules: &derived_conformance_rules,
            callable_signatures: None,
        };
        let declarations = DeclarationTypeTable::new();
        let hierarchy = MapTypeHierarchy::new();
        let resolver = EmptyResolver;
        let dispatch = SurfaceDispatchResolver::new();
        let mut context = CheckingContext::new_with_dispatch_ref(&mut store, &hierarchy, &resolver, &declarations, &dispatch, module);
        context.attach_conformance_semantics(&conformance_semantics);

        let expression = crate::identity::ExpressionId::new(crate::identity::BodyId(7), crate::identity::LocalExpressionId(0));
        let analysis =
            crate::checker::analysis::ExpressionAnalysis::ready(expression, SourceRange::default(), TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence))
                .with_denotation(Some(crate::types::denotation::SemanticDenotation::TypeForm(projection)));
        let expressions = BTreeMap::from([(expression, analysis)]);

        let projections = checked_member_projection_index(&context, &expressions);
        let checked = projections.get(&expression).expect("canonical structural projection site");
        assert_eq!(checked.subject, MemberProjectionSubject::Type(subject));
        assert_eq!(
            checked.member,
            MemberDeclarationIdentity::TraitAssociated {
                trait_ref: crate::traits::TraitRef::new(structural_ids.product.clone(), Vec::new()),
                requirement,
            }
        );
        assert_eq!(checked.result_domain, StaticValueDomain::ValueType(product_shape));
        assert!(
            context
                .semantic_dependencies_snapshot()
                .contains(&SemanticDependency::TraitSurface(structural_ids.product))
        );
    }

    #[test]
    fn runtime_outer_binding_read_is_a_static_phase_error() {
        let module = ModuleId::universe_root();
        let mut store = TypeStore::new();
        let declarations = DeclarationTypeTable::new();
        let hierarchy = MapTypeHierarchy::new();
        let resolver = EmptyResolver;
        let dispatch = SurfaceDispatchResolver::new();
        let mut context = CheckingContext::new_with_dispatch_ref(&mut store, &hierarchy, &resolver, &declarations, &dispatch, module.clone());
        let unit = context.store.unit();
        let outer = context.declare_binding(BindingSeed {
            name: "runtime_value".into(),
            parameter: None,
            range: SourceRange::default(),
            contract: Some(BindingContract {
                ty: unit,
                origin: BindingContractOrigin::SourceAnnotation,
                source: Some(SourceRange::default()),
            }),
            current: TypeKnowledge::assumed(unit, EvidenceOrigin::DeveloperAnnotation),
            denotation: None,
            causal_invalidity: CausalInvalidity::Clean,
            mutable: false,
            initialization: InitializationState::DefinitelyInitialized,
        });
        let crate::checker::binding::BindingDeclarationResult::Inserted(outer) = outer else {
            panic!("fixture binding must be inserted");
        };
        let definition = StaticDefinition {
            id: DeclarationId::new(module, "RuntimeCapture".into()),
            signature: StaticDefinitionSignature {
                parameters: Box::new([]),
                result_domain: StaticValueDomain::ValueType(unit),
                generic_signature: None,
                where_clause: None,
            },
            body: StaticDefinitionBodyRepr::Expression(Expr::Var {
                value: "runtime_value".into(),
                range: SourceRange { start: 1, end: 14 },
            }),
            source_range: SourceRange::default(),
        };
        let intrinsics = StaticIntrinsicRegistry::from_canonical_core().unwrap();
        let checked = analyze_static_body(context, &definition, &BTreeSet::new(), &intrinsics);

        assert!(
            checked
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == DiagnosticCode::TypeOnlyRuntimeValue)
        );
        assert!(
            checked
                .expressions
                .values()
                .any(|analysis| { analysis.binding_read == Some(outer) && matches!(analysis.status, AnalysisStatus::Invalid(_)) })
        );
        assert!(!checked.bindings.contains_key(&outer), "outer runtime state is not owned by this body product");
    }

    #[test]
    fn runtime_outer_binding_write_is_a_static_phase_error() {
        let module = ModuleId::universe_root();
        let mut store = TypeStore::new();
        let declarations = DeclarationTypeTable::new();
        let hierarchy = MapTypeHierarchy::new();
        let resolver = EmptyResolver;
        let dispatch = SurfaceDispatchResolver::new();
        let mut context = CheckingContext::new_with_dispatch_ref(&mut store, &hierarchy, &resolver, &declarations, &dispatch, module.clone());
        let unit = context.store.unit();
        let outer = context.declare_binding(BindingSeed {
            name: "runtime_mutable".into(),
            parameter: None,
            range: SourceRange::default(),
            contract: Some(BindingContract {
                ty: unit,
                origin: BindingContractOrigin::SourceAnnotation,
                source: Some(SourceRange::default()),
            }),
            current: TypeKnowledge::assumed(unit, EvidenceOrigin::DeveloperAnnotation),
            denotation: None,
            causal_invalidity: CausalInvalidity::Clean,
            mutable: true,
            initialization: InitializationState::DefinitelyInitialized,
        });
        let crate::checker::binding::BindingDeclarationResult::Inserted(outer) = outer else {
            panic!("fixture binding must be inserted");
        };
        let range = SourceRange { start: 1, end: 16 };
        let definition = StaticDefinition {
            id: DeclarationId::new(module, "RuntimeWriteCapture".into()),
            signature: StaticDefinitionSignature {
                parameters: Box::new([]),
                result_domain: StaticValueDomain::ValueType(unit),
                generic_signature: None,
                where_clause: None,
            },
            body: StaticDefinitionBodyRepr::Expression(Expr::Assignment(Box::new(phalcom_ast::ast::AssignmentExpr {
                name: Box::new(Expr::Var {
                    value: "runtime_mutable".into(),
                    range,
                }),
                value: Expr::Boolean { value: true, range },
                range,
            }))),
            source_range: SourceRange::default(),
        };
        let intrinsics = StaticIntrinsicRegistry::from_canonical_core().unwrap();
        let checked = analyze_static_body(context, &definition, &BTreeSet::new(), &intrinsics);

        assert!(
            checked
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == DiagnosticCode::TypeOnlyRuntimeValue)
        );
        assert!(
            checked
                .expressions
                .values()
                .any(|analysis| analysis.binding_write == Some(outer) && matches!(analysis.status, AnalysisStatus::Invalid(_)))
        );
    }

    #[test]
    fn static_evaluator_runs_block_local_initializers_and_reads_by_binding_id() {
        let module = ModuleId::universe_root();
        let mut store = TypeStore::new();
        let declarations = DeclarationTypeTable::new();
        let hierarchy = MapTypeHierarchy::new();
        let resolver = EmptyResolver;
        let dispatch = SurfaceDispatchResolver::new();
        let context = CheckingContext::new_with_dispatch_ref(&mut store, &hierarchy, &resolver, &declarations, &dispatch, module.clone());
        let unit = context.store.unit();
        let definition = StaticDefinition {
            id: DeclarationId::new(module, "StaticLocal".into()),
            signature: StaticDefinitionSignature {
                parameters: vec![StaticParameter {
                    name: "input".into(),
                    domain: StaticValueDomain::ValueType(unit),
                    mode: crate::types::parameter::GenericParameterMode::Single,
                    index: 0,
                    type_parameter_id: None,
                    range: SourceRange { start: 11, end: 16 },
                }]
                .into_boxed_slice(),
                result_domain: StaticValueDomain::ValueType(unit),
                generic_signature: None,
                where_clause: None,
            },
            body: StaticDefinitionBodyRepr::Block(phalcom_ast::ast::LexicalBlock {
                body: vec![
                    phalcom_ast::ast::Statement::Let(phalcom_ast::ast::LetBinding {
                        kind: phalcom_ast::ast::BindingKind::Let,
                        pattern: phalcom_ast::ast::Pattern::Name {
                            name: "local".into(),
                            range: SourceRange { start: 4, end: 9 },
                        },
                        annotation: None,
                        value: Some(Expr::Var {
                            value: "input".into(),
                            range: SourceRange { start: 12, end: 17 },
                        }),
                        range: SourceRange { start: 0, end: 14 },
                    }),
                    phalcom_ast::ast::Statement::Expr {
                        expr: Expr::Var {
                            value: "local".into(),
                            range: SourceRange { start: 21, end: 26 },
                        },
                        range: SourceRange { start: 21, end: 27 },
                    },
                ],
                range: SourceRange { start: 0, end: 27 },
            }),
            source_range: SourceRange { start: 0, end: 27 },
        };
        let intrinsics = StaticIntrinsicRegistry::from_canonical_core().unwrap();
        let checked = analyze_static_body(context, &definition, &BTreeSet::new(), &intrinsics);
        assert!(checked.diagnostics.is_empty(), "{:?}", checked.diagnostics);
        assert_eq!(checked.local_slots.len(), 1);

        let mut values = StaticValueStore::new();
        let argument = values.intern_unit();
        let outcome = StaticEvaluator::new(
            &checked,
            &mut values,
            &mut QueryBudget::default(),
            &CancellationToken::new(),
            &[StaticArgument::Value(argument)],
        )
        .evaluate();
        let StaticEvalOutcome::Reduced(value) = outcome else {
            panic!("checked local block should reduce, got {outcome:?}");
        };
        assert_eq!(value, argument);
    }
}
