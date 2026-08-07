# Resource v2 Test Specification

> **Status:** companion merge-gating test specification for `resource-v2-implementation-spec.md`.
>
> **Purpose:** prove lifecycle safety, exact handle encoding, idempotent close, stale-handle isolation, diagnostics, bootstrap integration, leak behavior, and Streams migration without making allocator internals part of the language contract.

## 1. Testing philosophy

Resource is kernel infrastructure. A superficially correct happy-path test is insufficient because failures here can silently redirect an old object to a new native resource, close a reused OS handle, hide leaks, or emit diagnostics pointing at runtime wrappers instead of user code.

The suite therefore has four layers:

1. **White-box Rust unit tests** for exact encoding and the ResourceTable state machine.
2. **VM/primitive integration tests** for hidden-slot behavior, errors, and user-frame attribution.
3. **Process/golden tests** for stderr, exit status, and shutdown ordering.
4. **Downstream regression tests** proving Streams still obey the Resource contract after migration.

Tests MUST verify semantic outcomes, not incidental allocator behavior. In particular, user-visible tests MUST NOT assume which closed slot the allocator reuses unless a test-only white-box table fixture explicitly controls the slot.

## 2. Required test fixtures

### 2.1 Test-only native close probe

Under `#[cfg(test)]`, provide a ResourceKind or ResourceTable fixture whose close action increments a Rust-side counter and can optionally return a controlled failure.

Conceptual shape:

```rust
struct CloseProbe {
    close_count: Arc<AtomicUsize>,
    fail: bool,
}
```

This fixture exists only to prove:

- native close executes exactly once;
- a second language/table close does not retry it;
- a close failure does not restore the row to `Open`;
- drain does not re-close an already closed row.

It MUST NOT become a production Phalcom capability.

### 2.2 User-level managed Resource fixture

Use an existing in-memory Resource subclass after Streams migration where practical. If a dedicated fixture is still needed, define a test-only `.ph` subclass that:

```phalcom
// Pseudocode; adapt to actual syntax.
class TestResource < Resource {
    new() {
        self.attach_("TestResource")
        return self
    }

    touch() {
        self.ensureOpen_
        return None
    }
}
```

The constructor MUST NOT declare or assign `_handle`.

### 2.3 Process harness

Leak and strict-mode behavior must be tested through the same command-runner path users execute, because the contract covers:

- stderr versus stdout;
- final exit status;
- ordering relative to normal runtime errors;
- leak snapshot before drain.

A pure in-process VM assertion is not sufficient for those cases.

## 3. Handle encoding unit tests

Target: Resource handle helper tests in or adjacent to `phalcom-core/src/resource.rs`.

### R-HANDLE-001 — minimum handle

```rust
let h = ResourceHandle { index: 0, generation: 0 };
assert_eq!(unpack(pack(h)?), h);
```

Expected: exact round-trip; packed `Number` is exactly `0.0`.

### R-HANDLE-002 — maximum valid handle

```rust
let h = ResourceHandle {
    index: (1 << 21) - 1,
    generation: u32::MAX,
};
```

Expected:

```text
packed == 2^53 - 1
unpack(pack(h)) == h
```

No precision loss is permitted.

### R-HANDLE-003 — first invalid index

Attempt to pack:

```rust
ResourceHandle {
    index: 1 << 21,
    generation: 0,
}
```

Expected: rejected before conversion to language `Number`.

### R-HANDLE-004 — above exact range

Unpack numeric value:

```text
2^53
```

Expected: malformed/rejected. The implementation MUST NOT round or reinterpret it as a valid index/generation pair.

### R-HANDLE-005 — negative number

Input: `-1.0`.

Expected: malformed/rejected.

### R-HANDLE-006 — fractional number

Input: `1.5`.

Expected: malformed/rejected.

### R-HANDLE-007 — NaN

Input: `NaN`.

Expected: malformed/rejected without panic.

### R-HANDLE-008 — positive infinity

Input: `+∞`.

Expected: malformed/rejected without panic.

### R-HANDLE-009 — negative infinity

Input: `-∞`.

Expected: malformed/rejected without panic.

### R-HANDLE-010 — representative bit-boundary corpus

Round-trip at least these combinations:

