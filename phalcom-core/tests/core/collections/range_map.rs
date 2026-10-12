use super::vm_support;
use phalcom_core::heap::ObjRef;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::collections::HashSet;

fn as_bool(value: Value) -> bool {
    value.as_bool().unwrap_or(false)
}

fn global(vm: &VM, module: ObjRef, name: &str) -> Value {
    let symbol = vm.interner.find(name).unwrap_or_else(|| panic!("source global `{name}` is interned"));
    vm.heap
        .module(module)
        .get(symbol)
        .unwrap_or_else(|| panic!("source global `{name}` is initialized"))
}

fn field_slot(vm: &VM, object: ObjRef, name: &str) -> usize {
    let class = vm.heap.instance(object).class;
    let symbol = vm.interner.find(name).unwrap_or_else(|| panic!("field `{name}` is interned"));
    usize::from(
        *vm.heap
            .class(class)
            .field_slots
            .get(&symbol)
            .unwrap_or_else(|| panic!("field `{name}` exists on {}", vm.heap.class(class).name)),
    )
}

fn instance_field(vm: &VM, object: ObjRef, name: &str) -> Value {
    let slot = field_slot(vm, object, name);
    vm.heap.instance(object).slots[slot]
}

fn option_object(value: Value) -> Option<ObjRef> {
    value.unwrap_some().map(|some| some.as_obj().expect("Option::Some tree child is a node object"))
}

#[derive(Clone, Copy)]
struct RangeMapNodeSlots {
    left: usize,
    right: usize,
    height: usize,
}

fn node_slots(vm: &VM, node: ObjRef) -> RangeMapNodeSlots {
    RangeMapNodeSlots {
        left: field_slot(vm, node, "_left"),
        right: field_slot(vm, node, "_right"),
        height: field_slot(vm, node, "_height"),
    }
}

fn validate_tree_node(vm: &VM, node: ObjRef, slots: RangeMapNodeSlots, seen: &mut HashSet<ObjRef>) -> i64 {
    assert!(seen.insert(node), "RangeMap tree contains a node cycle or duplicate child reference");
    let (left_value, right_value, stored_height) = {
        let fields = &vm.heap.instance(node).slots;
        (
            fields[slots.left],
            fields[slots.right],
            fields[slots.height].as_int().expect("RangeMap node height is an Int"),
        )
    };
    let left_height = option_object(left_value).map_or(0, |left| validate_tree_node(vm, left, slots, seen));
    let right_height = option_object(right_value).map_or(0, |right| validate_tree_node(vm, right, slots, seen));
    assert!((left_height - right_height).abs() <= 1, "RangeMap AVL balance factor must stay within one");
    let expected_height = 1 + left_height.max(right_height);
    assert_eq!(stored_height, expected_height, "RangeMap node height matches child heights");
    expected_height
}

fn tree_snapshot(vm: &VM, map_value: Value) -> (Option<ObjRef>, HashSet<ObjRef>, i64) {
    let map = map_value.as_obj().expect("RangeMap value is an instance");
    let root = option_object(instance_field(vm, map, "_root"));
    let Some(root) = root else {
        return (None, HashSet::new(), 0);
    };
    let slots = node_slots(vm, root);
    let mut seen = HashSet::new();
    let height = validate_tree_node(vm, root, slots, &mut seen);
    (Some(root), seen, height)
}

fn next_range_map_random(state: &mut u64) -> u64 {
    *state = (*state).wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *state
}

fn random_range_map_interval(state: &mut u64) -> (String, Option<i64>, bool, Option<i64>, bool) {
    let first = (next_range_map_random(state) % 13) as i64 - 6;
    let second = (next_range_map_random(state) % 13) as i64 - 6;
    let (lower, upper) = if first <= second { (first, second) } else { (second, first) };
    match next_range_map_random(state) % 9 {
        0 => (format!("Interval<Int>.closed({lower}, {upper})"), Some(lower), true, Some(upper), true),
        1 => (format!("Interval<Int>.open({lower}, {upper})"), Some(lower), false, Some(upper), false),
        2 => (format!("Interval<Int>.halfOpenRight({lower}, {upper})"), Some(lower), true, Some(upper), false),
        3 => (format!("Interval<Int>.halfOpenLeft({lower}, {upper})"), Some(lower), false, Some(upper), true),
        4 => (format!("Interval<Int>.atLeast({lower})"), Some(lower), true, None, false),
        5 => (format!("Interval<Int>.greaterThan({lower})"), Some(lower), false, None, false),
        6 => (format!("Interval<Int>.atMost({upper})"), None, false, Some(upper), true),
        7 => (format!("Interval<Int>.lessThan({upper})"), None, false, Some(upper), false),
        _ => ("Interval<Int>.all".to_owned(), None, false, None, false),
    }
}

