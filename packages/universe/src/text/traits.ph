from universe.scalar.string import (
  StringChars,
  StringByteSequence
)
from universe.text.substring import (
  Substring,
  SubstringChars,
  SubstringBytes
)

/// A type representing a sequence of UTF-8 encoded text.
trait Text {
  type Chars
  type Bytes

  /// The length of the text in bytes.
  byteCount -> Int

  /// Whether the text contains zero bytes.
  isEmpty -> Bool

  /// The number of Unicode scalars contained in the text.
  charCount -> Int

  /// An iterable sequence of Unicode scalar characters.
  chars -> Self::Chars

  /// An iterable sequence of UTF-8 bytes.
  utf8 -> Self::Bytes

  /// Produces a non-allocating Substring view over the text.
  asSubstring -> Substring
}

impl Text for String {
  type Chars = StringChars
  type Bytes = StringByteSequence
}

impl Text for Substring {
  type Chars = SubstringChars
  type Bytes = SubstringBytes
}

export Text
