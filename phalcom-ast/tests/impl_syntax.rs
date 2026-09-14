use phalcom_ast::ast::{BehaviorMember, Statement};
use phalcom_ast::lexer::Lexer;
use phalcom_ast::parse_source;
use phalcom_ast::token::Token;

#[test]
fn test_impl_is_keyword_in_lexer() {
    let mut lexer = Lexer::new("impl User");
    let tok1 = lexer.next().expect("token").expect("lexeme");
    assert_eq!(tok1.1, Token::Impl);
    let tok2 = lexer.next().expect("token").expect("lexeme");
    assert_eq!(tok2.1, Token::Identifier("User".to_string()));
}

#[test]
fn test_parse_basic_impl() {
    let source = "impl User {\n  name { \"x\" }\n}\n";
    let program = parse_source(source, 0).expect("parses");
    assert_eq!(program.statements.len(), 1);
    let Statement::Impl(impl_def) = &program.statements[0] else {
        panic!("expected Statement::Impl");
    };
    assert!(impl_def.generic_parameters.is_empty());
    assert!(impl_def.where_clause.is_none());
    assert_eq!(impl_def.members.len(), 1);
    let BehaviorMember::Getter(getter) = &impl_def.members[0] else {
        panic!("expected getter member");
    };
    assert_eq!(getter.name, "name");
}

#[test]
fn test_parse_generic_impl() {
    let source = "impl<T> Point<T> {\n  translated(_ dx: T, _ dy: T) -> Point<T> {\n    Point(x: x + dx, y: y + dy)\n  }\n}\n";
    let program = parse_source(source, 0).expect("parses");
    assert_eq!(program.statements.len(), 1);
    let Statement::Impl(impl_def) = &program.statements[0] else {
        panic!("expected Statement::Impl");
    };
    assert_eq!(impl_def.generic_parameters.len(), 1);
    assert_eq!(impl_def.generic_parameters[0].name, "T");
    assert_eq!(impl_def.members.len(), 1);
    let BehaviorMember::Method(method) = &impl_def.members[0] else {
        panic!("expected method");
    };
    assert_eq!(method.name, "translated");
    assert_eq!(method.params.len(), 2);
}

#[test]
fn test_parse_impl_with_where_clause() {
    let source = "impl<T> Point<T> where T <: Number {\n  magnitude -> Float {\n    0.0\n  }\n}\n";
    let program = parse_source(source, 0).expect("parses");
    assert_eq!(program.statements.len(), 1);
    let Statement::Impl(impl_def) = &program.statements[0] else {
        panic!("expected Statement::Impl");
    };
    assert!(impl_def.where_clause.is_some());
    assert_eq!(impl_def.members.len(), 1);
}

#[test]
fn test_parse_impl_with_class_side_and_setter_and_index() {
    let source = r#"
impl Container {
  @class
  create() -> Container {
    Container.new()
  }

  value=(_ v: Int) {
    _val = v
  }

  [_ idx: Int] -> Item {
    _items.get(idx)
  }
}
"#;
    let program = parse_source(source, 0).expect("parses");
    assert_eq!(program.statements.len(), 1);
    let Statement::Impl(impl_def) = &program.statements[0] else {
        panic!("expected Statement::Impl");
    };
    assert_eq!(impl_def.members.len(), 3);
    assert!(matches!(&impl_def.members[0], BehaviorMember::Method(m) if m.attributes.iter().any(|a| a.name == "class")));
    assert!(matches!(&impl_def.members[1], BehaviorMember::Setter(_)));
    assert!(matches!(&impl_def.members[2], BehaviorMember::Index(_)));
}

#[test]
fn test_parse_impl_rejects_fields() {
    let source = "impl User {\n  const _cache: String\n}\n";
    let err = parse_source(source, 0);
    assert!(err.is_err(), "impl with const field must be rejected");

    let source_mut = "impl User {\n  _cache = 1\n}\n";
    let err_mut = parse_source(source_mut, 0);
    assert!(err_mut.is_err(), "impl with bare field must be rejected");
}

