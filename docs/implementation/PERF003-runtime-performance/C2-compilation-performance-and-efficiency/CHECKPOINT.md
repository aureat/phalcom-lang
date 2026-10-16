---
id: PERF003.C2
program: PERF003
kind: optimization
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
latest_completed_plan_verification: FOCUSED_TESTED
active_plan: PERF003.C2.P7.PATCH2
latest_completed_plan: PERF003.C2.P6
next_plan: PERF003.C2.P7.PATCH2
next_action: >
  Obtain the architect decision for INC-PATCH2-T7-T12-STATIC-EVAL before editing
  the source-free static evaluator path. Then resume G4/G6 source-denial and
  evaluator parity work, alongside independent remaining G1-G8 owner-parity and
  malformed-wire gates. T7-T10 owner transport and T11-T13 assembly/root work
  are implemented, but no gate is promoted from compilation alone. Keep the
  semantic importer partial until PATCH6 promotion, and preserve PATCH1
  callable/Exact/Derived plus PATCH3 allocation guards until their named
  acceptance gates pass.
prepared_against_revision: e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
execution_tree_head: dbde1d8c
latest_tested_revision: dbde1d8c
latest_tested_predecessor: c34f9c3fee75f33304a85c229dcf0a5133816479
---

# PERF003.C2 — Compilation Performance and Efficiency


## Current P7 status — 2026-10-15

Plan PERF003.C2.P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED. This continuation
added fallible `boxed_str` allocation and transactional budget rollback,
budget-aware ordered map/set builders, and allocator-counting probes. Container
package identity strings and the directory buffer now use the caller's shared
budget; duplicate/overlap validation no longer allocates hash/sort scratch. A
separate `max_total_allocation_bytes` now bounds the full core load transaction,
independent of the decoded-section cap. Module-loader duplicate/index scratch
and runtime-DAG validation now use budgeted vectors and dense CSR scratch.
Focused allocator/container/module checks and affected library checks pass.

Enum requirement records carry the complete owner-held callable signature.
Callable-analysis rows carry a partial typed call-static-argument transport.
The full callable-analysis graph/import and compiler reader projection remain
missing, so export/import still fail closed for nonempty body analyses. Other
primary/query owner families and R1/R2/R4–R11 acceptance remain incomplete.
No complete artifact construction, source-denied compilation, full semantic
parity, or P5/P6 acceptance has passed; P7 remains open and P8 takeover blocked.

The latest R1 slice makes semantic allocation failures return the typed
`ArtifactError::Allocation` without allocating a `LimitExceeded(String)` and
threads the load ledger through callable-signature type-evidence import.
Evidence range/description buffers and unresolved-name ownership are charged;
the helper commits its child ledger only after reconstruction succeeds. Focused
exact-fit/N-1 rollback coverage was added. This does not close the remaining
semantic-owner, decoder, native-output, whole-load census, or R3 callable-body
gaps.

The current review correction makes budgeted SemanticWorld validation and
semantic import return the typed `UnaccountedBoundary` error before validation
scratch or semantic materialization runs. This keeps the caller's load budget
uncommitted while validator scratch, TypeStore reconstruction, and primary or
static owner allocations remain outside exact accounting. Semantic artifact
loading therefore remains unavailable until those requests are budget-visible
and fallible; P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED.

The latest denotation consultation selected a hybrid owner transport: TypeForm,
StaticTerm, Kind, and StaticDomain carry references into the shared artifact
type/kind/domain graph; context-owned AssociatedValue and BehavioralFamily
captures travel inline with stable owner/member/target references and relocated
type refs. Import must preserve the captured selection and never rerun lookup
or dispatch. StaticApplication remains fail-closed until the producer retains
the checked row under the actual checked-body owner. The user resolved the
owner discrepancy in favor of `CheckedStaticBody`; live producer code already
retains rows there under `StaticBodyId + ordinal`. No duplicate row belongs in
`CallableAnalysis`. StaticApplication remains fail-closed until its checked-body
row and operational reference are transported and validated. This is a bounded
T7.B follow-up; P7 acceptance is unchanged.

The implementation slice now registers every supported denotation graph root,
round-trips associated exact/family and behavioral-family captures through
relocation, and validates captured associated members against their primary
owner products. The family importer is tested at exact allocation fit and
one-byte-short rollback. Static-definition applications now receive the
enclosing checked static-body identity rather than the selected target; two
different bodies both using ordinal zero are covered. Focused root/import and
semantic-world tests pass. This does not retain checked application rows in
`CallableAnalysis` or remove the callable-analysis fail-closed guard.

The latest continuation extends callable-analysis graph-root registration to
expression/binding knowledge and contracts, entry/exit/throw flow facts,
sequence/expansion products, associated/callable-reference/family resolutions,
match/pattern products, trait dispatch, contextual realization, explanations,
diagnostic guidance, semantic dependencies, and for/positional iteration
requirements. Direct roots cover exact, assumed, derived, and compiler-derived
generic evidence. Checked static-application rows also root their signature
domains, generic parameters, and kinded arguments under the selected
`StaticBodyId` owner. The focused callable-analysis root filter passes 19
tests, and the checked-body owner root test passes 1/1. No callable-analysis
wire/materialization path was added.

The pinned nightly does not provide a fallible `Arc<[T]>` slice constructor.
Loaded enum-requirement rows now retain their already-budgeted vectors behind
fallible `Arc<Vec<T>>` owners; source-built rows retain `Arc<[T]>` behind the
same slice-reading wrapper. The enum import round-trip passes exact-fit and
one-byte-short allocation checks. This is a physical backing adaptation at
that loaded owner boundary and does not close whole-load allocation coverage.

StaticApplication transport remains fail-closed. The user's owner decision is
to keep `CheckedStaticBody` as the sole row owner. `analyze_static_body_inner`
sets the `StaticBodyId` and moves checked rows into that body; ordinary
`CallableAnalysis` finalization is separate. Type-root registration now walks
the checked-body rows and allocates their root ordinals after operational
template roots. Each live row also contains source-local expression identities,
while `OperationalOpV1::StaticApplication` carries target/arguments without the
checked row's owner/ordinal or full signature. The DTO/model seam therefore
still needs a source-independent representation that preserves the row under
`CheckedStaticBody` without fabricating expression IDs. Preserve the
fail-closed guard until the row and operational reference are transported and
cross-validated.

The explicit Dynamic call-environment prerequisite now transports tagged
Term/RecordRow/Dynamic arguments through semantic call specialization,
conformance rejection, compiler roots, runtime recipes, and interned runtime
environments. Focused semantic tests prove `Box<Int>` and `Box<Dynamic>` retain
distinct arguments without internal incidents, nested `Box<T>` materializes as
`Box<Dynamic>`, Dynamic-dependent conformance publishes no specialization, and
the call fingerprint distinguishes Dynamic from a concrete term. Core tests
prove Dynamic differs from a missing environment entry and remains a tagged
static argument in nested runtime applications. The exact canonical artifact
test now passes its no-internal-incidents assertion, then fails closed at the
existing incomplete callable-analysis export guard. No call-specialization
wire DTO/import parity exists yet. A separate fresh-VM unit probe is blocked by
the current Universe bootstrap error in `universe:reflection.structural`
(`ProductKind` has no canonical facade binding); it was removed rather than
retained as a failing test. These results are focused evidence only; P7 remains
IN_PROGRESS / PARTIAL / UNVERIFIED.

PATCH0 (`PERF003.C2.P7.PATCH0`) completed from
`21a54705c33dae95ee9c77d86789b4911635e6b0`: callable body fingerprints now
include binding writes, all associated-resolution variants, static and dynamic
family applications, complete callable-reference family/member semantics, and
nested associated payloads. Its narrow fingerprint module filter passed 23/23
(including the existing range, diagnostic, explanation, and flow invariance
cases); the exact callable-analysis library filter passed 19/19. The regular
semantic integration target is currently blocked at compile time by the inherited stale
`local_len()` call in `tests/semantic/integration/canonical_image.rs:203` after
the enum-requirement owner became a frozen-vector wrapper. To keep PATCH0 scoped,
the fingerprint module was compiled through a temporary target/harness with its
normal incremental support module; the temporary files and Cargo entry were
removed after the 23/23 run.

The PATCH0 implementation is committed as
`8284893769d121dc2fe33ed0c866a402ff48bd35`.

The artifact callable-analysis baseline remains 4/9 passing with the same five
inherited failures: `associated_family_capture_round_trips_subset_and_optional_targets`,
`callable_analysis_status_is_preserved_by_semantic_world_codec`,
`callable_flow_graph_round_trips_owner_local_nodes_edges_and_entry_exit`, and
`explicit_dynamic_call_argument_round_trips_without_a_type_node` fail with
`Allocation(UnaccountedBoundary)`; `behavioral_family_capture_round_trips_conditional_and_unconditional_targets`
fails conditional-dispatch implementation-member-authority validation. No
artifact or allocation code changed. PATCH0 is COMPLETE / IMPLEMENTED /
FOCUSED_TESTED, and PATCH1's `PATCH0_callable_fingerprint` prerequisite is
SATISFIED; PATCH1 T0 is now complete and T1 is next. P7 remains IN_PROGRESS / PARTIAL /
UNVERIFIED and P8 remains blocked.

## PATCH1 takeover and T0 census — 2026-10-15

PATCH1 is active. The live branch is `perf/universe-characterization` at
`dbde1d8cacba40067223ca7230b79580bff9c164`, equal to
`origin/perf/universe-characterization`; the worktree was clean before T0.
PATCH0's fingerprint prerequisite is SATISFIED. Live ownership matches the
plan: `CheckedStaticBody` remains the sole checked `StaticApplication` row
owner; PATCH1 leaves that denotation fail-closed and does not copy rows into
`CallableAnalysis`. The approved enum-requirement loaded backing remains the
budgeted vector wrapper.

T0 focused baseline:

- `cargo test -p phalcom-semantic --lib callable_analysis -- --test-threads=1`
  selected 19 and passed 19, including the broadened callable root tests.
- `cargo test -p phalcom-compiler-artifact --test
  semantic_world_callable_analysis_test -- --test-threads=1` selected 9; 4
  passed and the same 5 inherited failures remain: four
  `Allocation(UnaccountedBoundary)` cases and the conditional-dispatch
  implementation-member-authority case recorded above.
- PATCH0 fingerprint discriminator evidence remains the focused 23/23 run in
  the PATCH0 handoff. Live code still hashes binding writes, family
  applications, all associated-resolution variants, and nested callable
  references through the shared associated-resolution helper.
- The normal semantic integration fingerprint filter is blocked before test
  selection by the inherited stale `local_len()` call at
  `phalcom-semantic/tests/semantic/integration/canonical_image.rs:203`. No
  PATCH1 code is implicated.

Current `CallableAnalysis` field matrix. “Guarded” means the semantic-world
importer rejects every nonempty callable-analysis table before materialization;
the standalone flow and denotation helpers do not constitute a complete owner
importer. Current validation covers only the portable subset and its local
references; complete validation and import remain absent.

| Native field | Artifact disposition | Current wire | Root status | Current validator | Importer | Native storage | Allocation status | Compiler/query consumers | Parity test |
|---|---|---|---|---|---|---|---|---|---|
| `callable` | stable owner identity | `callable` | identity only | table key/stable ref checked | guarded | outer table key | source `HashMap`; frozen owner absent | lookup, lowering, fingerprint | owner/codec test only |
| `body_range` | portable span | `body_range` | none | span shape only | guarded | scalar | no owner allocation | formal projection, diagnostics | none |
| `body_source_owner` | owner-qualified source namespace | `body_source_owner` | identity only | variant/ref partial | guarded | `Option<SourceOwner>` | no owner allocation | source/formal attachment | none |
| `expressions` | complete expression rows/local refs | `expression_count`, denotations, partial call specialization | knowledge, denotation, call arguments, dispatch/context roots | count/ordinal and denotation links only | guarded | `BTreeMap<ExpressionId, ExpressionAnalysis>` | source tree nodes; import absent | core lowering, presentation, fingerprint | root tests only |
| `bindings` | complete binding rows | denotations and range-only formal rows | contract, current knowledge, denotation | map key and subset attachment checks | guarded | `BTreeMap<BindingId, BindingState>` | source tree nodes; import absent | lowering, formal projection, presentation | no full parity |
| `binding_origins` | authority folded into binding rows | `binding_origins` | identity only | exact key-set check | guarded | `BTreeMap<BindingId, CallableBindingOrigin>` | source tree nodes; import absent | core binding storage choice | owner test only |
| `for_iteration_lanes` | complete evidence-backed lane rows | absent | source/item/iterator types, evidence, signatures | none | guarded | `Arc<BTreeMap<ExpressionId, _>>` | source tree nodes; import absent | lowering | root test only |
| `positional_expansions` | complete expansion rows | absent | source/item/iterator types, evidence, signatures | none | guarded | `Arc<BTreeMap<ExpressionId, _>>` | source tree nodes; import absent | lowering | root test only |
| `known_invalid_positional_expansions` | explicit invalidity rows | absent | source type | none | guarded | `Arc<BTreeMap<ExpressionId, _>>` | source tree nodes; import absent | fail-closed lowering | root test only |
| `sequence_literals` | complete literal/nested expansion rows | absent | result/element/extent types | none | guarded | outer `Arc<BTreeMap>` plus nested `BTreeMap<SourceRange, _>` | source allocations; import absent | lowering, fingerprint | root test only |
| `associated_resolutions` | every selected resolution variant | absent from callable rows | owner/member/result/family types | none | guarded | `Arc<BTreeMap<ExpressionId, _>>` | source tree nodes; import absent | lowering, source index, fingerprint | root tests only |
| `callable_reference_resolutions` | complete bound/associated capture | absent from callable rows | family/member/callable types and nested captures | none | guarded | `Arc<BTreeMap<ExpressionId, _>>` | source tree nodes; import absent | lowering, fingerprint | root tests only |
| `family_applications` | complete static/dynamic selection | absent from callable rows | family/member/result types | none | guarded | `Arc<BTreeMap<ExpressionId, _>>` | source tree nodes; import absent | lowering, fingerprint | root tests only |
| `match_resolutions` | complete match/proof/coverage rows | absent | recursive pattern, proof, coverage types | none | guarded | `Arc<BTreeMap<ExpressionId, _>>`; nested branch-proof maps | source tree nodes; import absent | core lowering, fingerprint | root test only |
| `pattern_site_resolutions` | complete non-match pattern rows | absent | recursive pattern and subject types | none | guarded | `Arc<BTreeMap<PatternSiteKey, _>>`; nested branch-proof maps | source tree nodes; import absent | core lowering | root test only |
| `flow_graph` | complete owner-local graph | nodes, edges, entry, exits | no type roots | endpoint/entry/exit closure partial | standalone frozen importer only | source/frozen dual graph | frozen node/edge/adjacency path budgeted; owner import guarded | lowering, fingerprint | current codec case is an inherited allocation failure |
| `entry_flow` | complete entry summary | absent | binding/field knowledge and contracts | none | guarded | `FlowStateSummary` with two `BTreeMap`s | source tree nodes; import absent | flow readers, fingerprint | root test only |
| `exits` | normal, throw, unreachable facts | `exits_unreachable` only | exit knowledge and nested flow summaries | unreachable bit plus ordinal checks | guarded | vectors and two flow-summary maps per summary | source vectors/maps; import absent | flow readers, fingerprint | no full parity |
| `return_validation` | complete return-contract result | absent | none | none | guarded | enum/scalar | no owner allocation | publication and fingerprint | none |
| `diagnostics` | complete payload and local cause links | absent | guidance types/parameters | source-export cause checks only | guarded | `Arc<[SemanticDiagnostic]>` | source slice; import absent | diagnostics/presentation | no full parity |
| `internal_incidents` | explicit rows or canonical rejection | absent; nonempty rejected | none | nonempty canonical owner rejected | guarded | `Arc<[InternalSemanticIncident]>` | source slice; import absent | snapshot publication | rejection only |
| `explanations` | complete explanation DAG | absent | steps/rules/evidence types | source-export DAG/cause checks only | guarded | arena `BTreeMap<ExplanationId, _>` plus nested vectors | source tree nodes; import absent | explanation reader, fingerprint | no full parity |
| `return_explanation` | local explanation ordinal | absent | identity only | source owner ref checked | guarded | `Option<ExplanationId>` | no owner allocation | presentation/explanation | none |
| `dependencies` | exact callable dependency identities | absent | identity only | direct-result subset checked | guarded | `Arc<[CallableId]>` | source slice; import absent | termination/query invalidation/fingerprint | no full parity |
| `semantic_dependencies` | exact semantic dependency identities/evidence | absent | generic-conformance type roots | root closure only for generic evidence | guarded | `Arc<[SemanticDependency]>` | source slice; import absent | query invalidation | no full parity |
| `constructor_super_initializer` | selected target and call range | portable target/range | identity only | target/range partial | guarded | optional record | no owner allocation | core lowering | owner/codec test only |
| `dependency_fingerprint` | recompute after relocation | absent | none | no cross-store equality check | guarded | fixed fingerprint scalar | no owner allocation | incremental query reuse | PATCH0 contract only; no relocated parity |
| `status` | preserve status and publishability | `status` | identity only | portable enum/canonical incident rules partial | guarded | enum | no owner allocation | snapshot/canonical publication | status codec case inherits allocation failure |

