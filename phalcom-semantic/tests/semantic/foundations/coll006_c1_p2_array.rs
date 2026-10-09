use phalcom_semantic::checker::{SequenceConstructionPlan, SequenceLiteralFamily};
use phalcom_semantic::diagnostic::DiagnosticCode;

use crate::semantic::support::Fixture;

#[test]
fn test_contextual_array_literal() {
    let f = Fixture::new(
        r#"
class Container {
    @class test() {
        let xs: Array<Int> = [1, 2, 3]
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

    let array_decl = f.decl("Array");
    let (decl, args) = f.analysis.snapshot.store.applied_nominal_parts(binding_ty).expect("applied nominal parts");
    assert_eq!(decl, array_decl);
    assert_eq!(args.len(), 1);
    assert_eq!(args[0], f.ty("Int"));

    assert_eq!(callable.sequence_literals.len(), 1);
    let resolution = callable.sequence_literals.values().next().unwrap();
    assert_eq!(resolution.family, SequenceLiteralFamily::Array);
    assert_eq!(resolution.result_type, binding_ty);
    assert_eq!(resolution.element_type, f.ty("Int"));
    assert_eq!(
        resolution.construction,
        SequenceConstructionPlan::Fixed { element_count: 3 }
    );
}

#[test]
fn test_contextual_array_hole_inference() {
    let f = Fixture::new(
        r#"
class Container {
    @class test() {
        let xs: Array<_> = [1, 2, 3]
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

    let array_decl = f.decl("Array");
    let (decl, args) = f.analysis.snapshot.store.applied_nominal_parts(binding_ty).expect("applied nominal parts");
    assert_eq!(decl, array_decl);
    assert_eq!(args.len(), 1);
    assert_eq!(args[0], f.ty("Int"));

    assert_eq!(callable.sequence_literals.len(), 1);
    let resolution = callable.sequence_literals.values().next().unwrap();
    assert_eq!(resolution.family, SequenceLiteralFamily::Array);
    assert_eq!(resolution.result_type, binding_ty);
    assert_eq!(resolution.element_type, f.ty("Int"));
}

#[test]
fn test_explicit_array_literal() {
    let f = Fixture::new(
        r#"
class Container {
    @class test() {
        let xs = <Int>[1, 2, 3]
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

    let array_decl = f.decl("Array");
    let (decl, args) = f.analysis.snapshot.store.applied_nominal_parts(binding_ty).expect("applied nominal parts");
    assert_eq!(decl, array_decl);
    assert_eq!(args.len(), 1);
    assert_eq!(args[0], f.ty("Int"));

    assert_eq!(callable.sequence_literals.len(), 1);
    let resolution = callable.sequence_literals.values().next().unwrap();
    assert_eq!(resolution.family, SequenceLiteralFamily::Array);
    assert_eq!(resolution.result_type, binding_ty);
    assert_eq!(resolution.element_type, f.ty("Int"));
    assert_eq!(
        resolution.construction,
        SequenceConstructionPlan::Fixed { element_count: 3 }
    );
}

#[test]
fn test_explicit_array_hole_inference() {
    let f = Fixture::new(
        r#"
class Container {
    @class test() {
        let xs = <_>[1, 2, 3]
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

    let array_decl = f.decl("Array");
    let (decl, args) = f.analysis.snapshot.store.applied_nominal_parts(binding_ty).expect("applied nominal parts");
    assert_eq!(decl, array_decl);
    assert_eq!(args.len(), 1);
    assert_eq!(args[0], f.ty("Int"));

    assert_eq!(callable.sequence_literals.len(), 1);
    let resolution = callable.sequence_literals.values().next().unwrap();
    assert_eq!(resolution.family, SequenceLiteralFamily::Array);
    assert_eq!(resolution.result_type, binding_ty);
    assert_eq!(resolution.element_type, f.ty("Int"));
}

#[test]
fn test_empty_array_contextual_typed() {
    let f = Fixture::new(
        r#"
class Container {
    @class test() {
        let xs: Array<Int> = []
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

    let array_decl = f.decl("Array");
    let (decl, args) = f.analysis.snapshot.store.applied_nominal_parts(binding_ty).expect("applied nominal parts");
    assert_eq!(decl, array_decl);
    assert_eq!(args.len(), 1);
    assert_eq!(args[0], f.ty("Int"));

    assert_eq!(callable.sequence_literals.len(), 1);
    let resolution = callable.sequence_literals.values().next().unwrap();
    assert_eq!(resolution.family, SequenceLiteralFamily::Array);
    assert_eq!(resolution.result_type, binding_ty);
    assert_eq!(resolution.element_type, f.ty("Int"));
    assert_eq!(
        resolution.construction,
        SequenceConstructionPlan::Fixed { element_count: 0 }
    );
}

#[test]
fn test_empty_array_explicit_typed() {
    let f = Fixture::new(
        r#"
class Container {
    @class test() {
        let xs = <Int>[]
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

    let array_decl = f.decl("Array");
    let (decl, args) = f.analysis.snapshot.store.applied_nominal_parts(binding_ty).expect("applied nominal parts");
    assert_eq!(decl, array_decl);
    assert_eq!(args.len(), 1);
    assert_eq!(args[0], f.ty("Int"));

    assert_eq!(callable.sequence_literals.len(), 1);
    let resolution = callable.sequence_literals.values().next().unwrap();
    assert_eq!(resolution.family, SequenceLiteralFamily::Array);
    assert_eq!(resolution.result_type, binding_ty);
    assert_eq!(resolution.element_type, f.ty("Int"));
    assert_eq!(
        resolution.construction,
        SequenceConstructionPlan::Fixed { element_count: 0 }
    );
}

#[test]
fn test_empty_array_contextual_hole_underconstrained() {
    let f = Fixture::new_allowing_internal_incidents(
        r#"
class Container {
    @class test() {
        let xs: Array<_> = []
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
fn test_empty_array_explicit_hole_underconstrained() {
    let f = Fixture::new_allowing_internal_incidents(
        r#"
class Container {
    @class test() {
        let xs = <_>[]
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
fn test_array_spread_publishes_expanded_construction() {
    let f = Fixture::new(
        r#"
class Container {
    @class test(rest: Array<Int>) {
        let xs = <Int>[1, *rest, 3]
    }
}
"#,
    );
    f.assert_no_error_diagnostics();
    let callable = f.callable("Container", "test", phalcom_semantic::DispatchSide::Class);
    assert_eq!(callable.sequence_literals.len(), 1);
    let resolution = callable.sequence_literals.values().next().unwrap();
    assert_eq!(resolution.family, SequenceLiteralFamily::Array);
    assert_eq!(resolution.construction, SequenceConstructionPlan::Expanded);
}

#[test]
fn test_contextual_list_literal_remains_available() {
    let f = Fixture::new(
        r#"
class Container {
    @class test() {
        let xs: List<Int> = [1, 2, 3]
    }
}
"#,
    );
    f.assert_no_error_diagnostics();
    let callable = f.callable("Container", "test", phalcom_semantic::DispatchSide::Class);
    assert_eq!(callable.sequence_literals.len(), 1);
    let resolution = callable.sequence_literals.values().next().unwrap();
    assert_eq!(resolution.family, SequenceLiteralFamily::List);
}
