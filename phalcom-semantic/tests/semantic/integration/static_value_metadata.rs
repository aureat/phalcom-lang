//! Integration tests for durable StaticValue metadata projection and fingerprints (P8.T2 / G2).

use num_bigint::BigInt;
use phalcom_modules::ProjectUniverse;
use phalcom_semantic::core_surface::universe_declaration;
use phalcom_semantic::field_key::FieldKey;
use phalcom_semantic::metadata::stable_identity::StableIdentityContext;
use phalcom_semantic::metadata::static_value::StaticValueTableExporter;
use phalcom_semantic::scalar::StaticInt;
use phalcom_semantic::static_value::{StaticFloat, StaticValueStore};
use phalcom_semantic::types::store::{StaticArgument, TypeData};
use phalcom_semantic::types::{KindId, TypeId, TypeStore};
use phalcom_type_meta::field_key::FieldKeyRef;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::static_value::{StaticSequenceFamilyRef, StaticValueNode};
use phalcom_type_meta::type_node::TypeNodeId;

fn test_universe() -> ProjectUniverse {
    ProjectUniverse::new()
}

fn array_type(types: &mut TypeStore, element: TypeId) -> TypeId {
    let origin_kind = types.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let origin = types.intern_with_kind(
        TypeData::Nominal {
            declaration: universe_declaration(phalcom_native_meta::UniverseKey::Array),
        },
        origin_kind,
    );
    types
        .apply_static_form(origin, &[StaticArgument::Term(element)])
        .expect("Array type application")
}

fn vector_type(types: &mut TypeStore, element: TypeId, extent: BigInt) -> TypeId {
    let static_int_kind = types.static_domain_kind(types.const_int_domain());
    let origin_kind = types.arrow_kind(vec![KindId::TYPE, static_int_kind].into_boxed_slice(), KindId::TYPE);
    let origin = types.intern_with_kind(
        TypeData::Nominal {
            declaration: universe_declaration(phalcom_native_meta::UniverseKey::Vector),
        },
        origin_kind,
    );
    let extent = types.const_int(extent);
    types
        .apply_static_form(
            origin,
            &[StaticArgument::Term(element), StaticArgument::Term(extent)],
        )
        .expect("Vector type application")
}

fn dummy_type_exporter() -> impl FnMut(TypeId) -> Result<(TypeNodeId, Fingerprint128), phalcom_semantic::metadata::export::MetadataExportError> {
    |ty: TypeId| Ok((TypeNodeId(ty.index() as u32), Fingerprint128::from_u128(ty.index() as u128)))
}

#[test]
fn static_value_metadata_primitives_scalars_and_floats() {
    let universe = test_universe();
    let context = StableIdentityContext::new(&universe);
    let mut store = StaticValueStore::new();

    let u_ref = store.intern_unit();
    let b_true_ref = store.intern_bool(true);
    let b_false_ref = store.intern_bool(false);
    let c_ref = store.intern_char('x');
    let c_unicorn = store.intern_char('\u{1F984}');

    // Exact bits for +0.0 and -0.0:
    let f_pos_zero = store.intern_float(StaticFloat::from_f64(0.0));
    let f_neg_zero = store.intern_float(StaticFloat::from_f64(-0.0));

    // Exact NaN bit payload:
    let nan_bits: u64 = 0x7ff8000000000001;
    let f_nan = store.intern_float(StaticFloat::from_bits(nan_bits));

    let mut export_type = dummy_type_exporter();
    let mut exporter = StaticValueTableExporter::new(&store, context);

    let u_id = exporter.export_value(u_ref, &mut export_type).unwrap();
    let bt_id = exporter.export_value(b_true_ref, &mut export_type).unwrap();
    let bf_id = exporter.export_value(b_false_ref, &mut export_type).unwrap();
    let c_id = exporter.export_value(c_ref, &mut export_type).unwrap();
    let cu_id = exporter.export_value(c_unicorn, &mut export_type).unwrap();
    let fpz_id = exporter.export_value(f_pos_zero, &mut export_type).unwrap();
    let fnz_id = exporter.export_value(f_neg_zero, &mut export_type).unwrap();
    let fnan_id = exporter.export_value(f_nan, &mut export_type).unwrap();

    let entries = exporter.finish();

    assert_eq!(entries[u_id.0 as usize].node, StaticValueNode::Unit);
    assert_eq!(entries[bt_id.0 as usize].node, StaticValueNode::Bool(true));
    assert_eq!(entries[bf_id.0 as usize].node, StaticValueNode::Bool(false));
    assert_eq!(entries[c_id.0 as usize].node, StaticValueNode::Char('x'));
    assert_eq!(entries[cu_id.0 as usize].node, StaticValueNode::Char('\u{1F984}'));

    assert_eq!(entries[fpz_id.0 as usize].node, StaticValueNode::FloatBits(0.0f64.to_bits()));
    assert_eq!(entries[fnz_id.0 as usize].node, StaticValueNode::FloatBits((-0.0f64).to_bits()));
    assert_ne!(
        entries[fpz_id.0 as usize].structural_fingerprint,
        entries[fnz_id.0 as usize].structural_fingerprint,
        "+0.0 and -0.0 must have distinct structural fingerprints"
    );

    assert_eq!(entries[fnan_id.0 as usize].node, StaticValueNode::FloatBits(nan_bits));
}

