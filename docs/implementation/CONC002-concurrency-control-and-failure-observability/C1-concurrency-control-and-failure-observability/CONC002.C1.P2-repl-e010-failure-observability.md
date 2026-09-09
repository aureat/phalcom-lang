---
id: CONC002.C1.P2
category: CONC
program: CONC002
checkpoint: CONC002.C1
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on: [CONC002.C1.P1]
follows: CONC002.C1.P1
supersedes: null
deferred_reason: null
---

# CONC002.C1.P2 — REPL/E010 failure observability

## Phalcom REPL Export Materialization + E010 Scheduler Failure Observability
## Patch-Grade Implementation Plan

**Repository:** `aureat/phalcom-lang`  
**Target branch:** `codex/concurrency-control-remediation`  
**Repository state inspected:** branch head `99e28aeacef31b44882ea02668411035cf6282ee`  
**Scope:** two bounded correctness patches only  
**Primary goals:**
1. Repair canonical Universe runtime export materialization and the REPL selective-import consumer path.
2. Implement E010 as explicit observability for unowned scheduler failures without weakening scheduler failure isolation.

---

# 0. Executive implementation order

Implement these as **two separate patches**.

This record is filed under `CONC002.C1.P2` for Patch B, the E010 scheduler
failure-observability work. Patch A remains in this source document as module
and REPL companion context; its implementation ownership is outside `CONC002`.

| Order | Patch | Problem | Primary outcome |
|---:|---|---|---|
| 1 | Patch A | Canonical Universe linked exports are not projected into runtime `ModuleObject.exports`; REPL selective imports additionally assume an export is a local slot | Runtime Universe export tables become faithful to linked interfaces; all three REPL import regressions pass |
| 2 | Patch B | Scheduler-mode Fiber failures are isolated correctly but an unowned failed scheduled Fiber has no reporting owner | Detached scheduled failures become visible without aborting sibling scheduled work |

Do **not** combine the two into a single implementation commit. They touch different architectural authorities and require independent regression gates.

---

# 1. Global invariants

Every implementation step must preserve the following invariants.

## 1.1 Module/runtime authority invariants

### MOD-RUNTIME-01 — Linked interfaces own public export identity

The authoritative chain is:

```text
source
  ↓
UnlinkedModuleInterface
  ↓
LinkedModuleInterface
  ↓
RuntimeExportRef
```

Runtime export tables must be a projection of linked interfaces. Native metadata may provide bootstrap implementation details, but must not become an independent export authority.

### MOD-RUNTIME-02 — Re-export identity is preserved

A façade export may target a binding in a different module:

```text
universe::PackageInfo
    ↓
universe.reflection.package_info::PackageInfo
```

The runtime export must preserve that target module and slot. It must not manufacture a local façade slot merely because the public export name is visible there.

### MOD-RUNTIME-03 — `expose` is not `export`

`expose .child` controls path visibility only.

It must never imply:

- a local runtime binding;
- a runtime export;
- a selective-import binding;
- child-module fallback for `from package import X`.

Existing negative tests for exposed-but-not-exported children must remain green.

### MOD-RUNTIME-04 — Universe bootstrap does not own a second export semantics

Canonical Universe bootstrap may allocate modules early and install native values early, but the final public export table must be materialized from the same linked interface model used by ordinary compiled programs.

---

## 1.2 Concurrency invariants

### CONC-ERR-01 — Scheduler failure isolation remains intact

If scheduled Fiber A fails and scheduled Fiber B is runnable:

```text
A fails
B still runs
```

A scheduler-mode failure must not be re-raised through the root merely to make it visible.

### CONC-ERR-02 — Owned terminal failures are not reported as unhandled

A Fiber with a durable completion observer has an owner.

Examples:

- `Future.async` action Fiber;
- pending `then` / `map` / `catch` callback Fiber;
- any future internal Task/TaskGroup owner.

These failures must continue through their completion observer and must not also enter the unhandled scheduler-failure channel.

### CONC-ERR-03 — Unowned scheduler failure is reported once

A scheduler-owned Fiber that terminates `Failed` without a completion owner becomes an **unhandled scheduled failure**.

It must:

- remain terminal;
- retain its captured `Error`;
- not abort siblings;
- be reported exactly once.

### CONC-ERR-04 — Raw scheduler resume values are not the observability protocol

Do not infer terminal failure by inspecting the return value of `_$resumeScheduled()`.

Terminal classification must happen where the VM already knows:

- terminal Fiber status;
- terminal error value;
- resume mode;
- whether a completion observer existed.

### CONC-ERR-05 — Reporting code does not run inline at the Fiber floor

Fiber-floor terminalization may classify and enqueue a report record, but must not execute arbitrary Phalcom callbacks or logging hooks inline while unwinding/switching.

---

# 2. Non-goals

This plan does **not** include:

