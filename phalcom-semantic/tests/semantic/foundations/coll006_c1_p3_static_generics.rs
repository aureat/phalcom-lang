use phalcom_semantic::types::kind::KindData;

use crate::semantic::support::Fixture;

#[test]
fn static_generic_argument_is_a_canonical_static_integer_term() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
class Sized<T: Type, N: Int> {}
class Container {
    @class test() {
        let value: Sized<Int, 3> = 0
    }
}
"#,
    );

    let callable = fixture.callable("Container", "test", phalcom_semantic::DispatchSide::Class);
    let binding = callable.bindings.values().find(|binding| binding.name == "value").expect("value binding");
    let value_type = binding
        .declared_type()
        .unwrap_or_else(|| panic!("declared type missing; diagnostics={:?}", fixture.analysis.snapshot.diagnostics));
    let (_, arguments) = fixture.analysis.snapshot.store.applied_nominal_parts(value_type).expect("Sized application");

    assert_eq!(arguments.len(), 2);
    assert_eq!(arguments[0], fixture.ty("Int"));
    assert!(
        matches!(fixture.analysis.snapshot.store.get(arguments[1]), phalcom_semantic::types::store::TypeData::ConstInt(value) if value.to_bigint() == 3.into())
    );
    assert!(matches!(
        fixture.analysis.snapshot.store.get_kind(fixture.analysis.snapshot.store.kind_of(arguments[1])),
        KindData::StaticDomain(_)
    ));
}

#[test]
fn static_generic_application_preserves_distinct_integer_identity() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
class Sized<T: Type, N: Int> {}
class Container {
    @class test() {
        let first: Sized<Int, 3> = 0
        let second: Sized<Int, 4> = 0
    }
}
"#,
    );

    let callable = fixture.callable("Container", "test", phalcom_semantic::DispatchSide::Class);
    let mut bindings = callable.bindings.values().filter(|binding| binding.name == "first" || binding.name == "second");
    let first = bindings.next().expect("first binding").declared_type().expect("first type");
    let second = bindings.next().expect("second binding").declared_type().expect("second type");
    assert_ne!(first, second);
}

#[test]
fn static_domain_generic_binder_is_published_with_static_int_kind() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
class Sized<T: Type, N: Int> {}
"#,
    );
    let sized = fixture.decl("Sized");
    let signature = fixture
        .analysis
        .snapshot
        .declarations
        .generic_signature(&sized)
        .unwrap_or_else(|| panic!("Sized signature missing; diagnostics={:?}", fixture.analysis.snapshot.diagnostics));
    assert_eq!(signature.parameters.len(), 2);
    assert_eq!(
        fixture.analysis.snapshot.store.type_parameter(signature.parameters[0]).kind,
        phalcom_semantic::types::id::KindId::TYPE
    );
    assert!(matches!(
        fixture
            .analysis
            .snapshot
            .store
            .get_kind(fixture.analysis.snapshot.store.type_parameter(signature.parameters[1]).kind),
        KindData::StaticDomain(_)
    ));
}

#[test]
fn partial_static_generic_annotations_solve_both_parameter_domains() {
    let fixture = Fixture::new(
        r#"
class Sized<T: Type, N: Int> {}
class Container {
    @class test(_ source: Sized<Int, 3>) {
        let first: Sized<_, 3> = source
        let second: Sized<Int, _> = source
        let third: Sized<_, _> = source
    }
}
"#,
    );
    fixture.assert_no_error_diagnostics();
    let callable = fixture.callable("Container", "test", phalcom_semantic::DispatchSide::Class);
    let source_type = fixture.binding(callable, "source").current.ty().expect("source type");
    for name in ["first", "second", "third"] {
        assert_eq!(fixture.binding(callable, name).declared_type(), Some(source_type));
    }
}

#[test]
fn symbolic_static_generic_arguments_keep_the_parameter_identity() {
    let fixture = Fixture::new(
        r#"
class Sized<T: Type, N: Int> {}
class Container<N: Int> {
    test(_ source: Sized<Int, N>) -> Sized<Int, N> { source }
}
"#,
    );
    fixture.assert_no_error_diagnostics();
    let callable = fixture.callable("Container", "test", phalcom_semantic::DispatchSide::Instance);
    let source_type = fixture.binding(callable, "source").current.ty().expect("source type");
    let (_, arguments) = fixture.analysis.snapshot.store.applied_nominal_parts(source_type).expect("Sized application");
    assert!(matches!(
        fixture.analysis.snapshot.store.get(arguments[1]),
        phalcom_semantic::types::store::TypeData::Parameter(_)
    ));
}

