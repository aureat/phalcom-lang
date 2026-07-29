# SheetCalc — Formula Parser

Part of the [SheetCalc specification](README.md).
Grounded in [00-language-findings.md](00-language-findings.md) §4, §8, and
[04-formula-lexer.md](04-formula-lexer.md).

## 1. Scope

`Parser.parse(tokens)` — `List<Token> -> Result<Ast, ParseError>`, a Pratt
(precedence-climbing) recursive-descent parser over the token stream
[04-formula-lexer.md](04-formula-lexer.md) produces.

`parse/` may import `support/`, `lex/`, and `grid/` — the last one
deliberately, so an `#ref` token's raw text becomes a real `Ref` object at
parse time rather than staying a string until `eval/` (01-architecture.md
§3). `Ast` node classes are specified here only to the depth the parser needs
to construct them; their `eval` behavior is [06-ast-and-eval.md](06-ast-and-eval.md)'s.

The grammar's one structural quirk, carried from the lexer: a formula's
source text always includes its leading `=` (`"=SUM(A1:A3)*2"`, not
`"SUM(A1:A3)*2"`), and `=` is *also* the infix equality operator (`IF(A1=1,
...)`). The lexer does not disambiguate these — every `=` lexes as `#eq`
(04-formula-lexer.md §5.5). The parser does: `run_()` consumes exactly one
leading `#eq` unconditionally before entering the expression grammar, and any
`#eq` encountered after that point is ordinary infix equality.

## 2. Precedence table

Binding power increases down the table (higher binds tighter). All binary
operators are left-associative except `^`.

| Tier | Operators | Assoc | Binding power |
|---|---|---|---|
| 1 (loosest) | `= <> < <= > >=` (comparison) | left | 10 |
| 2 | `+ -` (binary) | left | 20 |
| 3 | `* / %` | left | 30 |
| 4 | `^` (exponent) | **right** | 40 |
| 5 | unary `-` (prefix) | — | binds as if bp 45 |
| 6 | `:` (range) | left, non-chaining | 50 |
| 7 (tightest) | function calls, `(...)` | — | primary position, not on this scale |

**REQ-PARSE-1.** `%` is binary modulo at tier 3, matching Phalcom's own
`Number#%` and `02-value-model.md`'s `CellValue#%(o)` — **not** Excel's
postfix percent operator. This is a deliberate divergence: SheetCalc's value
model already commits to `%` as a binary arithmetic operator alongside
`+ - * /` (02-value-model.md §2), and Phalcom's own `%` is modulo (findings
§3), so making the formula grammar's `%` mean anything else would require
inventing a *different* token for modulo with no Phalcom precedent to lean
on. Recorded as **DEC-PARSE-1**.

> **Commentary — `-2^2` is `4` here, not `-4`, and that is a deliberate,
> flagged divergence from Excel.** Excel (and Python, and most spreadsheets)
> define unary minus as *looser* than `^`, so `-2^2` parses as `-(2^2) = -4`
> — a famous gotcha that trips up even experienced spreadsheet users. This
> grammar puts unary `-` at bp 45, tighter than `^`'s bp 40 (§3), so `-2^2`
> parses as `(-2)^2 = 4` — the conventional-language reading, not Excel's.
> This was a specific choice for this document, not an oversight: a
> precedence table is exactly the kind of thing that is easy to get
> "accidentally Excel-compatible" on by not thinking about it, and easy to
> get silently wrong the other way by copying a general-purpose language's
> table without checking it against spreadsheet convention. **DEC-PARSE-1**
> (recorded fully in §3) exists so a future implementer does not have to
> rediscover which behavior they got. `suites/parse_precedence.ph` (§7) pins
> `-2^2 == 4` explicitly, with a comment pointing here, precisely so nobody
> "fixes" it into an Excel-compatible regression without reading this
> paragraph first.

## 3. Tokens, `Ast`, and errors

```phalcom
// parse/parser.ph
import "./ast" as Ast
import "../grid/ref" as Grid     // for Ref decoding once §6's Ast.RefLit reaches eval/ (06-ast-and-eval.md)

class ParseError {
  @constructor
  at(pos, message) {
    _pos = pos
    _message = message
  }

  pos     => _pos
  message => _message

  toString => "ParseError at " + _pos.toString + ": " + _message
}

// A binding power plus its associativity, keyed by operator kind (§2).
class BindingPower {
  @constructor
  new(power, rightAssoc) {
    _power = power
    _rightAssoc = rightAssoc
  }

  power      => _power
  rightAssoc => _rightAssoc
}
```

