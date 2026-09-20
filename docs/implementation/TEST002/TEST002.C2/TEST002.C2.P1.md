# TEST002.C2.P1 — Runtime Correctness Remediation

```yaml
---
id: TEST002.C2.P1
category: TEST
program: TEST002
checkpoint: TEST002.C2
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - TEST002.C1.P2
follows: TEST002.C1.P2
supersedes: null
---
```

## 0. Executor contract

This is a Luna-oriented patch-grade implementation plan. It is intended to let a capable implementation agent perform the mechanical work continuously without re-deciding the architecture.

The implementer:

- may adapt mechanical details to the live repository;
- must preserve the architecture, ownership boundaries, and invariants in this plan;
- must not improvise material architecture;
- must re-read the named live files and symbols immediately before editing;
- must treat current repository state as authoritative for mechanics, while accepted specifications and the fixed decisions in this plan remain authoritative for intended behavior;
- must use the smallest discriminating tests during BUILD mode;
- must classify and defer unrelated failures instead of repairing them opportunistically;
- must STOP AND CONSULT whenever a global or task-local escalation trigger fires;
- must keep the shared `TEST002.C2` checkpoint record current at durable milestones;
- must produce the required walkthrough and handoff before declaring the plan administratively complete.

Testing is evidence gathering, not a ritual. Do not run a broad suite merely because an edit occurred.

This plan does **not** authorize commits or pushes by itself. If commit/push authorization exists separately, use the coherent commit groups in §26 and follow the repository's commit discipline. Do not create tiny edit-by-edit commits, do not stage unrelated changes, and do not push known-broken states.

---

## 1. Goal

Eliminate the retained confirmed runtime-correctness failures from the 2026-09-07 runtime audit by repairing the owning compiler/runtime boundaries: bounded executable operands, lexical `super` identity, GC authority edges, public native admission and native activation depth, raw-run teardown, native diagnostic provenance, exact numeric division/remainder, Resource identity/lifecycle transport, and short-write-safe buffered output.

---

## 2. Checkpoint acceptance objective

`TEST002.C2` is a bounded runtime-correctness checkpoint.

`TEST002.C2.P1` is intended to provide **full checkpoint implementation closure** if the fixed architecture can be implemented without an escalation that requires a follow-up plan.

The checkpoint is acceptable when all confirmed retained defects below are corrected at their canonical owner and proven with focused evidence:

| Audit | Acceptance claim |
|---|---|
| `AUD-RUNTIME-001` | No bounded executable operand can wrap, saturate, or become build-mode-dependent before publication. |
| `AUD-RUNTIME-002` | Every lexical `super` uses stable defining-behavior identity rather than mutable name lookup. |
| `AUD-RUNTIME-003` | Every strong runtime authority/layout/lexical identity edge held by a live runtime object participates in GC reachability. |
| `AUD-RUNTIME-004` | Invalid public native invocation is rejected before execution-window mutation, and native Rust activations have their own explicit depth budget. |
| `AUD-RUNTIME-005 §4` | Starting a fresh raw module run closes stale open captures before stack/frame storage is reused. |
| `AUD-RUNTIME-005 §6` | Active native identity is scoped, while an escaping error retains the native origin that actually produced it. |
| `AUD-RUNTIME-006` | `~/` and Float-domain `%` implement their specified mathematical domains without lossy convenience promotion. |
| `AUD-RUNTIME-008` | Every valid managed Resource handle encoding is exact/injective, stale generations cannot revive, and Resource lifecycle uses the selected hidden-slot Resource-v2 model. |
| `AUD-RUNTIME-007` | `BufferedWriter` tolerates legal short writes without loss, rejects zero progress, preserves pending suffixes on failure, propagates layered flush, and rejects overlapping mutation deterministically. |

The following findings are **not** checkpoint blockers and remain explicit follow-ups:

- full executable-chunk verification from `AUD-RUNTIME-005 §1`;
- unmeasured rest/native metadata optimization from `AUD-RUNTIME-005 §2`;
- sparse per-instruction metadata/cache layout from `AUD-RUNTIME-005 §3`;
- separate unexecuted numeric/string/performance leads not promoted by the retained audits.

A completed P1 may truthfully end as:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

`RELEASE_COMPLETE` is not required unless a later explicit certification gate runs successfully.

`TEST002.C1` currently has independent certification work recorded in its own checkpoint. C2 must not silently mark C1 complete or absorb unrelated C1 certification failures. C2 is grounded on the implementation state established through `TEST002.C1.P2`.

---

## 3. Repository grounding

### 3.1 Prepared-against state

Planning was grounded against the remote repository:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   6743b02725a11923cb1a4efdab9c39f87bb8f8c6
subject:    fix(semantic): repair generic subtype projection
```

The planning environment exposed the remote repository state, not the implementer's local working tree.

At execution start, run:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Record the actual starting revision in `TEST002.C2/CHECKPOINT.md`.

If the local tree contains unrelated modifications, preserve them. Do not reset, clean, restore, stash, or stage them as part of this plan.

### 3.2 Canonical implementation-record location

The canonical TEST002 family is:

```text
docs/implementation/test/TEST002-baseline-stabilization/
```

Current remote state contains:

```text
PROGRAM.md
C1-semantic-core-baseline-elimination/
```

No canonical `TEST002.C2` record currently exists.

Do **not** create C2 under the legacy/duplicate:

```text
docs/implementation/TEST002/
```

Create the canonical C2 family under:

```text
docs/implementation/test/TEST002-baseline-stabilization/
  C2-runtime-correctness-remediation/
    CHECKPOINT.md
    TEST002.C2.P1-runtime-correctness-remediation.md
```

At plan completion also create:

```text
TEST002.C2.P1-runtime-correctness-remediation-walkthrough.md
TEST002.C2.P1-runtime-correctness-remediation-handoff.md
```

### 3.3 Verified repository facts

The following facts were verified while preparing this plan.

#### Compiler representation

- `phalcom-core/src/chunk.rs::Chunk::add_constant` pushes first and narrows `(len - 1) as u16`.
- `ExecutableSemanticPool` in the same file already uses checked `u16::try_from(...)` before publication and provides the preferred local architectural precedent.
- `phalcom-core/src/compiler/lib/mod.rs::emit_global_reference` still contains both unchecked `binding.0 as u16` and `u16::try_from(index).unwrap_or(u16::MAX)`.
- `class_decl.rs`, `loops.rs`, `patterns.rs`, `expr.rs`, and `product_opt.rs` still contain executable `usize -> u16/u8` narrowing.
- `jumps.rs` narrows instruction positions to `i32` before computing relative offsets.

#### Lexical `super`

- `Bytecode::SuperSend` is currently `SuperSend(u8, u16, u16)`, with the third operand a defining-class-name constant.
- `Bytecode::SuperSendPack` carries `defining_class: u16`.
- `Compiler::compile_super_send` synthesizes the instance class name or `"<Name>.class"` and emits it as a constant.
- both exact and packed VM handlers reconstruct `ClassKey { module, name }` at execution time and consult mutable `VM::classes`.
- `Bytecode::Method` installation already knows the defining instance class/metaclass and stamps holder/access metadata and `ClosureObject.lexical_class`.

#### GC authority edges

- `ClosureObject` carries `lexical_class` and `foreign_receiver_guard`.
- `MethodObject` carries `access_owner`.
- `CallFrame` carries `foreign_receiver_guard`.
- the retained audit probe for closure lexical authority remains ignored under `tests/core/audit_probes/runtime_invariants.rs`.

#### Native entry

- `VM::send_dynamic` currently pushes receiver/arguments before `invocation_layout_for_selector(...)`.
- `native_reentry_depth` and `MAX_NATIVE_REENTRY = 32` intentionally measure recursive interpreter driving and are used by fiber-floor/yield semantics.
- `NativeMethodContext` currently contains access authority only.
- both legacy and shape-aware primitive activation paths still write singleton `native_selector` / `native_class`.
- the retained native-only recursion audit probe currently succeeds at recursion depth 40, demonstrating that the existing re-entry counter is not an activation counter.

#### Execution teardown

- `VM::run_in_module` lives in `phalcom-core/src/interpret.rs` and begins with raw `frames.clear(); stack.clear()`.
- `VM::unwind_cell` already delegates to `unwind_to(0, 0)`.
- `VM::unwind_to` closes open upvalues before truncating frame and value-stack storage.

#### Diagnostic transport

- `FrameRecord` currently has `Normal` and `FiberBoundary`.
- `RuntimeError::Raise` can carry captured `Vec<FrameRecord>`.
- accepted traceback/error architecture captures frame records at the first `on` boundary before bulk unwind; records contain Symbols rather than GC handles.
- the renderer currently adds native identity from ambient singleton VM state.

#### Numeric execution

- `number.rs::promote_pair` converts mixed exact integers to `f64`.
- `number_floor_div` uses rounded Float arithmetic on mixed inputs.
- `number_mod` uses host `%` for the Float branch.
- `float.rs::float_to_rational(f64)` already decodes a finite binary64 exactly into `BigInt` numerator/denominator and is the canonical implementation component to reuse.
- numeric integration tests already live in `tests/core/language/numeric_values.rs` and `numeric_diagnostics.rs`.

#### Resources

- `ResourceHandle` is `{ index: u32, generation: u32 }`.
- `ResourceHandle::pack` currently maps the full 64-bit concatenation numerically to `f64`; `unpack` casts it back.
- `ResourceTable::open` uses unchecked `entries.len() as u32` and `wrapping_add(1)`.
- `Resource` already has `field_count = 1` at bootstrap, but bootstrap also publishes `_handle` in `field_slots`, making the infrastructure slot source-addressable.
- `primitive/resource.rs` still exposes `Resource.register`, reads slot 0 as a user-stored number, and has no `ensureOpen` primitive.
- current resource unit tests contain the false endpoint proof using `u32::MAX/u32::MAX`.

#### Streams

- `core/universe/src/collections/bytes.ph` assigns `_handle = Resource.register(...)` in all managed Resource subclasses.
- `BufferedWriter.flush` performs one inner write, sets `_len = 0` before settlement, ignores the accepted count, and does not call `inner.flush`.
- existing stream fixtures already cover basic buffering, `finish`, boundary writes, dirty close, and synchronous flush rejection.
- the language-corpus test target supports focused `streams` execution.

### 3.4 Drift rule

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

## 4. Required reads before implementation

Read these before editing, in this order.

### Lifecycle and execution doctrine

```text
AGENTS.md
docs/implementation/README.md
docs/workflow/implementation-record-lifecycle.md
docs/workflow/luna-execution-and-escalation-protocol.md
docs/workflow/commit-and-push-discipline.md   # only if commits/pushes are authorized
```

If the workflow filenames have mechanically moved, locate their current canonical equivalents; do not use old duplicated project-source copies as state authority.

### TEST002 state

```text
docs/implementation/test/TEST002-baseline-stabilization/PROGRAM.md
docs/implementation/test/TEST002-baseline-stabilization/C1-semantic-core-baseline-elimination/CHECKPOINT.md
docs/implementation/test/TEST002-baseline-stabilization/C1-semantic-core-baseline-elimination/TEST002.C1.P2-walkthrough.md
docs/implementation/test/TEST002-baseline-stabilization/C1-semantic-core-baseline-elimination/TEST002.C1.P2-handoff.md
```

### Retained runtime audits

```text
docs/audit/runtime-2026-09-07/AUD-RUNTIME-001-unchecked-operand-narrowing.md
docs/audit/runtime-2026-09-07/AUD-RUNTIME-002-super-dispatch-loses-lexical-identity.md
docs/audit/runtime-2026-09-07/AUD-RUNTIME-003-gc-misses-authority-edges.md
docs/audit/runtime-2026-09-07/AUD-RUNTIME-004-native-entry-validation.md
docs/audit/runtime-2026-09-07/AUD-RUNTIME-005-grouped-contract-and-performance-findings.md
docs/audit/runtime-2026-09-07/AUD-RUNTIME-006-numeric-domain-contracts.md
docs/audit/runtime-2026-09-07/AUD-RUNTIME-007-buffered-writer-flush-contract.md
docs/audit/runtime-2026-09-07/AUD-RUNTIME-008-resource-handle-lossy-transport.md
```

### Normative / accepted behavioral authority

```text
docs/specs/callables/dispatch.md
docs/specs/vm/bytecodes.md
docs/specs/vm/vm-execution-model.md
docs/spec/library/numbers/numeric-tower.md
docs/spec/library/numbers/float-protocol.md
docs/spec/current/stdlib/stream-protocol.md
docs/decisions/accepted/0057-resources-are-disposable-handles-not-finalized.md
docs/decisions/accepted/0059-bounded-call-depth-and-native-reentrancy.md
docs/decisions/accepted/0062-errors-carry-structure-and-cheap-origin.md
docs/adr/accepted/0013-block-closure-upvalues.md
```

If decision numbering/paths have mechanically moved, resolve the current accepted record by title.

### Selected implementation architecture for Resources / Streams

```text
docs/implementation/NATV001-host-surfaces/C3-resource-host-surface/resource-host-surface-spec.md
docs/implementation/NATV001-host-surfaces/C4-streams-host-surface/streams-host-surface-spec.md
```

These documents are currently proposal-shaped implementation records, not automatically normative. For this plan, the architecture explicitly selected in §§7, 8, T9–T11 is fixed. Do not independently implement every NATV001 proposal item outside the bounded C2 scope.

### Test conventions

```text
phalcom-core/tests/README.md
phalcom-core/tests/core/audit_probes/runtime_invariants.rs
phalcom-core/tests/core/execution/depth_limits.rs
phalcom-core/tests/core/execution/send_arity.rs
phalcom-core/tests/core/repl/session.rs
phalcom-core/tests/core/memory/gc.rs
phalcom-core/tests/core/observability/super_disassembly.rs
phalcom-core/tests/core/observability/traceback.rs
phalcom-core/tests/core/language/numeric_values.rs
phalcom-core/tests/core/language/numeric_diagnostics.rs
phalcom-core/tests/fixtures/language/streams/
```

Avoid broad repository exploration unless evidence from these reads contradicts the plan.

---

## 5. Normative authority

Keep four categories separate.

### 5.1 Normative behavior

The following behaviors are fixed by accepted/current specification:

- lexical `super` begins above the Method's lexically defining behavior while keeping the dynamic receiver;
- binding/exact-invoking a Method on another receiver does not move its lexical `super` anchor;
- `~/` computes the floor of exact represented numeric values and returns exact `Int`;
- Float-domain `%` uses floor remainder, one final correctly-rounded binary64 result, and divisor-signed zero/remainder behavior;
- Writer `write` may legally report a short positive count;
- Writer `flush` propagates durability/acceptance to the next layer;
- Resource close is synchronous and idempotent;
- open captures must be closed before their backing stack storage is reclaimed/reused;
- traceback frame records are GC-immune Symbol-based records and caught-origin capture occurs at the first `on` boundary before bulk unwind;
- `native_reentry_depth` protects recursive Rust-stack interpreter driving and fiber restrictions.

### 5.2 Current implementation reality

Current source is authoritative for:

- exact file/symbol locations;
- current private APIs and representation mechanics;
- current test registration;
- current native descriptor/census mechanics;
- mechanical call fanout.

Current implementation is **not** authority for behavior proven divergent by the retained audits.

### 5.3 Plan-ratified implementation architecture

This plan fixes the following architectural choices:

- checked producer-side construction for bounded executable operands;
- separate closure-carried `super_anchor`;
- complete tracing of strong authority/layout edges;
- a separate native activation budget;
- public invocation shape validation before execution-window mutation;
- a lightweight error-associated native-origin transport using Symbol identity, integrated into `FrameRecord`;
- Resource-v2's exact 21-bit-index / 32-bit-generation managed handle domain;
- hidden Resource slot 0 owned by runtime primitives;
- transactional managed Resource attachment plus `ensureOpen`;
- non-wrapping Resource generation lifecycle;
- Streams-v2's BufferedWriter drain/gate/layered-flush architecture.

### 5.4 Known specification conflict

`docs/specs/callables/dispatch.md` describes stable Method-definition-owned lexical `super` identity.

Current `docs/specs/vm/bytecodes.md` still describes a `definingClass` operand resolved at runtime.

For C2, callable dispatch semantics win. T3 must change VM bytecode documentation to the stable-anchor design.

If a newer accepted specification reverses this rule at execution time, STOP AND CONSULT.

---

## 6. Takeover state

| Concept | Current symbol/path | Owner | Invariant inherited by C2 |
|---|---|---|---|
| Constant pool | `Chunk::constants`, `Chunk::add_constant` | compiler/chunk | bytecode operand is `u16`; compact representation remains |
| Checked semantic pools | `ExecutableSemanticPool::add_*` | compiler/chunk | validate before push/publication |
| Compiler errors | `compiler/lib/error.rs` | compiler | source failures are structured and span-aware |
| Local allocation | `Compiler::add_local` | compiler/scope | one canonical binding-local allocation seam |
| Jump patching | `compiler/lib/jumps.rs` | compiler | relative VM operands are `i32` |
| Lexical access | `ClosureObject.lexical_class` | runtime executable metadata | access privilege survives nested closures |
| Method install identity | `MethodObject::{holder,access_owner}` | method/runtime | holder and access owner are not interchangeable with receiver class |
| Foreign layout authority | `ForeignReceiverGuard.layout_owner` | runtime frame/closure | transplanted Method layout guard is execution authority |
| Native re-entry | `VM::native_reentry_depth` | VM control | protects recursive `run_until`, also fiber-floor/yield behavior |
| Native access context | `VM::native_method_contexts` | VM native activation | native bodies need their own lexical access authority |
| Canonical unwind | `VM::unwind_to` | VM control | closes upvalues before frame/stack truncate |
| Caught trace transport | `RuntimeError::Raise.traceback` / `FrameRecord` | runtime diagnostics | records use Symbols, not ObjRefs |
| Exact Float decoder | `primitive::float::float_to_rational` | numeric runtime | finite Float represented value can be recovered exactly |
| Resource table | `ResourceTable` | VM resource runtime | table contains no Phalcom heap roots |
| Resource slot | `Resource.field_count = 1` | bootstrap/runtime | inherited slot 0 already exists |
| Buffered pending state | `BufferedWriter._buf/_len` | stream wrapper | accepted-but-not-handed-off bytes belong to wrapper |
| Audit probes | `tests/core/audit_probes/runtime_invariants.rs` | TEST evidence | retained ignored probes are migration inputs, not permanent test architecture |

Do not redesign these unrelated established mechanisms.

---

## 7. Architecture

### 7.1 Overall correction shape

```text
source/compiler facts
    │
    ├─ checked bounded producer ──────────────► trustworthy bytecode operands
    │
    └─ Method-definition identity ────────────► closure.super_anchor
                                                  │
                                                  ├─ super dispatch
                                                  └─ GC strong edge

