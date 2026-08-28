use std::sync::Arc;

use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, FieldId};
use phalcom_semantic::{DeclaredTypeBasis, DeclaredTypeState};

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
        Selector::method("run", vec![SelectorSlot::Label("value".into())]).expect("selector"),
        DispatchSide::Instance,
    );

    assert!(analysis.snapshot.sources.contains_key(&module));
    assert!(analysis.snapshot.surfaces.contains_key(&declaration));

    let signature = analysis
        .snapshot
        .callable_signatures
        .get(&callable)
        .expect("a valid callable declaration must publish a canonical signature even when its return type is unknown");
    assert_eq!(signature.parameter_count(), 1);
}

#[test]
fn source_fields_publish_canonical_field_signatures() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
class Probe {
    value: String

    @class
    const shared
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "fixture must parse: {:#?}", parsed.errors);

    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let owner = DeclarationId::new(module, "Probe".into());
    let value_id = FieldId::new(owner.clone(), "value", DispatchSide::Instance);
    let shared_id = FieldId::new(owner, "shared", DispatchSide::Class);

    let value = analysis
        .snapshot
        .field_signatures
        .get(&value_id)
        .expect("annotated instance field must publish a canonical field signature");
    assert!(value.mutable);
    assert_eq!(value.declared_type.basis, DeclaredTypeBasis::SourceAnnotation);
    assert!(matches!(value.declared_type.state, DeclaredTypeState::Known(_)));

    let shared = analysis
        .snapshot
        .field_signatures
        .get(&shared_id)
        .expect("unannotated class field must still publish a partial canonical field signature");
    assert!(!shared.mutable);
    assert_eq!(shared.declared_type.basis, DeclaredTypeBasis::Unspecified);
    assert!(matches!(shared.declared_type.state, DeclaredTypeState::Unknown(_)));
}
