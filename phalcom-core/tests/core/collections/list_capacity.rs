use phalcom_core::heap::MAX_LIST_CAPACITY;
use phalcom_core::value::Value;

use super::vm_support::run_inline;

#[test]
fn list_with_capacity_reserves_and_rejects_invalid_requests() {
    let (vm, module) = run_inline("let items: List<Int> = List<Int>.withCapacity(4)\nitems.append(11)\nlet empty: List<Int> = List<Int>.withCapacity(0)\n")
        .expect("capacity constructors compile and run");

    let items_symbol = vm.interner.find("items").expect("items symbol");
    let items = vm.heap.module(module).get(items_symbol).expect("items binding");
    let items_id = items.as_obj().expect("List is heap-backed");
    assert_eq!(vm.heap.list(items_id).elements(), &[Value::int(11)]);

    let empty_symbol = vm.interner.find("empty").expect("empty symbol");
    let empty = vm.heap.module(module).get(empty_symbol).expect("empty binding");
    let empty_id = empty.as_obj().expect("List is heap-backed");
    assert!(vm.heap.list(empty_id).is_empty());

    let negative = match run_inline("List<Int>.withCapacity(-1)\n") {
        Err(error) => error,
        Ok(_) => panic!("negative capacity must fail"),
    };
    assert!(negative.to_string().contains("List capacity must be nonnegative"));

    let source = format!("List<Int>.withCapacity({})\n", MAX_LIST_CAPACITY + 1);
    let oversized = match run_inline(&source) {
        Err(error) => error,
        Ok(_) => panic!("over-budget capacity must fail"),
    };
    assert!(oversized.to_string().contains("shared limit"), "{oversized}");
}
