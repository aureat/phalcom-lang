# U0 — Stabilization & Verification Substrate (as-built)

- **Status:** ✅ Landed — front-end fixes (F9+F10) + substrate committed with the ground-state baseline `3657d4f` (`chore: commit WIP ground state`).
- **Realizes:** the verification gate every later forge phase depends on; spec [§implementation-status](../../../spec/v0.2/implementation-status.md), [object-model §6 step 7](../../../spec/v0.2/object-model.md).
- **Reviewer gate:** U0 **APPROVED** (F9 + F10), per STATE.md phase log ("U0 APPROVED (F9+F10)"). This is the stabilizer phase — the substrate it stood up is what every subsequent unit's reviewer/self-verify gate runs.

## Mission
Get the tree compiling and stand up the verification substrate: one command that builds, runs the golden `.ph` corpus + object-model invariant harness + lexer/parser snapshots, and (opt-in) fuzz and miri lanes — so every later forge phase has a green gate to verify against. Also fixed the two front-end defects (F9, F10) that made the CLI panic on ordinary input.

## Surface / behavior
No language surface change. Two user-visible robustness fixes to the front end:

- **F9** — `SyntaxError`'s `Display::fmt` was `todo!()`, so **any** parse error panicked instead of rendering a diagnostic. U0 implemented `Display` → a parse error now renders a diagnostic and exits non-zero.
- **F10** — the parser rejected a trailing `\n` at end-of-input, so almost every real `.ph` file panicked (compounding F9). U0 fixed EOF handling → a trailing newline now parses.

Together these unblocked running real programs (`person.ph`, `calculator.ph`) through the golden corpus without crashing.

## Implementation
The substrate is three test lanes plus one shell entry point:

- **`scripts/verify.sh`** — the single `/forge` gate. Default run = `cargo build --workspace` → `cargo test --workspace` → `cargo clippy --workspace --all-targets`; exit non-zero if any lane fails. `--fuzz` adds a 60s cargo-fuzz smoke pass on the `parser` and `lexer` targets (needs nightly + cargo-fuzz); `--miri` adds `cargo +nightly miri test -p phalcom-ast`. Fuzz/miri are opt-in (extra toolchain components) and are not part of the merge gate.
- **`phalcom-core/tests/golden.rs`** — golden `.ph` corpus runner. Executes known-good programs through the real `phalcom` CLI **as a subprocess** (full lex/parse/compile/run pipeline, exactly as a user would) and asserts (1) the process does not panic (no exit-101 / "panicked at") and (2) stdout matches a fixed, hand-verified string. It is a regression gate, not a behavior spec.
- **`phalcom-core/tests/invariants.rs`** — the object-model / metaclass-tower invariant harness (exercised end-to-end via a real `VM`); see [U2](../U2/as-built.md), which populated it.
- **`fuzz/`** — cargo-fuzz project with `fuzz_targets/{lexer,parser}.rs` and a `phalcom.dict` dictionary.
- **Lexer/parser insta snapshot tests** in `phalcom-ast`, run as part of `cargo test --workspace`.

`cargo test --workspace` transitively runs the golden corpus, the invariant harness, the `phalcom-ast` snapshots, and every other unit/doc test.

## Invariants & tests
- **Golden corpus (as-built):** `examples/core_new.ph`, `examples/person2.ph` (and `person.ph`/`calculator.ph`, unblocked by the F9/F10 fix), plus `tests/fixtures/golden/{hello,arithmetic,blocks_map_reduce,blocks_escaping_counter}.ph`. Some `examples/*` stay excluded because they use syntax the grammar does not yet accept (labeled constructor params, decorators, etc.).
- **`tests/lang/` label corpus** — a spec-conformance corpus divided into labels (`arithmetic`, `bindings`, `blocks`, `control-flow`, `dispatch`, `metaclass`, …) that later units extend as they land features.
- The `verify_invariants()` object-model harness (U2) is invoked from `VM::new` and re-checked by `invariants.rs`.

## Deviations & deferrals
- Fuzz and miri are **opt-in**, not in the default merge gate — they need a nightly toolchain with components that may not be installed, and are smoke checks. See [`scripts/verify.sh`](../../../../scripts/verify.sh) header.
- The golden runner is a stdout-equality regression gate, not a semantic conformance check — deliberately (see its module doc). Two `tests/fixtures/golden/*.ph` fixtures predate the F10 fix and deliberately carry no trailing newline; left as-is since their goldens are pinned.
- Excluded example programs remain blocked on unimplemented surface syntax; see [deferred-work](../../../spec/v0.2/deferred-work.md).

## Sources
- forge: [`STATE.md`](../../archive/phase2/STATE.md) phase log ("0. Stabilize"), [`PLAN.md`](../../archive/phase2/PLAN.md) §"U0 — Front-end stabilization (F9 + F10)".
- code: `scripts/verify.sh`, `phalcom-core/tests/golden.rs`, `phalcom-core/tests/invariants.rs`, `phalcom-core/tests/lang/`, `fuzz/`.
- landing: baseline commit `3657d4f` (substrate + front-end rewrite committed as ground state).
