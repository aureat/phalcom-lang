# U-STD — Implementation Specification (supersedes U-STD-plan.md on conflict)

_Grounded against actual HEAD as of commit `4e2ec73` (U10 landed, green;
`git log --oneline -1` = "U10: non-local return via `return` inside blocks").
This document exists because **`U-STD-plan.md` was written on 2026-07-11, before
U8, U9, U10, U-CORE-0, and U-CORE-2 landed** — its scope inventory (D1–D5) is
substantially stale: most of what it lists as "to build" is either **already
shipped by U8 / earlier units**, or has been **re-assigned to a finer-grained
`U-CORE-N` taxonomy** that did not exist when the plan was written. The single
authoritative, HEAD-grounded inventory is now
[`docs/spec/core/catalog-delta.md`](../spec/core/catalog-delta.md) +
[`docs/spec/core/floor-census.md`](../spec/core/floor-census.md) (U-CORE-0,
2026-07-12). **Where this document and `U-STD-plan.md` disagree, follow this
document.** Where this document is silent, `U-STD-plan.md` still governs
(guardrails, mandatory rules, return-contract shape)._

Written for a **medium-effort implementer** — but note **§0.1 is a hard
BLOCKED-ON-DECISION**: the unit's *scope* is contested between the forge index
and the newest spec artifacts, and an implementer must not proceed on the
contested surface until it is ruled. The unblocked residual (§2.6) is safe to
build today. If you hit a fact that contradicts this doc, STOP and report the
conflict rather than guessing.

---

## 0. Corrections to `U-STD-plan.md`

### 0.1 **BLOCKED-ON-DECISION — U-STD's scope has been re-carved out from under the plan.**

`U-STD-plan.md`'s mission (§0) and the forge index
([`PHASE2-INDEX.md`](PHASE2-INDEX.md) L37) both say U-STD =
"grow `core.ph` base-class method surface (**Object/Number/String/Symbol/System**)".
But the newer, HEAD-grounded U-CORE-0 census (2026-07-12,
`docs/spec/core/`) **re-assigns almost all of that** to a set of `U-CORE-N`
units that live **only in `docs/spec/core/`** — `grep -rl "U-CORE" docs/forge/`
returns nothing, so the forge scheduling docs and the spec taxonomy openly
disagree. Per [`catalog-delta.md`](../spec/core/catalog-delta.md) §2 and
[`docs/spec/core/HANDOFF.md`](../spec/core/HANDOFF.md):

| Plan says U-STD owns | Reality at HEAD | Newest owner |
|---|---|---|
| `Object`: `class` `==` `!=` `toString` `perform` `respondsTo` `doesNotUnderstand` | **already primitives** (U8 + earlier — `object.rs`, `install_primitives` L227-243) | done |
| `Object`: `isA(_)`, `hash` | **absent** | **U-CORE-1** (hash gated on Q1, an ADR-0019 amendment) |
| `Number`: arithmetic/comparison | **already primitives** (`number.rs`, L266-275) | done |
| `Number`/`String`/`Symbol`/`Bool`/`Option` **value `toString`** | **absent** (§4.4 — inherits `Object#toString` → class name) | **U-CORE-4** |
| `String`: `+`, `new` | **already primitives** (`string.rs`) | done |
| `String`: length, indexing→`Option`, comparison, interpolation, `toSymbol`/`toNumber` | **absent** | **U-CORE-4** |
| `Symbol`: `toString`, `new(_)` | **already primitives** (`symbol.rs`) | done |
| `System`: `print(_)` | **already a primitive + 4 passing fixtures** (`system.rs`; `system()` label is NOT `#[ignore]`d) | done |
| `System`: `clock`/`gc`/scheduler | **absent** | **deferred system unit** (out of core scope, catalog-delta §2.5) |

