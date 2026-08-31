use phalcom_ast::{
    ast::{EnumBehaviorMember, EnumMember, Statement},
    error::SyntaxErrorKind,
    parse_source,
    selector::selector_from_variant,
};

#[test]
fn bare_singleton_vs_zero_arg_constructor_variant() {
    let source = "enum Option<T> {\n  @variant None\n  @variant None()\n}\n";
    let program = parse_source(source, 0).expect("enum parses");
    let Statement::Enum(enum_def) = &program.statements[0] else {
        panic!("expected enum");
    };
    assert_eq!(enum_def.name, "Option");
    assert_eq!(enum_def.members.len(), 2);

    // 1. Bare singleton: @variant None -> payload is None, selector is #None
    let EnumMember::Variant(none_singleton) = &enum_def.members[0] else {
        panic!("expected variant");
    };
    assert_eq!(none_singleton.name, "None");
    assert!(none_singleton.payload.is_none());
    let selector_singleton = selector_from_variant(none_singleton);
    assert_eq!(selector_singleton.encode(), "None");

    // 2. Zero-arg constructor: @variant None() -> payload is Some([]), selector is #None()
    let EnumMember::Variant(none_nullary) = &enum_def.members[1] else {
        panic!("expected variant");
    };
    assert_eq!(none_nullary.name, "None");
    assert!(none_nullary.payload.is_some());
    assert!(none_nullary.payload.as_ref().unwrap().parameters.is_empty());
    let selector_nullary = selector_from_variant(none_nullary);
    assert_eq!(selector_nullary.encode(), "None()");
}

#[test]
fn variant_with_positional_and_labeled_payload() {
    let source = "enum Result<T, E> {\n  @variant Ok(value: T)\n  @variant Err(error: E)\n}\n";
    let program = parse_source(source, 0).expect("enum parses");
    let Statement::Enum(enum_def) = &program.statements[0] else {
        panic!("expected enum");
    };
    assert_eq!(enum_def.members.len(), 2);

    let EnumMember::Variant(ok_variant) = &enum_def.members[0] else {
        panic!("expected variant");
    };
    assert_eq!(ok_variant.name, "Ok");
    let payload = ok_variant.payload.as_ref().expect("payload exists");
    assert_eq!(payload.parameters.len(), 1);
    assert_eq!(payload.parameters[0].name, "value");

    let sel = selector_from_variant(ok_variant);
    assert_eq!(sel.encode(), "Ok(value)");
}

#[test]
fn gadt_syntax_with_return_type_annotation() {
    let source = "enum Expr<T> {\n  @variant IntLit(value: Int) -> Expr<Int>\n  @variant BoolLit(value: Bool) -> Expr<Bool>\n}\n";
    let program = parse_source(source, 0).expect("GADT enum parses");
    let Statement::Enum(enum_def) = &program.statements[0] else {
        panic!("expected enum");
    };
    assert_eq!(enum_def.members.len(), 2);

    let EnumMember::Variant(int_lit) = &enum_def.members[0] else {
        panic!("expected variant");
    };
    assert!(int_lit.result_annotation.is_some());
}

#[test]
fn variant_case_body_and_enum_behavior_members() {
    let source = r#"
enum Shape {
  @variant Circle(radius: Float) {
    area() -> Float { 3.14 * self.radius * self.radius }
  }
  @variant Square(side: Float) {
    area() -> Float { self.side * self.side }
  }

  describe() -> String { "shape" }
}
"#;
    let program = parse_source(source, 0).expect("enum with case bodies parses");
    let Statement::Enum(enum_def) = &program.statements[0] else {
        panic!("expected enum");
    };
    assert_eq!(enum_def.members.len(), 3);

    // Check variant case body
    let EnumMember::Variant(circle) = &enum_def.members[0] else {
        panic!("expected variant");
    };
    let body = circle.body.as_ref().expect("case body exists");
    assert_eq!(body.members.len(), 1);
    let EnumBehaviorMember::Method(method) = &body.members[0] else {
        panic!("expected method in case body");
    };
    assert_eq!(method.name, "area");

    // Check enum-level behavior member
    let EnumMember::Behavior(EnumBehaviorMember::Method(describe)) = &enum_def.members[2] else {
        panic!("expected behavior method");
    };
    assert_eq!(describe.name, "describe");
}

#[test]
fn variant_in_class_body_is_rejected() {
    let source = "class Shape {\n  @variant Circle(radius: Float)\n}\n";
    let err = parse_source(source, 0).unwrap_err();
    assert_eq!(err.kind, SyntaxErrorKind::VariantOutsideEnum);
}

#[test]
fn variant_rest_parameter_is_rejected() {
    let source = "enum VarArgs {\n  @variant Many(*items: Object)\n}\n";
    let err = parse_source(source, 0).unwrap_err();
    assert_eq!(err.kind, SyntaxErrorKind::VariantRestParameterUnsupported);
}

#[test]
fn nested_enum_declaration_in_block_is_rejected() {
    let source = "let f = || {\n  enum Local { @variant A }\n};\n";
    let err = parse_source(source, 0).unwrap_err();
    let msg = match &err.kind {
        SyntaxErrorKind::Message(m) => m.clone(),
        other => panic!("expected Message syntax error, got {other:?}"),
    };
    assert!(msg.contains("enum.nested_declaration"));
}
