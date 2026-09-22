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
    ConstTerm, IntrinsicTypeOperation, ProjectionSubject, TypeLevelProjection, TypeMemberArgument, TypeMemberSelector, lookup_intrinsic_type_operation,
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
                .map(|(label, ty)| phalcom_ast::ast::TypeTupleItem::Element(TypeTupleElement {
                    label: label.map(Into::into),
                    ty,
                    range: RANGE,
                }))
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

    // 2. Verify intrinsic lookup on "Tuple" produces IntrinsicTypeOperation::TupleAt for both
    assert_eq!(lookup_intrinsic_type_operation("Tuple", &sel_named), Some(IntrinsicTypeOperation::TupleAt));
    assert_eq!(lookup_intrinsic_type_operation("Tuple", &sel_subscript), Some(IntrinsicTypeOperation::TupleAt));

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

    let op_tuple = lookup_intrinsic_type_operation("Tuple", &map_selector);
    let op_record = lookup_intrinsic_type_operation("Record", &map_selector);
    let op_pack = lookup_intrinsic_type_operation("TypePack", &map_selector);
    let op_labeled_pack = lookup_intrinsic_type_operation("LabeledTypePack", &map_selector);
    let op_row = lookup_intrinsic_type_operation("RecordRow", &map_selector);

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
