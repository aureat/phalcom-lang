# U-NUMBERS-06 — integration, census, and verification

## Outcome

Land numbers as one coherent surface: frozen primitive census correct, docs/status synchronized,
and behavior covered across parser, VM, GC, diagnostics, and keyed collections.

## Checklist

1. Recompute ADR-0019 primitive delta from installed bindings. Account separately for the existing
   split and PDR-0027's +10 Float protocol / +2 power bindings; update the floor amendment and
   invariant constants in the same commit. Never trust historical 137/153 prose as a live count.
2. Update core-class census rows and all selector symbol/operator inventories. Verify `**` is a
   direct selector key, not a textual alias.
3. Run focused lexer/parser/compiler, numeric primitive, Map/Set, diagnostics, and invariant
   suites; then workspace tests. Add property/fuzz cases for literals, text round trips, integer
   laws, and Float key equivalence. Run GC-stress around constants and arithmetic.
4. Run the project verification command if available, `cargo fmt --check`, and `git diff --check`.
   Record exact commands/results in implementation handoff rather than claiming an unrun green
   suite.
5. Synchronize PDR status, numeric specs, deferred register, fixtures, and implementation status.
   Run `graphify update .` after the final edit; inspect only the scoped numeric graph result.

## Release blockers

- no compiler-created LargeInt constant until its GC-root test is green;
- no numeric path that panics, leaks raw host parse text, or bypasses `#numericLimit`;
- no numeric runtime error with an available source span that omits the rich traceback caret;
- no Map/Set violation of equal numeric keys having equal hashes.
