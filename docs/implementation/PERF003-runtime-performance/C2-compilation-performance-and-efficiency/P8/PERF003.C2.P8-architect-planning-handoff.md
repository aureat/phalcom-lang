---
id: PERF003.C2.P8
kind: architect-handoff
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
date: 2026-10-21
branch: feat/artifact-universe-consolidation
---

# PERF003.C2.P8 architect-planning handoff

## Request

Plan the remaining work needed to close P8 against the existing checkpoint
objective: publish a complete portable executable Universe artifact that a
fresh process can load, use to compile new user code, and execute without
Universe parsing, semantic analysis, lowering, or bytecode compilation.
Initializer execution remains per VM. Keep the checkpoint objective; decide
whether the remaining work fits the current P8 plan or needs one or more
follow-up plans within `PERF003.C2`.

This is a planning handoff. It does not authorize runtime probes, benchmarks,
workspace tests, or release certification. The current P8 plan explicitly
defers those gates pending policy authorization.

## Current state

- Checkpoint: `PERF003.C2`, `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.
- Plan: `PERF003.C2.P8`, `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.
- Branch: `feat/artifact-universe-consolidation`; the delivery commit is the
  source and record state for this handoff. The implementation was tested as
  the dirty tree based on `0fc8f4ee51fc91da26c6186f249bd70131899dd1`.
- P7's loaded compiler image and independent certification are accepted. P8.G1
  is PASS: the loaded compiler interface is present and the targeted compiler
  check passed.
- Executable capture and deterministic publication are focused-tested. The
  image remains structural staging data; the container capability
  `PORTABLE_EXECUTABLE` remains clear.
- Source-free executable loading, validation, VM materialization and runtime
  execution have not been implemented or verified. P8 is not complete.

The shared source of truth is the P8 section and plan ledger in
`../CHECKPOINT.md`. This handoff and the partial walkthrough add context; they
do not replace checkpoint state.

## Implemented findings and evidence

1. `phalcom-compiler-artifact/src/executable/` defines executable compatibility,
   instruction, constant, module, prototype, semantic-pool and image DTOs, plus
   a bounded codec and staging wrapper. The 128-opcode manifest assigns stable
   wire tags and records operand domains, relocations and future validation
   obligations.
2. `phalcom-core/src/artifact/capture.rs` observes the existing compiler and
   emitter in a disposable source-built VM. It captures canonical module
   initializers before initializer execution, nested/detached callables,
   constants, semantic pools, classes, native anchors, primordial roles and
   bootstrap disposition. It does not persist build-VM handles or initialized
   globals.
3. The persistence census classifies 128/128 bytecodes, 55/55 heap object
   variants, 22/22 constant recipe tags, all 8 callable fields, all 7 chunk
   fields and all 17 semantic-pool vector families. Unsupported constant
   products fail with typed capture errors. Structural decode is not executable
   semantic validation.
4. The prior `Interval` cross-process mismatch was investigated under the
   user's ruling not to change fingerprints based on an expected generic
   `Applied` form. The observed semantic inputs and exact metadata root
   fingerprints matched (`6eee82a7f40850c3dd262ed8161bc02f`). A lowering
   recipe's `Base(pool 0, node 2)` coordinate resolved through runtime pool 0
   to the unrelated `CannotYieldAcrossNativeFrame` root. The projector now
   derives the canonical recipe from that construction's attached metadata
   bundle root and checks that the root matches. No metadata-v15 or executable
   schema change resulted.
5. After the root-resolution fix, portable type/kind publication recursively
   orders stable recipe keys and rewrites IDs and references after projection.
   Only ordinary global `Union` members are sorted; union cardinality and
   repeats are retained, scoped-union order is retained, and ordered
   constructors such as tuples and applied arguments are unchanged. These are
   publication rules, not a second semantic identity owner.
6. Two independent canonical capture processes passed the selected structural
   round-trip test and produced identical 4,999,601-byte payloads with digest
   `c2af943406376eae636979f1223ea58895db3265a2d43dcf48c5da759267efea`.
   This proves capture-payload determinism for that discriminator. It does not
   prove full artifact publication or execution.
7. Capture returns `StagedExecutableSectionV1`. Container staging writes
   executable section tag 7/schema 1 only while `PORTABLE_EXECUTABLE` remains
   clear; the builder rejects the capability because no validator grants that
   authority.

