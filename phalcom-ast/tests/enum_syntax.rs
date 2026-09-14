use phalcom_ast::{
    ast::Statement,
    error::SyntaxErrorKind,
    parse_source,
    selector::selector_from_variant,
};

#[test]
fn bare_singleton_vs_zero_arg_constructor_variant() {
    let source = "enum Option<T> {
  @variant None
  @variant None()
}
";
    let program = parse_source(source, 0).expect("enum parses");
    let Statement::Enum(enum_def) = &program.statements[0] else {
        panic!("expected enum");
    };
    assert_eq!(enum_def.name, "Option");
    assert_eq!(enum_def.variants.len(), 2);

    // 1. Bare singleton: @variant None -> payload is None, selector is #None
    let none_singleton = &enum_def.variants[0];
    assert_eq!(none_singleton.name, "None");
    assert!(none_singleton.payload.is_none());
    let selector_singleton = selector_from_variant(none_singleton);
    assert_eq!(selector_singleton.encode(), "None");

    // 2. Zero-arg constructor: @variant None() -> payload is Some([]), selector is #None()
    let none_nullary = &enum_def.variants[1];
    assert_eq!(none_nullary.name, "None");
    assert!(none_nullary.payload.is_some());
    assert!(none_nullary.payload.as_ref().unwrap().parameters.is_empty());
    let selector_nullary = selector_from_variant(none_nullary);
    assert_eq!(selector_nullary.encode(), "None()");
}

#[test]
fn variant_with_positional_and_labeled_payload() {
    let source = "enum Result<T, E> {
  @variant Ok(value: T)
  @variant Err(error: E)
}
";
    let program = parse_source(source, 0).expect("enum parses");
    let Statement::Enum(enum_def) = &program.statements[0] else {
        panic!("expected enum");
    };
    assert_eq!(enum_def.variants.len(), 2);

    let ok_variant = &enum_def.variants[0];
    assert_eq!(ok_variant.name, "Ok");
    let payload = ok_variant.payload.as_ref().expect("payload exists");
    assert_eq!(payload.parameters.len(), 1);
    assert_eq!(payload.parameters[0].name, "value");

    let sel = selector_from_variant(ok_variant);
    assert_eq!(sel.encode(), "Ok(value)");
}

#[test]
fn gadt_syntax_with_return_type_annotation() {
    let source = "enum Expr<T> {
  @variant IntLit(value: Int) -> Expr<Int>
  @variant BoolLit(value: Bool) -> Expr<Bool>
}
";
    let program = parse_source(source, 0).expect("GADT enum parses");
    let Statement::Enum(enum_def) = &program.statements[0] else {
        panic!("expected enum");
    };
    assert_eq!(enum_def.variants.len(), 2);

    let int_lit = &enum_def.variants[0];
    assert!(int_lit.result_annotation.is_some());
}

#[test]
fn variant_local_generics_and_where_clause_are_preserved() {
    let source = "enum Expr<T> {
  @variant Pack<U>(_ value: U) -> Expr<U> where U <: Object
}
";
    let program = parse_source(source, 0).expect("variant generic syntax parses");
    let Statement::Enum(enum_def) = &program.statements[0] else {
        panic!("expected enum");
    };
    let pack = &enum_def.variants[0];

    assert_eq!(pack.name, "Pack");
    assert_eq!(pack.generic_parameters.len(), 1);
    assert_eq!(pack.generic_parameters[0].name, "U");
    assert_eq!(pack.where_clause.as_ref().expect("where clause").constraints.len(), 1);
    assert_eq!(selector_from_variant(pack).encode(), "Pack(_)");
}

#[test]
fn enum_with_nested_method_is_rejected() {
    let source = r#"
enum Shape {
  @variant Circle(radius: Float)
  describe() -> String { "shape" }
}
"#;
    let err = parse_source(source, 0).unwrap_err();
    assert_eq!(err.kind, SyntaxErrorKind::EnumBehaviorUnsupported);
}

#[test]
fn enum_with_nested_variant_body_is_rejected() {
    let source = r#"
enum Shape {
  @variant Circle(radius: Float) {
    area() -> Float { 3.14 }
  }
}
"#;
    let err = parse_source(source, 0).unwrap_err();
    assert_eq!(err.kind, SyntaxErrorKind::EnumBehaviorUnsupported);
}

#[test]
fn enum_with_class_method_is_rejected() {
    let source = r#"
enum Shape {
  @variant Circle(radius: Float)
  @class origin() -> String { "shape" }
}
"#;
    let err = parse_source(source, 0).unwrap_err();
    assert_eq!(err.kind, SyntaxErrorKind::EnumBehaviorUnsupported);
}

#[test]
fn enum_with_index_operator_is_rejected() {
    let source = r#"
enum Shape {
  @variant Circle(radius: Float)
  [index: Int] -> String { "elem" }
}
"#;
    let err = parse_source(source, 0).unwrap_err();
    assert_eq!(err.kind, SyntaxErrorKind::EnumBehaviorUnsupported);
}

#[test]
fn variant_in_class_body_is_rejected() {
    let source = "class Shape {
  @variant Circle(radius: Float)
}
";
    let err = parse_source(source, 0).unwrap_err();
    assert_eq!(err.kind, SyntaxErrorKind::VariantOutsideEnum);
}

#[test]
fn variant_rest_parameter_is_rejected() {
    let source = "enum VarArgs {
  @variant Many(*items: Object)
}
";
    let err = parse_source(source, 0).unwrap_err();
    assert_eq!(err.kind, SyntaxErrorKind::VariantRestParameterUnsupported);
}

#[test]
fn nested_enum_declaration_in_block_is_rejected() {
    let source = "let f = || {
  enum Local { @variant A }
};
";
    let err = parse_source(source, 0).unwrap_err();
    let msg = match &err.kind {
        SyntaxErrorKind::Message(m) => m.clone(),
        other => panic!("expected Message syntax error, got {other:?}"),
    };
    assert!(msg.contains("enum.nested_declaration"));
}

#[test]
fn bare_enum_variants_parse_without_variant_attribute() {
    let source = r#"
enum Expression {
  Literal(_ value: Int)
  Add(_ left: Expression, _ right: Expression)
}
"#;
    let program = parse_source(source, 0).expect("bare variant enum parses");
    let Statement::Enum(enum_def) = &program.statements[0] else { panic!() };
    assert_eq!(enum_def.name, "Expression");
    assert_eq!(enum_def.variants.len(), 2);
    let lit = &enum_def.variants[0];
    assert_eq!(lit.name, "Literal");
    let add = &enum_def.variants[1];
    assert_eq!(add.name, "Add");
}
