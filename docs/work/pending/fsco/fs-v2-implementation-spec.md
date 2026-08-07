# FS v2 Implementation Specification

> **Status:** proposed replacement implementation specification for U-FS.
>
> This document supersedes the pending `fs-implementation-spec.md` where the two
> disagree. It preserves the accepted public filesystem surface unless a downstream
> implementation detail must be tightened to reconcile Resource v2, Streams v2,
> the Path corrective patch, the shipped `Future`/reactor machinery, and the
> completed `Int`/`Float` numeric split.
>
> **Dependencies:**
>
> - U-BYTES plus Bytes v2 hardening;
> - Resource v2;
> - Path corrective patch / closed `OpenMode` semantics;
> - Streams v2;
> - U-REACTOR.
>
> **Critical prerequisite:** U-REACTOR's completion-drain and shutdown rules must
> accept the owned-resource restitution rule in §15 and the shutdown amendment in
> §28 before U-FS lands.
>
> **Scope:** Unix first, as already ruled by the filesystem design. Non-Unix targets
> MUST fail explicitly rather than silently reinterpret path bytes.
>
> **Primitive delta proposed by v2: `NEW_FS = 20`:**
>
> - 8 `File` natives;
> - 12 `Fs` natives.
>
> This replaces the old 7-File-native plan. `position_` is removed as redundant;
> the new load-bearing primitive is `File#ensureIdle_`, required so the synchronous
> `flush`/closeability surface can enforce Streams v2's one-in-flight-operation law.

## 1. Purpose

The original U-FS plan got the large architectural direction right:

```text
Path bytes
    |
    v
VM native boundary
    |
    v
plain-data reactor job
    |
    v
bounded worker pool
    |
    v
plain-data completion
    |
    v
VM-thread settlement
```

It also correctly required:

- no blocking filesystem syscall on the VM thread except explicit synchronous close;
- no `Value` or `ObjRef` on worker threads;
- paths crossing the OS boundary as bytes;
- `File < Resource`;
- File as an unbuffered Reader/Writer/Seekable;
- `Fs` as path-addressed asynchronous operations;
- immutable metadata/directory snapshots.

The v1 plan is nevertheless unsafe for external handle ownership because it copies
`RawFd` into worker jobs while the Resource row remains independently closeable. The
following execution is possible under that model:

```text
File resource owns fd 17
    |
submit FileRead(fd = 17)
    |
File.close()
    |
close(17)
    |
OS reuses descriptor number 17
    |
worker eventually executes read(17)
```

The delayed operation can target an unrelated file.

FS v2 therefore makes **native file ownership move**, rather than descriptor numbers copy.

The same change solves operation ordering: a File with a shared cursor can have only one
worker-owned baton at a time.

## 2. Public filesystem surface retained

The accepted public selector surface remains:

### `File`

```text
File.open(path)                 -> Future
File.create(path)               -> Future
File.openWith(path, mode:)      -> Future

file.read(dst)                  -> Future
file.write(src)                 -> Future
file.flush                      -> Future
file.sync                       -> Future
file.seek(from)                 -> Future
file.position                   -> Future
file.metadata                   -> Future
file.path                       -> Path
file.close                      -> Result
file.isClosed                   -> Bool
```

`flush` is the Writer protocol requirement. For an unbuffered File it performs no
durability syscall; `sync` is the explicit durability operation.

### `Fs`

```text
Fs.exists(path)                 -> Future
Fs.metadata(path)               -> Future
Fs.symlinkMetadata(path)        -> Future
Fs.readDir(path)                -> Future
Fs.createDir(path)              -> Future
Fs.createDirAll(path)           -> Future
Fs.removeFile(path)             -> Future
Fs.removeDir(path)              -> Future
Fs.removeDirAll(path)           -> Future
Fs.rename(path, to:)            -> Future
Fs.copy(path, to:)              -> Future
Fs.canonicalize(path)           -> Future
```

### Snapshot/value types

```text
Metadata
DirEntry
Permissions
SeekFrom
IoError
```

No buffering flag is added to File.

No String-to-Path coercion is added.

No options bags or recursive flags are added.

## 3. Future/error-channel reconciliation

The existing specifications use both:

```text
Future settles to Ok/Err
```

and tables phrased as:

```text
Future of Result
```

The reactor contract explicitly requires one settlement channel and forbids accidental
`Future<Result<...>>` nesting.

FS v2 therefore makes the implementation shape explicit.

### 3.1 User-visible asynchronous success

A successful operation fulfills the returned Future with its success value:

```text
File.open             -> File
File.read             -> Int count
File.write            -> Int count
File.flush            -> None
File.sync             -> None
File.seek/position    -> Int offset
File.metadata         -> Metadata

Fs.exists             -> Bool
Fs.metadata           -> Metadata
Fs.symlinkMetadata    -> Metadata
Fs.readDir            -> List<DirEntry>
Fs create/remove/...  -> None
Fs.copy               -> Int byte count
Fs.canonicalize       -> Path
```

### 3.2 IO failure

An operating-system/filesystem failure rejects the Future with `IoError`.

It is **not** represented as a fulfilled `.ph` `Err` object.

Therefore:

```text
Future<success> with rejection IoError
```

is the implementation model.

This is the reactor's single settlement channel.

### 3.3 Contract failure

A failure knowable synchronously before job submission raises synchronously and returns no
Future result from that call.

Examples:

```text
String supplied where Path is required
invalid OpenMode object/name
wrong Bytes argument
use after close
File operation while another File operation is unresolved
invalid SeekFrom
offset outside accepted Int domain
```

### 3.4 `await`

The shipped `Future#await` raises a rejected Future's Error.

That behavior is Future semantics, not a synchronous filesystem contract raise.

Documentation should phrase the channel distinction as:

> IO outcomes reject the Future; caller contract violations raise before submission.

This wording removes the earlier ambiguity without adding a nested Result layer.

### 3.5 Synchronous `close`

`Resource#close` remains the one intentional actual `Result` return in this subsystem:

```text
File#close -> Result
```

because close is synchronous and fallible.

## 4. Numeric representation after the tower split

The normative filesystem surface historically said `Number` because the old runtime had one
numeric representation.

The implementation now has `Int` and `Float`.

Filesystem counts, offsets, sizes, and timestamps are integer data and MUST be surfaced as
`Int` wherever representable.

Therefore:

```text
read count            Int
write count           Int
copy count            Int
position              Int
seek result           Int
Metadata.size         Int
Metadata.modified     Int | None
Metadata.accessed     Int | None
Metadata.created      Int | None
SeekFrom.offset       Int
```

Because `Int < Number`, this tightens representation without changing the broad Number
supertype contract.

No filesystem operation silently converts a large integer result to Float.

If a host result cannot fit in Phalcom `Int`, reject the Future with:

```text
IoError(kind = #valueOutOfRange)
```

rather than truncate, wrap, or round.

## 5. Path boundary

The Path corrective patch is authoritative.

Every filesystem selector accepts `Path`, never String.

The worker boundary receives raw path bytes.

On Unix:

```rust
use std::os::unix::ffi::{OsStrExt, OsStringExt};
```

Conceptually:

```text
Path
  -> defensive Bytes view/copy at .ph boundary
  -> Vec<u8>
  -> OsString::from_vec
  -> PathBuf
```

No conversion through UTF-8 String is permitted.

`Path#toString` remains display-only and MUST never feed an OS operation.

A byte sequence containing NUL is a valid Path value but may be rejected by the host open/syscall
layer; that becomes asynchronous `IoError(#invalidInput)` rather than Path corruption.

## 6. OpenMode mapping

The Path patch's closed four-value semantics are binding.

