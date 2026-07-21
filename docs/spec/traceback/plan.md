# Traceback & diagnostics — implementation plan

Dependency-ordered units per forge Phase-2 conventions. Each unit cites its spec section
([`implementation-spec.md`](implementation-spec.md), "IS §n" below) or decision record, carries a
write-set (exact files it may modify), dependency edges, and a test strategy. Verify gate for
every unit: `cargo build && cargo test && cargo clippy --workspace`, plus a throwaway-worktree
build at each commit SHA (clean-checkout rule). All Rust code fully rustdoc'd
(`docs/rust-documentation-guidelines.md`) — undocumented public API is an incomplete change.

**Standing constraints:** main has live concurrent sessions — commit narrow paths on `main`
itself, never `git add -a`, never `git checkout -b`. Commit per green checkpoint. Perf numbers
only from `docs/forge/perf-log/SCOREBOARD.md`.

---

## Gates (before any U-TRACE implementation)

### G1 — Ratify PDR-0010  ✅ RESOLVED (ratified 2026-07-20)

Ratified as-is; the normative `kind` table + isA-vs-`kind` rule live in IS §8.1 per the
ratification ruling. STATUS.md flipped the same pass. T3 and T6 are unblocked (T3 still waits
on G2).

### G2 — Fix E002 (fiber-floor upvalue close)  ✅ RESOLVED (fixed `a265684`, doc flip `3306fdf`)

Landed 2026-07-20: originating fiber's live upvalues closed before the switch
(`close_upvalues_from(0)` ahead of the cascade), plus a fiber-scoped
`close_fiber_upvalues_from` for each cascaded `Call`-mode resumer's parked stack before its
clear. Root-fiber exit untouched (frames stay live for the traceback). Negative-controlled,
clean-worktree verified. Both gates are now discharged; T3 is unblocked once G0's `error.rs`
write-set frees. Original ruling kept below for the record.

Ruled 2026-07-20: fixed standalone, not folded into T3 (which would chain a live crash to
governance latency) and not left to the unscheduled U-FIBER track. Re-derive the fix from code,
not from E002's recorded prescription (house rule). Confirm no concurrent session holds fiber
dispatch before dispatching. OPEN, confirmed 2026-07-20
(`docs/errors/E002-fiber-floor-upvalue-crash.md`). The cascade loop
T3 instruments is the code that must close upvalues before clearing
(`dispatch.rs:341-370`, `primitive/fiber.rs:55-57`). Landing T3 first would put the traceback
walk on a path that can hold dangling `ObjRef`s — converting a clean error into a panic at
`heap/mod.rs:188`. Write-set: `phalcom-core/src/vm/dispatch.rs` (fiber Err arm),
`phalcom-core/src/primitive/fiber.rs`. Test: E002's recorded repro as a golden negative +
escaped-block-after-fiber-failure fixture. E001 is already fixed (`cdd2117`) — no longer a gate.

### G0 — Map/Set reentrant `hash`/`==` *(separate unit, ranked ABOVE this whole track)*

Reproduced 2026-07-20, both modes; the panic mode is an uncatchable `.expect()` process abort
(`primitive/map.rs:80`). Not in U-TRACE; scheduled first as its own unit
(write-set: `primitive/map.rs`, `primitive/set.rs`, `heap/map.rs`). **Direction RULED
2026-07-20: reentrancy lock.** The collection is flagged for the duration of `locate()`'s
reentrant `hash`/`==` sends; any structural mutation of it while flagged raises a catchable
`Error` with `kind: #concurrentMutation` **at the mutation site** — the traceback then points
at the culprit line inside the user's `==`, with the outer operation as the frame above.
Fallible raw accessors (`set_value_at`/`remove_at` → `Option`) land underneath as
defense-in-depth. Scope: `locate()` only; iteration-during-mutation is a separate, known
question. Rejected: re-locate-after-mutation (order-dependent semantics + livelock bound),
version-counter-at-outer-op (blame one frame removed), fallible-accessors-alone (leaves silent
corruption).

---

## Units

### T1 — The walk and the accessor  *(IS §2)*

- **Delivers:** `vm/walk.rs` (`StackWalk`, `FrameView`, `FrameName`, logical-expansion seam),
  `Chunk::span_at`/`line_at`, all existing `spans[...]` consumers migrated to the accessor,
  `FiberObject::seq` display ids (IS §6).
- **Write-set:** `phalcom-core/src/vm/walk.rs` (new), `phalcom-core/src/vm/mod.rs` (module
  decl), `phalcom-core/src/chunk.rs`, `phalcom-core/src/vm/dispatch.rs` (accessor migration
  only), `phalcom-core/src/heap/fiber.rs`, `phalcom-core/src/primitive/fiber.rs` (seq at spawn).
- **Deps:** none. Independent of G1/G2.
- **Tests:** unit tests on `span_at` clamping (ip 0, past-end); a walk test over a 3-deep call
  asserting frame order oldest→first and selector-shaped names; negative-control each.

### T2 — Rendering substrate  *(IS §1, §3)*

