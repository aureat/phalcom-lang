use phalcom_ast::parse;
use phalcom_common::selector::SelectorBase;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::analysis::{BindingState, CallableAnalysis, ExpressionAnalysis};
use phalcom_semantic::checker::{BindingConsistency, BindingContractOrigin};
use phalcom_semantic::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::types::evidence::EvidenceStatus;
use phalcom_semantic::types::store::TypeData;
use phalcom_semantic::{TypeId, analyze_single_module, is_subtype};
use std::collections::HashSet;
use std::sync::Arc;

pub type Analysis = phalcom_semantic::workspace::SemanticAnalysis;

pub struct Fixture {
    pub module: ModuleId,
    pub source: Arc<str>,
    pub analysis: Analysis,
}

impl Fixture {
    pub fn new(source_text: &str) -> Self {
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
            .filter(|id| {
                id.owner == owner
                    && id.side == side
                    && matches!(&id.selector.base, SelectorBase::Named(base) if base == name)
            })
            .cloned()
            .collect::<Vec<_>>();
        matches.sort_by(|a, b| format!("{:?}", a.selector).cmp(&format!("{:?}", b.selector)));
        assert_eq!(
            matches.len(),
            1,
            "expected one callable {owner:?}.{name} on {side:?}, got: {matches:#?}"
        );
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
        self.analysis
            .snapshot
            .all_diagnostics()
            .filter(|diagnostic| diagnostic.code == code)
            .collect()
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
    assert_eq!(binding.declared, Some(expected), "unexpected declared type: {binding:#?}");
    let contract = binding.contract.as_ref().expect("expected source contract");
    assert_eq!(contract.ty, expected);
    assert_eq!(contract.origin, BindingContractOrigin::SourceAnnotation);
}
