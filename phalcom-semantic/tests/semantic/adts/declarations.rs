use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use phalcom_common::modifier::ImplementationKind;
use phalcom_common::selector::{Selector, SelectorBase, SelectorSlot};
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_native_meta::{EffectSpec, NativeLifecycleSpec, RaisesSpec, ReturnFlowSpec};
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::associated::build_associated_surface;
use phalcom_semantic::checker::AssociatedResolutionKind;
use phalcom_semantic::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::enum_requirements::{CaseRequirementStatus, EnumRequirement, EnumRequirementId, check_enum_requirements};
use phalcom_semantic::enum_semantics::{EnumInfo, VariantInfo, VariantShape, VariantVisibility};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, VariantId};
use phalcom_semantic::signature::{CallableSemanticSignature, ReturnContractValidation};
use phalcom_semantic::types::case_environment::{CaseEnvironmentError, derive_case_environment};
use phalcom_semantic::types::id::{KindId, VariantTypeId};
use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner, TypeTerm};
use phalcom_semantic::types::relation::{MapTypeHierarchy, is_subtype};
use phalcom_semantic::types::store::TypeStore;

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
        implementation: ImplementationKind::Phalcom,
        native_id: None,
        effects: EffectSpec::Unknown,
        raises: RaisesSpec::Unknown,
        flow: ReturnFlowSpec::Value,
        lifecycle: NativeLifecycleSpec::UNKNOWN,
    }
}

#[test]
fn exact_case_subtypes_only_its_specialized_enum_root() {
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
fn enum_associated_surface_rejects_namespace_collisions() {
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Status".into());

    let value_sel = Selector::getter("value").unwrap();
    let variant = VariantId::new(owner.clone(), value_sel.clone());

    // Conflict with behavior of same base
    let mut behavior_bases = HashSet::new();
    behavior_bases.insert(value_sel.base.clone());
    let (surface, diags) = build_associated_surface(&owner, Some(std::slice::from_ref(&variant)), &behavior_bases, &HashSet::new(), &module, None);
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::EnumFamilyCategoryConflict),
        "expected EnumFamilyCategoryConflict, got {diags:#?}"
    );
    assert_eq!(surface.families.len(), 0);

    // Conflict with inherited class behavior
    let mut inherited_bases = HashSet::new();
    inherited_bases.insert(SelectorBase::Named("value".into()));
    let (_surface, inherited_diags) = build_associated_surface(&owner, Some(&[variant]), &HashSet::new(), &inherited_bases, &module, None);
    assert!(
        inherited_diags.iter().any(|d| d.code == DiagnosticCode::EnumFamilyInheritedBehaviorConflict),
        "expected EnumFamilyInheritedBehaviorConflict, got {inherited_diags:#?}"
    );
}

