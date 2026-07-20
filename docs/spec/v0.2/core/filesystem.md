# Specification — Filesystem (`Path`, `File`, `Fs`, `DirEntry`, `Metadata`, `Permissions`, `OpenMode`)

> **Status:** split authority, stated per section. The `File` selector surface (§4) encodes
> [PDR-0005](../../../decisions/0005-resources-are-disposable-handles-not-finalized.md) §7,
> **Accepted** — normative now. Everything else (`Path`, `Fs`, `DirEntry`, `Metadata`,
> `Permissions`, `OpenMode` beyond its ratified spellings) encodes
> [PDR-0013](../../../decisions/0013-path-is-bytes-backed-filesystem-surface.md),
> **Proposed — normative upon its ratification** (`decisions/README.md` rule 5 applies).
> Depends on [`bytes.md`](bytes.md) / [PDR-0011](../../../decisions/0011-admit-bytes-native-octet-buffer.md)
> (Proposed) for the backing store, and on the reactor
> ([PDR-0004](../../../decisions/0004-io-is-future-shaped-reactor-owned.md) §2) before any
> implementation. **Floor delta: nonzero, enumerated in PDR-0013 ruling 10**, censused at
> impl time under PDR-0012 ruling 21's rebase discipline.
> Selector spellings follow ADR-0012 (labelled-parameter comma form) and ADR-0043 (no
> default arguments, no flags — every variant is its own selector).
>
> **Owner:** unassigned.

## 1. Scope and the shape of the surface

Three kinds of thing, split by what can block (PDR-0004 §1):

| Kind | Blocks? | Returns |
|---|---|---|
| `Path` operations (§2) | never — lexical byte manipulation | plain values |
| `File` handle operations (§4) | yes (except `path`, `close`) | `Future`; `close` is the synchronous `Result` from `Resource` |
| `Fs` path-addressed operations (§5) | yes, all of them | `Future` |
| `Metadata`/`DirEntry`/`Permissions` accessors (§6) | never — cached snapshot fields | plain values |

Everything that blocks runs on the worker pool (PDR-0004 §3/§4): workers see owned plain
data (path bytes, buffers), never a `Value`, and completions settle at the dispatch
safepoint.

## 2. `Path`

**Not a `String`.** `StringObject` enforces UTF-8 and caches a content hash
(`heap/string.rs:11-16`); POSIX paths are arbitrary bytes. A `String` path cannot hold
every name `readDir` can return — Python's `surrogateescape` retrofit is the cost of
pretending otherwise (PDR-0013 Context).

`Path` is an immutable `.ph` value class over a `Bytes` it owns exclusively (construction
copies in, `bytes` copies out — PDR-0013 ruling 1). Structural `==`, content `hash` cached
at construction, sound because of the exclusive ownership (ruling 2). Immutable +
value-hashed ⇒ **a valid `Map`/`Set` key** (collection-protocol law 4).

