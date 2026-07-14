# 56. Phalcom language intelligence is an in-process `phalcom-lsp` server

- Status: Proposed
- Date: 2026-07-13
- Related: [ADR-0012](../accepted/0012-selector-signature-encoding-and-dispatch.md)
  (selectors are comma-form label-encoded symbols — the symbol-index key);
  [ADR-0016](../accepted/0016-hand-written-lexer-and-recursive-descent-parser.md)
  (`phalcom-ast` is the standalone, VM-free front end this server embeds);
  [ADR-0025](../accepted/0025-external-internal-parameter-names.md) (external labels the
  completion snippets render); [ADR-0027](../retired/0027-modules-as-files-with-public-by-default-imports.md)
  (module = file — the workspace-index granularity); `docs/spec/v0.2/lexical-structure.md`
  §§9–14 (token surface for semanticTokens); `docs/spec/v0.2/experimental/doc-comments-phaldoc.md`
  §§3–5,8 (Phaldoc hover sources); `docs/forge/units/U-VSPHALCOM/plan.md`
  (DEC-VSP-A subprocess `phalcom check`, DEC-VSP-B harvested `core-table.json`,
  DEC-VSP-C grammar-approximation ceiling — all superseded/mooted here).

> **Provisional number.** `0056` was the next free slot at authoring time on a
> tree with live concurrent sessions (files exist through `0055`). If a
> concurrent ADR claims `0056`, renumber this one — no cross-file index is
> edited by this ADR.

## Context

Editor intelligence for Phalcom today (`tools/vsphalcom`, unit U-VSPHALCOM)
is a **subprocess-per-save** design:

- `src/diagnostics.ts` shells `phalcom check <file> --format json` on
  `onDidSaveTextDocument` / `onDidOpenTextDocument`. `cmd_check`
  (`phalcom-core/bin/phalcom/cli.rs:145`) calls **`phalcom_ast::parse_source`**
  — the *single-error* entry — and prints **one** JSON diagnostic. So the user
  sees at most one error, only on save, after a process spawn.
- `src/completions.ts` (219 lines) and `src/hover.ts` (344 lines) are driven by
  a checked-in `src/generated/core-table.json`, harvested build-time by
  `phalcom-core/bin/gen-core-table` (which itself embeds `phalcom-ast`). They
  are static: no view of the user's own classes, no go-to-definition, no
  find-references, no receiver narrowing — completions are the closed core-class
  set only.
- Coloring is a regex TextMate grammar whose accuracy ceiling is *accepted as a
  known approximation* under DEC-VSP-C (the `#`-adjacency and `\(expr)`
  interpolation rules are only approximated).

Three structural facts about the current tree make an in-process server the
right next step rather than more TypeScript:

1. **`phalcom-ast` is a standalone, VM-free lib crate.** Its public API already
   exposes **`phalcom_ast::parser::parse(source, offset) -> Parse { program,
   errors }`** — the *full-recovery, multi-error* entry
   (`phalcom-ast/src/parser.rs:97`) — alongside the single-error `parse_source`.
   Multi-error diagnostics need **no new `phalcom-ast` entry point**; the
   subprocess path simply calls the wrong one.
2. **Embedding `phalcom-ast` in a Rust tool is already precedent.**
   `phalcom-core/bin/gen-core-table` links `phalcom-ast` and walks a `Program`
   for class/selector harvest. A long-running server is the same embedding with
   a warm cache instead of a one-shot.
3. **Selectors are comma-form label-encoded symbols** ([ADR-0012](../accepted/0012-selector-signature-encoding-and-dispatch.md)):
   `foo`, `foo(_)`, `move(_,to,duration)` are **distinct**. Any symbol index,
   go-to-def target, or reference set must key by the comma-form selector, never
   a bare name — the same gate U-VSPHALCOM already enforces for its table.

The U-VSPHALCOM plan explicitly names this server as its intended successor:
"the harvest logic … is exactly what an LSP server would internalize … Keep the
harvested table format selector-keyed and source-tagged so an LSP can supersede
the subprocess without reshaping the data."

## Decision

