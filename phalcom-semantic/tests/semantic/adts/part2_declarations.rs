use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use phalcom_common::selector::{Selector, SelectorBase, SelectorSlot};
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_native_meta::{EffectSpec, ImplementationKind, NativeLifecycleSpec, RaisesSpec, ReturnFlowSpec};
use phalcom_semantic::associated::build_associated_surface;
use phalcom_semantic::checker::AssociatedResolutionKind;
use phalcom_semantic::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::enum_requirements::{
    CaseRequirementStatus, EnumRequirement, EnumRequirementId, check_enum_requirements,
};
use phalcom_semantic::enum_semantics::{EnumInfo, VariantInfo, VariantShape, VariantVisibility};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, VariantId};
use phalcom_semantic::signature::{CallableSemanticSignature, ReturnContractValidation};
use phalcom_semantic::types::case_environment::{CaseEnvironmentError, derive_case_environment};
use phalcom_semantic::types::id::{KindId, VariantTypeId};
use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner, TypeTerm};
use phalcom_semantic::types::relation::{MapTypeHierarchy, is_subtype};
use phalcom_semantic::types::store::TypeStore;
use phalcom_semantic::analyze_single_module;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

fn helper_signature(owner: DeclarationId, selector: Selector, return_type: DeclaredTypeFact) -> CallableSemanticSignature {
    let callable = CallableId::new(owner.clone(), selector.clone(), DispatchSide::Instance);
    CallableSemanticSignature {
        callable,
        owner,
        side: DispatchSide::Instance,
        selector,
        generics: None,
        parameters: Box::new([]),
        declared_return: return_type,
        return_validation: ReturnContractValidation::Unchecked,
        inferred_return: None,
        source: None,
        implementation: ImplementationKind::Source,
        native_id: None,
        effects: EffectSpec::Unknown,
        raises: RaisesSpec::Unknown,
        flow: ReturnFlowSpec::Value,
        lifecycle: NativeLifecycleSpec::UNKNOWN,
    }
}

#[test]
fn test_exact_case_nominal_subtyping() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let module = test_module();

    let option_decl = DeclarationId::new(module.clone(), "Option".into());
    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let str_decl = DeclarationId::new(module.clone(), "String".into());

    let int_ty = store.nominal_type(int_decl);
    let str_ty = store.nominal_type(str_decl);

    let option_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let option_root = store.nominal_form(option_decl.clone(), option_kind);
    let option_int_ty = store.apply_type_form(option_root, &[int_ty]).expect("apply option int");
    let option_str_ty = store.apply_type_form(option_root, &[str_ty]).expect("apply option str");

    let some_selector = Selector::method("Some", vec![SelectorSlot::Positional]).unwrap();
    let none_selector = Selector::getter("None").unwrap();

    let some_variant = VariantId::new(option_decl.clone(), some_selector);
    let none_variant = VariantId::new(option_decl.clone(), none_selector);

    let some_int_case = store.exact_case_type(&some_variant, option_int_ty).expect("some int case");
    let none_int_case = store.exact_case_type(&none_variant, option_int_ty).expect("none int case");
    let some_str_case = store.exact_case_type(&some_variant, option_str_ty).expect("some str case");

    // 1. Exact case is a subtype of its enclosing applied enum type
    assert!(is_subtype(&mut store, &hier, some_int_case, option_int_ty));
    assert!(is_subtype(&mut store, &hier, none_int_case, option_int_ty));

    // 2. Exact case is reflexively a subtype of itself
    assert!(is_subtype(&mut store, &hier, some_int_case, some_int_case));
    assert!(is_subtype(&mut store, &hier, none_int_case, none_int_case));

    // 3. Different variants are not subtypes of each other
    assert!(!is_subtype(&mut store, &hier, some_int_case, none_int_case));
    assert!(!is_subtype(&mut store, &hier, none_int_case, some_int_case));

    // 4. Exact case with different type argument is not a subtype of mismatched applied enum
    assert!(!is_subtype(&mut store, &hier, some_int_case, option_str_ty));
    assert!(!is_subtype(&mut store, &hier, some_int_case, some_str_case));
}

