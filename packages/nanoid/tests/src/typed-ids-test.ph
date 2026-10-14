from nanoid.id import NanoId, Id, TypedNanoIdGenerator
from nanoid.alphabet import Alphabet
from nanoid.generator import NanoIdGenerator
from nanoid.errors import NanoIdParseError
from .support.test import Test

data UserTag()
data OrderTag()

// Typed generation
let userId: Id<UserTag> = NanoId.typed<UserTag>()
Test.expectEqual(userId.value.byteCount, 21, "default typed id length")
Test.expectEqual(userId.toString, userId.value, "toString equals value")

let customTyped: Id<OrderTag> = NanoId.typed<OrderTag>(16)
Test.expectEqual(customTyped.value.byteCount, 16, "custom length typed id")

// Valid parse
let parsed = Id<UserTag>.parse(userId.value, Alphabet.urlSafe, 21)
match parsed {
  Result::Ok(id) => {
    Test.expectEqual(id.value, userId.value, "parsed value matches")
  }
  Result::Error(e) => {
    throw Error.new("expected Ok parse, got error: \(e)")
  }
}

// Invalid length parse
let badLength = Id<UserTag>.parse(userId.value, Alphabet.urlSafe, 10)
match badLength {
  Result::Ok(_) => {
    throw Error.new("expected error on length mismatch")
  }
  Result::Error(_) => {
    // Expected
  }
}

// Invalid character parse
let hexAlpha = Alphabet.new("0123456789abcdef")
let hexGenerator = NanoIdGenerator.new(hexAlpha, 16)
let generated: Id<UserTag> = TypedNanoIdGenerator<UserTag>.fromGenerator(hexGenerator).generate()
Test.expectEqual(generated.value.byteCount, 16, "configured typed generator length")
let parsedHex: Result<Id<UserTag>, NanoIdParseError> = Id<UserTag>.parse(generated.value, hexAlpha, 16)
if not parsedHex.isOk { throw Error.new("configured typed generator uses its alphabet") }
let short: Id<OrderTag> = TypedNanoIdGenerator<OrderTag>.fromGenerator(hexGenerator).generate(8)
Test.expectEqual(short.value.byteCount, 8, "configured typed generator override")

let badChar = Id<UserTag>.parse("0123456789abcdeg", hexAlpha, 16) // 'g' not in hex
match badChar {
  Result::Ok(_) => {
    throw Error.new("expected error on invalid character")
  }
  Result::Error(_) => {
    // Expected
  }
}

match badLength {
  Result::Error(NanoIdParseError::InvalidLength(_, _)) => {}
  _ => { throw Error.new("length failure must be typed") }
}
match badChar {
  Result::Error(NanoIdParseError::InvalidCharacter(_)) => {}
  _ => { throw Error.new("alphabet failure must be typed") }
}

Test.pass("nanoid-typed-ids")
