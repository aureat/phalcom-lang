use phalcom_ast::ast::{KindSyntax, PathSegment, StaticSymbolRef, TypeAnnotation, TypeAnnotationExpr, TypeCallableParameter, TypeTupleElement};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::declarations::{DeclarationTypeTable, bootstrap_universe_declarations};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::types::annotation::{
    KindResolution, SimpleTypeResolver, TypeFormResolution, TypeFormationInvalid, TypeFormationMissing, TypeFormationOutcome, TypeFormationSite,
    TypeFormationUnresolved, resolve_type_annotation,
};
use phalcom_semantic::types::evidence::{TypeKnowledge, UnknownReason};
use phalcom_semantic::types::id::{InferVarId, KindId, TypeId};
use phalcom_semantic::types::parameter::{GenericConstraint, GenericParameterMode, GenericSignature, TypeParameterData, TypeParameterOwner, TypeTerm};
use phalcom_semantic::types::row::{AnonymousRecordRemainder, RecordRowTail};
use phalcom_semantic::types::store::{CallableType, StaticArgument, TypeData, TypeStore};
use phalcom_semantic::types::substitution::TypeSubstitution;
use phalcom_semantic::types::type_lambda::{ScopedOpenRecord, ScopedRecordField, ScopedRecordRowTail, ScopedRecordTail, ScopedTypeData, TypeLambdaArena};

const RANGE: SourceRange = SourceRange { start: 0, end: 1 };

struct TestEnv {
    store: TypeStore,
    declarations: DeclarationTypeTable,
    resolver: SimpleTypeResolver,
    module: ModuleId,
    site: TypeFormationSite,
}

fn reference(name: &str) -> TypeAnnotation {
    TypeAnnotation {
        expr: TypeAnnotationExpr::Reference(StaticSymbolRef {
            root: name.into(),
            root_range: RANGE,
            members: Vec::<PathSegment>::new(),
            range: RANGE,
        }),
        range: RANGE,
    }
}

fn application(origin: TypeAnnotation, arguments: Vec<TypeAnnotation>) -> TypeAnnotation {
    TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(origin),
            arguments: arguments.into_iter().map(phalcom_ast::ast::GenericArgumentSyntax::Single).collect(),
            range: RANGE,
        },
        range: RANGE,
    }
}

fn pack_remainder(mode: phalcom_ast::ast::ExpansionMode, remainder: phalcom_ast::ast::AnonymousRemainderSyntax) -> phalcom_ast::ast::GenericArgumentSyntax {
    phalcom_ast::ast::GenericArgumentSyntax::Remainder(phalcom_ast::ast::TypePackRemainderSyntax { mode, remainder, range: RANGE })
}

fn type_lambda(names: &[&str], body: TypeAnnotation) -> TypeAnnotation {
    TypeAnnotation {
        expr: TypeAnnotationExpr::TypeLambda {
            parameters: names
                .iter()
                .map(|name| phalcom_ast::ast::TypeLambdaParameter {
                    name: (*name).into(),
                    name_range: RANGE,
                    kind: None,
                    range: RANGE,
                })
                .collect(),
            body: Box::new(body),
            range: RANGE,
        },
        range: RANGE,
    }
}

fn typed_type_lambda(parameters: Vec<(&str, KindSyntax)>, body: TypeAnnotation) -> TypeAnnotation {
    TypeAnnotation {
        expr: TypeAnnotationExpr::TypeLambda {
            parameters: parameters
                .into_iter()
                .map(|(name, kind)| phalcom_ast::ast::TypeLambdaParameter {
                    name: name.into(),
                    name_range: RANGE,
                    kind: Some(kind),
                    range: RANGE,
                })
                .collect(),
            body: Box::new(body),
            range: RANGE,
        },
        range: RANGE,
    }
}

fn unary_type_kind() -> KindSyntax {
    KindSyntax::Arrow {
        parameter: Box::new(KindSyntax::Type(RANGE)),
        result: Box::new(KindSyntax::Type(RANGE)),
        range: RANGE,
    }
}

fn setup() -> TestEnv {
    let mut store = TypeStore::new();
    let module = ModuleId::universe_root();
    let declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(module.clone(), key.name().into()));
    let mut resolver = SimpleTypeResolver::new();
    for name in ["Int", "String", "Bool", "List", "Map", "Object", "Tuple", "Pack"] {
        resolver.insert(name, DeclarationId::new(module.clone(), name.into()));
    }
    TestEnv {
        store,
        declarations,
        resolver,
        module: module.clone(),
        site: TypeFormationSite::module(module.clone()),
    }
}

fn resolve(env: &mut TestEnv, annotation: &TypeAnnotation) -> (TypeKnowledge, Vec<phalcom_semantic::SemanticDiagnostic>) {
    let mut diagnostics = Vec::new();
    let knowledge = resolve_type_annotation(&mut env.store, &env.declarations, &env.resolver, &env.site, annotation, &mut diagnostics);
    (knowledge, diagnostics)
}

#[test]
fn empty_tuple_annotation_and_unit_share_nested_generic_identity() {
    let mut env = setup();
    let empty = TypeAnnotation {
        expr: TypeAnnotationExpr::Tuple {
            elements: vec![],
            range: RANGE,
        },
        range: RANGE,
    };
    let unit = TypeAnnotation {
        expr: TypeAnnotationExpr::Unit { range: RANGE },
        range: RANGE,
    };
    let (empty_type, empty_diagnostics) = resolve(&mut env, &application(reference("List"), vec![empty]));
    let (unit_type, unit_diagnostics) = resolve(&mut env, &application(reference("List"), vec![unit]));
    assert!(empty_diagnostics.is_empty(), "{empty_diagnostics:?}");
    assert!(unit_diagnostics.is_empty(), "{unit_diagnostics:?}");
    assert_eq!(empty_type.ty(), unit_type.ty());
    let TypeData::Applied { arguments, .. } = env.store.get(empty_type.ty().expect("List<()> type")) else {
        panic!("expected applied List type");
    };
    assert_eq!(arguments.as_ref(), &[phalcom_semantic::types::store::StaticArgument::Term(env.store.unit())]);
}

#[test]
fn lowers_list_and_map_applications() {
    let mut env = setup();

    let list_int = resolve(&mut env, &application(reference("List"), vec![reference("Int")])).0;
    let TypeKnowledge::Known(list_int) = list_int else {
        panic!("expected List<Int>")
    };
    let TypeData::Applied { origin, arguments } = env.store.get(list_int.ty()) else {
        panic!("expected applied List<Int>")
    };
    assert_eq!(*origin, env.declarations.form(&DeclarationId::new(env.module.clone(), "List".into())).unwrap());
    assert_eq!(
        arguments.as_ref(),
        &[phalcom_semantic::types::store::StaticArgument::Term(
            env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap()
        )]
    );

    let map_string_int = resolve(&mut env, &application(reference("Map"), vec![reference("String"), reference("Int")])).0;
    let TypeKnowledge::Known(map_string_int) = map_string_int else {
        panic!("expected Map<String, Int>")
    };
    assert!(matches!(env.store.get(map_string_int.ty()), TypeData::Applied { arguments, .. } if arguments.len() == 2));
}

#[test]
fn lowers_labeled_tuple_and_callable_forms() {
    let mut env = setup();
    let tuple = TypeAnnotation {
        expr: TypeAnnotationExpr::Tuple {
            elements: vec![
                phalcom_ast::ast::TypeTupleItem::Element(TypeTupleElement {
                    label: None,
                    ty: reference("Int"),
                    range: RANGE,
                }),
                phalcom_ast::ast::TypeTupleItem::Element(TypeTupleElement {
                    label: Some("name".into()),
                    ty: reference("String"),
                    range: RANGE,
                }),
            ],
            range: RANGE,
        },
        range: RANGE,
    };
    let tuple = resolve(&mut env, &tuple).0;
    let TypeKnowledge::Known(tuple) = tuple else { panic!("expected tuple type") };
    assert!(matches!(env.store.get(tuple.ty()), TypeData::Tuple(elements) if elements.elements[1].label.as_deref() == Some("name")));

    let callable = TypeAnnotation {
        expr: TypeAnnotationExpr::Callable {
            parameters: vec![
                phalcom_ast::ast::TypeCallableItem::Parameter(TypeCallableParameter {
                    label: None,
                    ty: reference("Int"),
                    optional: false,
                    rest: false,
                    range: RANGE,
                }),
                phalcom_ast::ast::TypeCallableItem::Parameter(TypeCallableParameter {
                    label: Some("names".into()),
                    ty: reference("String"),
                    optional: false,
                    rest: true,
                    range: RANGE,
                }),
            ],
            result: Box::new(reference("Bool")),
            range: RANGE,
        },
        range: RANGE,
    };
    let callable = resolve(&mut env, &callable).0;
    let TypeKnowledge::Known(callable) = callable else {
        panic!("expected callable type")
    };
    assert!(
        matches!(env.store.get(callable.ty()), TypeData::Callable(CallableType { parameters, return_type })
        if parameters[1].label.as_deref() == Some("names")
            && parameters[1].is_rest()
            && *return_type == env.declarations.form(&DeclarationId::new(env.module.clone(), "Bool".into())).unwrap())
    );
}

#[test]
fn lowers_union_and_rejects_unsaturated_or_invalid_applications() {
    let mut env = setup();
    let union = TypeAnnotation {
        expr: TypeAnnotationExpr::Union {
            members: vec![reference("Int"), reference("String")],
            range: RANGE,
        },
        range: RANGE,
    };
    let (union, diagnostics) = resolve(&mut env, &union);
    assert!(diagnostics.is_empty());
    assert!(matches!(union, TypeKnowledge::Known(knowledge) if matches!(env.store.get(knowledge.ty()), TypeData::Union(members) if members.len() == 2)));

    let (bare_list, diagnostics) = resolve(&mut env, &reference("List"));
    assert!(matches!(bare_list, TypeKnowledge::Unknown(_)));
    assert_eq!(diagnostics[0].code, DiagnosticCode::AnnotationUnsaturatedConstructor);

    let (int_string, diagnostics) = resolve(&mut env, &application(reference("Int"), vec![reference("String")]));
    assert!(matches!(int_string, TypeKnowledge::Unknown(_)));
    assert_eq!(diagnostics[0].code, DiagnosticCode::ApplicationNotConstructor);

    let (list_list, diagnostics) = resolve(&mut env, &application(reference("List"), vec![reference("List")]));
    assert!(matches!(list_list, TypeKnowledge::Unknown(_)));
    assert_eq!(diagnostics[0].code, DiagnosticCode::ApplicationArgumentKindMismatch);
}

#[test]
fn unresolved_reference_retains_existing_diagnostic() {
    let mut env = setup();
    let (knowledge, diagnostics) = resolve(&mut env, &reference("Missing"));
    assert!(matches!(knowledge, TypeKnowledge::Unknown(_)));
    assert_eq!(diagnostics[0].code, DiagnosticCode::AnnotationUnresolved);
}

#[test]
fn resolved_but_unpublished_reference_does_not_fabricate_nominal_form() {
    let mut env = setup();
    let missing = DeclarationId::new(env.module.clone(), "Missing".into());
    env.resolver.insert("Missing", missing.clone());
    assert!(env.declarations.form(&missing).is_none());

    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &reference("Missing"),
        &mut diagnostics,
    );

    assert_eq!(result, TypeFormResolution::Missing(TypeFormationMissing::DeclarationProduct(missing)));
    assert!(diagnostics.is_empty());
}

#[test]
fn type_form_resolution_keeps_dynamic_separate_from_type_ids() {
    let mut env = setup();
    let mut diagnostics = Vec::new();
    let dynamic = TypeAnnotation {
        expr: TypeAnnotationExpr::Reference(StaticSymbolRef {
            root: "Dynamic".into(),
            root_range: RANGE,
            members: Vec::new(),
            range: RANGE,
        }),
        range: RANGE,
    };
    assert_eq!(
        TypeFormResolution::Dynamic,
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &dynamic, &mut diagnostics,)
    );
    assert!(diagnostics.is_empty());
    assert_eq!(
        KindId::TYPE,
        env.store.kind_of(env.declarations.form(&DeclarationId::new(env.module, "Int".into())).unwrap())
    );
}

#[test]
fn type_formation_distinguishes_unresolved_from_invalid() {
    let mut env = setup();
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &reference("Missing"),
        &mut diagnostics,
    );
    assert!(matches!(result, TypeFormResolution::Unresolved(TypeFormationUnresolved::Name(name)) if name.as_ref() == "Missing"));
    assert_eq!(diagnostics[0].code, DiagnosticCode::AnnotationUnresolved);

    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &application(reference("Int"), vec![reference("String")]),
        &mut diagnostics,
    );
    assert!(matches!(result, TypeFormResolution::Invalid(TypeFormationInvalid::NotAConstructor)));
    assert_eq!(diagnostics[0].code, DiagnosticCode::ApplicationNotConstructor);
}

#[test]
fn type_formation_never_uses_unannotated_for_application_failure() {
    let mut env = setup();
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &application(reference("List"), vec![reference("Int"), reference("String")]),
        &mut diagnostics,
    );
    assert!(matches!(result, TypeFormResolution::Invalid(TypeFormationInvalid::TooManyTypeArguments)));
    assert_eq!(diagnostics[0].code, DiagnosticCode::ApplicationTooManyArguments);
}

#[test]
fn proper_type_boundary_reports_unsaturated_constructor_as_invalid() {
    let mut env = setup();
    let (knowledge, diagnostics) = resolve(&mut env, &reference("List"));
    assert!(matches!(knowledge, TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause)));
    assert_eq!(diagnostics[0].code, DiagnosticCode::AnnotationUnsaturatedConstructor);
}

#[test]
fn invalid_type_syntax_is_invalid() {
    let mut env = setup();
    let annotation = TypeAnnotation {
        expr: TypeAnnotationExpr::Invalid {
            message: "recovered type syntax".into(),
            range: RANGE,
        },
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let result =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &annotation, &mut diagnostics);
    assert!(matches!(result, TypeFormResolution::Invalid(TypeFormationInvalid::Syntax)));
    assert_eq!(diagnostics[0].code, DiagnosticCode::AnnotationUnresolved);
}

