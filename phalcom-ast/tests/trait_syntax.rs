use phalcom_ast::ast::{BehaviorMember, ImplMember, IndexAccessor, MemberBody, Statement, TraitMember, TypeAnnotationExpr};
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

    assert!(matches!(&trait_def.members[0], TraitMember::Behavior(BehaviorMember::Method(method)) if method.body.is_declaration()));
    assert!(matches!(&trait_def.members[1], TraitMember::Behavior(BehaviorMember::Getter(getter)) if getter.body.is_declaration()));
    assert!(matches!(&trait_def.members[2], TraitMember::Behavior(BehaviorMember::Setter(setter)) if setter.body.is_declaration()));
    assert!(matches!(&trait_def.members[3], TraitMember::Behavior(BehaviorMember::Index(index)) if matches!(index.accessor, IndexAccessor::Get) && index.body.is_declaration()));
    assert!(matches!(&trait_def.members[4], TraitMember::Behavior(BehaviorMember::Method(method)) if matches!(method.body, MemberBody::Block(_))));
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

#[test]
fn c5p1_associated_type_declaration_preserves_name_and_range() {
    let source = "trait Iterable { type Item }\n";
    let program = parse_source(source, 0).expect("associated type declaration parses");
    let Statement::Trait(trait_def) = &program.statements[0] else { panic!("expected trait") };
    let TraitMember::AssociatedType(declaration) = &trait_def.members[0] else {
        panic!("expected associated type declaration")
    };
    assert_eq!(declaration.name, "Item");
    assert_eq!(&source[declaration.name_range.start..declaration.name_range.end], "Item");
    assert_eq!(&source[declaration.range.start..declaration.range.end], "type Item");
}

#[test]
fn c5p1_trait_property_requirements_preserve_mutability_and_annotation() {
    let source = "trait Counter {\n  count: Int\n  mut total: Int\n}\n";
    let program = parse_source(source, 0).expect("trait properties parse");
    let Statement::Trait(trait_def) = &program.statements[0] else { panic!("expected trait") };
    assert!(matches!(&trait_def.members[0], TraitMember::Property(property) if property.name == "count" && !property.mutable && matches!(property.annotation.expr, TypeAnnotationExpr::Reference(_))));
    assert!(matches!(&trait_def.members[1], TraitMember::Property(property) if property.name == "total" && property.mutable));
}

#[test]
fn c5p1_trait_associated_type_default_is_rejected() {
    let source = "trait Iterable { type Item = Int }\n";
    assert!(parse_source(source, 0).is_err(), "trait associated type defaults are not P1 syntax");
}

#[test]
fn c5p2_self_associated_type_projection_preserves_nested_ranges() {
    let source = "trait Iterator {\n  type Item\n  next -> Option<Self::Item>\n}\n";
    let program = parse_source(source, 0).expect("contextual projection parses");
    let Statement::Trait(trait_def) = &program.statements[0] else { panic!("expected trait") };
    let TraitMember::Behavior(BehaviorMember::Getter(getter)) = &trait_def.members[1] else {
        panic!("expected behavioral getter")
    };
    let annotation = getter.return_annotation.as_ref().expect("return annotation");
    let TypeAnnotationExpr::Application { arguments, .. } = &annotation.expr else {
        panic!("expected Option application")
    };
    let projection = &arguments[0];
    let TypeAnnotationExpr::AssociatedTypeProjection { subject, name, name_range, range } = &projection.expr else {
        panic!("expected associated projection")
    };
    assert!(matches!(subject.expr, TypeAnnotationExpr::SelfType { .. }));
    assert_eq!(name, "Item");
    assert_eq!(&source[name_range.start..name_range.end], "Item");
    assert_eq!(&source[range.start..range.end], "Self::Item");
    assert_eq!(&source[subject.range.start..subject.range.end], "Self");
}

#[test]
fn c5p2_only_self_projection_gets_the_new_type_syntax_category() {
    let source = "class Item {}\nclass Box {}\nimpl Box { value -> Item::Part }\n";
    let program = parse_source(source, 0).expect("legacy exact-case type syntax parses");
    let Statement::Impl(impl_def) = &program.statements[2] else { panic!("expected impl") };
    let ImplMember::Behavior(BehaviorMember::Getter(getter)) = &impl_def.members[0] else {
        panic!("expected getter")
    };
    let annotation = getter.return_annotation.as_ref().expect("return annotation");
    assert!(matches!(annotation.expr, TypeAnnotationExpr::ExactEnumCase { .. }));
}
