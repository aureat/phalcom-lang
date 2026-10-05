//! LANG007.C1 closure integration suite certifying canonical member identity,
//! source target identity, associated compatibility, structural algebra composition,
//! zero products, metadata persistence, and bounded resource behavior.

use phalcom_ast::ast::{StaticSymbolRef, TypeAnnotation, TypeAnnotationExpr, TypeMemberArgumentSyntax, TypeMemberSelectorSyntax, TypeTupleElement};
use phalcom_common::range::SourceRange;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::declarations::{DeclarationTypeTable, bootstrap_universe_declarations};
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::metadata::MetadataExporter;
use phalcom_semantic::traits::{AssociatedTypeRequirementId, TraitRef};
use phalcom_semantic::types::annotation::{
    SimpleTypeResolver, TypeFormResolution, TypeFormationInvalid, TypeFormationSite, resolve_type_annotation, resolve_type_form,
};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::outcome::{CancellationToken, QueryBudget};
use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::projection::{ProjectionNormalizationContext, ProjectionNormalizationMode, ProjectionNormalizationResult, normalize_type};
use phalcom_semantic::types::store::{LabeledTypePackElement, TupleTypeElement, TypeData, TypeStore};
use phalcom_semantic::types::substitution::TypeSubstitution;
use phalcom_semantic::types::type_member::{
    ConstTerm, IntrinsicTypeOperation, IntrinsicTypeOwner, ProjectionSubject, TypeLevelProjection, TypeMemberArgument, TypeMemberSelector,
    lookup_intrinsic_type_operation,
};
use phalcom_type_meta::header::MetadataProfile;
use phalcom_type_meta::type_node::TypeNode;
use phalcom_type_meta::validate::{ValidationLimits, validate_metadata_bundle};

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

fn at_ann(subject: TypeAnnotation, index: i64) -> TypeAnnotation {
    TypeAnnotation {
        expr: TypeAnnotationExpr::TypeMemberProjection {
            subject: Box::new(subject),
            selector: TypeMemberSelectorSyntax::Named {
                name: "At".into(),
                range: RANGE,
            },
            arguments: vec![TypeMemberArgumentSyntax::IntLiteral { value: index, range: RANGE }],
            range: RANGE,
        },
        range: RANGE,
    }
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
    ] {
        resolver.insert(name, decl(name));
    }
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
    use phalcom_semantic::types::type_member::{TypeMemberDeclarationId, TypeMemberDomain, TypeMemberParameter, TypeMemberSignature};

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
    let const_int = store.static_domain_kind(store.const_int_domain());
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
    add(
        resolver,
        tuple.clone(),
        TypeMemberSelector::named("At"),
        IntrinsicTypeOperation::TupleAt,
        vec![("index", const_int)],
        KindId::TYPE,
    );
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
}

// ---------------------------------------------------------------------------
// 1. Alias Identity: T[I] and T::At<I> denote one canonical Subscript member
// ---------------------------------------------------------------------------

#[test]
fn test_alias_identity_subscript_and_at_converge() {
    let mut env = setup();
    let int_id = env.store.nominal(decl("Int"));
    let str_id = env.store.nominal(decl("String"));
    let tup = tuple_ann(vec![(None, ref_ann("Int")), (Some("name"), ref_ann("String"))]);

    let mut diags = Vec::new();

    // 1. Verify syntax selector lowering produces canonical TypeMemberSelector::Subscript
    let named_at_syntax = TypeMemberSelectorSyntax::Named {
        name: "At".into(),
        range: RANGE,
    };
    let subscript_syntax = TypeMemberSelectorSyntax::Subscript { range: RANGE };

    let sel_named = TypeMemberSelector::from_syntax(&named_at_syntax);
    let sel_subscript = TypeMemberSelector::from_syntax(&subscript_syntax);

    assert_eq!(sel_named, TypeMemberSelector::Subscript);
    assert_eq!(sel_subscript, TypeMemberSelector::Subscript);
    assert_eq!(sel_named, sel_subscript);

    // 2. Verify canonical Tuple owner lookup produces IntrinsicTypeOperation::TupleAt for both
    assert_eq!(
        lookup_intrinsic_type_operation(IntrinsicTypeOwner::Tuple, &sel_named),
        Some(IntrinsicTypeOperation::TupleAt)
    );
    assert_eq!(
        lookup_intrinsic_type_operation(IntrinsicTypeOwner::Tuple, &sel_subscript),
        Some(IntrinsicTypeOperation::TupleAt)
    );

    // 3. Verify resolution of T[0] and T::At<0> yield identical canonical proper type Int
    let res_subscript_0 = resolve_type_annotation(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &subscript_ann(tup.clone(), 0),
        &mut diags,
    );
    let res_at_0 = resolve_type_annotation(&mut env.store, &env.declarations, &env.resolver, &env.site, &at_ann(tup.clone(), 0), &mut diags);
    assert_eq!(res_subscript_0.ty(), Some(int_id));
    assert_eq!(res_at_0.ty(), Some(int_id));
    assert_eq!(res_subscript_0.ty(), res_at_0.ty());

    // 4. Verify resolution of T[1] and T::At<1> yield identical canonical proper type String
    let res_subscript_1 = resolve_type_annotation(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &subscript_ann(tup.clone(), 1),
        &mut diags,
    );
    let res_at_1 = resolve_type_annotation(&mut env.store, &env.declarations, &env.resolver, &env.site, &at_ann(tup.clone(), 1), &mut diags);
    assert_eq!(res_subscript_1.ty(), Some(str_id));
    assert_eq!(res_at_1.ty(), Some(str_id));
    assert_eq!(res_subscript_1.ty(), res_at_1.ty());

    // 5. Negative index equivalence: T[-1] and T::At<-1>
    let res_subscript_neg1 = resolve_type_annotation(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &subscript_ann(tup.clone(), -1),
        &mut diags,
    );
    let res_at_neg1 = resolve_type_annotation(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &at_ann(tup.clone(), -1),
        &mut diags,
    );
    assert_eq!(res_subscript_neg1.ty(), Some(str_id));
    assert_eq!(res_at_neg1.ty(), Some(str_id));
    assert_eq!(res_subscript_neg1.ty(), res_at_neg1.ty());
    assert!(diags.is_empty());
}

