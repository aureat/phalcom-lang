# Handoff — comprehensive, label-divided Phalcom test corpus

_Dispatch-ready prompt for one testing agent. Builds on the preliminary corpus already in
`phalcom-core/tests/lang/` (flat `.ph` + `.expected` + `MANIFEST.md`, not yet wired to a harness).
This phase makes it **comprehensive**, covers **pending (not-yet-implemented) spec parts**, and
**divides every test by a feature label so the suite runs feature-by-feature**._

---

## 0. Mission (one sentence)
Turn the preliminary `.ph` corpus into a comprehensive, spec-anchored acceptance suite — organized
by feature **label**, wired into a Rust test harness that runs any single feature in isolation, with
not-yet-built spec features present as **PENDING** tests that light up as the VM is built out.

## 1. Where this runs — branch & git
- **Branch off `main`**: `git switch main && git switch -c test/lang-corpus`. `main` is the ground
  truth (specs, ADRs, forge plans, this file). Do all work here.
- **Rebase on `main` regularly.** The VM spine (U1 heap → U2 metaclass → U3 dispatch → …) is landing
  fast on `main`; each unit flips some PENDING tests to PASS. Your files never overlap the spine's
  write-set, so `git rebase main` is clean — do it often and re-baseline the manifest.
- **Commits**: Conventional Commits, `test(lang): …` scope. Small, feature-scoped commits (one label
  or one coverage area per commit) so history reads feature-by-feature too. End every commit body with:
  `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.
- **Gate before you hand back**: `cargo test -p phalcom-core --test lang` green (PASS + NEGATIVE
  lanes; PENDING is skipped), then `./scripts/verify.sh` green (nothing workspace-wide broke).
- **Do not push**; the user controls `origin`.

## 2. Orientation — core behavior first (read before writing tests)
Phalcom is a **class-based, Smalltalk-style** language compiled to bytecode on a stack VM (Rust
workspace). Tests are `.ph` programs exercised **end-to-end through the CLI**, each paired with its
expected stdout/exit.

- **Source of truth = `docs/spec/`.** Base every expected result on the spec — never invent syntax or
  semantics. Read at least: `README.md`, `lexical-structure.md`, `values-and-absence.md`,
  `messages-and-selectors.md`, `method-lookup.md`, `control-flow.md`, `classes.md`, `blocks.md`,
  `object-model.md`, plus the WIP parts `functions.md`, `error-handling.md`, `system.md`,
  `concurrency.md`. Cross-check ratified decisions in `docs/adr/0007`–`0016`.
- **`docs/spec/implementation-status.md`** tells you what is built vs greenfield. The current tree is
  a **partial** VM — many spec-correct programs fail today. That is expected: they become PENDING.
- **Calibrate current syntax empirically**: run `examples/core_new.ph`, `person2.ph`, `person.ph`,
  `calculator.ph`, and skim the existing `phalcom-core/tests/lang/*.ph` for the header/comment style.
- **Determinism**: no timestamps, addresses, or unordered iteration in expected output.

### graphify (structure only — spec + CLI runs are your real references)
`graphify-out/graph.json` exists. Before reading VM source to understand behavior, orient with
graphify, don't blind-grep:
- `graphify query "how does <feature> dispatch/execute"` — scoped subgraph.
- `graphify explain "<symbol>"` · `graphify path "<A>" "<B>"`.
You will rarely need source — the spec defines expected behavior and the CLI defines actual. If you
do add/rename anything structural, `graphify update . --no-cluster` after.

## 3. The label system + harness — THE core requirement
**Label = feature area = one directory under `phalcom-core/tests/lang/<label>/`** and **one `#[test]`
function** in a new harness file `phalcom-core/tests/lang.rs`. This gives feature-by-feature runs for
free via cargo's name filter:

```
cargo test -p phalcom-core --test lang arithmetic     # just the arithmetic feature
cargo test -p phalcom-core --test lang classes        # just classes
cargo test -p phalcom-core --test lang                 # whole suite (PENDING skipped)
cargo test -p phalcom-core --test lang -- --ignored     # run the PENDING lanes to check progress
```

### Directory & status model
- `tests/lang/<label>/*.ph` (+ sibling `.expected`) — **active** cases: must PASS (or clean-error for
  negative labels). These gate.
- `tests/lang/<label>/pending/*.ph` — **PENDING** target behavior for not-yet-built spec parts;
  registered under an `#[ignore]`d test so it documents the target without breaking the gate. Runs
  with `--ignored` to track progress; the owning spine unit removes the `#[ignore]` when it lands.
- **NEGATIVE** cases live in negative labels (`syntax-errors`, `runtime-errors`): they must exit
  **non-zero with a clean diagnostic — never a Rust panic**. A case that currently panics is a real
  bug: put it under `pending/` with a `// PANICS TODAY` note and record it in the manifest.

### Harness skeleton (`phalcom-core/tests/lang.rs` — self-contained, NO new Cargo.toml deps)
Resolve the CLI binary with `env!("CARGO_BIN_EXE_phalcom")` (do **not** touch `golden.rs`). One
support module + one fn per label:

```rust
//! Language acceptance corpus. One #[test] per feature label; each globs
//! tests/lang/<label>/*.ph and checks stdout+exit against the .expected sidecar.
//! Run one feature: `cargo test -p phalcom-core --test lang <label>`.
mod support; // run_ph(path)->Output; check_pass(dir); check_negative(dir);

#[test] fn lexical()      { support::check_pass("lexical"); }
#[test] fn arithmetic()   { support::check_pass("arithmetic"); }
#[test] fn booleans()     { support::check_pass("booleans"); }
#[test] fn bindings()     { support::check_pass("bindings"); }
#[test] fn messages()     { support::check_pass("messages"); }
#[test] fn dispatch()     { support::check_pass("dispatch"); }
#[test] fn classes()      { support::check_pass("classes"); }
#[test] fn control_flow() { support::check_pass("control-flow"); }

#[test] fn syntax_errors()  { support::check_negative("syntax-errors"); }
#[test] fn runtime_errors() { support::check_negative("runtime-errors"); }

// PENDING lanes — spec targets not yet built. Un-ignore in the unit that lands them.
#[test] #[ignore = "PENDING: absence/Option — U6"]        fn absence()     { support::check_pending("absence"); }
#[test] #[ignore = "PENDING: metaclass tower — U2"]       fn metaclass()   { support::check_pending("metaclass"); }
#[test] #[ignore = "PENDING: blocks/closures — U4"]       fn blocks()      { support::check_pending("blocks"); }
#[test] #[ignore = "PENDING: functions — later"]          fn functions()   { support::check_pending("functions"); }
#[test] #[ignore = "PENDING: errors/Result — later"]      fn errors()      { support::check_pending("errors"); }
#[test] #[ignore = "PENDING: System/IO — later"]          fn system()      { support::check_pending("system"); }
```

`support` (in `tests/support/mod.rs`) must, for each `.ph` in a label dir: run the CLI, and
- `check_pass`: assert exit 0 and stdout == the `.expected` sidecar (define a trailing-newline policy
  and apply it uniformly); a missing `.expected` is a hard error.
- `check_negative`: assert exit ≠ 0, stdout/stderr contains the expected diagnostic substring (from a
  `.expected` note), and the output is **not** a Rust panic (`thread 'main' panicked` ⇒ fail).
- `check_pending`: same as `check_pass` but this fn is only reached under `--ignored`; failures here
  are informational (they record how far the target is from reality).
Aggregate per-file results and, on failure, print `label/case.ph` + a stdout diff so a red run
pinpoints the exact file.

> Rationale for per-label fns over a file-discovery crate (`datatest-stable`/`libtest-mimic`): those
> give per-file test names but require a `[dev-dependencies]` + `[[test]]` edit to
> `phalcom-core/Cargo.toml` — which **U1 is actively rewriting**, so it would merge-conflict. The
> per-label harness needs zero Cargo.toml changes and still runs feature-by-feature. Once the spine
> settles, an upgrade to per-file discovery is a clean follow-up (note it in `DEFERRED.md`).

## 4. Coverage map — comprehensive, incl. PENDING WIP parts
Balanced comprehensiveness: broad surface + sharp **edge** and **negative** cases per label, not
exhaustive permutations. Each label anchors to a spec §; mark status per the model above.

| Label | Spec § | Focus (edge cases in **bold**) | Likely status today |
|---|---|---|---|
| `lexical` | lexical-structure | int/float (**zero, negative, large, fractional, precision boundary**), strings (**empty, escapes, unicode**), comments, blank lines, **empty program** | PASS |
| `arithmetic` | (numbers/ops) | precedence & associativity, **div-by-zero**, mixed float/int, **unary minus** | PASS |
| `booleans` | (control-flow) | `true`/`false`, `and`/`or` **short-circuit side-effect order**, `not`, comparisons | mixed |
| `bindings` | values-and-absence, ADR-0014 | `let` immutable, `var` mutable, **`var x` uninit reads as absence**, shadowing, **read-before-write compile error**, nested scope | mixed |
| `messages` | messages-and-selectors | unary/binary/keyword sends, **selector arity+labels (`move(to:)` ≠ `move(_:)`)** | mixed |
| `dispatch` | method-lookup | resolve to right method, inheritance & override, `super`, **doesNotUnderstand (negative)** | mixed |
| `classes` | classes, object-model | definition, fields (private/default), methods, **getter ≠ method**, setters, static/class-side, `construct`, identity, reflection (`name`/`toString`/`class`) | mixed |
| `control-flow` | control-flow | `if`/`else`, loops, **conditionals-as-messages**, empty body, false branch | mixed |
| `absence` | values-and-absence, ADR-0007 | `Option`/`Some`/`None`, no surface `nil`, **`if (opt)` is a compile error (no truthiness)** | **PENDING (U6)** |
| `metaclass` | object-model §5–6, ADR-0002/0003 | class-side inheritance, parallel tower, `Behavior` | **PENDING (U2)** |
| `blocks` | blocks, ADR-0013 | block literals, args, **capture + mutation of captured var**, **non-local return** | **PENDING (U4)** |
| `functions` | functions | standalone functions per spec | **PENDING** |
| `errors` | error-handling, ADR-0008 | `throw`/`try`/`catch`, `Result` vs `Option`, bridges | **PENDING** |
| `system` | system | `System.print`, `System.gc`, IO surface | partial/PENDING |
| `concurrency` | concurrency | only if spec is concrete enough to pin expected output | **PENDING (far)** |
| `syntax-errors` | (front end) | missing paren, unclosed string, **trailing-newline/EOF edges** → clean diagnostic | NEGATIVE |
| `runtime-errors` | error-handling | message-not-understood, type errors, div-by-zero semantics → clean error | NEGATIVE |

Absorb & reorganize the existing flat `tests/lang/*.ph` into these label dirs (don't duplicate);
expand each to the coverage above.

## 5. Test file format (self-describing)
Each `.ph` starts with a header using the language's line-comment syntax (confirm from
`lexical-structure.md` / existing `tests/lang/comments_inline.ph`):
```
<comment> test: <one line — what behavior this pins>
<comment> spec: <file.md §> (+ ADR-#### if relevant)
<comment> status: PASS | PENDING(<unit>) | NEGATIVE
```
- `check_pass`/`check_negative` derive the label from the **directory**, so the header is for humans;
  keep it accurate.
- Sidecar `<case>.expected`: exact stdout for PASS; for NEGATIVE, the required diagnostic **substring**.
- Keep a top-level **`tests/lang/MANIFEST.md`**: every case → label · spec § · status · **ACTUAL
  current behavior** (PASS / cleanly-errors / **panics** / silently-wrong). Run everything against the
  live build and record the real baseline — the manifest is empirical, not aspirational.

## 6. Guardrails (write-set — heavy work is live on the spine)
- **Only touch**: `phalcom-core/tests/lang/**`, new `phalcom-core/tests/lang.rs`, new
  `phalcom-core/tests/support/mod.rs`, and (optionally) `scripts/test-lang.sh` wrapper.
- **Do NOT modify**: `phalcom-core/src/**`, `phalcom-ast/**`, `phalcom-core/tests/golden.rs`,
  `tests/invariants.rs`, `tests/fixtures/**`, `docs/spec/**`, `docs/adr/**`, **`Cargo.toml`** (any).
  Reading them is fine.
- If a test reveals a real VM bug (panic on valid input, wrong output), **do not fix it** — record it
  in the manifest and append a one-liner to `docs/forge/DEFERRED.md`.

## 7. Return (compact report)
Labels created + test count per label, the PASS/PENDING/NEGATIVE breakdown, how `cargo test --test
lang` and `./scripts/verify.sh` came out green, and 5–8 notable findings (spec features that **panic**
vs cleanly error today, any wrong-output surprises, spec ambiguities you had to resolve). No raw file
dumps. Do not self-approve harness correctness beyond the green gate.