```text
index:      0, 1, 2^20, 2^21 - 1
generation: 0, 1, 2^31, 2^32 - 1
```

Expected: every Cartesian-product member round-trips exactly.

## 4. ResourceTable lifecycle unit tests

### R-TABLE-001 — new allocation opens generation zero

Allocate into an empty table.

Expected:

```text
index == first valid slot
generation == 0
state == Open
leak count == 1
```

The exact initial index may be asserted only in the isolated white-box table fixture.

### R-TABLE-002 — open resolves as open

Given a matching live handle, `ensure_open` succeeds and `is_closed` is false.

### R-TABLE-003 — explicit close installs tombstone

Close a matching live handle.

Expected:

```text
state == Closed
generation unchanged
is_closed == true
native close count == 1
```

### R-TABLE-004 — double close is idempotent

Close the same matching handle twice.

Expected:

```text
first close  -> success
second close -> success
native close count == 1
generation unchanged
```

This test is merge-critical. It must fail if generation is incremented on close.

### R-TABLE-005 — close failure is not retried

Use a close probe configured to return a controlled failure.

Expected:

```text
first close  -> failure outcome
state        -> Closed
second close -> success/no-op
close count  -> 1
```

The row MUST NOT return to `Open` after failure.

### R-TABLE-006 — closed handle rejects operation

After close, call `ensure_open` with the same generation.

Expected internal error classification:

```text
reason == Closed
kind/open/close metadata preserved
```

### R-TABLE-007 — closed slot reuse increments generation

In a controlled white-box allocator fixture:

1. allocate A at generation `g`;
2. close A;
3. force/select the same reusable slot for B.

Expected:

```text
B.generation == g + 1
B.state == Open
```

### R-TABLE-008 — old handle is stale after reuse

Continue R-TABLE-007 and query A's old handle.

Expected:

```text
ensure_open(A) -> Stale
close(A)       -> Stale
is_closed(A)   -> true
```

The new occupant B remains open and untouched.

### R-TABLE-009 — stale handle never aliases new occupant

Give A and B distinguishable kind names/close probes.

Attempt close through stale A.

Expected:

```text
B close count == 0
B remains Open
error reason == Stale
```

This is the primary protection against use-after-reuse.

### R-TABLE-010 — maximum generation closes to terminal retirement

Construct/seed an open slot at `generation == u32::MAX`, then close it.

Expected:

```text
state == Retired
matching handle isClosed == true
matching handle close again succeeds
matching handle ensure_open -> Closed
native close count == 1
```

### R-TABLE-011 — retired slot is never reused

After R-TABLE-010, allocate another resource.

Expected: allocation uses another slot or fails because capacity is exhausted. It MUST NOT reopen the retired slot or wrap generation to zero.

### R-TABLE-012 — retired tombstone preserves idempotence

Close the matching retired handle repeatedly.

Expected: every repeated close succeeds/no-ops; no extra native close occurs.

### R-TABLE-013 — malformed table index classification

Construct a validly shaped internal handle whose index is outside the current table length.

Expected: malformed/internal-invalid classification, never panic/out-of-bounds access.

### R-TABLE-014 — matching Vacant classification

If `Vacant` can occur in a materialized slot, query a matching handle.

Expected: malformed/internal-invalid classification, not `Open` and not user-closed.

### R-TABLE-015 — leak snapshot contains only Open

Create rows in states:

```text
Open A
Closed B
Retired C
Open D
```

Expected leak snapshot contains exactly A and D.

### R-TABLE-016 — leak snapshot ordered by serial, not slot

Create/reuse slots so table-index order differs from allocation order.

Expected snapshot order follows ascending `serial`.

### R-TABLE-017 — leak snapshot is immutable after capture

Capture snapshot, then mutate/drain table.

Expected captured rows remain valid Rust-owned data and do not borrow invalidated table storage.

### R-TABLE-018 — `leak_snapshot` has no side effects

Call `leak_snapshot` twice.

Expected identical rows; table state unchanged; later shutdown still reports them.

### R-TABLE-019 — drain closes every remaining Open row once

Create several open close probes; call `drain`.

Expected every probe count becomes exactly `1`, and no previously closed probe increments again.

