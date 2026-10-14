from snowflake import SnowflakeId
from universe.collections.bytes import Bytes

let id = SnowflakeId.parse("1116766490855473152").unwrap
if id.value != 1116766490855473152 or id.toString != "1116766490855473152" {
  throw Error.new("decimal conversion")
}
if SnowflakeId.parseBase2(id.base2).unwrap != id { throw Error.new("binary round trip") }
if SnowflakeId.parseBase32(id.base32).unwrap != id { throw Error.new("z-base-32 round trip") }
if SnowflakeId.parseBase36(id.base36).unwrap != id { throw Error.new("base-36 round trip") }
if SnowflakeId.parseBase58(id.base58).unwrap != id { throw Error.new("base-58 round trip") }
if SnowflakeId.parseBase64(id.base64).unwrap != id { throw Error.new("base-64 round trip") }
if SnowflakeId.parseJSON(id.json).unwrap != id { throw Error.new("JSON string round trip") }
if SnowflakeId.fromBytes(id.toBytes()).unwrap != id { throw Error.new("big-endian bytes round trip") }
if SnowflakeId.parseTextBytes(id.toTextBytes()).unwrap != id { throw Error.new("Go text bytes round trip") }

if SnowflakeId.parseBase32("b8wjm1zroyyyy").unwrap.value != 1427970479175499776 {
  throw Error.new("Go z-base-32 fixture")
}
if SnowflakeId.fromInt(1427970479175499776).unwrap.base32 != "b8wjm1zroyyyy" {
  throw Error.new("Go z-base-32 encoding")
}
if SnowflakeId.parseBase58("4jgmnx8Js8A").unwrap.value != 1428076403798048768 {
  throw Error.new("Go base-58 fixture")
}
if SnowflakeId.fromInt(1428076403798048768).unwrap.base58 != "4jgmnx8Js8A" {
  throw Error.new("Go base-58 encoding")
}
if SnowflakeId.parseBase64("MTExNjgxOTQ5NDY2MDk5NzEyMA==").unwrap.value != 1116819494660997120 {
  throw Error.new("Go base-64 fixture")
}
if SnowflakeId.fromInt(1116819494660997120).unwrap.base64 != "MTExNjgxOTQ5NDY2MDk5NzEyMA==" {
  throw Error.new("Go base-64 encoding")
}
if SnowflakeId.fromBytes(Bytes.fromList([0x0f, 0x7f, 0xc0, 0xfc, 0x2f, 0x80, 0, 0])).unwrap.value != 1116823421972381696 {
  throw Error.new("Go big-endian integer bytes fixture")
}
if SnowflakeId.fromInt(0).unwrap.base32 != "y" or SnowflakeId.fromInt(0).unwrap.base58 != "1" {
  throw Error.new("zero alphabets")
}
if SnowflakeId.parse("").isOk or SnowflakeId.parse("-1").isOk or SnowflakeId.parse("9223372036854775808").isOk {
  throw Error.new("invalid decimal accepted")
}
let max = SnowflakeId.fromInt(0x7fffffffffffffff).unwrap
if SnowflakeId.parse(max.toString).unwrap != max or
  SnowflakeId.parseBase32(max.base32).unwrap != max or
  SnowflakeId.parseBase58(max.base58).unwrap != max or
  SnowflakeId.parseBase36(max.base36.uppercase()).unwrap != max {
  throw Error.new("63-bit maximum round trip")
}
if SnowflakeId.parseBase32("B8WJM1ZROYYYY").isOk or SnowflakeId.parseBase58("0jgmnx8Js8A").isOk {
  throw Error.new("invalid alphabet accepted")
}
if SnowflakeId.parseJSON("1").isOk or SnowflakeId.parseBase64("eA==").isOk {
  throw Error.new("invalid JSON or nondecimal base-64 accepted")
}
if SnowflakeId.parseBase64("MB==").isOk or SnowflakeId.parseBase64("MA==MA==").isOk or
  SnowflakeId.parseBase64("MA=").isOk {
  throw Error.new("invalid Base64 padding accepted")
}
if SnowflakeId.parseBase64("MA==").unwrap.value != 0 {
  throw Error.new("Base64 zero")
}
if SnowflakeId.fromBytes(Bytes.new(7)).isOk or SnowflakeId.fromBytes(Bytes.fromList([0x80, 0, 0, 0, 0, 0, 0, 0])).isOk {
  throw Error.new("out-of-range binary accepted")
}

System.print("PASS snowflake-conversions")
