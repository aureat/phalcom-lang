# ARTF001.C1.P3 — Portable Executable Universe: Architecture and Implementation Requirements Analysis

**Date:** 2026-10-21  
**Classification:** architecture/requirements assessment; *not* an approved implementation plan or certification report  
**Target:** `aureat/phalcom-lang`, branch `feat/artifact-universe-consolidation`  
**Verified remote HEAD:** [`aad6a88db904de04059ee9027240ab168b136c5c`](https://github.com/aureat/phalcom-lang/commit/aad6a88db904de04059ee9027240ab168b136c5c) (GitHub branch API, 2026-10-21 13:36:58 UTC).  
**Evidence boundary:** Remote GitHub files inspected read-only and two user-uploaded lifecycle documents; no local branch checkout, execution, test, Git working-tree status inspection, or implementation modification. The implementer's clean worktree claim is reported, not independently verified by remote branch inspection.

## 1. Executive decision

Retire `PERF003.C2.P8` and proposed `P9` **as active planning identities**, preserve their recorded engineering evidence and committed implementations, and bring the executable-Universe objective into **`ARTF001.C1.P3`**. This is a lifecycle and ownership transition, *not* a reason to delete the existing executable modules, reset their schemas, or discard their research. P3 must end with a portable, validated, fully materializable `universe.telosc` that can compile novel user code and initialize fresh VMs in a source-denied process, including eager and deferred imports, native/primordial initialization, GC correctness, deterministic construction, and explicit CLI/API operation. Compiler-image P2 certification is a **hard prerequisite**, not something P3 can silently inherit merely from partial P8 work.

**Principal blocker:** The portable `MethodRecipeV1` row does not independently bind a canonical callable identity to the `Bytecode::Method` / `VariantMethod` installation that consumes it. The current validator proves numerous local properties, but ordinary method-installation identity still needs an operand-tied, sound, bounded CFG dataflow proof. No executable authority should be minted before that proof and all other validation obligations close. In parallel, the detached-closure owner equality has already been incorporated in the reported and inspected T3 groundwork; retain and adversarially test it.

**Critical scope rule:** P3 is an executable *Universe* cutover while `phalcom-core/core/universe` remains source authority until P5; it is **not** general package incremental compilation, artifact auto-discovery/cache policy, registry distribution, or the `packages/universe` source migration.

## 2. Current evidence, reconciliation and limitations

### 2.1 Branch audit

GitHub's branch endpoint resolves `feat/artifact-universe-consolidation` to `aad6a88db904de04059ee9027240ab168b136c5c`, parent `9b4da01af1297450243eea55c8110938af48e850`, with preceding `953acc6cb8f459a7f4dae9f7da07730e6b3f913e`. The latest commit message is `fix(artifact): complete P8 T3 validation groundwork`. This matches the handoff's named pushed revision. A remote GitHub branch cannot establish whether a *separate local* worktree contains uncommitted changes; confirm `git status --porcelain=v1` locally before making implementation changes. The uploaded `ARTF001.C1/CHECKPOINT.md` still labels P3 `PROPOSED / NOT_STARTED / UNVERIFIED` and P2.R4 `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. Do not overwrite those with an inferred P2 completion. Its 2026-10-20 continuation header cites an older commit, so lifecycle prose needs re-pinning to this audited HEAD rather than being mistaken for live branch state.

The uploaded P8 plan records T1 and T2 `IMPLEMENTED / FOCUSED_TESTED`, T3 `PARTIAL / UNVERIFIED`, T4-T7 unimplemented, T8/T9 partial and T10 open. Its current gating explicitly prohibits running behavioral tests in that old continuation. The user is now requesting a *new* P3 plan with actual test acceptance: the new P3 should explicitly supersede the old **no-test execution restriction for P3**, while respecting any still-active independent instructions and separating planning from actual execution. No tests were run in this assessment.

### 2.2 Read-only file findings at the pinned HEAD

- `phalcom-core/src/artifact/validate.rs` (~137 KB): real `ExecutableValidationContext`; structural, module, bootstrap, native, primordial, class, prototype/owner, method-recipe, detached-method, operand, stack, CFG, activation, semantic-pool and canonical-root validators. Functions include `validate_executable_image` (line ~330), `validate_detached_methods` (~968), `validate_method_recipes` (~830), `instruction_stack_transfer` (~1548), `validate_prototype_control_flow` (~2098). The validator entry still yields `Result<(), ExecutableValidationError>`; `ValidatedExecutableImage` is defined but should not be treated as minted authority.
- `phalcom-core/src/artifact/mod.rs`: `canonicalize`, `capture`, `instruction`, `project`, `semantic`, `validate`; **no `materialize` module** at the inspected revision.
- `phalcom-core/src/artifact/project.rs`: `project_callable_prototype` exhaustively extracts `Callable` and `Chunk` layouts, carries portable frame/parameter/upvalue/source/semantic data, and excludes runtime source IDs and caches. It explicitly declares projection non-authoritative.
- `phalcom-compiler-artifact/src/executable/mod.rs`: executable DTO families (`compatibility`, `constant`, `image`, `instruction`, `module`, `prototype`, `semantic`, `staging`) and separately numbered executable schema, bytecode, emitter, VM operand/frame, native ABI revisions (all V1). Keep them separately versioned from the compiler-image semantic ABI.
- `phalcom-core/src/modules/canonical_artifact.rs`: existing P2/P7 deterministic compiler-image writer/loader (`TeloscContainerBuilder`, loaded semantic authority, budgeted reconstruction, atomic transport). P3 should **extend**, not duplicate, this ownership.

These are targeted file inspections, **not** a full compiler call-graph, all-file diff, build, benchmark, or complete code audit. File-level tasks below distinguish confirmed paths from proposed additions and paths to inspect.

### 2.3 P2 evidence must remain separate

The uploaded ARTF001 checkpoint records the successful exact allocation boundary `N_image = 450,922,288`, N−1 failure, retry, child rollback, four prepublication failpoints, direct immutable-image-to-session `Arc` continuity, source-denied compilation of the slotmap fixture, and focused semantic-artifact tests. It also explicitly leaves R4.T2, T4-T9 and final P2 acceptance open in that record. This evidence supports an existing **compiler** artifact foundation, not executable VM startup. Its source-denied test compiled four slotmap modules and eight runtime tests but did *not* execute them. Reusing this history as executable P3 success would be incorrect.

### 2.4 Required reconciliation table

| Existing identity | P3 disposition | Evidence requirement |
|---|---|---|
| P8.T1 executable DTO/manifest | Retain as P3 baseline | Review V1 wire table, 128-opcode exhaustive manifest, compatibility limits; compile and codec/hostile tests |
| P8.T2 capture | Retain as P3 baseline | Prove every canonical initializer, nested callable and later import covered; frozen bytes deterministic; no build VM heap leak |
| P8.T3 structural/CFG groundwork | Continue and strengthen as P3 validator | Method/class/variant operand provenance, repeated class allocation, reuse, native anchors, unreachable rows, rigorous hostile tests |
| P8.T4 runtime relocation | New P3 execution slice | New per-VM materializer, allocation/rooting, identity and transaction semantics |
| P8.T5 native/primordial floor | New P3 execution slice | Remove provider-source edge from artifact construction, preserve ordering/anchors |
| P8.T6 first VM boot | New P3 vertical tracer | Fresh-source-denied VM, initializer ordering and identical behavior |
| P8.T7 later imports | New P3 slice | No AST/emitter fallback for deferred Universe modules |
| P8.T8 explicit CLI / artifact publication | New P3 integration slice | Build and run commands use exactly one loaded immutable authority |
| P8.T9/T10 acceptance | New mandatory P3 certification | Actually execute behavioral, adversarial, two-VM and determinism gates; do not inherit old compiler-only acceptance |
| P9 automatic caches/discovery | Retired from current scope | Do not silently implement under P3 |

## 3. Normative P3 product contract

### 3.1 Produce

Compile the current source-authoritative Universe through **the existing compiler/emitter**, inside an isolated build VM, capture initializer and reachable nested prototypes at the correct pre-execution boundary, canonicalize into portable executable DTOs, cross-validate against the fully reconstructed compiler/semantic/module/metadata authority, and atomically emit one artifact with an explicit executable section. Capture must cover *all* canonical modules, not only eager bootstrap modules. The build VM may run earlier initializers for compilation prerequisites but must export **recipes**, not runtime objects, initialized globals, mutable caches, closure environments, or GC handles.

### 3.2 Consume

Bounded decode → section and compatibility checks → compiler-image source-free load → executable and semantic cross-check → private validated executable authority → per-VM fresh allocation/relocation → source-free native floor → ordered eager initializer execution → deferred import prototype registry → user program execution. An invalid executable must never reach native installation or execute an initializer. The existing compiler-only P2 entry remains usable without an executable. Explicit *runtime* artifact mode requires one.

### 3.3 Invariants

1. P7/P2 semantics, canonical P7 `CallableId`/class ownership, project-qualified stable identity, ordinary `SemanticWorkspaceSession`, and semantic query authority remain singly owned; executable DTOs do not become a second ontology.
2. Portable bytes contain no `ObjRef`, runtime `ClassId`, source-local metadata-pool ID, raw pointer/address, captured mutable value or mutable cache. Symbolic references are resolved only against validated authority.
3. Semantic/image ABI and executable/emitter/native/VM-frame ABI are independently checked. Unknown versions/tags fail closed.
4. Decode, structural validation, and cross-section validation are VM-free and precede all runtime side effects. Public callers cannot forge `ValidatedExecutableImage`.
5. Each VM receives fresh globals, heaps, metadata caches, classes and module initialization state; sharing inside the VM is preserved where specified; one immutable loaded artifact can be shared across VMs.
6. Compiler/emitter and source-built VM order and failure semantics remain authoritative. No second emitter or optional hidden source fallback.
7. Runtime holder/access owner/lexical anchor derive from actual installation semantics; canonical callable identity is independently proved. Class-side method installation must not equate holder with access owner.
8. Recursive prototype/constant/semantic-pool reference safety, bounded memory/time, checked arithmetic and GC rooting apply during decode, validation and materialization.
9. Runtime `ReturnNonLocal` remains governed by the user-selected existing-schema *all-activation-route block-home-token* graph proof; no name-based approximation or schema churn.
10. P5 will move the Universe source; P3 must not depend on that source move.

## 4. The central unresolved proof: method installation

### 4.1 Exact trust defect

The compiler checks `compile_behavior_member`'s selector/side against an expected callable, but ordinary capture records a `MethodRecipeV1` with body/signature and frequently a `Nested` closure-prototype owner; `Bytecode::Method` installs a method from stack values independently. Proving that the recipe is structurally valid and that a nearby owner exists is **not** proving that the actual operand at this bytecode site denotes the authorized canonical callable. Runtime installation mutates holder, access owner and closure lexical anchors. `VariantMethod` instead gets behavior authority through its semantic variant operand; it needs an equally exact, but *different*, binding proof.

### 4.2 Chosen P3 contract: conservative, schema-preserving provenance

Use a forward, bounded abstract interpreter over each prototype's real CFG. Define a full abstract frame with operand stack, mutable locals, scratch slots and any identity-carrying implicit stack entries. Seed facts only when producer semantics prove the precise runtime object: method constants/recipes and native anchors; stable primordial class objects; validated `Class` allocation instruction *sites* uniquely tied to `SourceDeclared` recipes; and directly verified variant authority. Do not seed a class object solely from its stable class name. `Closure` normally creates a block closure, **not** an installed method value.

Suggested conceptual domain (names are design proposals, not verified existing Rust types):

```rust
enum Provenance {
    Unknown,
    Method { recipe: MethodRecipeId, object: MethodObjectToken },
    Class { class: StableClassRefV1, allocation: AllocationIdentity },
    Native { anchor: NativeAnchorId },
    Other,
}
struct FrameFacts {
    operand_stack: Vec<Provenance>,
    locals: Vec<Provenance>,
    scratch: Vec<Provenance>,
}
```

This skeleton requires an **explicit identity policy**: the constant pool may reconstruct the *same method object* at multiple uses, or may allocate separately depending on runtime constant materialization. A `recipe` ID by itself is not sufficient identity unless proven by the materializer. Distinguish recipe provenance, runtime *object identity*, and installation binding in the actual verifier. Set `Unknown` as the top (loss-of-proof) element; retain a singleton fact at joins **only on exact agreement among every reachable predecessor**. Two semantically equal classes allocated at different instruction sites are not identical objects. Track worklist convergence with bounded per-prototype states, graph sizes, slots and iteration count; unsupported state becomes rejection, never an infinite analysis loop or optimistic acceptance.

Transfer functions must exactly mirror VM opcode semantics. Implement `Constant`, `Class`, `Dup`, `Pop`, local Get/Set, swap/rotation if present, scratch Get/Set, conditional branches, method/variant installation, returns, and all 128 opcode categories through one exhaustively checked transfer table. For calls, globals/upvalues, captures, mutation and unknown producers, **kill only what can no longer be proved**—but verify aliasing and method mutation explicitly rather than supposing identity fields survive. If any supported transfer cannot be established, make it conservatively unknown; reject only where proof is mandatory. Keep stack-height safety and provenance analysis consistent and fail when CFG merges have incompatible stack heights.

At `Method`, take the actual method operand and behavior-class operand from VM-defined stack positions, including class-side metaclass selection. Determine canonical callable owner from the P7 class/callable tables; compare selector, dispatch side, method signature and body/native anchor, concrete allocation object, and installation-derived anchor semantics. At `VariantMethod`, use actual method operand and the *semantic variant operand* to resolve the exact variant/behavior-class authority, then apply the same callable/selector/side/signature/body checks. This is not equivalent to `Method` with a fabricated class stack operand.

### 4.3 Explicit decisions recommended for the new plan

**Fresh `Class` allocation:** Require a statically unique allocation instruction site per `SourceDeclared` class recipe, uniquely owned and not reachable by a cycle capable of re-execution. A syntactically acyclic CFG alone does not prove uniqueness across repeated *prototype invocations*; require a canonical one-shot initializer route or a stronger call-graph/lifecycle proof. If this cannot be shown, reject the `SourceDeclared` class provenance. Do not ban all loops globally; ban loops whose class producer could repeat.

**Method recipe reuse:** Prefer **rejecting multiple runtime installations of one mutable method object**, unless the materializer demonstrably constructs distinct objects per use or the VM semantics permit provably harmless repeated same-object installation. Merely equal target class, selector and side is insufficient: a second invocation can change mutable holder/anchors or observe different initialization state. Provide a precise object/token model and allow benign reuse only with a tested semantic contract. This is safer than the old plan's proposed allowance on equal symbolic binding alone.

**Alias handling:** Precisely preserve stack/local identity through identity-preserving operations; track aliases as the **same abstract object token**, not copied independent method values. Mutation or escape through globals, upvalues, calls or heap containers must invalidate all possibly aliased tracked facts or reject that path. `SetLocal` must follow actual stack-preservation/pop behavior. Scratch-slot operations need equal scrutiny.

**Coverage:** Every *reachable* installation must have complete operand-tied proof. Additionally, every serialized prototype/recipe/anchor/class row, **including unreachable rows**, must be structurally and cross-section valid; reject unused but malformed rows. Unused method recipes need not have invented installation identity, but must not make false claims of canonical callable ownership. A recipe with a claimed binding, detached owner, or native anchor must validate that claim whether installed or not. Declare unreachable bytecode policy: structurally validate all instructions; identity provenance is required at reachable installs; either reject unreachable install instructions as invalid canonical output or allow them only with a documented non-executability proof. Prefer explicit rejection of unreachable method installs in a canonical product to reduce ambiguity.

**Detached methods:** For a closure-body detached row, follow detached callable → method constant → closure template → prototype owner. Demand exact `PrototypeOwnerV1::Callable.callable == detached.callable` along with P7 membership and selector checks. For native detached entries, validate native anchor ownership/side/arity independently; no closure-only rule on native bodies.

**ReturnNonLocal:** Retain the existing activation-route graph proof, refusing mixed/unknown/method routes for a template containing this opcode; add adversarial graphs including nested aliases, reachable and unreachable mixed use, home-frame loss, recursion and closure sharing.

### 4.4 Safety and completeness trade-off

A fail-closed bytecode verifier is sound only relative to a faithful VM transfer model and all aliases/escapes that can affect the installation. It can reject legal runtime configurations it cannot prove. P3's producer and verifier must be **compatible**: every output emitted by the supported canonical Universe build must validate. If a legal emitter output becomes `Unknown`, first determine the precise producer pattern and expand a *semantically justified transfer*; do not loosen the validator, hard-code names/spans, or add unvalidated metadata. If a sound existing-schema proof cannot accept required legal code, stop for a new architectural decision (for example, authenticated/install-operand-linked metadata plus executable enforcement and a versioned schema). No implicit widening.

## 5. Full implementation architecture and ordered work packages

### P3.0 — Lifecycle takeover and baseline certification

Rewrite the `ARTF001.C1` ledger in place to identify P3 as the executable-Universe owner, keeping historical P8/P9 documents archived/superseded, not rewritten as if work never existed. Pin the SHA. Verify P2.R4 full exit gates, not just T3 allocation. Record compiler-only image semantics, the immutable ordinary session, typed query roots and source-denied compile parity. Establish the exact test policy for P3: **running focused and end-to-end tests is a required acceptance condition**; if forbidden by a higher-priority current instruction, P3 remains unverified and cannot close. Evidence: Git clean revision, P2 gate manifest, measured build inputs, no source fallback.

### P3.1 — Executable wire/producer freeze and census

Audit the 128-opcode manifest, operand stack effects, constant and semantic-pool recipes, class/primordial/native binding catalogs, closure upvalue semantics, bootstrap order, deferred modules and source-map fidelity. Reuse current `ExecutableImageV1`, canonical capture and projection; add producer-to-recipe/identity-coverage audit. Validate deterministic canonical numbering despite differing filesystem roots, hash seeds and allocator histories. Check compatibility header and digest independently from P2 ABIs. Reject unsupported runtime constants with a typed owner/prototype/constant diagnostic; do not silently serialize live heap state.

### P3.2 — Complete bounded static executable verification

Refactor `validate.rs` into narrowly-owned submodules **after** preserving all current checks: `context`, `structural`, `control_flow`, `provenance`, `installation`, `native`, `semantic` (names proposed). Keep a single validation entry and one ABI/policy source. Define a testable, exhaustive transfer/CFG model; exact provenance lattice; allocation one-shot proof; alias invalidation; ordinary and variant installation; detached closure/native equality; all-row checking; semantic/native/P7 cross-section validation; full error taxonomy and budgets. Promote to `ValidatedExecutableImage` only once every proof returns success. Prefer an immutable `Arc<ExecutableImageV1>` bound to the already validated compiler image identity and compatibility digest, inaccessible to unvalidated consumers. Do **not** cache successful validation against mutable bytes or another loaded P7 image.

### P3.3 — Atomic artifact publication and read-boundary integration

Extend `canonical_artifact.rs` container assembly with required EXECUTABLE and full-image validation *before* atomic rename/publication. Preserve compiler-only load/API compatibility. A typed executable loader should bind the decoded image to the exact semantic/module/native authority of the same artifact. Reject missing executable section only for VM-required paths, duplicate sections, mixed semantic/executable provenance, stale native ABI or profiles. Expand cumulative limits for executable sections without weakening the proven P2 allocation ledger. Create a **new independently measured executable load budget** and transaction rather than claim P2's `N_image` still covers newly added payloads.

### P3.4 — Fresh VM relocation and GC rooting

Add `phalcom-core/src/artifact/materialize.rs` and, as useful, narrowly-scoped `relocation.rs` and `roots.rs`. Define per-VM relocation maps keyed by validated recipe/prototype identity; allocate class/metaclass/module/callable/method shells prior to cyclic wiring; map primordial identities onto the VM's kernel, not duplicate shells; materialize semantic descriptor pools and fresh empty caches. Root every intermediate allocation using existing GC temporary-root APIs. Reserve all canonical module identities before side effects; stage unpublished state; ensure failures expose no half-registered modules and fresh independent VMs can retry. Confirm exact materializer constant sharing semantics before finalizing P3.2's object-token rules.

### P3.5 — Source-free native and primordial setup

Extract common source-backed/native bootstrap mechanisms so the artifact path starts from `new_kernel` (or actual equivalent), reads only loaded compiler/executable authority, installs stable binding keys and native anchors, reconstructs constants/aliases/None/static runtime behavior, attaches source-owned primitive contracts, and respects sealing/field reservations. Audit `builtin_materialize.rs`, `native/install.rs`, `vm/bootstrap.rs` and any provider call reachable from them. Preserve source VM path unchanged. Neither constructing an inert provider nor assigning a diagnostic URI counts as reading source; invoking it to parse/typecheck canonical modules does.

### P3.6 — First full source-free VM tracer

Implement `VM::try_new_with_universe_artifact(Arc<LoadedUniverseArtifact>) -> Result<VM, TypedBootstrapError>` or a semantically equivalent current API. Require verified executable authority before any native/initializer effects. Install kernel/native floor, materialize and root shells, execute eager initializer prototypes exactly in stored dependency order, attach native bindings at their established phases, complete alias/metadata roots, and retain the artifact `Arc`. The first vertical tracer should compile a novel user program using the same artifact semantic authority and *execute* a representative user workload, with all canonical source-access denial guards armed.

### P3.7 — Deferred imports and runtime diagnostics

Create one per-VM canonical prototype registry for every non-eager Universe module and integrate it in `interpret.rs` / module initialization. Before any source-text fallback, recognize exact artifact canonical ownership; initialize dependencies and requested modules once, preserve error/re-entrancy handling, and never run a missing canonical prototype through a source compiler. Test runtime diagnostics with logical URIs/spans even though canonical source bytes are unavailable.

### P3.8 — Explicit CLI/API and deterministic full build

Add or complete `phalcom artifact build-universe --output <path>` and `phalcom --universe-artifact <path> <user-program>`. Single artifact load, same immutable image for `ProgramCompiler` and VM, no hidden reload or alternate semantic engine. Explicit failure is typed and has a stable nonzero exit status; legacy no-option CLI continues existing source path. Source canonical compilation profile remains Debug while user-facing compiler options behave as before. Do not introduce automatic discovery, registry, distribution or general incremental package policy.

### P3.9 — Behavioral/security certification and closure

Execute full hostile codec/validator/identity/native/profile matrix, source-vs-artifact behavior matrix, fresh-process denial, deferred import, two-VM sharing/isolation, failure retry, determinism, performance breakdown and appropriate broader workspace regressions. Record exact commands, versions, commits, test counts and failures. Retire P8/P9 as active only through explicit lifecycle markers; preserve their plans as historical evidence. Close P3 only with actual **executed** source-free VM acceptance, not compilation or prepared harnesses.

**Dependency order:** P2 exit → P3.0/1 → P3.2 → P3.3 → P3.4 → P3.5 → P3.6 → P3.7 → P3.8 → P3.9. A narrow materializer *semantic investigation* may run in parallel with verifier design because method-constant object identity is prerequisite to proving reuse; runtime **exposure** must still wait on validation. CLI skeleton and test fixtures may be developed early, but never promoted as an executable tracer.

## 6. Code ownership and change inventory

| Path | Planned operation | Ownership rationale |
|---|---|---|
| `phalcom-core/src/artifact/validate.rs` | Major revise/split | Complete VM-free CFG, identity, alias, allocation and installation proofs; authoritative private success token |
| `phalcom-core/src/artifact/mod.rs` | Revise | Register validator and new materialization/relocation modules |
| `phalcom-core/src/artifact/{capture,canonicalize,instruction,project,semantic}.rs` | Audit and targeted fixes | Existing emitter capture, stable canonicalization, exact provenance producers, full reachable object census |
| `phalcom-core/src/artifact/materialize.rs` **(new)** | Implement | Per-VM fresh allocations and relocation, GC roots, failure-atomic staging |
| `phalcom-core/src/artifact/{relocation,roots}.rs` **(optional new)** | Implement if justified | Encapsulate maps and lifetime/root management rather than generic VM redesign |
| `phalcom-compiler-artifact/src/executable/{image,instruction,prototype,constant,module,semantic,compatibility,staging}.rs` | Audit/revise sparingly | Bounded DTOs and codecs, exhaustive tags/ABI, reference and shape constraints; version only on material contract change |
| `phalcom-core/src/modules/canonical_artifact.rs` | Extend | P2-loaded authority and atomic full executable build/load; no parallel reader |
| `phalcom-core/src/modules/{materialize,builtin_materialize}.rs` | Revise | Native module wiring and later import initialization without source |
| `phalcom-core/src/vm/bootstrap.rs` | Revise | Kernel/native floor split and typed artifact constructor preserving source order |
| `phalcom-core/src/native/install.rs` | Revise | Stable native descriptor/anchor binding and contract parity |
| `phalcom-core/src/interpret.rs` | Revise | Canonical late-import dispatch to persisted prototypes, no source fallback |
| `phalcom-core/bin/phalcom/{cli,main}.rs` | Revise | Explicit commands and shared loaded-image authority |
| `phalcom-core/src/{bytecode,callable,chunk}.rs` (actual ownership to verify) | Read first; minimal correction only | Concrete execution/constant/stack semantics are verifier's source of truth |
| `docs/specs/compiler-artifacts/telosc.md` | Revise | Canonical executable section layout/compatibility/security semantics |
| `docs/implementation/ARTF001-artifact-and-universe-consolidation/C1/{CHECKPOINT.md,ARTF001.C1.P3-...md}` | Revise/create | New P3 goal/ledger, closure evidence and historical P8 supersession |
| Legacy `docs/implementation/PERF003-runtime-performance/.../P8/` | Mark superseded | Preserve implementation chronology; never delete proof evidence |

File names other than those directly inspected derive from the supplied P8 source inventory and architectural targets; confirm their current location/API before patching. Avoid broad VM, semantic ontology, query-store or GC policy refactors absent direct causal evidence.

## 7. Test design: normative acceptance matrix

**Gate interpretation:** Every gate below needs an executed pass, exact target and transcript. `cargo check` alone establishes no runtime or verifier behavioral correctness. Old P8 test prohibition is superseded **for the proposed P3 execution plan**, but this document itself does not run tests or claim that policy has already been changed in repository records.

| Gate | Exact behavior to prove | Representative command/target (implement if absent) |
|---|---|---|
| G0 baseline and P2 entry | Clean pinned branch; P2 full exit evidence, compiler-image query roots, source-denied compilation, exact old import accounting | `git status --short`; P2.R4 published full gate manifest and `cargo test -p phalcom-core --test artifact-allocation` |
| G1 executable codec | 128 opcode tags, operand widths, canonical ordering, all constant/semantic families, overflow/budget limits, roundtrip, deterministic bytes | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact` |
| G2 validator | Good real Universe image accepted; mutated selector/side/class/variant/signature/native anchor rejected; poisoned CFG joins, local alias, repeated class allocation/recipe installs, cycles, detached wrong-owner, `ReturnNonLocal` mixed routes rejected | New focused `phalcom-core` artifact validator tests; run filtered tests then all relevant target |
| G3 authority construction | Only full successful verification builds private validated wrapper; structural P7 mismatch, stale ABI, native mismatch, malformed unused recipes never mint authority | New validator/loader negative integration tests |
| G4 full image publication | Compiler+executable section saved atomically; bad executable leaves previous artifact intact; compiler-only image remains compiler-loadable but VM-rejected | New full-build/loader integration target |
| G5 GC/relocation | Rooting survives forced collections; no stale handles; shared recipe identity within VM; two separate VMs share only immutable image | New VM artifact relocation tests with forced-GC hooks |
| G6 eager runtime parity | Native anchors, primordial class/meta identity, fields/None, closures/upvalues, static/ADT/data, generics/traits/reflection/default dispatch, exceptions and initializer order | New source-vs-artifact VM parity integration suites |
| G7 source-denied process | Process B physically denies provider/root/embedded canonical source; loads artifact, compiles **novel** user code and executes it; Universe parse/analyze/lower/emit counters all zero | Fresh child-process integration harness |
| G8 deferred import | Import module omitted from eager order; persisted prototype executed once; dependency ordering/errors preserved, zero source access; missing entry typed failure | New later-import integration target |
| G9 CLI/API | Explicit build/run roundtrip, same loaded Arc for compiler/VM; missing, stale, corrupt artifact fail without fallback; no-option parity | `cargo test -p phalcom-core --test cli-smoke` plus new artifact CLI integration |
| G10 determinism and lifecycle | Two independent fresh builds produce byte-identical artifact, even under varied paths/allocation history; N/N−1 transactional budget and failpoints; clean retry | Two child process builders, loader allocation target, prepublication failpoint tests |
| G11 broad regression | Relevant semantic/module/core suites and workspace build compile; baseline-known issues classified, no new failures | `cargo check --workspace --all-targets`, `cargo test -p phalcom-modules`, semantic-artifact suites, selected core parity suites; expand based on budget |
| G12 performance/observability | Measure build vs read/decode/validate vs relocate vs native init vs eager init vs user compilation/execution; report unmodified source startup comparator; do not claim speedup without samples | Dedicated timed fresh processes with reported environment and repetitions |

### 7.1 Specific adversarial verifier fixtures

Construct genuine valid baseline images and mutate *one property per test* while recomputing only those checksums needed to reach the intended validation layer. Require a typed error **before any initializer/native counter increments**. Negative corpus minimum: wrong closure target callable owner; same selector with wrong callable owner; `Nested` owner incorrectly accepted as canonical; detached native/closure tag confusion; swapped two same-signature `MethodRecipe` references; method/class stack swap; class-side metaclass/holder mismatch; variant operand of unrelated variant; branch merge with differing method or class producer; same method aliased through `Dup`/`SetLocal`; calls/escapes destroying identity; `Class` inside cycle; multiple class sites claiming same `SourceDeclared` row; class site inside repeatedly invoked prototype; double installation of one mutable method object; missing source-free initializer prototype; altered native binding signature/digest; corrupted unreachable constant/prototype; `ReturnNonLocal` via both block and method routes; wrong upvalue slot; arithmetic overflow and cap exhaustion. Add positive genuine emitter fixtures featuring locals, stack duplication, conditional control flow, class-side methods, variant methods, detached closure/native methods, alias-preserving moves, and the entire canonical Universe.

### 7.2 Fresh-process parity workload

Use one real user program exercising several dimensions together: imported Universe data/ADT constructors and pattern matches, an inherited class with instance and class-side dispatch, a closure closing over a local, a generic/trait method, native call, exception handling, reflection and a non-eager module import. Run exact same user file against source VM and artifact VM, compare stdout, exit/error category, semantic observable values and ordered event markers; do not rely solely on exact object addresses or timing. Source-denied process must be a genuinely fresh process with all source denial hooks *armed before loading*, not a warmed process with canonical compiler state already materialized.

### 7.3 Two-VM isolation workload

From one `Arc<LoadedUniverseArtifact>`, construct VM A and VM B; compare canonical stable owners but assert distinct mutable module globals, heap objects, inline caches and initialization counters, preserving expected shared references only within each VM. Mutate global A; B must be unchanged. Force collections during relocation and initialization. Re-construct after a staged failure, check no persistent poisoned registry/global state; reject any global static relocation cache.

### 7.4 Commands and interpretation

```sh
# Fast compilation feedback (not acceptance)
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-compiler-artifact --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib --bin phalcom

git diff --check

# Behavioral gates, mandatory after the P3 plan authorizes them
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic-artifact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test cli-smoke
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets
```

New tests should receive stable descriptive target names; their final exact `cargo test ...` command must be recorded when the files exist. An empty filtered test run is a **FAIL**, not PASS. Long real Universe bootstrap targets may take minutes: run single Cargo processes in bounded sequences, preserve logs, perform independent static work during waits if supported by the implementing environment, and never report cancellation/timeout as passing. Record real baseline failures separately from introduced regressions. Include `cargo fmt --all -- --check` and `git diff --check` before commits.

## 8. Compatibility and transaction requirements

- **Compatibility matrix:** old compiler-only `.telosc` → compiler load YES, VM load NO; new full `.telosc` → compiler and VM YES if every executable ABI/native/profile matches; unknown/new executable schema → fail closed; cross-mixed semantic/executable sections → reject; exact same immutable loaded instance shared by compiler and VM.
- **Versioning:** Do not increment outer container or semantic ABI solely for adding previously specified executable section. Any changed executable byte meaning (new producer identity fields or revised opcode/family interpretation) requires its own independently versioned executable schema/ABI and explicit reader compatibility. Existing V1 must not be silently reinterpreted if external bytes already exist. Check whether any V1 artifacts were emitted before choosing in-place pre-freeze revision.
- **Native ABI:** Stable binding key + dispatch side + selector/arity/contract/primordial requirements hashed in canonical order; no function pointers. Check native anchor membership and actual method recipe reference equality, not just catalog validity.
- **Budgets:** P2's ledger proves a compiler-image load shape; full executable bytes add new decode/validation/relocation allocations. Separate immutable artifact-load budget from **per-VM** materialization budget and execution-state allocation. Diagnostic allocations remain accounted or intentionally outside a named measured boundary; typed N/N−1 and rollback at each boundary.
- **Atomicity:** Pre-execution validation returns no published executable on failure. During VM materialization, all relocations are rooted/staged; handle failures without publishing partially initialized global/module state. Runtime initializer exceptions follow ordinary VM semantics and are **not** equivalent to load-time invalidity or a transactional rollback unless the source VM does so.

## 9. Architecture research and transfer to Telos

1. **WebAssembly Core Specification — phases and validation.** W3C distinguishes decoding, validation, instantiation and invocation; validation checks operand stacks and locals before any instantiation. Apply the same separation to `.telosc`: byte decode/structural checking, identity/provenance validation, then fresh VM object construction and initializer execution. Telos additionally needs class/method identity verification beyond Wasm's type validation. https://www.w3.org/TR/wasm-core/ (consulted 2026-10-21).
2. **WebAssembly validation algorithm — bounded symbolic machine.** The W3C algorithm has operand/control stacks and an explicit unknown state. Reuse that conceptual discipline for exact opcode transfer semantics and unreachable paths, but do not copy structured-control rules directly into Telos's unstructured CFG. https://www.w3.org/TR/wasm-core-2/ and https://www.w3.org/2021/11/wasm-stage/appendix/algorithm.html.
3. **LLVM Language Reference — SSA/dominance and undefined-state discipline.** LLVM's distinction between concrete values, `undef` and poison highlights that unknown dataflow facts cannot be treated as positive evidence. This is an analogy only, not an LLVM-compatible verifier or reason to change the Telos language semantics. https://llvm.org/docs/LangRef.html.
4. **Abstract interpretation (design principle).** Use finite abstract object identities, worklist fixed point, monotone joins and explicit widening-to-unknown where needed; soundness derives from faithful concrete opcode transfers and accounting for all aliases. This is recommended design analysis, not an externally verified claim about the current Telos implementation. The key choice is a conservative **may-run identity** analysis: if any reachable operand can be unbound, the installation is unproved.

## 10. Risks, forbidden shortcuts and escalation conditions

**High risk — repeated execution:** CFG acyclicity at an initializer allocation site does not prove an enclosing callable executes once. Demand lifecycle/invocation proof. **High risk — method constants:** Recipe ID may not identify a runtime method object; inspect and test actual materialization semantics before deciding allowed reuse. **High risk — setter/alias behavior:** Existing stack and local transfer code is not automatically a provenance transfer proof. **High risk — native phase order:** A fully deserialized recipe is not equivalent to primordial/native installation order. **High risk — GC:** Valid recipes may still be unsafe if intermediate objects are unrooted. **High risk — P2 closure:** Current uploaded P2 checkpoint does not certify full exit. **High risk — lifecycle drift:** Legacy P8 appears active in uploaded plan; P3 must explicitly supersede without deleting evidence.

Forbidden: infer ownership from selector/name/span/`Nested` alone; use a global name lookup to pass ambiguous installs; blindly trust compiler metadata; allocate imported classes multiple times from one declared site; permit unknown provenance to mint validated authority; equate class holder and access owner; serialize object handles/captured runtime environments; create a duplicate compiler or semantic engine; fall back to source on explicit artifact failure; insert provider-based bootstrap secretly; record `cargo check` as a behavioral pass; suppress actual baseline/hostile failures; close P3 before fresh-process eager and deferred execution actually passes.

Escalate for review when: genuine canonical emitter output cannot be proved by schema-preserving dataflow; class one-shot lifecycle is dynamically contingent; installation alias/escape semantics demand a VM semantic change; native/primordial setup order differs from source; a wire revision conflicts with previously published artifacts; validated wrapper cannot remain tied to the precise P2 authority; P2 closure remains unsatisfied; or GC rooting requires a new global policy. Preserve the smallest isolated failing fixture and precise failing proof premise.

## 11. Suggested P3 task ledger and acceptance contracts

| Task | Name | Exit artifact | Blocking gate |
|---|---|---|---|
| P3.T0 | Baseline, P2 entry and lifecycle transfer | P3 approved design, pinned SHA, P2 acceptance receipt | G0 |
| P3.T1 | Freeze executable producer/codec/identity contract | Complete real image capture and census | G1 |
| P3.T2 | Static CFG and identity proof | Private, verified executable authority; exhaustive hostile tests | G2/G3 |
| P3.T3 | Full container writer/reader, transaction | Atomic full executable artifact, loader budgets | G4/G10 |
| P3.T4 | VM materialization/relocation/rooting | Per-VM staged object graph | G5 |
| P3.T5 | Native/primordial source-free setup | Artifact-only native floor | G6 native subgate |
| P3.T6 | First eager full-VM tracer | Source-denied execution parity | G6/G7 |
| P3.T7 | Deferred import and diagnostics | No-source lazy canonical imports | G8 |
| P3.T8 | Explicit CLI and public API | One-Arc full build/run | G9 |
| P3.T9 | Determinism, negative matrix and broad certification | Signed-off exact evidence table, timing breakdown | G10/G11/G12 |
| P3.T10 | Lifecycle closure and P4 handoff | `COMPLETE / IMPLEMENTED / VERIFIED` only with all required PASS results | All |

**Note:** This is a requirements-analysis proposed task decomposition, not an instruction to patch the repository immediately. The final implementation plan should specify dependency edges, exact test file paths/filters after fixture discovery, expected outputs and bounded slice-by-slice commits. A genuine end-to-end tracer cannot be considered complete until all required canonical Universe modules, native phases, late imports and two-VM proof pass.

## 12. Implementer handoff (design-ready, not execution authorization)

1. Re-fetch the branch HEAD; confirm `aad6a88db` still tip (or re-audit any newer changes), `git status`, and the current P2.R4 records. Do not assume the uploaded checkpoint is newer than the repository.
2. Resolve P2.R4 closure; no P3 acceptance without source-free compiler image certification. Create P3 record in ARTF001 and explicitly retire P8/P9 planning identities while preserving historical evidence.
3. Read the runtime *actual* `Bytecode::Method`, `VariantMethod`, `Class`, constant materialization, `GetLocal`/`SetLocal`/scratch and class-side metaclass paths. Write concrete stack diagrams and identity-alias rules **before** implementing the verifier. Confirm native anchors and exact P7 callable/class resolution.
4. Define and review the small provenance lattice, CFG join, precise kill/alias transfers, one-shot allocation policy and recipe-reuse policy. First produce positive and negative authentic image fixtures; then implement proof modules and private validated authority.
5. Enable focused tests as an explicit P3 plan change. Run them and preserve outputs; no acceptance from build-only checks. Verify all current canonical capture output is actually accepted rather than weakening the verifier.
6. Implement complete executable writer/loader; add fresh per-VM relocation and GC-safe native floor; then source-free eager VM tracer, deferred imports and explicit CLI/API.
7. Run fresh-process denial, native/class/closure/ADT/generic/trait/error parity, two-VM independence and saved-prototype tests; compare bytes/timings and full compiler image budget separately.
8. Audit after each task and before final closure: inspect pushed diff and tests at the new SHA; record deviations, decisions and intentional unsupported cases in ARTF001 checkpoint. Never mark P3 closed if a required gate is `NOT RUN`, `PARTIAL` or `FAIL`.

### Final audit judgement

**P3 is architecturally feasible, but it is not an almost-completed continuation of P8.** The codec/capture and much of static structural validation are existing assets; the central callable-installation proof, actual executable authority publication, runtime materializer, native floor, full VM bootstrap, late-import registry, explicit artifact workflow and exhaustive behavioral certification remain substantive work. P3 should preserve and test what exists, close exact proof boundaries before execution is exposed, and make real source-denied VM execution—not type-checking or capture—a mandatory completion criterion.
