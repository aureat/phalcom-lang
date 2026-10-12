use crate::semantic::incremental::support::{multi_module_input, single_module_input};
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::linker::{ImportBindingId, LinkedReadSpec, SymbolId};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::session::SemanticWorkspaceSession;
use std::sync::Arc;

fn module(name: &str) -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]),
    )
}

#[test]
fn direct_static_array_subscript_and_named_replacement_are_diagnosed() {
    let main = module("main");
    let update = SemanticWorkspaceSession::new().update(single_module_input(
        main.clone(),
        "const values: Array<Int> = <Int>[1, 2]\nvalues[0] = 3\nvalues.at(1, put: 4)\n",
        1,
    )).expect("semantic test update should succeed");
    let diagnostics = update.snapshot.diagnostics.get(&main).map(|items| items.as_ref()).unwrap_or_default();
    let mutations = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == DiagnosticCode::StaticObjectMutation)
        .collect::<Vec<_>>();
    assert_eq!(mutations.len(), 2, "both direct mutation forms must identify const-object mutation: {:?}", update.snapshot.diagnostics);
    assert!(mutations.iter().all(|diagnostic| !diagnostic.message.contains("let mut")));
}

#[test]
fn imported_static_array_binding_keeps_mutation_provenance() {
    let provider = module("provider");
    let consumer = module("consumer");
    let mut input = multi_module_input(
        vec![
            (provider.clone(), "const values: Array<Int> = <Int>[1, 2]\n".to_owned()),
            (consumer.clone(), "from .provider import values\nvalues[0] = 3\n".to_owned()),
        ],
        1,
    );
    let mut linked = (*input.linked).clone();
    let consumer_module = linked.modules.get_mut(&consumer).unwrap();
    consumer_module.bindings.imports.insert("values".into(), ImportBindingId(0));
    consumer_module.linked_reads.push(LinkedReadSpec::Binding(SymbolId {
        module: provider,
        name: "values".into(),
    }));
    input.linked = Arc::new(linked);

    let update = SemanticWorkspaceSession::new().update(input).expect("semantic test update should succeed");
    let diagnostics = update.snapshot.diagnostics.get(&consumer).expect("consumer diagnostics");
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::StaticObjectMutation),
        "an imported const binding must keep its static phase: {:?}",
        diagnostics
    );
}

#[test]
fn ordinary_array_bindings_remain_mutable_without_const_object_diagnostics() {
    let main = module("main");
    let update = SemanticWorkspaceSession::new().update(single_module_input(
        main.clone(),
        "let values: Array<Int> = <Int>[1, 2]\nvalues[0] = 3\nvalues.at(1, put: 4)\nlet mut changing: Array<Int> = <Int>[5]\nchanging[0] = 6\n",
        1,
    )).expect("semantic test update should succeed");
    let diagnostics = update.snapshot.diagnostics.get(&main).map(|items| items.as_ref()).unwrap_or_default();
    assert!(
        diagnostics.iter().all(|diagnostic| diagnostic.code != DiagnosticCode::StaticObjectMutation),
        "ordinary Array bindings must not be mistaken for static constants: {:?}",
        diagnostics
    );
    assert!(!update.snapshot.has_errors(), "ordinary Array bindings retain element replacement: {diagnostics:?}");
}
