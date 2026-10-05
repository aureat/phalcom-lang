use phalcom_ast::parser::parse_source;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::metadata::export::MetadataExporter;
use phalcom_semantic::types::relation::{MapTypeHierarchy, is_subtype};
use phalcom_semantic::types::row::{AnonymousRecordRemainder, RecordRowField, RecordRowTail};
use phalcom_semantic::types::store::{AnonymousTupleRemainder, TupleTypeElement, TypeData, TypeStore};
use phalcom_type_meta::encode::{decode_metadata_json, encode_metadata_json};
use phalcom_type_meta::header::MetadataProfile;
use phalcom_type_meta::type_node::{AnonymousTupleRemainderRef, StructuralMemberPresenceRef, TypeNode};
use phalcom_type_meta::validate::{ValidationLimits, validate_metadata_bundle};

fn decl(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::universe_root(), name.into())
}

// ----------------------------------------------------------------------------
// R1: Tuple presence removal & invariants
// ----------------------------------------------------------------------------

#[test]
fn tuple_optional_labeled_member_is_rejected() {
    let result = parse_source("type T = (name?: String)\n", 0);
    assert!(result.is_err(), "Optional labeled tuple member (name?: String) must be rejected");
}

#[test]
fn tuple_optional_positional_member_is_rejected() {
    let result = parse_source("type T = (String?)\n", 0);
    assert!(result.is_err(), "Optional positional tuple member (String?) must be rejected");

    let result2 = parse_source("type T = (Int, Float?)\n", 0);
    assert!(result2.is_err(), "Optional positional tuple member (Int, Float?) must be rejected");
}

#[test]
fn tuple_required_labeled_member_remains_valid() {
    let result = parse_source("type T = (name: String)\n", 0);
    assert!(result.is_ok(), "Required labeled tuple member (name: String) must remain valid");
}

#[test]
fn tuple_open_positional_remainder_remains_valid() {
    let result = parse_source("type T = (Int, *: Number)\n", 0);
    assert!(result.is_ok(), "Positional remainder (Int, *: Number) must remain valid");
}

#[test]
fn tuple_open_labeled_remainder_remains_valid() {
    let result = parse_source("type T = (name: String, **: JsonValue)\n", 0);
    assert!(result.is_ok(), "Labeled remainder (name: String, **: JsonValue) must remain valid");
}

#[test]
fn tuple_expansion_and_remainder_roundtrip_without_presence() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal(decl("Int"));
    let str_ty = store.nominal(decl("String"));
    let num_ty = store.nominal(decl("Number"));

    let open_tuple = store.open_tuple(
        Box::new([TupleTypeElement::unlabeled(int_ty), TupleTypeElement::labeled("name".into(), str_ty)]),
        AnonymousTupleRemainder::Typed(num_ty),
        AnonymousTupleRemainder::Unconstrained,
    );

    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let node_id = exporter.export_type_form(open_tuple).expect("export open tuple");
    let bundle = exporter.build_bundle(&[]).expect("build bundle");
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).expect("validate bundle");

    let json = encode_metadata_json(&bundle).expect("encode json");
    let decoded = decode_metadata_json(&json, &ValidationLimits::default()).expect("decode json");

    let TypeNode::OpenTuple(decoded_open) = &decoded.types[node_id.0 as usize].form else {
        panic!("expected OpenTuple");
    };
    assert_eq!(decoded_open.elements.len(), 2);
    assert_eq!(decoded_open.elements[0].label, None);
    assert_eq!(decoded_open.elements[0].presence, StructuralMemberPresenceRef::Required);
    assert_eq!(decoded_open.elements[1].label.as_deref(), Some("name"));
    assert_eq!(decoded_open.elements[1].presence, StructuralMemberPresenceRef::Required);
    assert!(matches!(decoded_open.positional_remainder, AnonymousTupleRemainderRef::Typed(_)));
    assert_eq!(decoded_open.labeled_remainder, AnonymousTupleRemainderRef::Unconstrained);
}

// ----------------------------------------------------------------------------
// R2: Record presence subtyping
// ----------------------------------------------------------------------------

#[test]
fn record_required_source_satisfies_required_target() {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let int = store.nominal(decl("Int"));
    let number = store.nominal(decl("Number"));
    hierarchy.insert(decl("Int"), decl("Number"));

    let source = store.record(Box::new([RecordRowField::symbol_required("a", int)]));
    let target = store.record(Box::new([RecordRowField::symbol_required("a", number)]));

    assert!(is_subtype(&mut store, &hierarchy, source, target));
}

#[test]
fn record_required_source_satisfies_optional_target() {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let int = store.nominal(decl("Int"));
    let number = store.nominal(decl("Number"));
    hierarchy.insert(decl("Int"), decl("Number"));

    let source = store.record(Box::new([RecordRowField::symbol_required("a", int)]));
    let target = store.record(Box::new([RecordRowField::symbol_optional("a", number)]));

    assert!(is_subtype(&mut store, &hierarchy, source, target));
}

#[test]
fn record_optional_source_satisfies_optional_target() {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let int = store.nominal(decl("Int"));
    let number = store.nominal(decl("Number"));
    hierarchy.insert(decl("Int"), decl("Number"));

    let source = store.record(Box::new([RecordRowField::symbol_optional("a", int)]));
    let target = store.record(Box::new([RecordRowField::symbol_optional("a", number)]));

    assert!(is_subtype(&mut store, &hierarchy, source, target));
}

