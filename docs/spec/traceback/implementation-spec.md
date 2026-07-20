# Traceback & diagnostics — implementation spec

Dispatch-ready specification for the traceback/diagnostics/observability unit. Written against
HEAD after PDR-0007/PDR-0008 landed; every `file:line` verified 2026-07-20 (see
[`verification-2026-07-20.md`](verification-2026-07-20.md)). Companion documents:
[`README.md`](README.md) (scope + decisions), [`output-catalog.md`](output-catalog.md) (targets
by example), [`color.md`](color.md) (roles + palette), [`plan.md`](plan.md) (unit order,
write-sets, edges).

**Governance gate:** §4 (capture) and §8 (kind/messages) are written against
[PDR-0010](../../decisions/0010-errors-carry-structure-and-cheap-origin.md) §2–§5, which is
**Proposed**. Per the decisions-folder rule, no implementation of those sections may start until
PDR-0010 is ratified (or amended-and-ratified). Every other section stands on Accepted records
and may proceed. If PDR-0010 is *rejected*, §4 falls back to the original capture-at-raise
ruling in `tracing.md` — and its `attempt()` cost must then be re-priced, because PDR-0010's
refutation of that ruling is priced and specific.

---

## 1. Ruling: the renderer is ours — extend, don't adopt miette

**README §3.1 is hereby ruled: option (b), extended with a named style layer. miette is dropped
from the workspace.**

Grounds, in decreasing weight:

1. **Most of this unit's output is not a diagnostic in miette's sense.** Traceback frame lines,
   fiber-switch log lines, disassembly, the JSON stream — none of these fit a
   `Diagnostic`-trait/`Report` shape. miette could only ever own the caret block, so adopting it
   still leaves a second hand-rolled renderer for everything else. Two systems permanently, which
   is the exact cost the option-(a) analysis attributed to (b).
2. **[`color.md`](color.md) is a hard spec miette only approximates.** 16-ANSI-only, no
   backgrounds, role-stable colors across *all* surfaces including disasm and the fiber log,
   `--trace-format=json` forcing color off, a single ASCII-fallback switch shared with the box
   glyphs. Owning the styler makes each of these one line; theming miette makes each a fight
   with `GraphicalReportHandler`'s defaults.
3. **The hard part of miette is bounded here.** The catalog needs at most two labels on one
   snippet (§6: opener + expected-closer). A two-label caret renderer over one source window is
   an afternoon of careful column arithmetic — specified precisely in §3 — not a re-derivation of
   miette.
4. The corpus argument is weaker than recorded: golden fixtures assert **fields** (locked), so
   the corpus survives either choice; only the handful of intentionally-churn-prone snapshot
   canaries move.

Consequences: remove `miette` from `[workspace.dependencies]` and from `CLAUDE.md`'s
"thiserror + miette" convention line (it has been aspirational since it was written; decision
0066 was already amended for this once, `bb4f365`). `color_print::ceprintln` call sites migrate
to the style layer as each surface is rebuilt; `color-print` leaves the tree when the last one
does. Add `unicode-width` (workspace-pinned) for display-width measurement — it is the same
crate the REPL stack already pulls transitively via `reedline`.

The catalog's `╭─ │ · ╰──` look is kept — it is a good look; we implement it, not import it.

---

## 2. Architecture — one walk, one styler, many consumers

```
                        ┌───────────────────────── diagnostics::style  (roles → SGR)
                        │
vm::walk::StackWalk ────┼── diagnostics::traceback   (human)
  (FrameView iterator)  ├── diagnostics::traceback   (json line)
                        ├── fiber-switch trace events
                        └── REPL `where`             (deferred, ADR-0050 §7 gate)

chunk::span_at(ip) ─────┬── the walk (line resolution)
  (THE spans accessor)  └── disasm (recursive)

diagnostics::caret ──────── every snippet: traceback innermost frame,
                            syntax, compile, contract diagnostics
```

Modules (all in `phalcom-core`, all fully rustdoc'd per
`docs/rust-documentation-guidelines.md`):

