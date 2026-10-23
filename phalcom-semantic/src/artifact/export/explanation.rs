use super::*;
use crate::checker::expected::ExpectationOrigin;
use crate::dispatch::CallableSemanticKind;
use crate::explain::node::*;
use crate::types::outcome::{RelationFailure, RelationOutcome};
use phalcom_compiler_artifact::semantic::aggregates::DynamicReasonV1;
use phalcom_compiler_artifact::semantic::world::*;

struct ExplanationExport<'borrow, 'identity> {
    ordinals: &'borrow CallableAnalysisOrdinalMap,
    types: &'borrow TypeExportContext<'identity>,
}
impl ExplanationExport<'_, '_> {
    fn derivationrule(&mut self, value: &DerivationRule) -> Result<CallableDerivationRuleV1, ArtifactError> {
        Ok(match value {
            DerivationRule::LiteralSynthesis => CallableDerivationRuleV1::LiteralSynthesis,
            DerivationRule::ExpressionSynthesis => CallableDerivationRuleV1::ExpressionSynthesis,
            DerivationRule::NameResolution => CallableDerivationRuleV1::NameResolution,
            DerivationRule::AnnotationConstraint => CallableDerivationRuleV1::AnnotationConstraint,
            DerivationRule::BindingContract => CallableDerivationRuleV1::BindingContract,
            DerivationRule::TypeRequirement => CallableDerivationRuleV1::TypeRequirement,
            DerivationRule::TypeRelation => CallableDerivationRuleV1::TypeRelation,
            DerivationRule::CallableSelection => CallableDerivationRuleV1::CallableSelection,
            DerivationRule::UnionArm => CallableDerivationRuleV1::UnionArm,
            DerivationRule::CallableReturn => CallableDerivationRuleV1::CallableReturn,
            DerivationRule::SelfSpecialization => CallableDerivationRuleV1::SelfSpecialization,
            DerivationRule::ArgumentChecking => CallableDerivationRuleV1::ArgumentChecking,
            DerivationRule::CallShape => CallableDerivationRuleV1::CallShape,
            DerivationRule::GenericConstraint => CallableDerivationRuleV1::GenericConstraint,
            DerivationRule::GenericSolution => CallableDerivationRuleV1::GenericSolution,
            DerivationRule::GenericConflict => CallableDerivationRuleV1::GenericConflict,
            DerivationRule::CollectionSynthesis => CallableDerivationRuleV1::CollectionSynthesis,
            DerivationRule::ProductDecomposition => CallableDerivationRuleV1::ProductDecomposition,
            DerivationRule::FlowRefinement { predicate_kind } => CallableDerivationRuleV1::FlowRefinement {
                predicate_kind: self.predicatekind(predicate_kind)?,
            },
            DerivationRule::BranchJoin { branch_count } => CallableDerivationRuleV1::BranchJoin {
                branch_count: u64::try_from(*branch_count).map_err(|_| referential("explanation index overflow"))?,
            },
            DerivationRule::LoopJoin => CallableDerivationRuleV1::LoopJoin,
            DerivationRule::IterationElementResolution => CallableDerivationRuleV1::IterationElementResolution,
            DerivationRule::AssignmentPropagation => CallableDerivationRuleV1::AssignmentPropagation,
            DerivationRule::ReturnTypeCheck => CallableDerivationRuleV1::ReturnTypeCheck,
            DerivationRule::CallableReturnSummary => CallableDerivationRuleV1::CallableReturnSummary,
            DerivationRule::UnknownPropagation => CallableDerivationRuleV1::UnknownPropagation,
            DerivationRule::DynamicPropagation => CallableDerivationRuleV1::DynamicPropagation,
            DerivationRule::InternalBlocked => CallableDerivationRuleV1::InternalBlocked,
            DerivationRule::MethodCallReturn { selector } => CallableDerivationRuleV1::MethodCallReturn { selector: selector.clone() },
            DerivationRule::GenericInstantiation { type_args } => CallableDerivationRuleV1::GenericInstantiation {
                type_args: type_args
                    .iter()
                    .map(|value| Ok::<_, ArtifactError>(self.types.ty(*value)?))
                    .collect::<Result<Vec<_>, _>>()?,
            },
            DerivationRule::PolicyEnforcement { code } => CallableDerivationRuleV1::PolicyEnforcement { code: code.as_str().into() },
        })
    }
    fn evidenceref(&mut self, value: &EvidenceRef) -> Result<CallableExplanationEvidenceV1, ArtifactError> {
        Ok(match value {
            EvidenceRef::SourceSpan(value) => CallableExplanationEvidenceV1::SourceSpan {
                value: export_source_range(*value)?,
            },
            EvidenceRef::TypeId(value) => CallableExplanationEvidenceV1::TypeId { value: self.types.ty(*value)? },
            EvidenceRef::BindingVersion { binding, version } => CallableExplanationEvidenceV1::BindingVersion {
                binding: *self
                    .ordinals
                    .bindings
                    .get(binding)
                    .ok_or_else(|| referential("explanation local reference has no owner"))?,
                version: *version,
            },
            EvidenceRef::Suppressed { cause } => CallableExplanationEvidenceV1::Suppressed {
                cause: *self
                    .ordinals
                    .diagnostic_causes
                    .get(cause)
                    .ok_or_else(|| referential("explanation local reference has no owner"))?,
            },
            EvidenceRef::CallResolution(_) => return Err(referential("call-resolution explanation evidence has no canonical artifact owner")),
        })
    }
    fn predicatekind(&mut self, value: &PredicateKind) -> Result<CallableExplainPredicateKindV1, ArtifactError> {
        Ok(match value {
            PredicateKind::IsInstance => CallableExplainPredicateKindV1::IsInstance,
            PredicateKind::IsNotInstance => CallableExplainPredicateKindV1::IsNotInstance,
            PredicateKind::IsNil => CallableExplainPredicateKindV1::IsNil,
            PredicateKind::NotNil => CallableExplainPredicateKindV1::NotNil,
            PredicateKind::EqualLiteral => CallableExplainPredicateKindV1::EqualLiteral,
            PredicateKind::NotEqualLiteral => CallableExplainPredicateKindV1::NotEqualLiteral,
            PredicateKind::Ordered => CallableExplainPredicateKindV1::Ordered,
            PredicateKind::Truthy => CallableExplainPredicateKindV1::Truthy,
            PredicateKind::Falsy => CallableExplainPredicateKindV1::Falsy,
        })
    }
    fn callshapeexplanation(&mut self, value: &CallShapeExplanation) -> Result<CallableExplainCallShapeExplanationV1, ArtifactError> {
        Ok(match value {
            CallShapeExplanation::MissingRequired { parameter_index, label } => CallableExplainCallShapeExplanationV1::MissingRequired {
                parameter_index: *parameter_index,
                label: label.clone(),
            },
            CallShapeExplanation::UnexpectedPositional { argument_index } => CallableExplainCallShapeExplanationV1::UnexpectedPositional {
                argument_index: *argument_index,
            },
            CallShapeExplanation::UnknownLabel { label } => CallableExplainCallShapeExplanationV1::UnknownLabel { label: label.clone() },
            CallShapeExplanation::DuplicateLabel { label } => CallableExplainCallShapeExplanationV1::DuplicateLabel { label: label.clone() },
            CallShapeExplanation::DuplicateParameter { parameter_index } => CallableExplainCallShapeExplanationV1::DuplicateParameter {
                parameter_index: *parameter_index,
            },
        })
    }
    fn collectionkind(&mut self, value: &CollectionKind) -> Result<CallableExplainCollectionKindV1, ArtifactError> {
        Ok(match value {
            CollectionKind::List => CallableExplainCollectionKindV1::List,
            CollectionKind::Array => CallableExplainCollectionKindV1::Array,
            CollectionKind::Vector => CallableExplainCollectionKindV1::Vector,
            CollectionKind::Set => CallableExplainCollectionKindV1::Set,
            CollectionKind::Map => CallableExplainCollectionKindV1::Map,
            CollectionKind::Tuple => CallableExplainCollectionKindV1::Tuple,
            CollectionKind::Record => CallableExplainCollectionKindV1::Record,
        })
    }
    fn genericconstraintorigin(&mut self, value: &GenericConstraintOrigin) -> Result<CallableExplainGenericConstraintOriginV1, ArtifactError> {
        Ok(match value {
            GenericConstraintOrigin::Argument { parameter_index } => CallableExplainGenericConstraintOriginV1::Argument {
                parameter_index: *parameter_index,
            },
            GenericConstraintOrigin::ExpectedResult => CallableExplainGenericConstraintOriginV1::ExpectedResult,
            GenericConstraintOrigin::Receiver => CallableExplainGenericConstraintOriginV1::Receiver,
            GenericConstraintOrigin::ExplicitTypeArgument => CallableExplainGenericConstraintOriginV1::ExplicitTypeArgument,
            GenericConstraintOrigin::WhereClause => CallableExplainGenericConstraintOriginV1::WhereClause,
        })
    }
    fn genericconstraintrelation(&mut self, value: &GenericConstraintRelation) -> Result<CallableExplainGenericConstraintRelationV1, ArtifactError> {
        Ok(match value {
            GenericConstraintRelation::Equal(value) => CallableExplainGenericConstraintRelationV1::Equal { value: self.types.ty(*value)? },
            GenericConstraintRelation::SubtypeOf(value) => CallableExplainGenericConstraintRelationV1::SubtypeOf { value: self.types.ty(*value)? },
            GenericConstraintRelation::SupertypeOf(value) => CallableExplainGenericConstraintRelationV1::SupertypeOf { value: self.types.ty(*value)? },
            GenericConstraintRelation::AssignableTo(value) => CallableExplainGenericConstraintRelationV1::AssignableTo { value: self.types.ty(*value)? },
        })
    }
    fn unionarmoutcome(&mut self, value: &UnionArmOutcome) -> Result<CallableExplainUnionArmOutcomeV1, ArtifactError> {
        Ok(match value {
            UnionArmOutcome::Resolved => CallableExplainUnionArmOutcomeV1::Resolved,
            UnionArmOutcome::Missing { visited_owners } => CallableExplainUnionArmOutcomeV1::Missing {
                visited_owners: visited_owners
                    .iter()
                    .map(|value| Ok::<_, ArtifactError>(to_stable_declaration_with_context(value, self.types.identity_context())))
                    .collect::<Result<Vec<_>, _>>()?,
            },
            UnionArmOutcome::Ambiguous => CallableExplainUnionArmOutcomeV1::Ambiguous,
            UnionArmOutcome::Dynamic { reason } => CallableExplainUnionArmOutcomeV1::Dynamic {
                reason: self.dynamicreason(reason)?,
            },
            UnionArmOutcome::Invalid => CallableExplainUnionArmOutcomeV1::Invalid,
            UnionArmOutcome::ContextConflict => CallableExplainUnionArmOutcomeV1::ContextConflict,
        })
    }
    fn explanationstep(&mut self, value: &ExplanationStep) -> Result<CallableExplanationStepV1, ArtifactError> {
        Ok(match value {
            ExplanationStep::Literal { expression, ty } => CallableExplanationStepV1::Literal {
                expression: *self
                    .ordinals
                    .expressions
                    .get(expression)
                    .ok_or_else(|| referential("explanation local reference has no owner"))?,
                ty: self.types.ty(*ty)?,
            },
            ExplanationStep::ExpressionResult { expression, knowledge } => CallableExplanationStepV1::ExpressionResult {
                expression: *self
                    .ordinals
                    .expressions
                    .get(expression)
                    .ok_or_else(|| referential("explanation local reference has no owner"))?,
                knowledge: export_type_knowledge(knowledge, self.types)?,
            },
            ExplanationStep::BindingRead {
                expression,
                binding,
                knowledge,
            } => CallableExplanationStepV1::BindingRead {
                expression: *self
                    .ordinals
                    .expressions
                    .get(expression)
                    .ok_or_else(|| referential("explanation local reference has no owner"))?,
                binding: *self
                    .ordinals
                    .bindings
                    .get(binding)
                    .ok_or_else(|| referential("explanation local reference has no owner"))?,
                knowledge: export_type_knowledge(knowledge, self.types)?,
            },
            ExplanationStep::Declared { binding, range, ty } => CallableExplanationStepV1::Declared {
                binding: binding
                    .as_ref()
                    .map(|value| {
                        Ok::<_, ArtifactError>(
                            *self
                                .ordinals
                                .bindings
                                .get(value)
                                .ok_or_else(|| referential("explanation local reference has no owner"))?,
                        )
                    })
                    .transpose()?,
                range: export_source_range(*range)?,
                ty: self.types.ty(*ty)?,
            },
            ExplanationStep::BindingContract {
                binding,
                actual,
                contract,
                consistency,
            } => CallableExplanationStepV1::BindingContract {
                binding: *self
                    .ordinals
                    .bindings
                    .get(binding)
                    .ok_or_else(|| referential("explanation local reference has no owner"))?,
                actual: export_type_knowledge(actual, self.types)?,
                contract: self.types.ty(*contract)?,
                consistency: export_binding_consistency(consistency, self.types)?,
            },
            ExplanationStep::TypeRequirement { expected, origin, source } => CallableExplanationStepV1::TypeRequirement {
                expected: self.types.ty(*expected)?,
                origin: self.expectationorigin(origin)?,
                source: source
                    .as_ref()
                    .map(|value| Ok::<_, ArtifactError>(export_source_span(value, self.types.identity_context())?))
                    .transpose()?,
            },
            ExplanationStep::TypeRelation { actual, expected, outcome } => CallableExplanationStepV1::TypeRelation {
                actual: export_type_knowledge(actual, self.types)?,
                expected: self.types.ty(*expected)?,
                outcome: self.relationoutcome(outcome)?,
            },
            ExplanationStep::CallableSelection {
                callable,
                receiver,
                declaring_owner,
                specialization_path,
            } => CallableExplanationStepV1::CallableSelection {
                callable: to_stable_callable_with_context(callable, self.types.identity_context()),
                receiver: self.types.ty(*receiver)?,
                declaring_owner: to_stable_declaration_with_context(declaring_owner, self.types.identity_context()),
                specialization_path: specialization_path
                    .iter()
                    .map(|value| Ok::<_, ArtifactError>(to_stable_declaration_with_context(value, self.types.identity_context())))
                    .collect::<Result<Vec<_>, _>>()?,
            },
            ExplanationStep::UnionArm { receiver, callable, outcome } => CallableExplanationStepV1::UnionArm {
                receiver: self.types.ty(*receiver)?,
                callable: callable
                    .as_ref()
                    .map(|value| Ok::<_, ArtifactError>(to_stable_callable_with_context(value, self.types.identity_context())))
                    .transpose()?,
                outcome: self.unionarmoutcome(outcome)?,
            },
            ExplanationStep::CallableKind { callable, kind } => CallableExplanationStepV1::CallableKind {
                callable: to_stable_callable_with_context(callable, self.types.identity_context()),
                kind: self.callablesemantickind(kind)?,
            },
            ExplanationStep::CallableReturn { callable, ty } => CallableExplanationStepV1::CallableReturn {
                callable: to_stable_callable_with_context(callable, self.types.identity_context()),
                ty: self.types.ty(*ty)?,
            },
            ExplanationStep::SelfTypeSpecialization { self_ty, receiver, resolved } => CallableExplanationStepV1::SelfTypeSpecialization {
                self_ty: self.types.ty(*self_ty)?,
                receiver: self.types.ty(*receiver)?,
                resolved: self.types.ty(*resolved)?,
            },
            ExplanationStep::ArgumentCheck {
                call,
                argument,
                parameter_index,
                actual,
                expected,
            } => CallableExplanationStepV1::ArgumentCheck {
                call: *self
                    .ordinals
                    .expressions
                    .get(call)
                    .ok_or_else(|| referential("explanation local reference has no owner"))?,
                argument: *self
                    .ordinals
                    .expressions
                    .get(argument)
                    .ok_or_else(|| referential("explanation local reference has no owner"))?,
                parameter_index: *parameter_index,
                actual: export_type_knowledge(actual, self.types)?,
                expected: self.types.ty(*expected)?,
            },
            ExplanationStep::CallShape { callable, failures } => CallableExplanationStepV1::CallShape {
                callable: callable
                    .as_ref()
                    .map(|value| Ok::<_, ArtifactError>(to_stable_callable_with_context(value, self.types.identity_context())))
                    .transpose()?,
                failures: failures
                    .iter()
                    .map(|value| Ok::<_, ArtifactError>(self.callshapeexplanation(value)?))
                    .collect::<Result<Vec<_>, _>>()?,
            },
            ExplanationStep::MethodCall { call, callable, return_ty } => CallableExplanationStepV1::MethodCall {
                call: *self
                    .ordinals
                    .expressions
                    .get(call)
                    .ok_or_else(|| referential("explanation local reference has no owner"))?,
                callable: to_stable_callable_with_context(callable, self.types.identity_context()),
                return_ty: self.types.ty(*return_ty)?,
            },
            ExplanationStep::UnresolvedCall { call, return_ty } => CallableExplanationStepV1::UnresolvedCall {
                call: *self
                    .ordinals
                    .expressions
                    .get(call)
                    .ok_or_else(|| referential("explanation local reference has no owner"))?,
                return_ty: self.types.ty(*return_ty)?,
            },
            ExplanationStep::GenericConstraint { parameter, origin, relation } => CallableExplanationStepV1::GenericConstraint {
                parameter: self.types.parameter(*parameter)?,
                origin: self.genericconstraintorigin(origin)?,
                relation: self.genericconstraintrelation(relation)?,
            },
            ExplanationStep::GenericSolution { parameter, ty, status } => CallableExplanationStepV1::GenericSolution {
                parameter: self.types.parameter(*parameter)?,
                ty: self.types.ty(*ty)?,
                status: self.evidencestatus(status)?,
            },
            ExplanationStep::GenericConflict { parameter, constraints } => CallableExplanationStepV1::GenericConflict {
                parameter: parameter
                    .as_ref()
                    .map(|value| Ok::<_, ArtifactError>(self.types.parameter(*value)?))
                    .transpose()?,
                constraints: constraints
                    .iter()
                    .map(|value| {
                        Ok::<_, ArtifactError>(
                            *self
                                .ordinals
                                .explanations
                                .get(value)
                                .ok_or_else(|| referential("explanation local reference has no owner"))?,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            },
            ExplanationStep::CollectionSynthesis {
                expression,
                kind,
                element_types,
                result,
            } => CallableExplanationStepV1::CollectionSynthesis {
                expression: *self
                    .ordinals
                    .expressions
                    .get(expression)
                    .ok_or_else(|| referential("explanation local reference has no owner"))?,
                kind: self.collectionkind(kind)?,
                element_types: element_types
                    .iter()
                    .map(|value| Ok::<_, ArtifactError>(self.types.ty(*value)?))
                    .collect::<Result<Vec<_>, _>>()?,
                result: self.types.ty(*result)?,
            },
            ExplanationStep::ProductComponent { source, index, result } => CallableExplanationStepV1::ProductComponent {
                source: *self
                    .ordinals
                    .explanations
                    .get(source)
                    .ok_or_else(|| referential("explanation local reference has no owner"))?,
                index: u64::try_from(*index).map_err(|_| referential("explanation index overflow"))?,
                result: export_type_knowledge(result, self.types)?,
            },
            ExplanationStep::FlowRefinement {
                binding,
                predicate,
                prior,
                refined,
            } => CallableExplanationStepV1::FlowRefinement {
                binding: *self
                    .ordinals
                    .bindings
                    .get(binding)
                    .ok_or_else(|| referential("explanation local reference has no owner"))?,
                predicate: self.predicatekind(predicate)?,
                prior: export_type_knowledge(prior, self.types)?,
                refined: export_type_knowledge(refined, self.types)?,
            },
            ExplanationStep::BranchJoin {
                binding,
                branches,
                reachable,
                joined,
            } => CallableExplanationStepV1::BranchJoin {
                binding: binding
                    .as_ref()
                    .map(|value| {
                        Ok::<_, ArtifactError>(
                            *self
                                .ordinals
                                .bindings
                                .get(value)
                                .ok_or_else(|| referential("explanation local reference has no owner"))?,
                        )
                    })
                    .transpose()?,
                branches: branches
                    .iter()
                    .map(|value| Ok::<_, ArtifactError>(export_type_knowledge(value, self.types)?))
                    .collect::<Result<Vec<_>, _>>()?,
                reachable: reachable.iter().map(|value| Ok::<_, ArtifactError>(*value)).collect::<Result<Vec<_>, _>>()?,
                joined: export_type_knowledge(joined, self.types)?,
            },
            ExplanationStep::LoopJoin { inputs, joined } => CallableExplanationStepV1::LoopJoin {
                inputs: inputs
                    .iter()
                    .map(|value| Ok::<_, ArtifactError>(export_type_knowledge(value, self.types)?))
                    .collect::<Result<Vec<_>, _>>()?,
                joined: export_type_knowledge(joined, self.types)?,
            },
            ExplanationStep::IterationElement { iterable, element, callable } => CallableExplanationStepV1::IterationElement {
                iterable: export_type_knowledge(iterable, self.types)?,
                element: export_type_knowledge(element, self.types)?,
                callable: callable
                    .as_ref()
                    .map(|value| Ok::<_, ArtifactError>(to_stable_callable_with_context(value, self.types.identity_context())))
                    .transpose()?,
            },
            ExplanationStep::ReturnCheck { actual, expected } => CallableExplanationStepV1::ReturnCheck {
                actual: export_type_knowledge(actual, self.types)?,
                expected: expected.as_ref().map(|value| Ok::<_, ArtifactError>(self.types.ty(*value)?)).transpose()?,
            },
            ExplanationStep::CallableReturnSummary { callable, returns, result } => CallableExplanationStepV1::CallableReturnSummary {
                callable: to_stable_callable_with_context(callable, self.types.identity_context()),
                returns: returns
                    .iter()
                    .map(|value| Ok::<_, ArtifactError>(export_type_knowledge(value, self.types)?))
                    .collect::<Result<Vec<_>, _>>()?,
                result: export_type_knowledge(result, self.types)?,
            },
            ExplanationStep::UnknownBoundary { reason, source } => CallableExplanationStepV1::UnknownBoundary {
                reason: export_unknown_reason(reason),
                source: source.as_ref().map(|value| Ok::<_, ArtifactError>(export_source_range(*value)?)).transpose()?,
            },
            ExplanationStep::DynamicBoundary { reason, source } => CallableExplanationStepV1::DynamicBoundary {
                reason: self.dynamicreason(reason)?,
                source: source.as_ref().map(|value| Ok::<_, ArtifactError>(export_source_range(*value)?)).transpose()?,
            },
            ExplanationStep::InternalBlocked { reason } => CallableExplanationStepV1::InternalBlocked {
                reason: export_callable_block_reason(reason, self.types)?,
            },
            ExplanationStep::Subtyping { actual, expected, proven } => CallableExplanationStepV1::Subtyping {
                actual: self.types.ty(*actual)?,
                expected: self.types.ty(*expected)?,
                proven: *proven,
            },
        })
    }
    fn expectationorigin(&mut self, value: &ExpectationOrigin) -> Result<CallableExplainExpectationOriginV1, ArtifactError> {
        Ok(match value {
            ExpectationOrigin::DeclarationContract => CallableExplainExpectationOriginV1::DeclarationContract,
            ExpectationOrigin::ReturnContract => CallableExplainExpectationOriginV1::ReturnContract,
            ExpectationOrigin::CallableSignature => CallableExplainExpectationOriginV1::CallableSignature,
            ExpectationOrigin::AssignmentContract => CallableExplainExpectationOriginV1::AssignmentContract,
            ExpectationOrigin::ContextualBlockParameter => CallableExplainExpectationOriginV1::ContextualBlockParameter,
            ExpectationOrigin::GenericArgument => CallableExplainExpectationOriginV1::GenericArgument,
            ExpectationOrigin::GenericResult => CallableExplainExpectationOriginV1::GenericResult,
            ExpectationOrigin::CollectionElement => CallableExplainExpectationOriginV1::CollectionElement,
            ExpectationOrigin::ProductComponent => CallableExplainExpectationOriginV1::ProductComponent,
            ExpectationOrigin::ExplicitCheck => CallableExplainExpectationOriginV1::ExplicitCheck,
            ExpectationOrigin::AlgebraicArithmetic => CallableExplainExpectationOriginV1::AlgebraicArithmetic,
        })
    }
    fn callablesemantickind(&mut self, value: &CallableSemanticKind) -> Result<CallableExplainCallableSemanticKindV1, ArtifactError> {
        Ok(match value {
            CallableSemanticKind::Ordinary => CallableExplainCallableSemanticKindV1::Ordinary,
            CallableSemanticKind::Constructor => CallableExplainCallableSemanticKindV1::Constructor,
            CallableSemanticKind::Native => CallableExplainCallableSemanticKindV1::Native,
        })
    }
    fn evidencestatus(&mut self, value: &EvidenceStatus) -> Result<EvidenceStatusV1, ArtifactError> {
        Ok(match value {
            EvidenceStatus::Established => EvidenceStatusV1::Established,
            EvidenceStatus::Assumed => EvidenceStatusV1::Assumed,
        })
    }
    fn evidenceorigin(&mut self, value: &EvidenceOrigin) -> Result<EvidenceOriginV1, ArtifactError> {
        Ok(match value {
            EvidenceOrigin::Syntax => EvidenceOriginV1::Syntax,
            EvidenceOrigin::DeclarationSemantics => EvidenceOriginV1::DeclarationSemantics,
            EvidenceOrigin::ConstructorSemantics => EvidenceOriginV1::ConstructorSemantics,
            EvidenceOrigin::CallableSignature => EvidenceOriginV1::CallableSignature,
            EvidenceOrigin::NativeSignature => EvidenceOriginV1::NativeSignature,
            EvidenceOrigin::DeveloperAnnotation => EvidenceOriginV1::DeveloperAnnotation,
            EvidenceOrigin::GenericInference => EvidenceOriginV1::GenericInference,
            EvidenceOrigin::Flow => EvidenceOriginV1::Flow,
            EvidenceOrigin::FieldLifecycle => EvidenceOriginV1::FieldLifecycle,
            EvidenceOrigin::ContextualDerivation => EvidenceOriginV1::ContextualDerivation,
            EvidenceOrigin::PatternDecomposition => EvidenceOriginV1::PatternDecomposition,
        })
    }
    fn dynamicreason(&mut self, value: &DynamicReason) -> Result<DynamicReasonV1, ArtifactError> {
        Ok(match value {
            DynamicReason::ExplicitEscape => DynamicReasonV1::ExplicitEscape,
            DynamicReason::DynamicRestPack => DynamicReasonV1::DynamicRestPack,
            DynamicReason::RuntimeReflection => DynamicReasonV1::RuntimeReflection,
        })
    }
    fn relationfailure(&mut self, value: &RelationFailure) -> Result<CallableExplainRelationFailureV1, ArtifactError> {
        Ok(match value {
            RelationFailure::TypeMismatch { actual, expected } => CallableExplainRelationFailureV1::TypeMismatch {
                actual: self.types.ty(*actual)?,
                expected: self.types.ty(*expected)?,
            },
            RelationFailure::IncompatibleNominal { actual, expected } => CallableExplainRelationFailureV1::IncompatibleNominal {
                actual: to_stable_declaration_with_context(actual, self.types.identity_context()),
                expected: to_stable_declaration_with_context(expected, self.types.identity_context()),
            },
            RelationFailure::UnionMemberMismatch { actual, expected } => CallableExplainRelationFailureV1::UnionMemberMismatch {
                actual: self.types.ty(*actual)?,
                expected: self.types.ty(*expected)?,
            },
            RelationFailure::CycleDetected { sub, sup } => CallableExplainRelationFailureV1::CycleDetected {
                sub: self.types.ty(*sub)?,
                sup: self.types.ty(*sup)?,
            },
            RelationFailure::DepthExceeded => CallableExplainRelationFailureV1::DepthExceeded,
            RelationFailure::Custom(value) => CallableExplainRelationFailureV1::Custom { value: value.clone() },
        })
    }
    fn relationoutcome(&mut self, value: &RelationOutcome<()>) -> Result<CallableExplainRelationOutcomeV1, ArtifactError> {
        Ok(match value {
            RelationOutcome::Proven { value: (), evidence } => CallableExplainRelationOutcomeV1::Proven { notes: evidence.notes.clone() },
            RelationOutcome::Refuted(value) => CallableExplainRelationOutcomeV1::Refuted {
                value: self.relationfailure(value)?,
            },
            RelationOutcome::DynamicBoundary(obligation) => CallableExplainRelationOutcomeV1::DynamicBoundary {
                reason: obligation.reason.clone(),
            },
            RelationOutcome::Blocked(value) => CallableExplainRelationOutcomeV1::Blocked {
                value: export_callable_block_reason(value, self.types)?,
            },
            RelationOutcome::Cancelled => CallableExplainRelationOutcomeV1::Cancelled,
            RelationOutcome::BudgetExceeded(value) => CallableExplainRelationOutcomeV1::BudgetExceeded {
                value: export_binding_budget(value),
            },
            RelationOutcome::InternalFailure(value) => CallableExplainRelationOutcomeV1::InternalFailure { value: value.clone() },
        })
    }
}
pub(super) fn export_explanations(
    analysis: &crate::checker::analysis::CallableAnalysis,
    ordinals: &CallableAnalysisOrdinalMap,
    types: &TypeExportContext<'_>,
) -> Result<Vec<CallableExplanationNodeV1>, ArtifactError> {
    let mut export = ExplanationExport { ordinals, types };
    analysis
        .explanations
        .iter()
        .map(|(_, node)| {
            Ok(CallableExplanationNodeV1 {
                step: export.explanationstep(&node.step)?,
                rule: export.derivationrule(&node.rule)?,
                status: export.evidencestatus(&node.status)?,
                origin: export.evidenceorigin(&node.origin)?,
                evidence: node
                    .evidence
                    .iter()
                    .map(|evidence| export.evidenceref(evidence))
                    .collect::<Result<Vec<_>, _>>()?,
                parents: node
                    .parents
                    .iter()
                    .map(|id| {
                        ordinals
                            .explanations
                            .get(id)
                            .copied()
                            .ok_or_else(|| referential("explanation parent has no owner"))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            })
        })
        .collect()
}
