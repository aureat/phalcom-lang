---
id: ARTF001.C1.P3
category: ARTF
program: ARTF001
checkpoint: ARTF001.C1
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - ARTF001.C1.P2.R4
follows: ARTF001.C1.P2.R4
supersedes: PERF003.C2.P8
---

# ARTF001.C1.P3 — Portable Executable Universe and Source-Free VM Bootstrap

## 0. Executor contract

This plan is written for GPT-5.6 Luna High/Extra High implementation under principal-architect supervision. It converts the retired `PERF003.C2.P8` executable-Universe work into the current `ARTF001.C1` artifact architecture and incorporates the repository audit performed against `feat/artifact-universe-consolidation` at `aad6a88db904de04059ee9027240ab168b136c5c`.

The implementer:

- may adapt mechanical details to the live repository after re-reading the named paths;
- must preserve the ownership, identity, validation, allocation, materialization, and bootstrap architecture below;
- must not improvise a new executable schema, semantic ontology, runtime identity model, loader transaction model, or module-initialization state machine;
- must not weaken P2's exact allocation-accounting contract;
- must not convert source fallback into a recovery mechanism for an explicit executable artifact path;
- must not treat structurally decoded executable bytes as runtime authority;
- must not preallocate source-declared behavior classes whose allocation is part of bytecode execution;
- must follow the task-local testing budget rather than reflexively running broad suites;
- must classify unrelated failures rather than "fixing the world";
- must STOP AND CONSULT whenever a fixed architectural premise is false or a schema/authority boundary would need to change;
- must keep `docs/implementation/ARTF001-artifact-and-universe-consolidation/C1/CHECKPOINT.md` truthful after durable gates;
- must produce the required walkthrough and handoff before plan closure.

Testing is evidence gathering, not ritual. A compile-only check proves compilation, not runtime behavior. A test existing proves nothing until it is run. `NOT RUN` is never converted into `PASS`.

---

## 1. Goal

Produce one validated full `universe.telosc` that contains the canonical compiler image plus a portable executable image and can, in a fresh source-denied process, both compile novel user code and construct fresh independent VMs—including eager and deferred canonical Universe initialization—without canonical Universe source access, canonical re-analysis, canonical relinking, or AST-to-bytecode recompilation.

---

## 2. Checkpoint acceptance objective

`ARTF001.C1` owns consolidation of the current-main semantic/compiler artifact architecture with the retired PERF003 portable Universe work. `ARTF001.C1.P3` is **one of several checkpoint plans**, not full checkpoint closure.

P3 specifically closes the executable-runtime half that P2 intentionally does not own:

```text
P2 compiler authority
    CanonicalUniverseFormalImage
    current semantic/query products
    runtime-public metadata
    exact artifact import/publication
            ↓
P3 executable authority
    portable bytecode + executable side tables
    validation against P2 identity
    per-VM materialization
    native floor
    eager/deferred initialization
            ↓
P4+
    later canonical artifact compiler/source-cutover/generalization work
```

P3 must not begin authoritative runtime publication until P2 has an accepted closure receipt. Existing executable groundwork inherited from retired P8 may be audited, repaired, and completed under this plan once that entry condition is satisfied; its presence does not retroactively close P2.

P3 is therefore a **runtime portability, soundness, and source-free bootstrap plan**.

---

## 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: feat/artifact-universe-consolidation
revision: aad6a88db904de04059ee9027240ab168b136c5c
relevant predecessor:
  - ARTF001.C1.P2.R4
  - retired PERF003.C2.P8
```

Repository facts verified during planning:

1. `phalcom-compiler-artifact` already owns executable wire DTOs, `ExecutableImageV1`, executable compatibility identities, structural image validation, staging, `SectionKind::Executable`, and `PORTABLE_EXECUTABLE`.
2. Executable staging is explicitly non-authoritative: `StagedExecutableSectionV1` can encode a structurally valid image but is not an execution authorization token.
3. `phalcom-core/src/artifact/validate.rs` already owns extensive executable validation groundwork: structural/operand checks, control-flow handling, stack analysis, non-local-return activation validation, native compatibility checks, and related proof code.
4. The branch already contains a private `ValidatedExecutableImage` seam, but publication/materialization/runtime consumption are incomplete.
5. Capture already compiles and captures every canonical module. `VM::run_universe_modules` executes only `canonical.bootstrap_order()`; when capture is active, non-eager modules are additionally compiled and captured without executing their initializers.
6. The live compiler can place ordinary `Bytecode::Method` after behavior-class values produced by:
   - `Bytecode::Class`,
   - `Bytecode::Enum`,
   - `Bytecode::Data`,
   - `Bytecode::Constant` referring to an existing class.
7. Intrinsic structural data facades deliberately install behavior declared by one semantic declaration onto an existing primordial runtime class (`Tuple`, `Record`, `Pack`). Therefore runtime class identity and canonical callable-owner identity are related but are not the same fact.
8. `Bytecode::VariantMethod` uses exact variant semantic targets and inserts a method into the hidden variant behavior class, but it does **not** execute the full ordinary `Method` holder/access-owner/lexical-anchor mutation path.
9. `ReserveScratchLocal` and `ReleaseScratchLocal` manipulate actual frame/local stack storage; there is no independent runtime scratch array.
10. P2's current loader acceptance style requires exact whole-load ledger accounting against an independent allocator, exact `N`, `N-1` rejection, rollback, and retry. P3 validator allocations must participate in the same discipline.
11. `VM::new_native()` is not source-independent. `install_native_runtime` calls `builtin_materialize::initialize_canonical_universe`, which enumerates `UniverseSourceProvider`, loads interfaces, and obtains source IDs.
12. `VM::initialize_single_module` already owns the canonical `Prepared → Initializing → Initialized|Failed` state machine and sticky failure semantics.
13. `VM::run_compiled` compiles a missing module initializer from `source_text` only when its runtime `ModuleObject.closure` is absent. An artifact-backed canonical initializer can therefore avoid source fallback by attaching the persisted initializer before this branch is reached.
14. `phalcom-core/src/modules/canonical_artifact.rs::finish_loaded_universe_telosc` still publishes `executable_reserved_p8: None`; full executable publication is not complete.
15. The current compiler-only `build_universe_telosc` and writer intentionally produce/reject compiler-only capability. Preserve these APIs; add explicit full-artifact APIs rather than silently changing their contract.
16. Source-declared class/data/enum behavior objects are execution-time objects. They must not be reconstructed as already-initialized build-VM heap snapshots.
17. The branch checkpoint still records P2.R4 as incomplete and P3 as proposed. Do not infer lifecycle completion from code presence.

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

## 4. Required reads before implementation

Read in this order; do not start with broad repository exploration.

```text
Repository / lifecycle
- AGENTS.md, if present at repository or worktree root
- docs/implementation/README.md
- docs/implementation/ARTF001-artifact-and-universe-consolidation/C1/CHECKPOINT.md
- active ARTF001.C1.P2.R4 plan, walkthrough, and handoff
- retired PERF003.C2.P8 plan only as historical implementation context

Normative/runtime behavior
- docs/specs/vm/bytecodes.md
- docs/specs/compiler-artifacts/telosc.md, if still the canonical artifact spec
- any accepted decision/spec directly referenced by the live class/data/enum/native/module-init code being changed

Executable wire / capture / validation
- phalcom-compiler-artifact/src/executable/image.rs
- phalcom-compiler-artifact/src/executable/prototype.rs
- phalcom-compiler-artifact/src/executable/constant.rs
- phalcom-compiler-artifact/src/executable/module.rs
- phalcom-compiler-artifact/src/executable/semantic.rs
- phalcom-compiler-artifact/src/executable/staging.rs
- phalcom-compiler-artifact/src/container.rs
- phalcom-compiler-artifact/src/header.rs
- phalcom-compiler-artifact/src/section.rs
- phalcom-core/src/artifact/capture.rs
- phalcom-core/src/artifact/canonicalize.rs
- phalcom-core/src/artifact/semantic.rs
- phalcom-core/src/artifact/validate.rs

Compiler / VM semantic authority for installation
- phalcom-core/src/compiler/lib/class_decl.rs
- phalcom-core/src/compiler/lib/enum_decl.rs
- phalcom-core/src/compiler/lib/data_decl.rs
- phalcom-core/src/compiler/lib/impl_decl.rs
- phalcom-core/src/bytecode.rs
- phalcom-core/src/vm/dispatch.rs
- phalcom-core/src/modules/semantic_lowering.rs
- phalcom-semantic/src/declarations.rs

Container / module / runtime bootstrap
- phalcom-core/src/modules/canonical_artifact.rs
- phalcom-core/src/modules/canonical_universe.rs
- phalcom-core/src/modules/builtin_materialize.rs
- phalcom-core/src/modules/materialize.rs
- phalcom-core/src/modules/initialize.rs
- phalcom-core/src/modules/registry.rs
- phalcom-core/src/interpret.rs
- phalcom-core/src/native/install.rs
- phalcom-core/src/vm/bootstrap.rs

Existing test conventions
- phalcom-core/tests/artifact_allocation.rs
- phalcom-core/tests/core/modules/canonical_artifact.rs
- existing executable codec/validation tests in phalcom-compiler-artifact and phalcom-core
- phalcom-core/tests/core compiler declaration-dispatch tests relevant to intrinsic runtime facades
```

Avoid broad exploration unless one of these reads exposes a missing owner or an architectural premise becomes false.

---

## 5. Normative authority

### 5.1 Normative behavior

P3 depends on these already-established behaviors:

1. **Bytecode semantics are normative.** Stack effects, control-flow behavior, local/scratch semantics, method installation behavior, enum/data allocation behavior, non-local-return semantics, and malformed-bytecode rejection follow the VM bytecode specification and live VM semantics.
2. **P2 semantic/compiler identity is canonical.** Stable project/module/declaration/callable/variant identity and imported canonical semantic/query products come from the P2 image; executable validation consumes this authority and does not recreate it.
3. **Wire DTOs are transport, not semantic authority.** `phalcom-compiler-artifact` executable structures are portable representations. They do not become trusted runtime facts merely because they decode.
4. **One canonical Universe project identity exists.** Artifact-backed Universe modules retain canonical Universe identity; no ordinary external-project reinterpretation is allowed.
5. **Runtime module initialization semantics already exist.** `ModuleState` and `initialize_single_module` own once-only initialization, re-entry rejection, dependency failure propagation, and sticky failure behavior.
6. **Source declarations allocate runtime behavior objects at execution when the bytecode says they do.** P3 cannot turn execution recipes into heap snapshots.
7. **Native descriptors remain native authority.** The executable image identifies/validates native bindings and owners; it does not replace the Rust native descriptor registry.
8. **Explicit artifact execution is fail-closed.** A missing/corrupt/incompatible executable payload is an error, not a signal to fall back to canonical source bootstrap.

### 5.2 Current implementation

The repository already implements substantial executable capture and validation infrastructure, but has not published a complete executable artifact into runtime authority and has no source-free artifact VM constructor.

### 5.3 Future/non-normative proposals

P3 does **not** authorize:

- general incremental compilation of arbitrary package executables;
- automatic artifact discovery or cache policy;
- `packages/universe` source migration;
- a new runtime bytecode version merely because verifier implementation is difficult;
- general heap snapshot/restore;
- a new semantic/query engine;
- a second module loader/initializer architecture;
- optimization-specific semantics beyond reproducing current canonical executable behavior.

---

## 6. Takeover state

### 6.1 Stable interface map

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| Canonical compiler product | `CanonicalUniverseProgram` in `modules/canonical_universe.rs` | `phalcom-core` | One immutable source-built canonical product supplies compiler image and runtime capture inputs. |
| Compiler artifact builder | `build_universe_telosc` | `modules/canonical_artifact.rs` | Remains compiler-only. |
| Compiler artifact loader | `load_universe_telosc*` | `modules/canonical_artifact.rs` | P2 validation/publication remains authoritative. |
| Loaded compiler image | `LoadedUniverseArtifact` | `phalcom-core` | Immutable shared artifact authority; executable field is not yet real. |
| Executable DTO | `ExecutableImageV1` | `phalcom-compiler-artifact` | Structural portable data only. |
| Executable structural validation | `ExecutableImageV1::validate_structure` | `phalcom-compiler-artifact` | Bounds/references/wire shape, not runtime authorization. |
| Executable staging | `StagedExecutableSectionV1` | `phalcom-compiler-artifact` | Explicitly non-authoritative. |
| Core executable validator | `phalcom-core/src/artifact/validate.rs` | `phalcom-core` | Owns cross-section/runtime proof. |
| Validated executable authority | `ValidatedExecutableImage` | `phalcom-core` | Must remain privately constructible only after full proof. |
| Executable capture | `ExecutableImageCapture` / canonical capture path | `phalcom-core` | Produces recipes from disposable source-built VM. |
| Semantic executable projection | `artifact/semantic.rs` | `phalcom-core` | Runtime → portable side-table projection. |
| Native runtime descriptors | `native/install.rs`, descriptor registry | `phalcom-core` | Sole primitive implementation authority. |
| Native module materializer | `modules/builtin_materialize.rs` | `phalcom-core` | Currently source-provider-backed; must be authority-factored. |
| Module state machine | `ModuleState`, `initialize_single_module` | `phalcom-core` | Reuse; do not duplicate. |
| User program runtime | `VM::run_compiled` | `phalcom-core` | User source compilation remains ordinary; canonical Universe source fallback must be bypassed under artifact authority. |
| VM kernel | `VM::new_kernel` | `phalcom-core` | Correct source-free start point. |

### 6.2 Inherited implementation

The plan inherits and must preserve:

- executable wire V1;
- 128-opcode portable conversion and compatibility identifiers;
- explicit generated control-flow classification;
- capture of eager and deferred canonical module initializers;
- deterministic executable canonicalization;
- existing structural/prototype/native/activation validation;
- P2 exact immutable artifact publication and ordinary session authority;
- exact allocation budgeting infrastructure;
- existing source-denial test controls;
- existing module initialization and failure semantics.

Do not repeat or fork these mechanisms.

---

## 7. Architecture

### 7.1 Authority flow

```text
canonical source + current semantics
        │
        ▼
