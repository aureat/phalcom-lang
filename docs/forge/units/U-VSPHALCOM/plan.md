# U-VSPHALCOM — modernize the `tools/vsphalcom` VS Code extension (grammar + language intelligence)

Status: **PLANNED**. Two units. Scoped tooling work, **not** a compiler-core
unit — floor **+0**, no `Value`/opcode/primitive changes. The extension now
lives in-tree at `tools/vsphalcom` (git-detached from its old standalone
origin, committed in-house going forward — a plain subfolder, **not** a Cargo
workspace member; no `Cargo.toml`, not built by `cargo build`). The 2023-era
contents are stale: 4-rule TextMate grammar with a wrong keyword list,
hardcoded 2023 type table (`ObjectType`/`NullType`/`VoidType` — object model
that no longer exists), no LSP client, wrong file extension (`.phal`).

Split into two write-set-disjoint-*except-`package.json`* units:

- **U-VSPHALCOM-1 (grammar + file association)** — self-contained, zero
  compiler dependency. Can land alone, immediately.
- **U-VSPHALCOM-2 (language intelligence: diagnostics + autocomplete +
  hover)** — depends on 1 (shares `package.json`; sequence after), and its
  diagnostics leg is **BLOCKED-ON-DECISION** on a small upstream CLI addition
  (see DEC-VSP-A) that is compiler-core work outside this unit's write-set.

## Role

Closes the drift between the editor tooling and the current v0.2 language
surface. Concretely:
- syntax highlighting today mis-colors current keywords, colors dead ones
  (`const`/`is`/`in`/`and`/`or`/`not` are not current keywords), and has no
  scope at all for `#` symbol literals, `::` method-refs, `@` attributes,
  `?.`/`??` Option operators, `[]`/`[]=` index sugar, `\(expr)` string
  interpolation, `///`/`//!` Phaldoc, or `_field` identifiers.
- completions come from a hardcoded 2023 type-descriptor table that names
  types (`Null`, `Void`) the spec deleted — worse than nothing, it teaches
  the wrong model.
- `.ph` files are not even associated (extension registered as `.phal`).

## Spec anchor

- `docs/spec/current/lexical-structure.md` §§9–14 — the token surface the new
  grammar must encode: Option operators `?.`/`??` (§ Option ops), `#` symbol
  literal **incl. the whitespace-adjacency rule** (`#foo` symbol vs `#` used
  elsewhere), `::` method reference, `@` attribute token, `[]`/`[]=` index
  sugar (desugars to `.at(_)`/`.at(_,put:)` — see [U-INDEX](../U-INDEX/plan.md)).
- [ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md) —
  selectors are canonical comma-form symbols; `foo`, `foo(_)`,
  `move(_,to,duration)` are **distinct** selectors. Load-bearing for
  U-VSPHALCOM-2: any harvested selector table, completion entry, or hover key
  MUST key by the comma-form selector symbol (name + arity + labels), **never**
  by bare name. A table keyed by bare name is wrong-by-construction.
- `docs/spec/current/experimental/doc-comments-phaldoc.md` — Phaldoc: `///` outer
  doc, `//!` inner doc; tag vocabulary §3; adjacency rule §5 (a doc block
  associates with the *next non-blank, non-`///` line* item), §4 (keyed by
  selector, not bare name). STATUS experimental/unratified — but `//`-prefixed,
  so **already lexically inert trivia today**; a client-side text harvester
  reading raw source needs **zero** compiler changes.
- `docs/spec/current/experimental/annotations-core.md` +
  `doc-comments-phaldoc.md` §8 — `@`-attribute contracts
  (`@requires`/`@ensures`/`@invariant`) and the contract-view hover harvest
  order. STATUS experimental/unratified, mid-implementation on main
  (`U-ANNOT-CONTRACTS`). This is a **named follow-on seam only** in this plan
  (a stub in the hover leg) — **not** implemented here, **not** a dependency.
