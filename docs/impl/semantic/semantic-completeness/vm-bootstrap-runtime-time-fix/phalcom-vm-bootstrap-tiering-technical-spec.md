# Phalcom VM Bootstrap Tiering and Shared Canonical Universe Compilation
## Technical Specification

**Status:** Proposed  
**Scope:** Short-term runtime/test architecture correction; production-quality, non-disposable  
**Repository:** `aureat/phalcom-lang`  
**Prepared against remote branch:** `main`  
**Prepared against remote revision:** `9f04681201e4e15388b4a32d09a2a502486e9367` (`feat: extend semantic type-system closure`)  
**Local working tree:** Not visible from the repository inspection environment; local uncommitted changes are therefore unknown.  
**Pinned Rust toolchain:** `nightly-2026-07-10`

---

## 1. Executive summary

Phalcom currently conflates three materially different operations inside `VM::new()`:

1. creating a fresh VM execution kernel;
2. materializing and installing the native runtime/Universe floor;
3. compiling and executing the source-authored canonical Universe.

The third operation currently performs a source-complete whole-Universe semantic analysis, projects semantic lowering, compiles source modules into VM-local closures, and executes their initializers. The source-complete analysis restored required canonical identities such as `Universe::errors::result::Result::Ok`, but it also moved an expensive compiler operation onto every `VM::new()` path.

That architecture causes two forms of unnecessary work:

- every full VM instance repeats process-invariant compiler work;
- low-level tests and consumers that need only heap/kernel/native behavior nevertheless pay for the entire source-authored Universe.

This specification separates those responsibilities without changing the language model and without introducing the planned future persisted project-artifact architecture.

The correction has two central pieces:

1. **VM bootstrap becomes explicitly tiered.**
   - `VM::new_kernel()` creates only fresh VM-local kernel state.
   - `VM::new_native()` adds the canonical module/native runtime floor but does not semantically analyze or execute Universe `.ph` source.
   - `VM::new()` remains the shipping, fully bootstrapped VM and preserves existing observable behavior.

2. **Canonical Universe compiler work becomes process-shared immutable state.**
   - Canonical source parsing/interface derivation continues to use the caches already owned by `phalcom-modules`.
   - Linking, source/native contract verification, whole-Universe semantic analysis, lowering projection, and bootstrap-order derivation are computed once per process/test binary.
   - Each full VM still receives a fresh `Heap`, runtime modules, classes, globals, fibers, closures, caches, and source execution state.
   - No mutable VM is shared between tests.

The target runtime relationship is:

```text
                         PROCESS-IMMUTABLE
              ┌─────────────────────────────────┐
              │ Universe source/provider caches │
              │ canonical linked program        │
              │ source/native verification      │
              │ semantic lowering projections   │
              │ bootstrap execution order       │
              └────────────────┬────────────────┘
                               │
                shared read-only compiler product
                               │
             ┌─────────────────┼─────────────────┐
             ▼                 ▼                 ▼
        fresh VM #1       fresh VM #2       fresh VM #3
        fresh Heap        fresh Heap        fresh Heap
        fresh ObjRefs     fresh ObjRefs     fresh ObjRefs
        fresh globals     fresh globals     fresh globals
        fresh closures    fresh closures    fresh closures
        fresh caches      fresh caches      fresh caches
```

This is an architectural improvement rather than a test-only shortcut. It establishes the same ownership boundary required by the later compiled-project artifact design: compiler-derived immutable products are distinct from VM-local materialization and execution state.

---

## 2. Motivation and observed regression

### 2.1 The correctness change that exposed the architectural problem

The canonical Universe lowering repair changed bootstrap from source-incomplete per-module semantic analysis to one source-complete semantic analysis over the linked canonical Universe corpus.

That change was semantically necessary. The runtime compiler needs exact canonical identities for source-authored ADTs, associated families, matches, and other lowering products. In particular, `Result::Ok` and `Result::Error` must retain their authored owner:

```text
ProjectIdentity::Universe
  → universe::errors::result
  → Result
  → Ok / Error
```

rather than falling back to an approximate or synthesized identity.

The problem is not whole-Universe analysis itself. The problem is its current lifecycle:

```text
VM::new()
  → build/inspect canonical source
  → resolve/link Universe
  → SemanticWorkspaceSession::new()
  → analyze whole Universe
  → project lowering
  → compile source bodies
  → execute source bodies
```

Every fresh VM repeats that flow.

Repository work logs already recorded approximately 55 seconds for debug Universe bootstrap after the generic-inference recursion bug was repaired. Earlier focused core regressions recorded approximately 71 seconds for tests whose meaningful body was otherwise small. The expensive operation is therefore multiplicative across tests that construct VMs.

### 2.2 Why this is especially harmful in the current test architecture

`phalcom-core` exposes one broad integration test binary (`--test core`) combining:

- Either;
- monads;
- language behavior;
- compiler projection;
- module/linking/runtime tests;
- native contracts;
- object-model invariants;
- reflection;
- collections;
- execution;
- memory/GC;
- observability;
- REPL behavior.

A process-local immutable canonical product is therefore particularly valuable: one expensive derivation can serve many independently isolated test VMs in the same binary.

At the same time, low-level tests such as heap/product/cache/native-function tests often do not require source-authored Universe behavior at all. Their dependency on full `VM::new()` is accidental.

---

## 3. Repository architecture and current ownership

