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
    match name {
        "Tuple" => phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Tuple),
        "Record" => phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Record),
        "Int" => phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Int),
        "String" => phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::String),
        "Bool" => phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Bool),
        "List" => phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::List),
        "Array" => phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Array),
        "Map" => phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Map),
        "Option" => phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Option),
        "TypePack" | "LabeledTypePack" | "CompleteTypePack" => DeclarationId::new(
            ModuleId::universe(phalcom_modules::identity::ModulePath::from_components(vec![
                phalcom_modules::identity::ModuleComponent::from_identifier("collections").unwrap(),
                phalcom_modules::identity::ModuleComponent::from_identifier("tuple").unwrap(),
            ])),
            name.into(),
        ),
        "RecordFields" => DeclarationId::new(
            ModuleId::universe(phalcom_modules::identity::ModulePath::from_components(vec![
                phalcom_modules::identity::ModuleComponent::from_identifier("collections").unwrap(),
                phalcom_modules::identity::ModuleComponent::from_identifier("record").unwrap(),
            ])),
            name.into(),
        ),
        "FieldKey" => DeclarationId::new(
            ModuleId::universe(phalcom_modules::identity::ModulePath::from_components(vec![
                phalcom_modules::identity::ModuleComponent::from_identifier("collections").unwrap(),
                phalcom_modules::identity::ModuleComponent::from_identifier("record").unwrap(),
            ])),
            name.into(),
        ),
        "UnorderedShape" | "FieldLookup" | "UnorderedShapeView" | "UnorderedTail" => DeclarationId::new(
            ModuleId::universe(phalcom_modules::identity::ModulePath::from_components(vec![
                phalcom_modules::identity::ModuleComponent::from_identifier("reflection").unwrap(),
                phalcom_modules::identity::ModuleComponent::from_identifier("structural").unwrap(),
            ])),
            name.into(),
        ),
        other => DeclarationId::new(ModuleId::universe_root(), other.into()),
    }
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
                .map(|(name, ty)| phalcom_ast::ast::RecordTypeField::symbol(name, ty, false, RANGE))
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

