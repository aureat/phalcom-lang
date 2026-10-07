use super::{StaticEvalError, StaticEvalOutcome, StaticEvalTraceFrame};
use crate::checker::analysis::{BindingAnalysisIndex, ExpressionAnalysisIndex};
use crate::checker::associated::AssociatedResolutionIndex;
use crate::db::{CancellationToken, QueryBudget};
use crate::field_key::FieldKey;
use crate::identity::{BindingId, ExpressionId};
use crate::static_constant::{CheckedStaticConstantInitializer, CheckedStaticExpression, StaticConstantTable};
use crate::static_definition::{CheckedStaticApplication, CheckedStaticBody, CheckedStaticConstruction, StaticApplicationId, StaticDefinitionTable};
use crate::static_intrinsic::{StaticIntrinsicOperation, StaticOperationResolutionIndex};
use crate::static_value::{StaticValueData, StaticValueRef, StaticValueStore};
use crate::types::id::TypeId;
use num_bigint::BigInt;
use phalcom_ast::ast::{
    BinaryOp, Expr, GenericArgumentSyntax, IfElseBody, LexicalBlock, Pattern, ProductLabel, RecordLiteralEntry, Statement, SymbolLiteralKind,
    TupleLiteralEntry, TypeAnnotationExpr,
};
use phalcom_common::range::SourceRange;
use std::collections::BTreeMap;

/// One argument aligned with the checked static definition parameter order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaticArgument {
    /// A canonical C3 static type/term argument for a kinded parameter.
    Kinded(TypeId),
    /// A semantic StaticValue for an ordinary value parameter.
    Value(StaticValueRef),
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

    #[allow(dead_code)]
    pub fn associated_resolutions(&self) -> &'a AssociatedResolutionIndex {
        match self {
            Self::Definition(d) => &d.associated_resolutions,
            Self::Constant(c) => &c.associated_resolutions,
            Self::Expression(expression) => &expression.associated_resolutions,
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
    kinded_arguments: BTreeMap<u32, TypeId>,
    argument_error: Option<StaticEvalError>,
    expression_sites: BTreeMap<(usize, usize), Vec<ExpressionId>>,
    definitions: Option<&'a StaticDefinitionTable>,
    constants: Option<&'a StaticConstantTable>,
    canonical_types: Option<&'a crate::types::store::TypeStore>,
    semantic_tables: Option<(
        &'a crate::types::store::TypeStore,
        &'a crate::data_semantics::DataSemanticTable,
        &'a crate::enum_semantics::EnumSemanticTable,
    )>,
    active_definitions: Vec<crate::identity::DeclarationId>,
    saved_frames: Vec<SavedFrame<'a>>,
    pending_application_arguments: BTreeMap<StaticApplicationId, Vec<Option<StaticArgument>>>,
    completed_kinded_result: Option<TypeId>,
    trace: Vec<StaticEvalTraceFrame>,
}

struct SavedFrame<'a> {
    body: EvaluatorBody<'a>,
    locals: Vec<Option<StaticValueRef>>,
    parameter_values: BTreeMap<BindingId, StaticValueRef>,
    kinded_arguments: BTreeMap<u32, TypeId>,
    expression_sites: BTreeMap<(usize, usize), Vec<ExpressionId>>,
    definition: Option<crate::identity::DeclarationId>,
}

