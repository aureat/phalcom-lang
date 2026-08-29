use phalcom_ast::parse;
use phalcom_common::selector::{Selector, SelectorBase};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::BindingConsistency;
use phalcom_semantic::checker::analysis::{BindingState, CallableAnalysis, ExpressionAnalysis};
use phalcom_semantic::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use phalcom_semantic::explain::{EvidenceRef, ExplanationStep};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::types::evidence::{EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use phalcom_semantic::{Assignability, TypeId, analyze_single_module, check_assignability, is_subtype};
use std::sync::Arc;

type Analysis = phalcom_semantic::workspace::SemanticAnalysis;
use crate::semantic::support::Fixture;

fn analyze(source_text: &str) -> (ModuleId, Arc<str>, Analysis) {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(source_text);
    let parsed = parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    (module, source, analysis)
}

fn decl(module: &ModuleId, name: &str) -> DeclarationId {
    DeclarationId::new(module.clone(), name.into())
}

fn ty(analysis: &Analysis, module: &ModuleId, name: &str) -> TypeId {
    analysis
        .snapshot
        .declarations
        .form(&decl(module, name))
        .unwrap_or_else(|| panic!("missing type form for {name}"))
}

fn zero_arg_callable(module: &ModuleId, owner: &str, name: &str, side: DispatchSide) -> CallableId {
    CallableId::new(decl(module, owner), Selector::method(name, vec![]).unwrap(), side)
}

fn named_callable(analysis: &Analysis, module: &ModuleId, owner: &str, name: &str, side: DispatchSide) -> CallableId {
    let owner = decl(module, owner);
    let matches = analysis
        .snapshot
        .callable_analyses
        .keys()
        .filter(|id| id.owner == owner && id.side == side && matches!(&id.selector.base, SelectorBase::Named(base) if base == name))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "expected one callable {owner:?}.{name} on {side:?}: {matches:#?}");
    matches[0].clone()
}

fn callable_analysis<'a>(analysis: &'a Analysis, id: &CallableId) -> &'a CallableAnalysis {
    analysis
        .snapshot
        .callable_analyses
        .get(id)
        .unwrap_or_else(|| panic!("missing callable analysis for {id:?}"))
}

fn binding<'a>(analysis: &'a CallableAnalysis, name: &str) -> &'a BindingState {
    analysis
        .bindings
        .values()
        .find(|binding| binding.name == name)
        .unwrap_or_else(|| panic!("missing binding `{name}`; bindings={:#?}", analysis.bindings))
}

fn expression<'a>(analysis: &'a CallableAnalysis, source: &str, expected_text: &str) -> &'a ExpressionAnalysis {
    let matches = analysis
        .expressions
        .values()
        .filter(|expr| source.get(expr.range.start..expr.range.end) == Some(expected_text))
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "expected one `{expected_text}` expression in {:?}, found {}: {:#?}",
        analysis.callable,
        matches.len(),
        matches
    );
    matches[0]
}

fn diagnostics(analysis: &Analysis, code: DiagnosticCode) -> Vec<&SemanticDiagnostic> {
    analysis.snapshot.all_diagnostics().filter(|diagnostic| diagnostic.code == code).collect()
}

fn assert_method_call_evidence(analysis: &CallableAnalysis, expression: &ExpressionAnalysis, expected_type: TypeId, expected_origin: EvidenceOrigin) {
    assert_eq!(
        expression.knowledge.ty(),
        Some(expected_type),
        "expected method call to be proven to return {expected_type:?}: {expression:#?}"
    );
    let explanation_id = expression.explanation.expect("known method call must retain explanation evidence");
    let node = analysis
        .explanations
        .get(explanation_id)
        .unwrap_or_else(|| panic!("missing explanation {explanation_id:?}"));

    match &node.step {
        ExplanationStep::MethodCall { return_ty, .. } => assert_eq!(*return_ty, expected_type),
        other => panic!("expected method-call explanation, got {other:?}"),
    }
    assert_eq!(node.status, EvidenceStatus::Established);
    assert_eq!(node.origin, expected_origin);
    assert!(
        node.evidence
            .iter()
            .any(|evidence| matches!(evidence, EvidenceRef::SourceSpan(range) if *range == expression.range)),
        "method-call explanation must retain source span: {node:#?}"
    );
    assert!(
        node.evidence
            .iter()
            .any(|evidence| matches!(evidence, EvidenceRef::TypeId(ty) if *ty == expected_type)),
        "method-call explanation must retain result type: {node:#?}"
    );
}

