use phalcom_ast::ast::{BehaviorMember, IndexAccessor, MemberBody, Statement};
use phalcom_ast::lexer::Lexer;
use phalcom_ast::parse_source;
use phalcom_ast::token::Token;

#[test]
fn trait_is_a_reserved_keyword_and_preserves_name_range() {
    let mut lexer = Lexer::new("trait Display");
    assert_eq!(lexer.next().unwrap().unwrap().1, Token::Trait);

    let source = "trait Display {}\n";
    let program = parse_source(source, 0).expect("empty trait parses");
    let Statement::Trait(trait_def) = &program.statements[0] else {
        panic!("expected trait")
    };
    assert_eq!(trait_def.name, "Display");
    assert_eq!(&source[trait_def.name_range.start..trait_def.name_range.end], "Display");
    assert_eq!(&source[trait_def.range.start..trait_def.range.end], "trait Display {}");
    assert!(trait_def.members.is_empty());
}

#[test]
fn trait_members_share_behavior_syntax_and_body_representation() {
    let source = r#"trait Collection<T> where T <: Object {
  required(_ value: T) -> T
  name -> String
  value=(_ next: T) -> T
  [_ index: Int] -> T
  fallback(_ value: T) -> T { value }
}
"#;
    let program = parse_source(source, 0).expect("trait members parse");
    let Statement::Trait(trait_def) = &program.statements[0] else {
        panic!("expected trait")
    };
    assert_eq!(trait_def.generic_parameters.len(), 1);
    assert_eq!(trait_def.generic_parameters[0].name, "T");
    assert_eq!(trait_def.where_clause.as_ref().unwrap().constraints.len(), 1);
    assert_eq!(trait_def.members.len(), 5);

    assert!(matches!(&trait_def.members[0], BehaviorMember::Method(method) if method.body.is_declaration()));
    assert!(matches!(&trait_def.members[1], BehaviorMember::Getter(getter) if getter.body.is_declaration()));
    assert!(matches!(&trait_def.members[2], BehaviorMember::Setter(setter) if setter.body.is_declaration()));
    assert!(matches!(&trait_def.members[3], BehaviorMember::Index(index) if matches!(index.accessor, IndexAccessor::Get) && index.body.is_declaration()));
    assert!(matches!(&trait_def.members[4], BehaviorMember::Method(method) if matches!(method.body, MemberBody::Block(_))));
}

#[test]
fn trait_rejects_storage_and_constructor_members() {
    for source in [
        "trait Invalid {\n  const _state: Int\n}\n",
        "trait Invalid {\n  _state = 1\n}\n",
        "trait Invalid {\n  construct() {}\n}\n",
        "trait Invalid {\n  @class create() {}\n}\n",
    ] {
        assert!(parse_source(source, 0).is_err(), "expected trait rejection: {source}");
    }
}
