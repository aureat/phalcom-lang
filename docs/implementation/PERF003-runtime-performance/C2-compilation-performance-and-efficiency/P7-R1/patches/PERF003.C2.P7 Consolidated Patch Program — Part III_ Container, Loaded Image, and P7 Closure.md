
# 7. Patch Plan P7.PATCH.5 — Canonicalize and harden `.telosc` bytes and publication

## Objective

Make one semantic artifact correspond to one canonical P7 byte representation and make publication efficient without coupling correctness to an arbitrary runtime load budget.

The container audit found accepted trailing/gap bytes, unchecked uncompressed `decoded_len`, truncated unknown capability bits, noncanonical section layouts, and avoidable section-buffer cloning.

## 7.1 Enforce contiguous canonical layout

For current P7:

```text
header
directory
section 0
section 1
section 2
section 3
end-of-file
```

Require:

```text
first_section.offset == directory_end

for every adjacent pair:
    current.end == next.offset

last_section.end == data.len()
```

No unreferenced trailing bytes.

No interior gaps.

Future extension data must appear as explicit sections.

This repairs whole-artifact digest identity without requiring anonymous padding semantics.

## 7.2 Enforce canonical directory ordering

Require section tags in the same canonical ordering emitted by the builder.

Reject:

```text
duplicates
out-of-order required sections
wrong required flags
unexpected noncanonical flags
```

Unknown optional sections should have a formally specified extension rule before being accepted.

## 7.3 Validate uncompressed lengths

For the current no-compression profile:

```text
decoded_len == encoded_len
decoded_len <= max_decoded_section_bytes
encoded_len <= max_encoded_section_bytes
compression == None
```

Reject before checksum/materialization.

## 7.4 Reject unknown capability bits

Do not use truncating bitflag decoding for artifact capabilities.

Use checked conversion.

Conceptually:

```rust
let capabilities =
    TeloscCapabilities::from_bits(raw)
        .ok_or(ArtifactError::UnknownCapabilityBits(raw))?;
```

Then retain the P7 compiler-only capability-set check.

Do not accidentally make unknown future features appear absent.

## 7.5 Move section buffers into the builder

Capture metadata before transfer:

```rust
let semantic_len = semantic_world_bytes.len();
let semantic_digest = digest(&semantic_world_bytes);

builder.add_section(
    SectionKind::SemanticWorld,
    semantic_world_bytes,
);
```

Do not `.clone()` the entire four encoded sections merely to retain their lengths.

A later optimization may stream into the final container, but a move-only builder is the correct low-risk first repair.

## 7.6 Separate publication validation from runtime budget policy

Preserve the important invariant:

```text
validate the actual bytes before replacing the existing artifact
```

But do not hard-code:

```text
ValidationLimits::default()
```

and force the writer through exactly the same consumer materialization path used by runtime loading.

Define explicit layers:

```text
validate_container_bytes
validate_artifact_owner_graph
materialize_loaded_image
```

Writer publication should prove:

```text
bytes are canonical
checksums/fingerprints are correct
semantic/query/module owner relations are valid
artifact is loadable under the validation configuration requested by the build
```

Runtime load should additionally enforce its configured cumulative materialization budget.

Do not weaken semantic validation; separate it from incidental runtime-budget defaults.

## 7.7 Publication durability semantics

The current same-directory temporary write plus file sync and atomic rename is a sound base.

Decide the product contract:

### If `.telosc` is rebuildable cache material

Document:

```text
atomic replacement guaranteed
crash may lose latest cache entry
corrupt/missing entry is safely rebuildable
```

### If durable publication is required

Add parent-directory synchronization on supported platforms after rename.

Do not claim power-loss durability merely because the file itself was `sync_all()`ed.

## PATCH.5 tests

Required byte-level tests:

```text
append one trailing byte -> reject
interior rehashed gap -> reject
decoded_len = 0 -> reject
decoded_len = encoded_len + 1 -> reject
decoded_len = u64::MAX -> reject
unknown capability bit + valid digest -> reject
permuted required sections -> reject
wrong mandatory flag -> reject
writer bytes satisfy strict parser
moving section buffers leaves output byte-identical
previous artifact survives pre-persist failure
```

---

# 8. Patch Plan P7.PATCH.6 — Assemble one complete immutable loaded canonical parent

## Objective

