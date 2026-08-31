use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact};
use phalcom_semantic::enum_requirements::{CaseRequirementStatus, EnumRequirement, EnumRequirementId, check_enum_requirements};
use phalcom_semantic::enum_semantics::{EnumInfo, VariantInfo, VariantShape, VariantVisibility};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, VariantId};
use phalcom_semantic::signature::{CallableSemanticSignature, ReturnContractValidation};
use phalcom_semantic::types::case_environment::CaseTypeEnvironment;
use phalcom_semantic::types::id::{KindId, VariantTypeId};
use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner, TypeTerm};
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;
use phalcom_native_meta::{EffectSpec, ImplementationKind, NativeLifecycleSpec, RaisesSpec, ReturnFlowSpec};
use std::collections::HashMap;

#[test]
fn adt_req_00_requirement_identity_is_owner_and_selector_qualified() {
    let owner = DeclarationId::new(ModuleId::core(), "Shape".into());
    let describe = Selector::getter("describe").expect("describe selector");
    let other = Selector::getter("render").expect("render selector");

    let first = EnumRequirementId::new(owner.clone(), describe.clone());
    let same = EnumRequirementId::new(owner.clone(), describe);
    let different = EnumRequirementId::new(owner, other);

    assert_eq!(first, same);
    assert_ne!(first, different);
}

fn singleton_variant(store: &mut TypeStore, owner: &DeclarationId, name: &str, handle: u32) -> VariantInfo {
    let selector = Selector::getter(name).expect("singleton selector");
    let id = VariantId::new(owner.clone(), selector);
    let root = store.nominal_type(owner.clone());
    let exact = store.exact_case_type(&id, root).expect("exact singleton case");
    VariantInfo {
        id: id.clone(),
        type_handle: VariantTypeId(handle),
        family: id.family(),
        shape: VariantShape::Singleton,
        fields: Box::new([]),
        result_type_template: root,
        exact_case_template: exact,
        case_environment: CaseTypeEnvironment::default(),
        constructor: None,
        visibility: VariantVisibility::default(),
        source: None,
    }
}

fn enum_info(owner: DeclarationId, root: phalcom_semantic::types::id::TypeId, variants: &[VariantInfo]) -> EnumInfo {
    EnumInfo {
        owner,
        root_form: root,
        generic_signature: None,
        default_result_type: root,
        variants: variants.iter().map(|variant| variant.id.clone()).collect(),
        variant_families: Box::new([]),
        source: None,
    }
}

fn signature(owner: DeclarationId, selector: Selector, return_type: DeclaredTypeFact) -> CallableSemanticSignature {
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
fn adt_req_03_selector_or_arity_mismatch_is_missing_not_satisfied() {
    let module = ModuleId::core();
    let owner = DeclarationId::new(module.clone(), "Shape".into());
    let mut store = TypeStore::new();
    let root = store.nominal_type(owner.clone());
    let variant = singleton_variant(&mut store, &owner, "Circle", 1);
    let info = enum_info(owner.clone(), root, std::slice::from_ref(&variant));
    let describe = Selector::getter("describe").expect("getter requirement");
    let wrong_shape = Selector::method("describe", []).expect("method implementation");
    let requirement = EnumRequirement {
        id: EnumRequirementId::new(owner.clone(), describe),
        signature: signature(owner.clone(), Selector::getter("describe").unwrap(), DeclaredTypeFact::known(TypeTerm::Canonical(root), DeclaredTypeBasis::SourceAnnotation)),
        source: None,
    };
    let mut methods = HashMap::new();
    methods.insert(variant.id.clone(), vec![signature(owner.clone(), wrong_shape, DeclaredTypeFact::known(TypeTerm::Canonical(root), DeclaredTypeBasis::SourceAnnotation))]);

    let (statuses, diagnostics) = check_enum_requirements(
        &owner,
        &info,
        std::slice::from_ref(&variant),
        &[requirement],
        &methods,
        &mut store,
        &MapTypeHierarchy::new(),
        &module,
    );
    assert!(matches!(statuses[0].status, CaseRequirementStatus::Missing));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::EnumRequirementMissing));
}

#[test]
fn adt_req_05_generic_requirement_specializes_through_case_environment() {
    let module = ModuleId::core();
    let owner = DeclarationId::new(module.clone(), "Expr".into());
    let mut store = TypeStore::new();
    let parameter = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(owner.clone()), 0, "T", KindId::TYPE));
    let kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let root_form = store.nominal_form(owner.clone(), kind);
    let int = store.nominal_type(DeclarationId::new(module.clone(), "Int".into()));
    let specialized_root = store.apply_type_form(root_form, &[int]).expect("Expr<Int>");
    let selector = Selector::method("Int", [phalcom_common::selector::SelectorSlot::Positional]).expect("Int variant");
    let variant_id = VariantId::new(owner.clone(), selector);
    let exact = store.exact_case_type(&variant_id, specialized_root).expect("exact Int case");
    let environment = phalcom_semantic::types::case_environment::derive_case_environment(&mut store, &owner, &[parameter], Some(specialized_root)).expect("case environment");
    let variant = VariantInfo {
        id: variant_id.clone(),
        type_handle: VariantTypeId(1),
        family: variant_id.family(),
        shape: VariantShape::Constructor,
        fields: Box::new([]),
        result_type_template: specialized_root,
        exact_case_template: exact,
        case_environment: environment,
        constructor: None,
        visibility: VariantVisibility::default(),
        source: None,
    };
    let info = enum_info(owner.clone(), root_form, std::slice::from_ref(&variant));
    let requirement_selector = Selector::method("eval", []).expect("eval selector");
    let requirement = EnumRequirement {
        id: EnumRequirementId::new(owner.clone(), requirement_selector.clone()),
        signature: signature(owner.clone(), requirement_selector.clone(), DeclaredTypeFact::known(TypeTerm::Canonical(store.parameter_form(parameter)), DeclaredTypeBasis::SourceAnnotation)),
        source: None,
    };
    let mut methods = HashMap::new();
    methods.insert(variant_id, vec![signature(owner.clone(), requirement_selector, DeclaredTypeFact::known(TypeTerm::Canonical(int), DeclaredTypeBasis::SourceAnnotation))]);
    let (statuses, diagnostics) = check_enum_requirements(
        &owner,
        &info,
        std::slice::from_ref(&variant),
        &[requirement],
        &methods,
        &mut store,
        &MapTypeHierarchy::new(),
        &module,
    );
    assert!(diagnostics.is_empty(), "generic requirement should specialize cleanly: {diagnostics:#?}");
    assert!(matches!(statuses[0].status, CaseRequirementStatus::Satisfied { .. }));
}