Under **either** reading, the plan's D1–D5 are ~90% already-landed or
reassigned. The genuinely-remaining, unblocked, additive `.ph` work the newest
artifact **explicitly keeps on U-STD** is a *different* surface than the plan's
title suggests (catalog-delta §2.2/§2.4, DEFERRED.md #18/#20/#25):

- **`Option` transform/extract combinators** — `map(_)`, `flatMap(_)`,
  `filter(_)`, `ifSome(_)`, `unwrapOr(_)` — layered over `match` in `core.ph`.
- **`List` combinators** — `map(_)`, `reduce(...)`, `filter(_)`, `includes(_)`,
  `isEmpty`, and `at(_, put:)` (wrap `rawSet`) — layered over the frozen floor.
- **New collection classes** `Tuple`/`Map`/`Set`/`Range` — but per ADR-0020 each
  is its own unit, and `Map`/`Set` **block on `Object#hash`** (U-CORE-1, Q1), so
  they are **not** buildable in this unit regardless of the ruling.

**Options:**
- **(A)** Keep the plan's literal scope (Object/Number/String/Symbol/System).
  Result: the unit is nearly a no-op — everything is done except `isA`/`hash`
  and value-type `toString`, which the newest artifact says are U-CORE-1 /
  U-CORE-4, and would *collide* with those units on `core.ph`.
- **(B) [recommended]** Adopt the catalog-delta re-carving: U-STD's residual =
  the **Option + List combinator layer** (§2.6). This is (a) additive `.ph` over
  the existing frozen floor, (b) needs **zero new native primitives**, (c) needs
  **no blocked decision**, (d) is exactly what DEFERRED.md #20/#25 and
  `blocks/pending/blocks_argument_to_method.ph` are waiting on, and (e) is a
  tight, independently-verifiable green unit. Route `isA`/`hash` to U-CORE-1,
  value-type `toString` to U-CORE-4, collections to their own units.

**Do not pick this yourself.** The rest of this spec is written for **Option (B)**
(the buildable residual, §2.6/§3/§6) but fully documents the contested
Object/Number/String/Symbol surface (§2.1–2.5) so that if the ruling is (A) the
implementer still has the exact delta.

### 0.2 The plan's "avoid `primitive/object.rs` — U8 owns it in the same wave" is obsolete.
U8 **landed** (`b99ad22`/`806c9ea`, 2026-07-12). There is no concurrent U8. The
Wave-F collision the plan worried about (`primitive/object.rs` — U-STD ✕ U8,
PHASE2-INDEX §3) is **gone**. It is now safe to touch `object.rs` — but under
Option (B) you have no reason to (the residual is pure `.ph`).

### 0.3 The plan's D5 "fix the `print` stub" and "un-ignore the `system` label" are stale.
`system_class_print` is a real, working primitive (`system.rs:13`) returning the
`None` singleton, and there are **four passing** `system` fixtures
(`system_class`, `system_print_bool/number/string`). In `tests/lang.rs`,
`fn system()` (L152) is **already active** (`check_pass`, no `#[ignore]`); only
`fn system_pending()` (L157) is ignored, and it pins the *deferred* surface
(`clock`/`gc`/`readLine`), which is **out of core scope** (catalog-delta §2.5).
There is nothing for U-STD to un-ignore or fix on `System`.

### 0.4 The map/reduce/filter boundary — confirmed present, but the plan MIS-STATES which side U-STD is on.
The parent's brief references a `core.ph:50` comment "do not add map/reduce/
filter/literal syntax". **The line drifted** (U-CORE-2 inserted the `Option`
combinators, pushing the `List` block down): at HEAD the comment is
**`core.ph:71-73`**, inside the `List`-block header: _"…along with
`map`/`reduce`/`filter`/literal syntax. Do not add those bodies to this
skeleton."_ **The substance is confirmed, but read it carefully:** that is
**U-LIST telling itself not to build them, deferring them TO U-STD** — it is
**not** a prohibition on U-STD. Per catalog-delta §2.4 and DEFERRED.md #20/#25,
`List.map`/`reduce`/`filter` **are U-STD's job**. So:
- U-STD **MAY and (under Option B) SHOULD** add `List.map`/`reduce`/`filter`.
  When it does, it must also **update that `core.ph` comment** (the "do not add"
  line becomes false).
- U-STD **MUST NOT** add **list-literal syntax** `[a, b, c]` — that needs a new
  ADR + parser work and is deferred to a collections/lexer unit (DEFERRED.md #6).
- The parent's phrasing ("confirm U-STD does NOT add List.map/reduce/filter")
  reflects the *plan's* deferral (plan §6 puts **all** collections out of
  scope), which directly **contradicts** the newer catalog-delta. This is part
  of the §0.1 scope conflict; flag it, don't silently reconcile.