// ---------------------------------------------------------------------------
// 2. Domain Identity: Same-spelled Map members on distinct domains are distinct
// ---------------------------------------------------------------------------

#[test]
fn test_domain_identity_map_members_remain_distinct() {
    let map_selector = TypeMemberSelector::named("Map");

    let op_tuple = lookup_intrinsic_type_operation(IntrinsicTypeOwner::Tuple, &map_selector);
    let op_record = lookup_intrinsic_type_operation(IntrinsicTypeOwner::Record, &map_selector);
    let op_pack = lookup_intrinsic_type_operation(IntrinsicTypeOwner::TypePack, &map_selector);
    let op_labeled_pack = lookup_intrinsic_type_operation(IntrinsicTypeOwner::LabeledTypePack, &map_selector);
    let op_row = lookup_intrinsic_type_operation(IntrinsicTypeOwner::RecordRow, &map_selector);

    assert_eq!(op_tuple, Some(IntrinsicTypeOperation::TupleMap));
    assert_eq!(op_record, Some(IntrinsicTypeOperation::RecordMap));
    assert_eq!(op_pack, Some(IntrinsicTypeOperation::TypePackMap));
    assert_eq!(op_labeled_pack, Some(IntrinsicTypeOperation::LabeledTypePackMap));
    assert_eq!(op_row, Some(IntrinsicTypeOperation::RecordRowMap));

    // Ensure all 5 are pairwise distinct
    let ops = [op_tuple, op_record, op_pack, op_labeled_pack, op_row];
    for i in 0..ops.len() {
        for j in (i + 1)..ops.len() {
            assert_ne!(ops[i], ops[j], "ops[{i}] and ops[{j}] must be distinct");
        }
    }
}

