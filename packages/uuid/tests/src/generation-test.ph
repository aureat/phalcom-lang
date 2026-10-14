from uuid import UUID, UUIDVersion, UUIDVariant, UUIDv7Generator
from universe.collections.bytes import Bytes
from universe.time.timestamp import Timestamp

let random = UUID.v4()
if not (random.version === Some(UUIDVersion::Random)) or not (random.variant === UUIDVariant::RFC9562) {
  throw Error.new("UUIDv4 bit layout")
}
if UUID.fromBytes(random.toBytes()).unwrap != random { throw Error.new("UUIDv4 byte round trip") }

let ordered = UUID.v7()
if not (ordered.version === Some(UUIDVersion::UnixTime)) or not (ordered.variant === UUIDVariant::RFC9562) {
  throw Error.new("UUIDv7 bit layout")
}
let nowMillis = Timestamp.now.unixNanoseconds ~/ 1000000
if ordered.timestamp.unwrap > nowMillis or ordered.timestamp.unwrap < nowMillis - 10000 {
  throw Error.new("UUIDv7 timestamp")
}
let vector = UUID.parseCanonical("017f22e2-79b0-7cc3-98c4-dc0c0c07398f").unwrap
if not (vector.timestamp === Some(1645557742000)) { throw Error.new("RFC 9562 v7 timestamp") }

let dns = UUID.v5(namespace: UUID.namespaceDNS, name: "www.example.com")
if dns.toString != "2ed6657d-e927-568b-95e1-2665a8aea6a2" {
  throw Error.new("RFC 9562 UUIDv5 vector")
}
if dns != UUID.v5Bytes(namespace: UUID.namespaceDNS, name: Bytes.fromString("www.example.com")) {
  throw Error.new("UUIDv5 binary name")
}
if not (dns.version === Some(UUIDVersion::NameBasedSHA1)) { throw Error.new("UUIDv5 bit layout") }

let generator = UUIDv7Generator.monotonic()
let mut previous = generator.generate()
let mut i = 0
while i < 20 {
  let next = generator.generate()
  if not (previous < next) { throw Error.new("monotonic UUIDv7 sequence") }
  previous = next
  i = i + 1
}

System.print("PASS uuid-generation")
