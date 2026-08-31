use std::sync::Arc;

use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;

#[test]
fn associated_expression_attaches_to_one_formal_source_site() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
enum State {
  @variant Ready
}
class Probe {
  @class run() { State::Ready }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let start = source.find("State::Ready").expect("associated expression");
    let site = analysis.snapshot.formal_fact_at(&module, start).expect("formal source site");

    assert_eq!(site.range.start, start);
    assert_eq!(site.range.end, start + "State".len());
}