#[test]
fn dynamic_type_formation_remains_explicit() {
    let mut env = setup();
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &TypeAnnotation {
            expr: TypeAnnotationExpr::Dynamic { range: RANGE },
            range: RANGE,
        },
        &mut diagnostics,
    );
    assert_eq!(result, TypeFormResolution::Dynamic);
    assert!(diagnostics.is_empty());
}

#[test]
fn recovered_invalid_kind_never_becomes_type() {
    let mut env = setup();
    let recovered = KindSyntax::Invalid {
        message: "recovered kind syntax".into(),
        range: RANGE,
    };
    let result = phalcom_semantic::types::annotation::resolve_kind_syntax(&mut env.store, &recovered);
    assert_eq!(result, KindResolution::Invalid(TypeFormationInvalid::InvalidKindSyntax));
}

#[test]
fn lowers_primitive_and_self_type_annotations() {
    let mut env = setup();
    let decl = DeclarationId::new(env.module.clone(), "Point".into());
    env.site = TypeFormationSite::member(env.module.clone(), decl.clone(), DispatchSide::Instance);

    // Unit
    let unit_ann = TypeAnnotation {
        expr: TypeAnnotationExpr::Unit { range: RANGE },
        range: RANGE,
    };
    let (unit_res, _) = resolve(&mut env, &unit_ann);
    assert_eq!(unit_res.ty().unwrap(), env.store.unit());

    // Never
    let never_ann = TypeAnnotation {
        expr: TypeAnnotationExpr::Never { range: RANGE },
        range: RANGE,
    };
    let (never_res, _) = resolve(&mut env, &never_ann);
    assert_eq!(never_res.ty().unwrap(), env.store.never());

    // Self
    let self_ann = TypeAnnotation {
        expr: TypeAnnotationExpr::SelfType { range: RANGE },
        range: RANGE,
    };
    let (self_res, _) = resolve(&mut env, &self_ann);
    assert!(matches!(env.store.get(self_res.ty().unwrap()), TypeData::SelfType(term) if term.owner == decl && term.side == DispatchSide::Instance));
}

#[test]
fn self_type_outside_owner_remains_unresolved() {
    let mut env = setup();
    let annotation = TypeAnnotation {
        expr: TypeAnnotationExpr::SelfType { range: RANGE },
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let result =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &annotation, &mut diagnostics);
    assert_eq!(result, TypeFormResolution::Unresolved(TypeFormationUnresolved::SelfOutsideOwner));
    assert_eq!(diagnostics[0].code, DiagnosticCode::AnnotationUnresolved);
}

#[test]
fn lowers_structural_record_annotations() {
    let mut env = setup();
    let record_ann = TypeAnnotation {
        expr: TypeAnnotationExpr::Record {
            fields: vec![
                phalcom_ast::ast::RecordTypeField::symbol("x", reference("Int"), false, RANGE),
                phalcom_ast::ast::RecordTypeField::symbol("y", reference("Int"), false, RANGE),
            ],
            tail: None,
            range: RANGE,
        },
        range: RANGE,
    };
    let (res, diags) = resolve(&mut env, &record_ann);
    assert!(diags.is_empty());
    assert!(matches!(env.store.get(res.ty().unwrap()), TypeData::Record(row_id) if env.store.record_row(*row_id).fields.len() == 2));
}

fn parsed_record_type(source: &str) -> TypeAnnotation {
    let program = phalcom_ast::parse_source(source, 0).expect("record type source should parse");
    program
        .statements
        .iter()
        .find_map(|statement| match statement {
            phalcom_ast::ast::Statement::TypeDefinition(definition) if definition.name == "R" => definition.type_form_body().cloned(),
            _ => None,
        })
        .expect("source should contain type R")
}

fn record_with_tail(tail: phalcom_ast::ast::RecordRowTailSyntax) -> TypeAnnotation {
    TypeAnnotation {
        expr: TypeAnnotationExpr::Record {
            fields: Vec::new(),
            tail: Some(tail),
            range: RANGE,
        },
        range: RANGE,
    }
}

fn resolve_record_tail(env: &mut TestEnv, tail: phalcom_ast::ast::AnonymousRecordRemainderSyntax) -> RecordRowTail {
    let annotation = record_with_tail(phalcom_ast::ast::RecordRowTailSyntax::Anonymous(tail));
    let mut diagnostics = Vec::new();
    let result =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &annotation, &mut diagnostics);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    let TypeFormResolution::Ready(ty) = result else {
        panic!("expected Record type for anonymous tail, got {result:?}");
    };
    let TypeData::Record(row_id) = env.store.get(ty) else {
        panic!("expected canonical Record type");
    };
    env.store.record_row(*row_id).tail
}

#[test]
fn record_type_static_literal_keys_reduce_to_canonical_field_keys() {
    let mut env = setup();
    let annotation = parsed_record_type("type R = { [#token]: Int, [\"text\"]: Int, [42]: Int, [true]: Int }\n");
    let mut diagnostics = Vec::new();
    let result =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &annotation, &mut diagnostics);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    let TypeFormResolution::Ready(ty) = result else {
        panic!("expected a concrete Record type, got {result:?}");
    };
    let TypeData::Record(row_id) = env.store.get(ty) else {
        panic!("expected canonical Record type");
    };
    let row = env.store.record_row(*row_id);
    assert_eq!(
        row.fields.iter().map(|field| field.key.clone()).collect::<Vec<_>>(),
        vec![
            phalcom_semantic::field_key::FieldKey::symbol("token"),
            phalcom_semantic::field_key::FieldKey::string("text"),
            phalcom_semantic::field_key::FieldKey::int(phalcom_semantic::scalar::StaticInt::from(42)),
            phalcom_semantic::field_key::FieldKey::bool(true),
        ]
    );
}

#[test]
fn record_type_names_and_general_expressions_fail_closed_before_row_interning() {
    let sources = [
        "type R = { [Key]: Int }\n",
        "const Key = #token\ntype R = { [Key]: Int }\n",
        "type R<T> = { [T]: Int }\n",
        "type R = { [1 + 2]: Int }\n",
    ];

    for source in sources {
        let mut env = setup();
        let annotation = parsed_record_type(source);
        let rows_before = env.store.record_row_count();
        let mut diagnostics = Vec::new();
        let result =
            phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &annotation, &mut diagnostics);
        assert_eq!(
            result,
            TypeFormResolution::Invalid(TypeFormationInvalid::RecordKeyNotStatic),
            "source: {source}"
        );
        assert_eq!(
            diagnostics.iter().map(|diagnostic| diagnostic.code).collect::<Vec<_>>(),
            vec![DiagnosticCode::RecordKeyNotStatic]
        );
        assert_eq!(
            env.store.record_row_count(),
            rows_before,
            "unresolved key must not enter canonical row storage: {source}"
        );
    }
}

#[test]
fn anonymous_record_tail_key_domains_default_and_substitute_correctly() {
    let mut env = setup();
    let default_key_type = env.store.field_key_type(&env.declarations);
    let typed_value = TypeAnnotation {
        expr: TypeAnnotationExpr::Record {
            fields: Vec::new(),
            tail: Some(phalcom_ast::ast::RecordRowTailSyntax::Anonymous(
                phalcom_ast::ast::AnonymousRecordRemainderSyntax::TypedValue {
                    value_type: Box::new(reference("String")),
                    range: RANGE,
                },
            )),
            range: RANGE,
        },
        range: RANGE,
    };
    let (knowledge, diagnostics) = resolve(&mut env, &typed_value);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    let TypeKnowledge::Known(typed_value) = knowledge else {
        panic!("expected typed Record remainder");
    };
    let TypeData::Record(row_id) = env.store.get(typed_value.ty()) else {
        panic!("expected canonical Record");
    };
    assert_eq!(
        env.store.record_row(*row_id).tail,
        RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
            key_type: default_key_type,
            value_type: env.declarations.form(&DeclarationId::new(env.module.clone(), "String".into())).unwrap(),
        })
    );

    let body = TypeAnnotation {
        expr: TypeAnnotationExpr::Record {
            fields: Vec::new(),
            tail: Some(phalcom_ast::ast::RecordRowTailSyntax::Anonymous(
                phalcom_ast::ast::AnonymousRecordRemainderSyntax::Typed {
                    key_type: Box::new(reference("K")),
                    value_type: Box::new(reference("V")),
                    range: RANGE,
                },
            )),
            range: RANGE,
        },
        range: RANGE,
    };
    let lambda = type_lambda(&["K", "V"], body);
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &lambda, &mut diagnostics);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    let TypeFormResolution::Ready(lambda) = result else {
        panic!("expected scoped Record type lambda");
    };
    let int_type = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let string_type = env.declarations.form(&DeclarationId::new(env.module.clone(), "String".into())).unwrap();
    let applied = env
        .store
        .apply_type_form(lambda, &[int_type, string_type])
        .expect("tail key/value binders should substitute");
    let TypeData::Record(row_id) = env.store.get(applied) else {
        panic!("expected specialized Record");
    };
    assert_eq!(
        env.store.record_row(*row_id).tail,
        RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
            key_type: int_type,
            value_type: string_type,
        })
    );
}

#[test]
fn record_tail_syntax_preserves_all_anonymous_key_domains() {
    let mut env = setup();
    let field_key_type = env.store.field_key_type(&env.declarations);
    let string_type = env.declarations.form(&DeclarationId::new(env.module.clone(), "String".into())).unwrap();
    let int_type = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();

    assert_eq!(
        resolve_record_tail(&mut env, phalcom_ast::ast::AnonymousRecordRemainderSyntax::Unconstrained(RANGE)),
        RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: field_key_type })
    );
    assert_eq!(
        resolve_record_tail(
            &mut env,
            phalcom_ast::ast::AnonymousRecordRemainderSyntax::TypedValue {
                value_type: Box::new(reference("Int")),
                range: RANGE,
            },
        ),
        RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
            key_type: field_key_type,
            value_type: int_type,
        })
    );
    assert_eq!(
        resolve_record_tail(
            &mut env,
            phalcom_ast::ast::AnonymousRecordRemainderSyntax::TypedKey {
                key_type: Box::new(reference("String")),
                range: RANGE,
            },
        ),
        RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: string_type })
    );
    assert_eq!(
        resolve_record_tail(
            &mut env,
            phalcom_ast::ast::AnonymousRecordRemainderSyntax::Typed {
                key_type: Box::new(reference("String")),
                value_type: Box::new(reference("Int")),
                range: RANGE,
            },
        ),
        RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
            key_type: string_type,
            value_type: int_type,
        })
    );
}

#[test]
fn anonymous_record_tail_rejects_non_type_key_domains() {
    let mut env = setup();
    let annotation = TypeAnnotation {
        expr: TypeAnnotationExpr::Record {
            fields: Vec::new(),
            tail: Some(phalcom_ast::ast::RecordRowTailSyntax::Anonymous(
                phalcom_ast::ast::AnonymousRecordRemainderSyntax::TypedKey {
                    key_type: Box::new(reference("List")),
                    range: RANGE,
                },
            )),
            range: RANGE,
        },
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let result =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &annotation, &mut diagnostics);
    assert!(matches!(result, TypeFormResolution::Invalid(TypeFormationInvalid::ExpectedProperType { .. })));
    assert_eq!(diagnostics.first().map(|diagnostic| diagnostic.code), Some(DiagnosticCode::KindExpectedType));
}

#[test]
fn open_record_tail_is_preserved_without_closed_record_recovery() {
    let mut env = setup();
    let owner_decl = DeclarationId::new(env.module.clone(), "RowCallableOwner".into());
    let owner = CallableId::new(owner_decl, Selector::getter("row").unwrap(), DispatchSide::Instance);
    let row_parameter = env.store.intern_type_parameter(phalcom_semantic::types::parameter::TypeParameterData::new(
        phalcom_semantic::types::parameter::TypeParameterOwner::Callable(owner),
        0,
        "R",
        KindId::RECORD_ROW,
    ));
    env.resolver.insert_record_row_binding("R", row_parameter);
    let closed = TypeAnnotation {
        expr: TypeAnnotationExpr::Record {
            fields: vec![phalcom_ast::ast::RecordTypeField::symbol("name", reference("String"), false, RANGE)],
            tail: None,
            range: RANGE,
        },
        range: RANGE,
    };
    let (closed_knowledge, _) = resolve(&mut env, &closed);
    let closed_type = closed_knowledge.ty().expect("closed record type");

    let open = TypeAnnotation {
        expr: TypeAnnotationExpr::Record {
            fields: vec![phalcom_ast::ast::RecordTypeField::symbol("name", reference("String"), false, RANGE)],
            tail: Some(phalcom_ast::ast::RecordRowTailSyntax::Captured(phalcom_ast::ast::RecordRowTail {
                name: "R".into(),
                range: RANGE,
            })),
            range: RANGE,
        },
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &open, &mut diagnostics);
    assert!(matches!(result, TypeFormResolution::Ready(ty) if {
        matches!(env.store.get(ty), TypeData::Record(row_id) if env.store.record_row(*row_id).tail == RecordRowTail::Parameter(row_parameter))
    }));
    assert_ne!(result, TypeFormResolution::Ready(closed_type));
    assert!(diagnostics.is_empty());
}

#[test]
fn wrong_kind_record_row_tail_is_reported_at_checked_boundary() {
    let mut env = setup();
    let int_form = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).expect("Int form");
    env.resolver.insert_type_form_binding("R", int_form);
    let annotation = TypeAnnotation {
        expr: TypeAnnotationExpr::Record {
            fields: vec![phalcom_ast::ast::RecordTypeField::symbol("value", reference("Int"), false, RANGE)],
            tail: Some(phalcom_ast::ast::RecordRowTailSyntax::Captured(phalcom_ast::ast::RecordRowTail {
                name: "R".into(),
                range: RANGE,
            })),
            range: RANGE,
        },
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let result =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &annotation, &mut diagnostics);
    assert!(matches!(
        result,
        TypeFormResolution::Invalid(TypeFormationInvalid::RecordRowTailKindMismatch { actual: KindId::TYPE })
    ));
    assert_eq!(
        diagnostics.iter().map(|diagnostic| diagnostic.code).collect::<Vec<_>>(),
        vec![DiagnosticCode::RecordRowTailKindMismatch]
    );
}

