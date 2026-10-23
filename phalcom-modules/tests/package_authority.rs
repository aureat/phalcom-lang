use phalcom_compiler_artifact::digest::ArtifactDigest;
use phalcom_compiler_artifact::identity::TeloscPackageIdentity;
use phalcom_compiler_artifact::module_world::{ArtifactModuleId, ModuleWorldV2};
use phalcom_modules::{
    ProjectIdentity, ProjectUniverse, ResolvedProjectId, TrustedPackageResolver, ValidatedPackageAuthorityReceipt, stage_artifact_project_authorities,
};
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::StableProjectRef;
use std::fs;
use tempfile::TempDir;

fn project(root: &std::path::Path, directory: &str, name: &str) -> std::path::PathBuf {
    let project_root = root.join(directory);
    fs::create_dir_all(project_root.join("src")).expect("create source root");
    let namespace = name.replace('-', "_");
    fs::write(
        project_root.join("project.toml"),
        format!("[project]\nname = \"{name}\"\nnamespace = \"{namespace}\"\nversion = \"1.0.0\"\n"),
    )
    .expect("write manifest");
    fs::write(project_root.join("src/package.ph"), "").expect("write package root");
    project_root.join("project.toml")
}

fn package_ref(fingerprint: u8) -> StableProjectRef {
    StableProjectRef::Package {
        package: "shared-package".into(),
        version: "2.4.1".into(),
        artifact_fingerprint: Fingerprint128([fingerprint; 16]),
    }
}

struct TestResolver(Option<ValidatedPackageAuthorityReceipt>);

impl TrustedPackageResolver for TestResolver {
    fn package_authority_receipt(&self, stable_project: &StableProjectRef) -> Option<ValidatedPackageAuthorityReceipt> {
        self.0.as_ref().filter(|receipt| receipt.stable_project() == stable_project).cloned()
    }
}

fn package_module_world(package: StableProjectRef) -> phalcom_compiler_artifact::module_world_v5::ModuleWorldV5 {
    let mut world = ModuleWorldV2 {
        entry: ArtifactModuleId(0),
        initialization_order: Vec::new(),
        modules: Vec::new(),
        semantic_rows: Vec::new(),
        external_projects: vec![package.clone()],
        topology_fingerprint: 0,
    };
    world.refresh_portable_topology_fingerprint().unwrap();
    let mut world: phalcom_compiler_artifact::module_world_v5::ModuleWorldV5 = world.into();
    // Staging requires the entry to be the package's own root module.
    world.modules.push(phalcom_compiler_artifact::module_world_v5::ModuleV3 {
        path: vec![],
        identity: phalcom_type_meta::identity::StableModuleRef {
            project: package,
            path: Box::new([]),
        },
        kind: phalcom_compiler_artifact::module_world::ModuleKindV1::Package,
        declarations: vec![],
        exports: vec![],
        imports: vec![],
        exposed_children: vec![],
        metadata: vec![],
        linked_exports: vec![],
        globals: vec![],
        import_bindings: vec![],
        linked_reads: vec![],
        runtime_dependencies: vec![],
        references: vec![],
        runtime_edges: vec![],
        topology_parent: None,
        topology_children: vec![],
        interface_fingerprint: 0,
        linked_interface_fingerprint: 0,
        dependency_fingerprint: 0,
    });
    world
}

#[test]
fn package_receipts_bind_exact_identity_digest_and_existing_project() {
    let temp = TempDir::new().expect("create temp directory");
    let manifest = project(temp.path(), "project", "host");
    let mut universe = ProjectUniverse::new();
    let project_id = universe.load_root(manifest).expect("load persistent project");
    let package = package_ref(7);
    let digest = ArtifactDigest([9; 32]);
    let receipt = ValidatedPackageAuthorityReceipt::from_trusted_resolver(package.clone(), digest, project_id).expect("package receipt");

    assert_eq!(
        universe.register_package_authority_receipt(&package, receipt.clone(), Some(digest)),
        Ok(project_id)
    );
    assert_eq!(
        universe.resolve_stable_project(&package),
        Some(ProjectIdentity::Resolved(project_id)),
        "exact package tuple resolves to its existing project"
    );
    assert_eq!(universe.projects().len(), 1, "receipt registration must not allocate a project ID");
    assert_eq!(universe.get_project(project_id).map(|project| project.id), Some(ResolvedProjectId::from_raw(1)));

    assert_eq!(
        universe.resolve_stable_project(&package_ref(8)),
        None,
        "matching package and version with a different fingerprint must fail"
    );
}