#[test]
fn static_value_metadata_large_and_negative_integers() {
    let universe = test_universe();
    let context = StableIdentityContext::new(&universe);
    let mut store = StaticValueStore::new();

    let small = store.intern_int(42);
    let big_pos = StaticInt::new(BigInt::parse_bytes(b"999999999999999999999999999999999999999", 10).unwrap());
    let big_neg = StaticInt::new(BigInt::parse_bytes(b"-999999999999999999999999999999999999999", 10).unwrap());

    let i_big_pos = store.intern_int(big_pos.to_bigint());
    let i_big_neg = store.intern_int(big_neg.to_bigint());

    let mut export_type = dummy_type_exporter();
    let mut exporter = StaticValueTableExporter::new(&store, context);

    let sm_id = exporter.export_value(small, &mut export_type).unwrap();
    let bp_id = exporter.export_value(i_big_pos, &mut export_type).unwrap();
    let bn_id = exporter.export_value(i_big_neg, &mut export_type).unwrap();

    let entries = exporter.finish();
    assert_eq!(entries[sm_id.0 as usize].node, StaticValueNode::Int(StaticInt::Small(42)));
    assert_eq!(entries[bp_id.0 as usize].node, StaticValueNode::Int(big_pos));
    assert_eq!(entries[bn_id.0 as usize].node, StaticValueNode::Int(big_neg));
    assert_ne!(
        entries[bp_id.0 as usize].structural_fingerprint,
        entries[bn_id.0 as usize].structural_fingerprint
    );
}

#[test]
fn static_value_metadata_string_and_symbol_distinction() {
    let universe = test_universe();
    let context = StableIdentityContext::new(&universe);
    let mut store = StaticValueStore::new();

    let s_ref = store.intern_string("tag");
    let sym_ref = store.intern_symbol("tag");

    let mut export_type = dummy_type_exporter();
    let mut exporter = StaticValueTableExporter::new(&store, context);

    let s_id = exporter.export_value(s_ref, &mut export_type).unwrap();
    let sym_id = exporter.export_value(sym_ref, &mut export_type).unwrap();

    let entries = exporter.finish();
    assert_eq!(entries[s_id.0 as usize].node, StaticValueNode::String("tag".into()));
    assert_eq!(entries[sym_id.0 as usize].node, StaticValueNode::Symbol("tag".into()));
    assert_ne!(
        entries[s_id.0 as usize].structural_fingerprint,
        entries[sym_id.0 as usize].structural_fingerprint,
        "String and Symbol with same spelling must have distinct fingerprints"
    );
}

