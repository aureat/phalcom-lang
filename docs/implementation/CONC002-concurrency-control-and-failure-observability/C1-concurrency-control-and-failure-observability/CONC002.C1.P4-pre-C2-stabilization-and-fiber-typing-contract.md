---
id: CONC002.C1.P4
program: CONC002
checkpoint: CONC002.C1
kind: implementation-plan
status: COMPLETE
completion: COMPLETE
verification: VERIFIED
prepared: 2026-09-12
baseline_branch: main
baseline_head: 2db7e3780772e847a178918ede239d91e0bcc5fd
requires: CONC002.C1.P3 COMPLETE
---

# CONC002.C1.P4 — Pre-C2 stabilization and Fiber typing contract

## 0. Mission and authority

This plan replaces the old “remaining Fiber and native activation work” P4. Native callback re-entry, `on`/`ensure`, call-site failure injection, and VM-owned cleanup/control activations are **removed from P4** and owned solely by `CONC002.C2.P1-R1`. Manual generator-await composition is moved to C2.P2. Cancellation/structured concurrency is C5. Channels/select is C6.

P4 has one job: leave C1 in a stable, verified, semantically honest state that C2 can safely rewrite underneath. It closes the Fiber typing design enough to forbid unsound generic shapes, proves the current runtime behaviors that constrain the future type, inventories the exact C2 native-boundary handoff, and freezes the public C1 Fiber contract until C2.P2 can implement the final manual-coroutine/executor stop protocol.

Prepared against remote `main` at `2db7e3780772e847a178918ede239d91e0bcc5fd`. Execution must rebase onto the HEAD produced by P3. Remote inspection does not expose local uncommitted state; record local branch/HEAD/worktree before editing.

### Primary completion claim

After P4, C1 has no remaining implementation work that belongs to C2 or later. The current Fiber surface is frozen and adversarially documented, `Fiber<I,R>` is formally rejected, the preferred long-lived nominal target is terminal-result-only `Fiber<R>`, the repository's actual type machinery and erasure constraints are measured rather than guessed, final public generic adoption is deliberately deferred to C2.P2, and C2.P1-R1 receives an exact, current native-reentry and runtime-invariant handoff.

### Fiber typing decision to ratify

The old proposal:
```text
Fiber<I, R>
```
where `I` is “resume input” and `R` is terminal success is unsound as a model of the current protocol.

Current `call(_)` uses its argument for two different state-dependent operations:

```text
New Fiber:
    call(x) -> x participates in entry Function argument binding

Yielded Fiber:
    call(x) -> x becomes the result of the suspended Fiber.yield(...)
```

Those types are not required to be equal. A Fiber may also have multiple yield sites with different yield/resume types. Therefore neither `Fiber<I,R>` nor a simple homogeneous `Fiber<Y,S,R>` precisely models arbitrary current manual coroutines.

A typestate Fiber is also inappropriate as the immediate public solution because Fibers are aliased ordinary heap objects; resuming one alias cannot safely rewrite the static type of every other alias without a much stronger linear/refinement system.

The only obvious lifetime-stable type property is terminal successful return. The preferred nominal target is therefore:

```text
Fiber<R>
```

where `R` means only “successful terminal result type.” It **does not** imply `call(...) -> R`. Manual transfer operations remain Dynamic until C2.P2 has an explicit stop/consumer protocol capable of distinguishing yield from terminal return/failure.

P4 does not force `Fiber<R>` into the canonical Universe merely because the target is attractive. It first proves that existing callable/generic machinery can infer and erase it without narrowing `Fiber.new`, fabricating `Fiber<Dynamic>` subtyping, or breaking `Fiber.current`/heterogeneous scheduler internals. Final public adoption is owned by C2.P2 unless all feasibility conditions below are satisfied without new type-system architecture and the user explicitly promotes it.

### Sources of truth

| Concern | Source of truth | Must remain distinct from |
|---|---|---|
| Fiber runtime identity | `ObjRef` → `Object::Fiber(FiberObject)` | applied semantic `Fiber<R>` identity |
| Entry call shape | Fiber entry `Function` / canonical callable domain | later `yield` resume input |
| Manual stop payload | dynamic coroutine transfer at current yield/return boundary | terminal `R` alone |
| Terminal success | `FiberObject::result` when status `Done` | yielded data, failed Error |
| Terminal failure | `FiberStatus::Failed` + captured Error | returned/yielded `Error` data |
| Scheduler queue | erased runtime Fiber handles | source-level generic covariance/erasure |
| Current Fiber | runtime `VM.current` | fake `Fiber<Dynamic>` nominal supertype |
| Future parking | exact `Parked(generation)` episode | manual coroutine consumer |
| Native re-entry safety | `native_reentry_depth` / Fiber floor checks | public async coloring |

