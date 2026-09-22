use phalcom_ast::{
    ast::{
        AlgebraicPhase, Expr, ImplementationKind, ProductShapeSyntax, Statement, Visibility,
    },
    error::SyntaxErrorKind,
    parse_source,
};

#[test]
fn data_positional_tuple_syntax() {
    let source = "data Pair<A, B>(_ first: A, _ second: B)\n";
    let program = parse_source(source, 0).expect("data parses");
    let Statement::Data(data_def) = &program.statements[0] else {
        panic!("expected data");
    };
    assert_eq!(data_def.name, "Pair");
    assert_eq!(data_def.constructor_visibility(), Visibility::Public);
    assert!(data_def.constructor_visibility_range().is_none());
    assert_eq!(data_def.generic_parameters.len(), 2);
    let Some(ProductShapeSyntax::Tuple { components, .. }) = data_def.product_shape() else {
        panic!("expected tuple shape");
    };
    assert_eq!(components.len(), 2);
    assert_eq!(components[0].local_name, "first");
    assert!(components[0].external_label.is_none());
    assert_eq!(components[1].local_name, "second");
    assert!(components[1].external_label.is_none());
}

#[test]
fn data_labeled_tuple_with_where_clause() {
    let source = "data Point<T>(x: T, y: T) where T <: Number\n";
    let program = parse_source(source, 0).expect("data parses");
    let Statement::Data(data_def) = &program.statements[0] else {
        panic!("expected data");
    };
    assert_eq!(data_def.name, "Point");
    assert_eq!(data_def.generic_parameters.len(), 1);
    assert!(data_def.where_clause.is_some());
    let Some(ProductShapeSyntax::Tuple { components, .. }) = data_def.product_shape() else {
        panic!("expected tuple shape");
    };
    assert_eq!(components.len(), 2);
    assert_eq!(components[0].local_name, "x");
    assert_eq!(components[0].external_label.as_deref(), Some("x"));
    assert_eq!(components[1].local_name, "y");
    assert_eq!(components[1].external_label.as_deref(), Some("y"));
}

#[test]
fn data_constructor_visibility_forms_parse() {
    for (source, expected) in [
        ("data Date private(year: Int, month: Int, day: Int)\n", Visibility::Private),
        ("data Token module(kind: Int, span: Int)\n", Visibility::Module),
        ("data PackageKey package { value: String }\n", Visibility::Package),
        ("data Box<T> private(_ value: T) where T <: Object\n", Visibility::Private),
        ("data Box<T> private where T <: Object (_ value: T)\n", Visibility::Private),
        ("data RuntimeId internal(raw: Int)\n", Visibility::Internal),
    ] {
        let program = parse_source(source, 0).unwrap_or_else(|error| panic!("data constructor visibility form must parse: {source:?}: {error:?}"));
        let Statement::Data(data_def) = &program.statements[0] else {
            panic!("expected data statement for {source:?}");
        };
        assert_eq!(data_def.constructor_visibility(), expected);
        assert!(data_def.constructor_visibility_range().is_some());
    }
}

#[test]
fn data_constructor_visibility_rejects_protected_and_duplicate_modifiers() {
    for (source, expected) in [
        ("data Date protected(year: Int)\n", "data.constructor_visibility.protected"),
        ("data Date private module(year: Int)\n", "data.constructor_visibility.duplicate"),
    ] {
        let error = parse_source(source, 0).expect_err("invalid data visibility must fail");
        let SyntaxErrorKind::Message(message) = error.kind else {
            panic!("expected targeted data visibility diagnostic, got {error:?}");
        };
        assert!(message.contains(expected), "expected {expected:?}, got {message:?}");
    }
}

#[test]
fn data_constructor_visibility_does_not_accept_public_or_top_level_visibility() {
    let _public = parse_source("data Date public(year: Int)\n", 0).expect_err("public is not a keyword");

    let top_level = parse_source("private data Date(year: Int)\n", 0).expect_err("top-level visibility remains illegal");
    let SyntaxErrorKind::Message(message) = top_level.kind else {
        panic!("expected top-level visibility diagnostic, got {top_level:?}");
    };
    assert!(message.contains("modifier.illegal_target"), "got {message:?}");
}