enum Work<'a> {
    Evaluate(&'a Expr),
    EvaluateTypeForm(&'a phalcom_ast::ast::TypeAnnotation),
    EvaluateBlock(&'a LexicalBlock),
    FinishRoot,
    BlockNext(&'a LexicalBlock, usize, Option<StaticValueRef>),
    BlockAfter(&'a LexicalBlock, usize),
    FinishReturn,
    StoreLocal(BindingId, &'a LexicalBlock, usize),
    StoreAssignment(BindingId),
    AfterIf(&'a phalcom_ast::ast::IfExpr),
    FinishTuple(usize),
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
    FinishStaticApplication(StaticApplicationId),
    AbortStaticApplication(StaticApplicationId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StaticEvalResult {
    Value(StaticValueRef),
    KindedTerm(TypeId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EvalControl {
    Return(StaticEvalResult),
    Break,
    Continue,
}

#[allow(dead_code)] // The Loop boundary is consumed by T6's explicit while work item.
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
        let mut kinded_arguments = BTreeMap::new();
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
        Self {
            body: EvaluatorBody::Definition(body),
            values,
            budget,
            cancellation,
            locals,
            parameter_values,
            kinded_arguments,
            argument_error,
            expression_sites,
            definitions: None,
            constants: None,
            canonical_types: None,
            semantic_tables: None,
            active_definitions: vec![body.owner.0.clone()],
            saved_frames: Vec::new(),
            pending_application_arguments: BTreeMap::new(),
            completed_kinded_result: None,
            trace: Vec::new(),
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
            kinded_arguments: BTreeMap::new(),
            argument_error: None,
            expression_sites,
            definitions: None,
            constants: None,
            canonical_types: None,
            semantic_tables: None,
            active_definitions: Vec::new(),
            saved_frames: Vec::new(),
            pending_application_arguments: BTreeMap::new(),
            completed_kinded_result: None,
            trace: Vec::new(),
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
            kinded_arguments: BTreeMap::new(),
            argument_error: None,
            expression_sites,
            definitions: None,
            constants: None,
            canonical_types: None,
            semantic_tables: None,
            active_definitions: Vec::new(),
            saved_frames: Vec::new(),
            pending_application_arguments: BTreeMap::new(),
            completed_kinded_result: None,
            trace: Vec::new(),
        }
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
        self.canonical_types = Some(types);
        self.semantic_tables = Some((types, data, enums));
        self
    }

    /// Supplies the immutable canonical TypeStore needed for checked kinded
    /// application arguments and TypeForm results.
    pub fn with_type_store(mut self, types: &'a crate::types::store::TypeStore) -> Self {
        self.canonical_types = Some(types);
        self
    }

    pub fn evaluate(&mut self) -> StaticEvalOutcome {
        match self.run() {
            Ok(StaticEvalResult::Value(value)) => StaticEvalOutcome::Reduced(value),
            Ok(StaticEvalResult::KindedTerm(term)) => StaticEvalOutcome::ReducedKinded(term),
            Err(Stop::Cancelled) => StaticEvalOutcome::Cancelled,
            Err(Stop::Recursive) => StaticEvalOutcome::Recursive,
            Err(Stop::Budget(report)) => StaticEvalOutcome::BudgetExceeded(report),
            Err(Stop::Invalid(error)) => StaticEvalOutcome::Invalid(error),
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
                    Work::PopTrace | Work::FinishRoot | Work::FinishStaticApplication(_) | Work::AbortStaticApplication(_)
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
                                ) if self.body.result_as_kind().is_some() => self.complete_checked_type_form(id)?,
                                _ => stack.push(self.read_binding_value(id)?),
                            }
                        }
                        Expr::TypeForm(annotation) => {
                            let id = self.expression_id(expression.range())?;
                            if let Some(application) = self.body.static_applications().iter().find(|application| application.expression == id) {
                                let TypeAnnotationExpr::Application { arguments, .. } = &annotation.expr else {
                                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                };
                                work.push(Work::StartStaticApplication(application, arguments));
                            } else {
                                self.complete_checked_type_form(id)?;
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
                        _ => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                    }
                }
                Work::EvaluateTypeForm(annotation) => {
                    let expression = self.expression_id(annotation.range)?;
                    if let Some(application) = self.body.static_applications().iter().find(|item| item.expression == expression) {
                        let TypeAnnotationExpr::Application { arguments, .. } = &annotation.expr else {
                            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                        };
                        work.push(Work::StartStaticApplication(application, arguments));
                    } else {
                        self.complete_checked_type_form(expression)?;
                    }
                }
                Work::EvaluateBlock(block) => {
                    work.push(Work::BlockNext(block, 0, None));
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
                Work::BlockNext(block, index, last) => match block.body.get(index) {
                    Some(Statement::Expr { expr, .. }) => {
                        work.push(Work::BlockAfter(block, index + 1));
                        work.push(Work::Evaluate(expr));
                    }
                    Some(Statement::Let(binding)) => {
                        let binding_id = self.local_binding(binding)?;
                        if let Some(initializer) = &binding.value {
                            work.push(Work::StoreLocal(binding_id, block, index + 1));
                            work.push(Work::Evaluate(initializer));
                        } else {
                            let slot = self.local_slot(binding_id)?;
                            self.locals[slot] = None;
                            work.push(Work::BlockNext(block, index + 1, Some(self.values.intern_unit())));
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
                    Some(_) => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
                    None => stack.push(last.unwrap_or_else(|| self.values.intern_unit())),
                },
                Work::BlockAfter(block, next) => {
                    let value = stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    work.push(Work::BlockNext(block, next, Some(value)));
                }
                Work::StoreLocal(binding, block, next) => {
                    let value = stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let slot = self.local_slot(binding)?;
                    self.locals[slot] = Some(value);
                    work.push(Work::BlockNext(block, next, Some(self.values.intern_unit())));
                }
                Work::FinishReturn => {
                    let result = if let Some(term) = self.completed_kinded_result.take() {
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
                Work::FinishTuple(count) => {
                    let Some(start) = stack.len().checked_sub(count) else {
                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                    };
                    let elements = stack.split_off(start);
                    stack.push(self.values.intern_tuple(elements).map_err(Stop::from)?);
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
                    let (types, data, enums) = self.semantic_tables.ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
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
                                    GenericArgumentSyntax::Expansion(_) | GenericArgumentSyntax::Remainder(_) => {
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
                                    GenericArgumentSyntax::Expansion(_) | GenericArgumentSyntax::Remainder(_) => {
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
                Work::EvaluateBinding(expression) => stack.push(self.read_binding_value(expression)?),
                Work::StoreStaticArgument(application, index) => {
                    let value = stack.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let arguments = self
                        .pending_application_arguments
                        .get_mut(&application)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let slot = arguments
                        .get_mut(index as usize)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    *slot = Some(StaticArgument::Value(value));
                }
                Work::StoreStaticKindedArgument(application, index) => {
                    let term = self
                        .completed_kinded_result
                        .take()
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let arguments = self
                        .pending_application_arguments
                        .get_mut(&application)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    let slot = arguments
                        .get_mut(index as usize)
                        .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    *slot = Some(StaticArgument::Kinded(term));
                }
                Work::EnterStaticApplication(application) => {
                    if self.active_definitions.contains(&application.target) {
                        return Err(Stop::Recursive);
                    }
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
                    self.validate_application_arguments(&product.definition.signature, &args)?;
                    if target_body.result != product.definition.signature.result_domain {
                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                    }
                    self.save_current_frame();
                    self.load_frame(target_body, &args)?;
                    self.active_definitions.push(application.target.clone());
                    self.push_trace(StaticEvalTraceFrame::Definition(application.target.clone()))?;
                    work.push(Work::FinishStaticApplication(application.id.clone()));
                    match &target_body.root {
                        crate::static_definition::StaticDefinitionBodyRepr::Expression(expression) => work.push(Work::Evaluate(expression)),
                        crate::static_definition::StaticDefinitionBodyRepr::Block(block) => work.push(Work::EvaluateBlock(block)),
                        crate::static_definition::StaticDefinitionBodyRepr::TypeForm(annotation) => work.push(Work::EvaluateTypeForm(annotation)),
                    }
                }
                Work::FinishStaticApplication(application) => {
                    if self.pending_application_arguments.contains_key(&application) {
                        return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                    }
                    self.trace.pop();
                    self.trace.pop();
                    let control = pending_control.take();
                    let disposition = control.map(|control| control_disposition(ControlBoundary::StaticApplication, control));
                    let validated_result = match disposition {
                        Some(ControlDisposition::ConsumeReturn(result)) => Some(self.validate_result_domain(result)),
                        _ => None,
                    };
                    self.restore_frame()?;
                    if let Some(result) = validated_result {
                        match result? {
                            StaticEvalResult::Value(value) => stack.push(value),
                            StaticEvalResult::KindedTerm(term) => {
                                if self.completed_kinded_result.replace(term).is_some() {
                                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                                }
                            }
                        }
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
            if !stack.is_empty() || self.completed_kinded_result.is_some() {
                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
            }
            let result = self.validate_result_domain(result)?;
            if self.cancellation.is_cancelled() {
                return Err(Stop::Cancelled);
            }
            return Ok(result);
        }
        let result = if let Some(term) = self.completed_kinded_result.take() {
            if !stack.is_empty() {
                return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
            }
            StaticEvalResult::KindedTerm(term)
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
            StaticEvalResult::KindedTerm(term) => {
                let Some(expected_kind) = self.body.result_as_kind() else {
                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                };
                let types = self.canonical_types.ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                if types.kind_of(term) != expected_kind {
                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                }
                Ok(result)
            }
            _ => Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
        }
    }

    fn complete_checked_type_form(&mut self, expression: ExpressionId) -> Result<(), Stop> {
        let analysis = self
            .body
            .expressions()
            .get(&expression)
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        let form = match analysis.denotation.as_ref() {
            Some(crate::types::denotation::SemanticDenotation::TypeForm(form) | crate::types::denotation::SemanticDenotation::StaticTerm(form)) => *form,
            _ => return Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
        };
        let expected_kind = self
            .body
            .result_as_kind()
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        let types = self.canonical_types.ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;

        // A checked parameter by itself can be forwarded without constructing
        // or interning any replacement type. Composite substitution is outside
        // this immutable-snapshot evaluator contract.
        if let crate::types::store::TypeData::Parameter(parameter) = types.get(form) {
            if let Some(index) = self.body.kinded_parameter_index(*parameter) {
                let forwarded = *self
                    .kinded_arguments
                    .get(&index)
                    .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                if types.kind_of(forwarded) != expected_kind {
                    return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
                }
                self.completed_kinded_result = Some(forwarded);
                return Ok(());
            }
        }
        if self.body.contains_kinded_parameter(types, form) || types.kind_of(form) != expected_kind {
            return Err(Stop::invalid(StaticEvalError::UnsupportedExpression));
        }
        self.completed_kinded_result = Some(form);
        Ok(())
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
                    let types = self.canonical_types.ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
                    if types.kind_of(*term) != expected {
                        return Err(Stop::invalid(StaticEvalError::InvalidArgumentKind { index: parameter.index }));
                    }
                }
                _ => return Err(Stop::invalid(StaticEvalError::InvalidArgumentKind { index: parameter.index })),
            }
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

    fn read_binding_value(&self, id: ExpressionId) -> Result<StaticValueRef, Stop> {
        let binding = self
            .body
            .expressions()
            .get(&id)
            .and_then(|entry| entry.binding_read)
            .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        if let Some(value) = self.parameter_values.get(&binding) {
            return Ok(*value);
        }
        if let Some(symbol) = self.body.static_constant_bindings().get(&binding) {
            if let Some(constants) = self.constants {
                let const_id = crate::static_constant::StaticConstantId(symbol.clone());
                if let Some(constant) = constants.get(&const_id) {
                    if let Some(value) = constant.value {
                        return Ok(value);
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
            kinded_arguments: std::mem::take(&mut self.kinded_arguments),
            expression_sites: std::mem::take(&mut self.expression_sites),
            definition,
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
        let mut kinded_arguments = BTreeMap::new();
        for (index, binding) in body.parameter_bindings.iter() {
            match arguments.get(*index as usize) {
                Some(StaticArgument::Value(value)) => {
                    parameter_values.insert(*binding, *value);
                }
                _ => return Err(Stop::invalid(StaticEvalError::InvalidArgumentKind { index: *index })),
            }
        }
        for index in body.kinded_parameters.keys() {
            match arguments.get(*index as usize) {
                Some(StaticArgument::Kinded(ty)) => {
                    kinded_arguments.insert(*index, *ty);
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
        self.kinded_arguments = kinded_arguments;
        self.expression_sites = expression_sites;
        Ok(())
    }

    fn restore_frame(&mut self) -> Result<(), Stop> {
        self.active_definitions.pop();
        let frame = self.saved_frames.pop().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        self.body = frame.body;
        self.locals = frame.locals;
        self.parameter_values = frame.parameter_values;
        self.kinded_arguments = frame.kinded_arguments;
        self.expression_sites = frame.expression_sites;
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
        let checkpoints = work_units.div_ceil(1024).max(1);
        for _ in 0..checkpoints {
            if self.cancellation.is_cancelled() {
                return Err(Stop::Cancelled);
            }
            self.budget.charge_step().map_err(Stop::Budget)?;
        }
        Ok(())
    }

    fn push_trace(&mut self, frame: StaticEvalTraceFrame) -> Result<(), Stop> {
        if self.cancellation.is_cancelled() {
            return Err(Stop::Cancelled);
        }
        self.budget.charge_step().map_err(Stop::Budget)?;
        self.trace.push(frame);
        Ok(())
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

enum Stop {
    Cancelled,
    Recursive,
    Budget(crate::types::outcome::BudgetReport),
    Invalid(StaticEvalError),
}

impl Stop {
    fn invalid(error: StaticEvalError) -> Self {
        Self::Invalid(error)
    }
}

impl From<crate::static_value::StaticValueStoreError> for Stop {
    fn from(error: crate::static_value::StaticValueStoreError) -> Self {
        Self::invalid(StaticEvalError::ValueStore(error.to_string()))
    }
}
