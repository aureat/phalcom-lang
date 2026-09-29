from universe.collections.record import Record, RecordFields
from universe.collections.tuple import CompleteTypePack, Tuple

let runtimeValue = 1
let recordValue = { recordField: /*@record_value*/ runtimeValue }
let mapValue = Map { mapField: /*@map_value*/ runtimeValue }
let setValue = #{/*@set_value*/ runtimeValue}