CanonicalUniverseProgram
        │
        ├─────────────── compiler image ───────────────┐
        │                                              │
        └─ disposable build VM                         │
              │                                        │
              ▼                                        │
       raw ExecutableImageV1                           │
              │                                        │
              ▼                                        │
   core cross-section validation                       │
              │                                        │
              ▼                                        │
   ValidatedExecutableImage                            │
              │                                        │
              └──────── EXECUTABLE section ────────────┤
                                                       ▼
                                             full universe.telosc
                                                       │
                                                       ▼
                                              source-free load
                                                       │
                  ┌────────────────────────────────────┴────────────────────┐
                  ▼                                                         ▼
       canonical compiler authority                              executable authority
                  │                                                         │
                  ▼                                                         ▼
         compile novel user code                                per-VM materialization
                                                                            │
                                                                            ▼
                                                                       VM::new_kernel
                                                                            │
                                                                            ▼
                                                                    native/artifact floor
                                                                            │
                                                                            ▼
                                                                   eager initialization
                                                                            │
                                                                            ▼
                                                                  deferred initialization
```

### 7.2 Decode, validation, publication, execution are different phases

The following boundary is fixed:

```text
bytes decode successfully
    ≠
executable is structurally valid
    ≠
executable is authorized against P2 semantic/runtime identity
    ≠
runtime objects exist
    ≠
initializers have executed
```

No API may collapse these distinctions.

### 7.3 Executable proof architecture

The validator must prove both:

```text
runtime behavior target identity
+
canonical semantic callable-owner identity
```

These are separate facts because intrinsic structural facades can install behavior authored by one declaration onto an existing primordial runtime class.

Recommended private conceptual model:

```rust
enum ExecutableValueProvenance {
    Unknown,
    Other,

    MethodObject {
        constant: ConstantId,
    },

    BehaviorClass {
        runtime_class: StableClassRefV1,
        callable_owner: StableDeclarationRef,
        origin: BehaviorClassOrigin,
    },
}

enum BehaviorClassOrigin {
    ExistingConstant,

    ClassSite {
        prototype: PrototypeId,
        instruction: u32,
    },

    EnumSite {
        prototype: PrototypeId,
        instruction: u32,
        semantic_spec: u32,
    },

    DataSite {
        prototype: PrototypeId,
        instruction: u32,
        semantic_spec: u32,
    },
}
```

Exact private type names may differ. The information model may not.

### 7.4 Producer set

Before accepting the proof lattice as exhaustive, T1 must census all live `Bytecode::Method` emitter sites and demonstrate that canonical behavior-class operands are produced only by:

```text
Bytecode::Class
Bytecode::Enum
Bytecode::Data
Bytecode::Constant(existing/stable class)
```

If another producer exists in current canonical output, STOP AND CONSULT before broadening the architecture.

### 7.5 Ordinary Method proof

At each reachable `Bytecode::Method`:

```text
[..., behavior_class, method_object]
    ↓
exact method ConstantId
exact behavior runtime class
exact semantic callable owner
exact selector
exact side
exact callable identity
exact method recipe/body/native anchor
    ↓
ValidatedMethodInstallation
```

The validator must not infer method authorization merely because a matching selector exists somewhere in the image.

Class-side installation must preserve the distinction among source class, target metaclass, holder, access owner, closure lexical class, and super anchor.

### 7.6 VariantMethod proof

At each reachable `Bytecode::VariantMethod`:

```text
actual method ConstantId
+
exact semantic-pool variant target
+
selector
+
instance side
    ↓
exact StableCallableRef owned by that variant
```

Do not fabricate an ordinary class receiver for this proof.

`VariantMethod` shares method-object/canonical-callable validation with ordinary `Method`, but its runtime postcondition remains the actual VM behavior: installation into the hidden variant behavior class without ordinary `Method` holder/access-owner/lexical-anchor mutation.

### 7.7 Frame/provenance model

Scratch locals are actual frame slots. The provenance analysis must use the same frame/local coordinate semantics as current stack validation.

Conceptually:

```rust
struct ProvenanceFrame {
    locals: Vec<ExecutableValueProvenance>,
    operand_stack: Vec<ExecutableValueProvenance>,
}
```

but the implementation should reuse current stack-transfer/frame calculations rather than duplicating them.

At CFG joins:

```text
same exact provenance on all reachable predecessors → keep
otherwise                                          → Unknown
```

Unreachable is not `Unknown`; unreachable means no state exists for the PC.

### 7.8 One stack/control-flow authority

Do not build an independent 128-opcode stack-effect table.

Reuse/extract the current authoritative machinery around:

```text
instruction_stack_transfer
validate_prototype_stack
validate_prototype_control_flow
generated ControlFlowKindV1
```

The provenance pass adds identity semantics on top of that structural transfer authority.

### 7.9 One-shot identity-establishing sites

Canonical supported behavior is intentionally narrow.

The first P3 implementation should require identity-establishing operations to be in module initializer prototypes and not in a reachable repeated CFG region:

```text
Class
Enum
Data
Method
VariantMethod
```

The validator must reject repeated fresh behavior allocation or conflicting installation unless a real canonical image proves a legitimate unsupported pattern. A shipping counterexample is a consultation trigger, not an invitation to weaken the rule ad hoc.

### 7.10 Materialization architecture

The artifact is a recipe graph, not a heap snapshot.

Per VM:

```text
ValidatedExecutableImage
    ↓
fresh module objects
fresh symbols
fresh method/closure/chunk objects
fresh runtime semantic pools
primordial refs → existing kernel objects
source behavior classes → allocated only by executing Class/Enum/Data
    ↓
existing VM initialization semantics
```

Within one VM, the same object-producing `ConstantId` must memoize to the same runtime object. Across two VMs, mutable runtime objects must be distinct.

### 7.11 Native-floor architecture

Artifact VM construction starts from `VM::new_kernel`, not `VM::new_native`.

Refactor native setup so source and artifact paths provide different topology/owner authorities to shared native mutation logic:

```text
source path:
UniverseSourceProvider
    ↓
canonical runtime topology
    ↓
shared native-floor mutation

artifact path:
loaded ModuleWorld/P2 authority
    ↓
canonical runtime topology
    ↓
shared native-floor mutation
```

No provider/source call edge may remain on the artifact path.

### 7.12 Deferred modules

Deferred canonical initialization reuses the existing module state machine:

```text
Prepared
  ↓
attach persisted initializer closure if needed
  ↓
initialize_single_module
  ↓
Initializing
  ↓
