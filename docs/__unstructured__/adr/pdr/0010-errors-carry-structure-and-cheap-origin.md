# PDR-0010 — Errors carry structure and cheap origin: one cause chain, a `kind` symbol, incremental capture

- Status: Accepted (ratified 2026-07-20, same day as proposal; ratification ruled that the
  normative `kind` Symbol table and the isA-vs-`kind` usage rule are specified in
  [`docs/spec/current/traceback/implementation-spec.md`](../spec/current/traceback/implementation-spec.md) §8
  rather than by amending this record)
- Date: 2026-07-20
- Related: [ADR-0008](../adr/accepted/0008-layered-exceptions-and-result.md) (layered exceptions +
  `Result`), [ADR-0038](../adr/accepted/0038-amend-floor-admit-block-on-ensure.md)
  (`Block#on`/`Block#ensure` — the unwind path this record instruments),
  [ADR-0050](../adr/accepted/0050-non-moving-mark-sweep-collector.md) §7 (why an off-heap `Value`
  is a UAF), [PDR-0005](0005-resources-are-disposable-handles-not-finalized.md) §4 (use-after-close
  raises — a `kind` consumer), [PDR-0007](0007-bounded-call-depth-and-native-reentrancy.md) §2
  (depth-limit error — another `kind` consumer),
  [`docs/deferred/tracing.md`](../deferred/tracing.md) (the U-TRACE brief this constrains),
  [`docs/deferred/error-handling-followups.md`](../deferred/error-handling-followups.md) §2
  (the reification gap §2 below closes)

## Context

Three error-surface questions are open at once, in three different documents, and each has a
default answer that is cheap now and unfixable later.

**Origin.** `docs/deferred/tracing.md` records a locked decision to capture a compact frame record
**at raise** — module id + closure id + ip per frame — justified because `.attempt()`/`on(_)`
handlers already exist, so capturing during the walk would lose origin for every *caught* error.
The correctness argument holds. The cost was not priced, and Phalcom's shape makes it expensive:

```phalcom
attempt() {
  return { Ok.new(self.call()) }.on(Error) { e => Err.new(e) }
}
```

— [`core.ph:627-629`](../../phalcom-core/core/core.ph). **Every `Result` produced through
`attempt` is a raise plus a catch.** Under capture-at-raise, every such `Err` pays a full frame
walk for an error that is converted to a value immediately and whose traceback nobody reads.

**Structure.** Every non-`Raise` `RuntimeError` is wrapped, on catch, into a generic base `Error`
([`block.rs:253-263`](../../phalcom-core/src/primitive/block.rs), mirrored by `capture_error_value`
at [`dispatch.rs:370-379`](../../phalcom-core/src/vm/dispatch.rs)). `RangeError`, `TypeError`, and
`DeadFrameError` kernel classes do not exist, so the three worked examples at
[`error-handling.md:143-146`](../spec/current/error-handling.md) are false today. Reification into
new classes is deferred by `U-ERR/plan.md:277-281` — and [PDR-0001](0001-classes-are-closed.md)
plus [ADR-0041](../adr/accepted/0041-hierarchy-stability-policy.md) make minting kernel classes
expensive *by design*, so the deferred fix has become more costly since it was deferred.

**Chaining.** `docs/deferred/tracing.md` also locks a cross-fiber traceback link — *"raised in
fiber #N, spawned at file:line"* — citing Python's `__context__`/`__cause__` as precedent. Surface
`Error` has exactly one field, `_message` ([`core.ph:54-61`](../../phalcom-core/core/core.ph)), and
two primitives, `message` and `raise` ([`primitives.rs:354-356`](../../phalcom-core/src/universe/primitives.rs)).
Nothing prevents a native chain and a surface chain from both existing under the same name.

## Decision

### 1. One user-visible chain: `Error#cause`

Surface `Error` gains `_cause` and a `cause` getter. Pure `.ph`, **zero floor delta**.

The rule that keeps it one chain:

> If the user can reach it through a selector, it is `cause`. If it is only ever rendered, it
> belongs to the traceback, not to the error graph.

So U-TRACE's cross-fiber link is a **frame-record annotation rendered inline**, not a second
chain reachable from `Error`.

Python is both the precedent and the warning. PEP 3134 shipped two chains — `__cause__`
(explicit, `raise X from Y`) and `__context__` (implicit, raised-while-handling). Fifteen years
on the distinction is still a recurring confusion, and `raise ... from None` exists solely to
suppress the one nobody asked for. Java's two — `getCause()` and `getSuppressed()` — survive only
because they mean genuinely different things (causality versus a second concurrent failure). Go
collapsed to one (`%w` / `Unwrap`) and has the cleanest story of the three.

