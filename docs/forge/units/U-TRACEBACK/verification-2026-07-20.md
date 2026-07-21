# U-TRACE verification report — 2026-07-20

Adversarial re-verification of every claim in [`tracing.md`](../../deferred/tracing.md) and the
U-TRACE handoff prompt, performed before the implementation spec was written. Method: each claim
re-derived from the tree at HEAD (post-PDR-0007/0008), reproduced with the built binary where
behavioral, and two claims delegated to independent verifier agents with instructions to refute.

**Headline: the tree moved under the audit.** PDR-0008 (Accepted, shipped `dcc4420`+`8466867`)
already wired `cmd_run` through both reporters and fixed two of the four `runtime_error` defects.
Five of the handoff's "personally verified, treat as CONFIRMED" claims are now stale. This is the
`landed-state-claims-go-stale` failure mode, again — the audit was correct when written and wrong
by the time it was read.

## 1. Claims from the handoff marked CONFIRMED — re-checked anyway

| Claim | Verdict now | Evidence |
|---|---|---|
| `cmd_run` bypasses the diagnostic path (`eprintln!("{e}"); exit(1)`) | **STALE — fixed** | `cli.rs:166-180` now calls `vm.compiler_error(err)` and `vm.runtime_error(err)`; comment cites PDR-0008 |
| `compiler_error` is an empty stub | **STALE — fixed** | `dispatch.rs:180-182` calls `print_compile` (message-only, span-less; see followups §4) |
| `interpret_source` called only from tests/benches | **STALE — narrowed** | Still true for the CLI (`cmd_run` does not call it), but it now reports on both paths; duplication recorded in `repl-diagnostics-and-limits-followups.md` §5 |
| Empirical: bogusSelector prints bare message, no frames | **STALE — changed** | Same repro now prints a `Traceback` header + per-frame caret blocks (see §2 defect table for what is still wrong with it) |
| `print_rt`/`SourceLoc` exist; `print_frame` half-commented | CONFIRMED | `diagnostics.rs:109-135`; `print_frame` computes then discards `module_name`/`method_name`, prints no `File "...", line N` header |
| miette unused repo-wide | CONFIRMED | zero `use miette`/`miette::` in any `.rs`; incumbent is `color_print::ceprintln` (`diagnostics.rs:2`) |
| Zero hits for `NO_COLOR`/`is_terminal`/`atty` | CONFIRMED | also confirmed empirically: piped stderr is full of SGR escapes |
| `SourceRange` = `CopyRange<usize>` = 16 B, one per instruction | CONFIRMED | `range.rs:122`, `chunk.rs:47,92` |
| `ClosureObject.module`, module source retention | CONFIRMED, **shape changed** | `ModuleObject.source: Option<Arc<String>>` is now `sources: Vec<Arc<String>>` indexed by `Chunk::source_id` (U-REPL §D2) |
| No tail-call/inlining code | CONFIRMED (unchanged) | |

## 2. MUST-VERIFY items (a–h)

### a. The four `runtime_error` defects

| # | Defect | Verdict | Evidence |
|---|---|---|---|
| 1 | `module_source.unwrap()` panics on `None` | **REFUTED (already fixed)** | `dispatch.rs:150-156`: `module.source_at(source_id).cloned()`, doc comment names the old `.unwrap()` |
| 2 | `spans[ip - 1]` underflow at `ip == 0` | **REFUTED (already fixed)** | `dispatch.rs:136`: `saturating_sub(1)`; fixed by PDR-0007 (its STATUS row names this defect) |
| 3 | `self.frames.clone()` per error | **CONFIRMED** | `dispatch.rs:123`. PDR-0010 §3 dissolves it (capture at the `on` boundary walks borrowed frames; no clone) |
| 4 | Ordering inverted | **CONFIRMED, empirically** | Repro prints innermost frame first under a "most recent call last" header; `print_rt`'s doc (`diagnostics.rs:108`) demands caller→callee input, `runtime_error`'s `.rev()` (`dispatch.rs:123`) delivers callee→caller. **The code is wrong, not the doc** — both the doc and the locked Python-ordering decision agree. Also: the error message renders at the *top*, and every frame gets a caret block — both contradict the locked decisions (message at bottom, caret innermost-only) |

