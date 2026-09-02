#![allow(dead_code)]

use phalcom_ast::parse;
use phalcom_common::selector::SelectorBase;
use phalcom_core::error::PhError;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::analysis::{AnalysisStatus, BindingState, CallableAnalysis, ExpressionAnalysis};
use phalcom_semantic::declarations::DeclarationTypeInfo;
use phalcom_semantic::diagnostic::{DiagnosticCode, DiagnosticSeverity, SemanticDiagnostic};
use phalcom_semantic::explain::{ExplanationStep, GenericConstraintOrigin, GenericConstraintRelation};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::types::evidence::EvidenceStatus;
use phalcom_semantic::types::id::{KindId, TypeId, TypeParameterId};
use phalcom_semantic::types::kind::KindData;
use phalcom_semantic::types::outcome::BudgetReport;
use phalcom_semantic::types::parameter::TypeParameterOwner;
use phalcom_semantic::types::specialization::{ReceiverSpecialization, SpecializationControl, specialize_receiver_to_owner};
use phalcom_semantic::types::store::{TypeData, TypeStore};
use phalcom_semantic::{analyze_single_module, causal_trace};
use std::sync::Arc;

const MONADS_SOURCE: &str = include_str!("monads.ph");
const SEMANTIC_PROBES: &str = include_str!("semantic_probes.ph");
const RUNTIME_PROBES: &str = include_str!("runtime_probes.ph");

pub fn monads_source() -> &'static str {
    MONADS_SOURCE
}

pub fn semantic_source() -> String {
    format!("{MONADS_SOURCE}\n{SEMANTIC_PROBES}")
}

pub fn runtime_source() -> String {
    format!("{MONADS_SOURCE}\n{RUNTIME_PROBES}")
}

pub fn with_monads(extra: &str) -> String {
    format!("{MONADS_SOURCE}\n{extra}")
}

#[derive(Default)]
pub struct UnlimitedSpecialization;

impl SpecializationControl for UnlimitedSpecialization {
    fn charge_step(&self) -> Result<(), BudgetReport> {
        Ok(())
    }

    fn is_cancelled(&self) -> bool {
        false
    }
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

    pub fn info(&self, name: &str) -> &DeclarationTypeInfo {
        self.analysis
            .snapshot
            .declarations
            .get(&self.decl(name))
            .unwrap_or_else(|| panic!("missing declaration type info for `{name}`"))
    }

    pub fn ty(&self, name: &str) -> TypeId {
        self.analysis
            .snapshot
            .declarations
            .form(&self.decl(name))
            .unwrap_or_else(|| panic!("missing canonical type form for `{name}`"))
    }

    pub fn type_parameter(&self, owner: &str, index: u32) -> TypeParameterId {
        self.analysis
            .snapshot
            .store
            .find_type_parameter_id(&TypeParameterOwner::Declaration(self.decl(owner)), index)
            .unwrap_or_else(|| panic!("missing parameter {owner}[{index}]"))
    }

    pub fn callable_generic_parameter(&self, owner: &str, name: &str, side: DispatchSide, index: usize) -> TypeParameterId {
        let callable = self.callable_id(owner, name, side);
        let signature = self
            .analysis
            .snapshot
            .callable_signatures
            .get(&callable)
            .unwrap_or_else(|| panic!("missing canonical callable signature for {callable:?}"));
        signature
            .generics
            .as_ref()
            .and_then(|generics| generics.parameter_at(index))
            .unwrap_or_else(|| panic!("missing generic parameter {index} for {callable:?}"))
    }

    pub fn type_parameter_form(&self, owner: &str, index: u32) -> TypeId {
        let parameter = self.type_parameter(owner, index);
        let mut cloned = (*self.analysis.snapshot.store).clone();
        let form = cloned.parameter_form(parameter);
        assert!(
            form.index() < self.analysis.snapshot.store.type_count(),
            "test helper would return a TypeId interned only in a cloned store: {form:?}"
        );
        assert!(
            matches!(self.analysis.snapshot.store.get(form), TypeData::Parameter(found) if *found == parameter),
            "canonical parameter form in original store does not match {parameter:?}"
        );
        form
    }