#[test]
fn tuple_entries_is_complete_pack_projection_for_concrete_and_symbolic_tuples() {
    let mut env = setup();
    let tuple_syntax = tuple_ann(vec![(None, ref_ann("Int")), (None, ref_ann("String")), (Some("name"), ref_ann("Bool"))]);
    let mut diagnostics = Vec::new();
    let TypeFormResolution::Ready(tuple) = resolve_type_form(&mut env.store, &env.declarations, &env.resolver, &env.site, &tuple_syntax, &mut diagnostics)
    else {
        panic!("concrete tuple form must resolve");
    };
    assert!(diagnostics.is_empty(), "tuple form diagnostics: {diagnostics:?}");
    let entries_resolution = resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &proj_ann(tuple_syntax, "Entries", vec![]),
        &mut diagnostics,
    );
    let TypeFormResolution::Ready(entries) = entries_resolution else {
        panic!("Tuple::Entries resolves: {entries_resolution:?}; {diagnostics:?}");
    };
    assert_eq!(env.store.kind_of(entries), KindId::COMPLETE_TYPE_PACK);
    assert_eq!(entries, env.store.project_complete(tuple).expect("complete projection"));

    let owner = TypeParameterOwner::Declaration(decl("SymbolicTuple"));
    let positionals = env
        .store
        .intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "P", KindId::TYPE_PACK));
    let labeled = env
        .store
        .intern_type_parameter(TypeParameterData::new(owner.clone(), 1, "L", KindId::LABELED_TYPE_PACK));
    let positional_form = env.store.parameter_form(positionals);
    let labeled_form = env.store.parameter_form(labeled);
    let symbolic_tuple = env.store.tuple_from_packs(positional_form, labeled_form).expect("symbolic tuple from packs");
    let expected = env.store.project_complete(symbolic_tuple).expect("symbolic complete projection");
    let projection = env.store.type_member_projection(
        TypeLevelProjection {
            subject: ProjectionSubject::Type(symbolic_tuple),
            selector: TypeMemberSelector::named("Entries"),
            arguments: Box::new([]),
            source_declaration: None,
        },
        KindId::COMPLETE_TYPE_PACK,
    );
    let mut budget = QueryBudget::new(100);
    let cancel = CancellationToken::new();
    let conformance_index = phalcom_semantic::impls::ConformanceIndex::new();
    let witness_plans = std::collections::BTreeMap::new();
    let trait_surfaces = phalcom_semantic::traits::TraitSurfaceTable::new();
    let empty_hierarchy = phalcom_semantic::types::relation::MapTypeHierarchy::new();
    let mode = ProjectionNormalizationMode::Exact {
        conformance_index: &conformance_index,
        witness_plans: &witness_plans,
        trait_surfaces: &trait_surfaces,
        derived_rules: None,
        declarations: &env.declarations,
        hierarchy: &empty_hierarchy,
    };
    let mut context = ProjectionNormalizationContext::new(mode, &mut budget, &cancel);
    assert_eq!(
        normalize_type(&mut env.store, projection, &mut context),
        ProjectionNormalizationResult::Normalized(expected)
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn complete_type_pack_projections_and_map_preserve_both_lanes() {
    use phalcom_semantic::types::type_member::TypeMemberDeclarationId;

    let mut env = setup();
    let int_ty = env.store.nominal(decl("Int"));
    let bool_ty = env.store.nominal(decl("Bool"));
    let string_ty = env.store.nominal(decl("String"));
    let list_form = env.declarations.form(&decl("List")).expect("canonical List form");
    let owner = decl("CompleteTypePack");
    let positionals_member = TypeMemberDeclarationId::new(owner.clone(), TypeMemberSelector::named("Positionals"));
    let labeled_member = TypeMemberDeclarationId::new(owner.clone(), TypeMemberSelector::named("Labeled"));
    let map_member = TypeMemberDeclarationId::new(owner, TypeMemberSelector::named("Map"));

    let positionals = env.store.type_pack(Box::new([int_ty, bool_ty])).expect("positional lane");
    let labeled = env
        .store
        .labeled_type_pack(Box::new([
            LabeledTypePackElement {
                label: "name".into(),
                element: string_ty,
            },
            LabeledTypePackElement {
                label: "flag".into(),
                element: int_ty,
            },
        ]))
        .expect("labeled lane");
    let complete = env.store.complete_type_pack(positionals, labeled).expect("complete pack");

    let positional_projection = env.store.type_member_projection(
        TypeLevelProjection {
            subject: ProjectionSubject::Type(complete),
            selector: TypeMemberSelector::named("Positionals"),
            arguments: Box::new([]),
            source_declaration: Some(positionals_member.clone()),
        },
        KindId::TYPE_PACK,
    );
    let labeled_projection = env.store.type_member_projection(
        TypeLevelProjection {
            subject: ProjectionSubject::Type(complete),
            selector: TypeMemberSelector::named("Labeled"),
            arguments: Box::new([]),
            source_declaration: Some(labeled_member.clone()),
        },
        KindId::LABELED_TYPE_PACK,
    );
    let mapped_projection = env.store.type_member_projection(
        TypeLevelProjection {
            subject: ProjectionSubject::Type(complete),
            selector: TypeMemberSelector::named("Map"),
            arguments: Box::new([TypeMemberArgument::Type(list_form)]),
            source_declaration: Some(map_member.clone()),
        },
        KindId::COMPLETE_TYPE_PACK,
    );

    let mut budget = QueryBudget::new(200);
    let cancel = CancellationToken::new();
    let conformance_index = phalcom_semantic::impls::ConformanceIndex::new();
    let witness_plans = std::collections::BTreeMap::new();
    let trait_surfaces = phalcom_semantic::traits::TraitSurfaceTable::new();
    let hierarchy = phalcom_semantic::types::relation::MapTypeHierarchy::new();
    let mode = ProjectionNormalizationMode::Exact {
        conformance_index: &conformance_index,
        witness_plans: &witness_plans,
        trait_surfaces: &trait_surfaces,
        derived_rules: None,
        declarations: &env.declarations,
        hierarchy: &hierarchy,
    };
    let mut context = ProjectionNormalizationContext::new(mode, &mut budget, &cancel);

    assert_eq!(
        normalize_type(&mut env.store, positional_projection, &mut context),
        ProjectionNormalizationResult::Normalized(positionals),
        "Positionals returns the exact positional lane"
    );
    assert_eq!(
        normalize_type(&mut env.store, labeled_projection, &mut context),
        ProjectionNormalizationResult::Normalized(labeled),
        "Labeled returns the exact labeled lane"
    );

    let list_int = env.store.apply_type_form(list_form, &[int_ty]).expect("List<Int>");
    let list_bool = env.store.apply_type_form(list_form, &[bool_ty]).expect("List<Bool>");
    let list_string = env.store.apply_type_form(list_form, &[string_ty]).expect("List<String>");
    let expected_positionals = env.store.type_pack(Box::new([list_int, list_bool])).expect("mapped positional lane");
    let expected_labeled = env
        .store
        .labeled_type_pack(Box::new([
            LabeledTypePackElement {
                label: "name".into(),
                element: list_string,
            },
            LabeledTypePackElement {
                label: "flag".into(),
                element: list_int,
            },
        ]))
        .expect("mapped labeled lane");
    let expected_complete = env
        .store
        .complete_type_pack(expected_positionals, expected_labeled)
        .expect("mapped complete pack");
    assert_eq!(
        normalize_type(&mut env.store, mapped_projection, &mut context),
        ProjectionNormalizationResult::Normalized(expected_complete),
        "Map applies independently to both lanes without moving elements across them"
    );
    let TypeData::CompleteTypePack {
        positionals: mapped_pos,
        labeled: mapped_lab,
    } = env.store.get(expected_complete)
    else {
        panic!("mapped result must remain a CompleteTypePack");
    };
    assert_eq!(*mapped_pos, expected_positionals);
    assert_eq!(*mapped_lab, expected_labeled);

    let symbolic_lane_owner = TypeParameterOwner::Declaration(decl("SymbolicCompletePackLanes"));
    let positional_parameter = env
        .store
        .intern_type_parameter(TypeParameterData::new(symbolic_lane_owner.clone(), 0, "P", KindId::TYPE_PACK));
    let labeled_parameter = env
        .store
        .intern_type_parameter(TypeParameterData::new(symbolic_lane_owner, 1, "L", KindId::LABELED_TYPE_PACK));
    let symbolic_positional_form = env.store.parameter_form(positional_parameter);
    let symbolic_labeled_form = env.store.parameter_form(labeled_parameter);
    let symbolic_complete = env
        .store
        .complete_type_pack(symbolic_positional_form, symbolic_labeled_form)
        .expect("symbolic CompleteTypePack lanes");
    let symbolic_lane_map = env.store.type_member_projection(
        TypeLevelProjection {
            subject: ProjectionSubject::Type(symbolic_complete),
            selector: TypeMemberSelector::named("Map"),
            arguments: Box::new([TypeMemberArgument::Type(list_form)]),
            source_declaration: Some(map_member.clone()),
        },
        KindId::COMPLETE_TYPE_PACK,
    );
    assert_eq!(
        normalize_type(&mut env.store, symbolic_lane_map, &mut context),
        ProjectionNormalizationResult::Symbolic(symbolic_lane_map),
        "a complete pack with unresolved lane contents retains its canonical Map projection"
    );

    let symbolic_parameter = env.store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(decl("SymbolicCompletePack")),
        0,
        "P",
        KindId::COMPLETE_TYPE_PACK,
    ));
    let symbolic_subject = env.store.parameter_form(symbolic_parameter);
    for (selector, arguments, source_declaration, expected_kind) in [
        (
            TypeMemberSelector::named("Positionals"),
            Box::new([]) as Box<[TypeMemberArgument]>,
            positionals_member,
            KindId::TYPE_PACK,
        ),
        (
            TypeMemberSelector::named("Labeled"),
            Box::new([]) as Box<[TypeMemberArgument]>,
            labeled_member,
            KindId::LABELED_TYPE_PACK,
        ),
        (
            TypeMemberSelector::named("Map"),
            Box::new([TypeMemberArgument::Type(list_form)]) as Box<[TypeMemberArgument]>,
            map_member,
            KindId::COMPLETE_TYPE_PACK,
        ),
    ] {
        let symbolic_projection = env.store.type_member_projection(
            TypeLevelProjection {
                subject: ProjectionSubject::Type(symbolic_subject),
                selector: selector.clone(),
                arguments: arguments.clone(),
                source_declaration: Some(source_declaration.clone()),
            },
            expected_kind,
        );
        assert_eq!(
            normalize_type(&mut env.store, symbolic_projection, &mut context),
            ProjectionNormalizationResult::Symbolic(symbolic_projection),
            "symbolic CompleteTypePack projections remain canonical residual projections"
        );
        let TypeData::TypeMemberProjection(residual) = env.store.get(symbolic_projection) else {
            panic!("symbolic result must preserve its projection node");
        };
        assert_eq!(residual.subject, ProjectionSubject::Type(symbolic_subject));
        assert_eq!(residual.selector, selector);
        assert_eq!(residual.arguments.as_ref(), arguments.as_ref());
        assert_eq!(residual.source_declaration.as_ref(), Some(&source_declaration));
    }
}

