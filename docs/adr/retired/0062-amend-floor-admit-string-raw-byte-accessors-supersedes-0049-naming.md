# ADR-0062: Amend floor — admit `String` raw byte accessors + `System.rawWrite(_)`

**Status:** **Retired 2026-07-15 — the user re-ruled the trailing-`_` convention; `raw*` is removed.**

> This ADR existed for exactly one purpose: to bless the `raw*` names U-STRING shipped
> in place of the trailing-`_` convention [ADR-0049](../accepted/0049-amend-floor-admit-string-byte-and-raw-write-primitives.md)
> had specified. That deviation is now reverted — `rawByteCount`/`rawByteAt(_)`/
> `rawSlice(_,_)`/`rawWrite(_)` are renamed to **`byteCount_`/`byteAt_(_)`/`slice_(_,_)`/
> `write_(_)`**, exactly as ADR-0049 asked. **ADR-0049 is authoritative again**; nothing
> in this document survives except its history.
>
> The floor amendment itself (4 bindings: byte-level `String` access + raw stdout
> write) was never in question and is unaffected — it is ADR-0049's decision, not this
> one's. This ADR only ever changed spellings.
>
> **Why it happened, worth recording:** the user ruled trailing-`_` on 2026-07-13
> (see [U-NATIVE-MARKER](../../forge/units/U-NATIVE-MARKER/plan.md)); U-STRING shipped
> `raw*` anyway; this ADR was then written on 2026-07-14 to make the record match the
> code. That is backwards — a ruling should move the code, not the other way round. The
> collections half of the same rename (`length_`, `at_`, `keyAt_`, `size_` …) had
> *already* landed correctly, so the tree carried both conventions at once for a day.

Accepted (original text below, kept for history)

**Supersedes:** [ADR-0049](0049-amend-floor-admit-string-byte-and-raw-write-primitives.md)'s
selector names. Both ADRs approve the same 4-binding amendment (byte-level `String`
access + raw stdout write); ADR-0049 specified the U-NATIVE-MARKER trailing-`_`
convention (`byteCount_`/`byteAt_(_)`/`slice_(_,_)`/`write_(_)`) but the U-STRING unit
shipped the `raw*`-prefixed names below instead (`rawByteCount`/`rawByteAt(_)`/
`rawSlice(_,_)`/`rawWrite(_)`) — confirmed live in `primitive/string.rs`,
`primitive/system.rs`, and every `core.ph` call site. This ADR is the accurate record
of what is actually bound; ADR-0049 is retained for history and marked superseded on
naming only (the decision to admit the 4 bindings itself was never in question). The
`_`-suffix rename ADR-0049 called for is **not** applied — see "Alternatives
considered" below.

**Scope:** U-STRING (docs/forge/units/U-STRING/plan.md)

**Ratified by:** Commit 0bae56d (U-STRING step 1: ArgumentError) + this unit's binding and test.

## Summary

ADR-0019 froze the VM-blessed primitive floor at 73 bindings, with the explicit expectation that `String`'s protocol gap would be "the first place a future amendment is likely." This ADR realizes that prediction by admitting exactly 4 irreducible native bindings:

1. **`String::rawByteCount`** (instance getter, 0-arity) — byte length of UTF-8 buffer.
2. **`String::rawByteAt(_)`** (instance method, 1-arity) — raw byte read, returns `Number` or `None`.
3. **`String::rawSlice(_,_)`** (instance method, 2-arity) — substring extraction by byte range `[start, end)`, validates UTF-8 boundaries.
4. **`System::rawWrite(_)`** (static method, 1-arity) — raw stdout write of an already-formed string.

**Floor delta:** +4 bindings. Census 113 → 117 (pre-amendment count; post-drift 121 → 125 in current tree).

## Motivation

The U-STRING unit ports Wren's string protocol to Phalcom via a `.ph` library layer over a minimal native floor. The 4 bindings are justified per ADR-0019's derivability test:

- **Byte-level access** (`rawByteCount`, `rawByteAt`) — `.ph` code cannot observe the buffer at all; these are primitive observations.
- **Substring allocation** (`rawSlice`) — the only way to produce a new `String` from computed byte offsets; the single `.ph` constructor (`String.new(_)`) stringifies an arbitrary value, not construct-from-bytes.
- **Raw I/O** (`System.rawWrite`) — the irreducible literal I/O act (ADR-0019 rule 6); the `System.write`/`writeObject_` `.ph` funnel rides on this seam.

All other operations—`split`, `replace`, `trim`, indexing, substring search (`indexOf`), codepoint iteration—are **derived in `.ph`** over these 4 primitives, maintaining ADR-0019's philosophy: "a smaller native surface is more auditable."

## Consequences

- **`String` and `System` class definitions reopen** in `phalcom-core/core/core.ph` to add derivations (§2.1–§2.5 of the U-STRING plan).
- **`ArgumentError` exception class is introduced** as the boundary-guard exception for library argument validation (error-handling.md §1).
- **`StringByteSequence` and `StringCodePointSequence`** are introduced as sub-accessor classes for per-byte and per-codepoint iteration (shaped to extend `Iterable` when U-ITERABLE lands).
- **`floor-census.md` § 2.5 and § 2.11 are updated** to reflect the 4 new bindings.
- **`docs/spec/v0.2/core/core-classes.md`** row for `String` is updated to mark it as "◐ partially implemented" and document the new accessors.
- **`docs/spec/v0.2/deferred-work.md`** is updated to mark U-STRING as landed and add follow-on rows for `print(_)/writeObject_` funnel unification, character indexing, and derived search methods.
- **Invariant R-INV-0.1** (floor census audit, `tests/invariants.rs`) is updated to assert 125 bindings (post-drift).

## Notes

- The char-boundary validation in `rawSlice` uses Rust's `str::is_char_boundary` to prevent panics on misaligned slices; malformed input → `RuntimeError::Type`, never a panic.
- `codePointAt(_)` and `leadByteLen_(_)` are derived in `.ph` using only numeric range tests and modulo (no bitwise operators, per ADR-0024's deferral).
- `indexOf(_)` is `.ph`-derived over `rawByteAt`, accepting O(n·m) cost per ADR-0019's stated trade.
- The `System.print(_)` pathway is **unchanged** — it continues to bypass message dispatch and use native `Value::to_string`. The new `System.write(_)`/`writeObject_(_)` funnel is **additive only**, addressing ADR-0019's anticipation that a future unit would unify these paths (blocked here by the `Map`/`Set`/`Tuple`/`Range`/instance `toString` wording gap, per the plan's Rubric).

## Alternatives considered (naming, vs. ADR-0049)

- **Rename the shipped `raw*` bindings to ADR-0049's `_`-suffix convention post hoc.**
  Rejected for this pass — U-STRING is landed, green (44 lang tests + the floor-census
  invariant), and referenced by name throughout `core.ph`/`primitive/string.rs`/
  `primitive/system.rs`/this test corpus. A rename is real, if mechanical, work (touch
  every call site, re-verify) rather than a docs fix; tracked as its own follow-up
  under U-NATIVE-MARKER's scope instead of bundled into this consolidation pass.

