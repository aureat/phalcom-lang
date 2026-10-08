@!documentation("A Unicode scalar value, distinct from Int and String.")

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
}