    pub fn unary_kind(&self) -> KindId {
        self.info("Functor")
            .generic_signature
            .as_ref()
            .expect("Functor must be generic")
            .parameters
            .first()
            .map(|parameter| self.analysis.snapshot.store.type_parameter(*parameter).kind)
            .expect("Functor must have F")
    }

    pub fn assert_unary_constructor_kind(&self, kind: KindId) {
        match self.analysis.snapshot.store.get_kind(kind) {
            KindData::Arrow { parameters, result } if parameters.as_ref() == [KindId::TYPE] && *result == KindId::TYPE => {}
            other => panic!(
                "expected unary constructor kind Type -> Type, got {other:?} ({})",
                self.analysis.snapshot.store.format_kind(kind)
            ),
        }
    }

    pub fn parameter_kind(&self, owner: &str, index: u32) -> KindId {
        let parameter = self.type_parameter(owner, index);
        self.analysis.snapshot.store.type_parameter(parameter).kind
    }

    pub fn assert_nominal(&self, ty: TypeId, expected: &str) {
        let expected_decl = self.decl(expected);
        match self.analysis.snapshot.store.get(ty) {
            TypeData::Nominal { declaration } if declaration == &expected_decl => {}
            other => panic!(
                "expected nominal `{expected}` ({expected_decl:?}), got {other:?} ({})",
                self.analysis.snapshot.store.format_type(ty)
            ),
        }
    }

    pub fn assert_applied(&self, ty: TypeId, expected_origin: &str, expected_arity: usize) -> &[TypeId] {
        let TypeData::Applied { origin, arguments } = self.analysis.snapshot.store.get(ty) else {
            panic!(
                "expected applied `{expected_origin}`, got {:?} ({})",
                self.analysis.snapshot.store.get(ty),
                self.analysis.snapshot.store.format_type(ty)
            );
        };
        self.assert_nominal(*origin, expected_origin);
        assert_eq!(arguments.len(), expected_arity, "wrong arity for `{expected_origin}`");
        arguments
    }

    pub fn assert_either(&self, ty: TypeId, left: TypeId, right: TypeId) {
        let arguments = self.assert_applied(ty, "Either", 2);
        assert_eq!(arguments, [left, right], "wrong Either specialization: {}", self.analysis.snapshot.store.format_type(ty));
    }

    pub fn applied_receiver(&self, store: &mut TypeStore, owner: &str, arguments: &[TypeId]) -> TypeId {
        store
            .apply_type_form(self.ty(owner), arguments)
            .unwrap_or_else(|error| panic!("failed to apply `{owner}`: {error:?}"))
    }