### 0.5 The plan defers ALL collections (§6); the catalog-delta puts List-extras IN U-STD.
`U-STD-plan.md` §6 lists `List`/`Map`/`Set`/`Tuple`/`Range` as entirely
deferred. That was written before U-LIST landed. `List` now exists with a live
`.ph` protocol (`size`/`at`/`add`/`each`), so **List combinators are buildable
today** and are assigned to U-STD (§0.4). `Map`/`Set`/`Tuple`/`Range` remain
deferred here — `Map`/`Set` on the `hash` blocker, all four needing literal
syntax + per-class storage decisions.

### 0.6 Stale reference: `tests/lang.rs:104` and `MANIFEST.md` still say "printString" is U-STD's.
The `absence` label is `#[ignore]`d with reason _"prettier printString + Some(x)
sugar are U-STD"_ (`lang.rs:104`), and `MANIFEST.md` (L102-103) says the
`absence`/`bindings` pending cases pin "a pretty `None` printString and `Some(x)`
sugar that U-STD/later units deliver". **`printString` is forbidden** — BD-2 /
[ADR-0015](../adr/0015-object-default-tostring.md) explicitly rejected a
`printString` selector (plan §1 even repeats "Do not invent `printString`").
What those fixtures actually want is a **value-rendering `toString` on
`None`/`Some`** so `System.print(None)` shows a pretty form instead of
`<None instance>`. Per [`README.md`](../spec/core/README.md) L48 that
`None`/`Some` `toString` is **U-CORE-2's residue**, and general value-type
`toString` is **U-CORE-4** (§4.4) — **not U-STD**. Treat these stale strings as
documentation drift; do not chase `printString`, and do not un-ignore the
`absence` label in this unit.

---

## 1. Preconditions — already verified, do not re-check

- U1–U10 + U-LIST + U-CORE-2 are merged and green (HEAD `4e2ec73`).
  `./scripts/verify.sh` is green on `main` right now. Run it on your starting
  tree before the first edit to confirm your baseline.
