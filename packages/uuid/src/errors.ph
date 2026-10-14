/// Text is not a supported encoding of exactly 128 UUID bits.
data UUIDParseError {
  InvalidLength(_ actual: Int)
  InvalidCharacter(_ offset: Int)
  InvalidSeparator(_ offset: Int)
  InvalidFormat
}

/// Binary UUID input must contain exactly 16 octets.
data UUIDBytesError {
  InvalidLength(_ actual: Int)
}

/// The clock cannot be represented, or a monotonic generator cannot advance.
class UUIDGenerationError is Error {}

export UUIDParseError, UUIDBytesError, UUIDGenerationError
