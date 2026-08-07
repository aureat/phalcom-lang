# Resource v2 Implementation Specification

> **Status:** proposed replacement for the already-implemented `resource-implementation-spec.md`.
>
> **Purpose:** reconcile the shipped Resource substrate with the lifecycle semantics required by Streams and future external resources such as files and sockets, without introducing GC finalization or reactor ownership into the Resource layer.
>
> **Normative posture:** this document supersedes the original U-RESOURCE design where the two disagree. Existing public behavior that remains sound is preserved; implementation details that made idempotent close, stale-handle detection, or diagnostics unsound are replaced.

## 1. Executive contract

`Resource` is the VM's deterministic ownership substrate for objects whose lifetime must not be delegated to garbage collection. A Resource instance owns no native handle directly. Instead, it contains one hidden inherited slot holding either `None` or a generation-tagged numeric handle into a VM-owned resource table.

The Resource layer MUST provide all of the following properties simultaneously:

1. **Explicit close.** Resource lifetime is ended by `close`, never by a heap-object finalizer.
2. **Idempotent close.** Closing the same still-identifiable resource more than once succeeds and performs the native close action at most once.
3. **Stale-handle protection.** Reuse of a resource-table slot cannot make an old object operate on a newly allocated resource.
4. **No GC-visible native ownership.** The resource table MUST NOT contain Phalcom `Value`s, heap object references, or other GC roots.
5. **Leak visibility.** Open resources remaining at orderly VM shutdown are reported deterministically; strict mode turns an otherwise successful run into failure.
6. **Diagnostic attribution.** Resource-open, explicit-close, and attempted-use diagnostics identify user code rather than wrapper code in `core.ph`.
7. **Downstream composability.** Streams may use managed Resource rows immediately. FS/reactor integration must use a distinct external-resource adoption seam and must define pending-operation ownership before external handles are added.

The design intentionally follows the durable cross-language convergence seen in Python context managers, Java `AutoCloseable`/try-with-resources, Lua to-be-closed variables, Node's move away from GC-driven file closure, and Rust's explicit ownership model: deterministic cleanup is a correctness mechanism; GC cleanup is, at most, a diagnostic fallback and MUST NOT be required for correctness.

## 2. Non-goals

This unit does **not** introduce:

- filesystem resources or file descriptors;
- reactor job ownership or cancellation;
- asynchronous `Resource#close`;
- a `using`, `with`, RAII, or scope-cleanup syntax;
- heap finalizers or `Drop` behavior that reaches into the resource table;
- a general-purpose public API for fabricating external native resources;
- a new generic structured-error carrier solely for Resource.

Future structured cleanup must be compatible with the lifecycle rules in §16, but its syntax and error-composition mechanism are outside this unit.

## 3. Runtime representation

A live language-level Resource is represented by two pieces of state:

```text
Resource instance
    hidden inherited slot 0
        |
        v
exact integer-valued Number
        |
        v
ResourceHandle { index, generation }
        |
        v
VM::resources[index]
```

The Resource instance's hidden slot begins as `None`. `None` means **unattached/inert**: the object does not own a resource-table row. An attached object stores one packed handle and never mutates that handle again. Slot reuse is represented only by the table generation; an old object therefore naturally becomes stale if its table slot is reused.

### 3.1 Hidden field layout

`Resource` is a bootstrapped kernel class with one inherited hidden field. The implementation MUST define one constant rather than duplicating a magic integer:

```rust
const RESOURCE_HANDLE_SLOT: usize = 0;
```

The bootstrap field count for `Resource` MUST be exactly `1`. Subclass-declared fields append after this inherited slot, so the first source-declared field in a `Resource` subclass occupies slot `1`.

The hidden handle slot is runtime infrastructure, not a source-declared user field:

- `.ph` subclasses MUST NOT declare `_handle` themselves;
- ordinary field access MUST NOT be required to read or write it;
- Resource primitives read/write slot `0` directly through the existing instance-layout machinery;
- reflective field enumeration, if it only exposes source-visible fields today, MUST NOT begin exposing this hidden slot as a new language feature.

At minimum, the invariant suite MUST prove the actual stream subclasses continue to receive correct inherited offsets after the field stamp changes.

## 4. Handle encoding