### Invariants to preserve

1. `Fiber.call` / `try` are mixed manual transfer operations; do not claim they always return terminal `R`.
2. `Fiber.yield` result type is not derivable from Fiber entry parameter type.
3. `Error` may be successfully yielded or returned; failed Fiber is distinguished by terminal status/error slot.
4. `None`/Unit payloads are ordinary values.
5. `Fiber.current` may refer to a Fiber with any terminal result type.
6. scheduler queues are heterogeneous and runtime-erased; source typing must not invent subtyping to explain the queue.
7. scheduled user-yield remains rejected until a consumer exists.
8. a manually consumed Fiber may not await a pending Future under C1; C2.P2 owns that composition.
9. native-depth guards remain until C2.P1-R1 migrates the owning host continuation.
10. C1.P4 does not implement cancellation, queue-admission revocation, tasks/scopes, channels, select, reactor registrations, or VM control activations.

---

# 1. Checkpoint map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| C0 — Final C1 baseline and scope excision | 1–3 | P4 starts from completed P3; obsolete P4 ownership is removed; exact C2/C5/C6 boundaries are recorded | local HEAD/worktree; P3 evidence review; scope/deletion ledger | Fiber protocol hostility → C1; native handoff → C3 |
| C1 — Manual Fiber protocol truth | 4–6 | Entry arguments, yielded values, resume values, terminal values, and failure are demonstrated as distinct protocol positions | new/existing language fixtures; current `call/try/yield` runtime controls | final typed stop API → C2.P2 |
| C2 — Fiber generic feasibility and decision | 7–10 | terminal-only `Fiber<R>` is assessed against actual callable generics, native generic identity, `Fiber.current`, scheduler erasure, and reflection; unsound shapes are rejected | focused semantic/parser/type-syntax probes; inventory of bare Fiber consumers | final public generic implementation → C2.P2 unless promotion gate satisfied |
| C3 — Pre-C2 native-boundary handoff | 11–13 | every remaining user-code-driving native path is classified as C2 migration target, guarded residual, or already VM-visible | production-call search; current negative native-boundary controls; C2-R1 cross-check | actual migration → C2.P1-R1 |
| C4 — Contract freeze and C1 closure | 14–17 | canonical Fiber surface and docs are frozen for C2; unrelated typing findings are moved out; C2 handoff is exact | concurrency/semantic gates; documentation negative searches; state-file closure | C2+ implementation |

---

# 2. Checkpoint C0 — Final C1 baseline and scope excision

Tasks:
- Task 1 — rebase P4 onto completed P3;
- Task 2 — delete old P4 ownership of native activation/generator/cancellation;
- Task 3 — establish the downstream ownership matrix.

Why this is a checkpoint:
The old P4 is structurally wrong for the new program order. Implementation must not start until it is impossible for two agents to build the same control stack or for C1 to absorb C5/C6 policy work.

Entry conditions:
- P3 COMPLETE;
- P1/P2 statuses corrected;
- final P3 evidence ledger available.

Primary working set:
- this P4 document;
- C1 `CHECKPOINT.md`;
- completed P3 architecture/state;
- `CONC002.C2.P1-R1-native-suspension-and-reactor-groundwork.md` (inspection only; C2 owns its code changes);
- CONC002 `PROGRAM.md` / `STATUS.md`.

Semantic contract:
- P4 contains only pre-C2 work;
- C2.P1-R1 is the sole owner of VM-owned native control activations and parent-failure injection;
- C2.P2 owns manual consumer/executor composition and final Fiber stop/type implementation;
- C5 owns cancellation/structured concurrency;
- C6 owns channels/select.

### Task 1 — Rebase onto the P3-produced HEAD

Risk:
- Semantic: LOW
- Implementation fanout: documentation/state

Commands:
```bash
git rev-parse HEAD
git status --short
git log -8 --oneline --decorate
```

Record exact HEAD in P4 state and update source anchors mechanically if P3 moved files/symbols. Do not repeat P3's full investigation.

### Task 2 — Excise obsolete P4 scope

Risk:
- Semantic: MEDIUM
- Implementation fanout: documentation graph