fn record_expansion_application(operand: TypeAnnotation) -> TypeAnnotation {
    TypeAnnotation {
        expr: TypeAnnotationExpr::Application {
            origin: Box::new(ref_ann("Record")),
            arguments: vec![phalcom_ast::ast::GenericArgumentSyntax::Expansion(phalcom_ast::ast::TypePackExpansionSyntax {
                mode: phalcom_ast::ast::ExpansionMode::Labeled,
                operator_range: RANGE,
                operand,
                range: RANGE,
            })],
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
    assert!(matches!(env.store.get(*argument), TypeData::ConstInt(phalcom_semantic::StaticInt::Small(3))));
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

    let good = proj_ann(ref_ann("Geometry"), "Choose", vec![TypeMemberArgumentSyntax::TypeForm(ref_ann("Geometry"))]);
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

    let wrong = proj_ann(
        ref_ann("Geometry"),
        "Choose",
        vec![TypeMemberArgumentSyntax::IntLiteral { value: 1, range: RANGE }],
    );
    let mut diagnostics = Vec::new();
    assert!(matches!(
        resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &wrong, &mut diagnostics),
        TypeFormResolution::Invalid(TypeFormationInvalid::TypeArgumentKindMismatch)
    ));
    assert!(!diagnostics.is_empty());
}

#[test]
fn intrinsic_type_member_source_products_match_the_evaluator_catalog() {
    use phalcom_common::modifier::ImplementationKind;
    use phalcom_semantic::types::TypeResolver;
    use phalcom_semantic::types::type_member::{IntrinsicTypeOperation, lookup_intrinsic_type_operation};

    let mut env = setup();
    for operation in IntrinsicTypeOperation::ALL {
        let mut matching = env
            .resolver
            .type_member_declarations
            .iter()
            .filter(|(_, product)| product.signature.intrinsic == Some(operation));
        let (member_id, product) = matching.next().unwrap_or_else(|| panic!("missing source member for {}", operation.name()));
        assert!(matching.next().is_none(), "multiple source members select {}", operation.name());
        let owner = env.declarations.intrinsic_type_owner(&member_id.owner).expect("intrinsic source owner");
        assert_eq!(lookup_intrinsic_type_operation(owner, &member_id.selector), Some(operation));
        let selected = env
            .resolver
            .resolve_associated_type_member(&member_id.owner, &member_id.selector)
            .expect("source member must be admitted by its AssociatedSurface");
        assert_eq!(selected.id, product.id);
        assert_eq!(selected.signature.intrinsic, Some(operation));

        match operation {
            IntrinsicTypeOperation::UnorderedShapeLookup => {
                assert_eq!(selected.signature.parameters.len(), 1);
                assert!(matches!(
                    selected.signature.parameters[0].domain,
                    phalcom_semantic::types::type_member::TypeMemberDomain::ValueType(_)
                ));
                assert!(matches!(
                    selected.signature.result_domain,
                    phalcom_semantic::types::type_member::TypeMemberDomain::ValueType(_)
                ));
            }
            IntrinsicTypeOperation::UnorderedShapeView | IntrinsicTypeOperation::UnorderedShapeTail => {
                assert_eq!(selected.signature.parameters.len(), 0);
                assert!(matches!(
                    selected.signature.result_domain,
                    phalcom_semantic::types::type_member::TypeMemberDomain::ValueType(_)
                ));
            }
            _ => {
                let (expected_parameters, expected_result) = operation.signature_kinds(&mut env.store);
                let actual_parameters = selected
                    .signature
                    .parameters
                    .iter()
                    .map(|parameter| match &parameter.domain {
                        phalcom_semantic::types::type_member::TypeMemberDomain::Kind(kind) => *kind,
                        phalcom_semantic::types::type_member::TypeMemberDomain::ValueType(_) => panic!("unexpected ValueType in intrinsic kind test"),
                    })
                    .collect::<Vec<_>>();
                let phalcom_semantic::types::type_member::TypeMemberDomain::Kind(actual_result) = &selected.signature.result_domain else {
                    panic!("expected Kind result domain");
                };
                assert_eq!(
                    actual_parameters.as_slice(),
                    expected_parameters.as_ref(),
                    "{} parameter domains",
                    operation.name()
                );
                assert_eq!(*actual_result, expected_result, "{} result domain", operation.name());
            }
        }
    }

    for product in env.resolver.type_member_declarations.values() {
        if product.signature.implementation != ImplementationKind::Intrinsic {
            continue;
        }
        let owner = env
            .declarations
            .intrinsic_type_owner(&product.id.owner)
            .expect("every authored intrinsic source member belongs to an intrinsic owner");
        let operation = lookup_intrinsic_type_operation(owner, &product.id.selector)
            .unwrap_or_else(|| panic!("source intrinsic member {} has no evaluator", product.id.selector.name()));
        assert_eq!(product.signature.intrinsic, Some(operation));
    }
}

#[test]
fn source_visible_complete_pack_lane_projections_keep_canonical_member_declarations() {
    use phalcom_semantic::types::id::KindId;
    use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
    use phalcom_semantic::types::type_member::{TypeMemberDeclarationId, TypeMemberSelector};

    let mut env = setup();
    let parameter = env.store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(decl("CompletePackSourceProjectionTest")),
        0,
        "P",
        KindId::COMPLETE_TYPE_PACK,
    ));
    let subject = env.store.parameter_form(parameter);
    assert_eq!(env.store.kind_of(subject), KindId::COMPLETE_TYPE_PACK);
    env.resolver.insert_parameter("P", subject);

    use phalcom_semantic::types::TypeResolver;
    let source_owner = env
        .declarations
        .intrinsic_type_owner_declaration(phalcom_semantic::types::type_member::IntrinsicTypeOwner::CompleteTypePack)
        .expect("canonical CompleteTypePack source owner");
    assert_eq!(source_owner, decl("CompleteTypePack"));
    assert!(
        env.resolver
            .resolve_associated_type_member(&source_owner, &TypeMemberSelector::named("Positionals"))
            .is_some(),
        "canonical Positionals source product should be admitted"
    );
    assert!(
        env.resolver
            .resolve_associated_type_member(&source_owner, &TypeMemberSelector::named("Labeled"))
            .is_some(),
        "canonical Labeled source product should be admitted"
    );

    let positionals_product = env
        .resolver
        .resolve_associated_type_member(&source_owner, &TypeMemberSelector::named("Positionals"))
        .expect("published Positionals product");
    let labeled_product = env
        .resolver
        .resolve_associated_type_member(&source_owner, &TypeMemberSelector::named("Labeled"))
        .expect("published Labeled product");
    let owner = decl("CompleteTypePack");
    let positionals_member = TypeMemberDeclarationId::new(owner.clone(), TypeMemberSelector::named("Positionals"));
    let labeled_member = TypeMemberDeclarationId::new(owner, TypeMemberSelector::named("Labeled"));
    assert_eq!(positionals_product.id, positionals_member);
    assert_eq!(labeled_product.id, labeled_member);
    assert_eq!(
        positionals_product.signature.intrinsic,
        Some(phalcom_semantic::types::type_member::IntrinsicTypeOperation::CompleteTypePackPositionals)
    );
    assert_eq!(
        labeled_product.signature.intrinsic,
        Some(phalcom_semantic::types::type_member::IntrinsicTypeOperation::CompleteTypePackLabeled)
    );

    let positionals_ty = env.store.type_member_projection(
        phalcom_semantic::types::type_member::TypeLevelProjection {
            subject: phalcom_semantic::types::type_member::ProjectionSubject::Type(subject),
            selector: TypeMemberSelector::named("Positionals"),
            arguments: Box::new([]),
            source_declaration: Some(positionals_product.id.clone()),
        },
        KindId::TYPE_PACK,
    );
    let labeled_ty = env.store.type_member_projection(
        phalcom_semantic::types::type_member::TypeLevelProjection {
            subject: phalcom_semantic::types::type_member::ProjectionSubject::Type(subject),
            selector: TypeMemberSelector::named("Labeled"),
            arguments: Box::new([]),
            source_declaration: Some(labeled_product.id.clone()),
        },
        KindId::LABELED_TYPE_PACK,
    );

    let TypeData::TypeMemberProjection(positionals) = env.store.get(positionals_ty) else {
        panic!("expected source-backed Positionals projection");
    };
    let TypeData::TypeMemberProjection(labeled) = env.store.get(labeled_ty) else {
        panic!("expected source-backed Labeled projection");
    };

    assert_eq!(positionals.source_declaration.as_ref(), Some(&positionals_member));
    assert_eq!(labeled.source_declaration.as_ref(), Some(&labeled_member));
    assert!(positionals.arguments.is_empty());
    assert!(labeled.arguments.is_empty());
    assert_eq!(positionals.subject, labeled.subject);
    assert!(matches!(
        &positionals.subject,
        phalcom_semantic::types::type_member::ProjectionSubject::Type(subject_ty) if *subject_ty == subject
    ));

    assert_eq!(env.store.kind_of(positionals_ty), KindId::TYPE_PACK);
    assert_eq!(env.store.kind_of(labeled_ty), KindId::LABELED_TYPE_PACK);
    assert_eq!(
        env.store.complete_type_pack(positionals_ty, labeled_ty).expect("published intrinsic lanes"),
        subject
    );
}

