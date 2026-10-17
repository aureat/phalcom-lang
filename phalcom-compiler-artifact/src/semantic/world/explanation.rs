//! Lossless callable-local explanation graph wire records.
use super::*;
use crate::semantic::aggregates::DynamicReasonV1;
tagged!(CallableDerivationRuleV1 {
 0 => LiteralSynthesis,
 1 => ExpressionSynthesis,
 2 => NameResolution,
 3 => AnnotationConstraint,
 4 => BindingContract,
 5 => TypeRequirement,
 6 => TypeRelation,
 7 => CallableSelection,
 8 => UnionArm,
 9 => CallableReturn,
 10 => SelfSpecialization,
 11 => ArgumentChecking,
 12 => CallShape,
 13 => GenericConstraint,
 14 => GenericSolution,
 15 => GenericConflict,
 16 => CollectionSynthesis,
 17 => ProductDecomposition,
 18 => FlowRefinement { predicate_kind: CallableExplainPredicateKindV1 },
 19 => BranchJoin { branch_count: u64 },
 20 => LoopJoin,
 21 => IterationElementResolution,
 22 => AssignmentPropagation,
 23 => ReturnTypeCheck,
 24 => CallableReturnSummary,
 25 => UnknownPropagation,
 26 => DynamicPropagation,
 27 => InternalBlocked,
 28 => MethodCallReturn { selector: String },
 29 => GenericInstantiation { type_args: Vec<ArtifactTypeId> },
 30 => PolicyEnforcement { code: Box<str> },
});
tagged!(CallableExplanationEvidenceV1 {
 0 => SourceSpan { value: SpanV1 },
 1 => TypeId { value: ArtifactTypeId },
 2 => BindingVersion { binding: u32, version: u32 },
 3 => Suppressed { cause: u32 },
});
tagged!(CallableExplainPredicateKindV1 {
 0 => IsInstance,
 1 => IsNotInstance,
 2 => IsNil,
 3 => NotNil,
 4 => EqualLiteral,
 5 => NotEqualLiteral,
 6 => Ordered,
 7 => Truthy,
 8 => Falsy,
});
tagged!(CallableExplainCallShapeExplanationV1 {
 0 => MissingRequired { parameter_index: u16, label: Option<String> },
 1 => UnexpectedPositional { argument_index: u16 },
 2 => UnknownLabel { label: String },
 3 => DuplicateLabel { label: String },
 4 => DuplicateParameter { parameter_index: u16 },
});
tagged!(CallableExplainCollectionKindV1 {
 0 => List,
 1 => Array,
 2 => Vector,
 3 => Set,
 4 => Map,
 5 => Tuple,
 6 => Record,
});
tagged!(CallableExplainGenericConstraintOriginV1 {
 0 => Argument { parameter_index: u16 },
 1 => ExpectedResult,
 2 => Receiver,
 3 => ExplicitTypeArgument,
 4 => WhereClause,
});
tagged!(CallableExplainGenericConstraintRelationV1 {
 0 => Equal { value: ArtifactTypeId },
 1 => SubtypeOf { value: ArtifactTypeId },
 2 => SupertypeOf { value: ArtifactTypeId },
 3 => AssignableTo { value: ArtifactTypeId },
});
tagged!(CallableExplainUnionArmOutcomeV1 {
 0 => Resolved,
 1 => Missing { visited_owners: Vec<StableDeclarationRef> },
 2 => Ambiguous,
 3 => Dynamic { reason: DynamicReasonV1 },
 4 => Invalid,
 5 => ContextConflict,
});
tagged!(CallableExplanationStepV1 {
 0 => Literal { expression: u32, ty: ArtifactTypeId },
 1 => ExpressionResult { expression: u32, knowledge: TypeKnowledgeV1 },
 2 => BindingRead { expression: u32, binding: u32, knowledge: TypeKnowledgeV1 },
 3 => Declared { binding: Option<u32>, range: SpanV1, ty: ArtifactTypeId },
 4 => BindingContract { binding: u32, actual: TypeKnowledgeV1, contract: ArtifactTypeId, consistency: CallableBindingConsistencyV1 },
 5 => TypeRequirement { expected: ArtifactTypeId, origin: CallableExplainExpectationOriginV1, source: Option<EnumRequirementSourceV1> },
 6 => TypeRelation { actual: TypeKnowledgeV1, expected: ArtifactTypeId, outcome: CallableExplainRelationOutcomeV1 },
 7 => CallableSelection { callable: StableCallableRef, receiver: ArtifactTypeId, declaring_owner: StableDeclarationRef, specialization_path: Vec<StableDeclarationRef> },
 8 => UnionArm { receiver: ArtifactTypeId, callable: Option<StableCallableRef>, outcome: CallableExplainUnionArmOutcomeV1 },
 9 => CallableKind { callable: StableCallableRef, kind: CallableExplainCallableSemanticKindV1 },
 10 => CallableReturn { callable: StableCallableRef, ty: ArtifactTypeId },
 11 => SelfTypeSpecialization { self_ty: ArtifactTypeId, receiver: ArtifactTypeId, resolved: ArtifactTypeId },
 12 => ArgumentCheck { call: u32, argument: u32, parameter_index: u16, actual: TypeKnowledgeV1, expected: ArtifactTypeId },
 13 => CallShape { callable: Option<StableCallableRef>, failures: Vec<CallableExplainCallShapeExplanationV1> },
 14 => MethodCall { call: u32, callable: StableCallableRef, return_ty: ArtifactTypeId },
 15 => UnresolvedCall { call: u32, return_ty: ArtifactTypeId },
 16 => GenericConstraint { parameter: ArtifactTypeParameterId, origin: CallableExplainGenericConstraintOriginV1, relation: CallableExplainGenericConstraintRelationV1 },
 17 => GenericSolution { parameter: ArtifactTypeParameterId, ty: ArtifactTypeId, status: EvidenceStatusV1 },
 18 => GenericConflict { parameter: Option<ArtifactTypeParameterId>, constraints: Vec<u32> },
 19 => CollectionSynthesis { expression: u32, kind: CallableExplainCollectionKindV1, element_types: Vec<ArtifactTypeId>, result: ArtifactTypeId },
 20 => ProductComponent { source: u32, index: u64, result: TypeKnowledgeV1 },
 21 => FlowRefinement { binding: u32, predicate: CallableExplainPredicateKindV1, prior: TypeKnowledgeV1, refined: TypeKnowledgeV1 },
 22 => BranchJoin { binding: Option<u32>, branches: Vec<TypeKnowledgeV1>, reachable: Vec<bool>, joined: TypeKnowledgeV1 },
 23 => LoopJoin { inputs: Vec<TypeKnowledgeV1>, joined: TypeKnowledgeV1 },
 24 => IterationElement { iterable: TypeKnowledgeV1, element: TypeKnowledgeV1, callable: Option<StableCallableRef> },
 25 => ReturnCheck { actual: TypeKnowledgeV1, expected: Option<ArtifactTypeId> },
 26 => CallableReturnSummary { callable: StableCallableRef, returns: Vec<TypeKnowledgeV1>, result: TypeKnowledgeV1 },
 27 => UnknownBoundary { reason: UnknownReasonV1, source: Option<SpanV1> },
 28 => DynamicBoundary { reason: DynamicReasonV1, source: Option<SpanV1> },
 29 => InternalBlocked { reason: CallableBlockReasonV1 },
 30 => Subtyping { actual: ArtifactTypeId, expected: ArtifactTypeId, proven: bool },
});
tagged!(CallableExplainExpectationOriginV1 {
 0 => DeclarationContract,
 1 => ReturnContract,
 2 => CallableSignature,
 3 => AssignmentContract,
 4 => ContextualBlockParameter,
 5 => GenericArgument,
 6 => GenericResult,
 7 => CollectionElement,
 8 => ProductComponent,
 9 => ExplicitCheck,
 10 => AlgebraicArithmetic,
});
tagged!(CallableExplainCallableSemanticKindV1 {
 0 => Ordinary,
 1 => Constructor,
 2 => Native,
});
tagged!(CallableExplainRelationFailureV1 {
 0 => TypeMismatch { actual: ArtifactTypeId, expected: ArtifactTypeId },
 1 => IncompatibleNominal { actual: StableDeclarationRef, expected: StableDeclarationRef },
 2 => UnionMemberMismatch { actual: ArtifactTypeId, expected: ArtifactTypeId },
 3 => CycleDetected { sub: ArtifactTypeId, sup: ArtifactTypeId },
 4 => DepthExceeded,
 5 => Custom { value: String },
});
tagged!(CallableExplainRelationOutcomeV1 {
 0 => Proven { notes: Vec<String> },
 1 => Refuted { value: CallableExplainRelationFailureV1 },
 2 => DynamicBoundary { reason: String },
 3 => Blocked { value: CallableBlockReasonV1 },
 4 => Cancelled,
 5 => BudgetExceeded { value: CallableBudgetReportV1 },
 6 => InternalFailure { value: String },
});
record!(CallableExplanationNodeV1 { step: CallableExplanationStepV1, rule: CallableDerivationRuleV1, status: EvidenceStatusV1, origin: EvidenceOriginV1, evidence: Vec<CallableExplanationEvidenceV1>, parents: Vec<u32> });

