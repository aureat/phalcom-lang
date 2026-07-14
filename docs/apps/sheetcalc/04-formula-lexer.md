# SheetCalc — Formula Lexer

Part of the [SheetCalc specification](README.md).
Grounded in [00-language-findings.md](00-language-findings.md) §3, §5, §10.

## 1. Scope

`Lexer.tokenize(source)` — `String -> Result<List<Token>, LexError>`.

Input is one formula's full source text, leading `=` included — e.g. the
Phalcom string literal `"=SUM(A1:A3)*2"` from the architecture doc's data-flow
example (01-architecture.md §4). The lexer does not know the grammar; it does
not know that `=` must lead, that `SUM` takes a range argument, or that `:`
only makes sense between two refs. Those are the parser's job
([05-formula-parser.md](05-formula-parser.md)). The lexer's only contract is:
turn a byte string into a flat `List<Token>` ending in `#eof`, or fail with a
positioned `LexError`.

`lex/` may import only `support/` (01-architecture.md §3). It has no
dependency on `grid/` — a `Ref` token carries its raw lexeme text, not a
constructed `Ref` object; decoding happens in `parse/`, which is allowed to
import `grid/` for exactly that reason.

## 2. Token model

```phalcom
// lex/token.ph
class Token {
  // A structural token: operators, punctuation, EOF. No payload beyond the
  // raw lexeme.
  construct new(kind, text, pos) {
    _kind = kind
    _text = text
    _pos = pos
    _value = None
  }

  // A literal token: carries a decoded Phalcom value alongside the raw
  // lexeme — a Number for #number, a String for #text (post-unescape), a
  // Bool for #bool.
  construct literal(kind, text, pos, value) {
    _kind = kind
    _text = text
    _pos = pos
    _value = value
  }

  kind  => _kind    // a Symbol — see the kind table below
  text  => _text    // the raw source slice, unescaped for #text
  pos   => _pos      // BYTE offset of the token's first byte (see §6)
  value => _value    // None for non-literal kinds

  is(k) => _kind == k

  toString => "" + _kind.toString + "(" + _text + ")@" + _pos.toString
}
```

### Kind table

| Symbol | Lexeme(s) | Carries `value` |
|---|---|---|
| `#number` | `12`, `3.5` | `Number` |
| `#text` | `'hello'` | `String` (unescaped) |
| `#ref` | `A1`, `$A$1` | — (raw text only; parser decodes) |
| `#bool` | `TRUE`, `false` (case-insensitive) | `Bool` |
| `#ident` | `SUM`, `COUNTIF` | — |
| `#plus #minus #star #slash #percent #caret` | `+ - * / % ^` | — |
| `#comma #colon #lparen #rparen` | `, : ( )` | — |
| `#lt #le #gt #ge #eq #ne` | `< <= > >= = <>` | — |
| `#eof` | (empty) | — |

`Symbol` kind tags follow the convention `02-value-model.md` already uses for
`ErrorVal`'s `_kind` field — hashable, comparable with `==`, cheap.

There is no separate "range" token kind. `A1:B7` lexes as three tokens
(`#ref`, `#colon`, `#ref`); the range is a parser-level construct
(`Ast.RangeLit`), because whether `:` is even legal here (both sides must be
refs) is a grammar question, not a lexical one.

## 3. Error type

```phalcom
// lex/lexer.ph (co-located; small enough not to need its own file)
class LexError {
  construct at(pos, message) {
    _pos = pos
    _message = message
  }

  pos     => _pos
  message => _message

  toString => "LexError at " + _pos.toString + ": " + _message
}
```

Exactly two conditions produce a `LexError`:

1. An unterminated `'...'` text literal (ran off the end of the source).
2. A byte that starts none of: whitespace, digit, `'`, letter, `$`, or a
   recognized operator/punctuation byte.

A malformed-number `LexError` (§5) is included for defensive completeness but
is unreachable given the scan rule in §5 — the scanner never hands
`Number.new` a string it didn't just build byte-by-byte from `[0-9.]`.

## 4. Character-class helpers (`support/str.ph` additions)