fn setup() -> TestEnv {
    let mut store = TypeStore::new();
    let declarations = bootstrap_universe_declarations(&mut store, &phalcom_semantic::core_surface::universe_declaration);
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
        "CompleteTypePack",
        "RecordFields",
        "UnorderedShape",
        "FieldKey",
        "FieldLookup",
        "UnorderedShapeView",
        "UnorderedTail",
    ] {
        resolver.insert(name, decl(name));
    }
    let mut declarations = declarations;
    declarations.register_intrinsic_type_owner(decl("UnorderedShape"), phalcom_semantic::types::type_member::IntrinsicTypeOwner::UnorderedShape);
    // These low-level tests use SimpleTypeResolver rather than the source
    // database, so install the checked source member products that Universe
    // declares for the compiler intrinsic evaluators.
    install_intrinsic_source_members(&mut store, &mut resolver);
    let site = TypeFormationSite::module(ModuleId::universe_root());
    TestEnv {
        store,
        declarations,
        resolver,
        site,
    }
}

fn install_intrinsic_source_members(store: &mut TypeStore, resolver: &mut SimpleTypeResolver) {
    use phalcom_common::modifier::ImplementationKind;
    use phalcom_common::selector::SelectorBase;
    use phalcom_semantic::associated::{AssociatedFamilyInfo, AssociatedFamilyKind, AssociatedMemberId, AssociatedSurface};
    use phalcom_semantic::db::product::TypeMemberDeclarationProduct;
    use phalcom_semantic::types::id::KindId;
    use phalcom_semantic::types::type_member::{
        IntrinsicTypeOperation, TypeMemberDeclarationId, TypeMemberDomain, TypeMemberParameter, TypeMemberSelector, TypeMemberSignature,
    };

    fn add(
        resolver: &mut SimpleTypeResolver,
        owner: DeclarationId,
        selector: TypeMemberSelector,
        operation: IntrinsicTypeOperation,
        parameters: Vec<(&str, KindId)>,
        result_kind: KindId,
    ) {
        let base = match &selector {
            TypeMemberSelector::Named(name) => SelectorBase::Named(name.to_string()),
            TypeMemberSelector::Subscript => SelectorBase::Subscript,
        };
        let id = TypeMemberDeclarationId::new(owner.clone(), selector.clone());
        let product = std::sync::Arc::new(TypeMemberDeclarationProduct {
            id: id.clone(),
            signature: TypeMemberSignature {
                selector,
                parameters: parameters
                    .into_iter()
                    .map(|(name, kind)| TypeMemberParameter {
                        name: name.into(),
                        domain: TypeMemberDomain::Kind(kind),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                result_domain: TypeMemberDomain::Kind(result_kind),
                intrinsic: Some(operation),
                implementation: ImplementationKind::Intrinsic,
            },
            source_range: RANGE,
        });
        let mut surface = resolver
            .associated_surfaces
            .get(&owner)
            .map(|surface| (**surface).clone())
            .unwrap_or_else(|| AssociatedSurface::new(owner.clone()));
        surface.families.insert(
            base.clone(),
            AssociatedFamilyInfo {
                id: phalcom_semantic::identity::AssociatedFamilyId::new(owner.clone(), base),
                kind: AssociatedFamilyKind::IntrinsicTypeMember,
                members: Box::new([AssociatedMemberId::TypeMember(id.clone())]),
            },
        );
        resolver.associated_surfaces.insert(owner, std::sync::Arc::new(surface));
        resolver.type_member_declarations.insert(id, product);
    }

    let type_to_type = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let tuple = decl("Tuple");
    for (name, operation, result) in [
        ("Positionals", IntrinsicTypeOperation::TuplePositionals, KindId::TYPE_PACK),
        ("Labeled", IntrinsicTypeOperation::TupleLabeled, KindId::LABELED_TYPE_PACK),
        ("Entries", IntrinsicTypeOperation::TupleEntries, KindId::COMPLETE_TYPE_PACK),
        ("Element", IntrinsicTypeOperation::TupleElement, KindId::TYPE),
        ("Map", IntrinsicTypeOperation::TupleMap, KindId::TYPE),
    ] {
        let parameters = if name == "Map" { vec![("F", type_to_type)] } else { Vec::new() };
        add(&mut *resolver, tuple.clone(), TypeMemberSelector::named(name), operation, parameters, result);
    }
    let const_int = store.static_domain_kind(store.const_int_domain());
    add(
        resolver,
        tuple,
        TypeMemberSelector::subscript(),
        IntrinsicTypeOperation::TupleAt,
        vec![("index", const_int)],
        KindId::TYPE,
    );

    let record = decl("Record");
    for (name, operation, result) in [
        ("Fields", IntrinsicTypeOperation::RecordFields, KindId::RECORD_ROW),
        ("Value", IntrinsicTypeOperation::RecordValue, KindId::TYPE),
        ("Map", IntrinsicTypeOperation::RecordMap, KindId::TYPE),
    ] {
        let parameters = if name == "Map" { vec![("F", type_to_type)] } else { Vec::new() };
        add(resolver, record.clone(), TypeMemberSelector::named(name), operation, parameters, result);
    }

    for (owner_name, selector_name, operation, result_kind) in [
        ("TypePack", "Map", IntrinsicTypeOperation::TypePackMap, KindId::TYPE_PACK),
        ("LabeledTypePack", "Map", IntrinsicTypeOperation::LabeledTypePackMap, KindId::LABELED_TYPE_PACK),
        (
            "CompleteTypePack",
            "Positionals",
            IntrinsicTypeOperation::CompleteTypePackPositionals,
            KindId::TYPE_PACK,
        ),
        (
            "CompleteTypePack",
            "Labeled",
            IntrinsicTypeOperation::CompleteTypePackLabeled,
            KindId::LABELED_TYPE_PACK,
        ),
        (
            "CompleteTypePack",
            "Map",
            IntrinsicTypeOperation::CompleteTypePackMap,
            KindId::COMPLETE_TYPE_PACK,
        ),
        ("RecordFields", "Map", IntrinsicTypeOperation::RecordRowMap, KindId::RECORD_ROW),
    ] {
        add(
            resolver,
            decl(owner_name),
            TypeMemberSelector::named(selector_name),
            operation,
            if selector_name == "Map" { vec![("F", type_to_type)] } else { Vec::new() },
            result_kind,
        );
    }

    fn add_domain(
        resolver: &mut SimpleTypeResolver,
        owner: DeclarationId,
        selector: TypeMemberSelector,
        operation: IntrinsicTypeOperation,
        parameters: Vec<(&str, TypeMemberDomain)>,
        result_domain: TypeMemberDomain,
    ) {
        let base = match &selector {
            TypeMemberSelector::Named(name) => SelectorBase::Named(name.to_string()),
            TypeMemberSelector::Subscript => SelectorBase::Subscript,
        };
        let id = TypeMemberDeclarationId::new(owner.clone(), selector.clone());
        let product = std::sync::Arc::new(TypeMemberDeclarationProduct {
            id: id.clone(),
            signature: TypeMemberSignature {
                selector,
                parameters: parameters
                    .into_iter()
                    .map(|(name, domain)| TypeMemberParameter { name: name.into(), domain })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                result_domain,
                intrinsic: Some(operation),
                implementation: ImplementationKind::Intrinsic,
            },
            source_range: RANGE,
        });
        let mut surface = resolver
            .associated_surfaces
            .get(&owner)
            .map(|surface| (**surface).clone())
            .unwrap_or_else(|| AssociatedSurface::new(owner.clone()));
        surface.families.insert(
            base.clone(),
            AssociatedFamilyInfo {
                id: phalcom_semantic::identity::AssociatedFamilyId::new(owner.clone(), base),
                kind: AssociatedFamilyKind::IntrinsicTypeMember,
                members: Box::new([AssociatedMemberId::TypeMember(id.clone())]),
            },
        );
        resolver.associated_surfaces.insert(owner, std::sync::Arc::new(surface));
        resolver.type_member_declarations.insert(id, product);
    }

    let unordered_shape = decl("UnorderedShape");
    let field_key_ty = store.nominal(decl("FieldKey"));
    let field_lookup_ty = store.nominal(decl("FieldLookup"));
    let view_ty = store.nominal(decl("UnorderedShapeView"));
    let tail_ty = store.nominal(decl("UnorderedTail"));
    add_domain(
        resolver,
        unordered_shape.clone(),
        TypeMemberSelector::named("Lookup"),
        IntrinsicTypeOperation::UnorderedShapeLookup,
        vec![("K", TypeMemberDomain::ValueType(field_key_ty))],
        TypeMemberDomain::ValueType(field_lookup_ty),
    );
    add_domain(
        resolver,
        unordered_shape.clone(),
        TypeMemberSelector::named("View"),
        IntrinsicTypeOperation::UnorderedShapeView,
        vec![],
        TypeMemberDomain::ValueType(view_ty),
    );
    add_domain(
        resolver,
        unordered_shape,
        TypeMemberSelector::named("Tail"),
        IntrinsicTypeOperation::UnorderedShapeTail,
        vec![],
        TypeMemberDomain::ValueType(tail_ty),
    );
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
    let expected_tup = env.store.tuple(Box::new([
        TupleTypeElement::unlabeled(list_int),
        TupleTypeElement::labeled("name".into(), list_str),
    ]));
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
        RecordRowField::symbol_required("age", list_int),
        RecordRowField::symbol_required("name", list_str),
    ]));
    assert_eq!(res_rec.ty(), Some(expected_rec));
    assert!(diags.is_empty());
}

