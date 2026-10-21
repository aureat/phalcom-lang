# ARTF001.C1.P3 — Portable Executable Universe: Patch-Grade Implementation Requirements Analysis

**Date:** 2026-10-21  
**Role:** principal architect / implementation specifier  
**Classification:** repository-grounded implementation requirements analysis; suitable as the direct basis for the numbered `ARTF001.C1.P3` implementation plan  
**Repository:** `aureat/phalcom-lang`  
**Branch:** `feat/artifact-universe-consolidation`  
**Verified remote HEAD:** `aad6a88db904de04059ee9027240ab168b136c5c` — `fix(artifact): complete P8 T3 validation groundwork`  
**Remote tree:** `fa548db35cbca96d77db3d67868c0437b26efd2a`  
**Evidence boundary:** read-only inspection of the remote branch and supplied lifecycle/planning documents. No local worktree status, Cargo execution, runtime probe, patch application, commit, or push was performed by this analysis.

> **Purpose.** Translate the already-ratified executable-Universe architecture into a Luna-executable, patch-grade implementation contract: exact current code owners, exact symbols, proposed interfaces, proof algorithms, materialization boundaries, runtime integration order, tests to create, tests to execute at named gates, and objective escalation conditions.
>
> This document deliberately follows the project’s Luna patch-plan doctrine: architecture is fixed here; the implementer should spend time implementing rather than rediscovering ownership, identity, proof boundaries, or test intent.

---

# 0. Executive implementation verdict

`ARTF001.C1.P3` should absorb the useful implementation already produced under the retired `PERF003.C2.P8` work and finish one coherent product:

> **A full `universe.telosc` that carries both the already-proven canonical compiler image and a statically validated portable executable image, and from which a fresh VM can initialize and execute the Universe without reading, parsing, semantically analyzing, lowering, or bytecode-compiling Universe source.**

At `aad6a88db`, the repository is **not** starting from zero. It already has:

- an executable wire schema and bounded codec;
- all current VM opcodes represented portably;
- deterministic source-built executable capture;
- deterministic constant/prototype/semantic-pool canonicalization;
- P7/P2 compiler-image load authority;
- a large core validator with structural, ABI, P7 cross-reference, stack, CFG, local-slot, activation-role, semantic-pool, method-shape, and detached-method checks;
- an opaque `ValidatedExecutableImage` type whose constructor boundary is intentionally unfinished;
- the `EXECUTABLE` container section tag;
- the `PORTABLE_EXECUTABLE` capability bit;
- `TeloscContainerBuilder::stage_executable_section`, which explicitly stages bytes without granting executable authority;
- source-denial and artifact-backed compiler harnesses that can be extended for the runtime cutover.

The remaining work is therefore **not another serialization project**. The critical path is:

```text
finish sound executable proof
        ↓
make successful proof mint private authority
        ↓
integrate EXECUTABLE into atomic artifact build/load
        ↓
materialize runtime recipes into one fresh VM
        ↓
replace source-backed native/bootstrap inputs
        ↓
execute eager Universe initializers source-free
        ↓
support deferred canonical imports source-free
        ↓
expose explicit build/load/run APIs and CLI
        ↓
certify parity, isolation, determinism, failure atomicity
```

The most important architectural gap remains ordinary method-installation identity. The validator must prove that the **actual method object and actual class/variant operand reaching each `Method`/`VariantMethod` instruction** correspond to one canonical P7 callable. A matching method recipe somewhere else in the image is insufficient.

The existing detached-method owner equality is already correct groundwork and must remain independently tested.

---

# 1. Planning doctrine applied

The attached planning instructions establish four requirements that materially shape this implementation specification:

1. **Repository reality precedes planning.** Paths and signatures below were re-read at the live branch revision rather than copied from the retired P8 plan.
2. **Architecture is made explicit so the implementer acts narrowly.** Ownership, identity, proof boundaries, runtime order, and testing gates are fixed here.
3. **Testing is selective evidence gathering.** BUILD mode uses focused discriminators; broad crate/workspace gates are reserved for stabilization/certification.
4. **Architectural uncertainty is not patched speculatively.** Exact STOP/CONSULT triggers are included where a false repository premise would require changing representation, identity, VM semantics, or the artifact contract.

This analysis does **not** silently convert every architectural desire into a mandatory refactor. Existing owners are preserved where they already match the desired architecture.

---

# 2. Repository grounding

## 2.1 Verified branch state

Remote branch:

```text
feat/artifact-universe-consolidation
  HEAD aad6a88db904de04059ee9027240ab168b136c5c
  parent 9b4da01af1297450243eea55c8110938af48e850
```

Compared with `953acc6cb8f459a7f4dae9f7da07730e6b3f913e`, the two subsequent commits changed:

```text
docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/CHECKPOINT.md
docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P8/PERF003.C2.P8-portable-executable-universe-detailed-implementation-plan.md
phalcom-compiler-artifact/src/executable/image.rs
phalcom-compiler-artifact/src/executable/prototype.rs
phalcom-core/src/artifact/validate.rs
```

The dominant implementation change is `phalcom-core/src/artifact/validate.rs` (+1091/-21 lines across that comparison).

## 2.2 Lifecycle mismatch to correct before implementation

The consolidation checkpoint currently still records:

```text
ARTF001.C1.P3 — PROPOSED / NOT_STARTED / UNVERIFIED
```

and older PERF003 records still describe P8 as active. The user has now explicitly retired P8/P9 as active planning identities and moved this work under `ARTF001.C1.P3`.

Therefore the first implementation bookkeeping patch must:

- preserve historical PERF003 evidence;
- stop treating PERF003.P8 as the active owner;
- mark `ARTF001.C1.P3` active when implementation actually starts;
- copy only durable inherited facts into the ARTF001 checkpoint rather than duplicating the entire historical P8 plan.

Do not delete history merely because ownership moved.

## 2.3 Required live reads before code edits

The implementer should re-read these exact paths at the branch tip before touching code:

```text
AGENTS.md
docs/implementation/README.md
docs/implementation/ARTF001-artifact-and-universe-consolidation/C1/CHECKPOINT.md
docs/specs/compiler-artifacts/telosc.md
docs/specs/vm/bytecodes.md
docs/specs/vm/vm-execution-model.md

phalcom-compiler-artifact/src/container.rs
phalcom-compiler-artifact/src/header.rs
phalcom-compiler-artifact/src/section.rs
phalcom-compiler-artifact/src/executable/mod.rs
phalcom-compiler-artifact/src/executable/compatibility.rs
phalcom-compiler-artifact/src/executable/image.rs
phalcom-compiler-artifact/src/executable/instruction.rs
phalcom-compiler-artifact/src/executable/constant.rs
phalcom-compiler-artifact/src/executable/prototype.rs
phalcom-compiler-artifact/src/executable/module.rs
phalcom-compiler-artifact/src/executable/semantic.rs
phalcom-compiler-artifact/src/executable/staging.rs

phalcom-core/src/artifact/mod.rs
phalcom-core/src/artifact/capture.rs
phalcom-core/src/artifact/canonicalize.rs
phalcom-core/src/artifact/instruction.rs
phalcom-core/src/artifact/project.rs
phalcom-core/src/artifact/semantic.rs
phalcom-core/src/artifact/validate.rs

phalcom-core/src/modules/canonical_artifact.rs
phalcom-core/src/modules/materialize.rs
phalcom-core/src/modules/builtin_materialize.rs
phalcom-core/src/modules/registry.rs
phalcom-core/src/native/install.rs
phalcom-core/src/vm/bootstrap.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/mod.rs
phalcom-core/src/interpret.rs
phalcom-core/src/error.rs

phalcom-core/src/bytecode.rs
phalcom-core/src/callable.rs
phalcom-core/src/chunk.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/closure.rs
phalcom-core/src/heap/module.rs
phalcom-core/src/method/object.rs

phalcom-compiler-artifact/tests/executable_image_codec_test.rs
phalcom-core/tests/core/modules/executable_serialization.rs
phalcom-core/tests/core/modules/canonical_artifact.rs
phalcom-core/tests/artifact_allocation.rs
phalcom-core/tests/README.md
```

> Re-read named live paths before editing. Mechanical drift may be adapted locally. Any drift that changes authority, identity, lifetime, GC/rooting, VM execution semantics, or the artifact trust boundary is a STOP/CONSULT condition.

---

# 3. Exact takeover interface map

The following table is the implementation handoff map at `aad6a88db`.

