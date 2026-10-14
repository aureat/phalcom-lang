from universe.random import EntropyError

/// Raised when an alphabet contains invalid characters, duplicates, or unsupported cardinality.
class InvalidAlphabetError is Error {}

/// Raised when a requested identifier length is negative or exceeds the maximum supported length.
class InvalidIdLengthError is Error {}

/// A parsed identifier violates the expected length or validated alphabet.
data NanoIdParseError {
  InvalidLength(_ expected: Int, _ actual: Int)
  InvalidCharacter(_ offset: Int)
}

export InvalidAlphabetError, InvalidIdLengthError, NanoIdParseError, EntropyError