Current empirical output (repro: `class T { a { self.b } b { self.c } c { 1.bogusSelector } }` + `T.new().a`):
header, then message, then four caret blocks newest-first, no frame names, no module names, ANSI
escapes unconditionally. Exit 1 (not the `ExitCode::RuntimeError = 70` that `interpret.rs:22-30`
already defines).

### b. Which paths leave module source unrecorded — ENUMERATED

The question dissolved: since U-REPL §D2, **every compiled unit records its source** —
`compile_closure_as` (`interpret.rs:146`) pushes the text into `ModuleObject::sources` and stamps
the returned `source_id` into every chunk it compiles. File mode, `-i` inline, every REPL cell,
and `core.ph` (bootstrap-compiled through the same path) all resolve. `create_module`
(`api.rs:86-92`) seeds `sources` empty, but nothing runs code in a module without compiling into
it first.

Remaining source-less frames: chunks **not** produced by the compiler (default `source_id 0` on a
module with empty `sources`). The only in-tree constructor of such a chunk is a test helper
(`chunk.rs:176`). Degradation is designed for (`SourceLoc.source: Option`, `diagnostics.rs:25`)
and currently unreachable from the CLI. Also found: `register_source`/`SOURCE_MAP`
(`api.rs:103-112`) is a **legacy parallel mechanism with a duplicated double-insert body** —
cleanup candidate, scheduled in the plan.

### c. Map/Set reentrant `hash`/`==` — CONFIRMED, both failure modes

Independent verifier reproduced both:

- **Panic mode:** `panicked at phalcom-core/src/primitive/map.rs:80:51: slot from locate() is
  live` — a raw `.expect()` unwrap, i.e. a **process abort uncatchable by `on(_)`**, one notch
  worse than the recorded "indexing panic" phrasing suggests.
- **Silent-corruption mode:** key `C` vanishes, key `D`'s value silently becomes the value
  destined for `C`, `size` shrinks, exit 0, no error.

`locate()` really does send user `hash` (`map.rs:55`) and `==` (`map.rs:61`) via `send_dynamic`.
The recorded trigger sketch needed corrections: `hash { 0 }` (getter, no parens), and a
reentrancy guard flag — the naive sketch infinitely recurses into PDR-0007's native-reentrancy
limit instead of reaching the stale slot. Repro files preserved in the session scratchpad
(`repro4.ph` panic, `repro6.ph` corruption). Stays **out of U-TRACE**, ranked above it.

### d. Typed-catch reification — CONFIRMED

`error-handling.md:140-148` still presents `on(RangeError)` / `on(DeadFrameError)` as working;
no `RangeError`/`DeadFrameError`/`TypeError` kernel classes exist; every non-`Raise` error wraps
to base `Error`. Now **owned by PDR-0010 §2** (`kind` Symbol), which explicitly closes
followups §2 — but PDR-0010 is Proposed. Until ratified, the spec examples remain false and
unannotated.

### e. "VERIFIED CLEAN" list — spot-checked behaviorally, CONFIRMED-CLEAN

Independent verifier ran a 7-row adversarial matrix (normal / NLR / uncaught raise / caught
raise / NLR-through-`on` / raise-inside-cleanup / NLR-inside-cleanup): `ensure` fired exactly
once on every path; cleanup-raise supersedes the body error; cleanup-NLR wins; NLR passes
through `on` without invoking handlers. Cited line numbers in the audit have drifted
(`ReturnNonLocal` now ~`dispatch.rs:1139-1189`) — drift only, no behavioral defect. Surface
facts confirmed while testing: `Error.new("msg").raise()`, `throw X` sugar, `System.print(_)`,
`e.message`.

