# PDR-0006 — REPL completeness is a parser signal; the lexer reports unterminated modes

- Status: Accepted
- Date: 2026-07-20
- Related: [U-REPL §D7 / impl/04-continuation.md](../forge/units/U-REPL/impl/04-continuation.md)
  (the classification rule this record preserves and extends),
  [ADR-0016](../adr/accepted/0016-hand-written-lexer-and-recursive-descent-parser.md) (hand-written lexer +
  recursive-descent parser, panic-mode recovery),
  `2fe6aba` (the prior fix of the same shape — routing end-of-file to `UnrecognizedEof`)

## Context

A REPL must sort each submitted buffer into three outcomes: **Complete** (evaluate it),
**Incomplete** (prompt `...` and keep reading), **Invalid** (submit it so the user sees the
error rather than being trapped in a continuation they cannot escape).

U-REPL's rule, at `impl/04-continuation.md:19-25`, is deliberately minimal:

```
if parsed.errors.is_empty()                                  { Complete }
else if any error is SyntaxErrorKind::UnrecognizedEof { .. } { Incomplete }
else                                                          { Invalid }
```

followed by: *"That is the whole rule. Do not add delimiter counting, and do not special-case
strings."* `phalcom-repl/src/validator.rs:25-44` implements it verbatim.

This is the right architecture. It is Lisp's balanced-delimiter rule delegated to the parser:
the completeness oracle is *the grammar itself*, not a second model of the grammar that can
drift. The alternatives in wide use are worse. **CPython's `codeop`** classified continuation
by string-matching the error message `"unexpected EOF while parsing"`, so every new
EOF-adjacent error silently landed in the wrong bucket. **Node's REPL** does the same against
`'Unexpected end of input'`. **Ruby's `irb`** avoids string-matching by running a second lexer
state machine (`RubyLex`) that counts nesting and tracks heredocs — correct, but it is a
duplicate model of the grammar that must be maintained in lockstep with the real parser.

The rule has one hole, and it is not where it appears to be.

`let s = "abc` classifies **Incomplete correctly**, but *not* because the lexer flagged
`UnterminatedString`. It is Incomplete because the parser is left mid-expression — it wants an
expression after `=`, hits end of input, and emits `UnrecognizedEof`. The lexical error is
incidental. `impl/04-continuation.md:34-37` says exactly this.

`/* foo` behaves differently for a structural reason. An empty top-level statement list **is a
grammatically complete parse**. The parser is not mid-construct, wants nothing, and emits no
`UnrecognizedEof`. The only error produced is `LexicalError::UnterminatedBlockComment`
(`phalcom-ast/src/token.rs:294`), lowered to `SyntaxErrorKind::UnterminatedComment`
(`phalcom-ast/src/error.rs:158`) at the lowering site `phalcom-ast/src/parser.rs:146-147`. So
`classify` falls through to `Invalid` and the REPL submits a buffer the user was still writing.

So the asymmetry is not "strings are special-cased and comments were forgotten." It is that a
*parser*-level oracle is structurally blind to a state that lives entirely in the **lexer**.
This is the generic `lexer modes ⊗ completeness` hazard: an unterminated lexical mode is
invisible to any oracle defined over the token grammar, because the mode never produces a
token the grammar can miss.

## Decision

### 1. The parser's EOF-want remains the sole completeness oracle

`classify` is not broadened, not given delimiter counting, and not given a second grammar
model. `impl/04-continuation.md:25` stands unamended. `phalcom-repl/src/validator.rs` is not
modified by this record.

### 2. An unterminated lexical mode must co-emit `UnrecognizedEof`

When input ends inside a non-default lexer mode, the resulting diagnostic set must contain its
specific lexical error **and** `SyntaxErrorKind::UnrecognizedEof`. The existing narrow rule then
classifies the buffer `Incomplete` with no change to the validator.

The natural site is the lexical-error lowering in `phalcom-ast/src/parser.rs:146-147`, which
already maps `LexicalError::UnterminatedBlockComment` to `SyntaxErrorKind::UnterminatedComment`
— it emits both kinds rather than one. Note this is *not* a change inside the lexer proper: the
lexer's error vocabulary is unchanged, and only the lowering that feeds the parser's diagnostic
list gains a second entry.

Today the only such mode is the block comment. The obligation is stated over *modes*, not over
that one case, so it binds any future mode.

### 3. The obligation is permanent and belongs to whoever adds a mode

> **Every lexer mode that can be left open at end of input must co-emit `UnrecognizedEof`.**

A new heredoc, raw string, or nested-interpolation mode that does not honour this silently
regresses REPL continuation for its own syntax. Enforce it with a test per mode, in the same
commit that adds the mode.

## Consequences

- Block comments continue across REPL lines; one rule still governs classification.
- The fix is the same shape as `2fe6aba`, which routed end-of-file to `UnrecognizedEof` for the
  parser half of §D7. That precedent is now a pattern rather than a one-off.
- The layer boundary is deliberately, mildly muddied: the lexer emits an error kind named for
  a *parser* condition. The alternative — a distinct `IncompleteMode` kind that `classify`
  also checks — reintroduces the second oracle §D7 exists to prevent. The muddiness is the
  cheaper of the two costs and is accepted knowingly.
- **The cost, named plainly:** this is a standing obligation with no compiler enforcement. It
  is a rule in a document, and the failure mode is silent — a future heredoc simply stops
  continuing, and nobody notices until a user types one into the REPL. §3's per-mode test is
  the only real defence.

**What this precludes.** Committing to a single parser-signal oracle forecloses a
mode-stack-aware validator of the Ruby `irb` kind. If Phalcom ever grows enough lexer modes
that per-mode co-emission becomes unmanageable, reversing this needs a superseding PDR — and
that reversal is a genuine redesign, because a mode-aware validator is a second model of the
grammar with all the drift that implies.

## Alternatives rejected

- **Broaden `classify` to treat any unterminated-lexical-construct error as Incomplete.**
  Directly violates `impl/04-continuation.md:25`, and it is wrong on its own terms: it makes
  the validator carry lexical knowledge, so lexer and validator must then agree about which
  errors imply "more input might help." Two models, one grammar.
- **Delimiter counting in the REPL.** The named prohibition in §D7. Duplicates the grammar and
  breaks on the first construct where a delimiter is not a delimiter — a brace inside a string
  or a comment.
- **String-match the diagnostic** (CPython `codeop`, Node). Every new error message is a
  latent misclassification, and error wording stops being editable without breaking the REPL.
- **Expose lexer-mode state directly to `classify`.** Generalizes better than §2 and would
  scale to many modes without a per-mode obligation. Rejected today because it is precisely the
  second oracle §D7 bans, and Phalcom has exactly one mode. Revisit via superseding PDR if the
  mode count grows.
- **Leave it.** Defensible — many REPLs cannot continue a block comment either, and the cost
  is one keystroke of annoyance. Rejected because the inconsistency with unterminated strings
  is arbitrary from the user's seat: both are "I have not finished typing this," and only one
  is honoured.
