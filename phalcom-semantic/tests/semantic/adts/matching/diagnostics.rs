use std::sync::Arc;

use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::diagnostic::DiagnosticCode;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

#[test]
fn reports_match_pattern_unresolved_for_nonexistent_variant() {
    let source = r#"
enum Option<T> {
    @variant Some(_ value: T) -> Option<T>
    @variant None -> Option<T>
}

class Test {
    check(_ opt: Option<Int>) {
        match opt {
            Option::NonExistent => 1
            _ => 2
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed));

    let diags = analysis.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::MatchPatternUnresolved),
        "reports MatchPatternUnresolved"
    );
}

#[test]
fn reports_match_pattern_arity_mismatch() {
    let source = r#"
enum Option<T> {
    @variant Some(_ value: T) -> Option<T>
    @variant None -> Option<T>
}

class Test {
    check(_ opt: Option<Int>) {
        match opt {
            Option::Some(a, b) => 1
            _ => 2
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed));

    let diags = analysis.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::MatchPatternArityMismatch),
        "reports MatchPatternArityMismatch"
    );
}

#[test]
fn reports_match_pattern_field_mismatch_for_invalid_label() {
    let source = r#"
enum Tree<T> {
    @variant Leaf(_ value: T) -> Tree<T>
    @variant Node(_ left: Tree<T>, right: Tree<T>) -> Tree<T>
}

class Test {
    check(_ t: Tree<Int>) {
        match t {
            Tree::Leaf(v) => 1
            Tree::Node(left: l, wrongLabel: r) => 2
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed));

    let diags = analysis.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::MatchPatternFieldMismatch),
        "reports MatchPatternFieldMismatch"
    );
}

#[test]
fn reports_match_pattern_contradictory_for_wrong_nominal_enum() {
    let source = r#"
enum Option<T> {
    @variant Some(_ value: T) -> Option<T>
    @variant None -> Option<T>
}

enum Tree<T> {
    @variant Leaf(_ value: T) -> Tree<T>
}

class Test {
    check(_ opt: Option<Int>) {
        match opt {
            Tree::Leaf(x) => 1
            _ => 2
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed));

    let diags = analysis.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::MatchPatternContradictory),
        "reports MatchPatternContradictory"
    );
}

#[test]
fn reports_match_pattern_contradictory_for_refuted_gadt_arm() {
    let source = r#"
enum Expr<T> {
    @variant LitInt(_ value: Int) -> Expr<Int>
    @variant LitBool(_ value: Bool) -> Expr<Bool>
}

class Test {
    evalInt(_ e: Expr<Int>) {
        match e {
            Expr::LitInt(n) => n
            Expr::LitBool(b) => 0
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed));

    let diags = analysis.snapshot.diagnostics.get(&module).expect("diagnostics for module");
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::MatchPatternContradictory),
        "reports MatchPatternContradictory for refuted GADT variant"
    );
}

#[test]
fn duplicate_binding_has_its_own_machine_diagnostic() {
    let case = super::super::support::analyze_adt(
        r#"
enum Pair {
    @variant Both(_ left: Int, right: Int) -> Pair
}

class Test {
    inspect(_ value: Pair) {
        match value {
            Pair::Both(x, right: x) => x
        }
    }
}
"#,
    );
    assert_eq!(case.diagnostics_for(DiagnosticCode::MatchPatternDuplicateBinding).len(), 1);
    assert_eq!(
        case.diagnostic(DiagnosticCode::MatchPatternDuplicateBinding).code,
        DiagnosticCode::MatchPatternDuplicateBinding
    );
}

#[test]
#[ignore = "GATED: ambiguous contextual owner fixture requires union-domain support"]
fn match_diag_02_ambiguous_variant_has_owner_candidates() {
    let case = super::super::support::analyze_adt("enum Left { @variant Same }\nenum Right { @variant Same }\nclass Test { run(_ value: Object) { match value { Same => 1 _ => 0 } } }\n");
    assert!(case.diagnostics().any(|diagnostic| diagnostic.code == DiagnosticCode::MatchPatternUnresolved));
}

#[test]
#[ignore = "GATED: cross-module visibility fixture is required"]
fn match_diag_03_inaccessible_variant_points_at_explicit_name() {
    let case = super::super::support::analyze_adt("enum Choice { @variant Ready }\nclass Test { run(_ value: Choice) { match value { Other::Ready => 1 _ => 0 } } }\n");
    assert!(!case.diagnostics().collect::<Vec<_>>().is_empty());
}