Delete as executable P4 work:
- Fiber-owned handler/cleanup/control activation implementation;
- `on`/`ensure` migration;
- ReturnNonLocal control routing;
- parent Call failure injection;
- generator await implementation;
- cancellation request/outcome/cleanup design;
- queue-admission generation for cancellation.

Replace with explicit owner links:
```text
native/control continuation work -> C2.P1-R1
manual generator-await/consumer work -> C2.P2
cancellation/structured concurrency -> C5
channels/select -> C6
```

Negative gate:
P4 must contain no task instructing edits to `vm/control.rs`, `block_on`, `block_ensure`, cancellation state, TaskScope, Channel, or select.

### Task 3 — Establish downstream ownership matrix

Add a single authoritative table to P4 and C1 checkpoint state. Do not duplicate implementation designs from downstream plans.

Checkpoint completion:
- [x] P4 rebased;
- [x] duplicate C2 ownership removed;
- [x] C5/C6 explicitly out of scope;
- [x] next checkpoints can proceed without ownership ambiguity.

Suggested commit group:
`docs(concurrency): narrow P4 to pre-C2 stabilization`

---

# 3. Checkpoint C1 — Manual Fiber protocol truth

Tasks:
- Task 4 — add entry-vs-resume hostile behavior fixture;
- Task 5 — add heterogeneous multi-yield protocol fixture;
- Task 6 — preserve Error/None/Unit data versus terminal failure distinction.

Why this is a checkpoint:
Fiber generic design must start from executable behavior. The easiest bad type design (`Fiber<I,R>`) looks plausible until tests demonstrate that first-entry input and post-yield resume input can differ, and that one Fiber can have multiple distinct yield/resume pairs.

Entry conditions:
- C0 COMPLETE;
- existing manual `call`/`try`/`yield` behavior is green.

Working set — primary:
- `phalcom-core/core/universe/src/concurrency/fiber.ph` — current API declarations (inspect; do not genericize yet);
- `phalcom-core/src/primitive/fiber.rs` — `fiber_resume`/`fiber_yield` semantics;
- existing concurrency language-corpus source directory/harness;
- `docs/spec/current/concurrency.md` — behavioral contract.

Out of scope:
- new compiler/type syntax;
- explicit public `FiberStop` enum;
- C2 consumer/executor relation;
- native suspension migration.

Semantic contract:
- first entry argument shape belongs to the entry Function;
- a later `call` value belongs to the suspended yield continuation;
- those positions can have different runtime value classes;
- different yield sites in one Fiber may use different yield/resume value classes;
- terminal result/failure is orthogonal to all of the above.

Hostile case 1 — entry input differs from resume input:
```phalcom
const f = Fiber.new(|initial| {
  Assert.equal(initial, 41)
  const response = Fiber.yield("ready")
  Assert.equal(response, #continue)
  return true
})

Assert.equal(f.call(41), "ready")
Assert.equal(f.call(#continue), true)
```

Hostile case 2 — distinct yield sites:
```phalcom
const f = Fiber.new(|| {
  const first = Fiber.yield(1)
  Assert.equal(first, "one")
  const second = Fiber.yield(#confirm)
  Assert.equal(second, true)
  return 42
})

Assert.equal(f.call(), 1)
Assert.equal(f.call("one"), #confirm)
Assert.equal(f.call(true), 42)
```

These are runtime protocol controls. They deliberately do not claim static precision for the local variables yet.

### Task 4 — Entry-versus-resume hostile fixture

Risk:
- Semantic: HIGH (design evidence)
- Implementation fanout: local language fixture

Edit operations:
1. Locate the existing positive manual Fiber fixture registered under `corpus::concurrency`.
2. Add the first hostile case to that fixture or a sibling source registered by the same non-recursive harness.
3. Preserve current `call`/`try` semantics; this task should not require production code changes.
4. If it fails, classify whether runtime semantics differ from `concurrency.md`; do not change typing assumptions first.

Evidence:
```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --nocapture
```

### Task 5 — Heterogeneous multi-yield protocol fixture

Risk:
- Semantic: HIGH (invalidates homogeneous generic protocol shortcuts)
- Implementation fanout: local fixture

Add hostile case 2. The test proves why a simple `Fiber<Yield,Send,Return>` is only a widening approximation, not a precise ordered protocol.

Must not:
- add union-based generic parameters to Fiber merely to make the fixture typeable;
- introduce session/typestate machinery in C1.

### Task 6 — Data-versus-failure fixture