#[test]
fn test_associated_family_namespace_collisions() {
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Status".into());

    let value_sel = Selector::getter("value").unwrap();
    let variant = VariantId::new(owner.clone(), value_sel.clone());

    // Conflict with class-side callable of same base
    let class_callable = CallableId::new(owner.clone(), value_sel.clone(), DispatchSide::Class);
    let (surface, diags) = build_associated_surface(
        &owner,
        Some(&[variant.clone()]),
        &[class_callable],
        &HashSet::new(),
        &module,
        None,
    );
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::EnumFamilyCategoryConflict),
        "expected EnumFamilyCategoryConflict, got {diags:#?}"
    );
    assert_eq!(surface.families.len(), 1);

    // Conflict with inherited class behavior
    let mut inherited_bases = HashSet::new();
    inherited_bases.insert(SelectorBase::Named("value".into()));
    let (_surface, inherited_diags) = build_associated_surface(
        &owner,
        Some(&[variant]),
        &[],
        &inherited_bases,
        &module,
        None,
    );
    assert!(
        inherited_diags.iter().any(|d| d.code == DiagnosticCode::EnumFamilyInheritedBehaviorConflict),
        "expected EnumFamilyInheritedBehaviorConflict, got {inherited_diags:#?}"
    );
}

#[test]
fn test_closed_enum_requirements_validation() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Shape".into());

    let str_ty = store.nominal_type(DeclarationId::new(module.clone(), "String".into()));
    let int_ty = store.nominal_type(DeclarationId::new(module.clone(), "Int".into()));

    let describe_sel = Selector::getter("describe").unwrap();
    let req_id = EnumRequirementId::new(owner.clone(), describe_sel.clone());
    let root_req = EnumRequirement {
        id: req_id.clone(),
        signature: helper_signature(
            owner.clone(),
            describe_sel.clone(),
            DeclaredTypeFact::known(TypeTerm::Canonical(str_ty), DeclaredTypeBasis::SourceAnnotation),
        ),
        source: None,
    };

    let circle_sel = Selector::getter("Circle").unwrap();
    let square_sel = Selector::getter("Square").unwrap();
    let bad_sel = Selector::getter("Bad").unwrap();

    let circle_var = VariantId::new(owner.clone(), circle_sel.clone());
    let square_var = VariantId::new(owner.clone(), square_sel.clone());
    let bad_var = VariantId::new(owner.clone(), bad_sel.clone());

    let nominal_owner = store.nominal_type(owner.clone());
    let circle_exact = store.exact_case_type(&circle_var, nominal_owner).expect("circle exact");
    let square_exact = store.exact_case_type(&square_var, nominal_owner).expect("square exact");
    let bad_exact = store.exact_case_type(&bad_var, nominal_owner).expect("bad exact");

    let variants = vec![
        VariantInfo {
            id: circle_var.clone(),
            type_handle: VariantTypeId(1),
            family: circle_var.family(),
            shape: VariantShape::Singleton,
            fields: Box::new([]),
            result_type_template: nominal_owner,
            exact_case_template: circle_exact,
            case_environment: phalcom_semantic::types::case_environment::CaseTypeEnvironment::default(),
            constructor: None,
            visibility: VariantVisibility::default(),
            source: None,
        },
        VariantInfo {
            id: square_var.clone(),
            type_handle: VariantTypeId(2),
            family: square_var.family(),
            shape: VariantShape::Singleton,
            fields: Box::new([]),
            result_type_template: nominal_owner,
            exact_case_template: square_exact,
            case_environment: phalcom_semantic::types::case_environment::CaseTypeEnvironment::default(),
            constructor: None,
            visibility: VariantVisibility::default(),
            source: None,
        },
        VariantInfo {
            id: bad_var.clone(),
            type_handle: VariantTypeId(3),
            family: bad_var.family(),
            shape: VariantShape::Singleton,
            fields: Box::new([]),
            result_type_template: nominal_owner,
            exact_case_template: bad_exact,
            case_environment: phalcom_semantic::types::case_environment::CaseTypeEnvironment::default(),
            constructor: None,
            visibility: VariantVisibility::default(),
            source: None,
        },
    ];

    let mut case_methods = HashMap::new();
    // Circle satisfies requirement with matching return type String
    case_methods.insert(
        circle_var.clone(),
        vec![helper_signature(
            owner.clone(),
            describe_sel.clone(),
            DeclaredTypeFact::known(TypeTerm::Canonical(str_ty), DeclaredTypeBasis::SourceAnnotation),
        )],
    );
    // Bad variant has incompatible return type Int instead of String
    case_methods.insert(
        bad_var.clone(),
        vec![helper_signature(
            owner.clone(),
            describe_sel,
            DeclaredTypeFact::known(TypeTerm::Canonical(int_ty), DeclaredTypeBasis::SourceAnnotation),
        )],
    );
    // Square has no implementation (missing)

    let enum_info = EnumInfo {
        owner: owner.clone(),
        root_form: nominal_owner,
        generic_signature: None,
        default_result_type: nominal_owner,
        variants: variants.iter().map(|v| v.id.clone()).collect(),
        variant_families: Box::new([]),
        source: None,
    };

    let (statuses, diags) = check_enum_requirements(
        &owner,
        &enum_info,
        &variants,
        &[root_req],
        &case_methods,
        &mut store,
        &hier,
        &module,
    );

    let circle_status = statuses.iter().find(|s| s.variant == circle_var).expect("circle status");
    assert!(matches!(circle_status.status, CaseRequirementStatus::Satisfied { .. }));

    let square_status = statuses.iter().find(|s| s.variant == square_var).expect("square status");
    assert!(matches!(square_status.status, CaseRequirementStatus::Missing));
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::EnumRequirementMissing),
        "expected EnumRequirementMissing for square, got {diags:#?}"
    );

    let bad_status = statuses.iter().find(|s| s.variant == bad_var).expect("bad status");
    assert!(matches!(bad_status.status, CaseRequirementStatus::Incompatible { .. }));
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::EnumRequirementIncompatible),
        "expected EnumRequirementIncompatible for bad, got {diags:#?}"
    );
}

