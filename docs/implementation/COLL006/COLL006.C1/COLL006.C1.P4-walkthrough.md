---
id: COLL006.C1.P4
program: COLL006
checkpoint: COLL006.C1
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
---

# COLL006.C1.P4 — Static homogeneous sequences and bracket-default cutover

## Outcome and architecture

P4 implemented canonical static homogeneous Array/Vector values, recursive
descriptors, native VM materialization, const Array freezing, explicit List
migration, and the semantic runtime Array/static Vector bare-bracket defaults.
The continuation started at committed `d4fe4473` after T10/G6 and completed
T11/G7/T12 on `feat/telos-collection-upgrade`.

`StaticValueData::Sequence` retains the checked result type and ordered static
children in the existing C4 store; equality, hash, fingerprint, and type
projection consume that canonical product. The static evaluator consumes
`SequenceLiteralResolution` rather than choosing a family. Exact symbolic
identity spreads remain symbolic until closed. A reduced sequence projects to
the self-contained `MaterializationDescriptor::Sequence`. The VM recursively
materializes native Vector or frozen ordinary Array storage through the
canonical `StaticConstantId`/`RuntimeStaticConstantRef` source owner and caches
one value per constant. Intermediate heap children are temporary roots until
owner publication; failed construction does not publish a partial cache value.

Direct const Array writes are diagnosed by semantic binding provenance, while
runtime alias writes reach the frozen Array guard. The parser stays neutral;
the compiler fails closed for headless/Vector brackets without checked lowering.
TDR-0091 and the indexed sequence-literal specification supersede the legacy
bare-List wording in TDR-0025/TDR-0028. Existing List-dependent source is
explicit. The T12 migration census records 505 classified bare nodes with zero
ambiguous or unclassified entries.

## T11 continuation

`phalcom-semantic/tests/semantic/incremental/coll006_c1_p3_publication.rs`
now exercises static Array content edits, annotation removal, Vector extent
changes, `const`/`let` phase transitions, invalid-to-valid-to-invalid
publication, and Vector spread source loss of exactness. It compares cold and
incremental semantic results and diagnostics, rejects stale values/descriptors,
checks changed content fingerprints, and verifies unrelated constant reuse.

`phalcom-core/tests/core/collections/vector.rs` adds a checked explicit Vector
fixture proving ordered, exactly-once evaluation and the zero-extent case.
`phalcom-core/tests/core/modules/parity.rs` now adds a second consumer of the
re-exported constants and asserts it shares the source owner. Existing module,
static-constant VM unit, and Array tests cover canonical ownership, forced GC,
frozen aliases, ordinary Array mutation, runtime source order, and the
constructor/static-List boundaries.

## Verification performed

All commands below ran with `RUSTFLAGS='' RUSTC_WRAPPER=''`:

| Command/filter | Result |
|---|---|
| `cargo test -p phalcom-semantic --test semantic coll006` | 51 passed |
| `cargo test -p phalcom-semantic --test semantic static_sequence` | 10 passed |
| `cargo test -p phalcom-semantic --test semantic incremental::coll006_c1_p3_publication` | 3 passed |
| `cargo test -p phalcom-semantic --test semantic static_constants` | 21 passed, 3 inherited D4 failures |
| `cargo test -p phalcom-core --test core core_array` | 11 passed |
| `cargo test -p phalcom-core --test core core_vector` | 9 passed |
| `cargo test -p phalcom-core --test core static_const` | 23 passed |
| `cargo test -p phalcom-core --test core modules_parity` | 8 passed |
| `cargo run -p phalcom-native-surface-gen -- --root . --check` | 417 declarations current |
| `cargo check --workspace --all-targets` | passed |
| `git diff --check` | passed |

The broad `incremental` filter ran once, exposed many failures outside COLL006,
and timed out. It is not reported as a pass; the exact COLL006 incremental
filter is the focused acceptance evidence. `cargo fmt --all -- --check` reports
inherited formatting drift in pre-existing files, so no global formatting pass
is claimed. Full workspace tests, clippy, and release certification were
deliberately deferred per the P4 gate.

G7's `static_constants` filter failed only the same three computed Record-key
`RecordKeyNotStatic` cases previously recorded as D4. The unchanged failures
remain recorded as D4 in the checkpoint and are outside P4's sequence behavior.
There are no unresolved P4-owned A/B failures. P4 is `COMPLETE /
IMPLEMENTED / BASELINE_BLOCKED` because the named aggregate filter cannot be
reported as a clean pass. There was no new architectural consultation in T11;
the prior P4 symbolic-spread, empty-static-element, and inherited-G2 decisions
remain recorded in the checkpoint and plan amendments.

GC between descriptor-child allocations and controlled allocator failure
cannot be injected with current test hooks; the existing temporary-root and
fallible-reservation paths were inspected and nested post-materialization GC
and failed-publication tests passed. P5 inherits the stable interfaces listed
in the [P4 handoff](COLL006.C1.P4-handoff.md).

## Bounded correctness follow-up and closure — 2026-10-10

The follow-up closed the inference-constraint, publication/incremental,
materialization, and checked-count concerns identified in the P4 analysis. The
checker now treats inference registration as pending owner work, checks
concrete spread compatibility through the canonical subtype relation, and
fails closed if the expected query-local inference owner is missing. Fixed
construction counts now use `usize`; compiler compact-bytecode narrowing is
checked, with the existing builder path retained.

New regression coverage proves that partial Vector spread inference solves or
rejects without leaking invalid sequence products, late generic owner rejection
discards only the dependent sequence, multi-hop extent edits invalidate stale
values and agree with cold analysis, and descriptor conversion handles 128
nested sequences. Runtime coverage proves nested static Arrays stay frozen
through aliases and survive forced GC. Failed materialization restores root
depth and leaves no cache entry before a successful retry. The production
`intern_sequence` caller still consumes checked, committed semantic resolution.

Follow-up focused evidence: semantic `coll006` 54 passed; `static_sequence` 10;
the multi-hop and deep-descriptor tests passed; core Vector 9, Array 11,
`static_const` 24, and `modules_parity` 8 passed; failed-materialization retry
passed; targeted module-owner and nested-Array tests passed under
`PHALCOM_GC_STRESS=1`; workspace all-target `cargo check` passed; native surface
check reported 417 declarations current. Semantic `static_constants` remains
21 passed with the same three inherited D4 computed-Record-key
`RecordKeyNotStatic` failures. The exact results and residual classification
are in the [C1 checkpoint](COLL006.C1-CHECKPOINT.md).

No architecture or plan amendment was needed. P4 remains `COMPLETE /
IMPLEMENTED / BASELINE_BLOCKED`; P5 remains next.
