# 69. Native resources are closeable handles with a generation-tagged table; no finalizers

- Status: Accepted
- Date: 2026-07-20
- Revised: 2026-07-20, same day, **before any implementation** — the interface was ratified in a
  follow-up pass. Changes: `dispose`/`isDisposed` → **`close`/`isClosed`** (§3a); `close` returns
  **`Future`** settling to `Result` rather than `None`, so a lost-write `EIO` is not discarded
  (§3b); **`Resource` is a real root class** while Reader/Writer/Seekable stay duck-typed (§3);
  use-after-close **raises** rather than returning `Err` (§4); **`using` sugar withdrawn** (§6);
  the full selector surface is ratified in §7. Amended in place rather than as a separate decision
  because nothing was built against the original.
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

### 3. `Resource` is a real root class; the other IO axes are duck-typed

`File` needs to be readable, writable, seekable **and** closeable — four axes against one `extends`
slot (U-INH single inheritance; Phalcom has no traits or mixins). Only one axis gets reified, and
it is closeability:

```
Resource                          // kernel root class
Resource#close    -> Future       // settles to Result; idempotent
Resource#isClosed -> Bool
```

`File < Resource`, `TcpStream < Resource`, `Dir < Resource`. Reader / Writer / Seekable stay
**informal protocols** — a type participates by responding to `read(_)` / `write(_)` / `seek(_)`,
with no declaration and no class.

Closeability is the axis that earns a class because two mechanisms need to *ask the type*: leak
reporting (§5) and any generic cleanup path. Nothing needs to ask "is this a Reader?" — it just
sends `read(_)`. This follows the [ADR-0048](../adr/accepted/0048-amend-iteration-bare-cursor-sentinel-and-iterable-root.md)
`Iterable`-as-kernel-root precedent for the one axis that benefits, and declines it for the three
that would collide with single inheritance.

Reifying all four properly needs mixins or traits. That is a real language feature, deliberately
**not** decided here — see [`docs/deferred/io-protocol-axes-need-mixins.md`](../deferred/io-protocol-axes-need-mixins.md).

### 3a. The name is `close`, not `dispose`

This protocol exists to serve files and sockets, so it uses their word. `dispose` (this decision's
original spelling) reads as generic-resource ceremony on a `File`.

### 3b. `close` returns `Future` settling to `Result`

`close(2)` can fail, and an `EIO` on close means **buffered data was lost after the write already
reported success** — discarding that is a silent-data-loss bug, which is why Go linters flag
`defer f.Close()` on writable files.

It returns `Future`, not a bare `Result`, because closing can block on flush and
[decision 0068](0068-io-is-future-shaped-reactor-owned.md) §1 admits no exceptions. The cost is
accepted with open eyes: **cleanup becomes asynchronous**, and asynchronous cleanup is cleanup that
can be forgotten. §5's leak reporting is what makes that failure observable rather than silent.

`close` is idempotent — closing twice settles to `Ok`, not an error.

### 4. A generation-tagged resource table in the VM

The OS handle lives in a VM-side table, not in the Phalcom object. The object holds a
generation-tagged index. Disposal bumps the generation.

Consequences that make this the load-bearing choice:

- Use-after-close **raises**, and never becomes a reused-fd write to the wrong file — the same
  guarantee, by the same mechanism, that `SlotMap` gives the object heap. It raises rather than
  returning `Err` because it is a contract violation, not an expected condition: an `Err` would
  hide a programmer bug in the same channel as a genuine IO error, where it gets ignored. The
  diagnostic names the resource and the site that closed it.
- Collected-without-close is a **leak, not a UAF**. It is detectable and reportable.
- The table is a GC root for nothing: it holds OS handles, not `Value`s.

### 5. Leaks are reported, not silently tolerated

```
System.leakReport -> List        // undisposed resources, with their allocation site
System.strictResources(_)        // Bool; raise on leak instead of warning
```

Undisposed resources at exit produce a diagnostic naming the allocation site. Test lanes should
set `strictResources(true)`.

### 6. ~~`using` — scoped disposal sugar~~ **WITHDRAWN 2026-07-20, before implementation**

No dedicated syntax for now. Scoped cleanup is written with the existing `ensure`:

```phalcom
const f = File.open("a.txt").await
{ f.readAll.await } .ensure { f.close.await }
```

Withdrawn rather than deferred-with-a-design, because sugar should follow evidence of the pain it
removes, and no `File` user exists yet to produce that evidence. Revisit once the surface has real
callers; the lowering was never the hard part.

Note for whoever revisits: `ensure` dropped live values whenever its cleanup block collected until
`cdd2117` (`docs/logs/2026-07-19-ensure-temp-root-uaf.md`), so any `using` lowering onto `ensure`
inherits that fix as a hard dependency. And per §3b, `close` returns a `Future` — so the sugar must
decide whether it awaits, which is the async-`using` problem and the real reason this is not a
five-line desugar.

### 7. The ratified surface

```
Resource#close                 -> Future     // settles to Result; idempotent
Resource#isClosed              -> Bool

File.open(_)                   -> Future     // read-only
File.create(_)                 -> Future     // write + truncate
File.openWith(_, mode:)        -> Future     // OpenMode.read/.write/.append/.readWrite
File#read(_)                   -> Future     // fills a Bytes, settles to count; 0 = EOF
File#write(_)                  -> Future     // settles to count written
File#flush                     -> Future
File#seek(_)                   -> Future     // SeekFrom.start(_)/.current(_)/.end(_)
File#position                  -> Future
File#metadata                  -> Future
File#path                      -> Path       // cached; non-blocking, so no Future

System.leakReport              -> List       // open resources + allocation site
System.strictResources(_)      -> None       // Bool; raise on leak instead of warn
```

Selector spellings follow [ADR-0012](../adr/accepted/0012-selector-signature-encoding-and-dispatch.md)
(comma form; `openWith(_, mode:)` is one selector with a labelled second parameter) and
[ADR-0043](../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md) (no default
arguments — `open`/`create`/`openWith` are three selectors, not one with defaults). Native
primitives carry the trailing `_` marker (`close_`, `read_`, `write_`) with the `.ph` surface above
them, per U-NATIVE-MARKER.

`File#path` is deliberately **not** a `Future`: it is cached at open time and cannot block. 0068 §1
governs operations that *can* block, not every member.

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