#[test]
fn tuple_element_unions_payloads_and_preserves_unsolved_pack_projection() {
    use phalcom_semantic::types::type_member::TypeMemberDeclarationId;

    let mut env = setup();
    let tuple_owner = decl("Tuple");
    let element_member = TypeMemberDeclarationId::new(tuple_owner.clone(), TypeMemberSelector::named("Element"));
    let int_ty = env.store.nominal(decl("Int"));
    let bool_ty = env.store.nominal(decl("Bool"));
    let string_ty = env.store.nominal(decl("String"));

    let mixed_tuple = env.store.tuple(Box::new([
        TupleTypeElement::unlabeled(int_ty),
        TupleTypeElement::unlabeled(bool_ty),
        TupleTypeElement::labeled("name".into(), string_ty),
    ]));
    let mixed_element = env.store.type_member_projection(
        TypeLevelProjection {
            subject: ProjectionSubject::Type(mixed_tuple),
            selector: TypeMemberSelector::named("Element"),
            arguments: Box::new([]),
            source_declaration: Some(element_member.clone()),
        },
        KindId::TYPE,
    );
    let empty_tuple = env.store.tuple(Box::new([]));
    let empty_element = env.store.type_member_projection(
        TypeLevelProjection {
            subject: ProjectionSubject::Type(empty_tuple),
            selector: TypeMemberSelector::named("Element"),
            arguments: Box::new([]),
            source_declaration: Some(element_member.clone()),
        },
        KindId::TYPE,
    );

    let mut budget = QueryBudget::new(200);
    let cancel = CancellationToken::new();
    let conformance_index = phalcom_semantic::impls::ConformanceIndex::new();
    let witness_plans = std::collections::BTreeMap::new();
    let trait_surfaces = phalcom_semantic::traits::TraitSurfaceTable::new();
    let hierarchy = phalcom_semantic::types::relation::MapTypeHierarchy::new();
    let mode = ProjectionNormalizationMode::Exact {
        conformance_index: &conformance_index,
        witness_plans: &witness_plans,
        trait_surfaces: &trait_surfaces,
        derived_rules: None,
        declarations: &env.declarations,
        hierarchy: &hierarchy,
    };
    let mut context = ProjectionNormalizationContext::new(mode, &mut budget, &cancel);

    let expected_union = env.store.union(&[int_ty, bool_ty, string_ty]);
    assert_eq!(
        normalize_type(&mut env.store, mixed_element, &mut context),
        ProjectionNormalizationResult::Normalized(expected_union),
        "Element unions both positional and labeled payloads"
    );
    assert_eq!(
        normalize_type(&mut env.store, empty_element, &mut context),
        ProjectionNormalizationResult::Normalized(env.store.never()),
        "the empty Tuple element union is Never"
    );

    let symbolic_owner = TypeParameterOwner::Declaration(decl("SymbolicTupleElement"));
    let positionals = env
        .store
        .intern_type_parameter(TypeParameterData::new(symbolic_owner.clone(), 0, "Ps", KindId::TYPE_PACK));
    let labeled = env
        .store
        .intern_type_parameter(TypeParameterData::new(symbolic_owner, 1, "Ls", KindId::LABELED_TYPE_PACK));
    let positionals_form = env.store.parameter_form(positionals);
    let labeled_form = env.store.parameter_form(labeled);
    let symbolic_tuple = env
        .store
        .tuple_from_packs(positionals_form, labeled_form)
        .expect("symbolic tuple from unresolved lanes");
    assert!(matches!(env.store.get(symbolic_tuple), TypeData::TupleFromPacks { .. }));
    let symbolic_element = env.store.type_member_projection(
        TypeLevelProjection {
            subject: ProjectionSubject::Type(symbolic_tuple),
            selector: TypeMemberSelector::named("Element"),
            arguments: Box::new([]),
            source_declaration: Some(element_member.clone()),
        },
        KindId::TYPE,
    );
    let ProjectionNormalizationResult::Symbolic(residual_id) = normalize_type(&mut env.store, symbolic_element, &mut context) else {
        panic!("an unenumerable TupleFromPacks Element must remain symbolic");
    };
    let TypeData::TypeMemberProjection(residual) = env.store.get(residual_id) else {
        panic!("symbolic Element must remain a type-member projection");
    };
    assert_eq!(residual.selector, TypeMemberSelector::named("Element"));
    assert_eq!(residual.subject, ProjectionSubject::Type(symbolic_tuple));
    assert_eq!(residual.source_declaration.as_ref(), Some(&element_member));
    assert_eq!(env.store.kind_of(residual_id), KindId::TYPE);
}

