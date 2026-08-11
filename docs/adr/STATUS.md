# ADR status tracker

> **This folder is being migrated to [`docs/pdr/`](../pdr/README.md).** New
> decisions are recorded there, numbered continuing from 0064 so migration is a `git mv`.
> ADR-0001…0064 stay tracked here until they move. A decision in the new folder that
> supersedes an ADR updates that ADR in both trackers.

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
| 0007 | Absence as abstract `Option` + `Some`/`None` | Accepted (amended by PDR-0033) | PDR-0033 | ✅ PDR-0033 shipped 2026-08-11; full `cargo test -p phalcom-core` passed |
| 0008 | Layered exceptions + `Result`, terminating | Accepted | | ✅ |
| 0009 | Handle/arena heap | Accepted | | ✅ |
| 0010 | `Value` tagged enum, private `Nil` sentinel | Accepted (numeric arm amended by 0024; immediate Option arm amended by PDR-0033) | PDR-0033 | ✅ PDR-0033 shipped 2026-08-11; full `cargo test -p phalcom-core` passed |
| 0011 | Static per-class instance slot layout | Accepted | | ✅ |
| 0012 | Label-encoded selectors, IC-ready dispatch | Accepted | | ✅ |
| 0013 | Open/closed upvalues, frame-token non-local return | Accepted | | ✅ |
| 0014 | `let`/`var` bindings | **Superseded by [0064](accepted/0064-let-const-bindings-and-field-mutability.md)** (2026-07-15) | | ✅ built as written — superseded on **spelling**, not semantics: 0064 renames `var`→`let`, `let`→`const` and keeps every rule (uninitialized-mutable reads `None`, immutable requires an initializer). A citation to 0014's *behavior* is still correct; only its keywords moved. Unmigrated until U-BINDINGS lands |
| 0015 | `Object` default `toString` | Accepted | | ✅ |
| 0016 | Hand-written lexer + recursive-descent parser | Accepted | | ✅ |
| 0017 | Class-side stored static fields | Accepted | | ✅ |
| 0018 | Sacred-selector inliner + override-epoch guard | Accepted | | ✅ |
| 0019 | Freeze the VM-blessed primitive floor | Accepted | | ✅ |
| 0020 | Kernel `List` — native-array-backed protocol | Accepted | | ✅ |
| 0021 | No-truthiness enforcement | Accepted | | ✅ |
| 0022 | String interpolation `\(expr)` sigil | Accepted (amended 2026-07-15 — sigil unchanged; desugar target moved `String.new(expr)` → `expr.toString`, the revisit the ADR itself pre-authorised once U-CORE-4 landed) | | ✅ |
| 0023 | Amend floor — `hash`, kernel reflection, `Number#toString`, `Error#message`/`raise` (omnibus pre-clearance) | Accepted | | partial — pre-clears 0028/0036/0037, see those rows |
| 0024 | Split `Number` → `Int` (bignum) + `Float` | Accepted | | ❌ **not built** — code is still flat (`core.ph:75`); committed design, zero implementation |
| 0025 | External labels vs internal param names | Accepted | | ✅ |
| 0026 | Methods open; superclass reparenting sealed | Retired | [PDR-0001](../pdr/0001-classes-are-closed.md) (Axis 1 only; Axis 2 kept) | ✅ shipped, being removed |
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
| 0043 | No default arguments | Accepted (prose amended 2026-07-15 — decision unchanged; records that open-Q12 *fixed* the if-ever mechanism: call-site fold permanently forbidden, definition-time trailing-only expansion) | | ✅ |
| 0044 | `Option` bootstrap formalization; defer niche-encoding | Accepted (amended by PDR-0033) | PDR-0033 | ✅ bootstrap correctness and immediate representation; physical encoding remains deferred |
| 0045 | `import` relative-path, whole-module binding | Accepted | | ✅ |
| 0046 | Destructuring `let`/`var` — tuple + list/`*rest` | Accepted | | ✅ |
| 0047 | `::` method references (Open form); amend floor | Accepted | | ✅ |
| 0048 | Amend iteration — bare-cursor sentinel + `Iterable` root | Accepted | | ✅ |
| 0049 | Amend floor — String byte/slice + raw stdout write | **Accepted — authoritative on naming again** (2026-07-15) | | ✅ built **and now spelled as this ADR specified**: `byteCount_`/`byteAt_(_)`/`slice_(_,_)`/`write_(_)`. U-STRING had shipped `raw*` against this ADR and 0062 blessed it; the user re-ruled trailing-`_` on 2026-07-15 and the rename landed (70 sites, 26/26 green). **0062 is Retired** |
| 0050 | Non-moving precise mark-sweep collector | Accepted (ratified 2026-07-14; immediate Option edge amended by PDR-0033) | PDR-0033 | ◐ **partially verified.** ✅ §Decision 6 (safepoint latching) and §Decision 7 (temp roots) checked against the tree 2026-07-19: `Heap::gc_pending` latches in `insert` and is serviced only at the dispatch back-edge (`vm/dispatch.rs`), and `VM::temp_roots` + `push_temp_root`/`temp_root_depth`/`truncate_temp_roots` exist and are enumerated by `collect_roots` (`cdd2117` — shipped to fix a real `block_ensure` UAF, [log](../logs/2026-07-19-ensure-temp-root-uaf.md)). Immediate `Some` payload tracing is covered by `Value::gc_obj_ref()` and focused GC tests in `tests/gc.rs`; the rest of the ADR — non-moving/handle-stability claims, §Decision 9's growth policy, the Consequences list — **still not diffed** |
| 0051 | Performance strategy — measure-first, tiered | **Accepted (ratified 2026-07-14)** | | — (policy doc, no code to ship) — but **realized by cuts 001–007** (`docs/forge/perf-log/`): its laws P1/P2/P3 were the operative gate for every one. Ratified *after* seven cuts had already shipped under it |
| 0052 | Invariant re-entrancy receiver-scoped; decorator state Layout-confined | Accepted | | ? |
| 0053 | Runtime decorator interception reuses override-epoch guard | Accepted | | ? (pristine-flag mechanism confirmed present; per-class `has_runtime_interceptor` bit not re-checked this pass) |
| 0054 | Two-speed decorator ratification | Accepted (broad, ruled 2026-07-14) | | ? |
| 0055 | Index syntax sugar over `at` selectors | Retired | ADR-0060 (ruled 2026-07-14) | ❌ not built as designed — superseded before full implementation |
| 0056 | `phalcom-lsp` in-process language server | Proposed | | ✅ **shipped despite Proposed status** — `phalcom-lsp` crate exists in the workspace (`Cargo.toml` members), multiple `feat(phalcom-lsp)`/`feat(U-LSP)` commits landed. Same status/reality gap class as 0028/0036/0037/0040 before their fix — not yet reconciled |
| 0057 | Decorator vs proxy granularity split | Accepted | | ? |
| 0058 | Reactive tracking-context needs a native module | Accepted | | ? |
| 0059 | Amend ADR-0058/0033 — reactive tracking context bound to native-frame switch guard | Proposed (needs user ratification) | | — (row missing pre-edit; added 2026-07-14, not yet code-checked) |
| 0060 | `[]` is a real, overridable selector — no `at` lowering | Accepted (setter identity amended by PDR-0032) | | ✅ direct bracket dispatch retained; setters now encode `[index-args]=(put)` |
| 0062 | ~~Amend floor — String raw byte accessors + `System.rawWrite(_)` (supersedes 0049 naming)~~ | **Retired 2026-07-15** | | ❌ **reverted** — existed only to bless the `raw*` names U-STRING shipped against ADR-0049's trailing-`_` spec. User re-ruled trailing-`_`; `raw*` is gone from the tree. 0049 is authoritative again; the 4-binding floor amendment itself was never in question and is unaffected. **Do not cite 0062 for selector names** |
| 0064 | `let`/`const` bindings; unkeyworded mutable fields; `const` fields writable only in constructors | **Accepted** (ratified 2026-07-15) | | ❌ **decision ratified, not built** — supersedes 0014 on spelling only (`var`→`let`, `let`→`const`), plus one genuinely new rule: `const` field writes are legal only inside a `@constructor` (syntactic, no flow analysis). Motivated by a measured gap — `let` on a *field* is unenforced today (`clobber(v) { _n = v }` on a `let _n` → 99, no error). Implementation is [U-BINDINGS](../forge/units/U-BINDINGS/plan.md), which **lands before U-CTOR**. Risk is the 1080-site codemod (352 `var` + 728 `let`, 395 files): it is a *swap* (two passes turn `var`→`const`) and position-dependent (class-body `var _x`→`_x`, statement `var x`→`let x`), so it must be single-pass and AST-driven, not a `sed` |
| 0063 | Constructors are ordinary class-side methods — `@constructor`/`@class` decorators, `new_` allocator | **Superseded by PDR-0028 (2026-07-21)** | | ❌ historical decision, not built; current surface canon lives in PDR-0028 and `docs/spec/current` |
| 0061 | Underscore prefixes reserved — `_` fields, `_$` language internals, `__` reserved | Retired | PDR-0032 | superseded before shipment; PDR-0032's structural namespaces are implemented |