**One runtime-set slot beside it: `Error#displaced`.** Phalcom's `ensure` is cleanup-supersedes —
a raising cleanup wins and the body outcome is discarded
([`block.rs:326-340`](../../phalcom-core/src/primitive/block.rs); independently confirmed correct
by the U-TRACE audit's *"VERIFIED CLEAN"* list). The discarded body error is **not** the cause of
the cleanup error; the two failures are independent. Folding it into `cause` would be exactly the
conflation Python avoided by separating `__context__` from `__cause__`.

So `ensure` sets `displaced` on the cleanup error when it supersedes a raising body. This is
Java's `cause`/`suppressed` split, which survives *because* the two mean different things — and
unlike Python's pair, both of Phalcom's are set by one stated rule each, neither by implicit
raise-during-handling machinery.

This case is not hypothetical and is about to get more common:
[PDR-0005](0005-resources-are-disposable-handles-not-finalized.md) §3b makes `close` return
`Result` precisely because close can fail, and §6's canonical pattern is
`{ f.readAll } .ensure { f.close }`. A failing close while the body is already raising is the
exact scenario that forced Java to build suppression, and PDR-0005 §3b cites it by name.

**Two slots, and no third without a superseding record.** `cause` is user-set causation;
`displaced` is set by exactly one runtime rule. Anything else is rendered, not reachable.

### 2. `kind` is a `Symbol` on `Error`, not a class per condition

Every `Error` carries a `kind` holding a `Symbol` — `#range`, `#type`, `#deadFrame`, `#notFound`,
`#permissionDenied`, `#wouldBlock`, `#depthExceeded`, `#useAfterClose`. `capture_error_value`
([`dispatch.rs:370-379`](../../phalcom-core/src/vm/dispatch.rs)) sets it when wrapping a native
`RuntimeError` instead of discarding the variant's identity.

This **closes `error-handling-followups.md` §2** without minting a single kernel class:

```phalcom
{ list[99] }.on(Error) { e => e.kind == #range }        // works on today's machinery
```

Kind-as-data beats kind-as-class when the set will grow, because adding a variant is a data
change rather than a hierarchy change. Rust's `io::ErrorKind` is the one uncontroversial part of
Rust's error story and it is `#[non_exhaustive]` for exactly this reason. Java took the
class-hierarchy route (`FileNotFoundException extends IOException`) and pays a class per
condition. Go took bare strings and had to bolt on `errors.Is` plus sentinel values a decade
later — after production code was already doing `strings.Contains(err.Error(), "not found")`.

The rule matters more here than elsewhere: PDR-0001 rules classes closed and ADR-0041 is a
hierarchy-*stability* policy, so a kernel class per error condition is the most expensive
available answer to the cheapest question.

**Ship `kind` before the first stdlib error surface.** Once errors are string-only, someone
parses the string and it becomes API.

### 3. Capture happens at the first `on` the error meets, bounded by the protected region — not at raise

The frame record is built at the **first `on` boundary the error reaches**, walking
`vm.frames[snapshot..]` immediately inside that boundary's `Err` arm and **before** it unwinds.
Not at the raise site, and not accumulated per frame.

`block_on` snapshots `stack_len`/`frames_len` before running the protected block, and — since
PDR-0007 shipped — calls `unwind_to(stack_len, frames_len)` **before** probing `isA`, on both the
matching and the non-matching branch ([`block.rs:247-292`](../../phalcom-core/src/primitive/block.rs)).
That ordering is load-bearing for PDR-0007 §2: the probe is a full dynamic send needing frames of
its own, so probing on the abandoned stack made a depth-ceiling error uncatchable.

Two consequences for capture:

- **The window is the top of the `Err` arm**, after the surface `Error` is in hand and before
  `unwind_to`. That is the only point at which the raise-site frames still exist.
- **Every `on` truncates, matching or not**, so the *first* boundary the error meets is the only
  one that ever sees those frames. Capture is unconditional there, not conditional on consuming
  the error.

Cost is proportional to **the depth of the protected region** — the frames between the `on` and
the raise site — not to the whole stack. A shallow `attempt()` around a deep computation still
walks that computation; a deep stack with a nearby enclosing `on` walks almost nothing. The saving
over capture-at-raise is everything below the nearest enclosing `on`, and an uncaught error still
yields the full stack, which is the case where the full stack is wanted.

> **Not Python's mechanism, deliberately.** Python appends a frame at each unwind step because
> its unwinding *is* frame-by-frame. Phalcom's is a bulk `unwind_to`, so there is no per-frame
> hook to attach to — an incremental design would require frame-level unwind records the VM does
> not have. Same cost profile, different implementation, and this one needs no new machinery.

**Verified: frames survive propagation itself; only `unwind_to` destroys them.** No error path in
`run_until_inner` touches `self.frames` — every one is a bare `return Err(…)` or `?`. The only
`frames.pop()` is the normal `Return` opcode
([`dispatch.rs:1105`](../../phalcom-core/src/vm/dispatch.rs)) and the only `frames.truncate` is
`unwind_to` ([`dispatch.rs:112`](../../phalcom-core/src/vm/dispatch.rs)). So an error arrives at
its first `on` — and at the fiber floor if it meets no `on` — with the stack at full raise depth.

> **This section was wrong twice; both corrections came from reading the code.** The first draft
> specified Python-style per-frame accumulation, which Phalcom's bulk `unwind_to` cannot support.
> The second specified capture at the *consuming* boundary, which was true only while a
> non-matching `on` re-propagated without unwinding — an ordering PDR-0007 deliberately reversed
> to make the depth-ceiling error catchable. Anyone revising this section should re-read
> `block_on` rather than trusting the prose.

### 3a. Cross-fiber capture happens **inside** the cascade, not after it

The fiber floor is a consuming boundary, but a `Call`-mode failure does not stop there — it
cascades up the resumer chain ([`dispatch.rs:315-346`](../../phalcom-core/src/vm/dispatch.rs)),
and **each hop's parked `frames`/`stack`/`open_upvalues` are cleared as the cascade walks**
([`dispatch.rs:330-332`](../../phalcom-core/src/vm/dispatch.rs)). `capture_error_value` runs once,
before the loop.

The originating fiber is safe — its frames are the live `vm.frames` at capture time. Intermediate
`Call`-mode resumers are not: they hold real parked state (the code comment says so explicitly)
and the cascade destroys it before the `Try`-mode resumer that finally consumes the error is
reached.

So `docs/deferred/tracing.md`'s locked cross-fiber link — *"raised in fiber #N, spawned at
file:line"* — must be built **per hop inside the cascade loop**, before each `clear()`. Capturing
at the boundary that ultimately consumes the error yields a chain missing every intermediate
frame. The root-fiber exit (`return Err(e)`,
[`dispatch.rs:333`](../../phalcom-core/src/vm/dispatch.rs)) needs no special handling — it leaves
with frames intact, which is exactly the uncaught case where the full stack is wanted.

Where the field landed:

| Language | Capture policy |
|---|---|
| **Python** | traceback built incrementally during unwinding; cost proportional to frames actually unwound |
| **Rust** | `Backtrace::capture()` reads `RUST_BACKTRACE`, returns `Disabled` if unset — off by default, runtime opt-in |
| **Go, Swift** | errors are values; zero capture |
| **C++** | no capture at all until C++23's opt-in `std::stacktrace` |
| **Java** | eager `fillInStackTrace()` — and the cure was worse than the disease |

Java is the cautionary case and the closest analogue. Eager capture made exception construction
so expensive that HotSpot added the *fast-throw* optimization: hot implicit exceptions are
silently replaced by a preallocated instance carrying **no stack trace at all**. That is the
origin of the famous "my NullPointerException has no stack trace in production."

Unconditional capture-at-raise is the wrong default for **any** language, and specifically wrong
for one whose `Result` channel is built on raise-and-catch.

### 4. The capture record holds no object handles

A frame record entry stores a module-name `Symbol`, a method-name `Symbol`, and a line — **never
an `ObjRef`**.

This is a soundness requirement, not an optimization. `RuntimeError` travels the **Rust** stack
during unwinding. `collect_roots` exhaustively destructures `VM`
([`gc.rs:60-118`](../../phalcom-core/src/vm/gc.rs)) and can therefore only see what lives *on the
VM*. An in-flight error's payload does not.

This is precisely the bug fixed in `cdd2117`: `block_ensure` needs `push_temp_root` because
*"neither `vm.stack` nor `vm.frames` describes `outcome`"*
([`block.rs:316-322`](../../phalcom-core/src/primitive/block.rs)). A record of closure handles,
one per frame, held across an arbitrary unwind during which `on`/`ensure` handlers allocate and
can hit a safepoint, reproduces that hazard multiplied by frame depth.

Symbols are safe by construction — they live in the interner and are never collected
([`gc.rs:77`](../../phalcom-core/src/vm/gc.rs) marks `interner: _` a non-root). `temp_roots` is
not an alternative here: it is a depth-and-truncate stack and does not obviously survive an
arbitrary unwind.

### 5. `spans` is consumed through an accessor from day one

No call site indexes `chunk.spans[ip]` directly. All reads go through one accessor.

`spans` is `Vec<SourceRange>` at 16 bytes per instruction, pushed once per instruction
([`chunk.rs:47`](../../phalcom-core/src/chunk.rs), [`chunk.rs:92`](../../phalcom-core/src/chunk.rs)),
and it is one of a family of `ip`-indexed parallel arrays
([`chunk.rs:115`](../../phalcom-core/src/chunk.rs)) — so the per-instruction tax is worse than one
array. `docs/deferred/tracing.md` already records this as debt with the right migration shape
(delta-encoded line table plus binary search).

Nobody stores a range per instruction: CPython uses `co_lnotab`, then `co_linetable` (PEP 626),
both delta-encoded; the JVM's `LineNumberTable` is sparse; DWARF emits a line *program*; V8 uses
delta-encoded source position tables detached from the code object.

The accessor is required **in the same unit that introduces the traceback**, not later. The
traceback is what makes `spans` hot; adding the consumer while leaving direct indexing in place
manufactures the call sites the migration would then have to chase.

## Consequences

- `Error` gains two fields, `_cause` and `_kind`. Both `.ph`; **zero floor delta**, no ADR-0019
  amendment, no new kernel classes.
- `error-handling.md:143-146`'s three false examples become writable rather than needing
  annotation as aspirational.
- PDR-0005 §4 (use-after-close raises) and PDR-0007 §2 (depth-limit raise) both get a machine-
  checkable discriminator instead of a message string: `#useAfterClose`, `#depthExceeded`.
- U-TRACE's implementation spec must be written against §3 and §4 rather than the capture-at-raise
  item currently in its locked list. That list was locked before the `attempt()` cost and the
  rooting hazard were identified.
- `self.frames.clone()` (U-TRACE defect 3, `dispatch.rs:121-144`) disappears under §3 rather than
  needing its own fix.
- Adding a `kind` value later is a Symbol, not a release-note-worthy hierarchy change.
- **Two idioms for one question, named rather than hidden.** `e.is(MessageNotUnderstood)` and
  `e.kind == #range` both ask "what went wrong," and Phalcom code reaches for `isA` everywhere
  ([`core.ph`](../../phalcom-core/core/core.ph) uses it ~15 times). The split is principled —
  `isA` for the three conditions that already have kernel classes and are raised as real `Raise`
  instances, `kind` for conditions the VM detects natively and would otherwise flatten to a bare
  `Error` — but it is a split, and the spec must say which to use rather than leaving readers to
  guess. If sealed types later make `kind` a real type, the two idioms converge.

**What this precludes.** Committing to `kind`-as-data makes a later class-per-condition hierarchy
redundant rather than impossible — but shipping both would be the Go-plus-Java worst case, so a
superseding record should pick one. It does **not** preclude sealed types: when they land, `kind`'s
Symbol domain becomes a sealed enum with no call-site change, which is the whole reason for
choosing a Symbol over a string.

**What this does not cover.** Error *rendering* — frame ordering, core-frame elision, the miette
caret block — stays with U-TRACE, which has already ruled it. This record governs what an error
*carries*, not how it prints.

## Alternatives rejected

- **Capture at raise, unconditionally.** The locked U-TRACE position. Rejected on the measured
  shape of `attempt()` ([`core.ph:627-629`](../../phalcom-core/core/core.ph)): the `Result`
  channel raises and catches, so every `Err` would pay a full frame walk. This is Java's
  `fillInStackTrace` and it produced fast-throw.
- **Capture at raise, gated on whether a converting handler is installed.** Preserves eager
  origin and skips the `attempt()` tax. Rejected *for now* only because knowing a handler is
  installed requires frame-level handler records, which Rust-`?` unwinding does not provide.
  Revisit if the unwind path is ever reified.
- **Python-style incremental accumulation, one frame per unwind step.** This record's first
  draft. Rejected on inspection: Phalcom unwinds in bulk via `unwind_to`, so there is no
  per-frame hook to attach to, and building one is a frame-record redesign. §3 reaches the same
  cost profile through the snapshot that already exists.
- **Two chains, or three slots on `Error`.** Rejected under §1 — Python's outcome, reached
  deliberately rather than by accident. `displaced` earns its slot only because it is set by one
  stated rule and means something `cause` does not.
- **No capture at all (Go/Swift).** Cheapest and defensible for a value-error language. Rejected:
  Phalcom has exceptions, and a language with exceptions and no traceback is undebuggable — the
  status quo, and the reason U-TRACE exists.
- **Reify native `RuntimeError` variants as kernel classes.** The obvious reading of
  `error-handling.md` §6. Rejected: PDR-0001 and ADR-0041 make each new kernel class expensive,
  `kind` answers the same question as data, and the two together would be redundant.
- **A second, native-only cause chain for the cross-fiber link.** Rejected under §1 — this is
  Python's two-chain outcome, reached deliberately instead of by accident.
- **Store closure `ObjRef`s in the capture record and root them.** Rejected under §4: correct
  rooting across an arbitrary unwind is not something `temp_roots`' depth-and-truncate discipline
  provides, and Symbols make the question moot.