Worker-side mode is a Rust enum:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenModeName {
    Read,
    Write,
    Append,
    ReadWrite,
}
```

Mapping:

| Mode | Must exist | Create | Truncate | Append | Read | Write |
|---|---:|---:|---:|---:|---:|---:|
| `read` | yes | no | no | no | yes | no |
| `write` | no | yes | yes | no | no | yes |
| `append` | no | yes | no | yes | no | yes |
| `readWrite` | yes | no | no | no | yes | yes |

`readWrite` is deliberately non-destructive and analogous to conventional `r+`.

`append` MUST use the host append flag. It MUST NOT be implemented as:

```text
seek(end)
write
```

because that is not atomic with concurrent writers.

`File.create(path)` is exactly:

```phalcom
File.openWith(path, mode: OpenMode.write)
```

at the semantic level.

The VM/native boundary validates the closed mode name **before** worker submission.

Workers never string-dispatch arbitrary user-provided mode names.

## 7. File is an owned native baton

### 7.1 Never use a copied `RawFd` as job ownership

The Resource row MUST NOT store a descriptor integer that remains independently usable while a
worker job also holds that integer.

The owned native object is:

```rust
std::fs::File
```

or an equivalent unique owned handle abstraction.

The v2 design uses the term **file baton** for that owned value.

### 7.2 ResourceKind extension

Extend Resource v2 with:

```rust
pub enum ResourceKind {
    Managed { name: Box<str> },
    File(FileResource),
}
```

where:

```rust
pub struct FileResource {
    pub state: FileResourceState,
    pub next_operation: u64,
}

pub enum FileResourceState {
    Idle(std::fs::File),
    Busy {
        operation: FileOperationId,
        operation_name: &'static str,
    },
}
```

No `Value` or `ObjRef` is stored.

`std::fs::File` is native Rust ownership, not a GC root.

### 7.3 Resource state while a File operation is pending

The outer Resource slot remains:

```text
ResourceState::Open
```

for the whole operation.

Its File kind transitions:

```text
File::Idle(file)
    |
    | claim operation
    v
File::Busy(op)
    |
    | completion restitution
    v
File::Idle(file)
```

A busy File is still open:

```text
isClosed == false
```

but it cannot accept another stateful operation or close.

### 7.4 One baton, one owner

At all times exactly one component owns the native file handle:

```text
idle:
    ResourceTable owns std::fs::File

pending:
    worker Job owns std::fs::File

completed but undrained:
    Completion owns std::fs::File

after drain:
    ResourceTable owns std::fs::File again
```

There is no duplicated descriptor lifetime and no integer fd alias.

## 8. File operation token

Resource generation protects slot reuse. A second token protects operation restitution.

Recommended plain-data token:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FileOperationId {
    pub resource: ResourceHandle,
    pub serial: u64,
}
```

The serial increments monotonically per File resource.

A job/completion carries this operation ID.

The Resource row's Busy state stores the same ID.

Restitution succeeds only when:

```text
resource handle generation matches
resource is Open(File)
File state is Busy
busy operation id == completion operation id
```

Any mismatch is an internal invariant violation, not a user-facing stale-file result.

On mismatch, the completion-owned `std::fs::File` MUST still be disposed safely so it cannot leak.

## 9. ResourceTable File API

Resource v2's generic table remains authoritative for handle classification and leak metadata.

U-FS adds table operations with semantics equivalent to:

```rust
impl ResourceTable {
    fn adopt_file(
        &mut self,
        file: std::fs::File,
        open_site: Option<SourceRange>,
    ) -> Result<ResourceHandle, ResourceTableError>;

    fn claim_file_operation(
        &mut self,
        handle: ResourceHandle,
        operation_name: &'static str,
        attempt_site: Option<SourceRange>,
    ) -> Result<(FileOperationId, std::fs::File), FileAccessError>;

    fn restore_file_operation(
        &mut self,
        operation: FileOperationId,
        file: std::fs::File,
    ) -> Result<(), FileRestoreError>;

    fn ensure_file_idle(
        &self,
        handle: ResourceHandle,
        attempt_site: Option<SourceRange>,
    ) -> Result<(), FileAccessError>;

    fn close_file(
        &mut self,
        handle: ResourceHandle,
        close_site: Option<SourceRange>,
    ) -> Result<CloseOutcome, FileAccessError>;
}
```

Exact Rust names may differ.

The state transitions may not.

## 10. Claim semantics

`claim_file_operation` is atomic on the VM thread.

For a matching idle File:

1. classify Resource handle;
2. verify kind is File;
3. allocate next operation serial with checked arithmetic;
4. move `std::fs::File` out of `Idle`;
5. install `Busy(operation)`;
6. return operation ID + owned file baton.

For a Busy File:

```text
raise/return ConcurrentOperationError
```

for conversion to the existing Streams v2 `.ph` class.

For Closed/Retired/stale/malformed:

```text
UseAfterCloseError
```

using Resource v2 diagnostic rules.

A failed claim performs no partial state transition.

## 11. Operation submission rollback

Claiming the baton is not the final step; a reactor job must still be registered/enqueued.

Therefore every native File operation follows this transactional discipline:

```text
validate arguments
prepare any fallible owned input
create/register pending Future metadata
claim file baton
enqueue job
```

If queue/enqueue fails after claim:

```text
restore baton to the same Busy operation immediately
remove registration
return/raise the original submission failure
```

No error path may leave a File permanently Busy without a worker job that can return its baton.

For `write`, snapshot the caller's `Bytes` **before** the baton claim or provide an exact rollback
path if snapshot allocation can fail.

## 12. Close semantics for File

### 12.1 `File#close_` overrides the Resource native selector

The `.ph` `Resource#close` wrapper sends:

```phalcom
self.close_
```

through normal dynamic dispatch.

Bind a File-specific native implementation of the same internal selector:

```text
File#close_
```

This is one of v2's eight File natives.

The public `File#close` remains inherited from Resource and still returns `Result`.

### 12.2 Matching idle File

For an idle File:

1. resolve handle;
2. move the owned `std::fs::File` out of the row;
3. capture stable diagnostic kind/open/close metadata;
4. install Resource v2 Closed/Retired tombstone **before** host close;
5. explicitly close the OS handle exactly once through an API that reports close failure;
6. return bare `None` on success or an Error instance on close failure;
7. never restore the File to Open.

### 12.3 Matching busy File

For a Busy File:

```text
raise ConcurrentOperationError
```

synchronously.

Do not:

- wait for the worker;
- cancel the operation;
- mark Resource closed;
- close a copied descriptor;
- enqueue close behind the operation.

This satisfies Streams v2's one-in-flight rule while preserving synchronous Resource close.

### 12.4 Double close

A matching Closed/Retired handle:

```text
close_ -> None
```

without another OS close.

### 12.5 Stale/malformed

Use Resource v2's UseAfterCloseError classification.

### 12.6 Close syscall and EINTR

The implementation MUST NOT retry `close(2)` after an error.

On Unix, retrying close after an indeterminate failure can close an unrelated newly-reused fd.

Resource v2's "consume before reporting close error" law is binding.

### 12.7 Do not use `Drop` for explicit close result

`std::fs::File` Drop may ignore close errors.

That is insufficient for language-level explicit `close`.

The explicit close path must consume the File into a raw/owned platform handle and invoke a host close
operation whose result can be observed.

Drop remains acceptable only for internal orphan-cleanup cases where no language-level owner ever
received the handle, such as a canceled unopened completion.

## 13. `File#ensureIdle_`

Streams v2 requires a direct File to reject overlapping operations, including `flush` even though
File flush performs no worker syscall.

Add:

```text
File#ensureIdle_
```

as an internal native.

Behavior:

```text
idle File          -> None
busy File          -> raise ConcurrentOperationError
closed/stale/etc.  -> raise UseAfterCloseError
```

It uses first-user-frame attempt attribution.

This primitive is why v2 has eight rather than seven File natives.

## 14. External adoption: open is transactional

### 14.1 The atomicity problem

A worker may successfully create an OS file handle before any `File` object owns a Resource row.

The implementation MUST NOT:

```text
open OS file
-> create resource row
-> settle raw numeric handle
-> later hope .ph File construction succeeds
```

because failure between row creation and object attachment strands a Resource row.

### 14.2 Pending File object

`File.open_` creates an unattached pending File instance on the VM thread **before** worker
submission.

The instance:

```text
class = bootstrapped File
Resource inherited handle slot = None
File path slot = original Path object
```

It is not returned to user code yet.

The reactor registration roots:

```text
pending File object
Future
```

until open completion.

### 14.3 `File.open_` internal signature

Recommended internal signature:

```text
File.open_(pathObject, pathBytes, modeName, future)
```

where the last argument remains the pending Future per the reactor registration convention.

The `.ph` wrapper:

```phalcom
static openWith(path, mode:) {
  // validate Path
  // validate OpenMode
  const future = Future.new()
  File.open_(path, path.bytes, mode.name, future)
  return future
}
```

