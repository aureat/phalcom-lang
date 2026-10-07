---
id: LANG007.C4.P11-P13
kind: implementation-walkthrough
title: "LANG007.C4.P11–P13 String/Char Remediation and Pattern Authority Walkthrough"
status: COMPLETE
completion: COMPLETE
verification: FOCUSED_TESTED
recorded_against_revision: 112396d13cb281534b07f2b9347dff054df35d1f
recorded: 2026-10-07
branch: main
worktree: .worktrees/lang007-c4-p4-r1-stdlib
---

# LANG007.C4.P11–P13 — Remediation and Completion Walkthrough

## 1. Overview and Objectives

This walkthrough documents the successful execution and delivery of `LANG007.C4.P11-P13-string-char-remediation-implementation-plan.md`.

The implementation converges the repository on:
1. **One semantic pattern authority**: `phalcom-semantic` resolves patterns and projects canonical executable specs into `phalcom-core` (`ExecutablePattern`). The compiler never re-interprets pattern semantics or uses unverified fallback paths.
2. **One UTF-8 decoding authority**: Rust's UTF-8 implementation powers String boundary validation and Unicode scalar decoding directly.
3. **P11 verification closure**: Lexer, parser, static/semantic, and runtime tests for `Char` including exact Unicode 17.0.0 toolchain pin.
4. **P12 String↔Char bridge**: Native `isCharBoundary`, `decodeCharAt`, and canonical `String.chars` returning `StringChars` (implementing `CanonicalIterable` with `Item = Char`).
5. **P13 String hygiene**: Fixed `rawByteAt` contract (`Option<Int>`), exact arbitrary-precision integer conversions rejecting Floats, bounds validation on `rawSlice`, and typed String APIs.
6. **Isolation of Math**: `universe.math` remains isolated on sibling branch `stdlib-random-math`. `phalcom-native-surface/src/generated.rs` verified clean with 375 declarations.

---

## 2. Completed Work by Task

### Preflight & Tasks 1–5 (`a3215f76`)
- Fixed `Pattern::Char` borrow typo (`*value`) in `phalcom-ast/src/parser.rs`.
- Wrapped `Some(value)` in `usefulness.rs::specialize_matrix_for_char_value`.
- Fixed `alloc_string_value(text.to_string())` and `returns = "Option<Char>"` in `phalcom-core/src/primitive/char.rs`.
- Registered `&["scalar", "char"]` in `UNIVERSE_NODES` and added `include_str!` for `char.ph` in `phalcom-modules/src/builtin.rs`.
- **Task 1**: Added `PatternSiteKind`, `PatternSiteKey`, `PatternSiteResolution`, `PatternSiteResolutionIndex`, and checking context recording. Published `pattern_site_resolutions` on `CallableAnalysis`. Added deterministic fingerprint hashing.
- **Task 2**: Migrated `if let` in semantic checker to formal `resolve_pattern` with causal invalidity preservation.
- **Task 3**: Migrated `while let` to formal `resolve_pattern`, isolating probe loops from publishing.
- **Task 4**: Migrated `for` lane patterns to formal `resolve_pattern` during probe and final passes.
- **Task 5**: Projected pattern sites into `ModuleLoweringSemantics` (`PatternSiteLoweringSpec`, `LoweringSiteKind::{IfLetPattern, WhileLetPattern, ForLanePattern}`).

### Task 6: Cutover to Staged Patterns (`c763f1ae`)
- Added `pattern_site_spec` helper in `phalcom-core/src/compiler/lib/patterns.rs`.
- Migrated `compile_if_let` and `compile_while_let` in `expr.rs` to staged `setup_pattern_frame`, `emit_executable_pattern`, and `commit_pattern_frame`.
- Migrated `compile_canonical_for_lanes` in `loops.rs` to staged frame execution with multi-lane atomic commit.
- Retired legacy raw-AST refutable pattern methods (`declare_pattern_locals`, `emit_pattern_match_tests`, `commit_pattern_bindings`, etc.).
- Fixed wildcard exhaustiveness in `CoverageEngine` when scrutinee is unannotated (`UnknownType`).
- Adjusted `MatchPatternImpossible` emission in `synthesize_match_expr` and `resolve_pattern_with_mode` so variant arms belonging to the scrutinee's enum family are not falsely rejected when the scrutinee has a refined variant subtype.