#[test]
fn static_value_metadata_tuples_and_sequences() {
    let universe = test_universe();
    let context = StableIdentityContext::new(&universe);
    let mut store = StaticValueStore::new();
    let mut types = TypeStore::new();

    let int_type = types.nominal_type(universe_declaration(phalcom_native_meta::UniverseKey::Int));
    let array_int = array_type(&mut types, int_type);
    let vector_two_int = vector_type(&mut types, int_type, BigInt::from(2));

    let one = store.intern_int(1);
    let two = store.intern_int(2);
    let tuple_ref = store.intern_tuple(vec![one, two]).unwrap();

    let array_ref = store.intern_sequence(array_int, vec![one, two], &types).unwrap();
    let vector_ref = store.intern_sequence(vector_two_int, vec![one, two], &types).unwrap();

    let mut export_type = dummy_type_exporter();
    let mut exporter = StaticValueTableExporter::new(&store, context);

    let tup_id = exporter.export_value(tuple_ref, &mut export_type).unwrap();
    let arr_id = exporter.export_value(array_ref, &mut export_type).unwrap();
    let vec_id = exporter.export_value(vector_ref, &mut export_type).unwrap();

    let entries = exporter.finish();
    assert!(matches!(entries[tup_id.0 as usize].node, StaticValueNode::Tuple(ref elems) if elems.len() == 2));
    assert!(matches!(
        entries[arr_id.0 as usize].node,
        StaticValueNode::Sequence { family: StaticSequenceFamilyRef::Array, ref elements, .. } if elements.len() == 2
    ));
    assert!(matches!(
        entries[vec_id.0 as usize].node,
        StaticValueNode::Sequence { family: StaticSequenceFamilyRef::Vector, ref elements, .. } if elements.len() == 2
    ));
    assert_ne!(
        entries[arr_id.0 as usize].structural_fingerprint,
        entries[vec_id.0 as usize].structural_fingerprint,
        "Array and Vector with identical elements must have distinct fingerprints"
    );
}

#[test]
fn static_value_metadata_record_all_four_key_kinds() {
    let universe = test_universe();
    let context = StableIdentityContext::new(&universe);
    let mut store = StaticValueStore::new();

    let v1 = store.intern_int(10);
    let v2 = store.intern_int(20);
    let v3 = store.intern_int(30);
    let v4 = store.intern_int(40);

    let rec_ref = store.intern_record(vec![
        (FieldKey::symbol("sym"), v1),
        (FieldKey::string("str"), v2),
        (FieldKey::int(StaticInt::Small(42)), v3),
        (FieldKey::bool(false), v4),
    ]).unwrap();

    let mut export_type = dummy_type_exporter();
    let mut exporter = StaticValueTableExporter::new(&store, context);

    let rec_id = exporter.export_value(rec_ref, &mut export_type).unwrap();
    let entries = exporter.finish();

    let StaticValueNode::Record(ref fields) = entries[rec_id.0 as usize].node else {
        panic!("expected Record node");
    };
    assert_eq!(fields.len(), 4);
    assert_eq!(fields[0].0, FieldKeyRef::Symbol("sym".into()));
    assert_eq!(fields[1].0, FieldKeyRef::String("str".into()));
    assert_eq!(fields[2].0, FieldKeyRef::Int(StaticInt::Small(42)));
    assert_eq!(fields[3].0, FieldKeyRef::Bool(false));
}

#[test]
fn static_value_metadata_pack_root_and_slice() {
    let universe = test_universe();
    let context = StableIdentityContext::new(&universe);
    let mut store = StaticValueStore::new();

    let one = store.intern_int(1);
    let two = store.intern_int(2);
    let three = store.intern_int(3);
    let sym_a = store.intern_symbol_id("a");
    let sym_b = store.intern_symbol_id("b");

    let pack_root = store.intern_pack(vec![one, two, three], vec![(sym_a, one), (sym_b, two)]).unwrap();
    // Slice: skip first positional (take 2, 3) and skip first labeled (take b: 2):
    let pack_slice = store.intern_pack_slice(pack_root, 1, 2, 1, 1).unwrap();

    let mut export_type = dummy_type_exporter();
    let mut exporter = StaticValueTableExporter::new(&store, context);

    let root_id = exporter.export_value(pack_root, &mut export_type).unwrap();
    let slice_id = exporter.export_value(pack_slice, &mut export_type).unwrap();
    let entries = exporter.finish();

    let StaticValueNode::Pack { positionals: ref r_pos, labeled: ref r_lab } = entries[root_id.0 as usize].node else {
        panic!("expected Pack node");
    };
    assert_eq!(r_pos.len(), 3);
    assert_eq!(r_lab.len(), 2);
    assert_eq!(r_lab[0].0.as_ref(), "a");
    assert_eq!(r_lab[1].0.as_ref(), "b");

    let StaticValueNode::Pack { positionals: ref s_pos, labeled: ref s_lab } = entries[slice_id.0 as usize].node else {
        panic!("expected Pack node");
    };
    assert_eq!(s_pos.len(), 2, "slice must reflect logical positional length");
    assert_eq!(s_lab.len(), 1, "slice must reflect logical labeled length");
    assert_eq!(s_lab[0].0.as_ref(), "b");
}

