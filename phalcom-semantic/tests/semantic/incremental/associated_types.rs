//! Incremental ownership and cold-parity coverage for LANG005.C5.P1 products.

use super::support::single_module_input;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModulePath, ResolvedProjectId};
use phalcom_modules::ModuleId;
use phalcom_semantic::core_surface::CoreDeclarationIds;
use phalcom_semantic::db::fingerprint::{declaration_surface_product_fingerprint, trait_surface_product_fingerprint};
use phalcom_semantic::identity::{DeclarationId, ImplId, ImplLocalId, SemanticTargetId};
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
  next -> Self::Item
}
class Box {}
class Other {}
impl Iterable for Box {
  type Item = Int
  next -> Int { 0 }
}
"#;
    let source_b = source_a.replace("type Item = Int", "type Item = String").replace("next -> Int", "next -> String").replace("{ 0 }", "{ \"text\" }");
    let source_c = source_b.replace("  type Item = String\n", "");

    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental.update(single_module_input(module.clone(), source_a, 1));
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    let first_plan = first.snapshot.conformance_witness_plans.get(&impl_id(&module)).expect("initial witness plan");
    let first_binding_fingerprint = first_plan.associated_type_plan.fingerprint;
    assert!(first_plan.associated_type_plan.failures.is_empty());
    let trait_ref = phalcom_semantic::traits::TraitRef::new(
        DeclarationId::new(module.clone(), "Iterable".into()),
        Vec::new().into_boxed_slice(),
    );
    let box_declaration = DeclarationId::new(module.clone(), "Box".into());
    let first_box = first.snapshot.declarations.form(&box_declaration).expect("initial Box form");
    let first_evidence = first.snapshot.conformance_evidence_for(first_box, &trait_ref).expect("initial exact evidence");
    let next_requirement = first
        .snapshot
        .trait_surfaces
        .get(&trait_ref.declaration)
        .expect("Iterable surface")
        .get_by_selector(&Selector::getter("next").expect("next selector"), phalcom_semantic::identity::DispatchSide::Instance)
        .expect("next requirement")
        .requirement
        .clone();
    assert_eq!(
        first_evidence
            .requirement_views
            .get(&next_requirement)
            .expect("initial next view")
            .signature
            .declared_return
            .canonical_type(),
        Some(first.snapshot.declarations.form(&CoreDeclarationIds::default().int).expect("Int form")),
    );

    let second = incremental.update(single_module_input(module.clone(), &source_b, 2));
    assert!(!second.snapshot.has_errors(), "replacement diagnostics: {:?}", second.snapshot.diagnostics);
    let second_plan = second.snapshot.conformance_witness_plans.get(&impl_id(&module)).expect("replaced witness plan");
    assert_ne!(first_binding_fingerprint, second_plan.associated_type_plan.fingerprint, "RHS edit must change binding plan");
    assert!(second_plan.associated_type_plan.failures.is_empty());
    let second_box = second.snapshot.declarations.form(&box_declaration).expect("replaced Box form");
    let second_evidence = second.snapshot.conformance_evidence_for(second_box, &trait_ref).expect("replaced exact evidence");
    assert_ne!(first_evidence.fingerprint, second_evidence.fingerprint, "RHS edit must change normalized exact evidence");
    assert_eq!(
        second_evidence
            .requirement_views
            .get(&next_requirement)
            .expect("replaced next view")
            .signature
            .declared_return
            .canonical_type(),
        Some(second.snapshot.declarations.form(&CoreDeclarationIds::default().string).expect("String form")),
    );

    let third = incremental.update(single_module_input(module.clone(), &source_c, 3));
    assert!(third.snapshot.conformance_evidence_for(
        third.snapshot.declarations.form(&box_declaration).expect("retained Box form"),
        &trait_ref,
    ).is_none(), "removed binding must remove exact evidence");
    let third_plan = third.snapshot.conformance_witness_plans.get(&impl_id(&module)).expect("retained witness plan");
    assert!(third_plan.associated_type_plan.bindings.is_empty(), "removed binding must not remain in the plan");
    assert!(third_plan.associated_type_plan.failures.iter().any(|failure| failure.reason.contains("no conformance binding")));

    let fourth = incremental.update(single_module_input(module.clone(), &source_b, 4));
    let fourth_plan = fourth.snapshot.conformance_witness_plans.get(&impl_id(&module)).expect("re-added witness plan");
    assert!(fourth_plan.associated_type_plan.failures.is_empty());
    let fourth_evidence = fourth.snapshot.conformance_evidence_for(
        fourth.snapshot.declarations.form(&box_declaration).expect("re-added Box form"),
        &trait_ref,
    ).expect("re-added exact evidence");

    let mut cold = SemanticWorkspaceSession::new();
    let cold_result = cold.update(single_module_input(module.clone(), &source_b, 1));
    let cold_plan = cold_result.snapshot.conformance_witness_plans.get(&impl_id(&module)).expect("cold witness plan");
    assert_eq!(fourth_plan.associated_type_plan, cold_plan.associated_type_plan, "re-added binding must match cold analysis");
    assert_eq!(fourth_plan.completeness, cold_plan.completeness, "re-added completeness must match cold analysis");
    let cold_evidence = cold_result.snapshot.conformance_evidence_for(
        cold_result.snapshot.declarations.form(&box_declaration).expect("cold Box form"),
        &trait_ref,
    ).expect("cold exact evidence");
    assert_eq!(fourth_evidence.fingerprint, cold_evidence.fingerprint, "re-added normalized evidence must match cold analysis");
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
fn projection_bearing_trait_method_addition_and_removal_updates_exact_products() {
    let module = module();
    let source_without_method = r#"
trait Iterable {
  type Item
}
class Box {}
impl Iterable for Box { type Item = Int }
"#;
    let source_with_method = r#"
trait Iterable {
  type Item
  next -> Self::Item
}
class Box {}
impl Iterable for Box {
  type Item = Int
  next -> Int { 0 }
}
"#;
    let trait_declaration = DeclarationId::new(module.clone(), "Iterable".into());
    let box_declaration = DeclarationId::new(module.clone(), "Box".into());
    let trait_ref = phalcom_semantic::traits::TraitRef::new(trait_declaration.clone(), Vec::new().into_boxed_slice());
    let mut session = SemanticWorkspaceSession::new();

    let first = session.update(single_module_input(module.clone(), source_without_method, 1));
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    let first_box = first.snapshot.declarations.form(&box_declaration).expect("initial Box form");
    let first_evidence = first.snapshot.conformance_evidence_for(first_box, &trait_ref).expect("initial evidence");
    let first_surface_fingerprint = trait_surface_product_fingerprint(first.snapshot.trait_surfaces.get(&trait_declaration).expect("initial surface"));
    assert!(first_evidence.requirement_views.is_empty(), "the initial trait has no behavioral requirement");

    let second = session.update(single_module_input(module.clone(), source_with_method, 2));
    assert!(!second.snapshot.has_errors(), "method addition diagnostics: {:?}", second.snapshot.diagnostics);
    let second_box = second.snapshot.declarations.form(&box_declaration).expect("updated Box form");
    let second_evidence = second.snapshot.conformance_evidence_for(second_box, &trait_ref).expect("updated evidence");
    assert_ne!(
        first_surface_fingerprint,
        trait_surface_product_fingerprint(second.snapshot.trait_surfaces.get(&trait_declaration).expect("updated surface")),
        "adding a projection-bearing method must update the trait surface product"
    );
    let next = second
        .snapshot
        .trait_surfaces
        .get(&trait_declaration)
        .expect("updated surface")
        .get_by_selector(&Selector::getter("next").expect("next selector"), phalcom_semantic::identity::DispatchSide::Instance)
        .expect("next requirement")
        .requirement
        .clone();
    assert!(second_evidence.requirement_views.contains_key(&next), "exact evidence must publish the added method");

    let third = session.update(single_module_input(module.clone(), source_without_method, 3));
    assert!(!third.snapshot.has_errors(), "method removal diagnostics: {:?}", third.snapshot.diagnostics);
    let third_box = third.snapshot.declarations.form(&box_declaration).expect("removed-method Box form");
    let third_evidence = third.snapshot.conformance_evidence_for(third_box, &trait_ref).expect("removed-method evidence");
    assert!(third_evidence.requirement_views.is_empty(), "removing the projection-bearing method must retire its evidence view");
    assert!(third.snapshot.trait_surfaces.get(&trait_declaration).expect("removed-method surface").get_by_selector(
        &Selector::getter("next").expect("next selector"),
        phalcom_semantic::identity::DispatchSide::Instance,
    ).is_none());

    let mut cold = SemanticWorkspaceSession::new();
    let cold_result = cold.update(single_module_input(module.clone(), source_without_method, 1));
    let cold_box = cold_result.snapshot.declarations.form(&box_declaration).expect("cold Box form");
    let cold_evidence = cold_result.snapshot.conformance_evidence_for(cold_box, &trait_ref).expect("cold evidence");
    assert_eq!(third_evidence.fingerprint, cold_evidence.fingerprint, "method removal must converge with cold evidence");
}

