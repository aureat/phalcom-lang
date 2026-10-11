use phalcom_semantic::checker::{SequenceConstructionPlan, SequenceLiteralFamily};
use phalcom_semantic::diagnostic::DiagnosticCode;

use crate::semantic::support::Fixture;

#[test]
fn test_list_hole_inference_solves_element_type() {
    let f = Fixture::new(
        r#"
class Container {
    @class test() {
        let xs: List<_> = [1, 2, 3]
    }
}
"#,
    );
    f.assert_no_error_diagnostics();
    let callable = f.callable("Container", "test", phalcom_semantic::DispatchSide::Class);
    let binding = callable
        .bindings
        .values()
        .find(|b| b.name == "xs")
        .expect("binding for xs");
    let binding_ty = binding.current.ty().expect("known binding type");

    let list_decl = f.decl("List");
    let (decl, args) = f.analysis.snapshot.store.applied_nominal_parts(binding_ty).expect("applied nominal parts");
    assert_eq!(decl, list_decl);
    assert_eq!(args.len(), 1);
    assert_eq!(args[0], f.ty("Int"));

    assert_eq!(callable.sequence_literals.len(), 1);
    let resolution = callable.sequence_literals.values().next().unwrap();
    assert_eq!(resolution.family, SequenceLiteralFamily::List);
    assert_eq!(resolution.result_type, binding_ty);
    assert_eq!(resolution.element_type, f.ty("Int"));
    assert_eq!(
        resolution.construction,
        SequenceConstructionPlan::Fixed { element_count: 3 }
    );
}

#[test]
fn test_list_hole_empty_is_underconstrained() {
    let f = Fixture::new_allowing_internal_incidents(
        r#"
class Container {
    @class test() {
        let xs: List<_> = []
    }
}
"#,
    );
    assert!(
        !f.diagnostics(DiagnosticCode::GenericInferenceUnderconstrained).is_empty(),
        "expected GenericInferenceUnderconstrained diagnostic"
    );
}

#[test]
fn test_strict_type_formation_rejects_holes_in_declarations() {
    let f = Fixture::new_allowing_internal_incidents(
        r#"
class Box<T> {}
class Target {
    _field: Box<_>
}
"#,
    );
    assert!(
        !f.diagnostics(DiagnosticCode::AnnotationUnsupported).is_empty(),
        "expected AnnotationUnsupported diagnostic for hole in declaration field"
    );
}

#[test]
fn test_spread_list_literal_publishes_expanded_construction() {
    let f = Fixture::new(
        r#"
class Container {
    @class test(rest: List<Int>) {
        let xs: List<Int> = [1, *rest, 3]
    }
}
"#,
    );
    f.assert_no_error_diagnostics();
    let callable = f.callable("Container", "test", phalcom_semantic::DispatchSide::Class);
    assert_eq!(callable.sequence_literals.len(), 1);
    let resolution = callable.sequence_literals.values().next().unwrap();
    assert_eq!(resolution.family, SequenceLiteralFamily::List);
    assert_eq!(resolution.construction, SequenceConstructionPlan::Expanded);
}

#[test]
fn test_static_constant_demand_with_hole() {
    let f = Fixture::new(
        r#"
const xs: List<_> = [10, 20, 30]
"#,
    );
    f.assert_no_error_diagnostics();
}