Phalcom currently stores the Resource handle in a `Number`, so the encoding MUST remain inside the exact integer range of IEEE-754 binary64.

### 4.1 Bit layout

```text
bits  0..31   generation (32 bits)
bits 32..52   table index (21 bits)
bits 53+      unused; any value requiring these bits is invalid
```

Normative constants:

```rust
const RESOURCE_INDEX_BITS: u32 = 21;
const RESOURCE_GENERATION_BITS: u32 = 32;
const RESOURCE_MAX_SLOTS: u32 = 1 << RESOURCE_INDEX_BITS;      // 2^21
const RESOURCE_MAX_INDEX: u32 = RESOURCE_MAX_SLOTS - 1;
const RESOURCE_MAX_HANDLE: u64 = (1u64 << 53) - 1;
```

Encoding:

```rust
packed = ((index as u64) << 32) | generation as u64;
```

The exactness proof is part of the contract:

```text
max packed
= ((2^21 - 1) * 2^32) + (2^32 - 1)
= 2^53 - 1
```

Every integer in `[0, 2^53 - 1]` is exactly representable by binary64, therefore all valid Resource handles round-trip exactly through `Number`.

### 4.2 Validation

Packing MUST reject an index greater than `RESOURCE_MAX_INDEX`.

Unpacking a language-level `Number` MUST reject all of the following before table lookup:

- NaN;
- positive or negative infinity;
- negative values;
- fractional values;
- values greater than `RESOURCE_MAX_HANDLE`;
- any conversion that cannot be represented exactly by the internal integer form.

A malformed encoded value is never truncated, rounded, wrapped, or modulo-reduced.

The pure Rust pack/unpack helpers SHOULD use an integer intermediate (`u64`) and convert to/from `f64` only at the language boundary.

## 5. Resource-table state model

The original `closed: bool` representation is replaced by an explicit state machine. The generation belongs to the slot, not the state payload.

Recommended shape:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceHandle {
    pub index: u32,
    pub generation: u32,
}

pub struct ResourceSlot {
    pub generation: u32,
    pub state: ResourceState,
}

pub enum ResourceState {
    Vacant,
    Open(OpenResource),
    Closed(ClosedResource),
    /// A terminal closed tombstone at generation == u32::MAX.
    /// It is never reused, but a matching old handle is still recognized as
    /// closed so repeated close remains idempotent forever.
    Retired(ClosedResource),
}

pub struct OpenResource {
    pub kind: ResourceKind,
    pub open_site: Option<SourceRange>,
    pub serial: u64,
}

pub struct ClosedResource {
    pub kind_name: Box<str>,
    pub open_site: Option<SourceRange>,
    pub close_site: Option<SourceRange>,
    pub serial: u64,
}
```

`ResourceKind` MUST contain only Rust-owned/native state. It MUST NOT contain `Value`, `ObjRef`, an `InstanceObject`, a heap pointer, or any other GC root.

For this unit the production kind set is intentionally small:

```rust
pub enum ResourceKind {
    Managed { name: Box<str> },
    // External native variants are added by later units only after their
    // operation-ownership contract is specified.
}
```

A test-only close-probe variant MAY exist under `#[cfg(test)]` to prove exactly-once close behavior.

### 5.1 State transitions

A newly appended slot begins at generation `0` and becomes open immediately:

```text
new slot
    -> Open(g = 0)
```

A reusable closed slot advances its generation exactly once when allocated again:

```text
Open(g < MAX)
    -- explicit close --> Closed(g)

Closed(g < MAX)
    -- slot reuse --> Open(g + 1)
```

The generation does **not** change on close. That rule is what allows both idempotent close and stale-handle detection to hold simultaneously.

Generation exhaustion is terminal without sacrificing idempotence:

```text
Open(MAX)
    -- close --> Retired(MAX)

Retired(MAX)
    -- allocation --> never
```

A matching handle for `Retired(MAX)` is still treated as a matching closed handle by `close`, `isClosed`, and `ensureOpen`. The slot is merely unavailable for future allocation.

The implementation MUST NOT wrap `u32::MAX` to generation `0`. Ancient stale handles must never become valid again.

### 5.2 Slot allocation policy

