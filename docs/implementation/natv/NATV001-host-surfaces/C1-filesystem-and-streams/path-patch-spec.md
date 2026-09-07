# Path Corrective Patch Specification

> **Status:** proposed corrective patch over the shipped U-PATH implementation.
> This document does not redesign `Path` or `OpenMode`. It fixes one observable lexical bug,
> freezes previously underspecified separator/empty-path behavior, and defines `OpenMode`
> semantics sufficiently for U-FS to implement host flags without inventing policy.
>
> **No native primitives are added.** `Path` and `OpenMode` remain pure `.ph` classes.

## 1. Scope

The shipped `Path` design is retained:

- byte-backed, not String-backed;
- defensive copy on `Path.ofBytes`;
- defensive copy from `Path#bytes`;
- cached content hash;
- immutable value semantics;
- `/` as the lexical separator;
- no implicit `String` coercion;
- no filesystem access from `Path`;
- no normalization of `.` or `..`;
- lossy UTF-8 only for display.

This patch changes only:

1. empty-path `join` behavior;
2. construction rules for newly-derived separator runs;
3. tests around roots, empty operands, and trailing separators;
4. `OpenMode` validation and OS-facing semantics.

## 2. The shipped `join` bug

The current implementation trims trailing `/` bytes from the receiver and then unconditionally
inserts one separator before appending the right-hand path:

```phalcom
let recv = _bytes
let recvLen = recv.size
while ((recvLen > 0) and (recv.at(recvLen - 1) == 47)) {
  recvLen = recvLen - 1
}
const trimmedRecv = recv.slice(0, recvLen)
const sep = Bytes.fromString("/")
const combined = trimmedRecv.concat(sep).concat(other.bytes)
return Path.ofBytes(combined)
```

For an empty receiver:

```phalcom
Path.of("").join(Path.of("a"))
```

this produces:

```text
/a
```

and therefore changes a relative path into an absolute one.

That behavior is incorrect and MUST be patched.

## 3. Normative `join` semantics

`Path#join(other)` remains a purely lexical operation.

The rules, in precedence order, are:

```text
1. other must be a Path; otherwise raise ArgumentError.
2. if other is absolute, return other.
3. if receiver is empty, return other.
4. if other is empty, return receiver.
5. otherwise join with exactly one separator between operands.
6. do not normalize "." or "..".
```

Examples:

```text
""      join "a"      => "a"
"a"     join ""       => "a"
""      join ""       => ""
"a"     join "b"      => "a/b"
"a/"    join "b"      => "a/b"
"a///"  join "b"      => "a/b"
"/"     join "a"      => "/a"
"//"    join "a"      => "/a"
"a"     join "/b"     => "/b"
"a/.."  join "b"      => "a/../b"
```

Returning an existing `Path` operand in rules 2-4 is allowed because `Path` is immutable.

## 4. Separator-run policy

### 4.1 Construction remains byte-exact

`Path.of(s)` and `Path.ofBytes(b)` MUST preserve the supplied byte sequence exactly except for
the already-required ownership copy.

Therefore:

```text
Path.of("//").bytes
```

still contains two `/` bytes.

The constructor is not a normalizer.

### 4.2 Derived operations may canonicalize join boundaries

When a lexical operation constructs a **new** path across a join boundary, runs of separators at
that boundary are canonicalized to one `/`.

Thus:

```text
"a///".join("b") => "a/b"
"//".join("b")   => "/b"
```

This is a rule about the derived result, not about mutation or normalization of the original
operands.

### 4.3 Root model

For Path's lexical API, any nonempty path whose first byte is `/` is absolute.

For operations that derive a root result, the canonical root spelling is:

```text
/
```

Phalcom deliberately does not expose POSIX's implementation-defined special handling of a
leading `//`.

This keeps the language deterministic and portable to future non-Unix backends.

## 5. `parent` semantics

Freeze the following behavior:

```text
Path.of("").parent          => None
Path.of("/").parent         => None
Path.of("//").parent        => None
Path.of("a").parent         => None
Path.of("a/b").parent       => Path.of("a")
Path.of("a/b/").parent      => Path.of("a")
Path.of("/a").parent        => Path.of("/")
Path.of("/a/b").parent      => Path.of("/a")
Path.of("a/../b").parent    => Path.of("a/..")
```