#[test]
fn associated_type_rename_republishes_projection_occurrences_with_canonical_identity() {
    let module = module();
    let source_a = r#"
trait Iterable {
  type Item
  next -> Self::Item
}
class Box {}
impl Iterable for Box {
  type Item = Int
  next -> Int { 0 }
}
"#;
    let source_b = source_a.replace("Item", "Element");
    let declaration = DeclarationId::new(module.clone(), "Iterable".into());
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(module.clone(), &source_a, 1));
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    let old_requirement = first
        .snapshot
        .trait_surfaces
        .get(&declaration)
        .expect("initial surface")
        .associated_type_by_name("Item")
        .expect("initial Item")
        .requirement
        .clone();
    let second = session.update(single_module_input(module.clone(), &source_b, 2));
    assert!(!second.snapshot.has_errors(), "rename diagnostics: {:?}", second.snapshot.diagnostics);
    let surface = second.snapshot.trait_surfaces.get(&declaration).expect("renamed surface");
    assert!(surface.associated_type_by_name("Item").is_none(), "old associated declaration must not remain");
    let new_requirement = surface.associated_type_by_name("Element").expect("renamed associated declaration").requirement.clone();
    assert_eq!(old_requirement, new_requirement, "owner-relative requirement identity remains stable for an in-place rename");
    let target = SemanticTargetId::AssociatedType(new_requirement);
    let declaration_offset = source_b.find("type Element").expect("renamed declaration") + "type ".len();
    let binding_offset = source_b.rfind("type Element").expect("renamed binding") + "type ".len();
    let projection_offset = source_b.find("Self::Element").expect("renamed projection") + "Self::".len();
    assert_eq!(second.snapshot.editor().target_at(&module, declaration_offset), Some(target.clone()));
    assert_eq!(second.snapshot.editor().target_at(&module, binding_offset), Some(target.clone()));
    assert_eq!(second.snapshot.editor().target_at(&module, projection_offset), Some(target));
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
