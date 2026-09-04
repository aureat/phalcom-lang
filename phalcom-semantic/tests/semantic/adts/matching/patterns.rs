use std::sync::Arc;

use super::super::support::analyze_adt;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::match_semantics::PatternResolution;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

#[test]
fn resolve_exact_and_labeled_variant_patterns() {
    let source = r#"
enum Result<T, E> {
    @variant Ok(_ value: T) -> Result<T, E>
    @variant Error(_ code: Int, reason: E) -> Result<T, E>
}

class Test {
    check(_ res: Result<Int, String>) {
        match res {
            Result::Ok(x) => x
            Result::Error(_, reason: msg) => 0
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module, Arc::from(source), Arc::new(parsed));

    assert!(analysis.snapshot.diagnostics.values().all(|d| d.is_empty()), "no semantic errors");
}

#[test]
fn resolve_family_and_gap_patterns() {
    let source = r#"
enum Animal {
    @variant Dog(_ name: String) -> Animal
    @variant Dog(_ name: String, age: Int) -> Animal
    @variant Dog(_ name: String, age: Int, breed: String) -> Animal
    @variant Cat -> Animal
}

class Test {
    check(_ a: Animal) {
        match a {
            Animal::Dog(name, ..., breed: b) => 1
            Animal::Dog* => 2
            Animal::Cat => 3
        }
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module, Arc::from(source), Arc::new(parsed));
    assert!(analysis.snapshot.diagnostics.values().all(|d| d.is_empty()), "no semantic errors");
}

#[test]
fn match_pat_01_wildcard_payload_publishes_no_binding() {
    let case = analyze_adt(
        "enum Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\nclass Test { run(_ value: Option<Int>) { match value { Option::Some(_) => 1 _ => 0 } } }\n",
    );
    assert!(case.only_match().arm(0).resolution().bindings.is_empty());
}

#[test]
fn match_pat_02_nested_adt_resolution_is_recursive() {
    let case = analyze_adt(
        "enum Result<T, E> { @variant Ok(_ value: T) -> Result<T, E> @variant Error(_ value: E) -> Result<T, E> }\nenum Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\nclass Test { run(_ value: Option<Result<Int, String>>) { match value { Some(Ok(x)) => x _ => 0 } } }\n",
    );
    let handle = case.only_match();
    let arm = handle.arm(0);
    let phalcom_semantic::match_semantics::PatternResolution::Variant(parent) = &arm.resolution().pattern else {
        panic!("expected outer variant pattern");
    };
    assert!(matches!(
        parent.candidates[0].fields[0].child.as_ref(),
        phalcom_semantic::match_semantics::PatternResolution::Variant(_)
    ));
}

#[test]
fn match_pat_03_tuple_pattern_is_published_as_recursive_product() {
    let case = analyze_adt(
        "enum Color { @variant Red @variant Green }\nclass Test { run(_ value: (Color, Color)) { match value { (Color::Red, Color::Green) => 1 _ => 0 } } }\n",
    );
    let handle = case.only_match();
    assert!(matches!(
        handle.arm(0).resolution().pattern,
        phalcom_semantic::match_semantics::PatternResolution::Tuple(_)
    ));
}

#[test]
fn match_pat_04_adt_payload_tuple_keeps_child_pattern() {
    let case = analyze_adt(
        "enum Pair { @variant Pair(_ values: (Int, String)) }\nclass Test { run(_ value: Pair) { match value { Pair::Pair((x, y)) => x _ => 0 } } }\n",
    );
    let handle = case.only_match();
    assert_eq!(handle.arm(0).resolution().bindings.len(), 2);
}

#[test]
fn match_pat_05_nested_or_pattern_remains_one_resolved_or_node() {
    let case = analyze_adt(
        "enum Result { @variant Ok(_ value: Int) @variant Cached(_ value: Int) @variant Error }\nenum Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\nclass Test { run(_ value: Option<Result>) { match value { Some(Ok(x) | Cached(x)) => x _ => 0 } } }\n",
    );
    let handle = case.only_match();
    let arm = handle.arm(0);
    let phalcom_semantic::match_semantics::PatternResolution::Variant(parent) = &arm.resolution().pattern else {
        panic!("expected outer variant pattern");
    };
    assert!(matches!(
        parent.candidates[0].fields[0].child.as_ref(),
        phalcom_semantic::match_semantics::PatternResolution::Or(_)
    ));
}

#[test]
fn match_pat_06_family_pattern_joins_candidate_specific_field_projections() {
    let case = analyze_adt(
        "enum Animal { @variant Dog(_ name: String) @variant Dog(_ name: String, age: Int) }\nclass Test { run(_ value: Animal) { match value { Dog(name, ...) => name _ => \"unknown\" } } }\n",
    );
    let handle = case.only_match();
    let arm = handle.arm(0);
    let pattern = &arm.resolution().pattern;
    assert!(matches!(pattern, phalcom_semantic::match_semantics::PatternResolution::Variant(_)));
}

#[test]
fn match_pat_07_list_space_distinguishes_empty_and_non_empty_prefixes() {
    use phalcom_semantic::checker::{ListSpace, PatternSpace};
    let empty = PatternSpace::List(ListSpace {
        prefix: Box::new([]),
        rest: None,
    });
    let non_empty = PatternSpace::List(ListSpace {
        prefix: Box::new([PatternSpace::Opaque(phalcom_semantic::TypeId(1))]),
        rest: None,
    });
    assert_ne!(empty, non_empty);
    assert!(!empty.is_empty());
    assert!(!non_empty.is_empty());
}

#[test]
fn match_pat_08_r1_t03_selector_gap_suffix_position() {
    use phalcom_semantic::match_semantics::PatternResolution;
    let case = analyze_adt(
        r#"
enum Left { @variant A }
enum Middle { @variant M }
enum Right { @variant B @variant Other }

enum E {
    @variant V(_ first: Left, _ middle: Middle, last: Right)
}

class Test {
    run(_ value: E) {
        match value {
            E::V(Left::A, ..., last: Right::B) => 1
            E::V(Left::A, ..., last: Right::Other) => 2
            _ => 3
        }
    }
}
"#,
    );
    let handle = case.only_match();
    let arm0 = handle.arm(0);
    let PatternResolution::Variant(pat0) = &arm0.resolution().pattern else {
        panic!("expected variant pattern")
    };
    let cand0 = pat0.candidates.first().expect("cand0");
    // Check that field identities correspond to index 0 and index 2
    assert_eq!(cand0.fields.len(), 2);
    assert_eq!(cand0.fields[0].field.index, 0);
    assert_eq!(cand0.fields[1].field.index, 2);

    arm0.assert_usefulness(phalcom_semantic::match_semantics::PatternUsefulness::Useful);
    handle.arm(1).assert_usefulness(phalcom_semantic::match_semantics::PatternUsefulness::Useful);
    handle.arm(2).assert_usefulness(phalcom_semantic::match_semantics::PatternUsefulness::Redundant);
}

#[test]
fn match_pat_09_r1_t04_candidate_specific_family_layouts() {
    let case = analyze_adt(
        r#"
enum Animal {
    @variant Dog(_ name: String)
    @variant Dog(_ name: String, _ breed: String, age: Int)
}

class Test {
    run(_ value: Animal) {
        match value {
            Dog(name, ...) => name
            _ => "unknown"
        }
    }
}
"#,
    );
    let handle = case.only_match();
    let arm0 = handle.arm(0);
    let PatternResolution::Variant(pat0) = &arm0.resolution().pattern else {
        panic!("expected variant pattern")
    };
    assert_eq!(pat0.candidates.len(), 2, "family pattern should resolve 2 candidates");
    arm0.assert_usefulness(phalcom_semantic::match_semantics::PatternUsefulness::Useful);
    handle.arm(1).assert_usefulness(phalcom_semantic::match_semantics::PatternUsefulness::Redundant);
}

#[test]
#[ignore = "GATED: record pattern resolver must be made explicit before enablement"]
fn review_c4_01_record_pattern_is_not_silently_converted_to_wildcard() {
    let case = analyze_adt("class Test { run(_ value: Object) { match value { #{name: value} => value _ => 0 } } }\n");
    assert!(!matches!(
        case.only_match().arm(0).resolution().pattern,
        phalcom_semantic::match_semantics::PatternResolution::Wildcard
    ));
}

#[test]
#[ignore = "GATED: map pattern resolver must be made explicit before enablement"]
fn review_c4_02_map_pattern_is_not_silently_converted_to_wildcard() {
    let case = analyze_adt("class Test { run(_ value: Object) { match value { {#name: value} => value _ => 0 } } }\n");
    assert!(!matches!(
        case.only_match().arm(0).resolution().pattern,
        phalcom_semantic::match_semantics::PatternResolution::Wildcard
    ));
}

#[test]
#[ignore = "GATED: unsupported record plus enum fixture is required"]
fn review_c4_03_unsupported_record_does_not_make_match_exhaustive() {
    let case = analyze_adt("class Test { run(_ value: Object) { match value { #{name: value} => value } } }\n");
    case.only_match().assert_not_exhaustive();
}

#[test]
#[ignore = "GATED: unsupported map plus enum fixture is required"]
fn review_c4_04_unsupported_map_does_not_make_later_arm_redundant() {
    let case = analyze_adt("class Test { run(_ value: Object) { match value { {#name: value} => value _ => 0 } } }\n");
    assert_ne!(
        case.only_match().arm(1).resolution().usefulness,
        phalcom_semantic::match_semantics::PatternUsefulness::Redundant
    );
}

#[test]
#[ignore = "GATED: resolver fallback audit needs explicit unsupported-pattern product"]
fn review_c4_05_resolver_has_no_catch_all_wildcard_fallback() {
    let case = analyze_adt("class Test { run(_ value: Object) { match value { #{name: value} => value _ => 0 } } }\n");
    assert!(!matches!(
        case.only_match().arm(0).resolution().pattern,
        phalcom_semantic::match_semantics::PatternResolution::Wildcard
    ));
}