Risk:
- Semantic: MEDIUM
- Implementation fanout: local fixture

Ensure permanent controls distinguish:
- `Fiber.yield(Error.new("data"))` from terminal failure;
- `return Error.new("data")` from terminal failure;
- `try` terminal failure returning captured Error data from a successfully returned Error by checking `isDone`/`error`;
- `None` and `()` as normal transfer/return values.

Reuse existing coverage where present; add only missing rows.

Checkpoint completion:
- [x] entry/resume types demonstrated independent;
- [x] multiple yield sites demonstrated heterogeneous;
- [x] data-vs-failure cases green;
- [x] no production type claim was added prematurely.

Suggested commit group:
`test(concurrency): pin manual Fiber transfer protocol before C2`

---

# 4. Checkpoint C2 — Fiber generic feasibility and decision

Tasks:
- Task 7 — inventory every source/native use of `Fiber` typing;
- Task 8 — prove or refute sound terminal-result inference using existing callable-type machinery;
- Task 9 — resolve `Fiber.current` and heterogeneous-erasure semantics;
- Task 10 — ratify the terminal-only generic target and defer final public adoption to C2.P2 unless the promotion gate is fully satisfied.

Why this is a checkpoint:
Generic Fiber is high-risk shared type semantics. A five-line Universe declaration can create unsaturated constructor diagnostics, false assignability, broken reflection identity, or narrowed callable entry behavior. All four dimensions must be understood before changing the canonical class.

Entry conditions:
- C1 COMPLETE;
- current Fiber remains non-generic during investigation.

Primary working set:
- `phalcom-core/core/universe/src/concurrency/fiber.ph`
- `phalcom-semantic/tests/semantic/capabilities/generics.rs`
- `phalcom-semantic/src/types/{store,relation,substitution,instantiation}.rs` only for inspection unless tests reveal an owned defect;
- `phalcom-ast/src/parser.rs` type-annotation parsing;
- `phalcom-type-syntax/src/lib.rs` native callable/type metadata parsing;
- `docs/spec/collections/07-callable-domains.md` — ratified callable-domain semantics;
- native surface generator/macro metadata consumers only if canonical Fiber native signatures require them.

Secondary:
- reflection/type-descriptor products for applied generic identity;
- every canonical Universe declaration containing bare `Fiber`.

Out of scope:
- redesign of callable-domain type system;
- higher-rank existential types;
- linear/affine typestate;
- generic runtime class specialization/storage;
- C2 stop protocol implementation.

## 4.1 Required design conclusion

The plan must ratify these points:

1. `Fiber<I,R>` is rejected: entry input and post-yield resume input are distinct contracts.
2. homogeneous `Fiber<Y,S,R>` cannot precisely encode multiple differently typed yield/resume sites without losing sequencing.
3. typestate `Fiber<State>` is not adopted because aliased heap objects cannot have all aliases' static state rewritten after resume.
4. terminal successful result is the only obvious lifetime-stable generic property.
5. preferred nominal target: `Fiber<R>`.
6. `call`/`try`/`yield` remain Dynamic under that target until an explicit stop protocol exists.
7. entry callable domain remains separate from nominal Fiber identity.

### Task 7 — Exhaustive Fiber type-consumer inventory

Purpose:
Find every place that would be affected by parameterizing the canonical class.

Risk:
- Semantic: HIGH
- Implementation fanout: cross-crate inspection

Commands:
```bash
rg -n '\bFiber\b' phalcom-core/core/universe/src phalcom-semantic phalcom-native-meta phalcom-native-surface-gen phalcom-native-macros docs/spec/current
rg -n 'Option<Fiber>|List<Fiber>|-> Fiber|: Fiber|Fiber<' phalcom-core/core/universe/src phalcom-semantic
```

Classify every production source occurrence:
- exact user/source `Fiber<R>` candidate;
- intentionally erased runtime/internal handle;
- `Fiber.current` existential problem;
- scheduler/wake internal surface;
- documentation-only.

Do not “fix” bare internal handles by mechanically writing `Fiber<Dynamic>`.

Deliverable:
An inventory table in the P4 state file with intended final treatment for every canonical/native signature.

### Task 8 — Callable return inference feasibility

Purpose:
Determine whether existing type machinery can infer `R` from an arbitrary entry Function **without changing which entry call shapes Fiber supports**.

Risk:
- Semantic: HIGH
- Implementation fanout: semantic/parser/type-syntax

