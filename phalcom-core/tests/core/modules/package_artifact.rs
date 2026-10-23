use phalcom_compiler_artifact::container::TeloscContainerView;
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::module_world_v5::ModuleWorldV5;
use phalcom_compiler_artifact::section::SectionKind;
use phalcom_core::modules::{BuiltPackageTelosc, PackageCompilerImage, build_package_telosc};
use phalcom_type_meta::identity::StableProjectRef;
use std::collections::BTreeSet;

pub(super) fn build_and_verify_slotmap_package(
    image: &PackageCompilerImage,
    expected_modules: &BTreeSet<phalcom_modules::ModuleId>,
    limits: &ValidationLimits,
) -> BuiltPackageTelosc {
    assert_eq!(
        phalcom_semantic::canonical_image::canonical_universe_formal_image_build_count(),
        0,
        "package artifact build begins without canonical source bootstrap"
    );
    let built = build_package_telosc(image, limits).expect("build SlotMap package artifact");
    assert_eq!(
        phalcom_semantic::canonical_image::canonical_universe_formal_image_build_count(),
        0,
        "package artifact build must not bootstrap canonical source"
    );
    let container = TeloscContainerView::parse(&built.bytes, limits).expect("parse built SlotMap package artifact");
    assert_eq!(container.directory().len(), 4, "package artifact must contain exactly four required sections");
    assert_eq!(container.header().section_count, 4);
    assert!(
        !container
            .header()
            .capabilities
            .contains(phalcom_compiler_artifact::header::TeloscCapabilities::PORTABLE_EXECUTABLE),
        "package compiler artifact must not claim executable authority"
    );
    assert!(matches!(
        &container.header().package_identity,
        phalcom_compiler_artifact::identity::TeloscPackageIdentity::Package { package, version }
            if package.as_ref() == "slotmap" && version.as_ref() == "0.1.1"
    ));
    assert!(container.header().package_identity.matches_project(&built.identity));

    let module_bytes = container.get_section(SectionKind::ModuleWorld).expect("module-world section");
    let module_world = ModuleWorldV5::decode(module_bytes, limits).expect("decode package module world");
    assert_eq!(module_world.modules.len(), 4);
    assert_eq!(
        module_world.modules.iter().map(|row| row.identity.project.clone()).collect::<BTreeSet<_>>(),
        [built.identity.clone()].into()
    );
    assert_eq!(
        module_world
            .modules
            .iter()
            .map(|row| row.identity.path.iter().cloned().collect::<Vec<_>>())
            .collect::<BTreeSet<_>>()
            .len(),
        4
    );
    assert!(!module_world.external_modules.is_empty());
    assert!(
        module_world
            .external_modules
            .iter()
            .all(|module| matches!(module.project, StableProjectRef::Builtin { ref namespace, .. } if namespace.as_ref() == "universe"))
    );
    assert_eq!(
        module_world.external_projects,
        vec![StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        }]
    );
    assert!(module_world.modules.iter().all(|module| module.identity.project == built.identity));
    assert!(
        module_world
            .external_projects
            .iter()
            .chain(module_world.external_modules.iter().map(|module| &module.project))
            .all(|project| { !matches!(project, StableProjectRef::SourceArtifact { .. } | StableProjectRef::Session { .. }) })
    );
    assert_eq!(
        module_world
            .modules
            .iter()
            .map(|row| row.identity.path.iter().map(|component| component.to_string()).collect::<Vec<_>>())
            .collect::<BTreeSet<_>>(),
        expected_modules
            .iter()
            .map(|module| module
                .path
                .components()
                .iter()
                .map(|component| component.as_str().to_owned())
                .collect::<Vec<_>>())
            .collect::<BTreeSet<_>>(),
        "package module identities must cover the captured SlotMap modules"
    );

    let metadata = container.get_section(SectionKind::Metadata).expect("metadata section");
    let metadata_text = std::str::from_utf8(metadata).expect("metadata section is UTF-8");
    assert!(!metadata_text.contains("SourceArtifact"), "metadata section must not carry checkout identities");
    built
}
