use std::sync::Arc;

use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::{CallableId, CallableParameterId, DeclarationId, DispatchSide, analyze_single_module};

#[test]
fn callable_parameter_identity_connects_signature_binding_and_source_site() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
class ParameterProbe {
    run(value: Int) -> Int {
        value
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "fixture must parse: {:#?}", parsed.errors);
    let result = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let owner = DeclarationId::new(module.clone(), "ParameterProbe".into());
    let callable = CallableId::new(
        owner,
        Selector::method("run", [SelectorSlot::Label("value".into())]).expect("selector"),
        DispatchSide::Instance,
    );
    let signature = result.snapshot.callable_signatures.get(&callable).expect("canonical callable signature");
    let parameter = signature.parameters.first().expect("parameter");
    assert_eq!(parameter.id, CallableParameterId::new(callable.clone(), 0));
    let analysis = result.snapshot.callable_analyses.get(&callable).expect("callable analysis");
    let binding = analysis
        .bindings
        .values()
        .find(|binding| binding.parameter.as_ref() == Some(&parameter.id))
        .expect("checker binding keeps canonical parameter identity");
    let module_index = result.snapshot.source_index.modules.get(&module).expect("module source index");
    let source_info = module_index.structure.callable_sources.get(&callable).expect("callable source info");
    let parameter_site = source_info.parameter_sites.get(&parameter.id).expect("canonical parameter source site");
    assert_eq!(binding.range, module_index.structure.site(parameter_site).expect("parameter site").range);
    let attachment = module_index.attachments.get(&callable).expect("formal source attachment");
    assert_eq!(attachment.source_site_for_binding(binding.binding), Some(parameter_site));
}
