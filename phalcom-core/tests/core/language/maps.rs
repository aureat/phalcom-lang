use super::vm_support;

#[test]
fn map_literal_expansion_preserves_values_and_rejects_duplicate_keys() {
    let source = r#"
let source = Map { first: 1 }
let expanded = Map { **source, second: 2 }
let result = expanded.size == 2 and expanded[#first] == 1 and expanded[#second] == 2
"#;
    let (vm, module) = vm_support::run_inline(source).expect("nonconflicting map expansion executes");
    let result = vm.interner.find("result").expect("result symbol");
    assert_eq!(vm.heap.module(module).get(result).expect("result binding").to_string(&vm), "true");

    let duplicate_source = r#"
let source = Map { first: 1 }
let duplicate = Map { **source, first: 2 }
"#;
    let error = match vm_support::run_inline(duplicate_source) {
        Ok(_) => panic!("expanded duplicate key should be rejected"),
        Err(error) => error,
    };
    assert!(error.to_string().to_lowercase().contains("duplicate"), "unexpected expansion error: {error}");
}
