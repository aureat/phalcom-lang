#![allow(dead_code)]

use phalcom_ast::parse;
use phalcom_common::selector::SelectorBase;
use phalcom_core::error::PhError;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::analysis::{AnalysisStatus, BindingState, CallableAnalysis, ExpressionAnalysis};
use phalcom_semantic::diagnostic::{DiagnosticCode, DiagnosticSeverity, SemanticDiagnostic};
use phalcom_semantic::explain::{ExplanationStep, GenericConstraintOrigin};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::types::evidence::{EvidenceStatus, TypeKnowledge, UnknownReason};
use phalcom_semantic::types::id::{TypeId, TypeParameterId};
use phalcom_semantic::types::store::TypeData;
use phalcom_semantic::{analyze_single_module, causal_trace};
use std::sync::Arc;

const EITHER_SOURCE: &str = include_str!("either.ph");
const SEMANTIC_PROBES: &str = include_str!("semantic_probes.ph");
const RUNTIME_PROBES: &str = include_str!("runtime_probes.ph");

pub fn semantic_source() -> String {
    format!("{EITHER_SOURCE}\n{SEMANTIC_PROBES}")
}

pub fn runtime_source() -> String {
    format!("{EITHER_SOURCE}\n{RUNTIME_PROBES}")
}

pub fn with_either(extra: &str) -> String {
    format!("{EITHER_SOURCE}\n{extra}")
}

pub struct Fixture {
    pub module: ModuleId,
    pub source: Arc<str>,
    pub analysis: phalcom_semantic::workspace::SemanticAnalysis,
}

impl Fixture {
    pub fn new(source_text: &str) -> Self {
        let module = ModuleId::universe_root();
        let source: Arc<str> = Arc::from(source_text);
        let parsed = parse(&source, 0);
        assert!(parsed.errors.is_empty(), "parse errors: {:#?}\nsource:\n{source_text}", parsed.errors);
        let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
        assert!(
            analysis.snapshot.internal_incidents.is_empty(),
            "semantic analyzer produced internal incidents: {:#?}",
            analysis.snapshot.internal_incidents
        );
        Self { module, source, analysis }
    }