### f. Spans-vs-bytecode weight — MEASURED

`size_of::<Bytecode>() == 8`, `size_of::<SourceRange>() == 16` (throwaway test, deleted). The
span table outweighs the code it annotates **2:1**, before counting the other `ip`-indexed
parallel arrays (`caches`, `gcaches`, `chunk.rs:115`). The "very likely outweighs" hedge can be
retired; the debt record and accessor requirement (PDR-0010 §5) stand on a measurement now.

### g. Catalog §1 `help: did you mean 'floor'?` — CONFIRMED WRONG

Number's selector table (`universe/primitives.rs:105-123`) is `+ - * / % < <= > >= negated hash
toString new`; **no `floor`**. Fixed in the catalog this pass: the example now typos `negated`
(a selector Number actually has) so the `help:` line is honest.

### h. Cross-fiber link vs DEC-FIB-A — NO COLLISION

DEC-FIB-A was resolved and its fix has **landed**: the fiber-floor capture + `Call`-mode cascade
is live at `dispatch.rs:325-373`. PDR-0010 §3a already specifies exactly where the trace link
must be captured (per hop, inside the cascade loop, before each `clear()` at
`dispatch.rs:354-356`). What remains is **write-set coordination** — that loop is U-FIBER
territory and U-TRACE-3 must edit it — not a design conflict.

## 3. Findings the audit did not contain

- **PDR-0010 supersedes the locked capture-at-raise decision** (Proposed 2026-07-20, commit
  `2c6e022`): capture at the first `on` boundary, per-hop inside the fiber cascade, record holds
  Symbols + line — never `ObjRef`s — plus `Error#kind`/`#cause`/`#displaced`. It names U-TRACE
  directly: *"U-TRACE's implementation spec must be written against §3 and §4."* It is **not
  ratified** (STATUS.md row: "do not design against it"). This is the unit's governance gate —
  escalated.
- **An exit-code table already exists**: `interpret.rs:22-30` defines BSD-`sysexits` values
  (0/1/64/65/66/70/74) and `interpret_file` uses them; `cmd_run` exits 1 for everything. README
  §4.6's "specify the exit code table" is half-done — the spec adopts the existing table.
- **`@native` is implemented** (contra the design-session note "not built yet"):
  `attributes.rs:663-702`, spec at `docs/spec/current/decorators/native.md`, fixtures live. The
  anchors⊆bindings invariant test is the recorded follow-up. This unlocks the native-frame
  rendering section of the spec.
- **E002 (fiber-floor upvalue-close crash, the "F1" hazard) is still OPEN** — status header
  confirmed. E001 (GC ensure temp-root UAF) is fixed (`cdd2117`, `temp_roots` built). The
  sequencing hazard therefore reduces to E002 alone, and it sits exactly on the fiber-failure
  path the traceback will run on. Plan gates on it.
- `bin/phalcom/main.rs:1` is `#![allow(warnings)]` and `disasm.rs` carries three allow's — the
  observability unit inherits a hygiene debt in its write-set.
- `vm-trace`'s cost is documented in-tree: 18.2% of arith wall-clock with subscribers OFF
  (`phalcom-core/Cargo.toml:20-26`, citing perf-log 003) — the per-opcode `cfg` gate must stay.

## 4. What did NOT survive, in one list

From the handoff's claims: defects a1 and a2 (already fixed), the `cmd_run` bypass (fixed), the
empty `compiler_error` (fixed), the empirical no-traceback repro (now renders, differently
wrong), the "module source is None in REPL/-i/core.ph" hypothesis (all three record source now),
the catalog's `floor` suggestion (no such selector), the capture-at-raise locked decision
(superseded by Proposed PDR-0010 — pending ratification), and "@native is not built" (it is).
