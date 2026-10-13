---
id: LANG007.C5
category: LANG
program: LANG007
checkpoint: LANG007.C5
kind: checkpoint-record
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
prepared: 2026-10-13
prepared_against_revision: b9c8ddd9551c2e15a8a728085f2f259a3e92b6d3
requires:
  - existing canonical runtime closed-sum matching and executable projection
  - canonical Universe module and generic declaration ownership
---

# LANG007.C5 — Interval algebra, immutable Progression, and typed relational matching

## Objective and ownership

Deliver interval and range relationship values that ordinary exhaustive `match`
can decompose, including shared regions, touching boundaries, and the portions
left after subtraction. Provide a substantial hotel quotation example that uses
these results to diagnose conflicts and construct stays across rooms.
Also deliver immutable affine Progression values, independent canonical
iteration, and phase-preserving interaction with intervals and numeric ranges.

The selected surface is:

```phalcom
match requested.relate(existing) {
    IntervalRelation::Intersects(shared, requestedOnly, existingOnly) => ...
    IntervalRelation::TouchesAfter(boundary, gap) => ...
    // remaining closed-sum cases
}
```

The user selected typed relationship values on 2026-10-13. This selection settles
the surface direction for planning; the detailed API and boundary rules in the
companion design remain proposed until the P1 decision gate records adoption.
The user subsequently authorized implementation in a new isolated worktree on
2026-10-13. Detailed contract adoption and execution evidence remain separate
G1 records; implementation authorization is already established.

C5 owns a bounded expansion of LANG007's structural elimination concern:
ordinary library values expose interval algebra to the existing canonical match
engine. It does not introduce a second pattern resolver or type solver.

Owns:

- immutable validated intervals with explicit ordered domains;
- structural views, set relations, and finite proper-span Allen relations;
- intersection, subtraction, union, and explicit convex hull;
- numeric Range adapters with an explicit membership interpretation;
- immutable Progression recipes, finite transforms, clipping, and canonical iteration;
- semantic, compiler, runtime, and incremental acceptance evidence;
- the hotel example as an executable acceptance fixture after implementation.

Does not own:

- new match keywords, guards, extractor protocols, or scalar range patterns;
- generic nominal-product pattern syntax;
- general interval constraint networks or Allen relation-set composition;
- an interval tree, a general multirange collection, or a query optimizer;
- Date/TimeZone redesign, recurrence, calendar uncertainty, or persistence;
- production booking transactions or distributed inventory locking;
- arbitrary user-defined ordered domains or implicit numeric coercions;
- automatic static evaluation of ordinary interval method sends.

## Records and plan ledger

| Record | Purpose |
| --- | --- |
| [Design](interval-relational-matching-spec.md) | Proposed semantics, alternatives, cases, and ownership |
| [Progression design](progression-spec.md) | Immutable recipes, steps, iteration, exact clipping, and bridges |
| [Hotel example](hotel-booking-example.md) | Algorithms, worked inventory, expected decisions, and acceptance scenarios |
| [P1 plan](LANG007.C5.P1-interval-algebra-and-relational-matching-plan.md) | Ordered implementation tasks and bounded gates |

| Plan | Scope | Status | Completion | Verification | Outcome |
| --- | --- | --- | --- | --- | --- |
| LANG007.C5.P1 | Contract adoption, interval/Progression algebra, Range bridges, matching, hotel fixture | PROPOSED | NOT_STARTED | UNVERIFIED | Planning complete; user authorized isolated implementation next |

## Inherited established boundaries

These are inherited contracts, not claims that C5 exists:

1. `MatchResolution` and `PatternResolution` own canonical pattern meaning.
2. Runtime closed sums are eliminated using declaration-backed `VariantId`
   and payload identities; lowering consumes the semantic projection.
3. Scrutinees evaluate once and match execution stages bindings before commit.
4. Guarded or arbitrary runtime computations cannot silently contribute formal
   interval-order proofs or coverage of the original inputs.
5. Universe source declarations own library identity and exports. A source-only
   interval algorithm does not require an interval opcode or native floor entry.

## Proposed invariants

The design's `INV-01` through `INV-12` and Progression `PR-01` through `PR-08`
are proposed C5 acceptance contracts.
Move adopted rules into the established ledger only after G1 records the
decision; record implemented interfaces only after the relevant task gate.

## Stable takeover map

