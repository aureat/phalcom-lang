# IMPLEMENTATION INCIDENT

> Status: the original ownership question was resolved by the user's
> publication-order decision. Its overlay-order and first-seen-ID hypotheses
> are superseded by the later root-resolution discriminator recorded in
> [the checkpoint](../CHECKPOINT.md). The source semantic input and exact
> metadata root were stable; a pool-0 placeholder in the runtime recipe was
> resolved against an unrelated loaded pool. ESI now uses the exact metadata
> root and its portable recipe publication has passed two-process byte
> determinism. This packet remains historical evidence.

## 1. Identity

- Program: PERF003
- Checkpoint: PERF003.C2
- Plan: PERF003.C2.P8
- Task/Gate: ESI cross-process determinism; handoff SUI-6 and ESI-G5/G8
- Repository: `aureat/phalcom-lang`
- Branch: `feat/artifact-universe-consolidation`
- Incident working-tree anchor: `0fc8f4ee51fc91da26c6186f249bd70131899dd1`, before the root-resolution and publication-determinism closure. Current status is in `../CHECKPOINT.md`.

## 2. Escalation trigger

Mandatory trigger 10: the same nontrivial determinism failure remains after
the approved semantic fingerprint correction. Any further canonicalization
change may also trigger 4 because it could change canonical identity.

## 3. Required invariant

Global unions are semantically unordered. Their persistent fingerprint is the
union tag, cardinality, and sorted multiset of stable child fingerprints;
fingerprints are not deduplicated. Scoped union order and repeated
contributions remain represented. `TypeStore::union` keeps its TypeId order
for store-local interning. ESI must consume semantic identity authority and
must not add a second canonical identity rule.

## 4. Expected architecture

The semantic metadata exporter owns canonical type fingerprints. ESI projects
emitter and runtime products into portable recipes, using metadata fingerprints
for `RuntimeTypeRef::Base`. Runtime overlay types are projected structurally.
The schema cut is metadata v14; semantic-model and executable ABIs remain
unchanged. P8's image must be byte-identical across independent builds before
the determinism gate can pass.

## 5. Observed behavior

Two independent invocations of the canonical capture test passed their
within-process repeat checks and structural round trips. Both produced the
same 4,999,535-byte inventory: 168 modules, 3,067 prototypes, 130,964
instructions, 28,115 constants, 3,067 semantic pools, 506 classes, 488 native
anchors, 72 primordial roles, and 149 detached methods. Neither reported
unresolved references or unsupported reachable products.

The payload digests differed:

```text
process 1: 0a722e2e610447e18c97bf0891b8d7d25322a514b08a41538b8329939e377fe5
process 2: 7f737d951c995dd4b9df164526f298ce1001e4f6d2def6fa17bf0c1bff569871
```

Module, prototype, constant, class, native-anchor, primordial-role,
source-map, detached-method, and bootstrap table fingerprints matched. Of the
17 semantic-pool families, 16 matched. Only type recipes differed:

```text
process 1: 6fab3c8de1f08642fde85b48da157038f7bdb76389008df2634571ec593d99f0
process 2: a164ef667384a92912b16bd5c328654687bab4a9caf0c487d8b3f755e672af57
```

The confirmed global-union `TypeId` ordering defect was corrected in the
semantic exporter and the focused allocation-order regression passes. This
repair did not close the cross-process ESI gate.

## 6. Minimal reproduction

Run the following command twice as separate processes:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  modules_executable_serialization::canonical_universe_executable_capture_structurally_round_trips \
  -- --exact --nocapture --test-threads=1
```

Observed output includes the two different payload digests above, while each
individual run passes. Expected output: byte-identical payloads and matching
table and semantic-family fingerprints across processes.

## 7. Relevant code path

```text
capture.rs::capture semantic-pool traversal
  -> artifact::semantic::project_semantic_pool
  -> SemanticPoolProjector::type_recipe
       RuntimeTypeRef::Base: read metadata structural_fingerprint
       RuntimeTypeRef::Overlay: project RuntimeOverlayTypeNode variants
         Union: visit members in overlay order and intern portable recipe
  -> PortableSemanticPoolV1::type_recipes
