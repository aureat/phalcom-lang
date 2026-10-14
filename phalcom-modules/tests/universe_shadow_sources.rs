use phalcom_ast::ast::{ImportPath, ImportRoot};
use phalcom_ast::parse;
use phalcom_modules::{ImportSurface, InterfaceBuilder, ModuleComponent, ModuleId, ModuleKind, ModulePath, UnlinkedModuleInterface};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const ITERATION_DECLARATIONS: [&str; 16] = [
    "Iterator",
    "Iterable",
    "MappedIterator",
    "FilterIterator",
    "FilterMapIterator",
    "FlatMapIterator",
    "FlattenIterator",
    "ChainIterator",
    "SkipIterator",
    "SkipWhileIterator",
    "TakeIterator",
    "TakeWhileIterator",
    "MapWhileIterator",
    "InspectIterator",
    "StrictZipIterator",
    "IndexedIterator",
];

struct ShadowModule {
    source_path: PathBuf,
    interface: UnlinkedModuleInterface,
}

#[test]
fn universe_shadow_sources_parse_and_have_valid_unlinked_interfaces() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../packages/universe/src");
    let (modules, mut errors) = load_shadow_modules(&root);
    validate_shadow_relationships(&modules, &mut errors);

    errors.sort();
    assert!(errors.is_empty(), "shadow source validation failed:\n{}", errors.join("\n"));
    assert!(!modules.is_empty(), "no shadow source modules found under {}", root.display());

    let iteration_path = iteration_package_path();
    let iteration = modules.get(&iteration_path).expect("iteration package source exists");
    assert_eq!(iteration.interface.kind, ModuleKind::Package);
    assert_eq!(iteration.interface.declarations.len(), ITERATION_DECLARATIONS.len());
    for name in ITERATION_DECLARATIONS {
        assert!(iteration.interface.declarations.contains_key(name), "iteration declaration `{name}` is missing");
        assert!(iteration.interface.exports.contains_key(name), "iteration declaration `{name}` is not exported");

        let owners = modules
            .iter()
            .filter(|(_, module)| module.interface.declarations.contains_key(name))
            .map(|(path, _)| path.clone())
            .collect::<Vec<_>>();
        let expected_owners = if name == "ChainIterator" {
            vec![iteration_path.clone(), functional_chain_path()]
        } else {
            vec![iteration_path.clone()]
        };
        assert_eq!(owners, expected_owners, "unexpected shadow owners for `{name}`");
    }
    assert!(!root.join("collections/iterator_laws.ph").exists(), "old shadow source path remains");
}

fn load_shadow_modules(root: &Path) -> (BTreeMap<ModulePath, ShadowModule>, Vec<String>) {
    let mut sources = Vec::new();
    let mut errors = Vec::new();
    if let Err(error) = collect_sources(root, root, &mut sources) {
        errors.push(format!("{}: {error}", root.display()));
        return (BTreeMap::new(), errors);
    }
    sources.sort_by(|left, right| left.0.cmp(&right.0));

    let mut modules = BTreeMap::new();
    for (relative_path, source_path) in sources {
        let (module_path, kind) = match module_path_for_source(&relative_path) {
            Ok(module) => module,
            Err(error) => {
                errors.push(format!("{}: {error}", source_path.display()));
                continue;
            }
        };
        let source = match fs::read_to_string(&source_path) {
            Ok(source) => source,
            Err(error) => {
                errors.push(format!("{}: {error}", source_path.display()));
                continue;
            }
        };
        if source.contains("iterator_laws") {
            errors.push(format!("{}: stale `iterator_laws` source reference", source_path.display()));
        }
        let parsed = parse(&source, 0);
        if !parsed.errors.is_empty() {
            for error in parsed.errors {
                errors.push(format!("{}: parse error: {error}", source_path.display()));
            }
            continue;
        }

        let id = ModuleId::universe(module_path.clone());
        let interface = match InterfaceBuilder::build(id, kind, &parsed.program) {
            Ok(interface) => interface,
            Err(error) => {
                errors.push(format!("{}: interface error: {error}", source_path.display()));
                continue;
            }
        };

        let module = ShadowModule {
            source_path: source_path.clone(),
            interface,
        };
        if let Some(previous) = modules.insert(module_path.clone(), module) {
            errors.push(format!(
                "duplicate module path universe.{module_path}: {} and {}",
                previous.source_path.display(),
                source_path.display()
            ));
        }
    }
    (modules, errors)
}

fn collect_sources(root: &Path, directory: &Path, sources: &mut Vec<(PathBuf, PathBuf)>) -> std::io::Result<()> {
    let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_sources(root, &path, sources)?;
        } else if file_type.is_file() && path.extension().is_some_and(|extension| extension == "ph") {
            sources.push((path.strip_prefix(root).expect("source is under root").to_path_buf(), path));
        }
    }
    Ok(())
}

fn module_path_for_source(relative_path: &Path) -> Result<(ModulePath, ModuleKind), String> {
    let mut components = Vec::new();
    let file_name = relative_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "source path is not valid UTF-8".to_string())?;

    let parent = relative_path.parent().unwrap_or_else(|| Path::new(""));
    for component in parent.components() {
        let name = component
            .as_os_str()
            .to_str()
            .ok_or_else(|| "module directory is not valid UTF-8".to_string())?;
        components.push(module_component(name)?);
    }

    let kind = if file_name == "package.ph" {
        ModuleKind::Package
    } else {
        let stem = relative_path
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "module name is not valid UTF-8".to_string())?;
        components.push(module_component(stem)?);
        ModuleKind::Module
    };

    Ok((ModulePath::from_components(components), kind))
}