Repository facts to respect:
- callable types are `ArgumentPackType -> ResultType`;
- generic forwarding can theoretically quantify a pack `P` as `(***P,) -> R`;
- `(...) -> R` denotes a callable that accepts any well-formed pack, not an existential “some pack returning R”; therefore it is **not** a sound drop-in type for `Fiber.new`.

INVESTIGATE-BEFORE-EDIT sequence:
1. Prove whether source semantic analysis currently parses/forms a callable generic pack such as `(***P,) -> R`, rather than relying only on design docs.
2. Prove whether generic inference can infer both `P` and `R` from a closure/method Function value.
3. Prove whether a constructor/class-side method on a generic native `Fiber<R>` can use that evidence without requiring the caller to write `Fiber<Int>` explicitly.
4. Test zero-arg, one-positional, multiple-positional, and labeled entry callables because current Fiber entry is shape-aware.
5. If any step requires implementing missing pack-kind/type-lambda machinery, stop: that work belongs to the type-system program, not C1.

Prototype tests belong in semantic test code, not production declarations. Example target shape **only if existing grammar supports it**:
```phalcom
class ProbeFiber<R> {
  @class
  new<P: Tuple>(_ body: (***P,) -> R) -> ProbeFiber<R> { ... }
}
```

Do not copy this into Universe until the semantic test proves it.

Promotion gate for pre-C2 `Fiber<R>` implementation:
- arbitrary current entry shapes remain accepted;
- `R` is inferred without explicit application in normal cases;
- no new type-system architecture is required;
- no unsaturated generic diagnostics appear;
- `Fiber.current` and internal heterogeneous uses have a principled erasure/existential presentation;
- reflection identity remains canonical;
- full semantic suite remains green.

If any item fails, final public adoption remains C2.P2. This is the expected conservative outcome.

### Task 9 — Resolve current/queue erasure requirements

Purpose:
Prevent a generic surface from fabricating an invalid `Fiber<Dynamic>` hierarchy.

Risk:
- Semantic: HIGH
- Implementation fanout: semantic + native declarations

Questions that must be answered and recorded:
- What source type can `Fiber.current` truthfully return when current Fiber has unknown terminal `R`?
- Does Phalcom currently have an existential/wildcard applied-type facility suitable for `exists R. Fiber<R>`?
- If not, is an erased bare native class reference a supported internal-only signature distinct from source generic types?
- How are heterogeneous ready-queue handles represented? (Runtime `ObjRef` is fine; source metadata still needs truthful presentation.)
- Does reflection expose the applied `Fiber<R>` or only root class identity for native Fiber instances? Applied generics must not imply new runtime classes.

Must not:
- assert `Fiber<Int> <: Fiber<Dynamic>` unless generic variance/gradual semantics explicitly proves it;
- use `Dynamic` as a wildcard by convenience;
- allocate runtime-specialized Fiber classes for applied types.

### Task 10 — Ratify target and C2.P2 handoff

The normal P4 outcome is:
```text
Target nominal type: Fiber<R>
Meaning of R: terminal successful return only
Manual call/try/yield transfer types: Dynamic
Entry argument domain: property of entry Function, not Fiber generic identity
Current/queue internal handling: erased runtime capability, not fake nominal subtyping
Final implementation: C2.P2 after explicit consumer/executor stop protocol
```

If the full promotion gate unexpectedly passes with no type-system expansion, record the evidence and request an explicit design decision before parameterizing canonical Fiber in C1. Do not silently promote because it is mechanically possible.

Required evidence:
```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic <fiber_callable_pack_probe_tests> -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic
```

Checkpoint completion:
- [x] all Fiber type consumers classified;
- [x] arbitrary-domain inference feasibility measured;
- [x] `(...) -> R` misuse explicitly rejected;
- [x] current/queue erasure problem explicitly resolved or deferred;
- [x] terminal-only target recorded;
- [x] no unsound `Fiber<I,R>` implementation remains planned.

Suggested commit group:
`docs/test(concurrency): establish terminal-only Fiber typing contract`

---

# 5. Checkpoint C3 — Pre-C2 native-boundary handoff

Tasks:
- Task 11 — re-inventory all user-code-driving native re-entry;
- Task 12 — classify migration vs residual guarded host continuations;
- Task 13 — verify current guard controls and hand the exact inventory to C2-R1.

Why this is a checkpoint:
C2-P1-R1 should not spend its first implementation pass rediscovering what P4 already knows, but P4 must not implement the C2 solution. The correct deliverable is an exact current inventory plus preserved safety controls.

