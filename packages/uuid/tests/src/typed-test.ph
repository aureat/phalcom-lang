from uuid import UUIDParseError, UUIDv7, UUIDv7ParseError, TaggedUUID, UUIDv7Generator

data UserTag()
data OrderTag()

let user: TaggedUUID<UserTag> = TaggedUUID<UserTag>.v4()
let parsed: Result<TaggedUUID<UserTag>, UUIDParseError> = TaggedUUID<UserTag>.parse(user.toString)
if parsed.unwrap.value != user.value { throw Error.new("tagged UUID parse") }
let order: TaggedUUID<OrderTag> = TaggedUUID<OrderTag>.fromUUID(user.value)
if order.value != user.value { throw Error.new("explicit UUID domain assertion") }

let seven: UUIDv7 = UUIDv7.generate()
if seven.timestamp != seven.value.timestamp.unwrap { throw Error.new("checked UUIDv7 timestamp") }
if UUIDv7.parse(seven.toString).unwrap != seven { throw Error.new("checked UUIDv7 round trip") }
match UUIDv7.fromUUID(user.value) {
  Result::Error(UUIDv7ParseError::UnexpectedVersion(_)) => {}
  _ => { throw Error.new("UUIDv4 is not UUIDv7") }
}
match UUIDv7.parse("broken") {
  Result::Error(UUIDv7ParseError::InvalidUUID(_)) => {}
  _ => { throw Error.new("malformed UUIDv7 text") }
}

let monotonic = UUIDv7Generator.monotonic()
let first: UUIDv7 = monotonic.generateTyped()
let next: UUIDv7 = monotonic.generateTyped()
if not (first < next) { throw Error.new("typed UUIDv7 generator order") }

System.print("PASS uuid-typed")
