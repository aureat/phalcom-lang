# 10 — Hazard catalogue: where designs actually die

> **Thesis:** language designs rarely fail at a feature. They fail at an *interaction* — two
> individually correct decisions that, composed, produce something neither owner intended. The
> notation `A ⊗ B` names one. This file is the catalogue of the ones this project has hit,
> because a hazard you have hit is worth more than ten you have read about.

The strongest evidence for the thesis is a verdict recorded at the end of the concurrency
documentation track, after four documents on four separate mechanisms:

> **`[V]`** Every individual decision here would survive review. Four of them in a row produced a
> busy spin, a dropped upvalue, and a feature that cannot run.

---

## 1. The crown jewels

Three hazards this project calls "crown-jewel," meaning they shape whole subsystems rather than
single call sites.

### `native-stack frames ⊗ suspendable control`

**`[V]`** When a host-language primitive re-enters the interpreter to get a value back from guest
code, host stack frames sit between a coroutine's entry and its yield point — and *those frames are
the suspended position*. You cannot move them into a heap object, so you cannot suspend.

**Status: contained, not solved.** The restricted execution model raises rather than corrupting.
Full resolution requires de-recursing every callback primitive, which is reachable additively.
Developed in [`01`](01-coroutines-and-the-suspension-problem.md).

### `stackful-fiber ⊗ moving-GC`

**`[V]`** Give each coroutine a real machine stack and every parked stack becomes a root a moving
collector must scan and relocate — which it cannot do precisely without stack maps for every
function, coupling codegen to the collector permanently.

**Status: does not arise.** No native fiber stacks. The overlay is careful to note the second-order
drift, though: the shipped collector is non-moving anyway, so the optionality preserved here is
"mechanically intact, directionally stale."

### `speculative inlining ⊗ late binding`

**`[V]`** Inlining `ifTrue:`/`whileTrue:` assumes nobody redefines them. In a language where every
method is overridable, that assumption is a bet.

**Status: resolved** by a per-family pristine flag flipped on install, with the deopt path being a
real send. The soundness property is that guard failure is *observably identical* to the slow path.
Two caveats recorded: the guard is coarse (installing `and` deopts every `ifTrue`), and the claimed
observational identity **is false in one case** — see §4.

---

## 2. Hazards resolved by declining the feature

The most underrated resolution technique. Three examples.

### `default arguments ⊗ selector-identity dispatch`

**`[V]`** Omitting a defaulted argument produces a different selector, so lookup misses the
full-arity method. The only repairs were combinatorial arity-family expansion or static callee
knowledge a dynamic language lacks.

**Status: resolved by not taking the feature.** With a follow-up that is itself a technique: the
amendment permanently forbids call-site folding and states that *a superseding record inherits the
constraint and does not get to reopen it.*

### `adding match to a message language ⊗ open classes`

**`[V]`** A real `match` in a language with runtime class addition gets no exhaustiveness
guarantee and degrades to sugar over `isKindOf:` chains. Squeak shipped `caseOf:` as exactly this;
Pharo dropped it.

**Status: resolved by not adding grammar.** The eliminator convention gets totality from selector
identity instead — see [`02`](02-dispatch-and-selector-identity.md) §3.

### `multiple inheritance ⊗ fixed slot offsets and single-probe dispatch`

**`[V]`** C3 linearization would require a per-send ancestor walk (breaking one-probe dispatch) and
multi-ancestor instance state (breaking fixed offsets). Rejected as **inadmissible rather than
deferred**: "adopting it would be a redesign, not a feature."

**The pattern across all three:** the cost of a feature is what it precludes, and sometimes the
cheapest resolution is to decline. What makes these good decisions rather than mere conservatism is
that each one *names what was given up* and *states the conditions under which it could return*.

---

## 3. Hazards sidestepped by narrowing an axis

### `inline cache ⊗ mutable hierarchy`

**`[V]`** A cache keyed on class identity must be invalidated when the hierarchy changes. Sealing
superclass reparenting means a future cache keys on `ClassId` with **no invalidate-on-reparent
case at all** — the case is deleted, not made cheap. Method mutation keeps the epoch guard.

### `Option bootstrap cycle`

**`[V]`** Fields default to `None`; constructing `None` seemed to require a class whose fields
default to `None`. **Dissolved**: `None` is fieldless, so the rule never re-enters its own
construction. No code change was needed — the tree was already correct and the record's
contribution was to explain why.

### `truthiness ban ⊗ no flow analysis`

**`[V]`** Banning `if (opt)` has no static analyzer to enforce it. Resolved with **both** halves —
a runtime guard floor *and* a compile-time rejection of syntactically literal cases — with the
escape (`let x = None; if (x)`) documented as an accepted gap rather than hidden.

---

## 4. Live hazards, and hazards that fired

These are the valuable ones.

### `Reactive.current ⊗ trampolined block calls`

**`[V]`** A single VM-owned tracking context is sound **only because** the block-call trampoline
was deferred. Trampolining would let a suspended computed value have *another fiber's* reactive
reads register into *its* dependency set — silent dependency-set corruption, no error, no
diagnostic. The record's own words: the design "stays safe *by accident*… Nothing records that
this is load-bearing." A later record exists specifically to make the coupling deliberate.

