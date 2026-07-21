# PDR-0013 — `Path` is bytes-backed, not a `String`; the filesystem surface

- Status: **Accepted** (ratified 2026-07-20, same day as proposed)
- Date: 2026-07-20
- Related: [PDR-0005](0005-resources-are-disposable-handles-not-finalized.md) §7 (the `File`
  selector surface, **already ratified there** — this record adds the types around it, it does
  not reopen them), [PDR-0004](0004-io-is-future-shaped-reactor-owned.md) §1/§3 (every
  blocking selector returns `Future`; the filesystem half runs on the worker pool),
  [PDR-0011](0011-admit-bytes-native-octet-buffer.md) (`Bytes`, the backing store — this
  record **composes with it** and depends on its ratification),
  [PDR-0012](0012-numeric-tower-implementation-and-floor-amendment.md) ruling 21's
  rebase discipline (three Proposed floor amendments now share the 137 base; whichever
  ratifies later rebases),
  [ADR-0043](../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md)
  (no flags parameters — every variant is its own selector),
  [ADR-0024](../adr/accepted/0024-numeric-surface-split-int-float-and-division.md) /
  PDR-0012 (why no numeric mode bits appear in any signature)
- Spec: [`docs/spec/current/stdlib/filesystem.md`](../spec/current/stdlib/filesystem.md) holds the
  protocol, laws, and harness.

## Context

PDR-0005 §7 ratified `File`'s selectors, and `File#path -> Path` names a type that has no
design. The obvious shortcut — `Path` *is* `String` — is wrong on this tree as a matter of
representation, not taste: `StringObject` wraps a Rust `String` (UTF-8 **enforced**) and
caches a content hash at construction (`heap/string.rs:11-16`), while POSIX paths are
arbitrary bytes with no encoding. A `String`-typed path cannot even *hold* every path the
OS can hand back from `readDir`. Python shipped `str` paths, hit exactly this, and had to
retrofit `os.fsencode`/`surrogateescape` across the entire stdlib; Rust's `OsStr`/`PathBuf`
split exists for the same reason.

## Rulings

1. **`Path` is a distinct immutable value class backed by octets, authored in `.ph` over
   `Bytes`.** Not a `String` (above); not a bare `Bytes` either — `Bytes` is mutable,
   identity-hashed, and has no path operations. `Path` wraps a `Bytes` it **owns
   exclusively**: construction copies in, `bytes` copies out, no reference the caller holds
   is ever the wrapped buffer. Exclusive ownership is what makes an immutability claim about
   a mutable backing store true rather than aspirational.
