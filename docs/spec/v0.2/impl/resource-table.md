# Implementation spec — the resource table and `Resource` (U-RESOURCE)

> **Status:** dispatch-ready. Governing records **Accepted**:
> [PDR-0005](../../../decisions/0005-resources-are-disposable-handles-not-finalized.md)
> §3/§3a/§3b/§4/§5; surface laws in
> [`../core/stream-protocol.md`](../core/stream-protocol.md) §3. No dependency on the
> reactor or on any IO surface — this unit is the substrate both need, and it ships
> with **in-memory consumers only** (U-STREAMS is the first real user; a test-only
> resource kind proves the table here).
> **Floor delta:** `Resource#close`/`Resource#isClosed` + `System.leakReport`/
> `System.strictResources(_)` — enumerate in the census with a `NEW_RESOURCE` constant;
> exact spellings below.
> Read [`bytes.md`](bytes.md) §7 (as-built obligations) before starting.
> Anchors verified 2026-07-20 on `3d4174d`.

## 1. Shape

A generation-tagged table of native resources on the VM, a kernel `Resource` root class
whose instances hold only a tagged index, and the leak-reporting surface. No finalizers:
GC sweep must find nothing OS-visible to drop (PDR-0005 §1; the `Object::Bytes` rustdoc
already states this hazard — copy its posture).

## 2. File-by-file

### 2.1 `phalcom-core/src/resource.rs` — new

```rust
/// A generation-tagged handle into [`VM::resources`] (PDR-0005 §4).
pub struct ResourceHandle { pub index: u32, pub generation: u32 }

/// One live table row.
pub struct ResourceEntry {
    pub generation: u32,
    pub kind: ResourceKind,          // enum: Test(..) now; File(fd), TcpStream(..) later
    pub open_site: SourceRange,      // allocation site for the leak report (PDR-0005 §5)
    pub closed: bool,
}
```

- The generation-tag discipline is [`FrameToken`](../../../phalcom-core/src/frame.rs)'s
  (`frame.rs:19`, ADR-0013) — third use of the same idea (PDR-0005 Context). A stale
  handle (generation mismatch) is a **defined error**, never a reuse.
- `ResourceKind` starts with a `Test` variant only (holds a `Vec<u8>` or unit) — enough
  to prove open/close/leak semantics without an fd. `File` arrives with U-FS.
- Table ops (all `&mut VM` methods or on a `ResourceTable` struct owned by `VM`):
  `open(kind, site) -> ResourceHandle`, `resolve(handle) -> Result<&mut ResourceEntry, Stale|Closed>`,
  `close(handle) -> Result<(), CloseError>` (idempotent: closing a closed row is `Ok` —
  stream-protocol §3.1 law 4), `drain()` for VM shutdown (PDR-0005 Consequences),
  `leaks() -> Vec<(kind, open_site)>`.
- **The table is a GC root for nothing** (PDR-0005 §4): it holds no `Value`s. Assert
  this structurally — no `Value`/`ObjRef` field anywhere in the module.

### 2.2 The `Resource` kernel class

- `make_core_class(heap, "Resource", object_class, metaclass_class)` in
  `universe/core_classes.rs` (near `error_class`, `:147-160` region), `CoreClasses`
  field, verify row in `universe/invariants.rs`, **and the `add_class!` row in
  `vm/bootstrap.rs::install_core`** — bytes.md §7 obligation 1, the one that bites.
- An open resource is an `InstanceObject` of a `Resource` subclass whose **first slot
  holds the handle packed as a `Number`** (`index * 2^32 + generation` is exact in f64;
  pack/unpack helpers in `resource.rs` with unit tests at the boundaries). No new heap
  arm and no new `Value` arm — the handle is plain data in an ordinary slot, which is
  what keeps sweep drop glue inert. Do **not** store the handle as two slots (one
  compare-and-swap-free update site, not two).
- Concrete kinds subclass in `.ph` (`File < Resource` later); this unit adds a
  test-only `.ph` subclass in the harness, not in core.ph.

### 2.3 Native primitives — `phalcom-core/src/primitive/resource.rs` (new)

Conventions: `primitive/list.rs` / `primitive/bytes.rs` exactly (bare-or-`None` reads,
type errors for contract violations, `.ph` lifts).

