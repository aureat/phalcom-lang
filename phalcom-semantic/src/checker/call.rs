//! Message send and callable argument verification (Spec 04.5 / E5).

use super::context::{CallCapture, CheckingContext};
use super::expected::{ExpectationOrigin, ExpectedType};
use super::inference::{ConstraintOrigin, InferenceLabeledTypePackElement, InferenceRelation, InferenceSession, InferenceSupport, InferenceTerm};
use super::row_inference::{CombinedInferenceFailure, GenericApplicationSession, term_has_row_variables};
use crate::checker::analysis::AnalysisStatus;
use crate::checker::causal::CausalInvalidity;
use crate::checker::incident::{InternalSemanticIncidentDetails, InternalSemanticIncidentKind};
use crate::checker::typed_expr::TypedExpression;
use crate::data_semantics::{DataConstructorParameterContribution, DataConstructorTemplate, DataShape};
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::dispatch::{
    CallableParameter, CallableParameterContract, CallableSemanticKind, CallableSignature, DispatchSignatureSpecialization, RestValueContractKnowledge,
};
use crate::identity::{CallableId, ExplanationId};
use crate::types::evidence::DynamicReason;
use crate::types::evidence::{EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason, join_type_knowledge};
use crate::types::id::{KindId, RecordRowId, ScopedTypeId, StaticDomainId, TypeId, TypeLambdaId, TypeParameterId};
use crate::types::instantiation::TypeMaterializationError;
use crate::types::parameter::{GenericConstraint, GenericSignature, TypeParameterOwner, TypeTerm};
use crate::types::row::{RecordRowFormationError, RecordRowTail};
use crate::types::store::{StaticArgument, StaticDomainData, TypeData, TypeStore};
use phalcom_ast::ast::{ExpansionMode, Expr, PackItem, PackLabel, RestMode};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorSlot};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallTargetAuthority {
    ExactDispatch,
    AbstractContract,
    CallableValue(EvidenceStatus),
    StructuralBuiltin,
}

#[derive(Clone, Debug)]
pub(crate) struct CallableApplicationTarget {
    pub signature: CallableSignature,
    pub callable: Option<CallableId>,
    pub target: Option<crate::identity::InvocationTargetId>,
    pub authority: CallTargetAuthority,
    pub specialization: Option<DispatchSignatureSpecialization>,
    pub fixed_generics: Vec<(TypeParameterId, TypeId)>,
    /// Declaration-owned generic domain needed by constructor/variant
    /// application. It is intentionally kept separate from the callable's
    /// canonical local signature.
    pub declaration_generics: Option<GenericSignature>,
    /// Shape-aware application contract for a product data constructor.
    /// Fixed formals remain in `signature.parameters`; expansion operands are
    /// consumed by the shared inference coordinator through this adapter.
    pub data_constructor_shape: Option<DataConstructorShapeAdapter>,
    pub access_denied: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct DataConstructorShapeAdapter {
    shape: DataShape,
    template: DataConstructorTemplate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DataConstructorArgumentRole {
    Fixed {
        parameter_index: usize,
    },
    PositionalPack {
        parameter: TypeParameterId,
    },
    LabeledPack {
        parameter: TypeParameterId,
        label: Box<str>,
    },
    RecordField {
        parameter: TypeParameterId,
        label: Box<str>,
    },
    Expansion {
        parameter: TypeParameterId,
        mode: phalcom_ast::ast::ExpansionMode,
    },
}

#[derive(Default)]
struct DataConstructorExpansionTerms {
    parameter: Option<TypeParameterId>,
    complete: bool,
    positional: Vec<InferenceTerm>,
    labeled: Vec<crate::checker::inference::InferenceLabeledTypePackElement>,
    fields: Vec<crate::checker::row_inference::InferenceRecordField>,
    premises: Vec<DataConstructorExpansionPremise>,
}

struct DataConstructorExpansionPremise {
    parameter: TypeParameterId,
    argument: SourceRange,
    knowledge: TypeKnowledge,
    explanation: Option<ExplanationId>,
    record_field: bool,
    proof_term: Option<InferenceTerm>,
}

impl DataConstructorShapeAdapter {
    pub(crate) fn new(shape: DataShape, template: DataConstructorTemplate) -> Self {
        Self { shape, template }
    }

    fn expansion(&self) -> Option<&crate::data_semantics::DataProductExpansionSemantic> {
        let mut expansions = self.template.parameters.iter().filter_map(|contribution| match contribution {
            DataConstructorParameterContribution::Expansion(expansion) => Some(expansion),
            DataConstructorParameterContribution::Parameter(_) => None,
        });
        let expansion = expansions.next()?;
        expansions.next().is_none().then_some(expansion)
    }

    fn fixed_record_field_names(&self) -> Vec<crate::field_key::FieldKey> {
        self.template
            .parameters
            .iter()
            .filter_map(|contribution| match contribution {
                DataConstructorParameterContribution::Parameter(parameter) if self.shape == DataShape::Record => Some(crate::field_key::FieldKey::symbol(
                    parameter.external_label.as_ref().unwrap_or(&parameter.local_name).clone(),
                )),
                _ => None,
            })
            .collect()
    }

    fn bind_arguments(
        &self,
        arguments: &[ApplicationArgument<'_>],
        parameters: &[CallableParameter],
        store: &TypeStore,
    ) -> Result<Vec<Option<DataConstructorArgumentRole>>, Vec<ArgumentShapeFailure>> {
        let expansions = self
            .template
            .parameters
            .iter()
            .filter_map(|contribution| match contribution {
                DataConstructorParameterContribution::Expansion(expansion) => Some(expansion),
                DataConstructorParameterContribution::Parameter(_) => None,
            })
            .collect::<Vec<_>>();
        if expansions.len() != 1 {
            return Err(vec![ArgumentShapeFailure::DynamicShape]);
        }
        let expansion = expansions[0];
        let Some(expansion_parameter) = data_expansion_parameter(expansion, self.shape, store) else {
            return Err(vec![ArgumentShapeFailure::DynamicShape]);
        };

        let fixed = self
            .template
            .parameters
            .iter()
            .filter_map(|contribution| match contribution {
                DataConstructorParameterContribution::Parameter(parameter) => Some(parameter),
                DataConstructorParameterContribution::Expansion(_) => None,
            })
            .collect::<Vec<_>>();
        if fixed.len() != parameters.len() {
            return Err(vec![ArgumentShapeFailure::DynamicShape]);
        }

        let mut roles = vec![None; arguments.len()];
        let mut fixed_by_label = std::collections::HashMap::<String, usize>::new();
        for (parameter_index, parameter) in fixed.iter().enumerate() {
            let label = parameter
                .external_label
                .as_deref()
                .or_else(|| (self.shape == DataShape::Record).then_some(parameter.local_name.as_ref()));
            if let Some(label) = label {
                fixed_by_label.insert(label.to_string(), parameter_index);
            }
        }
        let mut bound_fixed = vec![false; fixed.len()];
        let mut failures = Vec::new();

        let mut positional_arguments = Vec::new();
        for (argument_index, argument) in arguments.iter().copied().enumerate() {
            match argument {
                ApplicationArgument::Positional { .. } | ApplicationArgument::PreAnalyzed { label: None, .. } => {
                    if self.shape == DataShape::Record {
                        failures.push(ArgumentShapeFailure::UnexpectedPositional { argument_index });
                    } else {
                        positional_arguments.push(argument_index);
                    }
                }
                ApplicationArgument::Labeled { label, .. } | ApplicationArgument::PreAnalyzed { label: Some(label), .. } => {
                    if let Some(&parameter_index) = fixed_by_label.get(label) {
                        if std::mem::replace(&mut bound_fixed[parameter_index], true) {
                            failures.push(ArgumentShapeFailure::DuplicateParameterBinding { parameter_index });
                        } else {
                            roles[argument_index] = Some(DataConstructorArgumentRole::Fixed { parameter_index });
                        }
                    } else {
                        let role = if self.shape == DataShape::Record {
                            Some(DataConstructorArgumentRole::RecordField {
                                parameter: expansion_parameter,
                                label: label.into(),
                            })
                        } else if matches!(
                            expansion.mode,
                            phalcom_ast::ast::ExpansionMode::Labeled | phalcom_ast::ast::ExpansionMode::Complete
                        ) {
                            Some(DataConstructorArgumentRole::LabeledPack {
                                parameter: expansion_parameter,
                                label: label.into(),
                            })
                        } else {
                            None
                        };
                        if let Some(role) = role {
                            roles[argument_index] = Some(role);
                        } else {
                            failures.push(ArgumentShapeFailure::UnknownLabel {
                                argument_index,
                                label: label.to_string(),
                            });
                        }
                    }
                }
                ApplicationArgument::DynamicLabel { .. } => {
                    failures.push(ArgumentShapeFailure::DynamicShape);
                }
                ApplicationArgument::Expansion { mode, .. } => {
                    let has_fixed_positional = self
                        .template
                        .parameters
                        .iter()
                        .any(|c| matches!(c, DataConstructorParameterContribution::Parameter(p) if p.external_label.is_none()));
                    let mode_compatible = match (self.shape, expansion.mode, mode) {
                        (DataShape::Record, _, phalcom_ast::ast::ExpansionMode::Positional) => false,
                        (DataShape::Record, _, _) => true,
                        (DataShape::Tuple, phalcom_ast::ast::ExpansionMode::Complete, _) => true,
                        (DataShape::Tuple, phalcom_ast::ast::ExpansionMode::Positional, phalcom_ast::ast::ExpansionMode::Positional) => true,
                        (DataShape::Tuple, phalcom_ast::ast::ExpansionMode::Labeled, phalcom_ast::ast::ExpansionMode::Labeled) => true,
                        _ => false,
                    };
                    if mode_compatible && !has_fixed_positional {
                        roles[argument_index] = Some(DataConstructorArgumentRole::Expansion {
                            parameter: expansion_parameter,
                            mode,
                        });
                    } else if self.shape == DataShape::Record && mode == phalcom_ast::ast::ExpansionMode::Positional {
                        failures.push(ArgumentShapeFailure::UnexpectedPositional { argument_index });
                    } else {
                        failures.push(ArgumentShapeFailure::DynamicShape);
                    }
                }
            }
        }

        if self.shape == DataShape::Tuple {
            let expansion_contribution = self
                .template
                .parameters
                .iter()
                .position(|contribution| matches!(contribution, DataConstructorParameterContribution::Expansion(_)))
                .unwrap_or(0);
            let positional_prefix = self
                .template
                .parameters
                .iter()
                .take(expansion_contribution)
                .filter_map(|contribution| match contribution {
                    DataConstructorParameterContribution::Parameter(parameter) if parameter.external_label.is_none() => {
                        fixed.iter().position(|candidate| candidate.component == parameter.component)
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            let positional_suffix = self
                .template
                .parameters
                .iter()
                .skip(expansion_contribution + 1)
                .filter_map(|contribution| match contribution {
                    DataConstructorParameterContribution::Parameter(parameter) if parameter.external_label.is_none() => {
                        fixed.iter().position(|candidate| candidate.component == parameter.component)
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            let accepts_positionals = matches!(
                expansion.mode,
                phalcom_ast::ast::ExpansionMode::Positional | phalcom_ast::ast::ExpansionMode::Complete
            );
            let unbound = |indices: &[usize], bound: &[bool]| indices.iter().copied().filter(|index| !bound[*index]).collect::<Vec<_>>();
            let prefix = unbound(&positional_prefix, &bound_fixed);
            let suffix = unbound(&positional_suffix, &bound_fixed);

            if accepts_positionals {
                if positional_arguments.len() >= prefix.len() + suffix.len() {
                    for (&argument_index, &parameter_index) in positional_arguments.iter().zip(prefix.iter()) {
                        bound_fixed[parameter_index] = true;
                        roles[argument_index] = Some(DataConstructorArgumentRole::Fixed { parameter_index });
                    }
                    let suffix_start = positional_arguments.len() - suffix.len();
                    for (&argument_index, &parameter_index) in positional_arguments[suffix_start..].iter().zip(suffix.iter()) {
                        bound_fixed[parameter_index] = true;
                        roles[argument_index] = Some(DataConstructorArgumentRole::Fixed { parameter_index });
                    }
                    for &argument_index in &positional_arguments[prefix.len()..suffix_start] {
                        roles[argument_index] = Some(DataConstructorArgumentRole::PositionalPack {
                            parameter: expansion_parameter,
                        });
                    }
                } else {
                    let fixed_order = prefix.into_iter().chain(suffix).collect::<Vec<_>>();
                    for (&argument_index, &parameter_index) in positional_arguments.iter().zip(fixed_order.iter()) {
                        bound_fixed[parameter_index] = true;
                        roles[argument_index] = Some(DataConstructorArgumentRole::Fixed { parameter_index });
                    }
                }
            } else {
                let fixed_order = positional_prefix
                    .into_iter()
                    .chain(positional_suffix)
                    .filter(|index| !bound_fixed[*index])
                    .collect::<Vec<_>>();
                let mut available = fixed_order.into_iter();
                for &argument_index in &positional_arguments {
                    if let Some(parameter_index) = available.next() {
                        bound_fixed[parameter_index] = true;
                        roles[argument_index] = Some(DataConstructorArgumentRole::Fixed { parameter_index });
                    } else {
                        failures.push(ArgumentShapeFailure::UnexpectedPositional { argument_index });
                    }
                }
            }
        }

        for (parameter_index, bound) in bound_fixed.into_iter().enumerate() {
            if !bound {
                failures.push(ArgumentShapeFailure::MissingRequiredParameter { parameter_index });
            }
        }

        // Duplicate labels in expanded lanes are invalid pack/row shapes.
        let mut expanded_labels = std::collections::HashSet::<(TypeParameterId, Box<str>)>::new();
        for (argument_index, role) in roles.iter().enumerate() {
            let Some(DataConstructorArgumentRole::LabeledPack { parameter, label } | DataConstructorArgumentRole::RecordField { parameter, label }) = role
            else {
                continue;
            };
            if !expanded_labels.insert((*parameter, label.clone())) {
                failures.push(ArgumentShapeFailure::DuplicateLabel {
                    argument_index,
                    label: label.to_string(),
                });
            }
        }

        if failures.is_empty() { Ok(roles) } else { Err(failures) }
    }

    pub(crate) fn argument_to_components(
        &self,
        arguments: &[ApplicationArgument<'_>],
        parameters: &[CallableParameter],
        store: &TypeStore,
        shape: &crate::data_semantics::DataProductShapeMaterialization,
    ) -> Option<Box<[u32]>> {
        if shape.state != crate::data_semantics::DataProductShapeState::Concrete {
            return None;
        }
        let roles = self.bind_arguments(arguments, parameters, store).ok()?;
        let fixed = self
            .template
            .parameters
            .iter()
            .filter_map(|contribution| match contribution {
                DataConstructorParameterContribution::Parameter(parameter) => Some(parameter),
                DataConstructorParameterContribution::Expansion(_) => None,
            })
            .collect::<Vec<_>>();
        let expansion = self.expansion()?;
        let mut positional_components = shape.components.iter().enumerate().filter_map(|(index, component)| {
            (component.component.is_none() && component.label.is_none() && component.expansion_source == expansion.source).then_some(index as u32)
        });
        let mut mapping = Vec::with_capacity(arguments.len());
        let mut seen = std::collections::HashSet::new();
        for role in roles {
            let role = role?;
            let component_index = match role {
                DataConstructorArgumentRole::Fixed { parameter_index } => {
                    let component = &fixed.get(parameter_index)?.component;
                    shape.components.iter().position(|candidate| candidate.component.as_ref() == Some(component))? as u32
                }
                DataConstructorArgumentRole::PositionalPack { .. } => positional_components.next()?,
                DataConstructorArgumentRole::LabeledPack { label, .. } | DataConstructorArgumentRole::RecordField { label, .. } => {
                    shape.components.iter().position(|candidate| {
                        candidate.component.is_none() && candidate.label.as_deref() == Some(label.as_ref()) && candidate.expansion_source == expansion.source
                    })? as u32
                }
                DataConstructorArgumentRole::Expansion { .. } => return None,
            };
            if !seen.insert(component_index) {
                return None;
            }
            mapping.push(component_index);
        }
        Some(mapping.into_boxed_slice())
    }
}

fn data_expansion_parameter(expansion: &crate::data_semantics::DataProductExpansionSemantic, shape: DataShape, store: &TypeStore) -> Option<TypeParameterId> {
    let operand = expansion.operand.canonical_type()?;
    match (shape, store.get(operand)) {
        (DataShape::Tuple, TypeData::Parameter(parameter)) => Some(*parameter),
        (DataShape::Record, TypeData::Record(row)) => match store.record_row(*row).tail {
            crate::types::row::RecordRowTail::Parameter(parameter) => Some(parameter),
            _ => None,
        },
        _ => None,
    }
}

fn extract_pack_expansion_elements(store: &TypeStore, ty: TypeId) -> Option<(Vec<TypeId>, Vec<(Box<str>, TypeId)>)> {
    match store.get(ty) {
        TypeData::Pack(pack) if pack.is_closed() => {
            let mut positionals = Vec::new();
            let mut labeled = Vec::new();
            for element in pack.elements.iter() {
                if let Some(label) = &element.label {
                    labeled.push((label.clone(), element.ty));
                } else {
                    positionals.push(element.ty);
                }
            }
            Some((positionals, labeled))
        }
        TypeData::Tuple(tuple) if tuple.is_closed() => {
            let mut positionals = Vec::new();
            let mut labeled = Vec::new();
            for element in tuple.elements.iter() {
                if let Some(label) = &element.label {
                    labeled.push((label.clone(), element.ty));
                } else {
                    positionals.push(element.ty);
                }
            }
            Some((positionals, labeled))
        }
        TypeData::Record(row_id) => {
            let row = store.record_row(*row_id);
            if !matches!(row.tail, RecordRowTail::Closed) {
                return None;
            }
            let mut labeled = Vec::new();
            for field in row.fields.iter() {
                if let crate::field_key::FieldKey::Symbol(name) = &field.key {
                    labeled.push((name.as_ref().into(), field.ty));
                }
            }
            Some((Vec::new(), labeled))
        }
        TypeData::TypePack(elements) => Some((elements.to_vec(), Vec::new())),
        TypeData::LabeledTypePack(elements) => {
            let labeled = elements.iter().map(|e| (e.label.clone(), e.element)).collect();
            Some((Vec::new(), labeled))
        }
        TypeData::CompleteTypePack { positionals, labeled }
        | TypeData::PackFromPacks { positionals, labeled }
        | TypeData::TupleFromPacks { positionals, labeled } => {
            let (pos, _) = extract_pack_expansion_elements(store, *positionals)?;
            let (_, lab) = extract_pack_expansion_elements(store, *labeled)?;
            Some((pos, lab))
        }
        TypeData::Unit => Some((Vec::new(), Vec::new())),
        _ => None,
    }
}

/// Dispatch result for one canonical member of a union receiver.
#[derive(Clone, Debug)]
pub(crate) enum UnionCallArm {
    Found {
        receiver: TypeId,
        target: Box<CallableApplicationTarget>,
    },
    Missing {
        receiver: TypeId,
        visited_owners: Box<[crate::identity::DeclarationId]>,
    },
    Ambiguous {
        receiver: TypeId,
    },
    Dynamic {
        receiver: TypeId,
        reason: DynamicReason,
    },
    TraitTerminal {
        receiver: TypeId,
        terminal: crate::trait_dispatch::TraitDispatchTerminal,
    },
}

impl CallableApplicationTarget {
    pub(crate) fn exact(callable: CallableId, signature: CallableSignature) -> Self {
        Self {
            signature,
            callable: Some(callable.clone()),
            target: Some(crate::identity::InvocationTargetId::Behavioral(callable)),
            authority: CallTargetAuthority::ExactDispatch,
            specialization: None,
            fixed_generics: Vec::new(),
            declaration_generics: None,
            data_constructor_shape: None,
            access_denied: false,
        }
    }

    pub(crate) fn variant_constructor(variant: crate::identity::VariantId, signature: CallableSignature) -> Self {
        Self {
            signature,
            callable: None,
            target: Some(crate::identity::InvocationTargetId::variant_constructor(variant)),
            authority: CallTargetAuthority::ExactDispatch,
            specialization: None,
            fixed_generics: Vec::new(),
            declaration_generics: None,
            data_constructor_shape: None,
            access_denied: false,
        }
    }

    pub(crate) fn data_constructor(data_constructor: crate::identity::DataConstructorId, signature: CallableSignature) -> Self {
        Self {
            signature,
            callable: None,
            target: Some(crate::identity::InvocationTargetId::DataConstructor(data_constructor)),
            authority: CallTargetAuthority::ExactDispatch,
            specialization: None,
            fixed_generics: Vec::new(),
            declaration_generics: None,
            data_constructor_shape: None,
            access_denied: false,
        }
    }

    pub(crate) fn with_fixed_generics(mut self, fixed_generics: Vec<(TypeParameterId, TypeId)>) -> Self {
        self.fixed_generics = fixed_generics;
        self
    }

    pub(crate) fn with_declaration_generics(mut self, declaration_generics: GenericSignature) -> Self {
        self.declaration_generics = Some(declaration_generics);
        self
    }

    pub(crate) fn with_data_constructor_shape(mut self, adapter: DataConstructorShapeAdapter) -> Self {
        self.data_constructor_shape = Some(adapter);
        self
    }

    pub(crate) fn from_dispatch(resolved: Box<crate::dispatch::ResolvedDispatch>) -> Self {
        let mut target = if resolved.abstract_contract {
            Self::abstract_contract(resolved.callable, resolved.signature)
        } else {
            Self::exact(resolved.callable, resolved.signature)
        };
        target.specialization = resolved.specialization;
        target.access_denied = resolved.access_denied;
        target
    }

    pub(crate) fn callable_value(signature: CallableSignature, status: EvidenceStatus) -> Self {
        Self {
            signature,
            callable: None,
            target: None,
            authority: CallTargetAuthority::CallableValue(status),
            specialization: None,
            fixed_generics: Vec::new(),
            declaration_generics: None,
            data_constructor_shape: None,
            access_denied: false,
        }
    }

    pub(crate) fn abstract_contract(callable: CallableId, signature: CallableSignature) -> Self {
        Self {
            signature,
            callable: Some(callable),
            target: None,
            authority: CallTargetAuthority::AbstractContract,
            specialization: None,
            fixed_generics: Vec::new(),
            declaration_generics: None,
            data_constructor_shape: None,
            access_denied: false,
        }
    }

    pub(crate) fn structural(signature: CallableSignature) -> Self {
        Self {
            signature,
            callable: None,
            target: None,
            authority: CallTargetAuthority::StructuralBuiltin,
            specialization: None,
            fixed_generics: Vec::new(),
            declaration_generics: None,
            data_constructor_shape: None,
            access_denied: false,
        }
    }
}

/// Builds the canonical application target for a structural callable value.
///
/// Both direct invocation and explicit `.call()` invocation use this target so
/// argument binding, generic proof, status propagation, and return publication
/// remain identical.
pub(crate) fn callable_value_target(store: &TypeStore, callable_ty: TypeId, authority: EvidenceStatus) -> Option<CallableApplicationTarget> {
    let TypeData::Callable(callable) = store.get(callable_ty) else {
        return None;
    };

    let mut parameters = Vec::with_capacity(callable.parameters.len());
    let mut slots = Vec::with_capacity(callable.parameters.len());
    for parameter in callable.parameters.iter() {
        let mut formal = CallableParameter::new("argument", TypeKnowledge::assumed(parameter.ty, EvidenceOrigin::CallableSignature)).with_rest(parameter.rest);
        if let Some(label) = &parameter.label {
            formal = formal.with_label(label.to_string());
            if parameter.rest == RestMode::None {
                slots.push(SelectorSlot::Label(label.to_string()));
            }
        } else if parameter.rest == RestMode::None {
            slots.push(SelectorSlot::Positional);
        }
        parameters.push(formal);
    }

    let signature = CallableSignature::new(
        Selector::method("call", slots).ok()?,
        parameters,
        TypeKnowledge::assumed(callable.return_type, EvidenceOrigin::CallableSignature),
    );
    Some(CallableApplicationTarget::callable_value(signature, authority))
}

#[derive(Clone, Debug)]
pub(crate) struct CallPremise {
    pub knowledge: TypeKnowledge,
    pub status: AnalysisStatus,
    pub causal_invalidity: CausalInvalidity,
    pub explanation: Option<ExplanationId>,
}

impl CallPremise {
    pub(crate) fn from_typed(ctx: &CheckingContext<'_>, expression: &crate::checker::typed_expr::TypedExpression) -> Self {
        Self {
            knowledge: expression.knowledge.clone(),
            status: expression.status.clone(),
            causal_invalidity: expression.causal_invalidity,
            explanation: expression.expression_id.and_then(|id| ctx.explanation_for_expression(id)),
        }
    }

    pub(crate) fn established(knowledge: TypeKnowledge) -> Self {
        Self {
            knowledge,
            status: AnalysisStatus::Ready,
            causal_invalidity: CausalInvalidity::Clean,
            explanation: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ApplicationArgument<'a> {
    Positional {
        expression: &'a Expr,
        range: SourceRange,
    },
    Labeled {
        label: &'a str,
        expression: &'a Expr,
        range: SourceRange,
    },
    DynamicLabel {
        expression: &'a Expr,
        range: SourceRange,
    },
    Expansion {
        expression: &'a Expr,
        mode: ExpansionMode,
        range: SourceRange,
    },
    /// Source operand analyzed before target selection because bilateral
    /// dispatch needs both operand facts. Canonical application records the
    /// existing child product in its call frame instead of traversing it again.
    PreAnalyzed {
        label: Option<&'a str>,
        typed: &'a TypedExpression,
        range: SourceRange,
    },
}

impl<'a> ApplicationArgument<'a> {
    pub(crate) fn label(&self) -> Option<&'a str> {
        match self {
            Self::Labeled { label, .. } => Some(label),
            Self::PreAnalyzed { label, .. } => *label,
            _ => None,
        }
    }

    pub(crate) fn expression(self) -> Option<&'a Expr> {
        match self {
            Self::Positional { expression, .. }
            | Self::Labeled { expression, .. }
            | Self::DynamicLabel { expression, .. }
            | Self::Expansion { expression, .. } => Some(expression),
            Self::PreAnalyzed { .. } => None,
        }
    }

    pub(crate) fn range(self) -> SourceRange {
        match self {
            Self::Positional { range, .. }
            | Self::Labeled { range, .. }
            | Self::DynamicLabel { range, .. }
            | Self::Expansion { range, .. }
            | Self::PreAnalyzed { range, .. } => range,
        }
    }
}

fn analyze_application_argument(ctx: &mut CheckingContext<'_>, argument: ApplicationArgument<'_>, expected: &ExpectedType) -> TypedExpression {
    analyze_application_argument_owned(ctx, argument, expected, None)
}

fn analyze_application_argument_owned(
    ctx: &mut CheckingContext<'_>,
    argument: ApplicationArgument<'_>,
    expected: &ExpectedType,
    owner: Option<super::sequence_publication::PublicationOwner>,
) -> TypedExpression {
    let mut dependencies = if owner.is_some() { ctx.sequence_dependencies() } else { Default::default() };
    dependencies.extend(owner);
    let mut typed = match argument {
        ApplicationArgument::PreAnalyzed { typed, .. } => {
            ctx.record_call_dependency(typed.causal_invalidity, typed.expression_id.and_then(|id| ctx.explanation_for_expression(id)));
            (*typed).clone()
        }
        _ => super::expression::analyze_expression_with_publication(
            ctx,
            argument.expression().expect("ordinary application argument has an expression"),
            expected,
            dependencies,
        ),
    };
    ctx.record_sequence_dependencies(&typed.publication_dependencies);
    // Solver-local expectations are still inside the generic application
    // boundary. Preserve their local rigid evidence for inference and defer
    // escape checking until the resulting call crosses a concrete boundary.
    let concrete_expectation = !matches!(expected, ExpectedType::Inference { .. });
    if concrete_expectation {
        if let (Some(ty), Some(local_type)) = (typed.knowledge.ty(), typed.local_type.clone()) {
            ctx.record_call_local_type(ty, local_type);
            if !ctx.check_local_type_escape(typed.local_type.as_ref(), expected.ty(), &[], argument.range()) {
                typed.knowledge = TypeKnowledge::Unknown(UnknownReason::ExistentialEscape);
                typed.local_type = None;
            }
        }
    } else if let (Some(ty), Some(local_type)) = (typed.knowledge.ty(), typed.local_type.clone()) {
        ctx.record_call_local_type(ty, local_type);
    }
    if let ApplicationArgument::Expansion { mode, range, .. } = argument
        && matches!(mode, ExpansionMode::Labeled | ExpansionMode::Complete)
        && let Some(key) = crate::checker::composition::known_non_symbol_record_key(ctx.store, &typed.knowledge)
    {
        let diagnostic = SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::RecordSpreadNonSymbolCallKey,
            format!("Record key `{key}` cannot enter a Symbol-only ordered labeled lane"),
            range,
        );
        ctx.emit_diagnostic(diagnostic);
    }
    if let ApplicationArgument::Expansion { range, .. } = argument {
        if let TypeKnowledge::Known(evidence) = &typed.knowledge {
            let (resolution, _) = crate::checker::iteration::resolve_positional_expansion(ctx, typed.expression_id, evidence.ty(), range);
            if let Some(resolution) = resolution {
                ctx.positional_expansions
                    .insert_source(resolution.source_expression, resolution)
                    .expect("checking context positional expansions are source mutable");
            } else {
                crate::checker::iteration::record_known_invalid_positional_expansion(ctx, typed.expression_id, evidence.ty(), range);
            }
        }
    }
    typed
}

pub(crate) fn application_arguments(args: &[PackItem]) -> Vec<ApplicationArgument<'_>> {
    args.iter()
        .map(|item| match item {
            PackItem::Positional { expr, range } => ApplicationArgument::Positional {
                expression: expr,
                range: *range,
            },
            PackItem::Labeled {
                label: PackLabel::Static { text, .. },
                value,
                range,
            } => ApplicationArgument::Labeled {
                label: text.as_str(),
                expression: value,
                range: *range,
            },
            PackItem::Labeled { value, range, .. } => ApplicationArgument::DynamicLabel {
                expression: value,
                range: *range,
            },
            PackItem::Expand { expr, mode, range } => ApplicationArgument::Expansion {
                expression: expr,
                mode: *mode,
                range: *range,
            },
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StaticCallShape {
    Exact(Vec<SelectorSlot>),
    Dynamic(DynamicReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArgumentBinding {
    pub argument_index: usize,
    pub parameter_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArgumentBindingPlan {
    pub bindings: Vec<ArgumentBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ArgumentShapeFailure {
    MissingRequiredParameter { parameter_index: usize },
    UnexpectedPositional { argument_index: usize },
    UnknownLabel { argument_index: usize, label: String },
    DuplicateParameterBinding { parameter_index: usize },
    DuplicateLabel { argument_index: usize, label: String },
    DynamicShape,
}

pub(crate) fn static_call_shape(arguments: &[ApplicationArgument<'_>]) -> StaticCallShape {
    let mut slots = Vec::with_capacity(arguments.len());
    for argument in arguments {
        match argument {
            ApplicationArgument::Positional { .. } => slots.push(SelectorSlot::Positional),
            ApplicationArgument::Labeled { label, .. } => slots.push(SelectorSlot::Label((*label).to_string())),
            ApplicationArgument::PreAnalyzed { label: Some(label), .. } => slots.push(SelectorSlot::Label((*label).to_string())),
            ApplicationArgument::PreAnalyzed { label: None, .. } => slots.push(SelectorSlot::Positional),
            ApplicationArgument::DynamicLabel { .. } | ApplicationArgument::Expansion { .. } => {
                return StaticCallShape::Dynamic(DynamicReason::DynamicRestPack);
            }
        }
    }
    StaticCallShape::Exact(slots)
}

pub(crate) fn bind_static_arguments(
    arguments: &[ApplicationArgument<'_>],
    parameters: &[CallableParameter],
) -> Result<ArgumentBindingPlan, Vec<ArgumentShapeFailure>> {
    if arguments
        .iter()
        .any(|argument| matches!(argument, ApplicationArgument::DynamicLabel { .. } | ApplicationArgument::Expansion { .. }))
    {
        return Err(vec![ArgumentShapeFailure::DynamicShape]);
    }

    let mut bindings = Vec::new();
    let mut bound = vec![false; parameters.len()];
    let mut failures = Vec::new();
    let mut positional_cursor = 0;

    let positional_rest_idx = parameters
        .iter()
        .position(|parameter| matches!(parameter.rest, RestMode::Positional | RestMode::Complete));
    let labeled_rest_idx = parameters
        .iter()
        .position(|parameter| matches!(parameter.rest, RestMode::Labeled | RestMode::Complete));

    for (argument_index, argument) in arguments.iter().enumerate() {
        match argument {
            ApplicationArgument::Positional { .. } | ApplicationArgument::PreAnalyzed { label: None, .. } => {
                let mut found_fixed = None;
                while positional_cursor < parameters.len() {
                    let index = positional_cursor;
                    positional_cursor += 1;
                    if parameters[index].rest == RestMode::None && parameters[index].external_label.is_none() && !bound[index] {
                        found_fixed = Some(index);
                        break;
                    }
                }

                if let Some(parameter_index) = found_fixed {
                    bound[parameter_index] = true;
                    bindings.push(ArgumentBinding {
                        argument_index,
                        parameter_index,
                    });
                } else if let Some(rest_idx) = positional_rest_idx {
                    bound[rest_idx] = true;
                    bindings.push(ArgumentBinding {
                        argument_index,
                        parameter_index: rest_idx,
                    });
                } else {
                    failures.push(ArgumentShapeFailure::UnexpectedPositional { argument_index });
                }
            }
            ApplicationArgument::Labeled { label, .. } | ApplicationArgument::PreAnalyzed { label: Some(label), .. } => {
                let found_fixed = parameters
                    .iter()
                    .enumerate()
                    .find(|(_, parameter)| parameter.rest == RestMode::None && parameter.external_label.as_deref() == Some(*label));

                if let Some((parameter_index, _)) = found_fixed {
                    if bound[parameter_index] {
                        failures.push(ArgumentShapeFailure::DuplicateParameterBinding { parameter_index });
                    } else {
                        bound[parameter_index] = true;
                        bindings.push(ArgumentBinding {
                            argument_index,
                            parameter_index,
                        });
                    }
                } else if let Some(rest_idx) = labeled_rest_idx {
                    bound[rest_idx] = true;
                    bindings.push(ArgumentBinding {
                        argument_index,
                        parameter_index: rest_idx,
                    });
                } else {
                    failures.push(ArgumentShapeFailure::UnknownLabel {
                        argument_index,
                        label: (*label).to_string(),
                    });
                }
            }
            ApplicationArgument::DynamicLabel { .. } | ApplicationArgument::Expansion { .. } => {
                failures.push(ArgumentShapeFailure::DynamicShape);
            }
        }
    }

    for (parameter_index, parameter) in parameters.iter().enumerate() {
        if parameter.rest == RestMode::None && !bound[parameter_index] {
            failures.push(ArgumentShapeFailure::MissingRequiredParameter { parameter_index });
        }
    }

    if failures.is_empty() {
        Ok(ArgumentBindingPlan { bindings })
    } else {
        Err(failures)
    }
}

fn target_base_authority(target: &CallableApplicationTarget) -> EvidenceStatus {
    match target.authority {
        CallTargetAuthority::ExactDispatch | CallTargetAuthority::AbstractContract | CallTargetAuthority::StructuralBuiltin => EvidenceStatus::Established,
        CallTargetAuthority::CallableValue(status) => status,
    }
}

fn target_fixed_return_origin(target: &CallableApplicationTarget) -> EvidenceOrigin {
    match target.authority {
        CallTargetAuthority::ExactDispatch | CallTargetAuthority::AbstractContract => match target.signature.kind {
            CallableSemanticKind::Ordinary => EvidenceOrigin::CallableSignature,
            CallableSemanticKind::Constructor => EvidenceOrigin::ConstructorSemantics,
            CallableSemanticKind::Native => EvidenceOrigin::NativeSignature,
        },
        // A structural callable value is executable flow, not a nominal
        // declaration surface. Its result is therefore published as a flow
        // fact while retaining the callable's authority cap.
        CallTargetAuthority::CallableValue(_) => EvidenceOrigin::Flow,
        CallTargetAuthority::StructuralBuiltin => EvidenceOrigin::DeclarationSemantics,
    }
}

fn weaken_known_to_status(knowledge: TypeKnowledge, maximum: EvidenceStatus, origin: EvidenceOrigin, range: SourceRange) -> TypeKnowledge {
    match knowledge {
        TypeKnowledge::Known(evidence) => {
            let status = evidence.status().meet(maximum);
            TypeKnowledge::Known(evidence).with_status_and_origin(status, origin, range)
        }
        other => other.with_range(range),
    }
}

fn derive_fixed_return(target: &CallableApplicationTarget, premise: &CallPremise, range: SourceRange) -> TypeKnowledge {
    let Some(premise_status) = premise.knowledge.status() else {
        return premise.knowledge.clone();
    };
    let authority = match target.authority {
        CallTargetAuthority::ExactDispatch | CallTargetAuthority::AbstractContract => {
            let return_status = target.signature.return_type.status().unwrap_or(EvidenceStatus::Assumed);
            target_base_authority(target).meet(premise_status).meet(return_status)
        }
        CallTargetAuthority::CallableValue(status) => status.meet(premise_status),
        CallTargetAuthority::StructuralBuiltin => target_base_authority(target).meet(premise_status),
    };
    let origin = target_fixed_return_origin(target);
    let return_type = match &target.signature.return_type {
        TypeKnowledge::Known(evidence) => TypeKnowledge::established(evidence.ty(), origin),
        other => other.clone(),
    };
    weaken_known_to_status(return_type, authority, origin, range)
}

fn parameter_for_argument<'a>(plan: &ArgumentBindingPlan, argument_index: usize, parameters: &'a [CallableParameter]) -> Option<&'a CallableParameter> {
    let parameter_index = plan.bindings.iter().find(|binding| binding.argument_index == argument_index)?.parameter_index;
    parameters.get(parameter_index)
}

fn shape_failure_message(failure: &ArgumentShapeFailure) -> Option<String> {
    match failure {
        ArgumentShapeFailure::MissingRequiredParameter { parameter_index } => Some(format!("missing required parameter at position {}", parameter_index + 1)),
        ArgumentShapeFailure::UnexpectedPositional { argument_index } => Some(format!("unexpected positional argument at position {}", argument_index + 1)),
        ArgumentShapeFailure::UnknownLabel { label, .. } => Some(format!("unknown argument label `{label}:`")),
        ArgumentShapeFailure::DuplicateLabel { label, .. } => Some(format!("duplicate expanded label `{label}:`")),
        ArgumentShapeFailure::DuplicateParameterBinding { parameter_index } => {
            Some(format!("parameter at position {} is bound more than once", parameter_index + 1))
        }
        ArgumentShapeFailure::DynamicShape => None,
    }
}

fn emit_shape_failures(
    ctx: &mut CheckingContext<'_>,
    failures: &[ArgumentShapeFailure],
    range: SourceRange,
    callable: Option<CallableId>,
    parameters: &[CallableParameter],
) {
    let structured = failures
        .iter()
        .filter_map(|failure| match failure {
            ArgumentShapeFailure::MissingRequiredParameter { parameter_index } => Some(crate::explain::CallShapeExplanation::MissingRequired {
                parameter_index: *parameter_index as u16,
                label: parameters.get(*parameter_index).and_then(|parameter| parameter.external_label.clone()),
            }),
            ArgumentShapeFailure::UnexpectedPositional { argument_index } => Some(crate::explain::CallShapeExplanation::UnexpectedPositional {
                argument_index: *argument_index as u16,
            }),
            ArgumentShapeFailure::UnknownLabel { label, .. } => Some(crate::explain::CallShapeExplanation::UnknownLabel { label: label.clone() }),
            ArgumentShapeFailure::DuplicateLabel { label, .. } => Some(crate::explain::CallShapeExplanation::DuplicateLabel { label: label.clone() }),
            ArgumentShapeFailure::DuplicateParameterBinding { parameter_index } => Some(crate::explain::CallShapeExplanation::DuplicateParameter {
                parameter_index: *parameter_index as u16,
            }),
            ArgumentShapeFailure::DynamicShape => None,
        })
        .collect::<Vec<_>>();
    let explanation = (!structured.is_empty()).then(|| {
        ctx.record_derivation(
            crate::explain::ExplanationStep::CallShape {
                callable: callable.clone(),
                failures: structured.into_boxed_slice(),
            },
            crate::explain::DerivationRule::CallShape,
            EvidenceStatus::Established,
            EvidenceOrigin::DeclarationSemantics,
            vec![crate::explain::EvidenceRef::SourceSpan(range)],
            Vec::new(),
        )
    });
    for failure in failures {
        let Some(message) = shape_failure_message(failure) else {
            continue;
        };
        let mut diagnostic = SemanticDiagnostic::error_in(ctx.current_module.clone(), DiagnosticCode::CallShapeMismatch, message, range);
        if let (Some(owner), Some(explanation)) = (ctx.current_callable.clone(), explanation) {
            diagnostic = diagnostic.with_explanation(crate::diagnostic::ExplanationRef::new(owner, explanation));
        }
        if let Some(callable) = callable.clone() {
            diagnostic = diagnostic.with_guidance(crate::diagnostic::DiagnosticGuidance::UseCallableShape { callable });
        }
        if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
            ctx.record_call_status(AnalysisStatus::Invalid(cause));
        }
    }
}

fn argument_relation_message(argument_index: usize, argument: ApplicationArgument<'_>, parameter: &CallableParameter) -> String {
    match argument {
        ApplicationArgument::Labeled { label, .. } => format!(
            "argument for label `{label}:` does not match expected parameter type `{}`",
            parameter.local_name
        ),
        _ => format!(
            "argument at position {} does not match expected parameter type `{}`",
            argument_index + 1,
            parameter.local_name
        ),
    }
}

fn debug_assert_call_result_coherent(result: &CallCheckResult) {
    if let AnalysisStatus::Invalid(cause) = result.status {
        debug_assert!(result.causal_invalidity.contains(cause));
    }
    if matches!(result.status, AnalysisStatus::Suppressed(_)) {
        debug_assert!(!matches!(result.causal_invalidity, CausalInvalidity::Clean));
    }
}

fn lane_kind(mode: RestMode) -> KindId {
    match mode {
        RestMode::Positional => KindId::TYPE_PACK,
        RestMode::Labeled => KindId::LABELED_TYPE_PACK,
        RestMode::Complete => KindId::COMPLETE_TYPE_PACK,
        RestMode::None => KindId::TYPE,
    }
}

fn lane_argument_term(
    ctx: &mut CheckingContext<'_>,
    typed: &TypedExpression,
    context: crate::checker::inference::InferenceContextId,
    session: &std::rc::Rc<std::cell::RefCell<crate::checker::inference::InferenceCoordinator>>,
) -> Option<InferenceTerm> {
    if let Some(result) = typed.symbolic_result.as_ref().filter(|result| result.context == context) {
        return Some(session.borrow().term_for_expected(&result.term));
    }
    if let Some(ty) = typed.knowledge.ty() {
        return Some(session.borrow().type_id_to_inference(ty, &std::collections::HashMap::new(), ctx.store));
    }
    match &typed.knowledge {
        TypeKnowledge::Unknown(reason) => {
            ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::UnknownType(reason.clone())));
        }
        TypeKnowledge::Dynamic(reason) => ctx.record_call_status(AnalysisStatus::DynamicBoundary(reason.clone())),
        TypeKnowledge::Known(_) => ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::UnknownType(
            UnknownReason::InferenceBlocked,
        ))),
    }
    None
}

fn apply_structural_lane_conformance<'a>(
    ctx: &mut CheckingContext<'_>,
    lane_args: &[(usize, ApplicationArgument<'a>, TypedExpression)],
    expected: &InferenceTerm,
    mode: RestMode,
    call_range: SourceRange,
    origin: ConstraintOrigin,
    context: crate::checker::inference::InferenceContextId,
    session: &std::rc::Rc<std::cell::RefCell<crate::checker::inference::InferenceCoordinator>>,
) {
    let (positionals, labeled) = match (mode, expected) {
        (RestMode::Positional, InferenceTerm::TypePack(elements)) => (Some(elements.as_ref()), None),
        (RestMode::Labeled, InferenceTerm::LabeledTypePack(elements)) => (None, Some(elements.as_ref())),
        (RestMode::Complete, InferenceTerm::CompleteTypePack { positionals, labeled }) => {
            let (InferenceTerm::TypePack(pos), InferenceTerm::LabeledTypePack(lab)) = (&**positionals, &**labeled) else {
                ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
                return;
            };
            if lane_args.iter().filter(|(_, arg, _)| arg.label().is_none()).count() != pos.len()
                || lane_args.iter().filter(|(_, arg, _)| arg.label().is_some()).count() != lab.len()
            {
                let diagnostic = SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::CallShapeMismatch,
                    "complete rest lane shape mismatch",
                    call_range,
                );
                if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
                    ctx.record_call_status(AnalysisStatus::Invalid(cause));
                }
                return;
            }
            apply_structural_lane_conformance(
                ctx,
                &lane_args.iter().filter(|(_, arg, _)| arg.label().is_none()).cloned().collect::<Vec<_>>(),
                &InferenceTerm::TypePack(pos.clone()),
                RestMode::Positional,
                call_range,
                origin.clone(),
                context,
                session,
            );
            apply_structural_lane_conformance(
                ctx,
                &lane_args.iter().filter(|(_, arg, _)| arg.label().is_some()).cloned().collect::<Vec<_>>(),
                &InferenceTerm::LabeledTypePack(lab.clone()),
                RestMode::Labeled,
                call_range,
                origin,
                context,
                session,
            );
            return;
        }
        _ => {
            ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
            return;
        }
    };

    if positionals.is_some_and(|expected| expected.len() != lane_args.len()) || labeled.is_some_and(|expected| expected.len() != lane_args.len()) {
        let diagnostic = SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::CallShapeMismatch,
            "rest lane arity mismatch",
            call_range,
        );
        if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
            ctx.record_call_status(AnalysisStatus::Invalid(cause));
        }
        return;
    }
    for (index, (_, argument, typed)) in lane_args.iter().enumerate() {
        let expected_term = if let Some(expected) = positionals {
            expected[index].clone()
        } else {
            let expected = &labeled.expect("labeled lane selected")[index];
            if argument.label() != Some(expected.label.as_ref()) {
                let diagnostic = SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::CallShapeMismatch,
                    format!("labeled rest lane expected `{}`, found `{}`", expected.label, argument.label().unwrap_or("_")),
                    call_range,
                );
                if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
                    ctx.record_call_status(AnalysisStatus::Invalid(cause));
                }
                continue;
            }
            expected.term.clone()
        };
        if let Some(actual_term) = lane_argument_term(ctx, typed, context, session) {
            session
                .borrow_mut()
                .add_constraint(InferenceRelation::Subtype(actual_term, expected_term), origin.clone(), None);
        }
    }
}

fn check_concrete_lane_conformance_non_generic<'a>(
    ctx: &mut CheckingContext<'_>,
    lane_args: &[(usize, ApplicationArgument<'a>, TypedExpression)],
    lane_id: TypeId,
    mode: RestMode,
    call_range: SourceRange,
) {
    match mode {
        RestMode::Positional => {
            let Ok(expected_pos) = ctx.store.project_positionals(lane_id) else {
                return;
            };
            let TypeData::TypePack(expected_elems) = ctx.store.get(expected_pos).clone() else {
                return;
            };
            if lane_args.len() != expected_elems.len() {
                let diagnostic = SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::CallShapeMismatch,
                    format!("rest lane expects {} arguments, found {}", expected_elems.len(), lane_args.len()),
                    call_range,
                );
                if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
                    ctx.record_call_status(AnalysisStatus::Invalid(cause));
                }
                return;
            }
            for (idx, (_, arg, typed)) in lane_args.iter().enumerate() {
                let expected_elem = expected_elems[idx];
                ctx.apply_assignability(
                    &typed.knowledge,
                    &TypeKnowledge::established(expected_elem, EvidenceOrigin::DeveloperAnnotation),
                    DiagnosticCode::ArgumentMismatch,
                    format!("argument at lane position {} does not match expected element type", idx + 1),
                    arg.range(),
                );
            }
        }
        RestMode::Labeled => {
            let Ok(expected_lab) = ctx.store.project_labeled(lane_id) else {
                return;
            };
            let TypeData::LabeledTypePack(expected_elems) = ctx.store.get(expected_lab).clone() else {
                return;
            };
            if lane_args.len() != expected_elems.len() {
                let diagnostic = SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::CallShapeMismatch,
                    format!("labeled rest lane expects {} arguments, found {}", expected_elems.len(), lane_args.len()),
                    call_range,
                );
                if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
                    ctx.record_call_status(AnalysisStatus::Invalid(cause));
                }
                return;
            }
            for (idx, (_, arg, typed)) in lane_args.iter().enumerate() {
                let expected = &expected_elems[idx];
                if arg.label() != Some(expected.label.as_ref()) {
                    let diagnostic = SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::CallShapeMismatch,
                        format!("labeled rest lane expected `{}`, found `{}`", expected.label, arg.label().unwrap_or("_")),
                        call_range,
                    );
                    if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
                        ctx.record_call_status(AnalysisStatus::Invalid(cause));
                    }
                    continue;
                }
                ctx.apply_assignability(
                    &typed.knowledge,
                    &TypeKnowledge::established(expected.element, EvidenceOrigin::DeveloperAnnotation),
                    DiagnosticCode::ArgumentMismatch,
                    format!("argument for labeled lane `{}` does not match expected element type", expected.label),
                    arg.range(),
                );
            }
        }
        RestMode::Complete => {
            let Ok(expected_pos) = ctx.store.project_positionals(lane_id) else {
                return;
            };
            let Ok(expected_lab) = ctx.store.project_labeled(lane_id) else {
                return;
            };
            let TypeData::TypePack(pos_elems) = ctx.store.get(expected_pos).clone() else {
                return;
            };
            let TypeData::LabeledTypePack(lab_elems) = ctx.store.get(expected_lab).clone() else {
                return;
            };

            let pos_args: Vec<_> = lane_args.iter().filter(|(_, arg, _)| arg.label().is_none()).cloned().collect();
            let lab_args: Vec<_> = lane_args.iter().filter(|(_, arg, _)| arg.label().is_some()).cloned().collect();

            if pos_args.len() != pos_elems.len() || lab_args.len() != lab_elems.len() {
                let diagnostic = SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::CallShapeMismatch,
                    format!(
                        "complete rest lane shape mismatch: expected {} positionals and {} labeled, found {} and {}",
                        pos_elems.len(),
                        lab_elems.len(),
                        pos_args.len(),
                        lab_args.len()
                    ),
                    call_range,
                );
                if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
                    ctx.record_call_status(AnalysisStatus::Invalid(cause));
                }
                return;
            }
            for (idx, (_, arg, typed)) in pos_args.iter().enumerate() {
                let expected_elem = pos_elems[idx];
                ctx.apply_assignability(
                    &typed.knowledge,
                    &TypeKnowledge::established(expected_elem, EvidenceOrigin::DeveloperAnnotation),
                    DiagnosticCode::ArgumentMismatch,
                    format!("argument at positional lane position {} does not match expected element type", idx + 1),
                    arg.range(),
                );
            }
            for (idx, (_, arg, typed)) in lab_args.iter().enumerate() {
                let expected = &lab_elems[idx];
                if arg.label() != Some(expected.label.as_ref()) {
                    let diagnostic = SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::CallShapeMismatch,
                        format!(
                            "complete rest labeled mismatch: expected `{}`, found `{}`",
                            expected.label,
                            arg.label().unwrap_or("_")
                        ),
                        call_range,
                    );
                    if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
                        ctx.record_call_status(AnalysisStatus::Invalid(cause));
                    }
                    continue;
                }
                ctx.apply_assignability(
                    &typed.knowledge,
                    &TypeKnowledge::established(expected.element, EvidenceOrigin::DeveloperAnnotation),
                    DiagnosticCode::ArgumentMismatch,
                    format!("argument for labeled lane `{}` does not match expected element type", expected.label),
                    arg.range(),
                );
            }
        }
        RestMode::None => unreachable!(),
    }
}

