# PERF003.C2.P7 implementation continuation guide

Date: 2026-10-14.

Status at capture: `IN_PROGRESS`; completion: `PARTIAL`; verification:
`UNVERIFIED`.

This is an implementation guide, not a lifecycle state authority. The shared
[`CHECKPOINT.md`](../CHECKPOINT.md) remains authoritative for checkpoint state,
plan status, durable verification evidence, consultations, and next action. Use
the active
[`remaining execution plan`](PERF003.C2.P7-remaining-execution-plan.md) as the
execution contract and the
[`detailed implementation plan`](PERF003.C2.P7-telosc-persisted-semantic-compiler-image-detailed-implementation-plan.md)
for acceptance requirements. This guide records source-grounded findings and
the exact continuation route so the next implementer does not need to repeat
the investigation.

## 1. Resume coordinates

- Worktree:
  `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization`
- Branch: `perf/universe-characterization`
- Inherited starting revision: `3ca30fca`
- Preserve all unrelated modified and untracked work. Do not reset, clean,
  restore, or broadly format this worktree.
- No commit or push has been requested.
- The inherited check policy is compiler-driven checks only. Do not run tests
  or broad certification unless the user changes that instruction.
- The worktree contains generated patch and text files under this directory and
  P9. They are not evidence that the represented implementation is complete.

Read these records before editing:

1. [`PERF003.C2.P7-handoff.md`](PERF003.C2.P7-handoff.md)
2. [`PERF003.C2.P7-walkthrough.md`](PERF003.C2.P7-walkthrough.md)
3. [`PERF003.C2.P7-remaining-execution-plan.md`](PERF003.C2.P7-remaining-execution-plan.md)
4. [`PERF003.C2.P7-P8-execution-research.md`](../Program-Architecture-and-Planning/PERF003.C2.P7-P8-execution-research.md)
5. The task-specific section of the detailed implementation plan, rather than
   rereading the entire plan for each implementation unit.

Do not re-audit T1 through T4. Treat the handoff's established interfaces as
inherited unless the live source directly contradicts them.

## 2. Current worktree state

At capture time, tracked modifications were present in:

- `phalcom-compiler-artifact/src/semantic/type_store.rs`
- `phalcom-semantic/src/lib.rs`
- `phalcom-semantic/src/types/artifact.rs`

Untracked implementation files included:

- `phalcom-compiler-artifact/src/metadata.rs`
- `phalcom-compiler-artifact/src/query.rs`
- `phalcom-compiler-artifact/src/semantic/aggregates.rs`
- `phalcom-compiler-artifact/src/semantic/static_value.rs`
- `phalcom-core/src/modules/canonical_artifact.rs`
- `phalcom-semantic/src/artifact/`

