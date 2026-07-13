# U-LSP — in-process `phalcom-lsp` language server (supersedes subprocess-per-save)

Status: **PLANNED**. New Cargo workspace member `phalcom-lsp` (lib + bin) that
embeds `phalcom-ast` directly and serves LSP over stdio. Replaces, in staged and
flag-gated increments, the subprocess-per-save intelligence U-VSPHALCOM shipped
in `tools/vsphalcom`. **Floor: +0** — parse-only, no VM/`Value`/opcode/primitive
changes; does not link `phalcom-core`.

Grounded in **[ADR-0056](../../../adr/0056-phalcom-lsp-architecture.md)** (this
unit implements it). Successor to **[U-VSPHALCOM](../U-VSPHALCOM/plan.md)**,
whose "What must this not preclude" section named exactly this crate as its
intended endgame.

## Role

Closes the gap between what the front end already knows and what the editor
shows. Today (U-VSPHALCOM as-built):

- **Diagnostics** are single-error, save-only, one subprocess spawn per save
  (`tools/vsphalcom/src/diagnostics.ts` → `phalcom check <file> --format json`,
  and `cmd_check` at `phalcom-core/bin/phalcom/cli.rs:145` uses the
  **single-error** `parse_source`, printing one JSON object).
- **Completion/hover** are static, driven by the checked-in
  `src/generated/core-table.json`; they have **no view of the user's own
  classes**, no go-to-def, no find-refs, no receiver narrowing.
- **Coloring** is a regex TextMate grammar at the DEC-VSP-C approximation
  ceiling.

This unit stands up a warm, in-process server that reuses the real parser, and
grows it capability-by-capability.

## Spec / ADR anchors

- **[ADR-0056](../../../adr/0056-phalcom-lsp-architecture.md)** — the governing
  decision: `tower-lsp` framework; `phalcom-lsp` = new workspace member (lib +
  bin) depending on `phalcom-ast` + `phalcom-common` only (no `phalcom-core`);
  five staged capabilities; index keyed by (file URI, comma-form selector);
  server owns a UTF-16 `LineIndex`; first increment = Stage 1 diagnostics behind
  `phalcom.lsp.enabled`, alongside the subprocess path.
- **[ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md)** —
  selectors are canonical comma-form label-encoded symbols; `foo`, `foo(_)`,
  `move(_,to,duration)` are **distinct**. The symbol-index key, go-to-def target,
  find-refs set, completion entry, and hover key MUST be the comma-form selector,
  **never** a bare name. A bare-name key is wrong-by-construction (same gate
  U-VSPHALCOM enforces on its table).
- **[ADR-0016](../../../adr/0016-hand-written-lexer-and-recursive-descent-parser.md)** —
  `phalcom-ast` is the single, standalone, VM-free front end the server embeds;
  no second grammar in TypeScript.
- **[ADR-0025](../../../adr/0025-external-internal-parameter-names.md)** — the
  external labels the completion snippets and hover signatures render.
- **[ADR-0027](../../../adr/0027-modules-as-files-with-public-by-default-imports.md)** —
  module = file: the workspace index's per-file granularity and warm-scan unit.
- `docs/spec/v0.2/lexical-structure.md` §§9–14 — the token surface Stage 5
  (`semanticTokens`) classifies from the real lexer, mooting DEC-VSP-C.
- `docs/spec/v0.2/experimental/doc-comments-phaldoc.md` §§3–5,8 — Phaldoc hover
  sources (`///` outer / `//!` inner, adjacency rule §5, selector-keyed §4,
  contract-view harvest order §8). STATUS experimental; `//`-prefixed so
  lexically inert trivia — a raw-source harvest needs no compiler change.

## Preconditions (confirmed this session — do not re-derive)

- **`phalcom-ast::parser::parse(source, offset) -> Parse { program, errors }`
  already exists and is public** (`phalcom-ast/src/parser.rs:97`). It is the
  full-recovery, **multi-error** entry: `errors: Vec<SyntaxError>`, `program`
  holds every statement that parsed. **No new `phalcom-ast` entry point is
  needed for Stage 1.** `parse_source` (`:82`) is the single-error wrapper the
  CLI/subprocess uses — the reason today's diagnostics are single-error.
- **`SyntaxError { kind: SyntaxErrorKind, range: Range<usize> }`**
  (`phalcom-ast/src/error.rs:27`). `kind.to_string()` is the message; `range` is
  a **byte** half-open span. Zero-width range = point (e.g. EOF).