Initialized | Failed
```

Never create `ArtifactModuleState`, a second initialization DAG, or a special once flag.

---

## 8. Ownership boundaries

### 8.1 Owns

P3 owns:

- executable cross-section/runtime validation;
- proof that executable operands correspond to P2 canonical identity;
- private executable authorization;
- full compiler+executable container profile;
- exact allocation accounting for executable load/validation/publication;
- VM-local executable recipe materialization;
- artifact-backed native Universe topology setup;
- artifact-backed VM bootstrap;
- eager and deferred canonical initialization from persisted code;
- explicit same-artifact compiler/runtime API and CLI workflow;
- source-free execution/isolation/recovery/determinism evidence.

### 8.2 Does not own

P3 does not own:

- current semantic meaning;
- query reconstruction policy;
- P2 compiler image contents except consuming them;
- new type/checker/query ontology;
- native primitive implementation definitions;
- general package executable format;
- `packages/universe` migration;
- automatic caches/discovery;
- runtime GC redesign;
- module initialization semantics;
- user program source compilation semantics;
- new optimizer semantics.

### 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| Stable project/module/declaration identity | P2 semantic/module artifact authority | executable validator/materializer | name-only executable identity |
| Callable identity | P2 semantic snapshot/metadata | `Method`/`VariantMethod` verifier | selector search as authority |
| Intrinsic runtime owner relation | semantic declaration authority | stable class-constant verifier | hard-coded facade mapping in verifier if semantic authority already carries it |
| Bytecode stack/control semantics | bytecode declaration + VM semantics | structural validator/provenance | second opcode stack table |
| Method recipe identity | executable constant table | validator/materializer | reconstructed method by signature |
| Runtime behavior object identity | per-VM materializer + execution site | VM | persisted build-VM `ObjRef` |
| Native implementation | native descriptor registry | artifact native floor | executable bytecode/native clone |
| Module once-only state | `ModuleState` / initializer | source/artifact runtime | artifact-specific state machine |
| Loader allocation authority | `AllocationBudget` transaction | full loader | post-hoc estimated accounting |
| Executable authority | private `ValidatedExecutableImage` | materializer/bootstrap | raw `ExecutableImageV1` consumer |
| Full capability | validated full builder/loader | CLI/VM | staging path setting capability |

---

## 9. Global invariants

`INV-01 — P2 compiler/semantic authority is singular.`  
Executable validation consumes the loaded canonical formal image and metadata. It never reconstructs a competing semantic ontology.

`INV-02 — decoded executable data is not execution authority.`  
Only `phalcom-core` can produce `ValidatedExecutableImage`, after cross-section/runtime proof.

`INV-03 — runtime class identity and callable-owner identity are separately proved.`  
Do not infer one mechanically from the other when intrinsic runtime facades or metaclass dispatch make them distinct.

`INV-04 — every reachable Method installation is operand-tied.`  
The exact method object operand, runtime behavior target, semantic callable owner, selector, side, recipe/body/native anchor, and canonical callable must agree.

`INV-05 — every reachable VariantMethod installation is variant-tied.`  
The exact method operand, variant target, selector, instance side, and variant-owned canonical callable must agree.

`INV-06 — fresh behavior allocation is site-specific.`  
`Class`, `Enum`, and `Data` fresh runtime identities are distinguished by validated allocation site and may not be replaced with stable-reference equality.

`INV-07 — canonical identity-establishing execution is one-shot.`  
Fresh behavior allocation and method installation may not occur in reachable repeated control-flow regions in the supported P3 profile.

`INV-08 — scratch locals follow actual frame semantics.`  
No independent scratch namespace or alternate stack model exists in validation.

`INV-09 — unsupported provenance fails closed.`  
Unknown/conflicting/dynamically obscured method or behavior-class provenance is rejection, not heuristic recovery.

`INV-10 — executable validation is allocation-accounted.`  
All successful-path allocations inside the full artifact load transaction are charged to the same budget discipline and must match independent allocator evidence at the certification gate.

`INV-11 — the executable wire schema is unchanged unless evidence proves it insufficient.`  
Validator convenience is not sufficient reason for a schema bump.

`INV-12 — the full artifact builder binds compiler and executable products to one canonical source product.`  
Compiler image and executable capture must derive from the same `CanonicalUniverseProgram`.

`INV-13 — source-declared runtime behavior objects are not preallocated by materialization.`  
Their normal `Class`/`Enum`/`Data` bytecodes perform allocation.

`INV-14 — one object-producing ConstantId preserves within-VM object identity.`  
Repeated references resolve to the same VM-local object; separate VMs receive fresh mutable objects.

`INV-15 — no build-VM heap handles survive serialization.`  
No `ObjRef`, raw pointer, allocator address, runtime symbol ID, or equivalent process-local identity is authoritative on wire.

`INV-16 — artifact VM bootstrap starts from the kernel.`  
`new_native()` is not used unchanged while it retains provider-backed canonical materialization.

`INV-17 — native descriptors remain the primitive implementation authority.`  
Artifact data locates/validates owners and anchors; it does not clone native implementations.

`INV-18 — canonical artifact execution has no source fallback.`  
Missing/incompatible executable data is typed failure.

`INV-19 — existing module initialization state remains authoritative.`  
Eager and deferred artifact initialization reuse `ModuleState` and `initialize_single_module`.

`INV-20 — one loaded immutable artifact supplies compiler and runtime authority.`  
Explicit CLI/runtime flow loads once and shares the same `Arc<LoadedUniverseArtifact>`.

`INV-21 — separate VMs share immutable artifact data only.`  
Globals, heap objects, classes, method mutable state, inline caches, module state, and runtime registries are fresh per VM.

`INV-22 — compiler-only artifacts remain valid compiler products but cannot construct an artifact VM.`

`INV-23 — full executable capability is set only after core executable validation succeeds.`  
`StagedExecutableSectionV1` alone cannot confer capability.

`INV-24 — malformed/incompatible executable bytes cannot run an initializer.`

`INV-25 — determinism excludes process/runtime identity.`  
Fresh builds of the same canonical product produce byte-identical full artifacts.

---

## 10. Non-goals

- No general arbitrary-package executable serialization.
- No redesign of `ExecutableImageV1` unless a STOP/CONSULT condition proves V1 inadequate.
- No heap snapshot format.
- No persisted mutable inline cache contents.
- No persisted VM module state.
- No new canonical source discovery logic.
- No P5 `packages/universe` cutover.
- No automatic artifact search path.
- No global artifact cache policy.
- No new module dependency model.
- No source compiler removal; normal source path remains shipping behavior unless later plans change it.
- No "fallback to source if artifact bootstrap fails."
- No new native primitive table.
- No duplicate method dispatch semantics in the materializer.
- No broad refactor of `validate.rs` in the same patch that establishes the proof unless purely mechanical after G1.
- No unrelated performance work.

---

## 11. Expected impact map

### 11.1 Expected source areas

- `phalcom-core/src/artifact/validate.rs` — complete identity/provenance proof and private authority construction.
- `phalcom-core/src/artifact/validate/tests.rs` or existing adjacent test module — focused hostile/real-image proof tests.
- `phalcom-core/src/artifact/capture.rs` — expose capture against an explicit `CanonicalUniverseProgram` if needed.
- `phalcom-core/src/artifact/canonicalize.rs` — only if real determinism/identity proof requires a bounded correction.
- `phalcom-core/src/artifact/semantic.rs` — source of portable semantic-pool authority; later inverse materialization peer.
- `phalcom-core/src/artifact/materialize.rs` — new per-VM relocation/materialization owner.
- `phalcom-core/src/modules/canonical_artifact.rs` — full build/load/capability/publication APIs.
- `phalcom-core/src/modules/builtin_materialize.rs` — factor source-provider topology from shared native materialization.
- `phalcom-core/src/native/install.rs` — owner/anchor-based primitive origin/install factoring.
- `phalcom-core/src/vm/bootstrap.rs` — artifact VM constructor and shared bootstrap tail.
- `phalcom-core/src/modules/initialize.rs` — minimal deferred initializer attachment seam only if needed.
- `phalcom-core/src/modules/materialize.rs` — share existing module-object/read/export mechanics where architecture matches.
- `phalcom-core/src/interpret.rs` — only the narrow canonical artifact-closure pre-source-fallback seam if required.
- `phalcom-core/bin/phalcom/cli.rs` — explicit full build and `--universe-artifact`.
- `phalcom-core/bin/phalcom/main.rs` — dispatch/load-once wiring.
- `phalcom-compiler-artifact` executable/container code — only transport/capability corrections proven necessary.

### 11.2 Expected tests

- executable validator unit/hostile tests;
- executable codec/capability tests;
- canonical real-image validation test;
- whole-load executable allocation test extensions;
- full artifact build/load test;
- source-denied artifact VM bootstrap test;
- deferred canonical module test;
- two-VM isolation test;
- explicit CLI smoke tests;
- deterministic fresh-build test.

### 11.3 Expected documentation/state

- `docs/implementation/ARTF001-artifact-and-universe-consolidation/C1/CHECKPOINT.md`
- `ARTF001.C1.P3-walkthrough.md`
- `ARTF001.C1.P3-handoff.md`
- this plan, amended only when a material approved decision occurs.

### 11.4 Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically required by a named task.

STOP AND CONSULT before entering a materially different subsystem not anticipated above when that entry changes identity, ownership, lifetime, representation, semantic authority, wire schema, module semantics, or native authority.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

1. P2 semantic/compiler authority is upstream and singular.
2. `ExecutableImageV1` remains non-authoritative until core validation.
3. Runtime class identity and canonical callable-owner identity are separate proof dimensions.
4. `Method` and `VariantMethod` use operand-tied exact proof, not selector/name search.
5. `VariantMethod` does not inherit ordinary `Method` installation postconditions.
6. `Class`/`Enum`/`Data` source behavior allocation remains execution-time.
7. Scratch locals are modeled as frame/local storage, not a separate scratch state.
8. Core executable validation is bounded and allocation-accounted.
9. No wire-schema change is made merely for implementation convenience.
10. Full builder compiler/executable products derive from one canonical product.
11. Artifact VM begins from `new_kernel`.
12. Native descriptor registry remains implementation authority.
13. Existing `ModuleState` remains initialization authority.
14. Explicit artifact runtime never falls back to canonical source.
15. Same loaded artifact `Arc` supplies compiler and runtime authority.
16. Separate VMs reconstruct fresh mutable runtime state.
17. Compiler-only artifact behavior remains supported.
18. Full executable capability is granted only after validated authority exists.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- private helper names;
- whether proof structs live in `validate.rs` or a private submodule after G1;
- exact worklist container implementation;
- exact budget-aware vector/index representation;
- exact names of private materialization maps;
- exact error enum variant names when they preserve typed failure classification;
- private helper signatures around native topology installation;
- whether deferred initializer attachment occurs in `initialize.rs`, `materialize.rs`, or a narrow helper invoked by `run_compiled`;
- test module/file placement matching current conventions;
- CLI subcommand naming details if the live CLI parser makes the specified spelling mechanically awkward, provided explicit build and explicit artifact-run workflows remain.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| Branch still descends from `aad6a88db` without conflicting executable architecture | branch head, checkpoint, recent diff | adapt mechanical drift; STOP on architectural drift |
| P2.R4 is formally accepted before runtime publication | `CHECKPOINT.md`, P2 completion records | do not start authoritative P3 runtime work |
| Canonical `Method` receivers come only from Class/Enum/Data/stable class Constant | compiler emitter census + real captured image | STOP AND CONSULT before expanding producer model |
| `Method`/`VariantMethod` identity-establishing sites are module-initializer/one-shot in shipping image | captured image census + CFG | STOP AND CONSULT on real counterexample |
| One method `ConstantId` is installed at no more than one reachable site | real image census | consult before weakening single-install rule |
| P2 loaded authority preserves intrinsic runtime owner relation required for Tuple/Record/Pack facade proof | P2 snapshot/declarations after artifact load | STOP; may imply authority/schema gap |
| Existing stack-transfer helper can serve provenance without duplicate opcode semantics | `validate.rs` | refactor mechanically; STOP only if semantics genuinely missing |
| Budgeted containers/helpers are sufficient for validator scratch accounting | allocation library + P2 loader patterns | implement equivalent budget-aware representation; STOP if transaction model must change |
| Full builder can take explicit `CanonicalUniverseProgram` | `canonical_universe.rs`, current call graph | adapt API mechanically |
| Source-declared behavior objects can remain unallocated until initializer execution | materializer dependencies + constant recipes | STOP if valid recipe requires preexisting source class object |
| Portable semantic pools reconstruct without semantic analysis | `artifact/semantic.rs`, runtime types | STOP on missing runtime authority |
| Artifact native topology can be derived from P2 module world + executable records | module world, native bindings | STOP if source provider remains semantically required |
| Existing module initializer can initialize deferred external canonical dependencies without a second state model | `initialize.rs`, user `CompiledProgram` dependency graph | trace and adapt shared dependency seam; STOP before introducing second graph |
| CLI can load artifact once and pass same `Arc` to compiler/runtime | current CLI/compiler API | adapt mechanically |

---

## 13. Global STOP / CONSULT triggers

The implementer must stop editing and build a consultation packet if any of the following occurs:

1. The architecture appears impossible or materially wrong.
2. A core plan premise is false.
3. Ownership, identity, lifetime, representation, semantic-authority, or allocation-transaction boundaries must change.
4. A downstream contract must materially change.
5. A semantic expected behavior or test oracle appears wrong.
6. The same nontrivial semantic failure persists after one serious correction.
7. Two plausible fixes have materially different architecture.
8. An unexpected subsystem is required.
9. A workaround/special-case architecture is emerging.
10. Duplicate canonical reasoning/state would be introduced.
11. Soundness-critical behavior is uncertain.
12. Spec/code/test/plan authorities materially conflict.
13. An invariant must be weakened to pass tests.
14. Scope becomes materially more cross-cutting than planned.
15. Real canonical `Method` behavior-class provenance uses a producer outside the verified producer set.
16. Runtime behavior target cannot be mapped to exactly one authorized semantic behavior owner.
17. Intrinsic runtime-facade ownership is unavailable after P2 source-free import.
18. A canonical behavior allocation or installation legitimately occurs in a repeatedly invoked/cyclic execution region.
19. One mutable method recipe is legitimately installed at multiple distinct sites and the single-install rule rejects shipping output.
20. Correct source behavior requires preallocating a class/data/enum object before its allocation bytecode executes.
21. A portable semantic-pool row cannot reconstruct its runtime counterpart without re-running semantic analysis.
22. Correct artifact native setup still requires `UniverseSourceProvider` or parsed canonical source.
23. Deferred canonical initialization appears to require a second state machine or independent dependency graph.
24. Executable validation cannot participate in the established exact allocation transaction.
25. A wire-schema/ABI change appears necessary.
26. Source fallback is proposed as a fix for explicit artifact execution failure.
27. Native implementation logic would be duplicated into artifact materialization.
28. Malformed executable data could reach initializer execution before rejection.

A triggered consultation cannot be self-waived.

---

## 14. Debugging budget

### 14.1 Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress.

A cycle is:

```text
inspect
identify concrete cause
make one coherent correction
rerun smallest discriminating command
```

Mechanical examples:

- renamed private helper;
- borrow/lifetime issue after validator factoring;
- budget-aware container API mismatch;
- test fixture constructor updated for a private signature;
- CLI parser wiring error.

After three cycles without convergence, classify whether the problem is semantic/architectural and consult if applicable.

### 14.2 Semantic failures

Before changing code, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic failure.

If it persists: STOP AND CONSULT.

Semantic examples:

- canonical `Method` owner mismatch;
- intrinsic facade maps to unexpected runtime class;
- deferred module dependency order differs from source semantics;
- two-VM identities unexpectedly alias;
- initializer behavior differs between source and artifact path.

### 14.3 Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

Never rerun an unchanged failing test unless something relevant to the failure changed.

---

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| CV-01 | canonical success | INV-02–INV-09 | real captured canonical executable validates |
| CV-02 | method identity negative | INV-03, INV-04 | wrong owner/selector/side/class/method constant rejects |
| CV-03 | variant identity negative | INV-05 | wrong variant/selector/method rejects |
| CV-04 | behavior producer coverage | INV-06 | Class, Enum, Data, stable Constant all handled |
| CV-05 | repeated allocation/install | INV-07 | loop/cyclic identity-establishing site rejects |
| CV-06 | scratch/frame transfer | INV-08 | reserve/set/get/release preserves/invalidates exact facts correctly |
| CV-07 | provenance loss | INV-09 | global/upvalue/dynamic call path cannot fabricate exact install proof |
| CV-08 | exact validation allocation | INV-10 | executable proof scratch participates in exact N/N−1 accounting |
| CV-09 | schema stability | INV-11 | canonical image validates without wire V1 change |
| CV-10 | one-product full build | INV-12 | compiler/executable sections derive from same canonical product |
| CV-11 | execution-time class allocation | INV-13 | materializer does not preallocate source Class/Enum/Data |
| CV-12 | constant identity | INV-14 | repeated object ConstantId shares within VM, differs across VMs |
| CV-13 | no runtime handle on wire | INV-15, INV-25 | fresh builds byte-identical across process history |
| CV-14 | source-free native floor | INV-16–INV-18 | provider/source denial armed before artifact VM construction |
| CV-15 | eager init | INV-18, INV-19, INV-24 | eager bootstrap succeeds source-free once |
| CV-16 | deferred init | INV-18, INV-19 | deferred module initializes from stored closure once |
| CV-17 | compiler-only rejection | INV-22 | compiler-only artifact cannot construct artifact VM |
| CV-18 | capability authority | INV-02, INV-23 | staged/raw executable cannot grant full capability |
| CV-19 | corruption before execution | INV-24 | malformed executable rejected before initializer counter changes |
| CV-20 | one-Arc workflow | INV-20 | compiler and VM consume same loaded artifact Arc |
| CV-21 | two-VM isolation | INV-21 | globals/classes/caches/module state independent |
| CV-22 | recovery | INV-19, INV-21 | failed VM bootstrap does not poison artifact; clean fresh retry works |
| CV-23 | diagnostics | INV-18 | source-free runtime failures retain useful logical module/span identity |
| CV-24 | source/artifact parity | INV-13–INV-21 | representative natives/classes/ADTs/generics/traits/reflection/contracts match source path |
| CV-25 | deterministic publication | INV-25 | two fresh full builds are byte-identical |

Applicable dimensions intentionally emphasized:

- canonical success;
- negative identity/soundness;
- lifecycle/publication;
- aliasing/object identity;
- repetition;
- failure recovery;
- initialization ordering;
- source fallback denial;
- differential source/artifact oracle;
- GC/lifetime during materialization;
- diagnostics;
- allocation/resource boundedness;
- deterministic serialization.

Concurrency/liveness is not a standalone P3 gate unless existing canonical bootstrap exercises it; do not add new concurrency work solely for this plan.

---

## 16. Verification execution budget

### 16.1 Modes

#### BUILD MODE

Sparse discriminating feedback only. Use exact new tests and owning-library `cargo check`.

#### STABILIZE MODE

Run focused validator/container/materializer/bootstrap regressions and directly affected crate suites.

#### CERTIFY MODE

Run the plan's explicit integration/allocation/source-denial/isolation/determinism gates plus selected owning crate checks. Do not automatically run the entire workspace test suite.

### 16.2 Verification ladder

```text
T0 — exact reproducer/new regression
T1 — directly affected feature tests
T2 — owning subsystem/module suite
T3 — adjacent cross-layer integration
T4 — broad crate/package verification
T5 — workspace/release verification
```

### 16.3 Mandatory during BUILD

After semantic validator changes:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --lib artifact::validate -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check -p phalcom-core --lib

git diff --check
```