#[test]
fn adt_req_01_02_04_requirements_report_satisfied_missing_and_incompatible_cases() {
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
            payload_kind: phalcom_semantic::enum_semantics::AlgebraicCasePayloadKind::Singleton,
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
            payload_kind: phalcom_semantic::enum_semantics::AlgebraicCasePayloadKind::Singleton,
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
            payload_kind: phalcom_semantic::enum_semantics::AlgebraicCasePayloadKind::Singleton,
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
        static_data: None,
        default_result_type: nominal_owner,
        variants: variants.iter().map(|v| v.id.clone()).collect(),
        variant_families: Box::new([]),
        source: None,
    };

    let (statuses, diags) = check_enum_requirements(&owner, &enum_info, &variants, &[root_req], &case_methods, &mut store, &hier, &module);

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
fn gadt_case_environment_refines_and_rejects_cyclic_results() {
    let mut store = TypeStore::new();
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Expr".into());

    let t_param = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(owner.clone()), 0, "T", KindId::TYPE));
    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let int_ty = store.nominal_type(int_decl);

    let expr_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let expr_nominal = store.nominal_form(owner.clone(), expr_kind);
    let specialized_result = store.apply_type_form(expr_nominal, &[int_ty]).expect("Expr[Int]");

    // 1. Successful GADT refinement: Expr[T] specialized to Expr[Int] -> binds T: Int
    let env = derive_case_environment(&mut store, &owner, &[t_param], Some(specialized_result)).expect("derive case env");
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
fn enum_declaration_publishes_structural_enum_product() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Option<T> {
  Some(value: T)
  None
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
fn class_owned_associated_type_data_publishes_stable_domain_and_cases() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Container<T> {
  type data State {
    Start
    Done
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let owner = DeclarationId::new(module, "Container".into());
    let id = phalcom_semantic::identity::AssociatedStaticDomainDeclarationId::new(owner.clone(), "State");
    let declaration = analysis.snapshot.associated_static_domains.get(&id).expect("published declaration product");
    assert_eq!(declaration.constructors.len(), 2);
    let surface = analysis.snapshot.associated_surfaces.surfaces.get(&owner).expect("associated surface");
    let family = surface.families.get(&SelectorBase::Named("State".into())).expect("admitted static domain");
    assert_eq!(family.kind, phalcom_semantic::associated::AssociatedFamilyKind::StaticDomain);
    assert_eq!(family.members.as_ref(), &[phalcom_semantic::AssociatedMemberId::StaticDomain(id.clone())]);
    assert!(matches!(
        analysis.snapshot.store.static_domain(declaration.domain),
        Some(phalcom_semantic::types::store::StaticDomainData::Associated { declaration: found, owner_arguments })
            if found == &id && owner_arguments.len() == 1
    ));
}

#[test]
fn associated_static_domain_payload_uses_owner_generic_parameter_kind() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Container<T: Type> {
  type data State {
    Box(_ value: T)
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let owner = DeclarationId::new(module, "Container".into());
    let id = phalcom_semantic::identity::AssociatedStaticDomainDeclarationId::new(owner, "State");
    let declaration = analysis.snapshot.associated_static_domains.get(&id).expect("published declaration");
    assert_eq!(declaration.constructors.len(), 1);
    let constructor = analysis
        .snapshot
        .store
        .static_constructor(declaration.constructors[0])
        .expect("constructor metadata");
    assert_eq!(constructor.parameter_kinds.as_ref(), &[KindId::TYPE]);
}

#[test]
fn runtime_data_inherent_impl_publishes_associated_static_domain() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
data Packet(bytes: Int)
impl Packet {
  type data State {
    Unread
    Parsed
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let owner = DeclarationId::new(module, "Packet".into());
    let id = phalcom_semantic::identity::AssociatedStaticDomainDeclarationId::new(owner.clone(), "State");
    assert_eq!(
        analysis
            .snapshot
            .associated_static_domains
            .get(&id)
            .expect("published declaration")
            .constructors
            .len(),
        2
    );
    assert!(
        analysis
            .snapshot
            .associated_surfaces
            .surfaces
            .get(&owner)
            .unwrap()
            .families
            .contains_key(&SelectorBase::Named("State".into()))
    );
}

#[test]
fn associated_static_domain_duplicate_across_inherent_impl_fragments_is_rejected() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
data Packet(bytes: Int)
impl Packet { type data State { Unread } }
impl Packet { type data State { Parsed } }
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let owner = DeclarationId::new(module, "Packet".into());
    let id = phalcom_semantic::identity::AssociatedStaticDomainDeclarationId::new(owner.clone(), "State");
    assert!(!analysis.snapshot.associated_static_domains.contains_key(&id));
    assert!(
        analysis
            .snapshot
            .diagnostics
            .values()
            .flat_map(|diagnostics| diagnostics.iter())
            .any(|diagnostic| diagnostic.code == DiagnosticCode::EnumFamilyCategoryConflict)
    );
}

#[test]
fn conformance_impl_cannot_introduce_associated_static_domain() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
trait Marker {}
data Packet(bytes: Int)
impl Marker for Packet {
  type data State { Unread }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let owner = DeclarationId::new(module, "Packet".into());
    let id = phalcom_semantic::identity::AssociatedStaticDomainDeclarationId::new(owner, "State");
    assert!(!analysis.snapshot.associated_static_domains.contains_key(&id));
    assert!(
        analysis
            .snapshot
            .diagnostics
            .values()
            .flat_map(|diagnostics| diagnostics.iter())
            .any(|diagnostic| { diagnostic.message.contains("not permitted in a conformance impl") })
    );
}

#[test]
fn associated_static_domain_and_case_resolve_as_static_semantic_results() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Parser {
  type data State { Start Done }
  @class status() { Parser::State::Done }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let owner = DeclarationId::new(module.clone(), "Parser".into());
    let status = CallableId::new(owner, Selector::method("status", []).expect("status selector"), DispatchSide::Class);
    let callable = analysis.snapshot.callable_analyses.get(&status).expect("status analysis");
    assert!(
        callable
            .associated_resolutions
            .values()
            .any(|resolution| matches!(resolution.kind, phalcom_semantic::checker::AssociatedResolutionKind::StaticDomain { .. }))
    );
    assert!(
        callable
            .associated_resolutions
            .values()
            .any(|resolution| matches!(resolution.kind, phalcom_semantic::checker::AssociatedResolutionKind::StaticTerm { .. }))
    );
    assert!(
        !callable
            .associated_resolutions
            .values()
            .any(|resolution| matches!(resolution.kind, phalcom_semantic::checker::AssociatedResolutionKind::StaticInvoke { .. }))
    );
}

#[test]
fn associated_static_domain_lookup_specializes_generic_owner_canonically() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
class Container<T: Type> {
  type data State { Start }
}
class Probe {
  @class intState() { Container<Int>::State::Start }
  @class stringState() { Container<String>::State::Start }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let owner = DeclarationId::new(module.clone(), "Container".into());
    let declaration = phalcom_semantic::identity::AssociatedStaticDomainDeclarationId::new(owner.clone(), "State");
    let probe = DeclarationId::new(module, "Probe".into());
    let int_method = CallableId::new(probe.clone(), Selector::method("intState", []).expect("selector"), DispatchSide::Class);
    let string_method = CallableId::new(probe, Selector::method("stringState", []).expect("selector"), DispatchSide::Class);
    let domain_for = |callable: &CallableId| {
        analysis
            .snapshot
            .callable_analyses
            .get(callable)
            .expect("callable analysis")
            .associated_resolutions
            .values()
            .find_map(|resolution| match &resolution.kind {
                phalcom_semantic::checker::AssociatedResolutionKind::StaticDomain {
                    member: phalcom_semantic::AssociatedMemberId::StaticDomain(id),
                    domain,
                } if id == &declaration => Some(*domain),
                _ => None,
            })
            .expect("specialized associated domain")
    };
    let int_domain = domain_for(&int_method);
    let string_domain = domain_for(&string_method);
    assert_ne!(int_domain, string_domain);
    assert!(
        matches!(analysis.snapshot.store.static_domain(int_domain), Some(phalcom_semantic::types::store::StaticDomainData::Associated { declaration: id, .. }) if id == &declaration)
    );
    assert!(
        matches!(analysis.snapshot.store.static_domain(string_domain), Some(phalcom_semantic::types::store::StaticDomainData::Associated { declaration: id, .. }) if id == &declaration)
    );
}

#[test]
fn singleton_associated_getter_publishes_exact_value_resolution() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Probe {
  @class run() { State::Ready }
}

data State {
  Ready
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
fn variant_constructor_invocation_uses_canonical_call_binding() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Probe {
  @class run() { State::Value(value: 1) }
}

data State {
  Value(value: Int)
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
        AssociatedResolutionKind::StaticInvoke { member: phalcom_semantic::AssociatedMemberId::Variant(id), target, result_type, .. }
            if id == &value && matches!(target, phalcom_semantic::InvocationTargetId::VariantConstructor(_)) && *result_type == expected
    ));
}

#[test]
fn adt_decl_01_ordinary_closed_enum_publishes_owner_and_ordered_variants() {
    let case = super::support::analyze_adt("data Status {\n  Active -> Status\n  Inactive -> Status\n}\n");
    let info = case.enum_info("Status");
    assert_eq!(info.variants.len(), 2);
    assert_eq!(info.variants[0].owner.name.as_ref(), "Status");
    assert_eq!(info.variants[1].owner.name.as_ref(), "Status");
    assert_eq!(info.variants[0].selector.encode(), "Active");
    assert_eq!(info.variants[1].selector.encode(), "Inactive");
    case.assert_no_diagnostics();
}

#[test]
fn adt_decl_02_generic_parameter_is_owned_by_enum_and_reused_by_case() {
    let case = super::support::analyze_adt("data Option<T> {\n  Some(_ value: T) -> Option<T>\n  None -> Option<T>\n}\n");
    let owner = &case.declaration("Option").declaration;
    let signature = case.declaration("Option").generic_signature.as_ref().expect("generic signature");
    assert_eq!(signature.parameter_count(), 1);
    let parameter = signature.parameters[0];
    let store = case.analysis.snapshot.store.as_ref();
    assert_eq!(
        &store.type_parameter(parameter).owner,
        &phalcom_semantic::TypeParameterOwner::Declaration(owner.clone())
    );
    let some = case.variant("Option", Selector::method("Some", [SelectorSlot::Positional]).expect("Some selector"));
    assert_eq!(
        store.get(some.fields[0].declared_type.canonical_type().expect("Some field type")),
        &phalcom_semantic::types::store::TypeData::Parameter(parameter)
    );
    assert!(some.case_environment.is_empty());
    case.assert_no_diagnostics();
}

#[test]
fn adt_decl_03_multi_parameter_result_keeps_parameter_order_and_roles() {
    let case = super::support::analyze_adt("data Result<T, E> {\n  Ok(_ value: T) -> Result<T, E>\n  Err(_ error: E) -> Result<T, E>\n}\n");
    let declaration = case.declaration("Result");
    let signature = declaration.generic_signature.as_ref().expect("Result generic signature");
    assert_eq!(signature.parameter_count(), 2);
    let store = case.analysis.snapshot.store.as_ref();
    let ok = case.variant("Result", Selector::method("Ok", [SelectorSlot::Positional]).expect("Ok selector"));
    let err = case.variant("Result", Selector::method("Err", [SelectorSlot::Positional]).expect("Err selector"));
    assert_eq!(
        store.get(ok.fields[0].declared_type.canonical_type().expect("Ok field type")),
        &phalcom_semantic::types::store::TypeData::Parameter(signature.parameters[0])
    );
    assert_eq!(
        store.get(err.fields[0].declared_type.canonical_type().expect("Err field type")),
        &phalcom_semantic::types::store::TypeData::Parameter(signature.parameters[1])
    );
    assert_eq!(store.type_parameter(signature.parameters[0]).index, 0);
    assert_eq!(store.type_parameter(signature.parameters[1]).index, 1);
}

#[test]
fn adt_decl_04_gadt_result_templates_publish_case_environments() {
    let case = super::support::analyze_adt("data Expr<T> {\n  Int(_ value: Int) -> Expr<Int>\n  Bool(_ value: Bool) -> Expr<Bool>\n}\n");
    let expr = case.enum_info("Expr");
    assert_eq!(expr.variants.len(), 2);
    let int = case.variant("Expr", Selector::method("Int", [SelectorSlot::Positional]).expect("Int selector"));
    let bool_case = case.variant("Expr", Selector::method("Bool", [SelectorSlot::Positional]).expect("Bool selector"));
    assert_eq!(int.case_environment.bindings.len(), 1);
    assert_eq!(bool_case.case_environment.bindings.len(), 1);
    assert_ne!(int.result_type_template, bool_case.result_type_template);
}

#[test]
fn adt_decl_05_same_base_distinct_selector_declarations_do_not_collapse() {
    let case =
        super::support::analyze_adt("data Animal {\n  Dog\n  Dog()\n  Dog(_ name: String)\n  Dog(named age: Int)\n}\n");
    let info = case.enum_info("Animal");
    assert_eq!(info.variants.len(), 4);
    let selectors = info.variants.iter().map(|variant| variant.selector.encode()).collect::<Vec<_>>();
    assert_eq!(
        selectors,
        vec!["Dog".to_string(), "Dog()".to_string(), "Dog(_)".to_string(), "Dog(named)".to_string()]
    );
    assert_eq!(info.variant_families.len(), 1);
}

#[test]
fn adt_decl_06_duplicate_exact_selector_reports_specific_diagnostic() {
    let case = super::support::analyze_adt("data Duplicate {\n  A\n  A\n}\n");
    assert_eq!(case.diagnostics_for(DiagnosticCode::EnumVariantDuplicate).len(), 1);
    assert_eq!(case.enum_info("Duplicate").variants.len(), 2);
}

#[test]
fn adt_decl_08_source_ranges_cover_enum_and_variant_metadata() {
    let case = super::support::analyze_adt("data Status {\n  Active(_ code: Int) -> Status\n}\n");
    let info = case.enum_info("Status");
    assert!(info.source.as_ref().is_some_and(|span| span.range.start < span.range.end));
    let variant = case.variant("Status", Selector::method("Active", [SelectorSlot::Positional]).expect("Active selector"));
    assert!(variant.source.as_ref().is_some_and(|span| span.range.start < span.range.end));
    assert!(variant.fields[0].source.as_ref().is_some_and(|span| span.range.start < span.range.end));
}

#[test]
fn equivalent_legacy_enum_and_data_produce_equal_semantic_facts() {
    let enum_case = super::support::analyze_adt("data State {\n  Ready\n  Failed(_ error: Int)\n}\n");
    let data_case = super::support::analyze_adt("data State {\n  Ready\n  Failed(_ error: Int)\n}\n");

    let enum_info = enum_case.enum_info("State");
    let data_info = data_case.enum_info("State");

    assert_eq!(enum_info.owner.name, data_info.owner.name);
    assert_eq!(enum_info.variants.len(), data_info.variants.len());

    let ready_sel = Selector::getter("Ready").expect("Ready getter");
    let enum_ready = enum_case.variant("State", ready_sel.clone());
    let data_ready = data_case.variant("State", ready_sel);
    assert_eq!(enum_ready.shape, data_ready.shape);
    assert_eq!(enum_ready.payload_kind, data_ready.payload_kind);
    assert_eq!(enum_ready.fields.len(), data_ready.fields.len());

    let failed_sel = Selector::method("Failed", [SelectorSlot::Positional]).expect("Failed method");
    let enum_failed = enum_case.variant("State", failed_sel.clone());
    let data_failed = data_case.variant("State", failed_sel);
    assert_eq!(enum_failed.shape, data_failed.shape);
    assert_eq!(enum_failed.payload_kind, data_failed.payload_kind);
    assert_eq!(enum_failed.fields.len(), data_failed.fields.len());
    assert_eq!(enum_failed.fields[0].local_name, data_failed.fields[0].local_name);
}

#[test]
fn record_payload_case_produces_labeled_constructor_and_fields() {
    let case = super::support::analyze_adt("data Message {\n  Request {\n    method: Int,\n    path: String\n  }\n  Closed\n}\n");
    let info = case.enum_info("Message");
    assert_eq!(info.variants.len(), 2);

    let req_sel = Selector::method("Request", [SelectorSlot::Label("method".into()), SelectorSlot::Label("path".into())]).expect("Request selector");
    let req_variant = case.variant("Message", req_sel);
    assert_eq!(req_variant.shape, VariantShape::Constructor);
    assert_eq!(req_variant.payload_kind, phalcom_semantic::enum_semantics::AlgebraicCasePayloadKind::Record);
    assert_eq!(req_variant.fields.len(), 2);
    assert_eq!(req_variant.fields[0].local_name.as_ref(), "method");
    assert_eq!(req_variant.fields[0].external_label.as_deref(), Some("method"));
    assert_eq!(req_variant.fields[1].local_name.as_ref(), "path");
    assert_eq!(req_variant.fields[1].external_label.as_deref(), Some("path"));

    let ctor = req_variant.constructor.as_ref().expect("constructor exists");
    assert_eq!(ctor.parameters.len(), 2);
    assert_eq!(ctor.parameters[0].external_label.as_deref(), Some("method"));
    assert_eq!(ctor.parameters[1].external_label.as_deref(), Some("path"));
}

#[test]
fn associated_static_domain_and_case_paths_have_stable_source_index_targets() {
    let module = test_module();
    let source: Arc<str> = Arc::from("class Parser {\n  type data State { Start; Done }\n  @class status() { Parser::State::Done }\n}\n");
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let index = analysis.snapshot.source_index.module(&module).expect("module source index");
    let domain_id = phalcom_semantic::identity::AssociatedStaticDomainDeclarationId::new(DeclarationId::new(module, "Parser".into()), "State");

    let state_offset = source.find("State::Done").expect("associated domain path") + 1;
    let state_target = index.occurrences.occurrence_at(state_offset).and_then(|occurrence| occurrence.target.cloned());
    assert_eq!(
        state_target,
        Some(phalcom_semantic::identity::SemanticTargetId::StaticDomain(
            phalcom_semantic::identity::StaticDomainDeclarationId::Associated(domain_id.clone())
        ))
    );

    let case_offset = source.rfind("Done").expect("static case path") + 1;
    let case_target = index.occurrences.occurrence_at(case_offset).and_then(|occurrence| occurrence.target.cloned());
    assert_eq!(
        case_target,
        Some(phalcom_semantic::identity::SemanticTargetId::StaticCase(
            phalcom_semantic::identity::StaticCaseId::new(phalcom_semantic::identity::StaticDomainDeclarationId::Associated(domain_id), "Done",)
        ))
    );
}

#[test]
fn nominal_static_case_publishes_canonical_target_before_runtime_rejection() {
    use phalcom_semantic::identity::{SemanticTargetId, StaticCaseId, StaticDomainDeclarationId};

    let module = test_module();
    let source: Arc<str> = Arc::from("type data Shape { Scalar }\nclass Probe {\n  @class status() { Shape::Scalar }\n}\n");
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    assert!(
        analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| { diagnostic.code == DiagnosticCode::TypeOnlyRuntimeValue }),
        "runtime use must remain rejected: {:?}",
        analysis.snapshot.diagnostics
    );

    let index = analysis.snapshot.source_index.module(&module).expect("module source index");
    let case_offset = source.rfind("Scalar").expect("case path") + 1;
    let case_target = index.occurrences.occurrence_at(case_offset).and_then(|occurrence| occurrence.target.cloned());
    assert_eq!(
        case_target,
        Some(SemanticTargetId::StaticCase(StaticCaseId::new(
            StaticDomainDeclarationId::Nominal(DeclarationId::new(module, "Shape".into())),
            "Scalar",
        )))
    );

    let owner = DeclarationId::new(test_module(), "Probe".into());
    let status = CallableId::new(owner, Selector::method("status", []).expect("selector"), DispatchSide::Class);
    let callable = analysis.snapshot.callable_analyses.get(&status).expect("status analysis");
    assert!(
        callable
            .associated_resolutions
            .values()
            .any(|resolution| matches!(resolution.kind, AssociatedResolutionKind::StaticTerm { .. })),
        "canonical static-term resolution must coexist with the runtime diagnostic"
    );
}

#[test]
fn missing_nominal_static_case_has_no_source_target() {
    let module = test_module();
    let source: Arc<str> = Arc::from("type data Shape { Scalar }\nclass Probe {\n  @class status() { Shape::Missing }\n}\n");
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    assert!(
        analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| { diagnostic.code == DiagnosticCode::AssociatedMemberMissing }),
        "missing case must be diagnosed: {:?}",
        analysis.snapshot.diagnostics
    );

    let index = analysis.snapshot.source_index.module(&module).expect("module source index");
    let case_offset = source.rfind("Missing").expect("missing case path") + 1;
    let case_target = index.occurrences.occurrence_at(case_offset).and_then(|occurrence| occurrence.target.cloned());
    assert!(case_target.is_none(), "missing case must not produce a semantic target: {case_target:?}");
}