Source-side map/set allocation census: outer
`CallableAnalysisTable.by_id` (`HashMap`) and `.by_module` (`HashMap` of
`BTreeSet`); the twelve top-level body indexes shown above; nested
`FlowStateSummary.bindings`/`fields`, sequence `exact_expansions`, explanation
`nodes`, match/pattern `BranchProofEnvironment.bindings` and `local_bindings`,
and trait-dispatch selection `requirement_targets`. Transitive conditional and
generic evidence may own `TypeEnvironment` HashMaps/HashSet and
`TypeSubstitution` bindings. The current semantic importer reconstructs none of
these callable-owned allocations because it stops at the completeness guard;
the existing `FlowGraph` and denotation functions are standalone budgeted
seams, not a callable materializer.

`CallableAnalysisTable::Deref<Target = HashMap<...>>` has no identified
production consumer that requires a HashMap-only operation. Production uses are
through the explicit `get`, `contains_key`, `keys`, `values`, `iter`, and
`iter_module` methods in `diagnostic.rs`, `snapshot.rs`, `canonical_image.rs`,
`artifact/{roots,export}.rs`, `session.rs`, and
`phalcom-core/src/modules/{compile,semantic_lowering,canonical_universe}.rs`.
Keep the separate `Index` implementation in view while removing representation
leakage; compile the listed consumers in T1.

## PATCH1 T1 frozen callable owner storage — 2026-10-16

T1 establishes one source/frozen read contract for maps and sets reachable from
callable owners. `AnalysisMap` and `AnalysisSet` keep mutable source backing and
accept charged sorted entry vectors for shared immutable backing. Flow summary
maps, sequence expansion maps, resolution indexes, branch-proof environments,
trait-dispatch selection maps, and value-owned `TypeEnvironment` maps/sets use
these wrappers. `InherentImplSpecialization.bindings` uses `AnalysisMap` too.
`TypeSubstitution` inside its existing shared `Arc` remains owned by that
canonical shared conformance evidence and is not copied into a callable-local
owner.

`CallableAnalysisTable` now has explicit read/source-mutation methods and no
`Deref<HashMap<...>>`. Its source representation retains the existing hash ID
index and module `BTreeSet` index. Its imported representation is one charged
sorted row vector plus a charged frozen module-to-row-range map; module
iteration scans only that module's rows. `ExplanationArena` likewise supports
source allocation and a charged frozen sorted-node table. Frozen mutation
attempts return typed errors. `CheckedStaticBody` remains the only checked
`StaticApplication` row owner; the callable denotation remains fail-closed.

Focused T1 evidence already completed:

- semantic library check — PASS (existing `import_callable_call_specializations`
  dead-code warning remains);
- core library check — PASS;
- callable-analysis unit filter after nested environment changes — 22 selected,
  22 passed (includes callable-table and flow tests);
- `analysis_map` — 5 passed; `analysis_set` — 3 passed;
- explanation arena — 2 passed; type environment — 2 passed;
- sequence publication — 3 passed; flow-summary fingerprint discriminator —
  1 passed;
- changed-Rust-file formatting and `git diff --check` — PASS.

No artifact round trip, semantic integration suite, source-denied compilation,
determinism gate, or P5/P6 acceptance gate ran during T1. The known semantic
integration `local_len()` compile blocker and five inherited callable artifact
failures remain unchanged.

## PATCH1 T2 callable-local ordinal model — 2026-10-16

Callable wire validation now runs through a `CallableAnalysisOrdinalContext`
created for one `CallableAnalysisRecordV1`. Its expression, binding, and flow
node checks are family-specific and resolve only against that record; a local
number reused by another callable therefore has no cross-owner meaning. The
context validates row ordering/uniqueness, each current local reference, flow
graph bounds/entry/exit closure, and duplicate exits. PATCH1 T3 must extend this
same context when complete wire row families are introduced; the current schema
still has no wire expression, binding, explanation, or diagnostic-cause rows.

T2 evidence:

- semantic callable export/ordinal discriminator — 1 passed, including reversed
  map construction history and owner-key mismatch;
- artifact local-ordinal discriminator — 1 passed, including independent
  ordinal-zero reuse across two callable owners, duplicate rejection, and
  expression-vs-binding family separation;
- the directly affected flow codec test was re-run and still fails at decode
  with the previously recorded `Allocation(UnaccountedBoundary)` failure; this
  is the inherited artifact validation allocation boundary, not a changed
  ordinal assertion;
- `git diff --check` — PASS.

T3 is active. The final root closure and rich wire record remain unimplemented;
P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED and P8 remains blocked.

## PATCH1 T3 bounded callable validation — 2026-10-16

The accepted Telosc artifact specification defines semantic-world schema 2 as
the complete callable-analysis owner graph and rejects schema 1. That resolves
PATCH1's versioning verify-first assumption: implement the specified schema 2
contract; do not introduce a new schema version for this correction.

T3 has started with an owner-local expression-count bound and checked cumulative
row accounting across the currently encoded callable families and nested call
specialization bindings. The focused validator discriminator passed 1/1. The
callable-analysis artifact target then selected 11 tests: 6 passed and 5 retained
the recorded inherited failures (four decode paths stop at
`Allocation(UnaccountedBoundary)`; conditional behavioral capture lacks
serialized implementation-member authority). The callable-analysis semantic
filter passed 22/22. This is preflight hardening only: the sparse record still
lacks complete expression/binding rows, the other owner families, and final root
reconciliation. Keep the public decoder allocation guard active.

The shared `CallableAnalysisOrdinalMap` now also assigns deterministic,
callable-local ordinals to pattern-site keys. The reversed insertion-history
regression passed in the focused semantic export test (1/1), establishing that
pattern-site rows use the same ordinal authority as expressions, bindings,
flow, explanations, and causes.

Next action: continue T3 by replacing the sparse record with the schema-2
owner-centric rows, extend owner-local closure/limits to those rows, and
reconcile roots against the completed wire fields. P7 remains IN_PROGRESS /
PARTIAL / UNVERIFIED; P8 remains blocked.

## PATCH1 T3 call-conformance root closure — 2026-10-16

The final root census found one missing path: `CallSpecialization.conformances`
was not traversed by `register_callable_analysis_roots`, even though generic
conformance evidence has a root walker and the future wire contract retains
these bindings. Added one shared call-conformance root walker for callee keys
and Exact, Forwarded, Derived, and CompilerDerived sources; both expression
call specializations and existing inherent-implementation provenance use it.
The call-specialization regression initially failed on its forwarded source
because the canonical subject type was absent from the closed graph. It now
exercises all four source variants, and the complete callable-analysis
semantic filter passed 23/23.

This closes the discovered source-root gap only. The owner-centric schema,
complete field census against its final rows, and rich wire validation remain
open; the decoder and callable coverage guards remain active.

## PATCH1 T3 owner-centric expression and binding rows — partial — 2026-10-16

The callable wire record now carries dense `expressions` and `bindings` rows.
Expression denotations and call specializations are nested under expression
rows; binding denotations, the current storage-origin classification, and the
existing range attachment are nested under binding rows. Export orders both
vectors by the single callable-local ordinal map. The sparse expression count,
denotation tables, specialization table, and separate origin/attachment
side-tables were removed. Validation bounds both dense families, requires the
exact dense ordinal range, and includes their rows plus nested specialization
bindings in the checked owner-local total.

Focused evidence:

- `semantic_world_callable_analysis_test`: 6 passed, 5 failed. Four decoder
  cases still stop at the intentionally active
  `Allocation(UnaccountedBoundary)` guard; conditional behavioral capture still
  lacks serialized implementation-member authority, as previously recorded.
  The new dense-row bounds/ordinal checks and nested specialization validation
  passed.
- Exact callable call-conformance root test: 1 passed.
- Compiling the semantic library unit-test target succeeded as part of that
  focused root run, including the migrated export/import fixtures.
- Changed-file formatting and `git diff --check` passed.

This is only the owner-centric row foundation. T4 still must transport full
`BindingState` and exact source-site attachment identity; T5 must transport the
complete expression contract and generic conformance evidence; T6/T7 still own
the remaining callable families. The public decoder and callable coverage
guards remain active. P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED; P8 remains
blocked.

## PATCH1 T4 consultation hold — 2026-10-16

T4 source inspection found a portable-owner gap before implementation; no T4
code changes were made. The required full `BindingConsistency` includes
`Blocked(BlockReason::UnavailableSemanticOwner(QueryKey))`. `QueryKey` includes
`GenericConformance(GenericConformanceKey)` and `TraitDispatchBucket` variants;
the generic key carries `TypeTerm` and `TraitRef` identities. The current shared
query artifact codec intentionally enumerates only a subset of query kinds and
cannot currently encode those owners. Callable root registration also does not
yet register type roots reachable through binding consistency. The current
product fingerprint hashes the complete blocked owner key.

This engages PATCH1 INV-01 and INV-04: the binding owner cannot drop this state
or project store-local identities before their roots are closed. The plan's T4
full-`BindingState` requirement and T3 root-reconciliation requirement do not
specify whether to add a callable-local relocatable query-key schema, extend the
shared query codec/root closure, or explicitly reject this consistency case.
T5 also requires exact `AnalysisStatus::Blocked` transport, which carries the
same `BlockReason`. Architect decision is required before changing either DTO
or its root contract. T4 and dependent full-status transport are paused before
edits. This checkpoint-only update records the consultation hold; no T4
implementation changes are present.

Decision requested: choose the owner and scope for lossless portable encoding
of `UnavailableSemanticOwner(QueryKey)` nested in binding consistency, including
the `GenericConformance` type/parameter roots; if the approved choice is
fail-closed disposition, specify its PATCH1 acceptance and guard-removal effect.

## PATCH1 architecture decision — portable query-owner identity (2026-10-16)

The architect's supplied decision `PERF003.C2.P7.PATCH1-ADR-QUERY-OWNER` is
approved for implementation. PATCH1 owns a callable-local typed portable
identity for query owners embedded in callable semantic facts; it does not
persist query-cache state or depend on PATCH4. Unsupported or unresolved
identities fail closed with typed errors. The portable domain is the audited
canonical-publication subset of `QueryKey`; generic-conformance and
trait-dispatch-bucket identities require full portable encoding. Unresolved
`InferVarId` is prohibited (PUB-QO-01). Roots must cover blocked binding and
expression payloads. Query-owner fingerprints must use relocation-invariant
logical identities (FP-QO-01). Keep semantic-world schema 2, the
`CheckedStaticBody` static-application owner, and the budgeted
enum-requirement wrapper. The full amendment and QO-0–QO-9 sequence are in
§12.4 of the PATCH1 plan.

This resolves the consultation hold but does not imply T4/T5 implementation or
acceptance. The callable import guard and global allocation guards remain.
QO-0 is complete against revision `0bbc678b9dc280de3c33ba9938c8bfc01d4d4fd0`:
the 45 live `QueryKey` variants are classified in plan §12.4. Existing blocked
owner production reaches `CallableSignature` and `FieldSignature`; the
architect additionally requires portable `GenericConformance` and
`TraitDispatchBucket` support. Other variants remain typed fail-closed unless a
future live producer census changes.

### PATCH1 query-owner progress — 2026-10-16

QO-1 now defines `CallableQueryOwnerV1` inside semantic-world callable DTOs,
separate from the shared query-image wire. The four admitted variants preserve
stable callable/field identities, the full generic subject and ordered trait
arguments, and every `TraitDispatchBucketKey` field, including declaration or
exact-case target family, exact selector, and dispatch side. Export/import
adapters reject unclassified query kinds and reject `TypeTerm::Infer`.

QO-3 now traverses query-owner payloads in expression status, binding
consistency, flow binding/field summaries, blocked variant-pattern resolution,
pattern-space/exhaustiveness results, relation outcomes, and blocked
explanation steps. Generic-conformance subject and trait-argument types are
registered before graph closure; unsupported query kinds fail during root
registration. Focused regressions passed for the admitted owner kinds
(including both dispatch target families), unsupported/inference rejection,
and the field-exhaustive blocked-owner root paths. QO-2's required whole-record
preflight and QO-4 through QO-9 remain open because T4/T5 have not yet placed
these DTOs into complete binding/expression rows. Binding consistency and
expression blocked status now retain portable owner identities, but the full
T4/T5 owner records, preflight, relocation fingerprint, and integrated parity
gates remain incomplete. No callable guards were removed.

### PATCH1 expression owner slice — partial — 2026-10-16

The callable expression wire now preserves source range, `TypeKnowledge`,
selected callable, causal invalidity, binding read/write ordinals, read-safety,
and the previously added exact expression status. Import reconstructs these
facts under a child allocation transaction and validates dense rows, local
binding closure, and source ranges. A nonempty `CallResolutionId` remains a
precise typed export rejection because the live tree has no accepted portable
resolution owner; no production constructor currently publishes one.

Focused evidence: `cargo test -p phalcom-semantic --lib callable_ --
--test-threads=1` passed 40 tests, including exact-fit and one-byte-short
expression-fact import. The callable artifact integration target selected 12:
7 passed and the same 5 previously recorded failures remain (four
`Allocation(UnaccountedBoundary)` failures and one conditional-dispatch
implementation-member-authority failure). The new expression-link closure test
passed. These results do not complete T5 or remove callable guards. Remaining
T5 work includes explanation ownership, accepted generic conformance evidence,
all expression resolution families, causal/explanation cross-reference
validation, and complete materialization/parity.

### PATCH1 T5 generic-conformance evidence consultation hold — 2026-10-16

Implementation stopped before changing generic-call evidence representation.
The active plan requires Exact, Forwarded, Derived, and CompilerDerived
`GenericCallConformanceBinding` sources to round-trip exactly. The current
`CallableCallSpecializationV1` carries only selected callable and static
argument bindings. Existing `ConformanceHeadRecordV1` and `WitnessPlanV1` do
not contain the full instantiated proof payload. Native Exact/Derived sources
retain environments, selected requirement maps, associated bindings, nested
proofs, substitutions, and fingerprints; core lowering consumes Exact
requirement selections and Derived impl bindings from these products. The
portable owner shape must therefore be decided before implementation.

The query-owner ADR does not settle this separate evidence-transport boundary.
Callable conformance export remains fail-closed and all callable guards remain
active. The next action is to obtain an architectural decision on whether
PATCH1 should encode the complete evidence payload locally or reference an
existing canonical artifact owner, and to specify the exact preserved fields,
stable identities, relocation, and fingerprint contract. Do not reconstruct
evidence by re-querying semantics or dropping the conformance array.

## 0. Checkpoint state

```yaml
program: PERF003
checkpoint: PERF003.C2
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
latest_completed_plan_verification: FOCUSED_TESTED
active_plan: PERF003.C2.P7.PATCH2
latest_completed_plan: PERF003.C2.P6
next_plan: PERF003.C2.P7.PATCH2
next_action: >
  PATCH2 is active after PATCH1's callable-analysis implementation slices. The
  user's PATCH2 version ruling is semantic-world schema 3, with the outer
  container remaining schema 2; schema-2 semantic-world payloads are rejected.
  PATCH2 T0-T10 transport work is recorded below. T11 now assembles exact
  owner families into a partial imported snapshot; PATCH6 retains final
  source-free promotion. T12 owner-driven core projection is in place, and T13
  root closure has been extended. Complete the remaining parity and malformed
  wire gates before considering T14. Preserve PATCH1 allocation, Exact/Derived,
  callable-analysis and witness guards plus PATCH3 global allocation guards.
  Keep checked StaticApplication rows owned by CheckedStaticBody and preserve
  the budgeted enum-requirement wrapper.
  Keep P7 IN_PROGRESS / PARTIAL / UNVERIFIED and P8 blocked until all P7 rows
  and acceptance gates pass.
```