| Concept | Current symbol / path | Current owner | Status / invariant |
|---|---|---|---|
| Complete portable executable DTO | `ExecutableImageV1` — `phalcom-compiler-artifact/src/executable/image.rs:52` | artifact crate | Wire data only; never runtime authority |
| Executable bounds | `ExecutableImageLimitsV1` — `image.rs:15` | artifact crate | Bounded structural decode |
| Executable compatibility | `ExecutableCompatibilityV1` — `executable/compatibility.rs` | artifact crate | Separate executable/schema/native ABI |
| Portable prototypes | `PortablePrototypeV1` — `executable/prototype.rs:66` | artifact crate | Carries bytecode, constant refs, frame layout, parameters, upvalues, source/semantic pool |
| Prototype stable owner | `PrototypeOwnerV1` — `prototype.rs:41` | artifact crate | Module initializer / canonical callable / nested parent+ordinal |
| Portable methods | `MethodRecipeV1` — `constant.rs:71` | artifact crate | Contains body/signature/runtime attachment fields but no canonical callable identity |
| Portable method body | `MethodBodyV1` — `constant.rs:67` | artifact crate | Closure template or native anchor |
| Portable class recipe | `PortableClassRecipeV1` — `module.rs:37` | artifact crate | Stable class ref + allocation/layout |
| Detached method row | `PortableDetachedMethodV1` — `module.rs:50` | artifact crate | Explicit canonical callable → method constant relation |
| Module executable row | `PortableModuleV1` — `module.rs:15` | artifact crate | Stable module + initializer + linked reads + source + eager bit |
| Bootstrap plan | `PortableBootstrapPlanV1` — `module.rs:60` | artifact crate | Native roles, eager modules, deferred modules, required declarations |
| Executable section tag | `SectionKind::Executable` — `section.rs:15` | artifact crate | Existing tag 7 |
| Portable executable capability | `TeloscCapabilities::PORTABLE_EXECUTABLE` — `header.rs:37` | artifact crate | Exists but remains reserved in current compiler-only loader |
| Staging transport | `TeloscContainerBuilder::stage_executable_section` — `container.rs:42` | artifact crate | Explicitly non-authoritative |
| Source executable capture entry | `capture_canonical_universe_executable()` — `capture.rs:103` | core | Builds through normal emitter in disposable source VM |
| Capture state | `ExecutableImageCapture` — `capture.rs:107` | core | Roots captured heap objects; projects recipes |
| Capture finalization | `ExecutableImageCapture::finish` — `capture.rs:972` | core | Completes class recipes, coverage, canonical graph IDs |
| Opcode adapter | `impl From<PortableInstructionV1> for Bytecode` — `artifact/instruction.rs` | core | Exhaustive portable→VM instruction conversion already exists |
| Core executable validator | `validate_executable_image` — `artifact/validate.rs:330` | core | Returns `Result<(), _>` today; does **not** mint authority |
| Intended validated wrapper | `ValidatedExecutableImage` — `validate.rs:139` | core | Private field prevents arbitrary construction; currently not produced |
| P7 validation context | `ExecutableValidationContext::new` — `validate.rs:171` | core | Currently accepts `&LoadedUniverseArtifact` |
| Detached canonical-owner check | `validate_detached_methods` — `validate.rs:968` | core | Closure target must be `PrototypeOwnerV1::Callable` for exact detached callable |
| Activation route proof | `classify_prototype_activation_roles` — `validate.rs:1445` | core | Supports selected `ReturnNonLocal` proof |
| Stack/CFG proof | `instruction_stack_transfer`, `validate_prototype_stack`, `validate_prototype_control_flow` | core | Substantial complete groundwork |
| Compiler-only loaded authority | `LoadedUniverseArtifact` — `canonical_artifact.rs:190` | core | Holds compiler image and metadata |
| Reserved executable slot | `LoadedUniverseArtifact::executable_reserved_p8: Option<()>` — line 195 | core | Exact seam to replace |
| Current compiler-only build | `build_universe_telosc` — `canonical_artifact.rs:266` | core | Emits four sections and compiler-only capabilities |
| Current loader | `load_universe_telosc*` — `canonical_artifact.rs:410+` | core | Requires exactly compiler-only capability set |
| Final loaded publication | `finish_loaded_universe_telosc` — `canonical_artifact.rs:595` | core | Currently always publishes executable `None` |
| Existing VM source bootstrap | `VM::try_new_with_native_install_mode_and_output_capture` — `vm/bootstrap.rs:204` | VM | Source-backed canonical boot path |
| Executable build helper | `VM::build_canonical_executable_payload` — `bootstrap.rs:188` | VM/core artifact build | Already drives capture |
| Runtime class allocation | `Bytecode::Class` handler — `vm/dispatch.rs:2474` | VM | **Always allocates fresh** for emitted `Class` |
| Runtime method installation | `Bytecode::Method` handler — `dispatch.rs:2676` | VM | Reads method from pop and class from stack top; derives metaclass for static |
| Variant method installation | `Bytecode::VariantMethod` handler — `dispatch.rs:3035` | VM | Reads method from stack; behavior class derives from semantic variant |
| Method installation helper | `VM::install_method_binding` — `dispatch.rs:660` | VM | Validates rest-family collision then mutates class dictionary |
| Runtime method object | `MethodObject` — `method/object.rs:448` | VM heap | holder/access owner/closure anchors are mutable installation results |
| Runtime closure object | `ClosureObject` — `heap/closure.rs:25` | VM heap | `Rc<Callable>`, module, upvalues, lexical class, super anchor |
| Runtime callable | `Callable` — `callable.rs:21` | compiler/VM | `Chunk`, frame/upvalue/parameter/name metadata |
| Module registry | `ModuleRegistry` / `ModuleRecord` — `modules/registry.rs` | VM module runtime | Owns prepared/initialized/failed once-only state |
| Source class slot reservation | `VM::prepare_universe_source_bindings` — `bootstrap.rs:337` | source bootstrap | Reserves module slots but deliberately keeps normal runtime class allocation |
| Native primitive install | `install_registered_primitives`, `refresh_registered_primitive_origins`, `install_source_owned_primitives` | native/install.rs | Existing native authority/order to split by input source |
| Core executable codec tests | `phalcom-compiler-artifact/tests/executable_image_codec_test.rs` | artifact crate tests | Already covers deterministic codec, N/N-1 decode, staging |
| Real capture test | `canonical_universe_executable_capture_structurally_round_trips` | core tests | Real source-built capture, deterministic repeated bytes |
| Existing fresh-process source denial harness | `phalcom-core/tests/core/modules/canonical_artifact.rs` | core integration | Best existing owner for source-free runtime acceptance |

---

# 4. Fixed architecture and ownership

## 4.1 Authority flow

The implementation must preserve this direction:

```text
P2 semantic/module/query/metadata authority
        ↓
Loaded canonical compiler image
        ↓ exact stable identities and compatibility
Portable executable validation
        ↓
ValidatedExecutableImage   [private, immutable proof authority]
        ↓
per-VM materialization/relocation
        ↓
fresh runtime objects
        ↓
normal VM bytecode execution
```

No runtime or materializer is allowed to reconstruct semantic truth that P2 already owns.

## 4.2 Source-of-truth table

| Fact | Canonical owner | Runtime consumer | Forbidden duplicate |
|---|---|---|---|
| Stable module identity | P2 module world | executable validator/materializer | runtime path/name inference |
| Stable declaration/class identity | P2 semantic metadata | executable validator | display class name as independent authority |
| Canonical callable identity | P2 callable metadata | method-install validator | selector-only or prototype-name identity |
| Opcode meaning | `Bytecode` + VM specs | portable manifest/validator | second hand-maintained semantics table |
| Opcode wire tag/operand domain | portable opcode declaration | decoder/validator | ad-hoc validator name checks |
| Method object identity | canonical executable constant recipe **plus materializer one-object-per-recipe invariant** | validator/materializer | signature equality |
| Fresh class allocation identity | validated `Class` allocation site tied to exact class authority | runtime execution | equal class names |
| Runtime holder/access owner | actual `Method` installation semantics | VM | inferred callable owner |
| Closure lexical/super anchors | actual installation semantics | VM | prefilled from canonical callable alone |
| Native primitive identity | compiled-in native catalog + executable native anchor | materializer/bootstrap | serialized function address |
| Module initialization state | `ModuleRegistry` / VM | eager/deferred bootstrap | artifact mutable state |
| Source text | source system only; absent from runtime artifact requirement | diagnostics may use logical URI/span | hidden runtime source fallback |

## 4.3 Global invariants

`P3-INV-01` — A decoded executable DTO is data, not authority.

`P3-INV-02` — Only successful completion of all core proof obligations may construct `ValidatedExecutableImage`.

`P3-INV-03` — `ValidatedExecutableImage` is bound to the exact loaded compiler-image/metadata authority against which it was checked.

`P3-INV-04` — Ordinary `Method` and `VariantMethod` validation is operand-tied. Existence of a matching selector/class/recipe elsewhere does not prove installation identity.

`P3-INV-05` — Detached closure method prototypes retain exact `PrototypeOwnerV1::Callable { callable == detached.callable }`.

`P3-INV-06` — `ReturnNonLocal` remains legal only when every activation route for the body is block-only and supplies the home-frame token. Mixed/unknown activation is rejected.

`P3-INV-07` — `Bytecode::Class` remains semantically fresh allocation. Artifact materialization must not silently turn it into a preexisting shared class unless an explicit VM contract is amended.

`P3-INV-08` — Canonical constant recipes are materialized at most once per VM per artifact `ConstantId`, so an object-producing recipe has stable within-VM object identity.

`P3-INV-09` — Two VMs loading one immutable artifact share no mutable runtime heap/module/global/cache state.

`P3-INV-10` — Primordial/native identities resolve to existing fresh-VM kernel/native objects; artifact materialization must not create shadow primordial classes.

`P3-INV-11` — Source-owned runtime setup preserves source bootstrap observable ordering and failure behavior.

`P3-INV-12` — Explicit artifact runtime mode never falls back to Universe source, `canonical_universe_program`, `UniverseSourceProvider::source_text`, or AST→bytecode compilation for canonical Universe modules.

`P3-INV-13` — Deferred canonical modules use persisted initializer prototypes and existing once-only module initialization state.

`P3-INV-14` — The same loaded artifact `Arc` supplies compiler authority and executable authority to an explicit build/run path.

`P3-INV-15` — Artifact decode/validation/materialization uses checked arithmetic, structural caps, cumulative budgets where allocations belong to load, and existing GC temporary-root discipline where VM allocations occur.

`P3-INV-16` — Full executable artifact publication is atomic: invalid bytes never replace a prior valid artifact.

`P3-INV-17` — Compiler-only artifacts remain loadable for compiler tooling; VM construction requires `PORTABLE_EXECUTABLE`.

`P3-INV-18` — The existing source-backed no-option VM path remains behaviorally intact until a later explicit cutover plan says otherwise.

---

# 5. Non-goals

P3 must **not** expand into:

- general incremental compilation of arbitrary projects;
- registry distribution;
- automatic artifact discovery;
- cache key policy or cache eviction;
- package-level executable artifacts beyond the canonical Universe case;
- P5 migration from `phalcom-core/core/universe` to `packages/universe`;
- a second bytecode emitter;
- a second semantic checker;
- serialized VM heap snapshots;
- persisted `ObjRef`, `ClassId`, `Symbol`, metadata pool ID, source ID, inline-cache contents, or global values;
- new GC policy;
- changing `Bytecode::Class` freshness semantics merely to simplify relocation;
- redesigning callable/method semantics;
- redesigning P2 query/image/session authority.

---

# 6. Critical proof design: ordinary `Method` / `VariantMethod`

This is the highest-risk part of P3 and must be implemented before runtime materialization.

## 6.1 Runtime facts that the proof must match

Current runtime behavior is concrete.

### `Bytecode::Method`

`vm/dispatch.rs:2676`:

```rust
Bytecode::Method(selector_idx, is_static) => {
    let selector_val = callable.chunk.constants[selector_idx as usize];
    let selector = selector_val.as_symbol().unwrap();
    let method_val = self.stack.pop().unwrap();
    let class_val = *self.stack.last().unwrap();

    if let (Some(method_id), Some(class_id)) = (method_val.as_obj(), class_val.as_obj()) {
        let target_class = if is_static {
            self.heap.class(class_id).class
        } else {
            class_id
        };

        // class-side:
        // holder = metaclass
        // access_owner = source class
        // lexical_class = source class
        // super_anchor = metaclass
        //
        // instance-side:
        // holder = source class
        // access_owner = source class
        // lexical_class = source class
        // super_anchor = source class

        self.install_method_binding(target_class, selector, method_id)?;
    }
}
```

Therefore validation must separately prove:

```text
actual popped method object
actual class object still on stack
selector constant
is_static side
derived target class (class or metaclass)
canonical callable identity
method body/signature/native anchor
```

It must **not** prove holder/access owner by equating them to one stable callable owner.