`Number` has no character type and `String` has no `isDigit`/`isAlpha`/`at(_)`
(findings §3, §5). Every character test the lexer needs is a numeric range
check on a byte value, hand-rolled once here and pushed down into
`support/str.ph` per **REQ-ARCH-2** (01-architecture.md §3): this is exactly
the kind of spreadsheet-agnostic backfill that module exists for, and
01-architecture.md's file layout should be read as amended to include it.

```phalcom
// support/str.ph — additions alongside padLeft/padRight/repeat/startsWith
class Str {
  static isDigit(s, i) {
    let b = s.rawByteAt(i)
    return (b != None) and (b >= 48) and (b <= 57)          // '0'..'9'
  }

  static isAlpha(s, i) {
    let b = s.rawByteAt(i)
    return (b != None)
      and (((b >= 65) and (b <= 90)) or ((b >= 97) and (b <= 122)))  // A-Z a-z
  }

  static isAlphaNumeric(s, i) => Str.isAlpha(s, i) or Str.isDigit(s, i)

  // '$' (36) is a word character for lexing purposes only (A1-notation
  // qualifiers), not a general identifier character.
  static isWordChar(s, i) => Str.isAlphaNumeric(s, i) or (s.rawByteAt(i) == 36)

  // Uppercases a single ASCII byte value; non-letters pass through.
  // Hand-rolled because String has no toUpper/toUpperCase (findings §5).
  static upperByte(b) => ((b >= 97) and (b <= 122)).ifTrue({ b - 32 }, ifFalse: { b })

  // Case-insensitive ASCII compare. `literal` is always a Phalcom string
  // constant like "TRUE" — never user input — so no normalization on that
  // side is needed.
  static equalsIgnoreCase(word, literal) {
    (word.rawByteCount != literal.rawByteCount).ifTrue { return false }
    var i = 0
    var same = true
    while ((i < word.rawByteCount) and same) {
      same = (Str.upperByte(word.rawByteAt(i)) == Str.upperByte(literal.rawByteAt(i)))
      i = i + 1
    }
    return same
  }
}
```

> **Commentary — every one of these is a language gap wearing a helper's
> clothes.** `isDigit`, `isAlpha`, and `upperByte` are not spreadsheet logic.
> They are the character-class primitives a lexer needs on day one, and they
> do not exist anywhere in `core.ph` under any name. I did not have to guess
> the range constants — I derived them from `core.ph`'s own `String` class
> (`phalcom-core/core/core.ph`, `codePointAt`/`leadByteLen_`), which hand-rolls
> the *identical* kind of numeric-range logic internally to decode UTF-8, byte
> by byte, with no bitwise operators available either. If the runtime's own
> standard library has to do this, a user program with a lexer in it
> obviously will too. Filed as **GAP-LEX-1**.

## 5. Lexing rules

### 5.1 Whitespace

Space, tab, CR, LF (bytes 32, 9, 13, 10) are skipped and produce no token.
Formulas are single-line strings in v1's scope, but the skip treats all four
uniformly rather than assuming no `\n` ever appears — cheap defensiveness, not
a feature.

```phalcom
skipWhitespace_() {
  while ((_pos < _len) and self.isSpaceByte_(_src.rawByteAt(_pos))) {
    _pos = _pos + 1
  }
}

isSpaceByte_(b) => (b == 32) or (b == 9) or (b == 10) or (b == 13)
```

### 5.2 Numbers

Grammar: `[0-9]+ ('.' [0-9]+)?`. No sign (unary `-` is the parser's job —
`-` is also the subtraction operator, so the lexer must not swallow it), and
no scientific notation in v1 (a deliberate simplification — see the
commentary below).

```phalcom
scanNumber_(tokens, start) {
  while ((_pos < _len) and Str.isDigit(_src, _pos)) { _pos = _pos + 1 }

  ((_pos < _len) and (_src.rawByteAt(_pos) == 46)).ifTrue {   // '.'
    let save = _pos
    _pos = _pos + 1
    (Str.isDigit(_src, _pos)).ifTrue({
      while ((_pos < _len) and Str.isDigit(_src, _pos)) { _pos = _pos + 1 }
    }, ifFalse: {
      _pos = save    // lone '.' with nothing after it is not part of the number
    })
  }

  let text = _src.rawSlice(start, _pos)
  let parsed = { Number.new(text) }.attempt()
  (parsed.isErr).ifTrue {
    return Err.new(LexError.at(start, "malformed number literal '" + text + "'"))
  }

  tokens.add(Token.literal(#number, text, start, parsed.unwrap))
  return Ok.new(None)
}
```