Entry conditions:
- C2 COMPLETE;
- no native-depth guard removed.

Primary working set — inspect only:
- `phalcom-core/src/primitive/block.rs` — `block_call`, `block_on`, `block_ensure`, loop fallback;
- Bool/Option primitives that call arbitrary Phalcom blocks;
- `phalcom-core/src/vm/send.rs` — `activate_function`, host `send_dynamic`, forwarding gateways;
- `phalcom-core/src/vm/dispatch.rs` — reverse-ordering validation/user sends;
- Map/Set hash/equality callback paths;
- rendering/`toString` callback paths;
- reflective typing/native orchestration paths;
- current C2-R1 source-of-truth/disposition table.

Semantic contract:
Every production native path that can run arbitrary Phalcom code has one disposition:
1. already VM-visible and suspension-safe;
2. C2 migration target because it represents language/control flow;
3. deliberately retained synchronous host algorithm guarded against Fiber switch.

### Task 11 — Production re-entry search

Commands:
```bash
rg -n 'block_call\(|send_dynamic\(|invoke_method_object\(|activate_function\(' phalcom-core/src
rg -n 'native_reentry_depth|floor_depth|CannotYieldAcrossNativeFrame' phalcom-core/src
```

For each production hit record:
- caller;
- Rust state held across user-code execution;
- whether continuation state is language-level or native-algorithm state;
- current guard;
- C2 disposition.

Known starting classifications to verify, not assume:
- ordinary Function and source-level `Iterable.each`: already VM-visible;
- `on` / `ensure`: C2 migration;
- native `whileTrue`, Bool lazy/conditional callbacks, `Option.match`: likely C2 migration;
- `Error.raise -> message`, reversed ordering validation: likely C2 migration;
- Map/Set probing/hash/equality: retained guarded host algorithm unless C2 proves a bounded better design;
- rendering/host embedder/reflection orchestration: residual guard unless explicitly migrated.

### Task 12 — Classify without designing a second control stack

Risk:
- Semantic: HIGH
- Implementation fanout: cross-runtime inspection

Must not:
- add `ControlActivation` or router code;
- remove `native_reentry_depth` checks;
- make host algorithms resumable;
- rewrite collection traversal already safe in Phalcom;
- add an async/native continuation ABI.

Output:
A handoff table suitable for direct insertion/rebase into C2-R1.

### Task 13 — Verify current refusal is local and non-mutating

Use existing negative fixtures for `CannotYieldAcrossNativeFrame` where they still represent residual re-entry. Add one minimal control only if the existing suite does not prove that park/yield refusal occurs before waiter/ownership mutation.

Evidence:
```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib scheduler_tests -- --nocapture
```

Proves:
- current safety guard still protects pre-C2 host continuations;
- ticketed park/wake remains intact before C2 changes native control.

Checkpoint completion:
- [x] exhaustive direct re-entry inventory recorded;
- [x] each hit has exactly one disposition;
- [x] current guard tests pass;
- [x] no C2 mechanism implemented in P4.

Suggested commit group:
`docs(concurrency): freeze native re-entry handoff for C2`

---

# 6. Checkpoint C4 — Contract freeze and C1 closure

Tasks:
- Task 14 — freeze current public Fiber surface through C2.P1-R1;
- Task 15 — move unrelated typing findings out of C1;
- Task 16 — update canonical docs and P4/C2 handoff;
- Task 17 — run final P4/C1 gates and close state.

Why this is a checkpoint:
C2 changes high-risk VM control flow. P4 must prevent simultaneous public API/type experimentation from obscuring failures. The result is a narrow, stable contract C2 may preserve while changing implementation underneath.

### Task 14 — Freeze public Fiber surface

Current C1 surface to preserve through C2.P1-R1:
```phalcom
@native
class Fiber is Object {
  @class @native new(_ body: Function) -> Fiber
  @native call() -> Dynamic
  @native call(_ value: Dynamic) -> Dynamic
  @native try() -> Dynamic
  @native try(_ value: Dynamic) -> Dynamic
  @class @native yield() -> Dynamic
  @class @native yield(_ value: Dynamic) -> Dynamic
  @class @native current -> Fiber
  @class @native abort(_ error: Error) -> Never
  @native isDone -> Bool
  @native isRoot -> Bool
  @native error -> Option<Error>
  // internal scheduler/park/observer seams unchanged
}
```

This is a **contract freeze**, not a claim that it is the final C2.P2 type surface.