- **`core.ph` is live.** U-LIST fixed the pre-existing bug where `core.ph` was
  registered but never executed; `VM::run_core_module()` now runs it at
  `VM::new` (STATE.md "U-LIST — LANDED"). So every method you add to a `core.ph`
  class reopen actually takes effect. `List.each` is confirmed live and used by
  real tests (U10's `blocks/blocks_non_local_return.ph`).
- **The frozen floor you build over (floor-census.md §2), all confirmed against
  `install_primitives` (`universe.rs:225-369`):**
  - `List` (`core.ph:75-94`): `size => self.rawLength`, `at(i)`, `add(v)`
    (returns `self`), `each(f)` (a `while`-loop calling `f.call(self.at(i))`).
    Native floor: `new()`, `rawLength`, `rawAt(_)`, `rawSet(_,_)` (installed but
    **unwrapped** — no `at(_,put:)` yet), `rawPush(_)`, and native `toString`
    (`list.rs:135`, renders `[e0, e1, …]` via `Value::to_string`).
  - `Option` (`core.ph:42-60`): `match(some:none:)` eliminator (native, on
    abstract `Option`, `universe.rs:328-335`); `Some.new(_)` (native,
    `some_new`); `ifNone(_)`/`orElse(_)`/`isSome`/`isNone` (`.ph`, over `match`,
    added by U-CORE-2). `None` is the shared singleton **value** global (there is
    deliberately **no `class None {}` reopen** — the `DefineGlobal`-clobber trap,
    DEFERRED.md #17).
  - `Block#call(_)` at arities 0–4, `Number#< + - * / % <= > >=`, `Bool` sacred
    selectors + `while` lowering — everything the combinators need. **No new
    native primitive is required for the §2.6 residual.**
- **The absence boundary is load-bearing and you must respect it:** unassigned
  slots / value-less positions surface as the `None` singleton via U6's helper,
  never the raw `Value::Nil` sentinel. New `.ph` combinators must return `None`
  (or `Some.new(x)`), never construct or expose `nil`.
- **No-truthiness (ADR-0021):** every `if`/`while`/`ifTrue` condition in your new
  `.ph` must be a real `Bool` (e.g. `i < self.size`, `x == y`). An `Option`- or
  other-typed condition is a hard error. `==`/`!=` are ordinary sends
  (`object_eq`/`object_neq`, identity/value via `value_eq`).
- **Isolation / working model:** every unit since U2 landed **in-tree on `main`,
  no worktree**, committing per green checkpoint (STATE.md). The plan's "isolated
  worktree" instruction is stale; confirm the convention with the orchestrator
  before starting (mirrors U9/U10 spec §1).

---

## 2. The real catalog delta (per class) + the recommended residual scope

This is the "real catalog delta" the brief asked for, re-derived against HEAD and
cross-checked with [`catalog-delta.md`](../spec/core/catalog-delta.md) and
[`floor-census.md`](../spec/core/floor-census.md). **✅ = live at HEAD ·
❌ = absent · → = owner if not U-STD.**

### 2.1 `Object` (object-model §8) — nothing for U-STD
`class` ✅ `==(_)` ✅ `!=(_)` ✅ `toString` ✅(default→class name) `name` ✅
`perform(_)`/`perform(_,_)` ✅ `respondsTo(_)` ✅ `doesNotUnderstand(_)` ✅
(all primitives, `object.rs` + `install_primitives` L227-243).
`isA(_)` ❌ · `hash` ❌ → **U-CORE-1**. `isA` is derivable in pure `.ph`
(walk `self.class` → `superclass` until the apex reads `None`, comparing with
`==`); `hash` is **not** derivable without a floor primitive → **Q1 /
ADR-0019 amendment** (BLOCKED — also blocks `Map`/`Set`).

### 2.2 `Number` (object-model §4, ADR-0005 flat f64) — nothing unblocked for U-STD
`+ - * / % < <= > >= negated` ✅, `new()`/`new(_)` ✅ (`number.rs`).
`toString` (value) ❌ → **U-CORE-4** (§4.4 — today `42.toString` returns
`"Number"`; the value path is a *separate* Rust `Value::to_string`, not the
`toString` message). Keep any numeric work written against the **abstract**
numeric protocol so open-Q2 (Int/Float split) is not foreclosed (plan §7 watch).

### 2.3 `String` (object-model §4) — nothing unblocked for U-STD
`+(_)` ✅ `new()`/`new(_)` ✅ (`string.rs`). length, indexing→`Option`,
comparison, interpolation, `toSymbol`/`toNumber`, **value `toString`** (returns
self) ❌ → **U-CORE-4**. (Note: `"foo".toString` currently returns `"String"`,
not `"foo"` — the same §4.4 gap.)

### 2.4 `Symbol` (object-model §4) — nothing for U-STD
`toString` ✅ (`symbol_tostring`, `symbol.rs:13`) `new(_)` ✅. `asString` /
interning-identity `==` semantics ❌ → **U-CORE-4**.

### 2.5 `System` (system.md) — nothing for U-STD; already done/deferred
`print(_)` ✅ (`system.rs:13`, 4 passing fixtures). `clock`/`now`/`gc`/`version`/
`readLine`/scheduler ❌ → **deferred system unit** (out of core scope). See §0.3.

### 2.6 **The residual U-STD surface (recommended, Option B) — all additive `.ph`, zero new primitives**

| Class | Method | Semantics | Derived over |
|---|---|---|---|
| `Option` | `map(f)` | `Some(v) → Some(f(v))`; `None → None` | `match` |
| `Option` | `flatMap(f)` | `Some(v) → f(v)` (already an `Option`); `None → None` | `match` |
| `Option` | `filter(pred)` | `Some(v) → Some(v)` if `pred(v)` else `None`; `None → None` | `match` |
| `Option` | `ifSome(f)` | runs `f(v)` for effect if `Some`, returns `self` (mirror of `ifNone`) | `match` |
| `Option` | `unwrapOr(default)` | `Some(v) → v`; `None → default` | `match` |
| `List` | `map(f)` | new `List` of `f(x)` for each element | `each`/`add`/`List.new` |
| `List` | `reduce(init, f)` | fold `f(acc, x)` from `init` (labeled or 2-arg form — pick the spelling the fixtures use; see §6) | `each` + `var` acc |
| `List` | `filter(pred)` | new `List` of elements where `pred(x)` | `each`/`add` + `if` |
| `List` | `includes(x)` | `Bool` — any element `== x` | `each` + `==` |
| `List` | `isEmpty` | `self.size == 0` | `size` + `==` |
| `List` | `at(i, put: v)` | wrap `rawSet(i, v)` (DEFERRED.md #18) | `rawSet` |

All are pure `.ph` added **additively** to the existing `class Option { … }` and
`class List { … }` reopens in `core.ph`. **No `primitive/*.rs` change is
needed for this residual** — delete the plan's write-set row for
`primitive/{number,string,symbol,system}.rs` under Option (B).

**Deferred out of this unit** (append to DEFERRED.md, do not build):
`Map`/`Set`/`Tuple`/`Range` classes (own units; `Map`/`Set` block on `hash`);
list-literal syntax `[a,b,c]` (needs ADR + parser); `Object#isA`/`hash` →
U-CORE-1; value-type `toString` (Number/String/Symbol/Bool/Option) → U-CORE-4;
`None`/`Some` pretty `toString` → U-CORE-2.

---

## 3. Confirmed write-set (Option B; re-grep line numbers before editing)

| File | Exact change |
|---|---|
| `phalcom-core/core/core.ph` | **Shared file — additive only.** Add the §2.6 `Option` combinators inside the existing `class Option { … }` block (`core.ph:42-60`, after `isNone`) and the §2.6 `List` combinators inside the existing `class List { … }` block (`core.ph:75-94`, after `each`). Each new method carries a spec-referencing `//` comment (values-and-absence.md §3.3 for `Option`; ADR-0020 / catalog-delta §2.4 for `List`). **Update the `List`-block header comment (`core.ph:71-73`)**: it currently says "Do not add `map`/`reduce`/`filter`" — that deferral is now being discharged; reword to note map/reduce/filter now exist and only **literal syntax** remains deferred. |
| `phalcom-core/tests/lang/{absence,list}/*` | `.ph`/`.expected` PASS fixtures pinning the new surface (data only; §6). `Option` combinators go under `absence/` (its spec anchor is values-and-absence.md); `List` combinators under `list/`. Match the existing header-comment convention (`// area:` / `// spec:` / `// status: PASS`). |
| `phalcom-core/tests/lang.rs` | Only if you add a *new* label directory (you should not — reuse `absence`/`list`). The `absence` label is currently `#[ignore]`d for an unrelated reason (§0.6); **do not un-ignore it** — instead put the new `Option`-combinator PASS cases in `list/` alongside a `List` case, or add a **new, active** `option`/`combinators` label with its own `check_pass` fn. Implementer's call; document it. Do **not** touch `system()`/`system_pending()`. |
| `docs/forge/DEFERRED.md` | Append entries (start at **#27**): `Map`/`Set`/`Tuple`/`Range` still deferred (+`hash` blocker); list-literal syntax; the §0.1 scope-taxonomy divergence (forge index vs `docs/spec/core/`) surfaced for the orchestrator; and close/annotate #25 (`blocks_argument_to_method.ph`) once `List.reduce` lands — rewrite that pending fixture off the real `reduce`. |

**Explicitly NOT touched (Option B):** any `primitive/*.rs` (no new native
backing needed), `universe.rs`, `vm.rs`, `bytecode.rs`, `phalcom-ast/*` (U-LEX
owns the parser; parallel-safe), and any kernel wiring (U2 tower, U6 `Option`
substrate, U11 `Bool`). If the ruling is Option (A), add
`primitive/{number,string,symbol}.rs` for the U-CORE-4 `toString` overrides and
`object.rs` for `isA`/`hash` — but that is U-CORE-1/4 work, not this unit's.

---

## 4. Build order (each step verifies green before the next)

1. **`Option` combinators** (`map`/`flatMap`/`filter`/`ifSome`/`unwrapOr`) in
   `core.ph`'s `Option` block + `absence/` (or new `option`) PASS fixtures. Verify green.
2. **`List` combinators** (`map`/`reduce`/`filter`/`includes`/`isEmpty`/
   `at(_,put:)`) in `core.ph`'s `List` block + `list/` PASS fixtures, and update
   the `List`-block header comment. Verify green.
3. **Discharge DEFERRED.md #25:** rewrite
   `blocks/pending/blocks_argument_to_method.ph` off the now-real `List.reduce`
   and promote it out of `pending/` (or leave it, with the note updated). Verify green.
4. **Golden sweep** — confirm `examples/core_new.ph`, `person2.ph`, and the
   `tests/fixtures/golden/*` corpus stay **byte-identical** (you only added
   methods; nothing existing should shift).

---

## 5. The one architectural hazard — the `toString`-message ⊗ `Value::to_string` split

**Read this before writing any combinator that stringifies.** There are **two
independent rendering paths** in the tree, and they disagree (catalog-delta §4.4,
DEFERRED.md #19):

- `Value::to_string(vm)` — a Rust method that renders **values** correctly
  (`42` → `"42"`, `"foo"` → `"foo"`). Used by `System.print` and by
  `list_to_string` (`list.rs:138`).
- The **`toString` message** — dispatched Phalcom-side; for `Number`/`String`/
  `Symbol`/`Bool`/`Option` it currently resolves to `Object#toString`, which
  returns the **class name** (`42.toString` → `"Number"`).

**Consequence for your fixtures and combinators:**
- **Safe:** printing a `List` result via `System.print(myList)` renders elements
  correctly (it goes through `list_to_string` → `Value::to_string`). So
  `System.print([1,2,3].map { x => x * 2 })` → `[2, 4, 6]` works.
- **Safe:** extracting an `Option` to a `Number`/`String` and printing that
  (e.g. `System.print(Some.new(5).map { v => v + 1 }.unwrapOr(0))` → `6`).
- **UNSAFE — do not do this:** writing any combinator body (or fixture) that
  calls `.toString` **on an element** and expects its value (e.g. building a
  `List.join` in `.ph` over `x.toString`). It will render class names. This is
  exactly the trap U-LIST hit (DEFERRED.md #19), which is why `List.toString`
  stayed native. **Anything that needs element-value stringification is BLOCKED
  on U-CORE-4's value-type `toString` overrides — sequence it after, do not
  build it here.** Keep the §2.6 residual free of element stringification (it is,
  by design — `map`/`filter`/`reduce`/`includes` never stringify).

---

## 6. Test strategy — concrete fixtures

Match the existing corpus convention (header comment, `.ph` + exact-stdout
`.expected`; `None` prints as `<None instance>`; a `List` prints as
`[e0, e1, …]`). Build lists with `List.new()` + chained `.add(_)` (no literals).

- **`Option.map`/`unwrapOr` (both arms):**
  `System.print(Some.new(5).map { v => v + 1 }.unwrapOr(0))` → `6`;
  `System.print(None.map { v => v + 1 }.unwrapOr(0))` → `0`.
- **`Option.filter`:** `Some.new(4).filter { v => v > 3 }.unwrapOr(-1)` → `4`;
  `Some.new(2).filter { v => v > 3 }.unwrapOr(-1)` → `-1` (filtered to `None`).
- **`Option.flatMap`:** `Some.new(5).flatMap { v => Some.new(v * 2) }.unwrapOr(0)` → `10`;
  confirm chaining `None` short-circuits (`None.flatMap { … }.unwrapOr(-1)` → `-1`).
- **`Option.ifSome` (effect + passthrough):** call `ifSome` with a block that
  `System.print`s the value, assert it fired for `Some` and did not for `None`,
  and that the return value is the receiver (chain another combinator after it).
- **`List.map` / `filter`:** build `[1,2,3]`, `System.print(l.map { x => x * 2 })`
  → `[2, 4, 6]`; `System.print(l.filter { x => x > 1 })` → `[2, 3]`.
- **`List.reduce`:** build `[1,2,3]`, reduce to the sum `6` (this is the exact
  case DEFERRED.md #25 / `blocks_argument_to_method.ph` is waiting on — reuse the
  same accumulator shape). **Pin the selector spelling** you choose (`reduce(init){acc,x => …}`
  2-arg vs a labeled `reduce(_, with:)` form) in the return report.
- **`List.includes` / `isEmpty`:** `[1,2,3].includes(2)` → `true`,
  `.includes(9)` → `false`; `List.new().isEmpty` → `true`, non-empty → `false`.
- **`List.at(_, put:)`:** set index 1, read it back, assert the new value; assert
  the labeled selector spelling matches `rawSet`'s arity.
- **No-truthiness regression:** every loop/branch condition in the new bodies is
  a real `Bool` — a fixture that accidentally branches on an `Option` must be a
  hard error, not silently pass (guards against reintroducing truthiness).
- **Golden byte-identity:** `core_new.ph` / `person2.ph` / `tests/fixtures/golden/*`
  unchanged (you added methods only).

---

## 7. Mandatory rules (unchanged from `U-STD-plan.md` §8 — repeated for emphasis)

- **Scheduling constraint (make this explicit for the orchestrator):** `core.ph`
  is a **single shared file**. Per PHASE2-INDEX §2/§3 it is edited additively
  along `U6 → U-STD → U11`; **never co-schedule two `core.ph` editors.** U11
  (Bool tower) touches `core.ph` and therefore **MUST NOT run concurrently with
  U-STD — it must wait for U-STD to land first.** **U-LEX is parallel-safe**
  (it touches `phalcom-ast`, not `core.ph`). If the §0.1 ruling is Option (A) or
  keeps any U-CORE-1/2/4 work live, note those also edit `core.ph` and must
  serialize against U-STD too (U-CORE-2 already landed, so only U-CORE-1/4 remain
  a concern).
- **`core.ph`:** every added method carries a spec-referencing `//` comment.
  Methods stay consistent with U3 label-encoded selectors (ADR-0012) and the
  corrected tower; additive, non-clobbering — never rewrite a class another unit
  owns; keep additions in clearly-commented blocks so U11's later edit merges cleanly.
- **Touch no kernel wiring:** no U2 tower, no U6 `Option` substrate (`match`/
  `Some.new`/`None` singleton), no `Bool` subtree (U11), and — under Option (B) —
  no `primitive/*.rs` at all.
- **Green gate:** `./scripts/verify.sh` exits 0 is your sole sign-off — reviewer
  is **OFF** for U-STD (STATE.md policy). Goldens byte-identical, new `lang`
  cases pass, no new clippy/`cargo doc` warnings. Any (unexpected) Rust glue
  ships full rustdoc.
- Run `graphify update . --no-cluster` before every commit; commit per green
  checkpoint, never a non-compiling tree.
- **Hard stop when green:** do not begin U11 / U-LEX / any U-CORE-N unit.

---

## 8. Return contract (answer all of these — mirrors `U-STD-plan.md` §9)

- **State which §0.1 scope option you were told to build** (A or B), and by whom —
  an implementer must not have proceeded on the contested Object/Number/String/
  Symbol surface without a ruling.
- Classes/methods added to `core.ph`, grouped by class (`Option` vs `List`), each
  with the spec § it realizes; confirm all were **pure `.ph` over the existing
  floor** (no new native primitive) — or, if any primitive was unavoidable, name
  it and justify against the ADR-0019 freeze.
- The exact **selector spelling** chosen for `List.reduce` and `List.at(_,put:)`.
- Confirmation you **updated** the `core.ph:71-73` "do not add map/reduce/filter"
  comment, and did **not** add list-literal syntax.
- Confirmation you did **not** chase `printString`, did **not** un-ignore the
  `absence` label, and did **not** touch `system()`/`system_pending()` (§0.3/§0.6).
- Which `pending/` fixture(s) you promoted (DEFERRED.md #25 /
  `blocks_argument_to_method.ph`), and how.
- Explicit confirmation you touched **no kernel wiring (U2), no `Option`
  substrate (U6), no `Bool` subtree (U11), no U-CORE-1/4 surface, and no
  `primitive/*.rs`** (Option B).
- Proof the goldens stayed byte-identical (`verify.sh` tail, `cargo doc` tail).
- New `DEFERRED.md` entries (from #27): collections + `hash` blocker, list
  literals, and the forge-index-vs-`docs/spec/core/` scope-taxonomy divergence.