- **Positions need new mapping code.** `cmd_check`'s `byte_offset_to_line_col`
  (`cli.rs:177`) is **`char`-based and 1-based** and lives in the bin — **not
  reusable** for LSP (which needs 0-based line + **UTF-16** code-unit column).
  The server owns a `LineIndex`.
- **Embedding `phalcom-ast` in a Rust tool is precedent.**
  `phalcom-core/bin/gen-core-table/main.rs` links `phalcom-ast`, walks a
  `Program` for class/selector harvest (`harvest_primitives_rs()` at `:137`),
  and emits `core-table.json`. The server is the same embedding, warm.
- **`core-table.json` is a ready data artifact** for builtin completion/hover
  (`tools/vsphalcom/src/generated/core-table.json`), already ADR-0012-comma-form
  keyed. The server reads it; it does **not** relink `phalcom-core`.
- **The client already has a config-flag pattern** (`phalcom.executablePath`,
  `package.json` `contributes.configuration`) — `phalcom.lsp.enabled` slots in
  beside it.
- **`Cargo.toml` `members`** = `phalcom-ast`, `phalcom-common`, `phalcom-core`,
  `phalcom-repl`. `phalcom-lsp` is appended.

## Design

New crate `phalcom-lsp/` at the workspace root, **lib + bin**:

- **lib** — `src/lib.rs`, `src/backend.rs` (the `tower_lsp::LanguageServer`
  impl), `src/documents.rs` (open-document store: text + cached `Parse` +
  `LineIndex`), `src/line_index.rs` (byte↔UTF-16 position mapping),
  `src/diagnostics.rs` (`SyntaxError` → `Diagnostic`), `src/index.rs` (workspace
  symbol index), `src/selectors.rs` (comma-form reconstruction from AST decls,
  ADR-0012), `src/core_table.rs` (load `core-table.json`).
- **bin** — `src/main.rs`: thin stdio wiring (`tower_lsp::Server::new(stdin,
  stdout, socket)` → `.serve()`).

Deps: `phalcom-ast`, `phalcom-common`, `tower-lsp` (+ transitive `lsp-types`),
`tokio`, `serde`/`serde_json`, a concurrent doc map (`dashmap` or
`tokio::sync::RwLock<HashMap>`). **Not** `phalcom-core`.

### Stage boundaries (each independently verifiable green)

Ordered per ADR-0056 §3; each stage is a separately-committable checkpoint.

**Stage 1 — live multi-error diagnostics (the first increment).**
`initialize` (advertise `textDocumentSync=Full`, `positionEncoding`
negotiation), `initialized`, `shutdown`. `did_open`/`did_change`/`did_close`
maintain the document store. On change (debounced), `phalcom_ast::parse(text,
0)` → map **every** `SyntaxError` in `Parse.errors` via the `LineIndex` →
`publish_diagnostics`. Nothing else. This is the whole first ship.

**Stage 2 — workspace symbol index → go-to-def + find-refs.** On `initialize`,
scan root(s) for `.ph`, parse each once, build the index (ADR-0056 §4):
definitions from `ClassDef → ClassMember` decls keyed `(file_uri, selector)`;
references from send expressions, reverse-keyed `selector → [(file_uri,
range)]`. `textDocument/definition`, `textDocument/references`,
`workspace/symbol`. On `did_change`, reparse the one file, replace its index
slice wholesale.

**Stage 3 — receiver-aware completion.** `textDocument/completion`. A **pluggable
receiver resolver** whose first implementation is light local dataflow: within a
method/block scope, bind `var x = Cls.construct(...)` (and `Cls.new(...)`) ⇒
`x : Cls`; on `x.` offer `Cls`'s comma-form selectors (index for user classes,
`core-table.json` for builtins), rendered as label snippets
(`move(${1:_}, to: ${2:_}, duration: ${3:_})`). Unknown receiver ⇒ degrade to
the core-table set. Resolver is an interface, not a hardcoded rule (forward-gate
for future inference).

**Stage 4 — server-side hover.** `textDocument/hover`: the cross-file port of
`hover.ts`'s three sources — (a) keyword blurbs (closed static map), (b)
comma-form selector signature + kind + defining class (index for user symbols,
`core-table.json` for builtins), (c) Phaldoc `///`/`//!` harvest
(`doc-comments-phaldoc.md` §§3–5) keyed by selector (§4). Keep the named
`renderContractView(selector)` seam (returns nothing; gated on U-ANNOT-CONTRACTS,
harvest order §8) so the contract layer drops in without reshaping the pipeline.

