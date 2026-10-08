---
id: COLL006.C1.P3
program: COLL006
checkpoint: COLL006.C1
kind: walkthrough
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
---

# COLL006.C1.P3 partial implementation walkthrough

## Outcome and delivery boundary

P3 is **not complete**. The user requested a stop, record update, commit, and push during the initial Vector runtime verification command. This walkthrough records a resumable work-in-progress patch.

Starting revision: `5fee70c9` (completed P2), following P1 `fb41492d`.

Delivery branch: `feat/telos-collection-upgrade`, in `.worktrees/telos-collections-upgrade`. The original plural branch `feat/telos-collections-upgrade` remains at the P2 anchor.

The supplied `COLL006.C1.P3-requirements-analysis.md` defines the bounded P3 scope. It was already staged as an empty added file with its content present in the working tree at takeover; this delivery includes its full supplied content.

The main checkout and other worktrees were left outside this work unit. No further feature implementation is authorized by this stop request. Subsequent work requires resuming P3.

## Implemented slices

### Exact static integer prerequisite

- Introduced the shared exact scalar owner `phalcom_type_meta::StaticInt`, re-exported through semantic scalar APIs.
- Replaced machine-width payloads in `TypeData::ConstInt`, type-member arguments, and metadata with the shared normalized scalar.
- Preserved immediate small integers and arbitrary-precision large integers, including mathematical equality/hash for non-normalized manually constructed variants.
- Updated metadata schema to 12, exact decimal serialization with legacy numeric reading, fingerprints, canonical interning, and reflection regressions.
- Converted large type-member indexes only at concrete indexing boundaries; huge diagnostic values retain their exact identity.

Primary owners: `phalcom-type-meta/src/scalar.rs`, `phalcom-semantic/src/types/store.rs`, metadata export/import consumers, and type-member projection.

### Heterogeneous source generic formation

- Complete fixed applications accept value-shaped generic arguments in their destination static domain.
- Partial annotation elaboration uses the same static integer value helper instead of using ordinary expression type `Int` as the static argument `3`.
- Removed the missing-parameter-kind fallback to Type in partial elaboration; missing argument slots produce diagnostics.
- Recognized the intrinsic Int domain during resolver-free kind/bootstrap formation.
- Corrected contextual domain classification through `static_domain_kind_of_form`.
- Added `KindSpec::StaticDomain` and Vector native generic metadata as a source-signature projection.

The current value helper handles integer literals and unary-negative integer literals. Symbolic binders resolve through existing type-level binding identities. General checked static expression evaluation and broader application paths remain follow-up work.

Primary owners: `types/annotation.rs`, `checker/annotation_elaboration.rs`, `declarations.rs`, and `phalcom-native-meta/src/types.rs`.

### Vector semantic slice

- Registered source-owned `Vector<T, N: Int>` in Universe identities, modules, package exposure, and core semantic IDs.
- Introduced `checker/sequence_literal.rs` as the active neutral literal checker.
- Supported all eight explicit/contextual concrete-and-hole combinations in the positive semantic matrix.
- Published canonical result type, element type, extent, construction strategy, and exact expansion lanes.
- Added closed tuple positional and native Vector storage cardinality evidence, excluding tuple labels.
- Rejected runtime-unknown Vector spread cardinality in the initial implementation.
- Included exact expansion evidence in semantic fingerprints.
- Preserved bare brackets as List during P3.

The old synthesis function remains in `expression.rs` as an inactive `legacy_synthesize_bracket_literal` with `allow(dead_code)`. Cleanup is incomplete. Family-correct explanations are also pending.

### Initial runtime and lowering slice

- Added distinct immutable `VectorObject` dense storage, a heap arm, read-only accessors, precise element tracing, class mapping, class-tower registration, and debug rendering.
- Added native read floor `rawLength` and `rawAt`. Invalid native reads fail rather than returning an absence sentinel as T.
- Authored Vector source reads, iterator, same-family equality, structural hash, rendering, and copying conversions to Array/List.
- Regenerated the native surface artifact with 406 primitive records.
- Extended `SequenceLiteralLoweringSpec` with canonical extent, exact lane evidence, and checked concrete count projections.
- Added `BuildVector`, `BeginVectorLiteral`, `VectorExpandExactPositionals`, and `FinishVectorLiteral` bytecodes.
- Used a private dense builder, never a public List/Array staging value. Finalization moves its payload and replaces the private heap object in place.
- Corrected Array builder finalization to move storage instead of cloning it.
- Context-free compilation of an explicit two-argument Vector head fails with missing checked lowering metadata.

These runtime changes have compile evidence only. The new runtime tests have no completed result.

## Tests added

