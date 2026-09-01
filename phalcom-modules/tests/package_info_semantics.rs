use phalcom_modules::manifest::ProjectManifest;
use phalcom_modules::package_info::{PackageArtifactIdentity, PackageAuthorDescriptor, PackageInfoDescriptor};

#[test]
fn test_package_author_parsing() {
    let a1 = PackageAuthorDescriptor::parse("Alice <alice@example.com> (https://alice.dev)");
    assert_eq!(&a1.name, "Alice");
    assert_eq!(a1.email.as_deref(), Some("alice@example.com"));
    assert_eq!(a1.url.as_deref(), Some("https://alice.dev"));

    let a2 = PackageAuthorDescriptor::parse("Bob <bob@example.com>");
    assert_eq!(&a2.name, "Bob");
    assert_eq!(a2.email.as_deref(), Some("bob@example.com"));
    assert_eq!(a2.url, None);

    let a3 = PackageAuthorDescriptor::parse("Charlie (https://charlie.org)");
    assert_eq!(&a3.name, "Charlie");
    assert_eq!(a3.email, None);
    assert_eq!(a3.url.as_deref(), Some("https://charlie.org"));

    let a4 = PackageAuthorDescriptor::parse("Simple Author");
    assert_eq!(&a4.name, "Simple Author");
    assert_eq!(a4.email, None);
    assert_eq!(a4.url, None);
}

#[test]
fn test_package_artifact_identity() {
    let id1 = PackageArtifactIdentity::Resolved {
        name: "my_pkg".to_string(),
        version: Some("1.2.3".to_string()),
    };
    assert_eq!(id1.canonical_uri(), "pkg:my_pkg@1.2.3");
    assert_eq!(id1.to_string(), "pkg:my_pkg@1.2.3");

    let id2 = PackageArtifactIdentity::Standalone("standalone_pkg".into());
    assert_eq!(id2.canonical_uri(), "pkg:standalone_pkg");
}

#[test]
fn test_package_info_from_manifest() {
    let toml = r#"
[project]
name = "demo_pkg"
namespace = "demo_pkg"
version = "0.2.1"
authors = ["Alice <alice@example.com> (https://alice.dev)"]
description = "A demo package"
license = "MIT"
homepage = "https://demo.dev"
repository = "https://github.com/example/demo"
source = "src"
entry = "demo_pkg.main"
default_entry = "demo_pkg.cli"

[dependencies]
helper = { path = "../helper" }
"#;

    let manifest = ProjectManifest::parse(toml).unwrap();
    let validated = manifest.validate().unwrap();

    let info = PackageInfoDescriptor::from_manifest(&validated);
    assert_eq!(&*info.name, "demo_pkg");
    assert_eq!(&*info.namespace, "demo_pkg");
    assert_eq!(info.version.as_deref(), Some("0.2.1"));
    assert_eq!(info.authors.len(), 1);
    assert_eq!(&info.authors[0].name, "Alice");
    assert_eq!(info.description.as_deref(), Some("A demo package"));
    assert_eq!(info.license.as_deref(), Some("MIT"));
    assert_eq!(info.homepage.as_deref(), Some("https://demo.dev"));
    assert_eq!(info.repository.as_deref(), Some("https://github.com/example/demo"));
    assert_eq!(info.default_entry.as_deref(), Some("demo_pkg.cli"));
    assert_eq!(info.identity.canonical_uri(), "pkg:demo_pkg@0.2.1");

    assert_eq!(info.requirements.len(), 1);
    assert_eq!(&*info.requirements[0].alias, "helper");
}

#[test]
fn test_builtin_package_info() {
    let uni = PackageInfoDescriptor::builtin_universe(None);
    assert_eq!(&*uni.name, "universe");
    assert_eq!(&*uni.namespace, "universe");
    assert_eq!(uni.identity.canonical_uri(), "pkg:universe");
}