#[test]
fn signed_large_static_generic_arguments_are_exact() {
    let fixture = Fixture::new(
        r#"
class Sized<T: Type, N: Int> {}
class Container {
    @class test(_ positive: Sized<Int, 100000000000000000000>, _ negative: Sized<Int, -100000000000000000000>) {
        let first = positive
        let second = negative
    }
}
"#,
    );
    fixture.assert_no_error_diagnostics();
    let callable = fixture.callable("Container", "test", phalcom_semantic::DispatchSide::Class);
    for (name, expected) in [("first", "100000000000000000000"), ("second", "-100000000000000000000")] {
        let ty = fixture.binding(callable, name).current.ty().expect("parameter type");
        let (_, arguments) = fixture.analysis.snapshot.store.applied_nominal_parts(ty).expect("Sized application");
        assert!(
            matches!(fixture.analysis.snapshot.store.get(arguments[1]), phalcom_semantic::types::store::TypeData::ConstInt(value) if value.to_string() == expected)
        );
    }
}

#[test]
fn vector_literal_explicit_and_contextual_holes_have_the_same_identity() {
    let fixture = Fixture::new(
        r#"
class Container {
    @class test() {
        let a = <Int, 3>[1, 2, 3]
        let b = <Int, _>[1, 2, 3]
        let c = <_, 3>[1, 2, 3]
        let d = <_, _>[1, 2, 3]
        let e: Vector<Int, 3> = [1, 2, 3]
        let f: Vector<Int, _> = [1, 2, 3]
        let g: Vector<_, 3> = [1, 2, 3]
        let h: Vector<_, _> = [1, 2, 3]
    }
}
"#,
    );
    fixture.assert_no_error_diagnostics();
    let callable = fixture.callable("Container", "test", phalcom_semantic::DispatchSide::Class);
    let first = fixture.binding(callable, "a").current.ty().expect("Vector type");
    for name in ["b", "c", "d", "e", "f", "g", "h"] {
        assert_eq!(fixture.binding(callable, name).current.ty(), Some(first), "{name}");
    }
    assert_eq!(callable.sequence_literals.len(), 8);
    for resolution in callable.sequence_literals.values() {
        assert_eq!(resolution.family, phalcom_semantic::checker::SequenceLiteralFamily::Vector);
        assert_eq!(resolution.result_type, first);
        assert!(
            matches!(fixture.analysis.snapshot.store.get(resolution.extent.expect("extent")), phalcom_semantic::types::store::TypeData::ConstInt(value) if value.to_bigint() == 3.into())
        );
    }
}

#[test]
fn vector_invalid_literals_never_publish_construction_products() {
    for source in [
        "let value = <Int, 2>[1]",
        "let value: Vector<Int, 2> = <Int, 1>[1]",
        "let value: Vector<_, 2> = <Int, 1>[1]",
        "let value: Vector<Int, _> = <String, 1>[\"wrong\"]",
        "let value: Vector<_, 2> = <Int, _>[1]",
        "let value: Vector<String, 1> = <Int, 1>[1]",
        "let value = <Int, 1>[\"wrong\"]",
        "let value: Vector<Int, 1> = [\"wrong\"]",
        "let value = <_, _>[]",
        "let value = <Int, String>[1]",
        "let value: Array<Int> = <Int, 1>[1]",
        "let value: List<Int> = <Int, 1>[1]",
    ] {
        let fixture = Fixture::new_allowing_internal_incidents(&format!("class Container {{ @class test() {{ {source} }} }}\n"));
        assert!(
            fixture
                .analysis
                .snapshot
                .diagnostics
                .values()
                .flat_map(|items| items.iter())
                .any(|d| d.severity == phalcom_semantic::diagnostic::DiagnosticSeverity::Error),
            "expected rejection: {source}"
        );
        let callable = fixture.callable("Container", "test", phalcom_semantic::DispatchSide::Class);
        assert!(
            callable.sequence_literals.is_empty(),
            "invalid executable product: {source}: {:?}",
            callable.sequence_literals
        );
    }
}

