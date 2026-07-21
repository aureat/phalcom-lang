# 13 — Lane D: Future Conformance

> **Oracle:** generated cross-product against the spec's own transition table, plus algebraic laws (metamorphic).
> **Closes:** [G6](02-coverage-ledger.md#2-what-that-leaves-uncovered).
> **Status:** specified, not built.

## 1. Why generated, not hand-written

`Future` is a settle-once state machine (concurrency.md §2, ADR-0030 §1). Eleven
hand-written fixtures cover it today, chosen by intuition.

The precedent is unambiguous. **Promises/A+ is a one-page specification with an
872-test conformance suite.** That ratio is not ceremony — it is what it takes to
cover a settle-once state machine, and the reason it was built is that every
major JS engine shipped settle-twice and double-resolve bugs against
hand-written suites first. The cells that break are the ones nobody pictures:
settle during a `then` callback, reject after fulfil, `then` registered on an
already-settled future *from inside another future's callback*.

Phalcom's surface is smaller than A+ (no thenable assimilation, no
`Promise.all`/`race` — both **OPEN**), so the product is dozens of cases, not
872. But dozens is still more than eleven, and more than anyone enumerates by
hand.

## 2. The space

```
states  = { pending, fulfilled, rejected }
ops     = { then, map, catch, settle, reject, await, isReady, value, error }
timing  = { before-settle, after-settle }
```

Full product with pruning of meaningless combinations (`settle` has no
`after-settle` timing distinct from the settle-twice case, which is its own
row). Estimated 60–90 live cells.

**Expectations come from the spec's transition table, not from hand-written
`.expected` files.** This is [01 §"generated input demands an oracle of kind 2
or 3"](01-oracle-model.md#corollary-what-to-do-with-generated-inputs) applied:
the generator emits `(case, expected)` pairs from the same table, so nobody
hand-verifies 90 outputs, and a spec change regenerates rather than invalidates
the suite.

Prerequisite, and it is load-bearing: **concurrency.md §2 must state the
transition table explicitly** — for each `(state, op)`, the resulting state and
the observable result. If the spec does not say it, the generator cannot encode
it, and writing the table down is where most of the ambiguity gets found. Expect
this step to raise spec questions; that is the step working, not failing.

### Cells known to be underexercised

Derived from the 11 existing fixtures against the product:

| Cell | Why it matters |
|---|---|
| `settle` during a `then` callback of the same future | re-entrant settle; the classic double-resolve |
| `reject` after `settle` **and** `settle` after `reject` | settle-once in both orders — only one is covered |
| `then` registered from inside another future's callback | waiter list mutated during drain |
| `catch` on a `pending` future that is then *fulfilled* | the pass-through path |
| `await` on an already-settled future | must not schedule; must not suspend |
| `await` on a future settled by the awaiting fiber itself | self-deadlock — must raise, not hang |
| `map` whose function raises, on a `rejected` future | the skip path plus the raise path together |
| chain of ≥3 `then` where the middle rejects | rejection propagation depth |

The self-deadlock row is the sharpest: a test suite that hangs is worse than one
that fails, so this case needs a timeout wrapper regardless of what the spec
says the behavior should be.

## 3. Algebraic laws (metamorphic layer)

Cross-product coverage proves each cell behaves as tabulated. The laws prove the
cells **compose**. Both are needed; neither implies the other.

| Law | Form |
|---|---|
| Functor identity | `f.map(x => x) ≡ f` |
| Functor composition | `f.map(g).map(h) ≡ f.map(x => h(g(x)))` |
| `then` associativity | `f.then(g).then(h) ≡ f.then(x => g(x).then(h))` |
| Settle idempotence | `f.settle(v); f.settle(w)` ≡ `f.settle(v)` |
| Catch neutrality on fulfilled | `f.catch(g) ≡ f` when `f` is fulfilled |
| Await/async inverse | `await(async { e }) ≡ e`, for scheduler-invisible `e` |

Each law is a property test over a small generated value domain. `≡` means
"same observable result and same settled state" — **not** same object identity,
which the spec does not promise for derived futures.

## 4. Interaction with the scheduler

`Future` settling drives `System.schedule` (`core.ph` reschedules waiters on
settle). So Lane D partly tests the scheduler, and every law above implicitly
assumes drain order does not affect the *result* — only timing.

That assumption is itself worth one explicit test, because it is exactly where
an unspecified fairness policy ([G8](02-coverage-ledger.md#2-what-that-leaves-uncovered))
could leak into observable semantics. If any law's outcome depends on drain
order, that is a finding for the fairness ADR, not a test to weaken.

## 5. Cost and gating

Moderate authoring (the generator plus the spec table), cheap to run. **In the
default green gate.**

Generated cases should be emitted as real `.ph` files under
`tests/lang/concurrency/generated/`, committed, and regenerated by a checked-in
script — not generated at test time. Committed cases are diffable in review,
bisectable, and each failure names a file a human can run directly. A test-time
generator gives none of that.

## 6. Preclusion

- Encoding a transition table makes the table a **contract**. Changing
  `Future` semantics then means amending concurrency.md §2 *and* regenerating,
  in one change. That is the intended discipline, and it is a real cost to
  acknowledge up front rather than discover mid-change.
- The laws assume `Future` stays a **functor over settle-once**. Adding
  `Future.all`/`race`/cancellation (all **OPEN**) adds cells the table must
  grow to cover; adding *cancellation* specifically breaks settle-once as
  currently stated, since a cancelled future settles to neither value nor error.
  Do not generate cells for those until an ADR lands — an empty region is
  honest, a guessed region is a fabricated contract.
- Committing generated fixtures means a spec change produces a large diff. That
  is a feature (review sees the blast radius) but it will be argued about; the
  argument is settled by noting the alternative is an invisible blast radius.
