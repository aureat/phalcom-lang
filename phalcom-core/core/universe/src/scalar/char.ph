@!documentation("A Unicode scalar value, distinct from Int and String.")

from universe.text.substring import Substring

@sealed
native class Char is Object {
  native codePoint -> Int
  native utf8Length -> Int
  native toString -> String
  native isAscii -> Bool
  native isAsciiDigit -> Bool
  native isAsciiAlphabetic -> Bool
  native isAsciiLowercase -> Bool
  native isAsciiUppercase -> Bool
  native isAsciiAlphanumeric -> Bool
  native isAsciiHexDigit -> Bool
  native isWhitespace -> Bool
  native isControl -> Bool
  native hash -> Int
  native simpleLowercase -> Char
  native simpleUppercase -> Char
  @class
  native fromCodePoint(_ value: Int) -> Option<Char>

  /// Canonical Telos source representation of this character scalar.
  repr -> String {
    let cp = codePoint
    if cp == 39 { return "'\\''" }
    if cp == 92 { return "'\\\\'" }
    if cp == 10 { return "'\\n'" }
    if cp == 13 { return "'\\r'" }
    if cp == 9 { return "'\\t'" }
    if cp == 0 { return "'\\0'" }
    if cp < 32 { return "'\\u{" + cp.toString + "}'" }
    return "'" + toString + "'"
  }

  /// Searches for the first occurrence of this character in `source` from the beginning.
  findIn(_ source: Substring) -> Option<(Int, Int)> {
    self.findIn(source, 0)
  }

  /// Searches for the first occurrence of this character in `source` at or after byte offset `offset`.
  findIn(_ source: Substring, _ offset: Int) -> Option<(Int, Int)> {
    if offset < 0 or offset >= source.byteCount {
      return None
    }
    let mut i = offset
    let srcLen = source.byteCount
    while i < srcLen {
      let optCh = source.decodeCharAt(i)
      match optCh {
        Option::Some(ch) => {
          if ch == self {
            return Some((i, i + ch.utf8Length))
          }
          i = i + ch.utf8Length
        }
        _ => {
          i = i + 1
        }
      }
    }
    return None
  }

  /// Searches for the first occurrence of this character in `source` at or after byte offset `from`.
  findIn(_ source: Substring, from offset: Int) -> Option<(Int, Int)> {
    self.findIn(source, offset)
  }
}