`Ast` node classes used below (full protocol, including `eval`, is
[06-ast-and-eval.md](06-ast-and-eval.md)'s job — this is the constructor
surface the parser needs):

```phalcom
// parse/ast.ph
class NumberLit {@constructor new(n) { _n = n }              n => _n }
class TextLit    {@constructor new(s) { _s = s }              s => _s }
class BoolLit    {@constructor new(b) { _b = b }              b => _b }
class RefLit     {@constructor new(text) { _text = text }     text => _text }
class RangeLit   {@constructor new(from, to) { _from = from; _to = to }
                    from => _from
                    to   => _to }
class UnaryOp    {@constructor new(op, operand) { _op = op; _operand = operand }
                    op      => _op
                    operand => _operand }
class BinOp      {@constructor new(op, left, right) { _op = op; _left = left; _right = right }
                    op    => _op
                    left  => _left
                    right => _right }
class Call       {@constructor new(name, args) { _name = name; _args = args }
                    name => _name
                    args => _args }
```

## 4. The Pratt loop

```phalcom
// parse/parser.ph
class Parser {
  @constructor
  on(tokens) {
    _toks = tokens
    _pos = 0
  }

  static parse(tokens) => Parser.on(tokens).run_()

  run_() {
    let leadEq = self.expect_(#eq, "formula must start with '='")
    (leadEq.isErr).ifTrue { return leadEq }

    let bodyR = self.parseExpr_(0)
    (bodyR.isErr).ifTrue { return bodyR }

    let endCheck = self.expect_(#eof, "unexpected trailing input")
    (endCheck.isErr).ifTrue { return endCheck }

    return Ok.new(bodyR.unwrap)
  }

  // --- token stream primitives ---

  peek_()    => _toks.at(_pos)
  advance_() { let t = _toks.at(_pos); _pos = _pos + 1; return t }
  check_(k)  => self.peek_().kind == k

  expect_(kind, msg) {
    (self.check_(kind)).ifTrue {
      return Ok.new(self.advance_())
    }
    return Err.new(ParseError.at(self.peek_().pos, msg + " (found '" + self.peek_().text + "')"))
  }

  // --- the precedence-climbing core ---

  parseExpr_(minBp) {
    let leftR = self.parsePrefix_()
    (leftR.isErr).ifTrue { return leftR }
    var left = leftR.unwrap

    while (true) {
      let bp = self.infixBp_(self.peek_().kind)
      ((bp == None) or (bp.power < minBp)).ifTrue { break }

      let opTok = self.advance_()

      (opTok.kind == #colon).ifTrue({
        let rangeR = self.finishRange_(left, opTok)
        (rangeR.isErr).ifTrue { return rangeR }
        left = rangeR.unwrap
      }, ifFalse: {
        let nextMinBp = (bp.rightAssoc).ifTrue({ bp.power }, ifFalse: { bp.power + 1 })
        let rightR = self.parseExpr_(nextMinBp)
        (rightR.isErr).ifTrue { return rightR }
        left = Ast.BinOp.new(opTok.kind, left, rightR.unwrap)
      })
    }

    return Ok.new(left)
  }

  infixBp_(kind) => Parser.infixTable_().at(kind)

  static infixTable_() {
    var m = Map.new()
    m.at(#eq,      put: BindingPower.new(10, false))
    m.at(#ne,      put: BindingPower.new(10, false))
    m.at(#lt,      put: BindingPower.new(10, false))
    m.at(#le,      put: BindingPower.new(10, false))
    m.at(#gt,      put: BindingPower.new(10, false))
    m.at(#ge,      put: BindingPower.new(10, false))
    m.at(#plus,    put: BindingPower.new(20, false))
    m.at(#minus,   put: BindingPower.new(20, false))
    m.at(#star,    put: BindingPower.new(30, false))
    m.at(#slash,   put: BindingPower.new(30, false))
    m.at(#percent, put: BindingPower.new(30, false))
    m.at(#caret,   put: BindingPower.new(40, true))
    m.at(#colon,   put: BindingPower.new(50, false))
    return m
  }

  // --- prefix position ("nud") ---

  parsePrefix_() {
    let t = self.peek_()

    (t.kind == #minus).ifTrue {
      self.advance_()
      let operandR = self.parseExpr_(45)     // 45 > ^'s 40: unary binds tighter (§2, DEC-PARSE-1)
      (operandR.isErr).ifTrue { return operandR }
      return Ok.new(Ast.UnaryOp.new(#minus, operandR.unwrap))
    }

    (t.kind == #number).ifTrue { self.advance_(); return Ok.new(Ast.NumberLit.new(t.value)) }
    (t.kind == #text).ifTrue   { self.advance_(); return Ok.new(Ast.TextLit.new(t.value)) }
    (t.kind == #bool).ifTrue   { self.advance_(); return Ok.new(Ast.BoolLit.new(t.value)) }
    (t.kind == #ref).ifTrue    { self.advance_(); return Ok.new(Ast.RefLit.new(t.text)) }
    (t.kind == #lparen).ifTrue { return self.parseParen_() }
    (t.kind == #ident).ifTrue  { return self.parseCall_() }

    return Err.new(ParseError.at(t.pos, "unexpected token '" + t.text + "'"))
  }

  parseParen_() {
    self.advance_()                                    // '('
    let innerR = self.parseExpr_(0)
    (innerR.isErr).ifTrue { return innerR }
    let closeR = self.expect_(#rparen, "expected ')'")
    (closeR.isErr).ifTrue { return closeR }
    return Ok.new(innerR.unwrap)
  }

  parseCall_() {
    let name = self.advance_()                         // #ident
    let openR = self.expect_(#lparen, "expected '(' after function name")
    (openR.isErr).ifTrue { return openR }

    var args = List.new()
    (self.check_(#rparen)).ifTrue({}, ifFalse: {
      var more = true
      while (more) {
        let argR = self.parseExpr_(0)
        (argR.isErr).ifTrue { return argR }
        args.add(argR.unwrap)
        (self.check_(#comma)).ifTrue({ self.advance_() }, ifFalse: { more = false })
      }
    })

    let closeR = self.expect_(#rparen, "expected ')' or ',' in argument list")
    (closeR.isErr).ifTrue { return closeR }
    return Ok.new(Ast.Call.new(name.text, args))
  }

  finishRange_(left, colonTok) {
    (left.isA(Ast.RefLit)).ifTrue({}, ifFalse: {
      return Err.new(ParseError.at(colonTok.pos, "':' requires a cell reference on its left"))
    })
    let rightR = self.parseExpr_(51)        // one above ':' own bp: no A1:B2:C3 chaining
    (rightR.isErr).ifTrue { return rightR }
    let right = rightR.unwrap
    (right.isA(Ast.RefLit)).ifTrue({}, ifFalse: {
      return Err.new(ParseError.at(colonTok.pos, "':' requires a cell reference on its right"))
    })
    return Ok.new(Ast.RangeLit.new(left, right))
  }
}
```

