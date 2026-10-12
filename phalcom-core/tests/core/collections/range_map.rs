use super::vm_support;
use phalcom_core::value::Value;

fn as_bool(value: Value) -> bool {
    value.as_bool().unwrap_or(false)
}

#[test]
fn range_map_construction_and_lookup() {
    let source = r#"
let m0 = RangeMap<Int, String>.new()
let m0Empty = m0.size == 0 and m0.isEmpty and m0.get(10).isNone and not m0.containsPoint(10)

let m1 = m0.inserting(Interval<Int>.closed(10, 20), "first")
let m1Props = m1.size == 1 and not m1.isEmpty and m1.get(10).unwrapOr("") == "first" and m1.get(15).unwrapOr("") == "first" and m1.get(20).unwrapOr("") == "first"
let m1Outside = m1.get(9).isNone and m1.get(21).isNone and not m1.containsPoint(5) and m1.containsPoint(15)
let m0Unchanged = m0.size == 0 and m0.isEmpty

let m2 = m1.inserting(Interval<Int>.closed(30, 40), "second")
let m2Props = m2.size == 2 and m2.get(15).unwrapOr("") == "first" and m2.get(35).unwrapOr("") == "second"
let m2Outside = m2.get(25).isNone and m2.get(45).isNone
let m1Unchanged = m1.size == 1 and m1.get(35).isNone

let passed = m0Empty and m1Props and m1Outside and m0Unchanged and m2Props and m2Outside and m1Unchanged
"#;
    let (vm, module) = vm_support::run_inline(source).expect("RangeMap construction and lookup compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "RangeMap maintains disjoint intervals and logarithmic point lookup immutably");
}

#[test]
fn range_map_strict_insertion_and_overlap_rejection() {
    let source = r#"
let base = RangeMap<Int, String>.new().inserting(Interval<Int>.closed(10, 20), "A").inserting(Interval<Int>.closed(30, 40), "B")

let overlapLeft = || { base.inserting(Interval<Int>.closed(5, 15), "conflict") }.on(Error) |_| { true }
let overlapRight = || { base.inserting(Interval<Int>.closed(15, 25), "conflict") }.on(Error) |_| { true }
let overlapFull = || { base.inserting(Interval<Int>.closed(5, 45), "conflict") }.on(Error) |_| { true }
let overlapInterior = || { base.inserting(Interval<Int>.closed(12, 18), "conflict") }.on(Error) |_| { true }
let invalidInterval = || { base.inserting(Interval<Int>.open(20, 10), "invalid") }.on(Error) |_| { true }
let emptyInterval = || { base.inserting(Interval<Int>.open(5, 5), "empty") }.on(Error) |_| { true }

let adjoining = base.inserting(Interval<Int>.open(20, 30), "adjoining")
let adjoiningOk = adjoining.size == 3 and adjoining.get(25).unwrapOr("") == "adjoining"

let passed = overlapLeft == true and overlapRight == true and overlapFull == true and overlapInterior == true and invalidInterval == true and emptyInterval == true and adjoiningOk
"#;
    let (vm, module) = vm_support::run_inline(source).expect("RangeMap strict insertion test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "RangeMap rejects invalid, empty, or overlapping intervals and permits adjoining intervals");
}

#[test]
fn range_map_unbounded_point_lookup() {
    let source = r#"
let m = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.lessThan(0), "negative")
    .inserting(Interval<Int>.singleton(0), "zero")
    .inserting(Interval<Int>.greaterThan(0), "positive")

let negOk = m.get(-100).unwrapOr("") == "negative" and m.get(-1).unwrapOr("") == "negative"
let zeroOk = m.get(0).unwrapOr("") == "zero"
let posOk = m.get(1).unwrapOr("") == "positive" and m.get(9999).unwrapOr("") == "positive"

let passed = m.size == 3 and negOk and zeroOk and posOk
"#;
    let (vm, module) = vm_support::run_inline(source).expect("RangeMap unbounded point lookup test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "RangeMap correctly looks up points in unbounded interval regions");
}

#[test]
fn range_map_coverage_removal() {
    let source = r#"
let base = RangeMap<Int, String>.new().inserting(Interval<Int>.closed(0, 100), "all")

// Remove middle [30, 70] -> split into [0, 30) and (70, 100]
let split = base.removing(Interval<Int>.closed(30, 70))
let splitLeftOk = split.get(10).unwrapOr("") == "all" and split.get(29).unwrapOr("") == "all"
let splitHoleOk = split.get(30).isNone and split.get(50).isNone and split.get(70).isNone
let splitRightOk = split.get(71).unwrapOr("") == "all" and split.get(100).unwrapOr("") == "all"
let baseUnchanged = base.get(50).unwrapOr("") == "all"

// Truncate left remainder: remove [-10, 15] from split -> leaves (15, 30) and (70, 100]
let truncated = split.removing(Interval<Int>.closed(-10, 15))
let truncOk = truncated.get(10).isNone and truncated.get(15).isNone and truncated.get(20).unwrapOr("") == "all"

// Remove empty interval is identity
let emptyRem = split.removing(Interval<Int>.open(5, 5))
let emptyOk = emptyRem.size == split.size

let passed = splitLeftOk and splitHoleOk and splitRightOk and baseUnchanged and truncOk and emptyOk
"#;
    let (vm, module) = vm_support::run_inline(source).expect("RangeMap coverage removal test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "RangeMap coverage removal correctly splits and truncates mapped intervals immutably");
}

#[test]
fn range_map_overlay_and_range_splicing() {
    let source = r#"
let base = RangeMap<Int, String>.new().inserting(Interval<Int>.closed(0, 50), "base")

// Overlay [20, 30] with "override"
let overlaid = base.overlaying(Interval<Int>.closed(20, 30), "override")
let leftBase = overlaid.get(10).unwrapOr("") == "base" and overlaid.get(19).unwrapOr("") == "base"
let middleOverride = overlaid.get(20).unwrapOr("") == "override" and overlaid.get(25).unwrapOr("") == "override" and overlaid.get(30).unwrapOr("") == "override"
let rightBase = overlaid.get(31).unwrapOr("") == "base" and overlaid.get(50).unwrapOr("") == "base"
let basePreserved = base.get(25).unwrapOr("") == "base"

let passed = leftBase and middleOverride and rightBase and basePreserved
"#;
    let (vm, module) = vm_support::run_inline(source).expect("RangeMap overlay test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "RangeMap overlaying splices new coverage and preserves unaffected remainders");
}

#[test]
fn range_map_normalization_and_coalescing() {
    let source = r#"
// Adjacent contiguous intervals with equal values coalesce: [0, 10] and (10, 20]
let m1 = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.closed(0, 10), "same")
    .inserting(Interval<Int>.halfOpenLeft(10, 20), "same")
let coalesced = m1.entries.size == 1 and m1.entries.at(0).interval == Interval<Int>.closed(0, 20)

// Discontiguous intervals (gap at 10): [0, 10) and (10, 20] do not coalesce
let m2 = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.halfOpenRight(0, 10), "same")
    .inserting(Interval<Int>.halfOpenLeft(10, 20), "same")
let gapNotCoalesced = m2.entries.size == 2

// Contiguous intervals with different values do not coalesce
let m3 = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.closed(0, 10), "val1")
    .inserting(Interval<Int>.halfOpenLeft(10, 20), "val2")
let diffValNotCoalesced = m3.entries.size == 2

let passed = coalesced and gapNotCoalesced and diffValNotCoalesced
"#;
    let (vm, module) = vm_support::run_inline(source).expect("RangeMap normalization and coalescing test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "RangeMap normalizes entries by coalescing contiguous intervals with equal values");
}

#[test]
fn range_map_equality_and_hash() {
    let source = r#"
// m1 built from two adjoining parts: [0, 10] -> "A" and (10, 20] -> "A" (coalesces to [0, 20])
let m1 = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.closed(0, 10), "A")
    .inserting(Interval<Int>.halfOpenLeft(10, 20), "A")

// m2 built directly: [0, 20] -> "A"
let m2 = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.closed(0, 20), "A")

// m3 has different value: [0, 20] -> "B"
let m3 = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.closed(0, 20), "B")

let eqCommutative = (m1 == m2) == true and (m2 == m1) == true
let hashMatches = m1.hash == m2.hash
let diffNotEq = (m1 == m3) == false

let passed = eqCommutative and hashMatches and diffNotEq
"#;
    let (vm, module) = vm_support::run_inline(source).expect("RangeMap equality and hash test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "RangeMap satisfies extensional order-independent equality and hash law");
}

#[test]
fn range_map_iteration() {
    let source = r#"
let m = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.closed(0, 10), "first")
    .inserting(Interval<Int>.closed(20, 30), "second")
    .inserting(Interval<Int>.closed(40, 50), "third")

let it = m.iterator
let mut count = 0
let mut step = it.next
while step.isSome {
  count = count + 1
  step = it.next
}

let passed = count == 3
"#;
    let (vm, module) = vm_support::run_inline(source).expect("RangeMap iteration test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "RangeMap iterator yields all in-order entries");
}
