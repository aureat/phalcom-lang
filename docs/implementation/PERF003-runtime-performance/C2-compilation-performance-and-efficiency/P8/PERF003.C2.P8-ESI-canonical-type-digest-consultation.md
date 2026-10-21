# IMPLEMENTATION INCIDENT

> Status: superseded by the root-resolution discriminator recorded in
> `../CHECKPOINT.md`. The two source semantic inputs and their exact metadata
> root fingerprints matched; the lowering recipe's pool-0 coordinate resolved
> to an unrelated runtime pool. ESI now reads the data construction's own
> metadata root, without changing metadata-v14 identity. Subsequent two-process
> capture passed with identical payload bytes. No semantic fingerprint change
> is indicated by this incident.

## 1. Identity

- Program: PERF003
- Checkpoint: PERF003.C2
- Plan: PERF003.C2.P8
- Task/Gate: ESI cross-process determinism; ESI-G5/G8
- Repository: `aureat/phalcom-lang`
- Branch: `feat/artifact-universe-consolidation`
- Incident working-tree anchor: `0fc8f4ee51fc91da26c6186f249bd70131899dd1`, before the root-resolution and publication-determinism closure. Current status is in `../CHECKPOINT.md`.

## 2. Escalation trigger

Mandatory trigger 4: correcting the newly isolated mismatch would change or
rely on canonical semantic type identity. Trigger 16 also applies because the
approved ESI publication decision and the live canonical fingerprint evidence
do not yet explain one another.

## 3. Required invariant

The user-approved decision establishes that runtime typing/semantic identity
owns constructor order semantics, ESI owns deterministic wire publication, and
artifact-local type recipe IDs must be independent of traversal. Only proper
global unions may have their recipe children sorted at publication. Scoped
union behavior and all established ordered constructors remain unchanged.

The P8 image must carry stable canonical semantic identities. ESI must not
substitute discovery coordinates or introduce a second semantic identity rule.

## 4. Expected architecture

`phalcom-semantic` metadata export owns stable `TypeNode` fingerprints.
`phalcom-core` runtime typing resolves base runtime references to those
fingerprints; ESI projects them into `Canonical` recipes and assigns portable
table IDs. ESI's approved union-order and post-capture numbering fixes are
publication operations and cannot repair a different canonical fingerprint
for the same semantic input.

## 5. Observed behavior

Two separate invocations of the canonical capture test each passed their
within-process repeated-capture check and structural round-trip. Both images
were 4,999,535 bytes and had the same inventory: 168 modules, 3,067
prototypes, 130,964 instructions, 28,115 constants, 3,067 semantic pools,
506 classes, 488 native anchors, 72 primordial roles, and 149 detached
methods.

The separate-process payload digests differed:

```text
process A: 81cfe8b992a0f0404352d416ada57e9bfe3adc2538ba358283f6d938a7c9f8e5
process B: 9a06ca78a67531f67e1850d66e5509bf0f6eb696bd27147e92905d5bed0df34c
```

All top-level table fingerprints except `semantic_pools` matched. Of 17
semantic-pool families, 16 matched; only `type_recipes` differed:

```text
process A: cd81aa8c61748f356cbc01bd44dbad5a79a75d2ae2d2346b084df37f0965cd9d
process B: e3558967f9e68e774e2f8a182b6fd466d9f20822e8596808d0ff4f3c4cdd5007
```

One recursive diagnostic pass replaced recipe and kind references with
recursive keys, sorted children only for `Union`, and compared each pool's
recipe-key multiset. It found differences in 164 pools after normalization.
The canonical-digest multisets differed in all 164 pools, while noncanonical
recipe-shape multisets matched. This is case C from the approved decision, not
table numbering alone or union-child permutations.

The first mismatch is `semantic_pools[407].type_recipes[0]`, a `Canonical`
recipe used by an `Interval` data-construction template in nested callable
`closed(_,_)` (prototype parent 396, nested ordinal 10). Its two portable
digests are:

```text
process A: eff6060365d3fb325f8ef302437ef7b9ef56c2f053152020829fb11568808762
process B: 615ddf0865eb326dc80d22af9fb9b0f08803759a274768105ada71b8c0cb9632
```

The executable image does not retain the source `TypeNode` form or its
metadata pool/node coordinates, so the underlying type constructor is not yet
identified. The temporary diagnostic harness used to establish this
classification has been removed.

## 6. Minimal reproduction

Run this exact test in two separate processes:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  modules_executable_serialization::canonical_universe_executable_capture_structurally_round_trips \
  -- --exact --nocapture --test-threads=1