### R-TABLE-020 — drain continues after close failure

Arrange reverse-serial drain order where the middle resource reports close failure.

Expected later resources are still closed. Drain returns/records the failure without aborting the loop.

### R-TABLE-021 — drain order is reverse serial

Use close probes that record their close sequence.

Allocate A, B, C.

Expected drain order:

```text
C, B, A
```

### R-TABLE-022 — production table contains no Phalcom heap roots

Add a structural/compiler-facing guard appropriate to the repository. At minimum, review/invariant coverage must ensure production Resource table types have no fields of `Value`, `ObjRef`, or equivalent GC-reference types.

If a compile-time trait/invariant can encode this property without disproportionate complexity, prefer it; otherwise retain an explicit unit/invariant review guard.

## 5. Attachment and hidden-slot integration tests

### R-ATTACH-001 — unattached instance is inert

Construct a Resource subclass instance before attachment through a controlled test path.

Expected:

```text
isClosed == true
close     == Ok(None)
```

An operational method using `ensureOpen_` raises `UseAfterCloseError`.

### R-ATTACH-002 — attach opens and writes hidden slot

Attach a valid managed kind.

Expected:

```text
return == None
isClosed == false
exactly one Open row exists
hidden slot contains valid packed Number
```

No source-declared `_handle` field is required.

### R-ATTACH-003 — duplicate attach is rejected

Call `attach_` twice on the same receiver.

Expected: second call raises the existing primitive/state contract error. It MUST NOT create a second table row and MUST NOT replace the original handle.

### R-ATTACH-004 — invalid kind type

Pass non-String value.

Expected: normal primitive type error; no table row created; receiver remains unattached.

### R-ATTACH-005 — empty kind rejected

Expected: contract error; no row created.

### R-ATTACH-006 — oversized kind rejected

Use a UTF-8 string whose encoded length exceeds 128 bytes.

Expected: contract error; no row created.

### R-ATTACH-007 — control characters rejected

Test at least newline, carriage return, and one other control character.

Expected: contract error; no row created.

### R-ATTACH-008 — multibyte UTF-8 boundary counted in bytes

Use valid multibyte names immediately below/at the configured byte limit.

Expected: validation follows UTF-8 byte length, not character count, and never splits/corrupts the string.

### R-ATTACH-009 — failed constructor before attach creates no leak

A test constructor performs validation and deliberately fails before `attach_`.

Expected:

```text
leakReport unchanged
process shutdown has no resource leak for that object
```

### R-ATTACH-010 — failure after attach is visible as leak

A test constructor deliberately raises after a successful `attach_`.

Expected: the attached row remains visible to Resource leak detection unless explicit cleanup runs. This test documents why attachment must be the final potentially failing acquisition action.

### R-ATTACH-011 — inherited field offsets remain correct

For at least one actual migrated Streams class with its own fields:

1. initialize subclass fields;
2. attach Resource;
3. read/write the subclass fields;
4. close Resource;
5. read non-lifecycle fields that are contractually readable after close, if applicable.

Expected: no collision with hidden slot `0`; subclass fields begin at inherited offset `1`.

## 6. Primitive lifecycle behavior tests

### R-PRIM-001 — `isClosed_` matching Open

Expected: `false`.

### R-PRIM-002 — `isClosed_` matching Closed

Expected: `true`.

### R-PRIM-003 — `isClosed_` matching Retired

Expected: `true`.

### R-PRIM-004 — `isClosed_` stale

Expected: `true`, no exception.

### R-PRIM-005 — `isClosed_` malformed

Inject a malformed handle through a test-only VM/object fixture.

Expected: `true`, no exception or panic.

### R-PRIM-006 — `close_` stale

Expected: raises `UseAfterCloseError`; does not touch new occupant.

### R-PRIM-007 — `close_` malformed

Expected: raises `UseAfterCloseError` with malformed semantic classification.

### R-PRIM-008 — `ensureOpen_` Open

Expected: returns `None`.

### R-PRIM-009 — `ensureOpen_` Closed

Expected: raises `UseAfterCloseError` with closed semantic classification.

### R-PRIM-010 — `ensureOpen_` stale

Expected: raises `UseAfterCloseError` with stale semantic classification.