`Number.new(_)` — the **class-side** constructor, not an instance method — is
the conversion primitive: it parses a numeric `String` to `f64`, raising
(caught here via `.attempt()`) on garbage. This is not documented anywhere in
00-language-findings.md §3, which only enumerates `Number`'s *instance*
surface (`+ - * / % < <= > >= == negated hash toString` — genuinely zero
conversion methods there). §7 below records this as a fact the findings
document should carry.

### 5.3 Text literals — single quotes, and why

Grammar: `'` then any bytes until an unescaped `'`; `''` inside the literal
is one escaped `'`.

```phalcom
scanText_(tokens, start) {
  var i = _pos + 1                 // skip opening quote
  var closed = false
  var contentEnd = -1

  while ((i < _len) and (not closed)) {
    (_src.rawByteAt(i) == 39).ifTrue({                          // '\''
      ((i + 1 < _len) and (_src.rawByteAt(i + 1) == 39)).ifTrue({
        i = i + 2                  // '' -> one literal quote, keep scanning
      }, ifFalse: {
        contentEnd = i
        closed = true
      })
    }, ifFalse: {
      i = i + 1
    })
  }

  (not closed).ifTrue {
    return Err.new(LexError.at(start, "unterminated text literal"))
  }

  let raw = _src.rawSlice(_pos + 1, contentEnd)
  let value = raw.replace("''", "'")
  _pos = contentEnd + 1
  tokens.add(Token.literal(#text, _src.rawSlice(start, _pos), start, value))
  return Ok.new(None)
}
```

This uses only `rawByteAt`, `rawSlice`, and `replace` — every one
`VERIFIED-PRESENT`. It does **not** need `codePointAt` for delimiter
detection: `'` (byte 39) is ASCII, and no continuation byte or lead byte of a
multi-byte UTF-8 sequence can ever equal 39 (every non-ASCII UTF-8 byte has
its high bit set, i.e. is `>= 128`). So a raw byte scan for the delimiter is
correct even when the literal's *content* contains multi-byte characters —
the lexer never has to decode them, only copy them through via `rawSlice`.

Why single quotes at all, restated precisely: `"` is unreachable in Phalcom
source (**GAP-STR-1**, findings §5) — there is no `\"` escape and no
char-from-codepoint constructor, so a Phalcom program cannot even construct
the byte for `"`. Since this spec's own test fixtures are Phalcom string
literals *containing* formula text (`"=CONCAT('a','b')"`), a formula grammar
that used `"` as its text delimiter would be unwritable in this document set.
`'` has no special meaning to Phalcom's own lexer, so it costs nothing to
delimit with, and the `''`-doubling escape convention is the same one Excel
itself uses for `""` inside double-quoted formula text — the delimiter
changed, the escape mechanism didn't.

### 5.4 Cell references, booleans, and function names — one scan, three outcomes

All three start the same way: a letter or `$`. The lexer scans one maximal
run of "word" bytes (letter, digit, or `$`) and *then* classifies the whole
run — it does not decide up front what kind of thing it's looking at.

