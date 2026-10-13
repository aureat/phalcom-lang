---
id: PERF003.C2.P2
checkpoint: PERF003.C2
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# PERF003.C2.P2 handoff

Continue in `.worktrees/perf-universe-characterization`, branch
`perf/universe-characterization`. Current increments are committed at
`bf1a69c6` (code) and `c931faea` (records).
`CHECKPOINT.md` owns checkpoint-wide status and evidence.

Stable takeover APIs:

- `canonical_universe_formal_image()` returns the shared complete formal image.
- `CanonicalUniverseFormalImage::{snapshot,modules,query_image,reader,world}`.
- `CanonicalSemanticQueryImage::reader` validates exact world and final type/static prefixes.
- `SemanticDb::with_canonical_image(workspace, image)` installs an immutable parent
  without query copying. Use image final arenas or validated extensions in consumers.
- `SemanticDb::{product,product_origin,ready_product_fingerprint,is_current}`
  read both domains; `query_state`/`local_query_state` remain strictly local.
- `CallableSignatureTable::{get_conformance_witness,iter_module}` avoid global scans.

Next implementation: ordinary sessions need explicit image-backed base aggregates,
final TypeStore and StaticValueStore forks, and an immutable external module catalog.
Keep local publication lifecycle initially empty. Do not store the canonical snapshot
in `last_snapshot` or `last_known_good`, and do not seed canonical queries into local
rows. Existing source discovery still promotes Universe modules into mutable local
sources; cut that over before routing ordinary sessions through immutable query reuse.
A source-driven canonical recomputation with different inputs must not overwrite an
immutable key to bypass the guard.

Do not reuse negative/local candidate proofs from the image. Preserve observed
absence/candidate-set dependencies and incremental conformance coherence. Failed,
blocked and cyclic query rows are not eligible roots. Provider overrides cannot
consult a builtin semantic world or foundation arena with incompatible IDs.

User explicitly requested continued implementation and deferred broad verification.
Do not restart baseline investigations. Focused evidence and deferred gates are in
the walkthrough/checkpoint; no complete checkpoint/release claim is justified.
