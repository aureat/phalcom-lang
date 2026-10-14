from nanoid.id import NanoId
from nanoid.alphabet import Alphabet
from nanoid.generator import NanoIdGenerator
from .support.test import Test

// Boundary: Length 0
let emptyId = NanoId.generate(0)
Test.expectEqual(emptyId, "", "length 0 produces empty string")

// Boundary: Length 1
let oneId = NanoId.generate(1)
Test.expectEqual(oneId.byteCount, 1, "length 1 produces 1 character")

// Boundary: Length 4096 (exactly batch limit)
let gen = NanoIdGenerator.default
let batchId = gen.generate(4096)
Test.expectEqual(batchId.byteCount, 4096, "length 4096 produces 4096 characters")

// Boundary: Length 4097 (spans two batches)
let multiBatchId = gen.generate(4097)
Test.expectEqual(multiBatchId.byteCount, 4097, "length 4097 produces 4097 characters")

// Boundary: Maximum alphabet size (94 printable ASCII characters)
// 0x21 ('!') to 0x7E ('~') is 94 characters
let fullAscii = "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~"
let fullAlpha = Alphabet.new(fullAscii)
Test.expectEqual(fullAlpha.size, 94, "max alphabet size is 94")
let fullGen = NanoIdGenerator.new(fullAlpha, 15)
let fullId = fullGen.generate()
Test.expectEqual(fullId.byteCount, 15, "max alphabet id length")

Test.pass("nanoid-boundaries")