| Selector | Returns | Meaning |
|---|---|---|
| `Path.of(_)` | `Path` | from a `String` (its UTF-8 bytes) |
| `Path.ofBytes(_)` | `Path` | from a `Bytes` (defensive copy) |
| `join(_)` | `Path` | lexical append with exactly one separator; an absolute argument replaces the receiver (Rust `Path::join`'s rule) |
| `parent` | `Path` \| `None` | lexical parent; `None` at a root |
| `fileName` | `Path` \| `None` | final component; `None` for a root or a path ending in `..` |
| `extension` | `String` \| `None` | after the last `.` of the final component, decoded strictly; `None` if absent or not UTF-8 |
| `isAbsolute` | `Bool` | leading separator |
| `components` | `List` | of `Path`, split on separators, no normalization |
| `bytes` | `Bytes` | defensive copy of the octets |
| `toString` | `String` | **lossy** display (invalid UTF-8 → U+FFFD) via `Bytes#utf8Lossy_` (PDR-0013 ruling 4). For humans only |
| `==(_)` / `!=(_)` / `hash` | | value semantics (ruling 2) |

**Laws:**

1. **Lexical only.** No `Path` selector touches the filesystem, resolves `..`, or follows
   a symlink. `Fs.canonicalize(_)` is the selector that does (ruling 3).
2. **Syscalls take bytes.** Every native crossing uses the octets; `toString` is display,
   never round-tripped into a syscall (ruling 4).
3. **No aliasing.** No `Bytes` the caller can reach is the wrapped buffer, in either
   direction.
4. **`join` never normalizes.** `a.join(Path.of(".."))` keeps the `..`; equality is
   byte equality, so `Path.of("a/../b") != Path.of("b")` — resolving is `canonicalize`'s
   job, at syscall cost, with syscall truthfulness.

## 3. `OpenMode`

Four singleton objects, no numbers, no flags (PDR-0013 ruling 5; spellings from
PDR-0005 §7): `OpenMode.read`, `OpenMode.write` (write + truncate), `OpenMode.append`,
`OpenMode.readWrite`. Future sealed-enum members if the feature lands; until then a
plain `.ph` class with four static instances and `toString`.

## 4. `File` — encoding PDR-0005 §7, ratified

`File < Resource` ([`stream-protocol.md`](stream-protocol.md) §3 laws apply verbatim:
synchronous idempotent fallible `close`, use-after-close raises `kind: #useAfterClose`).
The descriptor lives in the VM-side generation-tagged resource table (PDR-0005 §4), never
in the object — GC sweep drop glue must have nothing OS-visible to drop.

| Selector | Returns | Meaning |
|---|---|---|
| `File.open(_)` | `Future` | open read-only; settles to `Ok(File)`/`Err` |
| `File.create(_)` | `Future` | create, write + truncate |
| `File.openWith(_, mode:)` | `Future` | one selector, labelled parameter (ADR-0012); mode is an `OpenMode` singleton |
| `read(_)` | `Future` | fill the given `Bytes`, settle to count; **0 = EOF** (stream law 1) |
| `write(_)` | `Future` | direct syscall — `File` is unbuffered (PDR-0005 §3c); settle to count accepted |
| `sync` | `Future` | fsync — the explicit blocking residue (§3b) |
| `seek(_)` | `Future` | `SeekFrom.start(_)` / `.current(_)` / `.end(_)` |
| `position` | `Future` | current offset |
| `metadata` | `Future` | fstat snapshot (§6) |
| `path` | `Path` | cached at open; deliberately **not** a `Future` (PDR-0004 §1 scope note) |
| `close` | `Result` | from `Resource` |

Buffering is a wrapper, never a parameter (stream-protocol §4): `BufferedWriter.new(f)`,
`BufferedReader.new(f)`; their contract — including the `#unflushed` dirty-close raise and
`finish` — is stream-protocol §5, not restated here.

All arguments named "a path" are `Path`. A `String` argument is a type error, not a
convenience coercion — silent `String`→`Path` would re-open the door ruling 1 closed.
Write `File.open(Path.of("a.txt"))`; if that reads as ceremony, that is Q-5's territory,
ruled there rather than by an implicit coercion.

## 5. `Fs`

Path-addressed, one syscall each, all `Future` (PDR-0004 §1 — including `exists`: a live
stat blocks), all worker-pool. Eleven selectors, each its own name, no `recursive:` flag,
no options bag (ADR-0043; PDR-0013 ruling 6):

| Selector | Settles to | Meaning |
|---|---|---|
| `Fs.exists(_)` | `Bool` | the path resolves; **advisory only** — see law 4 (TOCTOU) |
| `Fs.metadata(_)` | `Result` of `Metadata` | stat; **follows symlinks** |
| `Fs.symlinkMetadata(_)` | `Result` of `Metadata` | lstat — the no-follow spelling |
| `Fs.readDir(_)` | `Result` of `List` | of `DirEntry`; whole directory in one settlement (streaming is Q-3) |
| `Fs.createDir(_)` | `Result` | one level; parent must exist |
| `Fs.createDirAll(_)` | `Result` | intermediate levels too; `Ok` if already present |
| `Fs.removeFile(_)` | `Result` | unlink; not directories |
| `Fs.removeDir(_)` | `Result` | rmdir; empty only |
| `Fs.removeDirAll(_)` | `Result` | recursive delete — the one deliberately dangerous selector, named loudly for it |
| `Fs.rename(_, to:)` | `Result` | atomic where the OS gives it (same filesystem) |
| `Fs.copy(_, to:)` | `Result` of `Number` | bytes copied; contents + permissions, nothing else |
| `Fs.canonicalize(_)` | `Result` of `Path` | resolve `..`/symlinks against the real filesystem — the counterpart to `Path` law 1 |

## 6. `Metadata`, `DirEntry`, `Permissions`

All three are immutable snapshots: one syscall creates them, accessors read cached fields
and never block, so none returns a `Future` (PDR-0013 ruling 7; the PDR-0004 §1
"cached stat" case).

**`Metadata`:** `size -> Number` (bytes), `isFile`/`isDir`/`isSymlink -> Bool`,
`modified`/`accessed`/`created -> Number | None` (integral **milliseconds since the Unix
epoch**, ruling 8 — exact in f64; `None` where the OS does not track it),
`permissions -> Permissions`. Staleness is inherent to stat and documented, not fought.

**`DirEntry`:** `fileName -> Path` (final component only), `path -> Path` (joined with the
`readDir` argument), `isFile`/`isDir`/`isSymlink -> Bool` (from the readdir record — free
on most OSes, no extra stat). No `metadata` accessor: call `Fs.metadata(entry.path)` and
pay the syscall visibly.

**`Permissions`:** `isReadOnly -> Bool`. Nothing else yet (ruling 9; POSIX mode is Q-2 —
never a raw `Number`).

## 7. Laws, consolidated

1. **Blocking is visible in the type** (PDR-0004 §1): `Path`/snapshot accessors return
   plain values; every syscall-bearing selector returns `Future`; `close` alone is the
   synchronous `Result` (stream-protocol §3).
2. **Paths are bytes end to end.** Any byte sequence `readDir` returns can round-trip
   through `DirEntry#path` into every `Fs`/`File` selector unchanged. Display may be
   lossy; the pipeline never is.
3. **`Err` for the world, raise for the caller.** IO failures (`#notFound`,
   `#permissionDenied`, `#alreadyExists`, …) settle to `Err`; contract violations — wrong
   type, use-after-close, non-`OpenMode` mode — raise (stream-protocol law 5's split).
4. **`exists` proves nothing** (Q-4 pending a normative ruling): between its settlement
   and the next operation the world may change. The blessed idiom is open-and-handle-`Err`,
   not check-then-open.
5. **One selector, one operation.** No selector both checks and acts, both creates and
   opens differently on a flag, or takes an options bag (ADR-0043).

## 8. Conformance harness

Runs against a scratch directory; every row is a `.ph` golden test once the reactor
exists.

| Check | Asserts |
|---|---|
| byte round-trip | law 2; a file created with a deliberately non-UTF-8 name is returned by `readDir` and re-openable via `DirEntry#path` |
| lossy display | `Path#toString` on that name is total and contains U+FFFD; `bytes` is unchanged |
| `Path` value semantics | `==`/`hash` agree for equal byte content; `Path` keys a `Map` |
| lexical laws | `join`/`parent`/`fileName`/`components` per §2's table; `join` with absolute replaces; no selector touches the filesystem (runs with no scratch dir) |
| open modes | `read` refuses writes; `write` truncates; `append` appends; each via its own selector |
| EOF | `read(_)` at end settles `0`, repeatedly (stream law 1) |
| close contract | double-close `Ok`; use-after-close raises `#useAfterClose` (stream harness rows, re-run against `File`) |
| `Fs` errors are `Err` | `metadata` on a missing path settles `Err(#notFound)`, never raises |
| snapshot non-blocking | `Metadata`/`DirEntry` accessors return plain values (no `Future` in any accessor position) |
| `createDirAll` idempotent | second call `Ok` |
| `rename` visibility | old path `#notFound`, new path opens |
| leak report | an unclosed `File` at exit appears in `System.leakReport` naming its open site (PDR-0005 §5) |

## 9. Open questions

PDR-0013's Q-1 (Windows), Q-2 (POSIX mode surface), Q-3 (streaming `readDir`), Q-4
(TOCTOU posture), plus:

| # | Question | Notes |
|---|---|---|
| Q-5 | `Path` literal or coercion ergonomics | `File.open(Path.of("a.txt"))` is honest but heavy. A path literal is a lexer question (BY-1's sibling); implicit `String` coercion is ruled out by §4 — anything else needs its own ruling |

## 10. What this document does not cover

- **Sockets, DNS, TLS, process spawn.** Same reactor, separate spec.
- **The reactor and worker pool themselves** — [`reactor.md`](reactor.md).
- **File watching, memory mapping, locks.** No design, no owner.
- **A `Time` type.** Ruling 8's `Number` milliseconds is the contract; wrapping it is
  future work that changes no accessor here.