#[test]
fn lowers_type_lambda_annotations() {
    let mut env = setup();
    let lambda_ann = TypeAnnotation {
        expr: TypeAnnotationExpr::TypeLambda {
            parameters: vec![phalcom_ast::ast::TypeLambdaParameter {
                name: "T".into(),
                name_range: RANGE,
                kind: None,
                range: RANGE,
            }],
            body: Box::new(reference("Int")),
            range: RANGE,
        },
        range: RANGE,
    };
    let mut diags = Vec::new();
    let res = phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &lambda_ann, &mut diags);
    assert!(diags.is_empty());
    let TypeFormResolution::Ready(lambda_ty) = res else { panic!() };
    let TypeData::Lambda(lambda_id) = env.store.get(lambda_ty) else { panic!() };
    let lambda = env.store.arena().get_lambda(*lambda_id);
    assert!(matches!(env.store.arena().get_scoped(lambda.body), ScopedTypeData::Free(_)));
}

#[test]
fn scoped_complete_pack_expansion_projects_a_tuple_with_bound_elements() {
    let mut env = setup();
    let tuple_operand = TypeAnnotation {
        expr: TypeAnnotationExpr::Tuple {
            elements: vec![
                phalcom_ast::ast::TypeTupleItem::Element(TypeTupleElement {
                    label: None,
                    ty: reference("T"),
                    range: RANGE,
                }),
                phalcom_ast::ast::TypeTupleItem::Element(TypeTupleElement {
                    label: Some("enabled".into()),
                    ty: reference("Bool"),
                    range: RANGE,
                }),
            ],
            range: RANGE,
        },
        range: RANGE,
    };
    let tuple_application = TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(reference("Tuple")),
            arguments: vec![phalcom_ast::ast::GenericArgumentSyntax::Expansion(phalcom_ast::ast::TypePackExpansionSyntax {
                mode: phalcom_ast::ast::ExpansionMode::Complete,
                operator_range: RANGE,
                operand: tuple_operand,
                range: RANGE,
            })],
            range: RANGE,
        },
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let lambda = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &type_lambda(&["T"], tuple_application),
        &mut diagnostics,
    );
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:#?}");
    let TypeFormResolution::Ready(lambda) = lambda else {
        panic!("expected a scoped Tuple application")
    };

    let int_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let reduced = env.store.apply_type_form(lambda, &[int_ty]).expect("bound tuple element should reduce");
    let TypeData::Tuple(elements) = env.store.get(reduced) else {
        panic!("expected the expanded Tuple to reduce to its exact structural shape")
    };
    assert!(elements.elements.iter().any(|element| element.label.is_none() && element.ty == int_ty));
    assert!(elements.elements.iter().any(|element| {
        element.label.as_deref() == Some("enabled") && element.ty == env.declarations.form(&DeclarationId::new(env.module.clone(), "Bool".into())).unwrap()
    }));
}

#[test]
fn scoped_open_record_captures_free_record_row_tail() {
    let mut env = setup();
    let owner_decl = DeclarationId::new(env.module.clone(), "ScopedRowOwner".into());
    let owner = CallableId::new(owner_decl, Selector::getter("row").unwrap(), DispatchSide::Instance);
    let row_parameter = env
        .store
        .intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Callable(owner), 0, "R", KindId::RECORD_ROW));
    env.resolver.insert_record_row_binding("R", row_parameter);
    let body = TypeAnnotation {
        expr: TypeAnnotationExpr::Record {
            fields: vec![phalcom_ast::ast::RecordTypeField::symbol("value", reference("T"), false, RANGE)],
            tail: Some(phalcom_ast::ast::RecordRowTailSyntax::Captured(phalcom_ast::ast::RecordRowTail {
                name: "R".into(),
                range: RANGE,
            })),
            range: RANGE,
        },
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &type_lambda(&["T"], body),
        &mut diagnostics,
    );
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:#?}");
    let TypeFormResolution::Ready(lambda_ty) = result else {
        panic!("expected scoped open-record lambda")
    };
    let TypeData::Lambda(lambda_id) = env.store.get(lambda_ty) else {
        panic!("expected lambda type")
    };
    let lambda = env.store.arena().get_lambda(*lambda_id);
    let ScopedTypeData::OpenRecord(record) = env.store.arena().get_scoped(lambda.body) else {
        panic!("expected scoped open record")
    };
    assert_eq!(record.fields.len(), 1);
    assert_eq!(record.tail, ScopedRecordTail::FreeParameter(row_parameter));
}

#[test]
fn scoped_bound_row_tail_remains_domain_specific() {
    let mut arena = TypeLambdaArena::new();
    let field = arena.intern_scoped(ScopedTypeData::Free(TypeId::DUMMY));
    let open = arena.intern_scoped(ScopedTypeData::OpenRecord(ScopedOpenRecord {
        fields: Box::new([ScopedRecordField {
            key: phalcom_semantic::field_key::FieldKey::symbol("value"),
            ty: field,
            presence: phalcom_semantic::types::row::StructuralMemberPresence::Required,
        }]),
        tail: ScopedRecordTail::Bound { depth: 0, index: 0 },
    }));
    assert!(arena.has_free_bound(open, 0));
    let mut free_types = Vec::new();
    arena.collect_free_types(open, &mut free_types);
    assert_eq!(free_types, vec![TypeId::DUMMY]);

    let row_field = arena.intern_scoped(ScopedTypeData::Bound { depth: 0, index: 0 });
    let row = arena.intern_scoped(ScopedTypeData::RecordRow(phalcom_semantic::types::type_lambda::ScopedRecordRow {
        fields: vec![ScopedRecordField {
            key: phalcom_semantic::field_key::FieldKey::symbol("payload"),
            ty: row_field,
            presence: phalcom_semantic::types::row::StructuralMemberPresence::Required,
        }]
        .into_boxed_slice(),
        tail: ScopedRecordRowTail::Bound { depth: 0, index: 1 },
    }));
    assert!(arena.has_free_bound(row, 0), "a bound row tail is a distinct scoped binder reference");
    let mut store = TypeStore::new();
    let concrete_type = store.unit();
    let lambda = arena.intern_lambda_with_modes(
        vec![KindId::TYPE, KindId::RECORD_ROW].into_boxed_slice(),
        vec![GenericParameterMode::Single, GenericParameterMode::LabeledPack].into_boxed_slice(),
        row,
        KindId::RECORD_ROW,
        None,
    );
    let phalcom_semantic::types::type_lambda::BetaResult::ResidualLambda(residual) = arena.beta_reduce(lambda, &[concrete_type], &mut store).unwrap() else {
        panic!("partial type application should retain the unsupported row binder as a residual lambda")
    };
    let residual = arena.get_lambda(residual);
    let ScopedTypeData::RecordRow(row) = arena.get_scoped(residual.body) else {
        panic!("expected row-valued body")
    };
    assert_eq!(row.tail, ScopedRecordRowTail::Bound { depth: 0, index: 0 });
    assert!(matches!(arena.get_scoped(row.fields[0].ty), ScopedTypeData::Free(ty) if *ty == concrete_type));
}

#[test]
fn type_lambda_body_uses_bound_node() {
    let mut env = setup();
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &type_lambda(&["T"], reference("T")),
        &mut diagnostics,
    );
    let TypeFormResolution::Ready(lambda_ty) = result else {
        panic!("expected type lambda")
    };
    let TypeData::Lambda(lambda_id) = env.store.get(lambda_ty) else {
        panic!("expected lambda type")
    };
    let lambda = env.store.arena().get_lambda(*lambda_id);
    assert!(matches!(
        env.store.arena().get_scoped(lambda.body),
        ScopedTypeData::Bound { depth: 0, index: 0 }
    ));
}

#[test]
fn type_lambda_alpha_renaming_is_semantically_equal() {
    let mut env = setup();
    let mut diagnostics = Vec::new();
    let first = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &type_lambda(&["T"], reference("T")),
        &mut diagnostics,
    );
    let second = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &type_lambda(&["U"], reference("U")),
        &mut diagnostics,
    );
    let TypeFormResolution::Ready(first) = first else { panic!() };
    let TypeFormResolution::Ready(second) = second else { panic!() };
    assert_eq!(first, second);
}

#[test]
fn type_lambda_alpha_equivalence_preserves_application_shape() {
    let mut env = setup();
    let body_t = application(reference("List"), vec![reference("T")]);
    let body_u = application(reference("List"), vec![reference("U")]);
    let mut diagnostics = Vec::new();
    let first = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &type_lambda(&["T"], body_t),
        &mut diagnostics,
    );
    let second = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &type_lambda(&["U"], body_u),
        &mut diagnostics,
    );
    let TypeFormResolution::Ready(first) = first else {
        panic!("expected first type lambda")
    };
    let TypeFormResolution::Ready(second) = second else {
        panic!("expected second type lambda")
    };
    assert_eq!(first, second, "alpha-renamed applications must share canonical lambda identity");
    let TypeData::Lambda(lambda_id) = env.store.get(first) else {
        panic!("expected canonical lambda")
    };
    let lambda = env.store.arena().get_lambda(*lambda_id);
    let ScopedTypeData::Applied { origin, arguments } = env.store.arena().get_scoped(lambda.body) else {
        panic!("expected List<T> scoped application")
    };
    assert!(matches!(env.store.arena().get_scoped(*origin), ScopedTypeData::Free(_)));
    assert!(matches!(
        env.store.arena().get_scoped(arguments[0]),
        ScopedTypeData::Bound { depth: 0, index: 0 }
    ));
}

#[test]
fn nested_type_lambda_preserves_outer_and_inner_binders() {
    let mut env = setup();
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &type_lambda(&["T"], type_lambda(&["U"], reference("T"))),
        &mut diagnostics,
    );
    let TypeFormResolution::Ready(lambda_ty) = result else { panic!() };
    let TypeData::Lambda(outer_id) = env.store.get(lambda_ty) else { panic!() };
    let outer = env.store.arena().get_lambda(*outer_id);
    let ScopedTypeData::Lambda(inner_id) = env.store.arena().get_scoped(outer.body) else {
        panic!()
    };
    let inner = env.store.arena().get_lambda(*inner_id);
    assert!(matches!(env.store.arena().get_scoped(inner.body), ScopedTypeData::Bound { depth: 1, index: 0 }));
}

#[test]
fn type_lambda_keeps_declaration_parameter_free() {
    let mut env = setup();
    let owner = phalcom_semantic::types::parameter::TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "Owner".into()));
    let parameter_id = env
        .store
        .intern_type_parameter(phalcom_semantic::types::parameter::TypeParameterData::new(owner, 0, "T", KindId::TYPE));
    let parameter_form = env.store.parameter_form(parameter_id);
    env.resolver.insert_type_form_binding("T", parameter_form);
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &type_lambda(&["U"], reference("T")),
        &mut diagnostics,
    );
    let TypeFormResolution::Ready(lambda_ty) = result else { panic!() };
    let TypeData::Lambda(lambda_id) = env.store.get(lambda_ty) else { panic!() };
    let lambda = env.store.arena().get_lambda(*lambda_id);
    assert!(matches!(env.store.arena().get_scoped(lambda.body), ScopedTypeData::Free(ty) if *ty == parameter_form));
}

#[test]
fn type_lambda_beta_reduction_substitutes_without_capture() {
    let mut env = setup();
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &type_lambda(&["T"], reference("T")),
        &mut diagnostics,
    );
    let TypeFormResolution::Ready(lambda_ty) = result else { panic!() };
    let int_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    assert_eq!(env.store.apply_type_form(lambda_ty, &[int_ty]).unwrap(), int_ty);
}

#[test]
fn nested_type_lambda_beta_reduction_avoids_capture() {
    let mut env = setup();
    let body = TypeAnnotation {
        expr: TypeAnnotationExpr::Tuple {
            elements: vec![
                phalcom_ast::ast::TypeTupleItem::Element(TypeTupleElement {
                    label: None,
                    ty: reference("T"),
                    range: RANGE,
                }),
                phalcom_ast::ast::TypeTupleItem::Element(TypeTupleElement {
                    label: None,
                    ty: reference("U"),
                    range: RANGE,
                }),
            ],
            range: RANGE,
        },
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &type_lambda(&["T"], type_lambda(&["U"], body)),
        &mut diagnostics,
    );
    let TypeFormResolution::Ready(lambda_ty) = result else {
        panic!("expected nested type lambda")
    };
    let int_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let string_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "String".into())).unwrap();
    let residual = env.store.apply_type_form(lambda_ty, &[int_ty]).unwrap();
    let TypeData::Lambda(residual_id) = env.store.get(residual) else {
        panic!("expected residual inner lambda")
    };
    let residual_data = env.store.arena().get_lambda(*residual_id);
    let ScopedTypeData::Tuple(elements) = env.store.arena().get_scoped(residual_data.body) else {
        panic!("expected residual tuple body")
    };
    assert!(matches!(env.store.arena().get_scoped(elements[0].ty), ScopedTypeData::Free(ty) if *ty == int_ty));
    assert!(matches!(
        env.store.arena().get_scoped(elements[1].ty),
        ScopedTypeData::Bound { depth: 0, index: 0 }
    ));

    let reduced = env.store.apply_type_form(residual, &[string_ty]).unwrap();
    let TypeData::Tuple(elements) = env.store.get(reduced) else {
        panic!("expected fully reduced tuple")
    };
    assert_eq!(elements.elements[0].ty, int_ty);
    assert_eq!(elements.elements[1].ty, string_ty);
}

