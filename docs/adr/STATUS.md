# ADR status tracker

One row per ADR. **Status** is Proposed / Accepted / Retired (verbatim-derived
from each file's own status line, collapsed to one of three buckets — Retired
covers "Superseded"/"Deferred past v0.2"). **Superseded** names the ADR that
retired it, where applicable. **Shipped** is whether the design is actually
implemented in the tree, independent of paper status — the two drift apart in
both directions in this repo (see [`STATE.md`](../forge/STATE.md) and the
2026-07-14 overlay re-grounding: `cfbc17b`, `cd291b9`). `✅` = personally
code-verified this pass or in a prior session this document's author ran;
`—` = not applicable (policy/process doc, no code to ship); `?` = not
verified, do not assume either way.

Gap: **ADR-0034 does not exist** (no file in `docs/adr/`, numbering skips it).

## Maintenance rules — binding on any edit to an ADR or this file

1. **Two-way sync.** Flip an ADR's status line (Proposed→Accepted, Accepted→Retired, etc.) → flip its row here in the same edit pass. Flip a row here → flip the ADR file's own status line. Never let the two disagree after your edit — that mismatch is exactly the defect class this file exists to catch (see the 2026-07-14 rulings above).
2. **Record every Shipped transition.** The moment you code-verify (or code-*un*verify) an ADR's implementation — grep, read, or a subagent's confirmed finding — update its Shipped cell immediately, with the evidence inline (file:line or commit), not just `✅`/`?`/`❌` bare. Don't defer this to a later sweep.
3. **Record every Superseded/Retired relationship the moment it's ruled.** Update both ADRs: the retired one gets a dated callout + Status flip to Retired, the superseding one gets no special mark (it's just Accepted) but this table's Superseded-by column must name it. Do this in the same commit as the ruling — not as a follow-up.
4. **Don't guess.** `✅` requires evidence you (or a prior pass this document accounts for) actually produced. Unverified stays `?`. Upgrading `?`→`✅` without checking the tree is the ADR-0024/0042 mistake in reverse.

ADRs live under `accepted/`, `proposed/`, `retired/` (status-named folders, see
`README.md` "Conventions"). Regenerate the sweep with:
```sh
cd docs/adr && for f in accepted/*.md proposed/*.md retired/*.md; do grep -m1 -iE '^- Status:|^Status:|^## Status' "$f"; done
```