#[test]
fn record_fields_projection_preserves_exact_row_identity() {
    use phalcom_semantic::declarations::{DeclarationTypeBinding, DeclarationTypeInfo};
    use phalcom_semantic::types::id::KindId;
    use phalcom_semantic::types::parameter::{GenericBinderHeader, GenericParameterMode, GenericSignature};
    use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
    use phalcom_semantic::types::row::RecordRowTail;
    use phalcom_semantic::types::store::StaticArgument;
    use phalcom_semantic::types::type_member::{ProjectionSubject, TypeMemberDeclarationId, TypeMemberSelector};

    let mut env = setup();
    let source_shape = record_ann(vec![("name", ref_ann("String")), ("age", ref_ann("Int"))]);
    let mut diagnostics = Vec::new();
    let TypeFormResolution::Ready(source_record) =
        resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &source_shape, &mut diagnostics)
    else {
        panic!("the concrete Record shape should resolve: {diagnostics:?}");
    };
    let TypeData::Record(source_row) = env.store.get(source_record) else {
        panic!("the source shape should be a structural Record");
    };
    let source_row = *source_row;

    let fields = proj_ann(ref_ann("KnownRecord"), "Fields", vec![]);
    env.resolver.insert_type_form_binding("KnownRecord", source_record);
    diagnostics.clear();
    let TypeFormResolution::Ready(fields) = resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &fields, &mut diagnostics) else {
        panic!("Record::Fields should remain a source-selected row projection: {diagnostics:?}");
    };
    let TypeData::TypeMemberProjection(fields_projection) = env.store.get(fields) else {
        panic!("Record::Fields should remain a row-valued type-member projection");
    };
    let expected_member = TypeMemberDeclarationId::new(decl("Record"), TypeMemberSelector::named("Fields"));
    assert_eq!(fields_projection.source_declaration.as_ref(), Some(&expected_member));
    assert!(matches!(&fields_projection.subject, ProjectionSubject::Type(subject) if *subject == source_record));

    let mut exporter =
        phalcom_semantic::metadata::MetadataExporter::new(&env.store, None, None, None, phalcom_type_meta::header::MetadataProfile::RuntimePublic);
    let fields_node = exporter
        .export_type_form(fields)
        .expect("source-authored Record::Fields projection is exportable");
    let bundle = exporter.build_bundle(&[]).expect("source-authored projection metadata");
    let phalcom_type_meta::type_node::TypeNode::TypeMemberProjection(exported) = &bundle.types[fields_node.0 as usize].form else {
        panic!("expected exported type-member projection");
    };
    let exported_member = exported
        .source_declaration
        .as_ref()
        .expect("public source projection retains its member identity");
    let expected_stable_member = phalcom_type_meta::identity::StableTypeMemberRef {
        owner: phalcom_semantic::metadata::stable_identity::to_stable_declaration(&expected_member.owner),
        selector: expected_member.selector.name().into(),
    };
    assert_eq!(exported_member, &expected_stable_member);
    phalcom_type_meta::validate::validate_metadata_bundle(&bundle, &phalcom_type_meta::validate::ValidationLimits::default())
        .expect("source-member projection bundle validates");

    let TypeFormResolution::Ready(reapplied) = resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &record_expansion_application(proj_ann(ref_ann("KnownRecord"), "Fields", vec![])),
        &mut diagnostics,
    ) else {
        panic!("Record<**Record::Fields> should retain the known row: {diagnostics:?}");
    };
    assert!(matches!(env.store.get(reapplied), TypeData::Record(row) if *row == source_row));

    let family = decl("RecordFamily");
    let family_owner = TypeParameterOwner::Declaration(family.clone());
    let family_parameter = env
        .store
        .intern_type_parameter(TypeParameterData::new(family_owner.clone(), 0, "Fs", KindId::RECORD_ROW).with_mode(GenericParameterMode::LabeledPack));
    let family_signature =
        GenericSignature::new(family_owner, Box::new([family_parameter])).with_parameter_modes(Box::new([GenericParameterMode::LabeledPack]));
    let family_kind = env.store.arrow_kind(vec![KindId::RECORD_ROW].into_boxed_slice(), KindId::TYPE);
    let family_form = env.store.nominal_form(family.clone(), family_kind);
    env.declarations.insert(DeclarationTypeInfo {
        declaration: family.clone(),
        form: family_form,
        class_object_type: env.store.class_object_type(family.clone()),
        kind: family_kind,
        binding: DeclarationTypeBinding::ordinary(),
        generic_binder_header: Some(GenericBinderHeader::from_signature(&env.store, &family_signature)),
        generic_signature: Some(family_signature),
        supertype_template: None,
    });
    env.resolver.insert("RecordFamily", family);
    let TypeFormResolution::Ready(reapplied_family) = resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &TypeAnnotation {
            expr: TypeAnnotationExpr::Application {
                origin: Box::new(ref_ann("RecordFamily")),
                arguments: vec![phalcom_ast::ast::GenericArgumentSyntax::Expansion(phalcom_ast::ast::TypePackExpansionSyntax {
                    mode: phalcom_ast::ast::ExpansionMode::Labeled,
                    operator_range: RANGE,
                    operand: proj_ann(ref_ann("KnownRecord"), "Fields", vec![]),
                    range: RANGE,
                })],
                range: RANGE,
            },
            range: RANGE,
        },
        &mut diagnostics,
    ) else {
        panic!("RecordFamily<**Record::Fields> should preserve its row argument: {diagnostics:?}");
    };
    let TypeData::Applied { arguments, .. } = env.store.get(reapplied_family) else {
        panic!("the custom Record family application should remain nominal");
    };
    assert!(matches!(arguments.as_ref(), [StaticArgument::RecordRow(row)] if *row == source_row));

    let family = env.declarations.form(&decl("Record")).expect("canonical Record family form");
    let applied_subject = env
        .store
        .apply_static_form(family, &[StaticArgument::RecordRow(source_row)])
        .expect("Record row application");
    env.resolver.insert_type_form_binding("AppliedRecord", applied_subject);
    let applied_fields = proj_ann(ref_ann("AppliedRecord"), "Fields", vec![]);
    diagnostics.clear();
    let TypeFormResolution::Ready(_applied_fields_type) =
        resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &applied_fields, &mut diagnostics)
    else {
        panic!("applied Record::Fields should retain its row argument: {diagnostics:?}");
    };
    let TypeFormResolution::Ready(reapplied) = resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &record_expansion_application(applied_fields),
        &mut diagnostics,
    ) else {
        panic!("Record<**(Record<**Fs>::Fields)> should resolve: {diagnostics:?}");
    };
    assert!(matches!(env.store.get(reapplied), TypeData::Record(row) if *row == source_row));

    let owner = TypeParameterOwner::Declaration(decl("RecordFieldsMap"));
    let parameter = env.store.intern_type_parameter(TypeParameterData::new(owner, 0, "Fs", KindId::RECORD_ROW));
    env.resolver.insert_record_row_binding("Fs", parameter);
    let mapped = proj_ann(ref_ann("Fs"), "Map", vec![TypeMemberArgumentSyntax::TypeForm(ref_ann("List"))]);
    diagnostics.clear();
    let TypeFormResolution::Ready(mapped) = resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &mapped, &mut diagnostics) else {
        panic!("RecordFields::Map should accept a RecordFields binder: {diagnostics:?}");
    };
    assert_eq!(env.store.kind_of(mapped), KindId::RECORD_ROW);
    let TypeData::TypeMemberProjection(mapped_projection) = env.store.get(mapped) else {
        panic!("a symbolic RecordFields::Map should remain a row-domain projection");
    };
    assert!(matches!(&mapped_projection.subject, ProjectionSubject::RecordRow(row) if env.store.record_row(*row).tail == RecordRowTail::Parameter(parameter)));
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn symbolic_pack_maps_preserve_domain_and_source_member_identity() {
    use phalcom_semantic::types::id::KindId;
    use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};

    let mut env = setup();
    for (name, kind) in [("Ts", KindId::TYPE_PACK), ("Ls", KindId::LABELED_TYPE_PACK), ("Es", KindId::COMPLETE_TYPE_PACK)] {
        let parameter = env.store.intern_type_parameter(TypeParameterData::new(
            TypeParameterOwner::Declaration(decl("SymbolicPackMaps")),
            kind.index() as u32,
            name,
            kind,
        ));
        let form = env.store.parameter_form(parameter);
        env.resolver.insert_parameter(name, form);
        let mut diagnostics = Vec::new();
        let mapped = proj_ann(ref_ann(name), "Map", vec![TypeMemberArgumentSyntax::TypeForm(ref_ann("List"))]);
        let TypeFormResolution::Ready(mapped) = resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &mapped, &mut diagnostics)
        else {
            panic!("symbolic pack Map must resolve: {diagnostics:?}");
        };
        assert_eq!(env.store.kind_of(mapped), kind);
        let TypeData::TypeMemberProjection(projection) = env.store.get(mapped) else {
            panic!("symbolic mapping must remain a projection");
        };
        assert!(projection.source_declaration.is_some());
        // A second mapping exercises projection subjects, not only binders.
        env.resolver.insert_parameter("Mapped", mapped);
        let twice = proj_ann(ref_ann("Mapped"), "Map", vec![TypeMemberArgumentSyntax::TypeForm(ref_ann("Option"))]);
        let TypeFormResolution::Ready(twice) = resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &twice, &mut diagnostics) else {
            panic!("composed pack Map must resolve: {diagnostics:?}");
        };
        assert_eq!(env.store.kind_of(twice), kind);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }
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