The modified type-export implementation and artifact manifest previously passed:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
```

That result does **not** compile or validate the current static-value and
aggregate sketches. `phalcom-compiler-artifact/src/semantic/mod.rs` exports only
`type_store`; it does not yet export `static_value` or `aggregates`.
`phalcom-compiler-artifact/src/lib.rs` likewise does not export the untracked
`metadata.rs` or `query.rs`. The latest replacement of `static_value.rs`
happened after the last successful check and remains unverified.

Do not confuse an unreferenced Rust file with implemented code. Before adopting
any untracked DTO file, wire only the smallest coherent module seam, compile the
owning crate, and then correct mechanical errors. Do not wire every generated
file at once.

## 3. Established boundary through T5

The inherited module boundary is already established:

- `CanonicalExternalModuleWorld` owns completed `LinkedProgram`, interfaces,
  and `ModuleTopology` without requiring source-backed linking.
- `CanonicalModuleProducts` combines that external authority with optional
  source-build units and fingerprints.
- `CanonicalUniverseFormalImage` keeps source-free external authority separate
  from optional source presentation.
- External source loads fail explicitly. Logical source locations and compiled
  interfaces remain available from artifact authority.
- Artifact hashing is BLAKE3-256 with domain framing. The reader fails closed
  on unsupported schemas, flags, compression, and absent executable capability.
- T4 module-world projection and loading exist. T4/T5 are code-implemented but
  have not passed behavioral, determinism, or source-free acceptance gates.

The type graph currently exposes a guarded `TypeExportContext` in
`phalcom-semantic/src/types/artifact.rs`. Its intended writer-side use is:

1. Construct exactly one context with `TypeExportContext::new(type_store)`.
2. Let every artifact section register its native type-related roots against
   that same context, using stable `CanonicalTypeRootKey` values and the
   context's expected `TypeStoreLineageId`.
3. Register all roots before projection. Registration after closure is an
   error.
4. Call `close_graph` exactly once. Closure traverses reachable graph nodes and
   freezes native-to-artifact mappings.
5. Emit `TypeGraphV1` with `project_graph`.
6. Use the frozen `ty`, `kind`, `parameter`, `row`, `family`, `variant`,
   `domain`, `constructor`, `scoped`, and `lambda` relocation methods while
   projecting later sections.

`TypeStore::project_artifact` remains a whole-store compatibility path. The
eventual T11 writer must not use it as a parallel source of artifact identity.
Writer composition uses the one shared, closed export context.

The import path returns a fresh `(TypeStore, TypeRelocation)`. Every consumer
must call `TypeRelocation::verify_store` against the receiving store before
using relocated IDs. A process-local `TypeStoreId`, native arena index, or
source allocation ordinal must never enter the payload.

## 4. Fixed identity and graph rules

These are architecture constraints, not local implementation preferences:

- Wire identity is artifact-local and deterministic.
- Stable outer-row sorting alone is insufficient. Allocation history must not
  influence emitted IDs or bytes.
- Root ordering is by stable root key. Artifact IDs are assigned by
  deterministic first encounter while traversing those roots and their edges.
- Graph closure happens before any section projection. A later section cannot
  discover a new type root after IDs have been emitted.
- Reachability, not whole-arena enumeration, determines the canonical writer
  graph. Unreachable historical arena entries are pruned.
- Reachable historical parameter versions are retained. Current parameter slot
  selection remains distinct from the historical versions.
- Parameter variance overrides and parameter ownership must survive relocation.
- Scoped-union order and repetition are semantic and must not be normalized as
  global-union members.
- Within-graph sharing must survive round-trip reconstruction.
- Builtins are deterministic traversal seeds, not permission to expose native
  arena order.
- Inference-local variables and source-only provenance do not become published
  artifact identities.
- Duplicate stable root keys that designate different native roots fail closed.
- A foreign store lineage fails immediately; it is never adapted or silently
  accepted.

If a task appears to require a second type-ID allocator, section-local type
maps, or stable identities derived from native ordinals, stop. That would
violate the P7 identity model.

## 5. T7.A must precede static ADT materialization

The apparent T6-before-T7 order cannot be implemented as a naive load order.
Existing validated static factories establish a real dependency:

- `StaticValueStore::intern_data` requires the corresponding `DataInfo`.
- `StaticValueStore::intern_variant` requires the corresponding `VariantInfo`.
- Both validate owner, type, component/field, and index correspondence.

Therefore load the structural subset of T7 before materializing T6 ADT values.
This does not change publication atomicity. Build mutable loader-owned state in
phases, validate it, and publish only the complete immutable image.

The required structural phase is:

1. Type graph and `TypeRelocation`.
2. Declaration shells and current generic-parameter selection.
3. Hierarchy, aliases, and normalized generic constraints that do not require
   a static value yet.
4. `DataInfo`, `EnumInfo`, `VariantInfo`, component contracts, constructors,
   declared component/field indexes, and result types.
5. Static-value graph through the existing `StaticValueStore` APIs.
6. Deferred structural references to static values, after static relocation is
   available.
7. Remaining semantic aggregates and checked compiler attachments.

The untracked `semantic/aggregates.rs` is only a shape sketch. It currently has
plain DTO structs but no schema marker, codec implementation, decode budgets,
referential validation, semantic projection, or loader integration. Its
`kind_tag: u8` and string selector/name fields must not become an accidental
unvalidated ABI. Map them to the canonical semantic enums/selector codecs used
by the repository.

If structural validation reveals a real cycle that the staged sequence cannot
resolve, use only the detailed plan's validated two-phase construction contract.
Do not bypass `intern_data`, `intern_variant`, or their equivalent owning-table
validation by inserting directly into arenas.

## 6. T6 static-value graph contract

T6 needs its own graph export/import contexts, but it consumes the already
closed `TypeExportContext` and must not allocate type IDs. Follow the same
lineage, stable-root, deterministic-closure, and relocation discipline as T5.

The portable graph must represent every canonical static-value family:

- unit, boolean, character, integer, and float;
- interned strings and symbols;
- type and static-term values;
- tuples;
- every supported sequence family, without merging distinct families;
- records with canonical field keys;
- data values with stable owner and constructor identities plus declared
  component indexes;
- variant values with stable variant identity plus declared field indexes;
- argument packs, preserving shared backing storage and slices.

Representation requirements:

- Preserve integer values exactly. If the native authority is arbitrary
  precision, decimal transport must be canonical and validated before parsing.
- Preserve floating-point bits exactly, including signed zero and NaN payloads.
- Validate character scalar values.
- Data/variant ownership uses stable portable identities, never native IDs.
- Data and variant components are checked against the structural products
  loaded in T7.A.
- Pack backing and pack slices remain separate DTO families so shared backing
  and slice boundaries survive relocation.
- Validate every artifact ID, range, length, and cross-table edge before
  indexing.
- Enforce `ValidationLimits` during decode and before materialization.
- Reject trailing bytes and unsupported schema values.
- Reconstruct through existing validated interning/factory APIs so ordinary
  canonicalization and invariants remain authoritative.

The current untracked `semantic/static_value.rs` is a useful starting shape. It
already distinguishes graph-local families, exact float bits, stable ADT owners,
declared indexes, and pack backing/slices. It is not accepted implementation:

- the module is not exported and has not compiled in its current form;
- projection and import contexts do not exist;
- referential-integrity validation is incomplete;
- canonical ordering and root registration are absent;
- native static-value family parity has not been checked exhaustively;
- factory-based reconstruction is absent;
- no section-level composition or round-trip evidence exists.

First make the DTO codec a coherent, bounded artifact-crate unit. Then add
semantic-owned projection/import beside `StaticValueStore`; do not put semantic
knowledge into `phalcom-compiler-artifact`.

## 7. T7.B compiler attachments are required

Do not mark compiler source attachments omitted merely to obtain a source-free
snapshot. Live compiler and evaluator paths still consume checked operational
information:

- `EffectiveCallableDefinition` retains accepted-definition provenance and
  generic body context.
- `StaticDefinitionProduct.checked_body` contains checked operations and facts,
  but its current evaluator root still reaches syntax.
- `StaticDefinitionSignature` retains a syntactic where-clause.
- `EvaluatorBody::root_work` and `StaticEvalMachine::run` consume those roots.

P7 requires source-independent checked operational templates and normalized
constraints. The wire form must retain the compiler/evaluator information
needed to execute already-checked definitions without retaining or recreating:

- raw `Expr` or `LexicalBlock` nodes;
- `TypeAnnotation` syntax;
- parsed modules;
- source text;
- a requirement to parse or type-check dependency source after load.

Keep diagnostics/presentation capability separate from formal compiler
authority. Stable logical source locations may remain where already accepted;
editor source indexes and raw syntax do not become required compiler payload.

**Mandatory stop:** if implementing a checked attachment still requires
parsing, rebinding, type-checking, or semantically reinterpreting dependency
syntax, stop T7.B and prepare an escalation packet. Do not ship empty attachment
tables, weaken evaluator invariants, or introduce a second checker in the
loader.

## 8. Complete loader order

The complete source-free loader should materialize authority in this order:

1. Validate container header, compatibility identity, section directory,
   capabilities, schemas, flags, compression, lengths, and digests.
2. Decode and validate `MODULE_WORLD` without publishing it.
3. Decode the type graph; create the fresh `TypeStore` and verified
   `TypeRelocation`.
4. Materialize declaration shells and current generic parameters.
5. Materialize hierarchy, aliases, and normalized constraints that are
   independent of static values.
6. Materialize data, enum, variant, constructor, component, and field structural
   products.
7. Decode and reconstruct the static-value graph through validated factories;
   produce static relocation.
8. Resolve deferred static references in structural and semantic products.
9. Materialize remaining primary semantic aggregates and source-independent
   checked compiler attachments.
10. Rebuild owning-table indexes and P6 lookup structures from primary rows.
11. Decode only eligible query roots and separate dependency evidence from
    duplicated semantic payload.
12. Decode existing RuntimePublic metadata and cross-check its identity/digests
    against the container and loaded worlds.
13. Freeze the semantic snapshot and canonical image.
14. Publish the immutable image only after all validation succeeds.

No partially loaded image may become observable. A loader error must leave no
published authority and must identify the artifact/section validation class
without falling back to dependency source.

## 9. Remaining task route

### P7.T0/T5: finish the shared projection seam

- Check that every eventual T6-T10 projection source can register its type roots
  before `TypeExportContext::close_graph`.
- Keep registration APIs explicit and lineage-checked.
- Add no second whole-store projection path to the writer.
- Compile only at the coherent seam named by the active execution plan.
- Determinism and allocation-history parity remain later evidence; do not claim
  them from a compiler check.

### P7.T7.A: implement structural primary contracts

- Replace the aggregate sketch with versioned, bounded wire DTOs.
- Project stable declaration/variant identities and type artifact IDs.
- Load through declaration, hierarchy, generic-signature, data, and enum owning
  tables.
- Represent static-dependent fields as validated deferred artifact references,
  then resolve them after T6.
- Do not publish temporary shells as a snapshot.

### P7.T6: implement static projection and relocation

- Add deterministic root registration, graph closure, projection, validation,
  factory reconstruction, and relocation.
- Cover all native families exhaustively.
- Preserve exact scalar representations and graph sharing.
- Compile T6 together with the structural T7.A seam because static ADT import
  depends on it.

### P7.T7.B: complete `SEMANTIC_WORLD`

- Project every inventory row classified as primary wire authority.
- Add source-independent checked operational templates and normalized
  constraints.
- Use stable owners and T5/T6 artifact IDs throughout.
- Materialize through semantic owning tables; the artifact crate remains a DTO
  and codec crate.
- Freeze one immutable semantic snapshot only after cross-family validation.

### P7.T8: reconstruct indexes

- Rebuild indexes through owning-table insertion/rebuild APIs.
- Do not persist caches as competing authority unless the plan explicitly
  classifies them as wire primary products.
- Compare P6 index/counter shape only at the named acceptance gate.

### P7.T9: restore the eligible query image

- Use the exhaustive query-kind disposition in
  `phalcom-semantic/src/artifact/manifest.rs` as a census, not as proof of a
  complete loader.
- Persist reusable roots and required dependency evidence.
- Omit parsed-module, source-dependent, open-world, and non-compiler products as
  classified by the accepted plan.
- Do not duplicate canonical semantic payload in query records.
- Do not treat current process-local `u64` query fingerprints as eternal wire
  ABI.

The untracked `query.rs` is a generated sketch until it has schema/version
integration, stable key encoding, bounded validation, semantic projection,
loader composition, and freeze-invariant checks.

### P7.T10: transport RuntimePublic metadata

- Reuse the existing RuntimePublic metadata authority rather than reconstructing
  a parallel public model.
- Cross-check package identity, compatibility, world/section digests, and
  runtime pool contracts.
- Keep runtime-public metadata distinct from compiler-private semantic payload.

The untracked `metadata.rs` is likewise a sketch until codec, validation,
cross-header checks, and loader/writer integration are complete.

### P7.T11: deterministic writer

- Gather roots from every section into the shared contexts before graph closure.
- Project in the detailed plan's canonical section and row order.
- Validate the complete payload before output.
- Write atomically; do not leave a valid-looking partial artifact.
- Report truthful build summaries and digests.
- Run determinism evidence only at the named gate.

### P7.T12: explicit loader and image injection

- Finish the staged load order in section 8.
- Return explicit loaded authority; do not install hidden global state.
- Inject the immutable image into compiler/LSP consumers through existing
  canonical image/session boundaries.
- Keep current-process provenance outside portable identity.
- Fail closed; do not silently rebuild missing dependency authority from source.

The untracked `phalcom-core/src/modules/canonical_artifact.rs` is not integrated
through `phalcom-core/src/modules/mod.rs` and is not evidence of T11/T12
completion.

### P7.T13-T15: acceptance and lifecycle

- T13 fresh-process source-unavailable proof and T14 broader parity,
  determinism, and performance certification remain deferred under the current
  compiler-only policy.
- Do not call P7 complete without those required gates or an explicit plan
  amendment.
- At actual plan completion, update the shared checkpoint and normal walkthrough
  and handoff records. Keep implementation completion distinct from verification
  strength.

## 10. Generated-patch adoption checklist

For each untracked generated source file, answer all of these before treating it
as implemented:

- Is the file exported by its parent module and compiled?
- Does the wire family have an explicit schema/version?
- Are all enum tags, reserved values, and unsupported versions rejected?
- Are collection, byte, depth, and allocation limits enforced before allocation?
- Are all artifact IDs range-checked before indexing?
- Are stable owner identities used instead of process-local IDs?
- Is canonical ordering defined independently of hash-map and arena order?
- Is projection owned by the semantic/runtime authority rather than the DTO
  crate?
- Is reconstruction routed through the owning table/store's validation APIs?
- Is cross-section validation explicit?
- Does malformed input fail closed without panic?
- Is the file composed into the actual writer and loader?
- Does the focused compiler command compile the new seam rather than leave the
  file unreachable?

A `Default` derive, DTO equality, or local encode/decode round trip does not
answer these questions.

## 11. Escalation boundaries

Stop implementation and follow the shared consultation protocol if any of these
occurs:

- Correct output appears to require changing canonical ownership, stable
  identity, store lineage, or publication lifetime.
- A section needs its own type identity source or a second semantic authority.
- Existing static factories cannot support staged structural loading without an
  invariant change.
- Compiler attachments cannot be made source-independent without changing the
  compiler/evaluator contract.
- A loader would need to parse, bind, infer, check, or execute dependency source
  to recreate accepted authority.
- The accepted task order, live source dependencies, and staged loader contract
  materially conflict.
- A proposed fix weakens a semantic invariant, drops a required product, or
  substitutes an empty table.
- Correctness in GC rooting, stable IDs, generic reification, or VM control flow
  becomes nonlocal or uncertain.
- The implementation expands materially beyond T6/T7 into an unexpected
  subsystem for architectural rather than mechanical reasons.

For an escalation, provide the exact task, invariant, call path, minimal
reproducer, scoped diff, observed facts, causal hypothesis, attempted correction,
trigger, and precise decision requested.

## 12. Verification policy

During BUILD, use the smallest compiler command that proves the edited seam is
actually reachable. Run Cargo commands serially.

Expected coherent compiler checks from the active execution plan include:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-compiler-artifact --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core -p phalcom-lsp --lib
```