| ADR | Title | Status | Superseded by | Shipped |
|---|---|---|---|---|
| 0001 | Record architecture decisions | Accepted | | — |
| 0002 | Metaclass tower follows the parallel rule | Accepted | | ✅ |
| 0003 | Introduce `Behavior` as a shared kernel class | Accepted | | ✅ |
| 0004 | Bool as abstract `Bool` + `True`/`False` | Accepted | | ✅ |
| 0005 | Flat `Number` backed by `f64` | Retired | ADR-0024 (in part — `f64` survives as `Float`'s backing) | ✅ (partial — see 0024) |
| 0006 | `Function` as abstract callable root | Accepted | | ✅ |
| 0007 | Absence as abstract `Option` + `Some`/`None` | Accepted | | ✅ |
| 0008 | Layered exceptions + `Result`, terminating | Accepted | | ✅ |
| 0009 | Handle/arena heap | Accepted | | ✅ |
| 0010 | `Value` tagged enum, private `Nil` sentinel | Accepted (numeric arm amended by 0024) | | ✅ |
| 0011 | Static per-class instance slot layout | Accepted | | ✅ |
| 0012 | Label-encoded selectors, IC-ready dispatch | Accepted | | ✅ |
| 0013 | Open/closed upvalues, frame-token non-local return | Accepted | | ✅ |
| 0014 | `let`/`var` bindings | Accepted | | ✅ |
| 0015 | `Object` default `toString` | Accepted | | ✅ |
| 0016 | Hand-written lexer + recursive-descent parser | Accepted | | ✅ |
| 0017 | Class-side stored static fields | Accepted | | ✅ |
| 0018 | Sacred-selector inliner + override-epoch guard | Accepted | | ✅ |
| 0019 | Freeze the VM-blessed primitive floor | Accepted | | ✅ |
| 0020 | Kernel `List` — native-array-backed protocol | Accepted | | ✅ |
| 0021 | No-truthiness enforcement | Accepted | | ✅ |
| 0022 | String interpolation `\(expr)` sigil | Accepted | | ✅ |
| 0023 | Amend floor — `hash`, kernel reflection, `Number#toString`, `Error#message`/`raise` (omnibus pre-clearance) | Accepted | | partial — pre-clears 0028/0036/0037, see those rows |
| 0024 | Split `Number` → `Int` (bignum) + `Float` | Accepted | | ❌ **not built** — code is still flat (`core.ph:75`); committed design, zero implementation |
| 0025 | External labels vs internal param names | Accepted | | ✅ |
| 0026 | Methods open; superclass reparenting sealed | Accepted | | ✅ |
| 0027 | Module = file, public-by-default exports | Retired | ADR-0045 (partial — resolver + import-form grammar only) | partial |
| 0028 | Amend floor — admit `Method` reflection | Accepted | | ✅ (code-confirmed 2026-07-14: `primitive/block.rs`, `primitive/method.rs`) |
| 0029 | List literals `[a,b,c]` | Accepted (ratified w/ 0032) | | ✅ |
| 0030 | Fibers/Futures — cooperative concurrency | Accepted | | ✅ |
| 0031 | Error surface syntax `try`/`catch`/`on`/`ensure` | Accepted | | ✅ |
| 0032 | Collections — representation, protocol, literals | Accepted | | ✅ |
| 0033 | Amend fiber exec — trampoline block call-site | Retired | — (deferred past v0.2, not superseded by a specific ADR) | ❌ not built, explicitly deferred |
| 0034 | *(no file — numbering gap)* | — | | — |
| 0035 | Iteration protocol — two-selector cursor | Accepted | | ✅ (amended by 0048) |
| 0036 | Amend floor — admit `Number#toString` | Accepted | | ✅ (code-confirmed 2026-07-14: `primitive/number.rs:88`) |
| 0037 | Amend floor — admit `Error#message`/`raise` | Accepted | | ✅ (code-confirmed 2026-07-14: `Error.new().raise()` throughout `core.ph`) |
| 0038 | Amend floor — admit `Block#on`/`ensure` | Accepted | | ✅ |
| 0039 | Amend floor — admit collection-container primitives | Accepted | | ✅ |
| 0040 | `SuperSend` dispatch opcode | Accepted | | ✅ (verified against tree 2026-07-14: `bytecode.rs:93`, `vm/dispatch.rs:643`) |
| 0041 | Hierarchy-stability policy — sealed reparent, single inheritance | Accepted | | ✅ |
| 0042 | Flat `Number`, defer split | Retired | ADR-0024 (ruled 2026-07-14) | — moot, superseded before build |
| 0043 | No default arguments | Accepted | | ✅ |
| 0044 | `Option` bootstrap formalization; defer niche-encoding | Accepted | | ✅ (correctness half only — niche-encoding itself deferred; `8d401f4` Track 2 sealed `Option`/`Some`/`None` against user subclassing, answering this ADR's open subclass-compatibility question by ruling it moot) |
| 0045 | `import` relative-path, whole-module binding | Accepted | | ✅ |
| 0046 | Destructuring `let`/`var` — tuple + list/`*rest` | Accepted | | ✅ |
| 0047 | `::` method references (Open form); amend floor | Accepted | | ✅ |
| 0048 | Amend iteration — bare-cursor sentinel + `Iterable` root | Accepted | | ✅ |
| 0049 | Amend floor — String byte/slice + raw stdout write | Accepted | | ✅ |
| 0050 | Non-moving precise mark-sweep collector | Accepted (ratified 2026-07-14) | | ? (GC code present — `vm/gc.rs`, `force_gc` — not verified against this ADR's specific design this pass) |
| 0051 | Performance strategy — measure-first, tiered | Proposed | | — (process/policy doc, nothing to ship) |
| 0052 | Invariant re-entrancy receiver-scoped; decorator state Layout-confined | Accepted | | ? |
| 0053 | Runtime decorator interception reuses override-epoch guard | Accepted | | ? (pristine-flag mechanism confirmed present; per-class `has_runtime_interceptor` bit not re-checked this pass) |
| 0054 | Two-speed decorator ratification | Accepted (broad, ruled 2026-07-14) | | ? |
| 0055 | Index syntax sugar over `at` selectors | Retired | ADR-0060 (ruled 2026-07-14) | ❌ not built as designed — superseded before full implementation |
| 0056 | `phalcom-lsp` in-process language server | Proposed | | ✅ **shipped despite Proposed status** — `phalcom-lsp` crate exists in the workspace (`Cargo.toml` members), multiple `feat(phalcom-lsp)`/`feat(U-LSP)` commits landed. Same status/reality gap class as 0028/0036/0037/0040 before their fix — not yet reconciled |
| 0057 | Decorator vs proxy granularity split | Accepted | | ? |
| 0058 | Reactive tracking-context needs a native module | Accepted | | ? |
| 0059 | Amend ADR-0058/0033 — reactive tracking context bound to native-frame switch guard | Proposed (needs user ratification) | | — (row missing pre-edit; added 2026-07-14, not yet code-checked) |
| 0060 | `[]` is a real, overridable selector — no `at` lowering | Accepted | | ✅ built 2026-07-14 (U-INDEX) — `Parser::parse_index_member` (`phalcom-ast/src/parser.rs`) is a dedicated bracket-subscript class-member production (params inside `[...]`, not `parse_method_name`); compiler (`compiler/lib/expr.rs`) sends directly to `SignatureKind::Subscript` (`[_]`/`[_,put]`/`[]`/`[put]`), no `at` lowering; `List`/`Map`/`Tuple` opt in via `core.ph` wrapper methods |
| 0061 | Underscore prefixes reserved — `_` fields, `_$` language internals, `__` reserved | Proposed (needs user ratification) | | ❌ not built — designed 2026-07-14, citations re-verified against `6d0b3b4`; current state is the *pre*-decision one it changes: `parse_primary` (`phalcom-ast/src/parser.rs:2387`) routes every leading-`_` identifier to `Expr::Field` regardless of underscore count, `parse_method_name` (`:1374`) and `parse_field_decl` (`:1156`) have no prefix check, `$` appears nowhere in `lexer.rs`, and M-ATTR-ROOT still ships as `__attach`/`__attributes`/`__freezeAttributes` (`primitive/attribute.rs`, `universe/primitives.rs:73-75`, `compiler/lib/class_decl.rs:785,798`) |

## Known status/reality gaps not yet reconciled

- **ADR-0056** — Proposed, but `phalcom-lsp` is a real workspace crate with landed `U-LSP` stages (diagnostics, symbol index, completion, hover, semantic tokens per commit log). Same pattern as 0028/0036/0037/0040 (ADR left Proposed after the unit shipped) — not yet put to the user for a ruling.
- **0052/0053/0054/0057/0058** — Accepted, plausible given the decorator/reactivity work landed this session's history shows, but not individually code-verified in this pass. Marked `?` rather than asserted.
- **0055** — Retired 2026-07-14, superseded by ADR-0060 (`[]` real selector, no `at` lowering). No longer in the unreconciled-gap bucket.
- **0050** — GC code exists in the tree (`vm/gc.rs`, `force_gc`) but wasn't diffed against this ADR's specific non-moving-mark-sweep design before marking `?`.

Do not upgrade a `?` to `✅` without checking the tree — that is exactly the mistake ADR-0024/0042 made in reverse (a doc claiming a status the code didn't back).
