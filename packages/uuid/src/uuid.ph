from universe.collections.bytes import Bytes
from universe.random import SecureRandom
from universe.time.timestamp import Timestamp
from universe.crypto.sha1 import SHA1
from .errors import UUIDParseError, UUIDBytesError, UUIDGenerationError
from .metadata import UUIDVersion, UUIDVariant

/// Immutable 128-bit value in network byte order. Both Int components are
/// restricted to the unsigned 64-bit range, irrespective of Int's precision.
data UUID private(_ high: Int, _ low: Int)

impl UUID {
  @class
  internal fromWords(_ high: Int, _ low: Int) -> UUID {
    if high < 0 or high > 0xffffffffffffffff or low < 0 or low > 0xffffffffffffffff {
      throw ArgumentError.new("UUID words must be unsigned 64-bit values")
    }
    UUID(high, low)
  }

  @class
  internal fromBytesUnchecked(_ bytes: Bytes) -> UUID {
    if bytes.size != 16 { throw ArgumentError.new("UUID requires exactly 16 bytes") }
    let mut high = 0
    let mut low = 0
    let mut i = 0
    while i < 8 {
      high = (high << 8) | bytes.at(i)
      low = (low << 8) | bytes.at(i + 8)
      i = i + 1
    }
    UUID(high, low)
  }

  @class
  fromBytes(_ bytes: Bytes) -> Result<UUID, UUIDBytesError> {
    if bytes.size != 16 { return Result::Error(UUIDBytesError::InvalidLength(bytes.size)) }
    Result::Ok(UUID.fromBytesUnchecked(bytes))
  }

  toBytes() -> Bytes {
    let out = Bytes.new(16)
    let mut i = 0
    while i < 8 {
      out.set(i, (self._high >> ((7 - i) * 8)) & 255)
      out.set(i + 8, (self._low >> ((7 - i) * 8)) & 255)
      i = i + 1
    }
    out
  }

  /// Accepts lowercase/uppercase canonical or simple text, a lowercase URN
  /// prefix, or a braced canonical representation. No whitespace is stripped.
  @class
  parse(_ text: String) -> Result<UUID, UUIDParseError> {
    UUID.parseText(text, false)
  }

  /// Requires precisely the lowercase 8-4-4-4-12 textual representation.
  @class
  parseCanonical(_ text: String) -> Result<UUID, UUIDParseError> {
    UUID.parseText(text, true)
  }

  @class
  private parseText(_ text: String, _ strict: Bool) -> Result<UUID, UUIDParseError> {
    let length = text.byteCount
    if strict and length != 36 { return Result::Error(UUIDParseError::InvalidLength(length)) }
    if length != 32 and length != 36 and length != 38 and length != 45 {
      return Result::Error(UUIDParseError::InvalidLength(length))
    }
    let input = Bytes.fromString(text)
    let mut start = 0
    let mut canonical = length != 32
    if length == 38 {
      if strict or input.at(0) != 123 or input.at(37) != 125 {
        return Result::Error(UUIDParseError::InvalidFormat)
      }
      start = 1
    }
    if length == 45 {
      if strict { return Result::Error(UUIDParseError::InvalidFormat) }
      let prefix = Bytes.fromString("urn:uuid:")
      let mut p = 0
      while p < 9 {
        if input.at(p) != prefix.at(p) { return Result::Error(UUIDParseError::InvalidFormat) }
        p = p + 1
      }
      start = 9
    }

    let mut high = 0
    let mut low = 0
    let mut digits = 0
    let mut pos = start
    let end = if canonical { start + 36 } else { start + 32 }
    while pos < end {
      let offset = pos - start
      let byte = input.at(pos)
      if canonical and (offset == 8 or offset == 13 or offset == 18 or offset == 23) {
        if byte != 45 { return Result::Error(UUIDParseError::InvalidSeparator(pos)) }
      } else {
        let mut digit = -1
        if byte >= 48 and byte <= 57 { digit = byte - 48 }
        if byte >= 97 and byte <= 102 { digit = byte - 87 }
        if not strict and byte >= 65 and byte <= 70 { digit = byte - 55 }
        if digit < 0 { return Result::Error(UUIDParseError::InvalidCharacter(pos)) }
        if digits < 16 { high = (high << 4) | digit } else { low = (low << 4) | digit }
        digits = digits + 1
      }
      pos = pos + 1
    }
    Result::Ok(UUID(high, low))
  }

  @class
  nil -> UUID { UUID(0, 0) }
  @class
  max -> UUID { UUID(0xffffffffffffffff, 0xffffffffffffffff) }
  isNil -> Bool { self._high == 0 and self._low == 0 }
  isMax -> Bool { self._high == 0xffffffffffffffff and self._low == 0xffffffffffffffff }

  variant -> UUIDVariant {
    let field = (self._low >> 56) & 255
    if (field & 0x80) == 0 { return UUIDVariant::NCS }
    if (field & 0xc0) == 0x80 { return UUIDVariant::RFC9562 }
    if (field & 0xe0) == 0xc0 { return UUIDVariant::Microsoft }
    UUIDVariant::Future
  }

