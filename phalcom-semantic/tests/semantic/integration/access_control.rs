use phalcom_ast::parse;
use phalcom_common::modifier::Visibility;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ProjectIdentity, ResolvedProjectId};
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::editor::{AccessContext, is_member_accessible};
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::relation::MapTypeHierarchy;
use std::sync::Arc;

fn comp(name: &str) -> ModuleComponent {
    ModuleComponent::from_identifier(name).expect("valid module component")
}

#[test]
fn is_member_accessible_public() {
    let hierarchy = MapTypeHierarchy::default();
    let mod_a = ModuleId::universe(ModulePath::root());
    let decl_owner = DeclarationId::new(mod_a.clone(), "Target".into());
    let access = AccessContext::new(None, None);

    assert!(is_member_accessible(Visibility::Public, &decl_owner, None, &access, &hierarchy));
}

#[test]
fn is_member_accessible_private() {
    let hierarchy = MapTypeHierarchy::default();
    let mod_a = ModuleId::universe(ModulePath::root());
    let decl_owner = DeclarationId::new(mod_a.clone(), "Target".into());
    let other_decl = DeclarationId::new(mod_a.clone(), "Other".into());

    let same_access = AccessContext::new(Some(decl_owner.clone()), None);
    let other_access = AccessContext::new(Some(other_decl), None);
    let anon_access = AccessContext::new(None, None);

    assert!(is_member_accessible(Visibility::Private, &decl_owner, None, &same_access, &hierarchy));
    assert!(!is_member_accessible(Visibility::Private, &decl_owner, None, &other_access, &hierarchy));
    assert!(!is_member_accessible(Visibility::Private, &decl_owner, None, &anon_access, &hierarchy));
}

#[test]
fn is_member_accessible_protected() {
    let mut hierarchy = MapTypeHierarchy::default();
    let mod_a = ModuleId::universe(ModulePath::root());
    let base_decl = DeclarationId::new(mod_a.clone(), "Base".into());
    let sub_decl = DeclarationId::new(mod_a.clone(), "Sub".into());
    let unrelated_decl = DeclarationId::new(mod_a.clone(), "Unrelated".into());

    hierarchy.insert(sub_decl.clone(), base_decl.clone());

    let base_access = AccessContext::new(Some(base_decl.clone()), None);
    let sub_access = AccessContext::new(Some(sub_decl.clone()), None);
    let unrelated_access = AccessContext::new(Some(unrelated_decl), None);

    assert!(is_member_accessible(Visibility::Protected, &base_decl, None, &base_access, &hierarchy));
    assert!(is_member_accessible(Visibility::Protected, &base_decl, None, &sub_access, &hierarchy));
    assert!(!is_member_accessible(Visibility::Protected, &base_decl, None, &unrelated_access, &hierarchy));
}

#[test]
fn is_member_accessible_module() {
    let hierarchy = MapTypeHierarchy::default();
    let mod_a = ModuleId::universe(ModulePath::from_components(vec![comp("pkg"), comp("mod_a")]));
    let mod_b = ModuleId::universe(ModulePath::from_components(vec![comp("pkg"), comp("mod_b")]));

    let decl_target = DeclarationId::new(mod_a.clone(), "Target".into());
    let decl_same_mod = DeclarationId::new(mod_a.clone(), "OtherInSameMod".into());
    let decl_other_mod = DeclarationId::new(mod_b.clone(), "InOtherMod".into());

    let same_mod_access = AccessContext::with_provenance(Some(decl_same_mod), None, Some(mod_a.clone()), None, Some(mod_a.project));
    let other_mod_access = AccessContext::with_provenance(Some(decl_other_mod), None, Some(mod_b.clone()), None, Some(mod_b.project));

    assert!(is_member_accessible(Visibility::Module, &decl_target, None, &same_mod_access, &hierarchy));
    assert!(!is_member_accessible(Visibility::Module, &decl_target, None, &other_mod_access, &hierarchy));
}

#[test]
fn is_member_accessible_package() {
    let hierarchy = MapTypeHierarchy::default();
    let pkg_root = ModuleId::universe(ModulePath::from_components(vec![comp("pkg")]));
    let sub_pkg = ModuleId::universe(ModulePath::from_components(vec![comp("pkg"), comp("nested")]));
    let sibling_pkg = ModuleId::universe(ModulePath::from_components(vec![comp("other_pkg")]));

    let mod_a = ModuleId::universe(ModulePath::from_components(vec![comp("pkg"), comp("mod_a")]));
    let mod_b = ModuleId::universe(ModulePath::from_components(vec![comp("pkg"), comp("mod_b")]));
    let mod_nested = ModuleId::universe(ModulePath::from_components(vec![comp("pkg"), comp("nested"), comp("deep")]));

    let decl_target = DeclarationId::new(mod_a.clone(), "Target".into());

    // Sibling module in same package
    let same_pkg_access = AccessContext::with_provenance(None, None, Some(mod_b.clone()), Some(pkg_root.clone()), Some(mod_b.project));
    // Nested package module (different package)
    let nested_pkg_access = AccessContext::with_provenance(None, None, Some(mod_nested.clone()), Some(sub_pkg.clone()), Some(mod_nested.project));
    // Sibling package module (different package)
    let sibling_pkg_access = AccessContext::with_provenance(None, None, Some(sibling_pkg.clone()), Some(sibling_pkg.clone()), Some(sibling_pkg.project));

    assert!(is_member_accessible(
        Visibility::Package,
        &decl_target,
        Some(&pkg_root),
        &same_pkg_access,
        &hierarchy
    ));
    assert!(!is_member_accessible(
        Visibility::Package,
        &decl_target,
        Some(&pkg_root),
        &nested_pkg_access,
        &hierarchy
    ));
    assert!(!is_member_accessible(
        Visibility::Package,
        &decl_target,
        Some(&pkg_root),
        &sibling_pkg_access,
        &hierarchy
    ));
}

