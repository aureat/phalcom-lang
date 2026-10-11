@!documentation("Character-level parser factories over TextSource.")

from .core.error import ParseError
from .core.result import ParseResult
from .core.rule import Rule
from .source.text_source import TextSource

/// Text-level parser conveniences layered above the generic machinery.
///
/// Tokens are byte values (`Int`). Predicates receive raw bytes; ASCII
/// grammar structure therefore costs one comparison per token, and
/// non-ASCII payload flows through byte-wise predicates unchanged.
class Text {
  /// Matches any single byte satisfying `predicate`.
  @class
  satisfy(_ expected: String, _ predicate: (Int) -> Bool) -> Rule<TextSource, Int> {
    Rule.new |cursor| {
      let source = cursor.source
      if cursor.offset < source.size {
        let token = source.at(cursor.offset)
        if predicate.call(token) {
          ParseResult<TextSource, Int>::Success(token, cursor.advance)
        } else {
          ParseResult<TextSource, Int>::Failure(
            ParseError.expecting(cursor.offset, expected),
            false
          )
        }
      } else {
        ParseResult<TextSource, Int>::Failure(
          ParseError.expecting(cursor.offset, expected),
          false
        )
      }
    }
  }

  /// Matches the single byte of the one-byte string `c`.
  @class
  char(_ c: String) -> Rule<TextSource, Int> {
    let expected = c.bytes.at(0).unwrapOr(-1)
    Text.satisfy(c) |byte| { byte == expected }
  }

  /// Matches any one of the bytes in `chars`.
  @class
  oneOf(_ chars: String) -> Rule<TextSource, Int> {
    Text.satisfy("one of '" + chars + "'") |byte| {
      let mut found = false
      let mut i = 0
      while i < chars.size {
        if chars.bytes.at(i).unwrapOr(-1) == byte {
          found = true
        }
        i = i + 1
      }
      found
    }
  }

  /// Matches any single byte not present in `chars`.
  @class
  noneOf(_ chars: String) -> Rule<TextSource, Int> {
    Text.satisfy("none of '" + chars + "'") |byte| {
      let mut found = false
      let mut i = 0
      while i < chars.size {
        if chars.bytes.at(i).unwrapOr(-1) == byte {
          found = true
        }
        i = i + 1
      }
      not found
    }
  }

  /// Matches the exact string `s` and produces it.
  @class
  literal(_ s: String) -> Rule<TextSource, String> {
    Rule.new |cursor| {
      let source = cursor.source
      let width = s.size
      let mut fits = cursor.offset + width <= source.size
      if fits {
        let mut matched = true
        let mut i = 0
        while i < width {
          if source.at(cursor.offset + i) != s.bytes.at(i).unwrapOr(-1) {
            matched = false
          }
          i = i + 1
        }
        fits = matched
      }
      if fits {
        ParseResult<TextSource, String>::Success(s, cursor.advanceBy(width))
      } else {
        ParseResult<TextSource, String>::Failure(
          ParseError.expecting(cursor.offset, "\"" + s + "\""),
          false
        )
      }
    }
  }

  /// Matches `s` only when not followed by an ASCII letter, digit, or
  /// underscore: a whole keyword boundary.
  @class
  keyword(_ s: String) -> Rule<TextSource, String> {
    let boundary = Text.satisfy("identifier byte") |byte| {
      if (byte >= 97 and byte <= 122) or (byte >= 65 and byte <= 90) or (byte >= 48 and byte <= 57) or byte == 95 {
        true
      } else {
        false
      }
    }
    Text.literal(s).before(Text.notFollowedBy(boundary, "end of keyword '" + s + "'"))
  }

  /// Negative lookahead at the current position.
  @class
  notFollowedBy<U>(_ other: Rule<TextSource, U>, _ boundary: String) -> Rule<TextSource, Unit> {
    Rule<TextSource, Unit>.pure(()).notFollowedBy(other, boundary)
  }

  /// ASCII digit `0`-`9`.
  @class
  asciiDigit -> Rule<TextSource, Int> {
    Text.satisfy("digit") |byte| { if byte >= 48 and byte <= 57 { true } else { false } }
  }

  /// ASCII letter `a`-`z` or `A`-`Z`.
  @class
  asciiLetter -> Rule<TextSource, Int> {
    Text.satisfy("letter") |byte| { if (byte >= 97 and byte <= 122) or (byte >= 65 and byte <= 90) { true } else { false } }
  }

  /// Hexadecimal digit `0`-`9`, `a`-`f`, or `A`-`F`.
  @class
  hexDigit -> Rule<TextSource, Int> {
    Text.satisfy("hexadecimal digit") |byte| {
      if (byte >= 48 and byte <= 57) or (byte >= 97 and byte <= 102) or (byte >= 65 and byte <= 70) {
        true
      } else {
        false
      }
    }
  }

  /// Whitespace byte: space, tab, carriage return, or newline.
  @class
  whitespaceByte -> Rule<TextSource, Int> {
    Text.satisfy("whitespace") |byte| { if byte == 32 or byte == 9 or byte == 13 or byte == 10 { true } else { false } }
  }

  /// Contiguous run of bytes satisfying `predicate`, possibly empty.
  @class
  takeWhile(_ predicate: (Int) -> Bool) -> Rule<TextSource, String> {
    Rule.new |cursor| {
      let source = cursor.source
      let mut end = cursor.offset
      while end < source.size and predicate.call(source.at(end)) {
        end = end + 1
      }
      ParseResult<TextSource, String>::Success(
        source.slice(cursor.offset, end),
        cursor.advanceBy(end - cursor.offset)
      )
    }
  }

  /// Contiguous run of bytes satisfying `predicate`; at least one byte.
  @class
  takeWhile1(_ expected: String, _ predicate: (Int) -> Bool) -> Rule<TextSource, String> {
    Rule.new |cursor| {
      let source = cursor.source
      let mut end = cursor.offset
      while end < source.size and predicate.call(source.at(end)) {
        end = end + 1
      }
      if end == cursor.offset {
        ParseResult<TextSource, String>::Failure(
          ParseError.expecting(cursor.offset, expected),
          false
        )
      } else {
        ParseResult<TextSource, String>::Success(
          source.slice(cursor.offset, end),
          cursor.advanceBy(end - cursor.offset)
        )
      }
    }
  }

  /// Succeeds only at end of input.
  @class
  end -> Rule<TextSource, Unit> {
    Rule.new |cursor| {
      if cursor.atEnd {
        ParseResult<TextSource, Unit>::Success((), cursor)
      } else {
        ParseResult<TextSource, Unit>::Failure(
          ParseError.expecting(cursor.offset, "end of input"),
          false
        )
      }
    }
  }

  /// A rule succeeding with `value` without consuming input.
  @class
  pure<T>(_ value: T) -> Rule<TextSource, T> {
    Rule<TextSource, T>.pure(value)
  }

  /// A rule always failing with `message`.
  @class
  fail<T>(_ message: String) -> Rule<TextSource, T> {
    Rule<TextSource, T>.fail(message)
  }
}

export Text