#[test]
fn record_value_unions_closed_fields_and_preserves_open_row_projection() {
    use phalcom_semantic::types::row::{RecordRowField, RecordRowTail};
    use phalcom_semantic::types::type_member::TypeMemberDeclarationId;

    let mut env = setup();
    let value_member = TypeMemberDeclarationId::new(decl("Record"), TypeMemberSelector::named("Value"));
    let int_ty = env.store.nominal(decl("Int"));
    let bool_ty = env.store.nominal(decl("Bool"));
    let string_ty = env.store.nominal(decl("String"));

    let closed_record = env.store.record(Box::new([
        RecordRowField::symbol_required("count", int_ty),
        RecordRowField::symbol_required("enabled", bool_ty),
        RecordRowField::symbol_required("label", string_ty),
    ]));
    let closed_value = env.store.type_member_projection(
        TypeLevelProjection {
            subject: ProjectionSubject::Type(closed_record),
            selector: TypeMemberSelector::named("Value"),
            arguments: Box::new([]),
            source_declaration: Some(value_member.clone()),
        },
        KindId::TYPE,
    );
    let empty_record = env.store.record(Box::new([]));
    let empty_value = env.store.type_member_projection(
        TypeLevelProjection {
            subject: ProjectionSubject::Type(empty_record),
            selector: TypeMemberSelector::named("Value"),
            arguments: Box::new([]),
            source_declaration: Some(value_member.clone()),
        },
        KindId::TYPE,
    );

    let mut budget = QueryBudget::new(200);
    let cancel = CancellationToken::new();
    let conformance_index = phalcom_semantic::impls::ConformanceIndex::new();
    let witness_plans = std::collections::BTreeMap::new();
    let trait_surfaces = phalcom_semantic::traits::TraitSurfaceTable::new();
    let hierarchy = phalcom_semantic::types::relation::MapTypeHierarchy::new();
    let mode = ProjectionNormalizationMode::Exact {
        conformance_index: &conformance_index,
        witness_plans: &witness_plans,
        trait_surfaces: &trait_surfaces,
        derived_rules: None,
        declarations: &env.declarations,
        hierarchy: &hierarchy,
    };
    let mut context = ProjectionNormalizationContext::new(mode, &mut budget, &cancel);

    let expected_union = env.store.union(&[int_ty, bool_ty, string_ty]);
    assert_eq!(
        normalize_type(&mut env.store, closed_value, &mut context),
        ProjectionNormalizationResult::Normalized(expected_union),
        "Value unions every payload in a closed Record row"
    );
    let never = env.store.never();
    assert_eq!(
        normalize_type(&mut env.store, empty_value, &mut context),
        ProjectionNormalizationResult::Normalized(never),
        "Value of an empty closed Record is Never"
    );

    let row_parameter = env.store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(decl("SymbolRecordValue")),
        0,
        "R",
        KindId::RECORD_ROW,
    ));
    let open_record = env
        .store
        .record_row_type_checked(vec![RecordRowField::symbol_required("known", int_ty)], RecordRowTail::Parameter(row_parameter))
        .expect("open Record row with a canonical row-variable tail");
    let open_value = env.store.type_member_projection(
        TypeLevelProjection {
            subject: ProjectionSubject::Type(open_record),
            selector: TypeMemberSelector::named("Value"),
            arguments: Box::new([]),
            source_declaration: Some(value_member.clone()),
        },
        KindId::TYPE,
    );
    let ProjectionNormalizationResult::Symbolic(residual_id) = normalize_type(&mut env.store, open_value, &mut context) else {
        panic!("Value cannot enumerate a complete payload set for an open Record row");
    };
    assert_ne!(residual_id, int_ty, "the known field must not be unioned while ignoring the open tail");
    let TypeData::TypeMemberProjection(residual) = env.store.get(residual_id) else {
        panic!("open-row Value must remain a type-member projection");
    };
    assert_eq!(residual.selector, TypeMemberSelector::named("Value"));
    assert_eq!(residual.subject, ProjectionSubject::Type(open_record));
    assert_eq!(residual.source_declaration.as_ref(), Some(&value_member));
    assert_eq!(env.store.kind_of(residual_id), KindId::TYPE);
}