### 3.1 VM-local kernel and execution state

Primary owner:

- `phalcom-core/src/vm/mod.rs` — `VM`
- `phalcom-core/src/universe/mod.rs` — `Universe`
- `phalcom-core/src/vm/bootstrap.rs` — current VM construction/bootstrap

`VM` owns mutable execution state including:

- `Heap`;
- call frames and operand stack;
- root/current fiber;
- module registry;
- runtime typing registry;
- reflection cache;
- runtime roots;
- class maps and field layouts;
- prelude bindings;
- resources;
- runtime ADT registry;
- dispatch caches/world version;
- `Universe`, which stores VM-local `ClassId` handles.

`Universe::new(&mut Heap)` constructs the kernel class tower and typing classes. This is inherently VM-local because its `ClassId`s point into that VM's heap.

### 3.2 Canonical Universe source and interfaces

Primary owner:

- `phalcom-modules/src/builtin.rs` — `UniverseSourceProvider`
- `phalcom-modules/src/builtin_interface.rs` — `BuiltinInterfaceBuilder`

The repository already has process-wide caches for canonical builtin source products:

```rust
static BUILTIN_PARSED_CACHE: OnceLock<...>;
static BUILTIN_INTERFACE_CACHE: OnceLock<...>;
```

Therefore this program **must not** introduce a competing parsed-source or unlinked-interface cache in `phalcom-core`.

The source/provider layer remains authoritative for:

- canonical Universe module discovery;
- canonical source text and `SourceId`;
- parsed `ParsedModuleUnit`;
- source-derived unlinked interfaces.

### 3.3 Native source census and source/native conformance

Primary owner:

- `phalcom-core/src/native/source.rs` — `NativeSourceIndex` / `UniverseSourceIndex`
- `phalcom-core/src/native/verify.rs` — `verify_native_contracts`

`NativeSourceIndex` is already explicitly VM-free. It retains:

- `Vec<Arc<ParsedModuleUnit>>`;
- source census;
- native class/member anchors;
- canonical presentations.

The index currently derives bootstrap roots and dependency order as well.

### 3.4 Linking and semantic analysis

Primary owners:

- `phalcom-modules` — `ModuleResolver`, `ModuleLinker`, `LinkedProgram`
- `phalcom-semantic/src/workspace.rs` — `analyze_workspace`
- `phalcom-semantic/src/session.rs` — `SemanticWorkspaceSession`

`analyze_workspace` creates a fresh `SemanticWorkspaceSession` for every call. The session owns a `SemanticDb`, `TypeStore`, and other store/session-relative semantic state.

This specification deliberately does **not** define a process-global general semantic session or a process-global raw `SemanticSnapshot` baseline.

### 3.5 Compiler-facing semantic projection

Primary owner:

- `phalcom-core/src/modules/semantic_lowering.rs` — `ModuleLoweringSemantics`

`ModuleLoweringSemantics` is a compact immutable backend-facing projection containing stable semantic identities and source sites such as:

- `DeclarationId`;
- `CallableId`;
- `VariantId`;
- `SourceId`;
- `SourceRange`;
- executable family and associated targets;
- match lowering;
- enum/variant layouts.

The projection may consult store-local `TypeId`s while it is being built, but the resulting product is the appropriate boundary for sharing with runtime compilation.

### 3.6 Existing VM-independent program representation

Primary owner:

- `phalcom-core/src/modules/compile.rs`

The repository already defines:

```text
AnalyzedProgram
CompiledModule
CompiledProgram
ProgramAnalyzer
ProgramCompiler
```

`CompiledModule` already carries:

- `ModuleId`;
- module kind and source;
- retained `Arc<str>` source text;
- linked module interface;
- module materialization plan;
- symbolic linked reads;
- `Arc<ModuleLoweringSemantics>`.

`CompiledProgram` already carries:

- `Arc<ProjectUniverse>`;
- `Arc<LinkedProgram>`;
- compiled modules;
- entry;
- initialization order;
- optional semantic metadata.

The short-term canonical Universe shared product should reuse this representation instead of creating a second parallel compiled-module graph.

### 3.7 Runtime materialization

Primary owners:

- `phalcom-core/src/modules/builtin_materialize.rs` — canonical Universe module/native materialization
- `phalcom-core/src/modules/materialize.rs` — general `CompiledProgram` materialization
- `phalcom-core/src/modules/initialize.rs` — runtime module initialization

Canonical Universe materialization allocates VM-local `ModuleObject`s and native globals. These products contain `ObjRef`s and therefore **cannot** be shared between VMs.

---

## 4. Goals

### G-01 — Remove process-invariant compiler work from repeated VM construction

A process must perform canonical Universe:

- source/native verification;
- complete import resolution/linking;
- whole-source semantic analysis;
- lowering projection;
- bootstrap dependency/order derivation;

at most once for a given loaded binary/process.

### G-02 — Preserve fresh mutable VM isolation

Every `VM` must continue to own fresh:

- heap;
- `ClassId`/`ObjRef` namespace;
- module objects;
- globals;
- closures;
- dispatch caches;
- fiber state;
- resources;
- reflection runtime objects;
- runtime typing registrations;
- ADT runtime registrations.

No mutable VM state may be process-shared.

### G-03 — Make VM construction granular

Consumers must be able to request the minimum runtime capability required:

```text
Kernel
Native runtime
Full source-authored Universe
```

