use phalcom_ast::parser::parse;
use phalcom_modules::{
    ImportBindingId, InterfaceBuilder, LinkError, LinkedReadSpec, ModuleComponent, ModuleId, ModuleKind, ModuleLinker, ModulePath, ProjectUniverse,
    ResolvedProjectId, SymbolId, UnlinkedModuleInterface,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

fn module(name: &str) -> ModuleId {
    ModuleId {
        project: ResolvedProjectId::from_raw(1).into(),
        path: ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]),
    }
}

fn interfaces(sources: &[(ModuleId, &str)]) -> BTreeMap<ModuleId, UnlinkedModuleInterface> {
    sources
        .iter()
        .map(|(id, source)| {
            let program = parse(source, 0).program;
            let interface = InterfaceBuilder::build(id.clone(), ModuleKind::Module, &program).unwrap();
            (id.clone(), interface)
        })
        .collect()
}

/// LINK-01 — Selective import of exported name succeeds and produces LinkedReadSpec::Binding
#[test]
fn link_01_selective_import_produces_binding() {
    let exporter = module("exporter");
    let importer = module("importer");
    let iface_map = interfaces(&[
        (exporter.clone(), "class Exported {}\nexport Exported\n"),
        (importer.clone(), "from .exporter import Exported\n"),
    ]);
    let resolved = BTreeMap::from([((importer.clone(), ".exporter".to_string()), exporter.clone())]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);
    let linked = linker.link(importer.clone(), &resolved).expect("link should succeed");

    let expected_symbol = SymbolId {
        module: exporter.clone(),
        name: "Exported".into(),
    };
    assert_eq!(linked.modules[&importer].linked_reads, vec![LinkedReadSpec::Binding(expected_symbol)]);
    assert_eq!(linked.modules[&importer].bindings.imports["Exported"], ImportBindingId(0));
}

/// LINK-02 — Selective import of non-exported name produces LinkError::MissingExport
#[test]
fn link_02_selective_import_missing_export() {
    let exporter = module("exporter");
    let importer = module("importer");
    let iface_map = interfaces(&[(exporter.clone(), "class Private {}\n"), (importer.clone(), "from .exporter import Private\n")]);
    let resolved = BTreeMap::from([((importer.clone(), ".exporter".to_string()), exporter.clone())]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);
    let result = linker.link(importer, &resolved);

    assert!(
        matches!(result, Err(LinkError::MissingExport { ref name, .. }) if name == "Private"),
        "expected MissingExport for Private, got {:?}",
        result
    );
}

/// LINK-03 — Module-import produces LinkedReadSpec::Module
#[test]
fn link_03_module_import_produces_module_spec() {
    let exporter = module("exporter");
    let importer = module("importer");
    let iface_map = interfaces(&[(exporter.clone(), "class Widget {}\nexport Widget\n"), (importer.clone(), "import .exporter\n")]);
    let resolved = BTreeMap::from([((importer.clone(), ".exporter".to_string()), exporter.clone())]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);
    let linked = linker.link(importer.clone(), &resolved).expect("link should succeed");

    assert_eq!(linked.modules[&importer].linked_reads, vec![LinkedReadSpec::Module(exporter.clone())]);
    assert_eq!(linked.modules[&importer].bindings.imports["exporter"], ImportBindingId(0));
}

/// LINK-04 — Import alias rebinds the local name
#[test]
fn link_04_import_alias_rebinds_local_name() {
    let exporter = module("exporter");
    let importer = module("importer");
    let iface_map = interfaces(&[
        (exporter.clone(), "class Widget {}\nexport Widget\n"),
        (importer.clone(), "import .exporter as exp\n"),
    ]);
    let resolved = BTreeMap::from([((importer.clone(), ".exporter".to_string()), exporter.clone())]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);
    let linked = linker.link(importer.clone(), &resolved).expect("link should succeed");

    assert_eq!(linked.modules[&importer].bindings.imports.get("exp"), Some(&ImportBindingId(0)));
    assert_eq!(linked.modules[&importer].bindings.imports.get("exporter"), None);
}