#[test]
#[ignore = "GATED: private payload fixture is required"]
fn match_diag_04_inaccessible_payload_points_at_projection() {
    let case = super::super::support::analyze_adt("enum Choice { @variant Ready(_ value: Int) }\nclass Test { run(_ value: Choice) { match value { Choice::Ready() => 1 _ => 0 } } }\n");
    assert!(case.diagnostics().any(|diagnostic| diagnostic.code == DiagnosticCode::MatchPatternArityMismatch));
}

#[test]
#[ignore = "GATED: selector constraint validator fixture is required"]
fn match_diag_05_invalid_selector_is_machine_diagnostic() {
    let case = super::super::support::analyze_adt("enum Choice { @variant Ready(_ value: Int) }\nclass Test { run(_ value: Choice) { match value { Choice::Ready(x, y) => 1 _ => 0 } } }\n");
    assert!(case.diagnostics().any(|diagnostic| diagnostic.code == DiagnosticCode::MatchPatternArityMismatch));
}

#[test]
fn match_diag_06_selector_without_candidate_is_not_silently_widened() {
    let case = super::super::support::analyze_adt(
        "enum Animal { @variant Dog(_ name: String) @variant Cat }\nclass Test { run(_ value: Animal) { match value { Animal::Dog(named: x) => 1 _ => 0 } } }\n",
    );
    assert!(case.diagnostics().any(|diagnostic| {
        matches!(diagnostic.code, DiagnosticCode::MatchPatternFieldMismatch | DiagnosticCode::MatchPatternUnresolved | DiagnosticCode::MatchPatternContradictory)
    }));
}

#[test]
fn match_diag_07_shape_mismatch_points_at_exact_pattern() {
    let case = super::super::support::analyze_adt(
        "enum Animal { @variant Dog(_ name: String) @variant Cat }\nclass Test { run(_ value: Animal) { match value { Animal::Dog(x, y) => 1 _ => 0 } } }\n",
    );
    assert_eq!(case.diagnostics_for(DiagnosticCode::MatchPatternArityMismatch).len(), 1);
}

#[test]
fn match_diag_08_label_mismatch_is_not_routed_to_nearest_field() {
    let case = super::super::support::analyze_adt(
        "enum Animal { @variant Dog(named name: String) @variant Cat }\nclass Test { run(_ value: Animal) { match value { Animal::Dog(other: x) => 1 _ => 0 } } }\n",
    );
    assert_eq!(case.diagnostics_for(DiagnosticCode::MatchPatternFieldMismatch).len(), 1);
}

#[test]
fn match_diag_10_or_binding_mismatch_is_precise_machine_code() {
    let case = super::super::support::analyze_adt(
        "enum Either { @variant Left(_ value: Int) @variant Right(_ value: String) }\nclass Test { run(_ value: Either) { match value { Either::Left(x) | Either::Right(y) => 1 _ => 0 } } }\n",
    );
    assert_eq!(case.diagnostics_for(DiagnosticCode::MatchPatternOrBindingMismatch).len(), 1);
}

#[test]
#[ignore = "RED: alternative usefulness diagnostic is not yet published"]
fn match_diag_11_or_redundant_alternative_has_its_own_code() {
    let case = super::super::support::analyze_adt("enum Choice { @variant Left @variant Right }\nclass Test { run(_ value: Choice) { match value { Choice::Left | Choice::Left => 1 _ => 0 } } }\n");
    assert!(!case.diagnostics().collect::<Vec<_>>().is_empty());
}

#[test]
fn match_diag_12_impossible_gadt_pattern_explains_contradiction_code() {
    let case = super::super::support::analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Test { run(_ value: Expr<Int>) { match value { Expr::Bool(x) => x Expr::Int(x) => x } } }\n",
    );
    assert!(case.diagnostics().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::MatchPatternContradictory || diagnostic.code == DiagnosticCode::MatchPatternImpossible
    }));
}

#[test]
fn match_diag_13_redundant_arm_has_structured_primary_range() {
    let case = super::super::support::analyze_adt(
        "enum Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { _ => 1 Choice::A => 2 } } }\n",
    );
    assert_eq!(case.diagnostics_for(DiagnosticCode::MatchArmRedundant).len(), 1);
}

