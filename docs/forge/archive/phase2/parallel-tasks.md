# Parallel task briefs (3 independent agents)

These three units have **pairwise-disjoint write-sets** that are also disjoint from the two
active in-tree units (U-FE = `phalcom-ast`; the VM spine U1+ = `phalcom-core/src`). They can
run concurrently and safely. Each brief is self-contained — hand one to each agent verbatim.

## Shared rules (apply to all three)
- **Repo:** `/Users/altunhasanli/dev/phalcom/phalcom` (Rust workspace, edition 2024). Phalcom
  is a Smalltalk-style class-based language compiled to bytecode on a stack VM.
- **Isolation:** the main tree has heavy uncommitted work in progress on `phalcom-ast` and
  (soon) `phalcom-core/src`. Run your task on its own branch/worktree/clone, OR touch **only**
  the files in your write-set — never `phalcom-ast/**` or `phalcom-core/src/**` (except a
  crate's own file explicitly named below).
- **Documentation is mandatory** — the repo's standing rule (`docs/rust-documentation-guidelines.md`):
  `//!` on every crate/module, `///` on every public item (incl. fields, enum variants),
  verb-first summary lines, `# Errors`/`# Panics`/`# Safety` where they apply, intra-doc links.
- **Orientation:** `graphify-out/graph.json` exists — run `graphify explain/query` before raw
  file reads (there's a hook enforcing this).
- **Green gate:** `cargo build && cargo test && cargo clippy --workspace` clean, and
  `cargo doc --workspace --no-deps` must add **no new warnings from your crate's files**
  (pre-existing warnings in other crates — e.g. broken intra-doc links in
  `phalcom-core/src/module.rs` — are NOT yours; don't fix them, don't let them block you).
- **Return** a compact report: files changed, what you did, the green-gate tail. No file dumps.

---

## Brief A — `phalcom-common` (range only): documentation + hardening
**Write-set (HARD boundary):** `phalcom-common/src/range.rs` and `phalcom-common/src/lib.rs`
(only the `range` module's declaration/exports + crate doc). **Do NOT touch
`phalcom-common/src/refs.rs`** — the `PhRef` handle it defines is being rewritten by the VM
heap unit (U1, ADR-0009); documenting it now would collide with U1 and be discarded. Leave
`refs.rs` entirely alone. (Confirmed via `graphify affected "PhRef"`: `refs.rs` is on the
heap-rewrite's write-set.)

`range.rs` holds source spans/ranges — stable, used workspace-wide, safe to invest in.

**Task:**
1. Add a `//!` crate doc to `lib.rs` describing the crate's role; add a `//!` module doc to the
   `range` module. (Do NOT add crate-wide `#![warn(missing_docs)]` yet — it would fire on the
   deliberately-untouched `refs.rs`; instead put `#[warn(missing_docs)]` on the `range` module
   or just ensure every `range` item is documented.)
2. Document every public item in `range.rs` (structs, fields, enums, variants, fns, methods)
   per the standing rule — verb-first summaries, `# Panics` where indexing/slicing can panic,
   intra-doc links.
3. Add unit tests for the non-trivial span logic in `range.rs` (construction, merging,
   containment, ordering) — tests that would fail if the logic were wrong.
4. `cargo doc -p phalcom-common --no-deps` adds no new warnings from `range.rs`;
   `cargo test -p phalcom-common` green; clippy clean.

**Done when:** every `range.rs` public item documented, tests added and green, `cargo doc`
clean for the range module, `refs.rs` untouched.

---

## Brief B — `phalcom-repl`: warning cleanup + documentation + dedupe
**Write-set (HARD boundary):** `phalcom-repl/**` ONLY. Nothing else. (The REPL depends on
`phalcom-core`'s public API, which the VM redesign WILL change later — so do NOT try to fix
API mismatches by editing `phalcom-core`; if the REPL doesn't compile against current core,
report it and stop. Today it builds — your job is docs + warnings + structure, not API work.)

`phalcom-repl` is a `rustyline`/`reedline`-based REPL. Note: `src/*.rs` is the active editor
stack; a parallel `src/rustyline/` subdirectory is an alternate/experimental stack.

**Task:**
1. Resolve the ~12 existing compiler warnings (elided-lifetime `Cow<str>` → `Cow<'_, str>` in
   `main.rs`, and any unused imports / dead code) — cleanly, not with blanket `#[allow]`.
2. Determine whether the `src/rustyline/` experimental subdirectory is dead (nothing in the
   active `src/*.rs` path references it). If dead: remove it (or, if you're unsure, add a
   `//!` doc marking it experimental/unused and open a DEFERRED note rather than deleting).
   Confirm via `graphify` + `cargo build` which modules are actually reachable.
3. Add `//!` module docs and `///` docs for the public items of the active REPL modules
   (`repl.rs`, `editor.rs`, `helper.rs`, `completer.rs`, `highlighter.rs`, `common.rs`).
4. `cargo clippy -p phalcom-repl` clean (no warnings), `cargo doc -p phalcom-repl --no-deps`
   warning-free, build green.

**Done when:** zero warnings from `phalcom-repl`, experimental stack resolved (removed or
clearly documented), active modules documented, clippy + doc clean.

---

## Brief C — Spec ↔ decision consistency pass (docs only)
**Write-set (HARD boundary):** `docs/spec/**` ONLY. Do **not** touch `docs/adr/**` (another
unit is writing ADR-0016 + the ADR README there right now) and no code.

The spec in `docs/spec/` is the project's design source of truth, but several load-bearing
decisions were just ratified and recorded as ADRs 0007–0016. The spec's open-questions and
implementation-status docs are now partly stale.

**Task (read the relevant ADRs in `docs/adr/` for grounding — read-only — but edit only `docs/spec/`):**
1. `docs/spec/open-questions.md`: mark as RESOLVED the questions now decided, each with a
   pointer to the deciding ADR — at minimum: Q1 `let`/`var` → ADR-0014; absence-as-Option →
   ADR-0007; exceptions/Result → ADR-0008; heap/ownership → ADR-0009; Value repr → ADR-0010;
   instance `toString` → ADR-0015. Leave genuinely-open questions open.
2. `docs/spec/implementation-status.md`: update the "current state" narrative to reflect the
   redesign direction — the front end is being hand-written (LALRPOP removed, ADR-0016), the
   object graph moves to a handle/arena heap (ADR-0009), selectors gain label encoding
   (ADR-0012). Keep the "Recommended implementation order" but annotate which items are now
   in progress vs planned. Do NOT invent status you can't verify — frame as "planned per ADR".
3. Add cross-reference links from the relevant spec sections to their governing ADRs
   (object-model.md → 0002/0003/0009/0010; values-and-absence.md → 0007/0014; etc.).
4. Keep prose consistent with the existing spec voice; do not change any DECISION, only record
   what was decided and link it.

**Done when:** open-questions reflects the ratified decisions with ADR pointers,
implementation-status reflects the redesign, spec↔ADR cross-links added, no code touched.

---

## Brief D — CI/CD pipeline (GitHub Actions)
**Write-set (HARD boundary):** `.github/**` only. You MAY create `rustfmt.toml`/`clippy.toml`
*only if absent and needed for a check config* — but do **not** reformat or edit any Rust
source. No code, no docs outside `.github/`.

Set up continuous integration that enforces this repo's gates on every push/PR.

**Task:**
1. A workflow that runs the project's known-green gate — mirror `scripts/verify.sh`
   (`cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace`) as the
   **required** job. Pin a Rust toolchain, cache the cargo registry/target.
2. Add the **documentation gate** as a job: `cargo doc --workspace --no-deps` (the repo's
   standing rule requires docs — see `docs/rust-documentation-guidelines.md`). Since the tree
   isn't fully documented yet, make this job **advisory/`continue-on-error` for now** with a
   comment that it becomes required once crates adopt `#![warn(missing_docs)]`. Same for a
   `cargo fmt --check` job if you add one.
3. **Known gotcha (do not repeat it):** `Cargo.lock` is gitignored in this repo, so do **NOT**
   use `--locked` anywhere — it broke CI before.
4. Keep the required job green against the current tree. Note in a workflow comment that CI runs
   on committed state, so it only exercises the branch once it's committed/pushed.

**Done when:** workflow(s) exist under `.github/workflows/`, the required job mirrors
`scripts/verify.sh` without `--locked`, doc/fmt lanes are present (advisory), no source touched.

---

## Brief E — Fuzzing dictionary + fuzz docs (spec-derived)
**Write-set (HARD boundary):** `fuzz/phalcom.dict` and `fuzz/README.md` only. **Do NOT touch
`fuzz/fuzz_targets/**`** (they call the lexer/parser API, which is being rewritten right now) or
`fuzz/Cargo.toml`. The dictionary is just token strings — it depends on the *spec*, not the
implementation, so it stays valid across the front-end rewrite.

**Task (use the `fuzzing-dictionary` and `fuzzing-obstacles` skills):**
1. Build a comprehensive libFuzzer/AFL-style dictionary in `fuzz/phalcom.dict` of Phalcom
   surface tokens derived from `docs/spec/lexical-structure.md` + `messages-and-selectors.md`:
   keywords (`let`, `var`, `class`, `construct`, `super`, `true`/`false`, …), operators &
   punctuation, selector/label syntax (`foo:`, `move(to:duration:)`, `_`), literal forms
   (numbers incl. separators, strings incl. interpolation delimiters, tuple/list/map/set
   brackets), comment markers, and block/arrow syntax (`=>`, `{}`). Group and comment the dict
   by category. Include forms the spec defines even if not yet implemented — they harden the
   fuzzer for when they land.
2. Write/extend `fuzz/README.md`: how to run `cargo fuzz` with `-dict=phalcom.dict`, what each
   existing target covers (read the target names, don't edit them), and known obstacles
   (per `fuzzing-obstacles`).

**Done when:** a categorized, spec-grounded `phalcom.dict` exists and `fuzz/README.md` documents
running it; no fuzz target sources touched.

---

## Brief F — Top-level README + CONTRIBUTING guide
**Write-set (HARD boundary):** `README.md` (repo root — create or update) and `CONTRIBUTING.md`
(repo root — create) only. Do **not** touch `docs/spec/**`, `docs/adr/**`, `docs/forge/**`,
`CLAUDE.md`, or any source — link to them instead of duplicating.

**Task (read `CLAUDE.md`, `docs/spec/README.md`, `docs/rust-documentation-guidelines.md`,
`docs/adr/README.md`, `scripts/verify.sh` for accurate content):**
1. **`README.md`**: what Phalcom is (a Smalltalk-style class-based language compiled to bytecode
   on a stack VM, written in Rust); a quick start (build/run/REPL/test via `./scripts/verify.sh`);
   the workspace crate map (from `CLAUDE.md`); and pointers to the spec (`docs/spec/`), decisions
   (`docs/adr/`), and the documentation standard. State honestly that the implementation is
   under active redesign toward the spec.
2. **`CONTRIBUTING.md`**: the dev workflow, the green gate, the **mandatory documentation rule**
   (`docs/rust-documentation-guidelines.md`), the conventions from `CLAUDE.md`, how to add an
   ADR, and a one-paragraph overview of the `/forge` method (`.claude/skills/forge/SKILL.md`).
3. Keep it accurate and link-heavy; do not restate the spec or invent features.

**Done when:** a clear root `README.md` and `CONTRIBUTING.md` exist, accurate to the current
repo, linking (not duplicating) the spec/ADRs/doc-rule; nothing else touched.
