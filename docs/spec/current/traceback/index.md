# Traceback & diagnostic rendering

Specification for how Phalcom reports a failure to a human: runtime tracebacks, compile and
syntax diagnostics, execution tracing, and disassembly.

- [`u22-seq-spec.md`](implementation-spec.md) — the dispatch-ready implementation spec
  (renderer ruling, walk/styler architecture, capture, native/@native frames, style guide,
  did-you-mean, observability).
- [`err-plan.md`](plan.md) — dependency-ordered units with write-sets, edges, and gates.
- [`verification-2026-07-20.md`](verification-2026-07-20.md) — adversarial re-verification of
  the audit; lists every claim that did not survive.
- [`output-catalog.md`](output-catalog.md) — every rendering surface, by example.
- [`color.md`](color.md) — the color scheme: semantic roles, palette discipline, per-surface use.
- [`../../deferred/tracing.md`](../../deferred/tracing.md) — the U-TRACE audit and continuation
  prompt: what exists, what is unwired, what is broken.
- [`../../deferred/error-handling-followups.md`](../../deferred/error-handling-followups.md) —
  unowned error-handling defects found alongside.

**Status:** specified — implementation not started. §3.1 (renderer) is **ruled** —
[PDR-0014](../../pdr/0014-diagnostics-renderer-is-in-house.md), Accepted. The capture row
in §2 below is **superseded by
[PDR-0010](../../pdr/0010-errors-carry-structure-and-cheap-origin.md) §3, ratified
2026-07-20**; the normative `kind` table is
[`u22-seq-spec.md`](implementation-spec.md) §8.1. Parts of `tracing.md` went stale when
PDR-0008 landed; trust [`verification-2026-07-20.md`](verification-2026-07-20.md) over it where
they disagree.

---

## 1. Scope

One primitive with several consumers:

```
              ┌── traceback renderer (human)
StackWalk ────┼── traceback renderer (json)
              ├── fiber switch log
              └── REPL `where`          (deferred — see catalog §8)

Chunk walk ───── disassembler
```

The walk is the primitive. Every formatter is a consumer. Building the formatter *as* the
primitive is the failure mode — it forces a rewrite the moment a second consumer appears.

Not in scope: `Error.stackTrace` surface reflection (backlogged; rides on the compact capture
record), a debugger protocol, profiler integration.

---

## 2. Decisions locked

Carried from the U-TRACE design session. Do not re-litigate.

| Decision | Ruling |
|---|---|
| Frame ordering | Python's — most-recent-call-last, error at the bottom |
| Caret block | Innermost frame only, never all frames |
| Core frames | Elided by default with a count; `--trace-core` expands |
| Fiber boundary | Traceback **chains** across the floor with a spawn-site link; does not stop |
| Primitive shape | Walkable live stack object; formatter is a consumer |
| Capture timing | ~~Compact record at **raise**~~ **Superseded by PDR-0010 §3 (ratified 2026-07-20)**: capture at the first `on` boundary / per-hop in the fiber cascade; record holds Symbols + line, never `ObjRef`s (PDR-0010 §4) |
| Frame granularity | **Logical** frames, 1:many expansion from day one |
| Trace stability | Golden fixtures assert fields via JSON stream; human layout explicitly unstable |
| Fiber switch log | No `cfg` gate — cold path |

Two of these carry non-obvious rationale worth restating:

**Capture at raise, resolve at render.** `.attempt()` and `on(_)` handlers already exist
(`core.ph:1427`). Walk-time-only capture loses the origin of every *caught* error, because the
frames are gone by the time anyone asks. Capturing compact — ids and offsets, no strings, no
source resolution — is cheap enough to leave always-on, and it removes the
`module_source.unwrap()` panic by construction: there is no source to unwrap at capture time.

**Logical frames from day one.** There is no inlining and no TCO in the tree today, which is
exactly why the API is free to model 1:many now and expensive to retrofit later. V8 and the JVM
both retrofitted inline-frame expansion and both found it invasive. Precedent that the concern is
real here: superinstruction fusion already had to solve span-fidelity-under-transformation
([dispatch.rs:538-543](../../../phalcom-core/src/vm/dispatch.rs)).

