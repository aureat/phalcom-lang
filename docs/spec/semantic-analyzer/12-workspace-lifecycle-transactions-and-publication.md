# Phalcom Semantic Analyzer Specification
## 12 — Workspace Lifecycle, Transactions, and Publication

**Status:** Normative semantic-analyzer specification.

**Purpose:** Define persistent workspace state, source/module lifecycle transitions, transactional semantic updates, failure containment, invalidation, and atomic immutable snapshot publication.

---

## 1. Persistent semantic world

The semantic analyzer operates over a persistent workspace rather than reconstructing unrelated project/module identity for each edit.

Conceptually:

```text
CommittedWorkspace
├── canonical project/module/source state
├── committed semantic revision
├── semantic query/database state
├── type-store identity
├── published immutable snapshot
└── last-known-good publication
```

Project/module infrastructure owns project identity, source identity, overlays, resolution, linking, imports, and exposure. The semantic session owns formal/advisory products, source indexes, semantic dependencies, and snapshots. Protocol clients adapt lifecycle events; they do not become semantic owners.

---

## 2. Transaction law

Workspace mutation is transactional:

> A candidate becomes the next committed semantic world in full, or it does not alter the committed world.

Candidate work may use mutable scratch/query state. Before commit, no reader may observe a mixture of candidate and committed products.

```text
committed revision N
        ↓ apply mutation batch
candidate revision N+1
        ↓ resolve / link / analyze / validate
atomic commit and publish
        ↓
committed revision N+1
```

Cancellation, staleness, budget failure, or internal failure discards the candidate and every candidate-only mutation.

---

## 3. Lifecycle events

The workspace supports semantic transitions equivalent to:

- open, change, save, close, reopen;
- create, delete, rename/move;
- watched filesystem refresh;
- overlay addition/removal;
- project-root, manifest, dependency, or configuration change.

Events may be batched when one coherent client action affects several sources. One batch produces at most one committed semantic revision/publication.

---

## 4. Transition matrix

| Event | Source/overlay state | Identity consequence | Required semantic consequence |
|---|---|---|---|
| open | install editor overlay | preserve canonical module/source identity | analyze overlay-precedence text |
| change | replace overlay text/revision | preserve identity unless project semantics changed | invalidate exact dependency frontier |
| save | refresh disk while overlay remains authoritative | normally preserve identity | no duplicate semantic world |
| close | remove overlay; expose current disk source | preserve module identity when same source survives | analyze restored disk product if semantically different |
| create | register new canonical source/module | create identity by project/module rules | link and invalidate affected importers |
| delete | remove source/module and provider entries | old identity no longer resolves | remove products and invalidate reverse closure |
| rename/move | remove old source and register new location | preserve only through explicit canonical identity rule; otherwise old/new identities | invalidate both old reverse closure and new dependents |
| watched refresh | update closed disk source; never override active overlay | preserve according to source identity | publish one coherent batch |
| project/config change | rebuild affected project/module universe | identities may change by canonical project rules | invalidate every product whose semantic inputs changed |

Revision counters and fingerprints must not substitute for canonical identity decisions.

---

## 5. Overlay precedence

Open editor overlays have precedence over disk content for the same canonical source. A watched disk event while an overlay is active must not silently replace the analyzed overlay.

Closing the overlay exposes disk state without inventing a second module identity. If disk and overlay products are semantically equivalent, reuse is permitted; presentation/source revision still must remain coherent.

---

## 6. Candidate construction

A candidate update records:

- source/module mutations and removals;
- canonical identity changes;
- resolver/linker products;
- invalidated and recomputed semantic queries;
- formal, source-index, advisory, diagnostic, and presentation effects;
- cancellation/budget/incident state;
- the proposed immutable snapshot.

Every derived product names or encodes the semantic inputs on which it depends. “File changed” is not a sufficient dependency policy.

---

## 7. Commit and publication

A candidate may commit only when:

1. it is based on the current committed revision;
2. its required project/module/link products are coherent;
3. every published product belongs to the candidate revision or is proven reusable;
4. no cancellation, stale-generation, budget, or internal-failure policy forbids commit;
5. the complete immutable snapshot is available for atomic publication.

Commit updates module/session/query state and published snapshot as one semantic transaction. Readers holding the previous immutable snapshot may finish against it.

---

## 8. Failure taxonomy

| Outcome | Commit policy |
|---|---|
| ordinary syntax/semantic diagnostics with coherent recoverable products | publish current invalid program and diagnostics |
| unresolved dependency with coherent surviving world | publish surviving products and canonical unresolved-dependency diagnostic |
| cancellation | discard candidate; keep committed publication |
| stale base generation | discard candidate; schedule/retry from current revision |
| budget exhaustion that prevents coherent required products | discard candidate; keep last-known-good publication |
| internal/infrastructure failure | discard/contain affected candidate; keep last-known-good publication |

Last-known-good exists for infrastructure/cancellation failure. It must not hide a coherent current program merely because that program contains user errors.

---

## 9. Deletion and rename

Removed modules cannot be resurrected from provider, resolver, linker, or query caches.

Deletion must:

1. remove source/provider/module registration;
2. remove formal/source/advisory/presentation products;
3. remove exact and advisory reverse attachments;
4. invalidate reverse semantic dependencies;
5. retain remaining importers as publishable products with unresolved-import diagnostics where recovery permits.

Rename/move must perform explicit old-identity removal and new-identity creation or a canonical identity-preserving transition. Name/range/path similarity is not sufficient.

---

## 10. Invalidation and publication effects

The implementation distinguishes:

```text
recomputed queries/products
invalidated prior products
semantic products changed
source/presentation products changed
diagnostics changed
advisory products changed
snapshot published
```

A source mutation does not imply every formal product changed. A recomputed product does not imply its semantic fingerprint changed. Publication effects drive precise consumer refresh and observability.

---

## 11. Cancellation and concurrency

Cancellation checks occur at bounded points in discovery, linking, query evaluation, body analysis, advisory solving, and publication preparation.

A newer candidate may cancel an older candidate. Only a candidate based on the current committed revision may publish. No cancelled candidate may leave query-cache entries, contribution state, provider state, or module registration visible unless those entries are independently immutable and keyed so they cannot contaminate another revision.

One request may pin snapshot N while update N+1 publishes. The request remains coherent against N.

---

## 12. Performance and memory requirements

- ordinary body edits invalidate the dependency frontier, not the whole workspace;
- unchanged immutable products may be structurally shared after equivalence validation;
- reverse dependency and occurrence queries are indexed;
- retained snapshots, explanation graphs, advisory contributions, and caches have explicit bounds or ownership-based reclamation;
- metrics distinguish recomputation, change, invalidation, reuse, and publication.

Performance optimization cannot weaken transactional correctness or snapshot coherence.

---

## 13. Conformance requirements

Tests must cover:

1. open/change/save/close overlay precedence and identity preservation;
2. watched-file refresh with open and closed sources;
3. create/delete/rename across importers and reverse dependencies;
4. deleted provider/cache entries cannot resurrect modules;
5. semantic errors publish current coherent products;
6. cancellation/staleness/budget/internal failure preserves prior committed world;
7. candidate failure rolls back mutable query/session effects;
8. one mutation batch publishes one semantic revision;
9. old-snapshot readers remain immutable during concurrent publication;
10. cold and incremental final snapshots normalize equivalently;
11. narrow edits demonstrate bounded invalidation and truthful counters.
