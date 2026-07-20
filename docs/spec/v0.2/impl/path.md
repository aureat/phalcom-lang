# Implementation spec — `Path` and `OpenMode` (U-PATH)

> **Status:** dispatch-ready. Governing record **Accepted**:
> [PDR-0013](../../../decisions/0013-path-is-bytes-backed-filesystem-surface.md)
> rulings 1-5; surface contract [`../core/filesystem.md`](../core/filesystem.md) §2-§3.
> **Needs U-BYTES (✅ shipped).** No reactor, no resource table, no syscalls — this is a
> **pure `.ph` unit**: zero floor primitives, zero Rust files, no ADR-0019 traffic
> (`utf8Lossy_` already shipped with U-BYTES).
> Read [`bytes.md`](bytes.md) §7 before starting; obligation 1 (`add_class!`) does NOT
> apply here — `Path`/`OpenMode` are ordinary `.ph`-declared classes, not bootstrapped
> kernel rows, precisely because they need no native bindings.
> Anchors verified 2026-07-20 on `9f8a08a`.

## 1. Shape

Two `.ph` classes in `phalcom-core/core/core.ph`, placed after the `class Bytes` block.
`Path` is an immutable value object over an exclusively-owned `Bytes`; `OpenMode` is
four named mode values. Everything is derivation over the shipped `Bytes` protocol.

## 2. `Path`

### 2.1 Construction and ownership (PDR-0013 ruling 1)

```phalcom
class Path {
  construct of(s) {
    // from a String: its UTF-8 bytes — Bytes.fromString already copies
    _bytes = Bytes.fromString(s)
    _hash = Path.contentHash(_bytes)
  }
  construct ofBytes(b) {
    // defensive copy IN: b.slice(0, b.size) — the caller's buffer is never
    // the wrapped one (exclusive ownership is what makes the cached hash
    // and the immutability claim true over a mutable backing store)
    _bytes = b.slice(0, b.size)
    _hash = Path.contentHash(_bytes)
  }
  ...
}
```

- Argument validation raises `ArgumentError` (`throw ArgumentError.new(...)`, the
  `core.ph` house idiom — see `Bytes#set`).
- `bytes` accessor copies OUT the same way (`_bytes.slice(0, _bytes.size)`).
- **No selector ever hands `_bytes` to a caller and no selector ever mutates it.** This
  is a written contract enforced by review + the aliasing harness row (§5), the
  ADR-0052 posture — Phalcom has no `private`.

### 2.2 Value semantics (ruling 2)

- `==(other)`: `other.isA(Path)` guard, then hash-first —
  `(_hash == other.hash) and (_bytes == other.bytes)`. The `other.bytes` copy per
  comparison is accepted: paths are short, and hash-first keeps the common unequal
  case O(1). (`Bytes#==` does the structural walk.) `!=` routes through `==`
  (`return not (self == other)` — the `List#!=` decoupling rule, `core.ph`).
- `hash => _hash` — cached at construction, sound because of §2.1. Compute with
  `Range#hash`'s exact shape (`core.ph`, `acc = (acc * 31 + byte) % 999999937` over
  `_bytes`), as a `static contentHash(bytes)` helper.
- Immutable + value-hashed ⇒ a valid `Map`/`Set` key (collection-protocol law 4's
  immutable branch) — harness row asserts a `Path` keys a `Map` and two equal-content
  `Path`s hit the same entry.

### 2.3 Lexical operations (ruling 3) — all over `Bytes`, none touch the filesystem

Separator is `/` (byte 47); Windows normalization is PDR-0013 Q-1, out of scope.