`File.open` and `File.create` derive through `openWith`.

The native:

1. validates internal argument shapes;
2. allocates pending File instance;
3. stores `pathObject` in File's fixed path slot;
4. builds owned `PathBuf` from `pathBytes`;
5. validates mode to `OpenModeName`;
6. captures user open site;
7. registers pending File + Future;
8. enqueues `FileOpen`.

### 14.4 Successful open completion

The worker returns owned `std::fs::File`.

On the VM thread:

1. verify registration is still current;
2. verify pending File is still unattached;
3. allocate/adopt a ResourceKind::File row;
4. pack the Resource handle;
5. write the handle directly into the inherited Resource slot;
6. only after successful attachment fulfill the Future with that File object.

The table-row creation and handle-slot write are one VM-level adoption transaction.

If adoption fails:

```text
drop/explicitly dispose unadopted std::fs::File
leave pending File unattached
reject Future with appropriate capacity/runtime error
```

No leak is possible.

### 14.5 Failed open completion

No Resource row exists.

The pending File remains unattached/inert and becomes collectible when the registration is removed.

Reject the Future with `IoError`.

### 14.6 Stale/canceled open registration

If the worker successfully opens a file but the open registration is stale before drain:

```text
do not adopt
dispose the completion-owned file baton
do not settle the stale Future
```

This is safe because no language File ever acquired ownership.

## 15. Completion restitution comes before Future settlement

This is the load-bearing reactor amendment.

A File operation completion contains both:

```text
reactor registration token
FileOperationId
owned std::fs::File baton
operation result
```

The VM drain order is:

```text
1. restore native file baton to ResourceTable using FileOperationId
2. only then inspect user-facing registration freshness
3. if registration is current, mint/copy result and settle Future
4. if registration is stale, discard user-facing result without settling
```

The reactor's generic rule:

```text
stale completion -> drop on floor
```

MUST NOT literally drop an external-resource baton before restitution.

For File operations, "drop on floor" means:

```text
restore ownership first;
drop only the settlement payload.
```

This rule is necessary for cancellation, stale tokens, abandoned Futures, and orderly shutdown.

## 16. Worker Job model

Jobs remain owned plain data and `Send`.

Recommended variants:

```rust
pub enum Job {
    FileOpen {
        registration: RegistrationToken,
        path: PathBuf,
        mode: OpenModeName,
        open_site: Option<SourceRange>,
    },

    FileRead {
        registration: RegistrationToken,
        operation: FileOperationId,
        file: std::fs::File,
        len: usize,
    },

    FileWrite {
        registration: RegistrationToken,
        operation: FileOperationId,
        file: std::fs::File,
        data: Vec<u8>,
    },

    FileSync {
        registration: RegistrationToken,
        operation: FileOperationId,
        file: std::fs::File,
    },

    FileSeek {
        registration: RegistrationToken,
        operation: FileOperationId,
        file: std::fs::File,
        from: SeekFromPlain,
    },

    FileMetadata {
        registration: RegistrationToken,
        operation: FileOperationId,
        file: std::fs::File,
    },

    FsExists { ... },
    FsMetadata { ... },
    FsReadDir { ... },
    FsCreateDir { ... },
    FsCreateDirAll { ... },
    FsRemoveFile { ... },
    FsRemoveDir { ... },
    FsRemoveDirAll { ... },
    FsRename { ... },
    FsCopy { ... },
    FsCanonicalize { ... },
}
```

`Job` contains no:

```text
Value
ObjRef
Future object
Fiber object
Path .ph object
ResourceTable pointer
VM pointer
```

`std::fs::File`, `PathBuf`, `Vec<u8>`, scalars, source ranges, and plain enums are allowed.

## 17. Completion model

### 17.1 File completion envelope

Every non-open File operation completion returns the baton regardless of success/failure.

Recommended shape:

```rust
pub struct FileCompletion {
    pub registration: RegistrationToken,
    pub operation: FileOperationId,
    pub file: std::fs::File,
    pub result: Result<FileSuccess, IoErrorData>,
}
```

`FileSuccess`:

```rust
pub enum FileSuccess {
    Read(Vec<u8>),
    WriteCount(u64),
    Unit,
    Position(u64),
    Metadata(StatData),
}
```

The read Vec length is the exact count read.

### 17.2 FileOpen completion

```rust
pub struct FileOpenCompletion {
    pub registration: RegistrationToken,
    pub result: Result<std::fs::File, IoErrorData>,
}
```

The pending File object lives only in the VM registration, never the worker completion.

### 17.3 Fs completion

```rust
pub struct FsCompletion {
    pub registration: RegistrationToken,
    pub result: Result<FsSuccess, IoErrorData>,
}
```

with:

```rust
pub enum FsSuccess {
    Bool(bool),
    Unit,
    Metadata(StatData),
    DirList(Vec<DirEntryData>),
    CopyCount(u64),
    PathBytes(Vec<u8>),
}
```

No raw fd payload exists.

### 17.4 Plain-data structural assertion

Tests MUST prove worker jobs/completions are `Send`.

Repository review MUST grep/structurally inspect the enums so no `Value`/`ObjRef` can enter them.

## 18. `IoErrorData`

Workers return plain error data.

Recommended:

```rust
pub struct IoErrorData {
    pub kind: IoErrorKind,
    pub message: String,
    pub raw_os_error: Option<i32>,
}
```

`IoErrorKind` is a closed Rust enum, not an arbitrary string:

```rust
pub enum IoErrorKind {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    NotDirectory,
    IsDirectory,
    DirectoryNotEmpty,
    InvalidInput,
    InvalidData,
    ResourceExhausted,
    Unsupported,
    ValueOutOfRange,
    Other,
}
```

Use the closest stable `std::io::ErrorKind` mapping.

Unknown/future host errors map to `Other` while preserving the message and raw OS code where available.

Do not use arbitrary worker strings as the semantic kind.

## 19. `IoError` becomes a bootstrapped Error subclass

The v1 implementation proposed an ordinary `.ph` `IoError` while also requiring native completion
code to construct it.

That forces native code to depend on a dynamically-defined `.ph` class/layout.

V2 resolves the boundary explicitly.

Bootstrap:

```text
IoError < Error
```

as a kernel class.

It adds no new primitive of its own.

It inherits the fixed Error fields already used by the runtime:

```text
_message
_kind
_cause
_displaced
```

`.ph` may define:

```phalcom
class IoError {
  name => self.kind
}
```

for filesystem-facing terminology.

Native completion minting initializes:

```text
message    = IoErrorData.message
kind       = Symbol such as #notFound
cause      = None
displaced  = None
```

No additional IoError instance field is required.

### 19.1 Required bootstrap wiring

IoError MUST join all kernel-class integration points:

```text
make_core_class
CoreClasses field
universe invariant row
install_core add_class! row
class census row
```

This is the same standing obligation learned from U-BYTES.

## 20. Error-kind mapping

At minimum:

```text
NotFound             -> #notFound
PermissionDenied     -> #permissionDenied
AlreadyExists        -> #alreadyExists
NotDirectory         -> #notDirectory
IsDirectory          -> #isDirectory
DirectoryNotEmpty    -> #directoryNotEmpty
InvalidInput         -> #invalidInput
InvalidData          -> #invalidData
ResourceExhausted    -> #resourceExhausted
Unsupported          -> #unsupported
ValueOutOfRange      -> #valueOutOfRange
Other                -> #io
```

The exact raw host message is diagnostic data, not the stable programmatic kind.

Close failures also use IoError or another existing Error carrier consistently; do not return a
raw Rust string from `File#close_`.

## 21. File bootstrap and object layout

`File < Resource` is bootstrapped.

Resource v2 owns inherited slot 0:

```text
slot 0 = packed Resource handle | None
```

File owns exactly one additional field:

```text
slot 1 = _path
```

Therefore the effective instance field count is:

```text
2
```

and MUST be stamped explicitly wherever bootstrapped field layout requires it.

`File#path` returns `_path`.

The path object stored is the original immutable Path supplied at open time.

### 21.1 `path` after close

`path` is a cached immutable snapshot and performs no native operation.

V2 permits it after close.

This matches its nonblocking snapshot nature and makes diagnostics/introspection useful after resource
release.

`isClosed` and `path` are the two non-close File accessors that do not require an idle live baton.

All actual IO selectors require live ownership.

