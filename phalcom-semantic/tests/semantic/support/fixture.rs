#![allow(dead_code)]

use phalcom_ast::parse;
use phalcom_common::range::SourceRange;
use phalcom_common::selector::SelectorBase;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::analysis::SemanticDependency;
use phalcom_semantic::checker::analysis::{AnalysisStatus, BindingState, CallableAnalysis, ExpressionAnalysis};
use phalcom_semantic::checker::{BindingConsistency, BindingContractOrigin};
use phalcom_semantic::diagnostic::{DiagnosticCode, DiagnosticSeverity, SemanticDiagnostic};
use phalcom_semantic::explain::DerivationRule;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, SourceSiteId};
use phalcom_semantic::types::evidence::{DynamicReason, EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use phalcom_semantic::types::row::RecordRowTail;
use phalcom_semantic::types::store::TypeData;
use phalcom_semantic::{TypeId, analyze_single_module, is_subtype};
use std::collections::HashSet;
use std::sync::Arc;

pub type Analysis = phalcom_semantic::workspace::SemanticAnalysis;

/// Stable source location selector used by deep assertions.
pub enum SourceLocator<'a> {
    Text { text: &'a str, occurrence: usize },
    Offset(usize),
    Range(SourceRange),
    Site(SourceSiteId),
}

#[derive(Clone, Debug)]
pub enum TypeExpectation {
    Id(TypeId),
    Any,
    Nominal(String),
    ClassObject(String),
    Applied(Box<TypeExpectation>, Vec<TypeExpectation>),
    Union(Vec<TypeExpectation>),
    Tuple(Vec<TypeExpectation>),
    Record(Vec<(String, TypeExpectation)>),
}

impl From<TypeId> for TypeExpectation {
    fn from(value: TypeId) -> Self {
        Self::Id(value)
    }
}

pub fn nominal(name: impl Into<String>) -> TypeExpectation {
    TypeExpectation::Nominal(name.into())
}

pub fn class_object(name: impl Into<String>) -> TypeExpectation {
    TypeExpectation::ClassObject(name.into())
}

pub fn applied(origin: impl Into<String>, arguments: impl IntoIterator<Item = TypeExpectation>) -> TypeExpectation {
    TypeExpectation::Applied(Box::new(nominal(origin)), arguments.into_iter().collect())
}

pub fn union(members: impl IntoIterator<Item = TypeExpectation>) -> TypeExpectation {
    TypeExpectation::Union(members.into_iter().collect())
}

pub fn tuple(elements: impl IntoIterator<Item = TypeExpectation>) -> TypeExpectation {
    TypeExpectation::Tuple(elements.into_iter().collect())
}

#[derive(Clone, Debug)]
pub struct KnowledgeExpectation {
    pub ty: Option<TypeExpectation>,
    pub state: KnowledgeStateExpectation,
    pub status: Option<EvidenceStatus>,
    pub origin: Option<EvidenceOrigin>,
}

#[derive(Clone, Debug)]
pub enum KnowledgeStateExpectation {
    Known,
    Unknown(Option<UnknownReason>),
    Dynamic(Option<DynamicReason>),
}

pub fn known<T: Into<TypeExpectation>>(ty: T) -> KnowledgeExpectation {
    KnowledgeExpectation {
        ty: Some(ty.into()),
        state: KnowledgeStateExpectation::Known,
        status: None,
        origin: None,
    }
}

pub fn unknown(reason: UnknownReason) -> KnowledgeExpectation {
    KnowledgeExpectation {
        ty: None,
        state: KnowledgeStateExpectation::Unknown(Some(reason)),
        status: None,
        origin: None,
    }
}

pub fn dynamic(reason: DynamicReason) -> KnowledgeExpectation {
    KnowledgeExpectation {
        ty: None,
        state: KnowledgeStateExpectation::Dynamic(Some(reason)),
        status: None,
        origin: None,
    }
}

impl KnowledgeExpectation {
    pub fn established(mut self) -> Self {
        self.status = Some(EvidenceStatus::Established);
        self
    }

    pub fn assumed(mut self) -> Self {
        self.status = Some(EvidenceStatus::Assumed);
        self
    }

    pub fn origin(mut self, origin: EvidenceOrigin) -> Self {
        self.origin = Some(origin);
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct BindingExpectation {
    pub declared: Option<TypeExpectation>,
    pub current: Option<KnowledgeExpectation>,
    pub consistency: Option<ConsistencyExpectation>,
    pub mutable: Option<bool>,
}

#[derive(Clone, Debug)]
pub enum ConsistencyExpectation {
    Validated,
    Assumed,
    Refuted { actual: TypeExpectation, expected: TypeExpectation },
    Unconstrained,
}

pub fn binding() -> BindingExpectation {
    BindingExpectation::default()
}

impl BindingExpectation {
    pub fn declared<T: Into<TypeExpectation>>(mut self, ty: T) -> Self {
        self.declared = Some(ty.into());
        self
    }

    pub fn current(mut self, knowledge: KnowledgeExpectation) -> Self {
        self.current = Some(knowledge);
        self
    }

    pub fn validated(mut self) -> Self {
        self.consistency = Some(ConsistencyExpectation::Validated);
        self
    }

    pub fn assumed(mut self) -> Self {
        self.consistency = Some(ConsistencyExpectation::Assumed);
        self
    }

    pub fn refuted<T: Into<TypeExpectation>, U: Into<TypeExpectation>>(mut self, actual: T, expected: U) -> Self {
        self.consistency = Some(ConsistencyExpectation::Refuted {
            actual: actual.into(),
            expected: expected.into(),
        });
        self
    }

    pub fn mutable(mut self, mutable: bool) -> Self {
        self.mutable = Some(mutable);
        self
    }
}

pub struct Fixture {
    pub module: ModuleId,
    pub source: Arc<str>,
    pub analysis: Analysis,
}

impl Fixture {
    pub fn new(source_text: &str) -> Self {
        let fixture = Self::new_allowing_internal_incidents(source_text);
        fixture.assert_no_internal_incidents();
        fixture
    }

    pub fn new_allowing_internal_incidents(source_text: &str) -> Self {
        let module = ModuleId::core();
        let source: Arc<str> = Arc::from(source_text);
        let parsed = parse(&source, 0);
        assert!(parsed.errors.is_empty(), "parse errors: {:#?}\nsource:\n{source_text}", parsed.errors);
        let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
        Self { module, source, analysis }
    }

    pub fn decl(&self, name: &str) -> DeclarationId {
        DeclarationId::new(self.module.clone(), name.into())
    }

    pub fn ty(&self, name: &str) -> TypeId {
        self.analysis
            .snapshot
            .declarations
            .form(&self.decl(name))
            .unwrap_or_else(|| panic!("missing type form for `{name}`"))
    }

    pub fn callable_id(&self, owner: &str, name: &str, side: DispatchSide) -> CallableId {
        let owner = self.decl(owner);
        let mut matches = self
            .analysis
            .snapshot
            .callable_analyses
            .keys()
            .filter(|id| id.owner == owner && id.side == side && matches!(&id.selector.base, SelectorBase::Named(base) if base == name))
            .cloned()
            .collect::<Vec<_>>();
        matches.sort_by(|a, b| format!("{:?}", a.selector).cmp(&format!("{:?}", b.selector)));
        assert_eq!(matches.len(), 1, "expected one callable {owner:?}.{name} on {side:?}, got: {matches:#?}");
        matches.remove(0)
    }

    pub fn callable(&self, owner: &str, name: &str, side: DispatchSide) -> &CallableAnalysis {
        let id = self.callable_id(owner, name, side);
        self.analysis
            .snapshot
            .callable_analyses
            .get(&id)
            .unwrap_or_else(|| panic!("missing callable analysis for {id:?}"))
    }

    pub fn binding<'a>(&self, callable: &'a CallableAnalysis, name: &str) -> &'a BindingState {
        let matches = callable.bindings.values().filter(|binding| binding.name == name).collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "expected one binding `{name}`, got {matches:#?}");
        matches[0]
    }

    pub fn bindings_named<'a>(&self, callable: &'a CallableAnalysis, name: &str) -> Vec<&'a BindingState> {
        let mut matches = callable.bindings.values().filter(|binding| binding.name == name).collect::<Vec<_>>();
        matches.sort_by_key(|binding| binding.range.start);
        matches
    }

    pub fn expression<'a>(&'a self, callable: &'a CallableAnalysis, expected_text: &str) -> &'a ExpressionAnalysis {
        self.expression_n(callable, expected_text, 0)
    }

    pub fn expression_n<'a>(&'a self, callable: &'a CallableAnalysis, expected_text: &str, n: usize) -> &'a ExpressionAnalysis {
        let mut matches = callable
            .expressions
            .values()
            .filter(|expr| self.source.get(expr.range.start..expr.range.end) == Some(expected_text))
            .collect::<Vec<_>>();
        matches.sort_by_key(|expr| expr.range.start);
        matches.get(n).copied().unwrap_or_else(|| {
            panic!(
                "expected occurrence {n} of expression `{expected_text}` in {:?}; found {}: {matches:#?}",
                callable.callable,
                matches.len()
            )
        })
    }

    pub fn diagnostics(&self, code: DiagnosticCode) -> Vec<&SemanticDiagnostic> {
        self.analysis.snapshot.all_diagnostics().filter(|diagnostic| diagnostic.code == code).collect()
    }

    pub fn assert_no_diagnostic(&self, code: DiagnosticCode) {
        let found = self.diagnostics(code);
        assert!(found.is_empty(), "unexpected {code:?} diagnostics: {found:#?}");
    }

    pub fn assert_subtype(&self, sub: TypeId, sup: TypeId) {
        assert!(
            is_subtype(&self.analysis.snapshot.store, self.analysis.snapshot.hierarchy.as_ref(), sub, sup),
            "expected {sub:?} <: {sup:?}"
        );
    }

    pub fn assert_binding_type(&self, callable: &CallableAnalysis, name: &str, expected: TypeId) {
        let binding = self.binding(callable, name);
        assert_eq!(binding.current.ty(), Some(expected), "binding `{name}`: {binding:#?}");
    }

    pub fn assert_binding_established(&self, callable: &CallableAnalysis, name: &str, expected: TypeId) {
        let binding = self.binding(callable, name);
        assert_eq!(binding.current.ty(), Some(expected), "binding `{name}`: {binding:#?}");
        assert_eq!(
            binding.current.status(),
            Some(EvidenceStatus::Established),
            "binding `{name}` should be established: {binding:#?}"
        );
    }

    pub fn assert_expression_established(&self, expression: &ExpressionAnalysis, expected: TypeId) {
        assert_eq!(expression.knowledge.ty(), Some(expected), "expression: {expression:#?}");
        assert_eq!(
            expression.knowledge.status(),
            Some(EvidenceStatus::Established),
            "expression should be established: {expression:#?}"
        );
    }

    pub fn assert_union_members(&self, actual: TypeId, expected: &[TypeId]) {
        let actual_members: HashSet<TypeId> = match self.analysis.snapshot.store.get(actual) {
            TypeData::Union(members) => members.iter().copied().collect(),
            _ => [actual].into_iter().collect(),
        };
        let expected_members: HashSet<TypeId> = expected.iter().copied().collect();
        assert_eq!(actual_members, expected_members, "unexpected union for {actual:?}");
    }

    pub fn assert_tuple_types(&self, actual: TypeId, expected: &[TypeId]) {
        match self.analysis.snapshot.store.get(actual) {
            TypeData::Tuple(elements) => {
                let actual = elements.iter().map(|element| element.ty).collect::<Vec<_>>();
                assert_eq!(actual, expected, "unexpected tuple type");
            }
            other => panic!("expected tuple type, got {other:?}"),
        }
    }

    pub fn assert_record(&self, actual: TypeId) {
        assert!(matches!(self.analysis.snapshot.store.get(actual), TypeData::Record(_)), "expected record type");
    }

    pub fn assert_no_error_diagnostics(&self) {
        let errors = self
            .analysis
            .snapshot
            .all_diagnostics()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            .collect::<Vec<_>>();
        assert!(errors.is_empty(), "unexpected semantic errors: {errors:#?}");
    }

    pub fn assert_no_internal_incidents(&self) {
        assert!(
            self.analysis.snapshot.internal_incidents.is_empty(),
            "semantic analyzer produced internal incidents: {:#?}",
            self.analysis.snapshot.internal_incidents
        );
    }

    pub fn assert_diagnostic(&self, code: DiagnosticCode, count: usize) {
        assert_eq!(self.diagnostics(code).len(), count, "unexpected {code} diagnostics");
    }

    pub fn assert_only_error_codes(&self, expected: &[DiagnosticCode]) {
        let mut actual = self
            .analysis
            .snapshot
            .all_diagnostics()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>();
        actual.sort_by_key(|code| code.as_str());
        let mut expected = expected.to_vec();
        expected.sort_by_key(|code| code.as_str());
        assert_eq!(actual, expected);
    }

    pub fn assert_type(&self, actual: TypeId, expected: impl Into<TypeExpectation>) {
        self.assert_type_expectation(actual, &expected.into());
    }

    pub fn assert_knowledge(&self, actual: &TypeKnowledge, expected: &KnowledgeExpectation) {
        match (&expected.state, actual) {
            (KnowledgeStateExpectation::Known, TypeKnowledge::Known(_)) => {}
            (KnowledgeStateExpectation::Unknown(reason), TypeKnowledge::Unknown(actual_reason)) => {
                if let Some(reason) = reason {
                    assert_eq!(actual_reason, reason);
                }
            }
            (KnowledgeStateExpectation::Dynamic(reason), TypeKnowledge::Dynamic(actual_reason)) => {
                if let Some(reason) = reason {
                    assert_eq!(actual_reason, reason);
                }
            }
            (state, actual) => panic!("knowledge state mismatch: expected {state:?}, actual {actual:?}"),
        }
        if let Some(ty) = &expected.ty {
            let actual_ty = actual.ty().expect("expected known type");
            self.assert_type_expectation(actual_ty, ty);
        }
        if let Some(status) = expected.status {
            assert_eq!(actual.status(), Some(status), "knowledge status mismatch: {actual:#?}");
        }
        if let Some(origin) = expected.origin {
            assert_eq!(actual.origin(), Some(origin), "knowledge origin mismatch: {actual:#?}");
        }
    }

    pub fn assert_binding_expectation(&self, callable: &CallableAnalysis, name: &str, expected: BindingExpectation) {
        let actual = self.binding(callable, name);
        if let Some(declared) = expected.declared {
            let actual_declared = actual.declared_type().expect("expected declared binding type");
            self.assert_type_expectation(actual_declared, &declared);
        }
        if let Some(current) = expected.current {
            self.assert_knowledge(&actual.current, &current);
        }
        if let Some(mutable) = expected.mutable {
            assert_eq!(actual.mutable, mutable, "binding mutability mismatch: {actual:#?}");
        }
        if let Some(consistency) = expected.consistency {
            match consistency {
                ConsistencyExpectation::Validated => assert!(matches!(actual.consistency, BindingConsistency::Validated), "{actual:#?}"),
                ConsistencyExpectation::Assumed => assert!(matches!(actual.consistency, BindingConsistency::Assumed { .. }), "{actual:#?}"),
                ConsistencyExpectation::Unconstrained => assert!(matches!(actual.consistency, BindingConsistency::Unconstrained), "{actual:#?}"),
                ConsistencyExpectation::Refuted {
                    actual: want_actual,
                    expected: want_expected,
                } => {
                    let BindingConsistency::Refuted {
                        actual: actual_ty,
                        expected: expected_ty,
                        ..
                    } = actual.consistency
                    else {
                        panic!("expected refuted binding: {actual:#?}");
                    };
                    self.assert_type_expectation(actual_ty, &want_actual);
                    self.assert_type_expectation(expected_ty, &want_expected);
                }
            }
        }
    }

    pub fn assert_expression_knowledge(&self, expression: &ExpressionAnalysis, expected: KnowledgeExpectation) {
        self.assert_knowledge(&expression.knowledge, &expected);
    }

    pub fn assert_expression_ready(&self, expression: &ExpressionAnalysis) {
        assert!(matches!(expression.status, AnalysisStatus::Ready), "expected ready expression: {expression:#?}");
    }

    pub fn assert_expression_call_target(&self, expression: &ExpressionAnalysis, expected: &CallableId) {
        assert_eq!(expression.callable.as_ref(), Some(expected), "unexpected resolved callable: {expression:#?}");
    }

    pub fn assert_explanation_rule(&self, callable: &CallableAnalysis, expression: &ExpressionAnalysis, expected: DerivationRule) {
        let id = expression.explanation.expect("expected expression explanation");
        let node = callable.explanations.get(id).expect("missing explanation node");
        assert_eq!(node.rule, expected, "unexpected explanation node: {node:#?}");
        assert_eq!(node.status, expression.knowledge.status().expect("explained expression must be known"));
        assert_eq!(node.origin, expression.knowledge.origin().expect("explained expression must be known"));
    }

    pub fn assert_callable_dependency(&self, callable: &CallableAnalysis, expected: &CallableId) {
        assert!(
            callable.dependencies.iter().any(|dependency| dependency == expected),
            "missing callable dependency {expected:?}: {callable:#?}"
        );
    }

    pub fn assert_semantic_dependency(&self, callable: &CallableAnalysis, expected: &SemanticDependency) {
        assert!(
            callable.semantic_dependencies.iter().any(|dependency| dependency == expected),
            "missing semantic dependency {expected:?}: {callable:#?}"
        );
    }

    pub fn assert_normal_return(&self, callable: &CallableAnalysis, expected: KnowledgeExpectation) {
        assert_eq!(callable.exits.normal_return_values.len(), 1, "expected one published normal return");
        self.assert_knowledge(&callable.exits.normal_return_values[0], &expected);
    }

    pub fn expression_at<'a>(&'a self, callable: &'a CallableAnalysis, locator: SourceLocator<'_>) -> &'a ExpressionAnalysis {
        match locator {
            SourceLocator::Text { text, occurrence } => self.expression_n(callable, text, occurrence),
            SourceLocator::Offset(offset) => callable
                .expressions
                .values()
                .find(|expression| expression.range.contains(offset))
                .expect("no expression at offset"),
            SourceLocator::Range(range) => callable
                .expressions
                .values()
                .find(|expression| expression.range == range)
                .expect("no expression at range"),
            SourceLocator::Site(site) => self.analysis.snapshot.formal_expression_at(&site).expect("site has no formal expression"),
        }
    }

    fn assert_type_expectation(&self, actual: TypeId, expected: &TypeExpectation) {
        match expected {
            TypeExpectation::Any => {}
            TypeExpectation::Id(expected) => assert_eq!(actual, *expected, "unexpected type: {}", self.analysis.snapshot.store.format_type(actual)),
            TypeExpectation::Nominal(name) => match self.analysis.snapshot.store.get(actual) {
                TypeData::Nominal { declaration } if declaration.name.as_ref() == name => {}
                other => panic!("expected nominal `{name}`, got {other:?}"),
            },
            TypeExpectation::ClassObject(name) => match self.analysis.snapshot.store.get(actual) {
                TypeData::ClassObject { declaration } if declaration.name.as_ref() == name => {}
                other => panic!("expected class object `{name}`, got {other:?}"),
            },
            TypeExpectation::Applied(origin, arguments) => {
                let TypeData::Applied {
                    origin: actual_origin,
                    arguments: actual_arguments,
                } = self.analysis.snapshot.store.get(actual)
                else {
                    panic!("expected applied type, got {:?}", self.analysis.snapshot.store.get(actual));
                };
                self.assert_type_expectation(*actual_origin, origin);
                assert_eq!(actual_arguments.len(), arguments.len());
                for (actual, expected) in actual_arguments.iter().zip(arguments) {
                    self.assert_type_expectation(*actual, expected);
                }
            }
            TypeExpectation::Union(expected_members) => {
                let actual_members = match self.analysis.snapshot.store.get(actual) {
                    TypeData::Union(members) => members.to_vec(),
                    _ => vec![actual],
                };
                assert_eq!(actual_members.len(), expected_members.len());
                let mut used = vec![false; actual_members.len()];
                for expected in expected_members {
                    let Some((index, _)) = actual_members
                        .iter()
                        .enumerate()
                        .find(|(index, actual)| !used[*index] && self.type_matches(**actual, expected))
                    else {
                        panic!("missing union member {expected:?}");
                    };
                    used[index] = true;
                }
            }
            TypeExpectation::Tuple(expected_elements) => {
                let TypeData::Tuple(actual_elements) = self.analysis.snapshot.store.get(actual) else {
                    panic!("expected tuple, got {:?}", self.analysis.snapshot.store.get(actual));
                };
                assert_eq!(actual_elements.len(), expected_elements.len());
                for (actual, expected) in actual_elements.iter().zip(expected_elements) {
                    self.assert_type_expectation(actual.ty, expected);
                }
            }
            TypeExpectation::Record(expected_fields) => {
                let TypeData::Record(row) = self.analysis.snapshot.store.get(actual) else {
                    panic!("expected record, got {:?}", self.analysis.snapshot.store.get(actual));
                };
                let row = self.analysis.snapshot.store.record_row(*row);
                assert_eq!(row.tail, RecordRowTail::Closed);
                assert_eq!(row.fields.len(), expected_fields.len());
                for (name, expected) in expected_fields {
                    let field = row.fields.iter().find(|field| field.name.as_ref() == name).expect("missing record field");
                    self.assert_type_expectation(field.ty, expected);
                }
            }
        }
    }

    fn type_matches(&self, actual: TypeId, expected: &TypeExpectation) -> bool {
        match expected {
            TypeExpectation::Id(expected) => actual == *expected,
            TypeExpectation::Any => true,
            TypeExpectation::Nominal(name) => {
                matches!(self.analysis.snapshot.store.get(actual), TypeData::Nominal { declaration } if declaration.name.as_ref() == name)
            }
            TypeExpectation::ClassObject(name) => {
                matches!(self.analysis.snapshot.store.get(actual), TypeData::ClassObject { declaration } if declaration.name.as_ref() == name)
            }
            TypeExpectation::Applied(expected_origin, expected_arguments) => {
                let TypeData::Applied { origin, arguments } = self.analysis.snapshot.store.get(actual) else {
                    return false;
                };
                arguments.len() == expected_arguments.len()
                    && self.type_matches(*origin, expected_origin)
                    && arguments
                        .iter()
                        .zip(expected_arguments)
                        .all(|(actual, expected)| self.type_matches(*actual, expected))
            }
            TypeExpectation::Union(expected_members) => {
                let actual_members = match self.analysis.snapshot.store.get(actual) {
                    TypeData::Union(members) => members.to_vec(),
                    _ => vec![actual],
                };
                if actual_members.len() != expected_members.len() {
                    return false;
                }
                let mut used = vec![false; actual_members.len()];
                for expected in expected_members {
                    let Some((index, _)) = actual_members
                        .iter()
                        .enumerate()
                        .find(|(index, actual)| !used[*index] && self.type_matches(**actual, expected))
                    else {
                        return false;
                    };
                    used[index] = true;
                }
                true
            }
            TypeExpectation::Tuple(expected_elements) => {
                let TypeData::Tuple(actual_elements) = self.analysis.snapshot.store.get(actual) else {
                    return false;
                };
                actual_elements.len() == expected_elements.len()
                    && actual_elements
                        .iter()
                        .zip(expected_elements)
                        .all(|(actual, expected)| self.type_matches(actual.ty, expected))
            }
            TypeExpectation::Record(expected_fields) => {
                let TypeData::Record(row) = self.analysis.snapshot.store.get(actual) else {
                    return false;
                };
                let row = self.analysis.snapshot.store.record_row(*row);
                row.tail == RecordRowTail::Closed
                    && row.fields.len() == expected_fields.len()
                    && expected_fields.iter().all(|(name, expected)| {
                        row.fields
                            .iter()
                            .find(|field| field.name.as_ref() == name)
                            .is_some_and(|field| self.type_matches(field.ty, expected))
                    })
            }
        }
    }
}

pub fn assert_validated(binding: &BindingState) {
    assert!(
        matches!(binding.consistency, BindingConsistency::Validated),
        "expected validated binding contract: {binding:#?}"
    );
}

pub fn assert_refuted(binding: &BindingState, actual: TypeId, expected: TypeId) {
    assert!(
        matches!(binding.consistency, BindingConsistency::Refuted { actual: a, expected: e, .. } if a == actual && e == expected),
        "expected refuted binding contract {actual:?} !<: {expected:?}: {binding:#?}"
    );
}

pub fn assert_source_contract(binding: &BindingState, expected: TypeId) {
    assert_eq!(binding.declared_type(), Some(expected), "unexpected declared type: {binding:#?}");
    let contract = binding.contract.as_ref().expect("expected source contract");
    assert_eq!(contract.ty, expected);
    assert_eq!(contract.origin, BindingContractOrigin::SourceAnnotation);
}
