use phalcom_ast::parse;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::analysis::{CallableAnalysis, ExpressionAnalysis};
use phalcom_semantic::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use phalcom_semantic::explain::{EvidenceRef, ExplanationStep};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::types::evidence::{EvidenceAuthority, TypeKnowledge};
use phalcom_semantic::{Assignability, TypeId, analyze_single_module, check_assignability, is_subtype};
use std::sync::Arc;

fn analyze(source_text: &str) -> (ModuleId, Arc<str>, phalcom_semantic::workspace::SemanticAnalysis) {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(source_text);
    let parsed = parse(&source, 0);
    assert!(parsed.diagnostics.is_empty(), "parse diagnostics: {:?}", parsed.diagnostics);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    (module, source, analysis)
}

fn declaration(module: &ModuleId, name: &str) -> DeclarationId {
    DeclarationId::new(module.clone(), name.into())
}

fn method_callable(module: &ModuleId, owner: &str, name: &str, side: DispatchSide) -> CallableId {
    CallableId::new(
        declaration(module, owner),
        Selector::method(name, vec![]).unwrap(),
        side,
    )
}

fn callable_analysis<'a>(
    analysis: &'a phalcom_semantic::workspace::SemanticAnalysis,
    callable: &CallableId,
) -> &'a CallableAnalysis {
    analysis
        .snapshot
        .callable_analyses
        .get(callable)
        .unwrap_or_else(|| panic!("missing callable analysis for {callable:?}"))
}

fn binding<'a>(analysis: &'a CallableAnalysis, name: &str) -> &'a phalcom_semantic::checker::analysis::BindingState {
    analysis
        .bindings
        .values()
        .find(|binding| binding.name == name)
        .unwrap_or_else(|| panic!("missing binding `{name}`; bindings={:#?}", analysis.bindings))
}

fn expression_with_text<'a>(analysis: &'a CallableAnalysis, source: &str, text: &str) -> &'a ExpressionAnalysis {
    let matches = analysis
        .expressions
        .values()
        .filter(|expr| source.get(expr.range.start..expr.range.end) == Some(text))
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "expected one expression `{text}` in {:?}, found {}: {:#?}",
        analysis.callable,
        matches.len(),
        matches
    );
    matches[0]
}

fn diagnostics<'a>(
    analysis: &'a phalcom_semantic::workspace::SemanticAnalysis,
    code: DiagnosticCode,
) -> Vec<&'a SemanticDiagnostic> {
    analysis
        .snapshot
        .all_diagnostics()
        .filter(|diagnostic| diagnostic.code == code)
        .collect()
}