The default `VM::new()` remains full and behavior-compatible.

### G-04 — Permit tests that do not need Universe source semantics to avoid them completely

A kernel-only consumer must not:

- build the canonical Universe source index;
- invoke semantic analysis;
- link canonical Universe source;
- compile Universe `.ph` bodies;
- execute Universe `.ph` bodies.

A native-runtime consumer must not invoke whole-Universe semantic analysis or source-body execution.

### G-05 — Keep semantic capability honest

A partially bootstrapped VM must never carry fake placeholder values that make it appear fully bootstrapped.

In particular, canonical source-derived `unsupported`, `ellipsis`, and `Ordering` roots must have explicit availability.

### G-06 — Reuse existing authoritative abstractions

The implementation must reuse:

- `UniverseSourceProvider`;
- `BuiltinInterfaceBuilder` caches;
- `NativeSourceIndex`;
- `LinkedProgram`;
- `CompiledModule` / `CompiledProgram`;
- `ModuleLoweringSemantics`.

It must not introduce redundant caches or alternate module/semantic identities.

### G-07 — Preserve canonical ADT/GADT/associated lowering correctness

The source-complete semantic repair remains intact. There is no return to per-module source-incomplete analysis.

### G-08 — Improve evidence and performance diagnosis

The architecture must make it easy to prove:

- which bootstrap stage a test requested;
- that the canonical compiler product is process-shared;
- that kernel/native tiers do not accidentally trigger semantic Universe compilation;
- that full VMs remain isolated.

---

## 5. Non-goals

This program does **not** implement:

1. build-time embedded Universe images;
2. serialized project artifacts;
3. compiled user-project artifacts;
4. precompiled/relocatable Universe bytecode templates;
5. VM heap snapshots or cloning;
6. copy-on-write initialized VMs;
7. a process-global general `SemanticWorkspaceSession`;
8. a process-global general semantic `TypeStore`;
9. narrowing or changing current Universe bootstrap-root semantics;
10. lazy Universe module execution;
11. a redesign of `ModuleObject::sources` from `Arc<String>` to `Arc<str>`;
12. a general migration of Universe materialization onto `VM::materialize_program`;
13. a new module resolver;
14. generic-inference algorithm optimization;
15. Cargo/rustc threading or build-profile tuning;
16. repairing repeated `run_compiled()` compilation — current HEAD already skips initialized/failed modules before compiling missing initializers.

The future project-artifact architecture remains compatible with this work but is intentionally deferred.

---

## 6. Terminology and bootstrap capability model

### 6.1 Kernel VM

A **kernel VM** is a fresh execution machine containing only the runtime substrate required for heap/class/value/compiler mechanics that do not depend on source-authored Universe behavior.

It includes:

- `Heap`;
- `Universe::new(&mut heap)` kernel class/typing rows;
- root fiber;
- stacks;
- registries and caches initialized empty;
- VM configuration fields.

It does **not** include:

- canonical Universe module objects;
- canonical native bindings;
- installed native primitive methods;
- source-derived prelude;
- source-derived semantic roots;
- Universe semantic analysis;
- Universe `.ph` execution.

### 6.2 Native VM

A **native VM** extends a kernel VM with the canonical native runtime floor:

- canonical Universe package/module objects;
- canonical native bindings;
- primordial class bindings;
- required hard-coded native layouts;
- registered native primitive methods;
- typing reflection native installation;
- primordial base-name finalization;
- native/prelude aliases that can be established without source-body execution.

It does **not** execute canonical Universe `.ph` bodies and does not expose source-derived semantic roots.

This tier is intended for:

- direct primitive tests;
- heap/object-model/native dispatch tests;
- compiler/runtime tests whose behavior is satisfied by the native floor.

It is not a substitute for the shipping runtime.

### 6.3 Full VM

A **full VM** is the shipping runtime and remains the meaning of:

```rust
VM::new()
```

It extends a native VM by:

- consuming the process-shared canonical Universe compiler product;
- attaching canonical lowering;
- compiling required `.ph` module bodies into fresh VM-local closures;
- executing the existing bootstrap initialization closure;
- resynchronizing source-authored prelude/class aliases;
- installing canonical source-derived semantic roots;
- finalizing post-bootstrap pristine flags;
- running current full bootstrap invariants.

### 6.4 Capability is structural, not a duplicate string/enum authority

The implementation must not add an independent mutable `vm.bootstrap_level` field whose value can drift from actual state.

Capability is represented by concrete state:

- module/runtime roots exist or do not;
- source-derived semantic roots exist or do not;
- module registry states record initialization.

Constructors define valid transitions; they do not publish a second authority that merely names the transition.

---

## 7. Proposed process-shared canonical compiler product

### 7.1 New responsibility

Introduce a compiler/modules-owned canonical Universe compilation product, provisionally named:

```rust
CanonicalUniverseProgram
```

The exact type name may be reconciled with repository naming during implementation, but ownership belongs under:

```text
phalcom-core/src/modules/
```

not under the VM.

### 7.2 Required shape

Structurally:

```rust
pub struct CanonicalUniverseProgram {
    /// Existing VM-independent compiler representation.
    pub program: Arc<CompiledProgram>,

    /// Complete canonical pre-parsed source corpus used by the current
    /// VM-coupled AST compiler. Parsing itself remains owned/cached by
    /// phalcom-modules.
    pub source_index: Arc<NativeSourceIndex>,

    /// Canonical root-only dependency closure for measurement/diagnostics.
    pub root_reachable: Arc<[ModuleId]>,

    /// Existing eager-bootstrap semantics, precomputed once.
    pub bootstrap_order: Arc<[ModuleId]>,
}
```