fn range_map_interval_contains(lower: Option<i64>, lower_included: bool, upper: Option<i64>, upper_included: bool, point: i64) -> bool {
    let above_lower = lower.is_none_or(|bound| point > bound || (lower_included && point == bound));
    let below_upper = upper.is_none_or(|bound| point < bound || (upper_included && point == bound));
    above_lower && below_upper
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
    assert!(
        as_bool(passed),
        "RangeMap rejects invalid, empty, or overlapping intervals and permits adjoining intervals"
    );
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
    assert!(
        as_bool(passed),
        "RangeMap coverage removal correctly splits and truncates mapped intervals immutably"
    );
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
fn range_map_boundary_splices_and_overlay_matrix() {
    let source = r#"
let base = RangeMap<Int, String>.new().inserting(Interval<Int>.closed(0, 4), "base")
let intervalEquality = Bound<Int>::Included(1) == Bound<Int>::Included(1) and Bound<Int>::Unbounded == Bound<Int>::Unbounded and Bound<Int>::Included(1) != Bound<Int>::Excluded(1) and Interval<Int>.closed(0, 4) == Interval<Int>.closed(0, 4) and Interval<Int>.closed(0, 4).hash == Interval<Int>.closed(0, 4).hash

let removeClosed = base.removing(Interval<Int>.closed(1, 3))
let closedDebug = removeClosed.toString
let closedExact = removeClosed.entries.size == 2 and removeClosed.entries.at(0).interval == Interval<Int>.halfOpenRight(0, 1) and removeClosed.entries.at(1).interval == Interval<Int>.halfOpenLeft(3, 4)

let removeOpen = base.removing(Interval<Int>.open(1, 3))
let openExact = removeOpen.entries.size == 2 and removeOpen.entries.at(0).interval == Interval<Int>.closed(0, 1) and removeOpen.entries.at(1).interval == Interval<Int>.closed(3, 4)

let removeHalfRight = base.removing(Interval<Int>.halfOpenRight(1, 3))
let halfRightExact = removeHalfRight.entries.size == 2 and removeHalfRight.entries.at(0).interval == Interval<Int>.halfOpenRight(0, 1) and removeHalfRight.entries.at(1).interval == Interval<Int>.closed(3, 4)

let removeHalfLeft = base.removing(Interval<Int>.halfOpenLeft(1, 3))
let halfLeftExact = removeHalfLeft.entries.size == 2 and removeHalfLeft.entries.at(0).interval == Interval<Int>.closed(0, 1) and removeHalfLeft.entries.at(1).interval == Interval<Int>.halfOpenLeft(3, 4)

let openBase = RangeMap<Int, String>.new().inserting(Interval<Int>.open(0, 4), "base")
let openBaseExact = openBase.removing(Interval<Int>.halfOpenRight(1, 3)).entries.size == 2 and openBase.removing(Interval<Int>.halfOpenRight(1, 3)).entries.at(0).interval == Interval<Int>.open(0, 1) and openBase.removing(Interval<Int>.halfOpenRight(1, 3)).entries.at(1).interval == Interval<Int>.halfOpenRight(3, 4)

let lowerUnbounded = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.atMost(4), "base")
    .removing(Interval<Int>.atLeast(2))
let lowerUnboundedExact = lowerUnbounded.entries.size == 1 and lowerUnbounded.entries.at(0).interval == Interval<Int>.lessThan(2)

let upperUnbounded = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.atLeast(0), "base")
    .removing(Interval<Int>.atMost(2))
let upperUnboundedExact = upperUnbounded.entries.size == 1 and upperUnbounded.entries.at(0).interval == Interval<Int>.greaterThan(2)

let allRemainders = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.all, "base")
    .removing(Interval<Int>.closed(1, 3))
let allRemaindersExact = allRemainders.entries.size == 2 and allRemainders.entries.at(0).interval == Interval<Int>.lessThan(1) and allRemainders.entries.at(1).interval == Interval<Int>.greaterThan(3)

let emptyOverlay = RangeMap<Int, String>.new().overlaying(Interval<Int>.closed(1, 3), "new")
let emptyOverlayExact = emptyOverlay.entries.size == 1 and emptyOverlay.entries.at(0).interval == Interval<Int>.closed(1, 3) and emptyOverlay.get(2).unwrapOr("") == "new"
let emptyInterval = Interval<Int>.open(2, 2)
let emptyRemoveIdentity = base.removing(emptyInterval).size == 1 and base.removing(emptyInterval).get(2).unwrapOr("") == "base"
let emptyOverlayIdentity = base.overlaying(emptyInterval, "ignored").size == 1 and base.overlaying(emptyInterval, "ignored").get(2).unwrapOr("") == "base"
let invalidRemoveRaises = || { base.removing(Interval<Int>.closed(3, 1)) }.on(Error) |_| { true }
let invalidOverlayRaises = || { base.overlaying(Interval<Int>.closed(3, 1), "invalid") }.on(Error) |_| { true }

let gapBase = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.closed(0, 1), "left")
    .inserting(Interval<Int>.closed(4, 5), "right")
let gapOverlay = gapBase.overlaying(Interval<Int>.closed(2, 3), "middle")
let gapOverlayExact = gapOverlay.size == 3 and gapOverlay.get(0).unwrapOr("") == "left" and gapOverlay.get(2).unwrapOr("") == "middle" and gapOverlay.get(5).unwrapOr("") == "right"

let exactOverlay = base.overlaying(Interval<Int>.closed(0, 4), "new")
let exactOverlayOk = exactOverlay.size == 1 and exactOverlay.get(2).unwrapOr("") == "new"

let insideOverlay = base.overlaying(Interval<Int>.closed(1, 3), "new")
let insideOverlayExact = insideOverlay.entries.size == 3 and insideOverlay.entries.at(0).interval == Interval<Int>.halfOpenRight(0, 1) and insideOverlay.entries.at(1).interval == Interval<Int>.closed(1, 3) and insideOverlay.entries.at(2).interval == Interval<Int>.halfOpenLeft(3, 4)

let leftPartial = base.overlaying(Interval<Int>.closed(-1, 1), "new")
let leftPartialExact = leftPartial.entries.size == 2 and leftPartial.entries.at(0).interval == Interval<Int>.closed(-1, 1) and leftPartial.entries.at(1).interval == Interval<Int>.halfOpenLeft(1, 4)

let rightPartial = base.overlaying(Interval<Int>.closed(3, 5), "new")
let rightPartialExact = rightPartial.entries.size == 2 and rightPartial.entries.at(0).interval == Interval<Int>.halfOpenRight(0, 3) and rightPartial.entries.at(1).interval == Interval<Int>.closed(3, 5)

let spanningBase = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.closed(0, 2), "left")
    .inserting(Interval<Int>.closed(3, 4), "middle")
    .inserting(Interval<Int>.closed(5, 7), "right")
let spanningOverlay = spanningBase.overlaying(Interval<Int>.closed(1, 6), "new")
let spanningExact = spanningOverlay.entries.size == 3 and spanningOverlay.entries.at(0).interval == Interval<Int>.halfOpenRight(0, 1) and spanningOverlay.entries.at(1).interval == Interval<Int>.closed(1, 6) and spanningOverlay.entries.at(2).interval == Interval<Int>.halfOpenLeft(6, 7) and spanningOverlay.get(3).unwrapOr("") == "new"

let lowerOverlay = base.overlaying(Interval<Int>.lessThan(2), "lower")
let lowerOverlayExact = lowerOverlay.entries.size == 2 and lowerOverlay.entries.at(0).interval == Interval<Int>.lessThan(2) and lowerOverlay.entries.at(1).interval == Interval<Int>.closed(2, 4)