| Selector | Derivation |
|---|---|
| `isAbsolute` | `_bytes.size > 0 and _bytes.at(0) == 47` |
| `join(other)` | `other.isA(Path)` guard; absolute `other` **replaces** the receiver (Rust `Path::join`'s rule, filesystem.md §2); else `Path.ofBytes(_bytes ++ "/" ++ other)` — via `concat`, collapsing a trailing separator on the receiver so exactly one separator joins them. Never normalizes `.`/`..` (law 4) |
| `parent` | scan backward for the last separator; `None` at a root (`/` or empty); strips trailing separators first |
| `fileName` | bytes after the last separator as a new `Path`; `None` for a root or a path ending in `/` |
| `extension` | within `fileName`'s range, bytes after the last `.` (byte 46), decoded **strictly** via `utf8`; `None` if absent, if the name starts with the dot (`.bashrc` has no extension), or if not UTF-8 |
| `components` | split on runs of separators, each component a `Path`; no empty components; leading `/` is not a component |
| `toString` | `_bytes.utf8Lossy` — total, display-only (ruling 4; never round-tripped, law 2) |

Implement the scans as `while` loops over `at(i)` — index math, no combinators needed;
where a block IS used, flat-entry makes it yield-safe either way (bytes.md §7.3).

### 2.4 What `Path` must NOT have

No `exists`, no `canonicalize`, no stat of any kind (ruling 3 — those are `Fs`'s,
U-FS); no mutation selectors; no implicit `String` coercion anywhere (filesystem.md §4:
a `String` where a `Path` is due is a **type error**).

## 3. `OpenMode`

Four mode values with **value identity**: `read` / `write` / `append` / `readWrite`
(spellings ratified in PDR-0005 §7). The tree's static-accessor precedent
(`Tracer.stdout => Tracer.new()`, `core.ph`) returns a fresh instance per call —
`.ph` classes have no static-field memo — so mode identity must not be pointer
identity:

```phalcom
class OpenMode {
  construct named_(n) { _name = n }
  static read => OpenMode.named_("read")
  static write => OpenMode.named_("write")
  static append => OpenMode.named_("append")
  static readWrite => OpenMode.named_("readWrite")
  name => _name
  ==(other) { return other.isA(OpenMode) and (_name == other.name) }
  !=(other) { return not (self == other) }
  toString => "OpenMode." + _name
}
```

`OpenMode.read == OpenMode.read` is `true` by value, which is the whole observable
contract (filesystem.md §3). No numeric mode, no flags — ever (ruling 5). U-FS later
matches on `name` at the native boundary.

## 4. Ordering

1. `class OpenMode` (self-contained), then `class Path`, both after `class Bytes` in
   core.ph. Boot green after each (`cargo run -p phalcom-core --bin phalcom` on a
   smoke file).
2. Golden lanes (§5). Clean-worktree verify.

Single file touched (`core.ph`) plus tests — stage and `git commit -- ` exactly those.

## 5. Test plan — golden lanes `path/` + `path/negative/`

| Check | Asserts |
|---|---|
| construction + display | `Path.of("a/b.txt").toString`; a non-UTF-8 `Path.ofBytes` displays with U+FFFD and `bytes` round-trips unchanged (filesystem.md §8 rows 1-2) |
| exclusive ownership | mutate the source `Bytes` after `ofBytes`, and the buffer returned by `bytes` — the `Path` observes neither (ruling 1) |
| value semantics | `==`/`!=`/cached `hash` agreement; equal-content paths key one `Map` entry; `Path` accepted as a `Map` key (unlike `Bytes`) |
| lexical table | every §2.3 selector: absolute/relative `join`, absolute-replaces, `parent` to root and `None` past it, `fileName`/`extension` edge cases (`.bashrc`, trailing `/`, no dot), `components` on `//` runs, no normalization (`Path.of("a/../b") != Path.of("b")`) |
| `OpenMode` | the four values, `==` across separately-obtained instances, `!=` across different modes, `toString` |
| negative | `Path.of(42)` raises; `Path.ofBytes("str")` raises; `join` with a `String` raises (no coercion) |

## 6. Not in this unit

`File`/`Fs` (U-FS), `SeekFrom` (U-FS — same value-object pattern as `OpenMode`),
Windows path rules (Q-1), any path literal (Q-5).
