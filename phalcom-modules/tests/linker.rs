use phalcom_ast::parser::parse;
use phalcom_modules::{
    InterfaceBuilder, LinkError, LinkedReadSpec, ModuleComponent, ModuleId, ModuleKind, ModuleLinker, ModulePath, ProjectUniverse, ResolvedProjectId, SymbolId,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

fn module(name: &str) -> ModuleId {
    ModuleId {
        project: ResolvedProjectId(0),
        path: ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]),
    }
}

fn interfaces(sources: &[(ModuleId, &str)]) -> BTreeMap<ModuleId, phalcom_modules::UnlinkedModuleInterface> {
    sources
        .iter()
        .map(|(id, source)| {
            let program = parse(source, 0).program;
            let interface = InterfaceBuilder::build(id.clone(), ModuleKind::Module, &program).unwrap();
            (id.clone(), interface)
        })
        .collect()
}

#[test]
fn selective_import_and_reexport_share_canonical_symbol() {
    let point = module("point");
    let facade = module("facade");
    let consumer = module("consumer");
    let interface_map = interfaces(&[
        (point.clone(), "class Point {}\nexport Point\n"),
        (facade.clone(), "export Point as P from .point\n"),
        (consumer.clone(), "from .facade import P as Point\n"),
    ]);
    let resolved = BTreeMap::from([
        ((facade.clone(), ".point".to_string()), point.clone()),
        ((consumer.clone(), ".facade".to_string()), facade.clone()),
    ]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), interface_map);
    let linked = linker.link(consumer.clone(), &resolved).unwrap();
    let symbol = SymbolId {
        module: point.clone(),
        name: "Point".into(),
    };
    assert_eq!(linked.modules[&facade].interface.exports["P"].symbol, symbol);
    assert_eq!(linked.modules[&consumer].linked_reads, vec![LinkedReadSpec::Binding(symbol.clone())]);
    assert_eq!(linked.modules[&consumer].bindings.imports["Point"].0, 0);
}

#[test]
fn missing_export_is_a_link_error() {
    let source = module("source");
    let consumer = module("consumer");
    let interface_map = interfaces(&[
        (source.clone(), "class Present {}\nexport Present\n"),
        (consumer.clone(), "from .source import Missing\n"),
    ]);
    let resolved = BTreeMap::from([((consumer.clone(), ".source".to_string()), source)]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), interface_map);
    assert!(matches!(linker.link(consumer, &resolved), Err(LinkError::MissingExport { .. })));
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
