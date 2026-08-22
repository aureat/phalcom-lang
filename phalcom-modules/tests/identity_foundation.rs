use phalcom_modules::{ModuleComponent, ModulePath, ProjectRevisionFingerprint, ProjectSourceIdentity, StableModuleKey, StableProjectKey};
use std::path::PathBuf;

#[test]
fn stable_module_key_is_source_backed_and_module_qualified() {
    let project = StableProjectKey::from_path("/workspace/app");
    let path = ModulePath::from_components(vec![ModuleComponent::from_identifier("geometry").unwrap()]);
    let key = StableModuleKey::new(project.clone(), path.clone());

    assert_eq!(key.project, project);
    assert_eq!(key.path, path);
    assert_ne!(
        key,
        StableModuleKey::new(
            StableProjectKey::from_path("/workspace/other"),
            ModulePath::from_components(vec![ModuleComponent::from_identifier("geometry").unwrap()]),
        )
    );
}

#[test]
fn stable_project_key_preserves_source_identity() {
    let source = ProjectSourceIdentity::from_path(PathBuf::from("/workspace/app"));
    let key = StableProjectKey::from_source(source.clone());

    assert_eq!(key.source, source);
    assert_eq!(StableProjectKey::from_path("/workspace/app"), key);
}

#[test]
fn project_revision_fingerprint_is_copyable_and_byte_addressable() {
    let fingerprint = ProjectRevisionFingerprint::from_bytes([7; 16]);

    assert_eq!(fingerprint.as_bytes(), &[7; 16]);
    assert_eq!(fingerprint, fingerprint);
}