- changing import syntax;
- redesigning `ModuleObject`;
- redesigning the linker;
- changing `expose` semantics;
- changing public `System.schedule` ownership semantics beyond error observability;
- introducing `Task`, `TaskGroup`, structured concurrency, cancellation, supervision trees, or nurseries;
- changing `Fiber#call`, `Fiber#try`, parking, ticketed wake, or completion-observer semantics;
- changing `Future.async`, `await`, `then`, `map`, or `catch` completion ownership;
- lifting `CannotYieldAcrossNativeFrame`;
- changing scheduler fairness;
- making `Fiber#error` mark a failure as observed;
- turning detached scheduler failure into root exception propagation.

If any of these becomes necessary to make the patch work, stop and re-evaluate the implementation instead of widening scope.

---

# 3. Repository facts this plan relies on

At the inspected branch head:

1. `VM::new()` constructs the canonical Universe program, installs the native runtime, prepares source bindings, compiles and executes canonical Universe modules, then finishes bootstrap.
2. `VM::materialize_program` already contains the correct ordinary-program runtime export-table projection in its Phase 5.
3. `initialize_canonical_universe` eagerly allocates canonical Universe modules and `install_universe_native_bindings` installs native bindings plus direct root child-package exports.
4. The canonical builtin interface builder already gives root convenience exports canonical declaration targets rather than synthetic root declaration ownership.
5. `ModuleExecutionContext::ensure_module_materialized` returns immediately for already-registered canonical Universe modules.
6. REPL selective import checks `target_obj.exports`, but then reconstructs a `BindingRef` using `target_obj.slot_of(public_name)`.
7. `FiberResumeMode::Scheduler` isolates terminal failure and delivers the captured error value back to the scheduler resumer.
8. `Future.async` and suspending continuation Fibers already use durable completion observers.
9. Existing regression behavior requires a failed scheduled Fiber not to abort later scheduled work.

These facts must be rechecked if implementation begins from a materially newer revision.

---

# Patch A — Canonical Universe Runtime Export Materialization + REPL Import Repair

# 4. Patch A objective

Repair the mismatch:

```text
static linked Universe exports     correct
runtime Universe Module.exports    incomplete
```

and remove the REPL assumption that every exported binding lives in a local slot on the exported-from module.

After Patch A:

```phalcom
from universe.errors.unsupported import unsupported
```

must succeed.

```phalcom
import universe.errors.unsupported
unsupported.unsupported
```

must resolve through module export dispatch.

```phalcom
from universe import PackageInfo
```

must resolve the root façade export to the canonical target binding in:

```text
universe.reflection.package_info::PackageInfo
```

without manufacturing a root-owned declaration or requiring a local root slot with the same public name.

---

# 5. Patch A current failure path

## 5.1 Canonical bootstrap path

Current conceptual path:

```text
canonical Universe source/index/link
        ↓
initialize_canonical_universe
        ↓
allocate every ModuleObject
        ↓
install_universe_native_bindings
        ├── native owner bindings
        ├── exported native-owner entries
        ├── direct root child packages
        └── context intrinsics
        ↓
run_universe_modules
        ↓
source globals/classes initialized
        ↓
NO NORMAL LINKED EXPORT MATERIALIZATION PHASE
```

Ordinary program materialization instead has:

```text
linked module interface
        ↓
VM::materialize_program Phase 5
        ↓
RuntimeExportRef::Binding / RuntimeExportRef::Module
        ↓
ModuleObject.exports
```

The missing operation is therefore not source execution. It is runtime export projection.

---

## 5.2 REPL selective-import path

Current conceptual behavior:

```text
target runtime module
        ↓
exports.contains(public_name) ?
        ↓ yes
target_module.slot_of(public_name)
        ↓
BindingRef { module: target_module, slot }
```

This is invalid for a re-export.

Legal runtime state:

```text
root.exports["PackageInfo"]
    =
RuntimeExportRef::Binding {
    module: universe.reflection.package_info,
    slot: <PackageInfo slot>
}
```

No root-local `PackageInfo` slot is required.

---

# 6. Patch A exact files and seams

Primary files:

- `phalcom-core/src/modules/materialize.rs`
- `phalcom-core/src/modules/builtin_materialize.rs`
- `phalcom-core/src/vm/bootstrap.rs`
- `phalcom-core/src/modules/context.rs`
- `phalcom-repl/tests/repl_import_bugs.rs`

Likely supporting tests:

- `phalcom-core/tests/core/modules/...`
- `phalcom-core/tests/core/object_model/...`
- `phalcom-modules/tests/...` only if a static-interface regression is needed to lock canonical export identity; do not modify linker semantics if current linked targets are already correct.

Potential cleanup target after behavior is green:

- duplicate Universe export-population code in `ModuleExecutionContext::ensure_module_materialized`.

Do not start by editing `phalcom-modules/src/builtin_interface.rs` or `phalcom-modules/src/linker.rs`. The current branch already has canonical root export targets and linked canonical declaration resolution. Patch A should consume that existing authority.

---

# 7. Patch A tests to add first

Add red tests before changing implementation.

## A-T01 — Runtime canonical source export exists

Create a core-level test that bootstraps `VM::new()`, resolves:

```text
universe.errors.unsupported
```

and verifies:

```text
module.exports["unsupported"]
```

exists as `RuntimeExportRef::Binding`.

Required assertions:

1. the runtime export is present;
2. the referenced slot contains the canonical `unsupported` value after bootstrap;
3. the export target is the canonical owner module;
4. no child-exposure fallback is involved.

---

## A-T02 — Runtime root canonical re-export points to real owner

Bootstrap `VM::new()` and inspect root export:

```text
universe.exports["PackageInfo"]
```

Assert:

```text
RuntimeExportRef::Binding {
    module: package_info_module,
    slot: package_info_slot
}
```

and:

```text
package_info_module != universe_root
```

unless future canonical ownership explicitly changes.

Also assert the bound value is the canonical `PackageInfo` class identity.

This test prevents a future "fix" that merely creates a root-local alias slot and loses canonical ownership.

---

## A-T03 — Runtime export names match linked canonical interface names

Add an integration invariant over canonical Universe modules:

```text
for each canonical Universe linked module:
    public linked export names
        ==
    runtime ModuleObject.export names
```

If there are legitimate runtime-only entries, enumerate them explicitly in the test rather than weakening comparison to a subset check.

Prefer exact equality for public exports.

---

## A-T04 — REPL source export selective import

Retain/fix:

```rust
selective_import_selector_class_is_non_none
```

or rename it to reflect the real invariant.

Input:

```phalcom
from universe.errors.unsupported import unsupported
```

Expected:

- dependency resolution succeeds;
- imported value is available;
- no `ModuleNotFound`;
- no missing runtime export error.

---

## A-T05 — REPL module export associated/property access

Retain:

```rust
module_import_selector_property_access_is_non_none
```

Input:

```phalcom
import universe.errors.unsupported
unsupported.unsupported
```

Expected:

- module import succeeds;
- export dispatch returns the exported value;
- no `doesNotUnderstand`.

---

## A-T06 — REPL canonical root façade re-export

Retain:

```rust
universe_root_exports_package_info
```

Input:

```phalcom
from universe import PackageInfo
```

Expected:

- selective import succeeds;
- resulting value is the canonical class;
- no root-local-slot assumption is required.

---

## A-T07 — Exposed child still is not a selective export

Retain the existing negative regression where an exposed child module is not an exported binding.

Expected:

```text
from universe.<package> import <exposed-child-name>
```

fails unless that exact name is separately exported.

This is a mandatory anti-regression gate.

---

## A-T08 — Re-export consumer uses target `RuntimeExportRef`

Add a focused runtime/REPL test using any façade binding whose runtime export target lives in a different module.

Prove:

```text
selective import
    ↓
uses RuntimeExportRef::Binding target directly
```

rather than:

```text
target façade slot_of(public_name)
```

This can use `PackageInfo` if it provides sufficient structural evidence.

---

# 8. Patch A implementation tasks

## Task A1 — Extract reusable linked-export runtime projection

### Goal

Move the ordinary export-table materialization logic out of the monolithic `VM::materialize_program` Phase 5 into a reusable helper.

Preferred conceptual API:

```rust
fn materialize_linked_export_table(
    &mut self,
    module: &ModuleId,
    interface: &LinkedModuleInterface,
) -> PhResult<()>
```

or:

```rust
fn materialize_linked_exports<'a>(
    &mut self,
    modules: impl Iterator<Item = (&'a ModuleId, &'a LinkedModuleInterface)>,
) -> PhResult<()>
```

Choose the smallest API that can be called from:

1. ordinary `materialize_program`;
2. canonical Universe bootstrap.

### Required behavior

For:

```rust
LinkedExportTarget::Binding(symbol)
```

resolve:

- target module object;
- target symbol;
- target slot, declaring only when required by the existing materialization contract;
- canonical Universe class value if `resolve_universe_declaration_class` applies.

Install:

```rust
RuntimeExportRef::Binding(BindingRef {
    module: target_module_object,
    slot,
})
```

For:

```rust
LinkedExportTarget::Module(target_module_id)
```

install:

```rust
RuntimeExportRef::Module(target_module_object)
```

### Critical rule

The helper must materialize the **linked target**, not assume:

```text
exporting module == target module
public export name == target local symbol name
```

### What not to do

Do not:

- clone the old Phase 5 body into `builtin_materialize.rs`;
- introduce `if Universe { ... } else { ... }` target-identity semantics;
- read `UNIVERSE_BINDINGS` to decide runtime export visibility;
- derive exports from `ModuleObject.globals`;
- derive exports from `expose`.

---

## Task A2 — Route ordinary materialization through the helper

Replace `VM::materialize_program` Phase 5's inline projection with calls to the new helper.

Run ordinary module/materialization tests before changing Universe bootstrap.

This proves extraction did not change existing user-program semantics.

---

## Task A3 — Materialize canonical Universe linked exports during bootstrap

### Goal

After canonical modules are allocated and the linked canonical program is available, materialize the linked export tables onto those existing module objects.

### Ordering

Preferred ordering:

```text
allocate canonical Universe modules
    ↓
install required native values / canonical owner slots
    ↓
prepare source bindings / linked reads as required
    ↓
materialize linked export tables
    ↓
execute canonical Universe initializers
```

The exact point must be chosen so every `LinkedExportTarget::Binding` can resolve a stable module + slot without requiring its final source value to have executed yet.

