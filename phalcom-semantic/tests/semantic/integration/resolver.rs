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
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::new(),
            imports: BTreeMap::from([("pkg".into(), ImportBindingId(0))]),
        },
        linked_reads: vec![LinkedReadSpec::Module(imported.clone())],
        runtime_dependencies: vec![imported.clone()],
    };
    let linked_imported = LinkedModule {
        interface: LinkedModuleInterface {
            module: imported.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::from([(
                "Leaf".into(),
                LinkedExport {
                    public_name: "Leaf".into(),
                    target: LinkedExportTarget::Binding(SymbolId {
                        module: imported.clone(),
                        name: "Leaf".into(),
                    }),
                    range: Default::default(),
                },
            )]),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout::default(),
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: BTreeMap::from([(current.clone(), linked_current), (imported.clone(), linked_imported)]),
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