- **Delivers:** `diagnostics/style.rs`, `diagnostics/caret.rs`, `RenderConfig`
  (TTY/`NO_COLOR`/`--color`/`--plain`/width), `unicode-width` dep, miette removed from the
  workspace, `CLAUDE.md` convention line corrected.
- **Write-set:** `phalcom-core/src/diagnostics/` (new submodules; `mod.rs` reshuffle),
  `phalcom-core/bin/phalcom/cli.rs` (flags), root `Cargo.toml` + `phalcom-core/Cargo.toml`
  (deps), `CLAUDE.md` (one line).
- **Deps:** none. Fully parallel with T1 (disjoint write-sets except trivial `Cargo.toml`).
- **Tests:** IS §11's caret unit battery (tabs, CJK, combining, windowing, two labels, ASCII);
  color-off invariance harness.

### T3 — Capture record + surface error structure  ⛔ G2  *(IS §4; PDR-0010 §1-§4 — ratified; G1 resolved)*

- **Delivers:** boundary capture in `block_on`, per-hop cascade capture, record beside
  `Raise.rendered`, `Error#kind`/`#cause`/`#displaced` (`.ph` fields + `capture_error_value`
  setting `kind`), `frames.clone()` deleted.
- **Write-set:** `phalcom-core/src/primitive/block.rs`, `phalcom-core/src/vm/dispatch.rs`
  (cascade loop + `capture_error_value`), `phalcom-core/src/error.rs`,
  `phalcom-core/core/core.ph` (Error fields/getters), `phalcom-core/src/universe/primitives.rs`
  (only if a getter needs a binding — PDR-0010 says zero floor delta, so expect no change).
- **Deps:** G1, G2, T1 (record uses walk vocabulary). **Coordination:** the cascade loop is
  U-FIBER-landed code; do not run concurrently with any session touching fiber dispatch.
- **Tests:** caught-error origin survives `attempt()` round-trip; cross-fiber chain fields via
  JSON fixture; `e.kind == #range`-shaped fixture (closes `error-handling.md:143-146`'s false
  examples the honest way); `displaced` set on ensure-supersession (reuses the verified 7-row
  ensure matrix programs).

### T4 — Traceback renderer, wired  *(IS §5)*

- **Delivers:** `diagnostics/traceback.rs` human + JSON; Python order, innermost-only caret,
  core elision + `--trace-core`, budgets + repeat collapse, fiber/cause chain rendering, native
  frame synthesis + `anchor_of` seam (bare form); `runtime_error` rewritten onto the
  walk/renderer; `print_rt`/`print_frame` deleted.
- **Write-set:** `phalcom-core/src/diagnostics/traceback.rs` (new),
  `phalcom-core/src/diagnostics/mod.rs` (deletions), `phalcom-core/src/vm/dispatch.rs`
  (`runtime_error` body, native send context stores), `phalcom-core/bin/phalcom/cli.rs`
  (`--trace-core`).
- **Deps:** T1, T2 hard; T3 for chain/caught rendering (an uncaught-only slice of T4 could land
  on T1+T2 alone if G1 stalls — the fallback that keeps the pipeline moving).
- **Tests:** JSON-fixture frame sequences for: base case, recursion collapse (DepthExceeded),
  fiber chain, native frame; the two churn-prone snapshot canaries.

### T5 — Entry-path unification + exit codes  *(IS §3.5, §7)*

- **Delivers:** `cmd_run` delegates to `interpret_source`; `sysexits` table honored; syntax
  errors one register (caret renderer, run + check); `compiler_error(err, module, source_id)`
  with spans for span-carrying variants; `SOURCE_MAP`/`register_source` deleted.
- **Write-set:** `phalcom-core/bin/phalcom/cli.rs`, `phalcom-core/src/interpret.rs`,
  `phalcom-core/src/vm/dispatch.rs` (`compiler_error` signature),
  `phalcom-core/src/vm/api.rs` (delete `register_source`), `phalcom-core/src/diagnostics/mod.rs`
  (`print_parse`/`print_compile` restyle), `phalcom-repl/src/repl.rs` (caller signature).
- **Deps:** T2 (renderer). Parallel with T3/T4 except the shared `dispatch.rs` hunk — sequence
  the `compiler_error` edit after T4's `runtime_error` edit or vice versa (same file, disjoint
  functions; trivial rebase).
- **Tests:** exit-code fixture per class (syntax 65, compile 65, runtime 70, missing file 66);
  run-vs-check same-bytes syntax diagnostic (strip-SGR compare); negative-lane migration for
  restyled parse errors.

### T6 — Message hygiene + did-you-mean  ✅ COMPLETE  *(IS §8, §9; G1 resolved — kind table normative at IS §8.1)*

- **Delivers:** style-guide enforcement pass over live messages; dead variants deleted
  (`UndefinedVar`, `UnsupportedOperation`, `BinaryNotSupported`, `UnaryNotSupported`,
  `ZeroDivision` + `number_div` doc); `Internal`-leak sites → typed `#type` errors;
  `diagnostics/suggest.rs` wired to selector miss / undefined var / unknown class; arity-miss
  dedicated hint.