- `src/vm/walk.rs` — `StackWalk<'vm>`, `FrameView`, logical-frame expansion. **The primitive.**
- `src/diagnostics/style.rs` — `Role`, `ColorMode`, `GlyphSet`, `Styler`.
- `src/diagnostics/caret.rs` — the snippet/caret renderer (`Snippet`, `Label`).
- `src/diagnostics/traceback.rs` — human + JSON traceback renderers; replaces
  `print_rt`/`print_frame` outright.
- `src/diagnostics/suggest.rs` — did-you-mean engine.
- `src/diagnostics/mod.rs` — keeps `line_col`, `print_parse`/`print_compile` (restyled), and the
  legacy `SOURCE_MAP` until §7 deletes it.

### 2.1 `FrameView` — the walk's yield type

```rust
pub struct FrameView {
    pub module: Symbol,        // module name symbol
    pub name: FrameName,       // Main | Method(Symbol) | Block { enclosing: Symbol } | Native(Symbol)
    pub line: u32,             // resolved via span_at(ip)
    pub span: SourceRange,     // byte span, for the caret block
    pub source: Option<Arc<String>>,  // chunk's own source text (source_id-resolved)
    pub is_core: bool,         // module == core module → elidable
    pub fiber: u32,            // owning fiber's display id (§6)
}
```

- The walk iterates `vm.frames` **oldest→newest** (no `.rev()` — the renderer wants Python
  order natively; defect a4 dies here, not in a patch).
- `FrameName` renders in **selector shape**: `Cart.total`, `Cart.sum(_)`,
  `<block in Cart.sum(_)>`, `<main>` — `foo` and `foo(_)` are different methods and the trace
  must say which. Names resolve through the interner at render time; the walk itself carries
  only `Symbol`s.
- **Logical frames, 1:many from day one (locked):** `StackWalk` yields via an internal
  `expand(physical) -> impl Iterator<Item = FrameView>` seam that today returns exactly one view
  per `CallFrame`. Superinstruction fusion already proved span-fidelity-under-transformation
  matters (`dispatch.rs:538-543`); this seam is where a future inliner emits its N views.
- `is_core`: module identity comparison against the core module handle — **not** a name check.

### 2.2 The spans accessor (PDR-0010 §5, measured debt)

```rust
impl Chunk {
    /// Span of the instruction at `ip`. Clamps: `ip == 0` → first span,
    /// past-end → last span. Never panics, never underflows.
    pub fn span_at(&self, ip: usize) -> SourceRange;
    /// Line of `span_at(ip).start`, resolved against this chunk's source.
    pub fn line_at(&self, ip: usize, source: &str) -> u32;
}
```

Every consumer — the walk, disasm, trace events — goes through `span_at`. Direct
`chunk.spans[...]` indexing outside `chunk.rs` is a review-blockable offense from this unit on.
Rationale is now **measured**: `size_of::<SourceRange>() == 16` vs `size_of::<Bytecode>() == 8`
— the table is 2× its code before counting the sibling `ip`-indexed arrays. The recorded
migration (delta-encoded line table + binary search) changes `span_at`'s body and nothing else.

---

## 3. The rendering substrate

### 3.1 Styler — roles, not colors

`Role` is the closed enum of the 13 roles in [`color.md`](color.md) §2 (severity.error/warn/help,
location, identifier, rail, line-number, source, span.primary, span.secondary, label, elision,
chain). `Styler::paint(role, &str) -> Cow<str>` is the **only** place SGR bytes are produced.
Rules enforced by construction:

- 16 ANSI indices + bold/dim/italic only. No 256-color, no truecolor, no backgrounds, no pure
  black/white — the type of the color table makes these unrepresentable.
- `ColorMode::Never` returns the input unchanged; every distinction must survive it (the
  color-off render of each catalog example is a required test).

### 3.2 Color and TTY resolution

```
--color=auto|always|never   (default auto)
NO_COLOR (any non-empty)    → never, unless --color=always given explicitly
auto                        → color iff stderr is a TTY (std::io::IsTerminal — no atty dep)
--trace-format=json         → the JSON stream is ALWAYS unstyled, regardless of everything above
```

Precedence: explicit `--color=always|never` > `NO_COLOR` > auto-detection. Detection happens
once at CLI startup into a `RenderConfig { color: ColorMode, glyphs: GlyphSet, width: u16 }`
passed down (no global mutable state; the REPL builds its own against stdout).