Turn the now-complete imported semantic owners into the exact immutable parent model that P5/P6 workspace overlays already consume.

This patch is intentionally an **assembly and consumer-closure patch**, not another serialization patch.

## 8.1 Eliminate detached imported authority

After PATCH.1/PATCH.2, `ValidatedSemanticParts` should not contain meaningful compiler authority that never enters the snapshot.

The target loaded structure should conceptually be:

```text
ValidatedImportedSemanticImage
    type store
    canonical semantic snapshot
    canonical module products
    reconstructed compiler projection
    required owner indexes
    canonical query image inputs
```

Free-floating decoded products should exist only as temporary construction state.

Before return, they must either be:

```text
installed into their real owner
```

or:

```text
explicitly proven unnecessary and discarded
```

## 8.2 Populate every canonical parent family

A complete imported snapshot must contain exact canonical authority for all P7-required consumers, including at minimum the families repeatedly identified by the audits:

```text
declarations
hierarchy
dispatch
callable signatures
accepted callable definitions
callable analyses
field signatures
data semantics
enum semantics
trait headers
trait surfaces
associated families
conformance authority
witness plans
derived conformance rules
trait dispatch
static cases/signatures
static definitions
static constants
module static bindings
formal compiler projection
module products
compiler-required semantic indexes
```

No canonical required family may remain empty simply because an independent DTO was decoded.

## 8.3 Install module products explicitly

Do not assume that loading `CanonicalExternalModuleWorld` automatically fulfills snapshot `module_products`.

Build or connect:

```text
unlinked module products
linked module products
```

through the canonical owning relationship expected by the snapshot.

Avoid duplicate competing module authority.

## 8.4 Build formal projection from owner facts

The broad editor/source index remains absent.

The compiler-required formal projection must be rebuilt from validated artifact authority:

```text
stable callable
binding ordinal
explicit attachment classification
canonical source/formal attachment identity
module symbol where module-owned
```

Test lowering with an empty broad `source_index`.

Any canonical compiler reader that silently falls back to source syntax must fail a source-denial test.

## 8.5 Remove detached debug indexes

Objects keyed by:

```text
format!("{:?}", module)
```

must not be treated as canonical indexes.

Use typed module identity inside the actual owner/table representation.

Compiler reader complexity must stay module-bounded as intended by P6.

## 8.6 Preserve immutable parent overlay semantics

Do **not** copy canonical rows into every workspace.

Keep:

```text
with_parent(...)
parent maps
shared Arc products
canonical image references
```

for local sessions.

Add regression tests ensuring the artifact-loaded canonical image retains the pointer/sharing behavior already proven for the source-built canonical image.

## 8.7 Two-phase loaded-image promotion

Use:

```text
1. reconstruct
2. validate structural completeness
3. validate canonical Universe profile
4. validate query/semantic cross-fingerprints
5. validate compiler-reader closure
6. mark Complete
7. publish Arc<CanonicalUniverseFormalImage>
```

Never set `SnapshotStatus::Complete` first and hope downstream code proves it.

## 8.8 Source-denial is a semantic property

Introduce an explicit provider/harness mode where canonical Universe source access produces an immediate test failure.

Then exercise:

```text
artifact load
project analysis
package analysis
semantic lowering
backend preparation
```

No canonical syntax may be acquired.

User/package source remains available.

## PATCH.6 completion gate

A source-built canonical image and a loaded artifact image must be equivalent for compiler consumers, modulo expected omitted presentation/source-only data.

Required parity dimensions:

```text
accepted definition products
binding identities
expression read/write attachments
formal projections
match/pattern plans
iteration products
family/associated selections
trait dispatch
conformance/witness evidence
static evaluation authority
module-static bindings
query fingerprints
module-bounded readers
canonical parent sharing
```

---

# 9. Patch Plan P7.PATCH.7 — P7 closure, package acceptance, determinism, and performance certification

## Objective

Convert the implementation from “locally plausible” into a defensible P7 release state.

Current artifact tests cannot yet serve as closure evidence because important tests fail in fixture setup/artifact build before they reach the named loader/compiler behavior. The first audit explicitly warns that the existing inline `print(42)` case is insufficient for the intended package-level closure.

## 9.1 Make test-frontier truth explicit

Every end-to-end test should record or assert the deepest stage reached:

```text
ArtifactProjected
BytesBuilt
BytesValidated
Written
ColdLoaded
SemanticMaterialized
QueryReconstructed
CanonicalImagePublished
UserAnalyzed
UserLowered
ExecutableProduced
RuntimeExecuted
```

Do not accept:

```rust
assert!(load(...).is_err());
```

as evidence for a later malformed semantic condition when the artifact failed at an earlier export guard.

Negative tests must assert:

```text
typed error
expected frontier
no earlier unrelated failure
```

## 9.2 Canonical build/load test

Required sequence:

```text
source-build Universe
        ↓
build universe.telosc
        ↓
validate strict bytes
        ↓
write atomically
        ↓
terminate builder process
        ↓
fresh process
        ↓
load artifact
        ↓
publish complete canonical parent
```

No source-built image may leak across the process boundary.

## 9.3 Cross-process deterministic bytes

Build from:

```text
fresh type/static stores
different native allocation histories
reversed insertion orders where semantics are unchanged
unreachable historical values
```

Require exact equality of:

```text
MODULE_WORLD digest
SEMANTIC_WORLD digest
QUERY_IMAGE digest
METADATA digest
whole artifact bytes
```

This specifically addresses root-ordering risks tied to native identifier allocation.

## 9.4 Source-denied novel project test

Create a project not used to build the Universe.

Requirements:

```text
explicit Universe import
multiple user modules
generic call
enum use
trait/conformance use
flow-sensitive binding
match/pattern
static operation
```

Run in a fresh process where canonical source access aborts.

Require successful compile through the ordinary loaded-image path.

## 9.5 Standalone package acceptance

Use real package entry paths, not only `Inline`.

Required cases should exercise:

```text
parse
slotmap
similarity
```

The first audit notes a concrete fixture discrepancy: `packages/similarity/project.toml` declares `similarity.main`, while the inspected source layout did not contain the corresponding `main.ph`.

Resolve this as a **fixture/package-layout decision**, not by weakening artifact semantics.

If `similarity` is intentionally a library, create a thin executable consuming fixture.

Do not test a project-style package under the analyzer's different flat `EntrySelection::Package` convention and misclassify the resulting layout failure as an artifact problem.

## 9.6 Existing-runtime execution

P7 does not need a P8 portable executable Universe section.

It does need to prove that compiler outputs produced with the artifact-backed Universe can continue through the **existing runtime pipeline**.

At least one package smoke test must reach runtime execution.

This proves:

```text
compiler source-free with respect to Universe
```

without claiming:

```text
runtime itself was restored from a P8 executable artifact
```

## 9.7 Zero canonical work telemetry

Record counters:

```text
Universe source files opened          = 0
Universe parse invocations            = 0
Universe interfaces rebuilt           = 0
Universe link modules rebuilt         = 0
Universe callable analyses performed  = 0
Universe static definitions checked   = 0
Universe constants evaluated          = 0
Universe witness plans rebuilt        = 0
Universe presentation rebuilt         = 0
```

Any nonzero canonical semantic rebuild invalidates the source-free P7 claim.

## 9.8 Allocation certification

For the complete artifact load:

```text
counting allocator observed requests
==
AllocationBudget committed requested bytes
```

Record:

```text
whole-load requested allocation
retained canonical allocation
peak RSS
```

These are different metrics.

Also test:

```text
exact N
N - 1
aggregate limit where each individual stage would fit
failure rollback
bad artifact followed by corrected artifact retry
```

## 9.9 Performance certification

Measure independently:

```text
Universe source build time
artifact serialization time
artifact publication time
cold artifact load time
first user overlay compile
warm repeated overlay compile
workspace test/corpus time
loaded-image retained memory
peak artifact-build memory
peak artifact-load memory
```

Compare:

```text
source-built Universe path
vs
persisted-image Universe path
```

Do not publish an aspirational latency threshold as achieved data.

## 9.10 P5/P6 non-regression

Verify:

```text
canonical parent tables remain shared
user-local edits do not copy/rebuild Universe
module-bounded iteration stays module-bounded
local user semantic changes do not invalidate canonical products
workspace revisions preserve parent identity
```

## 9.11 Writer/concurrency tests

Exercise:

```text
existing valid artifact
failed replacement
successful replacement
concurrent reader during replacement
repeated writes
parallel independent reads
```

