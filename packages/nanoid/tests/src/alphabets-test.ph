from nanoid.alphabet import Alphabet
from nanoid.generator import NanoIdGenerator
from .support.test import Test

// Default URL-safe alphabet
let defaultAlpha = Alphabet.urlSafe
Test.expectEqual(defaultAlpha.size, 64, "urlSafe size")

// 2-character alphabet (minimum allowed)
let binaryAlpha = Alphabet.new("01")
Test.expectEqual(binaryAlpha.size, 2, "binary alphabet size")
let binaryGen = NanoIdGenerator.new(binaryAlpha, 32)
let binaryId = binaryGen.generate()
Test.expectEqual(binaryId.byteCount, 32, "binary id length")
let mut i = 0
while i < binaryId.byteCount {
  let ch = binaryId.rawSlice(i, i + 1)
  Test.expect(ch == "0" or ch == "1", "binary id symbols must be 0 or 1")
  i = i + 1
}

// Hex alphabet (size 16, power of two)
let hexAlpha = Alphabet.new("0123456789abcdef")
Test.expectEqual(hexAlpha.size, 16, "hex alphabet size")
let hexGen = NanoIdGenerator.new(hexAlpha, 16)
let hexId = hexGen.generate()
Test.expectEqual(hexId.byteCount, 16, "hex id length")

// 62-character alphabet (base62, rejection sampling required)
let base62Alpha = Alphabet.new("0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz")
Test.expectEqual(base62Alpha.size, 62, "base62 size")
let base62Gen = NanoIdGenerator.new(base62Alpha, 20)
let base62Id = base62Gen.generate()
Test.expectEqual(base62Id.byteCount, 20, "base62 id length")

// Decimal alphabet (size 10)
let decAlpha = Alphabet.new("0123456789")
Test.expectEqual(decAlpha.size, 10, "decimal size")
let decGen = NanoIdGenerator.new(decAlpha, 10)
let decId = decGen.generate()
Test.expectEqual(decId.byteCount, 10, "decimal id length")

// Verify ordering preservation
Test.expectEqual(hexAlpha.symbols, "0123456789abcdef", "symbols order preserved")

Test.pass("nanoid-alphabets")