**The general shape, and it is the most dangerous one in this file:** *a correctness property that
holds because an unrelated feature has not shipped yet.* Nothing in the code says so; nothing
fails when the precondition is removed; and the feature that removes it will be owned by someone
who has never read the record that depends on it.

### `restricted yield ⊗ a library written in the language`

**`[V]`** The `await` implementation probes whether it may suspend by attempting a yield inside an
`.attempt()` wrapper. But `.attempt()` is pure guest code expanding to two primitives, each
incrementing the native-reentrancy counter — so **the probe is itself the obstruction**,
unconditionally, for every fiber, forever. Worse, the two failure readings differ: on the root
fiber the refusal arrives untyped, is misread as "nothing to wait on yet," and degrades to a busy
spin; off the root it arrives typed and kills the awaiting fiber.

The record's framing:

> **`[V]`** a library written in the language does not get an exemption from the language's rules.
> That is normally the point of writing the library in the language. Here it is the bug.

Two individually correct decisions — a deliberately permissive depth-relative guard, and
implementing futures in pure guest code with zero new primitives — composed into a feature that
could not run. **`[V]`** Fixed later, at the cost of one new primitive, "because the question
`await` needed to ask had no answer in the language."

### `fiber-floor teardown ⊗ upvalue closing`

**`[V]`** Containment at the fiber floor is implemented as *deletion* — a status write and three
bulk clears — not as a completed unwind. Everything the unwind does on the way down, including
closing open upvalues, the deletion forgets. Confirmed panic: an uncaught fiber failure drops a
live stack without closing open upvalues, and an escaped capturing block then indexes an empty
stack.

**`[V]`** And the note that makes it a hazard rather than a bug: it is currently survivable only
because the restricted execution model forbids resuming from inside an `ensure`. So —
"**the restricted execution model is quietly containing the consequences of the teardown's
incompleteness**… a reason to be careful, not comfortable, about narrowing the guard. Widening
what a fiber may do while suspended widens what the floor can drop."

### `field privacy ⊗ read-before-write diagnostics`

**`[V]`** A privacy violation is reported as a missing assignment, and the diagnostic's suggested
fix silently produces field shadowing — one object, one field name, **two values**.

**The general shape:** a diagnostic is part of the language surface. A wrong-but-plausible error
message with an actionable fix is worse than no message, because the user will follow it.

### `lexer modes ⊗ REPL completeness`

**`[V]`** An unterminated lexical mode is invisible to any completeness oracle defined over the
*token* grammar — the lexer never emits the token that would signal incompleteness. Resolved by
requiring every mode to co-emit an end-of-input signal at its lowering site, and recorded as a
**permanent obligation with no compiler enforcement and a silent failure mode** — honest about the
weakness of the fix.

### `key hashing ⊗ arena borrows`

**`[V]`** Hash-map primitives run user `hash`/`==` implementations, which re-enter dispatch, which
can invalidate a live slot index. User code mutating the collection from inside `==` produces a
panic or silent corruption. The stated discipline: **re-resolve the object reference after every
such send.**

---

## 5. Hazards in the process, not the language

Worth a section because they cost as much and are less discussed.

**`[V]` Ratification by fait accompli.** A unit shipped a naming convention different from the
ratified record; a follow-up record was written to bless what shipped; the user reversed it. The
retirement banner states the principle: **"a ruling should move the code, not the other way
round."** The tree carried both conventions simultaneously for a day.

**`[V]` Concurrent records contradicting each other.** Two numeric-model records were accepted
about twelve hours apart, neither citing the other, in direct contradiction — "the concurrent-session
hazard landing in the decision record itself." Mitigation adopted later: when two proposals amend
the same frozen count off the same baseline, **whichever ratifies second rebases off the other's
final number**, and neither may quietly restate the original base.

**`[V]` Counts that do not chain.** Two records state incompatible before/after totals for the same
frozen inventory. Verdict: **never quote a census number from a record; run the census.** The true
figure was later obtained by running the check in a clean worktree at a pinned commit, and differed
from every number in every document.

**`[V]` Shipped is not designed.** A plan recommended per-class invalidation epochs with a global
counter as fallback; the fallback shipped, in commits outside the plan's own scope. The
documentation warns against the flattering reading: do not say the project "chose" coarse
invalidation for simplicity — no record says that, and the plan recommends the opposite.

**`[V]` A deferral reason that was never checked.** A unit was deferred partly because "the inliner
already covers arithmetic." It does not; arithmetic is ordinary message sends. The reason survived
unexamined for weeks because it sounded right. **Deferral rationales are claims and decay like any
other.**

---

## 6. How to write one of these

The `A ⊗ B` notation earns its keep by forcing three things into the open:

1. **Name both sides.** "Fibers are tricky" is not a hazard. "Native stack frames ⊗ suspendable
   control" tells you which two owners must talk.
2. **State the status honestly** — *resolved*, *dissolved*, *does not arise*, *contained*, *live*,
   or *fired*. "Contained" and "does not arise" are different: the first is a guard you could
   remove, the second is a structural impossibility.
3. **State what would reopen it.** Most entries above are one feature away from returning. A hazard
   record without a reopening condition is a snapshot; with one, it is a tripwire.

And the meta-lesson from §4: the most dangerous entries are not the ones marked *live*. They are
the ones marked *does not arise* **because something else has not shipped yet** — where the
protection is real, load-bearing, and recorded nowhere near the code that depends on it.
