---
id: COLL006.C1.P3
program: COLL006
checkpoint: COLL006.C1
kind: handoff
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
---

# COLL006.C1.P3 continuation handoff

## Start here

P3 is incomplete. This is a user-requested stop/commit/push checkpoint, with the runtime test command aborted during compilation.

- Branch: `feat/telos-collection-upgrade` (singular), based on P2 `5fee70c9`.
- Worktree: `.worktrees/telos-collections-upgrade`.
- Delivery revision: the commit containing this handoff; retrieve with `git log -1`.
- Preserve other worktrees/main work. Work **inline without subagents**, per the user's latest instruction.
- Must read: `AGENTS.md`, implementation-record convention, this handoff, P3 walkthrough, P3 implementation plan, and the supplied P3 requirements analysis. The checkpoint is the single state authority.

First command after status/diff inspection:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_vector
```

It has never completed. Avoid treating the current runtime/source integration as established merely because source checking compiled.

## Established focused evidence

P1/P2 remain completed prerequisites. The final completed COLL006 semantic filter selected **22 tests, all passed**, including seven new P3 tests. Exact static generic formation, signed huge integer identity, symbolic binders, partial holes, and all eight positive Vector literal/context combinations have evidence.

`cargo check -p phalcom-core --all-targets` passed before the final runtime test file was added. The later runtime test invocation was aborted while compiling core. The latest patch is not runtime-certified.

Exact-integer prerequisite metadata/scalar/reflection evidence is in the walkthrough. Broader metadata and reflection failures were classified as suspected unrelated by bounded inspection; they were not compared against a clean predecessor.

## Interface map

| Concern | Current owner / interface |
| --- | --- |
| Exact scalar | `phalcom-type-meta/src/scalar.rs`: `StaticInt::Small(i64)` / `Large(Arc<BigInt>)`; semantics re-exports the same type |
| Canonical integer term | `TypeData::ConstInt(StaticInt)`, `TypeStore::const_int(impl Into<StaticInt>)`; small callers remain valid |
| Durable format | Metadata schema 12; exact decimal serialization, legacy numeric decoding |
| Static value generic argument | `types/annotation.rs::resolve_static_generic_value`; current support is integer literals and unary-negative integer literals |
| Generic holes | `checker/annotation_elaboration.rs`; parameter kinds are required, missing slot fails closed |
| Source domain kind | `resolve_kind_syntax` intrinsic Int bootstrap and contextual `static_domain_kind_of_form` |
| Active sequence checker | `checker/sequence_literal.rs::synthesize`, called by `expression.rs::synthesize_bracket_literal` |
| Old checker | `legacy_synthesize_bracket_literal` remains inactive in `expression.rs`; remove after regression validation |
| Semantic construction | `SequenceLiteralResolution` plus `exact_expansions: BTreeMap<SourceRange, SequenceExpansionResolution>` |
| Exact spread lanes | `TuplePositionals { extent: TypeId }` / `VectorStorage { extent: TypeId }` |
| Compiler projection | `SequenceLiteralLoweringSpec`: canonical extent/lane plus `expected_extent: Option<u32>` and exact count map |
| Vector runtime | `heap/vector.rs::VectorObject`, immutable boxed Value slots; no mutable accessor |
| Identity/bootstrap | `UniverseKey::Vector`, `CoreDeclarationIds.vector`, `CoreClasses.vector_class`, provider `collections.vector`, source package expose |
| Read floor/source | `primitive/vector.rs` rawLength/rawAt; `collections/vector.ph` typed reads, Iterator/Iterable, equality/hash/render/conversions |
| Bytecode | `BuildVector(u16)`, `BeginVectorLiteral`, `VectorExpandExactPositionals { vector, expected: u32 }`, `FinishVectorLiteral(u32)` |
| Private builder | Existing `ArrayLiteralBuilderObject` Vec reused privately; `take_elements` moves storage into Array/Vector via in-place heap replacement |
| Native artifact | Generator completed with 406 primitive records; run artifact `--check` after authored signature changes |

## Invariants to inherit

- Vector remains an ordinary heterogeneous canonical applied type, with exact Int N.
- Holes stay solver-local. No Dynamic fallback or unsolved durable variables.
- Exact cardinality and the runtime expansion lane describe the same storage operation.
- General Iterable, List, and Array cardinality is runtime-unknown for Vector inference.
- Vector is a separate immutable native object. The builder is private, with traced initialized values.
- Compiler requires checked Vector resolution; context-free explicit Vector heads fail closed.
- Bare runtime brackets remain List during P3. Const sequence materialization and default cutover belong to P4.
- Mutable descendants do not automatically acquire deep hash stability; global Map/Set policy has not been redesigned.

## Remaining tasks, in execution order

1. **Runtime/bootstrap discriminator.** Finish `core_vector`. Diagnose source/constructor/native/semantic boundary failures from the actual result. Also run `core_array` because builder finalization and shared literal checking changed.
2. **Formation correctness.** Add the central nonnegative concrete N validation. Add wrong-kind/arity tests, nested/aliased and scoped generic application cases. Shared argument elaboration currently covers fixed canonical applications and partial annotations; pack-prefix/scoped/static-evaluation paths need review where P3 requires them.
3. **Symbolic construction contract.** Semantic identity can retain N and a lone exact symbolic spread, but lowering rejects nonconcrete counts. Decide and implement the bounded proof-preserving symbolic equality path using canonical facilities, or record an explicit authorized limitation. Do not add Vector-only reification or a second arithmetic solver.
4. **Publication and diagnostics.** Cover extent mismatch, empty underconstraint, element mismatch, runtime-unknown spreads, family conflict, and no checked product after failed inference/checks. Outer expected-type constraints are applied after synthesis; verify this cannot leave invalid executable products. Fix explanations that still label bracket literals List.
5. **P2 corrective work.** `array.ph` still has Dynamic/untyped element APIs. Correct homogeneous public replacement/read signatures with sound bounds behavior. Array move-finalization is implemented but needs regression evidence, including frozen guard and failure cleanup.
6. **Runtime invariants.** Add actual disassembly assertions (one current test only creates a Bytecode value), native/checked extent inconsistency tests, no mutation/resize, zero/large dimension boundaries, cross-family inequality, stable nested Vector keys, iterator owner GC, nested GC, evaluation/spread allocation and throw/suspend cases where supported.
7. **Metadata and incremental gates.** Prove Vector parameter domains and exact N identity across metadata/import/export, reflection, cold analysis, and incremental edits that change N or exactness.
8. **Cleanup/closure.** Remove inactive legacy synthesis. Finish scoped formatting/lint/native artifact validation, workspace all-targets check at the named gate, and final lifecycle records. Do not broaden into unrelated baseline repair.

## Commands and gate order

Run Cargo commands serially:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_vector
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_array
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic coll006
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::exact_static_int
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_domain_inference
RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-native-surface-gen -- --root . --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets
```

Add the directly relevant metadata/incremental/GC selectors after implementing their coverage. Confirm selected test counts. Broad release tests and clippy certification belong at the specified checkpoint gate, not after every edit.

## Record status

P3: `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`, paused by request. P3 must remain partial until runtime, negative typing/cardinality, durability/incremental, and corrective acceptance requirements are satisfied.

The supplied requirements analysis retains its original Markdown hard-break trailing spaces. Later Vector edits have not completed formatting/lint checks. Document actual delivery checks separately from behavior verification.