### 3.3 Glyphs and the `--plain` axis

Resolves [`color.md`](color.md) §6's open: **two orthogonal switches, one umbrella.**

- `GlyphSet::Unicode` (default): `╭ │ · ╰ ─ ┬ × ⤷ └`.
- `GlyphSet::Ascii`: `+- | : ` + backtick-dash rails, `x` for the error marker, `->` for chain.
- `--plain` = `GlyphSet::Ascii` **and** `--color=never` in one flag (the friendly umbrella).
  An explicit `--color=always --plain` yields ASCII glyphs *with* color — the axes stay
  separable for whoever needs exactly that.
- Piped (non-TTY) output keeps Unicode glyphs — fixtures assert fields, and modern CI is UTF-8.
  ASCII is an explicit opt-in, not a TTY-detection consequence.

### 3.4 Caret renderer — the column-arithmetic contract

`Snippet::render(source, labels: &[Label], config) -> String` where
`Label { span, text, kind: Primary | Secondary }` (≤ 2 labels in v1; the type takes a slice so
a third is a data change).

Alignment rules (README §4.2, made concrete):

- **Measure display columns, never bytes, never `char`s.** `unicode_width::UnicodeWidthStr` on
  the *expanded* line prefix gives the caret's column.
- **Tabs expand to the next multiple of 4** in the echoed line, and identically in the measure —
  the echoed text and the underline are computed from the same expanded string.
- **Combining marks** are width-0 under `unicode-width` and therefore do not advance the caret —
  free once measurement is width-based.
- **Wide (CJK/emoji) spans** underline their full display width (a 2-column char gets 2 underline
  cells).
- **Width windowing** (README §4.3): terminal width from the TTY (fallback 80, also 80 when not
  a TTY). A line wider than the window is cut to `[…window…]` centered on the primary span, with
  `…` markers on the trimmed side(s); a span wider than the window renders both endpoints with a
  `…` elision in the middle of the underline.
- **Multi-line spans** (README §4.4, ruled): v1 **anchors to the start line** and appends the
  extent to the label — `╰── string literal opened here (spans 4 lines)`. Bracketing the whole
  region is a v2 upgrade the `Label` type does not preclude. Reachable today: Phalcom strings
  permit literal newlines.

`print_line_information`'s known off-by-one behaviors (0-based column in the header, caret
misplacement on multi-line spans — visible in the §verification empirical output as
`Error at 2:0`) die with it; the caret renderer states 1-based line:col.

### 3.5 Streams and exit codes (README §4.6, ruled)

- **All diagnostics and all trace streams → stderr.** stdout stays byte-exact for the golden
  corpus and Wren comparisons (same rule `opcode_stats::dump` already follows).
- **The exit-code table is the one that already exists** at `interpret.rs:22-30`
  (BSD `sysexits`): 0 success · 1 generic · 64 usage · 65 compile error (incl. syntax) ·
  66 no input · 70 runtime error · 74 I/O error. `cmd_run` currently exits 1 for everything —
  it adopts this table (§7). Exit 101 is documented as "the VM itself panicked — a Phalcom bug,
  please report", and no Phalcom-authored path may produce it deliberately.

---

## 4. Capture — PDR-0010 §3/§4, applied  ⛔ *gated on PDR-0010 ratification*

What the traceback renders for a *caught-then-rethrown* or *fiber-crossing* error is the capture
record; for an uncaught error it is the live walk. Both produce `FrameView`s.

- **Record shape (PDR-0010 §4):** per frame — module-name `Symbol`, method-name `Symbol`,
  `line: u32`. **Never an `ObjRef`**, never source text. Symbols are GC-immune by construction
  (`gc.rs:77`). Stored beside `RuntimeError::Raise`'s existing `rendered` snapshot
  (`error.rs:107-115`).
- **Capture site (PDR-0010 §3):** top of `block_on`'s `Err` arm (`block.rs:247-292`), **after**
  the surface `Error` is in hand, **before** `unwind_to` — the only point the raise-site frames
  still exist, on both the matching and non-matching branch. Cost is bounded by protected-region
  depth; `attempt()`'s raise-and-catch `Result` channel pays for its own depth only. This
  **dissolves defect a3**: no `frames.clone()` anywhere.