#[test]
fn direct_constructor_result_is_proven_and_records_constructor_dependency() {
    let source_text = r#"
class CellNum {
  @constructor
  new() {}
}
class Probe {
  @class
  run() {
    let x = CellNum.new()
  }
}
"#;
    let (module, source, analysis) = analyze(source_text);
    let cell_num = ty(&analysis, &module, "CellNum");
    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);
    let new_id = zero_arg_callable(&module, "CellNum", "new", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);

    assert_method_call_evidence(run, expression(run, &source, "CellNum.new()"), cell_num, EvidenceOrigin::ConstructorSemantics);
    assert_eq!(binding(run, "x").current.ty(), Some(cell_num));

    assert_eq!(binding(run, "x").current.status(), Some(EvidenceStatus::Established));
    assert_eq!(binding(run, "x").current.origin(), Some(EvidenceOrigin::ConstructorSemantics));

    assert!(run.dependencies.contains(&new_id));
    assert!(diagnostics(&analysis, DiagnosticCode::BindingInitializerMismatch).is_empty());
}

#[test]
fn class_factory_tail_inference_propagates_constructor_proof_to_callsite() {
    let source_text = r#"
class CellNum {
  @constructor
  new() {}

  @class
  of() {
    CellNum.new()
  }
}
class Probe {
  @class
  run() {
    let x = CellNum.of()
  }
}
"#;
    let (module, source, analysis) = analyze(source_text);
    let cell_num = ty(&analysis, &module, "CellNum");
    let new_id = zero_arg_callable(&module, "CellNum", "new", DispatchSide::Class);
    let of_id = zero_arg_callable(&module, "CellNum", "of", DispatchSide::Class);
    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);

    let factory = callable_analysis(&analysis, &of_id);
    assert_method_call_evidence(
        factory,
        expression(factory, &source, "CellNum.new()"),
        cell_num,
        EvidenceOrigin::ConstructorSemantics,
    );
    assert!(factory.dependencies.contains(&new_id));

    let run = callable_analysis(&analysis, &run_id);
    assert_eq!(binding(run, "x").current.ty(), Some(cell_num));
    assert_method_call_evidence(run, expression(run, &source, "CellNum.of()"), cell_num, EvidenceOrigin::CallableSignature);
    assert!(run.dependencies.contains(&of_id));
    assert_eq!(binding(run, "x").current.ty(), Some(cell_num));
}

#[test]
fn refuted_annotation_cannot_override_proven_constructor_type_or_downstream_dispatch() {
    let source_text = r#"
class CellNum {
  @constructor
  new() {} // implicitly -> Self

  cellOnly() -> Int { 1 } // -> Int - exact syntax (literal) evidence
}
class Probe {
  @class
  run() {
    let x: Int = CellNum.new()
    let y = x.cellOnly()
  }
}
"#;
    let (module, source, analysis) = analyze(source_text);
    let cell_num = ty(&analysis, &module, "CellNum");
    let int_ty = ty(&analysis, &module, "Int");
    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);
    let new_id = zero_arg_callable(&module, "CellNum", "new", DispatchSide::Class);
    let cell_only_id = zero_arg_callable(&module, "CellNum", "cellOnly", DispatchSide::Instance);
    let run = callable_analysis(&analysis, &run_id);

    assert_method_call_evidence(run, expression(run, &source, "CellNum.new()"), cell_num, EvidenceOrigin::ConstructorSemantics);
    let x = binding(run, "x");
    assert_eq!(x.declared_type(), Some(int_ty));
    assert_eq!(
        x.current.ty(),
        Some(cell_num),
        "refuted developer annotation must not replace proven checker knowledge"
    );
    assert_eq!(x.current.status(), Some(EvidenceStatus::Established));
    assert_eq!(x.current.origin(), Some(EvidenceOrigin::ConstructorSemantics));

    let mismatch = diagnostics(&analysis, DiagnosticCode::BindingInitializerMismatch);
    assert_eq!(mismatch.len(), 1, "expected one contradiction: {mismatch:#?}");
    assert!(mismatch[0].labels.iter().any(|label| label.message == "declared type"));
    assert!(mismatch[0].labels.iter().any(|label| label.message == "inferred type"));

    assert_method_call_evidence(run, expression(run, &source, "x.cellOnly()"), int_ty, EvidenceOrigin::CallableSignature);
    assert_eq!(binding(run, "y").current.ty(), Some(int_ty));
    assert!(run.dependencies.contains(&new_id));
    assert!(run.dependencies.contains(&cell_only_id));
}

