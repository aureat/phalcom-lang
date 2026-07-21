# Implementation spec — `File`, `Fs`, and the snapshot types (U-FS)

> **Status:** dispatch-ready, **last in the chain**. Governing records **Accepted**:
> [PDR-0005](../../../pdr/0005-resources-are-disposable-handles-not-finalized.md) §7
> (the `File` surface, ratified there — encode, don't redesign),
> [PDR-0013](../../../pdr/0013-path-is-bytes-backed-filesystem-surface.md); surface
> contract [`../stdlib/filesystem.md`](../../spec/current/stdlib/filesystem.md) §4-§8.
> **Needs shipped: U-BYTES ✅, U-RESOURCE, U-PATH, U-REACTOR** (U-STREAMS is not a hard
> dependency but its harness is reused for `File`'s Reader/Writer conformance).
> **Floor delta: 19** — 7 `File` natives + 12 `Fs` natives (`NEW_FS`), all taking the
> pending future as their last argument (the U-REACTOR registration rule). Surface
> selector count is larger than native count on the `File` side deliberately:
> ADR-0043's three open spellings share one native (§2.4).
> Read [`bytes.md`](bytes.md) §7 first; obligations 1 (`add_class!` — `File` only) and
> the GC-root rule from [`reactor.md`](reactor.md) §2.1 both apply.
> Anchors as of `e10951a`.

## 1. Shape

The filesystem half of PDR-0004 §3: every operation is a `Job` on U-REACTOR's worker
pool (regular files are not pollable — no poller involvement at all), every handle is a
row in U-RESOURCE's table, every path crosses the native boundary as **bytes**
(filesystem.md law 2). `.ph` composes the contract futures; natives only register jobs.

## 2. File-by-file

### 2.1 `reactor.rs` — `Job`/`Payload` extensions

New `Job` variants, plain data only (the structural boundary): `FileOpen { path:
PathBuf, mode: OpenModeName, site: SourceRange }`, `FileRead { fd: RawFd, len: usize }`,
`FileWrite { fd: RawFd, data: Vec<u8> }`, `FileSync { fd }`, `FileSeek { fd, from:
SeekFromPlain }`, `FileStat { fd }`, and the `Fs*` family mirroring §2.5's table
(`FsStat { path, follow: bool }`, `FsReadDir { path }`, …). `PathBuf` is built from
path bytes via `OsString::from_vec` (unix; the Windows branch is PDR-0013 Q-1,
`compile_error!` on non-unix for now rather than a silent wrong path type). New
`Payload` variants: `Fd(RawFd, SourceRange)`, `Stat(StatData)`, `DirList(Vec<DirEntryData>)`,
`PathBytes(Vec<u8>)` — each a plain-data struct. Errors cross as `IoErrorData { name:
&'static str /* "notFound", "permissionDenied", … */, message: String }`, mapped from
`std::io::ErrorKind` in one match, in `reactor.rs`, tested directly.

### 2.2 `System.nextCompletion_` — kind-aware minting

The one place plain data becomes `Value`s (PDR-0004 §4) grows per-payload arms:
`Fd` → open a `ResourceKind::File(fd)` row in U-RESOURCE's table (open site from the
job) and mint the packed handle `Number`; `Stat` → a `Tuple` of plain values in a
documented slot order; `DirList` → a `List` of `Tuple`s (name-`Bytes`, `isFile`,
`isDir`, `isSymlink`); `PathBytes` → a `Bytes`; `IoErrorData` → an `IoError` instance
(§2.6). `.ph` shapes these into `File`/`Metadata`/`DirEntry` instances — minting stays
dumb, shaping stays `.ph`.

### 2.3 `ResourceKind::File(RawFd)` — U-RESOURCE extension

The kind-specific close branch runs `close(2)`; per PDR-0005 §3b that is synchronous
and legal inside `Resource#close_` (nothing to flush — `File` is unbuffered, §3c). An
`EIO` from `close(2)` surfaces as the `Err` arm of `close`'s `Result` (stream-protocol
law: close is fallible; discarding it is the Go-linter bug). No other native touches
the fd except through jobs.

### 2.4 `primitive/file.rs` — 7 natives

| Native | Serves | Notes |
|---|---|---|
| `File.open_(_,_,_)` static | `.ph` `File.open`/`File.create`/`File.openWith(_, mode:)` | path-`Bytes`, mode-name `String`, future. **One native, three surface selectors** — ADR-0043 governs selector identity, not native plumbing; the `.ph` spellings pass `"read"`/`"write"`/mode.name |
| `read_(_,_)` | `File#read(_)` | resolves handle → fd (raises `UseAfterCloseError` on stale/closed — before job submission, not after), registers `FileRead` sized to the dst `Bytes` |
| `write_(_,_)` | `File#write(_)` | snapshots the src `Bytes` into the job's `Vec<u8>` (caller may mutate after; direct syscall, §3c) |
| `sync_(_)` / `seek_(_,_,_)` / `position_(_)` / `metadata_(_)` | the rest of PDR-0005 §7's instance surface | `seek_` takes `(fromName, offset, future)`; `position` is `seek(current, 0)` in `.ph` — no separate job |