### R-PRIM-011 — `ensureOpen_` malformed

Expected: raises `UseAfterCloseError` with malformed semantic classification.

### R-PRIM-012 — `.ph` `close` result shape

For an infallible managed resource:

```text
first close  -> Ok(None)
second close -> Ok(None)
```

The return is synchronous; it is not a Future.

## 7. Diagnostic attribution tests

The tests in this section MUST assert user source locations, not merely the error class name.

### R-DIAG-001 — open site points to user constructor call

User program:

```phalcom
const r = TestResource.new()
```

Expected leak/open diagnostic identifies the user program line that caused construction/acquisition, not the internal `self.attach_` line in `core.ph` or another installed core module.

### R-DIAG-002 — close site points to user close call

User program:

```phalcom
const r = TestResource.new()
r.close
r.touch()
```

Expected `UseAfterCloseError` includes the user line containing `r.close` as close site.

### R-DIAG-003 — attempt site points to user operation

Same program.

Expected attempted-use site identifies `r.touch()` in the user file, not `ensureOpen_` wrapper internals.

### R-DIAG-004 — closed error semantic content

For a matching closed tombstone, assert diagnostic contains:

```text
UseAfterCloseError
resource kind: TestResource
open site: user file/location
close site: user file/location
attempt site: user file/location
```

The exact punctuation/glyph style may follow the repository's diagnostic renderer, but all semantic fields are required.

### R-DIAG-005 — stale error does not borrow new occupant metadata

Create stale A and live B in the same slot using a white-box VM/test fixture.

Expected stale diagnostic MUST NOT claim B's kind/open site as A's history.

### R-DIAG-006 — malformed error points to attempted use

Inject malformed packed handle, call guarded operation.

Expected malformed semantic classification plus user attempted-use site; no crash.

### R-DIAG-007 — native/no-user-frame fallback

Invoke Resource operation from a controlled native/test context with no user frame.

Expected a stable repository-standard native/unknown location rather than panic or invalid source range.

## 8. Public leak-report API tests

### R-LEAKAPI-001 — empty snapshot

With no open resources:

```phalcom
System.leakReport
```

Expected: empty `List`.

### R-LEAKAPI-002 — one open resource

Expected: one `String` containing managed kind and compact open site.

### R-LEAKAPI-003 — only open resources included

Open A and B; close A.

Expected list contains B only.

### R-LEAKAPI-004 — deterministic allocation order

Allocate A, B, C.

Expected list rows are A, B, C by allocation serial.

### R-LEAKAPI-005 — repeated call has no side effects

Call `System.leakReport` twice before close.

Expected identical results; resource remains open; shutdown still reports it.

### R-LEAKAPI-006 — leakReport does not change strict mode

Set strict false, call leakReport, leak resource.

Expected normal-mode exit behavior remains unchanged.

### R-LEAKAPI-007 — report uses user location

Expected row names user acquisition site rather than `core.ph`.

## 9. Process shutdown and strict-mode tests

These tests MUST capture `stdout`, `stderr`, and process exit status independently.

### R-SHUTDOWN-001 — clean default shutdown

Program creates and explicitly closes all resources.

Expected:

```text
exit status: 0
resource leak stderr: empty
```

### R-SHUTDOWN-002 — non-strict leak is visible

Program leaks one resource with strict mode false/default.

Expected:

```text
exit status: 0
stderr: contains one resource leak diagnostic
stdout: contains no leak diagnostic unless user program printed it itself
```

### R-SHUTDOWN-003 — strict leak fails otherwise successful run

Program enables strict resources and leaks one resource.

Expected:

```text
exit status: non-zero
stderr: leak diagnostic present
```

### R-SHUTDOWN-004 — strict clean run succeeds

Program enables strict resources and closes all resources.

Expected exit `0`, no resource leak diagnostic.

### R-SHUTDOWN-005 — multiple leak ordering

Allocate A, B, C and leak all three.

Expected shutdown diagnostics appear in allocation-serial order A, B, C.

### R-SHUTDOWN-006 — shutdown reports before drain destroys evidence

Use close probes or harness instrumentation to show:

```text
leak snapshot/render occurs while rows are Open
drain then closes them exactly once
```

