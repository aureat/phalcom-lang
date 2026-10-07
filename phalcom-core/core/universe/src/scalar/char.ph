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
  @class
  native fromCodePoint(_ value: Int) -> Option<Char>
}
