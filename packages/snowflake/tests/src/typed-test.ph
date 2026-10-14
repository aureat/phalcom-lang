from snowflake import SnowflakeId, SnowflakeLayout, TypedSnowflake, TypedSnowflakeNode

data OrderTag()
data UserTag()
data CompactFormat()
data TwitterFormat()

let layout = SnowflakeLayout.configured(epochMillis: 100, nodeBits: 3, sequenceBits: 4)
let worker = TypedSnowflakeNode<UserTag, CompactFormat>.configured(5, layout)
let first: TypedSnowflake<UserTag, CompactFormat> = worker.generateAt(102)
if first.value.value != 336 or first.timeMillis != 102 or first.node != 5 or first.step != 0 {
  throw Error.new("typed custom layout")
}
let parsed = TypedSnowflake<UserTag, CompactFormat>.parse(first.toString, layout).unwrap
if parsed != first { throw Error.new("layout-bound parse") }
if TypedSnowflake<UserTag, CompactFormat>.parse("-1", layout).isOk {
  throw Error.new("invalid typed Snowflake input")
}
let different = SnowflakeLayout.configured(epochMillis: 0, nodeBits: 3, sequenceBits: 4)
if TypedSnowflake<UserTag, CompactFormat>.fromId(first.value, different) == first {
  throw Error.new("layout participates in typed equality")
}
let order: TypedSnowflake<OrderTag, TwitterFormat> = TypedSnowflakeNode<OrderTag, TwitterFormat>.twitter(1).generateAt(1288834974657)
if order.value != SnowflakeId.fromInt(4096).unwrap { throw Error.new("typed Twitter generation") }

System.print("PASS snowflake-typed")