- **Fiber cascade (PDR-0010 §3a):** the spawn-site chain link is captured **per hop inside** the
  `Call`-mode cascade loop (`dispatch.rs:341-370`), before each hop's
  `frames/stack/open_upvalues` `clear()` at `:354-356`. Capturing only at the consuming boundary
  provably loses every intermediate hop's frames. This loop is U-FIBER's landed territory —
  coordination note in the plan.
- **Uncaught errors** need no record: frames survive propagation (verified in PDR-0010 — the only
  destroyers are `Return`'s pop and `unwind_to`), so `runtime_error` walks live state at the
  report site, which PDR-0008 §2 already places before any unwind on both the REPL and file
  paths.
- `Error#cause` (user-set), `Error#displaced` (ensure-supersession, set by exactly one runtime
  rule) and `Error#kind` are PDR-0010 §1/§2 surface work; the traceback renders `cause` chains
  with the same linking grammar as fiber chains (§5.3) and renders `displaced` as a titled
  secondary traceback: `note: this error displaced an earlier one during cleanup:`.

## 5. The traceback renderer

### 5.1 Human format — catalog §1, normative rules

Locked decisions applied: Python order (oldest first, error at bottom), two-line frame form,
caret block on the innermost frame only, core frames elided with a count (`--trace-core`
expands), `help:` only when §9 clears its threshold.

```
Traceback (most recent call last):
  shop.ph:7   in <main>
      cart.total
  [2 core frames elided — pass --trace-core to expand]
  shop.ph:3   in <block in Cart.sum(_)>

  × 1 does not understand 'negatd'
   ╭─[shop.ph:3:48]
 3 │   sum(items) { items.fold(0) { acc, it => acc + it.negatd } }
   ·                                                    ───┬──
   ·                                                       ╰── Number has no method 'negatd'
   ╰────
  help: did you mean 'negated'?
```

- A frame with `source: None` prints its header line only (no echoed source) — degradation, not
  failure. (Reachable today only from hand-built chunks; see verification §2b.)
- The innermost frame's echoed source is replaced by the caret block (no duplicate echo).

### 5.2 Deep and repetitive stacks (README §4.5, ruled)

- **Frame budget 40.** Beyond it: keep the oldest 15 and newest 15, elide the middle as
  `[… 973 frames elided …]` (elision role, dim italic).
- **Repeat collapse first, budget second:** a run of >3 consecutive frames identical in
  (module, name, line) collapses to one occurrence plus
  `[previous frame repeated 996 more times]` — recursion is the case where budgets alone stay
  unreadable. A `DepthExceeded` traceback (PDR-0007) therefore renders in ~10 lines.
- Fiber chains: budget 8 links; longer chains keep the first 2 and last 4 with a
  `[… 12 fiber hops elided …]` link line.

### 5.3 Chains

Fiber boundary (catalog §2): each side a complete traceback, joined by
`⤷ raised inside fiber #3, spawned at job.ph:1` (chain role, magenta). `Error#cause` chains use
the same grammar with `⤷ caused by:`. Only the innermost side carries the caret block and
message; per PDR-0010 §1 the fiber link is a rendered frame-record annotation, **not** a second
reachable chain.

### 5.4 JSON traceback

An uncaught error under `--trace-format=json` emits **one line** on stderr:

```json
{"ev":"traceback","error":{"message":"1 does not understand 'negatd'","kind":"doesNotUnderstand"},"frames":[{"module":"shop","file":"shop.ph","line":7,"name":"<main>","core":false,"fiber":1},…]}
```

Frames in the same oldest-first order. This is the fixture contract (§11); the human layout is
explicitly unstable.

### 5.5 Native frames — errors with no `.ph` frame to point at

A primitive never pushes a `CallFrame`, so today an error raised *inside* the floor (arity/type
guards in `primitive/*.rs`) renders as if the caller itself failed, with no mention of the
native selector. Two-part fix:

1. **Native send context.** The dispatch send path records the in-flight selector + receiver
   class (two `Symbol`s on the VM, overwritten per native call — no allocation, no cost beyond
   two stores on the *native* path only). When a primitive returns `Err`, `runtime_error`/the
   capture site synthesizes one logical frame:
   `  [native]    in List.fold(_,_)` — rendered with the `identifier` role, no source echo, and
   never elided as a core frame (it is the failure site).
2. **`@native` anchors.** `@native` is implemented (`attributes.rs:663-702`,
   `decorators/native.md`): a dropped `.ph` member marking where the Rust implementation is
   anchored for tooling. When the anchor registry exists (native.md's recorded follow-up — the
   anchors⊆bindings invariant test builds it as a byproduct), the native frame line upgrades to
   `  [native]    in List.fold(_,_)   anchored at core.ph:212` and `--trace-core` echoes the
   anchor's `.ph` body under it, exactly like a real frame. The renderer consumes the registry
   through one `fn anchor_of(selector, class) -> Option<(Symbol, SourceRange)>` seam and renders
   the bare form when it returns `None` — so U-TRACE does not block on the registry unit.

This is the "@native for dev experience" payoff: the user sees *which* floor method failed and
can jump to readable `.ph` source for it, while the implementation stays native.

### 5.6 Decorator and contract diagnostics

- **Attribute compile errors** (`attr.unknown`, illegal target, e.g.
  `decorators_native_illegal_on_class.ph`) carry spans already — they ride §7's compile-error
  span plumbing and render as catalog §7 with the attribute name as the primary label.
- **Contract violations** (`@requires`/`@ensures`/`@invariant`, woven guards): the raise
  happens inside woven bytecode whose spans point at the *contract expression* — so the caret
  block lands on the `@requires(…)` condition itself, with the callsite as the frame above.
  Required label text: `× contract violated: @requires(amount > 0)` + secondary label on the
  call span `╰── called from here with amount = -3` **only when** the argument is nameable from
  the guard's own slots (Class A hint, §10 — the woven guard has the value on its stack; no
  dataflow needed). `kind: #contractViolation`.

