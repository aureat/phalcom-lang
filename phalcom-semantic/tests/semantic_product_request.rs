use phalcom_modules::authority::ModuleAuthorityMap;
use phalcom_modules::identity::{ModuleComponent, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{InterfaceBuilder, LinkedModuleInterface};
use phalcom_modules::linker::{LinkedModule, LinkedProgram, ModuleBindingLayout, ModuleLinker, SymbolId};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::request::SemanticProductRequest;
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use phalcom_semantic::{ModuleId, identity::DeclarationId};
use std::collections::BTreeMap;
use std::sync::Arc;

fn request_input(module: ModuleId, generation: u64) -> SemanticWorkspaceInput {
    request_input_with_source(module, "class Box { value() -> Int { return 1 } }\nlet top = 1\n", generation)
}

fn request_input_with_source(module: ModuleId, source_text: &str, generation: u64) -> SemanticWorkspaceInput {
    let parsed = phalcom_ast::parse(source_text, 0);
    assert!(parsed.errors.is_empty(), "parser errors: {:?}", parsed.errors);
    let linked_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: module.clone(),
            kind: ModuleKind::Module,
            exports: ModuleAuthorityMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout::default(),
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: ModuleAuthorityMap::from_source(BTreeMap::from([(module.clone(), linked_module)])),
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: module.clone(),
        initialization_order: vec![module.clone()],
    });
    let source = Arc::new(ParsedModuleUnit::new(
        module.clone(),
        ModuleKind::Module,
        None,
        Arc::from(source_text),
        Arc::new(parsed.program),
    ));
    SemanticWorkspaceInput::new(linked, BTreeMap::from([(module, source)]), generation)
}

fn module_id() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    )
}

fn named_module(name: &str) -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(808),
        ModulePath::from_components(vec![ModuleComponent::from_identifier(name).expect("valid module name")]),
    )
}

fn universe_declaration(path: &[&str], name: &str) -> DeclarationId {
    let components = path
        .iter()
        .map(|component| phalcom_modules::identity::ModuleComponent::from_identifier(component).unwrap())
        .collect::<Vec<_>>();
    DeclarationId::new(ModuleId::universe(ModulePath::from_components(components)), name.into())
}

#[test]
fn canonical_compiler_image_request_skips_optional_products_and_keeps_required_semantics() {
    let module = module_id();

    let mut full_session = SemanticWorkspaceSession::new();
    let full = full_session.update(request_input(module.clone(), 1)).expect("full session update");
    assert!(full.snapshot.source_index().module(&module).is_some());
    assert!(full.snapshot.source_index().modules().any(|(module, _)| module.project.is_universe()));
    assert!(!full.snapshot.presentation_sources.is_empty());
    assert!(!full.snapshot.advisory().modules.is_empty());
    assert!(full.stats.advisory_sources_recomputed > 0);
    for declaration in [
        universe_declaration(&["object", "class"], "Class"),
        universe_declaration(&["collections", "interval"], "Bound"),
        universe_declaration(&["collections", "interval"], "Interval"),
        universe_declaration(&["collections", "range_map"], "RangeMap"),
        universe_declaration(&["errors", "argument"], "InvalidIntervalError"),
        universe_declaration(&["errors", "argument"], "RangeOverlapError"),
    ] {
        assert!(
            full.snapshot.declarations.get(&declaration).is_some(),
            "missing baseline declaration {declaration:?}"
        );
    }

    let mut compiler_session = SemanticWorkspaceSession::new().with_product_request(SemanticProductRequest::canonical_compiler_image());
    let compiler = compiler_session
        .update(request_input(module.clone(), 2))
        .expect("compiler-image session update");
    assert!(compiler.snapshot.source_index().module(&module).is_some());
    assert_eq!(compiler.snapshot.source_index().len(), 1);
    assert!(!compiler.snapshot.formal_projection().is_empty());
    assert!(compiler.snapshot.callable_analyses.values().any(|analysis| !analysis.expressions.is_empty()));
    assert!(compiler.snapshot.presentation_sources.is_empty());
    assert!(compiler.snapshot.advisory().modules.is_empty());
    assert!(compiler.snapshot.advisory().callables.is_empty());
    assert_eq!(compiler.stats.advisory_sources_recomputed, 0);
    assert_eq!(compiler.stats.advisory_callables_recomputed, 0);
    assert!(compiler.effects.advisory_changed.is_empty());
}