- **No spec governs the extension itself** → no new ADR required (this is
  tooling, not language surface). The one *upstream* change U-VSPHALCOM-2 wants
  (a `phalcom check` mode, DEC-VSP-A) is compiler-core and, if adopted, gets
  its own CLI-scoped treatment — not authored here.

## Preconditions (confirmed this session — do not re-derive)

- **`phalcom-ast` is a standalone lib crate with no VM dependency.** Public
  entry `phalcom_ast::parser::{parse, parse_source}`
  (`phalcom-ast/src/parser.rs:76,91`) returns `Parse`/`Program`. Embeddable in
  a Rust codegen tool or a future `phalcom-lsp` without pulling in
  `phalcom-core`/the VM. **`core.ph` is itself parseable Phalcom source** via
  `parse_source` — the class/method-decl harvest for autocomplete can be a real
  parse, not a regex.
- **NO parse-only/`--check` CLI flag exists** (`phalcom-core/bin/phalcom/cli.rs`,
  full file read). What exists:
  - `phalcom parse <path>` subcommand (`cmd_parse`, `cli.rs:116`) — runs
    lex+parse only (`phalcom_ast::parse_source`, no VM), **but** prints the
    full AST `{program:#?}` to stdout on success and surfaces a
    `CompilerError`/anyhow error on failure. Not a clean diagnostics emitter.
  - `cmd_run` (`cli.rs:76`) — compiles+runs; on compile error calls
    `vm.compiler_error(e)` then `std::process::exit(1)`; on runtime error
    `eprintln!("{e}")` + exit 1. Miette-fancy output goes through
    `compiler_error`, not a stable machine format.
  - There is **no** mode that does *lex+parse only, emits miette diagnostics to
    stderr, exits nonzero, and prints nothing else*. This is the gap
    DEC-VSP-A names.
- **Native primitive registration is uniformly greppable/parseable**
  (`phalcom-core/src/universe/primitives.rs:37+`): every native method is a
  macro call `primitive!(vm, <class_var>, "<selector-name>",
  SignatureKind::<Getter|Setter|Method(N)>, <fn>)` (and `primitive_static!` for
  metaclass-side). The string literal is the selector name; `SignatureKind`
  gives the arity/kind → the comma-form can be reconstructed deterministically
  (`Getter` → `name`, `Method(1)` → `name(_)`, etc.). Per-class because the
  class var (`object_cls`, …) is the first arg. This is a clean harvest source
  alongside `core.ph`.
- **`core.ph` shape confirmed** (`core.ph:1–70`): ordinary
  `class Name { selector(params) { ... } }` / `selector => expr` /
  `construct new(...) { }` declarations — exactly what `parse_source` produces
  `Program` nodes for.

## Design

### Unit A — U-VSPHALCOM-1: grammar rewrite + file association

Pure editor-config work; no Rust, no subprocess, no compiler coupling. A
TextMate grammar is best-effort lexical coloring, **not** a parser — it does
not need to be selector-accurate (that is U-VSPHALCOM-2's job via the real
parser). Scope:

1. **Rewrite `syntaxes/phalcom.tmLanguage.json`.** Replace the 4-rule
   comment/string/keyword/variable set. New scopes, each mapped to a
   conventional TextMate scope name so themes color them:
   - **keywords** — correct current set only:
     `class extends super self static try catch on ensure throw break continue
     match return while for var` (drop `const is in and or not`; verify the
     live set against `docs/spec/current/lexical-structure.md` keyword table and
     the lexer token enum before finalizing — the caller's list is the floor,
     not necessarily exhaustive).
   - **`#` symbol literal** — `keyword.other.symbol` / `constant.other.symbol`,
     encoding the whitespace-adjacency rule (a `#` immediately followed by an
     identifier/operator-name is a symbol; a lone `#` is not). Regex-approx is
     acceptable here.
   - **`::` method reference** — `keyword.operator.methodref`.
   - **`@` attribute** — `entity.name.tag` / `meta.attribute`
     (`@requires`/`@ensures`/`@invariant` and bare `@name`).
   - **Option operators `?.` / `??`** — `keyword.operator.option`.
   - **index sugar `[` `]` / `[]=`** — bracket/operator scope (colors the
     subscript; the desugaring is runtime, invisible to the grammar).
   - **string interpolation `\(expr)`** — a `meta.embedded` /
     `meta.interpolation` scope inside the string rule so the interpolated
     expression is not colored as flat string text.
   - **Phaldoc `///` / `//!`** — a dedicated `comment.block.documentation`
     scope distinct from ordinary `//` line comments, so themes can style docs
     differently (and so U-VSPHALCOM-2's client-side harvester and the grammar
     agree on what a doc line is).
   - **field identifier `_foo`** — `variable.other.field` (leading-underscore
     convention).
