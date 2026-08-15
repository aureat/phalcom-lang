# Plan — Spec-conformance test corpus + selectors.md integration

**Status:** Ready to execute. Authored 2026-07-11.
**Goal:** (A) Integrate `docs/spec/selectors.md` into the spec suite, adopting **comma
canonical selector form** everywhere. (B) Build a **comprehensive, spec-driven test
corpus** covering every document in `docs/spec/`, where each case is either an active
**PASS** lane (feature works today) or a `pending/` **spec-target** (`#[ignore]`d until
implemented). Full spec coverage, build stays green.

**Two user decisions are baked in (do not revisit):**
1. Test scope = *full spec, pending-gated*.
2. Selector notation = *comma form wins* (`move(_,to,duration)`); ADR-0012 gets amended.

---

## Phase 0 — Consolidated discovery (READ FIRST; do not re-derive)

### 0.1 Ground truth: what runs today (empirically measured at green HEAD `037da3d`)

> ⚠️ **The current working tree does NOT compile** — the in-flight U4 blocks work has
> 6 compile errors (`FrameToken` import, `Callable.upvalues`, `CoreClasses.block_class`,
> non-exhaustive `Expr::Block` / `Bytecode::Closure|GetUpvalue|SetUpvalue|CloseUpvalue`).
> **All test authoring and `cargo test` runs must happen on a GREEN build.** Phase 1a
> pins this down. Do not write tests against the dirty tree.

**WORKS today (target as PASS lanes):**
- Arithmetic: `+ - * / %`, precedence, unary minus. All numbers are **f64** — `7/2`→`3.5`,
  `6/2`→`3` (whole values print without decimal), `1/0`→`inf`.
- Equality `==` / `!=`.
- Strings: literals, empty string, `+` concatenation.
- Booleans: `and` / `or` / `!` — **operands must already be booleans** (no coercion, no
  short-circuit of non-bool operands).
- `nil` literal (still surface-visible today — prints `nil`).
- `let` binding + reassignment.
- Classes: method defs, getters, arrow-getters (`pi => 3.14`), `static` methods,
  **arity-based** selector overloading (`m()`/`m(a)`/`m(a,b)` coexist), operator methods
  (`+(o)`, `==(o)`), instantiation, unary/positional sends, `.class` reflection.
- `System.print`; multi-statement via `;` and newline.

**NOT implemented (write as `pending/` spec targets):**
- Comparison `< > <= >=`; string interpolation `"{x}"`; string methods (`.length`).
- `var`; Option / `Some` / `None`.
- `construct` / `@construct` / `@get` / `@set`.
- Blocks / closures (`{ x }`, `{ |x| }`, `{ x => ... }`), non-local return.
- Control flow: `if` / `else` / `while` / `for` / `ifTrue:` / short-circuit coercion / inliner.
- Collections: list `[…]`, map `{…}`, `.each` / `.map` / `.filter`.
- Inheritance / `super`; `doesNotUnderstand` hook (miss is a hard error today);
  `perform` / `respondsTo`; rest `*p` / spread / `SEND_DYNAMIC`.
- **Labeled-argument method *definitions*** — call syntax `m(to: x)` parses to selector
  `m(to:)` but no surface syntax defines a matching method, so it always misses.
- `#` symbols, `::` families, `@` attributes; `throw`/`try`/`catch`/`finally`/`Result`;
  fibers/futures/scheduler; `System` clock/process beyond `print`.

**Two real bugs to pin with regression tests (PASS lane asserting current buggy behavior
is wrong → file as `pending/` that documents the intended behavior, plus a spawn-task):**
- **Setter param name hardcoded** — `age=(value)` works, `name=(v)` throws
  `Undefined variable 'v'` (`phalcom-ast/src/parser.rs:537-539`).
- **Keyword-arg call ≠ any definable method** — `move(to: a, at: b)` builds selector
  `move(to:at:)`; no def can match it.

### 0.2 Test harness — the machinery (copy these patterns, don't invent)

