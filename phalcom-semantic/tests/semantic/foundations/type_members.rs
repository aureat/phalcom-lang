use phalcom_ast::ast::{StaticSymbolRef, TypeAnnotation, TypeAnnotationExpr, TypeMemberArgumentSyntax, TypeMemberSelectorSyntax, TypeTupleElement};
use phalcom_common::range::SourceRange;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::declarations::{DeclarationTypeTable, bootstrap_universe_declarations};
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::annotation::{
    SimpleTypeResolver, TypeFormResolution, TypeFormationInvalid, TypeFormationSite, resolve_type_annotation, resolve_type_form,
};
use phalcom_semantic::types::row::RecordRowField;
use phalcom_semantic::types::store::{LabeledTypePackElement, TupleTypeElement, TypeData, TypeStore};

const RANGE: SourceRange = SourceRange { start: 0, end: 1 };

struct TestEnv {
    store: TypeStore,
    declarations: DeclarationTypeTable,
    resolver: SimpleTypeResolver,
    site: TypeFormationSite,
}

fn decl(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::universe_root(), name.into())
}

fn ref_ann(name: &str) -> TypeAnnotation {
    TypeAnnotation {
        expr: TypeAnnotationExpr::Reference(StaticSymbolRef {
            root: name.into(),
            root_range: RANGE,
            members: vec![],
            range: RANGE,
        }),
        range: RANGE,
    }
}

fn tuple_ann(elements: Vec<(Option<&str>, TypeAnnotation)>) -> TypeAnnotation {
    TypeAnnotation {
        expr: TypeAnnotationExpr::Tuple {
            elements: elements
                .into_iter()
                .map(|(label, ty)| {
                    phalcom_ast::ast::TypeTupleItem::Element(TypeTupleElement {
                        label: label.map(Into::into),
                        ty,
                        range: RANGE,
                    })
                })
                .collect(),
            range: RANGE,
        },
        range: RANGE,
    }
}

fn record_ann(fields: Vec<(&str, TypeAnnotation)>) -> TypeAnnotation {
    TypeAnnotation {
        expr: TypeAnnotationExpr::Record {
            fields: fields
                .into_iter()
                .map(|(name, ty)| phalcom_ast::ast::RecordTypeField {
                    name: name.into(),
                    ty,
                    range: RANGE,
                })
                .collect(),
            tail: None,
            range: RANGE,
        },
        range: RANGE,
    }
}

fn proj_ann(subject: TypeAnnotation, selector: &str, arguments: Vec<TypeMemberArgumentSyntax>) -> TypeAnnotation {
    TypeAnnotation {
        expr: TypeAnnotationExpr::TypeMemberProjection {
            subject: Box::new(subject),
            selector: TypeMemberSelectorSyntax::Named {
                name: selector.into(),
                range: RANGE,
            },
            arguments,
            range: RANGE,
        },
        range: RANGE,
    }
}

fn subscript_ann(subject: TypeAnnotation, index: i64) -> TypeAnnotation {
    TypeAnnotation {
        expr: TypeAnnotationExpr::TypeMemberProjection {
            subject: Box::new(subject),
            selector: TypeMemberSelectorSyntax::Subscript { range: RANGE },
            arguments: vec![TypeMemberArgumentSyntax::IntLiteral { value: index, range: RANGE }],
            range: RANGE,
        },
        range: RANGE,
    }
}

#[test]
fn integer_type_member_arguments_use_canonical_const_int_terms() {
    let mut env = setup();
    let ann = subscript_ann(ref_ann("Tuple"), 3);
    let mut diags = Vec::new();
    let projection = match resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &ann, &mut diags) {
        TypeFormResolution::Ready(projection) => projection,
        other => panic!("abstract Tuple subscript should resolve symbolically, got {other:?}"),
    };

    let TypeData::TypeMemberProjection(projection_data) = env.store.get(projection) else {
        panic!("abstract Tuple subscript should remain a type-member projection");
    };
    let [phalcom_semantic::types::type_member::TypeMemberArgument::Type(argument)] = projection_data.arguments.as_ref() else {
        panic!("integer subscript argument should be represented by a canonical type term");
    };
    assert!(matches!(env.store.get(*argument), TypeData::ConstInt(3)));
    assert!(diags.is_empty());
}

