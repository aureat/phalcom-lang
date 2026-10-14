from universe.collections.bytes import Bytes
from .alphabet import Alphabet
from .generator import NanoIdGenerator
from .errors import NanoIdParseError

/// Convenient facade for default cryptographically secure identifier generation.
class NanoId {
  /// Generates a secure URL-friendly identifier of 21 characters.
  @class
  generate() -> String {
    NanoIdGenerator.default.generate()
  }

  /// Generates a secure URL-friendly identifier of the specified length.
  @class
  generate(_ size: Int) -> String {
    NanoIdGenerator.default.generate(size)
  }

  /// Generates a domain-tagged identifier with default length.
  @class
  typed<Tag>() -> Id<Tag> {
    Id<Tag>.wrap(NanoId.generate())
  }

  /// Generates a domain-tagged identifier of specified length.
  @class
  typed<Tag>(_ size: Int) -> Id<Tag> {
    Id<Tag>.wrap(NanoId.generate(size))
  }
}

/// Domain-tagged identifier. Tag is part of the exact applied data type.
data Id<Tag> private(_ value: String)

impl<Tag> Id<Tag> {
  value -> String { self._value }
  toString -> String { self._value }

  /// Explicitly asserts a domain tag without checking the alphabet or length.
  /// Use parse when handling untrusted text.
  @class
  wrap(_ value: String) -> Id<Tag> { Id<Tag>(value) }

  @class
  parse(_ text: String, _ alphabet: Alphabet, _ expectedLength: Int) -> Result<Id<Tag>, NanoIdParseError> {
    if expectedLength < 0 or text.byteCount != expectedLength {
      return Result::Error(NanoIdParseError::InvalidLength(expectedLength, text.byteCount))
    }
    let textBytes = Bytes.fromString(text)
    let alphaBytes = Bytes.fromString(alphabet.symbols)
    let mut i = 0
    while i < textBytes.size {
      let b = textBytes.at(i)
      let mut found = false
      let mut j = 0
      while j < alphaBytes.size {
        if alphaBytes.at(j) == b {
          found = true
          break
        }
        j = j + 1
      }
      if not found { return Result::Error(NanoIdParseError::InvalidCharacter(i)) }
      i = i + 1
    }
    Result::Ok(Id<Tag>(text))
  }
}

/// A configured generator whose results retain one domain tag.
class TypedNanoIdGenerator<Tag> {
  _generator: NanoIdGenerator

  @constructor
  private create(_ generator: NanoIdGenerator) { self._generator = generator }

  @class
  fromGenerator(_ generator: NanoIdGenerator) -> TypedNanoIdGenerator<Tag> {
    TypedNanoIdGenerator<Tag>.create(generator)
  }

  @class
  configured(_ alphabet: Alphabet, _ size: Int) -> TypedNanoIdGenerator<Tag> {
    TypedNanoIdGenerator<Tag>.fromGenerator(NanoIdGenerator.new(alphabet, size))
  }

  alphabet -> Alphabet { self._generator.alphabet }
  size -> Int { self._generator.size }
  generate() -> Id<Tag> { Id<Tag>.wrap(self._generator.generate()) }
  generate(_ length: Int) -> Id<Tag> { Id<Tag>.wrap(self._generator.generate(length)) }
}

export NanoId, Id, TypedNanoIdGenerator
