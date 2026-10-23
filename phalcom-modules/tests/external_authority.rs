use phalcom_compiler_artifact::digest::ArtifactDigest;
use phalcom_modules::{
    CanonicalExternalModuleWorld, ExternalModuleAuthority, ExternalModuleAuthoritySet, InterfaceBuilder, ModuleId, ModuleKind, ModuleLinker,
    ModuleLoadError, ModulePath, ModuleResolutionError, ModuleResolver, ModuleTopology, NullDependencyProvider, ProjectIdentity, ProjectUniverse,
    ResolvedProject, ResolverGeneration, SourceError, SourceId, SourceLocation, SourceProvider, SourceRevision, SourceUnit,
    ValidatedPackageAuthorityReceipt, WorkspaceModuleSession,
};
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::StableProjectRef;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

fn project(root: &Path, name: &str, dependencies: &str) -> std::path::PathBuf {
    let root = root.join(name);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("project.toml"), format!("[project]\nname = \"{name}\"\nnamespace = \"{name}\"\nversion = \"1.0.0\"\n{dependencies}")).unwrap();
    fs::write(root.join("src/package.ph"), "").unwrap();
    root
}

fn world(projects: Arc<ProjectUniverse>, module: ModuleId) -> Arc<CanonicalExternalModuleWorld> {
    let parsed = phalcom_ast::parse("class Model<T> {}\nexport Model\n", 0);
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let interface = InterfaceBuilder::build(module.clone(), ModuleKind::Package, &parsed.program).unwrap();
    let interfaces = BTreeMap::from([(module.clone(), interface)]);
    let topology = Arc::new(ModuleTopology::from_parts(ResolverGeneration(0), &projects, &interfaces, &BTreeMap::new()));
    let linked = ModuleLinker::new(projects, interfaces.clone()).link(module, &BTreeMap::new()).unwrap();
    Arc::new(CanonicalExternalModuleWorld::from_parts(Arc::new(linked), interfaces.into_iter().map(|(id, interface)| (id, Arc::new(interface))).collect::<BTreeMap<_, _>>(), topology).unwrap())
}

struct DeniedSource;
impl SourceProvider for DeniedSource {
    fn locate(&self, _: &ResolvedProject, _: &ModulePath) -> Result<SourceUnit, ModuleResolutionError> { panic!("artifact lookup tried to locate source") }
    fn read(&self, _: &SourceId) -> Result<Arc<str>, SourceError> { panic!("artifact lookup tried to read source") }
}

#[test]
fn multiple_package_authorities_resolve_imports_without_source() {
    let temp = TempDir::new().unwrap();
    let a = project(temp.path(), "dep_a", "");
    let b = project(temp.path(), "dep_b", "");
    let consumer = project(temp.path(), "consumer", "[dependencies]\na = { path = \"../dep_a\" }\nb = { path = \"../dep_b\" }\n");
    let mut projects = ProjectUniverse::new();
    let consumer_id = projects.load_root(consumer.join("project.toml")).unwrap();
    let deps = &projects.get_project(consumer_id).unwrap().dependencies;
    let a_id = ModuleId::resolved(deps[&phalcom_modules::ModuleComponent::from_identifier("a").unwrap()], ModulePath::root());
    let b_id = ModuleId::resolved(deps[&phalcom_modules::ModuleComponent::from_identifier("b").unwrap()], ModulePath::root());
    let projects = Arc::new(projects);
    let authority = Arc::new(ExternalModuleAuthoritySet::from_worlds(projects.clone(), vec![world(projects.clone(), a_id.clone()), world(projects.clone(), b_id.clone())]).unwrap());
    assert_ne!(authority.origin(&a_id), authority.origin(&b_id));
    fs::remove_dir_all(a.join("src")).unwrap();
    fs::remove_dir_all(b.join("src")).unwrap();
    let mut resolver = ModuleResolver::with_external_modules(&projects, &DeniedSource, authority.clone());
    let importer = ModuleId::resolved(consumer_id, ModulePath::root());
    for (path, expected) in [("a", &a_id), ("b", &b_id)] {
        let parsed = phalcom_ast::parse(&format!("from {path} import Model\n"), 0);
        let interface = InterfaceBuilder::build(importer.clone(), ModuleKind::Package, &parsed.program).unwrap();
        let target = resolver.resolve_import(&importer, interface.imports[0].path()).unwrap();
        assert_eq!(&target.id, expected);
        assert!(resolver.load_interface(expected).unwrap().exports.contains_key("Model"));
        assert!(matches!(resolver.load_parsed(expected), Err(ModuleLoadError::Resolution(ModuleResolutionError::Source(SourceError::CapabilityUnavailable { module }))) if module == *expected));
    }
    let local = InterfaceBuilder::build(importer.clone(), ModuleKind::Package, &phalcom_ast::parse("from a import Model\n", 0).program).unwrap();
    let linked = ModuleLinker::new(projects, BTreeMap::from([(importer.clone(), local)]))
        .with_external_modules(authority)
        .link(importer.clone(), &BTreeMap::from([((importer.clone(), "a".into()), a_id)]))
        .unwrap();
    assert_eq!(linked.modules.len(), 1, "external modules must remain immutable linking leaves");
}