    pub fn specialize_receiver(&self, receiver_owner: &str, arguments: &[TypeId], target_owner: &str) -> (TypeStore, ReceiverSpecialization) {
        let mut store = (*self.analysis.snapshot.store).clone();
        let receiver = self.applied_receiver(&mut store, receiver_owner, arguments);
        let specialization = specialize_receiver_to_owner(
            &mut store,
            self.analysis.snapshot.hierarchy.as_ref(),
            receiver,
            &self.decl(target_owner),
            &UnlimitedSpecialization,
        )
        .unwrap_or_else(|error| panic!("failed to specialize {receiver_owner} to {target_owner}: {error:?}"));
        (store, specialization)
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

    pub fn assert_expression_call(
        &self,
        expression: &ExpressionAnalysis,
        expected_callable: &CallableId,
        expected_type: TypeId,
    ) {
        assert!(matches!(expression.status, AnalysisStatus::Ready), "call expression must be ready: {expression:#?}");
        assert_eq!(expression.callable.as_ref(), Some(expected_callable), "unexpected resolved callable: {expression:#?}");
        assert_eq!(expression.knowledge.ty(), Some(expected_type), "unexpected call result type: {expression:#?}");
    }

    pub fn generic_trace<'a>(&'a self, callable: &'a CallableAnalysis, expression: &ExpressionAnalysis) -> Vec<&'a phalcom_semantic::ExplanationNode> {
        let id = expression.explanation.expect("expression must have an explanation");
        causal_trace(&callable.explanations, id)
    }

    fn parameter_name(&self, parameter: TypeParameterId) -> &str {
        self.analysis.snapshot.store.type_parameter(parameter).name.as_ref()
    }

    pub fn generic_solution_type_for(&self, callable: &CallableAnalysis, expression: &ExpressionAnalysis, parameter: TypeParameterId) -> TypeId {
        let trace = self.generic_trace(callable, expression);
        let matches = trace
            .iter()
            .filter_map(|node| match &node.step {
                ExplanationStep::GenericSolution { parameter: found, ty, .. } if *found == parameter => Some(*ty),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "expected one generic solution for {parameter:?}: {trace:#?}");
        matches[0]
    }

    pub fn generic_solution_type(&self, callable: &CallableAnalysis, expression: &ExpressionAnalysis, parameter_name: &str) -> TypeId {
        let trace = self.generic_trace(callable, expression);
        let matches = trace
            .iter()
            .filter_map(|node| match &node.step {
                ExplanationStep::GenericSolution { parameter, ty, .. } if self.parameter_name(*parameter) == parameter_name => Some((*parameter, *ty)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            matches.len(),
            1,
            "name-based generic solution lookup for `{parameter_name}` is ambiguous or missing; use exact TypeParameterId when scopes overlap: {trace:#?}"
        );
        matches[0].1
    }

    pub fn assert_callable_selection(
        &self,
        callable: &CallableAnalysis,
        expression: &ExpressionAnalysis,
        expected_callable: &CallableId,
        expected_receiver: TypeId,
        expected_declaring_owner: &DeclarationId,
        expected_path: &[DeclarationId],
    ) {
        let trace = self.generic_trace(callable, expression);
        let matches = trace
            .iter()
            .filter_map(|node| match &node.step {
                ExplanationStep::CallableSelection {
                    callable,
                    receiver,
                    declaring_owner,
                    specialization_path,
                } if callable == expected_callable => Some((*receiver, declaring_owner, specialization_path)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "expected one callable selection for {expected_callable:?}: {trace:#?}");
        let (receiver, owner, path) = matches[0];
        assert_eq!(receiver, expected_receiver, "wrong receiver in callable selection");
        assert_eq!(owner, expected_declaring_owner, "wrong declaring owner in callable selection");
        assert_eq!(path.as_ref(), expected_path, "wrong receiver specialization path");
    }

    pub fn assert_callable_selection_path(
        &self,
        callable: &CallableAnalysis,
        expression: &ExpressionAnalysis,
        declaring_owner: &str,
        expected_path: &[&str],
    ) {
        let trace = self.generic_trace(callable, expression);
        let expected_owner = self.decl(declaring_owner);
        let expected_path = expected_path.iter().map(|name| self.decl(name)).collect::<Vec<_>>();
        let matches = trace
            .iter()
            .filter_map(|node| match &node.step {
                ExplanationStep::CallableSelection {
                    declaring_owner,
                    specialization_path,
                    ..
                } if declaring_owner == &expected_owner && specialization_path.as_ref() == expected_path.as_slice() => Some(()),
                _ => None,
            })
            .count();
        assert_eq!(matches, 1, "expected one exact owner/path callable selection: {trace:#?}");
    }

    pub fn assert_generic_solution_exact(
        &self,
        callable: &CallableAnalysis,
        expression: &ExpressionAnalysis,
        parameter: TypeParameterId,
        expected: TypeId,
        expected_status: EvidenceStatus,
    ) {
        let trace = self.generic_trace(callable, expression);
        let matches = trace
            .iter()
            .filter_map(|node| match &node.step {
                ExplanationStep::GenericSolution { parameter: found, ty, status } if *found == parameter => Some((*ty, *status)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "expected one generic solution for {parameter:?}: {trace:#?}");
        assert_eq!(matches[0].0, expected, "wrong generic solution for {parameter:?}");
        assert_eq!(matches[0].1, expected_status, "wrong evidence status for {parameter:?}");
    }

    pub fn assert_generic_solution(
        &self,
        callable: &CallableAnalysis,
        expression: &ExpressionAnalysis,
        parameter_name: &str,
        expected: TypeId,
    ) {
        let trace = self.generic_trace(callable, expression);
        let matches = trace
            .iter()
            .filter_map(|node| match &node.step {
                ExplanationStep::GenericSolution { parameter, ty, status } if self.parameter_name(*parameter) == parameter_name => {
                    Some((*parameter, *ty, *status))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            matches.len(),
            1,
            "name-based generic solution lookup for `{parameter_name}` is ambiguous or missing; use exact TypeParameterId when scopes overlap: {trace:#?}"
        );
        assert_eq!(matches[0].1, expected, "wrong solution for `{parameter_name}`");
    }

    pub fn assert_generic_constraint_exact(
        &self,
        callable: &CallableAnalysis,
        expression: &ExpressionAnalysis,
        parameter: TypeParameterId,
        expected_origin: GenericConstraintOrigin,
        expected_relation: GenericConstraintRelation,
    ) {
        let trace = self.generic_trace(callable, expression);
        assert!(
            trace.iter().any(|node| {
                matches!(
                    &node.step,
                    ExplanationStep::GenericConstraint { parameter: found, origin, relation }
                        if *found == parameter && *origin == expected_origin && *relation == expected_relation
                )
            }),
            "missing exact generic constraint for {parameter:?} from {expected_origin:?} with {expected_relation:?}: {trace:#?}"
        );
    }

    pub fn generic_constraint_count(&self, callable: &CallableAnalysis, expression: &ExpressionAnalysis, parameter: TypeParameterId) -> usize {
        self.generic_trace(callable, expression)
            .iter()
            .filter(|node| matches!(&node.step, ExplanationStep::GenericConstraint { parameter: found, .. } if *found == parameter))
            .count()
    }

    pub fn assert_generic_constraint_origin(
        &self,
        callable: &CallableAnalysis,
        expression: &ExpressionAnalysis,
        parameter_name: &str,
        expected_origin: GenericConstraintOrigin,
    ) {
        let trace = self.generic_trace(callable, expression);
        let matching_parameters = trace
            .iter()
            .filter_map(|node| match &node.step {
                ExplanationStep::GenericConstraint { parameter, origin, .. }
                    if self.parameter_name(*parameter) == parameter_name && *origin == expected_origin => Some(*parameter),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            matching_parameters.len(),
            1,
            "name-based generic constraint lookup for `{parameter_name}` is ambiguous or missing; use exact TypeParameterId when scopes overlap: {trace:#?}"
        );
    }

    pub fn assert_solution_parameter_is_callable_owned(
        &self,
        callable: &CallableAnalysis,
        expression: &ExpressionAnalysis,
        parameter_name: &str,
    ) {
        let trace = self.generic_trace(callable, expression);
        let parameters = trace
            .iter()
            .filter_map(|node| match &node.step {
                ExplanationStep::GenericSolution { parameter, .. } if self.parameter_name(*parameter) == parameter_name => Some(*parameter),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(parameters.len(), 1, "expected one solution parameter named `{parameter_name}`: {trace:#?}");
        assert!(
            matches!(self.analysis.snapshot.store.type_parameter(parameters[0]).owner, TypeParameterOwner::Callable(_)),
            "method parameter `{parameter_name}` must remain callable-owned"
        );
    }

    pub fn diagnostics(&self, code: DiagnosticCode) -> Vec<&SemanticDiagnostic> {
        self.analysis.snapshot.all_diagnostics().filter(|diagnostic| diagnostic.code == code).collect()
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

    pub fn assert_no_errors(&self) {
        let errors = self
            .analysis
            .snapshot
            .all_diagnostics()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            .collect::<Vec<_>>();
        assert!(errors.is_empty(), "unexpected semantic errors: {errors:#?}");
    }
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