#[test]
fn data_record_syntax() {
    let source = "data Person {\n  name: String,\n  age: Int\n}\n";
    let program = parse_source(source, 0).expect("data parses");
    let Statement::Data(data_def) = &program.statements[0] else {
        panic!("expected data");
    };
    assert_eq!(data_def.name, "Person");
    let Some(ProductShapeSyntax::Record { components, .. }) = data_def.product_shape() else {
        panic!("expected record shape");
    };
    assert_eq!(components.len(), 2);
    assert_eq!(components[0].local_name, "name");
    assert_eq!(components[0].external_label.as_deref(), Some("name"));
    assert_eq!(components[1].local_name, "age");
    assert_eq!(components[1].external_label.as_deref(), Some("age"));
}

#[test]
fn data_nullary_tuple_syntax() {
    let source = "data Signal<State>()\n";
    let program = parse_source(source, 0).expect("data parses");
    let Statement::Data(data_def) = &program.statements[0] else {
        panic!("expected data");
    };
    assert_eq!(data_def.name, "Signal");
    assert_eq!(data_def.generic_parameters.len(), 1);
    let Some(ProductShapeSyntax::Tuple { components, .. }) = data_def.product_shape() else {
        panic!("expected tuple shape");
    };
    assert!(components.is_empty());
}

#[test]
fn data_point_tuple_syntax() {
    let source = "data Point(_ x: Int, _ y: Int)\n";
    let program = parse_source(source, 0).expect("data parses");
    let Statement::Data(data_def) = &program.statements[0] else {
        panic!("expected data");
    };
    assert_eq!(data_def.name, "Point");
    assert!(data_def.is_product());
    let Some(ProductShapeSyntax::Tuple { components, .. }) = data_def.product_shape() else {
        panic!("expected tuple shape");
    };
    assert_eq!(components.len(), 2);
    assert_eq!(components[0].local_name, "x");
    assert_eq!(components[1].local_name, "y");
}

#[test]
fn data_state_sum_syntax() {
    let source = "data State {\n  Ready\n  Failed(_ error: Error)\n}\n";
    let program = parse_source(source, 0).expect("data sum parses");
    let Statement::Data(data_def) = &program.statements[0] else {
        panic!("expected data");
    };
    assert_eq!(data_def.name, "State");
    assert!(data_def.is_sum());
    let cases = data_def.cases().expect("expected cases");
    assert_eq!(cases.len(), 2);
    assert_eq!(cases[0].name, "Ready");
    assert!(cases[0].is_singleton());
    assert_eq!(cases[1].name, "Failed");
    let params = cases[1].tuple_parameters().expect("expected tuple params");
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "error");
}

#[test]
fn data_weekday_comma_separated_singleton_shorthand() {
    let source = "data Weekday {\n  Monday, Tuesday, Wednesday\n}\n";
    let program = parse_source(source, 0).expect("data shorthand parses");
    let Statement::Data(data_def) = &program.statements[0] else {
        panic!("expected data");
    };
    assert_eq!(data_def.name, "Weekday");
    assert!(data_def.is_sum());
    let cases = data_def.cases().expect("expected cases");
    assert_eq!(cases.len(), 3);
    assert_eq!(cases[0].name, "Monday");
    assert!(cases[0].is_singleton());
    assert_eq!(cases[1].name, "Tuesday");
    assert!(cases[1].is_singleton());
    assert_eq!(cases[2].name, "Wednesday");
    assert!(cases[2].is_singleton());
}