**Stage 5 — semantic tokens.** `textDocument/semanticTokens/full` from the
`phalcom-ast` token stream. Exact `#`-adjacency and `\(expr)` classification —
**moots DEC-VSP-C**. TextMate grammar demoted to the `lsp.enabled=false`
fallback.

### Client migration (in `tools/vsphalcom`, sequenced after each stage proves)

`extension.ts` gains a `phalcom.lsp.enabled` branch: when on, launch the
`vscode-languageclient` shim spawning `phalcom-lsp` over stdio; when off, keep
today's TS providers. As each stage proves green, delete its TS provider:
Stage 1 → `diagnostics.ts`; Stages 3–4 → `completions.ts` + `hover.ts` +
`context.ts`; Stage 5 → grammar demoted. End state: `extension.ts` ≈ 30-line
LanguageClient launcher; ~800 lines of TS intelligence retired.

## Write-set (STOP-and-report if outside)

- `Cargo.toml` (root) — **only** append `phalcom-lsp` to `members`.
- `phalcom-lsp/**` (**new crate**) — all server code (`Cargo.toml`, `src/*`,
  `tests/*`).
- `tools/vsphalcom/src/extension.ts` — add the `phalcom.lsp.enabled` launch
  branch (LanguageClient shim); per-stage deletion of superseded providers.
- `tools/vsphalcom/src/diagnostics.ts`, `completions.ts`, `hover.ts`,
  `context.ts` — **removed** as their stage supersedes them (Stage 1 / 3 / 4).
- `tools/vsphalcom/package.json` — **only** `contributes.configuration`
  (`phalcom.lsp.enabled`), `dependencies` (`vscode-languageclient`),
  `activationEvents`.
- **Floor: +0.** No `phalcom-*` runtime crate touched. `phalcom-ast`/`common`
  are consumed as-is (`parse` already public). `phalcom-core` is **not** linked.
- **Sequencing vs live worktrees:** `package.json` and `extension.ts` are
  single-writer; sequence any concurrent `tools/vsphalcom` edit, don't parallel
  it ([[phalcom-concurrent-session-hazards]]).

## Build order

1. **Crate skeleton + Stage 1.** Add member; `phalcom-lsp` with `initialize`/
   `did_change` → `parse` → multi-error `publish_diagnostics`; `LineIndex` with
   its own unit tests **first** (positions are the correctness hotspot). Wire the
   `vscode-languageclient` shim behind `phalcom.lsp.enabled` (default off),
   alongside the existing subprocess. Commit ([[commit-frequently]]).
2. **Stage 2** — workspace scan + index + definition/references/symbol; per-file
   reparse-and-replace on change. Commit; delete nothing yet.
3. **Stage 3** — completion + receiver resolver; delete `completions.ts` +
   `context.ts` once green. Commit.
4. **Stage 4** — hover (keyword/selector/Phaldoc + contract stub); delete
   `hover.ts` once green. Commit.
5. **Stage 5** — semantic tokens; demote grammar to fallback. Commit.
6. Only once Stage 1 diagnostics are proven: flip `phalcom.lsp.enabled` default
   and delete `diagnostics.ts`. Commit.

## Tests / verification

An LSP server is tested at three levels; the split matters because a live stdio
round-trip is slow and flaky for logic that can be tested directly.

- **Pure logic (fast, `cargo test` in `phalcom-lsp/tests/`), no transport:**
  - `LineIndex`: byte offset → (line, UTF-16 char) over ASCII, multibyte
    (`é`, emoji, CRLF) and EOF/zero-width fixtures. This is the standing hotspot
    — table-driven, exhaustive.
  - `SyntaxError → Diagnostic` mapping: a fixture with **N** syntax errors
    produces **N** diagnostics (the multi-error win over the single-error
    subprocess), each at the correct mapped range; a clean file → zero.
  - Selector reconstruction (`selectors.rs`): AST decls → comma-form must be
    ADR-0012-correct (`move(_,to,duration)`, not `move`); getter/setter/method/
    construct kinds distinct; a lint asserting no bare-name key aliases a
    label-bearing selector (the ADR-0012 gate, mirroring U-VSPHALCOM's).
  - Index build over a small multi-file fixture: definitions and references land
    at the right (file, selector, range); a `did_change` reparse replaces only
    the changed file's slice (other files' entries byte-identical).
- **Server integration (golden JSON-RPC over an in-memory duplex, per stage):**
  drive `backend` through `tower-lsp`'s `LspService` with an in-process
  transport (`tokio::io::duplex`) — no real subprocess. Send an
  `initialize`→`initialized`→`didOpen`→`didChange` sequence and assert the
  emitted `publish_diagnostics` notification matches a golden (Stage 1); likewise
  a `definition`/`references`/`completion`/`hover`/`semanticTokens` request →
  golden response per later stage. Golden request/response fixtures checked into
  `phalcom-lsp/tests/fixtures/`, regen-able.
