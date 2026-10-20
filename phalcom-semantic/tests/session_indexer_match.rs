use phalcom_common::selector::Selector;
use phalcom_modules::authority::ModuleAuthorityMap;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::LinkedModuleInterface;
use phalcom_modules::linker::{LinkedModule, LinkedProgram, ModuleBindingLayout};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::db::QueryKey;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::BTreeMap;
use std::sync::Arc;

fn single_module_input(module: ModuleId, source: &str, generation: u64) -> SemanticWorkspaceInput {
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);

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
    let modules = BTreeMap::from([(module.clone(), linked_module)]);
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: ModuleAuthorityMap::from_source(modules),
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: module.clone(),
        initialization_order: vec![module.clone()],
    });

    let sources = BTreeMap::from([(
        module.clone(),
        Arc::new(ParsedModuleUnit::new(
            module,
            ModuleKind::Module,
            None,
            Arc::from(source),
            Arc::new(parsed.program),
        )),
    )]);
    SemanticWorkspaceInput::new(linked, sources, generation)
}

#[test]
fn class_indexer_match_publishes_callable_body_and_proven_resolution() {
    let module = ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root());
    let source = r#"
data Lookup {
  @variant Found(_ value: Int)
  @variant Missing
}

class Table {
  [_ key: Int] -> Int {
    match Lookup::Found(key) {
      Lookup::Found(value) => value
      Lookup::Missing => 0
    }
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session
        .update(single_module_input(module.clone(), source, 1))
        .expect("semantic test update should succeed");
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let callable = CallableId::new(
        DeclarationId::new(module.clone(), "Table".into()),
        Selector::subscript_get([phalcom_common::selector::SelectorSlot::Positional]).unwrap(),
        DispatchSide::Instance,
    );
    let analysis = update
        .snapshot
        .callable_analyses
        .get(&callable)
        .expect("indexer body analysis should be present");
    let (match_expression, resolution) = analysis.match_resolutions.iter().next().expect("indexer match resolution should be present");
    assert_eq!(
        resolution.exhaustiveness,
        phalcom_semantic::match_semantics::ExhaustivenessResult::Proven,
        "indexer match should retain proven coverage"
    );
    let expression = analysis.expressions.get(match_expression).expect("match expression analysis should be present");
    assert_eq!(
        source.get(expression.range.start..expression.range.end),
        Some("match Lookup::Found(key) {\n      Lookup::Found(value) => value\n      Lookup::Missing => 0\n    }")
    );

    let query_key = QueryKey::CallableBody(callable.clone());
    let changed_source = source.replace("Lookup::Missing => 0", "Lookup::Missing => 1");
    assert_ne!(changed_source, source, "the indexer body must change to test query replay");
    let refreshed = session
        .update(single_module_input(module, &changed_source, 2))
        .expect("incremental semantic update should succeed");
    assert!(
        !refreshed.snapshot.has_errors(),
        "incremental diagnostics: {:?}",
        refreshed.snapshot.diagnostics
    );
    assert_eq!(
        session.db().query_state(&query_key).and_then(|state| state.revision()),
        Some(refreshed.snapshot.id.revision()),
        "the indexer body should be selected for source-body replay"
    );
    assert!(
        refreshed.snapshot.callable_analyses.get(&callable).is_some_and(|analysis| {
            analysis
                .match_resolutions
                .values()
                .any(|resolution| resolution.exhaustiveness == phalcom_semantic::match_semantics::ExhaustivenessResult::Proven)
        }),
        "incremental replay should retain the proven indexer match"
    );
}
