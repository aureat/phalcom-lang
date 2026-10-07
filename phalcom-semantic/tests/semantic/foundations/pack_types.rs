use phalcom_modules::{DeclarationId, ModuleId};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::relation::{MapTypeHierarchy, check_subtype_bounded};
use phalcom_semantic::types::store::{AnonymousTupleRemainder, PackTypeElement, TupleTypeElement, TypeData, TypeStore};
use phalcom_semantic::types::substitution::TypeSubstitution;

fn decl(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::universe_root(), name.into())
}

#[test]
fn empty_pack_is_distinct_proper_type_and_not_unit() {
    let mut store = TypeStore::new();
    let empty_pack = store
        .open_pack(Box::new([]), AnonymousTupleRemainder::None, AnonymousTupleRemainder::None)
        .unwrap();
    let unit = store.unit();

    assert_ne!(empty_pack, unit, "CV-02: Pack() != Unit");
    assert_eq!(store.kind_of(empty_pack), KindId::TYPE, "Pack is a proper type");

    let proper_pack = store.proper_type(empty_pack).expect("empty Pack is a proper type");
    assert_eq!(proper_pack.raw(), empty_pack);
}

#[test]
fn closed_exact_pack_type_preserves_lanes_and_distinct_from_tuple() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal_type(decl("Int"));
    let str_ty = store.nominal_type(decl("String"));

    // Pack with 1 positional (Int) and 1 labeled (x: String)
    let pack_ty = store
        .pack(Box::new([
            PackTypeElement { label: None, ty: int_ty },
            PackTypeElement {
                label: Some("x".into()),
                ty: str_ty,
            },
        ]))
        .unwrap();

    assert_eq!(store.kind_of(pack_ty), KindId::TYPE);

    // Corresponding Tuple
    let tuple_ty = store.tuple(Box::new([
        TupleTypeElement { label: None, ty: int_ty },
        TupleTypeElement {
            label: Some("x".into()),
            ty: str_ty,
        },
    ]));

    assert_ne!(pack_ty, tuple_ty, "CV-02: Pack != Tuple");

    // Ordinary subtyping between Pack and Tuple must be refuted (CV-02)
    let hierarchy = MapTypeHierarchy::new();
    let mut budget1 = phalcom_semantic::db::QueryBudget::default();
    let token1 = phalcom_semantic::db::CancellationToken::new();
    let sub1 = check_subtype_bounded(&mut store, &hierarchy, pack_ty, tuple_ty, &mut budget1, &token1);
    assert!(!sub1.is_proven(), "Pack must not subtype Tuple");

    let mut budget2 = phalcom_semantic::db::QueryBudget::default();
    let token2 = phalcom_semantic::db::CancellationToken::new();
    let sub2 = check_subtype_bounded(&mut store, &hierarchy, tuple_ty, pack_ty, &mut budget2, &token2);
    assert!(!sub2.is_proven(), "Tuple must not subtype Pack");
}

#[test]
fn labeled_order_affects_pack_type() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal_type(decl("Int"));
    let str_ty = store.nominal_type(decl("String"));

    // Pack with (x: Int, y: String)
    let pack1 = store
        .pack(Box::new([
            PackTypeElement {
                label: Some("x".into()),
                ty: int_ty,
            },
            PackTypeElement {
                label: Some("y".into()),
                ty: str_ty,
            },
        ]))
        .unwrap();

    // Pack with (y: String, x: Int)
    let pack2 = store
        .pack(Box::new([
            PackTypeElement {
                label: Some("y".into()),
                ty: str_ty,
            },
            PackTypeElement {
                label: Some("x".into()),
                ty: int_ty,
            },
        ]))
        .unwrap();

    assert_ne!(pack1, pack2, "CV-03: Labeled encounter order is semantic in Pack types");
}

