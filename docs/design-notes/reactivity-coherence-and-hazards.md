# Reactivity — OOP coherence, and hazards found before `U-REACTIVE-NATIVE`

Status: investigation notes. The design itself is **not** open — `docs/spec/v0.2/next/reactivity.md`
is Accepted (ratified 2026-07-13, R-1–R-5 all resolved) and [ADR-0058](../adr/0058-reactive-tracking-context-needs-a-native-module.md)
is Accepted. Nothing here reopens either. Findings 1–3 are gaps *inside* the
accepted design, surfaced pre-implementation; finding 1 is a cross-ADR coupling
that no document currently records.

## The question that started this

"Is there a design-philosophy clash between Smalltalk-style pure OOP and built-in
reactivity with compiler support?"

Answered by the tree, not by analysis: reactivity is already specced and ratified.
But two sub-questions were worth settling, and the answers are worth keeping.

### Pure OOP vs reactivity — no clash; OOP is the *better* substrate

`reactivity.md`'s thesis (reactivity = the fourth facet of message interception)
is historically well-supported:

- **Smalltalk-80 shipped reactivity in 1980** — `Object>>changed:`/`update:`,
  DependentsFields. Purest OOP language, library layer, no compiler support.
  Cost: *manual* notify — forgotten `changed:` sends → stale views. That failure
  mode is exactly what automatic tracking removes.
- **CLOS AMOP `slot-value-using-class`** makes slot reads an overridable generic
  function; Kenny Tilton's **Cells** built auto-tracked reactivity on it in a
  dynamic MOP language. Existence proof in the same family. Cost: implementations
  must special-case `standard-class` or every slot read pays generic dispatch.

Uniform access is why this is easy here and was hard elsewhere: JS needed `Proxy`
and Swift needed macros only because they have raw property access. The spec's
choice to ride `@observable`-derived accessors rather than instrument `GetField`
keeps [ADR-0011](../adr/0011-static-instance-slot-layout.md)'s slot path
barrier-free and confines tracking cost to observed properties — the same
per-property granularity Swift's `@Observable` gets, without its per-read
thread-local + lock (which Phalcom does not need at all: one `VM`,
cooperative single-threaded, [ADR-0030](../adr/0030-fibers-and-futures-cooperative-concurrency.md)).

### "Compiler support" — already answered *no*, correctly, by construction

A compiler cannot know dependency edges in a language where any send is
redefinable ([ADR-0026](../adr/0026-class-hierarchy-mutability.md)/[ADR-0041](../adr/0041-hierarchy-stability-policy.md)),
proxyable via `doesNotUnderstand`, or reachable via `perform`/`SEND_DYNAMIC`.
Precedent with consequence: **Svelte retreated from exactly this.** Svelte 3/4's
`$:` was compile-time dependency analysis; it broke as soon as state crossed a
module or function boundary and the compiler lost sight of it. Svelte 5 runes =
runtime signal graph + *light* compiler assist. Solid is the same shape (compiler
helps only at a closed JSX boundary); React Compiler must prove purity and ships
a bailout plus a lint rule.

`reactivity.md` is already the Svelte-5 shape: runtime graph, decorators generate
accessors, no compile-time dep analysis. The question is closed by construction.

### One claimed clash that does not survive — recorded so it isn't re-raised

This session first argued that *transparent* reactive reads (`cart.total`, not
`cart.totalSignal.value`) contradict the explicitness posture of
[ADR-0021](../adr/0021-no-truthiness-enforcement.md) (no truthiness) — that
auto-tracking is "truthiness for control flow." **That argument is wrong**, for
two reasons:

1. Truthiness changes what an expression *means*; a tracked read does not. The
   value of `cart.total` is identical whether or not a computation is currently
   tracking. Dependency registration is a side effect of reading, not a
   reinterpretation of the read.
2. The spec marks at the **declaration** site (`@observable var _items`), which is
   the same discipline as `let`/`var` ([ADR-0014](../adr/0014-let-and-var-bindings.md)) —
   declare-site marking, not use-site re-marking. Requiring `.value` at every use
   would be redundant re-marking, and would buy no information the declaration
   doesn't already carry.

The spec's transparent-read choice is coherent with ADR-0021. Do not reopen it.

## Finding 1 — ADR-0058's `Reactive.current` is sound only because ADR-0033 is Deferred (undocumented coupling)

**The strongest finding here.** [ADR-0058](../adr/0058-reactive-tracking-context-needs-a-native-module.md)
makes `Reactive.current` a single **`VM`-owned** `Option<ObjRef>`, with
`trackedBy`/`untracked` as native save/set/run/restore — explicitly mirroring
`Fiber`'s `store_live_into`/`load_live_from` swap. That is safe **only** while the
`run.call()` inside `trackedBy` cannot suspend.

Today it cannot: [ADR-0030](../adr/0030-fibers-and-futures-cooperative-concurrency.md)
§4 is restricted Option A, so `f.call(x)` routes through the re-entrant
`block_call` primitive → recursive `run_until` → a native Rust frame, and a
`Fiber.yield` beneath it raises `CannotYieldAcrossNativeFrame`. The tracking
context cannot be observed mid-flight because the block cannot suspend mid-flight.

[ADR-0033](../adr/0033-amend-fiber-execution-trampolined-block-callsite.md) is
**Deferred, not Rejected** ("revisit as the general lift"). It trampolines the
bytecode block call-site, making block calls yield-transparent. If a `Computed`'s
compute block then suspends inside `Reactive.trackedBy`, another fiber resumes and
**its** reactive reads register into the **suspended** computation's dependency
set. Silent dep-set corruption: wrong edges, missed invalidations, spurious reruns.
No error, no diagnostic.

What makes this a documentation bug rather than a live one:

- ADR-0058's Negative consequence *does* mention per-fiber isolation, but frames it
  as a future **sandboxing/isolation** nice-to-have — "not a v0.2 concern" — not as
  a **correctness** precondition on ADR-0033's deferral. Different severity, and
  the framing is what a future reader will act on.
- ADR-0033 §Decision 4 already carries a sequencing constraint listing what it must
  not land before (the ADR-0030 §5 typed fiber-switch signal). `Reactive.current`
  is **not** on that list and should be.
- ADR-0033 §Decision 2 retains re-entrant `block_call` for **native** callers, and
  `trackedBy` is native — so the natural implementation stays native-framed and
  stays safe *by accident*. Nothing records that this is load-bearing. A later
  "why does `trackedBy` block yields? trampoline it" optimization silently removes
  the safety.

**Action:** when `U-REACTIVE-NATIVE` lands, state in the ADR/implementation that
`trackedBy`/`untracked` invoke their block via the re-entrant `block_call` path
(ADR-0033 §Decision 2) and that this is a soundness requirement, not an oversight.
Add `Reactive.current` to ADR-0033 §Decision 4's sequencing constraint: whenever
ADR-0033 is revisited, the tracking context must become per-fiber (saved/restored
at switch, alongside `store_live_into`/`load_live_from`) *in the same unit*.

## Finding 2 — a raise mid-`flush` permanently loses the remaining scheduled effects

`reactivity.md` `## The runtime`, `Reactive.flush`:

```phalcom
static flush { let due = _pending; _pending = Set.new(); due.each { e => e.run } }
```

`_pending` is cleared **before** iterating. Under
[ADR-0008](../adr/0008-layered-exceptions-and-result.md) the error model is
terminating (Smalltalk `resume:` was rejected), so if any `e.run` raises, the
remaining effects in `due` never run **and** are no longer in `_pending` — they are
dropped, not retried. The graph is left half-updated with no recovery point and no
record of what was skipped.

ADR-0058 is careful to specify that `trackedBy` restores its field "even across a
raise". `flush` has no equivalent guarantee, and it is the one that loses data.

Precedent: MobX and Vue both punt here (log and continue to the next effect);
React uses error boundaries. Both are *choices*; the spec currently makes neither.

**Action:** `flush` needs a specified raise policy. Cheapest fix consistent with
the terminating model — drain one at a time so unrun effects stay in `_pending`,
and wrap each `e.run` in `ensure` so one failing effect cannot strand the rest.
Needs an explicit decision, not an implementation guess.

## Finding 3 — `!=` as the write-time bail is wrong on `NaN` and `±0`

`Signal.value=(next)` bails with `if (_value != next)`. `Number` is flat `f64`
([ADR-0005](../adr/0005-number-as-flat-f64.md), reaffirmed by
[ADR-0042](../adr/0042-flat-number-defer-integer-float-split.md)), so IEEE-754
semantics apply and `!=` is wrong in **both** directions:

- `NaN != NaN` is **true** → writing `NaN` over `NaN` propagates. Spurious rerun
  on every write. (Not a hang: `markStale`'s `_stale == false` bail stops the
  cascade.)
- `-0 != 0` is **false** → writing `-0` over `0` is swallowed. A real value change
  that never propagates. **Missed update — the worse direction.**

JS has `Object.is` for exactly this pair, and React uses it for state-change
detection for exactly this reason.

The spec's "Equality is a message — user-overridable" design call does not cover
this: `Number`'s `==` is floor ([ADR-0019](../adr/0019-freeze-vm-blessed-primitive-floor.md),
amended by [ADR-0036](../adr/0036-amend-floor-admit-number-tostring.md)), so a user
cannot override it, and should not have to.

**Action:** `Signal.value=` needs a same-value predicate rather than `!=`. Decide
before `U-REACTIVE-NATIVE`, and before any Int/Float surface split revisits
ADR-0042 — the predicate's contract is easier to pin now than to change under a
split later. Note R-1 shipped the boolean stale-flag skeleton and deferred
three-color marking to v0.3; three-color's bailout also compares values, so it
inherits whatever predicate is chosen here.

## Lower-severity notes

- **Effect block escape ⊗ non-local return.** `Effect.new(run: { … return … })` —
  the block's home frame is dead by flush time, so
  [ADR-0013](../adr/0013-closure-upvalues-and-frame-token-return.md) raises
  `DeadFrameError` at flush, blamed on whoever wrote the triggering assignment,
  arbitrarily far from the offending block. Sound by construction; diagnostics will
  be poor. Worth a targeted error message.
- **Allocation shape.** `recompute` does `_sources = Set.new()` per evaluation and
  `flush` does `_pending = Set.new()` per drain. Per
  [ADR-0051](../adr/accepted/0051-performance-strategy-measure-first-tiered-optimization.md)
  (measure-first) this is **not** a reason to pre-optimize — recording it only so
  it is instrumented when `U-REACTIVE-NATIVE` lands, given allocation is the #1
  measured cost mechanism to date and F5 already showed a pooling null result.
- **Computed purity is unenforceable.** A `Computed` reading untracked ambient
  state gives stale values with no diagnostic; there is no effect system, by design
  (overlay Axis 6: "every send would need an effect row the runtime can't check").
  Same shape as the truthiness-without-flow-analysis problem ADR-0021 already
  settled: a runtime floor plus rejection of obvious cases is the ceiling. Inherent,
  accepted, not actionable — MobX's documented gotcha surface is this tax paid in
  another language.

## What this precludes

Nothing — no decision is taken here. Findings 1–3 each need a call before
`U-REACTIVE-NATIVE` implements the module; finding 1 additionally needs a line
added to ADR-0033 §Decision 4 whenever that ADR is next touched, independent of
whether reactivity ships first.