After executable transport changes:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-compiler-artifact --test executable_image_codec_test

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check -p phalcom-core --lib
```

If the exact current test target name differs, identify the owning executable codec test and use that target. Do not broaden to all workspace tests for a naming mismatch.

### 16.4 Mandatory at named gates

- G1 — focused core validator proof suite + real canonical captured image.
- G2 — executable codec/capability + full builder/load round trip.
- G3 — exact allocation target, single-threaded, diagnostics enabled as required by current test.
- G4 — materialization-only test with initializer counter at zero.
- G5 — source-denied native-floor test.
- G6 — source-denied eager full VM tracer.
- G7 — source-denied deferred module test.
- G8 — CLI/API one-artifact tests.
- G9 — two-VM isolation.
- G10 — failure/retry.
- G11 — deterministic fresh-build comparison.
- G12 — final focused acceptance matrix.

### 16.5 Mandatory during STABILIZE

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic-artifact

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check -p phalcom-core --lib --bin phalcom

cargo fmt --all -- --check
git diff --check
```

Run only after the associated cross-layer behavior exists.

### 16.6 Only if evidence demands

- full `phalcom-core --lib` test suite;
- adjacent language integration suite implicated by a parity failure;
- broader module/native tests when a shared helper was materially changed;
- workspace `cargo check --all-targets` at final certification if repository policy requires it.

### 16.7 Explicitly deferred / DO NOT RUN during early BUILD

- workspace test suite;
- workspace clippy;
- broad language corpus;
- performance benchmark suite;
- source/artifact differential corpus before a complete artifact VM exists;
- deferred-import runtime tests before T7;
- two-VM tests before T6;
- CLI runtime tests before T8.

Reason: these intermediate states are intentionally incomplete and broad failures would provide low-signal evidence while consuming substantial repository execution time.

---

## 17. Baseline / unrelated failure policy

Classify:

```text
A — definitely caused by this patch
B — probably caused by this patch
C — unclear
D — clearly unrelated/baseline
```

- A/B: active responsibility.
- C: perform one bounded classification pass; if nonblocking and still unclear, record and continue.
- D: record and continue immediately.
- Do not consume adviser compute on D unless it blocks a required acceptance gate.
- Do not weaken/skip assertions to make broad gates green.
- Do not reclassify a prior known baseline failure as P3 work merely because a broader command encounters it.
- Record deferred failures in the checkpoint ledger with command, observed result, classification, and why they do or do not block P3.

---

# 18. Tasks

## T0 — Activate P3 and establish the entry receipt

### Purpose

Make the lifecycle and branch state truthful before new P3 authority is introduced.

### Preconditions

- repository access;
- current `feat/artifact-universe-consolidation`;
- P2.R4 completion evidence available.

### Consumes

- `CHECKPOINT.md`;
- P2.R4 final plan/walkthrough/handoff;
- retired P8 history;
- live branch head.

### Produces

- exact P3 starting revision;
- explicit P2 acceptance receipt;
- checkpoint state showing P3 active;
- record that retired P8 implementation is inherited code, not a separately active plan.

### Files and symbols

**Read:**
- `docs/implementation/ARTF001-artifact-and-universe-consolidation/C1/CHECKPOINT.md`
- P2.R4 completion records
- retired P8 plan/checkpoint records

**Modify:**
- `CHECKPOINT.md`
- this P3 plan only if live branch drift requires a material approved amendment

**Create:**
- none

**Tests:**
- none

### Required implementation shape

1. Fetch/re-read branch head.
2. Record exact SHA.
3. Confirm P2.R4 has a truthful accepted exit state sufficient for P3 consumption.
4. Reconcile any post-`aad6a88db` executable commits with this plan.
5. Mark P3 `IN_PROGRESS` only when its entry condition is satisfied.
6. Record the inherited executable groundwork and unresolved P3 work without claiming runtime source-free completion.

### Forbidden approaches

- no code edits before resolving a false P2 entry premise;
- no treating "focused tests passed" as P2 closure unless the lifecycle record says so;
- no reviving PERF003.C2.P8 as a parallel active owner.

### Test changes required

None.

### Tests to run now

None.

### Tests explicitly deferred

All Cargo commands.

### Acceptance

- branch revision recorded;
- P2 entry dependency truthfully resolved;
- one active plan owns executable completion.

### Local STOP / CONSULT triggers

- P2 is still formally blocked;
- branch moved with material architecture conflicting with this plan.

### Checkpoint update

Mandatory: record P3 activation and exact starting revision.

---

## T1 — Complete bounded executable identity validation

### Purpose

Turn a structurally decoded `ExecutableImageV1` into private executable authority only after exact control-flow, runtime-object, and canonical semantic identity proof.

### Preconditions

- T0 complete;
- P2 canonical formal image and metadata authority available;
- existing structural/stack/control-flow validation compiles.

### Consumes

- `ExecutableImageV1`;
- P2 `CanonicalUniverseFormalImage`;
- P2 metadata bundle;
- existing executable compatibility/native anchor/class/semantic-pool rows;
- current opcode control-flow/stack semantics;
- allocation budget.

### Produces

Conceptual API:

```rust
pub(crate) struct ExecutableProofLimits {
    pub max_cfg_edges: usize,
    pub max_reachable_states: usize,
    pub max_provenance_facts: usize,
    pub max_worklist_steps: usize,
}

pub(crate) struct ValidatedExecutableImage {
    image: Arc<ExecutableImageV1>,
    // private validated proof/index products as required
}

pub(crate) fn validate_executable_image(
    image: Arc<ExecutableImageV1>,
    formal_image: &CanonicalUniverseFormalImage,
    metadata: &SemanticMetadataBundle,
    image_limits: &ExecutableImageLimitsV1,
    proof_limits: &ExecutableProofLimits,
    allocations: &mut AllocationBudget,
) -> Result<ValidatedExecutableImage, ExecutableValidationError>;
```

Exact signatures may follow live module ownership, but validation must no longer require an already-published `LoadedUniverseArtifact` if that creates a publication cycle.

### Files and symbols

**Read:**
- `phalcom-core/src/artifact/validate.rs`
- `phalcom-core/src/vm/dispatch.rs`
- `phalcom-core/src/compiler/lib/class_decl.rs`
- `phalcom-core/src/compiler/lib/enum_decl.rs`
- `phalcom-core/src/compiler/lib/data_decl.rs`
- `phalcom-core/src/compiler/lib/impl_decl.rs`
- `phalcom-core/src/bytecode.rs`
- `phalcom-core/src/artifact/capture.rs`
- `phalcom-core/src/artifact/canonicalize.rs`
- `phalcom-core/src/artifact/semantic.rs`
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-semantic/src/declarations.rs`

**Modify:**
- `phalcom-core/src/artifact/validate.rs`
- existing adjacent validation test module/file

**Create:**
- optional private `phalcom-core/src/artifact/validate/tests.rs` if consistent with current module layout

**Tests:**
- focused unit/hostile validator tests
- one real canonical captured-image validator test/census

### Required implementation shape

#### T1.1 Refactor authority inputs

Replace validation context dependence on `LoadedUniverseArtifact` with the minimum immutable P2 authority:

```text
CanonicalUniverseFormalImage
SemanticMetadataBundle
existing compatibility identity
```

Do not publish a temporary fake loaded artifact to break the cycle.

#### T1.2 Add explicit proof resource limits

Before allocating CFG/provenance scratch:

- checked arithmetic for state/edge/fact counts;
- budget-aware vectors/maps/sets or fixed indexed arrays where possible;
- bounded worklist operations;
- no successful-path ordinary allocation outside the transaction.

#### T1.3 Census behavior-class producers

Search all current compiler emitter sites for `Bytecode::Method` and trace the immediately available behavior target.

Required supported seeds:

```text
Class
Enum
Data
stable/existing class Constant
```

Add a real canonical image diagnostic/census assertion so future compiler changes break loudly.

#### T1.4 Define provenance state

Track at least:

```text
MethodObject { ConstantId }
BehaviorClass {
    runtime_class,
    callable_owner,
    origin,
}
Unknown
Other
```

Use allocation-site identity for `Class`, `Enum`, `Data`.

Do not merge distinct fresh sites merely because stable declarations compare equal.

#### T1.5 Seed stable class constants

For class-bearing constants:

- validate the portable class reference;
- resolve exact P2 declaration/runtime-owner authority;
- retain runtime class ref and callable owner separately;
- prove intrinsic structural facade mapping through semantic authority.

Do not hard-code name-only matches.

#### T1.6 Seed Class

At reachable `Bytecode::Class`:

- require the prototype ownership expected by canonical lowering;
- identify exact module/declaration from P2 stable authority;
- match exactly one `SourceDeclared` class recipe;
- tag the exact `(prototype, instruction)` allocation site;
- reject a duplicate site claiming the same source-declared class unless consultation approves a real canonical case;
- reject reachable cyclic/repeated allocation.

#### T1.7 Seed Enum

At `Bytecode::Enum`:

- obtain exact portable semantic enum spec;
- use its exact stable declaration owner;
- validate the runtime class recipe relationship;
- tag exact allocation site;
- apply the same one-shot restrictions as Class.

#### T1.8 Seed Data

At `Bytecode::Data`:

- obtain exact portable semantic data spec;
- use exact stable declaration owner;
- validate runtime class recipe relationship;
- tag exact allocation site;
- apply one-shot restrictions.

#### T1.9 Model actual frame/local transfer

Reuse current stack-transfer authority.

Important transfer rules:

- `Constant` produces exact constant provenance where supported.
- `Dup` duplicates exact provenance.
- local get/set operations preserve copied identity according to VM semantics.
- `ReserveScratchLocal` inserts a frame/local slot at the exact runtime coordinate.
- `ReleaseScratchLocal` removes it at the exact coordinate.
- arbitrary dynamic reads/calls do not fabricate exact method/class identity.
- unknown/conflicting join inputs become `Unknown`.

Do not create a separate scratch array.

#### T1.10 Ordinary Method proof

For every reachable `Bytecode::Method(selector, static)`:

1. read method provenance from actual consumed stack operand;
2. read behavior-class provenance from actual retained class operand;
3. require singleton exact facts;
4. resolve exact selector symbol;
5. derive exact dispatch side;
6. construct/resolve exact canonical callable from `callable_owner + side + selector`;
7. require P2 authority for that callable;
8. validate method recipe signature/rest/arity against callable/selector;
9. validate closure prototype or native anchor;
10. validate runtime behavior target and class/metaclass role;
11. validate class-side holder/access-owner/lexical/super relationships without conflating them;
12. record a private validated installation proof.

Unknown/conflicting operands reject.

#### T1.11 VariantMethod proof

For each reachable `Bytecode::VariantMethod`:

- obtain exact method `ConstantId`;
- obtain exact `PortableSemanticPoolV1.variant_targets[variant]`;
- resolve exact selector;
- construct exact variant-owned instance callable;
- validate method recipe body/signature;
- record variant installation proof;
- do not apply ordinary `Method` postcondition requirements that the VM does not perform.

#### T1.12 Recipe reuse census

For the real canonical image, record each method `ConstantId`'s reachable installation sites.

Default V1 rule:

```text
one installed method ConstantId
→ at most one reachable installation site
```

If real shipping output violates this, STOP AND CONSULT before permitting multiple same-binding sites.

#### T1.13 Unreachable identity-mutating instructions

Require structural validity for all bytecode. For the canonical executable profile, reject unreachable `Class`, `Enum`, `Data`, `Method`, and `VariantMethod` unless a verified current producer requires otherwise.

#### T1.14 Termination

The analysis must be monotone: facts can remain equal or degrade to `Unknown`; it must not grow unbounded candidate sets.

Enforce explicit worklist-step/resource caps anyway.

#### T1.15 Authority construction

Only after every executable/prototype/install proof succeeds, construct the private `ValidatedExecutableImage`.

No other constructor is permitted.

### Forbidden approaches

- no selector-only search;
- no class-name-only search;
- no "matching signature means same method object";
- no stable declaration equality as substitute for fresh runtime object identity;
- no separate scratch-slot storage model;
- no duplicate opcode stack-effect table;
- no raw `ExecutableImageV1` runtime consumer;
- no unbudgeted successful-path validation collections;
- no schema bump to carry facts already derivable from P2 + semantic pools;
- no source AST inspection during loaded executable validation;
- no permitting loops merely because the captured image came from the compiler.

### Test changes required

Create exact focused cases mapped to CV-01 through CV-09, including at minimum:

- canonical real image succeeds;
- Class-produced behavior target succeeds;
- Enum-produced behavior target succeeds;
- Data-produced behavior target succeeds;
- stable primordial/intrinsic class constant succeeds;
- intrinsic facade owner mismatch rejects;
- wrong method constant rejects;
- wrong callable owner rejects;
- wrong selector rejects;
- wrong side rejects;
- wrong runtime class rejects;
- wrong variant rejects;
- conflicting CFG join rejects;
- global/upvalue-obscured install rejects;
- repeated Class/Enum/Data site rejects;
- repeated Method install rejects;
- malformed `ReturnNonLocal`/activation invariant remains rejecting;
- exact proof resource-limit failure is typed and rollback-safe.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --lib artifact::validate -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check -p phalcom-core --lib

git diff --check
```