#[test]
fn compatible_supertype_annotation_is_proven_but_narrow_value_knowledge_is_preserved() {
    let source_text = r#"
class Base {}
class Derived is Base {
  @constructor
  new() {}

  derivedOnly() -> Int { 1 }
}
class Probe {
  @class
  run() {
    let x: Base = Derived.new()
    let y = x.derivedOnly()
  }
}
"#;
    let (module, source, analysis) = analyze(source_text);
    let base = ty(&analysis, &module, "Base");
    let derived = ty(&analysis, &module, "Derived");
    let int_ty = ty(&analysis, &module, "Int");

    assert!(is_subtype(&analysis.snapshot.store, analysis.snapshot.hierarchy.as_ref(), derived, base));
    assert_eq!(
        check_assignability(
            &analysis.snapshot.store,
            analysis.snapshot.hierarchy.as_ref(),
            &TypeKnowledge::established(derived, EvidenceOrigin::Flow),
            &TypeKnowledge::assumed(base, EvidenceOrigin::DeveloperAnnotation),
        ),
        Assignability::Assignable
    );

    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);
    let derived_only_id = zero_arg_callable(&module, "Derived", "derivedOnly", DispatchSide::Instance);
    let run = callable_analysis(&analysis, &run_id);
    assert_method_call_evidence(run, expression(run, &source, "Derived.new()"), derived, EvidenceOrigin::ConstructorSemantics);

    let x = binding(run, "x");
    assert_eq!(x.declared_type(), Some(base));
    assert_eq!(
        x.current.ty(),
        Some(derived),
        "a compatible annotation is a constraint, not permission to erase stronger proof"
    );
    assert_eq!(x.current.status(), Some(EvidenceStatus::Established));
    assert_eq!(x.current.origin(), Some(EvidenceOrigin::ConstructorSemantics));
    assert!(diagnostics(&analysis, DiagnosticCode::BindingInitializerMismatch).is_empty());

    assert_method_call_evidence(run, expression(run, &source, "x.derivedOnly()"), int_ty, EvidenceOrigin::CallableSignature);
    assert_eq!(binding(run, "y").current.ty(), Some(int_ty));
    assert!(run.dependencies.contains(&derived_only_id));
}

#[test]
fn unknown_initializer_allows_developer_annotation_to_supply_binding_knowledge() {
    let source_text = r#"
class Probe {
  @class
  run(value) {
    let x: Int = value
  }
}
"#;
    let (module, source, analysis) = analyze(source_text);
    let int_ty = ty(&analysis, &module, "Int");
    let run_id = named_callable(&analysis, &module, "Probe", "run", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);

    let parameter_use = expression(run, &source, "value");
    assert!(
        parameter_use.knowledge.is_unknown(),
        "unannotated parameter should remain unknown evidence: {parameter_use:#?}"
    );

    let x = binding(run, "x");
    assert_eq!(x.declared_type(), Some(int_ty));
    assert_eq!(x.current.ty(), Some(int_ty));
    assert_eq!(x.current.status(), Some(EvidenceStatus::Assumed));
    assert_eq!(x.current.origin(), Some(EvidenceOrigin::DeveloperAnnotation));
    assert!(diagnostics(&analysis, DiagnosticCode::BindingInitializerMismatch).is_empty());
}