The public export table may point to a slot before its final initializer value is written, exactly as ordinary materialization can reserve binding identity before initialization.

### Requirement

Do not re-run whole program materialization against already-allocated Universe modules if that would duplicate:

- module allocation;
- ownership/context initialization;
- declaration blueprint installation;
- runtime roots;
- semantic metadata loading.

Call only the extracted export projection.

---

## Task A4 — Reduce native bootstrap export authority

`install_universe_native_bindings` currently inserts native exports directly.

After linked export projection is authoritative, review these insertions.

Preferred end state:

- native bootstrap installs canonical values/slots;
- linked interface projection installs public exports;
- direct child package exports are also linked-interface-owned if the linked model already represents them.

If removing all direct export insertion is too large for this patch, at minimum:

1. ensure linked projection overwrites/normalizes the final table;
2. document direct insertion as bootstrap staging only;
3. add equality tests proving final runtime exports equal linked exports.

Do not leave two independently meaningful final export authorities.

---

## Task A5 — Repair REPL selective import to consume `RuntimeExportRef`

In `ModuleExecutionContext::process_cell_dependencies`, replace:

```text
exports.contains_key(name)
    ↓
slot_of(name)
    ↓
BindingRef { target_obj, slot }
```

with direct export resolution:

```rust
match vm.heap.module(target_obj).exports.get(&item_sym) {
    Some(RuntimeExportRef::Binding(binding)) => {
        RuntimeLinkedRead::Binding(*binding)
    }
    Some(RuntimeExportRef::Module(module)) => {
        RuntimeLinkedRead::Module(*module)
    }
    None => {
        missing export diagnostic
    }
}
```

Adapt ownership/copy semantics to the actual enum definitions.

### `LinkedImportInfo`

Preserve the correct symbolic target:

- for a Binding export, use the canonical target module/name if the runtime export structure retains enough information;
- if the REPL bookkeeping currently requires `SymbolId`, derive it from the linked/static resolution path rather than reverse-engineering a name from a runtime slot.

Do not write a fake `SymbolId` using the façade module solely because the import was spelled through that façade.

If the REPL's `LinkedImportInfo.symbol` cannot represent a module-valued export, keep it `None` consistently with existing whole-module behavior.

---

## Task A6 — Remove or constrain duplicate fallback Universe materialization

`ensure_module_materialized` returns immediately for already-registered Universe modules during a normal `VM::new()` REPL session.

After Patch A is green, inspect the remaining `ProjectIdentity::Universe` fallback branch.

Choose one:

### Preferred

Delete unreachable duplicate export construction if no legitimate runtime path uses it.

### Acceptable

Keep it only for a documented alternate construction path, but route its export materialization through the same shared helper or an equivalent authoritative linked-interface path.

### Forbidden

Leave a second hand-built export algorithm that can drift from canonical bootstrap again.

---

# 9. Patch A expected intermediate failures

During staging, expect:

### After A1 extraction

Possible failures:

- ordinary module export tests;
- missing canonical target module registration;
- slot overflow conversion issues;
- borrow conflicts from mutating target modules while iterating interfaces.

Resolve without changing semantics.

### After A3 Universe projection

Possible failures:

- a linked canonical export targets a slot not yet reserved;
- source-only class slot preparation order is insufficient;
- root façade canonical exports target modules not yet registered.

These are ordering/slot-preparation defects. Do not "fix" them by creating façade-local duplicates.

### After A5 REPL repair

Possible failures:

- `LinkedImportInfo.symbol` assumes the public façade identity rather than canonical target identity;
- a module-valued export reaches code that assumed selective imports are always bindings.

Repair the data model at that exact consumer seam without widening module semantics.

---

# 10. Patch A deletion/negative gates

Run:

```bash
rg 'exports\.contains_key.*slot_of|slot_of\(item_sym\)' phalcom-core/src/modules/context.rs
```

Expected after repair:

- no selective-import path that confirms export visibility and then assumes a local same-name slot.

Search for duplicate Universe runtime export construction:

```bash
rg 'RuntimeExportRef::(Binding|Module)' phalcom-core/src/modules
```

Review every match.

Final state must have one clearly authoritative linked-export projection path.

Search for accidental exposed-child fallback:

```bash
rg 'expose|child' phalcom-core/src/modules/context.rs phalcom-core/src/modules/builtin_materialize.rs
```

No new selective-import fallback should have been added.

---

# 11. Patch A verification commands

Minimum focused gate:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-repl --test repl_import_bugs -- --nocapture
```

Expected: all tests pass.

Core module/runtime gates:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core object_model_invariants -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus -- --nocapture
```

Module-layer gate:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --all-targets
```

Workspace behavioral gate:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets
```

At the inspected branch head, the three REPL failures are the remaining workspace test blocker. Patch A completion requires this workspace test to become green unless a new unrelated baseline failure appears and is independently reproduced.

Formatting/lint:

```bash
cargo fmt --all -- --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
```

---

# 12. Patch A completion checklist