Require at least one validator test to execute; a zero-test filter is failure.

### Tests explicitly deferred

- full artifact VM bootstrap;
- source-denied runtime execution;
- deferred imports;
- CLI runtime;
- two-VM isolation;
- workspace tests.

> **Do not run runtime/integration tests after this task. The intermediate state is intentionally incomplete.**

### Acceptance

- real canonical executable image validates;
- every reachable `Method`/`VariantMethod` is backed by exact operand-tied proof;
- Class/Enum/Data/stable-constant behavior producers are explicitly supported and exhaustively censused;
- unsupported/conflicting/repeated identity fails closed;
- `ValidatedExecutableImage` cannot be constructed from unvalidated raw data;
- focused tests and core library check pass;
- no wire V1 change was required, or a consultation has explicitly amended this plan.

### Local STOP / CONSULT triggers

All T1-related global triggers, especially producer-set counterexample, intrinsic-owner absence, legitimate repeated install, required schema change, or inability to account proof allocations.

### Checkpoint update

Record the durable executable authorization contract and G1 evidence.

---

## T2 — Publish a full compiler+executable artifact profile

### Purpose

Compose one deterministic full `universe.telosc` in which executable capability is conferred only after T1 core validation.

### Preconditions

- T1 accepted;
- compiler-only P2 builder/loader still passing focused checks.

### Consumes

- `CanonicalUniverseProgram`;
- P2 compiler image view;
- raw executable capture;
- T1 validator;
- existing container builder/section/capability types.

### Produces

Recommended API shape:

```rust
pub fn build_full_universe_telosc(
    canonical: &CanonicalUniverseProgram,
    limits: &ValidationLimits,
) -> Result<BuiltUniverseTelosc, TeloscArtifactError>;

pub fn write_full_universe_telosc(
    path: &Path,
    built: &BuiltUniverseTelosc,
) -> Result<(), TeloscArtifactError>;
```

And a loader capable of publishing an optional/private validated executable when the exact full profile is present.

### Files and symbols

**Read:**
- `modules/canonical_artifact.rs`
- `modules/canonical_universe.rs`
- `artifact/capture.rs`
- executable staging/image/container/header/section files

**Modify:**
- `phalcom-core/src/modules/canonical_artifact.rs`
- `phalcom-core/src/artifact/capture.rs` if explicit canonical input is needed
- minimal `phalcom-compiler-artifact` transport files only if current profile validation is incomplete

**Create:**
- focused full-artifact tests if current canonical-artifact test module is already too broad

**Tests:**
- executable container round trip;
- exact capability profile rejection matrix;
- compiler-only artifact remains compiler-only.

### Required implementation shape

#### T2.1 Bind compiler/executable construction to one canonical product

Prefer:

```rust
capture_canonical_universe_executable_image(
    canonical: &CanonicalUniverseProgram
)
```

instead of reacquiring `canonical_universe_program()` internally after the caller supplied compiler-image authority.

Both compiler sections and executable capture must derive from the same canonical object.

#### T2.2 Build full image in authority order

```text
derive P2 sections
capture raw executable image
validate raw executable against the same P2 image
encode validated executable payload
assemble five-section container
set exact full capabilities
self-load final bytes
publish BuiltUniverseTelosc
```

Staging alone must not set `PORTABLE_EXECUTABLE`.

#### T2.3 Preserve compiler-only APIs

`build_universe_telosc` remains compiler-only.

`write_universe_telosc` retains its current compiler-only contract, including rejecting a full executable product if that is its current safety contract.

Use an explicit full writer.

#### T2.4 Exact profiles

Accept exactly:

```text
CompilerOnly:
MODULE_WORLD
SEMANTIC_WORLD
QUERY_IMAGE
METADATA

Full:
MODULE_WORLD
SEMANTIC_WORLD
QUERY_IMAGE
METADATA
EXECUTABLE
+ PORTABLE_EXECUTABLE
```

Reject:

- executable capability without section;
- executable required section without capability;
- duplicate executable section;
- wrong executable schema;
- unknown capability bits;
- incompatible executable ABI/native digest;
- compiler/executable source/compatibility mismatch.

#### T2.5 Loader publication

Load ordering:

```text
container
P2 sections/import
formal image
metadata
executable decode
T1 validation
validated executable authority
final LoadedUniverseArtifact allocation
```

Replace the P8 placeholder with a private validated executable field.

Expose only safe public queries such as:

```rust
pub fn has_portable_executable(&self) -> bool;
```

Runtime code can use a crate-private accessor returning validated authority.

#### T2.6 Atomic full writer

Validate the actual built bytes before replacing destination.

Retain same-directory temp + sync + atomic persist behavior.

### Forbidden approaches

- no changing compiler-only builder semantics in place;
- no `PORTABLE_EXECUTABLE` based solely on section presence;
- no loader publishing raw executable DTO;
- no executable validation after `LoadedUniverseArtifact` publication;
- no two independent canonical-product acquisitions;
- no source bootstrap during full artifact load.

### Test changes required

CV-10, CV-17, CV-18, CV-19:

- compiler-only round trip remains valid;
- compiler-only VM capability absent;
- full round trip publishes validated executable;
- malformed executable blocks loaded artifact publication;
- wrong capability/section combinations reject;
- full write does not replace existing file on validation failure.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-compiler-artifact --test executable_image_codec_test

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --lib canonical_artifact -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check -p phalcom-core --lib

git diff --check
```

Use the nearest exact current canonical-artifact filter if module naming differs.

### Tests explicitly deferred

Runtime materialization/bootstrap/deferred/CLI.

### Acceptance

- compiler-only and full profiles are distinct and exact;
- full load publishes executable authority only after T1;
- malformed executable cannot yield `LoadedUniverseArtifact`;
- full bytes self-load before publication;
- compiler-only behavior is preserved.

### Local STOP / CONSULT triggers

- outer container schema must change;
- executable V1 must change;
- P2 loader transaction semantics must be weakened;
- same-product build cannot be expressed without changing canonical product ownership.

### Checkpoint update

Record full profile and exact build/load APIs after G2.

---

## T3 — Extend exact allocation accounting through executable publication

### Purpose

Preserve P2's exact whole-load allocation invariant after adding executable decode, proof, retained indexes, and publication.

### Preconditions

- T2 full artifact loads functionally;
- allocation diagnostics infrastructure still available.

### Consumes

- full canonical artifact bytes;
- `AllocationBudget`;
- executable decode/proof/publication stages;
- existing independent counting allocator test.

### Produces

- exact `N_full`;
- stage-by-stage executable allocation diagnostics;
- N/N−1/rollback/retry evidence.

### Files and symbols

**Read:**
- `phalcom-core/tests/artifact_allocation.rs`
- `modules/canonical_artifact.rs`
- allocation helpers used by P2
- T1 validator scratch allocation code

**Modify:**
- loader allocation diagnostics
- `artifact_allocation.rs`
- validator budget code if deltas exist

**Create:**
- none unless executable allocation census needs a small private helper

**Tests:**
- whole-load exact accounting.

### Required implementation shape

Add named stages for at least:

```text
executable decode
executable core validation/proof
validated executable retained proof/index publication
```

Measure a successful full load with diagnostics outside the independent allocator run to discover `N_full`.

Then require:

```text
ledger requested == independently observed allocator requests == N_full
```

Run exact budget `N_full`: success.

Run `N_full - 1`: typed budget failure with no partial published artifact.

Retain/re-run child-transaction rollback checks and clean retry.

Add/extend prepublication failpoint coverage so executable validation/publication failure cannot leak an artifact.

Do not "fix" accounting by adding the observed delta after the fact. Every allocation source must be charged at its owning code path.

### Forbidden approaches

- no tolerance window;
- no "close enough" accounting;
- no untracked `BTreeMap`/`Vec`/`Arc` successful-path allocation in loader/proof;
- no weakening existing P2 exact assertions;
- no excluding executable validation from the loader transaction without a separately approved accounting architecture.

### Test changes required

CV-08, CV-19, CV-22.

### Tests to run now

Use the current diagnostics feature required by the repository; at the audited revision the owning pattern was:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --features allocation-diagnostics \
  --test artifact-allocation -- --nocapture --test-threads=1
```

If the feature name changed, re-read the test and use its current declared feature.

### Tests explicitly deferred

Runtime bootstrap/isolation/CLI.

### Acceptance

- exact full-load ledger equality;
- exact N success;
- N−1 typed failure;
- failed child/executable stage does not commit parent allocation transaction;
- clean retry succeeds;
- no artifact published on executable prepublication failure.

### Local STOP / CONSULT triggers

- exact accounting appears incompatible with T1 proof representation;
- proposal emerges to exclude proof from the transaction;
- retained validation proof requires an unbounded/unaccountable structure.

### Checkpoint update

Mandatory: record `N_full`, exact equality, N−1, rollback, retry, and any new failpoint.

---

## T4 — Materialize VM-local executable recipes without running initializers

### Purpose