public host/native call
    │
    ├─ validate selector/argument shape
    ├─ materialize execution window
    └─ native activation
          ├─ native_activation_depth
          ├─ NativeMethodContext
          └─ NativeOrigin transport on error
                 └─ FrameRecord::Native at capture/render

fresh raw execution
    └─ unwind_to(0, 0) before frame/stack reuse

Number
    ├─ Float-result ordinary arithmetic may promote
    ├─ ~/ uses exact represented values
    └─ Float % converts into Float domain, decodes exactly, rounds once

Resource instance
    └─ hidden inherited slot 0
          └─ exact packed handle <= 2^53 - 1
                └─ ResourceTable state machine

BufferedWriter
    ├─ local accepted bytes
    ├─ one unresolved mutation gate
    ├─ short-write-safe drain
    └─ layered flush -> inner.flush
```

### 7.2 Key identity separation

The runtime must preserve these distinct identities:

```text
receiver dynamic class
Method installation holder
lexical access owner
lexical super anchor
foreign layout owner
```

They often coincide for ordinary source methods. That does not make them one concept.

### 7.3 `super` architecture

Add a stable behavior identity to closure-backed executable code:

```rust
// STRUCTURAL — exact private type/name may adapt.
ClosureObject {
    ...
    lexical_class: Option<ClassId>, // access authority
    super_anchor: Option<ClassId>,  // lexical lookup origin
    foreign_receiver_guard: Option<ForeignReceiverGuard>,
}
```

Rules:

- ordinary instance Method installation sets `super_anchor = Some(defining_instance_class)`;
- class-side Method installation sets `super_anchor = Some(defining_metaclass)`;
- nested block materialization inherits the executing closure's anchor;
- rebinding/transplanting a Method does not move the anchor;
- each super send reads the **current superclass of the stable anchor** at execution time;
- exact and packed super use the same anchor source;
- source class names are not used to reconstruct identity at execution.

Target bytecode shape:

```rust
// STRUCTURAL
SuperSend(u8, u16) // arity, selector
SuperSendPack {
    base_name: u16,
}
```

No defining-class constant remains.

### 7.4 Native activation architecture

Keep:

```text
native_reentry_depth
```

for recursive guest-interpreter driving.

Add:

```text
native_activation_depth
```

for actual nested native Rust primitive invocations.

Initial ceiling:

```text
MAX_NATIVE_ACTIVATION_DEPTH = 32
```

may equal the re-entry ceiling, but it remains separately named and accounted.

Every legacy and shape-aware primitive invocation:

```text
check activation budget
increment activation depth
push NativeMethodContext
call Rust primitive
associate native origin with Err if needed
pop context
decrement activation depth
reconcile CallOutcome / stack window
```

The counter is released when the Rust primitive returns, including `EnteredFrame`; guest bytecode execution is not itself a native activation.

### 7.5 Native diagnostic provenance architecture

Ambient singleton `native_selector/native_class` cannot represent nested failure identity.

Extend active native context with Symbol-only diagnostic identity:

```rust
// STRUCTURAL
struct NativeFrameIdentity {
    class: Symbol,
    selector: Symbol,
}

struct NativeMethodContext {
    access_owner: Option<ClassId>,
    origin: RuntimeAccessOrigin,
    diagnostic: NativeFrameIdentity,
}
```

Add a lightweight error-associated transport. Preferred shape:

```rust
// STRUCTURAL: name may adapt, semantics may not.
PhError::NativeOrigin {
    source: Box<PhError>,
    class: Symbol,
    selector: Symbol,
}
```

Required behavior:

- first native boundary producing an error attaches its identity;
- an outer native boundary propagating an already-originated error must **not overwrite the inner origin**;
- if inner failure is caught/dropped and outer later creates a new error, the new error receives the outer origin;
- access authority is still popped immediately when the native Rust body returns; diagnostic provenance does not keep authority ambient;
- the wrapper carries Symbols only and therefore introduces no GC rooting problem;
- at first `on` capture, unwrap transport for ordinary error semantics and append/merge a `FrameRecord::Native { class, selector }`;
- at an uncaught root renderer, render the error-associated native origin, not VM-global ambient state;
- after capture into `RuntimeError::Raise.traceback`, native origin must not be duplicated by outer boundaries.

If live error architecture makes a transparent `PhError` transport variant materially incompatible with accepted TDR-0062 semantics, STOP AND CONSULT rather than substituting ambient "last native failure" state.

### 7.6 Numeric architecture

There is no universal numeric promotion relation.

```text
+ - * / and other Float-result operations
    may use Float-domain conversion per numeric spec

~/
    exact represented-value rational arithmetic

Float-domain %
    ordinary Int->Float conversion first
    then exact dyadic arithmetic
    then exactly one correctly-rounded binary64 result
```

Reuse `float_to_rational`. Do not create guest Rational values.

### 7.7 Resource-v2 architecture selected for C2

Managed Resource identity remains semantically opaque but may use a numeric physical carrier because the valid domain is explicitly bounded to exact binary64 integers:

```text
bits  0..31  generation
bits 32..52  table index
bits 53+     invalid
```

Constants:

```rust
const RESOURCE_INDEX_BITS: u32 = 21;
const RESOURCE_GENERATION_BITS: u32 = 32;
const RESOURCE_MAX_SLOTS: u32 = 1 << RESOURCE_INDEX_BITS;
const RESOURCE_MAX_INDEX: u32 = RESOURCE_MAX_SLOTS - 1;
const RESOURCE_MAX_HANDLE: u64 = (1u64 << 53) - 1;
```

The table lifecycle is:

```text
Vacant
Open(g)
Closed(g)
Retired(MAX)
```

Generation changes on reuse, never on close, and never wraps.

`Resource.field_count` stays `1`, but slot 0 ceases to have source-visible `_handle` authority. Resource primitives access it directly.

Managed acquisition becomes transactional instance attachment:

```phalcom
self.attach("BytesWriter") // exact internal selector spelling: verify live convention
```

rather than:

```phalcom
_handle = Resource.register("BytesWriter")
```

The internal physical unattached marker may be the allocator's private unread sentinel or `None`; it is not a source-visible value. `attach`, `close`, `isClosed`, and `ensureOpen` must normalize that state consistently.

External files/sockets/reactor adoption are out of scope.

### 7.8 BufferedWriter architecture selected for C2

State:

```text
_inner
_buf
_len
_busy
hidden Resource slot
```

An internal drain operation owns short-write handling:

```text
snapshot local pending bytes
accepted = 0

repeat:
    inner.write(remaining)
    validate settled count

    invalid count -> StreamProtocolError
    zero for nonempty remainder -> WriteZeroError
    positive short count -> advance accepted and continue

success:
    _len = 0

failure after accepted prefix:
    preserve exactly unaccepted suffix in _buf
    _len = suffix length