#[test]
fn static_value_metadata_shared_child_deduplicated() {
    let universe = test_universe();
    let context = StableIdentityContext::new(&universe);
    let mut store = StaticValueStore::new();

    let one = store.intern_int(1);
    let shared_tuple = store.intern_tuple(vec![one]).unwrap();
    let outer_a = store.intern_tuple(vec![shared_tuple]).unwrap();
    let outer_b = store.intern_tuple(vec![shared_tuple]).unwrap();

    let mut export_type = dummy_type_exporter();
    let mut exporter = StaticValueTableExporter::new(&store, context);

    let id_a = exporter.export_value(outer_a, &mut export_type).unwrap();
    let id_b = exporter.export_value(outer_b, &mut export_type).unwrap();

    let entries = exporter.finish();
    assert_eq!(id_a, id_b, "outer identical tuples must deduplicate");
    // Node count: 1 (Int 1) + 1 (shared tuple) + 1 (outer tuple) = 3 total nodes:
    assert_eq!(entries.len(), 3);
}

#[test]
fn static_value_metadata_independent_store_lineage_equivalence() {
    let universe = test_universe();
    let context = StableIdentityContext::new(&universe);

    let mut store_a = StaticValueStore::new();
    let mut store_b = StaticValueStore::new();

    // Allocate unrelated clutter in store_b first to offset internal indices:
    let _ = store_b.intern_string("clutter");
    let _ = store_b.intern_int(999);
    let _ = store_b.intern_bool(false);

    // Build identical structural packs in both stores:
    let val_a = {
        let i1 = store_a.intern_int(10);
        let i2 = store_a.intern_int(20);
        let sym = store_a.intern_symbol_id("foo");
        store_a.intern_pack(vec![i1, i2], vec![(sym, i1)]).unwrap()
    };
    let val_b = {
        let i1 = store_b.intern_int(10);
        let i2 = store_b.intern_int(20);
        let sym = store_b.intern_symbol_id("foo");
        store_b.intern_pack(vec![i1, i2], vec![(sym, i1)]).unwrap()
    };

    let mut export_type_a = dummy_type_exporter();
    let mut exporter_a = StaticValueTableExporter::new(&store_a, context);
    let id_a = exporter_a.export_value(val_a, &mut export_type_a).unwrap();
    let entries_a = exporter_a.finish();

    let mut export_type_b = dummy_type_exporter();
    let mut exporter_b = StaticValueTableExporter::new(&store_b, context);
    let id_b = exporter_b.export_value(val_b, &mut export_type_b).unwrap();
    let entries_b = exporter_b.finish();

    assert_eq!(
        entries_a[id_a.0 as usize].structural_fingerprint,
        entries_b[id_b.0 as usize].structural_fingerprint,
        "identical structural values across different store lineages must yield identical durable fingerprints"
    );
}

