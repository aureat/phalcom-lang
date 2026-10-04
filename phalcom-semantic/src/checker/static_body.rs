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
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::identity::{BindingId, ExpressionId};
use crate::static_definition::{CheckedStaticBody, StaticBodyId, StaticDefinition, StaticDefinitionBodyRepr};
use crate::static_domain::StaticValueDomain;
use crate::static_intrinsic::StaticIntrinsicRegistry;
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge};
use crate::types::store::TypeData;
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
        let inserted = context.declare_checked_binding(visible_name.to_owned(), &fact.state);
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
    mut context: CheckingContext<'_>,
    constant_id: &crate::static_constant::StaticConstantId,
    declaration_range: SourceRange,
    initializer_range: SourceRange,
    initializer: &Expr,
    expected_type: Option<crate::types::id::TypeId>,
    module_bindings: &crate::module_bindings::ModuleStaticBindingIndex,
    intrinsics: &StaticIntrinsicRegistry,
) -> crate::static_constant::CheckedStaticConstantInitializer {
    let diagnostic_start = context.diagnostics.len();
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
    for (visible_name, fact) in module_bindings.visible_bindings() {
        let inserted = context.declare_checked_binding(visible_name.to_owned(), &fact.state);
        if let crate::checker::binding::BindingDeclarationResult::Inserted(binding) = inserted {
            if fact.phase == crate::module_bindings::ModuleBindingPhase::StaticConstant {
                readable.insert(binding);
                stable_bindings.insert(binding, fact.symbol.clone());
            } else {
                runtime_module_bindings.insert(binding);
            }
        }
    }
    let first_body_binding = context.next_binding_id;

    let expected = match expected_type {
        Some(ty) => ExpectedType::proper_from(ty, ExpectationOrigin::DeclarationContract),
        None => ExpectedType::None,
    };
    let typed_result = analyze_expression(&mut context, initializer, &expected);
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
    for read_sym in &static_constant_reads {
        dependencies.push(SemanticDependency::StaticConstant(crate::static_constant::StaticConstantId(read_sym.clone())));
    }
    dependencies.sort();
    dependencies.dedup();

    let diagnostics = Arc::from(context.diagnostics[diagnostic_start..].to_vec().into_boxed_slice());
    let input_fingerprint = crate::db::fingerprint::static_constant_input_fingerprint(
        constant_id,
        initializer,
        expected_type,
    );

    crate::static_constant::CheckedStaticConstantInitializer {
        id: constant_id.clone(),
        declaration_range,
        initializer_range,
        root: initializer.clone(),
        expressions: Arc::new(expressions),
        bindings: Arc::new(bindings),
        static_operations: Arc::new(static_operations),
        static_applications: Arc::from(static_applications),
        static_constructions: Arc::new(static_constructions),
        associated_resolutions: Arc::new(associated_resolutions),
        local_slots: Arc::new(local_slots),
        static_constant_bindings: Arc::new(stable_bindings),
        static_constant_reads: static_constant_reads.into(),
        input_fingerprint,
        dependencies: dependencies.into(),
        result_type: expected_type,
        diagnostics,
    }
}

fn analyze_static_body_inner(
    mut context: CheckingContext<'_>,
    definition: &StaticDefinition,
    statically_readable_bindings: &BTreeSet<BindingId>,
    static_binding_symbols: &BTreeMap<BindingId, phalcom_modules::linker::SymbolId>,
    module_binding_fingerprint: Option<crate::db::key::ProductFingerprint>,
    intrinsics: &StaticIntrinsicRegistry,
) -> CheckedStaticBody {
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
        let StaticValueDomain::ValueType(ty) = parameter.domain else {
            continue;
        };
        let current = TypeKnowledge::assumed(ty, EvidenceOrigin::CallableSignature);
        let inserted = context.declare_binding(BindingSeed {
            name: parameter.name.clone(),
            parameter: None,
            range: parameter.range,
            contract: Some(BindingContract {
                ty,
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
    } else if let StaticValueDomain::Kind(expected_kind) = definition.signature.result_domain {
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
    let dependencies: Arc<[SemanticDependency]> = context.semantic_dependencies_snapshot().into_iter().collect::<Vec<_>>().into();
    let static_constructions = std::mem::take(&mut context.checked_static_constructions);
    let associated_resolutions = context
        .associated_resolutions
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
        associated_resolutions: Arc::new(associated_resolutions),
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