fn assert_method_call_evidence(analysis: &CallableAnalysis, expression: &ExpressionAnalysis, expected_type: TypeId) {
    assert_eq!(expression.knowledge.ty(), Some(expected_type));
    let explanation_id = expression
        .explanation
        .expect("known method-call expression must retain an explanation");
    let node = analysis
        .explanations
        .get(explanation_id)
        .unwrap_or_else(|| panic!("missing explanation node {explanation_id:?}"));

    match &node.step {
        ExplanationStep::MethodCall { return_ty, .. } => assert_eq!(*return_ty, expected_type),
        other => panic!("expected method-call derivation, got {other:?}"),
    }
    assert_eq!(node.authority, EvidenceAuthority::Proven);
    assert!(
        node.evidence
            .iter()
            .any(|evidence| matches!(evidence, EvidenceRef::SourceSpan(range) if *range == expression.range)),
        "method-call explanation must retain source evidence: {node:#?}"
    );
    assert!(
        node.evidence
            .iter()
            .any(|evidence| matches!(evidence, EvidenceRef::TypeId(ty) if *ty == expected_type)),
        "method-call explanation must retain resulting type evidence: {node:#?}"
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
    let cell_num = analysis.snapshot.declarations.form(&declaration(&module, "CellNum")).unwrap();
    let run_id = method_callable(&module, "Probe", "run", DispatchSide::Class);
    let new_id = method_callable(&module, "CellNum", "new", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);

    let call = expression_with_text(run, &source, "CellNum.new()");
    assert_method_call_evidence(run, call, cell_num);
    assert_eq!(binding(run, "x").current.ty(), Some(cell_num));
    assert_eq!(binding(run, "x").current.authority(), Some(EvidenceAuthority::Proven));
    assert!(run.dependencies.contains(&new_id), "constructor dispatch path missing: {:?}", run.dependencies);
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
    let cell_num = analysis.snapshot.declarations.form(&declaration(&module, "CellNum")).unwrap();
    let of_id = method_callable(&module, "CellNum", "of", DispatchSide::Class);
    let new_id = method_callable(&module, "CellNum", "new", DispatchSide::Class);
    let run_id = method_callable(&module, "Probe", "run", DispatchSide::Class);

    let factory = callable_analysis(&analysis, &of_id);
    let factory_tail = expression_with_text(factory, &source, "CellNum.new()");
    assert_method_call_evidence(factory, factory_tail, cell_num);
    assert!(factory.dependencies.contains(&new_id), "factory must depend on constructor: {:?}", factory.dependencies);

    let run = callable_analysis(&analysis, &run_id);
    let callsite = expression_with_text(run, &source, "CellNum.of()");
    assert_method_call_evidence(run, callsite, cell_num);
    assert!(run.dependencies.contains(&of_id), "callsite must depend on factory: {:?}", run.dependencies);
    assert_eq!(binding(run, "x").current.ty(), Some(cell_num));
}

#[test]
fn refuted_annotation_cannot_override_proven_constructor_type_or_downstream_dispatch() {
    let source_text = r#"
class CellNum {
  @constructor
  new() {}

  cellOnly() -> Int {
    1
  }
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
    let cell_num = analysis.snapshot.declarations.form(&declaration(&module, "CellNum")).unwrap();
    let int_ty = analysis.snapshot.declarations.form(&declaration(&module, "Int")).unwrap();
    let run_id = method_callable(&module, "Probe", "run", DispatchSide::Class);
    let new_id = method_callable(&module, "CellNum", "new", DispatchSide::Class);
    let cell_only_id = method_callable(&module, "CellNum", "cellOnly", DispatchSide::Instance);
    let run = callable_analysis(&analysis, &run_id);

    let initializer = expression_with_text(run, &source, "CellNum.new()");
    assert_method_call_evidence(run, initializer, cell_num);
    let x = binding(run, "x");
    assert_eq!(x.declared, Some(int_ty), "developer annotation must remain a declared constraint");
    assert_eq!(x.current.ty(), Some(cell_num), "refuted annotation must not replace proven value knowledge");
    assert_eq!(x.current.authority(), Some(EvidenceAuthority::Proven));

    let mismatch = diagnostics(&analysis, DiagnosticCode::BindingInitializerMismatch);
    assert_eq!(mismatch.len(), 1, "expected exactly one annotation contradiction: {mismatch:#?}");
    assert!(mismatch[0].labels.iter().any(|label| label.message == "declared type"));
    assert!(mismatch[0].labels.iter().any(|label| label.message == "inferred type"));

    let downstream = expression_with_text(run, &source, "x.cellOnly()");
    assert_method_call_evidence(run, downstream, int_ty);
    assert_eq!(binding(run, "y").current.ty(), Some(int_ty));
    assert!(run.dependencies.contains(&new_id));
    assert!(run.dependencies.contains(&cell_only_id), "downstream dispatch must use proven CellNum receiver");
}

#[test]
fn compatible_supertype_annotation_is_proven_but_narrow_value_knowledge_is_preserved() {
    let source_text = r#"
class Base {}

class Derived is Base {
  @constructor
  new() {}

  derivedOnly() -> Int {
    1
  }
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
    let base = analysis.snapshot.declarations.form(&declaration(&module, "Base")).unwrap();
    let derived = analysis.snapshot.declarations.form(&declaration(&module, "Derived")).unwrap();
    let int_ty = analysis.snapshot.declarations.form(&declaration(&module, "Int")).unwrap();
    assert!(is_subtype(&analysis.snapshot.store, &analysis.snapshot.hierarchy, derived, base));
    assert_eq!(
        check_assignability(
            &analysis.snapshot.store,
            &analysis.snapshot.hierarchy,
            &TypeKnowledge::known(derived, EvidenceAuthority::Proven),
            &TypeKnowledge::known(base, EvidenceAuthority::Declared),
        ),
        Assignability::Assignable
    );

    let run_id = method_callable(&module, "Probe", "run", DispatchSide::Class);
    let derived_only_id = method_callable(&module, "Derived", "derivedOnly", DispatchSide::Instance);
    let run = callable_analysis(&analysis, &run_id);
    assert_method_call_evidence(run, expression_with_text(run, &source, "Derived.new()"), derived);

    let x = binding(run, "x");
    assert_eq!(x.declared, Some(base));
    assert_eq!(x.current.ty(), Some(derived), "successful supertype constraint must not erase stronger proven knowledge");
    assert_eq!(x.current.authority(), Some(EvidenceAuthority::Proven));
    assert!(diagnostics(&analysis, DiagnosticCode::BindingInitializerMismatch).is_empty());

    assert_method_call_evidence(run, expression_with_text(run, &source, "x.derivedOnly()"), int_ty);
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
    let int_ty = analysis.snapshot.declarations.form(&declaration(&module, "Int")).unwrap();
    let run_id = CallableId::new(
        declaration(&module, "Probe"),
        Selector::method("run", vec![phalcom_common::selector::SelectorSlot::Positional]).unwrap(),
        DispatchSide::Class,
    );
    let run = callable_analysis(&analysis, &run_id);

    let parameter_use = expression_with_text(run, &source, "value");
    assert!(parameter_use.knowledge.is_unknown(), "unannotated parameter should be unknown evidence: {parameter_use:#?}");

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
    let cell_num = analysis.snapshot.declarations.form(&declaration(&module, "CellNum")).unwrap();
    let string_ty = analysis.snapshot.declarations.form(&declaration(&module, "String")).unwrap();
    let run_id = method_callable(&module, "Probe", "run", DispatchSide::Class);
    let from_int_id = CallableId::new(
        declaration(&module, "CellNum"),
        Selector::method("fromInt", vec![phalcom_common::selector::SelectorSlot::Positional]).unwrap(),
        DispatchSide::Class,
    );
    let run = callable_analysis(&analysis, &run_id);

    let argument = expression_with_text(run, &source, "\"bad\"");
    assert_eq!(argument.knowledge.ty(), Some(string_ty));
    assert_eq!(argument.knowledge.authority(), Some(EvidenceAuthority::ExactSyntax));

    let call = expression_with_text(run, &source, "CellNum.fromInt(\"bad\")");
    assert_method_call_evidence(run, call, cell_num);
    assert_eq!(binding(run, "x").current.ty(), Some(cell_num));
    assert!(run.dependencies.contains(&from_int_id));

    let argument_mismatches = diagnostics(&analysis, DiagnosticCode::ArgumentMismatch);
    assert_eq!(argument_mismatches.len(), 1, "argument contradiction must be diagnosed without losing return fact: {argument_mismatches:#?}");
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
    let cell_num = analysis.snapshot.declarations.form(&declaration(&module, "CellNum")).unwrap();
    let int_ty = analysis.snapshot.declarations.form(&declaration(&module, "Int")).unwrap();
    let make_id = method_callable(&module, "Factory", "make", DispatchSide::Class);
    let new_id = method_callable(&module, "CellNum", "new", DispatchSide::Class);
    let make = callable_analysis(&analysis, &make_id);

    let tail = expression_with_text(make, &source, "CellNum.new()");
    assert_method_call_evidence(make, tail, cell_num);
    assert!(make.dependencies.contains(&new_id));

    let return_mismatches = diagnostics(&analysis, DiagnosticCode::ReturnMismatch);
    assert_eq!(return_mismatches.len(), 1, "return contradiction must be retained: {return_mismatches:#?}");

    let surface = analysis.snapshot.surfaces.get(&declaration(&module, "Factory")).unwrap();
    let signature = surface
        .get_callable(DispatchSide::Class, &Selector::method("make", vec![]).unwrap())
        .unwrap();
    assert_eq!(signature.return_type.ty(), Some(int_ty), "declared contract remains Int while body proof remains CellNum");
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
    let derived = analysis.snapshot.declarations.form(&declaration(&module, "Derived")).unwrap();
    let string_ty = analysis.snapshot.declarations.form(&declaration(&module, "String")).unwrap();
    let run_id = method_callable(&module, "Probe", "run", DispatchSide::Class);
    let inherited_new_id = method_callable(&module, "Base", "new", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);

    let constructor_call = expression_with_text(run, &source, "Derived.new()");
    assert_method_call_evidence(run, constructor_call, derived);
    assert!(run.dependencies.contains(&inherited_new_id), "dispatch evidence must point through inherited Base constructor");

    let x = binding(run, "x");
    assert_eq!(x.declared, Some(string_ty));
    assert_eq!(x.current.ty(), Some(derived), "specialized Self proof must outrank contradictory annotation");

    assert!(matches!(
        check_assignability(
            &analysis.snapshot.store,
            &analysis.snapshot.hierarchy,
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
    let base = analysis.snapshot.declarations.form(&declaration(&module, "Base")).unwrap();
    let derived = analysis.snapshot.declarations.form(&declaration(&module, "Derived")).unwrap();
    let run_id = method_callable(&module, "Probe", "run", DispatchSide::Class);
    let inherited_new_id = method_callable(&module, "Base", "new", DispatchSide::Class);
    let ordinary_id = method_callable(&module, "Base", "ordinary", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);

    assert_method_call_evidence(run, expression_with_text(run, &source, "Derived.new()"), derived);
    assert_method_call_evidence(run, expression_with_text(run, &source, "Derived.ordinary()"), base);
    assert_eq!(binding(run, "constructed").current.ty(), Some(derived));
    assert_eq!(binding(run, "ordinary").current.ty(), Some(base));
    assert!(run.dependencies.contains(&inherited_new_id));
    assert!(run.dependencies.contains(&ordinary_id));
    assert!(!analysis.snapshot.has_errors(), "unexpected diagnostics: {:#?}", analysis.snapshot.diagnostics);
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
    let int_ty = analysis.snapshot.declarations.form(&declaration(&module, "Int")).unwrap();
    let string_ty = analysis.snapshot.declarations.form(&declaration(&module, "String")).unwrap();
    let run_id = method_callable(&module, "Probe", "run", DispatchSide::Class);
    let run = callable_analysis(&analysis, &run_id);

    let literal = expression_with_text(run, &source, "42");
    assert_eq!(literal.knowledge.ty(), Some(int_ty));
    assert_eq!(literal.knowledge.authority(), Some(EvidenceAuthority::ExactSyntax));
    let explanation_id = literal.explanation.expect("literal must have explanation");
    let explanation = run.explanations.get(explanation_id).expect("literal explanation node");
    assert!(matches!(explanation.step, ExplanationStep::Literal { ty, .. } if ty == int_ty));

    assert!(matches!(
        check_assignability(
            &analysis.snapshot.store,
            &analysis.snapshot.hierarchy,
            &literal.knowledge,
            &TypeKnowledge::known(string_ty, EvidenceAuthority::Declared),
        ),
        Assignability::Refuted { .. }
    ));

    let x = binding(run, "x");
    assert_eq!(x.declared, Some(string_ty));
    assert_eq!(x.current.ty(), Some(int_ty), "exact syntax proof must outrank refuted developer annotation");
    assert_eq!(x.current.authority(), Some(EvidenceAuthority::ExactSyntax));
    assert_eq!(diagnostics(&analysis, DiagnosticCode::BindingInitializerMismatch).len(), 1);
}