#[test]
fn static_value_metadata_data_and_variant() {
    use phalcom_common::selector::Selector;
    use phalcom_common::Visibility;
    use phalcom_modules::identity::{ModuleId, ProjectIdentity};
    use phalcom_semantic::data_semantics::{
        DataComponentSemantic, DataConstructorParameter, DataConstructorSignature, DataInfo, DataProductContribution,
        DataProductShapeTemplate, DataShape,
    };
    use phalcom_semantic::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact};
    use phalcom_semantic::enum_semantics::{
        AlgebraicCasePayloadKind, VariantInfo, VariantShape, VariantVisibility,
    };
    use phalcom_semantic::identity::{DataComponentId, DataConstructorId, DeclarationId, VariantId};
    use phalcom_semantic::types::evidence::{EvidenceOrigin, TypeKnowledge};

    let universe = test_universe();
    let context = StableIdentityContext::new(&universe);
    let mut store = StaticValueStore::new();
    let mut types = TypeStore::new();

    let module = ModuleId {
        project: ProjectIdentity::Universe,
        path: Default::default(),
    };
    let decl = DeclarationId::new(module.clone(), "Point".into());
    let point_ty = types.nominal_type(decl.clone());
    let int_ty = types.nominal_type(universe_declaration(phalcom_native_meta::UniverseKey::Int));

    let declared_int = DeclaredTypeFact::from_knowledge_with_basis(
        &TypeKnowledge::established(int_ty, EvidenceOrigin::DeclarationSemantics),
        DeclaredTypeBasis::DeclarationSemantics,
    );

    let comp_x = DataComponentSemantic {
        id: DataComponentId::new(decl.clone(), 0),
        local_name: "x".into(),
        external_label: None,
        declared_type: declared_int.clone(),
        source: None,
    };
    let comp_y = DataComponentSemantic {
        id: DataComponentId::new(decl.clone(), 1),
        local_name: "y".into(),
        external_label: None,
        declared_type: declared_int.clone(),
        source: None,
    };
    let data_info = DataInfo {
        owner: decl.clone(),
        root_form: point_ty,
        generic_signature: None,
        static_data: None,
        shape: DataShape::Tuple,
        components: Box::new([comp_x.clone(), comp_y.clone()]),
        constructor: DataConstructorSignature {
            constructor: DataConstructorId::new(decl.clone()),
            visibility: Visibility::Public,
            parameters: Box::new([
                DataConstructorParameter {
                    component: comp_x.id.clone(),
                    external_label: None,
                    local_name: "x".into(),
                    declared_type: declared_int.clone(),
                },
                DataConstructorParameter {
                    component: comp_y.id.clone(),
                    external_label: None,
                    local_name: "y".into(),
                    declared_type: declared_int,
                },
            ]),
            result_type_template: point_ty,
            source: None,
        },
        shape_template: DataProductShapeTemplate {
            shape: DataShape::Tuple,
            contributions: Box::new([
                DataProductContribution::Component(comp_x.clone()),
                DataProductContribution::Component(comp_y.clone()),
            ]),
        },
        constructor_template: None,
        source: None,
    };

    let ten = store.intern_int(10);
    let twenty = store.intern_int(20);
    let data_ref = store.intern_data(&data_info, point_ty, vec![(comp_x.id, ten), (comp_y.id, twenty)], &types).unwrap();

    // Variant:
    let enum_decl = DeclarationId::new(module, "Option".into());
    let enum_ty = types.nominal_type(enum_decl.clone());
    let variant = VariantId::new(enum_decl, Selector::getter("None").unwrap());
    let exact_none = types.exact_case_type(&variant, enum_ty).unwrap();
    let variant_type = types.intern_variant_identity(variant.clone());
    let variant_info = VariantInfo {
        id: variant.clone(),
        type_handle: variant_type,
        family: None,
        shape: VariantShape::Singleton,
        payload_kind: AlgebraicCasePayloadKind::Singleton,
        fields: Box::new([]),
        result_type_template: enum_ty,
        exact_case_template: exact_none,
        case_environment: Default::default(),
        constructor: None,
        visibility: VariantVisibility::default(),
        source: None,
    };
    let variant_ref = store.intern_variant(&variant_info, exact_none, vec![], &types).unwrap();

    let mut export_type = dummy_type_exporter();
    let mut exporter = StaticValueTableExporter::new(&store, context);

    let d_id = exporter.export_value(data_ref, &mut export_type).unwrap();
    let v_id = exporter.export_value(variant_ref, &mut export_type).unwrap();
    let entries = exporter.finish();

    assert!(matches!(entries[d_id.0 as usize].node, StaticValueNode::Data { ref components, .. } if components.len() == 2));
    assert!(matches!(entries[v_id.0 as usize].node, StaticValueNode::Variant { ref fields, .. } if fields.is_empty()));
}

