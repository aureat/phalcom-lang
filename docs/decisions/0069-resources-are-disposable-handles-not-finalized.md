# 69. Native resources are disposable handles with a generation-tagged table; no finalizers

- Status: Accepted
- Date: 2026-07-20
- Related: [ADR-0050](../adr/accepted/0050-non-moving-mark-sweep-collector.md) (§Context banks
  "no finalizers exist" as a reason the collector is hazard-free — **this decision keeps that
  true**), [ADR-0013](../adr/accepted/0013-closure-upvalues-and-frame-token-return.md)
  (frame tokens — the generation-tag precedent),
  [ADR-0008](../adr/accepted/0008-layered-exceptions-and-result.md) §4 (unified unwind — what
  makes `ensure`-scoping sound), [decision 0068](0068-io-is-future-shaped-reactor-owned.md)
  (the resources this governs), [ffi.md](../spec/v0.2/drafts/ffi.md) §8 **F-3** (this closes it),
  `docs/logs/2026-07-19-ensure-temp-root-uaf.md`

## Context

ADR-0050 lists "No finalizers exist" among the reasons its collector has no ordering or
resurrection hazards. Native resources — file descriptors, sockets, directory handles — are the
classic reason a language grows finalizers anyway, and [ffi.md](../spec/v0.2/drafts/ffi.md) F-3
recorded the collision without ruling on it.

The precedents are unusually clear. Java deprecated `finalize`. Python's `__del__` still permits
resurrection. C# shipped finalizers *and* still needed `IDisposable`, which is what people
actually use. Every language that took release-on-collect ended up with explicit disposal as
well, having paid for both.

Phalcom also has the mechanism already, twice: frame tokens (ADR-0013) and the generational
`SlotMap` that makes a stale handle a **defined panic rather than undefined behavior**. A
resource table is that same idea a third time.

## Decision

### 1. No finalizers. ADR-0050 §Context stands unamended

Nothing is released because it was collected. Collection frees memory and nothing else.

### 2. Resources are ordinary escapable handles

`File.open(_)` returns a handle that may be stored in a field, returned, and passed around — a
server holds a listening socket for its whole life, and a scope-only design cannot express that.

### 3. `Disposable` protocol

```
Disposable#dispose -> None       // idempotent; releasing twice is not an error
Disposable#isDisposed -> Bool
```

Operating on a disposed handle raises, and the error names the resource and the disposal site.

### 4. A generation-tagged resource table in the VM

The OS handle lives in a VM-side table, not in the Phalcom object. The object holds a
generation-tagged index. Disposal bumps the generation.

Consequences that make this the load-bearing choice:

- Use-after-dispose is a **defined raise**, never a reused-fd write to the wrong file — the same
  guarantee, by the same mechanism, that `SlotMap` gives the object heap.
- Collected-without-dispose is a **leak, not a UAF**. It is detectable and reportable.
- The table is a GC root for nothing: it holds OS handles, not `Value`s.

### 5. Leaks are reported, not silently tolerated

```
System.leakReport -> List        // undisposed resources, with their allocation site
System.strictResources(_)        // Bool; raise on leak instead of warning
```

Undisposed resources at exit produce a diagnostic naming the allocation site. Test lanes should
set `strictResources(true)`.

### 6. `using` — scoped disposal sugar

```phalcom
using f = File.open("a.txt") {
    f.readAll
}
```

desugars to `let f = File.open("a.txt"); ensure { f.dispose } { ... }`. Multiple bindings dispose
in **reverse** order:

```phalcom
using inFile = File.open(a), outFile = File.create(b) { ... }
```

`using` is sugar, not a new mechanism: it lowers onto `ensure`, which is ADR-0008 §4's unified
unwind. Note the dependency — `ensure` dropped live values whenever its cleanup block collected
until `cdd2117` (`docs/logs/2026-07-19-ensure-temp-root-uaf.md`). This ruling is only safe to
implement on top of that fix.

## Consequences

- ADR-0050 is **not** reopened; its "no finalizers" premise remains literally true.
- Forgetting `dispose` leaks an fd for the process lifetime. That is the accepted cost, and §5 is
  what keeps it observable rather than mysterious. A long-running server that leaks will exhaust
  its fd limit — the leak report is the intended way to find that, and it must be built with the
  table rather than after it.
- The resource table must be drained on VM shutdown, and on isolate teardown if
  [decision 0067](0067-no-user-visible-threads-fibers-and-isolates.md) §2 is ever built.
- `using`'s desugaring makes `ensure` load-bearing for resource safety, so `ensure`'s own
  correctness is now a resource-safety property. Its GC-rooting regression test
  (`ensure_outcome_survives_collecting_cleanup`) should be read as guarding this decision too.
- **Closes ffi.md F-3.**

## Alternatives rejected

- **Scope-only (`using` with no escapable handle).** Provably leak-free and cheapest, but cannot
  express a socket held in a field. Too tight for the surface 0068 implies.
- **Real finalizers / release-on-collect.** Reopens ADR-0050, imports finalizer ordering and
  object resurrection, and — per §Context — historically arrives *alongside* explicit disposal
  rather than instead of it, so it buys a hazard without removing the API.
- **Refcounted handles.** Would make disposal automatic and deterministic, but reintroduces the
  `Rc` shape ADR-0009 removed by construction, and cycles through a resource-holding object would
  leak silently.