**`.ph` corpus (`phalcom-core/tests/lang/`)** — driven by `tests/support/mod.rs`:
- Each case = a `<name>.ph` **plus a sibling `<name>.expected`** (`with_extension("expected")`).
- `check_pass(label)` / `check_negative(label)` / `check_pending(label)` — the three lanes.
- **PASS/PENDING** → stdout must match `.expected` **exactly** (one trailing `\n` tolerated).
- **NEGATIVE** → process exits non-zero AND `stdout+stderr` **contains** the `.expected`
  string (substring, trimmed); no panic (exit ≠ 101, stderr has no `"panicked at"`).
- `collect_cases` is **single-level and panics if the dir is missing** → every label dir
  referenced by a test MUST exist on disk (even if empty it needs the dir).
- PENDING cases live in `lang/<label>/pending/`, still need `.expected`, and are gated by
  `#[ignore]` on the `#[test]` in `lang.rs`. Their `.expected` pins **current** behavior.
- Wire a label in `phalcom-core/tests/lang.rs`:
  - PASS: `#[test] fn foo() { support::check_pass("foo"); }`  (template: `lang.rs:13-16`)
  - NEG:  `check_negative("syntax-errors")`  (template: `lang.rs:49-57`)
  - PEND: `#[ignore = "reason"] … check_pending("foo")`  (template: `lang.rs:18-22`)

**Golden examples (`phalcom-core/tests/golden.rs`)** — `assert_golden(path, expected_stdout)`
with **inline** expected strings; covers `examples/*.ph` + `tests/fixtures/golden/*.ph`.
Exact stdout, no trailing-newline trimming. `person3.ph`/`simple.ph` are excluded
(unsupported `construct`/`@construct`).

**Object-model invariants (`phalcom-core/tests/invariants.rs`)** — in-process `VM` tests,
no subprocess. 10 tests over the metaclass tower + `verify_invariants`. None ignored.

**Lexer/parser snapshots (`phalcom-ast/tests/{lexer,parser}.rs`)** — **insta**.
`assert_debug_snapshot!(tokens(src))` / `assert_snapshot!(parse(src))`. Snapshots in
`phalcom-ast/tests/snapshots/<file>__<fn>.snap`. Regenerate with
`INSTA_UPDATE=always cargo test -p phalcom-ast`.

**Run commands:**
```sh
cargo test -p phalcom-core --test lang               # all lang labels
cargo test -p phalcom-core --test lang arithmetic    # one label
cargo test -p phalcom-core --test lang blocks -- --ignored   # a pending label
cargo test -p phalcom-core --test golden
cargo test -p phalcom-core --test invariants
cargo test -p phalcom-ast                            # lexer + parser snapshots
INSTA_UPDATE=always cargo test -p phalcom-ast        # bless snapshots
```

### 0.3 Anti-patterns (guards for every phase)