#[test]
fn data_message_record_payload_sum() {
    let source = "data Message {\n  Request {\n    method: Method,\n    path: String\n  }\n  Closed\n}\n";
    let program = parse_source(source, 0).expect("data with record payload parses");
    let Statement::Data(data_def) = &program.statements[0] else {
        panic!("expected data");
    };
    assert_eq!(data_def.name, "Message");
    assert!(data_def.is_sum());
    let cases = data_def.cases().expect("expected cases");
    assert_eq!(cases.len(), 2);
    assert_eq!(cases[0].name, "Request");
    let comps = cases[0].record_components().expect("expected record components");
    assert_eq!(comps.len(), 2);
    assert_eq!(comps[0].local_name, "method");
    assert_eq!(comps[1].local_name, "path");
    assert_eq!(cases[1].name, "Closed");
    assert!(cases[1].is_singleton());
}

#[test]
fn data_example_singleton_vs_constructed() {
    let source = "data Example {\n  Singleton\n  Constructed()\n}\n";
    let program = parse_source(source, 0).expect("data parses");
    let Statement::Data(data_def) = &program.statements[0] else {
        panic!("expected data");
    };
    assert_eq!(data_def.name, "Example");
    assert!(data_def.is_sum());
    let cases = data_def.cases().expect("expected cases");
    assert_eq!(cases.len(), 2);
    assert_eq!(cases[0].name, "Singleton");
    assert!(cases[0].is_singleton());
    assert_eq!(cases[1].name, "Constructed");
    assert!(cases[1].is_explicit_empty());
}

#[test]
fn data_impossible_empty_sum() {
    let source = "data Impossible {}\n";
    let program = parse_source(source, 0).expect("empty sum parses");
    let Statement::Data(data_def) = &program.statements[0] else {
        panic!("expected data");
    };
    assert_eq!(data_def.name, "Impossible");
    assert!(data_def.is_sum());
    let cases = data_def.cases().expect("expected cases");
    assert!(cases.is_empty());
}

#[test]
fn type_data_shape_syntax() {
    let source = "type data Shape {\n  Scalar\n  Vector(_ size: Int)\n}\n";
    let program = parse_source(source, 0).expect("type data parses");
    let Statement::Data(data_def) = &program.statements[0] else {
        panic!("expected data");
    };
    assert_eq!(data_def.name, "Shape");
    assert_eq!(data_def.phase, AlgebraicPhase::TypeOnly);
    assert!(data_def.is_sum());
    let cases = data_def.cases().expect("expected cases");
    assert_eq!(cases.len(), 2);
    assert_eq!(cases[0].name, "Scalar");
    assert_eq!(cases[1].name, "Vector");
}

#[test]
fn intrinsic_data_unit_syntax() {
    let source = "intrinsic data Unit()\n";
    let program = parse_source(source, 0).expect("intrinsic data parses");
    let Statement::Data(data_def) = &program.statements[0] else {
        panic!("expected data");
    };
    assert_eq!(data_def.name, "Unit");
    assert_eq!(data_def.phase, AlgebraicPhase::Runtime);
    assert_eq!(data_def.implementation, ImplementationKind::Intrinsic);
    assert!(data_def.is_product());
    let Some(ProductShapeSyntax::Tuple { components, .. }) = data_def.product_shape() else {
        panic!("expected tuple shape");
    };
    assert!(components.is_empty());
}

#[test]
fn intrinsic_type_data_syntax() {
    let source = "intrinsic type data ShapeKind {\n  Dimension\n}\n";
    let program = parse_source(source, 0).expect("intrinsic type data parses");
    let Statement::Data(data_def) = &program.statements[0] else {
        panic!("expected data");
    };
    assert_eq!(data_def.name, "ShapeKind");
    assert_eq!(data_def.phase, AlgebraicPhase::TypeOnly);
    assert_eq!(data_def.implementation, ImplementationKind::Intrinsic);
    assert!(data_def.is_sum());
}

