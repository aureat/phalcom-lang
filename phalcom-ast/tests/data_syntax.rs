use phalcom_ast::{
    ast::{DataShapeSyntax, Expr, Statement},
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