#[test]
fn fresh_captured_nested_lambda_is_applied_during_outer_reduction() {
    let mut env = setup();
    let inner_body = TypeAnnotation {
        expr: TypeAnnotationExpr::Tuple {
            elements: vec![
                phalcom_ast::ast::TypeTupleItem::Element(TypeTupleElement {
                    label: None,
                    ty: reference("T"),
                    range: RANGE,
                }),
                phalcom_ast::ast::TypeTupleItem::Element(TypeTupleElement {
                    label: None,
                    ty: reference("U"),
                    range: RANGE,
                }),
            ],
            range: RANGE,
        },
        range: RANGE,
    };
    let outer = type_lambda(&["T"], application(type_lambda(&["U"], inner_body), vec![reference("String")]));
    let mut diagnostics = Vec::new();
    let TypeFormResolution::Ready(outer_ty) =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &outer, &mut diagnostics)
    else {
        panic!("expected outer type lambda")
    };
    let int_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let result = env
        .store
        .apply_type_form(outer_ty, &[int_ty])
        .expect("fresh inner lambda is applied in the same reduction");
    let TypeData::Tuple(elements) = env.store.get(result) else {
        panic!("expected reduced tuple")
    };
    let string_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "String".into())).unwrap();
    assert_eq!(elements.elements[0].ty, int_ty);
    assert_eq!(elements.elements[1].ty, string_ty);
}

#[test]
fn type_lambda_applications_preserve_canonical_identity() {
    let mut env = setup();
    let mut diagnostics = Vec::new();
    let TypeFormResolution::Ready(lambda_ty) = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &type_lambda(&["T", "U"], application(reference("List"), vec![reference("T")])),
        &mut diagnostics,
    ) else {
        panic!("expected type lambda")
    };
    let int_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let string_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "String".into())).unwrap();

    let first_residual = env.store.apply_type_form(lambda_ty, &[int_ty]).unwrap();
    let second_residual = env.store.apply_type_form(lambda_ty, &[int_ty]).unwrap();
    assert_eq!(first_residual, second_residual, "the same prefix application must reuse its residual lambda");
    let first_full = env.store.apply_type_form(first_residual, &[string_ty]).unwrap();
    let second_full = env.store.apply_type_form(lambda_ty, &[int_ty, string_ty]).unwrap();
    assert_eq!(first_full, second_full, "equivalent full applications must canonicalize to one type")
}

#[test]
fn checked_lambda_application_failure_restores_the_active_arena() {
    let mut env = setup();
    let mut diagnostics = Vec::new();
    let resolve = |env: &mut TestEnv, syntax: TypeAnnotation, diagnostics: &mut Vec<_>| {
        let TypeFormResolution::Ready(ty) =
            phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &syntax, diagnostics)
        else {
            panic!("expected type lambda")
        };
        ty
    };
    let first = resolve(&mut env, type_lambda(&["T"], reference("T")), &mut diagnostics);
    let second = resolve(&mut env, type_lambda(&["U"], reference("U")), &mut diagnostics);
    let TypeData::Lambda(first_id) = env.store.get(first) else { panic!() };
    let TypeData::Lambda(second_id) = env.store.get(second) else { panic!() };
    let first_id = *first_id;
    let second_id = *second_id;
    let int_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let list_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "List".into())).unwrap();

    assert!(matches!(
        env.store.apply_type_form(first, &[list_ty]),
        Err(phalcom_semantic::TypeApplicationError::ArgumentKindMismatch { index: 0, .. })
    ));
    assert_eq!(env.store.arena().get_lambda(first_id).parameter_kinds.len(), 1);
    assert_eq!(env.store.arena().get_lambda(second_id).parameter_kinds.len(), 1);
    assert_eq!(env.store.apply_type_form(first, &[int_ty]).unwrap(), int_ty);
}

#[test]
fn free_capture_specialization_threads_nested_lambda_applications() {
    let mut env = setup();
    let owner = TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "CapturedOwner".into()));
    let parameter = env.store.intern_type_parameter(TypeParameterData::new(owner, 0, "T", KindId::TYPE));
    let captured_parameter = env.store.parameter_form(parameter);
    let int_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let list_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "List".into())).unwrap();
    let nested_identity = {
        let arena = env.store.arena_mut();
        let bound = arena.intern_scoped(ScopedTypeData::Bound { depth: 0, index: 0 });
        arena.intern_lambda(Box::new([KindId::TYPE]), bound, KindId::TYPE, None)
    };
    let constructor_kind = env.store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let nested_lambda_ty = env.store.intern_with_kind(TypeData::Lambda(nested_identity), constructor_kind);
    let nested_application = env.store.intern_with_kind(
        TypeData::Applied {
            origin: nested_lambda_ty,
            arguments: vec![phalcom_semantic::types::store::StaticArgument::Term(int_ty)].into_boxed_slice(),
        },
        KindId::TYPE,
    );
    let captured_type = env
        .store
        .tuple(Box::new([phalcom_semantic::types::store::TupleTypeElement::unlabeled(nested_application)]));

    let arena = env.store.arena_mut();
    let captured = arena.intern_scoped(ScopedTypeData::Free(captured_parameter));
    let list = arena.intern_scoped(ScopedTypeData::Free(list_ty));
    let body = arena.intern_scoped(ScopedTypeData::Applied {
        origin: list,
        arguments: vec![captured].into_boxed_slice(),
    });
    let lambda_id = arena.intern_lambda(Box::new([KindId::TYPE]), body, KindId::TYPE, None);
    let mut substitution = TypeSubstitution::new();
    substitution.bind(parameter, captured_type);

    let specialized_ty = env.store.substitute_type_lambda(lambda_id, &substitution);
    let same_specialized_ty = env.store.substitute_type_lambda(lambda_id, &substitution);
    assert_eq!(specialized_ty, same_specialized_ty, "repeated free-capture specialization must canonicalize");
    let string_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "String".into())).unwrap();
    let result = env.store.apply_type_form(specialized_ty, &[string_ty]).unwrap();
    let expected = env.store.apply_type_form(list_ty, &[captured_type]).unwrap();
    assert_eq!(
        result, expected,
        "the captured nested lambda application must reduce before the outer application completes"
    );
}

#[test]
fn higher_kinded_type_lambda_applies_constructor_and_argument() {
    let mut env = setup();
    let lambda = typed_type_lambda(
        vec![("F", unary_type_kind()), ("T", KindSyntax::Type(RANGE))],
        application(reference("F"), vec![reference("T")]),
    );
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &lambda, &mut diagnostics);
    let TypeFormResolution::Ready(lambda_ty) = result else {
        panic!("expected higher-kinded lambda")
    };
    let list_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "List".into())).unwrap();
    let int_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let applied = env.store.apply_type_form(lambda_ty, &[list_ty, int_ty]).unwrap();
    let expected = env.store.apply_type_form(list_ty, &[int_ty]).unwrap();
    assert_eq!(applied, expected, "F<T> with F := List must beta-reduce to List<Int>");
    assert_eq!(env.store.kind_of(applied), KindId::TYPE);
}

#[test]
fn nested_higher_kinded_lambda_shadowing_is_capture_safe() {
    let mut env = setup();
    let lambda = typed_type_lambda(
        vec![("F", unary_type_kind())],
        typed_type_lambda(vec![("F", unary_type_kind())], application(reference("F"), vec![reference("Int")])),
    );
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &lambda, &mut diagnostics);
    let TypeFormResolution::Ready(lambda_ty) = result else {
        panic!("expected nested higher-kinded lambda")
    };
    let list_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "List".into())).unwrap();
    let list_kind = env.store.kind_of(list_ty);
    let other_ty = env.store.nominal_form(DeclarationId::new(env.module.clone(), "Other".into()), list_kind);
    let residual = env.store.apply_type_form(lambda_ty, &[list_ty]).unwrap();
    let reduced = env.store.apply_type_form(residual, &[other_ty]).unwrap();
    let int_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let expected = env.store.apply_type_form(other_ty, &[int_ty]).unwrap();
    assert_eq!(reduced, expected, "inner F must shadow outer F");
}

#[test]
fn partial_type_lambda_application_returns_residual_lambda() {
    let mut env = setup();
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &type_lambda(&["T", "U"], reference("T")),
        &mut diagnostics,
    );
    let TypeFormResolution::Ready(lambda_ty) = result else { panic!() };
    let int_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let residual = env.store.apply_type_form(lambda_ty, &[int_ty]).unwrap();
    let TypeData::Lambda(residual_id) = env.store.get(residual) else {
        panic!("expected residual lambda")
    };
    let residual_data = env.store.arena().get_lambda(*residual_id);
    assert_eq!(residual_data.parameter_kinds.len(), 1);
    assert!(matches!(env.store.arena().get_scoped(residual_data.body), ScopedTypeData::Free(ty) if *ty == int_ty));
}

#[test]
fn partial_type_lambda_application_can_be_applied_again() {
    let mut env = setup();
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &type_lambda(&["T", "U"], application(reference("List"), vec![reference("T")])),
        &mut diagnostics,
    );
    let TypeFormResolution::Ready(lambda_ty) = result else {
        panic!("expected type lambda")
    };
    let int_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let string_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "String".into())).unwrap();
    let residual = env.store.apply_type_form(lambda_ty, &[int_ty]).unwrap();
    let reduced = env.store.apply_type_form(residual, &[string_ty]).unwrap();
    let expected = env
        .store
        .apply_type_form(
            env.declarations.form(&DeclarationId::new(env.module.clone(), "List".into())).unwrap(),
            &[int_ty],
        )
        .unwrap();
    assert_eq!(reduced, expected, "residual application must preserve substituted outer argument");
}

#[test]
fn partially_applied_map_has_residual_constructor_kind() {
    let mut env = setup();
    let mut diagnostics = Vec::new();
    let map_string = application(reference("Map"), vec![reference("String")]);
    let TypeFormResolution::Ready(map_string) =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &map_string, &mut diagnostics)
    else {
        panic!("expected partially applied Map")
    };
    let residual_kind = env.store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    assert_eq!(env.store.kind_of(map_string), residual_kind);
}

#[test]
fn record_fields_generic_binder_defaults_without_creating_type_parameter_form() {
    let mut env = setup();
    let owner_decl = DeclarationId::new(env.module.clone(), "RowContainer".into());
    let owner_callable = CallableId::new(owner_decl, Selector::getter("row").unwrap(), DispatchSide::Instance);
    let owner = phalcom_semantic::types::parameter::TypeParameterOwner::Callable(owner_callable);
    let params = vec![phalcom_ast::ast::GenericParameterSyntax {
        mode: Some(phalcom_ast::ast::ExpansionMode::Labeled),
        mode_range: Some(RANGE),
        variance: phalcom_ast::ast::VarianceSyntax::Invariant,
        name: "R".into(),
        name_range: RANGE,
        kind: None,
        range: RANGE,
    }];
    let mut diags = Vec::new();
    let signature = phalcom_semantic::types::annotation::resolve_generic_signature(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        owner,
        phalcom_semantic::types::annotation::GenericBinderSite::Callable,
        &params,
        None,
        &mut diags,
    );
    assert!(diags.is_empty());
    let phalcom_semantic::types::annotation::TypeFormationOutcome::Ready(signature) = signature else {
        panic!("expected ready signature")
    };
    let parameter_id = signature.parameter_at(0).expect("row binder parameter");
    assert_eq!(env.store.type_parameter(parameter_id).kind, KindId::RECORD_ROW);
    assert!(
        !env.store.contains_parameter_type(parameter_id),
        "row binders must not publish TypeData::Parameter"
    );
    assert!(signature.validate_publishable(&env.store).is_ok());

    env.resolver.insert_record_row_binding("R", parameter_id);
    let (knowledge, diagnostics) = resolve(&mut env, &reference("R"));
    assert!(matches!(knowledge, TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause)));
    assert_eq!(diagnostics[0].code, DiagnosticCode::KindExpectedType);
}

#[test]
fn lowers_generic_signature_with_where_constraints() {
    let mut env = setup();
    let owner = phalcom_semantic::types::parameter::TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "Container".into()));
    let params = vec![phalcom_ast::ast::GenericParameterSyntax {
        mode: None,
        mode_range: None,
        variance: phalcom_ast::ast::VarianceSyntax::Covariant,
        name: "T".into(),
        name_range: RANGE,
        kind: None,
        range: RANGE,
    }];
    let where_clause = phalcom_ast::ast::WhereClauseSyntax {
        constraints: vec![phalcom_ast::ast::GenericConstraintSyntax::Subtype {
            lower: reference("T"),
            upper: reference("Object"),
            range: RANGE,
        }],
        range: RANGE,
    };

    let mut diags = Vec::new();
    let sig = phalcom_semantic::types::annotation::resolve_generic_signature(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        owner,
        phalcom_semantic::types::annotation::GenericBinderSite::NominalDeclaration,
        &params,
        Some(&where_clause),
        &mut diags,
    );

    let phalcom_semantic::types::annotation::TypeFormationOutcome::Ready(sig) = sig else {
        panic!("expected ready signature")
    };
    assert_eq!(sig.parameter_count(), 1);
    assert_eq!(sig.constraint_count(), 1);
}

