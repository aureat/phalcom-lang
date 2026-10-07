---
id: LANG007.C4.P11-P13
kind: implementation-handoff
title: "LANG007.C4.P11–P13 String/Char Remediation and Pattern Authority Handoff"
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
recorded_against_revision: a3215f76297de502bcec14b7aaf1205762c8d865
recorded: 2026-10-07
branch: codex/lang007-c4-p4-r1-stdlib
worktree: .worktrees/lang007-c4-p4-r1-stdlib
---

# LANG007.C4.P11–P13 — Remediation and Completion Handoff

## 1. Executive Summary & Active Context

This handoff documents the implementation state for `LANG007.C4.P11-P13-string-char-remediation-implementation-plan.md`.

- **Repository:** `aureat/phalcom-lang`
- **Active Branch:** `codex/lang007-c4-p4-r1-stdlib`
- **Active Worktree:** `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/lang007-c4-p4-r1-stdlib`
- **Starting Tip:** `51c4d35a6820b334599a3872185fa64d8fab0651`
- **Intermediate Head:** `a3215f76297de502bcec14b7aaf1205762c8d865` (`feat: advance Unicode scalar support`)
- **Sibling Lane:** `stdlib-random-math` (checked out in root repo workspace `/Users/altunhasanli/dev/phalcom/phalcom`)
- **Common Sibling Merge Base:** `615f5887f262f8adcb666f4cb4cb431e1a828e11`

> **CRITICAL WORKTREE INSTRUCTION:**
> The main directory `/Users/altunhasanli/dev/phalcom/phalcom` is checked out on branch `stdlib-random-math`. All development, tests, and git operations for this plan MUST be performed inside `.worktrees/lang007-c4-p4-r1-stdlib`.

---

## 2. Completed Tasks in Revision `a3215f76`

Tasks 0 through 5 from the implementation plan were completed and committed in `a3215f76`:

1. **Preflight Mechanical Fixes (from commit `800e1793`):**
   - `phalcom-ast/src/parser.rs:3646`: Fixed `*value` borrow typo for `Pattern::Char`.
   - `phalcom-semantic/src/checker/coverage/usefulness.rs:554`: Wrapped `Some(value)` in `specialize_matrix_for_char_value`.
   - `phalcom-core/src/primitive/char.rs:36, 104`: Fixed `text.to_string()` for `alloc_string_value` and corrected `returns = "Option<Char>"`.
   - `phalcom-modules/src/builtin.rs:270, 804`: Registered `&["scalar", "char"]` in `UNIVERSE_NODES` and added `include_str!` for `char.ph`. (Previously missing, causing `ordinary native Universe binding Char has no canonical source declaration at universe:scalar.char::Char` panic).

2. **Task 1 — Semantic Pattern-Site Products:**
   - Added `PatternSiteKind` (`IfLet`, `WhileLet`, `ForLane`), `PatternSiteKey`, `PatternSiteResolution`, and `PatternSiteResolutionIndex` to `phalcom-semantic/src/match_semantics.rs`.
   - Added context storage and `record_pattern_site_resolution` to `phalcom-semantic/src/checker/context.rs`.
   - Published `pattern_site_resolutions: Arc<PatternSiteResolutionIndex>` on `CallableAnalysis` in `phalcom-semantic/src/checker/analysis.rs`.
   - Added deterministic fingerprint hashing `hash_pattern_site_resolutions` in `phalcom-semantic/src/db/fingerprint.rs`.
   - Added `attach_causal_invalidity_to_binding` in `CheckingContext`.

3. **Task 2 — Migrate `if let` Semantic Analysis:**
   - Replaced legacy incomplete `bind_pattern` in `Expr::IfLet` with formal `resolve_pattern` in then-branch scope.
   - Preserved `val_typed.causal_invalidity` attachment.
   - Extracted shared `coverage_subject_for_expression`.
   - Published `PatternSiteResolution(kind=IfLet)`.

