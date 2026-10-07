---
id: LANG007.C4.P11-P13
kind: implementation-handoff
title: "LANG007.C4.P11–P13 String/Char Remediation and Pattern Authority Handoff"
status: COMPLETE
completion: COMPLETE
verification: FOCUSED_TESTED
recorded_against_revision: 112396d13cb281534b07f2b9347dff054df35d1f
recorded: 2026-10-07
branch: main
worktree: .worktrees/lang007-c4-p4-r1-stdlib
---

# LANG007.C4.P11–P13 — Remediation and Completion Handoff

## 1. Executive Summary & Active Context

This handoff documents the completed implementation state for `LANG007.C4.P11-P13-string-char-remediation-implementation-plan.md`.

- **Repository:** `aureat/phalcom-lang`
- **Active Branch:** `main`
- **Delivered Revision:** `112396d1`
- **Status:** `COMPLETE / COMPLETE / FOCUSED_TESTED`

All tasks of `LANG007.C4.P11-P13` have been implemented and verified. The
`universe.math` implementation and its Family/static-constant corrections are
now consolidated into `main` with this work. The merge titled
`merge: consolidate math and Char corrective work` preserves both parent
histories (`e0de68b2` and `617d99f5`). See the
[combined verification record](../math/stdlib-math-char-main-consolidation.md).

---

## 2. Established Architecture & Invariants

1. **Semantic Pattern Authority**:
   - `phalcom-semantic` owns all pattern analysis and publishes canonical `PatternSiteResolutionIndex` for non-match control patterns (`if let`, `while let`, `for`).
   - `phalcom-core` consumes `ModuleLoweringSemantics::pattern_sites` and executes patterns through staged `PatternExecutionFrame` and `ExecutablePattern`.
   - Raw-AST refutable pattern compilation helpers in `Compiler` have been retired.
   - `compile_match_expr` requires attached semantic `MatchLoweringSpec` and fails closed with `MissingMatchLoweringSemantics` if missing. Fallback pattern synthesis is eliminated.
   - `DeclarationPatternRefutable` semantic diagnostic is emitted for refutable patterns in `let` / `var` destructuring.

2. **Char Representation & Unicode Pin**:
   - `Char` is an immediate 16-byte `Value` representing exactly one Unicode scalar `U+0000..U+D7FF ∪ U+E000..U+10FFFF`.
   - Toolchain Unicode tables pinned to version `(17, 0, 0)` in `chars.rs`.
   - Pattern matching on Char literals compiles to `Bytecode::Constant` + `Bytecode::Same`, without calling user equality methods.

3. **String↔Char Bridge**:
   - `String.isCharBoundary(_ byteOffset: Int) -> Bool` verifies UTF-8 byte boundaries natively using Rust `str::is_char_boundary`.
   - `String.decodeCharAt(_ byteOffset: Int) -> Option<Char>` decodes the Unicode scalar at byte boundaries natively.
   - `String.chars -> StringChars` provides canonical character iteration yielding `Char` instances, implementing `CanonicalIterable` with `Item = Char`.
   - `StringCodePointSequence` coordinate system repaired: `at(_ ordinal: Int) -> Option<Int>` indexes by scalar ordinal, matching `size`.

4. **String Native Contracts**:
   - `rawByteAt` signature is `Option<Int>`, using `expect_int_big` to reject floats and handle arbitrary precision integers.
   - `rawSlice` validates integer bounds, float rejection, and UTF-8 char boundaries.
   - String manipulation methods (`split`, `indexOf`, `contains`, `replace`, `trim`) return typed collections (`List<String>`, `String`, etc.).

5. **Surface Generation**:
   - `phalcom-native-surface-gen` includes 399 primitive declarations covering both math and char/string primitives. Verified clean with `--check`.

---

## 3. Verification Commands

The focused verification suite for this plan:

```sh
# Char tests (runtime, primitives, Unicode pin, identity, pattern matching)
RUSTFLAGS='' cargo test -p phalcom-core --test core language::chars

# AST lexer & parser tests
RUSTFLAGS='' cargo test -p phalcom-ast --test integration character_literal

# Semantic ADT matching & pattern destructuring
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::matching
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic capabilities::patterns

# Module parity and math integration
RUSTFLAGS='' cargo test -p phalcom-core --test core parity

# Language corpus strings negative tests
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus corpus::strings_negative

# Workspace compilation and native surface check
RUSTFLAGS='' cargo check --workspace
RUSTFLAGS='' cargo run -p phalcom-native-surface-gen -- --check
```

---

## 4. Next Actions

1. Review and close out `LANG007.C4.P11-P13`.
2. Follow-on plans may consume `String.chars`, `Char`, and the unified pattern lowering infrastructure.
