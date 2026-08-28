use std::sync::Arc;

use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};

#[test]
fn partial_callable_signature_survives_unknown_return() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
class Probe {
    run(value: String) {
        ...
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "fixture must parse: {:#?}", parsed.errors);

    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let declaration = DeclarationId::new(module.clone(), "Probe".into());
    let callable = CallableId::new(
        declaration.clone(),
        Selector::method("run", vec![SelectorSlot::Positional]).expect("selector"),
        DispatchSide::Instance,
    );

    assert!(
        analysis.snapshot.sources.contains_key(&module),
        "partial callable analysis must publish the source snapshot instead of falling back to the bootstrap snapshot; sources={:?}",
        analysis.snapshot.sources.keys().collect::<Vec<_>>()
    );
    assert!(
        analysis.snapshot.surfaces.contains_key(&declaration),
        "partial callable analysis must retain its declaration surface"
    );

    let signature = analysis
        .snapshot
        .callable_signatures
        .get(&callable)
        .unwrap_or_else(|| {
            panic!(
                "a valid callable declaration must publish a canonical signature even when its return type is unknown; published={:?}",
                analysis.snapshot.callable_signatures.iter().map(|(id, _)| id).collect::<Vec<_>>()
            )
        });
    assert_eq!(signature.parameter_count(), 1);
}