/// LINK-05 — Re-export chains correctly to the original symbol
#[test]
fn link_05_reexport_chains_to_original_symbol() {
    let mod_a = module("mod_a");
    let mod_b = module("mod_b");
    let mod_c = module("mod_c");
    let iface_map = interfaces(&[
        (mod_a.clone(), "class Foo {}\nexport Foo\n"),
        (mod_b.clone(), "export Foo from .mod_a\n"),
        (mod_c.clone(), "from .mod_b import Foo\n"),
    ]);
    let resolved = BTreeMap::from([
        ((mod_b.clone(), ".mod_a".to_string()), mod_a.clone()),
        ((mod_c.clone(), ".mod_b".to_string()), mod_b.clone()),
    ]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);
    let linked = linker.link(mod_c.clone(), &resolved).expect("link should succeed");

    let original_symbol = SymbolId {
        module: mod_a.clone(),
        name: "Foo".into(),
    };
    assert_eq!(linked.modules[&mod_c].linked_reads, vec![LinkedReadSpec::Binding(original_symbol)]);
}

/// LINK-06 — Cyclic re-export produces LinkError::CyclicReExport
#[test]
fn link_06_cyclic_reexport_rejected() {
    let mod_a = module("mod_a");
    let mod_b = module("mod_b");
    let iface_map = interfaces(&[(mod_a.clone(), "export Foo from .mod_b\n"), (mod_b.clone(), "export Foo from .mod_a\n")]);
    let resolved = BTreeMap::from([
        ((mod_a.clone(), ".mod_b".to_string()), mod_b.clone()),
        ((mod_b.clone(), ".mod_a".to_string()), mod_a.clone()),
    ]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);
    let result = linker.link(mod_a, &resolved);

    assert!(
        matches!(result, Err(LinkError::CyclicReExport { .. })),
        "expected CyclicReExport, got {:?}",
        result
    );
}

/// LINK-07 — Diamond dependency is deduplicated in initialization order
#[test]
fn link_07_diamond_dependency_deduplicated_order() {
    let main_mod = module("main_mod");
    let mod_a = module("mod_a");
    let mod_b = module("mod_b");
    let base_mod = module("base_mod");

    let iface_map = interfaces(&[
        (base_mod.clone(), "class Base {}\nexport Base\n"),
        (mod_a.clone(), "from .base_mod import Base\nclass A {}\nexport A\n"),
        (mod_b.clone(), "from .base_mod import Base\nclass B {}\nexport B\n"),
        (main_mod.clone(), "from .mod_a import A\nfrom .mod_b import B\n"),
    ]);
    let resolved = BTreeMap::from([
        ((mod_a.clone(), ".base_mod".to_string()), base_mod.clone()),
        ((mod_b.clone(), ".base_mod".to_string()), base_mod.clone()),
        ((main_mod.clone(), ".mod_a".to_string()), mod_a.clone()),
        ((main_mod.clone(), ".mod_b".to_string()), mod_b.clone()),
    ]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);
    let linked = linker.link(main_mod.clone(), &resolved).expect("link should succeed");

    let order = &linked.initialization_order;
    // base_mod must appear exactly once
    assert_eq!(order.iter().filter(|m| **m == base_mod).count(), 1);

    let pos_base = order.iter().position(|m| *m == base_mod).unwrap();
    let pos_a = order.iter().position(|m| *m == mod_a).unwrap();
    let pos_b = order.iter().position(|m| *m == mod_b).unwrap();
    let pos_main = order.iter().position(|m| *m == main_mod).unwrap();

    assert!(pos_base < pos_a, "base must precede a");
    assert!(pos_base < pos_b, "base must precede b");
    assert!(pos_a < pos_main, "a must precede main");
    assert!(pos_b < pos_main, "b must precede main");
}

/// LINK-08 — Missing module in link universe produces LinkError::MissingModule
#[test]
fn link_08_missing_module_in_universe() {
    let entry = module("entry");
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), BTreeMap::new());
    let result = linker.link(entry.clone(), &BTreeMap::new());

    assert!(
        matches!(result, Err(LinkError::MissingModule { ref module }) if *module == entry),
        "expected MissingModule, got {:?}",
        result
    );
}