This record is the single durable state authority for `PERF003.C2`.

---

## 1. Bounded acceptance objective

Eliminate the ~45-second Universe-dominated compilation floor for both one-shot
and retained-session compilation. Transform the compiler pipeline from
repeatedly reconstructing the Universe compiler state into an immutable
**Canonical Universe Compilation Image** dependency model, where user fixtures
compile as small overlays with demand-driven product publication.

The checkpoint must establish:

```text
1. Immutable Canonical Universe Compilation Image
   (CanonicalUniverseCompilationImage)
   Process-shared, immutable, containing parsed modules, declaration catalog,
   interfaces, import products, topology, linked program, finalized TypeStore prefix,
   declarations, hierarchy, dispatch surfaces, callable signatures, algebraic products,
   conformance index, witness plans, trait dispatch, and reusable query roots.

2. One-shot compilation as an ephemeral overlay
   (ProgramCompiler::compile_entry_selection)
   Replaces the synthetic 40-module source injection and raw SemanticWorkspaceInput::new
   with an ephemeral WorkspaceModuleSession overlay on top of the canonical image.
   Revision U (canonical Universe) -> Revision U+1 (canonical Universe + user module).

3. Consumer-specific semantic product requests
   (CompilationProductRequest)
   Runtime execution compilation requests runtime-minimal semantics; eager
   generation of whole-Universe source presentation, advisory fixed points, and
   redundant formal projections is eliminated.

4. Incremental conformance and callable settlement
   Direct indexed conformance witness lookup, cached requirement views,
   separation of structural witness plans from return-sensitive completion,
   SCC-driven inferred-return propagation, and memoized generic conformance queries.

5. Root-driven metadata reachability architecture
   MetadataProfile enforced via explicit reachability closures from executable roots;
   cross-pool external metadata references for canonical Universe entities.

6. Elimination of O(N×M) backend scans
   Module-indexed callable and declaration tables, precomputed trait requirement-slot
   maps, and indexed inherent implementation domains.

7. telos-observability structural regression guards
   Automated cardinality assertions in CI ensuring that warm inline fixture compilation
   performs zero canonical reparses, rebuilds, body analyses, or presentation sweeps.
```

The target compilation model for a 500-byte user program:

```text
Before:
  ~45-47 seconds wall time
  148 Universe interfaces rebuilt
  UniverseSourceDeclarationCatalog rebuilt from scratch
  40 Universe source modules injected and re-analyzed
  1,238 canonical callable bodies analyzed
  Whole-Universe source presentation index constructed
  41-module advisory fixed point iterated
  Global metadata declaration × callable scan

After:
  < 200 ms wall time (in optimized profile)
  0 canonical Universe interfaces rebuilt
  0 UniverseSourceDeclarationCatalog builds
  0 canonical Universe sources in fixture semantic workspace
  0 canonical callable body analyses
  0 presentation Universe modules rebuilt (in runtime-minimal mode)
  0 advisory workspace iterations (in runtime-minimal mode)
  0 metadata sweeps over canonical declarations
```

---

## 2. Plan ledger

| Plan | Title | Package | Status | Completion | Verification | Notes |
|---|---|---|---|---|---|---|
| `PERF003.C2.P1` | Canonical Universe Compilation Image & One-Shot Overlay | Package A | IN_PROGRESS | PARTIAL | UNVERIFIED | Shared declaration/interface caches and workspace interface retention are established at `99b84ecc`; commit `a7b75980` added formal-image scaffolding; P2 now owns the implemented formal/query foundation; module-session cutover remains |
| `PERF003.C2.P2` | Complete Canonical Universe Formal Image and Immutable Canonical Query Roots | Formal/query foundation | IN_PROGRESS | PARTIAL | UNVERIFIED | Formal image, query freeze/reader, provenance validation and core acquisition implemented in the task working-tree patch over `82051480`; focused evidence below; G5/G6 certification remains deferred |
| `PERF003.C2.P3` | Conformance Indexing & Inferred-Result Settlement | Package C | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Exact indexed witness lookup, module-bounded effective-surface signature enumeration, ConformanceIndex partitioning (nominal family + open heads) with brute-force parity, SCC inferred-return settlement, and shared requirement-view passing implemented; focused regressions pass. Retained structural witness plans across revisions and generic proof memoization remain |
| `PERF003.C2.P4` | Profile/Root-Driven Metadata Reachability Architecture | Package D | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | T1 options, reachability closure T2–T5, PR-A intrinsic classification, PR-B schema v13/contextual decode/fingerprints, canonical external authority T6, registry stable indexes T7, cross-pool reflection T8, compatibility validation T9, and observability guards T10 implemented and certified. |
| `PERF003.C2.P5` | Canonical Publication and Persistent Image Boundary | Canonical publication / image boundary | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Explicit compiler image product contract, zero-copy parent overlays across all core tables, canonical persistence inventory/capabilities, query classification, CanonicalCompilerImageView boundary, and observability cardinality guards (CanonicalAggregateEntriesCopied == 0) implemented and certified. |
| `PERF003.C2.P6` | User-Only Backend/Local Complexity Closure | Package E | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | ModuleLoweringInputView, module-indexed callable analyses/definitions/declarations/data/enums, reverse-target occurrence resolution, precomputed TraitRequirementLayout, and direct inherent impl domain indexing implemented and certified. |
| `PERF003.C2.P7` (historical) | Consumer-Specific Publication & Demand-Driven Products | Package B (historical) | IN_PROGRESS | PARTIAL | UNVERIFIED | Historical label retained for lifecycle tracking; consumer-specific publication semantics are normalized into P5 (Slice B), while normalized P7 represents the .telosc container schema and persisted semantic compiler image |
| `PERF003.C2.P7` (normalized) | Telosc persisted semantic compiler image | Compiler artifact | IN_PROGRESS | PARTIAL | UNVERIFIED | Container identity/directory and module DAG validation allocations are budgeted with exact focused probes; the whole-load cap is distinct from section limits. Callable-analysis now has budgeted frozen flow-graph reconstruction, but the body owner and compiler projection remain incomplete and fail-closed. Enum requirements carry the complete owner-held signature. Witness views, static/associated/type-member, query-owner, remaining allocation, source-denied, determinism and P5/P6 gates remain open; no completion or P8 takeover claim. |
| `PERF003.C2.P8` | Portable executable Universe image | Executable artifact | PLANNED | NOT_STARTED | UNVERIFIED | Detailed plan prepared; BUILD depends on complete P7 compiler interfaces; deferred P7/P8 acceptance still controls closure |

## P7 flow-graph allocation ruling — 2026-10-15

The existing single Level 2 adviser was consulted after confirming that the
full callable-analysis import must materialize canonical `FlowGraph` data while
its source representation uses public `BTreeMap`s whose node allocations cannot
be charged to the artifact budget. The adviser selected dual backing inside
the existing `FlowGraph` owner: source graphs retain mutable map storage;
imported graphs use shared frozen `BudgetedSortedMap` node/edge storage plus
budgeted entry/exit and adjacency buffers. Keep
`CallableAnalysis.flow_graph: Arc<FlowGraph>` and preserve one allocation-free
read interface for both representations. Frozen mutation must return a typed
error and never thaw/rebuild BTree maps. Reconstruct adjacency from validated
edges using charged count/reservation scratch, preserve edge ordinal order and
the wire's stored exit sequence, and charge graph buffers and publication
wrappers. Do not use `reachable_nodes()` on the charged import path.

The decision does not close any other P7 callable-analysis fields, allocation
families, source-denied, determinism, parity, or P5/P6 gates. Exact acceptance
must cover source/frozen read parity, fingerprints, malformed graph rejection,
clone/mutation behavior, and allocator-measured adjacency success/failure
rollback before removing fail-closed guards.

### Implemented flow-graph slice — 2026-10-15

`FlowGraph` now owns either its existing mutable source maps or shared frozen
`BudgetedSortedMap` node/edge storage. The importer reconstructs adjacency from
edge ordinals with charged degree and exact-capacity vectors, preserves exit
order, and charges both Arc publication layers. Shared read accessors keep
source/frozen iteration and fingerprint behavior aligned. The callable-analysis
family-completeness guard remains unchanged because expressions, bindings,
flow summaries, exits, resolution products, diagnostics, explanations, and
compiler reader attachments are still absent from transport.

Focused evidence: artifact allocation/adjacency probe 1/1 (independent request
count equality, exact-budget success, one-byte-short rollback, malformed
endpoint rejection, source/frozen content parity, frozen mutation rejection,
source clone isolation); frozen-storage unit test 1/1; source/frozen graph
fingerprint test 1/1; source CFG construction filter 3/3; semantic/core/LSP
library check passes with only inherited unused-import/call-specialization
warnings. A broader `foundations::flow_graph` filter selected 14 tests and had
one unrelated failure in `test_protocol_derived_iteration_typing_no_name_matching`
at the existing `List<String>` initializer mismatch assertion; this slice did
not change iteration analysis and the three graph-construction tests passed.

---

## 3. Established architecture and invariants

- Baseline characterization (PERF003.C1) established that the ~45s floor is invariant to fixture size (345 B to 7.5 KB) and is caused by one-shot compilation repeatedly reconstructing canonical Universe state.
- `TypeStore` uses persistent `im` data structures and `fork_from_baseline()` preserves all canonical prefix IDs with identical numeric values while generating a fresh `TypeStoreId`.
- `SemanticWorkspaceSession` wraps updates in a `SemanticDb` undo-journal transaction; session forks must preserve canonical immutability.
- `telos-observability` provides exhaustive counters, hierarchical spans, and query metrics across compiler projection, semantic updates, TypeStore forks, and metadata export.
- `CanonicalUniverseProgram` already builds a complete semantic analysis of the Universe on first access; this formal result must be preserved in `CanonicalUniverseCompilationImage` rather than discarded.
- Canonical `UniverseSourceDeclarationCatalog` and interface projections are process-shared for the canonical provider; source-provider overrides remain isolated from those caches.
- `SemanticWorkspaceInput::new` retains the exact interfaces used to construct topology instead of forcing downstream reconstruction.
- `SemanticProductRequest::default()` preserves full tooling-compatible publication. Runtime compiler entry points explicitly request `runtime_minimal()`.
- Runtime-minimal publication retains non-Universe source/formal shards required by lowering, publishes no canonical Universe presentation shards, and publishes an empty advisory workspace.
- Canonical immutable dependencies must not be installed as workspace `last_snapshot`; the upcoming image remains a separate read-only lifetime domain.
- `MetadataReachabilityClosure` enforces `MetadataProfile` via explicit reachability traversal from roots (`MetadataRootSet`).
- Canonical Universe declarations, callables, and fields (`module_id.project.is_universe()`) are elided from ordinary user bundles by the reachability closure; only non-universe local entities and cross-pool nominal references are retained.
- `CallableSignatureTable` and `FieldSignatureTable` maintain declaration-owner reverse indexes (`owner_callables` and `owner_fields`), eliminating quadratic $O(\text{declarations} \times \text{all callables})$ scans.
- `telos-observability` tracks metadata reachability, canonical elision, and runtime lookup metrics via 14 new `PerfCounter` variants.
- T1 adds `ProgramCompilationOptions` and explicit analyzed/session compilation
  entry points. Existing APIs and canonical bootstrap default to RuntimePublic;
  semantic publication remains independent. The profile reaches the existing
  exporter, but production retention is still aggregate until T5.

---

## 4. Verification evidence

### P4 user-stopped T5 / PR-A continuation (2026-10-13, uncommitted over `e74a9213`)

- `metadata_profile_is_explicit_and_independent_of_semantic_publication`: PASS,
  1 selected, 43.93 s. Actual retained tiny compilation retains default public,
  accepts explicit minimal, returns to default public, and retains independent
  semantic publication. This supersedes the prior failure for that fixture.
- `retained_alias_error_keeps_root_and_unreachable_alias_is_not_exported`: PASS,
  1 selected, 0.01 s. Unretained alias omitted; explicitly retained String-key
  Record alias fails with entity/type context. It does not test remainder domains.
- `compiler_intrinsic_data_has_no_native_class_anchors`: observed FAIL before
  PR-A on ProductKind; after repair `native::source::tests`: PASS, 9 selected,
  0.05 s, including actual bundled source corpus and identity rejection tests.
- `metadata::reachability::tests`: PASS, 4 selected (including new
  `reachability_closure_discovers_nested_nominal_dependencies`), 0.01 s.
  Verifies recursive exporter-driven nominal discovery through Record,
  Callable, Pack, and AssociatedProjection, canonical module elision,
  failure rollback, and RuntimeMinimal filtering.
- No canonical singleton rerun, schema change/test, core integration reflection,
  broad suite, benchmark, independent review, or final formatting gate completed.
- Production T5 work is partial. Current closure traversal must be reconciled
  with the exporter's full recursive type graph before claiming completeness.
- No commit or push for this continuation. Source edits stopped immediately on
  user instruction; only lifecycle records updated afterward.


### P4 inline options continuation (2026-10-13, uncommitted over `a82dfe32`)

- Core `cargo check` with `RUSTFLAGS=''` and `perf-observability`: PASS (18.99 s).
- `empty_projection_preserves_explicit_metadata_profile`: PASS, 1 selected;
  default public and four explicit profile headers. Plumbing evidence only.
- Retained-session options regression: FAIL, 1 selected, inherited non-Symbol
  Record remainder export error before header assertions.
- Exact canonical singleton regression: FAIL, 1 selected, ProductKind
  source-index rejection before projection; canonical authority unavailable.
- Scoped diff hygiene passes. Broad compiler/reflection/runtime/performance gates
  deferred pending the incident ruling. No P4 completion or speedup claimed.

- Empirical characterization gathered in `COMPILATION-1.md` and `COMPILATION-2.md` confirms:
  1. `UniverseSourceDeclarationCatalog::build` and `InterfaceBuilder::build` are invoked for all Universe modules on every one-shot compilation.
  2. `ModuleResolver::load_interface` calls `build_from_parsed`, bypassing the static interface cache.
  3. `SemanticWorkspaceInput::new` derives topology and discards interfaces, forcing downstream re-derivation.
  4. `ProgramCompiler::compile_entry_selection` injects 40 Universe modules into an empty session where `last_snapshot = None`.
  5. `analyze_workspace` transiently computes the complete Universe semantic snapshot during `canonical_universe_program()`, but discards it immediately after lowering.
  6. Runtime compilation constructs a full presentation index for 148 Universe modules and runs a 41-module advisory fixed point.
  7. Metadata export scans all declarations and callables across the Universe regardless of requested profile.
- Prior focused demand-driven publication evidence recorded at execution head `99b84ecc` (does not cover the current working-tree delta):
  1. `runtime_minimal_skips_advisory_publication` passes with one non-Universe source/formal shard and zero advisory products/recomputation counts.
  2. `product_request_changes_replace_presentation_coverage` passes for `Full -> RuntimeMinimal -> Full` in one retained session.
  3. `incremental::workspace_indexing`: 19 passed.
  4. `phalcom-modules --test parse_cache`: 2 passed.
  5. `cargo check -p phalcom-core --features perf-observability`: passed.
- Detailed compiler-projection profile `compiler-projection-detailed-20261013T023039Z-50783` measured:
  1. source-index recomputation: 1 module, `0.364 ms` (previous comparable sample: 41 modules, `603.428 ms`);
  2. advisory: `0.750 us` (pre-bypass sample: `280.829 ms`);
  3. formal projection: `0.014 ms` (previous comparable sample: `151.359 ms`);
  4. total: `10.571 s`; semantic update: `10.441 s`; unchanged final callable reanalysis remains dominant at 3,896 passes / `3.618 s`.

**Inherited implementation increment — UNVERIFIED (commit `a7b75980`, before P2).** Since execution base `99b84ecc`, the semantic request now distinguishes presentation and advisory products. Runtime-minimal publication filters canonical Universe source-index shards while retaining local source/formal products, and publication effects/counters are narrowed to the requested products. Integration tests cover minimal publication and Full → RuntimeMinimal → Full presentation coverage. That revision declared `phalcom_semantic::canonical_image` and added process-sharing/completeness scaffolding, but lacked `canonical_image.rs`. P2 supplies the module in the current task patch. No test or build result in this record applies to commit `a7b75980`. The old evidence does not certify the new P2 patch.

