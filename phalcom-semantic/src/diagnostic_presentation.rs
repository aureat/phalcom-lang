//! Protocol-neutral rich diagnostic presentation over compiler-owned semantic evidence.
//!
//! This module is intentionally a projection layer. It does not perform type
//! inference, assignability, dispatch, or any other semantic reasoning.

use crate::diagnostic::{DiagnosticCode, DiagnosticFix, DiagnosticGuidance, DiagnosticSeverity, ExplanationRef, SemanticDiagnostic, SemanticSourceSpan};
use crate::explain::{DerivationRule, ExplanationNode, ExplanationStep, causal_trace};
use crate::presentation::{AdvisoryPresenter, TypePresenter};
use crate::snapshot::SemanticSnapshot;
use crate::types::{EvidenceOrigin, EvidenceStatus, RelationOutcome, TypeKnowledge};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticDetail {
    Compact,
    Explain,
    Trace,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentedDiagnostic {
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub headline: String,
    pub primary: SemanticSourceSpan,
    pub labels: Vec<PresentedLabel>,
    pub explanation: Vec<PresentedLine>,
    pub guidance: Vec<PresentedLine>,
    pub context: Vec<PresentedLine>,
    pub trace: Vec<PresentedTraceNode>,
    pub fixes: Vec<DiagnosticFix>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentedLabel {
    pub span: SemanticSourceSpan,
    pub message: String,
    pub role: PresentedLabelRole,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentedLabelRole {
    Primary,
    Required,
    Established,
    Supporting,
}

impl PresentedLabelRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Required => "required",
            Self::Established => "established",
            Self::Supporting => "supporting",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentedLine {
    pub text: String,
}

impl PresentedLine {
    fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentedTraceNode {
    pub reference: ExplanationRef,
    pub rule: DerivationRule,
    pub status: EvidenceStatus,
    pub origin: EvidenceOrigin,
    pub text: String,
}

pub struct DiagnosticPresenter<'a> {
    snapshot: &'a SemanticSnapshot,
    types: TypePresenter<'a>,
}

impl<'a> DiagnosticPresenter<'a> {
    pub fn new(snapshot: &'a SemanticSnapshot) -> Self {
        Self {
            snapshot,
            types: TypePresenter::new(snapshot.store().as_ref()),
        }
    }

    pub fn present(&self, diagnostic: &SemanticDiagnostic, detail: DiagnosticDetail) -> PresentedDiagnostic {
        let trace_nodes = self.trace_nodes(diagnostic);
        let headline = self.headline(diagnostic, &trace_nodes);
        let labels = self.labels(diagnostic, &trace_nodes);
        let explanation = if matches!(detail, DiagnosticDetail::Compact) {
            self.compact_explanation(diagnostic, &trace_nodes)
        } else {
            self.explanation_lines(diagnostic, &trace_nodes)
        };
        let guidance = self.guidance_lines(diagnostic);
        let context = self.advisory_context(diagnostic, &trace_nodes);
        let trace = if matches!(detail, DiagnosticDetail::Trace) {
            trace_nodes
                .iter()
                .map(|(reference, node)| PresentedTraceNode {
                    reference: reference.clone(),
                    rule: node.rule.clone(),
                    status: node.status,
                    origin: node.origin,
                    text: self.node_text(node),
                })
                .collect()
        } else {
            Vec::new()
        };

        PresentedDiagnostic {
            code: diagnostic.code,
            severity: diagnostic.severity,
            headline,
            primary: diagnostic.primary.clone(),
            labels,
            explanation,
            guidance,
            context,
            trace,
            fixes: diagnostic.fixes.clone(),
        }
    }

    fn trace_nodes<'b>(&'b self, diagnostic: &'b SemanticDiagnostic) -> Vec<(ExplanationRef, &'b ExplanationNode)> {
        let mut result = Vec::new();
        for root in &diagnostic.explanations {
            let Some(arena) = self.snapshot.explanation_arena(&root.callable) else {
                continue;
            };
            for node in causal_trace(arena, root.explanation) {
                let reference = ExplanationRef::new(root.callable.clone(), node.id);
                if !result.iter().any(|(existing, _)| existing == &reference) {
                    result.push((reference, node));
                }
            }
        }
        result
    }

    fn headline(&self, diagnostic: &SemanticDiagnostic, trace: &[(ExplanationRef, &ExplanationNode)]) -> String {
        let terminal = trace.iter().rev().find_map(|(_, node)| match &node.step {
            ExplanationStep::TypeRelation { outcome, .. } => Some(outcome),
            _ => None,
        });
        if terminal.is_some_and(|outcome| !matches!(outcome, RelationOutcome::Refuted(_))) {
            return match terminal.expect("checked above") {
                RelationOutcome::Blocked(_) => "cannot establish the required type".into(),
                RelationOutcome::DynamicBoundary(_) => "formal analysis stops at a dynamic boundary".into(),
                RelationOutcome::Cancelled => "type analysis was cancelled".into(),
                RelationOutcome::BudgetExceeded(_) => "type analysis exceeded its budget".into(),
                RelationOutcome::InternalFailure(_) => "type analysis failed internally".into(),
                RelationOutcome::Proven { .. } | RelationOutcome::Refuted(_) => diagnostic.message.clone(),
            };
        }

        match diagnostic.code {
            DiagnosticCode::BindingInitializerMismatch => "initializer conflicts with declared type".into(),
            DiagnosticCode::AssignmentMismatch => "assignment conflicts with binding type".into(),
            DiagnosticCode::ReturnMismatch => "return value conflicts with declared return type".into(),
            DiagnosticCode::ExistentialEscape => "branch-local existential type escapes its scope".into(),
            DiagnosticCode::ArgumentMismatch => "argument conflicts with parameter type".into(),
            DiagnosticCode::FieldMismatch => "assigned value conflicts with field type".into(),
            DiagnosticCode::CallShapeMismatch => "call does not match the callable shape".into(),
            DiagnosticCode::GenericInferenceConflict => "generic constraints conflict".into(),
            DiagnosticCode::GenericInferenceUnderconstrained => "generic parameter is underconstrained".into(),
            DiagnosticCode::GenericInferenceAmbiguous => "generic inference has multiple admissible solutions".into(),
            DiagnosticCode::GenericConstraintUnsatisfied => "generic constraint is not satisfied".into(),
            DiagnosticCode::RecordDuplicateField => "record contains duplicate field".into(),
            DiagnosticCode::RecordRowTailUnresolved => "record row tail cannot be resolved".into(),
            DiagnosticCode::RecordRowTailKindMismatch => "record row tail has the wrong kind".into(),
            DiagnosticCode::RecordRowLacksViolation => "record row contains a forbidden field".into(),
            DiagnosticCode::RecordRowOccursCheck => "record row is recursive".into(),
            DiagnosticCode::RecordRowInferenceUnderconstrained => "record row is underconstrained".into(),
            DiagnosticCode::RecordRowInferenceConflict => "record row constraints conflict".into(),
            DiagnosticCode::RecordRowLacksUnproven => "record extension requires a disjoint row field".into(),
            DiagnosticCode::KindExpectedType => "type constructor used where a proper type is required".into(),
            DiagnosticCode::ApplicationTooManyArguments => "too many type arguments".into(),
            _ => diagnostic.message.clone(),
        }
    }

    fn labels(&self, diagnostic: &SemanticDiagnostic, trace: &[(ExplanationRef, &ExplanationNode)]) -> Vec<PresentedLabel> {
        let mut labels = Vec::new();
        let relation = trace.iter().rev().find_map(|(_, node)| match &node.step {
            ExplanationStep::TypeRelation {
                actual,
                expected,
                outcome: RelationOutcome::Refuted(_),
            } => Some((actual, *expected)),
            _ => None,
        });
        if let Some((actual, expected)) = relation {
            labels.push(PresentedLabel {
                span: diagnostic.primary.clone(),
                message: format!("required `{}`", self.types.present_type(expected)),
                role: PresentedLabelRole::Required,
            });
            if let Some(actual) = actual.ty() {
                labels.push(PresentedLabel {
                    span: diagnostic.primary.clone(),
                    message: format!("{} `{}`", self.knowledge_verb(actual_status(actual, trace)), self.types.present_type(actual)),
                    role: PresentedLabelRole::Established,
                });
            }
        }
        for label in &diagnostic.labels {
            labels.push(PresentedLabel {
                span: label.span.clone(),
                message: label.message.clone(),
                role: PresentedLabelRole::Supporting,
            });
        }
        if labels.is_empty() {
            labels.push(PresentedLabel {
                span: diagnostic.primary.clone(),
                message: diagnostic.message.clone(),
                role: PresentedLabelRole::Primary,
            });
        }
        labels
    }

    fn compact_explanation(&self, diagnostic: &SemanticDiagnostic, trace: &[(ExplanationRef, &ExplanationNode)]) -> Vec<PresentedLine> {
        let mut lines = self.explanation_lines(diagnostic, trace);
        lines.truncate(2);
        lines
    }

    fn explanation_lines(&self, diagnostic: &SemanticDiagnostic, trace: &[(ExplanationRef, &ExplanationNode)]) -> Vec<PresentedLine> {
        let mut lines = Vec::new();
        for (_, node) in trace {
            if !self.is_user_explanation_node(node) {
                continue;
            }
            let text = self.node_text(node);
            if !text.is_empty() && !lines.iter().any(|line: &PresentedLine| line.text == text) {
                lines.push(PresentedLine::new(text));
            }
        }
        if lines.is_empty() {
            lines.extend(diagnostic.notes.iter().cloned().map(PresentedLine::new));
        }
        lines
    }

    fn is_user_explanation_node(&self, node: &ExplanationNode) -> bool {
        matches!(
            node.step,
            ExplanationStep::BindingRead { .. }
                | ExplanationStep::Declared { .. }
                | ExplanationStep::BindingContract { .. }
                | ExplanationStep::TypeRequirement { .. }
                | ExplanationStep::TypeRelation { .. }
                | ExplanationStep::CallableSelection { .. }
                | ExplanationStep::CallableKind { .. }
                | ExplanationStep::CallableReturn { .. }
                | ExplanationStep::SelfTypeSpecialization { .. }
                | ExplanationStep::ArgumentCheck { .. }
                | ExplanationStep::CallShape { .. }
                | ExplanationStep::GenericConstraint { .. }
                | ExplanationStep::GenericSolution { .. }
                | ExplanationStep::GenericConflict { .. }
                | ExplanationStep::ProductComponent { .. }
                | ExplanationStep::FlowRefinement { .. }
                | ExplanationStep::BranchJoin { .. }
                | ExplanationStep::LoopJoin { .. }
                | ExplanationStep::IterationElement { .. }
                | ExplanationStep::ReturnCheck { .. }
                | ExplanationStep::CallableReturnSummary { .. }
                | ExplanationStep::UnknownBoundary { .. }
                | ExplanationStep::DynamicBoundary { .. }
                | ExplanationStep::InternalBlocked { .. }
        )
    }

    fn guidance_lines(&self, diagnostic: &SemanticDiagnostic) -> Vec<PresentedLine> {
        let mut lines = diagnostic
            .guidance
            .iter()
            .map(|guidance| PresentedLine::new(self.guidance_text(guidance)))
            .collect::<Vec<_>>();
        lines.extend(diagnostic.helps.iter().cloned().map(PresentedLine::new));
        lines
    }

    fn guidance_text(&self, guidance: &DiagnosticGuidance) -> String {
        match guidance {
            DiagnosticGuidance::ChangeAnnotation { ty, .. } => format!("the annotation can be changed to `{}`", self.types.present_type(*ty)),
            DiagnosticGuidance::SupplyAssignableValue { expected } => {
                format!("supply a value assignable to `{}`", self.types.present_type(*expected))
            }
            DiagnosticGuidance::UseCallableShape { callable } => {
                if let Some(signature) = self.snapshot.callable_signatures().get(callable) {
                    format!("this call accepts `{}`", signature.selector)
                } else {
                    format!("use the declared callable shape `{}`", callable.selector)
                }
            }
            DiagnosticGuidance::EstablishTypeEvidence { expected, .. } => match expected {
                Some(expected) => format!("establish formal type evidence compatible with `{}`", self.types.present_type(*expected)),
                None => "establish formal type evidence at this source site".into(),
            },
            DiagnosticGuidance::ResolveGenericParameter { parameter } => {
                format!("provide value or contextual evidence for generic parameter `T{}`", parameter.index())
            }
        }
    }

    fn advisory_context(&self, diagnostic: &SemanticDiagnostic, trace: &[(ExplanationRef, &ExplanationNode)]) -> Vec<PresentedLine> {
        let formal_unavailable = trace.iter().any(|(_, node)| {
            matches!(
                node.step,
                ExplanationStep::UnknownBoundary { .. } | ExplanationStep::DynamicBoundary { .. } | ExplanationStep::InternalBlocked { .. }
            ) || matches!(
                &node.step,
                ExplanationStep::TypeRelation {
                    outcome: RelationOutcome::Blocked(_)
                        | RelationOutcome::DynamicBoundary(_)
                        | RelationOutcome::Cancelled
                        | RelationOutcome::BudgetExceeded(_)
                        | RelationOutcome::InternalFailure(_),
                    ..
                }
            )
        });
        if !formal_unavailable {
            return Vec::new();
        }
        let view = self.snapshot.semantic_site_at(&diagnostic.primary.module, diagnostic.primary.range.start);
        let Some(fact) = view.advisory else {
            return Vec::new();
        };
        if matches!(fact.shape, crate::advisory::ValueShape::Unknown) {
            return Vec::new();
        }
        vec![PresentedLine::new(format!(
            "tooling currently observes this value as `{}`",
            AdvisoryPresenter::present_shape(&fact.shape)
        ))]
    }

    fn knowledge_verb(&self, status: EvidenceStatus) -> &'static str {
        match status {
            EvidenceStatus::Established => "proven",
            EvidenceStatus::Assumed => "assumed",
        }
    }

    fn knowledge_text(&self, knowledge: &TypeKnowledge) -> String {
        match knowledge {
            TypeKnowledge::Known(evidence) => format!("`{}`", self.types.present_type(evidence.ty())),
            TypeKnowledge::Unknown(_) => "an unknown type".into(),
            TypeKnowledge::Dynamic(_) => "a dynamic value".into(),
        }
    }

    fn node_text(&self, node: &ExplanationNode) -> String {
        match &node.step {
            ExplanationStep::Literal { ty, .. } => format!("literal has type `{}`", self.types.present_type(*ty)),
            ExplanationStep::ExpressionResult { knowledge, .. } => format!("expression produces {}", self.knowledge_text(knowledge)),
            ExplanationStep::BindingRead { knowledge, .. } => format!("binding read produces {}", self.knowledge_text(knowledge)),
            ExplanationStep::Declared { ty, .. } => format!("declaration requires `{}`", self.types.present_type(*ty)),
            ExplanationStep::BindingContract { actual, contract, .. } => format!(
                "binding requires `{}` and the value produces {}",
                self.types.present_type(*contract),
                self.knowledge_text(actual)
            ),
            ExplanationStep::TypeRequirement { expected, origin, .. } => {
                format!("this context requires `{}` ({origin:?})", self.types.present_type(*expected))
            }
            ExplanationStep::TypeRelation { actual, expected, outcome } => {
                let actual = self.knowledge_text(actual);
                let expected = self.types.present_type(*expected);
                match outcome {
                    RelationOutcome::Proven { .. } => format!("{actual} satisfies required `{expected}`"),
                    RelationOutcome::Refuted(_) => format!("{actual} is not assignable to `{expected}`"),
                    RelationOutcome::Blocked(_) => format!("cannot establish whether {actual} satisfies `{expected}`"),
                    RelationOutcome::DynamicBoundary(_) => format!("formal analysis stops before proving {actual} against `{expected}`"),
                    RelationOutcome::Cancelled => "the relation check was cancelled".into(),
                    RelationOutcome::BudgetExceeded(_) => "the relation check exceeded its analysis budget".into(),
                    RelationOutcome::InternalFailure(_) => "the relation check failed internally".into(),
                }
            }
            ExplanationStep::CallableSelection { callable, .. } => format!("selected callable `{}`", callable.selector),
            ExplanationStep::UnionArm { receiver, outcome, .. } => match outcome {
                crate::explain::UnionArmOutcome::Resolved => format!("union receiver arm `{}` resolved selector", self.types.present_type(*receiver)),
                crate::explain::UnionArmOutcome::Missing { .. } => {
                    format!("union receiver arm `{}` has no matching selector", self.types.present_type(*receiver))
                }
                crate::explain::UnionArmOutcome::Ambiguous => format!("union receiver arm `{}` has an ambiguous selector", self.types.present_type(*receiver)),
                crate::explain::UnionArmOutcome::Dynamic { .. } => {
                    format!("union receiver arm `{}` crosses a dynamic boundary", self.types.present_type(*receiver))
                }
                crate::explain::UnionArmOutcome::Invalid => format!("union receiver arm `{}` has an invalid application", self.types.present_type(*receiver)),
                crate::explain::UnionArmOutcome::ContextConflict => {
                    format!(
                        "union receiver arm `{}` conflicts with contextual closure expectations",
                        self.types.present_type(*receiver)
                    )
                }
            },
            ExplanationStep::CallableKind { kind, .. } => match kind {
                crate::dispatch::CallableSemanticKind::Constructor => "the selected callable is an @constructor".into(),
                crate::dispatch::CallableSemanticKind::Ordinary => "the selected callable is an ordinary method".into(),
                crate::dispatch::CallableSemanticKind::Native => "the selected callable uses a native signature".into(),
            },
            ExplanationStep::CallableReturn { ty, .. } => format!("its declared result is `{}`", self.types.present_type(*ty)),
            ExplanationStep::SelfTypeSpecialization { self_ty, receiver, resolved } => format!(
                "`{}` specialized for receiver `{}` resolves to `{}`",
                self.types.present_type(*self_ty),
                self.types.present_type(*receiver),
                self.types.present_type(*resolved)
            ),
            ExplanationStep::ArgumentCheck {
                actual,
                expected,
                parameter_index,
                ..
            } => format!(
                "argument {} produces {}; parameter requires `{}`",
                *parameter_index as usize + 1,
                self.knowledge_text(actual),
                self.types.present_type(*expected)
            ),
            ExplanationStep::CallShape { failures, .. } => failures
                .iter()
                .map(|failure| match failure {
                    crate::explain::CallShapeExplanation::MissingRequired { label, parameter_index } => label
                        .as_ref()
                        .map(|label| format!("missing required argument `{label}:`"))
                        .unwrap_or_else(|| format!("missing required argument {}", *parameter_index as usize + 1)),
                    crate::explain::CallShapeExplanation::UnexpectedPositional { .. } => "unexpected positional argument".into(),
                    crate::explain::CallShapeExplanation::UnknownLabel { label } => format!("unknown argument label `{label}:`"),
                    crate::explain::CallShapeExplanation::DuplicateParameter { parameter_index } => {
                        format!("parameter {} is supplied twice", *parameter_index as usize + 1)
                    }
                })
                .collect::<Vec<_>>()
                .join("; "),
            ExplanationStep::MethodCall { callable, return_ty, .. } => {
                format!("`{}` produces `{}`", callable.selector, self.types.present_type(*return_ty))
            }
            ExplanationStep::UnresolvedCall { return_ty, .. } => format!(
                "call result is `{}` but callable provenance is unavailable",
                self.types.present_type(*return_ty)
            ),
            ExplanationStep::GenericConstraint { parameter, relation, origin } => {
                format!("generic parameter `T{}` receives {relation:?} evidence from {origin:?}", parameter.index())
            }
            ExplanationStep::GenericSolution { parameter, ty, status } => format!(
                "generic parameter `T{}` resolves to `{}` ({status:?})",
                parameter.index(),
                self.types.present_type(*ty)
            ),
            ExplanationStep::GenericConflict { parameter, .. } => parameter
                .map(|parameter| format!("constraints for generic parameter `T{}` have no common solution", parameter.index()))
                .unwrap_or_else(|| "generic constraints have no common solution".into()),
            ExplanationStep::CollectionSynthesis { result, .. } => format!("collection synthesizes `{}`", self.types.present_type(*result)),
            ExplanationStep::ProductComponent { index, result, .. } => format!("component {} produces {}", index + 1, self.knowledge_text(result)),
            ExplanationStep::FlowRefinement { predicate, prior, refined, .. } => {
                format!("{predicate:?} refines {} to {}", self.knowledge_text(prior), self.knowledge_text(refined))
            }
            ExplanationStep::BranchJoin {
                branches, reachable, joined, ..
            } => {
                let abrupt = reachable.iter().filter(|reachable| !**reachable).count();
                if abrupt > 0 {
                    format!(
                        "reachable branch values join to {}; {abrupt} abrupt branch does not reach this join",
                        self.knowledge_text(joined)
                    )
                } else {
                    format!("{} branch values join to {}", branches.len(), self.knowledge_text(joined))
                }
            }
            ExplanationStep::LoopJoin { joined, .. } => format!("loop flow joins to {}", self.knowledge_text(joined)),
            ExplanationStep::IterationElement { iterable, element, .. } => {
                format!("iterating {} produces element {}", self.knowledge_text(iterable), self.knowledge_text(element))
            }
            ExplanationStep::ReturnCheck { actual, expected } => match expected {
                Some(expected) => format!(
                    "return produces {}; callable requires `{}`",
                    self.knowledge_text(actual),
                    self.types.present_type(*expected)
                ),
                None => format!("return produces {}", self.knowledge_text(actual)),
            },
            ExplanationStep::CallableReturnSummary { returns, result, .. } => {
                format!("{} normal return path(s) publish {}", returns.len(), self.knowledge_text(result))
            }
            ExplanationStep::UnknownBoundary { reason, .. } => format!("formal type evidence becomes unavailable here ({reason:?})"),
            ExplanationStep::DynamicBoundary { reason, .. } => format!("formal analysis stops at this dynamic boundary ({reason:?})"),
            ExplanationStep::InternalBlocked { reason } => format!("analysis is blocked here ({reason:?})"),
            ExplanationStep::Subtyping { actual, expected, proven } => {
                let relation = if *proven { "satisfies" } else { "does not satisfy" };
                format!("`{}` {relation} `{}`", self.types.present_type(*actual), self.types.present_type(*expected))
            }
        }
    }
}

fn actual_status(_actual: crate::types::TypeId, trace: &[(ExplanationRef, &ExplanationNode)]) -> EvidenceStatus {
    trace
        .iter()
        .rev()
        .find_map(|(_, node)| match &node.step {
            ExplanationStep::TypeRelation {
                actual: TypeKnowledge::Known(evidence),
                ..
            } => Some(evidence.status()),
            _ => None,
        })
        .unwrap_or(EvidenceStatus::Established)
}