Trailing separators are ignored for the purpose of finding the parent.

The operation does not interpret `.` or `..`.

For all-separator absolute inputs, the root is canonicalized to `/` when returned as a newly
derived path.

## 6. `fileName` semantics

Freeze:

```text
Path.of("").fileName       => None
Path.of("/").fileName      => None
Path.of("//").fileName     => None
Path.of("a").fileName      => Path.of("a")
Path.of("a/b").fileName    => Path.of("b")
Path.of("/a/b").fileName   => Path.of("b")
Path.of("a/b/").fileName   => None
```

A path ending in `/` names a directory position rather than a final lexical filename and
therefore returns `None`.

No normalization of `.` or `..` occurs:

```text
Path.of("a/..").fileName => Path.of("..")
```

## 7. `extension` semantics

Retain the existing strict UTF-8 result rule and freeze the edge cases:

```text
"file.txt"     => "txt"
"file."        => None
".bashrc"      => None
".config.json" => "json"
"archive.tar"  => "tar"
"a/b.txt"      => "txt"
"a/b/"         => None
""             => None
```

Only the final filename component is inspected.

If the extension bytes are not valid UTF-8, return `None`.

No lossy decode is used for extension.

## 8. `components` semantics

`components` remains lexical.

Rules:

- separator runs split components;
- empty components are omitted;
- the leading root separator is not emitted as a component;
- `.` and `..` remain ordinary component byte sequences.

Examples:

```text
""            => []
"/"           => []
"//"          => []
"a/b"         => ["a", "b"]
"a//b///c"    => ["a", "b", "c"]
"/a/b"        => ["a", "b"]
"a/../b"      => ["a", "..", "b"]
```

Each component remains a `Path`.

## 9. Equality and hash

No behavioral change.

Path equality remains byte equality under the immutable value abstraction.

The cached hash remains sound because:

- input is defensively copied;
- output from `bytes` is defensively copied;
- no selector exposes the internal mutable `Bytes`;
- Path itself exposes no mutation operation.

The implementation MAY optimize equality later to compare internal bytes without allocating
`other.bytes`; that is not required by this patch and must not change semantics.

## 10. `OpenMode` is a closed semantic set

The shipped class currently uses an internal name constructor:

```phalcom
class OpenMode {
  @constructor
  named_(n) { _name = n }

  static read => OpenMode.named_("read")
  static write => OpenMode.named_("write")
  static append => OpenMode.named_("append")
  static readWrite => OpenMode.named_("readWrite")
}
```

The four public values remain exactly:

```text
read
write
append
readWrite
```

No numeric flag surface is introduced.

U-FS MUST treat any other internal `_name` as invalid and raise before job submission.

Prefer additionally validating `named_` itself so malformed values cannot be constructed even
through reflective/internal calls:

```text
name in {"read", "write", "append", "readWrite"}
```

If reflective APIs deliberately permit bypassing this internal constructor, the native FS
boundary still MUST validate independently.

## 11. Normative OS semantics for `OpenMode`

U-FS needs exact semantics; the names alone are insufficient.

The four values mean:

| Mode | File must exist | Create if missing | Truncate existing | Append writes | Read | Write |
|---|---:|---:|---:|---:|---:|---:|
| `read` | yes | no | no | no | yes | no |
| `write` | no | yes | yes | no | no | yes |
| `append` | no | yes | no | yes | no | yes |
| `readWrite` | yes | no | no | no | yes | yes |

### 11.1 `read`

Equivalent intent to conventional read-only open:

```text
existing file required
no creation
no truncation
```

### 11.2 `write`

Equivalent intent to conventional create/truncate write mode:

```text
create when missing
truncate when present
write-only
```

`File.create(path)` SHOULD be exactly equivalent to:

```phalcom
File.openWith(path, mode: OpenMode.write)
```

unless the governing filesystem surface explicitly says otherwise.

### 11.3 `append`

```text
create when missing
never truncate on open
each write is positioned according to host append semantics
write-only
```

U-FS SHOULD use the host's atomic append flag rather than implementing append as
`seek(end)` followed by `write`, because the latter races with other writers.

### 11.4 `readWrite`

`readWrite` is non-destructive:

```text
existing file required
no creation
no truncation
read + write
```

This is intentionally analogous to conventional `r+`, not `w+`.

If Phalcom later needs a create/truncate read-write mode, it must receive a distinct explicit
`OpenMode` value rather than overloading `readWrite`.

## 12. Filesystem-boundary validation

Before U-FS submits an open job, it MUST convert the `OpenMode` object into a closed Rust enum:

```rust
enum OpenModeName {
    Read,
    Write,
    Append,
    ReadWrite,
}
```

Do not send arbitrary user strings into worker code and switch there.

The VM/native boundary performs:

```text
OpenMode object
    -> validated name
    -> OpenModeName enum
    -> worker job
```

Unknown names are contract failures and raise synchronously before a `Future` is registered or a
worker job is queued.

## 13. File-by-file patch

### 13.1 `phalcom-core/core/core.ph`

Patch `Path#join` to handle empty operands before separator insertion.

Recommended shape:

```phalcom
join(other) {
  if (not other.is(Path)) {
    throw ArgumentError.new("Path#join: argument must be a Path")
  }

  if (other.isAbsolute) {
    return other
  }

  if (_bytes.size == 0) {
    return other
  }

  if (other.bytes.size == 0) {
    return self
  }

  // trim receiver trailing separators
  // preserve/canonicalize an absolute root
  // insert exactly one separator
  // append other bytes
}
```

Avoid repeatedly calling `other.bytes` if that accessor performs a defensive copy. Capture it
once:

```phalcom
const otherBytes = other.bytes
```

The implementation MUST special-case an absolute receiver whose trimmed non-separator length
becomes zero so that `/` is retained rather than producing a relative result.

Patch or confirm `parent`, `fileName`, `extension`, and `components` against §§5-8.

Validate `OpenMode.named_` if doing so is compatible with existing internal-constructor policy.

### 13.2 Tests

No Rust or primitive changes are required for this patch.

## 14. Required regression tests

Add positive golden rows for:

```text
Path.of("").join(Path.of("a"))      == Path.of("a")
Path.of("a").join(Path.of(""))      == Path.of("a")
Path.of("").join(Path.of(""))       == Path.of("")
Path.of("/").join(Path.of("a"))     == Path.of("/a")
Path.of("//").join(Path.of("a"))    == Path.of("/a")
Path.of("a/").join(Path.of("b"))    == Path.of("a/b")
Path.of("a///").join(Path.of("b"))  == Path.of("a/b")
Path.of("a").join(Path.of("/b"))    == Path.of("/b")
Path.of("a/..").join(Path.of("b"))  == Path.of("a/../b")
```

Add the parent/fileName/extension/components tables from §§5-8.

Retain the ownership tests proving that mutating:

```text
the Bytes passed to Path.ofBytes
the Bytes returned by Path#bytes
```

does not mutate the Path.

## 15. OpenMode tests

Required `.ph` tests:

```text
OpenMode.read == OpenMode.read
OpenMode.read != OpenMode.write
OpenMode.write.toString == "OpenMode.write"
```

If `named_` validation is surfaced to tests through internal access, reject:

```text
""
"Read"
"rw"
"truncate"
arbitrary non-String
```

U-FS later adds host integration tests for the actual flag behavior:

```text
read missing -> notFound
write missing -> creates
write existing -> truncates
append existing -> preserves then appends
append missing -> creates
readWrite missing -> notFound
readWrite existing -> preserves content
```

## 16. Compatibility

The patch intentionally preserves:

- `Path` construction bytes;
- equality/hash behavior;
- lossy display behavior;
- absence of filesystem queries on Path;
- absence of normalization;
- four existing OpenMode public spellings.

The only intentional behavior correction is the set of previously underspecified or erroneous
derived lexical edge cases, principally empty-receiver `join`.

## 17. Acceptance gates

The patch is complete when:

1. empty-path join never manufactures an absolute path;
2. join emits exactly one separator at a constructed boundary;
3. original Path objects remain byte-preserving;
4. root behavior is deterministic for separator runs;
5. parent/fileName/extension/components edge cases are frozen by tests;
6. OpenMode has a closed four-value semantic interpretation;
7. U-FS can map each mode to host flags without inventing create/truncate policy;
8. the existing Path golden suite remains green.