### Task 7: Match Fallback Hardening (`1d9c6043`)
- Enforced that `compile_match_expr` requires attached semantic `MatchLoweringSpec` or fails with `CompilerError::MissingMatchLoweringSemantics`.
- Removed `synthesize_fallback_pattern` from the compiler production path.

### Task 8: Declaration Pattern Semantic Legality (`d8bff298`)
- Added `DeclarationPatternRefutable` semantic diagnostic code (`"declaration.pattern_refutable"`).
- Implemented `validate_declaration_pattern` in `phalcom-semantic/src/checker/statement.rs` to reject refutable patterns (specifically `Pattern::Char`, `Pattern::Or`, `Pattern::Variant`, `Pattern::Map`) in declaration destructuring (`let` / `var`), including nested occurrences.

### Task 9: P11 Verification Matrix (`14dce5b6`)
- **Lexer tests** (`phalcom-ast/tests/lexer.rs`): Added valid scalar tokens (`'a'`, `'é'`, `'中'`, `'🙂'`, escapes, boundary scalars `\u{D7FF}`, `\u{E000}`, `\u{10FFFF}`) and invalid cases (`''`, `'ab'`, combining marks, raw newlines, invalid/surrogate escapes).
- **Parser tests** (`phalcom-ast/tests/parser.rs`): Tested `Expr::Char` AST expressions and patterns in `match`, `if let`, and or-patterns.
- **Semantic static tests** (`phalcom-semantic/tests/semantic/foundations/static_values.rs`): Verified `intern_char`, equality, hashing, content fingerprinting, and type domains.
- **Coverage tests** (`phalcom-semantic/tests/semantic/adts/matching/exhaustiveness.rs`): Verified exact + wildcard exhaustiveness, duplicate redundant arm detection, and impossible pattern detection against `Int`.
- **Coverage utility tests** (`usefulness.rs`): Verified `first_missing_scalar` around `U+D7FF`, `U+D800..=U+DFFF` excluded, `U+E000`, and `U+10FFFF`.
- **Runtime tests & Unicode pin** (`phalcom-core/tests/core/language/chars.rs`): Verified Unicode 17.0.0 toolchain pin, `codePoint`, `utf8Length`, `toString`, classification methods, `Char.fromCodePoint` boundaries/rejections, equality/identity, and exact pattern matching emitting `Bytecode::Same`.

### Tasks 10–13: String Native Root Fixes & P12/P13 (`112396d1`)
- **Task 10**: `rawByteAt` returns `Option<Int>`, using `expect_int_big` to reject floats and handle arbitrary precision; `rawSlice` validates integer bounds, rejection of floats, and char boundary alignment.
- **Task 11**: Implemented `String.isCharBoundary(_ byteOffset: Int) -> Bool` and `String.decodeCharAt(_ byteOffset: Int) -> Option<Char>`.
- **Task 12**: Implemented `String.chars -> StringChars` with `StringCharIterator` implementing `CanonicalIterator<Item = Char>` and `CanonicalIterable`. Fixed `StringCodePointSequence` coordinate system so `at(_ ordinal: Int) -> Option<Int>` uses scalar ordinals consistent with `size`.
- **Task 13**: Tightened String method return types (`split` returning `List<String>`, `contains`, `includes`, `replace`, `trim`), removed obsolete review comments in `primitive/string.rs`, and updated `phalcom-lsp` semantic tokens for `Token::Char` and `Expr::Char`.

### Tasks 14–15: Main Consolidation & Verification
- `universe.math` implementation remains on sibling branch `stdlib-random-math`.
- `phalcom-native-surface/src/generated.rs` verified clean with 375 declarations.

---

## 3. Verification Summary

All verification passed:
- `cargo test -p phalcom-core --test core language::chars`: 11 passed.
- `cargo test -p phalcom-ast --test integration character_literal`: 4 passed.
- `cargo test -p phalcom-semantic --test semantic adts::matching`: 158 passed.
- `cargo test -p phalcom-semantic --test semantic capabilities::patterns`: 6 passed.
- `cargo test -p phalcom-core --test language-corpus corpus::strings_negative`: 1 passed.
- `cargo test -p phalcom-core --test cli-smoke`: 3 passed.
- `cargo run -p phalcom-native-surface-gen -- --check`: verified clean 375 declarations.
- `cargo check --workspace`: clean, 0 errors, 0 warnings.
