# PERF003.C2.P5 — Canonical Image Continuation Handoff

**Program:** PERF003
**Checkpoint:** PERF003.C2
**Plan:** PERF003.C2.P5
**Status:** IN_PROGRESS
**Completion:** PARTIAL
**Verification:** FOCUSED_TESTED
**Date:** 2026-10-13
**Branch:** `perf/universe-characterization`
**Worktree:** `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization`
**Source revision:** `7c9bd6d7` (continuation over `9143262b`).
**Delivery:** source committed; lifecycle records accompany it on `perf/universe-characterization`.

## Established continuation

The earlier Universe-wide dependency bypass has been replaced with exact image
aggregate fingerprints. `CanonicalUniverseFormalImage::dependency_fingerprint`
consults frozen roots and its derived immutable structural index. Hierarchy
entries retain validated source-sensitive query fingerprints and require exact
agreement with final declaration/parent identity. Other approved structural
families use final aggregate fingerprints. Catalog/frozen overlap must agree.

`SemanticDb::dependency_fingerprint` preserves local-state precedence. Edge
recording, validation and immutable-leaf indexes use exact membership. An
aggregate-only dependency does not become a Ready/reusable query or typed query
product. Formal replay and computed-key readiness use exact evidence; invented
Universe identities fail. Query freezing excludes structural rows changed by
final settlement and rejects their dependent closure.

`ParentMap` and `with_parent` factories now cover dispatch/surfaces, conditional
member sets/type registrations, data, enums/variants, associated surfaces,
enum requirements and type-member declarations. Snapshot surfaces share the
dispatch map; associated resolver views avoid flattening canonical maps.
First updates use base overlays; subsequent updates clone local contributions.
Declaration/signature/hierarchy overlays remain established from Slice 1.

Alias result-kind inference from the inherited fix is retained. Static-definition
fingerprints now include formed alias semantics, and both publication variants
compare the freshly formed fingerprint before reuse. The imported-alias A6
regression passes with its original shell-publication assertion intact.
Cycles quarantine participants and transitive alias dependents, removing alias
products, declarations, static definitions/cached shells and retained forms.
Unrelated aliases survive. Stale global-empty test assertions now check exact
invalid identities; diagnostics and declaration-absence checks remain.

## Invariants and rulings

See CHECKPOINT.md for adopted Level 1 adviser clarifications. Frozen query
eligibility and aggregate dependency authority are distinct; never restore a
Universe-wide bypass or synthesize Ready rows. Parent mutation/collision policy
stays strict: foreign inherent impls of canonical targets are forbidden by
`docs/specs/objects/impl.md`. Independent focused review has no remaining
findings after correcting a computed-key readiness consumer.

## Completed verification

- Incremental recomputation filter: 13 passed (120.97 s), including all four
  inherited failures.
- Workspace integration filter: 18 passed (44.89 s), including new
  valid→cycle→valid quarantine/preservation regression.
- Alias capability filter: 7 passed (6.57 s).
- Final canonical-image library filter: 12 passed (41.84 s), including exact
  aggregate dependency evidence and final-settlement freeze rejection.
- Parent library filter: 10 passed (0.06 s).
- Canonical-image integration: 4 passed (123.33 s), including source parity,
  provider isolation and actual canonical sharing across local replacements.
- Core with observability and LSP checks passed; LSP retains an unrelated
  unused import warning.
- Computed Record-key callable-signature regression passed (one selected).
- Scoped diff whitespace check passed. Full format/workspace/release and
  benchmark certification were deliberately deferred.

## Next work

`TypeAliasTable`, `StaticDefinitionTable`, `StaticConstantTable`, and accepted
callable-definition maps now parent-read the image. Retained alias forms share
the snapshot alias table; resolver static-definition views no longer flatten
canonical maps. The exhaustive `canonical_query_persistence(QueryKind)` is
consumed by freeze; eligibility never bypasses Ready/current/final-content and
dependency-closure checks. Exact canonical sharing, removal/collision safety,
13 recomputation, 18 workspace and seven alias tests passed after the table
migration; 13 image library tests passed after classification extraction.
The final source increment `eeca76f3` passed the exact image replacement,
focused callable and core check. The pre-existing
test-harness unused-import warning persists.

Continue residual P5.T6 Slice 4 static-case catalogs/signatures, associated
static domains, module static bindings and other copied aggregates, then T7–T8
read/persistence contracts and T11 observability. T9's family classification
is implemented; P7 still needs an artifact DTO/relocation contract. Do not claim
zero copies across all product families yet. T10/final acceptance remain gated on P4 metadata
composition. P5 does not persist `.telosc` and still source-builds the image
on first process access.

Keep unrelated worktree state intact. Commit and push were authorized on
2026-10-13. Broad workspace/release gates and speedup certification were
deliberately deferred. Delivery does not promote P5 to COMPLETE.

## Continuation at `ca3c3c03`

Module bindings, constructor paths and trait headers/surfaces now parent-read
the image. Canonical-image integration (four selected), workspace integration
(18), module-value dependency regression, parent-map tests and core check
passed. Parent/local count checks passed in exact three-revision replacement.
Static-case catalogs/signatures have zero entries in this canonical image;
no general parent contract is established. Conformance index, witness plans
and derived-rule buckets still copy canonical entries. The user approved a
layered canonical/local conformance view with exact joint query/coherence
parity; no P5.G6 exception was approved.

The user resumed P4 prerequisites and approved loader-supplied Symbol identity
for v12→v13 normalization when an old bundle lacks a Symbol node. The current
exact singleton failure is public `FixedFields`: metadata cannot encode its
non-Symbol Record remainder. PR-A Record/Pack facade coverage passes; PR-B is
not implemented and the exploratory schema change was removed. Continue PR-B
schema, global/scoped/row export, validation, fingerprints, contextual legacy
decoding, runtime typing and canonical RuntimePublic proof before P5.T10.
P5 stays `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.

## Layered conformance progress

`ConformanceIndex` and `DerivedConformanceRuleIndex` now attach immutable
canonical parents on the first local revision. Exact candidates, stable-ID
iteration, derived trait buckets and canonical/local overlap queries are
composed without copying canonical buckets. New cross-boundary brute-force
parity, derived-rule parent safety and image replacement tests pass, along
with focused derived cold/incremental parity and a core check. Current
canonical witness plans have zero entries, but a general parented witness
contract is still needed before declaring all P5 families complete. P4.PR-B
remains the metadata gate; do not infer P5 completion or `.telosc` persistence.