#[test]
fn source_type_member_projection_uses_surface_identity_and_checks_argument_domain() {
    use phalcom_common::modifier::ImplementationKind;
    use phalcom_common::selector::SelectorBase;
    use phalcom_semantic::associated::{AssociatedFamilyInfo, AssociatedFamilyKind, AssociatedMemberId, AssociatedSurface};
    use phalcom_semantic::db::product::TypeMemberDeclarationProduct;
    use phalcom_semantic::types::type_member::{TypeMemberDeclarationId, TypeMemberDomain, TypeMemberParameter, TypeMemberSelector, TypeMemberSignature};

    let mut env = setup();
    let owner = decl("Geometry");
    let header = phalcom_semantic::declarations::NominalDeclarationHeader::from_signature(&mut env.store, owner.clone(), None);
    env.declarations.insert(header.into_type_info(None));
    env.resolver.insert("Geometry", owner.clone());
    let selector = TypeMemberSelector::named("Choose");
    let member_id = TypeMemberDeclarationId::new(owner.clone(), selector.clone());
    let product = std::sync::Arc::new(TypeMemberDeclarationProduct {
        id: member_id.clone(),
        signature: TypeMemberSignature {
            selector: selector.clone(),
            parameters: Box::new([TypeMemberParameter {
                name: "candidate".into(),
                domain: TypeMemberDomain::Kind(phalcom_semantic::types::id::KindId::TYPE),
            }]),
            result_domain: TypeMemberDomain::Kind(phalcom_semantic::types::id::KindId::TYPE),
            intrinsic: None,
            implementation: ImplementationKind::Phalcom,
        },
        source_range: RANGE,
    });
    let mut surface = AssociatedSurface::new(owner.clone());
    surface.families.insert(
        SelectorBase::Named("Choose".into()),
        AssociatedFamilyInfo {
            id: phalcom_semantic::identity::AssociatedFamilyId::new(owner.clone(), SelectorBase::Named("Choose".into())),
            kind: AssociatedFamilyKind::TypeMember,
            members: Box::new([AssociatedMemberId::TypeMember(member_id.clone())]),
        },
    );
    env.resolver.associated_surfaces.insert(owner, std::sync::Arc::new(surface));
    env.resolver.type_member_declarations.insert(member_id.clone(), product);

    let good = proj_ann(
        ref_ann("Geometry"),
        "Choose",
        vec![TypeMemberArgumentSyntax::TypeForm(ref_ann("Geometry"))],
    );
    let mut diagnostics = Vec::new();
    let projection = match resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &good, &mut diagnostics) {
        TypeFormResolution::Ready(projection) => projection,
        other => panic!("selected type member should resolve, got {other:?}; diagnostics: {diagnostics:?}"),
    };
    let TypeData::TypeMemberProjection(projection_data) = env.store.get(projection) else {
        panic!("source type member should remain a symbolic projection");
    };
    assert_eq!(projection_data.source_declaration.as_ref(), Some(&member_id));
    assert!(diagnostics.is_empty());

    let wrong = proj_ann(ref_ann("Geometry"), "Choose", vec![TypeMemberArgumentSyntax::IntLiteral { value: 1, range: RANGE }]);
    let mut diagnostics = Vec::new();
    assert!(matches!(
        resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &wrong, &mut diagnostics),
        TypeFormResolution::Invalid(TypeFormationInvalid::TypeArgumentKindMismatch)
    ));
    assert!(!diagnostics.is_empty());
}

