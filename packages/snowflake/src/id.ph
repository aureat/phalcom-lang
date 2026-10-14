from universe.collections.bytes import Bytes
from .codec import SnowflakeCodec
from .errors import SnowflakeParseError
from .layout import SnowflakeLayout

/// A nonnegative 63-bit Snowflake integer. The layout is supplied explicitly
/// when decoding non-Twitter IDs; it is not part of the integer's identity.
data SnowflakeId private(_ raw: Int)

impl SnowflakeId {
  @class
  fromInt(_ raw: Int) -> Result<SnowflakeId, SnowflakeParseError> {
    if raw < 0 or raw > 0x7fffffffffffffff { return Result::Error(SnowflakeParseError::OutOfRange) }
    Result::Ok(SnowflakeId(raw))
  }

  @class
  internal assembled(_ raw: Int) -> SnowflakeId {
    SnowflakeId.fromInt(raw).unwrap
  }

  value -> Int { self._raw }
  toString -> String { self._raw.toString }
  base2 -> String { self._raw.toString(radix: 2) }
  base32 -> String { SnowflakeCodec.encode(self._raw, "ybndrfg8ejkmcpqxot1uwisza345h769") }
  base36 -> String { self._raw.toString(radix: 36) }
  base58 -> String { SnowflakeCodec.encode(self._raw, "123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ") }
  base64 -> String { SnowflakeCodec.encodeBase64(self.toString) }
  json -> String { "\"" + self.toString + "\"" }

  /// Fixed-width unsigned big-endian encoding; distinct from Go's decimal Bytes().
  toBytes() -> Bytes {
    let out = Bytes.new(8)
    let mut i = 0
    while i < 8 {
      out.set(i, (self._raw >> ((7 - i) * 8)) & 255)
      i = i + 1
    }
    out
  }

  /// Go Snowflake's Bytes() representation: UTF-8 decimal text, not binary.
  toTextBytes() -> Bytes { Bytes.fromString(self.toString) }

  @class
  fromBytes(_ bytes: Bytes) -> Result<SnowflakeId, SnowflakeParseError> {
    if bytes.size != 8 { return Result::Error(SnowflakeParseError::InvalidLength(bytes.size)) }
    let mut raw = 0
    let mut i = 0
    while i < 8 {
      raw = (raw << 8) | bytes.at(i)
      i = i + 1
    }
    SnowflakeId.fromInt(raw)
  }

  @class
  parseTextBytes(_ bytes: Bytes) -> Result<SnowflakeId, SnowflakeParseError> {
    match bytes.utf8 {
      Option::Some(text) => SnowflakeId.parse(text)
      Option::None => Result::Error(SnowflakeParseError::InvalidFormat)
    }
  }

  @class
  private fromEncoding(_ text: String, _ alphabet: String, _ ignoreCase: Bool) -> Result<SnowflakeId, SnowflakeParseError> {
    match SnowflakeCodec.decode(text, alphabet, ignoreCase) {
      Result::Ok(value) => SnowflakeId.fromInt(value)
      Result::Error(error) => Result::Error(error)
    }
  }

  @class
  parse(_ text: String) -> Result<SnowflakeId, SnowflakeParseError> {
    SnowflakeId.fromEncoding(text, "0123456789", false)
  }

  @class
  parseBase2(_ text: String) -> Result<SnowflakeId, SnowflakeParseError> {
    SnowflakeId.fromEncoding(text, "01", false)
  }

  @class
  parseBase32(_ text: String) -> Result<SnowflakeId, SnowflakeParseError> {
    SnowflakeId.fromEncoding(text, "ybndrfg8ejkmcpqxot1uwisza345h769", false)
  }

  @class
  parseBase36(_ text: String) -> Result<SnowflakeId, SnowflakeParseError> {
    SnowflakeId.fromEncoding(text, "0123456789abcdefghijklmnopqrstuvwxyz", true)
  }

  @class
  parseBase58(_ text: String) -> Result<SnowflakeId, SnowflakeParseError> {
    SnowflakeId.fromEncoding(text, "123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ", false)
  }

  @class
  parseBase64(_ text: String) -> Result<SnowflakeId, SnowflakeParseError> {
    match SnowflakeCodec.decodeBase64(text) {
      Result::Ok(decimal) => SnowflakeId.parse(decimal)
      Result::Error(error) => Result::Error(error)
    }
  }

  @class
  parseJSON(_ text: String) -> Result<SnowflakeId, SnowflakeParseError> {
    let bytes = Bytes.fromString(text)
    if bytes.size < 3 or bytes.at(0) != 34 or bytes.at(bytes.size - 1) != 34 {
      return Result::Error(SnowflakeParseError::InvalidFormat)
    }
    SnowflakeId.parse(bytes.slice(1, bytes.size - 1).utf8.unwrap)
  }

  /// Default Twitter interpretation. Custom layouts must use the overloads
  /// taking `using:`; the same raw bits have different meanings by layout.
  timeMillis -> Int { self.timeMillis(using: SnowflakeLayout.twitter) }
  node -> Int { self.node(using: SnowflakeLayout.twitter) }
  step -> Int { self.step(using: SnowflakeLayout.twitter) }

  timeMillis(using layout: SnowflakeLayout) -> Int {
    layout.epochMillis + (self._raw >> layout.timeShift)
  }

  node(using layout: SnowflakeLayout) -> Int {
    (self._raw >> layout.sequenceBits) & layout.nodeMax
  }

  step(using layout: SnowflakeLayout) -> Int { self._raw & layout.sequenceMax }

  ==(_ other: Dynamic) -> Bool {
    if other is not SnowflakeId { return false }
    self._raw == other.value
  }

  hash -> Int { self._raw.hash }
  compare(_ other: SnowflakeId) -> Ordering { self._raw.compare(other.value) }
  <(_ other: SnowflakeId) -> Bool { self.compare(other) === Ordering::Less }
}

export SnowflakeId