fn cap_result_to_premise_authority(target: &CallableApplicationTarget, premise: &CallPremise, result: TypeKnowledge, range: SourceRange) -> TypeKnowledge {
    let Some(premise_status) = premise.knowledge.status() else {
        return result;
    };
    let maximum = target_base_authority(target).meet(premise_status);
    let origin = result.origin().unwrap_or(EvidenceOrigin::GenericInference);
    weaken_known_to_status(result, maximum, origin, range)
}

fn apply_non_generic_callable(
    ctx: &mut CheckingContext<'_>,
    target: &CallableApplicationTarget,
    premise: &CallPremise,
    arguments: &[ApplicationArgument<'_>],
    call_range: SourceRange,
) -> TypeKnowledge {
    let plan = match bind_static_arguments(arguments, &target.signature.parameters) {
        Ok(plan) => Some(plan),
        Err(failures) => {
            emit_shape_failures(ctx, &failures, call_range, target.callable.clone(), &target.signature.parameters);
            None
        }
    };

    let mut analyzed_arguments = Vec::with_capacity(arguments.len());

    for (argument_index, argument) in arguments.iter().copied().enumerate() {
        let parameter = plan
            .as_ref()
            .and_then(|plan| parameter_for_argument(plan, argument_index, &target.signature.parameters));
        let expected = parameter
            .and_then(|parameter| parameter.ty.ty())
            .map(|ty| ExpectedType::proper_from(ty, ExpectationOrigin::CallableSignature))
            .unwrap_or_default();
        let typed = analyze_application_argument(ctx, argument, &expected);
        analyzed_arguments.push((argument_index, argument, typed.clone()));
        if !typed.status.is_ready() {
            ctx.record_call_status(typed.status.clone());
        }
        if let Some(parameter) = parameter {
            // An omitted formal is analyzed without an expectation, just as
            // in generic application. Its absence is not an unknown contract
            // that should block a Ready argument; preserve the formal as
            // unknown and do not manufacture a relation from the argument.
            // Other unknown reasons represent unresolved contracts and retain
            // the normal assignability check below.
            if matches!(&parameter.ty, TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)) {
                continue;
            }
            let relation = ctx.apply_assignability(
                &typed.knowledge,
                &parameter.ty,
                DiagnosticCode::ArgumentMismatch,
                argument_relation_message(argument_index, argument, parameter),
                argument.range(),
            );
            if let (Some(call), Some(argument_id), Some(expected_ty)) = (ctx.current_expression_id(), typed.expression_id, parameter.ty.ty()) {
                let mut parents = Vec::new();
                if let Some(argument_explanation) = ctx.explanation_for_expression(argument_id) {
                    parents.push(argument_explanation);
                }
                if let Some(relation_explanation) = relation.explanation {
                    parents.push(relation_explanation);
                }
                let explanation = ctx.record_derivation(
                    crate::explain::ExplanationStep::ArgumentCheck {
                        call,
                        argument: argument_id,
                        parameter_index: plan
                            .as_ref()
                            .and_then(|plan| plan.bindings.iter().find(|binding| binding.argument_index == argument_index))
                            .map(|binding| binding.parameter_index as u16)
                            .unwrap_or(argument_index as u16),
                        actual: typed.knowledge.clone(),
                        expected: expected_ty,
                    },
                    crate::explain::DerivationRule::ArgumentChecking,
                    typed.knowledge.status().unwrap_or(EvidenceStatus::Assumed),
                    typed.knowledge.origin().unwrap_or(EvidenceOrigin::Flow),
                    vec![
                        crate::explain::EvidenceRef::SourceSpan(argument.range()),
                        crate::explain::EvidenceRef::TypeId(expected_ty),
                    ],
                    parents,
                );
                if let Some(cause) = relation.cause {
                    ctx.attach_explanation_to_cause(cause, explanation);
                }
                ctx.record_call_dependency(CausalInvalidity::Clean, Some(explanation));
            }
        }
    }

    if let Some(plan) = &plan {
        for (param_idx, parameter) in target.signature.parameters.iter().enumerate() {
            if let CallableParameterContract::Rest {
                mode,
                contract: RestValueContractKnowledge::Lane(lane_id),
            } = &parameter.contract
            {
                let lane_args: Vec<_> = analyzed_arguments
                    .iter()
                    .filter(|(arg_idx, _, _)| {
                        plan.bindings
                            .iter()
                            .find(|b| b.argument_index == *arg_idx)
                            .is_some_and(|b| b.parameter_index == param_idx)
                    })
                    .cloned()
                    .collect();
                check_concrete_lane_conformance_non_generic(ctx, &lane_args, *lane_id, *mode, call_range);
            }
        }
    }

    derive_fixed_return(target, premise, call_range)
}