The allocator MAY choose any closed reusable slot; slot-reuse order is not a language-level guarantee. Tests MUST NOT depend on a particular free-list order unless they are white-box ResourceTable tests.

If no reusable slot exists and the table already contains `RESOURCE_MAX_SLOTS` slots, resource acquisition fails through the VM's existing runtime/primitive failure mechanism. It MUST NOT silently wrap the table index or overwrite a live/retired slot.

Allocation serials are independent of slot indices. `serial` is a monotonically increasing `u64` used only to make diagnostics deterministic across slot reuse. Serial increment MUST be checked; it must never wrap. Exhaustion is an unrecoverable resource-allocation failure rather than an ordering reset.

## 6. Table operations

The table SHOULD expose narrow operations rather than one overloaded `resolve` routine. At minimum:

```rust
impl ResourceTable {
    fn open_managed(
        &mut self,
        name: Box<str>,
        open_site: Option<SourceRange>,
    ) -> Result<ResourceHandle, ResourceTableError>;

    fn close(
        &mut self,
        handle: ResourceHandle,
        close_site: Option<SourceRange>,
    ) -> Result<CloseOutcome, ResourceAccessError>;

    fn is_closed(&self, handle: ResourceHandle) -> bool;

    fn ensure_open(
        &self,
        handle: ResourceHandle,
        attempt_site: Option<SourceRange>,
    ) -> Result<(), ResourceAccessError>;

    fn leak_snapshot(&self) -> Vec<ResourceLeak>;

    fn drain(&mut self) -> Vec<DrainFailure>;
}
```

The exact Rust naming may follow repository conventions, but the semantic separation is normative.

### 6.1 Handle lookup classification

Lookup MUST distinguish:

```text
index outside table                   -> malformed
matching generation + Open           -> open
matching generation + Closed         -> closed
matching generation + Retired        -> closed terminal tombstone
different generation                 -> stale
matching generation + Vacant         -> malformed/internal-invalid
```

A stale lookup MUST NOT report metadata from the newly occupying resource as though it belonged to the old handle.

## 7. Exactly-once close semantics

`Resource#close` is synchronous and idempotent. It never returns a `Future`, never flushes buffered data, and never waits for reactor work.

For a matching open resource, close performs a logical ownership transition **before** invoking kind-specific close code:

1. Validate and resolve the handle.
2. Move the `ResourceKind` out of the `Open` state.
3. Materialize the stable diagnostic kind name.
4. Install `Closed` or `Retired` tombstone metadata immediately.
5. Invoke the kind-specific native close action exactly once.
6. Return the close failure, if any, without restoring `Open` state.

This ordering is deliberate. A native close failure does not imply the underlying OS/native handle remains safe to retry. Repeated language-level close therefore never performs a second native close attempt.

For the managed in-memory Resource kinds shipped by this unit, native close is a no-op and cannot fail. The primitive return shape remains future-compatible with external kinds:

```text
close_ returns:
    None            close succeeded, or resource was already closed/unattached
    Error instance  native close was attempted once and reported failure

close_ raises:
    UseAfterCloseError for stale or malformed handles
    existing primitive contract errors for invalid receiver/internal misuse
```

A matching `Closed` or `Retired` handle returns `None` and does nothing.

An unattached Resource whose hidden slot is `None` also closes successfully with no effect. This keeps partially constructed objects inert and makes constructor failure before attachment safe.

## 8. Primitive surface

Resource v2 installs exactly six Resource-related primitives:

| Primitive | Binding | Contract |
|---|---|---|
| `resource_attach` | `Resource#attach_(_)` | atomically attach a managed table row to an unattached Resource instance |
| `resource_raw_close` | `Resource#close_` | exactly-once close transition; bare `None`/failure return |
| `resource_raw_is_closed` | `Resource#isClosed_` | total Boolean lifecycle query |
| `resource_ensure_open` | `Resource#ensureOpen_` | validate operability and raise rich use-after-close diagnostics |
| `system_leak_report` | `System.leakReport_` | stable side-effect-free snapshot rendered as a `List<String>` |
| `system_strict_resources` | `System.strictResources_(_)` | set strict-resource shutdown mode |

`Resource.register_(_)` is removed and MUST NOT remain as an alternate path.