## 22. Eight File natives

| Native | Internal binding | Purpose |
|---|---|---|
| `file_open` | `File.open_(_,_,_,_)` static | allocate pending File registration and submit open |
| `file_close` | `File#close_` override | busy-aware exactly-once explicit close |
| `file_ensure_idle` | `File#ensureIdle_` | Streams v2 overlap guard for no-op `flush` |
| `file_read` | `File#read_(_,_)` | claim baton and submit read |
| `file_write` | `File#write_(_,_)` | snapshot source, claim baton, submit write |
| `file_sync` | `File#sync_(_)` | claim baton and submit fsync |
| `file_seek` | `File#seek_(_,_,_)` | claim baton and submit seek |
| `file_metadata` | `File#metadata_(_)` | claim baton and submit fstat |

Every reactor-registering primitive takes the pending Future as its final argument.

`close_` and `ensureIdle_` are synchronous and do not take a Future.

There is no `position_`.

There is no `flush_`.

## 23. `.ph` File surface

Conceptual implementation:

```phalcom
class File is Resource {
  static open(path) =>
    File.openWith(path, mode: OpenMode.read)

  static create(path) =>
    File.openWith(path, mode: OpenMode.write)

  static openWith(path, mode:) {
    // validate Path
    // validate OpenMode
    const future = Future.new()
    File.open_(path, path.bytes, mode.name, future)
    return future
  }

  read(dst) {
    // validate Bytes
    const future = Future.new()
    self.read_(dst, future)
    return future
  }

  write(src) {
    // validate Bytes
    const future = Future.new()
    self.write_(src, future)
    return future
  }

  flush {
    self.ensureIdle_
    return Future.value(None)
  }

  sync {
    const future = Future.new()
    self.sync_(future)
    return future
  }

  seek(from) {
    // validate SeekFrom
    const future = Future.new()
    self.seek_(from.name, from.offset, future)
    return future
  }

  position =>
    self.seek(SeekFrom.current(0))

  metadata {
    const future = Future.new()
    self.metadata_(future)
    return future
  }

  path => _path
}
```

Exact syntax follows current core.ph conventions.

Operation natives themselves perform authoritative Resource handle/busy validation; `.ph` type checks
do not replace that.

## 24. File read

### 24.1 Entry

`File#read(dst)` validates `dst` is Bytes.

If:

```text
dst.size == 0
```

then Streams v2 requires an immediate fulfilled `0` and no OS read.

The implementation may return before native submission for that case.

### 24.2 Pending registration roots destination

For a non-empty read, the VM registration stores:

```text
Future
destination Bytes ObjRef
```

as GC roots.

The worker receives only:

```text
owned std::fs::File
requested length
```

### 24.3 Worker

The worker allocates an owned Vec with fallible allocation.

It performs one `Read::read`.

Legal success is a short prefix:

```text
0 <= n <= requested
```

It truncates the Vec to exactly `n` and returns it with the baton.

`n == 0` is EOF for the non-empty destination.

### 24.4 Drain

On VM thread:

1. restore file baton;
2. check registration freshness;
3. if stale, drop Vec and do not mutate destination;
4. if current and success, bulk-copy Vec into destination starting at 0;
5. fulfill Future with Int count;
6. if IO failure, reject with IoError.

The destination is not mutated before completion.

The caller must obey Streams v2's rule not to mutate/reuse `dst` while the read Future is unresolved.

## 25. File write

### 25.1 Source snapshot

Before returning from `File#write(src)`, the native submission path copies the source bytes into owned:

```rust
Vec<u8>
```

This is mandatory even though the File is unbuffered.

The worker never reads a Phalcom Bytes object.

The caller may mutate `src` immediately after `write` returns.

### 25.2 Zero-length

A zero-length write fulfills immediately with `0` and need not claim the baton.

### 25.3 One syscall

Worker performs one `Write::write`.

It does not loop to completion.

A short positive count is a valid Writer result.

The returned count must satisfy:

```text
0 <= n <= source length
```

Rust's `Write` contract already enforces the upper bound; assert/debug-check in the worker conversion.

### 25.4 Zero count

A direct `File#write` may report `0` if the host API does.

Streams v2's `BufferedWriter` converts zero progress into `WriteZeroError` only when it is obliged
to drain already-accepted pending bytes.

File itself reports the actual accepted count.

## 26. File `flush` and `sync`

### `flush`

File is unbuffered at the language layer.

`flush`:

```text
ensure live + idle
return Future.value(None)
```

It does not call `fsync`.

It does not wait behind another operation.

If File is Busy:

```text
raise ConcurrentOperationError
```

This prevents a misleading "flush succeeded" result while an earlier write is still unresolved.

### `sync`

`sync` claims the baton and worker-runs:

```text
fsync / File::sync_all
```

or the platform-equivalent full file synchronization ruled by the filesystem contract.

`sync` fulfills `None` on success and rejects IoError on failure.

## 27. Seek and position

`SeekFrom` remains a pure `.ph` value type with exactly:

```text
SeekFrom.start(offset)
SeekFrom.current(offset)
SeekFrom.end(offset)
```

Recommended fields:

```text
_name
_offset
```

Rules:

```text
start(offset): offset Int and >= 0
current(offset): offset Int, signed allowed
end(offset): offset Int, signed allowed
```

Invalid constructor values raise synchronously.

Worker plain form:

```rust
pub enum SeekFromPlain {
    Start(u64),
    Current(i64),
    End(i64),
}
```

Successful host position is `u64`.

Before settlement:

```text
position <= i64::MAX
```

else reject:

```text
IoError(#valueOutOfRange)
```

`File#position` is derived as:

```phalcom
self.seek(SeekFrom.current(0))
```

No native is allocated for it.

## 28. Reactor shutdown amendment required by owned batons

The existing reactor text permits a bounded worker-join wait followed by abandoning a worker stuck
in an uninterruptible syscall.

That is incompatible with unique File baton ownership.

A worker that owns the only `std::fs::File` cannot be abandoned while Resource shutdown pretends
the table can drain that File.

Therefore FS v2 requires the following rule for jobs owning external Resource batons:

> Orderly VM shutdown does not advance to Resource leak snapshot/drain until every external-resource
> baton has either returned to the VM or been explicitly classified as unrecoverable process-exit
> ownership.

The preferred v2 implementation is stronger and simpler:

```text
stop new submissions
wait for all File jobs to complete
join worker pool
drain all completions and restore batons
then run Resource shutdown
```

There is no fake bounded abandonment for File jobs.

A host filesystem call can therefore delay orderly process shutdown. That is an honest consequence
of blocking filesystem IO and unique ownership.

If the project later requires bounded shutdown, it needs a separate cancellation/worker-isolation
design. U-FS must not invent descriptor duplication or unsound abandonment to simulate it.

## 29. Completion drain at shutdown

After workers are stopped/joined:

1. drain every queued completion;
2. restore every File operation baton, even for stale user registrations;
3. dispose successful but stale FileOpen batons without adoption;
4. do not newly settle pending Futures merely for shutdown;
5. verify no ResourceKind::File remains Busy;
6. take Resource v2 leak snapshot;
7. render leaks;
8. Resource drain closes every idle leaked File exactly once;
9. determine strict-resource exit status without replacing an existing primary failure.

A Busy File after step 5 is an internal shutdown invariant failure.

## 30. File metadata

Worker calls descriptor-based metadata (`fstat` equivalent / `File::metadata`).

It returns plain `StatData`.

No path lookup is performed, avoiding TOCTOU substitution after open.

`File#metadata` therefore describes the opened file object, not whatever now occupies `file.path`.

## 31. StatData

Recommended plain form:

```rust
pub struct StatData {
    pub size: u64,
    pub file_type: FileTypeData,
    pub modified_ms: Option<i64>,
    pub accessed_ms: Option<i64>,
    pub created_ms: Option<i64>,
    pub read_only: bool,
}

pub struct FileTypeData {
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,
}
```

All fields are plain worker data.

### 31.1 Time conversion

Convert SystemTime to signed integral milliseconds since Unix epoch.

Pre-epoch times are negative, not automatically `None`.

`None` means the OS does not expose that timestamp.

Conversion uses checked arithmetic.

An out-of-Int-range timestamp rejects the operation with `#valueOutOfRange`.

### 31.2 File size

