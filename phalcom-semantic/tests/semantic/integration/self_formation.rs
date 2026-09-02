use std::sync::Arc;

use phalcom_ast::parse;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::types::parameter::{SelfTypeTerm, TypeTerm};
use phalcom_semantic::types::store::TypeData;
use phalcom_semantic::{analyze_single_module, DeclaredTypeState};

#[test]
fn source_self_signature_records_owner_and_dispatch_side() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Base {
    instance() -> Self { ... }

    @class
    class_side() -> Self { ... }
}

class Derived is Base {}
"#,
    );
    let parsed = parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    assert!(!analysis.snapshot.has_errors(), "diagnostics: {:?}", analysis.snapshot.diagnostics);

    let owner = DeclarationId::new(module.clone(), "Base".into());
    let derived = DeclarationId::new(module, "Derived".into());
    for (name, side) in [("instance", DispatchSide::Instance), ("class_side", DispatchSide::Class)] {
        let callable = CallableId::new(owner.clone(), Selector::method(name, []).unwrap(), side);
        let signature = analysis.snapshot.callable_signatures.get(&callable).expect("source signature");
        let DeclaredTypeState::Known(term) = &signature.declared_return.state else {
            panic!("expected known Self return for {name}");
        };
        let TypeTerm::Canonical(ty) = term else {
            panic!("expected canonical Self type");
        };
        let TypeData::SelfType(SelfTypeTerm {
            owner: actual_owner,
            side: actual_side,
            ..
        }) = analysis.snapshot.store.get(*ty)
        else {
            panic!("expected Self type in {name} signature");
        };
        assert_eq!(actual_owner, &owner);
        assert_eq!(actual_side, &side);
    }

    let inherited = analysis.snapshot.dispatch.resolve_callable_id(
        &*analysis.snapshot.hierarchy,
        &derived,
        DispatchSide::Instance,
        &Selector::method("instance", []).unwrap(),
    );
    assert_eq!(
        inherited,
        Some(CallableId::new(owner, Selector::method("instance", []).unwrap(), DispatchSide::Instance))
    );
}

#[test]
fn declaration_generics_are_instance_only_for_member_annotations() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Box<T> {
    @constructor
    new(value: T) {}

    value() -> T { ... }

    @class
    class_value() -> T { ... }
}
"#,
    );
    let parsed = parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    assert!(analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::AnnotationUnresolved));

    let owner = DeclarationId::new(module, "Box".into());
    let instance = analysis
        .snapshot
        .callable_signatures
        .get(&CallableId::new(owner.clone(), Selector::method("value", []).unwrap(), DispatchSide::Instance))
        .expect("instance signature");
    let DeclaredTypeState::Known(TypeTerm::Canonical(instance_ty)) = &instance.declared_return.state else {
        panic!("instance member should resolve ambient T");
    };
    assert!(matches!(analysis.snapshot.store.get(*instance_ty), TypeData::Parameter(_)));

    let class_side = analysis
        .snapshot
        .callable_signatures
        .get(&CallableId::new(
            owner.clone(),
            Selector::method("class_value", []).unwrap(),
            DispatchSide::Class,
        ))
        .expect("class-side signature");
    assert!(matches!(class_side.declared_return.state, DeclaredTypeState::Unknown(_)));

    let constructor = analysis
        .snapshot
        .callable_signatures
        .iter()
        .map(|(_, signature)| signature)
        .find(|signature| {
            signature.owner == owner
                && signature.side == DispatchSide::Class
                && matches!(&signature.selector.base, phalcom_common::selector::SelectorBase::Named(name) if name == "new")
        })
        .expect("constructor signature");
    let DeclaredTypeState::Known(TypeTerm::Canonical(constructor_parameter_ty)) =
        &constructor.parameter_declared_type_at(0).expect("constructor parameter").state
    else {
        panic!("constructor should resolve owner generic parameter");
    };
    assert!(matches!(analysis.snapshot.store.get(*constructor_parameter_ty), TypeData::Parameter(_)));
    let DeclaredTypeState::Known(TypeTerm::Canonical(constructor_ty)) = &constructor.declared_return.state else {
        panic!("constructor should publish Self return");
    };
    assert!(matches!(analysis.snapshot.store.get(*constructor_ty), TypeData::SelfType(term) if term.owner == owner));
}