Stand up a new **`phalcom-lsp`** crate: an in-process Language Server Protocol
server that embeds `phalcom-ast` directly (no subprocess, no CLI shelling), and
grow it in five capability stages behind a `phalcom.lsp.enabled` flag so it runs
**alongside** the subprocess path until each capability is proven, then
supersedes it.

### 1. Server framework: `tower-lsp`

Build on **`tower-lsp`** (the `LanguageServer` async trait + JSON-RPC framing +
document-sync lifecycle), not raw `lsp-server` + `lsp-types`.

- `tower-lsp` supplies typed handlers for exactly our staged capabilities
  (`did_change`, `publish_diagnostics`, `goto_definition`, `references`,
  `completion`, `hover`, `semantic_tokens_full`) and owns the stdio transport,
  `initialize`/`shutdown` handshake, and cancellation — the boilerplate we would
  otherwise reimplement on `lsp-server`.
- It pulls `tokio`; the parse is CPU-bound and fast (one file, recursive
  descent), so handlers parse inline. If a workspace scan ever measures as a
  stall, move it to `spawn_blocking` — a local change, not a redesign.
- `lsp-types` still appears transitively (tower-lsp re-exports it) so the wire
  types are the same either way.

Pin the version and treat the `tower-lsp` vs `tower-lsp-server` (community fork)
choice as a maintenance detail resolvable at implementation time; the crate's
public surface is the `LanguageServer` trait either way.

### 2. Crate boundary and embedding

`phalcom-lsp` is a **new Cargo workspace member** at the repo root
(`phalcom-lsp/`), added to `Cargo.toml` `members`. It is **lib + bin**:

- **lib** (`src/lib.rs`, `src/backend.rs`, `src/index.rs`, `src/line_index.rs`,
  `src/diagnostics.rs`, …) — the server logic, unit- and integration-testable
  without a live stdio pipe.
- **bin** (`src/bin/phalcom-lsp.rs` or `src/main.rs`) — a thin `main` that wires
  `tower_lsp::Server::new(stdin, stdout, …)` and serves over stdio.

**Dependency graph is deliberately tight and VM-free:**

- `phalcom-lsp` → **`phalcom-ast`** (`parse`, `parse_source`, `ast::*`,
  `error::{SyntaxError, SyntaxErrorKind}`) and **`phalcom-common`**
  (`SourceRange`). Plus `tower-lsp`, `tokio`, `serde`/`serde_json`, and a
  concurrent document map (`dashmap` or a `tokio::sync::RwLock<HashMap>`).
- `phalcom-lsp` does **not** depend on `phalcom-core`. Diagnostics, the symbol
  index, completion, and hover are all parse-only; no VM, compiler, or heap is
  linked. This keeps the server small and its startup instant.
- The **core/native selector table** (builtin completions + builtin hover) is
  the existing checked-in `core-table.json` produced by
  `phalcom-core/bin/gen-core-table` — the server reads it as a data artifact.
  It is **not** re-derived by linking `phalcom-core`. If live primitive harvest
  is ever wanted, factor `gen-core-table`'s harvest into a small helper lib and
  depend on *that*, never on the VM crate.

Reused as-is: `phalcom-ast::parse` (multi-error recovery), the `SyntaxError`
`{ kind, range: Range<usize> }` shape, the `Program`/`ClassDef`/`ClassMember`
AST, and `core-table.json`. **New** code: the server itself, the workspace
symbol index, and a per-document **`LineIndex`** (see §4).

### 3. What supersedes what, in five stages (each flag-gated, additive)

- **Stage 1 — diagnostics.** `textDocument/didChange` → in-process
  `phalcom_ast::parse(text, 0)` → map every `SyntaxError` in `Parse.errors` to
  an LSP `Diagnostic` → `publish_diagnostics`. This is **multi-error and live**
  (per keystroke, debounced), superseding the single-error, save-only,
  subprocess path.
- **Stage 2 — workspace symbol index → go-to-definition + find-references**
  (`textDocument/definition`, `textDocument/references`, `workspace/symbol`).