Readers must observe:

```text
complete old image
or
complete new image
```

never partially written bytes.

## 9.12 Repository/build hygiene

Before final P7 closure:

```text
remove generated rustc ICE artifact from production tree
pin/document toolchain required for reproducible verification
confirm clean build with appropriate incremental setting
reconcile the perf branch against the intended integration/consolidation base
rerun final gates after the final merge/rebase
```

Historical test records must not be quoted as evidence for a later source revision.

---

# 10. Dependency graph and execution order

The eight patches should not be implemented simply in numeric serial order. Use these waves:

```text
WAVE 1
    PATCH.0  contract/fingerprint/validation truth
    PATCH.5  isolated parser/container hardening
    PATCH.3  typed allocation errors + currently reachable module accounting

WAVE 2
    PATCH.1  full callable owner graph
    PATCH.2  remaining semantic/static/conformance owners
    PATCH.3  frozen imported storage + TypeStore + semantic accounting

WAVE 3
    PATCH.4  query reconstruction/fingerprint closure
    PATCH.6  complete canonical parent assembly and source-free readers

WAVE 4
    remove remaining family guards
    remove allocation UnaccountedBoundary barriers
    PATCH.7 full closure matrix
```

The important concurrency rule is:

```text
schema work and allocation design may proceed in parallel,
but no imported native owner representation should be finalized
without an allocation-compatible storage plan.
```

Otherwise P7 will repeatedly serialize a correct owner into a representation that cannot legally be materialized under its own load contract.

---

# 11. Guard-removal ledger

Maintain this as an explicit checklist in the implementation records.

| Guard / restriction | May be removed only after |
|---|---|
| Callable-analysis export rejection | PATCH.1 full source/import/consumer parity |
| `ensure_complete_callable_analysis_payload` | PATCH.1 importer and lowering parity |
| Static-family export rejection | PATCH.2 real static owners and evaluator/compiler parity |
| Trait/witness rejection clauses | PATCH.2 complete proof/member transport |
| Derived conformance rejection | PATCH.2 primary rules restored and dispatch parity |
| `SemanticDenotation::StaticApplication` rejection | Checked-body owner + application resolver complete |
| Semantic validation `UnaccountedBoundary` | PATCH.3 validator exact-accounting proof |
| Semantic import `UnaccountedBoundary` | PATCH.3 whole native materialization accounting proof |
| Query unsupported-owner restrictions | PATCH.4 exact loaded owner adapter |
| `SnapshotStatus != Complete` | PATCH.6 structural completeness + canonical profile |
| P7 lifecycle `IN_PROGRESS` | PATCH.7 complete acceptance/performance matrix |

---

# 12. What must explicitly **not** be done

The implementer should reject any shortcut resembling:

```text
delete the coverage guard
weaken has_source_free_artifact_contents
fill missing owner fields with Default
replace missing type knowledge with TypeId(0)
treat source range as canonical source-site identity
derive accepted callable definition from signature
copy checked static applications into CallableAnalysis
rerun Universe static checking during load
rerun Universe witness planning during user compilation
preserve query evidence in bytes but ignore it after decoding
estimate native map allocations and call them exact
convert allocation errors to Strings inside the budget boundary
flatten the canonical image into every workspace
declare package closure from print(42)
claim freshness/incremental artifact rebuild as P7 work
```

Every one of those either appears directly in the audits as an existing defect/risk or recreates the architectural problem that produced the current guards.

---

# 13. Final P7 closure definition

P7 is ready to close only when the following statement is true:

> A clean source build can compile the canonical Universe once and deterministically produce a canonical compiler-only `universe.telosc`. A fresh process can validate and load that artifact under one exact cumulative allocation transaction, reconstruct semantically equivalent immutable canonical compiler authority without opening Universe source, reconstruct the required canonical query/dependency image, and use that parent to compile and run genuinely new projects and standalone packages through ordinary Telos compiler/runtime paths. Canonical work remains shared and immutable across user overlays, and measured cold/warm behavior demonstrates the intended performance benefit.

That is the correct boundary between:

```text
P7 — persisted canonical compiler authority
P8 — portable executable image
P9 — freshness, discovery, incremental artifact caching
```

and it provides a strong foundation for both later phases rather than leaving them to compensate for semantic debt in the P7 artifact.