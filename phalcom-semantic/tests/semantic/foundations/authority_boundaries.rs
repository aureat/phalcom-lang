use crate::semantic::support::Fixture;
use phalcom_modules::ModulePath;
use phalcom_modules::identity::{ModuleId, SyntheticProjectIdAllocator};
use phalcom_native_meta::UniverseKey;
use phalcom_semantic::checker::{ExpectationOrigin, ExpectedType};
use phalcom_semantic::declarations::GenericSupertypeTemplate;
use phalcom_semantic::identity::{DeclarationId, DispatchSide};
use phalcom_semantic::types::evidence::{EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::relation::{MapTypeHierarchy, check_assignability, is_subtype};
use phalcom_semantic::types::store::TypeStore;
use phalcom_semantic::types::variance::Variance;

#[test]
fn user_object_name_is_not_universal_supertype() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let mut alloc = SyntheticProjectIdAllocator;

    let user_module = ModuleId::synthetic(alloc.allocate(), ModulePath::root());
    let user_object_decl = DeclarationId::new(user_module.clone(), "Object".into());
    let user_unrelated_decl = DeclarationId::new(user_module, "Unrelated".into());

    let t_user_obj = store.nominal(user_object_decl);
    let t_unrelated = store.nominal(user_unrelated_decl);

    // Unrelated is NOT a subtype of user-defined Object
    assert!(!is_subtype(&mut store, &hier, t_unrelated, t_user_obj));

    // But Unrelated IS a subtype of canonical core Object
    let core_object = phalcom_semantic::core_surface::universe_declaration(UniverseKey::Object);
    let t_core_obj = store.nominal(core_object);
    assert!(is_subtype(&mut store, &hier, t_unrelated, t_core_obj));
}

#[test]
fn user_function_name_is_not_callable_supertype() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let mut alloc = SyntheticProjectIdAllocator;

    let user_module = ModuleId::synthetic(alloc.allocate(), ModulePath::root());
    let user_func_decl = DeclarationId::new(user_module, "Function".into());

    let t_user_func = store.nominal(user_func_decl);

    let callable_ty = store.callable(phalcom_semantic::types::store::CallableType {
        parameters: Box::new([]),
        return_type: store.unit(),
    });

    // Callable is NOT a subtype of user-defined Function
    assert!(!is_subtype(&mut store, &hier, callable_ty, t_user_func));

    // Callable IS a subtype of core Function
    let core_func = phalcom_semantic::core_surface::universe_declaration(UniverseKey::Function);
    let t_core_func = store.nominal(core_func);
    assert!(is_subtype(&mut store, &hier, callable_ty, t_core_func));
}

#[test]
fn proven_relation_does_not_upgrade_assumed_actual() {
    let mut store = TypeStore::new();
    let mut hier = MapTypeHierarchy::new();

    let int_decl = phalcom_semantic::core_surface::universe_declaration(UniverseKey::Int);
    let num_decl = phalcom_semantic::core_surface::universe_declaration(UniverseKey::Number);
    hier.insert(int_decl.clone(), num_decl.clone());

    let t_int = store.nominal(int_decl);
    let t_num = store.nominal(num_decl);

    let actual = TypeKnowledge::assumed(t_int, EvidenceOrigin::DeveloperAnnotation);
    let expected = TypeKnowledge::assumed(t_num, EvidenceOrigin::DeveloperAnnotation);

    let assignability = check_assignability(&mut store, &hier, &actual, &expected);
    assert!(assignability.is_assignable());
    // Assignability proven does not change actual status
    assert_eq!(actual.status(), Some(EvidenceStatus::Assumed));
}

