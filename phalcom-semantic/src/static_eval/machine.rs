use super::{StaticEvalError, StaticEvalOutcome, StaticEvalTraceFrame};
use crate::checker::analysis::{
    BindingAnalysisIndex, ExpressionAnalysisIndex, SequenceConstructionPlan, SequenceExpansionResolution, SequenceLiteralFamily,
    SequenceLiteralResolution, SequenceLiteralResolutionIndex,
};
use crate::checker::associated::AssociatedResolutionIndex;
use crate::checker::proof_predicate::ProofPredicateGoal;
use crate::db::{CancellationToken, QueryBudget};
use crate::field_key::FieldKey;
use crate::generic_evidence::{
    ConformanceProofContext, GenericConformanceKey, GenericConformanceProof, GenericEquivalenceProof, GenericObligationProof,
    StructuralConformanceContext, prove_conformance_with_context, prove_constraint,
};
use crate::identity::{BindingId, DeclarationId, ExpressionId};
use crate::match_semantics::{BranchProofEnvironment, MatchResolutionIndex, PatternSiteKey, PatternSiteKind, PatternSiteResolutionIndex};
use crate::member_projection::{MemberDeclarationIdentity, MemberProjection, MemberProjectionArgument, MemberProjectionSubject};
use crate::static_constant::{CheckedStaticConstantInitializer, CheckedStaticExpression, StaticConstantTable};
use crate::static_definition::{CheckedStaticApplication, CheckedStaticBody, CheckedStaticConstruction, StaticApplicationId, StaticDefinitionTable};
use crate::static_domain::StaticValueDomain;
use crate::static_eval::application::{
    CanonicalStaticProofFact, StaticApplicationEnvironmentKey, StaticApplicationKey, StaticArgumentIdentity, StaticArgumentSummary,
    StaticCycle, StaticCycleFrame, StaticMemoResult, StaticSpecializationArgument, StaticSpecializationEnvironmentKey,
    StaticTrustedBranchContext,
};
use crate::static_eval::pattern::{StaticPatternMatchOutcome, match_pattern_against_value};
use crate::static_eval::residual::{
    StaticEvalTerm, StaticResidualApplication, StaticResidualData, StaticResidualParameter, StaticResidualRef, StaticResidualStore,
};
use crate::static_intrinsic::{StaticIntrinsicOperation, StaticOperationResolutionIndex};
use crate::static_value::{
    StaticDataValueId, StaticPackId, StaticRecordId, StaticSequenceId, StaticTupleId, StaticValueData, StaticValueRef, StaticValueStore,
};
use crate::traits::TraitRef;
use crate::types::id::{KindId, TypeId};
use crate::types::outcome::RelationOutcome;
use crate::types::parameter::{GenericConstraint, TypeTerm};
use crate::types::relation::{TypeHierarchy, check_subtype_bounded};
use crate::types::substitution::TypeSubstitution;
use num_bigint::BigInt;
use phalcom_ast::ast::{
    BinaryOp, BracketLiteralElement, BracketLiteralExpr, Expr, ForIndexBinding, ForStatement, GenericArgumentSyntax, IfElseBody,
    LexicalBlock, MatchArmBody, MatchExpr, Pattern, ProductLabel, RecordLiteralEntry, Statement, SymbolLiteralKind, TupleLiteralEntry,
    TypeAnnotationExpr,
};
use phalcom_common::range::SourceRange;
use std::collections::{BTreeMap, BTreeSet};

/// One argument aligned with the checked static definition parameter order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaticArgument {
    /// A canonical C3 static type/term argument for a kinded parameter.
    Kinded(TypeId),
    /// A semantic StaticValue for an ordinary value parameter.
    Value(StaticValueRef),
    /// A canonical semantic residual term.
    Residual(StaticResidualRef),
}

/// Unified body reference for a static definition, constant initializer, or
/// independently checked static expression.
#[derive(Clone, Copy)]
pub enum EvaluatorBody<'a> {
    Definition(&'a CheckedStaticBody),
    Constant(&'a CheckedStaticConstantInitializer),
    Expression(&'a CheckedStaticExpression),
}

pub enum EvaluatorRootWork<'a> {
    Expression(&'a Expr),
    Block(&'a LexicalBlock),
    TypeForm(&'a phalcom_ast::ast::TypeAnnotation),
}

impl<'a> EvaluatorBody<'a> {
    pub fn expressions(&self) -> &'a ExpressionAnalysisIndex {
        match self {
            Self::Definition(d) => &d.expressions,
            Self::Constant(c) => &c.expressions,
            Self::Expression(expression) => &expression.expressions,
        }
    }

    pub fn bindings(&self) -> &'a BindingAnalysisIndex {
        match self {
            Self::Definition(d) => &d.bindings,
            Self::Constant(c) => &c.bindings,
            Self::Expression(expression) => &expression.bindings,
        }
    }

    pub fn static_operations(&self) -> &'a StaticOperationResolutionIndex {
        match self {
            Self::Definition(d) => &d.static_operations,
            Self::Constant(c) => &c.static_operations,
            Self::Expression(expression) => &expression.static_operations,
        }
    }

    pub fn static_applications(&self) -> &'a [CheckedStaticApplication] {
        match self {
            Self::Definition(d) => &d.static_applications,
            Self::Constant(c) => &c.static_applications,
            Self::Expression(expression) => &expression.static_applications,
        }
    }

    pub fn static_constructions(&self) -> &'a BTreeMap<ExpressionId, CheckedStaticConstruction> {
        match self {
            Self::Definition(d) => &d.static_constructions,
            Self::Constant(c) => &c.static_constructions,
            Self::Expression(expression) => &expression.static_constructions,
        }
    }

    pub fn member_projections(&self) -> Option<&'a crate::static_definition::CheckedStaticMemberProjectionIndex> {
        match self {
            Self::Definition(d) => Some(&d.member_projections),
            Self::Constant(_) | Self::Expression(_) => None,
        }
    }

    pub fn sequence_literals(&self) -> &'a SequenceLiteralResolutionIndex {
        match self {
            Self::Definition(d) => &d.sequence_literals,
            Self::Constant(c) => &c.sequence_literals,
            Self::Expression(expression) => &expression.sequence_literals,
        }
    }

    #[allow(dead_code)]
    pub fn associated_resolutions(&self) -> &'a AssociatedResolutionIndex {
        match self {
            Self::Definition(d) => &d.associated_resolutions,
            Self::Constant(c) => &c.associated_resolutions,
            Self::Expression(expression) => &expression.associated_resolutions,
        }
    }

    #[allow(dead_code)]
    pub fn match_resolutions(&self) -> &'a MatchResolutionIndex {
        match self {
            Self::Definition(d) => &d.match_resolutions,
            Self::Constant(c) => &c.match_resolutions,
            Self::Expression(expression) => &expression.match_resolutions,
        }
    }

    #[allow(dead_code)]
    pub fn pattern_site_resolutions(&self) -> &'a PatternSiteResolutionIndex {
        match self {
            Self::Definition(d) => &d.pattern_site_resolutions,
            Self::Constant(c) => &c.pattern_site_resolutions,
            Self::Expression(expression) => &expression.pattern_site_resolutions,
        }
    }

    pub fn proof_predicates(&self) -> &'a crate::checker::proof_predicate::ProofPredicateResolutionIndex {
        match self {
            Self::Definition(d) => &d.proof_predicates,
            Self::Constant(c) => &c.proof_predicates,
            Self::Expression(expression) => &expression.proof_predicates,
        }
    }

    pub fn static_iteration_lanes(&self) -> Option<&'a crate::checker::static_iteration::StaticIterationLaneResolutionIndex> {
        match self {
            Self::Definition(d) => Some(&d.static_iteration_lanes),
            Self::Constant(_) | Self::Expression(_) => None,
        }
    }

    pub fn local_slots(&self) -> &'a BTreeMap<BindingId, u32> {
        match self {
            Self::Definition(d) => &d.local_slots,
            Self::Constant(c) => &c.local_slots,
            Self::Expression(expression) => &expression.local_slots,
        }
    }

    pub fn static_constant_bindings(&self) -> &'a BTreeMap<BindingId, phalcom_modules::linker::SymbolId> {
        match self {
            Self::Definition(d) => &d.static_constant_bindings,
            Self::Constant(c) => &c.static_constant_bindings,
            Self::Expression(expression) => &expression.static_constant_bindings,
        }
    }

    pub fn trace_frame(&self) -> StaticEvalTraceFrame {
        match self {
            Self::Definition(d) => StaticEvalTraceFrame::Definition(d.owner.0.clone()),
            Self::Constant(c) => StaticEvalTraceFrame::Constant(c.id.clone()),
            Self::Expression(expression) => StaticEvalTraceFrame::Expression(expression.root.range()),
        }
    }

    pub fn root_work(&self) -> EvaluatorRootWork<'a> {
        match self {
            Self::Definition(d) => match &d.root {
                crate::static_definition::StaticDefinitionBodyRepr::Expression(expression) => EvaluatorRootWork::Expression(expression),
                crate::static_definition::StaticDefinitionBodyRepr::Block(block) => EvaluatorRootWork::Block(block),
                crate::static_definition::StaticDefinitionBodyRepr::TypeForm(annotation) => EvaluatorRootWork::TypeForm(annotation),
            },
            Self::Constant(c) => EvaluatorRootWork::Expression(&c.root),
            Self::Expression(expression) => EvaluatorRootWork::Expression(&expression.root),
        }
    }

    pub fn result_as_kind(&self) -> Option<crate::types::id::KindId> {
        match self {
            Self::Definition(d) => d.result.as_kind(),
            Self::Constant(_) | Self::Expression(_) => None,
        }
    }

    pub fn is_value_result(&self) -> bool {
        match self {
            Self::Definition(d) => matches!(d.result, crate::static_domain::StaticValueDomain::ValueType(_)),
            Self::Constant(_) | Self::Expression(_) => true,
        }
    }

    pub fn contains_kinded_parameter(&self, types: &crate::types::store::TypeStore, form: crate::types::id::TypeId) -> bool {
        match self {
            Self::Definition(d) => d.kinded_parameters.values().any(|param| types.contains_type_parameter(form, *param)),
            Self::Constant(_) | Self::Expression(_) => false,
        }
    }

    pub fn kinded_parameter_index(&self, parameter: crate::types::id::TypeParameterId) -> Option<u32> {
        match self {
            Self::Definition(d) => d.kinded_parameters.iter().find(|(_, candidate)| **candidate == parameter).map(|(idx, _)| *idx),
            Self::Constant(_) | Self::Expression(_) => None,
        }
    }
}

/// Evaluates the currently supported, checker-authorized static expression subset.
///
/// Each guest expression is represented by a work item. The host call stack is
/// independent of guest expression nesting.
pub struct StaticEvaluator<'a> {
    body: EvaluatorBody<'a>,
    values: &'a mut StaticValueStore,
    budget: &'a mut QueryBudget,
    cancellation: &'a CancellationToken,
    locals: Vec<Option<StaticValueRef>>,
    parameter_values: BTreeMap<BindingId, StaticValueRef>,
    residual_values: BTreeMap<BindingId, StaticResidualRef>,
    kinded_arguments: BTreeMap<u32, TypeId>,
    residual_kinded_arguments: BTreeMap<u32, StaticResidualRef>,
    argument_error: Option<StaticEvalError>,
    expression_sites: BTreeMap<(usize, usize), Vec<ExpressionId>>,
    definitions: Option<&'a StaticDefinitionTable>,
    constants: Option<&'a StaticConstantTable>,
    canonical_types: Option<EvaluatorTypeStore<'a>>,
    semantic_tables: Option<(&'a crate::data_semantics::DataSemanticTable, &'a crate::enum_semantics::EnumSemanticTable)>,
    active_applications: Vec<StaticApplicationKey>,
    active_application_ranges: Vec<Option<SourceRange>>,
    active_application_index: BTreeMap<StaticApplicationKey, usize>,
    application_memo: BTreeMap<StaticApplicationKey, StaticMemoResult>,
    memo_hits: usize,
    current_application_key: Option<StaticApplicationKey>,
    root_result_domain: Option<StaticValueDomain>,
    root_application_key: Option<StaticApplicationKey>,
    inherited_specialization_environment: StaticSpecializationEnvironmentKey,
    inherited_trusted_branch_context: StaticTrustedBranchContext,
    active_branch_proofs: Vec<&'a BranchProofEnvironment>,
    hierarchy: Option<&'a dyn TypeHierarchy>,
    conformance: Option<(
        &'a crate::impls::ConformanceIndex,
        &'a BTreeMap<crate::identity::ImplId, std::sync::Arc<crate::impls::ConformanceWitnessPlan>>,
        &'a crate::traits::TraitSurfaceTable,
        &'a crate::declarations::DeclarationTypeTable,
    )>,
    saved_frames: Vec<SavedFrame<'a>>,
    pending_application_arguments: BTreeMap<StaticApplicationId, Vec<Option<StaticArgument>>>,
    completed_kinded_result: Option<TypeId>,
    completed_residual_result: Option<StaticResidualRef>,
    trace: Vec<StaticEvalTraceFrame>,
    residuals: Option<&'a mut StaticResidualStore>,
    owned_residuals: Option<StaticResidualStore>,
}

enum EvaluatorTypeStore<'a> {
    Snapshot(&'a crate::types::store::TypeStore),
    Session(&'a mut crate::types::store::TypeStore),
}

impl EvaluatorTypeStore<'_> {
    fn as_ref(&self) -> &crate::types::store::TypeStore {
        match self {
            Self::Snapshot(types) => types,
            Self::Session(types) => types,
        }
    }

    fn as_mut(&mut self) -> Option<&mut crate::types::store::TypeStore> {
        match self {
            Self::Snapshot(_) => None,
            Self::Session(types) => Some(types),
        }
    }
}

fn evaluator_type_store_ref<'s>(store: &'s Option<EvaluatorTypeStore<'_>>) -> Option<&'s crate::types::store::TypeStore> {
    store.as_ref().map(EvaluatorTypeStore::as_ref)
}

fn evaluator_type_store_mut<'s>(store: &'s mut Option<EvaluatorTypeStore<'_>>) -> Option<&'s mut crate::types::store::TypeStore> {
    store.as_mut().and_then(EvaluatorTypeStore::as_mut)
}

fn charge_variable_work(budget: &mut QueryBudget, cancellation: &CancellationToken, work_units: usize) -> Result<(), Stop> {
    let checkpoints = work_units.div_ceil(1024).max(1);
    for _ in 0..checkpoints {
        if cancellation.is_cancelled() {
            return Err(Stop::Cancelled);
        }
        budget.charge_step().map_err(Stop::Budget)?;
    }
    Ok(())
}

struct SavedFrame<'a> {
    body: EvaluatorBody<'a>,
    locals: Vec<Option<StaticValueRef>>,
    parameter_values: BTreeMap<BindingId, StaticValueRef>,
    residual_values: BTreeMap<BindingId, StaticResidualRef>,
    kinded_arguments: BTreeMap<u32, TypeId>,
    residual_kinded_arguments: BTreeMap<u32, StaticResidualRef>,
    expression_sites: BTreeMap<(usize, usize), Vec<ExpressionId>>,
    definition: Option<crate::identity::DeclarationId>,
    active_branch_proofs: Vec<&'a BranchProofEnvironment>,
    application_key: Option<StaticApplicationKey>,
    inherited_specialization_environment: StaticSpecializationEnvironmentKey,
    inherited_trusted_branch_context: StaticTrustedBranchContext,
}

#[derive(Clone, Copy)]
enum StatementContainer<'a> {
    Block(&'a LexicalBlock),
    ForBody(&'a ForStatement),
}

impl<'a> StatementContainer<'a> {
    fn statements(&self) -> &'a [Statement] {
        match self {
            Self::Block(b) => &b.body,
            Self::ForBody(f) => &f.body,
        }
    }
}

#[derive(Clone, Debug)]
pub enum StaticIterationCursor {
    Empty,
    Pack {
        pack_id: StaticPackId,
        pos_len: usize,
        total_len: usize,
        current_index: usize,
    },
    Tuple {
        tuple_id: StaticTupleId,
        len: usize,
        current_index: usize,
    },
    Sequence {
        seq_id: StaticSequenceId,
        len: usize,
        current_index: usize,
    },
    Record {
        rec_id: StaticRecordId,
        len: usize,
        current_index: usize,
    },
    Data {
        data_id: StaticDataValueId,
        len: usize,
        current_index: usize,
    },
    TypeTuple {
        ty: TypeId,
        len: usize,
        current_index: usize,
    },
    TypePack {
        ty: TypeId,
        len: usize,
        current_index: usize,
    },
    TypeElements {
        elements: Box<[TypeId]>,
        current_index: usize,
    },
}

impl StaticIterationCursor {
    pub fn is_done(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Pack { total_len, current_index, .. } => *current_index >= *total_len,
            Self::Tuple { len, current_index, .. }
            | Self::Sequence { len, current_index, .. }
            | Self::Record { len, current_index, .. }
            | Self::Data { len, current_index, .. }
            | Self::TypeTuple { len, current_index, .. }
            | Self::TypePack { len, current_index, .. } => *current_index >= *len,
            Self::TypeElements { elements, current_index } => *current_index >= elements.len(),
        }
    }

    pub fn current_loop_index(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Pack { current_index, .. }
            | Self::Tuple { current_index, .. }
            | Self::Sequence { current_index, .. }
            | Self::Record { current_index, .. }
            | Self::Data { current_index, .. }
            | Self::TypeTuple { current_index, .. }
            | Self::TypePack { current_index, .. }
            | Self::TypeElements { current_index, .. } => *current_index,
        }
    }

    pub fn advance(&mut self) {
        match self {
            Self::Empty => {}
            Self::Pack { current_index, .. }
            | Self::Tuple { current_index, .. }
            | Self::Sequence { current_index, .. }
            | Self::Record { current_index, .. }
            | Self::Data { current_index, .. }
            | Self::TypeTuple { current_index, .. }
            | Self::TypePack { current_index, .. }
            | Self::TypeElements { current_index, .. } => *current_index += 1,
        }
    }
}

