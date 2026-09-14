# PDR-0018 — Decorator governance: ADR carry-forward verified; Install/Dispatch/Runtime tiers, `@effect`, and the framework families mandated as the v0.3 experimental track

- Status: **Accepted** (mandate ruled by the user 2026-07-20, this session; §1–§2
  are verification, not new law; §4's design specs are experimental and bind only
  the v0.3 track)
- Date: 2026-07-20
- Related: [`docs/design/decorators/canonical/`](../design/decorators/canonical/README.md) (the canonical
  decorator tree this record anchors),
  [ADR-0052](../adr/accepted/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md),
  [ADR-0053](../adr/accepted/0053-runtime-decorator-interception-reuses-override-epoch-guard.md),
  [ADR-0054](../adr/accepted/0054-two-speed-ratification-annotation-decorator-tiers.md),
  [ADR-0057](../adr/accepted/0057-decorator-granularity-vs-proxy-granularity-split.md),
  [ADR-0063](../adr/accepted/0063-constructors-are-ordinary-class-side-methods.md)
  (decorator surface: `@constructor`/`@class`),
  [ADR-0058](../adr/accepted/0058-reactive-tracking-context-needs-a-native-module.md)
  (`Reactive` native module — `@effect`'s substrate),
  [PDR-0001](0001-classes-are-closed.md) (closure; composes with A-5's frozen
  retention), [PDR-0005](0005-resources-are-disposable-handles-not-finalized.md)
  (no finalizers — the constraint that shapes `@effect` disposal).

## Context

The decorator corpus was consolidated 2026-07-20 into `docs/design/decorators/canonical/`
(commit `0bdfff5`): taxonomy, per-family verification against the committed
positions, an 11-entry spec-vs-code defect ledger (DEF-1…11), a 5-entry
collision registry (COLL-1…5), and implementation plans. Four decorator ADRs
were verified against HEAD in that pass. Separately, the tree recorded a
conservative build posture — Dispatch tier "not until a decorator forces it,"
`@effect` "not before reactivity R-5 resolves," frameworks "surfaces of systems
that don't exist." The user has now overruled that posture.

## Decision

### 1. Carry-forward: the four decorator ADRs are verified and remain binding

Verified against the tree 2026-07-20 (evidence in the decorator tree's family
files; method: registry/expander/driver read in `compiler/attributes.rs`,
retention floor in `primitive/attribute.rs`, grep-negative checks for the
runtime machinery):

- **ADR-0052** — ratified; its re-entrancy guard is **absent from the weave**
  (DEF-2). The ADR stands; the gap is an implementation debt, not a decision
  gap.
- **ADR-0053** — ratified; zero machinery on HEAD (`has_runtime_interceptor`:
  no hits). Stands as the priced design for §3's mandate.
- **ADR-0054** — ratified broadly (2026-07-14 correction confirmed): the
  Install/Dispatch/Runtime mechanism is ratified for any decorator built on
  it, A-1–A-5 resolved, A-6 deferred to v0.3.
- **ADR-0057** — ratified; the decorator/proxy granularity split governs the
  behavioral family's naming (COLL-3 interaction recorded, unresolved).

These ADRs are **not** superseded by this record; `docs/adr/STATUS.md` is
untouched. The canonical organizing spec for the decorator system is
`docs/design/decorators/canonical/`; the as-built files under `docs/spec/current/decorators/`
remain authoritative for what HEAD does.

### 2. The defect ledger is the debt register

DEF-1…11 (decorator tree README) are acknowledged as verified divergences.
Their fixes are **units, not decisions** — with one exception: contract
inheritance (covariant contract replacement on overrides, Liskov-unsound) has
no ruling anywhere and requires its own PDR before contracts are advertised
for polymorphic use.

### 3. The mandate: runtime decorator tiers are built, v0.3, experimental

User ruling, this session, recorded verbatim in substance: **runtime
decorators are a must, regardless of need**; Dispatch tier, framework
families, and `@effect` are to be fully designed now and built as the **v0.3
experimental track**.

This supersedes the build-when-forced posture recorded in the decorator tree:

- interception.md's preclusion "building the Dispatch tier speculatively,
  with no shipping decorator" — **withdrawn**; the tier is built with
  `@ForwardMissing` as its first decorator.
- reactive.md's preclusion "`@effect` before R-5" — **replaced**: the
  `@effect` design ships an explicit ownership/disposal model
  ([effect.md](../design/decorators/canonical/effect.md)) rather than waiting for
  reactivity.md's R-5 to resolve independently; that design *is* R-5's answer
  for the decorator surface.
- frameworks.md's "nothing here is scheduled" — the families receive a
  concrete experimental design ([frameworks-design.md](../design/decorators/canonical/frameworks-design.md))
  resolving E-1…E-4 / W-1…W-3; build follows the v0.3 track.
- proposals.md's "deliberately not proposed" entries for Dispatch/`@effect`
  — amended to point here.

**Not overridden** (the mandate covers the runtime tiers and the named
families, nothing else): the rejections of `@async`/`@await`, performance
hints, `@cfg`/`@intrinsic`, doc-content decorators, and scheduling
decorators all stand.

### 4. The v0.3 experimental design set

Four design specs land with this record under `docs/design/decorators/canonical/`,
**experimental**: normative for the v0.3 track once reviewed, binding nothing
in v0.2:

| Spec | Designs |
|---|---|
| [runtime-tier.md](../design/decorators/canonical/runtime-tier.md) | `Invocation`, `proceed`, chain composition and caching, the ADR-0053 guard bit, per-fiber trace-bypass, deopt and erasure obligations |
| [dispatch-tier.md](../design/decorators/canonical/dispatch-tier.md) | `resolveMissing(_)` protocol, transient (non-installing) resolution, chaining, dNU interaction, `@ForwardMissing(to:)` |
| [effect.md](../design/decorators/canonical/effect.md) | `@effect` with explicit activation, `Effect` handles, scope-owned disposal (no finalizers — PDR-0005 constraint honored) |
| [frameworks-design.md](../design/decorators/canonical/frameworks-design.md) | experimental rulings for E-1…E-4 and W-1…W-3; `T.schema` reflective contract; the request pipeline as fixture |

Build-order constraints carried from the tree, restated as binding on the
track: the metaobject gate (`Method.fromBlock`/`invokeOn`/
`Behavior.defineMethod` — its own floor PDR) precedes every Install-tier
decorator; the per-member erasure golden test lands **before** the first
Runtime-tier decorator ships; `defineMethod` routes through the same
install choke point as `Bytecode::Method` (`world_version` +
`note_method_installed` + kernel-closure rejection).

### 5. Explicitly left open

The Proposed items in the decorator tree are **not** ratified by this record:
the naming convention, COLL-3 suffix resolution, `@override`,
`@Deprecated`, `@suspends`, `attr.redundant` stacking, the N-2 pre-drop
harvest, and the contract-inheritance ruling (§2). Each remains a PDR
candidate in [proposals.md](../design/decorators/canonical/proposals.md).

## Consequences

- v0.3 gains an experimental decorator track with a fixed dependency spine:
  metaobject gate → Install decorators → Runtime machinery (`Invocation`,
  guard bit) → Dispatch tier → reactive (`@observable`/`@computed` behind
  U-REACTIVE-NATIVE, then `@effect`) → frameworks.
- "Experimental" is a real tier, not a mood: v0.3-experimental surfaces may
  change without supersession ceremony, must be flagged in their own status
  lines, and must not be load-bearing for v0.2 semantics or tests.
- The conservative posture this replaces is preserved in git history and in
  the amended files' change notes — if the experimental track shows the
  posture was right (a tier ships and finds no users), retiring it is a
  status flip, not an archaeology project.

## What this precludes

- Building any runtime-tier decorator ahead of the erasure golden or outside
  the single install choke point.
- Treating §4's specs as v0.2-binding, or citing them to justify v0.2
  changes.
- Re-litigating the mandate per-family: the ruling is "design and build the
  track," and a family opts out only by a superseding PDR.