Every `.ifTrue { return X }` and `.ifTrue { break }` above is a **non-local
return / non-local break out of nested blocks**, not a local exit from the
block itself — `break` inside a block passed to `.ifTrue`/`.ifFalse` exits
the enclosing `while`, and `return` inside one returns from the enclosing
*method*, even three block-layers deep (verified directly: a `return`
statement inside `.ifTrue { }` nested inside a `while` nested inside another
`.ifTrue({...}, ifFalse:{...})` correctly returns from the containing method
call, with no `DeadFrameError`, because — unlike the fiber case in findings
§8 Trap 2 — the home frame is still on the stack the whole time). This is
exactly the machinery `parseCall_`'s argument loop depends on: `return argR`
fires from inside an `ifFalse` block, inside a `while`, inside an `ifTrue`
call, and correctly unwinds all the way back to `parseCall_`'s caller.

## 5. Result propagation has no shortcut, and that is the finding

Every one of the ~14 recursive/token-consuming calls in `run_()` through
`finishRange_()` above follows the same shape:

```phalcom
let r = self.someParse_(...)
(r.isErr).ifTrue { return r }
// only now: r.unwrap, or continue using the value some other way
```

Two lines of pure control-flow scaffolding, repeated at every single call
site that can fail — which, in a recursive-descent parser, is *every* call
site, because every production can fail. There is no `?` operator
(Rust/Swift), no exception unwind (most languages), and no implicit
short-circuit (`errors.Is`-style Go idiom aside, Go at least has `if err !=
nil { return err }` as the *entire* idiom — Phalcom's version needs the
`.isErr` message send plus a block literal plus the non-local return, which
is more syntax for the same two lines, not less).

Counting this parser's own body: 13 of its roughly 20 statements that consume
a token or recurse are immediately followed by an `isErr`-check-and-return
pair. That is not a stylistic choice — removing any one of them means a
`ParseError` three frames down silently becomes a nonsense `Ast` node three
frames up. Rough estimate: a third to a half of this file's line count is
Result-plumbing that would not exist at all with `?` or exceptions.