#[test]
fn variadic_generic_binders_derive_modes_kinds_and_reject_ambiguous_order() {
    let mut env = setup();
    let owner = TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "Variadic".into()));
    let params = vec![
        phalcom_ast::ast::GenericParameterSyntax {
            mode: Some(phalcom_ast::ast::ExpansionMode::Positional),
            mode_range: Some(RANGE),
            variance: phalcom_ast::ast::VarianceSyntax::Invariant,
            name: "Ts".into(),
            name_range: RANGE,
            kind: None,
            range: RANGE,
        },
        phalcom_ast::ast::GenericParameterSyntax {
            mode: Some(phalcom_ast::ast::ExpansionMode::Labeled),
            mode_range: Some(RANGE),
            variance: phalcom_ast::ast::VarianceSyntax::Invariant,
            name: "Ls".into(),
            name_range: RANGE,
            kind: None,
            range: RANGE,
        },
        phalcom_ast::ast::GenericParameterSyntax {
            mode: Some(phalcom_ast::ast::ExpansionMode::Complete),
            mode_range: Some(RANGE),
            variance: phalcom_ast::ast::VarianceSyntax::Invariant,
            name: "Es".into(),
            name_range: RANGE,
            kind: None,
            range: RANGE,
        },
    ];
    let mut diagnostics = Vec::new();
    let phalcom_semantic::types::annotation::TypeFormationOutcome::Ready(signature) = phalcom_semantic::types::annotation::resolve_generic_signature(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        owner,
        phalcom_semantic::types::annotation::GenericBinderSite::NominalDeclaration,
        &params,
        None,
        &mut diagnostics,
    ) else {
        panic!("expected variadic signature")
    };
    assert!(diagnostics.is_empty());
    assert_eq!(
        signature.parameter_modes.as_ref(),
        [
            phalcom_semantic::types::parameter::GenericParameterMode::PositionalPack,
            phalcom_semantic::types::parameter::GenericParameterMode::LabeledPack,
            phalcom_semantic::types::parameter::GenericParameterMode::CompletePack,
        ]
    );
    assert_eq!(env.store.type_parameter(signature.parameters[0]).kind, KindId::TYPE_PACK);
    assert_eq!(env.store.type_parameter(signature.parameters[1]).kind, KindId::RECORD_ROW);
    assert_eq!(env.store.type_parameter(signature.parameters[2]).kind, KindId::COMPLETE_TYPE_PACK);

    let mut explicit_labeled = params.clone();
    explicit_labeled[1].kind = Some(KindSyntax::LabeledTypePack(RANGE));
    let mut diagnostics = Vec::new();
    let TypeFormationOutcome::Ready(explicit_signature) = phalcom_semantic::types::annotation::resolve_generic_signature(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "ExplicitLabeled".into())),
        phalcom_semantic::types::annotation::GenericBinderSite::NominalDeclaration,
        &explicit_labeled,
        None,
        &mut diagnostics,
    ) else {
        panic!("explicit **Ls: LabeledTypePack remains valid: {diagnostics:?}");
    };
    assert!(diagnostics.is_empty());
    assert_eq!(env.store.type_parameter(explicit_signature.parameters[1]).kind, KindId::LABELED_TYPE_PACK);

    let mut explicit_record_fields = params.clone();
    explicit_record_fields[1].kind = Some(KindSyntax::RecordFields(RANGE));
    let mut diagnostics = Vec::new();
    let TypeFormationOutcome::Ready(explicit_record_signature) = phalcom_semantic::types::annotation::resolve_generic_signature(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "ExplicitRecordFields".into())),
        phalcom_semantic::types::annotation::GenericBinderSite::NominalDeclaration,
        &explicit_record_fields,
        None,
        &mut diagnostics,
    ) else {
        panic!("explicit **Fs: RecordFields remains valid: {diagnostics:?}");
    };
    assert!(diagnostics.is_empty());
    assert_eq!(env.store.type_parameter(explicit_record_signature.parameters[1]).kind, KindId::RECORD_ROW);

    let mut diagnostics = Vec::new();
    let invalid = vec![
        params[0].clone(),
        phalcom_ast::ast::GenericParameterSyntax {
            mode: None,
            mode_range: None,
            variance: phalcom_ast::ast::VarianceSyntax::Invariant,
            name: "T".into(),
            name_range: RANGE,
            kind: None,
            range: RANGE,
        },
    ];
    let result = phalcom_semantic::types::annotation::resolve_generic_signature(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "InvalidVariadic".into())),
        phalcom_semantic::types::annotation::GenericBinderSite::NominalDeclaration,
        &invalid,
        None,
        &mut diagnostics,
    );
    assert!(matches!(
        result,
        phalcom_semantic::types::annotation::TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax)
    ));
    assert!(!diagnostics.is_empty());
}

#[test]
fn explicit_pack_kinds_must_match_the_expansion_prefix() {
    let mut env = setup();
    let incompatible = [
        (phalcom_ast::ast::ExpansionMode::Positional, KindSyntax::RecordFields(RANGE)),
        (phalcom_ast::ast::ExpansionMode::Complete, KindSyntax::RecordFields(RANGE)),
        (phalcom_ast::ast::ExpansionMode::Labeled, KindSyntax::TypePack(RANGE)),
    ];

    for (index, (mode, kind)) in incompatible.into_iter().enumerate() {
        let parameter = phalcom_ast::ast::GenericParameterSyntax {
            mode: Some(mode),
            mode_range: Some(RANGE),
            variance: phalcom_ast::ast::VarianceSyntax::Invariant,
            name: format!("P{index}"),
            name_range: RANGE,
            kind: Some(kind),
            range: RANGE,
        };
        let mut diagnostics = Vec::new();
        let result = phalcom_semantic::types::annotation::resolve_generic_signature(
            &mut env.store,
            &env.declarations,
            &env.resolver,
            &env.site,
            TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), format!("BadPack{index}").into())),
            phalcom_semantic::types::annotation::GenericBinderSite::NominalDeclaration,
            &[parameter],
            None,
            &mut diagnostics,
        );
        assert!(matches!(result, TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax)));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::AnnotationUnsupported));
    }
}

#[test]
fn variadic_nominal_application_is_binder_aligned_and_empty_specializes() {
    let mut env = setup();
    let declaration = DeclarationId::new(env.module.clone(), "V".into());
    let parameter = env.store.intern_type_parameter(
        TypeParameterData::new(TypeParameterOwner::Declaration(declaration.clone()), 0, "Ts", KindId::TYPE_PACK)
            .with_mode(phalcom_semantic::types::parameter::GenericParameterMode::PositionalPack),
    );
    let signature = GenericSignature::new(TypeParameterOwner::Declaration(declaration.clone()), vec![parameter].into_boxed_slice())
        .with_parameter_modes(vec![phalcom_semantic::types::parameter::GenericParameterMode::PositionalPack].into_boxed_slice());
    let kind = env.store.arrow_kind(vec![KindId::TYPE_PACK].into_boxed_slice(), KindId::TYPE);
    let form = env.store.nominal_form(declaration.clone(), kind);
    let class_object_type = env.store.class_object_type(declaration.clone());
    env.declarations.insert(phalcom_semantic::declarations::DeclarationTypeInfo {
        declaration: declaration.clone(),
        form,
        class_object_type,
        kind,
        binding: phalcom_semantic::declarations::DeclarationTypeBinding::ordinary(),
        generic_binder_header: Some(phalcom_semantic::types::parameter::GenericBinderHeader::from_signature(&env.store, &signature)),
        generic_signature: Some(signature),
        supertype_template: None,
    });
    env.resolver.insert("V", declaration);

    let empty = TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(reference("V")),
            arguments: vec![],
            range: RANGE,
        },
        range: RANGE,
    };
    let (empty, diagnostics) = resolve(&mut env, &empty);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let TypeData::Applied { arguments, .. } = env.store.get(empty.ty().expect("V<> proper type")) else {
        panic!("expected binder-aligned application")
    };
    assert!(matches!(env.store.get(arguments[0].as_type_id().expect("type-only argument")), TypeData::TypePack(elements) if elements.is_empty()));

    let direct = application(reference("V"), vec![reference("Int"), reference("String")]);
    let (_, diagnostics) = resolve(&mut env, &direct);
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::AnnotationUnsupported),
        "{diagnostics:?}"
    );

    let tuple = TypeAnnotation {
        expr: TypeAnnotationExpr::Tuple {
            elements: vec![
                phalcom_ast::ast::TypeTupleItem::Element(TypeTupleElement {
                    label: None,
                    ty: reference("Int"),
                    range: RANGE,
                }),
                phalcom_ast::ast::TypeTupleItem::Element(TypeTupleElement {
                    label: None,
                    ty: reference("String"),
                    range: RANGE,
                }),
            ],
            range: RANGE,
        },
        range: RANGE,
    };
    let expanded = TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(reference("V")),
            arguments: vec![phalcom_ast::ast::GenericArgumentSyntax::Expansion(phalcom_ast::ast::TypePackExpansionSyntax {
                mode: phalcom_ast::ast::ExpansionMode::Positional,
                operator_range: RANGE,
                operand: tuple,
                range: RANGE,
            })],
            range: RANGE,
        },
        range: RANGE,
    };
    let (expanded, diagnostics) = resolve(&mut env, &expanded);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let TypeData::Applied { arguments, .. } = env.store.get(expanded.ty().expect("V<*(Int, String)> proper type")) else {
        panic!("expected binder-aligned application")
    };
    assert!(matches!(env.store.get(arguments[0].as_type_id().expect("type-only argument")), TypeData::TypePack(elements) if elements.len() == 2));
}

#[test]
fn fixed_generic_prefix_accepts_explicit_complete_pack_expansion_only() {
    let mut env = setup();
    let declaration = DeclarationId::new(env.module.clone(), "Product".into());
    let fixed = env.store.intern_type_parameter(
        TypeParameterData::new(TypeParameterOwner::Declaration(declaration.clone()), 0, "T", KindId::TYPE).with_mode(GenericParameterMode::Single),
    );
    let entries = env.store.intern_type_parameter(
        TypeParameterData::new(TypeParameterOwner::Declaration(declaration.clone()), 1, "Es", KindId::COMPLETE_TYPE_PACK)
            .with_mode(GenericParameterMode::CompletePack),
    );
    let signature = GenericSignature::new(TypeParameterOwner::Declaration(declaration.clone()), vec![fixed, entries].into_boxed_slice())
        .with_parameter_metadata(
            vec![KindId::TYPE, KindId::COMPLETE_TYPE_PACK].into_boxed_slice(),
            vec![
                phalcom_semantic::types::variance::Variance::Invariant,
                phalcom_semantic::types::variance::Variance::Invariant,
            ]
            .into_boxed_slice(),
        )
        .with_parameter_modes(vec![GenericParameterMode::Single, GenericParameterMode::CompletePack].into_boxed_slice());
    let kind = env
        .store
        .arrow_kind(vec![KindId::TYPE, KindId::COMPLETE_TYPE_PACK].into_boxed_slice(), KindId::TYPE);
    let form = env.store.nominal_form(declaration.clone(), kind);
    let class_object_type = env.store.class_object_type(declaration.clone());
    env.declarations.insert(phalcom_semantic::declarations::DeclarationTypeInfo {
        declaration: declaration.clone(),
        form,
        class_object_type,
        kind,
        binding: phalcom_semantic::declarations::DeclarationTypeBinding::ordinary(),
        generic_binder_header: Some(phalcom_semantic::types::parameter::GenericBinderHeader::from_signature(&env.store, &signature)),
        generic_signature: Some(signature),
        supertype_template: None,
    });
    env.resolver.insert("Product", declaration);

    let tuple = TypeAnnotation {
        expr: TypeAnnotationExpr::Tuple {
            elements: vec![
                phalcom_ast::ast::TypeTupleItem::Element(TypeTupleElement {
                    label: None,
                    ty: reference("String"),
                    range: RANGE,
                }),
                phalcom_ast::ast::TypeTupleItem::Element(TypeTupleElement {
                    label: Some("flag".into()),
                    ty: reference("Bool"),
                    range: RANGE,
                }),
            ],
            range: RANGE,
        },
        range: RANGE,
    };
    let expanded = TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(reference("Product")),
            arguments: vec![
                phalcom_ast::ast::GenericArgumentSyntax::Single(reference("Int")),
                phalcom_ast::ast::GenericArgumentSyntax::Expansion(phalcom_ast::ast::TypePackExpansionSyntax {
                    mode: phalcom_ast::ast::ExpansionMode::Complete,
                    operator_range: RANGE,
                    operand: tuple,
                    range: RANGE,
                }),
            ],
            range: RANGE,
        },
        range: RANGE,
    };
    let (expanded_type, diagnostics) = resolve(&mut env, &expanded);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let TypeData::Applied { arguments, .. } = env.store.get(expanded_type.ty().expect("Product application is a type")) else {
        panic!("expected the generic Product application")
    };
    let StaticArgument::Term(complete_pack) = arguments[1] else {
        panic!("the complete pack must remain a type-domain argument")
    };
    assert!(matches!(env.store.get(complete_pack), TypeData::CompleteTypePack { .. }));

    let direct = TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(reference("Product")),
            arguments: vec![
                phalcom_ast::ast::GenericArgumentSyntax::Single(reference("Int")),
                phalcom_ast::ast::GenericArgumentSyntax::Single(reference("String")),
            ],
            range: RANGE,
        },
        range: RANGE,
    };
    let (_, diagnostics) = resolve(&mut env, &direct);
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::AnnotationUnsupported),
        "{diagnostics:?}"
    );
}

#[test]
fn tuple_pack_expansion_preserves_an_abstract_positional_lane() {
    let mut env = setup();
    let owner = TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "TupleAlias".into()));
    let parameter = env.store.intern_type_parameter(TypeParameterData::new(owner, 0, "P", KindId::TYPE_PACK));
    let pack = env.store.parameter_form(parameter);
    env.resolver.insert_type_form_binding("P", pack);
    let tuple = TypeAnnotation {
        expr: TypeAnnotationExpr::Tuple {
            elements: vec![phalcom_ast::ast::TypeTupleItem::Expansion(phalcom_ast::ast::TypePackExpansionSyntax {
                mode: phalcom_ast::ast::ExpansionMode::Positional,
                operator_range: RANGE,
                operand: reference("P"),
                range: RANGE,
            })],
            range: RANGE,
        },
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let TypeFormResolution::Ready(tuple) =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &tuple, &mut diagnostics)
    else {
        panic!("expected Tuple pack expansion to lower")
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(matches!(env.store.get(tuple), TypeData::TupleFromPacks { positionals, .. } if *positionals == pack));
}

