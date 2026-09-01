use super::support::*;
use phalcom_common::selector::Selector;
use phalcom_semantic::CoreDeclarationIds;
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::{DeclarationId, ModuleId, VariantId};

#[test]
fn bootstrapped_core_option_is_canonical_enum() {
    let option = CoreDeclarationIds::default().option;
    let case = analyze_adt(r#"
class Test {
    check_option(_ opt: Option<Int>) {
        match opt {
            Some(value) => value
            None => 0
        }
    }
}
"#);

    let enum_info = case
        .analysis
        .snapshot
        .enum_semantics
        .enum_info(&option)
        .expect("core Option must be an enum");

    assert_eq!(enum_info.variants.len(), 2);

    let some_sel = Selector::method("Some", vec![phalcom_common::selector::SelectorSlot::Positional]).unwrap();
    let some = VariantId::new(option.clone(), some_sel);
    let none_sel = Selector::getter("None").unwrap();
    let none = VariantId::new(option.clone(), none_sel);

    assert_eq!(
        case.analysis.snapshot.enum_semantics.variant_info(&some).unwrap().shape,
        VariantShape::Constructor,
    );

    assert_eq!(
        case.analysis.snapshot.enum_semantics.variant_info(&none).unwrap().shape,
        VariantShape::Singleton,
    );

    assert!(
        case.analysis
            .snapshot
            .enum_semantics
            .enum_info(&DeclarationId::new(ModuleId::core(), "Some".into()))
            .is_none()
    );
}

#[test]
fn test_native_option_canonical_enum_semantics() {
    let source = r#"
@native
enum Option<T> {
    @variant Some(_ value: T)
    @variant None
}

class Test {
    check_option(_ opt: Option<Int>) {
        match opt {
            Some(value) => value
            None => 0
        }
    }
}
"#;
    let case = analyze_adt(source);
    case.assert_no_diagnostics();

    let enum_info = case.enum_info("Option");
    assert_eq!(enum_info.variants.len(), 2);
    assert_eq!(enum_info.variant_families.len(), 2);

    let some_sel = Selector::method("Some", vec![phalcom_common::selector::SelectorSlot::Positional]).unwrap();
    let some_id = VariantId::new(DeclarationId::new(ModuleId::core(), "Option".into()), some_sel.clone());
    let some_variant = case.analysis.snapshot.enum_semantics.variant_info(&some_id).expect("Some variant");
    assert_eq!(some_variant.shape, VariantShape::Constructor);
    assert_eq!(some_variant.fields.len(), 1);

    let none_sel = Selector::getter("None").unwrap();
    let none_id = VariantId::new(DeclarationId::new(ModuleId::core(), "Option".into()), none_sel.clone());
    let none_variant = case.analysis.snapshot.enum_semantics.variant_info(&none_id).expect("None variant");
    assert_eq!(none_variant.shape, VariantShape::Singleton);
    assert_eq!(none_variant.fields.len(), 0);
}

#[test]
fn test_native_result_canonical_enum_semantics() {
    let source = r#"
@native
enum Result<T, E> {
    @variant Ok(_ value: T)
    @variant Error(_ error: E)
}

class Test {
    check_result(_ res: Result<Int, String>) {
        match res {
            Ok(v) => v
            Error(_) => 0
        }
    }
}
"#;
    let case = analyze_adt(source);
    case.assert_no_diagnostics();

    let enum_info = case.enum_info("Result");
    assert_eq!(enum_info.variants.len(), 2);
    assert_eq!(enum_info.variant_families.len(), 2);

    let ok_sel = Selector::method("Ok", vec![phalcom_common::selector::SelectorSlot::Positional]).unwrap();
    let ok_id = VariantId::new(DeclarationId::new(ModuleId::core(), "Result".into()), ok_sel.clone());
    let ok_variant = case.analysis.snapshot.enum_semantics.variant_info(&ok_id).expect("Ok variant");
    assert_eq!(ok_variant.shape, VariantShape::Constructor);
    assert_eq!(ok_variant.fields.len(), 1);

    let err_sel = Selector::method("Error", vec![phalcom_common::selector::SelectorSlot::Positional]).unwrap();
    let err_id = VariantId::new(DeclarationId::new(ModuleId::core(), "Result".into()), err_sel.clone());
    let err_variant = case.analysis.snapshot.enum_semantics.variant_info(&err_id).expect("Error variant");
    assert_eq!(err_variant.shape, VariantShape::Constructor);
    assert_eq!(err_variant.fields.len(), 1);
}

#[test]
fn test_native_ordering_canonical_enum_semantics() {
    let source = r#"
@native
enum Ordering {
    @variant Less
    @variant Equal
    @variant Greater
    @variant Unordered
}

class Test {
    check_ordering(_ ord: Ordering) {
        match ord {
            Less => 0
            Equal => 1
            Greater => 2
            Unordered => 3
        }
    }
}
"#;
    let case = analyze_adt(source);
    case.assert_no_diagnostics();

    let enum_info = case.enum_info("Ordering");
    assert_eq!(enum_info.variants.len(), 4);
    assert_eq!(enum_info.variant_families.len(), 4);

    for name in &["Less", "Equal", "Greater", "Unordered"] {
        let sel = Selector::getter(*name).unwrap();
        let var_id = VariantId::new(DeclarationId::new(ModuleId::core(), "Ordering".into()), sel);
        let var_info = case.analysis.snapshot.enum_semantics.variant_info(&var_id).expect("ordering variant");
        assert_eq!(var_info.shape, VariantShape::Singleton);
    }
}

#[test]
fn test_bool_remains_primitive_not_enum() {
    let source = r#"
class Test {
    check_bool(_ b: Bool) {
        if b { 1 } else { 0 }
    }
}
"#;
    let case = analyze_adt(source);
    case.assert_no_diagnostics();

    // Verify Bool is nominal type, not registered in enum_semantics
    let bool_decl = DeclarationId::new(ModuleId::core(), "Bool".into());
    assert!(case.analysis.snapshot.enum_semantics.enum_info(&bool_decl).is_none());
}