#[test]
fn static_constant_metadata_export_and_import_round_trip() {
    use std::sync::Arc;
    use phalcom_modules::identity::ModuleId;
    use phalcom_type_meta::encode::{decode_metadata_json, encode_metadata_json};
    use phalcom_type_meta::header::MetadataProfile;
    use phalcom_type_meta::validate::{validate_metadata_bundle, ValidationLimits};
    use phalcom_semantic::metadata::export::MetadataExporter;
    use phalcom_semantic::metadata::SemanticMetadataImporter;
    use phalcom_semantic::static_constant::{StaticConstantId, StaticConstantTable};
    use phalcom_semantic::static_value::StaticValueData;
    use phalcom_semantic::analyze_single_module;

    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        "const Answer: Int = 42\n\
         const Greeting: String = \"hello\"\n\
         const MyPack: Pack<***: Int> = Pack(1, 2)\n",
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program)).expect("semantic test analysis should succeed");
    assert!(!analysis.snapshot.has_errors(), "semantic diagnostics: {:?}", analysis.snapshot.diagnostics);

    let ans_id = StaticConstantId::new(module.clone(), "Answer");
    let greet_id = StaticConstantId::new(module.clone(), "Greeting");
    let pack_id = StaticConstantId::new(module.clone(), "MyPack");

    let ans_prod = analysis.snapshot.static_constant(&ans_id).expect("Answer product");
    let greet_prod = analysis.snapshot.static_constant(&greet_id).expect("Greeting product");
    let pack_prod = analysis.snapshot.static_constant(&pack_id).expect("MyPack product");

    assert!(ans_prod.is_successful());
    assert!(greet_prod.is_successful());
    assert!(pack_prod.is_successful());

    let projects = ProjectUniverse::new();
    let exporter = MetadataExporter::new(
        &analysis.snapshot.store,
        Some(&analysis.snapshot.declarations),
        None,
        None,
        MetadataProfile::RuntimePublic,
    )
    .with_project_universe(&projects)
    .with_static_constant_table(
        &analysis.snapshot.static_constants,
        analysis.snapshot.static_values(),
    );

    let bundle = exporter.build_bundle(&[]).expect("build bundle with static constants");
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).expect("exported bundle should validate");
    assert!(!bundle.static_values.is_empty(), "bundle must contain static value nodes");
    assert_eq!(bundle.static_constants.len(), 3, "bundle must contain three static constants");

    let encoded = encode_metadata_json(&bundle).expect("encode bundle to json");
    let decoded = decode_metadata_json(&encoded, &ValidationLimits::default()).expect("decode bundle from json");
    validate_metadata_bundle(&decoded, &ValidationLimits::default()).expect("decoded bundle should validate");

    // Session B: Fresh store, fresh types, fresh constant table:
    let mut store_b = StaticValueStore::new();
    let mut types_b = TypeStore::new();
    let mut table_b = StaticConstantTable::new();

    let importer = SemanticMetadataImporter::new(&decoded, &mut store_b, &mut types_b, &projects);
    importer.import_all(&mut table_b).expect("import bundle should succeed");

    let ans_b = table_b.get(&ans_id).expect("imported Answer constant");
    let greet_b = table_b.get(&greet_id).expect("imported Greeting constant");
    let pack_b = table_b.get(&pack_id).expect("imported MyPack constant");

    assert!(ans_b.is_successful());
    assert!(greet_b.is_successful());
    assert!(pack_b.is_successful());

    assert!(ans_b.imported.is_some(), "imported constant must have imported origin");
    assert!(ans_b.checked.is_none(), "imported constant has no local source initializer");

    let ans_val = store_b.get(ans_b.value().unwrap()).unwrap();
    assert_eq!(ans_val, &StaticValueData::Int(StaticInt::Small(42)));

    let greet_val = store_b.get(greet_b.value().unwrap()).unwrap();
    let StaticValueData::String(str_id) = greet_val else { panic!("expected String value"); };
    assert_eq!(store_b.get_string(*str_id), Some("hello"));

    let pack_val = store_b.get(pack_b.value().unwrap()).unwrap();
    let StaticValueData::Pack(p_id) = pack_val else { panic!("expected Pack value"); };
    let pos = store_b.pack_positionals(*p_id).unwrap();
    assert_eq!(pos.len(), 2);
}