fn setup() -> TestEnv {
    let mut store = TypeStore::new();
    let module = ModuleId::universe_root();
    let declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(module.clone(), key.name().into()));
    let mut resolver = SimpleTypeResolver::new();
    for name in [
        "Int",
        "String",
        "Bool",
        "List",
        "Map",
        "Option",
        "Tuple",
        "Record",
        "TypePack",
        "LabeledTypePack",
    ] {
        resolver.insert(name, DeclarationId::new(module.clone(), name.into()));
    }
    let site = TypeFormationSite::module(module);
    TestEnv {
        store,
        declarations,
        resolver,
        site,
    }
}

#[test]
fn test_tuple_positionals_projection() {
    let mut env = setup();
    let int_id = env.store.nominal(decl("Int"));
    let str_id = env.store.nominal(decl("String"));

    // `(Int, String)::Positionals` -> `TypePack([Int, String])`
    let ann = proj_ann(tuple_ann(vec![(None, ref_ann("Int")), (None, ref_ann("String"))]), "Positionals", vec![]);
    let mut diags = Vec::new();
    let res = resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &ann, &mut diags);
    assert_eq!(res, TypeFormResolution::Ready(env.store.type_pack(Box::new([int_id, str_id])).unwrap()));
    assert!(diags.is_empty());

    // `(Int, label: String)::Positionals` -> `TypePack([Int])`
    let mixed_ann = proj_ann(
        tuple_ann(vec![(None, ref_ann("Int")), (Some("label"), ref_ann("String"))]),
        "Positionals",
        vec![],
    );
    let res_mixed = resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &mixed_ann, &mut diags);
    assert_eq!(res_mixed, TypeFormResolution::Ready(env.store.type_pack(Box::new([int_id])).unwrap()));
    assert!(diags.is_empty());

    // `()::Positionals` -> `TypePack([])`
    let unit_ann = proj_ann(tuple_ann(vec![]), "Positionals", vec![]);
    let res_unit = resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &unit_ann, &mut diags);
    assert_eq!(res_unit, TypeFormResolution::Ready(env.store.type_pack(Box::new([])).unwrap()));
    assert!(diags.is_empty());
}

#[test]
fn test_tuple_labeled_projection() {
    let mut env = setup();
    let int_id = env.store.nominal(decl("Int"));
    let str_id = env.store.nominal(decl("String"));

    // `(Int, age: Int, name: String)::Labeled` -> `LabeledTypePack([age: Int, name: String])`
    let ann = proj_ann(
        tuple_ann(vec![(None, ref_ann("Int")), (Some("age"), ref_ann("Int")), (Some("name"), ref_ann("String"))]),
        "Labeled",
        vec![],
    );
    let mut diags = Vec::new();
    let res = resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &ann, &mut diags);
    assert_eq!(
        res,
        TypeFormResolution::Ready(
            env.store
                .labeled_type_pack(Box::new([
                    LabeledTypePackElement {
                        label: "age".into(),
                        element: int_id,
                    },
                    LabeledTypePackElement {
                        label: "name".into(),
                        element: str_id,
                    },
                ]))
                .unwrap()
        )
    );
    assert!(diags.is_empty());
}

#[test]
fn test_tuple_subscript_indexing() {
    let mut env = setup();
    let int_id = env.store.nominal(decl("Int"));
    let str_id = env.store.nominal(decl("String"));
    let bool_id = env.store.nominal(decl("Bool"));

    // Valid tuple: positionals first, then labeled elements
    let tup = tuple_ann(vec![(None, ref_ann("Int")), (None, ref_ann("Bool")), (Some("name"), ref_ann("String"))]);

    let mut diags = Vec::new();

    // `(Int, Bool, name: String)[0]` -> `Int`
    let res0 = resolve_type_annotation(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &subscript_ann(tup.clone(), 0),
        &mut diags,
    );
    assert_eq!(res0.ty(), Some(int_id));

    // `(Int, Bool, name: String)[1]` -> `Bool`
    let res1 = resolve_type_annotation(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &subscript_ann(tup.clone(), 1),
        &mut diags,
    );
    assert_eq!(res1.ty(), Some(bool_id));

    // `(Int, Bool, name: String)[2]` -> `String`
    let res2 = resolve_type_annotation(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &subscript_ann(tup.clone(), 2),
        &mut diags,
    );
    assert_eq!(res2.ty(), Some(str_id));

    // `(Int, Bool, name: String)[-1]` -> `String` (last element)
    let res_neg1 = resolve_type_annotation(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &subscript_ann(tup.clone(), -1),
        &mut diags,
    );
    assert_eq!(res_neg1.ty(), Some(str_id));

    // `(Int, Bool, name: String)[-2]` -> `Bool`
    let res_neg2 = resolve_type_annotation(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &subscript_ann(tup.clone(), -2),
        &mut diags,
    );
    assert_eq!(res_neg2.ty(), Some(bool_id));
    assert!(diags.is_empty());
}

