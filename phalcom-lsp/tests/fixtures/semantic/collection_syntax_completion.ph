from universe.collections.record import Record, RecordFields
from universe.collections.tuple import CompleteTypePack, Tuple

let mut runtimeValue = 1
let mut recordValue = { recordField: /*@record_value*/ runtimeValue }
let mut mapValue = Map { mapField: /*@map_value*/ runtimeValue }
let mut setValue = #{/*@set_value*/ runtimeValue}
