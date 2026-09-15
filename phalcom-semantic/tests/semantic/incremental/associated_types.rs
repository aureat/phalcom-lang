//! Incremental ownership and cold-parity coverage for LANG005.C5.P1 products.

use super::support::single_module_input;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModulePath, ResolvedProjectId};
use phalcom_modules::ModuleId;
use phalcom_semantic::db::fingerprint::{declaration_surface_product_fingerprint, trait_surface_product_fingerprint};
use phalcom_semantic::identity::{DeclarationId, ImplId, ImplLocalId};
use phalcom_semantic::session::SemanticWorkspaceSession;

fn module() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(805),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("associated_types").unwrap()]),
    )
}

fn impl_id(module: &ModuleId) -> ImplId {
    // trait, Box, Other, impl
    ImplId::new(module.clone(), ImplLocalId(3))
}

#[test]
fn associated_binding_replacement_removal_readdition_matches_cold_products() {
    let module = module();
    let source_a = r#"
trait Iterable {
  type Item
}
class Box {}
class Other {}
impl Iterable for Box { type Item = Box }
"#;
    let source_b = source_a.replace("type Item = Box", "type Item = Other");
    let source_c = source_b.replace(" { type Item = Other }", " {}");

    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental.update(single_module_input(module.clone(), source_a, 1));
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    let first_plan = first.snapshot.conformance_witness_plans.get(&impl_id(&module)).expect("initial witness plan");
    let first_binding_fingerprint = first_plan.associated_type_plan.fingerprint;
    assert!(first_plan.associated_type_plan.failures.is_empty());

    let second = incremental.update(single_module_input(module.clone(), &source_b, 2));
    assert!(!second.snapshot.has_errors(), "replacement diagnostics: {:?}", second.snapshot.diagnostics);
    let second_plan = second.snapshot.conformance_witness_plans.get(&impl_id(&module)).expect("replaced witness plan");
    assert_ne!(first_binding_fingerprint, second_plan.associated_type_plan.fingerprint, "RHS edit must change binding plan");
    assert!(second_plan.associated_type_plan.failures.is_empty());

    let third = incremental.update(single_module_input(module.clone(), &source_c, 3));
    let third_plan = third.snapshot.conformance_witness_plans.get(&impl_id(&module)).expect("retained witness plan");
    assert!(third_plan.associated_type_plan.bindings.is_empty(), "removed binding must not remain in the plan");
    assert!(third_plan.associated_type_plan.failures.iter().any(|failure| failure.reason.contains("no conformance binding")));

    let fourth = incremental.update(single_module_input(module.clone(), &source_b, 4));
    let fourth_plan = fourth.snapshot.conformance_witness_plans.get(&impl_id(&module)).expect("re-added witness plan");
    assert!(fourth_plan.associated_type_plan.failures.is_empty());

    let mut cold = SemanticWorkspaceSession::new();
    let cold_result = cold.update(single_module_input(module.clone(), &source_b, 1));
    let cold_plan = cold_result.snapshot.conformance_witness_plans.get(&impl_id(&module)).expect("cold witness plan");
    assert_eq!(fourth_plan.associated_type_plan, cold_plan.associated_type_plan, "re-added binding must match cold analysis");
    assert_eq!(fourth_plan.completeness, cold_plan.completeness, "re-added completeness must match cold analysis");
}

#[test]
fn trait_default_body_edit_preserves_associated_identity_and_contract_product() {
    let module = module();
    let source_a = "trait Iterable { type Item\n render -> Int { 1 } }\n";
    let source_b = source_a.replace("{ 1 }", "{ 2 }");
    let declaration = DeclarationId::new(module.clone(), "Iterable".into());

    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(module.clone(), source_a, 1));
    let second = session.update(single_module_input(module.clone(), &source_b, 2));
    let first_surface = first.snapshot.trait_surfaces.get(&declaration).expect("initial trait surface");
    let second_surface = second.snapshot.trait_surfaces.get(&declaration).expect("updated trait surface");
    assert_eq!(
        first_surface.associated_type_by_name("Item").expect("initial Item").requirement,
        second_surface.associated_type_by_name("Item").expect("updated Item").requirement,
    );
    assert_eq!(trait_surface_product_fingerprint(first_surface), trait_surface_product_fingerprint(second_surface));
}

#[test]
fn incremental_read_write_delegation_to_getter_only_removes_only_setter() {
    let module = module();
    let source_a = "class Counter { mut _count: Int\n mut count via _count }\n";
    let source_b = source_a.replace("mut count via", "count via");
    let declaration = DeclarationId::new(module.clone(), "Counter".into());
    let getter = Selector::getter("count").unwrap();
    let setter = Selector::setter("count").unwrap();

    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental.update(single_module_input(module.clone(), source_a, 1));
    let first_surface = first.snapshot.surfaces.get(&declaration).expect("initial Counter surface");
    assert!(first_surface.instance.get_callable(&getter).is_some());
    assert!(first_surface.instance.get_callable(&setter).is_some());

    let second = incremental.update(single_module_input(module.clone(), &source_b, 2));
    let second_surface = second.snapshot.surfaces.get(&declaration).expect("updated Counter surface");
    assert!(second_surface.instance.get_callable(&getter).is_some());
    assert!(second_surface.instance.get_callable(&setter).is_none());

    let mut cold = SemanticWorkspaceSession::new();
    let cold_result = cold.update(single_module_input(module.clone(), &source_b, 1));
    let second_surface = second.snapshot.surfaces.get(&declaration).expect("updated Counter surface");
    let cold_surface = cold_result.snapshot.surfaces.get(&declaration).expect("cold Counter surface");
    assert_eq!(declaration_surface_product_fingerprint(second_surface), declaration_surface_product_fingerprint(cold_surface));
}