Create a fresh per-VM relocation/materialization layer from validated executable recipes while preserving runtime object identity and leaving source behavior allocation to bytecode execution.

### Preconditions

- G1–G3 accepted;
- full artifact retains private validated executable.

### Consumes

- `Arc<LoadedUniverseArtifact>`;
- `ValidatedExecutableImage`;
- fresh `VM::new_kernel`;
- existing heap/rooting APIs;
- P2 metadata/module authority.

### Produces

A private artifact runtime preparation product, conceptually:

```rust
struct VmArtifactRuntime {
    // VM-local relocation/index state only
}

fn prepare_universe_artifact_runtime(
    vm: &mut VM,
    artifact: &Arc<LoadedUniverseArtifact>,
) -> Result<VmArtifactRuntime, VmBootstrapError>;
```

### Files and symbols

**Read:**
- heap allocation/rooting APIs
- `chunk.rs`
- `callable.rs`
- method/closure constructors
- module materialization code
- runtime typing/data/ADT/product registries
- `artifact/semantic.rs`

**Modify:**
- minimal existing helpers necessary for shared constructors

**Create:**
- `phalcom-core/src/artifact/materialize.rs`

**Tests:**
- private materialization identity/rooting tests.

### Required implementation shape

#### T4.1 Per-VM relocation only

No process-global relocation cache.

Key by portable IDs/stable identities only.

#### T4.2 Constant memoization

Use:

```rust
enum ConstantState {
    Unseen,
    Visiting,
    Ready(Value),
}
```

or equivalent.

Object-producing constants memoize globally within the VM materialization transaction.

#### T4.3 Allocation ordering

Allocate declarations/containers before wiring cyclic runtime edges where the runtime representation requires it.

Use existing temporary-root/guard discipline across allocations.

#### T4.4 Primordial mapping

Portable primordial/stable kernel references map to existing objects in the fresh kernel.

Never allocate a shadow Object/Number/String/etc. class for a primordial role.

#### T4.5 Source-declared behavior classes

Do **not** preallocate runtime class objects for recipes whose live execution uses:

```text
Class
Enum
Data
```

Instead materialize the data/layout/spec authority those opcodes need.

#### T4.6 Method objects

Reconstruct method recipe objects and bodies/chunks/closures as recipes permit.

Do not pre-execute ordinary `Method` installation mutations.

In particular, do not fabricate final holder/access-owner/lexical/super state from the canonical callable simply because validation knows where the method will be installed.

#### T4.7 Top-level initializers

Materialize/prepare top-level initializer closure recipes but do not run them.

Nested closures continue to capture at ordinary runtime closure creation; no persisted captured values from the build VM.

#### T4.8 Semantic pool inverse

Implement the inverse of the existing portable semantic projection for all rows actually used by captured bytecode:

- type/kind recipes;
- enum/data specs;
- variant/data targets;
- data constructions;
- anonymous product specs;
- call environments;
- associated targets;
- family/conditional/candidate descriptors;
- trait invocations;
- trait requirement invocations;
- type-parameter class receivers.

Use imported P2 semantic/type authority where the portable recipe references canonical types. Do not run semantic analysis.

#### T4.9 Cache state

All mutable runtime caches start empty/current-runtime-default.

Never serialize or restore inline cache entries, module initialization state, or build-VM cache state.

#### T4.10 GC stress hook

Add focused test-only ability to force/encourage GC during reconstruction if existing heap test APIs allow it without architecture changes.

### Forbidden approaches

- no raw build-VM `ObjRef`;
- no source AST;
- no semantic re-analysis;
- no preinitialized source class objects;
- no global relocation cache;
- no heap snapshot restore;
- no copied mutable inline caches;
- no initializer execution in this task.

### Test changes required

CV-11, CV-12, CV-13 and GC/lifetime portion.

Tests must prove:

- same object ConstantId resolves to same VM-local object;
- distinct VMs produce different mutable objects;
- primordial refs resolve to kernel objects;
- source Class/Enum/Data objects are absent before initializer execution;
- initializer counter remains zero;
- forced allocation/GC does not invalidate references.

### Tests to run now

Run only the new materialization unit/focused tests plus:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check -p phalcom-core --lib
```

> **Do not run full runtime/bootstrap integration yet. The native floor is intentionally incomplete.**

### Tests explicitly deferred

G5 onward.

### Acceptance

Fresh recipe reconstruction completes without executing initializers, source behavior classes remain unallocated until execution, within-VM sharing is exact, cross-VM mutable identity is fresh, and no build-VM handle survives.

### Local STOP / CONSULT triggers

- a valid recipe needs semantic analysis;
- a valid recipe needs a source-declared class object before its allocation bytecode;
- GC safety requires new policy rather than existing rooting discipline.

### Checkpoint update

Record only if a durable materializer interface/invariant is established; otherwise wait for G4.

---

## T5 — Factor and construct a source-free native/artifact floor

### Purpose

Replace provider-backed canonical module topology on the artifact path while preserving one native descriptor/primordial authority.

### Preconditions

- T4 materialization works;
- source path remains available as differential oracle.

### Consumes

- fresh kernel VM;
- P2 module world and metadata;
- executable native anchors/primordial roles;
- existing native descriptor registry;
- shared module/native mutation helpers.

### Produces

- artifact-derived canonical module topology;
- primordial/native binding floor;
- no provider/source call edge.

### Files and symbols

**Read:**
- `modules/builtin_materialize.rs`
- `native/install.rs`
- `vm/bootstrap.rs::install_native_runtime`
- `bind_primordial_universe`
- fixed native layout stamping
- P2 module world/runtime metadata structures

**Modify:**
- `modules/builtin_materialize.rs`
- `native/install.rs`
- `vm/bootstrap.rs`
- narrow module helpers as necessary

**Create:**
- optional private authority/topology input struct in owning module

**Tests:**
- source-denied native floor;
- primordial/native parity.

### Required implementation shape

#### T5.1 Separate authority acquisition from mutation

Factor current code so module/native mutation can consume an explicit topology/owner authority rather than directly calling `UniverseSourceProvider`.

Source path:

```text
provider → topology descriptor → shared installer
```

Artifact path:

```text
loaded module/executable authority → topology descriptor → shared installer
```

#### T5.2 Module objects and ownership

Allocate canonical Universe module/package objects from artifact module authority.

Establish:

- parent package;
- root package;
- `__module__`;
- `__package__`;
- `__root__`;
- `__project__`;
- direct child-package exports;
- canonical privilege/project identity.

#### T5.3 Primordial bindings

Bind exact existing kernel class objects into their canonical module slots.

Preserve special cases, including `None` being the immediate absence value where current runtime semantics require it.

#### T5.4 Native descriptors

Continue to use the existing registered primitive descriptors.

Factor primitive origin/source-owned attachment so artifact authority supplies exact canonical module/declaration ownership without `NativeSourceIndex`.

#### T5.5 Fixed layouts / sealing / aliases

Preserve current native-floor fixed layout stamping, sealing, class aliases, and base-name finalization in the same semantic order as source bootstrap.

#### T5.6 Static/lowering authority

Use loaded P2 semantic/static authority for checked constants/lowering facts required before initializers.

No canonical source compiler call.

### Forbidden approaches

- no `UniverseSourceProvider` on artifact path;
- no `new_native()` unchanged;
- no duplicate native descriptor table;
- no hard-coded complete module catalog beside P2 module authority;
- no source ID requirement for runtime correctness;
- no source fallback.

### Test changes required

CV-14 and native/class parity subset of CV-24.

Source denial must be armed before entering the artifact path.

### Tests to run now

Run the exact new native-floor focused test and:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check -p phalcom-core --lib
```

### Tests explicitly deferred

Full eager initializer execution, deferred imports, CLI.

### Acceptance

- artifact native floor constructs from `new_kernel`;
- provider/source denial remains untriggered;
- canonical module/native/primordial identities match the source path;
- no shadow primordial classes;
- no initializer execution yet unless a narrowly required existing native-floor action already does so.

### Local STOP / CONSULT triggers

- artifact topology lacks information source path requires semantically;
- primitive origin cannot be reconstructed without `NativeSourceIndex`;
- a new duplicate native catalog is proposed.

### Checkpoint update

Record the source-free native-floor boundary and G5 evidence.

---

## T6 — Bootstrap a complete eager VM from one loaded artifact

### Purpose

Construct the first complete source-free artifact-backed VM and execute exactly the stored eager bootstrap order.

### Preconditions

- G1–G5 accepted.

### Consumes

- one `Arc<LoadedUniverseArtifact>` with validated executable;
- fresh kernel/native floor;
- T4 materializer;
- stored bootstrap plan.

### Produces

Public seam:

```rust
pub fn try_new_with_universe_artifact(
    artifact: Arc<LoadedUniverseArtifact>,
) -> Result<VM, VmBootstrapError>;
```

### Files and symbols

**Read:**
- full source bootstrap in `vm/bootstrap.rs`
- `run_universe_modules`
- semantic-root establishment
- alias/error-constructor binding
- invariant checks
- module registry state handling

**Modify:**
- `vm/bootstrap.rs`
- `artifact/materialize.rs`
- `modules/canonical_artifact.rs` private accessor if needed
- error enum/types

**Create:**
- source-denied eager bootstrap integration test

**Tests:**
- G6 tracer and eager parity.

### Required implementation shape

Exact order:

```text
1. require portable executable capability + validated executable
2. reject compiler-only artifact
3. VM::new_kernel
4. materialize artifact module topology/native floor
5. install imported semantic metadata/runtime registries
6. prepare semantic executable pools/layout authority
7. reserve canonical source declaration slots required before execution
8. materialize eager module initializer closure as dependencies become available
9. execute stored eager bootstrap order
10. attach source-owned primitives from artifact authority
11. bind native error constructors
12. bind/synchronize canonical aliases
13. establish semantic roots
14. apply current post-bootstrap fixed/pristine state
15. run existing Universe invariant verification
16. retain same artifact Arc in VM-owned runtime authority
17. return VM
```

Keep source and artifact bootstrap tails shared where they mutate equivalent runtime state.

Do not call:

```text
canonical_universe_program()
NativeSourceIndex::build()
UniverseSourceProvider::source_text()
compile_universe_module()
compile_ast_as_with_bindings() for canonical Universe
```

on the artifact path.

### Forbidden approaches

- no compiler-only artifact acceptance;
- no source fallback;
- no hidden canonical source product;
- no shared mutable runtime state across VMs;
- no running eager modules in a newly invented state machine;
- no changing user program compile mode to satisfy Universe bootstrap.

### Test changes required

CV-15, CV-17, CV-19, source-free part of CV-24.

Fresh child process test:

1. build or load a prebuilt full artifact before source denial only if the test explicitly separates process A and B;
2. process B arms direct canonical source denial before artifact load/bootstrap;
3. load;
4. construct artifact VM;
5. assert eager module counts/order;
6. execute a representative no-user-source or tiny user program as appropriate;
7. assert canonical compilation/source counters remain zero.

### Tests to run now

