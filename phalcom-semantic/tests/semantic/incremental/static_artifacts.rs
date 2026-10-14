//! Incremental static artifact and constant dependency verification (P8.T11 / G11).

use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface};
use phalcom_modules::linker::SymbolId;
use phalcom_modules::linker::{
    GlobalBindingId, ImportBindingId, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout,
};
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::{ModuleKind, ParsedModuleUnit};
use phalcom_semantic::static_constant::StaticConstantId;
use phalcom_semantic::static_value::{StaticInt, StaticValueData};
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use phalcom_semantic::SemanticWorkspaceSession;
use std::collections::BTreeMap;
use std::sync::Arc;

struct ModulePair {
    #[allow(dead_code)]
    provider: ModuleId,
    consumer: ModuleId,
}

fn artifact_input(provider_source: &str, consumer_source: &str, generation: u64) -> (SemanticWorkspaceInput, ModulePair) {
    let project = ResolvedProjectId::from_raw(88);
    let module_id = |name: &str| {
        ModuleId::resolved(
            project,
            ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]),
        )
    };
    let provider = module_id("provider");
    let consumer = module_id("consumer");
    let provider_src: Arc<str> = Arc::from(provider_source);
    let consumer_src: Arc<str> = Arc::from(consumer_source);
    let provider_prog = Arc::new(phalcom_ast::parse(&provider_src, 0).program);
    let consumer_prog = Arc::new(phalcom_ast::parse(&consumer_src, 0).program);
    let sources = BTreeMap::from([
        (
            provider.clone(),
            Arc::new(ParsedModuleUnit::new(
                provider.clone(),
                ModuleKind::Module,
                None,
                provider_src,
                provider_prog,
            )),
        ),
        (
            consumer.clone(),
            Arc::new(ParsedModuleUnit::new(
                consumer.clone(),
                ModuleKind::Module,
                None,
                consumer_src,
                consumer_prog,
            )),
        ),
    ]);
    let symbol = SymbolId {
        module: provider.clone(),
        name: "exported_const".into(),
    };
    let export = LinkedExport {
        public_name: "exported_const".into(),
        target: LinkedExportTarget::Binding(symbol.clone()),
        range: Default::default(),
    };
    let provider_linked = LinkedModule {
        interface: LinkedModuleInterface {
            module: provider.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::from([("exported_const".into(), export)]),
            metadata: Default::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::from([("exported_const".into(), GlobalBindingId(0))]),
            imports: BTreeMap::new(),
        },
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let consumer_linked = LinkedModule {
        interface: LinkedModuleInterface {
            module: consumer.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: Default::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::new(),
            imports: BTreeMap::from([("imported_const".into(), ImportBindingId(0))]),
        },
        linked_reads: vec![LinkedReadSpec::Binding(symbol)],
        runtime_dependencies: vec![provider.clone()],
    };
    let program = LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: BTreeMap::from([(provider.clone(), provider_linked), (consumer.clone(), consumer_linked)]),
        graphs: Default::default(),
        entry: consumer.clone(),
        initialization_order: vec![provider.clone(), consumer.clone()],
    };
    (
        SemanticWorkspaceInput::new(Arc::new(program), sources, generation),
        ModulePair { provider, consumer },
    )
}

#[test]
fn exported_const_value_change_invalidates_importer() {
    let mut session = SemanticWorkspaceSession::new();
    let consumer_src = "from provider import exported_const as imported_const\nconst Derived: Int = imported_const + 2\n";

    // 1. Initial provider definition: 10
    let (input1, pair) = artifact_input("const exported_const: Int = 10\nexport exported_const\n", consumer_src, 1);
    let snap1 = session.update(input1).expect("update 1").snapshot;
    let derived_id = StaticConstantId::new(pair.consumer.clone(), "Derived");
    let prod1 = snap1.static_constant(&derived_id).cloned().expect("Derived rev 1");
    assert!(prod1.is_successful());
    let val1 = snap1.static_values().get(prod1.value().unwrap()).unwrap();
    assert_eq!(val1, &StaticValueData::Int(StaticInt::Small(12)));

    // 2. Provider value changes: 10 -> 25 (type is still Int)
    let (input2, _) = artifact_input("const exported_const: Int = 25\nexport exported_const\n", consumer_src, 2);
    let snap2 = session.update(input2).expect("update 2").snapshot;
    let prod2 = snap2.static_constant(&derived_id).cloned().expect("Derived rev 2");
    assert!(prod2.is_successful());
    assert!(!Arc::ptr_eq(&prod1, &prod2), "consumer must be invalidated when provider value changes");
    let val2 = snap2.static_values().get(prod2.value().unwrap()).unwrap();
    assert_eq!(val2, &StaticValueData::Int(StaticInt::Small(27)));

    // 3. Initializer changes but reduced value unchanged: 25 -> 20 + 5
    let (input3, _) = artifact_input("const exported_const: Int = 20 + 5\nexport exported_const\n", consumer_src, 3);
    let snap3 = session.update(input3).expect("update 3").snapshot;
    let prod3 = snap3.static_constant(&derived_id).cloned().expect("Derived rev 3");
    assert!(prod3.is_successful());
    let val3 = snap3.static_values().get(prod3.value().unwrap()).unwrap();
    assert_eq!(val3, &StaticValueData::Int(StaticInt::Small(27)));
    assert_eq!(prod2.value().unwrap(), prod3.value().unwrap(), "same reduced value retains value identity");

    // 4. Change unrelated private const: importer is reused
    let (input4, _) = artifact_input("const exported_const: Int = 20 + 5\nconst private_const: Int = 999\nexport exported_const\n", consumer_src, 4);
    let snap4 = session.update(input4).expect("update 4").snapshot;
    let prod4 = snap4.static_constant(&derived_id).cloned().expect("Derived rev 4");
    assert!(prod4.is_successful());
    assert_eq!(prod3.value().unwrap(), prod4.value().unwrap());
}