| Concept | Current owner | C5 use |
| --- | --- | --- |
| Source patterns | `phalcom-ast/src/ast.rs`, `Pattern` | Existing variant, nested, wildcard, and or-pattern grammar |
| Formal matching | `phalcom-semantic/src/match_semantics.rs` | Match result, payload bindings, coverage, branch case knowledge |
| Resolution and coverage | `phalcom-semantic/src/checker/pattern.rs`, `checker/coverage/` | Ordinary closed-sum analysis; no interval-special coverage |
| Executable projection | `phalcom-core/src/modules/semantic_lowering.rs` | `ExecutablePattern`, `MatchLoweringSpec` |
| Runtime match lowering | `phalcom-core/src/compiler/lib/match_expr.rs`, `patterns.rs` | Ordinary source-installed data values |
| Range observations | `phalcom-core/core/universe/src/collections/range.ph` | `rawLower`, `rawUpper`, `rawUpperInclusive` |
| Civil date order | `phalcom-core/core/universe/src/time/calendar/date.ph` | `compare`, `daysSince`, `advance` |
| Absolute time order | `phalcom-core/core/universe/src/time/timestamp.ph` | `compare`, `unixNanoseconds`, `since` |
| Builtin source delivery | `phalcom-modules/src/builtin.rs` | New opt-in `universe.intervals` source paths |

## Verification ledger

| Gate | Scope | Evidence | Result |
| --- | --- | --- | --- |
| Planning inspection | Named specs, source owners, test conventions, current revision | Read-only source inspection on 2026-10-13 | Design grounded; no behavioral verification |
| Planning artifact check | Local links, worked date calculations, algebra case partition | See delivery report for checks actually completed | Does not certify implementation |

No Cargo build, test, or lint was run for this documentation task. C5 remains
`PROPOSED / NOT_STARTED / UNVERIFIED`.

## Decision and prerequisite ledger

| ID | Observation / decision | Consequence |
| --- | --- | --- |
| D1 | User selected typed relationship values over direct relational patterns | Use `match a.relate(b)`; no keyword or matcher-protocol work |
| D2 | `collections/range.md` and `collection-literals.md` say operators are inactive; live parser, compiler, source Range, and fixtures use `..` / `..=` | G1 must reconcile authority explicitly; current source is evidence, not permission to amend a normative rule silently |
| D3 | Live `Pattern`/`PatternResolution` do not offer general interval/data-object deconstruction; match arms have no Boolean guard field | Structural examples match `interval.view`; do not claim new direct product patterns or guards |
| D4 | Some skill overlays and older program indexes describe earlier language states | Use current specs and named live owners; do not reinstate the historical “no match” model |
| D5 | Existing C4 state includes partial work and recorded bootstrap/lowering issues | Rebaseline only the exact generic sum / Universe / projection seam required by C5; do not assume C4-wide certification or repair unrelated work |

These are planning observations. No failing C5 test or current baseline test
failure was reproduced. Do not report historical failures as newly confirmed.

## Acceptance

Checkpoint closure requires all of the following:

1. The contract and Range authority conflict are resolved at G1.
2. The four supported domains implement all endpoint, empty, singleton,
   unbounded, validation, and cross-domain cases in the design.
3. `relate` partitions every valid input pair into exactly one of its eight
   cases and returns correct algebraic payloads.
4. Finite proper spans support all thirteen Allen relations and converses;
   other inputs return `None` rather than an invented Allen result.
5. Structural views and relationship values use the existing match engine;
   omission and redundancy diagnostics, payload binding types, generic
   reification, incremental retraction, and execution order are verified.
6. Numeric Range conversion preserves its chosen membership domain and leaves
   iteration, slicing, equality, hashing, and existing syntax behavior intact.
7. Progression validates signed strides/extents, iterates independently, retains
   recipe lifetime, and clips/subtracts exactly without scanning or shifting phase.
8. The hotel fixture computes the worked conflicts, available pieces, valid
   alternatives, ranking, and housekeeping boundaries exactly.
9. P1 walkthrough/handoff and this shared record describe completed evidence,
   adopted decisions, deferred failures, and the next action truthfully.

Closure target is `IMPLEMENTED / FOCUSED_TESTED`; workspace/release
certification is a separate request.

## Active work and next action

```yaml
active_plan: null
latest_completed_plan: null
next_plan: LANG007.C5.P1
next_action: >
  Review and adopt the proposed detailed contract at P1.G1, resolve the Range
  authority conflict, and establish the exact imported generic-sum baseline.
  Create the requested isolated worktree, then execute P1.T2 through T8 under
  the bounded verification budget.
```

Adding proposed C5 does not change the program's active C4 execution state.