#[test]
fn match_diag_14_non_exhaustive_has_witness_and_machine_code() {
    let case = super::super::support::analyze_adt(
        "enum Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 } } }\n",
    );
    case.assert_diagnostic_primary_contains(DiagnosticCode::MatchNonExhaustive, "match");
    assert!(matches!(case.only_match().resolution().exhaustiveness, phalcom_semantic::match_semantics::ExhaustivenessResult::Missing(_)));
}

#[test]
#[ignore = "GATED: blocked compatibility boundary fixture is required"]
fn match_diag_15_blocked_analysis_is_not_reported_as_non_exhaustive_proof() {
    let case = super::super::support::analyze_adt("class Test { run(_ value: MissingType) { match value { _ => 0 } } }\n");
    assert!(!case.diagnostics_for(DiagnosticCode::MatchNonExhaustive).iter().any(|_| true));
}

#[test]
fn review_m4_01_missing_singleton_presentation_keeps_machine_code_and_source_range() {
    let case = super::super::support::analyze_adt("enum Option { @variant Some @variant None }\nclass Test { run(_ value: Option) { match value { Option::Some => 1 } } }\n");
    let diagnostic = case.diagnostic(DiagnosticCode::MatchNonExhaustive);
    let presented = phalcom_semantic::DiagnosticPresenter::new(&case.analysis.snapshot).present(diagnostic, phalcom_semantic::DiagnosticDetail::Explain);
    assert_eq!(presented.code, DiagnosticCode::MatchNonExhaustive);
    assert!(presented.primary.range.start <= presented.primary.range.end);
}

#[test]
fn review_m4_02_missing_payload_presentation_has_structured_witness() {
    let case = super::super::support::analyze_adt("enum Option { @variant Some(_ value: Int) @variant None }\nclass Test { run(_ value: Option) { match value { Option::None => 0 } } }\n");
    let handle = case.only_match();
    let phalcom_semantic::match_semantics::ExhaustivenessResult::Missing(witnesses) = &handle.resolution().exhaustiveness else { panic!("expected witness") };
    assert!(witnesses.iter().any(|witness| matches!(witness, phalcom_semantic::match_semantics::CoverageWitness::Variant { fields, .. } if fields.len() == 1)));
}

#[test]
#[ignore = "GATED: external-label witness renderer is not yet source-owned"]
fn review_m4_03_missing_labeled_payload_presentation_uses_external_label() {
    let case = super::super::support::analyze_adt("enum Option { @variant Some(named value: Int) @variant None }\nclass Test { run(_ value: Option) { match value { Option::None => 0 } } }\n");
    let diagnostic = case.diagnostic(DiagnosticCode::MatchNonExhaustive);
    let presented = phalcom_semantic::DiagnosticPresenter::new(&case.analysis.snapshot).present(diagnostic, phalcom_semantic::DiagnosticDetail::Explain);
    assert_eq!(presented.code, DiagnosticCode::MatchNonExhaustive);
}

#[test]
#[ignore = "RED: singleton/nullary witness rendering needs final presentation contract"]
fn review_m4_04_missing_singleton_and_nullary_render_differently() {
    let case = super::super::support::analyze_adt("enum Option { @variant Some @variant Some() }\nclass Test { run(_ value: Option) { match value { Option::Some => 0 } } }\n");
    let diagnostic = case.diagnostic(DiagnosticCode::MatchNonExhaustive);
    let presented = phalcom_semantic::DiagnosticPresenter::new(&case.analysis.snapshot).present(diagnostic, phalcom_semantic::DiagnosticDetail::Explain);
    assert_eq!(presented.code, DiagnosticCode::MatchNonExhaustive);
}

#[test]
fn review_m4_05_diagnostic_presentation_has_no_debug_format_leakage() {
    let case = super::super::support::analyze_adt("enum Option { @variant Some @variant None }\nclass Test { run(_ value: Option) { match value { Option::Some => 1 } } }\n");
    let diagnostic = case.diagnostic(DiagnosticCode::MatchNonExhaustive);
    let presented = phalcom_semantic::DiagnosticPresenter::new(&case.analysis.snapshot).present(diagnostic, phalcom_semantic::DiagnosticDetail::Explain);
    let text = format!("{} {:?} {:?}", presented.headline, presented.explanation, presented.guidance);
    for forbidden in ["VariantId(", "TypeId(", "CoverageWitness {"] {
        assert!(!text.contains(forbidden), "presentation leaked internal fragment {forbidden:?}: {text}");
    }
}