#[test]
fn argument_refutation_does_not_erase_independently_known_call_return_type() {
    let source_text = r#"
class CellNum {
  @constructor
  new() {}

  @class
  fromInt(_ value: Int) -> CellNum {
    CellNum.new()
  }
}
class Probe {
  @class
  run() {
    let x = CellNum.fromInt("bad")
  }
}
"#;
    let (module, source, analysis) = analyze(source_text);
    let cell_num = ty(&analysis, &module, "CellNum");
    let int_ty = ty(&analysis, &module, "Int");
    let string_ty = ty(&analysis, &module, "String");
    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);
    let from_int_id = named_callable(&analysis, &module, "CellNum", "fromInt", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);

    assert!(matches!(
        check_assignability(
            &analysis.snapshot.store,
            analysis.snapshot.hierarchy.as_ref(),
            &TypeKnowledge::established(string_ty, EvidenceOrigin::Syntax),
            &TypeKnowledge::assumed(int_ty, EvidenceOrigin::DeveloperAnnotation),
        ),
        Assignability::Refuted { .. }
    ));

    assert_method_call_evidence(
        run,
        expression(run, &source, "CellNum.fromInt(\"bad\")"),
        cell_num,
        EvidenceOrigin::CallableSignature,
    );
    let invalid_call = expression(run, &source, "CellNum.fromInt(\"bad\")");
    assert!(
        invalid_call.status.is_invalid(),
        "argument mismatch must own call invalidity: {invalid_call:#?}"
    );
    assert!(!matches!(
        invalid_call.causal_invalidity,
        phalcom_semantic::checker::causal::CausalInvalidity::Clean
    ));
    let invalid_explanation = run
        .explanations
        .get(invalid_call.explanation.expect("invalid call explanation"))
        .expect("call explanation");
    assert!(
        !invalid_explanation.parents.is_empty(),
        "call explanation must retain argument derivation parents"
    );
    assert_eq!(binding(run, "x").current.ty(), Some(cell_num));
    assert!(run.dependencies.contains(&from_int_id));

    let mismatches = diagnostics(&analysis, DiagnosticCode::ArgumentMismatch);
    assert_eq!(mismatches.len(), 1, "expected one proven argument mismatch: {mismatches:#?}");
    assert!(mismatches[0].message.contains("does not match expected parameter type"));
}

#[test]
fn return_annotation_refutation_preserves_proven_tail_expression_evidence() {
    let source_text = r#"
class CellNum {
  @constructor
  new() {}
}
class Factory {
  @class
  make() -> Int {
    CellNum.new()
  }
}
"#;
    let (module, source, analysis) = analyze(source_text);
    let cell_num = ty(&analysis, &module, "CellNum");
    let int_ty = ty(&analysis, &module, "Int");
    let make_id = zero_arg_callable(&module, "Factory", "make", DispatchSide::Class);
    let new_id = zero_arg_callable(&module, "CellNum", "new", DispatchSide::Class);
    let make = callable_analysis(&analysis, &make_id);

    assert_method_call_evidence(make, expression(make, &source, "CellNum.new()"), cell_num, EvidenceOrigin::ConstructorSemantics);
    assert!(make.dependencies.contains(&new_id));
    assert_eq!(diagnostics(&analysis, DiagnosticCode::ReturnMismatch).len(), 1);

    let surface = analysis.snapshot.surfaces.get(&decl(&module, "Factory")).unwrap();
    let signature = surface.get_callable(DispatchSide::Class, &Selector::method("make", vec![]).unwrap()).unwrap();
    assert_eq!(
        signature.return_type.ty(),
        Some(int_ty),
        "declared public contract stays Int while body evidence independently proves CellNum"
    );
}

#[test]
fn inherited_constructor_specializes_self_before_annotation_refutation() {
    let source_text = r#"
class Base {
  @constructor
  new() {}
}
class Derived is Base {}
class Probe {
  @class
  run() {
    let x: String = Derived.new()
  }
}
"#;
    let (module, source, analysis) = analyze(source_text);
    let derived = ty(&analysis, &module, "Derived");
    let string_ty = ty(&analysis, &module, "String");
    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);
    let inherited_new_id = zero_arg_callable(&module, "Base", "new", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);

    assert_method_call_evidence(run, expression(run, &source, "Derived.new()"), derived, EvidenceOrigin::ConstructorSemantics);
    assert!(run.dependencies.contains(&inherited_new_id));

    let x = binding(run, "x");
    assert_eq!(x.declared_type(), Some(string_ty));
    assert_eq!(x.current.ty(), Some(derived));
    assert!(matches!(
        check_assignability(
            &analysis.snapshot.store,
            analysis.snapshot.hierarchy.as_ref(),
            &TypeKnowledge::established(derived, EvidenceOrigin::Flow),
            &TypeKnowledge::assumed(string_ty, EvidenceOrigin::DeveloperAnnotation),
        ),
        Assignability::Refuted { .. }
    ));
    assert_eq!(diagnostics(&analysis, DiagnosticCode::BindingInitializerMismatch).len(), 1);
}