#[test]
fn canonical_static_programs_are_published_only_for_compiler_image_requests() {
    let module = module_id();
    let declaration = DeclarationId::new(module.clone(), "Alias".into());
    let source = "type Alias = Int\n";

    let mut full_session = SemanticWorkspaceSession::new();
    let full = full_session
        .update(request_input_with_source(module.clone(), source, 1))
        .expect("full static-definition session update");
    let full_product = full.snapshot.static_definition(&declaration).expect("full session static definition");
    assert!(full_product.checked_body.is_some(), "full session must retain its checked source body");
    assert!(full_product.canonical_program.is_none(), "full session product shape must remain unchanged");

    let mut compiler_session = SemanticWorkspaceSession::new().with_product_request(SemanticProductRequest::canonical_compiler_image());
    let compiler = compiler_session
        .update(request_input_with_source(module, source, 2))
        .expect("compiler-image static-definition session update");
    let compiler_product = compiler.snapshot.static_definition(&declaration).expect("compiler-image static definition");
    let checked_body = compiler_product.checked_body.as_ref().expect("compiler image keeps the checked source body");
    let canonical_program = compiler_product
        .canonical_program
        .as_ref()
        .expect("compiler image publishes canonical static program");

    assert_eq!(canonical_program.owner.0, declaration);
    assert_eq!(canonical_program.input_fingerprint, checked_body.input_fingerprint);
    assert_eq!(canonical_program.module_binding_fingerprint, checked_body.module_binding_fingerprint);
    assert_eq!(canonical_program.dependencies.as_ref(), checked_body.dependencies.as_ref());
    assert_eq!(canonical_program.applications.len(), checked_body.static_applications.len());
    assert_eq!(canonical_program.result, checked_body.result);
}

#[test]
fn constructor_body_bindings_attach_to_the_canonical_source_callable() {
    use phalcom_common::selector::{Selector, SelectorSlot};
    use phalcom_semantic::identity::{CallableId, DispatchSide};

    let module = module_id();
    let owner = DeclarationId::new(module.clone(), "LineMap".into());
    let selector = Selector::method("new", [SelectorSlot::Positional]).expect("constructor selector");
    let body_callable = CallableId::new(owner.clone(), selector.clone(), DispatchSide::Instance);
    let declared_callable = CallableId::new(owner, selector, DispatchSide::Class);
    let source = "class LineMap { @constructor new(_ source: String) { let sub = source } }\n";
    let mut session = SemanticWorkspaceSession::new();
    let update = session
        .update(request_input_with_source(module.clone(), source, 1))
        .expect("constructor source-index update");

    assert_eq!(
        update.snapshot.callable_signatures.id_for_body(&body_callable),
        Some(declared_callable),
        "constructor body analysis must retain its class-side declaration identity"
    );
    let analysis = update
        .snapshot
        .callable_analyses
        .get(&body_callable)
        .expect("instance-side constructor body analysis");
    let local = analysis
        .bindings
        .values()
        .find(|binding| binding.name == "sub")
        .expect("constructor body local binding");
    let site = update
        .snapshot
        .source_index
        .source_site_for_binding(&body_callable, local.binding)
        .expect("constructor body local must have its canonical source attachment");
    assert_eq!(site.range, local.range);
}

#[test]
fn module_body_closure_bindings_attach_to_the_module_source_namespace() {
    use phalcom_common::selector::Selector;
    use phalcom_semantic::identity::{CallableId, DispatchSide};

    let module = module_id();
    let main = CallableId::new(
        DeclarationId::new(module.clone(), "<main>".into()),
        Selector::getter("<main>").expect("module body selector"),
        DispatchSide::Instance,
    );
    let source = "let parse = |text| { text }\n";
    let mut session = SemanticWorkspaceSession::new();
    let update = session
        .update(request_input_with_source(module.clone(), source, 1))
        .expect("module-body source-index update");

    let analysis = update.snapshot.callable_analyses.get(&main).expect("module body analysis");
    let local = analysis
        .bindings
        .values()
        .find(|binding| binding.name == "text")
        .expect("module-body closure parameter");
    assert!(matches!(
        analysis.binding_origins.get(&local.binding),
        Some(phalcom_semantic::checker::analysis::CallableBindingOrigin::BodyLocal)
    ));
    let site = update
        .snapshot
        .source_index
        .source_site_for_binding(&main, local.binding)
        .expect("module-body binding must attach to its module-owned source site");
    assert_eq!(site.range, local.range);
}

#[test]
fn source_and_snapshot_prelude_candidates_share_exact_identities() {
    let source = phalcom_semantic::prelude::PreludeTypeMap::build_source_canonical_universe();
    let snapshot_candidates = phalcom_semantic::prelude::PreludeTypeMap::from_declarations(|_| true);
    assert_eq!(source.entries(), snapshot_candidates.entries());
}

