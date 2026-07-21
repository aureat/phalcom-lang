# Behavioral decorators — `@Memoize`, `@Retry`, `@lazy`, `@synchronized`

- Status: **Design-ratified (ADR-0054 broad), nothing built.** Source drafts:
  [decorators-behavioral.md](../v0.2/drafts/decorators-behavioral.md) (current
  surface; B-1/B-2 resolved) superseding the stdlib sketch. `Backoff` is the
  one shipped piece (`core.ph`); `Backoff.fixed`/`.exponential` raise until
  `System.sleep(_)` exists.
- Spelling per the naming convention: user-authorable Install decorators are
  `Attribute` subclasses ⇒ **`@Memoize`, `@Retry`,
  `@SynchronizedClassWide`** (pending COLL-3's suffix-resolution ruling for
  `@Retry` vs the `Retry` proxy); Layout builtins stay lowercase **`@lazy`,
  `@synchronized`**.

## The organizing rule (ADR-0052): where the state lives decides the tier

| State scope | Lives in | Tier | Who may author |
|---|---|---|---|
| per-call | the frame | Install | user |
| per-method / class-wide | the attribute instance | Install | user |
| **per-receiver** | a reserved slot on the receiver | Layout | **builtin only** |

A decorator hook may never close over a receiver-keyed collection — that is a
mark-sweep leak by construction (ADR-0052 Fix 2's generalized rule, enforced
as written contract + goldens, not static analysis). This one rule sorts the
whole family, and every historical misplacement in the drafts (`@computed`
Install→Layout, `@synchronized` OS-mutex→cooperative monitor) was a violation
of it or of ADR-0030's concurrency model.

## Verified designs (deltas only; the draft is the spec of record)

- **`@Memoize`** — Install; class-wide cache keyed `(receiver-identity,
  args)`, unbounded by default (documented retention leak, priced), `max: n`
  LRU opt-in. Not fiber-safe across suspension (double-compute possible) —
  documented, accepted. Weak-key revisit rides ADR-0052's GC trigger.
- **`@Retry(times:, on:, backoff:)`** — Install; frame-local attempt counter;
  retries only matching errors; `Backoff` strategy object (`none`/`fixed`/
  `exponential`) chosen for its fake-clock test seam (B-2). Idempotence is the
  caller's contract, stated not enforced. Backoff waits are fiber-yielding —
  inside a native combinator frame they raise `CannotYieldAcrossNativeFrame`;
  that composition note must appear in the decorator's own doc.
- **`@lazy`** — Layout; reserved slot, compute-once; initializer throw leaves
  the slot empty and the next access retries (retryability over
  determinism — the deliberate choice, keep).
- **`@synchronized`** — Layout; **cooperative re-entrant monitor** guarding
  the suspension window only (owning fiber + depth in a reserved slot;
  waiters queue and yield; release wired through `ensure`). On a
  suspension-free method it is pure overhead — the draft's lint idea
  (warn when the body cannot suspend) pairs naturally with the `@suspends`
  marker in [concurrency.md](concurrency.md). `@SynchronizedClassWide` is the
  rare Install variant, kept with its "usually a design smell" note.

The `@synchronized` re-derivation is the family's philosophy-verification
showcase: the stdlib sketch imported an OS-mutex model that is *meaningless*
under ADR-0030 (cooperative, single-threaded — no preemption, no data races).
The hazard a monitor guards here is **interleaving across yield points**, a
different and real hazard. A spec assembled hastily from other languages'
furniture imported the wrong hazard; verification caught it. That is this
tree's job in one example.

## Sketch-only residue — dispositions

These appear in the stdlib sketch or worked examples with no dedicated spec.
Recorded so they are owned, not ambient:

| Name | Disposition |
|---|---|
| `@Timed` | Fold into `@Traced(timing: true)` — a second wrap-and-clock decorator duplicates a `Tracer` flag that already exists. **Rejected as a separate decorator.** |
| `@Authorize(role:)` | Sound Install shape, but it presupposes `Session` — an ambient-authority surface Phalcom does not have. Parked to [frameworks.md](frameworks.md); must not land before an ambient-context design (same gate as `Flags`, D-3's v0.3 note). |
| `@Transactional` | Parked to frameworks (presupposes `Database`). The one design note worth pinning now: it composes *outside* lifecycle hooks and *inside* binders — the web draft's pipeline order is the spec of record. |
| `@RateLimit(perMinute:)` | Parked to frameworks; needs a clock + window type; the fake-clock seam argument (B-2) applies verbatim. |
| `@Validate` | Install wrap over generated setters — sound, but its real design questions (validation vocabulary, error aggregation) are framework-scale. Parked. |
| `@Idempotent(key:)` | Worked-example-only. Parked; requires a store abstraction. |
| `@Metered` | See [interception.md](interception.md) — Runtime family. |

## Implementation plan — the metaobject gate first (D-2)

Every Install decorator needs the same three primitives, none of which exist:
`Method.fromBlock`, `Method.invokeOn(recv, args)`, `Behavior.defineMethod`.
This is decorators/README.md's open **D-2**, and it is the *actual* unit of
work — the decorators themselves are thin once it lands.

1. **U-METHOD-REIFY (floor amendment, needs its own PDR):**
   `Method.fromBlock` (wrap a `Block` as an installable `Method`),
   `Method.invokeOn` (invoke reified method against a receiver — `Method`
   protocol, not `Behavior`), `Behavior.defineMethod(selector, method)`
   (install into the *class's* dictionary — `Behavior`-side only, per on.md's
   ruling; instances have no dictionary). Each passes the ADR-0019 admission
   test (reads method-table representation below the `.ph` boundary —
   inexpressible in-language). Install path must bump `world_version` /
   `note_method_installed` exactly as `Bytecode::Method` does — the sacred
   inliner's guard depends on it, and PDR-0001's kernel closure must reject
   `defineMethod` on kernel classes at this same choke point.
2. **Hook dispatch:** after class-definition executes attribute
   instantiation+attach (built), the driver additionally sends `wrap(m)` to
   Install-tier instances, replacing the dictionary entry via `defineMethod`.
   Source-order, innermost-last composition (the ratified stacking rule).
3. **Then** `@Memoize` and `@Retry` land as `core.ph` `Attribute` subclasses
   — pure library code, each with the draft's own test list (per-receiver
   distinctness, LRU eviction, retry-exhaustion rethrow, backoff schedule
   under a fake clock).
4. `@lazy`/`@synchronized` additionally need the Layout half: a
   `finalizeLayout` builtin hook that reserves hidden slots
   (`reserveSlot`/`slotAt`/`setSlotAt`), i.e. a real Layout tier where today
   there is none. That is a second, separable unit (U-LAYOUT-SLOTS) — and it
   is also `@observable`'s gate ([reactive.md](reactive.md)), so it should be
   designed once for both.

## What this precludes

- User-authorable per-receiver caching (Layout is builtin-owned) — until the
  weak-reference revisit trigger fires, and then only by superseding PDR.
- An OS-thread `@synchronized`. If Phalcom ever takes real threads, this
  decorator's semantics need a superseding decision, not reinterpretation.
- `@Timed` as a separate decorator (folded into `@Traced`).