#[test]
fn missing_module_in_artifact_project_never_falls_back_to_source() {
    let temp = TempDir::new().unwrap();
    let dep = project(temp.path(), "dep", "");
    let mut projects = ProjectUniverse::new();
    let id = projects.load_root(dep.join("project.toml")).unwrap();
    let projects = Arc::new(projects);
    let authority = world(projects.clone(), ModuleId::resolved(id, ModulePath::root()));
    let mut resolver = ModuleResolver::with_external_modules(&projects, &DeniedSource, authority);
    let missing = ModuleId::resolved(id, ModulePath::from_components(vec![phalcom_modules::ModuleComponent::from_identifier("missing").unwrap()]));
    assert!(resolver.load_interface(&missing).is_err());
    assert!(matches!(resolver.load_parsed(&missing), Err(ModuleLoadError::Resolution(ModuleResolutionError::Source(SourceError::CapabilityUnavailable { .. })))));
}

#[test]
fn authority_composition_rejects_collisions_and_rebound_project_ids() {
    let temp = TempDir::new().unwrap();
    let dep = project(temp.path(), "dep", "");
    let other = project(temp.path(), "other", "");
    let mut projects = ProjectUniverse::new();
    let id = projects.load_root(dep.join("project.toml")).unwrap();
    let projects = Arc::new(projects);
    let world = world(projects.clone(), ModuleId::resolved(id, ModulePath::root()));
    assert!(ExternalModuleAuthoritySet::from_worlds(projects, vec![world.clone(), world.clone()]).is_err());
    let mut rebound = ProjectUniverse::new();
    assert_eq!(rebound.load_root(other.join("project.toml")).unwrap(), id);
    assert!(ExternalModuleAuthoritySet::from_worlds(Arc::new(rebound), vec![world]).is_err());
}