---

## 6. Fiber identity

`FiberObject` gains `seq: u32` — a per-VM monotonic display id assigned at spawn; the root fiber
is `#1`. `ObjRef` handles are unstable (recycling) and unreadable; every rendered surface
(traceback chains, fiber log, JSON stream) uses `seq`. Two stores and a counter; no lookup
table.

---

## 7. One register for every failure — entry-path unification

Resolves followups §5 by **deleting the duplication**: `cmd_run` is rewritten to call
`vm.interpret_source(module, &source)` and map the returned `PhError` to the §3.5 exit table.
`interpret_source` (`interpret.rs:203-215`) is already the single place that reports both
compile and runtime failures; the CLI stops owning reporting logic entirely.

- **Syntax errors, one register (catalog §6):** `parse_source`'s `map_err` in
  `compile_closure_as` (`interpret.rs:140-144`) is today the *only* caller of `print_parse` on
  the run path — after unification, run and check share it by construction. `print_parse` and
  `cmd_check`'s text mode both migrate to the §3.4 caret renderer (two labels: opener +
  expected-closer, when the parser provides both spans; single label until it does).
  `cmd_check --format=json` is unchanged (already field-asserting).
- **Compile errors get spans** (followups §4, catalog §7): `compiler_error` gains the
  `source_id`/module context it lacks — signature becomes
  `compiler_error(&mut self, err: PhError, module: ObjRef, source_id: u32)` (both callers have
  both values in hand: `interpret.rs:208`, `repl.rs:116`). `CompilerError` variants that carry a
  `SourceRange` render the caret block; variants without spans render message-only until they
  grow one. **Priced separately** (the catalog's cost warning): adding spans to the remaining
  variants is compiler work, scheduled as the plan's tail item, and each variant's fixture moves
  the moment its rendering does.
- **`SOURCE_MAP` dies.** `register_source` (`api.rs:103-112`, with its duplicated double-insert
  body) and the `SOURCE_MAP` lazy-static are superseded by `ModuleObject::sources` everywhere;
  the one live reader migrates to `source_at`.

## 8. Message style guide (README §4.7, ruled)  ⛔ *`kind` values gated on PDR-0010*

The rule set — enforced by review and by one fixture per rule class:

1. **Sentence shape:** start lowercase unless the first token is an identifier/class name; no
   trailing period on single-sentence messages; second sentences only in `help:`/`note:` slots.
2. **Code fragments in single quotes**, matching the live majority (`'price'`,
   `Undefined variable 'x'`). Backticks are for Markdown docs, not terminal output. The dead
   `UndefinedVar` variant's backtick form loses.
3. **Selectors print in signature shape:** `'sum(_)'`, `'fold(_,_)'`, `'total'` — arity is part
   of the name in this object model.
4. **Receivers render as `toString` output truncated to 40 display columns**, never Rust
   `{:?}`. A `Debug`-formatted `Value` in a user-reachable message is a defect
   (`NewInstance operand is not a class: Number(5.0)` — `dispatch.rs:1009/:1012`, plus the four
   field-access sites `:956/:959/:984/:987` — all rewritten).
5. **`RuntimeError::Internal` is reserved for genuine VM invariant breaks**, renders with prefix
   `internal error (this is a Phalcom bug, please report): …`, and is never constructible from
   well-typed user input. The five sites above become ordinary typed errors
   (`kind: #type`, message per rule 4: `cannot instantiate from 'Foo': it is a Number, not a
   class`).
6. **Dead variants are deleted**: `UndefinedVar` (`error.rs:115`), `UnsupportedOperation`
   (`:117`), `BinaryNotSupported` (`:120`), `UnaryNotSupported` (`:127`). The live undefined-var
   path (`RuntimeError::Message` at `dispatch.rs:664/:716`) becomes a structured variant with
   the variable name as a field (did-you-mean needs it as data, §9). `ZeroDivision` is deleted
   *and* the IEEE-754 rationale moves to a doc comment on `number_div`
   (`primitive/number.rs:104-114`) — an unconstructible variant is a trap, the doc is the part
   worth keeping. (Diverges from tracing.md's keep-with-doc note, deliberately.)
7. **Every message names what the user can act on** — the selector, the variable, the class —
   and `help:` carries the action (`add ')' before the end of the line`), `note:` carries
   context. One `help:` max, one `note:` max, per diagnostic.

## 9. did-you-mean (README §4.8, ruled)

`diagnostics::suggest::best_match(miss: &str, candidates: impl Iterator<Item = &str>) ->
Option<&str>`:

- **Metric:** Damerau-Levenshtein (OSA variant — adjacent transposition counts 1).
- **Threshold by miss length:** ≤4 → distance 1; 5–8 → 2; >8 → 3.
- **Ranking:** case-insensitive-equal beats transposition beats substitution beats
  insertion/deletion at equal distance; then lower distance; then shorter candidate; then
  lexicographic (determinism for fixtures).
- **Emit at most one suggestion, and only when the best is strictly better than the runner-up**
  — two candidates at the same rank means the metric is not confident and the help line is
  omitted (locked: a wrong suggestion is worse than none).
- **Candidate sets:**
  - selector miss (`doesNotUnderstand`, `primitive/object.rs:230-251`): the receiver's class
    method table walked through its ancestors; for a class-side send, the metaclass chain — the
    metaclass question resolves itself because the walk starts at the *receiver's* class, which
    for `Foo.bar` *is* the metaclass. Selector names compare on the base name; arity mismatch
    with an exact base-name match gets its own dedicated hint instead:
    `help: 'sum(_)' exists — did you mean to pass 1 argument?`
  - undefined variable (`dispatch.rs:664/:716`): locals in scope (compiler emits the candidate
    list into the error at compile time when resolvable) + module globals + core globals.
  - unknown class name / unknown import member: the module's binding namespace.
- Applies uniformly; the engine is pure and fixture-tested on its own.

## 10. Hint provenance (README §3.2, ruled by class)

- **Class A — free at the error site (v1):** selector, arity, receiver class, receiver
  `toString`, callsite span, enclosing method, contract-guard operand values (§5.6). Every v1
  hint is Class A.
- **Class B — needs compiler-emitted sub-expression spans (deferred, shaped):** "the receiver
  expression is `rows.first`" requires a receiver-span table per callsite (a second span per
  `Invoke`, compiler-emitted). Recorded as the upgrade path for multi-label runtime carets;
  costs one parallel table only for callsites, not all instructions.
- **Class C — needs value provenance (explicitly NOT promised):** catalog §2's
  `receiver is None — 'first' on an empty List` explains *why* the receiver is `None`. `None`
  is a shared singleton; tagging it with an origin means boxing absence in debug builds — a
  representation change this unit must not smuggle in. v1 renders `receiver is None` (Class A)
  and the catalog example is annotated aspirational. If ever built, it is its own decision
  record.

## 11. Testability (README §5, concretized)

- **Structure is the contract:** traceback fixtures assert the JSON line's frame sequence
  (module, name, line, core, fiber) and error (message, kind); trace fixtures assert event
  fields. Runner: existing golden-lane harness, `--trace-format=json` on stderr, stdout
  untouched.
- **Caret-renderer unit tests** (not goldens): tab expansion, CJK width, combining marks,
  window elision, two-label layout, ASCII glyph set — each a pure `Snippet::render` string
  assertion, stable because the substrate is *ours* and versioned with the tests.
- **Two snapshot canaries** (catalog §1 and §6 renders), marked churn-prone: a diff there means
  "review the look", never "regression".
- **Color-off invariance test:** every catalog example rendered with `--color=never` must
  contain the same character sequence as the styled render stripped of SGR bytes (mechanical
  strip-and-compare), enforcing "color is emphasis, never information".
- **Negative-control rule** (per the U-REPL lesson): every new fixture is first run against the
  pre-fix tree to confirm it fails.

## 12. Observability batch

- **Fiber-switch log** (catalog §4): events `spawn/switch/yield/done/fail` emitted from
  `switch_to_fiber_and_deliver` (`dispatch.rs:387` — the single choke point) plus the spawn and
  completion sites. `tracing::debug!` with **no `cfg` gate** (locked: cold path; the 18.2%
  perf-log-003 figure is about per-opcode callsites, `Cargo.toml:20-26`, and does not apply).
  Labels come from the same walk (`FrameView` of the switch site).
- **CLI:** `--trace=<targets>` (comma list; v1 target: `fibers`; the flag namespace is where
  `gc`, `dispatch` land later), `--trace-format=text|json` (default text), `--trace-core`,
  `--color`, `--plain` — all on the existing `Cli` derive (`cli.rs:12-48`).
- **`main.rs:15` unhardcoded:** the `LevelFilter::OFF` registry is replaced by a filter built
  from `--trace` (OFF when absent — today's behavior — scoped `DEBUG` per target when present).
  The `vm-trace` cfg feature **stays** as the per-opcode gate; `--trace=dispatch` without the
  feature prints one warning naming the feature. While in the file: `#![allow(warnings)]`
  (`main.rs:1`) is deleted and the bin brought up to the documentation mandate.
- **Disasm, recursive** (catalog §3): `disasm.rs` walks `constants` for closure objects and
  recurses with indentation; headers `name slots=N upvalues=M`; constants resolved
  (`<class Cart>`, `Symbol(total)`, nested closures by name); `line N` via `span_at`; `Invoke`
  operands resolved to selector shape; `Closure(_)` annotated `← captures: …` from the upvalue
  descriptors; fused superinstructions render the fusion *and* the dead `Invoke` slot they
  shadow (`dispatch.rs:538-543`) so a reader diffing against `spans` doesn't conclude the table
  is corrupt.

## 13. Sequencing hazards (unchanged in substance, narrowed in scope)

- **E002 is OPEN** (`docs/errors/E002-fiber-floor-upvalue-crash.md`, confirmed 2026-07-20) and
  sits on the exact path §4's cascade capture walks. A stale `ObjRef` turns the traceback into
  the crash site (`heap/mod.rs:188` "dangling ObjRef"). **E002 lands before U-TRACE-3.** E001
  is fixed (`cdd2117`); no longer a gate.
- The Map/Set reentrancy blocker (verification §2c) is **not** this unit and outranks it.
- REPL `where` stays deferred (post-mortem stack retention is the E001 shape; gate on
  ADR-0050 §7 as recorded in catalog §8). `StackWalk<'vm>` borrows; the owned post-mortem
  snapshot type stays unbuilt deliberately.
