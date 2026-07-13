# 54. Two-speed ratification: Compile/Layout tier now, Install/Dispatch/Runtime gated on ADR-0053

- Status: Accepted (Compile/Layout tier ratified by the user 2026-07-13; Install/Dispatch/Runtime tier ratified by the user 2026-07-13 — both gate conditions in Decision §2 satisfied: (a) ADR-0053 Accepted, (b) `attribute-classes.md` A-1–A-5 resolved, A-6 explicitly deferred to v0.3 as a non-blocking per-instance-behavior question)
- Date: 2026-07-13
- Related: `docs/spec/v0.2/experimental/annotations-core.md` (the original
  derive-macro-only foreclosure, amended below), `docs/spec/v0.2/next/decorators.md`
  (the five-tier model that reopened it), `docs/spec/v0.2/next/attribute-classes.md`
  (open questions A-1–A-6, the remaining gate), [ADR-0053](0053-runtime-decorator-interception-reuses-override-epoch-guard.md)
  (Runtime-tier cost model — the gate this ADR records as satisfied),
  [ADR-0052](0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)
  (contract/decorator runtime-state fixes folded into the Compile/Layout tier
  being ratified here)

## Context

Two annotation/decorator drafts contradict each other with no reconciling
ADR. `annotations-core.md` commits `@` to a closed, compile-time-only
derive-macro model and states explicitly: "Committing to derive-macro
semantics forecloses ever making `@` a Python-style runtime decorator hook
without a second mechanism." `decorators.md`, written one day later, reopens
exactly that door by introducing a five-tier model (Compile/Layout/Install/
Dispatch/Runtime) that includes real runtime hooks — justified by appeal to
`typing.md`'s erasure invariant `E`, a third, also-unratified, unrelated
draft's argument. An implementer following one document builds a
structurally different system than one following the other, and nothing in
either document says which one is authoritative.

Resolving this by picking a side wholesale is premature in either direction:
the Compile/Layout tier (`@data`, `@get`/`@set`, `@requires`/`@ensures`/
`@invariant`, `@construct`) is well-grounded against the actual compiler
member-loop (`compiler/lib.rs` L457–528) and has no open architectural
question left — it is ready. The Install/Dispatch/Runtime tiers had one
concrete open question (the Runtime-tier interception cost model), now
resolved by [ADR-0053](0053-runtime-decorator-interception-reuses-override-epoch-guard.md),
but still carry several unresolved design questions of their own
(`attribute-classes.md`'s A-1 through A-6: tier-inference mechanism,
inheritance-chain retention, constructor arbitrary-code restriction, dedup,
runtime mutability, v0.3 per-instance scoping) that are independent of the
cost-model question and not addressed by this ADR.

## Decision

### 1. Ratify the Compile/Layout tier

The following are accepted as the normative design for `@` in Phalcom,
folding in the fixes already made to the drafts during review:

- `annotations-core.md` — the derive-macro mechanism, four-layer change,
  expander registry, phase-ordered composition, span hygiene.
- `annotations-legality-grammar.md` — grammar, `Target`/legality table,
  newline binding, unknown/misplaced-attribute diagnostics.
- `annotations-contracts.md` + `annotations-contract-semantics.md` (as
  amended by [ADR-0052](0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)
  and the metadata-stripping fix) — `@requires`/`@ensures`/`@invariant`,
  reflectable contracts, receiver-scoped re-entrancy, two-axis
  guard/metadata stripping.
- `annotations-construct.md` + `annotations-construct-inheritance.md` (as
  amended by the super-signature-inference fix, tracked alongside this ADR)
  — `@construct`, `@get`/`@set`, field-declaration layout.
- `annotation-paradigm-bridges.md`'s method-table-macro/layout-derive tier
  line — the seam this whole split is organized around.
- `annotations-data.md` (new, 2026-07-13) — `@data`/`@sealed`/`@variant`,
  extracted from Bridge A into a standalone draft that depends only on the
  Compile/Layout mechanism above, not on the gated `decorators.md`. Its
  visitor-dispatch generation deliberately does **not** include new `match`
  grammar (open-Q7 remains open and unaffected by this ratification) — only
  a generated keyword-argument-selector visitor method, zero new syntax.
- `annotations-test-strategy.md` (as extended) — the diagnostics catalog and
  test corpus for all of the above.