- **Determinism:** re-running the workspace scan on an unchanged tree yields a
  byte-identical index snapshot (guards non-deterministic map ordering in
  `workspace/symbol`).
- **Client:** manual dev-host checklist (`phalcom.lsp.enabled=true` → live
  multi-error squiggles on type, go-to-def jumps, receiver completion, hover,
  semantic colors); the flag=false path still shows the subprocess/grammar
  behavior unchanged (proves additive, non-destructive cutover).

## Decisions to flag (DEC-LSP)

- **DEC-LSP-A — framework.** `tower-lsp` vs raw `lsp-server`+`lsp-types`.
  **Decided in ADR-0056 §1: `tower-lsp`** (typed staged handlers + stdio +
  lifecycle; tokio cost immaterial for a parse-only server). `lsp-server` kept
  as the documented fallback. Not re-opened here — recorded so no one relitigates
  at implementation time. Sub-detail left to the implementer: `tower-lsp` vs the
  `tower-lsp-server` community fork (same `LanguageServer` surface); pin the
  version.
- **DEC-LSP-B — builtin table source.** Read the checked-in `core-table.json`
  (produced by `gen-core-table`) as a data artifact vs relink `phalcom-core` for
  a live harvest. **Decided in ADR-0056 §2: read the artifact** — keeps the
  server VM-free and instant-start. Live harvest, if ever wanted, factors
  `gen-core-table`'s harvest into a small lib depended on by both; **never**
  depend on `phalcom-core` from `phalcom-lsp`. Not blocking.
- **DEC-LSP-C — position encoding.** Advertise/negotiate `positionEncoding` at
  `initialize`, default **UTF-16** (LSP default). The `LineIndex` column unit is
  the one thing a future UTF-8 client flips. Decided (ADR-0056 §5 Consequences);
  recorded, not blocking.

**No BLOCKED-ON-DECISION items.** Everything U-LSP needs already exists in-tree
(`parse` is public; `core-table.json` is generated; the config-flag pattern is
established). Unlike U-VSPHALCOM's DEC-VSP-A, this unit adds **no** upstream
compiler-core dependency.

## What must this not preclude (P4)

- **Future incremental (sub-file) parsing.** Index consumers treat a file's
  entries as an opaque, wholesale-replaceable slice; nothing assumes a full-file
  reparse. When `phalcom-ast` later grows a range/subtree reparse entry, only
  `index.rs`'s update step changes. **No new `phalcom-ast` entry is needed now** —
  the future one is additive, not a precondition.
- **Future multi-root workspaces.** Index key is the **absolute file URI**; the
  scan iterates **all** roots. No single-root or single-module assumption.
- **Future cross-file type inference.** Stage 3's receiver narrowing is a
  **pluggable resolver interface** (`construct`-site dataflow is just the first
  impl); the index already carries class + selector + hierarchy edges for a
  later inference pass to consume unchanged. Completion degrades to the
  core-table set on "unknown" — never blocks on inference.
- **The subprocess/grammar fallback.** Stage 1 is flag-gated and additive, so
  `phalcom check` and the TextMate grammar stay the zero-config default until each
  stage supersedes them; cutover is reversible per-capability via
  `phalcom.lsp.enabled`.
- **U-ANNOT-CONTRACTS contract-view hover.** Stage 4 keeps the named
  `renderContractView(selector)` seam (inert today) so the contract layer
  (`doc-comments-phaldoc.md` §8 harvest order) drops in without restructuring the
  hover pipeline.

## Return shape (implementer)

commit SHA(s) · **crate:** `phalcom-lsp` added to `Cargo.toml members`, builds
(`cargo build -p phalcom-lsp`), `cargo test -p phalcom-lsp` green · **Stage 1:**
`LineIndex` unit tests (list multibyte/CRLF/EOF cases), N-errors→N-diagnostics
proven, integration golden for `didChange`→`publishDiagnostics`, flag `phalcom.
lsp.enabled` present + default off, subprocess path untouched · **later stages
(if reached):** per-stage golden fixtures + the TS provider each one deleted ·
ADR-0012 comma-form gate asserted in selector tests · `phalcom-core` confirmed
**not** in the dependency graph (`cargo tree -p phalcom-lsp` shows no
`phalcom-core`) · `tsc` clean on the client shim · floor delta (exp 0) ·
write-set confirm.