The test MUST fail if drain occurs first and suppresses the report.

### R-SHUTDOWN-007 — drain reverse order

Allocate A, B, C and abandon them.

Expected native close probe order during drain: C, B, A.

### R-SHUTDOWN-008 — strict leak does not replace primary runtime error

Program:

1. enables strict resources;
2. leaks a resource;
3. raises an unrelated user runtime error.

Expected:

- process is non-zero for the pre-existing runtime failure;
- the user's runtime error/traceback remains the primary diagnostic;
- the resource leak is additionally reported on stderr;
- no second `UseAfterCloseError`/generic strict-resource exception replaces the primary failure.

### R-SHUTDOWN-009 — non-strict leak also preserves primary runtime error

Same as above with strict false.

Expected primary error unchanged; leak still reported.

### R-SHUTDOWN-010 — `strictResources` argument validation

Pass non-Bool argument.

Expected existing type error; strict flag unchanged.

### R-SHUTDOWN-011 — `strictResources` setter return

Assert the exact setter return chosen by the implementation spec (`None`, unless repository-wide setter conventions require the already-established alternative).

This must be frozen once against the current convention and then kept stable.

## 10. Use-after-close golden programs

Create explicit golden fixtures under the repository's current runtime-error test hierarchy.

### R-GOLDEN-001 — operation after close

```phalcom
const r = TestResource.new()
r.close
r.touch()
```

Expected class: `UseAfterCloseError`.

Expected diagnostic semantics:

```text
reason: closed
kind: TestResource
open: user location
close: user location
attempt: user location
```

### R-GOLDEN-002 — double close positive

```phalcom
const r = TestResource.new()
assert(r.close.ok)
assert(r.close.ok)
assert(r.isClosed)
```

Expected success.

### R-GOLDEN-003 — `isClosed` transition

```text
before close -> false
after close  -> true
```

### R-GOLDEN-004 — unattached/inert close

Through a controlled test fixture, close an unattached Resource.

Expected `Ok(None)`.

### R-GOLDEN-005 — use of unattached Resource

Expected `UseAfterCloseError`; diagnostic may describe it as closed/unattached and need not claim an open or close site that never existed.

## 11. Bootstrap and census tests

### R-BOOT-001 — Resource kernel class exists

Verify `Resource` is present in `CoreClasses`, inherits from the intended root, and has bootstrap field count `1`.

### R-BOOT-002 — UseAfterCloseError exists

Verify class registration and inheritance from `Error`.

### R-BOOT-003 — primitive bindings exactly match v2

Required:

```text
Resource#attach_(_)
Resource#close_
Resource#isClosed_
Resource#ensureOpen_
System.leakReport_
System.strictResources_(_)
```

Forbidden legacy binding:

```text
Resource.register_(_)
```

### R-BOOT-004 — census delta

Verify Resource primitive census count is six under the repository's current census scheme.

### R-BOOT-005 — invariant verifier green

The repository's global universe/bootstrap invariant verifier must pass with Resource and UseAfterCloseError rows installed.

## 12. Streams migration regression tests

Run the complete existing Streams suite after migration. Add targeted guards where existing tests do not prove the Resource v2 seam.

### R-STREAM-001 — BytesReader attaches without source `_handle`

Construct and use `BytesReader`; verify normal reads and field behavior.

### R-STREAM-002 — BytesWriter attaches without source `_handle`

Construct/write/close; verify normal behavior.

### R-STREAM-003 — stream double close

Every Resource-backed stream type that exposes close must preserve idempotence.

### R-STREAM-004 — stream operation after close uses common error path

Close a managed stream, then perform an operation requiring liveness.

Expected: `UseAfterCloseError` populated by Resource diagnostics, not an ad hoc stream-created generic error.

### R-STREAM-005 — stream `isClosed` observation

Expected false before close, true after close.

### R-STREAM-006 — stream leak attribution

Leak a managed stream.

Expected shutdown report names the user construction site, not its internal `attach_` call.

### R-STREAM-007 — inherited field-layout regression

Exercise each migrated Resource-backed stream class's ordinary fields sufficiently to detect a hidden-slot offset collision.

At minimum include the actual concrete stream classes that currently inherit Resource; a synthetic subclass alone is insufficient.