#[test]
fn type_member_value_type_parameters_and_results_publish_static_value_domains() {
    use phalcom_semantic::types::type_member::TypeMemberDomain;

    let mut env = setup();
    let module = ModuleId::universe_root();
    let field_key_decl = DeclarationId::new(module.clone(), "FieldKey".into());
    let field_lookup_decl = DeclarationId::new(module.clone(), "FieldLookup".into());
    let header_fk = phalcom_semantic::declarations::NominalDeclarationHeader::from_signature(&mut env.store, field_key_decl.clone(), None);
    env.declarations.insert(header_fk.into_type_info(None));
    let header_fl = phalcom_semantic::declarations::NominalDeclarationHeader::from_signature(&mut env.store, field_lookup_decl.clone(), None);
    env.declarations.insert(header_fl.into_type_info(None));
    env.resolver.insert("FieldKey", field_key_decl.clone());
    env.resolver.insert("FieldLookup", field_lookup_decl.clone());

    let field_key_ty = env.store.nominal_type(field_key_decl);
    let field_lookup_ty = env.store.nominal_type(field_lookup_decl);

    let owner = DeclarationId::new(module.clone(), "UnorderedShape".into());
    let def = phalcom_ast::ast::TypeMemberDef {
        selector: phalcom_ast::ast::TypeMemberSelectorSyntax::Named {
            name: "Lookup".to_string(),
            range: RANGE,
        },
        parameters: vec![phalcom_ast::ast::TypeMemberParameterSyntax {
            name: "K".to_string(),
            name_range: RANGE,
            domain: phalcom_ast::ast::TypeMemberParameterDomainSyntax::ConstType(ref_ann("FieldKey")),
            range: RANGE,
        }],
        result: phalcom_ast::ast::TypeMemberResultSyntax::Type(ref_ann("FieldLookup")),
        where_clause: None,
        implementation: phalcom_common::modifier::ImplementationKind::Phalcom,
        implementation_range: None,
        attributes: Vec::new(),
        range: RANGE,
    };

    let (product, diags) = phalcom_semantic::checker::type_member_declaration::build_type_member_declaration(
        &mut env.store,
        &owner,
        &def,
        &env.declarations,
        &env.resolver,
        &module,
    );
    assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
    let product = product.expect("type member declaration product");
    assert_eq!(product.signature.parameters.len(), 1);
    assert_eq!(product.signature.parameters[0].domain, TypeMemberDomain::ValueType(field_key_ty));
    assert_eq!(product.signature.result_domain, TypeMemberDomain::ValueType(field_lookup_ty));
}