fn module_component(name: &str) -> Result<ModuleComponent, String> {
    let component = if name.contains('-') {
        ModuleComponent::from_kebab(name)
    } else {
        ModuleComponent::from_identifier(name)
    };
    component.map_err(|error| error.to_string())
}

fn validate_shadow_relationships(modules: &BTreeMap<ModulePath, ShadowModule>, errors: &mut Vec<String>) {
    for (path, module) in modules {
        for import in &module.interface.imports {
            let target_path = match universe_import_target(path, module.interface.kind, import.path()) {
                Ok(Some(target)) => target,
                Ok(None) => continue,
                Err(error) => {
                    errors.push(format!("{}: {error}", module.source_path.display()));
                    continue;
                }
            };
            // Existing non-iteration imports include baseline package paths that are not exposed.
            // Enforce resolver-equivalent visibility on the migrated S4 path while still checking
            // target existence and selected exports for every Universe import in the tree.
            if target_path == iteration_package_path() {
                if let Err(error) = validate_import_visibility(&target_path, modules) {
                    errors.push(format!("{}: import target universe.{target_path}: {error}", module.source_path.display()));
                    continue;
                }
            }
            let Some(target) = modules.get(&target_path) else {
                errors.push(format!("{}: import target universe.{target_path} is absent", module.source_path.display()));
                continue;
            };

            let imported_names = match import {
                ImportSurface::Module(_) => continue,
                ImportSurface::Selective(selective) => selective.items.iter().map(|item| item.name.as_str()).collect::<Vec<_>>(),
                ImportSurface::ReExport(reexport) => reexport.items.iter().map(|item| item.local_or_remote_name.as_str()).collect::<Vec<_>>(),
            };
            for name in imported_names {
                if !target.interface.exports.contains_key(name) {
                    errors.push(format!(
                        "{}: import from universe.{target_path} selects `{name}`, which is not exported",
                        module.source_path.display()
                    ));
                }
            }
        }

        if module.interface.kind == ModuleKind::Package {
            for child in &module.interface.exposed_children {
                let target_path = path.join(child.clone());
                if !modules.contains_key(&target_path) {
                    errors.push(format!(
                        "{}: exposed child universe.{target_path} has no module or package source",
                        module.source_path.display()
                    ));
                }
            }
        }
    }
}

fn iteration_package_path() -> ModulePath {
    ModulePath::from_components(vec![
        ModuleComponent::from_identifier("collections").expect("static module component"),
        ModuleComponent::from_identifier("iteration").expect("static module component"),
    ])
}

fn functional_chain_path() -> ModulePath {
    ModulePath::from_components(vec![
        ModuleComponent::from_identifier("functional").expect("static module component"),
        ModuleComponent::from_identifier("chain").expect("static module component"),
    ])
}

fn validate_import_visibility(target: &ModulePath, modules: &BTreeMap<ModulePath, ShadowModule>) -> Result<(), String> {
    let mut parent_path = ModulePath::root();
    let mut parent = modules.get(&parent_path).ok_or_else(|| "root package source is absent".to_string())?;
    if parent.interface.kind != ModuleKind::Package {
        return Err("root source is not a package".to_string());
    }

    for (index, component) in target.components().iter().enumerate() {
        if parent.interface.kind != ModuleKind::Package {
            return Err(format!("intermediate module universe.{parent_path} is not a package"));
        }
        if !parent.interface.exposed_children.contains(component) {
            return Err(format!("path component `{component}` is not exposed by universe.{parent_path}"));
        }
        parent_path = parent_path.join(component.clone());
        parent = modules
            .get(&parent_path)
            .ok_or_else(|| format!("module or package universe.{parent_path} is absent"))?;
        let is_intermediate = index + 1 < target.components().len();
        if is_intermediate && parent.interface.kind != ModuleKind::Package {
            return Err(format!("intermediate module universe.{parent_path} is not a package"));
        }
    }
    Ok(())
}

fn universe_import_target(importer: &ModulePath, kind: ModuleKind, import: &ImportPath) -> Result<Option<ModulePath>, String> {
    let mut components = match &import.root {
        ImportRoot::Absolute(root) if root.name == "universe" => Vec::new(),
        ImportRoot::Absolute(_) => return Ok(None),
        ImportRoot::Relative { dots, .. } => {
            if *dots == 0 {
                return Err("relative import must begin with at least one dot".to_string());
            }
            let package = match kind {
                ModuleKind::Package => importer.clone(),
                ModuleKind::Module => importer.parent().unwrap_or_else(ModulePath::root),
            };
            let ascend = usize::from(*dots) - 1;
            if ascend > package.components().len() {
                return Err(format!("relative import ascends {dots} levels beyond its package"));
            }
            package.components()[..package.components().len() - ascend].to_vec()
        }
    };

    for segment in &import.segments {
        components.push(ModuleComponent::from_identifier(&segment.name).map_err(|error| error.to_string())?);
    }
    Ok(Some(ModulePath::from_components(components)))
}