#[test]
fn intrinsic_tuple_application_reuses_the_complete_pack_structural_family() {
    let mut env = setup();
    let tuple = DeclarationId::new(env.module.clone(), "Tuple".into());
    let mut tuple_binding = env.declarations.get(&tuple).expect("canonical Tuple binding").clone();
    tuple_binding.binding.runtime_owner = None;
    env.declarations.remove(&tuple);
    env.declarations.insert(tuple_binding);
    assert_eq!(env.declarations.intrinsic_runtime_owner(&tuple), None);
    env.resolver.insert("Tuple", tuple);

    let empty_application = application(reference("Tuple"), vec![]);
    let mut diagnostics = Vec::new();
    let TypeFormResolution::Ready(empty_tuple) = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &empty_application,
        &mut diagnostics,
    ) else {
        panic!("zero-contribution Tuple should select the empty CompleteTypePack: {diagnostics:?}");
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(empty_tuple, env.store.unit(), "the empty Tuple projection retains Unit identity");

    let owner = TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "TupleAlias".into()));
    let parameter = env.store.intern_type_parameter(TypeParameterData::new(owner, 0, "P", KindId::TYPE_PACK));
    let positional = env.store.parameter_form(parameter);
    let labeled = env.store.labeled_type_pack(Box::new([])).unwrap();
    let complete = env.store.complete_type_pack(positional, labeled).unwrap();
    env.resolver.insert_type_form_binding("Es", complete);
    let application = TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(reference("Tuple")),
            arguments: vec![phalcom_ast::ast::GenericArgumentSyntax::Expansion(phalcom_ast::ast::TypePackExpansionSyntax {
                mode: phalcom_ast::ast::ExpansionMode::Complete,
                operator_range: RANGE,
                operand: reference("Es"),
                range: RANGE,
            })],
            range: RANGE,
        },
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let TypeFormResolution::Ready(applied) =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &application, &mut diagnostics)
    else {
        panic!("expected intrinsic Tuple application to resolve")
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(
        matches!(env.store.get(applied), TypeData::TupleFromPacks { positionals: actual, labeled: actual_labeled } if *actual == positional && *actual_labeled == labeled)
    );
    assert!(
        !matches!(env.store.get(applied), TypeData::Applied { .. }),
        "intrinsic application must not remain nominal `Applied` data"
    );
}

#[test]
fn scoped_zero_tuple_and_record_applications_select_their_declared_domains() {
    let mut env = setup();
    env.resolver.insert("Record", DeclarationId::new(env.module.clone(), "Record".into()));
    let scoped_tuple = type_lambda(&["T"], application(reference("Tuple"), vec![]));
    let mut diagnostics = Vec::new();
    let TypeFormResolution::Ready(tuple_lambda) =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &scoped_tuple, &mut diagnostics)
    else {
        panic!("scoped zero Tuple should lower: {diagnostics:?}");
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let TypeData::Lambda(tuple_lambda_id) = env.store.get(tuple_lambda) else {
        panic!("scoped Tuple application should remain a type lambda");
    };
    assert!(matches!(
        env.store.arena().get_scoped(env.store.arena().get_lambda(*tuple_lambda_id).body),
        ScopedTypeData::TupleFromPacks { .. }
    ));
    let int_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    assert_eq!(env.store.apply_type_form(tuple_lambda, &[int_ty]).unwrap(), env.store.unit());

    let row_parameter = TypeParameterData::new(
        TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "ScopedRecordOwner".into())),
        0,
        "R",
        KindId::RECORD_ROW,
    );
    let row_parameter = env.store.intern_type_parameter(row_parameter);
    env.resolver.insert_record_row_binding("R", row_parameter);
    let record_application = TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(reference("Record")),
            arguments: vec![phalcom_ast::ast::GenericArgumentSyntax::Expansion(phalcom_ast::ast::TypePackExpansionSyntax {
                mode: phalcom_ast::ast::ExpansionMode::Labeled,
                operator_range: RANGE,
                operand: reference("R"),
                range: RANGE,
            })],
            range: RANGE,
        },
        range: RANGE,
    };
    let scoped_record = TypeAnnotation {
        expr: TypeAnnotationExpr::TypeLambda {
            parameters: vec![phalcom_ast::ast::TypeLambdaParameter {
                name: "T".into(),
                name_range: RANGE,
                kind: None,
                range: RANGE,
            }],
            body: Box::new(record_application),
            range: RANGE,
        },
        range: RANGE,
    };
    diagnostics.clear();
    let TypeFormResolution::Ready(record_lambda) =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &scoped_record, &mut diagnostics)
    else {
        panic!("scoped zero Record should lower: {diagnostics:?}");
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let TypeData::Lambda(record_lambda_id) = env.store.get(record_lambda) else {
        panic!("scoped Record application should remain a type lambda");
    };
    assert!(matches!(
        env.store.arena().get_scoped(env.store.arena().get_lambda(*record_lambda_id).body),
        ScopedTypeData::OpenRecord(ScopedOpenRecord { fields, tail: ScopedRecordTail::FreeParameter(parameter) })
            if fields.is_empty() && *parameter == row_parameter
    ));
}

#[test]
fn intrinsic_record_application_keeps_the_record_row_identity_outside_type_id() {
    let mut env = setup();
    let record = DeclarationId::new(env.module.clone(), "Record".into());
    let mut record_binding = env.declarations.get(&record).expect("canonical Record binding").clone();
    record_binding.binding.runtime_owner = None;
    env.declarations.remove(&record);
    env.declarations.insert(record_binding);
    assert_eq!(env.declarations.intrinsic_runtime_owner(&record), None);
    env.resolver.insert("Record", record);

    let empty_application = TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(reference("Record")),
            arguments: vec![],
            range: RANGE,
        },
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let TypeFormResolution::Ready(empty_record) = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &empty_application,
        &mut diagnostics,
    ) else {
        panic!("Record<> should resolve as the empty Record type: {diagnostics:?}");
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_ne!(empty_record, env.store.unit(), "empty Record and Unit retain distinct type identity");
    assert!(matches!(env.store.get(empty_record), TypeData::Record(row) if {
        let row = env.store.record_row(*row);
        row.fields.is_empty() && row.tail == RecordRowTail::Closed
    }));

    let owner = TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "RecordAlias".into()));
    let parameter = env.store.intern_type_parameter(TypeParameterData::new(owner, 0, "R", KindId::RECORD_ROW));
    env.resolver.insert_record_row_binding("R", parameter);
    let application = TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(reference("Record")),
            arguments: vec![phalcom_ast::ast::GenericArgumentSyntax::Expansion(phalcom_ast::ast::TypePackExpansionSyntax {
                mode: phalcom_ast::ast::ExpansionMode::Labeled,
                operator_range: RANGE,
                operand: reference("R"),
                range: RANGE,
            })],
            range: RANGE,
        },
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let TypeFormResolution::Ready(applied) =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &application, &mut diagnostics)
    else {
        panic!("expected intrinsic Record application to resolve")
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let TypeData::Record(row) = env.store.get(applied) else {
        panic!("Record<**R> must normalize to its structural RecordRowId")
    };
    assert_eq!(env.store.record_row(*row).tail, RecordRowTail::Parameter(parameter));

    let record_shape = TypeAnnotation {
        expr: TypeAnnotationExpr::Record {
            fields: vec![phalcom_ast::ast::RecordTypeField::symbol("name", reference("String"), false, RANGE)],
            tail: None,
            range: RANGE,
        },
        range: RANGE,
    };
    let explicit_shape = TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(reference("Record")),
            arguments: vec![phalcom_ast::ast::GenericArgumentSyntax::Expansion(phalcom_ast::ast::TypePackExpansionSyntax {
                mode: phalcom_ast::ast::ExpansionMode::Labeled,
                operator_range: RANGE,
                operand: record_shape,
                range: RANGE,
            })],
            range: RANGE,
        },
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let TypeFormResolution::Ready(explicit_shape) =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &explicit_shape, &mut diagnostics)
    else {
        panic!("Record<**{{ name: String }}> should resolve: {diagnostics:?}");
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(matches!(env.store.get(explicit_shape), TypeData::Record(row) if {
        let row = env.store.record_row(*row);
        row.fields.len() == 1 && row.fields[0].key.as_symbol().expect("symbol row field") == "name" && row.fields[0].ty == env.declarations.form(&DeclarationId::new(env.module.clone(), "String".into())).unwrap()
    }));
}

#[test]
fn record_fields_applications_require_expansion_and_preserve_row_arguments() {
    let mut env = setup();
    let family = DeclarationId::new(env.module.clone(), "RecordFamily".into());
    let family_parameter = env.store.intern_type_parameter(
        TypeParameterData::new(TypeParameterOwner::Declaration(family.clone()), 0, "Fs", KindId::RECORD_ROW)
            .with_mode(phalcom_semantic::types::parameter::GenericParameterMode::LabeledPack),
    );
    let signature = GenericSignature::new(TypeParameterOwner::Declaration(family.clone()), vec![family_parameter].into_boxed_slice())
        .with_parameter_modes(vec![phalcom_semantic::types::parameter::GenericParameterMode::LabeledPack].into_boxed_slice());
    let kind = env.store.arrow_kind(vec![KindId::RECORD_ROW].into_boxed_slice(), KindId::TYPE);
    let form = env.store.nominal_form(family.clone(), kind);
    env.declarations.insert(phalcom_semantic::declarations::DeclarationTypeInfo {
        declaration: family.clone(),
        form,
        class_object_type: env.store.class_object_type(family.clone()),
        kind,
        binding: phalcom_semantic::declarations::DeclarationTypeBinding::ordinary(),
        generic_binder_header: Some(phalcom_semantic::types::parameter::GenericBinderHeader::from_signature(&env.store, &signature)),
        generic_signature: Some(signature),
        supertype_template: None,
    });
    env.resolver.insert("RecordFamily", family.clone());

    let row_owner = TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "RowAlias".into()));
    let row_parameter = env.store.intern_type_parameter(TypeParameterData::new(row_owner, 0, "Fs", KindId::RECORD_ROW));
    env.resolver.insert_record_row_binding("Fs", row_parameter);
    let bound_expansion = TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(reference("RecordFamily")),
            arguments: vec![phalcom_ast::ast::GenericArgumentSyntax::Expansion(phalcom_ast::ast::TypePackExpansionSyntax {
                mode: phalcom_ast::ast::ExpansionMode::Labeled,
                operator_range: RANGE,
                operand: reference("Fs"),
                range: RANGE,
            })],
            range: RANGE,
        },
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let TypeFormResolution::Ready(applied) =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &bound_expansion, &mut diagnostics)
    else {
        panic!("bound RecordFields expansion should resolve: {diagnostics:?}");
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let TypeData::Applied { arguments, .. } = env.store.get(applied) else {
        panic!("custom RecordFields family should retain its nominal application")
    };
    let phalcom_semantic::types::store::StaticArgument::RecordRow(row) = arguments[0] else {
        panic!("RecordFields must remain a row argument")
    };
    assert_eq!(env.store.record_row(row).tail, RecordRowTail::Parameter(row_parameter));

    let row_expansion_application = TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(reference("RecordFamily")),
            arguments: vec![phalcom_ast::ast::GenericArgumentSyntax::Expansion(phalcom_ast::ast::TypePackExpansionSyntax {
                mode: phalcom_ast::ast::ExpansionMode::Labeled,
                operator_range: RANGE,
                operand: reference("Fs"),
                range: RANGE,
            })],
            range: RANGE,
        },
        range: RANGE,
    };
    let scoped_row_application = type_lambda(&["T"], row_expansion_application);
    diagnostics.clear();
    let scoped_result = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &scoped_row_application,
        &mut diagnostics,
    );
    let TypeFormResolution::Ready(scoped_lambda) = scoped_result else {
        panic!("scoped generic RecordFields expansion result {scoped_result:?}: {diagnostics:?}");
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let TypeData::Lambda(scoped_lambda_id) = env.store.get(scoped_lambda) else {
        panic!("generic RecordFields application should remain scoped in the enclosing type lambda");
    };
    assert!(matches!(
        env.store.arena().get_scoped(env.store.arena().get_lambda(*scoped_lambda_id).body),
        ScopedTypeData::Applied { arguments, .. }
            if matches!(env.store.arena().get_scoped(arguments[0]), ScopedTypeData::RecordRow(row)
                if row.fields.is_empty() && row.tail == ScopedRecordRowTail::FreeParameter(row_parameter))
    ));

    let scoped_empty_application = type_lambda(&["T"], application(reference("RecordFamily"), vec![]));
    diagnostics.clear();
    let TypeFormResolution::Ready(scoped_empty_lambda) = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &scoped_empty_application,
        &mut diagnostics,
    ) else {
        panic!("scoped zero RecordFields contribution should select a closed-empty row: {diagnostics:?}");
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let TypeData::Lambda(scoped_empty_lambda_id) = env.store.get(scoped_empty_lambda) else {
        panic!("zero RecordFields application should remain scoped in the enclosing type lambda");
    };
    assert!(matches!(
        env.store.arena().get_scoped(env.store.arena().get_lambda(*scoped_empty_lambda_id).body),
        ScopedTypeData::Applied { arguments, .. }
            if matches!(env.store.arena().get_scoped(arguments[0]), ScopedTypeData::RecordRow(row)
                if row.fields.is_empty() && row.tail == ScopedRecordRowTail::Closed)
    ));
    let int = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let canonical_zero = env.store.apply_type_form(scoped_empty_lambda, &[int]).unwrap();
    assert!(matches!(env.store.get(canonical_zero), TypeData::Applied { arguments, .. }
        if matches!(arguments[0], StaticArgument::RecordRow(row)
            if env.store.record_row(row).fields.is_empty() && env.store.record_row(row).tail == RecordRowTail::Closed)));

    let direct = application(reference("RecordFamily"), vec![reference("Int")]);
    diagnostics.clear();
    assert!(matches!(
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &direct, &mut diagnostics,),
        TypeFormResolution::Invalid(_)
    ));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::AnnotationUnsupported));
}