The primitive census constant for this unit is therefore `NEW_RESOURCE = 6`, unless the repository has since moved to a different census scheme. The binding census and bootstrap invariant rows MUST agree with the actual six bindings.

### 8.1 `Resource#attach_(_)`

`attach_` is an instance primitive. Its String argument is a **diagnostic managed-kind name**, not a capability identifier and not a selector for native resource behavior.

The name MUST:

- be a `String`;
- be non-empty;
- be no more than 128 UTF-8 bytes;
- contain no control characters, including line breaks.

`attach_` MUST reject a receiver that is not a `Resource` instance/subclass or whose hidden slot is already non-`None`. Such misuse is a primitive contract/state error, not `UseAfterCloseError`.

The operation is transactional from language code's point of view:

```text
validate receiver and argument
    -> capture diagnostic open site
    -> allocate/copy all fallible Rust-owned metadata
    -> allocate table row
    -> pack validated handle
    -> write hidden slot 0
    -> return None
```

No fallible language-level work may occur after the table row has been committed but before the object's slot has been written. If table allocation fails, the receiver remains unattached and no live row remains behind.

For managed `.ph` constructors, `self.attach_(...)` MUST be the final potentially failing acquisition action. Constructors SHOULD perform all validation and ordinary field initialization first, then attach, then return.

### 8.2 `Resource#isClosed_`

`isClosed_` is intentionally a total observation API:

| Hidden-slot/table condition | Result |
|---|---:|
| slot is `None` | `true` |
| matching `Open` | `false` |
| matching `Closed` | `true` |
| matching `Retired` | `true` |
| stale generation | `true` |
| malformed encoded handle | `true` |

It MUST NOT be used by operational methods as a substitute for `ensureOpen_`, because it deliberately discards diagnostic context.

### 8.3 `Resource#ensureOpen_`

`ensureOpen_` is the mandatory guard for operations that require ownership of a live resource.

```text
matching Open       -> return None
matching Closed     -> raise UseAfterCloseError(reason = closed)
matching Retired    -> raise UseAfterCloseError(reason = closed)
stale generation    -> raise UseAfterCloseError(reason = stale)
malformed handle    -> raise UseAfterCloseError(reason = malformed)
slot None           -> raise UseAfterCloseError(reason = closed/unattached)
```

Managed stream operations such as `read`, `write`, `flush`, or equivalent lifecycle-sensitive methods MUST call `ensureOpen_` rather than manually branching on `isClosed`.

### 8.4 `.ph` wrappers

The language-level Resource API remains shaped in `.ph`:

```phalcom
// Pseudocode; adapt to current core.ph syntax.
close {
    const failure = self.close_
    if failure == None {
        return Ok.new(None)
    }
    return Err.new(failure)
}

isClosed {
    self.isClosed_
}
```

`Resource#close` therefore remains synchronous and returns `Result`.

`System.leakReport` forwards the list snapshot from `leakReport_`.

`System.strictResources(flag)` requires a Boolean, stores the VM flag, and returns `None` unless current `System` setter conventions require another already-established return value. It MUST NOT silently coerce truthy/falsy values.

## 9. Use-after-close diagnostic model

Resource v2 requires a richer internal diagnostic model, but it MUST NOT create an ad hoc language-level payload representation if the general error-kind carrier is not yet implemented.

Recommended internal data:

```rust
pub enum ResourceAccessReason {
    Closed,
    Stale,
    Malformed,
}

pub struct ResourceAccessError {
    pub reason: ResourceAccessReason,
    pub kind_name: Option<Box<str>>,
    pub open_site: Option<SourceRange>,
    pub close_site: Option<SourceRange>,
    pub attempt_site: Option<SourceRange>,
    pub handle: Option<ResourceHandle>,
}
```

The language-level error class remains:

```text
UseAfterCloseError < Error
```

Until a generic structured error carrier exists, `reason`, kind, and sites are normative semantic data rendered into the error diagnostic; they are **not** required to become new source-visible instance fields solely for this unit.

### 9.1 Diagnostic guarantees by reason

For `closed`, the matching tombstone retains enough information to render:

- stable managed resource kind;
- open site when available;
- explicit close site when available;
- attempted-use site.