### R-STREAM-008 — no Resource regression masks independent Streams failures

Retain/execute the existing tests for buffered short-write behavior, overlap policy, and dirty-buffer semantics according to their own specification. Resource v2 must not "fix" those concerns by changing lifecycle behavior.

## 13. FS/reactor preparation tests

Resource v2 does not implement external adoption, so these are boundary tests rather than File tests.

### R-BOUNDARY-001 — managed attach cannot mint external kind by name

Call `attach_` with strings such as `"File"`, `"TcpStream"`, or arbitrary names.

Expected: at most a managed diagnostic row is created. No fd/socket/native capability is acquired or selected based on the string.

### R-BOUNDARY-002 — worker threads do not access ResourceTable

Enforce by code review/invariant/architecture test appropriate to the repository: current ResourceTable methods are VM-thread-owned and are not exposed as thread-safe worker APIs.

### R-BOUNDARY-003 — no production external ResourceKind added by this unit

Until the FS/reactor ownership spec lands, the production Resource kind inventory contains only managed/in-memory kinds introduced by the current implementation.

## 14. Negative robustness tests

### R-ROBUST-001 — table capacity boundary helper

White-box test the allocation branch at `RESOURCE_MAX_SLOTS` without actually allocating millions of heavyweight resources; inject/fake table length/free-list state if necessary.

Expected: acquisition fails cleanly, never wraps index.

### R-ROBUST-002 — serial overflow branch

White-box seed the next allocation serial at `u64::MAX`.

Expected: allocation fails deterministically; serial does not wrap to zero.

### R-ROBUST-003 — malformed hidden-slot type

Inject a non-Number, non-`None` value into the hidden slot through a test-only object fixture.

Expected:

```text
isClosed -> true
ensureOpen/close -> controlled malformed/contract error path
```

No panic or unsafe cast.

### R-ROBUST-004 — malformed packed Number cannot index table

Use a large/fractional/NaN value and instrument that no table index operation occurs before numeric validation.

### R-ROBUST-005 — stale close cannot mutate leak set of new occupant

After slot reuse, stale close A must leave live B in subsequent leak snapshot.

## 15. Test placement

Use the repository's current conventions, but keep the separation clear:

```text
phalcom-core/src/resource.rs
    white-box encoding/table tests

phalcom-core/src/primitive/resource.rs
    primitive argument/receiver mapping tests where local unit tests exist

phalcom-core/tests/...
    VM integration and golden language programs

command-runner/process tests
    stderr + exit status + primary-error preservation

universe/bootstrap invariant tests
    classes, field stamp, primitive census
```

If the current test corpus separates positive language tests, runtime-error goldens, and CLI/process tests, place each case in the corresponding existing lane rather than creating a parallel testing framework.

## 16. Assertions that are intentionally not language contracts

Do **not** freeze these implementation details in user-visible/golden tests:

- which closed slot is selected for reuse;
- the exact integer value of a hidden packed handle;
- the internal `serial` value;
- free-list data structure;
- `Vec` capacity or table memory layout;
- Rust enum discriminants;
- exact diagnostic punctuation/colors if the repository's renderer treats them as presentation details.

Do freeze:

- semantic error class/reason;
- required diagnostic sites and kind names;
- stdout/stderr channel;
- exit status;
- ordering of leak rows;
- exactly-once close behavior;
- generation safety;
- presence/absence of the six primitive bindings.

## 17. Merge gate

Resource v2 MUST NOT be considered complete until all of the following are green:

1. handle encoding boundary suite;
2. lifecycle/state-machine suite;
3. exactly-once close probe suite;
4. retired-generation/idempotence suite;
5. hidden-slot/attachment suite;
6. closed/stale/malformed diagnostic suite;
7. user-frame source-attribution suite;
8. leak-report side-effect and ordering suite;
9. CLI stderr/strict-exit/primary-error-preservation suite;
10. bootstrap/class/census invariants;
11. complete existing Streams suite after migration;
12. standard repository build/test/lint verification gate.

The key acceptance criterion is stronger than "tests pass": after this unit lands, no reachable state may allow an old Resource object to operate on or close a resource acquired by a later object through table-slot reuse.
