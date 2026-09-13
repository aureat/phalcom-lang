use std::sync::Arc;

use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::identity::{CallableId, DataComponentId, DeclarationId, DispatchSide};
use phalcom_semantic::{SemanticTargetId, analyze_single_module};

fn source_index_for(source: &str) -> (ModuleId, phalcom_semantic::workspace::SemanticAnalysis) {
    let module = ModuleId::universe_root();
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed.program));
    (module, analysis)
}

fn declaration_component_target(
    module: &ModuleId,
    analysis: &phalcom_semantic::workspace::SemanticAnalysis,
    source: &str,
    data_name: &str,
    component: &str,
    index: u32,
) -> SemanticTargetId {
    let owner = DeclarationId::new(module.clone(), data_name.into());
    let expected = SemanticTargetId::DataComponent(DataComponentId::new(owner, index));
    let range = source.find(component).expect("component declaration");
    let range = (range..range + component.len()).into();
    let occurrence = analysis
        .snapshot
        .source_index()
        .module(module)
        .expect("module source index")
        .occurrences
        .all()
        .iter()
        .find(|occurrence| occurrence.range == range && occurrence.role == phalcom_semantic::OccurrenceRole::Declaration)
        .unwrap_or_else(|| panic!("declaration occurrence for {component} should be indexed"));
    assert_eq!(analysis.snapshot.source_index().target_for(&occurrence.site), Some(&expected));
    expected
}

#[test]
fn tuple_data_components_publish_canonical_source_targets() {
    let source = "data Point(_ x: Int, _ y: Int)\n";
    let (module, analysis) = source_index_for(source);
    let x = declaration_component_target(&module, &analysis, source, "Point", "x", 0);
    let y = declaration_component_target(&module, &analysis, source, "Point", "y", 1);
    assert_eq!(
        x,
        SemanticTargetId::DataComponent(DataComponentId::new(DeclarationId::new(module.clone(), "Point".into()), 0))
    );
    assert_eq!(
        y,
        SemanticTargetId::DataComponent(DataComponentId::new(DeclarationId::new(module, "Point".into()), 1))
    );
}

#[test]
fn record_data_components_publish_canonical_source_targets() {
    let source = "data Point { x: Int, y: Int }\n";
    let (module, analysis) = source_index_for(source);
    declaration_component_target(&module, &analysis, source, "Point", "x", 0);
    declaration_component_target(&module, &analysis, source, "Point", "y", 1);
}

#[test]
fn projected_data_component_use_targets_the_declaration_component_identity() {
    let source = "data Point(_ x: Int, _ y: Int)\nclass Reader { read(_ p: Point) { p.x } }\n";
    let (module, analysis) = source_index_for(source);
    let expected = declaration_component_target(&module, &analysis, source, "Point", "x", 0);
    let use_start = source.rfind(".x").expect("component projection") + 1;
    let use_range = (use_start..use_start + 1).into();
    let module_index = analysis.snapshot.source_index().module(&module).expect("module source index");
    let use_occurrence = module_index
        .occurrences
        .all()
        .iter()
        .find(|occurrence| occurrence.range == use_range && occurrence.kind == phalcom_semantic::OccurrenceKind::Field)
        .expect("projected component occurrence");
    assert_eq!(module_index.occurrences.target_for(&use_occurrence.site), Some(&expected));

    let reader = DeclarationId::new(module.clone(), "Reader".into());
    let read = CallableId::new(
        reader,
        Selector::method("read", [SelectorSlot::Positional]).expect("read selector"),
        DispatchSide::Instance,
    );
    let attachment = module_index.attachments.get(&read).expect("Reader.read source attachment");
    assert!(attachment.exact_targets.values().any(|target| target == &expected));
}