/// PASS
/// Analysis understands the difference between a constructor call and an ordinary inherited class method call.
///
/// The constructor call is specialized to the receiving class (Derived.new() -> Derived),
/// while the ordinary inherited method call is not specialized (Derived.ordinary() -> Base).
/// Constructor has an implicit `Self` return type that is specialized to the derived class.
#[test]
fn ordinary_inherited_class_return_is_not_specialized_like_constructor_self() {
    let source_text = r#"
class Base {
  @constructor
  new() {}

  @class
  ordinary() -> Base {
    Base.new()
  }
}

class Derived is Base {}

class Probe {
  @class
  run() {
    let constructed = Derived.new() // -> Derived
    let ordinary = Derived.ordinary() // -> Base
  }
}
"#;
    let (module, source, analysis) = analyze(source_text);
    let base = ty(&analysis, &module, "Base"); // type Base
    let derived = ty(&analysis, &module, "Derived"); // type Derived
    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);
    let inherited_new_id = zero_arg_callable(&module, "Base", "new", DispatchSide::Class);
    let ordinary_id = zero_arg_callable(&module, "Base", "ordinary", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);

    assert_method_call_evidence(run, expression(run, &source, "Derived.new()"), derived, EvidenceOrigin::ConstructorSemantics);
    assert_method_call_evidence(run, expression(run, &source, "Derived.ordinary()"), base, EvidenceOrigin::CallableSignature);
    assert_eq!(binding(run, "constructed").current.ty(), Some(derived));
    assert_eq!(binding(run, "ordinary").current.ty(), Some(base));
    assert!(run.dependencies.contains(&inherited_new_id));
    assert!(run.dependencies.contains(&ordinary_id));
    assert!(!analysis.snapshot.has_errors(), "{:#?}", analysis.snapshot.diagnostics);
}

#[test]
fn binding_kind_controls_mutability_and_immutable_write_preserves_state() {
    let source_text = r#"
class Probe {
  @class
  run() {
    let mutable = 1
    mutable = 2
    const constant = 1
    constant = 2
  }
}
"#;
    let (module, source, analysis) = analyze(source_text);
    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);

    let mutable = binding(run, "mutable");
    assert!(mutable.mutable);
    assert_eq!(mutable.current.ty(), Some(ty(&analysis, &module, "Int")));

    let constant = binding(run, "constant");
    assert!(!constant.mutable);
    assert_eq!(constant.current.ty(), Some(ty(&analysis, &module, "Int")));
    assert_eq!(constant.version, 0, "rejected immutable write must not advance recovery state");
    assert_eq!(diagnostics(&analysis, DiagnosticCode::AssignmentToImmutable).len(), 1);
    assert!(run.expressions.values().any(|expression| expression.range
        == (source.find("constant = 2").unwrap()..source.find("constant = 2").unwrap() + "constant = 2".len()).into()
        && expression.status.is_invalid()));
}

#[test]
fn same_scope_redeclaration_preserves_first_binding_identity_and_fact() {
    let source_text = r#"
class Probe {
  @class
  run() {
    let value = 1
    let value = "shadow attempt"
    let copy = value
  }
}
"#;
    let (module, _source, analysis) = analyze(source_text);
    let int_ty = ty(&analysis, &module, "Int");
    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);

    let value = binding(run, "value");
    assert_eq!(value.current.ty(), Some(int_ty));
    assert_eq!(binding(run, "copy").current.ty(), Some(int_ty));
    assert_eq!(diagnostics(&analysis, DiagnosticCode::BindingRedeclared).len(), 1);
}

#[test]
fn missing_initializer_stays_ineligible_instead_of_laundering_annotation() {
    let source_text = r#"
class Probe {
  @class
  run() {
    const constant: Int
    let annotated: Int
    let plain
  }
}
"#;
    let (module, _source, analysis) = analyze(source_text);
    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);

    assert!(matches!(
        binding(run, "constant").current,
        TypeKnowledge::Unknown(UnknownReason::MissingInitializer)
    ));
    assert!(matches!(
        binding(run, "annotated").current,
        TypeKnowledge::Unknown(UnknownReason::MissingInitializer)
    ));
    assert!(matches!(
        binding(run, "plain").current,
        TypeKnowledge::Unknown(UnknownReason::MissingInitializer)
    ));
    assert_eq!(diagnostics(&analysis, DiagnosticCode::ConstWithoutInitializer).len(), 1);
}