#[test]
fn record_optional_source_does_not_satisfy_required_target() {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let int = store.nominal(decl("Int"));
    let number = store.nominal(decl("Number"));
    hierarchy.insert(decl("Int"), decl("Number"));

    // { a?: Int } </: { a: Number } even though Int <: Number
    let source = store.record(Box::new([RecordRowField::symbol_optional("a", int)]));
    let target = store.record(Box::new([RecordRowField::symbol_required("a", number)]));

    assert!(!is_subtype(&mut store, &hierarchy, source, target));
}

#[test]
fn record_closed_absence_satisfies_optional_target() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let number = store.nominal(decl("Number"));

    // { } <: { a?: Number }
    let source = store.record(Box::new([]));
    let target = store.record(Box::new([RecordRowField::symbol_optional("a", number)]));

    assert!(is_subtype(&mut store, &hierarchy, source, target));
}

#[test]
fn record_closed_absence_does_not_satisfy_required_target() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let number = store.nominal(decl("Number"));

    // { } </: { a: Number }
    let source = store.record(Box::new([]));
    let target = store.record(Box::new([RecordRowField::symbol_required("a", number)]));

    assert!(!is_subtype(&mut store, &hierarchy, source, target));
}

#[test]
fn record_typed_remainder_satisfies_compatible_optional_target() {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let int = store.nominal(decl("Int"));
    let number = store.nominal(decl("Number"));
    let field_key_ty = store.field_key_type_from_core();
    hierarchy.insert(decl("Int"), decl("Number"));

    // { **: Int } <: { a?: Number, ... }
    let source_row = store
        .record_row_checked(vec![], RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed { key_type: field_key_ty, value_type: int }))
        .unwrap();
    let source = store.intern_with_kind(TypeData::Record(source_row), phalcom_semantic::types::id::KindId::TYPE);

    let target_row = store
        .record_row_checked(
            vec![RecordRowField::symbol_optional("a", number)],
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: field_key_ty }),
        )
        .unwrap();
    let target = store.intern_with_kind(TypeData::Record(target_row), phalcom_semantic::types::id::KindId::TYPE);

    assert!(is_subtype(&mut store, &hierarchy, source, target));
}

#[test]
fn record_typed_remainder_does_not_prove_required_presence() {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let int = store.nominal(decl("Int"));
    let number = store.nominal(decl("Number"));
    let field_key_ty = store.field_key_type_from_core();
    hierarchy.insert(decl("Int"), decl("Number"));

    // { **: Int } </: { a: Number, ... } because remainder does not prove field presence
    let source_row = store
        .record_row_checked(vec![], RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed { key_type: field_key_ty, value_type: int }))
        .unwrap();
    let source = store.intern_with_kind(TypeData::Record(source_row), phalcom_semantic::types::id::KindId::TYPE);

    let target_row = store
        .record_row_checked(
            vec![RecordRowField::symbol_required("a", number)],
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: field_key_ty }),
        )
        .unwrap();
    let target = store.intern_with_kind(TypeData::Record(target_row), phalcom_semantic::types::id::KindId::TYPE);

    assert!(!is_subtype(&mut store, &hierarchy, source, target));
}

#[test]
fn record_unconstrained_remainder_does_not_satisfy_typed_optional_target() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let number = store.nominal(decl("Number"));
    let field_key_ty = store.field_key_type_from_core();

    // { ... } </: { a?: Number, ... } because an unconstrained tail cannot prove value type compatibility
    let source_row = store
        .record_row_checked(vec![], RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: field_key_ty }))
        .unwrap();
    let source = store.intern_with_kind(TypeData::Record(source_row), phalcom_semantic::types::id::KindId::TYPE);

    let target_row = store
        .record_row_checked(
            vec![RecordRowField::symbol_optional("a", number)],
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: field_key_ty }),
        )
        .unwrap();
    let target = store.intern_with_kind(TypeData::Record(target_row), phalcom_semantic::types::id::KindId::TYPE);

    assert!(!is_subtype(&mut store, &hierarchy, source, target));
}

#[test]
fn record_presence_and_value_subtyping_are_checked_independently() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let int = store.nominal(decl("Int"));
    let string = store.nominal(decl("String"));

    // Case 1: presence matches (both optional), value types incompatible (Int </: String) => Refuted
    let opt_int = store.record(Box::new([RecordRowField::symbol_optional("a", int)]));
    let opt_str = store.record(Box::new([RecordRowField::symbol_optional("a", string)]));
    assert!(!is_subtype(&mut store, &hierarchy, opt_int, opt_str));

    // Case 2: value types match (Int <: Int), presence incompatible (Optional </: Required) => Refuted
    let req_int = store.record(Box::new([RecordRowField::symbol_required("a", int)]));
    assert!(!is_subtype(&mut store, &hierarchy, opt_int, req_int));

    // Case 3: value types match (Int <: Int), presence compatible (Required <: Optional) => Proven
    assert!(is_subtype(&mut store, &hierarchy, req_int, opt_int));

    // Case 4: value types match (Int <: Int), presence compatible (Required <: Required) => Proven
    assert!(is_subtype(&mut store, &hierarchy, req_int, req_int));
}