let upperOverlay = base.overlaying(Interval<Int>.greaterThan(2), "upper")
let upperOverlayExact = upperOverlay.entries.size == 2 and upperOverlay.entries.at(0).interval == Interval<Int>.closed(0, 2) and upperOverlay.entries.at(1).interval == Interval<Int>.greaterThan(2)

let allOverlay = spanningBase.overlaying(Interval<Int>.all, "all")
let allOverlayExact = allOverlay.entries.size == 1 and allOverlay.entries.at(0).interval == Interval<Int>.all and allOverlay.get(100).unwrapOr("") == "all"

let adjacentBase = RangeMap<Int, String>.new().inserting(Interval<Int>.halfOpenRight(0, 1), "old")
let adjacentOverlay = adjacentBase.overlaying(Interval<Int>.closed(1, 2), "new")
let adjacentExact = adjacentOverlay.entries.size == 2 and adjacentOverlay.entries.at(0).interval == Interval<Int>.halfOpenRight(0, 1) and adjacentOverlay.entries.at(1).interval == Interval<Int>.closed(1, 2) and adjacentOverlay.get(1).unwrapOr("") == "new"

let oldRootsPreserved = base.get(2).unwrapOr("") == "base" and spanningBase.get(3).unwrapOr("") == "middle"

let passed = intervalEquality and closedExact and openExact and halfRightExact and halfLeftExact and openBaseExact and lowerUnboundedExact and upperUnboundedExact and allRemaindersExact and emptyOverlayExact and emptyRemoveIdentity and emptyOverlayIdentity and invalidRemoveRaises and invalidOverlayRaises and gapOverlayExact and exactOverlayOk and insideOverlayExact and leftPartialExact and rightPartialExact and spanningExact and lowerOverlayExact and upperOverlayExact and allOverlayExact and adjacentExact and oldRootsPreserved
"#;
    let (vm, module) = vm_support::run_inline(source).expect("RangeMap boundary and overlay matrix compiles and runs");
    let checks = [
        "intervalEquality",
        "closedExact",
        "openExact",
        "halfRightExact",
        "halfLeftExact",
        "openBaseExact",
        "lowerUnboundedExact",
        "upperUnboundedExact",
        "allRemaindersExact",
        "emptyOverlayExact",
        "emptyRemoveIdentity",
        "emptyOverlayIdentity",
        "invalidRemoveRaises",
        "invalidOverlayRaises",
        "gapOverlayExact",
        "exactOverlayOk",
        "insideOverlayExact",
        "leftPartialExact",
        "rightPartialExact",
        "spanningExact",
        "lowerOverlayExact",
        "upperOverlayExact",
        "allOverlayExact",
        "adjacentExact",
        "oldRootsPreserved",
    ];
    for check in checks {
        let symbol = vm.interner.find(check).expect("case result is interned");
        let value = vm.heap.module(module).get(symbol).expect("source publishes each case result");
        if !as_bool(value) && check == "closedExact" {
            let debug_symbol = vm.interner.find("closedDebug").expect("closed diagnostic is interned");
            let debug_value = vm.heap.module(module).get(debug_symbol).expect("source publishes the closed diagnostic");
            let debug = vm
                .heap
                .as_string(debug_value.as_obj().expect("closed diagnostic is a String"))
                .expect("closed diagnostic text is readable")
                .as_str();
            panic!("RangeMap splice case failed: {check}; actual map: {debug}");
        }
        assert!(as_bool(value), "RangeMap splice case failed: {check}");
    }
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its aggregate result");
    assert!(
        as_bool(passed),
        "RangeMap splicing preserves exact boundaries, unbounded regions, overlays, and old roots"
    );
}

