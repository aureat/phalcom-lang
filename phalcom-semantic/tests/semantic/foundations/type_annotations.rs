use phalcom_ast::ast::{KindSyntax, PathSegment, StaticSymbolRef, TypeAnnotation, TypeAnnotationExpr, TypeCallableParameter, TypeTupleElement};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::declarations::{DeclarationTypeTable, bootstrap_universe_declarations};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::types::annotation::{
    KindResolution, SimpleTypeResolver, TypeFormResolution, TypeFormationInvalid, TypeFormationMissing, TypeFormationSite, TypeFormationUnresolved,
    resolve_type_annotation,
};
use phalcom_semantic::types::evidence::{TypeKnowledge, UnknownReason};
use phalcom_semantic::types::id::{InferVarId, KindId, TypeId};
use phalcom_semantic::types::parameter::{GenericConstraint, GenericSignature, TypeParameterData, TypeParameterOwner, TypeTerm};
use phalcom_semantic::types::row::RecordRowTail;
use phalcom_semantic::types::store::{CallableType, TypeData, TypeStore};
use phalcom_semantic::types::substitution::TypeSubstitution;
use phalcom_semantic::types::type_lambda::{ScopedOpenRecord, ScopedRecordField, ScopedRecordTail, ScopedTypeData, TypeLambdaArena};

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
    for name in ["Int", "String", "Bool", "List", "Map", "Object"] {
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
    assert!(matches!(env.store.get(tuple.ty()), TypeData::Tuple(elements) if elements[1].label.as_deref() == Some("name")));

    let callable = TypeAnnotation {
        expr: TypeAnnotationExpr::Callable {
            parameters: vec![
                phalcom_ast::ast::TypeCallableItem::Parameter(TypeCallableParameter {
                    label: None,
                    ty: reference("Int"),
                    rest: false,
                    range: RANGE,
                }),
                phalcom_ast::ast::TypeCallableItem::Parameter(TypeCallableParameter {
                    label: Some("names".into()),
                    ty: reference("String"),
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
                phalcom_ast::ast::RecordTypeField {
                    name: "x".into(),
                    ty: reference("Int"),
                    range: RANGE,
                },
                phalcom_ast::ast::RecordTypeField {
                    name: "y".into(),
                    ty: reference("Int"),
                    range: RANGE,
                },
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
            fields: vec![phalcom_ast::ast::RecordTypeField {
                name: "name".into(),
                ty: reference("String"),
                range: RANGE,
            }],
            tail: None,
            range: RANGE,
        },
        range: RANGE,
    };
    let (closed_knowledge, _) = resolve(&mut env, &closed);
    let closed_type = closed_knowledge.ty().expect("closed record type");

    let open = TypeAnnotation {
        expr: TypeAnnotationExpr::Record {
            fields: vec![phalcom_ast::ast::RecordTypeField {
                name: "name".into(),
                ty: reference("String"),
                range: RANGE,
            }],
            tail: Some(phalcom_ast::ast::RecordRowTail {
                name: "R".into(),
                range: RANGE,
            }),
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
            fields: vec![phalcom_ast::ast::RecordTypeField {
                name: "value".into(),
                ty: reference("Int"),
                range: RANGE,
            }],
            tail: Some(phalcom_ast::ast::RecordRowTail {
                name: "R".into(),
                range: RANGE,
            }),
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
            fields: vec![phalcom_ast::ast::RecordTypeField {
                name: "value".into(),
                ty: reference("T"),
                range: RANGE,
            }],
            tail: Some(phalcom_ast::ast::RecordRowTail {
                name: "R".into(),
                range: RANGE,
            }),
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
            name: "value".into(),
            ty: field,
        }]),
        tail: ScopedRecordTail::Bound { depth: 0, index: 0 },
    }));
    assert!(arena.has_free_bound(open, 0));
    let mut free_types = Vec::new();
    arena.collect_free_types(open, &mut free_types);
    assert_eq!(free_types, vec![TypeId::DUMMY]);
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
    assert_eq!(elements[0].ty, int_ty);
    assert_eq!(elements[1].ty, string_ty);
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
    assert_eq!(elements[0].ty, int_ty);
    assert_eq!(elements[1].ty, string_ty);
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
    let captured_type = env.store.tuple(Box::new([phalcom_semantic::types::store::TupleTypeElement {
        label: None,
        ty: nested_application,
    }]));

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
fn record_row_generic_binder_does_not_create_type_parameter_form() {
    let mut env = setup();
    let owner_decl = DeclarationId::new(env.module.clone(), "RowContainer".into());
    let owner_callable = CallableId::new(owner_decl, Selector::getter("row").unwrap(), DispatchSide::Instance);
    let owner = phalcom_semantic::types::parameter::TypeParameterOwner::Callable(owner_callable);
    let params = vec![phalcom_ast::ast::GenericParameterSyntax {
        mode: None,
        mode_range: None,
        variance: phalcom_ast::ast::VarianceSyntax::Invariant,
        name: "R".into(),
        name_range: RANGE,
        kind: Some(KindSyntax::RecordRow(RANGE)),
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
        ]
    );
    assert_eq!(env.store.type_parameter(signature.parameters[0]).kind, KindId::TYPE_PACK);
    assert_eq!(env.store.type_parameter(signature.parameters[1]).kind, KindId::LABELED_TYPE_PACK);

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

    let applied = application(reference("V"), vec![reference("Int"), reference("String")]);
    let (applied, diagnostics) = resolve(&mut env, &applied);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let TypeData::Applied { arguments, .. } = env.store.get(applied.ty().expect("V<Int, String> proper type")) else {
        panic!("expected binder-aligned application")
    };
    assert!(matches!(env.store.get(arguments[0].as_type_id().expect("type-only argument")), TypeData::TypePack(elements) if elements.len() == 2));
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
    env.resolver.insert("Tuple", tuple);
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
fn intrinsic_record_application_keeps_the_record_row_identity_outside_type_id() {
    let mut env = setup();
    let record = DeclarationId::new(env.module.clone(), "Record".into());
    env.resolver.insert("Record", record);
    let owner = TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "RecordAlias".into()));
    let parameter = env.store.intern_type_parameter(TypeParameterData::new(owner, 0, "R", KindId::RECORD_ROW));
    env.resolver.insert_record_row_binding("R", parameter);
    let application = application(reference("Record"), vec![reference("R")]);
    let mut diagnostics = Vec::new();
    let TypeFormResolution::Ready(applied) =
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &application, &mut diagnostics)
    else {
        panic!("expected intrinsic Record application to resolve")
    };
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let TypeData::Record(row) = env.store.get(applied) else {
        panic!("Record<R> must normalize to its structural RecordRowId")
    };
    assert_eq!(env.store.record_row(*row).tail, RecordRowTail::Parameter(parameter));
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