`u64` size converts to Phalcom Int only when <= `i64::MAX`.

Otherwise reject `#valueOutOfRange`.

## 32. Metadata `.ph` snapshot

`Metadata` is ordinary `.ph` data.

Fields:

```text
_size
_isFile
_isDir
_isSymlink
_modified
_accessed
_created
_permissions
```

Accessors are plain and never return Future.

`Permissions` is a pure `.ph` snapshot:

```phalcom
class Permissions {
  @constructor
  new(readOnly) { _readOnly = readOnly }
  isReadOnly => _readOnly
}
```

No POSIX mode integer is exposed.

`Metadata` construction occurs in `.ph` from the plain tuple/list shape minted on the VM thread,
or through an internal constructor with explicit fields.

No worker constructs Metadata.

## 33. DirEntryData

Worker `readDir` record:

```rust
pub struct DirEntryData {
    pub file_name: Vec<u8>,
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,
}
```

The name is the final component only.

Worker obtains type through `DirEntry::file_type()`.

If enumeration or file-type lookup fails, the whole `readDir` operation rejects.

This matches the current whole-directory-one-settlement contract.

No implicit metadata syscall is added to `DirEntry`.

## 34. DirEntry `.ph` snapshot

`Fs.readDir(path)` retains the original directory Path in its `.ph` shaping continuation.

For each plain row:

```text
fileName = Path.ofBytes(nameBytes)
path     = originalDirectory.join(fileName)
```

The Path patch's join semantics are binding.

Fields:

```text
_fileName
_path
_isFile
_isDir
_isSymlink
```

All accessors are plain.

The filename byte sequence returned by the OS is preserved exactly.

## 35. Twelve Fs natives

The 12 native operations remain one-per-operation:

| Native | Internal selector |
|---|---|
| `fs_exists` | `Fs.exists_(_,_)` |
| `fs_metadata` | `Fs.metadata_(_,_)` |
| `fs_symlink_metadata` | `Fs.symlinkMetadata_(_,_)` |
| `fs_read_dir` | `Fs.readDir_(_,_)` |
| `fs_create_dir` | `Fs.createDir_(_,_)` |
| `fs_create_dir_all` | `Fs.createDirAll_(_,_)` |
| `fs_remove_file` | `Fs.removeFile_(_,_)` |
| `fs_remove_dir` | `Fs.removeDir_(_,_)` |
| `fs_remove_dir_all` | `Fs.removeDirAll_(_,_)` |
| `fs_rename` | `Fs.rename_(_,_,_)` |
| `fs_copy` | `Fs.copy_(_,_,_)` |
| `fs_canonicalize` | `Fs.canonicalize_(_,_)` |

Every selector takes a pending Future as the last argument.

`rename_` and `copy_` take two path-byte arguments plus Future.

No string-dispatched mega-native is introduced.

## 36. Fs `.ph` wrappers

Every wrapper:

1. validates every path argument is Path;
2. obtains defensive path bytes;
3. creates pending Future;
4. invokes exactly one native registration primitive;
5. returns a Future or a derived/shaped Future.

Contract violations happen before worker submission.

### 36.1 Shaping operations

Operations needing no high-level object shaping may return the raw registered Future directly:

```text
exists
createDir
createDirAll
removeFile
removeDir
removeDirAll
rename
copy
```

`metadata`, `symlinkMetadata`, `readDir`, and `canonicalize` use `.ph` continuation shaping of
successful VM-minted plain values.

A rejected raw Future remains rejected through continuation composition.

## 37. `Fs.exists`

`exists` is advisory and never a substitute for attempting the actual operation.

Semantics:

```text
metadata/stat succeeds                  -> fulfill true
NotFound / path component NotDirectory -> fulfill false
other IO errors                         -> reject IoError
```

Do not convert permission denial, IO failure, or resource exhaustion to false.

This is closer to a fallible existence probe than a truth claim about future operations.

## 38. `Fs.metadata` and `Fs.symlinkMetadata`

`Fs.metadata` follows symlinks.

Worker uses host `metadata/stat`.

`Fs.symlinkMetadata` does not follow the final symlink.

Worker uses host `symlink_metadata/lstat`.

Both return StatData and are shaped to Metadata.

For followed metadata, `isSymlink` ordinarily reflects the resolved target and is false.

For symlinkMetadata on a symlink, `isSymlink` is true.

## 39. `Fs.readDir`

Worker enumerates the whole directory because streaming readDir is explicitly out of scope.

Ordering is the host enumeration order unless the governing filesystem contract later specifies
sorting.

Do not silently sort entries.

Tests MUST NOT depend on directory order; compare by names/content unless the host fixture creates
a stronger portable invariant.

Each filename remains raw bytes.

The directory stream/iterator native object is worker-local and is fully consumed/dropped before
completion.

## 40. `Fs.createDir`

Creates exactly one directory level.

Parent must already exist.

Existing destination produces `#alreadyExists` unless host semantics identify an equivalent
specific error.

No implicit recursive behavior.

## 41. `Fs.createDirAll`

Creates intermediate directories.

If the complete path already names an existing directory:

```text
fulfill None
```

If an intermediate or final component exists as a non-directory, reject appropriate IoError.

This operation remains subject to filesystem races; it does not establish a transaction.

## 42. `Fs.removeFile`

Removes a non-directory file entry.

On Unix, removing a symlink through `removeFile` removes the symlink itself, not its target.

Passing a directory rejects.

No recursive behavior.

## 43. `Fs.removeDir`

Removes exactly one empty directory.

Non-empty directory rejects `#directoryNotEmpty` where host mapping permits.

No recursion.

## 44. `Fs.removeDirAll`

Recursive directory removal is intentionally loud and destructive.

It MUST NOT follow symlinks into their targets.

A symlink encountered within the tree is removed as a directory entry, not traversed.

Use a host/library operation with this property rather than hand-rolling recursion unless the
standard implementation cannot satisfy the contract.

The operation is not atomic.

A partial failure may leave part of the tree removed.

The Future rejects with the first reported IO error; no rollback is promised.

## 45. `Fs.rename`

Use the host rename primitive.

On the Unix-first implementation:

- same-filesystem rename is atomic where POSIX guarantees it;
- cross-filesystem rename rejects with the host error;
- destination replacement follows host `rename(2)` semantics;
- no copy/delete fallback is performed;
- no pre-existence check is performed.

The operation remains race-safe at the level the host rename primitive provides.

## 46. `Fs.copy`

Copies file contents and permissions as required by the filesystem contract.

V2 uses semantics equivalent to Rust `std::fs::copy` unless a lower-level implementation is needed
for byte-path fidelity.

Freeze:

```text
source is followed/opened as a file
destination is created if absent
destination is truncated/replaced if it is an existing file
returned value is bytes copied
permissions are copied
timestamps/extended attributes are not promised
```

The returned host `u64` count must fit Phalcom Int.

Otherwise reject `#valueOutOfRange`.

No recursive directory copy is introduced.

## 47. `Fs.canonicalize`

Worker resolves the actual filesystem path, including:

```text
.
..
symlinks
relative path against process working directory
```

according to the host canonicalization primitive.

The result is returned as raw path bytes.

On Unix:

```text
PathBuf -> OsStr::as_bytes().to_vec()
```

VM creates Bytes, `.ph` creates Path.ofBytes.

No lossy String conversion occurs.

## 48. Process working directory

Relative Path operations use the process current working directory at the time the worker executes
the syscall.

V2 does not snapshot or virtualize cwd per Future submission.

If Phalcom later introduces a mutable process cwd API, ordering/capability implications require a
separate decision.

## 49. Host descriptor inheritance

Opened File descriptors MUST be non-inheritable across process exec by default.

On Unix use close-on-exec behavior (`O_CLOEXEC` or standard-library equivalent).

Do not create inheritable descriptors and patch them afterward where an atomic close-on-exec open
is available.

This prevents descriptor leakage into future process-spawn functionality.

## 50. Worker allocation failures

Worker-side temporary allocations are fallible language/runtime outcomes, not process-abort paths.

Examples:

```text
FileRead Vec
readDir entry Vec
canonicalized path Vec
error-message/string aggregation
```

Use `try_reserve` or equivalent where user/environment-controlled cardinality can become large.

Map allocation refusal to:

```text
IoError(#resourceExhausted)
```

for this IO subsystem.