Run exact new eager bootstrap tracer only, then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check -p phalcom-core --lib
```

Do not broaden after PASS unless the tracer exposed shared-helper changes requiring one owning suite.

### Tests explicitly deferred

Deferred module imports, CLI, two-VM, full parity matrix.

### Acceptance

A fresh source-denied process constructs a complete eager Universe VM from the full artifact with zero canonical source parsing/linking/analysis/lowering/AST-bytecode compilation.

### Local STOP / CONSULT triggers

- source bootstrap ordering cannot be reproduced from stored plan;
- semantic roots require unavailable source-only authority;
- source-owned primitives cannot attach from artifact authority;
- normal VM invariant verification requires source compiler state.

### Checkpoint update

Mandatory: G6 is the first truthful "source-free VM bootstrap works" milestone.

---

## T7 — Integrate persisted deferred canonical initializers

### Purpose

Ensure any later canonical Universe runtime dependency uses stored executable code and existing once-only module semantics, never AST/source fallback.

### Preconditions

- T6 eager VM works;
- executable image includes every canonical module initializer recipe.

### Consumes

- artifact-backed VM;
- deferred module IDs from bootstrap plan;
- existing module records/state machine;
- user compiled program runtime dependencies.

### Produces

- exact persisted initializer attachment seam for deferred canonical modules.

### Files and symbols

**Read:**
- `modules/initialize.rs`
- `interpret.rs::run_compiled`
- `modules/materialize.rs`
- runtime dependency graph structures
- artifact module/prototype map

**Modify:**
- `artifact/materialize.rs`
- `modules/initialize.rs` and/or `interpret.rs` only at the narrow existing seam
- module record only if a minimal non-duplicative reference is required

**Create:**
- source-denied deferred module integration test

**Tests:**
- G7.

### Required implementation shape

#### T7.1 Trace dependency ownership first

Before editing, trace how a user `CompiledProgram` represents runtime dependencies on an external canonical Universe module and how `initialize_program` orders them.

Do not assume deferred canonical modules are present in the user's initialization list.

#### T7.2 Attach persisted closure before source fallback

When a canonical Universe module in an artifact-backed VM is `Prepared` and its runtime closure is absent:

```text
lookup validated stored initializer
materialize closure
attach to module object
continue through existing initialize_single_module
```

#### T7.3 Dependency order

Use existing runtime dependency authority.

If a dependency is still `Prepared`, initialize it through the same shared module initialization path in deterministic dependency order only if that is consistent with current module semantics.

Do not create a second artifact-specific graph.

#### T7.4 Failure semantics

Retain current sticky `Failed` and dependency-failure propagation semantics.

Missing persisted canonical initializer is a typed internal/artifact runtime error, never a source compile request.

### Forbidden approaches

- no `source_text` fallback;
- no second module state enum;
- no artifact-specific once boolean;
- no duplicate dependency DAG;
- no rerunning an eager initialized module;
- no silently marking a missing prototype initialized.

### Test changes required

CV-16.

Choose a canonical module that is actually deferred in the built artifact. Assert:

- not executed during eager bootstrap;
- first runtime dependency/import attaches stored code;
- dependencies initialize first;
- initializer executes exactly once;
- second use does not rerun;
- source denial remains armed;
- intentionally removed/corrupt deferred prototype fails typed.

### Tests to run now

Run only the exact deferred integration tests and:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check -p phalcom-core --lib
```

### Tests explicitly deferred

CLI, two-VM, broad parity.

### Acceptance

Deferred canonical runtime dependencies execute persisted code once under existing module semantics with source denial active.

### Local STOP / CONSULT triggers

- current dependency architecture cannot express deferred canonical dependency without a second graph;
- source fallback is the only apparent path;
- deferred module state semantics materially differ from source path.

### Checkpoint update

Record G7 and exact tested deferred module.

---

## T8 — Expose one-artifact compiler/runtime workflow and explicit CLI

### Purpose

Make the full artifact usable explicitly while guaranteeing compiler and runtime share one immutable loaded authority.

### Preconditions

- T6/G6 and T7/G7 accepted.

### Consumes

- full build API;
- full load API;
- loaded compiler-image user compilation API;
- artifact VM constructor.

### Produces

Explicit workflows:

```text
phalcom artifact build-universe --output universe.telosc

phalcom --universe-artifact universe.telosc <user entry>
```

Exact CLI spelling may adapt mechanically to current parser, but both capabilities are required.

### Files and symbols

**Read:**
- `phalcom-core/bin/phalcom/cli.rs`
- `phalcom-core/bin/phalcom/main.rs`
- current interpreter/program compiler entry flow

**Modify:**
- CLI parser/dispatch
- public artifact API exports
- minimal interpreter/runtime construction seam

**Create:**
- focused CLI smoke cases

**Tests:**
- G8.

### Required implementation shape

Load exactly once:

```rust
let artifact = load_universe_telosc(...)?;
let program = artifact.compile_user_entry(entry)?;
let vm = VM::try_new_with_universe_artifact(artifact.clone())?;
vm.run_compiled(&program)?;
```

or mechanically equivalent.

The compiler and VM must not independently reload the path.

Explicit missing/incompatible artifact errors fail without fallback.

Default no-option source behavior remains unchanged.

Full artifact build must use the full builder, not compiler-only builder + manually appended staged bytes.

### Forbidden approaches

- no implicit automatic cache/discovery;
- no second artifact load;
- no source fallback on explicit path;
- no changing default startup semantics;
- no broad CLI redesign.

### Test changes required

CV-20 plus CLI portions of CV-17/CV-18.

Tests:

- build command creates full capability artifact;
- explicit run works;
- compiler-only artifact passed to runtime option rejects;
- missing/corrupt explicit path fails;
- no-option path retains current behavior;
- internal API test verifies same `Arc` where accessible.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check -p phalcom-core --lib --bin phalcom
```

plus exact CLI smoke filters added by this task.

### Tests explicitly deferred

Two-VM isolation, full differential parity, determinism certification.

### Acceptance

Both explicit workflows exist; one loaded immutable artifact supplies compiler and VM; default behavior is unchanged.

### Local STOP / CONSULT triggers

- current compiler API requires a second incompatible image instance;
- CLI integration appears to require automatic cache policy or unrelated command architecture.

### Checkpoint update

Record public full build/load/runtime interfaces and G8.

---

## T9 — Certify isolation, recovery, determinism, diagnostics, and runtime parity

### Purpose

Establish the behavioral evidence required to treat the artifact as a portable executable Universe rather than merely a loadable data format.

### Preconditions

- T8 complete;
- all required focused functionality exists.

### Consumes

- source path as differential oracle;
- full artifact path;
- source-denial controls;
- test counters/observability;
- one loaded artifact shared into multiple VMs.

### Produces

- G9–G12 evidence;
- final coverage matrix.

### Files and symbols

**Read:**
- canonical artifact tests
- source-denial controls
- bootstrap counters/observability
- native/reflection/compiler regressions relevant to representative parity

**Modify:**
- focused integration tests
- test-only failpoints/counters where necessary
- no production architecture unless a real failure identifies a P3 defect

**Create:**
- separate test helpers/process runners if current test file becomes unwieldy

**Tests:**
- two-VM isolation;
- failure/retry;
- determinism;
- diagnostics;
- representative source/artifact parity.

### Required implementation shape

#### T9.1 Two-VM isolation

From one `Arc<LoadedUniverseArtifact>`:

```text
VM A != VM B runtime module objects
VM A != VM B source behavior classes
VM A != VM B method objects where mutable
VM A != VM B runtime caches/registries
artifact Arc shared
stable semantic identity equivalent
```

Mutate a safe test global/cache in A and prove B unchanged.

#### T9.2 Failure/retry

Test independently:

- malformed executable bytes → no artifact;
- validation proof failure → no artifact;
- materialization failpoint → constructor returns error and VM is discarded;
- initializer exception → current module failure semantics;
- same immutable artifact can construct a fresh VM afterward when failure type permits;
- one failed VM does not poison another VM.

Do not misclassify an ordinary source-language initializer exception as malformed artifact unless source path treats it that way.

#### T9.3 Determinism

Use separate fresh processes when practical.

Build full artifact twice from the same canonical source product under changed:

- working directory where safe;
- allocation/process history;
- prior VM activity.

Assert byte equality and digest equality.

#### T9.4 Diagnostics

Without canonical source text, representative runtime errors should still identify logical module/source-map locations from persisted diagnostic maps.

Do not require verbatim physical source excerpts if the artifact intentionally does not store source text; require correct logical identity/range information supported by the format.

#### T9.5 Representative differential parity

Use focused programs covering the runtime areas most likely to reveal reconstruction errors:

- primordial/native method calls;
- source-declared classes and class-side methods;
- closures/upvalues;
- enums/variants and `VariantMethod`;
- structural data;
- constants/ADTs;
- generic/trait/default dispatch;
- reflection/runtime type metadata;
- contracts if canonical bootstrap/runtime exposes them;
- eager initializer ordering;
- deferred module execution;
- exceptions/failure paths.

Compare observable output/result and required runtime identity facts, not raw heap addresses.

#### T9.6 Zero canonical rebuild counters

Fresh artifact process must prove zero canonical:

```text
source parse
interface/source read
semantic analysis/update
link
lower
AST-to-bytecode compilation
```

for canonical Universe.

User program compilation is expected and must not be counted as a failure.

### Forbidden approaches

- no broad test-suite pass substituted for focused evidence;
- no golden heap address comparison;
- no turning parity mismatches into source fallback;
- no suppressing runtime errors to make parity pass;
- no "artifact works because default source VM also exists in process" test.

### Test changes required

CV-13, CV-19, CV-21–CV-25 and remaining CV-24 matrix.

### Tests to run now

Run named G9–G12 tests individually first.

After they pass, run STABILIZE commands from §16.5.

### Tests explicitly deferred

Workspace-wide `cargo test` unless repository release policy explicitly requires it for checkpoint acceptance.

### Acceptance

All required coverage obligations are PASS or explicitly approved N/A; source-free execution, deferred code, VM isolation, recovery, determinism, and representative parity have direct evidence.

### Local STOP / CONSULT triggers

- parity failure reveals missing semantic authority rather than materialization bug;
- two-VM state necessarily aliases through shared mutable artifact objects;
- determinism failure traces to process-local identity in the wire format;
- useful diagnostics require a new source-storage contract.

### Checkpoint update

Mandatory after each coherent certification gate, not after each individual test.

---

## T10 — Close P3 lifecycle and hand off the next checkpoint slice

### Purpose

Record only verified completion, freeze stable interfaces/invariants, and make downstream work consume P3 without re-investigation.

### Preconditions

- all mandatory P3 gates complete;
- no unresolved blocking consultation;
- worktree state known.

### Consumes

- exact final revision;
- all gate results;
- accepted consultations/amendments;
- deferred unrelated failures.

### Produces

- `ARTF001.C1.P3-walkthrough.md`;
- `ARTF001.C1.P3-handoff.md`;
- updated checkpoint plan ledger;
- truthful P3 lifecycle metadata.

### Files and symbols

**Read:**
- checkpoint
- this plan
- test evidence/log summaries

**Modify:**
- `CHECKPOINT.md`
- P3 plan metadata if repository process records completion there

**Create:**
- walkthrough
- handoff

**Tests:**
- no new behavioral test solely for documentation

### Required implementation shape

Record:

- full build/load/runtime APIs;
- executable validation contract;
- exact allocation `N_full`;
- source-free eager and deferred proof;
- same-Arc compiler/runtime proof;
- two-VM isolation;
- recovery;
- determinism;
- representative parity;
- all tests actually run;
- all tests not run;
- unrelated failures and classifications.

Set P3 `COMPLETE / IMPLEMENTED` only if implementation is complete.

Set verification according to repository lifecycle definitions and actual evidence. Do not use `RELEASE_COMPLETE` unless its required gates were actually run.

Do not create or claim completion of P4 work here.

### Forbidden approaches

- no retrospective PASS for unrun tests;
- no hiding residual risk;
- no claiming workspace certification from focused gates;
- no creating a successor implementation plan unless explicitly requested.

### Tests to run now

Documentation/lint only if repository convention requires it:

```sh
git diff --check
```

### Tests explicitly deferred

Anything not part of final accepted P3 gate matrix.

### Acceptance

Lifecycle record matches reality; walkthrough/handoff are complete; downstream implementer has stable interfaces and no hidden executable/source fallback gap.

### Local STOP / CONSULT triggers

- a mandatory acceptance obligation is still unverified;
- a blocking failure is being proposed for deferral without architectural approval.

### Checkpoint update

Mandatory final update.

---

# 19. Verification gates

## G0 — P3 entry gate

Purpose: prove P3 is allowed to own executable completion.

Run: no Cargo command.

Expected:

```text
P2.R4 accepted by checkpoint lifecycle
one active executable plan: ARTF001.C1.P3
branch revision recorded
```

Do not begin authoritative runtime publication if this is false.

---

## G1 — Executable identity authorization

Purpose: prove the executable image is safe to authorize against current P2 identity.

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --lib artifact::validate -- --nocapture
```

Expected:

```text
exit 0
at least one test executed
real canonical captured image accepted
hostile identity/provenance cases rejected
```

Also:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib
git diff --check
```

Do not broaden after PASS unless G1 itself exposes an adjacent compile/test target that is directly affected.

---

## G2 — Full container authority

Purpose: prove exact compiler-only/full profiles and prepublication validation.

Run the owning executable codec test plus exact canonical-artifact full-profile tests.

Minimum commands:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-compiler-artifact --test executable_image_codec_test

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --lib canonical_artifact -- --nocapture
```

Expected:

```text
compiler-only round trip PASS
full round trip PASS
capability/section mismatch hostile cases PASS
corrupt executable fails before LoadedUniverseArtifact publication
```

---

## G3 — Exact full-load allocation gate

Purpose: preserve P2 exact transaction accounting through executable decode/proof/publication.

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --features allocation-diagnostics \
  --test artifact-allocation -- --nocapture --test-threads=1