This is a **STRUCTURAL** shape, not a requirement to duplicate fields already cheaply obtainable from an existing authoritative product.

If implementation can represent `root_reachable` or `bootstrap_order` through an existing immutable graph without recomputation, it should do so.

### 7.3 Shared storage

Use a process-local one-time immutable cell:

```rust
static CANONICAL_UNIVERSE_PROGRAM: OnceLock<Result<CanonicalUniverseProgram, ...>>;
```

Important properties:

- initialization is thread-safe;
- successful construction is immutable;
- failure is memoized as data, not implemented as a panic inside `get_or_init`;
- multiple concurrent `VM::new()` calls cannot perform duplicate semantic analyses;
- no `unsafe impl Send`/`Sync` is permitted to force a non-thread-safe product into the cache.

If a nested type is not `Send + Sync`, implementation must identify the exact non-shareable field and move the cache boundary inward rather than using unsafe trait assertions.

### 7.4 Builder flow

The builder performs the compiler-side work currently hidden in VM bootstrap:

```text
UniverseSourceProvider / BuiltinInterfaceBuilder caches
        ↓
NativeSourceIndex
        ↓
source/native contract verification
        ↓
resolve all canonical imports
        ↓
ModuleLinker / LinkedProgram
        ↓
one source-complete analyze_workspace(...)
        ↓
transient SemanticSnapshot
        ↓
ModuleLoweringSemantics projection
        ↓
CompiledModule / CompiledProgram
        ↓
CanonicalUniverseProgram
```

### 7.5 Semantic snapshot lifetime

The builder may create a `SemanticSnapshot` while deriving canonical products.

The shared runtime product should **not** retain that snapshot merely because it was available.

Reason:

- `SemanticWorkspaceSession` owns a session-local `TypeStore`;
- raw `TypeId` values are meaningful relative to that store;
- the repository's own semantic architecture warns against globalizing store-local semantic IDs without also defining a shared immutable store authority.

The runtime needs the projected lowering, not the complete semantic database.

If a future compiler/LSP optimization wants a reusable semantic Universe baseline, that is a separate semantic-architecture program.

### 7.6 Diagnostic policy must preserve current behavior

`ProgramCompiler::compile_analyzed` is an attractive reuse seam, but it currently rejects an analyzed program if its semantic snapshot contains errors and also builds runtime semantic metadata.

The current VM `universe_lowerings` path directly projects lowering after analysis.

Therefore implementation must first characterize the current canonical Universe analysis result:

- if the canonical snapshot is diagnostic-clean, using `ProgramCompiler::compile_analyzed` is behavior-equivalent and preferred;
- if not, do not silently add a new fatal diagnostic boundary as part of this performance work. Reuse/extract the `CompiledModule` projection path while preserving the current bootstrap acceptance contract.

The source of truth is current full bootstrap semantics, not convenience of one compiler helper.

---

## 8. VM construction decomposition

### 8.1 Public constructors

Required public behavior:

```rust
impl VM {
    /// Fresh VM execution kernel only.
    pub fn new_kernel() -> Self;

    /// Fresh VM with canonical native/runtime floor but without source
    /// Universe semantic compilation/execution.
    pub fn new_native() -> Self;

    /// Existing shipping behavior: fully bootstrapped Universe.
    pub fn new() -> Self;
}
```

`new_with_native_install_mode(...)` remains behavior-compatible and produces a full VM. `NativeInstallMode` currently aliases descriptor-only behavior and is not redesigned by this program.

### 8.2 Internal composition

Implementation should factor bootstrap into named internal phase methods rather than copying constructor bodies:

```text
allocate_kernel
    ↓
install_native_runtime
    ↓
bootstrap_source_universe(shared_program)
    ↓
finalize_full_bootstrap
```

The exact function names are not normative. The phase ownership is.

### 8.3 Kernel stage

Move the allocation/state-construction prefix of current `new_with_native_install_mode` into the kernel constructor.

Kernel stage must not call:

- `NativeSourceIndex::build`;
- `verify_native_contracts`;
- `initialize_canonical_universe`;
- `canonical_universe_program`;
- `analyze_workspace`;
- `run_universe_modules`.

### 8.4 Native stage

Native stage may call existing VM-local native/materialization facilities:

- `initialize_canonical_universe`;
- `bind_primordial_universe`;
- native field/layout stamps;
- `install_registered_primitives`;
- typing primitive installation;
- `finalize_all_primordial_base_names`;
- native alias/prelude synchronization appropriate before source execution.

Native stage must not call the canonical semantic program accessor.

This is important: a direct primitive test should not implicitly become a compiler benchmark.

### 8.5 Full stage

Full stage obtains:

```rust
canonical_universe_program()
```

and performs only VM-local work:

- identify the precomputed bootstrap module order;
- attach each module's precomputed `ModuleLoweringSemantics`;
- use retained parsed AST/source to compile the module into this VM;
- run the closure in this VM;
- mark runtime module state initialized;
- synchronize source-authored class/prelude state;
- populate source-derived semantic roots;
- run post-source bootstrap finalization/invariants.