  version -> Option<UUIDVersion> {
    if self.isNil or self.isMax or not (self.variant === UUIDVariant::RFC9562) { return None }
    let field = (self._high >> 12) & 15
    if field == 1 { return Some(UUIDVersion::TimeBased) }
    if field == 2 { return Some(UUIDVersion::DceSecurity) }
    if field == 3 { return Some(UUIDVersion::NameBasedMD5) }
    if field == 4 { return Some(UUIDVersion::Random) }
    if field == 5 { return Some(UUIDVersion::NameBasedSHA1) }
    if field == 6 { return Some(UUIDVersion::ReorderedTime) }
    if field == 7 { return Some(UUIDVersion::UnixTime) }
    if field == 8 { return Some(UUIDVersion::Custom) }
    Some(UUIDVersion::Reserved(field))
  }

  /// Unix milliseconds for RFC-variant version 7; absent for other layouts.
  timestamp -> Option<Int> {
    match self.version {
      Option::Some(UUIDVersion::UnixTime) => Some(self._high >> 16)
      _ => None
    }
  }

  @class
  internal hex(_ digit: Int) -> Int {
    if digit < 10 { 48 + digit } else { 87 + digit }
  }

  private format(_ canonical: Bool) -> String {
    let out = Bytes.new(if canonical { 36 } else { 32 })
    let mut i = 0
    let mut position = 0
    while i < 32 {
      if canonical and (i == 8 or i == 12 or i == 16 or i == 20) {
        out.set(position, 45)
        position = position + 1
      }
      let nibble = if i < 16 {
        (self._high >> ((15 - i) * 4)) & 15
      } else {
        (self._low >> ((31 - i) * 4)) & 15
      }
      out.set(position, UUID.hex(nibble))
      position = position + 1
      i = i + 1
    }
    out.utf8.unwrap
  }

  toString -> String { self.format(true) }
  simple -> String { self.format(false) }
  urn -> String { "urn:uuid:" + self.toString }
  braced -> String { "{" + self.toString + "}" }

  internal high -> Int { self._high }
  internal low -> Int { self._low }

  ==(_ other: Dynamic) -> Bool {
    if other is not UUID { return false }
    self._high == other.high and self._low == other.low
  }

  hash -> Int { ((self._high.hash * 31) + self._low.hash) % 999999937 }

  compare(_ other: UUID) -> Ordering {
    if self._high < other.high { return Ordering::Less }
    if self._high > other.high { return Ordering::Greater }
    self._low.compare(other.low)
  }

  <(_ other: UUID) -> Bool { self.compare(other) === Ordering::Less }

  @class
  v4() -> UUID {
    let bytes = SecureRandom.system.bytes(16)
    if bytes.size != 16 { throw UUIDGenerationError.new("UUIDv4 entropy length mismatch") }
    bytes.set(6, (bytes.at(6) & 15) | 0x40)
    bytes.set(8, (bytes.at(8) & 63) | 0x80)
    UUID.fromBytesUnchecked(bytes)
  }

  @class
  internal fromV7Parts(_ millis: Int, _ entropy: Bytes) -> UUID {
    if millis < 0 or millis > 0xffffffffffff {
      throw UUIDGenerationError.new("UUIDv7 timestamp exceeds 48-bit Unix milliseconds")
    }
    if entropy.size != 10 { throw UUIDGenerationError.new("UUIDv7 entropy length mismatch") }
    let high = (millis << 16) | 0x7000 | (((entropy.at(0) << 8) | entropy.at(1)) & 0xfff)
    let mut low = 0
    let mut i = 2
    while i < 10 {
      low = (low << 8) | entropy.at(i)
      i = i + 1
    }
    UUID(high, (low & 0x3fffffffffffffff) | 0x8000000000000000)
  }

  @class
  v7() -> UUID {
    let millis = Timestamp.now.unixNanoseconds ~/ 1000000
    if millis < 0 or millis > 0xffffffffffff {
      throw UUIDGenerationError.new("UUIDv7 timestamp exceeds 48-bit Unix milliseconds")
    }
    UUID.fromV7Parts(millis, SecureRandom.system.bytes(10))
  }

  /// RFC 9562 name-based SHA-1 over binary namespace and raw UTF-8 name bytes.
  @class
  v5(namespace: UUID, name: String) -> UUID {
    UUID.v5Bytes(namespace: namespace, name: Bytes.fromString(name))
  }

  @class
  v5Bytes(namespace: UUID, name: Bytes) -> UUID {
    let digest = SHA1.hash(namespace.toBytes().concat(name))
    let bytes = digest.slice(0, 16)
    bytes.set(6, (bytes.at(6) & 15) | 0x50)
    bytes.set(8, (bytes.at(8) & 63) | 0x80)
    UUID.fromBytesUnchecked(bytes)
  }

  @class
  namespaceDNS -> UUID { UUID.parseCanonical("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap }
  @class
  namespaceURL -> UUID { UUID.parseCanonical("6ba7b811-9dad-11d1-80b4-00c04fd430c8").unwrap }
  @class
  namespaceOID -> UUID { UUID.parseCanonical("6ba7b812-9dad-11d1-80b4-00c04fd430c8").unwrap }
  @class
  namespaceX500 -> UUID { UUID.parseCanonical("6ba7b814-9dad-11d1-80b4-00c04fd430c8").unwrap }
}

export UUID