#[test]
fn range_map_avl_rotations_and_structural_sharing() {
    let source = r#"
let empty = RangeMap<Int, Int>.new()

let ll1 = empty.inserting(Interval<Int>.singleton(30), 30)
let ll2 = ll1.inserting(Interval<Int>.singleton(20), 20)
let ll = ll2.inserting(Interval<Int>.singleton(10), 10)

let rr1 = empty.inserting(Interval<Int>.singleton(10), 10)
let rr2 = rr1.inserting(Interval<Int>.singleton(20), 20)
let rr = rr2.inserting(Interval<Int>.singleton(30), 30)

let lr1 = empty.inserting(Interval<Int>.singleton(30), 30)
let lr2 = lr1.inserting(Interval<Int>.singleton(10), 10)
let lr = lr2.inserting(Interval<Int>.singleton(20), 20)

let rl1 = empty.inserting(Interval<Int>.singleton(10), 10)
let rl2 = rl1.inserting(Interval<Int>.singleton(30), 30)
let rl = rl2.inserting(Interval<Int>.singleton(20), 20)

let rotationsOrdered = ll.entries.at(0).value == 10 and rr.entries.at(0).value == 10 and lr.entries.at(0).value == 10 and rl.entries.at(0).value == 10 and ll.entries.at(2).value == 30 and rr.entries.at(2).value == 30 and lr.entries.at(2).value == 30 and rl.entries.at(2).value == 30

let mut ascending = empty
let mut i = 0
while i < 31 {
    ascending = ascending.inserting(Interval<Int>.singleton(i * 2), i)
    i = i + 1
}
let largeBase = ascending
let largeAfter = largeBase.inserting(Interval<Int>.singleton(62), 31)
let largeRemoved = largeBase.removing(Interval<Int>.closed(20, 40))

let oldVersionsIntact = ll2.size == 2 and ll2.get(10).isNone and ll2.get(20).unwrapOr(-1) == 20 and largeBase.size == 31 and largeBase.get(20).unwrapOr(-1) == 10 and largeRemoved.size == 20 and largeRemoved.get(20).isNone
class ComparisonCounter {
    mut _count: Int

    @constructor
    new() { _count = 0 }

    reset() { _count = 0 }
    increment() { _count = _count + 1 }
    count -> Int { self._count }
}

let comparisonCounter = ComparisonCounter.new()
class CountedPoint {
    _value: Int

    @constructor
    new(_ value: Int) { _value = value }

    compare(_ other: CountedPoint) -> Ordering {
        comparisonCounter.increment()
        self._value <=> other.value
    }

    value -> Int { self._value }
}

let mut comparisonMap = RangeMap<CountedPoint, Int>.new()
let mut j = 0
while j < 31 {
    comparisonMap = comparisonMap.inserting(Interval<CountedPoint>.singleton(CountedPoint.new(j * 2)), j)
    j = j + 1
}
comparisonCounter.reset()
let countedLookupOk = comparisonMap.get(CountedPoint.new(60)).unwrapOr(-1) == 30
let lookupComparisons = comparisonCounter.count
comparisonCounter.reset()
let oneRemoved = comparisonMap.removing(Interval<CountedPoint>.singleton(CountedPoint.new(30)))
let singleRemovalComparisons = comparisonCounter.count
comparisonCounter.reset()
let manyRemoved = comparisonMap.removing(Interval<CountedPoint>.closed(CountedPoint.new(20), CountedPoint.new(40)))
let multiRemovalComparisons = comparisonCounter.count
let countedRemovalOk = oneRemoved.size == 30 and manyRemoved.size == 20

let passed = rotationsOrdered and oldVersionsIntact and countedLookupOk and countedRemovalOk
"#;
    let (vm, module) = vm_support::run_inline(source).expect("RangeMap AVL rotation and sharing fixture compiles and runs");
    assert!(
        as_bool(global(&vm, module, "passed")),
        "four AVL rotation fixtures and persistent versions behave correctly"
    );

    let lookup_comparisons = global(&vm, module, "lookupComparisons").as_int().expect("lookup comparison count is an Int");
    assert!(
        lookup_comparisons <= 32,
        "31-node point lookup stays within the conservative AVL comparison bound, used {lookup_comparisons}"
    );
    let single_removal_comparisons = global(&vm, module, "singleRemovalComparisons")
        .as_int()
        .expect("single-removal comparison count is an Int");
    let multi_removal_comparisons = global(&vm, module, "multiRemovalComparisons")
        .as_int()
        .expect("multi-removal comparison count is an Int");
    assert!(
        multi_removal_comparisons <= single_removal_comparisons + 16,
        "removing eleven entries does not multiply comparison work by k: one={single_removal_comparisons}, many={multi_removal_comparisons}"
    );

    for name in ["ll", "rr", "lr", "rl", "largeBase", "largeAfter", "largeRemoved"] {
        let (_, nodes, _) = tree_snapshot(&vm, global(&vm, module, name));
        let expected = match name {
            "ll" | "rr" | "lr" | "rl" => 3,
            "largeBase" => 31,
            "largeAfter" => 32,
            "largeRemoved" => 20,
            _ => unreachable!(),
        };
        assert_eq!(nodes.len(), expected, "{name} has exactly the expected number of reachable AVL nodes");
    }

    for name in ["ll", "rr", "lr", "rl"] {
        let (_, _, height) = tree_snapshot(&vm, global(&vm, module, name));
        assert_eq!(height, 2, "{name} is balanced after its single-rotation case");
    }

    let (base_root, base_nodes, base_height) = tree_snapshot(&vm, global(&vm, module, "largeBase"));
    let (after_root, after_nodes, after_height) = tree_snapshot(&vm, global(&vm, module, "largeAfter"));
    assert!(base_height <= 6 && after_height <= 6, "31 and 32 entries stay within AVL height bounds");
    let shared_after_insert = base_nodes.intersection(&after_nodes).count();
    assert!(
        shared_after_insert >= 24,
        "single insertion shares unaffected subtrees ({shared_after_insert} of 31 old nodes)"
    );
    let base_left = option_object(instance_field(&vm, base_root.expect("nonempty base tree"), "_left"));
    let after_left = option_object(instance_field(&vm, after_root.expect("nonempty updated tree"), "_left"));
    assert_eq!(base_left, after_left, "rightmost insertion retains the unaffected left subtree by identity");
    assert!(
        after_nodes.difference(&base_nodes).count() <= 8,
        "single insertion path-copies a bounded number of nodes"
    );

    let (_, removed_nodes, removed_height) = tree_snapshot(&vm, global(&vm, module, "largeRemoved"));
    assert!(removed_height <= 6, "range removal rejoins a balanced AVL tree");
    let shared_after_remove = removed_nodes.intersection(&base_nodes).count();
    assert!(
        shared_after_remove >= 10,
        "range removal retains unaffected subtrees by identity ({shared_after_remove} shared nodes)"
    );
}

#[test]
fn range_map_normalization_and_coalescing() {
    let source = r#"
// Adjacent contiguous intervals with equal values coalesce: [0, 10] and (10, 20]
let m1 = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.closed(0, 10), "same")
    .inserting(Interval<Int>.halfOpenLeft(10, 20), "same")
let mut m1Iterator = m1.iterator
let mut m1IteratorCount = 0
let mut m1Next = m1Iterator.next
while m1Next.isSome {
  m1IteratorCount = m1IteratorCount + 1
  m1Next = m1Iterator.next
}
let coalesced = m1.size == 1 and m1.entries.size == 1 and m1IteratorCount == 1 and m1.entries.at(0).interval == Interval<Int>.closed(0, 20)

// Discontiguous intervals (gap at 10): [0, 10) and (10, 20] do not coalesce
let m2 = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.halfOpenRight(0, 10), "same")
    .inserting(Interval<Int>.halfOpenLeft(10, 20), "same")
let gapNotCoalesced = m2.entries.size == 2

// Contiguous intervals with different values do not coalesce
let m3 = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.closed(0, 10), "val1")
    .inserting(Interval<Int>.halfOpenLeft(10, 20), "val2")
let diffValNotCoalesced = m3.size == 2 and m3.entries.size == 2

// Filling an isolated point coalesces both equal neighbors in storage.
let overlayBase = RangeMap<Int, String>.new()
    .inserting(Interval<Int>.halfOpenRight(0, 1), "same")
    .inserting(Interval<Int>.halfOpenLeft(1, 2), "same")