#[test]
fn scoped_record_row_type_fields_substitute_and_alpha_normalize() {
    use phalcom_semantic::metadata::MetadataExporter;
    use phalcom_type_meta::encode::{decode_metadata_json, encode_metadata_json};
    use phalcom_type_meta::header::MetadataProfile;
    use phalcom_type_meta::kind::KindNode;
    use phalcom_type_meta::scoped_type::{ScopedRecordRowTailRef, ScopedTypeNode};
    use phalcom_type_meta::type_node::TypeNode;
    use phalcom_type_meta::validate::{ValidationLimits, validate_metadata_bundle};

    let mut env = setup();
    let family = DeclarationId::new(env.module.clone(), "ScopedRecordFamily".into());
    let parameter = env.store.intern_type_parameter(
        TypeParameterData::new(TypeParameterOwner::Declaration(family.clone()), 0, "Fs", KindId::RECORD_ROW).with_mode(GenericParameterMode::LabeledPack),
    );
    let signature = GenericSignature::new(TypeParameterOwner::Declaration(family.clone()), vec![parameter].into_boxed_slice())
        .with_parameter_modes(vec![GenericParameterMode::LabeledPack].into_boxed_slice());
    let kind = env.store.arrow_kind(vec![KindId::RECORD_ROW].into_boxed_slice(), KindId::TYPE);
    let form = env.store.nominal_form(family.clone(), kind);
    env.declarations.insert(phalcom_semantic::declarations::DeclarationTypeInfo {
        declaration: family.clone(),
        form,
        class_object_type: env.store.class_object_type(family.clone()),
        kind,
        binding: phalcom_semantic::declarations::DeclarationTypeBinding::ordinary(),
        generic_binder_header: Some(phalcom_semantic::types::parameter::GenericBinderHeader::from_signature(&env.store, &signature)),
        generic_signature: Some(signature),
        supertype_template: None,
    });
    env.resolver.insert("ScopedRecordFamily", family);

    let row_application = |parameter: &str| TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(reference("ScopedRecordFamily")),
            arguments: vec![phalcom_ast::ast::GenericArgumentSyntax::Expansion(phalcom_ast::ast::TypePackExpansionSyntax {
                mode: phalcom_ast::ast::ExpansionMode::Labeled,
                operator_range: RANGE,
                operand: TypeAnnotation {
                    expr: TypeAnnotationExpr::Record {
                        fields: vec![phalcom_ast::ast::RecordTypeField::symbol("payload", reference(parameter), false, RANGE)],
                        tail: None,
                        range: RANGE,
                    },
                    range: RANGE,
                },
                range: RANGE,
            })],
            range: RANGE,
        },
        range: RANGE,
    };

    let mut diagnostics = Vec::new();
    let first = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &type_lambda(&["T"], row_application("T")),
        &mut diagnostics,
    );
    let second = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &type_lambda(&["U"], row_application("U")),
        &mut diagnostics,
    );
    let TypeFormResolution::Ready(first) = first else {
        panic!("expected first lambda: {diagnostics:?}")
    };
    let TypeFormResolution::Ready(second) = second else {
        panic!("expected alpha-renamed lambda: {diagnostics:?}")
    };
    assert_eq!(first, second, "row-valued arguments must participate in alpha-normalized interning");
    assert!(diagnostics.is_empty(), "{diagnostics:?}");

    let int = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let applied = env.store.apply_type_form(first, &[int]).unwrap();
    let TypeData::Applied { arguments, .. } = env.store.get(applied) else {
        panic!("expected nominal family application")
    };
    let StaticArgument::RecordRow(row) = arguments[0] else {
        panic!("RecordFields argument must remain a row")
    };
    let row = env.store.record_row(row);
    assert_eq!(row.tail, RecordRowTail::Closed);
    assert_eq!(row.fields.len(), 1);
    assert_eq!(row.fields[0].key.as_symbol().expect("symbol row field"), "payload");
    assert_eq!(
        row.fields[0].ty, int,
        "the enclosing Type binder must substitute inside the row-valued argument"
    );

    let mut exporter = MetadataExporter::new(&env.store, Some(&env.declarations), None, None, MetadataProfile::RuntimePublic);
    let lambda_id = exporter.export_type_form(first).expect("export scoped RecordFields application");
    let bundle = exporter.build_complete_test_bundle(&[]).expect("build scoped row metadata");
    let TypeNode::TypeLambda(lambda) = &bundle.types[lambda_id.0 as usize].form else {
        panic!("expected exported type lambda");
    };
    let ScopedTypeNode::Applied { arguments, .. } = &bundle.scoped_types[lambda.body.0 as usize].form else {
        panic!("expected exported scoped Applied form");
    };
    let [row_argument] = arguments.as_ref() else {
        panic!("expected one RecordFields argument")
    };
    let ScopedTypeNode::RecordRow(row) = &bundle.scoped_types[row_argument.0 as usize].form else {
        panic!("RecordFields argument must export as scoped RecordRow")
    };
    assert!(matches!(
        bundle.kinds[bundle.scoped_types[row_argument.0 as usize].kind.0 as usize].node,
        KindNode::RecordRow
    ));
    assert_eq!(row.fields.len(), 1);
    assert_eq!(row.fields[0].key, phalcom_type_meta::type_node::RecordKeyRef::Symbol("payload".into()));
    assert!(matches!(
        bundle.scoped_types[row.fields[0].ty.0 as usize].form,
        ScopedTypeNode::Bound { depth: 0, index: 0 }
    ));
    assert_eq!(row.tail, ScopedRecordRowTailRef::Closed);
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).expect("exported scoped row metadata validates");
    let encoded = encode_metadata_json(&bundle).expect("encode scoped row metadata");
    assert_eq!(decode_metadata_json(&encoded, &ValidationLimits::default()).unwrap(), bundle);
}

#[test]
fn scoped_type_lambda_record_fields_binders_remain_unsupported() {
    let mut env = setup();
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &typed_type_lambda(vec![("Fs", KindSyntax::RecordFields(RANGE))], reference("Fs")),
        &mut diagnostics,
    );
    assert!(matches!(result, TypeFormResolution::Invalid(_)));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::AnnotationUnsupported));
}

#[test]
fn generic_constraint_substitution_preserves_parameter_identity() {
    let mut env = setup();
    let owner = TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "Constrained".into()));
    let params = vec![phalcom_ast::ast::GenericParameterSyntax {
        mode: None,
        mode_range: None,
        variance: phalcom_ast::ast::VarianceSyntax::Invariant,
        name: "T".into(),
        name_range: RANGE,
        kind: None,
        range: RANGE,
    }];
    let where_clause = phalcom_ast::ast::WhereClauseSyntax {
        constraints: vec![phalcom_ast::ast::GenericConstraintSyntax::Subtype {
            lower: reference("T"),
            upper: reference("Object"),
            range: RANGE,
        }],
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let phalcom_semantic::types::annotation::TypeFormationOutcome::Ready(signature) = phalcom_semantic::types::annotation::resolve_generic_signature(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        owner,
        phalcom_semantic::types::annotation::GenericBinderSite::NominalDeclaration,
        &params,
        Some(&where_clause),
        &mut diagnostics,
    ) else {
        panic!("expected constrained generic signature")
    };
    let parameter = signature.parameter_at(0).expect("generic parameter");
    let parameter_form = env.store.parameter_form(parameter);
    let int_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let mut substitution = TypeSubstitution::new();
    substitution.bind(parameter, int_ty);
    let GenericConstraint::Subtype { lower, upper } = signature.constraint_at(0).expect("subtype constraint") else {
        panic!("expected subtype constraint")
    };
    let TypeTerm::Canonical(lower) = lower else {
        panic!("constraint lower must be canonical")
    };
    let TypeTerm::Canonical(upper) = upper else {
        panic!("constraint upper must be canonical")
    };
    assert_eq!(*lower, parameter_form);
    assert_eq!(substitution.apply(&mut env.store, *lower), int_ty);
    assert_eq!(substitution.apply(&mut env.store, *upper), *upper);
    assert!(!matches!(env.store.get(int_ty), TypeData::Parameter(_)));
}

#[test]
fn solver_inference_terms_never_publish_in_generic_signature() {
    let mut env = setup();
    let owner = TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "SolverLeak".into()));
    let parameter = env.store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "T", KindId::TYPE));
    let int_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let signature = GenericSignature::with_constraints(
        owner,
        vec![parameter].into_boxed_slice(),
        vec![GenericConstraint::Equivalent {
            left: TypeTerm::Infer(InferVarId(7)),
            right: TypeTerm::Canonical(int_ty),
        }]
        .into_boxed_slice(),
    );
    assert!(matches!(
        signature.validate_publishable(&env.store),
        Err(phalcom_semantic::types::parameter::GenericSignaturePublicationError::InferenceVariable { variable: InferVarId(7) })
    ));
}

#[test]
fn invalid_generic_kind_does_not_publish_signature() {
    let mut env = setup();
    let owner = phalcom_semantic::types::parameter::TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "Broken".into()));
    let params = vec![phalcom_ast::ast::GenericParameterSyntax {
        mode: None,
        mode_range: None,
        variance: phalcom_ast::ast::VarianceSyntax::Invariant,
        name: "T".into(),
        name_range: RANGE,
        kind: Some(KindSyntax::Invalid {
            message: "recovered kind".into(),
            range: RANGE,
        }),
        range: RANGE,
    }];
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_generic_signature(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        owner,
        phalcom_semantic::types::annotation::GenericBinderSite::NominalDeclaration,
        &params,
        None,
        &mut diagnostics,
    );
    assert!(matches!(
        result,
        phalcom_semantic::types::annotation::TypeFormationOutcome::Invalid(TypeFormationInvalid::InvalidKindSyntax)
    ));
}

#[test]
fn malformed_generic_constraint_does_not_publish_partial_signature() {
    let mut env = setup();
    let owner = phalcom_semantic::types::parameter::TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "BrokenConstraint".into()));
    let params = vec![phalcom_ast::ast::GenericParameterSyntax {
        mode: None,
        mode_range: None,
        variance: phalcom_ast::ast::VarianceSyntax::Invariant,
        name: "T".into(),
        name_range: RANGE,
        kind: None,
        range: RANGE,
    }];
    let where_clause = phalcom_ast::ast::WhereClauseSyntax {
        constraints: vec![phalcom_ast::ast::GenericConstraintSyntax::Invalid {
            message: "recovered constraint".into(),
            range: RANGE,
        }],
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_generic_signature(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        owner,
        phalcom_semantic::types::annotation::GenericBinderSite::NominalDeclaration,
        &params,
        Some(&where_clause),
        &mut diagnostics,
    );
    assert!(matches!(
        result,
        phalcom_semantic::types::annotation::TypeFormationOutcome::Invalid(TypeFormationInvalid::Syntax)
    ));
    assert!(!diagnostics.is_empty());
}

#[test]
fn callable_generic_variance_is_rejected() {
    let mut env = setup();
    let owner = phalcom_semantic::types::parameter::TypeParameterOwner::Callable(CallableId::new(
        DeclarationId::new(env.module.clone(), "Owner".into()),
        Selector::method("call", []).unwrap(),
        DispatchSide::Instance,
    ));
    let params = vec![phalcom_ast::ast::GenericParameterSyntax {
        mode: None,
        mode_range: None,
        variance: phalcom_ast::ast::VarianceSyntax::Covariant,
        name: "T".into(),
        name_range: RANGE,
        kind: None,
        range: RANGE,
    }];
    let mut diagnostics = Vec::new();
    let result = phalcom_semantic::types::annotation::resolve_generic_signature(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        owner,
        phalcom_semantic::types::annotation::GenericBinderSite::Callable,
        &params,
        None,
        &mut diagnostics,
    );
    assert!(matches!(
        result,
        phalcom_semantic::types::annotation::TypeFormationOutcome::Invalid(TypeFormationInvalid::InvalidVariance)
    ));
    assert!(!diagnostics.is_empty());
}

#[test]
fn callable_domain_pack_expansion_and_concrete_collapse() {
    let mut env = setup();
    let mut diagnostics = Vec::new();

    // 1. Concrete expansion collapses to standard CallableType
    let int_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let string_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "String".into())).unwrap();
    let bool_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Bool".into())).unwrap();
    let pack = env.store.type_pack(vec![int_ty, string_ty].into_boxed_slice()).unwrap();
    let p_annot = reference("Pack");

    let mut resolver = env.resolver.clone();
    resolver.insert_parameter("Pack", pack);

    let callable_expr = TypeAnnotation {
        expr: TypeAnnotationExpr::Callable {
            parameters: vec![
                phalcom_ast::ast::TypeCallableItem::Expansion(phalcom_ast::ast::TypePackExpansionSyntax {
                    mode: phalcom_ast::ast::ExpansionMode::Positional,
                    operator_range: RANGE,
                    operand: p_annot,
                    range: RANGE,
                }),
                phalcom_ast::ast::TypeCallableItem::Parameter(phalcom_ast::ast::TypeCallableParameter {
                    label: Some("flag".into()),
                    ty: reference("Bool"),
                    optional: false,
                    rest: false,
                    range: RANGE,
                }),
            ],
            result: Box::new(reference("Bool")),
            range: RANGE,
        },
        range: RANGE,
    };

    let res = phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &resolver, &env.site, &callable_expr, &mut diagnostics);

    let TypeFormResolution::Ready(callable_ty) = res else {
        panic!("expected callable resolution success, got {res:?}");
    };
    match env.store.get(callable_ty) {
        TypeData::Callable(call) => {
            assert_eq!(call.parameters.len(), 3);
            assert_eq!(call.parameters[0].ty, int_ty);
            assert_eq!(call.parameters[0].label, None);
            assert_eq!(call.parameters[1].ty, string_ty);
            assert_eq!(call.parameters[1].label, None);
            assert_eq!(call.parameters[2].ty, bool_ty);
            assert_eq!(call.parameters[2].label.as_deref(), Some("flag"));
            assert_eq!(call.return_type, bool_ty);
        }
        other => panic!("expected TypeData::Callable, got {other:?}"),
    }
}