#[test]
fn generic_supertype_specialization_materializes_in_live_store() {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let module = ModuleId::universe_root();
    let names_decl = DeclarationId::new(module.clone(), "Names".into());
    let sequence_decl = DeclarationId::new(module.clone(), "Sequence".into());
    let int_decl = phalcom_semantic::core_surface::universe_declaration(UniverseKey::Int);
    let object_decl = phalcom_semantic::core_surface::universe_declaration(UniverseKey::Object);

    let unary_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let names_form = store.nominal_form(names_decl.clone(), unary_kind);
    let sequence_form = store.nominal_form(sequence_decl.clone(), unary_kind);
    store.set_parameter_variance(sequence_decl, 0, Variance::Covariant);
    let int = store.nominal_type(int_decl);
    let object = store.nominal_type(object_decl);
    let parameter = store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(names_decl.clone()),
        0,
        "T",
        KindId::TYPE,
    ));
    let parameter_form = store.parameter_form(parameter);
    let template = store.apply_type_form(sequence_form, &[parameter_form]).expect("well-kinded template");
    hierarchy.insert_template(GenericSupertypeTemplate {
        declaration: names_decl,
        supertype: template,
        structural_form: None,
    });

    let names_int = store.apply_type_form(names_form, &[int]).expect("well-kinded Names application");
    let sequence_object = store.apply_type_form(sequence_form, &[object]).expect("well-kinded expected application");
    let before = store.type_count();

    assert!(is_subtype(&mut store, &hierarchy, names_int, sequence_object));
    assert!(store.type_count() > before, "specialized Sequence<Int> must be interned in the live store");
}

#[test]
fn comparison_chain_single_evaluation_and_operation_conjunction() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let a = 1
    let b = 2
    let c = 3
    let res = a < b < c
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let res_binding = f.binding(run, "res");
    assert_eq!(res_binding.current.ty(), Some(f.ty("Bool")));
    assert_eq!(res_binding.current.status(), Some(EvidenceStatus::Established));
}

#[test]
fn comparison_chain_missing_operator_fails_closed() {
    let f = Fixture::new(
        r#"
class NoOps {
  @constructor new() {}
}
class Probe {
  @class
  run() {
    let x = NoOps.new()
    let y = NoOps.new()
    let z = NoOps.new()
    let res = x < y < z
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let res_binding = f.binding(run, "res");
    assert!(matches!(res_binding.current, TypeKnowledge::Unknown(_)));
}

#[test]
fn membership_fails_closed_to_unknown() {
    let f = Fixture::new(
        r#"
class Container {
  @constructor new() {}
}
class Probe {
  @class
  run() {
    let c = Container.new()
    let res = 1 in c
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let res_binding = f.binding(run, "res");
    assert!(matches!(res_binding.current, TypeKnowledge::Unknown(UnknownReason::UncheckedExpression)));
}

#[test]
fn contextual_empty_list_inherits_expected_contract_authority() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let xs: List<Int> = []
    let ys = []
    let wrong: Map<String, Int> = []
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let xs_binding = f.binding(run, "xs");
    assert!(xs_binding.current.is_known());
    assert_eq!(xs_binding.current.status(), Some(EvidenceStatus::Assumed));

    let ys_binding = f.binding(run, "ys");
    assert!(matches!(ys_binding.current, TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)));

    let wrong_context_literal = f.expression_n(run, "[]", 2);
    assert!(matches!(wrong_context_literal.knowledge, TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)));
}

#[test]
fn contextual_empty_map_preserves_expected_type() {
    let mut store = TypeStore::new();
    let map_kind = store.arrow_kind(vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let map_form = store.nominal_form(phalcom_semantic::core_surface::universe_declaration(UniverseKey::Map), map_kind);
    let string = store.nominal_type(phalcom_semantic::core_surface::universe_declaration(UniverseKey::String));
    let int = store.nominal_type(phalcom_semantic::core_surface::universe_declaration(UniverseKey::Int));
    let expected = store.map_of(map_form, string, int).expect("well-kinded Map application");

    let contextual = ExpectedType::proper_from(expected, ExpectationOrigin::DeclarationContract)
        .contextual_knowledge(expected)
        .expect("matching contextual type");

    assert_eq!(contextual.ty(), Some(expected));
    assert_eq!(contextual.status(), Some(EvidenceStatus::Assumed));
    assert_eq!(contextual.origin(), Some(EvidenceOrigin::ContextualDerivation));
}