#[test]
fn test_tuple_and_record_map_projection() {
    let mut env = setup();
    let int_id = env.store.nominal(decl("Int"));
    let str_id = env.store.nominal(decl("String"));
    let list_id = env.declarations.form(&decl("List")).unwrap();

    let mut diags = Vec::new();

    // `(Int, name: String)::Map<List>` -> `(List<Int>, name: List<String>)`
    let tup_ann = proj_ann(
        tuple_ann(vec![(None, ref_ann("Int")), (Some("name"), ref_ann("String"))]),
        "Map",
        vec![TypeMemberArgumentSyntax::TypeForm(ref_ann("List"))],
    );
    let res_tup = resolve_type_annotation(&mut env.store, &env.declarations, &env.resolver, &env.site, &tup_ann, &mut diags);

    let list_int = env.store.apply_type_form(list_id, &[int_id]).unwrap();
    let list_str = env.store.apply_type_form(list_id, &[str_id]).unwrap();
    let expected_tup = env
        .store
        .tuple(Box::new([TupleTypeElement::unlabeled(list_int), TupleTypeElement::labeled("name", list_str)]));
    assert_eq!(res_tup.ty(), Some(expected_tup));
    assert!(diags.is_empty());

    // `{ age: Int, name: String }::Map<List>` -> `{ age: List<Int>, name: List<String> }`
    let rec_ann = proj_ann(
        record_ann(vec![("age", ref_ann("Int")), ("name", ref_ann("String"))]),
        "Map",
        vec![TypeMemberArgumentSyntax::TypeForm(ref_ann("List"))],
    );
    let res_rec = resolve_type_annotation(&mut env.store, &env.declarations, &env.resolver, &env.site, &rec_ann, &mut diags);

    let expected_rec = env.store.record(Box::new([
        RecordRowField {
            name: "age".into(),
            ty: list_int,
        },
        RecordRowField {
            name: "name".into(),
            ty: list_str,
        },
    ]));
    assert_eq!(res_rec.ty(), Some(expected_rec));
    assert!(diags.is_empty());
}

#[test]
fn test_map_rejects_non_constructor_argument() {
    let mut env = setup();
    let ann = proj_ann(
        tuple_ann(vec![(None, ref_ann("Int"))]),
        "Map",
        vec![TypeMemberArgumentSyntax::TypeForm(ref_ann("Int"))],
    );
    let mut diags = Vec::new();

    let result = resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &ann, &mut diags);

    assert_eq!(result, TypeFormResolution::Invalid(TypeFormationInvalid::TypeArgumentKindMismatch));
}

#[test]
fn test_at_rejects_non_integer_argument() {
    let mut env = setup();
    let ann = proj_ann(
        tuple_ann(vec![(None, ref_ann("Int"))]),
        "At",
        vec![TypeMemberArgumentSyntax::TypeForm(ref_ann("Option"))],
    );
    let mut diags = Vec::new();

    let result = resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &ann, &mut diags);

    assert_eq!(result, TypeFormResolution::Invalid(TypeFormationInvalid::TypeArgumentKindMismatch));
}