#[test]
fn test_gadt_equality_refinement_and_occurs_check() {
    let mut store = TypeStore::new();
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Expr".into());

    let t_param = store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(owner.clone()),
        0,
        "T",
        KindId::TYPE,
    ));
    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let int_ty = store.nominal_type(int_decl);

    let expr_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let expr_nominal = store.nominal_form(owner.clone(), expr_kind);
    let specialized_result = store.apply_type_form(expr_nominal, &[int_ty]).expect("Expr[Int]");

    // 1. Successful GADT refinement: Expr[T] specialized to Expr[Int] -> binds T: Int
    let env = derive_case_environment(
        &mut store,
        &owner,
        &[t_param],
        Some(specialized_result),
    )
    .expect("derive case env");
    assert_eq!(env.bindings.get(&t_param), Some(&int_ty));

    // 2. Default result (no specialized annotation) -> empty environment
    let default_env = derive_case_environment(&mut store, &owner, &[t_param], None).expect("default env");
    assert!(default_env.is_empty());

    // 3. Wrong owner result -> error
    let other_owner = DeclarationId::new(module.clone(), "Other".into());
    let other_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let other_nominal = store.nominal_form(other_owner.clone(), other_kind);
    let other_result = store.apply_type_form(other_nominal, &[int_ty]).expect("Other[Int]");
    let wrong_owner_err = derive_case_environment(&mut store, &owner, &[t_param], Some(other_result));
    assert!(matches!(wrong_owner_err, Err(CaseEnvironmentError::ResultWrongOwner { .. })));

    // 4. Unsaturated result -> error
    let unapplied = store.nominal_type(owner.clone());
    let unsaturated_err = derive_case_environment(&mut store, &owner, &[t_param], Some(unapplied));
    assert!(matches!(unsaturated_err, Err(CaseEnvironmentError::ResultUnsaturated { .. })));

    // 5. Cyclic occurs check: T == List[T]
    let list_decl = DeclarationId::new(module.clone(), "List".into());
    let list_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let list_nominal = store.nominal_form(list_decl, list_kind);
    let t_ty = store.parameter_form(t_param);
    let list_t = store.apply_type_form(list_nominal, &[t_ty]).expect("List[T]");
    let cyclic_result = store.apply_type_form(expr_nominal, &[list_t]).expect("Expr[List[T]]");

    let cyclic_err = derive_case_environment(&mut store, &owner, &[t_param], Some(cyclic_result));
    assert!(matches!(cyclic_err, Err(CaseEnvironmentError::CyclicEquality { .. })));
}