let filled = overlayBase.overlaying(Interval<Int>.singleton(1), "same")
let overlayCoalesced = overlayBase.size == 2 and filled.size == 1 and filled.entries.size == 1 and filled.entries.at(0).interval == Interval<Int>.closed(0, 2)
let removalKeepsGap = filled.removing(Interval<Int>.singleton(1)).size == 2 and filled.removing(Interval<Int>.singleton(1)).entries.size == 2

let passed = coalesced and gapNotCoalesced and diffValNotCoalesced and overlayCoalesced and removalKeepsGap
"#;
    let (vm, module) = vm_support::run_inline(source).expect("RangeMap normalization and coalescing test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(
        as_bool(passed),
        "RangeMap normalizes entries by coalescing contiguous intervals with equal values"
    );
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

let m1EqualsM2 = m1 == m2
let m2EqualsM1 = m2 == m1
let eqCommutative = m1EqualsM2 and m2EqualsM1
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

#[test]
fn range_map_gc_retains_live_versions_endpoints_values_and_iterators() {
    let source = r#"
let empty = RangeMap<Int, Int>.new()
let old = empty.inserting(Interval<Int>.singleton(1), 11)
let mut middle = old.inserting(Interval<Int>.singleton(3), 33)
let latest = middle.overlaying(Interval<Int>.singleton(3), 34).inserting(Interval<Int>.singleton(5), 55)
middle = empty

let mut objectPoint = HeapPoint.new(7)
let mut objectPayload = HeapPayload.new(91)
let objectMap = RangeMap<HeapPoint, HeapPayload>.new()
    .inserting(Interval<HeapPoint>.singleton(objectPoint), objectPayload)
objectPoint = HeapPoint.new(70)
objectPayload = HeapPayload.new(0)

let mut iterMap = RangeMap<Int, Int>.new()
    .inserting(Interval<Int>.singleton(10), 10)
    .inserting(Interval<Int>.singleton(20), 20)
    .inserting(Interval<Int>.singleton(30), 30)
let mut iterator = iterMap.iterator
iterMap = empty

System.gc
let versionsOk = old.size == 1 and old.get(1).unwrapOr(-1) == 11 and old.get(3).isNone and latest.size == 3 and latest.get(1).unwrapOr(-1) == 11 and latest.get(3).unwrapOr(-1) == 34 and latest.get(5).unwrapOr(-1) == 55
let objectOk = objectMap.get(HeapPoint.new(7)).unwrap.value == 91
let mut iteratorCount = 0
let mut next = iterator.next
while next.isSome {
    iteratorCount = iteratorCount + 1
    next = iterator.next
}
let iteratorOk = iteratorCount == 3

let mut discarded = RangeMap<Int, Int>.new()
let mut i = 0
while i < 5 {
    discarded = discarded.inserting(Interval<Int>.singleton(100 + i * 2), 100 + i)
    i = i + 1
}
discarded = empty
let passed = versionsOk and objectOk and iteratorOk
"#;
    let source = format!(
        "class HeapPoint {{\n    _value: Int\n    @constructor\n    new(_ value: Int) {{ _value = value }}\n    compare(_ other: HeapPoint) -> Ordering {{ self._value <=> other.value }}\n    value -> Int {{ self._value }}\n}}\nclass HeapPayload {{\n    _value: Int\n    @constructor\n    new(_ value: Int) {{ _value = value }}\n    value -> Int {{ self._value }}\n}}\n{source}"
    );
    let (mut vm, module) = vm_support::run_inline(&source).expect("RangeMap GC retention fixture compiles and runs");
    assert!(
        as_bool(global(&vm, module, "passed")),
        "live map versions, heap-backed endpoints and values, and iterators survive forced GC"
    );

    let old = global(&vm, module, "old");
    let latest = global(&vm, module, "latest");
    let (_, old_nodes, _) = tree_snapshot(&vm, old);
    let (_, latest_nodes, _) = tree_snapshot(&vm, latest);
    assert!(
        old_nodes.len() == 1 && latest_nodes.len() == 3,
        "retained roots still own the expected AVL nodes after source-level GC"
    );
    assert_eq!(old_nodes.len(), 1);
    assert_eq!(latest_nodes.len(), 3);

    let swept = vm.force_gc();
    assert!(
        swept >= 5,
        "nodes unique to the dropped five-entry version become reclaimable; collector swept {swept} objects"
    );
    let (_, old_nodes_after, _) = tree_snapshot(&vm, old);
    let (_, latest_nodes_after, _) = tree_snapshot(&vm, latest);
    assert_eq!(old_nodes_after, old_nodes, "the old version's nodes remain intact after reclamation");
    assert_eq!(latest_nodes_after, latest_nodes, "the latest version's nodes remain intact after reclamation");
}

#[test]
fn range_map_comparison_callbacks_reenter_allocate_and_fail_atomically() {
    let source = r#"
class CallbackObserver {
    mut _count: Int
    @constructor
    new() { _count = 0 }
    increment() { _count = _count + 1 }
    count -> Int { self._count }
}
let callbackObserver = CallbackObserver.new()
let mut sharedMap = RangeMap<CallbackPoint, Int>.new()
class CallbackPoint {
    _value: Int
    @constructor
    new(_ value: Int) { _value = value }
    compare(_ other: CallbackPoint) -> Ordering {
        let observedSize = sharedMap.size
        callbackObserver.increment()
        let allocated = List<Int>.new()
        allocated.append(observedSize)
        System.gc
        if self._value == 99 or other.value == 99 {
            InvalidIntervalError.new("comparison callback failed").raise()
        }
        if self._value == 88 or other.value == 88 {
            return Ordering::Unordered
        }
        self._value <=> other.value
    }
    value -> Int { self._value }
}

sharedMap = sharedMap.inserting(Interval<CallbackPoint>.singleton(CallbackPoint.new(1)), 11)
sharedMap = sharedMap.inserting(Interval<CallbackPoint>.singleton(CallbackPoint.new(3)), 33)
let beforeSize = sharedMap.size
let throwing = || { sharedMap.overlaying(Interval<CallbackPoint>.singleton(CallbackPoint.new(99)), 99); false }.on(Error) |_| { true }
let unordered = || { sharedMap.overlaying(Interval<CallbackPoint>.singleton(CallbackPoint.new(88)), 88); false }.on(Error) |_| { true }
let unchanged = sharedMap.size == beforeSize and sharedMap.get(CallbackPoint.new(1)).unwrapOr(-1) == 11 and sharedMap.get(CallbackPoint.new(3)).unwrapOr(-1) == 33
let passed = throwing and unordered and unchanged and callbackObserver.count > 0
"#;
    let (vm, module) = vm_support::run_inline(source).expect("RangeMap callback fixture compiles and runs");
    assert!(
        as_bool(global(&vm, module, "passed")),
        "comparison callbacks can read, allocate, trigger GC, and fail without publishing a changed root"
    );
}

