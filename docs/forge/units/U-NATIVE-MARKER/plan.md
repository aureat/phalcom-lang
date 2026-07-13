# U-NATIVE-MARKER — Work order: native/private primitive marker `raw*` → trailing `_`

_Self-contained mechanical rename unit for **one** implementer. Adopts the Wren convention
(`wren_core.wren`: a trailing `_` on a selector marks a native/private primitive —
`byteCount_`, `codePointAt_`, `iterateByte_`) in place of Phalcom's current `raw`-**prefix**.
No behavior change; pure renaming across the `.ph` call sites **and** the Rust-side registered
selector strings. **Green gate:** `./scripts/verify.sh` exits 0 + `cargo doc --workspace --no-deps`
clean. **New governing ADR: none** — a naming convention, not a semantic change; note it in
`lexical-structure.md` conventions. Grounded in a user ruling (2026-07-13)._

> **Independence.** Orthogonal to U-ITERABLE / U-SEQ / U-STRING. It touches the *names* of the
> native collection primitives, not the iteration protocol. **Serialize** against any unit that
> edits `core.ph` or `primitive/*.rs` (U-ITERABLE reopens the collection classes; land one, then
> rebase the other) — do not run in the same parallel wave.

---

## 1. Mission (one sentence)
Rename every native/private core primitive from the `raw`**prefix** form to the trailing-`_`
**suffix** form — `rawXxx` → `xxx_` (first letter lowercased) — across both the `.ph` call sites
and the Rust selector strings they are registered under, with **zero behavior change** and a green
verify.

## 2. Preconditions (verify on actual HEAD — do not assume)
- **THE GATE — lexer admits a trailing `_` in identifiers/selectors.** Confirm `phalcom-ast/src/lexer.rs`
  (and the selector-encoder) accept an identifier that **ends** in `_` (e.g. `length_`, `at_`), and
  that the parser does not treat a trailing `_` specially. If a trailing `_` is rejected or mangled,
  **this whole unit blocks** — stop and report; do not work around it. (Wren allows it; phalcom's
  identifier rule is almost certainly `[A-Za-z_][A-Za-z0-9_]*`, which admits it — but *verify* before
  the first rename.)
- Baseline `./scripts/verify.sh` green before the first edit.
- **No concurrent `core.ph` / `primitive/*.rs` editor.** U-ITERABLE and the live U-CORE track both edit
  these — confirm the slot is free (`graphify affected "core.ph"`, git status) and serialize.

## 3. The rename (the exact mapping)

Rule: **drop `raw`, lowercase the next letter, append `_`.** The trailing `_` is now the
native/private marker; the leading-`raw` disambiguation from the public selector is replaced by the
suffix, so `at` (public) and `at_` (native) coexist cleanly.

| Class | Current (`raw*`) | New (`*_`) |
|---|---|---|
| `List` | `rawLength` · `rawAt` · `rawPush` · `rawSet` | `length_` · `at_` · `push_` · `set_` |
| `Map` | `rawSize` · `rawGet` · `rawPut` · `rawHas` · `rawRemove` · `rawKeyAt` · `rawValueAt` | `size_` · `get_` · `put_` · `has_` · `remove_` · `keyAt_` · `valueAt_` |
| `Set` | `rawSize` · `rawAdd` · `rawHas` · `rawRemove` · `rawAt` | `size_` · `add_` · `has_` · `remove_` · `at_` |
| `Tuple` | `rawSize` · `rawAt` | `size_` · `at_` |
| `Range` | `rawStart` · `rawEnd` · `rawInclusive` | `start_` · `end_` · `inclusive_` |

> **Verify the list on HEAD** — `rg -n "raw[A-Z]" phalcom-core/core/core.ph phalcom-core/src` — and
> extend the table with anything not captured above (e.g. a `Tuple.fromList` construction helper is
> **not** a `raw*` primitive and is **out of scope** — do not rename it). String native primitives are
> owned by **U-STRING**, which already adopts the `_` suffix form for any new ones — nothing to rename
> here if String has no `raw*` primitives yet.

