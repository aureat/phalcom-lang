use phalcom_ast::parse;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::{FilesystemSourceProvider, ModuleKind, SourceProvider};
use phalcom_modules::{InterfaceBuilder, ModuleId, ModulePath};
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn uuid_package_and_conformance_sources_parse_and_export() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../packages/uuid");
    let mut universe = ProjectUniverse::new();
    let manifest = root.join("project.toml");
    let project_id = universe.load_root(&manifest).expect("UUID manifest loads");
    let project = universe.get_project(project_id).expect("UUID project");
    assert_eq!(project.manifest.as_ref().unwrap().namespace.as_str(), "uuid");
    let provider = FilesystemSourceProvider::new();
    let unit = provider.locate(project, &ModulePath::root()).expect("UUID package locates");
    assert_eq!(unit.kind, ModuleKind::Package);

    for dir in [root.join("src"), root.join("tests/src")] {
        for entry in fs::read_dir(dir).expect("source directory") {
            let path = entry.expect("source entry").path();
            if path.extension().is_some_and(|ext| ext == "ph") {
                let parsed = parse(&fs::read_to_string(&path).unwrap(), 0);
                assert!(parsed.errors.is_empty(), "{}: {:?}", path.display(), parsed.errors);
                if path == root.join("src/package.ph") {
                    let interface = InterfaceBuilder::build(ModuleId::resolved(project_id, ModulePath::root()), ModuleKind::Package, &parsed.program)
                        .expect("UUID interface builds");
                    for name in [
                        "UUID",
                        "UUIDVersion",
                        "UUIDVariant",
                        "UUIDParseError",
                        "UUIDBytesError",
                        "UUIDGenerationError",
                        "UUIDv7Generator",
                        "TaggedUUID",
                        "UUIDv7",
                        "UUIDv7ParseError",
                    ] {
                        assert!(interface.exports.contains_key(name), "missing UUID export {name}");
                    }
                }
            }
        }
    }
    let tests = root.join("tests/project.toml");
    universe.load_root(&tests).expect("UUID tests manifest loads");

    let sha1 = Path::new(env!("CARGO_MANIFEST_DIR")).join("../packages/universe/src/crypto/sha1.ph");
    let parsed = parse(&fs::read_to_string(sha1).unwrap(), 0);
    assert!(parsed.errors.is_empty(), "SHA-1 module: {:?}", parsed.errors);
    let sha1_interface = InterfaceBuilder::build(ModuleId::universe(ModulePath::root()), ModuleKind::Module, &parsed.program).expect("SHA-1 interface builds");
    assert!(sha1_interface.exports.contains_key("SHA1"));

    let bytes = root.join("../universe/src/collections/bytes.ph");
    let parsed = parse(&fs::read_to_string(bytes).unwrap(), 0);
    assert!(parsed.errors.is_empty(), "Bytes module: {:?}", parsed.errors);
    let bytes_interface = InterfaceBuilder::build(ModuleId::universe(ModulePath::root()), ModuleKind::Module, &parsed.program).expect("Bytes interface builds");
    assert!(bytes_interface.exports.contains_key("Bytes"));
}