// ---------------------------------------------------------------------------
// 3. Associated-Type Compatibility: Trait projections remain evidence-resolved
// ---------------------------------------------------------------------------

#[test]
fn test_associated_type_compatibility_preserves_trait_projections() {
    let mut store = TypeStore::new();
    let module = ModuleId::universe_root();
    let box_decl = DeclarationId::new(module.clone(), "Box".into());
    let cell_trait = DeclarationId::new(module.clone(), "Cell".into());

    let subject = store.nominal(box_decl);
    let requirement = AssociatedTypeRequirementId::new(cell_trait.clone(), 0);
    let trait_ref = TraitRef::new(cell_trait, Vec::new().into_boxed_slice());

    // Construct trait associated projection
    let associated_proj = store.associated_projection(subject, trait_ref, requirement);

    // Verify it is an AssociatedProjection, not a TypeMemberProjection
    let data = store.get(associated_proj);
    assert!(
        matches!(data, TypeData::AssociatedProjection { .. }),
        "Trait associated projection must retain distinct AssociatedProjection type data"
    );

    // Verify metadata export preserves AssociatedProjection structure
    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let exported = exporter.export_type_form(associated_proj).expect("export associated projection");
    let bundle = exporter.build_bundle(&[]).expect("build bundle");
    let node = &bundle.types[exported.0 as usize];
    assert!(matches!(&node.form, TypeNode::AssociatedProjection { .. }));
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).expect("valid metadata");
}

// ---------------------------------------------------------------------------
// 4. Structural Map/At Composition Law: (T::Map<F>)::At<I> == F<T::At<I>>
// ---------------------------------------------------------------------------

#[test]
fn test_structural_map_at_composition_law() {
    let mut env = setup();
    let int_id = env.store.nominal(decl("Int"));
    let str_id = env.store.nominal(decl("String"));
    let bool_id = env.store.nominal(decl("Bool"));
    let list_id = env.declarations.form(&decl("List")).unwrap();

    // T = (Int, Bool, name: String)
    let tup = tuple_ann(vec![(None, ref_ann("Int")), (None, ref_ann("Bool")), (Some("name"), ref_ann("String"))]);

    // T::Map<List>
    let mapped_tup = proj_ann(tup.clone(), "Map", vec![TypeMemberArgumentSyntax::TypeForm(ref_ann("List"))]);

    let mut diags = Vec::new();

    // For each valid index i in [0, 1, 2]:
    // (T::Map<List>)::At<i> must equal List<T::At<i>>
    let expected_elements = [int_id, bool_id, str_id];
    for (i, &elem_ty) in expected_elements.iter().enumerate() {
        // (T::Map<List>)::At<i>
        let lhs_ann = at_ann(mapped_tup.clone(), i as i64);
        let lhs_res = resolve_type_annotation(&mut env.store, &env.declarations, &env.resolver, &env.site, &lhs_ann, &mut diags);

        // List<T::At<i>> -> apply List to element type
        let rhs_ty = env.store.apply_type_form(list_id, &[elem_ty]).unwrap();

        assert_eq!(
            lhs_res.ty(),
            Some(rhs_ty),
            "Composition law failed at index {i}: (T::Map<List>)::At<{i}> != List<T::At<{i}>>"
        );
    }
    assert!(diags.is_empty());
}

// ---------------------------------------------------------------------------
// 5. Zero Products: Unit vs Empty Record Static Distinction
// ---------------------------------------------------------------------------