#[test]
fn range_map_value_equality_callback_failure_is_atomic() {
    let source = r#"
class EqualityObserver {
    mut _count: Int
    @constructor
    new() { _count = 0 }
    increment() { _count = _count + 1 }
    count -> Int { self._count }
}
let equalityObserver = EqualityObserver.new()
class CallbackValue {
    _value: Int
    @constructor
    new(_ value: Int) { _value = value }
    ==(_ other: Dynamic) {
        if other is not CallbackValue { return false }
        let observedSize = valueMap.size
        equalityObserver.increment()
        let allocated = List<Int>.new()
        allocated.append(observedSize)
        System.gc
        if self._value == 99 or other.value == 99 {
            InvalidIntervalError.new("value equality callback failed").raise()
        }
        self._value == other.value
    }
    value -> Int { self._value }
}
let mut valueMap = RangeMap<Int, CallbackValue>.new()
valueMap = valueMap.inserting(Interval<Int>.closed(0, 1), CallbackValue.new(11))
valueMap = valueMap.inserting(Interval<Int>.halfOpenLeft(1, 2), CallbackValue.new(11))
let normalized = valueMap.size == 1 and valueMap.entries.size == 1 and valueMap.entries.at(0).value.value == 11 and valueMap.get(0).unwrap.value == 11 and valueMap.get(2).unwrap.value == 11
valueMap = valueMap.inserting(Interval<Int>.closed(10, 11), CallbackValue.new(99))
valueMap = valueMap.inserting(Interval<Int>.closed(13, 14), CallbackValue.new(12))
let beforeSize = valueMap.size
let throwingInsert = || { valueMap.inserting(Interval<Int>.open(2, 10), CallbackValue.new(99)); false }.on(Error) |_| { true }
let throwingOverlay = || { valueMap.overlaying(Interval<Int>.open(2, 10), CallbackValue.new(99)); false }.on(Error) |_| { true }
let unchanged = valueMap.size == beforeSize and valueMap.size == 3 and valueMap.get(0).unwrap.value == 11 and valueMap.get(2).unwrap.value == 11 and valueMap.get(10).unwrap.value == 99 and valueMap.get(13).unwrap.value == 12
let passed = normalized and throwingInsert and throwingOverlay and unchanged and equalityObserver.count > 0
"#;
    let (vm, module) = vm_support::run_inline(source).expect("RangeMap value equality callback fixture compiles and runs");
    assert!(
        as_bool(global(&vm, module, "passed")),
        "value equality callbacks can read, allocate, trigger GC, and throw without publishing a changed root"
    );
}

#[test]
fn range_map_randomized_model_conformance_multiple_seeds() {
    const DOMAIN_MIN: i64 = -8;
    const DOMAIN_MAX: i64 = 8;
    const EMPTY: i64 = -999;

    let mut source = String::from("let mut checks = true\n");
    let mut snapshots: Vec<(String, Vec<Option<i64>>)> = Vec::new();
    for (seed_index, seed) in [0x51a7_u64, 0xbadc0ffe_u64, 0xdeadbeef_u64].into_iter().enumerate() {
        let map_name = format!("randomMap{seed_index}");
        let forced_value = 700 + seed_index as i64;
        let mut model = vec![None; (DOMAIN_MAX - DOMAIN_MIN + 1) as usize];
        let forced_point = 7;
        model[(forced_point - DOMAIN_MIN) as usize] = Some(forced_value);
        source.push_str(&format!(
            "let mut {map_name} = RangeMap<Int, Int>.new().inserting(Interval<Int>.singleton({forced_point}), {forced_value})\n"
        ));
        let rejected_name = format!("seed{seed_index}DuplicateRejected");
        source.push_str(&format!(
            "let {rejected_name} = || {{ {map_name}.inserting(Interval<Int>.singleton({forced_point}), -1); false }}.on(Error) |_| {{ true }}\nchecks = checks and {rejected_name}\n"
        ));

        let mut state = seed;
        for operation_index in 0..350 {
            let operation = next_range_map_random(&mut state) % 5;
            if operation <= 1 {
                let (interval, lower, lower_included, upper, upper_included) = random_range_map_interval(&mut state);
                let value = (next_range_map_random(&mut state) % 1000) as i64 + 1;
                if operation == 0 {
                    source.push_str(&format!("{map_name} = {map_name}.overlaying({interval}, {value})\n"));
                    for point in DOMAIN_MIN..=DOMAIN_MAX {
                        if range_map_interval_contains(lower, lower_included, upper, upper_included, point) {
                            model[(point - DOMAIN_MIN) as usize] = Some(value);
                        }
                    }
                } else {
                    source.push_str(&format!("{map_name} = {map_name}.removing({interval})\n"));
                    for point in DOMAIN_MIN..=DOMAIN_MAX {
                        if range_map_interval_contains(lower, lower_included, upper, upper_included, point) {
                            model[(point - DOMAIN_MIN) as usize] = None;
                        }
                    }
                }
            } else if operation == 2 {
                let point = (next_range_map_random(&mut state) % (DOMAIN_MAX - DOMAIN_MIN + 1) as u64) as i64 + DOMAIN_MIN;
                let expected = model[(point - DOMAIN_MIN) as usize].unwrap_or(EMPTY);
                source.push_str(&format!("checks = checks and {map_name}.get({point}).unwrapOr({EMPTY}) == {expected}\n"));
            } else if operation == 3 {
                let point = (next_range_map_random(&mut state) % 13) as i64 - 6;
                if next_range_map_random(&mut state) % 2 == 0 {
                    source.push_str(&format!("{map_name} = {map_name}.removing(Interval<Int>.open({point}, {point}))\n"));
                } else {
                    let value = (next_range_map_random(&mut state) % 1000) as i64 + 1;
                    source.push_str(&format!("{map_name} = {map_name}.overlaying(Interval<Int>.open({point}, {point}), {value})\n"));
                }
            } else {
                let point = (next_range_map_random(&mut state) % (DOMAIN_MAX - DOMAIN_MIN + 1) as u64) as i64 + DOMAIN_MIN;
                let index = (point - DOMAIN_MIN) as usize;
                if model[index].is_some() {
                    let rejected_name = format!("seed{seed_index}Overlap{operation_index}");
                    source.push_str(&format!(
                        "let {rejected_name} = || {{ {map_name}.inserting(Interval<Int>.singleton({point}), -2); false }}.on(Error) |_| {{ true }}\nchecks = checks and {rejected_name}\n"
                    ));
                } else {
                    let value = (next_range_map_random(&mut state) % 1000) as i64 + 1;
                    source.push_str(&format!("{map_name} = {map_name}.inserting(Interval<Int>.singleton({point}), {value})\n"));
                    model[index] = Some(value);
                }
            }

            if operation_index % 100 == 99 {
                let snapshot_name = format!("seed{seed_index}Snapshot{operation_index}");
                source.push_str(&format!("let {snapshot_name} = {map_name}\n"));
                snapshots.push((snapshot_name, model.clone()));
            }
        }

        for point in DOMAIN_MIN..=DOMAIN_MAX {
            let expected = model[(point - DOMAIN_MIN) as usize].unwrap_or(EMPTY);
            source.push_str(&format!("checks = checks and {map_name}.get({point}).unwrapOr({EMPTY}) == {expected}\n"));
        }
    }

    for (snapshot_name, snapshot_model) in snapshots {
        for point in DOMAIN_MIN..=DOMAIN_MAX {
            let expected = snapshot_model[(point - DOMAIN_MIN) as usize].unwrap_or(EMPTY);
            source.push_str(&format!("checks = checks and {snapshot_name}.get({point}).unwrapOr({EMPTY}) == {expected}\n"));
        }
    }
    source.push_str("let passed = checks\n");

    let (vm, module) = vm_support::run_inline(&source).expect("RangeMap deterministic randomized model fixture compiles and runs");
    assert!(
        as_bool(global(&vm, module, "passed")),
        "1050 deterministic operations across three fixed seeds match the pointwise reference model, including retained versions"
    );
}

