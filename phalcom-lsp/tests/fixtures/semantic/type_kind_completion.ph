from universe.collections.record import RecordFields
from universe.collections.tuple import CompleteTypePack

let mut runtimeValue = 1
type RecordCarrier<**Fs: RecordFields/*@record_kind*/> = Int
type TupleCarrier<***Es: CompleteTypePack/*@pack_kind*/> = Int
let mut anotherValue = /*@runtime_value*/ runtimeValue