## 4. Confirmed write-set (re-validate with `rg`/`graphify affected` on HEAD)
| File | Why |
|---|---|
| `phalcom-core/core/core.ph` | every `.ph` call site of the primitives (the `List`/`Map`/`Set`/`Tuple`/`Range` bodies) |
| `phalcom-core/src/primitive/*.rs` | the **registered selector strings** the primitives are installed under (list/map/set/tuple/range primitive modules) — the Rust side must rename in lockstep or dispatch misses |
| `phalcom-core/src/universe.rs` | any bootstrap interning / installation of those selector symbols |
| `phalcom-core/src/bin/phalcom/disasm.rs` | only if it string-matches any `raw*` selector (grep) |
| `phalcom-core/tests/lang/**` | any golden `.ph` that calls a `raw*` primitive directly (likely none — they are internal) |
| `docs/spec/v0.2/**`, `docs/adr/**` | prose references to `raw*` primitive names (grep; update to the `_` form) |
| `docs/spec/v0.2/lexical-structure.md` | **add** a one-line convention note: trailing `_` marks a native/private primitive selector (Wren-style) |

**Deliberately NOT in scope:** any logic, opcode, `Value`/`heap` change; the iteration protocol; the
`Tuple.fromList` helper; String primitives (U-STRING).

## 5. Build order
1. **Gate check** — confirm trailing-`_` lexes (§2). If not, stop + report.
2. **Rust + `.ph` in one lockstep pass, per class** — rename `List`'s four primitives in
   `primitive/*.rs` **and** `core.ph` together, `cargo build`, verify. Then `Map`, then `Set`, then
   `Tuple`, then `Range`. Each class is an independently-green commit (a half-renamed primitive is a
   dispatch miss — never commit a split rename).
3. **Docs + convention note** — sweep `docs/` prose; add the `lexical-structure.md` line.
4. **Full verify** — `./scripts/verify.sh` green; `cargo doc` clean; grep proves **zero** remaining
   `raw[A-Z]` outside intentional exclusions.

## 6. Mandatory rules
- **Green gate:** `./scripts/verify.sh` exits 0; no new clippy; no `unsafe`.
- **Lockstep:** the Rust registered selector and the `.ph` caller rename in the **same commit** — the
  registered string and the call site must never disagree (that is a silent `doesNotUnderstand`).
- **Docs:** update any `///`/`//!` that names a `raw*` primitive.

## 7. Test strategy (the green gate must assert)
- **No behavior change:** the full existing `tests/lang` corpus stays green byte-for-byte (the rename is
  invisible at the surface — no public selector changed).
- **Grep assertion:** `rg "raw[A-Z]"` over `core.ph` + `phalcom-core/src` returns only the documented
  exclusions (ideally nothing).
- **One targeted golden** (optional): a `.ph` that exercises a collection round-trip
  (`List.new().add(1).size`, `Map`/`Set`/`Tuple`/`Range` basics) passes unchanged, proving the renamed
  primitives still dispatch.

## 8. Decisions flagged
| ID | Decision | Recommendation |
|---|---|---|
| **DEC-NM-A** | Lowercase-first-letter (`rawKeyAt`→`keyAt_`) vs keep camel (`rawKeyAt`→`KeyAt_`)? | Lowercase — selectors are lowerCamel everywhere else; `keyAt_` matches `keyAt`-style naming. |
| **DEC-NM-B** | Reviewer on/off? | **Verify-gate mandatory**; reviewer optional — the risk is a split (dispatch-miss) rename, caught by the green corpus, not by logic review. Run the full corpus, not a subset. |

## 9. Must-not-preclude check
- **U-STRING native floor:** not precluded — U-STRING adopts the same `_` suffix for any new String
  primitive; this unit and it share the convention, not the files (String primitives are separate).
- **NaN-boxing / niche work:** untouched — this is a name change only.
- **Future native primitives:** the convention is now established and documented, so later units add
  `xxx_` directly.

## 10. Return contract
The confirmed trailing-`_` lexer result (the gate) · the final `raw*`→`*_` mapping table as applied
(with any HEAD additions) · confirmation the Rust selector strings and `.ph` call sites were renamed in
lockstep · the `rg "raw[A-Z]"` clean proof · `verify.sh` + `cargo doc` tails · the
`lexical-structure.md` convention line added.