---

## 9. Explicit source-derived semantic roots

### 9.1 Current defect

Current VM construction seeds:

```text
unsupported    = Value::nil()
ellipsis       = Value::nil()
ordering_class = ClassId::default()
```

and later overwrites them after Universe source execution.

Those placeholders are tolerable only while they are transient internal construction details. Once a kernel/native VM becomes a supported long-lived state, they become false semantic identities.

### 9.2 Required representation

Change VM ownership to express absence:

```rust
semantic_roots: Option<SemanticRoots>
```

or a semantically equivalent explicit capability type.

Do not use:

- `Value::nil()` as “not initialized”;
- `ClassId::default()` as “not initialized”;
- a boolean flag paired with invalid values.

### 9.3 Consumer migration

Current consumers include:

- numeric/int primitives returning canonical `unsupported`;
- dispatch of `GetEllipsis`;
- ordering checks;
- GC root enumeration.

Required behavior:

- full VMs read the exact canonical source-derived values exactly as today;
- GC traces these values only when present;
- a lower-tier VM encountering a source-dependent operation fails explicitly at the ownership boundary rather than returning the private Nil sentinel or interpreting `ClassId::default()` as real.

Prefer centralized accessors where mutation/error conversion is needed, for example:

```rust
fn semantic_roots(&self) -> Result<&SemanticRoots, RuntimeError>
```

The exact error variant should reuse an existing internal/bootstrap precondition error if one exists. Do not invent a user-facing language error merely to represent an invalid internal test configuration.

---

## 10. Source/native contract verification lifecycle

`verify_native_contracts` is a compiler/source-conformance invariant, not a property that changes from one VM heap to another.

Therefore:

- remove it from repeated full VM construction;
- execute it once while deriving `CanonicalUniverseProgram`;
- retain dedicated native/source contract tests at their ownership layer.

`VM::new_native()` intentionally represents a runtime floor without source-body conformance compilation. It must not be marketed as a complete validated Phalcom runtime. `VM::new()` remains the validated shipping path.

This keeps low-level native tests independent of whole-Universe semantic analysis without deleting conformance checks.

---

## 11. Bootstrap ordering and eager roots

### 11.1 Current behavior is preserved

`NativeSourceIndex::bootstrap_roots()` currently treats the Universe root and most units containing source declarations/statements as eager roots.

This heuristic likely executes more canonical modules than a future optimized runtime needs.

However changing it can alter:

- which class reopens run at startup;
- which globals exist before import;
- prelude population;
- reflection state;
- method installation;
- semantic roots.

Therefore narrowing eager roots is **not** part of this program.

### 11.2 What this program does optimize

The existing eager-root set and dependency closure are:

- computed once;
- retained in the shared canonical product;
- reused by every full VM.

This removes repeated graph work while holding runtime semantics constant.

### 11.3 Future follow-up

After this program lands and measurements identify source execution as the next material cost, a separate specification may define reachable/lazy Universe initialization with explicit source-initialization semantics.

---

## 12. Runtime compilation remains VM-local in this program

Even with shared lowering, current AST-to-bytecode compilation allocates VM-local runtime structures and uses the VM interner/heap.

This program therefore does not share:

- compiled closure `ObjRef`s;
- populated `Chunk`s;
- `Value` constant pools containing heap references;
- `ClassId`s;
- inline/global caches.

Each full VM still compiles bootstrap source bodies using:

- shared parsed ASTs;
- shared linked bindings/lowering;
- fresh VM-local compiler/runtime state.

If measurements after this work show that AST-to-bytecode compilation is still material, the next step is an immutable relocatable compiled-template product. That is intentionally deferred.

---

## 13. Test-runtime architecture

### 13.1 Test tiers

Tests should explicitly request the least runtime capability that proves their invariant.

| Tier | Constructor | Intended tests | Must not pay for |
|---|---|---|---|
| No VM | none | parser, modules, semantic, pure compiler products | heap/runtime |
| Kernel | `VM::new_kernel()` | heap, Value, structural class/module mechanics, low-level caches | native floor, Universe source |
| Native | `VM::new_native()` | direct native primitives, native dispatch/floor, kernel/native ownership | semantic Universe analysis, `.ph` bootstrap |
| Full | `VM::new()` | language, stdlib, prelude, reflection, ADT/GADT runtime, module integration | repeated process-invariant compiler work |

### 13.2 Shared test helper ownership

Extend the existing:

```text
phalcom-core/tests/support/vm.rs
```

rather than adding a second fixture framework.

Suggested domain-neutral helpers:

```rust
pub(crate) fn kernel_vm() -> VM;
pub(crate) fn native_vm() -> VM;
pub(crate) fn universe_vm() -> VM;
```

Existing `run_inline` / `compile_inline` should remain full by default until individual call sites prove they can run correctly on a lower tier. Source-language helpers must not silently downgrade semantics.

### 13.3 No shared mutable VM fixture

Forbidden:

```text
static VM: Mutex<VM>
once_cell VM clone baseline
one global heap reused across tests
```

Tests must remain isolated. Sharing is limited to immutable compiler products.

### 13.4 Migration policy

Do not mechanically replace every `VM::new()`.

Good downgrade candidates include tests whose assertions are solely about:

- `Heap`;
- `Value`;
- product normalization;
- `ModuleObject` mechanics;
- class-tower structure;
- direct Rust native primitive calls;
- inline/global cache mechanics independent of source-authored methods.

