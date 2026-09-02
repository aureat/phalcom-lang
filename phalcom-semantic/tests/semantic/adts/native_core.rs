use super::support::*;
use phalcom_common::selector::Selector;
use phalcom_semantic::CoreDeclarationIds;
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, ModuleId, VariantId};
use phalcom_semantic::types::evidence::EvidenceStatus;

static GENERATED_LIST_FORM: phalcom_native_meta::TypeExprSpec = phalcom_native_meta::TypeExprSpec::Universe(phalcom_native_meta::UniverseKey::List);
static GENERATED_PARAMETER_T: phalcom_native_meta::TypeExprSpec = phalcom_native_meta::TypeExprSpec::Parameter("T");
static GENERATED_LIST_ARGUMENTS: [phalcom_native_meta::TypeExprSpec; 1] = [GENERATED_PARAMETER_T];
static GENERATED_LIST_OF_T: phalcom_native_meta::TypeExprSpec = phalcom_native_meta::TypeExprSpec::Applied {
    origin: &GENERATED_LIST_FORM,
    arguments: &GENERATED_LIST_ARGUMENTS,
};

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
            .enum_info(&DeclarationId::new(ModuleId::universe_root(), "Some".into()))
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
    let some_id = VariantId::new(DeclarationId::new(ModuleId::universe_root(), "Option".into()), some_sel.clone());
    let some_variant = case.analysis.snapshot.enum_semantics.variant_info(&some_id).expect("Some variant");
    assert_eq!(some_variant.shape, VariantShape::Constructor);
    assert_eq!(some_variant.fields.len(), 1);

    let none_sel = Selector::getter("None").unwrap();
    let none_id = VariantId::new(DeclarationId::new(ModuleId::universe_root(), "Option".into()), none_sel.clone());
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
    let ok_id = VariantId::new(DeclarationId::new(ModuleId::universe_root(), "Result".into()), ok_sel.clone());
    let ok_variant = case.analysis.snapshot.enum_semantics.variant_info(&ok_id).expect("Ok variant");
    assert_eq!(ok_variant.shape, VariantShape::Constructor);
    assert_eq!(ok_variant.fields.len(), 1);

    let err_sel = Selector::method("Error", vec![phalcom_common::selector::SelectorSlot::Positional]).unwrap();
    let err_id = VariantId::new(DeclarationId::new(ModuleId::universe_root(), "Result".into()), err_sel.clone());
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
        let var_id = VariantId::new(DeclarationId::new(ModuleId::universe_root(), "Ordering".into()), sel);
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
    let bool_decl = DeclarationId::new(ModuleId::universe_root(), "Bool".into());
    assert!(case.analysis.snapshot.enum_semantics.enum_info(&bool_decl).is_none());
}

#[test]
fn source_and_generated_generic_return_forms_publish_same_canonical_type() {
    let fixture = crate::semantic::support::Fixture::new(
        r#"
class Probe {
  @class
  keep<T>(_ value: T) -> List<T> { mystery() }

  @class
  run(_ value: Int) {
    let result = Probe.keep(value)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let source_result = fixture.binding(run, "result").current.ty().expect("source generic result");
    let source_store = (*fixture.analysis.snapshot.store).clone();
    let declarations = (*fixture.analysis.snapshot.declarations).clone();
    let parameters = std::collections::HashMap::from([("T", fixture.ty("Int"))]);
    let mut generated_store = source_store;
    let generated_result = phalcom_semantic::types::native::resolve_native_type_form(
        &mut generated_store,
        &declarations,
        &parameters,
        &phalcom_semantic::core_surface::universe_declaration,
        &GENERATED_LIST_OF_T,
    )
    .expect("generated generic return form");

    assert_eq!(generated_result, source_result, "source and generated forms must intern the same canonical List<Int> TypeId");
    let call = fixture.expression(run, "Probe.keep(value)");
    assert!(matches!(call.status, phalcom_semantic::checker::analysis::AnalysisStatus::Ready), "{call:#?}");
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Assumed));
    fixture.assert_no_error_diagnostics();
}

#[test]
fn bootstrapped_and_native_option_constructor_application_have_same_shape() {
    let bootstrapped = analyze_adt(
        r#"
class Test {
  @class
  make() -> Option<Int> { Option::Some(1) }
}
"#,
    );
    let native = analyze_adt(
        r#"
@native
enum Option<T> {
  @variant Some(_ value: T)
  @variant None
}

class Test {
  @class
  make() -> Option<Int> { Option::Some(1) }
}
"#,
    );

    let result = |case: &super::support::AdtCase| {
        let make = CallableId::new(
            DeclarationId::new(ModuleId::universe_root(), "Test".into()),
            Selector::method("make", []).unwrap(),
            DispatchSide::Class,
        );
        let callable = case.analysis.snapshot.callable_analyses.get(&make).expect("Test.make");
        let expression = callable
            .expressions
            .values()
            .find(|expression| case.source.get(expression.range.start..expression.range.end) == Some("Option::Some(1)"))
            .expect("Option constructor expression");
        (case.analysis.snapshot.store.format_type(expression.knowledge.ty().expect("constructor result")), expression.status.clone())
    };

    let (bootstrapped_type, bootstrapped_status) = result(&bootstrapped);
    let (native_type, native_status) = result(&native);
    assert_eq!(bootstrapped_type, native_type);
    assert!(
        matches!(bootstrapped_status, phalcom_semantic::checker::analysis::AnalysisStatus::Ready),
        "bootstrapped constructor status: {bootstrapped_status:#?}; native status: {native_status:#?}"
    );
    assert!(matches!(native_status, phalcom_semantic::checker::analysis::AnalysisStatus::Ready));
    native.assert_no_diagnostics();
}
