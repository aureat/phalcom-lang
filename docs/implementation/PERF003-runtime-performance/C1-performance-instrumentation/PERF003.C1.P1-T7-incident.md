# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `PERF003`
- Checkpoint: `PERF003.C1`
- Plan: `PERF003.C1.P1`
- Task/Gate: `T7 / G3`
- Repository: Phalcom (`/Users/altunhasanli/dev/phalcom/phalcom`)
- Branch: `main` (shared worktree contains unrelated LANG007 changes)
- Starting revision: `5a5bf8b2`
- Current revision: `5a5bf8b218c2` (HEAD unchanged)

## 2. Escalation trigger

Mandatory RED triggers 1, 3, 5, 6, 8, and 15 from
`docs/workflow/shared-consultation-escalation-protocol.md`: the planned live
heap snapshot does not define the representation boundary, and implementation
would change runtime object ownership, mutable-cache lifetime, handle/aliasing
semantics, rooting, GC, and VM resource ownership.

## 3. Required invariant

- Preserve the full generational `ObjRef` identity and internal object-graph
  aliasing across every image instantiation.
- Keep executable mutable caches, heap state, roots, and runtime resources
  isolated per VM.
- Preserve canonical runtime identities and observable bootstrap behavior.
- Leave production `VM::new()` semantics unchanged.

## 4. Expected architecture

T7 proposes one process-shared, thread-safe frozen image of canonical
post-bootstrap state. Each fixture receives a fresh VM reconstructed from that
image without recompiling or re-executing Universe initializers. The image must
retain canonical runtime registries and roots, while stacks, active fibers,
upvalues, scheduler/reactor state, output, pending failures, temporary native
state, counters, and mutable execution caches remain instance-local.

The plan's `HeapImage` requires numerically stable slot-map keys. The attached
architect audit establishes that stable `ObjRef` values alone are not a
sufficient representation or soundness contract.

## 5. Observed behavior

No T7 implementation exists, so there is no failing runtime reproducer. Source
inspection shows that closures hold shared `Rc<Callable>` values, and callable
chunks contain interior-mutable dispatch, global, and executable-semantic
caches. A process-wide frozen value containing these live structures does not
meet the required independent-cache and thread-safe-sharing boundary. Replacing
`Rc` with `Arc` alone would still share those mutable caches.

The VM also owns a root fiber and active stack state, canonical registries,
output, a resource table, a reactor, timing state, and GC policy. Their freeze,
reconstruction, and per-instance ownership rules have not been specified.

## 6. Minimal reproduction

Not applicable: this is a representation and ownership conflict, not a runtime
failure. The structural evidence is in `phalcom-core/src/heap/closure.rs`
(`ClosureObject.callable`), `phalcom-core/src/callable.rs` (`Callable.chunk`),
`phalcom-core/src/chunk.rs` (`Chunk.caches`, `Chunk.gcaches`, and
`ExecutableSemanticPool` cache cells), `phalcom-core/src/vm/mod.rs` (`VM` state),
`phalcom-core/src/vm/bootstrap.rs` (fresh root-fiber and per-VM initialization),
and `phalcom-core/src/heap/mod.rs` (`Heap` slot map and GC policy).

## 7. Relevant code path

```text
VM::try_new_with_output
  -> VM::try_new_with_native_install_mode_and_output
  -> fresh kernel heap, root fiber, registries, resources, reactor, output
  -> native installation and source-Universe bootstrap
  -> VM-owned callable chunks with mutable Cell-backed caches

ClosureObject
  -> Rc<Callable>
  -> Callable.chunk
  -> dispatch/global/executable-semantic cache cells
```

`phalcom-core/src/modules/canonical_universe.rs` caches the compiled canonical
Universe program. It does not provide a frozen live-VM heap or a per-VM cache
reconstruction contract.

## 8. Scoped implementation diff

No T7 source changes were made. T1–T6 work is recorded in
`CHECKPOINT.md`; the directly affected VM-tier tests pass, while broad semantic
and core gates have separately recorded inherited or unresolved failures.

## 9. Evidence

- `ClosureObject` derives `Clone` and stores `Rc<Callable>`.
- `Callable` owns a `Chunk`; `Chunk` has `Cell`-backed inline/global caches and
  executable-semantic cache cells.
- `VM` contains the heap, root/current fiber handles, canonical module/typing/
  ADT/data/product registries, roots, output, resources, reactor, and active
  execution state.
- `Heap` stores generational slot-map object handles and GC threshold/stress
  state.
- The attached architect audit independently verified these obstacles and
  requires a frozen representation plus VM-local mutable-cache/root/GC rules.

## 10. Implementer's causal hypothesis

The plan treats `HeapImage` cloning as the central operation, but correctness
depends on defining the entire image boundary and rehydrating mutable executable
state. Directly retaining or cloning live closures would preserve shared cache
backing; directly putting the live heap in a process-shared `OnceLock` does not
establish thread safety. A typed frozen representation and explicit
VM-local reconstruction boundary are required before a sound implementation
can be selected.

## 11. Alternative hypotheses

The direct closure/cache sharing obstacle may be resolved by a purpose-built
frozen callable representation that shares immutable code while rebuilding all
mutable caches per VM. It may instead require cloning callable/chunk state
entirely per VM. The available evidence does not establish which boundary is
correct for every executable-semantic cache, closure/upvalue shape, runtime
registry, and root. No alternative has been implemented or selected.

## 12. Attempts already made

No speculative T7 changes were attempted. Source inspection and the architect's
audit were used to establish the ownership and cache boundary before editing.

## 13. Why local authority is insufficient

Choosing the image representation determines runtime identity, mutable-cache
isolation, GC reachability, root-fiber ownership, registry ownership, closure
capture behavior, resource lifetime, and initialization/error semantics. These
are cross-cutting VM contracts, not private implementation details, and T7's
plan-level `HeapImage` pseudocode does not settle them.

## 14. Decision requested

Provide a material T7 amendment that chooses the thread-safe frozen
representation and the exact freeze/rehydration boundary. It must specify:

- preservation of full slot/generation handles and internal aliasing;
- reconstruction or reset of VM-local callables, dispatch/global caches, and
  executable-semantic caches;
- root-fiber handling and ownership of module, typing, ADT, data, product,
  dispatch, canonical, and semantic registries;
- canonical roots, GC reachability, thresholds, stress configuration, and
  collection after instantiation;
- resource, reactor, clock, output, counters, scheduler, and pending-failure
  ownership;
- allowed captured closures and closed upvalues, plus freeze rejection rules;
- initializer side effects, failure/fallback behavior, and equivalence with
  ordinary bootstrap;
- focused acceptance tests for independent object/cache mutation, class
  reopening after cache warmup, closures/module references, reflection and
  canonical identities, GC, diagnostic/source-location parity, and zero warm
  Universe compilation/initializer execution.

Also decide whether this material architecture belongs in amended T7 or a
separate corrective plan in `PERF003.C1`. The implementer remains stopped on T7
until this decision is recorded in `CHECKPOINT.md` and the plan is amended.