Required notes:
- `call/try` remain Dynamic because they mix user yield and terminal return (and `try` terminal failure-as-data);
- `yield` remains Dynamic because resume input is suspension-site-specific;
- pending await from a manual chain remains rejected;
- scheduler-owned yield remains rejected;
- `abort` is Raise/Never, not cancellation.

Do not add public `result -> Option<R>` until the generic/existential decision is implemented. The internal `_$terminalValue` remains an implementation seam for Future completion observers.

### Task 15 — Move unrelated type-system findings out

The old P4 mentioned:
- raw generic type-form proper-type panic;
- executable applied type-form lowering to Nil;
- general receiver-dependent `Self` proof gaps.

Create or update references in the owning type/compiler program and replace P4 implementation instructions with links/issue IDs. These are not C2 blockers unless the C2 implementation directly reproduces one.

Must not:
- fix generic type-form execution inside P4;
- weaken `TypeStore` proper-type assertions;
- erase generic reflection identity;
- expand C1 into the general `Self` checker.

### Task 16 — Update docs and C2 handoff

Update:
- C1 `CHECKPOINT.md`: P4 is final C1 plan;
- P4 state: record terminal-only Fiber target and deferred final implementation owner C2.P2;
- `docs/spec/current/concurrency.md`: do **not** publish `Fiber<R>` yet unless promotion was explicitly approved and implemented; record the current Dynamic manual transfer truth;
- C2-R1 front matter/baseline/overlap language after P4 lands: remove claims that P4 §2 may independently implement native control; C2-R1 is sole owner.

The C2 entry contract must state that it may assume:
```text
truthful Fiber lifecycle
exclusive scheduler admission
exact park generations
wake = enqueue, not execution
durable terminal observer
stable Future<T>
scheduled user-yield rejection
manual pending-await restriction
native-depth guards intact
linked Call failure still transitional
public Fiber surface frozen through C2.P1-R1
```

### Task 17 — Final gates

Focused behavioral/type gates:
```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib scheduler_tests -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
cargo fmt --all -- --check
```

Broader C1 delivery gates, only after focused evidence:
```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
```

Workspace test:
```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets
```
Run at final delivery if useful; if the historical broad REPL import/export baseline still fails unchanged, classify BASELINE with exact comparator evidence rather than expanding P4.

Final negative gates:
```bash
rg -n 'Fiber<I, *R>|Fiber<I,R>' docs/implementation/CONC002-concurrency-control-and-failure-observability/C1-concurrency-control-and-failure-observability
rg -n 'cancellation|TaskScope|Channel|select' <P4-file>
rg -n 'ControlActivation|block_ensure.*migrate|block_on.*migrate' <P4-file>
```
Expected:
- no normative `Fiber<I,R>` target;
- cancellation/channels/select appear only in explicit out-of-scope/owner statements;
- no C2 implementation task remains in P4.

Checkpoint completion:
- [x] public C1 Fiber contract frozen;
- [x] terminal-only generic target recorded without unsound implementation;
- [x] unrelated typing findings reassigned;
- [x] C2-R1 receives current inventory and invariants;
- [x] focused P4 gates pass;
- [x] broad compatibility gates pass or are correctly classified;
- [x] C1 status becomes COMPLETE;
- [x] next resume action is C2.P1-R1.

Suggested commit groups:
1. `test(concurrency): pin Fiber manual transfer protocol`
2. `docs(type/concurrency): ratify terminal-only Fiber generic target`
3. `docs(concurrency): freeze C2 native-boundary handoff`
4. `docs(concurrency): close C1 before native suspension work`

---

# 7. Tempting wrong fixes — explicitly forbidden

1. **Do not implement `Fiber<I,R>`** by treating `I` as both entry and post-yield resume input.
2. **Do not implement `Fiber<Y,S,R>`** and claim it precisely represents ordered heterogeneous yield sites.
3. **Do not model arbitrary current Fiber as `Fiber<Dynamic>`** unless Phalcom's generic/gradual relation explicitly establishes that semantics.
4. **Do not use `(...) -> R` for `Fiber.new`** merely because it mentions only the result type; by callable-domain law it means a callable accepting any pack, which would reject/narrow ordinary exact-domain Functions rather than existentially erase their domain.
5. **Do not restrict `Fiber.new` to zero-argument bodies** just to infer `R`; manual Fibers currently support first-entry arguments.
6. **Do not add overloads per arity** as a substitute for the canonical argument-pack model unless the callable/type program explicitly chooses that representation.
7. **Do not create runtime-specialized Fiber classes for `Fiber<R>`**; applied generic type identity does not imply new runtime class identity.
8. **Do not add session/linear typestate machinery in C1**.
9. **Do not expose a public `FiberStop` enum before C2.P2 owns stop/consumer semantics**.
10. **Do not remove native-depth guards**; C2 must first remove/represent the host continuation they protect.
11. **Do not reintroduce generator-await or cancellation tasks into P4**.