/// Transitional semantic fallback.
/// Delete when canonical universe dispatch surfaces publish this indexer.
/// This helper may construct a target only; it must never construct the
/// final subscript expression result.
pub(crate) fn structural_list_index_get_target(ctx: &mut CheckingContext<'_>, receiver_ty: TypeId) -> Option<CallableApplicationTarget> {
    let TypeData::Applied { origin, arguments } = ctx.store.get(receiver_ty).clone() else {
        return None;
    };
    let TypeData::Nominal { declaration } = ctx.store.get(origin) else {
        return None;
    };
    if declaration != &ctx.core_ids.list || arguments.len() != 1 {
        return None;
    }
    let int_ty = ctx.core_type(&ctx.core_ids.int.clone())?;
    let selector = Selector::subscript_get(vec![SelectorSlot::Positional]).ok()?;
    let signature = CallableSignature::new(
        selector,
        vec![CallableParameter::new(
            "index",
            TypeKnowledge::established(int_ty, EvidenceOrigin::DeclarationSemantics),
        )],
        TypeKnowledge::established(arguments[0].as_type_id()?, EvidenceOrigin::DeclarationSemantics),
    );
    Some(CallableApplicationTarget::structural(signature))
}

/// Transitional semantic fallback for the current structural Map indexer.
/// This helper may construct a target only; it must never construct the final
/// subscript expression result.
pub(crate) fn structural_map_index_get_target(ctx: &mut CheckingContext<'_>, receiver_ty: TypeId) -> Option<CallableApplicationTarget> {
    let TypeData::Applied { origin, arguments } = ctx.store.get(receiver_ty).clone() else {
        return None;
    };
    let TypeData::Nominal { declaration } = ctx.store.get(origin) else {
        return None;
    };
    if declaration != &ctx.core_ids.map || arguments.len() != 2 {
        return None;
    }
    let selector = Selector::subscript_get(vec![SelectorSlot::Positional]).ok()?;
    let signature = CallableSignature::new(
        selector,
        vec![CallableParameter::new(
            "key",
            TypeKnowledge::established(arguments[0].as_type_id()?, EvidenceOrigin::DeclarationSemantics),
        )],
        TypeKnowledge::established(arguments[1].as_type_id()?, EvidenceOrigin::DeclarationSemantics),
    );
    Some(CallableApplicationTarget::structural(signature))
}

/// Transitional semantic fallback for List subscript assignment.
/// This helper may construct a target only; it must never construct the final
/// assignment expression result.
pub(crate) fn structural_list_index_set_target(ctx: &mut CheckingContext<'_>, receiver_ty: TypeId) -> Option<CallableApplicationTarget> {
    let TypeData::Applied { origin, arguments } = ctx.store.get(receiver_ty).clone() else {
        return None;
    };
    let TypeData::Nominal { declaration } = ctx.store.get(origin) else {
        return None;
    };
    if declaration != &ctx.core_ids.list || arguments.len() != 1 {
        return None;
    }
    let int_ty = ctx.core_type(&ctx.core_ids.int.clone())?;
    let element_ty = arguments[0];
    let selector = Selector::subscript_set(vec![SelectorSlot::Positional]).ok()?;
    let signature = CallableSignature::new(
        selector,
        vec![
            CallableParameter::new("index", TypeKnowledge::established(int_ty, EvidenceOrigin::DeclarationSemantics)),
            CallableParameter::new(
                "value",
                TypeKnowledge::established(element_ty.as_type_id()?, EvidenceOrigin::DeclarationSemantics),
            ),
        ],
        TypeKnowledge::established(element_ty.as_type_id()?, EvidenceOrigin::DeclarationSemantics),
    );
    Some(CallableApplicationTarget::structural(signature))
}

pub(crate) fn assignment_result_from_call(
    ctx: &mut CheckingContext<'_>,
    operation: CallCheckResult,
    range: SourceRange,
) -> crate::checker::typed_expr::TypedExpression {
    let mut typed = crate::checker::typed_expr::TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Syntax, range);
    typed.status = operation.status;
    typed.causal_invalidity = operation.causal_invalidity;
    typed.callable = operation.callable;
    typed.explanation_parents = operation.explanation_parents;
    typed.debug_assert_coherent();
    typed
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallCheckResult {
    pub knowledge: TypeKnowledge,
    pub status: AnalysisStatus,
    pub causal_invalidity: crate::checker::causal::CausalInvalidity,
    pub explanation_parents: Vec<crate::identity::ExplanationId>,
    pub callable: Option<crate::identity::CallableId>,
    pub(crate) local_type: Option<crate::types::rigid::LocalType>,
    pub(crate) symbolic_result: Option<crate::checker::typed_expr::SymbolicInferenceResult>,
}

fn local_type_from_arguments(
    ctx: &CheckingContext<'_>,
    result_ty: Option<TypeId>,
    local_types: &[(TypeId, crate::types::rigid::LocalType)],
) -> Option<crate::types::rigid::LocalType> {
    let result_ty = result_ty?;
    let replacements = local_types.iter().cloned().collect::<std::collections::HashMap<_, _>>();
    (!replacements.is_empty()).then(|| crate::types::rigid::LocalType::from_canonical_types(ctx.store, result_ty, &replacements))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExactReturnOrigin {
    CallableSignature,
    ConstructorSemantics,
    NativeSignature,
    GenericInference,
}

fn exact_return_origin(kind: CallableSemanticKind) -> ExactReturnOrigin {
    match kind {
        CallableSemanticKind::Ordinary => ExactReturnOrigin::CallableSignature,
        CallableSemanticKind::Constructor => ExactReturnOrigin::ConstructorSemantics,
        CallableSemanticKind::Native => ExactReturnOrigin::NativeSignature,
    }
}

/// Promotes a complete callable return contract to call-site knowledge.
/// Unknown and dynamic contracts remain unknown/dynamic; only a concrete
/// exact-dispatch return receives established call-site status.
fn promote_exact_return(return_type: &TypeKnowledge, origin: ExactReturnOrigin, range: SourceRange) -> TypeKnowledge {
    match return_type {
        TypeKnowledge::Known(evidence) => TypeKnowledge::established(
            evidence.ty(),
            match origin {
                ExactReturnOrigin::CallableSignature => EvidenceOrigin::CallableSignature,
                ExactReturnOrigin::ConstructorSemantics => EvidenceOrigin::ConstructorSemantics,
                ExactReturnOrigin::NativeSignature => EvidenceOrigin::NativeSignature,
                ExactReturnOrigin::GenericInference => EvidenceOrigin::GenericInference,
            },
        )
        .with_range(range),
        other => other.clone().with_range(range),
    }
}

struct GenericApplicationDomains<'a> {
    declaration: Option<&'a GenericSignature>,
    callable: Option<&'a GenericSignature>,
}

impl<'a> GenericApplicationDomains<'a> {
    fn signatures(&self) -> impl Iterator<Item = &'a GenericSignature> + '_ {
        self.declaration.into_iter().chain(self.callable)
    }

    fn has_parameters(&self) -> bool {
        self.signatures().any(|signature| !signature.parameters.is_empty())
    }

    fn has_record_row_parameters(&self, ctx: &CheckingContext<'_>) -> bool {
        self.signatures()
            .flat_map(|signature| signature.parameters.iter())
            .any(|parameter| ctx.store.type_parameter(*parameter).kind == crate::types::id::KindId::RECORD_ROW)
    }

    fn instantiate(
        &self,
        session: &mut InferenceSession,
        store: &TypeStore,
        frame: crate::checker::inference::InferenceFrameId,
        fixed_generics: &[(TypeParameterId, TypeId)],
    ) -> std::collections::HashMap<TypeParameterId, InferenceTerm> {
        let fixed = fixed_generics.iter().map(|(parameter, _)| *parameter).collect::<std::collections::HashSet<_>>();
        let mut result = std::collections::HashMap::new();
        for signature in self.signatures() {
            let instantiated = session.instantiate_generic_signature_in_frame_excluding(signature, store, frame, &fixed);
            result.extend(instantiated);
        }
        result
    }
}