- [ ] Runtime `universe.errors.unsupported.exports["unsupported"]` exists.
- [ ] Its target is the canonical binding.
- [ ] Runtime `universe.exports["PackageInfo"]` resolves to the canonical owner module/slot.
- [ ] Root `PackageInfo` is not turned into a fake root-owned declaration.
- [ ] `from universe.errors.unsupported import unsupported` passes in REPL.
- [ ] `import universe.errors.unsupported; unsupported.unsupported` passes.
- [ ] `from universe import PackageInfo` passes.
- [ ] Exposed child without explicit export still fails selective import.
- [ ] REPL selective import consumes `RuntimeExportRef` rather than reconstructing a local slot.
- [ ] Ordinary program export materialization is unchanged behaviorally.
- [ ] Canonical runtime export names match canonical linked-interface export names.
- [ ] No second final Universe export authority remains.
- [ ] `cargo test --workspace --all-targets` passes or any new unrelated blocker is independently reproduced and documented.
- [ ] Workspace Clippy remains green.
- [ ] Format check passes.

---

# Patch B — E010 Unhandled Scheduler Failure Observability

# 13. Patch B objective

Implement the semantic rule:

> A scheduler-owned Fiber whose uncaught failure has no durable completion owner is an **unhandled scheduled failure**. The scheduler records and reports it without aborting sibling scheduled work.

This must fix the diagnostic failure:

```phalcom
const fut = Future.new()

System.schedule(|| {
  fut.complete(42) // typo; should fail
})

System.print(fut.await)
```

The user must no longer see only:

```text
await: the future is still pending and the scheduler is empty; nothing can settle it
```

while the actual scheduled error disappears.

---

# 14. Patch B semantic model

Terminal scheduler Fiber:

```text
Scheduler-owned Fiber
        │
        ├── Done(value)
        │      └── no failure report
        │
        └── Failed(error)
               │
               ├── had completion observer
               │      └── observer owns terminal failure
               │
               └── no completion observer
                      └── record unhandled scheduler failure
```

The public behavior is:

```text
failed detached task
    +
later runnable sibling
        ↓
sibling still executes
        ↓
failure becomes visible exactly once
```

---

# 15. Patch B exact files and seams

Primary runtime files:

- `phalcom-core/src/vm/mod.rs`
- `phalcom-core/src/vm/dispatch.rs`
- `phalcom-core/src/vm/gc.rs` or whichever root-tracing file owns VM auxiliary roots
- `phalcom-core/src/primitive/system.rs`
- `phalcom-core/core/universe/src/concurrency/fiber.ph`

Tests:

- `phalcom-core/tests/fixtures/language/concurrency/...`
- core VM unit tests near scheduler/Fiber terminalization
- `docs/work/errors/E010-pump-swallows-task-errors.md`

Potential output abstraction file:

- `phalcom-core/src/vm/output.rs`

Use the existing `RuntimeOutput` sink; do not print directly to process stderr/stdout from new VM code if the repository already routes Phalcom-visible output through VM-owned sinks.

---

# 16. Patch B data model

Introduce a VM-owned record for unhandled scheduler failures.

Preferred shape:

```rust
#[derive(Clone, Copy, Debug)]
pub(crate) struct UnhandledSchedulerFailure {
    pub fiber: ObjRef,
    pub error: Value,
}
```

and:

```rust
pub(crate) unhandled_scheduler_failures:
    VecDeque<UnhandledSchedulerFailure>,
```

A simpler `VecDeque<ObjRef>` is acceptable only if:

- the Fiber's terminal `result` remains the authoritative error;
- the Fiber is guaranteed retained;
- reporting can recover all required context without ambiguity.

The explicit `{ fiber, error }` record is preferable because it makes the diagnostic channel self-contained.

---

# 17. Patch B GC requirements

If the queue stores `ObjRef` or `Value`, it is a VM root.

Add it to the same root tracing path that currently handles:

- current VM stack/frames;
- ready queue;
- temporary roots;
- parked Fiber-owned state;
- other VM auxiliary handles.

Required invariant:

```text
unhandled failure recorded
    ↓
forced GC
    ↓
Fiber + Error remain valid
    ↓
report still succeeds
```

Add a forced-GC regression.

Do not rely on incidental reachability from a local stack or the ready queue after terminalization.

---

# 18. Patch B tests to add first

## B-T01 — Detached scheduled failure remains isolated

Keep the existing regression:

```phalcom
System.schedule(|| { throw Error.new("boom") })
System.schedule(|| { System.print("second-ran") })
System.print("main-exits-cleanly")
```

Required output properties:

- `"second-ran"` appears;
- main/root execution does not abort before sibling work;
- new unhandled-failure report appears according to the chosen report format.

The old expected file will need intentional update because E010 changes observability, not isolation.

---

## B-T02 — Detached scheduled failure is reported exactly once

Schedule one failing detached Fiber and drain the scheduler.

Assert the error text appears once.

Drain scheduler again.

Assert no duplicate report.

---

## B-T03 — Multiple detached failures are all reported

Schedule:

```text
A fails
B fails
C succeeds
```

Expected:

- C runs;
- A report appears once;
- B report appears once;
- report order matches terminalization/FIFO execution order.