#[test]
fn artifact_project_receipts_survive_root_updates_and_source_edits() {
    let temp = TempDir::new().unwrap();
    let dep = project(temp.path(), "dep", "");
    let consumer = project(temp.path(), "consumer", "");
    let mut projects = ProjectUniverse::new();
    let dep_id = projects.load_root(dep.join("project.toml")).unwrap();
    let stable = StableProjectRef::Package { package: "dep".into(), version: "1.0.0".into(), artifact_fingerprint: Fingerprint128([7; 16]) };
    let digest = ArtifactDigest([9; 32]);
    projects.register_package_authority_receipt(&stable, ValidatedPackageAuthorityReceipt::from_trusted_resolver(stable.clone(), digest, dep_id).unwrap(), Some(digest)).unwrap();
    let projects = Arc::new(projects);
    let external = world(projects.clone(), ModuleId::resolved(dep_id, ModulePath::root()));
    let mut session = WorkspaceModuleSession::with_external_modules(external.clone());
    session.set_workspace_roots(&[consumer.clone()], &NullDependencyProvider).unwrap();
    let consumer_id = session.universe().projects().iter().find(|p| p.namespace.as_str() == "consumer").unwrap().id;
    let main = consumer.join("src/main.ph");
    let source = SourceLocation { source_id: SourceId(main.to_string_lossy().into()), display_path: main };
    session.set_overlay(source.clone(), Arc::from("class Local {}\n"), SourceRevision(1)).unwrap();
    let local_module = session.module_for_source(&source.source_id).unwrap().clone();
    assert_eq!(local_module.project, ProjectIdentity::Resolved(consumer_id));
    // The dependency source disappears, but the manifest boundary remains known.
    fs::remove_dir_all(dep.join("src")).unwrap();
    fs::write(consumer.join("project.toml"), "[project]\nname = \"consumer\"\nnamespace = \"updated_consumer\"\nversion = \"2.0.0\"\n").unwrap();
    session.set_workspace_roots(&[consumer], &NullDependencyProvider).unwrap();
    assert_eq!(session.universe().resolve_stable_project(&stable), Some(ProjectIdentity::Resolved(dep_id)));
    assert_eq!(session.universe().get_project(consumer_id).unwrap().namespace.as_str(), "updated_consumer");
    assert_eq!(session.module_for_source(&source.source_id), Some(&local_module));
    session.set_overlay(source, Arc::from("class Changed {}\n"), SourceRevision(2)).unwrap();
    assert_eq!(session.source(&local_module).unwrap().revision, SourceRevision(2));
    let generation = session.generation();
    let dep_source = SourceLocation { source_id: SourceId(dep.join("src/package.ph").to_string_lossy().into()), display_path: dep.join("src/package.ph") };
    assert!(session.set_overlay(dep_source, Arc::from("class Shadow {}\n"), SourceRevision(3)).is_err());
    assert_eq!(session.generation(), generation, "rejected artifact overlay must not publish");
    assert!(external.interface(&ModuleId::resolved(dep_id, ModulePath::root())).unwrap().exports.contains_key("Model"));
}


#[test]
fn trusted_package_receipts_are_bijective_over_native_project_ids() {
    let temp = TempDir::new().unwrap();
    let dep = project(temp.path(), "artifact_owner", "");
    let mut projects = ProjectUniverse::new();
    let id = projects.load_root(dep.join("project.toml")).unwrap();
    let first = StableProjectRef::Package {
        package: "artifact_owner".into(),
        version: "1.0.0".into(),
        artifact_fingerprint: Fingerprint128([10; 16]),
    };
    let conflicting_fingerprint = StableProjectRef::Package {
        package: "artifact_owner".into(),
        version: "1.0.0".into(),
        artifact_fingerprint: Fingerprint128([11; 16]),
    };
    let conflicting_package = StableProjectRef::Package {
        package: "another_package".into(),
        version: "1.0.0".into(),
        artifact_fingerprint: Fingerprint128([12; 16]),
    };
    let digest = ArtifactDigest([13; 32]);

    let receipt = ValidatedPackageAuthorityReceipt::from_trusted_resolver(first.clone(), digest, id).unwrap();
    projects.register_package_authority_receipt(&first, receipt.clone(), Some(digest)).unwrap();
    assert_eq!(projects.stable_package_owner(id), Some(&first));
    // Idempotent replay is permitted, but replacing the owner is not.
    projects.register_package_authority_receipt(&first, receipt, Some(digest)).unwrap();
    assert_eq!(projects.stable_package_owner(id), Some(&first));
    for conflicting in [conflicting_fingerprint, conflicting_package] {
        let forged = ValidatedPackageAuthorityReceipt::from_trusted_resolver(conflicting.clone(), digest, id).unwrap();
        assert!(matches!(
            projects.register_package_authority_receipt(&conflicting, forged, Some(digest)),
            Err(phalcom_modules::PackageAuthorityRegistrationError::ConflictingBinding)
        ));
        assert!(projects.resolve_stable_project(&conflicting).is_none());
        assert_eq!(projects.stable_package_owner(id), Some(&first));
    }
}
