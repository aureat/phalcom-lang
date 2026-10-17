use super::*;
use crate::checker::expected::ExpectationOrigin;
use crate::dispatch::CallableSemanticKind;
use crate::explain::node::*;
use crate::identity::{BindingId, BodyId, DiagnosticCauseId, ExplanationId, ExpressionId, LocalExpressionId};
use crate::types::evidence::{EvidenceOrigin, EvidenceStatus};
use crate::types::outcome::{DynamicBoundaryObligation, RelationEvidence, RelationFailure, RelationOutcome};
use phalcom_compiler_artifact::semantic::aggregates::DynamicReasonV1;
use phalcom_compiler_artifact::semantic::world::*;

struct ExplanationImport<'a> {
    body: BodyId,
    relocation: &'a TypeRelocation,
    allocations: &'a mut AllocationBudget,
}
impl ExplanationImport<'_> {
    fn map<T, U>(&mut self, rows: &[T], mut f: impl FnMut(&mut Self, &T) -> Result<U, ArtifactError>) -> Result<Vec<U>, ArtifactError> {
        let mut output = self.allocations.vector(rows.len()).map_err(allocation_error)?;
        for row in rows {
            output.push(f(self, row)?);
        }
        Ok(output)
    }
    fn derivationrule(&mut self, value: &CallableDerivationRuleV1) -> Result<DerivationRule, ArtifactError> {
        Ok(match value {
            CallableDerivationRuleV1::LiteralSynthesis => DerivationRule::LiteralSynthesis,
            CallableDerivationRuleV1::ExpressionSynthesis => DerivationRule::ExpressionSynthesis,
            CallableDerivationRuleV1::NameResolution => DerivationRule::NameResolution,
            CallableDerivationRuleV1::AnnotationConstraint => DerivationRule::AnnotationConstraint,
            CallableDerivationRuleV1::BindingContract => DerivationRule::BindingContract,
            CallableDerivationRuleV1::TypeRequirement => DerivationRule::TypeRequirement,
            CallableDerivationRuleV1::TypeRelation => DerivationRule::TypeRelation,
            CallableDerivationRuleV1::CallableSelection => DerivationRule::CallableSelection,
            CallableDerivationRuleV1::UnionArm => DerivationRule::UnionArm,
            CallableDerivationRuleV1::CallableReturn => DerivationRule::CallableReturn,
            CallableDerivationRuleV1::SelfSpecialization => DerivationRule::SelfSpecialization,
            CallableDerivationRuleV1::ArgumentChecking => DerivationRule::ArgumentChecking,
            CallableDerivationRuleV1::CallShape => DerivationRule::CallShape,
            CallableDerivationRuleV1::GenericConstraint => DerivationRule::GenericConstraint,
            CallableDerivationRuleV1::GenericSolution => DerivationRule::GenericSolution,
            CallableDerivationRuleV1::GenericConflict => DerivationRule::GenericConflict,
            CallableDerivationRuleV1::CollectionSynthesis => DerivationRule::CollectionSynthesis,
            CallableDerivationRuleV1::ProductDecomposition => DerivationRule::ProductDecomposition,
            CallableDerivationRuleV1::FlowRefinement { predicate_kind } => DerivationRule::FlowRefinement {
                predicate_kind: self.predicatekind(predicate_kind)?,
            },
            CallableDerivationRuleV1::BranchJoin { branch_count } => DerivationRule::BranchJoin {
                branch_count: usize::try_from(*branch_count).map_err(|_| referential("explanation index overflow"))?,
            },
            CallableDerivationRuleV1::LoopJoin => DerivationRule::LoopJoin,
            CallableDerivationRuleV1::IterationElementResolution => DerivationRule::IterationElementResolution,
            CallableDerivationRuleV1::AssignmentPropagation => DerivationRule::AssignmentPropagation,
            CallableDerivationRuleV1::ReturnTypeCheck => DerivationRule::ReturnTypeCheck,
            CallableDerivationRuleV1::CallableReturnSummary => DerivationRule::CallableReturnSummary,
            CallableDerivationRuleV1::UnknownPropagation => DerivationRule::UnknownPropagation,
            CallableDerivationRuleV1::DynamicPropagation => DerivationRule::DynamicPropagation,
            CallableDerivationRuleV1::InternalBlocked => DerivationRule::InternalBlocked,
            CallableDerivationRuleV1::MethodCallReturn { selector } => DerivationRule::MethodCallReturn {
                selector: self.allocations.string(selector).map_err(allocation_error)?,
            },
            CallableDerivationRuleV1::GenericInstantiation { type_args } => DerivationRule::GenericInstantiation {
                type_args: self.map(type_args, |this, value| Ok(this.relocation.ty(*value)?))?,
            },
            CallableDerivationRuleV1::PolicyEnforcement { code } => DerivationRule::PolicyEnforcement {
                code: DiagnosticCode::from_stable_code(code).ok_or_else(|| referential("unknown explanation diagnostic code"))?,
            },
        })
    }
    fn evidenceref(&mut self, value: &CallableExplanationEvidenceV1) -> Result<EvidenceRef, ArtifactError> {
        Ok(match value {
            CallableExplanationEvidenceV1::SourceSpan { value } => EvidenceRef::SourceSpan(import_callable_range(value)?),
            CallableExplanationEvidenceV1::TypeId { value } => EvidenceRef::TypeId(self.relocation.ty(*value)?),
            CallableExplanationEvidenceV1::BindingVersion { binding, version } => EvidenceRef::BindingVersion {
                binding: BindingId(*binding),
                version: *version,
            },
            CallableExplanationEvidenceV1::Suppressed { cause } => EvidenceRef::Suppressed {
                cause: DiagnosticCauseId(*cause),
            },
        })
    }
    fn predicatekind(&mut self, value: &CallableExplainPredicateKindV1) -> Result<PredicateKind, ArtifactError> {
        Ok(match value {
            CallableExplainPredicateKindV1::IsInstance => PredicateKind::IsInstance,
            CallableExplainPredicateKindV1::IsNotInstance => PredicateKind::IsNotInstance,
            CallableExplainPredicateKindV1::IsNil => PredicateKind::IsNil,
            CallableExplainPredicateKindV1::NotNil => PredicateKind::NotNil,
            CallableExplainPredicateKindV1::EqualLiteral => PredicateKind::EqualLiteral,
            CallableExplainPredicateKindV1::NotEqualLiteral => PredicateKind::NotEqualLiteral,
            CallableExplainPredicateKindV1::Ordered => PredicateKind::Ordered,
            CallableExplainPredicateKindV1::Truthy => PredicateKind::Truthy,
            CallableExplainPredicateKindV1::Falsy => PredicateKind::Falsy,
        })
    }
    fn callshapeexplanation(&mut self, value: &CallableExplainCallShapeExplanationV1) -> Result<CallShapeExplanation, ArtifactError> {
        Ok(match value {
            CallableExplainCallShapeExplanationV1::MissingRequired { parameter_index, label } => CallShapeExplanation::MissingRequired {
                parameter_index: *parameter_index,
                label: label
                    .as_ref()
                    .map(|value| Ok::<_, ArtifactError>(self.allocations.string(value).map_err(allocation_error)?))
                    .transpose()?,
            },
            CallableExplainCallShapeExplanationV1::UnexpectedPositional { argument_index } => CallShapeExplanation::UnexpectedPositional {
                argument_index: *argument_index,
            },
            CallableExplainCallShapeExplanationV1::UnknownLabel { label } => CallShapeExplanation::UnknownLabel {
                label: self.allocations.string(label).map_err(allocation_error)?,
            },
            CallableExplainCallShapeExplanationV1::DuplicateLabel { label } => CallShapeExplanation::DuplicateLabel {
                label: self.allocations.string(label).map_err(allocation_error)?,
            },
            CallableExplainCallShapeExplanationV1::DuplicateParameter { parameter_index } => CallShapeExplanation::DuplicateParameter {
                parameter_index: *parameter_index,
            },
        })
    }
    fn collectionkind(&mut self, value: &CallableExplainCollectionKindV1) -> Result<CollectionKind, ArtifactError> {
        Ok(match value {
            CallableExplainCollectionKindV1::List => CollectionKind::List,
            CallableExplainCollectionKindV1::Array => CollectionKind::Array,
            CallableExplainCollectionKindV1::Vector => CollectionKind::Vector,
            CallableExplainCollectionKindV1::Set => CollectionKind::Set,
            CallableExplainCollectionKindV1::Map => CollectionKind::Map,
            CallableExplainCollectionKindV1::Tuple => CollectionKind::Tuple,
            CallableExplainCollectionKindV1::Record => CollectionKind::Record,
        })
    }
    fn genericconstraintorigin(&mut self, value: &CallableExplainGenericConstraintOriginV1) -> Result<GenericConstraintOrigin, ArtifactError> {
        Ok(match value {
            CallableExplainGenericConstraintOriginV1::Argument { parameter_index } => GenericConstraintOrigin::Argument {
                parameter_index: *parameter_index,
            },
            CallableExplainGenericConstraintOriginV1::ExpectedResult => GenericConstraintOrigin::ExpectedResult,
            CallableExplainGenericConstraintOriginV1::Receiver => GenericConstraintOrigin::Receiver,
            CallableExplainGenericConstraintOriginV1::ExplicitTypeArgument => GenericConstraintOrigin::ExplicitTypeArgument,
            CallableExplainGenericConstraintOriginV1::WhereClause => GenericConstraintOrigin::WhereClause,
        })
    }
    fn genericconstraintrelation(&mut self, value: &CallableExplainGenericConstraintRelationV1) -> Result<GenericConstraintRelation, ArtifactError> {
        Ok(match value {
            CallableExplainGenericConstraintRelationV1::Equal { value } => GenericConstraintRelation::Equal(self.relocation.ty(*value)?),
            CallableExplainGenericConstraintRelationV1::SubtypeOf { value } => GenericConstraintRelation::SubtypeOf(self.relocation.ty(*value)?),
            CallableExplainGenericConstraintRelationV1::SupertypeOf { value } => GenericConstraintRelation::SupertypeOf(self.relocation.ty(*value)?),
            CallableExplainGenericConstraintRelationV1::AssignableTo { value } => GenericConstraintRelation::AssignableTo(self.relocation.ty(*value)?),
        })
    }
    fn unionarmoutcome(&mut self, value: &CallableExplainUnionArmOutcomeV1) -> Result<UnionArmOutcome, ArtifactError> {
        Ok(match value {
            CallableExplainUnionArmOutcomeV1::Resolved => UnionArmOutcome::Resolved,
            CallableExplainUnionArmOutcomeV1::Missing { visited_owners } => UnionArmOutcome::Missing {
                visited_owners: self
                    .map(visited_owners, |this, value| {
                        Ok(import_stable_declaration_with_allocation_budget(value, this.allocations)?)
                    })?
                    .into_boxed_slice(),
            },
            CallableExplainUnionArmOutcomeV1::Ambiguous => UnionArmOutcome::Ambiguous,
            CallableExplainUnionArmOutcomeV1::Dynamic { reason } => UnionArmOutcome::Dynamic {
                reason: self.dynamicreason(reason)?,
            },
            CallableExplainUnionArmOutcomeV1::Invalid => UnionArmOutcome::Invalid,
            CallableExplainUnionArmOutcomeV1::ContextConflict => UnionArmOutcome::ContextConflict,
        })
    }
    fn explanationstep(&mut self, value: &CallableExplanationStepV1) -> Result<ExplanationStep, ArtifactError> {
        Ok(match value {
            CallableExplanationStepV1::Literal { expression, ty } => ExplanationStep::Literal {
                expression: ExpressionId::new(self.body, LocalExpressionId(*expression)),
                ty: self.relocation.ty(*ty)?,
            },
            CallableExplanationStepV1::ExpressionResult { expression, knowledge } => ExplanationStep::ExpressionResult {
                expression: ExpressionId::new(self.body, LocalExpressionId(*expression)),
                knowledge: import_type_knowledge_with_allocation_budget(knowledge, self.relocation, self.allocations)?,
            },
            CallableExplanationStepV1::BindingRead {
                expression,
                binding,
                knowledge,
            } => ExplanationStep::BindingRead {
                expression: ExpressionId::new(self.body, LocalExpressionId(*expression)),
                binding: BindingId(*binding),
                knowledge: import_type_knowledge_with_allocation_budget(knowledge, self.relocation, self.allocations)?,
            },
            CallableExplanationStepV1::Declared { binding, range, ty } => ExplanationStep::Declared {
                binding: binding.as_ref().map(|value| Ok::<_, ArtifactError>(BindingId(*value))).transpose()?,
                range: import_callable_range(range)?,
                ty: self.relocation.ty(*ty)?,
            },
            CallableExplanationStepV1::BindingContract {
                binding,
                actual,
                contract,
                consistency,
            } => ExplanationStep::BindingContract {
                binding: BindingId(*binding),
                actual: import_type_knowledge_with_allocation_budget(actual, self.relocation, self.allocations)?,
                contract: self.relocation.ty(*contract)?,
                consistency: import_callable_binding_consistency(consistency, self.relocation, self.allocations)?,
            },
            CallableExplanationStepV1::TypeRequirement { expected, origin, source } => ExplanationStep::TypeRequirement {
                expected: self.relocation.ty(*expected)?,
                origin: self.expectationorigin(origin)?,
                source: source
                    .as_ref()
                    .map(|value| {
                        Ok::<_, ArtifactError>(SemanticSourceSpan::new(
                            import_stable_module_with_allocation_budget(&value.module, self.allocations)?,
                            import_callable_range(&value.range)?,
                        ))
                    })
                    .transpose()?,
            },
            CallableExplanationStepV1::TypeRelation { actual, expected, outcome } => ExplanationStep::TypeRelation {
                actual: import_type_knowledge_with_allocation_budget(actual, self.relocation, self.allocations)?,
                expected: self.relocation.ty(*expected)?,
                outcome: self.relationoutcome(outcome)?,
            },
            CallableExplanationStepV1::CallableSelection {
                callable,
                receiver,
                declaring_owner,
                specialization_path,
            } => ExplanationStep::CallableSelection {
                callable: import_stable_callable_with_allocation_budget(callable, self.allocations)?,
                receiver: self.relocation.ty(*receiver)?,
                declaring_owner: import_stable_declaration_with_allocation_budget(declaring_owner, self.allocations)?,
                specialization_path: self
                    .map(specialization_path, |this, value| {
                        Ok(import_stable_declaration_with_allocation_budget(value, this.allocations)?)
                    })?
                    .into_boxed_slice(),
            },
            CallableExplanationStepV1::UnionArm { receiver, callable, outcome } => ExplanationStep::UnionArm {
                receiver: self.relocation.ty(*receiver)?,
                callable: callable
                    .as_ref()
                    .map(|value| Ok::<_, ArtifactError>(import_stable_callable_with_allocation_budget(value, self.allocations)?))
                    .transpose()?,
                outcome: self.unionarmoutcome(outcome)?,
            },
            CallableExplanationStepV1::CallableKind { callable, kind } => ExplanationStep::CallableKind {
                callable: import_stable_callable_with_allocation_budget(callable, self.allocations)?,
                kind: self.callablesemantickind(kind)?,
            },
            CallableExplanationStepV1::CallableReturn { callable, ty } => ExplanationStep::CallableReturn {
                callable: import_stable_callable_with_allocation_budget(callable, self.allocations)?,
                ty: self.relocation.ty(*ty)?,
            },
            CallableExplanationStepV1::SelfTypeSpecialization { self_ty, receiver, resolved } => ExplanationStep::SelfTypeSpecialization {
                self_ty: self.relocation.ty(*self_ty)?,
                receiver: self.relocation.ty(*receiver)?,
                resolved: self.relocation.ty(*resolved)?,
            },
            CallableExplanationStepV1::ArgumentCheck {
                call,
                argument,
                parameter_index,
                actual,
                expected,
            } => ExplanationStep::ArgumentCheck {
                call: ExpressionId::new(self.body, LocalExpressionId(*call)),
                argument: ExpressionId::new(self.body, LocalExpressionId(*argument)),
                parameter_index: *parameter_index,
                actual: import_type_knowledge_with_allocation_budget(actual, self.relocation, self.allocations)?,
                expected: self.relocation.ty(*expected)?,
            },
            CallableExplanationStepV1::CallShape { callable, failures } => ExplanationStep::CallShape {
                callable: callable
                    .as_ref()
                    .map(|value| Ok::<_, ArtifactError>(import_stable_callable_with_allocation_budget(value, self.allocations)?))
                    .transpose()?,
                failures: self.map(failures, |this, value| Ok(this.callshapeexplanation(value)?))?.into_boxed_slice(),
            },
            CallableExplanationStepV1::MethodCall { call, callable, return_ty } => ExplanationStep::MethodCall {
                call: ExpressionId::new(self.body, LocalExpressionId(*call)),
                callable: import_stable_callable_with_allocation_budget(callable, self.allocations)?,
                return_ty: self.relocation.ty(*return_ty)?,
            },
            CallableExplanationStepV1::UnresolvedCall { call, return_ty } => ExplanationStep::UnresolvedCall {
                call: ExpressionId::new(self.body, LocalExpressionId(*call)),
                return_ty: self.relocation.ty(*return_ty)?,
            },
            CallableExplanationStepV1::GenericConstraint { parameter, origin, relation } => ExplanationStep::GenericConstraint {
                parameter: self.relocation.parameter(*parameter)?,
                origin: self.genericconstraintorigin(origin)?,
                relation: self.genericconstraintrelation(relation)?,
            },
            CallableExplanationStepV1::GenericSolution { parameter, ty, status } => ExplanationStep::GenericSolution {
                parameter: self.relocation.parameter(*parameter)?,
                ty: self.relocation.ty(*ty)?,
                status: self.evidencestatus(status)?,
            },
            CallableExplanationStepV1::GenericConflict { parameter, constraints } => ExplanationStep::GenericConflict {
                parameter: parameter
                    .as_ref()
                    .map(|value| Ok::<_, ArtifactError>(self.relocation.parameter(*value)?))
                    .transpose()?,
                constraints: self.map(constraints, |_, value| Ok(ExplanationId(*value)))?.into_boxed_slice(),
            },
            CallableExplanationStepV1::CollectionSynthesis {
                expression,
                kind,
                element_types,
                result,
            } => ExplanationStep::CollectionSynthesis {
                expression: ExpressionId::new(self.body, LocalExpressionId(*expression)),
                kind: self.collectionkind(kind)?,
                element_types: self.map(element_types, |this, value| Ok(this.relocation.ty(*value)?))?.into_boxed_slice(),
                result: self.relocation.ty(*result)?,
            },
            CallableExplanationStepV1::ProductComponent { source, index, result } => ExplanationStep::ProductComponent {
                source: ExplanationId(*source),
                index: usize::try_from(*index).map_err(|_| referential("explanation index overflow"))?,
                result: import_type_knowledge_with_allocation_budget(result, self.relocation, self.allocations)?,
            },
            CallableExplanationStepV1::FlowRefinement {
                binding,
                predicate,
                prior,
                refined,
            } => ExplanationStep::FlowRefinement {
                binding: BindingId(*binding),
                predicate: self.predicatekind(predicate)?,
                prior: import_type_knowledge_with_allocation_budget(prior, self.relocation, self.allocations)?,
                refined: import_type_knowledge_with_allocation_budget(refined, self.relocation, self.allocations)?,
            },
            CallableExplanationStepV1::BranchJoin {
                binding,
                branches,
                reachable,
                joined,
            } => ExplanationStep::BranchJoin {
                binding: binding.as_ref().map(|value| Ok::<_, ArtifactError>(BindingId(*value))).transpose()?,
                branches: self
                    .map(branches, |this, value| {
                        Ok(import_type_knowledge_with_allocation_budget(value, this.relocation, this.allocations)?)
                    })?
                    .into_boxed_slice(),
                reachable: self.map(reachable, |_, value| Ok(*value))?.into_boxed_slice(),
                joined: import_type_knowledge_with_allocation_budget(joined, self.relocation, self.allocations)?,
            },
            CallableExplanationStepV1::LoopJoin { inputs, joined } => ExplanationStep::LoopJoin {
                inputs: self
                    .map(inputs, |this, value| {
                        Ok(import_type_knowledge_with_allocation_budget(value, this.relocation, this.allocations)?)
                    })?
                    .into_boxed_slice(),
                joined: import_type_knowledge_with_allocation_budget(joined, self.relocation, self.allocations)?,
            },
            CallableExplanationStepV1::IterationElement { iterable, element, callable } => ExplanationStep::IterationElement {
                iterable: import_type_knowledge_with_allocation_budget(iterable, self.relocation, self.allocations)?,
                element: import_type_knowledge_with_allocation_budget(element, self.relocation, self.allocations)?,
                callable: callable
                    .as_ref()
                    .map(|value| Ok::<_, ArtifactError>(import_stable_callable_with_allocation_budget(value, self.allocations)?))
                    .transpose()?,
            },
            CallableExplanationStepV1::ReturnCheck { actual, expected } => ExplanationStep::ReturnCheck {
                actual: import_type_knowledge_with_allocation_budget(actual, self.relocation, self.allocations)?,
                expected: expected.as_ref().map(|value| Ok::<_, ArtifactError>(self.relocation.ty(*value)?)).transpose()?,
            },
            CallableExplanationStepV1::CallableReturnSummary { callable, returns, result } => ExplanationStep::CallableReturnSummary {
                callable: import_stable_callable_with_allocation_budget(callable, self.allocations)?,
                returns: self
                    .map(returns, |this, value| {
                        Ok(import_type_knowledge_with_allocation_budget(value, this.relocation, this.allocations)?)
                    })?
                    .into_boxed_slice(),
                result: import_type_knowledge_with_allocation_budget(result, self.relocation, self.allocations)?,
            },
            CallableExplanationStepV1::UnknownBoundary { reason, source } => ExplanationStep::UnknownBoundary {
                reason: import_unknown_reason_with_allocation_budget(reason, self.allocations)?,
                source: source.as_ref().map(|value| Ok::<_, ArtifactError>(import_callable_range(value)?)).transpose()?,
            },
            CallableExplanationStepV1::DynamicBoundary { reason, source } => ExplanationStep::DynamicBoundary {
                reason: self.dynamicreason(reason)?,
                source: source.as_ref().map(|value| Ok::<_, ArtifactError>(import_callable_range(value)?)).transpose()?,
            },
            CallableExplanationStepV1::InternalBlocked { reason } => ExplanationStep::InternalBlocked {
                reason: import_callable_block_reason(reason, self.relocation, self.allocations)?,
            },
            CallableExplanationStepV1::Subtyping { actual, expected, proven } => ExplanationStep::Subtyping {
                actual: self.relocation.ty(*actual)?,
                expected: self.relocation.ty(*expected)?,
                proven: *proven,
            },
        })
    }
    fn expectationorigin(&mut self, value: &CallableExplainExpectationOriginV1) -> Result<ExpectationOrigin, ArtifactError> {
        Ok(match value {
            CallableExplainExpectationOriginV1::DeclarationContract => ExpectationOrigin::DeclarationContract,
            CallableExplainExpectationOriginV1::ReturnContract => ExpectationOrigin::ReturnContract,
            CallableExplainExpectationOriginV1::CallableSignature => ExpectationOrigin::CallableSignature,
            CallableExplainExpectationOriginV1::AssignmentContract => ExpectationOrigin::AssignmentContract,
            CallableExplainExpectationOriginV1::ContextualBlockParameter => ExpectationOrigin::ContextualBlockParameter,
            CallableExplainExpectationOriginV1::GenericArgument => ExpectationOrigin::GenericArgument,
            CallableExplainExpectationOriginV1::GenericResult => ExpectationOrigin::GenericResult,
            CallableExplainExpectationOriginV1::CollectionElement => ExpectationOrigin::CollectionElement,
            CallableExplainExpectationOriginV1::ProductComponent => ExpectationOrigin::ProductComponent,
            CallableExplainExpectationOriginV1::ExplicitCheck => ExpectationOrigin::ExplicitCheck,
            CallableExplainExpectationOriginV1::AlgebraicArithmetic => ExpectationOrigin::AlgebraicArithmetic,
        })
    }
    fn callablesemantickind(&mut self, value: &CallableExplainCallableSemanticKindV1) -> Result<CallableSemanticKind, ArtifactError> {
        Ok(match value {
            CallableExplainCallableSemanticKindV1::Ordinary => CallableSemanticKind::Ordinary,
            CallableExplainCallableSemanticKindV1::Constructor => CallableSemanticKind::Constructor,
            CallableExplainCallableSemanticKindV1::Native => CallableSemanticKind::Native,
        })
    }
    fn evidencestatus(&mut self, value: &EvidenceStatusV1) -> Result<EvidenceStatus, ArtifactError> {
        Ok(match value {
            EvidenceStatusV1::Established => EvidenceStatus::Established,
            EvidenceStatusV1::Assumed => EvidenceStatus::Assumed,
        })
    }
    fn evidenceorigin(&mut self, value: &EvidenceOriginV1) -> Result<EvidenceOrigin, ArtifactError> {
        Ok(match value {
            EvidenceOriginV1::Syntax => EvidenceOrigin::Syntax,
            EvidenceOriginV1::DeclarationSemantics => EvidenceOrigin::DeclarationSemantics,
            EvidenceOriginV1::ConstructorSemantics => EvidenceOrigin::ConstructorSemantics,
            EvidenceOriginV1::CallableSignature => EvidenceOrigin::CallableSignature,
            EvidenceOriginV1::NativeSignature => EvidenceOrigin::NativeSignature,
            EvidenceOriginV1::DeveloperAnnotation => EvidenceOrigin::DeveloperAnnotation,
            EvidenceOriginV1::GenericInference => EvidenceOrigin::GenericInference,
            EvidenceOriginV1::Flow => EvidenceOrigin::Flow,
            EvidenceOriginV1::FieldLifecycle => EvidenceOrigin::FieldLifecycle,
            EvidenceOriginV1::ContextualDerivation => EvidenceOrigin::ContextualDerivation,
            EvidenceOriginV1::PatternDecomposition => EvidenceOrigin::PatternDecomposition,
        })
    }
    fn dynamicreason(&mut self, value: &DynamicReasonV1) -> Result<DynamicReason, ArtifactError> {
        Ok(match value {
            DynamicReasonV1::ExplicitEscape => DynamicReason::ExplicitEscape,
            DynamicReasonV1::DynamicRestPack => DynamicReason::DynamicRestPack,
            DynamicReasonV1::RuntimeReflection => DynamicReason::RuntimeReflection,
        })
    }
    fn relationfailure(&mut self, value: &CallableExplainRelationFailureV1) -> Result<RelationFailure, ArtifactError> {
        Ok(match value {
            CallableExplainRelationFailureV1::TypeMismatch { actual, expected } => RelationFailure::TypeMismatch {
                actual: self.relocation.ty(*actual)?,
                expected: self.relocation.ty(*expected)?,
            },
            CallableExplainRelationFailureV1::IncompatibleNominal { actual, expected } => RelationFailure::IncompatibleNominal {
                actual: import_stable_declaration_with_allocation_budget(actual, self.allocations)?,
                expected: import_stable_declaration_with_allocation_budget(expected, self.allocations)?,
            },
            CallableExplainRelationFailureV1::UnionMemberMismatch { actual, expected } => RelationFailure::UnionMemberMismatch {
                actual: self.relocation.ty(*actual)?,
                expected: self.relocation.ty(*expected)?,
            },
            CallableExplainRelationFailureV1::CycleDetected { sub, sup } => RelationFailure::CycleDetected {
                sub: self.relocation.ty(*sub)?,
                sup: self.relocation.ty(*sup)?,
            },
            CallableExplainRelationFailureV1::DepthExceeded => RelationFailure::DepthExceeded,
            CallableExplainRelationFailureV1::Custom { value } => RelationFailure::Custom(self.allocations.string(value).map_err(allocation_error)?),
        })
    }
    fn relationoutcome(&mut self, value: &CallableExplainRelationOutcomeV1) -> Result<RelationOutcome<()>, ArtifactError> {
        Ok(match value {
            CallableExplainRelationOutcomeV1::Proven { notes } => RelationOutcome::Proven {
                value: (),
                evidence: RelationEvidence {
                    notes: self.map(notes, |this, value| Ok(this.allocations.string(value).map_err(allocation_error)?))?,
                },
            },
            CallableExplainRelationOutcomeV1::Refuted { value } => RelationOutcome::Refuted(self.relationfailure(value)?),
            CallableExplainRelationOutcomeV1::DynamicBoundary { reason } => RelationOutcome::DynamicBoundary(DynamicBoundaryObligation {
                reason: self.allocations.string(reason).map_err(allocation_error)?,
            }),
            CallableExplainRelationOutcomeV1::Blocked { value } => {
                RelationOutcome::Blocked(import_callable_block_reason(value, self.relocation, self.allocations)?)
            }
            CallableExplainRelationOutcomeV1::Cancelled => RelationOutcome::Cancelled,
            CallableExplainRelationOutcomeV1::BudgetExceeded { value } => RelationOutcome::BudgetExceeded(import_callable_binding_budget(value)),
            CallableExplainRelationOutcomeV1::InternalFailure { value } => {
                RelationOutcome::InternalFailure(self.allocations.string(value).map_err(allocation_error)?)
            }
        })
    }
}
pub(super) fn import_explanations(
    rows: &[CallableExplanationNodeV1],
    body: BodyId,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<Arc<crate::explain::ExplanationArena>, ArtifactError> {
    let mut child = allocations.child(allocations.remaining());
    let mut import = ExplanationImport {
        body,
        relocation,
        allocations: &mut child,
    };
    let mut entries = import.allocations.vector(rows.len()).map_err(allocation_error)?;
    for (ordinal, row) in rows.iter().enumerate() {
        let id = ExplanationId(u32::try_from(ordinal).map_err(|_| referential("explanation ordinal overflow"))?);
        let node = ExplanationNode {
            id,
            step: import.explanationstep(&row.step)?,
            rule: import.derivationrule(&row.rule)?,
            status: import.evidencestatus(&row.status)?,
            origin: import.evidenceorigin(&row.origin)?,
            evidence: import.map(&row.evidence, |this, value| this.evidenceref(value))?,
            parents: import.map(&row.parents, |_, value| Ok(ExplanationId(*value)))?,
        };
        entries.push((id, node));
    }
    let arena = crate::explain::ExplanationArena::from_budgeted_entries(entries, &mut child).map_err(|error| match error {
        crate::explain::arena::ExplanationArenaBuildError::Allocation(error) => allocation_error(error),
        _ => referential("explanation entries are not canonical"),
    })?;
    let arena = child.arc(arena).map_err(allocation_error)?;
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(arena)
}