- ❌ Writing a `.ph` case without its `.expected` sibling → harness panics ("expected file
  missing"). ✅ Always create both.
- ❌ Referencing a new label dir that doesn't exist → `collect_cases` panics. ✅ `mkdir` the
  label dir (and `pending/` subdir) first.
- ❌ Putting a spec-target (unimplemented) case in an active lane → red build. ✅ Unimplemented
  → `pending/` + `#[ignore]`, with `.expected` pinning **today's actual** output/error.
- ❌ Asserting a feature works because the spec says so. ✅ Every PASS `.expected` is the
  **observed** output; run the file before committing the expectation.
- ❌ Assuming integer semantics. ✅ Numbers are f64; assert the f64 print form (`3`, `3.5`, `inf`).
- ❌ Hand-editing `.snap` files. ✅ Use `INSTA_UPDATE=always`.

---

## Phase 1 — Get to green + selectors.md integration (comma form)

### 1a. Establish a green build (BLOCKER for everything else)
- **Verify** `cargo build` on HEAD `037da3d` is green (subagent confirmed it is).
- The dirty working tree (U4 blocks WIP) is broken. **Decide with the user out-of-band**
  whether to (i) stash/branch the U4 WIP and author tests on green `037da3d`, or (ii) fix the
  6 U4 errors first. **This plan assumes (i)** unless told otherwise — tests target the green
  commit; U4 is separate work. Do NOT let this plan silently fix U4.
- **Verify checklist:** `cargo build` exits 0; `cargo test` (existing suites) is green.

### 1b. Adopt comma canonical selector form across the spec
The user chose **comma form** (`move(_,to,duration)`). Convert every colon-form occurrence.
Reference: subagent integration map located each site.
- `docs/adr/accepted/0012-selector-signature-encoding-and-dispatch.md` — **amend**: change canonical
  strings `add(_:_:)`, `move(to:duration:)`, `name=(_:)`, `+(_:)`, `sum(_...)` (lines ~13,34,54)
  to comma form (`add(_,_)`, `move(_,to,duration)`, `name=(_)`, `+(_)`, `sum(*)`); add an
  amendment note that comma form supersedes the original colon encoding.
- `docs/spec/messages-and-selectors.md` — rewrite the selector table (L15,27-32,34,44) and rest
  notation (`sum(*numbers)` stays; interned form `sum(_...)` → comma equivalent) to comma form.
- `docs/spec/README.md` — Invariant 2 (L37-38): `move(to:duration:)`/`move(_:_:)` → comma form.
- `docs/spec/object-model.md` — L66 symbol row `:name` and L278 selector example → comma form
  + `#`-prefixed symbol literal.
- `docs/spec/control-flow.md` — sacred-selector list (L37-39) → comma form.
- `docs/spec/functions.md` — L161 `#"+(_)"` example → comma form consistent with `#` literals.
- Fix selectors.md's own internal slip: L259 writes `ifTrue:ifFalse:` (colon) inside a comma doc.

### 1c. Weave selectors.md into the suite
- **Header** (selectors.md L1-6): rewrite to house style —
  `Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.` +
  `**Governing ADRs:** [ADR-0012](../../../adr/0012-…md)`. Drop the standalone `Supersedes:` line
  (ADR-0012 already owns that; it's now amended to comma form).
- **README reading order** (L12-27): add a row **between Messages & Selectors and Classes**:
  `[Selectors, Symbols & References](selectors.md) | Selector identity, # symbols, :: method references, @ attributes, field visibility`.
- **lexical-structure.md**: add sections for `#` symbol-literal lexing (selectors.md §2: Logos
  regex, shebang `#!`-at-offset-0, whitespace-adjacency ASI rule), `::` postfix token with
  `#`-lookahead, and `@` attribute token. Cross-link like the existing `?.`/`??` treatment.
- **object-model.md**: reconcile symbol-literal spelling to `#name` (was `:name`, L66); add the
  name-symbol vs selector-symbol distinction near the `Symbol` row.
- **messages-and-selectors.md**: fold in / cross-link selectors.md §1 (near-duplicate); add the
  `perform` name-symbol→type-error constraint (selectors.md L57) to §5.
- **method-lookup.md**: cross-link the base-name index (selectors.md §3.1) + family-candidate
  error enrichment from §2 DNU / §3 perform.
- **classes.md**: cross-link §1 `construct` / §3 accessors with selectors.md §4
  `@construct`/`@get`/`@set`; note whether decorators are sugar over the keyword or a
  replacement (mark "Planned — relationship TBD"). Cross-link the field-privacy statement
  (classes.md L50-52 ↔ selectors.md §5).
- **functions.md**: cross-link §3 `Method.bind`/`methodFor`/`invokeOn` with selectors.md §3
  `::` Family; flag "unify vs coexist" as an open question (see below).
- **open-questions.md**: add selectors.md §7 #3 (default arguments), #4 (Option bootstrap),
  #5 (Family introspection) as new questions. For §7 #1 (`var x`→None) and #2 (ifTrue/ifFalse→
  Option) — these **re-open items marked RESOLVED** (Q1→ADR-0014; values-and-absence §3). Flag
  explicitly in the doc: either drop them from selectors.md §7 or record the re-opening. **Leave
  the resolved decisions intact** unless the user says otherwise.
- **implementation-status.md**: fix selectors.md L248 "Parser (LALRPOP)" (LALRPOP is being
  removed per ADR-0016; parser is hand-written / `phalcom-ast`). Add gap rows for `#`/`::`/`@`/
  Family/base-name-index.
- **Verify checklist:** `grep -rn 'to:duration:\|add(_:_:)\|name=(_:)\|:name' docs/spec docs/adr`
  returns nothing (all comma form now); every `docs/spec/*.md` header matches house style;
  README reading-order has the Selectors row; no broken relative links (`grep` the `](../` and
  `](./` targets exist).

---

## Phase 2 — Lexical & literals corpus (`lexical-structure.md`, `values-and-absence.md`)

**PASS (active):** extend `lang/lexical/` — number literals (int-looking f64, float, negative),
string literal, empty string, `true`/`false`, `nil` prints, inline + full-line comments,
multi-statement separators. Add **lexer snapshot** tests in `phalcom-ast/tests/lexer.rs` for
each token class (numbers with `%`, operators `< > <= >=` as tokens even if unimpl in VM,
`#`/`::`/`@` sigils now that they're spec'd).
**PENDING (`lang/lexical/pending/`, `lang/absence/pending/` — create the `absence` dir):**
string interpolation `"{x}"`, escape sequences (exists), numeric separators `1_000_000`,
field-token distinction, tuple/list/map/set literals, `#` symbol literal eval, Option/`Some`/
`None`, `var x` defaults, `nil` becoming non-surface. `.expected` pins today's error/output.
- Un-ignore path: `lang.rs` already has `#[ignore] absence` — create `lang/absence/` +
  `pending/` so it doesn't panic; keep `#[ignore]`.
- **Verify:** `cargo test -p phalcom-core --test lang lexical` green; `-- --ignored` runs
  absence without panic; `cargo test -p phalcom-ast` snapshots blessed & committed.

## Phase 3 — Object model & method lookup (`object-model.md`, `method-lookup.md`)

**PASS:** extend `invariants.rs` — assert full core-class catalog wiring, `.class` for each
immediate (Number/String/Boolean/Nil/Symbol), superclass-chain lookup finds inherited methods.
Add `lang/dispatch/` PASS cases (currently empty dir) for arity-overload resolution and
operator-method dispatch. **PENDING (`lang/metaclass/`, `lang/dispatch/pending/` — create dirs):**
`doesNotUnderstand` hook + `Message` reification, `perform`, `respondsTo`, variadic table,
`super`. Un-ignore-safe the `metaclass` label (create dir).
- **Verify:** `invariants` green; `lang dispatch` green; `metaclass`/pending run under `--ignored`.

## Phase 4 — Messages, selectors, classes (`messages-and-selectors.md`, `classes.md`, `selectors.md`)

**PASS:** extend `lang/messages/` + `lang/classes/` — unary/positional sends, arity overloading,
operator methods, getters/arrow-getters, `static`, instantiation, field read via getter.
**Regression cases for the two bugs** as `pending/` (pin intended behavior): setter with
non-`value` param name; keyword-arg call matching a definition. **PENDING:** labeled-arg method
*definitions* (`move(to:, at:)`), rest `*p`, spread, `construct`, `@construct`/`@get`/`@set`,
inheritance, `#` selector symbols, `::` families. Update `golden.rs` once `construct` lands to
re-include `person3.ph`/`simple.ph`; for now leave excluded.
- **Anti-pattern guard:** do NOT add a labeled-def PASS case — it can't match today.
- **Verify:** `lang messages`, `lang classes`, `golden` green; pending run under `--ignored`.
- **Spawn follow-up tasks** for the two confirmed bugs (setter param, keyword-arg mismatch).

## Phase 5 — Blocks, functions, control flow (`blocks.md`, `functions.md`, `control-flow.md`)

Almost entirely **PENDING** (nothing here executes yet except `and`/`or`/`!` on bools).
**PASS:** `lang/control-flow/` (currently empty) — boolean `and`/`or`/`!` cases.
**PENDING (`lang/blocks/pending/` exists; create `functions`, and control-flow `pending/`):**
block literal + call, block-as-argument, non-local return, `Method.bind`/`methodFor`/`invokeOn`,
`if`/`else`/`while`/`for`, `ifTrue:`/`ifFalse:`, short-circuit coercion, the inliner. Un-ignore-safe
`functions` (create dir). `.expected` pins today's parse errors.
- **Verify:** `lang control-flow` green; `blocks`/`functions` run under `--ignored` without panic.

## Phase 6 — Errors, concurrency, system (`error-handling.md`, `concurrency.md`, `system.md`)

**PASS:** extend `lang/system/` — `System.print`, `System.class`; extend negative lanes
(`lang/runtime-errors/`, `lang/syntax-errors/`) with representative diagnostics. **PENDING
(create `lang/errors/`, `lang/concurrency/`, `lang/system/pending/`):** `throw`/`try`/`catch`/
`finally`/`Result`, fibers/futures/scheduler, `System` clock/process. Un-ignore-safe `errors`,
`system` (create dirs). Add a `concurrency` label + `#[ignore]` test in `lang.rs`.
- **Verify:** `lang system` + negative lanes green; `errors`/`concurrency` under `--ignored`.

## Phase 7 — Invariant coverage map (`README.md` §Invariants) + housekeeping

- Add one test (or documented case) per README invariant (1–6), mapping each to a concrete
  corpus case or invariant test; where an invariant isn't yet enforceable, a `pending/` target.
- **Refresh `phalcom-core/tests/lang/MANIFEST.md`** — it's stale (references
  `messages/send_system_new.ph`, wrong status counts). Regenerate the catalog to match the new
  on-disk tree; keep PASS/PENDING/NEGATIVE counts accurate.