**Mitigations considered, both worse than a `?` operator:**

1. **The pattern above** (`isErr` check + early return), used throughout this
   spec. It is at least uniform and greppable. Cost: 2 lines per fallible
   call, no exceptions.
2. **`Result#andThen` chaining** (verified present, `core.ph`:
   `andThen(f) { return self.match(ok: { v => f.call(v) }, err: { e => self }) }`).
   Rewriting `run_()` this way:

   ```phalcom
   run_() {
     return self.expect_(#eq, "formula must start with '='").andThen({ lead =>
       self.parseExpr_(0).andThen({ body =>
         self.expect_(#eof, "unexpected trailing input").andThen({ eof =>
           Ok.new(body)
         })
       })
     })
   }
   ```

   removes the `isErr`/return pairs entirely — genuinely less boilerplate for
   *this* three-step method. It does not scale to `parseExpr_`'s loop or
   `parseCall_`'s variable-length argument list: `andThen` presumes a fixed,
   known chain of steps, each nested one block deeper than the last (a
   "pyramid of doom" for `n` steps), and a `while` loop with early-exit
   conditions has no natural `andThen` shape at all — you would still be
   writing `isErr` checks inside the loop body regardless. It is a good fit
   for `run_()` alone and a poor fit for the file's other three-quarters.
3. **A non-local-return helper**, exploiting the same block semantics as
   §4's commentary: a method that takes a result and an "on error" block,
   and lets the block's own `return` unwind the *caller's* frame:

   ```phalcom
   unwrapOrReturn_(r, onErr) {
     (r.isErr).ifTrue { onErr.call() }
     return r.unwrap
   }
   ```

   called as `let lhs = self.unwrapOrReturn_(self.parsePrefix_(), { return leftR })` —
   works because the block literal `{ return leftR }` is lexically written
   inside the calling method, so its `return` targets that method's frame,
   not `unwrapOrReturn_`'s. It is a real Smalltalk trick and it does work.
   It is also **not adopted in this spec**: it requires the caller to
   duplicate the failing result in the block (`{ return leftR }` has to name
   `leftR`, which only exists because you just called the thing you're now
   wrapping), it is one more indirection to read through at every call site,
   and it is exactly the same shape of "block whose `return` outlives its
   apparent scope" that produces `DeadFrameError` when misused across a
   suspended frame (findings §8 Trap 2) — safe here only because parsing is
   fully synchronous, but a subtle enough precondition that it does not
   belong in a spec whose job is to be copied without re-deriving why it's
   safe.

None of the three is a `?` operator. All three were tried in drafting this
document before settling on option 1 uniformly. **Filed as GAP-PARSE-1** —
the single highest-value language addition for anyone writing a
recursive-descent anything in Phalcom, and this parser is the concrete
evidence for why.

> **Commentary — this is the parser's version of findings §4's headline
> finding.** `1 + errorValue` (findings §4) forced the *value model*; the
> absence of `?`/exceptions forces the *control-flow shape* of anything that
> can fail structurally. Both come from the same root cause: Phalcom commits
> fully to "errors are either raised `Error`s (via `throw`, caught only by
> `.attempt()`/`.on()`) or ordinary values (`Result`, `Option`)" with no
> lightweight glue between the two. That is a coherent design — findings §7
> of `error-handling.md` calls it out as deliberate — but a Pratt parser is
> exactly the program shape that feels the gap hardest, because it is
> *nothing but* "call something that might fail, and if it did, stop."

## 6. Recursion depth — a refuted worry, not a live risk

Findings §8 verified 50,000 stack frames with **no ceiling hit**, using an
ordinary recursive function, not a fiber. This parser's recursion depth is
bounded by the formula's own nesting depth (parenthesization, argument
nesting, chained ranges refused at §4's bp 51) — a hand-typed formula
nesting hundreds of levels deep is already pathological input long before it
threatens a 50,000-frame ceiling. Pre-probe, deep recursion in a
recursive-descent parser was the predicted risk in this document set's
design (00-language-findings.md §11 lists it explicitly as a prediction);
post-probe, it is a non-issue, and no depth-limiting counter or trampoline is
specified here for that reason.

## 7. Requirements

