# 16. Hand-written lexer and recursive-descent parser (replacing LALRPOP)

- Status: Accepted
- Date: 2026-07-11
- Related: `docs/spec/v0.2/lexical-structure.md`; `docs/spec/v0.2/classes.md`;
  `docs/spec/v0.2/messages-and-selectors.md`; forge fixes F9, F10;
  [ADR-0012](0012-selector-signature-encoding-and-dispatch.md)

## Context

The front end (`phalcom-ast`) tokenised with the `logos` derive-macro lexer and
parsed with a LALRPOP grammar (`parser.lalrpop`) compiled by a `build.rs`
codegen step. Two structural problems made this a poor foundation for the
language the spec describes:

- **The grammar has to grow, incrementally and with good diagnostics.** The spec
  roadmap adds blocks, `construct`, labeled/keyword selectors, string
  interpolation, and `let`/`var` and literal syntax
  ([Lexical Structure](../spec/v0.2/lexical-structure.md),
  [Messages & Selectors](../spec/v0.2/messages-and-selectors.md)). Several of these
  are context-sensitive (brace disambiguation, interpolation, newline
  suppression) and awkward or impossible to express cleanly in an LALR grammar.
- **Error quality and recovery were weak.** A LALR parser reports a single error
  and stops; it cannot easily synchronise and continue. The generator also
  produced opaque `expected`-set messages, and two concrete defects surfaced in
  the forge audit: `SyntaxError`'s `Display` was `todo!()` so *any* parse error
  panicked (F9), and a file ending in a trailing newline failed to parse (F10).
- **A build-time codegen dependency** (`lalrpop` as a `build-dependency` plus a
  `build.rs`) added toolchain surface and slowed clean builds for no runtime
  benefit.

## Decision

Replace the generated front end with a fully hand-written one in `phalcom-ast`:

- **Hand-written lexer** (`src/lexer.rs`): a byte-oriented scanner with maximal
  munch, no scanner generator and no regex/`logos` dependency. It emits the
  existing `Token` set, treats newlines as tokens, skips `//` comments and
  horizontal whitespace as trivia, and injects a single end-of-file token at the
  end of the last real token so trailing whitespace/newlines are located
  correctly.
- **Hand-written parser** (`src/parser.rs`): recursive descent for
  statements and declarations, and precedence climbing (Pratt-style) for
  expressions and message sends, over a fixed operator-precedence table. It
  produces the same `ast.rs` nodes with the same `@L`/`@R` span semantics the
  LALRPOP parser produced, so the compiler and all AST snapshots are unchanged.
- **Panic-mode error recovery**: on a bad top-level statement the parser records
  the error and synchronises to the next statement boundary, so one parse can
  surface many diagnostics. The public `parse_source` keeps the historical
  single-error contract (returns the first error); a new `parse` entry point
  returns the full recovered set.
- **F9 and F10 are folded in**: `SyntaxError`'s `Display` renders a real spanned
  diagnostic (the CLI exits non-zero instead of panicking), and a trailing
  newline / empty / whitespace-only file parses cleanly.
- **LALRPOP is removed from the front end**: `parser.lalrpop`, the `build.rs`
  codegen step, and the `lalrpop`/`lalrpop-util`/`logos` dependencies are
  dropped from `phalcom-ast`.

This unit deliberately keeps the *accepted language at parity* with the old
grammar (no new syntax); the greenfield constructs above land as later units
that extend this hand-written parser.

## Consequences

- The parser is now ordinary Rust: greppable, debuggable, and extensible without
  regenerating a grammar. Context-sensitive constructs on the roadmap can be
  added with targeted lookahead instead of fighting LALR conflicts.
- Diagnostics improve: multiple errors per run, precise spans, and no `todo!()`
  panic path. `parse_source` stays source-compatible for the compiler.
- One less build-time codegen dependency in `phalcom-ast`; clean builds no longer
  run a grammar generator.
- The hand-written parser owns operator precedence and associativity explicitly
  (a precedence table), which is the thing most likely to drift from the spec;
  it is covered by AST snapshot tests.
- **Residual cleanup (follow-up).** `phalcom-core` still carries a dead
  `CompilerError::ParseError` variant and `From` impl referencing
  `lalrpop_util::ParseError`, plus the `lalrpop-util`/`lalrpop` entries in its
  `Cargo.toml`. These compile but keep LALRPOP in the workspace dependency graph;
  removing them is out of this unit's write-set and is tracked for a follow-up so
  the removal is truly workspace-wide.

## Alternatives considered

- **Keep LALRPOP, improve error recovery.** LALRPOP's recovery story is limited
  and the context-sensitive roadmap constructs remain awkward; this only defers
  the problem. Rejected.
- **A different parser generator (e.g. `pest`, `chumsky`).** Swaps one codegen/DSL
  dependency for another and still constrains how context-sensitive syntax is
  expressed. A hand-written parser gives the most control for a language that is
  explicitly going to grow. Rejected.
- **A generated lexer (`logos`) with a hand-written parser.** Reasonable, but the
  lexer is small and needs its own context-sensitive behaviour (newline handling,
  future interpolation); owning it removes the last codegen dependency and keeps
  span control local. Rejected in favor of a fully hand-written front end.
