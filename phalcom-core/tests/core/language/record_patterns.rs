use super::vm_support;

#[test]
fn record_pattern_remainder_binds_exact_unmatched_record_fields() {
    let source = r#"
class Probe {
  @class
  run() {
    const source = { name: "Ada", age: 42, city: "Istanbul" }
    let { name, **rest } = source
    let { city, **_ } = source
    name == "Ada" and rest.class == Record and rest.size == 2 and
      city == "Istanbul" and
      rest.get(#age).match(some: |value| { value == 42 }, none: || { false }) and
      rest.get(#city).match(some: |value| { value == "Istanbul" }, none: || { false }) and
      rest.get(#name) == None
  }
}
let result = Probe.run()
"#;
    let (vm, module) = vm_support::run_inline(source).expect("Record remainder destructuring compiles and executes");
    let result = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(result).expect("result binding").to_string(&vm), "true");
}