- Ensure every `#[ignore]`d label in `lang.rs` has a real dir (no `collect_cases` panic under
  `--ignored`): `absence`, `metaclass`, `functions`, `errors`, `system`, `concurrency`.

---

## Final phase — Verification

1. **Green gate:** `cargo build` and `cargo test` (all suites, default) are green.
2. **Pending gate:** `cargo test -p phalcom-core --test lang -- --ignored` runs every pending
   label with **no panics** (dirs exist, `.expected` present) — pending cases may "fail" only
   in the sense of being ignored; when un-ignored they assert today's behavior and pass.
3. **Snapshot gate:** `cargo test -p phalcom-ast` green with all `.snap` committed.
4. **Coverage audit:** every `docs/spec/*.md` maps to at least one PASS case and/or one
   `pending/` spec-target. Produce a short coverage table (spec doc → suites → PASS/PENDING
   counts) at the top of the refreshed MANIFEST.
5. **No-invention audit:** `grep` PASS `.expected` files against the empirical matrix — no PASS
   case asserts an unimplemented feature.
6. **Doc audit:** `grep -rn 'to:duration:' docs/` empty; README reading-order includes Selectors;
   `cargo doc` still clean (no broken intra-doc links introduced).

---

## Sequencing notes for execution in fresh contexts

- **Phase 1a is a hard prerequisite** for Phases 2-7 (can't `cargo test` on a red tree).
- **Phase 1 (docs)** is independent of Phases 2-7 (tests target the *implementation*, not the
  spec notation) and can run in parallel by a separate agent.
- Phases 2-6 are largely independent per spec-area and can be sliced to separate agents; each
  must (a) create any missing label dir before wiring its `#[test]`, (b) run its own label green
  before handing off. Keep slices small (one or two spec docs each) per the subagent-handoff rule.
- Phase 7 + Final must run last (they audit the whole tree).