#[test]
fn test_zero_product_semantics_unit_vs_empty_record() {
    let mut env = setup();

    let mut diags = Vec::new();

    // 1. Tuple([], []) ≡ Unit
    let empty_tuple = tuple_ann(vec![]);
    let unit_res = resolve_type_annotation(&mut env.store, &env.declarations, &env.resolver, &env.site, &empty_tuple, &mut diags);
    assert_eq!(unit_res.ty(), Some(env.store.unit()));

    // 2. Record({}) is a distinct static Record type
    let empty_record = record_ann(vec![]);
    let empty_rec_res = resolve_type_annotation(&mut env.store, &env.declarations, &env.resolver, &env.site, &empty_record, &mut diags);
    let expected_rec = env.store.record(Box::new([]));
    assert_eq!(empty_rec_res.ty(), Some(expected_rec));
    assert_ne!(unit_res.ty(), empty_rec_res.ty());

    // 3. Mapping empty tuple -> Unit
    let mapped_unit = proj_ann(empty_tuple, "Map", vec![TypeMemberArgumentSyntax::TypeForm(ref_ann("List"))]);
    let mapped_unit_res = resolve_type_annotation(&mut env.store, &env.declarations, &env.resolver, &env.site, &mapped_unit, &mut diags);
    assert_eq!(mapped_unit_res.ty(), Some(env.store.unit()));

    // 4. Mapping empty record -> Record({})
    let mapped_rec = proj_ann(empty_record, "Map", vec![TypeMemberArgumentSyntax::TypeForm(ref_ann("List"))]);
    let mapped_rec_res = resolve_type_annotation(&mut env.store, &env.declarations, &env.resolver, &env.site, &mapped_rec, &mut diags);
    assert_eq!(mapped_rec_res.ty(), Some(expected_rec));
    assert!(diags.is_empty());
}

// ---------------------------------------------------------------------------
// 6. Index Boundary and Out-of-Bounds Validation
// ---------------------------------------------------------------------------

#[test]
fn test_index_boundaries_and_out_of_bounds() {
    let mut env = setup();
    let int_id = env.store.nominal(decl("Int"));
    let str_id = env.store.nominal(decl("String"));
    let tup = tuple_ann(vec![(None, ref_ann("Int")), (Some("name"), ref_ann("String"))]);

    let mut diags = Vec::new();

    // Valid: 0 -> Int
    let r0 = resolve_type_annotation(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &subscript_ann(tup.clone(), 0),
        &mut diags,
    );
    assert_eq!(r0.ty(), Some(int_id));

    // Valid: 1 -> String
    let r1 = resolve_type_annotation(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &subscript_ann(tup.clone(), 1),
        &mut diags,
    );
    assert_eq!(r1.ty(), Some(str_id));

    // Valid: -1 -> String
    let r_neg1 = resolve_type_annotation(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &subscript_ann(tup.clone(), -1),
        &mut diags,
    );
    assert_eq!(r_neg1.ty(), Some(str_id));

    // Valid: -2 -> Int
    let r_neg2 = resolve_type_annotation(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &subscript_ann(tup.clone(), -2),
        &mut diags,
    );
    assert_eq!(r_neg2.ty(), Some(int_id));

    // Invalid positive OOB: 2 (arity is 2) -> returns Invalid(Syntax) and pushes diagnostic
    let r_oob_pos = resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &subscript_ann(tup.clone(), 2),
        &mut diags,
    );
    assert_eq!(r_oob_pos, TypeFormResolution::Invalid(TypeFormationInvalid::Syntax));

    // Invalid negative OOB: -3 (min valid is -2) -> returns Invalid(Syntax) and pushes diagnostic
    let r_oob_neg = resolve_type_form(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &subscript_ann(tup.clone(), -3),
        &mut diags,
    );
    assert_eq!(r_oob_neg, TypeFormResolution::Invalid(TypeFormationInvalid::Syntax));
}

// ---------------------------------------------------------------------------
// 7. Generic Substitution and Symbolic Projection Preservation
// ---------------------------------------------------------------------------

#[test]
fn test_symbolic_projection_survives_generic_substitution() {
    let mut store = TypeStore::new();
    let module = ModuleId::universe_root();
    let decl_owner = DeclarationId::new(module.clone(), "Container".into());
    let owner = TypeParameterOwner::Declaration(decl_owner);

    // Generic parameter T of kind Type
    let param_t = store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "T", KindId::TYPE));
    let type_t = store.parameter_form(param_t);

    // Int proper type
    let int_id = store.nominal(DeclarationId::new(module.clone(), "Int".into()));

    // Symbolic tuple: (T, Int)
    let sym_tuple = store.tuple(Box::new([TupleTypeElement::unlabeled(type_t), TupleTypeElement::unlabeled(int_id)]));

    // Symbolic projection: (T, Int)::At<0>
    let proj_0 = store.type_member_projection(
        TypeLevelProjection {
            subject: ProjectionSubject::Type(sym_tuple),
            selector: TypeMemberSelector::Subscript,
            arguments: Box::new([TypeMemberArgument::Const(ConstTerm::Int(0))]),
            source_declaration: None,
        },
        KindId::TYPE,
    );

    // Substitute T => String
    let str_id = store.nominal(DeclarationId::new(module.clone(), "String".into()));
    let mut subst = TypeSubstitution::new();
    subst.bind(param_t, str_id);
    let substituted = subst.apply(&mut store, proj_0);

    // Now normalize the substituted projection: (String, Int)::At<0> -> String
    let mut budget = QueryBudget::new(100);
    let cancel = CancellationToken::new();
    let conformance_index = phalcom_semantic::impls::ConformanceIndex::new();
    let witness_plans = std::collections::BTreeMap::new();
    let trait_surfaces = phalcom_semantic::traits::TraitSurfaceTable::new();
    let empty_hier = phalcom_semantic::types::relation::MapTypeHierarchy::new();
    let declarations = DeclarationTypeTable::new();
    let mode = ProjectionNormalizationMode::Exact {
        conformance_index: &conformance_index,
        witness_plans: &witness_plans,
        trait_surfaces: &trait_surfaces,
        derived_rules: None,
        declarations: &declarations,
        hierarchy: &empty_hier,
    };
    let mut context = ProjectionNormalizationContext::new(mode, &mut budget, &cancel);

    let norm_res = normalize_type(&mut store, substituted, &mut context);
    assert_eq!(norm_res, ProjectionNormalizationResult::Normalized(str_id));
}

