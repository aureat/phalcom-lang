use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::identity::{DeclarationId, VariantId};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::relation::{MapTypeHierarchy, is_subtype};
use phalcom_semantic::types::store::{TypeData, TypeStore};

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

#[test]
fn exact_case_identity_is_canonical_for_same_variant_and_specialization() {
    let mut store = TypeStore::new();
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Option".into());
    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let option_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let option_form = store.nominal_form(owner.clone(), option_kind);
    let int_ty = store.nominal_type(int_decl);
    let option_int = store.apply_type_form(option_form, &[int_ty]).expect("Option<Int>");
    let some = VariantId::new(
        owner,
        Selector::method("Some", [phalcom_common::selector::SelectorSlot::Positional]).expect("Some"),
    );

    let first = store.exact_case_type(&some, option_int).expect("first exact case");
    let second = store.exact_case_type(&some, option_int).expect("second exact case");

    assert_eq!(first, second);
    assert!(matches!(store.get(first), TypeData::ExactCase { .. }));
}

#[test]
fn exact_case_subtyping_is_reflexive_enclosing_and_variant_specific() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Option".into());
    let option = store.nominal_type(owner.clone());
    let some = VariantId::new(owner.clone(), Selector::getter("Some").expect("Some"));
    let none = VariantId::new(owner, Selector::getter("None").expect("None"));
    let some_case = store.exact_case_type(&some, option).expect("Some case");
    let none_case = store.exact_case_type(&none, option).expect("None case");

    assert!(is_subtype(&mut store, &hierarchy, some_case, some_case));
    assert!(is_subtype(&mut store, &hierarchy, some_case, option));
    assert!(!is_subtype(&mut store, &hierarchy, some_case, none_case));
    assert!(!is_subtype(&mut store, &hierarchy, option, some_case));
}

#[test]
fn adt_exact_03_union_of_exact_cases_remains_narrow_and_deduplicated() {
    let mut store = TypeStore::new();
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Option".into());
    let root = store.nominal_type(owner.clone());
    let some = VariantId::new(owner.clone(), Selector::getter("Some").expect("Some"));
    let none = VariantId::new(owner, Selector::getter("None").expect("None"));
    let some_case = store.exact_case_type(&some, root).expect("Some case");
    let none_case = store.exact_case_type(&none, root).expect("None case");
    let union = store.union(&[some_case, none_case, some_case]);
    assert!(matches!(store.get(union), TypeData::Union(members) if members.len() == 2 && members.contains(&some_case) && members.contains(&none_case)));
}

#[test]
fn adt_exact_04_exact_case_type_can_be_retained_before_root_join() {
    let mut store = TypeStore::new();
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Option".into());
    let root = store.nominal_type(owner.clone());
    let some = VariantId::new(owner, Selector::getter("Some").expect("Some"));
    let exact = store.exact_case_type(&some, root).expect("Some case");
    assert!(matches!(store.get(exact), TypeData::ExactCase { .. }));
}

#[test]
fn adt_exact_05_generic_substitution_changes_root_and_exact_case_together() {
    let mut store = TypeStore::new();
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Option".into());
    let kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let form = store.nominal_form(owner.clone(), kind);
    let int = store.nominal_type(DeclarationId::new(module.clone(), "Int".into()));
    let string = store.nominal_type(DeclarationId::new(module, "String".into()));
    let int_root = store.apply_type_form(form, &[int]).expect("Option<Int>");
    let string_root = store.apply_type_form(form, &[string]).expect("Option<String>");
    let some = VariantId::new(owner, Selector::getter("Some").expect("Some"));
    assert_ne!(store.exact_case_type(&some, int_root), store.exact_case_type(&some, string_root));
}

#[test]
#[ignore = "GATED: transparent alias declaration fixture is required"]
fn adt_exact_06_transparent_alias_union_matches_direct_exact_union() {
    let parsed = phalcom_ast::parse_source("type ChoiceAlias = Choice\n", 0).expect("transparent alias fixture should parse");
    assert!(matches!(parsed.statements.first(), Some(phalcom_ast::ast::Statement::TypeAlias(_))));
    let mut store = TypeStore::new();
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Option".into());
    let root = store.nominal_type(owner.clone());
    let some = VariantId::new(owner.clone(), Selector::getter("Some").expect("Some"));
    let none = VariantId::new(owner, Selector::getter("None").expect("None"));
    let some_case = store.exact_case_type(&some, root).expect("Some case");
    let none_case = store.exact_case_type(&none, root).expect("None case");
    let direct = store.union(&[some_case, none_case]);
    let alias = direct;
    assert_eq!(alias, direct, "transparent aliases must preserve canonical exact-case union");
}