Do not silently retain only "last failure".

---

## B-T04 — `Future.async` failure is not double-reported

Create:

```phalcom
const f = Future.async(|| {
  Error.new("owned failure").raise()
})
```

Drive scheduler and observe/recover via:

```phalcom
f.catch(...)
```

or `f.await` under a controlled rejected path.

Expected:

- Future becomes rejected;
- no "unhandled scheduled failure" report for the action Fiber;
- the completion observer remains the sole owner.

---

## B-T05 — Suspended Future action later fails and is still owned

Exercise:

```text
Future.async action
    ↓
await pending dependency
    ↓
park
    ↓
wake
    ↓
later terminal failure
```

Expected:

- derived Future rejects;
- no detached scheduler failure report.

This protects the exact C3/C4 completion-observer remediation.

---

## B-T06 — Completion observer callback Fiber can itself fail unowned

A terminal observer is scheduled as a fresh Fiber.

If that observer Fiber itself has no owner and fails, classify that observer Fiber's own terminal failure as unhandled.

This proves "observer exists" ownership applies to the **observed Fiber**, not recursively to arbitrary work spawned by observation.

---

## B-T07 — Forced GC preserves pending report

Schedule failing detached work, cause terminalization, force GC before report consumption if the architecture permits that staging, then report.

Expected:

- no use-after-free/stale handle;
- original error is intact.

---

## B-T08 — Root `Future.await` quiescence includes relevant unhandled failures

Reproduce E010:

```phalcom
const fut = Future.new()

System.schedule(|| {
  fut.complete(42)
})

fut.await
```

Expected diagnostic must include:

1. Future is still pending / scheduler quiescent;
2. at least one unhandled scheduled failure occurred while driving;
3. the original `doesNotUnderstand 'complete(_)'` error or equivalent root cause is visible.

The diagnostic must **not** claim:

```text
this failed task was definitely the Future's settler
```

because the runtime cannot prove that causal relationship.

---

## B-T09 — Pre-existing unrelated unhandled failures do not contaminate a later await window

If root await diagnostics consume "failures seen while this await drove the scheduler", snapshot the failure sequence/index before pumping.

Test:

1. produce an earlier detached failure;
2. report/retain it according to global policy;
3. start a fresh pending Future await;
4. cause no new scheduled failure;
5. hit quiescence.

Expected: the earlier unrelated failure is not presented as if it occurred during this await drive.

---

# 19. Patch B implementation tasks

## Task B1 — Add scheduler failure record storage

Add VM queue/state.

Recommended supporting methods:

```rust
fn record_unhandled_scheduler_failure(
    &mut self,
    fiber: ObjRef,
    error: Value,
)

fn drain_unhandled_scheduler_failures(
    &mut self,
) -> Vec<UnhandledSchedulerFailure>
```

or equivalent.

For await-window correlation, add a monotonic report/failure sequence if useful:

```rust
next_scheduler_failure_seq: u64
```

and store:

```rust
seq: u64
```

This is optional but cleaner than indexing a mutable queue if diagnostics need a stable "since await started" boundary.

---

## Task B2 — Classify ownership at terminalization

The decisive moment is scheduler-mode terminal failure in `VM::run_until` / Fiber-floor handling.

Before detaching a completion observer, record:

```rust
let had_completion_owner =
    self.heap.fiber(failed).completion_observer.is_some();
```

Then perform existing terminalization:

- capture error;
- mark `Failed`;
- clear terminal stacks as currently required;
- enqueue completion observer exactly once.

Then:

```text
if resume_mode == Scheduler
and !had_completion_owner
    record unhandled scheduler failure
```

### Call-mode cascades

Do not automatically classify every `Call`-mode Fiber in a failure cascade as a detached scheduler failure.

The unhandled scheduler report belongs at the scheduler-owned terminal boundary.

If a call-mode cascade terminalizes multiple observer-bearing Fibers, preserve current completion-observer behavior.

If the final scheduler-owned Fiber in the chain fails without an owner, report that scheduler-owned terminal computation according to the established Fiber semantics.

Add a targeted test if this path is ambiguous.

---

## Task B3 — Preserve scheduler return-value behavior

Keep:

```rust
FiberResumeMode::Scheduler
```

as capture/not-propagate.

Do not alter it to `Call`.

Do not make `fiber_resume_scheduled` return `Err` for a guest Fiber's uncaught raise.

The scheduling loop must continue to be able to run later ready work.

---

## Task B4 — Add a safe report-consumption boundary

Reporting must occur outside Fiber-floor terminalization.

Preferred report points:

1. `System.runScheduled` after/between scheduler turns at a safe top-level Phalcom boundary;
2. native root-drive pump before returning final root result;
3. root `Future.await` when quiescence is detected.

Do not create three independent formatting/report implementations.

Add one VM/native reporting helper and call it from relevant boundaries.

Conceptual API:

```rust
fn report_unhandled_scheduler_failures(&mut self) -> PhResult<usize>
```

or:

```rust
fn take_unhandled_scheduler_failures(
    &mut self
) -> Vec<UnhandledSchedulerFailure>
```

with one formatter.