#[test]
fn exact_literal_proof_refutes_annotation_and_remains_current_binding_knowledge() {
    let source_text = r#"
class Probe {
  @class
  run() {
    let x: String = 42
  }
}
"#;
    let (module, source, analysis) = analyze(source_text);
    let int_ty = ty(&analysis, &module, "Int");
    let string_ty = ty(&analysis, &module, "String");
    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);

    let literal = expression(run, &source, "42");
    assert_eq!(literal.knowledge.ty(), Some(int_ty));
    assert_eq!(literal.knowledge.status(), Some(EvidenceStatus::Established));
    assert_eq!(literal.knowledge.origin(), Some(EvidenceOrigin::Syntax));
    let explanation = run
        .explanations
        .get(literal.explanation.expect("literal explanation"))
        .expect("literal explanation node");
    assert!(matches!(
        explanation.step,
        ExplanationStep::Literal { ty, .. } if ty == int_ty
    ));
    assert!(matches!(
        check_assignability(
            &analysis.snapshot.store,
            analysis.snapshot.hierarchy.as_ref(),
            &literal.knowledge,
            &TypeKnowledge::assumed(string_ty, EvidenceOrigin::DeveloperAnnotation),
        ),
        Assignability::Refuted { .. }
    ));

    let x = binding(run, "x");
    assert_eq!(x.declared_type(), Some(string_ty));
    assert_eq!(
        x.current.ty(),
        Some(int_ty),
        "exact checker proof must outrank contradictory developer annotation"
    );
    assert_eq!(x.current.status(), Some(EvidenceStatus::Established));
    assert_eq!(x.current.origin(), Some(EvidenceOrigin::Syntax));
    assert_eq!(diagnostics(&analysis, DiagnosticCode::BindingInitializerMismatch).len(), 1);
}

#[test]
fn exact_literal_proof_validates_supertype_annotation_without_losing_precision() {
    let source_text = r#"
class Probe {
  @class
  run() {
    let x: Number = 42
  }
}
"#;

    let (module, source, analysis) = analyze(source_text);

    let int_ty = ty(&analysis, &module, "Int");
    let number_ty = ty(&analysis, &module, "Number");
    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);

    let literal = expression(run, &source, "42");
    assert_eq!(literal.knowledge.ty(), Some(int_ty));
    assert_eq!(literal.knowledge.status(), Some(EvidenceStatus::Established));
    assert_eq!(literal.knowledge.origin(), Some(EvidenceOrigin::Syntax));

    let x = binding(run, "x");
    assert_eq!(x.declared_type(), Some(number_ty));
    assert_eq!(x.current.ty(), Some(int_ty));
    assert_eq!(x.current.status(), Some(EvidenceStatus::Established));
    assert_eq!(x.current.origin(), Some(EvidenceOrigin::Syntax));

    assert!(matches!(
        check_assignability(
            &analysis.snapshot.store,
            analysis.snapshot.hierarchy.as_ref(),
            &literal.knowledge,
            &TypeKnowledge::assumed(number_ty, EvidenceOrigin::DeveloperAnnotation),
        ),
        Assignability::Assignable
    ));
}

#[test]
fn binding_contract_explanation_preserves_actual_and_relation_outcome() {
    let source_text = r#"
class Probe {
  @class
  run() {
    let x: Number = 42
  }
}
"#;
    let (module, _source, analysis) = analyze(source_text);
    let int_ty = ty(&analysis, &module, "Int");
    let number_ty = ty(&analysis, &module, "Number");
    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);
    let x = binding(run, "x");
    let explanation = run
        .explanations
        .get(x.explanation.expect("binding contract explanation"))
        .expect("binding explanation node");

    match &explanation.step {
        ExplanationStep::BindingContract {
            actual, contract, consistency, ..
        } => {
            assert_eq!(actual.ty(), Some(int_ty));
            assert_eq!(*contract, number_ty);
            assert_eq!(*consistency, BindingConsistency::Validated);
        }
        other => panic!("expected binding-contract explanation, got {other:?}"),
    }
    assert_eq!(explanation.status, EvidenceStatus::Established);
    assert_eq!(explanation.origin, EvidenceOrigin::Syntax);
}