fn apply_generic_callable_inner(
    ctx: &mut CheckingContext<'_>,
    target: &CallableApplicationTarget,
    signature: &CallableSignature,
    callable: Option<CallableId>,
    domains: GenericApplicationDomains<'_>,
    fixed_generics: &[(TypeParameterId, TypeId)],
    args: &[ApplicationArgument<'_>],
    expected: &ExpectedType,
    call_range: SourceRange,
) -> TypeKnowledge {
    if !domains.has_parameters() {
        return promote_exact_return(&signature.return_type, exact_return_origin(signature.kind), call_range);
    }

    let (context, root_frame, owns_context) = match expected {
        ExpectedType::Inference { context, .. } => {
            let Some(parent) = ctx.current_inference_frame(*context) else {
                ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
                return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
            };
            let Some(frame) = ctx.begin_inference_frame(*context, parent) else {
                ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
                return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
            };
            (*context, frame, false)
        }
        _ => {
            let (context, frame) = ctx.create_inference_context();
            (context, frame, true)
        }
    };
    let owner = super::sequence_publication::PublicationOwner(context, root_frame);
    if let Some(expression) = ctx.current_expression_id() {
        ctx.call_publication_owners.entry(expression).or_default().push((owner, owns_context));
        ctx.record_sequence_dependencies(&[owner].into_iter().collect());
    }
    ctx.push_inference_frame(context, root_frame);
    let result = apply_generic_callable_in_context(
        ctx,
        target,
        signature,
        callable,
        domains,
        fixed_generics,
        args,
        expected,
        call_range,
        context,
        root_frame,
    );
    ctx.pop_inference_frame(context, root_frame);
    if let Some(session) = ctx.inference_session(context) {
        session.borrow_mut().close_frame(root_frame);
    }
    result
}

#[allow(clippy::too_many_arguments)]
fn apply_generic_callable_in_context(
    ctx: &mut CheckingContext<'_>,
    target: &CallableApplicationTarget,
    signature: &CallableSignature,
    callable: Option<CallableId>,
    domains: GenericApplicationDomains<'_>,
    fixed_generics: &[(TypeParameterId, TypeId)],
    args: &[ApplicationArgument<'_>],
    expected: &ExpectedType,
    call_range: SourceRange,
    context: crate::checker::inference::InferenceContextId,
    frame: crate::checker::inference::InferenceFrameId,
) -> TypeKnowledge {
    let Some(session_handle) = ctx.inference_session(context) else {
        ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
        return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
    };

    let publication_owner = super::sequence_publication::PublicationOwner(context, frame);
    let has_row_generics = domains.has_record_row_parameters(ctx) || !session_handle.borrow().row_parameter_terms_for_frame(frame).is_empty();
    let domain_signatures = domains.signatures().collect::<Vec<_>>();
    let mut var_map = domains.instantiate(&mut session_handle.borrow_mut(), ctx.store, frame, fixed_generics);
    for &(parameter, ty) in fixed_generics {
        var_map.insert(parameter, InferenceTerm::Canonical(ty));
    }
    let mut row_session = has_row_generics
        .then(|| GenericApplicationSession::new_for_query_context(&domain_signatures, ctx.store, session_handle.clone(), frame, &var_map, fixed_generics));
    if let Some(rows) = row_session.as_mut() {
        for parameter in signature.parameters.iter() {
            if let Some(ty) = parameter.ty.ty() {
                if let Err(failure) = rows.constrain_signature_type_lacks(ty, ctx.store) {
                    record_row_constraint_failure(ctx, failure, call_range);
                }
            }
        }
        if let Some(ty) = signature.return_type.ty() {
            if let Err(failure) = rows.constrain_signature_type_lacks(ty, ctx.store) {
                record_row_constraint_failure(ctx, failure, call_range);
            }
        }
    }
    let Some(call_id) = ctx.current_expression_id() else {
        ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
        return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
    };
    let return_term = signature.return_type.ty().map(|return_type| {
        row_session.as_ref().map_or_else(
            || session_handle.borrow().type_id_to_inference(return_type, &var_map, ctx.store),
            |rows| {
                session_handle
                    .borrow()
                    .type_id_to_inference_with_rows(return_type, &var_map, &rows.row_terms(), ctx.store)
            },
        )
    });
    let fixed_return = return_term.as_ref().and_then(|term| {
        (!session_handle.borrow().term_has_variables(term)
            && !term_has_row_variables(term)
            && signature.return_type.ty().is_some_and(|ty| ctx.store.is_proper_type(ty)))
        .then(|| promote_exact_return(&signature.return_type, ExactReturnOrigin::GenericInference, call_range))
    });

    // Seed contextual result evidence before checking data-constructor
    // arguments. A generic variant's result parameters are invariant: the
    // expected result must select those parameters before payload arguments
    // contribute lower bounds (for example, `Result<Unit, Base> =
    // Result::Error(Derived.new())`). The same early ordering is needed when
    // the selected result parameter is callable, so an unannotated closure
    // receives its parameter and return expectations before it is analyzed.
    let expected_result_seeded = if row_session.is_none() {
        let is_variant_constructor = target
            .target
            .as_ref()
            .is_some_and(|target| matches!(target, crate::identity::InvocationTargetId::VariantConstructor(_)));
        if let (Some(return_term), Some(expected_ty)) = (return_term.as_ref(), expected.ty())
            && (is_variant_constructor || target.data_constructor_shape.is_some() || expected_type_contains_callable_payload(ctx.store, expected_ty))
        {
            let lexically_equivalent = session_handle
                .borrow()
                .materialize(return_term, ctx.store)
                .ok()
                .is_some_and(|return_ty| ctx.generic_evidence.are_types_equivalent(ctx.store, return_ty, expected_ty).is_proven());
            if !lexically_equivalent {
                let expected_term = InferenceTerm::Canonical(expected_ty);
                let term_to_constrain = session_handle.borrow().term_for_expected(return_term);
                session_handle.borrow_mut().add_constraint(
                    InferenceRelation::Subtype(term_to_constrain, expected_term),
                    ConstraintOrigin::ExpectedResult { expression: call_id },
                    None,
                );
                let _ = ctx
                    .inference_session(context)
                    .map(|session| ctx.solve_inference_in_frame(&mut session.borrow_mut(), frame));
            }
            true
        } else {
            false
        }
    } else {
        false
    };

    let argument_roles = match target.data_constructor_shape.as_ref() {
        Some(adapter) => adapter.bind_arguments(args, &signature.parameters, ctx.store),
        None => bind_static_arguments(args, &signature.parameters).map(|plan| {
            let mut roles = vec![None; args.len()];
            for binding in plan.bindings {
                roles[binding.argument_index] = Some(DataConstructorArgumentRole::Fixed {
                    parameter_index: binding.parameter_index,
                });
            }
            roles
        }),
    };
    let argument_roles = match argument_roles {
        Ok(roles) => roles,
        Err(failures) => {
            emit_shape_failures(ctx, &failures, call_range, None, &signature.parameters);
            for argument in args.iter().copied() {
                let typed = analyze_application_argument(ctx, argument, &ExpectedType::None);
                record_generic_argument_capture(ctx, &typed);
            }
            return fixed_return.unwrap_or(TypeKnowledge::Unknown(UnknownReason::InferenceBlocked));
        }
    };
    let mut exact_argument_bindings = std::collections::BTreeMap::new();
    let mut expansion_terms = DataConstructorExpansionTerms {
        complete: true,
        ..DataConstructorExpansionTerms::default()
    };
    if let Some(adapter) = target.data_constructor_shape.as_ref() {
        if let Some(expansion) = adapter.expansion() {
            expansion_terms.parameter = data_expansion_parameter(expansion, adapter.shape, ctx.store);
            expansion_terms.complete = expansion_terms.parameter.is_some();
        } else {
            expansion_terms.complete = false;
        }
    }

    let mandatory_start = session_handle.borrow().constraint_count();
    for generic_sig in domains.signatures() {
        let generic_callable = match &generic_sig.owner {
            TypeParameterOwner::Callable(callable) => callable.clone(),
            TypeParameterOwner::Declaration(declaration) => crate::identity::CallableId::new(declaration.clone(), signature.selector.clone(), ctx.current_side),
            TypeParameterOwner::Impl(impl_id) => {
                let dummy_decl = crate::identity::DeclarationId::new(impl_id.module.clone(), "_".into());
                crate::identity::CallableId::new(dummy_decl, signature.selector.clone(), ctx.current_side)
            }
        };
        for (constraint_index, constraint) in generic_sig.constraints.iter().enumerate() {
            let relation = match constraint {
                GenericConstraint::Subtype { lower, upper } => {
                    let lower = session_handle.borrow().type_term_to_inference(lower, &var_map, ctx.store);
                    let upper = session_handle.borrow().type_term_to_inference(upper, &var_map, ctx.store);
                    match (lower, upper) {
                        (Ok(lower), Ok(upper)) => InferenceRelation::Subtype(lower, upper),
                        _ => {
                            ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
                            return fixed_return.unwrap_or(TypeKnowledge::Unknown(UnknownReason::InferenceBlocked));
                        }
                    }
                }
                GenericConstraint::Equivalent { left, right } => {
                    let left = session_handle.borrow().type_term_to_inference(left, &var_map, ctx.store);
                    let right = session_handle.borrow().type_term_to_inference(right, &var_map, ctx.store);
                    match (left, right) {
                        (Ok(left), Ok(right)) => InferenceRelation::Equivalent(left, right),
                        _ => {
                            ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
                            return fixed_return.unwrap_or(TypeKnowledge::Unknown(UnknownReason::InferenceBlocked));
                        }
                    }
                }
                GenericConstraint::Conforms { .. } => {
                    continue;
                }
            };
            session_handle.borrow_mut().add_constraint(
                relation,
                ConstraintOrigin::GenericWhere {
                    callable: generic_callable.clone(),
                    constraint_index: constraint_index as u16,
                },
                None,
            );
        }
    }

    let mandatory_constraints = mandatory_start..session_handle.borrow().constraint_count();
    let mut lane_args_by_param: Vec<Vec<(usize, ApplicationArgument<'_>, TypedExpression)>> = vec![Vec::new(); signature.parameters.len()];

    for (argument_index, argument) in args.iter().copied().enumerate() {
        let Some(role) = argument_roles.get(argument_index).cloned().flatten() else {
            continue;
        };
        let parameter_index = match role.clone() {
            DataConstructorArgumentRole::Fixed { parameter_index } => parameter_index,
            _ => {
                let argument_typed = analyze_application_argument(ctx, argument, &ExpectedType::None);
                record_generic_argument_capture(ctx, &argument_typed);
                let (expansion_parameter, record_field) = match &role {
                    DataConstructorArgumentRole::PositionalPack { parameter } | DataConstructorArgumentRole::LabeledPack { parameter, .. } => {
                        (Some(*parameter), false)
                    }
                    DataConstructorArgumentRole::RecordField { parameter, .. } => (Some(*parameter), true),
                    DataConstructorArgumentRole::Expansion { parameter, .. } => (
                        Some(*parameter),
                        target.data_constructor_shape.as_ref().is_some_and(|a| a.shape == DataShape::Record),
                    ),
                    DataConstructorArgumentRole::Fixed { .. } => (None, false),
                };
                if record_field
                    && let Some(parameter) = expansion_parameter
                    && let Some(rows) = row_session.as_mut()
                {
                    rows.record_record_row_expansion_premise(parameter, &argument_typed.knowledge);
                }
                let expansion_premise_index = if let Some(parameter) = expansion_parameter {
                    expansion_terms.premises.push(DataConstructorExpansionPremise {
                        parameter,
                        argument: argument.range(),
                        knowledge: argument_typed.knowledge.clone(),
                        explanation: argument_typed.expression_id.and_then(|id| ctx.explanation_for_expression(id)),
                        record_field,
                        proof_term: None,
                    });
                    Some(expansion_terms.premises.len() - 1)
                } else {
                    None
                };
                let symbolic_argument = argument_typed
                    .symbolic_result
                    .as_ref()
                    .filter(|result| result.context == context)
                    .map(|result| session_handle.borrow().term_for_expected(&result.term));
                let inferred_term = symbolic_argument.or_else(|| {
                    let ty = argument_typed.knowledge.ty()?;
                    Some(row_session.as_ref().map_or_else(
                        || session_handle.borrow().type_id_to_inference(ty, &std::collections::HashMap::new(), ctx.store),
                        |rows| {
                            session_handle
                                .borrow()
                                .type_id_to_inference_with_rows(ty, &std::collections::HashMap::new(), &rows.row_terms(), ctx.store)
                        },
                    ))
                });
                if let DataConstructorArgumentRole::Expansion { parameter, mode } = role {
                    let extracted = argument_typed.knowledge.ty().and_then(|ty| extract_pack_expansion_elements(ctx.store, ty));
                    if let Some((pos_types, lab_types)) = extracted {
                        if let Some(index) = expansion_premise_index {
                            expansion_terms.premises[index].proof_term = inferred_term;
                        }
                        expansion_terms.parameter = Some(parameter);
                        let include_pos = matches!(mode, phalcom_ast::ast::ExpansionMode::Positional | phalcom_ast::ast::ExpansionMode::Complete);
                        let include_lab = matches!(mode, phalcom_ast::ast::ExpansionMode::Labeled | phalcom_ast::ast::ExpansionMode::Complete);
                        if include_pos {
                            for pos_ty in pos_types {
                                let term = row_session.as_ref().map_or_else(
                                    || {
                                        session_handle
                                            .borrow()
                                            .type_id_to_inference(pos_ty, &std::collections::HashMap::new(), ctx.store)
                                    },
                                    |rows| {
                                        session_handle.borrow().type_id_to_inference_with_rows(
                                            pos_ty,
                                            &std::collections::HashMap::new(),
                                            &rows.row_terms(),
                                            ctx.store,
                                        )
                                    },
                                );
                                expansion_terms.positional.push(term);
                            }
                        }
                        if include_lab {
                            for (label, lab_ty) in lab_types {
                                let term = row_session.as_ref().map_or_else(
                                    || {
                                        session_handle
                                            .borrow()
                                            .type_id_to_inference(lab_ty, &std::collections::HashMap::new(), ctx.store)
                                    },
                                    |rows| {
                                        session_handle.borrow().type_id_to_inference_with_rows(
                                            lab_ty,
                                            &std::collections::HashMap::new(),
                                            &rows.row_terms(),
                                            ctx.store,
                                        )
                                    },
                                );
                                if record_field {
                                    expansion_terms.fields.push(crate::checker::row_inference::InferenceRecordField {
                                        key: crate::field_key::FieldKey::symbol(label),
                                        term,
                                    });
                                } else {
                                    expansion_terms
                                        .labeled
                                        .push(crate::checker::inference::InferenceLabeledTypePackElement { label, term });
                                }
                            }
                        }
                    } else {
                        expansion_terms.complete = false;
                        match &argument_typed.knowledge {
                            TypeKnowledge::Unknown(reason) if argument_typed.symbolic_result.is_none() => {
                                ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::UnknownType(reason.clone())));
                            }
                            TypeKnowledge::Dynamic(reason) => ctx.record_call_status(AnalysisStatus::DynamicBoundary(reason.clone())),
                            _ => {}
                        }
                    }
                    if argument_index + 1 < args.len() {
                        let _ = ctx.inference_session(context).map(|session| ctx.propagate_inference(&mut session.borrow_mut()));
                    }
                    continue;
                }
                if let Some(term) = inferred_term {
                    if let Some(index) = expansion_premise_index {
                        expansion_terms.premises[index].proof_term = Some(term.clone());
                    }
                    match role {
                        DataConstructorArgumentRole::PositionalPack { parameter } => {
                            expansion_terms.parameter = Some(parameter);
                            expansion_terms.positional.push(term);
                        }
                        DataConstructorArgumentRole::LabeledPack { parameter, label } => {
                            expansion_terms.parameter = Some(parameter);
                            expansion_terms
                                .labeled
                                .push(crate::checker::inference::InferenceLabeledTypePackElement { label, term });
                        }
                        DataConstructorArgumentRole::RecordField { parameter, label } => {
                            expansion_terms.parameter = Some(parameter);
                            expansion_terms.fields.push(crate::checker::row_inference::InferenceRecordField {
                                key: crate::field_key::FieldKey::symbol(label),
                                term,
                            });
                        }
                        DataConstructorArgumentRole::Fixed { .. } | DataConstructorArgumentRole::Expansion { .. } => unreachable!(),
                    }
                } else {
                    expansion_terms.complete = false;
                    match &argument_typed.knowledge {
                        TypeKnowledge::Unknown(reason) if argument_typed.symbolic_result.is_none() => {
                            ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::UnknownType(reason.clone())));
                        }
                        TypeKnowledge::Dynamic(reason) => ctx.record_call_status(AnalysisStatus::DynamicBoundary(reason.clone())),
                        TypeKnowledge::Known(_) | TypeKnowledge::Unknown(_) => {}
                    }
                }

                if argument_index + 1 < args.len() {
                    let _ = ctx.inference_session(context).map(|session| ctx.propagate_inference(&mut session.borrow_mut()));
                }
                continue;
            }
        };
        let parameter = &signature.parameters[parameter_index];
        if target.data_constructor_shape.is_none() {
            if let CallableParameterContract::Rest { mode: _, contract } = &parameter.contract {
                match contract {
                    RestValueContractKnowledge::Lane(_) => {
                        let typed = analyze_application_argument(ctx, argument, &ExpectedType::None);
                        record_generic_argument_capture(ctx, &typed);
                        lane_args_by_param[parameter_index].push((argument_index, argument, typed));
                        if argument_index + 1 < args.len() {
                            let _ = ctx.inference_session(context).map(|session| ctx.propagate_inference(&mut session.borrow_mut()));
                        }
                        continue;
                    }
                    RestValueContractKnowledge::Unconstrained => {
                        let typed = analyze_application_argument(ctx, argument, &ExpectedType::None);
                        record_generic_argument_capture(ctx, &typed);
                        if argument_index + 1 < args.len() {
                            let _ = ctx.inference_session(context).map(|session| ctx.propagate_inference(&mut session.borrow_mut()));
                        }
                        continue;
                    }
                    RestValueContractKnowledge::Element(_) => {}
                    RestValueContractKnowledge::Invalid(reason) => {
                        let typed = analyze_application_argument(ctx, argument, &ExpectedType::None);
                        record_generic_argument_capture(ctx, &typed);
                        ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::UnknownType(reason.clone())));
                        continue;
                    }
                }
            }
        }
        let Some(parameter_ty) = parameter.ty.ty() else {
            let typed = analyze_application_argument(ctx, argument, &ExpectedType::None);
            record_generic_argument_capture(ctx, &typed);
            // An untyped formal contributes no static constraint. The argument
            // is still analyzed without an expectation, and its own status is
            // preserved by `record_generic_argument_capture` above.
            continue;
        };
        let parameter_term = row_session.as_ref().map_or_else(
            || session_handle.borrow().type_id_to_inference(parameter_ty, &var_map, ctx.store),
            |rows| {
                session_handle
                    .borrow()
                    .type_id_to_inference_with_rows(parameter_ty, &var_map, &rows.row_terms(), ctx.store)
            },
        );
        let expected_term = session_handle.borrow().term_for_expected(&parameter_term);
        let argument_expected = session_handle.borrow().materialize_for_expected(&expected_term, ctx.store).map_or_else(
            || ExpectedType::inference_from(context, expected_term, ExpectationOrigin::GenericArgument),
            |ty| ExpectedType::proper_from(ty, ExpectationOrigin::GenericArgument),
        );
        let argument_typed = analyze_application_argument_owned(ctx, argument, &argument_expected, Some(publication_owner));
        record_generic_argument_capture(ctx, &argument_typed);
        let explanation = argument_typed.expression_id.and_then(|id| ctx.explanation_for_expression(id));
        let origin = argument_typed
            .expression_id
            .map(|argument_id| ConstraintOrigin::Argument {
                call: call_id,
                argument: argument_id,
                parameter_index: parameter_index as u16,
            })
            .unwrap_or(ConstraintOrigin::Explicit);
        let symbolic_argument = argument_typed
            .symbolic_result
            .as_ref()
            .filter(|result| result.context == context)
            .map(|result| result.term.clone());
        if symbolic_argument.is_none() {
            session_handle
                .borrow_mut()
                .record_required_premise(&parameter_term, origin.clone(), &argument_typed.knowledge, explanation);
        }
        let mut stable_constraint_explanation = explanation;
        if let Some(argument_ty) = argument_typed.knowledge.ty() {
            if matches!(ctx.store.get(argument_ty), crate::types::store::TypeData::ExactCase { .. })
                && let crate::types::store::TypeData::Parameter(parameter) = ctx.store.get(parameter_ty)
            {
                exact_argument_bindings.insert(*parameter, argument_ty);
            }
            for parameter in session_handle.borrow().projected_parameters_for_term(&parameter_term, &var_map) {
                let constraint = ctx.record_derivation(
                    crate::explain::ExplanationStep::GenericConstraint {
                        parameter,
                        origin: crate::explain::GenericConstraintOrigin::Argument {
                            parameter_index: parameter_index as u16,
                        },
                        relation: crate::explain::GenericConstraintRelation::SupertypeOf(argument_ty),
                    },
                    crate::explain::DerivationRule::GenericConstraint,
                    argument_typed.knowledge.status().unwrap_or(EvidenceStatus::Assumed),
                    argument_typed.knowledge.origin().unwrap_or(EvidenceOrigin::GenericInference),
                    vec![crate::explain::EvidenceRef::TypeId(argument_ty)],
                    explanation.into_iter().collect(),
                );
                stable_constraint_explanation = Some(constraint);
                ctx.record_call_dependency(CausalInvalidity::Clean, Some(constraint));
            }
        }
        if let Some(symbolic_argument) = symbolic_argument {
            let symbolic_argument = session_handle.borrow().term_for_expected(&symbolic_argument);
            session_handle.borrow_mut().add_constraint_with_support(
                InferenceRelation::Subtype(symbolic_argument, parameter_term.clone()),
                origin.clone(),
                stable_constraint_explanation,
                InferenceSupport::Assumed,
            );
        }
        let mut row_argument_constrained = false;
        match &argument_typed.knowledge {
            TypeKnowledge::Known(evidence) => {
                if let Some(support) = inference_support(&argument_typed.knowledge) {
                    // Actual argument types belong to caller scope. Keep their
                    // canonical generic parameters rigid; only the callable's
                    // signature terms use this invocation's fresh variables.
                    if let (Some(rows), InferenceTerm::Record(formal_record)) = (row_session.as_mut(), &parameter_term) {
                        match rows.constrain_known_record_argument(evidence.ty(), formal_record, ctx.store) {
                            Ok(Some(field_constraints)) => {
                                row_argument_constrained = true;
                                rows.record_known_record_argument_premise(formal_record, &argument_typed.knowledge);
                                for (actual_field, formal_field) in field_constraints {
                                    session_handle.borrow_mut().add_constraint_with_support(
                                        InferenceRelation::Subtype(actual_field, formal_field),
                                        origin.clone(),
                                        stable_constraint_explanation,
                                        support,
                                    );
                                }
                            }
                            Ok(None) => {}
                            Err(failure) => {
                                row_argument_constrained = true;
                                record_row_constraint_failure(ctx, failure, call_range);
                            }
                        }
                    }
                    if !row_argument_constrained {
                        let argument_term = row_session.as_ref().map_or_else(
                            || {
                                session_handle
                                    .borrow()
                                    .type_id_to_inference(evidence.ty(), &std::collections::HashMap::new(), ctx.store)
                            },
                            |rows| {
                                session_handle.borrow().type_id_to_inference_with_rows(
                                    evidence.ty(),
                                    &std::collections::HashMap::new(),
                                    &rows.row_terms(),
                                    ctx.store,
                                )
                            },
                        );
                        session_handle.borrow_mut().add_constraint_with_support(
                            InferenceRelation::Subtype(argument_term, parameter_term),
                            origin,
                            stable_constraint_explanation,
                            support,
                        );
                    }
                }
            }
            TypeKnowledge::Unknown(reason) => {
                if argument_typed.symbolic_result.is_none() {
                    ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::UnknownType(reason.clone())));
                }
            }
            TypeKnowledge::Dynamic(reason) => {
                let has_conformance_bound = domains
                    .signatures()
                    .any(|sig| sig.constraints.iter().any(|c| matches!(c, GenericConstraint::Conforms { .. })));
                if has_conformance_bound {
                    let diagnostic = SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::GenericConstraintUnsatisfied,
                        "generic constraint unsatisfied: dynamic value does not satisfy static trait conformance bound",
                        call_range,
                    );
                    if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
                        ctx.record_call_status(AnalysisStatus::Invalid(cause));
                    }
                } else {
                    ctx.record_call_status(AnalysisStatus::DynamicBoundary(reason.clone()));
                }
            }
        }

        // Solve the prefix before analyzing the next argument. The single
        // session is retained; only its established substitutions are exposed
        // as contextual expectations to later arguments such as closures.
        if argument_index + 1 < args.len() {
            let _ = ctx.inference_session(context).map(|session| ctx.propagate_inference(&mut session.borrow_mut()));
        }
    }

    if target.data_constructor_shape.is_none() {
        for (param_idx, parameter) in signature.parameters.iter().enumerate() {
            if let CallableParameterContract::Rest {
                mode,
                contract: RestValueContractKnowledge::Lane(lane_id),
            } = &parameter.contract
            {
                let lane_args = &lane_args_by_param[param_idx];
                let lane_param_term = session_handle.borrow().type_id_to_inference(*lane_id, &var_map, ctx.store);
                let origin = ConstraintOrigin::RestLane {
                    call: call_id,
                    parameter_index: param_idx as u16,
                };
                let whole_lane_variable =
                    matches!(&lane_param_term, InferenceTerm::Var(variable) if session_handle.borrow().variable_has_kind(*variable, lane_kind(*mode)));
                if whole_lane_variable {
                    let captured_lane_term = match mode {
                        RestMode::Positional => {
                            let mut pos_terms = Vec::with_capacity(lane_args.len());
                            let mut ready = true;
                            for (_, _, typed) in lane_args {
                                if let Some(term) = lane_argument_term(ctx, typed, context, &session_handle) {
                                    pos_terms.push(term);
                                } else {
                                    ready = false;
                                }
                            }
                            ready.then(|| InferenceTerm::TypePack(pos_terms.into_boxed_slice()))
                        }
                        RestMode::Labeled => {
                            let mut lab_terms = Vec::with_capacity(lane_args.len());
                            let mut ready = true;
                            for (_, arg, typed) in lane_args {
                                if let Some(term) = lane_argument_term(ctx, typed, context, &session_handle) {
                                    let label = arg.label().unwrap_or("").into();
                                    lab_terms.push(InferenceLabeledTypePackElement { label, term });
                                } else {
                                    ready = false;
                                }
                            }
                            ready.then(|| InferenceTerm::LabeledTypePack(lab_terms.into_boxed_slice()))
                        }
                        RestMode::Complete => {
                            let mut pos_terms = Vec::new();
                            let mut lab_terms = Vec::new();
                            let mut ready = true;
                            for (_, arg, typed) in lane_args {
                                if let Some(term) = lane_argument_term(ctx, typed, context, &session_handle) {
                                    if let Some(label) = arg.label() {
                                        lab_terms.push(InferenceLabeledTypePackElement { label: label.into(), term });
                                    } else {
                                        pos_terms.push(term);
                                    }
                                } else {
                                    ready = false;
                                }
                            }
                            ready.then(|| InferenceTerm::CompleteTypePack {
                                positionals: Box::new(InferenceTerm::TypePack(pos_terms.into_boxed_slice())),
                                labeled: Box::new(InferenceTerm::LabeledTypePack(lab_terms.into_boxed_slice())),
                            })
                        }
                        RestMode::None => unreachable!(),
                    };
                    let Some(captured_lane_term) = captured_lane_term else {
                        continue;
                    };
                    let mut session = session_handle.borrow_mut();
                    if lane_args.is_empty() {
                        let empty_lane_ty = match mode {
                            RestMode::Positional => ctx.store.type_pack(Box::default()).unwrap(),
                            RestMode::Labeled => ctx.store.labeled_type_pack(Box::default()).unwrap(),
                            RestMode::Complete => {
                                let p = ctx.store.type_pack(Box::default()).unwrap();
                                let l = ctx.store.labeled_type_pack(Box::default()).unwrap();
                                ctx.store.complete_type_pack(p, l).unwrap()
                            }
                            RestMode::None => unreachable!(),
                        };
                        session.record_required_premise(
                            &lane_param_term,
                            origin.clone(),
                            &TypeKnowledge::established(empty_lane_ty, EvidenceOrigin::GenericInference),
                            None,
                        );
                    } else {
                        for (_, _, typed) in lane_args {
                            let explanation = typed.expression_id.and_then(|id| ctx.explanation_for_expression(id));
                            session.record_required_premise(&lane_param_term, origin.clone(), &typed.knowledge, explanation);
                        }
                    }
                    session.add_constraint(InferenceRelation::Equivalent(captured_lane_term, lane_param_term), origin, None);
                } else {
                    let expected = lane_param_term.clone();
                    apply_structural_lane_conformance(ctx, lane_args, &expected, *mode, call_range, origin, context, &session_handle);
                }
            }
        }
    }

    if expansion_terms.complete
        && let Some(parameter) = expansion_terms.parameter
        && let Some(adapter) = target.data_constructor_shape.as_ref()
        && let Some(expansion) = adapter.expansion()
    {
        let parameter_data = ctx.store.type_parameter(parameter);
        let mode_matches = match expansion.mode {
            phalcom_ast::ast::ExpansionMode::Positional => {
                parameter_data.mode == crate::types::parameter::GenericParameterMode::PositionalPack
                    && parameter_data.kind == crate::types::id::KindId::TYPE_PACK
            }
            phalcom_ast::ast::ExpansionMode::Labeled if adapter.shape == DataShape::Record => {
                parameter_data.mode == crate::types::parameter::GenericParameterMode::LabeledPack && parameter_data.kind == crate::types::id::KindId::RECORD_ROW
            }
            phalcom_ast::ast::ExpansionMode::Labeled => {
                parameter_data.mode == crate::types::parameter::GenericParameterMode::LabeledPack
                    && parameter_data.kind == crate::types::id::KindId::LABELED_TYPE_PACK
            }
            phalcom_ast::ast::ExpansionMode::Complete => {
                parameter_data.mode == crate::types::parameter::GenericParameterMode::CompletePack
                    && parameter_data.kind == crate::types::id::KindId::COMPLETE_TYPE_PACK
            }
        };
        if !mode_matches {
            let diagnostic = SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::KindExpectedType,
                "data product expansion binder mode and kind do not match",
                expansion.source.as_ref().map_or(call_range, |source| source.range),
            );
            if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
                ctx.record_call_status(AnalysisStatus::Invalid(cause));
            }
        } else if adapter.shape == DataShape::Record {
            if let Some(rows) = row_session.as_mut() {
                for premise in expansion_terms
                    .premises
                    .iter()
                    .filter(|premise| premise.parameter == parameter && premise.record_field)
                {
                    if let Some(term) = premise.proof_term.as_ref() {
                        session_handle.borrow_mut().record_required_premise(
                            term,
                            ConstraintOrigin::DataExpansionArgument {
                                call: call_id,
                                argument: premise.argument,
                                parameter,
                            },
                            &premise.knowledge,
                            premise.explanation,
                        );
                    }
                }
                if let Err(failure) =
                    rows.constrain_record_row_expansion(parameter, std::mem::take(&mut expansion_terms.fields), adapter.fixed_record_field_names())
                {
                    record_row_constraint_failure(ctx, failure, call_range);
                }
            } else {
                ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
            }
        } else {
            let exact_empty_complete_pack = expansion.mode == phalcom_ast::ast::ExpansionMode::Complete
                && parameter_data.kind == crate::types::id::KindId::COMPLETE_TYPE_PACK
                && expansion_terms.premises.is_empty()
                && expansion_terms.positional.is_empty()
                && expansion_terms.labeled.is_empty();
            let pack_term = match parameter_data.kind {
                crate::types::id::KindId::TYPE_PACK => InferenceTerm::TypePack(expansion_terms.positional.into_boxed_slice()),
                crate::types::id::KindId::LABELED_TYPE_PACK => InferenceTerm::LabeledTypePack(expansion_terms.labeled.into_boxed_slice()),
                crate::types::id::KindId::COMPLETE_TYPE_PACK => InferenceTerm::CompleteTypePack {
                    positionals: Box::new(InferenceTerm::TypePack(expansion_terms.positional.into_boxed_slice())),
                    labeled: Box::new(InferenceTerm::LabeledTypePack(expansion_terms.labeled.into_boxed_slice())),
                },
                _ => unreachable!("data constructor pack kind was validated above"),
            };
            if let Some(variable) = var_map.get(&parameter).cloned() {
                let mut session = session_handle.borrow_mut();
                for premise in expansion_terms.premises.iter().filter(|premise| premise.parameter == parameter) {
                    session.record_required_premise(
                        &variable,
                        ConstraintOrigin::DataExpansionArgument {
                            call: call_id,
                            argument: premise.argument,
                            parameter,
                        },
                        &premise.knowledge,
                        premise.explanation,
                    );
                }
                if exact_empty_complete_pack {
                    let empty_positionals = ctx.store.type_pack(Box::default());
                    let empty_labeled = ctx.store.labeled_type_pack(Box::default());
                    if let (Ok(empty_positionals), Ok(empty_labeled)) = (empty_positionals, empty_labeled)
                        && let Ok(empty_complete) = ctx.store.complete_type_pack(empty_positionals, empty_labeled)
                    {
                        session.record_required_premise(
                            &variable,
                            ConstraintOrigin::Explicit,
                            &TypeKnowledge::established(empty_complete, EvidenceOrigin::ConstructorSemantics),
                            None,
                        );
                    }
                }
                session.add_constraint(InferenceRelation::Equivalent(variable, pack_term), ConstraintOrigin::Explicit, None);
            } else {
                ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
            }
        }
    }

    let argument_outcome = ctx
        .inference_session(context)
        .map(|session| ctx.solve_inference_in_frame(&mut session.borrow_mut(), frame))
        .unwrap_or(crate::checker::inference::InferenceOutcome::Blocked(
            crate::types::outcome::BlockReason::RecursiveFixpoint,
        ));
    let mut row_outcome = row_session
        .as_mut()
        .map(|rows| ctx.control.relation(|budget, cancellation| rows.solve_rows_for_query(budget, cancellation)));
    let previously_unsolved_rows = match row_outcome.as_ref() {
        Some(crate::types::row_solver::RecordRowSolveResult::Underconstrained(value)) => value.variables.to_vec(),
        _ => Vec::new(),
    };
    let mut row_pending_ancestor = match (row_session.as_ref(), row_outcome.as_ref()) {
        (Some(rows), Some(crate::types::row_solver::RecordRowSolveResult::Solved(solution))) => return_term
            .as_ref()
            .is_some_and(|term| rows.has_pending_ancestor_row_dependency(solution, term)),
        _ => false,
    };
    let argument_underconstrained = match &argument_outcome {
        crate::checker::inference::InferenceOutcome::Underconstrained(value) => Some(value.clone()),
        _ => None,
    };
    let pre_context_result = match &argument_outcome {
        crate::checker::inference::InferenceOutcome::Solved(solution) => Some(publish_generic_return_with_rows(
            ctx,
            &session_handle.borrow(),
            &var_map,
            row_session.as_mut(),
            row_outcome.as_ref(),
            row_pending_ancestor,
            Some(solution),
            GenericReturnPublication {
                return_term: return_term.as_ref(),
                signature_return: &signature.return_type,
                call_range,
            },
        )),
        _ => None,
    };

    let mut outcome = if matches!(
        &argument_outcome,
        crate::checker::inference::InferenceOutcome::Solved(_) | crate::checker::inference::InferenceOutcome::Underconstrained(_)
    ) && !expected.is_none()
    {
        if let Some(return_term) = return_term.as_ref() {
            let expected_term = expected.ty().map(InferenceTerm::Canonical).or_else(|| match expected {
                ExpectedType::Inference {
                    context: expected_context,
                    term,
                    ..
                } if *expected_context == context => Some(term.clone()),
                _ => None,
            });
            if let Some(expected_term) = expected_term {
                let mut row_expected_constrained = false;
                if let (Some(rows), InferenceTerm::Record(return_record)) = (row_session.as_mut(), return_term) {
                    if let Some(expected_ty) = expected.ty() {
                        match rows.constrain_known_record_argument(expected_ty, return_record, ctx.store) {
                            Ok(Some(field_constraints)) => {
                                row_expected_constrained = true;
                                for (expected_field, return_field) in field_constraints {
                                    session_handle.borrow_mut().add_constraint(
                                        InferenceRelation::Subtype(expected_field, return_field),
                                        ConstraintOrigin::ExpectedResult { expression: call_id },
                                        None,
                                    );
                                }
                            }
                            Ok(None) => {}
                            Err(failure) => {
                                row_expected_constrained = true;
                                record_row_constraint_failure(ctx, failure, call_range);
                            }
                        }
                    }
                }
                if !row_expected_constrained && !expected_result_seeded {
                    let canonicalized_return_term = if let (Some(rows), Some(crate::types::row_solver::RecordRowSolveResult::Solved(row_solution))) =
                        (row_session.as_ref(), row_outcome.as_ref())
                    {
                        if term_has_row_variables(return_term) {
                            if let Ok(instantiation) =
                                rows.build_instantiation_from_active_terms_in_query(&session_handle.borrow(), &var_map, row_solution, ctx.store)
                            {
                                if let Some(ret_ty) = signature.return_type.ty() {
                                    crate::types::materialize_type(ctx.store, ret_ty, &instantiation, crate::types::RowMaterializationMode::RequireSolvedTail)
                                        .ok()
                                        .map(InferenceTerm::Canonical)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let term_to_constrain = canonicalized_return_term.as_ref().unwrap_or(return_term);
                    let lexically_equivalent = expected.ty().is_some_and(|expected_ty| {
                        session_handle
                            .borrow()
                            .materialize(term_to_constrain, ctx.store)
                            .ok()
                            .is_some_and(|return_ty| {
                                ctx.generic_evidence
                                    .are_types_equivalent(ctx.store, return_ty, expected_ty)
                                    .is_proven()
                            })
                    });
                    if !lexically_equivalent {
                        let term_to_constrain = session_handle.borrow().term_for_expected(term_to_constrain);
                        session_handle.borrow_mut().add_constraint(
                            InferenceRelation::Subtype(term_to_constrain, expected_term),
                            ConstraintOrigin::ExpectedResult { expression: call_id },
                            None,
                        );
                    }
                }
                session_handle.borrow_mut().record_context_selection(return_term);
                if let Some(expected_ty) = expected.ty() {
                    for parameter in session_handle.borrow().projected_parameters_for_term(return_term, &var_map) {
                        let explanation = ctx.record_derivation(
                            crate::explain::ExplanationStep::GenericConstraint {
                                parameter,
                                origin: crate::explain::GenericConstraintOrigin::ExpectedResult,
                                relation: crate::explain::GenericConstraintRelation::SupertypeOf(expected_ty),
                            },
                            crate::explain::DerivationRule::GenericConstraint,
                            EvidenceStatus::Assumed,
                            EvidenceOrigin::ContextualDerivation,
                            vec![crate::explain::EvidenceRef::TypeId(expected_ty)],
                            Vec::new(),
                        );
                        ctx.record_call_dependency(CausalInvalidity::Clean, Some(explanation));
                    }
                }
            }
        }
        let result = ctx
            .inference_session(context)
            .map(|session| ctx.solve_inference_in_frame(&mut session.borrow_mut(), frame))
            .unwrap_or(crate::checker::inference::InferenceOutcome::Blocked(
                crate::types::outcome::BlockReason::RecursiveFixpoint,
            ));
        if let Some(rows) = row_session.as_mut() {
            row_outcome = Some(ctx.control.relation(|budget, cancellation| rows.solve_rows_for_query(budget, cancellation)));
            if matches!(&result, crate::checker::inference::InferenceOutcome::Solved(_))
                && let Some(crate::types::row_solver::RecordRowSolveResult::Solved(solution)) = row_outcome.as_ref()
                && let Some(return_term) = return_term.as_ref()
            {
                rows.record_expected_result_row_selection(return_term, &previously_unsolved_rows, solution);
            }
        }
        row_pending_ancestor = match (row_session.as_ref(), row_outcome.as_ref()) {
            (Some(rows), Some(crate::types::row_solver::RecordRowSolveResult::Solved(solution))) => return_term
                .as_ref()
                .is_some_and(|term| rows.has_pending_ancestor_row_dependency(solution, term)),
            _ => false,
        };
        result
    } else {
        argument_outcome
    };
    if let Some(row_outcome) = row_outcome.as_ref() {
        record_row_solve_outcome(ctx, row_outcome, call_range);
    }
    let proven_lexical_equalities = match (&outcome, callable.as_ref()) {
        (crate::checker::inference::InferenceOutcome::Underconstrained(underconstrained), Some(callable))
            if underconstrained.unsolved_vars.is_empty() && !ctx.call_status_is_recorded() && !matches!(expected, ExpectedType::Inference { .. }) =>
        {
            let session = session_handle.borrow();
            let residuals = session.residual_obligations();
            if residuals.is_empty() || !var_map.values().all(|term| session.materialize(term, ctx.store).is_ok()) {
                None
            } else {
                residuals
                    .iter()
                    .map(|obligation| {
                        if !matches!(
                            &obligation.origin,
                            ConstraintOrigin::GenericWhere { callable: owner, .. } if owner == callable
                        ) {
                            return None;
                        }
                        let InferenceRelation::Equivalent(left, right) = &obligation.relation else {
                            return None;
                        };
                        if !matches!(left, InferenceTerm::AssociatedProjection { .. }) && !matches!(right, InferenceTerm::AssociatedProjection { .. }) {
                            return None;
                        }
                        let (Ok(left_ty), Ok(right_ty)) = (session.materialize(left, ctx.store), session.materialize(right, ctx.store)) else {
                            return None;
                        };
                        let left_term = crate::types::parameter::TypeTerm::Canonical(left_ty);
                        let right_term = crate::types::parameter::TypeTerm::Canonical(right_ty);
                        let proof = ctx.generic_evidence.are_equivalent(&left_term, &right_term);
                        let proof = if proof.is_proven() {
                            proof
                        } else {
                            ctx.generic_evidence.are_types_equivalent(ctx.store, left_ty, right_ty)
                        };
                        proof.is_proven().then(|| (obligation.constraint_index, obligation.relation.clone(), proof))
                    })
                    .collect::<Option<Vec<_>>>()
                    .filter(|proofs| !proofs.is_empty())
            }
        }
        _ => None,
    };
    if let Some(proven_equalities) = proven_lexical_equalities {
        let mut session = session_handle.borrow_mut();
        let all_discharged = proven_equalities
            .into_iter()
            .all(|(index, relation, proof)| session.discharge_residual_equivalence_with_lexical_proof(index, &relation, proof));
        if all_discharged {
            outcome = session.finish_frame(frame);
        }
    }

    let context_resolved = match (&argument_underconstrained, &outcome) {
        (Some(initial), crate::checker::inference::InferenceOutcome::Solved(solution)) => {
            initial.unsolved_vars.iter().all(|variable| solution.substitutions.contains_key(variable))
        }
        _ => false,
    };
    let underconstrained = if context_resolved {
        None
    } else {
        argument_underconstrained.as_ref().or(match &outcome {
            crate::checker::inference::InferenceOutcome::Underconstrained(value) => Some(value),
            _ => None,
        })
    };

    if let Some(underconstrained) = underconstrained
        && !ctx.call_status_is_recorded()
        && !matches!(expected, ExpectedType::Inference { .. })
    {
        // A known callee binding with an unproved declaration equality is a
        // constraint failure, not missing value evidence. Keep the residual
        // mandatory and do not publish a specialization for this call.
        let unproved_equality = underconstrained.unsolved_vars.is_empty() && {
            let session = session_handle.borrow();
            let evidence = &ctx.generic_evidence;
            let store = &mut *ctx.store;
            var_map.values().all(|term| session.materialize(term, store).is_ok())
                && session.residual_obligations().iter().any(|obligation| {
                    if !matches!(&obligation.origin, ConstraintOrigin::GenericWhere { callable: owner, .. } if Some(owner) == callable.as_ref()) {
                        return false;
                    }
                    let InferenceRelation::Equivalent(left, right) = &obligation.relation else {
                        return false;
                    };
                    let Ok(left_ty) = session.materialize(left, store) else {
                        return true;
                    };
                    let Ok(right_ty) = session.materialize(right, store) else {
                        return true;
                    };
                    let left_term = crate::types::parameter::TypeTerm::Canonical(left_ty);
                    let right_term = crate::types::parameter::TypeTerm::Canonical(right_ty);
                    !evidence.are_equivalent(&left_term, &right_term).is_proven()
                        && !evidence.are_types_equivalent(store, left_ty, right_ty).is_proven()
                })
        };
        let parameter = underconstrained
            .unsolved_vars
            .iter()
            .find_map(|variable| session_handle.borrow().parameter_for_variable(*variable, &var_map));
        let constraint_roots = session_handle.borrow().all_constraint_explanation_roots();
        let explanation = ctx.record_derivation(
            crate::explain::ExplanationStep::UnknownBoundary {
                reason: UnknownReason::UnderconstrainedTypeVariable,
                source: Some(call_range),
            },
            crate::explain::DerivationRule::UnknownPropagation,
            EvidenceStatus::Assumed,
            EvidenceOrigin::GenericInference,
            vec![crate::explain::EvidenceRef::SourceSpan(call_range)],
            constraint_roots,
        );
        let mut diagnostic = SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            if unproved_equality {
                DiagnosticCode::GenericConstraintUnsatisfied
            } else {
                DiagnosticCode::GenericInferenceUnderconstrained
            },
            if unproved_equality {
                "generic constraint unsatisfied: associated type equality cannot be proven"
            } else {
                "generic inference has insufficient value-producing evidence"
            },
            call_range,
        );
        if let Some(owner) = ctx.current_callable.clone() {
            diagnostic = diagnostic.with_explanation(crate::diagnostic::ExplanationRef::new(owner, explanation));
        }
        if let Some(parameter) = parameter {
            diagnostic = diagnostic.with_guidance(crate::diagnostic::DiagnosticGuidance::ResolveGenericParameter { parameter });
        }
        ctx.diagnostics.push(diagnostic);
        ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
    }

    if let crate::checker::inference::InferenceOutcome::Solved(_) = &outcome {
        for parameter in domains.signatures().flat_map(|signature| signature.parameters.iter()).copied() {
            let projected_solution = {
                let mut session = session_handle.borrow_mut();
                session.projected_solution(parameter, &var_map, ctx.store)
            };
            if let Some(ty) = projected_solution {
                let status = var_map
                    .get(&parameter)
                    .and_then(|term| session_handle.borrow().term_support(term))
                    .map(|support| match support {
                        InferenceSupport::Established => EvidenceStatus::Established,
                        InferenceSupport::Assumed => EvidenceStatus::Assumed,
                    })
                    .unwrap_or(EvidenceStatus::Assumed);
                let explanation = ctx.record_derivation(
                    crate::explain::ExplanationStep::GenericSolution { parameter, ty, status },
                    crate::explain::DerivationRule::GenericSolution,
                    status,
                    EvidenceOrigin::GenericInference,
                    vec![crate::explain::EvidenceRef::TypeId(ty)],
                    Vec::new(),
                );
                ctx.record_call_dependency(CausalInvalidity::Clean, Some(explanation));
            }
        }
        publish_call_specialization(
            ctx,
            callable,
            &domains,
            fixed_generics,
            &exact_argument_bindings,
            &var_map,
            &session_handle,
            call_range,
        );
    }

    match &outcome {
        crate::checker::inference::InferenceOutcome::Underconstrained(_) => {}
        crate::checker::inference::InferenceOutcome::Ambiguous(ambiguity) => {
            let mut diagnostic = SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::GenericInferenceAmbiguous,
                format!("generic inference has {} admissible solutions", ambiguity.candidates.len()),
                call_range,
            );
            let explanation = if ctx.current_callable.is_some() {
                let constraint_roots = session_handle.borrow().all_constraint_explanation_roots();
                let explanation = ctx.record_derivation(
                    crate::explain::ExplanationStep::UnknownBoundary {
                        reason: UnknownReason::InferenceAmbiguous,
                        source: Some(call_range),
                    },
                    crate::explain::DerivationRule::UnknownPropagation,
                    EvidenceStatus::Assumed,
                    EvidenceOrigin::GenericInference,
                    vec![crate::explain::EvidenceRef::SourceSpan(call_range)],
                    constraint_roots,
                );
                if let Some(owner) = ctx.current_callable.clone() {
                    diagnostic = diagnostic.with_explanation(crate::diagnostic::ExplanationRef::new(owner, explanation));
                }
                Some(explanation)
            } else {
                None
            };
            if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
                if let Some(explanation) = explanation {
                    ctx.record_call_dependency(CausalInvalidity::One(cause), Some(explanation));
                }
                ctx.record_call_status(AnalysisStatus::Invalid(cause));
            }
        }
        crate::checker::inference::InferenceOutcome::Conflicting(conflict) => {
            let session = session_handle.borrow();
            let range = conflict_source_range(ctx, &session, conflict, call_range);
            let parameter = match &conflict.failure {
                crate::checker::inference::InferenceFailureReason::ConflictingBounds { var, .. }
                | crate::checker::inference::InferenceFailureReason::OccursCheck { var }
                | crate::checker::inference::InferenceFailureReason::KindMismatch { var, .. }
                | crate::checker::inference::InferenceFailureReason::MissingVariableMetadata { var } => session.parameter_for_variable(*var, &var_map),
                crate::checker::inference::InferenceFailureReason::StructuralMismatch { .. }
                | crate::checker::inference::InferenceFailureReason::RowRejected(_)
                | crate::checker::inference::InferenceFailureReason::UnresolvedSelf
                | crate::checker::inference::InferenceFailureReason::InternalRowSolverFailure { .. } => None,
            };
            let mut parents = session.constraint_explanation_roots(&conflict.constraint_indices);
            parents.extend(ctx.current_call_explanations());
            parents.sort_by_key(|id| id.0);
            parents.dedup();
            let explanation = ctx.record_derivation(
                crate::explain::ExplanationStep::GenericConflict {
                    parameter,
                    constraints: parents.clone().into_boxed_slice(),
                },
                crate::explain::DerivationRule::GenericConflict,
                EvidenceStatus::Established,
                EvidenceOrigin::GenericInference,
                vec![crate::explain::EvidenceRef::SourceSpan(range)],
                parents,
            );
            let (code, message) = match &conflict.failure {
                crate::checker::inference::InferenceFailureReason::RowRejected(failure) => row_failure_diagnostic(ctx.store, failure),
                _ => {
                    let violates_declared_constraint = matches!(conflict.origin, Some(ConstraintOrigin::GenericWhere { .. }))
                        || conflict
                            .constraint_indices
                            .iter()
                            .any(|index| matches!(session.constraint_origin(*index), Some(ConstraintOrigin::GenericWhere { .. })));
                    let code = if violates_declared_constraint {
                        DiagnosticCode::GenericConstraintUnsatisfied
                    } else {
                        DiagnosticCode::GenericInferenceConflict
                    };
                    (code, generic_conflict_message(conflict))
                }
            };
            let mut diagnostic = SemanticDiagnostic::error_in(ctx.current_module.clone(), code, message, range);
            if let Some(owner) = ctx.current_callable.clone() {
                diagnostic = diagnostic.with_explanation(crate::diagnostic::ExplanationRef::new(owner, explanation));
            }
            if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
                ctx.record_call_dependency(CausalInvalidity::One(cause), Some(explanation));
                ctx.record_call_status(AnalysisStatus::Invalid(cause));
            }
        }
        crate::checker::inference::InferenceOutcome::Blocked(reason) => {
            ctx.record_call_status(AnalysisStatus::Blocked(reason.clone()));
        }
        crate::checker::inference::InferenceOutcome::Cancelled => ctx.record_call_status(AnalysisStatus::Cancelled),
        crate::checker::inference::InferenceOutcome::BudgetExceeded(report) => {
            ctx.record_call_status(AnalysisStatus::BudgetExceeded(report.clone()));
        }
        crate::checker::inference::InferenceOutcome::InternalFailure(failure) => {
            let incident = ctx.record_internal_incident(
                InternalSemanticIncidentKind::InferenceInvariantViolation,
                InternalSemanticIncidentDetails::Message {
                    message: format!("generic inference invariant failure: {failure:?}").into_boxed_str(),
                },
                Some(call_range),
            );
            ctx.record_call_status(AnalysisStatus::InternalFailure(incident));
        }
        crate::checker::inference::InferenceOutcome::Solved(_) => {}
    }

    if row_pending_ancestor || matches!(outcome, crate::checker::inference::InferenceOutcome::Underconstrained(_)) {
        if let Some(return_term) = return_term.as_ref() {
            if row_pending_ancestor || !term_has_row_variables(return_term) {
                ctx.publish_symbolic_inference_result(
                    call_id,
                    crate::checker::typed_expr::SymbolicInferenceResult {
                        context,
                        term: return_term.clone(),
                    },
                );
            }
        }
    }

    let deferred = row_pending_ancestor || matches!(outcome, crate::checker::inference::InferenceOutcome::Underconstrained(_));
    if outcome.is_solved() || (deferred && session_handle.borrow().frame_parent(frame).is_some()) {
        ctx.deferred_applications.insert(
            publication_owner,
            super::sequence_publication::DeferredApplicationRequirements {
                owner: publication_owner,
                expression: call_id,
                range: call_range,
                terms: var_map.clone(),
                return_term: return_term.clone(),
                signature_return: signature.return_type.clone(),
                rows: row_session.as_ref().and_then(GenericApplicationSession::query_requirements),
                mandatory_constraints,
                signatures: domains.signatures().cloned().collect(),
                exact_arguments: exact_argument_bindings.clone(),
                deferred,
            },
        );
    }

    match &outcome {
        crate::checker::inference::InferenceOutcome::Solved(_) if row_pending_ancestor => TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
        crate::checker::inference::InferenceOutcome::Solved(solution) => publish_generic_return_with_rows(
            ctx,
            &session_handle.borrow(),
            &var_map,
            row_session.as_mut(),
            row_outcome.as_ref(),
            false,
            Some(solution),
            GenericReturnPublication {
                return_term: return_term.as_ref(),
                signature_return: &signature.return_type,
                call_range,
            },
        ),
        crate::checker::inference::InferenceOutcome::Ambiguous(_) => terminal_generic_return(&outcome, fixed_return),
        crate::checker::inference::InferenceOutcome::Conflicting(_)
        | crate::checker::inference::InferenceOutcome::Blocked(_)
        | crate::checker::inference::InferenceOutcome::Cancelled
        | crate::checker::inference::InferenceOutcome::BudgetExceeded(_)
        | crate::checker::inference::InferenceOutcome::InternalFailure(_) => terminal_generic_return_with_fallback(&outcome, pre_context_result, fixed_return),
        crate::checker::inference::InferenceOutcome::Underconstrained(_) => {
            if let Some(pre_context) = pre_context_result {
                pre_context
            } else {
                incomplete_generic_return(&session_handle.borrow(), return_term.as_ref(), &outcome, fixed_return)
            }
        }
    }
}

