use super::vm_support;

#[test]
fn static_labeled_sources_spread_into_record_and_collections() {
    let source = r#"
const tupleSource = (1, 2, name: "Ada", age: 42)
const recordSource = { original: "Record" }
const mapSource = Map { mapEntry: 7 }

const recordFromTuple = { **tupleSource }
const recordFromRecord = { id: 7, **recordSource }
const mapFromTuple = Map { **tupleSource }
const mapFromRecord = Map { **recordSource }
const mapFromMap = Map { **mapSource }

let tupleRecord = recordFromTuple.size == 2 and
  recordFromTuple.get(#name).match(some: |value| { value == "Ada" }, none: || { false }) and
  recordFromTuple.get(#age).match(some: |value| { value == 42 }, none: || { false })
let recordRecord = recordFromRecord.size == 2 and
  recordFromRecord.get(#id).match(some: |value| { value == 7 }, none: || { false }) and
  recordFromRecord.get(#original).match(some: |value| { value == "Record" }, none: || { false })
let tupleMap = mapFromTuple.size == 2
let recordMap = mapFromRecord.size == 1
let mapMap = mapFromMap.size == 1
let result = tupleRecord and recordRecord and tupleMap and recordMap and mapMap
"#;
    let (vm, module) = vm_support::run_inline(source).expect("statically labeled spread forms compile and execute");
    for name in ["tupleRecord", "recordRecord", "tupleMap", "recordMap", "mapMap", "result"] {
        let symbol = vm.interner.find(name).expect("result symbol");
        assert_eq!(vm.heap.module(module).get(symbol).expect("result binding").to_string(&vm), "true", "{name}");
    }
}