### Output channel

Route reporting through `VM::RuntimeOutput`.

Do not bypass embedders/tests by writing directly through `eprintln!`.

---

## Task B5 — Define the report format

Use a compact stable format suitable for golden fixtures.

Recommended v1 shape:

```text
Unhandled scheduled Fiber failure:
<rendered Error>
```

With Fiber identity if stable and useful:

```text
Unhandled scheduled Fiber #7 failure:
<rendered Error>
```

Avoid over-designing a multi-line task-supervision report before `Task` exists.

If traceback information is already retained in the `Error` value or Fiber tracing layer, reuse the existing renderer rather than inventing a second traceback system.

---

## Task B6 — Make `System.runScheduled` surface reports without aborting work

Current `.ph` method drains queue by repeated `_$resumeScheduled()`.

Two acceptable approaches:

### Preferred native support

Expose an **internal** `System._$reportUnhandledScheduledFailures` or equivalent primitive used by the `.ph` pump after it has drained runnable work.

Public surface remains unchanged.

### Alternative

Move `System.runScheduled` itself into a native scheduler-drain helper if and only if doing so does not reintroduce native re-entry suspension restrictions.

Given the existing deliberate `.ph` top-level resume structure, prefer the first approach.

Expected flow:

```phalcom
while queue has work {
    f._$resumeScheduled()
}
System._$reportUnhandledScheduledFailures
()
```

If reports should be emitted turn-by-turn rather than only at drain completion, document and test that policy. Drain-completion reporting is simpler and preserves sibling progress.

---

## Task B7 — Root-drive pump reporting

The root native pump in `VM::run_until` / `VM::run` may drain scheduler work after root activation completes.

Before returning the host/root final value, surface any unhandled scheduled failures collected during that drain.

Required invariant:

```text
root returns
scheduled A fails
scheduled B succeeds
    ↓
B runs
failure report emitted
    ↓
host run still returns root value
```

Do not convert reporting into host failure in this patch.

---

## Task B8 — Enhance root `Future.await` quiescence

Root `Future.await` currently sees only:

```text
queue empty + Future pending
```

Add an internal mechanism to query failures recorded during the current await drive.

Preferred design:

1. snapshot a scheduler-failure sequence/cursor when entering the root pump;
2. drive scheduler;
3. on quiescence, retrieve unhandled failures since that cursor;
4. construct the quiescence error message with a concise causal hint.

Recommended message structure:

```text
await: the future is still pending and the scheduler is empty;
nothing can currently settle it.
1 scheduled computation failed while this await was driving the scheduler:
<error>
```

For multiple failures:

```text
N scheduled computations failed while this await was driving the scheduler
```

Include a bounded number of rendered errors if diagnostics can otherwise explode. If a bound is introduced, make it deterministic and test it.

### Important ownership rule

Do not drain/delete globally unhandled failures in a way that prevents the ordinary scheduler report from occurring unless the await diagnostic itself is explicitly defined as consuming/reporting those failures.

Choose one policy and test exactly-once behavior.

Preferred:

```text
record
    ↓
await quiescence consumes/reports failures from its drive window
    ↓
global reporter does not print them again
```

This avoids duplicate diagnostics.

---

## Task B9 — Leave `Fiber#error` reflective

Do not add:

```text
failure_observed = true
```

on getter access.

Current v1 policy:

```text
no completion owner at terminalization
    → unhandled scheduled failure
```

A later Task API may introduce explicit `join`/observation semantics.

Document this choice in E010 closure notes.

---

# 20. Patch B expected intermediate failures

### After B1/B2

Existing golden fixture for a raising scheduled Fiber may still show no report until report consumption is added.

VM unit tests should nevertheless prove the failure record is created exactly once.

### After B4/B5

Golden output ordering may differ between:

- root's own `System.print`;
- sibling scheduled output;
- failure report.

Choose one deterministic boundary and lock it with tests.

### After B8

Be alert for duplicate reporting:

```text
await diagnostic includes error
+
end-of-pump reporter prints same error again
```

Exactly-once reporting is mandatory.

---

# 21. Patch B deletion/negative gates

Search for scheduler failure inference from raw return values:

```bash
rg '_\$resumeScheduled\(\).*error|resumeScheduled.*is|resumeScheduled.*Error' phalcom-core
```

Expected: no new semantic inference from scheduler resume values.

Search for direct process stderr/stdout reporting introduced by this patch:

```bash
rg 'eprintln!|println!' phalcom-core/src/vm phalcom-core/src/primitive/system.rs
```

Any new match must be justified. Scheduler reports should use VM-owned output.

Search for weakened isolation:

```bash
rg 'FiberResumeMode::Scheduler' phalcom-core/src
```

Review every match. Scheduler mode must remain capture/not-propagate.

Search for accidental public API growth:

```bash
rg 'Unhandled|unhandled|report.*Scheduled' phalcom-core/core/universe/src
```

New report/introspection selectors should be `@internal` unless explicitly ratified otherwise.

---

# 22. Patch B verification commands

Focused concurrency corpus:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::concurrency -- --nocapture
```

Core scheduler/Fiber unit tests:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib scheduler_tests -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib terminal_observer -- --nocapture
```