```

Each invocation passes, but the logged executable payload digests differ as
shown above. Expected behavior: byte-identical payloads across processes.

## 7. Relevant code path

```text
capture.rs::capture_closure
  -> artifact::semantic::project_semantic_pool
  -> SemanticPoolProjector::data_construction
  -> runtime_type_recipe
  -> type_recipe(RuntimeTypeRef::Base)
  -> typing_registry loaded bundle TypeNodeEntry::structural_fingerprint
  -> stable_type_digest
  -> PortableTypeRecipeV1::Canonical
```

`MetadataExporter::export_type_form` in
`phalcom-semantic/src/metadata/export.rs` owns the producing type fingerprint.
The other `Canonical` insertion path, `SemanticPoolProjector::semantic_type_id`,
is not used by this first differing `DataConstruction` runtime-type recipe.

## 8. Scoped implementation diff

The earlier user-approved change already sorts the stable child fingerprints
of global metadata unions, preserves scoped-union order and repetition,
clarifies that `TypeStore::union` uses local TypeId order only for interning,
and advances type metadata schema v13 to v14. Its focused regressions pass.

The current ESI projector still preserves overlay union storage order and
assigns portable type recipe IDs during first-seen interning. Those authorized
publication changes have not been attempted because this diagnostic found a
canonical-digest mismatch that survives recursive ID normalization and union
sorting.

## 9. Evidence

- The canonical capture test selected one real test and passed separately in
  each process; its within-process repeated payloads matched.
- Both process payloads had equal byte length and equal executable inventory.
- The top-level table fingerprints matched except semantic pools; semantic
  family fingerprints matched except type recipes.
- The recursive diagnostic found 164 affected semantic pools after replacing
  recipe/kind references with recursively derived keys and normalizing only
  union children.
- Canonical digest multisets differed in those 164 pools. Noncanonical recipe
  shapes matched in all pools.
- The first differing canonical recipe belongs to the `Interval`
  data-construction template in nested callable `closed(_,_)`.
- The diagnostic was performed on temporary decoded payloads and its test-only
  instrumentation was removed afterward.
- No ESI union sorting, recipe renumbering, semantic fingerprint correction,
  workspace test, or release gate was performed in this turn.

## 10. Implementer's causal hypothesis

The mismatch is upstream of ESI table ordering: the loaded metadata bundle
supplies different `structural_fingerprint` values for a base runtime type
that is projected into the same `Interval` construction recipe. The metadata
`TypeNode` form and the source of its process sensitivity are not present in
the executable image, so this hypothesis does not yet identify the exact
semantic defect.

## 11. Alternative hypotheses

- A type constructor other than the already corrected global union has an
  unstable or incomplete metadata fingerprint.
- The same type is being resolved to semantically different canonical nodes
  across process builds, indicating nondeterministic semantic construction
  rather than a fingerprint algorithm defect.
- Runtime typing resolves the `Interval` construction recipe to different
  metadata pool/node owners across processes, even though its portable
  canonical recipe shape is otherwise the same.

## 12. Attempts already made

- The user-approved global-union fingerprint rule and metadata v14 migration
  were implemented before this turn. Its semantic allocation-order,
  order-sensitive control, scoped-union, import, and stale-schema tests pass.
- A cross-process capture comparison still differed after that change.
- The requested recursive diagnostic normalized IDs and global union order.
  It confirmed that the remaining differences are canonical digest values,
  not recipe discovery order or union member permutations.
- No correction was attempted after that classification because it would
  change or rely on semantic canonical identity without identifying the
  unstable source `TypeNode`.

## 13. Why local authority is insufficient

Changing the fingerprint at the metadata exporter may alter persistent
semantic identity and require a metadata schema change. Sorting or replacing
the `Canonical` recipe digest in ESI would instead duplicate semantic
identity. The remaining evidence establishes the failing owner path but not
which source type form, identity invariant, or compatibility version needs
correction. The user-approved ESI decision explicitly directs a stop for this
case C result when the differing semantic shape is uncertain.

## 14. Decision requested

Please identify the source metadata type form and owning invariant behind the
two `structural_fingerprint` values for the `Interval` data-construction
template in `closed(_,_)`. Should the correction live in the semantic metadata
fingerprint producer or in runtime type resolution, and what regression and
schema-version change are required? Keep the approved global-union v14 rule,
scoped-union contract, and ESI publication/ID ownership split intact unless
the evidence requires a specific amendment.
