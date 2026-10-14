from nanoid.alphabet import Alphabet
from nanoid.generator import NanoIdGenerator
from universe.collections.bytes import Bytes
from .support.test import Test

// Statistical smoke test on a 4-character alphabet ("ABCD")
// Generate 1200 characters and check that all 4 symbols occur reasonably
let alpha = Alphabet.new("ABCD")
let gen = NanoIdGenerator.new(alpha, 1200)
let text = gen.generate()

let bytes = Bytes.fromString(text)
let counts = Bytes.new(4)
let mut i = 0

while i < 1200 {
  let b = bytes.at(i)
  if b == 65 {
    counts.set(0, counts.at(0) + 1)
  } else if b == 66 {
    counts.set(1, counts.at(1) + 1)
  } else if b == 67 {
    counts.set(2, counts.at(2) + 1)
  } else if b == 68 {
    counts.set(3, counts.at(3) + 1)
  } else {
    throw Error.new("unexpected character in output: \(b)")
  }
  i = i + 1
}

// In 1200 trials with uniform p=0.25, expected count is 300 each.
// Standard deviation is sqrt(1200 * 0.25 * 0.75) = 15.
// With overwhelming probability (> 99.999%), counts fall in 150..450.
let mut k = 0
while k < 4 {
  let count = counts.at(k)
  Test.expect(count > 100, "each symbol must appear substantially (got \(count))")
  Test.expect(count < 500, "no symbol should excessively dominate (got \(count))")
  k = k + 1
}

Test.pass("nanoid-distribution")
