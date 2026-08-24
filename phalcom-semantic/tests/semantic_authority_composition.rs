use phalcom_ast::parse;
use phalcom_common::selector::{Selector, SelectorBase};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::analysis::{BindingState, CallableAnalysis, ExpressionAnalysis};
use phalcom_semantic::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use phalcom_semantic::explain::{EvidenceRef, ExplanationStep};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::types::evidence::{EvidenceAuthority, TypeKnowledge};
use phalcom_semantic::{Assignability, TypeId, analyze_single_module, check_assignability, is_subtype};
use std::sync::Arc;

type Analysis = phalcom_semantic::workspace::SemanticAnalysis;

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
        .filter(|id| {
            id.owner == owner
                && id.side == side
                && matches!(&id.selector.base, SelectorBase::Named(base) if base == name)
        })
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

fn expression_by_fact(
    analysis: &CallableAnalysis,
    expected_type: TypeId,
    authority: EvidenceAuthority,
) -> &ExpressionAnalysis {
    let matches = analysis
        .expressions
        .values()
        .filter(|expr| {
            expr.knowledge.ty() == Some(expected_type) && expr.knowledge.authority() == Some(authority)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "expected one expression with type {expected_type:?} and authority {authority:?}, found {}: {matches:#?}",
        matches.len()
    );
    matches[0]
}

fn diagnostics(analysis: &Analysis, code: DiagnosticCode) -> Vec<&SemanticDiagnostic> {
    analysis.snapshot.all_diagnostics().filter(|diagnostic| diagnostic.code == code).collect()
}

fn assert_method_call_evidence(analysis: &CallableAnalysis, expression: &ExpressionAnalysis, expected_type: TypeId) {
    assert_eq!(expression.knowledge.ty(), Some(expected_type));
    let explanation_id = expression.explanation.expect("known method call must retain explanation evidence");
    let node = analysis
        .explanations
        .get(explanation_id)
        .unwrap_or_else(|| panic!("missing explanation {explanation_id:?}"));

    match &node.step {
        ExplanationStep::MethodCall { return_ty, .. } => assert_eq!(*return_ty, expected_type),
        other => panic!("expected method-call explanation, got {other:?}"),
    }
    assert_eq!(node.authority, EvidenceAuthority::Proven);
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

    assert_method_call_evidence(run, expression(run, &source, "CellNum.new()"), cell_num);
    assert_eq!(binding(run, "x").current.ty(), Some(cell_num));
    assert_eq!(binding(run, "x").current.authority(), Some(EvidenceAuthority::Proven));
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
    assert_method_call_evidence(factory, expression(factory, &source, "CellNum.new()"), cell_num);
    assert!(factory.dependencies.contains(&new_id));

    let run = callable_analysis(&analysis, &run_id);
    assert_method_call_evidence(run, expression(run, &source, "CellNum.of()"), cell_num);
    assert!(run.dependencies.contains(&of_id));
    assert_eq!(binding(run, "x").current.ty(), Some(cell_num));
}

#[test]
fn refuted_annotation_cannot_override_proven_constructor_type_or_downstream_dispatch() {
    let source_text = r#"
class CellNum {
  @constructor
  new() {}

  cellOnly() -> Int { 1 }
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

    assert_method_call_evidence(run, expression(run, &source, "CellNum.new()"), cell_num);
    let x = binding(run, "x");
    assert_eq!(x.declared, Some(int_ty));
    assert_eq!(
        x.current.ty(),
        Some(cell_num),
        "refuted developer annotation must not replace proven checker knowledge"
    );
    assert_eq!(x.current.authority(), Some(EvidenceAuthority::Proven));

    let mismatch = diagnostics(&analysis, DiagnosticCode::BindingInitializerMismatch);
    assert_eq!(mismatch.len(), 1, "expected one contradiction: {mismatch:#?}");
    assert!(mismatch[0].labels.iter().any(|label| label.message == "declared type"));
    assert!(mismatch[0].labels.iter().any(|label| label.message == "inferred type"));

    assert_method_call_evidence(run, expression(run, &source, "x.cellOnly()"), int_ty);
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
            &TypeKnowledge::known(derived, EvidenceAuthority::Proven),
            &TypeKnowledge::known(base, EvidenceAuthority::Declared),
        ),
        Assignability::Assignable
    );

    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);
    let derived_only_id = zero_arg_callable(&module, "Derived", "derivedOnly", DispatchSide::Instance);
    let run = callable_analysis(&analysis, &run_id);
    assert_method_call_evidence(run, expression(run, &source, "Derived.new()"), derived);

    let x = binding(run, "x");
    assert_eq!(x.declared, Some(base));
    assert_eq!(
        x.current.ty(),
        Some(derived),
        "a compatible annotation is a constraint, not permission to erase stronger proof"
    );
    assert_eq!(x.current.authority(), Some(EvidenceAuthority::Proven));
    assert!(diagnostics(&analysis, DiagnosticCode::BindingInitializerMismatch).is_empty());

    assert_method_call_evidence(run, expression(run, &source, "x.derivedOnly()"), int_ty);
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
    assert_eq!(x.declared, Some(int_ty));
    assert_eq!(x.current.ty(), Some(int_ty));
    assert_eq!(x.current.authority(), Some(EvidenceAuthority::Declared));
    assert!(diagnostics(&analysis, DiagnosticCode::BindingInitializerMismatch).is_empty());
}