Tests must stay full when they assert:

- source prelude behavior;
- source-authored `List`, `Option`, `Result`, `Either`, monad behavior;
- source reflection;
- canonical source enum/variant identities;
- source module initialization;
- traceback frames through source-authored Universe code;
- semantics of source-installed methods.

---

## 14. Concurrency and thread safety

The canonical product is process-shared and may be requested concurrently by Rust tests.

Requirements:

1. one-time initialization is thread-safe;
2. the product is immutable after publication;
3. failures are stable/memoized;
4. no mutable semantic session is retained;
5. no VM handle is retained;
6. no runtime cache `Cell`/`RefCell` is introduced into shared compiler state;
7. do not hold a coarse mutex during every read after initialization.

`OnceLock<Result<...>>` or an equivalent standard-library one-time cell is preferred.

---

## 15. Failure semantics

### 15.1 Canonical build failure

Today a broken canonical Universe causes `VM::new()` to panic through `expect`.

This program does not need to redesign public fallible VM construction, but the shared builder should itself be fallible and memoize its result.

Full `VM::new()` may continue to convert the shared error into the existing fatal bootstrap `expect` boundary.

### 15.2 Lower-tier misuse

Kernel/native tier misuse of a full-Universe-only capability must fail locally and explicitly.

Examples:

- requesting `ellipsis` bytecode semantics without semantic roots;
- numeric cooperative operation needing canonical `Unsupported` without full source roots.

Do not silently upgrade the VM to full bootstrap. Hidden lazy escalation would reintroduce unpredictable compiler work and destroy the test-tier contract.

### 15.3 No fallback identity restoration

If shared canonical lowering construction fails, do not restore approximate compiler/runtime fallbacks for `Result`, variants, associated families, or other canonical identities.

The correct response is to fix the shared canonical product.

---

## 16. Observability and performance evidence

### 16.1 Correctness counters over timing assertions

Do not make wall-clock thresholds primary correctness tests.

Architectural tests should prove:

- canonical shared product accessor returns the same immutable instance across repeated calls;
- lower tiers have no source-derived semantic roots;
- full tier has source-derived semantic roots;
- VM-local state remains isolated.

### 16.2 Bootstrap measurement

Continue using `UniverseBootstrapMeasurement`, but clarify semantics:

- `discovered_units`: canonical source catalog size;
- `root_reachable_units`: root dependency closure;
- `executed_units`: source modules executed in this VM.

For kernel/native VMs, source-execution measurement is zero/default.

For full VMs, counts are derived from the shared plan but execution count remains per VM.

### 16.3 Optional diagnostic timing

A non-gating environment-controlled timing surface may report:

```text
canonical shared product lookup/build
kernel allocation
native materialization/install
Universe AST compilation
Universe source execution
post-bootstrap finalization
total
```

Timing instrumentation must remain off hot paths unless explicitly enabled.

---

## 17. Performance expectations

### 17.1 Primary expected gain

Within one process/test binary:

```text
before:
    N full VMs
    → N whole-Universe semantic analyses

after:
    N full VMs
    → 1 whole-Universe semantic analysis
    → N VM-local materialize/compile/execute stages
```

The broad `phalcom-core --test core` target should therefore collapse the multiplicative semantic cost dramatically.

### 17.2 Cold first-VM cost remains

The first full VM in a new process still derives the canonical compiler product.

That cost may remain tens of seconds until semantic-analysis performance is separately optimized or the future build-time artifact architecture is implemented.

This is expected and does not invalidate this program.

### 17.3 Separate test binaries

Each OS process has its own one-time cache. Different Cargo test binaries may each pay one cold canonical build if they create full VMs.

This program optimizes the architecture without adding persistence.

---

## 18. Compatibility requirements

### C-01 — `VM::new()` behavior

Existing production callers, CLI, REPL, and full integration tests must continue to receive a fully bootstrapped runtime.

### C-02 — Canonical identities

The exact source-complete lowering semantics introduced by the canonical Universe repair must remain unchanged.

### C-03 — Runtime heap isolation

Mutating globals/classes/modules in one VM must not affect another VM.

Do not compare raw `ObjRef` or `ClassId` numeric values across different heaps as an isolation test; equal numeric indices in distinct heaps are valid.

### C-04 — Existing source tooling

No source identity, traceback, or go-to-definition contract is intentionally changed by this program.

### C-05 — Native installation mode

Existing `NativeInstallMode` public behavior remains compatible.

### C-06 — General `CompiledProgram`

User/project compilation behavior is not changed by introducing the canonical shared product.

---

## 19. Sources of truth

| Concern | Source of truth | Derived consumer | Forbidden competing authority |
|---|---|---|---|
| Canonical Universe source | `UniverseSourceProvider` / `BuiltinInterfaceBuilder` | source index, linker, diagnostics | core-local second source cache |
| Parsed builtin source | existing `BUILTIN_PARSED_CACHE` | `NativeSourceIndex`, analyzer | new VM cache of parsed source |
| Unlinked builtin interface | existing `BUILTIN_INTERFACE_CACHE` | linker/materializer | VM-specific reconstructed interface authority |
| Canonical module graph | `LinkedProgram` | compiler/runtime bindings | ad-hoc VM import resolution |
| Formal Universe semantics | transient `SemanticSnapshot` from source-complete analysis | lowering projection | source-incomplete per-module analysis |
| Backend semantic facts | `CompiledModule.lowering` / `ModuleLoweringSemantics` | VM compiler | fallback name matching |
| Runtime objects | each VM's `Heap` and registries | execution | process-global VM/heap |
| Source-derived runtime roots | actual values produced by full Universe bootstrap | numeric/dispatch semantics | Nil/default placeholders |
| Test capability | constructor and concrete initialized state | test fixture | mutable `bootstrap_level` string/flag |