#[test]
fn domain_neutral_member_projections_validate_values_and_keep_type_store_clean() {
    use phalcom_common::modifier::ImplementationKind;
    use phalcom_semantic::member_projection::{
        MemberDeclarationIdentity, MemberProjection, MemberProjectionArgument, MemberProjectionSubject, StaticMemberProjectionOutcome,
        validate_member_projection_arguments,
    };
    use phalcom_semantic::static_value::StaticValueStore;
    use phalcom_semantic::types::type_member::{
        ProjectionSubject, TypeLevelProjection, TypeMemberArgument, TypeMemberDeclarationId, TypeMemberDomain, TypeMemberParameter, TypeMemberSelector,
        TypeMemberSignature,
    };
    use phalcom_semantic::{KindId, StaticValueDomain};

    let mut env = setup();
    let mut values = StaticValueStore::new();

    let owner = decl("UnorderedShape");
    let field_key_decl = decl("FieldKey");
    let field_key_ty = env.store.nominal_type(field_key_decl);
    let field_lookup_decl = decl("FieldLookup");
    let field_lookup_ty = env.store.nominal_type(field_lookup_decl);

    let signature = TypeMemberSignature {
        selector: TypeMemberSelector::named("Lookup"),
        intrinsic: None,
        parameters: Box::new([TypeMemberParameter {
            name: "K".into(),
            domain: TypeMemberDomain::ValueType(field_key_ty),
        }]),
        result_domain: TypeMemberDomain::ValueType(field_lookup_ty),
        implementation: ImplementationKind::Intrinsic,
    };

    let sym_val = values.intern_symbol("age");
    let str_val = values.intern_string("age");
    let int_val = values.intern_int(42);
    let bool_val = values.intern_bool(true);
    let unit_val = values.intern_unit();

    // 1. Closed FieldKey arguments validate
    assert!(
        validate_member_projection_arguments(
            &mut env.store,
            &values,
            &signature,
            &[MemberProjectionArgument::Value(sym_val)],
            Some(field_key_ty),
        )
        .is_ok()
    );

    assert!(
        validate_member_projection_arguments(
            &mut env.store,
            &values,
            &signature,
            &[MemberProjectionArgument::Value(str_val)],
            Some(field_key_ty),
        )
        .is_ok()
    );

    assert!(
        validate_member_projection_arguments(
            &mut env.store,
            &values,
            &signature,
            &[MemberProjectionArgument::Value(int_val)],
            Some(field_key_ty),
        )
        .is_ok()
    );

    assert!(
        validate_member_projection_arguments(
            &mut env.store,
            &values,
            &signature,
            &[MemberProjectionArgument::Value(bool_val)],
            Some(field_key_ty),
        )
        .is_ok()
    );

    // 2. Wrong ordinary value type (Unit) is rejected
    assert!(
        validate_member_projection_arguments(
            &mut env.store,
            &values,
            &signature,
            &[MemberProjectionArgument::Value(unit_val)],
            Some(field_key_ty),
        )
        .is_err()
    );

    // 3. String/Int/Bool/Symbol remain distinct
    assert_ne!(sym_val, str_val);
    assert_ne!(sym_val, int_val);
    assert_ne!(sym_val, bool_val);

    // 4. Stuck Product::Shape projection can be represented without TypeId fabrication
    let shape_decl = decl("ProductShape");
    let shape_ty = env.store.nominal_type(shape_decl);
    let stuck_proj = MemberProjection {
        subject: MemberProjectionSubject::Type(env.store.unit()),
        member: MemberDeclarationIdentity::TypeMember(TypeMemberDeclarationId::new(owner, TypeMemberSelector::named("Shape"))),
        arguments: Box::new([]),
        evidence: None,
        result_domain: StaticValueDomain::ValueType(shape_ty),
    };
    let outcome = StaticMemberProjectionOutcome::Stuck(stuck_proj.clone());
    assert!(outcome.is_stuck());
    assert_eq!(outcome.as_reduced_kinded(), None);
    assert_eq!(outcome.as_reduced_value(), None);

    // 5. TypeStore remains clean of StaticValueRef
    let proj = TypeLevelProjection {
        subject: ProjectionSubject::Type(env.store.unit()),
        selector: TypeMemberSelector::named("At"),
        arguments: Box::new([TypeMemberArgument::Type(env.store.const_int(0))]),
        source_declaration: None,
    };
    let proj_ty = env.store.type_member_projection(proj, KindId::TYPE);
    assert!(matches!(env.store.get(proj_ty), TypeData::TypeMemberProjection(_)));
}