```phalcom
scanWord_(tokens, start) {
  while ((_pos < _len) and Str.isWordChar(_src, _pos)) { _pos = _pos + 1 }
  let word = _src.rawSlice(start, _pos)

  // Rule 1 (REQ-LEX-5): immediately followed by '(' => always a function
  // name, even if the word happens to look like a ref. This is the only
  // thing that lets LOG10, SIN, etc. coexist with A1-style refs at all —
  // see the commentary below.
  ((_pos < _len) and (_src.rawByteAt(_pos) == 40)).ifTrue {
    tokens.add(Token.new(#ident, word, start))
    return Ok.new(None)
  }

  // Rule 2 (REQ-LEX-6): TRUE/FALSE, case-insensitive, checked before ref-shape.
  (Str.equalsIgnoreCase(word, "TRUE")).ifTrue {
    tokens.add(Token.literal(#bool, word, start, true))
    return Ok.new(None)
  }
  (Str.equalsIgnoreCase(word, "FALSE")).ifTrue {
    tokens.add(Token.literal(#bool, word, start, false))
    return Ok.new(None)
  }

  // Rule 3 (REQ-LEX-7): ref-shaped => #ref, raw text only.
  (self.looksLikeRef_(word)).ifTrue {
    tokens.add(Token.new(#ref, word, start))
    return Ok.new(None)
  }

  // Fallback: an identifier that is not a call and not ref-shaped. Not an
  // error at lex time — v1's grammar has no bare defined names, so the
  // parser will reject this, most likely as an unexpected token.
  tokens.add(Token.new(#ident, word, start))
  return Ok.new(None)
}

// $?[A-Za-z]+$?[0-9]+ , with nothing left over.
looksLikeRef_(word) {
  let n = word.rawByteCount
  var i = 0
  ((i < n) and (word.rawByteAt(i) == 36)).ifTrue { i = i + 1 }     // '$'
  let letterStart = i
  while ((i < n) and Str.isAlpha(word, i)) { i = i + 1 }
  (i == letterStart).ifTrue { return false }                       // no letters
  ((i < n) and (word.rawByteAt(i) == 36)).ifTrue { i = i + 1 }     // '$'
  let digitStart = i
  while ((i < n) and Str.isDigit(word, i)) { i = i + 1 }
  return (i == n) and (i > digitStart)
}
```

> **Commentary — `LOG10` is not a hypothetical edge case.** A word made of
> letters followed by digits, with nothing else, is *exactly* the A1-notation
> shape (`$?[A-Za-z]+$?[0-9]+`) and *exactly* the shape of several real
> spreadsheet function names — `LOG10` being the obvious one, but the
> function library in [08-functions.md](08-functions.md) will likely add more
> as it grows. Bounding the letter run to 1–3 characters (Excel's real column
> limit, `A`..`XFD`) does **not** disambiguate `LOG10` from a ref shaped
> `LOG10` — three letters, two digits, no bound rules it out. The only sound
> disambiguator is context: a cell reference is never directly followed by
> `(` with no operator between them, so "next byte is `(`" is checked
> *before* the ref-shape test and wins unconditionally. This is not a
> Phalcom-specific finding — Excel's own tokenizer resolves the identical
> ambiguity the identical way — but it is real design work this spec would
> otherwise have glossed over, and it belongs on record because the ref-shape
> test alone, without the lookahead, is a latent bug waiting for someone to
> write `=LOG10(100)`.

### 5.5 Operators

Single-byte, no lookahead needed: `+ - * / % ^ , : ( )`.

```phalcom
static singleCharKinds_() {
  var m = Map.new()
  m.at(43, put: #plus)     // +
  m.at(45, put: #minus)    // -
  m.at(42, put: #star)     // *
  m.at(47, put: #slash)    // /
  m.at(37, put: #percent)  // %
  m.at(94, put: #caret)    // ^
  m.at(44, put: #comma)    // ,
  m.at(58, put: #colon)    // :
  m.at(40, put: #lparen)   // (
  m.at(41, put: #rparen)   // )
  return m
}
```

