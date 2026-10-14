# IMPLEMENTATION INCIDENT

## 1. Identity

- Program / checkpoint / plan: LANG007 / LANG007.C4 / LANG007.C4.P7
- Task / gate: T12 / G12
- Repository / branch: `aureat/phalcom-lang` / `main`
- Starting revision: `dbcdcd13`; observed continuation revision: `cb8a068c` plus scoped working-tree changes.

## 2. Escalation trigger

Repository RED triggers 8, 11, 15, and 16: removing the arena-ID hash from
the published symbolic constant fingerprint requires a stable type/value
identity and a change to the incremental product/evaluator-store boundary.
The plan §21.3 explicitly instructs STOP rather than inventing durability.

## 3. Required invariant

P7-I9, I10, and I14: store-qualified IDs are session-local; semantic product
identity must be independent of allocation history, and cold/incremental
results must agree. A failed or incomplete evaluation cannot enter the
successful product cache.

## 4. Expected architecture

P7 T12 hashes a normalized residual by stable semantic content. The P7
INC-003 architect ruling explicitly limits `StaticResidualStore::content_fingerprint`
to a store lineage; it identifies the incremental symbolic-constant hash as
a T12/P8 placeholder. The P7 audit calls for T12 closure. The proposed P8
plan §3.6 owns stable cross-session/artifact fingerprints.

## 5. Observed behavior

`db/fingerprint.rs::static_constant_product_fingerprint` hashes the store
ID and residual ID for `Symbolic`. It receives only the constant product and
`StaticValueStore`, not the `StaticResidualStore` or the canonical `TypeStore`.
The product itself contains a `StaticEvalOutcome::Symbolic(StaticResidualRef)`
but no owning residual DAG. Substituting the local residual content hash would
still hash raw `TypeId` and `StaticValueRef` from the source lineage.

## 6. Minimal reproduction

Read the `Symbolic` arm at `phalcom-semantic/src/db/fingerprint.rs:3087` and
the publication call in `phalcom-semantic/src/db/query.rs:1400`. Constructing
the same normalized residual in independently created stores changes the
store-qualified ID. Expected: a published semantic fingerprint is stable or
publication of that symbolic outcome is deferred; observed: the hash includes
the transient store ID.

## 7. Relevant code path

```
StaticEvaluator -> StaticEvalOutcome::Symbolic(StaticResidualRef)
  -> StaticConstantProduct (no residual store)
  -> query_static_constant(product, value_store, ...)
  -> static_constant_product_fingerprint(product, value_store)
  -> raw residual store/index hash -> published DB product
```

## 8. Scoped implementation diff

No changes to query, fingerprint, or static-constant ownership. Focused P7
continuation affects the evaluator's fallback and residual growth charging.

## 9. Evidence

- `StaticResidualStore::content_fingerprint` documents lineage-local meaning.
- `StaticConstantProduct` carries no residual store.
- `query_static_constant` accepts symbolic outcomes for publication; it only
  excludes cancellation and budget terminals up front.
- The accepted static-evaluation spec requires closed constants to reduce.

## 10. Implementer's causal hypothesis

The current product cannot hash or preserve a symbolic residual's semantic
contents at the DB publication boundary. Safe fixes are either a P7 policy
that does not publish session-local symbolic constants as reusable products,
or an explicit stable-identity projection and residual-store ownership path.

## 11. Alternative hypotheses

- Pass the residual store and a stable structural type/value hasher into the
  product query, and give the product a lifetime/ownership path for its DAG.
- Fail closed / defer symbolic constant publication and retain only reduced
  constants in P7, leaving cross-session residual products to P8.

## 12. Attempts already made

None: substituting the existing content hash would falsely claim durable
identity, and fail-closing publication changes incremental/query semantics.

## 13. Why local authority is insufficient

Both alternatives change product lifetime, stable semantic identity,
invalidation, and/or the P7/P8 contract. The proposed P8 plan's ownership
boundary needs to agree with the P7 T12 acceptance gate.

## 14. Decision requested

Should P7 T12 reject/defer publication of `Symbolic` static-constant products
until P8 supplies stable residual identity, or must P7 extend the product/query
interface now to carry and fingerprint a canonical residual DAG with stable
type/value meanings? Specify how the accepted cold/incremental G12 gate treats
currently non-reduced closed constants.

## User ruling — 2026-10-13

Implement stable cross-session symbolic fingerprints in P7. This supersedes
the defer-publication option: a store-lineage-local hash is insufficient at
the P7 incremental product boundary. P8 still owns artifact encoding and
cross-module transport. The exact owner/lifetime and stable type/value
projection must be established before editing publication; the current
`StaticConstantProduct` does not retain the residual DAG, and the metadata
type export's project identity includes the source revision. Neither arena
hashes nor replacing them with raw `TypeId`/`StaticValueRef` hashes satisfies
the ruling.

## Follow-up prerequisite after `origin/main` refresh — 2026-10-13

The fetched and pulled `origin/main` remains at `cb8a068c`; only an unrelated
performance branch advanced. The user selected revision-independent in-process
project coordinates while preserving revisioned serialized identities. Further
source inspection found `MetadataExporter::export_type_form` still routes every
Record field through `legacy_record_field_name` and rejects non-Symbol
`FieldKey`s and non-legacy anonymous key domains (`metadata/export.rs:1621–1663`).
`StaticValueStore::content_fingerprint` also hashes raw Type IDs in type-bearing
nodes. Therefore P7 cannot simply reuse the existing metadata Type fingerprint
for all accepted C4 structural residual inputs. Stable full-family hashing
requires extending the canonical type export representation for generalized
Record keys (currently assigned to P8), as well as owning the residual store
on published symbolic products. These are cross-plan representation and
product-lifetime changes; no source edits to those boundaries have been made.

**Follow-up decision required:** authorize a P7 prerequisite that moves the
minimum generalized Record/type-export support from P8 into P7 before T12,
or revise the T12/P8 gate without claiming stable fingerprints for residuals
whose type/value operands the current exporter cannot represent. The latter
does not satisfy the requested full-family P7 hash.

## User follow-up ruling — 2026-10-13

Move the minimum generalized Record/type-export representation prerequisite
from P8 into P7. It must remain owned by the existing metadata/type export
authority; do not invent a second TypeStore fingerprint walker. Preserve
versioned artifact compatibility while adding the in-process
revision-independent project identity mode requested above. P8 retains the
rest of its schema and cross-module transport work.
