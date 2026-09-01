use phalcom_ast::parse;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::{analyze_single_module, checker::CallableAnalysisStatus};
use std::sync::Arc;

#[test]
fn test_constructor_self_type_specialization_and_counterexample() {
    let source_text = r#"
class Base {
  @constructor
  new() {}

  @class
  ordinary() -> Base {
    Base.new()
  }
}

class Derived is Base {}

class Client {
  @class
  test() {
    let b = Base.new()
    let d = Derived.new()
    let o = Derived.ordinary()
  }
}
"#;

    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(source_text);
    let parse_res = parse(&source, 0);
    let program = Arc::new(parse_res.program);

    let analysis = analyze_single_module(module.clone(), source, program);
    assert!(!analysis.snapshot.has_errors(), "Diagnostics: {:?}", analysis.snapshot.diagnostics);

    let base_decl = DeclarationId::new(module.clone(), "Base".into());
    let derived_decl = DeclarationId::new(module.clone(), "Derived".into());
    let client_decl = DeclarationId::new(module.clone(), "Client".into());

    let base_form = analysis.snapshot.declarations.form(&base_decl).expect("Base form");
    let derived_form = analysis.snapshot.declarations.form(&derived_decl).expect("Derived form");

    // Client.test analysis
    let test_sel = Selector::method("test", vec![]).unwrap();
    let test_cid = CallableId::new(client_decl, test_sel, DispatchSide::Class);
    let test_analysis = analysis.snapshot.callable_analyses.get(&test_cid).expect("Client.test analysis exists");
    assert_eq!(test_analysis.status, CallableAnalysisStatus::Complete);

    let b_binding = test_analysis.bindings.values().find(|b| b.name == "b").expect("b binding");
    let d_binding = test_analysis.bindings.values().find(|b| b.name == "d").expect("d binding");
    let o_binding = test_analysis.bindings.values().find(|b| b.name == "o").expect("o binding");

    // Section 17.5 assertions:
    // Base.new() -> Base
    assert_eq!(b_binding.current.ty(), Some(base_form), "Base.new() must return Base");
    // Derived.new() -> Derived (inherited constructor returns specialized Self)
    assert_eq!(d_binding.current.ty(), Some(derived_form), "Derived.new() must return Derived");
    // Derived.ordinary() -> Base (ordinary method with explicit Base return must NOT be rewritten)
    assert_eq!(o_binding.current.ty(), Some(base_form), "Derived.ordinary() counterexample must return Base");
}
