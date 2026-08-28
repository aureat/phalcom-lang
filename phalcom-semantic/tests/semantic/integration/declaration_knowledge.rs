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
    let callable = CallableId::new(
        DeclarationId::new(module, "Probe".into()),
        Selector::method("run", vec![SelectorSlot::Positional]).expect("selector"),
        DispatchSide::Instance,
    );

    let signature = analysis
        .snapshot
        .callable_signatures
        .get(&callable)
        .expect("a valid callable declaration must publish a canonical signature even when its return type is unknown");
    assert_eq!(signature.parameter_count(), 1);
}
