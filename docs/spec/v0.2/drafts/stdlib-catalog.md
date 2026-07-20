# Standard library catalog — everything a modern language needs, and the order it must be built in

- Status: **Draft** (exploration only — not proposed, not ratified, no owning unit)
- Date: 2026-07-20
- Depends on:
  [ADR-0019](../../../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md) (the floor admission rule — **nothing in this document is a floor candidate**) ·
  [ADR-0020](../../../adr/accepted/0020-kernel-list-native-array-protocol.md) (native storage / `.ph` protocol — the pattern every container here follows) ·
  [ADR-0050](../../../adr/accepted/0050-non-moving-mark-sweep-collector.md) (mark-sweep, **no finalizers** — §1.1 is a direct consequence) ·
  [ADR-0024](../../../adr/accepted/0024-numeric-surface-split-int-float-and-division.md) (Accepted, **not built** — §0.1 blocks on it) ·
  [ADR-0045](../../../adr/accepted/0045-module-import-relative-path-whole-module-binding.md) (imports resolve to relative source only — Tier 6 blocks on it) ·
  [ADR-0012](../../../adr/accepted/0012-selector-signature-encoding-and-dispatch.md) (selector encoding — every signature below is written in it) ·
  [ADR-0043](../../../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md) (no default arguments — why no signature below has one) ·
  [ADR-0021](../../../adr/accepted/0021-no-truthiness-enforcement.md) (predicates return real `Bool`) ·
  [ADR-0048](../../../adr/accepted/0048-amend-iteration-bare-cursor-sentinel-and-iterable-root.md) (`Iterable` root, bare-cursor protocol)