// ---------------------------------------------------------------------------
// 8. Stable Metadata Export and Deterministic Fingerprints
// ---------------------------------------------------------------------------

#[test]
fn test_stable_metadata_export_for_packs_and_projections() {
    let mut store = TypeStore::new();
    let module = ModuleId::universe_root();
    let int_id = store.nominal(DeclarationId::new(module.clone(), "Int".into()));
    let str_id = store.nominal(DeclarationId::new(module.clone(), "String".into()));

    // 1. Export TypePack
    let pack = store.type_pack(Box::new([int_id, str_id])).unwrap();
    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let pack_node = exporter.export_type_form(pack).expect("export type pack");
    let bundle = exporter.build_bundle(&[]).expect("build bundle with pack");
    assert!(matches!(&bundle.types[pack_node.0 as usize].form, TypeNode::TypePack(elems) if elems.len() == 2));
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).expect("valid pack bundle");

    // 2. Export LabeledTypePack
    let labeled_pack = store
        .labeled_type_pack(Box::new([
            LabeledTypePackElement {
                label: "first".into(),
                element: int_id,
            },
            LabeledTypePackElement {
                label: "second".into(),
                element: str_id,
            },
        ]))
        .unwrap();
    let mut exporter2 = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let lab_node = exporter2.export_type_form(labeled_pack).expect("export labeled type pack");
    let bundle2 = exporter2.build_bundle(&[]).expect("build bundle with labeled pack");
    assert!(matches!(&bundle2.types[lab_node.0 as usize].form, TypeNode::LabeledTypePack(elems) if elems.len() == 2));
    validate_metadata_bundle(&bundle2, &ValidationLimits::default()).expect("valid labeled pack bundle");

    // 3. Verify fingerprint determinism across separate store instances
    let mut store_alt = TypeStore::new();
    let int_alt = store_alt.nominal(DeclarationId::new(module.clone(), "Int".into()));
    let str_alt = store_alt.nominal(DeclarationId::new(module.clone(), "String".into()));
    let pack_alt = store_alt.type_pack(Box::new([int_alt, str_alt])).unwrap();
    let mut exporter_alt = MetadataExporter::new(&store_alt, None, None, None, MetadataProfile::RuntimePublic);
    let pack_alt_node = exporter_alt.export_type_form(pack_alt).expect("export alt pack");
    let bundle_alt = exporter_alt.build_bundle(&[]).expect("build alt bundle");

    assert_eq!(
        bundle.types[pack_node.0 as usize].structural_fingerprint, bundle_alt.types[pack_alt_node.0 as usize].structural_fingerprint,
        "Structural fingerprints must match across independent store instances"
    );
}

// ---------------------------------------------------------------------------
// 9. Moderate Density Structural Operations (e.g. 64-element Tuple)
// ---------------------------------------------------------------------------

#[test]
fn test_moderate_density_tuple_indexing_and_mapping() {
    let mut env = setup();
    let int_id = env.store.nominal(decl("Int"));
    let str_id = env.store.nominal(decl("String"));

    const COUNT: usize = 64;
    const POS_COUNT: usize = 32;

    // Create 64 elements: 32 positional Ints followed by 32 labeled Strings
    let mut elements = Vec::with_capacity(COUNT);
    for _ in 0..POS_COUNT {
        elements.push((None, ref_ann("Int")));
    }
    for i in 0..(COUNT - POS_COUNT) {
        elements.push((Some(Box::leak(format!("f{i}").into_boxed_str()) as &str), ref_ann("String")));
    }
    let dense_tup = tuple_ann(elements);

    let mut diags = Vec::new();

    // Verify indexing at all positions in O(1) shape access
    for i in 0..COUNT {
        let ann = subscript_ann(dense_tup.clone(), i as i64);
        let res = resolve_type_annotation(&mut env.store, &env.declarations, &env.resolver, &env.site, &ann, &mut diags);
        let expected = if i < POS_COUNT { int_id } else { str_id };
        assert_eq!(res.ty(), Some(expected), "Dense indexing mismatch at {i}");
    }

    // Verify negative indexing at -1 and -COUNT
    let last = resolve_type_annotation(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &subscript_ann(dense_tup.clone(), -1),
        &mut diags,
    );
    assert_eq!(last.ty(), Some(str_id));

    let first = resolve_type_annotation(
        &mut env.store,
        &env.declarations,
        &env.resolver,
        &env.site,
        &subscript_ann(dense_tup.clone(), -(COUNT as i64)),
        &mut diags,
    );
    assert_eq!(first.ty(), Some(int_id));

    // Verify Map over 64 elements
    let map_ann = proj_ann(dense_tup, "Map", vec![TypeMemberArgumentSyntax::TypeForm(ref_ann("List"))]);
    let mapped_res = resolve_type_annotation(&mut env.store, &env.declarations, &env.resolver, &env.site, &map_ann, &mut diags);
    assert!(mapped_res.ty().is_some());
    assert!(diags.is_empty());
}