#[test]
fn legacy_enum_equivalent_canonicalizes_to_runtime_sum() {
    let enum_source = "enum State {\n  Ready\n  Failed(_ error: Error)\n}\n";
    let data_source = "data State {\n  Ready\n  Failed(_ error: Error)\n}\n";
    let enum_prog = parse_source(enum_source, 0).expect("enum parses");
    let data_prog = parse_source(data_source, 0).expect("data parses");

    let Statement::Data(enum_def) = &enum_prog.statements[0] else {
        panic!("expected data");
    };
    let Statement::Data(data_def) = &data_prog.statements[0] else {
        panic!("expected data");
    };

    assert_eq!(enum_def.name, data_def.name);
    assert_eq!(enum_def.phase, data_def.phase);
    assert_eq!(enum_def.phase, AlgebraicPhase::Runtime);
    assert!(enum_def.legacy_enum_keyword_range.is_some());
    assert!(data_def.legacy_enum_keyword_range.is_none());
    assert!(enum_def.is_sum());
    assert!(data_def.is_sum());
    assert_eq!(enum_def.cases().unwrap().len(), data_def.cases().unwrap().len());
}

#[test]
fn mixed_field_case_body_rejects() {
    let source = "data Invalid {\n  value: Int\n  None\n}\n";
    let error = parse_source(source, 0).expect_err("mixed field/case body must fail");
    assert_eq!(error.kind, SyntaxErrorKind::MixedAlgebraicBody);
}

#[test]
fn tuple_construction_expression_syntax() {
    let source = "Pair(1, 2)\n";
    let program = parse_source(source, 0).expect("call parses");
    let Statement::Expr { expr, .. } = &program.statements[0] else {
        panic!("expected expr");
    };
    let Expr::UnqualifiedCall(call) = expr else {
        panic!("expected unqualified call");
    };
    assert_eq!(call.name, "Pair");
    assert_eq!(call.args.len(), 2);
}

#[test]
fn record_construction_expression_syntax() {
    let source = "Person { name: \"Altun\", age: 24 }\n";
    let program = parse_source(source, 0).expect("record construction parses");
    let Statement::Expr { expr, .. } = &program.statements[0] else {
        panic!("expected expr");
    };
    let Expr::RecordConstruction(rc) = expr else {
        panic!("expected record construction");
    };
    assert_eq!(rc.target.root, "Person");
    assert_eq!(rc.entries.len(), 2);
    assert_eq!(rc.entries[0].label, "name");
    assert_eq!(rc.entries[1].label, "age");
}

#[test]
fn record_construction_with_generic_args_and_qualified_path() {
    let source = "geo.Point<Int> { x: 1, y: 2 }\n";
    let program = parse_source(source, 0).expect("record construction parses");
    let Statement::Expr { expr, .. } = &program.statements[0] else {
        panic!("expected expr");
    };
    let Expr::RecordConstruction(rc) = expr else {
        panic!("expected record construction");
    };
    assert_eq!(rc.target.root, "geo");
    assert_eq!(rc.target.members.len(), 1);
    assert_eq!(rc.target.members[0].name, "Point");
    assert_eq!(rc.type_arguments.len(), 1);
    assert_eq!(rc.entries.len(), 2);
}

#[test]
fn data_rejects_method_body() {
    let source = "data Bad {\n  foo() { return 1 }\n}\n";
    let res = parse_source(source, 0);
    assert!(res.is_err(), "expected error for method in data");
}

#[test]
fn data_rejects_rest_component() {
    let source = "data Bad(*rest: Int)\n";
    let res = parse_source(source, 0);
    assert!(res.is_err(), "expected error for rest component in data");
}

#[test]
fn data_rejects_unannotated_component() {
    let source = "data Bad(x)\n";
    let res = parse_source(source, 0);
    assert!(res.is_err(), "expected error for unannotated component in data");
}

#[test]
fn legacy_data_class_attribute_still_parses() {
    let source = "@data class Legacy { _x: Int }\n";
    let program = parse_source(source, 0).expect("legacy @data class parses");
    let Statement::Class(class_def) = &program.statements[0] else {
        panic!("expected class");
    };
    assert_eq!(class_def.name, "Legacy");
    assert_eq!(class_def.attributes.len(), 1);
    assert_eq!(class_def.attributes[0].name, "data");
}
