use phalcom_ast::ast::{PathSegment, StaticDomainSyntax, StaticSymbolRef, TypeAnnotation, TypeAnnotationExpr};
use phalcom_common::range::SourceRange;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::declarations::{DeclarationTypeTable, bootstrap_universe_declarations};
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::static_domain::{StaticValueDomain, is_proper_type, resolve_static_domain, validate_proper_type};
use phalcom_semantic::types::annotation::{SimpleTypeResolver, TypeFormationInvalid, TypeFormationOutcome, TypeFormationSite};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::store::TypeStore;
use phalcom_semantic::types::type_member::TypeMemberDomain;

const RANGE: SourceRange = SourceRange { start: 0, end: 1 };

struct TestEnv {
    store: TypeStore,
    declarations: DeclarationTypeTable,
    resolver: SimpleTypeResolver,
    site: TypeFormationSite,
}

fn setup() -> TestEnv {
    let mut store = TypeStore::new();
    let module = ModuleId::universe_root();
    let declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(module.clone(), key.name().into()));
    let mut resolver = SimpleTypeResolver::new();
    for name in ["Int", "String", "Bool", "Symbol", "List", "Map", "Object", "Tuple"] {
        resolver.insert(name, DeclarationId::new(module.clone(), name.into()));
    }
    TestEnv {
        store,
        declarations,
        resolver,
        site: TypeFormationSite::module(module),
    }
}

fn domain_syntax(name: &str) -> StaticDomainSyntax {
    let form = TypeAnnotation {
        expr: TypeAnnotationExpr::Reference(StaticSymbolRef {
            root: name.into(),
            root_range: RANGE,
            members: Vec::<PathSegment>::new(),
            range: RANGE,
        }),
        range: RANGE,
    };
    StaticDomainSyntax::new(form, RANGE)
}

#[test]
fn static_domain_resolves_standard_kinds() {
    let mut env = setup();
    let mut diagnostics = Vec::new();

    // Type -> Kind(Type)
    let res = resolve_static_domain(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &domain_syntax("Type"),
        &mut diagnostics,
    );
    assert_eq!(res, TypeFormationOutcome::Ready(StaticValueDomain::Kind(KindId::TYPE)));

    // TypePack -> Kind(TypePack)
    let res = resolve_static_domain(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &domain_syntax("TypePack"),
        &mut diagnostics,
    );
    assert_eq!(res, TypeFormationOutcome::Ready(StaticValueDomain::Kind(KindId::TYPE_PACK)));

    // RecordFields -> Kind(RecordRow)
    let res = resolve_static_domain(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &domain_syntax("RecordFields"),
        &mut diagnostics,
    );
    assert_eq!(res, TypeFormationOutcome::Ready(StaticValueDomain::Kind(KindId::RECORD_ROW)));

    // RecordRow -> Kind(RecordRow)
    let res = resolve_static_domain(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &domain_syntax("RecordRow"),
        &mut diagnostics,
    );
    assert_eq!(res, TypeFormationOutcome::Ready(StaticValueDomain::Kind(KindId::RECORD_ROW)));

    // LabeledTypePack -> Kind(LabeledTypePack)
    let res = resolve_static_domain(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &domain_syntax("LabeledTypePack"),
        &mut diagnostics,
    );
    assert_eq!(res, TypeFormationOutcome::Ready(StaticValueDomain::Kind(KindId::LABELED_TYPE_PACK)));

    // CompleteTypePack -> Kind(CompleteTypePack)
    let res = resolve_static_domain(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &domain_syntax("CompleteTypePack"),
        &mut diagnostics,
    );
    assert_eq!(res, TypeFormationOutcome::Ready(StaticValueDomain::Kind(KindId::COMPLETE_TYPE_PACK)));

    assert!(diagnostics.is_empty());
}