#[test]
fn package_access_requires_canonical_package_identity() {
    let hierarchy = MapTypeHierarchy::default();
    let pkg_root = ModuleId::universe(ModulePath::from_components(vec![comp("pkg")]));
    let mod_a = ModuleId::universe(ModulePath::from_components(vec![comp("pkg"), comp("mod_a")]));
    let mod_b = ModuleId::universe(ModulePath::from_components(vec![comp("pkg"), comp("mod_b")]));
    let decl_target = DeclarationId::new(mod_a.clone(), "Target".into());
    let caller = DeclarationId::new(mod_b.clone(), "Caller".into());

    // A caller without the canonical package product must fail closed. The
    // old path-based fallback incorrectly treated these sibling paths as a
    // package match.
    let access = AccessContext::new(Some(caller), None);
    assert!(!is_member_accessible(Visibility::Package, &decl_target, Some(&pkg_root), &access, &hierarchy));
}

#[test]
fn is_member_accessible_internal() {
    let hierarchy = MapTypeHierarchy::default();
    let proj_1 = ProjectIdentity::Resolved(ResolvedProjectId::from_raw(1));
    let proj_2 = ProjectIdentity::Resolved(ResolvedProjectId::from_raw(2));

    let mod_p1_a = ModuleId {
        project: proj_1,
        path: ModulePath::root(),
    };
    let mod_p1_b = ModuleId {
        project: proj_1,
        path: ModulePath::from_components(vec![comp("sub")]),
    };
    let mod_p2 = ModuleId {
        project: proj_2,
        path: ModulePath::root(),
    };

    let decl_target = DeclarationId::new(mod_p1_a.clone(), "Target".into());

    let same_proj_access = AccessContext::with_provenance(None, None, Some(mod_p1_b.clone()), None, Some(proj_1));
    let other_proj_access = AccessContext::with_provenance(None, None, Some(mod_p2.clone()), None, Some(proj_2));

    assert!(is_member_accessible(Visibility::Internal, &decl_target, None, &same_proj_access, &hierarchy));
    assert!(!is_member_accessible(Visibility::Internal, &decl_target, None, &other_proj_access, &hierarchy));
}

#[test]
fn default_field_visibility_is_public_and_modifiers_are_respected() {
    let source = r#"
class Entity {
    _id: Int
    private _secret: String
    module _mod_val: Int
    package _pkg_val: Int
    internal _token: String

    run -> Int { 1 }
    private helper -> Int { 2 }
    module mod_fn -> Int { 3 }
    package pkg_fn -> Int { 4 }
    internal int_fn -> Int { 5 }
}
"#;
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "parser errors: {:?}", parsed.errors);
    let module = ModuleId::universe_root();
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed.program)).expect("semantic test analysis should succeed");
    assert!(!analysis.snapshot.has_errors(), "semantic errors: {:?}", analysis.snapshot.diagnostics);

    let entity_decl = DeclarationId::new(module.clone(), "Entity".into());
    let surface = analysis.snapshot.surfaces.get(&entity_decl).expect("entity surface");

    // Fields
    assert_eq!(surface.instance.field_visibility.get("_id").copied(), Some(Visibility::Public));
    assert_eq!(surface.instance.field_visibility.get("_secret").copied(), Some(Visibility::Private));
    assert_eq!(surface.instance.field_visibility.get("_mod_val").copied(), Some(Visibility::Module));
    assert_eq!(surface.instance.field_visibility.get("_pkg_val").copied(), Some(Visibility::Package));
    assert_eq!(surface.instance.field_visibility.get("_token").copied(), Some(Visibility::Internal));

    // Methods
    let sel_run = phalcom_common::selector::Selector::getter("run").unwrap();
    let sel_helper = phalcom_common::selector::Selector::getter("helper").unwrap();
    let sel_mod_fn = phalcom_common::selector::Selector::getter("mod_fn").unwrap();
    let sel_pkg_fn = phalcom_common::selector::Selector::getter("pkg_fn").unwrap();
    let sel_int_fn = phalcom_common::selector::Selector::getter("int_fn").unwrap();

    assert_eq!(surface.instance.callable_visibility.get(&sel_run).copied(), Some(Visibility::Public));
    assert_eq!(surface.instance.callable_visibility.get(&sel_helper).copied(), Some(Visibility::Private));
    assert_eq!(surface.instance.callable_visibility.get(&sel_mod_fn).copied(), Some(Visibility::Module));
    assert_eq!(surface.instance.callable_visibility.get(&sel_pkg_fn).copied(), Some(Visibility::Package));
    assert_eq!(surface.instance.callable_visibility.get(&sel_int_fn).copied(), Some(Visibility::Internal));
}