- **Write-set:** `phalcom-core/src/error.rs`, `phalcom-core/src/vm/dispatch.rs` (undefined-var
  + NewInstance/field sites), `phalcom-core/src/primitive/object.rs`,
  `phalcom-core/src/primitive/number.rs` (doc only), `phalcom-core/src/diagnostics/suggest.rs`
  (new), compiler unknown-class site (`phalcom-core/src/compiler/`).
- **Deps:** T2. (`kind` values assigned per IS §8.1; T3 lands the `kind` field itself — T6
  after T3, or T6's `kind` assignments ride T3 if scheduled together.)
- **Tests:** pure suggest-engine table tests (metric, threshold, tie-suppression,
  determinism); one fixture per rewritten message; negative-control that the old `{:?}` leak
  strings no longer appear.

### T7 — Observability batch  *(IS §12)*

- **Delivers:** fiber-switch `debug!` events text+json, `--trace`/`--trace-format`, `main.rs`
  filter unhardcoded + `#![allow(warnings)]` removed + bin rustdoc'd, recursive disasm with
  resolved constants/captures/fusion notes.
- **Write-set:** `phalcom-core/src/vm/dispatch.rs` (`switch_to_fiber_and_deliver` + spawn/done
  sites), `phalcom-core/bin/phalcom/main.rs`, `phalcom-core/bin/phalcom/cli.rs`,
  `phalcom-core/bin/phalcom/disasm.rs`.
- **Deps:** T1 (walk labels, `span_at`), T2 (styler roles for the log + disasm). Independent of
  G1/G2 — a good pipeline filler while T3 waits on ratification.
- **Tests:** JSON trace fixtures on skynet-shaped programs (spawn/switch/yield/done fields);
  disasm golden on a nested-closure program asserting recursion + capture annotations (structure
  lane, not byte lane).

### T8 — Fixture consolidation sweep  *(IS §11)*

- **Delivers:** color-off invariance harness across all catalog examples; canary audit; a
  fixtures README stating the field-assert contract; confirmation every negative fixture was
  negative-controlled.
- **Write-set:** `phalcom-core/tests/` (goldens + harness only).
- **Deps:** T4, T5, T6, T7 (it audits their output).

### Sidecar S1 — cheap doc fixes *(scheduled now, one commit)*

- `error-handling.md:143-146`: annotate the three examples "unimplemented — see PDR-0010 §2"
  (if G1 ratifies quickly, annotate instead with the `e.kind` form). — followups §2.
- `block_on` rustdoc contradiction (`block.rs:226-227`): rewrite to match wrap-and-catch
  behavior. — followups §3.
- `docs/adr/README.md` still lists ADR-0014 as Accepted though the file says Superseded — fix
  the row (two-way sync rule).

### Governance: resolved

The ADR folder is frozen; design decisions live in `docs/decisions/`. Both records this track
needed now exist and are Accepted:
[PDR-0010](../../decisions/0010-errors-carry-structure-and-cheap-origin.md) (ratified
2026-07-20; G1) and
[PDR-0014](../../decisions/0014-diagnostics-renderer-is-in-house.md) (renderer is in-house,
miette leaves the workspace — written because the phantom-miette convention already produced
one wrong decision, 0066/`bb4f365`). No new ADR. `docs/adr/README.md`'s stale ADR-0014 row is
fixed in S1.

---

## Order and parallelism

```
G0 (Map/Set) ─ independent, FIRST in the queue, not U-TRACE
G2 (E002) ────────────────┐
G1 (PDR-0010 ratify) ─────┤
                          ▼
T1 ──┬────────────────► T3 ───► T4 ───┬─► T8
T2 ──┤                    ▲       │    │
     ├──► T5 ─────────────┼───────┼────┤
     └──► T7 ─────────────┘(none) │    │
S1 ─ anytime                      └────┘
```

- **Immediately dispatchable, no gates:** T1 ∥ T2, then T5 and T7 — four units of real progress
  while G1/G2 resolve.
- T3 is the only unit that touches the fiber cascade — schedule it when no concurrent session
  holds fiber dispatch.
- `dispatch.rs` appears in five write-sets; the touched functions are disjoint
  (`runtime_error` / `compiler_error` / cascade / switch site / message sites) but units editing
  it do not run concurrently — pipeline them.

## Named debt carried forward (recorded, not fixed here)

- `spans` representation: delta-encoded line table + binary search behind `span_at`
  (IS §2.2; measured 2:1). Migration is now contained by construction.
- Class B hint provenance (receiver-expression spans) — IS §10.
- Class C value provenance (`receiver is None because …`) — own decision record if ever.
- `@native` anchor registry (native.md follow-up) — T4 consumes it through `anchor_of` when it
  exists.
- Reporter/sink injection for embedders (followups §6) — untouched by this plan, unblocked by
  it (all rendering already flows through `RenderConfig`-carrying entry points).
- REPL `where` — deferred on ADR-0050 §7 (catalog §8).
