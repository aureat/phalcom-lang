use super::vm_support;
use phalcom_core::value::Value;

fn as_bool(value: Value) -> bool {
    value.as_bool().unwrap_or(false)
}

#[test]
fn probe_t0_p1_generic_comparison() {
    let source = r#"
class GenericComparisonProbe<K> {
    @constructor
    new() {}

    compare(_ a: K, _ b: K) -> Ordering {
        a <=> b
    }
}

let probe = GenericComparisonProbe<Int>.new()
let r1 = probe.compare(10, 20) === Ordering::Less
let r2 = probe.compare(20, 20) === Ordering::Equal
let r3 = probe.compare(30, 20) === Ordering::Greater

let passed = r1 and r2 and r3
"#;
    let (vm, module) = vm_support::run_inline(source).expect("T0-P1: generic comparison compiles and executes");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "T0-P1: generic <=> executes and yields canonical Ordering variants");
}

#[test]
fn probe_t0_p3_reflected_compare() {
    let source = r#"
class CustomDomain {
    _val: Int

    @constructor
    new(_ v: Int) {
        _val = v
    }

    compare(_ other: CustomDomain) -> Ordering {
        self._val <=> other.val
    }

    val -> Int { self._val }
}

let c1 = CustomDomain.new(10)
let c2 = CustomDomain.new(20)

let r1 = (c1 <=> c2) === Ordering::Less
let r2 = (c2 <=> c1) === Ordering::Greater
let r3 = (c1 <=> c1) === Ordering::Equal

let passed = r1 and r2 and r3
"#;
    let (vm, module) = vm_support::run_inline(source).expect("T0-P3: custom domain comparison compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "T0-P3: custom domain comparison preserves bilateral ordering");
}

#[test]
fn probe_t0_p4_unordered_and_throw() {
    let source = r#"
class PartiallyOrdered {
    _tag: Int

    @constructor
    new(_ t: Int) {
        _tag = t
    }

    compare(_ other: PartiallyOrdered) -> Ordering {
        if self._tag == 0 or other.tag == 0 {
            return Ordering::Unordered
        }
        self._tag <=> other.tag
    }

    tag -> Int { self._tag }
}

let p0 = PartiallyOrdered.new(0)
let p1 = PartiallyOrdered.new(1)
let p2 = PartiallyOrdered.new(2)

let rUnordered = (p0 <=> p1) === Ordering::Unordered
let rOrdered = (p1 <=> p2) === Ordering::Less

let passed = rUnordered and rOrdered
"#;
    let (vm, module) = vm_support::run_inline(source).expect("T0-P4: Ordering::Unordered compiles and executes");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "T0-P4: Ordering::Unordered returns correctly without coercion");
}

#[test]
fn probe_t0_p5_ordinary_sum_and_product_data() {
    let source = r#"
data BoundProbe<T> {
    @variant Unbounded
    @variant Included(value: T)
    @variant Excluded(value: T)
}

data IntervalProbe<T>(_ lower: BoundProbe<T>, _ upper: BoundProbe<T>)

let b1 = BoundProbe<Int>::Included(value: 10)
let b2 = BoundProbe<Int>::Excluded(value: 20)
let iv = IntervalProbe(b1, b2)

let isIncluded = match iv.lower {
    BoundProbe::Included(v) => v == 10
    _ => false
}

let isExcluded = match iv.upper {
    BoundProbe::Excluded(v) => v == 20
    _ => false
}

let passed = isIncluded and isExcluded
"#;
    let (vm, module) = vm_support::run_inline(source).expect("T0-P5: Bound and Interval data compile and execute");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "T0-P5: ordinary sum and product data work with pattern matching and field access");
}

#[test]
fn probe_t0_p6_recursive_generic_node() {
    let source = r#"
class NodeProbe<T> {
    _value: T
    _left: Option<NodeProbe<T>>
    _right: Option<NodeProbe<T>>
    _height: Int

    @constructor
    new(_ value: T, left: Option<NodeProbe<T>>, right: Option<NodeProbe<T>>, height: Int) {
        _value = value
        _left = left
        _right = right
        _height = height
    }

    value -> T { self._value }
    left -> Option<NodeProbe<T>> { self._left }
    right -> Option<NodeProbe<T>> { self._right }
    height -> Int { self._height }

    withLeft(_ newLeft: Option<NodeProbe<T>>) -> NodeProbe<T> {
        NodeProbe<T>.new(self._value, left: newLeft, right: self._right, height: self._height + 1)
    }
}

let n0 = NodeProbe<Int>.new(10, left: None, right: None, height: 1)
let nChild = NodeProbe<Int>.new(5, left: None, right: None, height: 1)
let n1 = n0.withLeft(Some(nChild))

let n0Unchanged = n0.left == None and n0.height == 1
let n1Updated = n1.left != None and n1.height == 2 and n1.value == 10

let passed = n0Unchanged and n1Updated
"#;
    let (vm, module) = vm_support::run_inline(source).expect("T0-P6: path-copy node compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "T0-P6: recursive generic node path-copy preserves old root immutability");
}

#[test]
fn probe_t0_p7_source_collection_wrapper() {
    let source = r#"
class RangeMapProbe<K, V> is Collection {
    _size: Int

    @constructor
    new(_ size: Int) {
        _size = size
    }

    size { self._size }
    isEmpty { self._size == 0 }
}

let m0 = RangeMapProbe<Int, String>.new(0)
let m1 = RangeMapProbe<Int, String>.new(1)

let m0Ok = m0.isEmpty and m0.size == 0 and m0 is Collection
let m1Ok = not m1.isEmpty and m1.size == 1 and m1 is Collection

let passed = m0Ok and m1Ok
"#;
    let (vm, module) = vm_support::run_inline(source).expect("T0-P7: source collection wrapper compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "T0-P7: source RangeMap class integrates with Collection hierarchy");
}

#[test]
fn test_cv01_bound_construction_and_accessors() {
    let source = r#"
let bUnbounded = Bound<Int>.unbounded
let bIncluded = Bound<Int>.included(10)
let bExcluded = Bound<Int>.excluded(20)

let vIncluded = match bIncluded.value {
    Option::Some(v) => v == 10
    _ => false
}
let vExcluded = match bExcluded.value {
    Option::Some(v) => v == 20
    _ => false
}

let tUnbounded = bUnbounded.isUnbounded and not bUnbounded.isIncluded and not bUnbounded.isExcluded and bUnbounded.value == None
let tIncluded = not bIncluded.isUnbounded and bIncluded.isIncluded and not bIncluded.isExcluded and vIncluded
let tExcluded = not bExcluded.isUnbounded and not bExcluded.isIncluded and bExcluded.isExcluded and vExcluded

let passed = tUnbounded and tIncluded and tExcluded
"#;
    let (vm, module) = vm_support::run_inline(source).expect("CV-01: Bound compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "CV-01: Bound variants and accessors satisfy specification");
}

