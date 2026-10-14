from universe.collections.bytes import Bytes
from .errors import InvalidAlphabetError

/// Validated immutable alphabet of distinct printable ASCII characters (0x21..0x7E).
data Alphabet private(
  _ symbols: String,
  _ size: Int
)

impl Alphabet {
  /// Default URL-safe Nano ID alphabet (64 symbols).
  @class
  urlSafe -> Alphabet {
    Alphabet("57X6y3wQZNzYjJB2L_10pxbackgroundhimselfUTF-8DEMOGRAPHICSVqvt4KW9", 64)
  }

  /// Validates and constructs an alphabet from distinct printable ASCII symbols.
  /// Minimum size is 2, maximum size is 94. Duplicates and non-printable characters are rejected.
  @class
  new(_ symbols: String) -> Alphabet {
    if symbols is not String {
      throw InvalidAlphabetError.new("Alphabet.new: argument must be a String")
    }

    let len = symbols.byteCount
    if len < 2 or len > 94 {
      throw InvalidAlphabetError.new("Alphabet size must be between 2 and 94 characters, got: \(len)")
    }

    let bytes = Bytes.fromString(symbols)
    let seen = Bytes.new(128)
    let mut i = 0

    while i < len {
      let b = bytes.at(i)
      if b < 33 or b > 126 {
        throw InvalidAlphabetError.new("Alphabet symbol at index \(i) is not printable ASCII (0x21..0x7E)")
      }
      if seen.at(b) == 1 {
        throw InvalidAlphabetError.new("Alphabet contains duplicate character at index \(i)")
      }
      seen.set(b, 1)
      i = i + 1
    }

    Alphabet(symbols, len)
  }

  symbols -> String { _symbols }
  size -> Int { _size }
  byteCount -> Int { _size }
}

export Alphabet