For `stale`, the historical row may already have been reused. Resource MUST NOT retain unbounded tombstone history merely to preserve diagnostics. Therefore stale diagnostics guarantee:

- reason `stale`;
- attempted-use site;
- stale handle index/generation information where useful;
- **no guarantee** that the historical kind/open/close sites remain available.

Most importantly, stale diagnostics MUST NOT display the new occupant's kind/open site as though they belonged to the stale object.

For `malformed`, diagnostics guarantee the attempted-use site and a statement that the encoded resource handle is invalid. The runtime SHOULD avoid dumping raw floating-point internals unless useful for debugging and safe under existing diagnostic conventions.

## 10. User-source attribution

The immediate primitive caller is often `core.ph`; reporting that frame would produce poor diagnostics. Resource therefore requires one shared VM helper conceptually equivalent to:

```rust
fn first_user_source_range(&self) -> Option<SourceRange>;
```

It walks outward from the current native call and returns the first source frame not belonging to the installed core/runtime module. The implementation MUST use module/frame identity where available; it MUST NOT rely on brittle filename substring matching such as `path.contains("core.ph")`.

The helper is used consistently for:

- `attach_` open-site attribution;
- explicit `close_` close-site attribution;
- `ensureOpen_` attempted-use attribution.

If no user source frame exists, diagnostics fall back to the nearest available source location or render an existing repository-standard `<unknown>`/native location.

Golden tests MUST prove ordinary user calls point to user program lines, not wrapper lines inside the installed core module.

## 11. Leak model and public report API

A resource leak is an entry whose state is `Open` when the orderly shutdown leak snapshot is taken. `Closed` and `Retired` tombstones are not leaks.

`ResourceTable::leak_snapshot()` returns immutable Rust-owned snapshot rows sorted by allocation `serial` ascending. It MUST NOT expose references whose validity depends on later table mutation.

Conceptual row:

```rust
pub struct ResourceLeak {
    pub kind_name: Box<str>,
    pub open_site: Option<SourceRange>,
    pub serial: u64,
}
```

### 11.1 `System.leakReport`

The existing public surface remains deliberately small: `System.leakReport` returns `List<String>`.

The call is side-effect free:

- it does not close resources;
- it does not mark rows as reported;
- it does not mutate strict mode;
- it does not affect the later shutdown report;
- it returns rows in allocation-serial order.

Each row MUST contain, at minimum, the managed kind name and compact rendered open site. Rendering SHOULD reuse the runtime's normal source-location formatter rather than inventing an absolute-path format inside Resource.

A future richer `ResourceLeak` value type may replace or complement the string API in a separate specification; v2 does not add one.

## 12. Shutdown semantics

Resource v2 defines the Resource-table side of orderly shutdown. It does not own reactor coordination.

The shutdown algorithm is:

```text
1. Preconditions from external subsystems:
   no component may still be able to create or use an external Resource row.

2. Capture leak snapshot, sorted by serial ascending.

3. Render each leak to stderr exactly once for this shutdown pass.

4. Drain remaining Open rows exactly once, in reverse allocation-serial order.
   Continue draining later rows even if one close reports failure.

5. Determine final process status:
   - an already-failing program remains failing for its original reason;
   - if the program was otherwise successful and strictResources == true
     and the snapshot was non-empty, shutdown becomes non-zero;
   - non-strict leaks do not change an otherwise successful exit status.

6. Destroy remaining VM/heap state.
```

The leak snapshot MUST be captured before drain; reporting after drain would erase the evidence.

Leak diagnostics go to `stderr`, never `stdout`.

Strict-resource failure MUST NOT replace an existing runtime error/traceback as the primary failure. If the command runner represents shutdown failures as a Rust result, use a distinct shutdown-status path rather than raising a new language-level exception after the user's primary error.

### 12.1 Drain ordering

Drain uses reverse allocation order (largest `serial` first). This is intentionally compatible with future structured cleanup and with the common dependency pattern where later-acquired resources depend on earlier ones.

Drain is not a GC finalizer. It is an orderly VM teardown operation executed while the ResourceTable is still valid.

## 13. Threading and reactor boundary

The ResourceTable is VM-owned and mutated on the VM thread. This unit does not add locks around table access and does not permit worker threads to resolve language Resource handles directly.