- **Stage 3 — receiver-aware completion** (`textDocument/completion`) with light
  local dataflow: track a variable's class from its `construct` call site
  (`var p = Point.construct(...)` ⇒ `p : Point`), then offer that class's
  comma-form selectors. Falls back to the core-table set when the receiver is
  unknown.
- **Stage 4 — server-side hover** (`textDocument/hover`): the cross-file version
  of today's `hover.ts` sources — keyword blurbs, comma-form selector signature
  (from the index for user classes, `core-table.json` for builtins), and the
  Phaldoc `///`/`//!` harvest (`doc-comments-phaldoc.md` §§3–5), keyed by
  selector (§4).
- **Stage 5 — semantic tokens** (`textDocument/semanticTokens/full`): real
  token classification from the `phalcom-ast` token stream, superseding the
  regex TextMate grammar. This **moots DEC-VSP-C** — the `#`-adjacency and
  `\(expr)` rules become exact because the same lexer the compiler uses drives
  the coloring. (The TextMate grammar stays as the zero-server fallback for when
  `phalcom.lsp.enabled` is off.)

### 4. Workspace symbol index shape (ADR-0012-keyed)

The index is keyed by **(file URI, comma-form selector)** — the ADR-0012 gate.

- **Definitions:** for each `.ph` file's `Parse.program`, walk `ClassDef →
  ClassMember` decls. Each method/getter/setter/`construct` yields an entry
  `{ key: (file_uri, selector), class, kind, name_range, full_range }` where
  `selector` is reconstructed comma-form (name + external labels per
  [ADR-0025](../accepted/0025-external-internal-parameter-names.md); `Getter`→`name`,
  `Method`→`name(_,label,…)`, `Setter`→`name=(_)`, `construct`→`name(…)`).
- **References:** walk send expressions; each dotted/operator/keyword send yields
  a reverse entry `selector → [(file_uri, call_range)]`.
- **Warmth & incrementality:** on `initialize`, scan the workspace root(s) for
  `.ph` files and parse each once, populating the index and caching the `Parse`
  per file. On `didChange`, **reparse only the changed file** (a full single-file
  reparse — cheap) and **replace that file's slice** of the index wholesale.
  Because the key carries the file URI, one file's entries are self-contained:
  no cross-file invalidation, no partial-stale window. This is the incremental
  strategy at **file granularity** — sub-file/subtree incremental reparse is a
  future optimization the shape does not preclude (§Consequences).

### 5. Positions: the server owns a `LineIndex`

`phalcom-ast` reports **byte** `Range<usize>` spans. LSP positions are
`(line, character)` with the negotiated encoding (**UTF-16** by default). The
CLI's `byte_offset_to_line_col` (`cli.rs:177`) is **`char`-based and 1-based**
and lives in the bin — **not reusable**. `phalcom-lsp` owns a per-document
`LineIndex` that maps byte offset → 0-based line + UTF-16 code-unit column, kept
in sync with the document store. This is new, small, and the single place
position math lives.

### 6. First concrete increment

Ship **only Stage 1**: `phalcom-lsp` over stdio, `didChange` → live multi-error
diagnostics, nothing else. Gate it behind a new `phalcom.lsp.enabled` setting
(default **off**), so it runs **alongside** the existing subprocess diagnostics
rather than replacing them. This de-risks cutover: the subprocess path stays the
default until the server is proven, and the two never both publish (the client
launches the LanguageClient *or* registers the TS providers based on the flag).

### 7. Migration path for `tools/vsphalcom`

As each stage lands and is proven, the corresponding TypeScript provider is
deleted, its capability now served in-process:

- Stage 1 lands → `diagnostics.ts` (147 lines) removed.
- Stages 3–4 land → `completions.ts` (219) + `hover.ts` (344) + `context.ts`
  (51) removed.
- Stage 5 lands → the TextMate grammar demoted to the `lsp.enabled=false`
  fallback.

End state: `extension.ts` collapses to a ~30-line `vscode-languageclient` shim
that spawns `phalcom-lsp` over stdio and forwards capabilities. The ~800 lines
of hand-rolled TS intelligence become server-side Rust reusing the real parser.

## Consequences

- **Multi-error, live, in-process diagnostics** replace single-error,
  save-only, subprocess ones — a strict improvement obtained by calling
  `parse` instead of `parse_source`, with no new front-end API.
- **One selector truth.** The index, go-to-def, references, completion, and
  hover all key on the ADR-0012 comma-form selector, the same key
  `core-table.json` already uses — no bare-name aliasing, no second encoder.
- **The server links no VM.** Parse-only capabilities keep `phalcom-lsp`'s
  dependency graph to `phalcom-ast` + `phalcom-common`, so it starts instantly
  and cannot be destabilized by runtime/heap churn.
- **Coloring becomes exact** at Stage 5, retiring the DEC-VSP-C approximation
  caveat while keeping the grammar as an offline fallback.
- **Standing obligation:** the `LineIndex` UTF-16 mapping is the one correctness
  hotspot (off-by-one or byte-vs-code-unit bugs misplace every diagnostic and
  jump target). It gets direct unit tests over multibyte fixtures, independent
  of any LSP round-trip.
- **Positions are negotiated, not assumed.** The server advertises its supported
  `positionEncoding`s at `initialize` and honors the client's choice, so a
  future UTF-8-preferring client needs no rework of the span math (only the
  `LineIndex` column unit).

### What this decision must NOT preclude

- **Future incremental (sub-file) parsing.** Index consumers must treat "a
  file's entries" as an opaque, wholesale-replaceable slice and never assume the
  whole file was reparsed as one unit. When `phalcom-ast` later grows a
  range/subtree reparse entry, the index updates the affected slice the same
  way. **No new `phalcom-ast` entry is needed now** (`parse` suffices); the
  future one is additive.
- **Future multi-root workspaces.** The index key is the **absolute file URI**,
  not a workspace-relative path, and the workspace scan iterates *all* roots.
  Nothing assumes a single root or a single `Cargo`/module tree.
- **Future cross-file type inference.** Stage 3's receiver narrowing is a
  **pluggable resolver** (`construct`-site dataflow is the first, deliberately
  minimal, implementation), not a hardcoded rule. The index already carries
  class + selector + hierarchy edges, so a later inference pass consumes the
  same index without reshaping it. Completion must degrade gracefully (core-table
  fallback) when the resolver returns "unknown", never block on inference.
- **The subprocess path.** Because Stage 1 is flag-gated and additive, the
  existing `phalcom check` subprocess and the TextMate grammar remain the
  zero-configuration default until each stage supersedes them — cutover is
  reversible per-capability by toggling `phalcom.lsp.enabled`.

## Alternatives considered

- **Raw `lsp-server` + `lsp-types` (rust-analyzer's stack).** Synchronous,
  thread-based, no async runtime, minimal dependencies — a good match for a
  sync parser and maximal control. Rejected as the default because it hands us
  the JSON-RPC dispatch loop, lifecycle, and cancellation to reimplement for
  every capability, with no offsetting benefit at our scale; the tokio cost of
  `tower-lsp` is immaterial for a parse-only server. Kept as the fallback if
  `tower-lsp` maintenance stalls — the capability handlers port with modest
  effort since both speak `lsp-types`.
- **Keep the subprocess, just call `parse` for multi-error and debounce on
  change.** Cheaper short-term, but re-spawns a process per keystroke and can
  never grow go-to-def / references / receiver completion / semantic tokens,
  which need a warm cross-file index a one-shot process cannot hold. It is the
  thing this ADR supersedes, not an alternative to it.
- **More TypeScript intelligence in the extension** (parse the buffer in JS).
  Rejected outright: it re-implements the Phalcom grammar in a second language
  — exactly the drift ([ADR-0016](../accepted/0016-hand-written-lexer-and-recursive-descent-parser.md))
  the single hand-written parser exists to prevent — and would re-rot like the
  2023 table did.
- **Big-bang cutover (delete the subprocess/grammar, ship the full server).**
  Rejected: it couples five independently-riskable capabilities into one
  irreversible switch. Staged, flag-gated supersession lets each capability be
  proven and each TS file be deleted only once its replacement is green.