2. **`Path` is value-semantic: structural `==`, content `hash` cached at construction.**
   The cache is sound *because of* ruling 1 (nothing can mutate the wrapped buffer), the
   same argument `StringObject` uses for its cached hash. Immutable + value-hashed ⇒ a
   valid `Map`/`Set` key (collection-protocol law 4's immutable branch) — paths as keys is
   the common case and must work.
3. **Path operations are lexical, `.ph`, and never touch the filesystem.** `join`,
   `parent`, `fileName`, `extension`, `isAbsolute`, `components` are byte manipulation over
   the `Bytes` protocol; none can block, so none returns a `Future` (PDR-0004 §1's
   dividing line). Anything that must consult the real filesystem — resolving `..`,
   following symlinks — is `Fs.canonicalize(_) -> Future`, not a `Path` method. This is
   Rust's `Path::join` vs `fs::canonicalize` split, taken deliberately.
4. **Display is lossy, and honest about it: one new `Bytes` floor primitive,
   `utf8Lossy_`.** `Path#toString` must be total (`Object#toString` contract), but a path
   may not be UTF-8. Strict `utf8_` returns `None`; a *lossy* decode (invalid sequences →
   U+FFFD, Rust's `from_utf8_lossy`) cannot be written in `.ph` — no primitive builds a
   `String` from arbitrary octets — and is a no-user-code bulk operation, so it admits
   under PDR-0011 ruling 3's posture. Carried by **this** record (+1, on class `Bytes`)
   rather than by amending PDR-0011, so the two Proposed records stay independently
   ratifiable; PDR-0012 ruling 21's rebase discipline absorbs the arithmetic.
   Round-tripping law: display is for humans; **syscalls always take the bytes**, never
   the `toString`.
5. **`OpenMode` members are singleton objects, not numbers.** `OpenMode.read` /
   `.write` / `.append` / `.readWrite` (spellings ratified in PDR-0005 §7). No numeric
   mode, no bit flags in any signature — there is no `Int`, `Number` is f64
   (PDR-0012 pending), and a flags word is exactly the shape ADR-0043 exists to forbid.
   Singletons become sealed-enum members if that feature ever lands.
6. **`Fs` is the namespace for path-addressed operations; every one returns `Future`;
   every variant is its own selector.** `exists`, `metadata`, `readDir`, `createDir`,
   `createDirAll`, `removeFile`, `removeDir`, `removeDirAll`, `rename`, `copy`,
   `canonicalize` — eleven selectors, no `recursive:` flag, no options object
   (ADR-0043). All hit the filesystem, so all are `Future` (PDR-0004 §1) and all run on
   the worker pool (§3) — including `exists`: a live statfs is a blocking syscall; the
   "cached stat" carve-out in PDR-0004 §1 describes `Metadata` accessors, not `Fs`.
7. **`Metadata` is an immutable snapshot with plain accessors.** One syscall at
   `Fs.metadata`/`File#metadata`; the returned object's accessors (`size`, `isFile`,
   `isDir`, `isSymlink`, `modified`, `permissions`) read cached fields and return plain
   values, no `Future`. Staleness is inherent to `stat` on every OS and is documented,
   not fought. `Fs.metadata` **follows symlinks**; `Fs.symlinkMetadata` is the `lstat`
   spelling (separate selector, ADR-0043 again).
8. **Timestamps are `Number` milliseconds since the Unix epoch.** No `Time` type exists
   and this record does not invent one; f64 holds integral milliseconds exactly until far
   past year 285,000. Worded representation-independently ("integral milliseconds") so a
   future `Time` type or PDR-0012's `Int` changes nothing here. A future `Time` wraps
   this value; it does not replace the accessor.
9. **`Permissions` starts minimal: `isReadOnly -> Bool`.** POSIX mode bits are real but
   platform-shaped; surfacing them is Q-2, not a silent `Number` getter.
10. **Floor delta, enumerated but censused at impl time.** New natives (all behind the
    worker pool per PDR-0004 §4, all `Future`-settling except the `Resource` pair and
    the `Bytes` addition): `Resource#close_`/`isClosed_` (the generation-tagged table,
    PDR-0005 §4); `File`'s `open_`/`create_`/`openWith_`/`read_`/`write_`/`sync_`/
    `seek_`/`position_`/`metadata_`; `Fs`'s eleven from ruling 6; `Bytes#utf8Lossy_`.
    Exact census arithmetic (base 137 + PDR-0011's 10 + PDR-0012's 16 + these) follows
    PDR-0012 ruling 21: whichever Proposed record ratifies later rebases. The count is
    deliberately *not* frozen here — the reactor spec may fold `sleep_` and completion
    plumbing into the same amendment.

## Open questions

| # | Question | Notes |
|---|---|---|
| Q-1 | Windows paths | UTF-16-ish, not arbitrary bytes — WTF-8 storage is the known answer (Rust `OsStr`). Rulings are worded byte-level and survive it; the *normalization* rules (`\` vs `/`, drive letters) do not exist yet. Blocked on caring about Windows |
| Q-2 | POSIX mode surface | `Permissions` beyond `isReadOnly`: mode as what? Symbols (`#ownerRead`)? A `Permissions` builder? Never a raw `Number` in a signature |
| Q-3 | Streaming `readDir` | Ruling 6 settles the whole `List` in one `Future`. A million-entry directory wants an async iterator — which is a `Future`⊗`Iterable` interaction with no design. Deferred until the reactor exists |
| Q-4 | TOCTOU posture | `exists`-then-`open` races are inherent; does the spec bless the check-free idiom (`open` and handle the `Err`) normatively? |

## Consequences

- `File#path -> Path` (PDR-0005 §7) finally has a type, and it is not a lie about UTF-8.
- Every `Fs`/`File` primitive takes `Path` **bytes** across the native boundary — the
  worker pool receives owned plain data (`PathBuf` from bytes, PDR-0004 §4), never a
  `Value`.
- A third Proposed floor amendment now shares the 137 base; the PDR-0012 ruling 21 rebase
  discipline is doing real work and STATUS.md must keep the composition note current.
- Implementation is **double-blocked**: on this record's ratification (rule 5) and on the
  reactor (PDR-0004 §2 — surface never ships before the machinery it settles on).