2. **Fix file association**, in **both**:
   - `package.json` `contributes.languages[].extensions`: `.phal` → `.ph`
     (and `aliases`/`id` sanity).
   - `language-configuration.json`: confirm comment tokens (`//`, and Phaldoc
     `///`/`//!` still recognized as line comments for toggle-comment),
     brackets, autoClosingPairs cover `[` `]` `\( )` `#`-none.

Grammar is theme-driven; no VS Code API surface, no activation cost.

### Unit B — U-VSPHALCOM-2: language intelligence (diagnostics + autocomplete + hover)

Three legs on the TypeScript client (`src/`), sharing one **harvested core
table** artifact. Ordered as the caller specified: diagnostics → autocomplete
→ hover.

**Leg 1 — diagnostics via CLI subprocess.** On save/change (debounced), spawn
the real `phalcom` binary on the document, parse its diagnostic output into VS
Code `Diagnostic` objects (range from the reported span, severity, message),
publish to a `DiagnosticCollection`. **This leg is BLOCKED-ON-DECISION
(DEC-VSP-A):** there is no clean parse-only diagnostics mode today.
- **Recommended path:** request a small upstream `phalcom check <path>` (or
  `phalcom --check`) that runs lex+parse **only** (reusing
  `phalcom_ast::parse_source`, no VM), emits miette diagnostics to **stderr**,
  prints nothing to stdout, exits nonzero on any diagnostic. Stable, machine-
  reasonable (miette has a JSON/`GraphicalReportHandler` option — prefer a
  `--format=json` or a line-oriented `file:line:col: severity: message` form so
  the TS side parses spans robustly, not by scraping ANSI-art). This is
  **compiler-core work in `phalcom-core/bin/phalcom/cli.rs`**, outside this
  unit's write-set — flag as a dependency, do not implement here.
- **Fallback if DEC-VSP-A is declined:** shell `phalcom parse <path>`, discard
  stdout (the AST dump), scrape stderr for the error. Fragile (anyhow debug
  format, single-error, not spanned cleanly) — acceptable stopgap, explicitly
  inferior. Ship the leg behind a config flag so it degrades gracefully if the
  binary/flag is absent (`phalcom.diagnostics.enabled`, default true only when
  a `check`-capable binary is found).