#[test]
fn list_dynamic_size_specialization_preserves_dynamic_boundary_and_type_control() {
    use phalcom_common::selector::{SelectorBase, SelectorKind};
    use phalcom_semantic::types::store::StaticArgument;

    let module = module_id();
    let source = r#"class Probe {
  dynamicSize(_ values: List<Dynamic>) -> Int { return values.size }
  intSize(_ values: List<Int>) -> Int { return values.size }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session
        .update(request_input_with_source(module.clone(), source, 1))
        .expect("List dynamic/type specialization session update");
    let list = universe_declaration(&["collections", "list"], "List");
    let size_arguments = update
        .snapshot
        .callable_analyses
        .values()
        .flat_map(|analysis| analysis.expressions.values())
        .filter_map(|expression| expression.call_specialization.as_ref())
        .filter(|specialization| {
            specialization.callable.try_declaration_owner() == Some(&list)
                && specialization.callable.selector.base == SelectorBase::Named("size".into())
                && specialization.callable.selector.kind == SelectorKind::Getter
        })
        .flat_map(|specialization| specialization.bindings.iter().map(|(_, argument)| *argument))
        .collect::<Vec<_>>();
    assert!(
        size_arguments.contains(&StaticArgument::DynamicType),
        "List<Dynamic>.size must retain its explicit Dynamic owner binding: {size_arguments:?}"
    );
    assert!(
        size_arguments
            .iter()
            .any(|argument| matches!(argument, StaticArgument::Term(ty) if update.snapshot.store.kind_of(*ty) == phalcom_semantic::types::id::KindId::TYPE)),
        "List<Int>.size must retain a proper type owner binding: {size_arguments:?}"
    );
}

#[test]
fn symbolic_module_import_retains_its_canonical_symbol_and_linked_slot() {
    let provider = named_module("provider");
    let consumer = named_module("consumer");
    let mut interfaces = BTreeMap::new();
    let mut sources = BTreeMap::new();
    for (id, text) in [
        (provider.clone(), "let importedValue = 7\nexport importedValue\n"),
        (consumer.clone(), "from .provider import importedValue\nlet consumed = importedValue\n"),
    ] {
        let parsed = phalcom_ast::parse(text, 0);
        assert!(parsed.errors.is_empty(), "parse errors in {id}: {:#?}", parsed.errors);
        let program = Arc::new(parsed.program);
        interfaces.insert(
            id.clone(),
            InterfaceBuilder::build(id.clone(), ModuleKind::Module, &program).expect("module interface builds"),
        );
        sources.insert(
            id.clone(),
            Arc::new(ParsedModuleUnit::new(id, ModuleKind::Module, None, Arc::from(text), program)),
        );
    }

    let semantic = Arc::new(
        ModuleLinker::new(Arc::new(ProjectUniverse::new()), interfaces)
            .link_semantic(
                consumer.clone(),
                &BTreeMap::from([((consumer.clone(), ".provider".to_owned()), provider.clone())]),
            )
            .expect("symbolic module link succeeds"),
    );
    let linked = semantic.to_unfinalized_program();
    let expected_import = linked.modules[&consumer]
        .bindings
        .imports
        .get("importedValue")
        .copied()
        .expect("source import has an executable linked slot");
    let mut session = SemanticWorkspaceSession::new();
    let snapshot = session
        .update(SemanticWorkspaceInput::new(linked, sources, 1).with_semantic_linked(semantic))
        .expect("workspace update succeeds")
        .snapshot;
    let main = phalcom_semantic::identity::CallableId::new(
        DeclarationId::new(consumer, "<main>".into()),
        phalcom_common::selector::Selector::getter("<main>").expect("module body selector"),
        phalcom_semantic::identity::DispatchSide::Instance,
    );
    let analysis = snapshot.callable_analyses.get(&main).expect("consumer module body analysis");
    let binding = analysis
        .bindings
        .values()
        .find(|binding| binding.name == "importedValue")
        .expect("imported module value is represented in the body analysis");

    assert_eq!(
        analysis.binding_origins.get(&binding.binding),
        Some(&phalcom_semantic::checker::analysis::CallableBindingOrigin::ModuleValue {
            symbol: SymbolId {
                module: provider,
                name: "importedValue".into(),
            },
            import: Some(expected_import),
        }),
        "symbolic import authority must preserve the canonical remote symbol and executable import slot"
    );
}