| REQ | Statement |
|---|---|
| **REQ-PARSE-1** | `%` is binary modulo, tier 3 (§2, DEC-PARSE-1). |
| **REQ-PARSE-2** | `Parser.parse(tokens)` requires exactly one leading `#eq` token (the formula's `=`) before any expression content; its absence is a `ParseError` at token 0. |
| **REQ-PARSE-3** | `^` is right-associative: `2^3^2` parses as `2^(3^2)`. |
| **REQ-PARSE-4** | Unary `-` binds tighter than `^`: `-2^2` parses as `(-2)^2` (§2 commentary, DEC-PARSE-1 — a deliberate divergence from Excel's `-(2^2)`). |
| **REQ-PARSE-5** | `:` requires an `Ast.RefLit` on both sides; anything else is a `ParseError` at the `:` token, not a runtime `#REF!`. `A1:B2:C3` is a `ParseError` (no chaining — `finishRange_` recurses at bp 51, one above `:`'s own bp 50). |
| **REQ-PARSE-6** | A parenthesized expression `(...)` parses at bp 0 internally and requires a matching `)`; an unmatched `(` is a `ParseError` at the position where `)` was expected. |
| **REQ-PARSE-7** | A function call is `#ident` `(` \[expr (`,` expr)*\] `)`; zero args (`FOO()`) and trailing-comma-free lists are the only supported shapes — `FOO(,)` and `FOO(1,)` are `ParseError`s. |
| **REQ-PARSE-8** | Any token remaining after a complete top-level expression is a `ParseError` ("unexpected trailing input") — `Parser.parse` never silently ignores a suffix. |
| **REQ-PARSE-9** | Every parse failure is a `ParseError` carrying the byte position of the offending token (inherited from `Token#pos`, 04-formula-lexer.md §6) and a human-readable message. |

## 8. Test hooks

| REQ | Test |
|---|---|
| REQ-PARSE-1 | `suites/parse_precedence.ph` — `=1+2*3` → `2*3` binds first; `=10%3` parses as `BinOp(#percent, 10, 3)` at tier 3, alongside `*`/`/`. |
| REQ-PARSE-2 | `suites/parse_malformed.ph` — `SUM(A1)` (no leading `=`) → `ParseError` at position 0, "formula must start with '='". |
| REQ-PARSE-3 | `suites/parse_precedence.ph` — `=2^3^2` → `BinOp(#caret, 2, BinOp(#caret, 3, 2))`, not the left-assoc tree. |
| REQ-PARSE-4 | `suites/parse_precedence.ph` — `=-2^2` → `BinOp(#caret, UnaryOp(#minus, 2), 2)` (evaluates to `4`, not Excel's `-4` — see §2 commentary). |
| REQ-PARSE-5 | `suites/parse_malformed.ph` — `='a':'b'` → `ParseError` ("requires a cell reference"); `=A1:B2:C3` → `ParseError` (no chaining). |
| REQ-PARSE-6 | `suites/parse_malformed.ph` — `=(1+2` (unclosed paren) → `ParseError` "expected ')'"; `=1+2)` → `ParseError` "unexpected trailing input" (the extra `)` is never consumed by anything). |
| REQ-PARSE-7 | `suites/parse_calls.ph` — `SUM()`, `SUM(A1)`, `SUM(A1,A2,A3)`; malformed: `SUM(A1,)` → `ParseError`; `SUM(,A1)` → `ParseError` (first arg position hits `#comma`, which is not a valid prefix token). |
| REQ-PARSE-8 | `suites/parse_malformed.ph` — `=1 1` (two numbers, no operator) → `ParseError` "unexpected trailing input" at the second `1`. |
| REQ-PARSE-9 | `suites/parse_malformed.ph` — every case above additionally asserts the reported `pos` matches the offending token's byte offset, not just that *some* error occurred. |
| GAP-PARSE-1 | No dedicated test — this is a code-shape finding, not a behavior. Evidenced by the parser's own source (§5) and cited in [13-language-gaps.md](13-language-gaps.md). |

> **Commentary — malformed-input tests are where this parser earns its
> keep.** A parser that only has to accept well-formed formulas is barely
> more than a recognizer; the `ParseError` position and message on every row
> of REQ-PARSE-6/7/8's test table is the actual point of building a real
> `Result`-returning parser instead of a "does it crash" one. Every one of
> those messages was written by hand in §4 rather than generated, because
> there is no parser-generator, no error-recovery framework, and no
> "expected one of {...}" combinator library to derive them from — another
> quiet cost of building this in a young language with no parsing ecosystem
> yet, distinct from GAP-PARSE-1 but adjacent to it.