---

## 3. Open decisions

### 3.1 Renderer — miette or color-print  ✅ RULED

**Ruled 2026-07-20: option (b), extended with a named style layer; miette leaves the
workspace.** Full grounds in [`u22-seq-spec.md`](implementation-spec.md) §1 — the short
version: most surfaces (frame lines, fiber log, disasm, JSON) are not miette-shaped, so (a)
still means two renderers; `color.md`'s palette discipline is easier to own than to impose on
miette; and the genuinely hard part (multi-label spans) is bounded at two labels. The catalog's
`╭─ │ · ╰──` style is kept and implemented in-house. Original analysis preserved below.

- `miette` is a declared workspace dependency and **nothing imports it**. Zero `use miette` /
  `miette::` across the repo. `CLAUDE.md` names "thiserror + miette" as the convention;
  that half of the convention is aspirational.
- The incumbent is `color_print::ceprintln` with inline markup
  ([diagnostics.rs:2](../../../phalcom-core/src/diagnostics.rs)), driving hand-rolled
  `print_line_information`.

The catalog's `╭─ │ · ╰──` look is miette's house style. Two paths:

- **(a) Wire miette for real.** Get multi-label spans, help/note slots, and severity for free.
  Cost: changes how every diagnostic prints, moves the entire negative-fixture corpus, and adds
  a second color system alongside `color-print`.
- **(b) Extend `print_line_information`.** Keeps one color system and one corpus. Cost: hand-roll
  multi-label spans, which is the part of miette that is genuinely hard.

Any prior doc or decision asserting diagnostics are "rendered as miette labels" is describing
something that does not exist. Decision 0066 did, and was amended for it (`bb4f365`).

### 3.2 Hint provenance

Catalog §2 shows `receiver is None — \`first\` on an empty List` — a hint explaining *why* the
receiver is `None`, not merely that it is. That requires the VM to know which expression produced
the receiver and reason about it. Specify what class of hint is derivable from spans plus the
opcode alone, and what would need dataflow the compiler does not currently keep. Do not promise
the hard ones in v1.

### 3.3 Remaining opens carried from `tracing.md`

- Does U-TRACE get its own ADR, or ride an existing one? If new: flip `docs/adr/STATUS.md` the
  same pass.
- Which paths leave `ModuleObject::source` as `None` (REPL? `-i` inline? core.ph bootstrap?) —
  determines where source echo degrades to bare `file:line`.
- Does the cross-fiber spawn-site link collide with U-FIBER's floor-capture ownership (DEC-FIB-A)?
- `spans` is 16 bytes/instruction, one per instruction. Record as named debt with a migration
  shape and consume it through an accessor, not `spans[ip]` at N call sites.

---

## 4. Rendering rules — unspecified today, all of it

None of this exists in the tree. Every item is a real gap a renderer must answer.

### 4.1 Color and TTY

Zero hits for `NO_COLOR`, `is_terminal`, or `atty` anywhere in the repo. `ceprintln!` emits
escapes unconditionally. Required:

- `--color=auto|always|never`, defaulting to `auto`.
- `auto` = color only when stderr is a TTY.
- Honour the `NO_COLOR` environment variable (any non-empty value disables).
- Piping to a file or a test harness must produce clean text with no escapes. Today it does not,
  which silently makes every diagnostic byte-fragile in fixtures.

Which elements get which colors is specified separately in [`color.md`](color.md). The governing
rule there: color is emphasis, never information — every distinction must survive
`--color=never`.

### 4.2 Unicode and alignment

The caret underline must line up with the source above it. Four things break that, none handled:

- **Tabs** in source. Column arithmetic on bytes puts the caret in the wrong place. Expand tabs
  to a fixed width before measuring, and expand them identically in the echoed line.
- **Wide characters** (CJK, emoji). One `char` is two terminal columns. Measure display width,
  not `char` count.
