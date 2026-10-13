# PERF003.C2.P4 handoff — user-stopped T5 / PR-A continuation

## 1. Execution boundary

- User explicitly stopped execution on 2026-10-13, then authorized a focused
  T5 reachability fix in the review follow-up. Other P4 work remains stopped.
- Worktree: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization`.
- Branch: `perf/universe-characterization`.
- Committed/pushed starting HEAD: `e74a9213` (T1 and previous lifecycle/incident records).
- Current T5 / PR-A source and tests are uncommitted. No commit/push requested for this continuation.
- The earlier continuation used no subagents. The review follow-up used one
  read-only architecture adviser; no implementation was delegated.
- P4: `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED`. Focused evidence below is narrower than T5/T6 certification.
- `CHECKPOINT.md` remains the durable authority. Read the active P4 plan and
  [accepted architectural ruling](PERF003.C2.P4-architectural-ruling.md).

An unrelated untracked `PERF003.C2.P5-canonical-publication-persistent-image-boundary-implementation-plan.md`
appeared during this continuation. It was not read, edited or included in task
work. Preserve it. The previously committed P1–P9 handoff is also not task-owned
source to rewrite without reason.

## 2. Accepted architecture

The user approved two bounded T6 prerequisites while T5 continues:

- PR-A: distinguish compiler/source-owned intrinsic data from native runtime
  Tuple/Record/Pack facades; preserve fail-closed canonical identity checks.
- PR-B: versioned heterogeneous fixed Record keys and anonymous remainder key
  domains, including scoped and RecordRow paths, fingerprints and legacy
  Symbol-key decoding.
- T1 stays exactly as accepted: default RuntimePublic, explicit profile request,
  independent semantic publication, unchanged retained-session assertions.
- Canonical RuntimePublic must faithfully retain public aliases, including
  `universe.reflection.structural.FixedFields = { **[FieldKey]: FieldInfo }`.
- T6 certification and T7–T9 remain behind usable faithful canonical authority.
  No lossy key/domain conversion or required-record omission is authorized.

## 3. T5 implementation present in the working tree

`phalcom-semantic/src/metadata/export.rs`:

- `build_bundle_from_closure` projects explicit retained declaration/alias/
  callable/field IDs, with closure-scoped project-identity validation.
- Declaration lists use callable and field owner indexes; lists filter against
  retained membership. Fields are exported and associated with declarations.
- `build_complete_test_bundle` is an explicit complete-table detached fixture
  helper. Existing integration fixture callers were mechanically renamed.
  Compiler production callers no longer use that complete-table API.
- Static-domain production catalog discovery starts from exported domain IDs
  and retained declaration products, following constructor-kind dependencies.
- `MetadataEntityIdentity` and `MetadataExportError::Projection` carry profile,
  retained top-level entity, optional failing TypeId and typed underlying cause.
- Known declared type facts fail closed instead of silently converting export
  failures to an unavailable slot. Inferred-return fallback still needs review.

`phalcom-semantic/src/metadata/reachability.rs`:

- `MetadataRetentionRequest::with_member_surfaces` supplies published visibility.
- Owned callable/field retention consults canonical Public member visibility
  when surfaces are supplied; production supplies it.
- Inherited roots/closure/owner indexes are preserved, not rebuilt from scratch.

`phalcom-core/src/modules/compile.rs`:

- Builds roots from executable runtime type/call facts, source-index Class/Data
  declaration IDs and linked public exports, including transparent aliases.
- Supplies field signatures and member surfaces and records reachability metrics.
- Uses explicit closure export and structured `ProgramCompileError::Metadata`.
- External-module predicate excludes canonical records for ordinary compilation
  while preserving canonical bootstrap ownership.

`phalcom-core/src/modules/semantic_lowering.rs`:

- Data-construction metadata uses an explicit runtime-type closure rather than
  the complete-table exporter API.

Important: this is a partial production cutover. Canonical elision is already
present in the working-tree compiler, while T6 installation/indexing and
compatibility work are not. Do not infer runtime safety/cross-pool certification
from the tiny compilation test.

## 4. PR-A implementation and bounded evidence

`phalcom-core/src/native/source.rs::index_program` classifies intrinsic runtime
Data before the facade indexer:

- Canonical Tuple/Record/Pack: existing facade/native-anchor path.
- Other canonical runtime-class key: rejection.
- Wrong canonical module: rejection.
- Not a runtime-catalog name: compiler/source-owned intrinsic data; no native
  anchor. Sum declarations use ordinary enum/census indexing.

No `UniverseKey`, native class relation, or ProductKind allowlist was added.
The new ProductKind/Presence regression failed before the repair and passed
after it. All nine `native::source::tests` pass, including actual bundled
Universe source indexing. The dedicated facade fixture asserts Tuple; add
explicit Record/Pack facade coverage before claiming the full ruling matrix.

The exact canonical singleton was NOT rerun after PR-A. Source-index success
alone does not prove native contract verification, semantic projection or
canonical metadata export.

## 5. Verification actually completed

| Command/filter | Result |
|---|---|
| Semantic integration filter `retained_alias_error_keeps_root_and_unreachable_alias_is_not_exported` | 1 PASS, 0.01 s |
| Core lib filter `metadata_profile_is_explicit_and_independent_of_semantic_publication` | 1 PASS, 43.93 s |
| New compiler-intrinsic source test before PR-A | 1 FAIL on ProductKind, expected reproducer |
| Core lib filter `native::source::tests` after PR-A | 9 PASS, 0.05 s |
| Semantic lib filter `metadata::reachability::tests` after nested fix | 4 PASS, 0.01 s |

The retained-session assertion is unchanged from committed T1 and now passes
public -> minimal -> public plus independent semantic publication. The old
Record failure was irrelevant aggregate export for this exact fixture.

The semantic alias test retains a fixed String-key Record alias and checks a
structured failure; it does NOT exercise a non-Symbol anonymous remainder. It
also proves the same alias is omitted when not retained. After PR-B, update this
valid form's expectation to successful faithful export and retain a separate
negative provenance test using a truly unsupported/invalid retained form.

The semantic alias run initially printed five mechanical warnings; their
unused import/identity-loop bindings were then cleaned up and subsequent core
builds completed. The alias filter itself was not rerun after that cleanup.
No broad renamed-fixture suite, canonical singleton, schema test, reflection,
product integration, benchmark, final formatting gate or independent review
completed in this continuation.

Logs from completed checks:

- `/tmp/perf003-p4-closure-test.log`
- `/tmp/perf003-p4-profile-t5.log`
- `/tmp/perf003-p4-source-red.log`
- `/tmp/perf003-p4-source-green.log`

## 6. T5 issues to resolve before acceptance

- The inherited closure traversed only a subset of `TypeData`, while the exporter
  recursively serializes the full type graph. The review follow-up adopts the
  exporter as the sole type-child traversal authority: a disposable exporter
  reports nominal dependencies from successfully exportable closure roots, and
  RuntimePublic closure expands to a fixed point. The partial walker was removed
  from reachability. This code is verified by `metadata::reachability::tests`
  including `reachability_closure_discovers_nested_nominal_dependencies` (covering
  Record, Callable, Pack, AssociatedProjection, external elision and rollback on
  failure).
- Review alias dependencies, generic/kind dependencies and all executable roots.
  Tiny-fixture success is not proof those closures are complete.
- Review provenance resetting: `failed_type` currently captures the first failed
  recursive type; ensure stale failures/fallbacks cannot misidentify later roots.
  Current errors identify top-level entity and TypeId, not a full edge-path trace.
- Inferred-return export still uses `.ok()` fallback; audit this against the
  ruling's no-lossy-export requirement.
- Review public retention, required/private dependency handling, deterministic
  lists, retained fingerprints and bounded static-domain catalog behavior.
- Add the explicit non-Symbol remainder positive control and profile/cardinality
  gates; existing fixed-key test is not that control.

## 7. PR-B investigation at the stopping point

No `phalcom-type-meta` source or schema was modified. Schema remains v12.
No runtime representation/identity/reflection changes or migration tests exist.

Owning paths inspected:

- `phalcom-type-meta/src/type_node.rs`: RecordFieldRef, anonymous remainders,
  RecordRowRef and global TypeNode graph.
- `phalcom-type-meta/src/scoped_type.rs`: scoped fixed keys/remainders/rows.
- `phalcom-type-meta/src/encode.rs`: direct serde JSON encode/decode and validation.
- `phalcom-type-meta/src/header.rs`: supported versions 1–12.
- `phalcom-type-meta/src/validate.rs`: key ordering/duplicates, graph topology,
  kinds, rows, scoped paths and feature/version floors.
- `phalcom-type-meta/tests/schema_compat.rs`: compatibility fixtures and sample
  bundle helpers; v1 JSON fixture exists under `tests/fixtures/schema-v1/`.
- `phalcom-semantic/src/metadata/export.rs`: fixed-key/remainder export and global/
  scoped/row fingerprints; current Symbol-only failures are still present.
- `phalcom-core/src/typing/{identity,inspect,environment}.rs`: runtime Symbol-only
  row conversions, display and child normalization needing faithful key domains.

The ruling's candidate representation is a tagged RecordKeyRef with Symbol,
String, exact StaticInt and Bool, plus explicit key_type/value_type node IDs on
anonymous remainders, global and scoped. StaticInt already has exact equality/
ordering and is shared by semantic and metadata crates.

A legacy JSON normalization approach was being evaluated, NOT chosen or coded:
convert old `name` to Symbol keys and give old anonymous remainders a canonical
Symbol domain. If new global/scoped Symbol nodes are prepended, every affected
node reference must be shifted (including generics, kind environments,
declarations, runtime roots and occurrences) to preserve topology. Simply
appending Symbol nodes can create forbidden forward edges. Compatibility must
preserve old semantics, supported-version behavior and meaningful fingerprints.
Do not mistake that investigation for an approved migration implementation.

## 8. Resume order

1. Consume the accepted ruling, this handoff and checkpoint; inspect the current
   scoped diff. Do not reimplement T1 or repeat the obsolete incident diagnosis.
2. Finish T5 closure/provenance/positive-control coverage while retaining the
   accepted defaults and fail-closed export behavior.
3. Finish PR-A facade boundary coverage; current bundled source-index test is
   green. Recheck canonical singleton at the coherent prerequisite gate.
4. Implement and test versioned PR-B global/scoped/row key fidelity, legacy
   migration, validation and fingerprints; certify canonical FixedFields.
5. Only with faithful usable canonical RuntimePublic authority proceed through
   T6 -> T7/T8 -> T9 and the plan's focused runtime/profile/cardinality gates.

The user's later request resumed only the nested nominal closure issue. The
focused source change is uncommitted and unverified; no tests, commit or push
were performed for it.