```

The owner-side correction is at
`phalcom-semantic/src/metadata/export.rs::MetadataExporter::export_type_form`,
the global `TypeNode::Union` fingerprint arm. Its scoped-union arm remains
order-sensitive. `phalcom-semantic/src/types/store.rs::TypeStore::union`
continues sorting by local `TypeId` for interning.

## 8. Scoped implementation diff

The approved fix sorts global-union child fingerprints at the semantic owner,
clarifies the store-local TypeId order, advances type metadata schema 13 to
14, and extends validation through schema 14. Tests cover allocation-order
independence, tuple order, scoped order and repetition, reordered import, and
stale v13 loader rejection. The normative Telosc text records the fingerprint
rule. No ESI-side identity normalization was added.

## 9. Evidence

- `cargo test -p phalcom-semantic --lib union_fingerprint_tests -- --nocapture --test-threads=1` — PASS, 3 tests.
- `cargo test -p phalcom-core --test core modules_canonical_artifact::loader_rejects_rehashed_incompatible_header_fields -- --exact --nocapture --test-threads=1` — PASS; the rehashed v13 header is rejected.
- The fresh-artifact source-denied compiler regression — PASS; loaded 341 declaration shells, 95 declaration surfaces, 314 callable signatures, and compiled the eight-module SlotMap project with source reads denied.
- Opcode manifest regression — PASS, 1 test.
- Executable image codec suite — PASS, 9 tests.
- Executable-capability staging rejection — PASS, 1 test.
- `cargo fmt --all -- --check` and `git diff --check` — PASS.
- The two cross-process capture invocations pass individually, but byte equality fails as described above.
- Workspace and release checks were not run. Full executable validation and VM materialization remain outside this slice.

## 10. Implementer's causal hypothesis

The remaining mismatch may originate in `RuntimeTypeRef::Overlay` projection.
In particular, `RuntimeOverlayTypeNode::Union` is converted by iterating its
members, and portable type recipe IDs are assigned by first-seen traversal.
That path may expose a store-local order or process-dependent encounter order
that the global metadata fingerprint fix does not cover. This has not been
confirmed.

## 11. Alternative hypotheses

- The set of recipes may be stable while `PortableTypeRecipeId` assignment
  changes with semantic-pool traversal order.
- A different overlay node form may contain a process-local ordering that
  affects only the type-recipe family.
- The remaining difference may come from a type representation whose order is
  semantically meaningful; sorting its recipe children would be incorrect.

## 12. Attempts already made

- Change: apply the user-approved semantic-owner rule for global union
  fingerprints and bump metadata schema to v14.
- Prediction: the divergent type recipes caused by global union fingerprints
  would converge and the whole payload would match across processes.
- Result: global-union allocation-order identity and import regressions pass,
  but the two payload digests still differ; only the type-recipe family hash
  differs among semantic families.
- No second correction was attempted. The remaining candidates cross the
  semantic identity and ESI representation boundary.

## 13. Why local authority is insufficient

Sorting `PortableTypeRecipeV1::Union` children in ESI could duplicate semantic
canonicalization. Preserving every runtime overlay member order could also
retain a process-local coordinate if the overlay represents a global
unordered union. The current evidence does not distinguish these cases or
establish whether the portable recipe IDs themselves are the source. That
choice affects canonical identity and a downstream wire contract.

## 14. Decision requested

For the remaining cross-process mismatch confined to type recipes, what owner
must define the canonical order: the runtime semantic overlay or ESI recipe
projection? Please specify the invariant that distinguishes global union
members from scoped or otherwise ordered type members, and whether portable
recipe numbering must be independent of first-seen traversal. Do not change
the existing metadata-v14 rule or scoped-union behavior.