`Map#at(_)` is total (`None` on miss, per `core.ph`'s `Map` — "Some-shaped by
convention"), so `Lexer.singleCharKinds_().at(b) != None` is a clean
membership test with no exception path.

One-byte lookahead operators — `<=`, `>=`, `<>` versus their single-byte
fallbacks:

```phalcom
scanLt_(tokens, start) {
  _pos = _pos + 1
  ((_pos < _len) and (_src.rawByteAt(_pos) == 61)).ifTrue {        // <=
    _pos = _pos + 1
    tokens.add(Token.new(#le, "<=", start))
    return Ok.new(None)
  }
  ((_pos < _len) and (_src.rawByteAt(_pos) == 62)).ifTrue {        // <>
    _pos = _pos + 1
    tokens.add(Token.new(#ne, "<>", start))
    return Ok.new(None)
  }
  tokens.add(Token.new(#lt, "<", start))
  return Ok.new(None)
}

scanGt_(tokens, start) {
  _pos = _pos + 1
  ((_pos < _len) and (_src.rawByteAt(_pos) == 61)).ifTrue {        // >=
    _pos = _pos + 1
    tokens.add(Token.new(#ge, ">=", start))
    return Ok.new(None)
  }
  tokens.add(Token.new(#gt, ">", start))
  return Ok.new(None)
}
```

`=` is always a single byte at the lexer level (`#eq`). It is used twice by
the *grammar* — once as the mandatory formula prefix, once as the infix
equality operator — but the lexer does not know that distinction and does not
need to; see [05-formula-parser.md §1](05-formula-parser.md).

## 6. Byte offset vs codepoint index

**Every position in this document — `Token#pos`, `LexError#pos`, `_pos` — is
a byte offset into the source `String`, not a codepoint index.** This is not
a simplification; it is forced by the shape of `String`'s own API.
`codePointAt(i)` (verified by reading `core.ph`'s own implementation, since
findings §5 records only that it exists, not its indexing convention) takes
`i` as a **byte offset**: it calls `rawByteAt(i)` for the lead byte and, if
that byte starts a multi-byte sequence, reads `i+1`, `i+2`, `i+3` — also byte
offsets. There is no method that takes a "the 5th character" index and no
method that converts one indexing scheme to the other. `String` has no
`at(_)` at all (findings §5).

Concretely, this lexer never advances `_pos` by "one character" — it advances
by however many bytes the rule just consumed, which for every token kind in
this grammar (numbers, refs, idents, operators, and the *delimiters* of text
literals) is always 1, because the formula grammar's structural characters
are all ASCII. The one place a formula could carry multi-byte content — text
inside `'...'` — is handled by treating it as opaque bytes (§5.3) and never
decoding it at all, sidestepping the byte/codepoint distinction rather than
resolving it. A future feature that needed to *count characters* inside a
text literal (e.g. an Excel `LEN` that counts codepoints, not bytes) would
have to walk it with `codePointAt` one lead byte at a time, using the same
numeric-range trick `core.ph`'s own `leadByteLen_` uses internally to find
each next lead byte — and that helper is not public API (its trailing `_`
is this codebase's convention for "internal", not a language-enforced
privacy mechanism, but there is no substitute exposed either way).

> **Commentary — scanning by byte offset is the only option, and it is
> mostly fine, and the one place it isn't is invisible until it is.** For a
> formula grammar whose structural grammar is 100% ASCII, byte-offset
> scanning with `rawByteAt` is not a workaround, it is simply correct and
> arguably simpler than codepoint-index scanning would be (no decoding on
> the hot path). The friction shows up only at the boundary: the moment any
> future requirement needs to reason about *characters* inside a text
> literal — length for display truncation, a codepoint-based `MID`/`LEFT`
> function mirroring Excel's actual semantics — the "no `at(_)`, no
> char-count helper, indices are bytes" trio turns a two-line function into a
> hand-rolled UTF-8 walker. None of that is needed by this document's scope
> (v1 has no string functions), but it is exactly the kind of debt that is
> invisible in a lexer and expensive in a function library. Filed as
> **GAP-LEX-2**, and flagged forward to [08-functions.md](08-functions.md).

## 7. A capability findings.md does not record

00-language-findings.md §3 states `Number`'s full method surface is `+ - * /
% < <= > >= == negated hash toString` and calls that "the complete list" —
true for *instance* methods. It says nothing about `Number`'s **class-side**
`new(_)`, which this lexer depends on directly (§5.2): `Number.new(_)`
accepts a `Number` (identity), a `Bool` (`1`/`0`), or a numeric `String`
(parsed via `f64::parse`, so it also accepts leading `+`/`-`, `.5`, `5.`, and
`5e3` scientific notation — all more permissive than this grammar's own
number rule, which is fine, since the lexer only ever feeds it strings it
already validated as `[0-9]+('.'[0-9]+)?`). Verified directly:

```phalcom
System.print({ Number.new("3.14") }.attempt())   // => Ok(3.14)
System.print({ Number.new("abc") }.attempt())    // => Err(<Error>)
System.print({ Number.new("5e3") }.attempt())    // => Ok(5000)
```

This is a real string→number conversion primitive, and it is the reason this
lexer does not need to hand-roll digit accumulation the way it hand-rolls
`isDigit`/`isAlpha` (§4). It should be added to 00-language-findings.md §3 as
a class-side addendum; it is recorded here because this document depends on
it and cannot assume the reader has re-derived it.

## 8. Requirements

| REQ | Statement |
|---|---|
| **REQ-LEX-1** | `Lexer.tokenize(source)` returns `Result<List<Token>, LexError>` and never raises for malformed *formula* input — only a scanner bug reaching the defensive `Number.new` guard (§5.2) would raise, and that path returns `Err`, not a raise, so even that is caught. |
| **REQ-LEX-2** | Every `Token#pos` and `LexError#pos` is a byte offset into `source` (§6). |
| **REQ-LEX-3** | Numeric literals match `[0-9]+('.'[0-9]+)?`. No sign, no exponent — those are grammar-level, not lexical. |
| **REQ-LEX-4** | Text literals are `'`-delimited; `''` inside one is an escaped `'`; an unterminated literal is a `LexError` at the opening `'`'s position. |
| **REQ-LEX-5** | A word immediately followed by `(` (no whitespace) lexes as `#ident`, regardless of ref-shape. |
| **REQ-LEX-6** | `TRUE`/`FALSE`, case-insensitive, lex as `#bool`, checked before ref-shape. |
| **REQ-LEX-7** | A word matching `$?[A-Za-z]+$?[0-9]+` (not already claimed by REQ-LEX-5/6) lexes as `#ref` with the raw lexeme as `text`. Anything else lexes as `#ident`. |
| **REQ-LEX-8** | `<=`, `>=`, `<>` are recognized by exactly one byte of lookahead; `<`, `>`, `=` are the single-byte fallbacks. |
| **REQ-LEX-9** | Space, tab, CR, LF are skipped between tokens and produce none. |
| **REQ-LEX-10** | The final token of every successful tokenization is `#eof` at `source.rawByteCount`. |

## 9. Test hooks

| REQ | Test |
|---|---|
| REQ-LEX-1, REQ-LEX-10 | `suites/lex_smoke.ph` — round-trips `01-architecture.md`'s `"=SUM(A1:A3)*2"` example against the exact token list in §4 of that document. |
| REQ-LEX-3 | `suites/lex_numbers.ph` — `12`, `3.5`, `0.5`, `5.` (accepted, per REQ-LEX-3's grammar — trailing dot with digits before it), and `.5` (rejected — no leading digit, so the scan never enters `scanNumber_` for it; it lexes as `#colon`-adjacent garbage, i.e. an unexpected-character `LexError` at the `.`). |
| REQ-LEX-4 | `suites/lex_text.ph` — `'hello'`, `'it''s'` → `it's`, and unterminated `'oops` → `LexError`. |
| REQ-LEX-5 | `suites/lex_word_disambiguation.ph` — `LOG10(2)` tokenizes `LOG10` as `#ident`; `LOG10` alone (no paren) tokenizes as `#ref`. This is the one test in the suite that exists purely because of §5.4's ambiguity. |
| REQ-LEX-6 | `suites/lex_bool.ph` — `TRUE`, `true`, `False`, and `TRUEFOO` (must **not** match — `equalsIgnoreCase` requires exact length). |
| REQ-LEX-7 | `suites/lex_refs.ph` — `A1`, `$A1`, `A$1`, `$A$1`, and a non-ref word like `FOO` (no trailing digits) lexing as `#ident`. |
| REQ-LEX-8 | `suites/lex_operators.ph` — every operator in the kind table, plus adjacency cases (`<=`, `< =` with a space — must lex as two tokens `#lt #eq`, not one `#le`, since REQ-LEX-8's lookahead is exactly one byte with no whitespace-skipping mid-operator). |
| REQ-LEX-9 | `suites/lex_whitespace.ph` — leading/trailing/interior whitespace variants of the same formula produce identical token streams modulo `pos`. |
| GAP-LEX-1 | `suites/lex_charclass.ph` — direct unit tests on `Str.isDigit`/`isAlpha`/`equalsIgnoreCase` against the ASCII boundary bytes (`/`, `:`, `@`, `` [ `` — the bytes immediately outside each range). |
