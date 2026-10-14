from universe.collections.bytes import Bytes
from .errors import SnowflakeParseError

/// Encoding helpers shared by the value's text representations. All decoders
/// accumulate bounded positive Ints; they never wrap the host's signed i64.
class SnowflakeCodec {
  @class
  internal encode(_ raw: Int, _ alphabet: String) -> String {
    let digits = Bytes.fromString(alphabet)
    let base = digits.size
    if raw == 0 { return digits.slice(0, 1).utf8.unwrap }
    let out = Bytes.new(64)
    let mut end = 64
    let mut value = raw
    while value > 0 {
      end = end - 1
      out.set(end, digits.at(value % base))
      value = value ~/ base
    }
    out.slice(end, 64).utf8.unwrap
  }

  @class
  internal decode(_ text: String, _ alphabet: String, _ ignoreCase: Bool) -> Result<Int, SnowflakeParseError> {
    let bytes = Bytes.fromString(text)
    if bytes.size == 0 { return Result::Error(SnowflakeParseError::InvalidLength(0)) }
    if bytes.size > 63 { return Result::Error(SnowflakeParseError::OutOfRange) }
    let digits = Bytes.fromString(alphabet)
    let base = digits.size
    let mut value = 0
    let mut i = 0
    while i < bytes.size {
      let byte = bytes.at(i)
      let normalized = if ignoreCase and byte >= 65 and byte <= 90 { byte + 32 } else { byte }
      let mut digit = -1
      let mut j = 0
      while j < base {
        if normalized == digits.at(j) { digit = j; break }
        j = j + 1
      }
      if digit < 0 { return Result::Error(SnowflakeParseError::InvalidDigit(i)) }
      if value > (0x7fffffffffffffff - digit) ~/ base {
        return Result::Error(SnowflakeParseError::OutOfRange)
      }
      value = value * base + digit
      i = i + 1
    }
    Result::Ok(value)
  }

  /// Go Snowflake's Base64 form encodes the decimal ASCII representation,
  /// rather than an eight-byte binary integer.
  @class
  internal encodeBase64(_ decimal: String) -> String {
    let input = Bytes.fromString(decimal)
    let alphabet = Bytes.fromString("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/")
    let out = Bytes.new(((input.size + 2) ~/ 3) * 4)
    let mut src = 0
    let mut dst = 0
    while src < input.size {
      let a = input.at(src)
      let b = if src + 1 < input.size { input.at(src + 1) } else { 0 }
      let c = if src + 2 < input.size { input.at(src + 2) } else { 0 }
      out.set(dst, alphabet.at(a >> 2))
      out.set(dst + 1, alphabet.at(((a & 3) << 4) | (b >> 4)))
      out.set(dst + 2, if src + 1 < input.size { alphabet.at(((b & 15) << 2) | (c >> 6)) } else { 61 })
      out.set(dst + 3, if src + 2 < input.size { alphabet.at(c & 63) } else { 61 })
      src = src + 3
      dst = dst + 4
    }
    out.utf8.unwrap
  }

  @class
  private base64Digit(_ byte: Int, _ alphabet: Bytes) -> Int {
    let mut i = 0
    while i < 64 {
      if alphabet.at(i) == byte { return i }
      i = i + 1
    }
    -1
  }

  @class
  internal decodeBase64(_ text: String) -> Result<String, SnowflakeParseError> {
    let input = Bytes.fromString(text)
    if input.size == 0 or input.size % 4 != 0 { return Result::Error(SnowflakeParseError::InvalidFormat) }
    if input.size > 28 { return Result::Error(SnowflakeParseError::OutOfRange) }
    let alphabet = Bytes.fromString("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/")
    let padding = if input.at(input.size - 2) == 61 { 2 } else if input.at(input.size - 1) == 61 { 1 } else { 0 }
    let out = Bytes.new((input.size ~/ 4) * 3 - padding)
    let mut src = 0
    let mut dst = 0
    while src < input.size {
      let a = SnowflakeCodec.base64Digit(input.at(src), alphabet)
      let b = SnowflakeCodec.base64Digit(input.at(src + 1), alphabet)
      let c = if input.at(src + 2) == 61 { 0 } else { SnowflakeCodec.base64Digit(input.at(src + 2), alphabet) }
      let d = if input.at(src + 3) == 61 { 0 } else { SnowflakeCodec.base64Digit(input.at(src + 3), alphabet) }
      if a < 0 or b < 0 or c < 0 or d < 0 {
        return Result::Error(SnowflakeParseError::InvalidFormat)
      }
      let last = src + 4 == input.size
      if (input.at(src + 2) == 61 and (not last or input.at(src + 3) != 61)) or
        (input.at(src + 3) == 61 and not last) {
        return Result::Error(SnowflakeParseError::InvalidFormat)
      }
      if padding == 2 and last and (b & 15) != 0 { return Result::Error(SnowflakeParseError::InvalidFormat) }
      if padding == 1 and last and (c & 3) != 0 { return Result::Error(SnowflakeParseError::InvalidFormat) }
      let bits = (a << 18) | (b << 12) | (c << 6) | d
      out.set(dst, (bits >> 16) & 255)
      if dst + 1 < out.size { out.set(dst + 1, (bits >> 8) & 255) }
      if dst + 2 < out.size { out.set(dst + 2, bits & 255) }
      dst = dst + 3
      src = src + 4
    }
    match out.utf8 {
      Option::Some(decoded) => Result::Ok(decoded)
      Option::None => Result::Error(SnowflakeParseError::InvalidFormat)
    }
  }
}

export SnowflakeCodec