Focused evidence recorded in the checkpoint:

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib`: PASS.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact --test executable_image_codec_test`: PASS, 9 tests.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib artifact::semantic:: -- --nocapture`: PASS, 4 tests.
- Focused canonical capture integration selector: PASS, one test per fresh
  process; identical payload bytes and digest as above.
- Focused executable staging-capability rejection: PASS, 1 test.
- Focused opcode-manifest test: PASS, 1 test; formatting and diff checks PASS.

These checks do not include an executable validator, a loaded executable
authority, runtime materialization, or a source-denied VM execution test.

## Live gaps and blockers

1. Validation authority is absent. The tree has no private `ValidatedExecutable`
   owner or core executable validator. Opcode operand/control-flow rules,
   cross-section references, module coverage, native ABI compatibility,
   semantic recipes, class layout, linked reads and bootstrap constraints are
   represented as future obligations, not fully checked before execution.
2. Runtime materialization is absent. There is no executable relocation path that
   allocates fresh classes, modules, metadata, callables, methods and closures
   in a VM, roots in-progress objects, publishes only after success, and leaves
   caches empty. Two-VM independence and within-VM sharing are therefore
   unproven.
3. An artifact-backed VM entry point is absent. The live tree has no
   `VM::try_new_with_universe_artifact` API. `VM` bootstrap still calls
   `canonical_universe_program()` and compiles canonical modules through the
   source-backed path (`vm/bootstrap.rs`). The capture helper itself also
   intentionally builds from source.
4. A later-import prototype path is absent. The executable image records modules,
   but runtime import has no verified stored-prototype path that forbids an
   AST-to-bytecode fallback for non-eager canonical modules.
5. The build/load/CLI flow is incomplete. Staging exists, but validation does not
   authorize `PORTABLE_EXECUTABLE`; there is no complete atomic compiler-plus-
   executable publication and no explicit CLI run path that loads one
   immutable artifact for both user compilation and VM construction.
6. Acceptance gaps: P8.G2 (malformed/incompatible validation), G3
   (source/artifact parity), G4 (fresh-process source-denied eager and later
   imports with zero Universe compiler work), G5 (two-VM state/rooting), and G7
   remain unrun/deferred. G6 is partial: capture-byte determinism passed;
   failed-load retry and separate startup measurements did not run.
7. Runtime-test authorization is still restricted. The continuation
   authorization covered focused semantic-projection unit tests and the
   two-process capture discriminator. The P8 plan says not to run runtime
   probes, benchmarks, workspace suites or release gates. The request to plan
   and deliver this handoff does not lift that restriction. The architect
   should lay out the gates and state what explicit authorization is needed
   before execution.

Known baseline context: P7's broad semantic integration lane was recorded as
blocked by 412 stale API fixtures reproduced at clean predecessor
`08e2b2f5`; its runtime suite recorded 20/20 `RUNTIME_ERROR` rows with empty
details. These are historical P7 baseline findings, not evidence that P8 code
fails. Do not repair them unless a P8 acceptance dependency requires it.

## Fixed architecture and invariants

- The existing source compiler and emitter produce executable prototypes;
  do not create a second compiler or semantic checker.
- The build VM is disposable. No `ObjRef`, `ClassId`, initialized globals,
  captured runtime values, runtime metadata-pool coordinates or mutable caches
  become persistent identities.
- P7's loaded artifact remains semantic/compiler authority. Runtime metadata
  and type identities are consumed through their existing canonical owners.
- ESI owns portable representation and stable wire publication. It must not
  reconstruct semantic identity from storage order, discovery coordinates or
  an ESI-only fingerprint.
- The v14 global-union fingerprint decision stands: sorted multiset of stable
  child fingerprints with tag/cardinality and repeats preserved; scoped union
  order remains. No v15 change is authorized by the old `Interval` incident.
- Preserve ordered constructors, instruction/constant/field/parameter order,
  module initialization order and failure behavior.
- Validate compatibility and the full executable graph before initializer
  execution or artifact-specific side effects. Explicit artifact mode must not
  fall back to Universe source compilation or provider access.
- Every VM owns fresh mutable runtime state. Preserve canonical class identity
  and sharing within that VM; share no relocation or mutable runtime cache
  across VMs. Use existing rooting and failure-atomicity owners.

## Architect decisions requested

1. Decide whether to amend the current P8 plan or create a successor plan under
   the same `PERF003.C2` checkpoint. Keep the checkpoint objective and retain
   the current partial implementation as a predecessor.
2. Reconcile T1/T2 acceptance against the live DTO/capture/census and define a
   dependency-ordered remaining sequence for validation, materialization,
   native floor, VM bootstrap, later imports and CLI publication.
3. Specify the owner and capability boundary for transitioning from staged
   executable bytes to a private validated executable artifact. State exactly
   which checks happen before any runtime allocation or initializer side effect.
4. Specify the per-VM relocation/rooting and publication/failure-atomicity
   contracts. Identify any architecture question that needs consultation
   before implementation rather than leaving it to a code-level guess.
5. For each gate G2 through G7, specify a test and the result required to pass
   it. Separate payload determinism from full artifact determinism and
   source-free compiler use
   from source-free VM execution. Include failure-before-initializer and retry
   behavior, later imports, two-VM independence, within-VM sharing and zero
   Universe compiler counters.
6. State the runtime-test authorization needed to execute the deferred gates.
   Keep workspace/release certification separate unless the user authorizes
   it or an integration dependency requires it.

## Must-read context

- `../CHECKPOINT.md`: canonical checkpoint state, architecture and evidence.
- Plan file `PERF003.C2.P8-portable-executable-universe-detailed-implementation-plan.md`:
  existing fixed contracts, tasks, gates and restrictions.
- `PERF003.C2.P8-executable-persistence-census.md`: field-level current
  persistence boundary.
- `PERF003.C2.P8-ESI-publication-determinism-walkthrough.md`: closed slice
  evidence.
- `../../../../specs/compiler-artifacts/telosc.md`: normative artifact
  schema and staging capability boundary.
- `phalcom-core/src/artifact/{capture,semantic,canonicalize,project}.rs` and
  `phalcom-core/src/vm/bootstrap.rs`: capture, semantic projection and live
  source-backed bootstrap.
- `phalcom-compiler-artifact/src/executable/` and
  `phalcom-compiler-artifact/src/container.rs`: DTOs, codec and staging gate.
- Existing VM heap/rooting and module registry owners reached by T4/T5; inspect
  those narrowly while planning materialization.

## Recommended planning sequence

First verify each gap against the committed tree and update the task map if a
symbol moved. Then produce a patch-grade plan that establishes the validator
boundary before any materialization work, sequences GC-safe relocation before
native/bootstrap integration, and leaves user-facing CLI publication until a
validated VM path exists. End the plan with an explicit acceptance matrix and
test-authorization boundary. Do not infer execution evidence from the current
capture tests.