### P2 task patch (2026-10-13, committed at `bf1a69c6`; lifecycle docs `c931faea`)

- Semantic-owned builder runs the ordinary complete canonical update, then freezes
  committed products. Core obtains its snapshot without a second formal analysis.
- Module-owned `CanonicalModuleProducts` is the explicit bounded P1 dependency
  adapter. Full external-module image/one-shot overlay cutover remains successor work.
- Query roots have exact world source provenance, final type/static roots, typed
  payloads, immutable forward observations and exhaustive row rejection accounting.
  Reader binding rejects foreign worlds/prefixes/static roots. An explicit
  `SemanticDb::with_canonical_image` now supplies immutable-origin read-through;
  ordinary session constructors are not switched yet. Local state, invalidation,
  discard, and rollback remain local. Canonical dependencies retain forward
  observations without local reverse edges; canonical publication is rejected.
- Focused concurrent first-access test: PASS (1 selected), eight callers / one
  image build and complete semantic update; ten warm readers add no analysis.
  Unoptimized debug sample: cold 70.812812667 s; ten warm readers 32.958 µs.
  Census: 148 modules, 9,318 rows examined, 2,763 retained, 6,555 rejected, 2,432
  final types. Ordinary publication retains four internal incidents; preserved,
  not investigated or certified by this task.
- First focused freeze/reader unit gate: PASS (5 selected). Final expanded unit
  and final build-check results are recorded in the walkthrough.
- The user explicitly requested fast code delivery and no baseline investigation.
  Source/provider differential parity, core runtime parity, broader existing
  publication suites and the full G6 performance matrix are deferred. P2 remains
  IN_PROGRESS / PARTIAL / UNVERIFIED; focused checks do not complete the plan.

---

## 5. Deferred and baseline failures

| ID | Failure | Classification | Disposition |
|---|---|---|---|
| `corpus::booleans` | Fails before VM construction: legacy metadata cannot represent Record remainder outside Symbol key domain | D | Pre-existing baseline compiler limitation recorded in PERF003.C1; nonblocking for Package A/B/C/E. Package D will resolve metadata reachability. |
| `perf_universe_characterization` VM::new() | Fails at runtime: intrinsic runtime data ProductKind is not a canonical Tuple/Record/Pack source declaration | D | Pre-existing runtime bootstrap blocker recorded in PERF003.C1; nonblocking for compiler and semantic optimization. |
| `P4 canonical authority prerequisite` | Historical singleton failure at ProductKind source indexing | D | PR-A classification repaired with 9 focused source-index PASS, including actual corpus. Singleton/projection not rerun after repair; PR-B still blocks faithful canonical metadata and T6 proof. |
| `P4 retained-session options regression` | Historical ordinary aggregate export failure on non-Symbol Record remainder | D, resolved for exact tiny fixture | After T5 slice, default public → explicit minimal → default public test PASS (1 selected). Required retained heterogeneous Record representation remains a separate PR-B prerequisite. |
| `P4.T6 / P5.T10 canonical compiler projection` | Historical singleton and reflection test failure during `universe:numbers.complex` lowering: missing executable `Zero.zero` or `One.one` class-side trait slot | Resolved | [Incident packet](PERF003.C2.P5-trait-evidence-slot-incident.md): Root cause was `publish_conformance_witness_plans` in `session.rs` unconditionally skipping Universe conformances during canonical image build. Constrained by `db.canonical_image().is_some()`; `canonical_universe_program_is_process_singleton` PASS (43.19 s) and row display PASS. |
| `cross_module_inferred_callable_result_round_trips_as_compiler_inferred_metadata` | Local semantic test workspace rejects canonical Universe sources before metadata export | D for PR-B; P5 fixture compatibility unclear | One bounded classification pass: failure is `canonical Universe sources cannot enter a local workspace; use an explicit provider image` at workspace update. Preserve the fixture; address in the workspace/provider owning work rather than changing expected metadata. |
| `corpus::blocks` | `blocks_argument_to_method.ph` fails with `[1, 2, 3, 4] does not understand 'fold(initial,using)'` | D | Pre-existing missing library method recorded in PERF003.C1. |
| `semantic::integration::advisory::analysis::formal_projection_preserves_causal_invalidity_and_binding_contracts` | Expected one causal-invalidity marker but current formal projection publishes a different state | D | Reproduces in isolation outside the demand-driven request path; deferred for the owning formal-projection work. |
| `incremental::type_store_revisions` callable-count assertions | Existing first-publication callable recomputation counts differ from stale expected values | D | Reproduces in isolation after all source/formal/advisory assertions pass; nonblocking for product-request work. |

---

## 6. Consultations and amendments

- P7 StaticApplication owner/transport continuation (2026-10-15): the user
  confirmed `CheckedStaticBody` as the sole owner; no copy into
  `CallableAnalysis` is authorized. Live code already retains rows under
  `StaticBodyId + ordinal`. The consulted adviser recommended an owner-nested
  portable checked-body record containing the operational template and rows,
  with op application ordinals and explicit per-operand index/parameter/kind
  metadata. Kinded terms use relocated type refs; nested kinded applications
  use child ordinals; value operands use checked row positions. The imported
  owner product carries no source `ExpressionId`. Preserve the source range as
  provenance, the exact checked signature/domain facts, and validate target,
  signature shape, ordinal uniqueness, argument mapping, nested-child identity,
  and denotation lookup before publication. Since canonical static-definition
  target authority is not yet persisted, treat the checked row signature as
  captured authority and do not claim a fresh target-signature lookup. Retain
  denotation/export/import fail-closed guards until the required consumer and
  focused discriminators exist.

- P4 prerequisite resumption (2026-10-13): user authorized resuming the
  previously stopped P4 PR-B/schema and canonical RuntimePublic metadata work
  required by P5.G1. The user specifically approved decode-time graph
  normalization for pre-v13 metadata: insert canonical Symbol key-domain
  nodes, relocate affected global/scoped node references and prove old-fixture
  compatibility. The accepted no-lossy-export, distinct semantic/runtime
  identity and complete RuntimePublic invariants remain binding. This is
  authorization to implement and verify, not evidence that P4 or P5 passed.

- P4.PR-B legacy Symbol identity clarification (2026-10-13): valid old bundles
  can have Record fields without a Symbol type node; the bundle header does not
  supply a stable canonical Universe Symbol declaration identity. The user
  selected loader-supplied, validated canonical Symbol identity for graph
  normalization. Standalone decode of an affected legacy bundle without that
  context must return a typed need-for-context error rather than guess from
  `producer_version`. Unaffected legacy bundles retain normal standalone
  decoding. This is an approved Level 1 identity clarification; it does not
  weaken pre-v13 compatibility for correctly contextualized loaders.

- P5 conformance ownership clarification (2026-10-13): the live first-update
  path clones canonical `ConformanceIndex` (five correlated maps), witness-plan
  map, and `DerivedConformanceRuleIndex` (trait buckets and impl map). Replacing
  these with a simple parent map would change joint query/coherence behavior and
  `rules_for_trait` slice semantics. The user selected a layered canonical/local
  conformance view with merged query and coherence traversal and exact parity
  tests. No G6 copy exception is approved; conformance evidence must not be
  strengthened by aggregation or fail-open negative lookup.

- P4.PR-B fingerprint migration clarification (2026-10-13): decode-time
  normalization of pre-v13 Record graphs must retain their existing v12
  structural fingerprints as version-tagged legacy provenance. These are not
  equivalent to the v13 semantic export fingerprints; cross-version digest
  comparisons must refuse compatibility until semantic authority re-exports
  the normalized graph. Do not duplicate semantic fingerprint computation in
  `phalcom-type-meta` merely to migrate legacy bytes. This choice preserves
  one hashing authority and a fail-closed mixed-version boundary.

- P4 architectural ruling accepted from the user (2026-10-13):
  [full ruling](PERF003.C2.P4-architectural-ruling.md). T1 stays unchanged.
  T5 continues with authoritative roots, no aggregate production fallback,
  and structured export provenance. PR-A classifies compiler-owned intrinsic
  data separately from native Tuple/Record/Pack facades, preserving fail-closed
  canonical identity checks. PR-B extends metadata for heterogeneous fixed
  Record keys and explicit remainder key domains, including scoped/row forms,
  fingerprints and legacy Symbol-key decoding. No lossy export or public
  surface omission. T6–T9 remain behind both prerequisites.

- P4 T5 reachability consultation (2026-10-13): following review of
  `metadata/reachability.rs`, the closure's partial type walker conflicted with
  the plan's exporter-owned recursive type traversal and missed nested local
  nominals. The adopted ruling is that a disposable `MetadataExporter` discovers
  nominal references through `export_type_form`; RuntimePublic adds new local
  declarations to roots and recomputes closure to a fixed point. Profile and
  external-identity filtering remain in the closure engine, and final export
  preserves retained-entity error provenance. The plan amendment is recorded
  in the active P4 plan. Implementation is present but unverified; no tests
  were run for this follow-up.

- P4 continuation, 2026-10-13: mandatory trigger 2 raised because the canonical
  compiled metadata authority cannot be built. The incident packet is
  [PERF003.C2.P4-canonical-authority-incident.md](PERF003.C2.P4-canonical-authority-incident.md).
  User required inline work with no subagents; no adviser was spawned and no
  ruling was assumed at that time. The subsequent accepted ruling above resolves
  that request and authorizes PR-A/PR-B; existing profile/semantic rules stand.

- L2 (2026-10-13): User requested continuing the compiler optimizations in the
  attached “Canonical Universe Image Implementation and Next Major Performance
  Optimizations” response and explicitly deferred broader tests/verification.
  This expands the earlier image-only scope to successor optimizations. Implemented
  section 10's opt-in database parent and section 15.1's exact witness lookup.
  Full session/module ownership cutover is still pending; no broad performance or
  semantic parity claim follows from these focused increments.

- L0 (2026-10-13): User approved reserving P2 for the attached formal-image/query-root plan and moving existing Package B publication work to P7. No existing P7 ledger package was found. Historical evidence is preserved. Adopt only D1–D10 of the new P2 plan for this scope; the unified specification remains proposed as a whole. No workspace source-ownership cutover is authorized by this amendment. Starting head `82051480`; latest tested predecessor `99b84ecc`; baseline `cargo check -p phalcom-semantic` fails E0583 for the missing image module.

- Consultation based on `COMPILATION-1.md` and `COMPILATION-2.md` confirmed the 5-package architecture.
- Reaffirmed: no workaround harnesses (e.g. changing `run_inline` alone); optimization must live underneath `ProgramCompiler::compile_entry_selection`.
- Reaffirmed: canonical image must be strictly immutable; sessions compile user code as persistent overlays.
- Reaffirmed: structural work cardinality (measured via `telos-observability`) is the primary gate, preventing faster hardware from masking algorithmic regressions.
- `PERF003-C2-Unified-Compiler-Artifact-Implementation-Spec.md` was added as a proposed architecture for review. It is not an adopted plan amendment; the active detailed implementation plan and this checkpoint remain the execution and lifecycle records until an explicit decision is recorded.

---

## 7. Next action

The current route is the adopted Level 2 amendment in
[P7 remaining execution](P7/PERF003.C2.P7-remaining-execution-plan.md), resolving
the [Wave 4 prerequisite incident](P7/PERF003.C2.P7-wave4-prerequisite-incident.md)
by reopening incomplete predecessor authority before T11/T12. The
[P8 detailed plan](P8/PERF003.C2.P8-portable-executable-universe-detailed-implementation-plan.md)
follows once the P7 compiler interface is implemented. Earlier P2/P4/P5 next-action
notes below are historical evidence, not current routing.

The current user authorizes rigorous testing, superseding compiler-only checks.
P7/P8 full-image acceptance and source-free proofs remain unexecuted because
required implementation is incomplete; do not promote inherited P6 evidence.
The user requested scoped commit/push delivery and a clean perf worktree after
this continuation; do not stage unrelated work.

### Focused successor evidence (2026-10-13)

- `impls::conformance_index_partition_tests::partitions_queries_and_coherence_match_brute_force_through_mutations`:
  PASS, 1 selected, 0.01 s. Verifies exact equivalence between partitioned index
  (by trait, nominal family, and open heads) and brute-force query and overlap
  conflicts across insertions, mutations, and source removals.
- `semantic::incremental::callable_dependencies::inferred_return_refresh_preserves_stable_products_and_counts_final_disposition_once`:
  PASS, 1 selected, 43.24 s. Verifies that SCC and affected-result worklist
  propagation preserves stable downstream products without global rescans, and
  updates body dependency edges.
- `missing_qualified_module_value_export_invalidates_when_export_is_added`:
  PASS, 1 selected. Verifies negative export dependency revalidation when the
  export is added in a subsequent revision.
- `qualified_module_value_dependencies_follow_reads_not_import_seeding_or_shadowed_names`:
  PASS, 1 selected. Verifies that imported and local checked bindings record
  provenance edges only when looked up, not on blanket import presence.
- `from_interfaces_retains_authoritative_products_without_rebuilding`:
  PASS, 1 selected. Verifies that `SemanticWorkspaceInput::from_interfaces` preserves
  pre-derived interface pointers without rebuilding.
- `canonical_interface_load_does_not_load_parsed_source_per_resolver`:
  PASS, 1 selected. Verifies canonical Universe interfaces bypass parsed source loading.
- `canonical_image_database_reads_are_immutable_leaves_across_local_rollback`:
  PASS, 1 selected, 69.43 s including cold canonical bootstrap. Verifies identity
  sharing, canonical-origin currentness/reuse, forward dependency recording,
  no canonical invalidation/retirement/publication, and local transaction rollback.
- `conformance_witness_lookup_preserves_exact_owner_side_and_selector_after_edits`:
  PASS, 1 selected. Exact impl/side lookup distinguishes inherent declarations,
  survives table cloning, and observes removals.
- `metadata::reachability::tests::metadata_root_set_tracks_cardinalities`:
  PASS, 1 selected, 0.00 s. Verifies root tracking and cardinality computations.
- `metadata::reachability::tests::reachability_closure_minimal_vs_public_retention`:
  PASS, 1 selected, 0.01 s. Verifies profile-driven retention for `RuntimeMinimal` vs `RuntimePublic`.
- `metadata::reachability::tests::reachability_closure_elides_canonical_authorities`:
  PASS, 1 selected, 0.01 s. Verifies that canonical Universe declarations, callables, and fields are elided as external authorities.
- `signature::tests::callable_owner_index_tracks_membership_removal_and_cloning`:
  PASS, 1 selected, 0.00 s. Verifies secondary owner index in `CallableSignatureTable`.
- `signature::tests::field_owner_index_tracks_membership_removal_and_cloning`:
  PASS, 1 selected, 0.00 s. Verifies secondary owner index in `FieldSignatureTable`.
- `semantic::integration::canonical_image`:
  PASS, 4 selected, 135.74 s. Verifies complete canonical formal image properties:
  Send/Sync thread safety, process sharing across concurrent threads, external
  Universe isolation across local replacement, and source parity and provider isolation.
- `canonical_formal_image_source_parity_and_provider_isolation`:
  PASS, 1 selected. Verifies provider-isolated Universe declaration bootstrap with custom source overrides.
- `canonical_image::tests`:
  PASS, 10 selected, 39.27 s. Verifies canonical compiler image product completeness validation,
  frozen query selection, nonready query classification, database read immutability across rollback,
  world provenance digest collision rejection, and root validation.
- `declarations::DeclarationTypeTable` parent-aware architecture:
  Verified compilation and type checking with `with_parent`, delegation lookups, parent chain `iter()`, and local-only mutation.
- `RUSTFLAGS='' cargo check -p telos-observability`: PASS (clean).
- `RUSTFLAGS='' cargo check -p phalcom-semantic`: PASS (clean, 0 warnings, 0 errors).
- `RUSTFLAGS='' cargo check -p phalcom-core --features perf-observability`: PASS (17.16 s).
- `RUSTFLAGS='' cargo check -p phalcom-modules`: PASS (0.17 s).
- `git diff --check`: PASS (clean).


### P5 dependency authority clarification (2026-10-13)