Do not panic/abort because an enormous directory or read buffer exhausted worker memory.

Managed Bytes allocation on the VM side follows Bytes v2.

## 51. File read destination settlement safety

The pending registration roots the destination Bytes while the read is unresolved.

Because completion restitution occurs before settlement:

```text
resource is Idle
then destination is copied
then Future is fulfilled
```

If copying into destination unexpectedly fails due an internal invariant after a successful OS read,
the File baton remains safely restored before the error propagates.

No error in language-value shaping may strand native ownership in a Completion.

## 52. File operation ordering

A File has one native baton.

Therefore:

```text
at most one unresolved File operation
```

is enforced for:

```text
read
write
sync
seek
position
metadata
```

`flush` checks the same idle state but submits no job.

`close` requires idle.

`path` and `isClosed` are observational and may be queried while Busy.

### 52.1 Caller pattern

Correct:

```phalcom
file.write(a).await
file.write(b).await
file.seek(SeekFrom.start(0)).await
```

Incorrect overlap:

```phalcom
const a = file.write(x)
const b = file.write(y) // raises ConcurrentOperationError
```

The implementation does not secretly reorder or queue these operations.

## 53. Busy diagnostics

Use the Streams v2 `ConcurrentOperationError` class.

The diagnostic SHOULD identify:

```text
File
attempted operation
currently pending operation
open site when available
attempt site
```

No new kernel busy-error class is required.

Native primitives may ask the VM to instantiate the existing pure `.ph`
`ConcurrentOperationError` by the same established runtime mechanism used for other `.ph`
error classes only if that mechanism is already robust.

If native code cannot reliably mint an ordinary `.ph` error class, promote
`ConcurrentOperationError` to a bootstrapped Error subclass in the same change rather than
encoding busy as IoError.

Busy is a contract violation, not an IO outcome.

## 54. Use-after-close

Every File operation claim uses Resource v2's exact handle classification.

Closed/stale/malformed attempts surface `UseAfterCloseError`, never IoError.

A Busy File is not closed and therefore is not a use-after-close condition.

The operation primitive records first-user attempted-use attribution.

## 55. Open-site attribution

`File.open_` is reached through `.ph` wrappers.

The native open registration MUST use Resource v2's:

```text
first user source range
```

rather than the immediate core.ph frame.

The adopted Resource row receives that user open site.

Leak diagnostics therefore point to:

```text
user source that requested File.open/create/openWith
```

not `core.ph`.

## 56. Close-site attribution

File-specific `close_` uses the same Resource v2 user-frame rule.

A later use-after-close diagnostic can therefore render:

```text
kind: File
opened: user.ph:...
closed: user.ph:...
attempted: user.ph:...
```

for a matching closed tombstone.

## 57. FileOpen registration state

The reactor's VM-side registration table may contain GC-managed roots.

For open:

```rust
PendingRegistration::FileOpen {
    future: ObjRef,
    pending_file: ObjRef,
    ...
}
```

or equivalent.

This does not violate the worker-thread plain-data law.

The registration table is a VM GC root by reactor design.

The `Job::FileOpen` sent to workers contains no ObjRef.

## 58. FileRead registration state

For read:

```rust
PendingRegistration::FileRead {
    future: ObjRef,
    destination: ObjRef,
    operation: FileOperationId,
    ...
}
```

or equivalent.

The destination remains live until completion/staleness cleanup.

Again, it never crosses to the worker.

## 59. Other File registration state

Write/sync/seek/metadata need only the Future plus ordinary registration metadata; owned source
write data is in the worker job.

Every registration records enough information to:

- match completion token;
- settle/reject correct Future;
- cleanly remove itself;
- report pending-registration leaks under reactor rules.

## 60. Cancellation/renunciation interaction

User-visible cancellation may be a later unit, but the token substrate already exists.

V2 defines behavior now so future cancellation cannot break file ownership.

### Open

Renounced/stale open completion:

```text
dispose unopened File baton
no Resource row
no settlement
```

### Existing File operation

Renounced/stale read/write/sync/seek/metadata completion:

```text
restore File baton to Resource row
discard operation result
do not settle stale Future
```

The operation may already have had OS side effects.

Cancellation is renunciation of the result, not rollback of completed filesystem work.

The File becomes idle again when the baton returns.

## 61. Orphaned File object during operation

The Resource table, not the heap object, owns the native lifecycle.

If a user File object becomes unreachable while an operation is pending:

```text
Resource row remains Open(Busy)
job owns native baton
completion restores baton
row becomes Open(Idle)
```

At orderly shutdown the row appears as a leak and Resource drain closes it.

No GC finalizer is required.

## 62. Open Future abandoned before completion

A merely unreachable Future remains rooted by a live reactor registration under the current reactor
contract, so it is not collected prematurely.

If cancellation later makes the registration stale, §60 applies.

If no cancellation exists, completion adopts the File and settles the rooted Future; if no user
reference remains afterward, the File object can become unreachable but its Resource row is a leak
until explicit shutdown drain.

This is consistent with Resource v2's leak-first posture.

## 63. Close failure and state

When explicit close attempts the native host close:

```text
Resource tombstone is already installed
```

If host close reports error:

```text
File remains language-level closed
close returns Err(IoError)
second close returns Ok
```

No retry.

No descriptor is put back into the Resource row.

## 64. Resource leak drain

At shutdown, after reactor restitution guarantees every File is Idle, Resource v2 drain moves the
File baton out, installs tombstone, performs explicit close once, and records any close failure.

Drain order remains reverse allocation serial.

Leak reporting happens from the pre-drain snapshot.

Close failures during drain do not prevent later resources from being drained.

## 65. Permissions

`Permissions` MUST be included in the implementation inventory.

It is pure `.ph`, immutable, and contains only:

```text
isReadOnly -> Bool
```

No primitive.

No POSIX mode surface.

No hidden raw integer mode field exposed to user code.

## 66. SeekFrom

`SeekFrom` is pure `.ph`.

Recommended:

```phalcom
class SeekFrom {
  @constructor
  named_(name, offset) {
    _name = name
    _offset = offset
  }

  static start(offset)   => ...
  static current(offset) => ...
  static end(offset)     => ...

  name => _name
  offset => _offset
}
```

The closed names are:

```text
start
current
end
```

Unknown names are rejected at the native boundary regardless of internal constructor convention.

`start` requires nonnegative Int.

## 67. Snapshot shaping boundary

Workers do not mint:

```text
Metadata
Permissions
DirEntry
Path
IoError
File
```

The VM thread is the only place that creates language objects.

Exceptions by category:

- `File` is minted/adopted natively on VM thread because atomic external ownership requires it;
- `IoError` is minted natively on VM thread because Future rejection requires an Error instance and
  IoError is bootstrapped;
- Metadata/Permissions/DirEntry/Path remain `.ph` shaping over plain VM values.

This is a deliberate boundary, not accidental inconsistency.

## 68. Native completion value shapes

Recommended VM-thread success shaping:

```text
FileOpen       -> File object
FileRead       -> Int count after destination copy
FileWrite      -> Int
FileSync       -> None
FileSeek       -> Int
FileMetadata   -> Tuple/plain List record

FsExists       -> Bool
FsMetadata     -> Tuple/plain List record
FsReadDir      -> List<Tuple>
FsCreate...    -> None
FsRemove...    -> None
FsRename       -> None
FsCopy         -> Int
FsCanonicalize -> Bytes
```

`.ph` maps raw stat records to Metadata and raw canonical bytes to Path.

The exact internal Tuple slot order MUST be documented in code and tests if Tuple is used.

Prefer a dedicated helper function per payload rather than scattering numeric slot constants.

## 69. Metadata raw record order

If Tuple is used, freeze:

```text
0 size Int
1 isFile Bool
2 isDir Bool
3 isSymlink Bool
4 modified Int|None
5 accessed Int|None
6 created Int|None
7 readOnly Bool
```

`.ph` `Metadata.fromRaw_` or equivalent converts this record into snapshot objects.

No public API exposes the tuple.

## 70. DirEntry raw record order

If Tuple is used:

```text
0 fileName Bytes
1 isFile Bool
2 isDir Bool
3 isSymlink Bool
```

`.ph` receives original directory Path separately when shaping.

No raw String filename exists.

## 71. Reactor safepoint settlement ordering

For File operations:

```text
completion channel receive
-> baton restitution
-> convert IoErrorData or success payload
-> mutate read destination if needed
-> settle/reject Future
-> enqueue resumed fibers at reactor-defined queue position
```

A resumed fiber can therefore immediately submit another File operation because the Resource row is
already Idle.

Do not settle first and restore later.

## 72. Worker operation implementations

Use `std::fs` / `std::io` abstractions where they preserve required semantics.

### Open

`std::fs::OpenOptions`.

### Read

`std::io::Read::read`.

### Write

`std::io::Write::write`.

### Sync

`std::fs::File::sync_all`.

### Seek

`std::io::Seek::seek`.

### File metadata

`std::fs::File::metadata`.

### Fs metadata

`std::fs::metadata`.

### Symlink metadata

`std::fs::symlink_metadata`.

### Directory

`std::fs::read_dir`.

### Directory creation/removal/rename/copy/canonicalize

Corresponding `std::fs` operations where their documented Unix behavior meets §§40-47.

Do not drop to libc merely for stylistic consistency.

Use lower-level APIs only where std cannot expose the required byte/close/error semantics.

## 73. Explicit close helper

Unix explicit close requires observable result.

Recommended helper boundary:

```rust
fn close_file_explicit(file: std::fs::File) -> std::io::Result<()>;
```

It consumes File.

Implementation may:

```text
File -> raw fd
single close(2)
```

using the project's accepted platform FFI/dependency.

Requirements:

- exactly one close call;
- no retry on EINTR;
- no Drop second-close;
- convert errno to IoError;
- function owns the handle for the entire transition.

If conversion to raw fd transfers ownership, ensure Rust Drop cannot also close it.

## 74. Non-Unix posture

Until the Windows path/handle ruling lands:

```rust
#[cfg(not(unix))]
compile_error!("filesystem byte-path backend is not implemented on this target");
```

or the project-equivalent explicit build failure.

Do not:

- lossy-convert paths to UTF-8;
- pretend Unix slash/OsString byte semantics work on Windows;
- silently disable non-UTF8 tests.

The internal `std::fs::File` baton design is intentionally portable enough that future Windows work
can replace only path/explicit-close platform seams.

## 75. Census

Proposed primitive count:

```text
File:
    open_
    close_
    ensureIdle_
    read_
    write_
    sync_
    seek_
    metadata_
    = 8

Fs:
    12

NEW_FS = 20
```

`position_` is removed.

`flush_` does not exist.

IoError is a new bootstrapped class but adds no primitive.

File and IoError both receive core-class/invariant/census class rows as required by repository
bootstrap policy.

Verify the live primitive baseline at implementation time before changing the running total.

## 76. File-by-file implementation map

### `phalcom-core/src/resource.rs`

Add:

```text
ResourceKind::File
FileResource
FileResourceState
FileOperationId
adopt_file
claim_file_operation
restore_file_operation
ensure_file_idle
close_file
```

Preserve Resource v2 no-Value/no-ObjRef structural rule.

### reactor module

Add:

```text
File/Fs Job variants
FileOpenCompletion
FileCompletion
FsCompletion
IoErrorData
StatData
DirEntryData
OpenModeName
SeekFromPlain
```

Amend stale completion handling for baton restitution.

Amend shutdown ordering per §28.

### File primitives module

Add all eight File natives.

### Fs primitives module

Add 12 Fs natives.

### universe/core classes

Bootstrap:

```text
File < Resource
IoError < Error
```

and all required registration/invariant rows.

### VM bootstrap

Stamp File effective field count:

```text
2
```

if current bootstrap layout requires explicit stamps.

### completion drain / `System.nextCompletion_`

Add per-payload VM-thread conversion and settlement.

Baton restitution happens before this language shaping.

### `core.ph`

Add/revise:

```text
File
IoError getter surface
Fs
Metadata
Permissions
DirEntry
SeekFrom
```

plus continuation shaping.

## 77. Implementation ordering

The implementation should land in green slices.

1. Reconcile/land Resource v2.
2. Land Streams v2.
3. Land Path patch OpenMode semantics.
4. Amend reactor completion/shutdown contract for owned batons.
5. Add plain `IoErrorData`, OpenModeName, SeekFromPlain, StatData, DirEntryData tests.
6. Add ResourceKind::File Idle/Busy and pure ResourceTable tests with temporary host Files.
7. Add explicit close helper tests.
8. Bootstrap File + IoError, all invariant wiring, boot green.
9. Add FileOpen pending-object/adoption path.
10. Test open success/failure/orphan completion before any other File operation.
11. Add File read + baton claim/restitution.
12. Add write/source snapshot.
13. Add sync/seek/metadata.
14. Add ensureIdle_/busy close/flush behavior.
15. Re-run Streams v2 harness against File.
16. Add Fs job family one operation at a time.
17. Add snapshot shaping classes.
18. Add non-UTF8/readDir/canonicalize tests.
19. Add shutdown/cancellation-staleness ownership tests.
20. Verify primitive census and clean worktree.

Do not implement all jobs first and attempt ownership integration afterward.

## 78. Rust unit tests — Resource File state

Use real temporary files where practical.

Required:

```text
adopt -> Idle
claim -> Busy and baton moved out
second claim while Busy -> busy error
restore correct operation -> Idle
restore wrong operation id -> invariant error + no leak
close Idle -> Closed + native close once
close Busy -> busy error + state unchanged
close Closed -> idempotent success
slot reuse -> old File handle stale
File kind leak snapshot works while Idle
Busy leak snapshot still identifies File
```

Add an instrumented/test resource where needed to prove exact close count.

## 79. Rust unit tests — open adoption

Required:

```text
pending File begins unattached
open failure -> no Resource row
open success -> exactly one row + handle installed
adoption table-capacity failure -> native File disposed, no row leak
stale/canceled open completion -> native File disposed, no adoption
pending File path slot preserved
user open site propagated into row
```

## 80. Rust unit tests — restitution

Required:

```text
read completion restores baton before settlement
write completion restores baton before settlement
error completion also restores baton
stale registration completion restores baton but does not settle
wrong FileOperationId cannot install baton into another File
resource generation mismatch cannot install baton into reused slot
```

The returned file handle must never be leaked on invariant-error paths.

## 81. File golden tests — modes

For scratch files:

### read

```text
missing -> IoError #notFound
existing -> opens
write attempt -> IO/permission-style failure from actual mode
```

### write

```text
missing -> creates
existing content -> truncated at open
write works
```

### append

```text
missing -> creates
existing content preserved
writes append
seek cannot cause subsequent append write to overwrite earlier bytes
```

### readWrite

```text
missing -> #notFound
existing content preserved
read works
seek + write works
open alone does not truncate
```

`File.create` duplicates write-mode semantics.

## 82. File golden tests — stream protocol

Re-run Streams v2 tests against File where applicable:

```text
read to EOF
repeated EOF
zero-length read
write reports actual count
zero-length write
flush succeeds when idle
flush while operation pending -> ConcurrentOperationError
double close
use after close
source Bytes mutation after write call does not affect submitted data
one unresolved File operation at a time
close while busy -> ConcurrentOperationError
```

BufferedReader(File) and BufferedWriter(File) must pass their wrapper harness without File-specific
special cases.

## 83. File seek tests

Required:

```text
start(0)
start(end)
current(+n)
current(-n)
end(0)
end(-n)
position == seek(current(0))
negative start rejected synchronously
seek before start where host rejects -> IoError
result Int exact
```

Test shared cursor ordering by awaiting each operation.

Add an overlap negative fixture:

```text
pending read + seek
pending write + position
pending metadata + close
```

all reject the second stateful operation.

## 84. File metadata tests

Required:

```text
size
isFile
isDir false for regular file
timestamp shape Int|None
permissions is Permissions
permissions.isReadOnly
metadata remains usable after File closes
File.metadata describes opened handle even if path is renamed
```

The rename-after-open row proves descriptor metadata is not a fresh path stat.

## 85. Fs error-channel tests

Every IO failure occurs through Future rejection with IoError.

At minimum:

```text
metadata missing           #notFound
open missing               #notFound
create existing collision  #alreadyExists where applicable
permission denial          #permissionDenied where harness can create reliably
removeDir nonempty         #directoryNotEmpty
removeFile directory       appropriate IoError
readDir non-directory      #notDirectory
invalid NUL host path      #invalidInput
```

