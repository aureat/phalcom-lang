use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface};
use phalcom_modules::linker::{ImportBindingId, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout, SymbolId};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::types::TypeResolver;
use phalcom_semantic::{DeclarationId, LinkedTypeResolver};
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

fn module(project: ResolvedProjectId, path: &[&str]) -> ModuleId {
    ModuleId::resolved(
        project,
        ModulePath::from_components(
            path.iter()
                .map(|component| ModuleComponent::from_identifier(component).expect("valid module component"))
                .collect::<Vec<_>>(),
        ),
    )
}

fn resolver_fixture() -> (LinkedTypeResolver, ModuleId, DeclarationId) {
    let project = ResolvedProjectId::from_raw(91);
    let current = module(project, &["consumer"]);
    let imported = module(project, &["types"]);
    let leaf = DeclarationId::new(imported.clone(), "Leaf".into());

    let linked_current = LinkedModule {
        interface: LinkedModuleInterface {
            module: current.clone(),
            kind: ModuleKind::Module,
            exports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::new()),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::new()),
            imports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([("pkg".into(), ImportBindingId(0))])),
        },
        linked_reads: vec![LinkedReadSpec::Module(imported.clone())],
        runtime_dependencies: vec![imported.clone()],
    };
    let linked_imported = LinkedModule {
        interface: LinkedModuleInterface {
            module: imported.clone(),
            kind: ModuleKind::Module,
            exports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([(
                "Leaf".into(),
                LinkedExport {
                    public_name: "Leaf".into(),
                    target: LinkedExportTarget::Binding(SymbolId {
                        module: imported.clone(),
                        name: "Leaf".into(),
                    }),
                    range: Default::default(),
                },
            )])),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout::default(),
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([
            (current.clone(), linked_current),
            (imported.clone(), linked_imported),
        ])),
        graphs: Default::default(),
        entry: current.clone(),
        initialization_order: vec![current.clone()],
    });
    let resolver = LinkedTypeResolver::new(linked, HashSet::from([leaf.clone()]), ModuleId::universe_root());
    (resolver, current, leaf)
}

#[test]
fn qualified_type_resolution_preserves_single_member_lookup() {
    let (resolver, current, leaf) = resolver_fixture();
    assert_eq!(resolver.resolve_type_name(&current, "pkg", &["Leaf".into()]), Some(leaf));
}

#[test]
fn qualified_type_resolution_never_drops_intermediate_components() {
    let (resolver, current, _) = resolver_fixture();
    assert_eq!(
        resolver.resolve_type_name(&current, "pkg", &["nested".into(), "Leaf".into()]),
        None,
        "pkg.nested.Leaf must not be silently reinterpreted as pkg.Leaf"
    );
}

#[test]
fn symbolic_import_resolution_does_not_require_runtime_slots() {
    let project = ResolvedProjectId::from_raw(91);
    let current = module(project, &["consumer"]);
    let provider = module(project, &["types"]);
    let interfaces = [
        (provider.clone(), "class Leaf {}\nexport Leaf\n"),
        (current.clone(), "from .types import Leaf as Alias\nimport .types as pkg\n"),
    ]
    .into_iter()
    .map(|(module, source)| {
        let parsed = phalcom_ast::parse(source, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        let interface = phalcom_modules::InterfaceBuilder::build(module.clone(), ModuleKind::Module, &parsed.program).unwrap();
        (module, interface)
    })
    .collect();
    let linker = phalcom_modules::ModuleLinker::new(Arc::new(ProjectUniverse::new()), interfaces);
    let symbolic = Arc::new(
        linker
            .link_semantic(current.clone(), &BTreeMap::from([((current.clone(), ".types".into()), provider.clone())]))
            .unwrap(),
    );
    let leaf = DeclarationId::new(provider.clone(), "Leaf".into());
    let resolver = LinkedTypeResolver::from_semantic(symbolic, HashSet::from([leaf.clone()]));
    assert_eq!(resolver.resolve_type_name(&current, "Alias", &[]), Some(leaf.clone()));
    assert_eq!(resolver.resolve_type_name(&current, "pkg", &["Leaf".into()]), Some(leaf));
    assert_eq!(resolver.resolve_module_alias(&current, "pkg"), Some(provider));
}

#[test]
fn symbolic_semantic_link_resolves_mutual_type_cycle_without_runtime_slots() {
    let project = ResolvedProjectId::from_raw(92);
    let mod_a = module(project, &["a"]);
    let mod_b = module(project, &["b"]);
    let interfaces = [
        (mod_a.clone(), "from .b import BType\nclass AType {}\nexport AType\n"),
        (mod_b.clone(), "from .a import AType\nclass BType {}\nexport BType\n"),
    ]
    .into_iter()
    .map(|(module, source)| {
        let parsed = phalcom_ast::parse(source, 0);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        let interface = phalcom_modules::InterfaceBuilder::build(module.clone(), ModuleKind::Module, &parsed.program).unwrap();
        (module, interface)
    })
    .collect();
    let linker = phalcom_modules::ModuleLinker::new(Arc::new(ProjectUniverse::new()), interfaces);
    let resolved = BTreeMap::from([((mod_a.clone(), ".b".into()), mod_b.clone()), ((mod_b.clone(), ".a".into()), mod_a.clone())]);
    // Note: link_all would fail with a cyclic dependency error here because of runtime DAG check.
    assert!(linker.link_all(mod_a.clone(), &resolved).is_err());

    // But symbolic linkage succeeds because it has no runtime DAG or ordering requirement.
    let symbolic = Arc::new(linker.link_semantic(mod_a.clone(), &resolved).unwrap());
    let a_type = DeclarationId::new(mod_a.clone(), "AType".into());
    let b_type = DeclarationId::new(mod_b.clone(), "BType".into());
    let resolver = LinkedTypeResolver::from_semantic(symbolic, HashSet::from([a_type.clone(), b_type.clone()]));
    assert_eq!(resolver.resolve_type_name(&mod_a, "BType", &[]), Some(b_type.clone()));
    assert_eq!(resolver.resolve_type_name(&mod_b, "AType", &[]), Some(a_type));
}
