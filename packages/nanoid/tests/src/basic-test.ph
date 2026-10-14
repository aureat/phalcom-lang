from nanoid.id import NanoId
from nanoid.alphabet import Alphabet
from nanoid.generator import NanoIdGenerator
from .support.test import Test

// Test default length
let id = NanoId.generate()
Test.expectEqual(id.byteCount, 21, "default id length")

// Test variable lengths
Test.expectEqual(NanoId.generate(0), "", "zero length id")
Test.expectEqual(NanoId.generate(1).byteCount, 1, "size 1 id")
Test.expectEqual(NanoId.generate(12).byteCount, 12, "size 12 id")
Test.expectEqual(NanoId.generate(21).byteCount, 21, "size 21 id")
Test.expectEqual(NanoId.generate(64).byteCount, 64, "size 64 id")

// Test that two consecutive IDs are different (unpredictability)
let id1 = NanoId.generate()
let id2 = NanoId.generate()
Test.expect(id1 != id2, "consecutive ids must differ")

// Test character set membership
let urlAlphabet = Alphabet.urlSafe.symbols
let mut i = 0
while i < id.byteCount {
  let ch = id.rawSlice(i, i + 1)
  Test.expect(urlAlphabet.contains(ch), "symbol must belong to url alphabet")
  i = i + 1
}

Test.pass("nanoid-basic")