4. **Task 3 — Migrate `while let` Semantic Analysis:**
   - Replaced legacy incomplete `bind_pattern` in `Expr::WhileLet` with formal `resolve_pattern`.
   - Isolated probe iterations (`is_real_pass = false`) so probes do not publish into callable products or emit premature diagnostics.
   - Published canonical `PatternSiteResolution(kind=WhileLet)` only during the final real pass.
   - Removed dead `bind_pattern` from `checker/expression.rs`.

5. **Task 4 — Migrate `for` Lane Semantic Analysis:**
   - Preserved canonical `resolve_for_iteration_lane`.
   - Formed `CoverageSubject` for each lane element fact.
   - Resolved lane patterns formally with `resolve_pattern` during probe and final passes.
   - Published canonical `PatternSiteResolution(kind=ForLane)` for each lane during the final real pass.
   - Removed dead `bind_pattern` from `checker/statement.rs`.

6. **Task 5 — Project Pattern Sites into `ModuleLoweringSemantics`:**
   - Introduced `PatternSiteLoweringSpec` in `phalcom-core/src/modules/semantic_lowering.rs`.
   - Added `LoweringSiteKind::{IfLetPattern, WhileLetPattern, ForLanePattern}`.
   - Projected `PatternSiteResolution` into `ExecutablePattern` using `project_pattern_site_resolution`.
   - Added `pattern_sites: BTreeMap<LoweringSite, PatternSiteLoweringSpec>` to `ModuleLoweringSemantics` and re-exported in `phalcom-core/src/modules/mod.rs`.

---

## 3. In-Progress Task 6: Compiler Cutover to Staged Patterns

The working tree in `.worktrees/lang007-c4-p4-r1-stdlib` contains the Task 6 implementation:

- **`phalcom-core/src/compiler/lib/patterns.rs`:**
  - Added helper `pattern_site_spec(kind: LoweringSiteKind, range: SourceRange) -> Result<PatternSiteLoweringSpec, CompilerError>`.
  - Retired the legacy raw-AST refutable pattern methods (`declare_pattern_locals`, `emit_pattern_match_tests`, `commit_pattern_bindings`, `assign_pattern_from_top`, `assign_pattern_from_slot`).
- **`phalcom-core/src/compiler/lib/expr.rs`:**
  - Migrated `compile_if_let` to `setup_pattern_frame`, `emit_executable_pattern`, and `commit_pattern_frame`.
  - Migrated `compile_while_let` to staged frame execution and atomic commit on success.
  - Removed dead `collect_pattern_names_for_control`.
- **`phalcom-core/src/compiler/lib/loops.rs`:**
  - Migrated `compile_canonical_for_lanes` to staged frame execution: sets up `lane_frames`, stages all lane patterns, checks shared pattern failure, and commits all frames atomically.
  - Removed dead `collect_pattern_names`.

---

## 4. Architectural Discoveries & Root Causes

During bootstrap verification, three crucial issues were uncovered:

### 1. Universe Match Exhaustiveness with Wildcards (`canonical_universe.rs`)
- **Symptom:** `canonical Universe semantic product is not executable: match in universe:concurrency.fiber ... is blocked coverage proof: UnknownType(UnannotatedDeclaration)`
- **Root Cause:** In commit `800e1793`, `scrutinee_knowledge_block` was added to `synthesize_match_expr`. In cold analysis, several Universe variables have `Unknown` type (e.g. `inputs.at(i)`).
  `CoverageEngine::finalize_exhaustiveness` had:
  ```rust
  if let Some(reason) = self.blocked.clone() {
      return ExhaustivenessResult::Blocked(reason);
  }
  ```
  If `self.blocked` was set at match start, it returned `Blocked` immediately, ignoring whether the user supplied a catch-all wildcard `_ => ()` arm!
  Moreover, `expression.rs:5381` had `if usefulness == Useful && engine.blocked_reason().is_none() { engine.commit_arm(coverage_pat); }`, which refused to commit the wildcard arm to the engine if `blocked_reason` was present.
- **Fix Applied:**
  1. In `usefulness.rs::finalize_exhaustiveness`, check `has_wildcard`: if a wildcard arm is present in the matrix, the match is proven exhaustive across all values. If `!has_wildcard`, it returns `Blocked(reason)`.
  2. In `expression.rs:5381`, commit useful arms without checking `engine.blocked_reason().is_none()`.

