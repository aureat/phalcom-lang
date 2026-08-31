use super::support::*;
use phalcom_common::selector::Selector;
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::{DeclarationId, ModuleId, VariantId};
use phalcom_semantic::reflection::{EnumReflection, ExactCaseTypeReflection, VariantReflection};
use phalcom_semantic::tooling::GeneratedMatchPlan;

#[test]
fn test_vertical_gadt_architecture_proof() {
    let source = r#"
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}

class Evaluator {
    eval_int(_ expr: Expr<Int>) {
        match expr {
            Expr::Int(value) => value
        }
    }
}
"#;
    let case = analyze_adt(source);
    case.assert_no_diagnostics();

    // 1. Declarations & EnumInfo
    let expr_decl = DeclarationId::new(ModuleId::core(), "Expr".into());
    let enum_info = case.analysis.snapshot.enum_semantics.enum_info(&expr_decl).expect("Expr enum info");
    assert_eq!(enum_info.variants.len(), 2);
    assert_eq!(enum_info.variant_families.len(), 2);

    // 2. Variants and GADT result types
    let int_sel = Selector::method("Int", vec![phalcom_common::selector::SelectorSlot::Positional]).unwrap();
    let int_id = VariantId::new(expr_decl.clone(), int_sel.clone());
    let int_variant = case.analysis.snapshot.enum_semantics.variant_info(&int_id).expect("Int variant");
    assert_eq!(int_variant.shape, VariantShape::Constructor);
    assert_eq!(int_variant.fields.len(), 1);

    let bool_sel = Selector::method("Bool", vec![phalcom_common::selector::SelectorSlot::Positional]).unwrap();
    let bool_id = VariantId::new(expr_decl.clone(), bool_sel.clone());
    let bool_variant = case.analysis.snapshot.enum_semantics.variant_info(&bool_id).expect("Bool variant");
    assert_eq!(bool_variant.shape, VariantShape::Constructor);

    // 3. Match analysis & GADT proof
    let callable_analysis = case.analysis.snapshot.callable_analyses.values().next().expect("Evaluator method analysis");
    assert!(!callable_analysis.match_resolutions.is_empty(), "match analysis produced formal resolution");

    // 4. Protocol-neutral reflection projection
    let enum_refl = EnumReflection::from_enum_info(enum_info, false);
    assert_eq!(enum_refl.declaration.name.as_ref(), "Expr");
    assert_eq!(enum_refl.variants.len(), 2);

    let int_refl = VariantReflection::from_variant_info(int_variant);
    assert_eq!(int_refl.shape, VariantShape::Constructor);

    let exact_int_refl = ExactCaseTypeReflection::from_exact_case(
        int_variant.exact_case_template,
        int_variant,
        int_variant.result_type_template,
        &case.analysis.snapshot.store,
    );
    assert_eq!(exact_int_refl.variant, int_id);
    assert_eq!(exact_int_refl.fields.len(), 1);

    // 5. Tooling generation plan
    let gen_plan = GeneratedMatchPlan::from_enum_info(
        enum_info,
        &[int_variant, bool_variant],
        "expr",
    );
    assert_eq!(gen_plan.arms.len(), 2);
    assert_eq!(gen_plan.arms[0].pattern_syntax.as_ref(), "Int(value)");
    assert_eq!(gen_plan.arms[1].pattern_syntax.as_ref(), "Bool(value)");
}
