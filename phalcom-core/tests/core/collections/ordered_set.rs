use super::vm_support;
use phalcom_core::value::Value;

fn as_bool(value: Value) -> bool {
    value.as_bool().unwrap_or(false)
}

#[test]
fn ordered_set_distinct_hash_collisions_remain_distinct() {
    let source = r#"
class CollisionPoint {
  mut _x
  mut _y
  x { _x }
  y { _y }
  @constructor
  new(_ x, _ y) { _x = x; _y = y }
  hash { _x * 31 + _y }
  ==(_ other) { _x == other.x and _y == other.y }
}
let first = CollisionPoint.new(1, 32)
let second = CollisionPoint.new(2, 1)
let duplicate = CollisionPoint.new(1, 32)

let ordered = OrderedSet<CollisionPoint>.new()
ordered.add(first)
ordered.add(second)

let sizeTwo = ordered.size == 2
let containsFirst = ordered.contains(first)
let containsSecond = ordered.contains(second)
let containsDuplicate = ordered.contains(duplicate)
let fallback = CollisionPoint.new(0, 0)
let atFirst = ordered.at(0).unwrapOr(fallback) == first
let atSecond = ordered.at(1).unwrapOr(fallback) == second
let indexFirst = ordered.indexOf(first).unwrapOr(-1) == 0
let indexSecond = ordered.indexOf(second).unwrapOr(-1) == 1
let indexDuplicate = ordered.indexOf(duplicate).unwrapOr(-1) == 0

ordered.add(duplicate)
let sizeAfterDuplicate = ordered.size == 2
let atFirstAfterDuplicate = ordered.at(0).unwrapOr(fallback) == first

let passed = sizeTwo and containsFirst and containsSecond and containsDuplicate and atFirst and atSecond and indexFirst and indexSecond and indexDuplicate and sizeAfterDuplicate and atFirstAfterDuplicate
"#;
    let (vm, module) = vm_support::run_inline(source).expect("OrderedSet collision source compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(
        as_bool(passed),
        "OrderedSet preserves distinct colliding elements and deduplicates identical elements"
    );
}

#[test]
fn ordered_set_collision_reindex_after_mutations() {
    let source = r#"
class CollisionKey {
  mut _id
  id { _id }
  @constructor
  new(_ id) { _id = id }
  hash { 42 }
  ==(_ other) { _id == other.id }
}
let A = CollisionKey.new("A")
let B = CollisionKey.new("B")
let C = CollisionKey.new("C")
let D = CollisionKey.new("D")
let E = CollisionKey.new("E")
let fallback = CollisionKey.new("none")

let set = OrderedSet<CollisionKey>.new()
set.add(A)
set.add(B)
set.add(C)
let initOk = set.size == 3 and set.at(0).unwrapOr(fallback) == A and set.at(1).unwrapOr(fallback) == B and set.at(2).unwrapOr(fallback) == C

let moveRes = set.move(2, to: 0)
let moveOk = moveRes.isOk and set.size == 3 and set.at(0).unwrapOr(fallback) == C and set.at(1).unwrapOr(fallback) == A and set.at(2).unwrapOr(fallback) == B and set.indexOf(C).unwrapOr(-1) == 0 and set.indexOf(A).unwrapOr(-1) == 1 and set.indexOf(B).unwrapOr(-1) == 2

set.remove(A)
let removeOk = set.size == 2 and set.at(0).unwrapOr(fallback) == C and set.at(1).unwrapOr(fallback) == B and not set.contains(A) and set.indexOf(C).unwrapOr(-1) == 0 and set.indexOf(B).unwrapOr(-1) == 1

let insertRes = set.insert(D, at: 1)
let insertOk = insertRes.isOk and set.size == 3 and set.at(0).unwrapOr(fallback) == C and set.at(1).unwrapOr(fallback) == D and set.at(2).unwrapOr(fallback) == B and set.indexOf(D).unwrapOr(-1) == 1

let replaceRes = set.replaceAt(1, with: E)
let replaceOk = replaceRes.isOk and set.size == 3 and set.at(0).unwrapOr(fallback) == C and set.at(1).unwrapOr(fallback) == E and set.at(2).unwrapOr(fallback) == B and not set.contains(D) and set.contains(E) and set.indexOf(E).unwrapOr(-1) == 1

let passed = initOk and moveOk and removeOk and insertOk and replaceOk
"#;
    let (vm, module) = vm_support::run_inline(source).expect("OrderedSet structural mutation source compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "OrderedSet structural mutations preserve bucket and slot coherence");
}

#[test]
fn ordered_set_duplicate_replace_is_atomic() {
    let source = r#"
class CollisionKey {
  mut _id
  id { _id }
  @constructor
  new(_ id) { _id = id }
  hash { 42 }
  ==(_ other) { _id == other.id }
}
let A = CollisionKey.new("A")
let B = CollisionKey.new("B")
let fallback = CollisionKey.new("none")

let set = OrderedSet<CollisionKey>.new()
set.add(A)
set.add(B)

let dupReplace = set.replaceAt(0, with: B)
let dupFailed = match dupReplace {
  Result::Ok(_) => false
  Result::Error(error) => error is ArgumentError
}
let statePreserved = set.size == 2 and set.at(0).unwrapOr(fallback) == A and set.at(1).unwrapOr(fallback) == B and set.indexOf(A).unwrapOr(-1) == 0 and set.indexOf(B).unwrapOr(-1) == 1

let sameReplace = set.replaceAt(0, with: A)
let sameOk = sameReplace.isOk and set.size == 2 and set.at(0).unwrapOr(fallback) == A

let passed = dupFailed and statePreserved and sameOk
"#;
    let (vm, module) = vm_support::run_inline(source).expect("OrderedSet duplicate replace compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "OrderedSet duplicate replacement is atomic and preserves receiver state");
}

#[test]
fn ordered_set_numeric_key_equivalence() {
    let source = r#"
let numSet = OrderedSet<Float>.new()
numSet.add(0.0)
numSet.add(-0.0)
let zeroDeduplicated = numSet.size == 1 and numSet.contains(0.0) and numSet.contains(-0.0)
let passed = zeroDeduplicated
"#;
    let (vm, module) = vm_support::run_inline(source).expect("OrderedSet numeric equivalence compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "OrderedSet follows numeric SameValueZero key equivalence");
}

#[test]
fn ordered_set_snapshot_equality_survives_mutation() {
    let source = r#"
class SnapshotKey {
  mut _left
  mut _right
  @constructor
  new(_ left, _ right) { _left = left; _right = right }
  hash { 0 }
  ==(_ other) { _left.clear; _right.clear; true }
}
let mut left = OrderedSet<SnapshotKey>.new()
let mut right = OrderedSet<SnapshotKey>.new()
left.add(SnapshotKey.new(left, right))
right.add(SnapshotKey.new(left, right))
let passed = left == right and left.size == 0 and right.size == 0
"#;
    let (vm, module) = vm_support::run_inline(source).expect("OrderedSet snapshot equality compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "OrderedSet equality uses GC-rooted snapshots surviving live mutation");
}