---

# 8. Failure protocol

If a Fiber generic probe or runtime fixture fails unexpectedly:

1. Record exact reproduction.
2. Trace the path from source annotation/call to parser → semantic type formation → generic inference/assignability, or from fixture → Fiber primitive → switch.
3. Find a passing comparator (for example, `Future<T>` generic inference or zero-arg callable typing).
4. Classify PRODUCT, FIXTURE, DEPENDENCY/PUBLICATION, BACKEND/HARNESS, BASELINE, or PLAN DRIFT.
5. State narrow repair boundary.
6. Do not patch parser/type system/runtime outside that boundary merely to make the probe pass.

Escalate immediately if:
- implementing terminal-only Fiber typing requires new higher-rank/existential type machinery;
- generic callable packs are not implemented even though design specs describe them;
- parameterizing native Fiber changes runtime class identity;
- `Fiber.current` cannot be represented truthfully without a new type feature;
- C2-R1 has already landed materially and changes the pre-C2 boundary.

A failed feasibility probe does **not** make P4 incomplete if its purpose was to decide ownership. It becomes evidence that final `Fiber<R>` implementation remains C2.P2/type-system-dependent. The semantic contract is “no unsound generic surface,” not “force a generic class before C2.”

---

# 9. State-file protocol

After each checkpoint record:

```md
## Established invariants
- ...

## Fiber typing decision
- terminal-only target: ...
- rejected shapes: ...
- current/queue erasure finding: ...
- implementation owner: ...

## Native-boundary handoff
| Path | Current owner/state | C2 disposition |

## Evidence ledger
| Checkpoint | Command | Result | Proves |

## Deferred gates
- ...

## Active incident
None.

## Next resume action
...
```

Do not record private scratch reasoning. Record claims, evidence, decisions, anchors, and rejected approaches.

---

# 10. Final evidence summary

| Checkpoint | Semantic contract | Evidence | Status |
|---|---|---|---|
| C0 | obsolete P4 ownership removed | scope/HEAD ledger | PASS |
| C1 | actual manual Fiber protocol pinned | concurrency hostile fixtures (`concurrency_fiber_protocol_hostile.ph`) | PASS |
| C2 | generic target/feasibility decided soundly | semantic/type probes (`generics::unsaturated_generic_constructors_are_cleanly_rejected_in_proper_type_positions`, `generics::generic_callable_return_inference_requires_exact_parameter_domain`) + census | PASS |
| C3 | exact native-boundary handoff prepared | production re-entry search + guard controls | PASS |
| C4 | C1 contract frozen and delivered | focused + broad gates, clippy, fmt, negative searches | PASS |

## Deferred-evidence audit

No deferred item may disappear. At completion each must be:
- executed;
- assigned to C2.P1-R1/C2.P2 with a concrete reason;
- assigned to the type-system program because repository support is genuinely missing;
- or recorded as a known baseline blocker.

## Known scope exclusions

- C2.P1-R1 implementation;
- manual coroutine/executor composition and final stop protocol (C2.P2);
- reactor (C3);
- concurrency library completion (C4);
- cancellation/structured concurrency (C5);
- channels/select (C6);
- parallel execution/preemption/memory model;
- general generic type-form and `Self` repair.

## Release-complete criteria for P4 / C1

P4 and C1 are complete only when:
- C0–C4 are COMPLETE;
- adversarial manual Fiber protocol fixtures pass;
- no plan or canonical doc treats entry input and resume input as one Fiber generic;
- the terminal-only `Fiber<R>` target and its implementation owner are explicit;
- no fake `Fiber<Dynamic>` erasure relation is introduced;
- every remaining native re-entry path has exactly one C2 disposition;
- native safety guards remain effective;
- all focused semantic/runtime evidence passes;
- documentation/status/state files agree;
- no C5/C6 implementation has leaked into C1;
- no unresolved C1 incident remains;
- the next implementation action is unambiguously `CONC002.C2.P1-R1`.