#[test]
fn vector_runtime_sized_spreads_never_publish_construction_products() {
    for source_type in ["Array<Int>", "List<Int>"] {
        let fixture = Fixture::new_allowing_internal_incidents(&format!(
            "class Container {{ @class test(_ source: {source_type}) {{ let value = <Int, _>[*source] }} }}\n"
        ));
        assert!(
            fixture
                .analysis
                .snapshot
                .diagnostics
                .values()
                .flat_map(|items| items.iter())
                .any(|d| d.severity == phalcom_semantic::diagnostic::DiagnosticSeverity::Error),
            "expected rejection: {source_type}"
        );
        let callable = fixture.callable("Container", "test", phalcom_semantic::DispatchSide::Class);
        assert!(callable.sequence_literals.is_empty());
    }
}

#[test]
fn sequence_explanations_consume_the_checked_family() {
    use phalcom_semantic::checker::SequenceLiteralFamily;
    use phalcom_semantic::explain::{CollectionKind, ExplanationStep};
    let fixture = Fixture::new("class Container { @class test() { let a = <Int>[1]; let v = <Int, 1>[1]; let l: List<Int> = [1] } }\n");
    fixture.assert_no_error_diagnostics();
    let callable = fixture.callable("Container", "test", phalcom_semantic::DispatchSide::Class);
    assert_eq!(callable.sequence_literals.len(), 3);
    for (id, resolution) in callable.sequence_literals.iter() {
        let expression = &callable.expressions[id];
        let node = callable.explanations.get(expression.explanation.expect("explanation")).expect("node");
        let expected = match resolution.family {
            SequenceLiteralFamily::List => CollectionKind::List,
            SequenceLiteralFamily::Array => CollectionKind::Array,
            SequenceLiteralFamily::Vector => CollectionKind::Vector,
        };
        assert!(matches!(&node.step, ExplanationStep::CollectionSynthesis { kind, element_types, result, .. }
            if *kind == expected && element_types.as_ref() == [resolution.element_type] && *result == resolution.result_type));
    }
}

#[test]
fn vector_symbolic_identity_spread_publishes_canonical_proof() {
    let fixture = Fixture::new("class Copier<N: Int> { @class copy(_ source: Vector<Int, N>) -> Vector<Int, N> { <Int, N>[*source] } }\n");
    fixture.assert_no_error_diagnostics();
    let callable = fixture.callable("Copier", "copy", phalcom_semantic::DispatchSide::Class);
    let resolution = callable.sequence_literals.values().next().expect("checked copy");
    assert!(
        matches!(resolution.construction, phalcom_semantic::checker::SequenceConstructionPlan::VectorIdentitySpread { extent, .. }
        if Some(extent) == resolution.extent)
    );
}

#[test]
fn vector_symbolic_identity_spread_rejects_different_binders_and_arithmetic() {
    for literal in ["<Int, M>[*source]", "<Int, N>[0, *source]", "<Int, N>[*(), *source]"] {
        let fixture = Fixture::new_allowing_internal_incidents(&format!(
            "class Copier<N: Int, M: Int> {{ @class copy(_ source: Vector<Int, N>) {{ let value = {literal} }} }}\n"
        ));
        assert!(
            fixture
                .analysis
                .snapshot
                .diagnostics
                .values()
                .flat_map(|items| items.iter())
                .any(|d| d.severity == phalcom_semantic::diagnostic::DiagnosticSeverity::Error)
        );
        let callable = fixture.callable("Copier", "copy", phalcom_semantic::DispatchSide::Class);
        assert!(callable.sequence_literals.is_empty(), "{literal}");
    }
}

#[test]
fn vector_rejects_negative_concrete_extent_at_type_formation() {
    for annotation in ["Vector<Int, -1>", "Vector<_, -1>"] {
        let fixture = Fixture::new_allowing_internal_incidents(&format!(
            "class Container {{ @class test() {{ let value: {annotation} = [1] }} }}\n"
        ));
        assert!(fixture.analysis.snapshot.has_errors(), "negative concrete extent must be rejected: {annotation}");
    }
}