Contract-type errors happen synchronously before submission.

## 86. `exists` tests

Required:

```text
existing file        true
existing directory   true
missing path         false
component not dir    false
other IO failure     rejected IoError, not false
```

No test uses exists as a prerequisite to another operation.

## 87. Directory operation tests

### createDir

```text
parent exists -> success
parent missing -> error
already exists -> error
```

### createDirAll

```text
nested creation
second call success
intermediate file -> error
```

### removeDir

```text
empty -> success
nonempty -> error
```

### removeDirAll

```text
nested tree removed
symlink inside tree does not traverse/remove target contents
partial-failure behavior does not claim rollback
```

The symlink safety test is non-negotiable on Unix.

## 88. Rename tests

Required:

```text
old disappears
new appears
contents preserved
same-filesystem operation
destination replacement follows Unix host rule
cross-filesystem behavior tested only when harness can provide it portably; otherwise unit-document
```

No implementation pre-checks exists.

## 89. Copy tests

Required:

```text
copy contents
returns exact Int byte count
copies read-only permission state as required
destination missing -> created
destination existing file -> overwritten/truncated
source missing -> #notFound
directory source rejected
```

Do not promise timestamps or xattrs.

## 90. Non-UTF8 path round-trip

Unix-only golden fixture:

1. construct filename bytes containing invalid UTF-8;
2. create/open file through Path.ofBytes;
3. `Fs.readDir(parent).await`;
4. find entry by raw Path bytes;
5. assert `DirEntry.fileName.bytes` exactly equals source bytes;
6. assert `DirEntry.path` reopens the file;
7. assert display is lossy/total;
8. never use display String as the reopen path.

This is the primary end-to-end proof of the Path design.

## 91. Canonicalize tests

Required:

```text
relative path -> host-resolved absolute Path
"." / ".." resolved
symlink resolved
non-UTF8 surviving path components preserved as bytes where host returns them
missing path -> #notFound
```

Do not assert a UTF-8 String representation.

## 92. Snapshot tests

### Metadata

After obtaining Metadata:

```text
close File
rename/remove path
```

accessors still return cached plain values.

### DirEntry

After readDir settlement, accessors are plain and do not perform additional stat calls.

Where practical, instrument worker job count to prove snapshot accessors create no jobs.

## 93. Close failure test seam

Real close errors are difficult to force portably.

Add a test seam around explicit native close so Rust unit tests can inject:

```text
close returns EIO
```

Assert:

```text
Resource tombstone installed
File language state closed
close returns Err(IoError)
second close Ok
native close hook invoked exactly once
```

Do not weaken production ownership to make this test easier.

## 94. Busy-close test

Use a controlled worker/job gate so a File operation remains unresolved.

While Busy:

```text
file.isClosed == false
file.path remains readable
file.close raises ConcurrentOperationError
file.flush raises ConcurrentOperationError
second read/write/seek/... raises ConcurrentOperationError
```

Release worker.

After completion:

```text
File Idle
close succeeds
```

## 95. Cancellation/stale-completion tests

Even before user-facing cancel ships, test the token mechanism directly.

### Open stale

```text
mark registration stale
complete successful FileOpen
drain
```

Assert native file is disposed and no resource row appears.

### File operation stale

```text
claim File
mark Future registration stale
complete operation
drain
```

Assert:

```text
baton restored
File Idle
Future not settled by stale completion
next File operation works
```

This is a critical ownership regression test.

## 96. Shutdown tests

Required integration rows:

### leaked idle File

```text
strictResources(false)
open File
exit
```

Assert:

```text
leak diagnostic names user open site
native close occurs during drain
exit otherwise success
```

### strict leaked File

Same with strict true -> nonzero.

### pending File operation at program end

Shutdown must not snapshot/drain Resource table while worker owns baton.

Assert ordering:

```text
worker completion
restitution
leak snapshot
drain close
```

### stale completion queued at shutdown

Drain restores/disposes ownership correctly even though no Future is settled.

### no Busy rows after worker join

White-box invariant.

## 97. Host-side scratch cleanup guard

Filesystem language fixtures use Fs for the behavior under test.

The outer Rust test harness MUST also install a host-side cleanup guard for the scratch directory.

A panic, VM crash, or early assertion must not leave debris that contaminates later parallel tests.

Unique per-test/per-process scratch paths are required.

Do not rely exclusively on the API under test to clean its own failed fixtures.

## 98. What MUST NOT happen

- No copied `RawFd` job ownership.
- No descriptor number remaining independently closeable while a worker uses it.
- No more than one unresolved native operation per File.
- No implicit operation queue.
- No synchronous wait in File#close.
- No close-while-busy state transition.
- No retry of explicit close after failure/EINTR.
- No reliance on `std::fs::File` Drop for language-level close result.
- No unadopted successful open handle leaked when a Future is stale.
- No stale File-operation completion dropped before baton restitution.
- No Resource shutdown while a worker owns a File baton.
- No worker `Value`, `ObjRef`, VM pointer, or heap access.
- No String path boundary.
- No arbitrary mode-name string dispatch in workers.
- No `position_` primitive.
- No File buffering parameter.
- No `Future<Result>` implementation layer for ordinary async IO.
- No IO failure raised synchronously when it can only be known by the host operation.
- No contract failure encoded as IoError.
- No native construction of ordinary `.ph` Metadata/DirEntry layouts.
- No missing Permissions implementation.
- No user-visible raw POSIX permission integer.
- No silent Float conversion for file sizes/positions/counts.
- No recursive symlink traversal in removeDirAll.
- No check-then-act `exists` pattern in implementation.
- No process-aborting worker allocation for environment-sized results.

## 99. Required documentation amendments on landing

FS v2 exposes several implementation clarifications that should be reflected in the normative
filesystem/reactor docs:

1. async IO success values are Future fulfillments; IO failures are Future rejections with IoError;
2. numeric filesystem counts/sizes/timestamps are Int in the current tower;
3. File permits one unresolved native operation at a time;
4. File close while Busy raises ConcurrentOperationError;
5. File flush is no-op-but-idle-checked; sync is durability;
6. `position` derives from `seek(current(0))`;
7. OpenMode semantics are the closed table in §6;
8. `exists` returns false only for absence/non-directory resolution failure and rejects other IO;
9. external resource completions restore ownership before stale-token discard;
10. reactor orderly shutdown may not abandon workers owning external Resource batons;
11. IoError is bootstrapped because native Future rejection must mint it;
12. File.path is cached and remains readable after close;
13. removeDirAll does not follow symlinks;
14. copy destination overwrite behavior is explicit.

These amendments clarify or reconcile existing accepted behavior; they do not add a new high-level
filesystem feature.

## 100. Acceptance gate

U-FS v2 is implementation-ready only when all of the following are true:

1. Resource v2 has landed;
2. Streams v2 has landed;
3. Path/OpenMode corrective patch has landed;
4. reactor stale-completion logic restores external batons before discarding settlement payloads;
5. reactor shutdown no longer advances past workers that still own File batons;
6. File Resource state has explicit Idle/Busy ownership;
7. worker jobs own `std::fs::File`, never copied RawFd;
8. explicit File close is busy-aware, exactly-once, and fallible;
9. open adoption is atomic with File handle-slot installation;
10. stale successful open completion disposes the native handle;
11. stale ordinary File completion restores the native handle;
12. every File completion returns its baton on success and failure;
13. File enforces one unresolved operation at a time;
14. File write snapshots Bytes before return;
15. File read roots destination until settlement;
16. File flush and close reject while Busy;
17. position is derived and has no primitive;
18. all counts/sizes/positions use checked Int conversion;
19. worker allocations are fallible;
20. IoError is a stable Error subtype that native completion code can mint safely;
21. Permissions is included and tested;
22. all 12 Fs operations have explicit worker/result mappings;
23. non-UTF8 filename round-trip passes end to end;
24. removeDirAll symlink safety is tested;
25. OpenMode host behavior matches the frozen table;
26. File passes Streams v2 conformance;
27. Resource leak/strict shutdown works with real Files;
28. pending-operation shutdown restores baton before drain;
29. primitive census reflects exactly 20 U-FS natives;
30. File and IoError bootstrap/invariant rows are complete;
31. no worker-side GC type appears in any Job/Completion;
32. clean-worktree full test suite is green.