`.ph` `File < Resource` (bootstrapped class — **`add_class!` obligation applies**;
field stamp: handle slot + `_path` slot): `construct` is not public — instances are
minted by `open`'s settle chain. `read(dst)` composes: raw future settles a payload
`Bytes` + count; `.ph` `then`-chain copies into `dst` via `copyInto` and settles the
count (U-REACTOR's compose pattern). `path => _path` (a `Path`, cached at open —
deliberately not a `Future`, PDR-0004 §1 scope note). Reader/Writer/Seekable conformance
comes from responding to the selectors — no declaration (stream-protocol §1).

### 2.5 `primitive/fs.rs` — 12 natives

One per filesystem.md §5 row (`exists_`, `metadata_`, `symlinkMetadata_`, `readDir_`,
`createDir_`, `createDirAll_`, `removeFile_`, `removeDir_`, `removeDirAll_`,
`rename_(_,_,_)`, `copy_(_,_,_)`, `canonicalize_`) — each takes path-`Bytes` (×2 for
`rename_`/`copy_`), registers the job, returns `None`. **No string-dispatched
mega-native**: per-op natives keep the census honest and the job mapping greppable.
`Fs` is `System`-style statics on a `.ph` `class Fs {}`; `.ph` wrappers validate
`Path`-typed arguments (a `String` raises — filesystem.md §4's no-coercion rule) and
compose the `Result`-settling futures (`Err(IoError)` from the payload, never a raise
for IO outcomes — law 3's channel split).

### 2.6 `.ph` snapshot types — core.ph

- `class IoError extends Error` with a `name` symbol-ish field (`#notFound`,
  `#permissionDenied`, `#alreadyExists`, mapped 1:1 from §2.1's table). The
  class-carries-kind pattern (U-RESOURCE §2.4); migrates to PDR-0010's `kind` when
  T3/T6 land, no surface change.
- `Metadata`: plain fields from the `Stat` tuple; accessors `size`/`isFile`/`isDir`/
  `isSymlink`/`modified`/`accessed`/`created` (integral-ms `Number | None`, PDR-0013
  ruling 8)/`permissions` (a `Permissions` with `isReadOnly` only — ruling 9). No
  `Future` in any accessor (the snapshot law).
- `DirEntry`: `fileName` (`Path`), `path` (joined with the `readDir` argument, lexical
  `join`), the three `Bool`s from the readdir record. No `metadata` accessor —
  `Fs.metadata(entry.path)` pays the syscall visibly (filesystem.md §6).
- `SeekFrom`: `start(_)`/`current(_)`/`end(_)` value objects — `OpenMode`'s exact
  pattern (U-PATH §3), name + offset.

### 2.7 Census

`NEW_FS: usize = 19` + class rows for `File` (bootstrapped). `IoError`/`Metadata`/
`DirEntry`/`SeekFrom`/`Fs` are `.ph`-declared, not censused. Verify the live baseline
against `floor_census_matches_installed_bindings` first — U-REACTOR's +3 and possibly
PDR-0012's tower land before this unit.

## 3. Ordering

1. `Job`/`Payload` extensions + the `ErrorKind` map (pure Rust, tested without a VM).
2. `ResourceKind::File` + close branch (U-RESOURCE harness extended with a real fd).
3. Minting arms in `nextCompletion_`.
4. `file.rs` + `File` bootstrap (all four registration sites!) + `.ph` `File`.
5. `fs.rs` + `.ph` `Fs`/snapshot types.
6. Golden lanes (§5) against a scratch dir. Clean-worktree verify.

## 4. What must NOT happen

- No `String` path crosses any boundary; no `Path`→`String`→syscall laundering
  (law 2 — the whole point of PDR-0013).
- No blocking syscall on the VM thread except `close(2)` inside `close_` (§2.3's
  narrow, ruled exception — it is "not a blocking operation", PDR-0005 §3b).
- No `Value`/`ObjRef` in any `Job`/`Payload`/worker path.
- IO failures never raise; contract violations never `Err` (law 3). `exists` proves
  nothing — fixtures use open-and-handle-`Err`, and the docs teach it (Q-4's posture).
- No `recursive:`/options-bag parameters anywhere (ADR-0043).

## 5. Test plan

filesystem.md §8's harness, row for row, in golden lanes `fs/` + `fs/negative/`
against a per-run scratch directory (the harness builds and removes it via `Fs`
itself — self-hosting cleanup, with `strictResources(true)` set so leaked `File`s
redden the lane). Non-negotiable rows: the **non-UTF-8 filename byte round-trip**
(create via `Path.ofBytes`, re-open via `DirEntry#path` — law 2's proof), lossy
display of that name, EOF repeatability, double-close/use-after-close re-run against
real fds, `Err(#notFound)` shape, `createDirAll` idempotence, `rename` visibility,
leak report naming the open site. Plus stream-protocol §8 re-run with `File` as the
Reader/Writer (U-STREAMS harness parameterized over the stream type).

## 6. Not in this unit

Sockets/DNS/TLS/process (network unit, needs the poller), file watching, locks, mmap,
streaming `readDir` (PDR-0013 Q-3), POSIX mode surface (Q-2), Windows (Q-1), `Time`
type (ruling 8's `Number` ms is the contract).
