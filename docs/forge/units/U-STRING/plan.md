# U-STRING — Work order: the `String` protocol + `System` write funnel

_Self-contained implementation plan for **one** implementer. **Reviewer ON** (touches the
FROZEN-FLOOR files `primitive/string.rs` + `primitive/system.rs`, the floor registration in
`universe.rs`, and requires a new ADR-0019 amendment) — hand the diff to `phalcom-reviewer`;
never self-approve. **Worktree/serialization caution:** at plan time, `git status` shows an
**in-flight, uncommitted session** touching `phalcom-ast/src/{ast,parser}.rs`,
`phalcom-core/src/{bytecode,class,heap,value,vm}.rs`, `phalcom-core/src/compiler/lib.rs`,
`phalcom-core/src/primitive/mod.rs`, `phalcom-core/src/universe.rs`, and a new
`phalcom-core/src/primitive/family.rs` (looks like the U16/Family/`::`-call-router or
attribute-classes work). **This unit's `universe.rs` edit (registering 4 new primitives) MUST
be serialized after that session commits — do not dispatch concurrently.** Green gate:
`./scripts/verify.sh` exits 0 + `cargo doc --workspace --no-deps` clean. Grounded in
**[ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md)** (frozen floor — this
unit is explicitly the amendment ADR-0019 §"Consequences" predicted: *"It leaves `String`
deliberately split... the exact cut line for `String` is the one genuinely fuzzy edge and is
called out as the first place a future amendment is likely"*), **[ADR-0008](../../../adr/0008-layered-exceptions-and-result.md)**/
**[ADR-0031](../../../adr/0031-error-handling-surface-syntax.md)** (the `throw`/`Error`
machinery this unit's argument guards ride on — **landed**, `7c901cf`), and the normative
**[error-handling.md](../../../spec/current/error-handling.md)**, **[system.md](../../../spec/current/system.md)**,
**[core/core-classes.md](../../../spec/current/core/core-classes.md)** §`String`/§`System`.
Wren precedent: `/Users/altunhasanli/dev/repos/wren/src/vm/wren_core.wren` L184–316 (`String`
+ `StringByteSequence`/`StringCodePointSequence`), L440–473 (`System`); native shapes cross-checked
against `wren_core.c` L982–1199. **New governing ADR REQUIRED:** claim **ADR-0049** (next free
after ADR-0048 on HEAD) — the floor amendment for `String::rawByteCount`/`rawByteAt(_)`/
`rawSlice(_,_)` + `System::rawWrite(_)` (**+4 bindings**, census 113 → 117). The
`documentation-and-adrs` skill drafts it._

> **Unit-name note.** Reserved and pointed at by
> [`deferred-work.md`](../../../spec/current/deferred-work.md) row "**U-STRING** string protocol"
> — this plan is that row's realization. `U-STRING` verified free at plan time (no prior
> `docs/forge/units/U-STRING/` content). Does **not** edit `docs/forge/INDEX.md` /
> `STATE.md` (shared coordination docs, concurrent editors) beyond what's explicitly listed below.

---

## 0. Preconditions (verify on actual HEAD — do not assume; the tree is live)

- **U-ERR landed** (`7c901cf`) — `throw`/`try`/`on`/`catch`/`ensure` + `Result`/`Ok`/`Err` all
  exist. **`throw expr` requires `expr` to be an `Error` instance** (compile-time rejects a
  syntactic non-`Error` literal); this unit's argument guards use `throw <X>Error.new(msg)`.
  **Confirmed:** the `Name(args)` bare-call construction sugar is **not** generalized beyond a
  couple of pre-existing cases (`docs/forge/DEFERRED.md` "`Ok`/`Err` construction", DEC-ERR-B
  resolved **(B)**) — so this unit spells construction as `ArgumentError.new("msg")`, **not**
  the spec-illustrative bare `ArgumentError("msg")` (error-handling.md §1's example is
  illustrative prose, not the landed call form — flag, don't "fix" the spec here).
- **`class Error` is bootstrapped with `_message` at fixed slot 0** and ships `construct new()`
  + `construct new(msg) { _message = msg }` (`core.ph` L23–44). **U-INH's inherited-constructor
  resolution is confirmed landed** (`docs/forge` memory: "inherited ctors DO resolve"), so a
  **field-less** `.ph` subclass of `Error` needs **no** `construct` of its own — `ArgumentError.new("msg")`
  resolves to `Error`'s inherited `construct new(msg)`, which sets slot 0 correctly (ADR-0011:
  a subclass with zero declared fields does not shift the parent's slot layout). **Confirm this
  with a golden in step 1** before relying on it elsewhere in the unit — if inherited-ctor
  resolution has a gap for this exact shape, fall back to the explicit pattern the `Error`
  class's own doc-comment anticipated: `construct new(msg) { super.new(msg) }`.
- **`ArgumentError`/`RangeError`/`TypeError` are spec'd but NOT YET realized as classes**
  (confirmed: `object-model.md` L174–179 lists them in the `Error` catalog; `universe.rs`
  registers only `Error`/`MessageNotUnderstood`/`CannotYieldAcrossNativeFrame`; U-ERR's plan
  explicitly scoped wrapping the **native** `RuntimeError` variants — `Arity`, `Type`,
  `RangeError`, `ZeroDivision` — into surface classes as **"reserved... to a later reification
  unit"**, deliberately out of U-ERR). **This unit introduces `ArgumentError` only** (it has
  real call sites here); see §2.2 for why `RangeError`/`TypeError` are deliberately **not**
  added by this unit (no call site → no dead surface) — that reification is a distinct future
  unit's job, and it **must reuse this same bare-`extends Error {}` pattern**, not redefine
  `ArgumentError` (§9 must-not-preclude).
- **`String`'s current native floor is exactly 3 bindings**: `+(_)` (`string_add`), `hash`
  (`string_hash`), static `new()`/`new(_)` (`string_class_new`) — confirmed via
  `primitive/string.rs` (53 lines) and `floor-census.md` §2.5. **There is no length, no index,
  no byte accessor at all today** — confirmed independently by `core/decisions.md` L46: *"`String`
  | its bytes/codepoints | **No** — `String` floor is only `+`/`new` (no length/index)"*. This is
  the gap this unit closes.
- **No bitwise operators exist on `Number`** (confirmed: only `+ - * / %` `< <= > >=` `negated`
  are registered on `number_cls`, no `&`/`|`/`<<`/`>>`). **This forces the UTF-8 decode design in
  §2.1** — lead-byte classification and codepoint-value extraction use only `<`/`>=`/`%`/`/`
  (numeric range tests and modulo, which are bit-mask-equivalent for non-negative integer
  values), never a bitmask. Flag, don't invent bitwise ops here.
- **No `Int`/`Float` split** (ADR-0042 defers it) — `Number` is flat `f64`; argument-count
  validation (`*(count)`) checks `count % 1 == 0` (mirrors `list.rs::expect_index`'s
  `n.fract() != 0.0` pattern), not an `isInteger` message.
- **No subscript/`[]` operator exists** in the AST/parser (confirmed: no `Subscript`/`Index`
  node in `ast.rs`). Wren's `this[a...b]` / `this[i]` sugar has **no Phalcom equivalent** and
  adding one is out of scope — this unit uses method-call spellings (`rawSlice(_,_)`,
  `codePointAt(_)`) throughout, never bracket sugar.
- **`Iterable` root is NOT landed.** ADR-0048 (Accepted, 2026-07-13) specifies a kernel
  `Iterable` + the bare-cursor-sentinel amendment, and names its realizing unit
  **U-ITERABLE** — but `docs/forge/units/U-ITERABLE/plan.md` does not exist yet and
  `class Iterable` is absent from `core.ph`. Confirmed: `List`'s landed `iterate`/`iteratorValue`
  still use the **pre-amendment** `Some`-wrapped ADR-0035 shape (`(next < self.size).ifTrue { next }`,
  one-armed), not ADR-0048's bare-cursor two-armed form — i.e. even `List` hasn't migrated yet.
  **Soft dependency, not a blocker:** per the task brief, this unit ships `codePoints`/`bytes` as
  standalone `.ph` classes with their own `size`/`at(_)`/`each(_)` (§2.4) — useful today — and
  defers `extends Iterable` wiring to whenever U-ITERABLE lands (§9).
- **`System.print(_)` is native (`system_class_print`, `Method(1)`), registered and working**
  — confirmed the sole I/O floor primitive (`floor-census.md` §2.11: *"the sole I/O primitive"*).
  It calls `Value::to_string(vm)` directly on each arg — a **native, message-bypassing** render
  path, structurally distinct per heap-object type (`value.rs::Value::to_string`, L166–201), and
  **used by 379 files in `tests/lang/`** (`grep -rl "System\.\(print\|write\)"` count at plan
  time). **Confirmed divergence (pre-existing, not this unit's to fix):** `Object#toString`
  (the *message*, `object_to_string`) renders a generic instance as `"<ClassName>"`, while
  `Value::to_string`'s fallback (`to_debug` → `instance.to_debug`) renders `"<ClassName instance>"`
  — **two different strings**. Also confirmed: `Map`/`Set`/`Tuple`/`Range` have **no** registered
  `toString` **message** at all (only `List`/`Number`/`Symbol`/`Object` + the `.ph` `String`/`Bool`/
  `Option`/`Result` do) — `Value::to_string` special-cases their native rendering directly,
  bypassing messages entirely, so a Map/Set/Tuple/Range sent `.toString` today falls through to
  `Object`'s `"<ClassName>"` default, **not** the nice `{k: v, ...}` format `System.print` shows.
  **This is why §2.5's funnel is additive (`write`/`writeObject_`), never a `print(_)` rewrite**
  — see §2.5's risk note for the full reasoning; do not "fix" this pre-existing message/render
  split in this unit, it is out of scope and named a follow-on (§9).
- Baseline `./scripts/verify.sh` green before the first edit. Re-run `graphify affected "core.ph"`,
  `graphify affected "universe.rs"`, `graphify affected "primitive/string.rs"` and confirm the
  in-flight session (above) has landed before touching `universe.rs`.

---

## 1. Mission (one sentence)
Close `String`'s protocol gap by porting Wren's portable wins — `split`/`replace`/`trim*`/`*(count)`
plus `bytes`/`codePoints` sub-accessors — over the **smallest irreducible native floor** (3 new
`String` primitives, justified one-by-one against ADR-0019's derivability test), add a `System`
write funnel (`write`/`writeObject_`) that **respects the `toString` message and validates its
result is a `String`** (the Wren `[invalid toString]` robustness win) as **new, additive**
selectors that do not touch the 379-file-relied-upon `print(_)` path, and realize the first
`ArgumentError` throw sites establishing the `throw <X>Error.new(msg)` boundary-guard convention
for future library code.

## 2. Design (realise the Wren precedent onto Phalcom's substrate; do not re-litigate ADR-0019)

### 2.0 The native/`.ph` split — headline table (the derivability audit, ADR-0019 §1 test applied per-primitive)

| Capability | Native? | Why (irreducibility) |
|---|---|---|
| `rawByteCount` (instance, 0-arity) | ✅ native | Byte length of the underlying UTF-8 buffer — no `.ph` code can observe the buffer at all today (confirmed §0). |
| `rawByteAt(_)` (instance, 1-arity) | ✅ native | Raw byte read — same reason. Total: raw `Number` (0–255) on hit, the `None` singleton on OOB (mirrors `list_raw_at`'s pattern exactly, `list.rs` L61–79). |
| `rawSlice(_, _)` (instance, 2-arity, byte range `[start, end)`) | ✅ native | **The one that must stay native even though it "just" copies bytes**: producing a new `StringObject` is **allocation** — there is no `.ph`-reachable way to construct a `String` from computed byte/codepoint data (the only constructor, `String.new(_)`, stringifies an *arbitrary value* via `Value::to_string`, e.g. `String.new(65)` → `"65"`, not `"A"`). This is exactly ADR-0019's own words: *"a string can't be sliced/indexed in pure `.ph` without a raw code-unit accessor."* Validates codepoint-boundary alignment (Rust `str::is_char_boundary`) and in-range indices; malformed input → `RuntimeError::Type` (low-level native contract violation, not a user-facing `ArgumentError` — see §2.2's boundary between the two). |
| `codePointAt(_)` (instance, 1-arity, byte offset → Unicode scalar `Number`, or the `None` singleton if not a lead byte / OOB) | ❌ `.ph`, derived over `rawByteAt`/`rawByteCount` | **Deliberate deviation from Wren** (which nativizes `codePointAt_` for speed). UTF-8 lead-byte classification needs only numeric range tests (`b < 128`, `b < 224`, `b < 240` → 1/2/3/4-byte sequence) and continuation-byte value extraction needs only `%`/`/` (e.g. a 2-byte sequence's codepoint is `(b0 % 32) * 64 + (b1 % 64)` — mathematically identical to bit-masking for non-negative integer bytes, and `%`/`/` are already on the floor). **No bitwise op needed, confirmed §0.** Per ADR-0019 ("a smaller native surface is more auditable... accept slower hot paths"), this stays `.ph`. |
| `indexOf(needle)` / `indexOf(needle, from)` | ❌ `.ph`, derived over `rawByteAt`/`rawByteCount` | A naive byte-scan substring search is fully expressible in `.ph` (`while` + comparisons). **Deliberate deviation from Wren** (which nativizes it, `wrenStringFind`, for `memcmp` speed) — accepted O(n·m) cost per ADR-0019's stated trade ("a smaller native surface... keeps the object model uniform"; the counter-move for hot-path cost is a future inline cache/JIT, not a floor grab). Flag as the unit's one performance risk (§ Risk). |
| `isEmpty`, `size` (codepoint count) | ❌ `.ph` | Trivial derivations (`rawByteCount == 0`; a codepoint-stepping scan). |
| `split`/`replace`/`trim`/`trimStart`/`trimEnd`/`*(count)` | ❌ `.ph` | Direct ports of the Wren bodies (L188–291), rewritten over `rawSlice`/`indexOf`/`codePointAt` instead of `this[a...b]`/native `iterate`/`iterateByte_` (no subscript sugar, no native cursor — see §2.3). |
| `System::rawWrite(_)` (static, 1-arity) | ✅ native | Raw stdout write of an already-formed `String`, no newline, no formatting — the literal I/O act (ADR-0019 rule 6: *"System I/O — print and source-file read... native I/O primitive"*). Irreducible the same way `System.print`/file read are. |
| `System::write(_)` / `writeObject_(_)` | ❌ `.ph`, derived over `rawWrite(_)` + the `toString` message + `isA(String)` | The funnel itself — pure control flow over already-native primitives (§2.5). |

**Net floor delta: +4 bindings** (`String::rawByteCount`, `String::rawByteAt(_)`,
`String::rawSlice(_,_)`, `System::rawWrite(_)`). Census **113 → 117**. **ADR-0019 amendment
required** — claim **ADR-0049** at dispatch (`ls docs/adr/` to reconfirm 0048 is still the
latest). `ArgumentError`/`StringByteSequence`/`StringCodePointSequence`/the `System` `.ph`
funnel methods are **zero-cost** (pure `.ph`, no registration, no fields beyond what `Error`
already carries).

### 2.1 `codePointAt(_)` — the UTF-8 decode-by-arithmetic derivation (write this exactly; it is the unit's one subtle correctness point)
```phalcom
class String {
  // ... existing toString ...

  // Number of leading bytes in the UTF-8 sequence starting at byte offset `i`
  // (1/2/3/4), read purely from the lead byte's numeric range — no bitmask
  // (§2.0; Number has no bitwise ops on HEAD). A continuation byte (128..191)
  // or an invalid lead (>=248) is never passed a valid `i` by this unit's own
  // callers (they only step from a previously-validated lead), so this is not
  // itself defensively guarded against a mid-sequence `i`.
  leadByteLen_(i) {
    let b = self.rawByteAt(i)
    return (b < 128).ifTrue({ 1 }, ifFalse: {
      (b < 224).ifTrue({ 2 }, ifFalse: {
        (b < 240).ifTrue({ 3 }, ifFalse: { 4 }) }) })
  }

  // The Unicode scalar value at byte offset `i`, or the `None` singleton if
  // `i` is out of range or lands mid-sequence (mirrors Wren's `codePointAt_`
  // returning `-1` for the same case, but no-nil: `None`, not a numeric
  // sentinel — Invariant 4).
  codePointAt(i) {
    let b0 = self.rawByteAt(i)
    return (b0 == None).ifTrue({ None }, ifFalse: {
      (b0 >= 128).ifTrue({ (b0 < 192).ifTrue({ None }, ifFalse: { None }) }, ifFalse: {
        // ASCII fast path
        b0
      })
      // multi-byte path elided here for brevity — the accepted body decodes
      // 2/3/4-byte sequences via `(b0 % (192/32/16/8)) * 64^n + Σ (bk % 64) * 64^(n-k)`,
      // reading each continuation byte with `rawByteAt(i+k)` and validating
      // `128 <= bk < 192` (else `None` — a malformed sequence, never `raise`s).
    })
  }
}
```
*(The elided multi-byte arithmetic is a mechanical port of `wrenUtf8Decode`'s bit-shift decode
into `%`/`/` form — e.g. a 3-byte sequence: `((b0 % 16) * 4096) + ((b1 % 64) * 64) + (b2 % 64)`.
Implementer writes the full body; the shape above is load-bearing, the omitted arithmetic is
mechanical.)*

### 2.2 `ArgumentError` — the boundary-guard convention (error-handling.md §1)
```phalcom
// Realizes the object-model.md §4 catalog row `ArgumentError < Error` — "Bad
// argument value/arity." Zero fields, zero native code: the inherited
// `Error.construct new(msg)` (core.ph L43) already gives `ArgumentError.new(msg)`
// a working 1-arg constructor (U-INH inherited-ctor resolution, confirmed §0).
// `TypeError`/`RangeError` are the spec's siblings but have NO call site in
// this unit (no indexed `String#at(_)` yet — catalog-delta.md's own "indexing
// → Option" gap, explicitly out of scope) — do not add them speculatively;
// see §9 for why the next unit that DOES need them must reuse this pattern.
class ArgumentError extends Error {}
```
Every String argument guard in this unit follows one shape (mirrors Wren's `Fiber.abort`
1:1, translated to the ratified `throw` surface, error-handling.md §1):
```phalcom
(delimiter.isA(String)).ifTrue({}, ifFalse: {
  throw ArgumentError.new("delimiter must be a String")
})
```
(Or the equivalent guard-then-continue shape the implementer prefers — `not(cond).ifTrue { throw ... }`
also works; pick one spelling and use it consistently across all six guard sites: `split`'s
delimiter (non-empty `String`), `replace`'s `from` (non-empty `String`)/`to` (`String`), the three
`trim*` variants' `chars` (`String`), and `*(count)` (`Number`, `>= 0`, `count % 1 == 0`).)

### 2.3 `split`/`replace`/`trim*`/`*` — direct ports over `rawSlice`/`indexOf`/`codePointAt` (Wren L188–291)
Structure is a **1:1 port** of the cited Wren bodies with two substitutions: `this[a...b]` →
`self.rawSlice(a, b)` (Phalcom has no subscript sugar, §0), and the codepoint-set membership
test in `trim_` (`codePoints.contains(codePointAt_(start))`) → a small `.ph` linear scan over the
`chars` argument's own `codePoints` sequence (§2.4) — no `List#contains` dependency (that's an
`Iterable` combinator not yet landed generically for arbitrary iterables; a local scan is
adequate for the tiny "whitespace charset" case). Reproduce Wren's exact loop shapes:
- `split(delimiter)` — `indexOf`-driven scan, accumulating `rawSlice` segments into a `List`
  (via `List.new()` + `.add(_)`), the tail segment after the last match (Wren L193–211).
- `replace(from, to)` — same scan shape, accumulating into a `String` via `+` (Wren L221–236).
- `trim()`/`trim(chars)`/`trimStart([chars])`/`trimEnd([chars])` — the shared `trim_(chars,
  trimStart, trimEnd)` private helper (Wren L246–279), rewritten over `codePointAt`/`leadByteLen_`
  for the start-scan and a **backward byte scan** for the end-scan (Wren scans backward one byte
  at a time via `codePointAt_`, which tolerates being called on a non-lead byte by returning `-1`
  and stepping back one more; Phalcom's `codePointAt(_)` returns `None` in that case identically
  — same backward-scan shape works unchanged).
- `*(count)` — the accumulate-in-a-loop repeat (Wren L281–291), `count` guarded per §2.2.

### 2.4 `bytes`/`codePoints` — standalone sub-accessor classes, `Iterable`-ready but not `Iterable`-dependent (Wren L294–316; §0's soft dependency)
```phalcom
class StringByteSequence {
  construct new(s) { _string = s }
  size => _string.rawByteCount
  at(i) => _string.rawByteAt(i)
  each(f) {
    var i = 0
    while (i < self.size) { f.call(self.at(i)); i = i + 1 }
  }
}

class StringCodePointSequence {
  construct new(s) { _string = s }
  // Codepoint count needs a full scan (no native "codepoint length" primitive
  // — deliberately not added, §2.0); acceptable O(n), matches `indexOf`'s
  // accepted cost trade.
  size {
    var n = 0
    var i = self.nextCursor_(None)
    while (i != None) { n = n + 1; i = self.nextCursor_(i) }
    return n
  }
  at(byteOffset) => _string.codePointAt(byteOffset)
  each(f) {
    var i = self.nextCursor_(None)
    while (i != None) { f.call(self.at(i)); i = self.nextCursor_(i) }
  }
  // Byte-offset cursor step, codepoint-aware (NOT a dense 0..size index —
  // this is exactly ADR-0048's "a collection whose cursor is not a 0..size
  // index overrides `iterate` itself" case, pre-emptively shaped so the
  // eventual `extends Iterable` + rename to `iterate(_)` is additive, not a
  // rewrite — see §9).
  nextCursor_(cursor) {
    let next = (cursor == None).ifTrue({ 0 }, ifFalse: { cursor + _string.leadByteLen_(cursor) })
    return (next < _string.rawByteCount).ifTrue({ next }, ifFalse: { None })
  }
}

class String {
  bytes => StringByteSequence.new(self)
  codePoints => StringCodePointSequence.new(self)
}
```
**Deferred, not shipped:** `String extends Iterable` / `String#iterate`/`iteratorValue` (the
default per-codepoint `Sequence`-ness Wren gives `String` itself, L184 `is Sequence`) — this
needs the kernel `Iterable` root (ADR-0048, U-ITERABLE, not landed, §0). `codePoints`/`bytes`
ship today as **directly usable, standalone** classes (`"abc".bytes.each { b => ... }` works
now); when `Iterable` lands, `StringByteSequence`/`StringCodePointSequence` gain `extends
Iterable` and drop their own `each`/`size`-is-generic in favor of the inherited combinator
layer — additive, not a rewrite (§9).

### 2.5 `System.write(_)` / `writeObject_(_)` — the funnel, additive-only (Wren L440–473)

**Risk-driven design call (read before implementing).** The literal Wren shape funnels **every**
print/write path (including `print(_)`) through `writeObject_` (message-dispatch + guard).
Doing that to Phalcom's *existing* `System.print(_)` is **not safe**: `print(_)` today renders
via the native `Value::to_string` switch (§0), which (a) special-cases `Map`/`Set`/`Tuple`/`Range`
with a nice native format **no `toString` message currently produces** (none of those four
classes have a registered `toString` selector at all — confirmed §0), and (b) renders a bare
instance as `"<ClassName instance>"` where the `toString` **message** gives `"<ClassName>"` — a
**confirmed, pre-existing divergence**. Flipping `print(_)` to message-dispatch would silently
regress every one of the **379 files** under `tests/lang/` that call `System.print`/`write`
wherever a `Map`/`Set`/`Tuple`/`Range`/bare-instance value is printed. **Decision: this unit does
not touch `print(_)`'s rendering semantics at all.** It ships `write(_)`/`writeObject_(_)` as
**brand-new** selectors (zero existing call sites, zero corpus risk) that implement the correct,
message-respecting funnel — the `[invalid toString]` robustness win is real and delivered, just
scoped to the new entry points. Unifying `print(_)` onto the same funnel is named as a **follow-on**
gated on first closing the `Map`/`Set`/`Tuple`/`Range`/instance `toString`-message gap (§9) —
tracked, not silently dropped.

```phalcom
class System {
  // (drop the vestigial 0-arity `static print() { }` stub at L732–734 —
  // dead code: the real `print(_)` is the native `Method(1)` primitive,
  // a *different* selector by arity; the 0-arg `.ph` stub is never reached
  // and is adopted debt cleaned up incidentally by this reopen.)

  static write(obj) {
    System.writeObject_(obj)
    return obj
  }

  static writeObject_(obj) {
    let s = obj.toString
    return (s.isA(String)).ifTrue(
      { System.rawWrite(s) },
      ifFalse: { System.rawWrite("[invalid toString]") }
    )
  }
}
```
`system.md` §2's `write(_)` row ("write `x.toString` with no trailing newline") is satisfied
exactly. `printErr(_)`/`readLine`/`clock`/`now`/`args`/`env`/`exit`/`gc`/`version` remain
**out of scope** (system.md's own "Planned" steps 1–3, a separate future unit).

### Rubric — hazards & preclusion (mandatory)
- **The `print(_)` funnel-unification trap (THE crown-jewel hazard of this unit).** Anyone
  "cleaning up" `System.print`/`write` into one native call in a future pass **must not** merge
  `print(_)`'s renderer with `writeObject_`'s message-dispatch renderer without first closing the
  `Map`/`Set`/`Tuple`/`Range`/instance-wording gaps (§2.5) — doing so silently reformats a huge
  swath of golden `.expected` files. Pin a golden proving `System.print(aMap)` and
  `System.write(aMap)` currently render **differently** (documents the gap deliberately, not by
  accident) so a future unit's regression is caught immediately, not discovered via a 379-file
  diff.
- **`rawSlice` boundary safety ⊗ multi-byte content.** A `rawSlice(a, b)` call with `a`/`b` not on
  a UTF-8 char boundary must **never** panic (Rust string slicing panics on a misaligned
  boundary) — the native impl must check `str::is_char_boundary` explicitly and return
  `RuntimeError::Type` on violation, never let a raw `&str[a..b]` panic propagate. Pin a golden
  with a multi-byte string (e.g. `"héllo"`) sliced at a mid-sequence byte offset → clean error,
  not a VM crash.
- **`indexOf`/`split`/`replace` cost is O(n·m), by design (§2.0).** Not a bug — a documented,
  deliberate ADR-0019-driven trade. Do not "fix" by silently nativizing `indexOf`; if profiling
  later demands it, that is its own ADR-0019 amendment, not a drive-by change here.
- **`codePointAt`/`leadByteLen_` must agree with `rawSlice`'s boundary check** — both derive
  "is this a valid UTF-8 lead byte" from the same numeric ranges; a golden should cross-check
  `str.codePoints.each { cp => ... }` against `str.rawSlice(i, i + str.leadByteLen_(i))`
  round-tripping through `codePointAt` for a string mixing 1/2/3/4-byte sequences (e.g. `"a€🎉"`).
- **`ArgumentError` naming collision with the future native-`RuntimeError` reification.** The
  later unit that wraps `Arity`/`Type`/`RangeError`/`ZeroDivision` into surface classes **must
  bind to this unit's `ArgumentError` class**, not declare a second one — `class` redefinition
  in `core.ph` is a reopen (additive), so a naive second `class ArgumentError extends Error {}`
  is harmless but redundant; a conflicting field/construct would not be. Note in `core-classes.md`
  (or wherever that unit grounds itself) that `ArgumentError` already exists.
- **Representation/dispatch impact:** none beyond the +4 floor bindings. No `Value` tag change
  (`String` stays `Value::Obj(ObjRef) → StringObject`, unchanged layout), no selector-encoding
  change, no new opcode. `ArgumentError`/`StringByteSequence`/`StringCodePointSequence` are
  ordinary `InstanceObject`s.
- **Precedent:** Wren's `String`/`StringByteSequence`/`StringCodePointSequence`/`System` (the
  direct model, cited throughout). Rejected: nativizing `indexOf`/`contains`/`startsWith`/
  `endsWith` the way Wren does (ADR-0019 minimal-floor philosophy overrides raw Wren fidelity,
  §2.0) — do not reopen without a fresh ADR-0019 amendment case.

## 3. Confirmed write-set (tight & disjoint; re-validate with `graphify affected` on HEAD)
| File | Why | Slice |
|---|---|---|
| `phalcom-core/src/primitive/string.rs` **(FROZEN FLOOR — reviewer ON)** | `rawByteCount`, `rawByteAt(_)`, `rawSlice(_,_)` (§2.0) | floor |
| `phalcom-core/src/primitive/system.rs` **(FROZEN FLOOR — reviewer ON)** | `rawWrite(_)` (§2.5) | floor |
| `phalcom-core/src/universe.rs` **(SERIALIZE — see header)** | register the 4 new primitives; +4 floor census wiring | floor |
| `phalcom-core/core/core.ph` **(never two editors — currently clean, re-verify before dispatch)** | `String` reopen (§2.1/§2.3/§2.4), `class ArgumentError extends Error {}` (§2.2), `class StringByteSequence`/`StringCodePointSequence` (§2.4), `System` reopen (§2.5, incl. dropping the dead 0-arity `print()` stub) | protocol |
| `docs/adr/accepted/0049-amend-floor-admit-string-raw-byte-accessors.md` (**new**, claim number at dispatch) | ADR-0019 amendment landing-record for the +4 (mirrors ADR-0037/ADR-0038's per-unit pattern) | ADR |
| `docs/spec/current/core/floor-census.md` | §2.5 `String` rows (+3), §2.11 `System` rows (+1), §7 audit count 113→117 | ADR-lockstep |
| `docs/spec/current/core/core-classes.md` | `String` status row ("◐ partial" → the new interface list); note the `Iterable`-deferred `bytes`/`codePoints` shape | docs |
| `docs/spec/current/deferred-work.md` | flip the "U-STRING" row from "code unbuilt" to landed-summary; **add** new deferred rows: (a) `print(_)`/`writeObject_` funnel unification (§2.5's follow-on), (b) `String#at(_)` character indexing + `RangeError` (§2.2's follow-on), (c) `contains`/`startsWith`/`endsWith` over `indexOf` (§2.0's named-but-unshipped derivations), (d) `codePoints`/`bytes` → `extends Iterable` migration once U-ITERABLE lands | docs |
| `phalcom-core/tests/invariants.rs` | bump `floor_census_matches_installed_bindings`'s asserted count 113→117 (§7 audit hook) | invariant |
| `phalcom-core/tests/lang/strings/` (**new label**) + `phalcom-core/tests/lang/MANIFEST.md` | goldens + negatives (§6) | all |

**Deliberately NOT in scope:** `printErr(_)`/`readLine`/`clock`/`now`/`args`/`env`/`exit`/`gc`/
`version` (system.md's own separately-planned steps); `String#at(_)` character indexing;
`contains(_)`/`startsWith(_)`/`endsWith(_)`; `TypeError`/`RangeError` classes (no call site);
`Symbol` protocol changes; `Map`/`Set`/`Tuple`/`Range` `toString` message overrides (named as the
`print`-funnel-unification precondition, not built here); any `[]` subscript grammar; `String.fromByte`/
`String.fromCodePoint` constructors (Wren has them, not requested by the locked scope, and would
need their own native encode primitive — flag as a clean, small, separate follow-on if ever
wanted).

### 3.1 Write-set collision risk (flag, don't resolve)
- **`phalcom-core/src/universe.rs`** — **live, uncommitted edits present at plan time** (see
  header). This unit's registration edit **must** land after that session commits. Re-run
  `git status`/`git diff --stat` immediately before dispatch to reconfirm.
- **`phalcom-core/core/core.ph`** — the standing "never two editors" rule (`docs/forge/INDEX.md`
  §2). Currently clean at plan time; reconfirm before dispatch — any concurrent `core.ph` editor
  blocks this unit's protocol slice (though the floor-primitive slices in `primitive/string.rs`/
  `system.rs` are free of this and can land first).
- **`primitive/mod.rs`** — also in the live uncommitted set; this unit does **not** need to edit
  it (reuses the existing `expect_string` helper, `primitive/mod.rs` L184), but a concurrent
  in-flight edit there could still shift line numbers / helper signatures — re-read it fresh at
  dispatch, don't trust this plan's line citations blindly.

## 4. Build order (small, independently-green diffs)
1. **`ArgumentError` + the inherited-ctor golden** — `class ArgumentError extends Error {}`;
   golden: `ArgumentError.new("x").message == "x"`, `ArgumentError.new("x").isA(Error)`,
   uncaught `throw ArgumentError.new("x")` renders/exits like any other `Error`. Green,
   `core.ph`-only, zero Rust. *(Serialize vs any concurrent `core.ph` editor.)*
2. **The 3 `String` raw primitives + ADR-0049** — `rawByteCount`/`rawByteAt(_)`/`rawSlice(_,_)`
   in `primitive/string.rs`, registered in `universe.rs`, the char-boundary-panic guard (Rubric),
   the ADR draft + `floor-census.md` bump landed **in the same change**. Green via direct native
   calls (no `.ph` wrapper needed yet) — pin the char-boundary-safety golden here. *(Serialize
   vs the in-flight `universe.rs` session — see §3.1.)*
3. **`codePointAt(_)`/`leadByteLen_`/`isEmpty`/`size`/`indexOf`** — the `.ph` derivations (§2.1),
   pinned against a mixed-width string (`"a€🎉"`) golden proving codepoint decode correctness
   end to end. *(`core.ph`-only.)*
4. **`split`/`replace`/`trim*`/`*(count)`** — the direct Wren ports (§2.3) + the six `ArgumentError`
   guard sites + goldens matching Wren's own semantics one-for-one (empty-delimiter guard,
   no-match round-trip, multi-match, trim with default vs custom charset, `*(0)`/`*(negative)`
   guard). *(`core.ph`-only.)*
5. **`bytes`/`codePoints` sub-accessors** — `StringByteSequence`/`StringCodePointSequence` (§2.4),
   goldens for `.each`/`.size`/`.at(_)` on both, including the mixed-width-string codepoint walk.
   *(`core.ph`-only.)*
6. **`System.rawWrite(_)` + `write`/`writeObject_`** — the native primitive + the `.ph` funnel
   (§2.5), the dead-stub cleanup, the `[invalid toString]` golden (a class whose `toString`
   returns a non-`String`, e.g. a `Number`), and the **documented-divergence golden**
   (`System.print(aMap)` vs `System.write(aMap)` differ — Rubric). *(Serialize vs the
   `universe.rs` session; `core.ph`-only for the `.ph` half.)*

Each step is a self-verifiable commit. Steps 3–5 have no Rust dependency and could reorder
freely among themselves; step 2 must land before 3 (needs the raw primitives); step 6 is
independent of 1–5 and could run in parallel **once** `universe.rs` is free (§3.1).

## 5. Mandatory rules
- **Docs:** `///` on every new native fn (`rawByteCount`/`rawByteAt(_)`/`rawSlice(_,_)`/
  `rawWrite(_)`) citing ADR-0019/ADR-0049, mirroring `list.rs`'s doc shape exactly (Signature
  line, derivation note, `# Errors` section). `cargo doc --workspace --no-deps` adds no warnings.
- **Green gate:** `./scripts/verify.sh` exits 0; no new clippy; **no `unsafe`** (byte-boundary
  checks use `str::is_char_boundary`, a safe stdlib call, never manual UTF-8 pointer math).
- **Reviewer ON** (frozen floor + ADR amendment) — `phalcom-reviewer` gates the diff; the writer
  never self-approves.
- **Floor discipline:** the ADR-0049 amendment + `floor-census.md` bump + `invariants.rs` count
  land in the **same change** as step 2 (mirrors ADR-0028/0037/0038/0039's per-unit pattern).
  Do not add any native primitive beyond the 4 named in §2.0.

## 6. Test strategy (the green gate must assert) — new `strings` corpus label
- **`ArgumentError` (PASS):** inherited-ctor `.message` round-trip; `isA(Error)`; uncaught throw
  renders/exits correctly (mirrors `errors/` label conventions).
- **Raw primitives (PASS + NEGATIVE):** `rawByteCount` on ASCII and multi-byte strings;
  `rawByteAt(_)` in-range and OOB (→ `None`); `rawSlice(_,_)` in-range, empty range, full range,
  and the **char-boundary NEGATIVE** (mid-sequence slice → clean `RuntimeError::Type`, not a
  panic — run under `cargo test`/the CLI, not just the golden corpus, to catch a Rust panic that
  a golden's exit-code check might mask).
- **`codePointAt`/decode (PASS):** ASCII, 2-byte (`é`), 3-byte (`€`), 4-byte (`🎉`) codepoints
  decode to their correct Unicode scalar values; a mixed string's `codePoints.each` visits every
  character exactly once in order.
- **`split`/`replace` (PASS):** multi-match, no-match (whole string as one segment), delimiter at
  start/end, consecutive delimiters (empty segments), `replace` preserving surrounding content.
  **NEGATIVE:** empty delimiter, non-`String` delimiter/`from`/`to` → `ArgumentError`, caught by
  `.on(ArgumentError)` or uncaught-renders correctly.
  <br>**Wren cross-check (mandatory ✅):** the golden fixtures for `split`/`replace` should include
  at least one hand-computed example already worked out from the cited Wren source's own doc/test
  semantics (Wren `"a,b,,c".split(",")` → `["a", "b", "", "c"]`) so behavior parity with the
  precedent is provable, not assumed.
- **`trim*` (PASS):** default whitespace set, custom `chars` set, all-trimmed-away → `""`,
  no-trim-needed round-trip. **NEGATIVE:** non-`String` `chars` → `ArgumentError`.
- **`*(count)` (PASS):** `0` → `""`, `1` → unchanged, `3` → tripled. **NEGATIVE:** negative count,
  fractional count, non-`Number` count → `ArgumentError`.
- **`bytes`/`codePoints` (PASS):** `.size`/`.at(_)`/`.each` agree with `rawByteCount`/`rawByteAt`/
  manual codepoint decode on a mixed-width string; both are usable standalone (no `Iterable`
  dependency asserted).
- **`System.write`/`writeObject_` (PASS):** `write(x)` returns `x` (pass-through); output has no
  trailing newline (byte-exact `.expected`, no implicit `\n`); a class overriding `toString`
  renders via the override, not the native default. **`[invalid toString]` (PASS, the robustness
  win):** a class whose `toString` returns a `Number` (or any non-`String`) → `write` emits
  `[invalid toString]` literally, does not crash. **Divergence-documented (PASS, the Rubric
  golden):** `System.print(aMap)` and `System.write(aMap)` produce *different* output today —
  pinned deliberately, not accidentally.
- **Floor audit (INVARIANT):** `floor_census_matches_installed_bindings` passes at count 117.
- **Full-corpus regression (mandatory gate, not a new fixture):** re-run the **entire** existing
  `tests/lang` corpus after step 6 and confirm **zero** diffs — this unit must not have touched
  `print(_)`'s behavior at all; any diff means the design boundary in §2.5 was violated.

## 7. Decisions flagged (flag, don't pick — none of these are open-question-gated, but each is a real fork the architect made a call on; surface for confirmation)
| ID | Decision | Options | Architect recommendation |
|---|---|---|---|
| **DEC-STR-A** | **`System.print(_)` funnel unification.** Should this unit also flip `print(_)` onto the message-dispatch `writeObject_` path? | **(A)** leave `print(_)` untouched, ship `write`/`writeObject_` as new selectors only (this plan's choice); **(B)** flip `print(_)` too, and port `Map`/`Set`/`Tuple`/`Range` `toString` overrides in the same unit to avoid regressing them. | **(A)** — confirmed 379-file blast radius + a second confirmed wording divergence (`<X>` vs `<X instance>`) makes (B) a much larger, riskier unit than "String protocol." Ship (A) now; (B) is a clean, separable follow-on once the `toString`-message gap is closed on its own. |
| **DEC-STR-B** | **`indexOf`/`contains`-family nativization.** Wren nativizes `indexOf`/`contains`/`startsWith`/`endsWith` for `memcmp` speed. Keep them native or `.ph`-derived? | **(A)** `.ph`-derived over `rawByteAt` (this plan's choice, §2.0); **(B)** nativize `indexOf` (still skip `contains`/`startsWith`/`endsWith`, derivable over it). | **(A)** — ADR-0019's stated philosophy (smaller floor, accept slower hot paths, fund an IC/JIT later if it matters) overrides raw Wren fidelity. Revisit only with a profiling-driven case, as its own ADR-0019 amendment. |
| **DEC-STR-C** | **`ArgumentError` construction spelling.** error-handling.md §1's example is the bare-call `ArgumentError("msg")`; the landed surface only supports `ArgumentError.new("msg")` (DEC-ERR-B resolved (B), confirmed §0). | **(A)** use `.new(_)` throughout this unit, note the spec-example/landed-surface gap as pre-existing (not this unit's to fix); **(B)** generalize the bare-call construction sugar here to also cover `ArgumentError`. | **(A)** — generalizing `Name(args)` construction sugar is a `phalcom-ast`/`compiler/lib.rs` change explicitly named out of scope by U-ERR's own DEC-ERR-B follow-on note; doing it here would smuggle a spine-adjacent change into a library unit. |

## 8. Must-not-preclude check ([deferred-work.md](../../../spec/current/deferred-work.md), ADR-0048)
- **`Iterable` root (ADR-0048, U-ITERABLE):** actively *served*, not precluded —
  `StringByteSequence`/`StringCodePointSequence` are shaped so `extends Iterable` + renaming
  `nextCursor_`→`iterate(_)` is a **pure addition** later (§2.4); the byte sequence's cursor is
  already a dense `0..size` index (fits the *generic* `Iterable#iterate` `List` will also use),
  the codepoint sequence's cursor already overrides its own stepper (exactly ADR-0048's named
  "non-`0..size`-index" case). No rework needed when U-ITERABLE lands.
- **The native-`RuntimeError`-reification unit (Arity/Type/RangeError/ZeroDivision → surface
  classes, named "later unit" by U-ERR):** actively *served* — this unit's `ArgumentError` is
  the exact class that reification should bind to for arity/argument-shaped native errors,
  established here first rather than left for that unit to invent ad hoc (§ Rubric).
- **`String#at(_)` character indexing (catalog-delta.md's own "indexing → Option" gap):** not
  precluded — `codePointAt(_)`/`rawSlice(_,_)` are exactly the primitives a future character-index
  API would build on (character-position → byte-offset walk over the same `leadByteLen_` stepper);
  `RangeError` is deliberately reserved for that unit to introduce (§2.2), following this unit's
  `extends Error {}` pattern.
- **A future `String.fromCodePoint`/`fromByte` constructor:** not precluded — would need its own
  small native encode primitive (UTF-8 encode is not derivable in `.ph` for the same allocation
  reason `rawSlice` isn't), cleanly additive to the floor this unit already established.
- **The `print`/`writeObject_` funnel unification (§2.5's named follow-on):** not precluded —
  `writeObject_` already exists in the exact shape `print(_)` would need to route through; the
  blocking precondition (`Map`/`Set`/`Tuple`/`Range`/instance `toString` parity) is named, not
  silently dropped, and is itself a small, independent, well-scoped unit.
- **`contains(_)`/`startsWith(_)`/`endsWith(_)` (named-but-unshipped Wren parity, §0):** not
  precluded — trivial one-line `.ph` derivations over `indexOf` once wanted; deliberately not
  added preemptively to keep this unit's surface exactly the locked scope.

## 9. Return contract (report to `phalcom-reviewer`)
The exact native floor delta (+4: `String::rawByteCount`/`rawByteAt(_)`/`rawSlice(_,_)`,
`System::rawWrite(_)`; census 113→117) + the ADR-0049 text and its ratification status ·
confirmation the char-boundary-panic guard on `rawSlice` is a checked `RuntimeError::Type`, never
a Rust panic (with the negative-path test proving it) · the `codePointAt`/`leadByteLen_`
arithmetic-decode bodies (no bitwise ops) verified against a 1/2/3/4-byte-sequence golden ·
`ArgumentError`'s inherited-ctor golden (confirming the U-INH resolution claim rather than just
citing memory) · the six `split`/`replace`/`trim*`/`*` argument-guard sites and their Wren-parity
goldens · the `bytes`/`codePoints` standalone classes + their `Iterable`-ready shape · the
`System.write`/`writeObject_` funnel + the `[invalid toString]` golden + the **deliberately
pinned** `print`-vs-`write` divergence golden (Rubric) · confirmation of the **zero-diff full
corpus re-run** after step 6 (the load-bearing proof `print(_)` was never touched) · how
DEC-STR-A/B/C were resolved · the `strings` corpus label + MANIFEST bump · the `floor-census.md`/
`core-classes.md`/`deferred-work.md`/`invariants.rs` updates landed in lockstep with the ADR ·
`verify.sh` + `cargo doc` tails · confirmation `universe.rs`/`core.ph` were free of concurrent
editors at the time each slice landed (§3.1).