#[test]
fn argument_refutation_does_not_erase_independently_known_call_return_type() {
    let source_text = r#"
class CellNum {
  @constructor
  new() {}

  @class
  fromInt(value: Int) -> CellNum {
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
    let string_ty = ty(&analysis, &module, "String");
    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);
    let from_int_id = named_callable(&analysis, &module, "CellNum", "fromInt", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);

    let argument = expression_by_fact(run, string_ty, EvidenceAuthority::ExactSyntax);
    assert!(source.get(argument.range.start..argument.range.end).is_some());

    assert_method_call_evidence(run, expression(run, &source, "CellNum.fromInt(\"bad\")"), cell_num);
    assert_eq!(binding(run, "x").current.ty(), Some(cell_num));
    assert!(run.dependencies.contains(&from_int_id));
    assert_eq!(diagnostics(&analysis, DiagnosticCode::ArgumentMismatch).len(), 1);
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

    assert_method_call_evidence(make, expression(make, &source, "CellNum.new()"), cell_num);
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

    assert_method_call_evidence(run, expression(run, &source, "Derived.new()"), derived);
    assert!(run.dependencies.contains(&inherited_new_id));

    let x = binding(run, "x");
    assert_eq!(x.declared, Some(string_ty));
    assert_eq!(x.current.ty(), Some(derived));
    assert!(matches!(
        check_assignability(
            &analysis.snapshot.store,
            analysis.snapshot.hierarchy.as_ref(),
            &TypeKnowledge::known(derived, EvidenceAuthority::Proven),
            &TypeKnowledge::known(string_ty, EvidenceAuthority::Declared),
        ),
        Assignability::Refuted { .. }
    ));
    assert_eq!(diagnostics(&analysis, DiagnosticCode::BindingInitializerMismatch).len(), 1);
}

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
    let constructed = Derived.new()
    let ordinary = Derived.ordinary()
  }
}
"#;
    let (module, source, analysis) = analyze(source_text);
    let base = ty(&analysis, &module, "Base");
    let derived = ty(&analysis, &module, "Derived");
    let run_id = zero_arg_callable(&module, "Probe", "run", DispatchSide::Class);
    let inherited_new_id = zero_arg_callable(&module, "Base", "new", DispatchSide::Class);
    let ordinary_id = zero_arg_callable(&module, "Base", "ordinary", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);

    assert_method_call_evidence(run, expression(run, &source, "Derived.new()"), derived);
    assert_method_call_evidence(run, expression(run, &source, "Derived.ordinary()"), base);
    assert_eq!(binding(run, "constructed").current.ty(), Some(derived));
    assert_eq!(binding(run, "ordinary").current.ty(), Some(base));
    assert!(run.dependencies.contains(&inherited_new_id));
    assert!(run.dependencies.contains(&ordinary_id));
    assert!(!analysis.snapshot.has_errors(), "{:#?}", analysis.snapshot.diagnostics);
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
    assert_eq!(literal.knowledge.authority(), Some(EvidenceAuthority::ExactSyntax));
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
            &TypeKnowledge::known(string_ty, EvidenceAuthority::Declared),
        ),
        Assignability::Refuted { .. }
    ));

    let x = binding(run, "x");
    assert_eq!(x.declared, Some(string_ty));
    assert_eq!(
        x.current.ty(),
        Some(int_ty),
        "exact checker proof must outrank contradictory developer annotation"
    );
    assert_eq!(x.current.authority(), Some(EvidenceAuthority::ExactSyntax));
    assert_eq!(diagnostics(&analysis, DiagnosticCode::BindingInitializerMismatch).len(), 1);
}