#[test]
fn annotation_diagnostic_root_cause_marks_binding_without_erasing_value() {
    let source_text = r#"
class Probe {
  @class
  run() {
    let x: Missing = 42
  }
}
"#;
    let (module, _source, analysis) = analyze(source_text);
    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);
    let x = binding(run, "x");

    assert_eq!(x.current.ty(), Some(ty(&analysis, &module, "Int")));
    assert_eq!(x.current.status(), Some(EvidenceStatus::Established));
    assert!(matches!(x.causal_invalidity, phalcom_semantic::checker::CausalInvalidity::One(_)));
    let diagnostic = diagnostics(&analysis, DiagnosticCode::AnnotationUnresolved)
        .into_iter()
        .next()
        .expect("unresolved annotation diagnostic");
    assert_eq!(
        diagnostic.root_cause,
        x.causal_invalidity.suppression_cause().and_then(|cause| match cause {
            phalcom_semantic::checker::SuppressionCause::One(id) => Some(id),
            phalcom_semantic::checker::SuppressionCause::Multiple => None,
        })
    );
}

#[test]
fn unannotated_callable_summary_distinguishes_value_unit_and_never_paths() {
    let source_text = r#"
class Probe {
  @class
  explicit() {
    return 42
  }

  @class
  tail() {
    42
  }

  @class
  binding() {
    const x = 42
  }

  @class
  abrupt() {
    throw 42
  }
}
"#;
    let (module, _source, analysis) = analyze(source_text);
    let int_ty = ty(&analysis, &module, "Int");
    let unit_ty = analysis.snapshot.store.unit();
    let never_ty = analysis.snapshot.store.never();
    let surface = analysis.snapshot.surfaces.get(&decl(&module, "Probe")).unwrap();

    let return_type = |name: &str| {
        surface
            .get_callable(DispatchSide::Class, &Selector::method(name, vec![]).unwrap())
            .unwrap()
            .return_type
            .ty()
    };

    assert_eq!(return_type("explicit"), Some(int_ty));
    assert_eq!(return_type("tail"), Some(int_ty));
    assert_eq!(return_type("binding"), Some(unit_ty));
    assert_eq!(return_type("abrupt"), Some(never_ty));

    for name in ["explicit", "tail", "binding", "abrupt"] {
        let id = zero_arg_callable(&module, "Probe", name, DispatchSide::Class);
        let body = callable_analysis(&analysis, &id);
        assert_eq!(
            body.exits.normal_returns.len(),
            if name == "abrupt" { 0 } else { 1 },
            "summary paths for {name}"
        );
    }
}

#[test]
fn exact_dispatch_does_not_upgrade_assumed_source_return() {
    let f = Fixture::new(
        r#"
class Echo {
  @class
  echo(_ value: String) -> String {
    value
  }
}

class Probe {
  @class
  run() {
    let result = Echo.echo("hello")
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let result = f.binding(run, "result");
    assert_eq!(result.current.status(), Some(EvidenceStatus::Assumed));
}

#[test]
fn established_body_certifies_declared_public_return_without_narrowing_api() {
    let f = Fixture::new(
        r#"
class Animal {}
class Dog is Animal {
  @constructor
  new() {}
}
class Factory {
  @class
  make() -> Animal {
    Dog.new()
  }
}
class Probe {
  @class
  run() {
    let result = Factory.make()
  }
}
"#,
    );

    let animal = f.ty("Animal");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let result = f.binding(run, "result");
    assert_eq!(result.current.ty(), Some(animal));
    assert_eq!(result.current.status(), Some(EvidenceStatus::Established));
}

#[test]
fn invalid_tail_recovery_knowledge_is_not_published_as_inferred_return() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  broken() {
    let value: Int = "wrong"
    value
  }

  @class
  run() {
    let result = Probe.broken()
  }
}
"#,
    );

    let broken = f.callable("Probe", "broken", DispatchSide::Class);
    assert!(broken.exits.normal_returns.iter().all(|exit| !exit.publication_knowledge().is_established()));
}

#[test]
fn incompatible_body_refutes_source_contract_without_establishing_it() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  broken() -> Int {
    "wrong"
  }
}
"#,
    );

    assert!(!f.diagnostics(DiagnosticCode::ReturnMismatch).is_empty());
}