Use actual current test filters if names differ.

Complete core:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus
```

Workspace:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets
cargo fmt --all -- --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
```

---

# 23. Patch B completion checklist

- [ ] Detached scheduled Fiber failure creates one VM-owned unhandled failure record.
- [ ] Failure record is GC-traced.
- [ ] Sibling scheduled work still runs after a failure.
- [ ] General detached failure becomes visible.
- [ ] One failure is reported once.
- [ ] Multiple failures are all reported in deterministic order.
- [ ] `Future.async` action failure is not double-reported.
- [ ] Suspending/parking Future action that later fails remains Future-owned.
- [ ] Completion observers remain detached exactly once.
- [ ] Root scheduler pump reports unhandled failures before host return.
- [ ] `System.runScheduled` reports unhandled failures without propagating them.
- [ ] Root `Future.await` quiescence includes failures from its own scheduler-drive window.
- [ ] Root await does not falsely claim a failed task was necessarily its settler.
- [ ] Await diagnostics and general reporting do not duplicate the same failure.
- [ ] `Fiber#error` remains a reflective getter with no observation side effect.
- [ ] Existing scheduler-isolation test remains semantically true.
- [ ] Full core tests pass.
- [ ] Full workspace tests pass.
- [ ] Workspace Clippy passes.
- [ ] Format check passes.

---

# 24. E010 documentation closure

After implementation, update:

```text
docs/work/errors/E010-pump-swallows-task-errors.md
```

Change:

```text
Status: OPEN
```

to the repository's normal fixed/closed status.

Record the final rule:

> Scheduler-mode Fiber failure is isolated from sibling work. A terminally failed scheduler-owned Fiber with no durable completion observer is recorded as an unhandled scheduled failure and reported from a safe scheduler/root boundary. Completion-observer-owned failures are not reported through this channel. Root `Future.await` quiescence includes unhandled scheduler failures produced while that await drove the scheduler.

Also explicitly document that this patch does **not** implement:

- Task ownership;
- TaskGroup aggregation;
- structured concurrency;
- cancellation;
- root failure propagation for detached tasks.

---

# 25. Commit structure

Recommended commits:

## Patch A

### Commit A1

```text
test(modules): expose canonical runtime export materialization gap
```

Tests only; red.

### Commit A2

```text
fix(modules): materialize linked exports for canonical universe
```

Includes reusable export projection + Universe bootstrap integration.

### Commit A3

```text
fix(repl): consume runtime export targets for selective imports
```

Repairs `RuntimeExportRef` consumption and removes local-slot assumption.

### Commit A4, only if warranted

```text
refactor(modules): remove duplicate universe export fallback
```

Behavior-preserving cleanup after all tests are green.

---

## Patch B

### Commit B1

```text
test(concurrency): specify unhandled scheduled failure semantics
```

Red tests / golden changes staged against intended behavior.

### Commit B2

```text
fix(concurrency): record unowned scheduler failures
```

VM data model, GC roots, terminal classification.

### Commit B3

```text
fix(concurrency): report detached scheduler failures
```

Safe report boundaries, `System.runScheduled`, root pump.

### Commit B4

```text
fix(concurrency): enrich await quiescence with task failures
```

Drive-window correlation and exactly-once diagnostics.

### Commit B5

```text
docs(concurrency): close E010 failure observability
```

Documentation only.

Do not squash Patch A and Patch B into one semantic commit during implementation review.

---

# 26. Final certification matrix

| Gate | Required result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo build --workspace --all-targets` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test -p phalcom-modules --all-targets` | PASS |
| `cargo test -p phalcom-repl --test repl_import_bugs -- --nocapture` | PASS |
| `cargo test -p phalcom-core --test core` | PASS |
| `cargo test -p phalcom-core --test language-corpus` | PASS |
| `cargo test --workspace --all-targets` | PASS |
| Runtime Universe linked-export equality invariant | PASS |
| Re-export target identity invariant | PASS |
| Exposed-child negative import invariant | PASS |
| Scheduler sibling-survival invariant | PASS |
| Detached failure exactly-once report invariant | PASS |
| Future-owned failure no-double-report invariant | PASS |
| GC retention of pending scheduler-failure report | PASS |
| Await quiescence root-cause diagnostic invariant | PASS |

---

# 27. Final architectural acceptance criteria

The implementation is complete only if all of the following statements are true.

## Modules / REPL

```text
The linker decides what a module exports.
The runtime materializer decides how that linked export becomes a runtime reference.
The REPL consumes that runtime reference.
No layer reconstructs export identity from a public name alone.
```

## Scheduler errors

```text
Fiber terminalization decides whether a scheduler failure is owned.
The scheduler records unowned failures.
Safe scheduler/root boundaries report them.
Sibling work remains isolated.
Future-owned failures remain Future-owned.
```

If the final patch instead produces either of these architectures, reject it:

```text
native metadata → hand-built final Universe export semantics
```

or:

```text
scheduler resume return value → guessed failure semantics
```

Those would recreate the exact split-authority problems this remediation is meant to remove.
