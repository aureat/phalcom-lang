use phalcom_ast::{
    ast::{DataShapeSyntax, Expr, Statement, Visibility},
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
    assert_eq!(data_def.constructor_visibility, Visibility::Public);
    assert!(data_def.constructor_visibility_range.is_none());
    assert_eq!(data_def.generic_parameters.len(), 2);
    let DataShapeSyntax::Tuple { components, .. } = &data_def.shape else {
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
    let DataShapeSyntax::Tuple { components, .. } = &data_def.shape else {
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
        assert_eq!(data_def.constructor_visibility, expected);
        assert!(data_def.constructor_visibility_range.is_some());
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
    let DataShapeSyntax::Record { components, .. } = &data_def.shape else {
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
    let DataShapeSyntax::Tuple { components, .. } = &data_def.shape else {
        panic!("expected tuple shape");
    };
    assert!(components.is_empty());
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
