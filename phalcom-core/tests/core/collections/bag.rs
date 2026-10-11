use super::vm_support;
use phalcom_core::value::Value;

fn as_bool(value: Value) -> bool {
    value.as_bool().unwrap_or(false)
}

#[test]
fn bag_construction_and_immutability() {
    let source = r#"
let b0 = Bag<String>.new()
let b0Empty = b0.size == 0 and b0.distinctCount == 0 and b0.count("apple") == 0

let b1 = b0.adding("apple")
let b1Counts = b1.size == 1 and b1.distinctCount == 1 and b1.count("apple") == 1
let b0Unchanged = b0.size == 0 and b0.distinctCount == 0

let b2 = b1.adding("apple", count: 4)
let b2Counts = b2.size == 5 and b2.distinctCount == 1 and b2.count("apple") == 5
let b1Unchanged = b1.size == 1 and b1.count("apple") == 1

let b2Zero = b2.adding("apple", count: 0)
let b2ZeroSame = b2Zero.size == 5 and b2Zero.count("apple") == 5

let b3 = b2.adding("banana", count: 3)
let b3Counts = b3.size == 8 and b3.distinctCount == 2 and b3.count("banana") == 3 and b3.count("apple") == 5

let passed = b0Empty and b1Counts and b0Unchanged and b2Counts and b1Unchanged and b2ZeroSame and b3Counts
"#;
    let (vm, module) = vm_support::run_inline(source).expect("Bag construction and immutability test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "Bag maintains immutability and exact multiplicities across versions");
}

#[test]
fn bag_removals_and_argument_errors() {
    let source = r#"
let b0 = Bag<String>.new().adding("apple", count: 5).adding("banana", count: 3)
let b1 = b0.removing("apple", count: 2)
let appleDecreased = b1.count("apple") == 3 and b1.size == 6 and b1.distinctCount == 2

let b2 = b1.removing("banana", count: 10)
let bananaSaturated = b2.count("banana") == 0 and b2.distinctCount == 1 and b2.size == 3

let b3 = b2.removing("nonexistent")
let nonexistentNoOp = b3.size == 3 and b3.distinctCount == 1 and b3.count("apple") == 3

let negAdd = || { b0.adding("apple", count: -1) }.on(Error) |_| { true }
let negAddCaught = negAdd == true and b0.count("apple") == 5

let negRemove = || { b0.removing("apple", count: -5) }.on(Error) |_| { true }
let negRemoveCaught = negRemove == true and b0.count("apple") == 5

let passed = appleDecreased and bananaSaturated and nonexistentNoOp and negAddCaught and negRemoveCaught
"#;
    let (vm, module) = vm_support::run_inline(source).expect("Bag removals and argument errors test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "Bag correctly handles removals, saturation, and argument errors");
}

#[test]
fn bag_multiset_algebra() {
    let source = r#"
let bA = Bag<String>.new().adding("x", count: 3).adding("y", count: 2)
let bB = Bag<String>.new().adding("y", count: 4).adding("z", count: 1)

let sum = bA + bB
let sumOk = sum.count("x") == 3 and sum.count("y") == 6 and sum.count("z") == 1 and sum.size == 10 and sum.distinctCount == 3

let unionBag = bA.union(bB)
let unionOk = unionBag.count("x") == 3 and unionBag.count("y") == 4 and unionBag.count("z") == 1 and unionBag.size == 8 and unionBag.distinctCount == 3

let inter = bA.intersection(bB)
let interOk = inter.count("x") == 0 and inter.count("y") == 2 and inter.count("z") == 0 and inter.size == 2 and inter.distinctCount == 1

let diff = bA.difference(bB)
let diffOk = diff.count("x") == 3 and diff.count("y") == 0 and diff.count("z") == 0 and diff.size == 3 and diff.distinctCount == 1

let passed = sumOk and unionOk and interOk and diffOk
"#;
    let (vm, module) = vm_support::run_inline(source).expect("Bag multiset algebra test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "Bag satisfies multiset +, union, intersection, and difference algebra");
}

#[test]
fn bag_equality_and_hash() {
    let source = r#"
let b1 = Bag<String>.new().adding("a", count: 2).adding("b", count: 3)
let b2 = Bag<String>.new().adding("b", count: 3).adding("a", count: 2)
let b3 = Bag<String>.new().adding("a", count: 2).adding("b", count: 2)

let eqCommutative = (b1 == b2) == true
let hashMatches = b1.hash == b2.hash
let notEqDifferentCount = (b1 == b3) == false

let passed = eqCommutative and hashMatches and notEqDifferentCount
"#;
    let (vm, module) = vm_support::run_inline(source).expect("Bag equality and hash test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "Bag satisfies commutative equality and hash consistency");
}

#[test]
fn bag_lazy_iteration() {
    let source = r#"
let bag = Bag<String>.new().adding("first", count: 2).adding("second", count: 1)
let it = bag.iterator

let mut count = 0
let mut step = it.next
while step.isSome {
  count = count + 1
  step = it.next
}

let passed = count == 3
"#;
    let (vm, module) = vm_support::run_inline(source).expect("Bag lazy iteration test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "Bag iterator lazily yields elements with correct multiplicities");
}
