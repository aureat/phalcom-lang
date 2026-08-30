use phalcom_ast::ast::{PathSegment, StaticSymbolRef, TypeAnnotation, TypeAnnotationExpr, TypeCallableParameter, TypeTupleElement};
use phalcom_common::range::SourceRange;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::declarations::{DeclarationTypeTable, bootstrap_universe_declarations};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::annotation::{SimpleTypeResolver, TypeFormResolution, resolve_type_annotation};
use phalcom_semantic::types::evidence::TypeKnowledge;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::store::{CallableType, TypeData, TypeStore};

const RANGE: SourceRange = SourceRange { start: 0, end: 1 };

struct TestEnv {
    store: TypeStore,
    declarations: DeclarationTypeTable,
    resolver: SimpleTypeResolver,
    module: ModuleId,
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
            arguments,
            range: RANGE,
        },
        range: RANGE,
    }
}

fn setup() -> TestEnv {
    let mut store = TypeStore::new();
    let module = ModuleId::core();
    let declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(module.clone(), key.name().into()));
    let mut resolver = SimpleTypeResolver::new();
    for name in ["Int", "String", "Bool", "List", "Map", "Object"] {
        resolver.insert(name, DeclarationId::new(module.clone(), name.into()));
    }
    TestEnv {
        store,
        declarations,
        resolver,
        module,
    }
}

fn resolve(env: &mut TestEnv, annotation: &TypeAnnotation) -> (TypeKnowledge, Vec<phalcom_semantic::SemanticDiagnostic>) {
    let mut diagnostics = Vec::new();
    let knowledge = resolve_type_annotation(&mut env.store, &env.declarations, &env.resolver, &env.module, annotation, &mut diagnostics);
    (knowledge, diagnostics)
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
        &[env.declarations.form(&DeclarationId::new(env.module.clone(), "Int".into())).unwrap()]
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
                TypeTupleElement {
                    label: None,
                    ty: reference("Int"),
                    range: RANGE,
                },
                TypeTupleElement {
                    label: Some("name".into()),
                    ty: reference("String"),
                    range: RANGE,
                },
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
                TypeCallableParameter {
                    label: None,
                    ty: reference("Int"),
                    rest: false,
                    range: RANGE,
                },
                TypeCallableParameter {
                    label: Some("names".into()),
                    ty: reference("String"),
                    rest: true,
                    range: RANGE,
                },
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
        phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.module, &dynamic, &mut diagnostics,)
    );
    assert!(diagnostics.is_empty());
    assert_eq!(
        KindId::TYPE,
        env.store.kind_of(env.declarations.form(&DeclarationId::new(env.module, "Int".into())).unwrap())
    );
}

#[test]
fn lowers_primitive_and_self_type_annotations() {
    let mut env = setup();
    let decl = DeclarationId::new(env.module.clone(), "Point".into());
    env.resolver.enclosing_declaration = Some(decl.clone());

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
    assert!(matches!(env.store.get(self_res.ty().unwrap()), TypeData::SelfType(term) if term.owner == decl));
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
    let res = phalcom_semantic::types::annotation::resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.module, &lambda_ann, &mut diags);
    assert!(diags.is_empty());
    let TypeFormResolution::Known(lambda_ty) = res else { panic!() };
    assert!(matches!(env.store.get(lambda_ty), TypeData::Lambda(_)));
}

#[test]
fn lowers_generic_signature_with_where_constraints() {
    let mut env = setup();
    let owner = phalcom_semantic::types::parameter::TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "Container".into()));
    let params = vec![phalcom_ast::ast::GenericParameterSyntax {
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
        &env.module,
        owner,
        &params,
        Some(&where_clause),
        &mut diags,
    );

    assert_eq!(sig.parameter_count(), 1);
    assert_eq!(sig.constraint_count(), 1);
}