/// LINK-09 — Binding collision (same local name imported twice) produces LinkError::BindingCollision
#[test]
fn link_09_binding_collision_rejected() {
    let mod_a = module("mod_a");
    let mod_b = module("mod_b");
    let importer = module("importer");

    let iface_a = interfaces(&[(mod_a.clone(), "class Foo {}\nexport Foo\n")]).remove(&mod_a).unwrap();
    let iface_b = interfaces(&[(mod_b.clone(), "class Foo {}\nexport Foo\n")]).remove(&mod_b).unwrap();

    let mut importer_iface = interfaces(&[(importer.clone(), "from .mod_a import Foo\n")]).remove(&importer).unwrap();
    let second_iface = interfaces(&[(importer.clone(), "from .mod_b import Foo\n")]).remove(&importer).unwrap();
    importer_iface.imports.extend(second_iface.imports);

    let iface_map = BTreeMap::from([(mod_a.clone(), iface_a), (mod_b.clone(), iface_b), (importer.clone(), importer_iface)]);
    let resolved = BTreeMap::from([
        ((importer.clone(), ".mod_a".to_string()), mod_a.clone()),
        ((importer.clone(), ".mod_b".to_string()), mod_b.clone()),
    ]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);
    let result = linker.link(importer, &resolved);

    assert!(
        matches!(result, Err(LinkError::BindingCollision { ref name, .. }) if name == "Foo"),
        "expected BindingCollision for Foo, got {:?}",
        result
    );
}

#[test]
fn unreachable_interface_is_not_in_linked_program() {
    let entry = module("entry");
    let unused = module("unused");
    let linker = ModuleLinker::new(
        Arc::new(ProjectUniverse::new()),
        interfaces(&[(entry.clone(), ""), (unused.clone(), "class Never {}\n")]),
    );
    let linked = linker.link(entry.clone(), &BTreeMap::new()).unwrap();
    assert_eq!(linked.modules.keys().cloned().collect::<BTreeSet<_>>(), BTreeSet::from([entry]));
}

#[test]
fn tolerant_linker_accumulates_diagnostics_and_preserves_valid_modules() {
    let mod_good = module("good");
    let mod_broken = module("broken");
    let mod_dep = module("dep_on_broken");
    let iface_map = interfaces(&[
        (mod_good.clone(), "class Good {}\nexport Good\n"),
        (mod_broken.clone(), "from .nonexistent import Missing\n"),
        (mod_dep.clone(), "from .broken import Missing\n"),
    ]);
    let resolved = BTreeMap::from([
        ((mod_broken.clone(), ".nonexistent".to_string()), module("nonexistent")),
        ((mod_dep.clone(), ".broken".to_string()), mod_broken.clone()),
    ]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);

    // Linking mod_good: standalone component, succeeds completely
    let res_good = linker.link_component_tolerant(mod_good.clone(), &resolved);
    assert!(res_good.program.modules.contains_key(&mod_good));
    assert!(res_good.diagnostics.is_empty());
    assert!(res_good.blocked_modules.is_empty());

    // Linking mod_dep: imports broken module which has unresolvable import
    let res_broken = linker.link_component_tolerant(mod_dep.clone(), &resolved);
    assert!(res_broken.blocked_modules.contains(&mod_dep) || res_broken.blocked_modules.contains(&mod_broken));
    assert!(!res_broken.diagnostics.is_empty());
}

#[test]
fn tolerant_runtime_cycle_preserves_independent_survivor_order() {
    let mod_x = module("x");
    let mod_y = module("y");
    let mod_z = module("z");
    let mod_w = module("w");

    // Cycle between X and Y: X imports Y, Y imports X
    // Independent pair Z -> W: Z imports W
    let iface_map = interfaces(&[
        (mod_x.clone(), "import .y as Y\n"),
        (mod_y.clone(), "import .x as X\n"),
        (mod_w.clone(), "class W {}\nexport W\n"),
        (mod_z.clone(), "import .w as W\n"),
    ]);

    let resolved = BTreeMap::from([
        ((mod_x.clone(), ".y".to_string()), mod_y.clone()),
        ((mod_y.clone(), ".x".to_string()), mod_x.clone()),
        ((mod_z.clone(), ".w".to_string()), mod_w.clone()),
    ]);

    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), iface_map);

    // Link cyclic component X
    let res_x = linker.link_component_tolerant(mod_x.clone(), &resolved);
    // Link independent component Z
    let res_z = linker.link_component_tolerant(mod_z.clone(), &resolved);

    // Z -> W produces valid topological order [W, Z]
    let order_z = res_z.program.graphs.runtime.initialization_order().expect("valid topological order for Z-W");
    let pos_w = order_z.iter().position(|m| m == &mod_w);
    let pos_z = order_z.iter().position(|m| m == &mod_z);
    assert!(pos_w.is_some() && pos_z.is_some());
    assert!(pos_w.unwrap() < pos_z.unwrap(), "W must precede Z in topological order");

    // X/Y cycle is blocked in res_x, diagnostic reported
    assert!(!res_x.diagnostics.is_empty() || !res_x.blocked_modules.is_empty());
}