- Related — **this document deliberately does not repeat them**:
  [drafts/ffi.md](ffi.md) (**the delivery vehicle for every Tier-3 entry**; §2 census math, §6 the three tiers, §7 precedent) ·
  [drafts/bytes.md](bytes.md) (**§1.2's full argument and memory math**) ·
  [drafts/crypto.md](crypto.md) (**§2.7's "bind, do not build" rule**) ·
  [drafts/native-math.md](native-math.md) (§2.1) ·
  [drafts/sealed-classes.md](sealed-classes.md) (§0.3) ·
  [concurrency.md](../concurrency.md) + [system.md](../system.md) (Tier 4's existing surface)

> **How to use this document.** An exploration doc, per
> [drafts/README.md](README.md). It is a **catalog and a build order**, not a plan and not
> a commitment — no unit owns any entry here. Its purpose is that the next time someone
> asks "what does Phalcom still need?", the enumeration and its dependency structure are
> one link away instead of re-derived. Sections are append-friendly; new uncertainties
> become a new `S-n` row in *Open questions* and are never renumbered. Tree claims carry `file:line`;
> committed positions cite an ADR §; anything unchecked is marked **unverified**.
>
> **Signatures below are illustrative, not designed.** They exist to make the *shape* and
> the *dependency* concrete — to show that `File#read` needs `Bytes` which needs `Int`.
> Each one would be designed properly by its owning unit. Do not treat a selector spelling
> here as settled; treat the dependency it exposes as the finding.

---

## Thesis

Phalcom's landed surface is a **complete object model with almost no standard library**.
That is the correct order to have built things in, and it is why the remaining work is
unusually tractable: the hard parts (dispatch, metaclasses, closures, fibers, a precise
collector) are done, and what is left is mostly ordinary library work over a substrate
that does not exist yet.

The **substrate** is the finding. This document's value is not the list — anyone can list
`File`, `Regex`, `DateTime`. Its value is the **build order**, and the observation that
**ten items block everything else**, of which three are language mechanisms rather than
libraries, and one is an already-confirmed soundness bug.

Three claims organize it:

1. **Nothing here belongs on the ADR-0019 floor.** The floor is what the object model
   *presupposes*. Filesystems presuppose nothing. ffi.md §2 does this argument in full
   with census math; it is not repeated here. Every Tier-3 entry arrives as a **native
   module** in ffi.md's tier (a), trailing-`_` natives with `.ph` surface above.
2. **The dependency graph is nearly linear, and it is not the graph people expect.**
   The intuitive order is "files first, they're the most useful". The actual order is
   `Int` → `temp_roots` → resource lifetime → `Bytes` → `Path` → stream protocol → `File`.
   Building `File` first means building it twice.
3. **Two forks must be ruled before any Tier-3 signature can be written**, because they
   change the signatures rather than the implementations: **finalizer/resource syntax**
   (§1.1) and **blocking-vs-reactor IO** (§4.1). A third — **user-visible threads**
   (§4.5) — decides §4.1's implementation. These are S-1, S-2, S-3.

### Landed surface, so the gaps are honest

Classes defined in `phalcom-core/core/core.ph` (1713 lines, read 2026-07-20):
`Object:1`, `Class:36`, `Metaclass:38`, `Error:54` (+ `PreconditionError:69`,
`PostconditionError:71`, `InvariantError:73`, `ArgumentError:80`), `Number:82`,
`String:84`, `StringByteSequence:363`, `StringCodePointSequence:388`, `Bool:421`
(+ `True:441`, `False:443`), `Symbol:445`, `Option:474` (+ `Some:544`), `Result:557`
(+ `Ok:606`, `Err:612`), `Function:623`, `Iterable:646`, `List:779`, `Map:866`,
`Set:975`, `Tuple:1043`, `Range:1117`, `MapView:1205`, `WhereView:1219`,
`SkipView:1240`, `TakeView:1261`, `System:1284`, `Future:1346`, `Tracer:1556`,
`OffBehavior:1571`, `Backoff:1592`, `Attribute:1631`, `On:1640`, `Tier:1657`
(+ `Compile`/`Layout`/`Install`/`Dispatch`/`Runtime`), `Behavior:1669`, `Method:1677`.

Native heap arms (`phalcom-core/src/heap/object.rs:26-114`): `Instance`, `Class`,
`Method`, `Module`, `Closure`, `Str`, `Block`, `BoundMethod`, `Upvalue`, `List`,
`Fiber`, `Map`, `Set`, `Tuple`, `Range`, `Family`.

**Everything named in Tier 0 – Tier 6 below is absent from both lists unless explicitly
marked otherwise.**

---

## Tier 0 — language mechanisms

These are not libraries. They are the reason the libraries cannot be written.

### 0.1 `Int` / `Float` split — **ADR-0024 Accepted, not built**

`Number` is a single `f64` arm. ADR-0024 ratified the split; `docs/adr/accepted/0024-…md:3`
reads `Status: Accepted`, and the overlay's standing warning applies — Accepted is not
built.

Every one of the following has an `Int` in its signature and is undesignable without it:
byte values (`0..255`), file offsets and lengths, file permission bits, epoch timestamps
and nanosecond fields, array and buffer indices, exit statuses, `errno`/`ErrorKind`
discriminants, port numbers, hash codes, and every bit-flag set in §3.3/§4.3.

**This is the single highest-leverage unbuilt item in the tree** for standard-library
purposes, and it is already ratified — no new decision is needed, only the unit.

### 0.2 Bitwise operations on `Int`

```
Int#and(_)  or(_)  xor(_)  not  shl(_)  shr(_)  ushr(_)
Int#bitAt(_) -> Bool   bitCount   leadingZeros   trailingZeros
```

Blocked on 0.1. Required by: open-flags, permission modes, `Bytes` fixed-width codecs
(§1.2), hex/base64 (§1.5), every hash function (§2.7), socket options (§4.3).

**Precedent with its cost.** JavaScript's bitwise operators coerce to int32 *inside* a
float64 number type; the result is that `>>> 0` is a widely-taught idiom and asm.js/wasm
had to specify integer semantics separately. **Cost:** a permanent performance and
correctness seam between "numbers" and "the integers you meant". Phalcom has ADR-0024 and
should not repeat it.

### 0.3 Sum types / exhaustive variants — **half built**

`@sealed`/`@variant` **exist in the compiler**: `VM::sealed_classes` is a compile-unit
sealed-class table, and `phalcom-core/src/compiler/attributes.rs:62-68` names it as the
"**second** source of 'is this class sealed?'", noting that "neither is complete on its
own (DEFERRED CB-3 / `drafts/sealed-classes.md`)". The missing half is the `match`
exhaustiveness checker (drafts/README.md's own note on `sealed-classes.md`).

Every entry below wants a closed variant set: `SeekFrom`, `OpenMode`, `FileType`,
`ErrorKind`, `IpAddr`, `SignalKind`, `Ordering`, `Endian`, `RoundingMode`. Without
exhaustiveness these become class families plus `is` chains, and adding a variant later is
a silent behavior change at every use site rather than a compile error.

`Option`/`Result` are today hand-rolled (`core.ph:474`, `:557`) precisely because this
was not available.

### 0.4 Resource lifetime — the finalizer question

Its own section: §1.1. Named here because it is a **language** decision, not a library
one, and because it wants syntax.

### 0.5 Weak references — **absent, and the collector has no weak worklist**

Grep for `weak` in `phalcom-core/src/vm/gc.rs`: **zero hits** (2026-07-20).

```
WeakRef.new(_)          WeakRef#get -> Option    WeakRef#isAlive -> Bool
WeakMap.new             WeakMap#at(_) -> Option   at(_, put:)   remove(_)   size
```

Precludes-if-absent: any cache that can shrink, observer/listener registries that do not
leak their subjects, identity side-tables keyed by object (the exact shape
`reactivity.md`'s dependency tracking wants), and interning that can release.

**Precedent with its cost.** Java added `WeakReference` plus a `ReferenceQueue` and four
strength levels (`Soft`/`Weak`/`Phantom`/`Final`); **cost:** the strength lattice is
widely misused, `SoftReference` behavior is JVM-tuning-dependent and effectively
unspecified, and `finalize()` — the fourth level — was deprecated for removal.
The lesson for Phalcom is to ship **one** strength (`Weak`) and no reference queue until
something concrete needs one.

**What it costs the collector.** A weak arm means the mark phase gains a second pass:
mark strongly, then clear every weak slot whose referent was not marked, before sweep.
That is a real change to `vm/gc.rs` and to ADR-0050's stated algorithm — this is
**unverified** as to difficulty; nobody has scoped it.

### 0.6 `Comparable` / `Hashable` / ordering — **absent**

Grep `core.ph` for `sort`, `compare`, `hashCode` (2026-07-20): the only hit is a comment
at `:1091`. `Map`/`Set` hash natively (`Object::Map`/`Set`, `heap/object.rs:79`/`:88`),
but **no user-facing hashing or ordering contract exists**, so a user class cannot be a
`Map` key with meaningful equality, and nothing can be sorted.

```
Ordering                                  // Less | Equal | Greater  (needs 0.3)
Comparable#compare(_) -> Ordering
Hashable#hashCode -> Int                  // needs 0.1
Iterable#sorted   sortedBy(_)   min   max   minBy(_)   maxBy(_)
List#sort   sortBy(_)                     // in place
```

**Precedent with its cost.** Python defines `__eq__`/`__hash__` with an explicit law
(equal objects hash equal) and makes defining `__eq__` alone silently set `__hash__ = None`
— a runtime `TypeError` at first dict insertion. **Cost:** the failure is late and
confusing, but the law is at least *stated*. Phalcom currently states nothing, which is
strictly worse: the law will be discovered by whoever first puts a user object in a `Map`.

### 0.7 Copying and the equality law

```
Object#copy -> Object          // shallow
Object#deepCopy -> Object
```

Plus a written contract for `==`/`hashCode` on user classes. Neither exists. Cheap, and
cheapest before Tier 2's containers proliferate.

---

## Tier 1 — substrate

### 1.1 Resource protocol and finalization — **the ruling that unblocks Tier 3**

**The constraint.** ADR-0050 §Context banks "No finalizers exist" as a reason the
collector is hazard-free (no ordering, no resurrection). Native resources — file
descriptors, sockets, child processes, mapped memory — are the classic reason a language
grows finalizers. This is ffi.md's **F-3**, restated here because every Tier-3 entry
depends on the answer.

**The recommendation (this doc's opinion, not a ruling): explicit scope plus leak
detection. Do not admit finalizers.**

```
Disposable#dispose -> None            // idempotent, required, returns None
Disposable#isDisposed -> Bool
```

Proposed syntax — a scoped-ownership form that desugars to `ensure`:

```phalcom
using f = File.open("a.txt") {
    f.readAll
}
// desugars to:  let f = File.open("a.txt); ensure { f.dispose } { f.readAll }
```

Multiple resources, disposed in reverse acquisition order:

```phalcom
using inFile = File.open(a), outFile = File.create(b) {
    outFile.writeAll(inFile.readAll)
}
```

**Backing mechanism** (VM-internal, not user-visible): a resource table of
generation-tagged slots — **structurally the same mechanism as frame tokens**
([ADR-0013](../../../adr/accepted/0013-closure-upvalues-and-frame-token-return.md)), which
Phalcom has already built and proven for a very similar problem (a handle that must
detect "the thing I refer to is gone" without holding it alive). A handle collected
without `dispose` is then a **reported leak, never a use-after-free**:

```
System.leakReport -> List             // undisposed resources still open at exit
System.strictResources(_)             // Bool: raise on leak instead of warning
```

**Precedent, each with its cost.**

- **Java `finalize()`** — ran on an unspecified thread at an unspecified time, could
  resurrect objects, and could not be relied on to run at all. **Cost:** deprecated for
  removal in Java 9/18; the ecosystem needed `try-with-resources` + `AutoCloseable`
  *anyway*, so the finalizer was pure hazard with no payoff.
- **Python `__del__`** — refcounting makes it usually-prompt, which is worse than never:
  code depends on promptness that cycles and alternative implementations (PyPy) do not
  provide. **Cost:** `__del__` participating in a reference cycle was uncollectable until
  3.4; PyPy's non-refcounting GC breaks programs that silently relied on CPython timing.
- **C# `IDisposable` + `using`** — explicit scope as the primary mechanism, finalizers
  demoted to a safety net behind `SafeHandle`. **Cost:** the "dispose pattern" boilerplate
  is notorious, and forgetting `using` is still just a leak. But it is the design every
  one of the above converged on after trying the other thing first.
- **Rust `Drop`** — deterministic, compiler-enforced by ownership, no runtime involvement
  at all. **Cost:** requires an ownership/borrow system to know *when*; Phalcom's object
  graph is a GC graph, so this is not available.
- **Zig `defer`** — purely lexical, no type-system involvement. **Cost:** nothing prevents
  forgetting it; it is a convention with syntax support. This is closest to the proposal
  above, and Zig's experience is that lexical scoping catches the overwhelming majority
  of cases in practice.

**What `using` precludes.** A resource whose lifetime is genuinely dynamic — an open file
stored in an object field and closed by a later, unrelated event (a connection pool, a
long-lived log handle, an editor's open buffers). The syntax handles the 90% case; the
escape hatch must be a bare `File.open` returning a `Disposable` the user closes manually,
or the design is too restrictive to be usable. **Both must exist.** A `using`-only design
is the mistake to avoid.

**What it depends on.** `ensure`-scoped ownership re-enters the interpreter while a native
handle is live — which is exactly ffi.md's **F-2**: `push_temp_root`/`pop_temp_root` is
specified by ADR-0050 §Decision 7, and **is not built**. `phalcom-core/src/vm/gc.rs`
carries the note that the escape hatch "that makes the native side safe [is] U-GC step 4;
until then this is driven only by tests", and `push_root_for_test` is documented as
scaffolding, explicitly "Not a temp-root". There is no `push_temp_root` definition in
`phalcom-core/src`.

This is not theoretical. A dangling-`ObjRef` crash from exactly this absence is already
confirmed in `block_ensure` (`docs/forge/perf-log/` GC record, 2026-07-19 commits
`293e923`/`bba96ee`). **This section's recommended design runs directly through the code path
that is already known to be unsound.** Fixing `temp_roots` is therefore a prerequisite,
not an adjacent nicety.

### 1.2 `Bytes` — mutable octet buffer

New heap arm `Object::Bytes`. **The full argument, memory math, and zeroization analysis
are in [bytes.md](bytes.md) and are not repeated here.** Catalogued for completeness of
the dependency graph: `Bytes` is the substrate for §1.5 encoding, all of Tier 3 IO, §4.3
sockets, and §2.7 crypto.

```
Bytes.new                     Bytes.withCapacity(_)      Bytes.zeroed(_)
Bytes.fromList(_)             Bytes.fromString(_)        Bytes.fromHex(_) -> Result
Bytes#size   isEmpty
Bytes#at(_) -> Int            at(_, put:)                // 0..255; raises out of range
Bytes#append(_)   appendBytes(_)   appendString(_)
Bytes#slice(_, _)   copyFrom(_, at:)   fill(_)   resize(_)   clear
Bytes#indexOf(_) -> Option    startsWith(_)   endsWith(_)
Bytes#toString -> Result                                 // UTF-8 validated
Bytes#toStringLossy -> String
Bytes#toHex   toBase64
Bytes#readInt(_, at:, endian:)    writeInt(_, at:, endian:)
Bytes#equalsConstantTime(_) -> Bool                      // cannot be .ph; crypto.md
Bytes#zeroize                                            // see bytes.md §7
```

Extends `Iterable` via the bare-cursor protocol (ADR-0048). Storage native, protocol `.ph`
— the ADR-0020 pattern, exactly as `List`.

### 1.3 `StringBuilder`

`Object::Str` is immutable and interned-by-content (`heap/string.rs`, cited via ffi.md §3).
Naive `s = s + x` in a loop is therefore both O(n²) **and** an interner-pollution problem:
every intermediate lands in the intern table.

```
StringBuilder.new    StringBuilder.withCapacity(_)
#append(_)   appendChar(_)   appendLine(_)   newline
#size   isEmpty   clear
#toString -> String                      // single intern, at the end
```

### 1.4 `Char` / code point

`StringCodePointSequence` exists (`core.ph:388`); its **element type does not** — iteration
currently yields what, exactly, is a question the sequence's own definition answers and
this doc has **not verified**.

```
Char.fromCode(_) -> Option        Char#code -> Int
Char#isDigit   isAlpha   isAlnum   isWhitespace   isUpper   isLower
Char#toUpper   toLower            Char#utf8Length -> Int
```

**Precedent with its cost.** Java's `char` is a UTF-16 code unit, not a code point, so
every astral-plane character is two `char`s and `String#length` lies about emoji.
**Cost:** unfixable without breaking the world; `codePointAt` was bolted on and the wrong
API remains the ergonomic one. Phalcom's `String` is UTF-8-backed and already has *two*
sequence views — the right structure. Make `Char` a **code point**, never a byte, never a
UTF-16 unit.

### 1.5 Encoding

```
Utf8.encode(_) -> Bytes             Utf8.decode(_) -> Result
Utf16.encode(_, endian:)            Utf16.decode(_, endian:) -> Result
Latin1.encode(_)                    Latin1.decode(_)
Hex.encode(_) -> String             Hex.decode(_) -> Result
Base64.encode(_)                    Base64.decode(_) -> Result
Base64Url.encode(_)                 Base64Url.decode(_) -> Result
```

All derivable in `.ph` over `Bytes` + `Int` once §0.1/§0.2/§3.2 exist — none is a floor
or FFI candidate. Note this is the honest shape of ffi.md §2's argument: the *encoding*
half of "crypto" is a library; only entropy is platform.

---

## Tier 2 — pure computation

No platform access. Every entry here is `.ph`-derivable over `Bytes`/`Int`, so none is a
floor candidate under ADR-0019 — some are FFI candidates purely for *implementation
quality* (§2.3, §2.7), which is a different argument and must be made as one.

### 2.1 `Math`

See [native-math.md](native-math.md) for the `f32`/`u32`/dtype question, not repeated here.

```
Math.pi   e   tau   inf   nan
Math.abs(_)  sign(_)  floor(_)  ceil(_)  round(_)  trunc(_)
Math.sqrt(_)  cbrt(_)  pow(_, _)  exp(_)  ln(_)  log(_, base:)  log2(_)  log10(_)
Math.sin(_) cos(_) tan(_) asin(_) acos(_) atan(_) atan2(_, _) sinh(_) cosh(_) tanh(_)
Math.min(_, _)  max(_, _)  clamp(_, min:, max:)  hypot(_, _)
Math.isNaN(_)  isInfinite(_)  isFinite(_)
```

### 2.2 `Random`

```
Random.new(seed: _)                  // deterministic, reproducible PRNG
Random.system                        // OS CSPRNG; not seedable, not reproducible
#nextInt   nextIntBelow(_)   nextIntIn(_, _)
#nextFloat   nextBool   nextBytes(_) -> Bytes
#pick(_) -> Option    shuffle(_)
```

The split is the design: a seedable PRNG is pure computation and belongs in `.ph`;
`Random.system`'s entropy is genuinely underivable and is roughly **one** native binding.
ffi.md §2 makes exactly this point — the honest floor-shaped subset of "crypto" is
entropy plus constant-time compare, and everything else is library.

### 2.3 `Regex`

```
Regex.compile(_) -> Result          Regex.compile(_, flags:) -> Result
#matches(_) -> Bool
#find(_) -> Option                  // Match
#findAll(_) -> Iterable
#replace(_, with:)    replaceAll(_, with:)
#split(_) -> List
Match#text   start   end   groups -> List   group(_) -> Option   named(_) -> Option
```

**Bind a linear-time engine; do not write a backtracker.** This is crypto.md's rule
generalized. **Precedent with its cost:** Perl-derived backtracking engines (PCRE, and
therefore Python `re`, Java, JavaScript, Ruby) admit catastrophic backtracking — the
Cloudflare 2019 global outage was one regex. **Cost:** every one of those ecosystems now
carries ReDoS as a permanent, un-removable class of vulnerability in user code, mitigated
only by linting. Rust's `regex` crate is linear-time by construction and closes the hole
before it opens, at the price of no backreferences and no lookaround.

That trade — **no backreferences, no lookaround, in exchange for no ReDoS** — is a real
design decision with a real cost, and it should be ruled deliberately rather than
inherited from whichever crate gets picked (**S-6**).

### 2.4 `Json`

```
Json.parse(_) -> Result             // -> Map / List / String / Number / Bool / None
Json.stringify(_) -> Result
Json.stringify(_, indent:) -> Result
JsonError#line   column   message
```

Open: parse-to-dynamic (`Map`) versus parse-to-typed. Dynamic first; typed needs §0.3 and
§5.6's `Decodable` protocol.

### 2.5 `Decimal` / `BigInt`

```
BigInt.fromString(_) -> Result      BigInt.fromInt(_)
#add(_) sub(_) mul(_) divMod(_) pow(_) modPow(_, mod:) gcd(_)   #toString(radix:)
Decimal.fromString(_) -> Result
#add(_) sub(_) mul(_)  div(_, scale:, rounding:)   #round(_, mode:)
```

Lower priority than the rest of Tier 2 — but note **what deferring it costs**: every
language that shipped float-only money grew a `Decimal` later *and* a permanent population
of programs using floats for currency. JavaScript is the extreme case (`0.1 + 0.2`, no
integer type until `BigInt` in ES2020, and `Number` still the default for money in most
code). The cost of deferring is not "we add it later"; it is "we add it later *and* the
wrong thing is already idiomatic".

### 2.6 `Uuid`

```
Uuid.v4   Uuid.v7   Uuid.parse(_) -> Result   #toString   #bytes -> Bytes
```

### 2.7 `Crypto`

**[crypto.md](crypto.md) §1 governs: bind audited crates, never implement — not in `.ph`
and not in Rust.** Catalogued here only for the dependency graph; the argument is there.

```
Crypto.sha256(_) -> Bytes    Crypto.sha512(_)   Crypto.blake3(_)
Digest.sha256                #update(_)  #finish -> Bytes            // streaming
Hmac.sha256(key: _)          #update(_)  #finish -> Bytes
Aead.chacha20Poly1305(key: _)
  #seal(_, nonce:, aad:) -> Bytes    #open(_, nonce:, aad:) -> Result
Kdf.hkdf(_, salt:, info:, length:)   Kdf.argon2id(_, salt:, params:)
Ed25519.generate             #sign(_) -> Bytes
Ed25519PublicKey#verify(_, signature:) -> Bool
```

---

## Tier 3 — platform

The genuinely underivable set: not computation at all, but access. This is the category
ffi.md §1 identifies as the FFI door's actual purpose.

### 3.1 `Path` — **must not be `String`**

`Object::Str` wraps a Rust `String`, which is UTF-8-enforced by the type. **POSIX paths
are arbitrary bytes; Windows paths are UTF-16.** A `Path` that is a `String` cannot
represent a file that exists on the user's disk.

```
Path.new(_) -> Path                 Path.fromBytes(_) -> Path
Path.separator   Path.cwd   Path.home   Path.temp
#toString -> Option                 // None when not valid UTF-8 — the honest signature
#toStringLossy -> String            #bytes -> Bytes
#join(_)   parent -> Option   fileName -> Option   stem -> Option   extension -> Option
#isAbsolute   isRelative   normalize   withExtension(_)   relativeTo(_) -> Option
#components -> Iterable
```

**Precedent with its cost.** Python 2 used `str` for paths and Python 3 initially used
`str`-only, then had to invent `surrogateescape` to round-trip undecodable POSIX filenames
through `str`. **Cost:** a bespoke, non-standard codec permanently embedded in the
language's filesystem layer, plus `os.fsencode`/`os.fsdecode`, plus `pathlib` arriving in
3.4 as a third path representation. Rust paid the cost up front with `OsStr`/`Path` and is
widely disliked for the ergonomics — but is correct. Go uses `string` (which is arbitrary
bytes, not enforced UTF-8) and gets correctness for free, which Phalcom cannot copy
because its `String` *is* enforced.

**This is a real fork (S-4):** `Path` as an opaque `Bytes`-backed class (correct, more
work, `toString` returns `Option`) versus `String`-only with lossy conversion (ships
faster, permanently wrong).

### 3.2 Stream protocol — **design before `File`, not after**

The abstraction that `File`, sockets, stdio, and in-memory buffers all implement. Building
`File` with its own bespoke read API and retrofitting a protocol later means four
incompatible read APIs — the single most common standard-library mistake in this list.

```
Reader#read(_) -> Result            // fill a Bytes; returns count read; 0 == EOF
Reader#readAll -> Result            readExact(_) -> Result
Reader#readLine -> Result           // Result<Option<String>>; None at EOF
Reader#lines -> Iterable            bytes -> Iterable
Writer#write(_) -> Result           // returns count written
Writer#writeAll(_) -> Result        writeString(_) -> Result
Writer#flush -> Result
Seekable#seek(_) -> Result          // SeekFrom.start(_) | .current(_) | .end(_)
Seekable#position -> Result         length -> Result
BufferedReader.new(_)   BufferedWriter.new(_)
BytesReader.new(_)      BytesWriter.new              // in-memory, for tests
```

Every one of these is `Disposable` (§1.1). `Reader`/`Writer` are the reason §1.1's ruling
gates Tier 3 as a whole rather than `File` specifically.

### 3.3 `File` / `Fs`

```
File.open(_) -> Result                              // read-only
File.create(_) -> Result                            // write, truncate
File.openWith(_, mode:) -> Result                   // OpenMode; needs §0.3
File.readAllBytes(_) -> Result                      // one-shot conveniences
File.readAllString(_) -> Result
File.writeBytes(_, to:) -> Result
File.writeString(_, to:) -> Result
File#path   metadata -> Result   setLength(_) -> Result   sync -> Result   dispose
// File is Reader + Writer + Seekable + Disposable

Fs.exists(_) -> Bool          Fs.isFile(_)   Fs.isDir(_)   Fs.isSymlink(_)
Fs.metadata(_) -> Result      Fs.symlinkMetadata(_) -> Result
Fs.copy(_, to:) -> Result     Fs.rename(_, to:) -> Result
Fs.remove(_) -> Result        Fs.removeDir(_, recursive:) -> Result
Fs.createDir(_) -> Result     Fs.createDirAll(_) -> Result
Fs.readDir(_) -> Result       // Iterable<DirEntry>
Fs.walk(_) -> Result          // recursive Iterable<DirEntry>
Fs.canonicalize(_) -> Result  Fs.symlink(_, to:) -> Result   Fs.readLink(_) -> Result
Fs.setPermissions(_, to:) -> Result
Fs.tempDir                    Fs.tempFile -> Result          Fs.tempDirIn(_) -> Result

Metadata#size   isFile   isDir   isSymlink
Metadata#modified -> Option   accessed -> Option   created -> Option
Metadata#permissions -> Permissions    #isReadonly -> Bool
DirEntry#path   fileName   fileType   metadata -> Result
Permissions.fromMode(_)       #mode -> Int   isReadonly   setReadonly(_)
```

`Fs.exists(_)` returning bare `Bool` rather than `Result` is deliberate and is the
conventional API, but note it is a **TOCTOU invitation** — the check-then-open race. The
`Result`-returning `File.open` is always the correct form; `exists` is a convenience whose
misuse should at least be documented at its definition.

### 3.4 Stdio

```
Stdio.out -> Writer      Stdio.err -> Writer      Stdio.in -> Reader
Stdio.out.isTerminal -> Bool
Terminal.size -> Option              Terminal.isatty(_) -> Bool
```

`System.print(_)`/`write(_)`/`printErr(_)`/`readLine` are **landed** (system.md §2;
`primitive/system.rs`, via ffi.md §5.1's precedent table). Under a stream protocol they
become sugar over `Stdio.out`. **Decide whether they forward or get deprecated (S-5)** —
Phalcom has no deprecation mechanism, so "both forever" is the default outcome unless
ruled otherwise.

### 3.5 `Env`

```
Env.get(_) -> Option        Env.set(_, to:)      Env.remove(_)
Env.all -> Map              Env.args -> List
Env.executablePath -> Result
Env.cwd -> Result           Env.setCwd(_) -> Result
```

`System.args` and `System.env(_)` are in system.md §2's table. Same S-5 question.

### 3.6 `Process`

```
Command.new(_)                                      // program name
#arg(_)   args(_)   env(_, to:)   clearEnv   cwd(_)
#stdin(_)  stdout(_)  stderr(_)                     // Stdio.inherit | .piped | .null
#run -> Result                                      // wait; returns Output
#spawn -> Result                                    // returns Child; does not wait
Output#status -> Int   stdout -> Bytes   stderr -> Bytes   succeeded -> Bool
Child#pid   wait -> Result   tryWait -> Result      // Result<Option>
Child#kill -> Result   dispose
Child#stdin -> Option    stdout -> Option    stderr -> Option
Process.pid    Process.exit(_)    Process.abort
Signal.on(_, handler:)                              // SigInt | SigTerm | SigHup
```

**Security note, stated once and load-bearing.** `Command.new(_)` takes a program and
**argument vector**, never a shell string. There is deliberately no `Command.shell(_)` in
this catalog. **Precedent with its cost:** every language that shipped a
string-to-`/bin/sh` convenience (`system()` in C/PHP/Perl, Python's `subprocess(shell=True)`,
Node's `child_process.exec`) made shell injection the default-easy path. **Cost:** a
permanent CWE-78 population in each ecosystem, and documentation that has to spend its
first paragraph telling people not to use the obvious function. If a shell escape is ever
wanted it should be named to look dangerous, and that is a ruling, not a default.

`Signal.on(_, handler:)` interacts with the scheduler: a signal arriving mid-fiber must be
queued to the ready-queue, not delivered re-entrantly. **Unverified** whether the current
ready-queue (`VM::ready_queue`, system.md §2) can accept a push from a signal context at
all — likely not, and this may be the first thing that forces §4.5's thread question.

### 3.7 `Os`

```
Os.name -> String            // "macos" | "linux" | "windows"
Os.family   Os.arch   Os.version   Os.hostname -> Result
Os.cpuCount   Os.pageSize   Os.uptime   Os.loadAverage -> Tuple
Os.userName -> Option        Os.homeDir -> Option
```

### 3.8 Time

```
Instant.now -> Instant                    // monotonic; durations only, never dates
Instant#elapsed -> Duration   since(_) -> Duration   plus(_)   minus(_)
Duration.seconds(_) millis(_) micros(_) nanos(_) minutes(_) hours(_)
Duration#asSeconds   asMillis   asNanos   plus(_)   minus(_)   times(_)   toString
DateTime.now -> DateTime                  // wall clock, UTC
DateTime.fromEpochSeconds(_)              DateTime.parseIso8601(_) -> Result
DateTime#year month day hour minute second nanosecond
DateTime#dayOfWeek   dayOfYear   epochSeconds   toIso8601
DateTime#plus(_)   minus(_)   inZone(_) -> DateTime
TimeZone.utc   TimeZone.local   TimeZone.named(_) -> Result
Date   Time   Weekday   Month             // calendar-only, zone-free
```

**The monotonic/wall-clock split is the whole design.** `System.clock` (monotonic) and
`System.now` (wall-clock epoch) are landed and **both return bare `Float`** (system.md §2).
That is exactly the shape that lets someone measure a duration with wall-clock time and
get a negative result when NTP steps the clock backwards.

**Precedent with its cost.** JavaScript's `Date.now()` is wall-clock and was the only
timer for two decades; `performance.now()` arrived later, so most existing timing code is
still wrong across clock adjustments. Java's `System.currentTimeMillis()` versus
`nanoTime()` is the same split, and the same population of code uses the wrong one.
**Cost in both cases:** unfixable, because the wrong function has the better name. Phalcom
can still make `Instant`/`DateTime` distinct *types* so the mistake does not typecheck —
but only if it does so before code accumulates on the two `Float`s.

---

## Tier 4 — concurrency and networking

### 4.1 Timers and the reactor — **absent; and the fork that gates Tier 3's signatures**

What is landed: `System.schedule(_)`, `System.nextScheduled`, `System.runScheduled`, plus
`VM::run`'s root-drive pump (system.md §2, §"Scheduler"). That is a **ready-queue and
nothing else**. `System.sleep(_)` is explicitly documented as **still open** —
`docs/spec/v0.2/system.md:75`: "U-SCHED deliberately splits the ready-queue from timers
(`open-questions.md` §15 fairness is unresolved for a timer completion source); tracked as
a follow-on unit, not built here."

So there is no timer, no completion source, no poll integration, and therefore **no way
for a fiber to wait on anything external**. `Future.async` gives concurrency of
computation, not of IO.

```
Timer.after(_) -> Future        Timer.at(_) -> Future
Timer.every(_, do:) -> Cancellable
System.sleep(_) -> Future
```

**The fork (S-2), which must be ruled before any Tier-3 signature is written:**

- **(a) Blocking IO, honestly labeled.** `File.read` blocks the whole VM. Entirely
  defensible for a scripting/CLI language. Zero new machinery. **Cost:** one fiber
  blocking on a socket stalls every other fiber, so `Fiber` remains a coroutine story and
  never becomes an async-IO story.
- **(b) Thread pool plus completion queue.** Blocking syscalls on worker threads; results
  settle a `Future` on the VM thread. **Cost:** needs a cross-thread wake into
  `VM::ready_queue` and confronts open-questions.md §15's unresolved fairness question.
- **(c) True reactor** (epoll/kqueue/IOCP). **Cost:** the largest, and it needs timers
  first anyway.

**Why this cannot be deferred.** It does not change *implementations*, it changes
*signatures* — whether `File#read` returns `Result` or `Future<Result>`, whether
`TcpStream.connect` is a `Future` at all. Shipping (a)'s signatures and later wanting (c)
means breaking every IO selector in the language. Pick the **signature** now even if only
(a) is built.

### 4.2 Channels and structured concurrency

```
Channel.new       Channel.bounded(_)
#send(_) -> Future    #receive -> Future    #close    #isClosed -> Bool
#trySend(_) -> Bool   #tryReceive -> Option
TaskGroup.new     #spawn(_)   #wait -> Future   #cancel
CancelToken.new   #cancel   #isCancelled -> Bool   #onCancel(_)
Future.all(_)   Future.any(_)   Future.race(_)    Future#timeout(_)
```

concurrency.md:274 already parks "structured concurrency / cancellation scopes, whether
`Future` gets `select`/`race`" as open. Cancellation in particular is a design that cannot
be added late: it must be threaded through every `Future`-returning signature in Tier 3 and
§4.3, which is the same argument as S-2.

### 4.3 Net

```
IpAddr.v4(_, _, _, _)   IpAddr.v6(...)   IpAddr.parse(_) -> Result
IpAddr#isLoopback   isPrivate   isMulticast
SocketAddr.new(_, port: _)      #ip   port   toString
Dns.resolve(_) -> Future                        // Future<List<IpAddr>>
TcpListener.bind(_) -> Result   #accept -> Future   localAddr   incoming -> Iterable
TcpStream.connect(_) -> Future  #peerAddr   setNoDelay(_)   shutdown(_)   dispose
// TcpStream is Reader + Writer + Disposable
UdpSocket.bind(_) -> Result     #sendTo(_, addr:) -> Future   receiveFrom(_) -> Future
UnixSocket.connect(_)  UnixSocket.bind(_)       // POSIX only
Tls.connect(_, host:) -> Future
Tls.acceptor(cert:, key:) -> Result
Url.parse(_) -> Result          #scheme host port path query fragment   #join(_)
```

TLS is a crypto.md §1 case: bind, never build.

### 4.4 HTTP

```
Http.get(_) -> Future            Http.post(_, body:) -> Future
Request.new(_, url: _)   #header(_, to:)   body(_)   timeout(_)   send -> Future
Response#status   headers -> Map   body -> Future   text -> Future   json -> Future
```

**Arguably not stdlib at all.** Precedent both ways: Go put `net/http` in the standard
library and got a universally-available server plus a permanent compatibility obligation
on an evolving protocol; Python's `urllib` is in the stdlib and is so unpleasant that
`requests` became the de-facto standard anyway, leaving two HTTP clients where one was
wanted. **Cost of the Python outcome is the relevant one for Phalcom**, which has no
deprecation mechanism: a bad stdlib HTTP client is permanent. Ship the client if any;
leave the server to a package.

### 4.5 User-visible threads — **an open question, not a given (S-3)**

If Phalcom stays single-VM-thread, IO concurrency comes from a pool behind §4.1(b)/(c) and
the user never sees a thread. If threads become user-visible, the language needs `Mutex`,
`Atomic`, a `Send`-analog, and **the entire object model's mutability and GC story
changes** — ADR-0050's precise-root enumeration assumes owned `VM::stack`/`frames` `Vec`s.

**Precedent with its cost.** CPython's GIL made C extensions easy and single-thread
performance good; **cost** is PEP 703, a decade-plus project to remove it, precisely
because the *ABI* exposed the assumption (ffi.md §7 makes this same point about the C-API).
Ruby is the same story. JavaScript refused shared-memory threading entirely and shipped
Web Workers with message passing plus, much later, `SharedArrayBuffer`; **cost** is that
CPU-bound work is awkward, but the object model never had to become thread-safe.

**Decide this before §4.1**, because it picks §4.1's implementation.

---

## Tier 5 — developer surface

### 5.1 Testing

Phalcom tests itself from Rust (golden fixtures, `phalcom-core/tests/`); **the language has
no test framework of its own**, so `.ph` libraries cannot be tested in `.ph`.

```
Test.suite(_, body:)    Test.case(_, body:)    Test.skip(_, body:)
Test.beforeEach(_)      Test.afterEach(_)
Assert.isTrue(_)   isFalse(_)   equals(_, _)   notEquals(_, _)
Assert.isNone(_)   isSome(_)    isOk(_)        isErr(_)
Assert.raises(_, kind:)         Assert.near(_, _, epsilon:)
Bench.measure(_, body:)         Bench.iterations(_)
```

### 5.2 Logging

```
Log.trace(_)  debug(_)  info(_)  warn(_)  error(_)
Log.level(_)  Log.withContext(_, do:)   Log.target(_) -> Logger
Logger#write(_, level:)                  // to any Writer (§3.2)
```

### 5.3 Reflection

Partly landed: `Method:1677`, `Behavior:1669`, `Class:36` in `core.ph`, plus
[ADR-0028](../../../adr/accepted/0028-amend-floor-admit-method-reflection.md).

```
Mirror.on(_) -> Mirror
#class   fields -> List   fieldAt(_) -> Option   methods -> List
#respondsTo(_) -> Bool    send(_, args:) -> Result
Class#name   superclass   subclasses   methods   fields   isSealed
```

`Mirror#send(_, args:)` is a capability hole if a sandbox is ever wanted (S-7).

### 5.4 Backtraces — **built but unwired**

`Error` exists (`core.ph:54`); no stack-trace object does. Note the tree already contains
`print_rt`/`runtime_error`/`SourceLoc` machinery that `cmd_run` bypasses — this is
**existing dead code, not missing code**, and grepping for "backtrace" does not find it.

```
Error#backtrace -> Backtrace      Error#cause -> Option      // error chaining
Backtrace#frames -> List          Backtrace#toString
Frame#function   file   line   column
```

### 5.5 CLI

```
Cli.new(_)   #flag(_, short:, help:)   option(_, help:)   positional(_, help:)
#subcommand(_, do:)   #parse(_) -> Result   #help -> String
```

### 5.6 Serialization protocol

One protocol, many formats — so `Toml`/`Yaml`/`Csv` can live outside the stdlib without
each inventing its own shape.

```
Encodable#encodeTo(_)             Decodable.decodeFrom(_) -> Result
```

---

## Tier 6 — ecosystem plumbing

**Module resolution.** ADR-0045 §Decision 7 resolves `import` to **relative source only** —
no package names, no versions, no lockfile, no registry. Everything in Tier 2 – Tier 5 has to live
somewhere, and today "somewhere" is `core.ph` or nothing.

**A `core`/`std` split.** `core.ph` is 1713 lines and already holds `Tracer:1556`,
`Backoff:1592`, `On:1640`, and `Tier:1657` alongside `Object:1` and `Bool:421`. Tiers 2–5
cannot all land there. Note the interaction with ADR-0019: `core.ph` is loaded at
bootstrap, so anything placed in it is in the kernel-load DAG whether it needs to be or
not.

**FFI module registry.** ffi.md §6's one free move, restated because this catalog is its
motivation: make `install_primitives` accept a registry of module descriptors rather than
hard-coding each. It costs approximately nothing now, and it is what keeps ffi.md's tier
(c) — out-of-tree source, in-tree binary — a later drop-in rather than a rewrite. With
~15 native modules implied by Tier 3 and Tier 4, the hard-coded form stops being tenable.

---

## Build order

Each item's blockers are strictly above it. This ordering is the actual deliverable of
this document.

```
 1.  Int/Float split + bitwise           §0.1, §0.2   ADR-0024 already ratified
 2.  temp_roots (U-GC step 4)            ffi.md F-2   soundness; already crashing
 3.  Resource protocol + `using`         §1.1, S-1    ruling required
 4.  Bytes                               §1.2         see bytes.md
 5.  Sealed variants + exhaustiveness    §0.3         half built
 6.  Comparable / Hashable / sort        §0.6
 7.  Path                                §3.1, S-4    ruling required
 8.  Reader/Writer/Seekable protocol     §3.2         BEFORE File, not after
 9.  Blocking-vs-reactor ruling          §4.1, S-2    decides File's signatures
10.  File / Fs / Stdio / Env             §3.3–§3.5
---- substrate complete; everything below is ordinary work ----
11.  Encoding, StringBuilder, Char       §1.3–§1.5
12.  Time                                §3.8
13.  Math, Random, Regex, Json           §2.1–§2.4
14.  Process, Os                         §3.6, §3.7
15.  Timers, channels, cancellation      §4.1, §4.2
16.  Net, Tls, Http                      §4.3, §4.4
17.  Testing, Logging, Backtraces, Cli   Tier 5
18.  WeakRef                             §0.5         anywhere after 2
19.  Crypto, BigInt/Decimal, Uuid        §2.5–§2.7
```

Steps 1–10 are the project. Steps 11–19 are ordinary library work once the substrate
exists — and are heavily parallelizable, which 1–10 are not.

---

## What this catalog deliberately excludes, and why

- **GUI, ORM, template engines, web frameworks, async runtimes-as-libraries.** Packages,
  not standard library. The decorator drafts (`decorators-web.md`,
  `decorators-persistence.md`) already sit at this boundary and both note they have no
  owning unit and no underlying surface.
- **Finalizers proper.** §1.1 is the alternative, and its precedent table is the
  argument: Java deprecated `finalize`, Python's `__del__` remains a resurrection and
  timing hazard, and C# needed `IDisposable` anyway. Skipping straight to the thing all
  three converged on is available to Phalcom precisely because it is late.
- **`dlopen` plugins.** ffi.md §6 tier (b) — the only tier that cannot be made safe. Wasm
  is the door if this is ever wanted (ffi.md F-10).
- **A shell-string `Command`.** §3.6 states why.

## What this document precludes

Nothing — it is a draft with no owning unit, and no entry here is a commitment to build.

What it is intended to preclude is a **specific failure mode**: adding `File` because
files are obviously useful, discovering it needs `Bytes`, adding `Bytes` over `List`
because `Int` is not built, and shipping a filesystem API whose read path allocates a
16-byte `Value` per octet — then rebuilding all of it. The build order exists so that if it is
departed from, the departure is deliberate.

The one **real** preclusion risk in this document is §4.1: if Tier-3 signatures are
written before S-2 is ruled, the ruling is made by accident, and reversing it means
breaking every IO selector in the language.

---

## Open questions

Numbered for citation. Add rows; do not renumber.

| # | Question | Why it is open | Would resolve via |
|---|---|---|---|
| **S-1** | Resource lifetime: `using`-syntax + `Disposable` + leak reporting, or admit finalizers? | ADR-0050 §Context banks "no finalizers exist" as a safety property. Native handles are the classic pressure to grow them. §1.1 recommends the former and gives the precedent; it is a **user ruling**. Also ffi.md **F-3**. | A ruling, then an ADR. If finalizers: ADR-0050 must be amended, not worked around. |
| **S-2** | IO shape: blocking, thread-pool + completion queue, or reactor? | It changes **signatures**, not implementations — `Result` vs `Future<Result>` on every Tier-3 read. `System.sleep(_)` is documented still-open (`system.md:75`) and `open-questions.md` §15's fairness question is its blocker. | A ruling on the *signature* even if only blocking is built. |
| **S-3** | Are threads ever user-visible? | Decides S-2's implementation, and if yes, changes the object model's mutability story and ADR-0050's root enumeration. §4.5. | A ruling. Should precede S-2. |
| **S-4** | `Path` as opaque `Bytes`-backed class, or `String` with lossy conversion? | `Object::Str` is UTF-8-enforced, so `String`-paths cannot represent real POSIX filenames. §3.1's precedent shows Python paid for the shortcut twice. | A ruling; cheap now, expensive after `Fs` ships. |
| **S-5** | Do `System.print`/`args`/`env`/`clock`/`now` forward to the new surfaces, or get deprecated? | system.md §2's table is landed. Phalcom has **no deprecation mechanism**, so "both forever" is the default outcome. | A ruling plus, possibly, a deprecation mechanism — which is its own missing feature. |
| **S-6** | Regex: linear-time engine (no backreferences, no lookaround) or backtracking? | §2.3 — the ReDoS trade is a real design decision and should not be inherited silently from whichever crate is picked. | A ruling at the time the unit is scoped. |
| **S-7** | Is there a capability/sandbox model, and does `System` remain the single effect receiver? | system.md §1's design rule is "effects are named, not ambient" — one place to stub or sandbox. If `File`/`Fs`/`Net` become plain globals that property is lost in one commit, and `Mirror#send` (§5.3) is a hole regardless. | A ruling **before** Tier 3 ships, or it is decided by default. |
| **S-8** | What does a weak arm cost `vm/gc.rs`? | §0.5 — a second mark pass plus weak-slot clearing before sweep. **Unverified**; nobody has scoped it against ADR-0050's stated algorithm. | Read `vm/gc.rs`; spike it. |
| **S-9** | What is `StringCodePointSequence`'s (`core.ph:388`) element type today? | §1.4 asserts `Char` is missing but **did not verify** what the existing sequence yields — possibly `Number` code points, possibly single-char `String`s. | Read `core.ph:388-420`. |
| **S-10** | Can `VM::ready_queue` accept a push from a signal handler context? | §3.6 — signal delivery must be queued, not re-entrant. Suspected **no**; this may be the first thing that forces S-3. | Read the ready-queue implementation; likely needs a self-pipe or an atomic flag drained at a safepoint. |
| **S-11** | Where do Tiers 2–5 physically live, given `core.ph` is bootstrap-loaded? | Tier 6 — a `core`/`std` split interacts with ADR-0019's kernel-load DAG, and ADR-0045 gives no package mechanism to put a `std` behind. | Design work, gated on the module-resolution question. |