Future FS/reactor work MUST NOT implement asynchronous operations by copying a raw descriptor out of the ResourceTable and allowing the table row to close/reuse independently. Before an external `File`, socket, or similar `ResourceKind` lands, that subsystem must specify:

- pending-operation ownership;
- per-resource operation ordering where the native resource has shared cursor/state;
- close-while-pending semantics;
- cancellation/settlement behavior;
- orphan completion cleanup;
- the point at which underlying native close actually occurs.

`Resource#attach_(_)` is **not** the external-resource seam. It creates managed diagnostic rows only.

External resources require a separate atomic adoption path, conceptually:

```text
external producer creates native handle
    -> VM receives ownership
    -> ResourceTable adopts native state
    -> target Resource instance receives packed handle atomically
```

The exact primitive/API for adoption belongs to U-FS/U-REACTOR and is not exposed by this unit.

## 14. Bootstrap and repository integration

Apply the v2 lifecycle to the current equivalents of the original U-RESOURCE integration points.

### 14.1 `phalcom-core/src/resource.rs`

Replace the old boolean-row table with:

- exact 21/32-bit handle pack/unpack helpers;
- `ResourceSlot` + state payloads;
- checked allocation serial;
- generation-on-reuse logic;
- terminal retired tombstones;
- dedicated `close`, `is_closed`, `ensure_open`, `leak_snapshot`, and `drain` operations;
- test-only native close probe.

Keep the module structurally free of Phalcom `Value`/heap reference fields.

### 14.2 `phalcom-core/src/primitive/resource.rs`

Replace `Resource.register_(_)` with `Resource#attach_(_)` and install `ensureOpen_`.

The primitive module is responsible for:

- extracting/validating the hidden slot;
- translating valid packed `Number` values to `ResourceHandle`;
- capturing user source locations;
- mapping `ResourceAccessError` to `UseAfterCloseError` diagnostics;
- preserving the existing bare-native / `.ph`-wrapper convention.

### 14.3 Core classes and bootstrap

`Resource` and `UseAfterCloseError` remain bootstrapped kernel classes. Update all repository registration/invariant sites that the original implementation required, including the current equivalents of:

- core-class creation;
- `CoreClasses` storage;
- core-class invariant verification;
- bootstrap installation/binding rows;
- primitive census rows.

Stamp `Resource.field_count = 1` before any Resource subclass layout is frozen.

### 14.4 `core.ph`

Update wrappers and managed constructors:

```phalcom
// Old
_handle = Resource.register_("BytesReader")

// New
self.attach_("BytesReader")
```

The subclass does not declare `_handle`.

Operational methods use:

```phalcom
self.ensureOpen_
```

rather than branching on `self.isClosed` to manufacture their own error.

### 14.5 Command-runner shutdown seam

At the current command-runner/VM teardown seam:

- capture and render the leak snapshot before table drain;
- emit to `stderr`;
- preserve the primary program failure;
- apply strict-resource status only to an otherwise successful run;
- drain before destroying VM-owned native state.

## 15. Invariants

The following are merge-gating invariants:

1. `Resource` has exactly one hidden inherited handle slot.
2. Every valid packed handle is an exact binary64 integer no greater than `2^53 - 1`.
3. Slot generation changes only when a reusable closed slot is reopened.
4. Generation never wraps.
5. A terminal-generation resource remains idempotently closable after retirement.
6. Native close executes at most once per resource acquisition.
7. Reusing a slot makes every old-generation handle stale.
8. A stale handle can never resolve to the new occupant.
9. `isClosed` is total and non-throwing for handle-state corruption.
10. Operability checks use `ensureOpen_` and raise `UseAfterCloseError` for closed/stale/malformed state.
11. The table contains no Phalcom heap roots.
12. `System.leakReport` does not mutate lifecycle state.
13. Leak output is deterministic by allocation serial.
14. Shutdown leak rendering occurs before drain and goes to `stderr`.
15. Strict leak handling never replaces an existing primary program error.

## 16. Compatibility with future structured cleanup

This unit does not add structured cleanup syntax, but future scope-based cleanup MUST preserve these semantics:

- cleanup runs on normal and exceptional scope exit;
- resources are closed in reverse acquisition order;
- every scheduled close is attempted even if an earlier close fails;
- an error already escaping the body remains primary;
- cleanup failures are retained as secondary/suppressed information rather than replacing the body failure;
- abandoned fibers/coroutines remain detectable by the VM-global leak table;
- scope cleanup never weakens explicit `close`, stale-handle checks, or leak reporting.

These requirements intentionally preserve compatibility with the lessons established by Python context managers, Java try-with-resources, Lua 5.4 to-be-closed values, and mature explicit-ownership systems.

## 17. Migration requirements for Streams

The already-implemented Streams layer must be migrated as part of Resource v2 integration.

For every managed Resource subclass:

1. remove assignment to a source-visible `_handle` field;
2. perform ordinary constructor validation/field initialization first;
3. make `self.attach_("StableKindName")` the final potentially failing acquisition action;
4. replace operation guards based on `isClosed` with `self.ensureOpen_`;
5. retain `isClosed` only for observation/public querying;
6. rerun stream conformance and field-layout tests.

Resource v2 does not by itself solve BufferedWriter short-write handling, overlap serialization, or dirty-buffer leak metadata. Those remain Streams concerns and must not be hidden inside Resource lifecycle code.

## 18. Preparation requirements for FS

FS MUST NOT add a `File` variant merely by copying a raw descriptor into `ResourceKind` and submitting copied descriptors to background jobs.

Before external adoption is implemented, the FS/reactor specifications must freeze:

```text
File open completion ownership
File object minting/adoption
pending job lease/reference semantics
per-File sequencing
close while jobs are pending
actual native close point
completion cleanup when no File object adopts the result
shutdown coordination before ResourceTable::drain
```

Only after those rules exist may Resource gain an external native kind or adoption primitive.

## 19. Implementation order

Implement in this order so each layer can be verified before the next depends on it:

1. **Handle helpers and white-box tests** — exact packing, malformed-number validation, boundary proof.
2. **ResourceTable rewrite** — state machine, generation-on-reuse, retired tombstones, serial ordering, leak snapshot, drain.
3. **Exactly-once close probe tests** — prove tombstone-before-close and repeated-close behavior.
4. **Bootstrap field stamp and invariants** — `Resource.field_count = 1`, `UseAfterCloseError`, all class/binding rows.
5. **Primitive migration** — remove `register_`, add `attach_` and `ensureOpen_`, update census to six primitives.
6. **User-frame attribution helper** — shared VM helper plus diagnostic tests.
7. **`.ph` wrappers and managed constructor migration** — Streams included.
8. **Command-runner leak/shutdown migration** — stderr, deterministic snapshot, strict exit semantics, drain ordering.
9. **Golden and integration tests** — complete `resource-v2-test-spec.md`.
10. **Clean-worktree verification** — full build, tests, invariants/census, clippy/standard repository verification gate.

## 20. Explicit prohibitions

The implementation MUST NOT:

- add a heap finalizer for Resource;
- add a Rust `Drop` implementation that closes native resources or mutates `VM::resources`;
- store `Value`, `ObjRef`, or heap-owned objects in the ResourceTable;
- increment generation on close;
- wrap a generation counter;
- make `close` asynchronous merely because future FS operations are asynchronous;
- use `isClosed` as the operational use-after-close guard;
- retain unbounded stale-handle history solely for diagnostics;
- let `attach_` create arbitrary external/native capabilities based on its String argument;
- expose raw table indices/generations as a supported language API;
- report leaks only in strict mode;
- drain before capturing the leak snapshot;
- overwrite an existing runtime failure with a strict-resource shutdown error;
- add external Resource kinds until their reactor/job ownership semantics are specified.

## 21. Completion criteria

Resource v2 is complete only when:

- every test in `resource-v2-test-spec.md` passes;
- all existing Resource tests are migrated or intentionally replaced;
- existing Streams tests remain green after constructor/guard migration;
- class/bootstrap invariants and primitive census are green;
- leak diagnostics identify user code;
- default leaks are visible on `stderr` without changing successful exit status;
- strict leaks produce a non-zero status without replacing a pre-existing primary error;
- no production ResourceTable type contains a Phalcom heap root;
- no external FS/reactor behavior is smuggled into this unit.