#[test]
fn test_parse_exact_case_impl_targets() {
    let source = r#"
impl Option::None {
  isSome -> Bool { false }
}

impl Example::Empty() {
  isEmpty -> Bool { true }
}

impl<T> Option<T>::Some(_) {
  isSome -> Bool { true }
}

impl Result::Error(reason: _) {
  isOk -> Bool { false }
}

impl Expression::Add(_, _) {
  precedence -> Int { 10 }
}

impl<T, U> Boxed<T>::Pack<U>(_) {
  unpack -> U { value }
}
"#;
    let program = parse_source(source, 0).expect("exact case impls parse");
    assert_eq!(program.statements.len(), 6);

    // 1. Option::None
    let Statement::Impl(impl1) = &program.statements[0] else { panic!() };
    let phalcom_ast::ast::TypeAnnotationExpr::ExactEnumCase { variant_name, payload_shape, .. } = &impl1.target.expr else { panic!() };
    assert_eq!(variant_name, "None");
    assert!(payload_shape.is_none());

    // 2. Example::Empty()
    let Statement::Impl(impl2) = &program.statements[1] else { panic!() };
    let phalcom_ast::ast::TypeAnnotationExpr::ExactEnumCase { variant_name, payload_shape, .. } = &impl2.target.expr else { panic!() };
    assert_eq!(variant_name, "Empty");
    assert!(payload_shape.as_ref().unwrap().parameters.is_empty());

    // 3. Option<T>::Some(_)
    let Statement::Impl(impl3) = &program.statements[2] else { panic!() };
    let phalcom_ast::ast::TypeAnnotationExpr::ExactEnumCase { variant_name, payload_shape, .. } = &impl3.target.expr else { panic!() };
    assert_eq!(variant_name, "Some");
    assert_eq!(payload_shape.as_ref().unwrap().parameters.len(), 1);
    assert!(payload_shape.as_ref().unwrap().parameters[0].label.is_none());

    // 4. Result::Error(reason: _)
    let Statement::Impl(impl4) = &program.statements[3] else { panic!() };
    let phalcom_ast::ast::TypeAnnotationExpr::ExactEnumCase { variant_name, payload_shape, .. } = &impl4.target.expr else { panic!() };
    assert_eq!(variant_name, "Error");
    assert_eq!(payload_shape.as_ref().unwrap().parameters.len(), 1);
    assert_eq!(payload_shape.as_ref().unwrap().parameters[0].label.as_deref(), Some("reason"));

    // 5. Expression::Add(_, _)
    let Statement::Impl(impl5) = &program.statements[4] else { panic!() };
    let phalcom_ast::ast::TypeAnnotationExpr::ExactEnumCase { variant_name, payload_shape, .. } = &impl5.target.expr else { panic!() };
    assert_eq!(variant_name, "Add");
    assert_eq!(payload_shape.as_ref().unwrap().parameters.len(), 2);

    // 6. Boxed<T>::Pack<U>(_)
    let Statement::Impl(impl6) = &program.statements[5] else { panic!() };
    let phalcom_ast::ast::TypeAnnotationExpr::ExactEnumCase { variant_name, generic_arguments, payload_shape, .. } = &impl6.target.expr else { panic!() };
    assert_eq!(variant_name, "Pack");
    assert_eq!(generic_arguments.len(), 1);
    assert_eq!(payload_shape.as_ref().unwrap().parameters.len(), 1);
}

#[test]
fn test_callable_reference_is_not_confused_with_impl_target() {
    let source = "let ref = &Option::Some(_);\n";
    let program = parse_source(source, 0).expect("callable reference expression parses");
    assert_eq!(program.statements.len(), 1);
}

#[test]
fn test_parse_impl_rejects_nested_in_block() {
    let source = "fn foo() {\n  impl User {\n    bar() {}\n  }\n}\n";
    let err = parse_source(source, 0);
    assert!(err.is_err(), "nested impl must be rejected");
}