## Known status/reality gaps not yet reconciled

- **ADR-0056** — Proposed, but `phalcom-lsp` is a real workspace crate with landed `U-LSP` stages (diagnostics, symbol index, completion, hover, semantic tokens per commit log). Same pattern as 0028/0036/0037/0040 (ADR left Proposed after the unit shipped) — not yet put to the user for a ruling.
- **0052/0053/0054/0057/0058** — Accepted, plausible given the decorator/reactivity work landed this session's history shows, but not individually code-verified in this pass. Marked `?` rather than asserted.
- **0055** — Retired 2026-07-14, superseded by ADR-0060 (`[]` real selector, no `at` lowering). No longer in the unreconciled-gap bucket.
- **0050** — upgraded `?` → `◐` on 2026-07-19. Two decisions were diffed against the tree and hold: §Decision 6 (safepoint latching) and §Decision 7 (temp roots). The upgrade is deliberately partial — the non-moving/handle-stability claims and §Decision 9's growth policy were **not** checked, so the row records which halves were verified rather than promoting the whole ADR off two data points. The §7 check was not a docs pass: it came out of fixing a live `block_ensure` use-after-free.

Do not upgrade a `?` to `✅` without checking the tree — that is exactly the mistake ADR-0024/0042 made in reverse (a doc claiming a status the code didn't back).
