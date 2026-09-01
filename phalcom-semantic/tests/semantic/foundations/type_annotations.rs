use phalcom_ast::ast::{KindSyntax, PathSegment, StaticSymbolRef, TypeAnnotation, TypeAnnotationExpr, TypeCallableParameter, TypeTupleElement};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::declarations::{bootstrap_universe_declarations, DeclarationTypeTable};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::types::annotation::{
    resolve_type_annotation, KindResolution, SimpleTypeResolver, TypeFormResolution, TypeFormationInvalid, TypeFormationSite, TypeFormationUnresolved,
};
use phalcom_semantic::types::evidence::{TypeKnowledge, UnknownReason};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::store::{CallableType, TypeData, TypeStore};
use phalcom_semantic::types::type_lambda::ScopedTypeData;

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
            arguments,
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
    let result = phalcom_semantic::types::annotation::resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &annotation,
        &mut diagnostics,
    );
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
fn record_row_generic_binder_does_not_create_type_parameter_form() {
    let mut env = setup();
    let owner = phalcom_semantic::types::parameter::TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "RowContainer".into()));
    let params = vec![phalcom_ast::ast::GenericParameterSyntax {
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
        phalcom_semantic::types::annotation::GenericBinderSite::NominalDeclaration,
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
fn invalid_generic_kind_does_not_publish_signature() {
    let mut env = setup();
    let owner = phalcom_semantic::types::parameter::TypeParameterOwner::Declaration(DeclarationId::new(env.module.clone(), "Broken".into()));
    let params = vec![phalcom_ast::ast::GenericParameterSyntax {
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
