use phalcom_ast::parse;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::{FilesystemSourceProvider, ModuleKind, SourceProvider};
use phalcom_modules::{InterfaceBuilder, ModuleComponent, ModuleId, ModuleLinker, ModulePath, ModuleResolver, SymbolId, SymbolicImportTarget};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn snowflake_package_sources_and_conformance_fixtures_parse_and_export() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../packages/snowflake");
    let mut universe = ProjectUniverse::new();
    let project_id = universe.load_root(root.join("project.toml")).expect("Snowflake manifest loads");
    let project = universe.get_project(project_id).expect("Snowflake project exists");
    assert_eq!(project.manifest.as_ref().unwrap().namespace.as_str(), "snowflake");
    let unit = FilesystemSourceProvider::new()
        .locate(project, &ModulePath::root())
        .expect("Snowflake root locates");
    assert_eq!(unit.kind, ModuleKind::Package);
    let tests_id = universe
        .load_root(root.join("tests/project.toml"))
        .expect("Snowflake conformance project loads");

    for (dir, project_id) in [(root.join("src"), project_id), (root.join("tests/src"), tests_id)] {
        for entry in fs::read_dir(dir).expect("source directory") {
            let path = entry.expect("source entry").path();
            if path.extension().is_some_and(|ext| ext == "ph") {
                let parsed = parse(&fs::read_to_string(&path).unwrap(), 0);
                assert!(parsed.errors.is_empty(), "{}: {:?}", path.display(), parsed.errors);
                let kind = if path.file_name().is_some_and(|name| name == "package.ph") {
                    ModuleKind::Package
                } else {
                    ModuleKind::Module
                };
                let module_path = if kind == ModuleKind::Package {
                    ModulePath::root()
                } else {
                    ModulePath::from_components(vec![ModuleComponent::from_kebab(path.file_stem().unwrap().to_str().unwrap()).unwrap()])
                };
                let interface = InterfaceBuilder::build(ModuleId::resolved(project_id, module_path), kind, &parsed.program).expect("source interface builds");
                if path == root.join("src/package.ph") {
                    for name in [
                        "SnowflakeId",
                        "SnowflakeNode",
                        "SnowflakeLayout",
                        "SnowflakeParseError",
                        "SnowflakeGenerationError",
                        "TypedSnowflake",
                        "TypedSnowflakeNode",
                    ] {
                        assert!(interface.exports.contains_key(name), "missing Snowflake export {name}");
                    }
                }
            }
        }
    }
}

#[test]
fn snowflake_package_and_fixtures_link_against_authored_bytes_interface() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../packages/snowflake");
    let mut universe = ProjectUniverse::new();
    let package_id = universe.load_root(root.join("project.toml")).expect("Snowflake project loads");
    let tests_id = universe.load_root(root.join("tests/project.toml")).expect("conformance project loads");
    let bytes_id = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("collections").unwrap(),
        ModuleComponent::from_identifier("bytes").unwrap(),
    ]));
    let bytes_path = root.join("../universe/src/collections/bytes.ph");
    let bytes_source = fs::read_to_string(&bytes_path).expect("authored Bytes source");
    let parsed = parse(&bytes_source, 0);
    assert!(parsed.errors.is_empty(), "authored Bytes: {:?}", parsed.errors);
    let bytes_interface = InterfaceBuilder::build(bytes_id.clone(), ModuleKind::Module, &parsed.program).expect("authored Bytes interface");
    assert!(bytes_interface.exports.contains_key("Bytes"));

    let filesystem = FilesystemSourceProvider::new();
    let mut resolver = ModuleResolver::new(&universe, &filesystem);
    let mut pending = vec![bytes_id.clone()];
    for (project, dir) in [(package_id, root.join("src")), (tests_id, root.join("tests/src"))] {
        for entry in fs::read_dir(dir).expect("source directory") {
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
    }

    let mut interfaces = BTreeMap::new();
    let mut resolved = BTreeMap::new();
    while let Some(id) = pending.pop() {
        if interfaces.contains_key(&id) {
            continue;
        }
        let mut interface = if id == bytes_id {
            bytes_interface.clone()
        } else {
            resolver.load_interface(&id).unwrap_or_else(|error| panic!("{id}: {error}"))
        };
        // This checks the consumer's linked imports against real Universe
        // exports. The authored Bytes implementation itself depends on the
        // staged collections.iteration tree, which the embedded catalog lacks.
        if id.project.is_universe() {
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

    let entry = ModuleId::resolved(tests_id, ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]));
    let linked = ModuleLinker::new(Arc::new(universe), interfaces)
        .link_semantic(entry.clone(), &resolved)
        .expect("Snowflake imports and exports link against the authored Bytes interface");
    assert!(linked.modules.contains_key(&entry));
    assert!(linked.modules.contains_key(&bytes_id));
    for name in ["codec", "id"] {
        let module = ModuleId::resolved(package_id, ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]));
        assert_eq!(
            linked.modules[&module].imports.get("Bytes").expect("Snowflake imports Bytes").target,
            SymbolicImportTarget::Binding(SymbolId {
                module: bytes_id.clone(),
                name: "Bytes".into()
            })
        );
    }
    let fixture = ModuleId::resolved(
        tests_id,
        ModulePath::from_components(vec![ModuleComponent::from_identifier("conversion_test").unwrap()]),
    );
    let id_module = ModuleId::resolved(package_id, ModulePath::from_components(vec![ModuleComponent::from_identifier("id").unwrap()]));
    assert_eq!(
        linked.modules[&fixture].imports.get("SnowflakeId").expect("fixture imports SnowflakeId").target,
        SymbolicImportTarget::Binding(SymbolId {
            module: id_module,
            name: "SnowflakeId".into()
        })
    );
}