#[test]
fn pack_from_packs_symbolic_bridge_and_concrete_normalization() {
    let mut store = TypeStore::new();
    let owner = TypeParameterOwner::Declaration(decl("PackOwner"));
    let p_param = store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "P", KindId::TYPE_PACK));
    let l_param = store.intern_type_parameter(TypeParameterData::new(owner, 1, "L", KindId::LABELED_TYPE_PACK));

    let p_form = store.parameter_form(p_param);
    let l_form = store.parameter_form(l_param);

    // Symbolic PackFromPacks
    let sym_pack = store.pack_from_packs(p_form, l_form).unwrap();
    assert!(
        matches!(store.get(sym_pack), TypeData::PackFromPacks { .. }),
        "CV-10: symbolic lane terms remain PackFromPacks"
    );
    assert_eq!(store.kind_of(sym_pack), KindId::TYPE);

    // Concrete substitution makes lanes concrete
    let int_ty = store.nominal_type(decl("Int"));
    let str_ty = store.nominal_type(decl("String"));
    let concrete_pos = store.type_pack(Box::new([int_ty])).unwrap();
    let concrete_lab = store
        .labeled_type_pack(Box::new([phalcom_semantic::types::store::LabeledTypePackElement {
            label: "a".into(),
            element: str_ty,
        }]))
        .unwrap();

    let mut subst = TypeSubstitution::new();
    subst.bind(p_param, concrete_pos);
    subst.bind(l_param, concrete_lab);

    let subst_pack = subst.apply(&mut store, sym_pack);
    assert!(
        matches!(store.get(subst_pack), TypeData::Pack(_)),
        "CV-10: substitution normalizes PackFromPacks to canonical concrete Pack"
    );

    // Direct formation with the same elements
    let direct_pack = store
        .pack(Box::new([
            PackTypeElement { label: None, ty: int_ty },
            PackTypeElement {
                label: Some("a".into()),
                ty: str_ty,
            },
        ]))
        .unwrap();

    assert_eq!(subst_pack, direct_pack, "CV-10: normalized Pack matches direct Pack formation");
}

#[test]
fn parameter_contract_derives_canonical_body_pack_types() {
    use phalcom_ast::ast::RestMode;
    use phalcom_semantic::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact};
    use phalcom_semantic::signature::{ParameterContract, RestValueContract};
    use phalcom_semantic::types::parameter::{GenericParameterMode, TypeParameterOwner, TypeTerm};

    let mut store = TypeStore::new();
    let int_ty = store.nominal_type(decl("Int"));
    let elem_fact = DeclaredTypeFact::known(TypeTerm::Canonical(int_ty), DeclaredTypeBasis::DeclarationSemantics);

    // *args: Int derives Pack with Typed(Int) positional remainder
    let element_contract = ParameterContract::Rest {
        mode: RestMode::Positional,
        contract: RestValueContract::Element(elem_fact),
    };
    let body_ty = element_contract.derive_body_type(&mut store);
    let TypeData::Pack(pack) = store.get(body_ty.canonical_type().unwrap()) else {
        panic!("expected Pack");
    };
    assert_eq!(pack.positional_remainder, AnonymousTupleRemainder::Typed(int_ty));
    assert_eq!(pack.labeled_remainder, AnonymousTupleRemainder::None);

    // *args: *P derives PackFromPacks(P, EmptyLabeled)
    let owner = TypeParameterOwner::Declaration(decl("Test"));
    let p_id = store.intern_type_parameter(TypeParameterData::new(owner, 0, "P", KindId::TYPE_PACK).with_mode(GenericParameterMode::PositionalPack));
    let p_form = store.parameter_form(p_id);
    let lane_contract = ParameterContract::Rest {
        mode: RestMode::Positional,
        contract: RestValueContract::Lane(p_form),
    };
    let lane_body_ty = lane_contract.derive_body_type(&mut store);
    let TypeData::PackFromPacks { positionals, labeled } = store.get(lane_body_ty.canonical_type().unwrap()) else {
        panic!("expected PackFromPacks");
    };
    assert_eq!(*positionals, p_form);
    let TypeData::LabeledTypePack(empty_labeled) = store.get(*labeled) else {
        panic!("expected empty labeled");
    };
    assert!(empty_labeled.is_empty());

    // *args (unconstrained) derives Pack with Unconstrained positional remainder
    let unconstrained_contract = ParameterContract::Rest {
        mode: RestMode::Positional,
        contract: RestValueContract::Unconstrained,
    };
    let unconstrained_body_ty = unconstrained_contract.derive_body_type(&mut store);
    let TypeData::Pack(u_pack) = store.get(unconstrained_body_ty.canonical_type().unwrap()) else {
        panic!("expected Pack");
    };
    assert_eq!(u_pack.positional_remainder, AnonymousTupleRemainder::Unconstrained);
}