struct ExplanationValidation<'a> {
    analysis: &'a CallableAnalysisRecordV1,
    world: &'a SemanticWorldV3,
    limits: &'a ValidationLimits,
}
impl ExplanationValidation<'_> {
    fn callable(&self, callable: &StableCallableRef) -> Result<(), ArtifactError> {
        self.string(&callable.selector)?;
        if let Some(owner) = callable.owner.enclosing_declaration() {
            self.world.validate_declaration_reference(owner)?;
        }
        Ok(())
    }

    fn local(&self, value: u32, count: usize) -> Result<(), ArtifactError> {
        if value as usize >= count {
            return Err(invalid("explanation local reference has no owner"));
        }
        Ok(())
    }
    fn string(&self, value: &str) -> Result<(), ArtifactError> {
        if value.len() > self.limits.max_string_bytes {
            return Err(invalid("explanation string limit exceeded"));
        }
        Ok(())
    }
    fn span(&self, value: &SpanV1) -> Result<(), ArtifactError> {
        if value.start > value.end {
            return Err(invalid("explanation source span is reversed"));
        }
        Ok(())
    }

    fn derivationrule(&self, value: &CallableDerivationRuleV1) -> Result<(), ArtifactError> {
        match value {
            CallableDerivationRuleV1::LiteralSynthesis => {}
            CallableDerivationRuleV1::ExpressionSynthesis => {}
            CallableDerivationRuleV1::NameResolution => {}
            CallableDerivationRuleV1::AnnotationConstraint => {}
            CallableDerivationRuleV1::BindingContract => {}
            CallableDerivationRuleV1::TypeRequirement => {}
            CallableDerivationRuleV1::TypeRelation => {}
            CallableDerivationRuleV1::CallableSelection => {}
            CallableDerivationRuleV1::UnionArm => {}
            CallableDerivationRuleV1::CallableReturn => {}
            CallableDerivationRuleV1::SelfSpecialization => {}
            CallableDerivationRuleV1::ArgumentChecking => {}
            CallableDerivationRuleV1::CallShape => {}
            CallableDerivationRuleV1::GenericConstraint => {}
            CallableDerivationRuleV1::GenericSolution => {}
            CallableDerivationRuleV1::GenericConflict => {}
            CallableDerivationRuleV1::CollectionSynthesis => {}
            CallableDerivationRuleV1::ProductDecomposition => {}
            CallableDerivationRuleV1::FlowRefinement { predicate_kind } => {
                self.predicatekind(predicate_kind)?;
            }
            CallableDerivationRuleV1::BranchJoin { branch_count: _ } => {}
            CallableDerivationRuleV1::LoopJoin => {}
            CallableDerivationRuleV1::IterationElementResolution => {}
            CallableDerivationRuleV1::AssignmentPropagation => {}
            CallableDerivationRuleV1::ReturnTypeCheck => {}
            CallableDerivationRuleV1::CallableReturnSummary => {}
            CallableDerivationRuleV1::UnknownPropagation => {}
            CallableDerivationRuleV1::DynamicPropagation => {}
            CallableDerivationRuleV1::InternalBlocked => {}
            CallableDerivationRuleV1::MethodCallReturn { selector } => {
                self.string(selector)?;
            }
            CallableDerivationRuleV1::GenericInstantiation { type_args } => {
                if type_args.len() > self.limits.max_semantic_entries_per_family {
                    return Err(invalid("explanation vector limit exceeded"));
                }
                for value in type_args {
                    validate_type_id(*value, &self.world.type_graph)?;
                }
            }
            CallableDerivationRuleV1::PolicyEnforcement { code } => {
                self.string(code)?;
            }
        }
        Ok(())
    }
    fn evidenceref(&self, value: &CallableExplanationEvidenceV1) -> Result<(), ArtifactError> {
        match value {
            CallableExplanationEvidenceV1::SourceSpan { value } => {
                self.span(value)?;
            }
            CallableExplanationEvidenceV1::TypeId { value } => {
                validate_type_id(*value, &self.world.type_graph)?;
            }
            CallableExplanationEvidenceV1::BindingVersion { binding, version: _ } => {
                self.local(*binding, self.analysis.bindings.len())?;
            }
            CallableExplanationEvidenceV1::Suppressed { cause } => {
                self.local(*cause, self.analysis.diagnostic_cause_count as usize)?;
            }
        }
        Ok(())
    }
    fn predicatekind(&self, value: &CallableExplainPredicateKindV1) -> Result<(), ArtifactError> {
        match value {
            CallableExplainPredicateKindV1::IsInstance => {}
            CallableExplainPredicateKindV1::IsNotInstance => {}
            CallableExplainPredicateKindV1::IsNil => {}
            CallableExplainPredicateKindV1::NotNil => {}
            CallableExplainPredicateKindV1::EqualLiteral => {}
            CallableExplainPredicateKindV1::NotEqualLiteral => {}
            CallableExplainPredicateKindV1::Ordered => {}
            CallableExplainPredicateKindV1::Truthy => {}
            CallableExplainPredicateKindV1::Falsy => {}
        }
        Ok(())
    }
    fn callshapeexplanation(&self, value: &CallableExplainCallShapeExplanationV1) -> Result<(), ArtifactError> {
        match value {
            CallableExplainCallShapeExplanationV1::MissingRequired { parameter_index: _, label } => {
                if let Some(value) = label {
                    self.string(value)?;
                }
            }
            CallableExplainCallShapeExplanationV1::UnexpectedPositional { argument_index: _ } => {}
            CallableExplainCallShapeExplanationV1::UnknownLabel { label } => {
                self.string(label)?;
            }
            CallableExplainCallShapeExplanationV1::DuplicateLabel { label } => {
                self.string(label)?;
            }
            CallableExplainCallShapeExplanationV1::DuplicateParameter { parameter_index: _ } => {}
        }
        Ok(())
    }
    fn collectionkind(&self, value: &CallableExplainCollectionKindV1) -> Result<(), ArtifactError> {
        match value {
            CallableExplainCollectionKindV1::List => {}
            CallableExplainCollectionKindV1::Array => {}
            CallableExplainCollectionKindV1::Vector => {}
            CallableExplainCollectionKindV1::Set => {}
            CallableExplainCollectionKindV1::Map => {}
            CallableExplainCollectionKindV1::Tuple => {}
            CallableExplainCollectionKindV1::Record => {}
        }
        Ok(())
    }
    fn genericconstraintorigin(&self, value: &CallableExplainGenericConstraintOriginV1) -> Result<(), ArtifactError> {
        match value {
            CallableExplainGenericConstraintOriginV1::Argument { parameter_index: _ } => {}
            CallableExplainGenericConstraintOriginV1::ExpectedResult => {}
            CallableExplainGenericConstraintOriginV1::Receiver => {}
            CallableExplainGenericConstraintOriginV1::ExplicitTypeArgument => {}
            CallableExplainGenericConstraintOriginV1::WhereClause => {}
        }
        Ok(())
    }
    fn genericconstraintrelation(&self, value: &CallableExplainGenericConstraintRelationV1) -> Result<(), ArtifactError> {
        match value {
            CallableExplainGenericConstraintRelationV1::Equal { value } => {
                validate_type_id(*value, &self.world.type_graph)?;
            }
            CallableExplainGenericConstraintRelationV1::SubtypeOf { value } => {
                validate_type_id(*value, &self.world.type_graph)?;
            }
            CallableExplainGenericConstraintRelationV1::SupertypeOf { value } => {
                validate_type_id(*value, &self.world.type_graph)?;
            }
            CallableExplainGenericConstraintRelationV1::AssignableTo { value } => {
                validate_type_id(*value, &self.world.type_graph)?;
            }
        }
        Ok(())
    }
    fn unionarmoutcome(&self, value: &CallableExplainUnionArmOutcomeV1) -> Result<(), ArtifactError> {
        match value {
            CallableExplainUnionArmOutcomeV1::Resolved => {}
            CallableExplainUnionArmOutcomeV1::Missing { visited_owners } => {
                if visited_owners.len() > self.limits.max_semantic_entries_per_family {
                    return Err(invalid("explanation vector limit exceeded"));
                }
            }
            CallableExplainUnionArmOutcomeV1::Ambiguous => {}
            CallableExplainUnionArmOutcomeV1::Dynamic { reason } => {
                self.dynamicreason(reason)?;
            }
            CallableExplainUnionArmOutcomeV1::Invalid => {}
            CallableExplainUnionArmOutcomeV1::ContextConflict => {}
        }
        Ok(())
    }
    fn explanationstep(&self, value: &CallableExplanationStepV1) -> Result<(), ArtifactError> {
        match value {
            CallableExplanationStepV1::Literal { expression, ty } => {
                self.local(*expression, self.analysis.expressions.len())?;
                validate_type_id(*ty, &self.world.type_graph)?;
            }
            CallableExplanationStepV1::ExpressionResult { expression, knowledge } => {
                self.local(*expression, self.analysis.expressions.len())?;
                validate_dispatch_knowledge(knowledge, &self.world.type_graph, self.limits)?;
            }
            CallableExplanationStepV1::BindingRead {
                expression,
                binding,
                knowledge,
            } => {
                self.local(*expression, self.analysis.expressions.len())?;
                self.local(*binding, self.analysis.bindings.len())?;
                validate_dispatch_knowledge(knowledge, &self.world.type_graph, self.limits)?;
            }
            CallableExplanationStepV1::Declared { binding, range, ty } => {
                if let Some(value) = binding {
                    self.local(*value, self.analysis.bindings.len())?;
                }
                self.span(range)?;
                validate_type_id(*ty, &self.world.type_graph)?;
            }
            CallableExplanationStepV1::BindingContract {
                binding,
                actual,
                contract,
                consistency,
            } => {
                self.local(*binding, self.analysis.bindings.len())?;
                validate_dispatch_knowledge(actual, &self.world.type_graph, self.limits)?;
                validate_type_id(*contract, &self.world.type_graph)?;
                self.consistency(consistency)?;
            }
            CallableExplanationStepV1::TypeRequirement { expected, origin, source } => {
                validate_type_id(*expected, &self.world.type_graph)?;
                self.expectationorigin(origin)?;
                if let Some(value) = source {
                    self.span(&value.range)?;
                }
            }
            CallableExplanationStepV1::TypeRelation { actual, expected, outcome } => {
                validate_dispatch_knowledge(actual, &self.world.type_graph, self.limits)?;
                validate_type_id(*expected, &self.world.type_graph)?;
                self.relationoutcome(outcome)?;
            }
            CallableExplanationStepV1::CallableSelection {
                callable,
                receiver,
                declaring_owner,
                specialization_path,
            } => {
                self.callable(callable)?;
                validate_type_id(*receiver, &self.world.type_graph)?;
                self.world.validate_declaration_reference(declaring_owner)?;
                if specialization_path.len() > self.limits.max_semantic_entries_per_family {
                    return Err(invalid("explanation vector limit exceeded"));
                }
            }
            CallableExplanationStepV1::UnionArm {
                receiver,
                callable: _,
                outcome,
            } => {
                validate_type_id(*receiver, &self.world.type_graph)?;
                self.unionarmoutcome(outcome)?;
            }
            CallableExplanationStepV1::CallableKind { callable, kind } => {
                self.callable(callable)?;
                self.callablesemantickind(kind)?;
            }
            CallableExplanationStepV1::CallableReturn { callable, ty } => {
                self.callable(callable)?;
                validate_type_id(*ty, &self.world.type_graph)?;
            }
            CallableExplanationStepV1::SelfTypeSpecialization { self_ty, receiver, resolved } => {
                validate_type_id(*self_ty, &self.world.type_graph)?;
                validate_type_id(*receiver, &self.world.type_graph)?;
                validate_type_id(*resolved, &self.world.type_graph)?;
            }
            CallableExplanationStepV1::ArgumentCheck {
                call,
                argument,
                parameter_index: _,
                actual,
                expected,
            } => {
                self.local(*call, self.analysis.expressions.len())?;
                self.local(*argument, self.analysis.expressions.len())?;
                validate_dispatch_knowledge(actual, &self.world.type_graph, self.limits)?;
                validate_type_id(*expected, &self.world.type_graph)?;
            }
            CallableExplanationStepV1::CallShape { callable: _, failures } => {
                if failures.len() > self.limits.max_semantic_entries_per_family {
                    return Err(invalid("explanation vector limit exceeded"));
                }
                for value in failures {
                    self.callshapeexplanation(value)?;
                }
            }
            CallableExplanationStepV1::MethodCall { call, callable, return_ty } => {
                self.local(*call, self.analysis.expressions.len())?;
                self.callable(callable)?;
                validate_type_id(*return_ty, &self.world.type_graph)?;
            }
            CallableExplanationStepV1::UnresolvedCall { call, return_ty } => {
                self.local(*call, self.analysis.expressions.len())?;
                validate_type_id(*return_ty, &self.world.type_graph)?;
            }
            CallableExplanationStepV1::GenericConstraint { parameter, origin, relation } => {
                self.local(parameter.0, self.world.type_graph.parameters.len())?;
                self.genericconstraintorigin(origin)?;
                self.genericconstraintrelation(relation)?;
            }
            CallableExplanationStepV1::GenericSolution { parameter, ty, status } => {
                self.local(parameter.0, self.world.type_graph.parameters.len())?;
                validate_type_id(*ty, &self.world.type_graph)?;
                self.evidencestatus(status)?;
            }
            CallableExplanationStepV1::GenericConflict { parameter, constraints } => {
                if let Some(value) = parameter {
                    self.local(value.0, self.world.type_graph.parameters.len())?;
                }
                if constraints.len() > self.limits.max_semantic_entries_per_family {
                    return Err(invalid("explanation vector limit exceeded"));
                }
                for value in constraints {
                    self.local(*value, self.analysis.explanations.len())?;
                }
            }
            CallableExplanationStepV1::CollectionSynthesis {
                expression,
                kind,
                element_types,
                result,
            } => {
                self.local(*expression, self.analysis.expressions.len())?;
                self.collectionkind(kind)?;
                if element_types.len() > self.limits.max_semantic_entries_per_family {
                    return Err(invalid("explanation vector limit exceeded"));
                }
                for value in element_types {
                    validate_type_id(*value, &self.world.type_graph)?;
                }
                validate_type_id(*result, &self.world.type_graph)?;
            }
            CallableExplanationStepV1::ProductComponent { source, index: _, result } => {
                self.local(*source, self.analysis.explanations.len())?;
                validate_dispatch_knowledge(result, &self.world.type_graph, self.limits)?;
            }
            CallableExplanationStepV1::FlowRefinement {
                binding,
                predicate,
                prior,
                refined,
            } => {
                self.local(*binding, self.analysis.bindings.len())?;
                self.predicatekind(predicate)?;
                validate_dispatch_knowledge(prior, &self.world.type_graph, self.limits)?;
                validate_dispatch_knowledge(refined, &self.world.type_graph, self.limits)?;
            }
            CallableExplanationStepV1::BranchJoin {
                binding,
                branches,
                reachable,
                joined,
            } => {
                if let Some(value) = binding {
                    self.local(*value, self.analysis.bindings.len())?;
                }
                if branches.len() > self.limits.max_semantic_entries_per_family {
                    return Err(invalid("explanation vector limit exceeded"));
                }
                for value in branches {
                    validate_dispatch_knowledge(value, &self.world.type_graph, self.limits)?;
                }
                if reachable.len() > self.limits.max_semantic_entries_per_family {
                    return Err(invalid("explanation vector limit exceeded"));
                }
                validate_dispatch_knowledge(joined, &self.world.type_graph, self.limits)?;
            }
            CallableExplanationStepV1::LoopJoin { inputs, joined } => {
                if inputs.len() > self.limits.max_semantic_entries_per_family {
                    return Err(invalid("explanation vector limit exceeded"));
                }
                for value in inputs {
                    validate_dispatch_knowledge(value, &self.world.type_graph, self.limits)?;
                }
                validate_dispatch_knowledge(joined, &self.world.type_graph, self.limits)?;
            }
            CallableExplanationStepV1::IterationElement {
                iterable,
                element,
                callable: _,
            } => {
                validate_dispatch_knowledge(iterable, &self.world.type_graph, self.limits)?;
                validate_dispatch_knowledge(element, &self.world.type_graph, self.limits)?;
            }
            CallableExplanationStepV1::ReturnCheck { actual, expected } => {
                validate_dispatch_knowledge(actual, &self.world.type_graph, self.limits)?;
                if let Some(value) = expected {
                    validate_type_id(*value, &self.world.type_graph)?;
                }
            }
            CallableExplanationStepV1::CallableReturnSummary { callable, returns, result } => {
                self.callable(callable)?;
                if returns.len() > self.limits.max_semantic_entries_per_family {
                    return Err(invalid("explanation vector limit exceeded"));
                }
                for value in returns {
                    validate_dispatch_knowledge(value, &self.world.type_graph, self.limits)?;
                }
                validate_dispatch_knowledge(result, &self.world.type_graph, self.limits)?;
            }
            CallableExplanationStepV1::UnknownBoundary { reason, source } => {
                if let UnknownReasonV1::UnresolvedName { name } = reason {
                    self.string(name)?;
                }
                if let Some(value) = source {
                    self.span(value)?;
                }
            }
            CallableExplanationStepV1::DynamicBoundary { reason, source } => {
                self.dynamicreason(reason)?;
                if let Some(value) = source {
                    self.span(value)?;
                }
            }
            CallableExplanationStepV1::InternalBlocked { reason } => {
                validate_callable_block_reason(reason, &self.world.type_graph, self.limits)?;
            }
            CallableExplanationStepV1::Subtyping { actual, expected, proven: _ } => {
                validate_type_id(*actual, &self.world.type_graph)?;
                validate_type_id(*expected, &self.world.type_graph)?;
            }
        }
        Ok(())
    }
    fn expectationorigin(&self, value: &CallableExplainExpectationOriginV1) -> Result<(), ArtifactError> {
        match value {
            CallableExplainExpectationOriginV1::DeclarationContract => {}
            CallableExplainExpectationOriginV1::ReturnContract => {}
            CallableExplainExpectationOriginV1::CallableSignature => {}
            CallableExplainExpectationOriginV1::AssignmentContract => {}
            CallableExplainExpectationOriginV1::ContextualBlockParameter => {}
            CallableExplainExpectationOriginV1::GenericArgument => {}
            CallableExplainExpectationOriginV1::GenericResult => {}
            CallableExplainExpectationOriginV1::CollectionElement => {}
            CallableExplainExpectationOriginV1::ProductComponent => {}
            CallableExplainExpectationOriginV1::ExplicitCheck => {}
            CallableExplainExpectationOriginV1::AlgebraicArithmetic => {}
        }
        Ok(())
    }
    fn callablesemantickind(&self, value: &CallableExplainCallableSemanticKindV1) -> Result<(), ArtifactError> {
        match value {
            CallableExplainCallableSemanticKindV1::Ordinary => {}
            CallableExplainCallableSemanticKindV1::Constructor => {}
            CallableExplainCallableSemanticKindV1::Native => {}
        }
        Ok(())
    }
    fn evidencestatus(&self, value: &EvidenceStatusV1) -> Result<(), ArtifactError> {
        match value {
            EvidenceStatusV1::Established => {}
            EvidenceStatusV1::Assumed => {}
        }
        Ok(())
    }
    fn evidenceorigin(&self, value: &EvidenceOriginV1) -> Result<(), ArtifactError> {
        match value {
            EvidenceOriginV1::Syntax => {}
            EvidenceOriginV1::DeclarationSemantics => {}
            EvidenceOriginV1::ConstructorSemantics => {}
            EvidenceOriginV1::CallableSignature => {}
            EvidenceOriginV1::NativeSignature => {}
            EvidenceOriginV1::DeveloperAnnotation => {}
            EvidenceOriginV1::GenericInference => {}
            EvidenceOriginV1::Flow => {}
            EvidenceOriginV1::FieldLifecycle => {}
            EvidenceOriginV1::ContextualDerivation => {}
            EvidenceOriginV1::PatternDecomposition => {}
        }
        Ok(())
    }
    fn dynamicreason(&self, value: &DynamicReasonV1) -> Result<(), ArtifactError> {
        match value {
            DynamicReasonV1::ExplicitEscape => {}
            DynamicReasonV1::DynamicRestPack => {}
            DynamicReasonV1::RuntimeReflection => {}
        }
        Ok(())
    }
    fn relationfailure(&self, value: &CallableExplainRelationFailureV1) -> Result<(), ArtifactError> {
        match value {
            CallableExplainRelationFailureV1::TypeMismatch { actual, expected } => {
                validate_type_id(*actual, &self.world.type_graph)?;
                validate_type_id(*expected, &self.world.type_graph)?;
            }
            CallableExplainRelationFailureV1::IncompatibleNominal { actual: _, expected: _ } => {}
            CallableExplainRelationFailureV1::UnionMemberMismatch { actual, expected } => {
                validate_type_id(*actual, &self.world.type_graph)?;
                validate_type_id(*expected, &self.world.type_graph)?;
            }
            CallableExplainRelationFailureV1::CycleDetected { sub, sup } => {
                validate_type_id(*sub, &self.world.type_graph)?;
                validate_type_id(*sup, &self.world.type_graph)?;
            }
            CallableExplainRelationFailureV1::DepthExceeded => {}
            CallableExplainRelationFailureV1::Custom { value } => {
                self.string(value)?;
            }
        }
        Ok(())
    }
    fn relationoutcome(&self, value: &CallableExplainRelationOutcomeV1) -> Result<(), ArtifactError> {
        match value {
            CallableExplainRelationOutcomeV1::Proven { notes } => {
                if notes.len() > self.limits.max_semantic_entries_per_family {
                    return Err(invalid("explanation vector limit exceeded"));
                }
                for value in notes {
                    self.string(value)?;
                }
            }
            CallableExplainRelationOutcomeV1::Refuted { value } => {
                self.relationfailure(value)?;
            }
            CallableExplainRelationOutcomeV1::DynamicBoundary { reason } => {
                self.string(reason)?;
            }
            CallableExplainRelationOutcomeV1::Blocked { value } => {
                validate_callable_block_reason(value, &self.world.type_graph, self.limits)?;
            }
            CallableExplainRelationOutcomeV1::Cancelled => {}
            CallableExplainRelationOutcomeV1::BudgetExceeded { value: _ } => {}
            CallableExplainRelationOutcomeV1::InternalFailure { value } => {
                self.string(value)?;
            }
        }
        Ok(())
    }
    fn consistency(&self, value: &CallableBindingConsistencyV1) -> Result<(), ArtifactError> {
        match value {
            CallableBindingConsistencyV1::Refuted { actual, expected, .. } => {
                validate_type_id(*actual, &self.world.type_graph)?;
                validate_type_id(*expected, &self.world.type_graph)?;
            }
            CallableBindingConsistencyV1::DynamicBoundary { reason } | CallableBindingConsistencyV1::InternalFailure { message: reason } => {
                self.string(reason)?
            }
            CallableBindingConsistencyV1::Blocked { reason } => validate_callable_block_reason(reason, &self.world.type_graph, self.limits)?,
            CallableBindingConsistencyV1::Assumed {
                basis:
                    CallableBindingAssumptionBasisV1::MissingValueEvidence {
                        reason: UnknownReasonV1::UnresolvedName { name },
                    },
            } => self.string(name)?,
            _ => {}
        }
        Ok(())
    }
}
pub(super) fn validate_explanations(
    analysis: &CallableAnalysisRecordV1,
    world: &SemanticWorldV3,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<(), ArtifactError> {
    let validate = ExplanationValidation { analysis, world, limits };
    let rows = &analysis.explanations;
    if rows.len() > limits.max_semantic_entries_per_family {
        return Err(invalid("explanation node limit exceeded"));
    }
    if let Some(id) = analysis.return_explanation {
        validate.local(id, rows.len())?;
    }
    let mut edge_count = 0usize;
    for row in rows {
        validate.explanationstep(&row.step)?;
        validate.derivationrule(&row.rule)?;
        validate.evidencestatus(&row.status)?;
        validate.evidenceorigin(&row.origin)?;
        if row.evidence.len() > limits.max_semantic_entries_per_family || row.parents.len() > limits.max_semantic_entries_per_family {
            return Err(invalid("explanation edge/evidence limit exceeded"));
        }
        for evidence in &row.evidence {
            validate.evidenceref(evidence)?;
        }
        for parent in &row.parents {
            validate.local(*parent, rows.len())?;
        }
        let extra = match &row.step {
            CallableExplanationStepV1::GenericConflict { constraints, .. } => constraints.len(),
            CallableExplanationStepV1::ProductComponent { .. } => 1,
            _ => 0,
        };
        edge_count = edge_count
            .checked_add(row.parents.len())
            .and_then(|n| n.checked_add(extra))
            .ok_or_else(|| invalid("explanation edge count overflow"))?;
    }
    if edge_count > limits.max_semantic_entries_per_family {
        return Err(invalid("explanation graph edge limit exceeded"));
    }
    let mut edges = allocations.vector::<(usize, usize)>(edge_count).map_err(ArtifactError::Allocation)?;
    for (index, row) in rows.iter().enumerate() {
        for parent in &row.parents {
            edges.push((index, *parent as usize));
        }
        match &row.step {
            CallableExplanationStepV1::GenericConflict { constraints, .. } => {
                for parent in constraints {
                    edges.push((index, *parent as usize));
                }
            }
            CallableExplanationStepV1::ProductComponent { source, .. } => edges.push((index, *source as usize)),
            _ => {}
        }
    }
    edges.sort_unstable();

    let mut indegree = allocations.vector::<usize>(rows.len()).map_err(ArtifactError::Allocation)?;
    indegree.resize(rows.len(), 0);
    let mut offsets = allocations.vector::<usize>(rows.len() + 1).map_err(ArtifactError::Allocation)?;
    offsets.resize(rows.len() + 1, 0);
    for &(source, target) in &edges {
        indegree[target] += 1;
        offsets[source + 1] += 1;
    }
    for i in 1..offsets.len() {
        offsets[i] += offsets[i - 1];
    }
    let mut queue = allocations.vector::<usize>(rows.len()).map_err(ArtifactError::Allocation)?;
    for (i, count) in indegree.iter().enumerate() {
        if *count == 0 {
            queue.push(i);
        }
    }
    let mut visited = 0;
    while visited < queue.len() {
        let node = queue[visited];
        visited += 1;
        for &(_, target) in &edges[offsets[node]..offsets[node + 1]] {
            indegree[target] -= 1;
            if indegree[target] == 0 {
                queue.push(target);
            }
        }
    }
    if visited != rows.len() {
        return Err(invalid("callable explanation graph contains a cycle"));
    }
    Ok(())
}
