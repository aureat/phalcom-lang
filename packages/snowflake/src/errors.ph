/// An external representation is malformed or outside the 63-bit ID domain.
data SnowflakeParseError {
  InvalidLength(_ actual: Int)
  InvalidDigit(_ offset: Int)
  InvalidFormat
  OutOfRange
}

/// Generation cannot safely advance without duplicating, wrapping, or reordering IDs.
class SnowflakeGenerationError is Error {}

export SnowflakeParseError, SnowflakeGenerationError