fn expected_type_contains_callable_payload(store: &TypeStore, ty: TypeId) -> bool {
    match store.get(ty) {
        TypeData::Callable(_) => true,
        TypeData::Applied { arguments, .. } => arguments.iter().any(|argument| {
            argument
                .as_type_id()
                .is_some_and(|argument| expected_type_contains_callable_payload(store, argument))
        }),
        TypeData::ExactCase { enum_type, .. } => expected_type_contains_callable_payload(store, *enum_type),
        _ => false,
    }
}

fn incomplete_generic_return(
    session: &InferenceSession,
    return_term: Option<&InferenceTerm>,
    outcome: &crate::checker::inference::InferenceOutcome,
    fixed_return: Option<TypeKnowledge>,
) -> TypeKnowledge {
    if let Some(return_term) = return_term {
        match session.proof_state_for_term(return_term) {
            crate::checker::inference::InferenceProofState::Unknown(reason) => return TypeKnowledge::Unknown(reason),
            crate::checker::inference::InferenceProofState::Dynamic(reason) => return TypeKnowledge::Dynamic(reason),
            crate::checker::inference::InferenceProofState::Established | crate::checker::inference::InferenceProofState::Assumed => {}
        }
    }
    terminal_generic_return(outcome, fixed_return)
}

fn record_generic_argument_capture(ctx: &mut CheckingContext<'_>, typed: &TypedExpression) {
    let explanation = typed.expression_id.and_then(|id| ctx.explanation_for_expression(id));
    ctx.record_call_dependency(typed.causal_invalidity, explanation);
    if !typed.status.is_ready() {
        ctx.record_call_status(typed.status.clone());
    }
}

fn publish_generic_return(
    ctx: &mut CheckingContext<'_>,
    session: &InferenceSession,
    return_term: Option<&InferenceTerm>,
    signature_return: &TypeKnowledge,
    call_range: SourceRange,
) -> TypeKnowledge {
    let Some(return_term) = return_term else {
        return promote_exact_return(signature_return, exact_return_origin(CallableSemanticKind::Ordinary), call_range);
    };
    if !session.term_has_variables(return_term) {
        return promote_exact_return(signature_return, ExactReturnOrigin::GenericInference, call_range);
    }
    let proof = session.proof_state_for_term(return_term);
    match proof {
        crate::checker::inference::InferenceProofState::Unknown(reason) => TypeKnowledge::Unknown(reason),
        crate::checker::inference::InferenceProofState::Dynamic(reason) => TypeKnowledge::Dynamic(reason),
        crate::checker::inference::InferenceProofState::Established | crate::checker::inference::InferenceProofState::Assumed => {
            let Ok(ty) = session.materialize(return_term, ctx.store) else {
                let incident = ctx.record_internal_incident(
                    InternalSemanticIncidentKind::InferenceInvariantViolation,
                    InternalSemanticIncidentDetails::Message {
                        message: "solved generic return could not be materialized".into(),
                    },
                    Some(call_range),
                );
                ctx.record_call_status(AnalysisStatus::InternalFailure(incident));
                return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
            };
            match proof {
                crate::checker::inference::InferenceProofState::Established => {
                    TypeKnowledge::established(ty, EvidenceOrigin::GenericInference).with_range(call_range)
                }
                crate::checker::inference::InferenceProofState::Assumed => TypeKnowledge::assumed(ty, EvidenceOrigin::GenericInference).with_range(call_range),
                crate::checker::inference::InferenceProofState::Unknown(_) | crate::checker::inference::InferenceProofState::Dynamic(_) => {
                    unreachable!("proof state matched above")
                }
            }
        }
    }
}

struct GenericReturnPublication<'a> {
    return_term: Option<&'a InferenceTerm>,
    signature_return: &'a TypeKnowledge,
    call_range: SourceRange,
}

fn publish_generic_return_with_rows(
    ctx: &mut CheckingContext<'_>,
    session: &InferenceSession,
    type_terms: &std::collections::HashMap<TypeParameterId, InferenceTerm>,
    row_session: Option<&mut GenericApplicationSession>,
    row_outcome: Option<&crate::types::row_solver::RecordRowSolveResult<InferenceTerm>>,
    pending_ancestor_row_dependency: bool,
    _type_solution: Option<&crate::checker::inference::InferenceSolution>,
    publication: GenericReturnPublication<'_>,
) -> TypeKnowledge {
    let GenericReturnPublication {
        return_term,
        signature_return,
        call_range,
    } = publication;
    let Some(rows) = row_session else {
        return publish_generic_return(ctx, session, return_term, signature_return, call_range);
    };
    if pending_ancestor_row_dependency {
        return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
    }
    let Some(return_term) = return_term else {
        return promote_exact_return(signature_return, ExactReturnOrigin::GenericInference, call_range);
    };
    if !session.term_has_variables(return_term) && !term_has_row_variables(return_term) {
        return promote_exact_return(signature_return, ExactReturnOrigin::GenericInference, call_range);
    }
    let row_solution = match row_outcome {
        Some(crate::types::row_solver::RecordRowSolveResult::Solved(solution)) => Some(solution),
        _ => None,
    };
    let proof = rows.proof_state_for_term(session, return_term, row_solution);
    match proof {
        crate::checker::inference::InferenceProofState::Unknown(reason) => TypeKnowledge::Unknown(reason),
        crate::checker::inference::InferenceProofState::Dynamic(reason) => TypeKnowledge::Dynamic(reason),
        crate::checker::inference::InferenceProofState::Established | crate::checker::inference::InferenceProofState::Assumed => {
            let Some(row_solution) = row_solution else {
                return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
            };
            let instantiation = match rows.build_instantiation_from_active_terms_in_query(session, type_terms, row_solution, ctx.store) {
                Ok(instantiation) => instantiation,
                Err(failure) => {
                    record_row_constraint_failure(ctx, failure, call_range);
                    return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
                }
            };
            let Some(return_type) = signature_return.ty() else {
                return promote_exact_return(signature_return, ExactReturnOrigin::GenericInference, call_range);
            };
            let ty = match crate::types::materialize_type(ctx.store, return_type, &instantiation, crate::types::RowMaterializationMode::RequireSolvedTail) {
                Ok(ty) => ty,
                Err(error) => return record_generic_return_materialization_failure(ctx, error, call_range),
            };
            match proof {
                crate::checker::inference::InferenceProofState::Established => {
                    TypeKnowledge::established(ty, EvidenceOrigin::GenericInference).with_range(call_range)
                }
                crate::checker::inference::InferenceProofState::Assumed => TypeKnowledge::assumed(ty, EvidenceOrigin::GenericInference).with_range(call_range),
                crate::checker::inference::InferenceProofState::Unknown(_) | crate::checker::inference::InferenceProofState::Dynamic(_) => unreachable!(),
            }
        }
    }
}

fn record_row_constraint_failure<V>(ctx: &mut CheckingContext<'_>, failure: CombinedInferenceFailure<V>, range: SourceRange) {
    if ctx.call_status_is_recorded() {
        return;
    }
    let failure = match failure {
        CombinedInferenceFailure::RowZonk { parameter, error } => {
            match error {
                crate::types::row_solver::RecordRowZonkError::Unsolved(_) => {
                    record_blocked_row_diagnostic(
                        ctx,
                        DiagnosticCode::RecordRowInferenceUnderconstrained,
                        format!(
                            "record row parameter `{}` is still unresolved while materializing the generic return",
                            ctx.store.type_parameter(parameter).name
                        ),
                        range,
                    );
                }
                crate::types::row_solver::RecordRowZonkError::Recursive(_) => {
                    record_invalid_row_diagnostic(
                        ctx,
                        DiagnosticCode::RecordRowOccursCheck,
                        format!(
                            "record row parameter `{}` would require a recursive row substitution",
                            ctx.store.type_parameter(parameter).name
                        ),
                        range,
                    );
                }
                crate::types::row_solver::RecordRowZonkError::Formation(error) => record_row_formation_failure(ctx, error, range),
            }
            return;
        }
        failure => failure,
    };
    let (code, message) = match failure {
        CombinedInferenceFailure::RowZonk { .. } => unreachable!("row-zonk failures are handled before ordinary row failures"),
        CombinedInferenceFailure::RowRejected(failure) => row_failure_diagnostic(ctx.store, &failure),
        CombinedInferenceFailure::UnderconstrainedType(_) => (
            DiagnosticCode::GenericInferenceUnderconstrained,
            "generic type inference is underconstrained".into(),
        ),
        CombinedInferenceFailure::UnderconstrainedRow(parameter) => (
            DiagnosticCode::RecordRowInferenceUnderconstrained,
            format!("record row parameter `{}` is underconstrained", ctx.store.type_parameter(parameter).name),
        ),
        CombinedInferenceFailure::Blocked(reason) => {
            ctx.record_call_status(AnalysisStatus::Blocked(reason));
            return;
        }
        CombinedInferenceFailure::Cancelled => {
            ctx.record_call_status(AnalysisStatus::Cancelled);
            return;
        }
        CombinedInferenceFailure::BudgetExceeded(report) => {
            ctx.record_call_status(AnalysisStatus::BudgetExceeded(report));
            return;
        }
    };
    let diagnostic = SemanticDiagnostic::error_in(ctx.current_module.clone(), code, message, range);
    if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
        ctx.record_call_status(AnalysisStatus::Invalid(cause));
    } else {
        ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
    }
}

fn record_generic_return_materialization_failure(ctx: &mut CheckingContext<'_>, error: TypeMaterializationError, range: SourceRange) -> TypeKnowledge {
    if ctx.call_status_is_recorded() {
        return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
    }
    match error {
        TypeMaterializationError::DynamicBoundary(_) => {
            let reason = DynamicReason::ExplicitEscape;
            ctx.record_call_status(AnalysisStatus::DynamicBoundary(reason.clone()));
            return TypeKnowledge::Dynamic(reason);
        }
        TypeMaterializationError::UnresolvedRowParameter(parameter) => record_blocked_row_diagnostic(
            ctx,
            DiagnosticCode::RecordRowInferenceUnderconstrained,
            format!(
                "record row parameter `{}` is still unresolved while materializing the generic return",
                ctx.store.type_parameter(parameter).name
            ),
            range,
        ),
        TypeMaterializationError::RecursiveRowSubstitution(parameter) => record_invalid_row_diagnostic(
            ctx,
            DiagnosticCode::RecordRowOccursCheck,
            format!(
                "record row parameter `{}` would require a recursive row substitution",
                ctx.store.type_parameter(parameter).name
            ),
            range,
        ),
        TypeMaterializationError::RecordRow(error) => record_row_formation_failure(ctx, error, range),
        TypeMaterializationError::TypeApplication => record_internal_row_failure(
            ctx,
            "generic return type could not be materialized after inference",
            "generic return materialization failed with an invalid type application".into(),
            range,
        ),
        TypeMaterializationError::Pack(error) => record_internal_row_failure(
            ctx,
            "generic return type could not be materialized after inference",
            format!("generic return materialization produced an invalid pack: {error}"),
            range,
        ),
    }
    TypeKnowledge::Unknown(UnknownReason::InferenceBlocked)
}

fn record_row_formation_failure(ctx: &mut CheckingContext<'_>, error: RecordRowFormationError, range: SourceRange) {
    match error {
        RecordRowFormationError::DuplicateField(field) => record_invalid_row_diagnostic(
            ctx,
            DiagnosticCode::RecordDuplicateField,
            format!("generic return materialization would contain duplicate record field `{field}`"),
            range,
        ),
        RecordRowFormationError::TailParameterWrongKind { parameter, .. } => record_invalid_row_diagnostic(
            ctx,
            DiagnosticCode::RecordRowTailKindMismatch,
            format!(
                "record row tail parameter `{}` has the wrong kind during generic return materialization",
                ctx.store.type_parameter(parameter).name
            ),
            range,
        ),
        RecordRowFormationError::TailParameterMissing(_) => record_internal_row_failure(
            ctx,
            "generic return type could not be materialized because its record row contract is invalid",
            "generic return materialization referred to a missing record row parameter".into(),
            range,
        ),
        RecordRowFormationError::FieldNotProperType { field, .. } => record_internal_row_failure(
            ctx,
            "generic return type contains an invalid record field type after inference",
            format!("generic return materialization found a non-proper type in record field `{field}`"),
            range,
        ),
        RecordRowFormationError::TailKeyDomainInvalid(ty) | RecordRowFormationError::TailValueNotProperType(ty) => record_internal_row_failure(
            ctx,
            "generic return type contains an invalid Record tail after inference",
            format!("generic return materialization found a non-proper tail type `{}`", ctx.store.format_type(ty)),
            range,
        ),
    }
}

fn record_invalid_row_diagnostic(ctx: &mut CheckingContext<'_>, code: DiagnosticCode, message: String, range: SourceRange) {
    let diagnostic = SemanticDiagnostic::error_in(ctx.current_module.clone(), code, message, range);
    if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
        ctx.record_call_status(AnalysisStatus::Invalid(cause));
    } else {
        ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
    }
}

fn record_blocked_row_diagnostic(ctx: &mut CheckingContext<'_>, code: DiagnosticCode, message: String, range: SourceRange) {
    ctx.emit_diagnostic(SemanticDiagnostic::error_in(ctx.current_module.clone(), code, message, range));
    ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
}

fn record_internal_row_failure(ctx: &mut CheckingContext<'_>, safe_message: &'static str, details: String, range: SourceRange) {
    let incident = ctx.record_internal_incident(
        InternalSemanticIncidentKind::InferenceInvariantViolation,
        InternalSemanticIncidentDetails::Message {
            message: details.into_boxed_str(),
        },
        Some(range),
    );
    ctx.emit_diagnostic(SemanticDiagnostic::error_in(
        ctx.current_module.clone(),
        DiagnosticCode::AnalysisInternalFailure,
        safe_message,
        range,
    ));
    ctx.record_call_status(AnalysisStatus::InternalFailure(incident));
}

fn row_failure_diagnostic<V>(store: &TypeStore, failure: &crate::types::row_solver::RecordRowFailure<V>) -> (DiagnosticCode, String) {
    match failure {
        crate::types::row_solver::RecordRowFailure::LacksViolation { field, .. } => (
            DiagnosticCode::RecordRowLacksViolation,
            format!("record row cannot contain forbidden field `{field}`"),
        ),
        crate::types::row_solver::RecordRowFailure::OccursCheckFailed { .. } => {
            (DiagnosticCode::RecordRowOccursCheck, "record row inference would be recursive".into())
        }
        crate::types::row_solver::RecordRowFailure::IncompatibleFields { field, expected, actual } => (
            DiagnosticCode::RecordRowInferenceConflict,
            format!(
                "record field `{field}` has incompatible types `{}` and `{}`",
                store.format_type(*expected),
                store.format_type(*actual)
            ),
        ),
        crate::types::row_solver::RecordRowFailure::MissingField { field } => (
            DiagnosticCode::RecordRowInferenceConflict,
            format!("record argument is missing required field `{field}`"),
        ),
        crate::types::row_solver::RecordRowFailure::ExtraField { field } => (
            DiagnosticCode::RecordRowInferenceConflict,
            format!("record row cannot accept extra field `{field}`"),
        ),
        crate::types::row_solver::RecordRowFailure::DuplicateField(field) => {
            (DiagnosticCode::RecordDuplicateField, format!("record row contains duplicate field `{field}`"))
        }
        crate::types::row_solver::RecordRowFailure::RigidTailMismatch { .. } => {
            (DiagnosticCode::RecordRowInferenceConflict, "record row tails conflict".into())
        }
        crate::types::row_solver::RecordRowFailure::KindMismatch => (DiagnosticCode::RecordRowInferenceConflict, "record row kinds conflict".into()),
    }
}

fn record_row_solve_outcome<V: Clone>(ctx: &mut CheckingContext<'_>, outcome: &crate::types::row_solver::RecordRowSolveResult<V>, range: SourceRange) {
    match outcome {
        crate::types::row_solver::RecordRowSolveResult::Solved(_) => {}
        crate::types::row_solver::RecordRowSolveResult::Underconstrained(_) => {
            if !ctx.call_status_is_recorded() {
                let diagnostic = SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::RecordRowInferenceUnderconstrained,
                    "record row inference has insufficient constraints",
                    range,
                );
                ctx.diagnostics.push(diagnostic);
                ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
            } else {
                ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
            }
        }
        crate::types::row_solver::RecordRowSolveResult::Rejected(failure) => {
            if !ctx.call_status_is_recorded() {
                record_row_constraint_failure(ctx, CombinedInferenceFailure::RowRejected(failure.clone()), range);
            }
        }
        crate::types::row_solver::RecordRowSolveResult::Blocked(_) => {
            ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint));
        }
        crate::types::row_solver::RecordRowSolveResult::Cancelled => ctx.record_call_status(AnalysisStatus::Cancelled),
        crate::types::row_solver::RecordRowSolveResult::BudgetExceeded(report) => {
            ctx.record_call_status(AnalysisStatus::BudgetExceeded(report.clone()));
        }
        crate::types::row_solver::RecordRowSolveResult::InternalFailure(_) => {
            let incident = ctx.record_internal_incident(
                InternalSemanticIncidentKind::InferenceInvariantViolation,
                InternalSemanticIncidentDetails::Message {
                    message: "record row solver reported an internal failure".into(),
                },
                Some(range),
            );
            ctx.record_call_status(AnalysisStatus::InternalFailure(incident));
        }
    }
}

fn conflict_source_range(
    ctx: &CheckingContext<'_>,
    session: &InferenceSession,
    conflict: &crate::checker::inference::InferenceConflict,
    fallback: SourceRange,
) -> SourceRange {
    let range = match conflict.origin.as_ref() {
        Some(ConstraintOrigin::Argument { argument, .. }) => ctx.expressions.get(argument).map(|analysis| analysis.range),
        _ => conflict.constraint_indices.iter().find_map(|index| match session.constraint_origin(*index) {
            Some(ConstraintOrigin::Argument { argument, .. }) => ctx.expressions.get(argument).map(|analysis| analysis.range),
            _ => None,
        }),
    };
    range
        .or_else(|| {
            let ty = match &conflict.failure {
                crate::checker::inference::InferenceFailureReason::StructuralMismatch { left, right } => {
                    [left.as_ref(), right.as_ref()].into_iter().find_map(|term| match term {
                        crate::checker::inference::InferenceTerm::Canonical(ty) => Some(*ty),
                        _ => None,
                    })
                }
                crate::checker::inference::InferenceFailureReason::ConflictingBounds { lower, .. } => Some(*lower),
                _ => None,
            }?;
            ctx.expressions
                .values()
                .filter(|expression| expression.range.start >= fallback.start && expression.range.end <= fallback.end)
                .filter(|expression| expression.knowledge.ty() == Some(ty))
                .min_by_key(|expression| (expression.range.end - expression.range.start, expression.range.start))
                .map(|expression| expression.range)
        })
        .unwrap_or(fallback)
}

fn generic_conflict_message(conflict: &crate::checker::inference::InferenceConflict) -> String {
    match &conflict.failure {
        crate::checker::inference::InferenceFailureReason::ConflictingBounds { .. } => "generic argument does not satisfy type constraints".into(),
        crate::checker::inference::InferenceFailureReason::StructuralMismatch { .. } => "generic argument constraints conflict".into(),
        failure => format!("generic inference failed: {failure:?}"),
    }
}

fn type_contains_any_parameter(store: &TypeStore, ty: TypeId, parameters: &[TypeParameterId]) -> bool {
    ParameterUseWalker::new(store, parameters).visit_type(ty)
}

/// Finds declaration-owned parameters anywhere in a canonical semantic form,
/// including non-Type arguments and associated static-domain owner environments.
struct ParameterUseWalker<'a> {
    store: &'a TypeStore,
    parameters: &'a [TypeParameterId],
    seen_types: std::collections::HashSet<TypeId>,
    seen_rows: std::collections::HashSet<RecordRowId>,
    seen_domains: std::collections::HashSet<StaticDomainId>,
    seen_lambdas: std::collections::HashSet<TypeLambdaId>,
    seen_scoped_types: std::collections::HashSet<ScopedTypeId>,
}

impl<'a> ParameterUseWalker<'a> {
    fn new(store: &'a TypeStore, parameters: &'a [TypeParameterId]) -> Self {
        Self {
            store,
            parameters,
            seen_types: std::collections::HashSet::new(),
            seen_rows: std::collections::HashSet::new(),
            seen_domains: std::collections::HashSet::new(),
            seen_lambdas: std::collections::HashSet::new(),
            seen_scoped_types: std::collections::HashSet::new(),
        }
    }

    fn visit_type(&mut self, ty: TypeId) -> bool {
        if !self.seen_types.insert(ty) {
            return false;
        }
        match self.store.get(ty).clone() {
            TypeData::Parameter(parameter) => self.parameters.contains(&parameter),
            TypeData::Applied { origin, arguments } => {
                self.visit_type(origin) || arguments.iter().copied().any(|argument| self.visit_static_argument(argument))
            }
            TypeData::StaticDomainForm { domain } => self.visit_static_domain(domain),
            TypeData::StaticData(term) => {
                self.visit_static_domain(term.domain) || term.arguments.iter().copied().any(|argument| self.visit_static_argument(argument))
            }
            TypeData::Union(members) => members.iter().copied().any(|member| self.visit_type(member)),
            TypeData::Tuple(elements) => elements.elements.iter().any(|element| self.visit_type(element.ty)),
            TypeData::Pack(pack) => pack.elements.iter().any(|element| self.visit_type(element.ty)),
            TypeData::Record(row) => self.visit_row(row),
            TypeData::TypePack(elements) | TypeData::TypePackConcat(elements) | TypeData::LabeledTypePackConcat(elements) => {
                elements.iter().copied().any(|element| self.visit_type(element))
            }
            TypeData::LabeledTypePack(elements) => elements.iter().any(|element| self.visit_type(element.element)),
            TypeData::CompleteTypePack { positionals, labeled }
            | TypeData::TupleFromPacks { positionals, labeled }
            | TypeData::PackFromPacks { positionals, labeled } => self.visit_type(positionals) || self.visit_type(labeled),
            TypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => self.visit_type(positionals) || self.visit_type(labeled) || self.visit_type(return_type),
            TypeData::Callable(callable) => callable.parameters.iter().any(|parameter| self.visit_type(parameter.ty)) || self.visit_type(callable.return_type),
            TypeData::Family(family_id) => self.store.get_family(family_id).members.iter().any(|member| self.visit_type(member.ty)),
            TypeData::ExactCase { enum_type, .. } => self.visit_type(enum_type),
            TypeData::AssociatedProjection(projection) => {
                self.visit_type(projection.subject) || projection.trait_ref.arguments.iter().copied().any(|arg| self.visit_type(arg))
            }
            TypeData::TypeMemberProjection(projection) => {
                use crate::types::type_member::{ProjectionSubject, TypeMemberArgument};
                let subject_contains = match projection.subject {
                    ProjectionSubject::Type(subject) => self.visit_type(subject),
                    ProjectionSubject::RecordRow(row) => self.visit_row(row),
                };
                subject_contains
                    || projection.arguments.iter().any(|argument| match argument {
                        TypeMemberArgument::Type(ty) => self.visit_type(*ty),
                        TypeMemberArgument::RecordRow(row) => self.visit_row(*row),
                        TypeMemberArgument::Const(_) => false,
                    })
            }
            TypeData::Lambda(lambda) => self.visit_lambda(lambda),
            TypeData::Never | TypeData::Unit | TypeData::ClassObject { .. } | TypeData::Nominal { .. } | TypeData::SelfType(_) | TypeData::ConstInt(_) => false,
        }
    }

