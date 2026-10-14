from uuid import UUID, UUIDVersion, UUIDVariant
from universe.collections.bytes import Bytes

let canonical = "550e8400-e29b-41d4-a716-446655440000"
let value = UUID.parseCanonical(canonical).unwrap
if value.toString != canonical { throw Error.new("canonical format") }
if value.simple != "550e8400e29b41d4a716446655440000" { throw Error.new("simple format") }
if value.urn != "urn:uuid:" + canonical { throw Error.new("URN format") }
if value.braced != "{" + canonical + "}" { throw Error.new("braced format") }
if UUID.parse(value.simple).unwrap != value { throw Error.new("simple parse") }
if UUID.parse(value.urn).unwrap != value { throw Error.new("URN parse") }
if UUID.parse(value.braced).unwrap != value { throw Error.new("braced parse") }
if UUID.parse("550E8400-E29B-41D4-A716-446655440000").unwrap != value {
  throw Error.new("uppercase parse")
}
if UUID.parseCanonical("550E8400-E29B-41D4-A716-446655440000").isOk {
  throw Error.new("canonical parse must reject uppercase")
}
if UUID.parse(canonical + " ").isOk { throw Error.new("no whitespace trimming") }
if UUID.parse("550e8400-e29b-41d4-a716-44665544000g").isOk {
  throw Error.new("invalid hex accepted")
}
if UUID.parse("550e8400_e29b-41d4-a716-446655440000").isOk {
  throw Error.new("invalid separator accepted")
}
let network = Bytes.fromList([0x55, 0x0e, 0x84, 0, 0xe2, 0x9b, 0x41, 0xd4,
  0xa7, 0x16, 0x44, 0x66, 0x55, 0x44, 0, 0])
if value.toBytes() != network { throw Error.new("network byte order") }
if UUID.fromBytes(network).unwrap != value { throw Error.new("binary round trip") }
if UUID.fromBytes(Bytes.new(15)).isOk { throw Error.new("binary width") }
network.set(0, 0)
if value.toString != canonical { throw Error.new("mutable buffer escaped into UUID") }
if UUID.nil.version != None or not UUID.nil.isNil { throw Error.new("nil metadata") }
if UUID.max.version != None or not UUID.max.isMax { throw Error.new("max metadata") }
if not (UUID.parse("ffffffff-ffff-ffff-ffff-fffffffffffe").unwrap.variant === UUIDVariant::Future) {
  throw Error.new("reserved variant not preserved")
}
let unknown = UUID.parse("00000000-0000-f000-8000-000000000000").unwrap
if not (unknown.version === Some(UUIDVersion::Reserved(15))) { throw Error.new("unknown version lost") }
if UUID.nil == UUID.max or UUID.nil.hash == UUID.max.hash { throw Error.new("value hashing") }
if not (UUID.nil < value and value < UUID.max) { throw Error.new("unsigned ordering") }
let ids = Set<UUID>.new()
ids.add(value)
ids.add(UUID.parse(canonical).unwrap)
if ids.size != 1 or not ids.contains(value) { throw Error.new("Set UUID key equality") }

System.print("PASS uuid-representation")