```

Expected evidence:

```text
N_full discovered
ledger == independent allocator == N_full
exact N succeeds
N-1 typed failure
child rollback succeeds
prepublication failure returns no artifact
clean retry succeeds
```

The exact numeric N is recorded in CHECKPOINT.md and walkthrough.

---

## G4 — Materialization-without-execution gate

Purpose: prove relocation identity/rooting before bootstrap semantics are involved.

Run exact new materializer tests.

Expected:

```text
source behavior Class/Enum/Data objects not preallocated
initializer count == 0
same ConstantId shares within VM
separate VMs receive fresh mutable objects
primordial references map to kernel objects
GC/root stress PASS
```

---

## G5 — Source-free native-floor gate

Purpose: prove artifact-native setup has no provider/source edge.

Run exact new child-process source-denial native-floor test.

Expected:

```text
PHALCOM_DENY_SOURCE_BOOTSTRAP active before artifact VM setup
no source/provider denial fires
primordial/native identities parity checks PASS
initializer count remains at expected pre-eager value
```

---

## G6 — Source-free eager VM gate

Purpose: prove complete artifact bootstrap without canonical compilation.

Run exact fresh-process eager tracer.

Expected:

```text
artifact load PASS
VM::try_new_with_universe_artifact PASS
all stored eager modules execute in stored order
canonical source/provider reads == 0
canonical parse == 0
canonical semantic analysis == 0
canonical link/lower == 0
canonical AST→bytecode compilation == 0
Universe invariants PASS
```

---

## G7 — Deferred canonical initializer gate

Purpose: prove non-eager code is persisted and initialized once through existing module semantics.

Run exact deferred module test.

Expected:

```text
chosen module not executed during eager bootstrap
first dependency/use attaches stored initializer and executes once
dependencies initialized first
second use executes zero additional initializers
source denial remains active
missing stored initializer typed-fails
```

---

## G8 — One-artifact API/CLI gate

Purpose: prove explicit compiler/runtime workflow shares one loaded authority.

Run exact API and CLI smoke tests.

Expected:

```text
full build command PASS
explicit artifact run PASS
same loaded Arc used by compiler and VM in API test
compiler-only artifact rejected by VM
missing/corrupt explicit path fails without fallback
default no-option behavior unchanged
```

---

## G9 — Two-VM isolation gate

Purpose: prove immutable artifact sharing does not share mutable runtime state.

Expected:

```text
Arc<LoadedUniverseArtifact> shared
module/class/method/global/cache mutable state distinct
mutation in VM A does not affect VM B
both VMs preserve equivalent stable canonical identity
```

---

## G10 — Failure/retry gate

Purpose: prove failures do not poison artifact authority or unrelated fresh VMs.

Expected:

```text
bad bytes → no artifact
proof failure → no artifact
materialization failpoint → no VM publication
initializer failure → current sticky module failure semantics
fresh VM from same valid artifact can be created afterward when semantically permitted
```

---

## G11 — Deterministic full-build gate

Purpose: prove the full executable artifact contains no process-local identity.

Expected:

```text
fresh process build A bytes == build B bytes
artifact digest equal
working-directory/allocation-history perturbation does not change bytes
```

---

## G12 — Focused source/artifact parity gate

Purpose: prove representative runtime semantics are preserved.

Required representative categories:

```text
native/primordial
source class instance + class methods
closures/upvalues
enum + VariantMethod
data/structural runtime
constants/ADTs
generic/trait/default dispatch
reflection/type metadata
contracts where applicable
initializer ordering
deferred module
exception/failure path
diagnostic location
```

Expected: observable source-path and artifact-path behavior agrees for each selected case.

---

# 20. Final focused acceptance

At plan completion, fill this table with actual test names/results. No row may remain an implied PASS.

| Coverage ID | Test/evidence | Result |
|---|---|---|
| CV-01 | real canonical executable validation | PENDING |
| CV-02 | hostile Method identity matrix | PENDING |
| CV-03 | hostile VariantMethod matrix | PENDING |
| CV-04 | Class/Enum/Data/stable-constant producer tests | PENDING |
| CV-05 | repeated/cyclic identity tests | PENDING |
| CV-06 | scratch/local transfer tests | PENDING |
| CV-07 | provenance-loss tests | PENDING |
| CV-08 | exact full allocation gate | PENDING |
| CV-09 | no executable schema change / approved amendment | PENDING |
| CV-10 | same canonical product full builder | PENDING |
| CV-11 | no source-class preallocation | PENDING |
| CV-12 | within-VM constant identity / cross-VM freshness | PENDING |
| CV-13 | no runtime identity on wire / deterministic builds | PENDING |
| CV-14 | source-free native floor | PENDING |
| CV-15 | eager source-free VM | PENDING |
| CV-16 | deferred source-free initializer | PENDING |
| CV-17 | compiler-only artifact VM rejection | PENDING |
| CV-18 | capability authorization | PENDING |
| CV-19 | corrupt executable before initializer execution | PENDING |
| CV-20 | one-Arc compiler/runtime workflow | PENDING |
| CV-21 | two-VM isolation | PENDING |
| CV-22 | failure/retry | PENDING |
| CV-23 | source-free diagnostics | PENDING |
| CV-24 | representative source/artifact parity | PENDING |
| CV-25 | deterministic full publication | PENDING |

Run only P3's required stabilization gates after the table is otherwise complete.

Do not automatically run workspace certification.

---

## 21. Performance/resource evidence

P3 has a **resource/soundness** requirement, not a speed benchmark acceptance target.

Required resource evidence:

1. exact full-load allocation ledger `N_full`;
2. independent allocator equality;
3. exact `N_full` success;
4. `N_full - 1` typed failure;
5. child transaction rollback;
6. clean retry;
7. bounded proof limits for CFG edges, reachable states, provenance facts, and worklist steps;
8. no unbounded candidate-set lattice;
9. full artifact byte size recorded for comparison with compiler-only artifact;
10. optional stage timings may be recorded for:
    - file read/container validation;
    - compiler section import;
    - executable decode;
    - executable proof;
    - per-VM relocation;
    - native floor;
    - eager initialization.

No performance regression/speedup claim is required to close P3 unless the checkpoint separately mandates one. Do not add benchmark ceremony solely because PERF003 was the historical source of executable code.

---

## 22. Checkpoint bookkeeping

### 22.1 At plan start

- record P2 acceptance receipt;
- mark P3 active;
- record branch/revision;
- record inherited P8 executable code as takeover state.

### 22.2 During plan

Update only for:

- G1 executable authority contract;
- schema/ABI consultation or explicit "no schema change" result when durable;
- G2 full profile API;
- G3 exact allocation evidence;
- G5 source-free native floor;
- G6 first source-free eager VM;
- G7 deferred source-free execution;
- G8 public one-artifact workflow;
- G9–G12 coherent certification results;
- unrelated/deferred failure that affects later work;
- any approved architectural amendment.

Do not write a checkpoint entry after every compile.

### 22.3 At plan completion

- update plans ledger;
- record completion/verification honestly;
- record final public/private interfaces;
- record exact allocation N;
- record final invariants;
- record consultations/amendments;
- record deferred failures;
- set next action without inventing a successor plan.

---

## 23. Walkthrough deliverable

Create:

```text
ARTF001.C1.P3-walkthrough.md
```

Must contain:

- final result;
- branch/revision range;
- architecture implemented;
- executable validator/proof model;
- full artifact build/load profile;
- allocation accounting result;
- materializer architecture;
- native-floor cutover;
- eager/deferred runtime behavior;
- CLI/API workflow;
- important code changes by file/symbol;
- deviations from this plan;
- consultations and adopted rulings;
- tests added;
- exact tests actually run and results;
- tests deliberately deferred;
- verification classification;
- residual risks;
- artifact format/API compatibility statement.

---

## 24. Handoff deliverable

Create:

```text
ARTF001.C1.P3-handoff.md
```

Must contain:

- current branch/revision;
- P3 lifecycle state;
- inherited stable interfaces:
  - full builder;
  - full loader;
  - `ValidatedExecutableImage` authority;
  - VM artifact constructor;
  - same-artifact compiler API;
  - deferred initializer seam;
- final invariants;
- exact `N_full`;
- next checkpoint objective;
- must-read files;
- deferred/unrelated failures;
- first recommended commands;
- things not to redesign/re-explore:
  - semantic authority;
  - executable identity proof;
  - native descriptor ownership;
  - module initialization state;
  - no-source-fallback rule;
  - per-VM relocation model.

---

## 25. Completion truth table

Do not conflate:

```text
source written
tests added
targeted tests passed
affected suite passed
source-free eager VM proved
deferred module proved
two-VM isolation proved
determinism proved
checkpoint plan accepted
workspace/release certified
```

Recommended lifecycle interpretation:

| Evidence | Completion | Verification |
|---|---|---|
| code partially written, no complete VM | PARTIAL | UNVERIFIED or FOCUSED_TESTED |
| implementation complete, focused gates incomplete | IMPLEMENTED | UNVERIFIED/FOCUSED_TESTED as repository definitions allow |
| all P3 focused gates pass | IMPLEMENTED | FOCUSED_TESTED |
| broader release policy also run and passed | IMPLEMENTED | RELEASE_COMPLETE only if repository definition is truly satisfied |

Do not mark P3 COMPLETE solely because `cargo check` passes.

---

## 26. Plan self-review

Before delivery/implementation use, verify:

- [x] Every invariant has implementation ownership.
- [x] Every important identity/soundness risk has focused coverage.
- [x] Runtime class identity and semantic callable owner are not conflated.
- [x] Class, Enum, Data, and stable class Constant are included in behavior producer analysis.
- [x] VariantMethod has distinct runtime postconditions.
- [x] Scratch locals reuse frame semantics.
- [x] No duplicate stack/control-flow authority is planned.
- [x] Validation scratch is budget/accounting aware.
- [x] No duplicate semantic ontology is planned.
- [x] No heap snapshot or build-VM identity is planned.
- [x] Source-declared behavior classes remain execution-time.
- [x] Native descriptor authority remains singular.
- [x] Artifact VM begins from kernel, not provider-backed native bootstrap.
- [x] Existing module initialization state machine remains authoritative.
- [x] Deferred initialization has no source fallback.
- [x] Same loaded artifact supplies compiler and runtime.
- [x] Separate VMs share no mutable runtime state.
- [x] Compiler-only artifact semantics are preserved.
- [x] Full capability is validation-gated.
- [x] Luna has objective STOP/CONSULT triggers.
- [x] Debugging loops are bounded.
- [x] Testing budget is intentionally narrow during BUILD.
- [x] Broad tests are not mandatory after every task.
- [x] Exact allocation certification is a named gate.
- [x] Checkpoint/walkthrough/handoff obligations exist.
- [x] Tasks form a valid dependency order:
      `T0 → T1 → T2 → T3 → T4 → T5 → T6 → T7 → T8 → T9 → T10`.
- [x] No implementation placeholder remains; only future result cells are intentionally `PENDING`.
- [x] Public/proposed private signatures are internally consistent and marked mechanically flexible where appropriate.

---

# Implementation dependency summary

```text
T0  P2/P3 lifecycle entry
 │
 ▼
T1  bounded executable identity proof
 │
 ▼
G1
 │
 ▼
T2  full container build/load authority
 │
 ▼
G2
 │
 ▼
T3  exact executable load allocation
 │
 ▼
G3
 │
 ▼
T4  per-VM recipe materialization
 │
 ▼
G4
 │
 ▼
T5  source-free native/artifact floor
 │
 ▼
G5
 │
 ▼
T6  source-free eager VM
 │
 ▼
G6
 │
 ▼
T7  source-free deferred modules
 │
 ▼
G7
 │
 ▼
T8  one-artifact compiler/runtime CLI
 │
 ▼
G8
 │
 ▼
T9  isolation/recovery/determinism/parity
 │
 ├── G9
 │   G10
 │   G11
 │   G12
 ▼
T10 lifecycle closure + handoff
```

# Final executor directive

Do not optimize for finishing many tasks in one turn. Finish one coherent slice, run only the gate that discriminates that slice, update durable state when warranted, and stop at the first material architecture/authority conflict.

The critical path is not "make serialized bytecode execute." It is:

```text
prove exact executable identity
→ publish authority atomically
→ reconstruct fresh VM-local recipes
→ remove source/provider dependency from artifact bootstrap
→ reuse existing runtime initialization semantics
→ prove source-free eager + deferred execution
→ prove isolation, recovery, and determinism
```

Any implementation that skips one of those authority boundaries is not an implementation of `ARTF001.C1.P3`.