#[test]
fn package_receipt_rejects_digest_target_and_mapping_conflicts() {
    let temp = TempDir::new().expect("create temp directory");
    let manifest_a = project(temp.path(), "a", "host-a");
    let manifest_b = project(temp.path(), "b", "host-b");
    let mut universe = ProjectUniverse::new();
    let project_a = universe.load_root(manifest_a).expect("load first project");
    let project_b = universe.load_root(manifest_b).expect("load second project");
    let package = package_ref(3);
    let digest = ArtifactDigest([4; 32]);

    let wrong_digest = ValidatedPackageAuthorityReceipt::from_trusted_resolver(package.clone(), digest, project_a).unwrap();
    assert!(matches!(
        universe.register_package_authority_receipt(&package, wrong_digest, Some(ArtifactDigest([5; 32]))),
        Err(phalcom_modules::PackageAuthorityRegistrationError::ArtifactDigestMismatch)
    ));
    assert_eq!(universe.resolve_stable_project(&package), None, "failed validation must not register a binding");

    let stale = ValidatedPackageAuthorityReceipt::from_trusted_resolver(package.clone(), digest, ResolvedProjectId::from_raw(99)).unwrap();
    assert!(matches!(
        universe.register_package_authority_receipt(&package, stale, None),
        Err(phalcom_modules::PackageAuthorityRegistrationError::UnknownProject)
    ));

    let first = ValidatedPackageAuthorityReceipt::from_trusted_resolver(package.clone(), digest, project_a).unwrap();
    universe.register_package_authority_receipt(&package, first, Some(digest)).unwrap();
    let conflicting = ValidatedPackageAuthorityReceipt::from_trusted_resolver(package.clone(), digest, project_b).unwrap();
    assert!(matches!(
        universe.register_package_authority_receipt(&package, conflicting, Some(digest)),
        Err(phalcom_modules::PackageAuthorityRegistrationError::ConflictingBinding)
    ));
    assert_eq!(
        universe.resolve_stable_project(&package),
        Some(ProjectIdentity::Resolved(project_a)),
        "a conflicting receipt leaves the existing authority unchanged"
    );
}

#[test]
fn package_receipt_rejects_nonpersistent_project_target() {
    let temp = TempDir::new().expect("create temp directory");
    let standalone = temp.path().join("standalone");
    fs::create_dir_all(&standalone).expect("create standalone root");
    fs::write(standalone.join("package.ph"), "").expect("write package root");

    let mut universe = ProjectUniverse::new();
    let project_id = universe.load_standalone_package(&standalone, None).expect("load standalone package");
    let package = package_ref(2);
    let receipt = ValidatedPackageAuthorityReceipt::from_trusted_resolver(package.clone(), ArtifactDigest([6; 32]), project_id).unwrap();

    assert!(matches!(
        universe.register_package_authority_receipt(&package, receipt, None),
        Err(phalcom_modules::PackageAuthorityRegistrationError::NonPersistentProject)
    ));
    assert_eq!(universe.resolve_stable_project(&package), None);
}

#[test]
fn artifact_package_authority_is_staged_from_resolver_and_validated_bytes() {
    let temp = TempDir::new().expect("create temp directory");
    let manifest = project(temp.path(), "project", "host");
    let mut universe = ProjectUniverse::new();
    let project_id = universe.load_root(manifest).expect("load persistent project");
    let package = package_ref(5);
    let digest = ArtifactDigest([0x31; 32]);
    let receipt = ValidatedPackageAuthorityReceipt::from_trusted_resolver(package.clone(), digest, project_id).unwrap();
    let resolver = TestResolver(Some(receipt));
    let candidate = stage_artifact_project_authorities(
        &universe,
        &package_module_world(package.clone()),
        &TeloscPackageIdentity::Package {
            package: "shared-package".into(),
            version: "2.4.1".into(),
        },
        digest,
        &resolver,
    )
    .expect("trusted receipt should produce a private catalog candidate");

    assert_eq!(universe.resolve_stable_project(&package), None, "staging must not mutate the caller's graph");
    assert_eq!(candidate.resolve_stable_project(&package), Some(ProjectIdentity::Resolved(project_id)));
    assert_eq!(
        candidate.projects().len(),
        universe.projects().len(),
        "receipt registration must not allocate a project ID"
    );
}

#[test]
fn artifact_package_authority_fails_closed_for_missing_receipt_or_wrong_digest() {
    let temp = TempDir::new().expect("create temp directory");
    let manifest = project(temp.path(), "project", "host");
    let mut universe = ProjectUniverse::new();
    let project_id = universe.load_root(manifest).expect("load persistent project");
    let package = package_ref(6);
    let digest = ArtifactDigest([0x41; 32]);
    let wrong_receipt = ValidatedPackageAuthorityReceipt::from_trusted_resolver(package.clone(), ArtifactDigest([0x42; 32]), project_id).unwrap();
    let identity = TeloscPackageIdentity::Package {
        package: "shared-package".into(),
        version: "2.4.1".into(),
    };

    assert!(stage_artifact_project_authorities(&universe, &package_module_world(package.clone()), &identity, digest, &TestResolver(None),).is_err());
    assert!(
        stage_artifact_project_authorities(
            &universe,
            &package_module_world(package.clone()),
            &identity,
            digest,
            &TestResolver(Some(wrong_receipt)),
        )
        .is_err()
    );
    assert_eq!(
        universe.resolve_stable_project(&package),
        None,
        "failed staging leaves the caller's graph unchanged"
    );
}

#[test]
fn artifact_header_must_match_one_exact_package_tuple() {
    let temp = TempDir::new().expect("create temp directory");
    let manifest = project(temp.path(), "project", "host");
    let mut universe = ProjectUniverse::new();
    let project_id = universe.load_root(manifest).expect("load persistent project");
    let package = package_ref(2);
    let receipt = ValidatedPackageAuthorityReceipt::from_trusted_resolver(package.clone(), ArtifactDigest([3; 32]), project_id).unwrap();
    let resolver = TestResolver(Some(receipt));
    let wrong_header = TeloscPackageIdentity::Package {
        package: "shared-package".into(),
        version: "2.4.2".into(),
    };

    assert!(
        stage_artifact_project_authorities(
            &universe,
            &package_module_world(package.clone()),
            &wrong_header,
            ArtifactDigest([3; 32]),
            &resolver,
        )
        .is_err()
    );
}