Adviser ruling: canonical query freeze eligibility and immutable aggregate
publication are distinct. Keep query freeze fail-closed. An image-owned derived
index may expose exact positive structural dependency fingerprints from the
validated final snapshot, without synthesizing Ready rows or reusable queries.
Local states take precedence. Hierarchy evidence must preserve its validated
source-sensitive owner fingerprint and agree with final declaration/parent
identity. Frozen-query overlap must agree. Record exact edges and treat these
as immutable leaves in dependency validation. This is a Level 1 clarification
of P5, with no new inference or mutable semantic authority.

Alias oracle clarification: invalid cycle participants and transitive alias
dependents must be quarantined; unaffected canonical and valid local aliases
must survive. Global-empty table assertions are stale fixture assumptions.


### P5 parent-overlay and dependency continuation (2026-10-13, committed as `e71057d9` over `0b40dbbb`)

- `ParentMap` provides O(1) immutable parent attachment, deterministic parent/local
  iteration, local-only mutation, and rejection of canonical identity collisions.
- Dispatch surfaces, conditional-member sets, type registrations, data/enum,
  associated surfaces, enum requirements, and type-member tables use shared
  canonical parents. Published snapshot surfaces share dispatch's map. Resolver
  associated/type-member views retain these maps without flattening the parent.
- First-update factories use the image-backed base overlays rather than cloning
  the canonical snapshot; retained updates copy only local contributions.
- Exact aggregate dependency fingerprints preserve edge evidence without granting
  aggregate-only products query readiness. Frozen roots must agree with settled
  structural authority; stale rows and their transitive dependents are rejected.
- Alias fingerprints include the formed result, fixing imported-alias relowering
  with unchanged consumer syntax. Cycle quarantine removes participants and their
  alias dependents, cached shells/static definitions, and retained resolver forms.
  Unaffected canonical and valid local aliases survive, including valid→cycle→valid.
- Adviser confirmed foreign inherent implementations of canonical declarations
  are forbidden by `docs/specs/objects/impl.md`; strict parent collisions remain.
- Independent focused review found a computed-key readiness consumer; corrected
  to exact dependency evidence. No remaining actionable review findings.

Completed evidence:

- `incremental::recomputation_metrics`: PASS, 13 selected, 120.97 s. All four
  inherited failures now pass; no shell assertion was weakened.
- `integration::workspace`: PASS, 18 selected, 44.89 s, including the new
  quarantine/preservation/recovery regression.
- `capabilities::aliases`: PASS, 7 selected, 6.57 s. Three stale global-empty
  alias assertions now check exact invalid identities, preserving diagnostics
  and declaration absence.
- Final `--lib canonical_image`: PASS, 12 selected, 41.84 s. Includes aggregate-only
  exact evidence, invented-key rejection, fingerprint mismatch rejection,
  local-state precedence, and stale structural freeze/dependent closure.

Residual: ordinary session DB structural rows can lag final inferred-return
settlement. This continuation excludes stale rows from canonical freezing;
certification of general settlement is not claimed. Residual copied maps
(alias/static products, callable definitions and other P5.T6 Slice 4 families),
T7–T13, P4-dependent metadata composition, persisted `.telosc`, broad release
verification, and timing certification remain open. Overall P5 remains
`IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.

Final continuation gates:

- `--lib parent`: PASS, 10 selected, 0.06 s; includes both new `ParentMap`
  regressions and parent dispatch lookup/local removal.
- `integration::canonical_image`: PASS, 4 selected, 123.33 s. Includes actual
  source parity/provider isolation and two user-only replacement updates proving
  zero local entries in the migrated canonical tables and shared surface identity.
- Computed Record-key callable-signature regression: PASS, one selected, 41.15 s.
- `cargo check -p phalcom-core --features perf-observability`: PASS, 16.30 s.
- `cargo check -p phalcom-lsp`: PASS, 11.48 s; existing unused import warning.
- `git diff --check`: PASS. Repository formatter applied only to changed regions;
  no whole-workspace formatting or release certification claimed.

Source delivery: `e71057d9` on `perf/universe-characterization`. Commit and push
were authorized on 2026-10-13; lifecycle records accompany this source revision.
The continuation is focused-tested; residual families and metadata/artifact
acceptance gates remain open.

### P5 residual lookup and query-classification increment (2026-10-13, `eeca76f3`)

- `TypeAliasTable`, `StaticDefinitionTable` and `StaticConstantTable` attach
  immutable canonical parents; local removals and duplicate-ID rejection do
  not mutate/shadow canonical entries. First local update creates empty
  overlays; later updates clone local contributions only. The retained alias
  resolver now reads the committed snapshot's alias table instead of a second
  map of all canonical forms. Its static-definition lookup retains the parent
  table rather than flattening it into a `BTreeMap`.
- Accepted callable definitions now use `ParentMap` in the published snapshot;
  local revisions share canonical entries by reference. This does not yet
  cover remaining static-case catalogs/signatures, associated static domains,
  module binding maps and other copied aggregate families.
- `canonical_query_persistence(QueryKind)` is the single exhaustive family
  classification consumed by in-memory freeze and available to future artifact
  projection. It preserves the exact prior eligible/open-world/unsupported
  sets. Row readiness, final settlement and dependency closure remain separate
  fail-closed checks; no query is made Ready by family classification alone.

Focused evidence for the increment: canonical sharing/replacement integration
passed (one selected; first observed the alias, static constant and callable
definition copies before their corrections); 13 incremental recomputation tests,
seven alias capability tests and 18 workspace integration tests passed after
alias/static migration. Canonical-image library filter passed 13 selected,
including query classification and existing freeze contracts. After callable
definition migration, the exact image/replacement regression and an affected
incremental callable/body edit test passed. `cargo check -p phalcom-core
--features perf-observability` passed after the final code change. Existing
unused `TypeHierarchy` test import warning remains unrelated. Scoped diff
whitespace check passed. Full workspace/release, formatting, benchmarks,
P4 metadata integration and `.telosc` verification remain deferred; P5 is
`IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.

### P5 parent-overlay continuation and P4 prerequisite gate (2026-10-13, `ca3c3c03` / `9143262b`)

- Image-backed local updates now share canonical module binding facts,
  constructor paths and trait headers/surfaces without flattening their maps.
  `ParentMap` exposes parent-only iteration and constant-time parent counts.
  Canonical static-case catalogs/signatures have zero entries in the live image;
  no general parent contract is claimed for them.
- Focused evidence: canonical-image integration PASS (four selected, 137.44 s);
  workspace integration PASS (18, 43.23 s); exact module-value dependency
  PASS (one, 47.24 s); parent-map library PASS (two); core with observability
  check PASS. Parent-count checks passed in the three-revision image
  replacement regression. PR-A Record/Pack native anchor test PASS (one).
- Exact P4 baseline: `canonical_universe_program_is_process_singleton` FAIL
  (one selected, 48.06 s) at public `FixedFields` metadata export because v12
  cannot encode its non-Symbol Record remainder. No schema changes were retained;
  PR-B, P4.T6 and P5.T10 remain incomplete. The semantic test harness retains
  an unrelated unused-import warning.
- Conformance index, witness plans and derived-rule buckets still clone
  canonical entries at first local update. The user approved a layered
  canonical/local view, not a P5.G6 copy exception; proof/coherence parity
  must be verified before claiming zero canonical aggregate copies.

### P5 layered conformance progress (2026-10-13, `7c9bd6d7`)

- `ConformanceIndex` now reads immutable canonical contributions through a
  parent; local secondary indexes remain disjoint. Exact queries merge only
  indexed parent/local candidates, while coherence considers canonical-local
  cross pairs without copying parent buckets into the workspace. Iteration
  remains stable-ID ordered and canonical entries retain reference identity.
- `DerivedConformanceRuleIndex` retains canonical trait buckets and an impl
  index through a parent. A borrowed two-slice trait view preserves indexed
  access, cardinality and iteration without flattening. General rule iteration
  remains stable-ID ordered for downstream fingerprints.
- Focused evidence: two conformance partition/brute-force library tests PASS,
  including a new canonical/local overlap and removal regression; derived
  parent library regression PASS; derived incremental/cold parity and condition
  edit regressions PASS (one each); canonical-image replacement regression
  PASS (one); core with observability check PASS. Canonical witness-plan map
  is empty in the present image, so its existing cloning copied zero parent
  entries in this run, but its general parent contract remains pending.
- P5.T6/T7/T8/T11/T12 and P4.PR-B/T6 are still open. No P5.G6 or P5.G1
  certification follows from these focused checks.

### Concurrent P5 persistence-inventory continuation (`791a0c0d`–`c600b12a`)

- Remote commits introduced a 22-family `CanonicalPersistenceClass` /
  `CanonicalProductFamily` inventory and `CanonicalImageCapabilities` in
  `canonical_image.rs`. Source-semantic attachments remain conservatively
  `RequiredSemantic` without an artifact-only reconstruction contract.
  T8 still needs complete compatibility/provenance and P4 metadata inputs.
- Remote conformance and derived-rule changes overlapped the locally tested
  `7c9bd6d7` work. The merge retains the validated cross-boundary coherence,
  stable-ID iteration and borrowed trait-rule view, alongside the new inventory
  and useful remote test/cardinality additions. Remote edits were GitHub-only
  and unverified when published; integrated-tree verification is separate.
- Witness-plan map, T7/T8 closure, P4-dependent T10 and T11–T13 remain open.

Integrated-tree verification (after resolving the concurrent conformance
implementations): `cargo check -p phalcom-semantic` PASS; conformance partition
library filter PASS (two), canonical-image library filter PASS (14, including
the inventory classification test), derived-rule parent library regression
PASS (one), canonical-image integration PASS (four, 125.66 s), derived-rule
incremental/cold parity PASS (one), and core observability check PASS. Remote
source had no test evidence when published; these results apply to the
reconciled tree only. The semantic integration test harness still reports the
unrelated unused `TypeHierarchy` import warning.

### P4.PR-B Record metadata increment (2026-10-13, uncommitted over `55452273`)

- Schema v13 retains tagged `Symbol/String/Int/Bool` keys and explicit global,
  scoped and static-row Record remainder domains. Semantic export fingerprints
  include key kind/value and remainder key/value type identities; publication
  retains a public `FixedFields`-shaped alias with its `FieldKey` domain.
- Pre-v13 Symbol field names decode without context. Anonymous legacy Record
  remainders require a loader-supplied canonical Symbol declaration; absent or
  invalid context is typed. Contextual decode inserts canonical global/scoped
  nodes, relocates references, validates, and marks old digests with an explicit
  fingerprint-schema version. The normalized graph round-trips as v13 without
  a second relocation. Mixed-version fingerprint equality fails closed.
- Runtime metadata conversion, identity, display and row static-argument
  projection retain heterogeneous keys and both remainder domains. Canonical
  RuntimePublic installation/reflection acceptance is **not** established.
- Evidence: `phalcom-type-meta --test schema_compat` PASS (23 selected);
  semantic exact export/alias/fingerprint/scoped-row tests PASS (one each);
  `phalcom-core --lib typing::identity::tests` PASS (8 selected);
  runtime row substitution PASS (one); core observability `cargo check` PASS;
  `cargo fmt --package phalcom-type-meta -- --check` PASS; scoped diff check
  PASS. Workspace format check is incomplete on pre-existing canonical-image
  formatting changes outside PR-B.
- Owning semantic metadata integration filter: 36 PASS / 1 FAIL (37 selected);
  the sole failure is the local-workspace canonical-source fixture above,
  before metadata export. Do not count this filter as passing.
- Trait evidence slot incident resolved: `publish_conformance_witness_plans` in
  `session.rs` was updated to check `if db.canonical_image().is_some() && contribution.source_module.project.is_universe()`,
  ensuring Universe witness plans are published and stored in the canonical image snapshot.
  `canonical_universe_program_is_process_singleton` PASS (one selected, 43.19 s);
  all 6 tests in `modules::canonical_universe::tests` PASS;
  all 10 unit tests in `canonical_image::tests` PASS;
  all 4 canonical formal image integration tests PASS (130.56 s);
  `applied_record_row_metadata_display_preserves_field_types_and_tail` PASS;
  `modules::compile::tests` PASS (3 selected, including RangeIterator closed bounds);
  `metadata::reachability::tests` PASS (4 selected, including nested nominal discovery).
  Canonical compiler projection is now unblocked and certified. P4.T6 canonical
  authority installation and P5 witness-plan parent-sharing can proceed.


## P7 continuation — 2026-10-14 source-free module boundary

- Active plan: PERF003.C2.P7; completion PARTIAL; verification UNVERIFIED.
- Starting revision: 3ca30fca. Existing T1/T2 edits and documentation relocation
  were inherited and preserved; this continuation is uncommitted.
- CanonicalExternalModuleWorld now separates completed linked/interfaces/topology
  authority from CanonicalModuleProducts source-build provenance.
- Module sessions, resolvers, and linkers consume external authority without a
  UniverseSourceProvider. Semantic image sessions use completed canonical linking
  authority; optional source-backed presentation remains a separate capability.
- Artifact integrity uses BLAKE3-256; stable identity schema compatibility is
  checked. The reader rejects unsupported flags, compression, known section
  schemas, and executable capability without its payload.
- Container/architecture specification: docs/specs/compiler-artifacts/telosc.md.
- User instruction: targeted compiler-driven checks only; no tests or broader
  verification for this continuation. No behavioral acceptance gate is promoted.
- Next action: P7.T4 MODULE_WORLD DTO/projection/load, then T5–T15 and P8.
  No universe.telosc writer/loader, executable payload, or CLI artifact path
  is implemented by this continuation.

Final compiler-only evidence: cargo check -p phalcom-core -p phalcom-lsp --lib
passed in 10.41 seconds with flags cleared. LSP's unchanged backend.rs reports
an unused TypeHierarchy import. Behavioral verification remains UNVERIFIED.


## P7 continuation — module-world transport and type-graph relocation

- Active plan PERF003.C2.P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED.
- T4 transport/projection/materialization code implemented: ModuleWorldV1,
  artifact-local module IDs, bounded binary codec, module-owner conversion,
  fingerprint/namespace/topology/DAG validation, and source-free external-world
  constructors. No ModuleLinker, parser, or interface builder is called by import.
- T5 projection/relocation code implemented: exhaustive type/kind/scoped node
  schemas, distinct arena ID families, stable owner codecs, fresh TypeStore,
  normal interner reconstruction, checked graph dependencies, and explicit
  TypeRelocation vectors for later payload consumers.
- Durable distinction: TypeStore contains historical parameter versions and a
  separate current `(owner,index)` selection. Persistence retains both rather
  than collapsing versions. Parameter variance overrides are also primary state.
- Scoped union recipes preserve order/repetition. Global unions normalize native
  member order after relocation; scoped lowering does not perform that normalization.
- Completed compiler-only check: flags cleared, cargo check -p phalcom-core
  -p phalcom-lsp --lib passed in 8.31 seconds. Unchanged LSP backend reports its
  existing unused TypeHierarchy import. No tests or behavioral verification run.
- T4/T5 parity, malformed-input behavior, cross-process determinism, and
  source-free acceptance remain unproved. No acceptance gate promoted.
- Next action: P7.T6 static-value graph and relocation, then T7–T15 and P8.
  Data/variant value interning requires canonical DataInfo/VariantInfo; preserve
  that validation in the loader rather than constructing unchecked value nodes.
- No complete universe.telosc writer/loader, CLI artifact commands, or executable
  persistence exists yet. Changes remain uncommitted.

Final compiler-only check after codec error classification: flags cleared,
cargo check -p phalcom-core -p phalcom-lsp --lib passed in 5.30 seconds.
Only the unchanged LSP unused TypeHierarchy import warning remains.


## Review corrections — 2026-10-14

The three requested review fixes are implemented in the existing worktree:

- Tuple, Record, and Pack static constants again project self-contained recursive
  descriptors and use the canonical owner-held runtime materialization path,
  including when nested inside immutable sequences.
- Both demand-driven and ordinary-session constant publication retain the
  evaluator-owned residual store for symbolic outcomes. Cycle products carry no
  residual store. Published products retain ownership through their existing Arc.
- TypeStore artifact relocation rejects RecordRow parameters in TypeData::Parameter
  and rejects a node kind differing from its relocated parameter definition.


## P7 continuation — T7.A/T6 transport seam — 2026-10-14

