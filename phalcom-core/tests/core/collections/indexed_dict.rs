use super::vm_support;
use phalcom_core::value::Value;

fn as_bool(value: Value) -> bool {
    value.as_bool().unwrap_or(false)
}

#[test]
fn indexed_dict_duplicate_result_preserves_state() {
    let source = r#"
let mut dict = IndexedDict<String, Int>.new()
dict.insert(10, for: "A")
dict.insert(20, for: "B")

let dupRes = dict.insert((key: "B", value: 99), at: 0)
let dupIsError = dupRes.isErr and dupRes.match(
  ok: |v| { false },
  err: |e| { e is DuplicateKeyError }
)

let sizeUnchanged = dict.size == 2
let key0 = dict.keyAt(0).unwrapOr("") == "A"
let val0 = dict.valueAt(0).unwrapOr(0) == 10
let key1 = dict.keyAt(1).unwrapOr("") == "B"
let val1 = dict.valueAt(1).unwrapOr(0) == 20
let getA = dict["A"] == 10
let getB = dict["B"] == 20
let idxA = dict.indexOfKey("A").unwrapOr(-1) == 0
let idxB = dict.indexOfKey("B").unwrapOr(-1) == 1

let nextInsert = dict.insert((key: "C", value: 30), at: 1)
let nextOk = nextInsert.isOk and dict.size == 3
let postOrder = dict.keyAt(0).unwrapOr("") == "A" and dict.keyAt(1).unwrapOr("") == "C" and dict.keyAt(2).unwrapOr("") == "B"

let passed = dupIsError and sizeUnchanged and key0 and val0 and key1 and val1 and getA and getB and idxA and idxB and nextOk and postOrder
"#;
    let (vm, module) = vm_support::run_inline(source).expect("IndexedDict duplicate test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "IndexedDict preserves state atomically on duplicate key insertion");
}

#[test]
fn indexed_dict_invalid_index_result_preserves_state() {
    let source = r#"
let mut dict = IndexedDict<String, Int>.new()
dict.insert(10, for: "A")
dict.insert(20, for: "B")

let invalidRes = dict.insert((key: "X", value: 50), at: 10)
let invalidIsError = invalidRes.isErr and invalidRes.match(
  ok: |v| { false },
  err: |e| { e is IndexError }
)

let sizeUnchanged = dict.size == 2
let key0 = dict.keyAt(0).unwrapOr("") == "A"
let val0 = dict.valueAt(0).unwrapOr(0) == 10
let key1 = dict.keyAt(1).unwrapOr("") == "B"
let val1 = dict.valueAt(1).unwrapOr(0) == 20

let negativeRes = dict.insert((key: "Y", value: 60), at: -10)
let negativeIsError = negativeRes.isErr and negativeRes.match(
  ok: |v| { false },
  err: |e| { e is IndexError }
)

let passed = invalidIsError and negativeIsError and sizeUnchanged and key0 and val0 and key1 and val1
"#;
    let (vm, module) = vm_support::run_inline(source).expect("IndexedDict invalid index test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "IndexedDict preserves state atomically on invalid index insertion");
}

#[test]
fn indexed_dict_collision_mutation_preserves_indexes() {
    let source = r#"
class CollisionKey {
  mut _id
  id { _id }
  @constructor
  new(_ id) { _id = id }
  hash { 77 }
  ==(_ other) { _id == other.id }
}
let A = CollisionKey.new("A")
let B = CollisionKey.new("B")
let C = CollisionKey.new("C")
let D = CollisionKey.new("D")

let mut dict = IndexedDict<CollisionKey, Int>.new()
dict.insert(100, for: A)
dict.insert(200, for: B)
dict.insert(300, for: C)

let initOk = dict.size == 3 and dict[A] == 100 and dict[B] == 200 and dict[C] == 300 and dict.indexOfKey(A).unwrapOr(-1) == 0 and dict.indexOfKey(B).unwrapOr(-1) == 1 and dict.indexOfKey(C).unwrapOr(-1) == 2

let overwriteRes = dict.insert(999, for: B)
let overwriteOk = overwriteRes.unwrapOr(0) == 200 and dict.size == 3 and dict[B] == 999 and dict.indexOfKey(B).unwrapOr(-1) == 1

let moveRes = dict.move(2, to: 0)
let moveOk = moveRes.isOk and dict.size == 3 and dict.indexOfKey(C).unwrapOr(-1) == 0 and dict.indexOfKey(A).unwrapOr(-1) == 1 and dict.indexOfKey(B).unwrapOr(-1) == 2

dict.remove(A)
let removeOk = dict.size == 2 and dict.indexOfKey(C).unwrapOr(-1) == 0 and dict.indexOfKey(B).unwrapOr(-1) == 1 and not dict.contains(A)

let insertRes = dict.insert((key: D, value: 400), at: 1)
let insertOk = insertRes.isOk and dict.size == 3 and dict[D] == 400 and dict.indexOfKey(C).unwrapOr(-1) == 0 and dict.indexOfKey(D).unwrapOr(-1) == 1 and dict.indexOfKey(B).unwrapOr(-1) == 2

let passed = initOk and overwriteOk and moveOk and removeOk and insertOk
"#;
    let (vm, module) = vm_support::run_inline(source).expect("IndexedDict collision mutations compile and run");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "IndexedDict collision-heavy mutations preserve key and positional consistency");
}

#[test]
fn indexed_dict_key_lookup_is_not_positional_lookup() {
    let source = r#"
let mut dict = IndexedDict<Int, String>.new()
dict.insert("zero_key", for: 100)
dict.insert("one_key", for: 0)

let keyedZero = dict[0] == "one_key"
let keyedHundred = dict[100] == "zero_key"

let posKeyZero = dict.keyAt(0).unwrapOr(-1) == 100
let posValZero = dict.valueAt(0).unwrapOr("") == "zero_key"
let posKeyOne = dict.keyAt(1).unwrapOr(-1) == 0
let posValOne = dict.valueAt(1).unwrapOr("") == "one_key"

let passed = keyedZero and keyedHundred and posKeyZero and posValZero and posKeyOne and posValOne
"#;
    let (vm, module) = vm_support::run_inline(source).expect("IndexedDict integer key test compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "IndexedDict subscript uses keyed lookup, not positional lookup");
}

#[test]
fn indexed_dict_reentrant_overwrite_obeys_commit_order() {
    let source = r#"
class ReentrantOverwriteKey {
  mut _dict
  mut _phase
  @constructor
  new(_ dict) { _dict = dict; _phase = 0 }
  hash { 0 }
  ==(_ other) {
    if _phase == 0 {
      _phase = 1
      _dict.insert("inner", for: other)
    }
    true
  }
}
let mut dict = IndexedDict<ReentrantOverwriteKey, String>.new()
let k1 = ReentrantOverwriteKey.new(dict)
let k2 = ReentrantOverwriteKey.new(dict)
dict.insert("initial", for: k1)

let outerRes = dict.insert("outer", for: k2)
let passed = dict.size == 1 and dict[k1] == "outer" and outerRes.unwrapOr("") == "inner"
"#;
    let (vm, module) = vm_support::run_inline(source).expect("IndexedDict reentrant overwrite compiles and runs");
    let passed_symbol = vm.interner.find("passed").expect("result is interned");
    let passed = vm.heap.module(module).get(passed_symbol).expect("source publishes its result");
    assert!(as_bool(passed), "IndexedDict reentrant value overwrite obeys outer-wins commit order");
}