Do not run all three reflexively after each edit. Select the owning crate at a
coherent seam. A check that leaves an untracked module unexported is not
evidence for that module.

Under the inherited policy, do not run tests, workspace builds, clippy, source-
free runtime experiments, determinism comparisons, parity suites, or performance
certification. Record them as deliberately deferred. If the user lifts that
restriction, follow the exact T6-T14 gates in the detailed plan rather than
inventing a new broad-test sequence.

Before every handoff:

```sh
git diff --check
git status --short
```

Review the scoped diff directly. Report precisely which successful checks ran
after the latest relevant edit, which generated files remain unreachable, and
which acceptance evidence remains deferred.

## 13. Immediate next action

Resume at the smallest coherent T7.A/T6 transport seam:

1. Preserve the current T0/T5 type-export edits and inspect their scoped diff;
   do not redesign them.
2. Export only `semantic::static_value` in `phalcom-compiler-artifact` and run the
   artifact-crate compiler check so the current DTO sketch receives real
   compiler feedback.
3. Correct only mechanical codec/module issues found by that check.
4. Compare the DTO families against the live `StaticValueStore` family enum and
   factory signatures; amend the DTO before writing semantic projection.
5. Make `semantic::aggregates` a versioned, bounded codec for the structural
   T7.A subset.
6. Implement semantic-owned T7.A projection/loading, including deferred static
   references, followed by T6 static projection/loading through validated
   factories.
7. Stop at the compiler-attachment boundary if source-independent operational
   templates require an architectural decision.

Do not begin T9-T12 composition while T7.A/T6/T7.B are incomplete. Later
generated sketches depend on stable relocation and semantic-world interfaces;
wiring them early would create parallel assumptions and rework.