```

Public `flush` is:

```text
drain local buffer fully
then inner.flush
then fulfill None
```

Even `_len == 0` must still call `inner.flush`.

No internal `await`; use Future continuations.

At most one unresolved state-mutating wrapper operation owns `_busy`. Do not create an implicit Future queue.

---

## 8. Ownership boundaries

### 8.1 Owns

C2.P1 owns:

- runtime-safe encoding of existing bounded bytecode/compiler operands;
- lexical `super` runtime identity;
- GC tracing of runtime authority/layout identity;
- public host/native call admission atomicity;
- native activation accounting;
- raw top-level run reuse hygiene;
- native diagnostic provenance;
- the audited `~/` and Float `%` numeric divergences;
- managed Resource handle exactness and selected Resource-v2 managed lifecycle;
- managed stream migration to hidden Resource attachment;
- BufferedWriter short-write/layered-flush correctness.

### 8.2 Does not own

C2.P1 does not own:

- parser syntax redesign;
- semantic type-system changes;
- full bytecode verifier design;
- bytecode serialization/external bytecode loading;
- broad cache layout redesign;
- unmeasured performance optimization;
- new public Resource capability representation;
- filesystem descriptors/sockets/reactor adoption;
- GC finalizers;
- implicit stream-operation queueing/backpressure policy;
- public Reader/Writer marker types;
- public Rational/BigInt surface redesign;
- unrelated semantic baseline failures;
- C1 release certification.

### 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| Constant/linked/local representability | producer-side compiler allocation helper | bytecode emitters | VM alias detection |
| Field count/slot representability | class layout construction | accessors/VM | per-field late truncation |
| Jump offset representability | jump patch/emit helper | bytecode | `as i32` before arithmetic |
| Lexical super identity | `ClosureObject.super_anchor` | exact/pack super | `ClassKey` rebuilt from class name |
| Lexical access authority | `lexical_class` / `access_owner` | access checks | `super_anchor` reused as privilege |
| Foreign layout authority | `ForeignReceiverGuard.layout_owner` | field guard | receiver dynamic class |
| Strong lifetime | GC trace routines | collector | registry rooting workaround |
| Public invocation shape | selector-layout admission | host send | primitive-local `args.get` defenses |
| Native Rust recursion | `native_activation_depth` | primitive activation | `native_reentry_depth` |
| Recursive interpreter driving | `native_reentry_depth` | fiber/re-entry control | activation counter |
| Active native access | `NativeMethodContext` stack | access checks | ambient singleton |
| Escaping native origin | `PhError` native-origin transport / `FrameRecord::Native` | catch/renderer | VM-global "last native" |
| Exact finite Float value | `float_to_rational` | `~/`, `%`, conversion helpers | second IEEE decoder |
| Managed Resource identity | hidden slot + `ResourceTable` | Resource primitives | source `_handle` field |
| Resource generation | table slot state | stale-handle checks | caller-managed version |
| Local buffered pending bytes | `BufferedWriter._buf/_len` | `pending`, drain, close | inner writer state guess |
| Wrapper mutation ownership | `_busy` | write/flush/finish/close | implicit queue |

---

## 9. Global invariants

### `INV-01 — Executable representability`

Every bounded executable operand is validated in a sufficiently wide domain before publication. No wrap, saturation, or debug/release semantic difference is allowed.

### `INV-02 — Lexical super identity`

Every executable `super` is anchored to the stable defining behavior of its Method, independent of later source-name rebinding.

### `INV-03 — Identity-axis separation`

Dynamic receiver class, Method holder, access owner, lexical super anchor, and foreign layout owner remain distinct runtime concepts.

### `INV-04 — Strong runtime lifetime`

Every strong identity required for later correct execution remains GC-reachable through every live runtime object/frame that owns it.

### `INV-05 — Public invocation atomicity`

Malformed public host/native invocation fails before receiver/argument/frame execution-window mutation.

### `INV-06 — Native activation bound`

Nested native Rust primitive activations are explicitly bounded independently of recursive guest-interpreter re-entry.

### `INV-07 — Coherent execution teardown`

Any fresh-execution transition closes open captures before referenced frame/stack storage is truncated or reused.

### `INV-08 — Diagnostic lifetime separation`

Active native authority is scoped to the native body. Escaping native error identity travels with the error/traceback it describes and is not ambient VM state.

### `INV-09 — Operation-specific numeric domains`

Numeric conversion is permitted only when the operation's normative mathematical contract permits approximation.

### `INV-10 — Transactional buffered output`

Only bytes reported accepted cease to be pending at the wrapper. Unaccepted bytes remain recoverable exactly.

### `INV-11 — Exact opaque Resource identity`

Every valid managed Resource handle encoding is injective and exact; malformed carriers fail closed; generation never wraps; stale identity never resolves to a reused occupant.

### `INV-12 — No speculative performance rewrite`

Correctness remediation must not absorb unrelated unmeasured runtime/cache redesign.

### `INV-13 — Durable checkpoint truth`

`TEST002.C2/CHECKPOINT.md` is the single shared durable state record for the checkpoint. The plan describes intent; walkthrough/checkpoint describe what actually happened.

---

## 10. Non-goals

Do not expand the plan into:

- a general typed bytecode IR;
- widening every bytecode operand to `usize`;
- a serialized bytecode compatibility version;
- a generic GC tracing framework rewrite;
- global rooting of historical class objects;
- generalized native exception redesign beyond the provenance transport needed here;
- a new Rational library;
- a new `Value::ResourceHandle` tag;
- a native finalizer/Drop-based Resource lifecycle;
- filesystem/reactor ownership;
- a public `writeAll` selector;
- configurable buffering capacity;
- transparent implicit serialization of stream calls;
- full NATV001 C3/C4 implementation outside what C2 needs;
- opportunistic TEST002 directory reorganization.

---

## 11. Expected impact map

### 11.1 Expected compiler/runtime source

```text
phalcom-core/src/chunk.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/compiler/lib/error.rs
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/scope.rs
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/jumps.rs
phalcom-core/src/compiler/lib/loops.rs
phalcom-core/src/compiler/lib/patterns.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/product_opt.rs
phalcom-core/src/heap/closure.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/frame.rs
phalcom-core/src/method/object.rs
phalcom-core/src/vm/mod.rs
phalcom-core/src/vm/api.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/control.rs
phalcom-core/src/interpret.rs
phalcom-core/src/error.rs
phalcom-core/src/diagnostics/traceback.rs
phalcom-core/src/primitive/number.rs
phalcom-core/src/primitive/float.rs
phalcom-core/src/resource.rs
phalcom-core/src/primitive/resource.rs
phalcom-core/src/vm/bootstrap.rs
phalcom-core/src/universe/invariants.rs
phalcom-core/core/universe/src/collections/bytes.ph
```

`vm/data.rs`, `vm/adt.rs`, native descriptor installation, disassembly, or generated Method installation may require mechanical updates where they construct `ClosureObject`, Method metadata, or primitive rows.

### 11.2 Expected tests

```text
phalcom-core/tests/core/audit_probes/runtime_invariants.rs
phalcom-core/tests/core/mod.rs
phalcom-core/tests/core/compiler/*
phalcom-core/tests/core/repl/session.rs
phalcom-core/tests/core/memory/gc.rs
phalcom-core/tests/core/execution/depth_limits.rs
phalcom-core/tests/core/execution/send_arity.rs
phalcom-core/tests/core/execution/repair_regressions.rs
phalcom-core/tests/core/observability/super_disassembly.rs
phalcom-core/tests/core/observability/traceback.rs
phalcom-core/tests/core/language/numeric_values.rs
phalcom-core/tests/core/language/numeric_diagnostics.rs
phalcom-core/tests/fixtures/language/streams/*
```

Private GC/frame tests may correctly live as `#[cfg(test)]` unit tests inside runtime modules when `pub(crate)` authority fields cannot be reached from integration tests. Prefer that over widening production visibility.

### 11.3 Expected specification/state docs

```text
docs/specs/vm/bytecodes.md
docs/specs/vm/vm-execution-model.md        # only if old super/native wording is affected
docs/implementation/test/TEST002-baseline-stabilization/PROGRAM.md
docs/implementation/test/TEST002-baseline-stabilization/C2-runtime-correctness-remediation/CHECKPOINT.md
.../TEST002.C2.P1-runtime-correctness-remediation.md
.../TEST002.C2.P1-runtime-correctness-remediation-walkthrough.md
.../TEST002.C2.P1-runtime-correctness-remediation-handoff.md
```

Do not opportunistically rewrite historical audit or NATV001 state records.

### 11.4 Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem—parser, semantic database, LSP, reactor/external FS ownership, serialized artifacts—when doing so changes architecture or ownership.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change these choices:

1. bounded operands remain compact and are checked before publication;
2. `super` anchor is stable runtime behavior identity, not a name;
3. access authority and `super` authority remain distinct;
4. current superclass is read from the stable anchor at send time;
5. strong authority identities are traced rather than globally rooted;
6. `native_reentry_depth` remains the recursive-interpreter/fiber-floor counter;
7. native Rust activation receives a separate counter;
8. public call shape is validated before stack-window mutation;
9. raw-run reuse uses close-before-truncate unwind;
10. native diagnostic provenance is error-associated, not singleton ambient state;
11. exact numeric operations reuse exact Float decoding;
12. Resource uses exact 21+32-bit managed packing and a hidden runtime slot;
13. Resource generation never wraps;
14. `Resource.register` is retired in favor of instance attachment;
15. BufferedWriter supports short writes;
16. zero progress during required draining is a defined failure, not an infinite retry;
17. public `flush` propagates to `inner.flush`, even when the wrapper local buffer is empty;
18. unresolved overlapping BufferedWriter mutation is rejected, not secretly queued;
19. full bytecode verifier/performance-cache work stays deferred.

### 12.2 MECHANICALLY FLEXIBLE

Luna may adapt without consultation:

- private helper names;
- whether small checked-conversion helpers live in `error.rs`, `scope.rs`, or a private limits module;
- local iterator/data-structure choices;
- private enum/struct names for Resource table state;
- exact name of `super_anchor`;
- exact transparent `PhError` native-origin variant name;
- test function names;
- splitting one source file-local helper into another existing runtime module;
- source spelling of internal Resource methods if the live selector convention differs mechanically from NATV001 prose;
- use of existing Future `then`/`catch`/settlement helpers rather than pseudocode names;
- mechanically changed call signatures required by current HEAD.

### 12.3 VERIFY-FIRST

| Assumption | Verify in | If false |
|---|---|---|
| Canonical TEST002 root is the lowercase `docs/implementation/test/...` family | `PROGRAM.md`, implementation index | use canonical current family; consult if two active state authorities compete |
| C1 implementation through P2 is present | C1 checkpoint + current HEAD | record drift; consult only if C2 premise is invalid |
| `ExecutableSemanticPool` remains checked-before-push | `chunk.rs` | reuse another current checked producer pattern |
| `add_local` remains canonical local-allocation seam | `scope.rs` + call search | adapt owner mechanically; consult if multiple independent local allocators now exist |
| `super` exact and pack handlers still reconstruct by class name | compiler/bytecode/dispatch | if already repaired, verify architecture and delete redundant planned edits |
| all closure constructors can carry the new anchor | search `ClosureObject {` | update all; consult if an alternative executable object can contain source `super` |
| `native_reentry_depth` still participates in fiber floor/yield | VM/fiber code | do not repurpose it; consult if architecture changed |
| both primitive ABIs still share `send.rs` activation seams | `MethodKind::Primitive` call sites | centralize if possible; consult if a third unrelated activation path exists |
| caught traceback still uses `FrameRecord` / `Raise.traceback` | `control.rs`, `diagnostics/traceback.rs` | adapt provenance transport; consult if accepted trace model changed |
| ordinary Int->Float conversion has one reusable checked path or can be factored from `number.rs` | numeric primitives | create one canonical helper; do not duplicate conversion rules |
| Resource bootstrap still has field_count=1 plus `_handle` field slot | `vm/bootstrap.rs` | retain hidden slot requirement; adapt mechanics |
| native primitive installation is descriptor-driven | native install + census tests | update current descriptor authority; do not resurrect obsolete `NEW_RESOURCE` machinery |
| Future supplies enough continuation/error composition for a nonblocking drain | live Future `.ph` class | adapt with existing continuation API; consult if error cleanup cannot be expressed without blocking |
| stream fixtures are selected by `language-corpus streams` | test target | adapt exact command mechanically |

---

## 13. Global STOP / CONSULT triggers

Stop editing and prepare a compact consultation packet if any of these occurs:

1. a fixed architecture appears impossible or materially incorrect;
2. a core plan premise is disproved;
3. ownership, identity, lifetime, representation, semantic authority, or a cross-layer contract must change;
4. a downstream public contract must materially change;
5. accepted semantic behavior/test oracle appears wrong;
6. the same nontrivial semantic failure persists after one serious corrective attempt;
7. two plausible fixes have materially different architecture;
8. an unexpected subsystem is required;
9. a workaround/special-case architecture is emerging;
10. duplicate canonical reasoning/state would be introduced;
11. soundness-critical GC, control, error, numeric, or stale-generation behavior is uncertain;
12. spec/code/test/plan authorities materially conflict;
13. an invariant must be weakened to make tests pass;
14. scope becomes materially more cross-cutting than this plan;
15. Method transplantation is discovered to intentionally redefine lexical `super`, contradicting the current callable dispatch rule;
16. a runtime identity this plan treats as strong is proven intentionally weak with a defined expiry protocol;
17. fiber correctness requires `native_activation_depth` and `native_reentry_depth` to be the same state variable;
18. no error-associated Symbol-only native-origin seam can be introduced without changing catch/error semantics materially;
19. the authoritative numeric contract differs from exact represented-value `~/` or floor-remainder `%`;
20. managed Resource requirements require a table capacity materially above the exact 21-bit index model;
21. stream semantics require implicit serialization rather than explicit overlap rejection.

A triggered consultation cannot be self-waived.

### Consultation packet

Provide only:

```text
Plan/task:
Starting/current revision:
Observed:
Expected invariant:
Smallest reproducer:
Current call/data path:
One passing comparator:
Serious correction already attempted (if any):
Why local architecture is insufficient:
Options with consequences:
Specific decision requested:
```

Do not ask an adviser to rediscover the repository.

---

## 14. Debugging budget

### 14.1 Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress:

```text
inspect
identify concrete cause
make one coherent correction
rerun smallest discriminating command
```

Examples: imports, changed helper signature, borrow checker, exhaustive match/caller migration, test filter.

### 14.2 Semantic failures

Before changing code after a semantic failure, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious correction for the same underlying semantic failure.

If it persists, STOP AND CONSULT.

### 14.3 Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

Never rerun an unchanged failing test unless something relevant to the failure changed.

---

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| `CV-OP-01` | first unrepresentable | INV-01 | constant index 65536 fails; index 65535 remains valid |
| `CV-OP-02` | other bounded families | INV-01 | local/upvalue/linked/field count and jump helper boundaries |
| `CV-OP-03` | hostile shortcut | INV-01 | no saturation/wrapped index aliases a valid entry |
| `CV-SUP-01` | identity persistence | INV-02,03 | old instance executes old lexical super after same-name class redefinition |
| `CV-SUP-02` | class-side | INV-02,03 | static/class Method anchors at metaclass |
| `CV-SUP-03` | nested closure | INV-02,04 | block containing super inherits enclosing anchor |
| `CV-SUP-04` | pack/rest | INV-02 | `SuperSendPack` uses same stable anchor |
| `CV-SUP-05` | binding/transplant | INV-02,03 | bound/exact Method invocation does not move anchor where production supports it |
| `CV-GC-01` | sole strong retainer | INV-04 | closure lexical class survives GC |
| `CV-GC-02` | new anchor | INV-04 | `super_anchor` survives as sole owner |
| `CV-GC-03` | access/layout | INV-04 | Method access owner and frame/closure layout owner survive sole-retainer GC |
| `CV-GC-04` | release | INV-04 | dropping true owner permits eventual collection |
| `CV-NAT-01` | admission negative | INV-05 | missing/excess selector arguments return controlled error |
| `CV-NAT-02` | atomic failure | INV-05 | stack/frame/native-context depths unchanged on invalid public call |
| `CV-NAT-03` | native-only recursion | INV-06 | native→native recursion trips native activation ceiling |
| `CV-NAT-04` | re-entry preservation | INV-06 | native→bytecode recursion still trips native re-entry ceiling |
| `CV-NAT-05` | fiber behavior | INV-06 | existing yield/re-entry floor tests remain valid |
| `CV-UNW-01` | reuse after failure | INV-07 | escaped capture from failed raw run cannot alias next run's stack slot |
| `CV-TRC-01` | direct native failure | INV-08 | direct native error names current native frame |
| `CV-TRC-02` | nested success | INV-08 | inner native success then outer failure names outer |
| `CV-TRC-03` | propagated inner failure | INV-08 | unchanged inner failure names inner, not outer |
| `CV-TRC-04` | caught/replaced | INV-08 | dropped inner error does not poison later outer error |
| `CV-TRC-05` | ABI parity | INV-08 | legacy and shape-aware primitive paths preserve same provenance |
| `CV-NUM-01` | precision boundary | INV-09 | mixed `~/` around 2^53 retains exact Int quotient |
| `CV-NUM-02` | sign matrix | INV-09 | positive/negative Float remainder follows divisor sign |
| `CV-NUM-03` | special values | INV-09 | signed zero, NaN, infinities match numeric spec |
| `CV-NUM-04` | limits | INV-09 | exact quotient still respects configured numeric bit limit |
| `CV-RES-01` | exact encoding | INV-11 | max valid 21/32 handle and interior 2^53 neighbors round-trip |
| `CV-RES-02` | malformed carrier | INV-11 | negative/fractional/NaN/Inf/>max fail closed |
| `CV-RES-03` | stale generation | INV-11 | reuse invalidates old handle without wrapping |
| `CV-RES-04` | terminal generation | INV-11 | MAX generation closes idempotently and retires |
| `CV-RES-05` | hidden layout | INV-11 | Resource subclass fields start after hidden slot; source `_handle` is absent |
| `CV-RES-06` | attachment transaction | INV-11 | invalid/double attachment leaves no orphan live row |
| `CV-STR-01` | repeated short write | INV-10 | 2/1/2 acceptance of `ABCDE` produces exactly `ABCDE` |
| `CV-STR-02` | zero progress | INV-10 | zero on nonempty drain rejects `WriteZeroError`, no loss |
| `CV-STR-03` | partial failure | INV-10 | accepted prefix stays accepted; exact suffix remains pending |
| `CV-STR-04` | layered flush | INV-10 | local-empty outer flush still invokes inner flush |
| `CV-STR-05` | overlap | INV-10 | concurrent mutation raises and `_busy` always releases |
| `CV-STR-06` | source ownership | INV-10 | delayed write snapshots caller Bytes before return |
| `CV-STR-07` | close recovery | INV-10 | dirty close mutates nothing; subsequent finish can recover |

Do not add redundant tests for every preparatory edit. Each coverage obligation needs sufficient evidence once.

---

## 16. Verification execution budget

### 16.1 Modes

#### BUILD MODE

Use only T0/T1 evidence that discriminates the current task.

Do not run whole `phalcom-core` or workspace tests after every task.

#### STABILIZE MODE

Run focused owning suites at G1–G7.

#### CERTIFY MODE

Run broader package/workspace gates only if explicitly requested for release/checkpoint certification.

### 16.2 Verification ladder

```text
T0 — exact reproducer/new regression
T1 — directly affected feature tests
T2 — owning subsystem/module suite
T3 — adjacent cross-layer integration
T4 — broad phalcom-core verification
T5 — workspace/release verification
```

### 16.3 Mandatory during BUILD

Only the task-local commands listed in §18.

### 16.4 Mandatory stabilization gates

- G1 bounded executable operands;
- G2 stable super identity + GC authority;
- G3 native admission/activation/teardown/provenance;
- G4 numeric correctness;
- G5 managed Resource-v2;
- G6 BufferedWriter/stream integration;
- G7 final focused C2 acceptance.

### 16.5 Explicitly deferred during BUILD

Do **not** reflexively run:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace --all-targets
```

Do not run unrelated language corpora.

These add little diagnostic value while architecture is incomplete.

---

## 17. Baseline / unrelated failure policy

Classify every unexpected failure:

```text
A — definitely caused by this patch
B — probably caused by this patch
C — unclear
D — clearly unrelated/baseline
```

- A/B: active responsibility.
- C: perform one bounded classification pass. If nonblocking and still unclear, record in `TEST002.C2/CHECKPOINT.md` and continue.
- D: record and defer immediately.
- do not consume adviser compute on D unless it blocks acceptance;
- do not weaken assertions or skip a required test to make a broad gate green;
- C1 certification failures stay in C1 unless C2 demonstrably caused them.

---

# 18. Tasks

## T1 — Establish canonical C2 execution records and drift baseline

### Purpose

Create the durable C2 state authority, attach P1 to TEST002, and record the actual local execution baseline before code changes.

### Risk

- Semantic: LOW
- Implementation fanout: documentation/local Git state

### Preconditions

- `TEST002.C1.P2` implementation exists on the starting revision.
- No active C2 state record already exists in another canonical location.

### Consumes

- TEST002 program record;
- C1 checkpoint/walkthrough/handoff;
- this plan.

### Produces

- canonical `TEST002.C2` checkpoint record;
- P1 ledger entry marked active when execution actually starts;
- exact starting revision and baseline notes.

### Files and symbols

**Read:**

```text
docs/implementation/test/TEST002-baseline-stabilization/PROGRAM.md
docs/implementation/test/TEST002-baseline-stabilization/C1-semantic-core-baseline-elimination/CHECKPOINT.md
```

**Create if absent:**

```text
docs/implementation/test/TEST002-baseline-stabilization/C2-runtime-correctness-remediation/CHECKPOINT.md
```

**Add/copy into repository execution record:**

```text
.../TEST002.C2.P1-runtime-correctness-remediation.md
```

**Modify:**

```text
docs/implementation/test/TEST002-baseline-stabilization/PROGRAM.md
```

### Required implementation shape

1. Inspect local Git state.
2. Confirm no canonical C2 record exists.
3. Add C2 to the TEST002 checkpoint ledger without rewriting C1 history.
4. If `PROGRAM.md` currently describes TEST002 too narrowly as semantic/core-only stabilization, minimally broaden the objective/ownership text to include retained correctness baselines, while preserving its historical C1 description.
5. Create `CHECKPOINT.md` with:
   - `id: TEST002.C2`;
   - `status: IN_PROGRESS`;
   - `completion: NOT_STARTED` initially;
   - `verification: UNVERIFIED`;
   - bounded objective from §2;
   - plans ledger with P1;
   - inherited invariants;
   - empty verification/deferred/consultation ledgers;
   - starting revision;
   - active plan / next action.
6. Do not create a plan-specific `implementation-state.md`.

### Forbidden approaches

- no legacy `docs/implementation/TEST002/` C2 state;
- no opportunistic migration of C1;
- no claim that C1 certification passed;
- no implementation edits in this task.

### Tests to run now

None.

Run only:

```sh
git status --short
git rev-parse HEAD
```

These prove repository state, not runtime behavior.

### Acceptance

- canonical C2 checkpoint record exists;
- TEST002 program ledger references C2;
- P1 is recorded as active only when work actually starts;
- starting revision is truthful;
- no competing C2 state file exists.

### Local STOP / CONSULT triggers

- two active canonical TEST002 program/checkpoint roots compete;
- TEST002 ownership is explicitly constrained so runtime audits cannot belong to C2;
- local HEAD lacks C1.P2 implementation facts needed by this plan.

### Checkpoint update

This task creates the checkpoint authority; record starting state immediately.

---

## T2 — Make bounded executable operands fail before publication (`AUD-RUNTIME-001`)

### Purpose

Eliminate unchecked executable narrowing by making the producer that mints each bounded operand prove representability before publication.

### Risk

- Semantic: HIGH
- Implementation fanout: multi-file compiler

### Preconditions

- T1 complete.
- `CompilerError` remains the source compilation failure channel.
- compact bytecode operand widths remain unchanged.

### Consumes

- `INV-01`;
- current checked `ExecutableSemanticPool` precedent.

### Produces

- checked constant-pool insertion;
- checked linked-read encoding;
- checked local/scratch/upvalue indices;
- checked field counts/slots;
- checked jump offsets;
- structured overflow diagnostics.

### Files and symbols

**Primary:**

```text
phalcom-core/src/chunk.rs                         Chunk::add_constant
phalcom-core/src/compiler/lib/error.rs            CompilerError + checked helpers
phalcom-core/src/compiler/lib/scope.rs            Compiler::add_local
phalcom-core/src/compiler/lib/mod.rs              constant/global/link emission
phalcom-core/src/compiler/lib/class_decl.rs       field/rest layout
phalcom-core/src/compiler/lib/jumps.rs            jump patch/loop offsets
```

**Exhaustive producer inventory:**

```text
phalcom-core/src/compiler/lib/loops.rs
phalcom-core/src/compiler/lib/patterns.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/product_opt.rs
phalcom-core/src/compiler/
```

**Tests:**

```text
phalcom-core/tests/core/audit_probes/runtime_invariants.rs
phalcom-core/tests/core/compiler/
phalcom-core/tests/core/mod.rs
```

### Inspect before editing

Run:

```sh
rg -n 'as u16|as u8|as i32|unwrap_or\(u16::MAX\)|try_from' \
  phalcom-core/src/chunk.rs phalcom-core/src/compiler
```

Classify every hit that contributes to executable meaning:

```text
A — producer must become checked
B — conversion is dominated by a stricter already-proven bound
C — not an executable operand/count
```

Do not mechanically replace category B/C.

### Source of truth

Producer-side wide integer state is authoritative until a checked narrow operand is created.

### Required implementation shape

#### A. General compiler overflow error

Prefer a general structured error rather than abusing semantic-pool-specific wording.

STRUCTURAL:

```rust
CompilerError::ExecutableOperandOverflow {
    kind: &'static str,
    found: usize,          // or a wider signed form for offsets
    limit: usize,
    span: SourceRange,
}
```

Equivalent names are allowed.

Provide checked helpers for at least:

```text
u16 index: max logical index 65535
u16 count: max logical count 65535
u8 count:  max logical count 255
i32 relative offset: full signed i32 range
```

Do not apply one blanket bound to indices and counts.

#### B. Constant pool

Change `Chunk::add_constant` so representability is checked **before** push.

Target flow:

```text
index = constants.len()
checked_u16_index(index)?
constants.push(value)
return encoded_index
```

A `u16` index permits 65,536 addressable entries (`0..=65535`). Entry 65,537 must fail before mutation.

Propagate fallibility through the compiler's constant helper and callers. Mechanical `?` propagation is expected and does not require consultation.

#### C. Locals/scratch

Make `Compiler::add_local` the canonical proof seam.

Preferred shape:

```rust
fn add_local(...) -> Result<u16, CompilerError>
```

where the returned slot is checked from the pre-increment `num_locals`.

Callers that need the slot consume the returned `u16`; callers that do not may discard it.

This permits slot index `65535` where the surrounding frame representation can support 65,536 locals. Do not copy class-field count limits into local-slot semantics.

Scratch allocators must reuse the same local-slot proof.

#### D. Upvalues

Search every `UpvalueDescriptor`, `GetUpvalue`, `SetUpvalue`, capture-index producer.

Check the index at the capture/upvalue descriptor owner. Once encoded, bytecode emission may trust it.

#### E. Linked reads

Remove:

```rust
binding.0 as u16
```

and:

```rust
u16::try_from(index).unwrap_or(u16::MAX)
```

If a canonical binding index is already guaranteed narrower by its creating type, make that proof explicit. Otherwise reject at the insertion/encoding boundary.

Never saturate to a valid unrelated entry.

#### F. Fields

Perform inherited-count arithmetic in `usize` (or wider), then check.

Important distinction:

```text
field_count: u16
```

cannot represent a count of 65,536, even though an isolated `u16` index can represent 65,535.

Therefore:

```text
max total field_count = 65,535
max valid field slot under that count = 65,534
```

unless live `ClassObject` representation has changed.

Check instance and static fields symmetrically.

#### G. Rest layout / u8 counts

For parameter/rest metadata stored in `u8`, either:

- rely on a verified earlier 255-argument declaration limit and use an explicit dominated conversion; or
- perform the check at `RestLayout` construction.

Do not leave unexplained raw narrowing.

#### H. Jumps

Compute relative offsets in a wide signed type first.

STRUCTURAL:

```rust
fn checked_relative_offset(
    from_next_ip: usize,
    target: usize,
    span: SourceRange,
) -> Result<i32, CompilerError>
```

Use it for forward jumps and loops.

Do not cast both endpoints to `i32` and subtract.

### Migration / deletion evidence

After implementation:

```sh
rg -n 'unwrap_or\(u16::MAX\)' phalcom-core/src/compiler phalcom-core/src/chunk.rs
```

Expected: zero production hits.

For raw `as u16/u8/i32`, every remaining executable conversion must be documented as dominated by an earlier checked bound or mechanically unrelated.

### Forbidden approaches

- widening all operands to `usize`;
- constant deduplication as the fix;
- VM bounds checks as primary correction;
- saturating indices;
- wrapping indices;
- relying on debug overflow;
- per-syntax ad-hoc limits;
- a full bytecode verifier in this task.

### Test changes required

1. Replace/promote the ignored constant-pool audit probe into a regression that expects controlled overflow rather than aliased execution.
2. Add boundary-unit tests for checked index/count/offset helpers.
3. Add one compiler/source-level regression proving a source program exceeding the constant pool fails compilation instead of executing an aliased value.
4. Add targeted helper-level boundary tests for fields/locals/jumps where constructing enormous source programs adds no information.

Coverage: `CV-OP-01..03`.

### Tests to run now

During BUILD:

```sh
cargo test -p phalcom-core chunk_constant_pool
cargo test -p phalcom-core --test core compiler_operand
cargo check -p phalcom-core --lib
```

Adapt filters to actual new test names.

`cargo check` is useful here because fallible `add_constant/add_local` signature fanout is large.

Do not run the full core target yet.

### Acceptance

- first unrepresentable constant/field/count/offset is a controlled compiler error;
- no producer publishes a truncated/saturated executable reference;
- compact operand representation is unchanged;
- existing argument-limit semantics are preserved.

### Local STOP / CONSULT triggers

- fixing one conversion requires changing a language-visible maximum rather than enforcing current representation;
- one executable producer bypasses all plausible shared ownership seams;
- checked construction requires a new serialized bytecode version.

### Checkpoint update

After G1, record `INV-01` as implemented and the checked-producer interface map.

---

## T3 — Replace name-based `super` with stable lexical behavior identity (`AUD-RUNTIME-002`)

### Purpose

Make lexical `super` depend on the Method definition's stable runtime behavior identity, then remove the mutable defining-name operand from both exact and packed super bytecodes.

### Risk

- Semantic: HIGH
- Implementation fanout: compiler + bytecode + VM + disassembly/spec

### Preconditions

- T2 is complete enough that new bytecode constants/operands use checked insertion.
- callable dispatch specification remains stable-anchor authority.

### Consumes

- `INV-02`, `INV-03`;
- Method install holder/metaclass identity.

### Produces

- `ClosureObject.super_anchor`;
- simplified exact and packed super bytecodes;
- no runtime class-name reconstruction for lexical super.

### Files and symbols

```text
phalcom-core/src/heap/closure.rs                 ClosureObject
phalcom-core/src/bytecode.rs                     SuperSend / SuperSendPack
phalcom-core/src/compiler/lib/scope.rs            compile_super_send
phalcom-core/src/vm/dispatch.rs                  Bytecode::Method / Closure / SuperSend*
phalcom-core/src/vm/data.rs                      generated closure construction if present
phalcom-core/src/vm/adt.rs                       generated closure construction if present
phalcom-core/src/method/object.rs                Method holder semantics
phalcom-core/src/diagnostics/disassembly*        bytecode display if applicable
phalcom-core/tests/core/repl/session.rs
phalcom-core/tests/core/observability/super_disassembly.rs
phalcom-core/tests/core/collections/outgoing_packs_completion.rs
docs/specs/vm/bytecodes.md
```

Search all:

```sh
rg -n 'ClosureObject \{' phalcom-core/src
rg -n 'SuperSend|SuperSendPack|defining_class' phalcom-core/src phalcom-core/tests docs/specs/vm
```

### Source of truth

`ClosureObject.super_anchor` is the executable lexical lookup authority.

### Required implementation shape

#### A. Add the independent anchor

STRUCTURAL:

```rust
pub struct ClosureObject {
    ...
    pub lexical_class: Option<ClassId>,
    pub super_anchor: Option<ClassId>,
    pub(crate) foreign_receiver_guard: Option<ForeignReceiverGuard>,
}
```

Do not replace `lexical_class`.

#### B. Stamp at Method installation

At `Bytecode::Method`:

```text
instance:
    holder        = defining instance class
    access_owner  = defining instance class
    lexical_class = defining instance class
    super_anchor  = defining instance class

static:
    holder        = defining metaclass
    access_owner  = defining instance class   # preserve current access rule
    lexical_class = defining instance class   # preserve current access rule
    super_anchor  = defining metaclass
```

This difference is load-bearing.

Generated class/ADT closure Methods that are installed outside `Bytecode::Method` must receive the same anchor if their source semantics permit `super`.

Do not assign a class anchor to non-class callables that cannot legally contain `super`.

#### C. Nested closures

At `Bytecode::Closure` materialization, inherit the lexical super anchor from the template when already stamped, otherwise from the currently executing enclosing closure in the same way lexical authority is inherited.

Do not recover it from a source name.

#### D. Simplify bytecode

Target:

```rust
SuperSend(u8, u16)
SuperSendPack { base_name: u16 }
```

Delete the defining-class constant from compiler lowering and VM decode.

`compile_super_send` must still reject `super` outside a class Method, but no longer synthesize `"<Name>.class"`.

#### E. Dispatch

Exact and packed paths:

```text
anchor = executing_closure.super_anchor
parent = heap.class(anchor).superclass
lookup selector from parent
```

If no parent: ordinary terminal miss / DNU path.

If trusted bytecode contains `super` while the executing closure has no anchor: return a controlled internal invariant error; do not fall back to receiver class or name lookup.

Read `anchor.superclass` at each send. Do not bake the parent into closure metadata.

#### F. Transplant/binding

Search Method exact-invocation/binding/transplant paths.

They may change receiver and/or installation holder depending on current feature semantics, but must not rewrite closure `super_anchor`.

If a current production path intentionally clones a Method closure, clone the anchor.

### Required documentation change

Update `docs/specs/vm/bytecodes.md` to describe anchor-carried super.

Only update `vm-execution-model.md` if it explicitly retains the old defining-name operand.

Do not weaken `callables/dispatch.md`.

### Forbidden approaches

- start at `receiver.class.superclass`;
- retain a versioned name lookup;
- use `lexical_class` as the sole super anchor;
- bake the current superclass into metadata;
- rewrite the anchor when a Method is bound to a new receiver;
- leave exact and pack paths with different authority sources.

### Test changes required

Coverage `CV-SUP-01..05`.

Required source/runtime tests:

1. **REPL class redefinition hostile case**:
   - cell 1 defines `A₁`, `B₁ < A₁` using `super`, retains `B₁` instance;
   - cell 2 redefines same names with different behavior;
   - old instance still executes through `B₁ -> A₁`;
   - new instance executes through new chain.
2. class-side/static super uses metaclass anchor.
3. nested block containing `super` preserves enclosing Method anchor.
4. packed/rest super path uses the same anchor.
5. where current Method binding/exact invocation surface permits a source Method to execute on another receiver, verify anchor remains source definition.

Update disassembly expectations for operand removal.

### Tests to run now

```sh
cargo test -p phalcom-core --test core repl_session
cargo test -p phalcom-core --test core observability_super_disassembly
cargo test -p phalcom-core --test core outgoing_packs_completion
```

Use exact new test filters during BUILD; run these module filters only at the end of T3.

### Acceptance

- no production super path reconstructs defining behavior via `ClassKey` + source name;
- class-side and instance-side anchors are correct;
- nested blocks inherit identity;
- current superclass mutation of the **same retained behavior**, if supported, remains observable.

### Local STOP / CONSULT triggers

- current accepted semantics intentionally move lexical super during Method transplantation;
- legal `super` is discovered in executable code with no class behavior identity;
- static super cannot be represented as metaclass anchoring without changing public class-side dispatch semantics.

### Checkpoint update

Do not mark G2 complete yet; T4 must add lifetime correctness for all new/existing identity edges.

---

## T4 — Complete GC tracing for runtime authority identities (`AUD-RUNTIME-003`)

### Purpose

Trace every strong lexical/access/layout identity retained by a live closure, Method, or frame, including the new super anchor, and make future payload-field additions harder to omit silently.

### Risk

- Semantic: HIGH
- Implementation fanout: local heap/frame tracing + tests

### Preconditions

- T3's final closure metadata shape exists.

### Consumes

- `INV-03`, `INV-04`;
- closure/Method/frame authority fields.

### Produces

- complete strong-edge tracing;
- compile-visible payload classification pressure.

### Files and symbols

```text
phalcom-core/src/heap/trace.rs                   Object::Closure / Object::Method / trace_frame
phalcom-core/src/heap/closure.rs                 ClosureObject fields
phalcom-core/src/method/object.rs                MethodObject fields
phalcom-core/src/frame.rs                        CallFrame / ForeignReceiverGuard
phalcom-core/tests/core/memory/gc.rs
phalcom-core/tests/core/audit_probes/runtime_invariants.rs
```

Private-field GC tests may be added inside `heap/trace.rs` or another in-crate `#[cfg(test)]` module.

### Source of truth

Owning runtime payload fields define strong edges; tracing enumerates them explicitly.

### Required implementation shape

Trace at minimum:

```text
ClosureObject:
    module
    upvalues
    constant Value edges
    lexical_class
    super_anchor
    foreign_receiver_guard.layout_owner

MethodObject:
    closure, if bytecode-backed
    holder
    access_owner
    contract Values
    attribute Values

CallFrame:
    closure
    receiver/context roots already required
    foreign_receiver_guard.layout_owner
```

#### Mechanical hardening

Where practical, destructure handle-bearing payloads exhaustively in tracer code without `..`.

Conceptual:

```rust
let ClosureObject {
    callable,
    module,
    upvalues,
    lexical_class,
    super_anchor,
    foreign_receiver_guard,
} = closure;
```

Then explicitly classify every field.

Use a small `trace_foreign_receiver_guard` helper if it removes duplication.

Do **not** create a large generic tracing framework in this checkpoint.

### Strong/weak rule

Only identities required for later correct execution become strong roots.

Do not extend this task into unresolved typing/cache registry ownership.

### Forbidden approaches

- root all classes forever through `VM::classes`;
- convert strong identity to weak lookup plus `try_get`;
- make the receiver dynamic class stand in for access/layout authority;
- speculative tracing of every cache in the VM.

### Test changes required

Coverage `CV-GC-01..04`.

Each edge test should isolate the field as the only strong route:

```text
allocate owner O
allocate holder H containing only tested edge to O
root H
remove alternative roots/registry paths
force GC
assert O survives
exercise semantic behavior if practical

drop H
force GC
assert O can be reclaimed
```

Promote/replace the ignored audit probe for `lexical_class`.

Add distinct evidence for `super_anchor`, `MethodObject.access_owner`, closure foreign guard, and frame foreign guard. Use in-crate unit tests where private fields make an integration test artificial.

### Tests to run now

```sh
cargo test -p phalcom-core --test core memory_gc
cargo test -p phalcom-core gc_lexical_owner
cargo test -p phalcom-core super_anchor_gc
```

Adapt filters to actual test names.

### Acceptance

Every live execution authority identity has a documented strong-edge owner and a tracer edge; sole-retainer tests pass; dropping the owner does not create a permanent global root.

### Local STOP / CONSULT triggers

- any audited authority edge is proven intentionally weak;
- correct tracing creates a cycle that violates an established weak-cache contract;
- parked fiber ownership requires a new GC root architecture rather than tracing the frame field.

### Checkpoint update

After G2, record stable `super_anchor` ownership and the strong-edge tracing invariant.

---

## T5 — Validate public sends before mutation and add a native activation budget (`AUD-RUNTIME-004`)

### Purpose

Make public call admission atomic and bound actual nested Rust native activations without corrupting the established re-entry/fiber model.

### Risk

- Semantic: HIGH
- Implementation fanout: VM send/activation

### Preconditions

- T3/T4 may be complete or mechanically independent.
- `native_reentry_depth` semantics remain as verified.

### Consumes

- `INV-05`, `INV-06`;
- `invocation_layout_for_selector`;
- both primitive ABI activation seams.

### Produces

- pre-mutation public call validation;
- `native_activation_depth`;
- `MAX_NATIVE_ACTIVATION_DEPTH`;
- parity across legacy/shape native activation.

### Files and symbols

```text
phalcom-core/src/vm/mod.rs                    VM fields, NativeMethodContext, constants
phalcom-core/src/vm/send.rs                   send_dynamic, call_method_legacy, shape activation
phalcom-core/src/vm/api.rs                    check_native_reentry + new activation check helper
phalcom-core/tests/core/execution/send_arity.rs
phalcom-core/tests/core/execution/depth_limits.rs
phalcom-core/tests/core/audit_probes/runtime_invariants.rs
```

### Source of truth

- selector/argument shape: canonical invocation-layout derivation;
- native Rust recursion: `native_activation_depth`;
- recursive interpreter/fiber floor: existing `native_reentry_depth`.

### Required implementation shape

#### A. Public `send_dynamic` admission

Before mutating execution stack/frame state:

```text
derive/validate invocation layout
then module/export forwarding as permitted by live architecture
then push receiver/arguments
then dispatch
```

If module-export dispatch needs a distinct internal route, still ensure the public selector/argument shape is admitted before guest execution state is changed.

Wrong public input should return a controlled arity/call-shape failure, not panic and not be described as an impossible internal error.

Compiler-produced bytecode paths do not need duplicate host-boundary validation if their compiler invariants already prove shape.

#### B. Native activation counter

Add:

```rust
pub const MAX_NATIVE_ACTIVATION_DEPTH: usize = 32;
pub(crate) native_activation_depth: usize;
```

or mechanically equivalent private state.

Add a check that returns:

```rust
RuntimeError::DepthExceeded {
    what: "native activation depth",
    limit: MAX_NATIVE_ACTIVATION_DEPTH,
}
```

before recursing into another Rust primitive body.

#### C. Activation lifecycle

For each primitive ABI:

```text
check
increment native_activation_depth
push NativeMethodContext
call native body
capture result/error origin as T7 will require
pop context
decrement native_activation_depth
reconcile result
```

During T5, it is acceptable for T7's provenance hook to be a no-op/placeholder helper only if the intermediate tree compiles; do not invent a temporary singleton mechanism.

Do not use an RAII guard that holds `&mut VM` across the primitive call and forces unsafe workarounds. Straight-line increment/call/decrement with explicit cleanup on every `PhResult` path is acceptable.

#### D. Preserve re-entry

Do not move the existing `native_reentry_depth` increment/check merely to catch native-only recursion.

Its existing placement/lifetime protects `run_until` and fiber switch restrictions.

### Forbidden approaches

- repurpose `native_reentry_depth`;
- primitive-local defensive `args.get(0)` as primary fix;
- mutate stack then rollback after shape failure;
- `catch_unwind` for public contract validation;
- recursively run interpreter for shape-aware `EnteredFrame`;
- tie native activation count to bytecode call-frame count.

### Test changes required

Coverage `CV-NAT-01..05`.

1. Rewrite the ignored arity audit probe to expect controlled error instead of panic.
2. Assert stack length, frame length, native-context depth, activation depth are unchanged on invalid public call. Prefer private unit test access over widening production API.
3. Convert native recursion audit probe into expected `DepthExceeded("native activation depth")`.
4. Preserve existing `native_reentry_is_bounded`.
5. Run existing fiber-yield/re-entry floor regressions affected by the counter distinction.

### Tests to run now

```sh
cargo test -p phalcom-core --test core execution_send_arity
cargo test -p phalcom-core --test core execution_depth_limits
```

Add one exact private activation test if integration visibility cannot prove state restoration.

### Acceptance

Malformed public sends are state-atomic; native-only recursion has a real independent ceiling; existing native re-entry and fiber-floor tests retain their semantics.

### Local STOP / CONSULT triggers

- module-export public sends require a materially different shape contract;
- a third primitive activation ABI exists outside the planned seams;
- activation accounting changes legal fiber behavior.

### Checkpoint update

Record the two-counter distinction as a durable invariant after the focused tests pass.

---

## T6 — Replace raw fresh-run clearing with canonical unwind (`AUD-RUNTIME-005 §4`)

### Purpose

Prevent stale open upvalues from aliasing future raw module runs while preserving post-failure diagnostic frames until the next fresh-entry transition.

### Risk

- Semantic: MEDIUM
- Implementation fanout: local VM entry point

### Preconditions

- `VM::unwind_to(0, 0)` still closes live-fiber upvalues before truncation.

### Consumes

- `INV-07`;
- existing REPL `unwind_cell` precedent.

### Produces

- safe fresh-run entry cleanup.

### Files and symbols

```text
phalcom-core/src/interpret.rs                    VM::run_in_module
phalcom-core/src/vm/dispatch.rs                  VM::unwind_to
phalcom-core/src/vm/api.rs                       unwind_cell comments
phalcom-core/tests/core/execution/repair_regressions.rs
phalcom-core/tests/core/repl/session.rs
```

### Required implementation shape

EXACT semantic change:

```rust
// old entry cleanup
self.frames.clear();
self.stack.clear();

// target entry cleanup
self.unwind_to(0, 0);
```

Do **not** automatically unwind immediately after `run()` returns an error.

Current reporting paths rely on failed-run frames remaining available until `report_runtime_error` / caller observation. The next fresh `run_in_module` invocation performs safe cleanup before storage reuse.

Use the current frame-construction helper if live code has since standardized entry frame generation; do not broaden this task solely to refactor frame construction.

### Forbidden approaches

- raw `frames.clear()` / `stack.clear()` as fresh-execution reset;
- closing only open upvalues but leaving another index-coupled structure stale;
- erasing error frames before report/capture;
- changing REPL cell behavior that already uses canonical unwind.

### Test changes required

Coverage `CV-UNW-01`.

Construct a raw-run analogue of the existing REPL test:

```text
run A:
    create local secret
    escape closure into durable module/global state
    raise before normal completion

observe/report failure if needed

run B:
    force stack slot reuse with unrelated locals
    invoke escaped closure

expected:
    old closure sees A's closed secret
    never B's reused value
```

Also assert existing traceback behavior for an uncaught raw-run failure remains present.

### Tests to run now

```sh
cargo test -p phalcom-core --test core repair_regressions raw_run
cargo test -p phalcom-core --test core repl_session open_upvalue_hygiene
```

### Acceptance

No raw-run fresh entry can reuse a stack slot beneath an open upvalue; existing error reporting remains intact.

### Local STOP / CONSULT triggers

- `unwind_to(0,0)` has side effects that are invalid at file/module entry;
- bootstrap/module initialization intentionally depends on retaining frames across separate `run_in_module` calls.

### Checkpoint update

No durable checkpoint update is necessary until G3 unless this reveals a new cross-run state invariant.

---

## T7 — Make native error provenance scoped and error-associated (`AUD-RUNTIME-005 §6`)

### Purpose

Remove singleton native diagnostic state and make nested native traceback identity follow the failure that actually escaped.

### Risk

- Semantic: HIGH
- Implementation fanout: VM activation + error/control + rendering

### Preconditions

- T5 activation/context lifecycle is stable.
- accepted TDR-0062 frame-record capture model remains in force.

### Consumes

- `INV-08`;
- `NativeMethodContext`;
- `FrameRecord`;
- `RuntimeError::Raise.traceback`;
- first-`on` capture flow.

### Produces

- scoped active native diagnostic identity;
- transparent error-associated native-origin transport;
- `FrameRecord::Native`;
- removal of `native_selector/native_class` ambient singleton fields.

### Files and symbols

```text
phalcom-core/src/vm/mod.rs                        NativeMethodContext / VM fields
phalcom-core/src/vm/send.rs                       legacy + shape primitive exits
phalcom-core/src/error.rs                         PhError / FrameRecord
phalcom-core/src/vm/control.rs                    first on-boundary capture
phalcom-core/src/vm/dispatch.rs                   fiber-floor/root failure conversion
phalcom-core/src/vm/api.rs                        capture helpers if appropriate
phalcom-core/src/diagnostics/traceback.rs          human + JSON rendering
phalcom-core/tests/core/observability/traceback.rs
phalcom-core/tests/core/execution/depth_limits.rs  ABI/re-entry interactions as needed
```

### Inspect before editing

Trace these paths on current HEAD:

```text
legacy primitive Err
shape primitive Err
    ↓
run_until / control transfer
    ↓
first on-boundary capture
    ↓
fiber-floor/root failure conversion
    ↓
human/json render
```

Also locate every pattern match on `PhError` that assumes only the current variants.

### Source of truth

- active native authority: `NativeMethodContext`;
- escaping native origin: error-associated Symbol-only transport;
- caught durable trace: `FrameRecord`.

### Required implementation shape

#### A. Active diagnostic identity

Extend `NativeMethodContext`:

```rust
// STRUCTURAL
diagnostic: NativeFrameIdentity {
    class: Symbol,
    selector: Symbol,
}
```

Use stable Symbols; do not retain `ClassId`/ObjRef in off-VM error transport.

#### B. Error-associated transport

Preferred structural shape:

```rust
// STRUCTURAL
PhError::NativeOrigin {
    source: Box<PhError>,
    class: Symbol,
    selector: Symbol,
}
```

Derive/display it transparently so user-visible message remains the underlying error.

Provide helpers conceptually equivalent to:

```text
with_native_origin(identity)
native_origin()
into_underlying()
```

`with_native_origin` rule:

```text
if error already carries a native origin:
    preserve it unchanged       # innermost producer wins
else:
    attach current native identity
```

Do not accumulate every native wrapper in a chain; this finding requires the producing native origin, not a synthetic native call stack.

#### C. Primitive boundaries

Before popping active native context:

```text
native result Ok -> no error transport
native result Err(e) -> e.with_native_origin(current diagnostic)
```

Then pop access context and decrement activation depth regardless of result.

#### D. First `on` boundary

At the existing first-boundary capture point:

1. extract any `NativeOrigin`;
2. use the underlying error for current surface-error wrapping/matching semantics;
3. capture normal guest frames as today;
4. append/insert exactly one `FrameRecord::Native { class, selector }` in correct innermost order;
5. store the combined records in `RuntimeError::Raise.traceback`;
6. do not leave the transport wrapper exposed to user `on(Error)` matching.

This preserves TDR-0062's rule that the durable traceback is captured at the first boundary rather than eagerly walking frames at raise.

#### E. Uncaught root/fiber path

If no `on` captures the error, the uncaught renderer/fiber-floor conversion must read the error-associated origin directly and render/capture it before the wrapper is discarded.

A propagated inner native failure remains inner.

#### F. Remove ambient singleton

Delete VM fields:

```text
native_selector
native_class
```

and all renderer dependence on them.

Do not replace them with `last_native_*`.

### FrameRecord

Add:

```rust
FrameRecord::Native {
    class: Symbol,
    selector: Symbol,
}
```

Update:

- human renderer;
- JSON renderer;
- grouping/partition logic;
- exhaustive matches;
- fiber/caught trace composition.

No `ObjRef` in the record.

### Forbidden approaches

- save/restore the singleton fields;
- leave outer native identity ambient on inner propagation;
- keep access authority alive just for rendering;
- VM-global `last_native_error`;
- store class ObjRef/ClassId in an error traveling the Rust stack;
- convert every native `PhError` to generic `RuntimeError::Raise` merely to carry provenance;
- duplicate native frame once in error transport and again from ambient renderer state.

### Test changes required

Coverage `CV-TRC-01..05`.

Required matrix:

| Scenario | Expected native frame |
|---|---|
| direct native failure | current native |
| outer native -> inner native success -> outer failure | outer |
| outer native -> inner native failure propagated unchanged | inner |
| outer native catches/drops inner failure -> creates later failure | outer |
| caught inner failure followed by later success | no stale native frame |
| legacy primitive path | same rule |
| shape-aware primitive path | same rule |
| native returns `EnteredFrame` successfully | no phantom native context |

Test both human and JSON rendering for at least one nested case.

### Tests to run now

```sh
cargo test -p phalcom-core --test core observability_traceback native
cargo test -p phalcom-core --test core execution_depth_limits
```

Add an exact unit/control test for transport preservation if CLI rendering alone would not distinguish propagated vs replaced failures.

### Acceptance

- singleton fields have zero production references;
- nested provenance matrix passes;
- caught errors expose ordinary underlying surface error semantics;
- no native authority context leaks past Rust body return.

### Local STOP / CONSULT triggers

- adding transparent `PhError` transport breaks accepted catch/GC semantics in a way not repairable locally;
- a non-`PhError` native failure route reaches the renderer;
- accepted TDR-0062 forbids even cheap Symbol-only origin attachment at the primitive boundary;
- correct provenance requires a general cause-chain redesign.

### Checkpoint update

After G3, record:
- separate activation/re-entry counters;
- fresh-run unwind;
- native-origin transport/FrameRecord design;
- exact focused evidence.

---

## T8 — Implement exact `~/` and Float floor remainder (`AUD-RUNTIME-006`)

### Purpose

Replace lossy convenience promotion in the audited operations with operation-specific exact arithmetic while preserving ordinary Float-domain conversion where the specification requires it.

### Risk

- Semantic: HIGH
- Implementation fanout: local numeric runtime + tests

### Preconditions

- current numeric specification still matches §5.
- `float_to_rational` remains exact and tested/usable.

### Consumes

- `INV-09`;
- `BigInt`;
- `float_to_rational`;
- existing numeric policy/normalization.

### Produces

- exact represented-value floor division;
- specified Float-domain remainder;
- one canonical checked Int->Float conversion helper if missing;
- one canonical exact dyadic->binary64 rounding helper if missing.

### Files and symbols

```text
phalcom-core/src/primitive/number.rs            promote_pair / number_floor_div / number_mod
phalcom-core/src/primitive/float.rs             float_to_rational + exact rounding helper
phalcom-core/src/value/*                        inspect only for normalization/policy
phalcom-core/tests/core/language/numeric_values.rs
phalcom-core/tests/core/language/numeric_diagnostics.rs
```

### Source of truth

- exact finite Float: `float_to_rational`;
- Int exact value: existing BigInt extraction;
- result physical normalization: existing `normalize_bigint`;
- numeric resource ceiling: existing `numeric_policy`.

### Required implementation shape

#### A. Do not delete useful ordinary Float promotion

`promote_pair` or equivalent may remain for operations whose contract is Float-domain arithmetic.

Do not force exact rational machinery into `+`, `-`, `*`, `/` unless already required by their spec.

#### B. Exact `~/`

Represent operands:

```text
Int i        -> i / 1
finite Float -> exact numerator / denominator from float_to_rational
```

For nonzero divisor:

```text
q = floor((a_num * b_den) / (a_den * b_num))
```

Return exact normalized Int.

Errors:

```text
zero divisor         -> divideByZero
NaN/infinity operand -> nonFiniteNumber
result bit limit     -> numericLimit
```

No Int operand is first rounded to Float.

Use `num_integer::Integer::div_floor` / `div_mod_floor` or current equivalent.

Avoid unnecessary guest allocation. Temporary `BigInt`s are Rust implementation details.

#### C. Canonical Int->Float conversion

For operations that **do** require Float-domain conversion (`%` when either operand is Float), factor/reuse one helper implementing the normative rule:

```text
round Int to nearest binary64, ties-to-even
if mathematical Int converts to infinity -> numericOverflow
```

Do not use `unwrap_or(NaN)`.

This helper may also improve existing Float-result arithmetic mechanically if it is currently the same incorrect conversion path; do not expand semantics beyond fixing the canonical conversion primitive.

#### D. Float-domain `%`

If either input is Float:

1. convert any Int operand through canonical Int->Float conversion;
2. handle NaN/Infinity/zero special cases exactly as numeric spec;
3. decode the resulting finite binary64 operands with `float_to_rational`;
4. compute:
   ```text
   q = floor(exact a / exact b)
   r_exact = exact a - q * exact b
   ```
5. round `r_exact` once to binary64, ties-to-even;
6. if result is zero, apply divisor's sign.

A host `%`/`fmod` plus sign patch is not the implementation authority.

#### E. Exact dyadic rounding helper

All finite Float operands are dyadic rationals. The exact remainder remains dyadic.

If no helper already exists, add a private helper that correctly rounds a signed dyadic rational to binary64 by:

- normalizing sign;
- locating the binary exponent from BigInt bit length and denominator power-of-two scale;
- selecting significand bits plus guard/sticky/remainder information;
- applying round-to-nearest, ties-to-even;
- carrying on significand overflow;
- handling normal/subnormal boundary;
- producing signed zero;
- producing infinity only where the operation's specified Float result overflow permits it.

Do not use an approximate intermediate that causes double rounding.

The exact helper signature is mechanically flexible; its correctness is not.

### Forbidden approaches

- `BigInt -> f64 -> exact Int quotient` for `~/`;
- host `%` plus a sign-only fix as normative `%`;
- decimal string conversion as arithmetic authority;
- guest Rational allocation;
- bypassing numeric bit-limit checks.

### Test changes required

Coverage `CV-NUM-01..04`.

Add to `numeric_values.rs`:

```text
2^53-1, 2^53, 2^53+1 mixed ~/
both signs / divisor signs
finite fractional Float exact ratios
-5.0 % 3.0 == 1.0
 5.0 % -3.0 == -1.0
-5.0 % -3.0 == -2.0
signed zero result follows divisor sign
subnormal operand cases
```

Add special-value/diagnostic assertions:

```text
~/ zero
~/ NaN / +/-Infinity
% +/-0.0 divisor -> NaN
% finite % Infinity -> NaN
% Infinity % finite -> NaN
% NaN -> NaN
numeric result limit
```

Add pure rounding-helper bit-pattern tests around:
- largest subnormal / smallest normal;
- halfway ties with even/odd low bit;
- signed zero.

### Tests to run now

```sh
cargo test -p phalcom-core --test core numeric_values
cargo test -p phalcom-core --test core numeric_diagnostics
cargo test -p phalcom-core dyadic
```

Adapt the last filter to the helper test name.

### Acceptance

The original audit probes produce specified results; 2^53 neighbors remain exact for `~/`; Float `%` sign/special/rounding behavior matches the numeric specification; ordinary numeric error kinds remain intact.

### Local STOP / CONSULT triggers

- numeric spec has changed;
- exact rounding requires a new external dependency with material policy consequences;
- canonical Int->Float conversion would materially change other operations beyond the accepted numeric spec;
- numeric resource-policy semantics conflict with exact temporary arithmetic.

### Checkpoint update

After G4, record operation-specific numeric-domain ownership and focused evidence.

---

## T9 — Rebuild managed Resource handle/table semantics around an exact bounded domain (`AUD-RUNTIME-008`, runtime half)

### Purpose

Replace lossy 64-bit numeric transport and wrapping table reuse with exact managed-handle encoding and non-wrapping lifecycle state.

### Risk

- Semantic: HIGH
- Implementation fanout: resource module + primitive consumers

### Preconditions

- accepted Resource remains deterministic/non-finalized managed ownership.
- no external native Resource kind currently depends on the old `ResourceEntry` layout.

### Consumes

- `INV-11`;
- selected Resource-v2 architecture.

### Produces

- exact pack/decode helpers;
- bounded table allocator;
- explicit lifecycle state;
- non-wrapping generation;
- deterministic stale/closed semantics.

### Files and symbols

```text
phalcom-core/src/resource.rs                 ResourceHandle / ResourceTable / ResourceKind
phalcom-core/src/primitive/resource.rs       consumers in preparation for T10
```

Inspect all `ResourceKind`, `ResourceTable::{open,resolve,close,is_closed,drain,leaks*}` consumers before editing.

### Source of truth

`ResourceTable` owns resource identity and generation. The hidden language-object slot stores only the validated handle carrier.

### Required implementation shape

#### A. Constants and exact encoding

Implement the fixed 21/32 domain.

Preferred pure helpers:

```rust
fn pack(handle: ResourceHandle) -> Result<f64, ResourceTableError>;
fn unpack(value: f64) -> Result<ResourceHandle, ResourceHandleDecodeError>;
```

Equivalent associated-function shapes are acceptable.

Pack validates `index <= RESOURCE_MAX_INDEX`.

Unpack rejects:

```text
NaN
+/-Infinity
negative
fractional
> 2^53 - 1
non-exact integer conversion
```

No truncation/rounding/modulo.

#### B. Table states

Use an explicit state model equivalent to:

```rust
enum ResourceState {
    Vacant,
    Open(OpenResource),
    Closed(ClosedResource),
    Retired(ClosedResource),
}
```

with generation stored on the slot.

The exact payload split is flexible. Preserve Rust-owned kind/open-site/leak data.

No `Value` or `ObjRef` in the table.

#### C. Generation transitions

```text
new slot -> Open(g=0 or established canonical initial generation)

Open(g < MAX) --close--> Closed(g)
Closed(g < MAX) --reuse--> Open(g+1)

Open(MAX) --close--> Retired(MAX)
Retired(MAX) --reuse--> never
```

Generation does not change on close.

Never use `wrapping_add`.

Matching closed/retired handle remains idempotently closed.

A stale generation never resolves to a new occupant.

#### D. Capacity

When no reusable slot exists and table length reaches `RESOURCE_MAX_SLOTS`, acquisition returns a controlled resource-allocation failure through current runtime error machinery.

Never `entries.len() as u32` without proof.

#### E. Narrow table API

Prefer operations with explicit semantics:

```text
open_managed
close
is_closed
ensure_open
leak_snapshot / current equivalent
drain
```

Avoid one overloaded mutable `resolve` becoming the public lifecycle authority.

#### F. Leak ordering

If current leak APIs are observable and slot reuse can reorder them, adopt the Resource-v2 allocation serial so snapshots remain deterministic across reuse.

Serial increment must be checked; no wrap.

Do not turn C2 into a shutdown-report redesign beyond what is required to preserve current leak behavior.

### Forbidden approaches

- full 32/32 `u64 -> f64`;
- merely changing carrier to unchecked Int;
- generation increment on close;
- generation wrap;
- unbounded stale-handle history;
- GC finalizers;
- heap references in ResourceTable;
- external descriptor/socket variants.

### Test changes required

Coverage `CV-RES-01..04`.

Replace the false `u32::MAX/u32::MAX` endpoint test.

White-box resource unit tests must cover:

```text
index 0 / generation 0
max valid 21-bit index / u32::MAX generation
2^53-1 exact endpoint
interior around 2^53 precision boundary
all malformed Float classes
close idempotence
reuse increments generation
stale old handle
MAX generation retirement
capacity boundary via helper/test-sized allocator seam if allocating 2^21 rows is wasteful
serial non-wrap/error if serial added
```

A test-only smaller capacity/generation helper is acceptable if production constants are separately asserted.

### Tests to run now

```sh
cargo test -p phalcom-core resource::tests
```

Do not run stream corpus yet; T10/T11 have intentionally incomplete integration.

### Acceptance

Pure handle domain is mathematically exact, malformed values fail closed, table reuse cannot revive stale identity, and no counter wraps.

### Local STOP / CONSULT triggers

- a live external ResourceKind owns OS/reactor state and depends on current table layout;
- current public API exposes raw index/generation numerically;
- capacity > 2^21 is an accepted requirement;
- deterministic leak ordering would require redesign outside the resource module.

### Checkpoint update

Wait for T10/G5 before recording Resource-v2 as integrated.

---

## T10 — Move managed Resource instances to hidden transactional attachment (`AUD-RUNTIME-008`, object/bootstrap half)

### Purpose

Make Resource identity runtime infrastructure rather than a source-managed `_handle`, then migrate all current managed Resource subclasses to the canonical lifecycle guard.

### Risk

- Semantic: HIGH
- Implementation fanout: bootstrap + primitives + Universe source + invariants

### Preconditions

- T9 ResourceTable APIs are stable.
- bootstrap still reserves Resource inherited slot 0.

### Consumes

- `INV-11`;
- exact table operations from T9.

### Produces

- hidden slot 0;
- `Resource#attach` internal primitive;
- `Resource#ensureOpen` internal primitive;
- removal of `Resource.register`;
- managed constructors attach transactionally;
- operational methods use canonical guard.

### Files and symbols

```text
phalcom-core/src/vm/bootstrap.rs
phalcom-core/src/primitive/resource.rs
phalcom-core/src/universe/invariants.rs
phalcom-core/src/native/* or descriptor census owner
phalcom-core/core/universe/src/collections/bytes.ph
phalcom-core/tests/core/object_model/invariants.rs
phalcom-core/tests/fixtures/language/streams/
```

### Source of truth

Hidden Resource slot 0 + ResourceTable.

### Required implementation shape

#### A. Keep inherited layout, hide symbol

Current bootstrap:

```text
field_slots["_handle"] = 0
field_count = 1
```

Target:

```text
field_count = 1
no source-visible `_handle` field slot
```

The slot remains physically present so subclass-declared fields continue to start at offset 1.

Add/strengthen bootstrap invariant asserting `Resource.field_count == 1`.

If reflective field enumeration derives from `field_slots`, verify `_handle` disappears rather than becoming a new public field.

#### B. Replace `register` with instance `attach`

Primitive responsibilities:

```text
validate receiver is Resource instance/subclass
validate diagnostic kind String
validate receiver is unattached
allocate/copy all fallible Rust metadata
allocate ResourceTable row
pack exact handle
write slot 0
return Unit/None per internal convention
```

No fallible guest-language work may occur between committed row allocation and slot write.

If an internal failure can still occur after row allocation, rollback the row in the same primitive before returning; do not leak an orphan row.

Selector spelling should follow current native descriptor conventions (`attach(_)` vs historical prose `attach_(_)`). The semantic operation is fixed; spelling is mechanical.

#### C. Slot extraction

Primitive handle decode:

- reads slot 0 directly by index;
- treats internal allocator-uninitialized/`None` state as unattached;
- decodes only through T9's validated helper;
- does not accept an arbitrary Int by first converting it through `f64`;
- distinguishes malformed/stale/closed as required by the operation.

#### D. `isClosed`

Remain total:

```text
unattached -> true
Open -> false
Closed/Retired -> true
stale -> true
malformed -> true
```

Do not use it as an operational guard.

#### E. `ensureOpen`

Add internal primitive:

```text
Open -> success
closed/retired -> UseAfterClose
stale -> UseAfterClose(stale)
malformed -> UseAfterClose(malformed)
unattached -> UseAfterClose
```

Use current Error `kind` convention.

#### F. Managed constructors

For all current managed stream Resource subclasses:

```text
validate args
initialize/copy ordinary managed state
attach as final potentially failing acquisition action
return
```

No source assignment to `_handle`.

This task performs constructor/guard migration sufficient for current classes; T11 completes BufferedWriter behavior.

#### G. Primitive authority / census

Current native installation is descriptor-driven. Update the actual descriptor authority and its invariant/census tests.

Do **not** resurrect a stale hand-written `NEW_RESOURCE` constant merely because an older implementation spec mentioned it.

### Deletion gates

```sh
rg -n 'Resource\.register|resource_register|_handle\s*=' \
  phalcom-core/src phalcom-core/core/universe/src phalcom-core/tests
```

Expected: no production managed Resource acquisition through old mechanism. Test text may mention old spelling only if explicitly asserting rejection/removal.

```sh
rg -n 'wrapping_add' phalcom-core/src/resource.rs
```

Expected: zero.

### Forbidden approaches

- keep both `register` and `attach`;
- expose raw handle slot under another source name;
- make `attach(kind)` select arbitrary native capability behavior;
- use `isClosed` as the operational guard;
- make close asynchronous;
- external handle adoption.

### Test changes required

Coverage `CV-RES-05..06`.

Add:

1. Resource bootstrap hidden-layout invariant.
2. subclass with ordinary fields proving offsets remain correct after hidden slot.
3. constructor failure before attach leaves no live row.
4. double attach fails transactionally.
5. use-after-close operation goes through `ensureOpen`.
6. `isClosed` is total for stale/malformed/unattached cases through unit/primitive boundary.
7. existing leak report/strict resource behavior remains observationally compatible unless T9's deterministic ordering intentionally improves it.

### Tests to run now

```sh
cargo test -p phalcom-core resource::tests
cargo test -p phalcom-core --test core object_model_invariants
cargo test -p phalcom-core --test core reflection_census
```

Use current exact invariant/census module names.

Do not require the full stream corpus until T11.

### Acceptance

All managed Resources acquire identity only through transactional instance attachment; slot 0 is hidden runtime infrastructure; old `register` path cannot silently run.

### Local STOP / CONSULT triggers

- hiding `_handle` changes subclass field offsets despite `field_count = 1`;
- source compiler requires a named inherited field to reserve the physical slot;
- current native descriptor authority cannot remove `register` without a public compatibility promise;
- a current Resource operation depends on source reading/writing `_handle`.

### Checkpoint update

After G5, record Resource-v2 managed identity/lifecycle and the retired `register` mechanism.

---

## T11 — Make BufferedWriter short-write-safe, layered, and overlap-safe (`AUD-RUNTIME-007`)

### Purpose

Implement the selected Streams-v2 BufferedWriter state machine while preserving synchronous dirty-close semantics and current public Writer surface.

### Risk

- Semantic: HIGH
- Implementation fanout: Universe `.ph` stream classes + fixtures

### Preconditions

- T10 Resource attachment/ensureOpen integration is complete.
- current Future implementation supports continuation composition and rejection handling without internal `await`.

### Consumes

- `INV-10`, `INV-11`;
- Resource-v2 managed guards;
- Writer protocol allowing short writes.

### Produces

- `_busy` gate;
- short-write-safe internal drain;
- layered `flush`;
- deterministic protocol errors;
- exact pending-suffix recovery;
- `finish` as one overlap-protected lifecycle operation.

### Files and symbols

```text
phalcom-core/core/universe/src/collections/bytes.ph
phalcom-core/tests/fixtures/language/streams/
phalcom-core/tests/core/language/corpus.rs or current language-corpus registration
```

Inspect current `Future.then`, `Future.catch`, rejection constructors, and error propagation before editing. Reuse them; do not create native stream primitives.

### Source of truth

`BufferedWriter._buf[0.._len]` is the exact local set of accepted bytes not yet handed off to `_inner`.

### Required implementation shape

#### A. State

Add:

```phalcom
_busy = false
```

Constructor order:

```text
validate inner ownership precondition required by current Resource wrapper contract
_inner = inner
_buf = Bytes.new(8192)
_len = 0
_busy = false
self.attach("BufferedWriter")   # final potentially failing acquisition
```

Capacity remains 8192 in one documented location.

#### B. Entry guards

For lifecycle-sensitive operations, use canonical `ensureOpen`.

For state-mutating async operations:

```text
reject if _busy
acquire gate before unresolved work starts
release on fulfillment
release on rejection
release if synchronous start throws before Future chain is safely established
```

Do not wedge `_busy`.

`pending` is observational local `_len` and may remain readable while busy if selected C4 architecture/current protocol permits; it is not recursive inner pending.

#### C. Internal drain

Implement a private/internal composition operation equivalent to `_drainBuffer`.

If `_len == 0`, drain itself may fulfill immediately **without** calling inner flush.

If nonempty:

1. snapshot `_buf[0.._len]`;
2. `accepted = 0`;
3. call `_inner.write(remaining)`;
4. validate settled count is integer `0 <= count <= remaining.size`;
5. `count == 0` on nonempty remainder -> reject `WriteZeroError`;
6. positive count -> advance and continue via Future composition;
7. success -> `_len = 0`;
8. rejection after prefix -> compact exactly `snapshot[accepted..]` into `_buf`, set `_len`, reject original error.

Do not resend an accepted prefix.

#### D. Structural Writer result validation

Add pure `.ph` error classes if not already present:

```phalcom
class ConcurrentOperationError is Error {}
class StreamProtocolError is Error {}
class WriteZeroError is Error {}
```

`UnflushedError` remains.

No native primitive delta.

#### E. `write(src)`

Entry:

```text
ensureOpen
validate Bytes
reject overlap
zero length -> fulfilled 0
```

If source is copied only after an asynchronous drain would complete, snapshot `src` **before return**.

Small writes that fit can settle immediately and need not leave `_busy` held after return.

When local capacity requires draining:
- drain existing accepted bytes;
- then accept the owned snapshot;
- do not use public `flush` merely to make room, because that would impose an inner durability barrier.

Large direct writes must preserve previously accepted ordering and obey the same short-write/source-ownership rules. If the selected Streams-v2 design delegates large writes directly after draining, validate/report the direct Writer count according to public `write` semantics; do not secretly promise `writeAll` unless the wrapper has already accepted the full input.

#### F. Public `flush`

Exactly:

```text
acquire operation gate
drain local buffer completely
then call _inner.flush
then fulfill None
release gate
```

Even `_len == 0` still invokes `_inner.flush`.

If local drain fails:
- do not call inner flush;
- retain exact unwritten suffix;
- reject.

If inner flush fails after local drain:
- `_len == 0`;
- do not recreate local bytes;
- reject;
- retrying public flush calls inner flush again.

#### G. `close`

Decision order:

```text
if wrapper already closed -> idempotent success
if busy -> ConcurrentOperationError
if _len > 0 -> UnflushedError, mutate nothing
close owned inner Resource
after inner close returns a Result, close wrapper Resource row
return inner result per current Resource contract
```

Do not flush or block in `close`.

Nested BufferedWriter case: if inner close raises dirty-close, outer remains open.

#### H. `finish`

One gate-owning operation:

```text
flush through inner layer
then perform clean close sequence
```

No write/flush/close may interleave between successful flush and close.

Release `_busy` on every success/failure path.

### BufferedReader scope

T10 migrates `BufferedReader` constructor/guard to Resource-v2.

C2.P1 does **not** require the full Streams-v2 BufferedReader overlap/refill redesign unless current tests expose a correctness dependency. Do not expand into it merely for symmetry.

### Forbidden approaches

- make Writer full-write-only;
- clear `_len` before acceptance;
- retry zero forever;
- call public `flush` just to make local room;
- skip `inner.flush` when local buffer is empty;
- implicit queue/serialization of overlapping operations;
- internal blocking `await`;
- asynchronous `close`;
- retain caller-owned Bytes across unresolved work without snapshot.

### Test changes required

Coverage `CV-STR-01..07`.

Build deterministic `.ph` test writers:

1. **Scripted short writer** accepts counts `2, 1, 2` for `ABCDE`; final bytes exactly `ABCDE`.
2. zero-progress writer returns 0 on nonempty write; flush rejects and pending bytes remain.
3. partial-prefix then rejection; pending equals exact suffix.
4. nested BufferedWriter where outer local buffer is empty but inner has pending bytes; outer flush reaches inner flush.
5. inner flush rejection after local drain; retry invokes inner flush again.
6. delayed write source snapshot: mutate caller Bytes immediately after `write` returns; output remains original bytes.
7. unresolved operation followed by write/flush/finish/close raises overlap error.
8. both success and rejection release `_busy`.
9. existing dirty-close fixture remains loud and recoverable through `finish`.
10. double close does not repeat inner close side effects.

Extend existing fixtures rather than creating redundant parallel harnesses.

### Tests to run now

During BUILD, run exact new fixture filters if available.

At the task boundary:

```sh
cargo test -p phalcom-core --test language-corpus streams
```

### Acceptance

Legal short writes cannot lose/reorder bytes; zero progress terminates; partial failure preserves exact suffix; layered flush propagates; overlap is deterministic; dirty close semantics remain recoverable.

### Local STOP / CONSULT triggers

- Future API cannot express rejection cleanup without internal blocking;
- accepted Writer protocol treats zero as a legitimate indefinitely retryable would-block signal;
- wrapper ownership of inner Resource differs from selected stream contract;
- fixing the writer requires reactor cancellation/backpressure policy.

### Checkpoint update

After G6, record BufferedWriter drain/gate/layered-flush architecture and stream evidence.

---

## T12 — Stabilize, remove obsolete mechanisms, synchronize specs/state, and produce completion records

### Purpose

Prove the integrated C2 correction, remove stale authority paths, update durable records, and create a compact next-session handoff.

### Risk

- Semantic: MEDIUM
- Implementation fanout: integration/docs

### Preconditions

- T2–T11 implemented.
- G1–G6 focused gates passed or all failures classified.

### Consumes

- all `INV-*`;
- verification ledger;
- consultation/deferred ledgers.

### Produces

- final negative/deletion evidence;
- updated VM bytecode documentation;
- current checkpoint/program state;
- walkthrough;
- handoff.

### Files

```text
docs/specs/vm/bytecodes.md
docs/specs/vm/vm-execution-model.md             # only if affected
docs/implementation/test/TEST002-baseline-stabilization/PROGRAM.md
docs/implementation/test/TEST002-baseline-stabilization/C2-runtime-correctness-remediation/CHECKPOINT.md
.../TEST002.C2.P1-runtime-correctness-remediation-walkthrough.md
.../TEST002.C2.P1-runtime-correctness-remediation-handoff.md
```

### Required implementation shape

1. Run negative searches in §25.
2. Update VM bytecode spec to the actual stable-anchor super representation.
3. Do not rewrite numeric/stream specs that already state the target semantics.
4. Record any mechanical divergence from proposed NATV001 C3/C4 in the C2 walkthrough; do not silently edit those other program records into "implemented".
5. Run G7.
6. Update checkpoint:
   - P1 status/completion/verification;
   - established invariants;
   - stable interface map;
   - verification evidence;
   - deferred findings;
   - consultations/amendments;
   - next action.
7. Create walkthrough following lifecycle convention.
8. Create handoff:
   - if a C2 corrective follow-up is required, point directly to it;
   - otherwise state C2 is closed and identify remaining TEST002/C1/program certification state.

### Deferred ledger entries required

At minimum:

```text
AUD-RUNTIME-005.1 — full executable chunk verifier
Classification: deferred hardening
Revisit: before externally supplied/cached executable chunks become trusted

AUD-RUNTIME-005.2 — rest/native metadata optimization
Classification: performance, unmeasured
Revisit: profiling evidence

AUD-RUNTIME-005.3 — sparse per-instruction metadata
Classification: performance/memory, unmeasured
Revisit: profiling/memory budget evidence
```

Record any other retained but unexecuted leads explicitly rather than implying C2 fixed them.

### Tests to run now

Only G7 from §19.

### Acceptance

- all C2 required coverage is mapped to passing evidence;
- no obsolete authority path remains unintentionally;
- checkpoint state is truthful;
- walkthrough/handoff exist;
- no active incident remains.

### Local STOP / CONSULT triggers

- a required negative gate reveals two active competing authorities;
- a final focused test contradicts a fixed invariant after one serious correction;
- completing C2 would require claiming an unexecuted broad certification gate passed.

### Checkpoint update

This task performs final checkpoint synchronization.

---

# 19. Verification gates

## G1 — Bounded executable construction

### Purpose

Prove `INV-01` before later runtime work depends on newly emitted bytecode.

Run the focused new tests from T2, then:

```sh
cargo check -p phalcom-core --lib
```

and the targeted compiler/core regression module.

Required proof:

```text
max representable operand succeeds
first unrepresentable operand returns controlled compiler error
no saturation alias
fallible API fanout compiles
```

Do not broaden after PASS.

On failure: classify before editing.

---

## G2 — Stable super identity and authority lifetime

### Purpose

Prove T3/T4 together; stable identity without lifetime is incomplete.

Run:

```sh
cargo test -p phalcom-core --test core repl_session
cargo test -p phalcom-core --test core observability_super_disassembly
cargo test -p phalcom-core --test core outgoing_packs_completion
cargo test -p phalcom-core --test core memory_gc
cargo test -p phalcom-core super_anchor_gc
```

Adapt test filters to live module names.

Required proof:

- old/new same-name classes coexist correctly;
- static/nested/pack super share stable anchor semantics;
- sole-retainer GC edges survive;
- dropping actual owner does not leak globally.

Negative search:

```sh
rg -n 'defining_class|defining_idx|"<Name>\.class"|ClassKey' \
  phalcom-core/src/compiler/lib/scope.rs \
  phalcom-core/src/vm/dispatch.rs \
  phalcom-core/src/bytecode.rs
```

Expected: no production name-reconstruction authority for `super`. `ClassKey` may remain for unrelated class registry behavior.

---

## G3 — Native boundary, raw-run hygiene, and diagnostic provenance

### Purpose

Prove `INV-05..08` as one control-boundary gate.

Run:

```sh
cargo test -p phalcom-core --test core execution_send_arity
cargo test -p phalcom-core --test core execution_depth_limits
cargo test -p phalcom-core --test core repair_regressions raw_run
cargo test -p phalcom-core --test core observability_traceback native
```

Also run directly affected fiber-floor/yield tests located during T5.

Required proof:

- invalid public call is state-atomic;
- native-only recursion trips activation ceiling;
- native→guest recursion retains re-entry ceiling;
- stale upvalue cannot alias fresh raw run;
- direct/nested/propagated/replaced native error provenance is correct;
- native access context and counters restore on all controlled failure paths.

Negative search:

```sh
rg -n 'native_selector|native_class' phalcom-core/src
```

Expected: zero production uses after T7.

---

## G4 — Numeric domain correctness

### Purpose

Prove `INV-09` against both value and diagnostic behavior.

Run:

```sh
cargo test -p phalcom-core --test core numeric_values
cargo test -p phalcom-core --test core numeric_diagnostics
```

Run the exact dyadic-rounding unit-test filter if it is not included.

Required proof:

- mixed `~/` around 2^53 exact;
- full sign matrix;
- zero/NaN/Infinity cases;
- signed zero;
- numeric limit behavior;
- no diagnostic regression.

Do not run unrelated arithmetic corpus merely for ceremony.

---

## G5 — Managed Resource-v2

### Purpose

Prove `INV-11` before BufferedWriter relies on the new lifecycle.

Run:

```sh
cargo test -p phalcom-core resource::tests
cargo test -p phalcom-core --test core object_model_invariants
cargo test -p phalcom-core --test core reflection_census
```

Use the current census module filter if mechanically renamed.

Required proof:

- exact 21/32 packing;
- malformed decode;
- stale/retired generation;
- hidden inherited slot;
- transactional attach;
- canonical ensureOpen;
- old acquisition seam deleted.

Negative searches from T10 must pass.

---

## G6 — Stream transactional output

### Purpose

Prove `INV-10` over the integrated Resource-v2 surface.

Run:

```sh
cargo test -p phalcom-core --test language-corpus streams
```

If new stream tests are core integration rather than corpus fixtures, add only their focused target.

Required proof:

- repeated short writes;
- zero progress;
- partial rejection;
- layered/nested flush;
- retry after inner flush rejection;
- overlap;
- `_busy` release;
- source snapshot;
- dirty close/finish recovery.

Do not run filesystem/reactor suites; they are out of scope.

---

## G7 — Final focused C2 acceptance

### Purpose

Prove package-level compatibility after all runtime corrections without automatically escalating to workspace release certification.

Run:

```sh
cargo fmt -p phalcom-core -- --check
cargo check -p phalcom-core --all-targets
cargo test -p phalcom-core --lib
cargo test -p phalcom-core --test core
cargo test -p phalcom-core --test language-corpus streams
```

If `cargo fmt -p` is unsupported by the live toolchain, use the repository's narrow equivalent. Do not auto-format unrelated modified files.

Optional only if Resource shutdown/CLI code was actually changed:

```sh
cargo test -p phalcom-core --test cli-smoke
```

Do **not** automatically run workspace test/clippy. Those are CERTIFY-mode gates.

Expected classification on complete success:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

If a broad unrelated failure occurs, apply §17 rather than expanding scope.

---

## 20. Final focused acceptance mapping

At completion, the walkthrough must contain a concrete table like:

| Coverage | Test/evidence | Result |
|---|---|---|
| `CV-OP-01..03` | compiler operand boundary tests + G1 | PASS/FAIL |
| `CV-SUP-01..05` | redefinition/static/nested/pack/binding regressions | PASS/FAIL |
| `CV-GC-01..04` | sole-retainer GC tests | PASS/FAIL |
| `CV-NAT-01..05` | send admission + depth/fiber tests | PASS/FAIL |
| `CV-UNW-01` | raw-run capture reuse regression | PASS/FAIL |
| `CV-TRC-01..05` | native provenance matrix | PASS/FAIL |
| `CV-NUM-01..04` | numeric values/diagnostics + rounding unit tests | PASS/FAIL |
| `CV-RES-01..06` | resource unit/bootstrap/primitive tests | PASS/FAIL |
| `CV-STR-01..07` | stream corpus scripted-writer tests | PASS/FAIL |

Do not pre-fill PASS before tests execute.

---

## 21. Performance / resource evidence

This is a correctness plan, not a performance project.

No benchmark is required merely because:

- a constant insertion became checked;
- `super` lost one constant/name lookup;
- native activation gains one counter;
- exact numeric paths use BigInt;
- Resource table state becomes explicit;
- BufferedWriter uses continuation composition.

However, prevent obvious accidental pathological behavior:

- do not make ordinary `super` perform a registry scan;
- do not allocate a guest Rational;
- do not copy already-owned buffered data once per byte;
- do not build an implicit unbounded Future queue;
- do not allocate historical Resource generations.

If a focused test shows a severe performance regression plausibly caused by C2, classify it. Do not launch general optimization work without evidence.

---

## 22. Checkpoint bookkeeping

### At plan start

- create/normalize C2 checkpoint record;
- add P1 ledger entry;
- record local starting revision;
- mark plan `IN_PROGRESS`, `PARTIAL`, `UNVERIFIED`;
- record inherited C1.P2 state without claiming C1 certification.

### During plan

Update C2 only at meaningful events:

- G1 establishes checked executable producer contract;
- G2 establishes super/lifetime identity;
- G3 establishes native/control/diagnostic contracts;
- G4 establishes numeric exactness;
- G5 establishes Resource-v2 managed lifecycle;
- G6 establishes BufferedWriter state machine;
- any consultation/amendment;
- any significant deferred/baseline failure.

Do not write chronological test spam.

### At plan completion

Record:

- final P1 status/completion/verification;
- implemented invariants;
- stable interface/takeover map;
- gate evidence;
- deferred findings;
- consultations;
- residual risks;
- next action.

A clean Git tree does not imply completion.

---

## 23. Walkthrough deliverable

Create:

```text
TEST002.C2.P1-runtime-correctness-remediation-walkthrough.md
```

Required content:

1. plan identity and final lifecycle state;
2. objective and actual outcome;
3. architecture actually implemented;
4. important files/components changed;
5. durable interfaces/invariants established;
6. deliberate deviations from this plan;
7. consultation decisions/amendments;
8. tests added;
9. tests actually executed;
10. tests deliberately deferred;
11. performance/resource evidence only if relevant;
12. residual/baseline failures;
13. verification classification;
14. follow-up work.

Never claim unexecuted gates passed.

---

## 24. Handoff deliverable

Create:

```text
TEST002.C2.P1-runtime-correctness-remediation-handoff.md
```

Required content:

1. current repository revision/state anchor;
2. completed prerequisites;
3. stable interface/takeover map;
4. inherited architecture;
5. invariants not to redesign;
6. known repository drift;
7. deferred failures;
8. must-read files;
9. next objective:
   - C2 corrective plan if an accepted bounded remainder exists; or
   - program/checkpoint certification/next TEST002 action if C2 is closed;
10. high-risk areas;
11. first recommended commands/tests;
12. explicit do-not-reexplore guidance.

If C2 is terminal and no successor exists, the handoff may be compact but should still make closure state unambiguous.

---

## 25. Final negative / deletion gates

Run after T12 changes.

### 25.1 No saturation in compiler executable producers

```sh
rg -n 'unwrap_or\(u16::MAX\)' phalcom-core/src/compiler phalcom-core/src/chunk.rs
```

Expected: zero.

### 25.2 Class-name-based super authority removed

```sh
rg -n 'defining_class|defining_idx' \
  phalcom-core/src/bytecode.rs \
  phalcom-core/src/compiler \
  phalcom-core/src/vm/dispatch.rs \
  docs/specs/vm/bytecodes.md
```

Expected: zero references representing lexical super authority.

Any unrelated historical identifier must be justified in walkthrough.

### 25.3 Ambient native singleton removed

```sh
rg -n 'native_selector|native_class' phalcom-core/src
```

Expected: zero production references.

### 25.4 Raw fresh-run clear removed

```sh
rg -n 'frames\.clear\(\)|stack\.clear\(\)' phalcom-core/src/interpret.rs
```

Expected: no raw clear in `run_in_module`.

### 25.5 Old Resource acquisition removed

```sh
rg -n 'Resource\.register|resource_register|_handle\s*=' \
  phalcom-core/src \
  phalcom-core/core/universe/src
```

Expected: zero production acquisition/source-handle assignments.

### 25.6 Resource generation does not wrap

```sh
rg -n 'wrapping_add' phalcom-core/src/resource.rs
```

Expected: zero generation/serial wrap.

### 25.7 No host Float remainder in audited path

Inspect `number_mod` directly. A broad `%` search will legitimately find BigInt/parity operations, so do not use a naive zero-hit oracle.

### 25.8 Remaining narrow casts classified

```sh
rg -n 'as u16|as u8|as i32' phalcom-core/src/chunk.rs phalcom-core/src/compiler
```

Expected: every executable-semantic hit is either:
- removed;
- checked at that line; or
- dominated by a documented earlier bound.

Do not demand zero raw casts when a proven mechanical conversion is legitimate.

---

## 26. Suggested coherent commit groups

Only create commits if commit authorization exists.

Prefer these durable milestones; combine adjacent ones if the diff is small and coherent.

1. `test002: enforce bounded executable operands`
   - T2 implementation/tests
   - C2 checkpoint G1 update

2. `test002: preserve lexical super identity and runtime authority`
   - T3 + T4
   - super/GC tests
   - bytecode spec update if complete at this point
   - C2 G2 state

3. `test002: harden native activation and execution reuse`
   - T5 + T6
   - admission/depth/raw-run tests
   - C2 partial G3 state

4. `test002: preserve native diagnostic provenance`
   - T7
   - traceback tests
   - complete G3 state

5. `test002: implement exact floor division and remainder`
   - T8 + numeric tests
   - G4 state

6. `test002: migrate managed resources to exact v2 handles`
   - T9 + T10 + tests
   - G5 state

7. `test002: make buffered writer short-write safe`
   - T11 + stream fixtures
   - G6 state

8. `test002: complete C2 runtime remediation records`
   - final spec synchronization if not already committed;
   - G7 checkpoint state;
   - walkthrough/handoff.

Before every authorized commit:

```sh
git status --short
git diff -- <scoped paths>
```

Stage only the coherent work unit and its tests/checkpoint update.

Do not commit a materially advanced implementation while leaving `CHECKPOINT.md` stale.

Push only after coherent successful commits and only when separately authorized.

---

## 27. Deferred-evidence audit

Before final response, verify every deferred item has one of:

```text
executed successfully
explicitly removed from C2 scope with reason
recorded as known blocker/follow-up in C2 checkpoint
```

No silent deferred gate.

Explicit C2 deferrals:

| Item | Destination |
|---|---|
| full executable verifier | future trusted-bytecode hardening |
| native/rest metadata performance optimization | profiling-driven runtime plan |
| sparse per-instruction metadata/cache memory | profiling/memory plan |
| external Resource adoption | FS/reactor ownership plan |
| full BufferedReader v2 overlap redesign | Streams follow-up if independently required |
| workspace/release test+clippy | explicit TEST002 release certification |

---

## 28. Completion truth table

Do not conflate:

```text
source edited
tests written
targeted regression passed
owning suite passed
P1 administratively complete
C2 checkpoint accepted
TEST002 program complete
release certified
```

A valid P1/C2 result is:

```yaml
P1:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

C2:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED
```

only when all required C2 coverage and G1–G7 evidence have actually passed.

`TEST002` remains `IN_PROGRESS` if C1 or program-wide certification remains outstanding.

---

## 29. Failure protocol

For any unexpected required-test failure:

1. record exact command;
2. record exact failure/assertion;
3. trace direct path from test to failure;
4. find one passing comparator;
5. classify:
   ```text
   PRODUCT
   FIXTURE
   DEPENDENCY/PUBLICATION
   BACKEND/HARNESS
   BASELINE
   PLAN DRIFT
   ```
6. state the narrow permitted repair boundary;
7. reject broad fixes explicitly;
8. make at most one serious semantic correction;
9. rerun the smallest discriminator.

If required evidence still fails, mark the gate as an incident. Later dependent tasks must not treat its semantic contract as established.

Independent tasks may proceed only when their dependency graph genuinely does not consume the failed invariant.

---

## 30. Residual risks to report explicitly

The walkthrough must explicitly state final status of these risks:

- whether any executable narrow conversion remains justified only by a nonlocal invariant;
- whether any Method construction path could execute `super` without a `super_anchor`;
- whether any authority edge remains intentionally weak;
- whether any native failure route bypasses `PhError` origin transport;
- whether exact Float remainder has untested rounding regions;
- whether Resource capacity/retirement has operational implications not covered by current managed resources;
- whether any Stream Future rejection path can leave `_busy` wedged;
- whether C1 or TEST002 broad certification remains outstanding.

---

## 31. Plan self-review

Architecture:

- [x] every major state/fact has one named owner;
- [x] lexical/access/layout identities are explicitly separated;
- [x] no planned fix creates a name-based or ambient duplicate authority;
- [x] Resource/stream architecture reuses existing forward-looking design instead of inventing a parallel model;
- [x] performance findings remain separated from correctness.

Luna executability:

- [x] fixed/flexible/verify-first decisions are explicit;
- [x] each task names files/symbols and forbidden shortcuts;
- [x] STOP/CONSULT triggers are objective;
- [x] semantic debugging is bounded;
- [x] mechanical drift is permitted without architectural improvisation.

Testing:

- [x] coverage is organized around semantic risks, not task count;
- [x] hostile cases defeat the obvious shortcuts;
- [x] existing test homes are reused;
- [x] broad workspace verification is explicitly deferred;
- [x] each gate states what it proves.

Lifecycle:

- [x] canonical C2 checkpoint creation is part of execution;
- [x] checkpoint updates occur at durable gates;
- [x] walkthrough and handoff are required;
- [x] commit groups are coherent and conditional on authorization;
- [x] no plan-specific competing state file is introduced.

No implementation or test result is claimed by this plan. All PASS/COMPLETE states must be established by the executing agent.