---

## 20. Tempting wrong fixes

The implementation must explicitly avoid the following.

### 20.1 Share one initialized VM across tests

Wrong because it introduces order dependence, global mutation leakage, shared dispatch caches, shared resources/fibers, lock contention, and difficult failure attribution.

### 20.2 Clone a bootstrapped VM

Wrong for this program because VM state contains complex heap identity, caches, resources, and runtime handles. A snapshot architecture requires its own semantics.

### 20.3 Cache raw `SemanticSnapshot` as a general process-global baseline

Wrong because the snapshot is tied to semantic session/store ownership. This program needs lowering projections, not global type-store identity.

### 20.4 Add another parsed-source/interface cache in `phalcom-core`

Wrong because `phalcom-modules` already owns process-wide canonical builtin caches.

### 20.5 Revert to source-incomplete per-module Universe analysis

Wrong because that loses canonical cross-module semantic identities and can reintroduce fallback variant ownership.

### 20.6 Cache runtime closures/chunks without relocation analysis

Wrong because current compilation may embed VM-local values, symbols, class identities, and mutable caches.

### 20.7 Narrow bootstrap roots inside this performance patch

Wrong because it changes runtime initialization semantics and can hide missing source-installed behavior.

### 20.8 Mechanically replace every `VM::new()` in tests

Wrong because many tests intentionally verify source-authored Universe behavior.

### 20.9 Hide lower-tier missing capabilities behind automatic full bootstrap

Wrong because a low-level test could unexpectedly trigger the expensive compiler path again.

### 20.10 Use wall-clock thresholds as the sole regression gate

Wrong because CI timing is noisy. Architectural work-count/state invariants are stronger.

---

## 21. Required production changes by subsystem

### `phalcom-core/src/modules/`

Add the canonical shared compiler-product owner.

Responsibilities:

- derive/memoize canonical linked+lowered Universe program;
- perform source/native verification once;
- expose immutable read access;
- reuse existing compiler/module products.

### `phalcom-core/src/vm/bootstrap.rs`

Reduce responsibility to:

- composing VM bootstrap stages;
- VM-local native materialization/install;
- VM-local source module compilation/execution from shared products;
- post-bootstrap runtime invariants.

Remove ownership of:

- canonical import resolution;
- `ModuleLinker`;
- `analyze_workspace`;
- lowering projection;
- per-VM source/native verification.

### `phalcom-core/src/vm/mod.rs`

Represent source-derived semantic roots explicitly as optional/unavailable below full bootstrap.

### Semantic-root consumers

Update:

- `phalcom-core/src/primitive/int.rs`;
- `phalcom-core/src/primitive/number.rs`;
- `phalcom-core/src/vm/dispatch.rs`;
- `phalcom-core/src/vm/gc.rs`;

and any additional exhaustive search hits.

### Tests/support

Update:

- `phalcom-core/tests/support/vm.rs`;
- `phalcom-core/tests/README.md`;

and migrate only ownership-appropriate tests.

---

## 22. Acceptance invariants

### I-01 — One compiler derivation per process

Repeated full VM construction obtains one immutable canonical Universe compiler product.

### I-02 — No compiler frontend in kernel/native constructors

Neither `VM::new_kernel()` nor `VM::new_native()` invokes whole-Universe semantic analysis.

### I-03 — Full VM semantics unchanged

Existing source-prelude, canonical ADT, Either, collection, reflection, and module behaviors remain correct.

### I-04 — Canonical lowering remains source-complete

`Result::Ok` / `Result::Error` and other canonical variant/associated/match targets retain authored semantic identity.

### I-05 — No fake semantic roots

Below full bootstrap, source-derived semantic roots are absent, not represented by Nil/default placeholders.

### I-06 — VM isolation

Each VM has independent mutable runtime state.

### I-07 — Existing parsed/interface cache remains sole authority

No duplicate canonical parsed/interface cache exists in core.

### I-08 — Existing eager execution semantics retained

The same source module set/order that current `bootstrap_roots()` selects is executed by full VMs unless a separate later specification changes it.

### I-09 — Full default remains full

`VM::new()` remains the shipping fully bootstrapped constructor.

### I-10 — No persisted artifacts introduced

All sharing is process-local and immutable.

---

## 23. Verification strategy

The implementation plan accompanying this specification defines exact checkpoint commands. At the specification level, verification must cover:

1. singleton/shared canonical compiler product;
2. kernel/native/full constructor state boundaries;
3. explicit semantic-root absence/presence;
4. source/native full runtime canonical behavior;
5. canonical Result/variant lowering;
6. VM mutable-state isolation;
7. selected low-level tests migrated to lower tiers;
8. negative searches proving semantic analysis/linking no longer live in VM bootstrap;
9. broad crate/workspace compatibility only after focused checkpoint evidence.

Performance measurements are recorded as evidence but are not the primary correctness oracle.