#[test]
fn static_domain_resolves_nominal_static_domain() {
    let mut env = setup();
    let mut diagnostics = Vec::new();
    let shape_decl = DeclarationId::new(ModuleId::universe_root(), "Shape".into());
    let shape_domain = env.store.nominal_static_domain(shape_decl.clone());
    let shape_kind = env.store.static_domain_kind(shape_domain);
    env.declarations.register_intrinsic_static_domain(shape_decl.clone(), shape_kind);
    env.resolver.insert("Shape", shape_decl);

    let res = resolve_static_domain(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &domain_syntax("Shape"),
        &mut diagnostics,
    );
    assert_eq!(res, TypeFormationOutcome::Ready(StaticValueDomain::Kind(shape_kind)));
    assert!(diagnostics.is_empty());
}

#[test]
fn static_domain_resolves_primitive_value_types() {
    let mut env = setup();
    let mut diagnostics = Vec::new();

    // Int -> ValueType(Int)
    let res = resolve_static_domain(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &domain_syntax("Int"),
        &mut diagnostics,
    );
    let int_type = match res {
        TypeFormationOutcome::Ready(StaticValueDomain::ValueType(ty)) => {
            assert!(is_proper_type(&env.store, ty));
            assert!(validate_proper_type(&env.store, ty).is_ok());
            ty
        }
        other => panic!("expected ValueType(Int), got {:?}", other),
    };

    // Bool -> ValueType(Bool)
    let res = resolve_static_domain(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &domain_syntax("Bool"),
        &mut diagnostics,
    );
    match res {
        TypeFormationOutcome::Ready(StaticValueDomain::ValueType(ty)) => {
            assert!(is_proper_type(&env.store, ty));
            assert_ne!(ty, int_type);
        }
        other => panic!("expected ValueType(Bool), got {:?}", other),
    };

    // String -> ValueType(String)
    let res = resolve_static_domain(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &domain_syntax("String"),
        &mut diagnostics,
    );
    match res {
        TypeFormationOutcome::Ready(StaticValueDomain::ValueType(ty)) => {
            assert!(is_proper_type(&env.store, ty));
        }
        other => panic!("expected ValueType(String), got {:?}", other),
    };

    // Symbol -> ValueType(Symbol)
    let res = resolve_static_domain(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &domain_syntax("Symbol"),
        &mut diagnostics,
    );
    match res {
        TypeFormationOutcome::Ready(StaticValueDomain::ValueType(ty)) => {
            assert!(is_proper_type(&env.store, ty));
        }
        other => panic!("expected ValueType(Symbol), got {:?}", other),
    };

    assert!(diagnostics.is_empty());
}

#[test]
fn static_domain_rejects_unsaturated_and_invalid() {
    let mut env = setup();
    let mut diagnostics = Vec::new();

    // Unsaturated constructor List (kind: Type -> Type, not proper type)
    let res = resolve_static_domain(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &domain_syntax("List"),
        &mut diagnostics,
    );
    match res {
        TypeFormationOutcome::Invalid(TypeFormationInvalid::ExpectedProperType { actual }) => {
            assert_ne!(actual, KindId::TYPE);
        }
        other => panic!("expected ExpectedProperType, got {:?}", other),
    }

    // Unknown name
    let res = resolve_static_domain(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &domain_syntax("NonExistent"),
        &mut diagnostics,
    );
    assert!(matches!(res, TypeFormationOutcome::Unresolved(_)));
}

#[test]
fn type_member_domain_adapter_round_trip() {
    let kind_domain = StaticValueDomain::Kind(KindId::TYPE);
    let member_domain: TypeMemberDomain = kind_domain.try_into().expect("kind converts to TypeMemberDomain");
    assert_eq!(member_domain, TypeMemberDomain::Kind(KindId::TYPE));
    assert_eq!(member_domain.to_static_value_domain(), kind_domain);

    let int_domain = StaticValueDomain::ValueType(phalcom_semantic::types::id::TypeId(42));
    let member_int: TypeMemberDomain = int_domain.try_into().expect("value type converts to TypeMemberDomain");
    assert_eq!(member_int, TypeMemberDomain::ValueType(phalcom_semantic::types::id::TypeId(42)));
    assert_eq!(member_int.to_static_value_domain(), int_domain);
    assert_eq!(int_domain.to_type_member_domain(), Some(member_int));
}