- **Combining marks.** Zero-width; must not advance the caret.
- **Non-UTF8 terminals.** Provide an ASCII box-drawing fallback (`+- | ^ \`--`) selected by the
  same switch as color, so a dumb terminal degrades rather than garbles.

### 4.3 Width and truncation

- Detect terminal width; fall back to 80 when not a TTY.
- A source line longer than the width truncates with a marker and keeps the span visible —
  window around the span rather than truncating the tail.
- A span wider than the window renders its endpoints with an elision in the middle.

### 4.4 Multi-line spans

A span crossing a line boundary (a multi-line argument list, a string literal with newlines —
Phalcom strings permit literal newlines) cannot render as one underline. Decide: bracket the
region across lines, or anchor to the start line and note the extent.

### 4.5 Deep and repetitive stacks

Runaway recursion produces thousands of frames. Required:

- A frame budget; beyond it, elide the middle and keep both ends. The innermost frames and the
  entry point are what a reader needs.
- Collapse repeats: Python's `[Previous line repeated N more times]` is the precedent, and
  recursion is exactly where a raw dump is least readable.
- Same budget applies to a long fiber chain (catalog §2).

### 4.6 Streams and exit codes

- Diagnostics go to **stderr**, always. The golden corpus and the Wren stdout comparison depend
  on stdout staying byte-exact — the same reason `opcode_stats::dump` already writes to stderr.
- Specify the exit code table. Today an uncaught runtime error exits 1 via a bare
  `std::process::exit(1)` in `cmd_run`. Compile errors, syntax errors, and runtime errors are
  arguably distinguishable; decide, then honour it everywhere.

### 4.7 Message style guide

There is no rule today, and the drift is already visible: `RuntimeError::UndefinedVar` renders
`` Undefined variable `x` `` while the live path (`RuntimeError::Message`, `dispatch.rs:664`)
renders `Undefined variable 'x'.` — different quoting, different trailing punctuation. One is
dead code, which is how the divergence survived.

Pick and enforce: sentence case or lower; trailing period or none; backtick or single-quote for
code fragments; how a selector is written (`price` vs `price()` vs `'price'`); how a receiver is
described. The style rule is what keeps a hundred `#[error(...)]` strings looking like one
language rather than five contributors.

Errors must never leak Rust internals. `RuntimeError::Internal` currently renders
`NewInstance operand is not a class: Number(5.0)` — raw enum `Debug`, on a path a user reaches
with `let Foo = 5; Foo.new()`, and mislabelled "Internal" as though it were a VM bug.

### 4.8 did-you-mean

Absent entirely from `object_does_not_understand`
([primitive/object.rs:230-251](../../../phalcom-core/src/primitive/object.rs)). Specify:

- Distance metric and threshold — Levenshtein ≤2 is the common choice; scale with token length so
  short selectors do not match everything.
- Candidate set: the receiver's class and its ancestors. Whether to include the metaclass.
- At most one suggestion. Two suggestions means the metric is not confident and the hint is noise.
- Case-insensitive and transposition matches should rank above pure edit distance — they are
  overwhelmingly what a typo actually is.

Applies to undefined variables and unknown class names too, not just selectors.

---

## 5. Testability without freezing the format

The tension: an untested renderer rots, and a byte-asserted renderer can never change.

- Human layout is **explicitly unstable**. No fixture asserts it byte-exact.
- Structure is the contract. Tracebacks assert the **frame sequence** — module, name, line per
  frame. Trace events assert **fields** via `--trace-format=json`.
- One or two snapshot tests may cover the pretty renderer as a canary, marked as intentionally
  churn-prone so a diff there reads as "review the look", not "regression".

The `--trace-format=json` stream exists for this reason, not for external consumers.

---

## 6. Sequencing hazard

`runtime_error` dereferences `heap.closure(frame.closure)` while walking frames. A stale `ObjRef`
from either the GC `ensure` temp-root UAF or the F1 fiber-floor upvalue-close crash turns the
traceback itself into the crash site — a panic at `heap/mod.rs:188` ("dangling ObjRef"),
converting a clean error into a hard crash at exactly the moment the user most needs a message.

Fix those before the traceback runs on every error path.
