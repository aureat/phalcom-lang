use super::vm_support;
use phalcom_core::value::Value;

fn as_bool(value: Value) -> bool {
    value.as_bool().unwrap_or(false)
}

#[test]
fn bijection_construction_and_lookup() {
    let source = r#"
let b0 = Bijection<String, Int>.new()
let b0Empty = b0.size == 0 and b0.isEmpty and b0.get("one").isNone and b0.keyFor(1).isNone

let b1 = b0.associating("one", 1)
let b1Props = b1.size == 1 and not b1.isEmpty and b1.get("one").unwrapOr(0) == 1 and b1.keyFor(1).unwrapOr("") == "one"
let b0Unchanged = b0.size == 0 and b0.isEmpty

let b2 = b1.associating("two", 2)
let b2Props = b2.size == 2 and b2.get("two").unwrapOr(0) == 2 and b2.keyFor(2).unwrapOr("") == "two"
let b1Unchanged = b1.size == 1 and b1.get("two").isNone

let containsOk = b2.containsKey("one") and b2.containsValue(2) and not b2.containsKey("three") and not b2.containsValue(3)

let passed = b0Empty and b1Props and b0Unchanged and b2Props and b1Unchanged and containsOk
"#;
    let (vm, module) = vm_support::run_inline(source).expect("Bijection construction and lookup test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "Bijection maintains forward and reverse mappings immutably");
}

#[test]
fn bijection_conflict_detection_and_idempotence() {
    let source = r#"
let b0 = Bijection<String, Int>.new().associating("alpha", 1).associating("beta", 2)

let keyConflict = || { b0.associating("alpha", 999) }.on(Error) |_| { true }
let valConflict = || { b0.associating("gamma", 1) }.on(Error) |_| { true }

let idempotent = b0.associating("alpha", 1)
let idempotentOk = idempotent.size == 2 and idempotent.get("alpha").unwrapOr(0) == 1

let passed = keyConflict == true and valConflict == true and idempotentOk
"#;
    let (vm, module) = vm_support::run_inline(source).expect("Bijection conflict detection test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "Bijection detects key/value mapping conflicts and permits idempotent association");
}

#[test]
fn bijection_inverse_symmetry() {
    let source = r#"
let b = Bijection<String, Int>.new().associating("k1", 10).associating("k2", 20)
let inv = b.inverse

let invSize = inv.size == 2
let invForward = inv.get(10).unwrapOr("") == "k1" and inv.get(20).unwrapOr("") == "k2"
let invReverse = inv.keyFor("k1").unwrapOr(0) == 10 and inv.keyFor("k2").unwrapOr(0) == 20

let roundTrip = (inv.inverse == b) == true

let passed = invSize and invForward and invReverse and roundTrip
"#;
    let (vm, module) = vm_support::run_inline(source).expect("Bijection inverse symmetry test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "Bijection inverse provides O(1) symmetric dual mapping and round-trip equality");
}

#[test]
fn bijection_removals() {
    let source = r#"
let b0 = Bijection<String, Int>.new().associating("a", 1).associating("b", 2)

let b1 = b0.removingKey("a")
let remKeyOk = b1.size == 1 and b1.get("a").isNone and b1.keyFor(1).isNone and b1.get("b").unwrapOr(0) == 2

let b2 = b0.removingValue(2)
let remValOk = b2.size == 1 and b2.get("b").isNone and b2.keyFor(2).isNone and b2.get("a").unwrapOr(0) == 1

let noOpKey = b0.removingKey("nonexistent")
let noOpKeyOk = noOpKey.size == 2 and noOpKey.get("a").unwrapOr(0) == 1

let noOpVal = b0.removingValue(999)
let noOpValOk = noOpVal.size == 2 and noOpVal.get("a").unwrapOr(0) == 1

let passed = remKeyOk and remValOk and noOpKeyOk and noOpValOk
"#;
    let (vm, module) = vm_support::run_inline(source).expect("Bijection removals test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "Bijection removingKey and removingValue preserve symmetrical invariants");
}

#[test]
fn bijection_equality_and_hash() {
    let source = r#"
let b1 = Bijection<String, Int>.new().associating("x", 100).associating("y", 200)
let b2 = Bijection<String, Int>.new().associating("y", 200).associating("x", 100)
let b3 = Bijection<String, Int>.new().associating("x", 100).associating("y", 300)

let eqCommutative = (b1 == b2) == true
let hashMatches = b1.hash == b2.hash
let diffNotEq = (b1 == b3) == false

let passed = eqCommutative and hashMatches and diffNotEq
"#;
    let (vm, module) = vm_support::run_inline(source).expect("Bijection equality and hash test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "Bijection satisfies order-independent equality and hash law");
}

#[test]
fn bijection_iteration() {
    let source = r#"
let b = Bijection<String, Int>.new().associating("first", 1).associating("second", 2)
let it = b.iterator

let mut count = 0
let mut step = it.next
while step.isSome {
  count = count + 1
  step = it.next
}

let passed = count == 2
"#;
    let (vm, module) = vm_support::run_inline(source).expect("Bijection iteration test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "Bijection iterator yields all entries");
}