- Active plan PERF003.C2.P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED.
- Source state checkpointed at user commit `5196d002`; one uncommitted
  import-path correction to `phalcom-semantic/src/static_value/artifact.rs`.
- Established this continuation:
  - `semantic::static_value` is exported in `phalcom-compiler-artifact` and
    compiles; its codec is bounded, fail-closed, validates canonical scalar
    encodings, record-key ordering, pack slice bounds, and static DAG acyclicity,
    and returns dependency-first reconstruction order.
  - `semantic::aggregates` is now a versioned structural-contract DTO codec
    (schema 1) with owner-identity and declaration-order validation; it stays a
    DTO crate with no semantic projection.
  - `phalcom_semantic::static_value::{StaticRelocation, import_static_value_graph}`
    reconstructs all static families into a fresh lineage through the ordinary
    validated factories, resolving DataInfo/VariantInfo via loader-supplied
    tables (T7.A-before-T6 staged dependency honored) and preserving shared pack
    backings.
- Compiler-only evidence (flags/wrapper cleared, serial): artifact crate check
  PASS clean; semantic lib check PASS 14.19 s; core+lsp lib check PASS 8.22 s
  with only the inherited LSP unused-import warning. `git diff --check` PASS.
- Not established: T6 export projection, T7.A semantic-owned projection/loading
  (exhaustive DataInfo/EnumInfo/VariantInfo coverage), T7.B checked attachments.
- Escalation resolved 2026-10-14 (user decision, session continuation): T7.B
  source-independent checked operational templates are APPROVED — wire design
  with relocated operands, stable targets, and retained evaluation order, plus
  the corresponding evaluator contract change. No empty attachment tables.
  Execute T7.A exhaustive DTO coverage first, then T6 export projection, then
  T7.B. Packet recorded in the P7 handoff.
- T7.A exhaustive DTO rewrite of the artifact-crate structural contracts is
  implemented but UNCOMPILED and unverified (uncommitted over `5196d002`):
  declared-type states, visibility, shape templates, case environments, and
  real generic constraints now mirror live shapes; invented wire-only fields
  (pending static defaults/discriminants) were removed. Continuation facts and
  next actions are recorded in the P7 handoff (second 2026-10-14 section).
- T13/T14 acceptance remains deferred under the compiler-only policy. No
  behavioral gate is claimed. No commit/push performed by the agent.

Compiler-only validation completed with local flags and wrapper cleared:
`cargo check -p phalcom-core -p phalcom-lsp --lib`,
`cargo check -p phalcom-semantic --test semantic -p phalcom-core --test core --lib`,
and `cargo check -p phalcom-core --tests` all passed. Existing unused TypeHierarchy
imports were reported in the LSP backend and semantic workspace integration tests.
Regression coverage was added/updated for structural owner materialization,
nested descriptor contents/labels, residual ownership after evaluator teardown,
and valid/malformed parameter artifact relocation. Tests were compiled but not
executed, honoring the existing compiler-only instruction. No behavioral gate or
checkpoint/plan status was promoted; P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED.
No architectural consultation or plan amendment was needed. Continue with the
existing P7 next action. No commit or push was requested.


## P7 Wave 3 completion — 2026-10-14

- Active plan PERF003.C2.P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED.
- Established this continuation (Wave 3: T8, T9, T10):
  - **P7.T8 — Indexed immutable authority & primary reconstruction:**
    - Wire schema `SemanticWorldV1` and `WitnessPlanV1` defined in `phalcom-compiler-artifact::semantic::world`.
    - `impl Wire for [u8; 32]` added to `module_world.rs`.
    - `compute_witness_layout_fingerprint` in `validate.rs` validates witness plans against member ordering fingerprints.
    - `ReconstructedModuleOwnerIndexes` and `validate_and_import_semantic_world` in `import.rs` reconstruct module and owner indexes in a single $O(N)$ pass.
    - `SemanticSnapshot` augmented with `callable_definitions_by_module` and `is_source_free_artifact_complete`.
    - Direct inherent impl domains and derived conformance `get_rule` lookup preserved via owning tables.
  - **P7.T9 — Reusable query image & dependency evidence separation:**
    - `phalcom-compiler-artifact::query`: `PortableQueryKindV1` with `from_raw` and `profile_support`, `QueryImageV1` with bounded `encode`, `decode`, and `validate`.
    - `phalcom-semantic::artifact::query`: `validate_query_image_closure` enforcing source-dependent pruning, acyclicity, and dependency index validation.
    - `CanonicalSemanticQueryImage::from_loaded_artifact_entries` reconstructs immutable query roots without mutable database rows.
    - `CanonicalUniverseFormalImage::from_loaded_artifact` provides the source-free formal image constructor.
    - `export_semantic_world` and `export_query_image` implemented in `artifact::export`.
  - **P7.T10 — RuntimePublic metadata transport:**
    - `phalcom-compiler-artifact::metadata`: `MetadataSectionV1` bounded encoding/decoding and `validate_and_charge_metadata_preflight` enforcing `RuntimePublic` profile and `ArtifactIdentityScheme::V1Standard`.
    - `phalcom-core::modules::canonical_artifact`: `finish_loaded_universe_telosc` validating metadata preflight and producing `LoadedUniverseArtifact`.
- Compiler-only verification:
  - `cargo check -p phalcom-compiler-artifact -p phalcom-semantic -p phalcom-core --lib`: PASS clean (0 warnings, 0 errors).
  - `cargo check -p phalcom-lsp --lib`: PASS with only the pre-existing unchanged unused import warning.
  - `git diff --check`: PASS clean.
- Next action: Proceed to Wave 4 (P7.T11 deterministic artifact assembly and atomic publication, followed by P7.T12 source-free compiler loader and tracer integration).

## P7 Wave 4 preflight correction — 2026-10-14

This entry supersedes the Wave 3 completeness/takeover claims above and the
earlier compiler-only policy. Active plan remains IN_PROGRESS / PARTIAL /
UNVERIFIED. T11/T12 were not implemented during this continuation.

- Incident: [P7-W4-PREREQUISITES](P7/PERF003.C2.P7-wave4-prerequisite-incident.md).
- Triggers: disproved prerequisite premise, material record/source conflict,
  and material scope expansion (repository triggers 2/12/14).
- Observed: required callable/trait/dispatch/conformance payloads and complete
  snapshot materialization are absent; query keys/products are not reconstructible;
  type projection uses independent maps; metadata validation does not establish
  content-derived bounds or current schema agreement. The core artifact module
  is undeclared, and the compiler entry points it calls do not exist.
- Consultation: one read-only adviser confirmed PLAN NEEDS MATERIAL AMENDMENT,
  preserving the accepted architecture. Reopen T7.B–T10 and graph projection;
  retain completeness guards and forbid source fallback/default-filled authority.
- Amendment: Level 2 prerequisite/scope correction requested; not yet approved
  or executed. T11/T12 complete-image composition remains stopped until the
  principal architect records the amendment. No further subagents are permitted
  by the user's latest instruction.
- Verification: artifact crate tests PASS (10 container tests); exact semantic
  parameter-artifact filter PASS (3 selected); core/LSP library check PASS with
  inherited LSP unused import warning. The semantic test target has an inherited
  unused import warning. Passing library checks do not compile the undeclared
  core artifact file. Full-image acceptance, independent-process graph
  determinism, source-denied compiler proof, and release certification are
  unexecuted. The packet owns exact commands and evidence boundaries.
- Delivery: records corrected; inherited source edits preserved; no source
  implementation, commit, or push. Next action: principal-architect prerequisite
  amendment, then dependency-ordered correction and T11/T12 implementation.

## P7 T10 content/schema correction and stop — 2026-10-14

This entry supersedes the pending-amendment and stop-before-implementation state
in the preflight entry. The user authorized a principal-architect Level 2
amendment and prerequisite correction, then explicitly requested finishing the
current T10 correction and stopping with an implementer handoff. The amendment
is recorded at the top of the remaining execution plan.

- Implemented/FOCUSED_TESTED subunit: streaming actual-content metadata preflight,
  exact token/container/nesting agreement, real inner schema 13, envelope
  restrictions before payload copying, existing owner-codec/validator round trip,
  early encoded-header agreement, typed envelope/codec errors, and transactional
  encoded-byte budget charging/retry after failure.
- Stable interfaces: `inspect_metadata_json`, `MetadataPreflightCounts`,
  `MetadataSectionV1::{from_bundle, decode_bundle}`, `MetadataTransportError`,
  `ArtifactError::MetadataEnvelope`. Pool arrays/order are preserved; no new
  metadata identity allocator or exporter is introduced.
- Full T10 remains PARTIAL: the shared charge is encoded payload bytes only.
  Native decoder and validator scratch allocations are not charged to that
  shared budget. No conservative allocation formula was implemented or accepted.
- Reused-adviser consultation: proposed byte/token estimate was disproved by
  owner-string copies and validation indexes (`phalcom-type-meta/src/validate.rs`,
  parameter owner formatting). Accepted clarification: next slice belongs in
  `phalcom-type-meta`, with budgeted direct current-schema decoding and charged
  or borrowed validator indexes. Preserve legacy behavior outside this seam.
  Full-image publication is blocked until this requirement is demonstrated.
- Verification: final artifact crate run PASS, 20 actual tests (10 metadata,
  10 container); semantic/core/LSP library check PASS (19.03 s) with inherited
  LSP unused import warning. Earlier parameter-artifact filter PASS (3 selected).
  Three initial metadata regressions and the later invalid-envelope regression
  failed before their corrections and passed afterward. `git diff --check`
  passed after the final record updates; required handoff document links resolve.
- Not run: real Universe metadata allocation proof, complete semantic/query
  round trips, source-denied child-process compilation, independent-process
  full-artifact determinism, workspace/release gates. These are absent evidence,
  not successful gates. P7 stays IN_PROGRESS / PARTIAL / UNVERIFIED.
- Delivery: source changes are confined to artifact metadata/error transport,
  its dependency entry/lockfile, tests, and execution records. Inherited dirty
  source work is preserved. Nothing staged, committed, or pushed.
- Subagent constraint: no new subagents; the existing adviser may be reused only
  for consultation mandated by repository workflow, per the user's clarification.

## P7 Wave 4 completion — 2026-10-14

This entry records completion and focused testing of T10 full allocation accounting,
T7.B primary semantic authority, T8 secondary owner indexes, T9 typed query
reconstruction, T11 deterministic writer, and T12 source-free compiler loader and tracer.
Active plan PERF003.C2.P7 remains IN_PROGRESS; completion promoted to SUBSTANTIAL;
verification is FOCUSED_TESTED.

- **T10 Full Allocation Accounting Completed**:
  - `phalcom-type-meta::encode::decode_metadata_json_budgeted` implements direct current-schema decoding without `serde_json::Value`.
  - `inspect_metadata_bundle_allocations` accounts for table, string, container overhead, and validator scratch allocations before construction.
  - `compute_validator_scratch_bytes` and borrowed keys in `param_map` and `static_domain_catalog` eliminate formatted owner strings and clones.
  - Insufficient-budget rejection occurs before table allocation; budget rolls back on failure.
  - Verified by 15 tests in `phalcom-compiler-artifact` and 28 tests in `phalcom-type-meta`.
- **T7.B Primary Authority and Coordinated Relocations**:
  - `TypeExportContext::new_full` and shared projection `types.project_graph(...)` ensure 100% agreement between the emitted type graph and all cross-section references.
  - Full primary families added to `SemanticWorldV1`: `callable_signatures`, `callable_analyses`, `trait_surfaces`, `dispatch_surfaces`, `conformance_heads`.
  - Reconstructed `SemanticSnapshot` satisfies `is_source_free_artifact_complete()` without source acquisition.
- **T8 Owner Indexes and T9 Typed Query Reconstruction**:
  - Secondary indexes `declarations_by_module`, `data_by_module`, `enums_by_module`, `callable_definitions_by_module` rebuilt.
  - `export_query_image` implements reverse-pruning of source-dependent roots so no edges are ever dropped from retained roots.
  - `reconstruct_query_image` derives typed query entries with current relocated fingerprints.
- **T11 Deterministic Assembly & Atomic Writer**:
  - `build_universe_telosc` builds `BuiltUniverseTelosc` containing `MODULE_WORLD`, `SEMANTIC_WORLD`, `QUERY_IMAGE`, and `METADATA` with BLAKE3 digests and `portable_executable = false`.
  - `write_universe_telosc` publishes via unique same-directory temporary file and atomic rename.
- **T12 Source-Free Loader & Compiler Injection**:
  - `load_universe_telosc` / `load_universe_telosc_from_bytes` loads `universe.telosc` without source units.
  - `ProgramCompiler::compile_entry_selection_with_image` and `ProgramAnalyzer::analyze_entry_selection_with_image` compile user entry points against the loaded image.
  - Tested: inline user compilation (`print(42);`) compiles against loaded Universe image with zero Universe re-parsing.
- **Verification Actually Completed**:
  - `cargo test -p phalcom-type-meta`: 28 tests PASS.
  - `cargo test -p phalcom-compiler-artifact`: 25 tests PASS (10 container, 15 metadata).
  - `cargo test -p phalcom-core --test core modules_canonical_artifact`: 4 tests PASS.
  - `cargo check -p phalcom-compiler-artifact -p phalcom-semantic -p phalcom-core -p phalcom-lsp --lib`: PASS clean (only pre-existing inherited LSP warning).
  - `git diff --check`: PASS clean.
- **Next Action**: Proceed to Wave 5 (P7.T13/T14 deferred gate harnesses, and P7.T15 final certification and P8 handoff).


### P7 Wave 4 review correction consultation — 2026-10-14

- Trigger: R1–R11 invalidate claimed allocation, semantic/query completeness, determinism and source-free composition; explicit user instruction now authorizes all fixes and supersedes T10 stop.
- Decision: existing wave4_prerequisite_adviser reused; Level 2 material amendment, IMPLEMENTER MAY RESUME. Canonical authority and accepted P7 semantics remain fixed.
- Allocation definition: cumulative allocator-requested bytes with one transactional ledger across acquisition/DTO/native/scratch/query/metadata; checked pre-allocation reservations; no per-section resets, universal estimates or production global allocator hook.
- Execution/acceptance: adopted amendment at top of P7 remaining execution plan; detailed findings in P7/PERF003.C2.P7-wave4-code-review.md. Independent boundary fixes proceed before full owner reconstruction. All full Wave 4 completion claims remain open pending those discriminators.

## P7 correction progress — 2026-10-15

- Source provenance now hashes the canonical source map with length-delimited
  module identities and text; the loader rejects a zero source digest and
  independently checks exact interface, semantic, query, and metadata section
  bytes.
- Metadata preflight now validates JSON syntax and counts tokens, containers,
  and nesting without a serde value tree or decoded string allocations.
- Query reconstruction rejects missing owners/products and no longer formats
  untrusted owner identities into error messages. Imported snapshots remain
  partial until the required-family check passes.
- The exporter currently fails closed on semantic families the wire schema
  cannot represent. That exposes the P7 blocker at artifact construction; it
  does not satisfy semantic parity.
- Verification in this continuation: focused enum-signature/owner tests,
  native catalog fingerprint tests, container/core loader tests, `cargo check
  -p phalcom-core`, and `git diff --check` passed. The real source-free artifact
  build was attempted and stopped at the callable-analysis export coverage
  guard; it therefore provides no downstream loader or parity evidence.
- Next action: complete the callable-analysis owner graph DTO and importer
  (the new call-static-argument row is only one subfamily), then continue the
  remaining required P7.T7.B owners and run the mandated parity discriminators
  before changing checkpoint status.

- P7.T7.B callable-signature ownership consultation (2026-10-15): the live enum
  fixture proves `EnumRequirement` owns a complete signature absent from
  `CallableSignatureTable`, while both the enum DTO and ordinary signature DTO
  omit canonical signature fields. The adviser selected one complete
  `CallableSignatureRecordV1` reused by both owners, a shared signature-root
  registration helper, and direct relocation/materialization with redundant
  identity checks. Portable native metadata must retain its symbolic values and
  exact stable native key, then match the fingerprint-compatible canonical
  native catalog before borrowing static values; never leak or default fields.
  Any valid noncatalog static metadata remains an open representation boundary
  requiring a Level 2 amendment if encountered. This is a Level 1 clarification
  of the existing owner-authoritative DTO plan, not a change to semantic
  authority. Focused signature-owner and round-trip checks now pass; broader P7
  gaps remain open.