    fn visit_static_argument(&mut self, argument: StaticArgument) -> bool {
        match argument {
            StaticArgument::Term(ty) => self.visit_type(ty),
            StaticArgument::RecordRow(row) => self.visit_row(row),
            StaticArgument::DynamicType => false,
        }
    }

    fn visit_static_domain(&mut self, domain: StaticDomainId) -> bool {
        if !self.seen_domains.insert(domain) {
            return false;
        }
        let Some(StaticDomainData::Associated { owner_arguments, .. }) = self.store.static_domain(domain).cloned() else {
            return false;
        };
        owner_arguments.iter().copied().any(|argument| self.visit_static_argument(argument))
    }

    fn visit_row(&mut self, row_id: RecordRowId) -> bool {
        use crate::types::row::RecordRowTail;

        if !self.seen_rows.insert(row_id) {
            return false;
        }
        let row = self.store.record_row(row_id).clone();
        row.fields.iter().any(|field| self.visit_type(field.ty))
            || match row.tail {
                RecordRowTail::Closed => false,
                RecordRowTail::Anonymous(crate::types::row::AnonymousRecordRemainder::Unconstrained { key_type }) => self.visit_type(key_type),
                RecordRowTail::Anonymous(crate::types::row::AnonymousRecordRemainder::Typed { key_type, value_type }) => {
                    self.visit_type(key_type) || self.visit_type(value_type)
                }
                RecordRowTail::Parameter(parameter) => self.parameters.contains(&parameter),
                RecordRowTail::Projection(projection) => self.visit_type(projection),
            }
    }

    fn visit_lambda(&mut self, lambda: TypeLambdaId) -> bool {
        if !self.seen_lambdas.insert(lambda) {
            return false;
        }
        let body = self.store.arena().get_lambda(lambda).body;
        self.visit_scoped_type(body)
    }

    fn visit_scoped_type(&mut self, scoped: ScopedTypeId) -> bool {
        use crate::types::type_lambda::{ScopedPackRemainder, ScopedRecordRowTail, ScopedRecordTail, ScopedTypeData};

        if !self.seen_scoped_types.insert(scoped) {
            return false;
        }
        match self.store.arena().get_scoped(scoped).clone() {
            ScopedTypeData::Bound { .. } => false,
            ScopedTypeData::Free(ty) => self.visit_type(ty),
            ScopedTypeData::Applied { origin, arguments } => {
                self.visit_scoped_type(origin) || arguments.iter().copied().any(|argument| self.visit_scoped_type(argument))
            }
            ScopedTypeData::Union(members) | ScopedTypeData::TypePack(members) | ScopedTypeData::LabeledTypePackConcat(members) => {
                members.iter().copied().any(|member| self.visit_scoped_type(member))
            }
            ScopedTypeData::Tuple(elements) => elements.iter().any(|element| self.visit_scoped_type(element.ty)),
            ScopedTypeData::Record(fields) => fields.iter().any(|field| self.visit_scoped_type(field.ty)),
            ScopedTypeData::OpenRecord(record) => {
                record.fields.iter().any(|field| self.visit_scoped_type(field.ty))
                    || match record.tail {
                        ScopedRecordTail::FreeParameter(parameter) => self.parameters.contains(&parameter),
                        ScopedRecordTail::Bound { .. } => false,
                        ScopedRecordTail::Anonymous(crate::types::type_lambda::ScopedAnonymousRecordRemainder::Unconstrained { key_type }) => {
                            self.visit_scoped_type(key_type)
                        }
                        ScopedRecordTail::Anonymous(crate::types::type_lambda::ScopedAnonymousRecordRemainder::Typed { key_type, value_type }) => {
                            self.visit_scoped_type(key_type) || self.visit_scoped_type(value_type)
                        }
                    }
            }
            ScopedTypeData::RecordRow(row) => {
                row.fields.iter().any(|field| self.visit_scoped_type(field.ty))
                    || match row.tail {
                        ScopedRecordRowTail::Closed | ScopedRecordRowTail::Bound { .. } => false,
                        ScopedRecordRowTail::Anonymous(crate::types::type_lambda::ScopedAnonymousRecordRemainder::Unconstrained { key_type }) => {
                            self.visit_scoped_type(key_type)
                        }
                        ScopedRecordRowTail::Anonymous(crate::types::type_lambda::ScopedAnonymousRecordRemainder::Typed { key_type, value_type }) => {
                            self.visit_scoped_type(key_type) || self.visit_scoped_type(value_type)
                        }
                        ScopedRecordRowTail::FreeParameter(parameter) => self.parameters.contains(&parameter),
                        ScopedRecordRowTail::Projection(projection) => self.visit_type(projection),
                    }
            }
            ScopedTypeData::Callable(callable) => {
                callable.parameters.iter().any(|parameter| self.visit_scoped_type(parameter.ty)) || self.visit_scoped_type(callable.return_type)
            }
            ScopedTypeData::Pack(pack) => {
                pack.elements.iter().any(|element| self.visit_scoped_type(element.ty))
                    || match pack.positional_remainder {
                        ScopedPackRemainder::Typed(ty) => self.visit_scoped_type(ty),
                        ScopedPackRemainder::None | ScopedPackRemainder::Unconstrained => false,
                    }
                    || match pack.labeled_remainder {
                        ScopedPackRemainder::Typed(ty) => self.visit_scoped_type(ty),
                        ScopedPackRemainder::None | ScopedPackRemainder::Unconstrained => false,
                    }
            }
            ScopedTypeData::LabeledTypePack(elements) => elements.iter().any(|element| self.visit_scoped_type(element.element)),
            ScopedTypeData::CompleteTypePack { positionals, labeled }
            | ScopedTypeData::TupleFromPacks { positionals, labeled }
            | ScopedTypeData::PackFromPacks { positionals, labeled } => self.visit_scoped_type(positionals) || self.visit_scoped_type(labeled),
            ScopedTypeData::TypePackConcat(segments) => segments.iter().copied().any(|segment| self.visit_scoped_type(segment)),
            ScopedTypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => self.visit_scoped_type(positionals) || self.visit_scoped_type(labeled) || self.visit_scoped_type(return_type),
            ScopedTypeData::PackProjection { subject, .. } => self.visit_scoped_type(subject),
            ScopedTypeData::Lambda(lambda) => self.visit_lambda(lambda),
        }
    }
}

fn type_term_contains_any_parameter(store: &TypeStore, term: &TypeTerm, parameters: &[TypeParameterId]) -> bool {
    matches!(term, TypeTerm::Canonical(ty) if type_contains_any_parameter(store, *ty, parameters))
}

fn callable_signature_mentions_parameters(ctx: &CheckingContext<'_>, target: &CallableApplicationTarget, parameters: &[TypeParameterId]) -> bool {
    target.signature.parameters.iter().any(|parameter| match &parameter.contract {
        CallableParameterContract::Fixed(ty) => ty.ty().is_some_and(|ty| type_contains_any_parameter(ctx.store, ty, parameters)),
        CallableParameterContract::Rest { contract, .. } => match contract {
            RestValueContractKnowledge::Element(ty) => ty.ty().is_some_and(|ty| type_contains_any_parameter(ctx.store, ty, parameters)),
            RestValueContractKnowledge::Lane(lane_id) => type_contains_any_parameter(ctx.store, *lane_id, parameters),
            RestValueContractKnowledge::Unconstrained => false,
            RestValueContractKnowledge::Invalid(_) => false,
        },
    }) || target
        .signature
        .return_type
        .ty()
        .is_some_and(|ty| type_contains_any_parameter(ctx.store, ty, parameters))
        || target.signature.generics.as_ref().is_some_and(|signature| {
            signature.constraints.iter().any(|constraint| match constraint {
                GenericConstraint::Subtype { lower, upper } => {
                    type_term_contains_any_parameter(ctx.store, lower, parameters) || type_term_contains_any_parameter(ctx.store, upper, parameters)
                }
                GenericConstraint::Equivalent { left, right } => {
                    type_term_contains_any_parameter(ctx.store, left, parameters) || type_term_contains_any_parameter(ctx.store, right, parameters)
                }
                GenericConstraint::Conforms { subject, trait_ref } => {
                    type_term_contains_any_parameter(ctx.store, subject, parameters)
                        || trait_ref.arguments.iter().any(|&arg| type_contains_any_parameter(ctx.store, arg, parameters))
                }
            })
        })
}

fn effective_declaration_generics(ctx: &CheckingContext<'_>, target: &CallableApplicationTarget) -> Option<GenericSignature> {
    let declaration_generics = target.declaration_generics.clone().or_else(|| {
        (target.signature.kind == CallableSemanticKind::Constructor)
            .then(|| target.callable.as_ref().map(|callable| callable.declaration_owner().clone()))
            .flatten()
            .and_then(|owner| ctx.declaration_generic_signature(&owner))
    })?;
    let used_by_signature = callable_signature_mentions_parameters(ctx, target, &declaration_generics.parameters);
    let fixed_by_receiver = target
        .fixed_generics
        .iter()
        .any(|(parameter, _)| declaration_generics.parameters.contains(parameter));
    (used_by_signature || fixed_by_receiver).then_some(declaration_generics)
}

fn apply_generic_callable(
    ctx: &mut CheckingContext<'_>,
    target: &CallableApplicationTarget,
    _premise: &CallPremise,
    arguments: &[ApplicationArgument<'_>],
    expected: &ExpectedType,
    call_range: SourceRange,
) -> TypeKnowledge {
    let declaration_generics = effective_declaration_generics(ctx, target);
    let domains = GenericApplicationDomains {
        declaration: declaration_generics.as_ref(),
        callable: target.signature.generics.as_ref(),
    };
    apply_generic_callable_inner(
        ctx,
        target,
        &target.signature,
        target.callable.clone(),
        domains,
        &target.fixed_generics,
        arguments,
        expected,
        call_range,
    )
}

fn discharge_generic_conformance_obligations(
    ctx: &mut CheckingContext<'_>,
    domains: &GenericApplicationDomains<'_>,
    bindings: &std::collections::BTreeMap<TypeParameterId, crate::types::store::StaticArgument>,
    call_range: SourceRange,
) -> Option<Box<[crate::checker::analysis::GenericCallConformanceBinding]>> {
    let mut instantiation = crate::types::instantiation::GenericInstantiation::default();
    for (&param, &argument) in bindings {
        match argument {
            crate::types::store::StaticArgument::Term(ty) => instantiation.bind_type(param, ty),
            crate::types::store::StaticArgument::RecordRow(row) => instantiation.bind_row(param, row),
            crate::types::store::StaticArgument::DynamicType => instantiation.bind_dynamic(param),
        }
    }

    discharge_generic_obligations_with_instantiation(ctx, domains, &instantiation, call_range)
}

fn discharge_generic_obligations_with_instantiation(
    ctx: &mut CheckingContext<'_>,
    domains: &GenericApplicationDomains<'_>,
    instantiation: &crate::types::instantiation::GenericInstantiation,
    call_range: SourceRange,
) -> Option<Box<[crate::checker::analysis::GenericCallConformanceBinding]>> {
    let mut conformances = Vec::new();
    let mut failed = false;

    for signature in domains.signatures() {
        for constraint in signature.constraints.iter() {
            if generic_constraint_depends_on_dynamic(ctx.store, constraint, instantiation) {
                failed = true;
                let diagnostic = SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::GenericConstraintUnsatisfied,
                    "generic constraint cannot be proven for an explicit Dynamic argument",
                    call_range,
                );
                if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
                    ctx.record_call_status(AnalysisStatus::Invalid(cause));
                }
                continue;
            }
            match constraint {
                GenericConstraint::Conforms { subject, trait_ref } => {
                    let instantiated_subject = match subject {
                        TypeTerm::Canonical(ty) => {
                            match crate::types::instantiation::materialize_type(
                                ctx.store,
                                *ty,
                                instantiation,
                                crate::types::instantiation::RowMaterializationMode::PreserveUnboundStableTail,
                            ) {
                                Ok(m_ty) => TypeTerm::Canonical(m_ty),
                                Err(error) => {
                                    ctx.publish_analysis_incident(format!("conformance subject instantiation failed: {error:?}"));
                                    return None;
                                }
                            }
                        }
                        TypeTerm::SelfType(_) | TypeTerm::Infer(_) => subject.clone(),
                    };

                    let mut instantiated_args = Vec::with_capacity(trait_ref.arguments.len());
                    for &arg in trait_ref.arguments.iter() {
                        match crate::types::instantiation::materialize_type(
                            ctx.store,
                            arg,
                            instantiation,
                            crate::types::instantiation::RowMaterializationMode::PreserveUnboundStableTail,
                        ) {
                            Ok(m_arg) => instantiated_args.push(m_arg),
                            Err(error) => {
                                ctx.publish_analysis_incident(format!("conformance trait argument instantiation failed: {error:?}"));
                                return None;
                            }
                        }
                    }
                    let instantiated_trait_ref = crate::traits::TraitRef::new(trait_ref.declaration.clone(), instantiated_args);

                    let structural_ctx = match (ctx.data_table, ctx.enum_table) {
                        (Some(data), Some(enums)) => {
                            static IDS: std::sync::OnceLock<crate::core_surface::identity::StructuralUniverseIds> = std::sync::OnceLock::new();
                            let ids = IDS.get_or_init(crate::core_surface::identity::StructuralUniverseIds::new);
                            Some(crate::generic_evidence::StructuralConformanceContext {
                                classifier: crate::structural::StructuralClassifier::new(data, enums),
                                ids,
                            })
                        }
                        _ => None,
                    };

                    let proof = crate::generic_evidence::query_generic_conformance_proof_with_structural(
                        &instantiated_subject,
                        &instantiated_trait_ref,
                        Some(&ctx.generic_evidence),
                        ctx.conformance_semantics.map(|cs| cs.conformance_index),
                        ctx.conformance_semantics.map(|cs| cs.witness_plans),
                        ctx.conformance_semantics.map(|cs| cs.trait_surfaces),
                        ctx.conformance_semantics.map(|cs| cs.derived_conformance_rules),
                        Some(ctx.declarations),
                        Some(ctx.hierarchy.inner()),
                        Some(&mut ctx.store),
                        structural_ctx,
                    );
                    let conformance_key = crate::generic_evidence::GenericConformanceKey::new(instantiated_subject.clone(), instantiated_trait_ref.clone());
                    ctx.record_exact_conformance_query_dependency(&conformance_key, &proof);

                    match proof {
                        crate::generic_evidence::GenericConformanceProof::Proven(crate::generic_evidence::GenericConformanceEvidence::Exact(ev)) => {
                            ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::ConformanceDispatch(ev.source_impl.clone()));
                            ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                                instantiated_trait_ref.declaration.clone(),
                            ));
                            conformances.push(crate::checker::analysis::GenericCallConformanceBinding {
                                callee_conformance: conformance_key.clone(),
                                source: crate::checker::analysis::GenericCallConformanceSource::Exact { evidence: ev },
                            });
                        }
                        crate::generic_evidence::GenericConformanceProof::Proven(crate::generic_evidence::GenericConformanceEvidence::Assumed(assumed)) => {
                            ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                                instantiated_trait_ref.declaration.clone(),
                            ));
                            conformances.push(crate::checker::analysis::GenericCallConformanceBinding {
                                callee_conformance: conformance_key.clone(),
                                source: crate::checker::analysis::GenericCallConformanceSource::Forwarded {
                                    caller_conformance: assumed.key,
                                },
                            });
                        }
                        crate::generic_evidence::GenericConformanceProof::Proven(crate::generic_evidence::GenericConformanceEvidence::Derived(derived)) => {
                            match &derived.origin {
                                crate::generic_evidence::DerivedConformanceOrigin::ConditionalSource { exact_source } => {
                                    ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::ConformanceDispatch(
                                        exact_source.source_impl.clone(),
                                    ));
                                    ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                                        exact_source.exact_trait_ref.declaration.clone(),
                                    ));
                                }
                                crate::generic_evidence::DerivedConformanceOrigin::GuardedRule { rule, exact_trait_ref, .. } => {
                                    ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::ConformanceDispatch(rule.impl_id.clone()));
                                    ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                                        exact_trait_ref.declaration.clone(),
                                    ));
                                }
                            }
                            for nested in derived.nested.iter() {
                                if let crate::generic_evidence::NestedGenericEvidence::Conformance(
                                    crate::generic_evidence::GenericConformanceEvidence::Exact(ev),
                                ) = nested
                                {
                                    ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::ConformanceDispatch(ev.source_impl.clone()));
                                    ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                                        ev.exact_trait_ref.declaration.clone(),
                                    ));
                                }
                            }
                            conformances.push(crate::checker::analysis::GenericCallConformanceBinding {
                                callee_conformance: conformance_key.clone(),
                                source: crate::checker::analysis::GenericCallConformanceSource::Derived { evidence: derived },
                            });
                        }
                        crate::generic_evidence::GenericConformanceProof::Proven(crate::generic_evidence::GenericConformanceEvidence::CompilerDerived(ev)) => {
                            ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                                instantiated_trait_ref.declaration.clone(),
                            ));
                            conformances.push(crate::checker::analysis::GenericCallConformanceBinding {
                                callee_conformance: conformance_key.clone(),
                                source: crate::checker::analysis::GenericCallConformanceSource::CompilerDerived { evidence: ev },
                            });
                        }
                        _ => {
                            failed = true;
                            let message = format!(
                                "generic constraint unsatisfied: type does not conform to trait `{}`",
                                instantiated_trait_ref.declaration.name
                            );
                            let diagnostic =
                                SemanticDiagnostic::error_in(ctx.current_module.clone(), DiagnosticCode::GenericConstraintUnsatisfied, message, call_range);
                            if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
                                ctx.record_call_status(AnalysisStatus::Invalid(cause));
                            }
                        }
                    }
                }
                GenericConstraint::Equivalent { left, right } => {
                    let instantiated_left = match left {
                        TypeTerm::Canonical(ty) => match crate::types::instantiation::materialize_type(
                            ctx.store,
                            *ty,
                            instantiation,
                            crate::types::instantiation::RowMaterializationMode::PreserveUnboundStableTail,
                        ) {
                            Ok(m_ty) => m_ty,
                            Err(error) => {
                                ctx.publish_analysis_incident(format!("equivalent left instantiation failed: {error:?}"));
                                return None;
                            }
                        },
                        _ => continue,
                    };
                    let instantiated_right = match right {
                        TypeTerm::Canonical(ty) => match crate::types::instantiation::materialize_type(
                            ctx.store,
                            *ty,
                            instantiation,
                            crate::types::instantiation::RowMaterializationMode::PreserveUnboundStableTail,
                        ) {
                            Ok(m_ty) => m_ty,
                            Err(error) => {
                                ctx.publish_analysis_incident(format!("equivalent right instantiation failed: {error:?}"));
                                return None;
                            }
                        },
                        _ => continue,
                    };

                    let left_term = TypeTerm::Canonical(instantiated_left);
                    let right_term = TypeTerm::Canonical(instantiated_right);

                    let satisfied = if instantiated_left == instantiated_right || ctx.generic_evidence.are_equivalent(&left_term, &right_term).is_proven() {
                        true
                    } else {
                        let empty_surfaces = crate::traits::TraitSurfaceTable::new();
                        let mut budget = ctx.control.budget.borrow_mut();
                        let mut norm_context = crate::types::ProjectionNormalizationContext::new(
                            crate::types::ProjectionNormalizationMode::Generic {
                                evidence: &ctx.generic_evidence,
                                conformance_index: ctx.conformance_semantics.map(|cs| cs.conformance_index),
                                witness_plans: ctx.conformance_semantics.map(|cs| cs.witness_plans),
                                trait_surfaces: ctx.conformance_semantics.map(|cs| cs.trait_surfaces).unwrap_or(&empty_surfaces),
                                derived_rules: ctx.conformance_semantics.map(|cs| cs.derived_conformance_rules),
                                declarations: ctx.declarations,
                                hierarchy: ctx.hierarchy.inner(),
                            },
                            &mut budget,
                            &ctx.control.cancellation,
                        );
                        let norm_left = crate::types::normalize_type(ctx.store, instantiated_left, &mut norm_context)
                            .type_id()
                            .unwrap_or(instantiated_left);
                        let norm_right = crate::types::normalize_type(ctx.store, instantiated_right, &mut norm_context)
                            .type_id()
                            .unwrap_or(instantiated_right);
                        let norm_left_term = TypeTerm::Canonical(norm_left);
                        let norm_right_term = TypeTerm::Canonical(norm_right);
                        norm_left == norm_right
                            || ctx.generic_evidence.are_equivalent(&norm_left_term, &norm_right_term).is_proven()
                            || ctx.generic_evidence.are_types_equivalent(ctx.store, norm_left, norm_right).is_proven()
                    };

                    if !satisfied {
                        failed = true;
                        let diagnostic = SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::GenericConstraintUnsatisfied,
                            "generic constraint unsatisfied: associated type equality cannot be proven",
                            call_range,
                        );
                        if let Some(cause) = ctx.emit_diagnostic(diagnostic) {
                            ctx.record_call_status(AnalysisStatus::Invalid(cause));
                        }
                    }
                }
                GenericConstraint::Subtype { .. } => {}
            }
        }
    }

    if failed { None } else { Some(conformances.into_boxed_slice()) }
}

fn generic_constraint_depends_on_dynamic(
    store: &TypeStore,
    constraint: &GenericConstraint,
    instantiation: &crate::types::instantiation::GenericInstantiation,
) -> bool {
    let term_depends = |term: &TypeTerm| match term {
        TypeTerm::Canonical(ty) => instantiation.dynamic_bindings().any(|parameter| store.contains_type_parameter(*ty, parameter)),
        TypeTerm::SelfType(_) | TypeTerm::Infer(_) => false,
    };
    let type_depends = |ty| instantiation.dynamic_bindings().any(|parameter| store.contains_type_parameter(ty, parameter));
    match constraint {
        GenericConstraint::Conforms { subject, trait_ref } => term_depends(subject) || trait_ref.arguments.iter().copied().any(type_depends),
        GenericConstraint::Equivalent { left, right } | GenericConstraint::Subtype { lower: left, upper: right } => term_depends(left) || term_depends(right),
    }
}

fn exact_conformance_arguments(
    store: &crate::types::store::TypeStore,
    domains: &GenericApplicationDomains<'_>,
    exact_arguments: &std::collections::BTreeMap<TypeParameterId, TypeId>,
) -> Vec<(TypeParameterId, TypeId)> {
    exact_arguments.iter().filter_map(|(&parameter, &exact_case)| {
        domains.signatures().any(|signature| signature.constraints.iter().any(|constraint| {
            matches!(constraint, GenericConstraint::Conforms { subject: TypeTerm::Canonical(subject), .. } if store.contains_type_parameter(*subject, parameter))
        })).then_some((parameter, exact_case))
    }).collect()
}

/// Reuses mandatory declaration proof rules after ancestor evidence arrives.
pub(crate) fn validate_deferred_application_obligations(
    ctx: &mut CheckingContext<'_>,
    signatures: &[GenericSignature],
    instantiation: &crate::types::instantiation::GenericInstantiation,
    exact_arguments: &std::collections::BTreeMap<TypeParameterId, TypeId>,
    range: SourceRange,
) -> bool {
    signatures.iter().all(|signature| {
        // Scalar equalities/subtyping already have exact registered residual
        // identities. Exact-case refinement supplies only conformance authority.
        let mut conformances = signature.clone();
        conformances.constraints = signature
            .constraints
            .iter()
            .filter(|constraint| matches!(constraint, GenericConstraint::Conforms { .. }))
            .cloned()
            .collect();
        let domains = GenericApplicationDomains {
            declaration: None,
            callable: Some(&conformances),
        };
        let mut conformance_instantiation = instantiation.clone();
        for (parameter, exact_case) in exact_conformance_arguments(ctx.store, &domains, exact_arguments) {
            conformance_instantiation.bind_type(parameter, exact_case);
        }
        discharge_generic_obligations_with_instantiation(ctx, &domains, &conformance_instantiation, range).is_some()
    })
}

fn publish_call_specialization(
    ctx: &mut CheckingContext<'_>,
    callable: Option<CallableId>,
    domains: &GenericApplicationDomains<'_>,
    fixed_generics: &[(TypeParameterId, TypeId)],
    exact_argument_bindings: &std::collections::BTreeMap<TypeParameterId, TypeId>,
    type_terms: &std::collections::HashMap<TypeParameterId, InferenceTerm>,
    session_handle: &std::rc::Rc<std::cell::RefCell<crate::checker::inference::InferenceCoordinator>>,
    call_range: SourceRange,
) {
    let Some(callable) = callable else {
        return;
    };
    let Some(expression) = ctx.current_expression_id() else {
        return;
    };
    let mut bindings = std::collections::BTreeMap::new();
    for &(parameter, ty) in fixed_generics {
        bindings.insert(parameter, crate::types::store::StaticArgument::Term(ty));
    }
    for parameter in domains.signatures().flat_map(|signature| signature.parameters.iter()).copied() {
        if ctx.store.type_parameter(parameter).kind == crate::types::id::KindId::RECORD_ROW {
            return;
        }
        let ty = {
            let mut session = session_handle.borrow_mut();
            session.projected_solution(parameter, type_terms, ctx.store)
        };
        let Some(ty) = ty else {
            return;
        };
        bindings.insert(parameter, crate::types::store::StaticArgument::Term(ty));
    }
    bindings.extend(
        exact_conformance_arguments(ctx.store, domains, exact_argument_bindings)
            .into_iter()
            .map(|(parameter, ty)| (parameter, crate::types::store::StaticArgument::Term(ty))),
    );

    let Some(conformances) = discharge_generic_conformance_obligations(ctx, domains, &bindings, call_range) else {
        return;
    };

    // Application checks the trait requirement's contract, whose method-local
    // binders differ from the selected witness's source binders. Witness
    // compatibility has already established their positional alpha-equivalence.
    // Publish executable bindings in the selected callable's domain; the exact
    // conformance proofs above are already instantiated and remain unchanged.
    if let Some(required) = domains.callable
        && required.owner != crate::types::parameter::TypeParameterOwner::Callable(callable.clone())
    {
        let Some(selected) = ctx.resolve_callable_signature(&callable).and_then(|signature| signature.generics.as_ref()) else {
            ctx.publish_analysis_incident("selected generic witness has no canonical method signature");
            return;
        };
        if required.parameters.len() != selected.parameters.len() {
            ctx.publish_analysis_incident("selected witness generic arity differs from its checked requirement");
            return;
        }
        let renamed = required
            .parameters
            .iter()
            .zip(selected.parameters.iter())
            .filter_map(|(&source, &target)| bindings.get(&source).copied().map(|argument| (source, target, argument)))
            .collect::<Vec<_>>();
        for (source, _, _) in &renamed {
            bindings.remove(source);
        }
        for (_, target, ty) in renamed {
            bindings.insert(target, ty);
        }
    }

    let bindings = bindings.into_iter().collect::<Vec<_>>().into_boxed_slice();
    ctx.publish_call_specialization(
        expression,
        crate::checker::analysis::CallSpecialization {
            callable,
            bindings,
            conformances,
        },
    );
}

fn target_has_generic_domains(ctx: &CheckingContext<'_>, target: &CallableApplicationTarget) -> bool {
    effective_declaration_generics(ctx, target).is_some_and(|signature| !signature.parameters.is_empty())
        || target.signature.generics.as_ref().is_some_and(|signature| !signature.parameters.is_empty())
}