**Leg 2 — static autocomplete from a harvested core table.** Delete the stale
hardcoded table in `src/language.ts`; replace with a **build-time-generated**
table (recommended over hand-maintained — the caller's two options, decision
below):
- **Recommended: a codegen script** (`tools/vsphalcom/scripts/gen-core-table`)
  producing a checked-in JSON (`src/generated/core-table.json`) with two
  sources merged:
  1. **`core.ph`** parsed via a tiny Rust helper binary that calls
     `phalcom_ast::parse_source` and walks the `Program` for class decls +
     their method selectors (real parse, ADR-0012-correct comma-form
     reconstructed from the decl's name+params+labels). *Or*, to avoid adding a
     Rust build step to a non-Cargo subfolder, a Node parse of `core.ph` is
     **not** recommended (re-implements the grammar) — prefer the Rust helper,
     run manually and its output committed (see DEC-VSP-B).
  2. **`primitive/*.rs`** via `primitives.rs`'s `primitive!(... "sel",
     SignatureKind::K ...)` macro calls — a regex/line scan keyed on the macro
     name is sufficient and stable (the macro form is uniform), reconstructing
     comma-form from `SignatureKind` (`Getter`→`sel`, `Method(n)`→`sel(_,…)`,
     `Setter`→`sel=`). Group by the class-var first argument.
  - Merge → `{ keyword: [...], classes: { ClassName: [ {selector: "move(_,to,duration)", kind, arity, source} ] } }`.
- `src/completions.ts` reads that JSON: keyword completions + per-receiver-class
  selector completions (comma-form label snippets: `move(${1:_}, to: ${2:_}, duration: ${3:_})`).
  **No parsing of the user's buffer** in this unit (that needs the workspace
  symbol index deferred to a future LSP unit) — completions are the static
  keyword set + core-class selector set. This is honest about scope: it teaches
  the *real* model, correctly keyed, without pretending to do type inference.

**Leg 3 — hover.** `src/hover.ts` (new):
- **Keyword docs** — a small static map (hand-written short blurbs; keywords
  are a closed set).
- **Selector signature hover** — look the hovered token up in the same
  harvested core table; render the comma-form signature + kind + source class.
  Keyed by selector per ADR-0012 (`foo` vs `foo(_)` render differently).
- **Phaldoc layer** — a **client-side raw-source text harvester** (mechanical
  trivia scan, zero compiler calls): scan the document (and, for core symbols,
  `core.ph`) for `///` blocks, apply the adjacency rule
  (`doc-comments-phaldoc.md` §5: doc block binds to the next non-blank,
  non-`///` line's declared item), key the harvested text by that item's
  **selector** (§4), attach the summary/tags to the hover. Because `///` is
  inert trivia, this is pure string work.
- **Contract-view hover — named stub only.** Leave a documented seam
  (`renderContractView(selector): undefined // TODO: gated on U-ANNOT-CONTRACTS`)
  that the hover composer calls and currently returns nothing. Harvest order
  when built: `doc-comments-phaldoc.md` §8 (selector → summary → signature+types
  → requires → ensures → invariant → raises → detail → example). **Do not
  implement** — `@`-attribute contract parsing is out of scope and gated on the
  in-flight `U-ANNOT-CONTRACTS` unit.

### Explicitly deferred (noted, NOT planned here)

- **Go-to-definition / find-references** — needs a workspace-wide symbol index
  and a real `phalcom-lsp` Rust crate (LSP server embedding `phalcom-ast`).
  Future unit; the harvested-table + subprocess design here is deliberately a
  stepping stone toward it, not a replacement.
- **Full type inference** — out.
- **`@`-attribute contract parsing / contract-view hover** — gated on
  `U-ANNOT-CONTRACTS`; only the named stub lands here.

## Write-set (STOP-and-report if outside)

**U-VSPHALCOM-1:**
- `tools/vsphalcom/syntaxes/phalcom.tmLanguage.json` — grammar rewrite.
- `tools/vsphalcom/language-configuration.json` — comment/bracket/autoclose config.
- `tools/vsphalcom/package.json` — **only** the `contributes.languages` /
  `contributes.grammars` / file-association (`.phal`→`.ph`) region.

**U-VSPHALCOM-2:**
- `tools/vsphalcom/src/language.ts` — delete stale type table.
- `tools/vsphalcom/src/completions.ts` — read harvested table.
- `tools/vsphalcom/src/diagnostics.ts` (**new**) — subprocess + `Diagnostic` mapping.
- `tools/vsphalcom/src/hover.ts` (**new**) — keyword/selector/Phaldoc hover + contract stub.
- `tools/vsphalcom/src/extension.ts` (or existing activation entry) — register
  diagnostics/hover providers, activation events.
- `tools/vsphalcom/scripts/gen-core-table/` (**new**) — codegen (Rust helper +
  glue); its output `tools/vsphalcom/src/generated/core-table.json` (checked in).
- `tools/vsphalcom/package.json` — **only** the `dependencies` (VS Code
  API/no-LSP-yet) + `activationEvents` + `contributes.configuration` region.
- **Shared file `package.json`:** both units touch it in disjoint regions.
  **Sequence A before B** (do not run in parallel) to avoid a merge conflict on
  the manifest — the format's single-writer rule ([[phalcom-concurrent-session-hazards]]).
- **Floor: +0.** No `phalcom-*` crate changes. (DEC-VSP-A, if adopted, edits
  `phalcom-core/bin/phalcom/cli.rs` — that is a **separate** compiler-core
  change with its own write-set, NOT part of this unit.)

## Build order

**U-VSPHALCOM-1:**
1. Rewrite grammar; smoke-test coloring on `examples/*.ph` + `core.ph` in a
   dev-host VS Code window (every new scope visibly colored, no dead keyword
   colored). 2. Fix `.phal`→`.ph` in `package.json` + `language-configuration.json`;
   confirm a `.ph` file activates the language. Commit ([[commit-frequently]]).

**U-VSPHALCOM-2:**
1. **Codegen first** — build `gen-core-table`, generate + commit
   `core-table.json`; eyeball that comma-form selectors are ADR-0012-correct
   (`move(_,to,duration)` not `move`) and that `Null`/`Void` are absent.
2. **Autocomplete** — wire `completions.ts` to the JSON; prove keyword + core
   selector completions appear, correctly labeled.
3. **Hover** — keyword map + selector-from-table + Phaldoc harvester; land the
   contract-view stub (returns nothing).
4. **Diagnostics** — **gated on DEC-VSP-A.** If the `check` mode exists: wire
   the subprocess + span parsing. If not: land the fallback behind the config
   flag, or descope the leg to a follow-on and ship legs 2–3 (they have no
   compiler dependency). Commit per green leg.

## Tests / verification

The extension is not covered by `cargo test`; verification is:
- **Grammar (U-1):** a scope-inspection check — open `core.ph` + a fixture
  exercising every new token (`#sym`, `a::b`, `@requires`, `x?.y`, `a ?? b`,
  `xs[i]`, `"v=\(x)"`, `/// doc`, `_field`) in the dev host; assert each token's
  scope via VS Code's *Inspect Editor Tokens and Scopes*. Optionally a
  `vscode-tmgrammar-test` snapshot fixture checked into `tools/vsphalcom/test/`
  if the toolchain is added (recommended, gives a regen-able golden).
- **Codegen (U-2):** the generated `core-table.json` must (a) contain **no**
  `Null`/`Void`/`ObjectType` legacy names, (b) key every selector in comma-form
  (a lint asserting no bare-name key collides where the real selector has
  arity/labels — the ADR-0012 gate), (c) round-trip: re-running the codegen on
  an unchanged tree produces a byte-identical file (determinism gate).
- **Autocomplete/hover:** VS Code integration tests
  (`@vscode/test-electron`) if the harness is stood up — assert a completion
  list contains a known core selector in comma-form, and a hover over a known
  selector renders its signature + (where present) its `///` summary. At
  minimum, a manual dev-host checklist in the unit's Return shape.
- **Diagnostics:** a fixture `.ph` with a known parse error must surface a
  `Diagnostic` at the correct span; a clean file must surface none. (Gated on
  DEC-VSP-A resolution.)
- `npm run compile` / `tsc` clean; extension activates without console errors.

## Decisions to flag (DEC-VSP)

- **DEC-VSP-A — parse-only diagnostics CLI mode. BLOCKED-ON-DECISION,
  upstream dependency (compiler-core, outside this write-set).** No `--check`
  exists; `phalcom parse` dumps the AST and surfaces errors in an unstable
  anyhow format. **Options:** (A, recommended) add a small `phalcom check
  <path>` that runs lex+parse only, emits miette diagnostics to stderr in a
  machine-parseable form (`--format=json` or `file:line:col: severity: msg`),
  stdout-silent, exits nonzero — a self-contained ~small addition to
  `phalcom-core/bin/phalcom/cli.rs` reusing `phalcom_ast::parse_source`; (B)
  scrape `phalcom parse` stderr as a fragile stopgap; (C) descope the
  diagnostics leg entirely to the future `phalcom-lsp` unit and ship only
  grammar + autocomplete + hover now. **Recommendation: A**, but this is a
  compiler-core change — the user/orchestrator must greenlight it; **do not
  author it inside this unit.** If not greenlit, fall to B (flagged) or C.
- **DEC-VSP-B — autocomplete table: generated vs hand-maintained.**
  **Recommendation: generated** (codegen from `core.ph` via `parse_source` +
  `primitives.rs` macro scan), checked-in output regenerated on demand. A
  hand-maintained table re-rots exactly as the 2023 one did. Sub-decision: the
  `core.ph` harvest uses a **tiny Rust helper** (embeds `phalcom-ast`,
  authoritative) run manually with committed output — **not** a Node
  re-implementation of the grammar, and **not** a new permanent Cargo workspace
  member (the helper can be a throwaway `cargo run --manifest-path` script or a
  dev-only bin under the tool dir; keep `tools/vsphalcom` a non-workspace
  subfolder per the move's intent). Confirm the packaging choice with the
  orchestrator if a Rust build step in the tool dir is undesirable.
- **DEC-VSP-C — grammar accuracy ceiling.** TextMate is regex lexical coloring,
  not a parser; the `#` whitespace-adjacency rule and `\(expr)` interpolation
  are approximated, not exact. Accepted: exactness is U-VSPHALCOM-2's real
  parser's job (diagnostics/hover), not the grammar's. No decision needed —
  recorded so no one files the grammar's approximation as a bug.

## What must this not preclude (P4)

- **A future `phalcom-lsp` crate + go-to-definition.** The subprocess-
  diagnostics + harvested-static-table design is a deliberate stepping stone:
  the harvest logic (parse `core.ph`, scan `primitives.rs`) is exactly what an
  LSP server would internalize. Keep the harvested table format
  selector-keyed and source-tagged so an LSP can supersede the subprocess
  without reshaping the data. Do not bake a subprocess-only assumption into
  `completions.ts`/`hover.ts` data shapes.
- **U-ANNOT-CONTRACTS contract-view hover.** The hover composer must call a
  named `renderContractView(selector)` seam (returning nothing today) so the
  contract layer drops in without restructuring the hover pipeline.
- **DEC-VSP-A option A’s `check` mode** must, if built upstream, emit spans
  compatible with the diagnostics parser this unit writes — coordinate the
  output format (prefer JSON) so the TS side is not rewritten when it lands.
- **The `.ph` association** must not conflict with any other extension claiming
  `.ph` (Perl 5 headers historically use `.ph`) — set a specific `language id`
  (`phalcom`) and rely on the grammar scope, don't globally reassign.

## Return shape (implementer)

commit SHA(s) · **U-1:** grammar rewritten, every new scope confirmed via
Inspect-Scopes (list them), `.phal`→`.ph` fixed in both files, `.ph` activates ·
**U-2:** `gen-core-table` built + `core-table.json` committed (assert no
`Null`/`Void`, all selectors comma-form, determinism round-trip green),
`completions.ts` wired (sample completion shown), `hover.ts` landed
(keyword+selector+Phaldoc, contract stub present-and-inert), diagnostics leg
status per DEC-VSP-A (wired / fallback-flagged / descoped) · `tsc` clean,
activates clean · DEC-VSP-A resolution taken · DEC-VSP-B packaging choice taken ·
floor delta (exp 0; note separately if DEC-VSP-A-A edited `cli.rs` as its own
change) · write-set confirm.