### `Bytecode::VariantMethod`

`vm/dispatch.rs:3035`:

```rust
Bytecode::VariantMethod { variant, selector } => {
    let variant_id =
        callable.chunk.executable_semantics.variant_target(variant).clone();
    let selector_sym =
        callable.chunk.constants[selector as usize].as_symbol().unwrap();
    let method_ref = self.pop()?.as_obj().ok_or(...)?;
    let runtime_var_id =
        self.adt_registry.variant_by_semantic(&variant_id).ok_or(...)?;
    let case_class_id =
        self.adt_registry.variant_descriptor(runtime_var_id).unwrap().behavior_class;
    self.heap.class_mut(case_class_id).methods.insert(selector_sym, method_ref);
}
```

There is no explicit class stack operand. The proof input is:

```text
method stack provenance
+ exact semantic variant operand
+ selector
→ canonical variant callable
→ exact behavior-class authority
```

Do not model this as `Method` with an invented class operand.

## 6.2 Abstract domain

Add a dedicated provenance dataflow domain rather than overloading `ReachableStackState` with identity semantics.

Recommended internal shape:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
enum ExecutableValueProvenance {
    Unknown,

    // Immediate/non-identity value. Useful so a known non-method/non-class
    // value cannot be mistaken for Unknown.
    Other,

    // One runtime method object reconstructed exactly once per VM from this
    // canonical executable constant recipe.
    MethodObject {
        recipe: ConstantId,
    },

    // Existing runtime class identity resolved from an immutable class recipe
    // or primordial role.
    StableClass {
        class: StableClassRefV1,
    },