/// Complete the callee's lexical evidence contract without adding receiver-owned
/// parameters to method-local inference. Dispatch has already solved those bindings.
fn finalize_owner_call_specialization(ctx: &mut CheckingContext<'_>, target: &CallableApplicationTarget, call_range: SourceRange) {
    if target.authority != CallTargetAuthority::ExactDispatch {
        return;
    }
    let (Some(callable), Some(expression)) = (target.callable.as_ref(), ctx.current_expression_id()) else {
        return;
    };
    if callable.side != crate::identity::DispatchSide::Instance {
        return;
    }
    // Trait requirements own trait-header dependencies, not nominal declaration
    // shells. Only nominal owners contribute receiver-owned generic evidence.
    let Some(owner_id) = callable.try_declaration_owner().filter(|owner| ctx.declarations.get(owner).is_some()) else {
        return;
    };
    let Some(owner) = ctx.declaration_generic_signature(owner_id) else {
        return;
    };
    if owner.parameters.is_empty()
        && !owner
            .constraints
            .iter()
            .any(|constraint| matches!(constraint, GenericConstraint::Conforms { .. }))
    {
        return;
    }
    let existing = ctx.take_call_specialization(expression);
    let callable_has_generic_domains = target.signature.generics.as_ref().is_some_and(|signature| !signature.parameters.is_empty());
    if ctx.call_status_is_recorded() || (callable_has_generic_domains && existing.is_none()) {
        return;
    }
    let mut bindings = existing
        .map(|specialization| specialization.bindings.into_vec().into_iter().collect::<std::collections::BTreeMap<_, _>>())
        .unwrap_or_default();
    for &parameter in owner.parameters.iter() {
        let Some(argument) = target.specialization.as_ref().and_then(|dispatch| dispatch.environment.argument(parameter)) else {
            ctx.publish_analysis_incident("receiver specialization is missing a required owner binding");
            return;
        };
        let parameter_kind = ctx.store.type_parameter(parameter).kind;
        let category_matches = match argument {
            crate::types::store::StaticArgument::Term(ty) => ctx.store.kind_of(ty) == parameter_kind,
            crate::types::store::StaticArgument::RecordRow(_) => parameter_kind == KindId::RECORD_ROW,
            crate::types::store::StaticArgument::DynamicType => parameter_kind == KindId::TYPE,
        };
        if !category_matches {
            ctx.publish_analysis_incident("receiver specialization argument category does not match its owner parameter kind");
            return;
        }
        if bindings.insert(parameter, argument).is_some_and(|previous| previous != argument) {
            ctx.publish_analysis_incident("receiver and callable specialization disagree on an owner binding");
            return;
        }
    }
    let domains = GenericApplicationDomains {
        declaration: Some(&owner),
        callable: target.signature.generics.as_ref(),
    };
    let Some(conformances) = discharge_generic_conformance_obligations(ctx, &domains, &bindings, call_range) else {
        return;
    };
    ctx.publish_call_specialization(
        expression,
        crate::checker::analysis::CallSpecialization {
            callable: callable.clone(),
            bindings: bindings.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            conformances,
        },
    );
}

pub(crate) fn apply_resolved_callable(
    ctx: &mut CheckingContext<'_>,
    target: &CallableApplicationTarget,
    premise: &CallPremise,
    arguments: &[ApplicationArgument<'_>],
    expected: &ExpectedType,
    call_range: SourceRange,
) -> CallCheckResult {
    if target.access_denied {
        let mut result = analyze_unresolved_application(ctx, premise, arguments, UnresolvedApplicationReason::DispatchInaccessible);
        let cause = ctx.emit_diagnostic(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::MemberInaccessible,
            format!("member `{}` is not accessible from this declaration", target.signature.selector.encode()),
            call_range,
        ));
        if let Some(cause) = cause {
            result.status = AnalysisStatus::Invalid(cause);
            result.causal_invalidity = result.causal_invalidity.join(CausalInvalidity::One(cause));
        }
        return result;
    }
    ctx.begin_call_causal_capture();
    if let (Some(callable), Some(call_id), Some(specialization)) = (target.callable.clone(), ctx.current_expression_id(), target.specialization.as_ref()) {
        let selection = ctx.record_derivation(
            crate::explain::ExplanationStep::CallableSelection {
                callable: callable.clone(),
                receiver: specialization.receiver,
                declaring_owner: specialization.declaring_owner.clone(),
                specialization_path: specialization.path.iter().map(|step| step.owner.clone()).collect(),
            },
            crate::explain::DerivationRule::CallableSelection,
            EvidenceStatus::Established,
            EvidenceOrigin::DeclarationSemantics,
            vec![crate::explain::EvidenceRef::TypeId(specialization.receiver)],
            premise.explanation.into_iter().collect(),
        );
        let kind = ctx.record_derivation(
            crate::explain::ExplanationStep::CallableKind {
                callable: callable.clone(),
                kind: target.signature.kind,
            },
            crate::explain::DerivationRule::CallableSelection,
            EvidenceStatus::Established,
            target_fixed_return_origin(target),
            Vec::new(),
            vec![selection],
        );
        let mut root = kind;
        if let Some(unspecialized_ty) = specialization.unspecialized_return.ty() {
            let declared_return = ctx.record_derivation(
                crate::explain::ExplanationStep::CallableReturn {
                    callable: callable.clone(),
                    ty: unspecialized_ty,
                },
                crate::explain::DerivationRule::CallableReturn,
                specialization.unspecialized_return.status().unwrap_or(EvidenceStatus::Assumed),
                specialization.unspecialized_return.origin().unwrap_or(EvidenceOrigin::CallableSignature),
                vec![crate::explain::EvidenceRef::TypeId(unspecialized_ty)],
                vec![kind],
            );
            root = declared_return;
            if let Some(resolved_ty) = target.signature.return_type.ty() {
                if resolved_ty != unspecialized_ty {
                    root = ctx.record_derivation(
                        crate::explain::ExplanationStep::SelfTypeSpecialization {
                            self_ty: unspecialized_ty,
                            receiver: specialization.receiver,
                            resolved: resolved_ty,
                        },
                        crate::explain::DerivationRule::SelfSpecialization,
                        target.signature.return_type.status().unwrap_or(EvidenceStatus::Established),
                        target_fixed_return_origin(target),
                        vec![
                            crate::explain::EvidenceRef::TypeId(unspecialized_ty),
                            crate::explain::EvidenceRef::TypeId(resolved_ty),
                        ],
                        vec![declared_return],
                    );
                }
            }
        }
        let _ = call_id;
        ctx.record_call_dependency(CausalInvalidity::Clean, Some(root));
    }
    let (knowledge, symbolic_result) = if target_has_generic_domains(ctx, target) {
        let result = apply_generic_callable(ctx, target, premise, arguments, expected, call_range);
        let symbolic_result = ctx
            .current_expression_id()
            .and_then(|expression| ctx.take_symbolic_inference_result(expression));
        (cap_result_to_premise_authority(target, premise, result, call_range), symbolic_result)
    } else {
        let result = apply_non_generic_callable(ctx, target, premise, arguments, call_range);
        (result, None)
    };
    // Intrinsic construction preserves structural type identity. Keep inference's
    // nominal binder template private and publish the canonical proper Pack.
    let knowledge = if let Some(crate::identity::InvocationTargetId::DataConstructor(constructor)) = &target.target
        && ctx.declarations.intrinsic_runtime_owner(&constructor.owner) == Some(crate::declarations::IntrinsicRuntimeOwner::Pack)
    {
        knowledge.map_type(|ty| {
            let Some((_, arguments)) = ctx.store.applied_nominal_static_parts(ty) else {
                return ty;
            };
            let [crate::types::store::StaticArgument::Term(complete)] = arguments.as_slice() else {
                return ty;
            };
            let complete = *complete;
            let Ok(positionals) = ctx.store.project_positionals(complete) else {
                return ty;
            };
            let Ok(labeled) = ctx.store.project_labeled(complete) else { return ty };
            ctx.store.pack_from_packs(positionals, labeled).unwrap_or(ty)
        })
    } else {
        knowledge
    };
    if premise.status.is_ready() {
        finalize_owner_call_specialization(ctx, target, call_range);
    }
    let (captured_causal_invalidity, mut explanation_parents, captured_status, local_types) = ctx.end_call_causal_capture();
    let owning_cause = ctx.owning_cause_for_current_expression();
    let mut causal_invalidity = premise.causal_invalidity.join(captured_causal_invalidity);
    if let AnalysisStatus::Invalid(cause) = &premise.status {
        causal_invalidity = causal_invalidity.join(CausalInvalidity::One(*cause));
    }
    if let Some(AnalysisStatus::Invalid(cause)) = &captured_status {
        causal_invalidity = causal_invalidity.join(CausalInvalidity::One(*cause));
    }
    if let Some(cause) = owning_cause {
        causal_invalidity = causal_invalidity.join(CausalInvalidity::One(cause));
    }
    if let Some(explanation) = premise.explanation {
        if !explanation_parents.contains(&explanation) {
            explanation_parents.push(explanation);
        }
    }
    let status = owning_cause
        .map(AnalysisStatus::Invalid)
        .or(captured_status)
        .or_else(|| (!premise.status.is_ready()).then(|| premise.status.clone()))
        .unwrap_or_else(|| match &knowledge {
            TypeKnowledge::Dynamic(reason) => AnalysisStatus::DynamicBoundary(reason.clone()),
            _ => AnalysisStatus::Ready,
        });
    let local_type = local_type_from_arguments(ctx, knowledge.ty(), &local_types);
    let mut status = status;
    ctx.finish_call_sequence_publication(&mut status, causal_invalidity);
    let result = CallCheckResult {
        knowledge,
        status,
        causal_invalidity,
        explanation_parents,
        callable: target.callable.clone(),
        local_type,
        symbolic_result,
    };
    if !matches!(
        target.target,
        Some(crate::identity::InvocationTargetId::VariantConstructor(_) | crate::identity::InvocationTargetId::DataConstructor(_))
    ) {
        ctx.flow.invalidate_opaque_calls();
    }
    debug_assert_call_result_coherent(&result);
    result
}

fn union_status_priority(status: &AnalysisStatus) -> u8 {
    match status {
        AnalysisStatus::Ready => 0,
        AnalysisStatus::DynamicBoundary(_) => 1,
        AnalysisStatus::Suppressed(_) => 2,
        AnalysisStatus::Blocked(_) => 3,
        AnalysisStatus::BudgetExceeded(_) => 4,
        AnalysisStatus::Cancelled => 5,
        AnalysisStatus::InternalFailure(_) => 6,
        AnalysisStatus::Invalid(_) => 7,
    }
}

fn meet_union_status(status: &mut Option<AnalysisStatus>, candidate: AnalysisStatus) {
    if candidate.is_ready() {
        return;
    }
    if status
        .as_ref()
        .is_none_or(|current| union_status_priority(&candidate) > union_status_priority(current))
    {
        *status = Some(candidate);
    }
}

fn union_publication_knowledge(knowledge: TypeKnowledge, status: &AnalysisStatus) -> TypeKnowledge {
    if status.is_ready() {
        return knowledge;
    }
    match knowledge {
        TypeKnowledge::Dynamic(reason) => TypeKnowledge::Dynamic(reason),
        TypeKnowledge::Known(_) | TypeKnowledge::Unknown(_) => match status {
            AnalysisStatus::Blocked(_) | AnalysisStatus::BudgetExceeded(_) | AnalysisStatus::Cancelled => {
                TypeKnowledge::Unknown(UnknownReason::InferenceBlocked)
            }
            _ => TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause),
        },
    }
}

fn record_union_arm_explanation(
    ctx: &mut CheckingContext<'_>,
    receiver: TypeId,
    callable: Option<CallableId>,
    outcome: crate::explain::UnionArmOutcome,
    parents: Vec<ExplanationId>,
) -> ExplanationId {
    let (status, origin) = match &outcome {
        crate::explain::UnionArmOutcome::Dynamic { .. } => (EvidenceStatus::Assumed, EvidenceOrigin::DeclarationSemantics),
        crate::explain::UnionArmOutcome::Resolved => (EvidenceStatus::Established, EvidenceOrigin::DeclarationSemantics),
        _ => (EvidenceStatus::Assumed, EvidenceOrigin::DeclarationSemantics),
    };
    ctx.record_derivation(
        crate::explain::ExplanationStep::UnionArm { receiver, callable, outcome },
        crate::explain::DerivationRule::UnionArm,
        status,
        origin,
        vec![crate::explain::EvidenceRef::TypeId(receiver)],
        parents,
    )
}

fn emit_union_dispatch_failure(ctx: &mut CheckingContext<'_>, message: String, range: SourceRange) -> (AnalysisStatus, CausalInvalidity) {
    let cause = ctx
        .emit_diagnostic(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::TypeMismatch,
            message,
            range,
        ))
        .unwrap_or_else(|| ctx.alloc_diagnostic_cause());
    (AnalysisStatus::Invalid(cause), CausalInvalidity::One(cause))
}

struct UnionGenericInferenceScope {
    context: crate::checker::inference::InferenceContextId,
    root: crate::checker::inference::InferenceFrameId,
    owns_context: bool,
    probe_frame: crate::checker::inference::InferenceFrameId,
    typed_arguments: Vec<TypedExpression>,
    expected_for_arms: ExpectedType,
}

impl UnionGenericInferenceScope {
    fn finish(self, ctx: &mut CheckingContext<'_>, status: &mut AnalysisStatus, causal: CausalInvalidity, all_found: bool) {
        let probe = super::sequence_publication::PublicationOwner(self.context, self.probe_frame);
        let root = super::sequence_publication::PublicationOwner(self.context, self.root);
        if all_found && status.is_ready() && matches!(causal, CausalInvalidity::Clean) {
            if let Err(failure) = ctx.accept_sequence_owner(probe) {
                *status = failure;
            }
            if self.owns_context && status.is_ready() {
                if let Err(failure) = ctx.accept_sequence_owner(root) {
                    *status = failure;
                }
            }
        } else {
            ctx.reject_sequence_owner(probe);
            if self.owns_context {
                ctx.reject_sequence_owner(root);
            }
        }
        if self.owns_context {
            ctx.pop_inference_frame(self.context, self.root);
            if let Some(session) = ctx.inference_session(self.context) {
                session.borrow_mut().close_frame(self.root);
            }
            ctx.finish_inference_context(self.context);
        }
    }
}