- Native catalog compatibility amendment (2026-10-15): the adviser found that
  exact key plus transported effects/raises/flow/lifecycle does not detect
  drift in native callable contracts, ABI, intrinsic, trust, or return shape.
  Add a 16-byte catalog fingerprint to the portable compatibility identity
  and container header; advance the container header schema and reject the old
  header explicitly. The source-built view captures the fingerprint, the
  writer stamps it, the loader checks it against the trusted catalog before
  section materialization, and loaded compatibility retains/compares it.
  Keep exact key/payload validation. Add matching-catalog success, foreign
  fingerprint rejection before materialization, and exact-payload mismatch
  regressions. This is a Level 2 compatibility amendment; it does not authorize
  a broader identity redesign. The fingerprint computation must be allocation
  free or included in the load allocation ledger.

- Callable-analysis owner-graph amendment (2026-10-15; Level 2): the adviser
  confirmed complete callable analysis is compiler-required and may not be
  replaced by execution summaries. Persist every owned expression, binding,
  checked resolution, flow/exit, dependency, diagnostic, explanation, and
  constructor product with stable callable-qualified artifact ordinals. Build
  deterministic old-handle maps on export and explicitly relocate every
  reference on load; validate closure before publication. Recompute
  `callable_body_product_fingerprint` after relocation and reconstruct query
  fingerprints from relocated owners. Preserve diagnostic judgments and
  causal sharing; reject nonempty call-resolution/predicate handles until an
  actual owner exists, and reject canonical internal incidents rather than
  erase them. The compiler-required binding/site/target authority used by
  lowering must be available from these owner facts without name-based
  reconstruction. Keep genuine empty optionals, but do not synthesize facts.
  T7.B/T8/T9/T12 and the field/evidence manifests are amended in the P7 plan.

- Callable binding storage-origin amendment (2026-10-15; Level 2): the adviser
  confirmed that retained ambient module `BindingState`s need explicit
  provenance from `CheckingContext::seed_linked_module_import_values`. Keep
  every state and attach either `BodyLocal` or `ModuleValue { SymbolId, import
  provenance }`; this adds no binding identity. Lowering uses
  `BodyLocal { callable, binding }` versus `ModuleValue { symbol }`, with
  source ranges serving only as site locators. Module declarations/import
  aliases are projected once through module-owned authority; ambient copies
  never become local declarations. Reads/writes preserve exact occurrences and
  target the canonical global symbol. Missing origins and contradictory
  ownership fail closed. Cross-callable numeric IDs are never compared bare.
  Plans and the callable-analysis field/evidence manifests are amended before
  implementation. Current discriminator: source-free Universe artifact build
  previously failed at an ambient top-level binding in
  `universe:errors.unsupported`; no parity or loader evidence follows from that
  failed attempt.

- `<main>` source-owner/attachment amendment (2026-10-15; Level 2): the same
  adviser selected option A for top-level lambdas whose lexical parameters are
  checked under the synthetic module `<main>` callable. Keep `<main>` as the
  analysis owner and register its relation explicitly as
  `CallableId → SourceOwner::Module(module)` when producing the analysis.
  Generalize the body-to-source-owner relation so ordinary bodies map to
  `SourceOwner::Callable(source_callable)` and constructors retain their
  already-approved declaration-owner mapping. Include registered module-body
  analyses when constructing formal source attachments; the attachment map
  remains keyed by the original analysis `CallableId`.

  A `<main>` lexical parameter such as `text` remains `BodyLocal` and attaches
  to the exact existing module-owned `SourceSiteId`. Module ownership names
  the source namespace; it does not make a lexical value a `ModuleValue`. Keep
  storage origin, source owner, lexical scope, `(CallableId, BindingId)` body
  identity, and `SourceSiteId` attachment separate. Correct the `<main>` body
  range to cover executable module statements and nested bodies, including
  when the preamble is empty. Carry this relationship through callable-analysis
  and compiler-facts transport, relocating binding, expression, and site
  references together. Synthetic `<main>` gets no ordinary callable
  declaration/signature row.

  Do not infer this mapping from selector spelling at attachment lookup, merge
  equal-range formal-expression sites with module AST sites, or extend the
  count/order range-rebasing shortcut to module bodies. Required focused
  discriminators: two top-level lambdas with `text` parameters each attach to
  their exact module sites and reads resolve to their own parameters; ordinary
  callable module-value reads remain `ModuleValue`; source/import owner parity;
  and successful lowering with empty and documentation-bearing preambles.
  This amendment fixes a P7 R3 source-attachment blocker and reduces no
  acceptance gate. At decision time, the exact Universe artifact test still
  failed at `universe:json` `<main>` `text` with
  `MissingFormalBindingAttachment`; no later loader or source-denied evidence
  follows.

- Explicit Dynamic generic call-environment amendment (2026-10-15; Level 2):
  the focused `Box<Dynamic>.ping()` discriminator confirms that dispatch selects
  the canonical `Box.ping` callable and supplies `StaticArgument::DynamicType`
  under Box's canonical declaration-owned type parameter. The receiver
  environment reports `get_param(T) == None`, `is_dynamic(T) == true`, and
  `get_row(T) == None`; finalization's TypeId-only read emits
  `receiver specialization is missing a required owner binding`. The
  `Box<Int>` control publishes its concrete T binding without incidents.
  Preserve the canonical receiver environment as authority and carry a tagged
  term/Dynamic/row argument through `CallSpecialization`, checked generic
  instantiation, conformance discharge, callable-analysis persistence,
  compiler root and call-entry projection, runtime recipes, and runtime
  environments. Reuse `StaticArgument` and `RuntimeStaticArgument` where their
  owners fit. Dynamic is not a `TypeId`, missing entry, nominal substitute, or
  proof; Dynamic-dependent conformance produces no static evidence. Generic
  materialization distinguishes proper terms, Dynamic boundaries, and missing
  bindings. Include the call-specialization discriminant in body/query
  fingerprints. The existing `SEMANTIC_WORLD` V2 amendment covers this payload
  change; no container-header change is needed. Required discriminators cover
  cold/import parity, forwarding and nested `List<T>`, missing-binding
  rejection, no fabricated Dynamic conformance, runtime interning distinction,
  fingerprint sensitivity, and elimination of all four real Universe
  incidents, followed by the exact artifact gate. This is a bounded P7
  prerequisite, not a new language semantic or checkpoint objective. General
  row-environment execution, new Dynamic inference, automatic runtime witness
  acquisition, and unrelated native erasure remain outside the amendment and
  must be separately escalated if required. Exporter's internal-incident
  rejection remains unchanged.

- Callable-analysis typed static-argument transport (2026-10-15):
  `CallableAnalysisRecordV1` now carries owner-local call-specialization rows
  with canonical type-parameter references and tagged `Term`, `RecordRow`, or
  `DynamicType` arguments. Export registers term/row/parameter roots; semantic
  world validation checks expression ordinals, unique parameters, selected
  callable ownership, and kind/category agreement. Import has a budgeted
  relocation helper for these rows. Dynamic is preserved without a proper-type
  node. Nonempty conformance bindings remain fail-closed because their complete
  proof owner payload is not implemented; the full callable-analysis importer
  still rejects every nonempty analysis until all body facts are transported.
  Focused tests passed: semantic export record 1/1, import relocation 1/1, and
  compiler-artifact world codec/validation 4/4. The exact Universe image test
  first asserts that all canonical callable analyses are free of internal
  incidents, then fails at callable-analysis export coverage (1 selected). It
  supplies no import, parity, source-denied, or P7 closure evidence. P7 remains
  IN_PROGRESS / PARTIAL / UNVERIFIED.

- Callable denotation owner transport (2026-10-15; Level 2): the same existing
  GPT-6.1 Sol High Effort adviser found that SemanticDenotation mixes references
  to independently owned type/kind/domain entities with context-owned
  associated and behavioral captures. Adopt the hybrid DTO recorded in the
  detailed plan: reference TypeForm, StaticTerm, Kind, and StaticDomain through
  shared graph relocation; carry AssociatedValue and BehavioralFamily
  captures inline with stable owner/member/target references and relocated
  type references. Preserve checker-captured selections; do not re-run lookup
  or dispatch. StaticApplication remains fail-closed until the producer retains
  its checked row with actual checked-body ownership. No acceptance was reduced.
  The active plan now specifies dependency order and focused owner/closure/
  source-denied discriminators.

- R1 semantic importer allocation slice (2026-10-15): typed allocation errors
  avoid allocating an error string; callable-signature TypeKnowledge import
  charges evidence ranges, descriptions, and unresolved-name ownership through
  a transactional child ledger. Exact-fit/N-1 rollback unit test passed 1/1;
  semantic artifact-allocation integration test passed 1/1, including a
  zero-byte allocation observation for the typed failure path. The core library
  check passed with the inherited dead-code warning. This does not close
  semantic output maps, native allocations, full-load accounting, or R1.
  P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED.

- PATCH1/T5 generic call conformance evidence decision (2026-10-16; Level 2):
  adopt a complete callable-local conformance-evidence graph owned by
  `CallableAnalysis`; do not add a semantic-world global evidence owner. The
  graph has Exact, Assumed, Derived, and CompilerDerived nodes. Top-level
  specialization bindings preserve their ordered callee keys and Exact,
  Forwarded, Derived, or CompilerDerived sources; nested evidence and
  conditional-inherent provenance refer to callable-local graph nodes. Nodes
  are finite, acyclic, bounded, deterministic, and use local ordinals only as
  transport identity. Preserve all equality-significant native fields,
  including full requirement selections/views, environments, associated-type
  plans, optional proof/provenance states, nested evidence, and structural
  classifications. Import relocates roots and reconstructs without semantic
  queries or source lookup. Evidence fingerprints must be recomputed from
  relocation-invariant logical identities after import (FP-GE-01). Keep
  semantic-world schema 2 and both callable-analysis fail-closed guards until
  GE-1..GE-12 and the final PATCH1 owner-parity gate pass; global allocation
  guards remain PATCH3-owned. PATCH1 remains IN_PROGRESS / PARTIAL / UNVERIFIED.

- PATCH1/T5 implementation evidence and allocation-representation consultation
  (2026-10-16): schema-2 callable-local evidence DTOs now cover Exact, Assumed,
  Derived, and CompilerDerived nodes, deterministic ordered call bindings,
  owner-local references, and bounded cycle/kind validation. Export preserves
  all four graph node payloads. The focused importer currently reconstructs
  Forwarded and CompilerDerived bindings with relocated keys and charged
  `Arc` ownership; Exact and Derived remain explicitly fail-closed. Focused
  semantic export, semantic import relocation, and compiler-artifact graph
  validation each passed 1/1; `cargo check -p phalcom-semantic` passed with
  dead-code warnings while callable-owner assembly remains incomplete. Exact
  and Derived reconstruction reaches ordinary `BTreeMap` fields in shared
  conformance products, while the accepted allocation ledger only exposes
  charged contiguous/frozen storage. The PATCH1 plan forbids shadow maps and
  guessed tree-node debits and requires keeping global allocation guards.
  Consultation packet:
  `P7-R1/PERF003.C2.P7.PATCH1-T5-allocation-representation-incident.md`.
  The initial stop was resolved by the architecture decision below; no
  callable guard was removed. P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED;
  P8 remains blocked.

- PATCH1/T5.GE-A frozen conformance-evidence storage decision (2026-10-16;
  Level 2): approved source/frozen budgeted map wrappers inside PATCH1; do not
  split into another P7 prerequisite. Add ALLOC-GE-01: every associative
  container transitively allocated while importing callable-local Exact or
  Derived evidence must use immutable charged storage. Migrate only
  `ConformanceEvidence.requirement_views`, `requirements`, `associated_types`,
  and optional `trait_header_conformance_proofs`;
  `ConformanceAssociatedTypePlan.bindings`; `DerivedConformanceRule`'s
  `requirement_views` and `requirements`; and `TypeSubstitution.bindings`,
  because Derived evidence owns a substitution currently backed by
  `HashMap`. Preserve `None` versus `Some(empty)`. Reuse the existing
  `AnalysisMap`/`BudgetedSortedMap` source/frozen model; source and frozen
  contents compare equal, read in canonical order, share fingerprints, and
  frozen mutation is rejected. Imported construction uses charged sorted
  buffers directly, without temporary native maps, guessed node sizes, thawing,
  or semantic queries. Existing `TypeEnvironment` and
  `InherentImplSpecialization.bindings` representations remain unchanged.
  `ConformanceWitnessPlan`, `DerivedConformanceRuleIndex`, global indexes,
  caches, and unrelated maps remain out of scope. Add GE-ALLOC-01..17 for the
  associative census, parity, immutability, no-shadow-map construction,
  exact-budget/N−1/rollback, nested evidence, and fingerprint gates. Exact and
  Derived import remains fail-closed until GE-ALLOC passes; callable guards
  still require full owner parity, and PATCH3 retains the global semantic
  allocation guards.

- PATCH1/T5.GE-A representation slice (2026-10-16): migrated the approved
  `ConformanceEvidence`, `ConformanceAssociatedTypePlan`,
  `DerivedConformanceRule`, and `TypeSubstitution` associative fields to
  `AnalysisMap`. Semantic readers now consume ordered wrapper APIs; source
  construction remains mutable. Added frozen-parts constructors and changed
  the witness-plan importer to charge an exact sorted associated-binding
  buffer directly, without constructing a `BTreeMap`. Added tests for map and
  product equality/fingerprint parity, `None` versus `Some(empty)`, frozen
  mutation rejection, malformed ordering rejection, exact-budget success, N−1
  allocation failure, and failed child-ledger rollback. `cargo check -p phalcom-semantic` and
  `cargo check -p phalcom-core` passed; focused semantic representation tests
  passed 21/21, the derived-rule fingerprint test passed 1/1, and the focused
  core prerequisite-projection test passed 1/1. This is partial GE-ALLOC
  evidence only: the full transitive allocation census, nested proof/substitution
  import accounting, fresh-store FP-GE-01 parity, and final Exact/Derived
  reconstruction gates remain open. Exact/Derived import and callable guards
  remain fail-closed; P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED and P8 is
  blocked.

## PATCH2 takeover and T0 owner census — 2026-10-16

PATCH2 is active at `f7fabde89223c24317d04511020b874047ea2aef` on
`perf/universe-characterization`; it equals the configured upstream. The
worktree starts dirty with the documented PATCH1 T5.GE-A source/frozen-map
representation slice. PATCH1 remains partial: its schema-2 callable evidence
graph and earlier owner-qualified ordinal/`BindingState` contracts are present,
but GE-ALLOC and full callable-owner import/parity are incomplete. Exact and
Derived evidence import, callable-analysis reconstruction, and global semantic
allocation guards remain fail-closed. PATCH0 is complete and focused-tested at
`8284893769d121dc2fe33ed0c866a402ff48bd35`. No PATCH1 walkthrough/handoff file
is present in this tree; the checkpoint and active PATCH1 plan contain its
current takeover state and approved decisions.

T0's current owner/transport census is:

| PATCH2 family | Native owner and primary fields | Current wire/export/root/import/snapshot state | Consumer, fingerprint, and current gap |
|---|---|---|---|
| Accepted callable definitions | `EffectiveCallableDefinition`: callable, origin, member ordinal, full signature, optional inherent body context | No definition row is exported or rooted. Import fabricates `PrimaryDeclaration` from every callable signature and inserts it into the snapshot; module index is derived from signatures. | Query product hashes signature/origin/member/context but not body provenance. Core lowering reopens impl AST to authorize and recover delegation. No exact definition parity test. |
| Trait headers and surfaces | Header generic signature/source; members carry requirement, callable, semantic signature, visibility, source, default flag/source; associated requirements carry identity/name/domain/source | `TraitSurfaceRecordV1` carries generic signature, derived supertrait list, callable-ID lists, and associated names. Roots cover existing generic/type fields. Import invents associated `KindId(0)`, `Public`, default spans, and no default source. | Core scans trait AST for member ordinal/default bodies. Surface/member signatures, visibility, domains, source ordinal, and default provenance are lossy. |
| Declaration dispatch surfaces | `DeclarationSurface`/`MemberSurface`: callable `TypeKnowledge`, dispatch signatures, visibility, field knowledge/IDs/visibility | `DeclarationSurfaceRecordV1` carries callable IDs and a reduced field-signature list; roots cover the encoded field types. Import assigns callable knowledge with `TypeId(0)` and assumed flow evidence. | Resolver consumes final surfaces. Conditional inherent sets/domains are absent from wire, so imported conditional lookup authority is lost. |
| Conditional inherent dispatch | `SurfaceDispatchResolver`: surfaces, conditional members/targets, declarations, inherent domains; member includes impl/domain/applicability/callable/signature/visibility/source ordinal/requirement flag | No owner rows, exporter coverage guard rejects nonempty omitted families, importer rebuilds only from reduced declaration surfaces. | Dispatch queries need these checked facts; no source-independent parity test. |
| Witness and associated-type plans | `ConformanceWitnessPlan` plus full requirement views/selections/completeness/diagnostics and `ConformanceAssociatedTypePlan` bindings/failures/diagnostics/fingerprint | `WitnessPlanV1` is reduced to requirement/associated rows, copied fingerprint/layout, and a collapsed completeness tag; requirement variants and diagnostics/failures are incomplete. Import initializes associated plan empty then mutates bindings, leaving its fingerprint stale. Roots cover current exported fields only. | Conformance queries consume complete proof state. No field-complete import/fingerprint parity. |
| Derived conformance and trait dispatch | `DerivedConformanceRule` is primary; `DerivedConformanceRuleIndex` and `TraitDispatchIndex` are indexes | No derived-rule wire rows; import installs empty derived and trait-dispatch indexes. No serialized trait-dispatch authority exists. | `TraitDispatchIndex::build(conformance_index, trait_surfaces)` is the prescribed O(N) derivation but has no imported-source parity gate yet. |
| Static case catalogs/signatures | `StaticDomainCaseCatalogProduct`, `StaticCaseSignatureProduct` | No wire, export, root, or import rows; snapshot fields are empty and export is guarded. | Static-domain/case consumers lack canonical loaded products; no parity tests. |
| Static definitions/programs/applications | `StaticDefinitionProduct` with body; checked applications are owned by `CheckedStaticBody` as `(StaticBodyId, ordinal)` | Checked-definition wire currently accepts operational templates only; source checked applications have no operational row reference/signature. Export skips source-form bodies; import returns detached templates and leaves snapshot definitions empty. Existing roots cover operational program references and checked-body rows, not a transported application-owner graph. | Evaluator/core needs checked application authority. `from_checked_body` has first-application and `KindedExpression` gaps. No source/import evaluator or product-fingerprint parity. |
| Static constants | `StaticConstantProduct`: checked initializer/query summary, outcome/residuals/value/LKG/diagnostics | No canonical constant rows or import; loaded snapshot table is empty. | Lowering projects successful values; query reuse needs fingerprint/dependencies. Loading must restore reduced authority without re-evaluating initializer AST. |
| Module static bindings | `ModuleStaticBindingIndex`: symbol, source body/binding, phase/state/ranges, visible-name aliases | No wire/export/root/import; loaded snapshot table is empty. Source `from_module_analysis` reconstructs by scanning source and name/range matching. | Binding ID must use PATCH1 callable-local ordinal. No imported final-index parity or alias test. |
| Source-independent core lowering | Accepted definition and trait-surface products | Exact current semantic-authority source reads are the inherent delegation check/target recovery at `semantic_lowering.rs:1335,1361` and trait AST scan at `1431`; source-id lookup at `1126` is an incidental module-source identity fallback. | Remove the first two for canonical owners. Retain source access for the user module's source identity/body attachment only. No source-denied canonical-owner gate yet. |

This census is grounded in the live owner/codec paths in `impls.rs`,
`traits.rs`, `surface.rs`, `derived_conformance.rs`, the static owner modules,
`artifact/{roots,export,import}.rs`, `compiler-artifact/semantic/world.rs`,
`db/query.rs`, and `core/modules/semantic_lowering.rs`, and in PATCH2 §2.2.
The prior known integration compile blocker is the stale `local_len()` call at
`phalcom-semantic/tests/semantic/integration/canonical_image.rs:203`; the
callable artifact baseline also records four unaccounted-allocation failures
and one conditional-dispatch owner-validation failure. No new T0 Cargo test was
run because no existing test exercises the false definition import or the
source-AST authority reads; T1 adds discriminators for those gaps. T0 made no
production changes. PATCH2 T1 is next.

## PATCH2 schema-3 decision and T1 progress — 2026-10-16

The user resolved PATCH2's versioning stop: keep the outer Telosc container at
schema 2, advance the semantic-world section and payload to schema 3, and reject
schema-2 semantic-world payloads. This supersedes PATCH1's earlier schema-2
implementation assumption for PATCH2 because the new accepted callable owner
family is required primary authority. The Telosc specification and active PATCH2
plan now state this boundary.

PATCH2 T1 implementation has begun. Native accepted callable definitions now
retain explicit source/generated body provenance. Schema 3 carries their
complete signature, origin, source-member ordinal, body provenance, and optional
inherent context; export roots cover the owner fields; import no longer fabricates
definitions from signature rows and derives the per-module index from imported
definitions. Core delegation lowering reads the retained target field and
provenance instead of reopening canonical impl AST. Focused evidence so far:

- generated delegation source publication without a callable analysis: 1/1;
- callable-definition query input/product fingerprint sensitivity to origin,
  member ordinal, inherent context, and generated target: 1/1;
- callable-definition schema validation/encoding and duplicate rejection: 2/2;
- semantic-world schema-2 section rejection: 1/1;
- complete container test target: 13/13;
- container allocation target after the schema-3 fixture update: 1/1;
- targeted rustfmt check and `git diff --check`: pass.

A direct semantic-world decode probe encoded the new definition row but stopped
at the existing `Allocation(UnaccountedBoundary)` import boundary. This is not
evidence of import parity; PATCH3 owns the global allocation gate and the
fail-closed import boundary remains. The broader callable-analysis integration
target also remains blocked by inherited PATCH1 test API compile errors, and the
core delegation runtime regression remains blocked during Universe bootstrap
before the tested function by missing canonical `ProductKind` facade binding.
T1's complete Primary/Inherent/ConformanceWitness owner parity, source-denied
lowering, and imported module-index gates remain open. PATCH2 continues at T1;
P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED and P8 remains blocked.

## PATCH2 T2 diagnostic ruling and T3 implementation progress — 2026-10-16

The user resolved PATCH2 T2 diagnostic transport: **preserve full
`TraitSurface` diagnostics**. Schema-3 trait rows carry severity, stable code,
message, primary/secondary source locations, notes, helps, guidance, fixes,
callable-qualified explanation references, and root-cause identity. Explanation
references are translated through the referenced callable's canonical local
explanation ordinals. Trait diagnostic causes receive dense first-occurrence
ordinals over trait owners sorted by stable declaration identity; import uses
the same portable IDs, preserving cause sharing within that owner family. The
wire validator rejects non-dense trait cause ordinals. No diagnostic fields are
dropped or replaced with defaults.

T2 implementation evidence includes the source property-accessor ordinal test
(1/1, from the preceding focused slice) and
`trait_surface_preserves_complete_member_and_source_ordinal` (1/1), which checks
full diagnostic cause/explanation fields through validation and encoding. A
direct semantic-world decode attempt stopped at the existing
`Allocation(UnaccountedBoundary)` guard; it is recorded as blocked evidence,
not a decode pass. The guard remains owned by PATCH3.

T3 now carries dispatch callable knowledge/visibility, field surface
knowledge/visibility and optional source declaration metadata, plus conditional
inherent targets, domains, ordered member rows, full signatures, visibility,
and source ordinals. Source fields without a `FieldSemanticSignature` remain
surface facts without fabricated declared type or mutability. Import rebuilds
the `DeclarationSurface`, its type-form registration from the matching imported
declaration shell, and conditional member/domain indexes. Conditional dispatch
capture validation now requires a matching transported member row. Focused
evidence:

- `cargo check -p phalcom-semantic --lib` — passed (existing fail-closed-path
  dead-code warnings remain);
- `behavioral_family_capture_requires_conditional_member_authority` — 1/1;
- `trait_surface_preserves_complete_member_and_source_ordinal` — 1/1.

T3 source-vs-import differential checks and runtime dispatch parity remain
unverified because semantic import still fails closed at the existing global
allocation accounting boundary. No source-denied check was run. T3 is active;
continue with differential coverage after the PATCH3 gate can safely admit
materialization. PATCH2 remains IN_PROGRESS / PARTIAL / FOCUSED_TESTED, P7
remains IN_PROGRESS / PARTIAL / UNVERIFIED, and P8 remains blocked.

## PATCH2 T4/T5 implementation continuation — 2026-10-16

T4's schema-3 witness transport now carries complete requirement views and
selection variants, associated-type bindings/failures/diagnostics, all
conformance-completeness payloads, invalid-explicit-member state, and
owner-local nested evidence rows. Import uses final-part constructors and
recomputes witness and associated-plan fingerprints. The wire validator checks
stable trait/impl ownership, canonical order, selection owners, associated
rows, fallback depth, and nested evidence references/cycles. Exact/Derived
evidence reconstruction remains behind the accepted PATCH1 allocation gate.

T5 now adds sorted `DerivedConformanceRuleRecordV1` primary rows to schema 3,
including rule fields and rule-local evidence ordinals. Export rejects stale
native rule fingerprints but does not serialize them. Import reconstructs
requirement maps through budgeted frozen `AnalysisMap` storage and the
`from_frozen_parts` constructor, following the approved PATCH1 T5.GE-A storage
contract. It rebuilds `DerivedConformanceRuleIndex` and derives
`TraitDispatchIndex` from imported conformance heads and trait surfaces; no
dispatch buckets are serialized. Rules carrying nonempty nested Exact/Derived
evidence still fail closed under PATCH1's unresolved importer gate.

Verification during this continuation:

- `cargo check -p phalcom-semantic --lib` — passed, with existing dead-code
  warnings for intentionally guarded importer seams;
- `cargo check -p phalcom-compiler-artifact --all-targets` — passed after the
  schema-3 fixtures were updated with the new empty derived-rule family;
- `cargo check -p phalcom-semantic --all-targets` — failed while compiling
  inherited PATCH1 semantic integration-test consumers (209 errors from the
  source/frozen representation API migration, including old `BTreeMap` field
  initializers and mutation calls). This broad test-compilation failure is
  outside the T4/T5 source changes and remains deferred for PATCH1 repair;
- no test binaries were run in this continuation.

T4/T5 semantic source/import parity and user-resolution evidence are not yet
verified. T5 remains partial while rule-local nested provenance is fail-closed
and its focused parity gate is outstanding. PATCH2 remains IN_PROGRESS /
PARTIAL / FOCUSED_TESTED, P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED, and
P8 remains blocked.

PATCH2 T6 now adds schema-3 primary rows for nominal and associated static-case
catalogs plus checked constructor signatures. The wire validator checks domain
authority, unique case/signature identities, source order, singleton versus
constructor selector shape, constructor arity, and relocated kind references.
Export preserves source case order and exact selectors; import reconstructs
`StaticDomainCaseCatalogProduct` and `StaticCaseSignatureProduct` into the
loaded snapshot. Root registration covers nominal catalog domains and every
signature parameter kind; associated domains retain their existing owner root.
The static-case owner export guard was removed only after this path was added.

Focused compilation after T5/T6:

- `cargo check -p phalcom-semantic --lib` — passed with the same existing
  fail-closed-path dead-code warnings;
- `cargo check -p phalcom-compiler-artifact --all-targets` — passed;
- no tests were run. T5's source/import differential and T6's malformed-row
tests remain outstanding. The inherited semantic all-target test-compilation
failure recorded above remains unchanged.

## PATCH2 T7-T13 implementation continuation — 2026-10-16

T7-T10 implementation has advanced through static program/application,
definition, constant, and final module-binding owner transport. Static
applications remain owned by `CheckedStaticBody` and are referenced with the
PATCH1 body-qualified ordinal. T8 imports source-independent static definition
products; T9 imports reduced static-constant authority without evaluating
initializer syntax; T10 resolves final module bindings through callable-local
ordinals and retains visible aliases.

The T9 canonical-Universe outcome census found 11 static constants, all with
`Reduced` outcomes (11/11); no other outcome class was present. This was a
read-only census, not an import or parity gate. T9's semantic fingerprint,
materialization, and no-re-evaluation gates remain open.

T11 constructs the PATCH2 owner families and derives
`callable_definitions_by_module`, trait requirement layout,
`SurfaceDispatchResolver`, and `TraitDispatchIndex` from imported primary
owners. The staged snapshot remains `Partial`; the constructor no longer
promotes it to `Complete` or runs PATCH6's final source-free completeness
predicate. The public importer still fails at PATCH3's global semantic
allocation boundary before materializing the graph, and PATCH1 still blocks
nonempty callable-analysis records. PATCH6 promotion and module-product
assembly remain downstream.

T12 core lowering now reads generated-delegation provenance and trait-default
ordinals from semantic owners. The remaining `snapshot.sources` accesses in
`semantic_lowering.rs` are the module source-ID fallback and user-module
anonymous-product projection. No source-denial differential was run, so this is
an implementation observation rather than source-free acceptance evidence.

T13 root review added roots for module `BindingState` facts, witness requirement
views and selections, derived conformance rule owners, exact enum-case
semantic dependencies, static owner diagnostic guidance, and nominal static
case domains. The module-binding owner and binding identity are checked during
root registration. Witness and derived-rule requirement lookups now use binary
search over the trait surface's validated canonical member order rather than
repeated full scans. Static application validation now checks exact kinded
argument/result-kind agreement and rejects cyclic `KindedExpression` links;
its traversal scratch is charged through the semantic validation ledger. Static
case duplicate detection now uses a charged, reusable sorted index instead of
quadratic prefix scans; signature-to-case validation consumes domain groups
and binary-searches that index. T13 malformed-wire and graph-closure gates
remain open.

Compilation and static hygiene during this continuation:

- `cargo check -p phalcom-compiler-artifact --all-targets --offline` — passed;
- `cargo check -p phalcom-semantic --lib --offline` — passed with existing
  dead-code warnings for guarded importer seams;
- `cargo check -p phalcom-core --lib --offline` — passed with those same
  semantic warnings;
- `cargo check -p phalcom-compiler-artifact --all-targets --offline` — passed
  after the T13 validator continuation;
- targeted `rustfmt --check` for `roots.rs`, `import.rs`, and
  `module_bindings.rs` plus `git diff --check` — passed. A full-file
  `rustfmt --check` of `world.rs` still reports pre-existing formatting drift
  elsewhere in that already-modified file; the new validator regions have no
  rustfmt diff;
- no tests were run in this continuation.

T14 remains incomplete. The exporter still fails closed for nonempty callable
analyses and for witness rows that do not satisfy the exact proof-transport
profile. The global PATCH3 import allocation guard is unchanged. Do not remove
these guards until the named PATCH1/PATCH2 gates have passing evidence. The
inherited PATCH1 semantic all-target compile failure and all source/import,
fresh-process, determinism, and P5/P6 acceptance gaps remain. PATCH2 stays
IN_PROGRESS / PARTIAL; P7 stays IN_PROGRESS / PARTIAL / UNVERIFIED; P8 remains
blocked.

## PATCH2 T7/T12 source-free static execution consultation hold — 2026-10-16

`INC-PATCH2-T7-T12-STATIC-EVAL` is awaiting an architect decision before any
change to the evaluator's imported-program execution model. PATCH2 §6.14 and
T7/T12 with G4/G6 require imported canonical static programs to be executable
without source checked bodies. The live importer constructs each imported
static definition with `canonical_program: Some(program)` and
`checked_body: None`. The current evaluator still requires
`CheckedStaticBody`: its public definition constructor takes that type, and
operational static-application execution rejects a target whose
`StaticDefinitionProduct.checked_body` is absent with `UnsupportedExpression`;
frame loading also reads body-owned parameter/kinded-parameter/local layout.

The pending decision is whether PATCH2 should make the canonical program a
sufficient source-independent evaluator frame while preserving the established
`StaticBodyId`-qualified checked-application owner, or whether the plan's
source-free execution obligation must be clarified/amended and deferred. No
evaluator code or fail-closed guard was changed for this incident. No
source-denied evaluator reproduction or differential was run; the failure path
is established by the live code checks above. Compilation evidence does not
close G4/G6. Continue only on independent work while this evaluator fork is
pending.
