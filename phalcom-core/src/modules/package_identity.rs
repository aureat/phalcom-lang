//! Stable package identity and source provenance inputs used while publishing.

use phalcom_compiler_artifact::codec::BinaryWriter;
use phalcom_compiler_artifact::digest::{ArtifactDigest, DOMAIN_PACKAGE_SOURCE};
use phalcom_modules::{ModuleId, ProjectIdentity, ResolvedProjectId};

/// Computes the package source digest from owned module paths and exact source text.
///
/// The resolved project number and all source locations are excluded. Modules
/// are sorted by their canonical `ModuleId`; only path components and source
/// bytes enter the digest.
pub(super) fn package_source_digest<'a>(owner: ResolvedProjectId, sources: impl IntoIterator<Item = (&'a ModuleId, &'a str)>) -> ArtifactDigest {
    let mut owned = sources
        .into_iter()
        .filter(|(module, _)| module.project == ProjectIdentity::Resolved(owner))
        .collect::<Vec<_>>();
    owned.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));

    let mut writer = BinaryWriter::digest(DOMAIN_PACKAGE_SOURCE);
    writer.write_u32(u32::try_from(owned.len()).expect("package module count fits the artifact wire format"));
    for (module, source) in owned {
        let components = module.path.components();
        writer.write_u32(u32::try_from(components.len()).expect("module path depth fits the artifact wire format"));
        for component in components {
            writer.write_str(component.as_str());
        }
        writer.write_str(source);
    }
    writer.into_digest()
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_compiler_artifact::identity::{PackageIdentityAbi, package_identity_fingerprint};
    use phalcom_type_meta::fingerprint::Fingerprint128;
    use phalcom_type_meta::identity::StableProjectRef;

    fn module(project: u32, components: &[&str]) -> ModuleId {
        let path = phalcom_modules::ModulePath::from_components(
            components
                .iter()
                .map(|component| phalcom_modules::ModuleComponent::from_identifier(component).unwrap())
                .collect::<Vec<_>>(),
        );
        ModuleId::resolved(ResolvedProjectId::from_raw(project), path)
    }

    fn abi() -> PackageIdentityAbi {
        PackageIdentityAbi {
            semantic_model_version: 1,
            stable_identity_schema: 2,
            module_semantic_abi: 3,
            semantic_artifact_abi: 4,
            query_fingerprint_abi: 5,
            native_surface_schema_version: 6,
            native_catalog_fingerprint: [7; 16],
            type_metadata_schema_version: 8,
        }
    }

    #[test]
    fn package_identity_fingerprint_covers_inputs_and_ignores_checkout_identity() {
        let first_module = module(11, &["slotmap", "runtime"]);
        let moved_module = module(29, &["slotmap", "runtime"]);
        let first_sources = [(&first_module, "pub fn run() -> Nil {}")];
        let moved_sources = [(&moved_module, "pub fn run() -> Nil {}")];
        let first_source_digest = package_source_digest(ResolvedProjectId::from_raw(11), first_sources);
        let moved_source_digest = package_source_digest(ResolvedProjectId::from_raw(29), moved_sources);
        assert_eq!(first_source_digest, moved_source_digest);

        let external = StableProjectRef::Package {
            package: "dependency".into(),
            version: "1.2.3".into(),
            artifact_fingerprint: Fingerprint128([9; 16]),
        };
        let externals = [external.clone()];
        let abi = abi();
        let base = package_identity_fingerprint("slotmap", "2.1.0", first_source_digest, &externals, &abi).unwrap();
        assert_eq!(
            base,
            package_identity_fingerprint("slotmap", "2.1.0", first_source_digest, &externals, &abi).unwrap()
        );
        assert_eq!(
            base,
            package_identity_fingerprint("slotmap", "2.1.0", moved_source_digest, &externals, &abi).unwrap(),
            "resolved-project slots and checkout locations do not enter either digest"
        );
        assert_ne!(
            base,
            package_identity_fingerprint("other", "2.1.0", first_source_digest, &externals, &abi).unwrap()
        );
        assert_ne!(
            base,
            package_identity_fingerprint("slotmap", "2.1.1", first_source_digest, &externals, &abi).unwrap()
        );
        assert_ne!(
            base,
            package_identity_fingerprint("slotmap", "2.1.0", ArtifactDigest::from_bytes([10; 32]), &externals, &abi).unwrap()
        );

        let changed_dependency = StableProjectRef::Package {
            package: "dependency".into(),
            version: "1.2.3".into(),
            artifact_fingerprint: Fingerprint128([10; 16]),
        };
        assert_ne!(
            base,
            package_identity_fingerprint("slotmap", "2.1.0", first_source_digest, &[changed_dependency], &abi).unwrap()
        );

        let mut changed_abi = abi;
        changed_abi.semantic_model_version += 1;
        assert_ne!(
            base,
            package_identity_fingerprint("slotmap", "2.1.0", first_source_digest, &externals, &changed_abi).unwrap()
        );
        let mut changed_abi = abi;
        changed_abi.stable_identity_schema += 1;
        assert_ne!(
            base,
            package_identity_fingerprint("slotmap", "2.1.0", first_source_digest, &externals, &changed_abi).unwrap()
        );
        let mut changed_abi = abi;
        changed_abi.module_semantic_abi += 1;
        assert_ne!(
            base,
            package_identity_fingerprint("slotmap", "2.1.0", first_source_digest, &externals, &changed_abi).unwrap()
        );
        let mut changed_abi = abi;
        changed_abi.semantic_artifact_abi += 1;
        assert_ne!(
            base,
            package_identity_fingerprint("slotmap", "2.1.0", first_source_digest, &externals, &changed_abi).unwrap()
        );
        let mut changed_abi = abi;
        changed_abi.query_fingerprint_abi += 1;
        assert_ne!(
            base,
            package_identity_fingerprint("slotmap", "2.1.0", first_source_digest, &externals, &changed_abi).unwrap()
        );
        let mut changed_abi = abi;
        changed_abi.native_surface_schema_version += 1;
        assert_ne!(
            base,
            package_identity_fingerprint("slotmap", "2.1.0", first_source_digest, &externals, &changed_abi).unwrap()
        );
        let mut changed_abi = abi;
        changed_abi.native_catalog_fingerprint[0] ^= 1;
        assert_ne!(
            base,
            package_identity_fingerprint("slotmap", "2.1.0", first_source_digest, &externals, &changed_abi).unwrap()
        );
        let mut changed_abi = abi;
        changed_abi.type_metadata_schema_version += 1;
        assert_ne!(
            base,
            package_identity_fingerprint("slotmap", "2.1.0", first_source_digest, &externals, &changed_abi).unwrap()
        );
    }

    #[test]
    fn package_source_digest_orders_modules_and_covers_path_components_and_exact_text() {
        let project = ResolvedProjectId::from_raw(41);
        let later = module(41, &["slotmap", "z"]);
        let earlier = module(41, &["slotmap", "a"]);
        let unrelated = module(42, &["dependency", "module"]);
        let forward = [(&later, "z text"), (&earlier, "a text"), (&unrelated, "ignored")];
        let reverse = [(&unrelated, "ignored"), (&earlier, "a text"), (&later, "z text")];
        assert_eq!(package_source_digest(project, forward), package_source_digest(project, reverse));

        let unchanged_text = [(&later, "z text"), (&earlier, "a text")];
        let changed_text = [(&later, "z text"), (&earlier, "a text\n")];
        assert_ne!(package_source_digest(project, unchanged_text), package_source_digest(project, changed_text));
        let changed_path = module(41, &["slotmap", "aa"]);
        assert_ne!(
            package_source_digest(project, [(&later, "z text"), (&earlier, "a text")]),
            package_source_digest(project, [(&later, "z text"), (&changed_path, "a text")])
        );
    }
}
