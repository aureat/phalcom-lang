from .uuid import UUID
from .metadata import UUIDVersion
from .errors import UUIDParseError

/// The caller chooses Tag to identify an application domain. Parsing validates
/// UUID syntax; fromUUID explicitly asserts the domain of an existing value.
data TaggedUUID<Tag> private(_ value: UUID)

impl<Tag> TaggedUUID<Tag> {
  @class
  fromUUID(_ value: UUID) -> TaggedUUID<Tag> { TaggedUUID<Tag>(value) }

  @class
  parse(_ text: String) -> Result<TaggedUUID<Tag>, UUIDParseError> {
    match UUID.parse(text) {
      Result::Ok(value) => Result::Ok(TaggedUUID<Tag>(value))
      Result::Error(error) => Result::Error(error)
    }
  }

  @class
  v4() -> TaggedUUID<Tag> { TaggedUUID<Tag>(UUID.v4()) }

  @class
  v5(namespace: UUID, name: String) -> TaggedUUID<Tag> {
    TaggedUUID<Tag>(UUID.v5(namespace: namespace, name: name))
  }

  @class
  v7() -> TaggedUUID<Tag> { TaggedUUID<Tag>(UUID.v7()) }

  value -> UUID { self._value }
  toString -> String { self._value.toString }

  ==(_ other: Dynamic) -> Bool {
    if other is not TaggedUUID<Tag> { return false }
    self._value == other.value
  }
  hash -> Int { self._value.hash }
  compare(_ other: TaggedUUID<Tag>) -> Ordering { self._value.compare(other.value) }
  <(_ other: TaggedUUID<Tag>) -> Bool { self.compare(other) === Ordering::Less }
}

/// A valid UUID can still have another version or an unrecognized variant.
data UUIDv7ParseError {
  InvalidUUID(_ reason: UUIDParseError)
  UnexpectedVersion(_ observed: Option<UUIDVersion>)
}

/// A checked RFC-variant UUIDv7. Its Unix timestamp is always present.
data UUIDv7 private(_ value: UUID)

impl UUIDv7 {
  @class
  fromUUID(_ value: UUID) -> Result<UUIDv7, UUIDv7ParseError> {
    if not (value.version === Some(UUIDVersion::UnixTime)) {
      return Result::Error(UUIDv7ParseError::UnexpectedVersion(value.version))
    }
    Result::Ok(UUIDv7(value))
  }

  @class
  parse(_ text: String) -> Result<UUIDv7, UUIDv7ParseError> {
    match UUID.parse(text) {
      Result::Ok(value) => UUIDv7.fromUUID(value)
      Result::Error(error) => Result::Error(UUIDv7ParseError::InvalidUUID(error))
    }
  }

  @class
  generate() -> UUIDv7 { UUIDv7(UUID.v7()) }

  value -> UUID { self._value }
  timestamp -> Int { self._value.timestamp.unwrap }
  toString -> String { self._value.toString }

  ==(_ other: Dynamic) -> Bool {
    if other is not UUIDv7 { return false }
    self._value == other.value
  }
  hash -> Int { self._value.hash }
  compare(_ other: UUIDv7) -> Ordering { self._value.compare(other.value) }
  <(_ other: UUIDv7) -> Bool { self.compare(other) === Ordering::Less }
}

export TaggedUUID, UUIDv7, UUIDv7ParseError