    pub fn decl(&self, name: &str) -> DeclarationId {
        DeclarationId::new(self.module.clone(), name.into())
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

    pub fn expression_containing<'a>(&'a self, callable: &'a CallableAnalysis, needle: &str) -> &'a ExpressionAnalysis {
        let mut matches = callable
            .expressions
            .values()
            .filter(|expr| self.source.get(expr.range.start..expr.range.end).is_some_and(|text| text.contains(needle)))
            .collect::<Vec<_>>();
        matches.sort_by_key(|expr| expr.range.end - expr.range.start);
        let shortest = matches.first().copied().unwrap_or_else(|| panic!("no expression containing `{needle}`"));
        if matches.len() > 1 {
            let shortest_len = shortest.range.end - shortest.range.start;
            let same_len = matches.iter().filter(|expr| expr.range.end - expr.range.start == shortest_len).count();
            assert_eq!(same_len, 1, "ambiguous shortest expression containing `{needle}`: {matches:#?}");
        }
        shortest
    }

    pub fn diagnostics(&self, code: DiagnosticCode) -> Vec<&SemanticDiagnostic> {
        self.analysis.snapshot.all_diagnostics().filter(|diagnostic| diagnostic.code == code).collect()
    }

    pub fn assert_no_errors(&self) {
        let errors = self
            .analysis
            .snapshot
            .all_diagnostics()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            .collect::<Vec<_>>();
        assert!(errors.is_empty(), "unexpected semantic errors: {errors:#?}");
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
        assert_eq!(actual, expected, "unexpected diagnostics");
    }

    pub fn assert_binding_applied(&self, callable: &CallableAnalysis, name: &str, origin: &str, args: &[Ty<'_>]) -> TypeId {
        let binding = self.binding(callable, name);
        let ty = binding.current.ty().unwrap_or_else(|| panic!("binding `{name}` has no known type: {binding:#?}"));
        self.assert_type(ty, &Ty::Applied(origin, args.to_vec()));
        self.family_type(ty)
    }

    pub fn assert_binding_nominal(&self, callable: &CallableAnalysis, name: &str, expected: &str) -> TypeId {
        let binding = self.binding(callable, name);
        let ty = binding.current.ty().unwrap_or_else(|| panic!("binding `{name}` has no known type: {binding:#?}"));
        self.assert_type(ty, &Ty::Nominal(expected));
        ty
    }

    pub fn assert_known_generic_binding(&self, callable: &CallableAnalysis, name: &str, expected: &Ty<'_>) {
        let binding = self.binding(callable, name);
        let ty = binding.current.ty().unwrap_or_else(|| panic!("binding `{name}` has no known type: {binding:#?}"));
        self.assert_type(ty, expected);
        assert!(matches!(binding.current, TypeKnowledge::Known(_)), "binding `{name}` must be statically known: {binding:#?}");
    }

    pub fn assert_ready(&self, expression: &ExpressionAnalysis) {
        assert!(matches!(expression.status, AnalysisStatus::Ready), "expression is not ready: {expression:#?}");
    }

    pub fn family_type(&self, ty: TypeId) -> TypeId {
        match self.analysis.snapshot.store.get(ty) {
            TypeData::ExactCase { enum_type, .. } => *enum_type,
            _ => ty,
        }
    }

    pub fn assert_type(&self, actual: TypeId, expected: &Ty<'_>) {
        let actual = self.family_type(actual);
        match expected {
            Ty::Nominal(name) => match self.analysis.snapshot.store.get(actual) {
                TypeData::Nominal { declaration } if declaration.name.as_ref() == *name => {}
                other => panic!("expected nominal `{name}`, got {other:?} ({})", self.analysis.snapshot.store.format_type(actual)),
            },
            Ty::Applied(name, args) => {
                let TypeData::Applied { origin, arguments } = self.analysis.snapshot.store.get(actual) else {
                    panic!("expected applied `{name}`, got {:?} ({})", self.analysis.snapshot.store.get(actual), self.analysis.snapshot.store.format_type(actual));
                };
                self.assert_type(*origin, &Ty::Nominal(name));
                assert_eq!(arguments.len(), args.len(), "wrong generic arity for `{name}`");
                for (actual, expected) in arguments.iter().zip(args.iter()) {
                    self.assert_type(*actual, expected);
                }
            }
            Ty::Tuple(elements) => {
                let TypeData::Tuple(actual) = self.analysis.snapshot.store.get(actual) else {
                    panic!("expected tuple, got {:?}", self.analysis.snapshot.store.get(actual));
                };
                assert_eq!(actual.len(), elements.len());
                for (actual, expected) in actual.iter().zip(elements.iter()) {
                    self.assert_type(actual.ty, expected);
                }
            }
        }
    }

    pub fn generic_trace<'a>(&'a self, callable: &'a CallableAnalysis, expression: &ExpressionAnalysis) -> Vec<&'a phalcom_semantic::ExplanationNode> {
        let id = expression.explanation.expect("expression must have an explanation");
        causal_trace(&callable.explanations, id)
    }

    pub fn assert_generic_constraint_origin(
        &self,
        callable: &CallableAnalysis,
        expression: &ExpressionAnalysis,
        parameter_name: &str,
        expected_origin: GenericConstraintOrigin,
    ) {
        let trace = self.generic_trace(callable, expression);
        assert!(
            trace.iter().any(|node| {
                matches!(
                    &node.step,
                    ExplanationStep::GenericConstraint { parameter, origin, .. }
                        if self.parameter_name(*parameter) == parameter_name && *origin == expected_origin
                )
            }),
            "missing generic constraint `{parameter_name}` from {expected_origin:?}: {trace:#?}"
        );
    }

    pub fn assert_generic_solution(
        &self,
        callable: &CallableAnalysis,
        expression: &ExpressionAnalysis,
        parameter_name: &str,
        expected: &Ty<'_>,
    ) {
        let trace = self.generic_trace(callable, expression);
        let solution = trace.iter().find_map(|node| match &node.step {
            ExplanationStep::GenericSolution { parameter, ty, status } if self.parameter_name(*parameter) == parameter_name => Some((*ty, *status)),
            _ => None,
        });
        let (ty, status) = solution.unwrap_or_else(|| panic!("missing generic solution for `{parameter_name}`: {trace:#?}"));
        self.assert_type(ty, expected);
        assert!(matches!(status, EvidenceStatus::Established | EvidenceStatus::Assumed));
    }

    pub fn assert_receiver_selection(&self, callable: &CallableAnalysis, expression: &ExpressionAnalysis, expected: &Ty<'_>) {
        let trace = self.generic_trace(callable, expression);
        let receiver = trace.iter().find_map(|node| match &node.step {
            ExplanationStep::CallableSelection { receiver, .. } => Some(*receiver),
            _ => None,
        });
        let receiver = receiver.unwrap_or_else(|| panic!("missing callable-selection receiver: {trace:#?}"));
        self.assert_type(receiver, expected);
    }

    fn parameter_name(&self, parameter: TypeParameterId) -> &str {
        self.analysis.snapshot.store.type_parameter(parameter).name.as_ref()
    }

    pub fn assert_unknown_underconstrained(&self, callable: &CallableAnalysis, name: &str) {
        let binding = self.binding(callable, name);
        assert!(
            matches!(binding.current, TypeKnowledge::Unknown(UnknownReason::UnderconstrainedTypeVariable)),
            "expected underconstrained unknown binding: {binding:#?}"
        );
    }
}

#[derive(Clone, Debug)]
pub enum Ty<'a> {
    Nominal(&'a str),
    Applied(&'a str, Vec<Ty<'a>>),
    Tuple(Vec<Ty<'a>>),
}

pub fn nominal(name: &str) -> Ty<'_> {
    Ty::Nominal(name)
}

pub fn either<'a>(left: Ty<'a>, right: Ty<'a>) -> Ty<'a> {
    Ty::Applied("Either", vec![left, right])
}

pub fn tuple<'a>(elements: impl IntoIterator<Item = Ty<'a>>) -> Ty<'a> {
    Ty::Tuple(elements.into_iter().collect())
}

pub fn run_inline(source: &str) -> Result<(VM, phalcom_core::heap::ObjRef), PhError> {
    let mut vm = VM::new();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).map_err(PhError::from)?;
    vm.run_compiled(&program)?;
    let entry_id = program.initialization_order.last().expect("entry module");
    let module = vm.module_registry.get(entry_id).expect("entry module registered").object;
    Ok((vm, module))
}

pub fn slot(vm: &VM, module: phalcom_core::heap::ObjRef, name: &str) -> Option<Value> {
    let symbol = vm.interner.find(name)?;
    vm.heap.module(module).get(symbol)
}