---

## 24. Follow-up opportunities explicitly deferred

After this architecture lands, measure the remaining per-VM phases independently.

Potential next programs, in likely order:

1. optimize cold canonical semantic analysis itself;
2. define precise reachable/lazy Universe source initialization;
3. create process-shared relocatable compiled Universe bytecode templates;
4. implement the previously ratified general compiled-project artifact model;
5. generate/embed the canonical Universe artifact at toolchain build time.

This specification intentionally creates the ownership seams those later programs need without implementing them prematurely.

---

## 25. Repository evidence map

The implementation plan should re-check these anchors for drift before editing.

| Path | Primary evidence |
|---|---|
| `phalcom-core/src/vm/bootstrap.rs` | current monolithic `VM::new`, `run_universe_modules`, `universe_lowerings` |
| `phalcom-core/src/vm/mod.rs` | VM ownership, `RuntimeRoots`, `SemanticRoots`, bootstrap measurement |
| `phalcom-core/src/universe/mod.rs` | VM-local kernel class tower |
| `phalcom-core/src/native/source.rs` | VM-free `NativeSourceIndex`, dependency/bootstrap order |
| `phalcom-core/src/native/verify.rs` | source/native conformance |
| `phalcom-modules/src/builtin_interface.rs` | existing process-wide parsed/interface caches |
| `phalcom-core/src/modules/compile.rs` | `AnalyzedProgram`, `CompiledModule`, `CompiledProgram` |
| `phalcom-core/src/modules/semantic_lowering.rs` | compact immutable semantic lowering projection |
| `phalcom-core/src/modules/builtin_materialize.rs` | VM-local canonical Universe module/native floor |
| `phalcom-core/src/modules/materialize.rs` | general VM-local `CompiledProgram` materialization |
| `phalcom-semantic/src/workspace.rs` | fresh `SemanticWorkspaceSession` per `analyze_workspace` call |
| `phalcom-semantic/src/session.rs` | session-owned DB/TypeStore and semantic state |
| `phalcom-core/src/interpret.rs` | current `run_compiled` already skips initialized/failed modules |
| `phalcom-core/tests/support/vm.rs` | existing shared VM/compiler fixture seam |
| `phalcom-core/tests/README.md` | test ownership architecture |
| `phalcom-core/tests/core/modules/universe.rs` | bootstrap measurement and full Universe regressions |
| `docs/work/logs/2026-09-02-generic-inference-bootstrap-regression.md` | observed bootstrap cost/regression history |

---

## 26. Completion definition

This architectural correction is complete when:

- full Universe linking/semantic/lowering derivation is absent from repeated VM construction;
- one immutable canonical compiler product is shared process-wide;
- `VM::new_kernel()`, `VM::new_native()`, and full `VM::new()` have explicit, tested contracts;
- source-derived semantic roots cannot exist in fake placeholder form;
- current full bootstrap behavior and canonical semantic identities pass focused regressions;
- low-level tests can opt out of source Universe bootstrap through the existing test-support seam;
- no mutable VM state is shared;
- no duplicate builtin parsed/interface cache is introduced;
- final crate/workspace delivery gates pass;
- persisted image/artifact work remains deferred.

## 27. Hardening addendum — 2026-09-03

Implementation baseline for this addendum is `47abba0e5b44d091768748420fd21dd91ae43742`.

The canonical process-global product now has private fields and exposes only
crate-internal accessors. Its construction validates that every root-reachable
and eager-bootstrap module has parsed-source, compiled-module, and linked-module
coverage, and rejects duplicate bootstrap IDs before publishing the `OnceLock`
value. The product owns the immutable linked/lowered program, source index,
reachability set, and bootstrap order; VM heaps, closures, chunks, linked-read
handles, and initialization remain local.

Canonical semantic acceptance is exact rather than blanket. The reviewed error
baseline is stored at
`phalcom-core/core/universe/semantic-diagnostics-baseline.txt`; it contains 146
sorted `(module, diagnostic code, start, end)` records, including duplicates.
Canonical analysis rejects any addition, removal, or range/code/module change.
Ordinary program compilation continues to reject semantic errors directly.

The canonical linked-read specification is the authority for source bootstrap.
Each VM materializes those symbolic reads into VM-local `RuntimeLinkedRead`
entries before compiling a canonical source module. Runtime bytecode compilation
receives `CompileBindings` derived from the same `LinkedModule` that fed formal
semantic analysis. No VM-independent bytecode cache or generic canonical
`materialize_program()` path was introduced.

Focused verification is green for canonical coverage, singleton/identity,
semantic baseline, bootstrap tiers, linked-read parity, module compilation,
module runtime, Universe behavior, native surface, Either, and monad suites.
Absent `examples/core_new.ph`, `examples/person2.ph`, `examples/person.ph`, and
`examples/calculator.ph` tests were removed; the remaining checked-in golden
fixtures pass. Workspace all-target compilation and formatting pass.

Release completion remains open: broad gates exposed existing baseline failures,
including the `modules_linking::mat_06` display-name assertion, callable-surface
sealing, `adt_lower_10`, boolean prelude, and bytes-negative fixture
expectations. Clean `47abba0e` reproduces both module/linking and callable-surface
failures. Workspace clippy also remains blocked by six existing
`phalcom-semantic` lint violations. These are dispositioned as baseline work,
not folded into this bootstrap hardening patch.