/// Pre-analyzes union arguments against one representative generic signature.
///
/// Generic receiver arms have distinct declaration-owned parameter ids even
/// when their callable shapes are equivalent. Comparing those ids as proper
/// types would reject compatible contextual closures before arm inference can
/// specialize them. Keep one symbolic argument product in one live context;
/// each arm then relates that product to its own fresh application variables.
fn prepare_union_generic_inference(
    ctx: &mut CheckingContext<'_>,
    arms: &[UnionCallArm],
    arguments: &[ApplicationArgument<'_>],
    expected: &ExpectedType,
) -> Option<UnionGenericInferenceScope> {
    if !arguments
        .iter()
        .any(|argument| argument.expression().is_some_and(|expression| matches!(expression, Expr::Block(_))))
    {
        return None;
    }
    let target = arms.iter().find_map(|arm| match arm {
        UnionCallArm::Found { target, .. } if target.signature.generics.as_ref().is_some_and(|generics| !generics.parameters.is_empty()) => Some(target),
        _ => None,
    })?;
    let generic_sig = target.signature.generics.as_ref()?.clone();
    let (context, root, owns_context) = match expected {
        ExpectedType::Inference { context, .. } => {
            let parent = ctx.current_inference_frame(*context)?;
            (*context, parent, false)
        }
        ExpectedType::Proper { ty, .. } => {
            let (context, root) = ctx.create_inference_context();
            ctx.push_inference_frame(context, root);
            let expected_for_arms = ExpectedType::inference_from(context, InferenceTerm::Canonical(*ty), ExpectationOrigin::GenericResult);
            return prepare_union_generic_inference_in_context(ctx, target, arguments, generic_sig, context, root, true, expected_for_arms);
        }
        ExpectedType::None => return None,
    };

    let expected_for_arms = expected.clone();
    prepare_union_generic_inference_in_context(ctx, target, arguments, generic_sig, context, root, owns_context, expected_for_arms)
}

#[allow(clippy::too_many_arguments)]
fn prepare_union_generic_inference_in_context(
    ctx: &mut CheckingContext<'_>,
    target: &CallableApplicationTarget,
    arguments: &[ApplicationArgument<'_>],
    generic_sig: crate::types::parameter::GenericSignature,
    context: crate::checker::inference::InferenceContextId,
    root: crate::checker::inference::InferenceFrameId,
    owns_context: bool,
    expected_for_arms: ExpectedType,
) -> Option<UnionGenericInferenceScope> {
    let Some(session_handle) = ctx.inference_session(context) else {
        if owns_context {
            ctx.pop_inference_frame(context, root);
            ctx.finish_inference_context(context);
        }
        return None;
    };
    let Some(probe_frame) = ctx.begin_inference_frame(context, root) else {
        if owns_context {
            ctx.pop_inference_frame(context, root);
            ctx.finish_inference_context(context);
        }
        return None;
    };
    ctx.push_inference_frame(context, probe_frame);
    let var_map = session_handle
        .borrow_mut()
        .instantiate_generic_signature_in_frame(&generic_sig, ctx.store, probe_frame);
    let binding_plan = match bind_static_arguments(arguments, &target.signature.parameters) {
        Ok(plan) => plan,
        Err(_) => {
            ctx.reject_sequence_owner(super::sequence_publication::PublicationOwner(context, probe_frame));
            ctx.pop_inference_frame(context, probe_frame);
            session_handle.borrow_mut().close_frame(probe_frame);
            if owns_context {
                ctx.pop_inference_frame(context, root);
                ctx.finish_inference_context(context);
            }
            return None;
        }
    };
    let call_id = ctx.current_expression_id();
    let mut typed_arguments = Vec::with_capacity(arguments.len());
    for (argument_index, argument) in arguments.iter().copied().enumerate() {
        let Some(binding) = binding_plan.bindings.iter().find(|binding| binding.argument_index == argument_index) else {
            continue;
        };
        let parameter = &target.signature.parameters[binding.parameter_index];
        let Some(parameter_ty) = parameter.ty.ty() else {
            ctx.reject_sequence_owner(super::sequence_publication::PublicationOwner(context, probe_frame));
            ctx.pop_inference_frame(context, probe_frame);
            session_handle.borrow_mut().close_frame(probe_frame);
            if owns_context {
                ctx.pop_inference_frame(context, root);
                ctx.finish_inference_context(context);
            }
            return None;
        };
        let parameter_term = session_handle.borrow().type_id_to_inference(parameter_ty, &var_map, ctx.store);
        let expected_term = session_handle.borrow().term_for_expected(&parameter_term);
        let argument_expected = session_handle.borrow().materialize_for_expected(&expected_term, ctx.store).map_or_else(
            || ExpectedType::inference_from(context, expected_term, ExpectationOrigin::GenericArgument),
            |ty| ExpectedType::proper_from(ty, ExpectationOrigin::GenericArgument),
        );
        let typed = analyze_application_argument_owned(
            ctx,
            argument,
            &argument_expected,
            Some(super::sequence_publication::PublicationOwner(context, probe_frame)),
        );
        record_generic_argument_capture(ctx, &typed);
        let explanation = typed.expression_id.and_then(|id| ctx.explanation_for_expression(id));
        let origin = match (call_id, typed.expression_id) {
            (Some(call), Some(argument_id)) => ConstraintOrigin::Argument {
                call,
                argument: argument_id,
                parameter_index: binding.parameter_index as u16,
            },
            _ => ConstraintOrigin::Explicit,
        };
        let symbolic_argument = typed
            .symbolic_result
            .as_ref()
            .filter(|result| result.context == context)
            .map(|result| result.term.clone());
        if let Some(symbolic_argument) = symbolic_argument {
            let symbolic_argument = session_handle.borrow().term_for_expected(&symbolic_argument);
            session_handle.borrow_mut().add_constraint_with_support(
                InferenceRelation::Subtype(symbolic_argument, parameter_term.clone()),
                origin.clone(),
                explanation,
                InferenceSupport::Assumed,
            );
        } else {
            session_handle
                .borrow_mut()
                .record_required_premise(&parameter_term, origin.clone(), &typed.knowledge, explanation);
            if let TypeKnowledge::Known(evidence) = &typed.knowledge {
                if let Some(support) = inference_support(&typed.knowledge) {
                    let argument_term = session_handle
                        .borrow()
                        .type_id_to_inference(evidence.ty(), &std::collections::HashMap::new(), ctx.store);
                    session_handle.borrow_mut().add_constraint_with_support(
                        InferenceRelation::Subtype(argument_term, parameter_term),
                        origin,
                        explanation,
                        support,
                    );
                }
            } else if let TypeKnowledge::Unknown(reason) = &typed.knowledge {
                ctx.record_call_status(AnalysisStatus::Blocked(crate::types::outcome::BlockReason::UnknownType(reason.clone())));
            } else if let TypeKnowledge::Dynamic(reason) = &typed.knowledge {
                ctx.record_call_status(AnalysisStatus::DynamicBoundary(reason.clone()));
            }
        }
        typed_arguments.push(typed);
        if argument_index + 1 < arguments.len() {
            let _ = ctx.inference_session(context).map(|session| ctx.solve_inference(&mut session.borrow_mut()));
        }
    }
    let _ = ctx.inference_session(context).map(|session| ctx.solve_inference(&mut session.borrow_mut()));
    ctx.pop_inference_frame(context, probe_frame);
    session_handle.borrow_mut().close_frame(probe_frame);
    Some(UnionGenericInferenceScope {
        context,
        root,
        owns_context,
        probe_frame,
        typed_arguments,
        expected_for_arms,
    })
}

/// Applies one source call to every statically reachable union receiver arm.
/// Receiver dispatch is resolved before this function, while argument ASTs are
/// analyzed once and reused through `PreAnalyzed` application arguments.
pub(crate) fn apply_union_resolved_call(
    ctx: &mut CheckingContext<'_>,
    premise: &CallPremise,
    arms: &[UnionCallArm],
    arguments: &[ApplicationArgument<'_>],
    expected: &ExpectedType,
    call_range: SourceRange,
    selector: &Selector,
) -> CallCheckResult {
    ctx.begin_call_causal_capture();

    let shared_scope = prepare_union_generic_inference(ctx, arms, arguments, expected);
    // Ordinary arguments are also shared by every arm. Their construction
    // capability waits for the aggregate call even when no shared solver is
    // needed to supply a contextual closure expectation.
    let argument_owner = shared_scope.is_none().then(|| {
        let (context, frame) = ctx.create_inference_context();
        super::sequence_publication::PublicationOwner(context, frame)
    });
    let mut contextual_conflicts = Vec::new();
    let mut common_expectations = vec![None; arguments.len()];
    let mut expectation_counts = vec![0usize; arguments.len()];
    let mut found_counts = vec![0usize; arguments.len()];
    let mut expectation_conflicts = vec![false; arguments.len()];
    let typed_arguments = if let Some(scope) = shared_scope.as_ref() {
        scope.typed_arguments.clone()
    } else {
        for arm in arms {
            let UnionCallArm::Found { target, .. } = arm else {
                continue;
            };
            let Ok(plan) = bind_static_arguments(arguments, &target.signature.parameters) else {
                continue;
            };
            for argument_index in 0..arguments.len() {
                let Some(parameter) = parameter_for_argument(&plan, argument_index, &target.signature.parameters) else {
                    continue;
                };
                found_counts[argument_index] += 1;
                let Some(parameter_ty) = parameter.ty.ty() else {
                    continue;
                };
                expectation_counts[argument_index] += 1;
                match common_expectations[argument_index] {
                    None => common_expectations[argument_index] = Some(parameter_ty),
                    Some(common) if common != parameter_ty => expectation_conflicts[argument_index] = true,
                    Some(_) => {}
                }
            }
        }
        contextual_conflicts = arguments
            .iter()
            .enumerate()
            .filter_map(|(index, argument)| {
                (expectation_conflicts[index]
                    && found_counts[index] > 0
                    && expectation_counts[index] == found_counts[index]
                    && argument.expression().is_some_and(|expression| matches!(expression, Expr::Block(_))))
                .then_some(index)
            })
            .collect();

        let mut typed_arguments = Vec::with_capacity(arguments.len());
        for (index, argument) in arguments.iter().copied().enumerate() {
            let argument_expected = if contextual_conflicts.contains(&index) {
                ExpectedType::None
            } else if !expectation_conflicts[index] && expectation_counts[index] == found_counts[index] {
                common_expectations[index]
                    .map(|ty| ExpectedType::proper_from(ty, ExpectationOrigin::CallableSignature))
                    .unwrap_or_default()
            } else {
                ExpectedType::None
            };
            let typed = analyze_application_argument_owned(ctx, argument, &argument_expected, argument_owner);
            record_generic_argument_capture(ctx, &typed);
            typed_arguments.push(typed);
        }
        typed_arguments
    };
    let arm_expected = shared_scope
        .as_ref()
        .map(|scope| scope.expected_for_arms.clone())
        .unwrap_or_else(|| expected.clone());
    let pre_analyzed = arguments
        .iter()
        .enumerate()
        .map(|(index, argument)| {
            let label = match argument {
                ApplicationArgument::Labeled { label, .. } => Some(*label),
                _ => None,
            };
            ApplicationArgument::PreAnalyzed {
                label,
                typed: &typed_arguments[index],
                range: argument.range(),
            }
        })
        .collect::<Vec<_>>();

    let context_cause = if contextual_conflicts.is_empty() {
        None
    } else {
        let positions = contextual_conflicts.iter().map(|index| (index + 1).to_string()).collect::<Vec<_>>().join(", ");
        let (status, causal) = emit_union_dispatch_failure(
            ctx,
            format!("union receiver arms require incompatible contextual closure expectations for argument position(s) {positions}"),
            call_range,
        );
        let cause = match status {
            AnalysisStatus::Invalid(cause) => cause,
            _ => unreachable!("union contextual conflict emits an error"),
        };
        ctx.record_call_dependency(causal, None);
        Some(cause)
    };

    let mut status = None;
    if !premise.status.is_ready() {
        meet_union_status(&mut status, premise.status.clone());
    }
    let mut causal_invalidity = premise.causal_invalidity;
    let mut explanation_parents = Vec::new();
    let mut publication_inputs = Vec::with_capacity(arms.len() + 1);
    let mut first_callable = None;
    let mut have_callable = false;
    let mut callable_consistent = true;
    let mut all_found = true;

    if !premise.status.is_ready() {
        publication_inputs.push(union_publication_knowledge(premise.knowledge.clone(), &premise.status));
    }

    for arm in arms {
        match arm {
            UnionCallArm::Found { receiver, target } => {
                if let Some(cause) = context_cause {
                    let arm_status = AnalysisStatus::Invalid(cause);
                    let arm_explanation = record_union_arm_explanation(
                        ctx,
                        *receiver,
                        target.callable.clone(),
                        crate::explain::UnionArmOutcome::ContextConflict,
                        Vec::new(),
                    );
                    explanation_parents.push(arm_explanation);
                    meet_union_status(&mut status, arm_status.clone());
                    causal_invalidity = causal_invalidity.join(CausalInvalidity::One(cause));
                    ctx.record_call_dependency(CausalInvalidity::One(cause), Some(arm_explanation));
                    publication_inputs.push(TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause));
                    all_found = false;
                    continue;
                }

                let result = apply_resolved_callable(ctx, target, premise, &pre_analyzed, &arm_expected, call_range);
                let outcome = if result.status.is_ready() {
                    crate::explain::UnionArmOutcome::Resolved
                } else {
                    crate::explain::UnionArmOutcome::Invalid
                };
                let arm_explanation = record_union_arm_explanation(ctx, *receiver, target.callable.clone(), outcome, result.explanation_parents.clone());
                explanation_parents.push(arm_explanation);
                for parent in result.explanation_parents.iter().copied() {
                    ctx.record_call_dependency(CausalInvalidity::Clean, Some(parent));
                }
                ctx.record_call_dependency(result.causal_invalidity, Some(arm_explanation));
                causal_invalidity = causal_invalidity.join(result.causal_invalidity);
                meet_union_status(&mut status, result.status.clone());
                publication_inputs.push(union_publication_knowledge(result.knowledge, &result.status));
                if result.status.is_ready() {
                    if !have_callable {
                        first_callable = Some(result.callable);
                        have_callable = true;
                    } else if first_callable.as_ref().and_then(|callable| callable.as_ref()) != result.callable.as_ref() {
                        callable_consistent = false;
                    }
                } else {
                    all_found = false;
                }
            }
            UnionCallArm::Missing { receiver, visited_owners } => {
                let (arm_status, arm_causal) = emit_union_dispatch_failure(
                    ctx,
                    format!(
                        "union receiver arm `{}` does not support selector `{selector}`",
                        ctx.store.format_type(*receiver)
                    ),
                    call_range,
                );
                let cause = match arm_status {
                    AnalysisStatus::Invalid(cause) => cause,
                    _ => unreachable!("union missing arm emits an error"),
                };
                let arm_explanation = record_union_arm_explanation(
                    ctx,
                    *receiver,
                    None,
                    crate::explain::UnionArmOutcome::Missing {
                        visited_owners: visited_owners.clone(),
                    },
                    Vec::new(),
                );
                explanation_parents.push(arm_explanation);
                ctx.record_call_dependency(arm_causal, Some(arm_explanation));
                causal_invalidity = causal_invalidity.join(arm_causal);
                meet_union_status(&mut status, AnalysisStatus::Invalid(cause));
                publication_inputs.push(TypeKnowledge::Unknown(UnknownReason::DynamicMessageSend));
                all_found = false;
            }
            UnionCallArm::Ambiguous { receiver } => {
                let (arm_status, arm_causal) = emit_union_dispatch_failure(
                    ctx,
                    format!("union receiver arm `{}` has ambiguous selector `{selector}`", ctx.store.format_type(*receiver)),
                    call_range,
                );
                let cause = match arm_status {
                    AnalysisStatus::Invalid(cause) => cause,
                    _ => unreachable!("union ambiguous arm emits an error"),
                };
                let arm_explanation = record_union_arm_explanation(ctx, *receiver, None, crate::explain::UnionArmOutcome::Ambiguous, Vec::new());
                explanation_parents.push(arm_explanation);
                ctx.record_call_dependency(arm_causal, Some(arm_explanation));
                causal_invalidity = causal_invalidity.join(arm_causal);
                meet_union_status(&mut status, AnalysisStatus::Invalid(cause));
                publication_inputs.push(TypeKnowledge::Unknown(UnknownReason::DynamicMessageSend));
                all_found = false;
            }
            UnionCallArm::Dynamic { receiver, reason } => {
                let arm_explanation = record_union_arm_explanation(
                    ctx,
                    *receiver,
                    None,
                    crate::explain::UnionArmOutcome::Dynamic { reason: reason.clone() },
                    Vec::new(),
                );
                explanation_parents.push(arm_explanation);
                ctx.record_call_dependency(CausalInvalidity::Clean, Some(arm_explanation));
                meet_union_status(&mut status, AnalysisStatus::DynamicBoundary(reason.clone()));
                publication_inputs.push(TypeKnowledge::Dynamic(reason.clone()));
                all_found = false;
            }
            UnionCallArm::TraitTerminal { receiver, terminal } => {
                let knowledge = match terminal {
                    crate::trait_dispatch::TraitDispatchTerminal::Unknown(reason) => TypeKnowledge::Unknown(reason.clone()),
                    crate::trait_dispatch::TraitDispatchTerminal::Dynamic(_) => TypeKnowledge::Dynamic(DynamicReason::RuntimeReflection),
                    crate::trait_dispatch::TraitDispatchTerminal::Cancelled => TypeKnowledge::Unknown(UnknownReason::InferenceCancelled),
                    crate::trait_dispatch::TraitDispatchTerminal::BudgetExceeded(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBudgetExceeded),
                    crate::trait_dispatch::TraitDispatchTerminal::Blocked(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
                    crate::trait_dispatch::TraitDispatchTerminal::Incomplete(_)
                    | crate::trait_dispatch::TraitDispatchTerminal::CoherenceConflict(_)
                    | crate::trait_dispatch::TraitDispatchTerminal::InternalFailure(_) => TypeKnowledge::Unknown(UnknownReason::UncheckedExpression),
                };
                let arm_status = match terminal {
                    crate::trait_dispatch::TraitDispatchTerminal::Blocked(reason) => Some(AnalysisStatus::Blocked(reason.clone())),
                    crate::trait_dispatch::TraitDispatchTerminal::Dynamic(_) => Some(AnalysisStatus::DynamicBoundary(DynamicReason::RuntimeReflection)),
                    crate::trait_dispatch::TraitDispatchTerminal::Cancelled => Some(AnalysisStatus::Cancelled),
                    crate::trait_dispatch::TraitDispatchTerminal::BudgetExceeded(report) => Some(AnalysisStatus::BudgetExceeded(report.clone())),
                    crate::trait_dispatch::TraitDispatchTerminal::InternalFailure(message) => {
                        let incident = ctx.record_internal_incident(
                            InternalSemanticIncidentKind::DatabaseInvariantViolation,
                            InternalSemanticIncidentDetails::Message { message: message.clone() },
                            Some(call_range),
                        );
                        Some(AnalysisStatus::InternalFailure(incident))
                    }
                    crate::trait_dispatch::TraitDispatchTerminal::Incomplete(_)
                    | crate::trait_dispatch::TraitDispatchTerminal::CoherenceConflict(_)
                    | crate::trait_dispatch::TraitDispatchTerminal::Unknown(_) => None,
                };
                if let Some(arm_status) = arm_status {
                    meet_union_status(&mut status, arm_status);
                }
                let arm_explanation = record_union_arm_explanation(ctx, *receiver, None, crate::explain::UnionArmOutcome::Invalid, Vec::new());
                explanation_parents.push(arm_explanation);
                publication_inputs.push(knowledge);
                all_found = false;
            }
        }
    }

    let (captured_causal, captured_explanations, captured_status, local_types) = ctx.end_call_causal_capture();
    causal_invalidity = causal_invalidity.join(captured_causal);
    explanation_parents.extend(captured_explanations);
    if let Some(captured_status) = captured_status {
        meet_union_status(&mut status, captured_status);
    }
    if let Some(premise_explanation) = premise.explanation {
        if !explanation_parents.contains(&premise_explanation) {
            explanation_parents.push(premise_explanation);
        }
    }
    let knowledge = join_type_knowledge(ctx.store, publication_inputs);
    let status = status.unwrap_or_else(|| match &knowledge {
        TypeKnowledge::Dynamic(reason) => AnalysisStatus::DynamicBoundary(reason.clone()),
        _ => AnalysisStatus::Ready,
    });
    if let AnalysisStatus::Invalid(cause) = status {
        causal_invalidity = causal_invalidity.join(CausalInvalidity::One(cause));
    }
    let mut status = status;
    if let Some(scope) = shared_scope {
        scope.finish(ctx, &mut status, causal_invalidity, all_found);
    }
    if let Some(owner) = argument_owner {
        if all_found && status.is_ready() && matches!(causal_invalidity, CausalInvalidity::Clean) {
            if let Err(failure) = ctx.accept_sequence_owner(owner) {
                status = failure;
            }
        } else {
            ctx.reject_sequence_owner(owner);
        }
        ctx.finish_inference_context(owner.0);
    }
    let local_type = local_type_from_arguments(ctx, knowledge.ty(), &local_types);
    let result = CallCheckResult {
        knowledge,
        status,
        causal_invalidity,
        explanation_parents,
        callable: (all_found && callable_consistent).then_some(first_callable).flatten().flatten(),
        local_type,
        symbolic_result: None,
    };
    ctx.flow.invalidate_opaque_calls();
    debug_assert_call_result_coherent(&result);
    result
}

fn analyze_unbound_arguments(ctx: &mut CheckingContext<'_>, arguments: &[ApplicationArgument<'_>]) -> CallCapture {
    ctx.begin_call_causal_capture();
    for argument in arguments.iter().copied() {
        let typed = analyze_application_argument(ctx, argument, &ExpectedType::None);
        if !typed.status.is_ready() {
            ctx.record_call_status(typed.status);
        }
    }
    ctx.end_call_causal_capture()
}

#[derive(Clone, Debug)]
pub(crate) enum UnresolvedApplicationReason {
    TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal),
    PremiseUnknown,
    PremiseInvalidUnavailable,
    PremiseDynamic(DynamicReason),
    DispatchMissing,
    DispatchInaccessible,
    DispatchAmbiguous(SourceRange),
    DynamicShape(DynamicReason),
}

pub(crate) fn analyze_unresolved_application(
    ctx: &mut CheckingContext<'_>,
    premise: &CallPremise,
    arguments: &[ApplicationArgument<'_>],
    reason: UnresolvedApplicationReason,
) -> CallCheckResult {
    let (argument_invalidity, mut explanation_parents, argument_status, local_types) = analyze_unbound_arguments(ctx, arguments);
    if let Some(explanation) = premise.explanation {
        if !explanation_parents.contains(&explanation) {
            explanation_parents.push(explanation);
        }
    }
    let mut causal_invalidity = premise.causal_invalidity.join(argument_invalidity);
    let knowledge = match &reason {
        UnresolvedApplicationReason::PremiseUnknown => premise.knowledge.clone(),
        UnresolvedApplicationReason::PremiseInvalidUnavailable => TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause),
        UnresolvedApplicationReason::PremiseDynamic(reason) | UnresolvedApplicationReason::DynamicShape(reason) => TypeKnowledge::Dynamic(reason.clone()),
        UnresolvedApplicationReason::DispatchMissing
        | UnresolvedApplicationReason::DispatchInaccessible
        | UnresolvedApplicationReason::DispatchAmbiguous(_) => TypeKnowledge::Unknown(UnknownReason::DynamicMessageSend),
        UnresolvedApplicationReason::TraitTerminal(terminal) => match terminal {
            crate::trait_dispatch::TraitDispatchTerminal::Unknown(reason) => TypeKnowledge::Unknown(reason.clone()),
            crate::trait_dispatch::TraitDispatchTerminal::Dynamic(_) => TypeKnowledge::Dynamic(DynamicReason::RuntimeReflection),
            crate::trait_dispatch::TraitDispatchTerminal::Cancelled => TypeKnowledge::Unknown(UnknownReason::InferenceCancelled),
            crate::trait_dispatch::TraitDispatchTerminal::BudgetExceeded(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBudgetExceeded),
            crate::trait_dispatch::TraitDispatchTerminal::Blocked(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
            crate::trait_dispatch::TraitDispatchTerminal::Incomplete(_)
            | crate::trait_dispatch::TraitDispatchTerminal::CoherenceConflict(_)
            | crate::trait_dispatch::TraitDispatchTerminal::InternalFailure(_) => TypeKnowledge::Unknown(UnknownReason::UncheckedExpression),
        },
    };
    let terminal_status = match &reason {
        UnresolvedApplicationReason::TraitTerminal(terminal) => match terminal {
            crate::trait_dispatch::TraitDispatchTerminal::Blocked(reason) => Some(AnalysisStatus::Blocked(reason.clone())),
            crate::trait_dispatch::TraitDispatchTerminal::Dynamic(_) => Some(AnalysisStatus::DynamicBoundary(DynamicReason::RuntimeReflection)),
            crate::trait_dispatch::TraitDispatchTerminal::Cancelled => Some(AnalysisStatus::Cancelled),
            crate::trait_dispatch::TraitDispatchTerminal::BudgetExceeded(report) => Some(AnalysisStatus::BudgetExceeded(report.clone())),
            crate::trait_dispatch::TraitDispatchTerminal::InternalFailure(message) => {
                let incident = ctx.record_internal_incident(
                    InternalSemanticIncidentKind::DatabaseInvariantViolation,
                    InternalSemanticIncidentDetails::Message { message: message.clone() },
                    None,
                );
                Some(AnalysisStatus::InternalFailure(incident))
            }
            crate::trait_dispatch::TraitDispatchTerminal::Incomplete(_)
            | crate::trait_dispatch::TraitDispatchTerminal::CoherenceConflict(_)
            | crate::trait_dispatch::TraitDispatchTerminal::Unknown(_) => None,
        },
        _ => None,
    };
    let ambiguity_cause = match reason {
        UnresolvedApplicationReason::DispatchAmbiguous(range) => emit_trait_dispatch_ambiguity(ctx, range),
        _ => None,
    };
    if let Some(cause) = ambiguity_cause {
        causal_invalidity = causal_invalidity.join(CausalInvalidity::One(cause));
    }
    let status = argument_status
        .or(terminal_status)
        .or_else(|| ambiguity_cause.map(AnalysisStatus::Invalid))
        .unwrap_or_else(|| match &reason {
            UnresolvedApplicationReason::PremiseInvalidUnavailable => premise
                .causal_invalidity
                .suppression_cause()
                .map(AnalysisStatus::Suppressed)
                .unwrap_or(AnalysisStatus::Ready),
            UnresolvedApplicationReason::PremiseDynamic(reason) | UnresolvedApplicationReason::DynamicShape(reason) => {
                AnalysisStatus::DynamicBoundary(reason.clone())
            }
            _ => AnalysisStatus::Ready,
        });
    let local_type = local_type_from_arguments(ctx, knowledge.ty(), &local_types);
    let result = CallCheckResult {
        knowledge,
        status,
        causal_invalidity,
        explanation_parents,
        callable: None,
        local_type,
        symbolic_result: None,
    };
    ctx.flow.invalidate_opaque_calls();
    debug_assert_call_result_coherent(&result);
    result
}

fn emit_trait_dispatch_ambiguity(ctx: &mut CheckingContext<'_>, range: SourceRange) -> Option<crate::identity::DiagnosticCauseId> {
    let candidates = ctx.trait_dispatch_candidates_for_current_expression()?;
    if candidates.is_empty() {
        return None;
    }

    let mut diagnostic = SemanticDiagnostic::error_in(
        ctx.current_module.clone(),
        DiagnosticCode::TraitDispatchAmbiguous,
        "trait dispatch is ambiguous; multiple proven conformance witnesses provide this member",
        range,
    );
    let mut labels = Vec::new();
    for (index, candidate) in candidates.iter().enumerate() {
        diagnostic = diagnostic.with_note(format!(
            "candidate {}: exact target {:?}, trait {:?}, selector {:?}, requirement {:?}, source impl {:?}, selection {:?}",
            index + 1,
            candidate.exact_target,
            candidate.exact_trait_ref,
            candidate.requirement.selector,
            candidate.requirement,
            candidate.source_impl,
            candidate.selection,
        ));
        if let Some(source) = ctx.conformance_semantics.and_then(|view| view.conformance_index.get(&candidate.source_impl)) {
            labels.push((
                source.source.module.clone(),
                source.source.range,
                format!("candidate {} conformance", index + 1),
            ));
        }
    }
    for (module, label_range, message) in labels {
        diagnostic = diagnostic.with_label_in(module, label_range, message);
    }
    ctx.emit_diagnostic(diagnostic)
}

pub(crate) fn analyze_non_callable_invocation(
    ctx: &mut CheckingContext<'_>,
    premise: &CallPremise,
    args: &[PackItem],
    call_range: SourceRange,
) -> CallCheckResult {
    let mut result = analyze_unresolved_application(ctx, premise, &application_arguments(args), UnresolvedApplicationReason::DispatchMissing);
    let cause = ctx.emit_diagnostic(SemanticDiagnostic::error_in(
        ctx.current_module.clone(),
        DiagnosticCode::NotCallable,
        "value is not callable",
        call_range,
    ));
    if let Some(cause) = cause {
        result.status = AnalysisStatus::Invalid(cause);
        result.causal_invalidity = result.causal_invalidity.join(CausalInvalidity::One(cause));
    }
    debug_assert_call_result_coherent(&result);
    result
}

fn terminal_generic_return(outcome: &crate::checker::inference::InferenceOutcome, fixed_return: Option<TypeKnowledge>) -> TypeKnowledge {
    if let Some(fixed_return) = fixed_return {
        return fixed_return;
    }
    match outcome {
        crate::checker::inference::InferenceOutcome::Underconstrained(_) => TypeKnowledge::Unknown(UnknownReason::UnderconstrainedTypeVariable),
        crate::checker::inference::InferenceOutcome::Ambiguous(_) => TypeKnowledge::Unknown(UnknownReason::InferenceAmbiguous),
        crate::checker::inference::InferenceOutcome::Conflicting(_) => TypeKnowledge::Unknown(UnknownReason::InferenceConflict),
        crate::checker::inference::InferenceOutcome::Blocked(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
        crate::checker::inference::InferenceOutcome::Cancelled => TypeKnowledge::Unknown(UnknownReason::InferenceCancelled),
        crate::checker::inference::InferenceOutcome::BudgetExceeded(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBudgetExceeded),
        crate::checker::inference::InferenceOutcome::InternalFailure(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
        crate::checker::inference::InferenceOutcome::Solved(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
    }
}

fn terminal_generic_return_with_fallback(
    outcome: &crate::checker::inference::InferenceOutcome,
    complete_pre_context: Option<TypeKnowledge>,
    fixed_return: Option<TypeKnowledge>,
) -> TypeKnowledge {
    if matches!(outcome, crate::checker::inference::InferenceOutcome::Conflicting(_)) {
        return terminal_generic_return(outcome, fixed_return);
    }
    complete_pre_context.unwrap_or_else(|| terminal_generic_return(outcome, fixed_return))
}

fn inference_support(knowledge: &TypeKnowledge) -> Option<InferenceSupport> {
    match knowledge.status() {
        Some(EvidenceStatus::Established) => Some(InferenceSupport::Established),
        Some(EvidenceStatus::Assumed) => Some(InferenceSupport::Assumed),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ApplicationArgument, ArgumentBindingPlan, ArgumentShapeFailure, StaticCallShape, application_arguments, bind_static_arguments, static_call_shape,
        terminal_generic_return,
    };
    use crate::checker::inference::{InferenceConflict, InferenceFailureReason, InferenceOutcome, InferenceTerm, UnderconstrainedInference};
    use crate::dispatch::CallableParameter;
    use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
    use crate::types::id::TypeId;
    use crate::types::outcome::{BlockReason, BudgetKind, BudgetReport};
    use phalcom_ast::ast::{Expr, RestMode, Statement};
    use phalcom_ast::parse_source;
    use phalcom_common::range::SourceRange;

    fn terminal_outcomes() -> [(InferenceOutcome, UnknownReason); 5] {
        [
            (
                InferenceOutcome::Underconstrained(UnderconstrainedInference { unsolved_vars: Vec::new() }),
                UnknownReason::UnderconstrainedTypeVariable,
            ),
            (
                InferenceOutcome::Conflicting(InferenceConflict {
                    constraint_indices: Box::from([2]),
                    constraint_index: Some(2),
                    origin: None,
                    failure: InferenceFailureReason::StructuralMismatch {
                        left: Box::new(InferenceTerm::Canonical(TypeId(1))),
                        right: Box::new(InferenceTerm::Canonical(TypeId(2))),
                    },
                }),
                UnknownReason::InferenceConflict,
            ),
            (InferenceOutcome::Blocked(BlockReason::RecursiveFixpoint), UnknownReason::InferenceBlocked),
            (InferenceOutcome::Cancelled, UnknownReason::InferenceCancelled),
            (
                InferenceOutcome::BudgetExceeded(BudgetReport::new(BudgetKind::Steps, 0, 1)),
                UnknownReason::InferenceBudgetExceeded,
            ),
        ]
    }

    #[test]
    fn every_generic_terminal_outcome_keeps_its_reason_without_fixed_return() {
        for (outcome, reason) in terminal_outcomes() {
            assert_eq!(terminal_generic_return(&outcome, None), TypeKnowledge::Unknown(reason));
        }
    }

    #[test]
    fn every_generic_terminal_outcome_preserves_only_independent_fixed_return() {
        let fixed = TypeKnowledge::established(TypeId(99), EvidenceOrigin::CallableSignature);
        for (outcome, _) in terminal_outcomes() {
            assert_eq!(terminal_generic_return(&outcome, Some(fixed.clone())), fixed);
        }
    }

    #[test]
    fn static_shape_preserves_positional_and_labeled_slots() {
        let parsed = parse_source("call(1, named: 2)", 0).expect("valid call");
        let Expr::UnqualifiedCall(call) = (match &parsed.statements[0] {
            Statement::Expr { expr, .. } => expr,
            _ => panic!("expected expression"),
        }) else {
            panic!("expected unqualified call");
        };
        let arguments = application_arguments(&call.args);
        assert_eq!(
            static_call_shape(&arguments),
            StaticCallShape::Exact(vec![
                phalcom_common::selector::SelectorSlot::Positional,
                phalcom_common::selector::SelectorSlot::Label("named".into()),
            ])
        );
    }

    #[test]
    fn static_shape_rejects_expansion_and_dynamic_label() {
        let expr = Expr::Ellipsis { range: SourceRange::default() };
        let expansion = [ApplicationArgument::Expansion {
            expression: &expr,
            mode: phalcom_ast::ast::ExpansionMode::Labeled,
            range: SourceRange::default(),
        }];
        assert_eq!(
            static_call_shape(&expansion),
            StaticCallShape::Dynamic(crate::types::evidence::DynamicReason::DynamicRestPack)
        );

        let dynamic_label = [ApplicationArgument::DynamicLabel {
            expression: &expr,
            range: SourceRange::default(),
        }];
        assert_eq!(
            static_call_shape(&dynamic_label),
            StaticCallShape::Dynamic(crate::types::evidence::DynamicReason::DynamicRestPack)
        );
    }

    #[test]
    fn static_binding_reports_shape_failures_and_exact_bindings() {
        let expr = Expr::Ellipsis { range: SourceRange::default() };
        let range = SourceRange::default();
        let arguments = vec![
            ApplicationArgument::Positional { expression: &expr, range },
            ApplicationArgument::Labeled {
                label: "named",
                expression: &expr,
                range,
            },
        ];
        let parameters = vec![
            CallableParameter::new("first", TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)),
            CallableParameter::new("second", TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)).with_label("named"),
        ];
        let plan = bind_static_arguments(&arguments, &parameters).expect("arguments bind");
        assert_eq!(
            plan.bindings,
            vec![
                super::ArgumentBinding {
                    argument_index: 0,
                    parameter_index: 0,
                },
                super::ArgumentBinding {
                    argument_index: 1,
                    parameter_index: 1,
                },
            ]
        );

        let missing = bind_static_arguments(&arguments[..1], &parameters).expect_err("missing parameter");
        assert!(missing.contains(&ArgumentShapeFailure::MissingRequiredParameter { parameter_index: 1 }));

        let extra_args = vec![arguments[0], arguments[0], arguments[1]];
        let extra = bind_static_arguments(&extra_args, &parameters).expect_err("duplicate/extra argument");
        assert!(extra.iter().any(|failure| matches!(failure, ArgumentShapeFailure::UnexpectedPositional { .. })));

        let unknown_label = [ApplicationArgument::Labeled {
            label: "other",
            expression: &expr,
            range,
        }];
        let unknown = bind_static_arguments(&unknown_label, &parameters).expect_err("unknown label");
        assert!(unknown.contains(&ArgumentShapeFailure::UnknownLabel {
            argument_index: 0,
            label: "other".into(),
        }));

        let duplicate = [arguments[1], arguments[1]];
        let duplicate = bind_static_arguments(&duplicate, &parameters).expect_err("duplicate label");
        assert!(duplicate.contains(&ArgumentShapeFailure::DuplicateParameterBinding { parameter_index: 1 }));

        let rest_pos = [CallableParameter::new("rest", TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)).with_rest(RestMode::Positional)];
        assert_eq!(bind_static_arguments(&[], &rest_pos), Ok(ArgumentBindingPlan { bindings: vec![] }));

        let positional_args = [
            ApplicationArgument::Positional { expression: &expr, range },
            ApplicationArgument::Positional { expression: &expr, range },
        ];
        let plan = bind_static_arguments(&positional_args, &rest_pos).expect("positional rest binds multiple arguments");
        assert_eq!(plan.bindings.len(), 2);
        assert_eq!(plan.bindings[0].parameter_index, 0);
        assert_eq!(plan.bindings[1].parameter_index, 0);

        let rest_labeled = [CallableParameter::new("rest", TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)).with_rest(RestMode::Labeled)];
        let labeled_args = [
            ApplicationArgument::Labeled {
                label: "foo",
                expression: &expr,
                range,
            },
            ApplicationArgument::Labeled {
                label: "bar",
                expression: &expr,
                range,
            },
        ];
        let plan_lab = bind_static_arguments(&labeled_args, &rest_labeled).expect("labeled rest binds multiple labeled arguments");
        assert_eq!(plan_lab.bindings.len(), 2);
        assert_eq!(plan_lab.bindings[0].parameter_index, 0);
        assert_eq!(plan_lab.bindings[1].parameter_index, 0);

        let rest_complete = [CallableParameter::new("rest", TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)).with_rest(RestMode::Complete)];
        let mixed_args = [
            ApplicationArgument::Positional { expression: &expr, range },
            ApplicationArgument::Labeled {
                label: "extra",
                expression: &expr,
                range,
            },
        ];
        let plan_comp = bind_static_arguments(&mixed_args, &rest_complete).expect("complete rest binds both positional and labeled arguments");
        assert_eq!(plan_comp.bindings.len(), 2);
        assert_eq!(plan_comp.bindings[0].parameter_index, 0);
        assert_eq!(plan_comp.bindings[1].parameter_index, 0);
    }
}

#[cfg(test)]
mod parameter_use_tests {
    use super::{CallableApplicationTarget, effective_declaration_generics, type_contains_any_parameter};
    use crate::checker::context::CheckingContext;
    use crate::declarations::DeclarationTypeTable;
    use crate::dispatch::{CallableParameter, CallableSignature};
    use crate::identity::{AssociatedStaticDomainDeclarationId, CallableId, DeclarationId, DispatchSide, ModuleId};
    use crate::types::evidence::{EvidenceOrigin, TypeKnowledge};
    use crate::types::id::{KindId, TypeParameterId};
    use crate::types::parameter::{TypeParameterData, TypeParameterOwner};
    use crate::types::row::RecordRowTail;
    use crate::types::store::{StaticArgument, TypeData, TypeStore};

    fn owner(name: &str) -> DeclarationId {
        DeclarationId::new(ModuleId::universe_root(), name.into())
    }

    fn row_parameter(store: &mut TypeStore) -> TypeParameterId {
        store.intern_type_parameter(TypeParameterData::new(
            TypeParameterOwner::Declaration(owner("RowCarrier")),
            0,
            "R",
            KindId::RECORD_ROW,
        ))
    }

    fn open_row(store: &mut TypeStore, parameter: TypeParameterId) -> crate::types::id::RecordRowId {
        store
            .record_row_checked(Vec::new(), RecordRowTail::Parameter(parameter))
            .expect("valid open row")
    }

    #[test]
    fn parameter_use_walk_reaches_a_record_rows_parameter_only_in_its_tail() {
        let mut store = TypeStore::new();
        let parameter = row_parameter(&mut store);
        let row = open_row(&mut store, parameter);
        let ty = store.record_type(row);

        assert!(type_contains_any_parameter(&store, ty, &[parameter]));
    }

    #[test]
    fn row_tail_only_declaration_parameter_keeps_call_inference_domain_active() {
        let mut store = TypeStore::new();
        let owner = owner("RowCarrier");
        let parameter_owner = TypeParameterOwner::Declaration(owner.clone());
        let parameter = store.intern_type_parameter(TypeParameterData::new(parameter_owner.clone(), 0, "R", KindId::RECORD_ROW));
        let row_type = store
            .record_row_type_checked(Vec::new(), RecordRowTail::Parameter(parameter))
            .expect("open row type");
        let selector = phalcom_common::selector::Selector::getter("value").expect("getter selector");
        let signature = CallableSignature::new(
            selector.clone(),
            vec![CallableParameter::new(
                "value",
                TypeKnowledge::established(row_type, EvidenceOrigin::CallableSignature),
            )],
            TypeKnowledge::established(row_type, EvidenceOrigin::CallableSignature),
        );
        let target = CallableApplicationTarget::exact(CallableId::new(owner.clone(), selector, DispatchSide::Instance), signature)
            .with_declaration_generics(crate::types::parameter::GenericSignature::new(parameter_owner, Box::new([parameter])));
        let hierarchy = crate::types::relation::MapTypeHierarchy::new();
        let resolver = crate::types::annotation::SimpleTypeResolver::new();
        let declarations = DeclarationTypeTable::new();
        let ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, ModuleId::universe_root());

        assert!(effective_declaration_generics(&ctx, &target).is_some());
    }

    #[test]
    fn parameter_use_walk_reaches_record_row_arguments_in_applied_and_static_data() {
        let mut store = TypeStore::new();
        let parameter = row_parameter(&mut store);
        let row = open_row(&mut store, parameter);
        let origin = store.nominal_type(owner("Applied"));
        let applied = store.intern_with_kind(
            TypeData::Applied {
                origin,
                arguments: vec![StaticArgument::RecordRow(row)].into_boxed_slice(),
            },
            KindId::TYPE,
        );
        assert!(type_contains_any_parameter(&store, applied, &[parameter]));

        let domain = store.nominal_static_domain(owner("Shape"));
        let constructor = store.intern_static_constructor(domain, owner("Shape"), "Row", vec![KindId::RECORD_ROW].into_boxed_slice());
        let static_data = store
            .static_data(domain, constructor, vec![StaticArgument::RecordRow(row)].into_boxed_slice())
            .expect("valid row-valued static argument");
        assert!(type_contains_any_parameter(&store, static_data, &[parameter]));
    }

    #[test]
    fn parameter_use_walk_reaches_associated_static_domain_owner_arguments() {
        let mut store = TypeStore::new();
        let parameter = row_parameter(&mut store);
        let row = open_row(&mut store, parameter);
        let owner = owner("Container");
        let declaration = AssociatedStaticDomainDeclarationId::new(owner.clone(), "State");
        let domain = store.associated_static_domain(declaration, vec![StaticArgument::RecordRow(row)].into_boxed_slice());
        let domain_form = store.associated_static_domain_form(domain);
        assert!(type_contains_any_parameter(&store, domain_form, &[parameter]));

        let constructor = store.intern_static_constructor(domain, owner, "Ready", Box::new([]));
        let static_data = store.static_data(domain, constructor, Box::new([])).expect("valid associated case");
        assert!(type_contains_any_parameter(&store, static_data, &[parameter]));
    }
}