| Rust fn | Binding | Behavior |
|---|---|---|
| `resource_raw_close` | `Resource#close_` | resolve handle: live → run kind-specific close, mark `closed`, bump generation, return `None`; already-closed → `None` (idempotent); stale generation → **raise** `UseAfterCloseError` |
| `resource_raw_is_closed` | `Resource#isClosed_` | `Bool`; stale generation reports `true` (a swept-then-reopened slot must never read as open) |
| `system_leak_report` | `System.leakReport_` | a `List` of `String`s (kind + open site rendered); `.ph` shapes richer rows later |
| `system_strict_resources` | `System.strictResources_(_)` | sets the `VM` flag; `Bool` argument, type error otherwise |

`.ph` layer in core.ph: `Resource#close` lifts `close_`'s result into `Result`
(`Ok(None)` / `Err(e)`) — **synchronous, never a `Future`** (PDR-0005 §3b; PDR-0004 §1's
scope note says *do not "restore consistency"* here); `Resource#isClosed => self.isClosed_`;
`System.leakReport`, `System.strictResources(_)` wrappers.

### 2.4 Error kinds without PDR-0010

`kind: #useAfterClose` (stream-protocol §3.1 law 5) has no carrier yet — PDR-0010
(structured errors) is **Proposed**; do not build against it (rule 5). The tree's
existing kind mechanism is a **dedicated error class**:
`CannotYieldAcrossNativeFrame < Error` (`core_classes.rs:160`) is the precedent. Add
`UseAfterCloseError < Error` the same way (bootstrapped class + `add_class!` row). The
diagnostic message names the resource kind and the site that closed it (PDR-0005 §4).
When PDR-0010 ratifies, the class gains a `kind` symbol; nothing here migrates.

### 2.5 Exit-time reporting

At VM teardown (the `cmd_run` epilogue — the same seam PDR-0008 wired reporting
through): if `leaks()` is non-empty, render one diagnostic per row (kind + open site).
`strictResources(true)` escalates report → raise **before** exit-success is decided;
test lanes set it (stream-protocol §7). A `BufferedWriter`-abandoned-dirty distinct
condition is **not** this unit's (U-STREAMS adds it); leave the rendering extensible
(one row type now, an enum later).

## 3. Ordering

1. `resource.rs` (table + tests for generation staleness, idempotent close, pack/unpack).
2. Kernel classes (`Resource`, `UseAfterCloseError`) + all four registration sites
   (create / `CoreClasses` / verify row / `add_class!`). Boot green.
3. Primitives + bindings + census (`NEW_RESOURCE`, class rows for both new classes).
4. core.ph `.ph` layer + harness `.ph` subclass.
5. Golden lanes (§5). Clean-worktree verify.

## 4. What must NOT happen

- No `Drop` impl anywhere that touches an OS handle or the table (ADR-0050's
  "no finalizers" premise, PDR-0005 §1).
- No `Value` inside the table module.
- `close` never returns `Future`, never blocks, never flushes (stream-protocol §3.1
  laws 1-2).
- Use-after-close never returns `Err` — it **raises** (law 5's channel split).
- No `using` sugar (PDR-0005 §6, withdrawn).

## 5. Test plan

| Lane | Content |
|---|---|
| Rust unit (`resource.rs`) | open/resolve/close/idempotent-close; stale generation errors; pack/unpack boundary values (`index` 0 and `u32::MAX`, generation rollover posture documented); `drain` closes everything once |
| golden positive (`resources/`) | open test-resource → `close` → `Ok`; double `close` → `Ok`; `isClosed` flips; `Result` shape of `close` |
| golden negative (`resources/negative/`) | use-after-close raises with `UseAfterCloseError` in the diagnostic; `strictResources(true)` + a leaked resource exits non-zero naming the open site |
| leak report | a program that leaks one resource prints the report at exit (positive lane, exact stdout) |
| invariants | census constant + both class rows; `verify_invariants` green |

## 6. Not in this unit

`File`/fd-backed kinds (U-FS), `BufferedWriter`'s dirty-abandon report row (U-STREAMS),
reactor registration tokens (U-REACTOR §7 — same discipline, separate table), PDR-0010
migration.
