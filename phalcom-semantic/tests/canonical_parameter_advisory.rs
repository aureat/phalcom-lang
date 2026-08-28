use std::collections::BTreeMap;
use std::sync::Arc;

use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::identity::{ModuleComponent, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::LinkedModuleInterface;
use phalcom_modules::linker::{LinkedModule, LinkedProgram, ModuleBindingLayout};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use phalcom_semantic::{
    AdvisoryConfidence, AdvisoryContributionSource, AdvisoryFact, AdvisoryParameterContributions, CallableId, CallableParameterId, DeclarationId, DispatchSide,
    ModuleId, ValueShape,
};

fn test_module_id() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    )
}

fn workspace_input(module: ModuleId, source: &str) -> SemanticWorkspaceInput {
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "parser errors: {:?}", parsed.errors);
    let linked_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: module.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout::default(),
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules: BTreeMap::from([(module.clone(), linked_module)]),
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: module.clone(),
        initialization_order: vec![module.clone()],
    });
    let unit = Arc::new(ParsedModuleUnit::new(
        module.clone(),
        ModuleKind::Module,
        None,
        Arc::from(source),
        Arc::new(parsed.program),
    ));
    SemanticWorkspaceInput {
        linked,
        sources: BTreeMap::from([(module, unit)]),
        generation: 1,
    }
}

#[test]
fn advisory_parameter_contributions_use_canonical_parameter_identity() {
    let callable = CallableId::new(
        DeclarationId::new(ModuleId::core(), "Probe".into()),
        Selector::method("consume", [SelectorSlot::Positional]).unwrap(),
        DispatchSide::Instance,
    );
    let parameter = CallableParameterId::new(callable.clone(), 0);
    let fact = AdvisoryFact::new(ValueShape::Unknown, AdvisoryConfidence::Interprocedural);
    let mut contributions = AdvisoryParameterContributions::default();

    let deltas = contributions.replace_source(
        AdvisoryContributionSource::Callable(callable),
        BTreeMap::from([(parameter.clone(), fact.clone())]),
    );

    assert_eq!(contributions.get(&parameter), Some(&fact));
    assert_eq!(deltas.len(), 1);
    assert_eq!(deltas[0].slot, parameter);
}

#[test]
fn constructor_argument_transfer_uses_public_canonical_parameter_identity() {
    let module = test_module_id();
    let source = "class Product { @constructor new() { } }\nclass Vessel { @constructor new(_ value) { value } }\nconst result = Vessel.new(Product.new())\n";
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(workspace_input(module.clone(), source));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let constructor = CallableId::new(
        DeclarationId::new(module.clone(), "Vessel".into()),
        Selector::method("new", [SelectorSlot::Positional]).unwrap(),
        DispatchSide::Class,
    );
    let parameter = CallableParameterId::new(constructor, 0);
    let fact = update
        .snapshot
        .advisory()
        .parameter(&parameter)
        .expect("constructor argument fact must use the declaration-owned parameter identity");

    assert_eq!(
        fact.shape,
        ValueShape::Instance(DeclarationId::new(module, "Product".into()))
    );
}
