# Brief: write a preliminary Phalcom-language test corpus (edge-case, spec-anchored)

## Role & context
You are writing test programs **in the Phalcom language** — a Smalltalk-style, class-based
language compiled to bytecode and run on a stack VM (Rust workspace at
`/Users/altunhasanli/dev/phalcom/phalcom`). These are `.ph` source programs that exercise the
language **end-to-end through the CLI**, each paired with its expected result. They form a
PRELIMINARY acceptance corpus to verify the implementation as it is built out.

Aim for **balanced comprehensiveness**: broad coverage of the important language surface with
high-signal **edge cases** and **negative cases** — NOT hundreds of exhaustive permutations. A
handful of sharp tests per area.

## Source of truth — read before writing anything
- `docs/spec/` is the authority on syntax + semantics. Read at least: `README.md`,
  `lexical-structure.md`, `values-and-absence.md`, `messages-and-selectors.md`,
  `method-lookup.md`, `control-flow.md`, `classes.md`, `blocks.md`, `object-model.md`.
  **Base every test's EXPECTED behavior on the spec — do not invent syntax or semantics.**
- `docs/spec/implementation-status.md` — tells you what is ALREADY implemented vs greenfield.
  **Critical:** the current tree is a partial Wren-style VM; much of the spec is not built yet,
  so many spec-correct programs will fail today. That is expected — label them (see STATUS).
- Calibrate current syntax by running the working examples: `examples/core_new.ph`,
  `examples/person2.ph`, `examples/person.ph`, `examples/calculator.ph`.

## Orientation
`graphify-out/graph.json` exists — you may `graphify query`/`explain` to understand structure,
but your real references are the spec docs and **empirical CLI runs**.

## How to run a test
- Build once: `cargo build`. Run a file: `./target/debug/phalcom <file.ph>`
  (inline snippet: `./target/debug/phalcom -i '<code>'`).
- Capture **stdout AND exit code**. A well-formed program exits 0; a program that SHOULD error
  must produce a clean diagnostic and exit **non-zero — never a Rust panic** (a panic is itself
  a bug worth noting).
- Keep outputs **deterministic** — no timestamps, pointer addresses, or unordered iteration.
- Note: the front end is being rewritten; some trailing-newline / edge-syntax handling may be
  rough. Check empirically and note anything surprising.

## What to cover (balanced — a few edge/negative cases each)
1. **Lexical / literals**: int & float numbers (zero, negative, large, fractional, precision
   boundary), strings (empty, escapes, special chars), comments, blank lines, empty program.
2. **Arithmetic**: operator precedence & associativity, division by zero, mixed float/int,
   unary minus.
3. **Booleans & logic**: `true`/`false`, `and`/`or` short-circuit (observe side-effect order),
   `not`, comparisons.
4. **Absence**: whatever the spec defines absence as (`values-and-absence.md`).
5. **Variables & scope**: `let` (and `var` if spec'd), shadowing, **read-before-write** (a
   compile error per `classes.md`), nested scope.
6. **Message sends & dispatch** (`messages-and-selectors.md`, `method-lookup.md`): unary /
   binary / keyword sends, selector arity + labels, dispatch to the right method, inheritance &
   override, `super`, and **doesNotUnderstand** on an unknown selector (negative).
7. **Classes** (`classes.md`, `object-model.md`): definition, fields (private/default), methods,
   getters vs methods, setters, static/class-side methods, `construct`/initialization, instance
   identity, and reflection (`name`, `toString`, `class`).
8. **Control flow** (`control-flow.md`): `if`/`else`, loops, conditionals-as-messages if that
   is the model, boundary conditions (empty body, false branch).
9. **Blocks / closures** (`blocks.md`) if in scope: block literals, arguments, capture, mutation
   of captured vars, non-local return — edge cases around capture.
10. **Errors / edge**: syntax errors (clean diagnostic, not panic), runtime errors (message not
    understood, type errors, div-by-zero), deeply nested expressions.

## Test format (each test is self-describing)
- One `.ph` file per test, named `area_case.ph` (e.g. `arithmetic_div_by_zero.ph`).
- A **header comment** in the file: what it tests, the spec § it anchors to, and its STATUS.
- The **expected result**: a sidecar `area_case.expected` (exact stdout) OR, for negative tests,
  a note that it must exit non-zero with a diagnostic (and the expected message substring).
- A **`MANIFEST.md`** listing every test with: area, spec §, and STATUS ∈
  - **PASS** — works on the current build (regression guard),
  - **PENDING** — valid spec target, not yet implemented → expected to fail now, should pass once
    the feature lands,
  - **NEGATIVE** — must produce a clean error, not a crash.
  **Run every test against the current build and record ACTUAL behavior** in the manifest — for
  PENDING/NEGATIVE cases, say whether it currently panics, cleanly errors, or is silently wrong.
  The manifest must reflect the real baseline, not aspiration.

## Write-set (HARD boundary — heavy work is in progress elsewhere in the repo)
- Put **everything in a NEW directory `phalcom-core/tests/lang/`** (create it): the `.ph`
  programs, their `.expected` sidecars, and `MANIFEST.md`.
- **Do NOT modify:** `phalcom-core/tests/golden.rs` or `tests/fixtures/golden/` (another agent
  owns it), `phalcom-ast/**`, `phalcom-core/src/**`, `docs/adr/**`, `docs/spec/**` (read-only),
  root `Cargo.toml`. Reading any of these is fine.
- **Do NOT wire the corpus into a Rust test harness** — just produce the files + manifest;
  harness integration happens later.

## Return (compact report)
The directory created, count of tests per area, the PASS/PENDING/NEGATIVE breakdown, and 3–5
notable findings (spec features that panic vs cleanly error today, any current wrong-output
surprises). No raw file dumps.