#[test]
fn test_enum_declaration_module_analysis() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
enum Option<T> {
  @variant Some(value: T)
  @variant None
}

const none_case = Option::None
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));

    // Verify snapshot has registered enum product and no fatal errors
    let opt_decl = DeclarationId::new(module, "Option".into());
    let enum_info = analysis.snapshot.enum_semantics.enum_info(&opt_decl);
    assert!(enum_info.is_some(), "enum product for Option should be published");
    let info = enum_info.unwrap();
    assert_eq!(info.variants.len(), 2);
}

#[test]
fn associated_singleton_getter_publishes_exact_value_resolution() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
class Probe {
  @class run() { State::Ready }
}

enum State {
  @variant Ready
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let probe = DeclarationId::new(module.clone(), "Probe".into());
    let run = CallableId::new(probe, Selector::method("run", []).expect("run selector"), DispatchSide::Class);
    let callable = analysis.snapshot.callable_analyses.get(&run).expect("Probe.run analysis");
    let expression = callable
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("State::Ready"))
        .expect("associated singleton expression");
    let state = DeclarationId::new(module, "State".into());
    let ready = VariantId::new(state, Selector::getter("Ready").expect("Ready selector"));
    let expected = analysis
        .snapshot
        .enum_semantics
        .variant_info(&ready)
        .expect("Ready metadata")
        .exact_case_template;

    assert_eq!(expression.knowledge.ty(), Some(expected), "{expression:#?}");
    let resolution = callable.associated_resolutions.get(&expression.id).expect("associated resolution");
    assert!(matches!(
        &resolution.kind,
        AssociatedResolutionKind::ExactValue { member: phalcom_semantic::AssociatedMemberId::Variant(id), value_type }
            if id == &ready && *value_type == expected
    ));
}

#[test]
fn associated_variant_constructor_uses_canonical_call_binding() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
class Probe {
  @class run() { State::Value(value: 1) }
}

enum State {
  @variant Value(value: Int)
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let probe = DeclarationId::new(module.clone(), "Probe".into());
    let run = CallableId::new(probe, Selector::method("run", []).expect("run selector"), DispatchSide::Class);
    let callable = analysis.snapshot.callable_analyses.get(&run).expect("Probe.run analysis");
    let expression = callable
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("State::Value(value: 1)"))
        .expect("associated constructor invocation");
    let state = DeclarationId::new(module, "State".into());
    let value = VariantId::new(
        state,
        Selector::method("Value", [SelectorSlot::Label("value".to_string())]).expect("Value selector"),
    );
    let expected = analysis
        .snapshot
        .enum_semantics
        .variant_info(&value)
        .expect("Value metadata")
        .exact_case_template;

    assert_eq!(expression.knowledge.ty(), Some(expected), "{expression:#?}");
    let resolution = callable.associated_resolutions.get(&expression.id).expect("associated invocation resolution");
    assert!(matches!(
        &resolution.kind,
        AssociatedResolutionKind::StaticInvoke { member: phalcom_semantic::AssociatedMemberId::Variant(id), target, result_type }
            if id == &value && matches!(target, phalcom_semantic::InvocationTargetId::VariantConstructor(_)) && *result_type == expected
    ));
}
