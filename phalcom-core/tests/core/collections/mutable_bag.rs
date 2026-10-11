use super::vm_support;
use phalcom_core::value::Value;

fn as_bool(value: Value) -> bool {
    value.as_bool().unwrap_or(false)
}

#[test]
fn mutable_bag_counts_remain_exact() {
    let source = r#"
let mut bag = MutableBag<String>.new()
bag.add("apple")
bag.add("apple", count: 4)
let countApple = bag.count("apple") == 5

bag.add("banana", count: 0)
let noBanana = bag.count("banana") == 0 and bag.distinctCount == 1

bag.add("banana", count: 2)
let totalCount7 = bag.totalCount == 7 and bag.size == 7 and bag.distinctCount == 2

bag.remove("apple", count: 2)
let apple3 = bag.count("apple") == 3

bag.remove("banana", count: 10)
let bananaSaturated = bag.count("banana") == 0 and bag.distinctCount == 1 and bag.totalCount == 3

let negAdd = || { bag.add("apple", count: -1) }.on(Error) |_| { true }
let negAddCaught = negAdd == true and bag.count("apple") == 3

let negRemove = || { bag.remove("apple", count: -5) }.on(Error) |_| { true }
let negRemoveCaught = negRemove == true and bag.count("apple") == 3

let passed = countApple and noBanana and totalCount7 and apple3 and bananaSaturated and negAddCaught and negRemoveCaught
"#;
    let (vm, module) = vm_support::run_inline(source).expect("MutableBag counts test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "MutableBag maintains exact counts and saturates removals");
}

#[test]
fn mutable_bag_collision_and_reentry() {
    let source = r#"
class BagCollisionKey {
  mut _id
  id { _id }
  @constructor
  new(_ id) { _id = id }
  hash { 55 }
  ==(_ other) { _id == other.id }
}
let A = BagCollisionKey.new("A")
let B = BagCollisionKey.new("B")

let mut bag = MutableBag<BagCollisionKey>.new()
bag.add(A, count: 3)
bag.add(B, count: 5)

let distinctTwo = bag.distinctCount == 2
let countA = bag.count(A) == 3
let countB = bag.count(B) == 5
let totalEight = bag.totalCount == 8

class ReentrantBagKey {
  mut _bag
  mut _mode
  @constructor
  new(_ bag, _ mode) { _bag = bag; _mode = mode }
  hash {
    if _mode == 1 { _bag.clear }
    0
  }
  ==(_ other) { true }
}
let mut guarded = MutableBag<ReentrantBagKey>.new()
guarded.add(ReentrantBagKey.new(guarded, 0))
let reentrantErr = || { guarded.add(ReentrantBagKey.new(guarded, 1)) }.on(Error) |e| { e.message }
let guardCaught = reentrantErr == "cannot mutate this MutableBag inside 'hash'/'==' of its own key (concurrent mutation)" and guarded.distinctCount == 1

let passed = distinctTwo and countA and countB and totalEight and guardCaught
"#;
    let (vm, module) = vm_support::run_inline(source).expect("MutableBag collision and reentry test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "MutableBag distinguishes collision keys and rejects reentrant mutation");
}

#[test]
fn mutable_bag_lazy_iteration_retains_owner() {
    let source = r#"
let mut bag = MutableBag<String>.new()
bag.add("first", count: 2)
bag.add("second", count: 1)

let it = bag.iterator
let item1 = it.next.unwrapOr("") == "first"
let item2 = it.next.unwrapOr("") == "first"
let item3 = it.next.unwrapOr("") == "second"
let itemEnd = it.next.isNone

let passed = item1 and item2 and item3 and itemEnd
"#;
    let (vm, module) = vm_support::run_inline(source).expect("MutableBag iteration test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "MutableBag iterator expands multiplicities in admission order");
}