This tier needs no further architectural decision before implementation
begins.

### 2. Amend `annotations-core.md`'s foreclosure — scoped, not deleted

The statement "committing to derive-macro semantics forecloses ever making
`@` a Python-style runtime decorator hook without a second mechanism" is
**amended to apply to the Compile/Layout tier only.** It is no longer a
blanket statement over all of `@`. `decorators.md`'s Install/Dispatch/Runtime
tiers are the permitted second mechanism this ADR admits — but they are
**were** gated on, both now satisfied — **ratified 2026-07-13**:

- **(a) Runtime-tier cost model — satisfied.** [ADR-0053](0053-runtime-decorator-interception-reuses-override-epoch-guard.md)
  closes the specific soundness/performance gap that was the substantive
  content behind `annotations-core.md`'s original foreclosure (an unguarded
  runtime hook interacting unsoundly with the sacred-selector inliner and
  imposing an unbounded tax on undecorated sends). That concern is now
  answered, not merely asserted away.
- **(b) `attribute-classes.md` open questions A-1–A-6 — satisfied.** A-1
  (explicit tier declaration), A-2 (`inherited:` opt-in), A-3 (arbitrary code
  allowed, Compile/Layout stay compiler-reserved), A-4 (repeated attributes
  compose), and A-5 (frozen retention post-class-definition) all resolved
  2026-07-13, recorded inline in `attribute-classes.md`. A-6 (per-instance
  behavior's effect on Install-hook scoping) is explicitly deferred to v0.3 —
  it concerns a feature (per-object method dictionaries) that does not exist
  yet, so it cannot block ratifying a mechanism that only needs to work over
  the `Behavior`-side surface that exists today.

With both satisfied, Install/Dispatch/Runtime tier moves from mechanism-only
to implementable: the 8 named decorators speced in
[decorators-behavioral.md](../spec/v0.2/next/decorators-behavioral.md),
[decorators-dispatch-observability.md](../spec/v0.2/next/decorators-dispatch-observability.md),
and [decorators-observable.md](../spec/v0.2/next/decorators-observable.md)
are ratification-ready, not just mechanism-illustrated.

The justification for reopening the foreclosure is **no longer**
`decorators.md`'s original one. Borrowing `typing.md`'s erasure-invariant
argument was building an amendment to a committed decision on top of an
unrelated, equally unratified draft — sound-sounding, not sound. The
justification going forward is ADR-0053 directly: a concrete, implementable
cost model that makes the runtime hook's soundness and performance
consequences explicit, the same bar `annotations-core.md`'s original
foreclosure was implicitly asking for. `decorators.md`'s Context section's
citation of `typing.md §5.2`'s erasure invariant `E` as the reason the
foreclosure "no longer holds" is superseded by this ADR and should be
struck or footnoted as historical rationale, not live justification.

## Consequences

- Implementers have one authoritative answer instead of two documents in
  tension: build the Compile/Layout tier fully now; treat Install/Dispatch/
  Runtime as still-Proposed and explicitly gated, not silently assumed.
- `annotations-core.md` is not rewritten — ADRs, not draft edits, are how a
  stated foreclosure gets amended in this repo's convention — but it should
  carry a one-line pointer to this ADR so a reader doesn't take the original
  "forecloses... without a second mechanism" sentence at face value.
- The gate on Install/Dispatch/Runtime is now legible and finite: one item
  satisfied (ADR-0053), six named open questions remaining (A-1–A-6), rather
  than an open-ended "later" with no checklist.
- **Negative / accepted.** The framework-tier documents that build on the
  open tiers (`decorators-web.md`, `decorators-persistence.md`,
  `decorators-stdlib.md`'s Install/Dispatch/Runtime sections, the vertical-
  slice `Resource` example) remain explicitly unratified pending the same
  gate — real, already-written design value stays parked until A-1–A-6
  resolve.

## What this precludes

Nothing structurally new for the Compile/Layout tier — it was already fully
specified; this ADR is a ratification, not a redesign. For Install/Dispatch/
Runtime, this precludes treating `decorators.md` as ratified by implication
— e.g., starting to build `Behavior.defineMethod`/`Method.invokeOn` against
it, or shipping any user-facing `Attribute` subclass surface — before
`attribute-classes.md`'s A-1–A-6 are resolved. That would be building
production surface against an admittedly-gated draft.
