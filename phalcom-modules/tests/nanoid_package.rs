use phalcom_ast::parse;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::{FilesystemSourceProvider, ModuleKind, SourceProvider};
use phalcom_modules::{InterfaceBuilder, LinkedExportTarget, ModuleComponent, ModuleId, ModuleLinker, ModulePath, ModuleResolver, SymbolId};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn nanoid_package_manifest_and_sources_are_valid() {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../packages/nanoid/project.toml")
        .canonicalize()
        .expect("nanoid project.toml exists");

    let mut universe = ProjectUniverse::new();
    let project_id = universe.load_root(&manifest_path).expect("nanoid project loads");
    let project = universe.get_project(project_id).expect("project exists in universe");

    let manifest = project.manifest.as_ref().expect("manifest exists");
    assert_eq!(manifest.name.as_str(), "nanoid");
    assert_eq!(manifest.namespace.as_str(), "nanoid");

    let fs_provider = FilesystemSourceProvider::new();
    let root_unit = fs_provider.locate(project, &ModulePath::root()).expect("nanoid root package locates");
    assert_eq!(root_unit.kind, ModuleKind::Package);

    // Verify each source file in packages/nanoid/src parses with 0 errors
    let src_dir = manifest_path.parent().unwrap().join("src");
    let entries = fs::read_dir(&src_dir).expect("read src dir");
    for entry in entries {
        let entry = entry.expect("valid entry");
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "ph") {
            let content = fs::read_to_string(&path).expect("read source");
            let parsed = parse(&content, 0);
            assert!(parsed.errors.is_empty(), "file {} failed to parse: {:?}", path.display(), parsed.errors);

            let kind = if path.file_name().unwrap() == "package.ph" {
                ModuleKind::Package
            } else {
                ModuleKind::Module
            };
            let module_id = ModuleId::resolved(project_id, ModulePath::root());
            let interface = InterfaceBuilder::build(module_id, kind, &parsed.program).expect("interface builds cleanly");

            if path.file_name().unwrap() == "package.ph" {
                assert!(interface.exports.contains_key("Alphabet"), "exports Alphabet");
                assert!(interface.exports.contains_key("NanoIdGenerator"), "exports NanoIdGenerator");
                assert!(interface.exports.contains_key("NanoId"), "exports NanoId");
                assert!(interface.exports.contains_key("Id"), "exports Id");
                assert!(interface.exports.contains_key("TypedNanoIdGenerator"), "exports TypedNanoIdGenerator");
                assert!(interface.exports.contains_key("NanoIdParseError"), "exports NanoIdParseError");
                assert!(interface.exports.contains_key("InvalidAlphabetError"), "exports InvalidAlphabetError");
                assert!(interface.exports.contains_key("InvalidIdLengthError"), "exports InvalidIdLengthError");
                assert!(interface.exports.contains_key("EntropyError"), "exports EntropyError");
            }
        }
    }

    // Verify test suite project manifest and test sources
    let test_manifest_path = manifest_path.parent().unwrap().join("tests/project.toml");
    let test_project_id = universe.load_root(&test_manifest_path).expect("nanoid-tests project loads");
    let test_project = universe.get_project(test_project_id).expect("test project exists in universe");
    let test_manifest = test_project.manifest.as_ref().expect("test manifest exists");
    assert_eq!(test_manifest.name.as_str(), "nanoid-tests");
    assert_eq!(test_manifest.namespace.as_str(), "nanoid_tests");

    let test_src_dir = test_manifest_path.parent().unwrap().join("src");
    let mut test_files = Vec::new();
    collect_ph_files(&test_src_dir, &mut test_files);
    assert!(!test_files.is_empty(), "test files found");
    for path in test_files {
        let content = fs::read_to_string(&path).expect("read test source");
        let parsed = parse(&content, 0);
        assert!(parsed.errors.is_empty(), "test file {} failed to parse: {:?}", path.display(), parsed.errors);
    }

    // Verify benchmark files parse cleanly
    let bench_dir = manifest_path.parent().unwrap().join("benchmarks");
    let mut bench_files = Vec::new();
    collect_ph_files(&bench_dir, &mut bench_files);
    assert!(!bench_files.is_empty(), "bench files found");
    for path in bench_files {
        let content = fs::read_to_string(&path).expect("read bench source");
        let parsed = parse(&content, 0);
        assert!(parsed.errors.is_empty(), "bench file {} failed to parse: {:?}", path.display(), parsed.errors);
    }
}

fn collect_ph_files(dir: &std::path::Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read dir") {
        let entry = entry.expect("valid entry");
        let path = entry.path();
        if path.is_dir() {
            collect_ph_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "ph") {
            files.push(path);
        }
    }
}

#[test]
fn nanoid_root_preserves_the_tagged_id_declaration_identity() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../packages/nanoid");
    let mut universe = ProjectUniverse::new();
    let project = universe.load_root(root.join("project.toml")).expect("nanoid project loads");
    let filesystem = FilesystemSourceProvider::new();
    let mut resolver = ModuleResolver::new(&universe, &filesystem);
    let bytes = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("collections").unwrap(),
        ModuleComponent::from_identifier("bytes").unwrap(),
    ]));
    let bytes_source = fs::read_to_string(root.join("../universe/src/collections/bytes.ph")).expect("authored Bytes source");
    let parsed = parse(&bytes_source, 0);
    assert!(parsed.errors.is_empty(), "{:#?}", parsed.errors);
    let bytes_interface = InterfaceBuilder::build(bytes.clone(), ModuleKind::Module, &parsed.program).expect("authored Bytes interface builds");
    let mut pending = vec![bytes.clone()];
    for entry in fs::read_dir(root.join("src")).expect("nanoid sources") {
        let file = entry.expect("source entry").path();
        if !file.extension().is_some_and(|ext| ext == "ph") {
            continue;
        }
        let name = file.file_stem().unwrap().to_str().unwrap();
        let path = if name == "package" {
            ModulePath::root()
        } else {
            ModulePath::from_components(vec![ModuleComponent::from_kebab(name).unwrap()])
        };
        pending.push(ModuleId::resolved(project, path));
    }
    let mut interfaces = BTreeMap::new();
    let mut resolved = BTreeMap::new();
    while let Some(id) = pending.pop() {
        if interfaces.contains_key(&id) {
            continue;
        }
        let mut interface = if id == bytes {
            bytes_interface.clone()
        } else {
            resolver.load_interface(&id).unwrap_or_else(|error| panic!("{id}: {error}"))
        };
        if id == bytes {
            // The authored Bytes internals depend on a staged Universe tree.
            interface.imports.clear();
        }
        for import in &interface.imports {
            let path = import.path();
            let target = resolver
                .resolve_import(&id, path)
                .unwrap_or_else(|error| panic!("{id} importing {path}: {error}"));
            resolved.insert((id.clone(), path.to_string()), target.id.clone());
            pending.push(target.id);
        }
        interfaces.insert(id, interface);
    }

    let root_id = ModuleId::resolved(project, ModulePath::root());
    let linked = ModuleLinker::new(Arc::new(universe), interfaces)
        .link_semantic(root_id.clone(), &resolved)
        .expect("nanoid package import/re-export graph links");
    let id = ModuleId::resolved(project, ModulePath::from_components(vec![ModuleComponent::from_identifier("id").unwrap()]));
    assert_eq!(
        linked.modules[&root_id].interface.exports["Id"].target,
        LinkedExportTarget::Binding(SymbolId { module: id, name: "Id".into() })
    );
}