### 2. Unqualified Variant in `iterator_laws.ph`
- **Symptom:** `variant Some has no declaration-backed owner for scrutinee type at CopyRange { start: 6478, end: 6487 }`
- **Root Cause:** `phalcom-core/core/universe/src/collections/iterator_laws.ph:263` had `while let Some(val) = it.next {`.
  In trait default code, `Some` was unqualified.
- **Fix Applied:** Changed to `while let Option::Some(val) = it.next {`.

### 3. `MatchPatternImpossible` Diagnostics for Refined Variants
- **Symptom:** In `adt_run_01` (`match value { State::Ready => 1, State::Done => 0 }` where `value = State::Ready`), compilation fails with:
  `Error MatchPatternImpossible: impossible match pattern: it cannot match any value of the scrutinee type at CopyRange { start: 114, end: 130 }`.
- **Root Cause:** Commit `800e1793` added:
  ```rust
  } else if usefulness == crate::match_semantics::PatternUsefulness::Impossible {
      ctx.emit_diagnostic(MatchPatternImpossible);
  }
  ```
  When `value` was inferred as an exact variant singleton (`State::Ready`), `CoverageSubject` used that narrowed type. The other variant arm (`State::Done`) was classified as `Impossible`. Emitting a hard error for valid enum arms breaks standard ADT matching.
- **Action Needed:** In `synthesize_match_expr`, do NOT emit `MatchPatternImpossible` if the pattern is a variant belonging to the scrutinee's parent enum/data declaration, or only emit it when `scrutinee_subject.canonical` is an incompatible nominal type (e.g. `Char` pattern against `Int`).

---

## 5. Next Steps for Implementation

The next agent should execute the remaining tasks in order:

1. **Finish Task 6 & Fix `MatchPatternImpossible` in `synthesize_match_expr`:**
   - Adjust `synthesize_match_expr` in `expression.rs` so that variant arms of the scrutinee's ADT/family do not emit `MatchPatternImpossible` when the scrutinee has a refined variant subtype.
   - Verify `cargo test -p phalcom-core --test core language::algebraic_data` passes.
   - Commit Task 6.

2. **Task 7 — Match Fallback Hardening (`match_expr.rs`):**
   - Enforce that `compile_match_expr` requires attached semantic `MatchLoweringSpec`.
   - Remove `synthesize_fallback_pattern` from production path.
   - Emit `CompilerError::MissingMatchLoweringSemantics` if lowering spec is absent.

3. **Task 8 — Declaration Pattern Semantic Legality:**
   - In `phalcom-semantic/src/checker/statement.rs`, validate that declaration patterns in `let` / `var` do not contain refutable patterns (specifically `Pattern::Char`).
   - Emit a semantic diagnostic instead of relying on compiler `contains_char_literal`.

4. **Task 9 — Close P11 Char Verification Matrix:**
   - Add lexer tests in `phalcom-ast/tests/lexer.rs` (valid scalar tokens, escapes, surrogates/overflow errors).
   - Add parser tests in `phalcom-ast/tests/parser.rs`.
   - Add semantic/static/coverage tests for Char in `phalcom-semantic/tests/semantic/`.
   - Add runtime Char tests in `phalcom-core/tests/`.
   - Add Unicode version pin test (`UNICODE_VERSION`).

5. **Tasks 10–13 — String Root Fixes and P12/P13:**
   - `rawByteAt` returns `Option<Int>`.
   - Arbitrary-precision Int extraction via `expect_int_big`. Reject Float for Int parameters.
   - Implement `isCharBoundary(_ index: Int) -> Bool` and `decodeCharAt(_ index: Int) -> Option<Char>`.
   - Implement `String.chars` yielding `Char`.
   - Repair `StringCodePointSequence` coordinate coherence.
   - Tighten String collection types (`List<String>`).
   - Remove stale blocker comments from `primitive/string.rs`.

6. **Task 14 & 15 — Integration, Surface Regeneration & Delivery:**
   - Reconcile authored sources with `stdlib-random-math`.
   - Regenerate `phalcom-native-surface/src/generated.rs`.
   - Run focused validation gates and record lifecycle completion.