    // Result of one actual Bytecode::Class instruction. Two sites are not equal
    // even when their display names are equal.
    FreshClass {
        class: StableClassRefV1,
        site: ClassAllocationSite,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct ClassAllocationSite {
    prototype: PrototypeId,
    instruction: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProvenanceFrame {
    stack: Vec<ExecutableValueProvenance>,
    locals: Vec<ExecutableValueProvenance>,
}
```

If scratch-local instructions physically insert/remove local stack positions in the VM, model their effect exactly in the same frame model. Do **not** invent a separate scratch namespace unless the runtime actually has one.

### Join

At a reachable CFG join:

```text
same exact fact from every predecessor → retain fact
otherwise                             → Unknown
```

For stack shape disagreement, keep using the existing hard validation error; do not merge different heights.

For locals absent/uninitialized on one predecessor, joined provenance is `Unknown`.

Do not use “candidate sets” that grow across joins. The selected architecture is **singleton proof or fail closed**, not path-set reasoning.

## 6.3 Seed rules

### `Constant`

Look up the prototype-local constant slot, then global `ConstantId`.

- `ConstantRecipeV1::Method` → `MethodObject { recipe }`.
- `ConstantRecipeV1::Class { reference }` → `StableClass { class: reference }`.
- all immediate/literal/non-identity constants → `Other`.
- a malformed reference has already failed structural validation and must not reach this phase.

### `Class`

The output is a fresh runtime class, never `StableClass` by equality of names.

Associate the instruction site to exactly one `PortableClassRecipeV1` with:

```text
allocation == SourceDeclared
```

and exact stable semantic owner.

The association algorithm must use canonical semantic identity, not a display-name heuristic. The current opcode carries only a symbol constant, so the implementer must verify the existing stable declaration encoding and compiler lowering relation:

```text
prototype stable module
+ exact Class declaration symbol operand
→ exact StableDeclarationRef
→ exact SourceDeclared PortableClassRecipeV1
```

This is acceptable only if the symbol operand is the canonical declaration component under the current semantic stable-identity contract. Verify against:

```text
phalcom-semantic/src/metadata/stable_identity...
compiler class declaration lowering/emission
capture::class_reference
PortableClassRecipeV1 producer
```

**STOP AND CONSULT** if the mapping requires source spans, runtime class display strings, prototype names, fuzzy path matching, or “the only class with this name” reasoning outside the stable-identity contract.

### Primordial/runtime classes

Class constants/roles that identify compiled-in primordial classes remain `StableClass`.

Do not mint `FreshClass` for primordial rows.

## 6.4 Identity-preserving transfers

The provenance analysis should preserve facts only through bytecodes whose runtime semantics are exact identity-preserving copies:

```text
Dup
GetLocal
SetLocal
GetLocalChecked          (same value, plus runtime init check)
ReserveScratchLocal      (index shift modeled exactly)
ReleaseScratchLocal      (index shift modeled exactly)
```

`SetLocal` must follow actual VM stack semantics. If it copies top-of-stack without popping, preserve both source stack fact and target local fact. If it pops, model the pop. Verify live dispatch before coding.

`GetUpvalue` and `GetGlobal` are **Unknown** unless an existing immutable artifact contract proves their exact object identity. P3 should not add global/upvalue alias analysis just to accept more bytecode.

Calls, sends, native invocations, associated/family dispatch, dynamic packs, field reads, construction operations, and semantic/runtime lookups produce `Unknown` unless their opcode contract proves a stronger identity fact required by P3.

## 6.5 Invalidation rules

Do not treat the whole frame as invalid merely because one unrelated dynamic operation occurs.

Instead:

- writes to a local overwrite that local fact;
- scratch insertion/removal shifts local indices exactly;
- a call result is `Unknown`;
- globals/upvalues are not used as identity-preserving storage unless proven;
- stack values not consumed by an opcode retain their provenance;
- method installation consumes the method value exactly as runtime does;
- class receiver remains on stack after `Method`, matching runtime.

Unsupported operations do not need to erase unrelated exact stack facts if their stack transfer is already known and they cannot mutate the identity of immutable handles. However, if an operation may replace/mutate a tracked storage location by alias in a way the proof cannot model, the affected fact becomes `Unknown`.

## 6.6 Installation proof product

Do not only return success/failure. Build a private derived binding table while validating:

```rust
#[derive(Clone, Debug)]
struct ValidatedMethodInstallation {
    method_recipe: ConstantId,
    callable: StableCallableRef,
    target: ValidatedMethodTarget,
    selector: String,
    side: StableDispatchSide,
    site: InstallationSite,
}

#[derive(Clone, Debug)]
enum ValidatedMethodTarget {
    Class(StableClassRefV1),
    Variant(StableVariantRef),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct InstallationSite {
    prototype: PrototypeId,
    instruction: u32,
}
```

The final exact field names are mechanically flexible; the information is not.

This table:

- records what the proof actually established;
- makes reuse/conflict detection deterministic;
- can be retained in `ValidatedExecutableImage` for materializer assertions/debugging;
- avoids recomputing canonical callable identity downstream.

## 6.7 Canonical callable derivation

At each `Method`:

1. obtain exact method provenance;
2. obtain exact class provenance;
3. resolve selector string from exact symbol constant;
4. derive dispatch side from `is_static`;
5. resolve the exact P7 callable key from:
   - canonical class declaration owner,
   - side,
   - selector;
6. require exact P7 membership;
7. require recipe signature selector/shape equality;
8. for closure bodies, require exact body prototype parameter/upvalue shape;
9. for native bodies, require exact native anchor identity/ABI;
10. record binding.

At each `VariantMethod`:

1. obtain exact method provenance;
2. resolve exact stable variant from portable semantic pool;
3. resolve selector;
4. derive exact variant callable owner;
5. require exact P7 callable membership;
6. require recipe/body/signature agreement;
7. record binding.

Do not resolve canonical callable by searching for “a P7 callable with this selector” globally.

## 6.8 Recipe reuse rule

Materialization will make one object-producing constant recipe one runtime object per VM. Therefore reuse of one `MethodRecipeV1` constant is identity reuse, not merely structural reuse.

Validation must group all installation proofs by `ConstantId`.

Allowed:

```text
one recipe
→ one exact canonical binding identity
```

Potentially allowed only after exact proof:

```text
same recipe used at multiple instruction sites
→ every site proves identical callable, target, selector, side
→ every site is inside once-only canonical module initialization
```

Rejected:

```text
same recipe
→ different class
→ different variant
→ different selector
→ different dispatch side
→ unknown target
```

### Verify-first rule

Before hard-coding “exactly one installation site per method recipe,” run a **read-only census** over the real canonical captured image. If current legal output has repeated sites for the same recipe, retain the more general same-binding rule above.

**STOP AND CONSULT** if current legal output reuses one method object for different canonical bindings. That would contradict the selected one-object-per-recipe/materialized-method identity model and require an architecture decision.

## 6.9 Fresh class execution cardinality

`Bytecode::Class` explicitly means fresh allocation each time it executes.

For executable-Universe canonical class identity, a `FreshClass` site may be accepted as the canonical source declaration only if its execution route is one-shot.

Minimum accepted proof boundary for P3:

- the site belongs to a `PrototypeOwnerV1::ModuleInitializer`;
- that initializer belongs to exactly one module row;
- module registry semantics execute that canonical initializer at most once per VM;
- the instruction is not in a CFG cycle that can execute the site repeatedly within that initializer;
- no second `Class` site maps to the same `SourceDeclared` class recipe.

Reject:

- class allocation in callable/nested prototypes when it would need repeated invocation reasoning;
- a class site inside a reachable cycle;
- two sites claiming the same stable class row;
- unsupported dynamic class provenance.

**STOP AND CONSULT** if the real canonical Universe image contains a required source-declared class allocation in a callable/nested prototype. Do not silently weaken the one-shot class identity contract.

---

# 7. Authority construction: exact validator interface change

## 7.1 Current code

Current wrapper:

```rust
pub(crate) struct ValidatedExecutableImage {
    image: Arc<ExecutableImageV1>,
}

impl ValidatedExecutableImage {
    pub(crate) fn image(&self) -> &ExecutableImageV1 {
        &self.image
    }
}
```

Current entry:

```rust
pub(crate) fn validate_executable_image(
    image: &ExecutableImageV1,
    loaded: &LoadedUniverseArtifact,
    limits: &ExecutableImageLimitsV1,
) -> Result<(), ExecutableValidationError>
```

Current context construction depends on a fully published `LoadedUniverseArtifact`:

```rust
fn new(
    loaded: &'a LoadedUniverseArtifact
) -> Result<Self, ExecutableValidationError>
```

That shape is wrong for atomic full-artifact loading because executable validation must finish **before** publishing the final `LoadedUniverseArtifact`.

## 7.2 Required refactor

Make the validation context depend on the underlying immutable authorities it actually consumes:

```rust
impl<'a> ExecutableValidationContext<'a> {
    fn new(
        formal_image: &'a CanonicalUniverseFormalImage,
        metadata: &'a SemanticMetadataBundle,
    ) -> Result<Self, ExecutableValidationError>;
}
```

Then make the validator consume owned immutable image bytes and mint authority:

```rust
pub(crate) fn validate_executable_image(
    image: Arc<ExecutableImageV1>,
    formal_image: &CanonicalUniverseFormalImage,
    metadata: &SemanticMetadataBundle,
    limits: &ExecutableImageLimitsV1,
) -> Result<ValidatedExecutableImage, ExecutableValidationError>;
```

Recommended final wrapper:

```rust
pub(crate) struct ValidatedExecutableImage {
    image: Arc<ExecutableImageV1>,
    method_installations: Box<[ValidatedMethodInstallation]>,
    class_sites: Box<[ValidatedClassAllocation]>,
}

impl ValidatedExecutableImage {
    pub(crate) fn image(&self) -> &ExecutableImageV1;
    pub(crate) fn image_arc(&self) -> Arc<ExecutableImageV1>;

    // Narrow internal access only if materialization actually needs proof tables.
    pub(crate) fn method_installations(&self) -> &[ValidatedMethodInstallation];
}
```

The exact use of `Box`, `Arc`, or frozen vectors is mechanically flexible. The fixed rule is that callers cannot construct the wrapper from arbitrary DTOs.

### Why `Arc<ExecutableImageV1>` enters validation

The loader and one/many VMs need to share the immutable validated executable without copying it. The wrapper should own the exact image instance that was checked.

Do **not** validate `&ExecutableImageV1`, later clone it, and treat the clone as implicitly authorized unless equality/copy semantics are made part of the trust boundary. Pass ownership into proof and retain that exact object.

## 7.3 Validation ordering

Recommended exact order:

```text
1. artifact structural validation
2. executable internal ABI/native digest/profile
3. P2 module coverage / linked-read equality
4. native anchors / primordial roles
5. class recipes
6. prototype owners / closure captures
7. method recipe shape
8. detached-method canonical-owner relation
9. required declarations / canonical semantic roots
10. semantic pools
11. stack/frame/control-flow proof
12. ReturnNonLocal activation proof
13. ordinary Method/VariantMethod provenance proof
14. class allocation one-shot proof
15. method-recipe binding conflict proof
16. construct ValidatedExecutableImage
```

If existing helper order naturally differs, keep it only if no later proof assumes facts not yet validated.

## 7.4 Error design

Add typed errors for proof failures rather than one giant string bucket. At minimum distinguish:

```rust
UnknownMethodProvenance { prototype, instruction }
UnknownClassProvenance { prototype, instruction }
ConflictingProvenanceJoin { prototype, instruction, slot }
MethodBindingMismatch { prototype, instruction, detail }
VariantMethodBindingMismatch { prototype, instruction, detail }
ConflictingMethodRecipeBinding { constant, first, second }
RepeatedClassAllocation { class, first, second }
CyclicClassAllocationSite { prototype, instruction }
UnsupportedClassAllocationOwner { prototype, instruction }
```

They may still be rendered through `InvalidExecutableReference` internally if the project strongly prefers one error enum, but tests need to discriminate the causal family.

---

# 8. Full `.telosc` build/load integration

## 8.1 Current transport facts

Already implemented:

```rust
SectionKind::Executable        // tag 7
TeloscCapabilities::PORTABLE_EXECUTABLE
TeloscContainerBuilder::stage_executable_section(...)
```

`stage_executable_section` explicitly refuses to treat staged bytes as authorized capability:

```rust
if self.header.capabilities.contains(TeloscCapabilities::PORTABLE_EXECUTABLE) {
    return Err(ContainerError::UnvalidatedExecutableCapability.into());
}
```

That is a useful producer-side guard, not the security boundary. The loader remains the trust boundary.

## 8.2 Preserve compiler-only builder

Do not change the existing semantics of:

```rust
pub fn build_universe_telosc(
    view: &CanonicalCompilerImageView<'_>,
    limits: &ValidationLimits,
) -> Result<BuiltUniverseTelosc, TeloscArtifactError>
```

It should continue producing the compiler-only artifact for tooling/tests that explicitly ask for it.

Add a full-build path instead of making every current caller accidentally pay source VM executable capture.

Recommended public API:

```rust
pub fn build_full_universe_telosc(
    view: &CanonicalCompilerImageView<'_>,
    limits: &ValidationLimits,
) -> Result<BuiltUniverseTelosc, TeloscArtifactError>;
```

Implementation decomposition:

```rust
fn build_universe_telosc_inner(
    view: &CanonicalCompilerImageView<'_>,
    executable: Option<&ValidatedBuildExecutable>,
    limits: &ValidationLimits,
) -> Result<BuiltUniverseTelosc, TeloscArtifactError>;
```

`ValidatedBuildExecutable` is a core-local build-time proof token or equivalent internal value. Do not export it as public artifact authority.

### Full builder sequence

```text
canonical compiler image view
    ↓
build four existing compiler sections
    ↓
capture_canonical_universe_executable()
    ↓
decode captured payload to ExecutableImageV1 if needed for proof
    ↓
validate against same canonical formal image + metadata
    ↓
final container assembly:
    - existing four sections
    - EXECUTABLE required section
    - PORTABLE_EXECUTABLE capability
    ↓
parse/self-load final bytes
    ↓
only then return BuiltUniverseTelosc
```

If capture already has `ExecutableImageV1` internally before staging, avoid encode→decode solely to validate. Introduce a narrow internal capture API that returns the image plus a staging adapter:

```rust
pub(crate) fn capture_canonical_universe_executable_image(
) -> Result<ExecutableImageV1, ExecutableCaptureError>;
```

Keep the existing public staged helper if tests/users need it:

```rust
pub fn capture_canonical_universe_executable(
) -> Result<StagedExecutableSectionV1, ExecutableCaptureError>
```

The staged helper can call the image helper then encode.

## 8.3 Producer-side authorization

Because `phalcom-compiler-artifact` cannot know P7 semantic authority, it cannot itself prove that executable bytes are safe.

Therefore core may assemble a final container with `PORTABLE_EXECUTABLE` **only after core validation**.

Do not weaken `stage_executable_section` merely to make the build easy.

Two acceptable mechanical implementations:

### Option A — explicit builder finalization API

Artifact crate:

```rust
pub fn add_authorized_executable_section(
    &mut self,
    schema_version: u32,
    payload: Vec<u8>,
) -> Result<(), ArtifactError>;
```

This method may require the capability already be set and reject duplicates. Its documentation must state that caller-side semantic/core validation is required.

### Option B — core uses generic `add_section`

After core proof, use:

```rust
builder.add_section(
    SectionKind::Executable,
    EXECUTABLE_SCHEMA_V1,
    SectionFlags::REQUIRED,
    executable_bytes,
);
```

with header `PORTABLE_EXECUTABLE`.

Option A is preferable because it prevents accidental misuse and centralizes duplicate/schema/flag rules. It is not itself a security proof.

## 8.4 Header/capability policy

Compiler-only accepted set:

```text
CANONICAL_COMPILER_IMAGE_V1
| RUNTIME_PUBLIC_METADATA
```

Full executable accepted set:

```text
CANONICAL_COMPILER_IMAGE_V1
| RUNTIME_PUBLIC_METADATA
| PORTABLE_EXECUTABLE
```

Refactor `validate_reader_header` to accept exactly these two known profiles and reject unknown capability combinations.

Do not make it “contains required bits” while silently ignoring arbitrary extra capability bits.

Recommended helper:

```rust
enum UniverseArtifactCapabilityProfile {
    CompilerOnly,
    CompilerAndExecutable,
}

fn validate_reader_header(
    header: &TeloscHeaderV2,
) -> Result<UniverseArtifactCapabilityProfile, TeloscArtifactError>;
```

## 8.5 Loader sequence

Current loader constructs the formal image before final publication. That is the correct seam.

After:

```rust
let formal_image =
    CanonicalUniverseFormalImage::from_loaded_artifact(...)?;
let metadata_bundle = allocations.arc(metadata_bundle)?;
```

but before final `LoadedUniverseArtifact` allocation:

```text
if capability profile is full:
    require EXECUTABLE
    check section schema == EXECUTABLE_SCHEMA_V1
    decode ExecutableImageV1 with allocation budget
    core-validate against formal_image + metadata_bundle
    retain validated executable
else:
    reject a REQUIRED executable capability mismatch
    optional staged executable section may be ignored/rejected according to explicit compiler-only policy
```

For a production compiler-only artifact, a stray executable staging section should preferably be rejected at the canonical Universe loader after P3 freeze; keeping silently ignored executable bytes creates confusing provenance. If historical tests need staged transport, test it at the container layer, not as an accepted canonical production artifact.

## 8.6 Replace reserved field

Current:

```rust
pub executable_reserved_p8: Option<()>,
```

Required:

```rust
pub(crate) executable: Option<Arc<ValidatedExecutableImage>>,
```

Expose narrow capability methods instead of the private type publicly:

```rust
impl LoadedUniverseArtifact {
    pub fn has_portable_executable(&self) -> bool {
        self.executable.is_some()
    }

    pub(crate) fn validated_executable(
        &self
    ) -> Result<&ValidatedExecutableImage, TeloscArtifactError>;
}
```

Do not return raw `ExecutableImageV1` from a public API intended for VM construction.

## 8.7 Final loaded publication helper

Avoid putting a `pub(crate)` validated type in a public function signature.

Refactor into:

```rust
fn finish_loaded_universe_telosc_inner(
    capabilities: TeloscCapabilities,
    formal_image: Arc<CanonicalUniverseFormalImage>,
    metadata: Arc<SemanticMetadataBundle>,
    executable: Option<Arc<ValidatedExecutableImage>>,
    timings: TeloscLoadTimings,
    allocation_budget: &mut AllocationBudget,
) -> Result<Arc<LoadedUniverseArtifact>, TeloscArtifactError>;
```

Keep a public compatibility wrapper only if live callers require the existing `finish_loaded_universe_telosc` API.

**VERIFY-FIRST:** search all branch callers before changing visibility/signature.

## 8.8 Full-load allocation accounting

The existing P2 exact loader accounting ends at compiler-only artifact publication. P3 adds:

- executable section decode;
- core validation scratch;
- validated proof tables;
- executable publication.

The full executable loader therefore needs new named allocation stages.

Required policy:

- retained executable DTO/proof allocations inside the artifact load transaction are budgeted;
- temporary validation scratch must either use budget-aware containers or be explicitly scoped outside the retained load transaction **only if the resource contract says so**;
- never “observe then adjust” the ledger;
- N/N-1/retry must be rerun for the full artifact load after integration.

Preferred stage additions:

```text
executable DTO decode
executable P7/core validation
validated executable publication
```

If core validator BTreeMap/BTreeSet scratch is not currently budget-aware, this is a known implementation obligation, not permission to ignore it.

---

# 9. Materialization architecture

Create:

```text
phalcom-core/src/artifact/materialize.rs
```

and register:

```rust
pub(crate) mod materialize;
```

in `artifact/mod.rs`.

## 9.1 Materialization contract

Input:

```rust
&Arc<LoadedUniverseArtifact>
&ValidatedExecutableImage
&mut VM
```

Output:

a VM-local relocation/recipe registry, with no partially published canonical module execution state on failure.

Recommended API:

```rust
pub(crate) struct VmArtifactRelocation {
    // exact types may vary after live API verification
    modules: BTreeMap<StableModuleRef, ObjRef>,
    classes: BTreeMap<StableClassRefV1, ClassId>,
    constants: Vec<Option<Value>>,
    callables: Vec<Option<Rc<Callable>>>,
    initializer_closures: BTreeMap<StableModuleRef, ObjRef>,
    source_ids: Vec<u32>,
    semantic_pools: Vec<MetadataPoolId>,
}

pub(crate) fn prepare_universe_artifact_runtime(
    vm: &mut VM,
    artifact: &Arc<LoadedUniverseArtifact>,
) -> Result<VmArtifactRelocation, VmBootstrapError>;
```

Do not assume all fields above are necessary; verify live APIs. The **identity rules** are fixed even if storage differs.

## 9.2 Important correction to the retired P8 materialization wording

Do **not** blindly preallocate all `SourceDeclared` classes before executing their `Bytecode::Class`.

The live bytecode contract explicitly says:

> every executed `Class` allocates fresh.

The source bootstrap also only reserves source class **module slots** in `prepare_universe_source_bindings`; it preserves normal class allocation and initialization.

Therefore P3 should follow this runtime-compatible model:

### Before initializer execution

Materialize:

- module objects and declared slots/exports;
- linked-read tables;
- source-map/diagnostic logical records;
- metadata pools needed by bytecode;
- portable class **layout plans** (`field_layouts`, abstract/layout facts);
- primordial class references to existing VM objects;
- constant/prototype recipes that do not require as-yet-unallocated source classes;
- initializer callable/closure shells.

### At `Bytecode::Class`

Keep normal fresh allocation behavior.

Use the validated class-site mapping to assert/record:

```text
this actual allocation site → this stable SourceDeclared class recipe
```

Then apply/reuse the preloaded class layout plan through existing `create_class`/`field_layouts`/finalization machinery.

### After source class exists

Any later constant recipe requiring that stable class can resolve through the VM-local stable-class relocation table.

If current chunk construction requires every constant eagerly before the class exists, implement **lazy/dependency-ordered constant materialization**, not premature source-class allocation.

**STOP AND CONSULT** if preserving normal `Bytecode::Class` freshness appears impossible without changing bytecode semantics.

## 9.3 Constant materialization

Implement one canonical constant materializer keyed by global `ConstantId`.

Proposed shape:

```rust
fn materialize_constant(
    &mut self,
    id: ConstantId,
) -> Result<Value, VmBootstrapError>;
```

Use memoization:

```text
Unseen
Visiting
Ready(Value)
```

Even though wire validation rejects constant cycles, keep `Visiting` as a defensive invariant and typed error rather than stack overflow.

Families:

- immediate `None`, `Unit`, booleans, integer bits, float bits, character;
- symbol → VM interner;
- string / bigint → fresh VM heap object;
- `Class` → exact relocation entry, never name lookup;
- `Module` → exact module relocation entry;
- `ClosureTemplate` → closure with `Rc<Callable>`, exact module, no persisted captures;
- `Method` → fresh per-VM `MethodObject`;
- selector pattern;
- vector/frozen-array/tuple/record/pack immutable constants;
- private `Nil` only where wire contract permits internal sentinel.

### One-object-per-recipe invariant

Object-producing `ConstantId`s are memoized globally within one VM artifact materialization.

This is the runtime fact that makes validator `MethodObject { recipe }` provenance sound.

Add a direct test asserting two prototype constant slots referring to the same method/global recipe receive the same `ObjRef` within one VM, while another VM receives a different `ObjRef`.

## 9.4 Prototype materialization

Existing conversion is already exhaustive:

```rust
impl From<PortableInstructionV1> for Bytecode
```

Build:

```rust
fn materialize_callable(
    &mut self,
    prototype: PrototypeId,
) -> Result<Rc<Callable>, VmBootstrapError>;
```

Required fields:

```text
Chunk.code                ← portable code.into()
Chunk.constants           ← per-prototype ConstantId relocation
Chunk.spans               ← portable spans
Chunk.source_id           ← VM-local diagnostic source id
Chunk.caches              ← empty
Chunk.gcaches             ← empty
Chunk.executable_semantics← reconstructed fresh semantic pool

Callable.max_slots
Callable.num_upvalues
Callable.upvalues
Callable.arity
Callable.parameter_shape
Callable.name_sym
Callable.local_names
```

Never restore emitted VM cache contents.

## 9.5 Closure templates

`ClosureObject` runtime shape:

```rust
pub struct ClosureObject {
    pub callable: Rc<Callable>,
    pub module: ObjRef,
    pub upvalues: Vec<ObjRef>,
    pub lexical_class: Option<ClassId>,
    pub super_anchor: Option<ClassId>,
    ...
}
```

For persisted closure templates:

- `upvalues` must begin empty;
- nested captures are filled only when `Bytecode::Closure` executes;
- lexical/super class refs relocate from validated stable refs;
- module relocates from exact `StableModuleRef`;
- no build-VM `ObjRef` survives.

## 9.6 Method recipes

Construct:

```rust
MethodObject::new_single(signature, MethodKind::Closure(closure))
```

or native primitive equivalent resolved from the validated native anchor.

Then apply portable non-installation metadata:

```text
visibility
origin
contracts
attributes
attributes_frozen
```

Treat serialized `holder` / `access_owner` as consistency witnesses only to the extent already authorized by validation. Actual install instructions remain authoritative for installation-derived mutable fields.

For ordinary method closure bodies, do not pre-set installation-derived lexical/super anchors to bypass `Bytecode::Method`.

For detached methods that are not installed through bytecode, follow their existing runtime consumer semantics and the detached callable proof.

## 9.7 Semantic-pool reverse materialization

`artifact/semantic.rs` currently projects runtime semantic lowering structures into `PortableSemanticPoolV1`. P3 needs the inverse.

Do **not** duplicate semantic analysis.

Implement an artifact-local decoder that reconstructs the exact runtime lowering recipes already represented by the portable pool:

```rust
fn materialize_semantic_pool(
    vm: &mut VM,
    artifact: &LoadedUniverseArtifact,
    pool: &PortableSemanticPoolV1,
) -> Result<ExecutableSemanticPool, VmBootstrapError>;
```

Use the loaded P2 semantic snapshot/metadata for canonical type/kind/declaration identities.

Reconstruct:

```text
type recipes
kind recipes
enum specs
variant targets
data specs
data targets
data constructions
anonymous product specs
call environments
associated targets
family descriptors
conditional family descriptors
family operations
family candidate sets
trait invocations
trait requirement invocations
type-parameter class receivers
```

All mutable runtime caches created by these structures start empty.

If a portable row lacks enough information to reconstruct a runtime structure without re-running semantic analysis, **STOP AND CONSULT**. Do not “look it up approximately” from source or rebuild semantic logic in core.

## 9.8 GC/rooting

Materialization allocates many interdependent heap objects. Use existing VM temporary-root APIs.

Pattern:

```rust
let root_depth = vm.temp_root_depth();

let result = (|| {
    // allocate object
    vm.push_temp_root(Value::obj(obj));
    // allocate dependencies / wire edges
    ...
    Ok(...)
})();

vm.restore_temp_root_depth(root_depth);
result
```

Use the exact live API name for root restoration after verification.

No new GC scheme.

Do not publish module initializer closure handles into the runtime module registry until all objects needed for that module have been built successfully.

---

# 10. Native/primordial source-free bootstrap

Current source boot order:

```text
canonical_universe_program
→ new native VM
→ capture native anchors (build only)
→ prepare_universe_source_bindings
→ prepare_universe_static_constants
→ refresh_registered_primitive_origins(source_index)
→ install semantic metadata
→ compile/run Universe modules
→ install_source_owned_primitives(source_index)
→ bind native error constructors
→ bind canonical variant aliases
→ sync class aliases
→ establish semantic roots
→ finalize pristine flags/invariants
```

Artifact mode must preserve the observable order but replace source-index inputs.

## 10.1 Split source-independent native work from source authority adapters

Existing:

```rust
install_registered_primitives(vm)
refresh_registered_primitive_origins(vm, source_index)
install_source_owned_primitives(vm, source_index)
```

Required direction:

```rust
install_registered_primitives(vm)                 // stays
refresh_registered_primitive_origins_from_artifact(...)
install_source_owned_primitives_from_artifact(...)
```

Prefer factoring the common mutation into owner-based helpers:

```rust
fn refresh_registered_primitive_origins_with(
    vm: &mut VM,
    resolve_origin: impl Fn(PrimitiveKey) -> Result<RuntimeOrigin, PhError>,
) -> PhResult<()>;

fn install_source_owned_primitives_with(
    vm: &mut VM,
    resolve_owner: impl Fn(PrimitiveKey) -> Result<ClassId, PhError>,
) -> PhResult<()>;
```

Source path supplies `NativeSourceIndex`; artifact path supplies validated native anchor/class relocation.

Do not duplicate the primitive catalog iteration.

## 10.2 Primordial roles

`PortableClassAllocationV1::Primordial { role }` resolves to the VM’s already-created native/kernel Universe class object.

Required test:

```text
artifact recipe for primordial Object/Number/etc
→ exact existing VM class handle
→ no second class allocated
```

## 10.3 Source class slot reservation

Implement artifact equivalent of `prepare_universe_source_bindings` from stable module/declaration facts.

Do not require source census.

Possible API:

```rust
fn prepare_universe_artifact_bindings(
    &mut self,
    executable: &ValidatedExecutableImage,
    relocation: &mut VmArtifactRelocation,
) -> Result<(), VmBootstrapError>;
```

The exact slot declarations must agree with loaded module authority and portable linked reads.

## 10.4 Static constants

Current source function mainly installs module lowering metadata before initializers. Artifact mode should use reconstructed semantic pools/P2 static authority.

Do not evaluate source static AST.

---

# 11. Artifact-backed VM constructor

Add the explicit constructor only after materialization and native-floor helpers exist.

Recommended API:

```rust
impl VM {
    pub fn try_new_with_universe_artifact(
        artifact: Arc<LoadedUniverseArtifact>,
    ) -> Result<Self, VmBootstrapError>;

    pub fn try_new_with_universe_artifact_and_output(
        artifact: Arc<LoadedUniverseArtifact>,
        output: Box<dyn RuntimeOutput>,
    ) -> Result<Self, VmBootstrapError>;
}
```

The public shape may be narrowed to one method if project API style prefers.

## 11.1 Constructor sequence

```text
1. require artifact.has_portable_executable()
2. VM::new_kernel_with_output
3. install source-independent native runtime
4. resolve validated primordial/native identities
5. install module objects/slots/linked reads from loaded module authority
6. install semantic metadata bundle
7. reconstruct class layout plans
8. prepare artifact source-class bindings
9. materialize eager initializer prototypes as dependencies become resolvable
10. execute eager bootstrap modules in stored order
11. install source-owned primitive attachments from artifact authority
12. bind native error constructors
13. bind canonical variant aliases
14. sync Universe class aliases
15. establish semantic roots from exact runtime exports
16. mark pristine optimization state
17. verify Universe invariants
18. retain Arc<LoadedUniverseArtifact> for deferred modules/diagnostics
19. return VM
```

## 11.2 Failure semantics

A failed VM bootstrap does not mutate the immutable artifact.

The half-built VM is dropped.

Retry with the same `Arc<LoadedUniverseArtifact>` constructs a fresh VM from scratch.

Do not persist failed initialization state in the artifact.

## 11.3 Error enum

Extend `VmBootstrapError` with a typed artifact case rather than converting every failure into `Invariant(String)`.

Recommended:

```rust
#[error("Universe artifact bootstrap failed: {0}")]
Artifact(String),
```

Better, if dependency visibility permits:

```rust
#[error(transparent)]
Artifact(#[from] UniverseArtifactRuntimeError),
```

Keep source `CanonicalUniverse`, runtime `Runtime`, and invariant variants intact.

---

# 12. Deferred canonical Universe modules

Current full source bootstrap differentiates eager and deferred module behavior. The executable image already carries:

```rust
bootstrap.eager_modules
bootstrap.deferred_modules
PortableModuleV1.initializer
```

P3 must persist runtime prototype authority for every canonical module.

## 12.1 VM-retained artifact authority

Add a VM field or an owned registry object for the loaded artifact only on artifact-backed VMs:

```rust
universe_artifact: Option<Arc<LoadedUniverseArtifact>>
```

If a narrower existing runtime-owner object is better, use it. The invariant is that deferred prototype lookup remains tied to the same validated image used at VM construction.

## 12.2 Import/materialization path

At canonical Universe import:

```text
module registry says already initialized → reuse
prepared/deferred canonical module       → obtain persisted initializer
                                             ↓
                                         materialize if lazy
                                             ↓
                                         run through existing once-only state machine
source-backed ordinary user module       → existing compiler path
```

Modify the lowest existing runtime module initialization owner that currently falls back to compiling canonical Universe source. Candidate paths:

```text
phalcom-core/src/interpret.rs::run_compiled
phalcom-core/src/modules/materialize.rs
phalcom-core/src/modules/initialize.rs
```

Do not add the check at many call sites.

## 12.3 Missing prototype behavior

For an artifact-backed canonical module:

```text
missing initializer prototype
→ typed artifact/runtime error
```

Never:

```text
missing prototype
→ compile Universe source
```

This is a hard source-denial invariant.

---

# 13. Explicit compiler + VM API integration

The explicit artifact path must use one loaded immutable artifact for both sides.

Conceptual API:

```rust
let artifact = load_universe_telosc(path, limits)?;

let compiled_user_program =
    artifact.compile_user_entry_with_options(entry, options)?;

let mut vm =
    VM::try_new_with_universe_artifact(artifact.clone())?;

vm.run_compiled(&compiled_user_program)?;
```

This is the architectural target.

Do not independently reload the artifact for VM construction.

Do not compile user code against one image and run against another artifact instance.

---

# 14. CLI implementation requirements

The prior architecture expects explicit commands, not automatic cache discovery.

Before editing, verify actual CLI owner. Current prior plan named `phalcom-core/bin/phalcom/cli.rs`; repository structure may have drifted.

Required user-facing behaviors:

```text
phalcom artifact build-universe --output universe.telosc
phalcom --universe-artifact universe.telosc app.ph
```

or mechanically equivalent spelling already ratified by current CLI specs.

## Build command

Must:

1. construct canonical compiler image;
2. capture full executable;
3. validate full executable;
4. build one full artifact;
5. self-load/self-validate final bytes;
6. atomically write destination.

## Run option

Must:

1. load exactly one artifact;
2. reject compiler-only artifact for runtime;
3. compile user code from its formal image;
4. bootstrap VM from its validated executable;
5. run user code;
6. never fallback if explicit artifact load/bootstrap fails.

No-option behavior remains current source/default behavior under P3.

---

# 15. File-by-file implementation impact map

## 15.1 Definitely modify

### `phalcom-core/src/artifact/validate.rs`

Implement:

- provenance domain;
- CFG provenance fixed point;
- exact join;
- `Method`/`VariantMethod` installation proof;
- class-site mapping and one-shot proof;
- method-recipe conflict table;
- final authority construction;
- context refactor away from `LoadedUniverseArtifact`.

### `phalcom-core/src/artifact/mod.rs`

Register materializer.

### `phalcom-core/src/artifact/capture.rs`

Likely expose internal raw `ExecutableImageV1` capture before staging, while preserving existing staging helper.

### `phalcom-core/src/modules/canonical_artifact.rs`

Implement:

- full artifact builder;
- capability profile validation;
- required EXECUTABLE section load;
- bounded executable decode;
- core validation before publication;
- replace `executable_reserved_p8`;
- full artifact self-validation;
- load timing/allocation stage additions.

### `phalcom-core/src/vm/bootstrap.rs`

Implement explicit artifact-backed constructor and source-free bootstrap helpers.

### `phalcom-core/src/native/install.rs`

Factor source-index-dependent native origin/owner installation behind common owner-based helpers.

### `phalcom-core/src/modules/materialize.rs` and/or `modules/initialize.rs`

Integrate stored canonical initializer path if that is the true once-only runtime owner.

### `phalcom-core/src/interpret.rs`

Only if the canonical import fallback is owned here after live call-path verification.

### `phalcom-core/src/error.rs`

Add typed artifact bootstrap/materialization errors.

## 15.2 Create

```text
phalcom-core/src/artifact/materialize.rs
```

Optionally split only if file size/ownership becomes unwieldy:

```text
phalcom-core/src/artifact/materialize/constants.rs
phalcom-core/src/artifact/materialize/semantic.rs
phalcom-core/src/artifact/materialize/tests.rs
```

Do not preemptively create a directory hierarchy unless the single owner becomes too large.

## 15.3 Possibly modify

### `phalcom-compiler-artifact/src/container.rs`

Add dedicated authorized-executable section insertion helper if preferred over generic `add_section`.

### `phalcom-compiler-artifact/src/header.rs`

Only comments/profile helpers; no new capability bit is needed.

### `phalcom-compiler-artifact/src/executable/*`

Only if live implementation uncovers a genuine missing portable recipe. Such a finding is a STOP/CONSULT condition if it changes schema/identity.

### `phalcom-core/src/vm/mod.rs`

Retain artifact `Arc` or per-VM relocation registry if no existing field can own it.

### CLI source files

Verify actual location before editing.

## 15.4 Must not require semantic-model changes under expected path

P3 should not need material changes to:

```text
phalcom-semantic type system
query model
P2 canonical snapshot semantics
module stable identity schema
```

If such changes become necessary for executable runtime reconstruction, STOP AND CONSULT.

---

# 16. Test architecture

Testing must be designed around invariants, not around files changed.

## 16.1 Coverage ledger

| ID | Dimension | Invariants | Required case |
|---|---|---|---|
| CV-01 | canonical success | 01–06 | real captured executable validates and mints authority |
| CV-02 | method identity | 04,08 | actual `Method` operands resolve exact callable |
| CV-03 | conflicting join | 04 | two path method/class facts merge to Unknown and reject |
| CV-04 | alias preservation | 04 | local/Dup-preserved method/class identities validate |
| CV-05 | unsupported provenance | 04 | global/upvalue/call-derived install rejects |
| CV-06 | recipe reuse | 08 | conflicting binding for one method recipe rejects |
| CV-07 | class freshness | 07 | repeated/cyclic/duplicate source class site rejects |
| CV-08 | detached owner | 05 | wrong callable owner rejects |
| CV-09 | non-local return | 06 | block-only accepts; mixed/unknown activation rejects |
| CV-10 | artifact capability | 01,17 | compiler-only loads for compiler; VM constructor rejects it |
| CV-11 | full load | 02,03,15 | full artifact EXECUTABLE decodes, validates, publishes |
| CV-12 | malformed full load | 02 | corrupt executable never mints loaded runtime authority |
| CV-13 | allocation transaction | 15 | full load N passes, N-1 fails atomically, retry passes |
| CV-14 | constant identity | 08,09 | same ConstantId same ObjRef within VM, different across VMs |
| CV-15 | primordial identity | 10 | no shadow primordial class |
| CV-16 | runtime parity | 11 | source/artifact eager bootstrap observable parity |
| CV-17 | source denial | 12 | physical/provider source access denied yet artifact VM succeeds |
| CV-18 | deferred import | 13 | non-eager Universe module executes persisted prototype once |
| CV-19 | two-VM isolation | 09 | globals/caches/classes fresh per VM as specified |
| CV-20 | one-Arc authority | 14 | compiler and VM retain same loaded artifact identity |
| CV-21 | failure recovery | 09,16 | failed VM bootstrap does not poison artifact; retry succeeds |
| CV-22 | diagnostics | 12 | runtime trace/source locations work without source text |
| CV-23 | determinism | 16 | independent full builds byte-identical |
| CV-24 | no regression source path | 18 | default no-artifact VM still behaves as before |

---

# 17. Exact test placement

## 17.1 Artifact codec / container tests

Extend:

```text
phalcom-compiler-artifact/tests/executable_image_codec_test.rs
```

Add:

- required executable section/capability combinations;
- duplicate executable section;
- wrong executable schema version;
- bounds/overflow for newly integrated full section;
- staging remains non-authoritative;
- deterministic encode remains exact.

Do **not** place P7 callable identity tests here; artifact crate cannot own semantic proof.

## 17.2 Core validator unit tests

Create:

```text
phalcom-core/src/artifact/validate/tests.rs
```

with:

```rust
#[cfg(test)]
mod tests;
```

inside `validate.rs`.

Why unit tests:

- validator and proof product remain `pub(crate)`;
- tests can inspect exact private failure modes without exporting a test-only public API;
- minimal synthetic images can target abstract dataflow.

Required focused cases:

```text
valid_method_direct_constant
valid_method_through_dup
valid_method_through_local
method_unknown_from_global_rejected
method_conflicting_cfg_join_rejected
method_wrong_selector_rejected
method_wrong_side_rejected
method_wrong_class_rejected
method_recipe_conflicting_reuse_rejected

variant_method_exact_variant_callable
variant_method_wrong_variant_rejected
variant_method_unknown_method_rejected

source_class_unique_initializer_site_accepted
source_class_duplicate_site_rejected
source_class_cycle_rejected
source_class_non_initializer_owner_rejected

detached_closure_wrong_callable_owner_rejected
return_non_local_mixed_activation_rejected
```

Use small synthetic fixtures where possible.

Add exactly one real captured-image acceptance test at a higher integration layer; do not make every unit test bootstrap the real Universe.

## 17.3 Real executable capture/validation

Extend:

```text
phalcom-core/tests/core/modules/executable_serialization.rs
```

Current test already performs real deterministic capture and structural roundtrip.

Add a test that:

1. obtains the canonical compiler image;
2. loads/builds the compiler authority required by the validator;
3. validates the captured executable;
4. proves the good canonical image mints validated authority;
5. optionally reports installation/class-site census.

Avoid printing unstable object handles.

## 17.4 Full artifact loader

Extend:

```text
phalcom-core/tests/core/modules/canonical_artifact.rs
```

Add:

```text
full_universe_telosc_loads_portable_executable_authority
compiler_only_artifact_remains_compiler_usable_but_not_vm_bootstrappable
full_artifact_rejects_corrupt_executable_before_publication
full_artifact_rejects_executable_abi_mismatch
full_artifact_rejects_native_anchor_mismatch
```

## 17.5 Allocation

Extend:

```text
phalcom-core/tests/artifact_allocation.rs
```

Keep the existing compiler-image exact ledger as a baseline.

Add a full-executable mode measuring:

```text
compiler image stages
+ executable decode
+ executable proof
+ executable publication
```

Required:

```text
exact N
independent allocator equality
N - 1 typed failure
child rollback
clean retry
```

VM heap materialization allocations should be measured separately from immutable artifact-load N unless P3 explicitly expands the artifact transaction boundary to include VM construction. Do not silently mix them.

## 17.6 Runtime source-denial/parity

Extend the existing child-process harness in:

```text
phalcom-core/tests/core/modules/canonical_artifact.rs
```

It already owns source denial and fresh-process artifact compiler checks.

Add Process A / Process B style cases:

### Process A

```text
source authority enabled
build full universe.telosc
record digest
```

### Process B

Before artifact load:

```text
deny direct root source
deny UniverseSourceProvider::source_text
deny canonical source bootstrap constructors
reset Universe compiler/bootstrap counters
```

Then:

```text
load full artifact
compile novel user code from artifact
construct artifact-backed VM
execute eager Universe behavior
import one deferred Universe module
execute user code
assert all Universe compiler counters == 0
```

Do not merely deny the top-level constructor. Keep the direct source-text guard active.

## 17.7 Two-VM isolation

One test:

```rust
let artifact = load_full_artifact(...)?;

let vm1 = VM::try_new_with_universe_artifact(artifact.clone())?;
let vm2 = VM::try_new_with_universe_artifact(artifact.clone())?;
```

Prove:

- different module `ObjRef`s;
- different source-declared class `ObjRef`s where freshness is required;
- same canonical semantic identity observations;
- per-VM globals are independent;
- caches start empty independently;
- within each VM, one repeated constant recipe shares exact object identity;
- artifact `Arc` pointer identity is shared.

## 17.8 Deferred module once-only behavior

Select a canonical module that is genuinely in `bootstrap.deferred_modules`.

Instrument only the test harness or existing module state.

Prove:

```text
first import → initializer executes once
second import → state reused, no re-execution
source denial remains active
```

Do not force a normally eager module to be deferred just for the test unless a dedicated synthetic fixture owns that behavior.

---

# 18. Verification execution budget

## 18.1 BUILD MODE

Run only discriminating checks after coherent slices.

### Validator slice

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib artifact::validate -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib
git diff --check
```

If the exact module test filter differs, list tests first and use a filter that selects non-zero tests.

### Artifact codec/container slice

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact --test executable_image_codec_test
```

### Materializer compile seam

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib
```

Do not run full runtime integration while materialization/bootstrap is intentionally incomplete.

### Bootstrap tracer seam

Run one focused real test only after `VM::try_new_with_universe_artifact` can complete eager initialization.

## 18.2 STABILIZE MODE

Required after the whole source-free path exists:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact --test executable_image_codec_test
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib artifact::validate -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture --test-threads=1
```

Then focused core module selectors covering:

```text
full executable artifact build/load
source-denied artifact VM bootstrap
deferred canonical import
two-VM isolation
source/artifact parity
```

Use exact test names once implemented.

## 18.3 CERTIFY MODE

Only after focused stabilization passes:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules::executable_serialization -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules::canonical_artifact -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets
cargo fmt --all -- --check
git diff --check
```

Run broader workspace tests only if the P3 acceptance record or user explicitly requires them.

Do not automatically run workspace Clippy.

---

# 19. Ordered implementation tasks

The following decomposition is the recommended numbered P3 task structure.

---

## P3.T0 — Lifecycle takeover and baseline pin

### Purpose

Make ARTF001.C1.P3 the active owner without losing PERF003 evidence.

### Preconditions

- branch still at/descends from verified `aad6a88db`;
- P2.R4 acceptance state is understood.

### Modify

```text
docs/implementation/ARTF001-artifact-and-universe-consolidation/C1/CHECKPOINT.md
new ARTF001.C1.P3 numbered plan file
```

### Required implementation shape

Record:

- retired active P8/P9 identities;
- inherited executable DTO/capture/validator state;
- selected schema-preserving ordinary-method provenance contract;
- source-free runtime acceptance objective;
- focused test authorization for P3.

### Tests

None for prose-only bookkeeping.

### Acceptance

One active plan owner and no contradictory “P8 is active” instruction in the continuation path.

---

## P3.T1 — Finish executable proof and mint authority

### Purpose

Close the only safety-critical blocker before runtime consumption.

### Read

```text
artifact/validate.rs
vm/dispatch.rs Method / VariantMethod / Class
bytecode.rs
method/object.rs
heap/closure.rs
capture.rs
canonicalize.rs
P2 stable identity conversion
```

### Modify

```text
phalcom-core/src/artifact/validate.rs
```

### Create

```text
phalcom-core/src/artifact/validate/tests.rs
```

### Required implementation shape

Implement Sections 6–7 of this document exactly.

### Forbidden

- selector-only proof;
- class display-name authority;
- source span proof;
- treating `Nested` prototype owner as canonical callable;
- trusting method `holder` from recipe as canonical owner;
- accepting Unknown at install;
- widening join to a candidate set;
- executing bytecode to prove validity.

### Gate G1

Good captured canonical image validates and mutated provenance cases fail.

---

## P3.T2 — Integrate executable section into full build/load transaction

### Purpose

Turn proven executable bytes into an immutable loaded artifact capability.

### Modify

```text
canonical_artifact.rs
possibly container.rs
```

### Required interfaces

```rust
build_full_universe_telosc(...)
LoadedUniverseArtifact::has_portable_executable()
LoadedUniverseArtifact::validated_executable() // crate-private
```

Replace reserved field.

### Required loader behavior

Decode/validate before final artifact publication.

### Gate G2

- compiler-only artifact still compiler-usable;
- full artifact loads validated executable;
- corrupt executable fails before publication;
- capability/schema mismatch fails closed.

---

## P3.T3 — Full executable load allocation certification

### Purpose

Extend the exact P2 load transaction to the executable section.

### Modify

```text
canonical_artifact.rs allocation diagnostics
tests/artifact_allocation.rs
```

### Required

- exact executable DTO allocation;
- retained proof-table allocation;
- core validation scratch decision;
- N/N-1/retry;
- failpoint before final publication.

### Gate G3

Independent allocator equals ledger for full immutable artifact load.

---

## P3.T4 — VM-local relocation/materialization

### Purpose

Reconstruct executable recipes as fresh runtime values without executing initializers yet.

### Create

```text
artifact/materialize.rs
```

### Modify as needed

```text
artifact/mod.rs
chunk/callable helpers only if necessary
vm/mod.rs for relocation ownership if required
```

### Required

- constant memoization;
- instruction conversion;
- semantic-pool reconstruction;
- source map IDs;
- closure templates empty captures;
- method objects;
- primordial class resolution;
- source-class layout plans;
- GC rooting;
- no build-VM handles.

### Gate G4

Focused synthetic materializer tests plus core library check.

---

## P3.T5 — Artifact-native floor and canonical binding preparation

### Purpose

Remove source-index dependency from native/bootstrap setup.

### Modify

```text
native/install.rs
vm/bootstrap.rs
modules/builtin_materialize.rs if actual owner
```

### Required

Factor common primitive installation around stable owner/origin resolution.

### Forbidden

Duplicating primitive registry iteration.

### Gate G5

Artifact native floor reaches same primordial/native identities with source access denied.

---

## P3.T6 — First complete artifact-backed VM bootstrap

### Purpose

Create a fully usable fresh VM from one loaded full artifact.

### Modify

```text
vm/bootstrap.rs
error.rs
artifact/materialize.rs
```

### Required API

```rust
VM::try_new_with_universe_artifact(...)
```

### Required runtime sequence

Use Section 11.1.

### Gate G6

Fresh child process, source denied, eager bootstrap succeeds and representative Universe behavior matches source VM.

This is the first point where a source-free VM claim is allowed.

---

## P3.T7 — Deferred canonical import path

### Purpose

Eliminate canonical source fallback after initial VM construction.

### Modify after call-path verification

```text
modules/initialize.rs
modules/materialize.rs
interpret.rs
```

### Required

Use stored prototype for every deferred canonical module.

### Gate G7

One genuine deferred module initializes once under source denial and re-import does not rerun.

---

## P3.T8 — Explicit build/run API and CLI

### Purpose

Expose the completed full artifact workflow without changing default startup.

### Required

```text
build full artifact
load once
compile user program from same Arc
VM from same Arc
run
```

### Gate G8

CLI/API happy path plus incompatible/missing explicit artifact fail-closed behavior.

---

## P3.T9 — Stabilization and behavioral certification

### Purpose

Run the complete focused proof matrix.

### Required gates

```text
codec
validator hostile matrix
full load allocation
source/artifact runtime parity
source denial
deferred import
two-VM isolation
failed bootstrap retry
full build determinism
diagnostics
```

No task is complete because “cargo check passed.”

---

## P3.T10 — Lifecycle closure

### Purpose

Record only evidence actually executed.

### Produce

```text
ARTF001.C1.P3-walkthrough.md
ARTF001.C1.P3-handoff.md
CHECKPOINT.md update
```

### Completion truth

P3 may be `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` only after all required focused runtime gates pass.

`RELEASE_COMPLETE` requires any broader checkpoint release gates separately required by ARTF001.C1.

---

# 20. Verification gates

## G0 — Entry gate

Prove:

- P2 compiler image load/session authority is accepted;
- branch state is pinned;
- P3 is the active owner.

No executable code acceptance yet.

## G1 — Static executable authority

Run focused validator tests.

Expected:

```text
real canonical image → ValidatedExecutableImage
hostile identity/CFG/install mutations → typed rejection
```

No VM execution.

## G2 — Full container/load authority

Expected:

```text
compiler-only → loaded compiler authority, no executable
full artifact → loaded compiler + validated executable authority
bad executable → no LoadedUniverseArtifact publication
```

## G3 — Immutable-load allocation

Expected:

```text
N pass
N-1 typed fail
no partial parent charge
retry pass
independent allocator == ledger
```

## G4 — Materialization

Expected:

```text
all portable recipes reconstruct
no source
no execution
no GC dangling refs
within-VM recipe sharing exact
```

## G5 — Native floor

Expected:

```text
primordial/native mapping exact
no source index
no shadow classes
```

## G6 — Eager source-free VM

Expected:

```text
fresh process
source denied before load
full artifact load
VM bootstrap
eager initializers execute
representative behavior succeeds
Universe compile/analyze/lower/emit counters zero
```

## G7 — Deferred import

Expected:

```text
deferred module imports and executes from persisted prototype
second import does not rerun
zero source accesses
```

## G8 — One-artifact compiler/runtime workflow

Expected:

```text
same Arc pointer identity
compile novel user code
fresh artifact VM
execute
```

## G9 — Two-VM isolation

Expected:

```text
same immutable artifact
fresh mutable runtime state
within-VM canonical sharing
cross-VM no mutable sharing
```

## G10 — Failure recovery

Expected:

```text
corrupt load → no artifact
bootstrap failpoint → VM discarded
same artifact retry → clean success
```

## G11 — Determinism

Expected:

```text
fresh independent full build A bytes == build B bytes
same digest
different host filesystem root / allocation history does not perturb bytes
```

## G12 — Focused final acceptance

All CV-01…CV-24 applicable obligations have an executed PASS or a documented explicit non-applicability ruling.

---

# 21. STOP / CONSULT triggers

Global doctrine triggers apply. P3-specific mandatory triggers:

1. A legal current canonical executable contains an object/semantic product with no portable recipe.
2. Ordinary method identity can only be proven by source spans, prototype display names, or selector uniqueness outside canonical owner identity.
3. A required `Class` site cannot be tied exactly to one P7 `SourceDeclared` class row under the existing schema.
4. A required source class is allocated from a repeatedly invoked callable/nested prototype and accepting it requires weakening the one-shot class rule.
5. One method recipe is legally installed under conflicting canonical bindings.
6. Materialization needs to re-run semantic analysis or source lowering to reconstruct `ExecutableSemanticPool`.
7. Correct relocation appears to require changing `Bytecode::Class` from fresh allocation to shared preallocation.
8. A primordial class must be duplicated to satisfy a recipe.
9. Native source-owned primitive identity cannot be reconstructed from existing validated executable/P2 authority.
10. GC safety would require a new rooting/lifetime policy.
11. Full executable validation cannot be made bounded without changing the wire schema or load transaction architecture.
12. A full executable needs a new stable identity not present in P2 or the executable DTO.
13. A source fallback is proposed to make a runtime test pass.
14. An existing semantic/runtime parity test appears wrong rather than the artifact path.
15. P3 needs a material change to the P2 semantic/query ownership model.
16. Full artifact publication cannot remain atomic with the existing container writer without a format-level change.

A trigger cannot be self-waived by the implementer.

---

# 22. Debugging budget

## Mechanical failures

Up to three coherent cycles:

```text
inspect
identify concrete local cause
make one mechanical correction
rerun smallest discriminator
```

Examples:

- import/signature drift;
- private/public visibility mismatch;
- moved CLI file;
- borrow checker around relocation maps;
- wrong test filter.

## Semantic/runtime failure

Before editing:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

One serious corrective attempt for the same causal issue.

If it persists, STOP AND CONSULT.

## Architectural failure

Zero speculative fixes.

---

# 23. Baseline/unrelated failure policy

Classify:

```text
A — definitely caused by P3: fix
B — probably caused by P3: investigate/fix
C — unclear: one bounded classification pass, then record/defer if nonblocking
D — clearly unrelated baseline: record and continue
```

Do not repair unrelated language/compiler failures while implementing executable Universe support.

Do not weaken tests or artifact invariants to make a broad suite green.

---

# 24. Expected implementer workflow

Recommended work slices:

```text
Slice 1
  lifecycle record
  provenance dataflow
  method/variant proof
  validator unit tests
  G1

Slice 2
  full builder
  full loader
  authority publication
  container tests
  G2

Slice 3
  executable load accounting
  G3

Slice 4
  materializer core
  constants/prototypes/semantic pools
  rooting
  G4

Slice 5
  native floor
  source class/static binding preparation
  G5

Slice 6
  artifact VM constructor
  eager bootstrap
  first source-denied runtime tracer
  G6

Slice 7
  deferred imports
  G7

Slice 8
  public API / CLI
  G8

Slice 9
  isolation/retry/determinism/diagnostics
  G9-G11

Slice 10
  focused final acceptance
  lifecycle records
  G12
```

Do not cram all slices into one implementation turn if doing so prevents causal testing or clean checkpoint updates.

---

# 25. Code examples: intended usage after P3

## 25.1 Build

```rust
let canonical = canonical_universe_program()?;
let view = canonical.compiler_image_view();

let built = build_full_universe_telosc(
    &view,
    &ValidationLimits::default(),
)?;

write_universe_telosc(
    Path::new("universe.telosc"),
    &built,
)?;
```

Exact view construction may differ; use live API.

## 25.2 Load for compiler tooling

```rust
let artifact = load_universe_telosc(
    Path::new("universe.telosc"),
    &ValidationLimits::default(),
)?;

let program = artifact.compile_user_entry(entry)?;
```

Works for compiler-only or full artifacts.

## 25.3 Load for execution

```rust
let artifact = load_universe_telosc(
    Path::new("universe.telosc"),
    &ValidationLimits::default(),
)?;

let program =
    artifact.compile_user_entry_with_options(entry, &options)?;

let mut vm =
    VM::try_new_with_universe_artifact(artifact.clone())?;

vm.run_compiled(&program)?;
```

For a compiler-only artifact:

```rust
VM::try_new_with_universe_artifact(artifact)
```

must return a typed error, not source fallback.

---

# 26. Architecture self-review

## One owner for every important fact?

Yes:

- semantic identity → P2;
- portable representation → artifact crate;
- executable proof → core validator;
- runtime object identity → one VM materializer/runtime;
- initialization state → module registry;
- native implementation → compiled-in native registry.

## Any duplicate semantic reasoning planned?

No. Materialization reconstructs persisted runtime recipes; it does not infer types/callables again.

## Identity explicit?

Yes:

- method object = canonical `ConstantId` under one-object-per-recipe materialization;
- fresh class = exact allocation site + canonical stable class row;
- canonical callable = P7 stable callable identity;
- VM runtime handles are never serialized.

## Lifetime/rooting explicit?

Yes. Immutable artifact may be shared; runtime objects are per VM; temporary heap construction uses existing temp roots.

## Failure/fallback explicit?

Yes. Explicit artifact mode fails closed and never sources the Universe.

## Test breadth controlled?

Yes. BUILD uses focused validator/codec/compile checks; runtime suites start only when the corresponding seam exists; broad workspace certification is not reflexive.

## Remaining architectural unknowns?

Only verify-first repository facts:

1. exact stable declaration mapping used to bind `Bytecode::Class` sites;
2. whether real canonical method recipes are installed at multiple identical sites;
3. exact current runtime owner for deferred canonical import fallback;
4. exact reverse constructors needed for every `PortableSemanticPoolV1` family;
5. whether core validation scratch must be included in exact full-load allocator equality or can use an explicitly separate bounded scratch contract.

If any of these invalidate the fixed model rather than merely changing mechanics, escalate.

---

# 27. Completion truth table

Do not conflate:

| State | Meaning |
|---|---|
| Source written | code exists |
| `cargo check` passes | Rust type/borrow integration compiles |
| Tests added | harness exists |
| G1 passes | executable proof behavior verified |
| G2/G3 pass | full immutable load authority verified |
| G4/G5 pass | relocation/native setup verified |
| G6 passes | first valid source-free eager VM claim |
| G7 passes | later canonical imports source-free |
| G9 passes | multi-VM mutable isolation proven |
| G11 passes | deterministic full bytes proven |
| G12 passes | focused P3 acceptance complete |
| Workspace/release certified | only after separately required broad gates |

A successful compiler check alone must never promote P3 to implemented/verified.

---

# 28. Immediate implementation starting point

The first code patch should be narrowly scoped to:

```text
phalcom-core/src/artifact/validate.rs
phalcom-core/src/artifact/validate/tests.rs
```

and should **not** touch runtime materialization yet.

Concrete order:

1. refactor validation context away from `LoadedUniverseArtifact`;
2. add provenance types;
3. reuse existing stack/CFG worklist infrastructure where safe, but keep identity state distinct;
4. implement exact transfer rules;
5. map `Class` sites to stable class rows;
6. prove `Method`;
7. prove `VariantMethod`;
8. group installation bindings by method recipe;
9. check class site uniqueness/cycles;
10. change successful validator return to `ValidatedExecutableImage`;
11. add focused hostile tests;
12. run only G1 BUILD-mode commands;
13. update checkpoint with the durable proof boundary.

Only after that seam is green should the implementer begin container/load integration.

---

# 29. Final implementation handoff

**Current revision:** `aad6a88db904de04059ee9027240ab168b136c5c`.

**Do not redesign:**

- executable wire schema unless a legal current product is unrepresentable;
- P2 stable identity;
- method/runtime semantics;
- `Bytecode::Class` freshness;
- native registry ownership;
- module once-only state;
- GC policy;
- default source-backed startup.

**First implementation target:** ordinary method/variant installation proof and actual validated-authority minting.

**First expected meaningful test result:**

```text
real current canonical executable image accepted
+
hostile method/class/variant provenance mutations rejected
```

**First expected architectural checkpoint update:** P3 validator authority is complete and downstream materialization may consume only `ValidatedExecutableImage`.

Everything after that should proceed in the ordered task graph above.