#[test]
fn callable_domain_symbolic_pack_expansion_produces_callable_from_packs() {
    let mut env = setup();
    let mut diagnostics = Vec::new();

    let owner = phalcom_semantic::types::parameter::TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "Owner".into()));
    let param_data =
        TypeParameterData::new(owner, 0, "Ts", KindId::TYPE_PACK).with_mode(phalcom_semantic::types::parameter::GenericParameterMode::PositionalPack);
    let param_id = env.store.intern_type_parameter(param_data);
    let param_ty = env.store.parameter_form(param_id);

    let mut resolver = env.resolver.clone();
    resolver.insert_parameter("Ts", param_ty);

    let callable_expr = TypeAnnotation {
        expr: TypeAnnotationExpr::Callable {
            parameters: vec![phalcom_ast::ast::TypeCallableItem::Expansion(phalcom_ast::ast::TypePackExpansionSyntax {
                mode: phalcom_ast::ast::ExpansionMode::Positional,
                operator_range: RANGE,
                operand: reference("Ts"),
                range: RANGE,
            })],
            result: Box::new(reference("Bool")),
            range: RANGE,
        },
        range: RANGE,
    };

    let res = phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &resolver, &env.site, &callable_expr, &mut diagnostics);

    let TypeFormResolution::Ready(callable_ty) = res else {
        panic!("expected callable resolution success, got {res:?}");
    };
    match env.store.get(callable_ty) {
        TypeData::CallableFromPacks {
            positionals,
            labeled,
            return_type,
        } => {
            assert_eq!(*positionals, param_ty);
            assert_eq!(env.store.kind_of(*positionals), KindId::TYPE_PACK);
            assert_eq!(env.store.kind_of(*labeled), KindId::LABELED_TYPE_PACK);
            let bool_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Bool".into())).unwrap();
            assert_eq!(*return_type, bool_ty);
        }
        other => panic!("expected TypeData::CallableFromPacks, got {other:?}"),
    }

    // Now test try_apply substitution
    let int_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let concrete_pack = env.store.type_pack(vec![int_ty].into_boxed_slice()).unwrap();
    let mut subst = TypeSubstitution::default();
    subst.bind(param_id, concrete_pack);
    let substituted = subst.try_apply(&mut env.store, callable_ty).unwrap();
    match env.store.get(substituted) {
        TypeData::Callable(call) => {
            assert_eq!(call.parameters.len(), 1);
            assert_eq!(call.parameters[0].ty, int_ty);
        }
        other => panic!("expected substitution to collapse to TypeData::Callable, got {other:?}"),
    }
}

#[test]
fn cv10_list_dynamic_formation_preserves_outer_applied_type_and_differs_from_any() {
    let mut env = setup();
    let mut diagnostics = Vec::new();

    let dynamic_annotation = TypeAnnotation {
        expr: TypeAnnotationExpr::Dynamic { range: RANGE },
        range: RANGE,
    };
    let list_dynamic_expr = application(reference("List"), vec![dynamic_annotation.clone()]);
    let res = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &list_dynamic_expr,
        &mut diagnostics,
    );
    let TypeFormResolution::Ready(list_dynamic_ty) = res else {
        panic!("List<Dynamic> should resolve as ready applied type: {diagnostics:?}");
    };

    let list_obj_expr = application(reference("List"), vec![reference("Object")]);
    let TypeFormResolution::Ready(list_obj_ty) =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &list_obj_expr, &mut diagnostics)
    else {
        panic!("List<Object> should resolve: {diagnostics:?}");
    };

    assert_ne!(list_dynamic_ty, list_obj_ty, "List<Dynamic> and List<Object> must be distinct TypeIds");

    let list_decl = DeclarationId::new(env.module.clone(), "List".into());
    let list_form = env.declarations.form(&list_decl).unwrap();

    let TypeData::Applied { origin, arguments } = env.store.get(list_dynamic_ty).clone() else {
        panic!("expected TypeData::Applied for List<Dynamic>");
    };
    assert_eq!(origin, list_form);
    assert_eq!(arguments.as_ref(), &[StaticArgument::DynamicType]);
}

#[test]
fn cv11_dynamic_argument_rejected_for_non_type_kind_parameter() {
    let mut env = setup();
    let mut diagnostics = Vec::new();

    let row_family_decl = DeclarationId::new(env.module.clone(), "RowFamily".into());
    let row_param = env.store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(row_family_decl.clone()),
        0,
        "Fs",
        KindId::RECORD_ROW,
    ));
    let signature = GenericSignature::new(TypeParameterOwner::Declaration(row_family_decl.clone()), vec![row_param].into_boxed_slice());
    let constructor_kind = env.store.arrow_kind(vec![KindId::RECORD_ROW].into_boxed_slice(), KindId::TYPE);
    let constructor = env.store.nominal_form(row_family_decl.clone(), constructor_kind);
    env.declarations.insert(phalcom_semantic::declarations::DeclarationTypeInfo {
        declaration: row_family_decl.clone(),
        form: constructor,
        class_object_type: env.store.class_object_type(row_family_decl.clone()),
        kind: constructor_kind,
        binding: phalcom_semantic::declarations::DeclarationTypeBinding::ordinary(),
        generic_binder_header: Some(phalcom_semantic::types::parameter::GenericBinderHeader::from_signature(&env.store, &signature)),
        generic_signature: Some(signature),
        supertype_template: None,
    });
    env.resolver.insert("RowFamily", row_family_decl);

    let dynamic_annotation = TypeAnnotation {
        expr: TypeAnnotationExpr::Dynamic { range: RANGE },
        range: RANGE,
    };
    let invalid_application = application(reference("RowFamily"), vec![dynamic_annotation]);
    let res = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &invalid_application,
        &mut diagnostics,
    );
    assert!(matches!(res, TypeFormResolution::Invalid(TypeFormationInvalid::TypeArgumentKindMismatch)));
}

#[test]
fn cv12_to_cv14_knowledge_bearing_use_site_materialization() {
    let mut env = setup();

    let owner = TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "TestHolder".into()));
    let param_id = env.store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "T", KindId::TYPE));
    let param_ty = env.store.parameter_form(param_id);

    let mut environment = phalcom_semantic::types::environment::TypeEnvironment::new();
    environment.bind_dynamic(param_id).unwrap();

    // CV-12: Bare T [T := Dynamic] -> TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape)
    let bare_view = phalcom_semantic::types::environment::TypeView::new(param_ty, environment.clone());
    let bare_knowledge = bare_view.materialize_knowledge(&mut env.store);
    assert!(matches!(
        bare_knowledge,
        TypeKnowledge::Dynamic(phalcom_semantic::types::evidence::DynamicReason::ExplicitEscape)
    ));

    // CV-13: Nested List<T> [T := Dynamic] -> TypeKnowledge::assumed(List<Dynamic>)
    let list_decl = DeclarationId::new(env.module.clone(), "List".into());
    let list_form = env.declarations.form(&list_decl).unwrap();
    let applied_t = env.store.apply_type_form(list_form, &[param_ty]).unwrap();

    let nested_view = phalcom_semantic::types::environment::TypeView::new(applied_t, environment.clone());
    let nested_knowledge = nested_view.materialize_knowledge(&mut env.store);
    let TypeKnowledge::Known(ev) = nested_knowledge else {
        panic!("expected Known(List<Dynamic>), got {nested_knowledge:?}");
    };
    let TypeData::Applied { origin, arguments } = env.store.get(ev.ty()).clone() else {
        panic!("expected TypeData::Applied");
    };
    assert_eq!(origin, list_form);
    assert_eq!(arguments.as_ref(), &[StaticArgument::DynamicType]);

    // CV-14: Fixed Int [T := Dynamic] -> TypeKnowledge::assumed(Int)
    let int_decl = DeclarationId::new(env.module.clone(), "Int".into());
    let int_form = env.declarations.form(&int_decl).unwrap();

    let fixed_view = phalcom_semantic::types::environment::TypeView::new(int_form, environment);
    let fixed_knowledge = fixed_view.materialize_knowledge(&mut env.store);
    let TypeKnowledge::Known(fixed_ev) = fixed_knowledge else {
        panic!("expected Known(Int), got {fixed_knowledge:?}");
    };
    assert_eq!(fixed_ev.ty(), int_form);
}

#[test]
fn pack_type_remainders_form_canonical_pack_types() {
    use phalcom_ast::ast::{AnonymousRemainderSyntax, ExpansionMode};
    use phalcom_semantic::types::store::AnonymousTupleRemainder as Remainder;

    let mut env = setup();
    let int = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let string = env.declarations.form(&DeclarationId::new(env.module.clone(), "String".into())).unwrap();
    let bool_ty = env.declarations.form(&DeclarationId::new(env.module.clone(), "Bool".into())).unwrap();

    let cases = [
        (ExpansionMode::Positional, Remainder::Typed(int), Remainder::None),
        (ExpansionMode::Labeled, Remainder::None, Remainder::Typed(string)),
        (ExpansionMode::Complete, Remainder::Typed(bool_ty), Remainder::Typed(bool_ty)),
    ];
    for (mode, positional, labeled) in cases {
        let annotation = TypeAnnotation {
            expr: TypeAnnotationExpr::Application {
                origin: Box::new(reference("Pack")),
                arguments: vec![pack_remainder(
                    mode,
                    AnonymousRemainderSyntax::Typed {
                        constraint: Box::new(reference(match mode {
                            ExpansionMode::Positional => "Int",
                            ExpansionMode::Labeled => "String",
                            ExpansionMode::Complete => "Bool",
                        })),
                        range: RANGE,
                    },
                )],
                range: RANGE,
            },
            range: RANGE,
        };
        let (knowledge, diagnostics) = resolve(&mut env, &annotation);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let actual = knowledge.ty().expect("proper Pack type");
        let expected = env.store.open_pack(Box::new([]), positional, labeled).unwrap();
        assert_eq!(actual, expected, "unexpected Pack remainder for {mode:?}");
    }
}

#[test]
fn pack_type_remainder_constraints_substitute_through_type_lambdas() {
    use phalcom_ast::ast::{AnonymousRemainderSyntax, ExpansionMode};
    use phalcom_semantic::types::store::AnonymousTupleRemainder as Remainder;

    let mut env = setup();
    let body = TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(reference("Pack")),
            arguments: vec![pack_remainder(
                ExpansionMode::Positional,
                AnonymousRemainderSyntax::Typed {
                    constraint: Box::new(reference("T")),
                    range: RANGE,
                },
            )],
            range: RANGE,
        },
        range: RANGE,
    };
    let lambda = typed_type_lambda(vec![("T", KindSyntax::Type(RANGE))], body);
    let mut diagnostics = Vec::new();
    let TypeFormResolution::Ready(lambda_ty) =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &lambda, &mut diagnostics)
    else {
        panic!("expected a Pack type lambda: {diagnostics:?}");
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let TypeData::Lambda(lambda_id) = env.store.get(lambda_ty) else {
        panic!("expected Pack type lambda form");
    };
    let ScopedTypeData::Pack(scoped_pack) = env.store.arena().get_scoped(env.store.arena().get_lambda(*lambda_id).body) else {
        panic!("Pack remainder must remain in the scoped Pack node until substitution");
    };
    assert!(matches!(
        scoped_pack.positional_remainder,
        phalcom_semantic::types::type_lambda::ScopedPackRemainder::Typed(constraint)
            if matches!(env.store.arena().get_scoped(constraint), ScopedTypeData::Bound { depth: 0, index: 0 })
    ));

    let int = env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap();
    let applied = env.store.apply_type_form(lambda_ty, &[int]).expect("substitute Pack remainder type");
    let expected = env.store.open_pack(Box::new([]), Remainder::Typed(int), Remainder::None).unwrap();
    assert_eq!(applied, expected);
}

#[test]
fn pack_type_unconstrained_remainders_are_retained_and_other_constructors_reject_them() {
    use phalcom_ast::ast::{AnonymousRemainderSyntax, ExpansionMode};
    use phalcom_semantic::types::store::AnonymousTupleRemainder as Remainder;

    let mut env = setup();
    for (mode, positional, labeled) in [
        (ExpansionMode::Positional, Remainder::Unconstrained, Remainder::None),
        (ExpansionMode::Labeled, Remainder::None, Remainder::Unconstrained),
        (ExpansionMode::Complete, Remainder::Unconstrained, Remainder::Unconstrained),
    ] {
        let annotation = TypeAnnotation {
            expr: TypeAnnotationExpr::Application {
                origin: Box::new(reference("Pack")),
                arguments: vec![pack_remainder(mode, AnonymousRemainderSyntax::Unconstrained(RANGE))],
                range: RANGE,
            },
            range: RANGE,
        };
        let (knowledge, diagnostics) = resolve(&mut env, &annotation);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let expected = env.store.open_pack(Box::new([]), positional, labeled).unwrap();
        assert_eq!(knowledge.ty(), Some(expected));
    }

    let unsupported = TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(reference("List")),
            arguments: vec![pack_remainder(
                ExpansionMode::Positional,
                AnonymousRemainderSyntax::Typed {
                    constraint: Box::new(reference("Int")),
                    range: RANGE,
                },
            )],
            range: RANGE,
        },
        range: RANGE,
    };
    let mut diagnostics = Vec::new();
    let result =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &unsupported, &mut diagnostics);
    assert!(matches!(result, TypeFormResolution::Invalid(_)));
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::AnnotationUnsupported),
        "{diagnostics:?}"
    );
}

#[test]
fn rest_parameter_lane_projection_and_kind_enforcement() {
    use crate::semantic::support::Fixture;
    use phalcom_semantic::diagnostic::DiagnosticCode;

    // 1. Explicit lane *P lowers to Lane(P)
    let f1 = Fixture::new(
        r#"
class C {
    @class f<*P>(*args: *P) {}
}
"#,
    );
    f1.assert_no_error_diagnostics();

    // 2. Bare *args: P where P: TypePack is rejected with KindExpectedType
    let f2 = Fixture::new(
        r#"
class C {
    @class f<*P>(*args: P) {}
}
"#,
    );
    assert!(
        !f2.diagnostics(DiagnosticCode::KindExpectedType).is_empty(),
        "expected KindExpectedType diagnostic"
    );

    // 3. Concrete tuple lane *args: *(Int, String) projects positionals
    let f3 = Fixture::new(
        r#"
class C {
    @class f(*args: *(Int, String)) {}
}
"#,
    );
    f3.assert_no_error_diagnostics();
}