enum Work<'a> {
    Evaluate(&'a Expr),
    EvaluateTypeForm(&'a phalcom_ast::ast::TypeAnnotation),
    EvaluateBlock(&'a LexicalBlock),
    FinishRoot,
    StatementNext(StatementContainer<'a>, usize, Option<StaticValueRef>),
    StatementAfter(StatementContainer<'a>, usize),
    FinishReturn,
    StoreLocal(BindingId, StatementContainer<'a>, usize),
    StoreAssignment(BindingId),
    AfterIf(&'a phalcom_ast::ast::IfExpr),
    AfterWhileCondition(&'a phalcom_ast::ast::WhileExpr),
    AfterWhileBody(&'a phalcom_ast::ast::WhileExpr),
    AfterMatchScrutinee(&'a MatchExpr, ExpressionId),
    TryMatchArm(&'a MatchExpr, ExpressionId, StaticValueRef, usize),
    FinishMatchArm(Vec<(usize, Option<StaticValueRef>)>),
    AfterForSource(StatementContainer<'a>, usize, &'a ForStatement),
    ForNext(StatementContainer<'a>, usize, &'a ForStatement, StaticIterationCursor),
    AfterForBody(
        StatementContainer<'a>,
        usize,
        &'a ForStatement,
        StaticIterationCursor,
        Vec<(usize, Option<StaticValueRef>)>,
    ),
    FinishTuple(usize),
    SequenceNext(&'a BracketLiteralExpr, &'a SequenceLiteralResolution, usize, Vec<StaticValueRef>),
    SequenceElementReady(&'a BracketLiteralExpr, &'a SequenceLiteralResolution, usize, Vec<StaticValueRef>),
    SequenceExpansionReady(
        &'a BracketLiteralExpr,
        &'a SequenceLiteralResolution,
        usize,
        Vec<StaticValueRef>,
        SourceRange,
        SequenceExpansionResolution,
    ),
    RecordNext(Vec<&'a phalcom_ast::ast::RecordLiteralField>, usize, Vec<(FieldKey, StaticValueRef)>),
    RecordKeyReady(Vec<&'a phalcom_ast::ast::RecordLiteralField>, usize, Vec<(FieldKey, StaticValueRef)>, &'a Expr),
    RecordValueReady(Vec<&'a phalcom_ast::ast::RecordLiteralField>, usize, Vec<(FieldKey, StaticValueRef)>, FieldKey),
    FinishStaticConstruction(ExpressionId, usize),
    PopTrace,
    ApplyBinary(ExpressionId),
    AfterLogicalLeft(&'a phalcom_ast::ast::BinaryExpr),
    FinishAnd,
    FinishOr,
    StartStaticApplication(&'a CheckedStaticApplication, &'a [GenericArgumentSyntax]),
    EvaluateBinding(ExpressionId),
    StoreStaticArgument(StaticApplicationId, u32),
    StoreStaticKindedArgument(StaticApplicationId, u32),
    EnterStaticApplication(&'a CheckedStaticApplication),
    FinishStaticApplication(StaticApplicationId, StaticApplicationKey, usize),
    AbortStaticApplication(StaticApplicationId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StaticEvalResult {
    Value(StaticValueRef),
    KindedTerm(TypeId),
    Residual(StaticResidualRef),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EvalControl {
    Return(StaticEvalResult),
    Break,
    Continue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ControlBoundary {
    Root,
    StaticApplication,
    Loop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ControlDisposition {
    ConsumeReturn(StaticEvalResult),
    ConsumeBreak,
    ConsumeContinue,
    Propagate(EvalControl),
    Invalid,
}

fn control_disposition(boundary: ControlBoundary, control: EvalControl) -> ControlDisposition {
    match (boundary, control) {
        (ControlBoundary::Root | ControlBoundary::StaticApplication, EvalControl::Return(result)) => ControlDisposition::ConsumeReturn(result),
        (ControlBoundary::Loop, EvalControl::Break) => ControlDisposition::ConsumeBreak,
        (ControlBoundary::Loop, EvalControl::Continue) => ControlDisposition::ConsumeContinue,
        (ControlBoundary::Loop, EvalControl::Return(result)) => ControlDisposition::Propagate(EvalControl::Return(result)),
        _ => ControlDisposition::Invalid,
    }
}

impl<'a> StaticEvaluator<'a> {
    pub fn new(
        body: &'a CheckedStaticBody,
        values: &'a mut StaticValueStore,
        budget: &'a mut QueryBudget,
        cancellation: &'a CancellationToken,
        arguments: &[StaticArgument],
    ) -> Self {
        let mut parameter_values = BTreeMap::new();
        let mut residual_values = BTreeMap::new();
        let mut kinded_arguments = BTreeMap::new();
        let mut residual_kinded_arguments = BTreeMap::new();
        let parameter_count = body
            .parameter_bindings
            .keys()
            .chain(body.kinded_parameters.keys())
            .max()
            .map_or(0, |index| *index as usize + 1);
        let mut argument_error = (arguments.len() != parameter_count).then_some(StaticEvalError::InvalidArgumentCount {
            expected: parameter_count,
            actual: arguments.len(),
        });
        for (index, binding) in body.parameter_bindings.iter() {
            match arguments.get(*index as usize) {
                Some(StaticArgument::Value(value)) => {
                    parameter_values.insert(*binding, *value);
                }
                Some(StaticArgument::Residual(residual)) => {
                    residual_values.insert(*binding, *residual);
                }
                _ => {
                    argument_error.get_or_insert(StaticEvalError::InvalidArgumentKind { index: *index });
                }
            };
        }
        for index in body.kinded_parameters.keys() {
            match arguments.get(*index as usize) {
                Some(StaticArgument::Kinded(ty)) => {
                    kinded_arguments.insert(*index, *ty);
                }
                Some(StaticArgument::Residual(residual)) => {
                    residual_kinded_arguments.insert(*index, *residual);
                }
                _ => {
                    argument_error.get_or_insert(StaticEvalError::InvalidArgumentKind { index: *index });
                }
            }
        }
        let mut expression_sites: BTreeMap<(usize, usize), Vec<ExpressionId>> = BTreeMap::new();
        for (id, expression) in body.expressions.iter() {
            expression_sites.entry((expression.range.start, expression.range.end)).or_default().push(*id);
        }
        let locals = vec![None; body.local_slots.len()];
        let root_key = StaticApplicationKey::new(
            body.owner.0.clone(),
            arguments.iter().copied(),
            StaticApplicationEnvironmentKey::default(),
        );
        let mut active_application_index = BTreeMap::new();
        active_application_index.insert(root_key.clone(), 0);
        Self {
            body: EvaluatorBody::Definition(body),
            values,
            budget,
            cancellation,
            locals,
            parameter_values,
            residual_values,
            kinded_arguments,
            residual_kinded_arguments,
            argument_error,
            expression_sites,
            definitions: None,
            constants: None,
            canonical_types: None,
            semantic_tables: None,
            active_applications: vec![root_key.clone()],
            active_application_ranges: vec![None],
            active_application_index,
            application_memo: BTreeMap::new(),
            memo_hits: 0,
            current_application_key: Some(root_key.clone()),
            root_result_domain: Some(body.result),
            root_application_key: Some(root_key.clone()),
            inherited_specialization_environment: StaticSpecializationEnvironmentKey::default(),
            inherited_trusted_branch_context: StaticTrustedBranchContext::default(),
            active_branch_proofs: Vec::new(),
            hierarchy: None,
            conformance: None,
            saved_frames: Vec::new(),
            pending_application_arguments: BTreeMap::new(),
            completed_kinded_result: None,
            completed_residual_result: None,
            trace: Vec::new(),
            residuals: None,
            owned_residuals: None,
        }
    }

    /// Creates an evaluator for a module-owned static constant initializer.
    pub fn for_constant(
        body: &'a CheckedStaticConstantInitializer,
        values: &'a mut StaticValueStore,
        budget: &'a mut QueryBudget,
        cancellation: &'a CancellationToken,
    ) -> Self {
        let mut expression_sites: BTreeMap<(usize, usize), Vec<ExpressionId>> = BTreeMap::new();
        for (id, expression) in body.expressions.iter() {
            expression_sites.entry((expression.range.start, expression.range.end)).or_default().push(*id);
        }
        let locals = vec![None; body.local_slots.len()];
        Self {
            body: EvaluatorBody::Constant(body),
            values,
            budget,
            cancellation,
            locals,
            parameter_values: BTreeMap::new(),
            residual_values: BTreeMap::new(),
            kinded_arguments: BTreeMap::new(),
            residual_kinded_arguments: BTreeMap::new(),
            argument_error: None,
            expression_sites,
            definitions: None,
            constants: None,
            canonical_types: None,
            semantic_tables: None,
            active_applications: Vec::new(),
            active_application_ranges: Vec::new(),
            active_application_index: BTreeMap::new(),
            application_memo: BTreeMap::new(),
            memo_hits: 0,
            current_application_key: None,
            root_result_domain: None,
            root_application_key: None,
            inherited_specialization_environment: StaticSpecializationEnvironmentKey::default(),
            inherited_trusted_branch_context: StaticTrustedBranchContext::default(),
            active_branch_proofs: Vec::new(),
            hierarchy: None,
            conformance: None,
            saved_frames: Vec::new(),
            pending_application_arguments: BTreeMap::new(),
            completed_kinded_result: None,
            completed_residual_result: None,
            trace: Vec::new(),
            residuals: None,
            owned_residuals: None,
        }
    }

    /// Creates an evaluator for one ordinary-checker-approved static
    /// expression, such as a computed Record key.
    pub fn for_checked_expression(
        body: &'a CheckedStaticExpression,
        values: &'a mut StaticValueStore,
        budget: &'a mut QueryBudget,
        cancellation: &'a CancellationToken,
    ) -> Self {
        let mut expression_sites: BTreeMap<(usize, usize), Vec<ExpressionId>> = BTreeMap::new();
        for (id, expression) in body.expressions.iter() {
            expression_sites.entry((expression.range.start, expression.range.end)).or_default().push(*id);
        }
        let locals = vec![None; body.local_slots.len()];
        Self {
            body: EvaluatorBody::Expression(body),
            values,
            budget,
            cancellation,
            locals,
            parameter_values: BTreeMap::new(),
            residual_values: BTreeMap::new(),
            kinded_arguments: BTreeMap::new(),
            residual_kinded_arguments: BTreeMap::new(),
            argument_error: None,
            expression_sites,
            definitions: None,
            constants: None,
            canonical_types: None,
            semantic_tables: None,
            active_applications: Vec::new(),
            active_application_ranges: Vec::new(),
            active_application_index: BTreeMap::new(),
            application_memo: BTreeMap::new(),
            memo_hits: 0,
            current_application_key: None,
            root_result_domain: None,
            root_application_key: None,
            inherited_specialization_environment: StaticSpecializationEnvironmentKey::default(),
            inherited_trusted_branch_context: StaticTrustedBranchContext::default(),
            active_branch_proofs: Vec::new(),
            hierarchy: None,
            conformance: None,
            saved_frames: Vec::new(),
            pending_application_arguments: BTreeMap::new(),
            completed_kinded_result: None,
            completed_residual_result: None,
            trace: Vec::new(),
            residuals: None,
            owned_residuals: None,
        }
    }

    /// Attaches an ambient specialization environment for residual evaluation.
    pub fn with_specialization_environment(self, environment: StaticSpecializationEnvironmentKey) -> Self {
        self.with_specialization_context(environment, StaticTrustedBranchContext::default())
    }

    pub(crate) fn with_specialization_context(
        mut self,
        environment: StaticSpecializationEnvironmentKey,
        trusted_context: StaticTrustedBranchContext,
    ) -> Self {
        let identity = environment
            .application_identity_projection()
            .with_trusted_branch_context(trusted_context.clone());
        if let Some(mut key) = self.current_application_key.take() {
            let old_key = key.clone();
            key.environment = identity.clone();
            if let Some(index) = self.active_application_index.remove(&old_key) {
                if let Some(active) = self.active_applications.get_mut(index) {
                    *active = key.clone();
                }
                self.active_application_index.insert(key.clone(), index);
            }
            self.current_application_key = Some(key);
        }
        if let Some(key) = &mut self.root_application_key {
            key.environment = identity;
        }
        self.inherited_specialization_environment = environment;
        self.inherited_trusted_branch_context = trusted_context;
        self
    }

    /// Supplies the snapshot's evaluated static constants for constant-read evaluation.
    pub fn with_static_constants(mut self, constants: &'a StaticConstantTable) -> Self {
        self.constants = Some(constants);
        self
    }

    /// Supplies the snapshot's checked static definitions for exact-target
    /// application evaluation. The default constructor remains useful for a
    /// single body that contains no static applications.
    pub fn with_static_definitions(mut self, definitions: &'a StaticDefinitionTable) -> Self {
        self.definitions = Some(definitions);
        self
    }

    /// Supplies canonical snapshot tables used by exact checked constructor
    /// products. The evaluator never resolves constructor names itself.
    pub fn with_semantic_tables(
        mut self,
        types: &'a crate::types::store::TypeStore,
        data: &'a crate::data_semantics::DataSemanticTable,
        enums: &'a crate::enum_semantics::EnumSemanticTable,
    ) -> Self {
        self.canonical_types = Some(EvaluatorTypeStore::Snapshot(types));
        self.semantic_tables = Some((data, enums));
        self
    }

    /// Supplies the mutable session-owned canonical TypeStore and semantic
    /// constructor tables for workspace evaluation. The caller must obtain the
    /// store through the session's copy-on-write boundary before evaluation.
    pub fn with_semantic_tables_mut(
        mut self,
        types: &'a mut crate::types::store::TypeStore,
        data: &'a crate::data_semantics::DataSemanticTable,
        enums: &'a crate::enum_semantics::EnumSemanticTable,
    ) -> Self {
        self.canonical_types = Some(EvaluatorTypeStore::Session(types));
        self.semantic_tables = Some((data, enums));
        self
    }

    /// Supplies the immutable canonical TypeStore needed for checked kinded
    /// application arguments and TypeForm results.
    pub fn with_type_store(mut self, types: &'a crate::types::store::TypeStore) -> Self {
        self.canonical_types = Some(EvaluatorTypeStore::Snapshot(types));
        self
    }

    /// Supplies the active session's canonical TypeStore for checked kinded
    /// application arguments and recursively specialized TypeForm results.
    /// Use only while holding the session's exclusive mutable/COW borrow.
    pub fn with_type_store_mut(mut self, types: &'a mut crate::types::store::TypeStore) -> Self {
        self.canonical_types = Some(EvaluatorTypeStore::Session(types));
        self
    }

    /// Supplies the type hierarchy for subtype relations and proof predicates.
    pub fn with_hierarchy(mut self, hierarchy: &'a dyn TypeHierarchy) -> Self {
        self.hierarchy = Some(hierarchy);
        self
    }

    /// Supplies conformance indexes and trait declarations for user-defined Conforms proof predicates.
    pub fn with_conformance(
        mut self,
        conformance_index: &'a crate::impls::ConformanceIndex,
        witness_plans: &'a BTreeMap<crate::identity::ImplId, std::sync::Arc<crate::impls::ConformanceWitnessPlan>>,
        trait_surfaces: &'a crate::traits::TraitSurfaceTable,
        declarations: &'a crate::declarations::DeclarationTypeTable,
    ) -> Self {
        self.conformance = Some((conformance_index, witness_plans, trait_surfaces, declarations));
        self
    }

    /// Supplies an external residual store for accumulating residual nodes.
    pub fn with_residuals(mut self, residuals: &'a mut StaticResidualStore) -> Self {
        self.residuals = Some(residuals);
        self
    }

    /// Returns a mutable reference to the residual store, creating an internal one if needed.
    pub fn residuals_mut(&mut self) -> &mut StaticResidualStore {
        if let Some(ref mut r) = self.residuals {
            r
        } else {
            self.owned_residuals.get_or_insert_with(StaticResidualStore::new)
        }
    }

    /// Returns an immutable reference to the active residual store, if allocated.
    pub fn residuals(&self) -> Option<&StaticResidualStore> {
        if let Some(ref r) = self.residuals {
            Some(r)
        } else {
            self.owned_residuals.as_ref()
        }
    }

    pub fn take_residuals(&mut self) -> Option<StaticResidualStore> {
        self.owned_residuals.take()
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    pub(crate) fn charge_step(&mut self) -> Result<(), Stop> {
        self.budget.charge_step().map_err(Stop::Budget)
    }

    pub(crate) fn intern_bool(&mut self, value: bool) -> StaticValueRef {
        self.values.intern_bool(value)
    }

    pub(crate) fn inherited_specialization_environment(&self) -> &StaticSpecializationEnvironmentKey {
        &self.inherited_specialization_environment
    }

    pub(crate) fn is_open_type(&self, ty: TypeId) -> Result<bool, Stop> {
        let types = evaluator_type_store_ref(&self.canonical_types)
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        Ok(self.is_open_type_term(ty, types))
    }

    pub(crate) fn compose_specialization_environment(
        &mut self,
        base: &StaticSpecializationEnvironmentKey,
        supplied: &StaticSpecializationEnvironmentKey,
    ) -> Result<StaticSpecializationEnvironmentKey, Stop> {
        let base_formal_bindings = self.formal_type_bindings(base)?;
        let supplied_formal_bindings = self.formal_type_bindings(supplied)?;
        let types = evaluator_type_store_mut(&mut self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        for (parameter, ty, expected_kind) in base_formal_bindings.iter().chain(&supplied_formal_bindings) {
            if ty.index() >= types.len() || types.type_parameter(*parameter).kind != *expected_kind || types.kind_of(*ty) != *expected_kind {
                return Err(Stop::from(
                    crate::static_eval::application::StaticSpecializationEnvironmentError::ArgumentDomainMismatch,
                ));
            }
        }
        let base = base.with_additional_type_bindings(base_formal_bindings.into_iter().map(|(parameter, ty, _)| (parameter, ty)));
        let supplied = supplied.with_additional_type_bindings(supplied_formal_bindings.into_iter().map(|(parameter, ty, _)| (parameter, ty)));
        let composed = base.compose_with_supplied(&supplied, types, self.budget, self.cancellation).map_err(Stop::from)?;
        let substitution = composed.substitution_closure(types).map_err(Stop::from)?;
        let trusted = self
            .inherited_trusted_branch_context
            .substitute(&substitution, types)
            .map_err(Stop::from)?;
        self.inherited_trusted_branch_context = self.validate_trusted_branch_context(trusted, &composed)?;
        Ok(composed)
    }

    fn validate_trusted_branch_context(
        &mut self,
        context: StaticTrustedBranchContext,
        environment: &StaticSpecializationEnvironmentKey,
    ) -> Result<StaticTrustedBranchContext, Stop> {
        let old_environment = std::mem::replace(&mut self.inherited_specialization_environment, environment.clone());
        let old_context = std::mem::replace(&mut self.inherited_trusted_branch_context, StaticTrustedBranchContext::default());
        let old_active_proofs = std::mem::take(&mut self.active_branch_proofs);
        let result = (|| {
            let Some(mut types) = evaluator_type_store_ref(&self.canonical_types).cloned() else {
                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
            };
            let mut retained = Vec::new();
            for fact in context.constraints() {
                match fact {
                    CanonicalStaticProofFact::Equal(left, right) => {
                        let left = self.resolve_type_term(*left, &types);
                        let right = self.resolve_type_term(*right, &types);
                        if self.is_open_type_term(left, &types) || self.is_open_type_term(right, &types) {
                            retained.push(fact.clone());
                        } else if self.prove_closed_equivalence(left, right, &mut types)? {
                            retained.push(fact.clone());
                        }
                    }
                    CanonicalStaticProofFact::Subtype { lower, upper } => {
                        let lower = self.resolve_type_term(*lower, &types);
                        let upper = self.resolve_type_term(*upper, &types);
                        if self.is_open_type_term(lower, &types) || self.is_open_type_term(upper, &types) {
                            retained.push(fact.clone());
                            continue;
                        }
                        let Some(hierarchy) = self.hierarchy else {
                            continue;
                        };
                        let mut local_types = types.clone();
                        match check_subtype_bounded(&mut local_types, hierarchy, lower, upper, self.budget, self.cancellation) {
                            RelationOutcome::Proven { .. } => retained.push(fact.clone()),
                            RelationOutcome::Cancelled => return Err(Stop::Cancelled),
                            RelationOutcome::BudgetExceeded(report) => return Err(Stop::Budget(report)),
                            RelationOutcome::Blocked(_) | RelationOutcome::InternalFailure(_) => {}
                            RelationOutcome::Refuted(_) | RelationOutcome::DynamicBoundary(_) => {}
                        }
                    }
                    // Conformance refinements are currently not produced by
                    // branch matching. Keep them only while they remain open.
                    CanonicalStaticProofFact::Conforms { key } => {
                        let open_subject = match key.subject {
                            TypeTerm::Canonical(ty) => self.is_open_type_term(self.resolve_type_term(ty, &types), &types),
                            TypeTerm::Infer(_) => false,
                            TypeTerm::SelfType(_) => true,
                        };
                        let open_argument = key
                            .trait_ref
                            .arguments
                            .iter()
                            .any(|ty| self.is_open_type_term(self.resolve_type_term(*ty, &types), &types));
                        if open_subject || open_argument {
                            retained.push(fact.clone());
                        }
                    }
                }
            }
            Ok(StaticTrustedBranchContext::from_parts(context.type_bindings().to_vec(), retained))
        })();
        self.inherited_specialization_environment = old_environment;
        self.inherited_trusted_branch_context = old_context;
        self.active_branch_proofs = old_active_proofs;
        result
    }

    fn prove_closed_equivalence(&mut self, left: TypeId, right: TypeId, types: &mut crate::types::store::TypeStore) -> Result<bool, Stop> {
        let evidence = self.active_generic_evidence(types);
        let (conformance_index, witness_plans, trait_surfaces, declarations) = match self.conformance {
            Some((index, plans, surfaces, declarations)) => (Some(index), Some(plans), Some(surfaces), Some(declarations)),
            None => (None, None, None, None),
        };
        let mut proof_context = ConformanceProofContext::new(
            Some(&evidence),
            conformance_index,
            witness_plans,
            trait_surfaces,
            declarations,
            self.hierarchy,
            Some(types),
        )
        .with_budget_and_cancellation(*self.budget, self.cancellation.clone());
        let proof = prove_constraint(
            &GenericConstraint::Equivalent {
                left: TypeTerm::from(left),
                right: TypeTerm::from(right),
            },
            &TypeSubstitution::new(),
            &mut proof_context,
        );
        *self.budget = proof_context.budget;
        match proof {
            GenericObligationProof::Equivalent(GenericEquivalenceProof::Proven) => Ok(true),
            GenericObligationProof::Equivalent(GenericEquivalenceProof::NotProven | GenericEquivalenceProof::Ambiguous) => Ok(false),
            GenericObligationProof::Equivalent(GenericEquivalenceProof::Blocked(reason)) => Err(Stop::Blocked(reason)),
            GenericObligationProof::Equivalent(GenericEquivalenceProof::Dynamic(obligation)) => Err(Stop::Dynamic(obligation)),
            GenericObligationProof::Equivalent(GenericEquivalenceProof::Recursive) => {
                Err(Stop::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint))
            }
            GenericObligationProof::Equivalent(GenericEquivalenceProof::Cancelled) => Err(Stop::Cancelled),
            GenericObligationProof::Equivalent(GenericEquivalenceProof::BudgetExceeded(report)) => Err(Stop::Budget(report)),
            GenericObligationProof::Equivalent(GenericEquivalenceProof::InternalFailure(_)) => {
                Err(Stop::invalid(StaticEvalError::UnsupportedExpression))
            }
            _ => Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
        }
    }

    fn formal_type_bindings(&self, environment: &StaticSpecializationEnvironmentKey) -> Result<Vec<(crate::types::id::TypeParameterId, TypeId, KindId)>, Stop> {
        let mut bindings = Vec::new();
        for (formal, argument) in environment.argument_bindings() {
            if let Some(product) = self.definitions.and_then(|definitions| definitions.get(&formal.owner)) {
                let Some(parameter) = product.definition.signature.parameters.get(formal.index as usize) else {
                    return Err(Stop::from(
                        crate::static_eval::application::StaticSpecializationEnvironmentError::MalformedEnvironment,
                    ));
                };
                if parameter.index != formal.index || parameter.domain != formal.domain {
                    return Err(Stop::from(
                        crate::static_eval::application::StaticSpecializationEnvironmentError::ArgumentDomainMismatch,
                    ));
                }
                match (parameter.domain, parameter.type_parameter_id, *argument) {
                    (StaticValueDomain::Kind(kind), Some(type_parameter), StaticSpecializationArgument::Kinded(ty)) => {
                        bindings.push((type_parameter, ty, kind));
                    }
                    (StaticValueDomain::ValueType(_), None, StaticSpecializationArgument::Value(_)) => {}
                    _ => {
                        return Err(Stop::from(
                            crate::static_eval::application::StaticSpecializationEnvironmentError::ArgumentDomainMismatch,
                        ));
                    }
                }
                continue;
            }

            let EvaluatorBody::Definition(body) = self.body else {
                continue;
            };
            if body.owner.0 != formal.owner {
                continue;
            }
            match (body.kinded_parameters.get(&formal.index), formal.domain, *argument) {
                (Some(parameter), StaticValueDomain::Kind(kind), StaticSpecializationArgument::Kinded(ty)) => {
                    let types = evaluator_type_store_ref(&self.canonical_types)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    if types.type_parameter(*parameter).kind != kind {
                        return Err(Stop::from(
                            crate::static_eval::application::StaticSpecializationEnvironmentError::ArgumentDomainMismatch,
                        ));
                    }
                    bindings.push((*parameter, ty, kind));
                }
                (None, StaticValueDomain::ValueType(_), StaticSpecializationArgument::Value(_)) => {}
                _ => {
                    return Err(Stop::from(
                        crate::static_eval::application::StaticSpecializationEnvironmentError::ArgumentDomainMismatch,
                    ));
                }
            }
        }
        Ok(bindings)
    }

    pub(crate) fn substitute_specialization_type(
        &mut self,
        ty: TypeId,
        environment: &StaticSpecializationEnvironmentKey,
    ) -> Result<TypeId, Stop> {
        let types = evaluator_type_store_mut(&mut self.canonical_types)
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        let subst = environment.substitution_closure(types)?;
        subst
            .try_apply(types, ty)
            .map_err(|_| Stop::from(crate::static_eval::application::StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution))
    }

    pub(crate) fn substitute_specialization_domain(
        &mut self,
        domain: StaticValueDomain,
        environment: &StaticSpecializationEnvironmentKey,
    ) -> Result<(StaticValueDomain, bool), Stop> {
        let specialized = match domain {
            StaticValueDomain::Kind(kind) => StaticValueDomain::Kind(kind),
            StaticValueDomain::ValueType(ty) => {
                let ty = self.substitute_specialization_type(ty, environment)?;
                let types = evaluator_type_store_ref(&self.canonical_types)
                    .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                if ty.index() >= types.len() || types.kind_of(ty) != KindId::TYPE {
                    return Err(Stop::from(
                        crate::static_eval::application::StaticSpecializationEnvironmentError::ArgumentDomainMismatch,
                    ));
                }
                StaticValueDomain::ValueType(ty)
            }
        };
        Ok((specialized, specialized != domain))
    }

    pub(crate) fn validate_specialization_argument(
        &mut self,
        domain: StaticValueDomain,
        argument: StaticSpecializationArgument,
    ) -> Result<StaticEvalTerm, Stop> {
        match (domain, argument) {
            (StaticValueDomain::Kind(expected), StaticSpecializationArgument::Kinded(ty)) => {
                let types = evaluator_type_store_ref(&self.canonical_types)
                    .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                if ty.index() >= types.len() || types.kind_of(ty) != expected {
                    return Err(Stop::from(
                        crate::static_eval::application::StaticSpecializationEnvironmentError::ArgumentDomainMismatch,
                    ));
                }
                Ok(StaticEvalTerm::Kinded(ty))
            }
            (StaticValueDomain::ValueType(_), StaticSpecializationArgument::Value(value)) => {
                if value.store_id() != self.values.id() {
                    return Err(Stop::from(
                        crate::static_eval::application::StaticSpecializationEnvironmentError::ForeignStaticValueStore,
                    ));
                }
                self.values.get(value).map_err(Stop::from)?;
                if let StaticValueDomain::ValueType(expected) = domain {
                    self.validate_specialization_value_type(value, expected)?;
                }
                Ok(StaticEvalTerm::Value(value))
            }
            _ => Err(Stop::from(
                crate::static_eval::application::StaticSpecializationEnvironmentError::ArgumentDomainMismatch,
            )),
        }
    }

    fn validate_specialization_value_type(&mut self, value: StaticValueRef, expected: TypeId) -> Result<(), Stop> {
        let types = evaluator_type_store_mut(&mut self.canonical_types)
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        let actual = if let Some((data, enums)) = self.semantic_tables {
            self.values.static_value_type(value, types, data, enums).map_err(Stop::from)?
        } else {
            match self.values.domain_of(value, types).map_err(Stop::from)? {
                StaticValueDomain::ValueType(actual) => actual,
                StaticValueDomain::Kind(_) => {
                    return Err(Stop::from(
                        crate::static_eval::application::StaticSpecializationEnvironmentError::ArgumentDomainMismatch,
                    ));
                }
            }
        };
        if actual == expected {
            return Ok(());
        }
        let Some(hierarchy) = self.hierarchy else {
            return Err(Stop::from(
                crate::static_eval::application::StaticSpecializationEnvironmentError::ArgumentDomainMismatch,
            ));
        };
        let actual = crate::types::evidence::TypeKnowledge::established(
            actual,
            crate::types::evidence::EvidenceOrigin::ContextualDerivation,
        );
        match crate::types::relation::check_knowledge_against_type_bounded(
            types,
            hierarchy,
            &actual,
            expected,
            self.budget,
            self.cancellation,
        ) {
            RelationOutcome::Proven { .. } => Ok(()),
            RelationOutcome::Refuted(_) => Err(Stop::from(
                crate::static_eval::application::StaticSpecializationEnvironmentError::ArgumentDomainMismatch,
            )),
            RelationOutcome::Blocked(reason) => Err(Stop::Blocked(reason)),
            RelationOutcome::DynamicBoundary(obligation) => Err(Stop::Dynamic(obligation)),
            RelationOutcome::Cancelled => Err(Stop::Cancelled),
            RelationOutcome::BudgetExceeded(report) => Err(Stop::Budget(report)),
            RelationOutcome::InternalFailure(_) => Err(Stop::from(
                crate::static_eval::application::StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution,
            )),
        }
    }

    pub(crate) fn substitute_proof_goal(
        &mut self,
        goal: &ProofPredicateGoal,
        env: &StaticSpecializationEnvironmentKey,
    ) -> Result<(ProofPredicateGoal, bool), Stop> {
        let types = evaluator_type_store_mut(&mut self.canonical_types)
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        let subst = env.substitution_closure(types)?;
        crate::static_eval::specialize::substitute_proof_goal(goal, &subst, types)
            .map_err(Stop::from)
    }

    pub(crate) fn substitute_member_projection(
        &mut self,
        projection: &MemberProjection,
        env: &StaticSpecializationEnvironmentKey,
    ) -> Result<(MemberProjection, bool), Stop> {
        let types = evaluator_type_store_mut(&mut self.canonical_types)
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        let subst = env.substitution_closure(types)?;
        crate::static_eval::specialize::substitute_member_projection(projection, &subst, types)
            .map_err(Stop::from)
    }

    pub(crate) fn evaluate_proof_predicate_with_environment(
        &mut self,
        goal: &ProofPredicateGoal,
        result_type: TypeId,
        environment: StaticSpecializationEnvironmentKey,
    ) -> Result<bool, Stop> {
        let prev_env = std::mem::replace(&mut self.inherited_specialization_environment, environment);
        let result = self.evaluate_proof_predicate(goal, result_type);
        self.inherited_specialization_environment = prev_env;
        result
    }

    pub(crate) fn project_member_with_environment(
        &mut self,
        projection: &MemberProjection,
        environment: StaticSpecializationEnvironmentKey,
    ) -> Result<crate::member_projection::StaticMemberProjectionOutcome, Stop> {
        let prev_env = std::mem::replace(&mut self.inherited_specialization_environment, environment);
        let result = self.project_checked_member(projection);
        self.inherited_specialization_environment = prev_env;
        result
    }

    pub(crate) fn intern_applied_residual(
        &mut self,
        application: StaticResidualApplication,
        result_domain: StaticValueDomain,
    ) -> Result<StaticResidualRef, Stop> {
        let trusted_context = self.capture_trusted_branch_context();
        self.intern_applied_residual_with_context(application, result_domain, trusted_context)
    }

    fn intern_applied_residual_with_context(
        &mut self,
        application: StaticResidualApplication,
        result_domain: StaticValueDomain,
        trusted_context: StaticTrustedBranchContext,
    ) -> Result<StaticResidualRef, Stop> {
        let budget = &mut *self.budget;
        let store = if let Some(ref mut residuals) = self.residuals {
            residuals
        } else {
            self.owned_residuals.get_or_insert_with(StaticResidualStore::new)
        };
        store
            .intern_with_trusted_context(StaticResidualData::Apply { application, result_domain }, trusted_context, budget)
            .map_err(Stop::from)
    }

    pub(crate) fn intern_parameter_residual_node(
        &mut self,
        parameter: StaticResidualParameter,
    ) -> Result<StaticResidualRef, Stop> {
        let budget = &mut *self.budget;
        let store = if let Some(ref mut residuals) = self.residuals {
            residuals
        } else {
            self.owned_residuals.get_or_insert_with(StaticResidualStore::new)
        };
        store.parameter(parameter, budget).map_err(Stop::from)
    }

    pub(crate) fn intern_proof_residual_node(
        &mut self,
        goal: ProofPredicateGoal,
        result_type: TypeId,
        environment: StaticSpecializationEnvironmentKey,
    ) -> Result<StaticResidualRef, Stop> {
        let trusted_context = self.capture_trusted_branch_context();
        let budget = &mut *self.budget;
        let store = if let Some(ref mut residuals) = self.residuals {
            residuals
        } else {
            self.owned_residuals.get_or_insert_with(StaticResidualStore::new)
        };
        store
            .intern_with_trusted_context(StaticResidualData::Proof { goal, result_type, environment }, trusted_context, budget)
            .map_err(Stop::from)
    }

    pub(crate) fn intern_member_projection_residual_node(
        &mut self,
        projection: MemberProjection,
        result_domain: StaticValueDomain,
        environment: StaticSpecializationEnvironmentKey,
    ) -> Result<StaticResidualRef, Stop> {
        let trusted_context = self.capture_trusted_branch_context();
        let budget = &mut *self.budget;
        let store = if let Some(ref mut residuals) = self.residuals {
            residuals
        } else {
            self.owned_residuals.get_or_insert_with(StaticResidualStore::new)
        };
        store
            .intern_with_trusted_context(
                StaticResidualData::MemberProjection { projection, result_domain, environment },
                trusted_context,
                budget,
            )
            .map_err(Stop::from)
    }

    pub(crate) fn evaluate_specialized_application(
        &mut self,
        target: DeclarationId,
        args: &[StaticArgument],
        environment: StaticSpecializationEnvironmentKey,
    ) -> Result<StaticEvalTerm, Stop> {
        let definitions = self.definitions.ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        let product = definitions
            .get(&target)
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        let target_body = product
            .checked_body
            .as_ref()
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;

        self.validate_application_arguments(&product.definition.signature, args)?;

        let trusted_context = self.capture_trusted_branch_context();
        let application_env = environment
            .application_identity_projection()
            .with_trusted_branch_context(trusted_context.clone());
        let key = StaticApplicationKey::new(target.clone(), args.iter().copied(), application_env);

        if let Some(&cycle_start) = self.active_application_index.get(&key) {
            return Err(Stop::Recursive(self.make_cycle(&key, None, cycle_start)));
        }
        if let Some(memoized) = self.application_memo.get(&key).cloned() {
            self.memo_hits = self.memo_hits.saturating_add(1);
            return match memoized {
                StaticMemoResult::Reduced(term) => Ok(term),
                StaticMemoResult::Symbolic(r) => Ok(StaticEvalTerm::Residual(r)),
            };
        }

        let mut sub_evaluator = StaticEvaluator::new(
            target_body,
            self.values,
            self.budget,
            self.cancellation,
            args,
        )
        .with_specialization_context(environment, trusted_context);

        if let Some(defs) = self.definitions {
            sub_evaluator = sub_evaluator.with_static_definitions(defs);
        }
        if let Some(consts) = self.constants {
            sub_evaluator = sub_evaluator.with_static_constants(consts);
        }
        match &mut self.canonical_types {
            Some(EvaluatorTypeStore::Snapshot(s)) => {
                sub_evaluator = sub_evaluator.with_type_store(*s);
            }
            Some(EvaluatorTypeStore::Session(s)) => {
                sub_evaluator = sub_evaluator.with_type_store_mut(*s);
            }
            None => {}
        }
        if let Some(tables) = self.semantic_tables {
            sub_evaluator.semantic_tables = Some(tables);
        }
        if let Some(h) = self.hierarchy {
            sub_evaluator = sub_evaluator.with_hierarchy(h);
        }
        if let Some((ci, wp, ts, d)) = self.conformance {
            sub_evaluator = sub_evaluator.with_conformance(ci, wp, ts, d);
        }
        let res_store = if let Some(ref mut residuals) = self.residuals {
            residuals
        } else {
            self.owned_residuals.get_or_insert_with(StaticResidualStore::new)
        };
        sub_evaluator = sub_evaluator.with_residuals(res_store);

        sub_evaluator.active_applications = self.active_applications.clone();
        sub_evaluator.active_application_index = self.active_application_index.clone();
        sub_evaluator.application_memo = self.application_memo.clone();
        let active_idx = sub_evaluator.active_applications.len();
        sub_evaluator.active_applications.push(key.clone());
        sub_evaluator.active_application_index.insert(key.clone(), active_idx);

        let outcome = sub_evaluator.evaluate();

        self.application_memo = sub_evaluator.application_memo;
        self.memo_hits = self.memo_hits.saturating_add(sub_evaluator.memo_hits);

        match outcome {
            StaticEvalOutcome::Reduced(v) => {
                let term = StaticEvalTerm::Value(v);
                self.application_memo.insert(key, StaticMemoResult::Reduced(term));
                Ok(term)
            }
            StaticEvalOutcome::ReducedKinded(ty) => {
                let term = StaticEvalTerm::Kinded(ty);
                self.application_memo.insert(key, StaticMemoResult::Reduced(term));
                Ok(term)
            }
            StaticEvalOutcome::Symbolic(r) => {
                self.application_memo.insert(key, StaticMemoResult::Symbolic(r));
                Ok(StaticEvalTerm::Residual(r))
            }
            StaticEvalOutcome::Invalid(err) => Err(Stop::invalid(err)),
            StaticEvalOutcome::Unknown(reason) => Err(Stop::Unknown(reason)),
            StaticEvalOutcome::Blocked(reason) => Err(Stop::Blocked(reason)),
            StaticEvalOutcome::Dynamic(obligation) => Err(Stop::Dynamic(obligation)),
            StaticEvalOutcome::Cancelled => Err(Stop::Cancelled),
            StaticEvalOutcome::BudgetExceeded(report) => Err(Stop::Budget(report)),
            StaticEvalOutcome::Recursive(cycle) => Err(Stop::Recursive(cycle)),
        }
    }

    /// Specializes a static residual using this evaluator's context.
    pub fn specialize_residual(
        &mut self,
        residual: StaticResidualRef,
        supplied: &StaticSpecializationEnvironmentKey,
    ) -> StaticEvalOutcome {
        match crate::static_eval::specialize::specialize_residual_term(self, residual, supplied) {
            Ok(StaticEvalTerm::Value(value)) => StaticEvalOutcome::Reduced(value),
            Ok(StaticEvalTerm::Kinded(term)) => {
                if let Some(parameter) = self.direct_parameter_of(term) {
                    match self.intern_parameter_residual(parameter) {
                        Ok(residual) => StaticEvalOutcome::Symbolic(residual),
                        Err(stop) => outcome_from_stop(stop),
                    }
                } else {
                    StaticEvalOutcome::ReducedKinded(term)
                }
            }
            Ok(StaticEvalTerm::Residual(r)) => StaticEvalOutcome::Symbolic(r),
            Err(stop) => outcome_from_stop(stop),
        }
    }

    fn intern_parameter_residual(&mut self, parameter: crate::types::id::TypeParameterId) -> Result<StaticResidualRef, Stop> {
        let parameter = {
            let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
            let data = types.type_parameter(parameter);
            let crate::types::parameter::TypeParameterOwner::Declaration(owner) = &data.owner else {
                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
            };
            let static_parameter_index = self
                .definitions
                .and_then(|definitions| definitions.get(owner))
                .and_then(|product| {
                    product
                        .definition
                        .signature
                        .parameters
                        .iter()
                        .find(|candidate| candidate.type_parameter_id == Some(parameter))
                        .map(|candidate| candidate.index)
                })
                .or_else(|| {
                    self.current_application_key
                        .as_ref()
                        .filter(|key| key.definition == *owner)
                        .and_then(|_| self.body.kinded_parameter_index(parameter))
                })
                .unwrap_or(data.index);
            StaticResidualParameter {
                owner: owner.clone(),
                index: static_parameter_index,
                domain: StaticValueDomain::Kind(data.kind),
            }
        };
        let budget = &mut *self.budget;
        let store = if let Some(ref mut residuals) = self.residuals {
            residuals
        } else {
            self.owned_residuals.get_or_insert_with(StaticResidualStore::new)
        };
        store.parameter(parameter, budget).map_err(Stop::from)
    }

    fn member_projection_for_type(&self, form: TypeId, result_kind: KindId) -> Option<MemberProjection> {
        let types = evaluator_type_store_ref(&self.canonical_types)?;
        match types.get(form) {
            crate::types::store::TypeData::AssociatedProjection(projection) => Some(MemberProjection {
                subject: MemberProjectionSubject::Type(projection.subject),
                member: MemberDeclarationIdentity::TraitAssociated {
                    trait_ref: projection.trait_ref.clone(),
                    requirement: projection.requirement.clone(),
                },
                arguments: Box::new([]),
                evidence: None,
                result_domain: StaticValueDomain::Kind(result_kind),
            }),
            crate::types::store::TypeData::TypeMemberProjection(projection) => {
                let member = MemberDeclarationIdentity::TypeMember(projection.source_declaration.clone()?);
                let subject = match projection.subject {
                    crate::types::type_member::ProjectionSubject::Type(ty) => MemberProjectionSubject::Type(ty),
                    crate::types::type_member::ProjectionSubject::RecordRow(row) => MemberProjectionSubject::RecordRow(row),
                };
                let arguments = projection
                    .arguments
                    .iter()
                    .cloned()
                    .map(MemberProjectionArgument::Kinded)
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                Some(MemberProjection {
                    subject,
                    member,
                    arguments,
                    evidence: None,
                    result_domain: StaticValueDomain::Kind(result_kind),
                })
            }
            _ => None,
        }
    }

    fn intern_member_projection_residual(&mut self, projection: MemberProjection) -> Result<StaticResidualRef, Stop> {
        let environment = self.capture_residual_environment()?;
        let trusted_context = self.capture_trusted_branch_context();
        let budget = &mut *self.budget;
        let store = if let Some(ref mut residuals) = self.residuals {
            residuals
        } else {
            self.owned_residuals.get_or_insert_with(StaticResidualStore::new)
        };
        store
            .intern_with_trusted_context(
                StaticResidualData::MemberProjection {
                    projection: projection.clone(),
                    result_domain: projection.result_domain,
                    environment,
                },
                trusted_context,
                budget,
            )
            .map_err(Stop::from)
    }

    fn residual_result_domain(&self, residual: StaticResidualRef) -> Result<StaticValueDomain, Stop> {
        self.residuals()
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?
            .get(residual)
            .map(|data| data.result_domain())
            .map_err(Stop::from)
    }

    fn direct_parameter_of(&self, form: TypeId) -> Option<crate::types::id::TypeParameterId> {
        let types = evaluator_type_store_ref(&self.canonical_types)?;
        match types.get(form) {
            crate::types::store::TypeData::Parameter(parameter) => Some(*parameter),
            _ => None,
        }
    }

    fn complete_residual_result(&mut self, residual: StaticResidualRef) -> Result<(), Stop> {
        if self.completed_residual_result.replace(residual).is_some() || self.completed_kinded_result.is_some() {
            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
        }
        Ok(())
    }

    fn make_symbolic_residual(&mut self) -> Result<StaticResidualRef, Stop> {
        let owner = match self.body {
            EvaluatorBody::Definition(d) => d.owner.0.clone(),
            _ => DeclarationId::new(crate::identity::ModuleId::universe_root(), "".into()),
        };
        let parameter = StaticResidualParameter {
            owner,
            index: 0,
            domain: StaticValueDomain::Kind(KindId::TYPE),
        };
        let budget = &mut *self.budget;
        let store = if let Some(ref mut r) = self.residuals {
            r
        } else {
            self.owned_residuals.get_or_insert_with(StaticResidualStore::new)
        };
        store.parameter(parameter, budget).map_err(Stop::from)
    }

    fn intern_proof_residual(&mut self, goal: ProofPredicateGoal, res_type: TypeId) -> Result<StaticResidualRef, Stop> {
        let environment = self.capture_residual_environment()?;
        let trusted_context = self.capture_trusted_branch_context();
        let budget = &mut *self.budget;
        let store = if let Some(ref mut r) = self.residuals {
            r
        } else {
            self.owned_residuals.get_or_insert_with(StaticResidualStore::new)
        };
        store
            .intern_with_trusted_context(StaticResidualData::Proof { goal, result_type: res_type, environment }, trusted_context, budget)
            .map_err(Stop::from)
    }

    pub(crate) fn capture_residual_environment(&self) -> Result<StaticSpecializationEnvironmentKey, Stop> {
        let Some(key) = &self.current_application_key else {
            return Ok(self.inherited_specialization_environment.clone());
        };
        self.capture_residual_environment_for_key(key)
    }

    fn capture_residual_environment_for_key(
        &self,
        key: &StaticApplicationKey,
    ) -> Result<StaticSpecializationEnvironmentKey, Stop> {
        let mut environment = self.inherited_specialization_environment.clone();

        {
            let signature = self
                .definitions
                .and_then(|definitions| definitions.get(&key.definition))
                .map(|product| &product.definition.signature);

            let mut current_args = Vec::new();
            for (index, argument) in key.arguments.iter().enumerate() {
                let argument = match argument {
                    StaticArgumentIdentity::Kinded(ty) => StaticSpecializationArgument::Kinded(*ty),
                    StaticArgumentIdentity::Value(value) => StaticSpecializationArgument::Value(*value),
                    StaticArgumentIdentity::Residual(_) => continue,
                };
                let domain = if let Some(signature) = signature {
                    signature.parameters.get(index).map(|parameter| parameter.domain)
                } else if let EvaluatorBody::Definition(body) = self.body
                    && body.owner.0 == key.definition
                    && let Some(parameter) = body.kinded_parameters.get(&(index as u32))
                {
                    let types = evaluator_type_store_ref(&self.canonical_types)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    Some(StaticValueDomain::Kind(types.type_parameter(*parameter).kind))
                } else {
                    None
                }
                .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;

                let formal = StaticResidualParameter {
                    owner: key.definition.clone(),
                    index: index as u32,
                    domain,
                };
                current_args.push((formal, argument));
            }
            environment = environment
                .overlay_arguments(current_args)
                .map_err(|err| Stop::invalid(StaticEvalError::SpecializationEnvironment(err)))?;
        }

        Ok(environment)
    }

    fn capture_trusted_branch_context(&self) -> StaticTrustedBranchContext {
        self.inherited_trusted_branch_context
            .overlay_branch_proofs(self.active_branch_proofs.iter().copied())
    }

    fn capture_root_trusted_branch_context(&self, root_key: &StaticApplicationKey) -> StaticTrustedBranchContext {
        let root_proofs = self
            .saved_frames
            .iter()
            .rev()
            .find(|frame| frame.application_key.as_ref() == Some(root_key))
            .map(|frame| frame.active_branch_proofs.as_slice())
            .unwrap_or_else(|| {
                if self.current_application_key.as_ref() == Some(root_key) {
                    self.active_branch_proofs.as_slice()
                } else {
                    &[]
                }
            });
        self.inherited_trusted_branch_context.overlay_branch_proofs(root_proofs.iter().copied())
    }

    pub(crate) fn replace_trusted_branch_context(&mut self, context: StaticTrustedBranchContext) -> StaticTrustedBranchContext {
        std::mem::replace(&mut self.inherited_trusted_branch_context, context)
    }

    pub(crate) fn trusted_branch_context(&self) -> &StaticTrustedBranchContext {
        &self.inherited_trusted_branch_context
    }

    pub(crate) fn reintern_residual_with_trusted_context(
        &mut self,
        data: StaticResidualData,
        context: StaticTrustedBranchContext,
    ) -> Result<StaticResidualRef, Stop> {
        let budget = &mut *self.budget;
        let store = if let Some(ref mut residuals) = self.residuals {
            residuals
        } else {
            self.owned_residuals.get_or_insert_with(StaticResidualStore::new)
        };
        store.intern_with_trusted_context(data, context, budget).map_err(Stop::from)
    }

    fn make_cycle(&self, repeated: &StaticApplicationKey, range: Option<SourceRange>, cycle_start: usize) -> StaticCycle {
        let mut applications = self
            .active_applications
            .iter()
            .enumerate()
            .map(|(index, key)| self.cycle_frame(key, self.active_application_ranges.get(index).copied().flatten()))
            .collect::<Vec<_>>();
        applications.push(self.cycle_frame(repeated, range));
        StaticCycle::new(applications.into_boxed_slice(), cycle_start as u32)
    }

    fn cycle_frame(&self, key: &StaticApplicationKey, source_range: Option<SourceRange>) -> StaticCycleFrame {
        let arguments = key
            .arguments
            .iter()
            .map(|argument| match argument {
                StaticArgumentIdentity::Kinded(ty) => StaticArgumentSummary::Kinded(*ty),
                StaticArgumentIdentity::Value(value) => StaticArgumentSummary::Value(*value),
                StaticArgumentIdentity::Residual(residual) => StaticArgumentSummary::Residual(*residual),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        StaticCycleFrame {
            definition: key.definition.clone(),
            arguments,
            source_range,
        }
    }

    fn pop_active_application(&mut self, key: &StaticApplicationKey) -> Result<(), Stop> {
        let popped = self
            .active_applications
            .pop()
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        let _ = self.active_application_ranges.pop();
        if popped != *key || self.active_application_index.remove(key).is_none() {
            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
        }
        Ok(())
    }

    fn root_is_direct_proof(&self) -> bool {
        let EvaluatorBody::Definition(body) = self.body else {
            return false;
        };
        let crate::static_definition::StaticDefinitionBodyRepr::TypeForm(annotation) = &body.root else {
            return false;
        };
        body.expressions
            .iter()
            .any(|(id, expression)| expression.range == annotation.range && body.proof_predicates.contains_key(id))
    }

    fn root_directly_returns_parameter(&self, candidate: &StaticResidualParameter, trusted_context: &StaticTrustedBranchContext) -> bool {
        let EvaluatorBody::Definition(body) = self.body else {
            return false;
        };
        let crate::static_definition::StaticDefinitionBodyRepr::TypeForm(annotation) = &body.root else {
            return false;
        };
        let Some(analysis) = body.expressions.values().find(|analysis| analysis.range == annotation.range) else {
            return false;
        };
        let Some(crate::types::denotation::SemanticDenotation::TypeForm(form) | crate::types::denotation::SemanticDenotation::StaticTerm(form)) =
            analysis.denotation.as_ref()
        else {
            return false;
        };
        let Some(parameter) = self.direct_parameter_of(*form) else {
            return false;
        };
        let Some(index) = body.kinded_parameters.iter().find_map(|(index, id)| (*id == parameter).then_some(*index)) else {
            return false;
        };
        candidate.owner == body.owner.0
            && candidate.index == index
            && candidate.domain == body.result
            && self.inherited_specialization_environment.is_empty()
            && trusted_context.is_empty()
    }

    fn residual_for_root(&mut self, residual: StaticResidualRef) -> Result<StaticResidualRef, Stop> {
        if self.root_is_direct_proof() {
            return Ok(residual);
        }
        let (Some(result_domain), Some(key)) = (self.root_result_domain, self.root_application_key.clone()) else {
            return Ok(residual);
        };
        let root_definition = key.definition.clone();
        let root_arguments = key.arguments.iter().copied().map(StaticEvalTerm::from).collect::<Vec<_>>();
        let environment = self.capture_residual_environment_for_key(&key)?;
        let trusted_context = self.capture_root_trusted_branch_context(&key);
        let residual_data = self
            .residuals()
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?
            .get(residual)
            .map_err(Stop::from)?
            .clone();
        if let StaticResidualData::Parameter { parameter } = residual_data {
            if self.root_directly_returns_parameter(&parameter, &trusted_context) {
                return Ok(residual);
            }
        }

        self.intern_applied_residual_with_context(
            StaticResidualApplication {
                definition: root_definition,
                arguments: root_arguments.into_boxed_slice(),
                environment,
            },
            result_domain,
            trusted_context,
        )
    }

    pub fn evaluate(&mut self) -> StaticEvalOutcome {
        match self.run() {
            Ok(StaticEvalResult::Value(value)) => StaticEvalOutcome::Reduced(value),
            Ok(StaticEvalResult::KindedTerm(term)) => {
                if let Some(parameter) = self.direct_parameter_of(term) {
                    match self.intern_parameter_residual(parameter) {
                        Ok(residual) => match self.residual_for_root(residual) {
                            Ok(residual) => StaticEvalOutcome::Symbolic(residual),
                            Err(stop) => outcome_from_stop(stop),
                        },
                        Err(stop) => outcome_from_stop(stop),
                    }
                } else {
                    StaticEvalOutcome::ReducedKinded(term)
                }
            }
            Ok(StaticEvalResult::Residual(residual)) => match self.residual_for_root(residual) {
                Ok(residual) => StaticEvalOutcome::Symbolic(residual),
                Err(stop) => outcome_from_stop(stop),
            },
            Err(Stop::Symbolic(residual)) => match self.residual_for_root(residual) {
                Ok(residual) => StaticEvalOutcome::Symbolic(residual),
                Err(stop) => outcome_from_stop(stop),
            },
            Err(stop) => outcome_from_stop(stop),
        }
    }

    /// Returns the active or most recent bounded diagnostic trace after evaluation.
    pub fn trace(&self) -> &[StaticEvalTraceFrame] {
        &self.trace
    }

    fn run(&mut self) -> Result<StaticEvalResult, Stop> {
        if let Some(error) = self.argument_error.take() {
            return Err(Stop::invalid(error));
        }
        self.push_trace(self.body.trace_frame())?;
        let mut work = vec![Work::FinishRoot];
        work.push(match self.body.root_work() {
            EvaluatorRootWork::Expression(expression) => Work::Evaluate(expression),
            EvaluatorRootWork::Block(block) => Work::EvaluateBlock(block),
            EvaluatorRootWork::TypeForm(annotation) => Work::EvaluateTypeForm(annotation),
        });
        let mut stack = Vec::new();
        let mut pending_control = None;
        let mut root_result = None;
        while let Some(item) = work.pop() {
            if self.cancellation.is_cancelled() {
                return Err(Stop::Cancelled);
            }
            self.budget.charge_step().map_err(Stop::Budget)?;
            if pending_control.is_some()
                && !matches!(
                    item,
                    Work::PopTrace
                        | Work::FinishRoot
                        | Work::FinishStaticApplication(..)
                        | Work::AbortStaticApplication(_)
                        | Work::AfterWhileBody(_)
                        | Work::FinishMatchArm(_)
                        | Work::AfterForBody(..)
                )
            {
                continue;
            }
            match item {
                Work::Evaluate(expression) => {
                    self.push_trace(StaticEvalTraceFrame::Expression(expression.range()))?;
                    work.push(Work::PopTrace);
                    match expression {
                        Expr::Int { digits, radix, .. } => {
                            self.charge_variable_work(digits.len())?;
                            let integer =
                                BigInt::parse_bytes(digits.as_bytes(), *radix).ok_or_else(|| Stop::invalid(StaticEvalError::InvalidIntegerLiteral))?;
                            stack.push(self.values.intern_int(integer));
                        }
                        Expr::Float { value, .. } => {
                            stack.push(self.values.intern_float(crate::static_value::StaticFloat::from_f64(*value)));
                        }
                        Expr::String { value, .. } => stack.push(self.values.intern_string(value)),
                        Expr::Boolean { value, .. } => stack.push(self.values.intern_bool(*value)),
                        Expr::Char { value, .. } => stack.push(self.values.intern_char(*value)),
                        Expr::Symbol(symbol) => {
                            let spelling = canonical_symbol_spelling(&symbol.kind).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                            self.charge_variable_work(spelling.len())?;
                            stack.push(self.values.intern_symbol(spelling));
                        }
                        Expr::Var { .. } => {
                            let id = self.expression_id(expression.range())?;
                            match self.body.expressions().get(&id).and_then(|analysis| analysis.denotation.as_ref()) {
                                Some(
                                    crate::types::denotation::SemanticDenotation::TypeForm(_) | crate::types::denotation::SemanticDenotation::StaticTerm(_),
                                ) if self.body.result_as_kind().is_some()
                                    || self.body.member_projections().is_some_and(|projections| projections.contains_key(&id)) =>
                                {
                                    self.complete_checked_type_form(id, &mut stack)?
                                }
                                _ => match self.read_binding_value(id)? {
                                    StaticEvalTerm::Value(value) => stack.push(value),
                                    StaticEvalTerm::Residual(residual) => self.complete_residual_result(residual)?,
                                    StaticEvalTerm::Kinded(_) => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                                },
                            }
                        }
                        Expr::TypeForm(annotation) => {
                            let id = self.expression_id(expression.range())?;
                            if let Some(predicate) = self.body.proof_predicates().get(&id) {
                                let result = self.evaluate_proof_predicate(&predicate.goal, predicate.result_type)?;
                                stack.push(self.values.intern_bool(result));
                            } else if let Some(application) = self.body.static_applications().iter().find(|application| application.expression == id) {
                                let TypeAnnotationExpr::Application { arguments, .. } = &annotation.expr else {
                                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                };
                                work.push(Work::StartStaticApplication(application, arguments));
                            } else {
                                self.complete_checked_type_form(id, &mut stack)?;
                            }
                        }
                        Expr::Binary(binary) if matches!(&binary.op, BinaryOp::And | BinaryOp::Or) => {
                            work.push(Work::AfterLogicalLeft(binary));
                            work.push(Work::Evaluate(&binary.left));
                        }
                        Expr::Binary(binary) => {
                            let id = self.expression_id(binary.range)?;
                            work.push(Work::ApplyBinary(id));
                            work.push(Work::Evaluate(&binary.right));
                            work.push(Work::Evaluate(&binary.left));
                        }
                        Expr::If(if_expression) => {
                            work.push(Work::AfterIf(if_expression));
                            work.push(Work::Evaluate(&if_expression.condition));
                        }
                        Expr::While(while_expression) => {
                            work.push(Work::AfterWhileCondition(while_expression));
                            work.push(Work::Evaluate(&while_expression.condition));
                        }
                        Expr::Assignment(assignment) => {
                            let id = self.expression_id(assignment.range)?;
                            let binding = self
                                .body
                                .expressions()
                                .get(&id)
                                .and_then(|entry| entry.binding_write)
                                .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                            if !self.body.local_slots().contains_key(&binding) || !self.body.bindings().get(&binding).is_some_and(|state| state.mutable) {
                                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                            }
                            work.push(Work::StoreAssignment(binding));
                            work.push(Work::Evaluate(&assignment.value));
                        }
                        Expr::TupleLiteral(tuple) => {
                            let mut elements = Vec::with_capacity(tuple.entries.len());
                            for entry in &tuple.entries {
                                match entry {
                                    TupleLiteralEntry::Positional { expr, .. } => elements.push(expr),
                                    TupleLiteralEntry::Labeled { .. } | TupleLiteralEntry::Expand { .. } => {
                                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                    }
                                }
                            }
                            self.charge_variable_work(elements.len())?;
                            work.push(Work::FinishTuple(elements.len()));
                            for element in elements.into_iter().rev() {
                                work.push(Work::Evaluate(element));
                            }
                        }
                        Expr::BracketLiteral(literal) => {
                            let id = self.expression_id(literal.range)?;
                            let resolution = self
                                .body
                                .sequence_literals()
                                .get(&id)
                                .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                            if resolution.source_expression != id || resolution.source_range != literal.range {
                                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                            }
                            if !matches!(resolution.family, SequenceLiteralFamily::Array | SequenceLiteralFamily::Vector) {
                                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                            }
                            match resolution.construction {
                                SequenceConstructionPlan::Fixed { element_count } => {
                                    if element_count != literal.elements.len()
                                        || literal.elements.iter().any(|element| !matches!(element, BracketLiteralElement::Element { .. }))
                                    {
                                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                    }
                                }
                                SequenceConstructionPlan::Expanded => {
                                    if !literal.elements.iter().any(|element| matches!(element, BracketLiteralElement::Expansion { .. })) {
                                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                    }
                                    for element in &literal.elements {
                                        if let BracketLiteralElement::Expansion { range, .. } = element
                                            && !resolution.exact_expansions.contains_key(range)
                                        {
                                            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                        }
                                    }
                                }
                                SequenceConstructionPlan::VectorIdentitySpread { source_range, extent } => {
                                    if resolution.family != SequenceLiteralFamily::Vector
                                        || resolution.extent != Some(extent)
                                        || !matches!(
                                            literal.elements.as_slice(),
                                            [BracketLiteralElement::Expansion { range, .. }] if *range == source_range
                                        )
                                        || resolution.exact_expansions.get(&source_range)
                                            != Some(&SequenceExpansionResolution::VectorStorage { extent })
                                    {
                                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                    }
                                    let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                                    if canonical_vector_extent(types, resolution.result_type) != Some(extent) {
                                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                    }
                                    if !is_static_int_type(types, extent) {
                                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                    }
                                    match types.get(extent) {
                                        crate::types::store::TypeData::ConstInt(value) => {
                                            if value.to_bigint().sign() == num_bigint::Sign::Minus {
                                                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                            }
                                        }
                                        _ => {
                                            let r = self.make_symbolic_residual()?;
                                            return Err(Stop::Symbolic(r));
                                        }
                                    }
                                }
                            }
                            let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                            if canonical_sequence_family(types, resolution.result_type) != Some(resolution.family) {
                                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                            }
                            self.charge_variable_work(literal.elements.len())?;
                            work.push(Work::SequenceNext(literal, resolution, 0, Vec::new()));
                        }
                        Expr::RecordLiteral(record) => {
                            let mut fields = Vec::with_capacity(record.entries.len());
                            for entry in &record.entries {
                                let RecordLiteralEntry::Field(field) = entry else {
                                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                };
                                fields.push(field);
                            }
                            self.charge_variable_work(fields.len())?;
                            work.push(Work::RecordNext(fields, 0, Vec::new()));
                        }
                        Expr::RecordConstruction(record) => {
                            let id = self.expression_id(expression.range())?;
                            let construction = self
                                .body
                                .static_constructions()
                                .get(&id)
                                .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                            if construction.arguments.len() != record.entries.len() {
                                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                            }
                            work.push(Work::FinishStaticConstruction(id, record.entries.len()));
                            for argument in construction.arguments.iter().rev() {
                                let entry = record
                                    .entries
                                    .iter()
                                    .find(|entry| {
                                        self.body
                                            .expressions()
                                            .get(&argument.expression)
                                            .is_some_and(|analysis| analysis.range == entry.value.range())
                                    })
                                    .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                                work.push(Work::Evaluate(&entry.value));
                            }
                        }
                        Expr::AssociatedInvoke(invoke) => {
                            let id = self.expression_id(expression.range())?;
                            let construction = self
                                .body
                                .static_constructions()
                                .get(&id)
                                .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                            if construction.arguments.len() != invoke.args.len() {
                                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                            }
                            work.push(Work::FinishStaticConstruction(id, invoke.args.len()));
                            for argument in construction.arguments.iter().rev() {
                                let item = invoke
                                    .args
                                    .iter()
                                    .find(|item| {
                                        let source = match item {
                                            phalcom_ast::ast::PackItem::Positional { expr, .. }
                                            | phalcom_ast::ast::PackItem::Labeled { value: expr, .. }
                                            | phalcom_ast::ast::PackItem::Expand { expr, .. } => expr,
                                        };
                                        self.body
                                            .expressions()
                                            .get(&argument.expression)
                                            .is_some_and(|analysis| analysis.range == source.range())
                                    })
                                    .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                                let source = match item {
                                    phalcom_ast::ast::PackItem::Positional { expr, .. } | phalcom_ast::ast::PackItem::Labeled { value: expr, .. } => expr,
                                    phalcom_ast::ast::PackItem::Expand { .. } => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                                };
                                work.push(Work::Evaluate(source));
                            }
                        }
                        Expr::Match(match_expr) => {
                            let expression_id = self.expression_id(match_expr.range)?;
                            if !self.body.match_resolutions().contains_key(&expression_id) {
                                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                            }
                            work.push(Work::AfterMatchScrutinee(match_expr, expression_id));
                            work.push(Work::Evaluate(&match_expr.value));
                        }
                        _ => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                    }
                }
                Work::EvaluateTypeForm(annotation) => {
                    let expression = self.expression_id(annotation.range)?;
                    if let Some(predicate) = self.body.proof_predicates().get(&expression) {
                        let result = self.evaluate_proof_predicate(&predicate.goal, predicate.result_type)?;
                        stack.push(self.values.intern_bool(result));
                    } else if let Some(application) = self.body.static_applications().iter().find(|item| item.expression == expression) {
                        let TypeAnnotationExpr::Application { arguments, .. } = &annotation.expr else {
                            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                        };
                        work.push(Work::StartStaticApplication(application, arguments));
                    } else {
                        self.complete_checked_type_form(expression, &mut stack)?;
                    }
                }
                Work::EvaluateBlock(block) => {
                    work.push(Work::StatementNext(StatementContainer::Block(block), 0, None));
                }
                Work::FinishRoot => {
                    if let Some(control) = pending_control.take() {
                        match control_disposition(ControlBoundary::Root, control) {
                            ControlDisposition::ConsumeReturn(result) => {
                                if root_result.replace(result).is_some() {
                                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                }
                            }
                            _ => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                        }
                    }
                }
                Work::StatementNext(container, index, last) => match container.statements().get(index) {
                    Some(Statement::Expr { expr, .. }) => {
                        work.push(Work::StatementAfter(container, index + 1));
                        work.push(Work::Evaluate(expr));
                    }
                    Some(Statement::Let(binding)) => {
                        let binding_id = self.local_binding(binding)?;
                        if let Some(initializer) = &binding.value {
                            work.push(Work::StoreLocal(binding_id, container, index + 1));
                            work.push(Work::Evaluate(initializer));
                        } else {
                            let slot = self.local_slot(binding_id)?;
                            self.locals[slot] = None;
                            work.push(Work::StatementNext(container, index + 1, Some(self.values.intern_unit())));
                        }
                    }
                    Some(Statement::Return(statement)) => {
                        if let Some(expression) = &statement.value {
                            work.push(Work::FinishReturn);
                            work.push(Work::Evaluate(expression));
                        } else {
                            pending_control = Some(EvalControl::Return(StaticEvalResult::Value(self.values.intern_unit())));
                        }
                    }
                    Some(Statement::Break { .. }) => pending_control = Some(EvalControl::Break),
                    Some(Statement::Continue { .. }) => pending_control = Some(EvalControl::Continue),
                    Some(Statement::For(for_stmt)) => {
                        let lane = for_stmt.lanes.first().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                        let source_expr_id = self.expression_id(lane.iter.range())?;
                        if let Some(lanes) = self.body.static_iteration_lanes() {
                            if !lanes.contains_key(&source_expr_id) {
                                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                            }
                        }
                        work.push(Work::AfterForSource(container, index, for_stmt));
                        work.push(Work::Evaluate(&lane.iter));
                    }
                    Some(_) => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                    None => stack.push(last.unwrap_or_else(|| self.values.intern_unit())),
                },
                Work::StatementAfter(container, next) => {
                    let value = stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    work.push(Work::StatementNext(container, next, Some(value)));
                }
                Work::StoreLocal(binding, container, next) => {
                    let value = stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let slot = self.local_slot(binding)?;
                    self.locals[slot] = Some(value);
                    work.push(Work::StatementNext(container, next, Some(self.values.intern_unit())));
                }
                Work::AfterMatchScrutinee(match_expr, expression_id) => {
                    let scrutinee = if let Some(term) = self.completed_kinded_result.take() {
                        let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                        let kind = types.kind_of(term);
                        if kind == crate::types::id::KindId::TYPE {
                            self.values.intern_type(term, types).map_err(Stop::from)?
                        } else {
                            self.values.intern_static_term(term, types).map_err(Stop::from)?
                        }
                    } else {
                        stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?
                    };
                    work.push(Work::TryMatchArm(match_expr, expression_id, scrutinee, 0));
                }
                Work::TryMatchArm(match_expr, expression_id, scrutinee, arm_index) => {
                    if arm_index >= match_expr.arms.len() {
                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                    }
                    let resolution = self
                        .body
                        .match_resolutions()
                        .get(&expression_id)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let arm_res = resolution
                        .arms
                        .get(arm_index)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let ast_arm = match_expr
                        .arms
                        .get(arm_index)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;

                    match match_pattern_against_value(
                        &arm_res.pattern,
                        scrutinee,
                        self.values,
                        self.budget,
                        self.cancellation,
                    ) {
                        StaticPatternMatchOutcome::Matched(bindings) => {
                            let mut saved_locals = Vec::new();
                            for (binding, value) in bindings {
                                if let Some(&slot) = self.body.local_slots().get(&binding) {
                                    let slot = slot as usize;
                                    if slot < self.locals.len() {
                                        saved_locals.push((slot, self.locals[slot]));
                                        self.locals[slot] = Some(value);
                                    }
                                }
                            }
                            self.active_branch_proofs.push(&arm_res.proof);
                            work.push(Work::FinishMatchArm(saved_locals));
                            match &ast_arm.branch {
                                MatchArmBody::Expression(expr) => work.push(Work::Evaluate(expr)),
                                MatchArmBody::Block(block) => work.push(Work::EvaluateBlock(block)),
                            }
                        }
                        StaticPatternMatchOutcome::NoMatch => {
                            work.push(Work::TryMatchArm(match_expr, expression_id, scrutinee, arm_index + 1));
                        }
                        StaticPatternMatchOutcome::Symbolic => {
                            let r = self.make_symbolic_residual()?;
                            return Err(Stop::Symbolic(r));
                        }
                        StaticPatternMatchOutcome::Invalid(err) => return Err(Stop::invalid(err)),
                        StaticPatternMatchOutcome::Cancelled => return Err(Stop::Cancelled),
                        StaticPatternMatchOutcome::BudgetExceeded(report) => return Err(Stop::Budget(report)),
                    }
                }
                Work::FinishMatchArm(saved_locals) => {
                    for (slot, prev) in saved_locals {
                        self.locals[slot] = prev;
                    }
                    self.active_branch_proofs.pop();
                }
                Work::AfterForSource(container, statement_index, for_stmt) => {
                    let source_val = if let Some(term) = self.completed_kinded_result.take() {
                        let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                        let kind = types.kind_of(term);
                        if kind == crate::types::id::KindId::TYPE {
                            self.values.intern_type(term, types).map_err(Stop::from)?
                        } else {
                            self.values.intern_static_term(term, types).map_err(Stop::from)?
                        }
                    } else {
                        stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?
                    };
                    let cursor = match self.values.get(source_val).map_err(Stop::from)? {
                        StaticValueData::Pack(pack_id) => {
                            let pos = self.values.pack_positionals(*pack_id).map_err(Stop::from)?;
                            let lab = self.values.pack_labeled(*pack_id).map_err(Stop::from)?;
                            StaticIterationCursor::Pack {
                                pack_id: *pack_id,
                                pos_len: pos.len(),
                                total_len: pos.len() + lab.len(),
                                current_index: 0,
                            }
                        }
                        StaticValueData::Tuple(tuple_id) => {
                            let tuple = self.values.get_tuple(*tuple_id).map_err(Stop::from)?;
                            StaticIterationCursor::Tuple {
                                tuple_id: *tuple_id,
                                len: tuple.len(),
                                current_index: 0,
                            }
                        }
                        StaticValueData::Sequence(seq_id) => {
                            let seq = self.values.get_sequence(*seq_id).map_err(Stop::from)?;
                            StaticIterationCursor::Sequence {
                                seq_id: *seq_id,
                                len: seq.elements().len(),
                                current_index: 0,
                            }
                        }
                        StaticValueData::Record(rec_id) => {
                            let rec = self.values.get_record(*rec_id).map_err(Stop::from)?;
                            StaticIterationCursor::Record {
                                rec_id: *rec_id,
                                len: rec.len(),
                                current_index: 0,
                            }
                        }
                        StaticValueData::Data(data_id) => {
                            let data = self.values.get_data(*data_id).map_err(Stop::from)?;
                            if data.components.len() == 2 {
                                if let Ok(StaticValueData::Record(rec_id)) = self.values.get(data.components[0].1) {
                                    if let Ok(StaticValueData::Variant(tail_var_id)) = self.values.get(data.components[1].1) {
                                        let tail_var = self.values.get_variant(*tail_var_id).map_err(Stop::from)?;
                                        if selector_name(&tail_var.variant.selector) != Some("Closed") {
                                            let r = self.make_symbolic_residual()?;
                                            return Err(Stop::Symbolic(r));
                                        }
                                        let rec = self.values.get_record(*rec_id).map_err(Stop::from)?;
                                        StaticIterationCursor::Record {
                                            rec_id: *rec_id,
                                            len: rec.len(),
                                            current_index: 0,
                                        }
                                    } else {
                                        StaticIterationCursor::Data {
                                            data_id: *data_id,
                                            len: data.components.len(),
                                            current_index: 0,
                                        }
                                    }
                                } else {
                                    StaticIterationCursor::Data {
                                        data_id: *data_id,
                                        len: data.components.len(),
                                        current_index: 0,
                                    }
                                }
                            } else {
                                StaticIterationCursor::Data {
                                    data_id: *data_id,
                                    len: data.components.len(),
                                    current_index: 0,
                                }
                            }
                        }
                        StaticValueData::Type(ty) => {
                            let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                            let ty = *ty;
                            match types.get(ty) {
                                crate::types::store::TypeData::Tuple(tuple) => {
                                    if !tuple.is_closed() {
                                        let r = self.make_symbolic_residual()?;
                                        return Err(Stop::Symbolic(r));
                                    }
                                    StaticIterationCursor::TypeTuple {
                                        ty,
                                        len: tuple.elements.len(),
                                        current_index: 0,
                                    }
                                }
                                crate::types::store::TypeData::Pack(pack) => {
                                    if !pack.is_closed() {
                                        let r = self.make_symbolic_residual()?;
                                        return Err(Stop::Symbolic(r));
                                    }
                                    StaticIterationCursor::TypePack {
                                        ty,
                                        len: pack.elements.len(),
                                        current_index: 0,
                                    }
                                }
                                crate::types::store::TypeData::TypePack(elements) => {
                                    StaticIterationCursor::TypeElements {
                                        elements: elements.clone(),
                                        current_index: 0,
                                    }
                                }
                                crate::types::store::TypeData::LabeledTypePack(elements) => {
                                    let elem_types: Box<[TypeId]> = elements.iter().map(|e| e.element).collect();
                                    StaticIterationCursor::TypeElements {
                                        elements: elem_types,
                                        current_index: 0,
                                    }
                                }
                                _ => {
                                    if let Some((data_table, enum_table)) = self.semantic_tables {
                                        let classifier = crate::structural::StructuralClassifier::new(data_table, enum_table);
                                        let mut local_types = (*types).clone();
                                        if let Ok(Some(unordered)) = classifier.unordered_product(&mut local_types, ty) {
                                            if !matches!(unordered.tail, crate::types::row::RecordRowTail::Closed) {
                                                let r = self.make_symbolic_residual()?;
                                                return Err(Stop::Symbolic(r));
                                            }
                                            let mut elem_types = Vec::new();
                                            for field in unordered.fields.iter() {
                                                if let Some(f_ty) = field.ty {
                                                    elem_types.push(f_ty);
                                                }
                                            }
                                            StaticIterationCursor::TypeElements {
                                                elements: elem_types.into_boxed_slice(),
                                                current_index: 0,
                                            }
                                        } else if let Ok(Some(ordered)) = classifier.ordered_product(&mut local_types, ty) {
                                            if ordered.positional_remainder != crate::types::store::AnonymousTupleRemainder::None
                                                || ordered.labeled_remainder != crate::types::store::AnonymousTupleRemainder::None
                                            {
                                                let r = self.make_symbolic_residual()?;
                                                return Err(Stop::Symbolic(r));
                                            }
                                            let mut elem_types = Vec::new();
                                            for entry in ordered.positional_entries.iter().chain(ordered.labeled_entries.iter()) {
                                                if let Some(entry_ty) = entry.ty {
                                                    elem_types.push(entry_ty);
                                                }
                                            }
                                            StaticIterationCursor::TypeElements {
                                                elements: elem_types.into_boxed_slice(),
                                                current_index: 0,
                                            }
                                        } else {
                                            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                        }
                                    } else {
                                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                    }
                                }
                            }
                        }
                        StaticValueData::Unit => StaticIterationCursor::Empty,
                        StaticValueData::StaticTerm(_) => {
                            let r = self.make_symbolic_residual()?;
                            return Err(Stop::Symbolic(r));
                        }
                        _ => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                    };
                    work.push(Work::ForNext(container, statement_index, for_stmt, cursor));
                }
                Work::ForNext(container, statement_index, for_stmt, cursor) => {
                    if cursor.is_done() {
                        work.push(Work::StatementNext(container, statement_index + 1, Some(self.values.intern_unit())));
                    } else {
                        self.charge_variable_work(1)?;
                        let current_item = self.fetch_cursor_item(&cursor)?;
                        let current_index = cursor.current_loop_index();
                        let lane = for_stmt.lanes.first().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                        let key = PatternSiteKey::new(PatternSiteKind::ForLane, lane.pattern.range());
                        let site_res = self
                            .body
                            .pattern_site_resolutions()
                            .get(&key)
                            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                        match match_pattern_against_value(
                            &site_res.pattern,
                            current_item,
                            self.values,
                            self.budget,
                            self.cancellation,
                        ) {
                            StaticPatternMatchOutcome::Matched(bindings) => {
                                let mut saved_locals = Vec::new();
                                for (binding, value) in bindings {
                                    if let Some(&slot) = self.body.local_slots().get(&binding) {
                                        let slot = slot as usize;
                                        if slot < self.locals.len() {
                                            saved_locals.push((slot, self.locals[slot]));
                                            self.locals[slot] = Some(value);
                                        }
                                    }
                                }
                                if let Some(index_binding) = &lane.index {
                                    let binding_id = self.for_index_binding(index_binding)?;
                                    if let Some(&slot) = self.body.local_slots().get(&binding_id) {
                                        let slot = slot as usize;
                                        if slot < self.locals.len() {
                                            saved_locals.push((slot, self.locals[slot]));
                                            let int_val = self.values.intern_int(BigInt::from(current_index));
                                            self.locals[slot] = Some(int_val);
                                        }
                                    }
                                }
                                work.push(Work::AfterForBody(
                                    container,
                                    statement_index,
                                    for_stmt,
                                    cursor,
                                    saved_locals,
                                ));
                                work.push(Work::StatementNext(StatementContainer::ForBody(for_stmt), 0, None));
                            }
                            StaticPatternMatchOutcome::NoMatch => {
                                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                            }
                            StaticPatternMatchOutcome::Symbolic => {
                                let r = self.make_symbolic_residual()?;
                                return Err(Stop::Symbolic(r));
                            }
                            StaticPatternMatchOutcome::Invalid(err) => return Err(Stop::invalid(err)),
                            StaticPatternMatchOutcome::Cancelled => return Err(Stop::Cancelled),
                            StaticPatternMatchOutcome::BudgetExceeded(report) => return Err(Stop::Budget(report)),
                        }
                    }
                }
                Work::AfterForBody(container, statement_index, for_stmt, mut cursor, saved_locals) => {
                    if pending_control.is_none() {
                        let _ = stack.pop();
                    }
                    for (slot, prev) in saved_locals {
                        self.locals[slot] = prev;
                    }
                    let control = pending_control.take();
                    match control.map(|c| control_disposition(ControlBoundary::Loop, c)) {
                        Some(ControlDisposition::ConsumeBreak) => {
                            work.push(Work::StatementNext(container, statement_index + 1, Some(self.values.intern_unit())));
                        }
                        Some(ControlDisposition::ConsumeContinue) | None => {
                            cursor.advance();
                            work.push(Work::ForNext(container, statement_index, for_stmt, cursor));
                        }
                        Some(ControlDisposition::Propagate(c)) => {
                            pending_control = Some(c);
                        }
                        Some(ControlDisposition::Invalid) | Some(ControlDisposition::ConsumeReturn(_)) => {
                            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                        }
                    }
                }
                Work::FinishReturn => {
                    let result = if let Some(residual) = self.completed_residual_result.take() {
                        StaticEvalResult::Residual(residual)
                    } else if let Some(term) = self.completed_kinded_result.take() {
                        StaticEvalResult::KindedTerm(term)
                    } else {
                        StaticEvalResult::Value(stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?)
                    };
                    pending_control = Some(EvalControl::Return(result));
                }
                Work::StoreAssignment(binding) => {
                    let value = stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let slot = self.local_slot(binding)?;
                    self.locals[slot] = Some(value);
                    stack.push(self.values.intern_unit());
                }
                Work::AfterIf(if_expression) => {
                    let condition = stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let StaticValueData::Bool(condition) = self.values.get(condition).map_err(Stop::from)? else {
                        return Err(Stop::invalid(StaticEvalError::ExpectedBool));
                    };
                    if *condition {
                        work.push(Work::EvaluateBlock(&if_expression.then_body));
                    } else {
                        match &if_expression.else_body {
                            Some(IfElseBody::Block(block)) => work.push(Work::EvaluateBlock(block)),
                            Some(IfElseBody::ElseIf(expression)) => work.push(Work::Evaluate(expression)),
                            None => stack.push(self.values.intern_unit()),
                        }
                    }
                }
                Work::AfterWhileCondition(while_expression) => {
                    let condition = stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let StaticValueData::Bool(condition) = self.values.get(condition).map_err(Stop::from)? else {
                        return Err(Stop::invalid(StaticEvalError::ExpectedBool));
                    };
                    if *condition {
                        work.push(Work::AfterWhileBody(while_expression));
                        work.push(Work::EvaluateBlock(&while_expression.body));
                    } else {
                        stack.push(self.values.intern_unit());
                    }
                }
                Work::AfterWhileBody(while_expression) => {
                    match pending_control
                        .take()
                        .map(|control| control_disposition(ControlBoundary::Loop, control))
                    {
                        Some(ControlDisposition::ConsumeBreak) => stack.push(self.values.intern_unit()),
                        Some(ControlDisposition::ConsumeContinue) => {
                            work.push(Work::AfterWhileCondition(while_expression));
                            work.push(Work::Evaluate(&while_expression.condition));
                        }
                        Some(ControlDisposition::Propagate(control)) => pending_control = Some(control),
                        Some(ControlDisposition::Invalid) => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                        Some(ControlDisposition::ConsumeReturn(_)) => {
                            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                        }
                        None => {
                            stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                            work.push(Work::AfterWhileCondition(while_expression));
                            work.push(Work::Evaluate(&while_expression.condition));
                        }
                    }
                }
                Work::FinishTuple(count) => {
                    let Some(start) = stack.len().checked_sub(count) else {
                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                    };
                    let elements = stack.split_off(start);
                    stack.push(self.values.intern_tuple(elements).map_err(Stop::from)?);
                }
                Work::SequenceNext(literal, resolution, index, elements) => {
                    let Some(element) = literal.elements.get(index) else {
                        charge_variable_work(self.budget, self.cancellation, elements.len())?;
                        let expected_extent = match resolution.construction {
                            SequenceConstructionPlan::Fixed { .. } => None,
                            SequenceConstructionPlan::VectorIdentitySpread { extent, .. } => Some(extent),
                            SequenceConstructionPlan::Expanded if resolution.family == SequenceLiteralFamily::Vector => {
                                let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                                canonical_vector_extent(types, resolution.result_type)
                            }
                            SequenceConstructionPlan::Expanded => None,
                        };
                        let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                        if expected_extent.is_some_and(|extent| !static_extent_matches_count(types, extent, elements.len())) {
                            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                        }
                        stack.push(self.values.intern_sequence(resolution.result_type, elements, types).map_err(Stop::from)?);
                        continue;
                    };
                    match element {
                        BracketLiteralElement::Element { expr, .. } => {
                            work.push(Work::SequenceElementReady(literal, resolution, index + 1, elements));
                            work.push(Work::Evaluate(expr));
                        }
                        BracketLiteralElement::Expansion { expr, range } => {
                            let proof = *resolution
                                .exact_expansions
                                .get(range)
                                .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                            work.push(Work::SequenceExpansionReady(literal, resolution, index + 1, elements, *range, proof));
                            work.push(Work::Evaluate(expr));
                        }
                    }
                }
                Work::SequenceElementReady(literal, resolution, next, mut elements) => {
                    elements.push(stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?);
                    work.push(Work::SequenceNext(literal, resolution, next, elements));
                }
                Work::SequenceExpansionReady(literal, resolution, next, mut elements, range, proof) => {
                    let source = stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let expanded = match (proof, self.values.get(source).map_err(Stop::from)?) {
                        (SequenceExpansionResolution::TuplePositionals { extent }, StaticValueData::Unit) => {
                            if !static_extent_matches_count(types, extent, 0) {
                                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                            }
                            Vec::new()
                        }
                        (SequenceExpansionResolution::TuplePositionals { extent }, StaticValueData::Tuple(tuple)) => {
                            let tuple = self.values.get_tuple(*tuple).map_err(Stop::from)?;
                            if !static_extent_matches_count(types, extent, tuple.len()) {
                                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                            }
                            tuple.to_vec()
                        }
                        (SequenceExpansionResolution::VectorStorage { extent }, StaticValueData::Sequence(sequence)) => {
                            let sequence = self.values.get_sequence(*sequence).map_err(Stop::from)?;
                            if canonical_vector_extent(types, sequence.result_type()) != Some(extent)
                                || !static_extent_matches_count(types, extent, sequence.elements().len())
                            {
                                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                            }
                            sequence.elements().to_vec()
                        }
                        _ => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                    };
                    // Expansion work is proportional to the number of copied
                    // children and is charged before growing the destination.
                    charge_variable_work(self.budget, self.cancellation, expanded.len())?;
                    elements.extend(expanded);
                    if let SequenceConstructionPlan::VectorIdentitySpread { source_range, extent } = resolution.construction {
                        if source_range != range
                            || resolution.extent != Some(extent)
                            || canonical_vector_extent(types, resolution.result_type) != Some(extent)
                            || !static_extent_matches_count(types, extent, elements.len())
                        {
                            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                        }
                    }
                    work.push(Work::SequenceNext(literal, resolution, next, elements));
                }
                Work::RecordNext(fields, index, accumulated) => {
                    let Some(field) = fields.get(index).copied() else {
                        self.charge_variable_work(accumulated.len())?;
                        stack.push(self.values.intern_record(accumulated).map_err(Stop::from)?);
                        continue;
                    };
                    match &field.label {
                        ProductLabel::Static {
                            symbol: SymbolLiteralKind::Name(name),
                            ..
                        } => {
                            work.push(Work::RecordValueReady(fields, index + 1, accumulated, FieldKey::symbol(name.as_str())));
                            work.push(Work::Evaluate(&field.value));
                        }
                        ProductLabel::Computed { expr, .. } => {
                            work.push(Work::RecordKeyReady(fields, index + 1, accumulated, &field.value));
                            work.push(Work::Evaluate(expr));
                        }
                        _ => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                    }
                }
                Work::RecordKeyReady(fields, next_index, accumulated, value) => {
                    let key_value = stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let work_units = match self.values.get(key_value).map_err(Stop::from)? {
                        StaticValueData::String(id) => self
                            .values
                            .get_string(*id)
                            .ok_or_else(|| Stop::from(crate::static_value::StaticValueStoreError::UnknownValue(id.raw())))?
                            .len(),
                        StaticValueData::Symbol(id) => self
                            .values
                            .get_symbol(*id)
                            .ok_or_else(|| Stop::from(crate::static_value::StaticValueStoreError::UnknownValue(id.raw())))?
                            .len(),
                        _ => 1,
                    };
                    self.charge_variable_work(work_units)?;
                    let key = self.values.field_key_of(key_value).map_err(Stop::from)?;
                    work.push(Work::RecordValueReady(fields, next_index, accumulated, key));
                    work.push(Work::Evaluate(value));
                }
                Work::RecordValueReady(fields, next_index, mut accumulated, key) => {
                    let value = stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    accumulated.push((key, value));
                    work.push(Work::RecordNext(fields, next_index, accumulated));
                }
                Work::FinishStaticConstruction(id, count) => {
                    // Data and Variant interning traverse/reorder payloads in
                    // proportion to their arity. Charge that work before
                    // invoking the canonical constructor.
                    self.charge_variable_work(count)?;
                    let Some(start) = stack.len().checked_sub(count) else {
                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                    };
                    let values = stack.split_off(start);
                    let construction = self
                        .body
                        .static_constructions()
                        .get(&id)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    if construction.arguments.len() != values.len() {
                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                    }
                    let (data, enums) = self.semantic_tables.ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let value = match &construction.target {
                        crate::identity::InvocationTargetId::DataConstructor(constructor) => {
                            let info = data
                                .get(&constructor.owner)
                                .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                            let components = construction
                                .arguments
                                .iter()
                                .zip(values)
                                .map(|(argument, value)| {
                                    let crate::static_definition::CheckedStaticPayloadId::DataComponent(component) = &argument.payload else {
                                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                    };
                                    Ok((component.clone(), value))
                                })
                                .collect::<Result<Vec<_>, Stop>>()?;
                            if info.constructor.constructor != *constructor {
                                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                            }
                            self.values.intern_data(info, construction.result_type, components, types).map_err(Stop::from)?
                        }
                        crate::identity::InvocationTargetId::VariantConstructor(constructor) => {
                            let info = enums
                                .variant_info(&constructor.variant)
                                .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                            let fields = construction
                                .arguments
                                .iter()
                                .zip(values)
                                .map(|(argument, value)| {
                                    let crate::static_definition::CheckedStaticPayloadId::VariantField(field) = &argument.payload else {
                                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                    };
                                    Ok((field.clone(), value))
                                })
                                .collect::<Result<Vec<_>, Stop>>()?;
                            self.values.intern_variant(info, construction.result_type, fields, types).map_err(Stop::from)?
                        }
                        _ => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                    };
                    stack.push(value);
                }
                Work::PopTrace => {
                    self.trace.pop();
                }
                Work::ApplyBinary(id) => {
                    let right = stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let left = stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let operation = self
                        .body
                        .static_operations()
                        .get(&id)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedOperation(id)))?;
                    if operation.operation != StaticIntrinsicOperation::NumberAdd {
                        return Err(Stop::invalid(StaticEvalError::UnsupportedOperation(id)));
                    }
                    let input_bits = match (self.values.get(left).map_err(Stop::from)?, self.values.get(right).map_err(Stop::from)?) {
                        (StaticValueData::Int(left), StaticValueData::Int(right)) => integer_bits(left).saturating_add(integer_bits(right)),
                        _ => return Err(Stop::invalid(StaticEvalError::InvalidOperationOperands)),
                    };
                    self.charge_variable_work(input_bits)?;
                    let sum = match (self.values.get(left).map_err(Stop::from)?, self.values.get(right).map_err(Stop::from)?) {
                        (StaticValueData::Int(left), StaticValueData::Int(right)) => left.to_bigint() + right.to_bigint(),
                        _ => return Err(Stop::invalid(StaticEvalError::InvalidOperationOperands)),
                    };
                    stack.push(self.values.intern_int(sum));
                }
                Work::AfterLogicalLeft(binary) => {
                    let left = stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let StaticValueData::Bool(condition) = self.values.get(left).map_err(Stop::from)? else {
                        return Err(Stop::invalid(StaticEvalError::ExpectedBool));
                    };
                    match (&binary.op, condition) {
                        (BinaryOp::And, false) | (BinaryOp::Or, true) => stack.push(self.values.intern_bool(*condition)),
                        (BinaryOp::And, true) => {
                            work.push(Work::FinishAnd);
                            work.push(Work::Evaluate(&binary.right));
                        }
                        (BinaryOp::Or, false) => {
                            work.push(Work::FinishOr);
                            work.push(Work::Evaluate(&binary.right));
                        }
                        _ => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                    }
                }
                Work::FinishAnd | Work::FinishOr => {
                    let value = stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    if !matches!(self.values.get(value).map_err(Stop::from)?, StaticValueData::Bool(_)) {
                        return Err(Stop::invalid(StaticEvalError::ExpectedBool));
                    }
                    stack.push(value);
                }
                Work::StartStaticApplication(application, syntax_arguments) => {
                    self.push_trace(StaticEvalTraceFrame::Application {
                        target: application.target.clone(),
                        source_range: application.source_range,
                    })?;
                    if syntax_arguments.len() != application.arguments.len() {
                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                    }
                    if self.pending_application_arguments.contains_key(&application.id) {
                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                    }
                    self.pending_application_arguments
                        .insert(application.id.clone(), vec![None; application.arguments.len()]);
                    work.push(Work::AbortStaticApplication(application.id.clone()));
                    work.push(Work::EnterStaticApplication(application));
                    for (checked, syntax) in application.arguments.iter().zip(syntax_arguments).rev() {
                        match checked {
                            crate::static_definition::CheckedStaticApplicationArgument::Kinded { parameter_index, term } => {
                                let arguments = self
                                    .pending_application_arguments
                                    .get_mut(&application.id)
                                    .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                                let slot = arguments
                                    .get_mut(*parameter_index as usize)
                                    .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                                *slot = Some(StaticArgument::Kinded(*term));
                            }
                            crate::static_definition::CheckedStaticApplicationArgument::Value { parameter_index, expression } => {
                                work.push(Work::StoreStaticArgument(application.id.clone(), *parameter_index));
                                match syntax {
                                    GenericArgumentSyntax::Value(expression) => work.push(Work::Evaluate(expression)),
                                    GenericArgumentSyntax::Single(annotation) => match &annotation.expr {
                                        TypeAnnotationExpr::Reference(reference) if reference.members.is_empty() => {
                                            work.push(Work::EvaluateBinding(*expression));
                                        }
                                        _ => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                                    },
                                    GenericArgumentSyntax::Expansion(_)
                                    | GenericArgumentSyntax::Remainder(_)
                                    | GenericArgumentSyntax::InferenceHole { .. } => {
                                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                    }
                                }
                            }
                            crate::static_definition::CheckedStaticApplicationArgument::KindedExpression { parameter_index, expression } => {
                                let nested = self
                                    .body
                                    .static_applications()
                                    .iter()
                                    .find(|candidate| candidate.expression == *expression)
                                    .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                                let annotation = match syntax {
                                    GenericArgumentSyntax::Single(annotation) => annotation,
                                    GenericArgumentSyntax::Value(expression) => match &**expression {
                                        Expr::TypeForm(annotation) => annotation.as_ref(),
                                        _ => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                                    },
                                    GenericArgumentSyntax::Expansion(_)
                                    | GenericArgumentSyntax::Remainder(_)
                                    | GenericArgumentSyntax::InferenceHole { .. } => {
                                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                    }
                                };
                                let TypeAnnotationExpr::Application { arguments, .. } = &annotation.expr else {
                                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                };
                                work.push(Work::StoreStaticKindedArgument(application.id.clone(), *parameter_index));
                                work.push(Work::StartStaticApplication(nested, arguments.as_slice()));
                            }
                        }
                    }
                }
                Work::EvaluateBinding(expression) => match self.read_binding_value(expression)? {
                    StaticEvalTerm::Value(value) => stack.push(value),
                    StaticEvalTerm::Residual(residual) => self.complete_residual_result(residual)?,
                    StaticEvalTerm::Kinded(_) => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                },
                Work::StoreStaticArgument(application, index) => {
                    let argument = if let Some(residual) = self.completed_residual_result.take() {
                        StaticArgument::Residual(residual)
                    } else {
                        let value = stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                        StaticArgument::Value(value)
                    };
                    let arguments = self
                        .pending_application_arguments
                        .get_mut(&application)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let slot = arguments
                        .get_mut(index as usize)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    *slot = Some(argument);
                }
                Work::StoreStaticKindedArgument(application, index) => {
                    let argument = if let Some(residual) = self.completed_residual_result.take() {
                        StaticArgument::Residual(residual)
                    } else {
                        let term = self
                            .completed_kinded_result
                            .take()
                            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                        StaticArgument::Kinded(term)
                    };
                    let arguments = self
                        .pending_application_arguments
                        .get_mut(&application)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let slot = arguments
                        .get_mut(index as usize)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    *slot = Some(argument);
                }
                Work::EnterStaticApplication(application) => {
                    let definitions = self.definitions.ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let product = definitions
                        .get(&application.target)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    if product.definition.signature != application.signature {
                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                    }
                    let target_body = product
                        .checked_body
                        .as_ref()
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let args = self
                        .pending_application_arguments
                        .remove(&application.id)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let args = args
                        .into_iter()
                        .map(|argument| argument.ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression)))
                        .collect::<Result<Vec<_>, _>>()?;
                    let mut args = args;
                    for argument in &mut args {
                        if let StaticArgument::Kinded(term) = argument {
                            *term = self.specialize_active_kinded_term(*term, application.source_range)?;
                        }
                    }
                    self.residualize_open_type_arguments(&mut args)?;
                    self.validate_application_arguments(&product.definition.signature, &args)?;
                    if target_body.result != product.definition.signature.result_domain {
                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                    }

                    let environment = self.capture_residual_environment()?;
                    let trusted_context = self.capture_trusted_branch_context();
                    let application_env = environment
                        .application_identity_projection()
                        .with_trusted_branch_context(trusted_context.clone());
                    let key = StaticApplicationKey::new(application.target.clone(), args.iter().copied(), application_env);
                    if let Some(&cycle_start) = self.active_application_index.get(&key) {
                        return Err(Stop::Recursive(self.make_cycle(&key, Some(application.source_range), cycle_start)));
                    }
                    if let Some(memoized) = self.application_memo.get(&key).cloned() {
                        self.memo_hits = self.memo_hits.saturating_add(1);
                        self.trace.pop();
                        match memoized {
                            StaticMemoResult::Reduced(StaticEvalTerm::Value(value)) => stack.push(value),
                            StaticMemoResult::Reduced(StaticEvalTerm::Kinded(ty)) => {
                                if self.completed_kinded_result.replace(ty).is_some() {
                                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                }
                            }
                            StaticMemoResult::Reduced(StaticEvalTerm::Residual(residual)) | StaticMemoResult::Symbolic(residual) => {
                                self.complete_residual_result(residual)?;
                            }
                        }
                        continue;
                    }

                    if args.iter().any(|argument| matches!(argument, StaticArgument::Residual(_))) {
                        let residual_arguments = args
                            .iter()
                            .map(|argument| match argument {
                                StaticArgument::Kinded(term) => StaticEvalTerm::Kinded(*term),
                                StaticArgument::Value(value) => StaticEvalTerm::Value(*value),
                                StaticArgument::Residual(residual) => StaticEvalTerm::Residual(*residual),
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice();
                        let residual = {
                            let budget = &mut *self.budget;
                            let store = if let Some(ref mut residuals) = self.residuals {
                                residuals
                            } else {
                                self.owned_residuals.get_or_insert_with(StaticResidualStore::new)
                            };
                            store
                                .intern_with_trusted_context(
                                    StaticResidualData::Apply {
                                        application: StaticResidualApplication {
                                        definition: application.target.clone(),
                                        arguments: residual_arguments,
                                        environment: environment.clone(),
                                    },
                                        result_domain: product.definition.signature.result_domain,
                                    },
                                    trusted_context,
                                    budget,
                                )
                                .map_err(Stop::from)?
                        };
                        self.application_memo.insert(key, StaticMemoResult::Symbolic(residual));
                        self.trace.pop();
                        self.complete_residual_result(residual)?;
                        continue;
                    }

                    let stack_base = stack.len();
                    self.save_current_frame();
                    if let Err(error) = self.load_frame(target_body, &args) {
                        self.restore_frame()?;
                        return Err(error);
                    }
                    let active_index = self.active_applications.len();
                    self.active_application_index.insert(key.clone(), active_index);
                    self.active_applications.push(key.clone());
                    self.active_application_ranges.push(Some(application.source_range));
                    self.current_application_key = Some(key.clone());
                    self.inherited_specialization_environment = key.environment.to_specialization_key();
                    self.push_trace(StaticEvalTraceFrame::Definition(application.target.clone()))?;
                    work.push(Work::FinishStaticApplication(application.id.clone(), key, stack_base));
                    match &target_body.root {
                        crate::static_definition::StaticDefinitionBodyRepr::Expression(expression) => work.push(Work::Evaluate(expression)),
                        crate::static_definition::StaticDefinitionBodyRepr::Block(block) => work.push(Work::EvaluateBlock(block)),
                        crate::static_definition::StaticDefinitionBodyRepr::TypeForm(annotation) => work.push(Work::EvaluateTypeForm(annotation)),
                    }
                }
                Work::FinishStaticApplication(application, key, stack_base) => {
                    let pending_arguments = self.pending_application_arguments.contains_key(&application);
                    let control = pending_control.take();
                    let disposition = control.map(|control| control_disposition(ControlBoundary::StaticApplication, control));
                    let returned = match disposition {
                        Some(ControlDisposition::ConsumeReturn(result)) => Some(result),
                        None => {
                            if let Some(residual) = self.completed_residual_result.take() {
                                Some(StaticEvalResult::Residual(residual))
                            } else if let Some(term) = self.completed_kinded_result.take() {
                                Some(StaticEvalResult::KindedTerm(term))
                            } else if stack.len() == stack_base.saturating_add(1) {
                                Some(StaticEvalResult::Value(stack[stack_base]))
                            } else {
                                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                            }
                        }
                        _ => None,
                    };
                    let validated_result = returned.map(|result| self.validate_result_domain(result));
                    self.pop_active_application(&key)?;
                    self.trace.pop();
                    self.trace.pop();
                    self.restore_frame()?;
                    if pending_arguments {
                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                    }
                    if let Some(result) = validated_result {
                        let result = result?;
                        stack.truncate(stack_base);
                        let memoized = match result {
                            StaticEvalResult::Value(value) => {
                                stack.push(value);
                                StaticMemoResult::Reduced(StaticEvalTerm::Value(value))
                            }
                            StaticEvalResult::KindedTerm(term) => {
                                if self.completed_kinded_result.replace(term).is_some() {
                                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                }
                                StaticMemoResult::Reduced(StaticEvalTerm::Kinded(term))
                            }
                            StaticEvalResult::Residual(residual) => {
                                self.complete_residual_result(residual)?;
                                StaticMemoResult::Symbolic(residual)
                            }
                        };
                        self.application_memo.insert(key, memoized);
                    }
                    if matches!(disposition, Some(ControlDisposition::Invalid)) {
                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                    }
                }
                Work::AbortStaticApplication(application) => {
                    if pending_control.is_some() {
                        self.pending_application_arguments.remove(&application);
                        self.trace.pop();
                    }
                }
            }
        }
        if let Some(result) = root_result {
            if !stack.is_empty() || self.completed_kinded_result.is_some() || self.completed_residual_result.is_some() {
                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
            }
            let result = self.validate_result_domain(result)?;
            if self.cancellation.is_cancelled() {
                return Err(Stop::Cancelled);
            }
            return Ok(result);
        }
        let result = if let Some(term) = self.completed_kinded_result.take() {
            if !stack.is_empty() || self.completed_residual_result.is_some() {
                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
            }
            StaticEvalResult::KindedTerm(term)
        } else if let Some(residual) = self.completed_residual_result.take() {
            if !stack.is_empty() || self.completed_kinded_result.is_some() {
                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
            }
            StaticEvalResult::Residual(residual)
        } else if stack.len() == 1 {
            StaticEvalResult::Value(stack[0])
        } else {
            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
        };
        let result = self.validate_result_domain(result)?;
        if self.cancellation.is_cancelled() {
            return Err(Stop::Cancelled);
        }
        Ok(result)
    }

    fn validate_result_domain(&self, result: StaticEvalResult) -> Result<StaticEvalResult, Stop> {
        match result {
            StaticEvalResult::Value(_) if self.body.is_value_result() => Ok(result),
            StaticEvalResult::Residual(residual) => {
                let crate::static_eval::machine::EvaluatorBody::Definition(body) = self.body else {
                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                };
                if self.residual_result_domain(residual)? == body.result {
                    Ok(result)
                } else {
                    Err(Stop::invalid(StaticEvalError::UnsupportedExpression))
                }
            }
            StaticEvalResult::KindedTerm(term) => {
                let Some(expected_kind) = self.body.result_as_kind() else {
                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                };
                let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                if types.kind_of(term) != expected_kind {
                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                }
                Ok(result)
            }
            _ => Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
        }
    }

    fn complete_checked_type_form(&mut self, expression: ExpressionId, stack: &mut Vec<StaticValueRef>) -> Result<(), Stop> {
        let analysis = self
            .body
            .expressions()
            .get(&expression)
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        let form = match analysis.denotation.as_ref() {
            Some(crate::types::denotation::SemanticDenotation::TypeForm(form) | crate::types::denotation::SemanticDenotation::StaticTerm(form)) => *form,
            _ => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
        };
        if let Some(projection) = self
            .body
            .member_projections()
            .and_then(|projections| projections.get(&expression))
            .cloned()
        {
            let projection = self.specialize_member_projection(projection, analysis.range)?;
            let outcome = self.project_checked_member(&projection)?;
            self.complete_member_projection_outcome(projection, outcome, stack)?;
            return Ok(());
        }
        let expected_kind = self
            .body
            .result_as_kind()
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        let parameter = match evaluator_type_store_ref(&self.canonical_types)
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?
            .get(form)
        {
            crate::types::store::TypeData::Parameter(parameter) => Some(*parameter),
            _ => None,
        };
        if let Some(index) = parameter.and_then(|parameter| self.body.kinded_parameter_index(parameter))
            && let Some(residual) = self.residual_kinded_arguments.get(&index).copied()
        {
            if self.residual_result_domain(residual)? != StaticValueDomain::Kind(expected_kind) {
                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
            }
            self.complete_residual_result(residual)?;
            return Ok(());
        }
        let specialized = self.specialize_active_kinded_term(form, analysis.range)?;
        let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        if types.kind_of(specialized) != expected_kind {
            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
        }
        if let Some(projection) = self.member_projection_for_type(specialized, expected_kind) {
            let residual = self.intern_member_projection_residual(projection)?;
            self.complete_residual_result(residual)?;
            return Ok(());
        }
        self.completed_kinded_result = Some(specialized);
        Ok(())
    }

    fn specialize_member_projection(&mut self, mut projection: MemberProjection, source_range: SourceRange) -> Result<MemberProjection, Stop> {
        projection.subject = match projection.subject {
            MemberProjectionSubject::Type(ty) => {
                MemberProjectionSubject::Type(self.specialize_active_kinded_term(ty, source_range)?)
            }
            MemberProjectionSubject::RecordRow(row) => MemberProjectionSubject::RecordRow(row),
            MemberProjectionSubject::Value(value) => MemberProjectionSubject::Value(value),
        };
        let mut arguments = Vec::with_capacity(projection.arguments.len());
        for argument in projection.arguments.iter() {
            let specialized = match argument {
                MemberProjectionArgument::Kinded(crate::types::type_member::TypeMemberArgument::Type(ty)) => {
                    MemberProjectionArgument::Kinded(crate::types::type_member::TypeMemberArgument::Type(
                        self.specialize_active_kinded_term(*ty, source_range)?,
                    ))
                }
                MemberProjectionArgument::Kinded(argument) => MemberProjectionArgument::Kinded(argument.clone()),
                MemberProjectionArgument::Value(value) => MemberProjectionArgument::Value(*value),
            };
            arguments.push(specialized);
        }
        projection.arguments = arguments.into_boxed_slice();
        projection.result_domain = match projection.result_domain {
            StaticValueDomain::Kind(kind) => StaticValueDomain::Kind(kind),
            StaticValueDomain::ValueType(ty) => {
                StaticValueDomain::ValueType(self.specialize_active_kinded_term(ty, source_range)?)
            }
        };
        if let Some(evidence) = projection.evidence.take() {
            let subject = match evidence.subject {
                TypeTerm::Canonical(ty) => TypeTerm::Canonical(self.specialize_active_kinded_term(ty, source_range)?),
                TypeTerm::SelfType(self_type) => TypeTerm::SelfType(self_type),
                TypeTerm::Infer(_) => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
            };
            let mut trait_arguments = Vec::with_capacity(evidence.trait_ref.arguments.len());
            for argument in evidence.trait_ref.arguments.iter() {
                trait_arguments.push(self.specialize_active_kinded_term(*argument, source_range)?);
            }
            projection.evidence = Some(GenericConformanceKey::new(
                subject,
                TraitRef::new(evidence.trait_ref.declaration, trait_arguments),
            ));
        }
        Ok(projection)
    }

    fn project_checked_member(&mut self, projection: &MemberProjection) -> Result<crate::member_projection::StaticMemberProjectionOutcome, Stop> {
        let (data, enums) = self
            .semantic_tables
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        let (_, _, _, declarations) = self
            .conformance
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        static STRUCTURAL_IDS: std::sync::OnceLock<crate::core_surface::identity::StructuralUniverseIds> = std::sync::OnceLock::new();
        let ids = STRUCTURAL_IDS.get_or_init(crate::core_surface::identity::StructuralUniverseIds::new);
        let classifier = crate::structural::StructuralClassifier::new(data, enums);
        let types = evaluator_type_store_mut(&mut self.canonical_types)
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        let mut projector = crate::structural_reflection::StructuralReflectionProjector::new(
            classifier,
            ids,
            types,
            self.values,
            declarations,
            data,
            enums,
        );
        projector
            .project_member(&projection.subject, &projection.member, &projection.arguments, None)
            .map_err(|error| Stop::invalid(StaticEvalError::MemberProjection(error.to_string().into_boxed_str())))
    }

    fn complete_member_projection_outcome(
        &mut self,
        request: MemberProjection,
        outcome: crate::member_projection::StaticMemberProjectionOutcome,
        stack: &mut Vec<StaticValueRef>,
    ) -> Result<(), Stop> {
        use crate::member_projection::StaticMemberProjectionOutcome as ProjectionOutcome;
        match outcome {
            ProjectionOutcome::ReducedValue(value) => {
                self.validate_projected_value(value, request.result_domain)?;
                stack.push(value);
                Ok(())
            }
            ProjectionOutcome::ReducedKinded(ty) => {
                let StaticValueDomain::Kind(expected_kind) = request.result_domain else {
                    return Err(Stop::invalid(StaticEvalError::MemberProjection(
                        "projector returned a kinded result for a value-domain site".into(),
                    )));
                };
                let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                if types.kind_of(ty) != expected_kind || !stack.is_empty() {
                    return Err(Stop::invalid(StaticEvalError::MemberProjection(
                        "reduced kinded result does not match the checked site".into(),
                    )));
                }
                if self.completed_kinded_result.replace(ty).is_some() || self.completed_residual_result.is_some() {
                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                }
                Ok(())
            }
            ProjectionOutcome::Stuck(projected) => {
                if projected.member != request.member
                    || projected.arguments != request.arguments
                    || projected.result_domain != request.result_domain
                {
                    return Err(Stop::invalid(StaticEvalError::MemberProjection(
                        "stuck projector result disagrees with the checked site".into(),
                    )));
                }
                let residual = self.intern_member_projection_residual(request)?;
                self.complete_residual_result(residual)
            }
            ProjectionOutcome::Invalid(reason) => Err(Stop::invalid(StaticEvalError::MemberProjection(reason))),
            ProjectionOutcome::Unknown(reason) => Err(Stop::Unknown(reason)),
            ProjectionOutcome::Blocked(reason) => Err(Stop::Blocked(reason)),
            ProjectionOutcome::Dynamic(obligation) => Err(Stop::Dynamic(obligation)),
            ProjectionOutcome::Cancelled => Err(Stop::Cancelled),
            ProjectionOutcome::BudgetExceeded(report) => Err(Stop::Budget(report)),
            ProjectionOutcome::InternalFailure(message) => Err(Stop::invalid(StaticEvalError::MemberProjection(message))),
        }
    }

    fn validate_projected_value(&mut self, value: StaticValueRef, domain: StaticValueDomain) -> Result<(), Stop> {
        let (data, enums) = self
            .semantic_tables
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        let Some(types) = evaluator_type_store_mut(&mut self.canonical_types) else {
            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
        };
        match domain {
            StaticValueDomain::Kind(expected) => {
                let actual = self.values.domain_of(value, types).map_err(Stop::from)?;
                if actual == StaticValueDomain::Kind(expected) {
                    Ok(())
                } else {
                    Err(Stop::invalid(StaticEvalError::MemberProjection(
                        "reduced value does not inhabit the checked kind domain".into(),
                    )))
                }
            }
            StaticValueDomain::ValueType(expected) => {
                let actual = self.values.static_value_type(value, types, data, enums).map_err(Stop::from)?;
                if actual == expected {
                    return Ok(());
                }
                let Some(hierarchy) = self.hierarchy else {
                    return Err(Stop::invalid(StaticEvalError::MemberProjection(
                        "reduced value type does not match the checked result domain".into(),
                    )));
                };
                let actual = crate::types::evidence::TypeKnowledge::established(
                    actual,
                    crate::types::evidence::EvidenceOrigin::ContextualDerivation,
                );
                match crate::types::relation::check_knowledge_against_type_bounded(
                    types,
                    hierarchy,
                    &actual,
                    expected,
                    self.budget,
                    self.cancellation,
                ) {
                    RelationOutcome::Proven { .. } => Ok(()),
                    RelationOutcome::Refuted(_) => Err(Stop::invalid(StaticEvalError::MemberProjection(
                        "reduced value type does not satisfy the checked result domain".into(),
                    ))),
                    RelationOutcome::Blocked(reason) => Err(Stop::Blocked(reason)),
                    RelationOutcome::DynamicBoundary(obligation) => Err(Stop::Dynamic(obligation)),
                    RelationOutcome::Cancelled => Err(Stop::Cancelled),
                    RelationOutcome::BudgetExceeded(report) => Err(Stop::Budget(report)),
                    RelationOutcome::InternalFailure(message) => {
                        Err(Stop::invalid(StaticEvalError::MemberProjection(message.into_boxed_str())))
                    }
                }
            }
        }
    }

    fn specialize_active_kinded_term(&mut self, form: TypeId, source_range: SourceRange) -> Result<TypeId, Stop> {
        let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        let expected_kind = types.kind_of(form);
        if let crate::types::store::TypeData::Parameter(parameter) = types.get(form) {
            if let Some(index) = self.body.kinded_parameter_index(*parameter) {
                let forwarded = *self
                    .kinded_arguments
                    .get(&index)
                    .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                if types.kind_of(forwarded) != expected_kind {
                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                }
                return Ok(forwarded);
            }
        }
        if !self.body.contains_kinded_parameter(types, form) {
            return Ok(form);
        }

        let mut substitution = TypeSubstitution::new();
        if let EvaluatorBody::Definition(definition) = &self.body {
            for (&index, &parameter) in definition.kinded_parameters.iter() {
                if let Some(&argument) = self.kinded_arguments.get(&index) {
                    substitution.bind(parameter, argument);
                }
            }
        }
        if self.cancellation.is_cancelled() {
            return Err(Stop::Cancelled);
        }
        // TypeSubstitution recursively traverses and interns the checked
        // canonical form. Precharge work proportional to its checked source
        // span before entering it, and check cancellation on both sides of the
        // synchronous canonical substitution call.
        self.charge_variable_work(source_range.end.saturating_sub(source_range.start))?;
        let specialized = {
            let types = evaluator_type_store_mut(&mut self.canonical_types)
                .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
            substitution
                .try_apply(types, form)
                .map_err(|_| Stop::invalid(StaticEvalError::UnsupportedExpression))?
        };
        if self.cancellation.is_cancelled() {
            return Err(Stop::Cancelled);
        }
        let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        if self.body.contains_kinded_parameter(types, specialized) || types.kind_of(specialized) != expected_kind {
            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
        }
        Ok(specialized)
    }

    fn validate_application_arguments(
        &self,
        signature: &crate::static_definition::StaticDefinitionSignature,
        arguments: &[StaticArgument],
    ) -> Result<(), Stop> {
        if arguments.len() != signature.parameters.len() {
            return Err(Stop::invalid(StaticEvalError::InvalidArgumentCount {
                expected: signature.parameters.len(),
                actual: arguments.len(),
            }));
        }
        for (parameter, argument) in signature.parameters.iter().zip(arguments) {
            match (parameter.domain, argument) {
                (crate::static_domain::StaticValueDomain::ValueType(_), StaticArgument::Value(_)) => {}
                (crate::static_domain::StaticValueDomain::Kind(expected), StaticArgument::Kinded(term)) => {
                    let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    if types.kind_of(*term) != expected {
                        return Err(Stop::invalid(StaticEvalError::InvalidArgumentKind { index: parameter.index }));
                    }
                }
                (expected, StaticArgument::Residual(residual)) => {
                    if self.residual_result_domain(*residual)? != expected {
                        return Err(Stop::invalid(StaticEvalError::InvalidArgumentKind { index: parameter.index }));
                    }
                }
                _ => return Err(Stop::invalid(StaticEvalError::InvalidArgumentKind { index: parameter.index })),
            }
        }
        Ok(())
    }

    fn residualize_open_type_arguments(&mut self, arguments: &mut [StaticArgument]) -> Result<(), Stop> {
        for argument in arguments {
            let StaticArgument::Kinded(form) = *argument else {
                continue;
            };
            let Some(parameter) = self.direct_parameter_of(form) else {
                continue;
            };
            *argument = StaticArgument::Residual(self.intern_parameter_residual(parameter)?);
        }
        Ok(())
    }

    fn expression_id(&self, range: SourceRange) -> Result<ExpressionId, Stop> {
        match self.expression_sites.get(&(range.start, range.end)).map(Vec::as_slice) {
            Some([id]) => Ok(*id),
            Some(_) => Err(Stop::invalid(StaticEvalError::AmbiguousExpressionSite {
                start: range.start,
                end: range.end,
            })),
            None => Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
        }
    }

    fn read_binding_value(&self, id: ExpressionId) -> Result<StaticEvalTerm, Stop> {
        let binding = self
            .body
            .expressions()
            .get(&id)
            .and_then(|entry| entry.binding_read)
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        if let Some(value) = self.parameter_values.get(&binding) {
            return Ok(StaticEvalTerm::Value(*value));
        }
        if let Some(residual) = self.residual_values.get(&binding) {
            return Ok(StaticEvalTerm::Residual(*residual));
        }
        if let Some(symbol) = self.body.static_constant_bindings().get(&binding) {
            if let Some(constants) = self.constants {
                let const_id = crate::static_constant::StaticConstantId(symbol.clone());
                if let Some(constant) = constants.get(&const_id) {
                    if let Some(value) = constant.value {
                        return Ok(StaticEvalTerm::Value(value));
                    }
                }
            }
            return Err(Stop::invalid(StaticEvalError::UnboundBinding(binding)));
        }
        if let Some(slot) = self.body.local_slots().get(&binding) {
            return self
                .locals
                .get(*slot as usize)
                .copied()
                .flatten()
                .map(StaticEvalTerm::Value)
                .ok_or_else(|| Stop::invalid(StaticEvalError::UninitializedBinding(binding)));
        }
        Err(Stop::invalid(StaticEvalError::UnboundBinding(binding)))
    }

    fn save_current_frame(&mut self) {
        let definition = match self.body {
            EvaluatorBody::Definition(d) => Some(d.owner.0.clone()),
            EvaluatorBody::Constant(_) | EvaluatorBody::Expression(_) => None,
        };
        self.saved_frames.push(SavedFrame {
            body: self.body,
            locals: std::mem::take(&mut self.locals),
            parameter_values: std::mem::take(&mut self.parameter_values),
            residual_values: std::mem::take(&mut self.residual_values),
            kinded_arguments: std::mem::take(&mut self.kinded_arguments),
            residual_kinded_arguments: std::mem::take(&mut self.residual_kinded_arguments),
            expression_sites: std::mem::take(&mut self.expression_sites),
            definition,
            active_branch_proofs: std::mem::take(&mut self.active_branch_proofs),
            application_key: self.current_application_key.clone(),
            inherited_specialization_environment: self.inherited_specialization_environment.clone(),
            inherited_trusted_branch_context: self.inherited_trusted_branch_context.clone(),
        });
    }

    fn load_frame(&mut self, body: &'a CheckedStaticBody, arguments: &[StaticArgument]) -> Result<(), Stop> {
        let parameter_count = body
            .parameter_bindings
            .keys()
            .chain(body.kinded_parameters.keys())
            .max()
            .map_or(0, |index| *index as usize + 1);
        if arguments.len() != parameter_count {
            return Err(Stop::invalid(StaticEvalError::InvalidArgumentCount {
                expected: parameter_count,
                actual: arguments.len(),
            }));
        }
        let mut parameter_values = BTreeMap::new();
        let mut residual_values = BTreeMap::new();
        let mut kinded_arguments = BTreeMap::new();
        let mut residual_kinded_arguments = BTreeMap::new();
        for (index, binding) in body.parameter_bindings.iter() {
            match arguments.get(*index as usize) {
                Some(StaticArgument::Value(value)) => {
                    parameter_values.insert(*binding, *value);
                }
                Some(StaticArgument::Residual(residual)) => {
                    residual_values.insert(*binding, *residual);
                }
                _ => return Err(Stop::invalid(StaticEvalError::InvalidArgumentKind { index: *index })),
            }
        }
        for index in body.kinded_parameters.keys() {
            match arguments.get(*index as usize) {
                Some(StaticArgument::Kinded(ty)) => {
                    kinded_arguments.insert(*index, *ty);
                }
                Some(StaticArgument::Residual(residual)) => {
                    residual_kinded_arguments.insert(*index, *residual);
                }
                _ => return Err(Stop::invalid(StaticEvalError::InvalidArgumentKind { index: *index })),
            }
        }
        let mut expression_sites: BTreeMap<(usize, usize), Vec<ExpressionId>> = BTreeMap::new();
        for (id, expression) in body.expressions.iter() {
            expression_sites.entry((expression.range.start, expression.range.end)).or_default().push(*id);
        }
        self.body = EvaluatorBody::Definition(body);
        self.locals = vec![None; body.local_slots.len()];
        self.parameter_values = parameter_values;
        self.residual_values = residual_values;
        self.kinded_arguments = kinded_arguments;
        self.residual_kinded_arguments = residual_kinded_arguments;
        self.expression_sites = expression_sites;
        self.active_branch_proofs = Vec::new();
        Ok(())
    }

    fn restore_frame(&mut self) -> Result<(), Stop> {
        let frame = self.saved_frames.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        self.body = frame.body;
        self.locals = frame.locals;
        self.parameter_values = frame.parameter_values;
        self.residual_values = frame.residual_values;
        self.kinded_arguments = frame.kinded_arguments;
        self.residual_kinded_arguments = frame.residual_kinded_arguments;
        self.expression_sites = frame.expression_sites;
        self.active_branch_proofs = frame.active_branch_proofs;
        self.current_application_key = frame.application_key;
        self.inherited_specialization_environment = frame.inherited_specialization_environment;
        self.inherited_trusted_branch_context = frame.inherited_trusted_branch_context;
        if let Some(def) = frame.definition {
            if let EvaluatorBody::Definition(current_def) = self.body {
                debug_assert_eq!(current_def.owner.0, def);
            }
        }
        Ok(())
    }

    fn local_binding(&self, binding: &phalcom_ast::ast::LetBinding) -> Result<BindingId, Stop> {
        let Pattern::Name { name, range } = &binding.pattern else {
            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
        };
        let mut candidates = self
            .body
            .bindings()
            .iter()
            .filter(|(_, state)| state.name == *name && state.range == *range && self.body.local_slots().contains_key(&state.binding))
            .map(|(id, _)| *id);
        let Some(binding) = candidates.next() else {
            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
        };
        if candidates.next().is_some() {
            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
        }
        Ok(binding)
    }

    fn local_slot(&self, binding: BindingId) -> Result<usize, Stop> {
        let Some(slot) = self.body.local_slots().get(&binding).copied() else {
            return Err(Stop::invalid(StaticEvalError::UnboundBinding(binding)));
        };
        let slot = slot as usize;
        if slot >= self.locals.len() {
            return Err(Stop::invalid(StaticEvalError::UnboundBinding(binding)));
        }
        Ok(slot)
    }

    fn charge_variable_work(&mut self, work_units: usize) -> Result<(), Stop> {
        charge_variable_work(self.budget, self.cancellation, work_units)
    }

    fn push_trace(&mut self, frame: StaticEvalTraceFrame) -> Result<(), Stop> {
        if self.cancellation.is_cancelled() {
            return Err(Stop::Cancelled);
        }
        self.budget.charge_step().map_err(Stop::Budget)?;
        self.trace.push(frame);
        Ok(())
    }

    fn for_index_binding(&self, index: &ForIndexBinding) -> Result<BindingId, Stop> {
        let mut candidates = self
            .body
            .bindings()
            .iter()
            .filter(|(_, state)| state.name == index.name && state.range == index.range && self.body.local_slots().contains_key(&state.binding))
            .map(|(id, _)| *id);
        let Some(binding) = candidates.next() else {
            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
        };
        if candidates.next().is_some() {
            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
        }
        Ok(binding)
    }

    fn resolve_type_term(&self, ty: TypeId, types: &crate::types::store::TypeStore) -> TypeId {
        self.resolve_type_term_bounded(ty, types, 0)
    }

    fn is_open_type_term(&self, ty: TypeId, types: &crate::types::store::TypeStore) -> bool {
        matches!(types.get(ty), crate::types::store::TypeData::Parameter(_)) || self.body.contains_kinded_parameter(types, ty)
    }

    fn active_generic_evidence(&self, types: &mut crate::types::store::TypeStore) -> crate::generic_evidence::GenericEvidenceEnvironment {
        let mut constraints = BTreeSet::new();
        for (parameter, ty) in self.inherited_specialization_environment.type_bindings() {
            if types.type_parameter(*parameter).kind == KindId::TYPE {
                let parameter_type = types.parameter_form(*parameter);
                constraints.insert(GenericConstraint::Equivalent {
                    left: TypeTerm::from(parameter_type),
                    right: TypeTerm::from(*ty),
                });
            }
        }
        for (parameter, ty) in self.inherited_trusted_branch_context.type_bindings() {
            if types.type_parameter(*parameter).kind == KindId::TYPE {
                let parameter_type = types.parameter_form(*parameter);
                constraints.insert(GenericConstraint::Equivalent {
                    left: TypeTerm::from(parameter_type),
                    right: TypeTerm::from(*ty),
                });
            }
        }
        for proof in &self.active_branch_proofs {
            for (parameter, ty) in &proof.bindings {
                if types.type_parameter(*parameter).kind == KindId::TYPE {
                    let parameter_type = types.parameter_form(*parameter);
                    constraints.insert(GenericConstraint::Equivalent {
                        left: TypeTerm::from(parameter_type),
                        right: TypeTerm::from(*ty),
                    });
                }
            }
            for equality in proof.equalities.iter() {
                match equality {
                    crate::types::constraint::TypeConstraint::Equal(left, right) => {
                        constraints.insert(GenericConstraint::Equivalent {
                            left: TypeTerm::from(*left),
                            right: TypeTerm::from(*right),
                        });
                    }
                    crate::types::constraint::TypeConstraint::Subtype(lower, upper) => {
                        constraints.insert(GenericConstraint::Subtype {
                            lower: TypeTerm::from(*lower),
                            upper: TypeTerm::from(*upper),
                        });
                    }
                    crate::types::constraint::TypeConstraint::HasMember(_, _) => {}
                }
            }
        }
        for fact in self.inherited_trusted_branch_context.constraints() {
            match fact {
                CanonicalStaticProofFact::Equal(left, right) => {
                    constraints.insert(GenericConstraint::Equivalent {
                        left: TypeTerm::from(*left),
                        right: TypeTerm::from(*right),
                    });
                }
                CanonicalStaticProofFact::Subtype { lower, upper } => {
                    constraints.insert(GenericConstraint::Subtype {
                        lower: TypeTerm::from(*lower),
                        upper: TypeTerm::from(*upper),
                    });
                }
                CanonicalStaticProofFact::Conforms { .. } => {}
            }
        }
        crate::generic_evidence::GenericEvidenceEnvironment::new(
            constraints.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            BTreeMap::new(),
            crate::generic_evidence::GenericEvidenceLayout::empty(),
        )
    }

    fn resolve_type_term_bounded(&self, ty: TypeId, types: &crate::types::store::TypeStore, depth: usize) -> TypeId {
        if depth > 16 || ty.index() >= types.len() {
            return ty;
        }

        if let crate::types::store::TypeData::Parameter(param) = types.get(ty) {
            if let Some(idx) = self.body.kinded_parameter_index(*param) {
                if let Some(&arg_ty) = self.kinded_arguments.get(&idx) {
                    return self.resolve_type_term_bounded(arg_ty, types, depth + 1);
                }
            }
            for proof in self.active_branch_proofs.iter().rev() {
                if let Some(&resolved) = proof.bindings.get(param) {
                    return self.resolve_type_term_bounded(resolved, types, depth + 1);
                }
            }
            for (inherited_param, resolved) in self.inherited_trusted_branch_context.type_bindings() {
                if inherited_param == param {
                    return self.resolve_type_term_bounded(*resolved, types, depth + 1);
                }
            }
            for (inherited_param, resolved) in self.inherited_specialization_environment.type_bindings() {
                if inherited_param == param {
                    return self.resolve_type_term_bounded(*resolved, types, depth + 1);
                }
            }
        }

        for proof in self.active_branch_proofs.iter().rev() {
            for eq in proof.equalities.iter() {
                if let crate::types::constraint::TypeConstraint::Equal(lhs, rhs) = eq {
                    if *lhs == ty {
                        return self.resolve_type_term_bounded(*rhs, types, depth + 1);
                    }
                    if *rhs == ty {
                        return self.resolve_type_term_bounded(*lhs, types, depth + 1);
                    }
                }
            }
        }

        for fact in self.inherited_trusted_branch_context.constraints() {
            if let CanonicalStaticProofFact::Equal(lhs, rhs) = fact {
                if *lhs == ty {
                    return self.resolve_type_term_bounded(*rhs, types, depth + 1);
                }
                if *rhs == ty {
                    return self.resolve_type_term_bounded(*lhs, types, depth + 1);
                }
            }
        }

        ty
    }

    fn evaluate_proof_predicate(&mut self, goal: &ProofPredicateGoal, res_type: TypeId) -> Result<bool, Stop> {
        let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        let goal_for_res = goal.clone();
        let is_open = |term: TypeId| self.is_open_type_term(term, types);
        let goal_has_open_terms = match goal {
            ProofPredicateGoal::Equivalent { left, right } => is_open(self.resolve_type_term(*left, types)) || is_open(self.resolve_type_term(*right, types)),
            ProofPredicateGoal::Subtype { lower, upper } => is_open(self.resolve_type_term(*lower, types)) || is_open(self.resolve_type_term(*upper, types)),
            ProofPredicateGoal::Conforms { key } => {
                let open_subject = match &key.subject {
                    TypeTerm::Canonical(ty) => is_open(self.resolve_type_term(*ty, types)),
                    TypeTerm::SelfType(_) => true,
                    TypeTerm::Infer(_) => false,
                };
                open_subject || key.trait_ref.arguments.iter().any(|ty| is_open(self.resolve_type_term(*ty, types)))
            }
        };
        if goal_has_open_terms {
            let residual = self.intern_proof_residual(goal_for_res, res_type)?;
            return Err(Stop::Symbolic(residual));
        }
        match goal {
            ProofPredicateGoal::Equivalent { left, right } => {
                let left = self.resolve_type_term(*left, types);
                let right = self.resolve_type_term(*right, types);
                if left == right {
                    return Ok(true);
                }
                let mut local_types = (*types).clone();
                let lexical_evidence = self.active_generic_evidence(&mut local_types);
                let (conf_idx, witness_plans, trait_surfs, decls) = match self.conformance {
                    Some((ci, wp, ts, d)) => (Some(ci), Some(wp), Some(ts), Some(d)),
                    None => (None, None, None, None),
                };
                let mut proof_context = ConformanceProofContext::new(
                    Some(&lexical_evidence),
                    conf_idx,
                    witness_plans,
                    trait_surfs,
                    decls,
                    self.hierarchy,
                    Some(&mut local_types),
                )
                .with_budget_and_cancellation(*self.budget, self.cancellation.clone());
                let constraint = GenericConstraint::Equivalent {
                    left: TypeTerm::from(left),
                    right: TypeTerm::from(right),
                };
                let proof = prove_constraint(&constraint, &TypeSubstitution::new(), &mut proof_context);
                *self.budget = proof_context.budget;
                match proof {
                    GenericObligationProof::Equivalent(GenericEquivalenceProof::Proven) => Ok(true),
                    GenericObligationProof::Equivalent(GenericEquivalenceProof::NotProven) => {
                        let r = self.intern_proof_residual(goal_for_res, res_type)?;
                        Err(Stop::Symbolic(r))
                    }
                    GenericObligationProof::Equivalent(GenericEquivalenceProof::Ambiguous)
                        => Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                    GenericObligationProof::Equivalent(GenericEquivalenceProof::Blocked(reason)) => Err(Stop::Blocked(reason)),
                    GenericObligationProof::Equivalent(GenericEquivalenceProof::Dynamic(obligation)) => Err(Stop::Dynamic(obligation)),
                    GenericObligationProof::Equivalent(GenericEquivalenceProof::Recursive) => Err(Stop::Blocked(
                        crate::types::outcome::BlockReason::RecursiveFixpoint,
                    )),
                    GenericObligationProof::Equivalent(GenericEquivalenceProof::Cancelled) => Err(Stop::Cancelled),
                    GenericObligationProof::Equivalent(GenericEquivalenceProof::BudgetExceeded(report)) => Err(Stop::Budget(report)),
                    _ => Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                }
            }
            ProofPredicateGoal::Subtype { lower, upper } => {
                let lower = self.resolve_type_term(*lower, types);
                let upper = self.resolve_type_term(*upper, types);
                let Some(hierarchy) = self.hierarchy else {
                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                };
                let mut local_types = (*types).clone();
                match check_subtype_bounded(&mut local_types, hierarchy, lower, upper, self.budget, self.cancellation) {
                    RelationOutcome::Proven { .. } => Ok(true),
                    RelationOutcome::Refuted(_) => Ok(false),
                    RelationOutcome::Blocked(reason) => Err(Stop::Blocked(reason)),
                    RelationOutcome::DynamicBoundary(obligation) => Err(Stop::Dynamic(obligation)),
                    RelationOutcome::Cancelled => Err(Stop::Cancelled),
                    RelationOutcome::BudgetExceeded(report) => Err(Stop::Budget(report)),
                    RelationOutcome::InternalFailure(_) => Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                }
            }
            ProofPredicateGoal::Conforms { key } => {
                let subject_term = match &key.subject {
                    TypeTerm::Canonical(ty) => {
                        let resolved = self.resolve_type_term(*ty, types);
                        TypeTerm::from(resolved)
                    }
                    other => other.clone(),
                };
                let mut resolved_args = Vec::with_capacity(key.trait_ref.arguments.len());
                for &arg_ty in key.trait_ref.arguments.iter() {
                    let resolved = self.resolve_type_term(arg_ty, types);
                    resolved_args.push(resolved);
                }
                let resolved_trait_ref = TraitRef::new(key.trait_ref.declaration.clone(), resolved_args);
                let resolved_key = GenericConformanceKey::new(subject_term, resolved_trait_ref);

                let mut local_types = (*types).clone();
                let (conf_idx, witness_plans, trait_surfs, decls) = match self.conformance {
                    Some((ci, wp, ts, d)) => (Some(ci), Some(wp), Some(ts), Some(d)),
                    None => (None, None, None, None),
                };
                let mut proof_context = ConformanceProofContext::new(
                    None,
                    conf_idx,
                    witness_plans,
                    trait_surfs,
                    decls,
                    self.hierarchy,
                    Some(&mut local_types),
                );
                if let Some((data_table, enum_table)) = self.semantic_tables {
                    let classifier = crate::structural::StructuralClassifier::new(data_table, enum_table);
                    static IDS: std::sync::OnceLock<crate::core_surface::identity::StructuralUniverseIds> = std::sync::OnceLock::new();
                    let ids = IDS.get_or_init(crate::core_surface::identity::StructuralUniverseIds::new);
                    proof_context.structural = Some(StructuralConformanceContext {
                        classifier,
                        ids,
                    });
                }
                let proof = prove_conformance_with_context(&resolved_key, &mut proof_context);
                match proof {
                    GenericConformanceProof::Proven(_) => Ok(true),
                    GenericConformanceProof::Missing => Ok(false),
                    GenericConformanceProof::Ambiguous(_)
                    | GenericConformanceProof::Incomplete(_)
                    | GenericConformanceProof::CoherenceConflict(_)
                        => Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                    GenericConformanceProof::Unknown(reason) => Err(Stop::Unknown(reason)),
                    GenericConformanceProof::Blocked(reason) => Err(Stop::Blocked(reason)),
                    GenericConformanceProof::Dynamic(obligation) => Err(Stop::Dynamic(obligation)),
                    GenericConformanceProof::Cancelled => Err(Stop::Cancelled),
                    GenericConformanceProof::BudgetExceeded(report) => Err(Stop::Budget(report)),
                    GenericConformanceProof::InternalFailure(_) => Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                }
            }
        }
    }

    fn fetch_cursor_item(&mut self, cursor: &StaticIterationCursor) -> Result<StaticValueRef, Stop> {
        match cursor {
            StaticIterationCursor::Empty => Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
            StaticIterationCursor::Pack { pack_id, pos_len, current_index, .. } => {
                if *current_index < *pos_len {
                    let pos = self.values.pack_positionals(*pack_id).map_err(Stop::from)?;
                    Ok(pos[*current_index])
                } else {
                    let lab = self.values.pack_labeled(*pack_id).map_err(Stop::from)?;
                    Ok(lab[*current_index - *pos_len].1)
                }
            }
            StaticIterationCursor::Tuple { tuple_id, current_index, .. } => {
                let tuple = self.values.get_tuple(*tuple_id).map_err(Stop::from)?;
                Ok(tuple[*current_index])
            }
            StaticIterationCursor::Sequence { seq_id, current_index, .. } => {
                let seq = self.values.get_sequence(*seq_id).map_err(Stop::from)?;
                Ok(seq.elements()[*current_index])
            }
            StaticIterationCursor::Record { rec_id, current_index, .. } => {
                let rec = self.values.get_record(*rec_id).map_err(Stop::from)?;
                Ok(rec[*current_index].value)
            }
            StaticIterationCursor::Data { data_id, current_index, .. } => {
                let data = self.values.get_data(*data_id).map_err(Stop::from)?;
                Ok(data.components[*current_index].1)
            }
            StaticIterationCursor::TypeTuple { ty, current_index, .. } => {
                let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                let elem_ty = match types.get(*ty) {
                    crate::types::store::TypeData::Tuple(t) => t.elements[*current_index].ty,
                    _ => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                };
                let kind = types.kind_of(elem_ty);
                if kind == crate::types::id::KindId::TYPE {
                    self.values.intern_type(elem_ty, types).map_err(Stop::from)
                } else {
                    self.values.intern_static_term(elem_ty, types).map_err(Stop::from)
                }
            }
            StaticIterationCursor::TypePack { ty, current_index, .. } => {
                let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                let elem_ty = match types.get(*ty) {
                    crate::types::store::TypeData::Pack(p) => p.elements[*current_index].ty,
                    _ => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                };
                let kind = types.kind_of(elem_ty);
                if kind == crate::types::id::KindId::TYPE {
                    self.values.intern_type(elem_ty, types).map_err(Stop::from)
                } else {
                    self.values.intern_static_term(elem_ty, types).map_err(Stop::from)
                }
            }
            StaticIterationCursor::TypeElements { elements, current_index } => {
                let types = evaluator_type_store_ref(&self.canonical_types).ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                let elem_ty = elements[*current_index];
                let kind = types.kind_of(elem_ty);
                if kind == crate::types::id::KindId::TYPE {
                    self.values.intern_type(elem_ty, types).map_err(Stop::from)
                } else {
                    self.values.intern_static_term(elem_ty, types).map_err(Stop::from)
                }
            }
        }
    }
}

fn selector_name(sel: &phalcom_common::selector::Selector) -> Option<&str> {
    match &sel.base {
        phalcom_common::selector::SelectorBase::Named(name) => Some(name.as_str()),
        phalcom_common::selector::SelectorBase::Subscript => None,
    }
}

fn integer_bits(value: &crate::static_value::StaticInt) -> usize {
    match value {
        crate::static_value::StaticInt::Small(value) => (i64::BITS - value.unsigned_abs().leading_zeros()) as usize,
        crate::static_value::StaticInt::Large(value) => value.bits() as usize,
    }
}

fn canonical_symbol_spelling(kind: &SymbolLiteralKind) -> Option<String> {
    use phalcom_common::selector::{Selector, SelectorSlot};

    let slots = |labels: &[Option<String>]| {
        labels
            .iter()
            .map(|label| label.clone().map_or(SelectorSlot::Positional, SelectorSlot::Label))
            .collect::<Vec<_>>()
            .into_boxed_slice()
    };
    match kind {
        SymbolLiteralKind::Name(name) => Some(name.clone()),
        SymbolLiteralKind::Selector { name, labels } => Selector::method(name.clone(), slots(labels)).ok().map(|selector| selector.encode()),
        SymbolLiteralKind::Subscript { labels, setter } => {
            let slots = slots(labels);
            if *setter {
                Selector::subscript_set(slots)
            } else {
                Selector::subscript_get(slots)
            }
            .ok()
            .map(|selector| selector.encode())
        }
        SymbolLiteralKind::Pattern(_) => None,
    }
}

fn canonical_sequence_family(types: &crate::types::store::TypeStore, ty: TypeId) -> Option<SequenceLiteralFamily> {
    if ty.index() >= types.len() {
        return None;
    }
    let crate::types::store::TypeData::Applied { origin, arguments } = types.get(ty) else {
        return None;
    };
    if origin.index() >= types.len() {
        return None;
    }
    let crate::types::store::TypeData::Nominal { declaration } = types.get(*origin) else {
        return None;
    };
    let core = crate::core_surface::CoreDeclarationIds::default();
    match declaration {
        declaration if *declaration == core.array => {
            (arguments.len() == 1 && matches!(arguments.first(), Some(crate::types::store::StaticArgument::Term(_))))
                .then_some(SequenceLiteralFamily::Array)
        }
        declaration if *declaration == core.vector => {
            (arguments.len() == 2
                && matches!(arguments.first(), Some(crate::types::store::StaticArgument::Term(_)))
                && matches!(arguments.get(1), Some(crate::types::store::StaticArgument::Term(_))))
            .then_some(SequenceLiteralFamily::Vector)
        }
        _ => None,
    }
}

fn canonical_vector_extent(types: &crate::types::store::TypeStore, ty: TypeId) -> Option<TypeId> {
    if canonical_sequence_family(types, ty) != Some(SequenceLiteralFamily::Vector) {
        return None;
    }
    let crate::types::store::TypeData::Applied { arguments, .. } = types.get(ty) else {
        return None;
    };
    match arguments.get(1) {
        Some(crate::types::store::StaticArgument::Term(extent)) => Some(*extent),
        _ => None,
    }
}

fn static_extent_matches_count(types: &crate::types::store::TypeStore, extent: TypeId, count: usize) -> bool {
    if extent.index() >= types.len() {
        return false;
    }
    matches!(types.get(extent), crate::types::store::TypeData::ConstInt(value) if value.to_bigint() == BigInt::from(count))
}

fn is_static_int_type(types: &crate::types::store::TypeStore, ty: TypeId) -> bool {
    if ty.index() >= types.len() {
        return false;
    }
    let kind = types.kind_of(ty);
    matches!(
        types.get_kind(kind),
        crate::types::KindData::StaticDomain(domain) if *domain == types.const_int_domain()
    )
}

pub(crate) enum Stop {
    Cancelled,
    Recursive(StaticCycle),
    Symbolic(StaticResidualRef),
    Budget(crate::types::outcome::BudgetReport),
    Invalid(StaticEvalError),
    Unknown(crate::types::evidence::UnknownReason),
    Blocked(crate::types::outcome::BlockReason),
    Dynamic(crate::types::outcome::DynamicBoundaryObligation),
}

pub(crate) fn outcome_from_stop(stop: Stop) -> StaticEvalOutcome {
    match stop {
        Stop::Cancelled => StaticEvalOutcome::Cancelled,
        Stop::Recursive(cycle) => StaticEvalOutcome::Recursive(cycle),
        Stop::Symbolic(residual) => StaticEvalOutcome::Symbolic(residual),
        Stop::Budget(report) => StaticEvalOutcome::BudgetExceeded(report),
        Stop::Invalid(error) => StaticEvalOutcome::Invalid(error),
        Stop::Unknown(reason) => StaticEvalOutcome::Unknown(reason),
        Stop::Blocked(reason) => StaticEvalOutcome::Blocked(reason),
        Stop::Dynamic(obligation) => StaticEvalOutcome::Dynamic(obligation),
    }
}

impl Stop {
    pub(crate) fn invalid(error: StaticEvalError) -> Self {
        Self::Invalid(error)
    }
}

impl From<crate::static_value::StaticValueStoreError> for Stop {
    fn from(error: crate::static_value::StaticValueStoreError) -> Self {
        Self::invalid(StaticEvalError::ValueStore(error.to_string()))
    }
}

impl From<crate::static_eval::residual::StaticResidualStoreError> for Stop {
    fn from(error: crate::static_eval::residual::StaticResidualStoreError) -> Self {
        match error {
            crate::static_eval::residual::StaticResidualStoreError::BudgetExceeded(report) => Self::Budget(report),
            other => Self::invalid(StaticEvalError::ValueStore(other.to_string())),
        }
    }
}

impl From<crate::static_eval::application::StaticSpecializationEnvironmentError> for Stop {
    fn from(error: crate::static_eval::application::StaticSpecializationEnvironmentError) -> Self {
        match error {
            crate::static_eval::application::StaticSpecializationEnvironmentError::Cancelled => Self::Cancelled,
            crate::static_eval::application::StaticSpecializationEnvironmentError::BudgetExceeded(report) => Self::Budget(report),
            error => Self::invalid(StaticEvalError::SpecializationEnvironment(error)),
        }
    }
}

#[cfg(test)]
mod terminal_outcome_tests {
    use super::*;

    #[test]
    fn unknown_blocked_and_dynamic_projection_states_remain_distinct() {
        assert_eq!(
            outcome_from_stop(Stop::Unknown(crate::types::evidence::UnknownReason::NoTypeEvidence)),
            StaticEvalOutcome::Unknown(crate::types::evidence::UnknownReason::NoTypeEvidence),
        );
        assert_eq!(
            outcome_from_stop(Stop::Blocked(crate::types::outcome::BlockReason::ReflectionBoundary)),
            StaticEvalOutcome::Blocked(crate::types::outcome::BlockReason::ReflectionBoundary),
        );
        let obligation = crate::types::outcome::DynamicBoundaryObligation {
            reason: "dynamic projection boundary".into(),
        };
        assert_eq!(
            outcome_from_stop(Stop::Dynamic(obligation.clone())),
            StaticEvalOutcome::Dynamic(obligation),
        );
    }
}
