from nanoid.id import NanoId
from nanoid.alphabet import Alphabet
from nanoid.generator import NanoIdGenerator
from nanoid.errors import InvalidAlphabetError, InvalidIdLengthError
from .support.test import Test

// Empty alphabet
let mut caught = false
try {
  Alphabet.new("")
} on InvalidAlphabetError error {
  caught = true
}
Test.expect(caught, "empty alphabet must raise InvalidAlphabetError")

// Single-character alphabet (size < 2)
caught = false
try {
  Alphabet.new("a")
} on InvalidAlphabetError error {
  caught = true
}
Test.expect(caught, "single-char alphabet must raise InvalidAlphabetError")

// Duplicate character
caught = false
try {
  Alphabet.new("abca")
} on InvalidAlphabetError error {
  caught = true
}
Test.expect(caught, "duplicate char in alphabet must raise InvalidAlphabetError")

// Non-printable character (newline = 10 < 33)
caught = false
try {
  Alphabet.new("ab\nc")
} on InvalidAlphabetError error {
  caught = true
}
Test.expect(caught, "control character in alphabet must raise InvalidAlphabetError")

// Negative size in NanoId.generate
caught = false
try {
  NanoId.generate(-1)
} on InvalidIdLengthError error {
  caught = true
}
Test.expect(caught, "negative length in NanoId.generate must raise InvalidIdLengthError")

// Generator construction with negative size
caught = false
try {
  NanoIdGenerator.new(Alphabet.urlSafe, -5)
} on InvalidIdLengthError error {
  caught = true
}
Test.expect(caught, "negative size in generator must raise InvalidIdLengthError")

// Length exceeding maximum supported limit (> 1048576)
caught = false
try {
  NanoId.generate(2000000)
} on InvalidIdLengthError error {
  caught = true
}
Test.expect(caught, "size > 1MB must raise InvalidIdLengthError")

Test.pass("nanoid-error-handling")