#[test]
fn range_map_complexity_scales_with_tree_height_and_affected_region() {
    let source = r#"
class ComplexityCounter {
    mut _count: Int
    @constructor
    new() { _count = 0 }
    reset() { _count = 0 }
    increment() { _count = _count + 1 }
    count -> Int { self._count }
}
let comparisonCounter = ComplexityCounter.new()
class ComplexityPoint {
    _value: Int
    @constructor
    new(_ value: Int) { _value = value }
    compare(_ other: ComplexityPoint) -> Ordering {
        comparisonCounter.increment()
        self._value <=> other.value
    }
    value -> Int { self._value }
}

let mut buildingMap = RangeMap<ComplexityPoint, Int>.new()
let mut map32 = RangeMap<ComplexityPoint, Int>.new()
let mut map1024 = RangeMap<ComplexityPoint, Int>.new()
let mut map32768 = RangeMap<ComplexityPoint, Int>.new()
let mut i = 0
while i < 32768 {
    buildingMap = buildingMap.inserting(Interval<ComplexityPoint>.singleton(ComplexityPoint.new(i * 2)), i)
    i = i + 1
    if i == 32 { map32 = buildingMap }
    if i == 1024 { map1024 = buildingMap }
    if i == 32768 { map32768 = buildingMap }
}

comparisonCounter.reset()
let lookup32Ok = map32.get(ComplexityPoint.new(62)).unwrapOr(-1) == 31
let lookup32Count = comparisonCounter.count
comparisonCounter.reset()
let oneRemoved32 = map32.removing(Interval<ComplexityPoint>.singleton(ComplexityPoint.new(32)))
let oneRemove32Count = comparisonCounter.count
comparisonCounter.reset()
let manyRemoved32 = map32.removing(Interval<ComplexityPoint>.closed(ComplexityPoint.new(24), ComplexityPoint.new(44)))
let manyRemove32Count = comparisonCounter.count
comparisonCounter.reset()
let oneOverlay32 = map32.overlaying(Interval<ComplexityPoint>.singleton(ComplexityPoint.new(32)), -100)
let oneOverlay32Count = comparisonCounter.count
comparisonCounter.reset()
let manyOverlay32 = map32.overlaying(Interval<ComplexityPoint>.closed(ComplexityPoint.new(24), ComplexityPoint.new(44)), -200)
let manyOverlay32Count = comparisonCounter.count

comparisonCounter.reset()
let lookup1024Ok = map1024.get(ComplexityPoint.new(2046)).unwrapOr(-1) == 1023
let lookup1024Count = comparisonCounter.count
comparisonCounter.reset()
let oneRemoved1024 = map1024.removing(Interval<ComplexityPoint>.singleton(ComplexityPoint.new(1024)))
let oneRemove1024Count = comparisonCounter.count
comparisonCounter.reset()
let manyRemoved1024 = map1024.removing(Interval<ComplexityPoint>.closed(ComplexityPoint.new(1000), ComplexityPoint.new(1020)))
let manyRemove1024Count = comparisonCounter.count
comparisonCounter.reset()
let oneOverlay1024 = map1024.overlaying(Interval<ComplexityPoint>.singleton(ComplexityPoint.new(1024)), -100)
let oneOverlay1024Count = comparisonCounter.count
comparisonCounter.reset()
let manyOverlay1024 = map1024.overlaying(Interval<ComplexityPoint>.closed(ComplexityPoint.new(1000), ComplexityPoint.new(1020)), -200)
let manyOverlay1024Count = comparisonCounter.count

comparisonCounter.reset()
let lookup32768Ok = map32768.get(ComplexityPoint.new(65534)).unwrapOr(-1) == 32767
let lookup32768Count = comparisonCounter.count
comparisonCounter.reset()
let oneRemoved32768 = map32768.removing(Interval<ComplexityPoint>.singleton(ComplexityPoint.new(32768)))
let oneRemove32768Count = comparisonCounter.count
comparisonCounter.reset()
let manyRemoved32768 = map32768.removing(Interval<ComplexityPoint>.closed(ComplexityPoint.new(32760), ComplexityPoint.new(32780)))
let manyRemove32768Count = comparisonCounter.count
comparisonCounter.reset()
let oneOverlay32768 = map32768.overlaying(Interval<ComplexityPoint>.singleton(ComplexityPoint.new(32768)), -100)
let oneOverlay32768Count = comparisonCounter.count
comparisonCounter.reset()
let manyOverlay32768 = map32768.overlaying(Interval<ComplexityPoint>.closed(ComplexityPoint.new(32760), ComplexityPoint.new(32780)), -200)
let manyOverlay32768Count = comparisonCounter.count

let passed = lookup32Ok and lookup1024Ok and lookup32768Ok and oneRemoved32.size == 31 and manyRemoved32.size == 21 and oneRemoved1024.size == 1023 and manyRemoved1024.size == 1013 and oneRemoved32768.size == 32767 and manyRemoved32768.size == 32757 and oneOverlay32.size == 32 and oneOverlay32.get(ComplexityPoint.new(32)).unwrapOr(0) == -100 and manyOverlay32.size == 22 and manyOverlay32.get(ComplexityPoint.new(24)).unwrapOr(0) == -200 and manyOverlay32.get(ComplexityPoint.new(44)).unwrapOr(0) == -200 and oneOverlay1024.size == 1024 and oneOverlay1024.get(ComplexityPoint.new(1024)).unwrapOr(0) == -100 and manyOverlay1024.size == 1014 and manyOverlay1024.get(ComplexityPoint.new(1000)).unwrapOr(0) == -200 and manyOverlay1024.get(ComplexityPoint.new(1020)).unwrapOr(0) == -200 and oneOverlay32768.size == 32768 and oneOverlay32768.get(ComplexityPoint.new(32768)).unwrapOr(0) == -100 and manyOverlay32768.size == 32758 and manyOverlay32768.get(ComplexityPoint.new(32760)).unwrapOr(0) == -200 and manyOverlay32768.get(ComplexityPoint.new(32780)).unwrapOr(0) == -200
"#;
    let (vm, module) = vm_support::run_inline(source).expect("RangeMap multi-size complexity fixture compiles and runs");
    assert!(
        as_bool(global(&vm, module, "passed")),
        "lookup and range removal produce correct results at 32, 1024, and 32768 entries"
    );

    for (name, expected_size) in [("map32", 32), ("map1024", 1024), ("map32768", 32768)] {
        let (_, nodes, height) = tree_snapshot(&vm, global(&vm, module, name));
        assert_eq!(nodes.len(), expected_size, "{name} contains the expected distinct nodes");
        let generous_log_bound = 2 * (usize::BITS - (expected_size as usize).leading_zeros()) as i64;
        assert!(
            height <= generous_log_bound,
            "{name} height {height} stays logarithmic within bound {generous_log_bound}"
        );

        let lookup_count = global(&vm, module, &format!("lookup{expected_size}Count"))
            .as_int()
            .expect("lookup counter is Int");
        assert!(
            lookup_count <= 4 * height + 4,
            "{name} lookup comparisons {lookup_count} remain proportional to AVL height {height}"
        );

        let one_count = global(&vm, module, &format!("oneRemove{expected_size}Count"))
            .as_int()
            .expect("single-removal counter is Int");
        let many_count = global(&vm, module, &format!("manyRemove{expected_size}Count"))
            .as_int()
            .expect("multi-removal counter is Int");
        assert!(
            many_count <= one_count + 48,
            "{name} removing eleven entries adds bounded affected-region work rather than eleven root searches: one={one_count}, many={many_count}"
        );

        let one_overlay_count = global(&vm, module, &format!("oneOverlay{expected_size}Count"))
            .as_int()
            .expect("single-overlay counter is Int");
        let many_overlay_count = global(&vm, module, &format!("manyOverlay{expected_size}Count"))
            .as_int()
            .expect("multi-overlay counter is Int");
        assert!(
            many_overlay_count <= one_overlay_count + 48,
            "{name} overlaying eleven entries adds bounded affected-region work rather than eleven root searches: one={one_overlay_count}, many={many_overlay_count}"
        );

        let single_name = format!("oneRemoved{expected_size}");
        let (_, single_nodes, _) = tree_snapshot(&vm, global(&vm, module, &single_name));
        let single_new_nodes = single_nodes.difference(&nodes).count();
        assert!(
            single_new_nodes <= 12 * height as usize + 16,
            "{single_name} allocation is bounded by copied paths, found {single_new_nodes} new nodes"
        );

        let many_name = format!("manyRemoved{expected_size}");
        let (_, many_nodes, _) = tree_snapshot(&vm, global(&vm, module, &many_name));
        let many_new_nodes = many_nodes.difference(&nodes).count();
        assert!(
            many_new_nodes <= 24 * height as usize + 32,
            "{many_name} allocation is bounded by boundary paths, found {many_new_nodes} new nodes"
        );

        let one_overlay_name = format!("oneOverlay{expected_size}");
        let (_, one_overlay_nodes, _) = tree_snapshot(&vm, global(&vm, module, &one_overlay_name));
        let one_overlay_new_nodes = one_overlay_nodes.difference(&nodes).count();
        assert!(
            one_overlay_new_nodes <= 24 * height as usize + 32,
            "{one_overlay_name} allocation is bounded by copied paths, found {one_overlay_new_nodes} new nodes"
        );

        let many_overlay_name = format!("manyOverlay{expected_size}");
        let (_, many_overlay_nodes, _) = tree_snapshot(&vm, global(&vm, module, &many_overlay_name));
        let many_overlay_new_nodes = many_overlay_nodes.difference(&nodes).count();
        assert!(
            many_overlay_new_nodes <= 48 * height as usize + 48,
            "{many_overlay_name} allocation is bounded by boundary paths and the replacement node, found {many_overlay_new_nodes} new nodes"
        );
    }
}