- Exact scalar normalization/equality/hash and metadata compatibility tests.
- Semantic exact static integer formation, metadata round-trip, and indexing-boundary regressions.
- Core exact-integer reflection regression.
- Seven P3 semantic tests: concrete static term, distinct integer identity, static binder kind, mixed partial holes, symbolic binder identity, signed huge arguments, and eight equivalent Vector literal/context forms.
- Four initial core Vector tests: fixed/exact spread identity and order, context-free fail-closed compilation, API/equality/hash/conversions, and projected extent/representation bounds.

The last runtime test is preliminary: its bytecode variable is not a disassembly assertion. Replace it with an assertion over generated instructions before claiming direct-allocation coverage.

The first three static-generic probes deliberately inspect annotation identity with incompatible integer initializers. They are formation probes and do not assert that those complete fixture programs are valid. The later four positive tests assert absence of error diagnostics.

## Executed verification

All Cargo commands below used `RUSTFLAGS='' RUSTC_WRAPPER=''`.

| Command | Completed result | Scope |
| --- | --- | --- |
| `cargo test -p phalcom-type-meta --tests` | 25 passed | Exact-integer prerequisite: 20 schema and 5 scalar tests |
| `cargo test -p phalcom-semantic --test semantic foundations::exact_static_int` | 7 passed | Exact-integer prerequisite |
| `cargo test -p phalcom-semantic --test semantic foundations::static_domain_inference` | 6 passed | Exact-integer prerequisite |
| `cargo test -p phalcom-semantic --test semantic foundations::type_members` | 12 passed | Exact-integer prerequisite |
| `cargo test -p phalcom-semantic --test semantic foundations::static_values::` | 1 passed | Exact-integer prerequisite |
| `cargo test -p phalcom-semantic --test semantic integration::metadata` | 32 passed, 1 failed | Residual broader metadata failure; no passing broad gate |
| `cargo test -p phalcom-core --test core reflection::exact_static_integer_metadata_reflection_preserves_signed_values_and_identity` | 1 passed | Exact-integer prerequisite |
| `cargo test -p phalcom-core --test core reflection` | 25 passed, 4 failed | Residual broader reflection failures |
| `cargo check --workspace --all-targets` | Passed | Exact-integer prerequisite, before later Vector edits |
| `cargo check -p phalcom-semantic -p phalcom-type-meta` | Passed | Inline takeover validation |
| `cargo test -p phalcom-semantic --test semantic foundations::coll006_c1_p3_static_generics` | 3 passed at initial formation gate | Binder/application corrections |
| `cargo test -p phalcom-semantic --test semantic coll006` | **22 passed** at final semantic gate | P1/P2 plus 7 new P3 semantic tests |
| `cargo check -p phalcom-core --all-targets` | Passed | Initial Vector runtime/lowering slice, before adding the final runtime test file |
| `cargo run -p phalcom-native-surface-gen -- --root .` | Generated 406 records | Generation completed; separate `--check` gate pending |
| `cargo test -p phalcom-core --test core core_vector` | **INCOMPLETE** | Aborted by user during compilation; no runtime assertions executed |

The initial three static-generic probes failed before the formation corrections. The new Vector identity matrix failed while Vector literal synthesis remained reserved. Two existing List/Array spread tests failed after the checker extraction and passed after restoring the canonical sequence element projection. Those failures were active corrections, not deferred baseline issues.

Formatting of the exact-integer prerequisite passed scoped checking earlier. A workspace format check reported existing formatting outside that slice. The later Vector additions have not completed scoped formatting/lint validation.

## Remaining acceptance gaps

1. Run and diagnose the incomplete runtime suite; source Universe bootstrap/native registration and execution are unverified.
2. Centralize negative concrete Vector extent validation across formation, inference materialization, substitution, and imported metadata publication as applicable.
3. Resolve the gap between symbolic exact extent evidence and current concrete-u32 bytecode projection. No new runtime generic reification is established.
4. Cover empty typed/underconstrained vectors, extent mismatch, wrong element/domain, family conflict, invalid-product nonpublication, exact/runtime-unknown spreads, and contextual generic/return/nested scenarios.
5. Complete proof-preserving expansion verification, defensive mismatched-cardinality tests, GC during evaluation/spread, iterator lifetime, exception cleanup, equality/hash/Map/Set laws, and deep mutable-content policy observations.
6. Correct Array public homogeneous reads/replacement typing; `array.ph` still uses Dynamic/untyped mutation parameters.
7. Remove inactive legacy literal synthesis, fix family explanations, and verify solve-before-publication and causal-invalidity behavior at outer expected checks.
8. Complete cross-module metadata and cold/incremental tests for Vector and heterogeneous static applications.
9. Finish scoped formatting/native artifact checks and named integration/build gates; review diff and update final records.

## Classification and handoff

Plan: `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.

Checkpoint and program remain partial. Focused semantic evidence does not certify runtime Vector behavior. Residual broad failures are described in checkpoint §14 and have no clean-predecessor confirmation.

P4 remains the owner of canonical static sequences, const Array/Vector materialization, bare-bracket default migration, and the decision/spec cutover. Resume P3 before proceeding to P4.
