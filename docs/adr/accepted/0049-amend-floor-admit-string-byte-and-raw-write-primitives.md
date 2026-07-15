# 49. Amend the floor: admit String byte/slice accessors + raw stdout write

- Status: **Accepted — authoritative on naming again** (2026-07-15; ADR-0062, which had superseded the names below with `raw*`, is **Retired**)
- Date: 2026-07-13
- **Naming note (2026-07-15, resolved):** this ADR specified the U-NATIVE-MARKER
  trailing-`_` convention (`byteCount_`/`byteAt_(_)`/`slice_(_,_)`/`write_(_)`). The
  U-STRING unit shipped `raw*`-prefixed names instead, and ADR-0062 was written to
  bless that deviation. **The user re-ruled trailing-`_` on 2026-07-15; the rename is
  applied and ADR-0062 is Retired.** The names in this ADR are the ones in the tree —
  verified green (26/26 test binaries, `R-INV-0.1` floor census included).

## Context

Phalcom's `String` is near-empty (`+`, `hash`, `new`). The Wren-modelled protocol —
`split`/`replace`/`trim*`/`*`/`indexOf`/`codePoints`/`bytes` — is almost entirely
derivable in `.ph`, **except** it cannot see the underlying UTF-8 bytes at all: `.ph` has
no bitwise ops and no code-unit accessor, so a string cannot be indexed, sliced, or
byte-length-measured without a native floor. Likewise `System` has exactly one I/O
primitive (`print`, which appends a newline); a `write` funnel that emits without a newline
has no raw act to build on.

Per ADR-0019's derivability test, a capability earns floor status only if it *cannot* be
expressed in `.ph` over lower `.ph`. Four capabilities fail that test; the rest of the
String protocol passes it and stays `.ph`.

## Decision

Admit **four** native primitives. They take the trailing-`_` native/private marker
(Wren convention, adopted repo-wide by U-NATIVE-MARKER) from the start, so the rename unit
does **not** touch them.

| Selector | Class (receiver) | Arity | Why it is irreducible |
|---|---|---|---|
| `byteCount_` | `String` | getter | UTF-8 byte length; `.ph` cannot read the backing bytes |
| `byteAt_(_)` | `String` | 1 | raw byte at an index; **total** — a raw `Number`, or `None` out of bounds (mirrors `List`'s raw-at shape); no `.ph` code-unit accessor exists |
| `slice_(_, _)` | `String` | 2 | allocates a new `StringObject` from a byte range — the one Wren-cited irreducible case; **must** validate `str::is_char_boundary` on both ends and **never panic** on a mid-code-point split (returns a defined error) |
| `write_(_)` | `System class` | 1 (static) | raw stdout write, no newline, no formatting — the literal I/O act the `write`/`writeObject_` funnel is built over |

Everything else in the String protocol is **`.ph`-derived** over these four plus existing
`Number` arithmetic (`%`/`/` decode UTF-8 lead-byte lengths without any bitwise op —
confirmed none exist on HEAD): `size`, `at(_)→Option`, `codePointAt(_)`, `indexOf(_,_)`,
`split(_)`, `replace(_,_)`, `trim`/`trimStart`/`trimEnd`(+char-set), `*(count)`, `isEmpty`,
and the `bytes`/`codePoints` sub-iterables. This is a **deliberate deviation from Wren**,
which nativizes `codePointAt_`/`indexOf` for speed; Phalcom keeps them `.ph` because they
are derivable, and revisits only if profiling demands it.

**Floor census delta: +4 bindings** (113 → 117). Machine-checked by
`floor_census_matches_installed_bindings` — U-STRING updates the census in lockstep.

## Consequences

- **`String` gains a real protocol** with a minimal, irreducible native base — the hybrid
  pattern (native floor, `.ph` control) held to its tightest form.
- **`System` gets a write funnel** (`write(_)`/`writeObject_(_)`, both `.ph` over `write_`)
  that guards `x.toString isA String` and emits `"[invalid toString]"` otherwise — Wren's
  robustness win. **`System.print(_)` is left native and untouched** (see the spec note; a
  message-dispatch flip would regress the corpus because `Map`/`Set`/`Tuple`/`Range` carry
  no `toString` *message* today, only `Value::to_string`'s native render path).
- **UTF-8 safety is a floor obligation:** `byteAt_`/`slice_` convert every out-of-range or
  mid-code-point input into a **defined** `Option`/error, never a Rust panic or UB
  (security.md: malformed input → defined error).
- **No `Value`/representation change**; `String` stays its existing `Object::Str` arm.

## Alternatives considered

- **Nativize the whole Wren String protocol** (`codePointAt_`, `indexOf`, `split` in Rust).
  Rejected — fails the derivability test; bloats the floor for speed that is not yet
  demanded. The four admitted here are the irreducible minimum.
- **A single `bytes` native list accessor** instead of `byteCount_`+`byteAt_`. Rejected —
  materializes a whole byte `List` per query; the two lazy accessors keep byte iteration
  allocation-free and mirror `List`'s existing raw shape.
- **Reuse `print` for `write`.** Rejected — `print` appends a newline; the funnel needs a
  newline-free raw act, and conflating them makes `write` un-derivable.
